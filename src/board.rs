use crate::{
    history::{History, Turn},
    pieces::*,
};
use bevy::{app::AppExit, prelude::*};
use bevy_mod_picking::*;

pub struct Square {
    pub x: u8,
    pub y: u8,
}
impl Square {
    fn is_white(&self) -> bool {
        (self.x + self.y + 1) % 2 == 0
    }
}

struct MovableSquare;

fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SquareMaterials>,
) {
    // Add meshes
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));

    // Spawn 64 squares
    for i in 0..8 {
        for j in 0..8 {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    // Change material according to position to get alternating pattern
                    material: if (i + j + 1) % 2 == 0 {
                        materials.white_color.clone()
                    } else {
                        materials.black_color.clone()
                    },
                    transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                    ..Default::default()
                })
                .insert_bundle(PickableBundle::default())
                .insert(Square { x: i, y: j });
        }
    }
}

fn color_squares(
    materials: Res<SquareMaterials>,
    selected_square: Query<Entity, With<Selected>>,
    mut query: Query<(Entity, &Square, &mut Handle<StandardMaterial>), Without<MovableSquare>>,
    mut movable_query: Query<(Entity, &Square, &mut Handle<StandardMaterial>), With<MovableSquare>>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Get entity under the cursor, if there is one
    let top_entity = match picking_camera_query.iter().last() {
        Some(picking_camera) => picking_camera
            .intersect_top()
            .map(|(entity, _intersection)| entity),
        None => None,
    };
    let selected_square = selected_square.single().ok();

    for (entity, square, mut material) in query.iter_mut() {
        // Change the material
        *material = if Some(entity) == top_entity {
            materials.highlight_color.clone()
        } else if Some(entity) == selected_square {
            materials.selected_color.clone()
        } else if square.is_white() {
            materials.white_color.clone()
        } else {
            materials.black_color.clone()
        };
    }
    for (entity, square, mut material) in movable_query.iter_mut() {
        *material = if Some(entity) == top_entity {
            materials.highlight_color.clone()
        } else if Some(entity) == selected_square {
            materials.selected_color.clone()
        } else if square.is_white() {
            materials.movable_white_color.clone()
        } else {
            materials.movable_black_color.clone()
        };
    }
}

struct SquareMaterials {
    highlight_color: Handle<StandardMaterial>,
    selected_color: Handle<StandardMaterial>,
    black_color: Handle<StandardMaterial>,
    white_color: Handle<StandardMaterial>,
    movable_white_color: Handle<StandardMaterial>,
    movable_black_color: Handle<StandardMaterial>,
}

impl FromWorld for SquareMaterials {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        SquareMaterials {
            highlight_color: materials.add(Color::rgb(0.8, 0.3, 0.3).into()),
            selected_color: materials.add(Color::rgb(0.9, 0.1, 0.1).into()),
            black_color: materials.add(Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials.add(Color::rgb(1., 0.9, 0.9).into()),
            movable_white_color: materials.add(Color::rgb(0.7, 0.9, 0.9).into()),
            movable_black_color: materials.add(Color::rgb(0., 0.3, 0.3).into()),
        }
    }
}

struct Selected;

pub struct PlayerTurn(pub PieceColor);
impl Default for PlayerTurn {
    fn default() -> Self {
        Self(PieceColor::White)
    }
}
impl PlayerTurn {
    fn change(&mut self) {
        self.0 = match self.0 {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

fn select_square(
    mut commands: Commands,
    mouse_button_inputs: Res<Input<MouseButton>>,
    selected_query: Query<Entity, With<Selected>>,
    picking_camera_query: Query<&PickingCamera>,
) {
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    // Get the square under the cursor and set it as the selected
    if let Some(picking_camera) = picking_camera_query.iter().last() {
        if let Some((square_entity, _intersection)) = picking_camera.intersect_top() {
            println!("selected square");
            commands.entity(square_entity).insert(Selected);
        } else {
            for entity in selected_query.iter() {
                println!("deselected entity");
                commands.entity(entity).remove::<Selected>();
            }
        }
    }
}

fn select_piece(
    mut commands: Commands,
    selected_square: Query<&Square, Added<Selected>>,
    turn: Res<PlayerTurn>,
    pieces_query: Query<(Entity, &Piece), Without<Selected>>,
    selected_pieces_query: Query<Entity, (With<Selected>, With<Piece>)>,
) {
    let square = if let Ok(x) = selected_square.single() {
        x
    } else {
        return;
    };

    // Select the piece in the currently selected square
    for (piece_entity, piece) in pieces_query.iter() {
        if piece.x == square.x && piece.y == square.y && piece.color == turn.0 {
            // piece_entity is now the entity in the same square
            for entity in selected_pieces_query.iter() {
                println!("deselected piece");
                commands.entity(entity).remove::<Selected>();
            }
            println!("selected piece");
            commands.entity(piece_entity).insert(Selected);
            break;
        }
    }
}

fn highlight_moves(
    mut commands: Commands,
    selected_piece: Query<&Piece, Added<Selected>>,
    squares_query: Query<(Entity, &Square), Without<MovableSquare>>,
    movable_squares_query: Query<Entity, With<MovableSquare>>,
    pieces_query: Query<&Piece>,
    history: Res<History>,
) {
    let piece = if let Ok(piece) = selected_piece.single() {
        piece
    } else {
        return;
    };
    for entity in movable_squares_query.iter() {
        commands.entity(entity).remove::<MovableSquare>();
    }
    let pieces: Vec<Piece> = pieces_query.iter().copied().collect();
    let positions = piece.valid_positions(&pieces, &history);
    for (entity, square) in squares_query.iter() {
        for &(x, y) in positions.iter() {
            if square.x == x && square.y == y {
                commands.entity(entity).insert(MovableSquare);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn move_piece(
    mut commands: Commands,
    selected_square: Query<&Square, Added<Selected>>,
    mut turn: ResMut<PlayerTurn>,
    mut selected_piece: Query<&mut Piece, With<Selected>>,
    target_pieces_query: Query<(Entity, &Piece), Without<Selected>>,
    movable_query: Query<(Entity, &MovableSquare, &Square)>,
    mut reset_selected_event: EventWriter<ResetSelectedEvent>,
    mut turn_event_w: EventWriter<Turn>,
) {
    let selected_square = if let Ok(square) = selected_square.single() {
        square
    } else {
        return;
    };
    let mut selected_piece = if let Ok(piece) = selected_piece.single_mut() {
        piece
    } else {
        return;
    };
    let target_square = movable_query
        .iter()
        .find(|(_, _, square)| square.x == selected_square.x && square.y == selected_square.y);
    if let Some((_, _, target_square)) = target_square {
        let target_piece = target_pieces_query.iter().find(|(_, target_piece)| {
            target_piece.x == target_square.x
                && target_piece.y == target_square.y
                && target_piece.color == selected_piece.color.opposite()
        });
        if let Some((target_piece_entity, _target_piece)) = target_piece {
            // selected_piece.energy = selected_piece.energy.saturating_add(KILL_ENERGY);
            // Mark the piece as taken
            commands.entity(target_piece_entity).insert(Taken);
            // En passant
            // if selected_piece.piece_type == PieceType::Pawn
            //     && target_piece.piece_type == PieceType::Pawn
            //     && selected_square.y == other_piece.y
            //     && (selected_square.x as i8 - other_piece.x as i8).abs() == 1
            //     && other_piece.color != selected_piece.color
            // {
            //     selected_piece.energy = selected_piece.energy.saturating_add(KILL_ENERGY);
            //     // Mark the piece as taken
            //     commands.entity(other_entity).insert(Taken);
            // }
        }
        // Move the selected piece to the selected square
        let event_turn = Turn {
            color: turn.0,
            piece_type: selected_piece.piece_type,
            from_x: selected_piece.x,
            from_y: selected_piece.y,
            to_x: selected_square.x,
            to_y: selected_square.y,
        };
        // Move piece
        selected_piece.x = selected_square.x;
        selected_piece.y = selected_square.y;

        // Change turn
        turn_event_w.send(event_turn);
        turn.change();
    }
    reset_selected_event.send(ResetSelectedEvent);
}

struct ResetSelectedEvent;

fn reset_selected(
    mut commands: Commands,
    mut event_reader: EventReader<ResetSelectedEvent>,
    selected: Query<Entity, With<Selected>>,
    movable_query: Query<Entity, With<MovableSquare>>,
) {
    for _event in event_reader.iter() {
        for entity in movable_query.iter() {
            commands.entity(entity).remove::<MovableSquare>();
        }
        for entity in selected.iter() {
            commands.entity(entity).remove::<Selected>();
        }
    }
}

struct Taken;
fn despawn_taken_pieces(
    mut commands: Commands,
    mut app_exit_events: EventWriter<AppExit>,
    query: Query<(Entity, &Piece, &Taken)>,
) {
    for (entity, piece, _taken) in query.iter() {
        // If the king is taken, we should exit
        if piece.piece_type == PieceType::King {
            println!(
                "{} won! Thanks for playing!",
                match piece.color {
                    PieceColor::White => "Black",
                    PieceColor::Black => "White",
                }
            );
            app_exit_events.send(AppExit);
        }

        // Despawn piece and children
        commands.entity(entity).despawn_recursive();
    }
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<SquareMaterials>()
            .init_resource::<PlayerTurn>()
            .add_event::<ResetSelectedEvent>()
            .add_startup_system(create_board.system())
            .add_system(color_squares.system())
            .add_system(select_square.system().label("select_square"))
            .add_system(
                // move_piece needs to run before select_piece
                move_piece
                    .system()
                    .after("select_square")
                    .before("select_piece")
                    .label("move_piece"),
            )
            .add_system(
                select_piece
                    .system()
                    .after("select_square")
                    .label("select_piece"),
            )
            .add_system(highlight_moves.system().after("select_piece"))
            .add_system(
                despawn_taken_pieces
                    .system()
                    .after("move_piece")
                    .before("select_piece"),
            )
            .add_system(reset_selected.system().after("move_piece").before("select_piece"));
    }
}
