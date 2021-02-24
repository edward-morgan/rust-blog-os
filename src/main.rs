#![no_std] // Don't use the standard library (Freestanding binary)
#![no_main] // Disable rust entrypoints
//TODO: As of May 2020, asm is depracated in favor of llvm_asm. rls is not currently working on `nightly` (last working build was 05-15)
#![feature(asm)] 
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::println; // our println function defined in lib.rs

////////////////////////////////// Main ////////////////////////////////// 

// Don't mangle the start function name or it won't be recognized
#[no_mangle] 
pub extern "C" fn _start() -> ! { // Should be divergent
    println!("Hello World{}", "!");

    rust_os::init();

    // Double fault: Writing outside of memory (page fault) with no page fault handler
    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // }

    // Kernel stack overflow (pushing return address too many times)
    // fn stack_overflow() {
    //    stack_overflow();
    // }
    // stack_overflow();

    // Breakpoint fault
    // x86_64::instructions::interrupts::int3();

    /* the 'cfg' attribute allows for conditional compilation-a certain function can only be called
     * if the compiler receives a certain flag. In this case, the 'test' flag has to be passed in
     * order for the tests to be run.
     */
    #[cfg(test)]
    test_main();
    println!("Didn't crash after running test_main.");

    loop {} // -> !
}

////////////////////////////////// Panics ////////////////////////////////// 

// General panic handler for normal (non-test) runs
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { // Should never return
    println!("{}", _info);
    loop {}
}
// Alternate panic handler for testing (prints to serial, not vga)
#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { // Should never return
    rust_os::test_panic_handler(_info)
}

