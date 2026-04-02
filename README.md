# Saddle Rendering Sprite Effects

Reusable sprite feedback effects for Bevy 0.18 2D games. The crate ships a deliberate hybrid backend: cheap native `Sprite` and `Transform` mutation for flash and squash/stretch, plus an internal `Material2d` proxy path for dissolve, palette swap, and screen-style flash.

## Quick Start

```toml
saddle-rendering-sprite-effects = { git = "https://github.com/julien-blanchon/saddle-rendering-sprite-effects" }
```

```rust,no_run
use bevy::prelude::*;
use saddle_rendering_sprite_effects::{
    DissolveConfig, DissolveEffect, FlashConfig, FlashEffect, PaletteConfig, PaletteSwap,
    SpriteEffectsPlugin, SquashStretchConfig, SquashStretchEffect,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(SpriteEffectsPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let sprite = asset_server.load("player.png");
    let palette = asset_server.load("palettes/teams.png");

    commands.spawn((
        Name::new("Effect Target"),
        Sprite::from_image(sprite),
        FlashEffect::new(FlashConfig::damage()),
        SquashStretchEffect::new(SquashStretchConfig::landing()),
        PaletteSwap::new(PaletteConfig::new(palette, 4)),
        DissolveEffect::new(DissolveConfig::reveal()),
    ));
}
```

Add, remove, or mutate the public effect components directly. Each channel owns its own lifetime and cleans up the temporary runtime state it created.

## Plugin Constructor

`SpriteEffectsPlugin::new(activate_schedule, deactivate_schedule, update_schedule)` accepts injected schedules so host apps can decide when the runtime exists. `SpriteEffectsPlugin::default()` is always-on and uses `Update`.

If you map the crate into a larger game pipeline, order against the public sets instead of private systems:

```rust,no_run
# use bevy::prelude::*;
# use saddle_rendering_sprite_effects::{SpriteEffectsPlugin, SpriteEffectsSystems};
# #[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
# enum Screen { #[default] Gameplay }
# #[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
# enum GameSet { Presentation }
App::new()
    .add_plugins(SpriteEffectsPlugin::new(
        OnEnter(Screen::Gameplay),
        OnExit(Screen::Gameplay),
        Update,
    ))
    .configure_sets(
        Update,
        SpriteEffectsSystems::UpdateMaterials.in_set(GameSet::Presentation),
    );
```

## Public API

| Type | Purpose |
| --- | --- |
| `SpriteEffectsPlugin` | Registers the runtime, proxy material path, diagnostics, and cleanup |
| `SpriteEffectsSystems` | Public ordering hooks: `Prepare`, `TickCpuEffects`, `UpdateMaterials`, `Cleanup`, `Diagnostics` |
| `FlashEffect` / `FlashConfig` | Tint or screen-style flash with configurable duration, easing, and time domain |
| `DissolveEffect` / `DissolveConfig` | Noise, directional, radial, or mask-backed dissolve and reveal |
| `SquashStretchEffect` / `SquashStretchConfig` | Transform-driven squash/stretch with area preservation and anchor compensation |
| `PaletteSwap` / `PaletteConfig` | Exact palette-bank remap using a row-based lookup texture |
| `SpriteEffectFinished` | Optional completion message for flash, dissolve, and squash/stretch |
| `SpriteEffectsDiagnostics` | Runtime counts for active channels and proxy usage |

## Backend Model

The crate keeps the public API backend-agnostic, but the implementation is intentionally not a monolithic shader:

- `FlashEffect` with `FlashBlendMode::Tint` stays on the native `Sprite.color` path when no shader-only effect is active on that entity.
- `SquashStretchEffect` always stays on the CPU transform path.
- `DissolveEffect`, `PaletteSwap`, and `FlashEffect` with `FlashBlendMode::Screen` create an internal proxy child using `Mesh2d` + `MeshMaterial2d`.
- If a shader proxy exists, the authored sprite is hidden by alpha while its original tint and alpha are copied into the proxy material.

This keeps the common hit-flash and squash cases cheap, while still supporting per-pixel effects without forcing every sprite onto a material-backed render path.

Deactivate schedules clear runtime-owned state only. The public authored components stay attached, so persistent channels such as `PaletteSwap` can resume cleanly after the host re-activates the plugin.

## Overlap Policy

The crate owns one active component channel per effect family on an entity.

| Channel | Policy |
| --- | --- |
| `FlashEffect` | Reapplying the component immediately restarts the flash timer |
| `DissolveEffect` | Reapplying the component immediately replaces the active dissolve state |
| `SquashStretchEffect` | Reapplying the component immediately restarts the squash/stretch envelope |
| `PaletteSwap` | Persistent authored state; changing the config updates the proxy material on the next frame |

`Refresh` and `Replace` are both exposed in the API today, but with the current component-driven trigger model they both resolve to an immediate restart of the authored channel. The distinction is preserved for forward-compatible expansion without changing the public types.

## Time Policy

- `EffectTimeDomain::GlobalScaled` reads `Time<Virtual>`
- `EffectTimeDomain::Unscaled` reads `Time<Real>`

This means flashes and squash/stretch can ignore hitstop or pause while dissolves can choose to respect it.

## Atlas And Pixel-Art Notes

- Atlas animation is supported. The proxy material samples the authored sprite's current atlas rect every frame, and dissolve patterns use local frame UVs so atlas effects stay frame-local.
- Palette lookup expects a row-based texture: one source row, one target row, and `columns <= 32`.
- Exact palette matching relies on nearest sampling. Set `PaletteConfig::enforce_nearest_sampling = true` and prefer `ImagePlugin::default_nearest()` for pixel-art projects.
- Alpha is preserved during palette remap and when the authored sprite is hidden behind a proxy.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal hybrid showcase | `cargo run -p saddle-rendering-sprite-effects --example basic` |
| `flash` | Native tint flash versus proxy-backed screen flash | `cargo run -p saddle-rendering-sprite-effects --example flash` |
| `dissolve` | Noise, radial, and authored-mask dissolves | `cargo run -p saddle-rendering-sprite-effects --example dissolve` |
| `palette_swap` | Palette-bank cycling for team/status variants | `cargo run -p saddle-rendering-sprite-effects --example palette_swap` |
| `atlas_animation` | Atlas animation compatibility while proxy effects are active | `cargo run -p saddle-rendering-sprite-effects --example atlas_animation` |
| `stress` | Dense proxy and effect lifetime stress pass | `cargo run -p saddle-rendering-sprite-effects --example stress` |

## Crate-Local Lab

The richer verification target lives at `shared/rendering/saddle-rendering-sprite-effects/examples/lab`:

```bash
cargo run -p saddle-rendering-sprite-effects-lab
```

Focused E2E scenarios:

```bash
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- smoke_launch
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_flash
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_dissolve
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_palette_swap
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_atlas_animation
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_stress
```

## Limitations And Tradeoffs

- One proxy material is allocated per proxied sprite. This avoids per-frame churn, but very large numbers of permanent palette swaps still imply one material per entity.
- Transient proxy-only effects tear their proxy down when the entity returns to the cheap path. Re-triggering that shader path later allocates a fresh proxy material for that entity.
- Palette lookup is exact-match oriented. If the source art is filtered or heavily tinted before lookup, color matching can fail unless the palette epsilon is widened.
- `PaletteConfig::columns` is practically capped at 32 by the current shader loop.
- The crate does not queue effect requests internally. If you need queued or blended effect recipes, layer that policy above the component API or use `game_feel` for higher-level orchestration.

More detail lives in [architecture.md](docs/architecture.md) and [configuration.md](docs/configuration.md).
