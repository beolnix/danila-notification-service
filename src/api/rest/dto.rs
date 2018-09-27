#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    pub message_num: usize
}

impl StatusResponse {
    pub fn new(message_num: usize) -> StatusResponse {
        StatusResponse {
            message_num: message_num
        }
    }
}
