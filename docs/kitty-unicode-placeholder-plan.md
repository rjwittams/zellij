# Kitty Unicode placeholder support plan

Status: historical / transitional design note.

This document is still useful for the original placeholder implementation shape and protocol notes,
but it is no longer the main source of truth for current status or future architecture.

Prefer these documents for current work:
- `docs/kitty-graphics-checklist.md` for implementation/stabilization status
- `docs/kitty-image-rearchitecture-sketch.md` for the next architecture pass

Goal: support the real-app kitty path used by `ratatui-image` / Ghostty-style placeholder rendering.

This builds on the existing Zellij kitty asset/placement work and the checklist in:
- `docs/kitty-graphics-checklist.md`

## What the docs and emitters say

## Protocol shape
Placeholder mode is not just another normal explicit placement.

There are two parts:

1. **Transmit/create virtual placement**
   - `a=T,U=1,...` can transmit and create a virtual placement in one step
   - or `a=p,U=1,i=<image_id>,c=<cols>,r=<rows>` can create a virtual placement for an already transmitted image
2. **Display via ordinary text cells**
   - the host app prints `U+10EEEE`
   - row/column are encoded by combining diacritics
   - image id is encoded in foreground color
   - placement id can be encoded in underline color

This means placeholder display is **text-flow integrated**.

## Observed `ratatui-image` behavior
`ratatui-image` does the following:

1. transmit chunked RGBA asset with virtual placement creation:
   - `i=<id>,a=T,U=1,f=32,t=d,s=<w>,v=<h>,m=...`
2. print placeholder rows as text:
   - first cell contains a symbol string including:
     - save cursor
     - foreground color carrying image id bytes
     - `U+10EEEE`
     - row diacritic
     - col diacritic
     - third diacritic for the image id high byte
   - later placeholder cells in that row are marked skipped by the TUI buffer, but terminal cursor movement in the symbol string effectively paints the row

Important consequence for Zellij:
- raw placeholder glyphs cannot just be treated as inert text forever
- but the first practical target is likely an emitter that writes placeholders as actual per-cell text, not only via one giant symbol string in the first cell
- Ghostty/chafa/docs all still confirm the same underlying placeholder contract

## Minimum viable interpretation model

## A. Asset store stays kitty-native
Keep current kitty asset logic for:
- `f=32`, `f=100`
- chunk assembly with `m=1/0`
- protocol image id remapping

Add enough metadata to know an asset/placement is usable in placeholder mode.

## B. Placeholder occupancy is pane-flow state
We need pane-owned state representing:
- image id
- optional placement id
- placeholder row
- placeholder col
- the canonical/text-flow location of the placeholder cell

Suggested conceptual shape:

```rust
struct KittyPlaceholderCell {
    image_id: u32,
    placement_id: Option<u32>,
    placeholder_row: u16,
    placeholder_col: u16,
}
```

This is not necessarily the final storage layout, but it is the information we need.

## C. Placeholder-backed placements should anchor to text flow
Unlike explicit placements, placeholder-backed placements should be anchored by the placeholder cells themselves.

That means:
- reflow should naturally move them with text
- the scene model should treat them as pane-flow-owned placements
- canonical-flow anchoring is the right fit

## Suggested implementation phases

### Phase 1: detect and suppress placeholder text leakage
Definition of done:
- recognize kitty placeholder characters/diacritics when they occur as actual printed text
- avoid rendering them as garbage text in the pane
- keep ordinary text behavior unchanged otherwise

Likely parser signal to detect:
- base char `U+10EEEE`
- following combining marks in the known kitty row/column table
- image id from current foreground color
- optional placement id from underline color later

### Phase 2: record placeholder occupancy in pane image scene
Definition of done:
- when placeholder text is printed, store logical placeholder cell references
- placeholder cells belong to canonical pane flow
- they survive scroll/reflow similarly to text

Likely first-pass API direction:
- `Grid::print(...)` / character ingestion detects placeholder cells
- informs `PaneImageScene`
- placeholder cells can be hidden from ordinary text rendering

### Phase 3: generate visible kitty chunks from placeholder occupancy
Definition of done:
- placeholder occupancy becomes visible image chunks in current viewport
- chunk source rect is derived from placeholder row/column bounds
- partial viewport visibility works

A coarse first pass could:
- gather all placeholder cells for an image/placement
- derive a bounding box
- render corresponding source/image region

A more correct pass would:
- respect actual placeholder row/column mapping for source projection

### Phase 4: virtual placement lifecycle semantics
Definition of done:
- support `a=p,U=1,...`
- placeholder disappearance updates visibility
- clear/reset/alt-screen semantics remain correct
- deletion semantics later hook into shared scene lifecycle

## Findings from the smoke harness / terminal matrix

The staged smoke harness now shows an important distinction between:
- direct terminal behavior in Kitty / Ghostty / WezTerm
- Zellij-interposed behavior

Key findings:

1. **Current placeholder support is now materially better than this document's midpoint captured**.
   - Real apps can render.
   - Restoring width-1 text-flow semantics for placeholder cells fixed the major Stage 4 smoke regression in Zellij-in-Kitty and Zellij-in-Ghostty.

2. **Normalization and projection still matter**.
   - Earlier direct Kitty/Ghostty vs Zellij differences were real.
   - But the remaining Stage 5 issue turned out to be strongly tied to preserving one-dimensional explicit sizing semantics and matching Zellij's internal occupancy/projection math, not just placeholder wire-form fidelity.
   - WezTerm still diverges materially both direct and interposed, so it remains a secondary comparison point rather than the primary design target.

3. **Explicit placement sizing is a real tradeoff, not a trivial bug**.
   - When both `c` and `r` are specified, the image is fit to the rectangle and may distort.
   - When only one is specified, aspect ratio can be preserved, but overflow / layout mismatch can occur.
   - Therefore, a simple serializer tweak is not a general fix.

## Architectural conclusion

The current shared scene direction is still correct, but we should avoid treating:
- explicit placements
- Unicode placeholder / virtual placements
as fully interchangeable output modes.

At the same time, the latest fixes show that the shared scene can preserve more native-looking behavior than this earlier note assumed, provided text-flow occupancy and explicit sizing semantics are modeled correctly.

The likely path to better fidelity is:
- keep a shared logical scene for ownership/lifecycle/reflow/composition
- **preserve placeholder origin semantics** in the model
- eventually prefer placeholder-preserving output for placeholder-originated kitty content
  instead of always flattening everything into explicit `a=p`-style redraw.

## Known open questions

1. Should first implementation target:
   - true per-cell placeholder text parsing in Zellij, or
   - enough compatibility for `ratatui-image`'s single-symbol-row trick?

2. Where exactly should placeholder cells live?
   - separate structure in `PaneImageScene`
   - annotations on rows/cells in `Grid`
   - hybrid

3. How much of placeholder source mapping is needed for first success?
   - exact row/col mapping
   - or just bounding-box image display for first proof

4. What minimum provenance should be kept so placeholder-originated content does not get irreversibly flattened into explicit-placement output semantics?

## Recommended next coding step
Keep current behavior for compatibility, but add enough provenance to avoid digging deeper into the
"everything becomes explicit redraw" hole.

That suggests the next implementation step should be:
- add output-mode / origin-mode metadata for kitty placements or chunks
- keep current output behavior initially
- then experiment with placeholder-preserving output for placeholder-originated content
