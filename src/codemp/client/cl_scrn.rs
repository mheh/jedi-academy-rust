#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// Stubs for external types referenced in this file
// These are defined in client.h and other headers
#[repr(C)]
pub struct console_t {
    pub xadjust: f32,
    pub yadjust: f32,
    // ... other fields omitted for structural parity
}

#[repr(C)]
pub struct cvar_t {
    // Stub: contains name, value, flags, etc.
    pub integer: c_int,
    pub value: f32,
    // Other fields omitted
}

pub type qboolean = c_int;
pub type qhandle_t = c_int;
pub type vec4_t = [f32; 4];
pub type vec3_t = [f32; 3];

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

const BIGCHAR_WIDTH: f32 = 16.0;
const SMALLCHAR_HEIGHT: i32 = 12;
const SMALLCHAR_WIDTH: i32 = 8;
const MAX_SCR_LINES: usize = 10;

// External types/externs from client.h and qcommon
extern "C" {
    pub static mut con: console_t;
    pub static mut cls: clientStatic_t;
    pub static mut clc: clientConnection_t;
    pub static mut uivm: *mut c_void;
    pub static mut re: refExport_t;
    pub static mut com_speeds: *mut cvar_t;
    pub static mut g_color_table: [[f32; 4]; 8];
    pub static mut cl_debugMove: *mut cvar_t;
    pub static mut time_frontend: c_int;
    pub static mut time_backend: c_int;

    pub fn Cvar_Get(varName: *const c_char, defaultValue: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Com_Memcpy(dest: *mut c_void, src: *const c_void, size: usize) -> *mut c_void;
    pub fn Com_DPrintf(msg: *const c_char, ...);
    pub fn Com_Printf(msg: *const c_char, ...);
    pub fn Com_Error(code: c_int, msg: *const c_char, ...);
    pub fn Q_IsColorString(p: *const c_char) -> qboolean;
    pub fn ColorIndex(c: c_char) -> usize;
    pub fn FS_FTell(f: *mut c_void) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    pub fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn VM_Call(vm: *mut c_void, callnum: c_int, ...) -> i32;
    pub fn S_StopAllSounds();
    pub fn SCR_DrawCinematic();
    pub fn CL_CGameRendering(stereoFrame: c_int);
    pub fn Con_DrawConsole();
    pub fn Con_ClearNotify();
}

// Stub types for structural coherence
#[repr(C)]
pub struct clientStatic_t {
    pub state: c_int,
    pub demorecording: qboolean,
    pub spDemoRecording: qboolean,
    pub demofile: *mut c_void,
    pub demoName: [c_char; 256],
    pub whiteShader: qhandle_t,
    pub charSetShader: qhandle_t,
    pub keyCatchers: c_int,
    pub framecount: c_int,
    pub realtime: c_int,
    pub glconfig: glConfig_t,
    // ... other fields omitted
}

#[repr(C)]
pub struct glConfig_t {
    pub vidWidth: c_int,
    pub vidHeight: c_int,
    pub stereoEnabled: qboolean,
    // ... other fields omitted
}

#[repr(C)]
pub struct clientConnection_t {
    pub demorecording: qboolean,
    pub spDemoRecording: qboolean,
    pub demofile: *mut c_void,
    pub demoName: [c_char; 256],
    // ... other fields omitted
}

#[repr(C)]
pub struct refExport_t {
    pub RegisterShader: Option<unsafe extern "C" fn(*const c_char) -> qhandle_t>,
    pub DrawStretchPic: Option<unsafe extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, qhandle_t)>,
    pub SetColor: Option<unsafe extern "C" fn(*const f32)>,
    pub BeginFrame: Option<unsafe extern "C" fn(c_int)>,
    pub EndFrame: Option<unsafe extern "C" fn(*mut c_int, *mut c_int)>,
    // Stub: contains other fields not used in this file
}

// Global variables

pub static mut scr_initialized: qboolean = qfalse; // ready to draw

pub static mut cl_timegraph: *mut cvar_t = core::ptr::null_mut();
pub static mut cl_debuggraph: *mut cvar_t = core::ptr::null_mut();
pub static mut cl_graphheight: *mut cvar_t = core::ptr::null_mut();
pub static mut cl_graphscale: *mut cvar_t = core::ptr::null_mut();
pub static mut cl_graphshift: *mut cvar_t = core::ptr::null_mut();

// ================
// SCR_DrawNamedPic
//
// Coordinates are 640*480 virtual values
// =================
pub unsafe fn SCR_DrawNamedPic(x: f32, y: f32, width: f32, height: f32, picname: *const c_char) {
    let hShader: qhandle_t;

    assert!(width != 0.0);

    hShader = re.RegisterShader.unwrap()(picname);
    re.DrawStretchPic.unwrap()(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}

// ================
// SCR_FillRect
//
// Coordinates are 640*480 virtual values
// =================
pub unsafe fn SCR_FillRect(x: f32, y: f32, width: f32, height: f32, color: *const f32) {
    re.SetColor.unwrap()(color);

    re.DrawStretchPic.unwrap()(x, y, width, height, 0.0, 0.0, 0.0, 0.0, cls.whiteShader);

    re.SetColor.unwrap()(core::ptr::null());
}

// ================
// SCR_DrawPic
//
// Coordinates are 640*480 virtual values
// =================
pub unsafe fn SCR_DrawPic(x: f32, y: f32, width: f32, height: f32, hShader: qhandle_t) {
    re.DrawStretchPic.unwrap()(x, y, width, height, 0.0, 0.0, 1.0, 1.0, hShader);
}

// ** SCR_DrawChar
// ** chars are drawn at 640*480 virtual screen size
static unsafe fn SCR_DrawChar(mut x: i32, mut y: i32, mut size: f32, ch: i32) {
    let mut row: i32 = 0;
    let mut col: i32 = 0;
    let mut frow: f32 = 0.0;
    let mut fcol: f32 = 0.0;
    let mut ax: f32 = 0.0;
    let mut ay: f32 = 0.0;
    let mut aw: f32 = 0.0;
    let mut ah: f32 = 0.0;

    let ch = ch & 255;

    if ch == ' ' as i32 {
        return;
    }

    if y < -(size as i32) {
        return;
    }

    ax = x as f32;
    ay = y as f32;
    aw = size;
    ah = size;

    row = ch >> 4;
    col = ch & 15;

    let mut size2: f32 = 0.0;

    frow = (row as f32) * 0.0625;
    fcol = (col as f32) * 0.0625;
    size = 0.03125;
    size2 = 0.0625;

    re.DrawStretchPic.unwrap()(ax, ay, aw, ah,
                       fcol, frow,
                       fcol + size, frow + size2,
                       cls.charSetShader);
}

// ** SCR_DrawSmallChar
// ** small chars are drawn at native screen resolution
pub unsafe fn SCR_DrawSmallChar(x: i32, y: i32, ch: i32) {
    let mut row: i32 = 0;
    let mut col: i32 = 0;
    let mut frow: f32 = 0.0;
    let mut fcol: f32 = 0.0;
    let mut size: f32 = 0.0;

    let ch = ch & 255;

    if ch == ' ' as i32 {
        return;
    }

    if y < -(SMALLCHAR_HEIGHT as i32) {
        return;
    }

    row = ch >> 4;
    col = ch & 15;

    let mut size2: f32 = 0.0;

    frow = (row as f32) * 0.0625;
    fcol = (col as f32) * 0.0625;

    #[cfg(feature = "jk2")]
    let size = 0.03125;
    #[cfg(not(feature = "jk2"))]
    let size = 0.0625;
    let size2 = 0.0625;

    re.DrawStretchPic.unwrap()((x as f32) * con.xadjust, (y as f32) * con.yadjust,
                        (SMALLCHAR_WIDTH as f32) * con.xadjust, (SMALLCHAR_HEIGHT as f32) * con.yadjust,
                       fcol, frow,
                       fcol + size, frow + size2,
                       cls.charSetShader);
}

// ==================
// SCR_DrawBigString[Color]
//
// Draws a multi-colored string with a drop shadow, optionally forcing
// to a fixed color.
//
// Coordinates are at 640 by 480 virtual resolution
// ==================
pub unsafe fn SCR_DrawStringExt(x: i32, y: i32, size: f32, string: *const c_char, setColor: *const f32, forceColor: qboolean) {
    let mut color: vec4_t = [0.0; 4];
    let mut s: *const c_char;
    let mut xx: i32;

    // draw the drop shadow
    color[0] = 0.0;
    color[1] = 0.0;
    color[2] = 0.0;
    color[3] = *setColor.add(3);
    re.SetColor.unwrap()(color.as_ptr());
    s = string;
    xx = x;
    while *s != 0 {
        if Q_IsColorString(s) != qfalse {
            s = s.add(2);
            continue;
        }
        SCR_DrawChar(xx + 2, y + 2, size, *s as i32);
        xx += size as i32;
        s = s.add(1);
    }

    // draw the colored text
    s = string;
    xx = x;
    re.SetColor.unwrap()(setColor);
    while *s != 0 {
        if Q_IsColorString(s) != qfalse {
            if forceColor == qfalse {
                let color_idx = ColorIndex(*s.add(1));
                Com_Memcpy(color.as_mut_ptr() as *mut c_void,
                           &g_color_table[color_idx] as *const _ as *const c_void,
                           core::mem::size_of::<vec4_t>());
                color[3] = *setColor.add(3);
                re.SetColor.unwrap()(color.as_ptr());
            }
            s = s.add(2);
            continue;
        }
        SCR_DrawChar(xx, y, size, *s as i32);
        xx += size as i32;
        s = s.add(1);
    }
    re.SetColor.unwrap()(core::ptr::null());
}

pub unsafe fn SCR_DrawBigString(x: i32, y: i32, s: *const c_char, alpha: f32) {
    let mut color: [f32; 4] = [0.0; 4];

    color[0] = 1.0;
    color[1] = 1.0;
    color[2] = 1.0;
    color[3] = alpha;
    SCR_DrawStringExt(x, y, BIGCHAR_WIDTH, s, color.as_ptr(), qfalse);
}

pub unsafe fn SCR_DrawBigStringColor(x: i32, y: i32, s: *const c_char, color: vec4_t) {
    SCR_DrawStringExt(x, y, BIGCHAR_WIDTH, s, color.as_ptr(), qtrue);
}

// ==================
// SCR_DrawSmallString[Color]
//
// Draws a multi-colored string with a drop shadow, optionally forcing
// to a fixed color.
//
// Coordinates are at 640 by 480 virtual resolution
// ==================
pub unsafe fn SCR_DrawSmallStringExt(x: i32, y: i32, string: *const c_char, setColor: *const f32, forceColor: qboolean) {
    let mut color: vec4_t = [0.0; 4];
    let mut s: *const c_char;
    let mut xx: i32;

    // draw the colored text
    s = string;
    xx = x;
    re.SetColor.unwrap()(setColor);
    while *s != 0 {
        if Q_IsColorString(s) != qfalse {
            if forceColor == qfalse {
                let color_idx = ColorIndex(*s.add(1));
                Com_Memcpy(color.as_mut_ptr() as *mut c_void,
                           &g_color_table[color_idx] as *const _ as *const c_void,
                           core::mem::size_of::<vec4_t>());
                color[3] = *setColor.add(3);
                re.SetColor.unwrap()(color.as_ptr());
            }
            s = s.add(2);
            continue;
        }
        SCR_DrawSmallChar(xx, y, *s as i32);
        xx += SMALLCHAR_WIDTH;
        s = s.add(1);
    }
    re.SetColor.unwrap()(core::ptr::null());
}

// ** SCR_Strlen -- skips color escape codes
static unsafe fn SCR_Strlen(str: *const c_char) -> i32 {
    let mut s: *const c_char = str;
    let mut count: i32 = 0;

    while *s != 0 {
        if Q_IsColorString(s) != qfalse {
            s = s.add(2);
        } else {
            count += 1;
            s = s.add(1);
        }
    }

    return count;
}

// ** SCR_GetBigStringWidth
pub unsafe fn SCR_GetBigStringWidth(str: *const c_char) -> i32 {
    return SCR_Strlen(str) * 16;
}

// ===============================================================================

// =================
// SCR_DrawDemoRecording
// =================
pub unsafe fn SCR_DrawDemoRecording() {
    #[cfg(not(feature = "xbox"))]
    {
        let mut string: [c_char; 1024] = [0; 1024];
        let pos: c_int;

        if clc.demorecording == qfalse {
            return;
        }
        if clc.spDemoRecording != qfalse {
            return;
        }

        pos = FS_FTell(clc.demofile);
        sprintf(string.as_mut_ptr(),
                "RECORDING %s: %ik\0".as_ptr() as *const c_char,
                clc.demoName.as_ptr(),
                pos / 1024);

        SCR_DrawStringExt(320 - (strlen(string.as_ptr()) as i32) * 4, 20, 8.0, string.as_ptr(),
                         g_color_table[7].as_ptr(), qtrue);
    }
}

// ===============================================================================
//
// DEBUG GRAPH
//
// ===============================================================================

#[repr(C)]
struct graphsamp_t {
    value: f32,
    color: i32,
}

static mut current: i32 = 0;
static mut values: [graphsamp_t; 1024] = [graphsamp_t { value: 0.0, color: 0 }; 1024];

// ==============
// SCR_DebugGraph
// ==============
pub unsafe fn SCR_DebugGraph(value: f32, color: i32) {
    values[(current & 1023) as usize].value = value;
    values[(current & 1023) as usize].color = color;
    current += 1;
}

// ==============
// SCR_DrawDebugGraph
// ==============
pub unsafe fn SCR_DrawDebugGraph() {
    let mut a: i32 = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut w: i32 = 0;
    let mut i: i32 = 0;
    let mut h: i32 = 0;
    let mut v: f32 = 0.0;
    let mut color: i32 = 0;

    // draw the graph
    w = 640;
    x = 0;
    y = 480;
    re.SetColor.unwrap()(g_color_table[0].as_ptr());
    re.DrawStretchPic.unwrap()((x as f32), (y as f32) - (*cl_graphheight).integer as f32,
        (w as f32), (*cl_graphheight).integer as f32, 0.0, 0.0, 0.0, 0.0, cls.whiteShader);
    re.SetColor.unwrap()(core::ptr::null());

    a = 0;
    while a < w {
        i = (current - 1 - a + 1024) & 1023;
        v = values[i as usize].value;
        color = values[i as usize].color;
        v = v * (*cl_graphscale).integer as f32 + (*cl_graphshift).integer as f32;

        if v < 0.0 {
            v += (*cl_graphheight).integer as f32 * (1.0 + (-v / (*cl_graphheight).integer as f32).floor());
        }
        h = (v as i32) % (*cl_graphheight).integer;
        re.DrawStretchPic.unwrap()((x + w - 1 - a) as f32, (y - h) as f32, 1.0, (h as f32), 0.0, 0.0, 0.0, 0.0, cls.whiteShader);
        a += 1;
    }
}

// =============================================================================

// ==================
// SCR_Init
// ==================
pub unsafe fn SCR_Init() {
    cl_timegraph = Cvar_Get(b"timegraph\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0x0020); // CVAR_CHEAT
    cl_debuggraph = Cvar_Get(b"debuggraph\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0x0020); // CVAR_CHEAT
    cl_graphheight = Cvar_Get(b"graphheight\0".as_ptr() as *const c_char, b"32\0".as_ptr() as *const c_char, 0x0020); // CVAR_CHEAT
    cl_graphscale = Cvar_Get(b"graphscale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0x0020); // CVAR_CHEAT
    cl_graphshift = Cvar_Get(b"graphshift\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0x0020); // CVAR_CHEAT

    scr_initialized = qtrue;
}

// =======================================================

// ==================
// SCR_DrawScreenField
//
// This will be called twice if rendering in stereo mode
// ==================
const CA_DISCONNECTED: c_int = 0;
const CA_CINEMATIC: c_int = 1;
const CA_CONNECTING: c_int = 2;
const CA_CHALLENGING: c_int = 3;
const CA_CONNECTED: c_int = 4;
const CA_LOADING: c_int = 5;
const CA_PRIMED: c_int = 6;
const CA_ACTIVE: c_int = 7;

const STEREO_LEFT: c_int = 0;
const STEREO_CENTER: c_int = 1;
const STEREO_RIGHT: c_int = 2;

const KEYCATCH_UI: c_int = 0x01;

const UIMENU_MAIN: c_int = 0;
const ERR_FATAL: c_int = 1;

const UI_IS_FULLSCREEN: c_int = 1;
const UI_SET_ACTIVE_MENU: c_int = 2;
const UI_REFRESH: c_int = 3;
const UI_DRAW_CONNECT_SCREEN: c_int = 4;

pub unsafe fn SCR_DrawScreenField(stereoFrame: c_int) {
    re.BeginFrame.unwrap()(stereoFrame);

    // wide aspect ratio screens need to have the sides cleared
    // unless they are displaying game renderings
    if cls.state != CA_ACTIVE {
        if cls.glconfig.vidWidth * 480 > cls.glconfig.vidHeight * 640 {
            re.SetColor.unwrap()(g_color_table[0].as_ptr());
            re.DrawStretchPic.unwrap()(0.0, 0.0, cls.glconfig.vidWidth as f32, cls.glconfig.vidHeight as f32, 0.0, 0.0, 0.0, 0.0, cls.whiteShader);
            re.SetColor.unwrap()(core::ptr::null());
        }
    }

    if uivm.is_null() {
        Com_DPrintf(b"draw screen without UI loaded\n\0".as_ptr() as *const c_char);
        return;
    }

    // if the menu is going to cover the entire screen, we
    // don't need to render anything under it
    // actually, yes you do, unless you want clients to cycle out their reliable
    // commands from sitting in the menu. -rww
    if VM_Call(uivm, UI_IS_FULLSCREEN) == 0 || ((cls.framecount & 7) != 0 && cls.state == CA_ACTIVE) {
        match cls.state {
            CA_CINEMATIC => {
                SCR_DrawCinematic();
            }
            CA_DISCONNECTED => {
                // force menu up
                S_StopAllSounds();
                VM_Call(uivm, UI_SET_ACTIVE_MENU, UIMENU_MAIN);
            }
            CA_CONNECTING | CA_CHALLENGING | CA_CONNECTED => {
                // connecting clients will only show the connection dialog
                // refresh to update the time
                VM_Call(uivm, UI_REFRESH, cls.realtime);
                VM_Call(uivm, UI_DRAW_CONNECT_SCREEN, qfalse);
            }
            CA_LOADING | CA_PRIMED => {
                // draw the game information screen and loading progress
                CL_CGameRendering(stereoFrame);

                // also draw the connection information, so it doesn't
                // flash away too briefly on local or lan games
                // refresh to update the time
                VM_Call(uivm, UI_REFRESH, cls.realtime);
                VM_Call(uivm, UI_DRAW_CONNECT_SCREEN, qtrue);
            }
            CA_ACTIVE => {
                CL_CGameRendering(stereoFrame);
                SCR_DrawDemoRecording();
            }
            _ => {
                Com_Error(ERR_FATAL, b"SCR_DrawScreenField: bad cls.state\0".as_ptr() as *const c_char);
            }
        }
    }

    // the menu draws next
    if (cls.keyCatchers & KEYCATCH_UI) != 0 && !uivm.is_null() {
        VM_Call(uivm, UI_REFRESH, cls.realtime);
    }

    // console draws next
    Con_DrawConsole();

    // debug graph can be drawn on top of anything
    if (*cl_debuggraph).integer != 0 || (*cl_timegraph).integer != 0 || (*cl_debugMove).integer != 0 {
        SCR_DrawDebugGraph();
    }
}

// ==================
// SCR_UpdateScreen
//
// This is called every frame, and can also be called explicitly to flush
// text to the screen.
// ==================
pub unsafe fn SCR_UpdateScreen() {
    static mut recursive: i32 = 0;

    if scr_initialized == qfalse {
        return;				// not initialized yet
    }

    recursive += 1;
    if recursive > 2 {
        Com_Error(ERR_FATAL, b"SCR_UpdateScreen: recursively called\0".as_ptr() as *const c_char);
    }
    recursive = 1;

    // if running in stereo, we need to draw the frame twice
    if cls.glconfig.stereoEnabled != qfalse {
        SCR_DrawScreenField(STEREO_LEFT);
        SCR_DrawScreenField(STEREO_RIGHT);
    } else {
        SCR_DrawScreenField(STEREO_CENTER);
    }

    if (*com_speeds).integer != 0 {
        re.EndFrame.unwrap()(&mut time_frontend, &mut time_backend);
    } else {
        re.EndFrame.unwrap()(core::ptr::null_mut(), core::ptr::null_mut());
    }

    recursive = 0;
}

static mut scr_centertime_off: f32 = 0.0;
pub static mut scr_center_y: i32 = 0;
// static scr_font: String;
static mut scr_centerstring: [c_char; 1024] = [0; 1024];
static mut scr_center_lines: i32 = 0;
static mut scr_center_widths: [i32; MAX_SCR_LINES] = [0; MAX_SCR_LINES];

pub static mut scr_centertime: *mut cvar_t = core::ptr::null_mut();

pub unsafe fn SCR_CenterPrint(str: *mut c_char) {
    let mut s: *mut c_char;
    let mut last: *mut c_char;
    let mut start: *mut c_char;
    let mut write_pos: *mut c_char;
    let mut save_pos: *mut c_char;
    let mut num_chars: i32;
    let mut num_lines: i32;
    let width: i32;
    let mut done: bool = false;
    let mut spaced: bool;

    if str.is_null() {
        scr_centertime_off = 0.0;
        return;
    }

    // scr_font = String("medium");

    // RWL - commented out
    // width = viddef.width / 8;	// rjr hardcoded yuckiness
    let width = 640 / 8;	// rjr hardcoded yuckiness
    let width = width - 4;

    // RWL - commented out
    /*
    if cl.frame.playerstate.remote_type != REMOTE_TYPE_LETTERBOX
    {
        width -= 30;
    }
    */

    scr_centertime_off = (*scr_centertime).value;

    Com_Printf(b"\n\0".as_ptr() as *const c_char);

    num_lines = 0;
    write_pos = scr_centerstring.as_mut_ptr();
    scr_center_lines = 0;
    spaced = false;
    s = str;
    start = str;
    last = core::ptr::null_mut();
    num_chars = 0;
    loop {
        num_chars += 1;
        if *s == ' ' as u8 as c_char {
            spaced = true;
            last = s;
            scr_centertime_off += 0.2; //give them an extra 0.05 second for each character
        }

        if *s == '\n' as u8 as c_char || *s == 0 {
            last = s;
            num_chars = width;
            spaced = true;
        }

        if num_chars >= width {
            scr_centertime_off += 0.8; //give them an extra half second for each newline
            if last.is_null() {
                last = s;
            }
            if !spaced {
                last = last.add(1);
            }

            save_pos = write_pos;
            strncpy(write_pos, start, (last as usize - start as usize));
            write_pos = write_pos.add(last as usize - start as usize);
            *write_pos = 0;
            write_pos = write_pos.add(1);

            Com_Printf(b"%s\n\0".as_ptr() as *const c_char, save_pos);

            // RWL - commented out
            // scr_center_widths[scr_center_lines] = re.StrlenFont(save_pos, scr_font);;
            scr_center_widths[scr_center_lines as usize] = 640;

            scr_center_lines += 1;

            if *s == 0 as c_char || scr_center_lines >= MAX_SCR_LINES as i32 {
                done = true;
            } else {
                s = last;
                if spaced {
                    last = last.add(1);
                }
                start = last;
                last = core::ptr::null_mut();
                num_chars = 0;
                spaced = false;
            }
            s = s.add(1);
            if done {
                break;
            }
            continue;
        }
        s = s.add(1);
    }

    // echo it to the console
    Com_Printf(b"\n\n\0".as_ptr() as *const c_char);
    Con_ClearNotify();
}
