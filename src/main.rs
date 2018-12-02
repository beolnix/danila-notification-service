#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;

mod api;
mod storage;

use futures::{future, Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::sync::{Arc, RwLock};

use crate::api::rest::dto::StatusResponse;

fn create_dispatcher(storage: Arc<RwLock<storage::Storage>>) -> api::dispatcher::Dispatcher {
    let alexa_controller = api::alexa::controller::AlexaController::new(storage.clone());
    let rest_controller = api::rest::controller::RestController::new(storage.clone());

    api::dispatcher::Dispatcher::new(rest_controller, alexa_controller)
}

fn main() {
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = Arc::new(create_dispatcher(storage.clone()));

    let new_svc = move || {
        let _dispatcher = dispatcher.clone();
        let _storage = storage.clone();
        service_fn(move |req: Request<Body>| _dispatcher.dispatch(req))
    };

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    // Run this server for... forever!
    hyper::rt::run(server);
}

// ------------- there are tests only below this point ------------

#[test]
fn smoke_test_slap_rest_creation() {
    // given
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = create_dispatcher(storage.clone());
    let for_city = String::from("BERLIN");

    // when
    let req = build_request_for_slap_notification_creation(for_city.clone());

    // then
    let response = dispatcher.dispatch(req).wait().unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let berlin_queue_size = storage.read().unwrap().size(&for_city);
    assert_eq!(berlin_queue_size, 1);
}

#[test]
fn smoke_test_message_rest_creation() {
    // given
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = create_dispatcher(storage.clone());
    let for_city = String::from("BERLIN");
    let message = String::from("test message text");

    // when
    let req = build_request_for_message_notification_creation(for_city.clone(), message.clone());

    // then
    let response = dispatcher.dispatch(req).wait().unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let berlin_queue_size = storage.clone().read().unwrap().size(&for_city);
    assert_eq!(berlin_queue_size, 1);

    match storage.clone().write().unwrap().pop_event(&for_city) {
        Some(event) => {
            assert!(event.message.clone().is_some());
            assert_eq!(message, event.message.unwrap());
        }
        None => {
            panic!();
        }
    }
}

#[test]
fn smoke_test_get_notifications() {
    // given
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = create_dispatcher(storage.clone());
    let for_city = String::from("BERLIN");

    let event = storage::Event::new_slap();
    storage
        .write()
        .unwrap()
        .add_event(event.clone(), for_city.clone());

    // when
    let req = build_request_for_get_notifications(for_city.clone());

    // then
    let response = dispatcher.dispatch(req).wait().unwrap();
    let rsp_body = consume_body(response);
    println!("received response: {}", rsp_body);
    let response_object: StatusResponse = serde_json::from_str(&rsp_body).unwrap();

    assert_eq!(response_object.message_num, 1);
}

#[test]
fn smoke_test_create_slap_notification() {
    // given
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = create_dispatcher(storage.clone());
    let city = String::from("BERLIN");

    // when
    create_notification_for(&city, &dispatcher);

    // then
    let berlin_queue_size = storage.read().unwrap().size(&String::from("BERLIN"));
    assert_eq!(berlin_queue_size, 1);
}

#[test]
fn smoke_test_create_and_retrieve_slap() {
    // given
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = create_dispatcher(storage.clone());
    let city = String::from("BERLIN");

    // STEP 1: create notification for Berlin
    create_notification_for(&city, &dispatcher);

    // STEP 1: verify notification created
    let berlin_queue_size = storage.read().unwrap().size(&city);
    assert_eq!(berlin_queue_size, 1);

    // STEP 2: deliver notification for Berlin
    deliver_notification_for(&city, &dispatcher);

    // STEP 2: verify notification delinvered
    let berlin_queue_size = storage.read().unwrap().size(&city);
    assert_eq!(berlin_queue_size, 0);
}

fn deliver_notification_for(city: &String, dispatcher: &api::dispatcher::Dispatcher) {
    let raw_body = r###"{"version":"1.0","session":{"new":true,"sessionId":"amzn1.echo-api.session.cc4447e1-2363-4067-a557-8c5c8a04f4e5","application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"}},"context":{"System":{"application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"},"device":{"deviceId":"amzn1.ask.device.AFBBPRUJRVKP4BAHNQW4BS6FJZP32LOYQO2AYRVRMCKP7D3U5BHCS35VMMAPWMZEHJMDZTQJ5Z7EMJDRWXCADDHYR4OOCL7BTJ44MIZB2EFMCE2WM7DZ4QJDFMVNKAIXQ7OPW6UJDJGCJBKSE2IUOIPRJASFASF7CYBLYIMA725YQFMRGJPBO","supportedInterfaces":{}},"apiEndpoint":"https://api.amazonalexa.com","apiAccessToken":"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6IjEifQ.eyJhdWQiOiJodHRwczovL2FwaS5hbWF6b25hbGV4YS5jb20iLCJpc3MiOiJBbGV4YVNraWxsS2l0Iiwic3ViIjoiYW16bjEuYXNrLnNraWxsLjlmNGVmMWRkLWNlZTktNDBlNS1iMDFkLTMwYjlmNGVjY2U3ZiIsImV4cCI6MTUzODA0Njc3NCwiaWF0IjoxNTM4MDQzMTc0LCJuYmYiOjE1MzgwNDMxNzQsInByaXZhdGVDbGFpbXMiOnsiY29uc2VudFRva2VuIjpudWxsLCJkZXZpY2VJZCI6ImFtem4xLmFzay5kZXZpY2UuQUZCQlBSVUpSVktQNEJBSE5RVzRCUzZGSlpQMzJMT1lRTzJBWVJWUk1DS1A3RDNVNUJIQ1MzNVZNTUFQV01aRUhKTURaVFFKNVo3RU1KRFJXWENBRERIWVI0T09DTDdCVEo0NE1JWkIyRUZNQ0UyV003RFo0UUpERk1WTktBSVhRN09QVzZVSkRKR0NKQktTRTJJVU9JUFJKQVNGQVNGN0NZQkxZSU1BNzI1WVFGTVJHSlBCTyIsInVzZXJJZCI6ImFtem4xLmFzay5hY2NvdW50LkFHV0tQRzNKTTRaMzY0QVlLS1NBR0hLTDZDWVdNSktPQVpHWEc1Q1BYWVgyWTdVS1daVEg2WEVMRldQSUNCQ1daUDdPRjVWRUJTUVRRNFVNQ1ZFN0VWUldOMlBVS0JMTUpHVTNHRDIySFpTUlZVNlRURE1VTjJQSjVNN1RXS0FRT1Q3VkJGS1pKTEJJQ0szV1ZJWE9HREY3WUhYVFdXV0tDNzVEMk9OU0w0Sk9MUlVGRlkySktFQVA1VTQ0VENMSkpCUURERkpNRkdVRzVXWSJ9fQ.Atpu3ZcEb3T96hJ80Bv8crmbqNdMn_gHAwd8IpD_6HfblYxlEqSSulnfBpKfX4rY2t4Xup4b_XITTYYEty-sKn0cWACOzh0q3LXo2TkA-mXLjr2Px5w6C-9EHxXlW5k8Wjeg1li2A-zAD-0YAFmNRxiSwQFtKOX7r5kgC8GUJluJPoAjYHje4YsC3n6-Vgv0hpx6-x5OFIXY1RDuIFyOEY69GtE57vDlTgSclTSQ-xovddOYinAkcKPBV7c-hOzq4hjWlduGt7J2MPuA1Gjwv0G_skFfpPymsokI2pGZylTOWoilfonu-QU768vvNUwtgwZAapoyeZkUlaySfwtxuA"}},"request":{"type":"IntentRequest","requestId":"amzn1.echo-api.request.e4cc1710-ee0c-4c13-83c6-22ebe882d64c","timestamp":"2018-09-27T10:12:54Z","locale":"en-US","intent":{"name":"deliver_notification","confirmationStatus":"NONE","slots":{"city":{"name":"city","value":"Berlin","resolutions":{"resolutionsPerAuthority":[{"authority":"amzn1.er-authority.echo-sdk.amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f.city","status":{"code":"ER_SUCCESS_MATCH"},"values":[{"value":{"name":"BERLIN","id":"0"}}]}]},"confirmationStatus":"NONE"}}}}}"###;

    let raw_body_from_city = raw_body.replace("city_example", &city);

    let req = build_request_for_skill_api(raw_body_from_city);

    dispatcher.dispatch(req).wait().unwrap();
}

fn create_notification_for(city: &String, dispatcher: &api::dispatcher::Dispatcher) {
    let raw_body = r###"{"version":"1.0","session":{"new":true,"sessionId":"amzn1.echo-api.session.c9add14f-1b3d-40ad-a7e3-f2452e3c2f47","application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"}},"context":{"System":{"application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"},"device":{"deviceId":"amzn1.ask.device.AFBBPRUJRVKP4BAHNQW4BS6FJZP32LOYQO2AYRVRMCKP7D3U5BHCS35VMMAPWMZEHJMDZTQJ5Z7EMJDRWXCADDHYR4OOCL7BTJ44MIZB2EFMCE2WM7DZ4QJDFMVNKAIXQ7OPW6UJDJGCJBKSE2IUOIPRJASFASF7CYBLYIMA725YQFMRGJPBO","supportedInterfaces":{}},"apiEndpoint":"https://api.amazonalexa.com","apiAccessToken":"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6IjEifQ.eyJhdWQiOiJodHRwczovL2FwaS5hbWF6b25hbGV4YS5jb20iLCJpc3MiOiJBbGV4YVNraWxsS2l0Iiwic3ViIjoiYW16bjEuYXNrLnNraWxsLjlmNGVmMWRkLWNlZTktNDBlNS1iMDFkLTMwYjlmNGVjY2U3ZiIsImV4cCI6MTUzODA0NjgzOCwiaWF0IjoxNTM4MDQzMjM4LCJuYmYiOjE1MzgwNDMyMzgsInByaXZhdGVDbGFpbXMiOnsiY29uc2VudFRva2VuIjpudWxsLCJkZXZpY2VJZCI6ImFtem4xLmFzay5kZXZpY2UuQUZCQlBSVUpSVktQNEJBSE5RVzRCUzZGSlpQMzJMT1lRTzJBWVJWUk1DS1A3RDNVNUJIQ1MzNVZNTUFQV01aRUhKTURaVFFKNVo3RU1KRFJXWENBRERIWVI0T09DTDdCVEo0NE1JWkIyRUZNQ0UyV003RFo0UUpERk1WTktBSVhRN09QVzZVSkRKR0NKQktTRTJJVU9JUFJKQVNGQVNGN0NZQkxZSU1BNzI1WVFGTVJHSlBCTyIsInVzZXJJZCI6ImFtem4xLmFzay5hY2NvdW50LkFHV0tQRzNKTTRaMzY0QVlLS1NBR0hLTDZDWVdNSktPQVpHWEc1Q1BYWVgyWTdVS1daVEg2WEVMRldQSUNCQ1daUDdPRjVWRUJTUVRRNFVNQ1ZFN0VWUldOMlBVS0JMTUpHVTNHRDIySFpTUlZVNlRURE1VTjJQSjVNN1RXS0FRT1Q3VkJGS1pKTEJJQ0szV1ZJWE9HREY3WUhYVFdXV0tDNzVEMk9OU0w0Sk9MUlVGRlkySktFQVA1VTQ0VENMSkpCUURERkpNRkdVRzVXWSJ9fQ.B5Y7wjEtxv6sH8lOaaf-jVps5yulE-EwpT84GESxd7WjPBfS7iJIjnmkmKatPpbfxRfwte_HerIW0sLKiJ2S9LJI_mg1_9t_iTiymW-ecacwHOjQeAKYRGXBhHfv41D1j_3gVouNe7cNUK8eckUDm5_o_1AjIaDLhqc9FJiNaphBYlJeyB2Mc_NjpKvFgtnS7yqcRiqESA_6imOZwHyVDS02Iq_3H2qvow9ZLfi09QTOjK3AVBkWtdif14ZD89d-jUuGVXZsvxCxB09sRoOkAQ--AZC1t2mm_AWxWsyLhfRinY6nJh4Y5RMfssBYZPfHD_HT8-aM8NsZ4p0r5SnGag"}},"request":{"type":"IntentRequest","requestId":"amzn1.echo-api.request.1fd8560b-185f-493e-b944-d2d860064e86","timestamp":"2018-09-27T10:13:58Z","locale":"en-US","intent":{"name":"create_slap_notification","confirmationStatus":"NONE","slots":{"city":{"name":"city","value":"Berlin","resolutions":{"resolutionsPerAuthority":[{"authority":"amzn1.er-authority.echo-sdk.amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f.city","status":{"code":"ER_SUCCESS_MATCH"},"values":[{"value":{"name":"BERLIN","id":"0"}}]}]},"confirmationStatus":"NONE"}}}}}"###;

    let raw_body_with_city = raw_body.replace("city_example", &city);

    let req = build_request_for_skill_api(raw_body_with_city);

    dispatcher.dispatch(req).wait().unwrap();
}

fn build_request_for_get_notifications(city: String) -> Request<Body> {
    Request::builder()
        .method(Method::GET)
        .uri(format!(
            "https://auto1.danila.app/rest-api/status?city={}",
            &city
        ))
        .body(Body::empty())
        .unwrap()
}

fn build_request_for_skill_api(body: String) -> Request<Body> {
    Request::builder()
        .uri("https://auto1.danila.app/alexa-skill")
        .body(Body::from(body))
        .unwrap()
}

fn build_request_for_slap_notification_creation(for_city: String) -> Request<Body> {
    let request_obj = api::rest::dto::CreateNotificationReqeust {
        type_name: String::from("SLAP"),
        for_city: for_city,
        message_text: None,
    };
    let json = serde_json::to_string(&request_obj).unwrap();

    Request::builder()
        .uri("https://auto1.danila.app/rest-api/notifications")
        .method(Method::POST)
        .body(Body::from(json))
        .unwrap()
}

fn build_request_for_message_notification_creation(
    for_city: String,
    message: String,
) -> Request<Body> {
    let request_obj = api::rest::dto::CreateNotificationReqeust {
        type_name: String::from("MESSAGE"),
        for_city: for_city,
        message_text: Some(message),
    };
    let json = serde_json::to_string(&request_obj).unwrap();

    Request::builder()
        .uri("https://auto1.danila.app/rest-api/notifications")
        .method(Method::POST)
        .body(Body::from(json))
        .unwrap()
}

fn consume_body(rsp: Response<Body>) -> String {
    let result = rsp
        .into_body()
        .fold(Vec::new(), |mut acc, chunk| {
            acc.extend_from_slice(&chunk);
            future::ok::<Vec<u8>, hyper::Error>(acc)
        })
        .and_then(move |acc| {
            let str_body = String::from_utf8(acc).unwrap();
            Ok(str_body)
        })
        .wait();

    result.unwrap()
}
