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
    ColorStop, DissolveConfig, DissolveEffect, EffectTimeDomain, FlashBlendMode, FlashConfig,
    FlashEffect, LoopMode, OutlineConfig, OutlineEffect, OverlapPolicy, PaletteConfig,
    PaletteSwap, ShakeConfig, ShakeEffect, SilhouetteConfig, SilhouetteEffect,
    SpriteEffectFinished, SpriteEffectKind, SpriteEffectStarted, SpriteEffectsDiagnostics,
    SpriteEffectsPlugin, SquashStretchConfig, SquashStretchEffect,
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

fn add_image(app: &mut App) -> Handle<Image> {
    app.world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default())
}

// ---------------------------------------------------------------------------
// Existing tests (updated for new API)
// ---------------------------------------------------------------------------

#[test]
fn flash_component_cleans_itself_up_after_duration() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((Sprite::from_image(image), FlashEffect::default()))
        .id();

    for _ in 0..20 {
        advance(&mut app);
    }

    assert!(app.world().get::<FlashEffect>(entity).is_none());
    assert!(app.world().get::<crate::systems::FlashRuntime>(entity).is_none());
}

#[test]
fn flash_completion_emits_finished_message() {
    let mut app = init_app();
    let image = add_image(&mut app);
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
    let image = add_image(&mut app);
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
fn dissolve_completion_can_despawn_entity() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let mut config = DissolveConfig::hide();
    config.duration_secs = 0.05;
    config.completion = crate::DissolveCompletion::DespawnEntity;

    let entity = app
        .world_mut()
        .spawn((Sprite::from_image(image), DissolveEffect::new(config)))
        .id();

    for _ in 0..8 {
        advance(&mut app);
    }

    assert!(app.world().get_entity(entity).is_err());
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

    assert!(app.world().get::<crate::systems::ShaderProxy>(entity).is_some());
    assert_eq!(
        app.world().resource::<SpriteEffectsDiagnostics>().active_palette_swaps,
        1
    );
}

#[test]
fn palette_swap_without_texture_keeps_native_path() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((Sprite::from_image(image), PaletteSwap::default()))
        .id();

    advance(&mut app);

    assert!(
        app.world().get::<crate::systems::ShaderProxy>(entity).is_none(),
        "palette swap without a concrete texture should not create a proxy"
    );
}

#[test]
fn outline_effect_creates_proxy_and_populates_outline_uniform() {
    let mut app = init_app();
    let image = add_image(&mut app);
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
    let materials = app.world().resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials.get(&proxy.material).expect("proxy material should exist");

    assert_eq!(material.uniform.outline.x, 2.5);
    assert_eq!(material.uniform.outline.y, 0.12);
    assert_eq!(material.uniform.outline.z, 1.0);
}

#[test]
fn silhouette_effect_updates_proxy_depth_and_uniforms() {
    let mut app = init_app();
    let image = add_image(&mut app);
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
    let materials = app.world().resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials.get(&proxy.material).expect("proxy material should exist");

    assert_eq!(material.uniform.silhouette.y, 0.75);
    assert_eq!(material.uniform.silhouette.z, 1.0);
    assert_eq!(
        app.world().get::<Transform>(proxy.child).unwrap().translation.z,
        0.6
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

    let proxy = app.world().get::<crate::systems::ShaderProxy>(entity).unwrap();
    let materials = app.world().resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials.get(&proxy.material).unwrap();

    assert_eq!(material.uniform.flags.w, 0.0);
}

#[test]
fn unscaled_effects_advance_while_virtual_time_is_paused() {
    let mut app = init_app();
    let image = add_image(&mut app);
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

    let proxy = app.world().get::<crate::systems::ShaderProxy>(entity).unwrap();
    let materials = app.world().resource::<Assets<crate::material::SpriteEffectsMaterial>>();
    let material = materials.get(&proxy.material).unwrap();

    assert_eq!(material.uniform.base_color.w, 0.75);
    assert_eq!(app.world().get::<Sprite>(entity).unwrap().color.alpha(), 0.0);
}

#[test]
fn squash_effect_restores_transform_after_cleanup() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            Transform::from_scale(Vec3::new(2.0, 2.0, 1.0)),
            SquashStretchEffect::default(),
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
    assert_eq!(app.world().get::<Transform>(entity).unwrap().scale, base_scale);
}

#[test]
fn disabling_effect_cleans_runtime_without_stripping_component() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((Sprite::from_image(image), FlashEffect::default()))
        .id();

    advance(&mut app);
    assert!(app.world().get::<crate::systems::FlashRuntime>(entity).is_some());

    app.world_mut().get_mut::<FlashEffect>(entity).unwrap().enabled = false;
    advance(&mut app);

    assert!(!app.world().get::<FlashEffect>(entity).unwrap().enabled);
    assert!(app.world().get::<crate::systems::FlashRuntime>(entity).is_none());
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

    assert!(app.world().get::<crate::systems::ShaderProxy>(entity).is_some());
    assert_eq!(app.world().get::<Sprite>(entity).unwrap().color.alpha(), 0.0);

    app.world_mut().run_schedule(Deactivate);

    assert!(app.world().get::<PaletteSwap>(entity).is_some());
    assert!(app.world().get::<crate::systems::ShaderProxy>(entity).is_none());
    assert_eq!(
        app.world().get::<Sprite>(entity).unwrap().color,
        Color::srgba(0.35, 0.55, 0.85, 0.8)
    );
}

// ---------------------------------------------------------------------------
// New feature tests
// ---------------------------------------------------------------------------

#[test]
fn delay_secs_delays_flash_start() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                delay_secs: 0.10,
                ..FlashConfig::default()
            }),
        ))
        .id();

    // At 16ms per frame, 6 frames = 96ms < 100ms delay. Effect should still be active.
    for _ in 0..6 {
        advance(&mut app);
    }
    assert!(app.world().get::<FlashEffect>(entity).is_some());

    // After delay + duration (~150ms total), should finish.
    for _ in 0..6 {
        advance(&mut app);
    }
    assert!(app.world().get::<FlashEffect>(entity).is_none());
}

#[test]
fn started_message_emits_after_delay() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let _entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.10,
                delay_secs: 0.05,
                ..FlashConfig::default()
            }),
        ))
        .id();

    let mut cursor = MessageCursor::<SpriteEffectStarted>::default();

    // First frame (16ms): 16ms < 50ms delay, so no started message yet.
    advance(&mut app);
    let msgs = read_messages(&app, &mut cursor);
    assert!(msgs.is_empty(), "should not emit started before delay");

    // Advance until past the 50ms delay.
    let mut found = false;
    for _ in 0..10 {
        advance(&mut app);
        let msgs = read_messages(&app, &mut cursor);
        if !msgs.is_empty() {
            assert_eq!(msgs[0].effect, SpriteEffectKind::Flash);
            found = true;
            break;
        }
    }
    assert!(found, "started message should have been emitted after delay");
}

#[test]
fn persistent_flash_stays_after_completion() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                persistent: true,
                ..FlashConfig::default()
            }),
        ))
        .id();

    for _ in 0..8 {
        advance(&mut app);
    }

    // Component should still exist (persistent mode).
    let effect = app.world().get::<FlashEffect>(entity);
    assert!(effect.is_some(), "persistent flash should not be removed");
    // Runtime should be cleaned up.
    assert!(app.world().get::<crate::systems::FlashRuntime>(entity).is_none());
}

#[test]
fn retrigger_restarts_persistent_effect() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                persistent: true,
                ..FlashConfig::default()
            }),
        ))
        .id();

    // Let it finish.
    for _ in 0..8 {
        advance(&mut app);
    }
    assert!(app.world().get::<crate::systems::FlashRuntime>(entity).is_none());

    // Retrigger.
    app.world_mut().get_mut::<FlashEffect>(entity).unwrap().retrigger();
    advance(&mut app);

    // Runtime should be recreated.
    assert!(app.world().get::<crate::systems::FlashRuntime>(entity).is_some());

    let mut finished_cursor = MessageCursor::<SpriteEffectFinished>::default();
    let mut got_finished = false;
    for _ in 0..10 {
        advance(&mut app);
        let msgs = read_messages(&app, &mut finished_cursor);
        if msgs.iter().any(|m| m.effect == SpriteEffectKind::Flash) {
            got_finished = true;
            break;
        }
    }
    assert!(got_finished, "retrigger should emit a second finished message");
}

#[test]
fn loop_mode_count_replays_effect() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                loop_mode: LoopMode::Count(3),
                ..FlashConfig::default()
            }),
        ))
        .id();

    // 3 loops × 0.05s = 0.15s. At 16ms/frame, ~10 frames should be enough.
    for _ in 0..15 {
        advance(&mut app);
    }

    assert!(
        app.world().get::<FlashEffect>(entity).is_none(),
        "flash should be removed after 3 loops"
    );
}

#[test]
fn loop_mode_forever_keeps_running() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                loop_mode: LoopMode::Forever,
                ..FlashConfig::default()
            }),
        ))
        .id();

    // Run many frames — should never finish.
    for _ in 0..100 {
        advance(&mut app);
    }

    assert!(
        app.world().get::<FlashEffect>(entity).is_some(),
        "forever loop should never finish"
    );
}

#[test]
fn shake_effect_displaces_transform_and_restores() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            Transform::from_translation(Vec3::new(100.0, 200.0, 0.0)),
            ShakeEffect::new(ShakeConfig {
                amplitude: 10.0,
                duration_secs: 0.10,
                ..ShakeConfig::default()
            }),
        ))
        .id();

    let original = app.world().get::<Transform>(entity).unwrap().translation;
    advance(&mut app);
    advance(&mut app);

    let displaced = app.world().get::<Transform>(entity).unwrap().translation;
    // Shake should have moved the entity (at least slightly).
    assert_ne!(displaced, original, "shake should displace the transform");

    // Wait for completion.
    for _ in 0..20 {
        advance(&mut app);
    }

    assert!(app.world().get::<ShakeEffect>(entity).is_none());
    let restored = app.world().get::<Transform>(entity).unwrap().translation;
    assert_eq!(restored, original, "transform should be restored after shake");
}

#[test]
fn shake_diagnostics_count() {
    let mut app = init_app();
    let image = add_image(&mut app);
    app.world_mut().spawn((
        Sprite::from_image(image),
        ShakeEffect::new(ShakeConfig {
            duration_secs: 1.0,
            ..ShakeConfig::default()
        }),
    ));

    advance(&mut app);

    assert_eq!(
        app.world().resource::<SpriteEffectsDiagnostics>().active_shakes,
        1
    );
}

#[test]
fn overlap_policy_ignore_does_not_reset() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.10,
                overlap: OverlapPolicy::Ignore,
                ..FlashConfig::default()
            }),
        ))
        .id();

    // Advance a few frames.
    for _ in 0..3 {
        advance(&mut app);
    }

    let elapsed_before = app
        .world()
        .get::<crate::systems::FlashRuntime>(entity)
        .unwrap()
        .elapsed_secs;

    // Re-insert the same effect (simulating overlap).
    app.world_mut().get_mut::<FlashEffect>(entity).unwrap().config.intensity = 0.5;
    advance(&mut app);

    let elapsed_after = app
        .world()
        .get::<crate::systems::FlashRuntime>(entity)
        .map(|r| r.elapsed_secs);

    // With Ignore policy, the elapsed should continue advancing (not reset to 0).
    assert!(
        elapsed_after.unwrap_or(0.0) > elapsed_before,
        "Ignore policy should not reset elapsed time"
    );
}

#[test]
fn zero_duration_effect_finishes_immediately() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.0,
                ..FlashConfig::default()
            }),
        ))
        .id();

    advance(&mut app);
    advance(&mut app);

    assert!(app.world().get::<FlashEffect>(entity).is_none());
}

#[test]
fn multiple_effects_on_same_entity() {
    let mut app = init_app();
    let (image, palette_tex) = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        (images.add(Image::default()), images.add(Image::default()))
    };
    let entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                blend: FlashBlendMode::Screen,
                duration_secs: 0.10,
                ..FlashConfig::default()
            }),
            PaletteSwap::new(PaletteConfig::new(palette_tex, 4)),
            OutlineEffect::new(OutlineConfig::default()),
        ))
        .id();

    advance(&mut app);

    // All effects should be active.
    let diag = app.world().resource::<SpriteEffectsDiagnostics>();
    assert_eq!(diag.active_flashes, 1);
    assert_eq!(diag.active_palette_swaps, 1);
    assert_eq!(diag.active_outlines, 1);

    // Proxy should exist (screen flash + palette + outline all need it).
    assert!(app.world().get::<crate::systems::ShaderProxy>(entity).is_some());
}

#[test]
fn color_ramp_flash_uses_ramp_color() {
    // This tests that the color ramp logic in math.rs works correctly.
    use crate::math::{flash_color_at, sample_color_ramp};

    let ramp = vec![
        ColorStop::new(0.0, Color::WHITE),
        ColorStop::new(0.5, Color::srgb(1.0, 0.0, 0.0)),
        ColorStop::new(1.0, Color::srgb(0.0, 0.0, 1.0)),
    ];

    // At t=0, should be white.
    let c0 = sample_color_ramp(&ramp, 0.0);
    assert_eq!(c0, Color::WHITE);

    // At t=1, should be blue.
    let c1 = sample_color_ramp(&ramp, 1.0);
    assert_eq!(c1, Color::srgb(0.0, 0.0, 1.0));

    // flash_color_at with ramp should use ramp.
    let config = FlashConfig {
        color_ramp: Some(ramp),
        duration_secs: 1.0,
        ..FlashConfig::default()
    };
    let c = flash_color_at(&config, 0.0);
    assert_eq!(c, Color::WHITE);
}

#[test]
fn flash_started_and_finished_messages_both_emit() {
    let mut app = init_app();
    let image = add_image(&mut app);
    let _entity = app
        .world_mut()
        .spawn((
            Sprite::from_image(image),
            FlashEffect::new(FlashConfig {
                duration_secs: 0.05,
                ..FlashConfig::default()
            }),
        ))
        .id();

    let mut started_cursor = MessageCursor::<SpriteEffectStarted>::default();
    let mut finished_cursor = MessageCursor::<SpriteEffectFinished>::default();

    let mut got_started = false;
    let mut got_finished = false;
    for _ in 0..10 {
        advance(&mut app);
        if !read_messages(&app, &mut started_cursor).is_empty() {
            got_started = true;
        }
        if !read_messages(&app, &mut finished_cursor).is_empty() {
            got_finished = true;
        }
    }

    assert!(got_started, "should emit started message");
    assert!(got_finished, "should emit finished message");
}
