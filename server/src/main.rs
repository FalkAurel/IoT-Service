mod http_handling;
mod data;
mod method_handling;

use std::{
    collections::VecDeque,
    io, sync::Arc, thread::spawn
};

use async_std::{
    io::{ReadExt, WriteExt}, net::{TcpListener, TcpStream}, stream::StreamExt, channel::{bounded, Sender},
};

use data::{DataFrame, Sendable};
use http_handling::{buffer_to_request, build_response, AsBytes};
use http::{Response, StatusCode};
use method_handling::method_handling;



#[async_std::main]
async fn main() -> Result<(), io::Error> {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let (sender, receiver) = bounded::<Sendable>(1000);

    let sender: Arc<Sender<Sendable>> = Arc::new(sender);



    spawn(|| async move {
        let mut in_memory_buffer: VecDeque<DataFrame> = VecDeque::with_capacity(100);
        while let Ok(message) = receiver.recv().await {
            let Sendable { sender, data } = message;

            // POST REQUEST
            if let Some(data) = data {
                in_memory_buffer.push_front(data);
                let _ = sender.send(build_response(StatusCode::OK, None)).await;
            } else {
                // GET REQUEST
                if let Some(response) = in_memory_buffer.pop_back() {
                    let string: String = serde_json::to_string(&response).unwrap();
                    let _ = sender.send(build_response(StatusCode::OK, Some(string))).await;
                } else {
                    let _ = sender.send(build_response(StatusCode::OK, Some("MEMORY QUEUE IS EMPTY".to_string()))).await;
                }
            }
        }
    });

    while let Some(connection) = listener.incoming().next().await {
        handle_connection(connection?, sender.clone()).await;
    }
    Ok(())
}

async fn handle_connection(mut stream: TcpStream, sender: Arc<Sender<Sendable>>) {
    let mut buffer: [u8; 1024] = [0; 1024];

    if let Ok(read_bytes) = stream.read(&mut buffer).await {
        match buffer_to_request(&buffer[..read_bytes]) {
            Ok(request) => {
                let response: Response<String> = method_handling(request, sender).await;

                let _ = stream.write(&response.bytes()).await;
                let _ = stream.flush().await;
            },
            Err(err) => println!("Error: {err:?}")
        }
    }
}
