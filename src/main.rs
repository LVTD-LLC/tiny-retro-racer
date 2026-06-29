use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use tiny_retro_racer::driving::{CarState, DriverInput, DrivingTuning};
use tiny_retro_racer::track::TrackSpec;

const CAR_WIDTH: f32 = 38.0;
const CAR_LENGTH: f32 = 66.0;
const CAMERA_FOLLOW_DECAY: f32 = 4.0;
const CAMERA_BEHIND_DISTANCE: f32 = 120.0;
const EDGE_RECOVERY_SPEED_FACTOR: f32 = 0.82;
const PLAY_FIELD_SIZE: f32 = 980.0;

const START_BUTTON_NORMAL: Color = Color::srgb(0.16, 0.22, 0.28);
const START_BUTTON_HOVERED: Color = Color::srgb(0.22, 0.32, 0.38);
const START_BUTTON_PRESSED: Color = Color::srgb(0.33, 0.64, 0.42);

#[derive(Component)]
struct PlayerCar;

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct StartButton;

type StartButtonInteractions<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<StartButton>),
>;

#[derive(Component)]
struct CarController {
    state: CarState,
}

#[derive(Resource, Clone, Copy)]
struct Tuning(DrivingTuning);

#[derive(Resource, Clone, Copy)]
struct Track(TrackSpec);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Start,
    Playing,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.08)))
        .insert_resource(Tuning(DrivingTuning::default()))
        .insert_resource(Track(TrackSpec::default()))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tiny Retro Racer".into(),
                        resolution: bevy::window::WindowResolution::new(960, 540),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Start), setup_start_screen)
        .add_systems(
            Update,
            start_screen_input.run_if(in_state(GameState::Start)),
        )
        .add_systems(OnEnter(GameState::Playing), setup_playing)
        .add_systems(
            FixedUpdate,
            update_player_car.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (restart_session, update_follow_camera)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, -CAMERA_BEHIND_DISTANCE, 999.0),
        FollowCamera,
    ));
}

fn setup_start_screen(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::Start),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(16),
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.07, 0.09)),
        children![
            (
                Text::new("Tiny Retro Racer"),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(Color::srgb(0.94, 0.96, 0.84)),
            ),
            (
                Text::new("Arrow keys drive. R resets. Esc returns here."),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.82, 0.86)),
            ),
            (
                StartButton,
                Button,
                Node {
                    width: px(180),
                    height: px(56),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(px(10)),
                    ..default()
                },
                BackgroundColor(START_BUTTON_NORMAL),
                children![(
                    Text::new("Play"),
                    TextFont {
                        font_size: FontSize::Px(28.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.98, 0.98, 0.9)),
                )],
            ),
            (
                Text::new("Press Enter or Space"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.63, 0.72, 0.78)),
            ),
        ],
    ));
}

fn setup_playing(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    track: Res<Track>,
) {
    let track_spec = track.0.sanitized();
    let inner = track_spec.inner_radii();
    let outer = track_spec.outer_radii();
    let center = track_spec.center_radii();
    let start_state = track_spec.start_state();
    let car_image = images.add(car_pixel_art());
    let start_line_image = images.add(start_line_pixel_art());
    let tree_image = images.add(tree_pixel_art());
    let barrier_image = images.add(barrier_pixel_art());

    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Mesh2d(meshes.add(Rectangle::new(PLAY_FIELD_SIZE, PLAY_FIELD_SIZE))),
        MeshMaterial2d(materials.add(Color::srgb(0.08, 0.42, 0.18))),
        Transform::from_xyz(0.0, 0.0, -2.0),
    ));

    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Mesh2d(meshes.add(Ring::new(
            Ellipse::new(outer.x, outer.y),
            Ellipse::new(inner.x, inner.y),
        ))),
        MeshMaterial2d(materials.add(Color::srgb(0.16, 0.16, 0.18))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Mesh2d(meshes.add(Ring::new(
            Ellipse::new(center.x * 1.015, center.y * 1.015),
            Ellipse::new(center.x * 0.985, center.y * 0.985),
        ))),
        MeshMaterial2d(materials.add(Color::srgb(0.92, 0.85, 0.26))),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Sprite {
            image: start_line_image,
            custom_size: Some(Vec2::new(112.0, 16.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -center.y, 2.0),
    ));

    spawn_pixel_scenery(&mut commands, tree_image, barrier_image);

    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Sprite {
            image: car_image,
            custom_size: Some(Vec2::new(CAR_WIDTH, CAR_LENGTH)),
            ..default()
        },
        Transform::from_xyz(start_state.position.x, start_state.position.y, 3.0)
            .with_rotation(Quat::from_rotation_z(-start_state.heading_radians)),
        PlayerCar,
        CarController { state: start_state },
    ));

    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Text::new("Arrow keys drive | R reset | Esc start screen"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(0.93, 0.95, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn spawn_pixel_scenery(
    commands: &mut Commands,
    tree_image: Handle<Image>,
    barrier_image: Handle<Image>,
) {
    for (x, y, size) in [
        (-360.0, -245.0, 42.0),
        (350.0, -255.0, 38.0),
        (-340.0, 255.0, 40.0),
        (360.0, 250.0, 44.0),
        (-455.0, 40.0, 36.0),
        (455.0, -45.0, 36.0),
    ] {
        commands.spawn((
            DespawnOnExit(GameState::Playing),
            Sprite {
                image: tree_image.clone(),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, 1.0),
        ));
    }

    for (x, y) in [
        (-122.0, -326.0),
        (122.0, -326.0),
        (-122.0, 326.0),
        (122.0, 326.0),
    ] {
        commands.spawn((
            DespawnOnExit(GameState::Playing),
            Sprite {
                image: barrier_image.clone(),
                custom_size: Some(Vec2::new(42.0, 18.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 2.0),
        ));
    }
}

fn car_pixel_art() -> Image {
    pixel_image(12, 18, |x, y| {
        let tire = [22, 24, 28, 255];
        let body = [226, 45, 48, 255];
        let shadow = [126, 28, 34, 255];
        let glass = [76, 172, 202, 255];
        let light = [255, 239, 128, 255];
        let transparent = [0, 0, 0, 0];

        if ((x <= 1 || x >= 10) && (4..=14).contains(&y))
            || ((x == 2 || x == 9) && (15..=16).contains(&y))
        {
            tire
        } else if (4..=7).contains(&x) && y <= 1 {
            light
        } else if (3..=8).contains(&x) && (4..=7).contains(&y) {
            glass
        } else if (4..=7).contains(&x) && (13..=16).contains(&y) {
            shadow
        } else if (2..=9).contains(&x) && (1..=16).contains(&y) {
            body
        } else {
            transparent
        }
    })
}

fn start_line_pixel_art() -> Image {
    pixel_image(16, 3, |x, y| {
        if (x + y) % 2 == 0 {
            [245, 245, 232, 255]
        } else {
            [18, 20, 24, 255]
        }
    })
}

fn tree_pixel_art() -> Image {
    pixel_image(8, 8, |x, y| {
        let transparent = [0, 0, 0, 0];
        let leaf_dark = [24, 96, 44, 255];
        let leaf_light = [62, 168, 70, 255];
        let trunk = [104, 72, 42, 255];

        if (3..=4).contains(&x) && y >= 5 {
            trunk
        } else if (1..=6).contains(&x) && (1..=5).contains(&y) {
            if (x + y) % 2 == 0 {
                leaf_light
            } else {
                leaf_dark
            }
        } else if (2..=5).contains(&x) && y == 0 {
            leaf_light
        } else {
            transparent
        }
    })
}

fn barrier_pixel_art() -> Image {
    pixel_image(8, 4, |x, y| {
        if (x / 2 + y) % 2 == 0 {
            [232, 48, 50, 255]
        } else {
            [245, 245, 232, 255]
        }
    })
}

fn pixel_image(width: u32, height: u32, mut color_at: impl FnMut(u32, u32) -> [u8; 4]) -> Image {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            data.extend_from_slice(&color_at(x, y));
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn start_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut buttons: StartButtonInteractions,
) {
    let keyboard_start =
        keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space);
    let mut should_start = keyboard_start;

    for (interaction, mut color) in &mut buttons {
        match *interaction {
            Interaction::Pressed => {
                *color = START_BUTTON_PRESSED.into();
                should_start = true;
            }
            Interaction::Hovered => {
                *color = START_BUTTON_HOVERED.into();
            }
            Interaction::None => {
                *color = START_BUTTON_NORMAL.into();
            }
        }
    }

    if should_start {
        next_state.set(GameState::Playing);
    }
}

fn update_player_car(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    tuning: Res<Tuning>,
    track: Res<Track>,
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

        let recovery = track.0.recover_position(controller.state.position);
        if recovery.corrected {
            controller.state.position = recovery.position;
            controller.state.speed *= EDGE_RECOVERY_SPEED_FACTOR;
        }

        transform.translation.x = controller.state.position.x;
        transform.translation.y = controller.state.position.y;
        // The placeholder car sprite points up by default; negate the model
        // heading so right turns rotate clockwise in screen space.
        transform.rotation = Quat::from_rotation_z(-controller.state.heading_radians);
    }
}

fn restart_session(
    keyboard: Res<ButtonInput<KeyCode>>,
    track: Res<Track>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cars: Query<(&mut Transform, &mut CarController), With<PlayerCar>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Start);
        return;
    }

    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    let start_state = track.0.start_state();
    for (mut transform, mut controller) in &mut cars {
        controller.state = start_state;
        transform.translation.x = start_state.position.x;
        transform.translation.y = start_state.position.y;
        transform.rotation = Quat::from_rotation_z(-start_state.heading_radians);
    }
}

fn update_follow_camera(
    time: Res<Time>,
    mut cameras: Query<&mut Transform, (With<FollowCamera>, Without<PlayerCar>)>,
    cars: Query<&CarController, With<PlayerCar>>,
) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    let Ok(controller) = cars.single() else {
        return;
    };

    let forward = Vec2::new(
        controller.state.heading_radians.sin(),
        controller.state.heading_radians.cos(),
    );
    let target = Vec3::new(
        controller.state.position.x - forward.x * CAMERA_BEHIND_DISTANCE,
        controller.state.position.y - forward.y * CAMERA_BEHIND_DISTANCE,
        camera.translation.z,
    );

    camera
        .translation
        .smooth_nudge(&target, CAMERA_FOLLOW_DECAY, time.delta_secs());
}
