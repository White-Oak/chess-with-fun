use std::iter::repeat;

use bevy::prelude::*;

use crate::history::History;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub fn opposite(&self) -> Self {
        match self {
            PieceColor::Black => PieceColor::White,
            PieceColor::White => PieceColor::Black,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

pub const KILL_ENERGY: u8 = 10;

// impl PieceType {
//     pub fn max_energy(&self) -> u8 {
//         match self {
//             PieceType::King => 100,
//             PieceType::Queen => 100,
//             PieceType::Bishop => 100,
//             PieceType::Knight => 100,
//             PieceType::Rook => 100,
//             PieceType::Pawn => 10,
//         }
//     }
// }

#[derive(Clone, Copy)]
pub struct Piece {
    pub color: PieceColor,
    pub piece_type: PieceType,
    // Current position
    pub x: u8,
    pub y: u8,
    pub energy: u8,
}

const FIELD_SIZE: u8 = 8;

fn check_add(mut a: u8, da: i8) -> Option<u8> {
    if da < 0 {
        let da = da.abs() as u8;
        a = a.checked_sub(da)?;
    } else {
        let res = a.checked_add(da as u8)?;
        if res >= FIELD_SIZE {
            return None;
        }
        a = res;
    }
    Some(a)
}

fn pos_mov() -> impl Iterator<Item = i8> {
    1..=(FIELD_SIZE as i8)
}

fn neg_mov() -> impl Iterator<Item = i8> {
    (1..=FIELD_SIZE).map(|x| -(x as i8))
}

fn valid_positions_for_rook(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece]) {
    try_move_in_line(poss, this, pieces, pos_mov(), repeat(0));
    try_move_in_line(poss, this, pieces, neg_mov(), repeat(0));
    try_move_in_line(poss, this, pieces, repeat(0), pos_mov());
    try_move_in_line(poss, this, pieces, repeat(0), neg_mov());
}

fn valid_positions_for_bishop(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece]) {
    try_move_in_line(poss, this, pieces, pos_mov(), pos_mov());
    try_move_in_line(poss, this, pieces, pos_mov(), neg_mov());
    try_move_in_line(poss, this, pieces, neg_mov(), neg_mov());
    try_move_in_line(poss, this, pieces, neg_mov(), pos_mov());
}

fn valid_positions_for_queen(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece]) {
    valid_positions_for_rook(poss, this, pieces);
    valid_positions_for_bishop(poss, this, pieces);
}

fn valid_positions_for_knight(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece]) {
    try_move(poss, this, pieces, 1, 2);
    try_move(poss, this, pieces, 2, 1);
    try_move(poss, this, pieces, 2, -1);
    try_move(poss, this, pieces, 1, -2);
    try_move(poss, this, pieces, -1, -2);
    try_move(poss, this, pieces, -2, -1);
    try_move(poss, this, pieces, -2, 1);
    try_move(poss, this, pieces, -1, 2);
}

fn valid_positions_for_pawn(
    poss: &mut Vec<(u8, u8)>,
    this: &Piece,
    pieces: &[Piece],
    history: &History,
) {
    let last_turn = history.turns.last();
    let multiplier = if this.color == PieceColor::White {
        1
    } else {
        -1
    };
    try_peace_move_pawn(poss, this, pieces, 1 * multiplier);
    try_aggr_move_pawn(poss, this, pieces, 1 * multiplier);
    if this.color == PieceColor::White {
        if this.x == 1 {
            try_peace_move_pawn(poss, this, pieces, 2);
        }
    } else {
        if this.x == 6 {
            try_peace_move_pawn(poss, this, pieces, -2);
        }
    }
    if let Some(last_turn) = last_turn {
        if last_turn.color.opposite() != this.color {
            return;
        }
        if last_turn.piece_type != PieceType::Pawn {
            return;
        }
        if (last_turn.to_x as i8 - last_turn.from_x as i8).abs() != 2 {
            return;
        }
        if last_turn.to_x != this.x {
            return;
        }
        if (last_turn.to_y as i8 - this.y as i8).abs() != 1 {
            return;
        }
        if last_turn.to_y > this.y {
            try_move(poss, this, pieces, 1 * multiplier, 1);
        }
        if last_turn.to_y < this.y {
            try_move(poss, this, pieces, 1 * multiplier, -1);
        }
    }
}

fn try_peace_move_pawn(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece], dx: i8) {
    let x = if let Some(x) = check_add(this.x, dx as i8) {
        x
    } else {
        return;
    };
    if pieces.iter().any(|piece| piece.x == x && piece.y == this.y) {
        return;
    }
    poss.push((x, this.y));
}

fn try_aggr_move_pawn(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece], dx: i8) {
    try_aggr_move_pawn_part(poss, this, pieces, dx, -1);
    try_aggr_move_pawn_part(poss, this, pieces, dx, 1);
}

fn try_aggr_move_pawn_part(
    poss: &mut Vec<(u8, u8)>,
    this: &Piece,
    pieces: &[Piece],
    dx: i8,
    dy: i8,
) {
    let x = if let Some(x) = check_add(this.x, dx as i8) {
        x
    } else {
        return;
    };
    let y = if let Some(y) = check_add(this.y, dy) {
        y
    } else {
        return;
    };
    if !pieces
        .iter()
        .any(|piece| this.color.opposite() == piece.color && piece.x == x && piece.y == y)
    {
        return;
    }
    poss.push((x, y));
}

fn valid_positions_for_king(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece]) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            if let Some(x) = check_add(this.x, dx) {
                if let Some(y) = check_add(this.y, dy) {
                    if !pieces
                        .iter()
                        .any(|piece| piece.color == this.color && piece.x == x && piece.y == y)
                    {
                        // TODO: check for checks
                        poss.push((x, y));
                    }
                }
            }
        }
    }
}

fn try_move_in_line(
    poss: &mut Vec<(u8, u8)>,
    this: &Piece,
    pieces: &[Piece],
    iter_dx: impl Iterator<Item = i8>,
    iter_dy: impl Iterator<Item = i8>,
) {
    for (dx, dy) in iter_dx.zip(iter_dy) {
        try_move(poss, this, pieces, dx, dy);
    }
}

fn try_move(poss: &mut Vec<(u8, u8)>, this: &Piece, pieces: &[Piece], dx: i8, dy: i8) {
    let x = if let Some(x) = check_add(this.x, dx as i8) {
        x
    } else {
        return;
    };
    let y = if let Some(y) = check_add(this.y, dy as i8) {
        y
    } else {
        return;
    };
    if pieces
        .iter()
        .any(|piece| piece.color == this.color && piece.x == x && piece.y == y)
    {
        return;
    }
    poss.push((x, y));
}

impl Piece {
    // TODO: maybe SmallVec
    /// History is only used for en passant
    pub fn valid_positions(&self, pieces: &[Piece], history: &History) -> Vec<(u8, u8)> {
        let mut poss = Vec::new();
        match self.piece_type {
            PieceType::King => {
                valid_positions_for_king(&mut poss, self, pieces);
            }
            PieceType::Rook => {
                valid_positions_for_rook(&mut poss, self, pieces);
            }
            PieceType::Bishop => {
                valid_positions_for_bishop(&mut poss, self, pieces);
            }
            PieceType::Queen => {
                valid_positions_for_queen(&mut poss, self, pieces);
            }
            PieceType::Knight => {
                valid_positions_for_knight(&mut poss, self, pieces);
            }
            PieceType::Pawn => valid_positions_for_pawn(&mut poss, self, pieces, history),
        }
        poss
    }

    /// Returns the possible_positions that are available
    pub fn is_move_valid(
        &self,
        new_position: (u8, u8),
        pieces: &[Piece],
        history: &History,
    ) -> bool {
        // If there's a piece of the same color in the same square, it can't move
        if color_of_square(new_position, pieces) == Some(self.color) {
            return false;
        }
        let positions = self.valid_positions(pieces, history);
        positions.contains(&new_position)
    }
}

/// Returns None if square is empty, returns a Some with the color if not
fn color_of_square(pos: (u8, u8), pieces: &[Piece]) -> Option<PieceColor> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(piece.color);
        }
    }
    None
}

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        // Get the direction to move in
        let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;

        // Only move if the piece isn't already there (distance is big)
        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * (time.delta_seconds() * 8.);
        }
    }
}

fn create_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load all the meshes
    let king_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh0/Primitive0");
    let king_cross_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh1/Primitive0");
    let pawn_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh2/Primitive0");
    let knight_1_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh3/Primitive0");
    let knight_2_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh4/Primitive0");
    let rook_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh5/Primitive0");
    let bishop_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh6/Primitive0");
    let queen_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh7/Primitive0");

    // Add some materials
    let white_material = materials.add(Color::rgb(1., 0.8, 0.8).into());
    let black_material = materials.add(Color::rgb(0.3, 0.3, 0.3).into());

    spawn_rook(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        rook_handle.clone(),
        (0, 0),
    );
    spawn_knight(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        (0, 1),
    );
    spawn_bishop(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        bishop_handle.clone(),
        (0, 2),
    );
    spawn_queen(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        queen_handle.clone(),
        (0, 3),
    );
    spawn_king(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        king_handle.clone(),
        king_cross_handle.clone(),
        (0, 4),
    );
    spawn_bishop(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        bishop_handle.clone(),
        (0, 5),
    );
    spawn_knight(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        (0, 6),
    );
    spawn_rook(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        rook_handle.clone(),
        (0, 7),
    );

    for i in 0..8 {
        spawn_pawn(
            &mut commands,
            white_material.clone(),
            PieceColor::White,
            pawn_handle.clone(),
            (1, i),
        );
    }

    spawn_rook(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        rook_handle.clone(),
        (7, 0),
    );
    spawn_knight(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        (7, 1),
    );
    spawn_bishop(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        bishop_handle.clone(),
        (7, 2),
    );
    spawn_queen(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        queen_handle.clone(),
        (7, 3),
    );
    spawn_king(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        king_handle.clone(),
        king_cross_handle.clone(),
        (7, 4),
    );
    spawn_bishop(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        bishop_handle.clone(),
        (7, 5),
    );
    spawn_knight(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        (7, 6),
    );
    spawn_rook(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        rook_handle.clone(),
        (7, 7),
    );

    for i in 0..8 {
        spawn_pawn(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            pawn_handle.clone(),
            (6, i),
        );
    }
}

fn spawn_king(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    mesh_cross: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::King,
            x: position.0,
            y: position.1,
            energy: 0,
        })
        // Add children to the parent
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh,
                material: material.clone(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
            parent.spawn_bundle(PbrBundle {
                mesh: mesh_cross,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

fn spawn_knight(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh_1: Handle<Mesh>,
    mesh_2: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Knight,
            x: position.0,
            y: position.1,
            energy: 0,
        })
        // Add children to the parent
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: mesh_1,
                material: material.clone(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 0.9));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
            parent.spawn_bundle(PbrBundle {
                mesh: mesh_2,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 0.9));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

fn spawn_queen(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Queen,
            x: position.0,
            y: position.1,
            energy: 0,
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -0.95));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

fn spawn_bishop(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Bishop,
            x: position.0,
            y: position.1,
            energy: 0,
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.1, 0., 0.));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

fn spawn_rook(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Rook,
            x: position.0,
            y: position.1,
            energy: 0,
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.1, 0., 1.8));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

fn spawn_pawn(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..Default::default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Pawn,
            x: position.0,
            y: position.1,
            energy: 0,
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 2.6));
                    transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                    transform
                },
                ..Default::default()
            });
        });
}

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(create_pieces.system())
            .add_system(move_pieces.system());
    }
}
