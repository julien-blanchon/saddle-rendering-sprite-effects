use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

use crate::SPRITE_EFFECTS_SHADER_HANDLE;

#[derive(Resource, Default)]
pub(crate) struct SpriteEffectsInternalAssets {
    pub quad_mesh: Handle<Mesh>,
}

#[derive(Clone, Debug, ShaderType)]
pub(crate) struct SpriteEffectsUniform {
    pub base_color: Vec4,
    pub flash_color: Vec4,
    pub edge_color: Vec4,
    pub outline_color: Vec4,
    pub silhouette_color: Vec4,
    pub uv_rect: Vec4,
    pub flash: Vec4,
    pub dissolve: Vec4,
    pub dissolve_aux: Vec4,
    pub outline: Vec4,
    pub silhouette: Vec4,
    pub palette: Vec4,
    pub flags: Vec4,
}

impl Default for SpriteEffectsUniform {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            flash_color: Vec4::ZERO,
            edge_color: Vec4::ZERO,
            outline_color: Vec4::ZERO,
            silhouette_color: Vec4::ZERO,
            uv_rect: Vec4::new(0.0, 0.0, 1.0, 1.0),
            flash: Vec4::ZERO,
            dissolve: Vec4::ZERO,
            dissolve_aux: Vec4::ZERO,
            outline: Vec4::ZERO,
            silhouette: Vec4::ZERO,
            palette: Vec4::ZERO,
            flags: Vec4::ZERO,
        }
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub(crate) struct SpriteEffectsMaterial {
    #[uniform(0)]
    pub uniform: SpriteEffectsUniform,
    #[texture(1)]
    #[sampler(2)]
    pub source_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub palette_texture: Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    pub mask_texture: Option<Handle<Image>>,
}

impl Material2d for SpriteEffectsMaterial {
    fn fragment_shader() -> ShaderRef {
        SPRITE_EFFECTS_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
