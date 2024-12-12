use std::fmt::Debug;

use super::snake::Snake;


const FOOD_TILE_ID: i32 = 1;
 

#[derive(Clone, Debug)]
pub enum Tile {
    EMPTY,
    FOOD,
    Snake {
        id: i32,
    },
    DeadSnake
}


impl Tile {

} 



