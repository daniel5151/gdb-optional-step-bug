# GDB optional single-step bug repro

This repo exhibits the behavior reported at https://sourceware.org/bugzilla/show_bug.cgi?id=28440

On certain architectures, the GDB client doesn't seem to respect `vCont?` responses that don't include `;s;S`, and will unconditionally send single-step resume packets, even when the target has not explicitly acknowledges support.

This repo includes two basic GDB remote targets, implemented using [`gdbstub`](https://github.com/daniel5151/gdbstub), a Rust library that implements the server-side of the GDB Remote Serial Protocol.

These  are both incredibly barebones "dummy" remote targets, whereby they present a memory space entirely filled with NOP instructions, and use dummy values when reporting register values.

Included are two different stub implementations:

1. An x86_64 stub
2. An armv4t stub

## Running

Running this code requires a relatively recent version of the Rust compiler.

```bash
# run the armv4t stub, with trace logging enabled + single-step support
RUST_LOG=trace cargo run --features 'stub_arm' -- --single-step
# run the armv4t stub, with trace logging enabled, *without* single-step support
RUST_LOG=trace cargo run --features 'stub_arm' --
# run the x86 stub, with trace logging enabled + single-step support
RUST_LOG=trace cargo run --features 'stub_x86' -- --single-step
# run the x86 stub, with trace logging enabled, *without* single-step support
RUST_LOG=trace cargo run --features 'stub_x86' --
```

The GDB client can connect to these targets over TCP loopback. i.e:

```
target remote :9001
```

Once connected, attempt to perform a `stepi`, and observe the result.

## Observations

The armv4t example works as expected: if `--single-step` is not provided, the GDB stub reports `vCont;c;C`, and the GDB client responds with `vCont;c:p1.-1`. If `--single-step` is provided, the GDB stub reports `vCont;c;C;s;S`, and the GDB client reponds with `vCont;s:p1.1;c:p1.-1`.

This matches the spec.

The x86 example does _not_ work as expected. If `--single-step` is not provided, the GDB stub reports `vCont;c;C`, and the GDB client nonetheless reponds with `vCont;s:p1.1;c:p1.-1`! This results in a internal `gdbstub` error, and the example terminates.

## Expected behavior

The GDB client should not send `s` resumption packets on x86 targets, when the `vCont?` response doesn't include support for `s` or `S`.
