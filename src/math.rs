use bevy::{
    color::LinearRgba,
    image::TextureAtlasLayout,
    math::curve::{Curve, easing::EaseFunction},
    prelude::*,
    sprite::Anchor,
    time::{Real, Virtual},
};

use crate::config::{
    ColorStop, DissolveConfig, DissolvePhase, EffectTimeDomain, FlashConfig, ShakeConfig,
    SquashStretchConfig,
};

pub(crate) const MIN_SCALE: f32 = 0.05;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SquashSample {
    pub scale: Vec2,
    pub translation: Vec2,
}

pub(crate) fn color_to_vec4(color: Color) -> Vec4 {
    let linear: LinearRgba = color.to_linear();
    Vec4::new(linear.red, linear.green, linear.blue, linear.alpha)
}

pub(crate) fn resolve_time_delta(
    domain: EffectTimeDomain,
    virtual_time: &Time<Virtual>,
    real_time: &Time<Real>,
) -> f32 {
    match domain {
        EffectTimeDomain::GlobalScaled => virtual_time.delta_secs(),
        EffectTimeDomain::Unscaled => real_time.delta_secs(),
    }
}

pub(crate) fn effect_progress(duration_secs: f32, elapsed_secs: f32) -> f32 {
    if duration_secs <= f32::EPSILON {
        1.0
    } else {
        (elapsed_secs / duration_secs).clamp(0.0, 1.0)
    }
}

pub(crate) fn flash_weight(easing: EaseFunction, duration_secs: f32, elapsed_secs: f32) -> f32 {
    1.0 - easing.sample_clamped(effect_progress(duration_secs, elapsed_secs))
}

pub(crate) fn dissolve_threshold(config: &DissolveConfig, elapsed_secs: f32) -> f32 {
    let progress = config
        .easing
        .sample_clamped(effect_progress(config.duration_secs, elapsed_secs));
    match config.phase {
        DissolvePhase::Hide => progress,
        DissolvePhase::Reveal => 1.0 - progress,
    }
}

/// Sample flash color, supporting optional color ramp.
pub(crate) fn flash_color_at(config: &FlashConfig, elapsed_secs: f32) -> Color {
    let Some(ref ramp) = config.color_ramp else {
        return config.color;
    };
    if ramp.is_empty() {
        return config.color;
    }
    let progress = effect_progress(config.duration_secs, elapsed_secs);
    sample_color_ramp(ramp, progress)
}

/// Linear interpolation through a sorted list of color stops.
pub(crate) fn sample_color_ramp(stops: &[ColorStop], t: f32) -> Color {
    if stops.is_empty() {
        return Color::WHITE;
    }
    if stops.len() == 1 || t <= stops[0].t {
        return stops[0].color;
    }
    if t >= stops[stops.len() - 1].t {
        return stops[stops.len() - 1].color;
    }
    // Find the two surrounding stops.
    for i in 0..stops.len() - 1 {
        if t >= stops[i].t && t <= stops[i + 1].t {
            let span = stops[i + 1].t - stops[i].t;
            if span <= f32::EPSILON {
                return stops[i + 1].color;
            }
            let local_t = (t - stops[i].t) / span;
            let a = stops[i].color.to_linear();
            let b = stops[i + 1].color.to_linear();
            return Color::LinearRgba(LinearRgba::new(
                a.red + (b.red - a.red) * local_t,
                a.green + (b.green - a.green) * local_t,
                a.blue + (b.blue - a.blue) * local_t,
                a.alpha + (b.alpha - a.alpha) * local_t,
            ));
        }
    }
    stops[stops.len() - 1].color
}

// ---------------------------------------------------------------------------
// Squash/Stretch
// ---------------------------------------------------------------------------

pub(crate) fn sprite_pixel_rect(
    sprite: &Sprite,
    images: &Assets<Image>,
    atlases: &Assets<TextureAtlasLayout>,
) -> Rect {
    let image_size = images
        .get(&sprite.image)
        .map(Image::size)
        .unwrap_or(UVec2::ONE);

    let atlas_rect = sprite
        .texture_atlas
        .as_ref()
        .and_then(|atlas| atlas.texture_rect(atlases))
        .map(|rect| rect.as_rect());

    match (atlas_rect, sprite.rect) {
        (None, None) => Rect::new(0.0, 0.0, image_size.x as f32, image_size.y as f32),
        (None, Some(rect)) => rect,
        (Some(rect), None) => rect,
        (Some(atlas_rect), Some(mut rect)) => {
            rect.min += atlas_rect.min;
            rect.max += atlas_rect.min;
            rect
        }
    }
}

pub(crate) fn sprite_draw_size(
    sprite: &Sprite,
    images: &Assets<Image>,
    atlases: &Assets<TextureAtlasLayout>,
) -> Vec2 {
    sprite
        .custom_size
        .unwrap_or_else(|| sprite_pixel_rect(sprite, images, atlases).size())
}

pub(crate) fn sprite_uv_rect(
    sprite: &Sprite,
    images: &Assets<Image>,
    atlases: &Assets<TextureAtlasLayout>,
) -> Rect {
    let rect = sprite_pixel_rect(sprite, images, atlases);
    let size = images
        .get(&sprite.image)
        .map(Image::size)
        .unwrap_or(UVec2::ONE)
        .as_vec2()
        .max(Vec2::ONE);

    Rect::from_corners(rect.min / size, rect.max / size)
}

pub(crate) fn sample_squash(
    config: &SquashStretchConfig,
    elapsed_secs: f32,
    current_anchor: Anchor,
    size: Vec2,
) -> SquashSample {
    let progress = effect_progress(config.duration_secs, elapsed_secs);
    let eased = config.easing.sample_clamped(progress);

    let signed = if progress < 0.35 {
        -config.amplitude * eased
    } else if progress < 0.7 {
        let rebound_progress = ((progress - 0.35) / 0.35).clamp(0.0, 1.0);
        let mix = config.easing.sample_clamped(rebound_progress);
        (-config.amplitude).lerp(config.amplitude * config.rebound, mix)
    } else {
        let settle_progress = ((progress - 0.7) / 0.3).clamp(0.0, 1.0);
        let mix = config.easing.sample_clamped(settle_progress);
        (config.amplitude * config.rebound).lerp(0.0, mix)
    };

    let axis = if config.axis_bias.length_squared() <= f32::EPSILON {
        Vec2::Y
    } else {
        config.axis_bias.abs().normalize_or_zero()
    };

    let axis_scale = (1.0 + signed).max(MIN_SCALE);
    let cross_scale = if config.preserve_area {
        (1.0 / axis_scale).max(MIN_SCALE)
    } else {
        (1.0 - signed * 0.5).max(MIN_SCALE)
    };

    let scale = Vec2::new(
        cross_scale + axis.x * (axis_scale - cross_scale),
        cross_scale + axis.y * (axis_scale - cross_scale),
    );

    let translation = config
        .compensation_anchor
        .map(|desired| (desired.as_vec() - current_anchor.as_vec()) * size * (Vec2::ONE - scale))
        .unwrap_or(Vec2::ZERO);

    SquashSample { scale, translation }
}

// ---------------------------------------------------------------------------
// Shake
// ---------------------------------------------------------------------------

/// Sample shake displacement at a given elapsed time.
pub(crate) fn sample_shake(config: &ShakeConfig, elapsed_secs: f32) -> Vec2 {
    let progress = effect_progress(config.duration_secs, elapsed_secs);
    let decay_envelope = 1.0 - config.decay * config.easing.sample_clamped(progress);

    // Use sine waves at different frequencies for X and Y to avoid diagonal patterns.
    let t = elapsed_secs * config.frequency * std::f32::consts::TAU;
    let x = (t).sin() * config.amplitude * decay_envelope * config.axis.x;
    // Offset Y frequency by golden ratio to decorrelate axes.
    let y = (t * 1.618).sin() * config.amplitude * decay_envelope * config.axis.y;

    Vec2::new(x, y)
}
