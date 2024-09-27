use http::{Request, Response, StatusCode, Method};
use serde_json::from_str;
use std::str::from_utf8;
use crate::{data::DataFrame, http_handling::build_response};


#[derive(Debug)]
enum MethodError {
    UTF8Conversion,
    JSONParsingError
}

pub fn method_handling(request: Request<Vec<u8>>) -> Response<String> {
    match request.method() {
        &Method::POST => match post_request(request.body()) {
            Ok(dataframe) => {
                println!("Received {dataframe:?}");
                build_response(StatusCode::OK, None)
            },
            Err(err) => build_response(StatusCode::BAD_REQUEST, Some(format!("{err:?}")))
        },
        &Method::GET => unimplemented!("Handle get request"),
        _ => build_response(StatusCode::BAD_REQUEST, Some("Method is not implemented".into()))
    }
}


fn post_request(body: &[u8]) -> Result<DataFrame, MethodError> {
    from_str::<DataFrame>(from_utf8(body)
    .map_err(|_| MethodError::UTF8Conversion)?)
    .map_err(|_| MethodError::JSONParsingError)
}
