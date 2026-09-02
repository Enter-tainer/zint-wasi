# zint-wasi

This is a Zint binding for WASI.

- `zint-wasm-sys` is a low-level binding to the Zint library.
- `zint-wasm-rs` is a high-level binding to the Zint library.
- `zint-typst-plugin` is a typst package for the Zint library.

This package compiles the Zint library, but none of its frontends, which are licensed under the
GPL. Our own code is MIT licensed and the bundled backend (libzint) is BSD-3-Clause, so everything
built here is distributed under `MIT AND BSD-3-Clause`.

Checkout examples and `typst-package/manual.typ` for more information.

## Manual

_(click on the image to open)_

<a aria-label="Link to manual" href="https://raw.githubusercontent.com/Enter-tainer/zint-wasi/master/typst-package/manual.pdf" target="_blank">
  <img src="/assets/manual-preview.svg">
</a>

## Build

Clone with:
```sh
git clone --recurse-submodules -j8 https://github.com/Enter-tainer/zint-wasi.git
```

You must have standard development tools pre-installed on your machine and in path:
- cargo (rustc; get with [rustup](https://rustup.rs/))
- tar
- wget/curl
- gcc/clang

On Windows you additionally need:
- the MSVC toolchain, i.e. Visual Studio Build Tools with the "Desktop
  development with C++" workload (`link.exe` and the Windows SDK), because
  `xtask` and the build scripts are compiled for the host
- LLVM, for the `libclang.dll` that `bindgen` loads; set `LIBCLANG_PATH` to its
  `bin` directory if it isn't in `PATH`
- `WASI_SDK_PATH` pointing at a WASI SDK, unless you let the build download
  one for you

To build the typst package, run:
```sh
cargo xtask package
```

See [`xtask` readme](./xtask/README.md) for more information.

## Tests

```sh
cargo test -p zint-wasm-sys -p zint-wasm-rs -p zint-typst-plugin -p xtask
```

The packages are listed rather than testing the whole workspace because it also
vendors third-party crates. Building `zint-wasm-rs` next to the plugin turns on
its `typst` feature, which renames every option key, so CI additionally runs
`cargo test -p zint-wasm-rs` on its own to cover the plain library.

### Golden files

`zint-wasm-rs/tests/golden` holds the SVG that every symbology and every option
renders to, one file per case. They are not a claim that the current output is
right; they are there so that a change to it is visible, which is what makes an
upgrade of the vendored Zint reviewable.

Every case is a test of its own, so a run says which barcodes moved rather than
that some did, and one can be re-run on its own:

```sh
cargo test -p zint-wasm-rs --test golden -- symbology::QRCode
```

After an intended change, rewrite the files and read the diff before committing
it:

```sh
UPDATE_EXPECT=1 cargo test -p zint-wasm-rs --test golden
```

### Mutation testing

[cargo-mutants](https://mutants.rs) changes the code in small ways and reports
the changes that no test noticed, which says a good deal more about a test suite
than line coverage does:

```sh
cargo install cargo-mutants --locked
cargo mutants -p zint-wasm-rs
```

It is not part of CI: a full run takes far longer than a pull request should
wait for, and what it produces is a list to read rather than a pass or a fail.

## Release

A release is cut by the `CD` workflow under the repository's Actions tab, started by hand
from `master`. Given a version, it moves that version into every file that repeats it, builds
the package, commits and tags the result as `v<version>`, publishes a GitHub release with the
package and the manual attached, and opens the pull request on
[typst/packages](https://github.com/typst/packages) that puts the version on Typst Universe.
Leaving the version empty releases the version the manifests already carry.

The workflow refuses to run if the tag exists or the version is already on Universe, so a
failed run can be started again. A dry run builds and uploads the assets without committing,
tagging, releasing or publishing, and may be started from any branch.

It needs two things set up once:

- A `TYPST_PACKAGES_TOKEN` repository secret holding a personal access token with the
  `public_repo` scope of the person who submits to Universe. The token creates or reuses that
  person's fork of `typst/packages`, pushes the version there and opens the pull request in
  their name, which is what Universe expects: an update should come from whoever submitted the
  previous version.
- If `master` is protected, the branch rules have to let the workflow push the release commit
  and the tag.

The same steps can be run by hand:

```sh
cargo xtask set-version 0.4.0   # moves the version, refreshes Cargo.lock
cargo xtask version             # prints it, after checking every copy agrees
cargo xtask bundle              # builds the package and lays it out under target/bundle/tiaoma/0.4.0
```

The bundle leaves out the files `typst.toml` excludes, as Universe does. The Universe pull
request, on the other hand, gets the whole `typst-package` directory, excluded files included,
which Universe links from the package page.

## License

The code in this repository is licensed under the MIT license; a copy can be found in the
[LICENSE](./LICENSE) file.

The plugin statically links the Zint backend (libzint), which is licensed under BSD-3-Clause. Its
notice is in [zint-wasm-sys/LICENSE-BSD-3-CLAUSE](./zint-wasm-sys/LICENSE-BSD-3-CLAUSE) and ships
with the typst package. Anything built from this repository is therefore distributed under
`MIT AND BSD-3-Clause`.
