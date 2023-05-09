use crate::{bundles::*, components::*, constants::*, lerp, resources::*, AppState};
use bevy::prelude::*;
use rand::random;

pub struct UfoPlugin;
impl Plugin for UfoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NextUfoScore::new())
            .add_systems(
                (
                    ufo_spawn_system,
                    ufo_movement_system,
                    ufo_animation_system,
                    ufo_shoot_system,
                    ship_projectile_ufo_hit_system,
                    ship_ufo_collision_system,
                    ship_ufo_laser_collision_system,
                )
                    .in_set(OnUpdate(AppState::InGame)),
            )
            .add_system(reset_next_ufo_score.in_schedule(OnEnter(AppState::NewGame)));
    }
}
#[derive(Component)]
struct Ufo {
    pub start_position: Vec2,
    pub end_position: Vec2,
    pub frequency: f32,
    pub amplitude: f32,
    pub duration: f32,
    pub time: f32,
    pub shoot_delay: f32,
    pub shoot_accuracy: f32,
    pub life: i32,
}
#[derive(Component)]
struct UfoLaser;

#[derive(Default, Resource)]
struct NextUfoScore(pub u32);

impl NextUfoScore {
    pub fn new() -> Self {
        Self(random_ufo_interval())
    }
    pub fn bump(&mut self, score: u32) -> bool {
        if score >= self.0 {
            self.0 = score + random_ufo_interval();
            true
        } else {
            false
        }
    }
}
fn random_ufo_interval() -> u32 {
    lerp(
        rand::random::<f32>(),
        MIN_UFO_SCORE_INTERVAL,
        MAX_UFO_SCORE_INTERVAL,
    ) as u32
}

fn reset_next_ufo_score(mut next_ufo_score: ResMut<NextUfoScore>) {
    *next_ufo_score = NextUfoScore::new();
}
fn ufo_spawn_system(
    mut commands: Commands,
    mut next_ufo_score: ResMut<NextUfoScore>,
    level: Res<Level>,
    score: Res<Score>,
    sprite_sheets: Res<SpriteSheets>,
) {
    if next_ufo_score.bump(score.value()) {
        let horizontal: bool = random();
        let direction: bool = random();
        let span = Vec2::new(GAME_WIDTH as f32 / 2.0, GAME_HEIGHT as f32 / 2.0);
        let d = random::<f32>() * span * 2.0;
        let position = match (horizontal, direction) {
            (false, false) => Vec2::new(d.x, span.y),
            (true, false) => Vec2::new(span.x, d.y),
            (false, true) => Vec2::new(d.x, 0.),
            (true, true) => Vec2::new(0., d.y),
        };

        let ufo = Ufo {
            start_position: position,
            end_position: -position,
            frequency: random::<f32>() * 5.0,
            amplitude: random::<f32>() * 90.0 + 10.0,
            duration: level.ufo_duration(),
            time: 0.0,
            shoot_delay: level.ufo_shoot_delay(),
            shoot_accuracy: level.ufo_shoot_accuracy(),
            life: 20,
        };
        commands.spawn(UfoBundle::new(&sprite_sheets.ufo, ufo));
    }
}

fn ufo_movement_system(
    mut commands: Commands,
    mut ufos_query: Query<(Entity, &mut Ufo, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut ufo, mut transform) in ufos_query.iter_mut() {
        ufo.time += time.delta_seconds();
        let t = ufo.time / ufo.duration;
        let journey = ufo.end_position - ufo.start_position;
        let deviation = ufo.amplitude * f32::sin(ufo.frequency * std::f32::consts::TAU * t);
        let position = ufo.start_position + journey * t + journey.normalize().perp() * deviation;
        let angle = 10.0 * std::f32::consts::TAU * t;
        let rotation = Quat::from_rotation_z(angle);
        *transform = Transform::from_rotation(rotation).with_translation(position.extend(0.));

        if ufo.time >= ufo.duration {
            commands.entity(entity).despawn();
        }
    }
}
fn ufo_animation_system(
    mut ufos_query: Query<(&Ufo, &mut Handle<Image>)>,
    sprite_sheets: Res<SpriteSheets>,
) {
    let frame_duration = 1. / 5.;
    for (ufo, mut image) in ufos_query.iter_mut() {
        let frame = (ufo.time / frame_duration) as usize % sprite_sheets.ufo.ship.len();
        *image = sprite_sheets.ufo.ship[frame].clone();
    }
}
fn ufo_shoot_system(
    mut commands: Commands,
    mut ufos_query: Query<(&mut Ufo, &Transform)>,
    ships_query: Query<&Transform, With<Ship>>,
    sprite_sheets: Res<SpriteSheets>,
    time: Res<Time>,
) {
    let ship_transform = ships_query.single();
    for (mut ufo, ufo_transform) in ufos_query.iter_mut() {
        ufo.shoot_delay -= time.delta_seconds();
        if ufo.shoot_delay <= 0.0 {
            ufo.shoot_delay = 2.0; // FIXME
            let target = (ship_transform.translation - ufo_transform.translation)
                .truncate()
                .normalize();
            let aim_error =
                (1.0 - ufo.shoot_accuracy) * (random::<f32>() - 0.5) * std::f32::consts::PI;
            let aim = Vec2::from_angle(aim_error).rotate(target);
            let speed = 500.0; // FIXME
            let velocity = aim * speed;
            let angle = Vec2::Y.angle_between(aim);
            let life = 2.0;
            commands.spawn(UfoLaserBundle::new(
                &sprite_sheets.ufo,
                ufo_transform.translation.truncate(),
                angle,
                velocity,
                life,
            ));
        }
    }
}
fn ship_ufo_collision_system(
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
    mut ships_query: Query<(&mut Ship, &Transform, &mut Moving, &CollisionShape)>,
    ufo_query: Query<(&Transform, &Moving, &CollisionShape), (With<Ufo>, Without<Ship>)>,
) {
    for (mut ship, ship_transform, mut ship_moving, ship_shape) in ships_query.iter_mut() {
        if ship.invulnerability > 0.0 {
            continue;
        }
        let ship_position = ship_transform.translation.truncate();
        for (ufo_transform, ufo_moving, ufo_shape) in ufo_query.iter() {
            let ufo_position = ufo_transform.translation.truncate();
            if ship_shape.intersects(ufo_shape) {
                if ship.shield_level > 0 {
                    ship.shield_level -= 1;
                    let diff = ship_position - ufo_position;
                    let speed =
                        (ufo_moving.velocity.project_onto(diff) - ship_moving.velocity).length();
                    ship_moving.velocity = diff.normalize() * speed;
                } else {
                    ship.respawn_delay = SHIP_RESPAWN_DELAY;
                    ship.lives = ship.lives.max(1) - 1;
                    commands.spawn(ExplosionBundle::new(
                        &sprite_sheets.explosion,
                        ship_position,
                    ));
                    commands.spawn(WaveParticleBundle::new(
                        ship_position,
                        &sprite_sheets.particles,
                    ));
                }
            }
        }
    }
}

fn ship_ufo_laser_collision_system(
    mut commands: Commands,
    mut ships_query: Query<(&mut Ship, &Transform, &mut Moving)>,
    ufo_laser_query: Query<(&Transform, &Moving), (With<UfoLaser>, Without<Ship>)>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for (mut ship, ship_transform, mut ship_moving) in ships_query.iter_mut() {
        if ship.invulnerability > 0.0 {
            continue;
        }
        let ship_position = ship_transform.translation.truncate();
        for (laser_transform, laser_moving) in ufo_laser_query.iter() {
            let laser_position = laser_transform.translation.truncate();
            let laser_radius: f32 = 1.0;
            let ship_radius: f32 = 16.0;
            let distance_sq = ship_position.distance_squared(laser_position);
            if distance_sq <= (laser_radius + ship_radius).powf(2.0) {
                if ship.shield_level > 0 {
                    ship.shield_level -= 1;
                    let diff = ship_position - laser_position;
                    let speed =
                        (laser_moving.velocity.project_onto(diff) - ship_moving.velocity).length();
                    ship_moving.velocity = diff.normalize() * speed;
                } else {
                    ship.respawn_delay = SHIP_RESPAWN_DELAY;
                    ship.lives = ship.lives.max(1) - 1;
                    commands.spawn(ExplosionBundle::new(
                        &sprite_sheets.explosion,
                        ship_position,
                    ));
                    commands.spawn(WaveParticleBundle::new(
                        ship_position,
                        &sprite_sheets.particles,
                    ));
                }
            }
        }
    }
}
fn ship_projectile_ufo_hit_system(
    mut commands: Commands,
    mut projectiles: Query<(
        Entity,
        &mut ShipProjectile,
        &mut Transform,
        &mut CollisionShape,
        Option<&mut Beam>,
    )>,
    mut ufos: Query<(Entity, &mut Ufo, &Transform, &CollisionShape), Without<ShipProjectile>>,
    sprite_sheets: Res<SpriteSheets>,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
) {
    for (
        projectile_entity,
        projectile,
        mut projectile_transform,
        mut projectile_shape,
        mut maybe_beam,
    ) in projectiles.iter_mut()
    {
        for (ufo_entity, mut ufo, ufo_transform, ufo_shape) in ufos.iter_mut() {
            if projectile_shape.intersects(ufo_shape) {
                match *projectile {
                    ShipProjectile::Rapid | ShipProjectile::Spread => {
                        commands.entity(projectile_entity).despawn();
                        if ufo.life > 0 {
                            ufo.life -= 1;
                        }
                    }
                    ShipProjectile::Plasma { mut power } => {
                        let overlap = -projectile_shape.distance(ufo_shape).min(0.0);
                        let effect = overlap.min(ufo.life as f32);
                        power -= effect;
                        *projectile_shape = CollisionShape::new(
                            Shape::Circle {
                                center: Vec2::ZERO,
                                radius: power,
                            },
                            *projectile_transform,
                        );
                        if power <= 0.0 {
                            commands.entity(projectile_entity).despawn();
                        } else {
                            projectile_transform.scale = Vec3::splat(power / 16.0);
                        }
                        if ufo.life > 0 {
                            ufo.life -= effect.ceil() as i32;
                        }
                    }
                    ShipProjectile::Beam { .. } => {
                        if let Some(ref mut beam) = maybe_beam {
                            beam.length = projectile_shape.distance(ufo_shape);
                            if beam.cooldown <= 0.0 {
                                ufo.life -= BEAM_DAMAGE_PER_HIT;
                                beam.cooldown = BEAM_HIT_INTERVAL;
                            }
                        }
                    }
                }

                let point = projectile_shape.collision_point(ufo_shape);
                let direction = (point - ufo_transform.translation.truncate()).normalize();
                for _ in 0..10 {
                    let speed = lerp(10.0, 100.0, random());
                    let velocity =
                        (direction + (direction.perp() * lerp(-0.5, 0.5, random()))) * speed;
                    let acceleration = Vec2::ZERO;
                    commands.spawn(SparkParticleBundle::new(
                        point,
                        velocity,
                        acceleration,
                        &sprite_sheets.particles,
                    ));
                }
            }
            if ufo.life <= 0 {
                let speed = lerp(30.0, 80.0, random());
                let velocity = Vec2::from_angle(random::<f32>() * std::f32::consts::TAU) * speed;
                let position = ufo_transform.translation.truncate();
                commands.spawn(PowerupBundle::new(
                    random(),
                    position,
                    velocity,
                    5.0,
                    &sprite_sheets.powerup,
                ));
                commands.spawn(ExplosionBundle::new(&sprite_sheets.explosion, position));
                commands.spawn(WaveParticleBundle::new(position, &sprite_sheets.particles));
                score.increase(100);
                commands.spawn(GameNotificationBundle::new(
                    format!("{}", score.value()),
                    asset_server.load("fonts/DejaVuSans.ttf"),
                    position,
                    20.0,
                    1.0,
                ));
                commands.entity(ufo_entity).despawn();
                break;
            }
        }
    }
}

#[derive(Bundle)]
struct UfoBundle {
    sprite_bundle: SpriteBundle,
    ufo: Ufo,
    level_entity: LevelEntity,
    collision_shape: CollisionShape,
}
impl UfoBundle {
    pub fn new(ufo_images: &UfoImages, ufo: Ufo) -> Self {
        let center = ufo.start_position.clone();
        UfoBundle {
            sprite_bundle: SpriteBundle {
                texture: ufo_images.ship[0].clone(),
                transform: Transform::from_translation(ufo.start_position.extend(0.)),
                ..Default::default()
            },
            ufo,
            level_entity: LevelEntity,
            collision_shape: CollisionShape::new(
                Shape::Circle {
                    center: Vec2::ZERO,
                    radius: 16.0,
                },
                Transform::from_translation(center.extend(0.)),
            ),
        }
    }
}

#[derive(Bundle)]
struct UfoLaserBundle {
    sprite_bundle: SpriteBundle,
    ufo_laser: UfoLaser,
    moving: Moving,
    expiring: Expiring,
    collision_shape: CollisionShape,
}
impl UfoLaserBundle {
    pub fn new(
        ufo_images: &UfoImages,
        position: Vec2,
        rotation: f32,
        velocity: Vec2,
        life: f32,
    ) -> Self {
        UfoLaserBundle {
            sprite_bundle: SpriteBundle {
                texture: ufo_images.laser.clone(),
                transform: Transform::from_translation(position.extend(0.))
                    .with_rotation(Quat::from_rotation_z(rotation)),
                ..Default::default()
            },
            ufo_laser: UfoLaser,
            moving: Moving {
                velocity,
                ..Default::default()
            },
            expiring: Expiring { life },
            collision_shape: CollisionShape::new(
                Shape::Circle {
                    center: Vec2::ZERO,
                    radius: 1.0,
                },
                Transform::from_translation(position.extend(0.)),
            ),
        }
    }
}