use macroquad::{color::WHITE, math::{Rect, vec2}, texture::{DrawTextureParams, Texture2D, draw_texture_ex}};


#[derive(Clone)]
pub struct AnimatedSprite {
    pub texture: Texture2D,
        pub frame_width: f32,
        pub frame_height: f32,
        pub start_frame: usize,
        pub total_frames: usize,
        pub fps: f32,
        pub current_frame: usize, // traking frame berapa
        pub frame_timer: f32,
        pub looping: bool, // true, looping ke 0 frame; false, finised and done
        pub finished: bool,
}

impl AnimatedSprite {
    pub fn new( texture: Texture2D,
           frame_width: f32,
           frame_height: f32,
           start_frame: usize,
           total_frames: usize,
           fps: f32,
           looping: bool) -> Self {
               Self {
            texture,
            frame_width,
            frame_height,
            start_frame,
            total_frames,
            fps,
            current_frame: 0,
            frame_timer: 0.0,
            looping,
            finished: false,
        }

    }

    // update frame animasi berdasarkan delta time
    pub fn update(&mut self, delta_time:f32) {
        if self.finished{
            return
        }

        //
        let frame_duration = 1.0 / self.fps;
        self.frame_timer += delta_time;

         // current frame itu adalah counter / index menunjukan frame nya dimana
        if self.frame_timer >= frame_duration {
            self.frame_timer -= frame_duration;
            self.current_frame += 1;
            if self.current_frame >= (self.start_frame + self.total_frames){
                if self.looping {
                    self.current_frame = self.start_frame;
                } else {
                    self.current_frame = (self.start_frame + self.total_frames) - 1;
                    self.finished = true;
                }
            }
        }
    }

    pub fn draw(&self, x: f32, y:f32, dest_w: f32, dest_h: f32) {

        let colums = (self.texture.width()/ self.frame_width) as usize;
        let colums = colums.max(1);

        // current frame itu adalah counter / index menunjukan frame nya dimana
        // kenapa frame_x itu di negasi (%) -> untuk bisa menentukan ini frame ganjil atau genap (berdasarkan colums / berdasarkan posisi frame)
        // misal (0, 0) itu di kiri, (16, 0) itu di kanan
        let frame_x = (self.current_frame % colums) as f32 * self.frame_width;
        let frame_y = (self.current_frame / colums) as f32 * self.frame_height;

        //Dan jika sprite sheet-nya hanya 1 baris horizontal (seperti explosion.png, misal 5 kolom, 1 baris):
        // - columns = 5
        // - Frame 0, 1, 2, 3, 4 semuanya akan menghasilkan frame / 5 = 0, sehingga frame_y akan selalu 0.0 secara otomatis
        draw_texture_ex(
            &self.texture,
            x - dest_w / 2.0, // center align dari draw
            y - dest_h / 2.0,
            WHITE,
            DrawTextureParams{
                // cut the area and focus on the active frame
                source: Some(Rect::new(frame_x, frame_y, self.frame_width, self.frame_height)),
                dest_size: Some(vec2(dest_w, dest_h)),
                ..Default::default()
            },
        );
    }
}
