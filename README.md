# TSideload

**English** | [Tiếng Việt](#tsideload--tiếng-việt)

Free, open-source iOS sideloading platform — install your own `.ipa` files on
iPhone/iPad with a free Apple ID. No jailbreak, no paid developer account.

> **Status: P0 scaffold (2026-11-02).** The desktop track (`apps/desktop`) has a
> working Rust core + CLI; the Tauri GUI is scaffolded. The on-device iOS app
> (`apps/ios`, a SideStore fork) is being built in parallel — see its own
> `apps/ios/AGENTS.md`.

## Layout

| Path | What |
|---|---|
| `apps/desktop/` | Desktop signer: Rust core (`tsideload-core`), CLI (`tsd`), Tauri GUI shell |
| `apps/ios/` | On-device iOS app (SideStore fork, Swift). **Owned by the iOS track — do not touch from desktop work.** |
| `tools/win-installer/` | First-time Windows installer (`.bat`) |
| `docs/` | Architecture + i18n conventions shared by both tracks |

## Quick start (desktop)

Requirements on the PC: iPhone/iPad connected over USB, "Trust this computer"
tapped on the device, and the Apple mobile-device driver (installed by
**Apple Devices** or **iTunes downloaded from Apple's website** — the Microsoft
Store versions do not expose the libraries third-party tools need).

```sh
# list connected devices
tsd devices

# sign with your free Apple ID and install (password via prompt or
# TSIDELOAD_APPLE_PASSWORD env; 2FA code via interactive prompt)
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

> **Trạng thái: scaffold P0 (02/11/2026).** Nhánh desktop (`apps/desktop`) đã có
> core Rust + CLI chạy được; GUI Tauri mới scaffold. App iOS on-device
> (`apps/ios`, fork từ SideStore) đang làm song song — xem `apps/ios/AGENTS.md`.

## Bắt đầu nhanh (desktop)

Máy tính cần: iPhone/iPad cắm cáp USB, đã bấm "Tin cậy máy tính này" trên máy,
và driver Apple (cài từ **Apple Devices** hoặc **iTunes bản tải từ trang Apple**
— bản Microsoft Store không dùng được cho tool bên thứ ba).

```sh
# liệt kê máy đang cắm
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
