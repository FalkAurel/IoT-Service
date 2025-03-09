use hyper::header::{HeaderValue, AUTHORIZATION};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

use rayon::prelude::*;
use crossbeam::channel::{Sender, unbounded};

use crossterm::event::read;
use crossterm::event::{poll, Event, KeyCode, KeyEvent};

use server::error::Error;
use postgres::Client;
use server::util::{init_db, terminate_db};
use server::data_managment::database::{delete, get, update};

use std::thread;
use std::sync::Arc;
use std::net::SocketAddr;
use std::convert::Infallible;
use std::time::Duration;

use server::data_managment::dataformat::DataFrame;
use server::data_managment::message_passing::{self, Message, ResponseMessage};
use server::connector::{response::build_response, middleware::{auth, serve_request}};

#[tokio::main]
async fn main() {
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let (sender, receiver) = unbounded::<Message>();
    let sender: Arc<Sender<Message>> = Arc::new(sender.clone());

    let working: thread::JoinHandle<()> = thread::spawn(move || {
        let mut client: Client = init_db().expect("Implement logging");
        /*

        if let Ok(k) = client.execute(
            r"CREATE TABLE Dataframe (
                temp            smallint,
                rpm             int,
                device_id       int,
                time_stamp      int
            )" , &[]) {
            println!("{k}")
        };

        */

        loop {
            if let Ok(true) = poll(Duration::from_millis(0)) {
                if let Ok(Event::Key(KeyEvent {code: KeyCode::Char('q'), ..})) = read() {
                    break;
                }
            }

            receiver.try_iter().for_each(|message| {
                match message {
                    Message::Post(dataframe) => {
                        let DataFrame {temp, rpm, device_id, time_stamp} = dataframe;

                        //"INSERT INTO author (name, country) VALUES ($1, $2)",
                        // &[&author.name, &author.country]

                        let query: &str = "INSERT INTO Dataframe (temp, rpm, device_id, time_stamp) VALUES ($1, $2, $3, $4)";

                        client.execute(query, &[&temp, &rpm, &device_id, &time_stamp]).expect("Implement logging | Insertion failed");
                    },
                    Message::Get(ref query, response_channel) => {
                        let response: Result<Vec<DataFrame>, Error> = get(query, &mut client);
                        let response: String = match response {
                            Ok(query) => {
                                let mut reponse: String = String::from("{\n\t \"response\": [");

                                for dataframe in query {
                                    reponse.push_str(&format!("\n\t\t{},", serde_json::to_string(&dataframe).unwrap()))
                                }

                                // remove the last ","
                                reponse.remove(reponse.len() - 1);

                                reponse.push_str("\n\t]");
                                reponse.push_str("\n}");

                                reponse
                            },
                            Err(err) => match err {
                                Error::DatabaseQueryFailed(err) => err,
                                _ => "Unknown Error".to_string()
                            }
                        };

                        let _ = response_channel.send(Box::new(ResponseMessage::new(response, StatusCode::OK)));
                    },
                    Message::Delete(ref query, response_channel) => {
                        let response: Box<dyn message_passing::Response> = match delete(query, &mut client) {
                            Ok(()) => Box::new(message_passing::ResponseMessage::new("", StatusCode::OK)),
                            Err(err) => {
                                let (status_code, content) = match err {
                                    Error::DatabaseDeletionError(inner) => (StatusCode::BAD_REQUEST, inner),
                                    _ => (StatusCode::INTERNAL_SERVER_ERROR, String::from("Undefined Behaviour"))
                                };

                                Box::new(message_passing::ResponseMessage::new(content, status_code))
                            }
                        };
                        let _ = response_channel.send(response);
                    },
                    Message::Put(ref query, ref dataframe, response_channel) => {
                        let response: Box<dyn message_passing::Response> = match update(query, dataframe, &mut client) {
                            Ok(()) => Box::new(message_passing::ResponseMessage::new("", StatusCode::OK)),
                            Err(err) => {
                                let (status_code, content) = match err {
                                    Error::DatabaseUpdateError(inner) => (StatusCode::BAD_REQUEST, inner),
                                    _ => (StatusCode::INTERNAL_SERVER_ERROR, String::from("Undefined Behaviour"))
                                };

                                Box::new(message_passing::ResponseMessage::new(content, status_code))
                            }
                        };
                        let _ = response_channel.send(response);
                    }
                }
            })
        }
        print!("Terminating IoT-Service...");
        terminate_db().expect("Implement logging");
        println!("Done");

        std::process::exit(0);
    });

    // Capture the sender and pass it to the service function
    let make_svc = make_service_fn(move |_conn| {

        // Find a way to optimize this clone away
        let sender: Arc<Sender<Message>> = sender.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                handle_request(req, sender.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    working.join().unwrap();
}

async fn handle_request(req: Request<Body>, sender: Arc<Sender<Message>>) -> Result<Response<Body>, Infallible> {
    let auth_header: Option<&HeaderValue> = req.headers().get(AUTHORIZATION);

    if let Some(auth_header) = auth_header {
        if let Ok(true) = auth(auth_header) {
            // Now you have access to the sender
            Ok(serve_request(req, sender).await)
        } else {
            Ok(build_response(StatusCode::UNAUTHORIZED, "Password and/or Username don't match".to_string()).expect("Implement Logging"))
        }
    } else {
        Ok(build_response(StatusCode::UNAUTHORIZED, "Request does not implement required BASIC AUTH".to_string()).expect("Implement Logging"))
    }
}
