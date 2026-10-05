use macroquad::{color::Color, math::vec2, rand::{self, ChooseRandom}, window::screen_width};
use macroquad_particles::Emitter;

use crate::{bullet::Bullet, shape::{Shape, ShapeKind}};

pub struct EnemyPool {
    pool: Vec<Shape>,
}


impl EnemyPool {
    pub fn new(capacity: usize) -> Self {
       return Self
       {
           pool: (0..capacity).map(|_| Shape::new_inactive()).collect()
       }
    }

    //bedanya vec[Color]  dan &[Color] adalah
    // vec[Color] itu menyimpan data + alamat memory (pointer to heap) + len
    // kalau Color menyimpan  alamat memory + len
    // Vec<Color> tanpa tanda & akan dianggap move,
    // Gunakan &[T] sebagai parameter fungsi, bukan &Vec<T> atau Vec<T>
    //
    // colors_square.choose itu returnya cuma alamat memory
    // kalo ada '*' untuk membantu dereference (copy) ke let color
    // .choose itu datang dari trait nya ChooseRandom, ini rust feature, otomatis soalnya
    // bisa di pisah atau dipilih kalo ada dua trait punya .choose yg sama
    // bisa juga diatur kalo misal colors_square itu punya bawaan .choose

    pub fn spawn(&mut self, colors: &[Color]) {
        let size = rand::gen_range(16.0, 64.0);
        let color:&Color = colors.choose().unwrap();
        let x = rand::gen_range(size / 2.0, screen_width() - size / 2.0);
        if let Some(enemy) = self.pool.iter_mut().find(|s| !s.active){
                enemy.size = size;
                enemy.speed = rand::gen_range(50.0, 150.0);
                enemy.x = x.clone();
                enemy.y = -size;
                enemy.color = *color;
                enemy.kind = ShapeKind::Rect { width: size, height: size };
                enemy.collided = false;
                enemy.active = true;
        };
    }

    pub fn update(&mut self, delta_time: f32, screen_height: f32){
        for enemy in self.pool.iter_mut().filter(|s| s.active){
            enemy.y += enemy.speed * delta_time;
            if enemy.y > screen_height + enemy.size {
                enemy.active = false;
            }
        }
    }

    pub fn draw(&self){
        for enemy in self.pool.iter().filter(|s| s.active) {
            enemy.draw();
        }
    }

    pub fn clear(&mut self){
        for enemy in self.pool.iter_mut() {
            enemy.active =false;
        }
    }

    pub fn active_enemies_mut(&mut self) -> impl Iterator<Item = &mut Shape> {
        self.pool.iter_mut().filter(|s| s.active)
    }

    pub fn cleanup_collided(&mut self) {
           for enemy in self.pool.iter_mut().filter(|s| s.active && s.collided) {
               enemy.active = false;   // Matikan musuh
               enemy.collided = false; // Reset status tabrakan
           }
       }

       pub fn check_bullet_collisions(
            &mut self,
            bullets: &mut Vec<Bullet>,
            explosion_emitter: &mut Emitter,
            score: &mut u32,
            high_score: &mut u32,
        ) {
            for enemy in self.pool.iter_mut().filter(|e| e.active) {
                for bullet in bullets.iter_mut().filter(|b| !b.shape.collided) {
                    if bullet.shape.collides_with(enemy) {
                        bullet.shape.collided = true;
                        enemy.active = false; // Langsung matikan musuh

                        *score += enemy.size.round() as u32;
                        *high_score = (*high_score).max(*score);

                        let amount = (enemy.size.round() as usize).clamp(10, 25);
                        explosion_emitter.emit(vec2(enemy.x, enemy.y), amount);
                    }
                }
            }
        }

        pub fn len(&self) -> usize{
            self.pool.len()
        }

}
