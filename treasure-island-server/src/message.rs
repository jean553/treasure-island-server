//! Contains the Message structure, sent through streams between server and client.

use serde_derive::Serialize;

/// Structure of a message between the server and the client.
/// NOTE: only one field wrapped into a structure for now, but more fields will be added later
///
/// FIXME: for tests only, we assume the map is 32 tiles-long, as it is not possible to serialize
/// arrays of more than 32 values; we should divide the map into separated arrays to sent its data;
/// or we should provide our own serialization function for that longer array
#[derive(Serialize, Clone, Copy)]
pub struct Message {
    map: [u8; 32]
}

impl Message {

    /// Message constructor.
    pub fn new() -> Self {

        /* FIXME: does not send 0 values as the client uses the first index
           value of the sent data to know if it should consider the message or not */
        const DEFAULT_VALUE: u8 = 10;
        const MAP_LENGTH: usize = 32;
        Message {
            map: [DEFAULT_VALUE; MAP_LENGTH]
        }
    }
}
