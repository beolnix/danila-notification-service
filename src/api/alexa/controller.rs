use crate::storage;
use std::sync::{Arc, RwLock};

use hyper::{Body, Request, Response, StatusCode};

use crate::api::alexa::dto::{GenericCall, GenericResult, CountrySlot, Slots};

pub struct AlexaController {
    storage: Arc<RwLock<storage::Storage>>
}


impl AlexaController {

    pub fn new(storage:Arc<RwLock<storage::Storage>>) -> AlexaController {
        AlexaController {
            storage: storage
        }
    }

    fn prepare_response(&self, result: GenericResult) -> Result<Response<Body>, hyper::Error> {
        match serde_json::to_string(&result) {
            Ok(json) => Ok(Response::builder()
                           .status(StatusCode::OK)
                           .body(Body::from(json))
                           .unwrap()),
            Err(err) => {
                println!("ERROR: failed to serialize response for notification creation: {:?}", err);
                Ok(Response::builder()
                   .status(StatusCode::INTERNAL_SERVER_ERROR)
                   .body(Body::empty())
                   .unwrap())
            }

        }
    }

    pub fn create_slap_notification(&self, call: GenericCall) -> Result<Response<Body>, hyper::Error> {

        if call.request.intent.slots.is_none() {
            let result_object = GenericResult::city_not_provided();
            return self.prepare_response(result_object);
        }

        let slots: Slots = call.request.intent.slots.unwrap();
        let for_city: String = slots.country.value;
        let from_city: String = call.context.system.device.device_id;
        println!("create slap notification for the city: {} from the city {}", &for_city, &from_city);

        let event = storage::Event::new_slap_from(from_city);

        self.storage.write().unwrap().add_event(event, for_city.clone());
        let response_object = GenericResult::notification_created(for_city.clone());

        return self.prepare_response(response_object);
    }

    pub fn deliver_notification(&self, call: &GenericCall) -> Result<Response<Body>, hyper::Error> {
        let for_city = &call.context.system.device.device_id;
        match self.storage.write().unwrap().pop_event(for_city) {
            Some(event) => {
                let result = GenericResult::for_event(event);
                self.prepare_response(result)
            },
            None => {
                println!("No notifications found for city: {}", &for_city);
                let result = GenericResult::city_unknown();
                self.prepare_response(result)
            }
        }

    }

    pub fn no_action_defined(&self, call: &GenericCall) -> Result<Response<Body>, hyper::Error> {
        println!("parsed intent: {} - no action defined", call.request.intent.name);
        Ok(Response::builder()
           .status(StatusCode::NOT_FOUND)
           .body(Body::empty())
           .unwrap())
    }

}
