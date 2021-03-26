//! TCP server app

use std::net::TcpListener;

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9500").unwrap();

    println!("Listening for incoming connections...");

    for income in listener.incoming() {

        let stream = income.unwrap();

        let client_address = stream.peer_addr()
            .unwrap();

        println!("New client connected from {}", client_address);
    }
}
