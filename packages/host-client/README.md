# @gpjs-ui/host-client

The Node end of gpjs-ui's dev protocol: resolves and spawns `gpjs-ui-host
--dev <bundle>`, and speaks the newline-delimited JSON-RPC 2.0 channel it
answers on. `@gpjs-ui/cli` drives this package rather than the host
directly, so it is the only package that knows the wire format.

`HostClient` correlates each request it sends with the response that
answers it and relays the host's stderr. Two notification methods carry
meaning of their own: `ready`, whose `params.protocol` it checks against
the revision it was built for, reporting and terminating the child on any
mismatch; and `appError`, an app fault the host caught and kept rendering
past, handed to its own callback rather than a generic one. Any other
method name is routed, `params` untouched and unparsed, to whichever
integration the caller registered for it — how a future bundler
integration (e.g. Vite) rides this channel.

A line the host writes that doesn't parse as a JSON-RPC message is treated
as stray output from a dependency, not a protocol violation: it's logged
and the channel keeps reading.

## Host binary resolution

`resolveHostBin` tries, in order:

1. `GPJS_UI_HOST_BIN`, if set — names the binary to spawn outright.
2. The `optionalDependency` matching this OS/arch
   (`@gpjs-ui/host-darwin-arm64`, `-darwin-x64`, `-linux-arm64`,
   `-linux-x64` — Windows isn't supported yet).
3. This workspace's own Cargo build output, at `target/debug/gpjs-ui-host`
   by default — pass `{ profile: "release" }` to resolve the release build
   instead.
