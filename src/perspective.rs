//! Pseudo-3D projection helpers for the behind-car renderer.
//!
//! The driving model stays top-down. This module projects the current car and
//! oval track state into screen-space road samples that the Bevy layer can draw
//! as stacked strips.

use crate::driving::{CarState, Vec2};
use crate::track::TrackSpec;

const MIN_SEGMENT_COUNT: usize = 2;
const MIN_DISTANCE_GAP: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct BehindProjection {
    pub near_distance: f32,
    pub far_distance: f32,
    pub horizon_y: f32,
    pub bottom_y: f32,
    pub world_to_screen: f32,
    pub camera_depth: f32,
    pub center_x_limit: f32,
}

impl Default for BehindProjection {
    fn default() -> Self {
        Self {
            near_distance: 18.0,
            far_distance: 780.0,
            horizon_y: 54.0,
            bottom_y: -266.0,
            world_to_screen: 7.2,
            camera_depth: 125.0,
            center_x_limit: 520.0,
        }
    }
}

impl BehindProjection {
    pub fn sample(
        self,
        state: CarState,
        track: TrackSpec,
        sample_index: usize,
        sample_count: usize,
    ) -> ProjectedRoadSample {
        let track = track.sanitized();
        let sample_count = sample_count.max(MIN_SEGMENT_COUNT);
        let clamped_index = sample_index.min(sample_count - 1);
        let depth_t = clamped_index as f32 / (sample_count - 1) as f32;
        let distance = self.distance_at(depth_t);
        let angle = track_angle_near(state.position, track)
            + travel_direction(state, track) * distance / average_radius(track);
        let center = ellipse_point(track, angle);
        let lateral = dot(
            sub(center, state.position),
            right_vector(state.heading_radians),
        );
        let scale = self.screen_scale(distance);
        let screen_y = self.screen_y(depth_t);
        let road_width = (track.half_width * 2.0 * scale).clamp(44.0, 1_060.0);

        ProjectedRoadSample {
            center_x: (lateral * scale).clamp(-self.center_x_limit, self.center_x_limit),
            y: screen_y,
            road_width,
            curb_width: (14.0 * scale).clamp(4.0, 42.0),
            lane_marker_width: (4.0 * scale).clamp(2.0, 11.0),
        }
    }

    fn distance_at(self, depth_t: f32) -> f32 {
        let near = finite_non_negative(
            self.near_distance,
            BehindProjection::default().near_distance,
        );
        let far = finite_non_negative(self.far_distance, BehindProjection::default().far_distance)
            .max(near + MIN_DISTANCE_GAP);
        near + (far - near) * depth_t
    }

    fn screen_scale(self, distance: f32) -> f32 {
        let camera_depth =
            finite_positive(self.camera_depth, BehindProjection::default().camera_depth);
        let world_to_screen = finite_positive(
            self.world_to_screen,
            BehindProjection::default().world_to_screen,
        );
        world_to_screen * camera_depth / (distance + camera_depth)
    }

    fn screen_y(self, depth_t: f32) -> f32 {
        let horizon_y = finite(self.horizon_y, BehindProjection::default().horizon_y);
        let bottom_y = finite(self.bottom_y, BehindProjection::default().bottom_y);
        horizon_y + (bottom_y - horizon_y) * (1.0 - depth_t).powf(1.55)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedRoadSample {
    pub center_x: f32,
    pub y: f32,
    pub road_width: f32,
    pub curb_width: f32,
    pub lane_marker_width: f32,
}

fn track_angle_near(position: Vec2, track: TrackSpec) -> f32 {
    let radii = track.center_radii();
    (position.y / radii.y).atan2(position.x / radii.x)
}

fn travel_direction(state: CarState, track: TrackSpec) -> f32 {
    let angle = track_angle_near(state.position, track);
    let radii = track.center_radii();
    let tangent = Vec2::new(-radii.x * angle.sin(), radii.y * angle.cos());
    let forward = forward_vector(state.heading_radians);

    if dot(tangent, forward) < 0.0 {
        -1.0
    } else {
        1.0
    }
}

fn average_radius(track: TrackSpec) -> f32 {
    let radii = track.center_radii();
    ((radii.x + radii.y) * 0.5).max(1.0)
}

fn ellipse_point(track: TrackSpec, angle: f32) -> Vec2 {
    let radii = track.center_radii();
    Vec2::new(radii.x * angle.cos(), radii.y * angle.sin())
}

fn forward_vector(heading_radians: f32) -> Vec2 {
    Vec2::new(heading_radians.sin(), heading_radians.cos())
}

fn right_vector(heading_radians: f32) -> Vec2 {
    Vec2::new(heading_radians.cos(), -heading_radians.sin())
}

fn sub(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(a.x - b.x, a.y - b.y)
}

fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn finite_non_negative(value: f32, fallback: f32) -> f32 {
    finite(value, fallback).max(0.0)
}

fn finite_positive(value: f32, fallback: f32) -> f32 {
    let value = finite(value, fallback);
    if value > 0.0 { value } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_samples_are_lower_and_wider_than_far_samples() {
        let projection = BehindProjection::default();
        let state = TrackSpec::default().start_state();

        let near = projection.sample(state, TrackSpec::default(), 0, 12);
        let far = projection.sample(state, TrackSpec::default(), 11, 12);

        assert!(near.y < far.y);
        assert!(near.road_width > far.road_width);
        assert!(near.curb_width > far.curb_width);
    }

    #[test]
    fn car_lateral_offset_moves_track_center_in_view() {
        let projection = BehindProjection::default();
        let track = TrackSpec::default();
        let mut state = track.start_state();
        state.position.y += 24.0;

        let near = projection.sample(state, track, 0, 12);

        assert!(near.center_x > 0.0);
    }

    #[test]
    fn reverse_heading_samples_the_opposite_direction() {
        let projection = BehindProjection::default();
        let track = TrackSpec::default();
        let forward = track.start_state();
        let mut reverse = forward;
        reverse.heading_radians = -std::f32::consts::FRAC_PI_2;

        let forward_far = projection.sample(forward, track, 11, 12);
        let reverse_far = projection.sample(reverse, track, 11, 12);

        assert_ne!(forward_far.center_x.signum(), reverse_far.center_x.signum());
    }

    #[test]
    fn invalid_projection_values_fall_back_to_finite_samples() {
        let projection = BehindProjection {
            near_distance: f32::NAN,
            far_distance: -1.0,
            world_to_screen: f32::NAN,
            camera_depth: 0.0,
            horizon_y: f32::INFINITY,
            bottom_y: f32::NEG_INFINITY,
            center_x_limit: 500.0,
        };

        let sample = projection.sample(
            TrackSpec::default().start_state(),
            TrackSpec::default(),
            1,
            1,
        );

        assert!(sample.center_x.is_finite());
        assert!(sample.y.is_finite());
        assert!(sample.road_width.is_finite());
    }
}
