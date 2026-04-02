use bevy::app::AppExit;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, DissolvePattern, FlashConfig, FlashEffect, PaletteConfig,
    PaletteSwap, SquashStretchConfig, SquashStretchEffect,
};

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Component)]
pub struct DemoAtlasAnimation {
    pub first: usize,
    pub last: usize,
    pub timer: Timer,
}

#[derive(Resource)]
pub struct PaletteCycle {
    pub rows: Vec<u32>,
    pub timer: Timer,
    pub index: usize,
}

pub fn install_auto_exit(app: &mut App, env_var: &str) {
    let timer = std::env::var(env_var)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .map(|seconds| AutoExitAfter(Timer::from_seconds(seconds.max(0.1), TimerMode::Once)));

    if let Some(timer) = timer {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }
}

pub fn setup_camera(
    commands: &mut Commands,
    title: impl Into<String>,
    subtitle: impl Into<String>,
) {
    commands.spawn((Name::new("Main Camera"), Camera2d));
    commands.spawn((
        Name::new("Backdrop"),
        Sprite::from_color(Color::srgb(0.07, 0.08, 0.11), Vec2::new(1800.0, 1200.0)),
        Transform::from_xyz(0.0, 0.0, -20.0),
    ));
    commands.spawn((
        Name::new("Overlay"),
        Node {
            position_type: PositionType::Absolute,
            left: px(20.0),
            top: px(20.0),
            width: px(430.0),
            padding: UiRect::all(px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.82)),
        children![
            (
                Text::new(title.into()),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ),
            (
                Text::new(subtitle.into()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.86, 0.94)),
                Node {
                    margin: UiRect::top(px(6.0)),
                    ..default()
                },
            ),
        ],
    ));
}

pub fn add_demo_assets(
    images: &mut Assets<Image>,
    atlases: &mut Assets<TextureAtlasLayout>,
) -> DemoAssets {
    let sprite = images.add(slime_image());
    let atlas_image = images.add(slime_atlas_image());
    let mask = images.add(radial_mask_image(48));
    let palette = images.add(palette_image());
    let atlas_layout = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::new(24, 24),
        4,
        1,
        None,
        None,
    ));

    DemoAssets {
        sprite,
        atlas_image,
        mask,
        palette,
        atlas_layout,
    }
}

pub struct DemoAssets {
    pub sprite: Handle<Image>,
    pub atlas_image: Handle<Image>,
    pub mask: Handle<Image>,
    pub palette: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
}

pub fn spawn_showcase_row(commands: &mut Commands, assets: &DemoAssets) {
    commands.spawn((
        Name::new("Tint Flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-280.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        FlashEffect::new(FlashConfig {
            color: Color::srgb(1.0, 0.26, 0.26),
            blend: saddle_rendering_sprite_effects::FlashBlendMode::Tint,
            duration_secs: 0.22,
            ..FlashConfig::default()
        }),
    ));

    commands.spawn((
        Name::new("Screen Flash"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(-90.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        FlashEffect::new(FlashConfig::damage()),
    ));

    commands.spawn((
        Name::new("Palette Swap"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(100.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        PaletteSwap::new(PaletteConfig {
            texture: assets.palette.clone(),
            columns: 4,
            source_row: 0,
            target_row: 1,
            ..PaletteConfig::default()
        }),
    ));

    commands.spawn((
        Name::new("Dissolve"),
        Sprite::from_image(assets.sprite.clone()),
        Transform::from_xyz(290.0, 40.0, 0.0).with_scale(Vec3::splat(6.0)),
        DissolveEffect::new(DissolveConfig {
            duration_secs: 0.8,
            pattern: DissolvePattern::Mask,
            mask_texture: Some(assets.mask.clone()),
            ..DissolveConfig::reveal()
        }),
    ));
}

pub fn spawn_animated_sprite(
    commands: &mut Commands,
    assets: &DemoAssets,
    translation: Vec3,
) -> Entity {
    commands
        .spawn((
            Name::new("Animated Proxy Sprite"),
            Sprite::from_atlas_image(
                assets.atlas_image.clone(),
                TextureAtlas {
                    layout: assets.atlas_layout.clone(),
                    index: 0,
                },
            ),
            Transform::from_translation(translation).with_scale(Vec3::splat(6.0)),
            DemoAtlasAnimation {
                first: 0,
                last: 3,
                timer: Timer::from_seconds(0.10, TimerMode::Repeating),
            },
        ))
        .id()
}

pub fn animate_atlas_sprites(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut DemoAtlasAnimation)>,
) {
    for (mut sprite, mut animation) in &mut query {
        if animation.timer.tick(time.delta()).just_finished()
            && let Some(atlas) = sprite.texture_atlas.as_mut()
        {
            atlas.index = if atlas.index >= animation.last {
                animation.first
            } else {
                atlas.index + 1
            };
        }
    }
}

pub fn cycle_palette_rows(
    time: Res<Time>,
    cycle: Option<ResMut<PaletteCycle>>,
    mut query: Query<&mut PaletteSwap>,
) {
    let Some(mut cycle) = cycle else {
        return;
    };

    if !cycle.timer.tick(time.delta()).just_finished() {
        return;
    }

    cycle.index = (cycle.index + 1) % cycle.rows.len();
    for mut palette in &mut query {
        if palette.enabled {
            palette.config.target_row = cycle.rows[cycle.index];
        }
    }
}

pub fn pulse_effects(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(
        Entity,
        Option<&FlashEffect>,
        Option<&DissolveEffect>,
        Option<&SquashStretchEffect>,
    )>,
) {
    let pulse = ((time.elapsed_secs() * 1.35).sin() * 0.5 + 0.5) > 0.98;
    if !pulse {
        return;
    }

    for (entity, flash, dissolve, squash) in &query {
        if flash.is_none() {
            commands
                .entity(entity)
                .insert(FlashEffect::new(FlashConfig::damage()));
        }
        if dissolve.is_none() {
            commands
                .entity(entity)
                .insert(DissolveEffect::new(DissolveConfig {
                    duration_secs: 0.42,
                    pattern: DissolvePattern::Noise,
                    ..DissolveConfig::hide()
                }));
        }
        if squash.is_none() {
            commands
                .entity(entity)
                .insert(SquashStretchEffect::new(SquashStretchConfig::landing()));
        }
    }
}

pub fn slime_image() -> Image {
    let outline = [28, 37, 42, 255];
    let dark = [59, 116, 74, 255];
    let mid = [99, 187, 101, 255];
    let light = [167, 240, 112, 255];
    let eye = [245, 247, 255, 255];

    let mut pixels = vec![[0, 0, 0, 0]; 24 * 24];
    fill_ellipse(&mut pixels, 24, 12.0, 12.0, 9.0, 8.0, outline);
    fill_ellipse(&mut pixels, 24, 12.0, 12.0, 8.0, 7.0, dark);
    fill_ellipse(&mut pixels, 24, 12.0, 13.0, 6.8, 5.8, mid);
    fill_ellipse(&mut pixels, 24, 11.0, 15.0, 4.8, 3.6, light);
    set_px(&mut pixels, 24, 9, 10, eye);
    set_px(&mut pixels, 24, 14, 10, eye);
    build_image(24, 24, &pixels)
}

pub fn slime_atlas_image() -> Image {
    let mut atlas = vec![[0, 0, 0, 0]; 24 * 4 * 24];
    for frame in 0..4 {
        let frame_image = slime_frame(frame);
        for y in 0..24 {
            for x in 0..24 {
                atlas[y * 96 + frame * 24 + x] = frame_image[y * 24 + x];
            }
        }
    }
    build_image(96, 24, &atlas)
}

pub fn palette_image() -> Image {
    let rows = [
        [
            [28, 37, 42, 255],
            [59, 116, 74, 255],
            [99, 187, 101, 255],
            [167, 240, 112, 255],
        ],
        [
            [31, 25, 37, 255],
            [135, 41, 56, 255],
            [220, 68, 81, 255],
            [255, 188, 122, 255],
        ],
        [
            [22, 28, 47, 255],
            [44, 68, 125, 255],
            [92, 140, 222, 255],
            [196, 222, 255, 255],
        ],
        [
            [28, 23, 41, 255],
            [93, 44, 116, 255],
            [162, 79, 176, 255],
            [242, 178, 255, 255],
        ],
    ];

    let mut pixels = vec![[0, 0, 0, 0]; 4 * 4];
    for (row_index, row) in rows.into_iter().enumerate() {
        for (col_index, color) in row.into_iter().enumerate() {
            pixels[row_index * 4 + col_index] = color;
        }
    }

    let mut image = build_image(4, 4, &pixels);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    image
}

pub fn radial_mask_image(size: u32) -> Image {
    let mut pixels = vec![[0, 0, 0, 255]; (size * size) as usize];
    let center = Vec2::splat((size as f32 - 1.0) * 0.5);
    let max_distance = center.length();

    for y in 0..size {
        for x in 0..size {
            let delta = Vec2::new(x as f32, y as f32) - center;
            let distance = (delta.length() / max_distance).clamp(0.0, 1.0);
            let value = (distance * 255.0) as u8;
            pixels[(y * size + x) as usize] = [value, value, value, 255];
        }
    }

    build_image(size, size, &pixels)
}

fn slime_frame(frame: usize) -> Vec<[u8; 4]> {
    let outline = [28, 37, 42, 255];
    let dark = [59, 116, 74, 255];
    let mid = [99, 187, 101, 255];
    let light = [167, 240, 112, 255];
    let eye = [245, 247, 255, 255];

    let mut pixels = vec![[0, 0, 0, 0]; 24 * 24];
    let wobble: (f32, f32) = match frame {
        0 => (0.0, 0.0),
        1 => (0.5, 1.0),
        2 => (0.0, 2.0),
        _ => (-0.5, 1.0),
    };
    fill_ellipse(
        &mut pixels,
        24,
        12.0,
        12.0 + wobble.1,
        9.0 + wobble.0.abs(),
        8.0 - wobble.0.abs() * 0.4,
        outline,
    );
    fill_ellipse(
        &mut pixels,
        24,
        12.0,
        12.0 + wobble.1,
        8.0 + wobble.0.abs(),
        7.0,
        dark,
    );
    fill_ellipse(&mut pixels, 24, 12.0, 13.0 + wobble.1, 6.8, 5.8, mid);
    fill_ellipse(&mut pixels, 24, 11.0, 15.0 + wobble.1, 4.8, 3.6, light);
    set_px(&mut pixels, 24, 9, 10 + wobble.1 as usize, eye);
    set_px(&mut pixels, 24, 14, 10 + wobble.1 as usize, eye);
    pixels
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}

fn build_image(width: u32, height: u32, pixels: &[[u8; 4]]) -> Image {
    let bytes = pixels
        .iter()
        .flat_map(|pixel| pixel.iter().copied())
        .collect::<Vec<_>>();
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn fill_ellipse(
    pixels: &mut [[u8; 4]],
    width: usize,
    center_x: f32,
    center_y: f32,
    radius_x: f32,
    radius_y: f32,
    color: [u8; 4],
) {
    for y in 0..(pixels.len() / width) {
        for x in 0..width {
            let dx = (x as f32 - center_x) / radius_x.max(0.001);
            let dy = (y as f32 - center_y) / radius_y.max(0.001);
            if dx * dx + dy * dy <= 1.0 {
                pixels[y * width + x] = color;
            }
        }
    }
}

fn set_px(pixels: &mut [[u8; 4]], width: usize, x: usize, y: usize, color: [u8; 4]) {
    if x < width && y < pixels.len() / width {
        pixels[y * width + x] = color;
    }
}
