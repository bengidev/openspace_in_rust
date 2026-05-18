//! ASCII / pixel particle orb rendered into an Iced canvas.
//!
//! This widget mirrors the visual identity of the iOS
//! `HomeAsciiParticleOrbView` while staying within the constraints of
//! the Iced 0.14 canvas API. Concretely:
//!
//! * The "orb" is a pixelated cluster of accent-tinted rectangles
//!   sampled from a 2D Gaussian core density. The shape is identical
//!   between frames apart from per-block oscillations driven by a
//!   deterministic noise function of `(seed, t)`.
//! * Each block has a per-block phase offset, scale range and opacity
//!   range — this is what produces the organic shimmer captured in
//!   the reference recording instead of a uniform pulse.
//! * Surrounding "dust" dots use the muted/border palette, giving the
//!   characteristic CRT-static halo around the cluster.
//!
//! All randomness is seeded; the orb is a pure function of the
//! elapsed time, so it is deterministic, cheap to redraw, and does
//! not allocate on the hot path beyond the geometry frame Iced
//! already manages internally.

use std::time::Instant;

use iced::Color;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;
use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program};

use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::{BackgroundToken, BorderToken, ForegroundToken};

/// Logical canvas the particle field is laid out in. The actual
/// canvas widget is scaled to fit while preserving aspect.
const LOGICAL_SIZE: Size = Size {
    width: 360.0,
    height: 240.0,
};

const CORE_FIELD: Size = Size {
    width: 156.0,
    height: 146.0,
};

const OUTER_FIELD: Size = Size {
    width: 324.0,
    height: 204.0,
};

/// Number of accent-tinted core blocks. Tuned to roughly match the
/// density of the reference recording without breaking 60 fps on
/// modest hardware.
const CORE_BLOCK_COUNT: usize = 220;
const OUTER_DUST_COUNT: usize = 168;
const PULSE_DOT_COUNT: usize = 86;
const SPARK_COUNT: usize = 36;

/// Snap the rendered block grid to whole pixels so the cluster keeps
/// the chunky, pixel-art look from the reference rather than being
/// anti-aliased into a smooth blob.
const SNAP_GRID: f32 = 3.0;

// ---------------------------------------------------------------------------
// Public canvas program
// ---------------------------------------------------------------------------

/// Iced canvas program rendering the welcome ASCII orb.
#[derive(Debug, Clone, Copy)]
pub struct AsciiOrbProgram {
    theme: OpenSpaceTheme,
    /// Time origin of the animation. Driven by the parent's `Tick`
    /// subscription rather than by `Instant::now` inside `draw` so
    /// animations remain referentially transparent.
    started_at: Instant,
    /// Most recent tick. Cached separately so we don't query the
    /// system clock from the render path.
    now: Instant,
}

impl AsciiOrbProgram {
    pub fn new(theme: OpenSpaceTheme, started_at: Instant, now: Instant) -> Self {
        Self {
            theme,
            started_at,
            now,
        }
    }

    fn elapsed_seconds(&self) -> f32 {
        self.now.saturating_duration_since(self.started_at).as_secs_f32()
    }
}

impl<Message> Program<Message> for AsciiOrbProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let t = self.elapsed_seconds();

        // Compute a uniform scale that fits the logical 360x240
        // canvas inside the actual widget bounds while preserving
        // aspect, then center the cluster.
        let scale = (bounds.width / LOGICAL_SIZE.width)
            .min(bounds.height / LOGICAL_SIZE.height);
        let translate = Point {
            x: (bounds.width - LOGICAL_SIZE.width * scale) * 0.5,
            y: (bounds.height - LOGICAL_SIZE.height * scale) * 0.5,
        };

        let project = |p: Point| Point {
            x: translate.x + p.x * scale,
            y: translate.y + p.y * scale,
        };

        // 1. outer dust — muted scatter halo
        draw_outer_dust(&mut frame, &self.theme, t, scale, project);

        // 2. orbital sparks — small accent flecks rotating around
        draw_orbit_sparks(&mut frame, &self.theme, t, scale, project);

        // 3. pulse ring — secondary scatter just outside the core
        draw_pulse_dots(&mut frame, &self.theme, t, scale, project);

        // 4. accent core — the pixel cluster itself, drawn last so
        //    it sits visually on top of the halo.
        draw_core_blocks(&mut frame, &self.theme, t, scale, project);

        vec![frame.into_geometry()]
    }
}

// ---------------------------------------------------------------------------
// Particle layers
// ---------------------------------------------------------------------------

fn draw_core_blocks(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let accent = theme.foreground(ForegroundToken::Accent);
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    let mut placed = 0usize;
    let mut attempt = 0usize;
    let max_attempts = CORE_BLOCK_COUNT * 12;

    while placed < CORE_BLOCK_COUNT && attempt < max_attempts {
        let seed = 2_400.0 + attempt as f32;
        let nx = noise(seed, 3.0) * 2.0 - 1.0;
        let ny = noise(seed, 9.0) * 2.0 - 1.0;
        let density = core_density(nx, ny, seed);

        if noise(seed, 15.0) < density {
            let raw_x = center.x + nx * CORE_FIELD.width * 0.5;
            let raw_y = center.y + ny * CORE_FIELD.height * 0.5;

            // Per-block drift: slow oscillation phased by seed.
            let phase = noise(seed, 53.0) * std::f32::consts::TAU;
            let drift_x = (t * 0.55 + phase).sin() * 1.6;
            let drift_y = (t * 0.42 + phase * 1.3).cos() * 1.2;

            let block_x = snap(raw_x + drift_x);
            let block_y = snap(raw_y + drift_y);

            let energy = (density + noise(seed, 25.0) * 0.12).clamp(0.0, 1.0);
            let base_alpha = (0.30 + energy * 0.65).clamp(0.0, 1.0);

            // Per-block opacity shimmer — fastest at high-energy
            // pixels so the bright centre flickers more than the
            // edges.
            let shimmer_phase = noise(seed, 71.0) * std::f32::consts::TAU;
            let shimmer = ((t * 1.6 + shimmer_phase).sin() * 0.5 + 0.5)
                * (0.20 + energy * 0.18);
            let alpha = (base_alpha * (0.78 + shimmer)).clamp(0.05, 1.0);

            // Per-block scale: high-energy blocks pulse a touch
            // bigger than the rest of the cluster.
            let scale_phase = noise(seed, 89.0) * std::f32::consts::TAU;
            let pulse = (t * 1.1 + scale_phase).sin() * 0.5 + 0.5;
            let block_size = (3.4 + energy * 4.6 + pulse * 0.9).clamp(2.5, 9.0);

            let projected = project(Point {
                x: block_x,
                y: block_y,
            });
            let size = block_size * scale;
            frame.fill_rectangle(
                Point {
                    x: projected.x - size * 0.5,
                    y: projected.y - size * 0.5,
                },
                Size {
                    width: size,
                    height: size,
                },
                with_alpha(accent, alpha),
            );
            placed += 1;
        }
        attempt += 1;
    }
}

fn draw_outer_dust(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let halo = theme.foreground(ForegroundToken::Muted);
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    for i in 0..OUTER_DUST_COUNT {
        let seed = i as f32;
        let radial = 0.28 + noise(seed, 3.0).powf(0.82) * 0.72;
        let angle = noise(seed, 11.0) * std::f32::consts::TAU
            + t * (0.05 + noise(seed, 19.0) * 0.04);

        let jitter_x = (noise(seed, 29.0) - 0.5) * 14.0;
        let jitter_y = (noise(seed, 37.0) - 0.5) * 11.0;

        let p = Point {
            x: center.x
                + angle.cos() * OUTER_FIELD.width * radial * 0.5
                + jitter_x,
            y: center.y
                + angle.sin() * OUTER_FIELD.height * radial * 0.5
                + jitter_y,
        };

        let phase = noise(seed, 47.0) * std::f32::consts::TAU;
        let opacity_pulse =
            ((t * 0.7 + phase).sin() * 0.5 + 0.5) * 0.18 + 0.10;
        let size = (1.2 + noise(seed, 47.0) * 2.4) * scale;

        let projected = project(p);
        frame.fill_rectangle(
            Point {
                x: projected.x - size * 0.5,
                y: projected.y - size * 0.5,
            },
            Size {
                width: size,
                height: size,
            },
            with_alpha(halo, opacity_pulse),
        );
    }
}

fn draw_pulse_dots(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let halo = theme.border(BorderToken::Strong);
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    for i in 0..PULSE_DOT_COUNT {
        let seed = 1_800.0 + i as f32;
        let angle = (i as f32 / PULSE_DOT_COUNT as f32) * std::f32::consts::TAU
            + (noise(seed, 5.0) - 0.5) * 0.22;
        let radius = 0.42 + noise(seed, 13.0) * 0.16;

        let phase = noise(seed, 29.0) * std::f32::consts::TAU;
        let pulse = (t * 0.9 + phase).sin() * 0.5 + 0.5;

        let p = Point {
            x: center.x
                + angle.cos() * CORE_FIELD.width * radius * (1.0 + pulse * 0.10),
            y: center.y
                + angle.sin() * CORE_FIELD.height * radius * 0.70 * (1.0 + pulse * 0.08),
        };

        let size = (1.0 + noise(seed, 19.0) * 2.0) * scale;
        let alpha = 0.10 + pulse * 0.18;

        let projected = project(p);
        frame.fill_rectangle(
            Point {
                x: projected.x - size * 0.5,
                y: projected.y - size * 0.5,
            },
            Size {
                width: size,
                height: size,
            },
            with_alpha(halo, alpha),
        );
    }
}

fn draw_orbit_sparks(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let accent_dim = theme.background(BackgroundToken::Elevated);
    let accent = theme.foreground(ForegroundToken::Accent);
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    for i in 0..SPARK_COUNT {
        let seed = 11_200.0 + i as f32;
        let angle_offset = noise(seed, 13.0) * std::f32::consts::TAU;
        let orbit_radius = 34.0 + noise(seed, 17.0) * 76.0;
        let vertical_scale = 0.58 + noise(seed, 31.0) * 0.26;
        let phase = noise(seed, 53.0) * std::f32::consts::TAU;

        let angle = angle_offset + (t * (0.18 + noise(seed, 23.0) * 0.18)) + phase;
        let radial_pulse = (t * 0.5 + phase).sin() * 4.0;
        let radius = orbit_radius + radial_pulse;

        let p = Point {
            x: center.x + angle.cos() * radius,
            y: center.y + angle.sin() * radius * vertical_scale,
        };

        let twinkle = ((t * 1.3 + phase).sin() * 0.5 + 0.5) * 0.42;
        let size = (2.0 + noise(seed, 23.0) * 2.5) * scale;

        // Two-layer fleck: a faint glow rectangle behind, accent pip
        // on top — gives the static a subtle warm tone instead of
        // grey-only.
        let projected = project(p);
        let glow_size = size * 1.6;
        frame.fill_rectangle(
            Point {
                x: projected.x - glow_size * 0.5,
                y: projected.y - glow_size * 0.5,
            },
            Size {
                width: glow_size,
                height: glow_size,
            },
            with_alpha(accent_dim, 0.10 + twinkle * 0.18),
        );
        frame.fill_rectangle(
            Point {
                x: projected.x - size * 0.5,
                y: projected.y - size * 0.5,
            },
            Size {
                width: size,
                height: size,
            },
            with_alpha(accent, 0.10 + twinkle * 0.32),
        );
    }
}

// ---------------------------------------------------------------------------
// Density + noise helpers
// ---------------------------------------------------------------------------

/// Same closed-form 2D density used by the iOS reference: a soft
/// shell, an inner ring, three Gaussian masses (centre, upper-left,
/// lower-right) and a small spiral. Two negative Gaussians "bite"
/// out of the cluster, giving it the irregular silhouette of the
/// recording instead of a perfect disc.
fn core_density(x: f32, y: f32, seed: f32) -> f32 {
    let radius = (x * x + y * y).sqrt();
    let angle = y.atan2(x);

    let shell = (1.0 - (radius / 1.05).powi(2)).max(0.0) * 0.36;
    let ring = (-((radius - 0.56) / 0.24).powi(2)).exp() * 0.34;
    let center_mass = gaussian2d(x + 0.03, y + 0.02, 0.34, 0.30) * 0.38;
    let upper_left = gaussian2d(x + 0.24, y + 0.15, 0.28, 0.18) * 0.28;
    let lower_right = gaussian2d(x - 0.22, y - 0.20, 0.22, 0.20) * 0.24;
    let spiral = (0.5 + 0.5 * (angle * 3.2 + radius * 8.4).sin()) * 0.16;
    let center_cut = gaussian2d(x - 0.02, y - 0.02, 0.18, 0.16) * 0.20;
    let bite = gaussian2d(x + 0.34, y - 0.23, 0.18, 0.14) * 0.18;
    let jitter = (noise(seed, 41.0) - 0.5) * 0.20;

    (shell + ring + center_mass + upper_left + lower_right + spiral
        - center_cut
        - bite
        + jitter)
        .clamp(0.0, 1.0)
}

fn gaussian2d(x: f32, y: f32, sigma_x: f32, sigma_y: f32) -> f32 {
    (-0.5 * ((x / sigma_x).powi(2) + (y / sigma_y).powi(2))).exp()
}

/// Cheap deterministic 1D noise. Same structure as the Swift
/// reference — `sin(value * a + seed * b) * c`, fractional part — so
/// the resulting density field has the same character.
fn noise(value: f32, seed: f32) -> f32 {
    // Mirrors the constants used by the iOS HomeAsciiParticleOrbView
    // so both platforms share the same density landscape.
    let mixed = (value * 12.9898 + seed * 78.233).sin() * 43_758.547;
    mixed - mixed.floor()
}

fn snap(value: f32) -> f32 {
    (value / SNAP_GRID).round() * SNAP_GRID
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

// Suppress an unused-import warning when the file is compiled in
// isolation: `Path` is part of the canvas API surface we may extend
// to with stroked particles in a follow-up.
#[allow(dead_code)]
fn _path_marker(_: &Path) {}
#[allow(dead_code)]
fn _canvas_marker(_: &canvas::Canvas<AsciiOrbProgram, ()>) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_in_unit_range() {
        for i in 0..256 {
            let n = noise(i as f32, (i as f32) * 1.7);
            assert!(
                (0.0..=1.0).contains(&n),
                "noise out of range at i={i}: {n}"
            );
        }
    }

    #[test]
    fn noise_is_deterministic() {
        let a = noise(7.0, 13.0);
        let b = noise(7.0, 13.0);
        assert_eq!(a, b);
    }

    #[test]
    fn core_density_is_in_unit_range() {
        for i in -10..=10 {
            for j in -10..=10 {
                let x = i as f32 / 10.0;
                let y = j as f32 / 10.0;
                let d = core_density(x, y, 17.0);
                assert!((0.0..=1.0).contains(&d), "density out of range at ({x},{y}): {d}");
            }
        }
    }

    #[test]
    fn snap_aligns_to_grid_multiples() {
        assert_eq!(snap(0.0), 0.0);
        assert_eq!(snap(SNAP_GRID), SNAP_GRID);
        assert_eq!(snap(SNAP_GRID * 2.4), SNAP_GRID * 2.0);
        assert_eq!(snap(SNAP_GRID * 2.6), SNAP_GRID * 3.0);
    }

    #[test]
    fn with_alpha_clamps_to_unit_interval() {
        let c = Color {
            r: 0.5,
            g: 0.4,
            b: 0.3,
            a: 1.0,
        };
        assert_eq!(with_alpha(c, -1.0).a, 0.0);
        assert_eq!(with_alpha(c, 0.5).a, 0.5);
        assert_eq!(with_alpha(c, 2.0).a, 1.0);
    }
}
