use bevy::prelude::*;
use saddle_rendering_sprite_effects::PaletteSwap;

pub fn overlay_text(world: &World) -> Option<String> {
    let overlay = world.get_resource::<crate::LabEntities>()?.overlay;
    Some(world.get::<Text>(overlay)?.0.clone())
}

pub fn palette_target_row(world: &World) -> Option<u32> {
    let target = world.get_resource::<crate::LabEntities>()?.palette_target;
    Some(world.get::<PaletteSwap>(target)?.config.target_row)
}

pub fn atlas_index(world: &World) -> Option<usize> {
    let target = world.get_resource::<crate::LabEntities>()?.atlas_target;
    world
        .get::<Sprite>(target)?
        .texture_atlas
        .as_ref()
        .map(|atlas| atlas.index)
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
    let Some(parent_z) = world.get::<Transform>(entity).map(|transform| transform.translation.z)
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
