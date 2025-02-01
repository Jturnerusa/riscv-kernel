use core::{arch::asm, fmt::Write};

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            asm!(
                "li a7, 0x4442434E",
                "li a6, 0x00",
                "la a2, 0",
                "ecall",
                inout("a0") s.bytes().len() as u64 => _,
                inout("a1") s.as_ptr().addr() as u64 => _,
                out("a2") _,
                out("a6") _,
                out("a7") _
            )
        }

        Ok(())
    }
}
