#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

global_asm!(include_str!("../asm/start.s"));

unsafe extern "C" {
    fn my_write(fd: i32, buf: *const u8, len: usize) -> isize;
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let message = b"Hello world!\n";

    unsafe {
        let _ = my_write(1, message.as_ptr(), message.len());
    }

    0

    // let message = b"Hello world!\n";
    // let result = unsafe {
    //     linux_syscall(
    //         1, // SYS_write
    //         1, // stdout
    //         message.as_ptr(),
    //         message.len(),
    //     )
    // };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
