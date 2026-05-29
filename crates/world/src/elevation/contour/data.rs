use bevy::math::Vec2;

pub struct ContourSegment {
    pub a: Vec2,
    pub b: Vec2,
}

pub struct ContourLine {
    pub level: f32,
    pub segments: Vec<ContourSegment>,
}
