use saddle_rendering_sprite_effects_example_common as common;

use bevy::prelude::*;
use common::{
    add_demo_assets, animate_atlas_sprites, install_auto_exit, setup_camera, spawn_animated_sprite,
};
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, FlashConfig, FlashEffect, PaletteConfig, PaletteSwap,
    SpriteEffectsPlugin, SquashStretchConfig, SquashStretchEffect,
};

#[derive(Resource)]
struct StressPulse(Timer);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.insert_resource(StressPulse(Timer::from_seconds(0.35, TimerMode::Repeating)));
    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_atlas_sprites, pulse_room));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects stress",
        "A dense room of sprites mixes idle palette swaps with bursts of flash, dissolve, and squash to check proxy/material churn.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    for y in 0..8 {
        for x in 0..14 {
            let translation = Vec3::new(-470.0 + x as f32 * 72.0, 210.0 - y as f32 * 60.0, 0.0);
            let entity = spawn_animated_sprite(&mut commands, &assets, translation);
            commands
                .entity(entity)
                .insert(PaletteSwap::new(PaletteConfig {
                    texture: assets.palette.clone(),
                    columns: 4,
                    source_row: 0,
                    target_row: (x as u32 % 3) + 1,
                    ..PaletteConfig::default()
                }));
        }
    }
}

fn pulse_room(
    time: Res<Time>,
    mut pulse: ResMut<StressPulse>,
    mut commands: Commands,
    query: Query<Entity, With<common::DemoAtlasAnimation>>,
) {
    if !pulse.0.tick(time.delta()).just_finished() {
        return;
    }

    for (index, entity) in query.iter().enumerate() {
        if index % 3 == 0 {
            commands
                .entity(entity)
                .insert(FlashEffect::new(FlashConfig::damage()));
        }
        if index % 4 == 0 {
            commands
                .entity(entity)
                .insert(SquashStretchEffect::new(SquashStretchConfig::landing()));
        }
        if index % 6 == 0 {
            commands
                .entity(entity)
                .insert(DissolveEffect::new(DissolveConfig {
                    duration_secs: 0.38,
                    ..DissolveConfig::hide()
                }));
        }
    }
}
