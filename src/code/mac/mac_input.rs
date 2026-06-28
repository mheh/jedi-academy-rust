#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::addr_of_mut;

// Opaque types from InputSprocket.h
#[repr(C)]
pub struct ISpDevice;
pub type ISpDeviceReference = *mut ISpDevice;

#[repr(C)]
pub struct ISpElementList;
pub type ISpElementListReference = *mut ISpElementList;

#[repr(C)]
pub struct ISpElement;
pub type ISpElementReference = *mut ISpElement;

// Type aliases for Mac/C types
pub type qboolean = bool;
pub type OSStatus = i32;
pub type UInt32 = u32;
pub type Boolean = bool;

// Opaque Quake engine types (incomplete stubs - defined in ../client/client.h)
#[repr(C)]
pub struct cvar_t;

#[repr(C)]
pub struct ClientState {
    pub keyCatchers: c_int,
    pub state: c_int,
}

#[repr(C)]
pub struct GlConfig {
    pub isFullscreen: bool,
}

// Opaque InputSprocket types (incomplete stubs - defined in InputSprocket.h)
#[repr(C)]
pub struct NumVersion;

#[repr(C)]
pub struct ISpElementInfo {
    pub theString: *mut c_char,
}

#[repr(C)]
pub struct ISpElementEvent {
    pub data: UInt32,
}

// Globals
pub static mut inputActive: qboolean = false;
pub static mut inputSuspended: qboolean = false;

const MAX_DEVICES: usize = 100;
pub static mut devices: [ISpDeviceReference; MAX_DEVICES] = [core::ptr::null_mut(); MAX_DEVICES];
pub static mut elementList: ISpElementListReference = core::ptr::null_mut();

const MAX_ELEMENTS: usize = 512;
const MAX_MOUSE_DEVICES: usize = 2;
pub static mut numDevices: UInt32 = 0;
pub static mut numElements: [UInt32; MAX_MOUSE_DEVICES] = [0; MAX_MOUSE_DEVICES];
pub static mut elements: [[ISpElementReference; MAX_ELEMENTS]; MAX_MOUSE_DEVICES] = [[core::ptr::null_mut(); MAX_ELEMENTS]; MAX_MOUSE_DEVICES];

pub static mut in_nomouse: *mut cvar_t = core::ptr::null_mut();

// Forward declarations
extern "C" {
    pub fn Input_Init();
    pub fn Input_GetState();
}

// External C functions
extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn ISpGetVersion() -> NumVersion;
    pub fn ISpStartup() -> OSStatus;
    pub fn ISpDevices_Extract(maxDevices: UInt32, numDevices: *mut UInt32, devices: *mut ISpDeviceReference) -> OSStatus;
    pub fn ISpDevices_Deactivate(numDevices: UInt32, devices: *mut ISpDeviceReference) -> OSStatus;
    pub fn ISpDevices_ExtractByClass(deviceClass: c_int, maxDevices: UInt32, numDevices: *mut UInt32, devices: *mut ISpDeviceReference) -> OSStatus;
    pub fn ISpDevices_Activate(numDevices: UInt32, devices: *mut ISpDeviceReference) -> OSStatus;
    pub fn ISpDevice_GetElementList(device: ISpDeviceReference, elementList: *mut ISpElementListReference) -> OSStatus;
    pub fn ISpElementList_Extract(elementList: ISpElementListReference, maxElements: UInt32, numElements: *mut UInt32, elements: *mut ISpElementReference) -> OSStatus;
    pub fn ISpElement_GetInfo(element: ISpElementReference, info: *mut ISpElementInfo) -> OSStatus;
    pub fn PStringToCString(pstring: *mut c_char);
    pub fn HideCursor();
    pub fn ShowCursor();
    pub fn ISpShutdown() -> OSStatus;
    pub fn ISpSuspend() -> OSStatus;
    pub fn ISpResume() -> OSStatus;
    pub fn ISpElement_GetNextEvent(element: ISpElementReference, eventSize: usize, event: *mut ISpElementEvent, wasEvent: *mut Boolean) -> OSStatus;
    pub fn Sys_QueEvent(time: c_int, type_: c_int, value: c_int, value2: c_int, ptrLength: c_int, ptr: *mut c_void);
    pub fn ISpElement_GetSimpleState(element: ISpElementReference, state: *mut UInt32) -> OSStatus;

    pub static mut com_dedicated: *mut cvar_t;
    pub static mut cls: ClientState;
    pub static mut glConfig: GlConfig;
}

// Constants
const MAC_MOUSE_SCALE: c_int = 163; // why this constant?

// Placeholder constants for Quake key codes and event types (should come from Quake headers)
const K_MOUSE1: c_int = 0;
const SE_KEY: c_int = 1;
const SE_MOUSE: c_int = 2;
const CA_ACTIVE: c_int = 3;
const kISpDeviceClass_Mouse: c_int = 0;

/*
=================
Sys_InitInput
=================
*/
pub unsafe fn Sys_InitInput() {
    let mut ver: NumVersion;
    let mut info: ISpElementInfo;
    let mut i: c_int;
    let mut j: c_int;
    let mut err: OSStatus;

    // no input with dedicated servers
    if (*com_dedicated).integer != 0 {
        return;
    }

    Com_Printf(b"------- Input Initialization -------\n\0".as_ptr() as *const c_char);
    in_nomouse = Cvar_Get(
        b"in_nomouse\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        0,
    );
    if (*in_nomouse).integer != 0 {
        Com_Printf(b"in_nomouse is set, skipping.\n\0".as_ptr() as *const c_char);
        Com_Printf(b"------------------------------------\n\0".as_ptr() as *const c_char);
        return;
    }

    ver = ISpGetVersion();
    Com_Printf(b"InputSprocket version: 0x%x\n\0".as_ptr() as *const c_char, ver);

    err = ISpStartup();
    if err != 0 {
        Com_Printf(b"ISpStartup failed: %i\n\0".as_ptr() as *const c_char, err);
        Com_Printf(b"------------------------------------\n\0".as_ptr() as *const c_char);
        return;
    }

    // disable everything
    ISpDevices_Extract(MAX_DEVICES as UInt32, addr_of_mut!(numDevices), devices.as_mut_ptr());
    Com_Printf(b"%i total devices\n\0".as_ptr() as *const c_char, numDevices);
    if numDevices > MAX_DEVICES as UInt32 {
        numDevices = MAX_DEVICES as UInt32;
    }
    err = ISpDevices_Deactivate(
            numDevices,
            devices.as_mut_ptr());

    // enable mouse
    err = ISpDevices_ExtractByClass(
            kISpDeviceClass_Mouse,
            MAX_DEVICES as UInt32,
            addr_of_mut!(numDevices),
            devices.as_mut_ptr());
    Com_Printf(b"%i mouse devices\n\0".as_ptr() as *const c_char, numDevices);
    if numDevices > MAX_MOUSE_DEVICES as UInt32 {
        numDevices = MAX_MOUSE_DEVICES as UInt32;
    }

    err = ISpDevices_Activate(numDevices, devices.as_mut_ptr());
    i = 0;
    while i < numDevices as c_int {
        ISpDevice_GetElementList(devices[i as usize], addr_of_mut!(elementList));

        //	ISpGetGlobalElementList( &elementList );

        // go through all the elements and asign them Quake key codes
        ISpElementList_Extract(elementList, MAX_ELEMENTS as UInt32, addr_of_mut!(numElements[i as usize]), elements[i as usize].as_mut_ptr());
        Com_Printf(b"%i elements in list\n\0".as_ptr() as *const c_char, numElements[i as usize]);

        j = 0;
        while j < numElements[i as usize] as c_int {
            ISpElement_GetInfo(elements[i as usize][j as usize], addr_of_mut!(info));
            PStringToCString(info.theString);
            Com_Printf(b"%i : %s\n\0".as_ptr() as *const c_char, i, info.theString);
            j += 1;
        }
        i += 1;
    }

    inputActive = true;

    HideCursor();

    Com_Printf(b"------------------------------------\n\0".as_ptr() as *const c_char);
}

/*
=================
Sys_ShutdownInput
=================
*/
pub unsafe fn Sys_ShutdownInput() {
    if !inputActive {
        return;
    }
    ShowCursor();
    ISpShutdown();
    inputActive = false;
}

pub unsafe fn Sys_SuspendInput() {
    if inputSuspended {
        return;
    }
    inputSuspended = true;
    ShowCursor();
    ISpSuspend();
}

pub unsafe fn Sys_ResumeInput() {
    if !inputSuspended {
        return;
    }
    inputSuspended = false;
    HideCursor();
    ISpResume();
}

/*
=================
Sys_Input
=================
*/
pub unsafe fn Sys_Input() {
    let mut event: ISpElementEvent;
    let mut wasEvent: Boolean;
    let mut state: UInt32;
    let mut state2: UInt32;
    let mut xmove: c_int;
    let mut ymove: c_int;
    let mut button: c_int;
    static mut xtotal: c_int = 0;
    static mut ytotal: c_int = 0;
    let mut device: c_int;

    if !inputActive {
        return;
    }

    // during debugging it is sometimes usefull to be able to kill mouse support
    if (*in_nomouse).integer != 0 {
        Com_Printf(b"Shutting down input.\n\0".as_ptr() as *const c_char);
        Sys_ShutdownInput();
        return;
    }

    // always suspend for dedicated
    if (*com_dedicated).integer != 0 {
        Sys_SuspendInput();
        return;
    }

    // temporarily deactivate if not in the game and
    if cls.keyCatchers != 0 || cls.state != CA_ACTIVE {
        if !glConfig.isFullscreen {
            Sys_SuspendInput();
            return;
        }
    }

    Sys_ResumeInput();

    // send all button events
    device = 0;
    while device < numDevices as c_int {
        // mouse buttons

        button = 2;
        while button < numElements[device as usize] as c_int {
            loop {
                ISpElement_GetNextEvent(elements[device as usize][button as usize], core::mem::size_of::<ISpElementEvent>(), addr_of_mut!(event), addr_of_mut!(wasEvent));
                if !wasEvent {
                    break;
                }
                if event.data != 0 {
                    Sys_QueEvent(0, SE_KEY, K_MOUSE1 + button - 2, 1, 0, core::ptr::null_mut());
                } else {
                    Sys_QueEvent(0, SE_KEY, K_MOUSE1 + button - 2, 0, 0, core::ptr::null_mut());
                }
            }
            button += 1;
        }

        // mouse movement

        // send mouse event
        ISpElement_GetSimpleState(elements[device as usize][0], addr_of_mut!(state));
        xmove = (state as c_int) / MAC_MOUSE_SCALE;

        ISpElement_GetSimpleState(elements[device as usize][1], addr_of_mut!(state2));
        ymove = (state2 as c_int) / (-MAC_MOUSE_SCALE);

        if xmove != 0 || ymove != 0 {
            xtotal += xmove;
            ytotal += ymove;
            //Com_Printf(b"%i %i = %i %i\n\0".as_ptr() as *const c_char, state, state2, xtotal, ytotal);
            Sys_QueEvent(0, SE_MOUSE, xmove, ymove, 0, core::ptr::null_mut());
        }
        device += 1;
    }
}
