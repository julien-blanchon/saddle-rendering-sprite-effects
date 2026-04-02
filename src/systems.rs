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
        DissolveEffect, FlashEffect, PaletteSwap, SpriteEffectFinished, SpriteEffectKind,
        SquashStretchEffect,
    },
    config::{DissolveCompletion, DissolvePattern, FlashBlendMode, FlashOverlap, SquashOverlap},
    diagnostics::SpriteEffectsDiagnostics,
    material::{SpriteEffectsInternalAssets, SpriteEffectsMaterial, SpriteEffectsUniform},
    math::{
        color_to_vec4, dissolve_threshold, effect_progress, flash_weight, resolve_time_delta,
        sample_squash, sprite_draw_size, sprite_uv_rect,
    },
};

#[derive(Component, Debug, Default)]
pub(crate) struct FlashRuntime {
    pub elapsed_secs: f32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct DissolveRuntime {
    pub elapsed_secs: f32,
}

#[derive(Component, Debug, Default)]
pub(crate) struct SquashRuntime {
    pub elapsed_secs: f32,
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

pub(crate) fn activate_runtime(mut state: ResMut<SpriteEffectsRuntimeState>) {
    state.active = true;
}

pub(crate) fn deactivate_runtime(mut state: ResMut<SpriteEffectsRuntimeState>) {
    state.active = false;
}

pub(crate) fn runtime_is_active(state: Res<SpriteEffectsRuntimeState>) -> bool {
    state.active
}

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

pub(crate) fn tick_flash_effects(
    mut commands: Commands,
    virtual_time: Res<Time<Virtual>>,
    real_time: Res<Time<Real>>,
    mut finished: MessageWriter<SpriteEffectFinished>,
    mut query: Query<(Entity, Ref<FlashEffect>, Option<&mut FlashRuntime>)>,
) {
    for (entity, effect, runtime) in &mut query {
        if !effect.enabled {
            continue;
        }

        let mut elapsed_secs = runtime.as_ref().map_or(0.0, |runtime| runtime.elapsed_secs);
        if runtime.is_none()
            || effect.is_changed()
                && matches!(
                    effect.config.overlap,
                    FlashOverlap::Refresh | FlashOverlap::Replace
                )
        {
            elapsed_secs = 0.0;
        } else {
            elapsed_secs +=
                resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);
        }
        commands
            .entity(entity)
            .insert(FlashRuntime { elapsed_secs });

        if effect_progress(effect.config.duration_secs, elapsed_secs) >= 1.0 {
            commands
                .entity(entity)
                .remove::<FlashEffect>()
                .remove::<FlashRuntime>();
            finished.write(SpriteEffectFinished {
                entity,
                effect: SpriteEffectKind::Flash,
            });
        }
    }
}

pub(crate) fn tick_dissolve_effects(
    mut commands: Commands,
    virtual_time: Res<Time<Virtual>>,
    real_time: Res<Time<Real>>,
    mut finished: MessageWriter<SpriteEffectFinished>,
    mut query: Query<(Entity, Ref<DissolveEffect>, Option<&mut DissolveRuntime>)>,
) {
    for (entity, effect, runtime) in &mut query {
        if !effect.enabled {
            continue;
        }

        let mut elapsed_secs = runtime.as_ref().map_or(0.0, |runtime| runtime.elapsed_secs);
        let needs_reset = runtime.is_none()
            || effect.is_changed()
                && matches!(
                    effect.config.overlap,
                    crate::config::DissolveOverlap::Replace
                        | crate::config::DissolveOverlap::Refresh
                );
        if needs_reset {
            elapsed_secs = 0.0;
        } else {
            elapsed_secs +=
                resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);
        }
        commands
            .entity(entity)
            .insert(DissolveRuntime { elapsed_secs });

        if effect_progress(effect.config.duration_secs, elapsed_secs) >= 1.0 {
            match effect.config.completion {
                DissolveCompletion::RestoreVisible => {
                    commands
                        .entity(entity)
                        .remove::<DissolveEffect>()
                        .remove::<DissolveRuntime>();
                }
                DissolveCompletion::HideEntity => {
                    commands
                        .entity(entity)
                        .insert(Visibility::Hidden)
                        .remove::<DissolveEffect>()
                        .remove::<DissolveRuntime>();
                }
                DissolveCompletion::DespawnEntity => {
                    commands.entity(entity).despawn_related::<Children>();
                    commands.entity(entity).despawn();
                }
            }

            finished.write(SpriteEffectFinished {
                entity,
                effect: SpriteEffectKind::Dissolve,
            });
        }
    }
}

pub(crate) fn tick_squash_effects(
    mut commands: Commands,
    virtual_time: Res<Time<Virtual>>,
    real_time: Res<Time<Real>>,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    mut finished: MessageWriter<SpriteEffectFinished>,
    mut query: Query<(
        Entity,
        Ref<SquashStretchEffect>,
        Option<&mut SquashRuntime>,
        &Anchor,
        &Sprite,
        &mut Transform,
        Option<&mut PresentedTransformState>,
    )>,
) {
    for (entity, effect, runtime, anchor, sprite, mut transform, presented) in &mut query {
        if !effect.enabled {
            continue;
        }

        let mut elapsed_secs = runtime.as_ref().map_or(0.0, |runtime| runtime.elapsed_secs);
        let needs_reset = runtime.is_none()
            || effect.is_changed()
                && matches!(
                    effect.config.overlap,
                    SquashOverlap::Refresh | SquashOverlap::Replace
                );
        if needs_reset {
            elapsed_secs = resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);
        } else {
            elapsed_secs +=
                resolve_time_delta(effect.config.time_domain, &virtual_time, &real_time);
        }
        commands
            .entity(entity)
            .insert(SquashRuntime { elapsed_secs });

        if effect_progress(effect.config.duration_secs, elapsed_secs) >= 1.0 {
            commands
                .entity(entity)
                .remove::<SquashStretchEffect>()
                .remove::<SquashRuntime>();
            finished.write(SpriteEffectFinished {
                entity,
                effect: SpriteEffectKind::SquashStretch,
            });
            continue;
        }

        let size = sprite_draw_size(sprite, &images, &atlases);
        let sample = sample_squash(&effect.config, elapsed_secs, *anchor, size);
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
            || palette.is_some_and(|palette| palette.enabled)
            || dissolve.is_some_and(|dissolve| dissolve.enabled)
            || proxy.is_some()
        {
            continue;
        }

        let elapsed_secs = runtime.map_or(0.0, |runtime| runtime.elapsed_secs);
        let weight = flash_weight(
            effect.config.easing,
            effect.config.duration_secs,
            elapsed_secs,
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
        let flash = effect.config.color.to_linear();
        sprite.color = Color::LinearRgba(LinearRgba::new(
            base.red + (flash.red - base.red) * weight,
            base.green + (flash.green - base.green) * weight,
            base.blue + (flash.blue - base.blue) * weight,
            base.alpha,
        ));
    }
}

pub(crate) fn enforce_palette_samplers(
    mut images: ResMut<Assets<Image>>,
    query: Query<&PaletteSwap>,
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

pub(crate) fn cleanup_disabled_effect_state(
    mut commands: Commands,
    flashes: Query<(Entity, &FlashEffect, Option<&FlashRuntime>)>,
    dissolves: Query<(Entity, &DissolveEffect, Option<&DissolveRuntime>)>,
    squashes: Query<(Entity, &SquashStretchEffect, Option<&SquashRuntime>)>,
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
}

#[allow(clippy::too_many_arguments)]
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
            Option<&FlashEffect>,
            Option<&FlashRuntime>,
            Option<&DissolveEffect>,
            Option<&DissolveRuntime>,
            Option<&PaletteSwap>,
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
        flash,
        flash_runtime,
        dissolve,
        dissolve_runtime,
        palette,
        proxy,
        presented,
    ) in &mut owners
    {
        let palette_enabled = palette
            .is_some_and(|palette| palette.enabled && palette.config.texture != Handle::default());
        let dissolve_enabled = dissolve.is_some_and(|effect| effect.enabled);
        let shader_flash_enabled = flash
            .is_some_and(|effect| effect.enabled && effect.config.blend == FlashBlendMode::Screen);
        let needs_proxy = palette_enabled || dissolve_enabled || shader_flash_enabled;

        if !needs_proxy {
            if let Some(proxy) = proxy {
                commands.entity(proxy.child).despawn_related::<Children>();
                commands.entity(proxy.child).despawn();
                commands.entity(entity).remove::<ShaderProxy>();
            }
            continue;
        }

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
                    Transform::from_xyz(0.0, 0.0, 0.001),
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
            let size = sprite_draw_size(&sprite, &images, &atlases);
            transform.translation = Vec3::new(
                -anchor.as_vec().x * size.x,
                -anchor.as_vec().y * size.y,
                0.001,
            );
            transform.scale = Vec3::new(size.x.max(1.0), size.y.max(1.0), 1.0);
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

        let uv_rect = sprite_uv_rect(&sprite, &images, &atlases);
        let mut uniform = SpriteEffectsUniform {
            base_color: color_to_vec4(authored_color),
            flash_color: Vec4::ZERO,
            edge_color: Vec4::ZERO,
            uv_rect: Vec4::new(uv_rect.min.x, uv_rect.min.y, uv_rect.max.x, uv_rect.max.y),
            flash: Vec4::ZERO,
            dissolve: Vec4::ZERO,
            dissolve_aux: Vec4::ZERO,
            palette: Vec4::ZERO,
            flags: Vec4::new(
                if sprite.flip_x { 1.0 } else { 0.0 },
                if sprite.flip_y { 1.0 } else { 0.0 },
                if palette_enabled { 1.0 } else { 0.0 },
                palette.map_or(1.0, |palette| {
                    if palette.config.preserve_alpha {
                        1.0
                    } else {
                        0.0
                    }
                }),
            ),
        };

        let mut palette_texture = None;
        let mut mask_texture = None;

        if let Some(flash) = flash.filter(|flash| flash.enabled) {
            let weight = flash_weight(
                flash.config.easing,
                flash.config.duration_secs,
                flash_runtime.map_or(0.0, |runtime| runtime.elapsed_secs),
            ) * flash.config.intensity;
            uniform.flash_color = color_to_vec4(flash.config.color);
            uniform.flash = Vec4::new(
                weight,
                if flash.config.blend == FlashBlendMode::Screen {
                    1.0
                } else {
                    0.0
                },
                1.0,
                0.0,
            );
        }

        if let Some(dissolve) = dissolve.filter(|dissolve| dissolve.enabled) {
            let threshold = dissolve_threshold(
                &dissolve.config,
                dissolve_runtime.map_or(0.0, |runtime| runtime.elapsed_secs),
            );
            uniform.edge_color = color_to_vec4(dissolve.config.edge_color);
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

        if let Some(palette) = palette.filter(|palette| palette.enabled) {
            palette_texture = Some(palette.config.texture.clone());
            uniform.palette = Vec4::new(
                palette.config.source_row as f32,
                palette.config.target_row as f32,
                palette.config.columns.max(1) as f32,
                palette.config.epsilon.max(0.0001),
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
            .remove::<PresentedTransformState>();
    }

    for (entity, proxy) in &proxies {
        commands.entity(proxy.child).despawn_related::<Children>();
        commands.entity(proxy.child).despawn();
        commands.entity(entity).remove::<ShaderProxy>();
    }
}

pub(crate) fn publish_diagnostics(
    mut diagnostics: ResMut<SpriteEffectsDiagnostics>,
    flashes: Query<&FlashEffect>,
    dissolves: Query<&DissolveEffect>,
    squashes: Query<&SquashStretchEffect>,
    palettes: Query<&PaletteSwap>,
    proxies: Query<&ShaderProxy>,
) {
    diagnostics.active_flashes = flashes.iter().filter(|effect| effect.enabled).count();
    diagnostics.active_dissolves = dissolves.iter().filter(|effect| effect.enabled).count();
    diagnostics.active_squashes = squashes.iter().filter(|effect| effect.enabled).count();
    diagnostics.active_palette_swaps = palettes.iter().filter(|effect| effect.enabled).count();
    diagnostics.active_shader_proxies = proxies.iter().count();
}
