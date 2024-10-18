# zint-wasi

This is a Zint binding for WASI.

- `zint-wasm-sys` is a low-level binding to the Zint library.
- `zint-wasm-rs` is a high-level binding to the Zint library.
- `zint-typst-plugin` is a typst package for the Zint library.

This package only uses the Zint library but not any of its frontends. So it is MIT licensed.

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

To build the typst package, run:
```sh
cargo xtask package
```

## License

This package is licensed under MIT license.
A copy of the license can be found in the [LICENSE](./LICENSE) file.
