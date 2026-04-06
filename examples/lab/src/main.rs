#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use saddle_rendering_sprite_effects_example_common as common;

use std::fmt::Write as _;

use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use common::{
    add_demo_assets, animate_atlas_sprites, install_auto_exit, showcase_dissolve_config,
    showcase_grounded_squash_config, showcase_hide_dissolve_config, showcase_screen_flash_config,
    spawn_animated_sprite,
};
use saddle_rendering_sprite_effects::{
    DissolveCompletion, DissolveConfig, DissolveEffect, DissolvePattern, DissolvePhase,
    FlashConfig, FlashEffect, PaletteConfig, PaletteSwap, SpriteEffectsDiagnostics,
    SpriteEffectsPlugin, SpriteEffectsSystems, SquashStretchEffect,
};

const DEFAULT_BRP_PORT: u16 = 15_743;

#[derive(Component)]
pub(crate) struct NativeFlashTarget;

#[derive(Component)]
pub(crate) struct ScreenFlashTarget;

#[derive(Component)]
pub(crate) struct DissolveTarget;

#[derive(Component)]
pub(crate) struct PaletteTarget;

#[derive(Component)]
pub(crate) struct AtlasTarget;

#[derive(Component)]
pub(crate) struct StressTarget;

#[derive(Component)]
pub(crate) struct LabOverlay;

#[derive(Resource, Clone)]
pub(crate) struct LabAssets {
    pub mask: Handle<Image>,
}

#[derive(Resource, Clone, Copy)]
pub(crate) struct LabEntities {
    pub native_flash: Entity,
    pub screen_flash: Entity,
    pub dissolve_target: Entity,
    pub palette_target: Entity,
    pub atlas_target: Entity,
    #[cfg_attr(not(feature = "e2e"), allow(dead_code))]
    pub overlay: Entity,
    pub stress_targets: usize,
}

#[derive(Resource)]
struct AutoShowcase {
    timer: Timer,
    phase: usize,
    palette_index: usize,
    dissolve_reveal: bool,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.045, 0.055, 0.075)));
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "sprite_effects crate-local lab".into(),
                    resolution: (1560, 960).into(),
                    ..default()
                }),
                ..default()
            }),
    );
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_port(lab_brp_port()));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::SpriteEffectsLabE2EPlugin);
    app.add_plugins(SpriteEffectsPlugin::default());
    install_auto_exit(&mut app, "SPRITE_EFFECTS_LAB_EXIT_AFTER_SECONDS");
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        animate_atlas_sprites.before(SpriteEffectsSystems::Prepare),
    );
    if automation_enabled() {
        app.insert_resource(AutoShowcase {
            timer: Timer::from_seconds(0.65, TimerMode::Repeating),
            phase: 0,
            palette_index: 0,
            dissolve_reveal: false,
        });
        app.add_systems(Update, auto_showcase.before(SpriteEffectsSystems::Prepare));
    }
    app.add_systems(
        Update,
        update_overlay.after(SpriteEffectsSystems::Diagnostics),
    );
    app.run();
}

#[cfg(feature = "dev")]
fn lab_brp_port() -> u16 {
    std::env::var("BRP_EXTRAS_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_BRP_PORT)
}

#[cfg(not(feature = "dev"))]
fn lab_brp_port() -> u16 {
    DEFAULT_BRP_PORT
}

fn automation_enabled() -> bool {
    requested_scenario_name().is_none()
}

fn requested_scenario_name() -> Option<String> {
    std::env::args().skip(1).find(|arg| !arg.starts_with('-'))
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    spawn_backdrop(&mut commands);
    commands.spawn((Name::new("Main Camera"), Camera2d));

    let assets = add_demo_assets(&mut images, &mut atlases);
    commands.insert_resource(LabAssets {
        mask: assets.mask.clone(),
    });

    let native_flash = commands
        .spawn((
            Name::new("Native Flash Target"),
            NativeFlashTarget,
            Sprite::from_image(assets.sprite.clone()),
            Transform::from_xyz(-360.0, 180.0, 0.0).with_scale(Vec3::splat(7.0)),
        ))
        .id();

    let screen_flash = commands
        .spawn((
            Name::new("Screen Flash Target"),
            ScreenFlashTarget,
            Sprite::from_image(assets.sprite.clone()),
            Transform::from_xyz(-145.0, 180.0, 0.0).with_scale(Vec3::splat(7.0)),
        ))
        .id();

    let dissolve_target = commands
        .spawn((
            Name::new("Dissolve Target"),
            DissolveTarget,
            Sprite::from_image(assets.sprite.clone()),
            Transform::from_xyz(80.0, 180.0, 0.0).with_scale(Vec3::splat(7.0)),
        ))
        .id();

    let palette_target = commands
        .spawn((
            Name::new("Palette Target"),
            PaletteTarget,
            Sprite::from_image(assets.sprite.clone()),
            Transform::from_xyz(300.0, 180.0, 0.0).with_scale(Vec3::splat(7.0)),
            PaletteSwap::new(PaletteConfig {
                texture: assets.palette.clone(),
                columns: 4,
                source_row: 0,
                target_row: 1,
                ..PaletteConfig::default()
            }),
        ))
        .id();

    let atlas_target = spawn_animated_sprite(&mut commands, &assets, Vec3::new(-10.0, 15.0, 0.0));
    commands.entity(atlas_target).insert((
        Name::new("Atlas Target"),
        AtlasTarget,
        PaletteSwap::new(PaletteConfig {
            texture: assets.palette.clone(),
            columns: 4,
            source_row: 0,
            target_row: 2,
            ..PaletteConfig::default()
        }),
    ));

    let mut stress_targets = 0usize;
    for y in 0..8 {
        for x in 0..14 {
            let translation = Vec3::new(-470.0 + x as f32 * 72.0, -180.0 + y as f32 * 44.0, 0.0);
            let entity = spawn_animated_sprite(&mut commands, &assets, translation);
            commands.entity(entity).insert((
                Name::new(format!("Stress Sprite {}", stress_targets + 1)),
                StressTarget,
                PaletteSwap::new(PaletteConfig {
                    texture: assets.palette.clone(),
                    columns: 4,
                    source_row: 0,
                    target_row: 1 + (x as u32 % 3),
                    ..PaletteConfig::default()
                }),
            ));
            stress_targets += 1;
        }
    }

    let overlay = commands
        .spawn((
            Name::new("Sprite Effects Lab Overlay"),
            LabOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: px(18.0),
                top: px(18.0),
                width: px(470.0),
                padding: UiRect::all(px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.82)),
            Text::new("Sprite Effects Lab"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ))
        .id();

    commands.insert_resource(LabEntities {
        native_flash,
        screen_flash,
        dissolve_target,
        palette_target,
        atlas_target,
        overlay,
        stress_targets,
    });
}

fn spawn_backdrop(commands: &mut Commands) {
    commands.spawn((
        Name::new("Backdrop"),
        Sprite::from_color(Color::srgb(0.04, 0.05, 0.07), Vec2::new(1900.0, 1300.0)),
        Transform::from_xyz(0.0, 0.0, -40.0),
    ));
    commands.spawn((
        Name::new("Hero Strip"),
        Sprite::from_color(
            Color::srgba(0.10, 0.13, 0.18, 0.95),
            Vec2::new(1500.0, 290.0),
        ),
        Transform::from_xyz(0.0, 170.0, -30.0),
    ));
    commands.spawn((
        Name::new("Atlas Panel"),
        Sprite::from_color(
            Color::srgba(0.08, 0.12, 0.10, 0.92),
            Vec2::new(340.0, 210.0),
        ),
        Transform::from_xyz(-10.0, 15.0, -25.0),
    ));
    commands.spawn((
        Name::new("Stress Panel"),
        Sprite::from_color(
            Color::srgba(0.07, 0.09, 0.13, 0.96),
            Vec2::new(1120.0, 420.0),
        ),
        Transform::from_xyz(0.0, -26.0, -25.0),
    ));
    commands.spawn((
        Name::new("Accent Stripe"),
        Sprite::from_color(
            Color::srgba(0.22, 0.54, 0.48, 0.24),
            Vec2::new(1750.0, 38.0),
        ),
        Transform {
            translation: Vec3::new(0.0, -58.0, -22.0),
            rotation: Quat::from_rotation_z(0.07),
            ..default()
        },
    ));
}

fn auto_showcase(
    time: Res<Time>,
    mut auto: ResMut<AutoShowcase>,
    assets: Res<LabAssets>,
    entities: Res<LabEntities>,
    mut commands: Commands,
    mut palette_targets: Query<&mut PaletteSwap, (With<PaletteTarget>, Without<AtlasTarget>)>,
    mut atlas_targets: Query<&mut PaletteSwap, (With<AtlasTarget>, Without<PaletteTarget>)>,
    stress_targets: Query<Entity, With<StressTarget>>,
) {
    if !auto.timer.tick(time.delta()).just_finished() {
        return;
    }

    commands
        .entity(entities.native_flash)
        .insert(FlashEffect::new(FlashConfig {
            color: Color::srgb(1.0, 0.26, 0.26),
            duration_secs: 0.18,
            ..FlashConfig::default()
        }));
    commands.entity(entities.screen_flash).insert((
        FlashEffect::new(showcase_screen_flash_config()),
        SquashStretchEffect::new(showcase_grounded_squash_config()),
    ));
    commands
        .entity(entities.atlas_target)
        .insert(FlashEffect::new(showcase_screen_flash_config()));

    const PALETTE_ROWS: [u32; 3] = [1, 2, 3];
    auto.palette_index = (auto.palette_index + 1) % PALETTE_ROWS.len();
    if let Ok(mut palette) = palette_targets.get_mut(entities.palette_target) {
        palette.config.target_row = PALETTE_ROWS[auto.palette_index];
    }
    if let Ok(mut palette) = atlas_targets.get_mut(entities.atlas_target) {
        palette.config.target_row = PALETTE_ROWS[(auto.palette_index + 1) % PALETTE_ROWS.len()];
    }

    commands
        .entity(entities.dissolve_target)
        .insert(DissolveEffect::new(DissolveConfig {
            duration_secs: 0.52,
            pattern: DissolvePattern::Mask,
            mask_texture: Some(assets.mask.clone()),
            phase: if auto.dissolve_reveal {
                DissolvePhase::Reveal
            } else {
                DissolvePhase::Hide
            },
            completion: DissolveCompletion::RestoreVisible,
            ..showcase_dissolve_config()
        }));
    auto.dissolve_reveal = !auto.dissolve_reveal;

    for (index, entity) in stress_targets.iter().enumerate() {
        if index % 4 == auto.phase % 4 {
            commands
                .entity(entity)
                .insert(FlashEffect::new(showcase_screen_flash_config()));
        }
        if index % 5 == auto.phase % 5 {
            commands
                .entity(entity)
                .insert(SquashStretchEffect::new(showcase_grounded_squash_config()));
        }
        if index % 7 == auto.phase % 7 {
            commands
                .entity(entity)
                .insert(DissolveEffect::new(DissolveConfig {
                    duration_secs: 0.32,
                    completion: DissolveCompletion::RestoreVisible,
                    ..showcase_hide_dissolve_config()
                }));
        }
    }

    auto.phase = auto.phase.wrapping_add(1);
}

fn update_overlay(
    entities: Res<LabEntities>,
    diagnostics: Res<SpriteEffectsDiagnostics>,
    palette_targets: Query<&PaletteSwap, With<PaletteTarget>>,
    atlas_targets: Query<&Sprite, With<AtlasTarget>>,
    mut overlays: Query<&mut Text, With<LabOverlay>>,
    auto: Option<Res<AutoShowcase>>,
) {
    let Ok(mut text) = overlays.single_mut() else {
        return;
    };

    let palette_row = palette_targets
        .get(entities.palette_target)
        .map(|palette| palette.config.target_row)
        .unwrap_or(0);
    let atlas_frame = atlas_targets
        .get(entities.atlas_target)
        .ok()
        .and_then(|sprite| sprite.texture_atlas.as_ref().map(|atlas| atlas.index))
        .unwrap_or(0);

    let mut body = String::from("Sprite Effects Lab\n");
    let _ = writeln!(
        body,
        "hybrid backend: native flash + squash, proxy shader for screen flash / dissolve / palette"
    );
    let _ = writeln!(
        body,
        "palette row: {palette_row} | atlas frame: {atlas_frame} | stress sprites: {}",
        entities.stress_targets
    );
    let _ = writeln!(
        body,
        "active flashes: {} | dissolves: {} | squashes: {}",
        diagnostics.active_flashes, diagnostics.active_dissolves, diagnostics.active_squashes
    );
    let _ = writeln!(
        body,
        "palette swaps: {} | shader proxies: {}",
        diagnostics.active_palette_swaps, diagnostics.active_shader_proxies
    );
    let _ = write!(
        body,
        "showcase loop: {}",
        if auto.is_some() {
            "running"
        } else {
            "disabled for scenario-driven verification"
        }
    );

    text.0 = body;
}
