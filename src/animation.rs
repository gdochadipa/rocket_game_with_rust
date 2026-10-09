use macroquad::{color::WHITE, math::{Rect, vec2}, texture::{DrawTextureParams, Texture2D, draw_texture_ex}};


#[derive(Clone)]
pub struct AnimatedSprite {
    pub texture: Texture2D,
    pub frame_width: f32,
    pub frame_height: f32,
    pub start_frame: usize,
    pub total_frames: usize,
    pub fps: f32,
    pub current_frame: usize,
    pub frame_timer: f32,
    pub looping: bool,
    pub finished: bool,
}

impl AnimatedSprite {
    pub fn new(
        texture: Texture2D,
        frame_width: f32,
        frame_height: f32,
        start_frame: usize,
        total_frames: usize,
        fps: f32,
        looping: bool,
    ) -> Self {
        Self {
            texture,
            frame_width,
            frame_height,
            start_frame,
            total_frames,
            fps,
            current_frame: start_frame,
            frame_timer: 0.0,
            looping,
            finished: false,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.finished {
            return;
        }

        let frame_duration = 1.0 / self.fps;
        self.frame_timer += delta_time;

        if self.frame_timer >= frame_duration {
            self.frame_timer -= frame_duration;
            self.current_frame += 1;
            if self.current_frame >= (self.start_frame + self.total_frames) {
                if self.looping {
                    self.current_frame = self.start_frame;
                } else {
                    self.current_frame = (self.start_frame + self.total_frames) - 1;
                    self.finished = true;
                }
            }
        }
    }

    pub fn draw(&self, x: f32, y: f32, dest_w: f32, dest_h: f32) {
        let colums = (self.texture.width() / self.frame_width) as usize;
        let colums = colums.max(1);

        let frame_x = (self.current_frame % colums) as f32 * self.frame_width;
        let frame_y = (self.current_frame / colums) as f32 * self.frame_height;

        draw_texture_ex(
            &self.texture,
            x - dest_w / 2.0,
            y - dest_h / 2.0,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(frame_x, frame_y, self.frame_width, self.frame_height)),
                dest_size: Some(vec2(dest_w, dest_h)),
                ..Default::default()
            },
        );
    }
}

#[derive(Clone)]
pub struct Explosion {
    pub x: f32,
    pub y: f32,
    pub sprite: AnimatedSprite,
}

impl Explosion {
    pub fn new(x: f32, y: f32, texture: Texture2D) -> Self {
        Self {
            x,
            y,
            sprite: AnimatedSprite::new(
                texture,
                16.0,
                16.0,
                0,
                5,
                15.0,
                false,
            ),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.sprite.update(delta_time);
    }

    pub fn draw(&self) {
        self.sprite.draw(self.x, self.y, 32.0, 32.0);
    }
}
