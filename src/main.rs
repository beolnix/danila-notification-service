#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate hyper;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::rt::Future;
use hyper::service::service_fn_ok;


const PHRASE: &str = "Hello, World!";

#[derive(Serialize, Deserialize, Debug)]
struct DANotificationResponse {
    message_num: i32
}

fn dispatch(_req: Request<Body>) -> Response<Body> {
    if _req.uri() != "/danila-skill" {
        return processNotificaitonRequest(_req);
    } else {
        return processAlexaSkillRequest(_req);
    }
}

fn processNotificaitonRequest(_req: Request<Body>) -> Response<Body> {
    let notRsp = DANotificationResponse{ message_num: 1 };
    let jsonRsp = serde_json::to_string(&notRsp).unwrap();

//    let response = Response::new(Body::from(jsonRsp));

    let errorDescription = "not found";
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(errorDescription)).unwrap();

    return response;
}

fn processAlexaSkillRequest(_req: Request<Body>) -> Response<Body> {
    return Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Body::empty())
        .unwrap()
}

fn main() {

    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates on of our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`
        service_fn_ok(dispatch)
    };

    let server = Server::bind(&addr).serve(new_svc).map_err(|e| {
        eprintln!("server error: {}", e)
    });

    // Run this server for... forever!
    hyper::rt::run(server);

}
