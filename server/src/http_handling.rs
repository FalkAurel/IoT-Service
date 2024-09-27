use http::{header::CONTENT_LENGTH, request::Builder, Method, Request, Response, StatusCode, Uri, Version};
use httparse::{Header, EMPTY_HEADER, Request as HttpRequest, Status};


#[derive(Debug)]
pub enum HttpError {
    IncompleteHeader,
    InvalidContent,
    NoMethod,
    InvalidMethod,
    NoURI,
    InvalidURI,
    NoVersion,
    InvalidVersion,
    RequestCreationError
}
pub fn buffer_to_request(buffer: &[u8]) -> Result<Request<Vec<u8>>, HttpError> {
    let mut header: [Header; 16] = [EMPTY_HEADER; 16];
    let mut request: HttpRequest = HttpRequest::new(&mut header);

    match request.parse(buffer).map_err(|_| HttpError::InvalidContent)? {
        Status::Complete(header_len) => {
            let method: Method  = request.method
                .ok_or_else(|| HttpError::NoMethod)?
                .parse::<Method>()
                .map_err(|_| HttpError::InvalidMethod)?;

            let uri: Uri = request.path
                .ok_or_else(|| HttpError::NoURI)?
                .parse::<Uri>()
                .map_err(|_| HttpError::InvalidURI)?;

            let version: Version = match request.version {
                Some(0) => Version::HTTP_10,
                Some(1) => Version::HTTP_11,
                Some(_) => Err(HttpError::InvalidVersion)?,
                _ => Err(HttpError::NoVersion)?
            };

            let mut builder: Builder = Builder::new()
                .method(method)
                .uri(uri)
                .version(version);

            for header in request.headers {
                builder = builder.header(header.name, header.value);
            }

            Ok(builder.body(buffer[header_len..].into()).map_err(|_| HttpError::RequestCreationError)?)
        },
        Status::Partial => Err(HttpError::IncompleteHeader)
    }
}

pub fn build_response(status: StatusCode, message: Option<String>) -> Response<String> {
    if let Some(body) = message {
        Response::builder().status(status).header(CONTENT_LENGTH, body.len()).body(body.to_string()).unwrap()
    } else {
        Response::builder().status(status).header(CONTENT_LENGTH, 0).body("".to_string()).unwrap()
    }
}

impl <T: ToString> AsBytes for Response<T> {
    fn bytes(&self) -> Vec<u8> {
        let data: String = self.body().to_string();
        format!("HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status(),
            self.status().canonical_reason().unwrap_or("Unknown"),
            data.len(),
            data
        ).into()
    }
}

pub trait AsBytes {
    fn bytes(&self) -> Vec<u8>;
}
