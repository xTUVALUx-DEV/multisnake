use std::collections::HashMap;
use macroquad::prelude::*;
use macroquad::ui::{Skin, Ui};
use macroquad::{math::{vec2, Vec2}, ui::{hash, root_ui, widgets::{self, Group}}};

use super::snake::{PlayerInfo, SnakeRefData};

fn with_skin<F>(ui: &mut Ui, skin: &Skin, func: F)
where
    F: FnOnce(&mut Ui),
{
    ui.push_skin(skin);
    func(ui);
    ui.pop_skin();
}

#[derive(Debug)]
pub struct Scoreboard {
    scoretable: HashMap<i32, (SnakeRefData, i32)>,
    default_position: Vec2,
    current_display: Option<i32> // Id of the displayed snake
}

impl Scoreboard {
    pub fn new(len_snakes: i32) -> Self {
        Self { scoretable: HashMap::with_capacity(len_snakes as usize), default_position: vec2(620., len_snakes as f32 * 30.), current_display: None }
    }
}

impl Scoreboard {
    pub fn initalize(&mut self, snake_refs: Vec<SnakeRefData>) {
        if !self.scoretable.is_empty() { // Dont reinitalize
            return;
        }

        snake_refs.iter().for_each(|snake| { self.scoretable.insert(snake.id, (snake.clone(), 0)); });
    }

    pub fn add_win(&mut self, snake: &SnakeRefData) {
        let default = (snake.clone(), 0);
        let mut val = self.scoretable.get(&snake.id).unwrap_or(&default).clone();
        val.1 += 1;
        self.scoretable.insert(snake.id,  val.clone());
    }

    pub fn draw_widget(&mut self, snake_infos: HashMap<i32, PlayerInfo>) {
        let scoretable = self.scoretable.clone();

        let mut skin = Scoreboard::get_style();
        skin.window_style = root_ui()
                            .style_builder()
                            .text_color(Color::from_rgba(47, 170, 151, 255))
                            .color(Color::from_rgba(0, 0, 0, 0))
                            .margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
                            .build();

        let sum: i32 = self.scoretable.values().map(|x| x.1).sum();

        
        with_skin(&mut root_ui(), &skin, |ui| {
            widgets::Window::new(hash!(), self.default_position, vec2(350., 1200.))
                .label("Scoreboard")
                .titlebar(true)
                .ui(ui, |ui| {
                for (id, (snake, score)) in scoretable {
                        Group::new(hash!("scores"), Vec2::new(300., 80.)).ui(ui, |ui| {
                            let mut cell_style = skin.clone();
                            cell_style.label_style = ui
                                .style_builder()
                                .text_color(Color::from_rgba(snake.color.0, snake.color.1, snake.color.2, 255))
                                .font_size(25)
                                .build();
                            
                            with_skin(ui, &cell_style, |ui: &mut Ui| {
                                ui.label(Vec2::new(2., 2.), &snake.name);
                            });

                            ui.label(Vec2::new(2., 22.), &format!("{}/{}", score, sum));

                            // Todo: Events to display a message log onclick
                            cell_style.button_style = ui
                                .style_builder()
                                .background_margin(RectOffset::new(8.0, 8.0, 8.0, 0.0))
                                .text_color(Color::from_rgba(47, 170, 151, 255))
                                .text_color_hovered(Color::from_rgba(47, 170, 151, 255))
                                .color_hovered(Color::from_rgba(20, 20, 20, 255))
                                .color_clicked(Color::from_rgba(20, 20, 20, 255))
                                .color(Color::from_rgba(0, 0, 0, 0))
                                .font_size(15)
                                .build();
                            
                            with_skin(ui, &cell_style, |ui: &mut Ui| {
                                if ui.button(Vec2::new(185., 40.), "Show Details") {
                                    println!("A{}", snake.id);
                                    self.current_display = Some(snake.id);
                                }
                            });
                        });

                    }
                });
        });
       
        if let Some(snake_id) = self.current_display {
            let snake_opt = self.scoretable.get(&snake_id);
            let info_opt = snake_infos.get(&snake_id);
            if snake_opt.is_none() {
                println!("[ERROR] Trying to display unknown snake");
                return;
            }
            if info_opt.is_none() {
                return;
            }
            
            let snake = snake_opt.unwrap();
            let infos = snake_infos.get(&snake_id).unwrap();

            let mut widget_style = skin.clone();
            widget_style.window_style = root_ui()
                .style_builder()
                .text_color(Color::from_rgba(47, 170, 151, 255))
                .color(Color::from_rgba(0, 0, 0, 255))
                .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
                .build();

            root_ui().push_skin(&widget_style);

            widgets::Window::new(hash!(), vec2(self.default_position.x+250., self.default_position.y), vec2(280., 140.))
            .label(&format!("Details for {}", snake.0.name))
            .titlebar(true)
            .ui(&mut *root_ui(), |ui| {
                let mut cell_style = skin.clone();
                cell_style.label_style = ui
                    .style_builder()
                    .text_color(Color::from_rgba(snake.0.color.0, snake.0.color.1, snake.0.color.2, 255))
                    .font_size(20)
                    .build();

                ui.push_skin(&cell_style);
                ui.label(Vec2::new(2., 2.), &snake.0.name);
                ui.pop_skin();

                for (i, info_line) in infos.info_lines.iter().enumerate() {
                    ui.label(Vec2::new(2., 42. + i as f32 * 20.), &info_line);
                }
            });

            root_ui().pop_skin();
        }
    }

    fn get_style() -> Skin {
        let label_style = root_ui()
            .style_builder()
            .text_color(Color::from_rgba(47, 170, 151, 255))
            .font_size(25)
            .build();

        let window_style = root_ui()
            .style_builder()
            .background_margin(RectOffset::new(0.0, 0.0, 0.0, 5.0))
            .color(Color::from_rgba(0, 0, 0, 0))
            .margin(RectOffset::new(0.0, 0.0, 0.0, 0.0))
            .build();

        let button_style = root_ui()
            .style_builder()
            .background_margin(RectOffset::new(8.0, 8.0, 8.0, 8.0))
            .text_color(Color::from_rgba(47, 170, 151, 255))
            .color_hovered(Color::from_rgba(20, 20, 20, 255))
            .color(Color::from_rgba(0, 0, 0, 0))
            .font_size(20)
            .build();

        Skin {
            window_style,
            button_style,
            label_style,
            ..root_ui().default_skin()
        }
    }
}

