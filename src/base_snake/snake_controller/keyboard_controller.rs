use macroquad::input::is_key_down;
use macroquad::input::KeyCode;
use crate::base_snake::snake::{Direction, SnakeController};

#[derive(Debug)]
pub struct KeyboardController {
    direction: Direction,
    up_input: KeyCode,
    down_input: KeyCode,
    left_input: KeyCode,
    right_input: KeyCode,
}
impl KeyboardController {
    pub fn arrows() -> Self {
        Self { direction: Direction::RIGHT, up_input: KeyCode::Up, down_input: KeyCode::Down, left_input: KeyCode::Left, right_input: KeyCode::Right }
    }
    pub fn wasd() -> Self {
        Self { direction: Direction::RIGHT, up_input: KeyCode::W, down_input: KeyCode::S, left_input: KeyCode::A, right_input: KeyCode::D }
    }

}

impl SnakeController for KeyboardController {
    fn next_direction(&self) -> Direction {
        self.direction
    }
    fn update(&mut self) {
        if is_key_down(self.up_input) && self.direction != Direction::DOWN {
            self.direction = Direction::UP;
        }
        else if is_key_down(self.down_input) && self.direction != Direction::UP {
            self.direction = Direction::DOWN;
        }
        else if is_key_down(self.left_input) && self.direction != Direction::RIGHT {
            self.direction = Direction::LEFT;
        }
        else if is_key_down(self.right_input) && self.direction != Direction::LEFT {
            self.direction = Direction::RIGHT;
        }
    }
    fn get_name(&self) -> String {
        format!("Player {:?}/{:?}/{:?}/{:?}", self.up_input, self.left_input, self.down_input, self.right_input)
    }

    fn clone_weak(&self) -> Box<(dyn SnakeController)> {
        Box::new(KeyboardController { direction: self.direction, up_input: self.up_input, down_input: self.down_input, left_input: self.left_input, right_input: self.right_input })
    }
}




