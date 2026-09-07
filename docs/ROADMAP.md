<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Roadmap

Planned phased implementation of the design in
[docs/ARCHITECTURE.md](./ARCHITECTURE.md). This file describes phase-level
design intent only — it doesn't track progress itself. See
[AGENTS.md](../AGENTS.md#status) for which phases have landed so far and
[docs/PLAN.md](./PLAN.md) for the checkbox-tracked, per-task breakdown.

Vue 3 support is built first end-to-end (Phases 1–3), through to the
`v0.0.1` release inside Phase 3.3. What follows that release is what an app
needs before it can be written at all — the input it is driven by, the
accessibility that input model makes possible, and the runtime facilities
every app reaches for — before the surface broadens to more styling, more
platform, and a second frontend framework.

| Phase | Scope |
| --- | --- |
| 1 | Rust host & FFI bridge core |
| 2 | JS core bridge & Vue 3 custom renderer |
| 3 | Developer tooling & HMR — `v0.0.1` ships at 3.3 |
| 4 | Input & text editing |
| 5 | Accessibility |
| 6 | Runtime standard library |
| 7 | Majority style & Tailwind coverage |
| 8 | Application shell & platform integration |
| 9 | React custom renderer |
| 10 | Cross-platform support |
| 11 | 100% style & Tailwind parity |
| 12 | App-owned Rust extensions |

Each phase below names the gate it waits on; the numbering is the order they
are built in, not a set of independent tracks.

## Phase 1: Rust host & FFI bridge core (`gpjs-ui`)

1. **QuickJS context setup**: use `rquickjs` to spin up a managed QuickJS
   runtime inside the GPUI event loop.
2. **Retained virtual tree**: an in-memory `VirtualNode` structure — see
   [docs/FFI.md](./FFI.md#retained-virtual-tree).
3. **Binding functions** exposed to JS as `globalThis.__gpjsui_native__` — see
   [docs/FFI.md](./FFI.md#binding-functions).
4. **GPUI rendering pipeline**: recursively convert the `VirtualNode` tree into
   GPUI `AnyElement` instances during GPUI's `render()` frame cycle.

## Phase 2: JS core bridge (`gpjs-ui`) & Vue 3 custom renderer (`@gpjs-ui/vue`)

1. **`gpjs-ui`** (`packages/gpjs-ui`): a framework-agnostic, typed JS wrapper
   around `globalThis.__gpjsui_native__` (see
   [docs/FFI.md](./FFI.md#binding-functions)) — see
   [docs/ARCHITECTURE.md](./ARCHITECTURE.md#tech-stack) for why this is a
   separate, shared package rather than logic duplicated into each framework
   adapter.
2. **`@gpjs-ui/vue`** (`packages/vue`): a custom Vue 3 runtime adapter
   using `@vue/runtime-core`'s `createRenderer`, built on `gpjs-ui` rather
   than calling `__gpjsui_native__` directly.
3. Map Vue node lifecycle methods (`createElement`, `insert`, `remove`,
   `patchProp`) to `gpjs-ui`'s calls.
4. A unified mount API, e.g. `createGpjsuiApp(App).mount('#root')`.

## Phase 3: Developer tooling & HMR integration

The `gpjsui` CLI's process orchestration is owned by the **JS/TS side**:
`@gpjs-ui/cli` (`packages/cli`) is the parent process, driving a bundler
adapter that holds Vite in-process, while `@gpjs-ui/host-client`
(`packages/host-client`) spawns the Rust host (`crates/gpjs-ui-host`) as a
child and bridges dev-server messages over its stdio. The alternatives
considered — a Rust-primary `crates/gpjs-ui-cli` owning everything, and
Rust-primary logic behind a thin npm `bin` wrapper — were rejected because:

- Node already owns every orchestration primitive this needs (Vite's own
  server/watch/restart API, `child_process`, terminal logging), where Rust
  would grow equivalent process/IPC/file-watching plumbing from scratch.
- The host binary ships through npm either way, since app authors aren't
  expected to have a Rust toolchain (Phase 12 is the one exception). A Rust
  CLI would add a second binary to distribute for no gain.
- It keeps `crates/gpjs-ui` and the host free of process/IPC concerns.

Phase 3 lands in four numbered stages, with real HMR deliberately **last**:
a full-reload dev loop already needs the whole spawn/teardown/remount
skeleton HMR builds on, and this ordering makes `dev`, `build`, and
packaging usable end-to-end before the hardest piece starts.

A GitHub Actions **CI** workflow running
[docs/TESTING.md](./TESTING.md)'s required checks lands before 3.1, so the
first multi-package phase isn't built without one. Its **CD** counterpart
lands after 3.3, when there is something to release.

### Phase 3.1: `gpjsui dev` (full reload)

1. **`@gpjs-ui/cli`** (`packages/cli`): owns the `gpjsui` commands, resolves
   an app's entry point, and wires the two packages below together. It holds
   the `Bundler` contract and injects an implementation, so swapping
   bundlers is a dependency change here and nothing else.
2. **`@gpjs-ui/vite`** (`packages/vite`): runs Vite in
   library/watch mode (not its browser dev server), using
   `@vitejs/plugin-vue` to compile `.vue` SFCs, and announces each rebuild.
   The only package that imports `vite`, and it depends on no first-party
   package — a `@gpjs-ui/rspack` would be a sibling, not a rewrite.
3. **`@gpjs-ui/host-client`** (`packages/host-client`): the Node end of
   [docs/PROTOCOL.md](./PROTOCOL.md) — resolves and launches the host
   binary, supervises the child, and carries messages both ways. It depends
   on no bundler and never parses a routed payload, so Vite's HMR traffic
   (Phase 3.4) rides the same channel as a registered `type` name.
4. **`crates/gpjs-ui-host`**: the runtime binary — opens the GPUI window
   and evaluates a bundle in QuickJS, and in dev mode reads newline-delimited
   JSON messages on stdin, re-evaluating the bundle in a fresh engine against
   a reset tree on each reload. Its stdout is the protocol channel; logs go
   to stderr.
5. **Native root handle**: a binding replacing Phase 2's
   `__GPJSUI_ROOT_ID__` source substitution, so an app's entry point is
   plain code (`createGpjsuiApp(App).mount()`) with no host-injected token
   in it.
6. **Core corrections**: node lifetime (nothing frees a detached node),
   error visibility (a listener's exception reaches nobody, and QuickJS has
   no `console`), and the per-frame cost of wiring every node for input.
   All three are `crates/gpjs-ui` gaps that a dev loop running a real app
   continuously makes unavoidable, so they land here rather than after the
   release.

Component state is *not* preserved across a reload — that's exactly what
Phase 3.4 adds.

### Phase 3.2: `gpjsui build`

The one-shot production counterpart of 3.1's pipeline, emitting a
self-contained bundle. Subsumes the per-example `scripts/build.mjs` files
Phase 2 Unit iv hand-rolled.

### Phase 3.3: Application packaging

Pairs a built bundle with a prebuilt host binary into a distributable
application — `.app` on macOS, with each other platform's target following
its support in Phase 10 — plus the per-platform npm distribution of those
prebuilt hosts. **Design constraint**: keep the host binary swappable, so
Phase 12 can substitute an app-compiled one.

This is the **first release milestone**: once packaging works, the framework
is published as `v0.0.1`, to npm only (`gpjs-ui`, `@gpjs-ui/vue`,
`@gpjs-ui/cli`, `@gpjs-ui/host-client`, `@gpjs-ui/vite`, and the
per-platform host packages). The Rust crates stay
`publish = false` — nothing outside this repo depends on them until Phase 12.

### Phase 3.4: HMR (`@gpjs-ui/vite-runtime`)

**HMR bridge** (`@gpjs-ui/vite-runtime`, at `packages/vite-runtime`): a
custom `ModuleRunnerTransport` and module evaluator against Vite's Runtime
API (`vite/module-runner`), so updated modules are evaluated inside QuickJS
and trigger a GPUI redraw while component state survives. The runner itself
runs inside QuickJS, not on the Node side — see
[docs/ARCHITECTURE.md](./ARCHITECTURE.md#hmr-delivery) for why, and for why
this is preferred over a hand-rolled HMR protocol.

## Phase 4: Input & text editing (future)

Not started, and not begun until Phase 3's tooling is stable. Phase 1 wired
exactly one input event — a click on a container, see
[docs/FFI.md](./FFI.md#event-dispatch-v1-click-only) — which is enough to
prove the dispatch path and not enough to write an application with. An app
is driven by input, so this is the first thing the `v0.0.1` release is
missing.

Everything here reaches JS through the existing `addEventListener` surface:
the host already dispatches any `(node id, event name)` pair, so a new event
is a name the host agrees to send, not a new binding.

1. **Pointer input**: press, release, move, enter, leave, wheel, and the
   button and modifier state each carries. `"click"` becomes one name among
   many rather than the only one.
2. **Keyboard input**: key press/release with modifiers, and a focus model
   deciding which node receives them. GPUI has its own focus handles, so
   this is a mapping rather than new machinery.
3. **Text editing**: an editable text element, with selection, caret, and
   IME composition. The largest item here, and the one with no partial
   version worth shipping — a text field that drops IME composition is
   unusable in Japanese, Chinese, or Korean.
4. **Scrolling**: a scrollable container. As much a layout capability as an
   input one, so `overflow` joins the style vocabulary here rather than
   waiting for Phase 7.
5. **Interactive visual state**: hover, active, and focus styling mapped
   onto GPUI's own element states rather than re-derived in JS. Phase 11's
   `hover:`/`focus:` Tailwind variants build on this.
6. **Event payloads**: a listener's callback takes only a node id today,
   which can't express a key, a pointer position, or a text edit. The
   callback signature and its `packages/gpjs-ui` wrapper are settled here,
   once, for every event that follows.

## Phase 5: Accessibility (future)

Not started, and not begun until Phase 4's focus and text models are stable
— a screen reader reads a focus path, so there is nothing to expose before
one exists.

Electron inherits this entire layer from Chromium. gpjs-ui renders to the
GPU directly and inherits nothing, so all of it is ours to build. It is
scheduled immediately after input rather than at the end because retrofitting
an accessibility tree onto a node vocabulary that grew without one means
rewriting that vocabulary.

1. **Semantics on the retained tree**: a node's role, name, value, and
   state, carried alongside `style_props`/`attributes`. The `tag_name`
   vocabulary is deliberately thin (see
   [docs/FFI.md](./FFI.md#tag-vocabulary-v1)), so semantics are declared
   rather than inferred from a tag.
2. **Platform accessibility APIs**: expose that tree through each platform's
   own API, following what GPUI already supports and filling in the rest.
3. **Keyboard reachability**: every interactive node reachable and operable
   without a pointer — Phase 4's focus model applied consistently.
4. **The author-facing surface**: `role`/`aria-*` on a `.vue` template maps
   onto the above, so an author writes what they already write for the web.

## Phase 6: Runtime standard library (future)

Not started. QuickJS provides the language and nothing else: no timers, no
network, no filesystem, no `console`. Phase 3.1 adds `console`; everything
else is still absent, and without it an app cannot poll, debounce, animate,
fetch, or read a file.

Each item is a host binding plus its typed wrapper in `packages/gpjs-ui`,
and each hands a new capability to app code — the FFI safety checklist in
[docs/PLAN.md](./PLAN.md) applies to all of them. They belong in
`crates/gpjs-ui-jsenv`, which depends on `rquickjs` alone so an
implementation can be swapped for a third-party one.

Check for one before writing any of these. `rquickjs-extra-*` (the rquickjs
org's own: timers, url, os, sqlite) and `llrt_modules` (AWS) both cover part
of this list, and both were pinned to `rquickjs` releases older than this
workspace's when `console` landed, which is why `console` is ours.

1. **Timers**: `setTimeout`/`setInterval` and their `clear` counterparts,
   driven by GPUI's own event loop rather than a second one. This is also
   what pumps the JS job queue between input events, which today is drained
   only when the host has a reason to run JS.
2. **Network**: `fetch` over a Rust HTTP client, and a WebSocket client.
   Both are asynchronous, so this is where promise integration stops being
   "drain what is already queued".
3. **Filesystem and paths**: reading and writing app data, with a recorded
   decision on what an app may reach. A desktop app is not a browser origin,
   so "everything the user can read" is a choice, not a default.
4. **Encoding and crypto**: `TextEncoder`/`TextDecoder`, `crypto`'s random
   sources, `structuredClone`, `URL` — small, standard, and assumed present
   by ordinary npm dependencies.
5. **Engine limits**: a memory ceiling, a stack ceiling, and an interrupt
   handler, so a runaway app stays recoverable instead of becoming a frozen
   window that answers no message (see
   [docs/PROTOCOL.md](./PROTOCOL.md#failure-handling)).

## Phase 7: Majority style & Tailwind coverage (future)

Not started, and not begun until Phase 3.1's Vite integration lands —
Tailwind's own JIT compiler runs as a build-time step, so it needs a real
Vite pipeline to plug into. Full CSS/Tailwind parity is not the goal here
(see Phase 11); this phase targets the "structural" utility categories that
cover the large majority of real-world usage and map cleanly onto GPUI's
native styling model:

1. **Native style vocabulary expansion** (`crates/gpjs-ui`): close the gaps
   flagged as "deliberately incomplete" since Phase 1 (see
   [docs/FFI.md](./FFI.md)) — margin/padding, percentage lengths, min/max
   size, flex-grow/shrink/basis, per-side border width/radius, basic
   box-shadow, font-weight/family, line-height/letter-spacing.
2. **Tailwind class resolver**: gpjs-ui has no real CSS engine, so Tailwind
   utility classes can't generate actual CSS — a Vite plugin (building on
   Phase 3's pipeline) scans `class="..."` usage and maps each recognized
   utility directly to a `setStyle` call, rather than through a stylesheet.
3. **Scope target: roughly 70–75% of Tailwind's utility classes** —
   layout/flexbox/grid, spacing, sizing, typography basics, solid
   background/text/border colors, borders/radius, basic shadow. Explicitly
   deferred to Phase 11: responsive breakpoint variants (`sm:`/`md:`/...),
   state variants (`hover:`/`focus:`/`group-*`), dark mode,
   animations/transitions, transforms, filters/backdrop-filters, and
   arbitrary bracket values (`w-[137px]`) — these need real design work
   (e.g. mapping `hover:` onto GPUI's own interactive element states)
   rather than a straightforward style-prop translation.

## Phase 8: Application shell & platform integration (future)

Not started, and not begun until Phase 3.3's packaging is stable — these are
the APIs a packaged application calls, and several have no meaning until
there is one.

A gpjs-ui app is one window with no way to address it. Everything an app
does *around* its content lives here.

1. **Windows**: title, size and position, minimize/maximize/fullscreen,
   close behaviour, and more than one window per app — which the host's
   one-session-per-process shape does not currently express.
2. **Native menus**: an application menu bar and context menus, with their
   keyboard shortcuts.
3. **Dialogs**: file open/save and message boxes, drawn by the platform
   rather than in-tree.
4. **System integration**: clipboard, notifications, a tray icon, and
   handing a URL or file to whatever the platform opens it with.
5. **Lifecycle beyond a reload**: window close, focus and blur, and the
   platform's own quit request — Phase 3.1 settles only the moments the dev
   protocol itself creates.

## Phase 9: React custom renderer (future)

Not started, and not begun until Vue 3 support (Phases 1–3) is stable. Adds
`@gpjs-ui/react` as an additional package alongside `@gpjs-ui/vue`, using
`react-reconciler` against the same `gpjs-ui` core package (not
`__gpjsui_native__` directly — see Phase 2), plus `@vitejs/plugin-react` for
JSX/TSX compilation and HMR.

## Phase 10: Cross-platform support (future)

Not started, and not begun until the core Rust host design (Phases 1–2)
is stable — same reasoning as Phase 9. macOS is the primary development
target until then. This phase properly supports Linux (resolving the
devcontainer's unconfirmed rendering — see docs/MANUAL_GUI_CHECK.md) and
adds the `gpui_windows` platform backend for Windows.

## Phase 11: 100% style & Tailwind parity (future)

Not started, and not begun until cross-platform support (Phase 10) is
stable. Closes exactly the gap Phase 7 deferred: full CSS-property parity
in the native style vocabulary/render pipeline (animations/transitions,
transforms, filters, gradients, arbitrary values), state variants mapped
onto GPUI's own interactive element states (hover/focus/active),
responsive breakpoints (no browser viewport concept exists here, so this
needs its own window-size-aware style-resolution design), and dark mode.
The final styling milestone: every Tailwind utility class Vue (and later
React) authors reach for should resolve to a correct native rendering, not
just the common ones Phase 7 covers.

## Phase 12: App-owned Rust extensions (future)

Not started, and not begun until Phase 3.3's packaging and Phase 11's
styling are stable. Every phase before this one assumes app authors write
only JS/TS and consume a prebuilt host binary; this phase adds the opt-in
case where an app moves its own heavy work (compute, native I/O) into Rust
and still ships as a single application:

1. **App-owned host build**: an app that carries its own Rust crate gets a
   host compiled from source with that crate linked in, in place of the
   prebuilt binary — the swappability Phase 3.3 is required to preserve.
   Apps without one keep needing no Rust toolchain.
2. **Extension binding surface**: a stable way for app-owned Rust code to
   register its own functions alongside `__gpjsui_native__` (see
   [docs/FFI.md](./FFI.md#binding-functions)), rather than patching the
   host's own bindings. This is the likely driver for
   `crates/gpjs-ui-macros` (see [docs/STRUCTURE.md](./STRUCTURE.md)).
3. **MSRV verification**: `rust-version` is held equal to the pinned
   toolchain while these crates have no consumers outside this repo. Once
   app crates compile against them it drops to a real floor, checked by its
   own job — see
   [docs/TESTING.md](./TESTING.md#toolchain-pinning-and-msrv).

## Known gaps, not yet scheduled

Recorded so they stay visible. Each needs a decision before it needs a
phase, and none belongs inside one above.

- **Release engineering beyond packaging**: auto-update, code signing and
  notarization. Phase 3.3 produces an application; neither is what keeps it
  running in the field.
- **Crash reporting**: a panic in the host leaves nothing behind for the
  person whose app died. Its cheap half is a `std::panic::set_hook` writing
  through the same reporter an app's own failures already use; a native fault
  under it needs an out-of-process collector, which is separate work.
- **A JS debugger**: QuickJS ships no inspector protocol, and nothing maps a
  running frame back to a `.vue` source line. Today a bundle's failure is a
  message and a stack, and nothing steps through it.
- **The security model for untrusted code**: every binding this roadmap adds
  is reachable by anything in the bundle, dependencies included. Whether that
  is acceptable, and what would constrain it, is unanswered.

## Implementation guidelines

See [AGENTS.md](../AGENTS.md) for the guiding principles (memory safety at the
FFI boundary, keeping the render loop zero-overhead, developer ergonomics)
that apply across every phase.
