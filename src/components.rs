use bevy::prelude::*;

use crate::config::{
    DissolveConfig, FlashConfig, OutlineConfig, PaletteConfig, ShakeConfig, SilhouetteConfig,
    SquashStretchConfig,
};

// ---------------------------------------------------------------------------
// Effect components
// ---------------------------------------------------------------------------

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct FlashEffect {
    pub enabled: bool,
    pub config: FlashConfig,
    /// Incremented by `restart()` to force a reset even while running.
    pub generation: u32,
}

impl Default for FlashEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: FlashConfig::default(),
            generation: 0,
        }
    }
}

impl FlashEffect {
    #[must_use]
    pub fn new(config: FlashConfig) -> Self {
        Self {
            enabled: true,
            config,
            generation: 0,
        }
    }

    /// Re-enable a persistent effect so it replays from the start.
    pub fn retrigger(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }

    /// Force-restart the effect from the beginning, even if already playing.
    pub fn restart(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct DissolveEffect {
    pub enabled: bool,
    pub config: DissolveConfig,
    pub generation: u32,
}

impl Default for DissolveEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: DissolveConfig::default(),
            generation: 0,
        }
    }
}

impl DissolveEffect {
    #[must_use]
    pub fn new(config: DissolveConfig) -> Self {
        Self {
            enabled: true,
            config,
            generation: 0,
        }
    }

    pub fn retrigger(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn restart(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct SquashStretchEffect {
    pub enabled: bool,
    pub config: SquashStretchConfig,
    pub generation: u32,
}

impl Default for SquashStretchEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: SquashStretchConfig::default(),
            generation: 0,
        }
    }
}

impl SquashStretchEffect {
    #[must_use]
    pub fn new(config: SquashStretchConfig) -> Self {
        Self {
            enabled: true,
            config,
            generation: 0,
        }
    }

    pub fn retrigger(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn restart(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct ShakeEffect {
    pub enabled: bool,
    pub config: ShakeConfig,
    pub generation: u32,
}

impl Default for ShakeEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: ShakeConfig::default(),
            generation: 0,
        }
    }
}

impl ShakeEffect {
    #[must_use]
    pub fn new(config: ShakeConfig) -> Self {
        Self {
            enabled: true,
            config,
            generation: 0,
        }
    }

    pub fn retrigger(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn restart(&mut self) {
        self.enabled = true;
        self.generation = self.generation.wrapping_add(1);
    }
}

// ---------------------------------------------------------------------------
// Persistent effects (unchanged)
// ---------------------------------------------------------------------------

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

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct OutlineEffect {
    pub enabled: bool,
    pub config: OutlineConfig,
}

impl Default for OutlineEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: OutlineConfig::default(),
        }
    }
}

impl OutlineEffect {
    #[must_use]
    pub fn new(config: OutlineConfig) -> Self {
        Self {
            enabled: true,
            config,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct SilhouetteEffect {
    pub enabled: bool,
    pub config: SilhouetteConfig,
}

impl Default for SilhouetteEffect {
    fn default() -> Self {
        Self {
            enabled: true,
            config: SilhouetteConfig::default(),
        }
    }
}

impl SilhouetteEffect {
    #[must_use]
    pub fn new(config: SilhouetteConfig) -> Self {
        Self {
            enabled: true,
            config,
        }
    }
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Message, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteEffectStarted {
    pub entity: Entity,
    pub effect: SpriteEffectKind,
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
    Shake,
}
