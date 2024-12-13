use std::fmt::Debug;

use windows::core::PWSTR;

use super::object::Tile;


pub struct SnakeData<'a> {
    pub height: u16,
    pub width: u16,
    pub grid: &'a Vec<Tile>,
    pub snakes: Vec<SnakeRefData>
}

impl<'a> SnakeData<'a> {
    pub fn encode(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        // push the height, width and snakes attrivutes to the u8 vec
        data.extend(&self.height.to_le_bytes());
        data.extend(&self.width.to_le_bytes());

        // Encode the grid by pushing each tile's ID
        for tile in self.grid {
            data.extend((tile.get_tile_id() as i16).to_le_bytes());
        }

        // Encode the number of snakes (as u32 for safety)
        let snake_count = self.snakes.len() as u16;
        data.extend(snake_count.to_le_bytes());

        // Encode each snake
        for snake in &self.snakes {
            // Encode the snake ID (i32 -> 4 bytes, in little-endian order)
            data.extend(&(snake.id as i16).to_le_bytes());

            // Encode the snake's name length and content
            let name_bytes = snake.name.as_bytes();
            let name_length = name_bytes.len() as u16; // Use u16 for name length
            data.extend(name_length.to_le_bytes());
            data.extend(name_bytes);

            // Encode the alive status (as a single byte: 1 for true, 0 for false)
            data.push(if snake.alive { 1 } else { 0 });
        }

        data

    }
}

#[derive(Debug, Clone)]
pub enum SnakeState {
    ALIVE,
    DEAD
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NONE
}


#[derive(Debug)]
pub struct Snake<'a> {
    id: i32,
    tiles: Vec<i32>,
    max_size: i32,
    controller: &'a mut dyn SnakeController,
    state: SnakeState
}

pub struct SnakeRefData {
    id: i32,
    pub name: String,
    alive: bool
}


impl<'a> Snake<'a> {
    pub fn new(id: i32, controller: &'a mut dyn SnakeController) -> Self {
        Self { id, tiles: Vec::new(), controller, max_size: 1, state: SnakeState::ALIVE }
    }
    pub fn update_controller(&mut self) {
        self.controller.update();
    }
    pub fn get_name(&self) -> String {
        self.controller.get_name()
    }
    pub fn send_gamestate(&self, data: SnakeData) {
        self.controller.report_data(data);
    }
    pub fn next_direction(&self) -> Direction {
         if let SnakeState::DEAD = self.state {
            return Direction::NONE;
        }        

        self.controller.next_direction()
    }
    pub fn get_head(&self) -> i32 {
        *self.tiles.first().expect("Tried to access uninitalized Snake")
    }
    pub fn get_tiles(&self) -> &Vec<i32> {
        &self.tiles
    }
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn is_dead(&self) -> bool {
        match self.state {
            SnakeState::ALIVE => false,
            SnakeState::DEAD => true
        }
    }
    pub fn move_head(&mut self, new_head: i32) -> Vec<i32> {
        // Input: New head location Output: Removed indexes
        if let SnakeState::DEAD = self.state {
            return Vec::new();
        }        

        let mut removed = Vec::new();
        self.tiles.insert(0, new_head);
        while self.tiles.len() > self.max_size as usize {
            removed.push(self.tiles.pop().unwrap());
        }
        removed
    }
    pub fn grow(&mut self) {
        self.max_size += 1;
    }
    pub fn kill(&mut self) -> &Vec<i32> {
        // Kills the snake. Returns its tiles.
        self.state = SnakeState::DEAD;
        &self.tiles
    }
    pub fn get_data(&self) -> SnakeRefData {
        SnakeRefData {
            id: self.id,
            name: self.get_name(),
            alive: match self.state {
                SnakeState::ALIVE => true,
                SnakeState::DEAD => false
            }
        }
    }
}

pub trait SnakeController : Debug {
    fn report_data(&self, data: SnakeData) {}
    fn connect(&mut self) -> bool { true } // Only used for ai_controllers
    fn disconnect(&self) {}
    fn get_name(&self) -> String;
    fn update(&mut self) {}
    fn next_direction(&self) -> Direction;
    fn clone_weak(&self) -> Box<dyn SnakeController>;
}