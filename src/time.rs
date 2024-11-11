use std::time::Instant;

pub struct Time {
    pub last_frame_time: Instant,
    frame_count: u32,
    pub fps: f32,
}

impl Time {
    pub fn new() -> Self {
        Time {
            last_frame_time: Instant::now(),
            frame_count: 0,
            fps: 0.0,
        }
    }
    pub fn update(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        self.frame_count += 1;

        if frame_time.as_secs_f32() > 0.0 {
            self.fps = 1.0 / frame_time.as_secs_f32();
        }
    }
}
