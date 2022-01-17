use gdbstub::arch::{Arch, SingleStepGdbBehavior};
use gdbstub::common::Signal;
use gdbstub::target;
use gdbstub::target::ext::base::singlethread::{SingleThreadBase, SingleThreadResume};
use gdbstub::target::{Target, TargetResult};

use crate::emu::{Emu, ExecMode};

impl Target for Emu<u64> {
    type Arch = gdbstub_arch::x86::X86_64_SSE;
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
        if !self.with_guard_rail {
            SingleStepGdbBehavior::Optional
        } else {
            Self::Arch::single_step_gdb_behavior()
        }
    }
}

impl SingleThreadResume for Emu<u64> {
    fn resume(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        if signal.is_some() {
            return Err("no support for continuing with signal");
        }

        self.exec_mode = ExecMode::Continue;

        Ok(())
    }

    #[inline(always)]
    fn support_single_step(
        &mut self,
    ) -> Option<target::ext::base::singlethread::SingleThreadSingleStepOps<Self>> {
        if self.with_single_step {
            Some(self)
        } else {
            None
        }
    }
}

impl SingleThreadBase for Emu<u64> {
    fn read_registers(
        &mut self,
        regs: &mut gdbstub_arch::x86::reg::X86_64CoreRegs,
        // regs: &mut gdbstub_arch::arm::reg::ArmCoreRegs,
    ) -> TargetResult<(), Self> {
        log::debug!("read_registers");

        for (i, reg) in regs.regs.iter_mut().enumerate() {
            *reg = i as u64;
        }

        regs.rip = 0x5555_5555_0000_0000;

        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &gdbstub_arch::x86::reg::X86_64CoreRegs,
        // regs: &gdbstub_arch::arm::reg::ArmCoreRegs,
    ) -> TargetResult<(), Self> {
        log::debug!("write_registers: {:#x?}", regs);
        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<(), Self> {
        log::debug!("read_addrs: {:#x?},{}", start_addr, data.len());
        data.fill(0x90); // nop
        Ok(())
    }

    fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self> {
        log::debug!("write_addrs: {:#x?},{:x?}", start_addr, data);
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
        if signal.is_some() {
            return Err("no support for stepping with signal");
        }

        self.exec_mode = ExecMode::Step;

        Ok(())
    }
}
