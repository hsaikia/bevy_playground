use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_playground::plugins::utils::UtilsPlugin;

const BLINK_COLOR_INTENSITY: f32 = 5.0;
const BLINK_DURATION: f32 = 0.3;
const SHOOTING_INTERVAL: f32 = 2.0;
const SPEED_LASER: f32 = 300.0;
const LASER_BOUND: f32 = 1000.0;
const HIT_DAMAGE: f32 = 1.;
const SCALE: f32 = 1.;
const IMAGE_PATH_PLAYER: &str = "images/playerShip2_blue.png";
const IMAGE_PATH_ENEMY: &str = "images/ufoRed.png";
const IMAGE_PATH_LASER: &str = "images/laserBlue01.png";
const LEFT: Vec3 = Vec3::new(-300., 0.0, 0.);
const RIGHT: Vec3 = Vec3::new(300., 0.0, 0.);

#[derive(Component)]
struct CircularCollider(f32);

#[derive(Component)]
struct Health(f32);

impl CircularCollider {
    fn collides_with(
        &self,
        self_position: &Vec3,
        other: &CircularCollider,
        other_position: &Vec3,
    ) -> bool {
        self_position.distance(*other_position) < self.0 + other.0
    }
}

#[derive(Component)]
struct Blink(Timer);

impl Default for Blink {
    fn default() -> Self {
        Self(Timer::from_seconds(BLINK_DURATION, TimerMode::Once))
    }
}

#[derive(Resource)]
struct ShootingTimer(Timer);

impl Default for ShootingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SHOOTING_INTERVAL, TimerMode::Repeating))
    }
}

#[derive(Component)]
struct Speed(f32);

fn main() {
    App::new()
        .init_resource::<ShootingTimer>()
        .add_plugins((DefaultPlugins, UtilsPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_shooting,
                move_entities,
                handle_collisions,
                handle_blinking,
            ),
        )
        .run();
}

fn sprite(
    image: Handle<Image>,
    translation: Vec3,
    scale_xyz: f32,
    heading_yaw: f32,
) -> impl Bundle {
    (
        Sprite::from_image(image),
        Transform::from_translation(translation)
            .with_rotation(Quat::from_rotation_z(heading_yaw))
            .with_scale(Vec3::splat(scale_xyz)),
    )
}

fn player(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_PLAYER);
    sprite(image, LEFT, SCALE, 0.)
}

fn enemy(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_ENEMY);
    (
        sprite(image, RIGHT, SCALE, PI),
        CircularCollider(0.5),
        Health(1000.),
    )
}

fn laser(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_LASER);
    (
        sprite(image, LEFT, SCALE, 0.),
        Speed(SPEED_LASER),
        CircularCollider(0.5),
        Health(0.01),
    )
}

fn player_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<ShootingTimer>,
    time: Res<Time>,
) {
    if timer.0.finished() {
        commands.spawn(laser(&asset_server));
    }
    timer.0.tick(time.delta());
}

fn move_entities(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Speed)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, speed) in query.iter_mut() {
        let forward = transform.local_x();
        transform.translation += speed.0 * forward * dt;
        if transform.translation.distance(LEFT) > LASER_BOUND {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_collisions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CircularCollider, &mut Health, &Transform)>,
) {
    let mut damages = vec![0.; query.iter().len()];
    for (i, (_, collider_i, _, transform_i)) in query.iter().enumerate() {
        for (j, (_, collider_j, _, transform_j)) in query.iter().enumerate() {
            if i < j
                && collider_i.collides_with(
                    &transform_i.translation,
                    collider_j,
                    &transform_j.translation,
                )
            {
                damages[i] += HIT_DAMAGE;
                damages[j] += HIT_DAMAGE;
            }
        }
    }

    for ((entity, _, mut health_i, _), damage) in
        query.iter_mut().zip(damages.iter())
    {
        health_i.0 -= damage;

        if health_i.0 < 0. {
            commands.entity(entity).despawn();
        } else if *damage > 0. {
            commands.entity(entity).insert(Blink::default());
        }
    }
}

fn handle_blinking(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Blink, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut blink, mut sprite) in query.iter_mut() {
        if blink.0.finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<Blink>();
        } else {
            sprite.color = Color::Srgba(
                BLINK_COLOR_INTENSITY * Srgba::RED * (1. - blink.0.fraction())
                    + blink.0.fraction() * Srgba::WHITE,
            );
            blink.0.tick(time.delta());
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(player(&asset_server));
    commands.spawn(enemy(&asset_server));
}
