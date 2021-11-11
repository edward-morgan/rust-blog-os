#![no_std]
#![cfg_attr(test, no_main)] // enable no_main iff the test flag is enabled
#![feature(custom_test_frameworks)] // allow custom test frameworks
#![test_runner(crate::test_runner)] // specify a test runner
#![reexport_test_harness_main = "test_main"] 
#![feature(abi_x86_interrupt)] // Allows us to use the unstable x86-interrupt calling convention

use core::panic::PanicInfo;

pub mod gdt; // Task State Segment (Interrupt Stack Table, https://os.phil-opp.com/double-fault-exceptions/#creating-a-tss)
pub mod serial;
pub mod vga_buffer;
pub mod interrupts; 

/**
 * General initialization function
 */
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    // initialize() is unsafe
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // Actually enable interrupts
}

// Continuously execute `hlt`, which makes the CPU sleep instead of loop (which would peg the CPU)
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/* Helper trait that allows us to simplify tests by automatically printing the 
 * test name/success 
 */
pub trait Testable {
    fn run(&self) -> ();
}
impl<T> Testable for T
where
    T: Fn(), // Defines Testable for any Fn()
{
    fn run (&self) {
        serial_print!("{}...\t", core::any::type_name::<T>()); // core fn that prints type name
        self(); // Run the function embedded in Fn()
        serial_println!("[ok]");
    }
}

/*
 * The `dyn` keyword is syntatic sugar to denote something called a trait object. 
 * Trait objects are used in Rust for 'dynamic dispatch' (run-time polymorphism).
 * Here, we want a slice of Testables that can be called-we don't care about the
 * exact nature of them.
 */
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run(); // Call the Testable wrapper around the Fn()
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
    hlt_loop();
}

// Entrypoint for `cargo xtest`
 #[cfg(test)]
 #[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/* Representing the exit code as an enum allows us type-level control over relatively fine-grained
 * port access. 
 * An exit code is calculated via: (x << 1) | 1. We don't want to use either 0 or 1, because those are codes
 * that QEMU uses.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] // Use a u32 to store each enum value
pub enum QemuExitCode {
    Success = 0x10, // (0001 0000 << 1) | 0001 = 0010 0001 = 33
    Failure = 0x11, // (0001 0001 << 1) | 0001 = 0010 0011 = 35
}

// Lets us exit from QEMU using the I/O port we configured
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    // Access port 0xf4 (configured in Cargo.toml) and write exit_code to it.
    unsafe {
        let mut port = Port::new(0xf4);
        // From Port in port.rs:
        // This function is unsafe because the I/O port could have side effects that violate memory
        // safety.
        port.write(exit_code as u32);
    }
}
