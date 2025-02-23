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
$ GDB=gdb-multiarch ./check_vcont.sh armv4t mips i386:x86-64 aarch64
GNU gdb (GDB) 15.2 for x86_64-pc-linux-gnu
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

 ===================== [aarch64] =====================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C#26
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7
 ERROR gdbstub::stub::core_impl::resume   > GDB client sent resume action not reported by `vCont?`

=================== [aarch64 (SS)] ===================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C;s;S#62
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7
```

However, when using a GDB version compiled for bare-metal targets:

```
$ GDB=arm-none-eabi-gdb ./check_vcont.sh armv4t mips
GNU gdb (GDB) 15.2 for arm-none-eabi
 ===================== [armv4t] =====================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C#26
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7
 ERROR gdbstub::stub::core_impl::resume   > GDB client sent resume action not reported by `vCont?`

=================== [armv4t (SS)] ===================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C;s;S#62
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7

 ===================== [mips] =====================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C#26
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7
 ERROR gdbstub::stub::core_impl::resume   > GDB client sent resume action not reported by `vCont?`

=================== [mips (SS)] ===================
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont?#49
 TRACE gdbstub::protocol::response_writer > --> $vCont;c;C;s;S#62
 TRACE gdbstub::protocol::recv_packet     > <-- $vCont;s:p1.1;c:p1.-1#f7
```

### Running Manually

You can test any architecture supported by GDB's `set architecture` command when connecting to the stub, such as:

- `armv4t`: ARMv4T
- `mips`: MIPS
- `i386:x86-64`: X86_64
- `aarch64`: AArch64

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

When using GDB for x86_64-pc-linux-gnu:

|        | `--single-step` (`vCont;c;C;s;S`) | (no support) (`vCont;c;C`) |
|:------:|:---------------------------------:|:--------------------------:|
| `armv4t` |          `vCont;c:p1.-1`          |   `vCont;s:p1.1;c:p1.-1`   |
| `mips` |          `vCont;c:p1.-1`          |       `vCont;c:p1.-1`      |
| `i386:x86-64` |       `vCont;s:p1.1;c:p1.-1`      |   `vCont;s:p1.1;c:p1.-1`   |
| `aarch64` |       `vCont;s:p1.1;c:p1.-1`      |   `vCont;s:p1.1;c:p1.-1`   |

When using GDB for bare-metal targets (e.g. arm-none-eabi-gdb):

|        | `--single-step` (`vCont;c;C;s;S`) | (no support) (`vCont;c;C`) |
|:------:|:---------------------------------:|:--------------------------:|
| `armv4t` |          `vCont;s:p1.1;c:p1.-1`          |   `vCont;s:p1.1;c:p1.-1`   |
| `mips` |          `vCont;s:p1.1;c:p1.-1`          |   `vCont;s:p1.1;c:p1.-1`   |

- The armv4t example works as expected.
  - If `--single-step` is not provided, the GDB stub reports `vCont;c;C`, and the GDB client responds with `vCont;c:p1.-1`. If `--single-step` is provided, the GDB stub reports `vCont;c;C;s;S`, and the GDB client responds with with `vCont;s:p1.1;c:p1.-1`.
  - This matches the spec.
- Both x86_64 and aarch64 examples do _not_ work as expected.
  - If `--single-step` is not provided, the GDB stub reports `vCont;c;C`, and the GDB client nonetheless responds with `vCont;s:p1.1;c:p1.-1`! This results in an internal `gdbstub` error, and the example terminates.
  - If `--single-step` is provided, both work correctly by sending `vCont;s:p1.1;c:p1.-1`.
- The MIPS example is interesting.
  - Regardless if `--single-step` was provided, the GDB client will _never_ send a `vCont;s:pX.X` packet!
  - While this isn't strictly an "error", it isn't nonetheless weird that the GDB client doesn't attempt to use the target's "native" single step feature.
- However, when using bare-metal GDB versions (e.g. arm-none-eabi-gdb):
  - Both armv4t and MIPS targets unconditionally receive single-step packets (`vCont;s:p1.1;c:p1.-1`), regardless of their reported capabilities.
  - This suggests the issue may be more prevalent in bare-metal GDB variants.

## Expected behaviors

- The GDB client should not send `s` resumption packets when the `vCont?` response doesn't include support for `s`, regardless of architecture or GDB variant
- The GDB client should use the `s` resumption packet when `vCont?` includes support for `s`
