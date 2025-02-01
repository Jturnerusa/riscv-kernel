macro_rules! kprintln {
    ($fmt:literal, $($e:expr),+) => {
        {
            use ::core::fmt::Write;
            let mut console = crate::sbi::Console;
            writeln!(console, $fmt, $($e),+).unwrap();
        }
    };
    ($s:literal) => {
        {
            use ::core::fmt::Write;
            let mut console = crate::sbi::Console;
            writeln!(console, $s).unwrap();
        }
    };
}
