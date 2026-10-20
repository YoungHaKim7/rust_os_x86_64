// ============================================================
// print! / println! macros
// ============================================================

use core::fmt::{self, Write};

use crate::serial::SerialWriter;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

pub fn _print(args: fmt::Arguments<'_>) {
    let mut writer = SerialWriter;
    let _ = writer.write_fmt(args);
}
