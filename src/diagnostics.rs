use bevy::prelude::*;

#[derive(Resource, Reflect, Clone, Debug, Default, PartialEq)]
#[reflect(Resource, Default)]
pub struct SpriteEffectsDiagnostics {
    pub active_flashes: usize,
    pub active_dissolves: usize,
    pub active_squashes: usize,
    pub active_palette_swaps: usize,
    pub active_outlines: usize,
    pub active_silhouettes: usize,
    pub active_shader_proxies: usize,
}
