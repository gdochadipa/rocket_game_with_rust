use macroquad::{input::{KeyCode, is_key_down}, math::clamp, window::{screen_height, screen_width}};

use crate::{CIRCLE_RADIUS, animation::AnimatedSprite, shape::Shape};

pub struct Player {
    pub shape: Shape,
    pub sprite: AnimatedSprite,
    pub hp: i32,
    pub max_hp: i32,
    pub invincibility_timer: f32,
}

impl Player {
    pub fn new(x: f32, y: f32, speed: f32, sprite: AnimatedSprite) -> Self {
        Self {
            shape: Shape {
                speed,
                x,
                y,
                size: 32.0,
                ..Default::default()
            },
            sprite,
            hp: 3,
            max_hp: 3,
            invincibility_timer: 0.0,
        }
    }

    pub fn take_damage(&mut self, amount: i32) -> bool {
        if self.invincibility_timer <= 0.0 {
            self.hp -= amount;
            self.invincibility_timer = 1.5;
            return true;
        }
        false
    }

    pub fn handle_input(&mut self, delta_time: f32) -> f32 {
        if self.invincibility_timer > 0.0 {
            self.invincibility_timer = (self.invincibility_timer - delta_time).max(0.0);
        }

        let mut direction_modifier_delta = 0.0;
        if is_key_down(KeyCode::Right) {
            direction_modifier_delta += 0.05 * delta_time;
            self.shape.x += self.shape.speed * delta_time;
        }

        if is_key_down(KeyCode::Left) {
            direction_modifier_delta -= 0.05 * delta_time;
            self.shape.x -= self.shape.speed * delta_time;
        }
        if is_key_down(KeyCode::Up) {
            self.shape.y -= self.shape.speed * delta_time;
        }
        if is_key_down(KeyCode::Down) {
            self.shape.y += self.shape.speed * delta_time;
        }

        self.shape.x = clamp(self.shape.x, CIRCLE_RADIUS, screen_width() - CIRCLE_RADIUS);
        self.shape.y = clamp(self.shape.y, CIRCLE_RADIUS, screen_height() - CIRCLE_RADIUS);

        if is_key_down(KeyCode::Left) {
            self.sprite.current_frame = 2;
        } else if is_key_down(KeyCode::Right) {
            self.sprite.current_frame = 4;
        } else {
            self.sprite.update(delta_time);
        }

        direction_modifier_delta
    }

    pub fn draw(&self) {
        if self.invincibility_timer > 0.0 {
            let blink = (self.invincibility_timer * 15.0) as i32 % 2 == 0;
            if blink {
                return;
            }
        }
        self.sprite.draw(self.shape.x, self.shape.y, 32.0, 48.0);
    }

    pub fn reset(&mut self, x: f32, y: f32) {
        self.shape.x = x;
        self.shape.y = y;
        self.hp = self.max_hp;
        self.invincibility_timer = 0.0;
    }
}
