// rww - so that I may utilize vm debugging features WITHOUT DROPPING TO 0.1FPS
// Porting: CRAZY_SYMBOL_MAP is defined when not targeting Xbox (#ifndef _XBOX / #define CRAZY_SYMBOL_MAP).
// Mapped to Cargo feature "crazy_symbol_map".

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use core::ffi::{c_char, c_int, c_void};

// Porting note: vm_local.h carries no explicit #include for q_shared.h; these base
// types (MAX_QPATH, qboolean, byte) are trusted to arrive from the enclosing
// compilation unit.  Imported here so the Rust module is self-contained.
use crate::codemp::qcommon::q_shared_h::*;

// vmHeader_t real definition lives in qfiles.h; imported from there.
use crate::codemp::qcommon::qfiles_h::*;

// #include <map> is conditional on CRAZY_SYMBOL_MAP
#[cfg(feature = "crazy_symbol_map")]
use std::collections::BTreeMap;

#[repr(C)]
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

#[repr(C)]
pub struct vm_s {
    // DO NOT MOVE OR CHANGE THESE WITHOUT CHANGING THE VM_OFFSET_* DEFINES
    // USED BY THE ASM CODE
    pub programStack: c_int, // the vm may be recursively entered
    pub systemCall: Option<unsafe extern "C" fn(*mut c_int) -> c_int>,

    //------------------------------------

    pub name: [c_char; MAX_QPATH as usize],

    // for dynamic linked modules
    pub dllHandle: *mut c_void,
    pub entryPoint: Option<unsafe extern "C" fn(callNum: c_int, ...) -> c_int>,

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

// Porting note: C++ makes `vm_s` and `vm_t` distinct names; `vm_t` is typedef'd
// as `struct vm_s` in the wider codebase and used extensively in this header.
pub type vm_t = vm_s;

const _: () =
    assert!(core::mem::offset_of!(vm_s, programStack) == VM_OFFSET_PROGRAM_STACK as usize);

#[cfg(feature = "crazy_symbol_map")]
pub type symbolMap_t = BTreeMap<c_int, *mut vmSymbol_s>;

#[cfg(feature = "crazy_symbol_map")]
pub type symbolVMMap_t = BTreeMap<*mut vm_t, symbolMap_t>;

#[cfg(feature = "crazy_symbol_map")]
#[allow(improper_ctypes)]
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
#[cfg(feature = "crazy_symbol_map")]
#[inline]
pub unsafe fn VM_SetSymbolMap(vm: *mut vm_t) {
    use core::ptr::addr_of_mut;
    // C++: g_symbolMap = &g_vmMap[vm];
    // std::map::operator[] inserts a default entry when the key is absent.
    // Translated using BTreeMap::entry().or_insert_with().
    *addr_of_mut!(g_symbolMap) = (*addr_of_mut!(g_vmMap))
        .entry(vm)
        .or_insert_with(symbolMap_t::new) as *mut symbolMap_t;
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
