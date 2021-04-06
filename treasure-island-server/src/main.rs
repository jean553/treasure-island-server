#[cfg(feature = "serde_derive")]

extern crate bincode;
extern crate serde_derive;

mod message;
mod threads;

use message::Message;

use threads::{
    forward_messages_from_global_receiver_to_all_clients,
    send_message_into_client_stream,
};

use std::net::TcpListener;

use std::sync::mpsc::{
    Sender,
    Receiver,
    channel,
};

use std::thread::spawn;

use std::sync::{
    Mutex,
    Arc,
};

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9500").unwrap();

    println!("Listening for incoming connections...");

    /* create the global receiver into which one every message
       is sent in order to be forwarded to all clients */
    let (_, global_receiver): (
        Sender<Message>,
        Receiver<Message>,
    ) = channel();

    /* create a dynamic array of senders (one per client)
       used to forward all messages from the global receiver
       in order to broadcast out messages to all clients */
    let clients_out_senders: Vec<Sender<Message>> = Vec::new();

    /* clients out senders array is part of the main thread,
       in order to dynamically add one client out sender
       when a new client connects; we also need it
       into the thread that forwards messages from
       the global receiver to all clients;
       we have to protect it with a mutex
       to prevent concurrent access;
       we can then safely clone the clients out senders array pointer */
    let clients_out_senders_mutex: Mutex<Vec<Sender<Message>>> = Mutex::new(clients_out_senders);
    let clients_out_senders_mutex_main_thread_arc: Arc<Mutex<Vec<Sender<Message>>>> = Arc::new(clients_out_senders_mutex);
    let clients_out_senders_mutex_global_receiver_to_all_clients_thread_arc = clients_out_senders_mutex_main_thread_arc.clone();

    spawn(|| {
        forward_messages_from_global_receiver_to_all_clients(
            global_receiver,
            clients_out_senders_mutex_global_receiver_to_all_clients_thread_arc
        );
    });

    for income in listener.incoming() {

        let stream = income.unwrap();

        let client_address = stream.peer_addr()
            .unwrap();

        println!("New client connected from {}", client_address);

        let (
            client_sender,
            client_receiver,
        ): (
            Sender<Message>,
            Receiver<Message>
        ) = channel();

        println!("Copy client stream and create dedicated thread to communicate with {}", client_address);

        let stream = stream.try_clone().unwrap();

        spawn(|| {
            send_message_into_client_stream(
                client_receiver,
                stream,
            );
        });

        let mut client_out_senders_mutex_guard = clients_out_senders_mutex_main_thread_arc.lock().unwrap();
        let client_out_senders = &mut *client_out_senders_mutex_guard;
        client_out_senders.push(client_sender);

        println!("{} added into clients list", client_address);

        const CLIENTS_PER_GAME_AMOUNT: usize = 2;
        if client_out_senders.len() != CLIENTS_PER_GAME_AMOUNT {

            println!("Waiting for more clients to connect...");
            continue;
        }

        println!("Sending map information to clients...");

        /* TODO: sends common level information to all clients */

        for client_out_sender in client_out_senders {

            let message = Message::new();
            client_out_sender.send(message).unwrap();
        }
    }
}
