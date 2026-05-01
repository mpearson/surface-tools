# Smart Map Layer Architecture (replaces in-flight Pointer Input + ActiveTool plan)

## Context

The Bevy app at [rust/bevy_experiment/](rust/bevy_experiment/) is being built to replace deck.gl in a TypeScript web app that has many "modal tools" — box-select, polygon edit, waypoint drag, hover highlight, etc. The TS app's tool implementations are mature and stay where they are; **business logic remains on the TS side**. Bevy's job is rendering plus *low-latency* feedback loops where crossing the JS↔WASM boundary every frame would feel bad (drag-a-waypoint and re-route its connecting lines, draw-and-update a rubber-band rectangle, hover-swap an icon).

The in-flight plan at [rust/bevy_experiment/src/common/plan.md](rust/bevy_experiment/src/common/plan.md) introduced a layered pipeline (`pointer_input` → `active_tool` → tool-typed events → tool controllers) modeled on a *modal tool* concept. That pipeline's bottom and top halves are correct; the **middle layer (`ActiveTool`) is the wrong abstraction** for this product. Tools-as-public-concept conflates spatial routing ("this entity is interactive, that empty space is camera") with modal exclusivity ("only the polygon tool reacts to clicks") — and in a deck.gl-style API there is no global "active tool", only a pile of layers each with their own hit-test and interaction policy.

This plan keeps the in-flight Layer 1 (`pointer_input`) and the picking-extraction work, **replaces Layer 2** with a smart-layer framework (per-layer plugins, hit-test arbitration, gesture routing), and reframes the public API as **interactive map layers** instead of tools. The existing `polygon_tool` proof-of-concept is preserved untouched as a reference; the first true smart layer to ship will be a new `IconLayer` (waypoints with hover + drag).

## Architecture

Five concerns, top to bottom. Each emits messages or writes a frame-scoped resource consumed by the next.

```
[ Raw input ]            bevy::input — ButtonInput, MouseMotion, MouseWheel, CursorMoved, KeyCode
       ↓
[ pointer_input ]        FSM emits PointerPress / Release / Click / DoubleClick
                         / DragStart / Drag / DragEnd / Move (modifiers frozen at press)
                         + private FSM resource, public read-only PointerState
       ↓
[ Per-layer hit-tests ]  Each layer plugin runs a hit-test system that writes LayerHit
                         candidates into a frame-scoped HitCandidates resource
       ↓
[ Dispatcher ]           Reads pointer messages + HitCandidates, routes each gesture
                         to a layer (frozen at press) or to camera fallback. Emits
                         LayerGesture (targeted) or UnclaimedGesture (camera + click-empty)
       ↓
[ Layer controllers ]    Each layer consumes its targeted LayerGesture, mutates its
                         own state locally (low-latency), emits typed outbound events
                         (e.g. IconEvent::DragStart) consumed by the bridge
       ↓
[ Bridge (future) ]      Forwards selected outbound events to JS via wasm_bindgen,
                         throttled per BridgePolicy (the seam, not implemented)
```

The `pointer_input` and picking layers from the in-flight plan are kept in full. The dispatcher and the smart-layer model are new.

## Smart Layer Model

A "smart layer" is **one Bevy plugin** that:

1. Renders its own data.
2. Tags its interactive entities with the `Interactive` component so the framework can hit-test them.
3. Runs a hit-test system in `PreUpdate` that writes candidates into `HitCandidates`.
4. Consumes targeted `LayerGesture` messages, mutates its state locally, emits typed outbound events.
5. Lives in its own folder (e.g. `layers/icon/`).

### Public contract: `Interactive` component

```rust
// common/interaction.rs

#[derive(Component)]
pub struct Interactive {
    pub layer: LayerId,           // owning layer plugin
    pub z_order: i32,             // higher = "on top" for hit-test priority
    pub flags: InteractionFlags,  // HOVERABLE | CLICKABLE | DRAGGABLE | DOUBLE_CLICKABLE
}

bitflags! {
    pub struct InteractionFlags: u8 {
        const HOVERABLE        = 1 << 0;
        const CLICKABLE        = 1 << 1;
        const DRAGGABLE        = 1 << 2;
        const DOUBLE_CLICKABLE = 1 << 3;
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct LayerId(pub &'static str);   // e.g. LayerId("icon")
```

Optional companion components live inside the layer that uses them — e.g. `HoverIcon`, `DragIcon`, `DragConstraint::OnSphere`. The framework doesn't need to know about them; the layer's own controller reads them.

### Hit-test: per-layer system, no dyn-trait

Each layer contributes one ordinary Bevy system that scans its entities and pushes candidates:

```rust
// common/layer_registry.rs

#[derive(Resource, Default)]
pub struct HitCandidates {
    pub at_cursor: Vec<LayerHit>,   // cleared each frame by the dispatcher
}

#[derive(Clone, Copy, Debug)]
pub struct LayerHit {
    pub layer: LayerId,
    pub entity: Entity,
    pub z_order: i32,
    pub flags: InteractionFlags,
    pub feature_key: u64,           // layer-private sub-feature handle
}
```

Layer hit-test systems run in parallel (different layers' queries don't conflict). The dispatcher runs in a `SystemSet::Dispatch` after them.

### Outbound callbacks: typed Bevy `Message`s

Layer-specific events are typed enums emitted by the layer's controller. Example:

```rust
// layers/icon/events.rs

#[derive(Message, Clone, Debug)]
pub enum IconEvent {
    HoverEnter  { entity: Entity, icon_id: IconId },
    HoverExit   { entity: Entity, icon_id: IconId },
    Click       { entity: Entity, icon_id: IconId, modifiers: ModifierKeys },
    DoubleClick { entity: Entity, icon_id: IconId, modifiers: ModifierKeys },
    DragStart   { entity: Entity, icon_id: IconId, world_pos: DVec3, modifiers: ModifierKeys },
    Drag        { entity: Entity, icon_id: IconId, world_pos: DVec3 },   // throttled at the bridge
    DragEnd     { entity: Entity, icon_id: IconId, world_pos: DVec3 },
}
```

The low-latency loop: while a drag is in progress, the layer's controller mutates the icon's `IconWorldPos` (and any connected lines) **every frame, locally**. `DragStart` and `DragEnd` always emit; `Drag` is throttled at the bridge layer. JS only feels the drag when the bridge says so.

## Input Dispatch (replaces `ActiveTool`)

A single `dispatcher::resolve_and_dispatch` system consumes Layer 1's pointer messages, reads `HitCandidates`, and emits routed messages. It owns a small private `GestureRouting` state per pointer button: `Idle` or `Routed { target: RouteTarget, frozen_entity, frozen_feature }`.

### Routed messages

```rust
#[derive(Message, Clone, Debug)]
pub struct LayerGesture {
    pub target: LayerId,
    pub entity: Entity,
    pub feature_key: u64,
    pub kind: LayerGestureKind,
    pub modifiers: ModifierKeys,
}

#[derive(Clone, Debug)]
pub enum LayerGestureKind {
    HoverEnter { position: Vec2 },
    HoverExit,
    Click       { position: Vec2 },
    DoubleClick { position: Vec2 },
    DragStart   { origin: Vec2, position: Vec2 },
    Drag        { origin: Vec2, position: Vec2, delta: Vec2 },
    DragEnd     { origin: Vec2, position: Vec2 },
}

/// Emitted when no layer claims the gesture. Consumed by camera AND by layers
/// like polygon that want clicks-on-empty-space.
#[derive(Message, Clone, Debug)]
pub struct UnclaimedGesture {
    pub kind: LayerGestureKind,
    pub button: PointerButton,
    pub modifiers: ModifierKeys,
    pub cursor_world: Option<DVec3>,    // pre-computed sphere intersection if applicable
}
```

### Routing rules

1. **On `PointerPress`**: pick the highest-`z_order` `LayerHit` whose flags allow press intent. If found, freeze routing on that layer + entity. Else, route to camera.

2. **Subsequent gesture events for the same button**: emit `LayerGesture` to the *frozen* target/entity. Hit-tests in later frames don't change routing — drag-off-the-icon still drags the originally-clicked icon.

3. **On `PointerDragStart` for an entity that lacks `DRAGGABLE`**: re-route mid-gesture to camera. The dispatcher swaps `target` to `Camera` and emits a synthetic `UnclaimedGesture::DragStart`. (Confirmed with user: feels natural — pressing on a hover-only icon and dragging should pan the camera.)

4. **On terminating event** (`DragEnd`, `Click`, `DoubleClick`, or release): reset routing for that button to `Idle`.

5. **Hover** (no button held): every `PointerMove`, look at the highest-z `HOVERABLE` candidate. Diff against the previous frame's hovered entity and emit `HoverEnter` / `HoverExit`. Hover is suppressed while any button is held (revisit if a layer needs hover-during-drag, e.g. snap-to-target).

6. **Modifier-priority layers** (e.g. `BoxSelectLayer`): the layer's hit-test pushes a synthetic `LayerHit { z_order: i32::MAX, .. }` whenever its modifier condition holds at press time. No special case in the dispatcher — it just wins by z-order. This generalizes "shift-drag is box-select even on icons."

### Camera precedence

Camera is a consumer of `UnclaimedGesture` for left-button gestures, **plus two always-on inputs that bypass the dispatcher**:

- **Right-button drag → orbit**: always camera, never routed. Right-*click* (release within dead zone) on a layer entity *is* routed to that layer (e.g. polygon's right-click-to-remove-point) because the click/drag FSM split happens upstream.
- **Scroll wheel → zoom**: always camera, never routed.

This gives layers full control over left-button gestures and click-style right-button interactions, while keeping camera navigation reflexively available.

## TS↔WASM Bridge (seam only)

The bridge plugin lives at `bridge/` and is not implemented in this plan — only the seam is designed.

```rust
// bridge/policy.rs (future)
#[derive(Resource)]
pub struct BridgePolicy {
    pub icon_drag_emit_every_n_frames: u32,
    pub icon_drag_emit_min_world_distance: f64,
    pub coalesce_hover: bool,
}

// bridge/outbound.rs (future)
#[wasm_bindgen]
pub fn subscribe_icon_events(callback: js_sys::Function) { /* stash in thread-local */ }

pub fn forward_icon_events(
    mut events: MessageReader<IconEvent>,
    policy: Res<BridgePolicy>,
    mut last_drag_emit: Local<Option<DVec3>>,
) {
    for ev in events.read() {
        if should_throttle(&ev, &policy, &mut last_drag_emit) { continue; }
        ICON_SINK.with(|s| s.call(ev));
    }
}
```

Inbound (JS → Bevy: `add_icon`, `move_icon`, etc.) is also out of scope and gets the same treatment in a future phase.

The point of including the seam in this plan: the layer-event types (`IconEvent` etc.) are designed so they serialize cleanly to JS and so throttling is the bridge's job, not the layer's. Layers always emit faithfully; the bridge filters.

## Critical Files

**New:**
- [rust/bevy_experiment/src/common/pointer_input.rs](rust/bevy_experiment/src/common/pointer_input.rs) — Layer 1 (per existing plan)
- [rust/bevy_experiment/src/common/picking.rs](rust/bevy_experiment/src/common/picking.rs) — extracted from `orbit_camera/geometry.rs`
- [rust/bevy_experiment/src/common/interaction.rs](rust/bevy_experiment/src/common/interaction.rs) — `Interactive`, `InteractionFlags`, `LayerId`
- [rust/bevy_experiment/src/common/layer_registry.rs](rust/bevy_experiment/src/common/layer_registry.rs) — `HitCandidates`, `LayerHit`, `LayerGesture`, `UnclaimedGesture`, dispatcher system, `SystemSet::Dispatch`
- `rust/bevy_experiment/src/layers/icon/{plugin,state,hit_test,controller,render,events}.rs` — first smart layer (waypoints, hover + drag)
- `rust/bevy_experiment/src/bridge/mod.rs` — placeholder module with `BridgePolicy` resource and a stub `forward_icon_events` system (one-layer proof-of-concept; full bridge is later work)

**Modified:**
- [rust/bevy_experiment/src/common/mod.rs](rust/bevy_experiment/src/common/mod.rs) — module exports
- [rust/bevy_experiment/src/main.rs](rust/bevy_experiment/src/main.rs) — register `LayerRegistryPlugin`, `IconLayerPlugin`, bridge stub
- [rust/bevy_experiment/src/orbit_camera/events.rs](rust/bevy_experiment/src/orbit_camera/events.rs) — split into always-on half (right-drag orbit, scroll zoom) and `UnclaimedGesture`-consuming half (left-drag pan); stop polling `MouseInteractionState`
- [rust/bevy_experiment/src/orbit_camera/plugin.rs](rust/bevy_experiment/src/orbit_camera/plugin.rs) — register both halves with appropriate ordering; update import paths
- [rust/bevy_experiment/src/orbit_camera/controller.rs](rust/bevy_experiment/src/orbit_camera/controller.rs) — update picking import
- [rust/bevy_experiment/src/polygon_tool/events.rs](rust/bevy_experiment/src/polygon_tool/events.rs) — consume `PointerClick` instead of polling `MouseInteractionState` (Phase 2 of the in-flight plan)
- [rust/bevy_experiment/src/polygon_tool/controller.rs](rust/bevy_experiment/src/polygon_tool/controller.rs) — update picking import

**Deleted:**
- [rust/bevy_experiment/src/common/mouse_interaction.rs](rust/bevy_experiment/src/common/mouse_interaction.rs) — replaced by `pointer_input.rs`
- [rust/bevy_experiment/src/orbit_camera/geometry.rs](rust/bevy_experiment/src/orbit_camera/geometry.rs) — contents moved to `common/picking.rs`

**Preserved untouched (per user decision):**
- All existing `polygon_tool/` files. The polygon prototype stays as-is alongside the new framework. It will receive the Phase-2 input-polling cleanup but is not reframed as a layer in this plan; that comes later if/when polygon control points need drag interactivity.

**Not built (deleted from in-flight plan):**
- `common/active_tool.rs` and `active_tool_is(...)` run-condition — replaced entirely by the dispatcher.

## Migration Path (8 phases, behavior-preserving until Phase 6)

Each phase compiles, runs, and matches current behavior unless noted. No flag day.

**Phase 1 — `pointer_input` alongside legacy `MouseInteractionState`.**
Per the in-flight plan's Phase 1: create [common/pointer_input.rs](rust/bevy_experiment/src/common/pointer_input.rs) with new message types and the FSM. Keep `MouseInteractionState` exported with the same shape; the new system writes both. Add the FSM unit tests.
**Verify:** identical app behavior; new tests pass.

**Phase 2 — Migrate `polygon_tool` to messages.**
Per the in-flight plan's Phase 2: rewrite `polygon_tool/events.rs` to consume `PointerClick` instead of polling.
**Verify:** click adds a control point, right-click removes one.

**Phase 3 — Migrate `orbit_camera` to messages + extract picking.**
Per the in-flight plan's Phase 3 + Phase 5: rewrite `orbit_camera/events.rs` to consume `PointerDragStart/Drag/DragEnd` and `MouseWheel`. Move `cursor_to_world_on_sphere_f64` and helpers from `orbit_camera/geometry.rs` to `common/picking.rs`. Update imports in [orbit_camera/controller.rs](rust/bevy_experiment/src/orbit_camera/controller.rs) and [polygon_tool/controller.rs](rust/bevy_experiment/src/polygon_tool/controller.rs). Don't gate yet — all inputs run unconditionally.
**Verify:** pan, orbit, zoom feel identical; grabbed point stays under cursor.

**Phase 4 — Introduce framework scaffolding (no layers yet, no behavior change).**
- Add [common/interaction.rs](rust/bevy_experiment/src/common/interaction.rs) and [common/layer_registry.rs](rust/bevy_experiment/src/common/layer_registry.rs).
- Add the dispatcher system and `SystemSet::Dispatch`.
- Camera switches to consuming `UnclaimedGesture` for left-button (everything is "unclaimed" at this point because no layer is registered).
- Right-drag and scroll keep their direct paths.
**Verify:** identical app behavior — every gesture falls through to camera. The framework is invisible. This is the key validation step: dispatcher wiring is correct before any layer competes for input.

**Phase 5 — Polygon click-on-empty-globe migrates to `UnclaimedGesture`.**
The polygon prototype has no `Interactive` entities, so its click-to-add and right-click-to-remove handlers move from raw `PointerClick` to `UnclaimedGesture { kind: Click, button: ... }`. This is a small mechanical change but it proves the dispatcher correctly fans `UnclaimedGesture` to multiple consumers (camera + polygon).
**Verify:** polygon prototype still works; pan/orbit/zoom unchanged.

**Phase 6 — Build `IconLayer` (first true smart layer; first new behavior).**
- New `layers/icon/` plugin: `IconWorldPos`, `IconRadius`, `IconAppearance`, `IconId`, `HoverIcon`, `DragIcon`, `DragConstraint::OnSphere`.
- Hit-test system projects each icon's world position to screen-space and checks distance to cursor.
- Controller consumes `LayerGesture { target: LayerId("icon"), .. }`, handles hover (icon swap), drag (mutate `IconWorldPos` locally during `Drag`), emits `IconEvent`.
- Render system uses gizmos for now (sphere markers); upgrade to billboarded sprites later.
- Spawn a handful of test icons in [basic_scene.rs](rust/bevy_experiment/src/basic_scene.rs) to exercise the layer.
**Verify:** hover an icon → it changes appearance. Drag an icon → it follows the cursor smoothly across the sphere. Release → it stays where dropped. Press on an icon and drag in empty space → camera pans (re-route rule). Right-drag still orbits, scroll still zooms.

**Phase 7 — Bridge stub for `IconEvent`.**
- `bridge/` plugin with `BridgePolicy` resource (defaults: emit `Drag` every 4 frames, no min-distance).
- `forward_icon_events` system that logs `IconEvent` to console (no actual `wasm_bindgen` export yet — the goal is just to prove the throttled-forwarding shape).
- Add a `bridge::log_icon_events` toggle in [main.rs](rust/bevy_experiment/src/main.rs) for development.
**Verify:** `cargo run` shows `IconEvent::DragStart` once, `IconEvent::Drag` periodically, `IconEvent::DragEnd` once on each interaction. No bridge work; just confirms the forwarding seam works.

**Phase 8 — Cleanup.**
- Delete `MouseInteractionState` and the legacy alias from `common/mod.rs`.
- Delete `orbit_camera/geometry.rs` (now empty after Phase 3 move).
- Confirm `cargo clippy` is clean.
**Verify:** full clean build with no `mouse_interaction` references; behavior unchanged.

After Phase 8 the foundation supports box-select (synthetic-priority hit-test), a future `PolygonLayer` (control points become `Interactive` entities), arbitrary new layer plugins, and the JS-side bridge — all as incremental additions, no further architectural change.

## Verification

After each phase: `cargo build && cargo run --bin bevy_experiment`. Exercise:

- **Phases 1–3:** existing interactions (pan via left-drag, orbit via right-drag, scroll zoom, polygon click add, polygon right-click remove) work identically. The 4-pixel "click" still registers as drag (3 px dead zone).
- **Phase 4:** dispatcher in place but invisible; behavior unchanged.
- **Phase 5:** polygon prototype unchanged; verify the dispatcher's `UnclaimedGesture` fan-out to two consumers.
- **Phase 6:** new — hover/drag icons. Check the re-route rule: press on icon → drag in empty space → camera pans. Check freeze rule: drag-off-icon-edge keeps dragging the icon, doesn't switch to camera.
- **Phase 7:** console logs show throttled `IconEvent::Drag` between un-throttled `DragStart`/`DragEnd`.
- **Phase 8:** no behavior change; just import cleanup.

**Unit tests** (added in Phase 1, expanded as new gestures land):
- `pointer_input` FSM: press → release in dead zone → `PointerClick` once. Press → move past 3 px → release → `DragStart` + `Drag` + `DragEnd`, no `PointerClick`. Press with Shift → `PointerClick.modifiers.shift == true`. Two clicks within 300 ms / 5 px / matching modifiers → `PointerDoubleClick`. Two clicks across 400 ms → no `PointerDoubleClick`.
- Dispatcher (added in Phase 4): table-driven scenarios with synthesized pointer messages and seeded `HitCandidates`. Press-on-empty → `UnclaimedGesture`. Press-on-layer → `LayerGesture` to that layer. Drag-off-entity preserves frozen routing. Press-on-non-DRAGGABLE + DragStart → re-routes to camera with synthetic `UnclaimedGesture::DragStart`. Modifier-priority layer wins over higher-z-order content layer.
- `IconLayer` controller (added in Phase 6): hover swap on `HoverEnter`/`HoverExit`, drag updates `IconWorldPos` along the sphere, emits `IconEvent` correctly.

The dispatcher tests are the load-bearing ones for the new architecture — they're cheap because the dispatcher's only inputs are `(pointer messages, HitCandidates, current routing state)`, no Bevy `App` needed.

## Open Questions / Future Considerations

These are flagged for awareness; none block this plan:

1. **Hover-during-drag** (e.g. drag a waypoint *over* another to snap them). Out of scope. Add a per-layer `HOVER_DURING_DRAG` flag if a use case appears.
2. **Multi-fan-out of `UnclaimedGesture::Click`.** If two layers both want "click on empty globe", both fire. Currently fine; if exclusivity needed later, add a `consume()` API or coordinate via TS-side config.
3. **`LayerId(&'static str)` works for compile-time-known layers; runtime layer registration from TS would need an interner.** Not a goal for now.
4. **Right-press-on-icon then drag past 3 px** is classified as drag → camera orbit. The user might have meant a click that wiggled. Acceptable per current FSM behavior.
5. **`feature_key: u64`** assumed sufficient for sub-feature handles. Switch to a richer payload only if a layer needs more than an integer.
6. **Pointer-capture on cursor-leaves-window during drag.** Layer 1 already accumulates raw `MouseMotion` so the dispatcher keeps emitting `Drag` while the button is held. Verify in Phase 6 that this works for icons, not just camera pan.
