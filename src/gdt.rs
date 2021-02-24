use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::gdt::SegmentSelector;
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0; // Use the first stack for Double Faults

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    GDT.0.load();
    unsafe { // Unsafe because it could load bad selectors
    // Use the code and tss selector entries to load
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

lazy_static! {
    /* On x86_64, the TSS doesn't really hold any task information. However, it does hold the Interrupt Stack Table (IST)
     * and the Privilege Stack Table (used for privilege level changes).
     */
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // Set the Double Fault IST entry
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // TODO: There is no guard page underneath this stack, so don't do anything that could overflow it.
            const STACK_SIZE: usize = 4096 * 5;
            // Populate it with all zeroes
            // Why `mut`? Well, if we make it immutable then the bootloader will map this stack to a read-only page.
            // TODO: Why does that matter?
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Why unsafe? Well, we're working with a static mut, which can't be guaranteed to be race-free. 
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK} );
            let stack_end = stack_start + STACK_SIZE;
            // Since stacks grow downwards, return the low address (stack_end) 
            stack_end
        };
        tss
    };
}

/* What is the GDT?
 * The Global Descriptor Table is a construct used by x86 to configure `segmented virtual memory.`
 * Segmented Virtual Memory is a memory management technique (like paging) that divides physical memory into 
 * a series of segments. The main difference between segmentation and paging is that segments are not of fixed
 * sizes, while pages are. This can lead to less fragmentation with segmentation.
 * 
 * Even though memory segmentation is obsolete, since x86 retains backwards compatibility you have to set up
 * basic segmentation, even before paging. 
 * 
 * In 64-bit mode, the GDT is mostly used for 1) switching between user- and kernel-space, and 2) loading a TSS.
 */
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // Create code and tss selector entries in the GDT, then return them as part of the static
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}