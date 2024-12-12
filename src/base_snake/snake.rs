use std::fmt::Debug;


pub struct SnakeData {}

#[derive(Debug)]
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

impl<'a> Snake<'a> {
    pub fn new(id: i32, controller: &'a mut dyn SnakeController) -> Self {
        Self { id, tiles: Vec::new(), controller, max_size: 1, state: SnakeState::ALIVE }
    }
    pub fn update_controller(&mut self) {
        self.controller.update();
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
}

pub trait SnakeController : Debug {
    fn report_data(&self, data: &SnakeData) {}
    fn update(&mut self) {}
    fn next_direction(&self) -> Direction;
}