//! Palette swap: three sprites cycle through palette rows.
//! Press R to reset cycle.

use bevy::prelude::*;
use common::{
    PaletteCycle, add_demo_assets, cycle_palette_rows, install_auto_exit, setup_camera, spawn_label,
};
use saddle_rendering_sprite_effects::{PaletteConfig, PaletteSwap, SpriteEffectsPlugin};
use saddle_rendering_sprite_effects_example_common as common;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.insert_resource(PaletteCycle {
        rows: vec![1, 2, 3],
        timer: Timer::from_seconds(1.2, TimerMode::Repeating),
        index: 0,
    });
    app.add_systems(Startup, setup);
    app.add_systems(Update, (cycle_palette_rows, reset_palette_on_r));
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects — palette swap",
        "Three sprites cycle through palette rows every 1.2s. Press R to reset.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);
    let labels = ["Row 0 → 1 (red)", "Row 0 → 2 (blue)", "Row 0 → 3 (purple)"];

    for (i, label) in labels.iter().enumerate() {
        let x = -220.0 + i as f32 * 220.0;
        commands.spawn((
            Name::new(format!("Palette Sprite {i}")),
            Sprite::from_image(assets.sprite.clone()),
            Transform::from_xyz(x, 20.0, 0.0).with_scale(Vec3::splat(8.0)),
            PaletteSwap::new(PaletteConfig {
                texture: assets.palette.clone(),
                columns: 4,
                source_row: 0,
                target_row: (i as u32 + 1).min(3),
                ..PaletteConfig::default()
            }),
        ));
        spawn_label(&mut commands, *label, x, -100.0);
    }

    spawn_label(
        &mut commands,
        "All three cycle: red → blue → purple → red ...\n4x4 palette texture, nearest sampling",
        0.0,
        -150.0,
    );
}

fn reset_palette_on_r(
    keys: Res<ButtonInput<KeyCode>>,
    mut cycle: ResMut<PaletteCycle>,
    mut query: Query<&mut PaletteSwap>,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }
    cycle.index = 0;
    cycle.timer.reset();
    for (i, mut palette) in query.iter_mut().enumerate() {
        palette.config.target_row = (i as u32 + 1).min(3);
    }
}
