//! Pixel-particle galaxy rendered into an Iced canvas.
//!
//! The widget paints a stylised top-down spiral galaxy made entirely
//! of chunky, snapped-to-grid rectangles so it reads as ASCII / pixel
//! art rather than smooth vector geometry. Conceptually it is built
//! out of seven stacked layers, drawn back-to-front:
//!
//! 1. **Starfield** — sparse deep-space stars across the canvas with
//!    a deterministic re-seed so they twinkle without ever shifting
//!    position abruptly.
//! 2. **Galactic halo** — diffuse, cool-tinted population II stars
//!    surrounding the disc.
//! 3. **Polar jets** — bipolar AGN-style streams emerging vertically
//!    from the nucleus, hot at the base and cooling toward the tips.
//! 4. **Globular clusters** — small, slow-orbiting flecks far from
//!    the disc plane, the analogue of the previous "orbit sparks".
//! 5. **Spiral disc** — the body of the galaxy: a tilted ellipse
//!    sampled from a logarithmic-spiral density with differential
//!    rotation. Stars closer to the centre rotate faster and read as
//!    warm; stars at the rim read as cooler blue-white.
//! 6. **Arm satellites** — bright knots riding along the spiral
//!    arms, picking up the warm/cool gradient from the disc.
//! 7. **Bulge + nucleus** — the soft yellow-white bulge cradling a
//!    tight white-hot AGN core.
//! 8. **Scanline** — a barely-perceptible CRT sweep tying the whole
//!    thing back to the OpenSpace terminal aesthetic.
//!
//! All randomness is deterministic — every layer is a pure function
//! of `(seed, t)` — so the orb is cheap to redraw and never
//! allocates on the hot path beyond the geometry frame Iced manages
//! internally.

use std::time::Instant;

use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program};
use iced::Color;
use iced::Point;
use iced::Rectangle;
use iced::Renderer;
use iced::Size;
use iced::Theme;

use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::{ForegroundToken, StatusToken};

use crate::application::welcome_dynamics::{MAX_ZOOM, SPEED_CLAMP};

/// Logical canvas the particle field is laid out in. The actual
/// canvas widget is scaled to fit while preserving aspect.
const LOGICAL_SIZE: Size = Size {
    width: 360.0,
    height: 240.0,
};

/// Disc semi-major axis in logical pixels. The disc is rendered as
/// a tilted ellipse using `DISC_TILT` as the y/x squash factor —
/// roughly equivalent to viewing a flat circular disc from ~25°
/// above its plane.
const DISC_RADIUS: f32 = 116.0;
const DISC_TILT: f32 = 0.40;

/// Spiral arms. `ARM_PITCH` controls how tightly the arms wind
/// (smaller = looser). `ARM_WIDTH` is the angular sigma of each arm
/// in radians.
const ARM_COUNT: usize = 2;
const ARM_PITCH: f32 = 0.42;
const ARM_WIDTH: f32 = 0.55;

/// Number of stars carrying the disc itself.
const DISC_STAR_COUNT: usize = 360;
/// Stars riding on top of the spiral arms — brighter knots.
const ARM_SATELLITE_COUNT: usize = 96;
/// Diffuse halo stars surrounding the disc.
const HALO_STAR_COUNT: usize = 168;
/// Globular clusters orbiting outside the disc plane.
const GLOBULAR_CLUSTER_COUNT: usize = 36;
/// Soft bulge pixels cradling the nucleus.
const BULGE_BLOCK_COUNT: usize = 70;
/// White-hot AGN core pixels.
const NUCLEUS_BLOCK_COUNT: usize = 30;
/// Sparse deep-space starfield.
const STARFIELD_COUNT: usize = 90;
/// Number of pixel segments along each polar jet.
const JET_SEGMENTS: usize = 16;

/// Snap the rendered block grid to whole pixels so the cluster keeps
/// the chunky, pixel-art look from the reference rather than being
/// anti-aliased into a smooth blob.
const SNAP_GRID: f32 = 3.0;

// ---------------------------------------------------------------------------
// Hold-to-zoom dynamics
// ---------------------------------------------------------------------------
//
// The pure speed/zoom curve and its tuning constants live in
// `welcome_application::welcome_dynamics`. This file only consumes
// the resulting `speed_multiplier` and `zoom` values per frame; the
// constants used here (`SPEED_CLAMP`, `MAX_ZOOM`) are imported above
// so the canvas program can defensively clamp inputs without
// duplicating tuning numbers.

// ---------------------------------------------------------------------------
// Public canvas program
// ---------------------------------------------------------------------------

/// Iced canvas program rendering the welcome galaxy orb.
///
/// In addition to the time-driven inputs, the program carries two
/// presentation knobs that can be animated by the parent:
///
/// * `speed_multiplier` scales `t` uniformly, so the entire galaxy
///   speeds up or slows down without any layer needing its own
///   bespoke speed knob.
/// * `zoom` scales the logical→screen projection around the canvas
///   centre, so the galaxy "dollies in" and the user can read finer
///   detail (arm knots, jet wobble, nucleus shimmer).
///
/// Both fields are plain `f32`s rather than animation handles —
/// easing is the parent's job, the canvas just renders whatever it
/// is handed each frame.
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
    /// Uniform scalar applied to elapsed seconds. `1.0` is the
    /// default rest state; higher values fast-forward every layer
    /// in lockstep.
    speed_multiplier: f32,
    /// Uniform scale applied to the logical→screen projection.
    /// `1.0` fits the galaxy to its widget bounds; higher values
    /// crop in to surface the inner-disc detail.
    zoom: f32,
}

impl AsciiOrbProgram {
    /// Build the program at rest — speed and zoom both clamped to
    /// the default `1.0`. Used by call-sites that do not yet drive
    /// the click-to-zoom behaviour and by the unit tests.
    pub fn new(theme: OpenSpaceTheme, started_at: Instant, now: Instant) -> Self {
        Self::with_dynamics(theme, started_at, now, 1.0, 1.0)
    }

    /// Build the program with caller-specified speed and zoom.
    ///
    /// Both scalars are sanitised here so the render path can treat
    /// them as well-behaved: `speed_multiplier` is clamped into
    /// `[0.0, SPEED_CLAMP]` and `zoom` into `[1.0, MAX_ZOOM]`.
    /// Callers driving the hold-to-zoom behaviour should pass the
    /// *eased* (interpolated) values, not the raw progress targets,
    /// so the transition reads as a smooth dolly rather than a snap.
    pub fn with_dynamics(
        theme: OpenSpaceTheme,
        started_at: Instant,
        now: Instant,
        speed_multiplier: f32,
        zoom: f32,
    ) -> Self {
        Self {
            theme,
            started_at,
            now,
            speed_multiplier: speed_multiplier.clamp(0.0, SPEED_CLAMP),
            zoom: zoom.clamp(1.0, MAX_ZOOM),
        }
    }

    fn elapsed_seconds(&self) -> f32 {
        self.now
            .saturating_duration_since(self.started_at)
            .as_secs_f32()
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
        // Scale elapsed time by the caller-supplied multiplier so
        // every layer speeds up in lockstep. Layers see exactly the
        // same `t` they always would; the only thing that changes
        // is how quickly that `t` advances.
        let t = self.elapsed_seconds() * self.speed_multiplier;

        // Compute a uniform scale that fits the logical 360x240
        // canvas inside the actual widget bounds while preserving
        // aspect, then center the cluster. The `zoom` factor
        // multiplies the fit-scale so callers can dolly in around
        // the canvas centre without changing layout.
        let fit_scale =
            (bounds.width / LOGICAL_SIZE.width).min(bounds.height / LOGICAL_SIZE.height);
        let scale = fit_scale * self.zoom;
        let translate = Point {
            x: (bounds.width - LOGICAL_SIZE.width * scale) * 0.5,
            y: (bounds.height - LOGICAL_SIZE.height * scale) * 0.5,
        };

        let project = |p: Point| Point {
            x: translate.x + p.x * scale,
            y: translate.y + p.y * scale,
        };

        // Layer order matters — earlier calls are painted further
        // back. The disc and bulge sit on top of jets/halo so the
        // body of the galaxy reads as a solid object instead of
        // disappearing behind its own outflow.
        draw_starfield(&mut frame, &self.theme, t, scale, project);
        draw_galactic_halo(&mut frame, &self.theme, t, scale, project);
        draw_jet(&mut frame, &self.theme, t, scale, project);
        draw_globular_clusters(&mut frame, &self.theme, t, scale, project);
        draw_disc(&mut frame, &self.theme, t, scale, project);
        draw_arm_satellites(&mut frame, &self.theme, t, scale, project);
        draw_bulge(&mut frame, &self.theme, t, scale, project);
        draw_nucleus(&mut frame, &self.theme, t, scale, project);
        draw_scanline(&mut frame, &self.theme, t, scale, project);

        vec![frame.into_geometry()]
    }
}

// ---------------------------------------------------------------------------
// Galaxy layers
// ---------------------------------------------------------------------------

/// Sparse deep-space stars across the entire canvas.
///
/// Each star's screen position is anchored by its seed so the layer
/// reads as a fixed starfield rather than animated noise. Twinkle is
/// driven by a per-star sine wave so stars brighten and dim out of
/// phase with each other.
fn draw_starfield(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    for i in 0..STARFIELD_COUNT {
        let seed = 13_700.0 + i as f32;
        let x = noise(seed, 3.0) * LOGICAL_SIZE.width;
        let y = noise(seed, 7.0) * LOGICAL_SIZE.height;

        // Skip stars that fall onto the disc footprint — the disc
        // is dense enough that background stars there would just
        // muddy it. We use a generous ellipse so we don't punch a
        // visible hole in the starfield.
        let dx = x - LOGICAL_SIZE.width * 0.5;
        let dy = (y - LOGICAL_SIZE.height * 0.5) / DISC_TILT;
        if (dx * dx + dy * dy).sqrt() < DISC_RADIUS * 0.65 {
            continue;
        }

        let phase = noise(seed, 19.0) * std::f32::consts::TAU;
        let twinkle = ((t * 1.2 + phase).sin() * 0.5 + 0.5).powf(1.4);
        let alpha = (0.05 + twinkle * 0.40).clamp(0.0, 1.0);
        let size = scale.max(1.0) * (0.9 + noise(seed, 31.0) * 1.1);
        let color = star_color(theme, seed);

        let p = project(Point {
            x: snap(x),
            y: snap(y),
        });
        frame.fill_rectangle(
            Point {
                x: p.x - size * 0.5,
                y: p.y - size * 0.5,
            },
            Size {
                width: size,
                height: size,
            },
            with_alpha(color, alpha),
        );
    }
}

/// Diffuse population-II halo around the disc — the "old stars" that
/// give a galaxy its cool-tinted glow when viewed from the side.
fn draw_galactic_halo(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let cool = blend(
        theme.foreground(ForegroundToken::Muted),
        theme.status(StatusToken::Info),
        0.35,
    );
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    for i in 0..HALO_STAR_COUNT {
        let seed = 2_900.0 + i as f32;
        // Halo radius distribution: peaks at ~1.05·disc radius and
        // falls off slowly — same shape as a dark-matter halo
        // projected to 2D, just sampled cheaply.
        let radial = 0.92 + noise(seed, 3.0).powf(0.65) * 0.55;
        let angle =
            noise(seed, 11.0) * std::f32::consts::TAU + t * (0.04 + noise(seed, 19.0) * 0.03);

        let jitter_x = (noise(seed, 29.0) - 0.5) * 12.0;
        let jitter_y = (noise(seed, 37.0) - 0.5) * 9.0;

        let p = Point {
            x: center.x + angle.cos() * DISC_RADIUS * radial + jitter_x,
            y: center.y + angle.sin() * DISC_RADIUS * radial * DISC_TILT + jitter_y,
        };

        let phase = noise(seed, 47.0) * std::f32::consts::TAU;
        let twinkle = ((t * 0.7 + phase).sin() * 0.5 + 0.5) * 0.18;
        let alpha = 0.06 + twinkle;
        let size = (1.0 + noise(seed, 53.0) * 1.6) * scale;

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
            with_alpha(cool, alpha),
        );
    }
}

/// Bipolar polar jets emerging from the AGN core.
///
/// Each side is a column of pixels from the centre outward, with the
/// alpha falling off toward the tip and the colour cooling from
/// accent-orange (synchrotron-warm) at the base to info-blue at the
/// tip. The whole jet pulses on a slow swell so it reads as a living
/// outflow rather than two static columns.
fn draw_jet(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let warm = theme.foreground(ForegroundToken::Accent);
    let cool = theme.status(StatusToken::Info);
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    // Slow swell so the jet visibly pulses without ever fully
    // disappearing.
    let swell = ((t * 0.35).sin() * 0.5 + 0.5).powf(1.4);
    let intensity = 0.35 + swell * 0.55;

    let inner = 12.0;
    let outer = 96.0;

    for direction in [-1.0, 1.0] {
        for s in 0..JET_SEGMENTS {
            let f = s as f32 / (JET_SEGMENTS - 1) as f32;
            let r = inner + (outer - inner) * f;

            // Slight horizontal wobble: jets in real galaxies are
            // never perfectly straight. The wobble shifts smoothly
            // with t so the column reads as turbulent.
            let wobble = (t * 0.6 + f * 6.0 + direction).sin() * (1.0 + f * 3.5);
            let raw_x = center.x + wobble;
            let raw_y = center.y + direction * r;

            let p = project(Point {
                x: snap(raw_x),
                y: snap(raw_y),
            });

            // Falloff: brightest near the core, dim at the tip.
            let falloff = (1.0 - f).powf(1.1);
            let alpha = (intensity * falloff * 0.55).clamp(0.0, 1.0);
            // Width tapers slightly toward the tip — base is
            // chunkier than the head.
            let width = (3.0 - f * 1.6).max(1.0) * scale;
            let height = (2.0 + falloff * 1.8) * scale;
            let colour = blend(warm, cool, f * 0.85);

            frame.fill_rectangle(
                Point {
                    x: p.x - width * 0.5,
                    y: p.y - height * 0.5,
                },
                Size { width, height },
                with_alpha(colour, alpha),
            );

            // Faint side-pixels for the first half of the jet so it
            // reads as a column rather than a line.
            if f < 0.55 {
                let side_size = scale.max(1.0);
                for off in [-1.0, 1.0] {
                    frame.fill_rectangle(
                        Point {
                            x: p.x + off * width * 0.65 - side_size * 0.5,
                            y: p.y - side_size * 0.5,
                        },
                        Size {
                            width: side_size,
                            height: side_size,
                        },
                        with_alpha(colour, alpha * 0.5),
                    );
                }
            }
        }
    }
}

/// Far-flung globular clusters orbiting the disc on inclined planes.
///
/// Each cluster is a two-pixel fleck — a faint warm halo behind a
/// brighter pip — twinkling on its own phase. They orbit slowly so
/// the eye registers movement without the layer ever competing with
/// the disc.
fn draw_globular_clusters(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let warm = theme.foreground(ForegroundToken::Accent);
    let cool = theme.status(StatusToken::Info);
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    for i in 0..GLOBULAR_CLUSTER_COUNT {
        let seed = 11_200.0 + i as f32;
        let angle_offset = noise(seed, 13.0) * std::f32::consts::TAU;
        let orbit_radius = DISC_RADIUS * (0.35 + noise(seed, 17.0) * 0.55);
        // Per-cluster orbital plane tilt so they don't all fall on
        // the same ellipse.
        let plane = 0.35 + noise(seed, 31.0) * 0.55;
        let phase = noise(seed, 53.0) * std::f32::consts::TAU;

        let omega = 0.10 + noise(seed, 23.0) * 0.16;
        let angle = angle_offset + t * omega + phase;

        let p = Point {
            x: center.x + angle.cos() * orbit_radius,
            y: center.y + angle.sin() * orbit_radius * plane,
        };

        let twinkle = ((t * 1.3 + phase).sin() * 0.5 + 0.5) * 0.42;
        let size = (1.6 + noise(seed, 23.0) * 1.8) * scale;
        let colour = if noise(seed, 41.0) < 0.7 { warm } else { cool };

        let projected = project(p);
        let glow_size = size * 1.8;
        frame.fill_rectangle(
            Point {
                x: projected.x - glow_size * 0.5,
                y: projected.y - glow_size * 0.5,
            },
            Size {
                width: glow_size,
                height: glow_size,
            },
            with_alpha(colour, 0.06 + twinkle * 0.10),
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
            with_alpha(colour, 0.18 + twinkle * 0.28),
        );
    }
}

/// The body of the galaxy: a tilted disc of stars sampled from a
/// logarithmic-spiral density with differential rotation. Inner
/// stars rotate faster than outer stars, which is the behaviour that
/// produces the spiral arm pattern in real galaxies (modulo dark
/// matter — we are not solving the rotation curve here).
fn draw_disc(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    let mut placed = 0usize;
    let mut attempt = 0usize;
    let max_attempts = DISC_STAR_COUNT * 14;

    while placed < DISC_STAR_COUNT && attempt < max_attempts {
        let seed = 2_400.0 + attempt as f32;
        let nx = noise(seed, 3.0) * 2.0 - 1.0;
        let ny = noise(seed, 9.0) * 2.0 - 1.0;
        let r = (nx * nx + ny * ny).sqrt();

        if !(0.04..=1.0).contains(&r) {
            attempt += 1;
            continue;
        }

        // Rejection-sample against the spiral density so most of
        // the placed stars trace the arms rather than scattering
        // uniformly across the disc.
        let (density, arm_distance) = arm_density(nx, ny, t);
        if noise(seed, 15.0) > density {
            attempt += 1;
            continue;
        }

        // Project from disc-local unit space onto the tilted
        // ellipse in logical space.
        let raw_x = center.x + nx * DISC_RADIUS;
        let raw_y = center.y + ny * DISC_RADIUS * DISC_TILT;

        // Per-star drift so the disc doesn't read as frozen even
        // when arms are mostly stationary.
        let phase = noise(seed, 53.0) * std::f32::consts::TAU;
        let drift_x = (t * 0.5 + phase).sin() * 0.9;
        let drift_y = (t * 0.4 + phase * 1.3).cos() * 0.6;

        let block_x = snap(raw_x + drift_x);
        let block_y = snap(raw_y + drift_y);

        // Energy: blend density with arm-distance penalty so the
        // brightest pixels sit on the arm spine.
        let energy = (density * (1.0 - arm_distance * 0.4)).clamp(0.0, 1.0);

        // Colour ramp: warm yellow-orange near the centre, cooler
        // blue-white at the rim. Dust-lane darker cells are
        // produced by skipping placements where arm_distance is
        // very small AND the side of the arm is the trailing one
        // (cheap dust-lane proxy).
        let warm = theme.foreground(ForegroundToken::Accent);
        let cool = blend(
            theme.foreground(ForegroundToken::Primary),
            theme.status(StatusToken::Info),
            0.45,
        );
        let colour = blend(warm, cool, (r * 0.95).clamp(0.0, 1.0));

        let shimmer_phase = noise(seed, 71.0) * std::f32::consts::TAU;
        let shimmer = ((t * 1.4 + shimmer_phase).sin() * 0.5 + 0.5) * (0.18 + energy * 0.18);
        let base_alpha = 0.22 + energy * 0.65;
        let alpha = (base_alpha * (0.78 + shimmer)).clamp(0.05, 1.0);

        let pulse_phase = noise(seed, 89.0) * std::f32::consts::TAU;
        let pulse = (t * 1.0 + pulse_phase).sin() * 0.5 + 0.5;
        let block_size = (2.6 + energy * 4.2 + pulse * 0.7).clamp(2.0, 8.0);

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
            with_alpha(colour, alpha),
        );

        // Specular highlight on the brightest stars — a small
        // white-hot pip in the upper-left corner so the brightest
        // arm knots read as glowing rather than flat.
        if energy > 0.55 {
            let hi_size = (size * 0.32).max(scale * 1.2);
            let hot = blend(
                colour,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                0.55,
            );
            frame.fill_rectangle(
                Point {
                    x: projected.x - size * 0.5 + hi_size * 0.4,
                    y: projected.y - size * 0.5 + hi_size * 0.4,
                },
                Size {
                    width: hi_size,
                    height: hi_size,
                },
                with_alpha(hot, (alpha * 0.85).clamp(0.0, 1.0)),
            );
        }

        placed += 1;
        attempt += 1;
    }
}

/// Brighter knots riding along the arms. They sit slightly above
/// the disc layer in the z-order so the arms get a clear,
/// punctuated skeleton rather than dissolving into the background
/// disc dust.
fn draw_arm_satellites(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let warm = theme.foreground(ForegroundToken::Accent);
    let cool = blend(
        theme.foreground(ForegroundToken::Primary),
        theme.status(StatusToken::Info),
        0.45,
    );
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    let mut placed = 0usize;
    let mut attempt = 0usize;
    let max_attempts = ARM_SATELLITE_COUNT * 18;

    while placed < ARM_SATELLITE_COUNT && attempt < max_attempts {
        let seed = 5_500.0 + attempt as f32;
        // Sample directly along arms by picking r and then forcing
        // theta onto an arm spine. This guarantees the satellites
        // sit on the brightest parts of the arms regardless of how
        // strict the disc rejection sampling is.
        let r = 0.18 + noise(seed, 3.0).powf(0.8) * 0.78;
        let arm_index = (noise(seed, 7.0) * ARM_COUNT as f32).floor();
        let arm_jitter = (noise(seed, 11.0) - 0.5) * ARM_WIDTH * 0.6;
        let omega = 0.18 / (0.40 + r);
        let theta_arm = ARM_PITCH * (1.0 + r * 6.0).ln()
            + arm_index * std::f32::consts::TAU / ARM_COUNT as f32
            + arm_jitter
            - t * omega;

        let nx = r * theta_arm.cos();
        let ny = r * theta_arm.sin();

        let raw_x = center.x + nx * DISC_RADIUS;
        let raw_y = center.y + ny * DISC_RADIUS * DISC_TILT;

        let phase = noise(seed, 29.0) * std::f32::consts::TAU;
        let twinkle = ((t * 1.3 + phase).sin() * 0.5 + 0.5) * 0.55;

        let colour = blend(warm, cool, (r * 0.95).clamp(0.0, 1.0));
        let alpha = (0.30 + twinkle * 0.45).clamp(0.0, 1.0);
        let size = (2.2 + (1.0 - r) * 2.0) * scale;

        let projected = project(Point {
            x: snap(raw_x),
            y: snap(raw_y),
        });
        // Two-layer fleck: a faint glow rectangle behind, accent
        // pip on top.
        let glow_size = size * 1.7;
        frame.fill_rectangle(
            Point {
                x: projected.x - glow_size * 0.5,
                y: projected.y - glow_size * 0.5,
            },
            Size {
                width: glow_size,
                height: glow_size,
            },
            with_alpha(colour, alpha * 0.30),
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
            with_alpha(colour, alpha),
        );

        placed += 1;
        attempt += 1;
    }
}

/// Soft yellow-white bulge cradling the nucleus. We sample inside a
/// tighter Gaussian than the disc and use a warm-to-white gradient
/// keyed on radius so the bulge fades smoothly into the AGN.
fn draw_bulge(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let warm = theme.foreground(ForegroundToken::Accent);
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    let breath = (t * 0.6).sin() * 0.5 + 0.5;
    let breath_alpha = 0.55 + breath * 0.30;

    let mut placed = 0usize;
    let mut attempt = 0usize;
    let max_attempts = BULGE_BLOCK_COUNT * 16;

    while placed < BULGE_BLOCK_COUNT && attempt < max_attempts {
        let seed = 6_200.0 + attempt as f32;
        let nx = noise(seed, 3.0) * 2.0 - 1.0;
        let ny = noise(seed, 9.0) * 2.0 - 1.0;
        let density = gaussian2d(nx, ny, 0.30, 0.24);

        if noise(seed, 15.0) > density {
            attempt += 1;
            continue;
        }

        let r = (nx * nx + ny * ny).sqrt();
        let phase = noise(seed, 53.0) * std::f32::consts::TAU;
        let drift_x = (t * 0.7 + phase).sin() * 0.7;
        let drift_y = (t * 0.5 + phase * 1.3).cos() * 0.5;

        let raw_x = center.x + nx * 38.0 + drift_x;
        let raw_y = center.y + ny * 32.0 + drift_y;

        let block_x = snap(raw_x);
        let block_y = snap(raw_y);

        let hot = blend(
            warm,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            (1.0 - r * 1.3).clamp(0.0, 0.6),
        );

        let shimmer_phase = noise(seed, 71.0) * std::f32::consts::TAU;
        let shimmer = (t * 1.6 + shimmer_phase).sin() * 0.5 + 0.5;
        let alpha = (breath_alpha * density * (0.7 + shimmer * 0.3)).clamp(0.06, 0.95);
        let size = (2.4 + density * 3.4) * scale;

        let projected = project(Point {
            x: block_x,
            y: block_y,
        });
        frame.fill_rectangle(
            Point {
                x: projected.x - size * 0.5,
                y: projected.y - size * 0.5,
            },
            Size {
                width: size,
                height: size,
            },
            with_alpha(hot, alpha),
        );
        placed += 1;
        attempt += 1;
    }
}

/// AGN core — a tight cluster of white-hot pixels at the very
/// centre. Drawn last so it sits visually on top of every other
/// layer, including the bulge.
fn draw_nucleus(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let warm = theme.foreground(ForegroundToken::Accent);
    let hot = blend(
        warm,
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        0.65,
    );
    let center = Point {
        x: LOGICAL_SIZE.width * 0.5,
        y: LOGICAL_SIZE.height * 0.5,
    };

    let breath = (t * 0.85).sin() * 0.5 + 0.5;
    let breath_alpha = 0.65 + breath * 0.30;

    let mut placed = 0usize;
    let mut attempt = 0usize;
    let max_attempts = NUCLEUS_BLOCK_COUNT * 16;

    while placed < NUCLEUS_BLOCK_COUNT && attempt < max_attempts {
        let seed = 7_700.0 + attempt as f32;
        let nx = noise(seed, 3.0) * 2.0 - 1.0;
        let ny = noise(seed, 9.0) * 2.0 - 1.0;
        let density = gaussian2d(nx, ny, 0.16, 0.14);

        if noise(seed, 15.0) < density {
            let phase = noise(seed, 53.0) * std::f32::consts::TAU;
            let drift_x = (t * 0.9 + phase).sin() * 0.6;
            let drift_y = (t * 0.7 + phase * 1.3).cos() * 0.5;

            let raw_x = center.x + nx * 18.0 + drift_x;
            let raw_y = center.y + ny * 14.0 + drift_y;

            let shimmer_phase = noise(seed, 71.0) * std::f32::consts::TAU;
            let shimmer = (t * 2.4 + shimmer_phase).sin() * 0.5 + 0.5;
            let alpha = (breath_alpha * (0.7 + shimmer * 0.3)).clamp(0.2, 1.0);
            let size = (2.4 + density * 3.0) * scale;

            let projected = project(Point {
                x: snap(raw_x),
                y: snap(raw_y),
            });
            frame.fill_rectangle(
                Point {
                    x: projected.x - size * 0.5,
                    y: projected.y - size * 0.5,
                },
                Size {
                    width: size,
                    height: size,
                },
                with_alpha(hot, alpha),
            );
            placed += 1;
        }
        attempt += 1;
    }
}

/// Subtle CRT scanline. We keep this layer on top of everything but
/// at very low alpha so it ties the galaxy back to the OpenSpace
/// terminal aesthetic without competing with the disc itself.
fn draw_scanline(
    frame: &mut Frame,
    theme: &OpenSpaceTheme,
    t: f32,
    scale: f32,
    project: impl Fn(Point) -> Point,
) {
    let accent = theme.foreground(ForegroundToken::Accent);

    let cycle = 8.0;
    let phase = ((t / cycle) - (t / cycle).floor()).clamp(0.0, 1.0);
    let band_y = phase * LOGICAL_SIZE.height;

    let cols = 60;
    for c in 0..cols {
        let fx = c as f32 / (cols - 1) as f32;
        let x = fx * LOGICAL_SIZE.width;
        let jitter = (noise(c as f32, 3.0) - 0.5) * 1.2;
        let p = project(Point {
            x: snap(x),
            y: snap(band_y + jitter),
        });
        let edge = (1.0 - (fx - 0.5).abs() * 1.8).clamp(0.0, 1.0);
        let alpha = 0.06 * edge;
        let size = scale * 1.6;

        frame.fill_rectangle(
            Point {
                x: p.x - size * 0.5,
                y: p.y - size * 0.5,
            },
            Size {
                width: size,
                height: size,
            },
            with_alpha(accent, alpha),
        );
    }
}

// ---------------------------------------------------------------------------
// Density + colour helpers
// ---------------------------------------------------------------------------

/// Logarithmic-spiral arm density with differential rotation.
///
/// Returns `(density, arm_distance)`:
///
/// * `density` ∈ `[0, 1]` — probability that a star at `(x, y)`
///   belongs to the disc / an arm. Highest along arm spines and at
///   intermediate radii; falls off in the dust gap at the centre and
///   at the disc edge.
/// * `arm_distance` ∈ `[0, 1]` — angular distance to the nearest arm
///   spine, normalised so callers can use it to colour or to skip
///   placements (cheap dust-lane proxy).
///
/// The rotation angle uses `Ω(r) = 0.18 / (0.40 + r)` so inner stars
/// sweep through the spiral pattern faster than outer stars — which
/// is what produces the time-evolving arm pattern. The inputs `x`
/// and `y` are expected to be normalised to roughly the unit disc.
fn arm_density(x: f32, y: f32, t: f32) -> (f32, f32) {
    let r = (x * x + y * y).sqrt();
    if r < 1e-3 {
        return (0.0, 0.0);
    }

    let theta = y.atan2(x);
    let omega = 0.18 / (0.40 + r);
    let theta_r = theta + t * omega;

    let arm_phase = theta_r - ARM_PITCH * (1.0 + r * 6.0).ln();
    let arm_n = ARM_COUNT as f32;
    // Wrap the multi-arm phase to `[-π, π]` and divide by the arm
    // count so `arm_distance` lives in `[0, π/N]`.
    let wrapped = wrap_pi(arm_phase * arm_n) / arm_n;
    let arm_distance = (wrapped.abs() / (std::f32::consts::PI / arm_n)).clamp(0.0, 1.0);

    let arm_strength = (-(wrapped / ARM_WIDTH).powi(2) * 4.0).exp();

    // Radial envelope: a Gaussian centred at r=0.55 (the ring of
    // peak star formation) plus a soft floor that lets the bulge
    // contribute density even between arms.
    let envelope = (-((r - 0.55) / 0.32).powi(2)).exp() * 0.85 + (1.0 - r).max(0.0).powi(2) * 0.25;

    let density = (arm_strength * envelope).clamp(0.0, 1.0);
    (density, arm_distance)
}

/// Wrap an angle into the `(-π, π]` interval. Used by `arm_density`
/// to compute the angular distance to the nearest arm spine without
/// branching on quadrants.
fn wrap_pi(angle: f32) -> f32 {
    let two_pi = std::f32::consts::TAU;
    let mut a = angle % two_pi;
    if a > std::f32::consts::PI {
        a -= two_pi;
    } else if a <= -std::f32::consts::PI {
        a += two_pi;
    }
    a
}

/// Pick a deterministic colour for a background starfield star.
///
/// We bucket stars into three populations — cool blue-white, warm
/// yellow-white and rare warm orange — at roughly the proportions
/// you would see in a deep-field photograph. The bucket is decided
/// by `noise(seed, 91.0)` so the same star always picks the same
/// colour across frames.
fn star_color(theme: &OpenSpaceTheme, seed: f32) -> Color {
    let r = noise(seed, 91.0);
    let primary = theme.foreground(ForegroundToken::Primary);
    let info = theme.status(StatusToken::Info);
    let accent = theme.foreground(ForegroundToken::Accent);
    if r < 0.55 {
        blend(primary, info, 0.30)
    } else if r < 0.88 {
        blend(primary, accent, 0.28)
    } else {
        accent
    }
}

fn gaussian2d(x: f32, y: f32, sigma_x: f32, sigma_y: f32) -> f32 {
    (-0.5 * ((x / sigma_x).powi(2) + (y / sigma_y).powi(2))).exp()
}

/// Cheap deterministic 1D noise. Same structure as the Swift
/// reference — `sin(value * a + seed * b) * c`, fractional part — so
/// the resulting density field has the same character.
fn noise(value: f32, seed: f32) -> f32 {
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

/// Linearly interpolate between two colours in straight-RGB space.
/// `t` is clamped to `[0, 1]`. The alpha channel is interpolated
/// alongside the RGB components so a fully-opaque source mixed with
/// a fully-opaque target stays opaque.
fn blend(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
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
            assert!((0.0..=1.0).contains(&n), "noise out of range at i={i}: {n}");
        }
    }

    #[test]
    fn noise_is_deterministic() {
        let a = noise(7.0, 13.0);
        let b = noise(7.0, 13.0);
        assert_eq!(a, b);
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

    #[test]
    fn blend_endpoints_are_pure_inputs() {
        let a = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let b = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        assert_eq!(blend(a, b, 0.0), a);
        assert_eq!(blend(a, b, 1.0), b);
    }

    #[test]
    fn blend_midpoint_averages_components() {
        let a = Color {
            r: 0.0,
            g: 0.2,
            b: 0.4,
            a: 1.0,
        };
        let b = Color {
            r: 1.0,
            g: 0.6,
            b: 0.8,
            a: 1.0,
        };
        let mid = blend(a, b, 0.5);
        assert!((mid.r - 0.5).abs() < 1e-6);
        assert!((mid.g - 0.4).abs() < 1e-6);
        assert!((mid.b - 0.6).abs() < 1e-6);
        assert!((mid.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn blend_clamps_t_outside_unit_interval() {
        let a = Color {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };
        let b = Color {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        };
        assert_eq!(blend(a, b, -1.0), a);
        assert_eq!(blend(a, b, 2.0), b);
    }

    #[test]
    fn arm_density_returns_unit_range_values() {
        for i in -10..=10 {
            for j in -10..=10 {
                let x = i as f32 / 10.0;
                let y = j as f32 / 10.0;
                let (d, ad) = arm_density(x, y, 1.234);
                assert!(
                    (0.0..=1.0).contains(&d),
                    "density out of range at ({x},{y}): {d}"
                );
                assert!(
                    (0.0..=1.0).contains(&ad),
                    "arm_distance out of range at ({x},{y}): {ad}"
                );
            }
        }
    }

    #[test]
    fn arm_density_is_zero_at_origin() {
        let (d, ad) = arm_density(0.0, 0.0, 4.2);
        assert_eq!(d, 0.0);
        assert_eq!(ad, 0.0);
    }

    #[test]
    fn arm_density_peaks_on_an_arm() {
        // Pick a radius and walk theta around it — at least one
        // angular slice should have density > 0.4 (i.e. we are on
        // top of an arm). Using a small t so the test is stable.
        let r = 0.55_f32;
        let mut peak = 0.0_f32;
        for k in 0..72 {
            let theta = (k as f32) * std::f32::consts::TAU / 72.0;
            let x = r * theta.cos();
            let y = r * theta.sin();
            let (d, _) = arm_density(x, y, 0.0);
            if d > peak {
                peak = d;
            }
        }
        assert!(peak > 0.4, "no arm peak found at r={r}: peak={peak}");
    }

    #[test]
    fn wrap_pi_keeps_angles_in_principal_branch() {
        for i in -16..=16 {
            let raw = (i as f32) * 0.7;
            let wrapped = wrap_pi(raw);
            assert!(
                wrapped > -std::f32::consts::PI - 1e-5 && wrapped <= std::f32::consts::PI + 1e-5,
                "wrap_pi({raw}) -> {wrapped} outside (-π, π]"
            );
        }
    }

    #[test]
    fn star_color_components_are_in_unit_range() {
        let theme = OpenSpaceTheme::dark();
        for i in 0..128 {
            let c = star_color(&theme, i as f32);
            assert!((0.0..=1.0).contains(&c.r));
            assert!((0.0..=1.0).contains(&c.g));
            assert!((0.0..=1.0).contains(&c.b));
            assert!((0.0..=1.0).contains(&c.a));
        }
    }

    #[test]
    fn with_dynamics_clamps_speed_and_zoom_into_safe_range() {
        let theme = OpenSpaceTheme::dark();
        let now = Instant::now();

        let saturated = AsciiOrbProgram::with_dynamics(theme, now, now, 999.0, 999.0);
        assert!(saturated.speed_multiplier <= SPEED_CLAMP + 1e-6);
        assert!(saturated.zoom <= MAX_ZOOM + 1e-6);

        let underflow = AsciiOrbProgram::with_dynamics(theme, now, now, -5.0, 0.2);
        assert!(underflow.speed_multiplier >= 0.0);
        assert!((underflow.zoom - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_defaults_to_rest_state() {
        let theme = OpenSpaceTheme::dark();
        let now = Instant::now();
        let orb = AsciiOrbProgram::new(theme, now, now);
        assert!((orb.speed_multiplier - 1.0).abs() < 1e-6);
        assert!((orb.zoom - 1.0).abs() < 1e-6);
    }
}
