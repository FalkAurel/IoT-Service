use async_std::channel::Sender;

use http::Response;
use serde::Deserialize;


#[derive(Deserialize, serde::Serialize, Debug)]
pub struct DataFrame {
    date: u64,
    temp: i16,
    device_id: u32
}

pub struct Sendable {
    pub sender: Sender<Response<String>>,
    pub data: Option<DataFrame>
}
