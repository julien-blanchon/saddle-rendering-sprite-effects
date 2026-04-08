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

/// Controls what happens when an effect is re-applied while already active.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OverlapPolicy {
    /// Reset the timer to zero and replay from the start.
    #[default]
    Restart,
    /// Ignore the new application; let the current one finish.
    Ignore,
}

/// Controls whether a transient effect loops.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// Play once, then finish.
    #[default]
    None,
    /// Repeat a fixed number of times (total plays = count).
    Count(u32),
    /// Repeat indefinitely until the component is removed.
    Forever,
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

// ---------------------------------------------------------------------------
// Persistent effect configs
// ---------------------------------------------------------------------------

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
            color: Color::WHITE,
            tint_strength: 1.0,
            alpha_threshold: 0.05,
            sort_offset: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Transient effect configs
// ---------------------------------------------------------------------------

/// A single stop in a color ramp: `(progress 0..1, color)`.
#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct ColorStop {
    pub t: f32,
    pub color: Color,
}

impl ColorStop {
    #[must_use]
    pub fn new(t: f32, color: Color) -> Self {
        Self { t, color }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct FlashConfig {
    pub color: Color,
    pub intensity: f32,
    pub duration_secs: f32,
    pub delay_secs: f32,
    pub easing: EaseFunction,
    pub blend: FlashBlendMode,
    pub overlap: OverlapPolicy,
    pub time_domain: EffectTimeDomain,
    pub loop_mode: LoopMode,
    /// If true, the component stays after completion (with `enabled = false`)
    /// so it can be re-triggered without re-insertion.
    pub persistent: bool,
    /// Optional multi-stop color ramp. When set, `color` is ignored and
    /// the flash color is sampled from this ramp based on effect progress.
    pub color_ramp: Option<Vec<ColorStop>>,
}

impl Default for FlashConfig {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            duration_secs: 0.12,
            delay_secs: 0.0,
            easing: EaseFunction::SineOut,
            blend: FlashBlendMode::Tint,
            overlap: OverlapPolicy::Restart,
            time_domain: EffectTimeDomain::Unscaled,
            loop_mode: LoopMode::None,
            persistent: false,
            color_ramp: None,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct DissolveConfig {
    pub duration_secs: f32,
    pub delay_secs: f32,
    pub easing: EaseFunction,
    pub pattern: DissolvePattern,
    pub phase: DissolvePhase,
    pub overlap: OverlapPolicy,
    pub time_domain: EffectTimeDomain,
    pub edge_width: f32,
    pub edge_color: Color,
    pub noise_scale: Vec2,
    pub mask_texture: Option<Handle<Image>>,
    pub completion: DissolveCompletion,
    pub loop_mode: LoopMode,
    pub persistent: bool,
    /// Optional multi-stop edge gradient. When set, `edge_color` is ignored
    /// and the edge color is sampled from this gradient based on distance
    /// from the dissolve threshold (0 = threshold edge, 1 = outer edge).
    pub edge_gradient: Option<Vec<ColorStop>>,
}

impl Default for DissolveConfig {
    fn default() -> Self {
        Self {
            duration_secs: 0.35,
            delay_secs: 0.0,
            easing: EaseFunction::SineInOut,
            pattern: DissolvePattern::Noise,
            phase: DissolvePhase::Hide,
            overlap: OverlapPolicy::Restart,
            time_domain: EffectTimeDomain::GlobalScaled,
            edge_width: 0.0,
            edge_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            noise_scale: Vec2::splat(24.0),
            mask_texture: None,
            completion: DissolveCompletion::RestoreVisible,
            loop_mode: LoopMode::None,
            persistent: false,
            edge_gradient: None,
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
    pub delay_secs: f32,
    pub easing: EaseFunction,
    pub overlap: OverlapPolicy,
    pub time_domain: EffectTimeDomain,
    pub loop_mode: LoopMode,
    pub persistent: bool,
}

impl Default for SquashStretchConfig {
    fn default() -> Self {
        Self {
            amplitude: 0.22,
            rebound: 0.34,
            axis_bias: Vec2::Y,
            preserve_area: true,
            compensation_anchor: None,
            duration_secs: 0.20,
            delay_secs: 0.0,
            easing: EaseFunction::SineOut,
            overlap: OverlapPolicy::Restart,
            time_domain: EffectTimeDomain::Unscaled,
            loop_mode: LoopMode::None,
            persistent: false,
        }
    }
}

#[derive(Reflect, Clone, Debug, PartialEq)]
pub struct ShakeConfig {
    /// Maximum displacement in pixels.
    pub amplitude: f32,
    /// Oscillation frequency in Hz.
    pub frequency: f32,
    /// How quickly amplitude decays (0 = no decay, 1 = fully decayed at end).
    pub decay: f32,
    /// Axis mask — (1,1) for 2D shake, (1,0) for horizontal only, etc.
    pub axis: Vec2,
    pub duration_secs: f32,
    pub delay_secs: f32,
    pub easing: EaseFunction,
    pub overlap: OverlapPolicy,
    pub time_domain: EffectTimeDomain,
    pub loop_mode: LoopMode,
    pub persistent: bool,
}

impl Default for ShakeConfig {
    fn default() -> Self {
        Self {
            amplitude: 4.0,
            frequency: 30.0,
            decay: 0.8,
            axis: Vec2::ONE,
            duration_secs: 0.25,
            delay_secs: 0.0,
            easing: EaseFunction::SineOut,
            overlap: OverlapPolicy::Restart,
            time_domain: EffectTimeDomain::Unscaled,
            loop_mode: LoopMode::None,
            persistent: false,
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
