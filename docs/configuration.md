# `saddle-rendering-sprite-effects` Configuration

Core defaults are intentionally neutral. If you want more stylized or game-specific recipes, compose them in your app or example layer instead of relying on semantic presets in the crate API.

## Shared Transient Effect Fields

All transient effects (Flash, Dissolve, Squash/Stretch, Shake) share these fields:

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `delay_secs` | `f32` | `0.0` | Time to wait before the effect starts animating. Useful for staggering effects across multiple entities. |
| `loop_mode` | `LoopMode` | `None` | `None` = play once. `Count(n)` = repeat n times. `Forever` = repeat until component is removed. |
| `persistent` | `bool` | `false` | When `true`, the component stays after completion (with `enabled = false`) instead of being removed. Call `retrigger()` to replay. |
| `overlap` | `OverlapPolicy` | `Restart` | `Restart` = reset timer on re-application. `Ignore` = let current playback finish. |
| `time_domain` | `EffectTimeDomain` | varies | `GlobalScaled` uses `Time<Virtual>`. `Unscaled` uses `Time<Real>`. |

## Time Domain

| Parameter | Default for | Effect | Tuning Guidance |
| --- | --- | --- | --- |
| `GlobalScaled` | dissolve | Uses `Time<Virtual>` | Use for gameplay-authored transitions that should pause or slow down with the game |
| `Unscaled` | flash, squash, shake | Uses `Time<Real>` | Use for impact feedback that should ignore hitstop, pause, or slow motion |

## Flash

`FlashConfig` defaults:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `color` | `Color` | `WHITE` | any color | Flash tint or screen color |
| `intensity` | `f32` | `1.0` | `0.0..=1.0` | Blend amount |
| `duration_secs` | `f32` | `0.12` | `> 0.0` | Lifetime of the flash |
| `delay_secs` | `f32` | `0.0` | `>= 0.0` | Pre-animation delay |
| `easing` | `EaseFunction` | `SineOut` | standard easing | Controls fade-out weight |
| `blend` | `FlashBlendMode` | `Tint` | `Tint` / `Screen` | Native tint path or proxy-backed screen flash |
| `color_ramp` | `Option<Vec<ColorStop>>` | `None` | sorted stops | Multi-stop color transition. Overrides `color` when set. |
| `loop_mode` | `LoopMode` | `None` | enum | Repeat behavior |
| `persistent` | `bool` | `false` | boolean | Keep component after completion |

### Color Ramp

When `color_ramp` is set, the flash color is sampled from the ramp based on effect progress (0.0 → 1.0). Each `ColorStop` has `t: f32` (progress position) and `color: Color`. Colors are linearly interpolated between stops.

Example: `white → yellow → orange → dark red` fire flash:
```rust
color_ramp: Some(vec![
    ColorStop::new(0.0, Color::WHITE),
    ColorStop::new(0.3, Color::srgb(1.0, 0.8, 0.2)),
    ColorStop::new(0.7, Color::srgb(1.0, 0.2, 0.1)),
    ColorStop::new(1.0, Color::srgb(0.3, 0.0, 0.0)),
]),
```

## Dissolve

`DissolveConfig` defaults:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `duration_secs` | `f32` | `0.35` | `> 0.0` | Total dissolve or reveal lifetime |
| `delay_secs` | `f32` | `0.0` | `>= 0.0` | Pre-animation delay |
| `easing` | `EaseFunction` | `SineInOut` | standard easing | Threshold progression curve |
| `pattern` | `DissolvePattern` | `Noise` | enum | Threshold field used for discard |
| `phase` | `DissolvePhase` | `Hide` | `Hide` / `Reveal` | Direction of threshold |
| `edge_width` | `f32` | `0.0` | `0.0..=1.0` | Width of the dissolve frontier band |
| `edge_color` | `Color` | transparent white | any color | Edge tint (used when `edge_gradient` is `None`) |
| `edge_gradient` | `Option<Vec<ColorStop>>` | `None` | sorted stops | Multi-stop edge gradient. Overrides `edge_color` when set. |
| `noise_scale` | `Vec2` | `(24, 24)` | positive | Noise frequency |
| `mask_texture` | `Option<Handle<Image>>` | `None` | grayscale image | Authored dissolve mask (pattern=Mask) |
| `completion` | `DissolveCompletion` | `RestoreVisible` | enum | Post-effect cleanup |
| `loop_mode` | `LoopMode` | `None` | enum | Repeat behavior |
| `persistent` | `bool` | `false` | boolean | Keep component after completion |

### Dissolve Completion

| Variant | Behavior |
| --- | --- |
| `RestoreVisible` | Remove the dissolve state and leave the entity visible |
| `HideEntity` | Set `Visibility::Hidden` after completion |
| `DespawnEntity` | Despawn the entity and its descendants after completion |

### Edge Gradient

Works like color ramp for flash but applied to the dissolve frontier. `t=0` is at the threshold edge, `t=1` is at the outer boundary of the edge band.

## Squash / Stretch

`SquashStretchConfig` defaults:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `amplitude` | `f32` | `0.22` | `0.0..=1.0` | Primary squash amount |
| `rebound` | `f32` | `0.34` | `0.0..=1.0` | Stretch rebound after squash |
| `axis_bias` | `Vec2` | `Y` | non-zero | Deformation axis |
| `preserve_area` | `bool` | `true` | boolean | Cross-axis compensation |
| `compensation_anchor` | `Option<Anchor>` | `None` | any anchor | Translation anchor |
| `duration_secs` | `f32` | `0.20` | `> 0.0` | Total envelope |
| `delay_secs` | `f32` | `0.0` | `>= 0.0` | Pre-animation delay |
| `easing` | `EaseFunction` | `SineOut` | standard easing | Envelope shape |
| `loop_mode` | `LoopMode` | `None` | enum | Repeat behavior |
| `persistent` | `bool` | `false` | boolean | Keep component after completion |

## Shake

`ShakeConfig` defaults:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `amplitude` | `f32` | `4.0` | `> 0.0` | Maximum displacement in pixels |
| `frequency` | `f32` | `30.0` | `> 0.0` | Oscillation frequency in Hz |
| `decay` | `f32` | `0.8` | `0.0..=1.0` | How quickly amplitude decays. `0` = no decay, `1` = fully decayed at end. |
| `axis` | `Vec2` | `(1, 1)` | any | Axis mask. `(1,0)` = horizontal only, `(0,1)` = vertical only |
| `duration_secs` | `f32` | `0.25` | `> 0.0` | Effect lifetime |
| `delay_secs` | `f32` | `0.0` | `>= 0.0` | Pre-animation delay |
| `easing` | `EaseFunction` | `SineOut` | standard easing | Decay envelope curve |
| `loop_mode` | `LoopMode` | `None` | enum | Repeat behavior |
| `persistent` | `bool` | `false` | boolean | Keep component after completion |

### Shake Notes

- Shake uses sine-based displacement, not random noise. This gives smooth, predictable motion.
- X and Y axes oscillate at decorrelated frequencies (golden ratio offset) to avoid diagonal patterns.
- Uses the native transform path (no shader proxy needed).

## Outline

`OutlineConfig` defaults:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `color` | `Color` | `BLACK` | any color | Outline tint |
| `width_pixels` | `f32` | `1.0` | `>= 0.0` | Sampling radius in source-texture texels |
| `alpha_threshold` | `f32` | `0.05` | `0.0..=1.0` | Alpha cutoff |

## Silhouette

`SilhouetteConfig` defaults:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `color` | `Color` | `WHITE` | any color | Tint color |
| `tint_strength` | `f32` | `1.0` | `0.0..=1.0` | Blend amount |
| `alpha_threshold` | `f32` | `0.05` | `0.0..=1.0` | Alpha cutoff |
| `sort_offset` | `f32` | `0.0` | any finite | Proxy Z offset |

## Palette Swap

`PaletteConfig` defaults:

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `texture` | `Handle<Image>` | empty | valid handle | Palette lookup texture |
| `source_row` | `u32` | `0` | `0..height-1` | Row matching source sprite colors |
| `target_row` | `u32` | `1` | `0..height-1` | Row to remap to |
| `columns` | `u32` | `4` | `1..=32` | Number of palette entries per row |
| `epsilon` | `f32` | `0.01` | `>= 0.0` | Color matching distance |
| `preserve_alpha` | `bool` | `true` | boolean | Keep source alpha after match |
| `enforce_nearest_sampling` | `bool` | `true` | boolean | Force nearest sampling on palette texture |

## Messages

| Message | Emitted when |
| --- | --- |
| `SpriteEffectStarted { entity, effect }` | A transient effect begins animating (after delay, on first frame of active playback) |
| `SpriteEffectFinished { entity, effect }` | A transient effect completes all loops (or the single playback) |
