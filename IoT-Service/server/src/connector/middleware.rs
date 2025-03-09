use hyper::Uri;
use hyper::{
    Body,
    Method,
    Request,
    Response,
    StatusCode,
    body::HttpBody,
    header::HeaderValue
};

use serde_json;

use base64::prelude::*;
use serde::Deserialize;
use tokio::sync::oneshot;
use crossbeam::channel::Sender;

use crate::error::Error;
use super::response::build_response;
use crate::util::load_authentification_data;
use crate::data_managment::dataformat::{DataFrame, Query};
use crate::data_managment::message_passing::{self, Message};

use std::str;
use std::sync::LazyLock;
use std::collections::HashMap;
use std::sync::Arc;


const BASIC_LENGTH: usize = "Basic ".len();
static AUTHENTIFICATION: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    load_authentification_data(".env")
});

pub fn auth(auth_header: &HeaderValue) -> Result<bool, Error> {
    if let Ok(auth_header) = auth_header.to_str() {
        if auth_header.starts_with("Basic ") {
            let base_64_encoded: &str = auth_header.get(BASIC_LENGTH..auth_header.len()).unwrap(); // This is safe, as the previous line gurantees that the string is at least of length "Basic "
            let decoded_auth_header: Vec<u8> = BASE64_STANDARD.decode(base_64_encoded).map_err(|_| Error::DecodeError)?;

            let mut decoded_auth_header = decoded_auth_header.split(|elem| *elem == b':');

            let username: &str = str::from_utf8(decoded_auth_header.next().ok_or_else(|| Error::InvalidAuthentificationFormat)?).map_err(|_| Error::ConversionError)?;
            let unchecked_password: &str = str::from_utf8(decoded_auth_header.next().ok_or_else(|| Error::InvalidAuthentificationFormat)?).map_err(|_| Error::ConversionError)?;


            Ok(AUTHENTIFICATION.get(username).map_or(false, |password: &String| unchecked_password == password))
        } else {
            Err(Error::InvalidAuthentificationFormat) // If the authentification header doesn't follow basic auth.
        }
    } else {
        Err(Error::FailedHeaderConversion)
    }
}


pub async fn serve_request(mut req: Request<Body>, sender_channel: Arc<Sender<Message>>) -> Response<Body> {
    match req.method() {
        &Method::POST => post_request(req.body_mut(), sender_channel).await,
        &Method::GET => get_request(req.uri(), sender_channel).await,
        &Method::DELETE => delete_request(req.uri(), sender_channel).await,
        &Method::PUT => put_request(&req.uri().clone(),req.body_mut(), sender_channel).await,
        _ => build_response(StatusCode::NOT_IMPLEMENTED, "".to_string()).expect("Implement logging")
    }
}

async fn parse_body<T: for <'a> Deserialize<'a>>(data: &mut Body) -> Result<T, Error> {
    if let Ok(body) = data.collect().await {
        let data: String = body.to_bytes().iter().map(|elem| *elem as char).collect::<String>();
        serde_json::from_str::<T>(&data).map_err(|_| Error::InvalidDataFormat)
    } else {
        Err(Error::TransmissionError)
    }
}

async fn post_request(data: &mut Body, sender: Arc<Sender<Message>>) -> Response<Body> {
    match parse_body::<DataFrame>(data).await {
        Ok(dataframe) => {
            if let Ok(_) = sender.send(Message::Post(dataframe)) {
                build_response(StatusCode::OK, "".to_string()).expect("Implement Logging")
            } else {
                build_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed Message Passing to Worker".to_string()).expect("Implement Logging")
            }
        },
        Err(err) => match err {
            Error::InvalidDataFormat => build_response(StatusCode::BAD_REQUEST, "Invalid data format".to_string()).expect("Implement logging"),
            Error::TransmissionError => build_response(StatusCode::BAD_REQUEST, "Data transmission failed".to_string()).expect("Implement logging"),
            _ => panic!("ATM undefined behaviour, open for future extension")
        }
    }
}

async fn get_request(uri: &Uri, sender: Arc<Sender<Message>>) -> Response<Body> {
    match extract_query(uri) {
        Ok(query) => {
            // Bi-directional channel to relay query back. One shot channels are non blocking, meaning the main thread can continue executing
            let (response_sender, response_receiver) = oneshot::channel::<Box<dyn message_passing::Response>>();

            if let Ok(_) = sender.send(Message::Get(query, response_sender)) {
                match response_receiver.await {
                    Ok(response) => build_response(response.status_code(), response.content()).expect("Implement Logging"),
                    Err(_) => build_response(StatusCode::BAD_GATEWAY, "Worker timed out".to_string()).expect("Implement Logging")
                }
            } else {
                build_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed Message Passing to Worker".to_string()).expect("Implement Logging")
            }
        },
        Err(err) => match err {
            Error::QueryInvalidAPI => build_response(StatusCode::BAD_REQUEST, "Invalid API. Consider using /api/v1/".to_string()).expect("Implement logging"),
            Error::QueryNotProvided => build_response(StatusCode::BAD_REQUEST, "No Query found. Consider adding one.".to_string()).expect("Implement logging"),
            Error::QueryParsingError => build_response(StatusCode::BAD_REQUEST, "Couldn't parse query".to_string()).expect("Implement logging"),
            _ => build_response(StatusCode::INTERNAL_SERVER_ERROR, "Request introduces undefined behaviour. Request rejected".to_string()).expect("Implement logging")
        }
    }
}

async fn delete_request(uri: &Uri, sender: Arc<Sender<Message>>) -> Response<Body> {
    match extract_query(uri) {
        Ok(query) => {
            // Bi-directional channel to relay query back. One shot channels are non blocking, meaning the main thread can continue executing
            let (response_sender, response_receiver) = oneshot::channel::<Box<dyn message_passing::Response>>();

            if let Ok(_) = sender.send(Message::Delete(query, response_sender)) {
                match response_receiver.await {
                    Ok(response) => build_response(response.status_code(), response.content()).expect("Implement Logging"),
                    Err(_) => build_response(StatusCode::BAD_GATEWAY, "Worker timed out".to_string()).expect("Implement Logging")
                }
            } else {
                build_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed Message Passing to Worker".to_string()).expect("Implement Logging")
            }
        },
        Err(err) => match err {
            Error::QueryInvalidAPI => build_response(StatusCode::BAD_REQUEST, "Invalid API. Consider using /api/v1/".to_string()).expect("Implement logging"),
            Error::QueryNotProvided => build_response(StatusCode::BAD_REQUEST, "No Query found. Consider adding one.".to_string()).expect("Implement logging"),
            Error::QueryParsingError => build_response(StatusCode::BAD_REQUEST, "Couldn't parse query".to_string()).expect("Implement logging"),
            _ => build_response(StatusCode::INTERNAL_SERVER_ERROR, "Request introduces undefined behaviour. Request rejected".to_string()).expect("Implement logging")
        }
    }
}

async fn put_request(uri: &Uri, data: &mut Body, sender: Arc<Sender<Message>>) -> Response<Body> {
    match (extract_query(uri), parse_body::<DataFrame>(data).await) {
        (Ok(query), Ok(dataframe))=> {

            // Bi-directional channel to relay query back. One shot channels are non blocking, meaning the main thread can continue executing
            let (response_sender, response_receiver) = oneshot::channel::<Box<dyn message_passing::Response>>();

            if let Ok(_) = sender.send(Message::Put(query, dataframe, response_sender)) {
                match response_receiver.await {
                    Ok(response) => build_response(response.status_code(), response.content()).expect("Implement Logging"),
                    Err(_) => build_response(StatusCode::BAD_GATEWAY, "Worker timed out".to_string()).expect("Implement Logging")
                }
            } else {
                build_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed Message Passing to Worker".to_string()).expect("Implement Logging")
            }
        },
        (Err(query_err), Err(parse_err)) => match (query_err, parse_err) {
            (Error::QueryInvalidAPI, _) => build_response(StatusCode::BAD_REQUEST, "Invalid API. Consider using /api/v1/".to_string()).expect("Implement logging"),
            (Error::QueryNotProvided, _) => build_response(StatusCode::BAD_REQUEST, "No Query found. Consider adding one.".to_string()).expect("Implement logging"),
            (Error::QueryParsingError, _) => build_response(StatusCode::BAD_REQUEST, "Couldn't parse query".to_string()).expect("Implement logging"),
            (_, Error::InvalidDataFormat) => build_response(StatusCode::BAD_REQUEST, "Invalid data format".to_string()).expect("Implement logging"),
            (_, Error::TransmissionError) => build_response(StatusCode::BAD_REQUEST, "Data transmission failed".to_string()).expect("Implement logging"),
            _ => build_response(StatusCode::INTERNAL_SERVER_ERROR, "Request introduces undefined behaviour. Request rejected".to_string()).expect("Implement logging")
        },
        _ => build_response(StatusCode::INTERNAL_SERVER_ERROR, "Request introduces undefined behaviour. Request rejected".to_string()).expect("Implement logging")
    }
}

fn extract_query(uri: &Uri) -> Result<Query, Error> {
    // Check if the URI path matches the expected endpoint

    if uri.path() == "/api/v1/" {
        if let Some(query_str) = uri.query() {
            match serde_urlencoded::from_str::<Query>(query_str) {
                Ok(query) => Ok(query),
                Err(_) => Err(Error::QueryParsingError),
            }
        } else {
            Err(Error::QueryNotProvided)
        }
    } else {
        Err(Error::QueryInvalidAPI)
    }
}
