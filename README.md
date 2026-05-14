# eso_enc

ESP32 Rust application that starts a Wi-Fi access point and listens for encrypted UDP commands to control GPIO2 (LED).

## What it does

- Starts an AP:
  - SSID: `ESP32-Control`
  - Password: `12345678`
- Listens on UDP port `4210`
- Returns its public key when it receives `GET_PUBKEY`
- Accepts encrypted messages and toggles GPIO2:
  - `led` → set high
  - `close_led` → set low

## Build requirements

- Rust with the ESP toolchain (`channel = "esp"` from `rust-toolchain.toml`)
- ESP-IDF Rust environment

## Build

```bash
cargo build
```

## Notes

- Change the default AP credentials before real-world use (see `AP_SSID` and `AP_PASS` in `/home/runner/work/eso_enc/eso_enc/src/main.rs`).
- The project depends on a local crate path:
  - `encrypt = { path = "../omar-test" }`
- Make sure that crate exists at the expected path before building.
