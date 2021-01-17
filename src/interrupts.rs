/*
 * The Interrupt Descriptor Table (IDT) is a global table containing an entry for each
 * kind of interrupt that the OS can handle, such as breakpoints, CPU exceptions, etc. 
 * The IDT holds information for how to handle each type of interrupt when it's encountered.
 * Interrupts are handled differently from normal function calls, in that they have addtl
 * information stored on the stack besides return info and caller-saved registers:
 * 
 * _________________________<- old stack pointer (before exception)
 * | Stack Segment (ss)    |
 * | Stack Pointer (rsp)   |
 * | RFLAGS Register       |
 * | Code Segment (cs)     |
 * | Instr Pointer (rsp)   |
 * | Error Code (optional) |
 * |_______________________|<- new stack pointer 
 *   Stack frame of handler
 *   ...
 * 
 * Because of this, interrupts have to adhere to a separate calling convention from the
 * 'traditional' C calling convention: `x86-interrupt`.
*/
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println; // our println function defined in lib.rs
use lazy_static::lazy_static; // So the IDT can be loaded and valid for the lifetime of the OS

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Set the handler functions
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
