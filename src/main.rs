use std::ffi::CString;
use std::os::windows;
use std::ptr;
use std::{thread::sleep, time::Duration};

use macroquad::prelude::*;
pub mod base_snake;
use base_snake::{snake_controller::keyboard_controller::KeyboardController, snakegrid::SnakeGrid};
use winapi::um::winbase::{CreateNamedPipeA, PIPE_ACCESS_DUPLEX};
use ::windows::Win32::Foundation::*;
use ::windows::Win32::System::Pipes::*;
use ::windows::Win32::System::IO::*;
use ::windows::core::*;

const PIPE_NAME: &str = r"\\.\pipe\SnakePipe";

#[macroquad::main("MyGame")]
async fn main() {
    let mut game_grid: SnakeGrid = SnakeGrid::new(10, 10);

    let mut player_snake_controller = KeyboardController::new();


    loop {

        draw_text("P - Spieler Hinzufügen", 20.0, 20.0, 30.0, BLUE);

        draw_text("P - Spieler Hinzufügen", 20.0, 80.0, 30.0, BLUE);
        
            println!("Creating named pipe...");


            let pipe: *mut winapi::ctypes::c_void = unsafe {
                CreateNamedPipeA(
                    CString::new(PIPE_NAME).expect("Failed to create the string").as_ptr(),
                    PIPE_ACCESS_DUPLEX,
                    (PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT).0,
                    1,
                    64 * 1024,
                    64 * 1024,
                    0,
                    ptr::null_mut(),
                )
            };

            if pipe == INVALID_HANDLE_VALUE {
                panic!("Failed to create named pipe");
            }

            println!("Waiting for client connection...");
            let connected = unsafe { ConnectNamedPipe(pipe, Some(ptr::null_mut())) };
            if connected.as_bool() {
                println!("Client connected!");
                handle_client(pipe)?;
            } else {
                unsafe { CloseHandle(pipe) };
                println!("Failed to connect client.");
            }

    }

    game_grid.add_snake(&mut player_snake_controller);    
    game_grid.start_game();
    game_grid.do_place_food();

    loop {
        clear_background(RED);

 
        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);
        game_grid.update_input();
        game_grid.tick();
        game_grid.draw();
        
        next_frame().await;
        sleep(Duration::from_secs(1));
    }
}
        