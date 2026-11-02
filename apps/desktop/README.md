# TSideload Desktop

Desktop signer: sign an `.ipa` with your free Apple ID and install it on a
USB-connected iPhone/iPad. No Xcode, no Mac, no paid developer account.

## Crates

- `crates/tsideload-core` — library wrapping the [`isideload`] crate (MIT):
  device listing, Apple ID auth (+2FA), sign & install, `.p12` import (P1).
- `crates/tsd` — CLI binary.
- `gui/` — Tauri v2 shell (scaffold; builds on Windows/macOS only).

## Build & check (any OS, incl. Linux)

```sh
cd apps/desktop
cargo check        # core + CLI
cargo build --release -p tsd
```

The GUI is excluded from the workspace — see `gui/README.md`.

## Use

```sh
./target/release/tsd devices
./target/release/tsd install app.ipa --apple-id you@example.com
```

[`isideload`]: https://github.com/nab138/isideload
