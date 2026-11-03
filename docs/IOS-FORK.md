# TSideload iOS app — fork notes (`apps/ios/`)

On-device sideloading app = fork of **SideStore** (Swift, AGPL-3.0),
rebranded **TSideload**. Full provenance in `apps/ios/ATTRIBUTION.md`.

## What was changed (P0b, 2026-11-02)

- **Vendor:** SideStore @ `cd260851ae99bf1fc946b123db24e89c8cd21dc6`
  (2026-11-02) + submodules flattened:
  minimuxer @ `12be70dc` (AGPL-3.0), SideSign @ `752d1ef6` (GPL-3.0).
- **Bundle ID:** `com.SideStore.SideStore` → `dev.hmt9x.tsideload`
  (via `ORG_IDENTIFIER`/`BASE_BUNDLE_ID` in `Build.xcconfig`;
  hardcoded IDs in `Shared/Extensions/Bundle+AltStore.swift`,
  both `ReleaseEntitlements.plist`, `Info.plist` query schemes +
  `OSLogPreferences` updated).
- **Display name:** `TSideload` (`PRODUCT_NAME` + explicit
  `CFBundleDisplayName` in `AltStore/Info.plist`).
- **Vietnamese:** `AltStore/Resources/vi.lproj/Localizable.strings`,
  200 main-screen strings translated EN→VI (placeholders `%@`
  verified, product name rebranded inside translations).
- **Untouched:** all signing / refresh / VPN-loopback / pairing logic,
  all `LICENSE` + copyright headers.

## AGPL compliance checklist

- [x] `LICENSE` (AGPL-3.0) kept verbatim; submodule licenses kept
      (`Dependencies/minimuxer/LICENSE`, SideSign GPL-3.0 notice).
- [x] Copyright headers in source files preserved.
- [x] `ATTRIBUTION.md` records upstream repo + vendored commits + changes.
- [x] Fork lives in public repo `goddie9x/TSideload` (source offer for any
      distributed binary).
- [ ] When distributing an IPA build, include a written source offer /
      link back to this repo (do at release time, P1).
- [ ] No upstream GitHub automation (no bot PRs/issues/comments) — see
      note on `AGENTS.md` in `ATTRIBUTION.md`.

## Localization status

- `vi.lproj/Localizable.strings`: 200/596 keys (main screens: tab bar,
  My Apps, Browse, News, Settings, App Detail, sign-in, Sources).
- Remaining 396 keys: TODO (error details, advanced diagnostics, onboarding).
- **Registration (done 2026-11-03, P1c):** this project uses Xcode 16+
  file-system synchronized groups (`PBXFileSystemSynchronizedRootGroup`
  for `AltStore/`), so there are no manual `PBXFileReference` /
  `PBXVariantGroup` / `PBXBuildFile` entries for resources — Xcode
  auto-discovers `Resources/vi.lproj/Localizable.strings` from disk and
  treats it as a localization of `Localizable.strings`. The only project
  change needed was adding `vi` to `knownRegions` in
  `AltStore.xcodeproj/project.pbxproj` (the same thing Xcode does when a
  localization is added via UI). Verified by CI
  (`.github/workflows/ios-validate.yml`: `xcodebuild -list` parse check
  + `knownRegions` grep + `plutil -lint` on the strings file).
- New strings added later must be translated; keep `%@` counts in sync.

## TODO (before first real build)

1. ~~Add `vi.lproj` to Xcode project (Mac).~~ Done via `knownRegions` +
   synchronized-folder auto-discovery (see above); pending CI green.
2. App icon: replace `AltStore/Resources/Icons.xcassets/AppIcon.appiconset`
   with TSideload artwork (all sizes).
3. Launch screen: `AltStore/iOS/LaunchScreen.storyboard` still shows
   SideStore branding.
4. URL scheme rename `sidestore://` → `tsideload://` (optional but cleaner):
   `AltStore/Info.plist` (`CFBundleURLSchemes`, `CFBundleURLName`),
   dynamic handling in `SideBackup/SideBackupApp.swift` (~L110-130),
   `AltStore/Core/Model/InstalledApp.swift` (~L472),
   `SideStore/Core/Operations/PipelineOperations/ResignAppOperation.swift` (~L88).
   Note: breaks `sidestore://source?url=` deep links from the web.
5. `CodeSigning.xcconfig`: copy from `CodeSigning.xcconfig.sample` on the
   build Mac, set real `DEVELOPMENT_TEAM`.
6. Decide on `ALTAnisetteURL` (`https://ani.sidestore.io`): keep using
   upstream's anisette server, or self-host.

## Roadmap → code pointers

### 1. Multi Apple ID (several IDs at once, not just switch-account)
- Account/team storage: `AltStore/Core/Model/Team.swift`,
  `AltStore/Core/Model/DatabaseManager/DatabaseManager.swift`
  (`activeTeam(in:)` — currently single active team).
- Credentials: `AltStore/Core/Components/Keychain.swift`,
  sign-in UI: `AltStore/Authentication/AuthenticationViewController.swift`,
  team picker: `AltStore/Authentication/SelectTeamViewController.swift`.
- Approach: allow N `Team` rows, per-app team assignment, UI to pick
  signing identity per install/refresh. Refresh ops in
  `SideStore/Core/Operations/StandaloneOperations/BackgroundRefreshAppsOperation.swift`
  currently assume one active team.

### 2. Import `.p12` + `.mobileprovision` (KSign-style, user-provided certs)
- No such feature upstream — greenfield.
- Cert handling reference: `AltStore/Core/Model/Team.swift` (dev cert
  fetch), `Dependencies/SideSign` (signing primitives, GPL-3.0).
- Desktop `isideload` Rust crate already has rudimentary imported-cert
  support — port the *flow* (import p12 → keychain → build
  provisioning profile → sign), not the code, to Swift.
- Security: never bundle certs; import UI must warn about revocation risk.

### 3. Deeper LiveContainer integration
- Existing hooks: `// @livecontainer` markers in
  `AltStore/Core/Model/Source.swift`, `AltStore/My Apps/MyAppsViewController.swift`,
  `Shared/Extensions/Bundle+AltStore.swift` (`activeBundle` swizzling).
- Upstream `isideload` crate mentions a "livecontainer entitlements patch" —
  mirror that entitlement patching on iOS when signing for LiveContainer.

### 4. Auto VPN + Shortcuts (hands-free refresh)
- Intents already exist: `AltStore/Intents/App Intents/RefreshAllAppsIntent.swift`,
  `AppShortcuts.swift` — wire a scheduled Shortcut:
  VPN on → Refresh All → VPN off.
- VPN profile onboarding: `SideStore/Views/Onboarding/OnboardingView.swift`;
  connection config UI:
  `SideStore/Views/Settings/Advanced/Connection/ConnectionConfig(View).swift`.
- Add a Settings toggle "auto-enable VPN only during refresh"
  (iSideload-style), default off.

### 5. Later / verify-first
- "Install to another iPhone over Wi-Fi": **unverified** — SideStore
  architecture has no such path; treat vendor marketing with suspicion,
  spike before promising.
- Own AltStore-format source feed on hmt9x.dev (`apps.json`) so TSideload
  can offer 1-tap installs of our own apps (Sources tab already supports
  AltStore-format sources: `AltStore/Sources/`).

## Build & test requirements

- **macOS + Xcode 15+ required.** Do NOT attempt on Linux (no toolchain).
- First open: Xcode resolves `Dependencies/` Swift Packages + EMProxy XCFramework.
- `make build fakesign ipa` produces a release IPA (see vendored `AGENTS.md`).
- **Real iPhone required** for any meaningful test (iPhone 13 available);
  target iOS 15+.
- CI (GitHub Actions macOS runner) for unsigned IPA builds: later, P1.
- Never commit: pairing files, `.p12`/certificates, private keys, anisette
  data, Apple IDs/passwords, device UDIDs.
