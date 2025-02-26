use bevy::prelude::*;

use super::AnimateSprite;

#[derive(Component, Clone)]
pub struct OrcsAnimation {
    pub walk: AnimateSprite,
    pub idle: AnimateSprite,
    pub attack: AnimateSprite,
    pub death: AnimateSprite,
    pub state: OrcsAnimationState,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum OrcsAnimationState {
    Walk,
    Idle,
    Attack,
    Death,
}

impl Default for OrcsAnimation {
    fn default() -> Self {
        Self {
            walk: AnimateSprite {
                first: 0,
                last: 7,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            idle: AnimateSprite {
                first: 0,
                last: 4,
                timer: Timer::from_seconds(0.25, TimerMode::Repeating),
            },
            attack: AnimateSprite {
                first: 0,
                last: 6,
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            death: AnimateSprite {
                first: 0,
                last: 4,
                timer: Timer::from_seconds(0.25, TimerMode::Repeating),
            },
            state: OrcsAnimationState::Idle,
        }
    }
}

pub fn animate_orcs(
    mut orcs_animation_query: Query<(&mut Transform, &mut Sprite, &mut OrcsAnimation)>,
    time: Res<Time>,
) {
    for (mut _transform, mut orc_sprite, mut orc_animation) in &mut orcs_animation_query {
        let animation = match orc_animation.state {
            OrcsAnimationState::Walk => &mut orc_animation.walk,
            OrcsAnimationState::Idle => &mut orc_animation.idle,
            OrcsAnimationState::Attack => &mut orc_animation.attack,
            OrcsAnimationState::Death => &mut orc_animation.death,
        };
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if let Some(atlas) = &mut orc_sprite.texture_atlas {
                atlas.index = if atlas.index < animation.first || atlas.index >= animation.last {
                    animation.first
                } else {
                    atlas.index + 1
                };
            };
        }
    }
}
