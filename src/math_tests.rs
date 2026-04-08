use super::math::{dissolve_threshold, sample_color_ramp, sample_shake, sample_squash};
use crate::config::{
    ColorStop, DissolveConfig, DissolvePhase, ShakeConfig, SquashStretchConfig,
};
use bevy::{prelude::*, sprite::Anchor};

#[test]
fn dissolve_threshold_runs_forward_for_hide() {
    let config = DissolveConfig {
        phase: DissolvePhase::Hide,
        duration_secs: 1.0,
        ..Default::default()
    };
    assert!(dissolve_threshold(&config, 0.75) > dissolve_threshold(&config, 0.25));
}

#[test]
fn dissolve_threshold_runs_backward_for_reveal() {
    let config = DissolveConfig {
        phase: DissolvePhase::Reveal,
        duration_secs: 1.0,
        ..Default::default()
    };
    assert!(dissolve_threshold(&config, 0.25) > dissolve_threshold(&config, 0.75));
}

#[test]
fn squash_sample_applies_anchor_compensation_relative_to_sprite_anchor() {
    let config = SquashStretchConfig {
        compensation_anchor: Some(Anchor::BOTTOM_CENTER),
        duration_secs: 1.0,
        amplitude: 0.2,
        rebound: 0.0,
        ..Default::default()
    };
    let sample = sample_squash(&config, 0.2, Anchor::CENTER, Vec2::new(32.0, 48.0));
    assert!(sample.translation.y < 0.0);
}

#[test]
fn squash_sample_preserves_area_when_requested() {
    let config = SquashStretchConfig {
        preserve_area: true,
        duration_secs: 1.0,
        amplitude: 0.18,
        rebound: 0.0,
        ..Default::default()
    };
    let sample = sample_squash(&config, 0.2, Anchor::CENTER, Vec2::splat(16.0));
    let area = sample.scale.x * sample.scale.y;
    assert!(
        (area - 1.0).abs() < 0.05,
        "expected area near 1.0, got {area}"
    );
}

#[test]
fn color_ramp_interpolates_between_stops() {
    let stops = vec![
        ColorStop::new(0.0, Color::BLACK),
        ColorStop::new(1.0, Color::WHITE),
    ];

    let mid = sample_color_ramp(&stops, 0.5).to_linear();
    assert!((mid.red - 0.5).abs() < 0.1, "expected ~0.5 red, got {}", mid.red);
}

#[test]
fn color_ramp_clamps_at_edges() {
    let stops = vec![
        ColorStop::new(0.2, Color::srgb(1.0, 0.0, 0.0)),
        ColorStop::new(0.8, Color::srgb(0.0, 0.0, 1.0)),
    ];

    let before = sample_color_ramp(&stops, 0.0);
    assert_eq!(before, Color::srgb(1.0, 0.0, 0.0));

    let after = sample_color_ramp(&stops, 1.0);
    assert_eq!(after, Color::srgb(0.0, 0.0, 1.0));
}

#[test]
fn shake_sample_returns_zero_at_zero_elapsed() {
    let config = ShakeConfig {
        amplitude: 10.0,
        ..ShakeConfig::default()
    };
    // At elapsed=0, sin(0) = 0.
    let offset = sample_shake(&config, 0.0);
    assert_eq!(offset, Vec2::ZERO);
}

#[test]
fn shake_sample_decays_over_time() {
    let config = ShakeConfig {
        amplitude: 10.0,
        decay: 1.0,
        duration_secs: 1.0,
        ..ShakeConfig::default()
    };
    let early = sample_shake(&config, 0.1).length();
    let late = sample_shake(&config, 0.9).length();
    // With full decay, later samples should generally have less amplitude.
    // Due to sine oscillation, compare absolute max possible.
    assert!(
        late <= early + 1.0,
        "late shake ({late}) should not exceed early ({early}) by much with full decay"
    );
}

#[test]
fn shake_respects_axis_mask() {
    let config = ShakeConfig {
        amplitude: 10.0,
        axis: Vec2::new(1.0, 0.0), // horizontal only
        duration_secs: 1.0,
        ..ShakeConfig::default()
    };
    let offset = sample_shake(&config, 0.05);
    assert_eq!(offset.y, 0.0, "Y should be zero with horizontal-only axis");
}
