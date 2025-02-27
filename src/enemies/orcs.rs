use bevy::prelude::*;

use crate::{
    animations::{animate_orcs, OrcsAnimation, OrcsAnimationState},
    player::Player,
};

use super::Enemy;

// define plugin
pub struct OrcsPlugin;

impl Plugin for OrcsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_orcs)
            .add_systems(Update, (animate_orcs, follow_player_and_attack));
    }
}

// define components

#[derive(Component)]
pub struct Orcs {
    pub life: u8,
    pub attack_damage: u8,
    pub speed: f32,
    pub attack_cooldown: Timer,
}

impl Default for Orcs {
    fn default() -> Self {
        Self {
            life: 100,
            attack_damage: 10,
            speed: 125.0,
            attack_cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
        }
    }
}

// define systems
const SPAWN_AMOUNT: u8 = 10;
pub fn spawn_orcs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("enemies/orcs/orc_sprite_sheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(48), 8, 6, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let orcs_animation = OrcsAnimation::default();

    for i in 0..SPAWN_AMOUNT {
        commands.spawn((
            Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: orcs_animation.idle.first,
                },
            ),
            Transform {
                translation: Vec3::new(150.0 * i as f32, -125.0, 1.0),
                scale: Vec3::splat(2.0),
                ..default()
            },
            Orcs::default(),
            Enemy,
            orcs_animation.clone(),
        ));
    }
}

pub fn attack() {}

const MAX_AGRO_DISTANCE: f32 = 250.0;
const ATTACK_DISTANCE: f32 = 50.0;
const MIN_ORC_SEPARATION: f32 = 25.0;
const SEPARATION_STRENGTH: f32 = 2.0;

pub fn follow_player_and_attack(
    player: Query<&Transform, With<Player>>,
    mut orcs: Query<(&mut Transform, &Orcs, &mut OrcsAnimation), Without<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player.get_single() {
        let player_position = player_transform.translation.truncate();

        let orcs_positions: Vec<Vec2> = orcs
            .iter()
            .map(|(transform, _, _)| transform.translation.truncate())
            .collect();

        for (i, (mut orc_transform, orcs, mut orcs_animation)) in orcs.iter_mut().enumerate() {
            let orc_position = orc_transform.translation.truncate();
            let mut direction = (player_position - orc_position).normalize_or_zero();
            let distance = orc_position.distance(player_position);

            for (j, other_position) in orcs_positions.iter().enumerate() {
                if i != j {
                    let distance = orc_position.distance(*other_position);
                    if distance < MIN_ORC_SEPARATION {
                        // calculate repelling force away from other orcs
                        let repel_force = (orc_position - *other_position).normalize_or_zero();
                        direction += repel_force * SEPARATION_STRENGTH;
                    }
                }
            }

            if distance <= MAX_AGRO_DISTANCE && distance > ATTACK_DISTANCE {
                orcs_animation.state = OrcsAnimationState::Walk;
                let orcs_speed = orcs.speed * time.delta_secs();
                orc_transform.translation.x += direction.x * orcs_speed;
                orc_transform.translation.y += direction.y * orcs_speed;

                if direction.x < 0.0 {
                    orc_transform.scale.x = -2.0;
                } else if direction.x > 0.0 {
                    orc_transform.scale.x = 2.0;
                }
            } else if distance <= ATTACK_DISTANCE {
                orcs_animation.state = OrcsAnimationState::Attack;
            } else {
                orcs_animation.state = OrcsAnimationState::Idle;
            }
        }
    }
}
