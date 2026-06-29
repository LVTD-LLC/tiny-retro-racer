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
const CENTER_RECOVERY_RADIUS: f32 = 0.25;
const TANGENT_FLIP_DOT_EPSILON: f32 = 0.001;

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

    pub fn with_margin(self, margin: f32) -> Self {
        let spec = self.sanitized();
        Self {
            half_width: (spec.half_width - margin.max(0.0)).max(MIN_HALF_WIDTH),
            ..spec
        }
    }

    pub fn recover_position_with_margin(self, position: Vec2, margin: f32) -> TrackRecovery {
        self.with_margin(margin).recover_position(position)
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
                heading_radians: None,
                corrected: false,
            };
        }

        if !radius.is_finite()
            || radius <= CENTER_EPSILON
            || radius < inner * CENTER_RECOVERY_RADIUS
        {
            return TrackRecovery {
                position: spec.start_position(),
                heading_radians: Some(FRAC_PI_2),
                corrected: true,
            };
        }

        let target_radius = radius.clamp(inner, outer);
        let scale = target_radius / radius;
        let position = Vec2::new(position.x * scale, position.y * scale);

        TrackRecovery {
            position,
            heading_radians: None,
            corrected: true,
        }
    }

    pub fn contains(self, position: Vec2) -> bool {
        !self.recover_position(position).corrected
    }

    pub fn contains_with_margin(self, position: Vec2, margin: f32) -> bool {
        !self
            .recover_position_with_margin(position, margin)
            .corrected
    }

    pub fn recovery_heading(self, position: Vec2, current_heading: f32) -> f32 {
        let spec = self.sanitized();
        let angle = (position.y / spec.center_radius_y).atan2(position.x / spec.center_radius_x);
        let tangent = Vec2::new(
            -spec.center_radius_x * angle.sin(),
            spec.center_radius_y * angle.cos(),
        );
        let current_forward = Vec2::new(current_heading.sin(), current_heading.cos());
        let dot = tangent.x * current_forward.x + tangent.y * current_forward.y;
        let tangent = if dot < -TANGENT_FLIP_DOT_EPSILON {
            Vec2::new(-tangent.x, -tangent.y)
        } else {
            tangent
        };

        tangent.x.atan2(tangent.y)
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
    pub heading_radians: Option<f32>,
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
        assert_eq!(recovery.heading_radians, None);
        assert!(!recovery.corrected);
    }

    #[test]
    fn recover_position_clamps_outside_outer_edge() {
        let track = TrackSpec::default();
        let position = Vec2::new(0.0, -1_000.0);

        let recovery = track.recover_position(position);

        assert!(recovery.corrected);
        assert!(track.contains(recovery.position));
        assert_eq!(recovery.heading_radians, None);
        assert!(recovery.position.y > position.y);
    }

    #[test]
    fn recover_position_pushes_inside_center_back_to_road() {
        let track = TrackSpec::default();

        let recovery = track.recover_position(Vec2::ZERO);

        assert!(recovery.corrected);
        assert!(track.contains(recovery.position));
        assert_eq!(recovery.heading_radians, Some(FRAC_PI_2));
        assert!(recovery.position.y < 0.0);
    }

    #[test]
    fn near_center_recovery_uses_safe_start_instead_of_huge_scale() {
        let track = TrackSpec::default();

        let recovery = track.recover_position(Vec2::new(0.01, 0.0));

        assert!(recovery.corrected);
        assert_eq!(recovery.position, track.start_position());
        assert_eq!(recovery.heading_radians, Some(FRAC_PI_2));
    }

    #[test]
    fn recovery_heading_follows_track_tangent_nearest_current_direction() {
        let track = TrackSpec::default();
        let position = track.start_position();

        let heading = track.recovery_heading(position, 0.0);

        assert!((heading - FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn recovery_heading_handles_cardinal_track_positions() {
        let track = TrackSpec::default();
        let center = track.center_radii();

        assert_heading_close(track.recovery_heading(Vec2::new(center.x, 0.0), 0.0), 0.0);
        assert_heading_close(
            track.recovery_heading(Vec2::new(0.0, center.y), 0.0),
            -FRAC_PI_2,
        );
        assert_heading_close(
            track.recovery_heading(Vec2::new(-center.x, 0.0), std::f32::consts::PI),
            std::f32::consts::PI,
        );
        assert_heading_close(
            track.recovery_heading(Vec2::new(0.0, -center.y), 0.0),
            FRAC_PI_2,
        );
    }

    #[test]
    fn recovery_heading_is_consistent_on_inner_and_outer_edges() {
        let track = TrackSpec::default();

        for radii in [track.inner_radii(), track.outer_radii()] {
            assert_heading_close(track.recovery_heading(Vec2::new(radii.x, 0.0), 0.0), 0.0);
            assert_heading_close(
                track.recovery_heading(Vec2::new(0.0, -radii.y), 0.0),
                FRAC_PI_2,
            );
        }
    }

    #[test]
    fn recovery_heading_flips_to_match_reverse_travel_direction() {
        let track = TrackSpec::default();
        let position = track.start_position();

        let heading = track.recovery_heading(position, -FRAC_PI_2);

        assert_heading_close(heading, -FRAC_PI_2);
    }

    #[test]
    fn margin_reduces_recoverable_band_for_car_footprint() {
        let track = TrackSpec::default();
        let margin = 40.0;

        let original_outer = track.outer_radii();
        let safe_outer = track.with_margin(margin).outer_radii();

        assert!(safe_outer.x < original_outer.x);
        assert!(safe_outer.y < original_outer.y);
        assert!(track.contains_with_margin(track.start_position(), margin));
    }

    #[test]
    fn margin_recovery_keeps_footprint_center_inside_safe_band() {
        let track = TrackSpec::default();
        let margin = 40.0;
        let original_outer = track.outer_radii();
        let boundary_center = Vec2::new(0.0, -original_outer.y);

        let recovery = track.recover_position_with_margin(boundary_center, margin);

        assert!(recovery.corrected);
        assert!(track.contains(recovery.position));
        assert!(track.contains_with_margin(recovery.position, margin));
        assert!(recovery.position.y > boundary_center.y);
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

    fn assert_heading_close(actual: f32, expected: f32) {
        assert!(
            angle_distance(actual, expected) < 1e-5,
            "expected {expected}, got {actual}"
        );
    }

    fn angle_distance(a: f32, b: f32) -> f32 {
        let mut diff = (a - b).rem_euclid(std::f32::consts::TAU);
        if diff > std::f32::consts::PI {
            diff = std::f32::consts::TAU - diff;
        }
        diff.abs()
    }
}
