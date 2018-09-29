use crate::storage;

use std::sync::{Arc, RwLock};
use crate::futures::Future;

use crate::api::rest::dto::{StatusResponse, CreateNotificationReqeust};
use crate::api::utils::{bad_request_rsp, created_rsp, internal_error_rsp, ok_rsp};

use hyper::{Body, Response};



pub struct RestController {
    storage: Arc<RwLock<storage::Storage>>
}

impl RestController {

    pub fn new(storage:Arc<RwLock<storage::Storage>>) -> RestController {
        RestController {
            storage: storage
        }
    }

    pub fn get_notifications_for(&self, device: &String) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        println!("DEBUG: received GET notifications request for device: {}", device);

        let count = self.storage.read().unwrap().size(&device);

        return prepare_response(count);
    }

    pub fn create_notification(&self, req: CreateNotificationReqeust) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        let event_type = req.type_name.clone();
        let for_city = req.for_city;
        let valid_city = self.storage.read().unwrap().is_registered(&for_city);

        // validate parameter value
        if !valid_city {
            let supported_cities = self.storage.read().unwrap().get_supported_cities_as_str();
            return bad_request_rsp(format!("The city {} is not supported. Supported cities are: {}.", &for_city, &supported_cities));
        }

        // process valid creation request
        match event_type.as_ref() {
            "SLAP" => self.create_slap_msg(for_city),
            "MESSAGE" => {
                match req.message_text {
                    Some(text) => self.create_text_msg(for_city, text),
                    None => bad_request_rsp(String::from("message_text property must not be missed if notification type is MESSAGE."))
                }
            },
            _ => bad_request_rsp(format!("The event type '{}' is not supported. Supported types are: SLAP, MESSAGE.", &event_type))
        }
    }

    fn create_text_msg(&self, for_city: String, text: String) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        let event = storage::Event::new_message(text);
        self.storage.write().unwrap().add_event(event, for_city);
        created_rsp()
    }

    fn create_slap_msg(&self, for_city: String) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        let event = storage::Event::new_slap();
        self.storage.write().unwrap().add_event(event, for_city);
        created_rsp()
    }

}

fn prepare_response(num: usize) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    let response_object = StatusResponse::new(num);
    match serde_json::to_string(&response_object) {
        Ok(json) => ok_rsp(json),
        Err(err) => {
            println!("ERROR: failed to serialize response for notification creation: {:?}", err);
            internal_error_rsp()
        }

    }
}
