# ADR-007: WINE-bridge research and interop strategy

- Status: Research complete, implementation pending
- Date: 2026-06-08
- Related: ADR-006 (modality), ADR-008 (MCP/CLI surfaces)

## Goal

Allow PlayCua running on a Linux host (or macOS) to drive a Windows binary
(`notepad.exe`, `chrome.exe`, `devenv.exe`, etc.) without requiring a Windows
VM. The WINE-bridge modality is the cornerstone of "NVMSCUA" — it's what
makes the `wsl` and `container` modalities actually useful on non-Windows
hosts.

## State of the art (2026-06-08)

### Foundational projects

| Project | License | Status | What it gives us |
|---------|---------|--------|------------------|
| [WINE](https://www.winehq.org/) | LGPL | mature (25+ years) | Windows API reimpl on POSIX. The canonical option. |
| [Proton](https://github.com/ValveSoftware/Proton) | BSD | mature (Steam) | WINE + DXvk + Steam glue. Optimized for games, not general use. |
| [DXvk](https://github.com/doitsujin/dxvk) | zlib | mature | Vulkan-based translation of Direct3D 9/10/11. Drop-in for WINE/Proton. |
| [VKD3D-Proton](https://github.com/HansKristian-Work/vkd3d-proton) | LGPL | mature | Vulkan translation of Direct3D 12. |
| [BoxedWine](http://www.boxedwine.org/) | — | niche | x86 emulator + WINE. Cross-arch (ARM host) capability. |

### Rust ecosystem

| Crate | License | Status | What it gives us |
|-------|---------|--------|------------------|
| `wine` ([ethanuppal/wine-rs](https://github.com/ethanuppal/wine-rs)) v0.1.0 | MPL-2.0 | early | Rust types and FFI to WINE's wineserver protocol |
| `wine-apc` v0.0.0 | — | prototype | Wineserver protocol client for non-Wine Linux processes |
| `is_wine` v0.1.2 | MIT | stable | Detect if running under WINE |
| `wsl` v0.1.0 | — | stable | Detect WSL runtime |
| `rdp-rs` v0.1.0 / `rdp-rs-2` v0.1.2 | — | early | Pure Rust RDP (useful for headless remote control) |
| `x11quic` v0.1.0 | — | early | X11 over QUIC (low-latency remote display) |

### Capability matrix

| Need | Today's Rust ecosystem | SOTA non-Rust | Recommendation |
|------|-----------------------|----------------|----------------|
| Detect host = WSL | `wsl` crate | — | use `wsl` crate, simple |
| Detect host = WINE | `is_wine` crate | — | use `is_wine` crate |
| Drive WINE wineserver from a non-Wine Linux process | `wine-apc` (prototype) | — | wait for `wine` crate to mature, write a thin wrapper in the meantime |
| Translate DirectX 9/10/11 → Vulkan | none | DXvk (zlib) | shell out to DXvk + WINE |
| Translate DirectX 12 → Vulkan | none | VKD3D-Proton (LGPL) | shell out to VKD3D-Proton + WINE |
| Run a Windows .exe in a Linux container | none | `wine-in-docker` patterns, `dockur/windows` | implement via Docker + WINE + DXvk layered image |
| RDP server (for headless remote display) | `rdp-rs` (early) | FreeRDP, xrdp | evaluate `rdp-rs-2` first, fall back to shelling out |

## Decision

1. **Adopt WINE + DXvk + VKD3D-Proton** as the implementation for the `wsl`
   and `container` modalities. Do **not** fork WINE — we don't have the
   maintainer bandwidth and the upstream is mature.
2. **Shell out** to WINE for the heavy lifting. The Rust `wine` crate is too
   early (v0.1.0) to depend on for production, but we can watch it.
3. **Detect** WINE/WSL at runtime with the `is_wine` and `wsl` crates — they
   are stable, MIT-licensed, and trivially small.
4. **DXvk and VKD3D-Proton** are added as build-time + run-time
   dependencies of the `container` modality image, not as Rust deps. The
   `wine-bridge` Rust code (TODO) wraps process spawn / pipes to them.
5. **WSL2** is treated as a special case of `Container` on Windows hosts:
   the `wsl` modality detects WSL2, spawns the .exe inside the WSL2 distro
   via `wsl.exe --exec`, and bridges stdout/stderr to the host. (This
   is the inverse of the Linux-host case.)

## Implementation strategy

### Phase 1 (M1) — detect

```rust
// native/src/wine_bridge.rs (sketch)
use is_wine::is_wine;
use wsl::is_wsl;

pub enum HostKind {
    Linux,
    Macos,
    Windows,
    Wsl,
    Wine,
}

pub fn detect() -> HostKind {
    if is_wsl() { return HostKind::Wsl; }
    if is_wine() { return HostKind::Wine; }
    if cfg!(target_os = "linux") { return HostKind::Linux; }
    if cfg!(target_os = "macos") { return HostKind::Macos; }
    if cfg!(target_os = "windows") { return HostKind::Windows; }
    unreachable!()
}
```

### Phase 2 (M2) — launch helper

```rust
// native/src/wine_bridge.rs (sketch)
pub fn launch_exe(exe_path: &Path, args: &[&str]) -> Result<Child, Error> {
    match detect() {
        HostKind::Linux | HostKind::Macos => {
            // Spawn `wine exe_path args...`
            Command::new("wine").arg(exe_path).args(args).spawn()
        }
        HostKind::Wsl => {
            // Spawn `wsl.exe --exec exe_path args...`
            Command::new("wsl.exe").arg("--exec").arg(exe_path).args(args).spawn()
        }
        HostKind::Windows => {
            // Direct spawn
            Command::new(exe_path).args(args).spawn()
        }
        HostKind::Wine => {
            // Already inside WINE — direct spawn
            Command::new(exe_path).args(args).spawn()
        }
    }
}
```

### Phase 3 (M3) — capture/input through WINE

For capture: hook the WINE desktop window via the host OS's capture API
(e.g. `xcap` on Linux X11, `CoreGraphics` on macOS). The WINE window is just
another X11 window to the host.

For input: WINE maps Windows input events to X11/Wayland events transparently.
PlayCua's existing X11/Wayland input adapters already work against WINE windows
without modification.

### Phase 4 (M4) — DXvk / VKD3D-Proton in container modality

`Dockerfile.containers.playcua` (proposed):

```dockerfile
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y wine dxvk vkd3d-proton xvfb
COPY playcua-native /usr/local/bin/
EXPOSE 3000
ENTRYPOINT ["playcua-mcp", "--transport", "http", "--bind", "0.0.0.0:3000"]
```

## Open questions

- Should `wine-bridge` modality be lazy-loaded (only when `--modality wine`
  is selected) so users who don't need it don't pull in `is_wine`/`wsl`?
  Yes — feature flag `wine-bridge` in `native/Cargo.toml`.
- Do we need to support macOS-hosted WINE? The `wine` fork for macOS is
  CrossOver / Wine-Staging. Yes, but lower priority than Linux.
- For ARM hosts (Apple Silicon, Linux ARM), x86 Windows apps need an
  emulator. `BoxedWine` is the only mature option. Track in a future ADR.

## Risks

- **DXvk license compatibility** with our MIT-licensed code: zlib is
  compatible, but the resulting binary's license obligations need review.
- **WineHQ policy on "commercial use"** of WINE: WINE is LGPL, fine for
  commercial use, but if we redistribute a WINE bundle, we need to honor
  LGPL linking obligations (dynamic linking recommended).
- **WINE wineserver protocol is not stable** — `wine-apc` will break.
  Mitigated by treating WINE spawn as a black box (don't reach into
  wineserver directly).
