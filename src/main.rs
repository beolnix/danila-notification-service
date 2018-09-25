#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::service_fn;

use futures::{future, Future, Stream};

mod storage;

use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Debug)]
struct DANotificationResponse {
    message_num: i32
}

#[derive(Serialize, Deserialize, Debug)]
struct DANotificationRequest {
    device_name: String
}

fn dispatch(req: Request<Body>, storage: Arc<RwLock<storage::Storage>>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    println!("dispatching uri: {}", req.uri());

    match req.uri().path() {
        "/danila-skill/create-notification" => create_notification(req, storage),
        "/danila-skill/retrieve-notification" => retrieve_notifications(req, storage),
        "/notifications" => process_notificaiton_request(req, storage),
        _ => Box::new(future::ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()))
    }
}

fn retrieve_notifications(_req: Request<Body>, _storage: Arc<RwLock<storage::Storage>>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    Box::new(future::ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()))
}

fn process_notificaiton_request(req: Request<Body>, storage: Arc<RwLock<storage::Storage>>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    let (parts, _body) = req.into_parts();
    let uri = parts.uri;
    let device = str::replace(uri.query().unwrap(), "device=", "");

    println!("DEBUG: received notifications request for device: {}", device);

    let count = storage.read().unwrap().size(device);
    let response = future::ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!("{{ \"count\": {} }}", count)))
                .unwrap());

    return Box::new(response);
}

fn create_notification(req: Request<Body>, storage: Arc<RwLock<storage::Storage>>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    let result = req.into_body()
        .fold(Vec::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            future::ok::<Vec<u8>, hyper::Error>(acc)
        })
        .and_then( move |acc| {
            let str_body = String::from_utf8(acc).unwrap();
            println!("request body: {}", str_body);
            let event = storage::Event {
                from_device: String::from("test"),
                event_type: storage::EventType::SLAP
            };
            storage.write().unwrap().add_event(event, String::from("Berlin"));
            Ok(Response::builder()
               .status(StatusCode::OK)
               .body(Body::from(str_body))
               .unwrap())
        });

    Box::new(result)
}

fn main() {
    let storage = Arc::new(RwLock::new(storage::Storage::new()));

    let new_svc = move || {
        let _storage = storage.clone();
        service_fn( move |req: Request<Body>| {
            dispatch(req, _storage.clone())
        })
    };

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(new_svc).map_err(|e| {
        eprintln!("server error: {}", e)
    });

    // Run this server for... forever!
    hyper::rt::run(server);

}
