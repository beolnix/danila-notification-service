use hyper::{Body, Method, Request, Response};

use crate::api::alexa::controller::AlexaController;
use crate::api::alexa::dto::GenericCall;
use crate::api::rest::controller::RestController;
use crate::api::rest::dto::CreateNotificationReqeust;
use crate::api::utils::{bad_request_rsp, internal_error_rsp, not_found_rsp};
use futures::future::ok;
use futures::{future, Future, Stream};
use std::sync::Arc;

pub struct DeconstructedRequest {
    pub method: hyper::Method,
    pub path: String,
    pub query: Option<String>,
    pub body: Box<Future<Item = String, Error = hyper::Error> + Send>,
}

pub struct Dispatcher {
    rest_controller: Arc<RestController>,
    alexa_controller: Arc<AlexaController>,
}

impl Dispatcher {
    pub fn new(rest_controller: RestController, alexa_controller: AlexaController) -> Dispatcher {
        Dispatcher {
            rest_controller: Arc::new(rest_controller),
            alexa_controller: Arc::new(alexa_controller),
        }
    }

    pub fn dispatch(
        &self,
        req: Request<Body>,
    ) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
        println!("dispatching uri: {}", req.uri());
        let d_request = DeconstructedRequest::from(req);

        match d_request.path.as_ref() {
            "/alexa-skill" => self.dispatch_alexa(d_request),
            _ => self.dispatch_rest(d_request),
        }
    }

    fn dispatch_rest(
        &self,
        req: DeconstructedRequest,
    ) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
        let _rest_controller = self.rest_controller.clone();

        let body = req.body;
        let path = req.path;
        let query = req.query;
        let method = req.method;

        let result = body.and_then(move |str_body| {
            println!("request body: {}", &str_body);

            match (method, path.as_ref()) {
                (Method::GET, "/rest-api/status") => match query {
                    Some(query_params) => {
                        let city = str::replace(&query_params, "city=", "");
                        _rest_controller.get_notifications_for(&city)
                    }
                    _ => bad_request_rsp(String::from(
                        "query parameter 'city' is mandatory but hasn't been provided.",
                    )),
                },
                (Method::POST, "/rest-api/notifications") => {
                    let request_object: Result<CreateNotificationReqeust, serde_json::Error> =
                        serde_json::from_str(&str_body);
                    match request_object {
                        Ok(object) => _rest_controller.create_notification(object),
                        _ => bad_request_rsp(String::from("cannot deserialize body.")),
                    }
                }
                _ => not_found_rsp(),
            }
        });

        return Box::new(result);
    }

    fn dispatch_alexa(
        &self,
        req: DeconstructedRequest,
    ) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
        let _alexa_controller = self.alexa_controller.clone();

        let result = req.body.and_then(move |str_body| {
            println!("request body: {}", &str_body);
            let parsed_result = GenericCall::from(&str_body);

            match parsed_result {
                Ok(call) => match call.request.intent.name.as_ref() {
                    "create_slap_notification" => _alexa_controller.create_slap_notification(call),
                    "deliver_notification" => _alexa_controller.deliver_notification(call),
                    _ => not_found_rsp(),
                },
                Err(err) => {
                    println!("alexa request deserialisation error: {:?}", err);
                    internal_error_rsp()
                }
            }
        });

        return Box::new(result);
    }
}

impl DeconstructedRequest {
    pub fn new(
        method: hyper::Method,
        path: String,
        query: Option<String>,
        body: Box<Future<Item = String, Error = hyper::Error> + Send>,
    ) -> DeconstructedRequest {
        DeconstructedRequest {
            method: method,
            path: path,
            query: query,
            body: body,
        }
    }

    pub fn from(req: Request<Body>) -> DeconstructedRequest {
        let (parts, body) = req.into_parts();
        let uri = parts.uri;
        let method = parts.method;
        let path = String::from(uri.path());
        let query = match uri.query() {
            Some(query_str) => Some(String::from(query_str)),
            _ => None,
        };
        let raw_body = body
            .fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&chunk);
                future::ok::<Vec<u8>, hyper::Error>(acc)
            })
            .and_then(move |acc| ok(String::from_utf8(acc).unwrap()));

        let result_body = Box::new(raw_body);

        return DeconstructedRequest::new(method, path, query, result_body);
    }
}
