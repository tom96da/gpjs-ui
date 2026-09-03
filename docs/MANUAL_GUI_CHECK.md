<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Manually verifying a GPUI window opens

Some PLAN.md tasks include a manual checklist item like "run the example
and look at the window" — a human visually confirming a real window opens
and renders, which an agent working inside the devcontainer can't do on its
own (the devcontainer has no display attached; see
[docs/STRUCTURE.md](./STRUCTURE.md)). This doc covers how to do that check
yourself, for the current example
(`crates/gpjs-ui/examples/gpui/hello_world.rs`) and any future one.

There are two ways to see the window, depending on which platform's
rendering backend you want to exercise. Option A is simpler and is enough
for most checks; use Option B when you specifically want to exercise the
Linux backend (e.g. because that's what CI/the devcontainer itself runs on).

## Option A — run natively on macOS (recommended)

The devcontainer just bind-mounts this repo into a container; the files
also exist on the host at whatever path you opened in VS Code. `gpui`'s
Cargo dependency graph already branches per OS (macOS uses `gpui_apple` /
Metal, Linux uses `gpui_linux` / Vulkan), so running the same `cargo`
command directly on macOS, outside the container, gets you the native
backend with no extra setup:

```sh
cargo run -p gpjs-ui --example gpui_hello_world
```

A window with a gray background, the text "Hello, World!", and a row of six
colored boxes should appear.

### No text, but the background/boxes render fine

`gpui_platform`'s `font-kit` feature isn't enabled for this platform (see
`crates/gpjs-ui/Cargo.toml`) — without it, `gpui_macos` silently skips all
text rendering while drawing everything else normally, with no error.

### Metal toolchain errors

If this fails with something like:

```
error: cannot execute tool 'metal' due to missing Metal Toolchain; use: xcodebuild -downloadComponent MetalToolchain
```

or

```
xcrun: error: unable to find utility "metal", not a developer tool or in PATH
```

`gpui_apple`'s build script needs the `metal` shader compiler, which ships
with the full Xcode.app, not the standalone Command Line Tools. Fix:

1. Install Xcode from the App Store if you haven't.
2. Point the active developer directory at it:
   ```sh
   sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
   sudo xcodebuild -license accept
   ```
3. Download the Metal toolchain component:
   ```sh
   xcodebuild -downloadComponent MetalToolchain
   ```
   This downloads ~688 MB. On macOS 15 (Sequoia) and later, you may see a
   log line like:
   ```
   Metal Toolchain unable to refresh cache with error: … "Operation not permitted"
   ```
   The download still completes, but the automatic installation step is
   blocked by SIP. In that case, install the toolchain manually:
   ```sh
   # 1. Find the downloaded DMG (path will contain a hash-like directory name)
   DMG=$(find /System/Library/AssetsV2/com_apple_MobileAsset_MetalToolchain \
         -name "*.dmg" 2>/dev/null | head -1)

   # 2. Mount it
   hdiutil attach "$DMG" -nobrowse

   # 3. Copy the toolchain into your user toolchains directory
   mkdir -p ~/Library/Developer/Toolchains
   cp -R /Volumes/MetalToolchainCryptex/Metal.xctoolchain \
         ~/Library/Developer/Toolchains/

   # 4. Unmount
   hdiutil detach /Volumes/MetalToolchainCryptex
   ```
4. Confirm: `xcrun -sdk macosx metal --version` should print a version line.

If `xcrun`/`xcodebuild` itself errors with a dynamic-library/symbol-loading
failure (e.g. `Symbol not found: _XPCTypeBool` from
`libxcodebuildLoader.dylib`), or the component download fails with
`Failed fetching catalog for assetType (com.apple.MobileAsset.MetalToolchain)`,
that's a broken or version-mismatched Xcode install, not something specific
to this project. Fall back to Option B rather than chasing it — reinstalling
Xcode from scratch is the usual fix if you want to come back to Option A
later.

## Option B — from the devcontainer, forwarded to macOS via XQuartz

Confirms the Linux backend (`gpui_linux`) instead, without leaving the
container. **Untested end-to-end.** An agent's headless `Xvfb` attempt
from inside the container (no XQuartz) rendered nothing — not even the
background — for a reason not yet found, so this path may hit the same
issue.

1. Install XQuartz on the Mac host (not inside the container):
   ```sh
   brew install --cask xquartz
   ```
   After a fresh install, log out and back in — this is a known Homebrew
   caveat, without it XQuartz doesn't finish registering itself.
2. Launch XQuartz. It has no window of its own to open — look for an "X"
   icon in the menu bar to confirm it's running.
3. From that menu bar icon, open Settings → Security, and check "Allow
   connections from network clients." Quit and relaunch XQuartz for this to
   take effect.
4. On the Mac host (not inside the container), allow incoming connections:
   ```sh
   xhost +
   ```
   (`xhost -` reverts this once you're done.)
5. Inside the devcontainer:
   ```sh
   DISPLAY=host.docker.internal:0 cargo run -p gpjs-ui --example gpui_hello_world
   ```

In principle, the same window should appear on the Mac desktop, rendered
by XQuartz — but see the "Untested end-to-end" note above.

## Why an agent can't just do this

The devcontainer has no display attached (see
[.devcontainer/devcontainer.json](../.devcontainer/devcontainer.json) — no
X11/Wayland socket is mounted in). An agent working inside it can run the
example under a headless `Xvfb` and confirm the process doesn't crash and a
correctly sized/positioned window gets created (e.g. via `xwininfo`) — but
not that pixel content actually renders (see the note under Option B).
Confirming that needs a real compositor and a human looking at it.
