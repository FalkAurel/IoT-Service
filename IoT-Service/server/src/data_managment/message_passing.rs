use hyper::StatusCode;
use tokio::sync::oneshot::Sender;

use super::dataformat::{DataFrame, Query};

/// This will be send over channels to the working thread to be processed
/// The enum variant wrapping the inner data defines the method, what should be done with the data
/// Message passing is necessary, since you can't have blocking fuctions (database connection) on async channels
pub enum Message {
    Post(DataFrame),
    // Sender is used for a bi directional channel. Take a look at get_request in middleware to get a better idea
    Get(Query, Sender<Box<dyn Response>>),
    Delete(Query, Sender<Box<dyn Response>>),
    Put(Query, DataFrame, Sender<Box<dyn Response>>)
}

pub struct ResponseMessage<T> {
    content: T,
    status_code: StatusCode
}

// Very simple thread safe trait for none blocking message passing
pub trait Response: Send + Sync {
    fn content(&self) -> String;

    fn status_code(&self) -> StatusCode;
}

impl <T> ResponseMessage<T> {
    pub fn new(content: T, status_code: StatusCode) -> Self {
        Self { content, status_code }
    }
}

impl <T: ToString + Send + Sync> Response for ResponseMessage<T> {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn content(&self) -> String {
        self.content.to_string()
    }
}
