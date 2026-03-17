# Development

## Render metrics (debug-only)

Enable render churn metrics in debug builds:

```sh
RUST_LOG=info TERMY_RENDER_METRICS=1 cargo run -p termy
```

Note:
- Metrics logs are `debug_assertions`-only, so `--release` will not emit `render_metrics` lines.
- Counter meaning:
  - `full`: full per-pane cell cache rebuild decisions
  - `partial`: dirty-span patch decisions
  - `reuse`: no cell cache update decisions
  - `dirty_span`: number of dirty spans consumed during partial updates
  - `patched_cell`: number of cells patched from dirty spans
  - `grid_paint` / `shape_line`: paint + text shaping work done that interval
- Cursor-blink sanity check: `full` should stay near `0`; `reuse` or small `partial` values are expected depending on reported terminal damage.

## Render benchmarks

Prerequisites: macOS with `xctrace` / Activity Monitor installed. This harness
is not supported on Linux or Windows.

Run the automated real-window render benchmark compare on macOS:

```sh
cargo run -p xtask -- benchmark-compare \
  --baseline-root /path/to/baseline/worktree \
  --candidate-root /path/to/candidate/worktree \
  --output /tmp/termy-benchmark-compare
```

This builds release binaries in both worktrees, launches real Termy windows with
deterministic benchmark scenarios, records an Activity Monitor trace with
`xctrace`, and writes a comparison report.

Artifacts are written under the chosen output directory:

- `report.md`: human-readable comparison summary
- `summary.json`: machine-readable top-level summary
- `raw/<build>/<scenario>/app/summary.json`: in-app aggregate metrics for a run
- `raw/<build>/<scenario>/app/timeline.ndjson`: sampled frame/CPU timeline
- `raw/<build>/<scenario>/app/frames.ndjson`: per-frame presentation events
- `raw/<build>/<scenario>/driver/markers.ndjson`: benchmark-driver event markers
- `energy/<build>/<scenario>/activity-monitor.trace`: raw `xctrace` recording
- `energy/<build>/<scenario>/energy.json`: parsed Activity Monitor summary

Current scenarios:

- `idle-burst`
- `echo-train`
- `steady-scroll`
- `alt-screen-anim`

## Tmux integration tests

Run the local end-to-end tmux split integration harness:

```sh
just test-tmux-integration
```

Requirements:
- tmux `>= 3.3`

Optional:
- Override tmux binary path with `TERMY_TEST_TMUX_BIN=/path/to/tmux`
