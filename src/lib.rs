use bevy::{
    app::PostStartup,
    asset::{load_internal_asset, uuid_handle},
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
    shader::Shader,
    sprite_render::Material2dPlugin,
};

mod components;
mod config;
mod diagnostics;
mod material;
mod math;
mod systems;

pub use components::{
    DissolveEffect, FlashEffect, PaletteSwap, SpriteEffectFinished, SpriteEffectKind,
    SquashStretchEffect,
};
pub use config::{
    DissolveCompletion, DissolveConfig, DissolveOverlap, DissolvePattern, DissolvePhase,
    EffectTimeDomain, FlashBlendMode, FlashConfig, FlashOverlap, PaletteConfig, SquashOverlap,
    SquashStretchConfig,
};
pub use diagnostics::SpriteEffectsDiagnostics;

pub(crate) const SPRITE_EFFECTS_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("55aa0654-9bdb-43f4-80d7-54bc28de5138");

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SpriteEffectsSystems {
    Prepare,
    TickCpuEffects,
    UpdateMaterials,
    Cleanup,
    Diagnostics,
}

#[derive(Resource, Default)]
pub(crate) struct SpriteEffectsRuntimeState {
    pub active: bool,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct SpriteEffectsPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl SpriteEffectsPlugin {
    #[must_use]
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    #[must_use]
    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for SpriteEffectsPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for SpriteEffectsPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        if app.world().contains_resource::<Assets<Shader>>() {
            load_internal_asset!(
                app,
                SPRITE_EFFECTS_SHADER_HANDLE,
                "shaders/sprite_effects.wgsl",
                Shader::from_wgsl
            );
        }

        app.add_plugins(Material2dPlugin::<material::SpriteEffectsMaterial>::default())
            .init_resource::<SpriteEffectsRuntimeState>()
            .init_resource::<SpriteEffectsDiagnostics>()
            .init_resource::<material::SpriteEffectsInternalAssets>()
            .add_message::<SpriteEffectFinished>()
            .register_type::<DissolveCompletion>()
            .register_type::<DissolveConfig>()
            .register_type::<DissolveEffect>()
            .register_type::<DissolveOverlap>()
            .register_type::<DissolvePattern>()
            .register_type::<DissolvePhase>()
            .register_type::<EffectTimeDomain>()
            .register_type::<FlashBlendMode>()
            .register_type::<FlashConfig>()
            .register_type::<FlashEffect>()
            .register_type::<FlashOverlap>()
            .register_type::<PaletteConfig>()
            .register_type::<PaletteSwap>()
            .register_type::<SpriteEffectFinished>()
            .register_type::<SpriteEffectKind>()
            .register_type::<SpriteEffectsDiagnostics>()
            .register_type::<SquashOverlap>()
            .register_type::<SquashStretchConfig>()
            .register_type::<SquashStretchEffect>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(
                self.deactivate_schedule,
                (systems::deactivate_runtime, systems::cleanup_all)
                    .chain()
                    .in_set(SpriteEffectsSystems::Cleanup),
            )
            .configure_sets(
                self.update_schedule,
                (
                    SpriteEffectsSystems::Prepare,
                    SpriteEffectsSystems::TickCpuEffects,
                    SpriteEffectsSystems::UpdateMaterials,
                    SpriteEffectsSystems::Cleanup,
                    SpriteEffectsSystems::Diagnostics,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::restore_presented_sprite_state.in_set(SpriteEffectsSystems::Prepare),
                    systems::restore_presented_transform_state
                        .in_set(SpriteEffectsSystems::Prepare),
                    systems::ensure_internal_mesh.in_set(SpriteEffectsSystems::Prepare),
                    systems::enforce_palette_samplers.in_set(SpriteEffectsSystems::Prepare),
                    (
                        systems::tick_flash_effects,
                        systems::tick_dissolve_effects,
                        systems::tick_squash_effects,
                        systems::apply_native_flash,
                    )
                        .chain()
                        .in_set(SpriteEffectsSystems::TickCpuEffects)
                        .run_if(systems::runtime_is_active),
                    systems::sync_shader_proxies
                        .in_set(SpriteEffectsSystems::UpdateMaterials)
                        .run_if(systems::runtime_is_active),
                    systems::cleanup_disabled_effect_state.in_set(SpriteEffectsSystems::Cleanup),
                    systems::publish_diagnostics.in_set(SpriteEffectsSystems::Diagnostics),
                ),
            );
    }
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;
