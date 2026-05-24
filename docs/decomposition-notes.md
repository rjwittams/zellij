# Decomposition Notes

Working notes for the re-slicing of `feat/kitty-image-plumbing` and `spike/native-plugins`. Captures decisions, constraints, and the working sketch of the slice DAG. Expect changes.

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
- **PNG / RGB / RGBA bundle into the foundation slice.** Kitty's payload formats share the parsing surface. The `KittyImageFormat` enum lands with the foundation, not as a later refactor.
- **Transport variants bundle.** File / temp / shared-memory / direct + the ack/lifetime model. One slice covering the transport mechanism + all variants the protocol expects.
- **Asset quota + lifecycle pruning is correctness, not polish.** A misbehaving program could exhaust memory uploading large images. Ships before transport variants (External media makes the problem worse, but the basic problem exists with Direct uploads too).
- **Multi-client / watcher support ships before quota.** zellij is fundamentally multi-client; shipping kitty support that breaks watchers is shipping a regression. Earliest realistic position = after generations + diff planner (#4), because before generations multi-consumer correctness requires re-sending everything every frame.
- **Unclipped rendering is acceptable for the foundation slice.** Boundary overflow is a visible bug, not a correctness gate. Clipping is its own slice that follows immediately.

## Spine sketch (kitty work — draft)

Order reflects current understanding. Not frozen — review at each landing.

1. **VTE prerequisite** — upstream-track PR with APC hooks API; fork as fallback.
2. **Foundation** — APC parser (full kitty grammar — transmit, placement, delete, query), payload formats (PNG / RGB / RGBA with `KittyImageFormat` enum), zlib compression, direct payload transport only, basic absolute placement (no relative), deletion, query/response, per-pane `PaneImageScene` state, per-client `ImageOutput` structure, direct-only output serialization, **unclipped rendering** (boundary overflow is a known visible bug), minimal `KittyAssetStore` (id allocator + insert/get/remove; `KittyAssetData` enum with only `Image` variant; **no `generation` field**; no quota; no ref-counting).
3. **Pane-boundary clipping** — carve clipping functions from `image_fragment.rs` (`clip_image_fragment`, `visible_image_fragments`, `clip_kitty_explicit_fragment`, `clip_sixel_fragment`, `clip_kitty_placeholder_fragment`, `build_split_explicit_fragment`, `synthesize_split_fragment_placement_id`). No diff planner yet.
4. **Generations + diff planner** — one slice. Add `generation: u64` to `KittyAsset`, increment on insert. Add `KittyScenePlan` / `KittySceneState` / `plan_kitty_scene` (`kitty_diff.rs`). Connect to render pipeline so unchanged assets/placements aren't re-emitted every frame.
5. **Multi-client / watcher support** — per-watcher state separation, watcher capability application, multi-client image output paths. Earliest realistic position by code dependency (requires generations from #4).
6. **Asset quota + eviction** — `decoded_bytes_total`, `decoded_byte_quota`, `placement_ref_counts`, `asset_order` LRU, `evict_to_decoded_byte_quota`. Correctness — prevents memory exhaustion.
7. **Unicode placeholder rendering** — `serialize_placeholder_render`, `serialize_virtual_placeholder_placement`, geometry selectors beyond `Cursor` in delete dispatch.
8. **Image transport variants + lifetime/ack model** — `KittyAssetData::External` variant (additive), `KittyExternalMedia*`, file/temp/shm readers, lifetime modes (always-ack, watermark), transport selection config.
9. **Capability probing** — `kitty_image_id_allocator`, `screen.rs` probe machinery, transport selection from capabilities, per-client probe, watcher probe application.

Estimated foundation (#2) size: ~2500–3500 LOC of code + tests across `panes/kitty.rs`, `panes/pane_image_scene.rs`, `output/image_output.rs`, plus VTE integration, `grid.rs` APC plumbing, `terminal_pane.rs`/`screen.rs` touches, minimal asset store (~100 LOC).

## Side-quests (sibling slices off main — NOT children of the kitty spine)

| Cluster | Kitty-dependent? |
|---|---|
| Plugin graphics API + scene + test stabilization | Yes (child of #4) |
| Client-scoped plugin messaging | No |
| Plugin pane resize API | No |
| Client teardown panic fix | No (tiny standalone) |
| VFS module + caller_cwd fix + output macros + resolve_host_path | No (probably one bundled slice — all plugin-tile additions) |

## Verifications applied (slice 2)

**Clipping cleanly separable from diff planner** — `image_fragment.rs:339-379` clipping functions are pure over `PaneGeom` + chunks; zero references to `KittyScenePlan` / `KittySceneState`. The `PreparedAfterTextImages` struct conflates them structurally but functionally they're independent. Slice 3 ships clipping without diff machinery.

**Minimal asset store v1 feasible** — of 715 lines in `kitty_asset_store.rs`, foundation needs ~100:
- `KittyAssetFormat` enum
- `KittyAssetData::Image(KittyImageData)` enum (only one variant; `External` added additively in #8)
- `KittyAsset { data }` — **no `generation` field** (added in #4)
- `KittyAssetStore { image_id_allocator, assets: HashMap }`
- Methods: `next_asset_id`, `insert_asset`, `asset`, `asset_mut`, `remove_asset`, `image_data`, `image_dimensions`

Deferred (~615 LOC) split cleanly:
- **#4 (generations)**: `generation: u64` field + bump-on-insert (~10 lines)
- **#6 (quota)**: `decoded_bytes_total`, `decoded_byte_quota`, `placement_ref_counts`, `asset_order`, `evict_to_decoded_byte_quota` (~80 lines)
- **#8 (transports)**: `KittyAssetData::External` variant, `KittyExternalMedia*`, file/temp/shm readers, `materialized_payload` etc. (~390 lines)

All purely additive — no retrofit needed in earlier slices.

## Not yet analyzed

- `spike/native-plugins` 20 unique commits above kitty-image-plumbing. Native plugin loader + andamento integration glue. Needs its own analysis pass — likely a mix of upstream-worthy plugin runtime work and fork-only integration glue.

## Methodology

- **No branches created yet** — analysis only. Slices are registered when their branch exists, not before.
- **Branch naming convention for kintsugi-managed slices**: `rjw<gen>/<order>-<slug>`, e.g. `rjw1/01-vte-apc-hooks`, `rjw1/02-kitty-foundation`. Existing branches (`feat/kitty-image-plumbing`, `spike/native-plugins`) stay as-is — pre-kintsugi, generation 0.

## Fold tables (commits → destination slice)

Slice numbers below refer to the spine sketch above (current numbering).

### Bug-introduce / bug-fix pairs to fold

| Fix commit | Folds into |
|---|---|
| `6ac7347d` honor kitty image deletion + placeholder cleanup | foundation (#2) |
| `c2f5eab6` preserve 1D sizing semantics | foundation (#2) |
| `46ae8c1b` inherit omitted placeholder diacritics | unicode placeholder (#7) |
| `4011ce6d` fix unnamed placeholder registration | unicode placeholder (#7) |
| `46b40de2` fix unnamed placement semantics | foundation (#2) |
| `af1991c6` fix host id collision + pending cleanup | generations + diff (#4) |
| `9ba58e1b` fix asset id exhaustion | quota (#6) |
| `c18193e5` fix media session rename cleanup | transports (#8) |
| `f4c55f7c` fix raw media resize handling | transports (#8) |
| `958c5a56` fix asset lifecycle pruning | quota (#6) |
| `25bd6e6a` preserve assets across alt screen | generations + diff (#4) — uses generations |
| `14e1cbe8` improve explicit geometry + placement flow | foundation (#2) — substantive, not just "fix" |
| `cd26de30` support image numbers + explicit fragments | clipping (#3) — substantive |
| `221202d2` complete phase-a parsing | foundation (#2) — substantive |
| `8b9ab090` / `bf2a7e35` / `a6f5bb70` / `da076f26` | into clipping (#3) / foundation (#2) per topic |

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
- `1e7b37af` return visible images through pane render output → foundation (#2).
- `5951f6e5` move placeholder helpers → unicode placeholder (#7).
- `6244b3ed` rendered image state plumbing — wide-reaching; likely foundation (#2) for the structure, with later sites updated by their slices.

### Combined commits to split

- `9284198f Optimize kitty image transport and state tracking` — one commit that introduces `kitty_output_media.rs` AND the transport-config option AND does asset_store rework. Needs splitting at slice-design time: transport machinery → #8, config option → #8 or its own slice, asset_store rework → into generations+diff (#4) or quota (#6) depending on what changed.
