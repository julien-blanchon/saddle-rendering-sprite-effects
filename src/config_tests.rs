use bevy::prelude::*;

use crate::{DissolveConfig, ShakeConfig, SilhouetteConfig, SquashStretchConfig, LoopMode};

#[test]
fn dissolve_defaults_keep_the_edge_disabled() {
    let config = DissolveConfig::default();
    assert_eq!(config.edge_width, 0.0);
    assert_eq!(config.edge_color, Color::srgba(1.0, 1.0, 1.0, 0.0));
}

#[test]
fn silhouette_defaults_use_a_neutral_tint_without_depth_offset() {
    let config = SilhouetteConfig::default();
    assert_eq!(config.color, Color::WHITE);
    assert_eq!(config.sort_offset, 0.0);
}

#[test]
fn squash_defaults_do_not_assume_a_ground_anchor() {
    let config = SquashStretchConfig::default();
    assert_eq!(config.axis_bias, Vec2::Y);
    assert_eq!(config.compensation_anchor, None);
}

#[test]
fn all_transient_configs_default_to_zero_delay() {
    assert_eq!(crate::FlashConfig::default().delay_secs, 0.0);
    assert_eq!(DissolveConfig::default().delay_secs, 0.0);
    assert_eq!(SquashStretchConfig::default().delay_secs, 0.0);
    assert_eq!(ShakeConfig::default().delay_secs, 0.0);
}

#[test]
fn all_transient_configs_default_to_no_loop() {
    assert_eq!(crate::FlashConfig::default().loop_mode, LoopMode::None);
    assert_eq!(DissolveConfig::default().loop_mode, LoopMode::None);
    assert_eq!(SquashStretchConfig::default().loop_mode, LoopMode::None);
    assert_eq!(ShakeConfig::default().loop_mode, LoopMode::None);
}

#[test]
fn all_transient_configs_default_to_non_persistent() {
    assert!(!crate::FlashConfig::default().persistent);
    assert!(!DissolveConfig::default().persistent);
    assert!(!SquashStretchConfig::default().persistent);
    assert!(!ShakeConfig::default().persistent);
}

#[test]
fn shake_defaults_are_sensible() {
    let config = ShakeConfig::default();
    assert!(config.amplitude > 0.0);
    assert!(config.frequency > 0.0);
    assert!(config.duration_secs > 0.0);
    assert_eq!(config.axis, Vec2::ONE);
}
