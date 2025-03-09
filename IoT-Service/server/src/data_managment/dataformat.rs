use serde::{Serialize, Deserialize};


/// This modul is for the user. This allows for a generic implementation of a dataframe while still maintaining the efficiency and safety granted by typed datatypes.
/// Each DataFrame represents one querry

/// Define your own Dataframe as you may see fit. Make sure that you sent everytime a valid dataframe via JSON.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DataFrame {
    pub temp: i16, // Struct member name must be identical to the corresponding json key
    pub rpm: i32,  // Struct member name must be identical to the corresponding json key
    pub device_id: i32,  // Struct member name must be identical to the corresponding json key
    pub time_stamp: i32,  // Struct member name must be identical to the corresponding json key
}

/// Define your own Dataframe as you may see fit. Make sure that you sent everytime a valid dataframe via JSON.
#[derive(Debug, Deserialize, Default)]
pub struct Query {
    #[serde(default = "default_device_id")]
    pub device_id: Option<i32>,
    #[serde(default = "default_time_start")]
    pub time_start: Option<i32>,
    #[serde(default = "default_time_end")]
    pub time_end: Option<i32>,
    #[serde(default = "default_time_now")]
    pub time_now: Option<i32>
}

fn default_device_id<T>() -> Option<T> {
    None
}

fn default_time_start<T>() -> Option<T> {
    None
}

fn default_time_end<T>() -> Option<T> {
    None
}

fn default_time_now<T>() -> Option<T> {
    None
}
