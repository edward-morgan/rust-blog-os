#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
/* Specify "test_main" as the test runner function
 * This is needed because otherwise, the test runner would automatically be called main(),
 * which isn't called here because of the no_main attribute. 
 */
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println;

#[test_case]
fn test_println_basic() -> () {
    println!("Test basic println");
}

#[test_case]
fn test_println_500x() -> () {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts( || {
        for _ in 0..500 {
            println!("Test println");
        }
    });
}

#[no_mangle] // Don't mangle the entrypoint of this integration test
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info);
}
