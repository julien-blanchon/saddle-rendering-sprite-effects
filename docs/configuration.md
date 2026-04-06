# `saddle-rendering-sprite-effects` Configuration

Core defaults are intentionally neutral. If you want more stylized or game-specific recipes, compose them in your app or example layer instead of relying on semantic presets in the crate API.

## Time Domain

| Parameter | Type | Default | Valid Range | Effect | Tuning Guidance |
| --- | --- | --- | --- | --- | --- |
| `EffectTimeDomain::GlobalScaled` | enum | default for dissolve | fixed enum | Uses `Time<Virtual>` | Use for gameplay-authored transitions that should pause or slow down with the game |
| `EffectTimeDomain::Unscaled` | enum | default for flash and squash | fixed enum | Uses `Time<Real>` | Use for impact feedback that should ignore hitstop, pause, or slow motion |

## Outline

`OutlineConfig` defaults:

| Field | Type | Default | Valid Range | Effect | Tuning Guidance |
| --- | --- | --- | --- | --- | --- |
| `color` | `Color` | `BLACK` | any color | Outline tint written around the sprite edge | Black is the safest default; bright colors work well for interactable or hostile highlighting |
| `width_pixels` | `f32` | `1.0` | `>= 0.0` | Neighbor sampling radius in source-texture texels | `1.0..2.0` is the sweet spot for pixel art; higher values quickly read as a glow |
| `alpha_threshold` | `f32` | `0.05` | `0.0..=1.0` | Alpha cutoff used for both source pixels and candidate outline neighbors | Raise it if soft alpha fringe is making the outline look fuzzy |

### Outline Notes

- Width is measured in source-texture texels, not world units or screen pixels.
- The outline is generated from the current atlas frame bounds, so atlas animations stay frame-local.
- Outline only draws outside the authored sprite body; it does not recolor interior opaque pixels.

## Silhouette

`SilhouetteConfig` defaults:

| Field | Type | Default | Valid Range | Effect | Tuning Guidance |
| --- | --- | --- | --- | --- | --- |
| `color` | `Color` | `WHITE` | any color | Tint color applied across the sprite body | Neutral white is the least opinionated baseline; use hue plus alpha when you want a more stylized readability pass |
| `tint_strength` | `f32` | `1.0` | `0.0..=1.0` | Blend amount between source RGB and silhouette RGB | `0.35..0.65` reads well for readability/x-ray presentation; `1.0` gives a full silhouette card |
| `alpha_threshold` | `f32` | `0.05` | `0.0..=1.0` | Alpha cutoff for which sprite pixels participate | Raise it to ignore soft edge pixels on painted sprites |
| `sort_offset` | `f32` | `0.0` | any finite value | Local proxy Z offset relative to the authored sprite | Leave it at `0.0` when you do not want the proxy to bias sorting; use small positive or negative values only when you need deliberate layering |

### Silhouette Notes

- Silhouette only affects source pixels whose alpha exceeds the configured threshold.
- `sort_offset` is purely a render-order nudge for the proxy child. It does not perform blocker detection by itself.
- Silhouette composes with outline, dissolve, palette swap, and screen flash because all of them share the same proxy material path.

## Flash

`FlashConfig` defaults:

| Field | Type | Default | Valid Range | Effect | Tuning Guidance |
| --- | --- | --- | --- | --- | --- |
| `color` | `Color` | `WHITE` | any color | Flash tint or screen color | White is the highest-contrast baseline; colored flashes are useful when you want a themed accent instead of neutral emphasis |
| `intensity` | `f32` | `1.0` | `0.0..=1.0` recommended | Blend amount | `0.35..0.6` is usually enough for light emphasis; reserve `1.0` for very strong flashes |
| `duration_secs` | `f32` | `0.12` | `> 0.0` | Lifetime of the flash | `0.08..0.16` reads as punchy; longer flashes start to feel like a temporary recolor |
| `easing` | `EaseFunction` | `SineOut` | standard Bevy easing enum | Controls fade-out weight | `SineOut` and `CubicOut` give readable snap without an abrupt cutoff |
| `blend` | `FlashBlendMode` | `Tint` | `Tint` or `Screen` | Native tint path or proxy-backed screen flash | Use `Tint` when you want the cheapest path; use `Screen` when you want additive-style punch |
| `overlap` | `FlashOverlap` | `Refresh` | enum | Restart policy when reauthored | Both current variants restart immediately; keep `Refresh` unless you need to signal stronger semantic intent in your own code |
| `time_domain` | `EffectTimeDomain` | `Unscaled` | enum | Scaled vs real-time lifetime | Keep short feedback flashes unscaled unless you intentionally want them frozen by hitstop |

## Dissolve

`DissolveConfig` defaults:

| Field | Type | Default | Valid Range | Effect | Tuning Guidance |
| --- | --- | --- | --- | --- | --- |
| `duration_secs` | `f32` | `0.35` | `> 0.0` | Total dissolve or reveal lifetime | `0.25..0.45` works for brisk transitions; slower values start reading as wipes or cinematic reveals |
| `easing` | `EaseFunction` | `SineInOut` | standard easing enum | Threshold progression curve | Use `SineInOut` or `SmoothStep` for neutral transitions; use `CubicIn` for aggressive disappearances |
| `pattern` | `DissolvePattern` | `Noise` | enum | Threshold field used for discard | `Noise` is the most general-purpose; directional modes work better for wipes or stealth reveals |
| `phase` | `DissolvePhase` | `Hide` | `Hide` or `Reveal` | Whether threshold moves toward disappearance or appearance | Use `Reveal` for spawns, teleport arrivals, or stealth decloak |
| `overlap` | `DissolveOverlap` | `Replace` | enum | Restart policy | Current behavior restarts the authored dissolve immediately |
| `time_domain` | `EffectTimeDomain` | `GlobalScaled` | enum | Scaled vs real-time lifetime | Keep cinematic or gameplay-state transitions scaled by default |
| `edge_width` | `f32` | `0.0` | `0.0..=1.0` practical | Width of the dissolve frontier band | Keep it at `0.0` for a plain dissolve; `0.03..0.10` adds a readable edge without turning into a soft glow |
| `edge_color` | `Color` | transparent white | any color | Edge tint mixed onto the dissolve frontier | Increase alpha to make the frontier visible; choose the hue only after you know what kind of stylization you want |
| `noise_scale` | `Vec2` | `Vec2::splat(24.0)` | positive values | Frequency used by the procedural noise pattern | Higher values give finer grain; lower values give chunkier breakup |
| `mask_texture` | `Option<Handle<Image>>` | `None` | any grayscale-compatible image | Optional authored dissolve mask | Only used when `pattern = Mask`; author masks in sprite-local UV space |
| `completion` | `DissolveCompletion` | `RestoreVisible` | enum | Post-effect cleanup behavior | Use `HideEntity` for temporary disappearances and `DespawnEntity` when the effect owns the entity lifetime |

### Dissolve Completion

| Variant | Behavior |
| --- | --- |
| `RestoreVisible` | Remove the dissolve state and leave the entity visible |
| `HideEntity` | Set `Visibility::Hidden` after completion |
| `DespawnEntity` | Despawn the entity and its descendants after completion |

## Squash / Stretch

`SquashStretchConfig` defaults:

| Field | Type | Default | Valid Range | Effect | Tuning Guidance |
| --- | --- | --- | --- | --- | --- |
| `amplitude` | `f32` | `0.22` | `0.0..=1.0` practical | Primary squash amount | `0.12..0.25` is a good responsive range before the deformation starts reading as very cartoony |
| `rebound` | `f32` | `0.34` | `0.0..=1.0` practical | Stretch rebound after the initial squash | `0.15..0.35` feels natural; higher values become cartoony quickly |
| `axis_bias` | `Vec2` | `Vec2::Y` | any non-zero vector recommended | Which axis receives the primary squash/stretch | `Vec2::Y` is a neutral baseline for upright sprites; switch to another axis when you want the deformation to read along motion or impact direction |
| `preserve_area` | `bool` | `true` | boolean | Whether the cross-axis compensates to preserve apparent volume | Keep enabled for characters and props; disable for intentionally “puffed” or “squeezed” stylization |
| `compensation_anchor` | `Option<Anchor>` | `None` | any Bevy anchor or `None` | Translation compensation target | Leave it `None` for an unanchored envelope; set a specific anchor only when the deformation must stay planted against a visual base |
| `duration_secs` | `f32` | `0.20` | `> 0.0` | Total envelope duration | `0.12..0.24` is the useful range for responsive feedback |
| `easing` | `EaseFunction` | `SineOut` | standard easing enum | Envelope shape | `SineOut` gives soft recovery; `CubicOut` reads snappier |
| `overlap` | `SquashOverlap` | `Refresh` | enum | Restart policy | Current behavior restarts immediately |
| `time_domain` | `EffectTimeDomain` | `Unscaled` | enum | Scaled vs real-time lifetime | Keep short deformation feedback unscaled when using hitstop |

### Area Preservation

- `preserve_area = true` keeps the cross-axis compensating inversely, which is the safer default for readable characters.
- `preserve_area = false` lets the cross-axis compress more loosely, which can feel softer or more gelatinous.

### Anchor Compensation

- Compensation is based on the difference between the sprite's current `Anchor` and `compensation_anchor`.
- Use it when you need the deformation to stay planted against a particular edge; leave it unset for free-floating squash/stretch.

## Palette Swap

`PaletteConfig` defaults:

| Field | Type | Default | Valid Range | Effect | Tuning Guidance |
| --- | --- | --- | --- | --- | --- |
| `texture` | `Handle<Image>` | empty handle | valid image handle required | Palette lookup texture | Author rows as palette banks; keep sampling nearest |
| `source_row` | `u32` | `0` | `0..height-1` | Row that matches the source sprite colors | Usually your neutral or authored palette row |
| `target_row` | `u32` | `1` | `0..height-1` | Row to remap to at runtime | Switch rows for factions, skins, or status looks |
| `columns` | `u32` | `4` | `1..=32` practical | Number of palette entries in each row | Keep this equal to the number of authored swatches you expect to match |
| `epsilon` | `f32` | `0.01` | `>= 0.0` | Allowed distance for matching source colors | Increase slightly if your source art is filtered or tinted before lookup |
| `preserve_alpha` | `bool` | `true` | boolean | If `true`, keep the source sprite alpha after a palette match; if `false`, use the target palette row alpha | Keep enabled for most pixel-art and authored-opacity sprites; disable it only when palette banks intentionally encode different opacity |
| `enforce_nearest_sampling` | `bool` | `true` | boolean | Forces the palette texture to nearest sampling | Leave enabled for exact lookup; disable only if you fully control the asset's sampler externally |

### Palette Texture Layout Expectations

- rows are palette banks
- columns are swatches
- the shader currently iterates up to 32 columns
- source and target rows are sampled at texel centers
- if `texture` is the default/empty handle, the crate keeps the authored `PaletteSwap` component but does not create a shader proxy for that sprite.

For pixel-art pipelines:

- use nearest sampling on the source art too
- avoid colors that drift off the authored palette before lookup
- keep alpha premultiplication or post-process recolor outside the palette path if exact lookup matters
