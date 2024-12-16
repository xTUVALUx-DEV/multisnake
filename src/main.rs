use std::{thread::sleep, time::Duration};

use base_snake::{scenes::connect::{add_players, connection_screen}, scoreboard::{self, Scoreboard}};
use macroquad::prelude::*;
pub mod base_snake;
use base_snake::{snakegrid::SnakeGrid};



#[macroquad::main("MultiSnake")]
async fn main() {

    let snake_controller_settings = add_players().await;

    let mut scoreboard: Scoreboard = Scoreboard::new(snake_controller_settings.len() as i32);
    
    loop {

        let mut snake_controllers = snake_controller_settings.iter().map(|x| (**x).clone_weak()).collect();
        connection_screen(&mut snake_controllers).await;
        
        sleep(Duration::from_secs_f32(0.5));

        let mut game_grid: SnakeGrid = SnakeGrid::new(20, 18);
        snake_controllers.iter_mut().for_each(|x| { game_grid.add_snake(x.as_mut()); } );

        game_grid.start_game();
        game_grid.do_place_food();

        game_grid.draw();

        sleep(Duration::from_secs(1));

        let mut winner = None;

        loop {
            clear_background(RED);

            game_grid.update_input();
            game_grid.tick();

            clear_background(BLACK);
            game_grid.draw();
            
            next_frame().await;

            game_grid.draw();
            scoreboard.draw_widget();
            
            match game_grid.check_end() {
                Ok(snake_data) => {
                    draw_end_message(&format!("{} Won!", snake_data.name)).await;
                    winner = Some(snake_data.clone());
                    break
                },
                Err(0) => {
                    if let Some(best_snake) = game_grid.get_all_snake_refs().iter().max_by_key(|item| item.size) {
                        println!("Hightest points");
                        draw_end_message(&format!("{} Won!", best_snake.name)).await;
                        winner = Some(best_snake.clone());
                        break;
                    }
                    draw_end_message(&format!("Tie!")).await;
                    break;
                },
                _ => ()
            }

            game_grid.send_gamestate();
            let now = Instance::now();
            while now.elapsed() < Duration::from_secs_f32(0.15) {
                game_grid.draw();
                scoreboard.draw_widget();
            }
        }


        snake_controllers.iter_mut().for_each(|x| x.send_winner(winner.as_ref().expect("No winner? How did we get here??").id));
        scoreboard.add_win(&winner.unwrap());

        sleep(Duration::from_secs_f32(0.3));

        snake_controllers.iter_mut().for_each(|x| x.disconnect());

        sleep(Duration::from_secs_f32(0.05));
    }
}

async fn draw_end_message(message: &str){
    let (height, width) = (screen_height(), screen_width());
    draw_text(message, width*0.2, height*0.4, 60.0, WHITE);
    next_frame().await;
    sleep(Duration::from_secs(2));
}
