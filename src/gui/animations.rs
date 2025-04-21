use eframe::egui::{self, Color32, Ui, Vec2, Pos2};
use std::time::{Duration, Instant};

/// Animation state for smooth transitions
pub struct Animation {
    start_time: Instant,
    duration: Duration,
    completed: bool,
}

impl Animation {
    /// Create a new animation with the specified duration in seconds
    pub fn new(duration_secs: f32) -> Self {
        Self {
            start_time: Instant::now(),
            duration: Duration::from_secs_f32(duration_secs),
            completed: false,
        }
    }

    /// Get the current progress of the animation (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.completed {
            return 1.0;
        }

        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            1.0
        } else {
            elapsed.as_secs_f32() / self.duration.as_secs_f32()
        }
    }

    /// Check if the animation is completed
    pub fn is_completed(&self) -> bool {
        self.completed || self.start_time.elapsed() >= self.duration
    }

    /// Mark the animation as completed
    pub fn complete(&mut self) {
        self.completed = true;
    }

    /// Reset the animation
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.completed = false;
    }

    /// Interpolate between two colors based on animation progress
    pub fn lerp_color(&self, from: Color32, to: Color32) -> Color32 {
        let t = self.progress();
        lerp_color(from, to, t)
    }

    /// Interpolate between two positions based on animation progress
    pub fn lerp_pos(&self, from: Pos2, to: Pos2) -> Pos2 {
        let t = self.progress();
        lerp_pos(from, to, t)
    }

    /// Interpolate between two sizes based on animation progress
    pub fn lerp_vec(&self, from: Vec2, to: Vec2) -> Vec2 {
        let t = self.progress();
        lerp_vec(from, to, t)
    }

    /// Interpolate between two values based on animation progress
    pub fn lerp(&self, from: f32, to: f32) -> f32 {
        let t = self.progress();
        lerp(from, to, t)
    }
}

/// Interpolate between two colors
pub fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let r = lerp(from.r() as f32, to.r() as f32, t) as u8;
    let g = lerp(from.g() as f32, to.g() as f32, t) as u8;
    let b = lerp(from.b() as f32, to.b() as f32, t) as u8;
    let a = lerp(from.a() as f32, to.a() as f32, t) as u8;
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// Interpolate between two positions
pub fn lerp_pos(from: Pos2, to: Pos2, t: f32) -> Pos2 {
    let t = t.clamp(0.0, 1.0);
    Pos2::new(
        lerp(from.x, to.x, t),
        lerp(from.y, to.y, t),
    )
}

/// Interpolate between two sizes
pub fn lerp_vec(from: Vec2, to: Vec2, t: f32) -> Vec2 {
    let t = t.clamp(0.0, 1.0);
    Vec2::new(
        lerp(from.x, to.x, t),
        lerp(from.y, to.y, t),
    )
}

/// Interpolate between two values
pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    from + (to - from) * t
}

/// Apply an ease-in-out function to a value
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}




