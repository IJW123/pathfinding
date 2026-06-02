/// Format a distance in metres as a human-readable string, switching to
/// kilometres at/above 1 km. Sign-aware. Trailing zeros are trimmed, so clean
/// values read naturally (`1 km`, not `1.00 km`).
#[must_use]
pub fn format_distance(metres: f32) -> String {
    if metres.abs() >= 1000.0 {
        format!("{} km", trim_zeros(metres / 1000.0, 2))
    } else {
        format!("{metres:.0} m")
    }
}

/// Round `raw_m` down to the nearest "nice" cartographic value (1, 2 or 5 times a
/// power of ten), with a 1 m floor. Used to pick the scale-bar's labelled length.
#[must_use]
pub fn nice_distance(raw_m: f32) -> f32 {
    let raw = raw_m.max(1.0);
    let pow = 10f32.powf(raw.log10().floor());
    let frac = raw / pow;
    let lead = if frac >= 5.0 {
        5.0
    } else if frac >= 2.0 {
        2.0
    } else {
        1.0
    };
    lead * pow
}

/// Format `value` with up to `max_decimals` decimal places, dropping any trailing
/// zeros (and a bare trailing decimal point).
fn trim_zeros(value: f32, max_decimals: usize) -> String {
    let formatted = format!("{value:.max_decimals$}");
    if formatted.contains('.') {
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        formatted
    }
}
