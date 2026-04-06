# `saddle-rendering-sprite-effects` Architecture

## Backend Choice

`saddle-rendering-sprite-effects` uses the hybrid backend the brief recommended.

- Native `Sprite` + `Transform` path:
  - tint flash
  - squash/stretch
- Internal `Mesh2d` + `Material2d` proxy path:
  - dissolve
  - palette swap
  - screen-style flash
  - alpha-edge outline
  - silhouette tint

Why this split:

- flash tint and squash/stretch are cheap and do not need a shader
- dissolve, palette remap, outline, and silhouette are fundamentally per-pixel operations
- projects that only need hit flash and squash do not pay for a material-backed path
- projects that do need shader effects still trigger them through the same authored component surface

The crate keeps those authored components low-level and parameter-driven. Gameplay- or demo-specific recipes belong in host code or example helpers, not in the core config API.

## Runtime Flow

`SpriteEffectsSystems` are ordered as:

1. `Prepare`
2. `TickCpuEffects`
3. `UpdateMaterials`
4. `Cleanup`
5. `Diagnostics`

### `Prepare`

- restore authored sprite color if a native tint flash or proxy hid it last frame
- restore authored transform translation and scale if squash/stretch modified it last frame
- lazily create the shared quad mesh used by proxy children
- force nearest sampling on palette textures when requested

### `TickCpuEffects`

- advance flash, dissolve, and squash/stretch lifetimes
- write `SpriteEffectFinished` messages when temporary channels end
- apply native tint flash when the entity does not need a shader proxy
- apply transform deformation for squash/stretch

### `UpdateMaterials`

- decide whether each entity needs the proxy child this frame
- spawn or despawn the proxy child explicitly
- sync atlas-aware UV bounds, authored tint/alpha, flash data, dissolve data, palette data, outline data, and silhouette data into the proxy material
- hide the authored sprite by alpha while a proxy is active

### `Cleanup`

- on runtime deactivation, remove runtime-only state
- restore authored sprite color and transform before dropping the runtime-owned state
- despawn proxy children explicitly

The public authored components are intentionally preserved during deactivate. This keeps schedule gating compatible with persistent authored state such as `PaletteSwap`, and lets a host re-activate the plugin without reconstructing its own config components.

### `Diagnostics`

- publish `SpriteEffectsDiagnostics`

## State Ownership

The authored components are public. Runtime-only state stays internal:

- `FlashRuntime`, `DissolveRuntime`, `SquashRuntime`
- `PresentedSpriteState`
- `PresentedTransformState`
- `ShaderProxy`

This split matters because restoration has a single source of truth:

- sprite color and alpha restoration come from `PresentedSpriteState`
- transform translation and scale restoration come from `PresentedTransformState`
- proxy material tint and alpha are copied from the authored sprite color before the authored sprite is hidden

The crate never mutates authored effect config to drive progression.

When the runtime is deactivated, the crate drops only internal progression and presentation state. Public effect components remain the source of truth.

## Overlap And Conflict Policy

Each entity gets one authored component slot per effect family.

- flash: restarting the component restarts the flash timer
- dissolve: restarting the component replaces the dissolve state immediately
- squash/stretch: restarting the component restarts the envelope immediately
- palette swap: changing the config updates the proxy material next frame
- outline: changing the config updates the proxy material next frame
- silhouette: changing the config updates the proxy material next frame

The crate intentionally does not queue or blend multiple authored components of the same family on one entity. That policy would require a different API surface than “component is the source of truth”.

## Atlas Compatibility

The proxy material tracks two UV spaces:

- local effect UVs in `[0, 1]`
- authored source texture UV rects for the atlas frame

Source sampling uses the atlas rect, while dissolve masks and directional patterns use local frame UVs. That keeps:

- left-to-right and radial dissolve behavior consistent across atlas frames
- mask textures authored in sprite-local UV space
- outlines confined to the current frame instead of bleeding across the full sheet
- silhouettes confined to the current frame's opaque pixels
- atlas animation compatible with proxy-backed effects

## Pixel-Art Constraints

- exact palette remap expects nearest sampling on the palette texture
- source textures should also use nearest sampling for pixel-art projects to avoid color bleed before lookup
- palette remap can either preserve source alpha or take alpha from the target palette row
- proxy sampling clamps to the current source rect with a half-texel inset to reduce atlas-edge bleed

## Time Model

The crate intentionally separates scaled and unscaled time:

- `EffectTimeDomain::GlobalScaled` uses `Time<Virtual>`
- `EffectTimeDomain::Unscaled` uses `Time<Real>`

This makes it valid to keep short flashes or squash/stretch envelopes alive through pause/hitstop while allowing dissolves to respect game-time scaling.

## Performance Model

Cheap path:

- one `Sprite` mutation for tint flash
- one `Transform` mutation for squash/stretch
- no material allocation for entities that stay on the native path

Shader path:

- one proxy child per active proxied entity
- one material asset per proxy child
- one shared quad mesh for all proxy children
- no per-frame mesh or material allocation after the proxy exists

Tradeoff:

- persistent palette swaps, outlines, or silhouettes imply persistent proxy materials
- this is acceptable for moderate counts and predictable for stress scenes
- if a project wants palette remap on thousands of sprites at once, it should budget for that explicitly rather than expecting zero-cost recolor
