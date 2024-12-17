use macroquad::input::is_key_down;
use macroquad::input::KeyCode;
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
        if is_key_down(KeyCode::Up) {
            self.direction = Direction::UP;
        }
        if is_key_down(KeyCode::Down) {
            self.direction = Direction::DOWN;
        }
        if is_key_down(KeyCode::Left) {
            self.direction = Direction::LEFT;
        }
        if is_key_down(KeyCode::Right) {
            self.direction = Direction::RIGHT;
        }
    }
    fn get_name(&self) -> String {
        "Keyboard-Input".to_string()
    }

    fn clone_weak(&self) -> Box<(dyn SnakeController)> {
        Box::new(KeyboardController::new())
    }
}




