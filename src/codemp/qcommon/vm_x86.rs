// vm_x86.c -- load time compiler and execution environment for x86
//Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void, c_char};

// Local stubs for external dependencies
extern "C" {
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Com_Memset(dest: *mut c_void, c: c_int, count: usize) -> *mut c_void;
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Z_Malloc(size: usize, tag: c_int, clear: c_int) -> *mut u8;
    fn Z_Free(ptr: *mut c_void);
    fn Hunk_Alloc(size: usize, low: c_int) -> *mut u8;

    static mut currentVM: *mut vm_t;
}

#[cfg(target_os = "windows")]
extern "C" {
    fn _ftol(x: f32) -> c_int;
    fn doAsmCall();
}

#[cfg(not(target_os = "windows"))]
extern "C" {
    fn qftol() -> c_int;
    fn qftol027F() -> c_int;
    fn qftol037F() -> c_int;
    fn qftol0E7F() -> c_int;
    fn qftol0F7F() -> c_int;
    fn doAsmCall();
    fn callAsmCall();
}

// Local stub types (from vm_local.h and related headers)
#[repr(C)]
pub struct vm_t {
    pub name: [c_char; 256],
    pub dataMask: c_int,
    pub dataBase: *mut u8,
    pub instructionPointers: *mut c_int,
    pub codeBase: *mut u8,
    pub codeLength: c_int,
    pub programStack: c_int,
    pub systemCall: extern "C" fn(*mut c_int) -> c_int,
    pub currentlyInterpreting: c_int,
}

#[repr(C)]
pub struct vmHeader_t {
    pub vmMagic: c_int,
    pub instructionCount: c_int,
    pub codeOffset: c_int,
    pub codeLength: c_int,
    pub dataOffset: c_int,
    pub dataLength: c_int,
    pub litLength: c_int,
    pub bssLength: c_int,
}

// Op codes
const OP_BREAK: u8 = 1;
const OP_ENTER: u8 = 2;
const OP_LEAVE: u8 = 3;
const OP_CALL: u8 = 4;
const OP_PUSH: u8 = 5;
const OP_POP: u8 = 6;
const OP_CONST: u8 = 7;
const OP_LOCAL: u8 = 8;
const OP_JUMP: u8 = 9;
const OP_EQ: u8 = 10;
const OP_NE: u8 = 11;
const OP_LTI: u8 = 12;
const OP_LEI: u8 = 13;
const OP_GTI: u8 = 14;
const OP_GEI: u8 = 15;
const OP_LTU: u8 = 16;
const OP_LEU: u8 = 17;
const OP_GTU: u8 = 18;
const OP_GEU: u8 = 19;
const OP_EQF: u8 = 20;
const OP_NEF: u8 = 21;
const OP_LTF: u8 = 22;
const OP_LEF: u8 = 23;
const OP_GTF: u8 = 24;
const OP_GEF: u8 = 25;
const OP_LOAD1: u8 = 26;
const OP_LOAD2: u8 = 27;
const OP_LOAD4: u8 = 28;
const OP_STORE1: u8 = 29;
const OP_STORE2: u8 = 30;
const OP_STORE4: u8 = 31;
const OP_ARG: u8 = 32;
const OP_BLOCK_COPY: u8 = 33;
const OP_SEX8: u8 = 34;
const OP_SEX16: u8 = 35;
const OP_NEGI: u8 = 36;
const OP_ADD: u8 = 37;
const OP_SUB: u8 = 38;
const OP_DIVI: u8 = 39;
const OP_DIVU: u8 = 40;
const OP_MODI: u8 = 41;
const OP_MODU: u8 = 42;
const OP_MULI: u8 = 43;
const OP_MULU: u8 = 44;
const OP_BAND: u8 = 45;
const OP_BOR: u8 = 46;
const OP_BXOR: u8 = 47;
const OP_BCOM: u8 = 48;
const OP_LSH: u8 = 49;
const OP_RSHI: u8 = 50;
const OP_RSHU: u8 = 51;
const OP_NEGF: u8 = 52;
const OP_ADDF: u8 = 53;
const OP_SUBF: u8 = 54;
const OP_DIVF: u8 = 55;
const OP_MULF: u8 = 56;
const OP_CVIF: u8 = 57;
const OP_CVFI: u8 = 58;

// Error codes
const ERR_FATAL: c_int = 3;
const ERR_DROP: c_int = 2;

// Tags
const TAG_VM: c_int = 9;

// Memory allocator tags
const h_low: c_int = 0;

/*

  eax	scratch
  ebx	scratch
  ecx	scratch (required for shifts)
  edx	scratch (required for divisions)
  esi	program stack
  edi	opstack

*/

// TTimo: initialised the statics, this fixes a crash when entering a compiled VM
static mut buf: *mut u8 = core::ptr::null_mut();
static mut jused: *mut u8 = core::ptr::null_mut();
static mut compiledOfs: c_int = 0;
static mut code: *mut u8 = core::ptr::null_mut();
static mut pc: c_int = 0;

static mut instructionPointers: *mut c_int = core::ptr::null_mut();

//#undef FTOL_PTR  // bk001213
#[cfg(target_os = "windows")]
const FTOL_PTR: bool = true;

#[cfg(not(target_os = "windows"))]
const FTOL_PTR: bool = true;

#[cfg(target_os = "windows")]
static mut ftolPtr: c_int = unsafe { _ftol as c_int };

#[cfg(target_os = "windows")]
static mut asmCallPtr: c_int = unsafe { AsmCall as c_int };

#[cfg(not(target_os = "windows"))]
static mut ftolPtr: c_int = unsafe { qftol0F7F as c_int };

#[cfg(not(target_os = "windows"))]
static mut asmCallPtr: c_int = unsafe { doAsmCall as c_int };

static mut callMask: c_int = 0; // bk001213 - init

static mut instruction: c_int = 0;
static mut pass: c_int = 0;
static mut lastConst: c_int = 0;
static mut oc0: c_int = 0;
static mut oc1: c_int = 0;
static mut pop0: c_int = 0;
static mut pop1: c_int = 0;

#[repr(u32)]
enum ELastCommand {
    LAST_COMMAND_NONE = 0,
    LAST_COMMAND_MOV_EDI_EAX = 1,
    LAST_COMMAND_SUB_DI_4 = 2,
    LAST_COMMAND_SUB_DI_8 = 3,
}

static mut LastCommand: ELastCommand = ELastCommand::LAST_COMMAND_NONE;

/*
=================
AsmCall
=================
*/
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub unsafe fn AsmCall() {
    // This is the Windows version with MSVC inline assembly
    // Since we can't translate MSVC __asm directly, this remains a stub
    // The actual implementation needs to be provided via external asm
}

#[cfg(not(target_os = "windows"))]
static mut callProgramStack: c_int = 0;
#[cfg(not(target_os = "windows"))]
static mut callOpStack: *mut c_int = core::ptr::null_mut();
#[cfg(not(target_os = "windows"))]
static mut callSyscallNum: c_int = 0;

#[cfg(not(target_os = "windows"))]
#[allow(unsafe_code)]
pub unsafe fn AsmCall() {
    // This is the Unix version with inline assembly
    // The actual assembly inline is preserved in external asm
}

static fn Constant4() -> c_int {
    unsafe {
        let v: c_int = (*code.offset(pc as isize) as c_int) |
                       ((*code.offset((pc + 1) as isize) as c_int) << 8) |
                       ((*code.offset((pc + 2) as isize) as c_int) << 16) |
                       ((*code.offset((pc + 3) as isize) as c_int) << 24);
        pc += 4;
        v
    }
}

static fn Constant1() -> c_int {
    unsafe {
        let v: c_int = *code.offset(pc as isize) as c_int;
        pc += 1;
        v
    }
}

static fn Emit1(v: c_int) {
    unsafe {
        *buf.offset(compiledOfs as isize) = (v & 255) as u8;
        compiledOfs += 1;
        LastCommand = ELastCommand::LAST_COMMAND_NONE;
    }
}

#[allow(dead_code)]
static fn Emit2(v: c_int) {
    Emit1(v & 255);
    Emit1((v >> 8) & 255);
}

static fn Emit4(v: c_int) {
    Emit1(v & 255);
    Emit1((v >> 8) & 255);
    Emit1((v >> 16) & 255);
    Emit1((v >> 24) & 255);
}

static fn Hex(c: c_int) -> c_int {
    if c >= b'a' as c_int && c <= b'f' as c_int {
        return 10 + c - b'a' as c_int;
    }
    if c >= b'A' as c_int && c <= b'F' as c_int {
        return 10 + c - b'A' as c_int;
    }
    if c >= b'0' as c_int && c <= b'9' as c_int {
        return c - b'0' as c_int;
    }

    unsafe {
        Com_Error(ERR_DROP, b"Hex: bad char '%c'\0".as_ptr() as *const c_char, c);
    }

    0
}

static fn EmitString(string: *const c_char) {
    unsafe {
        let mut string_mut = string as *const u8;
        loop {
            let c1: c_int = *string_mut as c_int;
            let c2: c_int = *string_mut.offset(1) as c_int;

            let v: c_int = (Hex(c1) << 4) | Hex(c2);
            Emit1(v);

            if *string_mut.offset(2) == 0 {
                break;
            }
            string_mut = string_mut.offset(3);
        }
    }
}

static fn EmitCommand(command: ELastCommand) {
    match command {
        ELastCommand::LAST_COMMAND_MOV_EDI_EAX => {
            EmitString(b"89 07\0".as_ptr() as *const c_char);  // mov dword ptr [edi], eax
        }
        ELastCommand::LAST_COMMAND_SUB_DI_4 => {
            EmitString(b"83 EF 04\0".as_ptr() as *const c_char);  // sub edi, 4
        }
        ELastCommand::LAST_COMMAND_SUB_DI_8 => {
            EmitString(b"83 EF 08\0".as_ptr() as *const c_char);  // sub edi, 8
        }
        ELastCommand::LAST_COMMAND_NONE => {}
    }
    LastCommand = command;
}

static fn EmitAddEDI4(vm: *mut vm_t) {
    unsafe {
        if matches!(LastCommand, ELastCommand::LAST_COMMAND_SUB_DI_4) &&
           *jused.offset((instruction - 1) as isize) == 0 {
            // sub di,4
            compiledOfs -= 3;
            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
            return;
        }
        if matches!(LastCommand, ELastCommand::LAST_COMMAND_SUB_DI_8) &&
           *jused.offset((instruction - 1) as isize) == 0 {
            // sub di,8
            compiledOfs -= 3;
            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
            EmitString(b"83 EF 04\0".as_ptr() as *const c_char);  // sub edi,4
            return;
        }
        EmitString(b"83 C7 04\0".as_ptr() as *const c_char);  // add edi,4
    }
}

static fn EmitMovEAXEDI(vm: *mut vm_t) {
    unsafe {
        if matches!(LastCommand, ELastCommand::LAST_COMMAND_MOV_EDI_EAX) {
            // mov [edi], eax
            compiledOfs -= 2;
            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
            return;
        }
        if pop1 == OP_DIVI as c_int || pop1 == OP_DIVU as c_int || pop1 == OP_MULI as c_int || pop1 == OP_MULU as c_int ||
           pop1 == OP_STORE4 as c_int || pop1 == OP_STORE2 as c_int || pop1 == OP_STORE1 as c_int {
            return;
        }
        if pop1 == OP_CONST as c_int &&
           *buf.offset((compiledOfs - 6) as isize) == 0xC7 &&
           *buf.offset((compiledOfs - 5) as isize) == 0x07 {
            // mov edi, 0x123456
            compiledOfs -= 6;
            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
            EmitString(b"B8\0".as_ptr() as *const c_char);  // mov	eax, 0x12345678
            Emit4(lastConst);
            return;
        }
        EmitString(b"8B 07\0".as_ptr() as *const c_char);  // mov eax, dword ptr [edi]
    }
}

static fn EmitMovEBXEDI(vm: *mut vm_t, andit: c_int) -> bool {
    unsafe {
        if matches!(LastCommand, ELastCommand::LAST_COMMAND_MOV_EDI_EAX) {
            // mov [edi], eax
            compiledOfs -= 2;
            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
            EmitString(b"8B D8\0".as_ptr() as *const c_char);  // mov bx, eax
            return false;
        }
        if pop1 == OP_DIVI as c_int || pop1 == OP_DIVU as c_int || pop1 == OP_MULI as c_int || pop1 == OP_MULU as c_int ||
           pop1 == OP_STORE4 as c_int || pop1 == OP_STORE2 as c_int || pop1 == OP_STORE1 as c_int {
            EmitString(b"8B D8\0".as_ptr() as *const c_char);  // mov bx, eax
            return false;
        }
        if pop1 == OP_CONST as c_int &&
           *buf.offset((compiledOfs - 6) as isize) == 0xC7 &&
           *buf.offset((compiledOfs - 5) as isize) == 0x07 {
            // mov edi, 0x123456
            compiledOfs -= 6;
            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
            EmitString(b"BB\0".as_ptr() as *const c_char);  // mov	ebx, 0x12345678
            if andit != 0 {
                Emit4(lastConst & andit);
            } else {
                Emit4(lastConst);
            }
            return true;
        }

        EmitString(b"8B 1F\0".as_ptr() as *const c_char);  // mov ebx, dword ptr [edi]
        false
    }
}

/*
=================
VM_Compile
=================
*/
pub unsafe fn VM_Compile(vm: *mut vm_t, header: *mut vmHeader_t) {
    let mut op: c_int;
    let mut maxLength: c_int;
    let mut v: c_int;
    let mut i: c_int;
    let mut opt: bool;

    // allocate a very large temp buffer, we will shrink it later
    maxLength = (*header).codeLength * 8;
    buf = Z_Malloc(maxLength as usize, TAG_VM, 1) as *mut u8;
    jused = Z_Malloc(((*header).instructionCount + 2) as usize, TAG_VM, 1) as *mut u8;

    Com_Memset(jused as *mut c_void, 0, ((*header).instructionCount + 2) as usize);

    for _ in 0..2 {
        oc0 = -23423;
        oc1 = -234354;
        pop0 = -43435;
        pop1 = -545455;

        // translate all instructions
        pc = 0;
        instruction = 0;
        code = (header as *mut u8).offset((*header).codeOffset as isize);
        compiledOfs = 0;

        LastCommand = ELastCommand::LAST_COMMAND_NONE;

        while instruction < (*header).instructionCount {
            if compiledOfs > maxLength - 16 {
                Com_Error(ERR_FATAL, b"VM_CompileX86: maxLength exceeded\0".as_ptr() as *const c_char);
            }

            *(*vm).instructionPointers.offset(instruction as isize) = compiledOfs;
            instruction += 1;

            if pc > (*header).codeLength {
                Com_Error(ERR_FATAL, b"VM_CompileX86: pc > header->codeLength\0".as_ptr() as *const c_char);
            }

            op = *code.offset(pc as isize) as c_int;
            pc += 1;
            match op as u8 {
                0 => {}
                OP_BREAK => {
                    EmitString(b"CC\0".as_ptr() as *const c_char);  // int 3
                }
                OP_ENTER => {
                    EmitString(b"81 EE\0".as_ptr() as *const c_char);  // sub	esi, 0x12345678
                    Emit4(Constant4());
                }
                OP_CONST => {
                    if *code.offset((pc + 4) as isize) == OP_LOAD4 {
                        EmitAddEDI4(vm);
                        EmitString(b"BB\0".as_ptr() as *const c_char);  // mov	ebx, 0x12345678
                        Emit4((Constant4() & (*vm).dataMask) + (*vm).dataBase as c_int);
                        EmitString(b"8B 03\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [ebx]
                        EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                        pc += 1;  // OP_LOAD4
                        instruction += 1;
                        continue;
                    }
                    if *code.offset((pc + 4) as isize) == OP_LOAD2 {
                        EmitAddEDI4(vm);
                        EmitString(b"BB\0".as_ptr() as *const c_char);  // mov	ebx, 0x12345678
                        Emit4((Constant4() & (*vm).dataMask) + (*vm).dataBase as c_int);
                        EmitString(b"0F B7 03\0".as_ptr() as *const c_char);  // movzx	eax, word ptr [ebx]
                        EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                        pc += 1;  // OP_LOAD4
                        instruction += 1;
                        continue;
                    }
                    if *code.offset((pc + 4) as isize) == OP_LOAD1 {
                        EmitAddEDI4(vm);
                        EmitString(b"BB\0".as_ptr() as *const c_char);  // mov	ebx, 0x12345678
                        Emit4((Constant4() & (*vm).dataMask) + (*vm).dataBase as c_int);
                        EmitString(b"0F B6 03\0".as_ptr() as *const c_char);  // movzx	eax, byte ptr [ebx]
                        EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                        pc += 1;  // OP_LOAD4
                        instruction += 1;
                        continue;
                    }
                    if *code.offset((pc + 4) as isize) == OP_STORE4 {
                        opt = EmitMovEBXEDI(vm, ((*vm).dataMask & !3));
                        EmitString(b"B8\0".as_ptr() as *const c_char);  // mov	eax, 0x12345678
                        Emit4(Constant4());
                        EmitString(b"89 83\0".as_ptr() as *const c_char);  // mov dword ptr [ebx+0x12345678], eax
                        Emit4((*vm).dataBase as c_int);
                        EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                        pc += 1;  // OP_STORE4
                        instruction += 1;
                        continue;
                    }
                    if *code.offset((pc + 4) as isize) == OP_STORE2 {
                        opt = EmitMovEBXEDI(vm, ((*vm).dataMask & !1));
                        EmitString(b"B8\0".as_ptr() as *const c_char);  // mov	eax, 0x12345678
                        Emit4(Constant4());
                        EmitString(b"66 89 83\0".as_ptr() as *const c_char);  // mov word ptr [ebx+0x12345678], eax
                        Emit4((*vm).dataBase as c_int);
                        EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                        pc += 1;  // OP_STORE4
                        instruction += 1;
                        continue;
                    }
                    if *code.offset((pc + 4) as isize) == OP_STORE1 {
                        opt = EmitMovEBXEDI(vm, (*vm).dataMask);
                        EmitString(b"B8\0".as_ptr() as *const c_char);  // mov	eax, 0x12345678
                        Emit4(Constant4());
                        EmitString(b"88 83\0".as_ptr() as *const c_char);  // mov byte ptr [ebx+0x12345678], eax
                        Emit4((*vm).dataBase as c_int);
                        EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                        pc += 1;  // OP_STORE4
                        instruction += 1;
                        continue;
                    }
                    if *code.offset((pc + 4) as isize) == OP_ADD {
                        EmitString(b"81 07\0".as_ptr() as *const c_char);  // add dword ptr [edi], 0x1234567
                        Emit4(Constant4());
                        pc += 1;  // OP_ADD
                        instruction += 1;
                        continue;
                    }
                    if *code.offset((pc + 4) as isize) == OP_SUB {
                        EmitString(b"81 2F\0".as_ptr() as *const c_char);  // sub dword ptr [edi], 0x1234567
                        Emit4(Constant4());
                        pc += 1;  // OP_ADD
                        instruction += 1;
                        continue;
                    }
                    EmitAddEDI4(vm);
                    EmitString(b"C7 07\0".as_ptr() as *const c_char);  // mov	dword ptr [edi], 0x12345678
                    lastConst = Constant4();
                    Emit4(lastConst);
                    if *code.offset(pc as isize) == OP_JUMP {
                        *jused.offset(lastConst as isize) = 1;
                    }
                }
                OP_LOCAL => {
                    EmitAddEDI4(vm);
                    EmitString(b"8D 86\0".as_ptr() as *const c_char);  // lea eax, [0x12345678 + esi]
                    oc0 = oc1;
                    oc1 = Constant4();
                    Emit4(oc1);
                    EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                }
                OP_ARG => {
                    EmitMovEAXEDI(vm);  // mov	eax,dword ptr [edi]
                    EmitString(b"89 86\0".as_ptr() as *const c_char);  // mov	dword ptr [esi+database],eax
                    // FIXME: range check
                    Emit4(Constant1() + (*vm).dataBase as c_int);
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_CALL => {
                    EmitString(b"C7 86\0".as_ptr() as *const c_char);  // mov dword ptr [esi+database],0x12345678
                    Emit4((*vm).dataBase as c_int);
                    Emit4(pc);
                    EmitString(b"FF 15\0".as_ptr() as *const c_char);  // call asmCallPtr
                    Emit4(core::ptr::addr_of_mut!(asmCallPtr) as c_int);
                }
                OP_PUSH => {
                    EmitAddEDI4(vm);
                }
                OP_POP => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_LEAVE => {
                    v = Constant4();
                    EmitString(b"81 C6\0".as_ptr() as *const c_char);  // add	esi, 0x12345678
                    Emit4(v);
                    EmitString(b"C3\0".as_ptr() as *const c_char);  // ret
                }
                OP_LOAD4 => {
                    if *code.offset(pc as isize) == OP_CONST &&
                       *code.offset((pc + 5) as isize) == OP_ADD &&
                       *code.offset((pc + 6) as isize) == OP_STORE4 {
                        if oc0 == oc1 && pop0 == OP_LOCAL as c_int && pop1 == OP_LOCAL as c_int {
                            compiledOfs -= 11;
                            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
                        }
                        pc += 1;  // OP_CONST
                        v = Constant4();
                        EmitMovEBXEDI(vm, (*vm).dataMask);
                        if v == 1 && oc0 == oc1 && pop0 == OP_LOCAL as c_int && pop1 == OP_LOCAL as c_int {
                            EmitString(b"FF 83\0".as_ptr() as *const c_char);  // inc dword ptr [ebx + 0x12345678]
                            Emit4((*vm).dataBase as c_int);
                        } else {
                            EmitString(b"8B 83\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [ebx + 0x12345678]
                            Emit4((*vm).dataBase as c_int);
                            EmitString(b"05\0".as_ptr() as *const c_char);  // add eax, const
                            Emit4(v);
                            if oc0 == oc1 && pop0 == OP_LOCAL as c_int && pop1 == OP_LOCAL as c_int {
                                EmitString(b"89 83\0".as_ptr() as *const c_char);  // mov dword ptr [ebx+0x12345678], eax
                                Emit4((*vm).dataBase as c_int);
                            } else {
                                EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                                EmitString(b"8B 1F\0".as_ptr() as *const c_char);  // mov	ebx, dword ptr [edi]
                                EmitString(b"89 83\0".as_ptr() as *const c_char);  // mov dword ptr [ebx+0x12345678], eax
                                Emit4((*vm).dataBase as c_int);
                            }
                        }
                        EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                        pc += 1;  // OP_ADD
                        pc += 1;  // OP_STORE
                        instruction += 3;
                        continue;
                    }

                    if *code.offset(pc as isize) == OP_CONST &&
                       *code.offset((pc + 5) as isize) == OP_SUB &&
                       *code.offset((pc + 6) as isize) == OP_STORE4 {
                        if oc0 == oc1 && pop0 == OP_LOCAL as c_int && pop1 == OP_LOCAL as c_int {
                            compiledOfs -= 11;
                            *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
                        }
                        EmitMovEBXEDI(vm, (*vm).dataMask);
                        EmitString(b"8B 83\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [ebx + 0x12345678]
                        Emit4((*vm).dataBase as c_int);
                        pc += 1;  // OP_CONST
                        v = Constant4();
                        if v == 1 && oc0 == oc1 && pop0 == OP_LOCAL as c_int && pop1 == OP_LOCAL as c_int {
                            EmitString(b"FF 8B\0".as_ptr() as *const c_char);  // dec dword ptr [ebx + 0x12345678]
                            Emit4((*vm).dataBase as c_int);
                        } else {
                            EmitString(b"2D\0".as_ptr() as *const c_char);  // sub eax, const
                            Emit4(v);
                            if oc0 == oc1 && pop0 == OP_LOCAL as c_int && pop1 == OP_LOCAL as c_int {
                                EmitString(b"89 83\0".as_ptr() as *const c_char);  // mov dword ptr [ebx+0x12345678], eax
                                Emit4((*vm).dataBase as c_int);
                            } else {
                                EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                                EmitString(b"8B 1F\0".as_ptr() as *const c_char);  // mov	ebx, dword ptr [edi]
                                EmitString(b"89 83\0".as_ptr() as *const c_char);  // mov dword ptr [ebx+0x12345678], eax
                                Emit4((*vm).dataBase as c_int);
                            }
                        }
                        EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                        pc += 1;  // OP_SUB
                        pc += 1;  // OP_STORE
                        instruction += 3;
                        continue;
                    }

                    if *buf.offset((compiledOfs - 2) as isize) == 0x89 &&
                       *buf.offset((compiledOfs - 1) as isize) == 0x07 {
                        compiledOfs -= 2;
                        *(*vm).instructionPointers.offset((instruction - 1) as isize) = compiledOfs;
                        EmitString(b"8B 80\0".as_ptr() as *const c_char);  // mov eax, dword ptr [eax + 0x1234567]
                        Emit4((*vm).dataBase as c_int);
                        EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                        continue;
                    }
                    EmitMovEBXEDI(vm, (*vm).dataMask);
                    EmitString(b"8B 83\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [ebx + 0x12345678]
                    Emit4((*vm).dataBase as c_int);
                    EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                }
                OP_LOAD2 => {
                    EmitMovEBXEDI(vm, (*vm).dataMask);
                    EmitString(b"0F B7 83\0".as_ptr() as *const c_char);  // movzx	eax, word ptr [ebx + 0x12345678]
                    Emit4((*vm).dataBase as c_int);
                    EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                }
                OP_LOAD1 => {
                    EmitMovEBXEDI(vm, (*vm).dataMask);
                    EmitString(b"0F B6 83\0".as_ptr() as *const c_char);  // movzx eax, byte ptr [ebx + 0x12345678]
                    Emit4((*vm).dataBase as c_int);
                    EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                }
                OP_STORE4 => {
                    EmitMovEAXEDI(vm);
                    EmitString(b"8B 5F FC\0".as_ptr() as *const c_char);  // mov	ebx, dword ptr [edi-4]
                    EmitString(b"89 83\0".as_ptr() as *const c_char);  // mov dword ptr [ebx+0x12345678], eax
                    Emit4((*vm).dataBase as c_int);
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                }
                OP_STORE2 => {
                    EmitMovEAXEDI(vm);
                    EmitString(b"8B 5F FC\0".as_ptr() as *const c_char);  // mov	ebx, dword ptr [edi-4]
                    EmitString(b"66 89 83\0".as_ptr() as *const c_char);  // mov word ptr [ebx+0x12345678], eax
                    Emit4((*vm).dataBase as c_int);
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                }
                OP_STORE1 => {
                    EmitMovEAXEDI(vm);
                    EmitString(b"8B 5F FC\0".as_ptr() as *const c_char);  // mov	ebx, dword ptr [edi-4]
                    EmitString(b"88 83\0".as_ptr() as *const c_char);  // mov byte ptr [ebx+0x12345678], eax
                    Emit4((*vm).dataBase as c_int);
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                }

                OP_EQ => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"75 06\0".as_ptr() as *const c_char);  // jne +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_NE => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"74 06\0".as_ptr() as *const c_char);  // je +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_LTI => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"7D 06\0".as_ptr() as *const c_char);  // jnl +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_LEI => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"7F 06\0".as_ptr() as *const c_char);  // jnle +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_GTI => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"7E 06\0".as_ptr() as *const c_char);  // jng +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_GEI => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"7C 06\0".as_ptr() as *const c_char);  // jnge +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_LTU => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"73 06\0".as_ptr() as *const c_char);  // jnb +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_LEU => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"77 06\0".as_ptr() as *const c_char);  // jnbe +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_GTU => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"76 06\0".as_ptr() as *const c_char);  // jna +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_GEU => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov	eax, dword ptr [edi+4]
                    EmitString(b"3B 47 08\0".as_ptr() as *const c_char);  // cmp	eax, dword ptr [edi+8]
                    EmitString(b"72 06\0".as_ptr() as *const c_char);  // jnae +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_EQF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"D9 47 04\0".as_ptr() as *const c_char);  // fld dword ptr [edi+4]
                    EmitString(b"D8 5F 08\0".as_ptr() as *const c_char);  // fcomp dword ptr [edi+8]
                    EmitString(b"DF E0\0".as_ptr() as *const c_char);  // fnstsw ax
                    EmitString(b"F6 C4 40\0".as_ptr() as *const c_char);  // test	ah,0x40
                    EmitString(b"74 06\0".as_ptr() as *const c_char);  // je +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_NEF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"D9 47 04\0".as_ptr() as *const c_char);  // fld dword ptr [edi+4]
                    EmitString(b"D8 5F 08\0".as_ptr() as *const c_char);  // fcomp dword ptr [edi+8]
                    EmitString(b"DF E0\0".as_ptr() as *const c_char);  // fnstsw ax
                    EmitString(b"F6 C4 40\0".as_ptr() as *const c_char);  // test	ah,0x40
                    EmitString(b"75 06\0".as_ptr() as *const c_char);  // jne +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_LTF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"D9 47 04\0".as_ptr() as *const c_char);  // fld dword ptr [edi+4]
                    EmitString(b"D8 5F 08\0".as_ptr() as *const c_char);  // fcomp dword ptr [edi+8]
                    EmitString(b"DF E0\0".as_ptr() as *const c_char);  // fnstsw ax
                    EmitString(b"F6 C4 01\0".as_ptr() as *const c_char);  // test	ah,0x01
                    EmitString(b"74 06\0".as_ptr() as *const c_char);  // je +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_LEF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"D9 47 04\0".as_ptr() as *const c_char);  // fld dword ptr [edi+4]
                    EmitString(b"D8 5F 08\0".as_ptr() as *const c_char);  // fcomp dword ptr [edi+8]
                    EmitString(b"DF E0\0".as_ptr() as *const c_char);  // fnstsw ax
                    EmitString(b"F6 C4 41\0".as_ptr() as *const c_char);  // test	ah,0x41
                    EmitString(b"74 06\0".as_ptr() as *const c_char);  // je +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_GTF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"D9 47 04\0".as_ptr() as *const c_char);  // fld dword ptr [edi+4]
                    EmitString(b"D8 5F 08\0".as_ptr() as *const c_char);  // fcomp dword ptr [edi+8]
                    EmitString(b"DF E0\0".as_ptr() as *const c_char);  // fnstsw ax
                    EmitString(b"F6 C4 41\0".as_ptr() as *const c_char);  // test	ah,0x41
                    EmitString(b"75 06\0".as_ptr() as *const c_char);  // jne +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_GEF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                    EmitString(b"D9 47 04\0".as_ptr() as *const c_char);  // fld dword ptr [edi+4]
                    EmitString(b"D8 5F 08\0".as_ptr() as *const c_char);  // fcomp dword ptr [edi+8]
                    EmitString(b"DF E0\0".as_ptr() as *const c_char);  // fnstsw ax
                    EmitString(b"F6 C4 01\0".as_ptr() as *const c_char);  // test	ah,0x01
                    EmitString(b"75 06\0".as_ptr() as *const c_char);  // jne +6
                    EmitString(b"FF 25\0".as_ptr() as *const c_char);  // jmp	[0x12345678]
                    v = Constant4();
                    *jused.offset(v as isize) = 1;
                    Emit4(core::ptr::addr_of!((*(*vm).instructionPointers.offset(v as isize))) as c_int);
                }
                OP_NEGI => {
                    EmitString(b"F7 1F\0".as_ptr() as *const c_char);  // neg dword ptr [edi]
                }
                OP_ADD => {
                    EmitMovEAXEDI(vm);  // mov eax, dword ptr [edi]
                    EmitString(b"01 47 FC\0".as_ptr() as *const c_char);  // add dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_SUB => {
                    EmitMovEAXEDI(vm);  // mov eax, dword ptr [edi]
                    EmitString(b"29 47 FC\0".as_ptr() as *const c_char);  // sub dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_DIVI => {
                    EmitString(b"8B 47 FC\0".as_ptr() as *const c_char);  // mov eax,dword ptr [edi-4]
                    EmitString(b"99\0".as_ptr() as *const c_char);  // cdq
                    EmitString(b"F7 3F\0".as_ptr() as *const c_char);  // idiv dword ptr [edi]
                    EmitString(b"89 47 FC\0".as_ptr() as *const c_char);  // mov dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_DIVU => {
                    EmitString(b"8B 47 FC\0".as_ptr() as *const c_char);  // mov eax,dword ptr [edi-4]
                    EmitString(b"33 D2\0".as_ptr() as *const c_char);  // xor edx, edx
                    EmitString(b"F7 37\0".as_ptr() as *const c_char);  // div dword ptr [edi]
                    EmitString(b"89 47 FC\0".as_ptr() as *const c_char);  // mov dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_MODI => {
                    EmitString(b"8B 47 FC\0".as_ptr() as *const c_char);  // mov eax,dword ptr [edi-4]
                    EmitString(b"99\0".as_ptr() as *const c_char);  // cdq
                    EmitString(b"F7 3F\0".as_ptr() as *const c_char);  // idiv dword ptr [edi]
                    EmitString(b"89 57 FC\0".as_ptr() as *const c_char);  // mov dword ptr [edi-4],edx
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_MODU => {
                    EmitString(b"8B 47 FC\0".as_ptr() as *const c_char);  // mov eax,dword ptr [edi-4]
                    EmitString(b"33 D2\0".as_ptr() as *const c_char);  // xor edx, edx
                    EmitString(b"F7 37\0".as_ptr() as *const c_char);  // div dword ptr [edi]
                    EmitString(b"89 57 FC\0".as_ptr() as *const c_char);  // mov dword ptr [edi-4],edx
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_MULI => {
                    EmitString(b"8B 47 FC\0".as_ptr() as *const c_char);  // mov eax,dword ptr [edi-4]
                    EmitString(b"F7 2F\0".as_ptr() as *const c_char);  // imul dword ptr [edi]
                    EmitString(b"89 47 FC\0".as_ptr() as *const c_char);  // mov dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_MULU => {
                    EmitString(b"8B 47 FC\0".as_ptr() as *const c_char);  // mov eax,dword ptr [edi-4]
                    EmitString(b"F7 27\0".as_ptr() as *const c_char);  // mul dword ptr [edi]
                    EmitString(b"89 47 FC\0".as_ptr() as *const c_char);  // mov dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_BAND => {
                    EmitMovEAXEDI(vm);  // mov eax, dword ptr [edi]
                    EmitString(b"21 47 FC\0".as_ptr() as *const c_char);  // and dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_BOR => {
                    EmitMovEAXEDI(vm);  // mov eax, dword ptr [edi]
                    EmitString(b"09 47 FC\0".as_ptr() as *const c_char);  // or dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_BXOR => {
                    EmitMovEAXEDI(vm);  // mov eax, dword ptr [edi]
                    EmitString(b"31 47 FC\0".as_ptr() as *const c_char);  // xor dword ptr [edi-4],eax
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_BCOM => {
                    EmitString(b"F7 17\0".as_ptr() as *const c_char);  // not dword ptr [edi]
                }
                OP_LSH => {
                    EmitString(b"8B 0F\0".as_ptr() as *const c_char);  // mov ecx, dword ptr [edi]
                    EmitString(b"D3 67 FC\0".as_ptr() as *const c_char);  // shl dword ptr [edi-4], cl
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_RSHI => {
                    EmitString(b"8B 0F\0".as_ptr() as *const c_char);  // mov ecx, dword ptr [edi]
                    EmitString(b"D3 7F FC\0".as_ptr() as *const c_char);  // sar dword ptr [edi-4], cl
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_RSHU => {
                    EmitString(b"8B 0F\0".as_ptr() as *const c_char);  // mov ecx, dword ptr [edi]
                    EmitString(b"D3 6F FC\0".as_ptr() as *const c_char);  // shr dword ptr [edi-4], cl
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_NEGF => {
                    EmitString(b"D9 07\0".as_ptr() as *const c_char);  // fld dword ptr [edi]
                    EmitString(b"D9 E0\0".as_ptr() as *const c_char);  // fchs
                    EmitString(b"D9 1F\0".as_ptr() as *const c_char);  // fstp dword ptr [edi]
                }
                OP_ADDF => {
                    EmitString(b"D9 47 FC\0".as_ptr() as *const c_char);  // fld dword ptr [edi-4]
                    EmitString(b"D8 07\0".as_ptr() as *const c_char);  // fadd dword ptr [edi]
                    EmitString(b"D9 5F FC\0".as_ptr() as *const c_char);  // fstp dword ptr [edi-4]
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                }
                OP_SUBF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                    EmitString(b"D9 07\0".as_ptr() as *const c_char);  // fld dword ptr [edi]
                    EmitString(b"D8 67 04\0".as_ptr() as *const c_char);  // fsub dword ptr [edi+4]
                    EmitString(b"D9 1F\0".as_ptr() as *const c_char);  // fstp dword ptr [edi]
                }
                OP_DIVF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                    EmitString(b"D9 07\0".as_ptr() as *const c_char);  // fld dword ptr [edi]
                    EmitString(b"D8 77 04\0".as_ptr() as *const c_char);  // fdiv dword ptr [edi+4]
                    EmitString(b"D9 1F\0".as_ptr() as *const c_char);  // fstp dword ptr [edi]
                }
                OP_MULF => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                    EmitString(b"D9 07\0".as_ptr() as *const c_char);  // fld dword ptr [edi]
                    EmitString(b"D8 4f 04\0".as_ptr() as *const c_char);  // fmul dword ptr [edi+4]
                    EmitString(b"D9 1F\0".as_ptr() as *const c_char);  // fstp dword ptr [edi]
                }
                OP_CVIF => {
                    EmitString(b"DB 07\0".as_ptr() as *const c_char);  // fild dword ptr [edi]
                    EmitString(b"D9 1F\0".as_ptr() as *const c_char);  // fstp dword ptr [edi]
                }
                OP_CVFI => {
                    #[cfg(not(feature = "FTOL_PTR"))]
                    {
                        // not IEEE complient, but simple and fast
                        EmitString(b"D9 07\0".as_ptr() as *const c_char);  // fld dword ptr [edi]
                        EmitString(b"DB 1F\0".as_ptr() as *const c_char);  // fistp dword ptr [edi]
                    }
                    #[cfg(feature = "FTOL_PTR")]
                    {
                        // call the library conversion function
                        EmitString(b"D9 07\0".as_ptr() as *const c_char);  // fld dword ptr [edi]
                        EmitString(b"FF 15\0".as_ptr() as *const c_char);  // call ftolPtr
                        Emit4(core::ptr::addr_of_mut!(ftolPtr) as c_int);
                        EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                    }
                }
                OP_SEX8 => {
                    EmitString(b"0F BE 07\0".as_ptr() as *const c_char);  // movsx eax, byte ptr [edi]
                    EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                }
                OP_SEX16 => {
                    EmitString(b"0F BF 07\0".as_ptr() as *const c_char);  // movsx eax, word ptr [edi]
                    EmitCommand(ELastCommand::LAST_COMMAND_MOV_EDI_EAX);  // mov dword ptr [edi], eax
                }

                OP_BLOCK_COPY => {
                    // FIXME: range check
                    EmitString(b"56\0".as_ptr() as *const c_char);  // push esi
                    EmitString(b"57\0".as_ptr() as *const c_char);  // push edi
                    EmitString(b"8B 37\0".as_ptr() as *const c_char);  // mov esi,[edi]
                    EmitString(b"8B 7F FC\0".as_ptr() as *const c_char);  // mov edi,[edi-4]
                    EmitString(b"B9\0".as_ptr() as *const c_char);  // mov ecx,0x12345678
                    Emit4(Constant4() >> 2);
                    EmitString(b"B8\0".as_ptr() as *const c_char);  // mov eax, datamask
                    Emit4((*vm).dataMask);
                    EmitString(b"BB\0".as_ptr() as *const c_char);  // mov ebx, database
                    Emit4((*vm).dataBase as c_int);
                    EmitString(b"23 F0\0".as_ptr() as *const c_char);  // and esi, eax
                    EmitString(b"03 F3\0".as_ptr() as *const c_char);  // add esi, ebx
                    EmitString(b"23 F8\0".as_ptr() as *const c_char);  // and edi, eax
                    EmitString(b"03 FB\0".as_ptr() as *const c_char);  // add edi, ebx
                    EmitString(b"F3 A5\0".as_ptr() as *const c_char);  // rep movsd
                    EmitString(b"5F\0".as_ptr() as *const c_char);  // pop edi
                    EmitString(b"5E\0".as_ptr() as *const c_char);  // pop esi
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_8);  // sub edi, 8
                }

                OP_JUMP => {
                    EmitCommand(ELastCommand::LAST_COMMAND_SUB_DI_4);  // sub edi, 4
                    EmitString(b"8B 47 04\0".as_ptr() as *const c_char);  // mov eax,dword ptr [edi+4]
                    // FIXME: range check
                    EmitString(b"FF 24 85\0".as_ptr() as *const c_char);  // jmp dword ptr [instructionPointers + eax * 4]
                    Emit4((*vm).instructionPointers as c_int);
                }
                _ => {
                    Com_Error(ERR_DROP, b"VM_CompileX86: bad opcode %i at offset %i\0".as_ptr() as *const c_char, op, pc);
                }
            }
            pop0 = pop1;
            pop1 = op;
        }
    }

    // copy to an exact size buffer on the hunk
    (*vm).codeLength = compiledOfs;
    (*vm).codeBase = Hunk_Alloc(compiledOfs as usize, h_low);
    Com_Memcpy((*vm).codeBase as *mut c_void, buf as *const c_void, compiledOfs as usize);
    Z_Free(buf as *mut c_void);
    Z_Free(jused as *mut c_void);
    Com_Printf(b"VM file %s compiled to %i bytes of code\n\0".as_ptr() as *const c_char, (*vm).name.as_ptr(), compiledOfs);

    // offset all the instruction pointers for the new location
    for i in 0..(*header).instructionCount {
        *(*vm).instructionPointers.offset(i as isize) += (*vm).codeBase as c_int;
    }

    #[cfg(any())]  // disabled mprotect code
    {
        // Must make the newly generated code executable
        // int r;
        // unsigned long addr;
        // int psize = getpagesize();
        // addr = ((int)vm->codeBase & ~(psize-1)) - psize;
        // r = mprotect((char*)addr, vm->codeLength + (int)vm->codeBase - addr + psize,
        //     PROT_READ | PROT_WRITE | PROT_EXEC );
        // if (r < 0)
        //     Com_Error( ERR_FATAL, "mprotect failed to change PROT_EXEC" );
    }
}

/*
==============
VM_CallCompiled

This function is called directly by the generated code
==============
*/
#[cfg(not(any()))]  // ifndef DLL_ONLY
pub unsafe fn VM_CallCompiled(vm: *mut vm_t, args: *mut c_int) -> c_int {
    let mut stack: [c_int; 1024] = [0; 1024];
    let mut programCounter: c_int;
    let mut programStack: c_int;
    let mut stackOnEntry: c_int;
    let mut image: *mut u8;
    let mut entryPoint: *mut c_void;
    let mut opStack: *mut c_void;
    let mut oldInstructionPointers: *mut c_int;

    oldInstructionPointers = instructionPointers;

    currentVM = vm;
    instructionPointers = (*vm).instructionPointers;

    // interpret the code
    (*vm).currentlyInterpreting = 1;

    callMask = (*vm).dataMask;

    // we might be called recursively, so this might not be the very top
    programStack = (*vm).programStack;
    stackOnEntry = programStack;

    // set up the stack frame
    image = (*vm).dataBase;

    programCounter = 0;

    programStack -= 48;

    *(image.offset((programStack + 44) as isize) as *mut c_int) = *args.offset(9);
    *(image.offset((programStack + 40) as isize) as *mut c_int) = *args.offset(8);
    *(image.offset((programStack + 36) as isize) as *mut c_int) = *args.offset(7);
    *(image.offset((programStack + 32) as isize) as *mut c_int) = *args.offset(6);
    *(image.offset((programStack + 28) as isize) as *mut c_int) = *args.offset(5);
    *(image.offset((programStack + 24) as isize) as *mut c_int) = *args.offset(4);
    *(image.offset((programStack + 20) as isize) as *mut c_int) = *args.offset(3);
    *(image.offset((programStack + 16) as isize) as *mut c_int) = *args.offset(2);
    *(image.offset((programStack + 12) as isize) as *mut c_int) = *args.offset(1);
    *(image.offset((programStack + 8) as isize) as *mut c_int) = *args.offset(0);
    *(image.offset((programStack + 4) as isize) as *mut c_int) = 0;  // return stack
    *(image.offset(programStack as isize) as *mut c_int) = -1;  // will terminate the loop on return

    // off we go into generated code...
    entryPoint = (*vm).codeBase as *mut c_void;
    opStack = stack.as_mut_ptr() as *mut c_void;

    #[cfg(target_os = "windows")]
    {
        // Windows x86 assembly: pushad, set registers, call, popad
        // Stub: actual asm implementation needed
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix x86 assembly: pushal, set registers, call, popal
        // Stub: actual asm implementation needed
    }

    if opStack != (stack.as_mut_ptr().offset(1) as *mut c_void) {
        Com_Error(ERR_DROP, b"opStack corrupted in compiled code\0".as_ptr() as *const c_char);
    }
    if programStack != stackOnEntry - 48 {
        Com_Error(ERR_DROP, b"programStack corrupted in compiled code\0".as_ptr() as *const c_char);
    }

    (*vm).programStack = stackOnEntry;

    // in case we were recursively called by another vm
    instructionPointers = oldInstructionPointers;

    *(opStack as *mut c_int)
}
