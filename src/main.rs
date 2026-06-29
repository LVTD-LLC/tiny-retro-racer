use bevy::prelude::*;
use tiny_retro_racer::driving::{CarState, DriverInput, DrivingTuning};

const ROAD_WIDTH: f32 = 420.0;
const ROAD_LENGTH: f32 = 720.0;

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
                resolution: (960.0, 540.0).into(),
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
            Vec2::new(ROAD_WIDTH, ROAD_LENGTH),
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.85, 0.34), Vec2::new(12.0, ROAD_LENGTH)),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.1, 0.55, 0.22), Vec2::new(900.0, 900.0)),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

    commands.spawn((
        Sprite::from_color(Color::srgb(0.92, 0.18, 0.2), Vec2::new(38.0, 66.0)),
        Transform::from_xyz(0.0, -220.0, 2.0),
        PlayerCar,
        CarController {
            state: CarState::default(),
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
        controller.state.step(input, tuning.0, time.delta_secs());
        controller.state.position.x = controller
            .state
            .position
            .x
            .clamp(-ROAD_WIDTH * 0.42, ROAD_WIDTH * 0.42);
        controller.state.position.y = controller
            .state
            .position
            .y
            .clamp(-ROAD_LENGTH * 0.46, ROAD_LENGTH * 0.46);

        transform.translation.x = controller.state.position.x;
        transform.translation.y = -220.0 + controller.state.position.y;
        transform.rotation = Quat::from_rotation_z(-controller.state.heading_radians);
    }
}
