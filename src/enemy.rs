use macroquad::{color::Color, math::vec2, rand::{self, ChooseRandom}, texture::Texture2D, window::screen_width};
use macroquad_particles::Emitter;

use crate::{animation::Explosion, bullet::Bullet, shape::{Shape, ShapeKind}};

pub struct EnemyPool {
    pool: Vec<Shape>,
}

impl EnemyPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: (0..capacity).map(|_| Shape::new_inactive()).collect(),
        }
    }

    pub fn spawn(&mut self, colors: &[Color]) {
        let size = rand::gen_range(16.0, 64.0);
        let color: &Color = colors.choose().unwrap();
        let x = rand::gen_range(size / 2.0, screen_width() - size / 2.0);
        if let Some(enemy) = self.pool.iter_mut().find(|s| !s.active) {
            enemy.size = size;
            enemy.speed = rand::gen_range(50.0, 150.0);
            enemy.x = x;
            enemy.y = -size;
            enemy.color = *color;
            enemy.kind = ShapeKind::Rect { width: size, height: size };
            enemy.collided = false;
            enemy.active = true;
        }
    }

    pub fn update(&mut self, delta_time: f32, screen_height: f32) {
        for enemy in self.pool.iter_mut().filter(|s| s.active) {
            enemy.y += enemy.speed * delta_time;
            if enemy.y > screen_height + enemy.size {
                enemy.active = false;
            }
        }
    }

    pub fn draw(&self) {
        for enemy in self.pool.iter().filter(|s| s.active) {
            enemy.draw();
        }
    }

    pub fn clear(&mut self) {
        for enemy in self.pool.iter_mut() {
            enemy.active = false;
        }
    }

    pub fn active_enemies_mut(&mut self) -> impl Iterator<Item = &mut Shape> {
        self.pool.iter_mut().filter(|s| s.active)
    }

    pub fn cleanup_collided(&mut self) {
        for enemy in self.pool.iter_mut().filter(|s| s.active && s.collided) {
            enemy.active = false;
            enemy.collided = false;
        }
    }

    pub fn check_bullet_collisions(
        &mut self,
        bullets: &mut Vec<Bullet>,
        explosion_emitter: &mut Emitter,
        explosions: &mut Vec<Explosion>,
        explosion_texture: &Texture2D,
        score: &mut u32,
        high_score: &mut u32,
    ) {
        for enemy in self.pool.iter_mut().filter(|e| e.active) {
            for bullet in bullets.iter_mut().filter(|b| !b.shape.collided) {
                if bullet.shape.collides_with(enemy) {
                    bullet.shape.collided = true;
                    enemy.active = false;

                    *score += enemy.size.round() as u32;
                    *high_score = (*high_score).max(*score);

                    let amount = (enemy.size.round() as usize).clamp(10, 25);
                    explosion_emitter.emit(vec2(enemy.x, enemy.y), amount);
                    explosions.push(Explosion::new(enemy.x, enemy.y, explosion_texture.clone()));
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }
}
