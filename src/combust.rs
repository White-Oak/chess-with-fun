use bevy::math::Quat;
use bevy::prelude::{
    FromWorld, Handle, IntoSystem,
};
use bevy::{
    core::{Time, Timer},
    math::{Vec2, Vec3},
    prelude::{
        AppBuilder, AssetServer, Assets, BuildChildren, Color, Commands, Entity, EventReader,
        Plugin, Query, Res, SpriteBundle, Transform, With,
    },
    sprite::{ColorMaterial, Sprite},
};
use rand::{thread_rng, Rng};

struct Particle;
struct Lifetime(i32);
struct Velocity(Vec3);
struct Acceleration(Vec3);
struct Alive(bool);

struct Combust(Timer);

pub struct StartCombust(pub Entity);

fn create_combust(mut event_reader: EventReader<StartCombust>, mut commands: Commands) {
    for StartCombust(entity) in event_reader.iter() {
        commands
            .entity(*entity)
            .insert(Combust(Timer::from_seconds(0.001, true)));
        println!("COMBUST GOES BRRRRRRRR")
    }
}

const INITIAL_SIZE: f32 = 0.2;
const MAX_LIFETIME: i32 = 100;

fn spawn_particles(
    time: Res<Time>,
    combust_materials: Res<CombustMaterials>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Combust)>,
) {
    for (entity, mut combust) in query.iter_mut() {
        combust.0.tick(time.delta());
        if combust.0.just_finished() {
            let mut rng = thread_rng();
            let amount = rng.gen_range(2..5);
            for _ in 0..amount {
                let tile_size = Vec2::splat(INITIAL_SIZE);
                let variety_idx = rng.gen_range(0..COMBUST_VARIETY);
                let material = combust_materials.0[variety_idx].clone();
                let mut transform = Transform::from_translation(Vec3::new(
                    rng.gen_range(-0.25..0.25),
                    rng.gen_range(0.0..0.05),
                    rng.gen_range(-0.25..0.25),
                ));
                transform.rotation = Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize();
                transform.scale = Vec3::new(1., 1., 1.);
                commands.entity(entity).with_children(|parent| {
                    parent
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite::new(tile_size),
                            material,
                            transform,
                            ..Default::default()
                        })
                        .insert(Particle)
                        .insert(Acceleration(Vec3::new(0.0, 0.0, 0.0)))
                        .insert(Velocity(Vec3::new(
                            rng.gen_range(-0.025..0.025),
                            rng.gen_range(0.0..0.1),
                            rng.gen_range(-0.025..0.025),
                        )))
                        .insert(Alive(true))
                        .insert(Lifetime(MAX_LIFETIME));
                });
            }
        }
    }
}

fn kill_particles(mut commands: Commands, mut query: Query<(Entity, &mut Lifetime, &mut Sprite)>) {
    for (entity, mut lifetime, mut sprite) in query.iter_mut() {
        lifetime.0 -= 3;
        let ratio = (lifetime.0 as f32) / MAX_LIFETIME as f32;
        sprite.size = Vec2::splat(INITIAL_SIZE * ratio);
        if lifetime.0 <= 0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_pos(
    mut query: Query<(&mut Transform, &mut Velocity, &Acceleration, &Alive), With<Particle>>,
) {
    for (mut pos, mut vel, accel, is_alive) in query.iter_mut() {
        if is_alive.0 {
            vel.0 += accel.0;
            pos.translation += vel.0;
        }
    }
}

fn apply_force(mut query: Query<&mut Acceleration>) {
    for mut accel in query.iter_mut() {
        accel.0 += Vec3::new(0.0, 0.0002, 0.0);
    }
}

pub struct CombustPlugin;

const COMBUST_VARIETY: usize = 50;
const MAX_YELLOW: f32 = 0.7;
const STEP_YELLOW: f32 = MAX_YELLOW / COMBUST_VARIETY as f32;

struct CombustMaterials(Vec<Handle<ColorMaterial>>);
impl FromWorld for CombustMaterials {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let world = world.cell();
        let mut materials = world
            .get_resource_mut::<Assets<ColorMaterial>>()
            .unwrap();
            let mut vec = vec![];
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        let texture = asset_server.load("combust_particle.png");
        for i in 0..COMBUST_VARIETY {
            let mut material: ColorMaterial = texture.clone().into();
            let yellow = i as f32 * STEP_YELLOW;
            material.color = Color::rgb(0.99, yellow, 0.01);
            let material = materials.add(material);
            vec.push(material);
        }
        CombustMaterials(vec)
    }
}

impl Plugin for CombustPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<CombustMaterials>()
            .add_event::<StartCombust>()
            .add_system(create_combust.system())
            .add_system(spawn_particles.system())
            .add_system(kill_particles.system())
            .add_system(update_pos.system())
            .add_system(apply_force.system());
    }
}
