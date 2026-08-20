#!/usr/bin/env bash
# Downloads real .sb3 test fixtures from scratchfoundation/scratch-vm's own
# official (archived, but still public) test suite into
# assets/scratch_vm_fixtures/ -- NOT checked into this repo (see
# .gitignore) and NOT run by default.
#
# Why not vendored: scratch-vm is AGPLv3-licensed; scratch-boring is
# GPL-3.0-or-later. The two are compatible copyleft licenses in general,
# but redistributing scratch-vm's own files verbatim inside this repo is a
# real licensing question worth keeping separate from casual `cargo test`
# runs rather than resolving unilaterally -- so these fixtures are fetched
# on demand instead of committed, and tests that use them are `#[ignore]`d
# by default (run explicitly with `cargo test -- --ignored` after fetching).
#
# Usage:
#   ./scripts/fetch_scratch_vm_fixtures.sh
#   cargo test -- --ignored
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$SCRIPT_DIR/../assets/scratch_vm_fixtures"
mkdir -p "$OUT_DIR"

BASE="https://raw.githubusercontent.com/scratchfoundation/scratch-vm/develop/test/fixtures"

# Only .sb3 fixtures -- this project's loader doesn't support the older
# .sb2 format (a structurally different JSON schema: nested sprites inside
# a stage object, rather than sb3's flat targets array), so scratch-vm's
# many .sb2-only fixtures (motion.sb2, control.sb2, looks.sb2, sound.sb2,
# procedure.sb2, event.sb2, saythink-and-wait.sb2, hat-execution-order.sb2,
# ...) aren't usable here at all without a separate sb2 loader -- a real,
# not-yet-attempted follow-up, not an oversight.
FIXTURES=(
    "edge-triggered-hat.sb3"      # event_whengreaterthan + sensing_timer -- drove this project's Timer/GreaterThanHats feature
    "broadcast_special_chars.sb3" # broadcast message names with special characters -- load-robustness check
    "top-level-reporters.sb3"     # orphaned reporter blocks with no hat -- load-robustness check
    "variable_characters.sb3"     # variable names with special characters
    "list-monitor-rename.sb3"     # a monitor's own params can be stale vs. the variable/list's current name -- drove the watcher-label fix
    "missing_png.sb3"             # a costume referenced in project.json but absent from the zip -- extraction robustness check
    "corrupt_png.sb3"             # a costume whose asset bytes exist but are corrupted -- extraction robustness check
    "monitors.sb3"                # every monitor opcode scratch-vm itself tests, incl. sprite-local variables -- drove the watcher sprite-local-skip fix
    "timer-monitor.sb3"           # a lone sensing_timer monitor with no spriteName -- drove the reporter-watcher Timer case
    "missing_svg.sb3"             # project.json + assets nested one directory deep in the zip (not at the root) -- drove the prefix-tolerant zip-entry lookup fix; costume asset also genuinely absent
    "corrupt_svg.sb3"             # same nested-zip shape as missing_svg.sb3, but the costume asset is present with deliberately invalid SVG content
    "missing_sound.sb3"           # a flat zip (no fix needed) with a sound asset genuinely absent -- extraction robustness check
    "corrupt_sound.sb3"           # a flat zip with a sound asset present but truncated/corrupted -- passes through unvalidated, same as corrupt_png.sb3
    "cloud_variables_simple.sb3"  # a Stage cloud variable (name starts with the ☁ character, 3-element [name, value, true] form)
    "origin.sb3"                  # a third, independently-found nested-zip fixture -- confirms the fix generalizes
    "cloud_variables_limit.sb3"   # ten simultaneous visible Stage cloud-variable watchers -- variable-watcher breadth stress test
    "monitored_variables.sb3"     # two visible monitors, both sprite-local -- the zero-surviving-watchers edge case
)

for f in "${FIXTURES[@]}"; do
    echo "Fetching $f..."
    curl -sL -o "$OUT_DIR/$f" "$BASE/$f"
done

echo "Done. Fixtures written to $OUT_DIR (gitignored)."
echo "Run the fixture-dependent tests with: cargo test -- --ignored"
