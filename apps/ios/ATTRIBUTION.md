# Attribution — TSideload iOS app

The iOS app in this directory (`apps/ios/`) is a **fork of SideStore**,
rebranded as **TSideload**. All original copyright notices and license
files are kept verbatim, as required by the license.

## Upstream

- **Project:** SideStore — on-device iOS sideloading with a personal Apple ID
  (local VPN loopback instead of a computer).
- **Repository:** https://github.com/SideStore/SideStore
- **Vendored commit:** `cd260851ae99bf1fc946b123db24e89c8cd21dc6`
  (default branch HEAD on 2026-11-02, shallow clone `--depth 1`).
- **License:** GNU Affero General Public License v3.0 — see `LICENSE`
  (kept verbatim from upstream). `CERTIFICATE-OF-ORIGIN.md` also kept.

### Vendored dependencies (flattened, `.git` removed)

Upstream uses git submodules; they are vendored here as plain directories:

| Directory | Upstream | Vendored commit | License |
|---|---|---|---|
| `Dependencies/minimuxer` | https://github.com/SideStore/minimuxer (`develop`) | `12be70dc2627307a16bfd2dc7a009080d5bec909` | AGPL-3.0 (`Dependencies/minimuxer/LICENSE` kept) |
| `Dependencies/SideSign` | https://github.com/SideStore/SideSign (`main`) | `752d1ef602aa792a432ffa1e71646dc1ecf7363c` | GPL-3.0 (per its README) |

`EMProxy.xcframework` is resolved via Swift Package Manager on first Xcode open
(macOS required).

## Changes made by the TSideload fork (P0b, 2026-11-02)

Minimal rebrand + Vietnamese localization. No signing/refresh logic touched.

1. **Bundle identifier** `com.SideStore.SideStore` → `dev.hmt9x.tsideload`:
   - `Build.xcconfig`: `ORG_IDENTIFIER = dev.hmt9x`,
     `BASE_BUNDLE_ID = $(ORG_IDENTIFIER).tsideload`
     (Release bundle ID = `dev.hmt9x.tsideload`; Debug appends `.$(DEVELOPMENT_TEAM)` as before).
   - `Shared/Extensions/Bundle+AltStore.swift`: `storeAppBundleIdentifier`,
     `appbundleIdentifier` hardcodes updated.
   - `AltStore/Resources/ReleaseEntitlements.plist`,
     `AltWidget/Resources/ReleaseEntitlements.plist`: application-identifier
     and keychain-access-group entries updated
     (`XYZ0123456.` is a placeholder team prefix, unchanged).
   - `AltStore/Info.plist`: `LSApplicationQueriesSchemes`
     (`sidestore-dev.hmt9x.tsideload`), `OSLogPreferences` dict key.
2. **Display name** → `TSideload`:
   - `Build.xcconfig`: `PRODUCT_NAME = TSideload`
     (flows into `CFBundleName`).
   - `AltStore/Info.plist`: explicit `CFBundleDisplayName = TSideload`.
3. **User-facing strings**: product name `SideStore` → `TSideload` inside the
   translated Vietnamese strings; English source strings untouched.
4. **Vietnamese localization**: new `AltStore/Resources/vi.lproj/Localizable.strings`
   (200 main-screen strings, EN→VI). NOTE: the file must be added to the
   Xcode project (AltStore target) on a Mac — see `docs/IOS-FORK.md`.
5. This file (`ATTRIBUTION.md`).

Deliberately **not** changed yet: URL schemes (`sidestore://`,
`sidestore-<bundleID>` — handled dynamically in code, rename is a TODO),
app icon / launch screen artwork (TODO), anisette server URL
(`https://ani.sidestore.io`, upstream infrastructure, still works).

## License compliance notes

- This fork is distributed under **AGPL-3.0**, same as upstream.
  Any binary distributed (IPA, TestFlight, EU marketplace) must be
  accompanied by an offer of the complete corresponding source —
  this public repo satisfies that.
- All upstream copyright headers in source files are preserved.
- `Dependencies/minimuxer` (AGPL-3.0) and `Dependencies/SideSign`
  (GPL-3.0) keep their own license notices; combined work stays AGPL-3.0.

## Upstream contribution rules

`AGENTS.md` (vendored from upstream) contains strict rules for contributing
**back to the SideStore repository** (LLM-usage declarations, no automated
GitHub posts/PRs/issues/comments, human-to-human maintainer interaction).
This fork does **not** interact with upstream GitHub in any automated way.
If a human maintainer later decides to upstream a patch, they must follow
those rules themselves.
