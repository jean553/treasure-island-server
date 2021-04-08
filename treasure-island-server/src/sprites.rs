//! Some routines specific to the sprites.

/// Refactored code that returns a sprite image index according to a given tile value.
///
/// # Args:
///
/// `tile_value` - the value of the tile to convert into sprite index
pub fn get_sprite_index_from_tile_value(tile_value: u8) -> u8 {

    const WATER_TILE_VALUE: u8 = 0;
    const FIRST_SAND_TILE_VALUE: u8 = 1;
    const SECOND_SAND_TILE_VALUE: u8 = 2;
    const TREE_TILE_VALUE: u8 = 3;

    const WATER_SPRITE_INDEX: u8 = 10;
    const SAND_SPRITE_INDEX: u8 = 0;
    const TREE_SPRITE_INDEX: u8 = 8;

    return match tile_value {
        WATER_TILE_VALUE => WATER_SPRITE_INDEX,
        FIRST_SAND_TILE_VALUE | SECOND_SAND_TILE_VALUE => SAND_SPRITE_INDEX,
        TREE_TILE_VALUE => TREE_SPRITE_INDEX,
        _ => panic!("no sprite index matches with tile value {}", tile_value)
    };
}
