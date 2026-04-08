//! Basic showcase — one sprite per effect type, all looping.
//! Press R to reset all effects.

use bevy::prelude::*;
use common::{
    add_demo_assets, install_auto_exit, reset_all_effects_on_r, setup_camera, spawn_label,
    showcase_reveal_dissolve_config, showcase_screen_flash_config,
};
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, DissolvePattern, FlashBlendMode, FlashConfig, FlashEffect,
    LoopMode, PaletteConfig, PaletteSwap, SpriteEffectsPlugin,
};
use saddle_rendering_sprite_effects_example_common as common;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.add_systems(Startup, setup);
    app.add_systems(Update, reset_all_effects_on_r);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects — basic",
        "Four effect types side by side. Press R to reset.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    let y = 40.0;
    let label_y = -55.0;

    // 1. Tint Flash — looping red tint
    commands.spawn((
        Name::new("Tint Flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-300.0, y, 0.0).with_scale(Vec3::splat(6.0)),
        FlashEffect::new(FlashConfig {
            color: Color::srgb(1.0, 0.26, 0.26),
            intensity: 1.0,
            duration_secs: 0.3,
            delay_secs: 0.5,
            blend: FlashBlendMode::Tint,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..FlashConfig::default()
        }),
    ));
    spawn_label(&mut commands, "Tint Flash\n(native path, loops)", -300.0, label_y);

    // 2. Screen Flash — looping additive white
    commands.spawn((
        Name::new("Screen Flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-100.0, y, 0.0).with_scale(Vec3::splat(6.0)),
        FlashEffect::new(FlashConfig {
            duration_secs: 0.25,
            delay_secs: 0.6,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_screen_flash_config()
        }),
    ));
    spawn_label(&mut commands, "Screen Flash\n(shader proxy, loops)", -100.0, label_y);

    // 3. Palette Swap — static recolor
    commands.spawn((
        Name::new("Palette Swap"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(100.0, y, 0.0).with_scale(Vec3::splat(6.0)),
        PaletteSwap::new(PaletteConfig {
            texture: assets.palette.clone(),
            columns: 4,
            source_row: 0,
            target_row: 1,
            ..PaletteConfig::default()
        }),
    ));
    spawn_label(&mut commands, "Palette Swap\n(persistent recolor)", 100.0, label_y);

    // 4. Dissolve Reveal — looping radial mask
    commands.spawn((
        Name::new("Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(300.0, y, 0.0).with_scale(Vec3::splat(6.0)),
        DissolveEffect::new(DissolveConfig {
            duration_secs: 1.5,
            delay_secs: 0.3,
            pattern: DissolvePattern::Mask,
            mask_texture: Some(assets.mask.clone()),
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_reveal_dissolve_config()
        }),
    ));
    spawn_label(&mut commands, "Dissolve Reveal\n(radial mask, loops)", 300.0, label_y);
}
