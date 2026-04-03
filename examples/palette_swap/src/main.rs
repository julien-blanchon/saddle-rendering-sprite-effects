use saddle_rendering_sprite_effects_example_common as common;

use bevy::prelude::*;
use common::{PaletteCycle, add_demo_assets, cycle_palette_rows, install_auto_exit, setup_camera};
use saddle_rendering_sprite_effects::{PaletteConfig, PaletteSwap, SpriteEffectsPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.insert_resource(PaletteCycle {
        rows: vec![1, 2, 3],
        timer: Timer::from_seconds(0.55, TimerMode::Repeating),
        index: 0,
    });
    app.add_systems(Startup, setup);
    app.add_systems(Update, cycle_palette_rows);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects palette_swap",
        "The palette texture stores a source row and multiple runtime-selectable target rows for team colors and status variants.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);

    for (index, x) in [-220.0, 0.0, 220.0].into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("Palette Sprite {index}")),
            Sprite::from_image(assets.sprite.clone()),
            Transform::from_xyz(x, 0.0, 0.0).with_scale(Vec3::splat(8.0)),
            PaletteSwap::new(PaletteConfig {
                texture: assets.palette.clone(),
                columns: 4,
                source_row: 0,
                target_row: (index + 1) as u32,
                ..PaletteConfig::default()
            }),
        ));
    }
}
