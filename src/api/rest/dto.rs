#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusResponse {
    pub message_num: usize,
}

impl StatusResponse {
    pub fn new(message_num: usize) -> StatusResponse {
        StatusResponse {
            message_num: message_num,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateNotificationReqeust {
    pub type_name: String,
    pub for_city: String,
    pub message_text: Option<String>,
}
