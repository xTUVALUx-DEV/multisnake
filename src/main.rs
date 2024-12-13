use std::ffi::CString;
use std::os::windows;
use std::ptr;
use std::{thread::sleep, time::Duration};

use base_snake::scenes::connect::{add_players, connection_screen};
use base_snake::snake::SnakeController;
use base_snake::snake_controller;
use base_snake::snake_controller::ai_controller::PipeController;
use macroquad::prelude::*;
pub mod base_snake;
use base_snake::{snakegrid::SnakeGrid};



#[macroquad::main("MyGame")]
async fn main() {

    let snake_controller_settings = add_players().await;

    loop {
        let mut snake_controllers = snake_controller_settings.iter().map(|x| (**x).clone_weak()).collect();
        connection_screen(&mut snake_controllers).await;
        
        sleep(Duration::from_secs(1));


        let mut game_grid: SnakeGrid = SnakeGrid::new(20, 18);
        snake_controllers.iter_mut().for_each(|x| { game_grid.add_snake(x.as_mut()); } );
        //for snake_controller in snake_controllers {
        //    game_grid.add_snake(snake_controller);
        //}

        game_grid.start_game();
        game_grid.do_place_food();

        game_grid.draw();

        sleep(Duration::from_secs(1));

        loop {
            clear_background(RED);

            draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);
            game_grid.update_input();
            game_grid.tick();

            clear_background(BLACK);
            game_grid.draw();
            
            next_frame().await;
            
            match game_grid.check_end() {
                Ok(snake_data) => {
                    draw_end_message(&format!("{} Won!", snake_data.name)).await;
                    break
                },
                Err(0) => {
                    // Todo: Make the person win whos the longest
                    draw_end_message(&format!("Tie!")).await;
                    break
                },
                _ => ()
            }
    
            game_grid.send_gamestate();
            sleep(Duration::from_secs_f32(0.5));
        }

        snake_controllers.iter_mut().for_each(|x| x.disconnect());
    }
}

async fn draw_end_message(message: &str){
    let (height, width) = (screen_height(), screen_width());
    draw_text(message, width*0.3, height*0.4, 60.0, WHITE);
    next_frame().await;
    sleep(Duration::from_secs(2));
}
