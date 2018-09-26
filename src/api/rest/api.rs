

pub struct RestController {
    storage: Arc<RwLock<storage::Storage>>
}

impl RestController {
    fn process_notificaiton_request(req: Request<Body>) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
        let (parts, _body) = req.into_parts();
        let uri = parts.uri;
        let device = str::replace(uri.query().unwrap(), "device=", "");

        println!("DEBUG: received notifications request for device: {}", device);

        let count = storage.read().unwrap().size(&device);
        let response = future::ok(Response::builder()
                                  .status(StatusCode::OK)
                                  .body(Body::from(format!("{{ \"count\": {} }}", count)))
                                  .unwrap());

        return Box::new(response);
    }
}
