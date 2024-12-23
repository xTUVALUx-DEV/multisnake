use std::{thread::sleep, time::{Duration, Instant}};

use multisnake::base_snake::{scenes::connect::{add_players, connection_screen, GameConfig}, scoreboard::{self, Scoreboard}};
use macroquad::prelude::*;
use multisnake::base_snake::snakegrid::SnakeGrid;

fn window_conf() -> Conf {
    Conf {
        window_title: "MultiSnake".to_owned(),
        window_width: 1200,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let GameConfig { snake_controller_list, grid_size: (grid_x, grid_y), sandbox } = add_players().await;

    let mut scoreboard: Scoreboard = Scoreboard::new(snake_controller_list.len() as i32);
    
    loop {

        let mut snake_controllers = snake_controller_list.iter().map(|x| (**x).clone_weak()).collect();
        connection_screen(&mut snake_controllers).await;
        sleep(Duration::from_secs_f32(0.5));

        let mut game_grid: SnakeGrid = SnakeGrid::new(grid_x, grid_y);
        snake_controllers.iter_mut().for_each(|x| { game_grid.add_snake(x.as_mut()); } );

        game_grid.start_game();  // Initialize all the Snakes (Spawnpoints)
        game_grid.do_place_food();

        scoreboard.initalize(game_grid.get_all_snake_refs()); // Draw the initial scoreboard now because the names and ids are unknown beforehand

        game_grid.draw();

        sleep(Duration::from_secs_f32(0.2));

        let mut winner = None;
        let start_time = Instant::now();

        loop {
            clear_background(BLACK);
            
            game_grid.update_input();
            game_grid.tick();

            game_grid.draw();
            scoreboard.draw_widget(game_grid.get_info_dict());

            next_frame().await;

            
            match game_grid.check_end() {
                Ok(snake_data) => {
                    if !sandbox {
                        game_grid.draw();
                        draw_end_message(&format!("{} Won!", snake_data.name)).await;
                        winner = Some(snake_data.clone());
                        break
                    }
                },
                Err(1) if (start_time.elapsed() < Duration::from_secs(60) || sandbox) => {} // Ongoing
                _ => {  // Err 0 or timeout
                    if let Some(best_snake) = game_grid.get_all_snake_refs().iter().filter(|x| x.alive).max_by_key(|item| item.size) {
                        if !sandbox {
                            game_grid.draw();
                            draw_end_message(&format!("{} Won!", best_snake.name)).await;
                            winner = Some(best_snake.clone());
                        }
                        break;
                    }
                    draw_end_message(&format!("Tie!")).await;
                    break;
                },
            }

            game_grid.send_gamestate();
            let now = Instant::now();
            while now.elapsed() < Duration::from_secs_f32(0.15) {
                game_grid.draw();
                scoreboard.draw_widget(game_grid.get_info_dict());
                next_frame().await;
            }
        }

        if !sandbox {
            if winner.is_some() {
                snake_controllers.iter_mut().for_each(|x| x.send_winner(winner.as_ref().expect("No winner? How did we get here??").id));
                scoreboard.add_win(&winner.unwrap());
            }
        }

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
