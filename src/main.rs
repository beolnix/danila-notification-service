#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate hyper;
extern crate futures;

mod api;
mod storage;

use std::sync::{Arc, RwLock};
use futures::{Future};
use hyper::{Body, Request, Server};
use hyper::service::service_fn;

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
        service_fn( move |req: Request<Body>| {
            _dispatcher.dispatch(req)
        })
    };

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(new_svc).map_err(|e| {
        eprintln!("server error: {}", e)
    });

    // Run this server for... forever!
    hyper::rt::run(server);

}

#[test]
fn smoke_test_create_slap_notification() {
    // given
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = create_dispatcher(storage.clone());
    let city = String::from("Berlin");

    // when
    create_notification_for(&city, &dispatcher);

    // then
    let berlin_queue_size = storage.read().unwrap().size(&String::from("Berlin"));
    assert_eq!(berlin_queue_size, 1);

}

#[test]
fn smoke_test_create_and_retrieve_slap() {
    // given
    let storage = Arc::new(RwLock::new(storage::Storage::new()));
    let dispatcher = create_dispatcher(storage.clone());
    let city = String::from("Berlin");

    // STEP 1: create notification for Berlin
    create_notification_for(&city, &dispatcher);

    // STEP 1: verify notification created
    let berlin_queue_size = storage.read().unwrap().size(&city);
    assert_eq!(berlin_queue_size, 1);

    // STEP 2: deliver notification for Berlin
    deliver_notification_for(&city, &dispatcher);

    // STEP 1: verify notification delinvered
    let berlin_queue_size = storage.read().unwrap().size(&city);
    assert_eq!(berlin_queue_size, 0);
}

fn deliver_notification_for(city: &String, dispatcher: &api::dispatcher::Dispatcher) {

    let raw_body = r###"{"version":"1.0","session":{"new":true,"sessionId":"amzn1.echo-api.session.7b8b523e-10fe-4f07-aaaa-69da946d27ab","application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"}},"context":{"System":{"application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"},"device":{"deviceId":"berlin","supportedInterfaces":{}},"apiEndpoint":"https://api.amazonalexa.com","apiAccessToken":"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6IjEifQ.eyJhdWQiOiJodHRwczovL2FwaS5hbWF6b25hbGV4YS5jb20iLCJpc3MiOiJBbGV4YVNraWxsS2l0Iiwic3ViIjoiYW16bjEuYXNrLnNraWxsLjlmNGVmMWRkLWNlZTktNDBlNS1iMDFkLTMwYjlmNGVjY2U3ZiIsImV4cCI6MTUzNzk3NDc1MCwiaWF0IjoxNTM3OTcxMTUwLCJuYmYiOjE1Mzc5NzExNTAsInByaXZhdGVDbGFpbXMiOnsiY29uc2VudFRva2VuIjpudWxsLCJkZXZpY2VJZCI6ImFtem4xLmFzay5kZXZpY2UuQUZCQlBSVUpSVktQNEJBSE5RVzRCUzZGSlpQMzJMT1lRTzJBWVJWUk1DS1A3RDNVNUJIQ1MzNVZNTUFQV01aRUhKTURaVFFKNVo3RU1KRFJXWENBRERIWVI0T09DTDdCVEo0NE1JWkIyRUZNQ0UyV003RFo0UUpERk1WTktBSVhRN09QVzZVSkRKR0NKQktTRTJJVU9JUFJKQVNGQVNGN0NZQkxZSU1BNzI1WVFGTVJHSlBCTyIsInVzZXJJZCI6ImFtem4xLmFzay5hY2NvdW50LkFHV0tQRzNKTTRaMzY0QVlLS1NBR0hLTDZDWVdNSktPQVpHWEc1Q1BYWVgyWTdVS1daVEg2WEVMRldQSUNCQ1daUDdPRjVWRUJTUVRRNFVNQ1ZFN0VWUldOMlBVS0JMTUpHVTNHRDIySFpTUlZVNlRURE1VTjJQSjVNN1RXS0FRT1Q3VkJGS1pKTEJJQ0szV1ZJWE9HREY3WUhYVFdXV0tDNzVEMk9OU0w0Sk9MUlVGRlkySktFQVA1VTQ0VENMSkpCUURERkpNRkdVRzVXWSJ9fQ.RTThUI8CAX8S-4R8gLuRRkdQ6s35aOvBLnK51KQH5Axyepi4rp-LZF-f76BgwBFb2VRorIpiyiJS7yZBqQSpV0MoyOMK_eJUh1i-d6aWjpqF5zm5M3P-ytahOeSOliVI89VxhooqW5iMutBvQAM-_sz0HC6RNYbLyZY-vO40N0jm1RtR_qwvACNz4VMIHFgGikDQcHxmD-VSOaTOl1Wflva3tQV1O5LVSReV-fCk6fNHaS6EZH3XjzJ2-Cht2pG-dV2Z5UIYfe6eUqlH0z0vnqB1-6oQVtmkKgkcXG0U8ykyNRhMFFSlD9vlVE0Te5FRfi4MQj5xvi9bMB_jhfTRGA"}},"request":{"type":"IntentRequest","requestId":"amzn1.echo-api.request.af6e3689-d023-4c6e-bdcf-e6e6984ed3e4","timestamp":"2018-09-26T14:12:30Z","locale":"en-US","intent":{"name":"deliver_notification","confirmationStatus":"NONE"}}}"###;

    let raw_body_from_city = raw_body.replace("berlin", &city);

    let req = build_request_for_skill_api(raw_body_from_city);

    dispatcher.dispatch(req).wait().unwrap();

}

fn create_notification_for(city: &String, dispatcher: &api::dispatcher::Dispatcher) {

    let raw_body = r###"{"version":"1.0","session":{"new":true,"sessionId":"amzn1.echo-api.session.ac79adc9-b325-45dd-97ae-7a01d4f6d80e","application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"}},"context":{"System":{"application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"},"device":{"deviceId":"amzn1.ask.device.AFBBPRUJRVKP4BAHNQW4BS6FJZP32LOYQO2AYRVRMCKP7D3U5BHCS35VMMAPWMZEHJMDZTQJ5Z7EMJDRWXCADDHYR4OOCL7BTJ44MIZB2EFMCE2WM7DZ4QJDFMVNKAIXQ7OPW6UJDJGCJBKSE2IUOIPRJASFASF7CYBLYIMA725YQFMRGJPBO","supportedInterfaces":{}},"apiEndpoint":"https://api.amazonalexa.com","apiAccessToken":"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6IjEifQ.eyJhdWQiOiJodHRwczovL2FwaS5hbWF6b25hbGV4YS5jb20iLCJpc3MiOiJBbGV4YVNraWxsS2l0Iiwic3ViIjoiYW16bjEuYXNrLnNraWxsLjlmNGVmMWRkLWNlZTktNDBlNS1iMDFkLTMwYjlmNGVjY2U3ZiIsImV4cCI6MTUzNzk1MjU1OCwiaWF0IjoxNTM3OTQ4OTU4LCJuYmYiOjE1Mzc5NDg5NTgsInByaXZhdGVDbGFpbXMiOnsiY29uc2VudFRva2VuIjpudWxsLCJkZXZpY2VJZCI6ImFtem4xLmFzay5kZXZpY2UuQUZCQlBSVUpSVktQNEJBSE5RVzRCUzZGSlpQMzJMT1lRTzJBWVJWUk1DS1A3RDNVNUJIQ1MzNVZNTUFQV01aRUhKTURaVFFKNVo3RU1KRFJXWENBRERIWVI0T09DTDdCVEo0NE1JWkIyRUZNQ0UyV003RFo0UUpERk1WTktBSVhRN09QVzZVSkRKR0NKQktTRTJJVU9JUFJKQVNGQVNGN0NZQkxZSU1BNzI1WVFGTVJHSlBCTyIsInVzZXJJZCI6ImFtem4xLmFzay5hY2NvdW50LkFHV0tQRzNKTTRaMzY0QVlLS1NBR0hLTDZDWVdNSktPQVpHWEc1Q1BYWVgyWTdVS1daVEg2WEVMRldQSUNCQ1daUDdPRjVWRUJTUVRRNFVNQ1ZFN0VWUldOMlBVS0JMTUpHVTNHRDIySFpTUlZVNlRURE1VTjJQSjVNN1RXS0FRT1Q3VkJGS1pKTEJJQ0szV1ZJWE9HREY3WUhYVFdXV0tDNzVEMk9OU0w0Sk9MUlVGRlkySktFQVA1VTQ0VENMSkpCUURERkpNRkdVRzVXWSJ9fQ.euhJ4OZAyXvTLBvrECWBFHWWF6b_KyL8x-XzUaHtPR-ZLXlojS1QrW7HWXolgcOME_PoV625GFVsCeepvepwi5mPPqKKsglY5Kq2FCwjiGSo_M0wGlA5-aINB74k9szyEUoNZuTiXDkU3ebCz6hIsTOuSw0GNUlzOhkDjkoqCmFKBfQFQs_ldDuv_unL240pl7CLY7I8kyQ3u-cbl4HUNqNRI5zX2rveBr89bW71uuJH8KdLdMipVJhVhDmKyqOz-XJUuYv71g4de5V84YlAaMeLa5y9e_KEKwFg_stKZ4UdP_celk46UuVChd1FkB4krCynxDSeBOB-Z4Wy8gjWJg"}},"request":{"type":"IntentRequest","requestId":"amzn1.echo-api.request.b995b603-e353-492a-a36b-68e247fd0415","timestamp":"2018-09-26T08:02:38Z","locale":"en-US","intent":{"name":"create_slap_notification","confirmationStatus":"NONE","slots":{"country":{"name":"country","value":"Berlin","resolutions":{"resolutionsPerAuthority":[{"authority":"amzn1.er-authority.echo-sdk.amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f.country","status":{"code":"ER_SUCCESS_MATCH"},"values":[{"value":{"name":"berlin","id":"1"}}]}]},"confirmationStatus":"NONE"}}}}}"###;

    let raw_body_with_city = raw_body.replace("berlin", &city);

    let req = build_request_for_skill_api(raw_body_with_city);

    dispatcher.dispatch(req).wait().unwrap();

}

fn build_request_for_skill_api(body: String) -> Request<Body> {
    Request::builder()
        .uri("https://auto1.danila.app/alexa-skill")
        .body(Body::from(body))
        .unwrap()
}
