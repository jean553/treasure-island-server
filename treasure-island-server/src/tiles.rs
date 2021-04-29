//! Contains the tiles routines. 

use crate::sprites::get_sprite_index_from_tile_value;

use rand::thread_rng;
use rand::Rng;

use std::collections::VecDeque;

/// Refactored code that loads all tiles of the map randomly. Only used once but refactored here for readability. 
pub fn load_tiles() -> [u8; 400] {

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

    return tiles;
}
