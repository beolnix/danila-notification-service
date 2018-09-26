
use hyper::{Body, Request, Response, Server, StatusCode};

use futures::{future, Future, Stream};
use std::sync::{Arc, RwLock};
use crate::api::rest::controller::RestController;
use crate::api::alexa::controller::AlexaController;
use crate::api::alexa::dto::GenericCall;

pub struct Dispatcher {
    restController: Arc<RestController>,
    alexaController: Arc<AlexaController>
}

impl Dispatcher {

    pub fn new(restController: RestController, alexaController: AlexaController) -> Dispatcher {
        Dispatcher {
            restController: Arc::new(restController),
            alexaController: Arc::new(alexaController)
        }
    }

    pub fn dispatch(&self, req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        println!("dispatching uri: {}", req.uri());

        match req.uri().path() {
            "/alexa-skill" => self.dispatch_alexa(req),
            "/rest-api/" => self.restController.process_notificaiton_request(req),
            _ => Box::new(future::ok(Response::builder()
                                     .status(StatusCode::NOT_FOUND)
                                     .body(Body::empty())
                                     .unwrap()))
        }
    }

    fn dispatch_alexa(&self, req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        let _alexaController = self.alexaController.clone();
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
                        "create_slap_notification" => _alexaController.create_slap_notification(&call.request.intent.slots.country.value, &call),
                        _ => _alexaController.no_action_defined(&call)
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

                // _alexaController.create_slap_notification(&String::from("Berlin"), &parse_result.unwrap());
                // Ok(Response::builder()
                //    .status(StatusCode::OK)
                //    .body(Body::empty())
                //    .unwrap())
            });

        return Box::new(result);
    }



}




