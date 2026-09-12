# @gpjs-ui/host-darwin-x64

Prebuilt `gpjs-ui-host` binary for macOS on Intel (x64). Not meant to be
depended on directly — `@gpjs-ui/host-client` installs whichever of these
matches the current OS/arch as an `optionalDependency` and resolves the
binary inside it.

This package carries no JS: `bin/gpjs-ui-host` is the whole of it.
