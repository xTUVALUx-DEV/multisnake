use std::ptr;

use windows::core::PCSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeA, PeekNamedPipe, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT};
use windows::Win32::System::IO::OVERLAPPED;
use crate::base_snake::snake::{Direction, SnakeController, SnakeData};

#[derive(Debug)]
pub struct PipeController {
    pipe: Option<HANDLE>,
    pipe_name: PCSTR,
    direction: Direction,
    ai_name: String


}
impl PipeController {
    pub fn new(pipe_name: PCSTR) -> Self {
        Self { direction: Direction::RIGHT, pipe: None, pipe_name, ai_name: "Unknown Ai".to_string() }
    }

    fn is_connected(&self) -> bool {
        self.pipe.is_some()
    }
}

impl SnakeController for PipeController {
    fn clone_weak(&self) -> Box<(dyn SnakeController)> {
        Box::new(PipeController { pipe: None, pipe_name: self.pipe_name, direction: self.direction, ai_name: self.ai_name.clone() })
    }

    fn next_direction(&self) -> Direction {
        self.direction
    }
    
    fn update(&mut self) {
        if !self.is_connected() {
            return;
        }

        let mut available_bytes = 0;
        let peek_result = unsafe {
            PeekNamedPipe(
                self.pipe.unwrap(),
                None,
                0,
                None,
                Some(&mut available_bytes),
                None,
            )
        };
        if peek_result.is_err() || available_bytes <= 0 {
            return;
        }

        let mut buffer = vec![0u8; available_bytes as usize];
        let mut bytes_read = 0;
        // Read the available data from the pipe
        let _ = unsafe {
            ReadFile(self.pipe.unwrap(), Some(&mut buffer), Some(&mut bytes_read), None)
        };
        
        match buffer.last().unwrap() {
            10 => self.direction = Direction::UP,
            11 => self.direction = Direction::DOWN,
            12 => self.direction = Direction::LEFT,
            13 => self.direction = Direction::RIGHT,
                _ => println!("Invalid Message"),    
        }

    }
    fn report_data(&self, data: SnakeData, snake_id: i32) {
        if !self.is_connected() {
            return;
        }

        let buffer = data.encode(snake_id).to_vec();
        
        unsafe {
            let mut overlapped = OVERLAPPED::default();
            let _ = WriteFile(self.pipe.unwrap(), Some(&buffer), Some(&mut (buffer.len() as u32)), Some(&mut overlapped));
        };

    }
    fn send_winner(&self, winner_id: i32) {
        if !self.is_connected() {
            return;
        }

        let mut buffer = Vec::from([2]); // PacketId + WinnerId
        buffer.extend((winner_id as i32).to_le_bytes());

        unsafe {
            let mut overlapped = OVERLAPPED::default();
            let _ = WriteFile(self.pipe.unwrap(), Some(&buffer), Some(&mut (buffer.len() as u32)), Some(&mut overlapped));
        };

    }
    fn connect(&mut self) -> bool {
        unsafe {
            let pipe = CreateNamedPipeA(
                self.pipe_name,
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                2044,
                2044,
                0,
                Some(ptr::null_mut()),
            ).unwrap_or_else(|e| {
                panic!("CreateNamedPipeA Failed With Error: {e}");
            });

            println!("Waiting for connection {:?}", self.pipe_name.to_string());
            if ConnectNamedPipe(pipe, None).is_err() {
                return false;
            }
            self.pipe = Some(pipe);
        }

        // Read Name
        let mut buffer = [0u8; 216];
        let mut bytes_read = buffer.len() as u32;

        unsafe {
            let _ = ReadFile(self.pipe.unwrap(), Some(&mut buffer), Some(&mut bytes_read), None);
        };
        self.ai_name = String::from_utf8_lossy(&buffer[..bytes_read as usize]).to_string();
 
        true
       
    }
    fn disconnect(&self) {
        if !self.is_connected() {
            return;
        } 

        unsafe { let _ = CloseHandle(self.pipe.unwrap()); };
    }
    fn get_name(&self) -> String {
        self.ai_name.clone()
    }
}





