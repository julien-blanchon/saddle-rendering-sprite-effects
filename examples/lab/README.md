# `saddle-rendering-sprite-effects-lab`

Crate-local runtime verification app for `saddle-rendering-sprite-effects`.

## Purpose

- verify the hybrid backend split in one place
- compare native tint flash against proxy-backed screen flash
- exercise dissolve and palette swap with runtime diagnostics visible
- prove atlas animation keeps advancing while proxy effects are active
- keep a 100+ sprite stress scene available for E2E and BRP inspection

## Run

```bash
cargo run -p saddle-rendering-sprite-effects-lab
SPRITE_EFFECTS_LAB_EXIT_AFTER_SECONDS=3 cargo run -p saddle-rendering-sprite-effects-lab
```

## E2E

```bash
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- smoke_launch
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_flash
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_dissolve
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_palette_swap
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_outline_silhouette
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_atlas_animation
cargo run -p saddle-rendering-sprite-effects-lab --features e2e -- sprite_effects_stress
```

## BRP

```bash
BRP_EXTRAS_PORT=15743 cargo run -p saddle-rendering-sprite-effects-lab
BRP_PORT=15743 uv run --active --project .codex/skills/bevy-brp/script brp resource get 'saddle_rendering_sprite_effects::diagnostics::SpriteEffectsDiagnostics'
BRP_PORT=15743 uv run --active --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name saddle_rendering_sprite_effects::components::PaletteSwap
BRP_PORT=15743 uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/sprite_effects_lab.png
```
