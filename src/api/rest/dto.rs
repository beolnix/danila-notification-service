#[derive(Serialize, Deserialize, Debug)]
pub struct DANotificationResponse {
    message_num: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DANotificationRequest {
    device_name: String
}
