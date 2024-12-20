use std::fmt::Debug;


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
    pub fn get_tile_id(&self) -> i32 {
        match self {
           Tile::EMPTY => 0,
           Tile::DeadSnake => -2,
           Tile::FOOD => -1,
           Tile::Snake { id } => *id + 10,
        }
    }
} 



