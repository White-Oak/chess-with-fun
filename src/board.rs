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
    selected_piece: Res<Option<SelectedPiece>>,
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

    let selected_square = selected_piece.as_ref().map(|x| x.square_entity);
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

struct SelectedPiece {
    square_entity: Entity,
    piece_entity: Entity,
    x: u8,
    y: u8,
}

#[derive(Debug, Clone, Copy)]
struct MovePieceEvent(u8, u8);

#[allow(clippy::too_many_arguments)]
fn select_square(
    mouse_button_inputs: Res<Input<MouseButton>>,
    movable_squares_query: Query<&Square, With<MovableSquare>>,
    squares_query: Query<&Square>,
    pieces_query: Query<(Entity, &Piece)>,
    picking_camera_query: Query<&PickingCamera>,
    mut selected_piece_res: ResMut<Option<SelectedPiece>>,
    turn: Res<PlayerTurn>,
    mut move_piece: EventWriter<MovePieceEvent>,
) {
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    let mut deselect = false;

    // Get the square under the cursor and set it as the selected
    let picking_camera = picking_camera_query.single().expect("where is the camera?");
    if let Some((square_entity, _intersection)) = picking_camera.intersect_top() {
        let square = squares_query
            .get(square_entity)
            .expect("where is the square");
        // Don't select piece if no friendly piece is selected.
        if let Some(piece_entity) = pieces_query
            .iter()
            .find(|(_, piece)| piece.x == square.x && piece.y == square.y && piece.color == turn.0)
            .map(|(entity, _)| entity)
        {
            let selected_piece = SelectedPiece {
                square_entity,
                piece_entity,
                x: square.x,
                y: square.y,
            };
            selected_piece_res.insert(selected_piece);
        } else {
            // Try to move piece otherwise
            if movable_squares_query
                .iter()
                .any(|move_square| square.x == move_square.x && square.y == move_square.y)
            {
                let event = MovePieceEvent(square.x, square.y);
                move_piece.send(event);
            } else {
                deselect = true;
            }
        }
    } else {
        deselect = true;
    }
    if deselect {
        // Clicked outside of board or clicked outide of movable positions
        selected_piece_res.take();
    }
}

fn highlight_moves(
    mut commands: Commands,
    selected_piece: Res<Option<SelectedPiece>>,
    squares_query: Query<(Entity, &Square), Without<MovableSquare>>,
    movable_squares_query: Query<Entity, With<MovableSquare>>,
    pieces_query: Query<&Piece>,
    pieces_to_take_query: Query<(Entity, &Piece)>,
    history: Res<History>,
) {
    if !selected_piece.is_changed() {
        return;
    }
    for entity in movable_squares_query.iter() {
        commands.entity(entity).remove::<MovableSquare>();
    }
    if let Some(selected_piece) = selected_piece.as_ref() {
        let pieces: Vec<Piece> = pieces_query.iter().copied().collect();
        let piece = pieces_query
            .get(selected_piece.piece_entity)
            .expect("where is the piece");
        let positions = piece.valid_positions(&pieces, &history);
        for (entity, square) in squares_query.iter() {
            for &(x, y, takeable) in positions.iter() {
                if square.x == x && square.y == y {
                    commands.entity(entity).insert(MovableSquare);
                    if let Some(takeable) = takeable {
                        for (entity, piece) in pieces_to_take_query.iter() {
                            if piece.x == takeable.0 && piece.y == takeable.1 {
                                let takeable = Takeable(x, y);
                                commands.entity(entity).insert(takeable);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn move_piece(
    mut commands: Commands,
    mut turn: ResMut<PlayerTurn>,
    mut pieces_query: Query<&mut Piece, Without<Takeable>>,
    target_pieces_query: Query<(Entity, &Takeable)>,
    mut reset_selected_event: EventWriter<ResetSelectedEvent>,
    mut turn_event_w: EventWriter<Turn>,
    mut move_piece_r: EventReader<MovePieceEvent>,
    selected_state: Res<Option<SelectedPiece>>,
) {
    let &MovePieceEvent(to_x, to_y) = if let Some(x) = move_piece_r.iter().next() {
        x
    } else {
        return;
    };
    let selected_state = selected_state
        .as_ref()
        .expect("move without selected piece");
    let mut selected_piece = pieces_query
        .get_mut(selected_state.piece_entity)
        .expect("invalid selected state");
    let target_piece = target_pieces_query
        .iter()
        .find(|(_, takeable)| takeable.0 == to_x && takeable.1 == to_y);
    if let Some((target_piece_entity, _)) = target_piece {
        selected_piece.energy = selected_piece.energy.saturating_add(KILL_ENERGY);
        // Mark the piece as taken
        commands.entity(target_piece_entity).insert(Taken);
    }
    // Move the selected piece to the selected square
    let event_turn = Turn {
        color: turn.0,
        piece_type: selected_piece.piece_type,
        from_x: selected_piece.x,
        from_y: selected_piece.y,
        to_x,
        to_y,
    };
    // Move piece
    selected_piece.x = to_x;
    selected_piece.y = to_y;

    // Change turn
    turn_event_w.send(event_turn);
    turn.change();
    reset_selected_event.send(ResetSelectedEvent);
}

struct ResetSelectedEvent;

fn reset_selected(
    mut commands: Commands,
    mut event_reader: EventReader<ResetSelectedEvent>,
    movable_query: Query<Entity, With<MovableSquare>>,
    takeable_query: Query<Entity, With<Takeable>>,
    mut selected_piece: ResMut<Option<SelectedPiece>>,
) {
    for _event in event_reader.iter() {
        for entity in movable_query.iter() {
            commands.entity(entity).remove::<MovableSquare>();
        }
        for entity in takeable_query.iter() {
            commands.entity(entity).remove::<Takeable>();
        }
        selected_piece.take();
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
            .init_resource::<Option<SelectedPiece>>()
            .add_event::<ResetSelectedEvent>()
            .add_event::<MovePieceEvent>()
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
            // .add_system(
            //     select_piece
            //         .system()
            //         .after("select_square")
            //         .label("select_piece"),
            // )
            .add_system(highlight_moves.system().after("select_piece"))
            .add_system(
                despawn_taken_pieces
                    .system()
                    .after("move_piece")
                    .before("select_piece"),
            )
            .add_system(
                reset_selected
                    .system()
                    .after("move_piece")
                    .before("select_piece"),
            );
    }
}
