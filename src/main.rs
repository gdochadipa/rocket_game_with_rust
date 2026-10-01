use macroquad::{prelude::*, rand::ChooseRandom};
use macroquad_particles::{self as particles, ColorCurve, Emitter};
use std::{fs, vec};


mod shape;
use shape::*;

mod diagnostic;
use diagnostic::*;

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

fn particle_explosion(texture: Texture2D) -> particles::EmitterConfig {
    particles::EmitterConfig{
        texture: Some(texture),
        local_coords:false, //Penting! Supaya partikel tetap di posisi dunia (world space) saat di-emit di lokasi berbeda
        one_shot: false, // Bisa dipanggil berulang kali
        emitting: false,  // Jangan emit terus menerus, kita yang kontrol manual
        lifetime: 0.6,
        lifetime_randomness: 0.3,
        initial_direction_spread: 2.0 * std::f32::consts::PI,
        initial_velocity: 360.0,
        initial_angular_velocity_randomness: 0.8,
        size: 3.0,
        size_randomness: 0.3,
        colors_curve: ColorCurve {
            start: RED,
            mid: ORANGE,
            end: RED,
        },
        ..Default::default()
    }
}

fn rocket_flame(texture: Texture2D) -> particles::EmitterConfig {
    particles::EmitterConfig{
        texture: Some(texture),
        local_coords: false,
        one_shot: false,
        emitting: false, // controll manual via .emit()
        lifetime: 0.25,
        lifetime_randomness: 0.1,
        initial_direction: vec2(0.0, 1.0),
        initial_direction_spread: std::f32::consts::FRAC_2_PI,
        initial_velocity: 180.0,
        size: 5.0,
        size_randomness: 0.2,
        colors_curve: ColorCurve {
            start: WHITE,  // Inti panas
            mid: ORANGE,   // Api tengah
            end: RED,      // Ujung asap/api
        },
        ..Default::default()
    }
}

#[macroquad::main("My game")]
async fn main() {
    let mut bullets: Vec<Shape> = vec![];
    let mut gameover:bool = false;
    let colors_square:Vec<Color> = vec![GREEN, RED, WHITE];
    let mut shot_cooldown = 0.0f32;
    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);
    let mut game_state = GameState::MainMenu;
    let mut square_pool:Vec<Shape> = (0..50).map(|_| Shape::new_inactive()).collect();
    let mut circle = Shape{
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        color: YELLOW,
        kind: ShapeKind::Circle { radius: 16.0 },
        collided: false,
        active: true,
        ..Default::default()
    };

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

    let mut explosion_emitter = Emitter::new(particle_explosion(pixel_texture.clone()));
    let mut rocket_flame_emitter = Emitter::new(rocket_flame(pixel_texture.clone())); // PENTING: Biar pinggirannya ga tajam

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
            let size = rand::gen_range(16.0, 64.0);
            // colors_square.choose itu returnya cuma alamat memory
            // lalo ada '*' untuk membantu dereference (copy) ke let color
            // .choose itu datang dari trait nya ChooseRandom, ini rust feature, otomatis soalnya
            // bisa di pisah atau dipilih kalo ada dua trait punya .choose yg sama
            // bisa juga diatur kalo misal colors_square itu punya bawaan .choose
            let color = *colors_square.choose().unwrap();
            if let Some(enemy) = square_pool.iter_mut().find(|s| !s.active){
                    enemy.size = size;
                    enemy.speed = rand::gen_range(50.0, 150.0);
                    enemy.x = rand::gen_range(size / 2.0, screen_width() - size / 2.0);
                    enemy.y = -size;
                    enemy.color = color;
                    enemy.kind = ShapeKind::Rect { width: size, height: size };
                    enemy.collided = false;
                    enemy.active = true;
            }
        }

        if gameover && is_key_pressed(KeyCode::Space) {
            for enemy in square_pool.iter_mut() {
                enemy.active = false;
            }
            bullets.clear();
            score = 0;
            circle.x = screen_width() / 2.0;
            circle.y = screen_height() / 2.0;
            gameover = false;

            // buat baru
            explosion_emitter = Emitter::new(particle_explosion(pixel_texture.clone()))
        }

        // draw
        circle.draw();

        rocket_flame_emitter.emit(vec2(circle.x, circle.y + CIRCLE_RADIUS), 2);
        rocket_flame_emitter.draw(vec2(0.0, 0.0));

        // // & itu borrowing data (reference gitu)
        // for square in &squares {
        //     square.draw();
        // }

        for enemy in square_pool.iter().filter(|s| s.active) {
            enemy.draw();
        }
        for bullet in &bullets{
            bullet.draw();
        }

        // Digambar 1x saja di offset (0, 0) karena partikel menggunakan world coordinates
        explosion_emitter.draw(vec2(0.0, 0.0));

        draw_text(
            format!("Score: {}", score).as_str(),
            10.0,
            35.0,
            25.0,
            WHITE,
        );
        let highscore_text = format!("High score: {}", high_score);
        let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
        draw_text(
            highscore_text.as_str(),
            screen_width() - text_dimensions.width - 10.0,
            35.0,
            25.0,
            WHITE,
        );

        if gameover {
            let text = "GAME OVER";
            let text_dimensions = measure_text(text, None, 50, 1.0);
            draw_text(text, screen_width()/ 2.0 - text_dimensions.width / 2.0, screen_height()/ 2.0, 50.0, RED);
        }

        match game_state {
            GameState::MainMenu => {
                if is_key_down(KeyCode::Escape){
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space){
                    for enemy in square_pool.iter_mut() {
                        enemy.active =false;
                    }
                    bullets.clear();
                    circle.x = screen_width() / 2.0;
                    circle.y = screen_height() / 2.0;
                    score = 0;
                    game_state = GameState::Playing;
                    // buat baru
                    explosion_emitter = Emitter::new(particle_explosion(pixel_texture.clone()))
                }

                let text = "Press space";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );

            }
            GameState::Playing => {
                let delta_time = get_frame_time();
                shot_cooldown = (shot_cooldown - delta_time).max(0.0);
                // & itu borrowing data (reference gitu)
                // &mut itu mengubah langsung ke asalnya
                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta_time;
                }

                // Remove squares below bottom of screen
                // squares.retain(|square| square.y < screen_height() + square.size);

                for enemy in square_pool.iter_mut().filter(|s| s.active){
                    enemy.y += enemy.speed *delta_time;
                    // Out of screen -> matikan
                        if enemy.y > screen_height() + enemy.size {
                            enemy.active = false;
                        }
                }

                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);

                // remove collided squares and bullet
                for enemy in square_pool.iter_mut().filter(|s| s.active && s.collided){
                    enemy.collided = false;
                    enemy.active = false;
                }
                bullets.retain(|bullet| !bullet.collided);

                if is_key_down(KeyCode::Right){
                    direction_modifier += 0.05 * delta_time;
                    circle.x += MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::Left){
                    direction_modifier -= 0.05 * delta_time;
                    circle.x -= MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::Up){
                    circle.y -= MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::Down){
                    circle.y += MOVEMENT_SPEED * delta_time;
                }

                // shooting bullet
                if is_key_down(KeyCode::Space) && shot_cooldown <= 0.0{
                    bullets.push(Shape {
                        x: circle.x,
                        y: circle.y,
                        speed: circle.speed * 2.0,
                        size: 5.0,
                        color: RED,
                        kind: ShapeKind::Rect { width: 5.0, height: 5.0 },
                        collided: false,
                        ..Default::default()
                    });

                        shot_cooldown = SHOT_DELAY;
                }

                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }

                circle.x = clamp(circle.x, CIRCLE_RADIUS, screen_width() - CIRCLE_RADIUS);
                circle.y = clamp(circle.y, CIRCLE_RADIUS, screen_height() - CIRCLE_RADIUS);

                for enemy in square_pool.iter_mut().filter(|e| e.active) {
                    for bullet in bullets.iter_mut(){
                        if bullet.collides_with(enemy) {
                            bullet.collided = true;
                            enemy.collided = true;
                            enemy.active = false;
                            score += enemy.size.round() as u32;
                            high_score = high_score.max(score);

                            let amount = (enemy.size.round() as usize).clamp(10, 25);
                            explosion_emitter.emit(vec2(enemy.x, enemy.y), amount);
                        }
                    }
                }

                // ketika circle colliades with square
                if square_pool.iter().filter(|e| e.active).any(|enemy| circle.collides_with(enemy)) {
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
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    WHITE,
                );
            }
            GameState::Gameover => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::MainMenu;
                }
                let text = "GAME OVER!";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    RED,
                );
            }
        }

        // Tampilkan FPS & Frame Time
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 60.0, 20.0, GREEN);
        draw_text(&format!("Frame Time: {:.2} ms", get_frame_time() * 1000.0), 10.0, 80.0, 20.0, GREEN);

        // Tampilkan Jumlah Objek Aktif (PENTING untuk deteksi memory leak/penumpukan)
        draw_text(&format!("Bullets: {}", bullets.len()), 10.0, 120.0, 20.0, YELLOW);
        draw_text(&format!("Squares: {}", square_pool.len()), 10.0, 140.0, 20.0, YELLOW);

        let mem_mb = get_memory_usage_mb();
        draw_text(&format!("Heap RAM: {:.2} MB", mem_mb), 10.0, 100.0, 20.0, GREEN);

        // flownya itu gini
        // value data kayak shape (circle dan square), itu di simpen datanya,
        // lalu value data itu kan include perpindahan posisi ya + delta_time (fps counter lah)
        // lalu di draw deh, lalu tunggu di next frame,
        // loop itu cepet banget, makanya kayak anmasi sebenarnya

        next_frame().await
    }
}
