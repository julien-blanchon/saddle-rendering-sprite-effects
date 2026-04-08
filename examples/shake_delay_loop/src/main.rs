//! New features showcase: shake, delay/stagger, looping, color ramp, edge gradient.
//! Press R to reset all effects. Press Space to retrigger persistent shake.

use bevy::prelude::*;
use common::{
    add_demo_assets, reset_all_effects_on_r, setup_camera, showcase_color_ramp_flash_config,
    showcase_fire_dissolve_config, showcase_looping_squash_config, spawn_label,
};
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, FlashConfig, FlashEffect, LoopMode, ShakeConfig, ShakeEffect,
    SpriteEffectsPlugin, SquashStretchEffect,
};
use saddle_rendering_sprite_effects_example_common as common;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(SpriteEffectsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (reset_all_effects_on_r, retrigger_persistent_on_space))
        .run();
}

#[derive(Component)]
struct PersistentShakeTarget;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let assets = add_demo_assets(&mut images, &mut atlases);

    setup_camera(
        &mut commands,
        "sprite_effects — shake, delay, loop",
        "New features. Press R to reset all, Space to retrigger persistent shake.",
    );

    let row1_y = 120.0;
    let row2_y = -30.0;
    let row3_y = -190.0;
    let label_offset = -65.0;

    // === Row 1: Shake variations ===
    commands.spawn((
        Name::new("Shake basic"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-340.0, row1_y, 0.0).with_scale(Vec3::splat(5.0)),
        ShakeEffect::new(ShakeConfig {
            amplitude: 8.0,
            frequency: 25.0,
            decay: 0.85,
            duration_secs: 0.5,
            delay_secs: 0.3,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..ShakeConfig::default()
        }),
    ));
    spawn_label(&mut commands, "Shake (2D)\n8px, 25Hz, decay 0.85\nLoops forever", -340.0, row1_y + label_offset);

    commands.spawn((
        Name::new("Shake horizontal"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-120.0, row1_y, 0.0).with_scale(Vec3::splat(5.0)),
        ShakeEffect::new(ShakeConfig {
            amplitude: 6.0,
            axis: Vec2::new(1.0, 0.0),
            duration_secs: 0.5,
            delay_secs: 0.5,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..ShakeConfig::default()
        }),
    ));
    spawn_label(&mut commands, "Shake (horizontal)\naxis=(1,0), 6px\nLoops forever", -120.0, row1_y + label_offset);

    commands.spawn((
        Name::new("Shake no decay"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(100.0, row1_y, 0.0).with_scale(Vec3::splat(5.0)),
        ShakeEffect::new(ShakeConfig {
            amplitude: 3.0,
            frequency: 15.0,
            decay: 0.0,
            duration_secs: 0.4,
            loop_mode: LoopMode::Forever,
            persistent: true,
            ..ShakeConfig::default()
        }),
    ));
    spawn_label(&mut commands, "Shake (no decay)\n3px constant wobble\nLoops forever", 100.0, row1_y + label_offset);

    commands.spawn((
        Name::new("Shake persistent"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(320.0, row1_y, 0.0).with_scale(Vec3::splat(5.0)),
        ShakeEffect::new(ShakeConfig {
            amplitude: 12.0,
            duration_secs: 0.4,
            persistent: true,
            ..ShakeConfig::default()
        }),
        PersistentShakeTarget,
    ));
    spawn_label(&mut commands, "Persistent shake\n12px, plays once\nPress SPACE to retrigger", 320.0, row1_y + label_offset);

    // === Row 2: Delay/Stagger ===
    for i in 0..6 {
        let x = -300.0 + i as f32 * 120.0;
        let delay = i as f32 * 0.15;
        commands.spawn((
            Name::new(format!("Stagger {i}")),
            Sprite::from_image(assets.sprite.clone()),
            Transform::from_xyz(x, row2_y, 0.0).with_scale(Vec3::splat(5.0)),
            DissolveEffect::new(DissolveConfig {
                duration_secs: 0.8,
                delay_secs: delay,
                edge_width: 0.08,
                edge_color: Color::srgb(1.0, 0.6, 0.1),
                loop_mode: LoopMode::Forever,
                persistent: true,
                ..DissolveConfig::default()
            }),
        ));
        spawn_label(&mut commands, &format!("delay: {delay:.2}s"), x, row2_y + label_offset);
    }
    spawn_label(&mut commands, "Staggered dissolve — each sprite starts 0.15s later", 0.0, row2_y - 90.0);

    // === Row 3: Looping, color ramp, edge gradient ===
    commands.spawn((
        Name::new("Looping squash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-300.0, row3_y, 0.0).with_scale(Vec3::splat(5.0)),
        SquashStretchEffect::new(showcase_looping_squash_config()),
    ));
    spawn_label(&mut commands, "Looping squash\n(forever)", -300.0, row3_y + label_offset);

    commands.spawn((
        Name::new("Flash 3x loop"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-100.0, row3_y, 0.0).with_scale(Vec3::splat(5.0)),
        FlashEffect::new(FlashConfig {
            duration_secs: 0.2,
            delay_secs: 0.3,
            loop_mode: LoopMode::Count(3),
            persistent: true,
            color: Color::srgb(1.0, 0.3, 0.3),
            ..FlashConfig::default()
        }),
    ));
    spawn_label(&mut commands, "Flash (3 loops)\nthen stops", -100.0, row3_y + label_offset);

    commands.spawn((
        Name::new("Color ramp flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(100.0, row3_y, 0.0).with_scale(Vec3::splat(5.0)),
        FlashEffect::new(FlashConfig {
            loop_mode: LoopMode::Forever,
            delay_secs: 0.5,
            persistent: true,
            ..showcase_color_ramp_flash_config()
        }),
    ));
    spawn_label(&mut commands, "Color ramp flash\nwhite→yellow→red→dark\nLoops forever", 100.0, row3_y + label_offset);

    commands.spawn((
        Name::new("Fire dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(300.0, row3_y, 0.0).with_scale(Vec3::splat(5.0)),
        DissolveEffect::new(DissolveConfig {
            loop_mode: LoopMode::Forever,
            delay_secs: 0.4,
            persistent: true,
            ..showcase_fire_dissolve_config()
        }),
    ));
    spawn_label(&mut commands, "Fire dissolve\nedge gradient:\nyellow→orange→red", 300.0, row3_y + label_offset);
}

fn retrigger_persistent_on_space(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut ShakeEffect, With<PersistentShakeTarget>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for mut shake in &mut query {
            shake.retrigger();
        }
    }
}
