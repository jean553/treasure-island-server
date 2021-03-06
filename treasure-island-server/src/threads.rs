//! Contains the different server threads.

use crate::message::Message;

use crate::tiles::load_tiles;

use std::sync::mpsc::{
    Sender,
    Receiver,
};

use std::net::TcpStream;

use std::sync::{
    Mutex,
    Arc,
};

use std::io::{
    Read,
    Write,
    BufReader,
};

use std::str::from_utf8;

/// Contains the whole code of a dedicated thread. Continuously forwards the messages from the
/// global receiver to all the clients out senders (so to all the clients individually).
///
/// # Args:
///
/// `global_receiver` - the unique global receiver that centralizes all messages for all clients
/// `clients_out_senders_mutex_arc` - protected pointer to the array of clients out senders
pub fn forward_messages_from_global_receiver_to_all_clients(
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
pub fn send_message_into_client_stream(
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

/// Conatins the whole code of the a dedicated thread. This thread is spawn once per new client.
/// Continuously checks for incoming messages from the client.
///
/// # Args:
///
/// `client_stream` - the dedicated stream of the client, to receive messages
pub fn receive_message_from_client_stream(
    client_stream: TcpStream,
    clients_usernames: Arc<Mutex<Vec<String>>>,
) {

    let mut buffer = BufReader::new(client_stream);

    const BUFFER_LENGTH: usize = 32;
    let mut message: [u8; BUFFER_LENGTH] = [0; BUFFER_LENGTH];

    loop {

        /* blocking */
        let _ = buffer.read(&mut message).unwrap();

        let message_action = message[0];

        const MESSAGE_ACTION_IGNORED: u8 = 0;
        if message_action == MESSAGE_ACTION_IGNORED {
            continue;
        }

        const MESSAGE_ACTION_SEND_USERNAME: u8 = 1;
        if message_action == MESSAGE_ACTION_SEND_USERNAME {

            const USERNAME_MAX_LENGTH: usize = 31;
            let mut username_bytes: [u8; USERNAME_MAX_LENGTH] = [0; USERNAME_MAX_LENGTH];
            username_bytes.copy_from_slice(&message[1..BUFFER_LENGTH]);
            let username: String = from_utf8(&username_bytes)
                .unwrap()
                .to_string();

            let mut clients_usernames_mutex_guard = clients_usernames.lock().unwrap();
            let clients_usernames = &mut *clients_usernames_mutex_guard;
            clients_usernames.push(username.clone());

            println!("New player registered: {}", username);
        }
    }
}

/// TODO
pub fn main_thread(
    clients_usernames_mutex_arc: Arc<Mutex<Vec<String>>>,
    clients_out_senders_mutex_arc: Arc<Mutex<Vec<Sender<Message>>>>,
) -> ! {

    let mut tiles = load_tiles();

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

    /* FIXME: should be an enumeration */
    const WAITING_FOR_PLAYERS_STATE: u8 = 0;
    let mut game_state: u8 = WAITING_FOR_PLAYERS_STATE;

    loop {

        if game_state == WAITING_FOR_PLAYERS_STATE {

            let mut clients_usernames_mutex_guard = clients_usernames_mutex_arc.lock().unwrap();
            let clients_usernames = &mut *clients_usernames_mutex_guard;

            const EXPECTED_USERNAMES_AMOUNT: usize = 2;
            if clients_usernames.len() == EXPECTED_USERNAMES_AMOUNT {

                println!("Sending map information to clients...");

                let mut clients_out_senders_mutex_guard = clients_out_senders_mutex_arc.lock().unwrap();
                let clients_out_senders = &mut *clients_out_senders_mutex_guard;

                for client_out_sender in clients_out_senders {

                    const MESSAGE_ACTION_PUSH_MAP: u8 = 1;
                    let mut message = Message::new(MESSAGE_ACTION_PUSH_MAP);
                    message.set_data(tiles);
                    client_out_sender.send(message).unwrap();

                    const MESSAGE_ACTION_START_GAME: u8 = 2;
                    let message = Message::new(MESSAGE_ACTION_START_GAME);
                    client_out_sender.send(message).unwrap();
                }

                const IN_GAME_STATE: u8 = 1;
                game_state = IN_GAME_STATE;
            }
        }
    }
}
