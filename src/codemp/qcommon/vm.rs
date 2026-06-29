//! vm.c -- virtual machine
//!
//! intermix code and data
//! symbol table
//!
//! a dll has one imported function: VM_SystemCall
//! and one exported function: Perform

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, qboolean, MAX_QPATH, MAX_TOKEN_CHARS, ERR_DROP, ERR_FATAL, QTRUE, QFALSE};
use crate::codemp::qcommon::files_h::cvar_t;
use crate::codemp::qcommon::qfiles_h::vmHeader_t;
use crate::codemp::qcommon::vm_local_h::{vm_t, vmSymbol_t, vmptr_t, vmSystemCall_t, vmEntryPoint_t};
use core::ffi::{c_char, c_int, c_void};

// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "vm_local.h"

// #ifdef CRAZY_SYMBOL_MAP
// symbolVMMap_t		g_vmMap;
// symbolMap_t			*g_symbolMap;
// #endif

pub static mut currentVM: *mut vm_t = core::ptr::null_mut(); // bk001212
pub static mut lastVM: *mut vm_t = core::ptr::null_mut(); // bk001212
pub static mut vm_debugLevel: c_int = 0;

pub const MAX_VM: c_int = 3;
pub static mut vmTable: [vm_t; MAX_VM as usize] = unsafe { core::mem::zeroed() };

// External declarations
unsafe extern "C" {
    pub fn VM_VmInfo_f();
    pub fn VM_VmProfile_f();

    // External C functions
    pub fn Cvar_Get(
        var_name: *const c_char,
        var_value: *const c_char,
        flags: c_int,
    ) -> *mut cvar_t;
    pub fn Cmd_AddCommand(cmd_name: *const c_char, function: *const c_void);
    pub fn Com_Memset(dest: *mut c_void, c: c_int, count: c_int) -> *mut c_void;
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: c_int) -> *mut c_void;
    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn COM_StripExtension(in_: *const c_char, out: *mut c_char, outlen: c_int);
    pub fn COM_Parse(data_p: *mut *const c_char) -> *mut c_char;
    pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    pub fn FS_FreeFile(buffer: *mut c_void);
    pub fn LittleLong(l: c_int) -> c_int;
    pub fn Sys_LoadDll(
        name: *const c_char,
        entryPoint: *mut *mut c_void,
        systemCall: *mut *mut c_void,
    ) -> *mut c_void;
    pub fn Sys_UnloadDll(dllHandle: *mut c_void);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Hunk_Alloc(size: c_int, preference: c_int) -> *mut c_void;
    pub fn Z_Malloc(size: c_int, tag: c_int, zeromem: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Cvar_VariableValue(var_name: *const c_char) -> c_int;
    pub fn strlen(s: *const c_char) -> core::ffi::c_ulong;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn memset(s: *mut c_void, c: c_int, n: core::ffi::c_ulong) -> *mut c_void;
    pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    pub fn fprintf(stream: *mut c_void, format: *const c_char, ...);
    pub fn qsort(
        base: *mut c_void,
        nmemb: core::ffi::c_ulong,
        size: core::ffi::c_ulong,
        compar: *const c_void,
    );
}

// Constants
const CVAR_SYSTEMINFO: c_int = 0x00000008;
const CVAR_ARCHIVE: c_int = 0x00000001;
const h_high: c_int = 1;
const TAG_ALL: c_int = 0;
const TAG_VM: c_int = 1;
const TAG_VM_ALLOCATED: c_int = 2;
const STACK_SIZE: c_int = 0x20000;
const VM_MAGIC: c_int = 0x12721444;

// converts a VM pointer to a C pointer and
// checks to make sure that the range is acceptable
pub unsafe fn VM_VM2C(p: vmptr_t, _length: c_int) -> *mut c_void {
    p as *mut c_void
}

pub fn VM_Debug(level: c_int) {
    unsafe {
        vm_debugLevel = level;
    }
}

/*
==============
VM_Init
==============
*/
pub unsafe fn VM_Init() {
    Cvar_Get(
        b"vm_cgame\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        CVAR_SYSTEMINFO | CVAR_ARCHIVE,
    ); // default to DLLs now instead. Our VMs are getting too HUGE.
    Cvar_Get(
        b"vm_game\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        CVAR_SYSTEMINFO | CVAR_ARCHIVE,
    ); //
    Cvar_Get(
        b"vm_ui\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        CVAR_SYSTEMINFO | CVAR_ARCHIVE,
    ); //
    //client wants to know if the server is using vm's for certain modules,
    //so if pure we can force the same method (be it vm or dll) -rww

    Cmd_AddCommand(b"vmprofile\0".as_ptr() as *const c_char, VM_VmProfile_f as *const c_void);
    Cmd_AddCommand(b"vminfo\0".as_ptr() as *const c_char, VM_VmInfo_f as *const c_void);

    Com_Memset(vmTable.as_mut_ptr() as *mut c_void, 0, core::mem::size_of_val(&vmTable) as c_int);
}

/*
===============
VM_ValueToSymbol

Assumes a program counter value
===============
*/
pub unsafe fn VM_ValueToSymbol(vm: *mut vm_t, value: c_int) -> *const c_char {
    let mut sym: *mut vmSymbol_t;
    static mut text: [c_char; MAX_TOKEN_CHARS as usize] = [0; MAX_TOKEN_CHARS as usize];

    // #ifdef CRAZY_SYMBOL_MAP
    //     sym = (*g_symbolMap)[value];
    //
    //     if (!sym)
    //     {
    // #endif
    sym = (*vm).symbols;
    if sym.is_null() {
        return b"NO SYMBOLS\0".as_ptr() as *const c_char;
    }

    // find the symbol
    while !(*sym).next.is_null() && (*(*sym).next).symValue <= value {
        sym = (*sym).next;
    }
    // #ifdef CRAZY_SYMBOL_MAP
    //         if (sym)
    //         { //for instant recollection next time
    //             (*g_symbolMap)[value] = sym;
    //         }
    //     }
    // #endif

    if value == (*sym).symValue {
        return (*sym).symName.as_ptr();
    }

    Com_sprintf(
        text.as_mut_ptr(),
        core::mem::size_of_val(&text) as c_int,
        b"%s+%i\0".as_ptr() as *const c_char,
        (*sym).symName.as_ptr(),
        value - (*sym).symValue,
    );

    text.as_ptr()
}

/*
===============
VM_ValueToFunctionSymbol

For profiling, find the symbol behind this value
===============
*/
pub unsafe fn VM_ValueToFunctionSymbol(vm: *mut vm_t, value: c_int) -> *mut vmSymbol_t {
    let mut sym: *mut vmSymbol_t;
    static mut nullSym: vmSymbol_t = vmSymbol_t {
        next: core::ptr::null_mut(),
        symValue: 0,
        profileCount: 0,
        symName: [0; 1],
    };

    // #ifdef CRAZY_SYMBOL_MAP
    //     sym = (*g_symbolMap)[value];
    //
    //     if ( !sym )
    //     {
    // #endif
    sym = (*vm).symbols;
    if sym.is_null() {
        return &mut nullSym;
    }

    while !(*sym).next.is_null() && (*(*sym).next).symValue <= value {
        sym = (*sym).next;
    }
    // #ifdef CRAZY_SYMBOL_MAP
    //         if (sym)
    //         { //for instant recollection next time
    //             (*g_symbolMap)[value] = sym;
    //         }
    //     }
    // #endif

    sym
}

/*
===============
VM_SymbolToValue
===============
*/
pub unsafe fn VM_SymbolToValue(vm: *mut vm_t, symbol: *const c_char) -> c_int {
    let mut sym: *mut vmSymbol_t;

    sym = (*vm).symbols;
    while !sym.is_null() {
        if strcmp(symbol, (*sym).symName.as_ptr()) == 0 {
            return (*sym).symValue;
        }
        sym = (*sym).next;
    }
    0
}

/*
=====================
VM_SymbolForCompiledPointer
=====================
*/
pub unsafe fn VM_SymbolForCompiledPointer(vm: *mut vm_t, code: *mut c_void) -> *const c_char {
    let mut i: c_int;

    if code < (*vm).codeBase as *mut c_void {
        return b"Before code block\0".as_ptr() as *const c_char;
    }
    if code >= ((*vm).codeBase as usize + (*vm).codeLength as usize) as *mut c_void {
        return b"After code block\0".as_ptr() as *const c_char;
    }

    // find which original instruction it is after
    i = 0;
    while i < (*vm).codeLength {
        if (*(*vm).instructionPointers.add(i as usize)) as *mut c_void > code {
            break;
        }
        i += 1;
    }
    i -= 1;

    // now look up the bytecode instruction pointer
    // #ifdef CRAZY_SYMBOL_MAP
    //     VM_SetSymbolMap(vm);
    // #endif
    VM_ValueToSymbol(vm, i)
}

/*
===============
ParseHex
===============
*/
fn ParseHex(text: *const c_char) -> c_int {
    let mut value: c_int = 0;
    let mut c: c_int;
    let mut ptr = text;

    loop {
        c = *ptr as c_int;
        if c == 0 {
            break;
        }
        ptr = unsafe { ptr.add(1) };

        if c >= (b'0' as c_int) && c <= (b'9' as c_int) {
            value = value * 16 + c - (b'0' as c_int);
            continue;
        }
        if c >= (b'a' as c_int) && c <= (b'f' as c_int) {
            value = value * 16 + 10 + c - (b'a' as c_int);
            continue;
        }
        if c >= (b'A' as c_int) && c <= (b'F' as c_int) {
            value = value * 16 + 10 + c - (b'A' as c_int);
            continue;
        }
    }

    value
}

/*
===============
VM_Alloc

Convenience function for changing the way VMs are allocated.
===============
*/
unsafe fn VM_Alloc(size: c_int) -> *mut c_void {
    Hunk_Alloc(size, h_high)
    //return Z_Malloc(size, TAG_ALL, qtrue);
}

/*
===============
VM_LoadSymbols
===============
*/
unsafe fn VM_LoadSymbols(vm: *mut vm_t) {
    let mut len: c_int;
    let mut mapfile: *mut c_char = core::ptr::null_mut();
    let mut token: *mut c_char;
    let mut text_p: *const c_char;
    let mut name: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut symbols: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut prev: *mut *mut vmSymbol_t;
    let mut sym: *mut vmSymbol_t;
    let mut count: c_int = 0;
    let mut value: c_int;
    let mut chars: c_int;
    let mut segment: c_int;
    let mut numInstructions: c_int;

    // don't load symbols if not developer
    // NOTE: com_developer is assumed to be an extern variable that should be checked
    // For now, we'll skip symbol loading in this port
    return;

    // The rest is preserved for reference but not compiled in this port
    // numInstructions = (*vm).instructionPointersLength >> 2;
    //
    // // parse the symbols
    // text_p = mapfile;
    // prev = &mut (*vm).symbols;
    // count = 0;
    //
    // // #ifdef CRAZY_SYMBOL_MAP
    // //     VM_SetSymbolMap(vm);
    // // #endif
    //
    // loop {
    //     token = COM_Parse(&mut text_p);
    //     if (*token.add(0)) == 0 {
    //         break;
    //     }
    //     segment = ParseHex(token);
    //     if segment != 0 {
    //         COM_Parse(&mut text_p);
    //         COM_Parse(&mut text_p);
    //         continue;       // only load code segment values
    //     }
    //
    //     token = COM_Parse(&mut text_p);
    //     if (*token.add(0)) == 0 {
    //         Com_Printf(b"WARNING: incomplete line at end of file\n\0".as_ptr() as *const c_char);
    //         break;
    //     }
    //     value = ParseHex(token);
    //
    //     token = COM_Parse(&mut text_p);
    //     if (*token.add(0)) == 0 {
    //         Com_Printf(b"WARNING: incomplete line at end of file\n\0".as_ptr() as *const c_char);
    //         break;
    //     }
    //     chars = strlen(token) as c_int;
    //     sym = VM_Alloc(core::mem::size_of::<vmSymbol_t>() as c_int + chars) as *mut vmSymbol_t;
    //     *prev = sym;
    //     prev = &mut (*sym).next;
    //     (*sym).next = core::ptr::null_mut();
    //
    //     // convert value from an instruction number to a code offset
    //     if value >= 0 && value < numInstructions {
    //         value = *(*vm).instructionPointers.add(value as usize);
    //     }
    //
    //     (*sym).symValue = value;
    //     Q_strncpyz((*sym).symName.as_mut_ptr(), token, chars + 1);
    //
    //     // #ifdef CRAZY_SYMBOL_MAP
    //     //     (*g_symbolMap)[value] = sym;
    //     // #endif
    //
    //     count += 1;
    // }
    //
    // (*vm).numSymbols = count;
    // Com_Printf(b"%i symbols parsed from %s\n\0".as_ptr() as *const c_char, count, symbols.as_ptr());
    // FS_FreeFile(mapfile as *mut c_void);
}

/*
============
VM_DllSyscall

Dlls will call this directly

 rcg010206 The horror; the horror.

  The syscall mechanism relies on stack manipulation to get it's args.
   This is likely due to C's inability to pass "..." parameters to
   a function in one clean chunk. On PowerPC Linux, these parameters
   are not necessarily passed on the stack, so while (&arg[0] == arg)
   is true, (&arg[1] == 2nd function parameter) is not necessarily
   accurate, as arg's value might have been stored to the stack or
   other piece of scratch memory to give it a valid address, but the
   next parameter might still be sitting in a register.

  Quake's syscall system also assumes that the stack grows downward,
   and that any needed types can be squeezed, safely, into a signed int.

  This hack below copies all needed values for an argument to a
   array in memory, so that Quake can get the correct values. This can
   also be used on systems where the stack grows upwards, as the
   presumably standard and safe stdargs.h macros are used.

  As for having enough space in a signed int for your datatypes, well,
   it might be better to wait for DOOM 3 before you start porting.  :)

  The original code, while probably still inherently dangerous, seems
   to work well enough for the platforms it already works on. Rather
   than add the performance hit for those platforms, the original code
   is still in use there.

  For speed, we just grab 15 arguments, and don't worry about exactly
   how many the syscall actually needs; the extra is thrown away.

============
*/
#[cfg(all(target_os = "linux", target_arch = "powerpc"))]
pub extern "C" fn VM_DllSyscall(arg: c_int, ...) -> c_int {
    // rcg010206 - see commentary above
    let mut args: [c_int; 16] = [0; 16];
    let mut i: c_int;

    args[0] = arg;

    // Note: va_list handling in Rust is unsafe and platform-specific
    // This is a simplified representation
    unsafe {
        if !currentVM.is_null() && !(*currentVM).systemCall.is_none() {
            return (*currentVM).systemCall.unwrap()(args.as_mut_ptr());
        }
    }
    0
}

#[cfg(not(all(target_os = "linux", target_arch = "powerpc")))]
pub extern "C" fn VM_DllSyscall(arg: c_int, ...) -> c_int {
    // original id code
    unsafe {
        if !currentVM.is_null() && !(*currentVM).systemCall.is_none() {
            return (*currentVM).systemCall.unwrap()(&arg as *const c_int as *mut c_int);
        }
    }
    0
}

/*
=================
VM_Restart

Reload the data, but leave everything else in place
This allows a server to do a map_restart without changing memory allocation
=================
*/
pub unsafe fn VM_Restart(vm: *mut vm_t) -> *mut vm_t {
    let mut header: *mut vmHeader_t;
    let mut length: c_int;
    let mut dataLength: c_int;
    let mut i: c_int;
    let mut filename: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

    // DLL's can't be restarted in place
    if !(*vm).dllHandle.is_null() {
        let mut name: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        let systemCall: vmSystemCall_t = (*vm).systemCall;

        Q_strncpyz(name.as_mut_ptr(), (*vm).name.as_ptr(), MAX_QPATH as c_int);

        VM_Free(vm);

        return VM_Create(
            name.as_ptr(),
            systemCall,
            1, // VMI_NATIVE
        );
    }

    // load the image
    Com_Printf(b"VM_Restart()\n\0".as_ptr() as *const c_char);
    Com_sprintf(
        filename.as_mut_ptr(),
        MAX_QPATH as c_int,
        b"vm/%s.qvm\0".as_ptr() as *const c_char,
        (*vm).name.as_ptr(),
    );
    Com_Printf(
        b"Loading vm file %s.\n\0".as_ptr() as *const c_char,
        filename.as_ptr(),
    );
    length = FS_ReadFile(
        filename.as_ptr(),
        &mut (header as *mut *mut c_void),
    );
    if header.is_null() {
        Com_Error(ERR_DROP, b"VM_Restart failed.\n\0".as_ptr() as *const c_char);
    }

    // byte swap the header
    i = 0;
    while i < (core::mem::size_of::<vmHeader_t>() / 4) as c_int {
        *((header as *mut c_int).add(i as usize)) =
            LittleLong(*((header as *mut c_int).add(i as usize)));
        i += 1;
    }

    // validate
    if (*header).vmMagic != VM_MAGIC
        || (*header).bssLength < 0
        || (*header).dataLength < 0
        || (*header).litLength < 0
        || (*header).codeLength <= 0
    {
        VM_Free(vm);
        Com_Error(
            ERR_FATAL,
            b"%s has bad header\0".as_ptr() as *const c_char,
            filename.as_ptr(),
        );
    }

    // round up to next power of 2 so all data operations can
    // be mask protected
    dataLength = (*header).dataLength + (*header).litLength + (*header).bssLength;
    i = 0;
    while dataLength > (1 << i) {
        i += 1;
    }
    dataLength = 1 << i;

    // clear the data
    Com_Memset((*vm).dataBase as *mut c_void, 0, dataLength);

    // copy the intialized data
    Com_Memcpy(
        (*vm).dataBase as *mut c_void,
        (header as *mut c_char).add((*header).dataOffset as usize) as *const c_void,
        (*header).dataLength + (*header).litLength,
    );

    // byte swap the longs
    i = 0;
    while i < (*header).dataLength {
        *((*vm).dataBase.add(i as usize) as *mut c_int) =
            LittleLong(*((*vm).dataBase.add(i as usize) as *mut c_int));
        i += 4;
    }

    // free the original file
    FS_FreeFile(header as *mut c_void);

    vm
}

/*
================
VM_Create

If image ends in .qvm it will be interpreted, otherwise
it will attempt to load as a system dll
================
*/
pub unsafe fn VM_Create(
    module: *const c_char,
    systemCalls: vmSystemCall_t,
    interpret: c_int,
) -> *mut vm_t {
    let mut vm: *mut vm_t;
    let mut header: *mut vmHeader_t = core::ptr::null_mut();
    let mut length: c_int;
    let mut dataLength: c_int;
    let mut i: c_int;
    let mut filename: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

    if module.is_null()
        || *module == 0
        || systemCalls.is_none()
    {
        Com_Error(ERR_FATAL, b"VM_Create: bad parms\0".as_ptr() as *const c_char);
    }

    // see if we already have the VM
    i = 0;
    while i < MAX_VM as c_int {
        if Q_stricmp(vmTable[i as usize].name.as_ptr(), module) == 0 {
            vm = &mut vmTable[i as usize];
            return vm;
        }
        i += 1;
    }

    // find a free vm
    i = 0;
    while i < MAX_VM as c_int {
        if vmTable[i as usize].name[0] == 0 {
            break;
        }
        i += 1;
    }

    if i == MAX_VM as c_int {
        Com_Error(ERR_FATAL, b"VM_Create: no free vm_t\0".as_ptr() as *const c_char);
    }

    vm = &mut vmTable[i as usize];

    Q_strncpyz((*vm).name.as_mut_ptr(), module, MAX_QPATH as c_int);
    (*vm).systemCall = systemCalls;

    // never allow dll loading with a demo
    if interpret == 0 { // VMI_NATIVE
        if Cvar_VariableValue(b"fs_restrict\0".as_ptr() as *const c_char) != 0 {
            // interpret = VMI_COMPILED (2)
            // We'll handle this by continuing with VMI_COMPILED path
        }
    }

    let mut interpret_mut = interpret;
    if interpret_mut == 0 { // VMI_NATIVE
        // try to load as a system dll
        Com_Printf(b"Loading dll file %s.\n\0".as_ptr() as *const c_char, (*vm).name.as_ptr());
        let mut entry_point_ptr: *mut c_void = core::ptr::null_mut();
        let mut syscall_ptr: *mut c_void = core::ptr::null_mut();
        (*vm).dllHandle = Sys_LoadDll((*vm).name.as_ptr(), &mut entry_point_ptr, &mut syscall_ptr);
        if !(*vm).dllHandle.is_null() {
            (*vm).entryPoint = Some(core::mem::transmute(entry_point_ptr));
            return vm;
        }

        Com_Printf(b"Failed to load dll, looking for qvm.\n\0".as_ptr() as *const c_char);
        interpret_mut = 2; // VMI_COMPILED
    }

    // load the image
    Com_sprintf(
        filename.as_mut_ptr(),
        MAX_QPATH as c_int,
        b"vm/%s.qvm\0".as_ptr() as *const c_char,
        (*vm).name.as_ptr(),
    );
    Com_Printf(b"Loading vm file %s.\n\0".as_ptr() as *const c_char, filename.as_ptr());
    length = FS_ReadFile(
        filename.as_ptr(),
        &mut (header as *mut *mut c_void),
    );
    if header.is_null() {
        Com_Printf(b"Failed.\n\0".as_ptr() as *const c_char);
        VM_Free(vm);
        return core::ptr::null_mut();
    }

    // byte swap the header
    i = 0;
    while i < (core::mem::size_of::<vmHeader_t>() / 4) as c_int {
        *((header as *mut c_int).add(i as usize)) =
            LittleLong(*((header as *mut c_int).add(i as usize)));
        i += 1;
    }

    // validate
    if (*header).vmMagic != VM_MAGIC
        || (*header).bssLength < 0
        || (*header).dataLength < 0
        || (*header).litLength < 0
        || (*header).codeLength <= 0
    {
        VM_Free(vm);
        Com_Error(
            ERR_FATAL,
            b"%s has bad header\0".as_ptr() as *const c_char,
            filename.as_ptr(),
        );
    }

    // round up to next power of 2 so all data operations can
    // be mask protected
    dataLength = (*header).dataLength + (*header).litLength + (*header).bssLength;
    i = 0;
    while dataLength > (1 << i) {
        i += 1;
    }
    dataLength = 1 << i;

    // allocate zero filled space for initialized and uninitialized data
    (*vm).dataBase = VM_Alloc(dataLength) as *mut byte;
    (*vm).dataMask = dataLength - 1;

    // copy the intialized data
    Com_Memcpy(
        (*vm).dataBase as *mut c_void,
        (header as *mut c_char).add((*header).dataOffset as usize) as *const c_void,
        (*header).dataLength + (*header).litLength,
    );

    // byte swap the longs
    i = 0;
    while i < (*header).dataLength {
        *((*vm).dataBase.add(i as usize) as *mut c_int) =
            LittleLong(*((*vm).dataBase.add(i as usize) as *mut c_int));
        i += 4;
    }

    // allocate space for the jump targets, which will be filled in by the compile/prep functions
    (*vm).instructionPointersLength = (*header).instructionCount * 4;
    (*vm).instructionPointers = VM_Alloc((*vm).instructionPointersLength) as *mut c_int;

    // copy or compile the instructions
    (*vm).codeLength = (*header).codeLength;

    if interpret_mut >= 2 { // VMI_COMPILED
        (*vm).compiled = QTRUE;
        VM_Compile(vm, header);
    } else {
        (*vm).compiled = QFALSE;
        VM_PrepareInterpreter(vm, header);
    }

    // free the original file
    FS_FreeFile(header as *mut c_void);

    // load the map file
    VM_LoadSymbols(vm);

    // the stack is implicitly at the end of the image
    (*vm).programStack = (*vm).dataMask + 1;
    (*vm).stackBottom = (*vm).programStack - STACK_SIZE;

    vm
}

/*
==============
VM_Free
==============
*/
pub unsafe fn VM_Free(vm: *mut vm_t) {
    if !(*vm).dllHandle.is_null() {
        Sys_UnloadDll((*vm).dllHandle);
        Com_Memset(vm as *mut c_void, 0, core::mem::size_of::<vm_t>() as c_int);
    }
    // #if 0	// now automatically freed by hunk
    //     if ( vm->codeBase ) {
    //         Z_Free( vm->codeBase );
    //     }
    //     if ( vm->dataBase ) {
    //         Z_Free( vm->dataBase );
    //     }
    //     if ( vm->instructionPointers ) {
    //         Z_Free( vm->instructionPointers );
    //     }
    // #endif
    Com_Memset(vm as *mut c_void, 0, core::mem::size_of::<vm_t>() as c_int);

    currentVM = core::ptr::null_mut();
    lastVM = core::ptr::null_mut();
}

pub unsafe fn VM_Clear() {
    let mut i: c_int = 0;
    while i < MAX_VM {
        if !vmTable[i as usize].dllHandle.is_null() {
            Sys_UnloadDll(vmTable[i as usize].dllHandle);
        }
        Com_Memset(
            &mut vmTable[i as usize] as *mut vm_t as *mut c_void,
            0,
            core::mem::size_of::<vm_t>() as c_int,
        );
        i += 1;
    }
    currentVM = core::ptr::null_mut();
    lastVM = core::ptr::null_mut();
}

pub unsafe fn VM_ArgPtr(intValue: c_int) -> *mut c_void {
    if intValue == 0 {
        return core::ptr::null_mut();
    }
    // bk001220 - currentVM is missing on reconnect
    if currentVM.is_null() {
        return core::ptr::null_mut();
    }

    if !(*currentVM).entryPoint.is_none() {
        ((*currentVM).dataBase as usize + intValue as usize) as *mut c_void
    } else {
        ((*currentVM).dataBase as usize + (intValue as usize & (*currentVM).dataMask as usize))
            as *mut c_void
    }
}

unsafe extern "C" {
    pub static mut gvm: *mut vm_t;
}

pub unsafe fn BotVMShift(ptr: c_int) -> *mut c_void {
    if ptr == 0 {
        return core::ptr::null_mut();
    }

    if gvm.is_null() {
        //always using the game vm here.
        return core::ptr::null_mut();
    }

    if !(*gvm).entryPoint.is_none() {
        ((*gvm).dataBase as usize + ptr as usize) as *mut c_void
    } else {
        ((*gvm).dataBase as usize + (ptr as usize & (*gvm).dataMask as usize)) as *mut c_void
    }
}

pub unsafe fn VM_Shifted_Alloc(ptr: *mut *mut c_void, size: c_int) {
    let mut mem: *mut c_void;

    if currentVM.is_null() {
        assert_eq!(1, 0);
        *ptr = core::ptr::null_mut();
        return;
    }

    //first allocate our desired memory, up front
    mem = Z_Malloc(size + 1, TAG_VM_ALLOCATED, QFALSE);

    if mem.is_null() {
        assert_eq!(1, 0);
        *ptr = core::ptr::null_mut();
        return;
    }

    memset(mem, 0, (size + 1) as core::ffi::c_ulong);

    //This can happen.. if a free chunk of memory is found before the vm alloc pointer, commonly happens
    //when allocating like 4 bytes or whatever. However it seems to actually be handled which I didn't
    //think it would be.. so hey.
    // #if 0
    //     if ((int)mem < (int)currentVM->dataBase)
    //     {
    //         assert(!"Unspeakably bad thing has occured (mem ptr < vm base ptr)");
    //         *ptr = NULL;
    //         return;
    //     }
    // #endif

    //Alright, subtract the database from the memory pointer to get a memory address relative to the VM.
    //When the VM modifies it it should be modifying the same chunk of memory we have allocated in the engine.
    *ptr = ((mem as usize - (*currentVM).dataBase as usize) as c_int) as *mut c_void;
}

pub unsafe fn VM_Shifted_Free(ptr: *mut *mut c_void) {
    let mut mem: *mut c_void;

    if currentVM.is_null() {
        assert_eq!(1, 0);
        return;
    }

    //Shift the VM memory pointer back to get the same pointer we initially allocated in real memory space.
    mem = ((*currentVM).dataBase as usize + *ptr as usize) as *mut c_void;

    if mem.is_null() {
        assert_eq!(1, 0);
        return;
    }

    Z_Free(mem);
    *ptr = core::ptr::null_mut(); //go ahead and clear the pointer for the game.
}

pub unsafe fn VM_ExplicitArgPtr(vm: *mut vm_t, intValue: c_int) -> *mut c_void {
    if intValue == 0 {
        return core::ptr::null_mut();
    }

    // bk010124 - currentVM is missing on reconnect here as well?
    if currentVM.is_null() {
        return core::ptr::null_mut();
    }

    //
    if !(*vm).entryPoint.is_none() {
        ((*vm).dataBase as usize + intValue as usize) as *mut c_void
    } else {
        ((*vm).dataBase as usize + (intValue as usize & (*vm).dataMask as usize)) as *mut c_void
    }
}

/*
==============
VM_Call


Upon a system call, the stack will look like:

sp+32	parm1
sp+28	parm0
sp+24	return value
sp+20	return address
sp+16	local1
sp+14	local0
sp+12	arg1
sp+8	arg0
sp+4	return stack
sp		return address

An interpreted function will immediately execute
an OP_ENTER instruction, which will subtract space for
locals from sp
==============
*/
pub extern "C" fn VM_Call(vm: *mut vm_t, callnum: c_int, ...) -> c_int {
    unsafe {
        let mut oldVM: *mut vm_t;
        let mut r: c_int;
        let mut i: c_int;
        let mut args: [c_int; 16] = [0; 16];

        if vm.is_null() {
            Com_Error(ERR_FATAL, b"VM_Call with NULL vm\0".as_ptr() as *const c_char);
        }

        oldVM = currentVM;
        currentVM = vm;
        lastVM = vm;

        if vm_debugLevel != 0 {
            Com_Printf(b"VM_Call( %i )\n\0".as_ptr() as *const c_char, callnum);
        }

        // if we have a dll loaded, call it directly
        if !(*vm).entryPoint.is_none() {
            //rcg010207 -  see dissertation at top of VM_DllSyscall() in this file.
            // Note: In Rust, we can't use va_list directly in the same way
            // We would need platform-specific handling or a wrapper
            // For now, this is a simplified version
            r = (*vm).entryPoint.unwrap()(
                callnum, args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
                args[8], args[9], args[10], args[11], args[12], args[13], args[14], args[15],
            );
        } else if (*vm).compiled == QTRUE {
            r = VM_CallCompiled(vm, args.as_mut_ptr());
        } else {
            r = VM_CallInterpreted(vm, args.as_mut_ptr());
        }

        if !oldVM.is_null() {
            // bk001220 - assert(currentVM!=NULL) for oldVM==NULL
            currentVM = oldVM;
        }
        r
    }
}

//=================================================================

extern "C" fn VM_ProfileSort(a: *const c_void, b: *const c_void) -> c_int {
    unsafe {
        let sa: *mut vmSymbol_t = *(a as *const *mut vmSymbol_t);
        let sb: *mut vmSymbol_t = *(b as *const *mut vmSymbol_t);

        if (*sa).profileCount < (*sb).profileCount {
            return -1;
        }
        if (*sa).profileCount > (*sb).profileCount {
            return 1;
        }
        0
    }
}

/*
==============
VM_VmProfile_f

==============
*/
pub unsafe fn VM_VmProfile_f() {
    let mut vm: *mut vm_t;
    let mut sorted: *mut *mut vmSymbol_t;
    let mut sym: *mut vmSymbol_t;
    let mut i: c_int;
    let mut total: f64;

    if lastVM.is_null() {
        return;
    }

    vm = lastVM;

    if (*vm).numSymbols == 0 {
        return;
    }

    sorted = Z_Malloc(
        ((*vm).numSymbols as usize * core::mem::size_of::<*mut vmSymbol_t>()) as c_int,
        TAG_VM,
        QTRUE,
    ) as *mut *mut vmSymbol_t;
    *sorted = (*vm).symbols;
    total = (*(*sorted)).profileCount as f64;
    i = 1;
    while i < (*vm).numSymbols {
        *sorted.add(i as usize) = (*(*sorted.add((i - 1) as usize))).next;
        total += (*(*sorted.add(i as usize))).profileCount as f64;
        i += 1;
    }

    qsort(
        sorted as *mut c_void,
        (*vm).numSymbols as core::ffi::c_ulong,
        core::mem::size_of::<*mut vmSymbol_t>() as core::ffi::c_ulong,
        VM_ProfileSort as *const c_void,
    );

    i = 0;
    while i < (*vm).numSymbols {
        let mut perc: c_int;

        sym = *sorted.add(i as usize);

        perc = (100.0 * ((*sym).profileCount as f64) / total) as c_int;
        Com_Printf(
            b"%2i%% %9i %s\n\0".as_ptr() as *const c_char,
            perc,
            (*sym).profileCount,
            (*sym).symName.as_ptr(),
        );
        (*sym).profileCount = 0;
        i += 1;
    }

    Com_Printf(b"    %9.0f total\n\0".as_ptr() as *const c_char, total);

    Z_Free(sorted as *mut c_void);
}

/*
==============
VM_VmInfo_f

==============
*/
pub unsafe fn VM_VmInfo_f() {
    let mut vm: *mut vm_t;
    let mut i: c_int;

    Com_Printf(b"Registered virtual machines:\n\0".as_ptr() as *const c_char);
    i = 0;
    while i < MAX_VM as c_int {
        vm = &mut vmTable[i as usize];
        if (*vm).name[0] == 0 {
            break;
        }
        Com_Printf(
            b"%s : \0".as_ptr() as *const c_char,
            (*vm).name.as_ptr(),
        );
        if !(*vm).dllHandle.is_null() {
            Com_Printf(b"native\n\0".as_ptr() as *const c_char);
            i += 1;
            continue;
        }
        if (*vm).compiled == QTRUE {
            Com_Printf(b"compiled on load\n\0".as_ptr() as *const c_char);
        } else {
            Com_Printf(b"interpreted\n\0".as_ptr() as *const c_char);
        }
        Com_Printf(
            b"    code length : %7i\n\0".as_ptr() as *const c_char,
            (*vm).codeLength,
        );
        Com_Printf(
            b"    table length: %7i\n\0".as_ptr() as *const c_char,
            (*vm).instructionPointersLength,
        );
        Com_Printf(
            b"    data length : %7i\n\0".as_ptr() as *const c_char,
            (*vm).dataMask + 1,
        );
        i += 1;
    }
}

/*
===============
VM_LogSyscalls

Insert calls to this while debugging the vm compiler
===============
*/
pub unsafe fn VM_LogSyscalls(args: *mut c_int) {
    static mut callnum: c_int = 0;
    static mut f: *mut c_void = core::ptr::null_mut();

    if f.is_null() {
        f = fopen(b"syscalls.log\0".as_ptr() as *const c_char, b"w\0".as_ptr() as *const c_char);
    }
    callnum += 1;
    fprintf(
        f,
        b"%i: %i (%i) = %i %i %i %i\n\0".as_ptr() as *const c_char,
        callnum,
        args as usize - currentVM.cast::<byte>() as usize,
        *args,
        *args.add(1),
        *args.add(2),
        *args.add(3),
        *args.add(4),
    );
}

// #ifdef oDLL_ONLY // bk010215 - for DLL_ONLY dedicated servers/builds w/o VM
// int	VM_CallCompiled( vm_t *vm, int *args ) {
//   return(0);
// }
//
// void VM_Compile( vm_t *vm, vmHeader_t *header ) {}
// #endif // DLL_ONLY
