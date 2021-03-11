/*
 * The Interrupt Descriptor Table (IDT) is a global table containing an entry for each
 * kind of interrupt that the OS can handle, such as breakpoints, CPU exceptions, etc. 
 * The IDT holds information for how to handle each type of interrupt when it's encountered.
 * 
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
use crate::{println, print}; // our println function defined in lib.rs
use crate::gdt; // Have to load the GDT double fault stack when handling a double fault
use lazy_static::lazy_static; // So the IDT can be loaded and valid for the lifetime of the OS
use pic8259_simple::ChainedPics; // chains primary and secondary PICs together
use spin; // Mutex


/* PICs by default send interrupt vectors in the range [0, 15]; However, this conflicts with the CPU exception interrupt
 * numbers 0-31. Because of this, we should start the PIC interrupts at a different range, which in practice defaults to [32, 47].
 */

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}
impl InterruptIndex {
    // fn as_u8(self) -> u8 {
    //     self as u8
    // }
    fn cast_to_usize(self) -> usize {
        usize::from(self as u8)
    }
}

// `unsafe` because going with the wrong offsets could create undefined behavior
pub static PICS: spin::Mutex<ChainedPics> = 
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) } );

/* We have to use lazy_static here because the IDT is used throughout the life of the program, but is created on the
 * stack. lazy_static allows for a global variable to be created and initialized when it is first used. Alternatively,
 * we could initialize it on the heap, but because we aren't using the stdlib, we don't have a heap yet!
 */
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Set the handler functions
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
              .set_handler_fn(double_fault_handler)
              .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // IST: Interrupt Stack Table
        }
        // We can do this because InterruptDescriptorTable implements IndexMut (https://doc.rust-lang.org/core/ops/trait.IndexMut.html)
        idt[InterruptIndex::Timer.cast_to_usize()].set_handler_fn(timer_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

// https://eli.thegreenplace.net/2011/01/27/how-debuggers-work-part-2-breakpoints
extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/* Double faults occur when an exception is triggered while handling an exception. If another fault occurs in the 
 * double fault handler, then a triple fault occurs, which usually results in a hardware reset.
 */
extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: &mut InterruptStackFrame) -> () {
    print!(".");
    // notify that we're done processing the timer interrupt
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8) };
}

// Tests 
#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
