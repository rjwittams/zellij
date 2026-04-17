# Pane Image Frame Unification Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `PaneRenderOutput`'s split image bundles with one pane-level image input model, and derive kitty frame redraws centrally in `ImageOutput`.

**Architecture:** Panes should report one image contribution per frame: the current visible kitty scene, the current sixel redraw chunks, and the row-based damage map already used by grid rendering. `ImageOutput` remains the owner of client-specific previous-frame kitty state and uses that plus the pane damage rows to decide whether to emit a full kitty scene replacement or only a kitty redraw subset.

**Tech Stack:** Rust, `zellij-server`, existing `PaneRenderOutput` / `Output` / `ImageOutput` pipeline, current kitty and sixel rendering helpers.

---

## Chunk 1: Replace the Pane-Level Data Model

### Task 1: Define the unified pane image input type

**Files:**
- Modify: `zellij-server/src/output/mod.rs`
- Reference: `zellij-server/src/panes/pane_image_scene.rs`
- Test: `zellij-server/src/output/unit/output_tests.rs`

- [ ] **Step 1: Write the failing compile-time test change**

Update one or more existing test helpers in `zellij-server/src/output/unit/output_tests.rs` so they construct the new pane image input type instead of `ImageRenderBundle`-only inputs. The change should intentionally leave production code uncompiled.

- [ ] **Step 2: Run the targeted test build to verify it fails**

Run: `cargo test -p zellij-server output::unit::output_tests::test_is_dirty_with_kitty_scene_diffs -- --exact`
Expected: FAIL to compile because the new pane image input type or fields do not exist yet.

- [ ] **Step 3: Add the new output types**

In `zellij-server/src/output/mod.rs`, replace the two image fields on `PaneRenderOutput` with one new struct. Keep `ImageRenderBundle` for reusable protocol payloads, but stop using it for two different pane-level meanings.

Suggested shape:

```rust
#[derive(Debug, Clone, Default)]
pub struct PaneImageRenderOutput {
    pub kitty_scene: KittyRenderBundle,
    pub sixel_chunks: Vec<SixelImageChunk>,
    pub changed_rects: HashMap<usize, usize>,
}

#[derive(Debug, Clone, Default)]
pub struct PaneRenderOutput {
    pub character_chunks: Vec<CharacterChunk>,
    pub raw_vte_output: Option<String>,
    pub image_output: PaneImageRenderOutput,
}
```

Keep names conservative if a better local name appears during editing, but the semantics should stay the same:

- `kitty_scene`: full current visible kitty scene for the pane
- `sixel_chunks`: current sixel redraw chunks for this frame
- `changed_rects`: row-based damage signal for central redraw derivation

- [ ] **Step 4: Run the targeted test build to verify the new types compile**

Run: `cargo test -p zellij-server output::unit::output_tests::test_is_dirty_with_kitty_scene_diffs -- --exact`
Expected: FAIL later in the pipeline because pane producers and output APIs still use the old split bundle model.

- [ ] **Step 5: Commit**

```bash
git add zellij-server/src/output/mod.rs zellij-server/src/output/unit/output_tests.rs
git commit -m "refactor: unify pane image render input model"
```

### Task 2: Update pane producers to emit the unified input

**Files:**
- Modify: `zellij-server/src/panes/grid.rs`
- Modify: `zellij-server/src/panes/terminal_pane.rs`
- Modify: `zellij-server/src/panes/plugin_pane.rs`
- Test: `zellij-server/src/panes/unit/grid_tests.rs`

- [ ] **Step 1: Write the failing test**

Adjust `kitty_placeholder_reflow_emits_changed_placeholder_render_bundle` in `zellij-server/src/panes/unit/grid_tests.rs` so it asserts through the new `render_output.image_output` shape. Keep the expected behavior identical.

- [ ] **Step 2: Run the targeted test to verify it fails**

Run: `cargo test -p zellij-server panes::unit::grid_tests::kitty_placeholder_reflow_emits_changed_placeholder_render_bundle -- --exact`
Expected: FAIL to compile because `Grid::render()` still returns the old split image fields.

- [ ] **Step 3: Implement the minimal producer changes**

In `zellij-server/src/panes/grid.rs`:

- stop constructing `visible_image_render_bundle`
- stop constructing `damage_redraw_image_render_bundle`
- return one `PaneImageRenderOutput`
- keep using the existing `changed_rects` and `sixel_image_chunks`
- set `kitty_scene` from `self.visible_kitty_render_bundle(...)`
- remove the precomputed `KittyDamageRedraw` derivation from grid render output assembly

In `zellij-server/src/panes/terminal_pane.rs` and `zellij-server/src/panes/plugin_pane.rs`:

- populate only `kitty_scene`
- leave `sixel_chunks` and `changed_rects` empty/default
- preserve existing `None` behavior when no visible kitty scene exists

- [ ] **Step 4: Run the targeted pane tests to verify they pass**

Run: `cargo test -p zellij-server panes::unit::grid_tests::kitty_placeholder_reflow_emits_changed_placeholder_render_bundle -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add zellij-server/src/panes/grid.rs zellij-server/src/panes/terminal_pane.rs zellij-server/src/panes/plugin_pane.rs zellij-server/src/panes/unit/grid_tests.rs
git commit -m "refactor: emit unified pane image render output"
```

## Chunk 2: Derive Kitty Redraw Centrally in ImageOutput

### Task 3: Replace the split output ingestion APIs with one unified path

**Files:**
- Modify: `zellij-server/src/output/image_output.rs`
- Modify: `zellij-server/src/output/mod.rs`
- Modify: `zellij-server/src/ui/pane_contents_and_ui.rs`
- Test: `zellij-server/src/output/unit/output_tests.rs`

- [ ] **Step 1: Write the failing test**

Add or update output tests so the call sites use one ingestion API for pane image data instead of separate `add_image_render_bundle_*` and `add_damage_redraw_image_render_bundle_*` calls.

Recommended helper shape in `zellij-server/src/output/unit/output_tests.rs`:

```rust
fn pane_image_output_with_kitty_scene(explicit_chunks: Vec<KittyImageChunk>) -> PaneImageRenderOutput
fn pane_image_output_with_sixels(sixel_chunks: Vec<SixelImageChunk>) -> PaneImageRenderOutput
```

- [ ] **Step 2: Run the targeted test to verify it fails**

Run: `cargo test -p zellij-server output::unit::output_tests::test_serialize_emits_kitty_damage_redraw_without_scene_change -- --exact`
Expected: FAIL to compile because the unified ingestion path does not exist yet.

- [ ] **Step 3: Implement the API consolidation**

In `zellij-server/src/output/mod.rs` and `zellij-server/src/output/image_output.rs`:

- remove `add_damage_redraw_image_render_bundle_to_client`
- remove `add_damage_redraw_image_render_bundle_to_multiple_clients`
- replace the old `add_image_render_bundle_*` APIs with one pane-image-ingestion API that accepts the new pane-level image struct

In `zellij-server/src/ui/pane_contents_and_ui.rs`:

- replace the two image calls with one call passing `render_output.image_output`

Keep the per-client accumulation model in `ImageOutput`, but change the inputs it stores:

- current kitty scene from `kitty_scene`
- current sixel redraw chunks from `sixel_chunks`
- current damage rows from `changed_rects`

- [ ] **Step 4: Run the targeted output test to verify it now reaches behavior assertions**

Run: `cargo test -p zellij-server output::unit::output_tests::test_serialize_emits_kitty_damage_redraw_without_scene_change -- --exact`
Expected: test compiles and either passes or fails on runtime behavior, but no longer fails because of the removed split APIs.

- [ ] **Step 5: Commit**

```bash
git add zellij-server/src/output/image_output.rs zellij-server/src/output/mod.rs zellij-server/src/ui/pane_contents_and_ui.rs zellij-server/src/output/unit/output_tests.rs
git commit -m "refactor: ingest pane image output through one path"
```

### Task 4: Move kitty damage-redraw derivation into ImageOutput

**Files:**
- Modify: `zellij-server/src/output/image_output.rs`
- Reference: `zellij-server/src/panes/pane_image_scene.rs`
- Test: `zellij-server/src/output/unit/output_tests.rs`

- [ ] **Step 1: Write the failing behavior test**

Add or adjust tests to verify:

- unchanged kitty scene + non-empty `changed_rects` emits kitty redraw output without kitty delete-all
- changed kitty scene still emits kitty delete-all plus full current kitty scene
- empty `changed_rects` with unchanged kitty scene emits no kitty output

If practical, extend `test_serialize_emits_kitty_damage_redraw_without_scene_change` instead of adding a separate test.

- [ ] **Step 2: Run the targeted test to verify it fails for the right reason**

Run: `cargo test -p zellij-server output::unit::output_tests::test_serialize_emits_kitty_damage_redraw_without_scene_change -- --exact`
Expected: FAIL on the behavior assertion until central kitty redraw derivation is implemented.

- [ ] **Step 3: Implement the minimal central derivation**

In `zellij-server/src/output/image_output.rs`:

- remove `kitty_damage_redraw_chunks`
- remove `kitty_damage_redraw_placeholder_renders`
- add stored damage rows to the current per-client image state
- in `prepare_render_body_for_client()`, when the kitty scene is unchanged:
  - build `KittyDamageRedraw` from stored `changed_rects`
  - filter `current_kitty_chunks` and `current_kitty_placeholder_renders` against that damage
  - emit only that filtered subset
- when the kitty scene changed:
  - keep the current full-scene replacement path unchanged
- clear consumed damage rows after frame preparation

Implement the filtering locally in `image_output.rs` if that keeps the change smallest. If duplication becomes awkward, extract a narrow shared helper rather than reintroducing a second bundle.

- [ ] **Step 4: Run the targeted tests to verify they pass**

Run:

```bash
cargo test -p zellij-server output::unit::output_tests::test_is_dirty_with_kitty_scene_diffs -- --exact
cargo test -p zellij-server output::unit::output_tests::test_serialize_emits_kitty_damage_redraw_without_scene_change -- --exact
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add zellij-server/src/output/image_output.rs zellij-server/src/output/unit/output_tests.rs
git commit -m "refactor: derive kitty redraw from pane damage"
```

## Chunk 3: Cleanup and Regression Coverage

### Task 5: Remove dead helpers and validate the full rendering slice

**Files:**
- Modify: `zellij-server/src/panes/grid.rs`
- Modify: `zellij-server/src/output/image_output.rs`
- Modify: `zellij-server/src/output/mod.rs`
- Modify: `zellij-server/src/output/unit/output_tests.rs`
- Modify: `zellij-server/src/panes/unit/grid_tests.rs`

- [ ] **Step 1: Remove obsolete code paths**

Delete any no-longer-used:

- split image bundle constructors/helpers in tests
- split output API wrappers
- now-unused imports such as `KittyDamageRedraw` in pane render assembly
- comments that still describe the old two-bundle model

- [ ] **Step 2: Run formatting**

Run: `cargo fmt --all`
Expected: formatting completes without error

- [ ] **Step 3: Run focused regression coverage**

Run:

```bash
cargo test -p zellij-server output::unit::output_tests
cargo test -p zellij-server panes::unit::grid_tests::kitty_placeholder_reflow_emits_changed_placeholder_render_bundle -- --exact
```

Expected: PASS

- [ ] **Step 4: Run the broader server test target if the focused slice is clean**

Run: `cargo test -p zellij-server`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add zellij-server/src/panes/grid.rs zellij-server/src/output/image_output.rs zellij-server/src/output/mod.rs zellij-server/src/output/unit/output_tests.rs zellij-server/src/panes/unit/grid_tests.rs
git commit -m "test: cover unified pane image rendering path"
```
