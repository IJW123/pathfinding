use bevy::color::Color;
use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy)]
pub struct ContourStyle {
    pub low: [f32; 3],
    pub high: [f32; 3],
}

impl Default for ContourStyle {
    fn default() -> Self {
        Self {
            low: [0.4, 0.25, 0.1],
            high: [0.95, 0.9, 0.8],
        }
    }
}

impl ContourStyle {
    #[must_use]
    pub fn color_for_t(&self, t: f32) -> Color {
        let r = self.low[0] + (self.high[0] - self.low[0]) * t;
        let g = self.low[1] + (self.high[1] - self.low[1]) * t;
        let b = self.low[2] + (self.high[2] - self.low[2]) * t;
        Color::srgb(r, g, b)
    }
}
