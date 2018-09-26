use crate::storage;
use std::sync::{Arc, RwLock};

use hyper::{Body, Request, Response, StatusCode};

use crate::api::alexa::dto::GenericCall;

pub struct AlexaController {
    storage: Arc<RwLock<storage::Storage>>
}


impl AlexaController {

    pub fn new(storage:Arc<RwLock<storage::Storage>>) -> AlexaController {
        AlexaController {
            storage: storage
        }
    }

    pub fn create_slap_notification(&self, for_city: &String, _call: &GenericCall) -> Result<Response<Body>, hyper::Error> {
        println!("create slap notification for the city: {}", &for_city);

        let event = storage::Event {
            from_device: String::from("test"),
            event_type: storage::EventType::SLAP
        };

        self.storage.write().unwrap().add_event(event, for_city.clone());
        Ok(Response::builder()
           .status(StatusCode::NOT_FOUND)
           .body(Body::empty())
           .unwrap())
    }

    pub fn retrieve_notifications(&self, _req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty())
                            .unwrap())
    }

    pub fn no_action_defined(&self, call: &GenericCall) -> Result<Response<Body>, hyper::Error> {
        println!("parsed intent: {} - no action defined", call.request.intent.name);
        Ok(Response::builder()
           .status(StatusCode::NOT_FOUND)
           .body(Body::empty())
           .unwrap())
    }


}
