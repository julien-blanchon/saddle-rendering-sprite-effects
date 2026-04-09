mod support;

use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use saddle_rendering_sprite_effects::{DissolveEffect, SpriteEffectsDiagnostics};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "sprite_effects_flash",
        "sprite_effects_dissolve",
        "sprite_effects_palette_swap",
        "sprite_effects_outline_silhouette",
        "sprite_effects_atlas_animation",
        "sprite_effects_stress",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "sprite_effects_flash" => Some(sprite_effects_flash()),
        "sprite_effects_dissolve" => Some(sprite_effects_dissolve()),
        "sprite_effects_palette_swap" => Some(sprite_effects_palette_swap()),
        "sprite_effects_outline_silhouette" => Some(sprite_effects_outline_silhouette()),
        "sprite_effects_atlas_animation" => Some(sprite_effects_atlas_animation()),
        "sprite_effects_stress" => Some(sprite_effects_stress()),
        _ => None,
    }
}

fn smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description(
            "Launch the sprite effects lab, verify baseline palette proxy coverage, and capture the opening composition.",
        )
        .then(Action::WaitFrames(30))
        .then(assertions::resource_exists::<SpriteEffectsDiagnostics>(
            "sprite effects diagnostics resource exists",
        ))
        .then(assertions::custom(
            "baseline palette proxies cover the showcase and stress grid",
            |world| {
                let diagnostics = world.resource::<SpriteEffectsDiagnostics>();
                let lab = world.resource::<crate::LabEntities>();
                diagnostics.active_palette_swaps >= lab.stress_targets + 2
                    && diagnostics.active_shader_proxies >= lab.stress_targets + 2
            },
        ))
        .then(assertions::custom("overlay reports the lab title", |world| {
            support::overlay_text(world)
                .is_some_and(|text| text.contains("Sprite Effects Lab"))
        }))
        .then(inspect::log_resource::<SpriteEffectsDiagnostics>(
            "sprite effects smoke diagnostics",
        ))
        .then(Action::Screenshot("smoke_launch".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("smoke_launch"))
        .build()
}

fn sprite_effects_flash() -> Scenario {
    Scenario::builder("sprite_effects_flash")
        .description(
            "Trigger the cheap tint flash and the proxy-backed screen flash together, then capture peak and recovery frames.",
        )
        .then(Action::WaitFrames(10))
        .then(support::flash_pair_action(Color::srgb(1.0, 0.26, 0.26)))
        .then(Action::WaitFrames(2))
        .then(assertions::custom("native tint flash modifies the sprite color", |world| {
            let lab = *world.resource::<crate::LabEntities>();
            world
                .get::<Sprite>(lab.native_flash)
                .is_some_and(|sprite| sprite.color != Color::WHITE)
        }))
        .then(assertions::custom("screen flash uses the proxy path", |world| {
            let lab = *world.resource::<crate::LabEntities>();
            support::has_proxy_child(world, lab.screen_flash)
                && world.resource::<SpriteEffectsDiagnostics>().active_flashes >= 2
        }))
        .then(Action::Screenshot("sprite_effects_flash_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(18))
        .then(assertions::resource_satisfies::<SpriteEffectsDiagnostics>(
            "flash effects cleaned up",
            |diagnostics| diagnostics.active_flashes == 0,
        ))
        .then(Action::Screenshot("sprite_effects_flash_recovered".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("sprite_effects_flash"))
        .build()
}

fn sprite_effects_dissolve() -> Scenario {
    Scenario::builder("sprite_effects_dissolve")
        .description(
            "Run a mask-backed dissolve to completion, assert the mid-effect proxy state, then verify the entity ends hidden.",
        )
        .then(Action::WaitFrames(10))
        .then(support::dissolve_target_action())
        .then(Action::WaitFrames(8))
        .then(assertions::custom("dissolve is active and proxied mid-transition", |world| {
            let lab = *world.resource::<crate::LabEntities>();
            world.resource::<SpriteEffectsDiagnostics>().active_dissolves == 1
                && support::has_proxy_child(world, lab.dissolve_target)
        }))
        .then(Action::Screenshot("sprite_effects_dissolve_mid".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(20))
        .then(assertions::custom("dissolve hides the entity on completion", |world| {
            let lab = *world.resource::<crate::LabEntities>();
            support::is_hidden(world, lab.dissolve_target)
                && world.get::<DissolveEffect>(lab.dissolve_target).is_none()
        }))
        .then(Action::Screenshot("sprite_effects_dissolve_done".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("sprite_effects_dissolve"))
        .build()
}

fn sprite_effects_palette_swap() -> Scenario {
    Scenario::builder("sprite_effects_palette_swap")
        .description(
            "Swap the palette row on the authored target, verify the component and proxy state, then capture the recolored result.",
        )
        .then(Action::WaitFrames(10))
        .then(Action::Screenshot("sprite_effects_palette_before".into()))
        .then(Action::WaitFrames(1))
        .then(support::set_palette_row_action(3))
        .then(Action::WaitFrames(4))
        .then(assertions::custom("palette row switches to the requested bank", |world| {
            let lab = *world.resource::<crate::LabEntities>();
            support::palette_target_row(world) == Some(3)
                && support::has_proxy_child(world, lab.palette_target)
        }))
        .then(Action::Screenshot("sprite_effects_palette_after".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("sprite_effects_palette_swap"))
        .build()
}

fn sprite_effects_atlas_animation() -> Scenario {
    Scenario::builder("sprite_effects_atlas_animation")
        .description(
            "Keep the atlas sprite animating while a screen flash and dissolve run through the proxy path, then capture both the active and recovered frames.",
        )
        .then(Action::WaitFrames(6))
        .then(support::atlas_combo_action())
        .then(Action::WaitFrames(12))
        .then(assertions::custom(
            "atlas animation advances while proxy effects are active",
            |world| {
                let lab = *world.resource::<crate::LabEntities>();
                support::atlas_index(world).is_some_and(|index| index != 0)
                    && support::has_proxy_child(world, lab.atlas_target)
                    && world.resource::<SpriteEffectsDiagnostics>().active_dissolves >= 1
            },
        ))
        .then(Action::Screenshot("sprite_effects_atlas_active".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(20))
        .then(assertions::custom(
            "atlas animation keeps running after the dissolve cleans up",
            |world| {
                let lab = *world.resource::<crate::LabEntities>();
                support::atlas_index(world).is_some()
                    && world.get::<DissolveEffect>(lab.atlas_target).is_none()
                    && support::has_proxy_child(world, lab.atlas_target)
            },
        ))
        .then(Action::Screenshot("sprite_effects_atlas_recovered".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("sprite_effects_atlas_animation"))
        .build()
}

fn sprite_effects_outline_silhouette() -> Scenario {
    Scenario::builder("sprite_effects_outline_silhouette")
        .description(
            "Apply an outline to one showcase actor, occlude a second actor, and verify its silhouette proxy sorts in front for readability.",
        )
        .then(Action::WaitFrames(10))
        .then(support::outline_silhouette_action())
        .then(Action::WaitFrames(4))
        .then(assertions::custom(
            "outline and silhouette paths both activate shader proxies",
            |world| {
                let lab = *world.resource::<crate::LabEntities>();
                let diagnostics = world.resource::<SpriteEffectsDiagnostics>();
                diagnostics.active_outlines >= 2
                    && diagnostics.active_silhouettes >= 1
                    && support::has_proxy_child(world, lab.native_flash)
                    && support::has_proxy_child(world, lab.screen_flash)
            },
        ))
        .then(assertions::custom(
            "silhouette proxy sorts ahead of the parent sprite",
            |world| {
                let lab = *world.resource::<crate::LabEntities>();
                support::proxy_sorts_ahead_of_parent(world, lab.screen_flash)
            },
        ))
        .then(Action::Screenshot("sprite_effects_outline_silhouette".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("sprite_effects_outline_silhouette"))
        .build()
}

fn sprite_effects_stress() -> Scenario {
    Scenario::builder("sprite_effects_stress")
        .description(
            "Burst flash, dissolve, and squash across the dense stress grid, then verify cleanup while palette proxies stay resident.",
        )
        .then(Action::WaitFrames(10))
        .then(support::stress_burst_action())
        .then(Action::WaitFrames(4))
        .then(assertions::custom("stress burst activates many concurrent effects", |world| {
            let diagnostics = world.resource::<SpriteEffectsDiagnostics>();
            let lab = world.resource::<crate::LabEntities>();
            diagnostics.active_palette_swaps >= lab.stress_targets + 2
                && diagnostics.active_shader_proxies >= lab.stress_targets + 2
                && diagnostics.active_flashes >= 20
        }))
        .then(Action::Screenshot("sprite_effects_stress_peak".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(24))
        .then(assertions::custom("stress burst cleans up transient channels", |world| {
            let diagnostics = world.resource::<SpriteEffectsDiagnostics>();
            let lab = world.resource::<crate::LabEntities>();
            diagnostics.active_flashes == 0
                && diagnostics.active_dissolves == 0
                && diagnostics.active_palette_swaps >= lab.stress_targets + 2
        }))
        .then(inspect::log_resource::<SpriteEffectsDiagnostics>(
            "sprite effects stress diagnostics",
        ))
        .then(Action::Screenshot("sprite_effects_stress_recovered".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("sprite_effects_stress"))
        .build()
}
