#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate hyper;

use hyper::{Body, Request, Response, Server, StatusCode, Chunk};
use hyper::service::service_fn;


use futures;
use futures::{future, Future, Stream};

use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use bson::Bson;

#[derive(Serialize, Deserialize, Debug)]
struct DANotificationResponse {
    message_num: i32
}

#[derive(Serialize, Deserialize, Debug)]
struct DANotificationRequest {
    device_name: String
}

fn dispatch(_req: Request<Body>, client: std::sync::Arc<mongodb::ClientInner>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    println!("dispatching uri: {}", _req.uri());

    if _req.uri() == "/danila-skill/create-notification" {
        return create_notification(_req, client);
    } else {
        return process_notificaiton_request(_req, client);
    }
}

fn process_notificaiton_request(_req: Request<Body>, client: std::sync::Arc<mongodb::ClientInner>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    let (parts, _body) = _req.into_parts();
    let uri = parts.uri;
    let device = str::replace(uri.query().unwrap(), "device=", "");

    println!("DEBUG: received notifications request for device: {}", device);

    let coll = client.db("danila_app").collection("notifications");

    let doc = doc! {
        "device": device
    };

    let mut cursor = coll.find(Some(doc.clone()), None)
        .ok().expect("failed to find notifications for device");

    let item = cursor.next();

    let result = match item {
        Some(ref test_doc) => match test_doc {
            Ok(ref found_doc) => match found_doc.get("count") {
                Some(&Bson::String(ref count)) => Some(count),
                _ => None
            },
            _ => None
        },
        _ => None
    };

    let response = match result {
        Some(count) => futures::future::ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!("{{ \"count\": {} }}", count)))
                .unwrap()),
        None => futures::future::ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("not found"))
                .unwrap())

    };

    return Box::new(response);
}

fn create_notification(_req: Request<Body>, _client: std::sync::Arc<mongodb::ClientInner>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {

//    let (parts, body) = _req.into_parts();
//    let raw_body = serde_json::from_slice(&body);

    let body = Body::wrap_stream(_req.into_body().map(|chunk| {
        let the_body = chunk.iter().cloned().collect::<Vec<u8>>();
                Chunk::from(the_body)
            }));

    Box::new(future::ok(Response::new(body)))

}

fn main() {

    let client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone client.");


    let new_svc = move || {
        let _client = client.clone();
        service_fn( move |req: Request<Body>| {
            dispatch(req, _client.clone())
        })
    };

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(new_svc).map_err(|e| {
        eprintln!("server error: {}", e)
    });

    // Run this server for... forever!
    hyper::rt::run(server);

}
