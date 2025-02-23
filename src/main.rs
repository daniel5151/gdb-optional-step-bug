use std::net::{TcpListener, TcpStream};

use gdbstub::arch::Arch;
use gdbstub::common::Signal;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::{run_blocking, DisconnectReason, GdbStub, SingleThreadStopReason};
use gdbstub::target::Target;

mod emu;

#[cfg(feature = "stub_arm")]
mod gdb_arm;
#[cfg(feature = "stub_mips")]
mod gdb_mips;
#[cfg(feature = "stub_x86")]
mod gdb_x86;
#[cfg(all(
    not(feature = "stub_arm"),
    not(feature = "stub_x86"),
    not(feature = "stub_mips")
))]
compile_error!("must compile with either --feature 'stub_arm' or --feature 'stub_x86' or --feature 'stub_mips'");

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

fn wait_for_tcp(port: u16) -> DynResult<TcpStream> {
    let sockaddr = format!("127.0.0.1:{}", port);
    eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

    let sock = TcpListener::bind(sockaddr)?;
    let (stream, addr) = sock.accept()?;
    eprintln!("Debugger connected from {}", addr);

    Ok(stream)
}

enum EmuGdbEventLoop<U> {
    _Foo(core::marker::PhantomData<U>),
}

#[allow(clippy::type_complexity)]
impl<U> run_blocking::BlockingEventLoop for EmuGdbEventLoop<U>
where
    emu::Emu<U>: Target,
    <emu::Emu<U> as Target>::Arch: Arch<Usize = U>,
{
    type Target = emu::Emu<U>;
    type Connection = Box<dyn ConnectionExt<Error = std::io::Error>>;
    type StopReason = SingleThreadStopReason<<<Self::Target as Target>::Arch as Arch>::Usize>;

    fn wait_for_stop_reason(
        target: &mut emu::Emu<U>,
        conn: &mut Self::Connection,
    ) -> Result<
        run_blocking::Event<Self::StopReason>,
        run_blocking::WaitForStopReasonError<<Self::Target as Target>::Error, std::io::Error>,
    > {
        let poll_incoming_data = || {
            // gdbstub takes ownership of the underlying connection, so the `borrow_conn`
            // method is used to borrow the underlying connection back from the stub to
            // check for incoming data.
            conn.peek().map(|b| b.is_some()).unwrap_or(true)
        };

        match target.run(poll_incoming_data) {
            emu::RunEvent::IncomingData => {
                let byte = conn
                    .read()
                    .map_err(run_blocking::WaitForStopReasonError::Connection)?;
                Ok(run_blocking::Event::IncomingData(byte))
            }
            emu::RunEvent::Event(event) => {
                // translate emulator stop reason into GDB stop reason
                let stop_reason = match event {
                    emu::Event::DoneStep => SingleThreadStopReason::DoneStep,
                    emu::Event::Halted => SingleThreadStopReason::Terminated(Signal::SIGSTOP),
                };

                Ok(run_blocking::Event::TargetStopped(stop_reason))
            }
        }
    }

    fn on_interrupt(
        _target: &mut emu::Emu<U>,
    ) -> Result<Option<Self::StopReason>, <emu::Emu<U> as Target>::Error> {
        // Because this emulator runs as part of the GDB stub loop, there isn't any
        // special action that needs to be taken to interrupt the underlying target. It
        // is implicitly paused whenever the stub isn't within the
        // `wait_for_stop_reason` callback.
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGTRAP)))
    }
}

fn main() -> DynResult<()> {
    pretty_env_logger::init();

    let with_single_step = std::env::args().any(|arg| arg == "--single-step");

    let mut emu = emu::Emu::new(with_single_step)?;

    let connection: Box<dyn ConnectionExt<Error = std::io::Error>> = Box::new(wait_for_tcp(9001)?);

    let gdb = GdbStub::new(connection);

    match gdb.run_blocking::<EmuGdbEventLoop<_>>(&mut emu) {
        Ok(disconnect_reason) => match disconnect_reason {
            DisconnectReason::Disconnect => {
                // run to completion
                while emu.step() != Some(emu::Event::Halted) {}
                println!("Program completed.")
            }
            DisconnectReason::TargetExited(code) => {
                println!("Target exited with code {}!", code)
            }
            DisconnectReason::TargetTerminated(sig) => {
                println!("Target terminated with signal {}!", sig)
            }
            DisconnectReason::Kill => println!("GDB sent a kill command!"),
        },
        Err(e) => {
            println!("gdbstub encountered a fatal error: {}", e)
        }
    }

    Ok(())
}
