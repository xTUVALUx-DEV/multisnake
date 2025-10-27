use std::{thread::sleep, time::{Duration, Instant}};

use macroquad::{input::{is_key_pressed, is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton}, miniquad::graphics, window::next_frame};

use crate::base_snake::snakegrid::{SnakeGrid, GRID_OFFSET_X, GRID_OFFSET_Y};

fn get_clicked_square(game_grid: &SnakeGrid<'_>) -> (i32, i32) {
    let mut pos: (f32, f32) = mouse_position();
    pos.0 -= GRID_OFFSET_X;
    pos.1 -= GRID_OFFSET_Y;

    ((pos.0 / game_grid.total_square_size) as i32, (pos.1 / game_grid.total_square_size) as i32)
}

pub async fn snake_draw(mut game_grid: SnakeGrid<'_>) {
    game_grid.send_gamestate();
    
    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            let (x,y) = get_clicked_square(&game_grid);
            game_grid.set_square((x + y*game_grid.width) as usize, Some(0));
            game_grid.send_gamestate();
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            let (x,y) = get_clicked_square(&game_grid);
            game_grid.set_square((x + y*game_grid.width) as usize, Some(1));
            game_grid.send_gamestate();
        }
        if is_mouse_button_pressed(MouseButton::Middle) {
            let (x,y) = get_clicked_square(&game_grid);
            game_grid.set_square((x + y*game_grid.width) as usize, None);
            game_grid.send_gamestate();
        }
        
        game_grid.update_input();
        
        if is_key_pressed(macroquad::input::KeyCode::N) {
            game_grid.tick();
            game_grid.send_gamestate();
        }
        
        if is_key_pressed(macroquad::input::KeyCode::R) {
            game_grid.reconnect();
            game_grid.send_gamestate();
        }

        if is_key_pressed(macroquad::input::KeyCode::C) {
            game_grid.clear();
        }
    
        game_grid.draw();
        next_frame().await;
        sleep(Duration::from_millis(300));
    }
}   

