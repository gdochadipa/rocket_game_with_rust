use macroquad::{input::{KeyCode, is_key_down}, math::clamp, window::{screen_height, screen_width}};

use crate::{CIRCLE_RADIUS, animation::AnimatedSprite, shape::{Shape}};

pub struct Player {
    pub shape: Shape,
    pub sprite: AnimatedSprite,
}

impl Player {
    pub  fn new(x: f32, y: f32, speed: f32, sprite: AnimatedSprite) -> Self{
        return Self { shape: Shape
            {
                speed, x, y,
                ..Default::default()
            },
            sprite
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

       if is_key_down(KeyCode::Left) {
           self.sprite.current_frame = 2; // Frame miring kiri
       } else if is_key_down(KeyCode::Right) {
           self.sprite.current_frame = 4; // Frame miring kanan
       } else {
           // Kalau diam, baru jalankan animasi idle (frame 0 dan 1 saja)
           self.sprite.update(delta_time);
       }


       // direction modifier buat bg star
       direction_modifier_delta
    }

    pub fn draw(&self) {
        self.sprite.draw(self.shape.x, self.shape.y, 32.0, 48.0);
    }

    pub fn reset(&mut self, x: f32, y:f32) {
        self.shape.x = x;
        self.shape.y = y;
    }

}
