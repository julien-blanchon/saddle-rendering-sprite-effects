use bevy::prelude::*;
use saddle_bevy_e2e::action::Action;
use saddle_rendering_sprite_effects::PaletteSwap;
use saddle_rendering_sprite_effects::{
    DissolveCompletion, DissolveConfig, DissolveEffect, DissolvePattern, FlashConfig,
    FlashEffect, OutlineConfig, OutlineEffect, SilhouetteConfig, SilhouetteEffect,
    SquashStretchEffect,
};
use saddle_rendering_sprite_effects_example_common::{
    showcase_grounded_squash_config, showcase_hide_dissolve_config, showcase_screen_flash_config,
};

pub fn overlay_text(world: &World) -> Option<String> {
    let overlay = world.get_resource::<crate::LabEntities>()?.overlay;
    Some(world.get::<Text>(overlay)?.0.clone())
}

pub fn palette_target_row(world: &World) -> Option<u32> {
    let target = world.get_resource::<crate::LabEntities>()?.palette_target;
    Some(world.get::<PaletteSwap>(target)?.config.target_row)
}

pub fn set_palette_row_action(target_row: u32) -> Action {
    Action::Custom(Box::new(move |world| {
        let lab = *world.resource::<crate::LabEntities>();
        world
            .entity_mut(lab.palette_target)
            .get_mut::<PaletteSwap>()
            .expect("palette target should exist")
            .config
            .target_row = target_row;
    }))
}

pub fn atlas_index(world: &World) -> Option<usize> {
    let target = world.get_resource::<crate::LabEntities>()?.atlas_target;
    world
        .get::<Sprite>(target)?
        .texture_atlas
        .as_ref()
        .map(|atlas| atlas.index)
}

pub fn flash_pair_action(native_color: Color) -> Action {
    Action::Custom(Box::new(move |world| {
        let lab = *world.resource::<crate::LabEntities>();
        world.entity_mut(lab.native_flash).insert(FlashEffect::new(FlashConfig {
            color: native_color,
            duration_secs: 0.18,
            ..FlashConfig::default()
        }));
        world.entity_mut(lab.screen_flash).insert((
            FlashEffect::new(showcase_screen_flash_config()),
            SquashStretchEffect::new(showcase_grounded_squash_config()),
        ));
    }))
}

pub fn dissolve_target_action() -> Action {
    Action::Custom(Box::new(|world| {
        let lab = *world.resource::<crate::LabEntities>();
        let assets = world.resource::<crate::LabAssets>().clone();
        world.entity_mut(lab.dissolve_target).insert(Visibility::Inherited);
        world.entity_mut(lab.dissolve_target).insert(DissolveEffect::new(DissolveConfig {
            duration_secs: 0.30,
            pattern: DissolvePattern::Mask,
            mask_texture: Some(assets.mask),
            completion: DissolveCompletion::HideEntity,
            ..showcase_hide_dissolve_config()
        }));
    }))
}

pub fn atlas_combo_action() -> Action {
    Action::Custom(Box::new(|world| {
        let lab = *world.resource::<crate::LabEntities>();
        world.entity_mut(lab.atlas_target).insert((
            FlashEffect::new(showcase_screen_flash_config()),
            DissolveEffect::new(DissolveConfig {
                duration_secs: 0.28,
                ..showcase_hide_dissolve_config()
            }),
        ));
    }))
}

pub fn outline_silhouette_action() -> Action {
    Action::Custom(Box::new(|world| {
        let lab = *world.resource::<crate::LabEntities>();
        world.entity_mut(lab.native_flash).insert(OutlineEffect::new(OutlineConfig {
            color: Color::srgba(0.03, 0.04, 0.06, 0.96),
            width_pixels: 3.0,
            alpha_threshold: 0.05,
        }));
        world.entity_mut(lab.screen_flash).insert((
            OutlineEffect::new(OutlineConfig {
                color: Color::srgba(1.0, 0.93, 0.84, 0.98),
                width_pixels: 2.0,
                alpha_threshold: 0.05,
            }),
            SilhouetteEffect::new(SilhouetteConfig {
                color: Color::srgba(0.16, 0.82, 1.0, 0.92),
                tint_strength: 1.0,
                alpha_threshold: 0.05,
                sort_offset: 1.2,
            }),
        ));
        world.spawn((
            Name::new("Silhouette Occluder"),
            Sprite::from_color(Color::srgba(0.07, 0.09, 0.11, 0.97), Vec2::new(180.0, 160.0)),
            Transform::from_xyz(-145.0, 180.0, 0.6),
        ));
    }))
}

pub fn stress_burst_action() -> Action {
    Action::Custom(Box::new(|world: &mut World| {
        let entities: Vec<Entity> = {
            let mut query = world.query_filtered::<Entity, With<crate::StressTarget>>();
            query.iter(world).collect()
        };
        for (index, entity) in entities.into_iter().enumerate() {
            if index % 3 == 0 {
                world
                    .entity_mut(entity)
                    .insert(FlashEffect::new(showcase_screen_flash_config()));
            }
            if index % 4 == 0 {
                world
                    .entity_mut(entity)
                    .insert(SquashStretchEffect::new(showcase_grounded_squash_config()));
            }
            if index % 6 == 0 {
                world.entity_mut(entity).insert(DissolveEffect::new(DissolveConfig {
                    duration_secs: 0.24,
                    completion: DissolveCompletion::RestoreVisible,
                    ..showcase_hide_dissolve_config()
                }));
            }
        }
    }))
}

pub fn has_proxy_child(world: &World, entity: Entity) -> bool {
    world.get::<Children>(entity).is_some_and(|children| {
        children.iter().any(|child| {
            world
                .get::<Name>(child)
                .is_some_and(|name| name.as_str() == "Sprite Effects Proxy")
        })
    })
}

pub fn proxy_sorts_ahead_of_parent(world: &World, entity: Entity) -> bool {
    let Some(parent_z) = world
        .get::<Transform>(entity)
        .map(|transform| transform.translation.z)
    else {
        return false;
    };

    world.get::<Children>(entity).is_some_and(|children| {
        children.iter().any(|child| {
            world
                .get::<Name>(child)
                .is_some_and(|name| name.as_str() == "Sprite Effects Proxy")
                && world
                    .get::<Transform>(child)
                    .is_some_and(|transform| transform.translation.z > parent_z)
        })
    })
}

pub fn is_hidden(world: &World, entity: Entity) -> bool {
    matches!(world.get::<Visibility>(entity), Some(Visibility::Hidden))
}
