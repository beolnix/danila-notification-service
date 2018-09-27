use crate::storage;
use std::sync::{Arc, RwLock};

use crate::futures::{future, Future};
use crate::api::rest::dto::{StatusResponse, CreateNotificationReqeust};

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

    pub fn create_notification(&self, req: CreateNotificationReqeust) -> Result<Response<Body>, hyper::Error> {
        let event_type = req.type_name.clone();
        let for_city = req.for_city;
        let valid_city = self.storage.read().unwrap().is_registered(&for_city);

        if !valid_city {
            let supported_cities = self.storage.read().unwrap().get_supported_cities_as_str();
            return Ok(Response::builder()
                      .status(StatusCode::BAD_REQUEST)
                      .body(Body::from(format!("The city {} is not supported. Supported cities are: {}.", &for_city, &supported_cities)))
                      .unwrap());
        }

        match event_type.as_ref() {
            "SLAP" => {
                let event = storage::Event::new_slap();
                self.storage.write().unwrap().add_event(event, for_city);
                Ok(Response::builder()
                   .status(StatusCode::CREATED)
                   .body(Body::empty())
                   .unwrap())
            },
            "MESSAGE" => {
                match req.message_text {
                    Some(text) => {
                        let event = storage::Event::new_message(text);
                        self.storage.write().unwrap().add_event(event, for_city);

                        Ok(Response::builder()
                           .status(StatusCode::CREATED)
                           .body(Body::empty())
                           .unwrap())
                    },
                    None => Ok(Response::builder()
                               .status(StatusCode::BAD_REQUEST)
                               .body(Body::from("message_text property must not be missed if notification type is MESSAGE."))
                               .unwrap())
                }
            },
            _ => Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("The event type '{}' is not supported. Supported types are: SLAP, MESSAGE.", &event_type)))
                    .unwrap())
        }
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
