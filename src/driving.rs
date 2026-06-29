//! Pure driving model for Tiny Retro Racer.
//!
//! Coordinates follow the Bevy 2D convention used by the app shell: `x`
//! increases to the right, and `y` increases upward along the road.

const STOP_EPSILON: f32 = 0.001;

/// Minimal vector type that keeps the driving model free of Bevy/glam dependencies.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrivingTuning {
    pub acceleration: f32,
    pub braking: f32,
    pub drag: f32,
    pub max_speed: f32,
    pub reverse_limit: f32,
    pub turn_rate: f32,
}

impl Default for DrivingTuning {
    fn default() -> Self {
        Self {
            acceleration: 280.0,
            braking: 420.0,
            drag: 65.0,
            max_speed: 520.0,
            reverse_limit: -60.0,
            turn_rate: 2.4,
        }
    }
}

impl DrivingTuning {
    pub fn sanitized(self) -> Self {
        let fallback = Self::default();
        let max_speed = finite_positive(self.max_speed, fallback.max_speed);

        Self {
            acceleration: finite_non_negative(self.acceleration, fallback.acceleration),
            braking: finite_non_negative(self.braking, fallback.braking),
            drag: finite_non_negative(self.drag, fallback.drag),
            max_speed,
            reverse_limit: finite(self.reverse_limit, fallback.reverse_limit)
                .min(max_speed)
                .min(0.0),
            turn_rate: finite_non_negative(self.turn_rate, fallback.turn_rate),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DriverInput {
    pub accelerate: bool,
    pub brake: bool,
    pub steer_left: bool,
    pub steer_right: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarState {
    pub position: Vec2,
    pub heading_radians: f32,
    pub speed: f32,
}

impl Default for CarState {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            heading_radians: 0.0,
            speed: 0.0,
        }
    }
}

impl CarState {
    pub fn step(&mut self, input: DriverInput, tuning: DrivingTuning, delta_seconds: f32) {
        let tuning = tuning.sanitized();
        let dt = finite(delta_seconds, 0.0).clamp(0.0, 1.0 / 20.0);
        self.speed += throttle_delta(input, tuning) * dt;
        self.apply_drag(tuning.drag, dt);
        self.speed = self.speed.clamp(tuning.reverse_limit, tuning.max_speed);

        let steering = steering_axis(input);
        let speed_factor = (self.speed.abs() / tuning.max_speed).clamp(0.15, 1.0);
        self.heading_radians += steering * tuning.turn_rate * speed_factor * dt;

        self.position.x += self.heading_radians.sin() * self.speed * dt;
        self.position.y += self.heading_radians.cos() * self.speed * dt;
    }

    fn apply_drag(&mut self, drag: f32, delta_seconds: f32) {
        let drag_amount = drag * delta_seconds;
        if self.speed > 0.0 {
            self.speed = (self.speed - drag_amount).max(0.0);
        } else if self.speed < 0.0 {
            self.speed = (self.speed + drag_amount).min(0.0);
        }

        if self.speed.abs() < STOP_EPSILON {
            self.speed = 0.0;
        }
    }
}

fn throttle_delta(input: DriverInput, tuning: DrivingTuning) -> f32 {
    match (input.accelerate, input.brake) {
        (true, false) => tuning.acceleration,
        (false, true) => -tuning.braking,
        (true, true) | (false, false) => 0.0,
    }
}

fn steering_axis(input: DriverInput) -> f32 {
    match (input.steer_left, input.steer_right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        (true, true) | (false, false) => 0.0,
    }
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
    fn accelerates_forward_without_exceeding_max_speed() {
        let tuning = DrivingTuning::default();
        let mut car = CarState::default();
        let input = DriverInput {
            accelerate: true,
            ..DriverInput::default()
        };

        for _ in 0..300 {
            car.step(input, tuning, 1.0 / 60.0);
        }

        assert!(car.speed > 0.0);
        assert!(car.speed <= tuning.max_speed);
    }

    #[test]
    fn braking_is_limited_to_small_reverse_speed() {
        let tuning = DrivingTuning::default();
        let mut car = CarState::default();
        let input = DriverInput {
            brake: true,
            ..DriverInput::default()
        };

        for _ in 0..300 {
            car.step(input, tuning, 1.0 / 60.0);
        }

        assert!((car.speed - tuning.reverse_limit).abs() < f32::EPSILON);
    }

    #[test]
    fn opposite_steering_inputs_cancel_each_other() {
        let tuning = DrivingTuning::default();
        let mut car = CarState {
            speed: 200.0,
            ..CarState::default()
        };
        let input = DriverInput {
            steer_left: true,
            steer_right: true,
            ..DriverInput::default()
        };

        car.step(input, tuning, 1.0);

        assert_eq!(car.heading_radians, 0.0);
    }

    #[test]
    fn left_steering_moves_toward_negative_x() {
        let tuning = DrivingTuning::default();
        let mut car = CarState {
            speed: 200.0,
            ..CarState::default()
        };
        let input = DriverInput {
            steer_left: true,
            ..DriverInput::default()
        };

        car.step(input, tuning, 1.0 / 60.0);

        assert!(car.position.x < 0.0);
    }

    #[test]
    fn invalid_tuning_does_not_panic_or_create_nan() {
        let tuning = DrivingTuning {
            max_speed: 0.0,
            reverse_limit: f32::NAN,
            turn_rate: f32::NAN,
            ..DrivingTuning::default()
        };
        let mut car = CarState::default();

        car.step(
            DriverInput {
                accelerate: true,
                steer_right: true,
                ..DriverInput::default()
            },
            tuning,
            10.0,
        );

        assert!(car.speed.is_finite());
        assert!(car.heading_radians.is_finite());
        assert!(car.position.x.is_finite());
        assert!(car.position.y.is_finite());
    }

    #[test]
    fn tiny_drift_speeds_snap_to_zero() {
        let tuning = DrivingTuning {
            drag: 0.0,
            ..DrivingTuning::default()
        };
        let mut car = CarState {
            speed: STOP_EPSILON / 2.0,
            ..CarState::default()
        };

        car.step(DriverInput::default(), tuning, 1.0 / 60.0);

        assert!(car.speed.abs() < f32::EPSILON);
    }
}
