//! Mechanical port of `codemp/qcommon/vm_console.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, qboolean};
use crate::codemp::qcommon::vm_local_h::{vm_t, vmInterpret_t};
use crate::codemp::qcommon::tags_h::TAG_VM_ALLOCATED;
use crate::ffi::types::QTRUE;
use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

// External functions from other modules
extern "C" {
    /// `int Q_stricmp (const char *s1, const char *s2)`.
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    /// `void* Z_Malloc(int size, int tag, qboolean zero)`
    pub fn Z_Malloc(size: c_int, tag: c_int, zero: qboolean) -> *mut c_void;

    /// `void Z_Free(void *ptr)`
    pub fn Z_Free(ptr: *mut c_void);
}

// External functions from cgame, game, ui namespaces
extern "C" {
    pub fn cgame_vmMain(
        command: c_int,
        arg0: c_int,
        arg1: c_int,
        arg2: c_int,
        arg3: c_int,
        arg4: c_int,
        arg5: c_int,
        arg6: c_int,
        arg7: c_int,
        arg8: c_int,
        arg9: c_int,
        arg10: c_int,
        arg11: c_int,
    ) -> c_int;
    pub fn cgame_dllEntry(syscallptr: extern "C" fn(c_int, ...) -> c_int);

    pub fn game_vmMain(
        command: c_int,
        arg0: c_int,
        arg1: c_int,
        arg2: c_int,
        arg3: c_int,
        arg4: c_int,
        arg5: c_int,
        arg6: c_int,
        arg7: c_int,
        arg8: c_int,
        arg9: c_int,
        arg10: c_int,
        arg11: c_int,
    ) -> c_int;
    pub fn game_dllEntry(syscallptr: extern "C" fn(c_int, ...) -> c_int);

    pub fn ui_vmMain(
        command: c_int,
        arg0: c_int,
        arg1: c_int,
        arg2: c_int,
        arg3: c_int,
        arg4: c_int,
        arg5: c_int,
        arg6: c_int,
        arg7: c_int,
        arg8: c_int,
        arg9: c_int,
        arg10: c_int,
        arg11: c_int,
    ) -> c_int;
    pub fn ui_dllEntry(syscallptr: extern "C" fn(c_int, ...) -> c_int);
}

pub static mut currentVM: *mut vm_t = core::ptr::null_mut();
pub static mut lastVM: *mut vm_t = core::ptr::null_mut();

const MAX_VM: usize = 3;
const UI_VM_INDEX: usize = 0;
const CG_VM_INDEX: usize = 1;
const G_VM_INDEX: usize = 2;

// SAFETY: vmTable is zero-initialized as all-null pointers, which is safe for vm_t.
// It is accessed through raw pointers only, matching C semantics.
pub static mut vmTable: [vm_t; MAX_VM] = [
    vm_t {
        programStack: 0,
        systemCall: None,
        name: [0; 64],
        dllHandle: core::ptr::null_mut(),
        entryPoint: None,
        currentlyInterpreting: 0,
        compiled: 0,
        codeBase: core::ptr::null_mut(),
        codeLength: 0,
        instructionPointers: core::ptr::null_mut(),
        instructionPointersLength: 0,
        dataBase: core::ptr::null_mut(),
        dataMask: 0,
        stackBottom: 0,
        numSymbols: 0,
        symbols: core::ptr::null_mut(),
        callLevel: 0,
        breakFunction: 0,
        breakCount: 0,
    },
    vm_t {
        programStack: 0,
        systemCall: None,
        name: [0; 64],
        dllHandle: core::ptr::null_mut(),
        entryPoint: None,
        currentlyInterpreting: 0,
        compiled: 0,
        codeBase: core::ptr::null_mut(),
        codeLength: 0,
        instructionPointers: core::ptr::null_mut(),
        instructionPointersLength: 0,
        dataBase: core::ptr::null_mut(),
        dataMask: 0,
        stackBottom: 0,
        numSymbols: 0,
        symbols: core::ptr::null_mut(),
        callLevel: 0,
        breakFunction: 0,
        breakCount: 0,
    },
    vm_t {
        programStack: 0,
        systemCall: None,
        name: [0; 64],
        dllHandle: core::ptr::null_mut(),
        entryPoint: None,
        currentlyInterpreting: 0,
        compiled: 0,
        codeBase: core::ptr::null_mut(),
        codeLength: 0,
        instructionPointers: core::ptr::null_mut(),
        instructionPointersLength: 0,
        dataBase: core::ptr::null_mut(),
        dataMask: 0,
        stackBottom: 0,
        numSymbols: 0,
        symbols: core::ptr::null_mut(),
        callLevel: 0,
        breakFunction: 0,
        breakCount: 0,
    },
];

//============
// VM_DllSyscall
//
// Dlls will call this directly
//
//  rcg010206 The horror; the horror.
//
//   The syscall mechanism relies on stack manipulation to get it's args.
//    This is likely due to C's inability to pass "..." parameters to
//    a function in one clean chunk. On PowerPC Linux, these parameters
//    are not necessarily passed on the stack, so while (&arg[0] == arg)
//    is true, (&arg[1] == 2nd function parameter) is not necessarily
//    accurate, as arg's value might have been stored to the stack or
//    other piece of scratch memory to give it a valid address, but the
//    next parameter might still be sitting in a register.
//
//   Quake's syscall system also assumes that the stack grows downward,
//    and that any needed types can be squeezed, safely, into a signed int.
//
//   This hack below copies all needed values for an argument to a
//    array in memory, so that Quake can get the correct values. This can
//    also be used on systems where the stack grows upwards, as the
//    presumably standard and safe stdargs.h macros are used.
//
//   As for having enough space in a signed int for your datatypes, well,
//    it might be better to wait for DOOM 3 before you start porting.  :)
//
//   The original code, while probably still inherently dangerous, seems
//    to work well enough for the platforms it already works on. Rather
//    than add the performance hit for those platforms, the original code
//    is still in use there.
//
//   For speed, we just grab 15 arguments, and don't worry about exactly
//    how many the syscall actually needs; the extra is thrown away.
//
//============
#[no_mangle]
pub extern "C" fn VM_DllSyscall(arg: c_int, _: ...) -> c_int {
    unsafe {
        let vm = *addr_of_mut!(currentVM);
        if let Some(syscall) = (*vm).systemCall {
            // SAFETY: Taking the address of the first parameter to pass stack args to systemCall,
            // matching C's variadic syscall behavior. The cast from *const to *mut is
            // intentional to preserve C semantics of stack manipulation.
            syscall(addr_of!(arg) as *const c_int as *mut c_int)
        } else {
            0
        }
    }
}

//================
// VM_Create
//
// If image ends in .qvm it will be interpreted, otherwise
// it will attempt to load as a system dll
//================

// #define	STACK_SIZE	0x20000

#[no_mangle]
pub extern "C" fn VM_Create(
    module: *const c_char,
    systemCalls: Option<unsafe extern "C" fn(*mut c_int) -> c_int>,
    interpret: vmInterpret_t,
) -> *mut vm_t {
    unsafe {
        if Q_stricmp(module, b"ui\0".as_ptr() as *const c_char) == 0 {
            // UI VM
            // SAFETY: Casting from fixed-arg function to variadic function pointer to match C semantics.
            (*addr_of_mut!(vmTable[UI_VM_INDEX])).entryPoint =
                Some(core::mem::transmute::<extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int) -> c_int, unsafe extern "C" fn(c_int, ...) -> c_int>(
                    ui_vmMain,
                ));
            (*addr_of_mut!(vmTable[UI_VM_INDEX])).systemCall = systemCalls;
            ui_dllEntry(VM_DllSyscall);
            addr_of_mut!(vmTable[UI_VM_INDEX])
        } else if Q_stricmp(module, b"cgame\0".as_ptr() as *const c_char) == 0 {
            // CG VM
            // SAFETY: Casting from fixed-arg function to variadic function pointer to match C semantics.
            (*addr_of_mut!(vmTable[CG_VM_INDEX])).entryPoint =
                Some(core::mem::transmute::<extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int) -> c_int, unsafe extern "C" fn(c_int, ...) -> c_int>(
                    cgame_vmMain,
                ));
            (*addr_of_mut!(vmTable[CG_VM_INDEX])).systemCall = systemCalls;
            cgame_dllEntry(VM_DllSyscall);
            addr_of_mut!(vmTable[CG_VM_INDEX])
        } else if Q_stricmp(module, b"jampgame\0".as_ptr() as *const c_char) == 0 {
            // G VM
            // SAFETY: Casting from fixed-arg function to variadic function pointer to match C semantics.
            (*addr_of_mut!(vmTable[G_VM_INDEX])).entryPoint =
                Some(core::mem::transmute::<extern "C" fn(c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int, c_int) -> c_int, unsafe extern "C" fn(c_int, ...) -> c_int>(
                    game_vmMain,
                ));
            (*addr_of_mut!(vmTable[G_VM_INDEX])).systemCall = systemCalls;
            game_dllEntry(VM_DllSyscall);
            addr_of_mut!(vmTable[G_VM_INDEX])
        } else {
            core::ptr::null_mut()
        }
    }
}

//==============
// VM_Call
//
//
// Upon a system call, the stack will look like:
//
// sp+32	parm1
// sp+28	parm0
// sp+24	return value
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
//==============
const MAX_STACK: c_int = 256;
const STACK_MASK: c_int = MAX_STACK - 1;

#[no_mangle]
pub extern "C" fn VM_Call(vm: *mut vm_t, callnum: c_int, _: ...) -> c_int {
    unsafe {
        // Remember what the current VM was when we started.
        let oldVM = *addr_of_mut!(currentVM);
        // Change current VM so that VMA() crap works
        *addr_of_mut!(currentVM) = vm;

        // Forward the call to the vm's vmMain function, passing through more data than
        // we should. I'm going to be sick.
        #[cfg(target_env = "gamecube")]
        {
            // PowerPC calling convention: parameters are passed in registers, not on stack.
            // On GameCube, we copy all variadic arguments to an array before calling entryPoint.
            // Note: This branch requires special handling for VaList, which is platform-specific.
            // For now, use the same pointer-arithmetic approach as other platforms.
            let callnum_addr = addr_of!(callnum) as *const c_int;
            let r = (*vm).entryPoint.map(|f| {
                f(
                    *callnum_addr,
                    *callnum_addr.add(1),
                    *callnum_addr.add(2),
                    *callnum_addr.add(3),
                    *callnum_addr.add(4),
                    *callnum_addr.add(5),
                    *callnum_addr.add(6),
                    *callnum_addr.add(7),
                    *callnum_addr.add(8),
                    *callnum_addr.add(9),
                    *callnum_addr.add(10),
                    *callnum_addr.add(11),
                    *callnum_addr.add(12),
                )
            }).unwrap_or(0);

            // Restore VM pointer XXX: Why does the below code check for non-NULL?
            *addr_of_mut!(currentVM) = oldVM;
            return r;
        }

        #[cfg(not(target_env = "gamecube"))]
        {
            let callnum_addr = addr_of!(callnum) as *const c_int;
            let r = (*vm).entryPoint.map(|f| {
                f(
                    *callnum_addr,
                    *callnum_addr.add(1),
                    *callnum_addr.add(2),
                    *callnum_addr.add(3),
                    *callnum_addr.add(4),
                    *callnum_addr.add(5),
                    *callnum_addr.add(6),
                    *callnum_addr.add(7),
                    *callnum_addr.add(8),
                    *callnum_addr.add(9),
                    *callnum_addr.add(10),
                    *callnum_addr.add(11),
                    *callnum_addr.add(12),
                )
            }).unwrap_or(0);

            // Restore VM pointer XXX: Why does the below code check for non-NULL?
            *addr_of_mut!(currentVM) = oldVM;
            return r;
        }
    }
}

// This function seems really suspect. Let's cross our fingers...
#[no_mangle]
pub extern "C" fn BotVMShift(ptr: c_int) -> *mut c_void {
    ptr as *mut c_void
}

// Functions to support dynamic memory allocation by VMs.
// I don't really trust these. Oh well.
#[no_mangle]
pub extern "C" fn VM_Shifted_Alloc(ptr: *mut *mut c_void, size: c_int) {
    unsafe {
        if (*addr_of_mut!(currentVM)).is_null() {
            // assert(0);
            *ptr = core::ptr::null_mut();
            return;
        }

        // first allocate our desired memory, up front
        *ptr = Z_Malloc(size, TAG_VM_ALLOCATED, QTRUE as qboolean);
    }
}

#[no_mangle]
pub extern "C" fn VM_Shifted_Free(ptr: *mut *mut c_void) {
    unsafe {
        if (*addr_of_mut!(currentVM)).is_null() {
            // assert(0);
            return;
        }

        Z_Free(*ptr);
        *ptr = core::ptr::null_mut(); // go ahead and clear the pointer for the game.
    }
}

// Stupid casting function. We can't do this in the macros, because sv_game calls this
// directly now.
#[no_mangle]
pub extern "C" fn VM_ArgPtr(intValue: c_int) -> *mut c_void {
    intValue as *mut c_void
}

#[no_mangle]
pub extern "C" fn VM_Free(_: *mut vm_t) {}

#[no_mangle]
pub extern "C" fn VM_Debug(_: c_int) {}

#[no_mangle]
pub extern "C" fn VM_Clear() {}

#[no_mangle]
pub extern "C" fn VM_Init() {}

#[no_mangle]
pub extern "C" fn VM_ExplicitArgPtr(_: *mut vm_t, _: c_int) -> *mut c_void {
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn VM_Restart(vm: *mut vm_t) -> *mut vm_t {
    vm
}
