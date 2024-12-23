use std::{cmp::min, collections::HashMap};

use macroquad::prelude::*;
use super::{object::Tile, snake::{self, Direction, PlayerInfo, Snake, SnakeController, SnakeData, SnakeRefData}};
use ::rand::{thread_rng, Rng};


const GRID_OFFSET_X: f32 = 10.;
const GRID_OFFSET_Y: f32 = 10.;

const GRID_SCREEN_SIZE: (f32, f32) = (600., 900.);

const SNAKE_COLORS: [Color; 4] = [BLUE, YELLOW, GREEN, ORANGE];


pub struct SnakeGrid<'a> {
    width: i32,
    height: i32,
    snakes: Vec<Snake<'a>>,
    grid: Vec<Tile>,
    snake_colors: Vec<Color>,
    square_size: f32,
    square_margin: f32,
    total_square_size: f32
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
        let empty_grid = vec![Tile::EMPTY; (width*height) as usize];
        let snake_colors = Vec::from(SNAKE_COLORS);
        
        let total_square_size = min((GRID_SCREEN_SIZE.0 / width as f32) as i32, (GRID_SCREEN_SIZE.1 / height as f32) as i32) as f32;
        let square_size = total_square_size * 0.7;
        let square_margin = total_square_size - square_size;

        Self {
            width, height, snakes: Vec::new(), grid: empty_grid, snake_colors, square_size, square_margin, total_square_size
        }
    }

    pub fn draw(&self) {

        for (i, object) in self.grid.iter().enumerate() {
            let x = i as i32 % self.width;
            let y = i  as i32 / self.width;

            
            let color = match object {
                &Tile::Snake {id} => self.snake_colors[id as usize],
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

    pub fn kill_snake(grid: &mut Vec<Tile>, snake: &mut Snake) {
        println!("{:?} died", snake);
        let tiles = snake.kill();
        for tile in tiles {
            grid[*tile as usize] = Tile::DeadSnake;
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
                SnakeGrid::kill_snake(&mut self.grid, snake);
                continue;
            }
            let new_head = y*self.width + x; // Where to move to

            // Check collisions
            if collisions.contains(&new_head) {
                SnakeGrid::kill_snake(&mut self.grid, snake);
                return;
            }
            
            match &self.grid[new_head as usize] {
                Tile::EMPTY => {},
                Tile::FOOD => { snake.grow(); SnakeGrid::place_food(&mut self.grid); },
                Tile::Snake { id: _ } => {
                    collisions.push(new_head);
                    SnakeGrid::kill_snake(&mut self.grid, snake);
                    return;
                }
                _ => {
                    SnakeGrid::kill_snake(&mut self.grid, snake);
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


}