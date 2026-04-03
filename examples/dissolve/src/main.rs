use saddle_rendering_sprite_effects_example_common as common;

use bevy::prelude::*;
use common::{add_demo_assets, install_auto_exit, setup_camera};
use saddle_rendering_sprite_effects::{DissolveConfig, DissolveEffect, DissolvePattern, SpriteEffectsPlugin};

#[derive(Resource)]
struct DissolveCycle(Timer);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.insert_resource(DissolveCycle(Timer::from_seconds(
        0.8,
        TimerMode::Repeating,
    )));
    app.add_systems(Startup, setup);
    app.add_systems(Update, trigger_dissolves);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects dissolve",
        "Noise, radial, and authored-mask dissolves share the same material-backed proxy path.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    commands.spawn((
        Name::new("Noise Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-220.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
    ));
    commands.spawn((
        Name::new("Radial Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
    ));
    commands.spawn((
        Name::new("Mask Dissolve"),
        Sprite::from_image(assets.sprite),
        Transform::from_xyz(220.0, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
        DissolveEffect::new(DissolveConfig {
            phase: saddle_rendering_sprite_effects::DissolvePhase::Reveal,
            pattern: DissolvePattern::Mask,
            mask_texture: Some(assets.mask),
            duration_secs: 0.6,
            ..DissolveConfig::default()
        }),
    ));
}

fn trigger_dissolves(
    time: Res<Time>,
    mut cycle: ResMut<DissolveCycle>,
    mut commands: Commands,
    query: Query<(Entity, &Name)>,
) {
    if !cycle.0.tick(time.delta()).just_finished() {
        return;
    }

    for (entity, name) in &query {
        let effect = match name.as_str() {
            "Noise Dissolve" => Some(DissolveEffect::new(DissolveConfig {
                pattern: DissolvePattern::Noise,
                duration_secs: 0.55,
                ..DissolveConfig::hide()
            })),
            "Radial Dissolve" => Some(DissolveEffect::new(DissolveConfig {
                pattern: DissolvePattern::RadialOut,
                duration_secs: 0.55,
                ..DissolveConfig::hide()
            })),
            _ => None,
        };

        if let Some(effect) = effect {
            commands.entity(entity).insert(effect);
        }
    }
}
