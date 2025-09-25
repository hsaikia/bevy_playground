use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_playground::{
    library::sprite::sprite_bundle_2d, plugins::utils::UtilsPlugin,
};

const BLINK_COLOR_INTENSITY: f32 = 5.0;
const BLINK_DURATION: f32 = 0.3;
const SPEED_LASER: f32 = 300.0;
const PLAYER_ACCELERATION: f32 = 1000.0;
const PLAYER_DRAG: f32 = 0.95;
const LASER_BOUND: f32 = 1000.0;
const COLLISION_RADIUS: f32 = 10.0;
const LASER_DAMAGE: f32 = 10.;
const ENEMY_SHIP_HEALTH: f32 = 100.;
const SCALE: f32 = 1.;
const IMAGE_PATH_PLAYER: &str = "images/playerShip2_blue.png";
const IMAGE_PATH_ENEMY: &str = "images/ufoRed.png";
const IMAGE_PATH_LASER: &str = "images/laserBlue01.png";
const LEFT: Vec2 = Vec2::new(-300., 0.0);
const RIGHT: Vec2 = Vec2::new(300., 0.0);

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
#[derive(Component)]
struct Speed(Vec2);

#[derive(Component)]
struct Acceleration(Vec2);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, UtilsPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_shooting,
                move_entities,
                handle_collisions,
                handle_blinking,
                handle_player_orientation,
                handle_acceleration,
                handle_player_movement,
                despawn_lasers,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Laser;

fn player(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_PLAYER);
    (
        Player,
        Acceleration(Vec2::ZERO),
        Speed(Vec2::ZERO),
        sprite_bundle_2d(image, LEFT, SCALE, 0.),
    )
}

fn enemy(asset_server: &Res<AssetServer>) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_ENEMY);
    (
        sprite_bundle_2d(image, RIGHT, SCALE, PI),
        CircularCollider(COLLISION_RADIUS),
        Health(ENEMY_SHIP_HEALTH),
    )
}

fn laser(
    asset_server: &Res<AssetServer>,
    transform: &Transform,
) -> impl Bundle {
    let image = asset_server.load(IMAGE_PATH_LASER);
    let direction = transform.right();
    let yaw = direction.y.atan2(direction.x);
    (
        Laser,
        sprite_bundle_2d(image, transform.translation.xy(), SCALE, yaw),
        Speed(SPEED_LASER * transform.local_x().xy()),
        CircularCollider(COLLISION_RADIUS),
        Health(LASER_DAMAGE),
    )
}

fn handle_acceleration(
    mut query: Query<(&mut Acceleration, &mut Speed)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut acceleration, mut speed) in query.iter_mut() {
        let old_speed = speed.0;
        speed.0 -= PLAYER_DRAG * old_speed * dt;
        speed.0 += acceleration.0 * dt;
        acceleration.0 = Vec2::ZERO;
    }
}

fn handle_player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<&mut Acceleration, With<Player>>,
) {
    if let Ok(mut acc) = q_player.single_mut() {
        let mut cumulative_acc = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            cumulative_acc.y = 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            cumulative_acc.y = -1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            cumulative_acc.x = -1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            cumulative_acc.x = 1.0;
        }
        if cumulative_acc.length() > 0. {
            acc.0 = cumulative_acc.normalize() * PLAYER_ACCELERATION;
        }
    }
}

fn handle_player_orientation(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut q_player_tr: Query<&mut Transform, With<Player>>,
) {
    if let Ok((camera, camera_transform)) = q_camera.single() {
        if let Some(mouse_pos) = q_window
            .single()
            .unwrap()
            .cursor_position()
            .and_then(|cursor| {
                camera.viewport_to_world_2d(camera_transform, cursor).ok()
            })
        {
            if let Ok(mut player_tr) = q_player_tr.single_mut() {
                let target_heading =
                    (mouse_pos - player_tr.translation.truncate()).normalize();
                player_tr.rotation = Quat::from_rotation_z(
                    target_heading.y.atan2(target_heading.x),
                );
            }
        }
    }
}

fn player_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_tr: Query<&Transform, With<Player>>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_released(MouseButton::Left) {
        commands.spawn(laser(&asset_server, player_tr.single().unwrap()));
    }
}

fn despawn_lasers(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<Laser>>,
    player: Query<&Transform, With<Player>>,
) {
    for (entity, transform) in query.iter_mut() {
        if transform
            .translation
            .distance(player.single().unwrap().translation)
            > LASER_BOUND
        {
            commands.entity(entity).despawn();
        }
    }
}

fn move_entities(mut query: Query<(&mut Transform, &Speed)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut transform, speed) in query.iter_mut() {
        transform.translation += speed.0.extend(0.) * dt;
    }
}

fn handle_collisions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CircularCollider, &mut Health, &Transform)>,
) {
    let mut damages = vec![0.; query.iter().len()];
    for (i, (_, collider_i, health_i, transform_i)) in query.iter().enumerate()
    {
        for (j, (_, collider_j, health_j, transform_j)) in
            query.iter().enumerate()
        {
            if i < j
                && collider_i.collides_with(
                    &transform_i.translation,
                    collider_j,
                    &transform_j.translation,
                )
            {
                let damage = f32::min(health_i.0, health_j.0);
                damages[i] += damage;
                damages[j] += damage;
            }
        }
    }

    for ((entity, _, mut health_i, _), damage) in
        query.iter_mut().zip(damages.iter())
    {
        if *damage > 0. {
            //println!("Health {} Damage {}", health_i.0, damage);
            health_i.0 -= damage;

            if health_i.0 <= 0. {
                commands.entity(entity).despawn();
            } else {
                commands.entity(entity).insert(Blink::default());
            }
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
