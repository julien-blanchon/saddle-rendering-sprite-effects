use bevy::prelude::*;
use saddle_rendering_sprite_effects::{
    OutlineConfig, OutlineEffect, SilhouetteConfig, SilhouetteEffect, SpriteEffectsPlugin,
};
use saddle_rendering_sprite_effects_example_common as common;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_plugins(SpriteEffectsPlugin::default());
    common::install_pane(&mut app);
    common::install_auto_exit(&mut app, "SPRITE_EFFECTS_EXIT_AFTER_SECONDS");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    common::setup_camera(
        &mut commands,
        "sprite_effects outline_silhouette",
        "Outline and silhouette readability layers on top of the shared proxy path. The center actor sits behind a foreground ruin while its silhouette is promoted above the occluder.",
    );
    let assets = common::add_demo_assets(&mut images, &mut atlases);

    commands.spawn((
        Name::new("Moon Glow"),
        Sprite::from_color(Color::srgba(0.48, 0.68, 1.0, 0.12), Vec2::splat(180.0)),
        Transform::from_xyz(320.0, 170.0, -15.0),
    ));

    commands.spawn((
        Name::new("Stone Platform"),
        Sprite::from_color(Color::srgb(0.12, 0.14, 0.16), Vec2::new(760.0, 80.0)),
        Transform::from_xyz(0.0, -170.0, -5.0),
    ));

    commands.spawn((
        Name::new("Foreground Ruin"),
        Sprite::from_color(Color::srgb(0.09, 0.11, 0.13), Vec2::new(190.0, 260.0)),
        Transform::from_xyz(0.0, -10.0, 2.0),
    ));

    commands.spawn((
        Name::new("Outline Actor"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-250.0, -58.0, 0.0).with_scale(Vec3::splat(8.0)),
        OutlineEffect::new(OutlineConfig {
            color: Color::BLACK,
            width_pixels: 1.5,
            alpha_threshold: 0.08,
        }),
    ));

    commands.spawn((
        Name::new("Silhouette Actor"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(0.0, -58.0, 0.0).with_scale(Vec3::splat(8.0)),
        OutlineEffect::new(OutlineConfig {
            color: Color::srgba(0.03, 0.04, 0.05, 1.0),
            width_pixels: 1.0,
            alpha_threshold: 0.08,
        }),
        SilhouetteEffect::new(SilhouetteConfig {
            color: Color::srgba(0.24, 0.86, 1.0, 0.88),
            tint_strength: 0.82,
            alpha_threshold: 0.08,
            sort_offset: 3.25,
        }),
    ));

    commands.spawn((
        Name::new("Boss Actor"),
        Sprite::from_image(assets.sprite),
        Transform::from_xyz(250.0, -58.0, 1.0).with_scale(Vec3::splat(10.0)),
        OutlineEffect::new(OutlineConfig {
            color: Color::srgba(0.98, 0.77, 0.24, 1.0),
            width_pixels: 2.25,
            alpha_threshold: 0.08,
        }),
        SilhouetteEffect::new(SilhouetteConfig {
            color: Color::srgba(0.93, 0.24, 0.32, 0.78),
            tint_strength: 0.48,
            alpha_threshold: 0.08,
            sort_offset: 0.75,
        }),
    ));

    spawn_label(
        &mut commands,
        "Clean outline",
        Vec3::new(-250.0, -175.0, 5.0),
    );
    spawn_label(
        &mut commands,
        "Behind-ruin silhouette",
        Vec3::new(0.0, -175.0, 5.0),
    );
    spawn_label(
        &mut commands,
        "Outline + boss tint",
        Vec3::new(250.0, -175.0, 5.0),
    );
}

fn spawn_label(commands: &mut Commands, text: &str, translation: Vec3) {
    commands.spawn((
        Name::new(format!("Label {text}")),
        Text2d::new(text),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.91, 0.94, 0.98)),
        Transform::from_translation(translation),
    ));
}
