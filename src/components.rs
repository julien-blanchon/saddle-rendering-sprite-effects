use bevy::prelude::*;

use crate::config::{DissolveConfig, FlashConfig, PaletteConfig, SquashStretchConfig};

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct FlashEffect {
    pub enabled: bool,
    pub config: FlashConfig,
}

impl Default for FlashEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: FlashConfig::default(),
        }
    }
}

impl FlashEffect {
    #[must_use]
    pub fn new(config: FlashConfig) -> Self {
        Self {
            enabled: true,
            config,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct DissolveEffect {
    pub enabled: bool,
    pub config: DissolveConfig,
}

impl Default for DissolveEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: DissolveConfig::default(),
        }
    }
}

impl DissolveEffect {
    #[must_use]
    pub fn new(config: DissolveConfig) -> Self {
        Self {
            enabled: true,
            config,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct SquashStretchEffect {
    pub enabled: bool,
    pub config: SquashStretchConfig,
}

impl Default for SquashStretchEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: SquashStretchConfig::default(),
        }
    }
}

impl SquashStretchEffect {
    #[must_use]
    pub fn new(config: SquashStretchConfig) -> Self {
        Self {
            enabled: true,
            config,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct PaletteSwap {
    pub enabled: bool,
    pub config: PaletteConfig,
}

impl Default for PaletteSwap {
    fn default() -> Self {
        Self {
            enabled: true,
            config: PaletteConfig::default(),
        }
    }
}

impl PaletteSwap {
    #[must_use]
    pub fn new(config: PaletteConfig) -> Self {
        Self {
            enabled: true,
            config,
        }
    }
}

#[derive(Message, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteEffectFinished {
    pub entity: Entity,
    pub effect: SpriteEffectKind,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SpriteEffectKind {
    #[default]
    Flash,
    Dissolve,
    SquashStretch,
}
