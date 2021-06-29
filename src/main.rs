use std::time::Duration;

use bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_mod_picking::*;

mod pieces;
use pieces::*;
mod board;
use board::*;
mod ui;
use ui::*;

use crate::{combust::CombustPlugin, history::HistoryPlugin};
mod combust;
mod history;

fn main() {
    App::build()
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 8 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Chess!".to_string(),
            width: 1000.,
            height: 1000.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .init_resource::<PickingCamera>()
        .add_plugin(PickingPlugin)
        .add_plugin(BoardPlugin)
        .add_plugin(PiecesPlugin)
        .add_plugin(HistoryPlugin)
        .add_plugin(CombustPlugin)
        .add_plugin(UIPlugin)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LogDiagnosticsPlugin {
            debug: false,
            wait_duration: Duration::from_secs(1),
            filter: Some(vec![FrameTimeDiagnosticsPlugin::FPS]),
        })
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands) {
    commands
        // Camera
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::from_rotation_translation(
                Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
                Vec3::new(-2.5, 12.0, 4.0),
            )),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default())
        // Light
        .commands()
        .spawn_bundle(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}
