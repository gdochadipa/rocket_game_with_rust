use macroquad::{prelude::*};
use macroquad_particles::{Emitter};
use std::{fs, vec};

mod shape;

use crate::{bullet::Bullet, diagnostic::get_memory_usage_mb, enemy::EnemyPool, player::Player};

mod diagnostic;
mod particles;
mod enemy;
mod player;
mod hud;
mod animation;
mod bullet;
use animation::AnimatedSprite;


const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

const MOVEMENT_SPEED: f32 = 200.0;
const CIRCLE_RADIUS:f32 = 16.0;
const SHOT_DELAY: f32 = 0.15;

enum GameState{
    MainMenu,
    Playing,
    Paused,
    Gameover
}


#[macroquad::main("My game")]
async fn main() {
    let ship_texture = load_texture("assets/ship.png").await.unwrap();
    ship_texture.set_filter(FilterMode::Nearest); // biar pxiel art ga ngeblur

    let laser_texture = load_texture("assets/laser-bolts.png").await.unwrap();
    laser_texture.set_filter(FilterMode::Nearest);

    let ship_sprite = AnimatedSprite::new(ship_texture, 16.0, 24.0,0, 5, 10.0, true);
    let bullet_sprite = AnimatedSprite::new(laser_texture.clone(), 16.0, 16.0,2, 2, 10.0, true);
    let mut bullets: Vec<Bullet> = vec![];
    let mut gameover:bool = false;
    let colors_square = [GREEN, RED, WHITE];
    let mut shot_cooldown = 0.0f32;
    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);
    let mut game_state = GameState::MainMenu;
    let mut enemy_pool = EnemyPool::new(50);

    let mut player = Player::new(screen_width() / 2.0, screen_height() / 2.0, MOVEMENT_SPEED, ship_sprite);

    let mut direction_modifier: f32 = 0.0;
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER
        },
        MaterialParams { uniforms: vec![
            UniformDesc::new("iResolution", UniformType::Float2),
                            UniformDesc::new("direction_modifier", UniformType::Float1),
            ],
            ..Default::default()
        }
    ).unwrap();

    let pixel_texture = Texture2D::from_rgba8(2, 2, &[255; 16]);
    pixel_texture.set_filter(FilterMode::Nearest);

    let mut explosion_emitter = Emitter::new(particles::particle_explosion(pixel_texture.clone()));
    let mut rocket_flame_emitter = Emitter::new(particles::rocket_flame(pixel_texture.clone())); // PENTING: Biar pinggirannya ga tajam

    loop {
        clear_background(BLACK);

        material.set_uniform("iResolution", (screen_width(),screen_height()));
        material.set_uniform("direction_modifier", direction_modifier);

        gl_use_material(&material);

        draw_texture_ex(
            &render_target.texture,
            0., 0., WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            });
        gl_use_default_material();

        // load enemy to memory
        // genereate enemy ke layar
        if rand::gen_range(0, 99) >= 95 {
            enemy_pool.spawn(&colors_square);
        }

        if gameover && is_key_pressed(KeyCode::Space) {
            enemy_pool.clear();
            bullets.clear();
            score = 0;
            player.shape.x = screen_width() / 2.0;
            player.shape.y = screen_height() / 2.0;
            gameover = false;

            // buat baru
            explosion_emitter = Emitter::new(particles::particle_explosion(pixel_texture.clone()))
        }

        // draw
        player.draw();

        rocket_flame_emitter.draw(vec2(0.0, 0.0));

        // // & itu borrowing data (reference gitu)
        // for square in &squares {
        //     square.draw();
        // }

        enemy_pool.draw();
        for bullet in &bullets{
            bullet.draw();
        }

        // Digambar 1x saja di offset (0, 0) karena partikel menggunakan world coordinates
        explosion_emitter.draw(vec2(0.0, 0.0));

        hud::draw_scores(score, high_score);

        if gameover {
            let text = "GAME OVER";
            hud::draw_center_message(&text, WHITE);
        }

        match game_state {
            GameState::MainMenu => {
                if is_key_down(KeyCode::Escape){
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space){
                    enemy_pool.clear();
                    bullets.clear();
                    player.reset(screen_width() / 2.0, screen_height() / 2.0);
                    score = 0;
                    game_state = GameState::Playing;
                    // buat baru
                    explosion_emitter = Emitter::new(particles::particle_explosion(pixel_texture.clone()))
                }

                let text = "Press space";
                hud::draw_center_message(&text, WHITE);

            }
            GameState::Playing => {
                let delta_time = get_frame_time();
                shot_cooldown = (shot_cooldown - delta_time).max(0.0);
                // & itu borrowing data (reference gitu)
                // &mut itu mengubah langsung ke asalnya
                for bullet in &mut bullets {
                    bullet.update(delta_time);
                }

                // Remove squares below bottom of screen
                // squares.retain(|square| square.y < screen_height() + square.size);

                enemy_pool.update(delta_time, screen_height());
                bullets.retain(|bullet| bullet.shape.y > 0.0 - bullet.shape.size / 2.0);

                // api api
                rocket_flame_emitter.emit(vec2(player.shape.x, player.shape.y + CIRCLE_RADIUS), 2);

                // remove collided squares and bullet
                enemy_pool.cleanup_collided();
                bullets.retain(|bullet| !bullet.shape.collided);

                // handle input of the player
                direction_modifier += player.handle_input(delta_time);

                // shooting bullet
                if is_key_down(KeyCode::Space) && shot_cooldown <= 0.0{

                    let bull = Bullet::new(player.shape.x, player.shape.y, player.shape.speed * 2.0, bullet_sprite.clone());
                    bullets.push(bull);

                    shot_cooldown = SHOT_DELAY;
                }

                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }

                enemy_pool.check_bullet_collisions(&mut bullets, &mut explosion_emitter, &mut score, &mut high_score);

                // ketika circle colliades with square
                if enemy_pool.active_enemies_mut().any(|enemy| player.shape.collides_with(enemy)) {
                    if score == high_score {
                        fs::write("highscore.dat", high_score.to_string()).ok();
                    }
                    game_state = GameState::Gameover;
                }
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Space){
                    game_state = GameState::Playing;
                }
                let text = "Paused";
                hud::draw_center_message(&text, WHITE);
            }
            GameState::Gameover => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::MainMenu;
                }
                let text = "GAME OVER!";
                hud::draw_center_message(&text, RED);
            }
        }

        let mem_mb = get_memory_usage_mb();
        hud::draw_debug_overlay(bullets.len(), enemy_pool.len(), mem_mb);


        // flownya itu gini
        // value data kayak shape (circle dan square), itu di simpen datanya,
        // lalu value data itu kan include perpindahan posisi ya + delta_time (fps counter lah)
        // lalu di draw deh, lalu tunggu di next frame,
        // loop itu cepet banget, makanya kayak anmasi sebenarnya

        next_frame().await
    }
}
