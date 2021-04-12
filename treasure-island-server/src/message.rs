//! Contains the Message structure, sent through streams between server and client.

use serde_derive::Serialize;

/* generates the function from the macro that is responsible
   to serialize arrays longer than 32 values */
big_array! {
    BigArray;
    400
}

#[derive(Serialize, Clone, Copy)]
pub struct Message {

    /* we do not use enums to send actions because:
       - this is a "raw network" information to be handled both on client and server,
         having an enumeration may require to maintain same version of the structure
         from both side,
       - enumerations might take a padded space in memory according to the architecture,
         but we have to be sure we only uses one bit,
       - enumerations are tagged unions, so they may take a few bytes for the type too */
    action: u8,

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
    /// `action` - the action of the message
    pub fn new(action: u8) -> Self {
        Message {
            action: action,
            data: [0; 400]
        }
    }

    /// Sets the data to send (400 bytes).
    ///
    /// # Args:
    ///
    /// `data` - the data to send
    pub fn set_data(
        &mut self,
        data: [u8; 400],
    ) {
        self.data = data;
    }
}
