# Architecture Review: Smart Map Layer Plan

## Context

You asked for a review of [rust/bevy_experiment/src/common/plan.md](rust/bevy_experiment/src/common/plan.md), focused on **event handling structure** and whether it sets you up for the long-term goal: a Bevy-based, plugin-extensible, OSS-quality map library where new layer types (waypoints, polygons, box-select, custom) can be added without touching the framework, while supporting Google Earth-style camera and arbitrary hover/click/drag on layer entities.

This review is directional, not migration-focused. Where the plan needs sharpening I describe what to change and why; the migration phases can be re-derived once the target architecture settles.

## Bottom Line

**The plan's overall shape is correct and worth committing to.** Five-stage pipeline (raw → pointer FSM → hit-test → dispatcher → controllers → bridge), camera as a fallback consumer of unclaimed gestures, layers as Bevy plugins that own their components and systems — this is the same pattern deck.gl, Mapbox GL JS, and Bevy's own [`bevy_picking`](https://docs.rs/bevy_picking) converge on. It cleanly satisfies your three goals (extensibility, Earth-style camera, arbitrary per-layer interactions).

There are **four refinements worth making before you start building** that meaningfully improve cleanliness, performance, and OSS-readiness:

1. **Strategic: decide your relationship to `bevy_picking`.** Bevy 0.17 ships a hit-test framework with pluggable backends and entity-targeted events that overlaps ~70% with what the plan is building. You should either use it (and build only the dispatcher + re-routing on top) or document the deliberate decision not to.
2. **Use observers, not `LayerId`-filtered messages, for layer dispatch.** This is the single highest-leverage change for cleanliness. Removes a runtime filter, eliminates a class of "I forgot to check `target ==`" bugs, and is idiomatic for Bevy 0.17.
3. **Fix the `HitCandidates` parallelism story.** A `Resource` with `Vec` write access serializes the systems writing into it; the plan claims they run in parallel, but they can't.
4. **Make SystemSet ordering explicit and load-bearing.** `PointerInput → HitTest → Dispatch → Controllers` should be the documented contract that plugin authors target.

Plus a handful of spec gaps (flag semantics, hover identity, plugin contract) that are easy to close once the four above are settled.

---

## 1. What the plan gets right

These are the load-bearing decisions; they should not change.

- **Tools-as-public-concept is the wrong abstraction.** The diagnosis in the Context section is exactly right. There is no global "active tool" in deck.gl or Google Earth — every interactive thing is a layer with its own hit-test and policy. Replacing `ActiveTool` with the dispatcher + smart-layer model is the correct call.
- **Pointer FSM as Layer 1 is reusable for any ECS-based interaction system.** Press / Release / Click / DoubleClick / DragStart / Drag / DragEnd / Move with modifiers frozen at press time is the right vocabulary. Almost every web/native interaction stack converges here.
- **Camera-as-fallback rather than camera-as-mode.** Right-drag-orbit and scroll-zoom always-on; left gestures route through the dispatcher with camera pan as the unclaimed-fallback. This cleanly mirrors what users expect from Google Earth and is the only model that scales to "arbitrary plugin-defined layers."
- **`UnclaimedGesture` fan-out is the right mechanism for click-on-empty-globe.** A polygon tool that wants to add a vertex on an empty click is a natural consumer of unclaimed clicks; so is the camera. Multi-consumer fan-out works for now (and the plan correctly flags exclusivity as a future concern).
- **Layer-event types are designed to be bridge-friendly without depending on the bridge.** `IconEvent` doesn't know about WASM throttling. The bridge filters; layers emit faithfully. Right separation of concerns.
- **Drag-routing freeze at press time.** Pressing on an icon and dragging off its bounds keeps dragging the original icon. This is non-obvious to get right and the plan handles it correctly.
- **Modifier-priority via synthetic max-z hit-test.** Letting box-select win by pushing a `LayerHit { z_order: i32::MAX, .. }` at press time when shift is held is elegant — no special case in the dispatcher, fully extensible to "shift-drag" and similar conventions any layer wants to claim.

---

## 2. Strategic question: relationship to `bevy_picking`

This is the most important question for the plan, and it isn't addressed.

Bevy 0.17 ships [`bevy_picking`](https://docs.rs/bevy_picking) which provides:

- **Pluggable backends** — mesh, sprite, UI, gizmo, and arbitrary user backends. A backend computes hits at the cursor and pushes them into a shared resource. Backends run in parallel.
- **Entity-targeted observer events** — `Pointer<Click>`, `Pointer<Drag>`, `Pointer<Over>`, `Pointer<Out>`, `Pointer<Move>`, etc., dispatched via Bevy's observer system to the picked entity directly.
- **A `Pickable` component** that opts entities in.

This overlaps roughly 70% with what the plan is building from scratch. The plan's `Interactive` ≈ `Pickable`, `LayerHit` ≈ a backend's hit output, and the `LayerGesture` events nearly mirror `Pointer<*>`.

**There are real reasons you might still build your own**:
- You need re-routing logic (press-on-hover-only-icon → camera pan) that `bevy_picking` doesn't model.
- You want sphere-projection picking for layers whose entities aren't on a Bevy mesh (an icon "is" a `DVec3` world position, not an entity with a `Mesh` AABB).
- You want gestures to fall through to "camera as a virtual layer", which `bevy_picking` doesn't natively express.
- You want full control over the API for OSS positioning (`bevy_picking` is general-purpose; a map library benefits from a tighter, map-specific surface).

**But there are also strong arguments to compose on top of it**:
- Custom backends are exactly the extension point `bevy_picking` is designed for. A `MapSphereBackend` projecting the cursor onto the globe and a `BillboardBackend` for icons are small.
- You inherit observer-based dispatch (point #3 below) for free.
- Existing Bevy users will recognize your layer model immediately.
- Less code to maintain, document, and test.

**My recommendation:** Spend 1–2 days reading `bevy_picking`'s source and writing a one-page memo on whether composing on top is viable. The likely answer is "yes, for hit-testing; no, for the dispatcher / re-routing / fallback semantics." That is fine — but it should be a deliberate decision documented in the plan, not an oversight. An OSS map library that ignores Bevy's first-party picking framework needs to justify itself.

If you compose: keep the dispatcher, the re-route rule, the fallback-to-camera model, the layer/controller pattern, and the bridge — but delete `HitCandidates`, `LayerHit`, and per-layer hit-test systems. They become `bevy_picking` backends.

---

## 3. Use observers, not `LayerId`-filtered messages, for layer dispatch

This is the highest-leverage cleanliness improvement.

The plan's `LayerGesture` carries a `target: LayerId` field. Every layer's controller reads `MessageReader<LayerGesture>` and filters by `target == LayerId("icon")`. With *N* layers and *G* gestures per frame, that's *N×G* reads, *N×G* filter checks, and *N* opportunities to forget the filter.

In Bevy 0.17 the idiomatic alternative is **entity-targeted observer triggers**:

```rust
// In the dispatcher:
commands.trigger_targets(
    LayerClick { position, modifiers, feature_key },
    entity,
);

// In the IconLayer plugin:
app.add_observer(
    |trigger: Trigger<LayerClick>, query: Query<&IconState>| {
        let Ok(icon) = query.get(trigger.target()) else { return };
        // handle click on this specific icon
    },
);
```

**Why this is a strict improvement:**
- **No `LayerId` filter at the consumer.** The observer's `Query<&IconState>` filter implicitly identifies "icons the IconLayer cares about." If the entity isn't an icon, the observer's `query.get()` returns `Err` and the observer no-ops.
- **No runtime LayerId-vs-LayerId mismatch bugs.** The plan acknowledges `LayerId(&'static str)` is fragile; observers eliminate the failure mode entirely.
- **Cheaper.** Each gesture fires exactly one observer chain (the entity's), not *N* readers.
- **Bevy-idiomatic.** Anyone reading your OSS library on Bevy 0.17 expects this. It's how `bevy_picking` itself dispatches.
- **Plays naturally with `bevy_picking`** if you go that direction in §2.

**`LayerId` doesn't disappear** — keep it on `Interactive` for diagnostics, logs, hit-test ordering tie-breaks, and any case where the dispatcher itself needs to reason about layers. Its role just shrinks from "runtime routing key" to "metadata."

`UnclaimedGesture` stays as a plain `Message<UnclaimedGesture>` — it has multiple consumers (camera + click-on-empty handlers) and no specific entity to target. That's the correct shape for that case.

---

## 4. Fix the `HitCandidates` parallelism story

Spec gap with a real consequence.

The plan says:

> Layer hit-test systems run in parallel (different layers' queries don't conflict).

But they all need write access to `ResMut<HitCandidates>`, which Bevy serializes. The claim is wrong as written.

Three reasonable fixes, in order of preference:

1. **Drop `HitCandidates` if you compose on `bevy_picking`** — backends accumulate hits via the framework's existing parallel-friendly mechanism. (See §2.)
2. **Convert hit-test systems to `MessageWriter<LayerHit>`.** Each layer writes its candidates as messages; the dispatcher reads them all. Messages don't conflict between writers, so layers' hit-test systems run in parallel. This is the smallest delta from the current plan and preserves all the existing semantics.
3. **Per-system `Local<Vec<LayerHit>>` + a gather system.** More plumbing, no clear advantage over messages.

Whichever you pick, make it explicit in the plan. The current text suggests parallelism that the data structure prevents.

---

## 5. Make SystemSet ordering the documented plugin contract

The plan mentions `SystemSet::Dispatch`. For an OSS library where third-party plugins register their own systems, you need the *full* schedule to be a documented contract:

```rust
pub enum MapSet {
    PointerInput,   // Layer 1 FSM runs here
    HitTest,        // every layer's hit-test system runs here
    Dispatch,       // dispatcher runs here, reads HitTest output
    Controllers,    // every layer's controller runs here, consumes triggered events
}
```

…with `PointerInput.before(HitTest).before(Dispatch).before(Controllers)` configured once by the framework's root plugin.

A new layer plugin's contract becomes:

> 1. Add hit-test systems in `MapSet::HitTest`.
> 2. Add controllers in `MapSet::Controllers`.
> 3. Use observers for entity-targeted gestures; consume `MessageReader<UnclaimedGesture>` for unclaimed events.

This is the kind of contract a third-party developer can implement without reading any framework code. That's the OSS-readiness bar.

**Bonus:** giving the schedule a name (`MapSet`, not `SystemSet::Dispatch` ad-hoc) reads better in user code: `add_systems(Update, my_hit_test.in_set(MapSet::HitTest))`.

---

## 6. Tighten the `Interactive` / flags contract

Two spec gaps in the plan to close before implementation:

**6a. Press-intent flag set is undefined.** Routing rule 1 says "pick the highest-z_order LayerHit whose flags allow press intent." Spell out the rule:

> A press routes to a layer iff that layer's hit pushed any of `CLICKABLE | DRAGGABLE | DOUBLE_CLICKABLE`. `HOVERABLE`-only entities are never press targets — the press falls through to camera.

Otherwise the dispatcher's behavior on a hover-only icon is genuinely ambiguous.

**6b. Hover identity is `(LayerId, Entity, feature_key)`, not `Entity`.** The plan says hover diff is "against the previous frame's hovered entity." But `feature_key: u64` exists precisely so a single `Entity` can host multiple sub-features (a polygon entity with several vertices). Hovering vertex 0 → vertex 1 on the *same entity* should fire `HoverExit`/`HoverEnter`. Specify the full tuple as the diff key.

**6c. Optional: tighten `LayerId` for compile-time-known plugins.** `LayerId(&'static str)` is fine for now and matches your OSS goal of letting plugins declare themselves. If you later need stronger type safety for first-party layers, a `LayerId<T: 'static>(PhantomData<T>)` works too. Not a now-decision.

---

## 7. OSS-readiness recommendations

Tactical, but worth deciding now since they shape the file layout:

- **Crate boundaries.** Plan to split into at least two crates eventually:
  - `bevy_map_layers` (or whatever) — pointer FSM, dispatcher, `Interactive`, `MapSet`. No camera, no specific layers.
  - `bevy_map_camera` — Google Earth-style orbit/pan/zoom. Optional; consumes `UnclaimedGesture`.
  - `bevy_map_icon`, `bevy_map_polygon`, … — example/reference layers. Each its own crate or feature.

  You don't need to split the crate today, but designing the directory structure to mirror this future split (`crates/core/`, `crates/camera/`, `crates/icon/`) prevents painful reshuffles later.
- **The `LayerId(&'static str)` contract is your stable public API.** Name it, document it, and don't break it lightly.
- **"Smart layer" → "interactive layer" or just "layer."** "Smart" is unusual terminology that doesn't add information. Pick a term and use it consistently in code, types, docs, and any future README.
- **`UnclaimedGesture::cursor_world: Option<DVec3>` is sphere-specific.** It hard-codes globe geometry into the framework's central event type. Two cleaner options:
  - Drop it. Each consumer (camera, polygon) does its own projection from `cursor_screen` using the picking module. One extra line per consumer; the framework stays geometry-neutral.
  - Make the projection extensible: a `WorldProjection` trait with a default sphere implementation, swappable by app config.

  The sphere assumption is fine for *this* app, but bakes a constraint into the framework's core types. For OSS, drop it.
- **Document the layer plugin contract as a "how to write a layer" guide.** The first user of your library shouldn't have to read the dispatcher's source. The IconLayer ends up being the canonical example — make sure it's pedagogical, not just functional.

---

## 8. Open questions for you

These are decisions I'd want answered before writing the framework, ranked by impact:

1. **Compose on `bevy_picking` or build standalone?** (§2) — biggest decision; affects ~30% of the code.
2. **Observers vs. `LayerId`-filtered `MessageReader<LayerGesture>` for dispatch?** (§3) — strongly recommend observers; want your sign-off before re-specifying.
3. **OSS scope: is this *the* library you intend to publish, or a prototype that informs a v2?** Affects how strict to be about §7.
4. **Is `cursor_world` sphere-projection a framework concern or a layer concern?** (§7) — affects framework neutrality vs. ergonomics tradeoff.
5. **What's the minimum-viable second layer after `IconLayer` for proving the framework?** The plan says polygon eventually, box-select eventually. Picking a concrete second layer to drive *during* IconLayer's design helps avoid "framework that fits exactly one user."

---

## 9. Critical files referenced

For the migration work, when you do return to it:

- [rust/bevy_experiment/src/common/plan.md](rust/bevy_experiment/src/common/plan.md) — the plan being reviewed
- [rust/bevy_experiment/src/common/mouse_interaction.rs](rust/bevy_experiment/src/common/mouse_interaction.rs) — current `MouseInteractionState`, becomes `pointer_input.rs`
- [rust/bevy_experiment/src/orbit_camera/geometry.rs](rust/bevy_experiment/src/orbit_camera/geometry.rs) — current picking, becomes `common/picking.rs`
- [rust/bevy_experiment/src/orbit_camera/events.rs](rust/bevy_experiment/src/orbit_camera/events.rs) — current input → camera-event mapping, splits into always-on (right-drag, scroll) and `UnclaimedGesture`-consuming halves
- [rust/bevy_experiment/src/polygon_tool/events.rs](rust/bevy_experiment/src/polygon_tool/events.rs) — first migration target for `UnclaimedGesture::Click`

---

## Summary

The plan is heading in the right direction — the bones are sound. Before implementation, settle:

1. `bevy_picking` relationship (compose on top, or document why not).
2. Observer-based dispatch instead of `LayerId` filtering.
3. Messages (or `bevy_picking` backends) instead of `HitCandidates` resource for parallel hit-test writes.
4. Explicit `MapSet` schedule as plugin contract.

Plus close the four spec gaps (press-intent flags, hover identity tuple, parallelism, sphere-projection scope).

If those land, the result is a clean, idiomatic, plugin-extensible Bevy library that natively supports Google-Earth-style camera plus arbitrary layer interactions, and is plausibly worth open-sourcing.
