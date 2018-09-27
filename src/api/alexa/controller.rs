use crate::storage;
use std::sync::{Arc, RwLock};

use hyper::{Body, Request, Response, StatusCode};

use crate::api::alexa::dto::{GenericCall, GenericResult, CitySlot, Slots};

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

        let for_city_opt = self.resolve_city(call.clone());
        if for_city_opt.is_none() {
            let result_object = GenericResult::city_not_provided();
            println!("city hasn't been provided");
            return self.prepare_response(result_object);
        }

        let for_city = for_city_opt.unwrap();
        let event = storage::Event::new_slap();

        self.storage.write().unwrap().add_event(event, for_city.clone());
        let response_object = GenericResult::notification_created(for_city.clone());

        return self.prepare_response(response_object);
    }

    pub fn deliver_notification(&self, call: GenericCall) -> Result<Response<Body>, hyper::Error> {

        let for_city_opt = self.resolve_city(call.clone());
        if for_city_opt.is_none() {
            let result_object = GenericResult::city_not_provided();
            println!("city hasn't been provided");
            return self.prepare_response(result_object);
        }

        let for_city = for_city_opt.unwrap();

        println!("city value is {}", &for_city);

        if !self.storage.read().unwrap().is_registered(&for_city) {
            let response_object = GenericResult::city_unknown();
            return self.prepare_response(response_object);
        }

        match self.storage.write().unwrap().pop_event(&for_city) {
            Some(event) => {
                let result = GenericResult::for_event(event);
                self.prepare_response(result)
            },
            None => {
                println!("No notifications found for city: {}", &for_city);
                let result = GenericResult::no_notifications_found_for(&for_city);
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

    fn resolve_city(&self, call: GenericCall) -> Option<String> {
        let city = call.request.intent.slots?.city.resolutions.resolutionsPerAuthority.first()?.values.first()?.value.name.clone();
        Some(city)
    }

}
