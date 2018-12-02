use crate::futures::future::ok;
use crate::futures::Future;

use hyper::{Body, Response, StatusCode};

pub fn ok_rsp(json: String) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    Box::new(ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(json))
        .unwrap()))
}

pub fn created_rsp() -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    Box::new(ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::empty())
        .unwrap()))
}

pub fn bad_request_rsp(
    msg: String,
) -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    Box::new(ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(msg))
        .unwrap()))
}

pub fn internal_error_rsp() -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    Box::new(ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap()))
}

pub fn not_found_rsp() -> Box<Future<Item = Response<Body>, Error = hyper::Error> + Send> {
    Box::new(ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()))
}
