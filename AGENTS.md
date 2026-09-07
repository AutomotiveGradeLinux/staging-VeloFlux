# Repository Guidelines

## Documentation and Design Background
- The functional design background and solutions for this project are documented in Markdown files under `docs/`. Consult the relevant documents first when learning about a feature's background, constraints, or design.
- When adding documentation under `docs/`, classify syntax-related content separately to make it easier to find and maintain.

## Build, Test, and Development Commands
- `make build` — Compile the entire workspace in debug mode.
- `make release` — Build optimized binaries under `target/release/`.
- `make test` — Run workspace tests and each crate's local test suite (parser, flow, and datatypes).
- `make fmt`, `make clippy` — Run formatting and lint checks (`-D warnings`).

## Code Style and Naming Conventions
- Run `make fmt` and follow the default Rustfmt style (four-space indentation and trailing commas).
- Use English for code, comments, and identifiers. Use `snake_case` for items, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants.
- Respect module boundaries: parser must not depend on flow, and flow must consume only `SelectStmt` output rather than raw `sqlparser::Expr` values.

## Pre-commit Checklist

Complete the following checks after every code change and before committing. These checks address recurring sources of rework found in historical PR reviews, such as missed call sites, documentation drift, and superficial fixes. Their purpose is to eliminate basic issues before a reviewer sees the code.

### Change Impact Analysis
- When deleting or renaming a public API (function, type, trait, enum variant, struct field, or configuration field), use `rg` (ripgrep) to search for the name globally and ensure no call sites, configuration references, or documentation references remain.
- For deletion changes, such as removing a module or feature, also inspect:
  - All integration, end-to-end, regression, and fuzzing tests under `tests/`
  - All scripts under `scripts/`, including variable references and usage instructions in shell scripts
  - `config.yaml`, `etc/config.example.yaml`, and every configuration file under `distros/`
  - All Markdown documents under `docs/`
- Do not search only for exact matches. Consider variants such as snake_case, kebab-case, CamelCase, path separator differences, and YAML indentation differences.

### Build and Test Validation
- After changing a public API, run compile checks for the affected test targets with `cargo check --test <target>` (for example, `cargo check --test regression_tests` and `cargo check --test fuzzing_tests`) and resolve all compilation errors before committing.
- If sed or another script is used for bulk modifications, run a compile check afterward. A sed command silently having no effect has caused real problems before.
- After modifying a shell script, run `bash -n <script>` to check its syntax.
- After modifying a YAML configuration example, verify that its field paths match references in related scripts and documentation. For example, cgroup paths must be consistent across the setup script, `config.yaml`, and README.

### Documentation Synchronization
- When changing a feature's runtime behavior, such as API return values, error-handling policy, or lifecycle semantics, inspect and update the corresponding descriptions under `docs/`. The implementation is the source of truth, and the documentation must match it rather than describing behavior X while the code implements behavior Y.
- When adding a feature, documentation must describe behavior implemented by the current PR. Do not document unimplemented plans, such as future tar.gz export support, unless they are explicitly marked as deferred or future work.
- When removing a feature, delete or rewrite every description of it in the documentation. Do not leave orphaned sections or incomplete sentences.

### Fix Strategy
- Address review findings at their root cause rather than patching only the specifically identified path.
  - Negative example: A reviewer identifies that no memory limit is configured, and the fix adds a growth guard only to `host_alloc` without considering whether initial memory or guest `memory.grow` can bypass it.
  - Positive example: Configure Wasmtime's `StoreLimits` or `ResourceLimiter` to cover all memory-allocation paths at once.
- For security or resource-limiting concerns such as sandboxes, capabilities, and bounds, do not use fragile techniques such as regular expressions or substring scanning. Prefer structured or semantic approaches, such as parsing an AST or using the engine's native limiting API.
- If a correct fix requires substantial work, explain the limitations of the current approach in the commit message and explicitly mark the remaining work as deferred or future work under `docs/`. Do not pretend the issue is fully resolved.

## Communication Guidelines
- Use Chinese when communicating with maintainers, including requirement clarification, design confirmation, review feedback, and change descriptions.

## Coding Guidelines
- Before writing code, present the proposed design for the code changes and obtain confirmation. Do not modify code without confirmation.
- Immediately after modifying Rust code, run `make fmt` and `make clippy` and ensure both pass before continuing.
- Do not manage `git status`; leave it to the user.
- Completed code must pass `make fmt` and `make clippy`. Prefer running these checks before submitting or merging; clippy runs with `-D warnings`.
- Write all code comments and documentation under `docs/` in English.
- Maintain a single method entry point for the same responsibility on a struct. Use `with_*` variants sparingly, and avoid proliferating nearly identical methods for minor parameter differences. Prefer an Options or Config struct, an enum parameter, or a separate builder struct for optional behavior and extension points.
- When asked to generate a PR title and description, the title must pass the CI `PR Title Lint` and follow Conventional Commits.
  - Format: `<type>(<scope>): <subject>`, where `(<scope>)` is optional. Use `!` for a breaking change, such as `<type>(<scope>)!: <subject>` or `<type>!: <subject>`.
  - Use lowercase for `type` and `scope`. Common types include `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, and `revert`.
  - Examples: `feat(flow): add ...`, `fix(parser): handle ...`, and `docs: update ...`.
  - Write PR titles and descriptions in English.
- When using `gh` to address a PR review comment, commit any resulting local changes proactively. Use `git commit -s` to preserve sign-off consistency, and explain the approach and trade-offs in the commit message body.
- When an unfamiliar CI tool such as Kani or cargo-deny reports an error, consult its official documentation first to understand how it works and its known limitations before designing a fix. Do not guess flags or parameters from the literal error message alone.

## Testing Guidelines
- A full `make test` run is not required after every code change. Prefer the smallest test set that covers the change, such as only the new or affected unit tests.
- Never run `cargo bench` locally when fixing benchmark-related code.
- Before adding a unit test, confirm the test case and expected result before implementing and running it.
- Changes involving the `flow` module, especially planner logic, require additional validation of the test cases under `src/flow/tests/planner/`.
- Tests under `src/flow/tests/planner/` use a table-driven style, with SQL as the primary input and JSON output from `EXPLAIN` as the primary assertion. They must also print the SQL and the pretty-formatted explain output for manual inspection and troubleshooting.
- `src/flow/tests/pipeline/` contains table-driven SQL input/output validation tests. When adding a similar input/output test, prefer adding a case there.

## Module-specific Development Guidelines
- The `parser` module parses SQL into an AST. Parsing results are stored in `SelectStmt`, which serves as the contract with downstream modules.
- When modifying the `parser` module, consult syntax-related documents under `docs/` first.
- For each logical or physical optimization rule in the `flow` module's planner and optimizer, first consult its design document under `docs/planner_optimize/logical/` or `docs/planner_optimize/physical/` to understand its constraints and applicability. Then validate the rule using the `EXPLAIN` JSON assertions under `src/flow/tests/planner/`. Confirm any new test case with the maintainer before adding it, and record conclusions and considerations in `AGENTS.md`.
- Window time precision: `ss` and `ms` in `tumblingwindow` and `slidingwindow` use `SystemTime` and `Duration` consistently as their time coordinates in both the standalone window processor and fused streaming aggregation paths. Boundary calculations are centralized in `window_metadata::floor_to_window_start` and `saturating_sub_to_epoch`, so the processor layer does not depend on a specific time unit. Adding a time unit requires only extending `TimeUnit::duration()`. A watermark reaching a window's end boundary triggers a flush, and `window_start()` and `window_end()` preserve microsecond precision.
- Fused sliding-window semantics: Without `OVER (WHEN ...)`, every row still triggers evaluation. Like the standalone processor, fused streaming aggregation must use logical boundaries of `[t - lookback, t + lookahead]` and include historical rows within the lookback interval. Do not reuse scheduling logic by changing the logical `window.start` to the trigger time.
- Fixed output layout audit: The planner's `OutputLayout` defines final sink columns. Direct columns and aliases inherit fixed message slots, while computed columns use ordered affiliate slots. The encoder, row-diff, template, and memory materialization code must not infer the layout again from source or name, and must not restore the `ByIndexProjection`, `late_projection`, or `passthrough_messages` compatibility paths. Incompatible fan-in layouts must fail during planning.
- The `processor` module contains operators that process streaming data. After processing data, each processor forwards it to the next operator.
- Channel constraint: Each processor has two subscription channel types, a `control channel` and a `data channel`. Data received from one channel type may be forwarded only to the same channel type; the channels must not be mixed or crossed.
- The decoder processor converts `[]byte` into a `Collection` or `Tuple`; the encoder converts a `Collection` or `Tuple` into `[]byte`.
- Performance constraint: Avoid copying values when processors handle a `Collection` or `Tuple`; prefer borrowing, references, or zero-copy structures.
- Hot-path constraint: In frequently executed processor paths, especially partition-key logic, avoid expressions such as `format!("{:?}", key_values)`. This approach usually allocates both a `Vec<Value>` and a `String` and incurs debug-formatting overhead, which can become a significant CPU hotspot. Preserve structured keys and use data structures that directly implement `Hash` and `Eq` instead.
