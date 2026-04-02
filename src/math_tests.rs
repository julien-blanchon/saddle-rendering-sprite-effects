use super::math::{dissolve_threshold, sample_squash};
use crate::config::{DissolveConfig, DissolvePhase, SquashStretchConfig};
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
