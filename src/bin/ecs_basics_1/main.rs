use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_playground::plugins::utils::UtilsPlugin;

#[derive(Component)]
struct Light;

#[derive(Component)]
struct Speed(pub f32);

// -- Tree --
const SCALE_TREE: f32 = 0.3;
const IMAGE_PATH_TREE: &str = "images/tree.png";

// -- Stroller --
const SCALE_STROLLER: f32 = 0.3;
const IMAGE_PATH_STROLLER: &str = "images/stroller.png";
const SPEED_STROLLER: f32 = -40.0;

// -- School Bus --
const SCALE_SCHOOL_BUS: f32 = 0.7;
const IMAGE_PATH_SCHOOL_BUS: &str = "images/school_bus.png";
const SCALE_HEADLIGHTS: f32 = 0.07;
const IMAGE_PATH_HEADLIGHTS: &str = "images/headlights.png";
const OFFSET_HEADLIGHTS: Vec3 = Vec3::new(240.0, -10.0, -1.0);
const SPEED_SCHOOL_BUS: f32 = 100.0;

// -- Street Lamp --
const SCALE_STREET_LAMP: f32 = 0.3;
const IMAGE_PATH_STREET_LAMP: &str = "images/street_light.png";
const IMAGE_PATH_LAMP_LIGHTS: &str = "images/street_light_rays.png";
const OFFSET_LAMP_LIGHTS: Vec3 = Vec3::new(0.0, 130.0, -1.0);
const SCALE_LAMP_LIGHTS: f32 = 1.0;

// -- Other --
const BOUNDARY_MIN: f32 = 100.0;
const BOUNDARY_MAX: f32 = 500.0;
const TOP_LEFT: Vec3 = Vec3::new(-300., 200.0, 0.);
const TOP_RIGHT: Vec3 = Vec3::new(300., 200.0, 0.);
const BOTTOM_LEFT: Vec3 = Vec3::new(-300., -200.0, 0.);
const BOTTOM_RIGHT: Vec3 = Vec3::new(300., -200.0, 0.);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UtilsPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle_lights, move_entities))
        .run();
}

fn sprite(
    image: Handle<Image>,
    translation: Vec3,
    scale_xyz: f32,
) -> impl Bundle {
    (
        Sprite::from_image(image),
        Transform::from_translation(translation)
            .with_scale(Vec3::splat(scale_xyz)),
    )
}

fn stroller(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_STROLLER);
    (
        sprite(image, BOTTOM_RIGHT, SCALE_STROLLER),
        Speed(SPEED_STROLLER),
    )
}

fn tree(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_TREE);
    sprite(image, BOTTOM_LEFT, SCALE_TREE)
}

fn school_bus(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image1 = asset_server.load(IMAGE_PATH_SCHOOL_BUS);
    let image2 = asset_server.load(IMAGE_PATH_HEADLIGHTS);
    (
        sprite(image1, TOP_RIGHT, SCALE_SCHOOL_BUS),
        Speed(SPEED_SCHOOL_BUS),
        children![(
            Light,
            Visibility::Visible,
            sprite(image2, OFFSET_HEADLIGHTS, SCALE_HEADLIGHTS),
        )],
    )
}

fn street_light(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image1 = asset_server.load(IMAGE_PATH_STREET_LAMP);
    let image2 = asset_server.load(IMAGE_PATH_LAMP_LIGHTS);
    (
        sprite(image1, TOP_LEFT, SCALE_STREET_LAMP),
        children![(
            Light,
            Visibility::Visible,
            sprite(image2, OFFSET_LAMP_LIGHTS, SCALE_LAMP_LIGHTS)
        )],
    )
}

fn toggle_lights(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Visibility, With<Light>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for mut visibility in query.iter_mut() {
            visibility.toggle_visible_hidden();
        }
    }
}

fn move_entities(mut query: Query<(&mut Transform, &Speed)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut transform, speed) in query.iter_mut() {
        let forward = transform.local_x();
        transform.translation += speed.0 * forward * dt;
        if transform.translation.x > BOUNDARY_MAX
            || transform.translation.x < BOUNDARY_MIN
        {
            transform.rotate_y(PI);
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(street_light(&asset_server));
    commands.spawn(school_bus(&asset_server));
    commands.spawn(tree(&asset_server));
    commands.spawn(stroller(&asset_server));
}
