use std::{cmp::{max, min}, collections::HashMap};
use macroquad::prelude::*;
use super::{object::Tile, snake::{self, Direction, Snake, SnakeController, SnakeData, SnakeRefData}};
use ::rand::{thread_rng, Rng};


const SQUARE_SIZE: f32 = 20.;
const SQUARE_SPACING: f32 = 10.;

const GRID_OFFSET_X: f32 = 10.;
const GRID_OFFSET_Y: f32 = 10.;


const snake_colors: [Color; 4] = [BLUE, YELLOW, GREEN, ORANGE];

pub(crate) struct SnakeGrid<'a> {
    width: i32,
    height: i32,
    snakes: Vec<Snake<'a>>,
    grid: Vec<Tile>
}

impl<'a> SnakeGrid<'a> {
    pub fn new(width: i32, height: i32) -> Self {
        let mut empty_grid = vec![Tile::EMPTY; (width*height) as usize];
        Self {
            width, height, snakes: Vec::new(), grid: empty_grid
        }
    }

    pub fn draw(&self) {

        for (i, object) in self.grid.iter().enumerate() {
            let x = i as i32 % self.width;
            let y = i  as i32 / self.width;


            let color = match object {
                &Tile::Snake {id} => snake_colors[id as usize],
                &Tile::DeadSnake => GRAY,
                &Tile::EMPTY => DARKGRAY,
                &Tile::FOOD => RED,
                _ => WHITE
            };
            
            draw_rectangle(x as f32*(SQUARE_SIZE+SQUARE_SPACING) + GRID_OFFSET_X, y as f32*(SQUARE_SIZE+SQUARE_SPACING) + GRID_OFFSET_Y, SQUARE_SIZE, SQUARE_SIZE, color);
        }

        let (x_offset, y_offset) = (self.width as f32*(SQUARE_SIZE+SQUARE_SPACING) + 30., 20.);
        for (i, snake) in self.snakes.iter().enumerate() {
            draw_text(&snake.get_name(), x_offset, y_offset + i as f32*30., 30.0, snake_colors[snake.get_id() as usize]);
        }
    }

    pub fn add_snake(&mut self, controller: &'a mut dyn SnakeController) {
        let new_snake = Snake::new(self.snakes.len() as i32, controller);
        self.snakes.push(new_snake);
    }

    pub fn index_to_xy(index: i32, width: i32) -> (i32, i32) {
        (index % width, index / width)
    }
    pub fn xy_to_index(x: i32, y: i32, width: i32) -> i32 {
        x + width*y
    }

    pub fn kill_snake(grid: &mut Vec<Tile>, snake: &mut Snake) {
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
        for i in 0..20 {
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
            match &self.grid[new_head as usize] {
                Tile::EMPTY => {},
                Tile::FOOD => { snake.grow(); SnakeGrid::place_food(&mut self.grid); },
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

    pub fn start_game(&mut self) {
        if self.snakes.len() > 4 {
            panic!("Only 4 Snakes supported");
        }

        let spawn_positions = [
            SnakeGrid::xy_to_index((self.width as f32*0.25) as i32, (self.height as f32*0.25) as i32, self.width),
            SnakeGrid::xy_to_index((self.width as f32*0.25) as i32, (self.height as f32*0.75) as i32, self.width),
            SnakeGrid::xy_to_index((self.width as f32*0.75) as i32, (self.height as f32*0.25) as i32, self.width),
            SnakeGrid::xy_to_index((self.width as f32*0.75) as i32, (self.height as f32*0.75) as i32, self.width),
        ];

        for (i, snake) in (&mut self.snakes).iter_mut().enumerate() {
            snake.move_head(spawn_positions[i]);
        }
    }
    pub fn update_input(&mut self) {
        for snake in &mut self.snakes {
            snake.update_controller();
        }
    }
    pub fn send_gamestate(&self) {
        for snake in &self.snakes {
            snake.send_gamestate(SnakeData {
                grid: &self.grid,
                height: self.height as u16,
                width: self.width as u16,
                snakes: self.snakes.iter().map(|x| x.get_data()).collect(),
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

}