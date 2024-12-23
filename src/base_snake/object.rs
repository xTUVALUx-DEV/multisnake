use std::fmt::Debug;


#[derive(Clone, Debug, PartialEq)]
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
    pub fn from_tile_id(id: i16) -> Self {
        match id {
            -2 => Tile::DeadSnake,
            -1 => Tile::FOOD,
            0 => Tile::EMPTY,
            _ => Tile::Snake { id: (id-10) as i32 }
        }
    }
} 



