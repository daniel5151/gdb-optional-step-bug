use core::num::NonZeroUsize;
use gdbstub::arch::{Arch, RegId, Registers, SingleStepGdbBehavior};
use gdbstub::common::Signal;
use gdbstub::target;
use gdbstub::target::ext::base::singlethread::{SingleThreadBase, SingleThreadResume};
use gdbstub::target::{Target, TargetResult};

use crate::emu::{Emu, ExecMode};

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct GenericRegId(pub usize);

impl RegId for GenericRegId {
    fn from_raw_id(id: usize) -> Option<(Self, Option<NonZeroUsize>)> {
        Some((GenericRegId(id), None))
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct GenericRegs {
    pub dummy: u64,
}

impl Registers for GenericRegs {
    type ProgramCounter = u64;

    fn pc(&self) -> Self::ProgramCounter {
        0
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        for byte in self.dummy.to_le_bytes() {
            write_byte(Some(byte))
        }
    }

    fn gdb_deserialize(&mut self, _bytes: &[u8]) -> Result<(), ()> {
        Ok(())
    }
}

pub struct GenericArch {}

impl Arch for GenericArch {
    type Usize = u64;
    type Registers = GenericRegs;
    type RegId = GenericRegId;
    type BreakpointKind = usize;
}

impl Target for Emu<u64> {
    type Arch = GenericArch;
    type Error = &'static str;

    #[inline(always)]
    fn base_ops(&mut self) -> target::ext::base::BaseOps<Self::Arch, Self::Error> {
        target::ext::base::BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn guard_rail_implicit_sw_breakpoints(&self) -> bool {
        true
    }

    #[inline(always)]
    fn guard_rail_single_step_gdb_behavior(&self) -> SingleStepGdbBehavior {
        SingleStepGdbBehavior::Optional
    }
}

impl SingleThreadResume for Emu<u64> {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline(always)]
    fn support_single_step(
        &mut self,
    ) -> Option<target::ext::base::singlethread::SingleThreadSingleStepOps<Self>> {
        Some(self)
    }
}

impl SingleThreadBase for Emu<u64> {
    fn read_registers(&mut self, _regs: &mut GenericRegs) -> TargetResult<(), Self> {
        log::debug!("read_registers");
        Ok(())
    }

    fn write_registers(&mut self, _regs: &GenericRegs) -> TargetResult<(), Self> {
        Ok(())
    }

    #[inline(always)]
    fn support_single_register_access(
        &mut self,
    ) -> Option<target::ext::base::single_register_access::SingleRegisterAccessOps<(), Self>> {
        Some(self)
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<(), Self> {
        log::debug!("read_addrs: {:#x?},{}", start_addr, data.len());
        data.fill(0x00); // nop
        Ok(())
    }

    fn write_addrs(&mut self, _start_addr: u64, _data: &[u8]) -> TargetResult<(), Self> {
        println!("SingleStepGdbBehavior should be set to Ignored on this architecture!");
        Ok(())
    }

    #[inline(always)]
    fn support_resume(
        &mut self,
    ) -> Option<target::ext::base::singlethread::SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::base::singlethread::SingleThreadSingleStep for Emu<u64> {
    fn step(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        println!("SingleStepGdbBehavior should be set to Required on this architecture!");
        if signal.is_some() {
            return Err("no support for stepping with signal");
        }

        self.exec_mode = ExecMode::Step;

        Ok(())
    }
}

impl target::ext::base::single_register_access::SingleRegisterAccess<()> for Emu<u64> {
    fn read_register(
        &mut self,
        _tid: (),
        _reg_id: GenericRegId,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        buf[0] = 0;
        buf[1] = 0;
        buf[2] = 0;
        buf[3] = 0;
        Ok(4)
    }

    fn write_register(
        &mut self,
        _tid: (),
        _reg_id: GenericRegId,
        _val: &[u8],
    ) -> TargetResult<(), Self> {
        Ok(())
    }
}
