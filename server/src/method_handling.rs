use async_std::channel::{unbounded, Sender};
use http::{Request, Response, StatusCode, Method};
use serde_json::from_str;
use std::{sync::Arc, str::from_utf8};
use crate::{data::{DataFrame, Sendable}, http_handling::build_response};


#[derive(Debug)]
enum MethodError {
    UTF8Conversion,
    JSONParsingError
}

pub async fn method_handling(request: Request<Vec<u8>>, sender: Arc<Sender<Sendable>>) -> Response<String> {
    match request.method() {
        &Method::POST => match post_request(request.body()) {
            Ok(dataframe) => {
                let (sender_response, receiver) = unbounded::<Response<String>>();
                let sendable: Sendable = Sendable { sender: sender_response , data: Some(dataframe) };

                let _ = sender.send(sendable).await;

                match receiver.recv().await {
                    Ok(response) => response,
                    Err(err ) =>build_response(StatusCode::BAD_GATEWAY, Some(format!("{err:?}")))
                }
            },
            Err(err) => build_response(StatusCode::BAD_REQUEST, Some(format!("{err:?}")))
        },
        &Method::GET => {
            let (sender_response, receiver) = unbounded::<Response<String>>();
            let sendable: Sendable = Sendable { sender: sender_response , data: None };

            let _ = sender.send(sendable).await;

            match receiver.recv().await {
                Ok(response) => response,
                Err(err ) =>build_response(StatusCode::BAD_GATEWAY, Some(format!("{err:?}")))
            }
        },
        _ => build_response(StatusCode::BAD_REQUEST, Some("Method is not implemented".into()))
    }
}


fn post_request(body: &[u8]) -> Result<DataFrame, MethodError> {
    from_str::<DataFrame>(from_utf8(body)
    .map_err(|_| MethodError::UTF8Conversion)?)
    .map_err(|_| MethodError::JSONParsingError)
}
