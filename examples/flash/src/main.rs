//! Flash effect comparison: native tint vs shader-backed screen flash.
//! Both loop automatically. Press R to reset.

use bevy::prelude::*;
use common::{
    add_demo_assets, install_auto_exit, reset_all_effects_on_r, setup_camera, spawn_label,
    showcase_grounded_squash_config, showcase_screen_flash_config,
};
use saddle_rendering_sprite_effects::{
    FlashBlendMode, FlashConfig, FlashEffect, LoopMode, SpriteEffectsPlugin,
    SquashStretchConfig, SquashStretchEffect,
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
        "sprite_effects — flash",
        "Native tint (left) vs screen flash + squash (right). Press R to reset.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    // Left: Native tint flash — cheap path, no shader proxy
    commands.spawn((
        Name::new("Native Tint Flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-180.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
        FlashEffect::new(FlashConfig {
            color: Color::srgb(1.0, 0.25, 0.25),
            blend: FlashBlendMode::Tint,
            duration_secs: 0.25,
            delay_secs: 0.6,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..FlashConfig::default()
        }),
    ));
    spawn_label(
        &mut commands,
        "Native Tint Flash\n(Sprite.color mutation, no proxy)\nLoops with 0.6s pause",
        -180.0,
        -110.0,
    );

    // Right: Screen flash + squash — shader proxy path
    commands.spawn((
        Name::new("Screen Flash + Squash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(180.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
        FlashEffect::new(FlashConfig {
            duration_secs: 0.2,
            delay_secs: 0.7,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_screen_flash_config()
        }),
        SquashStretchEffect::new(SquashStretchConfig {
            duration_secs: 0.25,
            delay_secs: 0.7,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_grounded_squash_config()
        }),
    ));
    spawn_label(
        &mut commands,
        "Screen Flash + Squash\n(additive blend + transform deform)\nLoops with 0.7s pause",
        180.0,
        -110.0,
    );
}
