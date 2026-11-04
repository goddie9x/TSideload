# Đóng gói MSIX cho TSideload (Tauri GUI)

Tài liệu này ghi lại quy trình chuẩn bị MSIX cho **TSideload GUI** để phát
hành qua Microsoft Store (hoặc phát hành tay). **Giai đoạn này chỉ viết doc,
chưa đụng tới CI.**

> Trạng thái: Tauri v2 + Rust. GUI mới scaffold (P0/P1b, `apps/desktop/gui/`),
> đang chạy được trên Windows/macOS — chưa từng build trên CI.

## Stack & cơ chế build hiện tại

- **Stack:** Tauri v2 (`tauri = "2"` trong `apps/desktop/gui/src-tauri/Cargo.toml`)
  + Rust; frontend là HTML/JS tĩnh trong `apps/desktop/gui/src/`
  (không có bước build Node — `beforeBuildCommand` trống, repo không có
  `package.json` nào dưới `gui/`).
- **Lệnh build GUI** (đã verify từ `apps/desktop/gui/README.md`) — chạy trên
  **Windows hoặc macOS** (GUI bị loại khỏi workspace vì cần WebView2/WebKit,
  không compile được trên Linux bare):

```sh
cd apps/desktop/gui/src-tauri
cargo build --release
```

  Metadata Tauri (`apps/desktop/gui/src-tauri/tauri.conf.json`): `productName`
  = `TSideload`, `version` = `0.1.0`, `identifier` = `dev.hmt9x.tsideload`,
  `bundle.category` = `Utility`, `frontendDist` = `../src`.

- **CI hiện tại chỉ build CLI `tsd`, GUI chưa từng build trên CI** —
  `release-desktop.yml` chạy `cargo build --release -p tsd` và đóng zip
  (`tsd-x86_64-pc-windows-msvc.zip`) khi push tag `desktop-v*`. GUI
  (`apps/desktop/gui/src-tauri`) bị exclude khỏi workspace cargo nên không
  dính vào build nào.

## Đóng gói MSIX

Tauri **không có target MSIX chính thức** (`tauri-bundler` chỉ ra NSIS/MSI/
deb/rpm/dmg/AppImage) → đóng gói MSIX bằng **Windows App Development CLI
(winapp)** của Microsoft. WinApp CLI hỗ trợ Rust/Electron/C++/Python, không
riêng .NET.

### Yêu cầu

- Máy **Windows 11** (cài winapp qua winget):

```pwsh
winget install microsoft.winappcli --source winget
```

- Không cần mua code-signing cert cho bản test local — `winapp cert generate`
  tạo cert dev, `winapp cert install` cài vào máy để cài thử MSIX.

### Quy trình

1. Build GUI như mục trên (`cargo build --release` trong
   `apps/desktop/gui/src-tauri`).
2. Chuẩn bị thư mục `dist/` chứa output cần pack: **file exe** (vd
   `target/release/tsideload-gui.exe`) **+ `WebView2Loader.dll`** (Tauri v2
   dùng WebView2 trên Windows — loader phải đi kèm exe).
3. Khởi tạo identity/manifest trong thư mục dự án (hỏi interactive: Package
   name, Publisher, Version, SDK...):

```pwsh
winapp init
```

   rồi đóng gói:

```pwsh
winapp pack .\dist --manifest .\Package.appxmanifest --cert .\devcert.pfx
```

   (`pack` copy manifest vào thư mục target rồi sinh file `.msix`.)

4. Viết `Package.appxmanifest` (có thể sửa tay sau `winapp init`) — các điểm
   bắt buộc:

   - **Identity placeholder** (khi chưa có identity thật từ Partner Center):

     ```xml
     <Identity Name="TSideload" Publisher="CN=Dev" Version="0.1.0.0" />
     ```
   - **Version 4 phần:** Tauri đang `0.1.0` → MSIX dùng `0.1.0.0`
     (phần cuối = 0). Mỗi lần repack phải tăng version, Windows từ chối cài
     bản trùng hoặc thấp hơn.
   - **Capability `runFullTrust`** (app desktop full-trust, cần truy cập USB/
     tiến trình ngoài sandbox) — khai báo namespace restricted capabilities:

     ```xml
     <Package
       xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
       IgnorableNamespaces="... rescap">
       <Capabilities>
         <rescap:Capability Name="runFullTrust" />
       </Capabilities>
     </Package>
     ```

### Lưu ý WebView2

- **Windows 11** có sẵn WebView2 runtime → chạy ngay.
- **Windows 10** máy chưa cài WebView2 runtime thì app **không chạy** (Tauri
  không tự tải runtime về) → **phải test cả Windows 10** trước khi release,
  và cân nhắc ghi rõ yêu cầu trong trang Store / trang tải về.

### Lưu ý phạm vi

- **MSIX target là Tauri GUI, không phải CLI `tsd`.** CLI `tsd` tiếp tục phát
  hành dạng zip qua `release-desktop.yml` + `tools/win-installer/` như hiện
  tại, không đưa vào MSIX.

## CI hiện tại

Đọc qua `gh api repos/goddie9x/TSideload/contents/.github/workflows` (repo
gồm 3 workflow):

| Workflow | Việc làm |
|---|---|
| `release-desktop.yml` | Push tag `desktop-v*` → build CLI `tsd` release (Windows + Linux), đóng zip, tạo GitHub Release. **Không build GUI.** |
| `build-ios-ipa.yml` | Build app iOS (SideStore fork) ra `.ipa`. |
| `ios-validate.yml` | Validate nhánh iOS. |

> **Workflow build MSIX + nộp Store sẽ bổ sung sau theo kiến trúc CI chung,
> hiện chưa đụng tới.**

## Nộp Microsoft Store

1. **Identity thật** (Name / Publisher) lấy từ **Partner Center → Product
   management → View app identity details** **sau** khi tạo app trên Store.
   Đã wire vào workflow (`msix.yml`): `goddie9x.TSideload` /
   `CN=8633D8C8-01ED-4486-B41B-1CD6E3B8F80D` / `goddie9x` (04/11) —
   bản manual release dùng identity này, ký tay trước khi đăng tải.
2. **Version 4 phần**, phần cuối = 0 (vd `0.1.0.0`); mỗi submission phải tăng
   version.
3. **Không cần mua code-signing cert:** nộp gói **unsigned** lên Store,
   Microsoft tự re-sign bằng cert của Store trong quá trình certification.
   (Cert dev chỉ dùng cho test local/sideload tay.)
4. **Rủi ro certification:** TSideload là tool **ký và sideload IPA** bằng
   Apple ID — Store có thể xem đây là hành vi lách cơ chế phân phối của
   Apple/hệ điều hành và **rớt certification**. Chuẩn bị phương án dự phòng:
   **MSIX trước mắt dùng để phát hành tay** (tải trực tiếp / GitHub Release),
   Store là mục tiêu thử sau, không đặt cược toàn bộ vào đó.
