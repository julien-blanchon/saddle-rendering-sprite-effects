use saddle_rendering_sprite_effects_example_common as common;

use bevy::prelude::*;
use common::{
    add_demo_assets, animate_atlas_sprites, install_auto_exit, setup_camera, spawn_animated_sprite,
};
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, DissolvePattern, FlashConfig, FlashEffect, PaletteConfig,
    PaletteSwap, SpriteEffectsPlugin,
};

#[derive(Resource)]
struct AtlasCycle(Timer);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.insert_resource(AtlasCycle(Timer::from_seconds(0.8, TimerMode::Repeating)));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_atlas_sprites, cycle_effects));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects atlas_animation",
        "The atlas index keeps advancing while palette remap, screen flash, and dissolve use the internal proxy path.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);
    let entity = spawn_animated_sprite(&mut commands, &assets, Vec3::new(0.0, 0.0, 0.0));
    commands
        .entity(entity)
        .insert(PaletteSwap::new(PaletteConfig {
            texture: assets.palette.clone(),
            columns: 4,
            source_row: 0,
            target_row: 2,
            ..PaletteConfig::default()
        }));
    commands
        .entity(entity)
        .insert(DissolveEffect::new(DissolveConfig {
            pattern: DissolvePattern::Noise,
            phase: saddle_rendering_sprite_effects::DissolvePhase::Reveal,
            duration_secs: 0.65,
            ..DissolveConfig::default()
        }));
}

fn cycle_effects(
    time: Res<Time>,
    mut cycle: ResMut<AtlasCycle>,
    mut commands: Commands,
    query: Query<Entity, With<common::DemoAtlasAnimation>>,
) {
    if !cycle.0.tick(time.delta()).just_finished() {
        return;
    }

    for entity in &query {
        commands.entity(entity).insert((
            FlashEffect::new(FlashConfig::damage()),
            DissolveEffect::new(DissolveConfig {
                pattern: DissolvePattern::Noise,
                duration_secs: 0.52,
                ..DissolveConfig::hide()
            }),
        ));
    }
}
