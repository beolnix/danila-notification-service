#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate hyper;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

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

fn dispatch(_req: Request<Body>, client: std::sync::Arc<mongodb::ClientInner>) -> Response<Body> {
    if _req.uri() == "/danila-skill" {
        return process_alexa_skill_request(_req, client);
    } else {
        return process_notificaiton_request(_req, client);
    }
}

fn process_notificaiton_request(_req: Request<Body>, client: std::sync::Arc<mongodb::ClientInner>) -> Response<Body> {
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

    let response: Response<Body> = match result {
        Some(count) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(format!("{{ \"count\": {} }}", count)))
                .unwrap(),
        None => {
            let error_description = "not found";
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(error_description)).unwrap();
        }
    };

    return response;
}

fn process_alexa_skill_request(_req: Request<Body>, _client: std::sync::Arc<mongodb::ClientInner>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Body::empty())
        .unwrap()
}

fn main() {

    let client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone client.");


    let new_svc = move || {
        let _client = client.clone();
        service_fn_ok( move |req: Request<Body>| {
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
