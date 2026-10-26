use macroquad::{color::{Color, GREEN, WHITE, YELLOW}, text::{draw_text, measure_text}, time::{get_fps, get_frame_time}, window::{screen_height, screen_width}};

pub fn draw_scores(score: u32, high_score: u32){
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
}

pub fn draw_player_hp(current_hp: i32, max_hp: i32) {
    let text = format!("HP: {}/{}", current_hp, max_hp);
    draw_text(&text, 10.0, screen_height() - 20.0, 25.0, GREEN);
}

pub fn draw_debug_overlay(bullets_count: usize, enemies_count: usize, memory_mb: f32){
    draw_text(&format!("FPS: {}", get_fps()), 10.0, 60.0, 20.0, GREEN);
    draw_text(&format!("Frame Time: {:.2} ms", get_frame_time() * 1000.0), 10.0, 80.0, 20.0, GREEN);

    draw_text(&format!("Bullets: {}", bullets_count), 10.0, 120.0, 20.0, YELLOW);
    draw_text(&format!("Squares: {}", enemies_count), 10.0, 140.0, 20.0, YELLOW);

    draw_text(&format!("Heap RAM: {:.2} MB", memory_mb), 10.0, 100.0, 20.0, GREEN);
}

pub fn draw_center_message(text: &str, color: Color){
    let text_dimensions = measure_text(text, None, 50, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - text_dimensions.width / 2.0,
        screen_height() / 2.0,
        50.0,
        color,
    );
}
