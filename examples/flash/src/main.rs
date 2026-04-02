use saddle_rendering_sprite_effects_example_common as common;

use bevy::prelude::*;
use common::{add_demo_assets, install_auto_exit, setup_camera};
use saddle_rendering_sprite_effects::{
    FlashBlendMode, FlashConfig, FlashEffect, SpriteEffectsPlugin, SquashStretchConfig,
    SquashStretchEffect,
};

#[derive(Resource)]
struct FlashCycle(Timer);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.insert_resource(FlashCycle(Timer::from_seconds(0.55, TimerMode::Repeating)));
    app.add_systems(Startup, setup);
    app.add_systems(Update, cycle_flash);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects flash",
        "Left sprite stays on the cheap native tint path; right sprite uses screen-style flash and squash for impact feedback.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    commands.spawn((
        Name::new("Native Flash Sprite"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-150.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
    ));
    commands.spawn((
        Name::new("Screen Flash Sprite"),
        Sprite::from_image(assets.sprite),
        Transform::from_xyz(150.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
    ));
}

fn cycle_flash(
    time: Res<Time>,
    mut cycle: ResMut<FlashCycle>,
    mut commands: Commands,
    query: Query<(Entity, &Name)>,
) {
    if !cycle.0.tick(time.delta()).just_finished() {
        return;
    }

    for (entity, name) in &query {
        match name.as_str() {
            "Native Flash Sprite" => {
                commands
                    .entity(entity)
                    .insert(FlashEffect::new(FlashConfig {
                        color: Color::srgb(1.0, 0.25, 0.25),
                        blend: FlashBlendMode::Tint,
                        duration_secs: 0.20,
                        ..FlashConfig::default()
                    }));
            }
            "Screen Flash Sprite" => {
                commands.entity(entity).insert((
                    FlashEffect::new(FlashConfig::damage()),
                    SquashStretchEffect::new(SquashStretchConfig::landing()),
                ));
            }
            _ => {}
        }
    }
}
