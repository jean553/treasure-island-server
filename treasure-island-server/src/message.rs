//! Contains the Message structure, sent through streams between server and client.

use serde_derive::Serialize;

/* generates the function from the macro that is responsible
   to serialize arrays longer than 32 values */
big_array! {
    BigArray;
    400
}


/// Structure of a message between the server and the client.
/// NOTE: only one field wrapped into a structure for now, but more fields will be added later
#[derive(Serialize, Clone, Copy)]
pub struct Message {

    /* bring the BigArray macro into scope for serialization of custom length array */
    #[serde(with = "BigArray")]
    map: [u8; 400]
}

impl Message {

    /// Message constructor.
    pub fn new() -> Self {

        /* FIXME: does not send 0 values as the client uses the first index
           value of the sent data to know if it should consider the message or not */
        const DEFAULT_VALUE: u8 = 10;
        const MAP_LENGTH: usize = 400;
        Message {
            map: [DEFAULT_VALUE; MAP_LENGTH]
        }
    }

    /// Map setter.
    ///
    /// # Args:
    ///
    /// `map` - the map grid
    pub fn set_map(&mut self, map: [u8; 400]) {
        self.map = map;
    }
}
