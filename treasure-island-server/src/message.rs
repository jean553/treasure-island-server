//! Contains the Message structure, sent through streams between server and client.

use serde_derive::Serialize;

/* generates the function from the macro that is responsible
   to serialize arrays longer than 32 values */
big_array! {
    BigArray;
    400
}

#[derive(Serialize, Clone, Copy)]
pub enum MessageAction {
    PushMap,
}

/// Structure of a message between the server and the client.
/// NOTE: only one field wrapped into a structure for now, but more fields will be added later
#[derive(Serialize, Clone, Copy)]
pub struct Message {

    action: MessageAction,

    /* bring the BigArray macro into scope for serialization of custom length array;
       the biggest information to send through a message is the map of 400 bytes long,
       this is why the data buffer of the message is 400 bytes */
    #[serde(with = "BigArray")]
    data: [u8; 400]
}

impl Message {

    /// Constructor
    ///
    /// # Args:
    ///
    /// `data` - the 400 bytes of data to send
    pub fn new(
        action: MessageAction,
        data: [u8; 400]
    ) -> Self {
        Message {
            action: action,
            data: data
        }
    }
}
