extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde_json::Error;

use crate::storage::Event;
use crate::storage::EventType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericCall {
    pub version: String,
    pub context: Context,
    pub request: Request,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    #[serde(rename = "System")]
    pub system: System,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct System {
    pub device: Device,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Device {
    #[serde(rename = "deviceId")]
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    #[serde(rename = "type")]
    pub type_name: String,

    #[serde(rename = "requestId")]
    pub request_id: String,
    pub intent: Intent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Intent {
    pub name: String,
    pub slots: Option<Slots>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Slots {
    pub city: CitySlot,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CitySlot {
    pub name: String,
    pub value: String,
    pub resolutions: Resolutions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resolutions {
    #[serde(rename = "resolutionsPerAuthority")]
    pub resolutions_per_authority: Vec<ResolutionsPerAuthority>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolutionsPerAuthority {
    pub values: Vec<ResolvedSlotValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolvedSlotValue {
    pub value: SlotValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SlotValue {
    pub name: String,
    pub id: String,
}

impl GenericCall {
    pub fn from(json: &String) -> Result<GenericCall, Error> {
        let call: GenericCall = serde_json::from_str(&json)?;
        Ok(call)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericResult {
    pub version: String,
    pub response: Response,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    #[serde(rename = "outputSpeech")]
    pub output_speech: OutputSpeech,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputSpeech {
    #[serde(rename = "type")]
    pub type_name: String,
    pub text: Option<String>,
    pub ssml: Option<String>,
}

impl GenericResult {
    pub fn notification_created(for_city: String) -> GenericResult {
        GenericResult {
            version: String::from("1.0"),
            response: Response {
                output_speech: OutputSpeech {
                    type_name: String::from("PlainText"),
                    text: Some(String::from(format!(
                        "notification for {} created",
                        &for_city
                    ))),
                    ssml: None,
                },
            },
        }
    }

    pub fn city_not_provided() -> GenericResult {
        GenericResult {
            version: String::from("1.0"),
            response: Response {
                output_speech: OutputSpeech {
                    type_name: String::from("PlainText"),
                    text: Some(String::from(
                        "City which I need to notify hasn't been provided, blame Amazon.",
                    )),
                    ssml: None,
                },
            },
        }
    }

    pub fn city_unknown() -> GenericResult {
        GenericResult {
            version: String::from("1.0"),
            response: Response {
                output_speech: OutputSpeech {
                    type_name: String::from("PlainText"),
                    text: Some(String::from("The device you used hasn't been registered in Danila's notification service properly, blame Danila.")),
                    ssml: None
                }
            }
        }
    }

    pub fn no_notifications_found_for(city: &String) -> GenericResult {
        GenericResult {
            version: String::from("1.0"),
            response: Response {
                output_speech: OutputSpeech {
                    type_name: String::from("PlainText"),
                    text: Some(String::from(format!(
                        "There are no pending notifications for {}",
                        &city
                    ))),
                    ssml: None,
                },
            },
        }
    }

    pub fn for_event(event: Event) -> GenericResult {
        let event_type = event.event_type.clone();
        match event_type {
            EventType::SLAP => GenericResult {
                version: String::from("1.0"),
                response: Response {
                    output_speech: OutputSpeech {
                        type_name: String::from("PlainText"),
                        text: Some(String::from(format!("Someone has just slapped you."))),
                        ssml: None,
                    },
                },
            },
            EventType::MESSAGE => {
                let message = event.message.unwrap();
                GenericResult {
                    version: String::from("1.0"),
                    response: Response {
                        output_speech: OutputSpeech {
                            type_name: String::from("SSML"),
                            text: None,
                            ssml: Some(String::from(format!(r###"<speak>Someone sent you a message: <emphasis level="strong"> {} </emphasis> </speak> "###, &message)))
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn smoke_test_create_custom_notification() {
    let request_json = r###"{"version":"1.0","session":{"new":true,"sessionId":"amzn1.echo-api.session.c9add14f-1b3d-40ad-a7e3-f2452e3c2f47","application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"}},"context":{"System":{"application":{"applicationId":"amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f"},"user":{"userId":"amzn1.ask.account.AGWKPG3JM4Z364AYKKSAGHKL6CYWMJKOAZGXG5CPXYX2Y7UKWZTH6XELFWPICBCWZP7OF5VEBSQTQ4UMCVE7EVRWN2PUKBLMJGU3GD22HZSRVU6TTDMUN2PJ5M7TWKAQOT7VBFKZJLBICK3WVIXOGDF7YHXTWWWKC75D2ONSL4JOLRUFFY2JKEAP5U44TCLJJBQDDFJMFGUG5WY"},"device":{"deviceId":"amzn1.ask.device.AFBBPRUJRVKP4BAHNQW4BS6FJZP32LOYQO2AYRVRMCKP7D3U5BHCS35VMMAPWMZEHJMDZTQJ5Z7EMJDRWXCADDHYR4OOCL7BTJ44MIZB2EFMCE2WM7DZ4QJDFMVNKAIXQ7OPW6UJDJGCJBKSE2IUOIPRJASFASF7CYBLYIMA725YQFMRGJPBO","supportedInterfaces":{}},"apiEndpoint":"https://api.amazonalexa.com","apiAccessToken":"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6IjEifQ.eyJhdWQiOiJodHRwczovL2FwaS5hbWF6b25hbGV4YS5jb20iLCJpc3MiOiJBbGV4YVNraWxsS2l0Iiwic3ViIjoiYW16bjEuYXNrLnNraWxsLjlmNGVmMWRkLWNlZTktNDBlNS1iMDFkLTMwYjlmNGVjY2U3ZiIsImV4cCI6MTUzODA0NjgzOCwiaWF0IjoxNTM4MDQzMjM4LCJuYmYiOjE1MzgwNDMyMzgsInByaXZhdGVDbGFpbXMiOnsiY29uc2VudFRva2VuIjpudWxsLCJkZXZpY2VJZCI6ImFtem4xLmFzay5kZXZpY2UuQUZCQlBSVUpSVktQNEJBSE5RVzRCUzZGSlpQMzJMT1lRTzJBWVJWUk1DS1A3RDNVNUJIQ1MzNVZNTUFQV01aRUhKTURaVFFKNVo3RU1KRFJXWENBRERIWVI0T09DTDdCVEo0NE1JWkIyRUZNQ0UyV003RFo0UUpERk1WTktBSVhRN09QVzZVSkRKR0NKQktTRTJJVU9JUFJKQVNGQVNGN0NZQkxZSU1BNzI1WVFGTVJHSlBCTyIsInVzZXJJZCI6ImFtem4xLmFzay5hY2NvdW50LkFHV0tQRzNKTTRaMzY0QVlLS1NBR0hLTDZDWVdNSktPQVpHWEc1Q1BYWVgyWTdVS1daVEg2WEVMRldQSUNCQ1daUDdPRjVWRUJTUVRRNFVNQ1ZFN0VWUldOMlBVS0JMTUpHVTNHRDIySFpTUlZVNlRURE1VTjJQSjVNN1RXS0FRT1Q3VkJGS1pKTEJJQ0szV1ZJWE9HREY3WUhYVFdXV0tDNzVEMk9OU0w0Sk9MUlVGRlkySktFQVA1VTQ0VENMSkpCUURERkpNRkdVRzVXWSJ9fQ.B5Y7wjEtxv6sH8lOaaf-jVps5yulE-EwpT84GESxd7WjPBfS7iJIjnmkmKatPpbfxRfwte_HerIW0sLKiJ2S9LJI_mg1_9t_iTiymW-ecacwHOjQeAKYRGXBhHfv41D1j_3gVouNe7cNUK8eckUDm5_o_1AjIaDLhqc9FJiNaphBYlJeyB2Mc_NjpKvFgtnS7yqcRiqESA_6imOZwHyVDS02Iq_3H2qvow9ZLfi09QTOjK3AVBkWtdif14ZD89d-jUuGVXZsvxCxB09sRoOkAQ--AZC1t2mm_AWxWsyLhfRinY6nJh4Y5RMfssBYZPfHD_HT8-aM8NsZ4p0r5SnGag"}},"request":{"type":"IntentRequest","requestId":"amzn1.echo-api.request.1fd8560b-185f-493e-b944-d2d860064e86","timestamp":"2018-09-27T10:13:58Z","locale":"en-US","intent":{"name":"create_slap_notification","confirmationStatus":"NONE","slots":{"city":{"name":"city","value":"Berlin","resolutions":{"resolutionsPerAuthority":[{"authority":"amzn1.er-authority.echo-sdk.amzn1.ask.skill.9f4ef1dd-cee9-40e5-b01d-30b9f4ecce7f.city","status":{"code":"ER_SUCCESS_MATCH"},"values":[{"value":{"name":"BERLIN","id":"0"}}]}]},"confirmationStatus":"NONE"}}}}}"###;

    let parsed_call = GenericCall::from(&String::from(request_json)).unwrap();

    assert_eq!("create_slap_notification", parsed_call.request.intent.name);
    assert_eq!(
        "BERLIN",
        parsed_call
            .request
            .intent
            .slots
            .unwrap()
            .city
            .resolutions
            .resolutions_per_authority
            .first()
            .unwrap()
            .values
            .first()
            .unwrap()
            .value
            .name
    );
}
