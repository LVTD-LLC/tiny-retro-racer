use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use tiny_retro_racer::driving::{CarState, DriverInput, DrivingTuning};
use tiny_retro_racer::perspective::BehindProjection;
use tiny_retro_racer::pixel_art::{self, PixelArt};
use tiny_retro_racer::track::TrackSpec;

const CAR_WIDTH: f32 = 38.0;
const CAR_LENGTH: f32 = 66.0;
const CAR_FOOTPRINT_PADDING: f32 = 2.0;
const CAMERA_FOLLOW_DECAY: f32 = 4.0;
const CAMERA_BEHIND_DISTANCE: f32 = 120.0;
const CAMERA_MAX_DELTA_SECONDS: f32 = 1.0 / 20.0;
const PLAY_FIELD_SIZE: f32 = 980.0;
const BEHIND_ROAD_STRIPS: usize = 42;
const BEHIND_ROAD_SAMPLE_COUNT: usize = BEHIND_ROAD_STRIPS + 1;
const BEHIND_BACKGROUND_WIDTH: f32 = 1_400.0;
const BEHIND_BACKGROUND_HEIGHT: f32 = 720.0;
const BEHIND_PLAYER_CAR_WIDTH: f32 = 132.0;
const BEHIND_PLAYER_CAR_HEIGHT: f32 = 106.0;
const BEHIND_PLAYER_CAR_Y: f32 = -204.0;
const BEHIND_BACKGROUND_Z: f32 = 0.0;
const BEHIND_ROAD_Z: f32 = 5.0;
const BEHIND_PLAYER_CAR_Z: f32 = 40.0;

const START_BUTTON_NORMAL: Color = Color::srgb(0.16, 0.22, 0.28);
const START_BUTTON_HOVERED: Color = Color::srgb(0.22, 0.32, 0.38);
const START_BUTTON_PRESSED: Color = Color::srgb(0.33, 0.64, 0.42);

#[derive(Component)]
struct PlayerCar;

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct GameplayEntity;

#[derive(Component)]
struct OverheadViewEntity;

#[derive(Component)]
struct BehindViewEntity;

#[derive(Component)]
struct BehindPlayerCar;

#[derive(Component)]
struct HudText;

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

#[derive(Resource, Clone, Copy, Debug, Eq, PartialEq)]
struct ViewModeState {
    mode: ViewMode,
}

impl Default for ViewModeState {
    fn default() -> Self {
        Self {
            mode: ViewMode::Behind,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ViewMode {
    Behind,
    Overhead,
}

impl ViewMode {
    fn toggled(self) -> Self {
        match self {
            Self::Behind => Self::Overhead,
            Self::Overhead => Self::Behind,
        }
    }
}

#[derive(Component, Clone, Copy)]
struct BehindRoadPiece {
    index: usize,
    kind: BehindRoadPieceKind,
}

#[derive(Clone, Copy)]
enum BehindRoadPieceKind {
    Road,
    LeftCurb,
    RightCurb,
    LaneMarker,
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
        .insert_resource(ViewModeState::default())
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
        .add_systems(OnExit(GameState::Playing), cleanup_gameplay_entities)
        .add_systems(
            FixedUpdate,
            update_player_car.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                restart_session,
                update_view_mode_input,
                update_follow_camera,
                update_view_visibility,
                update_hud_text,
                update_behind_view,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 999.0), FollowCamera));
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
                Text::new("Arrow keys drive. V switches view. R resets. Esc returns here."),
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
    view_mode: Res<ViewModeState>,
    mut cameras: Query<&mut Transform, With<FollowCamera>>,
) {
    let track_spec = track.0.sanitized();
    let inner = track_spec.inner_radii();
    let outer = track_spec.outer_radii();
    let center = track_spec.center_radii();
    let start_state = track_spec.start_state();
    let color_image = images.add(white_pixel_image());
    let car_image = images.add(pixel_image(pixel_art::car()));
    let rear_car_image = images.add(pixel_image(pixel_art::rear_race_car()));
    let start_line_image = images.add(pixel_image(pixel_art::start_line()));
    let tree_image = images.add(pixel_image(pixel_art::tree()));
    let barrier_image = images.add(pixel_image(pixel_art::barrier()));
    let overhead_visibility = view_visibility(ViewMode::Overhead, view_mode.mode);

    if let Ok(mut camera) = cameras.single_mut() {
        camera.translation =
            camera_target_for_view(&start_state, camera.translation.z, view_mode.mode);
    }

    commands.spawn((
        GameplayEntity,
        OverheadViewEntity,
        overhead_visibility,
        Mesh2d(meshes.add(Rectangle::new(PLAY_FIELD_SIZE, PLAY_FIELD_SIZE))),
        MeshMaterial2d(materials.add(Color::srgb(0.08, 0.42, 0.18))),
        Transform::from_xyz(0.0, 0.0, -2.0),
    ));

    commands.spawn((
        GameplayEntity,
        OverheadViewEntity,
        overhead_visibility,
        Mesh2d(meshes.add(Ring::new(
            Ellipse::new(outer.x, outer.y),
            Ellipse::new(inner.x, inner.y),
        ))),
        MeshMaterial2d(materials.add(Color::srgb(0.16, 0.16, 0.18))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        GameplayEntity,
        OverheadViewEntity,
        overhead_visibility,
        Mesh2d(meshes.add(Ring::new(
            Ellipse::new(center.x * 1.015, center.y * 1.015),
            Ellipse::new(center.x * 0.985, center.y * 0.985),
        ))),
        MeshMaterial2d(materials.add(Color::srgb(0.92, 0.85, 0.26))),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    commands.spawn((
        GameplayEntity,
        OverheadViewEntity,
        overhead_visibility,
        Sprite {
            image: start_line_image,
            custom_size: Some(Vec2::new(112.0, 16.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -center.y, 2.0),
    ));

    spawn_pixel_scenery(
        &mut commands,
        tree_image,
        barrier_image,
        overhead_visibility,
    );
    spawn_behind_view_scene(&mut commands, color_image, rear_car_image, view_mode.mode);

    commands.spawn((
        GameplayEntity,
        OverheadViewEntity,
        overhead_visibility,
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
        GameplayEntity,
        HudText,
        Text::new(hud_text_for(view_mode.mode)),
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

fn cleanup_gameplay_entities(
    mut commands: Commands,
    entities: Query<Entity, With<GameplayEntity>>,
) {
    // OnExit runs before the next state's OnEnter systems, so old track,
    // scenery, HUD, and car entities are gone before a later Play rebuild.
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

fn spawn_pixel_scenery(
    commands: &mut Commands,
    tree_image: Handle<Image>,
    barrier_image: Handle<Image>,
    visibility: Visibility,
) {
    // These sit outside the default oval's outer radii and inside the 980px
    // grass field, so scenery decorates only the grass around the road.
    for (x, y, size) in [
        (-360.0, -245.0, 42.0),
        (350.0, -255.0, 38.0),
        (-340.0, 255.0, 40.0),
        (360.0, 250.0, 44.0),
        (-455.0, 40.0, 36.0),
        (455.0, -45.0, 36.0),
    ] {
        commands.spawn((
            GameplayEntity,
            OverheadViewEntity,
            visibility,
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
            GameplayEntity,
            OverheadViewEntity,
            visibility,
            Sprite {
                image: barrier_image.clone(),
                custom_size: Some(Vec2::new(42.0, 18.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 2.0),
        ));
    }
}

fn spawn_behind_view_scene(
    commands: &mut Commands,
    color_image: Handle<Image>,
    rear_car_image: Handle<Image>,
    active_mode: ViewMode,
) {
    let visibility = view_visibility(ViewMode::Behind, active_mode);

    spawn_behind_color_sprite(
        commands,
        color_image.clone(),
        visibility,
        Color::srgb(0.19, 0.47, 0.93),
        Vec2::new(BEHIND_BACKGROUND_WIDTH, BEHIND_BACKGROUND_HEIGHT),
        Vec3::new(0.0, 210.0, BEHIND_BACKGROUND_Z),
    );
    spawn_behind_color_sprite(
        commands,
        color_image.clone(),
        visibility,
        Color::srgb(0.11, 0.54, 0.08),
        Vec2::new(BEHIND_BACKGROUND_WIDTH, 380.0),
        Vec3::new(0.0, -170.0, BEHIND_BACKGROUND_Z + 0.1),
    );
    spawn_behind_color_sprite(
        commands,
        color_image.clone(),
        visibility,
        Color::srgb(0.54, 0.58, 0.55),
        Vec2::new(BEHIND_BACKGROUND_WIDTH, 14.0),
        Vec3::new(0.0, 58.0, BEHIND_BACKGROUND_Z + 0.2),
    );
    spawn_behind_color_sprite(
        commands,
        color_image.clone(),
        visibility,
        Color::srgb(0.34, 0.61, 0.72),
        Vec2::new(BEHIND_BACKGROUND_WIDTH, 18.0),
        Vec3::new(0.0, 44.0, BEHIND_BACKGROUND_Z + 0.3),
    );
    spawn_behind_color_sprite(
        commands,
        color_image.clone(),
        visibility,
        Color::srgb(0.95, 0.96, 0.9),
        Vec2::new(92.0, 24.0),
        Vec3::new(-122.0, 164.0, BEHIND_BACKGROUND_Z + 1.0),
    );
    spawn_behind_color_sprite(
        commands,
        color_image.clone(),
        visibility,
        Color::srgb(0.95, 0.96, 0.9),
        Vec2::new(54.0, 18.0),
        Vec3::new(-74.0, 172.0, BEHIND_BACKGROUND_Z + 1.1),
    );
    spawn_behind_color_sprite(
        commands,
        color_image.clone(),
        visibility,
        Color::srgb(0.95, 0.96, 0.9),
        Vec2::new(38.0, 14.0),
        Vec3::new(-168.0, 158.0, BEHIND_BACKGROUND_Z + 1.2),
    );

    for index in 0..BEHIND_ROAD_STRIPS {
        for kind in [
            BehindRoadPieceKind::Road,
            BehindRoadPieceKind::LeftCurb,
            BehindRoadPieceKind::RightCurb,
            BehindRoadPieceKind::LaneMarker,
        ] {
            commands.spawn((
                GameplayEntity,
                BehindViewEntity,
                BehindRoadPiece { index, kind },
                visibility,
                color_sprite(
                    color_image.clone(),
                    Color::srgb(0.12, 0.13, 0.14),
                    Vec2::ZERO,
                ),
                Transform::from_xyz(0.0, 0.0, road_piece_z(index, kind)),
            ));
        }
    }

    commands.spawn((
        GameplayEntity,
        BehindViewEntity,
        BehindPlayerCar,
        visibility,
        Sprite {
            image: rear_car_image,
            custom_size: Some(Vec2::new(BEHIND_PLAYER_CAR_WIDTH, BEHIND_PLAYER_CAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, BEHIND_PLAYER_CAR_Y, BEHIND_PLAYER_CAR_Z),
    ));
}

fn spawn_behind_color_sprite(
    commands: &mut Commands,
    image: Handle<Image>,
    visibility: Visibility,
    color: Color,
    size: Vec2,
    translation: Vec3,
) {
    commands.spawn((
        GameplayEntity,
        BehindViewEntity,
        visibility,
        color_sprite(image, color, size),
        Transform::from_translation(translation),
    ));
}

fn color_sprite(image: Handle<Image>, color: Color, size: Vec2) -> Sprite {
    Sprite {
        image,
        color,
        custom_size: Some(size),
        ..default()
    }
}

fn pixel_image(art: PixelArt) -> Image {
    let width = art.width;
    let height = art.height;
    let data = art.into_rgba_bytes();

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn white_pixel_image() -> Image {
    Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
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

fn update_view_mode_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut view_mode: ResMut<ViewModeState>,
) {
    let next_mode = if keyboard.just_pressed(KeyCode::Digit1) {
        Some(ViewMode::Behind)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(ViewMode::Overhead)
    } else if keyboard.just_pressed(KeyCode::KeyV) {
        Some(view_mode.mode.toggled())
    } else {
        None
    };

    if let Some(next_mode) = next_mode
        && next_mode != view_mode.mode
    {
        view_mode.mode = next_mode;
    }
}

fn update_view_visibility(
    view_mode: Res<ViewModeState>,
    mut overhead_entities: Query<
        &mut Visibility,
        (With<OverheadViewEntity>, Without<BehindViewEntity>),
    >,
    mut behind_entities: Query<
        &mut Visibility,
        (With<BehindViewEntity>, Without<OverheadViewEntity>),
    >,
) {
    if !view_mode.is_changed() {
        return;
    }

    for mut visibility in &mut overhead_entities {
        *visibility = view_visibility(ViewMode::Overhead, view_mode.mode);
    }

    for mut visibility in &mut behind_entities {
        *visibility = view_visibility(ViewMode::Behind, view_mode.mode);
    }
}

fn update_hud_text(view_mode: Res<ViewModeState>, mut texts: Query<&mut Text, With<HudText>>) {
    if !view_mode.is_changed() {
        return;
    }

    for mut text in &mut texts {
        text.0 = hud_text_for(view_mode.mode).to_string();
    }
}

fn update_behind_view(
    track: Res<Track>,
    cars: Query<&CarController, With<PlayerCar>>,
    mut pieces: Query<(&BehindRoadPiece, &mut Sprite, &mut Transform)>,
    mut rear_cars: Query<&mut Transform, (With<BehindPlayerCar>, Without<BehindRoadPiece>)>,
) {
    let Ok(controller) = cars.single() else {
        return;
    };
    let projection = BehindProjection::default();

    for (piece, mut sprite, mut transform) in &mut pieces {
        let near = projection.sample(
            controller.state,
            track.0,
            piece.index,
            BEHIND_ROAD_SAMPLE_COUNT,
        );
        let far = projection.sample(
            controller.state,
            track.0,
            piece.index + 1,
            BEHIND_ROAD_SAMPLE_COUNT,
        );
        let segment_height = (far.y - near.y).abs() + 2.5;
        let y = (near.y + far.y) * 0.5;
        let lane_visible = piece.index > 5 && piece.index % 5 == 0;

        match piece.kind {
            BehindRoadPieceKind::Road => {
                sprite.color = if piece.index % 2 == 0 {
                    Color::srgb(0.19, 0.2, 0.2)
                } else {
                    Color::srgb(0.15, 0.16, 0.17)
                };
                sprite.custom_size = Some(Vec2::new(
                    near.road_width.max(far.road_width),
                    segment_height,
                ));
                transform.translation.x = near.center_x;
                transform.translation.y = y;
            }
            BehindRoadPieceKind::LeftCurb => {
                sprite.color = curb_color(piece.index);
                sprite.custom_size = Some(Vec2::new(near.curb_width, segment_height + 1.0));
                transform.translation.x =
                    near.center_x - near.road_width * 0.5 - near.curb_width * 0.5;
                transform.translation.y = y;
            }
            BehindRoadPieceKind::RightCurb => {
                sprite.color = curb_color(piece.index + 1);
                sprite.custom_size = Some(Vec2::new(near.curb_width, segment_height + 1.0));
                transform.translation.x =
                    near.center_x + near.road_width * 0.5 + near.curb_width * 0.5;
                transform.translation.y = y;
            }
            BehindRoadPieceKind::LaneMarker => {
                sprite.color = if lane_visible {
                    Color::srgb(0.9, 0.92, 0.9)
                } else {
                    Color::srgba(0.9, 0.92, 0.9, 0.0)
                };
                sprite.custom_size = Some(Vec2::new(
                    near.lane_marker_width,
                    if lane_visible {
                        (segment_height * 0.66).max(3.0)
                    } else {
                        0.0
                    },
                ));
                transform.translation.x = near.center_x;
                transform.translation.y = y;
            }
        }

        transform.translation.z = road_piece_z(piece.index, piece.kind);
    }

    for mut transform in &mut rear_cars {
        transform.translation.x = 0.0;
        transform.translation.y = BEHIND_PLAYER_CAR_Y;
        transform.translation.z = BEHIND_PLAYER_CAR_Z;
    }
}

fn update_player_car(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
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
    let footprint_margin = car_footprint_margin();

    for (mut transform, mut controller) in &mut cars {
        controller.state.step(input, tuning.0, time.delta_secs());

        controller.state = track
            .0
            .recover_car_state_with_margin(controller.state, input, tuning.0, footprint_margin)
            .state;

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
    view_mode: Res<ViewModeState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cars: Query<(&mut Transform, &mut CarController), With<PlayerCar>>,
    mut cameras: Query<&mut Transform, (With<FollowCamera>, Without<PlayerCar>)>,
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

    if let Ok(mut camera) = cameras.single_mut() {
        camera.translation =
            camera_target_for_view(&start_state, camera.translation.z, view_mode.mode);
    }
}

fn update_follow_camera(
    time: Res<Time>,
    view_mode: Res<ViewModeState>,
    mut cameras: Query<&mut Transform, (With<FollowCamera>, Without<PlayerCar>)>,
    cars: Query<&CarController, With<PlayerCar>>,
) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };
    let Ok(controller) = cars.single() else {
        return;
    };

    let target = camera_target_for_view(&controller.state, camera.translation.z, view_mode.mode);
    let delta_seconds = time.delta_secs().clamp(0.0, CAMERA_MAX_DELTA_SECONDS);
    // Exponential smoothing is stable across frame rates; the clamp prevents
    // one delayed frame from snapping the camera after a stall or tab switch.
    let blend = 1.0 - (-CAMERA_FOLLOW_DECAY * delta_seconds).exp();

    camera.translation = camera.translation.lerp(target, blend);
}

fn camera_target_for_view(state: &CarState, z: f32, mode: ViewMode) -> Vec3 {
    match mode {
        ViewMode::Behind => Vec3::new(0.0, 0.0, z),
        ViewMode::Overhead => camera_target_for(state, z),
    }
}

fn camera_target_for(state: &CarState, z: f32) -> Vec3 {
    let forward = Vec2::new(state.heading_radians.sin(), state.heading_radians.cos());
    Vec3::new(
        state.position.x - forward.x * CAMERA_BEHIND_DISTANCE,
        state.position.y - forward.y * CAMERA_BEHIND_DISTANCE,
        z,
    )
}

fn car_footprint_margin() -> f32 {
    (CAR_WIDTH * 0.5).hypot(CAR_LENGTH * 0.5) + CAR_FOOTPRINT_PADDING
}

fn view_visibility(entity_mode: ViewMode, active_mode: ViewMode) -> Visibility {
    if entity_mode == active_mode {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

fn hud_text_for(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::Behind => "Behind view | V switch | 2 overhead | R reset | Esc start screen",
        ViewMode::Overhead => "Overhead view | V switch | 1 behind | R reset | Esc start screen",
    }
}

fn road_piece_z(index: usize, kind: BehindRoadPieceKind) -> f32 {
    let near_order = (BEHIND_ROAD_STRIPS - index).max(1) as f32;
    let layer = match kind {
        BehindRoadPieceKind::Road => 0.0,
        BehindRoadPieceKind::LeftCurb | BehindRoadPieceKind::RightCurb => 0.04,
        BehindRoadPieceKind::LaneMarker => 0.08,
    };

    BEHIND_ROAD_Z + near_order * 0.08 + layer
}

fn curb_color(index: usize) -> Color {
    if index % 2 == 0 {
        Color::srgb(0.9, 0.08, 0.08)
    } else {
        Color::srgb(0.94, 0.95, 0.9)
    }
}
