use bevy::prelude::*;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};

use crate::{
    enemies::WaveControl,
    solana::Wallet,
    tower_building::{GameState, Gold, Lifes},
};

use super::*;

pub struct UiPlugin;

#[derive(Component)]
pub enum TextType {
    GoldText,
    WaveCountText,
    LifesText,
    WalletBalanceText,
    WalletAddressText,
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_sign_message_to_start)
            .add_systems(OnExit(GameState::Start), spawn_how_to_play_ui)
            .add_systems(OnExit(GameState::HowToPlay), spawn_game_ui)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui)
            .add_systems(OnEnter(GameState::Building), spawn_tower_selected_text)
            .add_systems(OnExit(GameState::Building), despawn_selected_tower_ui)
            .add_systems(Update, (handle_btn_interaction, update_ui_texts))
            .add_systems(
                Update,
                update_tower_selected_text.run_if(in_state(GameState::Building)),
            );
    }
}

pub const MAX_COLOR_VALUE: f32 = 255.0;
pub const BORDER_AND_TEXT_UI_COLOR: Color = Color::srgb(
    224.0 / MAX_COLOR_VALUE,
    162.0 / MAX_COLOR_VALUE,
    125.0 / MAX_COLOR_VALUE,
);
pub const BACKGROUND_COLOR: Color = Color::srgb(
    78.0 / MAX_COLOR_VALUE,
    43.0 / MAX_COLOR_VALUE,
    47.0 / MAX_COLOR_VALUE,
);

// This part is the stats/values the player have after start the game
pub fn spawn_game_ui(mut commands: Commands, wallet: Res<Wallet>) {
    // think of this root_ui like a div in html that wraps all the other divs xd
    // it defines where the ui will be positioned, and from there, you spawn
    // the rest of the components as children. Pretty much like how you'd do it in html
    let root_ui = commands
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Auto,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                position_type: PositionType::Absolute,
                right: Val::Percent(3.0),
                border: UiRect::all(Val::Px(5.0)),
                top: Val::Percent(60.0),
                ..default()
            },
            BorderColor(BORDER_AND_TEXT_UI_COLOR),
            BorderRadius::all(Val::Px(15.0)),
            Name::new("UI Root"),
            BackgroundColor(BACKGROUND_COLOR),
        ))
        .id();

    let create_text =
        |commands: &mut Commands, text: &str, text_type: TextType, bottom_padding: f32| {
            commands.entity(root_ui).with_children(|p| {
                p.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextColor(BORDER_AND_TEXT_UI_COLOR),
                    text_type,
                ));
            });
            commands.entity(root_ui).with_children(|p| {
                p.spawn(Node {
                    height: Val::Px(bottom_padding),
                    ..default()
                });
            });
        };

    let _gold_text = create_text(&mut commands, "Gold: 0", TextType::GoldText, 10.0);

    let _wave_count_text = create_text(
        &mut commands,
        "Wave Count: 0",
        TextType::WaveCountText,
        10.0,
    );

    let _lifes_text = create_text(&mut commands, "Lifes: 30", TextType::LifesText, 10.0);

    let _sol_balance_text = create_text(
        &mut commands,
        "Sol Balance: 0.0",
        TextType::WalletBalanceText,
        10.0,
    );

    let wallet_str = wallet.keypair.pubkey().to_string();
    let shortened_wallet = format!(
        "{}...{}",
        &wallet_str[0..4],
        &wallet_str[wallet_str.len() - 4..]
    );

    let _wallet_address = create_text(
        &mut commands,
        &format!("Wallet Address: {}", shortened_wallet),
        TextType::WalletAddressText,
        10.0,
    );
}

// Update in real-time the UI texts with the resources states
pub fn update_ui_texts(
    mut texts: Query<(&mut Text, &TextType)>,
    resources: (Res<Gold>, Res<Lifes>, Res<Wallet>, Res<WaveControl>),
) {
    let (gold, lifes, wallet, wave_control) = resources;
    for (mut text, text_type) in &mut texts {
        match text_type {
            TextType::GoldText => text.0 = format!("Gold: {:?}", gold.0),
            TextType::WaveCountText => {
                text.0 = format!("Wave count: {}", wave_control.wave_count + 1)
            }
            TextType::LifesText => text.0 = format!("Lifes: {:?}", lifes.0),
            TextType::WalletBalanceText => {
                text.0 = format!(
                    "Sol Balance: {:.2}",
                    wallet.balance as f32 / LAMPORTS_PER_SOL as f32
                )
            }
            TextType::WalletAddressText => {
                // here we can add logic to update the text wallet address if the wallet change in any time
            }
        }
    }
}
