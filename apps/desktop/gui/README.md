# TSideload GUI (Tauri shell — scaffold)

Graphical frontend for `tsideload-core`. **Status: scaffold (P0).**
`list_devices` is wired; the install flow (Apple ID + 2FA dialog) is P1 —
use `tsd install` until then.

## Why it is not in the workspace

The GUI needs WebView2 (Windows) or WebKit (macOS). It cannot compile on a
bare Linux box (no `webkit2gtk`), so `apps/desktop/Cargo.toml` excludes
`gui/src-tauri`. Build it on the target OS:

```sh
cd apps/desktop/gui/src-tauri
cargo build --release        # Windows or macOS only
```

Frontend is static HTML/JS in `gui/src` (`tauri.conf.json` points
`frontendDist` there, no Node build step). i18n files live in
`gui/src/i18n/{en,vi}.json` — key conventions: `docs/I18N.md`.
