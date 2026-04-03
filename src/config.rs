use bevy::{math::curve::easing::EaseFunction, prelude::*, sprite::Anchor};

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EffectTimeDomain {
    GlobalScaled,
    #[default]
    Unscaled,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlashBlendMode {
    #[default]
    Tint,
    Screen,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlashOverlap {
    #[default]
    Refresh,
    Replace,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DissolveOverlap {
    #[default]
    Replace,
    Refresh,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DissolvePhase {
    #[default]
    Hide,
    Reveal,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DissolvePattern {
    #[default]
    Noise,
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
    RadialIn,
    RadialOut,
    Mask,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DissolveCompletion {
    #[default]
    RestoreVisible,
    HideEntity,
    DespawnEntity,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SquashOverlap {
    #[default]
    Refresh,
    Replace,
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct OutlineConfig {
    pub color: Color,
    pub width_pixels: f32,
    pub alpha_threshold: f32,
}

impl Default for OutlineConfig {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width_pixels: 1.0,
            alpha_threshold: 0.05,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct SilhouetteConfig {
    pub color: Color,
    pub tint_strength: f32,
    pub alpha_threshold: f32,
    pub sort_offset: f32,
}

impl Default for SilhouetteConfig {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.18, 0.82, 1.0, 0.88),
            tint_strength: 1.0,
            alpha_threshold: 0.05,
            sort_offset: 0.25,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct FlashConfig {
    pub color: Color,
    pub intensity: f32,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub blend: FlashBlendMode,
    pub overlap: FlashOverlap,
    pub time_domain: EffectTimeDomain,
}

impl Default for FlashConfig {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            duration_secs: 0.12,
            easing: EaseFunction::SineOut,
            blend: FlashBlendMode::Tint,
            overlap: FlashOverlap::Refresh,
            time_domain: EffectTimeDomain::Unscaled,
        }
    }
}

impl FlashConfig {
    #[must_use]
    pub fn damage() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            duration_secs: 0.10,
            easing: EaseFunction::SineOut,
            blend: FlashBlendMode::Screen,
            overlap: FlashOverlap::Refresh,
            time_domain: EffectTimeDomain::Unscaled,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct DissolveConfig {
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub pattern: DissolvePattern,
    pub phase: DissolvePhase,
    pub overlap: DissolveOverlap,
    pub time_domain: EffectTimeDomain,
    pub edge_width: f32,
    pub edge_color: Color,
    pub noise_scale: Vec2,
    pub mask_texture: Option<Handle<Image>>,
    pub completion: DissolveCompletion,
}

impl Default for DissolveConfig {
    fn default() -> Self {
        Self {
            duration_secs: 0.35,
            easing: EaseFunction::SineInOut,
            pattern: DissolvePattern::Noise,
            phase: DissolvePhase::Hide,
            overlap: DissolveOverlap::Replace,
            time_domain: EffectTimeDomain::GlobalScaled,
            edge_width: 0.08,
            edge_color: Color::srgb(1.0, 0.68, 0.2),
            noise_scale: Vec2::splat(24.0),
            mask_texture: None,
            completion: DissolveCompletion::RestoreVisible,
        }
    }
}

impl DissolveConfig {
    #[must_use]
    pub fn hide() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn reveal() -> Self {
        Self {
            phase: DissolvePhase::Reveal,
            ..Self::default()
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct SquashStretchConfig {
    pub amplitude: f32,
    pub rebound: f32,
    pub axis_bias: Vec2,
    pub preserve_area: bool,
    pub compensation_anchor: Option<Anchor>,
    pub duration_secs: f32,
    pub easing: EaseFunction,
    pub overlap: SquashOverlap,
    pub time_domain: EffectTimeDomain,
}

impl Default for SquashStretchConfig {
    fn default() -> Self {
        Self {
            amplitude: 0.22,
            rebound: 0.34,
            axis_bias: Vec2::Y,
            preserve_area: true,
            compensation_anchor: Some(Anchor::BOTTOM_CENTER),
            duration_secs: 0.20,
            easing: EaseFunction::SineOut,
            overlap: SquashOverlap::Refresh,
            time_domain: EffectTimeDomain::Unscaled,
        }
    }
}

impl SquashStretchConfig {
    #[must_use]
    pub fn landing() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn recoil(direction: Vec2) -> Self {
        Self {
            axis_bias: direction,
            compensation_anchor: None,
            amplitude: 0.18,
            rebound: 0.18,
            duration_secs: 0.16,
            ..Self::default()
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct PaletteConfig {
    pub texture: Handle<Image>,
    pub source_row: u32,
    pub target_row: u32,
    pub columns: u32,
    pub epsilon: f32,
    pub preserve_alpha: bool,
    pub enforce_nearest_sampling: bool,
}

impl Default for PaletteConfig {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            source_row: 0,
            target_row: 1,
            columns: 4,
            epsilon: 0.01,
            preserve_alpha: true,
            enforce_nearest_sampling: true,
        }
    }
}

impl PaletteConfig {
    #[must_use]
    pub fn new(texture: Handle<Image>, columns: u32) -> Self {
        Self {
            texture,
            columns,
            ..Self::default()
        }
    }
}
