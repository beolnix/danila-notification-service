#[derive(Serialize, Deserialize, Debug)]
pub struct DACountNotificationResponse {
    pub message_num: usize
}

impl DACountNotificationResponse {
    pub fn new(message_num: usize) -> DACountNotificationResponse {
        DACountNotificationResponse {
            message_num: message_num
        }
    }
}
