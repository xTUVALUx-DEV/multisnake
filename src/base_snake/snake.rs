use std::fmt::{Debug, Display, Write};

use macroquad::color::Color;

use super::object::Tile;

#[derive(Debug)]
pub struct PlayerInfo  {
    pub marked_cells: Vec<u16>,
    pub info_lines: Vec<String>,
}

pub struct SnakeData<'a> {
    pub height: u16,
    pub width: u16,
    pub grid: &'a Vec<Tile>,
    pub snakes: Vec<SnakeRefData>
}

#[derive(Debug, Clone)]
pub struct SnakeResponseData {
    pub height: u16,
    pub width: u16,
    pub grid: Vec<Tile>,
    pub snakes: Vec<SnakeRefResponseData>,
    pub my_snake_id: u16
}
impl Display for SnakeResponseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("{}", self.grid.len());
        self.grid.iter().enumerate().for_each(|(i, x)| {
            if i % self.width as usize == 0{
                f.write_str("\n");
            } 
            let string = match x {
                Tile::EMPTY => "#",
                Tile::FOOD => "O",
                Tile::Snake { id } => &id.to_string(),
                Tile::DeadSnake => "H",
            };

            f.write_str(string);

           
        });
        Ok(())
    }
}

impl<'a> SnakeData<'a> {
    pub fn encode(&self, snake_id: i32) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        // Packet ID
        data.push(0);        

        // push the height, width and snakes attrivutes to the u8 vec
        data.extend(&self.height.to_le_bytes());
        data.extend(&self.width.to_le_bytes());
        data.extend((snake_id as u16).to_le_bytes());

        // Encode the grid by pushing each tile's ID
        for tile in self.grid {
            data.extend((tile.get_tile_id() as i16).to_le_bytes());
        }

        // Encode the number of snakes (as u32 for safety)
        let snake_count = self.snakes.len() as u16;
        data.extend(snake_count.to_le_bytes());

        // Encode each snake
        for snake in &self.snakes {
            // Encode the snake ID (i32 -> 4 bytes, in little-endian order)
            data.extend(&(snake.id as i16).to_le_bytes());

            // Encode the snake's name length and content
            let name_bytes = snake.name.as_bytes();
            let name_length = name_bytes.len() as u16; // Use u16 for name length
            data.extend(name_length.to_le_bytes());
            data.extend(name_bytes);
            data.extend((snake.size as u16).to_le_bytes());

            data.extend((snake.tiles.len() as u16).to_le_bytes());
            for tile in snake.tiles.iter() {
                data.extend((*tile as u16).to_le_bytes());
            }

            // Encode the alive status (as a single byte: 1 for true, 0 for false)
            data.push(if snake.alive { 1 } else { 0 });
        }

        data
    }
}

fn read_from_buffer<T: Sized + Copy>(buffer: &[u8], offset: &mut usize) -> Result<T, String> {
    let size = std::mem::size_of::<T>();
    if *offset + size > buffer.len() {
        return Err("Buffer too short to read value".into());
    }
    let value = unsafe { 
        std::ptr::read(buffer[*offset..(*offset + size)].as_ptr() as *const T)
    };
    *offset += size;
    Ok(value)
}

impl SnakeResponseData {
    pub fn decode(buffer: &[u8]) -> Result<SnakeResponseData, String> {
        let mut offset = 0;

        // Read the packet ID (skipped since it's not used here)
        offset += 1;

        // Read the height and width
        let height = read_from_buffer::<u16>(buffer, &mut offset)?;
        let width = read_from_buffer::<u16>(buffer, &mut offset)?;

        // Read the snake ID
        let snake_id = read_from_buffer::<u16>(buffer, &mut offset)?;
        // Read the grid (height * width tiles)
        let mut grid = Vec::new();
        for _ in 0..(height * width) {
            let tile_id = read_from_buffer::<i16>(buffer, &mut offset)?;
            grid.push(Tile::from_tile_id(tile_id));
        }

        // Read the number of snakes
        let snake_count = read_from_buffer::<u16>(buffer, &mut offset)?;
        let mut snakes = Vec::new();

        for _ in 0..snake_count {
            // Read the snake ID
            let id = read_from_buffer::<i16>(buffer, &mut offset)? as i32;

            // Read the name length and name
            let name_length = read_from_buffer::<u16>(buffer, &mut offset)? as usize;
            if offset + name_length > buffer.len() {
                return Err("Buffer too short to read snake name".into());
            }
            let name = String::from_utf8(buffer[offset..offset + name_length].to_vec())
                .map_err(|_| "Invalid UTF-8 string in snake name")?;
            offset += name_length;

            let max_size = read_from_buffer::<u16>(buffer, &mut offset)?;
            // Read the snake's head position
            let num_tiles = read_from_buffer::<u16>(buffer, &mut offset)?;
            let mut tiles = Vec::new();
            for i in 0..num_tiles {
                tiles.push(read_from_buffer::<u16>(buffer, &mut offset)?);
            }

            // Read the alive status
            let alive = match buffer.get(offset) {
                Some(&1) => true,
                Some(&0) => false,
                _ => return Err("Invalid alive status value".into()),
            };
            offset += 1;

            snakes.push(SnakeRefResponseData {
                id,
                name,
                tiles,
                alive,
                size: max_size
            });
        }

        Ok(SnakeResponseData {
            height: height as u16,
            width: width as u16,
            grid,
            snakes,
            my_snake_id: snake_id
        })
    }
}


#[derive(Debug, Clone)]
pub enum SnakeState {
    ALIVE,
    DEAD
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NONE
}
impl Direction {
    pub fn to_string(&self) -> String {
        match self {
            Direction::UP => "Up",
            Direction::DOWN => "Down",
            Direction::LEFT => "Left",
            Direction::RIGHT => "Right",
            Direction::NONE => "None",
        }.to_string()
    }

    pub fn to_int(&self) -> Option<i32> {
        match self {
            Direction::UP => Some(10),
            Direction::DOWN => Some(11),
            Direction::LEFT => Some(12),
            Direction::RIGHT => Some(13),
            Direction::NONE => None,
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct SnakeRefData {
    pub id: i32,
    pub name: String,
    pub alive: bool,
    pub size: i32,
    pub tiles: Vec<i32>,
    pub color: (u8, u8, u8)
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct SnakeRefResponseData { // Trucated to not include size and color
    pub id: i32,
    pub name: String,
    pub alive: bool,
    pub tiles: Vec<u16>,
    pub size: u16
}



#[derive(Debug)]
pub struct Snake<'a> {
    pub id: i32,
    pub tiles: Vec<i32>,
    max_size: i32,
    controller: &'a mut dyn SnakeController,
    state: SnakeState,
    pub color: (u8, u8, u8)
}


impl<'a> Snake<'a> {
    pub fn new(id: i32, controller: &'a mut dyn SnakeController, color: (u8, u8, u8) ) -> Self {
        Self { id, tiles: Vec::new(), controller, max_size: 1, state: SnakeState::ALIVE, color }
    }
    pub fn update_controller(&mut self) {
        self.controller.update();
    }
    pub fn get_name(&self) -> String {
        self.controller.get_name()
    }
    pub fn send_gamestate(&mut self, data: SnakeData) {
        if self.is_dead() {
            return;
        }

        self.controller.report_data(data, self.id);
    }
    pub fn next_direction(&self) -> Direction {
         if let SnakeState::DEAD = self.state {
            return Direction::NONE;
        }        

        self.controller.next_direction()
    }
    pub fn get_head(&self) -> i32 {
        *self.tiles.first().expect("Tried to access uninitalized Snake")
    }
    pub fn get_tiles(&self) -> &Vec<i32> {
        &self.tiles
    }
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn is_dead(&self) -> bool {
        match self.state {
            SnakeState::ALIVE => false,
            SnakeState::DEAD => true
        }
    }
    pub fn move_head(&mut self, new_head: i32) -> Vec<i32> {
        // Input: New head location Output: Removed indexes
        if let SnakeState::DEAD = self.state {
            return Vec::new();
        }

        let mut removed = Vec::new();
        self.tiles.insert(0, new_head);
        while self.tiles.len() > self.max_size as usize {
            removed.push(self.tiles.pop().unwrap());
        }
        removed
    }
    pub fn grow(&mut self) {
        self.max_size += 1;
    }
    pub fn kill(&mut self) -> &Vec<i32> {
        // Kills the snake. Returns its tiles.
        self.state = SnakeState::DEAD;
        &self.tiles
    }
    pub fn get_data(&self) -> SnakeRefData {
        SnakeRefData {
            id: self.id,
            name: self.get_name(),
            alive: match self.state {
                SnakeState::ALIVE => true,
                SnakeState::DEAD => false
            },
            size: self.max_size,
            tiles: self.tiles.clone(),
            color: self.color
        }
    }
    pub fn get_info(&self) -> Option<PlayerInfo> {
        self.controller.get_info()
    }
    pub fn disconnect_controller(&mut self) {
        self.controller.disconnect()
    }
    pub fn connect_controller(&mut self) -> bool {
        self.controller.connect()
    }
}

pub trait SnakeController : Debug {
    fn report_data(&mut self, _data: SnakeData, _snake_id: i32) {}
    fn send_winner(&mut self, winner: i32) {}
    fn connect(&mut self) -> bool { true } // Only used for ai_controllers
    fn disconnect(&self) {}
    fn get_name(&self) -> String;
    fn update(&mut self) {}
    fn next_direction(&self) -> Direction;
    fn clone_weak(&self) -> Box<dyn SnakeController>;
    fn get_info(&self) -> Option<PlayerInfo> { None } 
}
