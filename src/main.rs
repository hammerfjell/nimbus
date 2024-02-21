use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

mod database;
mod command_processor;
mod tokenizer;

use database::Database;
use crate::command_processor::process_command;

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();
    let mut port = 4040_u16; // default port
    let mut gc_timer = Duration::from_secs(60 * 10);

    for (i, arg) in args.iter().enumerate() {
        if arg == "--port" || arg == "-p" {
            if let Some(p) = args.get(i + 1) {
                match p.parse() {
                    Ok(p) => port = p,
                    Err(e) => {
                        eprintln!("Failed to parse port: {}", e);
                    }
                }
                break;
            }
        } else if arg == "--gc" || arg == "-g" {
            if let Some(g) = args.get(i + 1) {
                match g.parse() {
                    Ok(p) => {
                        if p >= 60 { // Check if p is more than 60 seconds
                            gc_timer = Duration::from_secs(p);
                        } else {
                            eprintln!("The gc timer must be greater or equal than 60 seconds.");
                            gc_timer = Duration::from_secs(60);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to parse gc timer: {}", e);
                    }
                }
                break;
            }
        }
    }

    // Initialize the shared database
    let db = Database::new();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();

    let local_addr = listener.local_addr().expect("Failed to get local address");

    println!("nimbus-server listening on port {}.", local_addr.port());

    // collect the garbage every 10 minutes
    let db_clone = db.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(gc_timer).await;
            db_clone.garbage_collect().await;
        }
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let db_clone = db.clone();

        tokio::spawn(async move {
            handle_connection(socket, db_clone).await;
        });
    }
}


async fn handle_connection(socket: TcpStream, db: Arc<Database>) {
    //let mut start = Instant::now();

    let mut reader = BufReader::new(socket);
    let mut line = String::new();

    // Attempt to read a line from the socket
    match reader.read_line(&mut line).await {
        Ok(0) => {}, // Connection was closed before any data was received
        Ok(_) => {
            // Trim the newline character(s) from the end of the line
            let line = line.trim_end();

            //start = Instant::now();

            let response = process_command(line, db).await;

            // Here we implement the send_response
            // Since reader.into_inner() consumes the reader, you need to use the socket directly.
            // Re-acquire the socket from the BufReader to send the response
            let mut socket = reader.into_inner();
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Failed to send response: {}", e);
            }


            // let elapsed = start.elapsed(); // Get the elapsed time as a Duration
            // println!("Elapsed time: {} ms, {} μs", elapsed.as_millis(), elapsed.as_micros());

            // Attempt to shut down the connection gracefully
            socket.shutdown().await.unwrap()
        },
        Err(e) => eprintln!("Failed to read from socket: {}", e),
    }
}