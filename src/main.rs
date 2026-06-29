use bevy::prelude::Vec2 as BevyVec2;
use bevy::prelude::*;
use tiny_retro_racer::driving::{CarState, DriverInput, DrivingTuning};

const ROAD_WIDTH: f32 = 420.0;
const ROAD_LENGTH: f32 = 720.0;
const CAR_WIDTH: f32 = 38.0;
const CAR_LENGTH: f32 = 66.0;
const INITIAL_CAR_Y: f32 = -220.0;

#[derive(Component)]
struct PlayerCar;

#[derive(Component)]
struct CarController {
    state: CarState,
}

#[derive(Resource, Clone, Copy)]
struct Tuning(DrivingTuning);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.08)))
        .insert_resource(Tuning(DrivingTuning::default()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tiny Retro Racer".into(),
                resolution: bevy::window::WindowResolution::new(960, 540),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update_player_car)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.18, 0.18, 0.2),
            BevyVec2::new(ROAD_WIDTH, ROAD_LENGTH),
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.9, 0.85, 0.34),
            BevyVec2::new(12.0, ROAD_LENGTH),
        ),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.1, 0.55, 0.22), BevyVec2::new(900.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.92, 0.18, 0.2),
            BevyVec2::new(CAR_WIDTH, CAR_LENGTH),
        ),
        Transform::from_xyz(0.0, INITIAL_CAR_Y, 2.0),
        PlayerCar,
        CarController {
            state: CarState {
                position: tiny_retro_racer::driving::Vec2::new(0.0, INITIAL_CAR_Y),
                ..CarState::default()
            },
        },
    ));
}

fn update_player_car(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut cars: Query<(&mut Transform, &mut CarController), With<PlayerCar>>,
) {
    let input = DriverInput {
        accelerate: keyboard.pressed(KeyCode::ArrowUp),
        brake: keyboard.pressed(KeyCode::ArrowDown),
        steer_left: keyboard.pressed(KeyCode::ArrowLeft),
        steer_right: keyboard.pressed(KeyCode::ArrowRight),
    };

    for (mut transform, mut controller) in &mut cars {
        controller
            .state
            .step(input, tuning.0, time.delta_secs().min(1.0 / 20.0));

        // The car sprite uses Bevy's default center pivot, so these safe halves
        // keep the whole car inside the centered placeholder road rectangle.
        let safe_half_width = ROAD_WIDTH * 0.5 - CAR_WIDTH * 0.5;
        let safe_half_length = ROAD_LENGTH * 0.5 - CAR_LENGTH * 0.5;
        controller.state.position.x = controller
            .state
            .position
            .x
            .clamp(-safe_half_width, safe_half_width);
        controller.state.position.y = controller
            .state
            .position
            .y
            .clamp(-safe_half_length, safe_half_length);

        transform.translation.x = controller.state.position.x;
        transform.translation.y = controller.state.position.y;
        // The placeholder car sprite points up by default; negate the model
        // heading so right turns rotate clockwise in screen space.
        transform.rotation = Quat::from_rotation_z(-controller.state.heading_radians);
    }
}
