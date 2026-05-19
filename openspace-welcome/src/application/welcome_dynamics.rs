//! Pure animation dynamics for the welcome orb.
//!
//! These functions translate a normalised "hold progress" — the
//! integrated user intent of pressing on the orb — into the target
//! `(speed_multiplier, zoom)` pair the canvas should render. They
//! are pure so tests can pin behaviour at exact endpoints, and so
//! the canvas program can call them from its render path without
//! caring about timing.

/// Speed multiplier reached at maximum hold progress (the "final
/// form"). The galaxy stays at 1× when not held; while held, time
/// stretches up to this ceiling. Tuned to keep the spiral pattern
/// legible at the climax — pushing this much higher tears the arms
/// apart visually.
pub const MAX_SPEED_MULTIPLIER: f32 = 3.0;

/// Hard ceiling enforced inside the canvas program so a buggy
/// caller cannot drive `t` to infinity. Set fractionally above the
/// natural ceiling so eased values can briefly overshoot during
/// transitions without being clipped.
pub const SPEED_CLAMP: f32 = MAX_SPEED_MULTIPLIER + 0.5;

/// Zoom factor reached at maximum hold progress. The default rest
/// state is `1.0` (fit-to-bounds); higher values dolly the galaxy
/// in toward the centre so finer detail becomes legible.
pub const MAX_ZOOM: f32 = 1.6;

/// Translate a normalised hold-progress in `[0, 1]` into the
/// `(speed_multiplier, zoom)` target the canvas should ease toward.
///
/// `0.0` is the rest state (1× speed, fit-to-bounds). `1.0` is the
/// final form (`MAX_SPEED_MULTIPLIER` and `MAX_ZOOM`). Values
/// outside `[0, 1]` are clamped so the call-site can pass raw
/// integration output without re-checking bounds.
pub fn dynamics_for_progress(progress: f32) -> (f32, f32) {
    let p = progress.clamp(0.0, 1.0);
    let speed = 1.0 + (MAX_SPEED_MULTIPLIER - 1.0) * p;
    let zoom = 1.0 + (MAX_ZOOM - 1.0) * p;
    (speed, zoom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamics_for_progress_anchors_endpoints() {
        let (speed_min, zoom_min) = dynamics_for_progress(0.0);
        assert!((speed_min - 1.0).abs() < 1e-6);
        assert!((zoom_min - 1.0).abs() < 1e-6);

        let (speed_max, zoom_max) = dynamics_for_progress(1.0);
        assert!((speed_max - MAX_SPEED_MULTIPLIER).abs() < 1e-6);
        assert!((zoom_max - MAX_ZOOM).abs() < 1e-6);
    }

    #[test]
    fn dynamics_for_progress_is_monotonic() {
        let mut prev_speed = 0.0;
        let mut prev_zoom = 0.0;
        for step in 0..=10 {
            let p = step as f32 / 10.0;
            let (speed, zoom) = dynamics_for_progress(p);
            assert!(
                speed >= prev_speed,
                "speed must be non-decreasing in progress; got {prev_speed} -> {speed} at p={p}"
            );
            assert!(
                zoom >= prev_zoom,
                "zoom must be non-decreasing in progress; got {prev_zoom} -> {zoom} at p={p}"
            );
            prev_speed = speed;
            prev_zoom = zoom;
        }
    }

    #[test]
    fn dynamics_for_progress_clamps_out_of_range_inputs() {
        assert_eq!(dynamics_for_progress(-1.0), dynamics_for_progress(0.0));
        assert_eq!(dynamics_for_progress(2.0), dynamics_for_progress(1.0));
    }
}
