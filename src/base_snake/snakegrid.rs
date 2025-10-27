use std::{cmp::min, collections::HashMap};

use macroquad::prelude::*;
use super::{object::Tile, snake::{self, Direction, PlayerInfo, Snake, SnakeController, SnakeData, SnakeRefData}};
use ::rand::{thread_rng, Rng};


pub const GRID_OFFSET_X: f32 = 10.;
pub const GRID_OFFSET_Y: f32 = 10.;

const GRID_SCREEN_SIZE: (f32, f32) = (600., 900.);


pub struct SnakeGrid<'a> {
    pub width: i32,
    height: i32,
    snakes: Vec<Snake<'a>>,
    grid: Vec<Tile>,
    snake_colors: Vec<Color>,
    square_size: f32,
    square_margin: f32,
    pub total_square_size: f32,

    on_food_handler: Option<Box<dyn Fn() -> ()>>,
    on_death_handler: Option<Box<dyn Fn() -> ()>>
}

fn random_color_bright_non_red() -> Color {
    let mut rng = thread_rng();

    let green = rng.gen_range(128..=255);
    let blue = rng.gen_range(128..=255);
    let red = rng.gen_range(0..=127);

    Color::from_rgba(red, green, blue, 255)
}

impl<'a> SnakeGrid<'a> {
    pub fn new(width: i32, height: i32) -> Self {
        let SNAKE_COLORS: [Color; 7] =  [Color::from_rgba(171, 2, 168, 255), Color::from_rgba(0, 134, 119, 255), Color::from_rgba(143, 0, 255, 255), BLUE, YELLOW, GREEN, ORANGE];

        let empty_grid = vec![Tile::EMPTY; (width*height) as usize];
        let snake_colors = Vec::from(SNAKE_COLORS);
        
        let total_square_size = min((GRID_SCREEN_SIZE.0 / width as f32) as i32, (GRID_SCREEN_SIZE.1 / height as f32) as i32) as f32;
        let square_size = total_square_size * 0.7;
        let square_margin = total_square_size - square_size;

        Self {
            width, height, snakes: Vec::new(), grid: empty_grid, snake_colors, square_size, square_margin, total_square_size, on_food_handler: None, on_death_handler: None
        }
    }

    pub fn clone_raw_grid(&self) -> Vec<Tile> {
        self.grid.clone()
    }

    pub fn draw(&self) {

        for (i, object) in self.grid.iter().enumerate() {
            let x = i as i32 % self.width;
            let y = i  as i32 / self.width;

            
            let color = match object {
                &Tile::Snake {id} =>  self.snake_colors[id as usize],
                &Tile::DeadSnake => GRAY,
                &Tile::EMPTY => DARKGRAY,
                &Tile::FOOD => RED,
            };
            
            draw_rectangle(
                x as f32*self.total_square_size + GRID_OFFSET_X, 
                y as f32*self.total_square_size + GRID_OFFSET_Y, 
                self.square_size, self.square_size, color);
        }

        let (x_offset, y_offset) = (self.width as f32*self.total_square_size + 30., 20.);
        for (i, snake) in self.snakes.iter().enumerate() {
            draw_text(&snake.get_name(), x_offset, y_offset + i as f32*30., 30.0, self.snake_colors[snake.get_id() as usize]);
        }

        self.snakes.iter()
            .map(|x| (x.get_info(), x.color))
            .filter(|x| x.0.is_some())
            .map(|(x, c)| (x.unwrap().marked_cells, c))
            .for_each(|(x, (r, g, b ))| {
                x.iter().for_each(|cell_index| {
                    draw_rectangle(
                        (*cell_index as i32 % self.width) as f32* self.total_square_size + GRID_OFFSET_X, 
                        (*cell_index as i32 / self.width) as f32* self.total_square_size + GRID_OFFSET_Y, 
                        self.square_size, self.square_size, Color::from_rgba(r, g, b, 40));
                });
            });  
    }

    pub fn add_snake(&mut self, controller: &'a mut dyn SnakeController) {
        while self.snakes.len()+1 > self.snake_colors.len() {
            self.snake_colors.push(random_color_bright_non_red());
        }
        let snake_id = self.snakes.len();
        let color = self.snake_colors[snake_id];
        let new_snake = Snake::new(snake_id as i32, controller, ((color.r * 255.) as u8, (color.g * 255.) as u8, (color.b * 255.) as u8));
        self.snakes.push(new_snake);
        
    }

    pub fn index_to_xy(index: i32, width: i32) -> (i32, i32) {
        (index % width, index / width)
    }
    pub fn xy_to_index(x: i32, y: i32, width: i32) -> i32 {
        x + width*y
    }

    pub fn kill_snake(grid: &mut Vec<Tile>, snake: &mut Snake, handler: &Option<Box<dyn Fn()>>) {
        println!("{:?} died", snake);
        let tiles = snake.kill();
        for tile in tiles {
            grid[*tile as usize] = Tile::DeadSnake;
        }
        if let Some(handler) = handler {
            handler();
        }
    }

    pub fn do_place_food(&mut self) {
        SnakeGrid::place_food(&mut self.grid);
    }

    pub fn place_food(grid: &mut Vec<Tile>) {
        let mut rng = thread_rng();
        for _ in 0..20 {
            let x = rng.gen_range(0..grid.len());
            if let Tile::EMPTY = grid[x] {
                grid[x] = Tile::FOOD;
                return;
            }
        }
        println!("[WARNING] Couldnt place food");
    }

    pub fn tick(&mut self) {
        let width = self.width;

        let mut collisions =  Vec::new();
        
        for snake in &mut self.snakes.iter_mut() {
            if snake.is_dead() {
                continue;
            }

            let head = snake.get_head();
            let (mut x, mut y) = SnakeGrid::index_to_xy(head, width);

            match snake.next_direction() {
                Direction::UP => y -= 1,
                Direction::DOWN => y += 1,
                Direction::LEFT => x -= 1,
                Direction::RIGHT => x += 1,
                _ => {}
            };
            
            // Check Borders
            if x < 0 || x >= self.width || y < 0 || y >= self.height {
                SnakeGrid::kill_snake(&mut self.grid, snake, &self.on_death_handler);
                continue;
            }
            let new_head = y*self.width + x; // Where to move to

            // Check collisions
            if collisions.contains(&new_head) {
                SnakeGrid::kill_snake(&mut self.grid, snake, &self.on_death_handler);
                return;
            }
            
            match &self.grid[new_head as usize] {
                Tile::EMPTY => {},
                Tile::FOOD => { 
                    snake.grow(); 
                    SnakeGrid::place_food(&mut self.grid);
                    if let Some(handler) = &mut &self.on_food_handler {
                        handler();
                    }
                },
                Tile::Snake { id: _ } => {
                    collisions.push(new_head);
                    SnakeGrid::kill_snake(&mut self.grid, snake, &self.on_death_handler);
                    return;
                }
                _ => {
                    SnakeGrid::kill_snake(&mut self.grid, snake, &self.on_death_handler);
                    return;
                }
            }

            // Move logic
            self.grid[new_head as usize] = Tile::Snake { id: snake.get_id() };
            for removed in snake.move_head(new_head) {
                self.grid[removed as usize] = Tile::EMPTY;
            }
        }
    }
    pub fn get_random_spawn_positions(&self) -> Vec<i32> {
        let mut rng = thread_rng();
        let mut new_vec: Vec<i32> = Vec::new();

        for i in 0..1000 {
            let x = rng.gen_range(0..self.grid.len()) as i32;
            if !new_vec.contains(&x) {
                new_vec.push(x);
            }
        }
        new_vec
    }

    pub fn start_game(&mut self) {
        let spawn_positions = if self.snakes.len() <= 4 {
            Vec::from([
                SnakeGrid::xy_to_index((self.width as f32*0.25) as i32, (self.height as f32*0.25) as i32, self.width),
                SnakeGrid::xy_to_index((self.width as f32*0.25) as i32, (self.height as f32*0.75) as i32, self.width),
                SnakeGrid::xy_to_index((self.width as f32*0.75) as i32, (self.height as f32*0.25) as i32, self.width),
                SnakeGrid::xy_to_index((self.width as f32*0.75) as i32, (self.height as f32*0.75) as i32, self.width),
            ])
        }
        else {
            self.get_random_spawn_positions()
        };
        
        for (i, snake) in (&mut self.snakes).iter_mut().enumerate() {
            snake.move_head(spawn_positions[i]);
        }
    }
    pub fn update_input(&mut self) {
        for snake in self.snakes.iter_mut().filter(|x| !x.is_dead()) {
            snake.update_controller();
        }
    }
    pub fn send_gamestate(&mut self) {
        let snakes: Vec<SnakeRefData> = self.snakes.iter().map(|x| x.get_data()).collect();
        for snake in &mut self.snakes {
            snake.send_gamestate(SnakeData {
                grid: &self.grid,
                height: self.height as u16,
                width: self.width as u16,
                snakes: snakes.clone(),
            });
        }
    }

    pub fn check_end(&self) -> Result<SnakeRefData, i32> {
        // Success: Last snake, Error: 0 = No snakes left, 1 = Game still ongoing
        let mut alive_snake = Err(0);
        for snake in &self.snakes {
            if !snake.is_dead() {
                if let Err(0) = alive_snake {
                    alive_snake = Ok(snake.get_data());
                } else {
                    return Err(1);
                }
            }
        }
        alive_snake
    }
    pub fn get_all_snake_refs(&self) -> Vec<SnakeRefData> {
        self.snakes.iter().map(|x| x.get_data()).collect()
    }

    pub fn get_info_dict(&self) -> HashMap<i32, PlayerInfo> {
        self.snakes.iter()
            .map(|x| (x.get_id(), x.get_info()))
            .filter(|x| x.1.is_some())
            .map(|x| (x.0, x.1.unwrap())).collect::<HashMap<_, _>>()
    }

    pub fn set_square(&mut self, index: usize, snake: Option<i32>) {
        // Very very unsafe! Only for debugging   (Because the snake'id is not bound to be the index) 
        let old = &self.grid[index];
        if let Tile::Snake { id } = old {

            let index_to_remove = self.snakes[*id as usize].tiles.iter().position(|x| *x == index as i32).unwrap();
            self.snakes[*id as usize].tiles.remove(index_to_remove);
        }

        match snake {
            None => {
                self.grid[index] = Tile::EMPTY;
            },
            Some(snake_id) => {
                let mut snake = &mut self.snakes[snake_id as usize];
                self.grid[index] = Tile::Snake { id: snake_id };
                snake.tiles.push(index as i32);
            }
        }
    }

    pub fn reconnect(&mut self) {
        self.snakes.iter_mut().for_each(|snake| { snake.disconnect_controller(); snake.connect_controller(); });
    }
    
    pub fn clear(&mut self) {
        // Clears the Grid and adds one food
        self.grid.iter_mut().for_each(|x| *x = Tile::EMPTY);
        self.snakes.iter_mut().for_each(|x| x.tiles.clear());
        self.do_place_food();
    }

    pub fn register_on_food_handler(&mut self, handler: Box<(dyn Fn())>) {
        self.on_food_handler = Some(handler)
    }

    pub fn register_on_death(&mut self, handler: Box<(dyn Fn())>) {
        self.on_death_handler = Some(handler);
    }



}