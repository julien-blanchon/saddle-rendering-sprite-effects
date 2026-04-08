//! Atlas animation compatibility: effects work while atlas frames advance.
//! Press R to retrigger effects.

use bevy::prelude::*;
use common::{
    add_demo_assets, animate_atlas_sprites, install_auto_exit, reset_all_effects_on_r,
    setup_camera, showcase_hide_dissolve_config, showcase_screen_flash_config,
    spawn_animated_sprite, spawn_label,
};
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, DissolvePattern, FlashEffect, LoopMode, PaletteConfig,
    PaletteSwap, SpriteEffectsPlugin,
};
use saddle_rendering_sprite_effects_example_common as common;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_atlas_sprites, reset_all_effects_on_r));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects — atlas animation",
        "Atlas frame index advances while shader proxy effects stay active. Press R to reset.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    // Single animated sprite with palette + looping dissolve + looping flash
    let entity = spawn_animated_sprite(&mut commands, &assets, Vec3::new(0.0, 20.0, 0.0));
    commands.entity(entity).insert((
        PaletteSwap::new(PaletteConfig {
            texture: assets.palette.clone(),
            columns: 4,
            source_row: 0,
            target_row: 2,
            ..PaletteConfig::default()
        }),
        DissolveEffect::new(DissolveConfig {
            pattern: DissolvePattern::Noise,
            duration_secs: 1.0,
            delay_secs: 0.5,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_hide_dissolve_config()
        }),
        FlashEffect::new(saddle_rendering_sprite_effects::FlashConfig {
            duration_secs: 0.2,
            delay_secs: 1.8,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..showcase_screen_flash_config()
        }),
    ));

    spawn_label(
        &mut commands,
        "Animated atlas sprite (4 frames)\n+ Blue palette swap (persistent)\n+ Noise dissolve (1s, loops)\n+ Screen flash (0.2s, loops with 1.8s delay)",
        0.0,
        -100.0,
    );
}
