#[cfg(feature = "serde_derive")]

extern crate bincode;
extern crate serde_derive;

use serde_derive::{Serialize};

use std::net::{
    TcpListener,
    TcpStream,
};

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

use std::io::Write;

/// Structure of a message between the server and the client.
/// NOTE: only one field wrapped into a structure for now, but more fields will be added later
///
/// FIXME: for tests only, we assume the map is 32 tiles-long, as it is not possible to serialize
/// arrays of more than 32 values; we should divide the map into separated arrays to sent its data;
/// or we should provide our own serialization function for that longer array
#[derive(Serialize, Clone, Copy)]
struct Message {
    map: [u8; 32]
}

/// Contains the whole code of a dedicated thread. Continuously forwards the messages from the
/// global receiver to all the clients out senders (so to all the clients individually).
///
/// # Args:
///
/// `global_receiver` - the unique global receiver that centralizes all messages for all clients
/// `clients_out_senders_mutex_arc` - protected pointer to the array of clients out senders
fn forward_messages_from_global_receiver_to_all_clients(
    global_receiver: Receiver<Message>,
    clients_out_senders_mutex_arc: Arc<Mutex<Vec<Sender<Message>>>>,
) {

    loop {

        /* blocking until messages come from the global receiver */
        let message = global_receiver.recv();

        /* FIXME: #5 investigate why the recv() function returns an error
           the first time it is executed, causing the whole thread to panic and to stop;
           simply ignore the error for now */
        if message.is_err() {
            continue;
        }

        let message = message.unwrap();

        /* blocking until there is no use of the clients out senders array from another thread */
        let clients_out_senders_mutex_guard = clients_out_senders_mutex_arc.lock().unwrap();

        let clients_out_senders = &*clients_out_senders_mutex_guard;

        for client_out_sender in clients_out_senders {
            client_out_sender.send(message).unwrap();
        }
    }
}

/// Contains the whole code of a dedicated thread. This thread is spawn once per new client.
/// Continuously forwards the client dedicated receiver to the client dedicated stream.
///
/// # Args:
///
/// `client_receiver` - the dedicated receiver of the client, to get messages to send
/// `client_stream` - the dedicated stream of the client, to send message to him
fn send_message_into_client_stream(
    client_receiver: Receiver<Message>,
    mut client_stream: TcpStream,
) {

    loop {

        /* blocks until message comes from the current client sender */
        let message = client_receiver.recv().unwrap();

        let data: Vec<u8> = bincode::serialize(&message).unwrap();
        client_stream.write(&data).unwrap();
    }
}

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

            /* FIXME: does not send 0 values as the client uses the first index
               value of the sent data to know if it should consider the message or not */
            const DEFAULT_VALUE: u8 = 10;
            const MAP_LENGTH: usize = 32;
            let message = Message {
                map: [
                    DEFAULT_VALUE;
                    MAP_LENGTH
                ]
            };

            client_out_sender.send(message).unwrap();
        }
    }
}
