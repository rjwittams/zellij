# Kitty image rearchitecture sketch

Status: draft design note for the post-proof cleanup / architecture pass.

This document captures what the current proof branch taught us about the next architecture step.
It is intentionally more structural than the checklist.

Related:
- `docs/kitty-graphics-checklist.md`
- `docs/kitty-unicode-placeholder-plan.md`

## Why a rearchitecture is needed

The current proof branch established that full in-core kitty support is feasible in Zellij:
- kitty APC reaches Zellij through `vte`
- assets are stored in-core
- explicit placements work
- query/delete/current real-app lifecycle works
- real apps such as `chafa`, `ratatui-image` / `flotilla`, and `yazi` can work

However, the branch also established an important limitation during the middle of the proof work: placeholder compatibility was easier to reach than native-looking fidelity.

Subsequent fixes improved this materially:
- restoring width-1 text-flow semantics for placeholder cells fixed the major Stage 4 smoke regression
- preserving explicit sizing intent (`c+r` vs `c only` vs `r only`) and matching Zellij's occupancy prediction to one-dimensional sizing fixed the main Stage 5 smoke regression in Kitty/Ghostty-class behavior

So the current picture is better than the earlier midpoint suggested:
- **placeholder compatibility is achievable**
- **smoke-harness fidelity is now good in Kitty/Ghostty-class behavior**
- **resize/reflow edge cases and broader completeness are still unfinished**
- **WezTerm still diverges similarly both direct and interposed**

## Architectural conclusion

The right overall shape still looks like:
- **protocol-specific ingest**
- **shared pane-owned image scene**
- **protocol-specific output**

But the shared middle layer must not collapse all image behavior into a single "visible image chunk" abstraction.

In particular, these are **not fully interchangeable**, even though the latest fixes show the shared scene can preserve more native behavior than it first appeared:
- kitty explicit placements
- kitty virtual/placeholder placements
- sixel placements

The new architecture should therefore preserve:
- shared ownership/lifecycle/composition state
- while also preserving enough **placement flavor / output semantics** to re-render each mode faithfully

A further finding from the ongoing cleanup is that not all image paths currently have the same ownership model:
- kitty is increasingly modeled as a **persistent scene-owned visible image path**
- sixel still behaves mostly as a **render-delta / changed-rect-driven image path**

That distinction should be acknowledged explicitly rather than prematurely flattened away.

## Core design principle

### Shared scene for ownership, not forced uniformity of output

The shared scene should answer questions like:
- what image assets exist in this pane?
- what logical placements exist?
- what text-flow anchors or explicit anchors exist?
- what lifecycle events affect them?
- what is visible in the viewport?
- how do pane clipping / floating occlusion / tab switches / clears / alt-screen affect them?

But it should **not** force every image mode into one final render primitive.

Instead, the shared scene should preserve enough provenance so that output can choose the right protocol-native form.

## Proposed model split

## 1. Asset layer

Protocol-native stored image asset.

Conceptual fields:
- asset id
- protocol provenance
- data form (`png`, `rgba`, later maybe `rgb`)
- intrinsic pixel dimensions
- source for retransmit / redraw

Important rule:
- preserve native asset form when possible
- avoid unnecessary transcoding

## 2. Logical placement layer

A pane-owned logical placement is the shared ownership object.

Conceptual fields:
- logical placement id
- asset id
- protocol identity
- placement flavor
- base geometry
- anchor kind
- content-flow behavior
- lifecycle domain (main screen / alt screen / etc.)

### Placement flavor must be explicit

Instead of only storing generic occupancy/geometry, keep an explicit placement flavor, eg:

```rust
enum PlacementFlavor {
    KittyExplicit {
        protocol_image_id: Option<u32>,
        protocol_placement_id: Option<u32>,
        source_rect: SourceRect,
        placement_rect: CellRect,
        x_offset: u32,
        y_offset: u32,
        z_index: i32,
        cursor_policy: KittyCursorPolicy,
    },
    KittyPlaceholder {
        protocol_image_id: Option<u32>,
        protocol_placement_id: Option<u32>,
        virtual_placement: KittyVirtualPlacement,
    },
    Sixel {
        // later / existing sixel-specific shape
    },
}
```

This is the key architectural shift:
- shared ownership / lifecycle
- protocol-aware placement semantics

## 3. Text-flow occupancy layer
n
Placeholder mode needs a first-class text-flow occupancy model, not just a derived visible chunk.

Conceptual fields:
- placement reference
- canonical/text-flow anchor per occupied cell
- placeholder row/col
- optional additional placeholder metadata if needed later

Something like:

```rust
struct PlaceholderCellRef {
    logical_placement_id: LogicalPlacementId,
    anchor: FlowAnchor,
    row: u16,
    col: u16,
}
```

The important shift is that placeholder occupancy should refer to the **logical placement**, not only to `(image_id, placement_id)` pairs reconstructed later.

## 4. Render-product layer

The shared scene should project into protocol-specific render products, not directly into one universal visible chunk type.

It is also likely that render products will continue to come from **two different ownership timings** for a while:
- persistent visible image products (currently the kitty direction)
- changed/delta image products (currently the sixel direction)

Conceptually:

```rust
enum RenderProduct {
    KittyExplicit(KittyExplicitRenderProduct),
    KittyPlaceholder(KittyPlaceholderRenderProduct),
    Sixel(SixelRenderProduct),
}
```

This allows viewport composition/clipping logic to stay shared where appropriate, while keeping the final output artifacts semantically correct.

The architecture should not assume that all protocols immediately converge on the same timing model.
A realistic intermediate state is:
- kitty: persistent scene-owned visible render products
- sixel: render-time changed image products

and the type/model boundary should make that distinction visible.

## Persistent vs delta image output

One important lesson from the current code is that image output has two distinct shapes:

### Persistent scene-owned image output
This is increasingly the kitty model.
The scene owns logical placements and can answer:
- what image content is currently visible?
- which placements survive regardless of whether text rerender happened?

### Render-delta-owned image output
This is still the sixel model today.
Sixel redraw is driven by changed text rects / changed viewport regions, not by a fully scene-owned visible image bundle.

That means the architecture should avoid forcing a false uniformity here too early.
A good intermediate design is to support both:
- persistent visible image bundles
- changed image render bundles

with a cleaner path to move protocols between those categories later if desired.

## What should stay shared

The following belong in the shared image scene / common infrastructure:
- asset ownership
- logical placement lifetime
- screen domain separation (main / alt)
- clear/reset/delete integration
- pane clipping
- floating pane occlusion
- canonical/text-flow anchoring helpers
- viewport intersection helpers
- per-client scene reconciliation / dirty tracking

## What should stay flavor-specific

The following should stay protocol/output-flavor-specific:
- exact kitty explicit placement serialization
- exact kitty virtual placement + placeholder redraw semantics
- sixel redraw details
- kitty query/response details
- kitty delete command variants
- kitty relative placement semantics later

## Proposed target data model

Below is a rough conceptual target, not an exact Rust API commitment.

```rust
struct PaneImageScene {
    assets: HashMap<ImageAssetId, ImageAsset>,
    placements: HashMap<LogicalPlacementId, LogicalPlacement>,
    placeholder_cells: Vec<PlaceholderCellRef>,
}

struct LogicalPlacement {
    id: LogicalPlacementId,
    asset_id: ImageAssetId,
    protocol_identity: Option<ProtocolPlacementIdentity>,
    flavor: PlacementFlavor,
    anchor: PlacementAnchor,
    lifecycle_scope: LifecycleScope,
    content_flow: ImageContentFlow,
}

enum PlacementAnchor {
    Explicit(FlowAnchor),
    PlaceholderTextFlow,
}
```

The important part is that `LogicalPlacement` owns the semantic placement flavor, while placeholder cells are separate occupancy references into that placement.

## Why the current model likely plateaus

The current proof branch moved a lot of important state into `PaneImageScene`, which was the right step.
But the current model still tends to flatten toward:
- generic occupancy
- generic source rect
- generic visible output artifact

That flattening is probably fine for:
- explicit kitty placements
- some sixel-like redraw cases

But it appears too lossy for:
- kitty placeholder-native redraw fidelity

The experiments suggest that native placeholder behavior depends on stronger preservation of:
- placement flavor
- logical placement identity
- text-flow occupancy semantics
- and perhaps a less reconstructed, more flavor-native redraw path

## Proposed migration path

This should be done incrementally rather than as a giant rewrite.

### Phase A: make placement flavor first-class in the shared scene
- move from ad-hoc kitty placement mode metadata toward a true `PlacementFlavor`
- make logical placements own flavor-specific geometry/identity
- make placeholder cells reference logical placements directly

### Phase B: separate render products by flavor
- stop using one kitty-visible-chunk path for both explicit and placeholder-backed content
- have the scene emit:
  - explicit kitty render products
  - placeholder kitty render products
  - sixel render products

### Phase C: centralize shared composition helpers
- clipping helpers should operate on logical placements / occupancy references
- output should receive flavor-specific render products already composed for the viewport
- keep the distinction between persistent visible image products and changed/delta image products explicit while protocols still differ

### Phase D: revisit placeholder fidelity
- after the above refactor, re-test direct Kitty/Ghostty vs Zellij-interposed behavior
- if fidelity is still off, the remaining issue will be much more localized and easier to reason about

## Non-goals for the immediate rearchitecture

This design pass is **not** proposing that we immediately solve:
- animation
- quotas/eviction
- every delete variant
- every query variant
- relative placements
- performance optimization

The immediate goal is narrower:
- stop losing semantics in the middle layer
- make the scene expressive enough that faithful output is possible later

## Success criteria for the rearchitecture pass

The rearchitecture is successful if:
- the shared scene remains the home for lifecycle/composition logic
- explicit and placeholder kitty paths no longer have to masquerade as the same final output primitive
- placeholder occupancy refers to logical placements, not just loosely reconstructed ids
- protocol-specific output layers can render from flavor-specific products
- the distinction between persistent visible image products and changed/delta image products is represented clearly where it still exists
- the current proof branch behavior is preserved or improved during migration

## Immediate next coding direction

When we return from stabilization work, the first concrete code move should likely be:

1. introduce a stronger `PlacementFlavor` shape in `PaneImageScene`
2. make placeholder cells reference `LogicalPlacementId`
3. stop deriving placeholder output from generalized kitty chunks
4. move toward flavor-specific render-product generation

That is probably the smallest meaningful step toward the eventual architecture without throwing away the proof branch's working functionality.
