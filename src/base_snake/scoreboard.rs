use std::collections::HashMap;
use macroquad::prelude::*;
use macroquad::ui::Skin;
use macroquad::{math::{vec2, Vec2}, ui::{hash, root_ui, widgets::{self, Group}}};

use super::snake::SnakeRefData;

pub struct Scoreboard {
    scoretable: HashMap<i32, (SnakeRefData, i32)>,
    default_position: Vec2
}

impl Scoreboard {
    pub fn new(len_snakes: i32) -> Self {
        Self { scoretable: HashMap::with_capacity(len_snakes as usize), default_position: vec2(600., len_snakes as f32*30. + 5.) }
    }
}

impl Scoreboard {
    pub fn add_win(&mut self, snake: &SnakeRefData) {
        let default = (snake.clone(), 0);
        let mut val = self.scoretable.get(&snake.id).unwrap_or(&default).clone();
        val.1 += 1;
        self.scoretable.insert(snake.id,  val.clone());
    }

    pub fn draw_widget(&self) {
        let skin = Scoreboard::get_style();
        root_ui().push_skin(&skin);

        widgets::Window::new(hash!(), self.default_position, vec2(320., 400.))
            .label("Scoreboard")
            .titlebar(true)
            .ui(&mut *root_ui(), |ui| {
                for (id, (snake, score)) in &self.scoretable {
                    let sum: i32 = self.scoretable.values().map(|x| x.1).sum();
                    Group::new(hash!("scores"), Vec2::new(200., 80.)).ui(ui, |ui| {
                        let mut name_label_style = skin.clone();
                        name_label_style.label_style = ui
                            .style_builder()
                            .text_color(Color::from_rgba(snake.color.0, snake.color.1, snake.color.2, 255))
                            .font_size(25)
                            .build();

                        ui.push_skin(&name_label_style);
                        ui.label(Vec2::new(2., 2.), &snake.name);
                        ui.pop_skin();

                        ui.label(Vec2::new(2., 22.), &format!("{}/{}", score, sum));

                        // Todo: Events to display a message log onclick
                        
                        //if ui.button(Vec2::new(260., 55.), "buy") {
                        //    println!("Item {}", i);
                        //}
                    });

                }
                //ui.pop_skin();
               //ui.push_skin();
            });
    }

    fn get_style() -> Skin {
        let label_style = root_ui()
            .style_builder()
            .text_color(Color::from_rgba(47, 170, 151, 255))
            .font_size(25)
            .build();

        let window_style = root_ui()
            .style_builder()
            .background_margin(RectOffset::new(52.0, 52.0, 52.0, 52.0))
            .color(Color::from_rgba(0, 0, 0, 0))
            .margin(RectOffset::new(-30.0, 0.0, -30.0, 0.0))
            .build();

        let button_style = root_ui()
            .style_builder()
            .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            .text_color(Color::from_rgba(47, 170, 151, 255))
            .color(Color::from_rgba(0, 0, 0, 0))
            .font_size(40)
            .build();
        Skin {
            window_style,
            button_style,
            label_style,
            ..root_ui().default_skin()
        }
    }
}

