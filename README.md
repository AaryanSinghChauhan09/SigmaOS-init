# SigmaOS Init

An evolving Rust library for system initialization and service/session models. It exposes `SigmaInit` through the `InitSystem` alias and includes service state and restart-policy types, systemd-style unit parsing and service structures, login session models, and timesyncd configuration.

The crate's APIs do not by themselves establish that it runs as PID 1, supervises real processes, mounts filesystems, or provides full systemd compatibility. Verify platform integration before using it to boot or manage a machine.

## Build

The manifest references sibling [`SigmaOS-kernel`](../SigmaOS-kernel). From the SigmaOS checkout:

```sh
cargo check --manifest-path SigmaOS-init/Cargo.toml --lib
```

## License

GPL-3.0-or-later. See [`Cargo.toml`](Cargo.toml).
