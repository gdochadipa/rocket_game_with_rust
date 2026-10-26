use macroquad::{color::WHITE};

use crate::{animation::AnimatedSprite, shape::{Shape, ShapeKind}};


pub struct Bullet {
    pub shape: Shape,
    pub sprite: AnimatedSprite,
}

impl Bullet {
    pub fn new(x: f32, y: f32, speed: f32, sprite: AnimatedSprite) -> Self{
        Self {
            shape: Shape {
                x,
                y,
                speed,
                size: 16.0,
                color: WHITE,
                kind: ShapeKind::Rect { width: 16.0, height: 16.0 },
                collided: false,
                active: true,
            },
            sprite
        }
    }

    pub fn update(&mut self, delta_time: f32) {
           self.shape.y -= self.shape.speed * delta_time;
           self.sprite.update(delta_time);
       }

       pub fn draw(&self) {
           self.sprite.draw(self.shape.x, self.shape.y, 16.0, 16.0);
       }
}
