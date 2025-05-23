//! The `enemies` module manages all logic in the `ecs.rs` file since all waves share the same mechanics.
//! The difficulty of each wave increases deterministically, so we only need to implement algorithms
//! for spawning and wave control over time.
//!
//! In the `tower_building` module, we don't have a dedicated `ecs.rs` file because tower mechanics
//! consist of two distinct processes:
//! 1️) **Building** - Placing and upgrading towers.
//! 2️) **Attacking** - Managing tower targeting and shooting behavior.
//!
//! These processes require separate handling to ensure proper management and scalability.

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::{
    solana::{update_player_values, PlayerInfo, SolClient, Tasks, Wallet},
    tower_building::{GameState, Lifes},
};

use super::{
    EnemyAnimation, EnemyAnimationState, WaveControl, INITIAL_ENEMY_LIFE, MAX_ENEMIES_PER_WAVE,
    SCALAR, SCALE, SPAWN_X_LOCATION, SPAWN_Y_LOCATION,
};

#[derive(Component)]
pub struct Enemy {
    pub life: u16,
    pub speed: f32,
}

#[derive(Debug, Component, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
pub struct BreakPointLvl(pub u8);

pub fn spawn_wave(mut commands: Commands, time: Res<Time>, mut wave_control: ResMut<WaveControl>) {
    if wave_control.wave_count == wave_control.textures.len() as u8 {
        return;
    }

    wave_control.time_between_spawns.tick(time.delta());

    if wave_control.spawned_count_in_wave < MAX_ENEMIES_PER_WAVE
        && wave_control.time_between_spawns.just_finished()
    {
        let wave_image = &wave_control.textures[wave_control.wave_count as usize];
        let enemy_animation = &wave_control.animations[wave_control.wave_count as usize];
        let enemy_life = (INITIAL_ENEMY_LIFE as f32
            * (1.2 + SCALAR).powf(wave_control.wave_count as f32))
        .round() as u16;
        let enemy_speed = (75.0 * (1.05f32).powf(wave_control.wave_count as f32)).min(300.0);
        info!("enemy life: {}, enemy speed: {:?}", enemy_life, enemy_speed);

        commands.spawn((
            Sprite::from_atlas_image(
                wave_image.0.clone(),
                TextureAtlas {
                    layout: wave_image.1.clone(),
                    index: enemy_animation.walk_left.first,
                },
            ),
            Transform {
                translation: Vec3::new(SPAWN_X_LOCATION, SPAWN_Y_LOCATION, 1.0),
                scale: Vec3::splat(SCALE),
                ..default()
            },
            Enemy {
                life: enemy_life,
                speed: enemy_speed,
            },
            enemy_animation.clone(),
            BreakPointLvl(0),
        ));
        wave_control.spawned_count_in_wave += 1;
    }
}

/// Defines a set of predefined points in the game world where enemies change direction.
/// These points dictate the movement path of the enemies.
pub const BREAK_POINTS: [Vec2; 6] = [
    Vec2::new(260.0, SPAWN_Y_LOCATION),
    Vec2::new(260.0, -205.0),
    Vec2::new(-230.0, -205.0),
    Vec2::new(-230.0, SPAWN_Y_LOCATION),
    Vec2::new(-455.0, SPAWN_Y_LOCATION),
    Vec2::new(-455.0, -375.0),
];

/// Moves enemies along a predefined path based on their current position and speed.
/// The movement is determined by comparing the enemy’s position to predefined breakpoints.
/// Once an enemy reaches a specific breakpoint, it updates its direction accordingly.
pub fn move_enemies(
    mut enemies: Query<(
        &mut Transform,
        &Enemy,
        &mut BreakPointLvl,
        &mut EnemyAnimation,
    )>,
    time: Res<Time>,
) {
    for (mut enemy_transform, enemy, mut breal_point_lvl, mut enemy_animation) in &mut enemies {
        let translation = enemy_transform.translation;
        let speed = enemy.speed * time.delta_secs();

        // 1. -x
        if translation.x > BREAK_POINTS[0].x {
            enemy_transform.translation.x -= speed;
            if enemy_animation.need_flip {
                enemy_transform.scale.x = -SCALE;
            }
        }
        // 2. -y
        else if translation.x <= BREAK_POINTS[0].x
            && translation.x > BREAK_POINTS[2].x
            && translation.y > BREAK_POINTS[1].y
        {
            enemy_transform.translation.y -= speed;
            enemy_animation.state = EnemyAnimationState::WalkDown;
            *breal_point_lvl = BreakPointLvl(1);
        }
        // 3. -x
        else if translation.y <= BREAK_POINTS[1].y && translation.x >= BREAK_POINTS[2].x {
            enemy_transform.translation.x -= speed;
            enemy_animation.state = EnemyAnimationState::WalkLeft;
            *breal_point_lvl = BreakPointLvl(2);
        }
        // 4. +y
        else if translation.y < SPAWN_Y_LOCATION
            && translation.x <= BREAK_POINTS[2].x
            && translation.x > BREAK_POINTS[4].x
        {
            enemy_animation.state = EnemyAnimationState::WalkUp;
            enemy_transform.translation.y += speed;
            *breal_point_lvl = BreakPointLvl(3);
        }
        // 5. -x
        else if translation.y >= SPAWN_Y_LOCATION && translation.x >= BREAK_POINTS[4].x {
            enemy_transform.translation.x -= speed;
            enemy_animation.state = EnemyAnimationState::WalkLeft;
            *breal_point_lvl = BreakPointLvl(4);
        }
        // 6. -y
        else if translation.y > BREAK_POINTS[5].y && translation.x <= BREAK_POINTS[4].x {
            enemy_transform.translation.y -= speed;
            enemy_animation.state = EnemyAnimationState::WalkDown;
            *breal_point_lvl = BreakPointLvl(5);
        }
    }
}

pub fn game_over(
    mut commands: Commands,
    mut enemies: Query<(&Transform, Entity), With<Enemy>>,
    mut lifes: ResMut<Lifes>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (enemy_transform, entity) in &mut enemies {
        let translation = enemy_transform.translation;
        if translation.y <= BREAK_POINTS[5].y {
            commands.entity(entity).despawn();
            lifes.0 = lifes.0.saturating_sub(1);
        }
    }
    if lifes.0 == 0 {
        game_state.set(GameState::GameOver);
    }
}

pub fn reset_wave_control_on_game_over(mut wave_control: ResMut<WaveControl>) {
    wave_control.wave_count = 0;
    wave_control.spawned_count_in_wave = 0;
    wave_control.time_between_waves.unpause();
    wave_control.time_between_waves.reset();
    wave_control.time_between_spawns.reset();
    wave_control.first_wave_spawned = false;
}

pub fn despawn_all_enemies_in_game_over(
    mut enemies: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) {
    for entity in &mut enemies {
        commands.entity(entity).despawn();
    }
}

pub fn wave_control(
    time: Res<Time>,
    mut wave_control: ResMut<WaveControl>,
    enemies: Query<Entity, With<Enemy>>,
    mut game_state: ResMut<NextState<GameState>>,
    solana_resources: (
        ResMut<Tasks>,
        ResMut<Wallet>,
        Res<SolClient>,
        Res<PlayerInfo>,
    ),
) {
    // tick cooldown timer
    wave_control.time_between_waves.tick(time.delta());

    if !wave_control.first_wave_spawned {
        // start first wave after timer ends
        if wave_control.time_between_waves.just_finished() {
            game_state.set(GameState::Attacking);
            wave_control.time_between_waves.pause();
            wave_control.time_between_waves.reset();
            info!("first wave started");
            wave_control.first_wave_spawned = true;
        }
    }

    let all_enemies_killed = enemies.iter().next().is_none();
    let wave_fully_spawned = wave_control.spawned_count_in_wave == MAX_ENEMIES_PER_WAVE;

    if wave_fully_spawned && all_enemies_killed {
        // control cooldown between waves
        if wave_control.time_between_waves.paused() {
            wave_control.time_between_waves.unpause();
            wave_control.time_between_waves.reset();
            game_state.set(GameState::Building);
        }

        if wave_control.time_between_waves.just_finished() {
            wave_control.spawned_count_in_wave = 0;
            wave_control.wave_count += 1;
            let (mut tasks, signer, client, player_info) = solana_resources;
            let now = SystemTime::now();
            let last_time_played = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
            info!("last_time_played: {}", last_time_played);
            tasks.add_task(update_player_values(
                signer.keypair.clone(),
                client.clone(),
                wave_control.wave_count,
                last_time_played,
                player_info.address,
            ));
            wave_control.time_between_waves.pause();
            wave_control.time_between_waves.reset();
            game_state.set(GameState::Attacking);
            info!(
                "cooldown finished, starting wave: {}",
                wave_control.wave_count
            );
        }
    }
}
