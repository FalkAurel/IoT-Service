use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct DataFrame {
    date: u64,
    temp: i16,
    device_id: u32
}
