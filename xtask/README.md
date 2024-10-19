# tiaoma xtask

[xtask](https://github.com/matklad/cargo-xtask) is an extension to `cargo build` that's made specifically to make building
this plugin and testing it simpler.

xtask commands are executed through cargo:

```lua
cargo xtask <COMMAND> [ARGS...]
```

## Commands

- `package-plugin`: builds the tiaoma wasm plugin
  - `--debug`: creates a debug build of the plugin
- `build-manual`: compiles the manual
- `package`: packages all requirements needed for publishing

Command arguments are transitive, which means that running
`cargo xtask package --debug` will pass that flag to all task that `package`
depends on (i.e. `package-plugin`).

There are some other internal commands (like `ci`), but they're not meant to be
run as part of the normal development process, but instead under very specific
conditions. Don't rely on them being present, consistent or free of
side-effects.

## State options

xtask can be configured with various state variables. These are stored in
[`xtask/state`](./state) and can be overriden with environment variables
prefixed with `XTASK_` (`XTASK_<OPTION>`).

- Versions (to download):
  - `WASI_SDK_VERSION`: WASI SDK version used to compile the package
  - `BINARYEN_VERSION`: binaryen version used for `wasm-opt`
  - `TYPST_VERSION`: typst version used to compile the manual
- Paths:
  - `WORK_DIR`: path to the output directory
  - `TYPST_PKG`: path to the output directory
  - `WASM_MIN_PROTOCOL_DIR`: path to `wasm-minimal-protocol` project used to compile `wasi-stub`

A bunch of other, unlisted, variables are also defined/used, but they're
constants related to project structure for the most part and changing them will
break things.

The `xtask/state` is expected to be part of the source tree in VCS, this allows
xtask to keep track of changes to state that become visible only during builds
(e.g. file hashes).
