use hyper::{http::Error, Body, Response, StatusCode};


pub fn build_response(status_code: StatusCode, status_msg: String) -> Result<Response<Body>, Error> {
    Response::builder()
        .status(status_code)
        .body(Body::from(status_msg))
}
