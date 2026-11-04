# TSideload

**English** | [Tiếng Việt](#tsideload--tiếng-việt)

Free, open-source iOS sideloading platform — install your own `.ipa` files on
iPhone/iPad with a free Apple ID. No jailbreak, no paid developer account.

---

## For everyone (no terminal needed)

### 1. Desktop app (Windows)

1. Download **TSideload_0.1.0_x64-setup.exe** from the latest
   [`desktop-v*` release](https://github.com/goddie9x/TSideload/releases)
   and install it (next-next-finish).
   > Windows SmartScreen may warn because the installer isn't code-signed
   > yet — click "More info" → "Run anyway". It's built by this repo's own CI.
2. Connect your iPhone:
   - **USB**: plug it in and tap "Trust this computer".
   - **Wi-Fi** (no cable): one-time setup — enable "Sync with this iPhone
     over Wi-Fi" in Apple Devices / iTunes / Finder (needs the cable once);
     afterwards phone and PC just need the same Wi-Fi network.
   - **Driver**: install **Apple Devices** or **iTunes from Apple's website**
     — the Microsoft Store versions don't work with third-party tools.
3. Open the TSideload app: pick an `.ipa` file → pick your iPhone →
   click **Install**. Sign in with your Apple ID when asked (2FA supported),
   or use your own `.p12` certificate.

### 2. iOS app (on-device — no PC needed after the first install)

1. Download **TSideload-unsigned.ipa** from the latest
   [`ios-v*` release](https://github.com/goddie9x/TSideload/releases).
2. Install it **once** with the desktop app above — this signs it with
   your Apple ID.
3. On the iPhone: turn on **Developer Mode** (Settings → Privacy & Security —
   one-time + restart; Apple requires this since iOS 16 for sideloaded apps),
   then trust your Apple ID under Settings → General → VPN & Device Management.
4. From now on, open the **TSideload** app on the phone to install IPAs and
   refresh them — no PC needed.

**Free Apple ID limits** (set by Apple, not by us): **3 sideloaded apps**
at a time (TSideload itself counts as one), signatures expire after
**7 days** — just open the app and refresh. No push notifications with
free signing.

> Developer Mode is official Apple functionality, not jailbreak — but a few
> banking apps with aggressive security SDKs may refuse to run while it's
> on. If one does, turn Developer Mode off (restart) while using it.

---

## For advanced users

### CLI (`tsd`)

Prefer the terminal? Grab the prebuilt `tsd` from the latest
[`desktop-v*` release](https://github.com/goddie9x/TSideload/releases)
(`tsd-x86_64-pc-windows-msvc.zip` / `tsd-x86_64-unknown-linux-gnu.zip`) —
no Rust needed.

```sh
tsd devices                                        # list connected devices
tsd install app.ipa --apple-id you@example.com     # sign + install (2FA via prompt)
tsd import-cert --p12 cert.p12 --provision app.mobileprovision --name mycert
tsd install app.ipa --cert mycert                  # install with your own certificate
```

### Build from source

```sh
# 1. Install Rust: https://rustup.rs
# 2. CLI:
git clone https://github.com/goddie9x/TSideload && cd TSideload
cargo install --path apps/desktop/crates/tsd
# 3. GUI (Windows/macOS): needs WebView2/WebKit
cd apps/desktop/gui && cargo install tauri-cli --version "^2" --locked && cargo tauri build
# 4. iOS app: built by CI (.github/workflows/build-ios-ipa.yml), macOS only
```

## Layout

| Path | What |
|---|---|
| `apps/desktop/` | Desktop signer: Rust core (`tsideload-core`), CLI (`tsd`), Tauri GUI |
| `apps/ios/` | On-device iOS app (SideStore fork, Swift). **Owned by the iOS track — do not touch from desktop work.** |
| `tools/win-installer/` | First-time Windows installer (`.bat`) |
| `docs/` | Architecture + i18n conventions shared by both tracks |

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).

---

## TSideload — Tiếng Việt

Nền tảng sideload iOS miễn phí, mã nguồn mở — cài file `.ipa` của bạn lên
iPhone/iPad bằng Apple ID miễn phí. Không jailbreak, không cần tài khoản
developer trả phí.

---

## Cho mọi người (không cần chạm terminal)

### 1. App desktop (Windows)

1. Tải **TSideload_0.1.0_x64-setup.exe** từ bản
   [`desktop-v*` mới nhất](https://github.com/goddie9x/TSideload/releases)
   rồi cài đặt (next-next-finish).
   > Windows SmartScreen có thể cảnh báo vì file chưa ký số — bấm "More info"
   > → "Run anyway". File do chính CI của repo này build ra.
2. Kết nối iPhone:
   - **Cáp USB**: cắm vào, bấm "Tin cậy máy tính này".
   - **Wi-Fi** (không cần cáp): làm 1 lần duy nhất — bật "Sync with this
     iPhone over Wi-Fi" trong Apple Devices / iTunes / Finder (cần cáp 1 lần);
     sau đó điện thoại và PC chỉ cần chung 1 mạng Wi-Fi.
   - **Driver**: cài **Apple Devices** hoặc **iTunes bản tải từ trang Apple**
     — bản Microsoft Store không dùng được cho tool bên thứ ba.
3. Mở app TSideload: chọn file `.ipa` → chọn iPhone → bấm **Install**.
   Đăng nhập Apple ID khi được hỏi (hỗ trợ 2FA), hoặc dùng chứng chỉ
   `.p12` của riêng bạn.

### 2. App iOS (trên máy — sau lần đầu không cần PC nữa)

1. Tải **TSideload-unsigned.ipa** từ bản
   [`ios-v*` mới nhất](https://github.com/goddie9x/TSideload/releases).
2. Cài **1 lần duy nhất** bằng app desktop ở trên — lúc này nó được ký
   bằng Apple ID của bạn.
3. Trên iPhone: bật **Developer Mode** (Cài đặt → Quyền riêng tư & Bảo mật —
   làm 1 lần + khởi động lại; Apple bắt buộc từ iOS 16 cho app sideload),
   rồi trust Apple ID trong Cài đặt → Cài đặt chung → VPN & Quản lý thiết bị.
4. Từ đây mở app **TSideload** trên điện thoại để cài IPA và tự gia hạn —
   khỏi cần PC nữa.

**Giới hạn của Apple ID miễn phí** (do Apple đặt, không phải do tool):
tối đa **3 app** cùng lúc (bản thân TSideload tính là 1), chữ ký hết hạn
sau **7 ngày** — mở app lên bấm gia hạn là xong. Ký miễn phí thì không có
push notification.

> Developer Mode là tính năng chính thức của Apple, không phải jailbreak —
> nhưng vài app ngân hàng dùng SDK bảo mật gắt có thể không chịu chạy khi
> nó đang bật. Nếu gặp thì tắt Developer Mode (khởi động lại) lúc dùng app đó.

---

## Cho dân kỹ thuật

### CLI (`tsd`)

Thích dùng lệnh? Tải `tsd` build sẵn từ bản
[`desktop-v*` mới nhất](https://github.com/goddie9x/TSideload/releases)
(`tsd-x86_64-pc-windows-msvc.zip` / `tsd-x86_64-unknown-linux-gnu.zip`) —
không cần cài Rust.

```sh
tsd devices                                        # liệt kê máy đang kết nối
tsd install app.ipa --apple-id you@example.com     # ký + cài (2FA nhập tương tác)
tsd import-cert --p12 cert.p12 --provision app.mobileprovision --name mycert
tsd install app.ipa --cert mycert                  # cài bằng chứng chỉ của bạn
```

### Build từ source

```sh
# 1. Cài Rust: https://rustup.rs
# 2. CLI:
git clone https://github.com/goddie9x/TSideload && cd TSideload
cargo install --path apps/desktop/crates/tsd
# 3. GUI (Windows/macOS): cần WebView2/WebKit
cd apps/desktop/gui && cargo install tauri-cli --version "^2" --locked && cargo tauri build
# 4. App iOS: build bằng CI (.github/workflows/build-ios-ipa.yml), chỉ chạy trên macOS
```

## Bố cục repo

| Đường dẫn | Chức năng |
|---|---|
| `apps/desktop/` | Ký app trên desktop: core Rust (`tsideload-core`), CLI (`tsd`), GUI Tauri |
| `apps/ios/` | App iOS on-device (fork SideStore, Swift). **Thuộc nhánh iOS — đừng đụng từ nhánh desktop.** |
| `tools/win-installer/` | Bộ cài Windows lần đầu (`.bat`) |
| `docs/` | Kiến trúc + quy ước i18n dùng chung 2 nhánh |

## Giấy phép

AGPL-3.0-or-later. Xem [LICENSE](LICENSE).
