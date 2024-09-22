use std::io;

mod http_handling;
use async_std::{
    io::{ReadExt, WriteExt}, net::{TcpListener, TcpStream}, stream::StreamExt
};

use http_handling::{buffer_to_request, build_response, HttpError};
use http::{Request, Response, StatusCode};


#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    while let Some(connection) = listener.incoming().next().await {
        handle_connection(connection?).await;
    }
    Ok(())
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 1024] = [0; 1024];

    if let Ok(read_bytes) = stream.read(&mut buffer).await {
        match buffer_to_request(&buffer[..read_bytes]) {
            Ok(request) => {
                println!("{request:?}");
                let response: Vec<u8> = build_response(StatusCode::OK, None);

                stream.write(&response).await;
                stream.flush().await;
            },
            Err(err) => println!("Error: {err:?}")
        }
    }
}
