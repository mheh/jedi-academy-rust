//! Mechanical port of `codemp/qcommon/vm_local.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use crate::codemp::game::q_shared_h::{byte, qboolean, MAX_QPATH};
use core::ffi::{c_char, c_int, c_void};

// rww - so that I may utilize vm debugging features WITHOUT DROPPING TO 0.1FPS
// #ifndef _XBOX
// #define CRAZY_SYMBOL_MAP
// #endif

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum opcode_t {
    OP_UNDEF,

    OP_IGNORE,

    OP_BREAK,

    OP_ENTER,
    OP_LEAVE,
    OP_CALL,
    OP_PUSH,
    OP_POP,

    OP_CONST,
    OP_LOCAL,

    OP_JUMP,

    //-------------------

    OP_EQ,
    OP_NE,

    OP_LTI,
    OP_LEI,
    OP_GTI,
    OP_GEI,

    OP_LTU,
    OP_LEU,
    OP_GTU,
    OP_GEU,

    OP_EQF,
    OP_NEF,

    OP_LTF,
    OP_LEF,
    OP_GTF,
    OP_GEF,

    //-------------------

    OP_LOAD1,
    OP_LOAD2,
    OP_LOAD4,
    OP_STORE1,
    OP_STORE2,
    OP_STORE4, // *(stack[top-1]) = stack[top]
    OP_ARG,

    OP_BLOCK_COPY,

    //-------------------

    OP_SEX8,
    OP_SEX16,

    OP_NEGI,
    OP_ADD,
    OP_SUB,
    OP_DIVI,
    OP_DIVU,
    OP_MODI,
    OP_MODU,
    OP_MULI,
    OP_MULU,

    OP_BAND,
    OP_BOR,
    OP_BXOR,
    OP_BCOM,

    OP_LSH,
    OP_RSHI,
    OP_RSHU,

    OP_NEGF,
    OP_ADDF,
    OP_SUBF,
    OP_DIVF,
    OP_MULF,

    OP_CVIF,
    OP_CVFI,
}

pub type vmptr_t = c_int;

#[repr(C)]
pub struct vmSymbol_s {
    pub next: *mut vmSymbol_s,
    pub symValue: c_int,
    pub profileCount: c_int,
    pub symName: [c_char; 1], // variable sized
}

pub type vmSymbol_t = vmSymbol_s;

pub const VM_OFFSET_PROGRAM_STACK: c_int = 0;
pub const VM_OFFSET_SYSTEM_CALL: c_int = 4;

// Header-local stub for `qfiles.h`, which owns the real VM bytecode header.
#[repr(C)]
pub struct vmHeader_t {
    _private: [u8; 0],
}

pub type vmEntryPoint_t = Option<unsafe extern "C" fn(callNum: c_int, ...) -> c_int>;
pub type vmSystemCall_t = Option<unsafe extern "C" fn(parms: *mut c_int) -> c_int>;

#[repr(C)]
pub struct vm_s {
    // DO NOT MOVE OR CHANGE THESE WITHOUT CHANGING THE VM_OFFSET_* DEFINES
    // USED BY THE ASM CODE
    pub programStack: c_int, // the vm may be recursively entered
    pub systemCall: vmSystemCall_t,

    //------------------------------------

    pub name: [c_char; MAX_QPATH],

    // for dynamic linked modules
    pub dllHandle: *mut c_void,
    pub entryPoint: vmEntryPoint_t,

    // for interpreted modules
    pub currentlyInterpreting: qboolean,

    pub compiled: qboolean,
    pub codeBase: *mut byte,
    pub codeLength: c_int,

    pub instructionPointers: *mut c_int,
    pub instructionPointersLength: c_int,

    pub dataBase: *mut byte,
    pub dataMask: c_int,

    pub stackBottom: c_int, // if programStack < stackBottom, error

    pub numSymbols: c_int,
    pub symbols: *mut vmSymbol_s,

    pub callLevel: c_int,     // for debug indenting
    pub breakFunction: c_int, // increment breakCount on function entry to this
    pub breakCount: c_int,
}

pub type vm_t = vm_s;

const _: () = assert!(core::mem::offset_of!(vm_s, programStack) == VM_OFFSET_PROGRAM_STACK as usize);

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct symbolMap_t {
    _private: [u8; 0],
}

#[cfg(not(feature = "xbox"))]
#[repr(C)]
pub struct symbolVMMap_t {
    _private: [u8; 0],
}

#[cfg(not(feature = "xbox"))]
unsafe extern "C" {
    pub static mut g_vmMap: symbolVMMap_t;
    pub static mut g_symbolMap: *mut symbolMap_t;
}

/*
Set the symbol map based on the VM currently
being in interpreted. This is done so that we
do not have to do a map lookup for the VM with
each symbol request.
-rww
*/
#[cfg(not(feature = "xbox"))]
#[inline]
pub unsafe fn VM_SetSymbolMap(vm: *mut vm_t) {
    // Porting deviation: the C++ original indexes `std::map<vm_t*, symbolMap_t>`.
    // The map ABI is not available to this header-only Rust port, so keep the
    // declaration shape and leave the lookup for a future C++ VM map binding.
    let _ = vm;
}

unsafe extern "C" {
    pub static mut currentVM: *mut vm_t;
    pub static mut vm_debugLevel: c_int;

    pub fn VM_Compile(vm: *mut vm_t, header: *mut vmHeader_t);
    pub fn VM_CallCompiled(vm: *mut vm_t, args: *mut c_int) -> c_int;

    pub fn VM_PrepareInterpreter(vm: *mut vm_t, header: *mut vmHeader_t);
    pub fn VM_CallInterpreted(vm: *mut vm_t, args: *mut c_int) -> c_int;

    pub fn VM_ValueToFunctionSymbol(vm: *mut vm_t, value: c_int) -> *mut vmSymbol_t;
    pub fn VM_SymbolToValue(vm: *mut vm_t, symbol: *const c_char) -> c_int;
    pub fn VM_ValueToSymbol(vm: *mut vm_t, value: c_int) -> *const c_char;
    pub fn VM_LogSyscalls(args: *mut c_int);
}
