#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rust_os::{QemuExitCode, exit_qemu, serial_print, serial_println};

fn should_fail() {
    // Since we're not using Testable, we need to print it explicitly
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {} // Never gets here
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failure);

    loop {}
}
