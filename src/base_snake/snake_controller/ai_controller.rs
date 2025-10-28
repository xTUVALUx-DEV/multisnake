use std::ptr;
use std::sync::Arc;
use std::fmt::Debug;
use crate::base_snake::snake::{Direction, PlayerInfo, SnakeController, SnakeData};

#[cfg(target_os = "windows")]
mod platform_imports {
    pub use windows::core::PCSTR;
    pub use windows::Win32::Foundation::{CloseHandle, HANDLE};
    pub use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX};
    pub use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeA, PeekNamedPipe, PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT};
    pub use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED}; 
    pub use windows::core::s;

}
#[cfg(target_os = "linux")]
mod platform_imports {
    pub use std::os::unix::net::{UnixListener, UnixStream};
    pub use std::io::{Read, Write};
}

use platform_imports::*;

// WINDOWS IMPLEMENTATION

#[cfg(target_os = "windows")]
pub struct PipeController {
    pipe: Option<HANDLE>,
    pipe_name: PCSTR,
    direction: Direction,
    ai_name: String,
    missed_inputs: i32,
    marked_cells: Vec<u16>,
    pending_writes: Vec<(Arc<Vec<u8>>, OVERLAPPED)>,
}

#[cfg(target_os = "windows")]
impl PipeController {
    pub fn new(pipe_index: usize) -> Self {

        const pipe_names: [PCSTR; 12] = [
            s!(r"\\.\pipe\SnakePipe1"),
            s!(r"\\.\pipe\SnakePipe2"),
            s!(r"\\.\pipe\SnakePipe3"),
            s!(r"\\.\pipe\SnakePipe4"),
            s!(r"\\.\pipe\SnakePipe5"),
            s!(r"\\.\pipe\SnakePipe6"),
            s!(r"\\.\pipe\SnakePipe7"),
            s!(r"\\.\pipe\SnakePipe8"),
            s!(r"\\.\pipe\SnakePipe9"),
            s!(r"\\.\pipe\SnakePipe10"),
            s!(r"\\.\pipe\SnakePipe11"),
            s!(r"\\.\pipe\SnakePipe12"),
        ];

        Self { direction: Direction::RIGHT, pipe: None, pipe_name: pipe_names[pipe_index], ai_name: "Unknown Ai".to_string(), missed_inputs: 0, marked_cells: Vec::new(), pending_writes: Vec::new() }
    }

    fn is_connected(&self) -> bool {
        self.pipe.is_some()
    }
    pub fn check_write_completion(&mut self) {
        self.pending_writes.retain_mut(|(_, mut overlapped)| {
            let mut bytes_transferred = 0;

            unsafe {
                let result = GetOverlappedResult(
                    self.pipe.unwrap(),
                    &mut overlapped,
                    &mut bytes_transferred,
                    false,
                );
                println!("Result: {:?}", result);
                if result.is_err() {
                    return true;
                }
                else {
                    return false;
                }
            }
        });
    }

}

#[cfg(target_os = "windows")]
impl SnakeController for PipeController {
    fn clone_weak(&self) -> Box<(dyn SnakeController)> {
        Box::new(PipeController { pipe: None, pipe_name: self.pipe_name, direction: self.direction, ai_name: self.ai_name.clone(), missed_inputs: 0, marked_cells: Vec::new(), pending_writes: Vec::new() })
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
            self.missed_inputs += 1;
            println!("{}", self.missed_inputs);
            return;
        }

        let mut buffer = vec![0u8; available_bytes as usize];
        let mut bytes_read = 0;
        // Read the available data from the pipe
        let _ = unsafe {
            ReadFile(self.pipe.unwrap(), Some(&mut buffer), Some(&mut bytes_read), None)
        };
        println!(" B {:?}", buffer);
        while !buffer.is_empty() {
            let a = buffer.remove(0);
            println!("PacketId {}", a);
            match a {
                10 => { self.direction = Direction::UP; println!("UP")},
                11 => { self.direction = Direction::DOWN; println!("DOWN")},
                12 => { self.direction = Direction::LEFT; println!("LEFT")},
                13 => { self.direction = Direction::RIGHT; println!("RIGHT")},
                20 if buffer.len() >= 2 => {
                    println!("InfoPacket");
                    let bytes: Vec<u8> = buffer.drain(0..2).collect();
                    let length = u16::from_le_bytes([bytes[0], bytes[1]]);
                    println!("C {}", length);
                    self.marked_cells = (0..length).map(|i| {
                        let bytes: Vec<u8> = buffer.drain(0..2).collect();
                        u16::from_le_bytes([bytes[0], bytes[1]])
                    }).collect();
                    //println!("A {:?}", self.marked_cells);
                    
                }
                _ => println!("Invalid Message"),    
            }
            
            if *(buffer.first().unwrap_or(&0)) == 0 {
                break;
            }
        }

    }
    fn report_data(&mut self, data: SnakeData, snake_id: i32) {
        if !self.is_connected() {
            return;
        }
        let pipe = match self.pipe {
            Some(ref pipe) if !pipe.is_invalid() => *pipe,
            _ => {
                eprintln!("Invalid pipe handle");
                return;
            }
        };


        let buffer = Arc::new(data.encode(snake_id).to_vec());
        let buffer_ptr = buffer.as_ptr(); 
        let mut overlapped = OVERLAPPED::default();
        unsafe {
            let _ = WriteFile(pipe, Some(std::slice::from_raw_parts(buffer_ptr, buffer.len())), Some(&mut (buffer.len() as u32)), Some(&mut overlapped));
        };
        // Storing the buffer for it to not be dropped
        self.check_write_completion();
        self.pending_writes.push((buffer, overlapped));

    }
    fn send_winner(&mut self, winner_id: i32) {
        if !self.is_connected() {
            return;
        }

        let mut buffer = Vec::from([2]); // PacketId + WinnerId
        buffer.extend((winner_id as i32).to_le_bytes());
        let buff_ptr = Arc::new(buffer);

        let mut overlapped = OVERLAPPED::default();
        unsafe {
            let _ = WriteFile(self.pipe.unwrap(), Some(std::slice::from_raw_parts(buff_ptr.as_ptr(), buff_ptr.len())), Some(&mut (buff_ptr.len() as u32)), Some(&mut overlapped));
        };

        self.pending_writes.push((buff_ptr, overlapped));

    }
    fn connect(&mut self) -> bool {
        unsafe {
            let pipe = CreateNamedPipeA(
                self.pipe_name,
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                5044,
                5044,
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
    fn get_info(&self) -> Option<PlayerInfo> {
        Some(PlayerInfo {
            marked_cells: self.marked_cells.clone(),
            info_lines: vec![
                format!("Missed Inputs: {}", self.missed_inputs),
                format!("Current Direction: {}", self.direction.to_string())
            ]
        })
    }
}

#[cfg(target_os = "windows")]
impl Debug for PipeController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipeController").field("pipe", &self.pipe).field("pipe_name", &self.pipe_name).field("direction", &self.direction).field("ai_name", &self.ai_name).field("missed_inputs", &self.missed_inputs).field("marked_cells", &self.marked_cells).finish()
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct UnixSocketController {
    direction: Direction,
    ai_name: String,
    missed_inputs: i32,
    marked_cells: Vec<u16>,
    socket_id: i32,
    stream: Option<UnixStream>,
    listener: Option<UnixListener>


}

#[cfg(target_os = "linux")]
impl UnixSocketController {
    pub fn new(socket_id: i32) -> Self {
        Self { direction: Direction::RIGHT, socket_id, ai_name: "Unknown Ai".to_string(), missed_inputs: 0, marked_cells: Vec::new(), stream: None, listener: None }
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}

#[cfg(target_os = "linux")]
impl SnakeController for UnixSocketController {
    fn report_data(&mut self, data: SnakeData, snake_id: i32) {
        if !self.is_connected() {
            return;
        }

        let buffer = Arc::new(data.encode(snake_id).to_vec());
        self.stream.as_ref().unwrap().write_all(buffer.as_slice());
    }
    fn send_winner(&mut self, winner_id: i32) {
        if !self.is_connected() {
            return;
        }

        let mut buffer = Vec::from([2]); // PacketId + WinnerId
        buffer.extend((winner_id as i32).to_le_bytes());

        self.stream.as_ref().unwrap().write_all(buffer.as_slice());
    }

    fn connect(&mut self) -> bool { 
        let socket_path = format!("/tmp/multisnake{}.sock", self.socket_id+1);
        std::fs::remove_file(&socket_path).ok();

        self.listener = UnixListener::bind(&socket_path).ok();
        let (mut stream, _addr) = self.listener.as_ref().unwrap().accept().unwrap();

        self.stream = Some(stream);

        println!(" {:?}", self.stream);

        // Read Name
        let mut buffer = [0u8; 216];

        if let Some(stream) = &mut self.stream {

            stream.set_nonblocking(false);
            println!("Waiting");
            let bytes_read = stream.read(&mut buffer).unwrap();
            println!("Connection {}", bytes_read);

            self.ai_name = String::from_utf8_lossy(&buffer[..bytes_read as usize]).to_string();
            
            stream.set_nonblocking(true);
        }

        true

    } 
    fn disconnect(&self) {}
    fn update(&mut self) {
        if !self.is_connected() {
            return;
        }

        let mut buffer = vec![0u8; 256];
        match self.stream.as_ref().unwrap().read(&mut buffer) {
            Ok(n) => {
                while !buffer.is_empty() {
                    let a = buffer.remove(0);
                    println!("PacketId {}", a);
                    match a {
                        10 => { self.direction = Direction::UP; println!("UP")},
                        11 => { self.direction = Direction::DOWN; println!("DOWN")},
                        12 => { self.direction = Direction::LEFT; println!("LEFT")},
                        13 => { self.direction = Direction::RIGHT; println!("RIGHT")},
                        20 if buffer.len() >= 2 => {
                            println!("InfoPacket");
                            let bytes: Vec<u8> = buffer.drain(0..2).collect();
                            let length = u16::from_le_bytes([bytes[0], bytes[1]]);
                            println!("C {}", length);
                            self.marked_cells = (0..length).map(|i| {
                                let bytes: Vec<u8> = buffer.drain(0..2).collect();
                                u16::from_le_bytes([bytes[0], bytes[1]])
                            }).collect();
                            //println!("A {:?}", self.marked_cells);
                            
                        }
                        _ => println!("Invalid Message"),    
                    }
                    
                    if *(buffer.first().unwrap_or(&0)) == 0 {
                        break;
                    }
                }
            }
            Err (ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return;
            }
            Err(e) => {
                println!("Some error reading socket!");
            }
        }
        println!(" B {:?}", buffer);

    }

    fn clone_weak(&self) -> Box<dyn SnakeController> {
        let mut newObj = UnixSocketController { direction: self.direction, socket_id: self.socket_id, ai_name: self.ai_name.clone(), missed_inputs: 0, marked_cells: Vec::new(), stream: None, listener: None };
        Box::new(newObj)
    }
    fn get_info(&self) -> Option<PlayerInfo> {
        Some(PlayerInfo {
            marked_cells: self.marked_cells.clone(),
            info_lines: vec![
                format!("Missed Inputs: {}", self.missed_inputs),
                format!("Current Direction: {}", self.direction.to_string())
            ]
        })
    }

    fn get_name(&self) -> String {
        self.ai_name.clone()
    }

    fn next_direction(&self) -> Direction {
        self.direction
    }

 
}


