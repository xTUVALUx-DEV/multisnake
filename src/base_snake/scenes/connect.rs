use std::{panic, sync::{Arc, Mutex}, thread};

use macroquad::prelude::*;
use windows::core::{s, PCSTR};

use crate::base_snake::{snake::SnakeController, snake_controller::{ai_controller::PipeController, keyboard_controller::KeyboardController}};

pub fn draw_player_names(names: &Vec<String>) {
    clear_background(RED);
    draw_text("Connecting", 20.0, 40.0, 40.0, WHITE);
    names.iter().enumerate().for_each(|(i, x)| {
            draw_text(&format!("{}: Connected!", x), 60.0, 70.0+30.*i as f32, 30.0, WHITE);
    });

}
pub async fn connection_screen(players: &mut Vec<Box<dyn SnakeController>>) {

    clear_background(RED);

    draw_text("Connecting", 20.0, 40.0, 40.0, WHITE);
    
    next_frame().await;

    let mut connected_players: Vec<String> = Vec::new();
    for (i, player) in players.iter_mut().enumerate(){ 
        if player.connect() {
            connected_players.push(player.get_name());
            draw_player_names(&connected_players);
            next_frame().await;
        } else {
            println!("Unable to connect {}", player.get_name());

            connected_players.push("<Connection Error>".to_owned());
            draw_player_names(&connected_players);
            next_frame().await;
        }
    }
}


pub async  fn add_players() -> Vec<Box<dyn SnakeController>> {
    let mut snake_controllers: Vec<Box<dyn SnakeController>> = Vec::new(); 
    let mut current_pipe_index = 0;

    let pipe_names: [PCSTR; 4] = [
        s!(r"\\.\pipe\SnakePipe1"),
        s!(r"\\.\pipe\SnakePipe2"),
        s!(r"\\.\pipe\SnakePipe3"),
        s!(r"\\.\pipe\SnakePipe4"),
    ];


    loop {

        clear_background(RED);

        draw_text("P - Spieler Hinzufügen", 20.0, 50.0, 30.0, WHITE);

        draw_text("O - Ai Hinzufügen", 20.0, 70.0, 30.0, WHITE);
        
        draw_text("Drücke <Enter> um zum Starten", 140., 500.0, 40.0, WHITE);

        if is_key_pressed(KeyCode::P) {
            snake_controllers.push(Box::new(KeyboardController::new()));
            println!("Added Player");
        }
        if is_key_pressed(KeyCode::O) {
            snake_controllers.push(Box::new(PipeController::new(pipe_names[current_pipe_index])));
            current_pipe_index += 1;
            println!("Added Ai");
        }
        if is_key_down(KeyCode::Enter) {
            break;
        }

        snake_controllers.iter().enumerate().for_each(|(i, x)| { 
            draw_text(&format!("> {}", x.get_name()), 20.0, 90.0 + 20.*i as f32, 20.0, WHITE);
        });

        next_frame().await;        
    }

    snake_controllers
}
