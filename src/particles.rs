use macroquad::prelude::*;
use macroquad_particles::{self as particles, ColorCurve};

pub fn particle_explosion(texture: Texture2D) -> particles::EmitterConfig {
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

pub fn rocket_flame(texture: Texture2D) -> particles::EmitterConfig {
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
