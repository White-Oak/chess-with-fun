use std::fmt::Display;

use bevy::prelude::{EventReader, IntoSystem, ResMut};
use bevy::prelude::{AppBuilder, Plugin};

use crate::pieces::{PieceColor, PieceType};

#[derive(Clone, Debug, Default)]
pub struct History {
    pub turns: Vec<Turn>,
}

#[derive(Clone, Copy, Debug)]
pub struct Turn {
    pub color: PieceColor,
    pub piece_type: PieceType,
    pub from_x: u8,
    pub from_y: u8,
    pub to_x: u8,
    pub to_y: u8,
}

impl Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self.color {
            PieceColor::White => "w",
            PieceColor::Black => "b",
        };
        let piece = match self.piece_type {
            PieceType::King => "K",
            PieceType::Queen => "Q",
            PieceType::Bishop => "B",
            PieceType::Knight => "k",
            PieceType::Rook => "R",
            PieceType::Pawn => "p",
        };
        write!(
            f,
            "{}{} {}:{} -> {}:{}",
            color, piece, self.from_x, self.from_y, self.to_x, self.to_y
        )
    }
}

// fn create_history(mut commands: Commands) {
//     commands.spawn().insert(History::default());
// }

fn add_turn_to_history(
    mut event_reader: EventReader<Turn>,
    mut history: ResMut<History>,
) {
    for turn in event_reader.iter() {
        history.turns.push(*turn);
    }
}


pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // app.add_startup_system(create_history.system());
        app.init_resource::<History>()
        .add_event::<Turn>()
        .add_system(add_turn_to_history.system());
    }
}
