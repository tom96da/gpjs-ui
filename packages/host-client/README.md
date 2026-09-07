# @gpjs-ui/host-client

The Node end of gpjs-ui's dev protocol: resolves and spawns `gpjs-ui-host
--dev <bundle>`, and speaks the newline-delimited JSON-RPC 2.0 channel it
answers on. `@gpjs-ui/cli` drives this package rather than the host
directly, so it is the only package that knows the wire format.

`HostClient` correlates each request it sends with the response that
answers it, relays the host's stderr, and hands every notification it
receives (`ready`, `appError`, and any a future bundler integration adds) to
a callback the caller supplies — this package assigns no special meaning to
any particular method name itself.

A line the host writes that doesn't parse as a JSON-RPC message is treated
as stray output from a dependency, not a protocol violation: it's logged
and the channel keeps reading.

## Host binary resolution

`GPJS_UI_HOST_BIN`, if set, names the binary to spawn outright. Otherwise
this package resolves it relative to its own install location, which today
only ever resolves inside this workspace's own Cargo build output — a
per-platform npm-distributed binary is a later addition, not yet wired in.
