use bevy::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum AsteroidSize {
    Tiny = 0,
    Small,
    Medium,
    Large,
}

impl AsteroidSize {
    pub fn smaller(&self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Tiny => None,
            AsteroidSize::Small => Some(AsteroidSize::Tiny),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Large => Some(AsteroidSize::Medium),
        }
    }
    pub fn radius(&self) -> f32 {
        match self {
            AsteroidSize::Tiny => 4.0,
            AsteroidSize::Small => 8.0,
            AsteroidSize::Medium => 16.0,
            AsteroidSize::Large => 24.0,
        }
    }
}

pub const ASTEROID_SIZES: usize = 4;
pub const ASTEROID_VARIANTS: usize = 12;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ShipWeapon {
    Rapid,
    Spread,
    Beam,
    Plasma,
}

#[derive(Component, Clone, Copy)]
pub enum ShipProjectile {
    Rapid,
    Spread,
    Beam { power: f32 },
    Plasma { power: f32 },
}

#[derive(Component)]
pub struct Beam {
    pub length: f32,
}

#[derive(Clone, Copy)]
pub enum ShipTurn {
    Neutral,
    Left,
    Right,
}

impl Default for ShipTurn {
    fn default() -> Self {
        Self::Neutral
    }
}

#[derive(Component)]
pub struct Expiring {
    pub life: f32,
}

impl Default for ShipWeapon {
    fn default() -> Self {
        Self::Rapid
    }
}

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
    pub integrity: i32,
    pub variant: usize,
}

#[derive(Component)]
pub struct Ufo {
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
pub struct UfoLaser;

#[derive(Component)]
pub enum Powerup {
    Laser = 0,
    Spread,
    Beam,
    Plasma,
    ExtraLife,
    LoseLife,
    Shield,
}

#[derive(Component, Default)]
pub struct Moving {
    pub velocity: Vec2,
    pub acceleration: Vec2,
}

#[derive(Component)]
pub struct Spinning {
    pub speed: f32,
}

#[derive(Component)]
pub struct Wrapping;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component, Default, PartialEq, Eq)]
pub struct HUD {
    pub level: u32,
    pub score: u32,
    pub lives: u8,
    pub weapon: ShipWeapon,
    pub weapon_rapid_level: u8,
    pub weapon_spread_level: u8,
    pub weapon_beam_level: u8,
    pub weapon_plasma_level: u8,
}

#[derive(Component, Default)]
pub struct Ship {
    pub throttle: bool,
    pub turn: ShipTurn,
    pub fire: bool,
    pub weapon: ShipWeapon,
    pub weapon_rapid_level: u8,
    pub weapon_spread_level: u8,
    pub weapon_beam_level: u8,
    pub weapon_plasma_level: u8,
    pub weapon_cooldown: f32,
    pub shield_level: u8,
    pub lives: u8,
}

#[derive(Component)]
pub struct ShipShield;
