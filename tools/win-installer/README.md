# Windows first-time installer

`install-tsideload.bat` prepares a Windows PC for TSideload:

1. Checks the Apple mobile-device driver (Apple Devices / iTunes **from
   Apple's website** — Microsoft Store builds don't work).
2. Downloads the portable `tsd` build from GitHub Releases
   (published by CI in P1; until then build from source).
3. Adds `%LOCALAPPDATA%\TSideload\bin` to the user PATH.
4. Runs `tsd devices` as a smoke test (iPhone on USB, tap "Trust").

Keep the `.bat` file **ASCII-only** — `cmd.exe` mangles other encodings
(same root cause as the old `.ps1` UTF-8-BOM lesson).
