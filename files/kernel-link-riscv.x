/* # Developer notes

- Symbols that start with a double underscore (__) are considered "private"

- Symbols that start with a single underscore (_) are considered "semi-public"; they can be
  overridden in a user linker script, but should not be referred from user code (e.g. `extern "C" {
  static mut _heap_size }`).

- `EXTERN` forces the linker to keep a symbol in the final binary. We use this to make sure a
  symbol is not dropped if it appears in or near the front of the linker arguments and "it's not
  needed" by any of the preceding objects (linker arguments)

- `PROVIDE` is used to provide default values that can be overridden by a user linker script

- `${ARCH_WIDTH}` is replaced by `32` (TODO: why not 4 ??)
- `${INCLUDE_LINKER_FILES}` is replaced by contents of `exceptions.x`, `interrupts.x` 
  and added `INCLUDE device.x` from riscv-rt crate.
    > This is similar to `cargo add riscv-rt` and `cargo add rp235x-pac -F rt`

- REGION_ALIAS are replaced in this file as mentioned below.
    - REGION_ALIAS("REGION_TEXT", FLASH);
    - REGION_ALIAS("REGION_RODATA", FLASH);
    - REGION_ALIAS("REGION_DATA", RAM);
    - REGION_ALIAS("REGION_BSS", RAM);
    - REGION_ALIAS("REGION_HEAP", RAM);
    - REGION_ALIAS("REGION_STACK", RAM);

- On alignment: it's important for correctness that the VMA boundaries of both .bss and .data *and*
  the LMA of .data are all `32`-byte aligned. These alignments are assumed by the RAM
  initialization routine. There's also a second benefit: `32`-byte aligned boundaries
  means that you won't see "Address (..) is out of bounds" in the disassembly produced by `objdump`.
*/

/* Memory layout defined externally (e.g. memory.x) */
INCLUDE memory.x

/* # Entry point: RP2350 Datasheet, Section 5.9.5.2. Minimum RISC-V IMAGE_DEF
Bootrom will enter the binary at its lowest address,
which is the default behaviour on RISC-V. This default entry point can be overridden by a
`PICOBIN_BLOCK_ITEM_1BS_ENTRY_POINT` item. Note that `PICOBIN_BLOCK_ITEM_1BS_VECTOR_TABLE` is not valid on RISC-V, 
as unlike Cortex-M the RISC-V vector table does not define the program entry point.
*/

/* Default abort entry point. If no abort symbol is provided, then abort maps to _default_abort. */
EXTERN(_default_abort);
PROVIDE(abort = _default_abort);

/* Trap for exceptions triggered during initialization. If the execution reaches this point, it
   means that there is a bug in the boot code. If no _pre_init_trap symbol is provided, then
  _pre_init_trap defaults to _default_abort. Note that _pre_init_trap must be 4-byte aligned */
PROVIDE(_pre_init_trap = _default_abort);

/* Multi-processor hook function (for multi-core targets only). If no _mp_hook symbol
   is provided, then _mp_hook maps to _default_mp_hook, which leaves HART 0 running while
   the other HARTS stuck in a busy loop. Note that _default_mp_hook cannot be overwritten.
   We use PROVIDE to avoid compilation errors in single hart targets, not to allow users
   to overwrite the symbol. */
PROVIDE(_default_mp_hook = abort);
PROVIDE(_mp_hook = _default_mp_hook);

/* Default trap entry point. If not _start_trap symbol is provided, then _start_trap maps to
   _default_start_trap, which saves caller saved registers, calls _start_trap_rust, restores
   caller saved registers and then returns. Note that _start_trap must be 4-byte aligned */
EXTERN(_default_start_trap);
PROVIDE(_start_trap = _default_start_trap);

/* Default interrupt setup entry point. If not _setup_interrupts symbol is provided, then
   _setup_interrupts maps to _default_setup_interrupts, which in direct mode sets the value
   of the xtvec register to _start_trap and, in vectored mode, sets its value to
   _vector_table and enables vectored mode. */
EXTERN(_default_setup_interrupts);
PROVIDE(_setup_interrupts = _default_setup_interrupts);

/* Default main routine. If no hal_main symbol is provided, then hal_main maps to main, which
   is usually defined by final users via the #[riscv_rt::entry] attribute. Using hal_main
   instead of main directly allow HALs to inject code before jumping to user main. */
PROVIDE(hal_main = main);

/* Default exception handler. By default, the exception handler is abort.
   Users can override this alias by defining the symbol themselves */
PROVIDE(ExceptionHandler = abort);

/* Default interrupt handler. By default, the interrupt handler is abort.
   Users can override this alias by defining the symbol themselves */
PROVIDE(DefaultHandler = abort);

/* Default interrupt trap entry point. When vectored trap mode is enabled,
   the riscv-rt crate provides an implementation of this function, which saves caller saved
   registers, calls the the DefaultHandler ISR, restores caller saved registers and returns.
   Note, however, that this provided implementation cannot be overwritten. We use PROVIDE
   to avoid compilation errors in direct mode, not to allow users to overwrite the symbol. */
PROVIDE(_start_DefaultHandler_trap = _start_trap);
PROVIDE(_max_hart_id = 0); /* TODO: Should be 1 for dual core hazard3 present in pico 2(w) */
PROVIDE(_hart_stack_size = SIZEOF(.stack) / (_max_hart_id + 1));
PROVIDE(_heap_size = 0);

/* $$$$> START: Contents from ${INCLUDE_LINKER_FILES} <$$$$ */

/* # EXCEPTION HANDLERS DESCRIBED IN THE STANDARD RISC-V ISA
   
   If the `no-exceptions` feature is DISABLED, this file will be included in link.x.in.
   If the `no-exceptions` feature is ENABLED, this file will be ignored.
*/

/* It is possible to define a special handler for each exception type.
   By default, all exceptions are handled by ExceptionHandler. However,
   users can override these alias by defining the symbol themselves */
PROVIDE(InstructionMisaligned = ExceptionHandler);
PROVIDE(InstructionFault = ExceptionHandler);
PROVIDE(IllegalInstruction = ExceptionHandler);
PROVIDE(Breakpoint = ExceptionHandler);
PROVIDE(LoadMisaligned = ExceptionHandler);
PROVIDE(LoadFault = ExceptionHandler);
PROVIDE(StoreMisaligned = ExceptionHandler);
PROVIDE(StoreFault = ExceptionHandler);
PROVIDE(UserEnvCall = ExceptionHandler);
PROVIDE(SupervisorEnvCall = ExceptionHandler);
PROVIDE(MachineEnvCall = ExceptionHandler);
PROVIDE(InstructionPageFault = ExceptionHandler);
PROVIDE(LoadPageFault = ExceptionHandler);
PROVIDE(StorePageFault = ExceptionHandler);

/* # CORE INTERRUPT HANDLERS DESCRIBED IN THE STANDARD RISC-V ISA
   
   If the `no-interrupts` feature is DISABLED, this file will be included in link.x.in.
   If the `no-interrupts` feature is ENABLED, this file will be ignored.
*/

/* It is possible to define a special handler for each interrupt type.
   By default, all interrupts are handled by DefaultHandler. However, users can
   override these alias by defining the symbol themselves */
PROVIDE(SupervisorSoft = DefaultHandler);
PROVIDE(MachineSoft = DefaultHandler);
PROVIDE(SupervisorTimer = DefaultHandler);
PROVIDE(MachineTimer = DefaultHandler);
PROVIDE(SupervisorExternal = DefaultHandler);
PROVIDE(MachineExternal = DefaultHandler);

/* When vectored trap mode is enabled, each interrupt source must implement its own
   trap entry point. By default, all interrupts start in _DefaultHandler_trap.
   However, users can override these alias by defining the symbol themselves */
PROVIDE(_start_SupervisorSoft_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineSoft_trap = _start_DefaultHandler_trap);
PROVIDE(_start_SupervisorTimer_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineTimer_trap = _start_DefaultHandler_trap);
PROVIDE(_start_SupervisorExternal_trap = _start_DefaultHandler_trap);
PROVIDE(_start_MachineExternal_trap = _start_DefaultHandler_trap);

/* $$$$> END  : Contents from ${INCLUDE_LINKER_FILES} <$$$$ */

SECTIONS
{
  /* Header containing data needed by the bootloader.  We specify
     _HUBRIS_IMAGE_HEADER_SIZE and _HUBRIS_IMAGE_HEADER_ALIGN in memory.x at
     build time, then reserve enough space for the header here in the linker
     script.
   */
  .header :
  {
    ASSERT(. == ALIGN(_HUBRIS_IMAGE_HEADER_ALIGN), "error: header alignment is invalid");
    HEADER = .;
    KEEP(*(.start_block));
    . = . + _HUBRIS_IMAGE_HEADER_SIZE;
  } > VECTORS

  /* Optional RP235x IMAGE_DEF block loop (tiny: typically 20 bytes)
     We put it AFTER the header to preserve the bootloader invariant.
     If no object defines .image_def, this section has size 0. */
  .image_def :
  {
    __image_def_start = .;
    . = ALIGN(4);
    KEEP(*(.image_def));
    __image_def_end = .;
  } > VECTORS

  .text : ALIGN(4)
  {
    _stext = .;
    __stext = .;

    /* Put reset handler first in .text section so it ends up as the entry */
    /* point of the program. */
    KEEP(*(.init));
    . = ALIGN(4);
    KEEP(*(.trap.vector));   /* for _trap_vector (vectored mode only) */
    KEEP(*(.trap.start));    /* for _start_trap routine */
    KEEP(*(.trap.start.*));  /* for _start_INTERRUPT_trap routines (vectored mode only) */
    KEEP(*(.trap.continue)); /* for _continue_trap routine (vectored mode only) */
    KEEP(*(.trap.rust));     /* for _start_trap_rust Rust function */
    KEEP(*(.trap .trap.*));  /* Other .trap symbols at the end */

    *(.text.abort);
    *(.text .text.*);

    . = ALIGN(4);
    __etext = .;
  } > FLASH

  .rodata __etext : ALIGN(4)
  {
    __srodata = .;

    *(.srodata .srodata.*);
    *(.rodata .rodata.*);
    /* We move this into a special section so we can ensure it is always
       included in the build */
    KEEP(*(.hubris_id));
    /* 32-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(32);
    __erodata = .;
  } > FLASH

  .stack (NOLOAD) : ALIGN(8) {
    _stack_base = .;
    . = ORIGIN(STACK) + LENGTH(STACK);
    _stack_start = .;
  } >STACK

  .data : ALIGN(32)
  {
    . = ALIGN(32);
    __sdata = .;

    /* Must be called __global_pointer$ for linker relaxations to work. */
    PROVIDE(__global_pointer$ = . + 0x800);
    *(.sdata .sdata.* .sdata2 .sdata2.*);
    *(.data .data.*);

  } > RAM AT > FLASH
  
  /* Allow sections from user `memory.x` injected using `INSERT AFTER .data` to
   * use the .data loading mechanism by pushing __edata. Note: do not change
   * output region or load region in those user sections! */
  . = ALIGN(32);
  __edata = .;
  
  /* LMA of .data */
  __sidata = LOADADDR(.data);

  .bss (NOLOAD) : ALIGN(32)
  {
    . = ALIGN(32);
    __sbss = .;

    *(.sbss .sbss.* .bss .bss.*);
  } > RAM

  /* Allow sections from user `memory.x` injected using `INSERT AFTER .bss` to
   * use the .bss zeroing mechanism by pushing __ebss. Note: do not change
   * output region or load region in those user sections! */
  . = ALIGN(32);
  __ebss = .;

  /* Uninitialized data segment. In contrast with .bss, .uninit is not initialized to zero by
   * the runtime, and might contain residual data from previous executions or random values
   * if not explicitly initialized. While .bss and .uninit are different sections, they are
   * both allocated at RAM, as their purpose is similar. */
  .uninit (NOLOAD) : ALIGN(32)
  {
    . = ALIGN(32);
    __suninit = .;
    *(.uninit .uninit.*);
    . = ALIGN(32);
    __euninit = .;
  } > RAM

  /* Place the heap right after `.uninit` in RAM */
  PROVIDE(__sheap = __euninit);

  /* fake output .got section */
  /* Dynamic relocations are unsupported. This section is only used to detect
     relocatable code in the input files and raise an error if relocatable code
     is found */
  .got (INFO) :
  {
    KEEP(*(.got .got.*));
  }
}

/* Device-specific exception and interrupt handlers */
INCLUDE device.x

/* Do not exceed this mark in the error messages above                                    | */
ASSERT(ORIGIN(FLASH) % 4 == 0, "
ERROR(riscv-rt): the start of the FLASH must be 4-byte aligned");

ASSERT(ORIGIN(FLASH) % 4 == 0, "
ERROR(riscv-rt): the start of the FLASH must be 4-byte aligned");

ASSERT(ORIGIN(RAM) % 32 == 0, "
ERROR(riscv-rt): the start of the RAM must be 32-byte aligned");

ASSERT(ORIGIN(RAM) % 4 == 0, "
ERROR(riscv-rt): the start of the RAM must be 4-byte aligned");

ASSERT(ORIGIN(RAM) % 4 == 0, "
ERROR(riscv-rt): the start of the RAM must be 4-byte aligned");

ASSERT(_stext % 4 == 0, "
ERROR(riscv-rt): `_stext` must be 4-byte aligned");

ASSERT(__sdata % 32 == 0 && __edata % 32 == 0, "
BUG(riscv-rt): .data is not 32-byte aligned");

ASSERT(__sidata % 32 == 0, "
BUG(riscv-rt): the LMA of .data is not 32-byte aligned");

ASSERT(__sbss % 32 == 0 && __ebss % 32 == 0, "
BUG(riscv-rt): .bss is not 32-byte aligned");

ASSERT(__sheap % 4 == 0, "
BUG(riscv-rt): start of .heap is not 4-byte aligned");

ASSERT(_pre_init_trap % 4 == 0, "
BUG(riscv-rt): _pre_init_trap is not 4-byte aligned");

ASSERT(_start_trap % 4 == 0, "
BUG(riscv-rt): _start_trap is not 4-byte aligned");

ASSERT(_stext + SIZEOF(.text) < ORIGIN(FLASH) + LENGTH(FLASH), "
ERROR(riscv-rt): The .text section must be placed inside the FLASH region.
Set _stext to an address smaller than 'ORIGIN(FLASH) + LENGTH(FLASH)'");

ASSERT(SIZEOF(.stack) >= (_max_hart_id + 1) * _hart_stack_size, "
ERROR(riscv-rt): .stack section is too small for allocating stacks for all the harts.
Consider changing `_max_hart_id` or `_hart_stack_size`.");

/* # Other checks */
ASSERT(SIZEOF(.got) == 0, "
ERROR(riscv-rt): .got section detected in the input files. Dynamic relocations are not
supported. If you are linking to C code compiled using the `cc` crate then modify your
build script to compile the C code _without_ the -fPIC flag. See the documentation of
the `cc::Build.pic` method for details.");

/* --- optional safety checks for RP235x builds --- */

/* Guard against the header growing large enough to push IMAGE_DEF out of the first 4k */
ASSERT(SIZEOF(.image_def) == 0
       || (SIZEOF(.header) + SIZEOF(.image_def)) <= 0x1000,
"Vector+header+IMAGE_DEF must fit in first 4 KiB");

/* Ensure it remains the expected n-byte form (adjust if items are added) */
ASSERT(SIZEOF(.image_def) == 0
       || SIZEOF(.image_def) == 20,
"Unexpected IMAGE_DEF size; edit kernel-link-riscv.x if this change was expected");

/* Sanity: keep IMAGE_DEF before .text */
ASSERT(SIZEOF(.image_def) == 0
       || ADDR(.image_def) < ADDR(.text),
"IMAGE_DEF must precede .text");

/* Do not exceed this mark in the error messages above                                    | */
