// vm_ppc.c
// ppc dynamic compiler

use core::ffi::{c_int, c_char, c_void};
use std::ptr;

// #pragma opt_pointer_analysis off

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum regNums_t {
    R_REAL_STACK = 1,
    // registers 3-11 are the parameter passing registers

    // state
    R_STACK = 3,           // local
    R_OPSTACK = 4,         // global

    // constants
    R_MEMBASE = 5,         // global
    R_MEMMASK = 6,
    R_ASMCALL = 7,         // global
    R_INSTRUCTIONS = 8,    // global
    R_NUM_INSTRUCTIONS = 9,// global
    R_CVM = 10,            // currentVM

    // temps
    R_TOP = 12,
    R_SECOND = 13,
    R_EA = 14               // effective address calculation
}

// #define	RG_REAL_STACK		r1
// #define	RG_STACK			r3
// #define	RG_OPSTACK			r4
// #define	RG_MEMBASE			r5
// #define	RG_MEMMASK			r6
// #define	RG_ASMCALL			r7
// #define	RG_INSTRUCTIONS		r8
// #define	RG_NUM_INSTRUCTIONS	r9
// #define	RG_CVM				r10
// #define	RG_TOP				r12
// #define	RG_SECOND			r13
// #define	RG_EA 				r14

// this doesn't have the low order bits set for instructions i'm not using...
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ppcOpcodes_t {
    PPC_TDI         = 0x08000000,
    PPC_TWI         = 0x0c000000,
    PPC_MULLI       = 0x1c000000,
    PPC_SUBFIC      = 0x20000000,
    PPC_CMPI        = 0x28000000,
    PPC_CMPLI       = 0x2c000000,
    PPC_ADDIC       = 0x30000000,
    PPC_ADDIC_      = 0x34000000,
    PPC_ADDI        = 0x38000000,
    PPC_ADDIS       = 0x3c000000,
    PPC_BC          = 0x40000000,
    PPC_SC          = 0x44000000,
    PPC_B           = 0x48000000,

    PPC_MCRF        = 0x4c000000,
    PPC_BCLR        = 0x4c000020,
    PPC_RFID        = 0x4c000000,
    PPC_CRNOR       = 0x4c000000,
    PPC_RFI         = 0x4c000000,
    PPC_CRANDC      = 0x4c000000,
    PPC_ISYNC       = 0x4c000000,
    PPC_CRXOR       = 0x4c000000,
    PPC_CRNAND      = 0x4c000000,
    PPC_CREQV       = 0x4c000000,
    PPC_CRORC       = 0x4c000000,
    PPC_CROR        = 0x4c000000,
    //------------
    PPC_BCCTR       = 0x4c000420,
    PPC_RLWIMI      = 0x50000000,
    PPC_RLWINM      = 0x54000000,
    PPC_RLWNM       = 0x5c000000,
    PPC_ORI         = 0x60000000,
    PPC_ORIS        = 0x64000000,
    PPC_XORI        = 0x68000000,
    PPC_XORIS       = 0x6c000000,
    PPC_ANDI_       = 0x70000000,
    PPC_ANDIS_      = 0x74000000,
    PPC_RLDICL      = 0x78000000,
    PPC_RLDICR      = 0x78000000,
    PPC_RLDIC       = 0x78000000,
    PPC_RLDIMI      = 0x78000000,
    PPC_RLDCL       = 0x78000000,
    PPC_RLDCR       = 0x78000000,
    PPC_CMP         = 0x7c000000,
    PPC_TW          = 0x7c000000,
    PPC_SUBFC       = 0x7c000010,
    PPC_MULHDU      = 0x7c000000,
    PPC_ADDC        = 0x7c000014,
    PPC_MULHWU      = 0x7c000000,
    PPC_MFCR        = 0x7c000000,
    PPC_LWAR        = 0x7c000000,
    PPC_LDX         = 0x7c000000,
    PPC_LWZX        = 0x7c00002e,
    PPC_SLW         = 0x7c000030,
    PPC_CNTLZW      = 0x7c000000,
    PPC_SLD         = 0x7c000000,
    PPC_AND         = 0x7c000038,
    PPC_CMPL        = 0x7c000040,
    PPC_SUBF        = 0x7c000050,
    PPC_LDUX        = 0x7c000000,
    //------------
    PPC_DCBST       = 0x7c000000,
    PPC_LWZUX       = 0x7c00006c,
    PPC_CNTLZD      = 0x7c000000,
    PPC_ANDC        = 0x7c000000,
    PPC_TD          = 0x7c000000,
    PPC_MULHD       = 0x7c000000,
    PPC_MULHW       = 0x7c000000,
    PPC_MTSRD       = 0x7c000000,
    PPC_MFMSR       = 0x7c000000,
    PPC_LDARX       = 0x7c000000,
    PPC_DCBF        = 0x7c000000,
    PPC_LBZX        = 0x7c0000ae,
    PPC_NEG         = 0x7c000000,
    PPC_MTSRDIN     = 0x7c000000,
    PPC_LBZUX       = 0x7c000000,
    PPC_NOR         = 0x7c0000f8,
    PPC_SUBFE       = 0x7c000000,
    PPC_ADDE        = 0x7c000000,
    PPC_MTCRF       = 0x7c000000,
    PPC_MTMSR       = 0x7c000000,
    PPC_STDX        = 0x7c000000,
    PPC_STWCX_      = 0x7c000000,
    PPC_STWX        = 0x7c00012e,
    PPC_MTMSRD      = 0x7c000000,
    PPC_STDUX       = 0x7c000000,
    PPC_STWUX       = 0x7c00016e,
    PPC_SUBFZE      = 0x7c000000,
    PPC_ADDZE       = 0x7c000000,
    PPC_MTSR        = 0x7c000000,
    PPC_STDCX_      = 0x7c000000,
    PPC_STBX        = 0x7c0001ae,
    PPC_SUBFME      = 0x7c000000,
    PPC_MULLD       = 0x7c000000,
    //------------
    PPC_ADDME       = 0x7c000000,
    PPC_MULLW       = 0x7c0001d6,
    PPC_MTSRIN      = 0x7c000000,
    PPC_DCBTST      = 0x7c000000,
    PPC_STBUX       = 0x7c000000,
    PPC_ADD         = 0x7c000214,
    PPC_DCBT        = 0x7c000000,
    PPC_LHZX        = 0x7c00022e,
    PPC_EQV         = 0x7c000000,
    PPC_TLBIE       = 0x7c000000,
    PPC_ECIWX       = 0x7c000000,
    PPC_LHZUX       = 0x7c000000,
    PPC_XOR         = 0x7c000278,
    PPC_MFSPR       = 0x7c0002a6,
    PPC_LWAX        = 0x7c000000,
    PPC_LHAX        = 0x7c000000,
    PPC_TLBIA       = 0x7c000000,
    PPC_MFTB        = 0x7c000000,
    PPC_LWAUX       = 0x7c000000,
    PPC_LHAUX       = 0x7c000000,
    PPC_STHX        = 0x7c00032e,
    PPC_ORC         = 0x7c000338,
    PPC_SRADI       = 0x7c000000,
    PPC_SLBIE       = 0x7c000000,
    PPC_ECOWX       = 0x7c000000,
    PPC_STHUX       = 0x7c000000,
    PPC_OR          = 0x7c000378,
    PPC_DIVDU       = 0x7c000000,
    PPC_DIVWU       = 0x7c000396,
    PPC_MTSPR       = 0x7c0003a6,
    PPC_DCBI        = 0x7c000000,
    PPC_NAND        = 0x7c000000,
    PPC_DIVD        = 0x7c000000,
    //------------
    PPC_DIVW        = 0x7c0003d6,
    PPC_SLBIA       = 0x7c000000,
    PPC_MCRXR       = 0x7c000000,
    PPC_LSWX        = 0x7c000000,
    PPC_LWBRX       = 0x7c000000,
    PPC_LFSX        = 0x7c000000,
    PPC_SRW         = 0x7c000430,
    PPC_SRD         = 0x7c000000,
    PPC_TLBSYNC     = 0x7c000000,
    PPC_LFSUX       = 0x7c000000,
    PPC_MFSR        = 0x7c000000,
    PPC_LSWI        = 0x7c000000,
    PPC_SYNC        = 0x7c000000,
    PPC_LFDX        = 0x7c000000,
    PPC_LFDUX       = 0x7c000000,
    PPC_MFSRIN      = 0x7c000000,
    PPC_STSWX       = 0x7c000000,
    PPC_STWBRX      = 0x7c000000,
    PPC_STFSX       = 0x7c000000,
    PPC_STFSUX      = 0x7c000000,
    PPC_STSWI       = 0x7c000000,
    PPC_STFDX       = 0x7c000000,
    PPC_DCBA        = 0x7c000000,
    PPC_STFDUX      = 0x7c000000,
    PPC_LHBRX       = 0x7c000000,
    PPC_SRAW        = 0x7c000630,
    PPC_SRAD        = 0x7c000000,
    PPC_SRAWI       = 0x7c000000,
    PPC_EIEIO       = 0x7c000000,
    PPC_STHBRX      = 0x7c000000,
    PPC_EXTSH       = 0x7c000734,
    PPC_EXTSB       = 0x7c000774,
    PPC_ICBI        = 0x7c000000,
    //------------
    PPC_STFIWX      = 0x7c0007ae,
    PPC_EXTSW       = 0x7c000000,
    PPC_DCBZ        = 0x7c000000,
    PPC_LWZ         = 0x80000000,
    PPC_LWZU        = 0x84000000,
    PPC_LBZ         = 0x88000000,
    PPC_LBZU        = 0x8c000000,
    PPC_STW         = 0x90000000,
    PPC_STWU        = 0x94000000,
    PPC_STB         = 0x98000000,
    PPC_STBU        = 0x9c000000,
    PPC_LHZ         = 0xa0000000,
    PPC_LHZU        = 0xa4000000,
    PPC_LHA         = 0xa8000000,
    PPC_LHAU        = 0xac000000,
    PPC_STH         = 0xb0000000,
    PPC_STHU        = 0xb4000000,
    PPC_LMW         = 0xb8000000,
    PPC_STMW        = 0xbc000000,
    PPC_LFS         = 0xc0000000,
    PPC_LFSU        = 0xc4000000,
    PPC_LFD         = 0xc8000000,
    PPC_LFDU        = 0xcc000000,
    PPC_STFS        = 0xd0000000,
    PPC_STFSU       = 0xd4000000,
    PPC_STFD        = 0xd8000000,
    PPC_STFDU       = 0xdc000000,
    PPC_LD          = 0xe8000000,
    PPC_LDU         = 0xe8000001,
    PPC_LWA         = 0xe8000002,
    PPC_FDIVS       = 0xec000024,
    PPC_FSUBS       = 0xec000028,
    PPC_FADDS       = 0xec00002a,
    //------------
    PPC_FSQRTS      = 0xec000000,
    PPC_FRES        = 0xec000000,
    PPC_FMULS       = 0xec000032,
    PPC_FMSUBS      = 0xec000000,
    PPC_FMADDS      = 0xec000000,
    PPC_FNMSUBS     = 0xec000000,
    PPC_FNMADDS     = 0xec000000,
    PPC_STD         = 0xf8000000,
    PPC_STDU        = 0xf8000001,
    PPC_FCMPU       = 0xfc000000,
    PPC_FRSP        = 0xfc000018,
    PPC_FCTIW       = 0xfc000000,
    PPC_FCTIWZ      = 0xfc00001e,
    PPC_FDIV        = 0xfc000000,
    PPC_FSUB        = 0xfc000028,
    PPC_FADD        = 0xfc000000,
    PPC_FSQRT       = 0xfc000000,
    PPC_FSEL        = 0xfc000000,
    PPC_FMUL        = 0xfc000000,
    PPC_FRSQRTE     = 0xfc000000,
    PPC_FMSUB       = 0xfc000000,
    PPC_FMADD       = 0xfc000000,
    PPC_FNMSUB      = 0xfc000000,
    PPC_FNMADD      = 0xfc000000,
    PPC_FCMPO       = 0xfc000000,
    PPC_MTFSB1      = 0xfc000000,
    PPC_FNEG        = 0xfc000050,
    PPC_MCRFS       = 0xfc000000,
    PPC_MTFSB0      = 0xfc000000,
    PPC_FMR         = 0xfc000000,
    PPC_MTFSFI      = 0xfc000000,
    PPC_FNABS       = 0xfc000000,
    PPC_FABS        = 0xfc000000,
    //------------
    PPC_MFFS        = 0xfc000000,
    PPC_MTFSF       = 0xfc000000,
    PPC_FCTID       = 0xfc000000,
    PPC_FCTIDZ      = 0xfc000000,
    PPC_FCFID       = 0xfc000000,
}

// the newly generated code
static mut buf: *mut c_int = ptr::null_mut();
static mut compiledOfs: c_int = 0;    // in dwords

// fromt the original bytecode
static mut code: *mut u8 = ptr::null_mut();
static mut pc: c_int = 0;

extern "C" {
    fn AsmCall();
    fn Com_Error(level: c_int, format: *const c_char, ...);
    fn Com_Printf(format: *const c_char, ...);
    fn Com_Memset(mem: *mut c_void, c: c_int, count: usize);
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
    fn Z_Malloc(size: usize, zero: c_int) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn Hunk_Alloc(size: c_int, tag: c_int) -> *mut c_void;

    static mut currentVM: *mut c_void;
}

pub static mut itofConvert: [f64; 2] = [0.0; 2];

unsafe fn Constant4() -> c_int {
    let v = *code.add(pc as usize) as c_int
        | ((*code.add((pc + 1) as usize) as c_int) << 8)
        | ((*code.add((pc + 2) as usize) as c_int) << 16)
        | ((*code.add((pc + 3) as usize) as c_int) << 24);
    pc += 4;
    v
}

unsafe fn Constant1() -> c_int {
    let v = *code.add(pc as usize) as c_int;
    pc += 1;
    v
}

unsafe fn Emit4(i: c_int) {
    *buf.add(compiledOfs as usize) = i;
    compiledOfs += 1;
}

unsafe fn Inst(opcode: c_int, destReg: c_int, aReg: c_int, bReg: c_int) {
    let r: c_int = opcode | (destReg << 21) | (aReg << 16) | (bReg << 11);
    *buf.add(compiledOfs as usize) = r;
    compiledOfs += 1;
}

unsafe fn Inst4(opcode: c_int, destReg: c_int, aReg: c_int, bReg: c_int, cReg: c_int) {
    let r: c_int = opcode | (destReg << 21) | (aReg << 16) | (bReg << 11) | (cReg << 6);
    *buf.add(compiledOfs as usize) = r;
    compiledOfs += 1;
}

unsafe fn InstImm(opcode: c_int, destReg: c_int, aReg: c_int, immediate: c_int) {
    if immediate > 32767 || immediate < -32768 {
        Com_Error(3, b"VM_Compile: immediate value %i out of range, opcode %x,%d,%d\0" as *const u8 as *const c_char, immediate, opcode, destReg, aReg);
    }
    let r: c_int = opcode | (destReg << 21) | (aReg << 16) | (immediate & 0xffff);
    *buf.add(compiledOfs as usize) = r;
    compiledOfs += 1;
}

unsafe fn InstImmU(opcode: c_int, destReg: c_int, aReg: c_int, immediate: c_int) {
    if immediate > 0xffff || immediate < 0 {
        Com_Error(3, b"VM_Compile: immediate value %i out of range\0" as *const u8 as *const c_char, immediate);
    }
    let r: c_int = opcode | (destReg << 21) | (aReg << 16) | (immediate & 0xffff);
    *buf.add(compiledOfs as usize) = r;
    compiledOfs += 1;
}

static mut rtopped: bool = false;
static mut pop0: c_int = 0;
static mut pop1: c_int = 0;
static mut oc0: c_int = 0;
static mut oc1: c_int = 0;
static mut tvm: *mut c_void = ptr::null_mut();
static mut instruction: c_int = 0;
static mut jused: *mut u8 = ptr::null_mut();
static mut pass: c_int = 0;

unsafe fn ltop() {
    if rtopped == false {
        InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);     // get value from opstack
    }
}

unsafe fn ltopandsecond() {
    // Porting note: Using transmute-free pointer arithmetic for buffer access
    let tvm_typed = tvm as *mut vm_t;
    if pass >= 0 && *buf.add((compiledOfs - 1) as usize) == (ppcOpcodes_t::PPC_STWU as c_int | (regNums_t::R_TOP as c_int) << 21 | (regNums_t::R_OPSTACK as c_int) << 16 | 4) && *jused.add(instruction as usize) == 0 {
        compiledOfs -= 1;
        if pass == 0 {
            (*tvm_typed).instructionPointers[instruction as usize] = (compiledOfs * 4) as c_int;
        }
        InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, 0);    // get value from opstack
        InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
    } else if pass >= 0 && *buf.add((compiledOfs - 1) as usize) == (ppcOpcodes_t::PPC_STW as c_int | (regNums_t::R_TOP as c_int) << 21 | (regNums_t::R_OPSTACK as c_int) << 16 | 0) && *jused.add(instruction as usize) == 0 {
        compiledOfs -= 1;
        if pass == 0 {
            (*tvm_typed).instructionPointers[instruction as usize] = (compiledOfs * 4) as c_int;
        }
        InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);   // get value from opstack
        InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -8);
    } else {
        ltop();      // get value from opstack
        InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);   // get value from opstack
        InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -8);
    }
    rtopped = false;
}

// TJW: Unused
#[allow(dead_code)]
unsafe fn fltop() {
    if rtopped == false {
        InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);     // get value from opstack
    }
}

unsafe fn fltopandsecond() {
    InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);     // get value from opstack
    InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);  // get value from opstack
    InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -8);
    rtopped = false;
}

// Porting note: Forward declaration of vm_t structure used by VM_Compile
#[repr(C)]
pub struct vm_t {
    // Minimal definition - only fields used in this file
    pub name: [c_char; 256],
    pub dataMask: c_int,
    pub instructionPointers: *mut c_int,
    pub codeBase: *mut c_void,
    pub codeLength: c_int,
    pub programStack: c_int,
    pub dataBase: *mut u8,
    pub currentlyInterpreting: bool,
}

// Porting note: Forward declaration of vmHeader_t structure
#[repr(C)]
pub struct vmHeader_t {
    pub codeOffset: c_int,
    pub codeLength: c_int,
    pub instructionCount: c_int,
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

    // set up the into-to-float variables
    ((&mut itofConvert[0]) as *mut f64 as *mut c_int).add(0).write(0x43300000);
    ((&mut itofConvert[0]) as *mut f64 as *mut c_int).add(1).write(0x80000000);
    ((&mut itofConvert[0]) as *mut f64 as *mut c_int).add(2).write(0x43300000);

    // allocate a very large temp buffer, we will shrink it later
    maxLength = (*header).codeLength * 8;
    buf = Z_Malloc(maxLength as usize, 0) as *mut c_int;
    jused = Z_Malloc(((*header).instructionCount + 2) as usize, 0) as *mut u8;
    Com_Memset(jused as *mut c_void, 0, ((*header).instructionCount + 2) as usize);

    // compile everything twice, so the second pass will have valid instruction
    // pointers for branches
    pass = -1;
    while pass < 2 {

        rtopped = false;
        // translate all instructions
        pc = 0;

        pop0 = 343545;
        pop1 = 2443545;
        oc0 = -2343535;
        oc1 = 24353454;
        tvm = vm as *mut c_void;

        code = (header as *mut u8).add((*header).codeOffset as usize);
        compiledOfs = 0;

        // metrowerks seems to require this header in front of functions
        // Porting note: GCC doesn't require this, GNUC check omitted
        #[cfg(not(target_env = "gnu"))]
        {
            Emit4((buf.add(2) as c_int));
            Emit4(0);
        }

        instruction = 0;
        while instruction < (*header).instructionCount {
            if compiledOfs * 4 > maxLength - 16 {
                Com_Error(1, b"VM_Compile: maxLength exceeded\0" as *const u8 as *const c_char);
            }

            op = *code.add(pc as usize) as c_int;
            if pass == 0 {
                (*vm).instructionPointers[instruction as usize] = compiledOfs * 4;
            }
            pc += 1;
            match op {
            0 => {
                // break (empty case)
            },
            1 /* OP_BREAK */ => {
                InstImmU(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_TOP as c_int, 0, 0);
                InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, 0);         // *(int *)0 to crash to debugger
                rtopped = false;
            },
            2 /* OP_ENTER */ => {
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_STACK as c_int, regNums_t::R_STACK as c_int, -Constant4());   // sub R_STACK, R_STACK, imm
                rtopped = false;
            },
            3 /* OP_CONST */ => {
                v = Constant4();
                if *code.add(pc as usize) == 20 || *code.add(pc as usize) == 21 || *code.add(pc as usize) == 22 {
                    v &= (*vm).dataMask;
                }
                if v < 32768 && v >= -32768 {
                    InstImmU(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_TOP as c_int, 0, v & 0xffff);
                } else {
                    InstImmU(ppcOpcodes_t::PPC_ADDIS as c_int, regNums_t::R_TOP as c_int, 0, (v >> 16) & 0xffff);
                    if v & 0xffff != 0 {
                        InstImmU(ppcOpcodes_t::PPC_ORI as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, v & 0xffff);
                    }
                }
                if *code.add(pc as usize) == 20 {
                    Inst(ppcOpcodes_t::PPC_LWZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                    pc += 1;
                    instruction += 1;
                } else if *code.add(pc as usize) == 21 {
                    Inst(ppcOpcodes_t::PPC_LHZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                    pc += 1;
                    instruction += 1;
                } else if *code.add(pc as usize) == 22 {
                    Inst(ppcOpcodes_t::PPC_LBZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                    pc += 1;
                    instruction += 1;
                }
                if *code.add(pc as usize) == 23 {
                    InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, 0);    // get value from opstack
                    InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                    //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );    // mask it
                    Inst(ppcOpcodes_t::PPC_STWX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                    pc += 1;
                    instruction += 1;
                    rtopped = false;
                } else if *code.add(pc as usize) == 24 {
                    InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, 0);    // get value from opstack
                    InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                    //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );    // mask it
                    Inst(ppcOpcodes_t::PPC_STHX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                    pc += 1;
                    instruction += 1;
                    rtopped = false;
                } else if *code.add(pc as usize) == 25 {
                    InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, 0);    // get value from opstack
                    InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                    //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );    // mask it
                    Inst(ppcOpcodes_t::PPC_STBX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                    pc += 1;
                    instruction += 1;
                    rtopped = false;
                }
                if *code.add(pc as usize) == 6 {
                    *jused.add(v as usize) = 1;
                }
                InstImm(ppcOpcodes_t::PPC_STWU as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 4);
                rtopped = true;
            },
            4 /* OP_LOCAL */ => {
                oc0 = oc1;
                oc1 = Constant4();
                if *code.add(pc as usize) == 20 || *code.add(pc as usize) == 21 || *code.add(pc as usize) == 22 {
                    oc1 &= (*vm).dataMask;
                }
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_TOP as c_int, regNums_t::R_STACK as c_int, oc1);
                if *code.add(pc as usize) == 20 {
                    Inst(ppcOpcodes_t::PPC_LWZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                    pc += 1;
                    instruction += 1;
                } else if *code.add(pc as usize) == 21 {
                    Inst(ppcOpcodes_t::PPC_LHZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                    pc += 1;
                    instruction += 1;
                } else if *code.add(pc as usize) == 22 {
                    Inst(ppcOpcodes_t::PPC_LBZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                    pc += 1;
                    instruction += 1;
                }
                if *code.add(pc as usize) == 23 {
                    InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, 0);        // get value from opstack
                    InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                    //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );    // mask it
                    Inst(ppcOpcodes_t::PPC_STWX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                    pc += 1;
                    instruction += 1;
                    rtopped = false;
                } else if *code.add(pc as usize) == 24 {
                    InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, 0);        // get value from opstack
                    InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                    //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );    // mask it
                    Inst(ppcOpcodes_t::PPC_STHX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                    pc += 1;
                    instruction += 1;
                    rtopped = false;
                } else if *code.add(pc as usize) == 25 {
                    InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, 0);        // get value from opstack
                    InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                    //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );    // mask it
                    Inst(ppcOpcodes_t::PPC_STBX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                    pc += 1;
                    instruction += 1;
                    rtopped = false;
                }
                InstImm(ppcOpcodes_t::PPC_STWU as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 4);
                rtopped = true;
            },
            5 /* OP_ARG */ => {
                ltop();                         // get value from opstack
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_EA as c_int, regNums_t::R_STACK as c_int, Constant1());    // location to put it
                Inst(ppcOpcodes_t::PPC_STWX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_EA as c_int, regNums_t::R_MEMBASE as c_int);
                rtopped = false;
            },
            7 /* OP_CALL */ => {
                Inst(ppcOpcodes_t::PPC_MFSPR as c_int, regNums_t::R_SECOND as c_int, 8, 0);            // move from link register
                InstImm(ppcOpcodes_t::PPC_STWU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_REAL_STACK as c_int, -16);   // save off the old return address

                Inst(ppcOpcodes_t::PPC_MTSPR as c_int, regNums_t::R_ASMCALL as c_int, 9, 0);           // move to count register
                Inst((ppcOpcodes_t::PPC_BCCTR as c_int) | 1, 20, 0, 0);        // jump and link to the count register

                InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_REAL_STACK as c_int, 0);      // fetch the old return address
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_REAL_STACK as c_int, regNums_t::R_REAL_STACK as c_int, 16);
                Inst(ppcOpcodes_t::PPC_MTSPR as c_int, regNums_t::R_SECOND as c_int, 8, 0);            // move to link register
                rtopped = false;
            },
            8 /* OP_PUSH */ => {
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, 4);
                rtopped = false;
            },
            9 /* OP_POP */ => {
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                rtopped = false;
            },
            10 /* OP_LEAVE */ => {
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_STACK as c_int, regNums_t::R_STACK as c_int, Constant4());   // add R_STACK, R_STACK, imm
                Inst(ppcOpcodes_t::PPC_BCLR as c_int, 20, 0, 0);                        // branch unconditionally to link register
                rtopped = false;
            },
            20 /* OP_LOAD4 */ => {
                ltop();                         // get value from opstack
                //Inst( PPC_AND, R_MEMMASK, R_TOP, R_TOP );     // mask it
                Inst(ppcOpcodes_t::PPC_LWZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);
                rtopped = true;
            },
            21 /* OP_LOAD2 */ => {
                ltop();                         // get value from opstack
                //Inst( PPC_AND, R_MEMMASK, R_TOP, R_TOP );     // mask it
                Inst(ppcOpcodes_t::PPC_LHZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);
                rtopped = true;
            },
            22 /* OP_LOAD1 */ => {
                ltop();                         // get value from opstack
                //Inst( PPC_AND, R_MEMMASK, R_TOP, R_TOP );     // mask it
                Inst(ppcOpcodes_t::PPC_LBZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);      // load from memory base
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);
                rtopped = true;
            },
            23 /* OP_STORE4 */ => {
                ltopandsecond();                    // get value from opstack
                //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );     // mask it
                Inst(ppcOpcodes_t::PPC_STWX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                rtopped = false;
            },
            24 /* OP_STORE2 */ => {
                ltopandsecond();                    // get value from opstack
                //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );     // mask it
                Inst(ppcOpcodes_t::PPC_STHX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                rtopped = false;
            },
            25 /* OP_STORE1 */ => {
                ltopandsecond();                    // get value from opstack
                //Inst( PPC_AND, R_MEMMASK, R_SECOND, R_SECOND );     // mask it
                Inst(ppcOpcodes_t::PPC_STBX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);   // store from memory base
                rtopped = false;
            },

            26 /* OP_EQ */ => {
                ltopandsecond();                    // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMP as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 2, 8);
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                Emit4((ppcOpcodes_t::PPC_B as c_int) | (v & 0x3ffffff));
                rtopped = false;
            },
            27 /* OP_NE */ => {
                ltopandsecond();                    // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMP as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 12, 2, 8);
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                Emit4((ppcOpcodes_t::PPC_B as c_int) | ((v as c_int & 0x3ffffff) as c_int));
                //                InstImm( PPC_BC, 4, 2, v );

                rtopped = false;
            },
            28 /* OP_LTI */ => {
                ltopandsecond();                    // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMP as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 0, 8);
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                Emit4((ppcOpcodes_t::PPC_B as c_int) | ((v as c_int & 0x3ffffff) as c_int));
                //                InstImm( PPC_BC, 12, 0, v );
                rtopped = false;
            },
            29 /* OP_LEI */ => {
                ltopandsecond();                    // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMP as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 12, 1, 8);
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                Emit4((ppcOpcodes_t::PPC_B as c_int) | ((v as c_int & 0x3ffffff) as c_int));
                //                InstImm( PPC_BC, 4, 1, v );
                rtopped = false;
            },
            30 /* OP_GTI */ => {
                ltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMP as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 1, 8);
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                Emit4((ppcOpcodes_t::PPC_B as c_int) | ((v as c_int & 0x3ffffff) as c_int));
                //                InstImm( PPC_BC, 12, 1, v );
                rtopped = false;
            },
            31 /* OP_GEI */ => {
                ltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMP as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 0, v);
                rtopped = false;
            },
            32 /* OP_LTU */ => {
                ltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMPL as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 12, 0, v);
                rtopped = false;
            },
            33 /* OP_LEU */ => {
                ltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMPL as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 1, v);
                rtopped = false;
            },
            34 /* OP_GTU */ => {
                ltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMPL as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 12, 1, v);
                rtopped = false;
            },
            35 /* OP_GEU */ => {
                ltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_CMPL as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 0, v);
                rtopped = false;
            },

            36 /* OP_EQF */ => {
                fltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FCMPU as c_int, 0, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 12, 2, v);
                rtopped = false;
            },
            37 /* OP_NEF */ => {
                fltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FCMPU as c_int, 0, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 2, v);
                rtopped = false;
            },
            38 /* OP_LTF */ => {
                fltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FCMPU as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 12, 0, v);
                rtopped = false;
            },
            39 /* OP_LEF */ => {
                fltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FCMPU as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 1, v);
                rtopped = false;
            },
            40 /* OP_GTF */ => {
                fltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FCMPU as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 12, 1, v);
                rtopped = false;
            },
            41 /* OP_GEF */ => {
                fltopandsecond();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FCMPU as c_int, 0, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                i = Constant4();
                *jused.add(i as usize) = 1;
                if pass == 1 {
                    v = (*(*vm as *mut vm_t)).instructionPointers[i as usize] - (buf.add(compiledOfs as usize) as c_int);
                } else {
                    v = 0;
                }
                InstImm(ppcOpcodes_t::PPC_BC as c_int, 4, 0, v);
                rtopped = false;
            },

            42 /* OP_NEGI */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_SUBFIC as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, 0);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            43 /* OP_ADD */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_ADD as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            44 /* OP_SUB */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_SUBF as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            45 /* OP_DIVI */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_DIVW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            46 /* OP_DIVU */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_DIVWU as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            47 /* OP_MODI */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_DIVW as c_int, regNums_t::R_EA as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                Inst(ppcOpcodes_t::PPC_MULLW as c_int, regNums_t::R_EA as c_int, regNums_t::R_TOP as c_int, regNums_t::R_EA as c_int);
                Inst(ppcOpcodes_t::PPC_SUBF as c_int, regNums_t::R_TOP as c_int, regNums_t::R_EA as c_int, regNums_t::R_SECOND as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            48 /* OP_MODU */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_DIVWU as c_int, regNums_t::R_EA as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                Inst(ppcOpcodes_t::PPC_MULLW as c_int, regNums_t::R_EA as c_int, regNums_t::R_TOP as c_int, regNums_t::R_EA as c_int);
                Inst(ppcOpcodes_t::PPC_SUBF as c_int, regNums_t::R_TOP as c_int, regNums_t::R_EA as c_int, regNums_t::R_SECOND as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            49 /* OP_MULI */ |
            50 /* OP_MULU */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_MULLW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            51 /* OP_BAND */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_AND as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            52 /* OP_BOR */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_OR as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            53 /* OP_BXOR */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_XOR as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            54 /* OP_BCOM */ => {
                ltop();     // get value from opstack
                Inst(ppcOpcodes_t::PPC_NOR as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            55 /* OP_LSH */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_SLW as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            56 /* OP_RSHI */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_SRAW as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },
            57 /* OP_RSHU */ => {
                ltop();     // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_SRW as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = true;
            },

            58 /* OP_NEGF */ => {
                InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // get value from opstack
                Inst(ppcOpcodes_t::PPC_FNEG as c_int, regNums_t::R_TOP as c_int, 0, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = false;
            },
            59 /* OP_ADDF */ => {
                InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LFSU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FADDS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = false;
            },
            60 /* OP_SUBF */ => {
                InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LFSU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FSUBS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = false;
            },
            61 /* OP_DIVF */ => {
                InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LFSU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst(ppcOpcodes_t::PPC_FDIVS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = false;
            },
            62 /* OP_MULF */ => {
                InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // get value from opstack
                InstImm(ppcOpcodes_t::PPC_LFSU as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);     // get value from opstack
                Inst4(ppcOpcodes_t::PPC_FMULS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, 0, regNums_t::R_TOP as c_int);
                InstImm(ppcOpcodes_t::PPC_STFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = false;
            },

            63 /* OP_CVIF */ => {
                v = (&itofConvert[0] as *const f64) as c_int;
                InstImmU(ppcOpcodes_t::PPC_ADDIS as c_int, regNums_t::R_EA as c_int, 0, (v >> 16) & 0xffff);
                InstImmU(ppcOpcodes_t::PPC_ORI as c_int, regNums_t::R_EA as c_int, regNums_t::R_EA as c_int, v & 0xffff);
                InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // get value from opstack
                InstImmU(ppcOpcodes_t::PPC_XORIS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, 0x8000);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_EA as c_int, 12);
                InstImm(ppcOpcodes_t::PPC_LFD as c_int, regNums_t::R_TOP as c_int, regNums_t::R_EA as c_int, 0);
                InstImm(ppcOpcodes_t::PPC_LFD as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_EA as c_int, 8);
                Inst(ppcOpcodes_t::PPC_FSUB as c_int, regNums_t::R_TOP as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_TOP as c_int);
    //            Inst( PPC_FRSP, R_TOP, 0, R_TOP );
                InstImm(ppcOpcodes_t::PPC_STFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // save value to opstack
                rtopped = false;
            },
            64 /* OP_CVFI */ => {
                InstImm(ppcOpcodes_t::PPC_LFS as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);      // get value from opstack
                Inst(ppcOpcodes_t::PPC_FCTIWZ as c_int, regNums_t::R_TOP as c_int, 0, regNums_t::R_TOP as c_int);
                Inst(ppcOpcodes_t::PPC_STFIWX as c_int, regNums_t::R_TOP as c_int, 0, regNums_t::R_OPSTACK as c_int);      // save value to opstack
                rtopped = false;
            },
            65 /* OP_SEX8 */ => {
                ltop();  // get value from opstack
                Inst(ppcOpcodes_t::PPC_EXTSB as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, 0);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);
                rtopped = true;
            },
            66 /* OP_SEX16 */ => {
                ltop();  // get value from opstack
                Inst(ppcOpcodes_t::PPC_EXTSH as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, 0);
                InstImm(ppcOpcodes_t::PPC_STW as c_int, regNums_t::R_TOP as c_int, regNums_t::R_OPSTACK as c_int, 0);
                rtopped = true;
            },

            67 /* OP_BLOCK_COPY */ => {
                v = Constant4() >> 2;
                ltop();     // source
                InstImm(ppcOpcodes_t::PPC_LWZ as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_OPSTACK as c_int, -4);   // dest
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -8);
                InstImmU(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_EA as c_int, 0, v as c_int);               // count
                // FIXME: range check
                Inst(ppcOpcodes_t::PPC_MTSPR as c_int, regNums_t::R_EA as c_int, 9, 0);                  // move to count register

                Inst(ppcOpcodes_t::PPC_ADD as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_MEMBASE as c_int);
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, -4);
                Inst(ppcOpcodes_t::PPC_ADD as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_MEMBASE as c_int);
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_SECOND as c_int, regNums_t::R_SECOND as c_int, -4);

                InstImm(ppcOpcodes_t::PPC_LWZU as c_int, regNums_t::R_EA as c_int, regNums_t::R_TOP as c_int, 4);    // source
                InstImm(ppcOpcodes_t::PPC_STWU as c_int, regNums_t::R_EA as c_int, regNums_t::R_SECOND as c_int, 4);  // dest
                Inst((ppcOpcodes_t::PPC_BC as c_int) | 0xfff8 , 16, 0, 0);                   // loop
                rtopped = false;
            },

            68 /* OP_JUMP */ => {
                ltop();  // get value from opstack
                InstImm(ppcOpcodes_t::PPC_ADDI as c_int, regNums_t::R_OPSTACK as c_int, regNums_t::R_OPSTACK as c_int, -4);
                Inst((ppcOpcodes_t::PPC_RLWINM as c_int) | (29 << 1), regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, 2);
                // FIXME: range check
                Inst(ppcOpcodes_t::PPC_LWZX as c_int, regNums_t::R_TOP as c_int, regNums_t::R_TOP as c_int, regNums_t::R_INSTRUCTIONS as c_int);
                Inst(ppcOpcodes_t::PPC_MTSPR as c_int, regNums_t::R_TOP as c_int, 9, 0);     // move to count register
                Inst(ppcOpcodes_t::PPC_BCCTR as c_int, 20, 0, 0);     // jump to the count register
                rtopped = false;
            },
            _ => {
                Com_Error(1, b"VM_CompilePPC: bad opcode %i at instruction %i, offset %i\0" as *const u8 as *const c_char, op, instruction, pc);
            }
            }
            pop0 = pop1;
            pop1 = op;
            instruction += 1;
        }

        Com_Printf(b"VM file %s pass %d compiled to %i bytes of code\n\0" as *const u8 as *const c_char, (*vm).name.as_ptr(), pass + 1, compiledOfs * 4);

        if pass == 0 {
            // copy to an exact size buffer on the hunk
            (*vm).codeLength = compiledOfs * 4;
            (*vm).codeBase = Hunk_Alloc((*vm).codeLength, 0); // h_low = 0
            Com_Memcpy((*vm).codeBase, buf as *const c_void, (*vm).codeLength as usize);
            Z_Free(buf as *mut c_void);

            // offset all the instruction pointers for the new location
            for i in 0..(*header).instructionCount {
                (*vm).instructionPointers[i as usize] += (*vm).codeBase as c_int;
            }

            // go back over it in place now to fixup reletive jump targets
            buf = (*vm).codeBase as *mut c_int;
        }
        pass += 1;
    }
    Z_Free(jused as *mut c_void);
}

/*
==============
VM_CallCompiled

This function is called directly by the generated code
==============
*/
pub unsafe fn VM_CallCompiled(vm: *mut vm_t, args: *mut c_int) -> c_int {
    let mut stack: [c_int; 1024] = [0; 1024];
    let mut programStack: c_int;
    let stackOnEntry: c_int;
    let image: *mut u8;

    currentVM = vm as *mut c_void;

    // interpret the code
    (*vm).currentlyInterpreting = true;

    // we might be called recursively, so this might not be the very top
    programStack = (*vm).programStack;
    stackOnEntry = programStack;
    image = (*vm).dataBase;

    // set up the stack frame
    programStack -= 48;

    *(image.add((programStack + 44) as usize) as *mut c_int) = *args.add(9);
    *(image.add((programStack + 40) as usize) as *mut c_int) = *args.add(8);
    *(image.add((programStack + 36) as usize) as *mut c_int) = *args.add(7);
    *(image.add((programStack + 32) as usize) as *mut c_int) = *args.add(6);
    *(image.add((programStack + 28) as usize) as *mut c_int) = *args.add(5);
    *(image.add((programStack + 24) as usize) as *mut c_int) = *args.add(4);
    *(image.add((programStack + 20) as usize) as *mut c_int) = *args.add(3);
    *(image.add((programStack + 16) as usize) as *mut c_int) = *args.add(2);
    *(image.add((programStack + 12) as usize) as *mut c_int) = *args.add(1);
    *(image.add((programStack + 8) as usize) as *mut c_int) = *args.add(0);
    *(image.add((programStack + 4) as usize) as *mut c_int) = 0;    // return stack
    *(image.add(programStack as usize) as *mut c_int) = -1;    // will terminate the loop on return

    // off we go into generated code...
    // the PPC calling standard says the parms will all go into R3 - R11, so
    // no special asm code is needed here
    #[cfg(target_env = "gnu")]
    {
        (std::mem::transmute::<*mut c_void, extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int)>((*vm).codeBase))(
            programStack, stack.as_mut_ptr() as c_int,
            image as c_int, (*vm).dataMask, AsmCall as *mut c_void as c_int,
            (*vm).instructionPointers as c_int, 0, // instructionPointersLength would be needed
            vm as c_int);
    }
    #[cfg(not(target_env = "gnu"))]
    {
        (std::mem::transmute::<*mut c_void, extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int)>((*vm).codeBase))(
            programStack, stack.as_mut_ptr() as c_int,
            image as c_int, (*vm).dataMask, *(AsmCall as *mut c_void as *mut c_int), // skip function pointer header
            (*vm).instructionPointers as c_int, 0, // instructionPointersLength would be needed
            vm as c_int);
    }

    (*vm).programStack = stackOnEntry;

    (*vm).currentlyInterpreting = false;

    stack[1]
}


/*
==================
AsmCall

Put this at end of file because gcc messes up debug line numbers
==================
*/
#[cfg(target_env = "gnu")]

pub extern "C" fn AsmCall() {
    // Porting note: Inline assembly for PowerPC is highly platform-specific.
    // The original C code uses inline asm with GCC syntax.
    // This cannot be directly translated to Rust without using unsafe asm! blocks.
    // For now, this is a stub that would need platform-specific implementation.
    // See original vm_ppc.cpp lines 1188-1271 for the assembly code.

    // The assembly performs:
    // 1. Pop instruction from opstack
    // 2. Check if system trap (compare with 0)
    // 3. If not trap: lookup in instructionPointers and branch
    // 4. If trap: convert call number, save registers, call systemCalls, restore registers

    // This would require:
    // #[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
    // with inline assembly using llvm_asm! or asm! macro
}

#[cfg(not(target_env = "gnu"))]
pub extern "C" fn AsmCall() {
    // Metrowerks compiler version - stub
    // Original assembly code would be different for non-GCC compilers
}
