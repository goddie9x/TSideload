# TSideload Architecture

Three parts, one repo. The two app tracks are developed in parallel and must
not step on each other's files.

```
TSideload/
├── apps/
│   ├── desktop/          # TRACK: desktop signer (Rust) — this doc's focus
│   │   ├── crates/
│   │   │   ├── tsideload-core/   # lib: devices, Apple ID auth, sign+install, certs
│   │   │   └── tsd/              # CLI binary: devices | install | import-cert
│   │   └── gui/                  # Tauri shell (scaffold): HTML + i18n en/vi
│   └── ios/              # TRACK: on-device app (Swift, SideStore fork, AGPL)
│                         # Owned by the iOS track. Desktop contributors: DO NOT TOUCH.
├── tools/
│   └── win-installer/    # install-tsideload.bat — first-time Windows setup
└── docs/
    ├── ARCHITECTURE.md   # this file
    └── I18N.md           # i18n key conventions shared by both tracks
```

## apps/desktop

- **`tsideload-core`** — thin, opinionated wrapper around the [`isideload`]
  Rust crate (MIT, https://github.com/nab138/isideload), which implements the
  Apple provisioning dance: anisette → Apple ID login (+2FA) → development
  certificate → App ID → provisioning profile → re-sign IPA → install over
  usbmuxd. Modules:
  - `devices` — list USB-connected devices via `idevice::usbmuxd`.
  - `auth` — `login(apple_id, password, ask_2fa)` → reusable `AppleSession`.
    The caller supplies the 2FA prompt (CLI reads stdin; GUI will show a dialog).
  - `install` — `install_ipa(session, ipa_path, opts)`; picks target device by
    UDID or the first connected one. Uses file-based storage
    (`FsStorage`, `./tsideload-data`) for anisette state/certs — no OS keyring
    dependency.
  - `certs` — **TODO (P1)**: import user-provided `.p12` + `.mobileprovision`
    (KSign-style). Upstream `isideload` support is still rudimentary/untested;
    the module documents the plan and `tsd import-cert` reports status.
- **`tsd`** — CLI for scripting and for users who prefer a terminal.
- **`gui/`** — Tauri v2 shell. Frontend is static HTML/JS with `i18n/en.json`
  + `i18n/vi.json` (see `docs/I18N.md`). Tauri commands call into
  `tsideload-core`; `list_devices` is wired, `install_ipa` is stubbed for P1
  (needs an interactive Apple ID + 2FA dialog flow).

### Sign & install flow (desktop)

```
iPhone --USB--> usbmuxd --> tsd/tsideload-core
                                    |
  1. AppleAccount::builder(apple_id) --anisette--> login(password, 2FA cb)
  2. DeveloperSession::from_account()  (fetch/create dev certificate)
  3. SideloaderBuilder { team: First, storage: FsStorage, machine: "tsideload-desktop" }
  4. install_app(provider, ipa): ensure_device_registered -> sign_app -> install
  5. iPhone: Settings > General > VPN & Device Management > trust the Apple ID
```

Free Apple ID limits (Apple's rules): 3 apps, 7-day certificates, no push
notifications. Re-running install re-signs and refreshes.

### Building

- `cargo check` / `cargo build` at `apps/desktop` covers `tsideload-core` + `tsd`
  on **any OS including Linux** — no system dependencies.
- The Tauri GUI (`apps/desktop/gui`) is **excluded** from the workspace and
  builds only on **Windows/macOS** (needs WebView2 / WebKit; this Linux VM
  lacks `webkit2gtk`, so the GUI is scaffold-only here and is never compiled
  in CI-on-Linux).

## apps/ios

On-device signer (Swift), forked from SideStore (AGPL-3.0): local VPN loopback
trick so the iPhone re-signs its own apps, no PC needed after the one-time
USB install. **Separate track, separate agent — see `apps/ios/AGENTS.md`.**
Shared contracts with desktop: AltStore source JSON format and `docs/I18N.md`.

## tools/win-installer

`install-tsideload.bat` — first-time setup on Windows (ASCII-only file, so
`cmd.exe` never chokes on encoding):
1. Verifies Apple Devices / iTunes **from Apple's website** is installed
   (Microsoft Store builds hide the libraries we need) and offers the download
   link if missing.
2. Downloads the portable `tsd` build from GitHub Releases (CI publishes it —
   P1; until then the script tells the user to build from source).
3. Prints the cable + "Trust this computer" checklist and runs `tsd devices`
   as a smoke test.

## Banking apps

Nothing here is a jailbreak and nothing modifies iOS system files: apps are
signed with the user's own Apple ID through Apple's standard free-provisioning
flow, exactly like Xcode does. Banking apps that refuse *jailbroken* devices
are unaffected. (No tool can promise every bank's heuristics, but the
mechanism is the legitimate one.)

[`isideload`]: https://github.com/nab138/isideload
