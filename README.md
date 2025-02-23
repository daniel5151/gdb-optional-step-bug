# GDB optional single-step bug repro

This repo exhibits the behavior reported at https://sourceware.org/bugzilla/show_bug.cgi?id=28440

## The Bug

On certain architectures, the GDB client doesn't respect `vCont?` responses that don't report support for single-stepping (i.e: via `;s;S`), and will _unconditionally_ send single-step resume packets. This can lead to very Bad Things happening when targets encounter unexpected single-step packets, ranging from silently ignoring the request, to failing to parse the `vCont` packet the request is encapsulated in.

## This Repo

This repo includes a few basic GDB remote targets, implemented using [`gdbstub`](https://github.com/daniel5151/gdbstub), a Rust library that implements the server-side of the GDB Remote Serial Protocol.

These are incredibly barebones "dummy" remote targets, whereby they present a memory space entirely filled with `NOP` instructions, and use dummy values when reporting register values.

## Running

Use the provided `check_vcont.sh` script to verify how `vCont?` is handled by
your version of GDB (use `$GDB` to configure the path):

```
$ GDB=gdb-multiarch ./check_vcont.sh armv4t mips i386:x86-64
GNU gdb (GDB) 15.2
 ===================== [armv4t] =====================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C#26
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;c:p1.-1#0f

=================== [armv4t (SS)] ===================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C;s;S#62
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7

 ===================== [mips] =====================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C#26
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;c:p1.-1#0f

=================== [mips (SS)] ===================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C;s;S#62
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;c:p1.-1#0f

 ===================== [i386:x86-64] =====================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C#26
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7
 ERROR gdbstub::stub::core_impl::resume   > GDB client sent resume action not reported by `vCont?`

=================== [i386:x86-64 (SS)] ===================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C;s;S#62
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7
```

### Running Manually

Individual architectures can be tested through `cargo run` by setting the GDB architecture:

- `armv4t`: ARMv4T
- `mips`: MIPS
- `i386:x86-64`: X86_64

By passing the `--single-step` flag, support can be added to the target. For
example:

```bash
# enable trace logs for the `gdbstub` library, dumping send/recv'd packets to stderr
export RUST_LOG=trace

# run the stub with trace logging enabled + single-step support
cargo run -- --single-step
# run the stub with trace logging enabled, *without* single-step support
cargo run --
```

The GDB client can connect to these targets over TCP loopback. i.e:

```
(gdb) set architecture armv4t
(gdb) target remote :9001
```

See the `try_stepi.gdb` script for more commands.

## Observations

|        | `--single-step` (`vCont;c;C;s;S`) | (no support) (`vCont;c;C`) |
|:------:|:---------------------------------:|:--------------------------:|
| `armv4t` |          `vCont;c:p1.-1`          |   `vCont;s:p1.1;c:p1.-1`   |
| `mips` |          `vCont;c:p1.-1`          |       `vCont;c:p1.-1`      |
| `i386:x86-64` |       `vCont;s:p1.1;c:p1.-1`      |   `vCont;s:p1.1;c:p1.-1`   |

- The armv4t example works as expected.
  - If `--single-step` is not provided, the GDB stub reports `vCont;c;C`, and the GDB client responds with `vCont;c:p1.-1`. If `--single-step` is provided, the GDB stub reports `vCont;c;C;s;S`, and the GDB client respond with `vCont;s:p1.1;c:p1.-1`.
  - This matches the spec.
- The x86 example does _not_ work as expected.
  - If `--single-step` is not provided, the GDB stub reports `vCont;c;C`, and the GDB client nonetheless respond with `vCont;s:p1.1;c:p1.-1`! This results in a internal `gdbstub` error, and the example terminates.
- The MIPS example is interesting.
  - Regardless if `--single-step` was provided, the GDB client will _never_ send a `vCont;s:pX.X` packet!
  - While this isn't strictly an "error", it is nonetheless weird that the GDB client doesn't attempt to use the target's "native" single step feature.

## Expected behaviors

- The GDB client should not send `s` resumption packets on x86 targets, when the `vCont?` response doesn't include support for `s`.
- The GDB client should use the `s` resumption packet when `vCont?` includes support for `s`
