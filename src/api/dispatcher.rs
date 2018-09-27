
use hyper::{Body, Request, Response, StatusCode, Method};

use futures::{future, Future, Stream};
use std::sync::{Arc};
use crate::api::rest::controller::RestController;
use crate::api::alexa::controller::AlexaController;
use crate::api::alexa::dto::GenericCall;

pub struct Dispatcher {
    rest_controller: Arc<RestController>,
    alexa_controller: Arc<AlexaController>
}

impl Dispatcher {

    pub fn new(rest_controller: RestController, alexa_controller: AlexaController) -> Dispatcher {
        Dispatcher {
            rest_controller: Arc::new(rest_controller),
            alexa_controller: Arc::new(alexa_controller)
        }
    }

    pub fn dispatch(&self, req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        println!("dispatching uri: {}", req.uri());

        match req.uri().path() {
            "/alexa-skill" => self.dispatch_alexa(req),
            _ => self.dispatch_rest(req)
        }
    }

    fn dispatch_rest(&self, req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        let _rest_controller = self.rest_controller.clone();
        let (parts, body) = req.into_parts();
        let uri = parts.uri;
        let method = parts.method;
        let path = String::from(uri.path());
        let query = match uri.query() {
            Some(query_str) => Some(String::from(query_str)),
            _ => None
        };

        let result = body
            .fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&chunk);
                future::ok::<Vec<u8>, hyper::Error>(acc)
            })
            .and_then( move |acc| {
                let str_body = String::from_utf8(acc).unwrap();
                println!("request body: {}", &str_body);

                let processing_result: Result<Response<Body>, hyper::Error> = match (method, path.as_ref()) {
                    (Method::GET, "/rest-api/status") => {
                        match query {
                            Some(query_params) => {
                                let device = str::replace(&query_params, "city=", "");
                                _rest_controller.get_notifications_for(&device)
                            },
                            _ => {
                                Ok(Response::builder()
                                           .status(StatusCode::BAD_REQUEST)
                                           .body(Body::from("query parameter 'city' is mandatory but hasn't been provided."))
                                           .unwrap())
                            }
                        }
                    },
                    (Method::POST, "rest-api/notifications") => {
                                Ok(Response::builder()
                                           .status(StatusCode::BAD_REQUEST)
                                           .body(Body::from("query parameter 'city' is mandatory but hasn't been provided."))
                                           .unwrap())

                    }
                    _ => {
                        Ok(Response::builder()
                                   .status(StatusCode::NOT_FOUND)
                                   .body(Body::empty())
                                   .unwrap())
                    }
                };

                processing_result
            });

        return Box::new(result);

    }

    fn dispatch_alexa(&self, req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        let _alexa_controller = self.alexa_controller.clone();
        let result = req.into_body()
            .fold(Vec::new(), |mut acc, chunk| {
                acc.extend_from_slice(&chunk);
                future::ok::<Vec<u8>, hyper::Error>(acc)
            })
            .and_then( move |acc| {
                let str_body = String::from_utf8(acc).unwrap();
                println!("request body: {}", &str_body);
                let parse_result = GenericCall::from(&str_body);

                let processing_result: Result<Response<Body>, hyper::Error> = match parse_result {
                    Ok(call) => match call.request.intent.name.as_ref() {
                        "create_slap_notification" => _alexa_controller.create_slap_notification(call),
                        "deliver_notification" => _alexa_controller.deliver_notification(call),
                        _ => _alexa_controller.no_action_defined(&call)
                    },
                    Err(err) => {
                        println!("alexa request deserialisation error: {:?}", err);
                        Ok(Response::builder()
                           .status(StatusCode::INTERNAL_SERVER_ERROR)
                           .body(Body::empty())
                           .unwrap())

                    }
                };
                processing_result

            });

        return Box::new(result);
    }



}




