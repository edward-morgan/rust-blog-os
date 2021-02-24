# OSTEP Book
www.ostep.org

## Chapter 13: Address Spaces

Address spaces are needed because it would be difficult to only let one process use all of memory at one time. Instead, we should let multiple processes use physical memory at the same time. 

The next problem is that it would be difficult for program designers/compilers to function if they didn't know *where* in memory they would be located. No process would know what the start of memory would be, since they could be located anywhere in memory.

Additionally, security is a problem. If a process can use memory that another process has been using, then what's stopping it from accessing that memory? 

The answer to this is **memory virtualization**, which maps a limited set of physical addresses to a (usually much larger) set of virtual addresses. 

### Three goals of memory virtualization
1. *Transparency*: The operating system should implement memory virtualization in a way that is invisible to any processes using it. 
2. *Efficiency*: In order to be usable for multiple processes, virtual address translation should be as fast as possible and should use as little memory as possible. In practice, this requires hardware-level support (with **Translation Lookaside Buffers**)
3. *Protection*: Processes must be protected from each other. This means that they shouldn't be affected if either fails, nor should they be able to access or manipulate each other's address space.

> Note: **Any** address you see as a user-level process is a virtual address. Only the OS can see true physical addresses.

---

## Chapter 15: Address Translation

Virtualizing the CPU relies on a principle called **Limited Direct Execution**. Essentially, this means that we should let the program run directly on the hardware. However, when certain events occur (for example, system calls or interrupts), make the OS take over to make sure the right thing happens. This process is referred to as **interposition**.

The efficiency of memory address translation depends on hardware implementations of the actual translation. While the OS must get involved to initialize and manage memory, the system calls to obtain and free memory must be done by the hardware.

### Software-based Address Relocation

Prior to hardware support, many systems used purely software methods for address translation, referred to as **static translation**. This method uses a piece of software called a **loader**, which puts an executable to be run into memory and rewrites its addresses based on the offsets. 

Problems:
- There is no memory protection; an executable could write to or read from any location in memory.
- It is difficult to relocate a running executable to another location in memory if needed.

### Dynamic Address Relocation

The simplest method by which you can translate addresses consists of two special registers attached to a CPU. First, a *base* register contains the physical address which should map to *virtual* address `0x0`. There also exists a *bounds* (or *limit*) register that contains the maximum bounds of memory the process can use. Alternatively, it can contain the maximum physical address that can be written to. Both methods are fairly equivalent.

The fact that this technique is so simple gives it a few advantages:
1. It's easy to implement and execute.
2. Memory relocation happens at runtime.
3. The physical memory addresses can change while the program is running.

---

## Chapter 16: Segmentation

---

## Chapter 18: Introduction to Paging

There are two general solutions to the problem of space management. The first is to split up space according to how much space is needed, resulting in variably-sized pieces, which is called **segmentation**. While this method has some advantages, it most notably causes **fragmentation**, when the total available space is much larger than the largest contiguous block of available space. The second approach is called **paging**, and it restricts blocks of memory to fixed-size chunks. While this can cause memory to be wasted (ex. in most programming languages a `Boolean` is stored in an 8 bit byte even though it only needs 1), it makes the problem of space allocation considerably easier. 

Paging in the context of physical memory involves binning-off chunks of memory and mapping virtual address spaces to those physical pages. For example, here is a representation of paged physical memory with an address space inside it:
```
|-----------------|--------------------------|
| Page 0          | Reserved for OS          |
| Page 1          | Page 2                   |
| Page 2          | (unused)                 |
| Page 3          | Page 1                   |
| Page 4          | (unused)                 |
| Page 5          | Page 3                   |
|-----------------|--------------------------|
| Physical memory | Address space of program |
```
Paging allows for two advantages:
1. *Flexibility*: Paging physical memory removes the need for making assumptions about the behavior of program memory. If we instead used segmentation for physical memory, then we would have to reason about how items in memory like the stack or heap would grow.
2. *