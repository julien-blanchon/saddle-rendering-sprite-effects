//! Dissolve patterns: noise, radial, and authored mask.
//! All loop automatically. Press R to reset.

use bevy::prelude::*;
use common::{
    add_demo_assets, install_auto_exit, reset_all_effects_on_r, setup_camera, spawn_label,
    showcase_hide_dissolve_config, showcase_reveal_dissolve_config,
};
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, DissolvePattern, DissolvePhase, LoopMode, SpriteEffectsPlugin,
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
        "sprite_effects — dissolve",
        "Noise, radial, and authored-mask dissolves. Press R to reset.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    // 1. Noise dissolve (hide) — loops
    commands.spawn((
        Name::new("Noise Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-250.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
        DissolveEffect::new(DissolveConfig {
            pattern: DissolvePattern::Noise,
            duration_secs: 1.0,
            delay_secs: 0.5,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_hide_dissolve_config()
        }),
    ));
    spawn_label(
        &mut commands,
        "Noise Dissolve (hide)\nProcedural hash pattern\nLoops with 0.5s pause",
        -250.0,
        -120.0,
    );

    // 2. Radial dissolve (hide) — loops
    commands.spawn((
        Name::new("Radial Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
        DissolveEffect::new(DissolveConfig {
            pattern: DissolvePattern::RadialOut,
            duration_secs: 1.0,
            delay_secs: 0.5,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_hide_dissolve_config()
        }),
    ));
    spawn_label(
        &mut commands,
        "Radial Dissolve (hide)\nDistance-from-center field\nLoops with 0.5s pause",
        0.0,
        -120.0,
    );

    // 3. Mask dissolve (reveal) — loops
    commands.spawn((
        Name::new("Mask Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(250.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
        DissolveEffect::new(DissolveConfig {
            phase: DissolvePhase::Reveal,
            pattern: DissolvePattern::Mask,
            mask_texture: Some(assets.mask.clone()),
            duration_secs: 1.2,
            delay_secs: 0.4,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_reveal_dissolve_config()
        }),
    ));
    spawn_label(
        &mut commands,
        "Mask Dissolve (reveal)\nAuthored radial mask texture\nLoops with 0.4s pause",
        250.0,
        -120.0,
    );
}
