# Long-term Pointer Input Architecture

## Context

The current setup ([common/mouse_interaction.rs](rust/bevy_experiment/src/common/mouse_interaction.rs)) classifies left/right mouse buttons into a 4-state FSM (`Idle` / `PendingClassification` / `Dragging` / `Clicked(Vec2)`) inside a `MouseInteractionState` resource. Two consumer modules poll that resource in `PreUpdate` and emit their own typed messages: [orbit_camera/events.rs](rust/bevy_experiment/src/orbit_camera/events.rs) reacts to `Dragging` for pan/orbit, and [polygon_tool/events.rs](rust/bevy_experiment/src/polygon_tool/events.rs) reacts to the single-frame `Clicked(Vec2)` for add/remove control points. The `PreUpdate`-mapping → `Update`-controller split is solid.

The design has three concrete weaknesses that will get worse as features grow:

1. **`Clicked(Vec2)` is an event masquerading as state.** The FSM manually clears it next frame; consumers must run before the clear. It works because of system ordering, but it's the exact pattern Bevy's messages exist for.
2. **No room for modifiers, double-click, or hover.** Each addition requires extending the FSM and the consumer's polling logic. There's no concept of "this gesture started with Shift held," which matters because mid-drag modifier changes shouldn't reclassify a gesture.
3. **No tool exclusivity.** Today, polygon\_tool only acts on `Clicked` and orbit\_camera only acts on `Dragging`, so they don't collide by accident. The moment we add box-select (also drag) or zoom-on-double-click (also click), there's nothing in the architecture to mediate.

**The user wants:** full long-term design + migration path, with a **modal** tool model (explicit `ActiveTool` resource routes left-button input to one tool at a time; camera-secondary inputs like right-drag orbit and scroll-zoom are always-on).

## Recommended Architecture

Four layers, top to bottom, each emitting messages consumed by the next:

```
[ Raw input ]   bevy::input    ButtonInput, MouseMotion, MouseWheel, CursorMoved, KeyCode
       ↓
[ Layer 1 ]     pointer_input  PointerPress / PointerRelease / PointerClick / PointerDoubleClick
                               PointerDragStart / PointerDrag / PointerDragEnd / PointerMove
                               + private FSM resource, public read-only PointerState
       ↓
[ Layer 2 ]     active_tool    ActiveTool resource (Pan, Polygon, BoxSelect, ...)
                               run_if filters gate tool-specific input mappers
       ↓
[ Layer 3 ]     <tool>/events  PolygonToolInputEvent, OrbitCameraInputEvent, BoxSelectInputEvent...
       ↓
[ Layer 4 ]     <tool>/controller  Apply state changes (existing pattern)
```

### Layer 1 — `common/pointer_input` (rename + expand `mouse_interaction`)

The gesture detector. **All output is messages**; the FSM resource becomes private.

**Messages** (all derive `Message`, all emitted in `PreUpdate`):

```rust
#[derive(Clone, Copy, Debug)]
pub enum PointerButton { Left, Right, Middle }

#[derive(Clone, Copy, Debug, Default)]
pub struct ModifierKeys { pub shift: bool, pub ctrl: bool, pub alt: bool, pub meta: bool }

// Snapshotted at button-down and frozen for the gesture's lifetime.

#[derive(Message)] pub struct PointerPress       { pub button: PointerButton, pub position: Vec2, pub modifiers: ModifierKeys }
#[derive(Message)] pub struct PointerRelease     { pub button: PointerButton, pub position: Vec2, pub modifiers: ModifierKeys }
#[derive(Message)] pub struct PointerClick       { pub button: PointerButton, pub position: Vec2, pub modifiers: ModifierKeys }
#[derive(Message)] pub struct PointerDoubleClick { pub button: PointerButton, pub position: Vec2, pub modifiers: ModifierKeys }
#[derive(Message)] pub struct PointerDragStart   { pub button: PointerButton, pub origin: Vec2, pub position: Vec2, pub modifiers: ModifierKeys }
#[derive(Message)] pub struct PointerDrag        { pub button: PointerButton, pub origin: Vec2, pub position: Vec2, pub delta: Vec2, pub modifiers: ModifierKeys }
#[derive(Message)] pub struct PointerDragEnd     { pub button: PointerButton, pub origin: Vec2, pub position: Vec2, pub modifiers: ModifierKeys }
#[derive(Message)] pub struct PointerMove        { pub position: Vec2, pub delta: Vec2, pub modifiers: ModifierKeys }
```

**Public read-only state** (a thin queryable façade over the FSM, for UI feedback that genuinely needs polling — e.g., showing a "currently panning" cursor):

```rust
#[derive(Resource, Default)]
pub struct PointerState {
    pub position: Option<Vec2>,
    pub modifiers: ModifierKeys,
    pub left:   PointerButtonStatus,   // Idle | Pressed | Dragging { origin, modifiers }
    pub right:  PointerButtonStatus,
    pub middle: PointerButtonStatus,
}
```

Tools should default to consuming **messages**; only reach for `PointerState` when polling current-frame state is genuinely simpler than tracking it via Start/End messages.

**Internal FSM** lives in a `pub(crate)` resource owned by the plugin; the FSM owns timing (last-click timestamp + position per button for double-click) and modifier snapshotting. Click/double-click thresholds: ~300 ms, ~5 px, modifiers must match between the two clicks.

**Double-click semantics:** emit `PointerClick` immediately on every qualifying release, then **also** emit `PointerDoubleClick` when the second click qualifies. Tools must make their click handlers compatible with a follow-up double-click, OR use modifiers/different buttons to disambiguate. (E.g., add-marker-on-click + zoom-on-double-click on empty globe is a real conflict — you'd resolve it by making double-click re-route through the active tool.)

**Modifier capture:** snapshot at `PointerPress`, freeze for the lifetime of that gesture. Subsequent `PointerDrag` events carry the press-time modifiers, **not** the current frame's modifiers. This matches user expectation (start panning, then press Shift — you stay in pan, not box-select). `PointerState.modifiers` always reflects the *current* frame, since polling consumers want live state.

### Layer 2 — `common/active_tool`

```rust
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveTool {
    #[default] Pan,
    Polygon,
    BoxSelect,
    // future: Measure, Path, ...
}
```

Plus a system condition helper:

```rust
pub fn active_tool_is(tool: ActiveTool) -> impl Condition<()> { /* run_if */ }
```

Tool-specific mapper systems are registered with `.run_if(active_tool_is(ActiveTool::Polygon))` so they only fire when their tool is active. A small `tool_switch_input` system reads `KeyCode` (e.g., `V`=Pan, `P`=Polygon, `B`=BoxSelect) and writes to `ActiveTool`. UI for tool switching can come later.

**Camera-secondary inputs are not gated.** Right-button drag (orbit) and scroll wheel (zoom) are always-active across all tools — they're navigation, not editing. Only **left**-button gestures are routed by `ActiveTool`. The orbit camera mapper splits in two: a modeless half (right-drag, scroll) and an `ActiveTool::Pan`-gated half (left-drag).

### Layer 3 — Tool input mappers (existing pattern, lightly evolved)

Each tool's `events.rs` becomes a thin translator: read pointer messages, emit tool-typed messages. Examples:

- `polygon_tool::events::step` (run_if `ActiveTool::Polygon`): consumes `PointerClick` (modifiers must be empty) → `PolygonToolInputEvent::AddPoint(pos)`. Consumes `PointerClick` for right button → `RemoveLastPoint`. Future: `PointerClick` with Ctrl on a marker → `DeselectMarker`.
- `orbit_camera::events::step` splits:
  - **always-on** half: `PointerDrag` for right button → orbit; `MouseWheel` → zoom.
  - **`ActiveTool::Pan`-gated** half: `PointerDrag` for left button (no modifiers) → pan.
- `box_select::events::step` (run_if `ActiveTool::BoxSelect`, **or** any tool when Shift held — TBD): `PointerDragStart`/`PointerDrag`/`PointerDragEnd` with Shift → rectangle update events.

This is the pattern that cleanly handles "shift-drag is box select even though Polygon is the active tool": the box\_select mapper's run condition is `active_tool_is(BoxSelect) OR (shift held at gesture start)`, and the camera/polygon mappers explicitly require empty modifiers. Filters stay mutually exclusive by design.

### Layer 4 — Tool controllers (no changes)

Existing `controller.rs` files continue to consume their tool-typed messages. Modal routing is invisible to them.

### Hit-testing (cross-cutting)

Move [`cursor_to_world_on_sphere_f64`](rust/bevy_experiment/src/orbit_camera/geometry.rs) and its helpers from `orbit_camera/geometry.rs` to a new `common/picking.rs`. Polygon tool already imports it from `orbit_camera`, which is a layering smell. Hover-driven hit-testing (e.g., highlight a marker under cursor) lives in each tool/layer that cares — they read `PointerMove` and run their own picking. No central hover dispatch yet; revisit if multiple layers want the same hover.

## Critical Files

**New:**
- `rust/bevy_experiment/src/common/pointer_input.rs` — Layer 1 (renamed/expanded `mouse_interaction.rs`)
- `rust/bevy_experiment/src/common/active_tool.rs` — Layer 2
- `rust/bevy_experiment/src/common/picking.rs` — extracted from `orbit_camera/geometry.rs`

**Modified:**
- `rust/bevy_experiment/src/common/mod.rs` — module exports
- `rust/bevy_experiment/src/main.rs` — register `ActiveToolPlugin`
- `rust/bevy_experiment/src/orbit_camera/events.rs` — split into modeless + `Pan`-gated mappers; consume pointer messages instead of polling resource
- `rust/bevy_experiment/src/orbit_camera/plugin.rs` — register both halves with run conditions; update import paths
- `rust/bevy_experiment/src/orbit_camera/geometry.rs` — slim down (re-export from `common::picking` during migration, or delete)
- `rust/bevy_experiment/src/orbit_camera/controller.rs` — update picking import
- `rust/bevy_experiment/src/polygon_tool/events.rs` — consume `PointerClick` instead of polling
- `rust/bevy_experiment/src/polygon_tool/plugin.rs` — add `run_if(active_tool_is(Polygon))`
- `rust/bevy_experiment/src/polygon_tool/controller.rs` — update picking import

**Deleted:**
- `rust/bevy_experiment/src/common/mouse_interaction.rs` (replaced by `pointer_input.rs`)

## Migration Path (5 phases, behavior-preserving until Phase 4)

Each phase compiles, runs, and matches current behavior. No flag day.

**Phase 1 — Add Layer 1 alongside the current FSM, keep state for back-compat.**
- Create `common/pointer_input.rs` with the new message types and the FSM. Keep `MouseInteractionState` exported with the same shape so existing consumers don't break.
- Internally, the new system emits the new messages **and** writes the legacy `MouseInteractionState`.
- Rename module from `mouse_interaction` to `pointer_input`; leave a `pub use` alias in `common/mod.rs` for the old path so consumers compile unchanged.
- **Verify:** app behaves identically. Add `cargo test` for the FSM (table-driven: press → drag threshold → release scenarios; modifier capture; double-click window).

**Phase 2 — Migrate `polygon_tool` to messages.**
- Rewrite `polygon_tool/events.rs` to consume `PointerClick` (left + right) instead of polling `MouseInteractionState`.
- **Verify:** click adds a control point, right-click removes one (current behavior).

**Phase 3 — Migrate `orbit_camera` to messages and split the mapper.**
- Rewrite `orbit_camera/events.rs` to consume `PointerDragStart`/`PointerDrag`/`PointerDragEnd` for left (pan) and right (orbit), and `MouseWheel` for zoom.
- Don't gate yet — all tools/inputs run unconditionally as today.
- **Verify:** pan, orbit, zoom all still feel identical (especially the cursor-position-based pan, which today preserves the grabbed point under the cursor).

**Phase 4 — Introduce `ActiveTool` and gate tool inputs.**
- Add `common/active_tool.rs` with default `Pan`. Add keyboard tool-switch system (`V`/`P`).
- Apply `run_if(active_tool_is(...))` to:
  - `orbit_camera`'s left-drag pan mapper → `Pan`
  - `polygon_tool`'s click mapper → `Polygon`
- Right-drag orbit and scroll zoom remain modeless.
- **Verify:** with default `Pan` active, app behaves as today. Press `P` and clicking adds polygon points but left-drag no longer pans (intended). Press `V` and pan returns.

**Phase 5 — Clean up and extract picking.**
- Delete `MouseInteractionState` and the legacy alias.
- Move `cursor_to_world_on_sphere_f64` and helpers from `orbit_camera/geometry.rs` to `common/picking.rs`. Update imports in `orbit_camera/controller.rs` and `polygon_tool/controller.rs`.
- **Verify:** full clean build with no `mouse_interaction` references; behavior unchanged.

After Phase 5 the foundation supports double-click, hover, modifiers, and box-select with no further architectural changes — those become incremental feature additions:

- **Double-click zoom-to-point:** orbit camera's modeless mapper consumes `PointerDoubleClick` → emit a zoom-to-cursor event.
- **Connected-marker selection:** polygon tool consumes `PointerDoubleClick` and runs marker hit-test.
- **Hover highlight:** new layer (e.g., marker tool) consumes `PointerMove`, runs marker hit-test, updates a `HoveredMarker` resource for rendering.
- **Box select:** new `box_select` module gated by `ActiveTool::BoxSelect` (or modifier-triggered overlay), consumes `PointerDragStart`/`PointerDrag`/`PointerDragEnd`.
- **Ctrl-click toggle selection:** any tool whose mapper checks `modifiers.ctrl` on `PointerClick`.

## Verification

After each phase: `cargo build`, then run `cargo run --bin bevy_experiment` and exercise:

- **Phase 1:** all existing interactions (pan via left-drag, orbit via right-drag, scroll zoom, polygon click add, polygon right-click remove) work identically.
- **Phase 2:** specifically polygon click add/remove. A 4-pixel "click" still registers as drag (not click) due to the 3 px dead zone.
- **Phase 3:** pan smoothness — the grabbed world point stays under the cursor (regression risk: cursor position must continue to flow into `controller::step` at f64 precision via `PointerState.position` or the message stream).
- **Phase 4:** keyboard tool switching changes input routing as expected; right-drag orbit and scroll zoom work in every tool.
- **Phase 5:** no behavior change; just import cleanup.

**Unit tests** for `pointer_input` FSM (added in Phase 1, expanded as new gestures land):
- press → release inside dead zone → `PointerClick` fires once, `PointerPress` and `PointerRelease` fire too.
- press → move past 3 px → release → `PointerDragStart` + `PointerDrag` events + `PointerDragEnd`, no `PointerClick`.
- press with Shift → release → `PointerClick.modifiers.shift == true`.
- two clicks within 300 ms / 5 px / matching modifiers → `PointerDoubleClick`.
- two clicks across 400 ms → no `PointerDoubleClick`.

These tests are cheap to write because the FSM's only inputs are `(just_pressed, pressed, just_released, cursor_position, modifiers, time)` — no Bevy `App` needed; just call the classifier function directly with synthesized inputs.






Hmm, actually I just remembered that I want my tool logic to exist outside of the Bevy app entirely - this will be embedded in a typescript web app and will only be used for things like rendering and detecting clicks on objects. For now, the polygon_tool is largely a prototyping/debugging feature.

So, please update the plan to facilitate this use case instead: an external API (exposed to javascript) will be used to enable/disable camera panning, and it should expose the following events to javascript:
- camera moved
- mouse down/up on object or terrain/background
- mouse click on object or terrain/background
- mouse double click on object or terrain/background
- mouse move over object or terrain/background
- mouse drag (move after starting a drag operation) over object or terrain/background
-
-
