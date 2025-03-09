use std::collections::HashMap;
use std::fs::read_to_string;
use std::io;
use std::process::Command;
use std::str::Split;
use std::thread::sleep;
use std::time::Duration;

use postgres::{Client, NoTls};

pub fn load_authentification_data(path: &str) -> HashMap<String, String> {
    let env_file: String = read_to_string(path).expect("Implement Logging | .env file doesn't exist");

    let mut hash_map: HashMap<String, String> = env_file.lines().map(|line| {
        let mut line: Split<&str> = line.split(" ");

        let user: &str = line.next().unwrap();
        let password: &str = line.next().unwrap();

        (user.to_string(), password.to_string())
    }).collect::<HashMap<String, String>>();

    hash_map.shrink_to_fit();

    hash_map
}


pub fn init_db() -> Result<Client, io::Error> {
    // Start the PostgreSQL server
    Command::new("pg_ctl")
        .args(["-D", "/opt/homebrew/var/postgresql@14", "start"])
        .output()?;

    // Wait and retry logic with logging
    let mut attempts = 0;
    let max_attempts = 10;

    loop {
        attempts += 1;

        // Attempt to connect to the PostgreSQL server
        match Client::configure()
            .user("test_user")
            .password("test_password")
            .host("localhost")
            .port(5432)
            .dbname("test_db").connect(NoTls) {
            Ok(client) => {
                println!("Successfully connected to the database after {} attempts.", attempts);
                return Ok(client);
            }
            Err(err) if attempts < max_attempts => {
                println!("Attempt {}: Failed to connect to the database. Error: {}", attempts, err);
                sleep(Duration::from_secs(1)); // Wait for 1 second before retrying
            }
            Err(err) => {
                println!("Failed to connect to the database after {} attempts. Error: {}",attempts, err);
                return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
            }
        }
    }
}

pub fn terminate_db() -> Result<(), io::Error> {
    Command::new("pg_ctl").args(["-D", "/opt/homebrew/var/postgresql@14 stop"]).output()?;
    Ok(())
}
