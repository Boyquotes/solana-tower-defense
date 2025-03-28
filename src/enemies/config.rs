//! Wave control determines how strong and fast enemies get with each wave.
//! Every wave is tougher than the last, so we need to **manage** how these values scale over time.
//!
//! This file handles that, so if you want enemies to attack faster, deal more damage, or take more hits,
//! this is where you make the changes.

use bevy::prelude::*;

use crate::tower_building::GameState;

use super::{AnimateSprite, EnemyAnimation, EnemyAnimationState};

pub const MAX_ENEMIES_PER_WAVE: u8 = 25;
pub const SPAWN_Y_LOCATION: f32 = 70.0;
pub const SPAWN_X_LOCATION: f32 = 610.0;
pub const TIME_BETWEEN_WAVES: f32 = 15.0;
pub const TIME_BETWEEN_SPAWNS: f32 = 1.5;
pub const INITIAL_ENEMY_LIFE: u16 = 60;
pub const SCALAR: f32 = 0.8;
pub const SCALE: f32 = 2.0;

/// Controls enemy waves, including spawn timing, textures, animations, and wave progression.
/// This resource is globally accessible to check and validate wave data.
#[derive(Resource, Debug)]
pub struct WaveControl {
    /// Current wave number.
    pub wave_count: u8,

    /// Timer controlling the interval between enemy spawns within a wave.
    pub time_between_spawns: Timer,

    /// List of enemy textures and their corresponding texture atlas layouts.
    pub textures: Vec<(Handle<Image>, Handle<TextureAtlasLayout>)>,

    /// Animations assigned to enemies.
    pub animations: Vec<EnemyAnimation>,

    /// Number of enemies spawned in the current wave.
    pub spawned_count_in_wave: u8,

    /// Timer controlling the interval between waves.
    pub time_between_waves: Timer,
}

pub fn load_enemy_sprites(
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut commands: Commands,
) {
    let mut textures: Vec<(Handle<Image>, Handle<TextureAtlasLayout>)> = Vec::new();
    let mut animations: Vec<EnemyAnimation> = Vec::new();

    let enemy_list = vec![
        (
            "enemies/orcs.png",
            UVec2::splat(48),
            8,
            6,
            EnemyAnimation {
                walk: AnimateSprite {
                    first: 8,
                    last: 15,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                death: AnimateSprite {
                    first: 40,
                    last: 43,
                    timer: Timer::from_seconds(0.25, TimerMode::Repeating),
                },
                state: EnemyAnimationState::Walk,
            },
        ),
        (
            "enemies/soldier.png",
            UVec2::new(43, 31),
            7,
            6,
            EnemyAnimation {
                walk: AnimateSprite {
                    first: 0,
                    last: 6,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                death: AnimateSprite {
                    first: 40,
                    last: 43,
                    timer: Timer::from_seconds(0.25, TimerMode::Repeating),
                },

                state: EnemyAnimationState::Walk,
            },
        ),
        (
            "enemies/Leafbug.png",
            UVec2::new(64, 64),
            8,
            9,
            EnemyAnimation {
                walk: AnimateSprite {
                    first: 40,
                    last: 47,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                death: AnimateSprite {
                    first: 55,
                    last: 62,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },

                state: EnemyAnimationState::Walk,
            },
        ),
        (
            "enemies/Firebug.png",
            UVec2::new(128, 64),
            11,
            9,
            EnemyAnimation {
                walk: AnimateSprite {
                    first: 55,
                    last: 62,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },
                death: AnimateSprite {
                    first: 55,
                    last: 62,
                    timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                },

                state: EnemyAnimationState::Walk,
            },
        ),
    ];

    for (path, tile_size, columns, row, animation) in enemy_list {
        let texture = asset_server.load(path);
        let texture_atlas = TextureAtlasLayout::from_grid(tile_size, columns, row, None, None);
        let atlas_handle = texture_atlas_layouts.add(texture_atlas);

        textures.push((texture, atlas_handle));
        animations.push(animation);
    }

    commands.insert_resource(WaveControl {
        textures,
        animations,
        wave_count: 0,
        time_between_spawns: Timer::from_seconds(TIME_BETWEEN_SPAWNS, TimerMode::Repeating),
        spawned_count_in_wave: 0,
        time_between_waves: Timer::from_seconds(TIME_BETWEEN_WAVES, TimerMode::Once),
    });
}

pub fn control_first_wave(
    time: Res<Time>,
    mut wave_control: ResMut<WaveControl>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if wave_control.wave_count == 0 {
        wave_control.time_between_waves.tick(time.delta());
        if wave_control.time_between_waves.just_finished() {
            game_state.set(GameState::Attacking);
            wave_control.time_between_waves.pause();
            wave_control.time_between_waves.reset();
            info!("fist wave controlled");
        }
    }
}
