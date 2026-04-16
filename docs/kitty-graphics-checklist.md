# Kitty graphics protocol checklist

Working checklist for Zellij kitty graphics support, based on:
- https://sw.kovidgoyal.net/kitty/graphics-protocol/
- observed real emitters: direct test script, `chafa`, `ratatui-image`

Scope for now:
- target the parts that are useful for real apps in Ghostty/Zellij
- explicitly defer animation for now
- prefer native kitty-in -> kitty-out handling when possible

## Suggested implementation order

### 1. Core APC and asset ingest
- [x] APC bytes reach Zellij through `vte`
- [x] Parse kitty APC envelope: `ESC _ G ... ; payload ESC \`
- [x] Support direct payload transmission `t=d`
- [x] Support `f=100` PNG payloads
- [x] Support `f=32` RGBA payloads
- [ ] Support `f=24` RGB payloads
- [x] Support chunked transfer with `m=1` / `m=0`
- [ ] Support compression `o=z`
- [ ] Support non-direct transmission media:
  - [ ] `t=f` file
  - [ ] `t=t` temp file
  - [ ] `t=s` shared memory
  - [ ] `S`/`O` partial reads for file/shm media

### 2. Basic image creation and display
- [x] Support `a=T` transmit-and-display for explicit placements
- [x] Support transmit-then-redraw internally via asset/placement split
- [x] Support image ids `i=`
- [x] Remap protocol image ids into pane-safe internal ids
- [ ] Support image numbers `I=` requested from terminal
- [x] Support placement ids `p=`
- [x] Replacement semantics for same `(image_id, placement_id)`
- [x] Multiple placements for same image id when `p` is omitted/zero

### 3. Placement geometry and rendering
- [x] `c`, `r` placement sizing in cells
- [x] `x`, `y`, `w`, `h` source rectangle crop
- [x] `X`, `Y` cell-local pixel offsets
- [x] `z` z-index in composed scene
- [x] `C=1` no-cursor-movement placement output
- [ ] Default kitty cursor movement semantics when `C!=1`
- [x] Viewport clipping
- [x] Floating-pane clipping (coarse first pass)
- [x] Scene diff + redraw reconciliation

### 4. Pane lifecycle and terminal actions
- [x] Reset clears visible graphics
- [x] `ESC[2J` clears graphics
- [x] Alternate screen starts empty and restores main-screen image scene on exit
- [x] Images scroll with pane text
- [x] Basic reflow-stable anchoring improved toward canonical flow
- [ ] Finish reflow/resize correctness near boundaries
- [ ] Page-margin / scroll-region clipping semantics

### 5. Query / response behavior
- [x] Enough query behavior for some real-world detection to work (`chafa` appears happy)
- [ ] Explicitly verify kitty graphics query handling matches docs:
  - [ ] `a=q`
  - [ ] `i=` echoed in response
  - [ ] `OK` / error responses
- [ ] Query available transmission media behavior
- [ ] Placement/image existence acknowledgements for `a=p`
- [ ] Suppressing responses / quiet mode semantics (`q` handling audit)
- [ ] Requesting image ids via `I=`

### 6. Delete semantics
- [x] `a=d` delete command support for current real-app needs
- [x] Delete all visible placements (`a=d,d=A`)
- [ ] Delete by image id / placement id
- [ ] Delete by cell / row / column / z-index
- [ ] Upper/lower case storage-freeing semantics
- [ ] Abort partial upload on delete during chunked transfer

### 7. Unicode placeholder / virtual placement mode
This now works for real apps, and the smoke-harness placeholder stage is back to looking correct in Kitty/Ghostty-class behavior after restoring width-1 text-flow semantics for placeholder cells.

- [x] Support virtual placement creation with `U=1`
- [x] Support `a=T,U=1` combined transmit + virtual placement
- [x] Support `a=p,U=1` virtual placement without immediate explicit draw
- [x] Parse placeholder text cells using `U+10EEEE`
- [x] Parse row/column diacritics
- [x] Decode image id from foreground color
- [x] Decode placement id from underline color if needed
- [x] Treat placeholder cells as pane-flow-owned image anchors
- [x] Prevent raw placeholder glyph leakage into normal text rendering
- [x] Compose placeholder-backed image visibility into pane image scene
- [ ] Reflow placeholder-backed images correctly with text
- [x] Match smoke-harness placeholder fidelity well enough for current Kitty/Ghostty proof branch
- [ ] Finish auditing native Kitty/Ghostty fidelity under more aggressive resize/reflow cases
- [ ] Preserve placeholder semantics without WezTerm-like normalization artifacts

### 8. Relative placements
- [ ] `P=` / `Q=` parent placement references
- [ ] `H=` / `V=` relative offsets
- [ ] Parent lifetime semantics
- [ ] Error cases: `ETOODEEP`, `ECYCLE`, `ENOPARENT`, invalid virtual-relative combinations

### 9. Persistence / quota semantics
- [ ] Quota-aware asset eviction policy
- [ ] Prefer eviction of unplaced/unreferenced images
- [ ] Decide what host-terminal quota behavior Zellij must emulate vs ignore internally

### 10. Animation
Deferred for now.

- [ ] animation frame transfer
- [ ] animation control
- [ ] frame composition

## Stabilization shortlist

### Must stabilize now
- [x] direct kitty query response used by real autodetection
- [x] delete-all-visible lifecycle (`a=d,d=A`) for current real apps
- [x] placeholder-backed image disappears when app explicitly deletes visible images
- [ ] placeholder cleanup on erase/line clear/region clear beyond direct overwrite
- [ ] resize/reflow sanity for both explicit and placeholder paths
- [ ] tab/pane switch sanity across image and non-image states
- [x] Stage 4 placeholder smoke behavior now looks correct in Zellij-in-Kitty and Zellij-in-Ghostty
- [x] Stage 5 explicit sizing smoke behavior now looks correct in Zellij-in-Kitty and Zellij-in-Ghostty
- [x] document current fidelity status clearly: Kitty/Ghostty smoke path now good, WezTerm still diverges similarly direct vs interposed

### Defer until after stabilization
- [ ] `f=24`
- [ ] `o=z`
- [ ] file/shared-memory media (`t=f/t/s`)
- [ ] richer delete variants beyond current real-app needs
- [ ] relative placements
- [ ] quotas/eviction policy
- [ ] animation

## Real-app checkpoints

### Already working
- [x] Direct PNG kitty test script
- [x] `chafa` immediate kitty path
- [x] `chafa` autodetected kitty path in current environment
- [x] `ratatui-image` / `flotilla` autodetected kitty path
- [x] `yazi` kitty path with `ZELLIJ_SESSION_NAME` unset

### Next target
- [x] richer spec-driven smoke/example coverage
  - [x] explicit PNG
  - [x] explicit RGBA chunked
  - [x] Unicode placeholders
  - [x] query
  - [x] delete-all-visible
  - [ ] later: mixed sixel + kitty
  - [ ] capture/record cross-terminal behavior matrix more formally

## Notes from the docs that matter architecturally
- The protocol has a real distinction between:
  - asset transmission
  - placement creation/display
  - deletion/query/lifecycle
- Re-transmitting image data for an existing image id deletes prior placements for that image.
- Multiple placements for the same image id are valid.
- `C=1` is the correct cursor policy for our redraw path.
- Clear/reset/alt-screen semantics are protocol semantics, not just renderer details.
- Unicode placeholder mode is not just another serializer detail; it is text-flow integrated.
- Current proof branch shows that placeholder compatibility is achievable with a shared scene.
- Earlier placeholder fidelity concerns were materially improved by restoring width-1 text-flow semantics for placeholder cells; the smoke-harness placeholder stage now looks correct in Zellij-in-Kitty and Zellij-in-Ghostty.
- Explicit sizing fidelity for `c+r` / `c only` / `r only` was materially improved by preserving sizing intent on output and matching Zellij's internal occupancy prediction to the one-dimensional sizing mode.
- WezTerm still diverges in similar ways both direct and interposed, so remaining WezTerm differences should not be treated as the primary design reference.
