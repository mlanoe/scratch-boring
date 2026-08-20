# scratch-boring

A Scratch **player** (not editor) runtime authored in the [Boring](https://github.com/mlanoe/boring) language,
rendered with [Bevy](https://bevyengine.org/) via [`boring-bevylib`](https://github.com/mlanoe/boring-bevylib).
Loads and runs `.sb3` project files exported from the official Scratch editor or
TurboWarp — it does not implement Scratch's own drag-and-drop block editor.

100% Boring-authored: every component, system, opcode, and asset-loading path —
the ECS wiring, the `.sb3` loader, the exact-arithmetic number tower, costume/sound
extraction — comes from `.br` source under [`boring/`](boring/). `src/boring_gen.rs`
is generated (`boring build --emit-rust`, see [Building](#building)) and is the only
file under `src/`; no hand-written Rust module remains.

## Current state

Every ordinary (non-legacy, non-extension) block opcode in real Scratch's visible
palette is implemented, verified against real projects fetched from
scratch.mit.edu (an opcode-histogram check plus a full scratch-vm source audit) as
well as against `scratchfoundation/scratch-vm`'s own archived integration-test
fixtures. What's left is a short, genuinely-blocked or deliberately-out-of-scope
list — see [Roadmap / known gaps](#roadmap--known-gaps).

### Scheduler & clone (Type/Instance) model

Each Scratch "thread" is plain data, not an OS thread or a Boring `task`/`stream`:
an explicit stack of `(sequence: [Block], index)` frames (`ExecFrame`, carrying a
`Frame` scope and a `FrameKind`), advanced one step per tick by a single Bevy
`Update` system (`run_pending_threads`/`step_thread`, in `runtime.br`/
`interpreter.br`). Yield points match real Scratch exactly (confirmed against
scratch-vm's own block source, not guessed): the end of every `repeat`/`forever`/
`repeat until` iteration (not `if`/`if-else`, which run their whole body in one
step), and a `wait`/`wait until` whose condition isn't satisfied yet. `warp` ("Run
without screen refresh," used by custom blocks) is honored — a warp-flagged call's
own loops skip yielding entirely (bounded by a 1,000,000-iteration safety cap so a
genuinely infinite `forever` inside a warp call still eventually yields rather than
hanging). `broadcast` is fire-and-forget (receivers start on the next tick);
`broadcast and wait` genuinely drives its receiver(s) to completion synchronously
before continuing (a receiver that doesn't finish within a bounded iteration cap is
abandoned rather than truly awaited — a documented gap). Every `when green flag
clicked` hat, project-wide across every target, starts its own thread at Startup —
not just the first one found.

Clones (`control_create_clone_of`/`control_start_as_clone`/
`control_delete_this_clone`) are a real Type/Instance model: every per-target
resource (position, costume/size/effects, layer order, sound volume/pitch/pan,
rotation style, draggable flag, sprite-local variables/lists) is keyed by live
*instance* id, copied from the source instance's own *current* state on
`create clone of` and removed on delete. A hat fires on every live clone of its
owning sprite, not just clones' own `start as clone` script — broadcast,
key-press, click, `event_whengreaterthan`, backdrop-switch, and
`event_whentouchingobject` hats all fan out across instances. Sprite-local
variables/lists are distinguished from Stage globals and are per-clone-instance,
not shared.

The real game runs as a normal windowed Bevy `App` (`DefaultPlugins`); costume
bitmaps/rasterized SVGs and extracted sounds are cached under
`assets/.costume_cache/`/`assets/.sound_cache/` (gitignored, rebuilt on demand) —
inside the project's own `assets/` root because Bevy's `AssetServer` rejects paths
outside it by default.

### Motion

Every Motion opcode in the modern editor's palette: `move`/`turn left`/
`turn right`/`point in direction`/`point towards` (sprite, mouse, or a random
direction)/`go to xy`/`go to` (sprite, mouse, or a random position)/
`glide secs to xy`/`glide to`/`change x|y by`/`set x|y`/`if on edge, bounce`/
`set rotation style`, plus the `x position`/`y position`/`direction` reporters.
Glides are real per-tick tweens (`FrameKind::Gliding`), not an instant jump.
Rotation style (`all around`/`left-right`/`don't rotate`) is applied to actual
rendering — full `Transform` rotation for "all around," `Sprite.flip_x` only for
"left-right." Collision/edge geometry is an axis-aligned bounding box (AABB) —
sprite rotation is not applied to it, a consistent, deliberate approximation (not
real Scratch's pixel-perfect test), scaled by `bitmapResolution` and per-sprite
`size%`.

### Looks

`say`/`think` and their `for secs` variants (a real `ChildOf` child entity per
sprite, offset above it, not sharing the sprite's own `Transform`);
`show`/`hide`; `set size to`/`change size by`; costume switching (`switch costume
to` by name, number, `"next costume"`/`"previous costume"`, or a non-numeric name
falling back through the same `Cast.toNumber`-coerces-to-0 rule real Scratch
uses) and its `costume #`/`costume name` reporters; backdrop switching (adds
`"random backdrop"` to costumes' two special strings, plus a synchronous
`...and wait` variant that drives every matching `when backdrop switches to` hat
to completion first) and its `backdrop #`/`backdrop name` reporters;
`go to front/back` and `go forward/backward N layers` (a real splice-based
reorder of every target's current front-to-back rank, not a raw order-value
bump); the `ghost` graphic effect (mapped onto real sprite alpha). The other six
graphic effects (`color`/`fisheye`/`whirl`/`pixelate`/`mosaic`/`brightness`) are
parsed but not rendered — real Scratch has no reporter for any of them either, so
there's nothing a project could observe as wrong, but the visual effect itself is
a known, undone gap. On-stage variable/list/reporter watchers are supported,
including `show variable`/`hide variable`/`show list`/`hide list` toggling a
monitor's visibility live, not just its initial saved state.

### Sound

`play` (fire-and-forget) and `play until done` (a fixed duration computed once
from the sound's own saved sb3 `sampleCount`/`rate`, not a live playback-completion
callback — this project's scheduler has no such callback); `stop all sounds`;
volume (0-100, clamped); pitch (a verified `2^(pitch/120)` playback-rate
multiplier, real Scratch's own formula, mapped onto `PlaybackSettings.speed`);
pan (approximated via `bevy_audio`'s spatial-audio primitive, calibrated to a
moderate ~2:1 stereo bias rather than a hard left/right cut — `bevy_audio` has no
direct pan value). Both WAV and MP3 source audio decode correctly (`bevy`'s
default `audio` feature only ships a Vorbis decoder; `symphonia-wav`/`mp3` are
explicitly enabled in `Cargo.toml` after a real crash — first sound played in any
project, of either format, otherwise panicked). A sound already playing is not
live-retargeted if its target's volume/pitch/pan changes after the fact — a
documented gap. No effects beyond pitch/pan are modeled, matching real Scratch
(which has no others).

### Sensing & input

Keyboard: `key pressed?` and `when key pressed` cover every individually-nameable
KEY_OPTION value real Scratch's own dropdown has (space, the four arrow keys,
every letter, every digit — confirmed against `scratch-blocks` source to be the
complete set; there is no punctuation option) plus a truly generic `"any"` that
reads Bevy's own pressed-keys iterator directly, not an enumerated OR-chain.
Mouse: `mouse x`/`mouse y`/`mouse down?`, per-sprite and Stage click hats, and a
real click-and-drag interaction (`set drag mode`, top-most-instance picking by
z-order, a script toggling drag mode mid-drag stops it cleanly). Mouse position
assumes the default, unscaled `Camera2d` — there is no zoom/pan/viewport-scaling
system yet for it to account for.

`touching object?`/`touching mouse-pointer?`/`touching edge?` (AABB, same
approximation as Motion's collision test); `touching color?` (a full per-tick
software-composited 480×360 buffer of the entire visible stage — backdrop, pen
marks, every visible sprite in real z-order — scanned with real scratch-render's
own pixel tolerance; no GPU readback involved); `color is touching color?` is
still a documented gap (needs sampling the querying sprite's own costume color per
pixel, not just the composite). `distance to` (sprite, mouse, or the Stage's fixed
`10000`); `ask and wait`/`answer` (a real FIFO queue — more than one script can be
mid-ask at once — an on-screen text box, and real character-by-character keyboard
capture); `[property] of [object]` (built-in properties plus a fallback to any
global or sprite-local variable by name); `current [date/time]` (real OS wall
clock via `chrono`) and `days since 2000`; `reset timer`; `loudness`/`is loud?`
(report real Scratch's own defined no-microphone sentinel values, `-1`/`false` —
there is no actual microphone-capture subsystem).

### Control, Operators, Data & Procedures

Control: `if`/`if-else`/`repeat`/`repeat until`/`forever`/`wait`/`wait until`/
`stop`, plus clone creation/deletion/start-as-clone (see the scheduler section
above) and `warp` support. Operators: all arithmetic, comparison, and boolean
operators, `join`/`letter of`/`length of`/`contains`, `round`, every `math op`
dropdown value, and `pick random` (a deterministic xorshift64 PRNG, fixed-seeded
for reproducibility, not real per-run entropy). Data: get/set/change-by on
variables, every list mutator (add/delete/delete all/insert/replace/item
of/item # of/length of/contains), and `show`/`hide` for both variables and lists
— all correctly distinguishing Stage-global from sprite-local storage, and
copying a sprite-local value's *current* (not origin-default) state on clone
creation. Procedures ("My Blocks"): parameters plus nested/recursive calls via
per-call frames, supported since the project's earliest headless phase.

### Events & broadcasts

`when green flag clicked` (every hat project-wide, not just the first found),
`when key pressed`, `when this sprite clicked`, `when stage clicked`,
`when backdrop switches to` (one tick of lag versus real Scratch's synchronous
fire — an edge-detection trade-off, not a bug), `when I receive` (`broadcast`/
`broadcast and wait`, see scheduler above — no broadcast-deduplication-per-caller,
a documented gap), `when [timer/loudness] > value`
(edge-detected each tick, same one-tick-lag trade-off), and
`when touching [object]` (edge-triggered per live instance, reusing the AABB
touching machinery). Every hat fans out to live clones of its owning sprite.

### Pen (extension)

Fully implemented: `erase all`, `pen down`/`pen up`, `set pen color to
[color]`, `change/set pen color param`, `change/set pen size`, `stamp`
(rotation/scale/ghost-aware costume compositing onto the pen canvas, sampled
via nearest-neighbor over the sprite's own rotated bounding box), and the four
legacy Scratch 2.0 hue/shade blocks (kept functional, though hidden from the
modern palette, matching real scratch-vm). The pen canvas is a persistent
480×360 RGBA image drawn in real Boring/CPU code (a Porter-Duff "over"
compositor, ported from real `scratch-render` semantics) — not a shader. Pen
trails are queued at the exact moment a pen-down target's position changes
(inside the interpreter itself, not a once-per-tick diff), so a script that
moves the same target many times in one tick (a common "replay the whole point
history" pattern) draws correctly instead of collapsing into one long,
misplaced line. Stamps composite about the costume's geometric center only —
a saved `rotationCenterX/Y` offset is not applied, matching this project's
existing rotation-rendering simplification.

### Costumes & rendering

Bitmap costumes pass through unchanged; SVG costumes rasterize to PNG via
`resvg`/`usvg`/`tiny-skia` (system fonts are loaded into `usvg`'s font database
so `<text>` elements inside an SVG costume actually render, not silently
disappear). `bitmapResolution` is applied so a retina-authored bitmap costume
renders and collides at its intended logical size, not double. Per-sprite
`size%` scaling, the `ghost` effect, rotation-style-aware facing, and z-order
layering are all live on the real rendered sprite. Every costume for a target is
pre-loaded as a real `Handle<Image>` at Startup (Bevy has no "swap this
sprite's texture by name" primitive), so costume switching is a same-tick
handle swap, not an asset load.

### A note on scratch-vm's own test suite

`scratchfoundation/scratch-vm` (Scratch 3.0's own reference VM, archived but
still public) has a real, useful `test/fixtures/*.sb3` corpus. Its own
`.sb2` fixtures aren't usable here (a structurally different, nested JSON
schema this project has no loader for). Its `.sb3` fixtures load fine, but its
own `*.js` assertions mostly check scratch-vm's internal bookkeeping, not
anything this project's architecture shares — the useful part is the real
project files themselves, with expected values hand-derived from each
fixture's own saved data. They're deliberately **not vendored into this
repo** (scratch-vm is AGPLv3; this project is GPL-3.0-or-later — compatible
licenses, but redistributing another project's files verbatim into a
differently-licensed repo is a separate decision from an ordinary `cargo
test` run). `scripts/fetch_scratch_vm_fixtures.sh` downloads the fixtures
this project currently uses on demand into a gitignored directory; the tests
that depend on them (`tests/scratch_vm_fixtures.rs` and others) are
`#[ignore]`d and skip gracefully with a pointer to the fetch script if the
fixture isn't present.

## Scope decisions

- **Player only.** No visual block editor. Real Scratch's drag-and-drop UI is a
  large, separate undertaking (palette, canvas, undo/redo) not needed to *run*
  Scratch projects.
- **Exact arithmetic**, not `f64`. Real Scratch (JS) uses plain doubles, but this
  project uses its own exact int/BigInteger/fraction tower instead —
  `ScratchNumber` in `boring/values.br`, built on `BigUint`/`BigInt`/`BigFraction`
  from the sibling `boring-numlib` project — deliberately more precise than spec,
  at the cost of extra implementation work.
- **`.sb3` only, not `.sb2`.** Scratch 2.0's project format nests sprites inside a
  stage object, structurally different from `.sb3`'s flat targets array; no
  loader exists for it.
- **Extensions were originally deferred**, but Pen has since landed in full (see
  above). Text-to-speech, video sensing, and music remain out of scope —
  text-to-speech specifically because it needs a live call to a remote,
  Scratch-Team-operated API (a real network/ToS/testability concern, not just
  difficulty); video sensing needs a camera dependency this headless-first
  project was never going to take on; music needs a synthesized/sampled
  instrument decision not yet made.
- **Legacy (Scratch 2.0-compatible, `hideFromPalette`) blocks are skipped** unless
  a real gap surfaced them anyway (Pen's four legacy hue/shade blocks did, and are
  implemented) — no modern editor can generate
  `control_all_at_once`/`control_for_each`/`control_while`/the counter blocks,
  `looks_hideallsprites`/`changestretchby`/`setstretchto`, or `sensing_userid`,
  confirmed against scratch-vm's own source comments.

## Architecture

### Project layout

```
boring.toml            # [project] main = "boring/scratch.br"; [deps] numlib = "../boring-numlib";
                        # pulls in boring-bevylib's [external_types]/[derives]
boring/scratch.br       # Boring-authored: Bevy ECS wiring (components, systems,
                        # build_app/run_game/main) -- the app's own composition root
boring/sb3_loader.br    # Boring-authored: .sb3/project.json -> raw structs (Sb3Project etc.)
boring/values.br        # Boring-authored: ScratchNumber/ScratchValue, the value model
boring/runtime.br       # Boring-authored: Block/BlockKind, StopSignal, every runtime
                        # resource's shape (Vars, Lists, Positions, watchers, ...)
boring/linker.br        # Boring-authored: Sb3Project -> real runtime resources + Block AST
boring/interpreter.br   # Boring-authored: eval/exec/step_thread, the tree-walking evaluator
boring/scheduler.br     # Boring-authored: headless test entry points (run_greenflag_and_report* etc.)
boring/assets.br        # Boring-authored: costume/sound extraction from a .sb3 zip
boring/pen.br           # Boring-authored: the Pen extension (canvas, line/stamp rasterization,
                        # scene compositing for sensing_touchingcolor)
                        # (all seven files above are siblings, pulled into scratch.br
                        # via a bare `use <name>` each; scratch.br itself used to hold
                        # everything directly and was split once it grew too large)
boring/regen.sh         # boring build --emit-rust -> src/boring_gen.rs, same convention
                        # as ../breakout-boring/boring/regen.sh
Cargo.toml              # [lib]+[[bin]] both -> src/boring_gen.rs, like breakout-boring
src/boring_gen.rs        # generated, do not hand-edit -- the only file under src/,
                        # no hand-written Rust module remains
tests/                  # headless integration tests (cargo test), fixtures under assets/
assets/                 # test .sb3/.json fixtures, hand-authored plus a few real demo projects
assets/.costume_cache/  # gitignored -- costume images extracted/rasterized on demand
assets/.sound_cache/    # gitignored -- sound files extracted on demand
```

### What's Boring-authored vs hand-written glue

**100% Boring — no hand-written Rust module remains.** Three modules were
originally hand-written Rust and were each ported to Boring source once the
relevant language gap closed: the exact-arithmetic number tower (`ScratchNumber`,
now built on the sibling `boring-numlib` project's `BigUint`/`BigInt`/
`BigFraction` via `boring.toml`'s `[deps]`); costume loading (zip extraction + SVG
rasterization + the `AssetServer`-compatible cache-file bridge); sound loading
(extraction + the same cache-file bridge) — all three once `boring.toml`'s
`[external_fns]` section made it possible to express the `&`/`&mut` argument
borrows `zip`'s and `resvg`'s real signatures need directly from Boring source.
`.sb3`/`project.json` loading (`boring/sb3_loader.br`) was the last one, staying
hand-written the longest until two genuine language gaps closed: field-renaming
on `@derive(Deserialize)`, and a dynamic/untyped JSON value type for the format's
genuinely dynamic shapes (a plain Boring-declared `@serde(untagged)` enum gets
real `serde`'s own untagged-enum inference for free, once verified against real
`serde_json` rather than `boring run`'s own, different interpreter behavior for
untagged enums). See `boring/sb3_loader.br`'s own header comment for the fuller
writeup of the transpiler quirks the port surfaced.

### Block AST & instance model

Each parsed script is a Boring `enum Block` with one variant per supported
opcode, `Vec<Block>` for sequences — a tree-walking interpreter with a real
optimization pass (constant folding, dead-branch elimination, loop/hat
specialization) rather than a re-interpretation of the raw sb3 block graph on
every tick. Instance state (position/direction/visibility/costume and every
user variable) lives in a flat, shape-shared slot array addressed the same way
for built-ins and user variables alike, generalized into the Type/Instance
model described above for clones.

Procedures ("My Blocks") are supported from this project's earliest phase,
including recursion — call frames fall out naturally from the scheduler's own
explicit frame stack (see below) rather than needing separate call-stack
bookkeeping.

### Scheduler: cooperative, no OS threads, no Boring `task`/`stream`

Running each Scratch script as a real OS thread (blocking on `wait`, locking
per instance) doesn't fit Boring's Bevy-hosted architecture: Boring's `task` is
real tokio async, and `stream`/`yield` runs eagerly to completion unless it
contains `task`/`wait` (a naive `forever` written as a `stream` would hang
immediately); using `task`/`wait`/`channel` anywhere also flips `main` to
`#[tokio::main]`, which collides with Bevy's own blocking `App.run()` event
loop.

Instead, a Scratch "thread" is plain data — an explicit stack of
`(sequence: [Block], index)` frames (`ExecFrame`), advanced by a single Bevy
`Update` system (`run_pending_threads`), one call per tick. There is no
work-budget step *within* one thread's own loop iterations — real scratch-vm's
own `WORK_TIME` budget governs how many *different threads* get a turn per
tick, never how many iterations one thread's own loop gets before yielding
(that always yields every iteration, unconditionally) — this project doesn't
need that budget yet since it never runs enough concurrent threads for tick
latency to matter. Because nothing ever blocks an OS thread, there's no
pool-vs-dedicated-thread split or per-instance locking to manage. (Running
different sprites' scripts on separate real threads is a possible future perf
optimization that would reintroduce exactly that problem — explicitly out of
scope for now.)

## Roadmap / known gaps

Everything below is either genuinely blocked on infrastructure this project
doesn't have, or a deliberate scope decision — not a sequence of planned work.

- **`sensing_coloristouchingcolor`** — needs sampling the querying sprite's own
  costume color at each candidate pixel, not just the composited scene buffer
  `sensing_touchingcolor` already builds.
- **Real microphone-based loudness** — `loudness`/`is loud?` correctly report
  real Scratch's own defined "no audio engine" values (`-1`/`false`); actual
  microphone capture would need a new OS-level audio-input dependency (e.g.
  `cpal`) this project has never taken on.
- **Six of seven graphic effects don't render** — only `ghost` is mapped onto a
  real Bevy primitive (sprite alpha). `color`/`fisheye`/`whirl`/`pixelate`/
  `mosaic`/`brightness` are parsed but have no visual effect; real Scratch also
  has no reporter for any of them, so a project can't observe the difference,
  but the rendering itself is undone.
- **AABB-only collision**, not pixel-perfect, and not rotation-aware — a
  consistent approximation across Motion's edge-bounce, Sensing's touching
  tests, and click hit-testing, not real Scratch's per-pixel alpha test.
  Costume/stamp rotation is always about the costume's geometric center, never
  a saved `rotationCenterX/Y` offset.
- **Sound parameters aren't live** — changing volume/pitch/pan after a sound
  has started playing doesn't retarget the already-playing instance.
  Broadcast has no per-caller deduplication: a second non-waiting `broadcast`
  from the same block doesn't wait on an earlier one, it piles up
  independently.
- **A few edge-triggered hats lag by one tick** versus real Scratch's
  synchronous fire: `when [timer/loudness] > value`, `when backdrop switches
  to`, and `reset timer`'s own effect — all detected by comparing state
  tick-to-tick rather than a synchronous callback at the point of change.
- **`.sb2` projects aren't loadable** — see Scope decisions above.
- **Legacy (`hideFromPalette`) blocks and extensions** — text-to-speech, video
  sensing, and music remain out of scope (see Scope decisions above); a
  handful of Scratch 2.0-only blocks are unimplemented because no modern
  editor can generate them.

## Building

`boring.toml`'s `[deps]`/`[external_types]`/`[derives]` sections resolve
against sibling directories, so building from source needs three other repos
cloned next to (not inside) this one — i.e. `scratch-boring/`,
`boring/`, `boring-bevylib/`, and `boring-numlib/` all under the same parent
directory:

```bash
# from the parent directory that scratch-boring itself is cloned into:
git clone https://github.com/mlanoe/scratch-boring
git clone https://github.com/mlanoe/boring
git clone https://github.com/mlanoe/boring-bevylib
git clone https://github.com/mlanoe/boring-numlib   # no boring.toml/Cargo.toml of its own, just a src/ tree
(cd boring && cargo build --release)

cd scratch-boring
export BORING_BIN=../boring/target/release/boring   # or just `boring` if it's on PATH
(cd boring && ./regen.sh)   # this project's own boring/regen.sh, regenerates src/boring_gen.rs
cargo build
cargo run
```

`cargo test` runs the full default suite (fixtures checked into `assets/`).
A handful of extra tests cross-check against real
`scratchfoundation/scratch-vm` test fixtures and are `#[ignore]`d by default
(see "A note on scratch-vm's own test suite" above for why they're not
vendored):

```bash
./scripts/fetch_scratch_vm_fixtures.sh
cargo test -- --ignored
```

## License

Copyright (C) 2026 Mickaël LANOË

This program is free software: you can redistribute it and/or modify it under
the terms of the [GNU General Public License v3.0](LICENSE) as published by
the Free Software Foundation, matching this repo's own `Cargo.toml`
(`license = "GPL-3.0-or-later"`).

This program is distributed in the hope that it will be useful, but
**without any warranty**. See the [LICENSE](LICENSE) file for the full terms.
