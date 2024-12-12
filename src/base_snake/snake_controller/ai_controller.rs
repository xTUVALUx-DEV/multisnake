use macroquad::{input::KeyCode};
use macroquad::prelude::is_key_pressed;
use crate::base_snake::snake::{Direction, SnakeController};

#[derive(Debug)]
pub struct PipeController {
    pipe: HANDLE,
    direction: Direction

}
impl PipeController {
    pub fn new(pipe: HANDLE) -> Self {
        Self { direction: Direction::RIGHT, pipe }
    }

}

impl SnakeController for PipeController {
    fn next_direction(&self) -> Direction {
        self.direction
    }
    fn update(&mut self) {
        let mut buffer = [0u8; 1024];

        // Read the message from the client
        let bytes_read = unsafe {
            let mut read_bytes = 0;
            ReadFile(pipe, buffer.as_mut_ptr() as *mut _, buffer.len() as u32, &mut read_bytes, std::ptr::null_mut());
            read_bytes
        };

        let message = String::from_utf8_lossy(&buffer[..bytes_read as usize]);

        println!("Recieved: {}", message);

    }
    fn report_data(&self, data: &crate::base_snake::snake::SnakeData) {
        
    }
}





