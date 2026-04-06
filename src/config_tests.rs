use bevy::prelude::*;

use crate::{DissolveConfig, SilhouetteConfig, SquashStretchConfig};

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
