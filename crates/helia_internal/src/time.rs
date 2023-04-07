use instant::Instant;

pub struct Time {
    /// total elapsed game time since startup 
    pub total_elapsed: f32,
    /// game time elapsed in last frame 
    pub elapsed: f32,
    /// speed at which time updates relative to wall clock time
    pub time_scale: f32,
    /// the maximum considered length of a frame
    /// game time will run slower if this is exceeded 
    pub max_frame_time_ms: Option<u32>,
    /// total elapsed time since startup
    pub total_elapsed_real_time: f32,
    /// real time elapsed in last frame 
    pub elapsed_real_time: f32,
    last_update_time: Instant,
    real_time_instant: Instant,
}

impl Time {
    pub fn update(&mut self) -> f32 {
        let elapsed = self.last_update_time.elapsed();

        self.elapsed_real_time = elapsed.as_secs_f32();
        self.total_elapsed_real_time = self.real_time_instant.elapsed().as_secs_f32();

        self.elapsed = elapsed.as_secs_f32() * self.time_scale;
        if let Some(max_ms) = self.max_frame_time_ms {
            if elapsed.as_millis() > max_ms as u128 {
                self.elapsed = max_ms as f32 / 1000.0 * self.time_scale
            }
        }
        self.last_update_time = Instant::now();

        self.elapsed
    }

    pub fn reset(&mut self) {
        self.total_elapsed = 0.0;
        self.real_time_instant = Instant::now();
    } 
}

impl Default for Time {
    fn default() -> Self {
        Self {
            total_elapsed: 0.0,
            elapsed: 0.0,
            time_scale: 1.0,
            total_elapsed_real_time: 0.0,
            elapsed_real_time: 0.0,
            max_frame_time_ms: None,
            last_update_time: Instant::now(),
            real_time_instant: Instant::now(),
        }
    }
}