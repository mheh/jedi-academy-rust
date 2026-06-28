// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "vm_local.h"

use core::ffi::c_int;

// ============================================================================
// LOCAL TYPES & EXTERN DECLARATIONS
// ============================================================================

// Local stub for vmSymbol_t (used only in DEBUG_VM)
#[repr(C)]
pub struct vmSymbol_t {
    // profileCount field is accessed in DEBUG_VM
    pub profileCount: c_int,
    // Additional fields may exist but are not accessed in vm_interpreted.cpp
}

// Local stub for vmHeader_t with fields accessed in VM_PrepareInterpreter
#[repr(C)]
pub struct vmHeader_t {
    pub codeOffset: c_int,
    pub instructionCount: c_int,
    pub codeLength: c_int,
    // Additional fields may exist but are not accessed in vm_interpreted.cpp
}

// Local stub for vm_t with fields accessed in vm_interpreted.cpp
// WARNING: This struct layout must match the C vm_t exactly for ABI compatibility.
// The actual field order and types are inferred from usage in vm_interpreted.cpp.
// Consult oracle/codemp/qcommon/vm_local.h for the authoritative definition.
#[repr(C)]
pub struct vm_t {
    pub codeBase: *mut u8,
    pub codeLength: c_int,
    // instructionPointers is a pointer to an array, sized to MAX_QVMCALLS or similar
    pub instructionPointers: *mut c_int,
    pub dataBase: *mut u8,
    pub dataMask: c_int,
    pub stackBottom: c_int,
    pub stackSize: c_int,
    pub programStack: c_int,
    pub callLevel: c_int,
    pub currentlyInterpreting: bool,
    pub breakFunction: c_int,
    pub breakCount: c_int,
    pub systemCall: Option<extern "C" fn(*mut c_int) -> c_int>,
    // Additional fields may exist but are not accessed in vm_interpreted.cpp
}

extern "C" {
    fn Com_Printf(fmt: *const u8, ...) -> ();
    fn Com_Error(code: c_int, fmt: *const u8, ...) -> !;
    fn Hunk_Alloc(size: c_int, tag: c_int) -> *mut u8;
    fn VM_ValueToSymbol(vm: *mut vm_t, pc: c_int) -> *const u8;
    fn VM_ValueToFunctionSymbol(vm: *mut vm_t, pc: c_int) -> *mut vmSymbol_t;
    fn VM_Debug(level: c_int) -> ();
    #[cfg(feature = "CRAZY_SYMBOL_MAP")]
    fn VM_SetSymbolMap(vm: *mut vm_t) -> ();
}

extern "C" {
    // Cvar definitions
    static mut com_vmdebug: *mut core::ffi::c_void;
    static mut vm_debugLevel: c_int;
}

// ============================================================================
// MACROS & CONSTANTS
// ============================================================================

// Opcode definitions (from vm_local.h)
const OP_UNDEF: u8 = 0;
const OP_IGNORE: u8 = 1;
const OP_BREAK: u8 = 2;
const OP_ENTER: u8 = 3;
const OP_LEAVE: u8 = 4;
const OP_CALL: u8 = 5;
const OP_PUSH: u8 = 6;
const OP_POP: u8 = 7;
const OP_CONST: u8 = 8;
const OP_LOCAL: u8 = 9;
const OP_JUMP: u8 = 10;
const OP_EQ: u8 = 11;
const OP_NE: u8 = 12;
const OP_LTI: u8 = 13;
const OP_LEI: u8 = 14;
const OP_GTI: u8 = 15;
const OP_GEI: u8 = 16;
const OP_LTU: u8 = 17;
const OP_LEU: u8 = 18;
const OP_GTU: u8 = 19;
const OP_GEU: u8 = 20;
const OP_EQF: u8 = 21;
const OP_NEF: u8 = 22;
const OP_LTF: u8 = 23;
const OP_LEF: u8 = 24;
const OP_GTF: u8 = 25;
const OP_GEF: u8 = 26;
const OP_LOAD1: u8 = 27;
const OP_LOAD2: u8 = 28;
const OP_LOAD4: u8 = 29;
const OP_STORE1: u8 = 30;
const OP_STORE2: u8 = 31;
const OP_STORE4: u8 = 32;
const OP_ARG: u8 = 33;
const OP_BLOCK_COPY: u8 = 34;
const OP_SEX8: u8 = 35;
const OP_SEX16: u8 = 36;
const OP_NEGI: u8 = 37;
const OP_ADD: u8 = 38;
const OP_SUB: u8 = 39;
const OP_DIVI: u8 = 40;
const OP_DIVU: u8 = 41;
const OP_MODI: u8 = 42;
const OP_MODU: u8 = 43;
const OP_MULI: u8 = 44;
const OP_MULU: u8 = 45;
const OP_BAND: u8 = 46;
const OP_BOR: u8 = 47;
const OP_BXOR: u8 = 48;
const OP_BCOM: u8 = 49;
const OP_LSH: u8 = 50;
const OP_RSHI: u8 = 51;
const OP_RSHU: u8 = 52;
const OP_NEGF: u8 = 53;
const OP_ADDF: u8 = 54;
const OP_SUBF: u8 = 55;
const OP_DIVF: u8 = 56;
const OP_MULF: u8 = 57;
const OP_CVIF: u8 = 58;
const OP_CVFI: u8 = 59;

const ERR_FATAL: c_int = 0;
const ERR_DROP: c_int = 1;

// h_high tag for Hunk_Alloc
const H_HIGH: c_int = 1;

#[cfg(DEBUG_VM)]
const DEBUG_VM: bool = true;
#[cfg(not(DEBUG_VM))]
const DEBUG_VM: bool = false;

const MAX_STACK: usize = 256;
const STACK_MASK: usize = MAX_STACK - 1;

// ============================================================================
// DEBUG TABLES
// ============================================================================

#[cfg(DEBUG_VM)]
static OPNAMES: [&str; 256] = [
    "OP_UNDEF",
    "OP_IGNORE",
    "OP_BREAK",
    "OP_ENTER",
    "OP_LEAVE",
    "OP_CALL",
    "OP_PUSH",
    "OP_POP",
    "OP_CONST",
    "OP_LOCAL",
    "OP_JUMP",
    "OP_EQ",
    "OP_NE",
    "OP_LTI",
    "OP_LEI",
    "OP_GTI",
    "OP_GEI",
    "OP_LTU",
    "OP_LEU",
    "OP_GTU",
    "OP_GEU",
    "OP_EQF",
    "OP_NEF",
    "OP_LTF",
    "OP_LEF",
    "OP_GTF",
    "OP_GEF",
    "OP_LOAD1",
    "OP_LOAD2",
    "OP_LOAD4",
    "OP_STORE1",
    "OP_STORE2",
    "OP_STORE4",
    "OP_ARG",
    "OP_BLOCK_COPY",
    "OP_SEX8",
    "OP_SEX16",
    "OP_NEGI",
    "OP_ADD",
    "OP_SUB",
    "OP_DIVI",
    "OP_DIVU",
    "OP_MODI",
    "OP_MODU",
    "OP_MULI",
    "OP_MULU",
    "OP_BAND",
    "OP_BOR",
    "OP_BXOR",
    "OP_BCOM",
    "OP_LSH",
    "OP_RSHI",
    "OP_RSHU",
    "OP_NEGF",
    "OP_ADDF",
    "OP_SUBF",
    "OP_DIVF",
    "OP_MULF",
    "OP_CVIF",
    "OP_CVFI",
    "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
];

// ============================================================================
// PLATFORM-SPECIFIC UTILITIES
// ============================================================================

// loadWord macro implementation
// #if idppc
//     #if defined(__GNUC__)
//         static inline unsigned int loadWord(void *addr) {
//             unsigned int word;
//             asm("lwbrx %0,0,%1" : "=r" (word) : "r" (addr));
//             return word;
//         }
//     #else
//         #define loadWord(addr) __lwbrx(addr,0)
//     #endif
// #else
//     #define loadWord(addr) *((int *)addr)
// #endif

#[inline]
unsafe fn loadWord(addr: *const u8) -> c_int {
    // For platforms that don't need byte-swapping, just dereference
    // For PPC platforms, this would need special handling
    *(addr as *const c_int)
}

// ============================================================================
// FUNCTIONS
// ============================================================================

pub unsafe fn VM_Indent(vm: *mut vm_t) -> *const u8 {
    static STRING: &[u8] = b"                                        ";

    // Preserve the bounds check logic from original
    let call_level = (*vm).callLevel;
    if call_level > 20 {
        STRING.as_ptr()
    } else {
        // Return pointer to substring starting at position 2 * (20 - callLevel)
        let offset = 2 * (20 - call_level) as usize;
        if offset < STRING.len() {
            STRING.as_ptr().add(offset)
        } else {
            b"".as_ptr()
        }
    }
}

pub unsafe fn VM_StackTrace(vm: *mut vm_t, mut programCounter: c_int, mut programStack: c_int) {
    let mut count = 0;

    loop {
        Com_Printf(b"%s\n\0".as_ptr(), VM_ValueToSymbol(vm, programCounter));
        programStack = *((*vm).dataBase.add(programStack as usize + 4) as *const c_int);
        programCounter = *((*vm).dataBase.add(programStack as usize) as *const c_int);

        count += 1;
        if programCounter == -1 || count >= 32 {
            break;
        }
    }
}

// ====================
// VM_PrepareInterpreter
// ====================
pub unsafe fn VM_PrepareInterpreter(vm: *mut vm_t, header: *mut vmHeader_t) {
    let mut op: u8;
    let mut pc: c_int;
    let mut code: *const u8;
    let mut instruction: c_int;
    let mut codeBase: *mut c_int;

    (*vm).codeBase = Hunk_Alloc((*vm).codeLength * 4, H_HIGH); // we're now int aligned
    // memcpy( vm->codeBase, (byte *)header + header->codeOffset, vm->codeLength );

    // we don't need to translate the instructions, but we still need
    // to find each instructions starting point for jumps
    pc = 0;
    instruction = 0;
    code = (header as *const u8).add((*header).codeOffset as usize);
    codeBase = (*vm).codeBase as *mut c_int;

    while instruction < (*header).instructionCount {
        let instructions_ptr = core::ptr::addr_of_mut!((*vm).instructionPointers);
        *instructions_ptr.add(instruction as usize) = pc;
        instruction += 1;

        op = *code.add(pc as usize);
        *codeBase.add(pc as usize) = op as c_int;
        if pc > (*header).codeLength {
            Com_Error(ERR_FATAL, b"VM_PrepareInterpreter: pc > header->codeLength\0".as_ptr() as *const u8);
        }

        pc += 1;

        // these are the only opcodes that aren't a single byte
        match op {
            OP_ENTER | OP_CONST | OP_LOCAL | OP_LEAVE | OP_EQ | OP_NE | OP_LTI | OP_LEI
            | OP_GTI | OP_GEI | OP_LTU | OP_LEU | OP_GTU | OP_GEU | OP_EQF | OP_NEF
            | OP_LTF | OP_LEF | OP_GTF | OP_GEF | OP_BLOCK_COPY => {
                *codeBase.add((pc) as usize) = loadWord(code.add(pc as usize));
                pc += 4;
            }
            OP_ARG => {
                *codeBase.add(pc as usize) = *code.add(pc as usize) as c_int;
                pc += 1;
            }
            _ => {}
        }
    }

    pc = 0;
    instruction = 0;
    code = (header as *const u8).add((*header).codeOffset as usize);
    codeBase = (*vm).codeBase as *mut c_int;

    let instructions_ptr = core::ptr::addr_of_mut!((*vm).instructionPointers);

    while instruction < (*header).instructionCount {
        op = *code.add(pc as usize);
        instruction += 1;
        pc += 1;

        match op {
            OP_ENTER | OP_CONST | OP_LOCAL | OP_LEAVE | OP_EQ | OP_NE | OP_LTI | OP_LEI
            | OP_GTI | OP_GEI | OP_LTU | OP_LEU | OP_GTU | OP_GEU | OP_EQF | OP_NEF
            | OP_LTF | OP_LEF | OP_GTF | OP_GEF | OP_BLOCK_COPY => {
                // Nested match for special cases that need address translation
                match op {
                    OP_EQ | OP_NE | OP_LTI | OP_LEI | OP_GTI | OP_GEI | OP_LTU | OP_LEU
                    | OP_GTU | OP_GEU | OP_EQF | OP_NEF | OP_LTF | OP_LEF | OP_GTF | OP_GEF => {
                        let idx = *codeBase.add(pc as usize) as usize;
                        *codeBase.add(pc as usize) = *instructions_ptr.add(idx);
                    }
                    _ => {}
                }
                pc += 4;
            }
            OP_ARG => {
                pc += 1;
            }
            _ => {}
        }
    }
}

// ==============
// VM_Call
//
// Upon a system call, the stack will look like:
//
// sp+32	parm1
// sp+28	parm0
// sp+24	return stack
// sp+20	return address
// sp+16	local1
// sp+14	local0
// sp+12	arg1
// sp+8	arg0
// sp+4	return stack
// sp		return address
//
// An interpreted function will immediately execute
// an OP_ENTER instruction, which will subtract space for
// locals from sp
// ==============

pub unsafe fn VM_CallInterpreted(vm: *mut vm_t, args: *const c_int) -> c_int {
    let mut stack: [c_int; MAX_STACK] = [0; MAX_STACK];
    let mut opStack: *mut c_int = stack.as_mut_ptr();
    let mut programCounter: c_int;
    let mut programStack: c_int;
    let stackOnEntry: c_int;
    let image: *mut u8;
    let mut codeImage: *const c_int;
    let mut v1: c_int;
    let dataMask: c_int;

    #[cfg(DEBUG_VM)]
    let mut profileSymbol: *mut vmSymbol_t = std::ptr::null_mut();

    // interpret the code
    (*vm).currentlyInterpreting = true; // qtrue

    #[cfg(feature = "CRAZY_SYMBOL_MAP")]
    VM_SetSymbolMap(vm);

    // we might be called recursively, so this might not be the very top
    programStack = stackOnEntry = (*vm).programStack;

    #[cfg(DEBUG_VM)]
    {
        let com_vmdebug_ptr = com_vmdebug as *const c_int;
        if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr != 0 {
            profileSymbol = VM_ValueToFunctionSymbol(vm, 0);
        }
        // uncomment this for debugging breakpoints
        (*vm).breakFunction = 0;
    }

    // set up the stack frame
    image = (*vm).dataBase;
    codeImage = (*vm).codeBase as *const c_int;
    dataMask = (*vm).dataMask;

    // leave a free spot at start of stack so
    // that as long as opStack is valid, opStack-1 will
    // not corrupt anything
    opStack = stack.as_mut_ptr();
    programCounter = 0;

    programStack -= 48;

    *(image.add(programStack as usize + 44) as *mut c_int) = *args.add(9);
    *(image.add(programStack as usize + 40) as *mut c_int) = *args.add(8);
    *(image.add(programStack as usize + 36) as *mut c_int) = *args.add(7);
    *(image.add(programStack as usize + 32) as *mut c_int) = *args.add(6);
    *(image.add(programStack as usize + 28) as *mut c_int) = *args.add(5);
    *(image.add(programStack as usize + 24) as *mut c_int) = *args.add(4);
    *(image.add(programStack as usize + 20) as *mut c_int) = *args.add(3);
    *(image.add(programStack as usize + 16) as *mut c_int) = *args.add(2);
    *(image.add(programStack as usize + 12) as *mut c_int) = *args.add(1);
    *(image.add(programStack as usize + 8) as *mut c_int) = *args.add(0);
    *(image.add(programStack as usize + 4) as *mut c_int) = 0; // return stack
    *(image.add(programStack as usize) as *mut c_int) = -1; // will terminate the loop on return

    (*vm).callLevel = 0;

    VM_Debug(0);

    // vm_debugLevel=2;
    // main interpreter loop, will exit when a LEAVE instruction
    // grabs the -1 program counter

    'main_loop: loop {
        // load r0 and r1 from stack
        let mut r0: c_int = *opStack;
        let mut r1: c_int = *(opStack.offset(-1) as *const c_int);

        'opcode_loop: loop {
            let opcode: u8 = *codeImage.add(programCounter as usize) as u8;
            programCounter += 1;

            #[cfg(DEBUG_VM)]
            {
                let com_vmdebug_ptr = com_vmdebug as *const c_int;
                if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr > 1 {
                    if (programCounter as u32) > (*vm).codeLength as u32 {
                        Com_Error(ERR_DROP, b"VM pc out of range\0".as_ptr() as *const u8);
                    }

                    if opStack < stack.as_mut_ptr() {
                        Com_Error(ERR_DROP, b"VM opStack underflow\0".as_ptr() as *const u8);
                    }
                    if opStack >= stack.as_mut_ptr().add(MAX_STACK) {
                        Com_Error(ERR_DROP, b"VM opStack overflow\0".as_ptr() as *const u8);
                    }

                    if programStack <= (*vm).stackBottom {
                        Com_Error(ERR_DROP, b"VM stack overflow\0".as_ptr() as *const u8);
                    }

                    if programStack & 3 != 0 {
                        Com_Error(ERR_DROP, b"VM program stack misaligned\0".as_ptr() as *const u8);
                    }

                    if vm_debugLevel > 1 {
                        let indent = VM_Indent(vm);
                        let stack_offset = opStack as usize - stack.as_ptr() as usize;
                        Com_Printf(b"%s %s\n\0".as_ptr(), indent, OPNAMES[opcode as usize].as_ptr());
                    }
                    (*profileSymbol).profileCount += 1;
                }
            }

            match opcode {
                #[cfg(DEBUG_VM)]
                _ if !matches!(
                    opcode,
                    OP_BREAK | OP_CONST | OP_LOCAL | OP_LOAD4 | OP_LOAD2 | OP_LOAD1
                    | OP_STORE4 | OP_STORE2 | OP_STORE1 | OP_ARG | OP_BLOCK_COPY
                    | OP_CALL | OP_PUSH | OP_POP | OP_ENTER | OP_LEAVE | OP_JUMP
                    | OP_EQ | OP_NE | OP_LTI | OP_LEI | OP_GTI | OP_GEI
                    | OP_LTU | OP_LEU | OP_GTU | OP_GEU
                    | OP_EQF | OP_NEF | OP_LTF | OP_LEF | OP_GTF | OP_GEF
                    | OP_NEGI | OP_ADD | OP_SUB | OP_DIVI | OP_DIVU | OP_MODI | OP_MODU
                    | OP_MULI | OP_MULU | OP_BAND | OP_BOR | OP_BXOR | OP_BCOM
                    | OP_LSH | OP_RSHI | OP_RSHU
                    | OP_NEGF | OP_ADDF | OP_SUBF | OP_DIVF | OP_MULF
                    | OP_CVIF | OP_CVFI | OP_SEX8 | OP_SEX16
                ) => {
                    Com_Error(ERR_DROP, b"Bad VM instruction\0".as_ptr() as *const u8);
                }

                OP_BREAK => {
                    (*vm).breakCount += 1;
                    continue 'opcode_loop;
                }

                OP_CONST => {
                    opStack = opStack.offset(1);
                    r1 = r0;
                    r0 = *opStack = *codeImage.add(programCounter as usize);
                    programCounter += 4;
                    continue 'opcode_loop;
                }

                OP_LOCAL => {
                    opStack = opStack.offset(1);
                    r1 = r0;
                    r0 = *opStack = *codeImage.add(programCounter as usize) + programStack;
                    programCounter += 4;
                    continue 'opcode_loop;
                }

                OP_LOAD4 => {
                    #[cfg(DEBUG_VM)]
                    {
                        let com_vmdebug_ptr = com_vmdebug as *const c_int;
                        if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr > 1 {
                            if *opStack & 3 != 0 {
                                Com_Error(ERR_DROP, b"OP_LOAD4 misaligned\0".as_ptr() as *const u8);
                            }
                        }
                    }
                    r0 = *opStack = *(image.add((*opStack & dataMask) as usize) as *const c_int);
                    continue 'opcode_loop;
                }

                OP_LOAD2 => {
                    r0 = *opStack = *(image.add((*opStack & dataMask) as usize) as *const u16) as c_int;
                    continue 'opcode_loop;
                }

                OP_LOAD1 => {
                    r0 = *opStack = *image.add((*opStack & dataMask) as usize) as c_int;
                    continue 'opcode_loop;
                }

                OP_STORE4 => {
                    *(image.add((r1 & (dataMask & !3)) as usize) as *mut c_int) = r0;
                    opStack = opStack.offset(-2);
                    break 'opcode_loop;
                }

                OP_STORE2 => {
                    *(image.add((r1 & (dataMask & !1)) as usize) as *mut i16) = r0 as i16;
                    opStack = opStack.offset(-2);
                    break 'opcode_loop;
                }

                OP_STORE1 => {
                    *image.add((r1 & dataMask) as usize) = r0 as u8;
                    opStack = opStack.offset(-2);
                    break 'opcode_loop;
                }

                OP_ARG => {
                    // single byte offset from programStack
                    *(image.add((*codeImage.add(programCounter as usize) + programStack) as usize) as *mut c_int) = r0;
                    opStack = opStack.offset(-1);
                    programCounter += 1;
                    break 'opcode_loop;
                }

                OP_BLOCK_COPY => {
                    let count = *codeImage.add(programCounter as usize);
                    // MrE: copy range check
                    let mut srci = r0 & dataMask;
                    let mut desti = r1 & dataMask;
                    let mut count_var = ((srci + count) & dataMask) - srci;
                    count_var = ((desti + count_var) & dataMask) - desti;

                    let src = image.add((r0 & dataMask) as usize) as *mut c_int;
                    let dest = image.add((r1 & dataMask) as usize) as *mut c_int;

                    if ((src as c_int) | (dest as c_int) | count_var) & 3 != 0 {
                        Com_Error(ERR_DROP, b"OP_BLOCK_COPY not dword aligned\0".as_ptr() as *const u8);
                    }

                    let count_dw = count_var >> 2;
                    let mut i = count_dw - 1;
                    while i >= 0 {
                        *dest.add(i as usize) = *src.add(i as usize);
                        i -= 1;
                    }
                    programCounter += 4;
                    opStack = opStack.offset(-2);
                    break 'opcode_loop;
                }

                OP_CALL => {
                    // save current program counter
                    *(image.add(programStack as usize) as *mut c_int) = programCounter;

                    // jump to the location on the stack
                    programCounter = r0;
                    opStack = opStack.offset(-1);

                    if programCounter < 0 {
                        // system call
                        #[cfg(DEBUG_VM)]
                        {
                            let com_vmdebug_ptr = com_vmdebug as *const c_int;
                            if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr > 1 {
                                if vm_debugLevel != 0 {
                                    let indent = VM_Indent(vm);
                                    let stack_offset = opStack as usize - stack.as_ptr() as usize;
                                    Com_Printf(b"%s---> systemcall(%i)\n\0".as_ptr(), indent, -1 - programCounter);
                                }
                            }
                        }

                        // save the stack to allow recursive VM entry
                        let temp = (*vm).callLevel;
                        (*vm).programStack = programStack - 4;

                        #[cfg(DEBUG_VM)]
                        let stomped = *(image.add(programStack as usize + 4) as *const c_int);

                        *(image.add(programStack as usize + 4) as *mut c_int) = -1 - programCounter;

                        // VM_LogSyscalls( (int *)&image[ programStack + 4 ] );
                        let r = if let Some(syscall_fn) = (*vm).systemCall {
                            syscall_fn(image.add(programStack as usize + 4) as *mut c_int)
                        } else {
                            0
                        };

                        #[cfg(DEBUG_VM)]
                        {
                            // this is just our stack frame pointer, only needed
                            // for debugging
                            *(image.add(programStack as usize + 4) as *mut c_int) = stomped;
                        }

                        // save return value
                        opStack = opStack.offset(1);
                        *opStack = r;
                        programCounter = *(image.add(programStack as usize) as *const c_int);
                        (*vm).callLevel = temp;

                        #[cfg(DEBUG_VM)]
                        {
                            let com_vmdebug_ptr = com_vmdebug as *const c_int;
                            if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr > 1 {
                                if vm_debugLevel != 0 {
                                    let indent = VM_Indent(vm);
                                    let stack_offset = opStack as usize - stack.as_ptr() as usize;
                                    Com_Printf(b"%s<--- %s\n\0".as_ptr(), indent, VM_ValueToSymbol(vm, programCounter));
                                }
                            }
                        }
                    } else {
                        let instructions_ptr = core::ptr::addr_of!((*vm).instructionPointers);
                        programCounter = *instructions_ptr.add(programCounter as usize);
                    }
                    break 'opcode_loop;
                }

                // push and pop are only needed for discarded or bad function return values
                OP_PUSH => {
                    opStack = opStack.offset(1);
                    break 'opcode_loop;
                }

                OP_POP => {
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_ENTER => {
                    #[cfg(DEBUG_VM)]
                    {
                        let com_vmdebug_ptr = com_vmdebug as *const c_int;
                        if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr != 0 {
                            profileSymbol = VM_ValueToFunctionSymbol(vm, programCounter);
                        }
                    }

                    // get size of stack frame
                    v1 = *codeImage.add(programCounter as usize);

                    programCounter += 4;
                    programStack -= v1;

                    #[cfg(DEBUG_VM)]
                    {
                        let com_vmdebug_ptr = com_vmdebug as *const c_int;
                        if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr > 1 {
                            // save old stack frame for debugging traces
                            *(image.add((programStack + 4) as usize) as *mut c_int) = programStack + v1;
                            if vm_debugLevel != 0 {
                                let indent = VM_Indent(vm);
                                let stack_offset = opStack as usize - stack.as_ptr() as usize;
                                Com_Printf(b"%s---> %s\n\0".as_ptr(), indent, VM_ValueToSymbol(vm, programCounter - 5));
                                if (*vm).breakFunction != 0 && programCounter - 5 == (*vm).breakFunction {
                                    // this is to allow setting breakpoints here in the debugger
                                    (*vm).breakCount += 1;
                                    // vm_debugLevel = 2;
                                    // VM_StackTrace( vm, programCounter, programStack );
                                }
                                (*vm).callLevel += 1;
                            }
                        }
                    }
                    break 'opcode_loop;
                }

                OP_LEAVE => {
                    // remove our stack frame
                    v1 = *codeImage.add(programCounter as usize);

                    programStack += v1;

                    // grab the saved program counter
                    programCounter = *(image.add(programStack as usize) as *const c_int);

                    #[cfg(DEBUG_VM)]
                    {
                        let com_vmdebug_ptr = com_vmdebug as *const c_int;
                        if !com_vmdebug_ptr.is_null() && *com_vmdebug_ptr != 0 {
                            profileSymbol = VM_ValueToFunctionSymbol(vm, programCounter);
                            if vm_debugLevel != 0 {
                                (*vm).callLevel -= 1;
                                let indent = VM_Indent(vm);
                                let stack_offset = opStack as usize - stack.as_ptr() as usize;
                                Com_Printf(b"%s<--- %s\n\0".as_ptr(), indent, VM_ValueToSymbol(vm, programCounter));
                            }
                        }
                    }

                    // check for leaving the VM
                    if programCounter == -1 {
                        break 'main_loop;
                    }
                    break 'opcode_loop;
                }

                // ===================================================================
                // BRANCHES
                // ===================================================================

                OP_JUMP => {
                    programCounter = r0;
                    let instructions_ptr = core::ptr::addr_of!((*vm).instructionPointers);
                    programCounter = *instructions_ptr.add(programCounter as usize);
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_EQ => {
                    opStack = opStack.offset(-2);
                    if r1 == r0 {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_NE => {
                    opStack = opStack.offset(-2);
                    if r1 != r0 {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_LTI => {
                    opStack = opStack.offset(-2);
                    if r1 < r0 {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_LEI => {
                    opStack = opStack.offset(-2);
                    if r1 <= r0 {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_GTI => {
                    opStack = opStack.offset(-2);
                    if r1 > r0 {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_GEI => {
                    opStack = opStack.offset(-2);
                    if r1 >= r0 {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_LTU => {
                    opStack = opStack.offset(-2);
                    if (r1 as u32) < (r0 as u32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_LEU => {
                    opStack = opStack.offset(-2);
                    if (r1 as u32) <= (r0 as u32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_GTU => {
                    opStack = opStack.offset(-2);
                    if (r1 as u32) > (r0 as u32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_GEU => {
                    opStack = opStack.offset(-2);
                    if (r1 as u32) >= (r0 as u32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        break 'opcode_loop;
                    }
                }

                OP_EQF => {
                    if *(opStack.add(-1) as *const f32) == *(opStack as *const f32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    }
                }

                OP_NEF => {
                    if *(opStack.add(-1) as *const f32) != *(opStack as *const f32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    }
                }

                OP_LTF => {
                    if *(opStack.add(-1) as *const f32) < *(opStack as *const f32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    }
                }

                OP_LEF => {
                    if *(opStack.add(-1) as *const f32) <= *(opStack as *const f32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    }
                }

                OP_GTF => {
                    if *(opStack.add(-1) as *const f32) > *(opStack as *const f32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    }
                }

                OP_GEF => {
                    if *(opStack.add(-1) as *const f32) >= *(opStack as *const f32) {
                        programCounter = *codeImage.add(programCounter as usize);
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    } else {
                        programCounter += 4;
                        opStack = opStack.offset(-2);
                        break 'opcode_loop;
                    }
                }

                // ===================================================================

                OP_NEGI => {
                    *opStack = -r0;
                    break 'opcode_loop;
                }

                OP_ADD => {
                    *opStack.offset(-1) = r1 + r0;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_SUB => {
                    *opStack.offset(-1) = r1 - r0;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_DIVI => {
                    *opStack.offset(-1) = r1 / r0;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_DIVU => {
                    *opStack.offset(-1) = ((r1 as u32) / (r0 as u32)) as c_int;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_MODI => {
                    *opStack.offset(-1) = r1 % r0;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_MODU => {
                    *opStack.offset(-1) = ((r1 as u32) % (r0 as u32)) as c_int;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_MULI => {
                    *opStack.offset(-1) = r1 * r0;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_MULU => {
                    *opStack.offset(-1) = ((r1 as u32) * (r0 as u32)) as c_int;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_BAND => {
                    *opStack.offset(-1) = ((r1 as u32) & (r0 as u32)) as c_int;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_BOR => {
                    *opStack.offset(-1) = ((r1 as u32) | (r0 as u32)) as c_int;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_BXOR => {
                    *opStack.offset(-1) = ((r1 as u32) ^ (r0 as u32)) as c_int;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_BCOM => {
                    *opStack.offset(-1) = !(r0 as u32) as c_int;
                    break 'opcode_loop;
                }

                OP_LSH => {
                    *opStack.offset(-1) = r1 << r0;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_RSHI => {
                    *opStack.offset(-1) = r1 >> r0;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_RSHU => {
                    *opStack.offset(-1) = ((r1 as u32) >> r0) as c_int;
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_NEGF => {
                    *(opStack as *mut f32) = -(*(opStack as *const f32));
                    break 'opcode_loop;
                }

                OP_ADDF => {
                    *(opStack.offset(-1) as *mut f32) =
                        *(opStack.offset(-1) as *const f32) + *(opStack as *const f32);
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_SUBF => {
                    *(opStack.offset(-1) as *mut f32) =
                        *(opStack.add(-1) as *const f32) - *(opStack as *const f32);
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_DIVF => {
                    *(opStack.offset(-1) as *mut f32) =
                        *(opStack.add(-1) as *const f32) / *(opStack as *const f32);
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_MULF => {
                    *(opStack.offset(-1) as *mut f32) =
                        *(opStack.add(-1) as *const f32) * *(opStack as *const f32);
                    opStack = opStack.offset(-1);
                    break 'opcode_loop;
                }

                OP_CVIF => {
                    *(opStack as *mut f32) = *opStack as f32;
                    break 'opcode_loop;
                }

                OP_CVFI => {
                    *opStack = *(opStack as *const f32) as c_int;
                    break 'opcode_loop;
                }

                OP_SEX8 => {
                    *opStack = (*opStack as i8) as c_int;
                    break 'opcode_loop;
                }

                OP_SEX16 => {
                    *opStack = (*opStack as i16) as c_int;
                    break 'opcode_loop;
                }

                _ => {
                    break 'opcode_loop;
                }
            }
        }
    }

    (*vm).currentlyInterpreting = false;

    if opStack != stack.as_mut_ptr().offset(1) {
        Com_Error(ERR_DROP, b"Interpreter error: opStack = %i\0".as_ptr() as *const u8, opStack as c_int - stack.as_ptr() as c_int);
    }

    (*vm).programStack = stackOnEntry;

    // return the result
    *opStack
}
