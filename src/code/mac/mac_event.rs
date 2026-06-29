#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Mac OS SDK types - stubs for compatibility with classic Mac APIs
#[repr(C)]
pub struct EventRecord {
    pub what: c_int,
    pub message: c_int,
    pub when: c_int,
    pub where_: Point,
    pub modifiers: c_int,
}

#[repr(C)]
pub struct Point {
    pub v: c_int,
    pub h: c_int,
}

#[repr(C)]
pub struct Rect {
    pub top: c_int,
    pub left: c_int,
    pub bottom: c_int,
    pub right: c_int,
}

pub type WindowPtr = *mut c_void;
pub type GrafPtr = *mut c_void;
pub type DialogPtr = *mut c_void;
pub type Str255 = [c_char; 256];

#[repr(C)]
pub struct Region {
    pub rgnBBox: Rect,
    // Other fields are opaque
}

// Extern C interface to Mac OS SDK functions and game engine functions
extern "C" {
    fn FindWindow(where_: Point, myWindow: *mut WindowPtr) -> c_int;
    fn DrawMenuBar();
    fn MenuSelect(where_: Point) -> c_int;
    fn SystemClick(event: *const EventRecord, myWindow: WindowPtr);
    fn DoDrag(myWindow: WindowPtr, mouseloc: Point);
    fn LocalToGlobal(point: *mut Point);
    fn Cvar_SetValue(name: *const c_char, value: c_int);
    fn DoGoAwayBox(myWindow: WindowPtr, mouseloc: Point);
    fn FrontWindow() -> WindowPtr;
    fn SelectWindow(myWindow: WindowPtr);
    fn GetGrayRgn() -> *mut *mut Region;
    fn DragWindow(myWindow: WindowPtr, mouseloc: Point, bounds: *const Rect);
    fn aglUpdateContext(ctx: *mut c_void);
    fn aglGetCurrentContext() -> *mut c_void;
    fn TrackGoAway(myWindow: WindowPtr, mouseloc: Point) -> c_int;
    fn DoCloseWindow(myWindow: WindowPtr);
    fn GetPort(port: *mut GrafPtr);
    fn SetPort(port: GrafPtr);
    fn BeginUpdate(myWindow: WindowPtr);
    fn EndUpdate(myWindow: WindowPtr);
    fn GetNewDialog(id: c_int, storage: *mut c_void, behind: WindowPtr) -> DialogPtr;
    fn ModalDialog(filterProc: *mut c_void, itemHit: *mut c_int);
    fn DisposeDialog(dialog: DialogPtr);
    fn HiWord(val: c_int) -> c_int;
    fn LoWord(val: c_int) -> c_int;
    fn GetMenuHandle(menuID: c_int) -> *mut c_void;
    fn GetMenuItemText(menu: *mut c_void, item: c_int, name: *mut c_char);
    fn OpenDeskAcc(name: *mut c_char) -> c_int;
    fn HiliteMenu(menuID: c_int);
    fn Sys_QueEvent(time: c_int, type_: c_int, value: c_int, extra: c_int, more: c_int, data: *mut c_void);
    fn Sys_Milliseconds() -> c_int;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Quit_f();
    fn Sys_ConsoleEvent(event: *const EventRecord) -> c_int;
    fn WaitNextEvent(mask: c_int, event: *mut EventRecord, sleep: c_int, mouseRgn: *mut c_void) -> c_int;
    fn GetOSEvent(mask: c_int, event: *mut EventRecord) -> c_int;

    pub static mut sys_lastEventTic: c_int;
    pub static mut sys_ticBase: c_int;
    pub static mut sys_msecBase: c_int;
    pub static mut glConfig: GLConfig;
    pub static mut sys_waitNextEvent: *mut cvar_t;
    pub static mut inputActive: c_int;
}

// Stub types for external dependencies
#[repr(C)]
pub struct GLConfig {
    pub isFullscreen: c_int,
    // Other fields are opaque
}

#[repr(C)]
pub struct cvar_t {
    pub value: c_int,
    // Other fields are opaque
}

// Mac event type constants
const inMenuBar: c_int = 1;
const inSysWindow: c_int = 2;
const inDrag: c_int = 3;
const inGoAway: c_int = 4;
const inContent: c_int = 5;

const mouseDown: c_int = 1;
const mouseUp: c_int = 2;
const keyDown: c_int = 3;
const keyUp: c_int = 4;
const autoKey: c_int = 5;
const updateEvt: c_int = 6;
const diskEvt: c_int = 7;
const activateEvt: c_int = 8;
const osEvt: c_int = 15;

const everyEvent: c_int = 0xFFFF;

const charCodeMask: c_int = 0xFF;
const keyCodeMask: c_int = 0xFF00;

const SE_KEY: c_int = 1;
const SE_CHAR: c_int = 2;

const K_ENTER: c_char = 13 as c_char;
const K_TAB: c_char = 9 as c_char;
const K_SPACE: c_char = 32 as c_char;
const K_BACKSPACE: c_char = 8 as c_char;
const K_ESCAPE: c_char = 27 as c_char;
const K_COMMAND: c_char = -1i8 as c_char;
const K_SHIFT: c_char = -2i8 as c_char;
const K_CAPSLOCK: c_char = -3i8 as c_char;
const K_ALT: c_char = -4i8 as c_char;
const K_CTRL: c_char = -5i8 as c_char;
const K_KP_DEL: c_char = -6i8 as c_char;
const K_KP_STAR: c_char = -7i8 as c_char;
const K_KP_PLUS: c_char = -8i8 as c_char;
const K_KP_NUMLOCK: c_char = -9i8 as c_char;
const K_KP_SLASH: c_char = -10i8 as c_char;
const K_KP_ENTER: c_char = -11i8 as c_char;
const K_KP_MINUS: c_char = -12i8 as c_char;
const K_KP_EQUALS: c_char = -13i8 as c_char;
const K_KP_INS: c_char = -14i8 as c_char;
const K_KP_END: c_char = -15i8 as c_char;
const K_KP_DOWNARROW: c_char = -16i8 as c_char;
const K_KP_PGDN: c_char = -17i8 as c_char;
const K_KP_LEFTARROW: c_char = -18i8 as c_char;
const K_KP_5: c_char = -19i8 as c_char;
const K_KP_RIGHTARROW: c_char = -20i8 as c_char;
const K_KP_HOME: c_char = -21i8 as c_char;
const K_KP_UPARROW: c_char = -22i8 as c_char;
const K_KP_PGUP: c_char = -23i8 as c_char;
const K_F1: c_char = -24i8 as c_char;
const K_F2: c_char = -25i8 as c_char;
const K_F3: c_char = -26i8 as c_char;
const K_F4: c_char = -27i8 as c_char;
const K_F5: c_char = -28i8 as c_char;
const K_F6: c_char = -29i8 as c_char;
const K_F7: c_char = -30i8 as c_char;
const K_F8: c_char = -31i8 as c_char;
const K_F9: c_char = -32i8 as c_char;
const K_F10: c_char = -33i8 as c_char;
const K_F11: c_char = -34i8 as c_char;
const K_F12: c_char = -35i8 as c_char;
const K_F13: c_char = -36i8 as c_char;
const K_F14: c_char = -37i8 as c_char;
const K_F15: c_char = -38i8 as c_char;
const K_INS: c_char = -39i8 as c_char;
const K_DEL: c_char = -40i8 as c_char;
const K_HOME: c_char = -41i8 as c_char;
const K_END: c_char = -42i8 as c_char;
const K_PGUP: c_char = -43i8 as c_char;
const K_PGDN: c_char = -44i8 as c_char;
const K_LEFTARROW: c_char = -45i8 as c_char;
const K_RIGHTARROW: c_char = -46i8 as c_char;
const K_DOWNARROW: c_char = -47i8 as c_char;
const K_UPARROW: c_char = -48i8 as c_char;
const K_POWER: c_char = -49i8 as c_char;
const K_MOUSE1: c_char = -50i8 as c_char;

const mApple: c_int = 1;
const iAbout: c_int = 1;
const mFile: c_int = 2;
const iQuit: c_int = 1;
const kAboutDialog: c_int = 128;

/*
================
Sys_MsecForMacEvent

Q3 event records take time in msec,
so convert the mac event record when
(60ths) to msec.  The base values
are updated ever frame, so this
is guaranteed to not drift.
=================
*/
pub unsafe fn Sys_MsecForMacEvent() -> c_int {
    let tics: c_int;

    tics = *core::ptr::addr_of!(sys_lastEventTic) - *core::ptr::addr_of!(sys_ticBase);

    return *core::ptr::addr_of!(sys_msecBase) + tics * 16;
}




pub unsafe fn DoMouseDown(event: *const EventRecord)
{
    let mut myPart: c_int = 0;
    let mut myWindow: WindowPtr = core::ptr::null_mut();
    let mut point: Point = Point { v: 0, h: 0 };

    myPart = FindWindow((*event).where_, &mut myWindow);

    match myPart
    {
        inMenuBar => {
            DrawMenuBar();
            DoMenuCommand(MenuSelect((*event).where_));
        },
        inSysWindow => {
            SystemClick(event, myWindow);
        },
        inDrag => {
            DoDrag(myWindow, (*event).where_);

            // update the vid_xpos / vid_ypos cvars
            point.h = 0;
            point.v = 0;
            LocalToGlobal(&mut point);
            Cvar_SetValue(b"vid_xpos\0".as_ptr() as *const c_char, point.h);
            Cvar_SetValue(b"vid_ypos\0".as_ptr() as *const c_char, point.v);
            return;
        },
        inGoAway => {
            DoGoAwayBox(myWindow, (*event).where_);
        },

        inContent => {
            if myWindow != FrontWindow()
            {
                SelectWindow(myWindow);
            }
        },
        _ => {}
    }
}

pub unsafe fn DoMouseUp(event: *const EventRecord)
{
}

pub unsafe fn DoDrag(myWindow: WindowPtr, mouseloc: Point)
{
    let dragBounds: Rect;

    dragBounds = (**GetGrayRgn()).rgnBBox;
    DragWindow(myWindow, mouseloc, &dragBounds);

    aglUpdateContext(aglGetCurrentContext());
}


pub unsafe fn DoGoAwayBox(myWindow: WindowPtr, mouseloc: Point)
{
    if TrackGoAway(myWindow, mouseloc) != 0
    {
        DoCloseWindow(myWindow);
    }
}

pub unsafe fn DoCloseWindow(myWindow: WindowPtr)
{
}

pub unsafe fn DoMenuAdjust()
{
}

pub static vkeyToQuakeKey: [c_char; 256] = [
/*0x00*/    'a' as c_char, 's' as c_char, 'd' as c_char, 'f' as c_char, 'h' as c_char, 'g' as c_char, 'z' as c_char, 'x' as c_char,
/*0x08*/    'c' as c_char, 'v' as c_char, '?' as c_char, 'b' as c_char, 'q' as c_char, 'w' as c_char, 'e' as c_char, 'r' as c_char,
/*0x10*/    'y' as c_char, 't' as c_char, '1' as c_char, '2' as c_char, '3' as c_char, '4' as c_char, '6' as c_char, '5' as c_char,
/*0x18*/    '=' as c_char, '9' as c_char, '7' as c_char, '-' as c_char, '8' as c_char, '0' as c_char, ']' as c_char, 'o' as c_char,
/*0x20*/    'u' as c_char, '[' as c_char, 'i' as c_char, 'p' as c_char, K_ENTER, 'l' as c_char, 'j' as c_char, '\'' as c_char,
/*0x28*/    'k' as c_char, ';' as c_char, '\\' as c_char, ',' as c_char, '/' as c_char, 'n' as c_char, 'm' as c_char, '.' as c_char,
/*0x30*/    K_TAB, K_SPACE, '`' as c_char, K_BACKSPACE, '?' as c_char, K_ESCAPE, '?' as c_char, K_COMMAND,
/*0x38*/    K_SHIFT, K_CAPSLOCK, K_ALT, K_CTRL, '?' as c_char, '?' as c_char, '?' as c_char, '?' as c_char,
/*0x40*/    '?' as c_char, K_KP_DEL, '?' as c_char, K_KP_STAR, '?' as c_char, K_KP_PLUS, '?' as c_char, K_KP_NUMLOCK,
/*0x48*/    '?' as c_char, '?' as c_char, '?' as c_char, K_KP_SLASH, K_KP_ENTER, '?' as c_char, K_KP_MINUS, '?' as c_char,
/*0x50*/    '?' as c_char, K_KP_EQUALS, K_KP_INS, K_KP_END, K_KP_DOWNARROW, K_KP_PGDN, K_KP_LEFTARROW, K_KP_5,
/*0x58*/    K_KP_RIGHTARROW, K_KP_HOME, '?' as c_char, K_KP_UPARROW, K_KP_PGUP, '?' as c_char, '?' as c_char, '?' as c_char,
/*0x60*/    K_F5, K_F6, K_F7, K_F3, K_F8, K_F9, '?' as c_char, K_F11,
/*0x68*/    '?' as c_char, K_F13, '?' as c_char, K_F14, '?' as c_char, K_F10, '?' as c_char, K_F12,
/*0x70*/    '?' as c_char, K_F15, K_INS, K_HOME, K_PGUP, K_DEL, K_F4, K_END,
/*0x78*/    K_F2, K_PGDN, K_F1, K_LEFTARROW, K_RIGHTARROW, K_DOWNARROW, K_UPARROW, K_POWER
];

pub unsafe fn DoKeyDown(event: *const EventRecord)
{
    let mut myCharCode: c_int = 0;
    let mut myKeyCode: c_int = 0;

    myCharCode = (*event).message & charCodeMask;
    myKeyCode = ((*event).message & keyCodeMask) >> 8;

    Sys_QueEvent(Sys_MsecForMacEvent(), SE_KEY, vkeyToQuakeKey[myKeyCode as usize] as c_int, 1, 0, core::ptr::null_mut());
    Sys_QueEvent(Sys_MsecForMacEvent(), SE_CHAR, myCharCode, 0, 0, core::ptr::null_mut());
}

pub unsafe fn DoKeyUp(event: *const EventRecord)
{
    let mut myCharCode: c_int = 0;
    let mut myKeyCode: c_int = 0;

    myCharCode = (*event).message & charCodeMask;
    myKeyCode = ((*event).message & keyCodeMask) >> 8;

    Sys_QueEvent(Sys_MsecForMacEvent(), SE_KEY, vkeyToQuakeKey[myKeyCode as usize] as c_int, 0, 0, core::ptr::null_mut());
}

/*
==================
Sys_ModifierEvents
==================
*/
unsafe fn Sys_ModifierEvents(modifiers: c_int) {
    static mut oldModifiers: c_int = 0;

    #[repr(C)]
    struct modifierKey_t {
        bit: c_int,
        keyCode: c_char,
    }

    static keys: [modifierKey_t; 7] = [
        modifierKey_t { bit: 128, keyCode: K_MOUSE1 },
        modifierKey_t { bit: 256, keyCode: K_COMMAND },
        modifierKey_t { bit: 512, keyCode: K_SHIFT },
        modifierKey_t { bit: 1024, keyCode: K_CAPSLOCK },
        modifierKey_t { bit: 2048, keyCode: K_ALT },
        modifierKey_t { bit: 4096, keyCode: K_CTRL },
        modifierKey_t { bit: -1, keyCode: -1i8 as c_char }
    ];

    let changed: c_int = modifiers ^ *core::ptr::addr_of!(oldModifiers);

    let mut i: c_int = 0;
    while keys[i as usize].bit != -1 {
        // if we have input sprockets running, ignore mouse events we
        // get from the debug passthrough driver
        if *core::ptr::addr_of!(inputActive) != 0 && keys[i as usize].keyCode == K_MOUSE1 {
            i += 1;
            continue;
        }

        if (changed & keys[i as usize].bit) != 0 {
            let flag = if (modifiers & keys[i as usize].bit) != 0 { 1 } else { 0 };
            Sys_QueEvent(Sys_MsecForMacEvent(),
            SE_KEY, keys[i as usize].keyCode as c_int, flag, 0, core::ptr::null_mut());
        }
        i += 1;
    }

    *core::ptr::addr_of_mut!(oldModifiers) = modifiers;
}


unsafe fn DoDiskEvent(event: *const EventRecord)
{

}

unsafe fn DoOSEvent(event: *const EventRecord)
{

}

unsafe fn DoUpdate(myWindow: WindowPtr)
{
    let mut origPort: GrafPtr = core::ptr::null_mut();

    GetPort(&mut origPort);
    SetPort(myWindow);

    BeginUpdate(myWindow);
    EndUpdate(myWindow);

    aglUpdateContext(aglGetCurrentContext());

    SetPort(origPort);
}

unsafe fn DoActivate(myWindow: WindowPtr, myModifiers: c_int) {

}

unsafe fn DoAboutBox() {
    let mut myDialog: DialogPtr = core::ptr::null_mut();
    let mut itemHit: c_int = 0;

    myDialog = GetNewDialog(kAboutDialog, core::ptr::null_mut(), core::ptr::null_mut() as WindowPtr);
    ModalDialog(core::ptr::null_mut(), &mut itemHit);
    DisposeDialog(myDialog);
}

unsafe fn DoMenuCommand(menuAndItem: c_int) {
    let mut myMenuNum: c_int = 0;
    let mut myItemNum: c_int = 0;
    let mut myResult: c_int = 0;
    let mut myDAName: Str255 = [0; 256];
    let mut myWindow: GrafPtr = core::ptr::null_mut();

    myMenuNum = HiWord(menuAndItem);
    myItemNum = LoWord(menuAndItem);

    GetPort(&mut myWindow);

    match myMenuNum {
    mApple => {
        match myItemNum {
        iAbout => {
            DoAboutBox();
        },
        _ => {
            GetMenuItemText(GetMenuHandle(mApple), myItemNum, myDAName.as_mut_ptr());
            myResult = OpenDeskAcc(myDAName.as_mut_ptr());
        }
        }
    },
    mFile => {
        match myItemNum {
        iQuit => {
            Com_Quit_f();
        },
        _ => {}
        }
    },
    _ => {}
    }

    HiliteMenu(0);
}

pub unsafe fn TestTime(ev: *const EventRecord) {
    let mut msec: c_int;
    let mut tics: c_int;
    static mut startTics: c_int = 0;
    static mut startMsec: c_int = 0;

    msec = Sys_Milliseconds();
    tics = (*ev).when;

    if *core::ptr::addr_of!(startTics) == 0 || (*ev).what == mouseDown {
        *core::ptr::addr_of_mut!(startTics) = tics;
        *core::ptr::addr_of_mut!(startMsec) = msec;
    }

    msec -= *core::ptr::addr_of!(startMsec);
    tics -= *core::ptr::addr_of!(startTics);

    if tics == 0 {
        return;
    }
    Com_Printf(b"%i msec to tic\n\0".as_ptr() as *const c_char, msec / tics);
}

/*
==================
Sys_SendKeyEvents
==================
*/
pub unsafe fn Sys_SendKeyEvents() {
    let mut gotEvent: c_int = 0;
    let mut event: EventRecord = EventRecord {
        what: 0,
        message: 0,
        when: 0,
        where_: Point { v: 0, h: 0 },
        modifiers: 0,
    };

    if (*core::ptr::addr_of!(glConfig)).isFullscreen == 0 || (*(*core::ptr::addr_of!(sys_waitNextEvent))).value != 0 {
        // this call involves 68k code and task switching.
        // do it on the desktop, or if they explicitly ask for
        // it when fullscreen
        gotEvent = WaitNextEvent(everyEvent, &mut event, 0, core::ptr::null_mut());
    } else {
        gotEvent = GetOSEvent(everyEvent, &mut event);
    }

    // generate faked events from modifer changes
    Sys_ModifierEvents(event.modifiers);

    *core::ptr::addr_of_mut!(sys_lastEventTic) = event.when;

    if gotEvent == 0 {
        return;
    }
    if Sys_ConsoleEvent(&event) != 0 {
        return;
    }
    match event.what
    {
        mouseDown => {
            DoMouseDown(&event);
        },
        mouseUp => {
            DoMouseUp(&event);
        },
        keyDown => {
            DoKeyDown(&event);
        },
        keyUp => {
            DoKeyUp(&event);
        },
        autoKey => {
            DoKeyDown(&event);
        },
        updateEvt => {
            DoUpdate(event.message as WindowPtr);
        },
        diskEvt => {
            DoDiskEvent(&event);
        },
        activateEvt => {
            DoActivate(event.message as WindowPtr, event.modifiers);
        },
        osEvt => {
            DoOSEvent(&event);
        },
        _ => {}
    }
}
