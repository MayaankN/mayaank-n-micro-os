# Kernel Debugging Log

**Project Name:** Mayaank N – Micro-OS  
**Student Name:** Mayaank N  
**USN:** 25BTRCL148  
**Specialization:** Scheduler Engineering – Round Robin with Dynamic Quantum  

---

## Incident 1: Nightly Rust Naked Function Attribute Syntax
- **Symptom:** Compiler error `error: unsafe attribute used without unsafe` on `#[naked]`.
- **Root Cause:** In modern Rust nightly (1.88+), naked assembly functions require the explicit `#[unsafe(naked)]` syntax to emphasize safety boundaries.
- **Resolution:** Replaced all occurrences of `#[naked]` with `#[unsafe(naked)]` in `switch_context` and exception interrupt stubs.

---

## Incident 2: Raw Pointers in Global Static Limine Requests
- **Symptom:** Compiler error `error[E0277]: *const LimineHhdmResponse cannot be shared between threads safely`.
- **Root Cause:** Rust statics require the `Sync` trait. Structs containing raw pointers (`*const T`) do not implement `Sync` by default.
- **Resolution:** Implemented `unsafe impl Sync for LimineHhdmRequest {}`, `LimineFramebufferRequest`, and `LimineMemmapRequest`. Because these static requests are only modified by Limine at boot time and read immutably by the kernel, the manual `Sync` implementation is sound.

---

## Incident 3: General Protection Fault (0x0d) with Error Code 0x0008 on Timer Interrupt
- **Symptom:** Live QEMU execution triple-faulted immediately upon enabling CPU interrupts (`sti`). QEMU interrupt trace reported:
  `check_exception old: 0xffffffff new 0xd`
  `1: v=0d e=0008 i=0 cpl=0 IP=0028:ffffffff80001de6 pc=ffffffff80001de6 SP=0030:ffffffff80013f88`
- **Root Cause Analysis:** Error code `0x0008` denotes segment selector `0x08` (index 1 in GDT). The IDT gate descriptor was initialized with a hardcoded `gdt_selector = 0x08`. However, Limine's active 64-bit kernel code segment selector is `0x28` (`CS = 0028`). When the timer interrupt fired, the CPU attempted to load selector `0x08`, triggered a GPF (`0x0d`), and when the GPF handler also referenced selector `0x08`, it double-faulted and triple-faulted.
- **Resolution:** Added a dynamic CS register query function:
  ```rust
  #[inline]
  fn read_cs() -> u16 {
      let cs: u16;
      unsafe { asm!("mov {0:x}, cs", out(reg) cs); }
      cs
  }
  ```
  Passes the live active `cs` selector (`0x28`) to all IDT gate initializations, ensuring compatibility with any bootloader environment.

---

## Incident 4: Context Switch Stack Drift
- **Symptom:** During initial multi-task switching, Task 1 yielded into Task 2, and Task 2 into Task 3, but switching back into Task 1 caused execution to stall without returning to the task loop.
- **Root Cause Analysis:** In the initial context switch implementation, `switch_context` pushed the saved `new.rip` onto `new.rsp` before executing `ret`. However, since `call switch_context` had already placed the return address onto the stack, pushing it a second time caused the stack pointer `rsp` to drift by 8 bytes on every switch. Upon subsequent returns, the function popped invalid values as return addresses.
- **Resolution:** Adopted the canonical Unix/Linux kernel stack switching pattern:
  - Callee-saved registers (`rbp`, `rbx`, `r12`, `r13`, `r14`, `r15`) are pushed directly onto the task's own stack.
  - The stack pointer `rsp` is saved to `[rdi]`, and loaded from `[rsi]`.
  - Callee-saved registers are popped from the target stack, and `ret` pops the original return address.
  - Initial task stacks are pre-seeded with this exact 7-register frame (`rbp..r15` + `entry_point`).
  - Result: 9,200+ clean context switches without any stack leakage or corruption.

---

## Incident 5: Dynamic Quantum Integer Truncation
- **Symptom:** In unit test `test_dynamic_quantum_cpu_bound_expansion`, a task executing bursts of 5 ticks did not expand its quantum.
- **Root Cause Analysis:** Moving average calculation used integer truncation `sum / samples`. With 4 samples of `[4, 5, 5, 5]`, `sum = 19`, and `19 / 4 = 4` due to truncation of `.75`. The algorithm evaluated `4 < 5`, treating the workload as non-exhausting.
- **Resolution:** Replaced truncation with rounded integer division `(sum + samples / 2) / samples`, correctly computing `(19 + 2) / 4 = 5` and triggering proper quantum expansion.
