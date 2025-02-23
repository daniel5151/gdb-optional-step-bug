use crate::DynResult;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    DoneStep,
    Halted,
}

pub enum ExecMode {
    Step,
    Continue,
}

pub struct Emu<U> {
    _usize: core::marker::PhantomData<U>,

    pub(crate) with_single_step: bool,

    pub(crate) exec_mode: ExecMode,
}

impl<U> Emu<U> {
    pub fn new(with_single_step: bool) -> DynResult<Emu<U>> {
        Ok(Emu {
            _usize: core::marker::PhantomData,

            with_single_step,
            exec_mode: ExecMode::Continue,
        })
    }

    /// single-step the interpreter
    pub fn step(&mut self) -> Option<Event> {
        Some(Event::DoneStep)
    }

    /// run the emulator in accordance with the currently set `ExecutionMode`.
    ///
    /// since the emulator runs in the same thread as the GDB loop, the emulator
    /// will use the provided callback to poll the connection for incoming data
    /// every 1024 steps.
    pub fn run(&mut self, mut poll_incoming_data: impl FnMut() -> bool) -> RunEvent {
        match self.exec_mode {
            ExecMode::Step => RunEvent::Event(self.step().unwrap_or(Event::DoneStep)),
            ExecMode::Continue => {
                let mut cycles = 0;
                loop {
                    if cycles % 1024 == 0 {
                        // poll for incoming data
                        if poll_incoming_data() {
                            break RunEvent::IncomingData;
                        }
                    }
                    cycles += 1;

                    if let Some(event) = self.step() {
                        break RunEvent::Event(event);
                    };
                }
            }
        }
    }
}

pub enum RunEvent {
    IncomingData,
    Event(Event),
}
