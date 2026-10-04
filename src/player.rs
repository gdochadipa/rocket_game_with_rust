use macroquad::{input::{KeyCode, is_key_down}, math::clamp, window::{screen_height, screen_width}};

use crate::{CIRCLE_RADIUS, shape::{self, Shape}};

pub struct Player {
    pub shape: Shape,
}

impl Player {
    pub  fn new(x: f32, y: f32, speed: f32) -> Self{
        return Self { shape: Shape
            {
                speed, x, y,
                ..Default::default()
            }
        };
    }

    pub fn handle_input(&mut self, delta_time: f32) -> f32 {
        let mut direction_modifier_delta = 0.0;
       if is_key_down(KeyCode::Right){
           direction_modifier_delta += 0.05 * delta_time;
           self.shape.x += self.shape.speed * delta_time;
       }

       if is_key_down(KeyCode::Left){
           direction_modifier_delta -= 0.05 * delta_time;
           self.shape.x -= self.shape.speed * delta_time;
       }
       if is_key_down(KeyCode::Up){
           self.shape.y -= self.shape.speed * delta_time;
       }
       if is_key_down(KeyCode::Down){
           self.shape.y += self.shape.speed * delta_time;
       }

       // clamp biar tidak diluar layar
       self.shape.x = clamp(self.shape.x, CIRCLE_RADIUS, screen_width() - CIRCLE_RADIUS);
       self.shape.y = clamp(self.shape.y, CIRCLE_RADIUS, screen_height() - CIRCLE_RADIUS);

       // direction modifier buat bg star
       direction_modifier_delta
    }

    pub fn draw(&self) {
        self.shape.draw();
    }

    pub fn reset(&mut self, x: f32, y:f32) {
        self.shape.x = x;
        self.shape.y = y;
    }

}
