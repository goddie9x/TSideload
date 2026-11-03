# TSideload GUI (Tauri shell)

Graphical frontend for `tsideload-core`. **Status: install flow wired (P1b).**
`list_devices` lists USB devices; the install section picks an IPA (via
`tauri-plugin-dialog`), logs in with an Apple ID (2FA dialog follows the
`tsd` CLI contract — no code entry when Apple leaves the method `unknown`),
then signs + installs with coarse stage events (`login`/`signing`/`done`/
`error`). Imported-certificate signing is still P1 (the method selector shows
it disabled, honestly).

The password field is cleared on submit and never logged or emitted.

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
