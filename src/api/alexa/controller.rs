use crate::storage;
use std::sync::{Arc, RwLock};

use futures::{Future};

use hyper::{Body, Response};

use crate::api::alexa::dto::{GenericCall, GenericResult};
use crate::api::utils::{internal_error_rsp, ok_rsp};

pub struct AlexaController {
    storage: Arc<RwLock<storage::Storage>>
}


impl AlexaController {

    pub fn new(storage:Arc<RwLock<storage::Storage>>) -> AlexaController {
        AlexaController {
            storage: storage
        }
    }


    pub fn create_slap_notification(&self, call: GenericCall) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {

        let for_city_opt = resolve_city(call.clone());

        // validate city
        if for_city_opt.is_none() {
            let result_object = GenericResult::city_not_provided();
            println!("city hasn't been provided");
            return prepare_response(result_object);
        }

        let for_city = for_city_opt.unwrap();
        let event = storage::Event::new_slap();

        self.storage.write().unwrap().add_event(event, for_city.clone());
        let response_object = GenericResult::notification_created(for_city.clone());

        return prepare_response(response_object);
    }

    pub fn deliver_notification(&self, call: GenericCall) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {

        let for_city_opt = resolve_city(call.clone());
        if for_city_opt.is_none() {
            let result_object = GenericResult::city_not_provided();
            println!("Failed notification delivery: city hasn't been provided");
            return prepare_response(result_object);
        }

        let for_city = for_city_opt.unwrap();
        println!("city value is {}", &for_city);

        if !self.storage.read().unwrap().is_registered(&for_city) {
            let response_object = GenericResult::city_unknown();
            return prepare_response(response_object);
        }

        match self.storage.write().unwrap().pop_event(&for_city) {
            Some(event) => {
                let result = GenericResult::for_event(event);
                prepare_response(result)
            },
            None => {
                println!("No notifications found for city: {}", &for_city);
                let result = GenericResult::no_notifications_found_for(&for_city);
                prepare_response(result)
            }
        }

    }

}

fn resolve_city(call: GenericCall) -> Option<String> {
    let city = call.request.intent.slots?.city.resolutions.resolutions_per_authority.first()?.values.first()?.value.name.clone();
    Some(city)
}

fn prepare_response(result: GenericResult) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    match serde_json::to_string(&result) {
        Ok(json) => ok_rsp(json),
        Err(err) => {
            println!("ERROR: failed to serialize response for notification creation: {:?}", err);
            internal_error_rsp()
        }

    }
}
