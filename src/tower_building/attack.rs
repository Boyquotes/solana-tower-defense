use core::f32;

use bevy::prelude::*;

use crate::{
    enemies::{BreakPointLvl, Enemy, WaveControl, BREAK_POINTS},
    tower_building::{DESPAWN_SHOT_RANGE, SHOT_HURT_DISTANCE, SHOT_SPEED},
};

use super::{Gold, Tower, TowerControl, TOWER_ATTACK_RANGE};

#[derive(Component)]
pub struct Shot {
    pub damage: u16,
    pub target: Option<(Entity, Vec3)>,
    pub animation_timer: Timer,
}

/// Spawns shots from towers targeting the most "dangerous" enemies.
///
/// # How it works:
/// Each tower scans for enemies within its attack range, filtering them based on their **breakpoint level**,
/// which represents how close they are to victory. The tower prioritizes enemies with the
/// highest breakpoint level, and if multiple enemies share the highest breakpoint level, it selects
/// the one closest to its designated **breakpoint position**.
/// Once a target is selected and the attack timer completes, the tower spawns a shot aimed at the enemy.
///
/// # Shot Behavior:
/// The shot is assigned a direction towards the targeted enemy and carries the tower's damage value. It includes
/// an animation timer and uses a **texture atlas** to handle sprite animation.

pub fn spawn_shots(
    enemies: Query<(&Transform, &BreakPointLvl, Entity), (Without<Tower>, With<Enemy>)>,
    mut towers: Query<(&Transform, &mut Tower)>,
    mut commands: Commands,
    time: Res<Time>,
    tower_control: Res<TowerControl>,
) {
    for (tower_transform, mut tower) in &mut towers {
        let tower_position = tower_transform.translation;
        tower.attack_speed.tick(time.delta());

        let mut target_enemy_position = None;
        let mut closest_distance_to_target = f32::MAX;
        // find all enemies within the tower's attack range
        let enemies_in_range: Vec<(&Transform, &BreakPointLvl, Entity)> = enemies
            .iter()
            .filter(|(t, _, _)| {
                let enemy_position = t.translation;
                let distance = tower_position.distance(enemy_position);
                distance < TOWER_ATTACK_RANGE && distance > 0.0
            })
            .collect();

        // identify the highest breakpoint level among the enemies in range
        let max_break_value = enemies_in_range
            .iter()
            .cloned()
            .map(|(_, b, _)| b)
            .max()
            .unwrap_or(&BreakPointLvl(0));

        // select all enemies that share this highest breakpoint level
        let closer_enemies_to_victory: Vec<(&Transform, &BreakPointLvl, Entity)> = enemies_in_range
            .iter()
            .filter(|(_, b, _)| **b == *max_break_value)
            .copied()
            .collect();

        // determine the enemy closest to its designated breakpoint
        let mut closest_enemy = None;
        for (enemy_transform, break_point_lvl, enemy_entity) in &closer_enemies_to_victory {
            let index = break_point_lvl.0 as usize;
            let enemy_position = enemy_transform.translation;
            let distance_to_target = enemy_position.truncate().distance(BREAK_POINTS[index]);

            if distance_to_target < closest_distance_to_target {
                closest_distance_to_target = distance_to_target;
                target_enemy_position = Some(enemy_position);
                closest_enemy = Some(enemy_entity);
            }
        }
        if let Some(enemy_position) = target_enemy_position {
            if tower.attack_speed.just_finished() {
                let shot = Shot {
                    damage: tower.attack_damage,
                    target: Some((*closest_enemy.unwrap(), enemy_position)),
                    animation_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
                };
                let (texture, atlas_handle) = tower_control
                    .shot_textures
                    .get(&tower.tower_type)
                    .expect("A shot texture layout is expected to be loaded");

                commands.spawn((
                    Sprite::from_atlas_image(
                        texture.clone(),
                        TextureAtlas {
                            layout: atlas_handle.clone(),
                            index: 0,
                        },
                    ),
                    shot,
                    Transform {
                        translation: Vec3::new(tower_position.x, tower_position.y + 40.0, 1.5),
                        ..default()
                    },
                ));
            }
        }
    }
}

pub fn move_shots_to_enemies(
    mut enemies: Query<(Entity, &Transform, &mut Enemy), Without<Shot>>,
    mut shots: Query<(Entity, &mut Transform, &mut Shot, &mut Sprite)>,
    mut commands: Commands,
    mut gold: ResMut<Gold>,
    time: Res<Time>,
    wave_control: Res<WaveControl>,
) {
    for (shot_entity, mut transform, mut shot, mut shot_sprite) in &mut shots {
        if let Some((target_entity, _)) = shot.target {
            if let Ok((enemy_entity, enemy_transform, mut enemy)) = enemies.get_mut(target_entity) {
                let direction = (enemy_transform.translation - transform.translation).normalize();
                transform.translation += direction * SHOT_SPEED * time.delta_secs();

                shot.target = Some((target_entity, enemy_transform.translation));

                let distance = transform
                    .translation
                    .distance_squared(enemy_transform.translation);

                if distance <= SHOT_HURT_DISTANCE {
                    shot.animation_timer.tick(time.delta());
                    if let Some(shot_texture_atlas) = &mut shot_sprite.texture_atlas {
                        if shot.animation_timer.just_finished() {
                            shot_texture_atlas.index += 1;
                        }
                    }

                    if shot_sprite
                        .texture_atlas
                        .as_ref()
                        .map_or(true, |atlas| atlas.index >= 7)
                    {
                        enemy.life = enemy.life.saturating_sub(shot.damage);
                        if enemy.life == 0 {
                            commands.entity(enemy_entity).despawn();

                            let wave_factor = wave_control.wave_count as f32 + 1.0;
                            let gold_reward =
                                ((enemy.life as f32 / 2.5) + (wave_factor * 2.0)).round() as u16;

                            gold.0 += gold_reward;
                            info!("Enemy killed! Gained {} gold.", gold_reward);
                        }

                        commands.entity(shot_entity).despawn();
                    }
                }
            }
        }
    }
}

pub fn despawn_shots_with_killed_target(
    mut shots: Query<(&Shot, &mut Sprite, &mut Transform, Entity), Without<Enemy>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (shot, mut shot_sprite, mut transform, shot_entity) in &mut shots {
        if let Some((target, enemy_last_position)) = shot.target {
            if enemies.get(target).is_ok() {
                continue;
            }

            if let Some(shot_texture_atlas) = &mut shot_sprite.texture_atlas {
                shot_texture_atlas.index = 0;
            }
            let direction = (enemy_last_position - transform.translation).normalize();
            let movement = direction * SHOT_SPEED * time.delta_secs();
            let new_position = transform.translation + movement;

            if new_position.distance_squared(enemy_last_position) <= 50.0 {
                transform.translation = enemy_last_position;
                commands.entity(shot_entity).despawn();
            } else {
                transform.translation = new_position;
            }

            if transform
                .translation
                .truncate()
                .distance(Vec2::new(0.0, 0.0))
                > DESPAWN_SHOT_RANGE
            {
                commands.entity(shot_entity).despawn();
            }
        }
    }
}

// this is necessary because, at the end of a wave, some shots can get stuck when the GameState
// switches to Building, causing all shot-related systems to stop running. this ensures any
// remaining shots are properly removed
pub fn delete_all_shots_on_building(mut shots: Query<Entity, With<Shot>>, mut commands: Commands) {
    for shot in &mut shots {
        commands.entity(shot).despawn();
    }
}
