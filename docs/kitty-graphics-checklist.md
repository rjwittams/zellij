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
- [ ] `a=d` delete command support
- [ ] Delete all visible placements
- [ ] Delete by image id / placement id
- [ ] Delete by cell / row / column / z-index
- [ ] Upper/lower case storage-freeing semantics
- [ ] Abort partial upload on delete during chunked transfer

### 7. Unicode placeholder / virtual placement mode
This is the big missing real-app feature for `ratatui-image` / Ghostty-style integrations.

- [ ] Support virtual placement creation with `U=1`
- [ ] Support `a=T,U=1` combined transmit + virtual placement
- [ ] Support `a=p,U=1` virtual placement without immediate explicit draw
- [ ] Parse placeholder text cells using `U+10EEEE`
- [ ] Parse row/column diacritics
- [ ] Decode image id from foreground color
- [ ] Decode placement id from underline color if needed
- [ ] Treat placeholder cells as pane-flow-owned image anchors
- [ ] Prevent raw placeholder glyph leakage into normal text rendering
- [ ] Compose placeholder-backed image visibility into pane image scene
- [ ] Reflow placeholder-backed images correctly with text

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

## Real-app checkpoints

### Already working
- [x] Direct PNG kitty test script
- [x] `chafa` immediate kitty path
- [x] `chafa` autodetected kitty path in current environment

### Next target
- [ ] `flotilla` / `ratatui-image` splash via kitty
  - likely blocked mainly on Unicode placeholders / virtual placements

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
