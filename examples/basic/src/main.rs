//! Sprite effects showcase — a row of sprites, each demonstrating a different effect type.
//!
//! Demonstrates:
//! - `FlashEffect` with tint blend mode and a screen-style showcase flash preset
//! - `PaletteSwap` mapping source palette row 0 to target row 1
//! - `DissolveEffect` using a radial mask texture for a reveal animation
//! - How each effect component is attached directly to a `Sprite` entity

use bevy::prelude::*;
use common::{
    add_demo_assets, install_auto_exit, setup_camera, showcase_reveal_dissolve_config,
    showcase_screen_flash_config,
};
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, DissolvePattern, FlashBlendMode, FlashConfig, FlashEffect,
    PaletteConfig, PaletteSwap, SpriteEffectsPlugin,
};
use saddle_rendering_sprite_effects_example_common as common;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects basic",
        "Hybrid sprite feedback: native tint/squash plus shader-backed dissolve and palette swap.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    // ---------------------------------------------------------------------------
    // Effect 1: Tint flash — red color, tint blend mode, 0.22s duration
    // ---------------------------------------------------------------------------
    commands.spawn((
        Name::new("Tint Flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-280.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        FlashEffect::new(FlashConfig {
            color: Color::srgb(1.0, 0.26, 0.26),
            intensity: 1.0,
            duration_secs: 0.22,
            blend: FlashBlendMode::Tint,
            ..FlashConfig::default()
        }),
    ));

    // ---------------------------------------------------------------------------
    // Effect 2: Screen flash — showcase preset using screen blend mode
    // ---------------------------------------------------------------------------
    commands.spawn((
        Name::new("Screen Flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-90.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        FlashEffect::new(showcase_screen_flash_config()),
    ));

    // ---------------------------------------------------------------------------
    // Effect 3: Palette swap — 4-column palette texture, row 0 -> row 1
    // ---------------------------------------------------------------------------
    commands.spawn((
        Name::new("Palette Swap"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(100.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        PaletteSwap::new(PaletteConfig {
            texture: assets.palette.clone(),
            columns: 4,
            source_row: 0,
            target_row: 1,
            epsilon: 0.01,
            preserve_alpha: true,
            enforce_nearest_sampling: true,
        }),
    ));

    // ---------------------------------------------------------------------------
    // Effect 4: Dissolve reveal — radial mask pattern, 0.8s duration
    // ---------------------------------------------------------------------------
    commands.spawn((
        Name::new("Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(290.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        DissolveEffect::new(DissolveConfig {
            duration_secs: 0.8,
            pattern: DissolvePattern::Mask,
            mask_texture: Some(assets.mask.clone()),
            ..showcase_reveal_dissolve_config()
        }),
    ));
}
