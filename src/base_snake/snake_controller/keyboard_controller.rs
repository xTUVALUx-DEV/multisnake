use macroquad::{input::KeyCode};
use macroquad::prelude::is_key_pressed;
use crate::base_snake::snake::{Direction, SnakeController};

#[derive(Debug)]
pub struct KeyboardController {
    direction: Direction

}
impl KeyboardController {
    pub fn new() -> Self {
        Self { direction: Direction::RIGHT }
    }

}

impl SnakeController for KeyboardController {
    fn next_direction(&self) -> Direction {
        self.direction
    }
    fn update(&mut self) {
        if is_key_pressed(KeyCode::Up) {
            self.direction = Direction::UP;
        }
        if is_key_pressed(KeyCode::Down) {
            self.direction = Direction::DOWN;
        }
        if is_key_pressed(KeyCode::Left) {
            self.direction = Direction::LEFT;
        }
        if is_key_pressed(KeyCode::Right) {
            self.direction = Direction::RIGHT;
        }
        println!("Direction: {:?}", self.direction);
    }
}




