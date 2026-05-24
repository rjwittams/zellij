# Decomposition Notes

Session notes for the re-slicing of `feat/kitty-image-plumbing` and `spike/native-plugins`. Captures decisions, constraints, and the working sketch of the slice DAG. Working document — expect changes.

## Principles

- **Designed units, not ontogeny recap.** Each slice is a coherent unit a reviewer wants to read as one piece. The original commit timeline is input. We are not reconstructing it.
- **Vertical end-to-end deltas, not horizontal layers.** Each slice ships some capability standalone. "All transports before any output" is wrong; "minimum pipeline that renders one image" then layer up is right.
- **Bundle when the reviewer would reload the same mental model.** PNG/RGB/RGBA share the parsing surface — bundle. Transport variants share the transport mechanism — bundle. Split only when the second slice introduces new surface (new abstraction, new code area, new test pattern).
- **Bugs introduced and fixed inside the original timeline fold into the slice that lands the feature correctly.** No "feature commit + fix commit" pairs.
- **Ratcheting refactors collapse into the slice that introduces the structure.** Ship v1-clean directly. We are not shipping v0 followed by "and here's how I cleaned it up."
- **Pure noise — rebase fallout, undo-yourself commits, stale planning docs — deleted entirely.**
- **Coherent roadmap is the deliverable.** Present the truth of what was built, organized so reviewers would have to actively choose to ignore it. Not negotiation, not apology.

## Cross-cutting decisions

- **VTE is a deliverable, not an obstacle.** Either upstream-track PR (APC hooks API + rationale) with the fork as fallback path, or two parallel slices (upstream attempt + workaround). Ship final state only; iteration noise is deleted.
- **PNG / RGB / RGBA bundle into slice 2.** Kitty's payload formats share the parsing surface. Splitting them is fragmentation for fragmentation's sake. The `KittyImageFormat` enum lands with the foundation, not as a later refactor.
- **Transport variants bundle.** File / temp / shared-memory / direct + the ack/lifetime model. One slice covering the transport mechanism + all variants the protocol expects.
- **Asset quota + lifecycle pruning bundle.** Same lifecycle surface.
- **Multi-client / watcher reorder constraint.** Currently positioned late in the sketch. Earliest realistic position = immediately after generations slice (#4) — before generations, multi-consumer correctness across renders requires resending everything every frame. Upstream may push for this to move up on the argument "zellij is fundamentally multi-client, kitty support that breaks watchers is shipping a regression." Not freezing position; flagging for re-decision at upstream-conversation time.

## Spine sketch (kitty work — draft, not frozen)

Slice boundaries need code-level verification before any branches are created — read `zellij-server/src/panes/kitty.rs`, trace the call chain "APC bytes arrive → pixels rendered in pane", verify which "refactor:" commits are load-bearing.

Rough order:

1. **VTE prerequisite** — upstream-track PR with APC hooks API; fork as fallback.
2. **Kitty protocol foundation** — APC parsing + PNG/RGB/RGBA payload + `KittyImageFormat` enum + basic placement + deletion + damage redraw + `ImageOutput` per-client structure (the v1-clean version, not v0+refactor).
3. **Fragment pipeline + diff planner** — `image_fragment.rs`, `kitty_diff.rs`.
4. **Shared asset store with generations** — `kitty_asset_store.rs` + generation tracking.
5. **Unicode placeholder support.**
6. **Image transport mechanism + variants + lifetime/ack model** — `kitty_output_media.rs`, file/temp/shm/direct, always-ack + watermark lifetimes.
7. **Asset quota + lifecycle pruning.**
8. **Capability probing.**
9. **Multi-client / watcher support.** (Earliest realistic = after #4; may be reordered up per upstream request.)
10. *(parallel to spine, depends on #4)* **Plugin graphics API** — graphics command + `plugin_graphics_scene.rs`.

Each slice expected to be ~500–2000 LOC + matching tests.

## Side-quests (sibling slices off main — NOT children of the kitty spine)

| Cluster | Kitty-dependent? |
|---|---|
| Plugin graphics API + scene + test stabilization | Yes (child of #4) |
| Client-scoped plugin messaging | No |
| Plugin pane resize API | No |
| Client teardown panic fix | No (tiny standalone) |
| VFS module + caller_cwd fix + output macros + resolve_host_path | No (probably one bundled slice — all plugin-tile additions) |

## Not yet analyzed

- `spike/native-plugins` 20 unique commits above kitty-image-plumbing. Native plugin loader + andamento integration glue. Needs its own analysis pass — likely a mix of upstream-worthy plugin runtime work and fork-only integration glue.

## Methodology

- **No branches created yet** — analysis only. Slices are registered when their branch exists, not before.
- **Branch naming convention for kintsugi-managed slices**: `rjw<gen>/<order>-<slug>`, e.g. `rjw1/01-vte-apc-hooks`, `rjw1/02-kitty-foundation`. Existing branches (`feat/kitty-image-plumbing`, `spike/native-plugins`) stay as-is — pre-kintsugi, generation 0.

## Fold tables (commits → destination slice)

### Bug-introduce / bug-fix pairs to fold

| Fix commit | Folds into |
|---|---|
| `6ac7347d` honor kitty image deletion + placeholder cleanup | foundation (#2) |
| `c2f5eab6` preserve 1D sizing semantics | foundation (#2) |
| `46ae8c1b` inherit omitted placeholder diacritics | placeholder (#5) |
| `4011ce6d` fix unnamed placeholder registration | placeholder (#5) |
| `46b40de2` fix unnamed placement semantics | foundation (#2) |
| `af1991c6` fix host id collision + pending cleanup | fragment/diff (#3) |
| `9ba58e1b` fix asset id exhaustion | asset store / quota (#4 or #7) |
| `c18193e5` fix media session rename cleanup | transport (#6) |
| `f4c55f7c` fix raw media resize handling | transport (#6) |
| `958c5a56` fix asset lifecycle pruning | generations (#4) / pruning (#7) |
| `25bd6e6a` preserve assets across alt screen | shared asset store (#4) |
| `14e1cbe8` improve explicit geometry + placement flow | foundation (#2) — substantive, not just "fix" |
| `cd26de30` support image numbers + explicit fragments | fragment (#3) — substantive |
| `221202d2` complete phase-a parsing | foundation (#2) — substantive |
| `8b9ab090` / `bf2a7e35` / `a6f5bb70` / `da076f26` | into fragment / placeholder slices |

### Pure noise — delete entirely

| Commit | Why |
|---|---|
| `0722bd97` Fix kitty branch rebase fallout | rebase artifact |
| `38172967` Update tests for kitty image config | snapshot follow-on of config addition |
| `5413cb49` Fix rebased screen test constructor | rebase artifact |
| `8e6190b6` move exploratory kitty docs/tests out of tree | undoes earlier additions; never should have been added in sliced version |
| `63fba511` move reflow wrap test out of tree | same |
| `67df4cd6` drop stray superpowers plan | undoes a doc added mid-branch |
| `8625652a` ignore local worktrees | unrelated tweak |

### Ratcheting refactors — collapse into the slice that lands the structure

- **`ImageOutput` extraction** (`da0c6727` / `59ca25ea` / `0c284240` / `325a1c36` / `b6f1c684` / `3e3b6dfa` — six commits): all collapse into foundation (#2). Ship the v1-clean structure from the start.
- `8a398f36` simplify image data types → foundation (#2).
- `1e7b37af` return visible images through pane render output → foundation (#2) or fragment (#3).
- `5951f6e5` move placeholder helpers → placeholder (#5).
- `6244b3ed` rendered image state plumbing — wide-reaching; if it sets up the next slice's structure, fold; otherwise its own slice.

### Combined commits to split

- `9284198f Optimize kitty image transport and state tracking` — one commit that introduces `kitty_output_media.rs` AND the transport-config option AND does asset_store rework. Needs splitting at slice-design time: transport machinery → #6, config option → #6 or its own slice, asset_store rework → into asset store (#4) if pre-#4 or into #6.
