use bevy::time::Virtual;
use bevy::{
    ecs::{
        message::{MessageCursor, Messages},
        schedule::ScheduleLabel,
    },
    prelude::*,
};
use std::time::Duration;

use crate::{
    DissolveConfig, DissolveEffect, EffectTimeDomain, FlashConfig, FlashEffect, OutlineConfig,
    OutlineEffect, PaletteConfig, PaletteSwap, SilhouetteConfig, SilhouetteEffect,
    SpriteEffectFinished, SpriteEffectKind, SpriteEffectsDiagnostics, SpriteEffectsPlugin,
    SquashStretchConfig, SquashStretchEffect,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn init_app_with_plugin(plugin: SpriteEffectsPlugin) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_millis(16),
    ));
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<TextureAtlasLayout>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    app.add_plugins(plugin);
    app
}

fn init_app() -> App {
    init_app_with_plugin(SpriteEffectsPlugin::always_on(Update))
}

fn init_scheduled_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_millis(16),
    ));
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<TextureAtlasLayout>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    app.add_plugins(SpriteEffectsPlugin::new(Activate, Deactivate, Tick));
    app
}

fn advance(app: &mut App) {
    app.update();
}

fn read_messages<T: Message + Clone>(app: &App, cursor: &mut MessageCursor<T>) -> Vec<T> {
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

#[test]
fn flash_component_cleans_itself_up_after_duration() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig::default()),
        ))
        .id();

    for _ in 0..20 {
        advance(&mut app);
    }

    assert!(app.world().get::<FlashEffect>(entity).is_none());
    assert!(
        app.world()
            .get::<crate::systems::FlashRuntime>(entity)
            .is_none()
    );
}

#[test]
fn flash_completion_emits_finished_message() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                ..FlashConfig::default()
            }),
        ))
        .id();

    let mut cursor = MessageCursor::<SpriteEffectFinished>::default();
    assert!(read_messages(&app, &mut cursor).is_empty());

    for _ in 0..8 {
        advance(&mut app);
        if app.world().get::<FlashEffect>(entity).is_none() {
            break;
        }
    }

    let messages = read_messages(&app, &mut cursor);
    assert_eq!(
        messages,
        vec![SpriteEffectFinished {
            entity,
            effect: SpriteEffectKind::Flash,
        }]
    );
}

#[test]
fn dissolve_completion_can_hide_entity() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let mut config = DissolveConfig::hide();
    config.duration_secs = 0.05;
    config.completion = crate::DissolveCompletion::HideEntity;

    let entity = app
        .world_mut()
        .spawn((Sprite::from_image(image), DissolveEffect::new(config)))
        .id();

    for _ in 0..8 {
        advance(&mut app);
    }

    assert_eq!(
        app.world().get::<Visibility>(entity),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn palette_swap_updates_diagnostics_and_creates_proxy() {
    let mut app = init_app();
    let (image, palette) = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        (images.add(Image::default()), images.add(Image::default()))
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            PaletteSwap::new(PaletteConfig::new(palette, 4)),
        ))
        .id();

    advance(&mut app);

    assert!(
        app.world()
            .get::<crate::systems::ShaderProxy>(entity)
            .is_some()
    );
    assert_eq!(
        app.world()
            .resource::<SpriteEffectsDiagnostics>()
            .active_palette_swaps,
        1
    );
}

#[test]
fn palette_swap_without_texture_keeps_native_path() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let entity = app
        .world_mut()
        .spawn((Sprite::from_image(image), PaletteSwap::default()))
        .id();

    advance(&mut app);

    assert!(
        app.world()
            .get::<crate::systems::ShaderProxy>(entity)
            .is_none(),
        "palette swap without a concrete texture should not create a proxy"
    );
    assert_eq!(
        app.world()
            .resource::<SpriteEffectsDiagnostics>()
            .active_palette_swaps,
        1
    );
}

#[test]
fn outline_effect_creates_proxy_and_populates_outline_uniform() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            OutlineEffect::new(OutlineConfig {
                width_pixels: 2.5,
                alpha_threshold: 0.12,
                color: Color::srgba(0.1, 0.1, 0.1, 0.95),
            }),
        ))
        .id();

    advance(&mut app);

    let proxy = app
        .world()
        .get::<crate::systems::ShaderProxy>(entity)
        .expect("outline should create a shader proxy");
    let materials = app
        .world()
        .resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials
        .get(&proxy.material)
        .expect("proxy material should exist");

    assert_eq!(material.uniform.outline.x, 2.5);
    assert_eq!(material.uniform.outline.y, 0.12);
    assert_eq!(material.uniform.outline.z, 1.0);
    assert_eq!(
        app.world()
            .resource::<SpriteEffectsDiagnostics>()
            .active_outlines,
        1
    );
}

#[test]
fn silhouette_effect_updates_proxy_depth_and_uniforms() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            SilhouetteEffect::new(SilhouetteConfig {
                tint_strength: 0.75,
                sort_offset: 0.6,
                ..SilhouetteConfig::default()
            }),
        ))
        .id();

    advance(&mut app);

    let proxy = app
        .world()
        .get::<crate::systems::ShaderProxy>(entity)
        .expect("silhouette should create a shader proxy")
        .clone();
    let materials = app
        .world()
        .resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials
        .get(&proxy.material)
        .expect("proxy material should exist");

    assert_eq!(material.uniform.silhouette.y, 0.75);
    assert_eq!(material.uniform.silhouette.z, 1.0);
    assert_eq!(
        app.world()
            .get::<Transform>(proxy.child)
            .expect("proxy child transform should exist")
            .translation
            .z,
        0.6
    );
    assert_eq!(
        app.world()
            .resource::<SpriteEffectsDiagnostics>()
            .active_silhouettes,
        1
    );
}

#[test]
fn palette_swap_respects_preserve_alpha_flag_in_proxy_uniform() {
    let mut app = init_app();
    let (image, palette) = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        (images.add(Image::default()), images.add(Image::default()))
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            PaletteSwap::new(PaletteConfig {
                texture: palette,
                columns: 4,
                preserve_alpha: false,
                ..PaletteConfig::default()
            }),
        ))
        .id();

    advance(&mut app);

    let proxy = app
        .world()
        .get::<crate::systems::ShaderProxy>(entity)
        .expect("proxy should be created");
    let materials = app
        .world()
        .resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials
        .get(&proxy.material)
        .expect("proxy material should exist");

    assert_eq!(material.uniform.flags.w, 0.0);
}

#[test]
fn unscaled_effects_advance_while_virtual_time_is_paused() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let unscaled = app
        .world_mut()
        .spawn((
            Sprite::from_image(image.clone()),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                time_domain: EffectTimeDomain::Unscaled,
                ..FlashConfig::default()
            }),
        ))
        .id();
    let scaled = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                time_domain: EffectTimeDomain::GlobalScaled,
                ..FlashConfig::default()
            }),
        ))
        .id();

    app.world_mut().resource_mut::<Time<Virtual>>().pause();

    for _ in 0..8 {
        advance(&mut app);
    }

    assert!(app.world().get::<FlashEffect>(unscaled).is_none());
    assert!(app.world().get::<FlashEffect>(scaled).is_some());
}

#[test]
fn shader_proxy_keeps_authored_tint_alpha_in_material_uniform() {
    let mut app = init_app();
    let (image, palette) = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        (images.add(Image::default()), images.add(Image::default()))
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite {
                image,
                color: Color::srgba(0.25, 0.50, 1.0, 0.75),
                ..default()
            },
            PaletteSwap::new(PaletteConfig::new(palette, 4)),
        ))
        .id();

    advance(&mut app);

    let proxy = app
        .world()
        .get::<crate::systems::ShaderProxy>(entity)
        .expect("proxy should be created");
    let materials = app
        .world()
        .resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials
        .get(&proxy.material)
        .expect("proxy material should exist");

    assert_eq!(material.uniform.base_color.w, 0.75);
    assert_eq!(
        app.world().get::<Sprite>(entity).unwrap().color.alpha(),
        0.0
    );
}

#[test]
fn squash_effect_restores_transform_after_cleanup() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            Transform::from_scale(Vec3::new(2.0, 2.0, 1.0)),
            SquashStretchEffect::new(SquashStretchConfig::default()),
        ))
        .id();

    let base_scale = app.world().get::<Transform>(entity).unwrap().scale;
    advance(&mut app);
    advance(&mut app);
    let active_scale = app.world().get::<Transform>(entity).unwrap().scale;
    assert_ne!(active_scale, base_scale);

    for _ in 0..30 {
        advance(&mut app);
    }

    assert!(app.world().get::<SquashStretchEffect>(entity).is_none());
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().scale,
        base_scale
    );
}

#[test]
fn disabling_effect_cleans_runtime_without_stripping_component() {
    let mut app = init_app();
    let image = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(Image::default())
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig::default()),
        ))
        .id();

    advance(&mut app);
    assert!(
        app.world()
            .get::<crate::systems::FlashRuntime>(entity)
            .is_some()
    );

    app.world_mut()
        .get_mut::<FlashEffect>(entity)
        .expect("flash effect should exist")
        .enabled = false;
    advance(&mut app);

    let effect = app
        .world()
        .get::<FlashEffect>(entity)
        .expect("disabled authored component should remain");
    assert!(!effect.enabled);
    assert!(
        app.world()
            .get::<crate::systems::FlashRuntime>(entity)
            .is_none()
    );
    assert_eq!(
        app.world().get::<Sprite>(entity).unwrap().color,
        Color::WHITE
    );
}

#[test]
fn deactivate_schedule_restores_runtime_state_without_stripping_authored_components() {
    let mut app = init_scheduled_app();
    let (image, palette) = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        (images.add(Image::default()), images.add(Image::default()))
    };

    let entity = app
        .world_mut()
        .spawn((
            Sprite {
                image,
                color: Color::srgba(0.35, 0.55, 0.85, 0.8),
                ..default()
            },
            PaletteSwap::new(PaletteConfig::new(palette, 4)),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    assert!(
        app.world()
            .get::<crate::systems::ShaderProxy>(entity)
            .is_some()
    );
    assert_eq!(
        app.world().get::<Sprite>(entity).unwrap().color.alpha(),
        0.0
    );

    app.world_mut().run_schedule(Deactivate);

    assert!(
        app.world().get::<PaletteSwap>(entity).is_some(),
        "authored palette state should remain"
    );
    assert!(
        app.world()
            .get::<crate::systems::ShaderProxy>(entity)
            .is_none(),
        "runtime proxy should be removed"
    );
    assert_eq!(
        app.world().get::<Sprite>(entity).unwrap().color,
        Color::srgba(0.35, 0.55, 0.85, 0.8)
    );
    assert!(
        app.world()
            .get::<crate::systems::PresentedSpriteState>(entity)
            .is_none()
    );

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    assert!(
        app.world()
            .get::<crate::systems::ShaderProxy>(entity)
            .is_some(),
        "proxy should rebuild from preserved authored state on reactivation"
    );
}
