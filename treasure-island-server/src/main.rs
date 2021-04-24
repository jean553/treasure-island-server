#[cfg(feature = "serde_derive")]

extern crate bincode;
extern crate serde_derive;
#[macro_use]
extern crate serde_big_array;

mod message;
mod threads;
mod sprites;

use message::Message;

use sprites::get_sprite_index_from_tile_value;

use threads::{
    forward_messages_from_global_receiver_to_all_clients,
    send_message_into_client_stream,
    receive_message_from_client_stream,
};

use rand::thread_rng;
use rand::Rng;

use std::collections::VecDeque;

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

    const TILES_AMOUNT: usize = 400;
    const DEFAULT_TILE_VALUE: u8 = 0;
    let mut tiles: [u8; TILES_AMOUNT] = [
        DEFAULT_TILE_VALUE;
        TILES_AMOUNT
    ];

    const TILES_PER_LINE: usize = 20;
    const LAST_LINE_FIRST_TILE_INDEX: usize = 380;
    
    const SAND_WATER_BOTTOM_SPRITE_INDEX: u8 = 4;
    const SAND_WATER_TOP_SPRITE_INDEX: u8 = 5;
    const SAND_WATER_LEFT_SPRITE_INDEX: u8 = 6;
    const SAND_WATER_RIGHT_SPRITE_INDEX: u8 = 7;

    let mut range = thread_rng();

    let mut previous_tile_value: u8 = 0;
    let mut previous_line: VecDeque<u8> = VecDeque::new();

    for (index, tile) in tiles.iter_mut().enumerate() {

        if index < TILES_PER_LINE {
            *tile = SAND_WATER_LEFT_SPRITE_INDEX; 
            continue;
        }

        if index % TILES_PER_LINE == 0 {
            *tile = SAND_WATER_TOP_SPRITE_INDEX;
            continue;
        }

        if index % TILES_PER_LINE == TILES_PER_LINE - 1 {
            *tile = SAND_WATER_RIGHT_SPRITE_INDEX;
            continue;
        }

        if index >= LAST_LINE_FIRST_TILE_INDEX {
            *tile = SAND_WATER_BOTTOM_SPRITE_INDEX;
            continue;
        }

        const WATER_TILE_VALUE: u8 = 0;
        const TREE_TILE_VALUE: u8 = 3;

        const FIRST_ISLAND_TILE_INDEX: usize = 21;
        if index == FIRST_ISLAND_TILE_INDEX {

            let tile_value = range.gen_range(WATER_TILE_VALUE..TREE_TILE_VALUE + 1);
            *tile = get_sprite_index_from_tile_value(tile_value);

            previous_tile_value = tile_value;
            previous_line.push_back(tile_value);

            continue;
        }

        const SECOND_LINE_OF_ISLAND_TILES_FIRST_INDEX: usize = 41;
        if index >= SECOND_LINE_OF_ISLAND_TILES_FIRST_INDEX {
            previous_tile_value = previous_line.pop_front().unwrap();
        }

        let mut minimum: u8 = WATER_TILE_VALUE;
        let mut maximum: u8 = TREE_TILE_VALUE;

        if previous_tile_value != WATER_TILE_VALUE {
            minimum = previous_tile_value - 1;
        }

        if previous_tile_value != TREE_TILE_VALUE {
            maximum = previous_tile_value + 1;
        }

        let tile_value = range.gen_range(minimum..maximum + 1);
        *tile = get_sprite_index_from_tile_value(tile_value);

        if index > FIRST_ISLAND_TILE_INDEX &&
            index < SECOND_LINE_OF_ISLAND_TILES_FIRST_INDEX {
            previous_tile_value = tile_value;
        }

        previous_line.push_back(tile_value);
    }

    /* force angles to have water */
    const FIRST_MAP_ANGLE_TILE_INDEX: usize = 0;
    const SECOND_MAP_ANGLE_TILE_INDEX: usize = 19;
    const THIRD_MAP_ANGLE_TILE_INDEX: usize = 380;
    const FOURTH_MAP_ANGLE_TILE_INDEX: usize = 399;
    const WATER_TILE_SPRITE_INDEX: u8 = 10;
    tiles[FIRST_MAP_ANGLE_TILE_INDEX] = WATER_TILE_SPRITE_INDEX;
    tiles[SECOND_MAP_ANGLE_TILE_INDEX] = WATER_TILE_SPRITE_INDEX;
    tiles[THIRD_MAP_ANGLE_TILE_INDEX] = WATER_TILE_SPRITE_INDEX;
    tiles[FOURTH_MAP_ANGLE_TILE_INDEX] = WATER_TILE_SPRITE_INDEX;

    for income in listener.incoming() {

        let stream = income.unwrap();
        let read_stream = stream.try_clone().unwrap();

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

        spawn(|| {
            send_message_into_client_stream(
                client_receiver,
                stream,
            );
        });

        spawn(|| {
            receive_message_from_client_stream(read_stream);
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

            const MESSAGE_ACTION_PUSH_MAP: u8 = 1;
            let mut message = Message::new(MESSAGE_ACTION_PUSH_MAP);
            message.set_data(tiles);
            client_out_sender.send(message).unwrap();

            const MESSAGE_ACTION_START_GAME: u8 = 2;
            let message = Message::new(MESSAGE_ACTION_START_GAME);
            client_out_sender.send(message).unwrap();
        }
    }
}
