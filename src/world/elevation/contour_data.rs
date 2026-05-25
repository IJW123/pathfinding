use bevy::math::Vec2;

pub struct ContourSegment {
    pub a: Vec2,
    pub b: Vec2,
}

pub struct ContourLine {
    #[expect(dead_code, reason = "consumed by upcoming label/highlight systems")]
    pub level: f32,
    pub segments: Vec<ContourSegment>,
}
