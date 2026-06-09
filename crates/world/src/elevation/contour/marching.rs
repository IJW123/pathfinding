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

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn collect(bl: Vec2, step: f32, corners: [f32; 4], iso: f32) -> Vec<(Vec2, Vec2)> {
        let mut out = Vec::new();
        emit_cell_segments(bl, step, corners, iso, &mut |a, b| out.push((a, b)));
        out
    }

    fn near(a: Vec2, b: Vec2) -> bool {
        a.distance(b) < EPS
    }

    /// True if `segs` contains a segment joining `p` and `q` in either orientation.
    fn has_seg(segs: &[(Vec2, Vec2)], p: Vec2, q: Vec2) -> bool {
        segs.iter()
            .any(|&(a, b)| (near(a, p) && near(b, q)) || (near(a, q) && near(b, p)))
    }

    #[test]
    fn uniform_cells_emit_nothing() {
        assert!(collect(Vec2::ZERO, 1.0, [0.0; 4], 0.5).is_empty()); // case 0
        assert!(collect(Vec2::ZERO, 1.0, [1.0; 4], 0.5).is_empty()); // case 15
    }

    #[test]
    fn single_corner_crosses_its_two_edges_at_midpoints() {
        // case 1: only C0 high ⇒ one segment joining left (edge3) and bottom (edge0).
        let segs = collect(Vec2::ZERO, 1.0, [1.0, 0.0, 0.0, 0.0], 0.5);
        assert_eq!(segs.len(), 1);
        let (a, b) = segs[0];
        assert!(near(a, Vec2::new(0.0, 0.5))); // left edge midpoint
        assert!(near(b, Vec2::new(0.5, 0.0))); // bottom edge midpoint
    }

    #[test]
    fn interpolation_follows_iso_level() {
        // case 1 with iso 0.25 ⇒ crossings at t = 0.75 along each edge.
        let segs = collect(Vec2::ZERO, 1.0, [1.0, 0.0, 0.0, 0.0], 0.25);
        let (a, b) = segs[0];
        assert!(near(a, Vec2::new(0.0, 0.75)));
        assert!(near(b, Vec2::new(0.75, 0.0)));
    }

    #[test]
    fn saddle_connects_high_corners() {
        // case 5: C0 (bottom-left) and C2 (top-right) high. Two segments, each isolating a
        // high corner — one cuts off C0 (left+bottom edges), the other C2 (right+top).
        let c5 = collect(Vec2::ZERO, 1.0, [1.0, 0.0, 1.0, 0.0], 0.5);
        assert_eq!(c5.len(), 2);
        assert!(
            has_seg(&c5, Vec2::new(0.0, 0.5), Vec2::new(0.5, 0.0)),
            "C0 not isolated: {c5:?}"
        );
        assert!(
            has_seg(&c5, Vec2::new(1.0, 0.5), Vec2::new(0.5, 1.0)),
            "C2 not isolated: {c5:?}"
        );

        // case 10: C1 (bottom-right) and C3 (top-left) high — the other diagonal.
        let c10 = collect(Vec2::ZERO, 1.0, [0.0, 1.0, 0.0, 1.0], 0.5);
        assert_eq!(c10.len(), 2);
        assert!(
            has_seg(&c10, Vec2::new(0.5, 0.0), Vec2::new(1.0, 0.5)),
            "C1 not isolated: {c10:?}"
        );
        assert!(
            has_seg(&c10, Vec2::new(0.5, 1.0), Vec2::new(0.0, 0.5)),
            "C3 not isolated: {c10:?}"
        );
    }

    #[test]
    fn complementary_cases_cross_the_same_edges() {
        // case 3 (C0,C1 high) and case 12 (C2,C3 high) are complements: identical contour.
        let case3 = collect(Vec2::ZERO, 1.0, [1.0, 1.0, 0.0, 0.0], 0.5);
        let case12 = collect(Vec2::ZERO, 1.0, [0.0, 0.0, 1.0, 1.0], 0.5);
        assert_eq!(case3.len(), 1);
        assert_eq!(case12.len(), 1);
        assert!(near(case3[0].0, case12[0].0));
        assert!(near(case3[0].1, case12[0].1));
    }

    #[test]
    fn bottom_left_offset_and_step_scale_endpoints() {
        let segs = collect(Vec2::new(10.0, 20.0), 2.0, [1.0, 0.0, 0.0, 0.0], 0.5);
        let (a, b) = segs[0];
        assert!(near(a, Vec2::new(10.0, 21.0))); // left edge midpoint, scaled + offset
        assert!(near(b, Vec2::new(11.0, 20.0))); // bottom edge midpoint
    }
}
