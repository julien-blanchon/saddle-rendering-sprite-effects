use saddle_rendering_sprite_effects_example_common as common;

use bevy::prelude::*;
use common::{add_demo_assets, install_auto_exit, setup_camera, spawn_showcase_row};
use saddle_rendering_sprite_effects::SpriteEffectsPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    setup_camera(
        &mut commands,
        "sprite_effects basic",
        "Hybrid sprite feedback: native tint/squash plus shader-backed dissolve and palette swap.",
    );
    let assets = add_demo_assets(&mut images, &mut atlases);
    spawn_showcase_row(&mut commands, &assets);
}
