use crate::storage;
use std::sync::{Arc, RwLock};

use crate::futures::{future, Future};
use crate::api::rest::dto::StatusResponse;

use hyper::{Body, Request, Response, StatusCode};

pub struct RestController {
    storage: Arc<RwLock<storage::Storage>>
}

impl RestController {

    pub fn new(storage:Arc<RwLock<storage::Storage>>) -> RestController {
        RestController {
            storage: storage
        }
    }

    pub fn get_notifications_for(&self, device: &String) -> Result<Response<Body>, hyper::Error> {
        println!("DEBUG: received GET notifications request for device: {}", device);

        let count = self.storage.read().unwrap().size(&device);

        return Ok(self.prepare_response(count));
    }

    fn prepare_response(&self, num: usize) -> Response<Body> {
        let response_object = StatusResponse::new(num);
        match serde_json::to_string(&response_object) {
            Ok(json) => Response::builder()
                           .status(StatusCode::OK)
                           .body(Body::from(json))
                           .unwrap(),
            Err(err) => {
                println!("ERROR: failed to serialize response for notification creation: {:?}", err);
                Response::builder()
                   .status(StatusCode::INTERNAL_SERVER_ERROR)
                   .body(Body::empty())
                   .unwrap()
            }

        }
    }

}
