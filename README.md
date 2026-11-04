# TSideload

**English** | [Tiếng Việt](#tsideload--tiếng-việt)

Free, open-source iOS sideloading platform — install your own `.ipa` files on
iPhone/iPad with a free Apple ID. No jailbreak, no paid developer account.

> **Status: P1 complete (2026-11-03).** The desktop track (`apps/desktop`) has a
> working Rust core + CLI (`tsd devices`/`install`/`import-cert`) and a Tauri
> GUI with both Apple ID and imported-certificate install flows. The on-device
> iOS app (`apps/ios`, a SideStore fork) has its first CI-built IPA — see
> `docs/IOS-FORK.md` ("Release builds").

## Layout

| Path | What |
|---|---|
| `apps/desktop/` | Desktop signer: Rust core (`tsideload-core`), CLI (`tsd`), Tauri GUI shell |
| `apps/ios/` | On-device iOS app (SideStore fork, Swift). **Owned by the iOS track — do not touch from desktop work.** |
| `tools/win-installer/` | First-time Windows installer (`.bat`) |
| `docs/` | Architecture + i18n conventions shared by both tracks |

## Quick start (desktop)

### 1. Install Rust

https://rustup.rs — on Windows run `rustup-init.exe`; on macOS/Linux:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone the repo and install the `tsd` CLI

```sh
git clone https://github.com/goddie9x/TSideload
cd TSideload
cargo install --path apps/desktop/crates/tsd
```

Reopen the terminal afterwards so `tsd` is on your `PATH`. (On Windows you
can alternatively run `tools/win-installer/install-tsideload.bat`, which
fetches a prebuilt `tsd` once the first desktop release is published.)

### 3. Connect your iPhone

- **USB**: plug it in and tap "Trust this computer" on the device.
- **Wi-Fi** (no cable): enable "Sync with this iPhone over Wi-Fi" in
  Apple Devices / iTunes / Finder — this one-time setup needs the USB cable;
  afterwards the phone and PC just need to be on the same Wi-Fi network.
- **Windows driver**: install **Apple Devices** or **iTunes downloaded from
  Apple's website** — the Microsoft Store versions do not expose the
  libraries third-party tools need.

### 4. Install an app

```sh
# list connected devices (usb, or network:<ip> over Wi-Fi)
tsd devices

# sign with your free Apple ID and install (password via prompt or the
# TSIDELOAD_APPLE_PASSWORD env var; 2FA code via interactive prompt)
tsd install /path/to/app.ipa --apple-id you@example.com
```

Free Apple IDs are limited by Apple (not by us): **3 sideloaded apps**,
certificates expire after **7 days** — re-run `tsd install` to refresh.
No push notifications with free signing.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).

---

## TSideload — Tiếng Việt

Nền tảng sideload iOS miễn phí, mã nguồn mở — cài file `.ipa` của bạn lên
iPhone/iPad bằng Apple ID miễn phí. Không jailbreak, không cần tài khoản
developer trả phí.

> **Trạng thái: xong P1 (03/11/2026).** Nhánh desktop (`apps/desktop`) đã có
> core Rust + CLI chạy được (`tsd devices`/`install`/`import-cert`) và GUI
> Tauri với cả 2 flow cài (Apple ID và chứng chỉ import). App iOS on-device
> (`apps/ios`, fork từ SideStore) đã có file IPA build từ CI đầu tiên — xem
> `docs/IOS-FORK.md` (mục "Release builds").

## Bắt đầu nhanh (desktop)

### 1. Cài Rust

https://rustup.rs — Windows: chạy `rustup-init.exe`; macOS/Linux:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone repo và cài CLI `tsd`

```sh
git clone https://github.com/goddie9x/TSideload
cd TSideload
cargo install --path apps/desktop/crates/tsd
```

Mở lại terminal sau đó để `tsd` nằm trong `PATH`. (Trên Windows có thể chạy
`tools/win-installer/install-tsideload.bat` để tải `tsd` build sẵn, sau khi
bản desktop release đầu tiên được publish.)

### 3. Kết nối iPhone

- **Cáp USB**: cắm vào, bấm "Tin cậy máy tính này" trên điện thoại.
- **Wi-Fi** (không cần cáp): bật "Sync with this iPhone over Wi-Fi" trong
  Apple Devices / iTunes / Finder — bước này làm 1 lần duy nhất và cần cáp;
  sau đó điện thoại và PC chỉ cần chung 1 mạng Wi-Fi.
- **Driver trên Windows**: cài **Apple Devices** hoặc **iTunes bản tải từ
  trang Apple** — bản Microsoft Store không dùng được cho tool bên thứ ba.

### 4. Cài app

```sh
# liệt kê máy đang kết nối (usb, hoặc network:<ip> qua Wi-Fi)
tsd devices

# ký bằng Apple ID miễn phí rồi cài (mật khẩu nhập tay hoặc qua biến môi trường
# TSIDELOAD_APPLE_PASSWORD; mã 2FA nhập tương tác)
tsd install /path/to/app.ipa --apple-id you@example.com
```

Giới hạn của Apple ID miễn phí (do Apple đặt, không phải do tool): tối đa
**3 app**, chứng chỉ hết hạn sau **7 ngày** — chạy lại `tsd install` để gia hạn.
Ký miễn phí thì không có push notification.

## Giấy phép

AGPL-3.0-or-later. Xem [LICENSE](LICENSE).
