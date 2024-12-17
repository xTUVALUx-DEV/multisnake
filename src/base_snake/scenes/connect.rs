use macroquad::{prelude::*, ui::{hash, root_ui, widgets::{self, Group}}};
use windows::core::{s, PCSTR};

use crate::base_snake::{consts, snake::SnakeController, snake_controller::{ai_controller::PipeController, keyboard_controller::KeyboardController}};

pub struct GameConfig {
    pub snake_controller_list: Vec<Box<dyn SnakeController>>,
    pub grid_size: (i32, i32),
    pub sandbox: bool

}

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
    
    draw_version_hud();
    next_frame().await;

    let mut connected_players: Vec<String> = Vec::new();
    for (_, player) in players.iter_mut().enumerate(){ 
        if player.connect() {
            connected_players.push(player.get_name());
            draw_player_names(&connected_players);
            draw_version_hud();
            next_frame().await;
        } else {
            println!("Unable to connect {}", player.get_name());

            connected_players.push("<Connection Error>".to_owned());
            draw_player_names(&connected_players);
            draw_version_hud();
            next_frame().await;
        }
    }
}



pub async  fn add_players() -> GameConfig {
    let mut snake_controllers: Vec<Box<dyn SnakeController>> = Vec::new(); 
    let mut current_pipe_index = 0;

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
    
    let (mut grid_size_x, mut grid_size_y) = (consts::GRID_SIZE.0.to_string(), consts::GRID_SIZE.1.to_string());
    let mut sandbox = false;

    loop {

        clear_background(RED);

        draw_text("P - Spieler Hinzufügen", 20.0, 50.0, 30.0, WHITE);

        draw_text("O - Ai Hinzufügen", 20.0, 70.0, 30.0, WHITE);
        
        draw_text("Drücke <Enter> um zum Starten", 140., 500.0, 40.0, WHITE);

        if is_key_pressed(KeyCode::P) {
            snake_controllers.push(Box::new(KeyboardController::arrows()));
            println!("Added Player");
        }
        if is_key_pressed(KeyCode::O) {
            snake_controllers.push(Box::new(PipeController::new(pipe_names[current_pipe_index])));
            current_pipe_index += 1;
            println!("Added Ai");
        }
        if is_key_down(KeyCode::Enter) {
            if snake_controllers.len() > 1 {
                break;
            }
        }

        snake_controllers.iter().enumerate().for_each(|(i, x)| { 
            draw_text(&format!("> {}", x.get_name()), 20.0, 90.0 + 20.*i as f32, 20.0, WHITE);
        });

        widgets::Window::new(hash!(), vec2(870., 30.), vec2(300., 300.))
            .label("Settings")
            .ui(&mut *root_ui(), |ui| {

                               
                ui.label(None, "Grid Size");
                ui.input_text(hash!(), "Grid X Size", &mut grid_size_x);
                ui.input_text(hash!(), "Grid Y Size", &mut grid_size_y);

                ui.tree_node(hash!(), "Debug", |ui| {
                    if ui.button(None, "Click me") {
                        sandbox = true;
                    }
                    if ui.button(None, "Add Player (Arrow)") {
                        snake_controllers.push(Box::new(KeyboardController::arrows()));
                    }

                    if ui.button(None, "Add Player (WASD)") {
                        snake_controllers.push(Box::new(KeyboardController::wasd()));
                    }

                });
            });
        draw_version_hud();
        next_frame().await;        

        if sandbox {
            break; // Sandbox button click
        }
    }

    let mut parsed_grid_size_x = grid_size_x.parse();
    let mut parsed_grid_size_y = grid_size_y.parse();
    if parsed_grid_size_x.is_err() {
        println!("Invalid Grid Size!");
        parsed_grid_size_x = consts::GRID_SIZE.0.parse();
    }
    if parsed_grid_size_y.is_err() {
        println!("Invalid Grid Size!");
        parsed_grid_size_y = consts::GRID_SIZE.1.parse();
    }

    GameConfig { 
        snake_controller_list: snake_controllers,
        grid_size: (parsed_grid_size_x.unwrap(), parsed_grid_size_y.unwrap()),
        sandbox,
    }
}

pub fn draw_version_hud(){
    draw_text(&("v".to_owned()+consts::VERSION), screen_width()-100., screen_height() - 20., 20.0, WHITE);


}

