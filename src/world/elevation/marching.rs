use bevy::math::Vec2;

// Corner layout (per cell, bottom-left origin):
//   C3 ---- C2
//   |        |
//   C0 ---- C1
//
// Edge numbering: 0 = bottom (C0-C1), 1 = right (C1-C2), 2 = top (C3-C2), 3 = left (C0-C3)
// Case index    : bit0=C0>=iso, bit1=C1>=iso, bit2=C2>=iso, bit3=C3>=iso
// Saddle rule   : "connect high corners" — high diagonal stays connected, isolates lows.
//
// Each row is up to two edge pairs (a,b),(c,d); -1 terminates / pads.
pub const CASE_TABLE: [[i8; 4]; 16] = [
    [-1, -1, -1, -1], // 0  : ....
    [3, 0, -1, -1],   // 1  : C0
    [0, 1, -1, -1],   // 2  : C1
    [3, 1, -1, -1],   // 3  : C0 C1
    [1, 2, -1, -1],   // 4  : C2
    [3, 0, 1, 2],     // 5  : C0 C2 (saddle, connect highs)
    [0, 2, -1, -1],   // 6  : C1 C2
    [3, 2, -1, -1],   // 7  : C0 C1 C2
    [2, 3, -1, -1],   // 8  : C3
    [2, 0, -1, -1],   // 9  : C0 C3
    [0, 1, 2, 3],     // 10 : C1 C3 (saddle, connect highs)
    [2, 1, -1, -1],   // 11 : C0 C1 C3
    [3, 1, -1, -1],   // 12 : C2 C3
    [0, 1, -1, -1],   // 13 : C0 C2 C3
    [3, 0, -1, -1],   // 14 : C1 C2 C3
    [-1, -1, -1, -1], // 15 : ....
];

pub fn emit_cell_segments(
    bl: Vec2,
    step: f32,
    corners: [f32; 4],
    iso: f32,
    push_segment: &mut impl FnMut(Vec2, Vec2),
) {
    let case = (corners[0] >= iso) as usize
        | ((corners[1] >= iso) as usize) << 1
        | ((corners[2] >= iso) as usize) << 2
        | ((corners[3] >= iso) as usize) << 3;

    let entries = CASE_TABLE[case];
    if entries[0] < 0 {
        return;
    }

    let edge_point = |edge: i8| -> Vec2 {
        let (a_idx, b_idx, a_pos, b_pos) = match edge {
            0 => (0, 1, bl, bl + Vec2::new(step, 0.0)),
            1 => (1, 2, bl + Vec2::new(step, 0.0), bl + Vec2::new(step, step)),
            2 => (3, 2, bl + Vec2::new(0.0, step), bl + Vec2::new(step, step)),
            3 => (0, 3, bl, bl + Vec2::new(0.0, step)),
            _ => unreachable!("CASE_TABLE only encodes edges 0..=3"),
        };
        let t = (iso - corners[a_idx]) / (corners[b_idx] - corners[a_idx]);
        a_pos + (b_pos - a_pos) * t
    };

    push_segment(edge_point(entries[0]), edge_point(entries[1]));
    if entries[2] >= 0 {
        push_segment(edge_point(entries[2]), edge_point(entries[3]));
    }
}
