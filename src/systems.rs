use bevy::{
    color::LinearRgba,
    image::ImageSampler,
    image::TextureAtlasLayout,
    prelude::*,
    sprite::Anchor,
    sprite_render::MeshMaterial2d,
    time::{Real, Virtual},
};

use crate::{
    SpriteEffectsRuntimeState,
    components::{
        DissolveEffect, FlashEffect, OutlineEffect, PaletteSwap, ShakeEffect, SilhouetteEffect,
        SpriteEffectFinished, SpriteEffectKind, SpriteEffectStarted, SquashStretchEffect,
    },
    config::{DissolveCompletion, DissolvePattern, FlashBlendMode, LoopMode},
    diagnostics::SpriteEffectsDiagnostics,
    material::{SpriteEffectsInternalAssets, SpriteEffectsMaterial, SpriteEffectsUniform},
    math::{
        color_to_vec4, dissolve_threshold, flash_color_at, flash_weight,
        resolve_time_delta, sample_shake, sample_squash, sprite_draw_size, sprite_uv_rect,
    },
};

// ---------------------------------------------------------------------------
// Runtime components (internal)
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Default)]
pub(crate) struct FlashRuntime {
    pub elapsed_secs: f32,
    pub started: bool,
    pub loops_completed: u32,
    pub generation: u32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct DissolveRuntime {
    pub elapsed_secs: f32,
    pub started: bool,
    pub loops_completed: u32,
    pub generation: u32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct SquashRuntime {
    pub elapsed_secs: f32,
    pub started: bool,
    pub loops_completed: u32,
    pub generation: u32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct ShakeRuntime {
    pub elapsed_secs: f32,
    pub started: bool,
    pub loops_completed: u32,
    pub generation: u32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct PresentedSpriteState {
    pub original_color: Color,
    pub active_last_frame: bool,
}

#[derive(Component, Debug, Default)]
pub(crate) struct PresentedTransformState {
    pub original_translation: Vec3,
    pub original_scale: Vec3,
    pub active_last_frame: bool,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct ShaderProxy {
    pub child: Entity,
    pub material: Handle<SpriteEffectsMaterial>,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ShaderProxyChild;

// ---------------------------------------------------------------------------
// Loop helper
// ---------------------------------------------------------------------------

/// Compute position within the current loop iteration.
///
/// Each loop period = `delay` + `duration`. During the first `delay` seconds
/// of each period, `None` is returned (effect is at rest). During the
/// remaining `duration` seconds, `Some(local_elapsed)` is returned.
///
/// Returns `Err(())` when all loop iterations have completed.
fn loop_position(
    raw_elapsed: f32,
    delay: f32,
    duration: f32,
    loop_mode: &LoopMode,
    loops_completed: &mut u32,
) -> Result<Option<f32>, ()> {
    let period = (delay + duration).max(f32::EPSILON);
    if duration <= f32::EPSILON {
        return Err(()); // instant
    }
    let total_loops_done = (raw_elapsed / period).floor() as u32;
    let max_loops = match loop_mode {
        LoopMode::None => 1,
        LoopMode::Count(n) => *n,
        LoopMode::Forever => u32::MAX,
    };
    *loops_completed = total_loops_done.min(max_loops);
    if total_loops_done >= max_loops {
        return Err(()); // all loops finished
    }
    let local_in_period = raw_elapsed - total_loops_done as f32 * period;
    if local_in_period < delay {
        Ok(None) // in the delay/rest portion
    } else {
        Ok(Some(local_in_period - delay))
    }
}

// ---------------------------------------------------------------------------
// Activation / deactivation
// ---------------------------------------------------------------------------

pub(crate) fn activate_runtime(mut state: ResMut<SpriteEffectsRuntimeState>) {
    state.active = true;
}

pub(crate) fn deactivate_runtime(mut state: ResMut<SpriteEffectsRuntimeState>) {
    state.active = false;
}

pub(crate) fn runtime_is_active(state: Res<SpriteEffectsRuntimeState>) -> bool {
    state.active
}

// ---------------------------------------------------------------------------
// Prepare
// ---------------------------------------------------------------------------

pub(crate) fn ensure_internal_mesh(
    mut internal: ResMut<SpriteEffectsInternalAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if internal.quad_mesh == Handle::default() {
        internal.quad_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    }
}

pub(crate) fn restore_presented_sprite_state(
    mut query: Query<(&mut Sprite, &mut PresentedSpriteState)>,
) {
    for (mut sprite, mut state) in &mut query {
        if state.active_last_frame {
            sprite.color = state.original_color;
            state.active_last_frame = false;
        }
    }
}

pub(crate) fn restore_presented_transform_state(
    mut query: Query<(&mut Transform, &mut PresentedTransformState)>,
) {
    for (mut transform, mut state) in &mut query {
        if state.active_last_frame {
            transform.translation = state.original_translation;
            transform.scale = state.original_scale;
            state.active_last_frame = false;
        }
    }
}

pub(crate) fn enforce_palette_samplers(
    mut images: ResMut<Assets<Image>>,
    query: Query<&PaletteSwap, Or<(Added<PaletteSwap>, Changed<PaletteSwap>)>>,
) {
    for palette in &query {
        if !palette.enabled || !palette.config.enforce_nearest_sampling {
            continue;
        }
        if let Some(image) = images.get_mut(&palette.config.texture) {
            image.sampler = ImageSampler::nearest();
        }
    }
}

// ---------------------------------------------------------------------------
// Tick: Flash
// ---------------------------------------------------------------------------

pub(crate) fn tick_flash_effects(
    mut commands: Commands,
    virtual_time: Res<Time<Virtual>>,
    real_time: Res<Time<Real>>,
    mut finished: MessageWriter<SpriteEffectFinished>,
    mut started: MessageWriter<SpriteEffectStarted>,
    mut query: Query<(Entity, Mut<FlashEffect>, Option<&mut FlashRuntime>)>,
) {
    for (entity, mut effect, runtime) in &mut query {
        if !effect.enabled {
            continue;
        }

        let dt = resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);

        if let Some(mut runtime) = runtime {
            let generation_changed = runtime.generation != effect.generation;
            if generation_changed {
                runtime.elapsed_secs = 0.0;
                runtime.started = false;
                runtime.loops_completed = 0;
                runtime.generation = effect.generation;
            }
            runtime.elapsed_secs += dt;

            let pos = loop_position(
                runtime.elapsed_secs,
                effect.config.delay_secs,
                effect.config.duration_secs,
                &effect.config.loop_mode,
                &mut runtime.loops_completed,
            );

            match pos {
                Err(()) => {
                    if effect.config.persistent {
                        effect.enabled = false;
                    }
                    finish_transient::<FlashEffect, FlashRuntime>(
                        &mut commands,
                        entity,
                        effect.config.persistent,
                        SpriteEffectKind::Flash,
                        &mut finished,
                    );
                }
                Ok(Some(_)) if !runtime.started => {
                    runtime.started = true;
                    started.write(SpriteEffectStarted {
                        entity,
                        effect: SpriteEffectKind::Flash,
                    });
                }
                _ => {}
            }
        } else {
            let mut lc = 0u32;
            let pos = loop_position(
                dt,
                effect.config.delay_secs,
                effect.config.duration_secs,
                &effect.config.loop_mode,
                &mut lc,
            );
            let first_started = matches!(pos, Ok(Some(_)));
            commands.entity(entity).insert(FlashRuntime {
                elapsed_secs: dt,
                started: first_started,
                loops_completed: lc,
                generation: effect.generation,
            });
            if first_started {
                started.write(SpriteEffectStarted {
                    entity,
                    effect: SpriteEffectKind::Flash,
                });
            }
        };
    }
}

// ---------------------------------------------------------------------------
// Tick: Dissolve
// ---------------------------------------------------------------------------

pub(crate) fn tick_dissolve_effects(
    mut commands: Commands,
    virtual_time: Res<Time<Virtual>>,
    real_time: Res<Time<Real>>,
    mut finished: MessageWriter<SpriteEffectFinished>,
    mut started: MessageWriter<SpriteEffectStarted>,
    mut query: Query<(Entity, Mut<DissolveEffect>, Option<&mut DissolveRuntime>)>,
) {
    for (entity, mut effect, runtime) in &mut query {
        if !effect.enabled {
            continue;
        }

        let dt = resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);

        if let Some(mut runtime) = runtime {
            let generation_changed = runtime.generation != effect.generation;
            if generation_changed {
                runtime.elapsed_secs = 0.0;
                runtime.started = false;
                runtime.loops_completed = 0;
                runtime.generation = effect.generation;
            }
            runtime.elapsed_secs += dt;

            let pos = loop_position(
                runtime.elapsed_secs,
                effect.config.delay_secs,
                effect.config.duration_secs,
                &effect.config.loop_mode,
                &mut runtime.loops_completed,
            );

            match pos {
                Err(()) => {
                    if effect.config.persistent {
                        effect.enabled = false;
                    }
                    match effect.config.completion {
                        DissolveCompletion::RestoreVisible => {
                            finish_transient::<DissolveEffect, DissolveRuntime>(
                                &mut commands,
                                entity,
                                effect.config.persistent,
                                SpriteEffectKind::Dissolve,
                                &mut finished,
                            );
                        }
                        DissolveCompletion::HideEntity => {
                            commands.entity(entity).insert(Visibility::Hidden);
                            finish_transient::<DissolveEffect, DissolveRuntime>(
                                &mut commands,
                                entity,
                                effect.config.persistent,
                                SpriteEffectKind::Dissolve,
                                &mut finished,
                            );
                        }
                        DissolveCompletion::DespawnEntity => {
                            commands.entity(entity).despawn_related::<Children>();
                            commands.entity(entity).despawn();
                            finished.write(SpriteEffectFinished {
                                entity,
                                effect: SpriteEffectKind::Dissolve,
                            });
                        }
                    }
                }
                Ok(Some(_)) if !runtime.started => {
                    runtime.started = true;
                    started.write(SpriteEffectStarted {
                        entity,
                        effect: SpriteEffectKind::Dissolve,
                    });
                }
                _ => {}
            }
        } else {
            let mut lc = 0u32;
            let pos = loop_position(
                dt,
                effect.config.delay_secs,
                effect.config.duration_secs,
                &effect.config.loop_mode,
                &mut lc,
            );
            let first_started = matches!(pos, Ok(Some(_)));
            commands.entity(entity).insert(DissolveRuntime {
                elapsed_secs: dt,
                started: first_started,
                loops_completed: lc,
                generation: effect.generation,
            });
            if first_started {
                started.write(SpriteEffectStarted {
                    entity,
                    effect: SpriteEffectKind::Dissolve,
                });
            }
        };
    }
}

// ---------------------------------------------------------------------------
// Tick: Squash/Stretch
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn tick_squash_effects(
    mut commands: Commands,
    virtual_time: Res<Time<Virtual>>,
    real_time: Res<Time<Real>>,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    mut finished: MessageWriter<SpriteEffectFinished>,
    mut started: MessageWriter<SpriteEffectStarted>,
    mut query: Query<(
        Entity,
        Mut<SquashStretchEffect>,
        Option<&mut SquashRuntime>,
        &Anchor,
        &Sprite,
        &mut Transform,
        Option<&mut PresentedTransformState>,
    )>,
) {
    for (entity, mut effect, mut runtime, anchor, sprite, mut transform, presented) in &mut query {
        if !effect.enabled {
            continue;
        }

        let dt = resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);

        // Advance or create runtime.
        let elapsed_secs = if let Some(ref mut rt) = runtime {
            let generation_changed = rt.generation != effect.generation;
            if generation_changed {
                rt.elapsed_secs = dt;
                rt.started = false;
                rt.loops_completed = 0;
                rt.generation = effect.generation;
            } else {
                rt.elapsed_secs += dt;
            }
            rt.elapsed_secs
        } else {
            commands.entity(entity).insert(SquashRuntime {
                elapsed_secs: dt,
                started: false,
                loops_completed: 0,
                generation: effect.generation,
            });
            dt
        };

        let mut lc = runtime.as_ref().map_or(0, |rt| rt.loops_completed);
        let pos = loop_position(
            elapsed_secs,
            effect.config.delay_secs,
            effect.config.duration_secs,
            &effect.config.loop_mode,
            &mut lc,
        );

        let local_elapsed = match pos {
            Err(()) => {
                if effect.config.persistent {
                    effect.enabled = false;
                }
                finish_transient::<SquashStretchEffect, SquashRuntime>(
                    &mut commands,
                    entity,
                    effect.config.persistent,
                    SpriteEffectKind::SquashStretch,
                    &mut finished,
                );
                continue;
            }
            Ok(None) => continue,
            Ok(Some(t)) => t,
        };

        // Fire started message on first active frame.
        let was_started = runtime.as_ref().is_some_and(|rt| rt.started);
        if !was_started {
            if let Some(ref mut rt) = runtime {
                rt.started = true;
            }
            started.write(SpriteEffectStarted {
                entity,
                effect: SpriteEffectKind::SquashStretch,
            });
        }

        if local_elapsed <= 0.0 {
            continue;
        }

        let size = sprite_draw_size(sprite, &images, &atlases);
        let sample = sample_squash(&effect.config, local_elapsed, *anchor, size);
        if let Some(mut presented) = presented {
            presented.original_translation = transform.translation;
            presented.original_scale = transform.scale;
            presented.active_last_frame = true;
        } else {
            commands.entity(entity).insert(PresentedTransformState {
                original_translation: transform.translation,
                original_scale: transform.scale,
                active_last_frame: true,
            });
        }

        transform.scale.x *= sample.scale.x;
        transform.scale.y *= sample.scale.y;
        transform.translation.x += sample.translation.x;
        transform.translation.y += sample.translation.y;
    }
}

// ---------------------------------------------------------------------------
// Tick: Shake
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn tick_shake_effects(
    mut commands: Commands,
    virtual_time: Res<Time<Virtual>>,
    real_time: Res<Time<Real>>,
    mut finished: MessageWriter<SpriteEffectFinished>,
    mut started: MessageWriter<SpriteEffectStarted>,
    mut query: Query<(
        Entity,
        Mut<ShakeEffect>,
        Option<&mut ShakeRuntime>,
        &mut Transform,
        Option<&mut PresentedTransformState>,
    )>,
) {
    for (entity, mut effect, mut runtime, mut transform, presented) in &mut query {
        if !effect.enabled {
            continue;
        }

        let dt = resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);

        let elapsed_secs = if let Some(ref mut rt) = runtime {
            let generation_changed = rt.generation != effect.generation;
            if generation_changed {
                rt.elapsed_secs = dt;
                rt.started = false;
                rt.loops_completed = 0;
                rt.generation = effect.generation;
            } else {
                rt.elapsed_secs += dt;
            }
            rt.elapsed_secs
        } else {
            commands.entity(entity).insert(ShakeRuntime {
                elapsed_secs: dt,
                started: false,
                loops_completed: 0,
                generation: effect.generation,
            });
            dt
        };

        let mut lc = runtime.as_ref().map_or(0, |rt| rt.loops_completed);
        let pos = loop_position(
            elapsed_secs,
            effect.config.delay_secs,
            effect.config.duration_secs,
            &effect.config.loop_mode,
            &mut lc,
        );

        let local_elapsed = match pos {
            Err(()) => {
                if effect.config.persistent {
                    effect.enabled = false;
                }
                finish_transient::<ShakeEffect, ShakeRuntime>(
                    &mut commands,
                    entity,
                    effect.config.persistent,
                    SpriteEffectKind::Shake,
                    &mut finished,
                );
                continue;
            }
            Ok(None) => continue,
            Ok(Some(t)) => t,
        };

        let was_started = runtime.as_ref().is_some_and(|rt| rt.started);
        if !was_started {
            if let Some(ref mut rt) = runtime {
                rt.started = true;
            }
            started.write(SpriteEffectStarted {
                entity,
                effect: SpriteEffectKind::Shake,
            });
        }

        if local_elapsed <= 0.0 {
            continue;
        }

        let offset = sample_shake(&effect.config, local_elapsed);

        if let Some(mut presented) = presented {
            presented.original_translation = transform.translation;
            presented.original_scale = transform.scale;
            presented.active_last_frame = true;
        } else {
            commands.entity(entity).insert(PresentedTransformState {
                original_translation: transform.translation,
                original_scale: transform.scale,
                active_last_frame: true,
            });
        }

        transform.translation.x += offset.x;
        transform.translation.y += offset.y;
    }
}

// ---------------------------------------------------------------------------
// Native flash (tint path, no proxy)
// ---------------------------------------------------------------------------

pub(crate) fn apply_native_flash(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &FlashEffect,
        Option<&FlashRuntime>,
        &mut Sprite,
        Option<&mut PresentedSpriteState>,
        Option<&PaletteSwap>,
        Option<&DissolveEffect>,
        Option<&ShaderProxy>,
    )>,
) {
    for (entity, effect, runtime, mut sprite, presented, palette, dissolve, proxy) in &mut query {
        if !effect.enabled
            || effect.config.blend != FlashBlendMode::Tint
            || palette.is_some_and(|p| p.enabled)
            || dissolve.is_some_and(|d| d.enabled)
            || proxy.is_some()
        {
            continue;
        }

        let elapsed_secs = runtime.map_or(0.0, |r| r.elapsed_secs);

        // Compute local elapsed within current loop.
        let mut lc = runtime.map_or(0, |r| r.loops_completed);
        let pos = loop_position(
            elapsed_secs,
            effect.config.delay_secs,
            effect.config.duration_secs,
            &effect.config.loop_mode,
            &mut lc,
        );

        let local_elapsed = match pos {
            Ok(Some(t)) => t,
            Ok(None) => 0.0,
            Err(()) => 0.0,
        };

        let weight = flash_weight(
            effect.config.easing,
            effect.config.duration_secs,
            local_elapsed,
        ) * effect.config.intensity.clamp(0.0, 1.0);

        if let Some(mut presented) = presented {
            presented.original_color = sprite.color;
            presented.active_last_frame = true;
        } else {
            commands.entity(entity).insert(PresentedSpriteState {
                original_color: sprite.color,
                active_last_frame: true,
            });
        }

        let base = sprite.color.to_linear();
        let flash_col = flash_color_at(&effect.config, local_elapsed).to_linear();
        sprite.color = Color::LinearRgba(LinearRgba::new(
            base.red + (flash_col.red - base.red) * weight,
            base.green + (flash_col.green - base.green) * weight,
            base.blue + (flash_col.blue - base.blue) * weight,
            base.alpha,
        ));
    }
}

// ---------------------------------------------------------------------------
// Cleanup
// ---------------------------------------------------------------------------

pub(crate) fn cleanup_disabled_effect_state(
    mut commands: Commands,
    flashes: Query<(Entity, &FlashEffect, Option<&FlashRuntime>)>,
    dissolves: Query<(Entity, &DissolveEffect, Option<&DissolveRuntime>)>,
    squashes: Query<(Entity, &SquashStretchEffect, Option<&SquashRuntime>)>,
    shakes: Query<(Entity, &ShakeEffect, Option<&ShakeRuntime>)>,
) {
    for (entity, effect, runtime) in &flashes {
        if !effect.enabled && runtime.is_some() {
            commands.entity(entity).remove::<FlashRuntime>();
        }
    }
    for (entity, effect, runtime) in &dissolves {
        if !effect.enabled && runtime.is_some() {
            commands.entity(entity).remove::<DissolveRuntime>();
        }
    }
    for (entity, effect, runtime) in &squashes {
        if !effect.enabled && runtime.is_some() {
            commands.entity(entity).remove::<SquashRuntime>();
        }
    }
    for (entity, effect, runtime) in &shakes {
        if !effect.enabled && runtime.is_some() {
            commands.entity(entity).remove::<ShakeRuntime>();
        }
    }
}

// ---------------------------------------------------------------------------
// Shader proxy sync
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(crate) fn sync_shader_proxies(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    internal: Res<SpriteEffectsInternalAssets>,
    mut materials: ResMut<Assets<SpriteEffectsMaterial>>,
    mut owners: Query<
        (
            Entity,
            &mut Sprite,
            &Anchor,
            Option<Ref<FlashEffect>>,
            Option<&FlashRuntime>,
            Option<Ref<DissolveEffect>>,
            Option<&DissolveRuntime>,
            Option<Ref<PaletteSwap>>,
            Option<Ref<OutlineEffect>>,
            Option<Ref<SilhouetteEffect>>,
            Option<&ShaderProxy>,
            Option<&mut PresentedSpriteState>,
        ),
        Without<ShaderProxyChild>,
    >,
    mut proxy_children: Query<&mut Transform, With<ShaderProxyChild>>,
) {
    for (
        entity,
        mut sprite,
        anchor,
        flash_ref,
        flash_runtime,
        dissolve_ref,
        dissolve_runtime,
        palette_ref,
        outline_ref,
        silhouette_ref,
        proxy,
        presented,
    ) in &mut owners
    {
        let palette_changed = palette_ref.as_ref().is_some_and(|r| r.is_changed());
        let outline_changed = outline_ref.as_ref().is_some_and(|r| r.is_changed());
        let silhouette_changed = silhouette_ref.as_ref().is_some_and(|r| r.is_changed());

        let flash = flash_ref.as_deref();
        let dissolve = dissolve_ref.as_deref();
        let palette = palette_ref.as_deref();
        let outline = outline_ref.as_deref();
        let silhouette = silhouette_ref.as_deref();

        let palette_enabled =
            palette.is_some_and(|p| p.enabled && p.config.texture != Handle::default());
        let dissolve_enabled = dissolve.is_some_and(|e| e.enabled);
        let outline_enabled = outline.is_some_and(|e| e.enabled);
        let silhouette_enabled = silhouette.is_some_and(|e| e.enabled);
        let shader_flash_enabled =
            flash.is_some_and(|e| e.enabled && e.config.blend == FlashBlendMode::Screen);
        let needs_proxy = palette_enabled
            || dissolve_enabled
            || shader_flash_enabled
            || outline_enabled
            || silhouette_enabled;

        if !needs_proxy {
            if let Some(proxy) = proxy {
                commands.entity(proxy.child).despawn_related::<Children>();
                commands.entity(proxy.child).despawn();
                commands.entity(entity).remove::<ShaderProxy>();
            }
            continue;
        }

        let size = sprite_draw_size(&sprite, &images, &atlases);
        let child_translation = Vec3::new(
            -anchor.as_vec().x * size.x,
            -anchor.as_vec().y * size.y,
            silhouette.map_or(0.001, |e| {
                if e.enabled {
                    e.config.sort_offset
                } else {
                    0.001
                }
            }),
        );
        let child_scale = Vec3::new(size.x.max(1.0), size.y.max(1.0), 1.0);
        let just_created = proxy.is_none();

        let (child, material_handle) = if let Some(proxy) = proxy {
            (proxy.child, proxy.material.clone())
        } else {
            let material_handle = materials.add(SpriteEffectsMaterial::default());
            let child = commands
                .spawn((
                    Name::new("Sprite Effects Proxy"),
                    ShaderProxyChild,
                    Mesh2d(internal.quad_mesh.clone()),
                    MeshMaterial2d(material_handle.clone()),
                    Transform {
                        translation: child_translation,
                        scale: child_scale,
                        ..default()
                    },
                    Visibility::Inherited,
                ))
                .id();
            commands.entity(entity).add_child(child);
            commands.entity(entity).insert(ShaderProxy {
                child,
                material: material_handle.clone(),
            });
            (child, material_handle)
        };

        if let Ok(mut transform) = proxy_children.get_mut(child) {
            transform.translation = child_translation;
            transform.scale = child_scale;
        }

        let authored_color = sprite.color;

        if let Some(mut presented) = presented {
            presented.original_color = authored_color;
            presented.active_last_frame = true;
        } else {
            commands.entity(entity).insert(PresentedSpriteState {
                original_color: authored_color,
                active_last_frame: true,
            });
        }

        sprite.color = authored_color.with_alpha(0.0);

        let has_transient = shader_flash_enabled || dissolve_enabled;
        let persistent_changed = palette_changed || outline_changed || silhouette_changed;
        let sprite_changed = sprite.is_changed();
        if !has_transient && !just_created && !persistent_changed && !sprite_changed {
            continue;
        }

        let uv_rect = sprite_uv_rect(&sprite, &images, &atlases);
        let mut uniform = SpriteEffectsUniform {
            base_color: color_to_vec4(authored_color),
            flash_color: Vec4::ZERO,
            edge_color: Vec4::ZERO,
            outline_color: Vec4::ZERO,
            silhouette_color: Vec4::ZERO,
            uv_rect: Vec4::new(uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y),
            flash: Vec4::ZERO,
            dissolve: Vec4::ZERO,
            dissolve_aux: Vec4::ZERO,
            outline: Vec4::ZERO,
            silhouette: Vec4::ZERO,
            palette: Vec4::ZERO,
            flags: Vec4::new(
                if sprite.flip_x { 1.0 } else { 0.0 },
                if sprite.flip_y { 1.0 } else { 0.0 },
                if palette_enabled { 1.0 } else { 0.0 },
                palette.map_or(1.0, |p| if p.config.preserve_alpha { 1.0 } else { 0.0 }),
            ),
        };

        let mut palette_texture = None;
        let mut mask_texture = None;

        if let Some(flash) = flash.filter(|f| f.enabled) {
            let elapsed = flash_runtime.map_or(0.0, |r| r.elapsed_secs);
            let mut lc = flash_runtime.map_or(0, |r| r.loops_completed);
            let pos = loop_position(
                elapsed,
                flash.config.delay_secs,
                flash.config.duration_secs,
                &flash.config.loop_mode,
                &mut lc,
            );
            let local = match pos {
                Ok(Some(t)) => t,
                Ok(None) | Err(()) => 0.0,
            };
            let weight =
                flash_weight(flash.config.easing, flash.config.duration_secs, local)
                    * flash.config.intensity;
            let fc = flash_color_at(&flash.config, local);
            uniform.flash_color = color_to_vec4(fc);
            uniform.flash = Vec4::new(
                weight,
                if flash.config.blend == FlashBlendMode::Screen { 1.0 } else { 0.0 },
                1.0,
                0.0,
            );
        }

        if let Some(dissolve) = dissolve.filter(|d| d.enabled) {
            let elapsed = dissolve_runtime.map_or(0.0, |r| r.elapsed_secs);
            let mut lc = dissolve_runtime.map_or(0, |r| r.loops_completed);
            let pos = loop_position(
                elapsed,
                dissolve.config.delay_secs,
                dissolve.config.duration_secs,
                &dissolve.config.loop_mode,
                &mut lc,
            );
            let local = match pos {
                Ok(Some(t)) => t,
                Ok(None) => 0.0,
                Err(()) => elapsed, // finished — use raw elapsed as fallback
            };
            let threshold = dissolve_threshold(&dissolve.config, local);

            if let Some(ref gradient) = dissolve.config.edge_gradient {
                if let Some(first) = gradient.first() {
                    uniform.edge_color = color_to_vec4(first.color);
                }
            } else {
                uniform.edge_color = color_to_vec4(dissolve.config.edge_color);
            }

            uniform.dissolve = Vec4::new(
                threshold,
                dissolve.config.edge_width.max(0.0),
                dissolve_pattern_code(dissolve.config.pattern),
                1.0,
            );
            uniform.dissolve_aux = Vec4::new(
                dissolve.config.noise_scale.x.max(0.001),
                dissolve.config.noise_scale.y.max(0.001),
                0.0,
                0.0,
            );

            if dissolve.config.pattern == DissolvePattern::Mask {
                mask_texture = dissolve.config.mask_texture.clone();
            }
        }

        if let Some(palette) = palette.filter(|p| p.enabled) {
            palette_texture = Some(palette.config.texture.clone());
            uniform.palette = Vec4::new(
                palette.config.source_row as f32,
                palette.config.target_row as f32,
                palette.config.columns.max(1) as f32,
                palette.config.epsilon.max(0.0001),
            );
        }

        if let Some(outline) = outline.filter(|o| o.enabled) {
            uniform.outline_color = color_to_vec4(outline.config.color);
            uniform.outline = Vec4::new(
                outline.config.width_pixels.max(0.0),
                outline.config.alpha_threshold.clamp(0.0, 1.0),
                1.0,
                0.0,
            );
        }

        if let Some(silhouette) = silhouette.filter(|s| s.enabled) {
            uniform.silhouette_color = color_to_vec4(silhouette.config.color);
            uniform.silhouette = Vec4::new(
                silhouette.config.alpha_threshold.clamp(0.0, 1.0),
                silhouette.config.tint_strength.clamp(0.0, 1.0),
                1.0,
                silhouette.config.sort_offset,
            );
        }

        if let Some(material) = materials.get_mut(&material_handle) {
            material.uniform = uniform;
            material.source_texture = Some(sprite.image.clone());
            material.palette_texture = palette_texture;
            material.mask_texture = mask_texture;
        }
    }
}

// ---------------------------------------------------------------------------
// Full cleanup (deactivation schedule)
// ---------------------------------------------------------------------------

pub(crate) fn cleanup_all(
    mut commands: Commands,
    mut sprites: Query<(Entity, &mut Sprite, Option<&mut PresentedSpriteState>)>,
    mut transforms: Query<(Entity, &mut Transform, Option<&mut PresentedTransformState>)>,
    proxies: Query<(Entity, &ShaderProxy)>,
) {
    for (entity, mut sprite, presented) in &mut sprites {
        if let Some(mut presented) = presented {
            if presented.active_last_frame {
                sprite.color = presented.original_color;
            }
            presented.active_last_frame = false;
        }
        commands
            .entity(entity)
            .remove::<FlashRuntime>()
            .remove::<DissolveRuntime>()
            .remove::<PresentedSpriteState>();
    }

    for (entity, mut transform, presented) in &mut transforms {
        if let Some(mut presented) = presented {
            if presented.active_last_frame {
                transform.translation = presented.original_translation;
                transform.scale = presented.original_scale;
            }
            presented.active_last_frame = false;
        }
        commands
            .entity(entity)
            .remove::<SquashRuntime>()
            .remove::<ShakeRuntime>()
            .remove::<PresentedTransformState>();
    }

    for (entity, proxy) in &proxies {
        commands.entity(proxy.child).despawn_related::<Children>();
        commands.entity(proxy.child).despawn();
        commands.entity(entity).remove::<ShaderProxy>();
    }
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn publish_diagnostics(
    mut diagnostics: ResMut<SpriteEffectsDiagnostics>,
    flashes: Query<&FlashEffect>,
    dissolves: Query<&DissolveEffect>,
    squashes: Query<&SquashStretchEffect>,
    shakes: Query<&ShakeEffect>,
    palettes: Query<&PaletteSwap>,
    outlines: Query<&OutlineEffect>,
    silhouettes: Query<&SilhouetteEffect>,
    proxies: Query<&ShaderProxy>,
) {
    diagnostics.active_flashes = flashes.iter().filter(|e| e.enabled).count();
    diagnostics.active_dissolves = dissolves.iter().filter(|e| e.enabled).count();
    diagnostics.active_squashes = squashes.iter().filter(|e| e.enabled).count();
    diagnostics.active_shakes = shakes.iter().filter(|e| e.enabled).count();
    diagnostics.active_palette_swaps = palettes.iter().filter(|e| e.enabled).count();
    diagnostics.active_outlines = outlines.iter().filter(|e| e.enabled).count();
    diagnostics.active_silhouettes = silhouettes.iter().filter(|e| e.enabled).count();
    diagnostics.active_shader_proxies = proxies.iter().count();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn dissolve_pattern_code(pattern: DissolvePattern) -> f32 {
    match pattern {
        DissolvePattern::Noise => 0.0,
        DissolvePattern::LeftToRight => 1.0,
        DissolvePattern::RightToLeft => 2.0,
        DissolvePattern::BottomToTop => 3.0,
        DissolvePattern::TopToBottom => 4.0,
        DissolvePattern::RadialIn => 5.0,
        DissolvePattern::RadialOut => 6.0,
        DissolvePattern::Mask => 7.0,
    }
}

fn finish_transient<E: Component, R: Component>(
    commands: &mut Commands,
    entity: Entity,
    persistent: bool,
    kind: SpriteEffectKind,
    finished: &mut MessageWriter<SpriteEffectFinished>,
) {
    if persistent {
        commands.entity(entity).remove::<R>();
    } else {
        commands.entity(entity).remove::<E>().remove::<R>();
    }
    finished.write(SpriteEffectFinished {
        entity,
        effect: kind,
    });
}
