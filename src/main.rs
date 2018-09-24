#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate hyper;

use hyper::{Body, Request, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;


const PHRASE: &str = "Hello, World!";

fn hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from(PHRASE))
}

fn main() {


    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates on of our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`
        service_fn_ok(hello_world)
    };

    let server = Server::bind(&addr).serve(new_svc).map_err(|e| {
        eprintln!("server error: {}", e)
    });

    // Run this server for... forever!
    hyper::rt::run(server);

}
