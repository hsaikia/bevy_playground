use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};

#[derive(Message)]
pub struct GenerateNewMessage;

#[derive(Message)]
pub struct SaveMessage;

pub struct UtilsPlugin;

impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_message::<GenerateNewMessage>()
            .add_message::<SaveMessage>()
            .add_systems(Update, handle_keyboard);
    }
}

fn handle_keyboard(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
    mut generate_new: MessageWriter<GenerateNewMessage>,
    mut save_new: MessageWriter<SaveMessage>,
    mut counter: Local<u32>,
) {
    if input.just_pressed(KeyCode::KeyP) {
        let path = format!("./screenshot-{}.png", *counter);
        *counter += 1;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    if input.just_pressed(KeyCode::KeyL) {
        save_new.write(SaveMessage);
    }
    if input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
    if input.just_pressed(KeyCode::Space) {
        generate_new.write(GenerateNewMessage);
    }
}
