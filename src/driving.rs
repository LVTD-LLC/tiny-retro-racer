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
        let dt = delta_seconds.max(0.0);
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
        (true, false) => 1.0,
        (false, true) => -1.0,
        (true, true) | (false, false) => 0.0,
    }
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

        assert_eq!(car.speed, tuning.reverse_limit);
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
}
