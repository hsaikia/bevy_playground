use bevy::prelude::*;

pub fn sprite_bundle_2d(
    image: Handle<Image>,
    translation: Vec2,
    scale_xy: f32,
    heading_yaw: f32,
) -> impl Bundle {
    (
        Sprite::from_image(image),
        Transform::from_translation(translation.extend(0.))
            .with_rotation(Quat::from_rotation_z(heading_yaw))
            .with_scale(Vec3::splat(scale_xy)),
    )
}
