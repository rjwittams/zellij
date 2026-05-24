# Session Handoff — 2026-05-24

Pausing the slicing work. This document captures what was tried, why it failed, and the approach to resume with.

## Current state of the world

- **`/Users/robert/dev/zellij.kintsugi/`** — jj-colocated clone of `zellij-org/zellij`, with `rjwittams` as a second remote.
  - Local bookmarks: `main`, `feat/kitty-image-plumbing`, `spike/native-plugins`, `slices/meta`.
  - No slice branches currently exist locally or on `rjwittams`.
- **`rjwittams/zellij` (remote)**:
  - `feat/kitty-image-plumbing` and `spike/native-plugins` pushed (Phase A safety pushes from earlier session).
  - `slices/meta` pushed, currently at the "remove slice vte-upgrade" commit on top of the decomposition-notes history.
  - No `rjw1/*` branches.
- **kintsugi manifest** (on `slices/meta`):
  - Tracks `kitty-image-plumbing` (audience=upstream, branch=`feat/kitty-image-plumbing`) and `native-plugins` (audience=undecided, branch=`spike/native-plugins`, parents=[kitty-image-plumbing]).
  - These are pre-kintsugi gen-0 slices registered for tracking; not actual decomposed slices yet.
  - The aborted `vte-upgrade` slice (added then removed during this session) is no longer in the manifest, but the add+remove pair lives in the slices/meta git history. Optional cleanup task: `jj abandon` both commits and force-push to remove the trail. Not blocking.
- **Decomposition notes** at `docs/decomposition-notes.md` on `slices/meta` are valid. Principles, fold tables, side-quest list, spine sketch — all still good. Latest content reflects: vte-upgrade as slice 1, fork-switch folds into slice 2, the audience/kind/clipping decisions from the session.

## What was tried this session and why it failed

We tried two approaches to compose slice 1 (the pure upstream vte 0.15.0 upgrade):

1. **Cherry-pick `e76ac675` onto the wrong base.** First attempt branched from `e76ac675-` which is 14 commits *into* the kitty chain, not the merge-base with main. The resulting "slice 1" branch had all 14 prior kitty commits as preamble — claimed to be slice 1 but was actually slice 1 plus a pile of stuff that hadn't been authored yet.
2. **Manual sed-based migration onto main.** After abort, switched to applying the byte→`&[byte]` migration manually with perl over 147 sites. Got further but discarded before verifying the build, because the whole approach was wrong-framed.

Both attempts shared the failure: they treated slice composition as "author the slice from scratch (or by copying from one commit)" rather than as "transform the existing chain into the desired structure."

Several smaller compounding failures: confabulating commit counts, misclassifying the VTE story (upstream PR vs permanent fork vs upgrade-vs-fork-switch as two slices), baking in decisions inside "verification reports" without surfacing them, using the question tool too aggressively after being told not to.

## The framing we converged on

We are NOT trying to recapitulate the original commits. The point of the work is to end up with the SAME tree as `feat/kitty-image-plumbing`'s tip (modulo deliberately-dropped noise) reorganized into clean slice tips, each of which is a coherent reviewable PR.

The right method is a sequence of jj transformations on the chain:
- **Moves**: `jj rebase -r X --insert-after Y` to reorder a commit
- **Squashes**: `jj squash` to combine commits into one
- **Splits**: `jj split` to break a commit apart
- **Edits**: `jj diffedit` to surgically change a commit's content
- **Abandons (only for undone things)**: `jj abandon` for commits whose diff is net-zero on the tip (rebase fallout, undo-yourself commits, stale planning docs). Real code is NEVER abandoned, only moved/transformed.

Each transformation preserves the tip's tree (or changes it in a calculable way — only when abandoning net-zero noise).

When a transformation goes wrong: `jj op restore <op>` rewinds. Try a different transformation.

## Verification per transformation

The mechanism: snapshot the original tip's tree before starting, and after each transformation verify `jj diff -r <current-tip> -r <baseline-snapshot>` is empty (or matches the running "deliberately dropped" set).

- jj's first-class conflicts let us tolerate "broken in the middle" — intermediate commits don't need to compile.
- Only slice TIPS (what would be submitted as PRs) need to compile and have tests pass.
- End-to-end check: after all transformations, the cleaned chain's tip matches the original tip's tree minus the deliberate noise drops.

## What kintsugi (this tool) does and doesn't do for this workflow

- **`init`, `slice add/edit/rm`, `render`, `doctor`, `status`** — still useful. They manage the slice manifest, render the site, surface inconsistencies.
- **`compose`** — was built for "combine already-authored slice tips into a final composed branch." That's downstream of the transformation work, not the work itself. Possibly still useful at the end to produce the final PR-ready branches if we author tips separately. Not useful for the transformation steps themselves.
- **`tree-diff`** — directly applicable to the verification step. Use to compare current chain tip's tree vs baseline.
- **`mark-verified`, `flatten-test`** — applicable at the end of slicing per-slice, not during transformations.

Open question for next session: is the kintsugi tool the right shape for transformation-based work? Or does it need new verbs (e.g. a "transformation log" or "baseline tree" verb)?

## Concrete first step when resuming

1. Snapshot the tree at `feat/kitty-image-plumbing` tip. Record the commit id and `jj log -r <tip> --summary` for the tip's contents.
2. Pick a single transformation to try first. Candidate: take `e76ac675`, modify its Cargo.toml in place (via `jj diffedit` or `jj split` and edit) to point at `vte = "0.15.0"` from crates.io instead of the rjwittams fork. Verify the tip's tree differs from baseline only in the Cargo.toml/Cargo.lock vte source (path-vs-version).
3. Verify with `jj diff -r feat/kitty-image-plumbing -r <baseline-snapshot>` — should be empty before any transformation, then after step 2 should show only the deliberate dep target change.
4. Continue: next transformation moves the `apc_` impls out of the wip commit (11fdf272) into where slice 2's position will be — likely via `jj squash` or `jj rebase --insert-after`.
5. Repeat. Verify after every step.

## Mental rules to honor in next session

- DO NOT branch slices from main as a separate hand-authored chain. Transform the existing chain in place.
- DO NOT abandon any commit that contains real code. Abandon is only for net-zero noise.
- DO verify tree-equivalence after every transformation.
- DO NOT batch multiple transformations before verifying.
- Slice tips compile + tests pass. Intermediate states do not have to.
- Stop and ask before any destructive remote operation (force-push, branch delete).

## Things from this session that survived

- `docs/decomposition-notes.md` on `slices/meta` — the spine sketch, principles, fold tables, side-quest list, verifications applied to slice 2 (clipping separable from diff planner, minimal asset store feasible).
- The kintsugi manifest's two pre-kintsugi slices (`kitty-image-plumbing`, `native-plugins`) registered for tracking.
- The understanding of what slice 2 contains in concrete code terms (~1500 lines of kitty.rs core, ~100 lines of minimal asset store, etc.).

These are usable inputs to next session.

## Things to not retry

- "Cherry-pick e76ac675 onto a different base" — the diff doesn't apply cleanly off-chain; it depends on earlier kitty commits' state.
- "Manual perl migration of advance() call sites" — works mechanically but loses the connection to the original work; we don't know if we matched what slice 1 should actually contain.
- "Use kintsugi compose for slice 1" — compose is for combining already-authored tips, not for authoring a single tip.
