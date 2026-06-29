//! Testable track model for Tiny Retro Racer.
//!
//! The first playable uses an oval road represented as an elliptical band. That
//! gives the Bevy layer a simple shape to draw while keeping recovery rules in
//! pure Rust.

use std::f32::consts::FRAC_PI_2;

use crate::driving::{CarState, Vec2};

const MIN_RADIUS: f32 = 32.0;
const MIN_HALF_WIDTH: f32 = 12.0;
const CENTER_EPSILON: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSpec {
    pub center_radius_x: f32,
    pub center_radius_y: f32,
    pub half_width: f32,
}

impl Default for TrackSpec {
    fn default() -> Self {
        Self {
            center_radius_x: 230.0,
            center_radius_y: 320.0,
            half_width: 86.0,
        }
    }
}

impl TrackSpec {
    pub fn sanitized(self) -> Self {
        let fallback = Self::default();

        Self {
            center_radius_x: finite_positive(self.center_radius_x, fallback.center_radius_x)
                .max(MIN_RADIUS),
            center_radius_y: finite_positive(self.center_radius_y, fallback.center_radius_y)
                .max(MIN_RADIUS),
            half_width: finite_positive(self.half_width, fallback.half_width).max(MIN_HALF_WIDTH),
        }
    }

    pub fn start_state(self) -> CarState {
        let spec = self.sanitized();

        CarState {
            position: spec.start_position(),
            heading_radians: FRAC_PI_2,
            speed: 0.0,
        }
    }

    pub fn start_position(self) -> Vec2 {
        let spec = self.sanitized();
        Vec2::new(0.0, -spec.center_radius_y)
    }

    pub fn inner_scale(self) -> f32 {
        (1.0 - self.band_half_scale()).max(0.1)
    }

    pub fn outer_scale(self) -> f32 {
        1.0 + self.band_half_scale()
    }

    pub fn inner_radii(self) -> Vec2 {
        let spec = self.sanitized();
        let scale = spec.inner_scale();
        Vec2::new(spec.center_radius_x * scale, spec.center_radius_y * scale)
    }

    pub fn outer_radii(self) -> Vec2 {
        let spec = self.sanitized();
        let scale = spec.outer_scale();
        Vec2::new(spec.center_radius_x * scale, spec.center_radius_y * scale)
    }

    pub fn center_radii(self) -> Vec2 {
        let spec = self.sanitized();
        Vec2::new(spec.center_radius_x, spec.center_radius_y)
    }

    pub fn recover_position(self, position: Vec2) -> TrackRecovery {
        let spec = self.sanitized();
        let radius = spec.normalized_radius(position);
        let inner = spec.inner_scale();
        let outer = spec.outer_scale();

        if radius.is_finite() && (inner..=outer).contains(&radius) {
            return TrackRecovery {
                position,
                corrected: false,
            };
        }

        if !radius.is_finite() || radius <= CENTER_EPSILON {
            return TrackRecovery {
                position: spec.start_position(),
                corrected: true,
            };
        }

        let target_radius = radius.clamp(inner, outer);
        let scale = target_radius / radius;

        TrackRecovery {
            position: Vec2::new(position.x * scale, position.y * scale),
            corrected: true,
        }
    }

    pub fn contains(self, position: Vec2) -> bool {
        !self.recover_position(position).corrected
    }

    fn normalized_radius(self, position: Vec2) -> f32 {
        let spec = self.sanitized();
        let x = position.x / spec.center_radius_x;
        let y = position.y / spec.center_radius_y;
        (x * x + y * y).sqrt()
    }

    fn band_half_scale(self) -> f32 {
        let spec = self.sanitized();
        let narrow_radius = spec.center_radius_x.min(spec.center_radius_y);
        (spec.half_width / narrow_radius).clamp(0.05, 0.85)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackRecovery {
    pub position: Vec2,
    pub corrected: bool,
}

fn finite_positive(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_state_is_on_the_track_and_points_along_the_loop() {
        let track = TrackSpec::default();
        let start = track.start_state();

        assert!(track.contains(start.position));
        assert!((start.heading_radians - FRAC_PI_2).abs() < f32::EPSILON);
    }

    #[test]
    fn recover_position_leaves_on_track_positions_alone() {
        let track = TrackSpec::default();
        let position = track.start_position();

        let recovery = track.recover_position(position);

        assert_eq!(recovery.position, position);
        assert!(!recovery.corrected);
    }

    #[test]
    fn recover_position_clamps_outside_outer_edge() {
        let track = TrackSpec::default();
        let position = Vec2::new(0.0, -1_000.0);

        let recovery = track.recover_position(position);

        assert!(recovery.corrected);
        assert!(track.contains(recovery.position));
        assert!(recovery.position.y > position.y);
    }

    #[test]
    fn recover_position_pushes_inside_center_back_to_road() {
        let track = TrackSpec::default();

        let recovery = track.recover_position(Vec2::ZERO);

        assert!(recovery.corrected);
        assert!(track.contains(recovery.position));
        assert!(recovery.position.y < 0.0);
    }

    #[test]
    fn invalid_track_values_fall_back_to_a_playable_shape() {
        let track = TrackSpec {
            center_radius_x: f32::NAN,
            center_radius_y: 0.0,
            half_width: f32::INFINITY,
        };

        let start = track.start_state();

        assert!(track.contains(start.position));
        assert!(start.position.x.is_finite());
        assert!(start.position.y.is_finite());
    }
}
