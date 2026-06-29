// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_char, c_void};

// Extern function declarations
extern "C" {
    fn Sys_GetClipboardData() -> *mut c_char;
    fn Z_Free(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);

    fn SCR_DrawSmallChar(x: c_int, y: c_int, ch: c_int);
    fn SCR_DrawBigString(x: c_int, y: c_int, s: *const c_char, alpha: f32);
    fn SCR_UpdateScreen();
    fn SCR_StopCinematic(stop: bool);

    fn Cbuf_AddText(text: *const c_char);

    fn Cmd_AddCommand(cmd_name: *const c_char, function: *const c_void);
    fn Cmd_CompleteCommand(partial: *const c_char) -> *const c_char;
    fn Cmd_CompleteCommandNext(partial: *const c_char, last: *const c_char) -> *const c_char;
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(arg: c_int) -> *const c_char;

    fn Cvar_CompleteVariable(partial: *const c_char) -> *const c_char;
    fn Cvar_CompleteVariableNext(partial: *const c_char, last: *const c_char) -> *const c_char;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);

    fn CL_AddReliableCommand(cmd: *const c_char);
    fn CL_Disconnect_f();
    fn CL_IsRunningInGameCinematic() -> bool;

    fn UI_SetActiveMenu(menu: *const c_char, arg: *const c_void);
    fn _UI_KeyEvent(key: c_int, down: bool);

    fn Con_ToggleConsole_f();
    fn Con_PageUp();
    fn Con_PageDown();
    fn Con_Top();
    fn Con_Bottom();

    fn FS_Printf(f: usize, fmt: *const c_char, ...);

    fn CopyString(str: *const c_char) -> *mut c_char;

    static mut cls: clientActive_t;
    static mut cvar_modifiedFlags: c_int;
}

// Extern type stubs
#[repr(C)]
pub struct field_t {
    pub buffer: [c_char; 256],
    pub cursor: c_int,
    pub scroll: c_int,
    pub widthInChars: c_int,
}

#[repr(C)]
pub struct keyname_t {
    pub upper: u16,
    pub lower: u16,
    pub name: *const c_char,
    pub keynum: c_int,
    pub printable: bool,
}

#[repr(C)]
pub struct keyCmd_t {
    pub down: bool,
    pub repeats: c_int,
    pub binding: *mut c_char,
}

#[repr(C)]
pub struct keyGlobals_t {
    pub keys: [keyCmd_t; 256],
    pub key_overstrikeMode: bool,
    pub anykeydown: bool,
    pub keyDownCount: c_int,
    pub g_consoleField: field_t,
    pub nextHistoryLine: c_int,
    pub historyLine: c_int,
    pub historyEditLines: [field_t; 32],
}

#[repr(C)]
pub struct clientActive_t {
    pub realtime: c_int,
    pub state: c_int,
    pub keyCatchers: c_int,
}

// Constants
const MAX_KEYS: usize = 256;
const MAX_STRING_CHARS: usize = 4096;
const MAX_EDIT_LINE: usize = 256;
const COMMAND_HISTORY: usize = 32;
const SMALLCHAR_WIDTH: c_int = 8;
const BIGCHAR_WIDTH: c_int = 16;

const A_NULL: c_int = 0x00;
const A_SHIFT: c_int = 0x01;
const A_CTRL: c_int = 0x02;
const A_ALT: c_int = 0x03;
const A_CAPSLOCK: c_int = 0x04;
const A_NUMLOCK: c_int = 0x05;
const A_SCROLLLOCK: c_int = 0x06;
const A_PAUSE: c_int = 0x07;
const A_BACKSPACE: c_int = 0x08;
const A_TAB: c_int = 0x09;
const A_ENTER: c_int = 0x0a;
const A_KP_PLUS: c_int = 0x0b;
const A_KP_MINUS: c_int = 0x0c;
const A_KP_ENTER: c_int = 0x0d;
const A_KP_PERIOD: c_int = 0x0e;
const A_PRINTSCREEN: c_int = 0x0f;
const A_KP_0: c_int = 0x10;
const A_KP_1: c_int = 0x11;
const A_KP_2: c_int = 0x12;
const A_KP_3: c_int = 0x13;
const A_KP_4: c_int = 0x14;
const A_KP_5: c_int = 0x15;
const A_KP_6: c_int = 0x16;
const A_KP_7: c_int = 0x17;
const A_KP_8: c_int = 0x18;
const A_KP_9: c_int = 0x19;
const A_CONSOLE: c_int = 0x1a;
const A_ESCAPE: c_int = 0x1b;
const A_F1: c_int = 0x1c;
const A_F2: c_int = 0x1d;
const A_F3: c_int = 0x1e;
const A_F4: c_int = 0x1f;
const A_SPACE: c_int = 0x20;
const A_PLING: c_int = 0x21;
const A_DOUBLE_QUOTE: c_int = 0x22;
const A_HASH: c_int = 0x23;
const A_STRING: c_int = 0x24;
const A_PERCENT: c_int = 0x25;
const A_AND: c_int = 0x26;
const A_SINGLE_QUOTE: c_int = 0x27;
const A_OPEN_BRACKET: c_int = 0x28;
const A_CLOSE_BRACKET: c_int = 0x29;
const A_STAR: c_int = 0x2a;
const A_PLUS: c_int = 0x2b;
const A_COMMA: c_int = 0x2c;
const A_MINUS: c_int = 0x2d;
const A_PERIOD: c_int = 0x2e;
const A_FORWARD_SLASH: c_int = 0x2f;
const A_0: c_int = 0x30;
const A_1: c_int = 0x31;
const A_2: c_int = 0x32;
const A_3: c_int = 0x33;
const A_4: c_int = 0x34;
const A_5: c_int = 0x35;
const A_6: c_int = 0x36;
const A_7: c_int = 0x37;
const A_8: c_int = 0x38;
const A_9: c_int = 0x39;
const A_COLON: c_int = 0x3a;
const A_SEMICOLON: c_int = 0x3b;
const A_LESSTHAN: c_int = 0x3c;
const A_EQUALS: c_int = 0x3d;
const A_GREATERTHAN: c_int = 0x3e;
const A_QUESTION: c_int = 0x3f;
const A_AT: c_int = 0x40;
const A_CAP_A: c_int = 0x41;
const A_CAP_B: c_int = 0x42;
const A_CAP_C: c_int = 0x43;
const A_CAP_D: c_int = 0x44;
const A_CAP_E: c_int = 0x45;
const A_CAP_F: c_int = 0x46;
const A_CAP_G: c_int = 0x47;
const A_CAP_H: c_int = 0x48;
const A_CAP_I: c_int = 0x49;
const A_CAP_J: c_int = 0x4a;
const A_CAP_K: c_int = 0x4b;
const A_CAP_L: c_int = 0x4c;
const A_CAP_M: c_int = 0x4d;
const A_CAP_N: c_int = 0x4e;
const A_CAP_O: c_int = 0x4f;
const A_CAP_P: c_int = 0x50;
const A_CAP_Q: c_int = 0x51;
const A_CAP_R: c_int = 0x52;
const A_CAP_S: c_int = 0x53;
const A_CAP_T: c_int = 0x54;
const A_CAP_U: c_int = 0x55;
const A_CAP_V: c_int = 0x56;
const A_CAP_W: c_int = 0x57;
const A_CAP_X: c_int = 0x58;
const A_CAP_Y: c_int = 0x59;
const A_CAP_Z: c_int = 0x5a;
const A_OPEN_SQUARE: c_int = 0x5b;
const A_BACKSLASH: c_int = 0x5c;
const A_CLOSE_SQUARE: c_int = 0x5d;
const A_CARET: c_int = 0x5e;
const A_UNDERSCORE: c_int = 0x5f;
const A_LEFT_SINGLE_QUOTE: c_int = 0x60;
const A_LOW_A: c_int = 0x61;
const A_LOW_B: c_int = 0x62;
const A_LOW_C: c_int = 0x63;
const A_LOW_D: c_int = 0x64;
const A_LOW_E: c_int = 0x65;
const A_LOW_F: c_int = 0x66;
const A_LOW_G: c_int = 0x67;
const A_LOW_H: c_int = 0x68;
const A_LOW_I: c_int = 0x69;
const A_LOW_J: c_int = 0x6a;
const A_LOW_K: c_int = 0x6b;
const A_LOW_L: c_int = 0x6c;
const A_LOW_M: c_int = 0x6d;
const A_LOW_N: c_int = 0x6e;
const A_LOW_O: c_int = 0x6f;
const A_LOW_P: c_int = 0x70;
const A_LOW_Q: c_int = 0x71;
const A_LOW_R: c_int = 0x72;
const A_LOW_S: c_int = 0x73;
const A_LOW_T: c_int = 0x74;
const A_LOW_U: c_int = 0x75;
const A_LOW_V: c_int = 0x76;
const A_LOW_W: c_int = 0x77;
const A_LOW_X: c_int = 0x78;
const A_LOW_Y: c_int = 0x79;
const A_LOW_Z: c_int = 0x7a;
const A_OPEN_BRACE: c_int = 0x7b;
const A_BAR: c_int = 0x7c;
const A_CLOSE_BRACE: c_int = 0x7d;
const A_TILDE: c_int = 0x7e;
const A_DELETE: c_int = 0x7f;
const A_EURO: c_int = 0x80;
const A_SHIFT2: c_int = 0x81;
const A_CTRL2: c_int = 0x82;
const A_ALT2: c_int = 0x83;
const A_F5: c_int = 0x84;
const A_F6: c_int = 0x85;
const A_F7: c_int = 0x86;
const A_F8: c_int = 0x87;
const A_CIRCUMFLEX: c_int = 0x88;
const A_MWHEELUP: c_int = 0x89;
const A_CAP_SCARON: c_int = 0x8a;
const A_MWHEELDOWN: c_int = 0x8b;
const A_CAP_OE: c_int = 0x8c;
const A_MOUSE1: c_int = 0x8d;
const A_MOUSE2: c_int = 0x8e;
const A_INSERT: c_int = 0x8f;
const A_HOME: c_int = 0x90;
const A_PAGE_UP: c_int = 0x91;
const A_RIGHT_SINGLE_QUOTE: c_int = 0x92;
const A_LEFT_DOUBLE_QUOTE: c_int = 0x93;
const A_RIGHT_DOUBLE_QUOTE: c_int = 0x94;
const A_F9: c_int = 0x95;
const A_F10: c_int = 0x96;
const A_F11: c_int = 0x97;
const A_F12: c_int = 0x98;
const A_TRADEMARK: c_int = 0x99;
const A_LOW_SCARON: c_int = 0x9a;
const A_ENTER: c_int = 0x9b;
const A_LOW_OE: c_int = 0x9c;
const A_END: c_int = 0x9d;
const A_PAGE_DOWN: c_int = 0x9e;
const A_CAP_YDIERESIS: c_int = 0x9f;
const A_SHIFT_SPACE: c_int = 0xa0;
const A_EXCLAMDOWN: c_int = 0xa1;
const A_CENT: c_int = 0xa2;
const A_POUND: c_int = 0xa3;
const A_SHIFT_KP_ENTER: c_int = 0xa4;
const A_YEN: c_int = 0xa5;
const A_MOUSE3: c_int = 0xa6;
const A_MOUSE4: c_int = 0xa7;
const A_MOUSE5: c_int = 0xa8;
const A_COPYRIGHT: c_int = 0xa9;
const A_CURSOR_UP: c_int = 0xaa;
const A_CURSOR_DOWN: c_int = 0xab;
const A_CURSOR_LEFT: c_int = 0xac;
const A_CURSOR_RIGHT: c_int = 0xad;
const A_REGISTERED: c_int = 0xae;
const A_UNDEFINED_7: c_int = 0xaf;
const A_UNDEFINED_8: c_int = 0xb0;
const A_UNDEFINED_9: c_int = 0xb1;
const A_UNDEFINED_10: c_int = 0xb2;
const A_UNDEFINED_11: c_int = 0xb3;
const A_UNDEFINED_12: c_int = 0xb4;
const A_UNDEFINED_13: c_int = 0xb5;
const A_UNDEFINED_14: c_int = 0xb6;
const A_UNDEFINED_15: c_int = 0xb7;
const A_UNDEFINED_16: c_int = 0xb8;
const A_UNDEFINED_17: c_int = 0xb9;
const A_UNDEFINED_18: c_int = 0xba;
const A_UNDEFINED_19: c_int = 0xbb;
const A_UNDEFINED_20: c_int = 0xbc;
const A_UNDEFINED_21: c_int = 0xbd;
const A_UNDEFINED_22: c_int = 0xbe;
const A_QUESTION_DOWN: c_int = 0xbf;
const A_CAP_AGRAVE: c_int = 0xc0;
const A_CAP_AACUTE: c_int = 0xc1;
const A_CAP_ACIRCUMFLEX: c_int = 0xc2;
const A_CAP_ATILDE: c_int = 0xc3;
const A_CAP_ADIERESIS: c_int = 0xc4;
const A_CAP_ARING: c_int = 0xc5;
const A_CAP_AE: c_int = 0xc6;
const A_CAP_CCEDILLA: c_int = 0xc7;
const A_CAP_EGRAVE: c_int = 0xc8;
const A_CAP_EACUTE: c_int = 0xc9;
const A_CAP_ECIRCUMFLEX: c_int = 0xca;
const A_CAP_EDIERESIS: c_int = 0xcb;
const A_CAP_IGRAVE: c_int = 0xcc;
const A_CAP_IACUTE: c_int = 0xcd;
const A_CAP_ICIRCUMFLEX: c_int = 0xce;
const A_CAP_IDIERESIS: c_int = 0xcf;
const A_CAP_ETH: c_int = 0xd0;
const A_CAP_NTILDE: c_int = 0xd1;
const A_CAP_OGRAVE: c_int = 0xd2;
const A_CAP_OACUTE: c_int = 0xd3;
const A_CAP_OCIRCUMFLEX: c_int = 0xd4;
const A_CAP_OTILDE: c_int = 0xd5;
const A_CAP_ODIERESIS: c_int = 0xd6;
const A_MULTIPLY: c_int = 0xd7;
const A_CAP_OSLASH: c_int = 0xd8;
const A_CAP_UGRAVE: c_int = 0xd9;
const A_CAP_UACUTE: c_int = 0xda;
const A_CAP_UCIRCUMFLEX: c_int = 0xdb;
const A_CAP_UDIERESIS: c_int = 0xdc;
const A_CAP_YACUTE: c_int = 0xdd;
const A_CAP_THORN: c_int = 0xde;
const A_GERMANDBLS: c_int = 0xdf;
const A_LOW_AGRAVE: c_int = 0xe0;
const A_LOW_AACUTE: c_int = 0xe1;
const A_LOW_ACIRCUMFLEX: c_int = 0xe2;
const A_LOW_ATILDE: c_int = 0xe3;
const A_LOW_ADIERESIS: c_int = 0xe4;
const A_LOW_ARING: c_int = 0xe5;
const A_LOW_AE: c_int = 0xe6;
const A_LOW_CCEDILLA: c_int = 0xe7;
const A_LOW_EGRAVE: c_int = 0xe8;
const A_LOW_EACUTE: c_int = 0xe9;
const A_LOW_ECIRCUMFLEX: c_int = 0xea;
const A_LOW_EDIERESIS: c_int = 0xeb;
const A_LOW_IGRAVE: c_int = 0xec;
const A_LOW_IACUTE: c_int = 0xed;
const A_LOW_ICIRCUMFLEX: c_int = 0xee;
const A_LOW_IDIERESIS: c_int = 0xef;
const A_LOW_ETH: c_int = 0xf0;
const A_LOW_NTILDE: c_int = 0xf1;
const A_LOW_OGRAVE: c_int = 0xf2;
const A_LOW_OACUTE: c_int = 0xf3;
const A_LOW_OCIRCUMFLEX: c_int = 0xf4;
const A_LOW_OTILDE: c_int = 0xf5;
const A_LOW_ODIERESIS: c_int = 0xf6;
const A_DIVIDE: c_int = 0xf7;
const A_LOW_OSLASH: c_int = 0xf8;
const A_LOW_UGRAVE: c_int = 0xf9;
const A_LOW_UACUTE: c_int = 0xfa;
const A_LOW_UCIRCUMFLEX: c_int = 0xfb;
const A_LOW_UDIERESIS: c_int = 0xfc;
const A_LOW_YACUTE: c_int = 0xfd;
const A_LOW_THORN: c_int = 0xfe;
const A_LOW_YDIERESIS: c_int = 0xff;
const A_JOY0: c_int = 0x100;
const A_JOY1: c_int = 0x101;
const A_JOY2: c_int = 0x102;
const A_JOY3: c_int = 0x103;
const A_JOY4: c_int = 0x104;
const A_JOY5: c_int = 0x105;
const A_JOY6: c_int = 0x106;
const A_JOY7: c_int = 0x107;
const A_JOY8: c_int = 0x108;
const A_JOY9: c_int = 0x109;
const A_JOY10: c_int = 0x10a;
const A_JOY11: c_int = 0x10b;
const A_JOY12: c_int = 0x10c;
const A_JOY13: c_int = 0x10d;
const A_JOY14: c_int = 0x10e;
const A_JOY15: c_int = 0x10f;
const A_JOY16: c_int = 0x110;
const A_JOY17: c_int = 0x111;
const A_JOY18: c_int = 0x112;
const A_JOY19: c_int = 0x113;
const A_JOY20: c_int = 0x114;
const A_JOY21: c_int = 0x115;
const A_JOY22: c_int = 0x116;
const A_JOY23: c_int = 0x117;
const A_JOY24: c_int = 0x118;
const A_JOY25: c_int = 0x119;
const A_JOY26: c_int = 0x11a;
const A_JOY27: c_int = 0x11b;
const A_JOY28: c_int = 0x11c;
const A_JOY29: c_int = 0x11d;
const A_JOY30: c_int = 0x11e;
const A_JOY31: c_int = 0x11f;
const A_AUX0: c_int = 0x120;
const A_AUX1: c_int = 0x121;
const A_AUX2: c_int = 0x122;
const A_AUX3: c_int = 0x123;
const A_AUX4: c_int = 0x124;
const A_AUX5: c_int = 0x125;
const A_AUX6: c_int = 0x126;
const A_AUX7: c_int = 0x127;
const A_AUX8: c_int = 0x128;
const A_AUX9: c_int = 0x129;
const A_AUX10: c_int = 0x12a;
const A_AUX11: c_int = 0x12b;
const A_AUX12: c_int = 0x12c;
const A_AUX13: c_int = 0x12d;
const A_AUX14: c_int = 0x12e;
const A_AUX15: c_int = 0x12f;
const A_AUX16: c_int = 0x130;
const A_AUX17: c_int = 0x131;
const A_AUX18: c_int = 0x132;
const A_AUX19: c_int = 0x133;
const A_AUX20: c_int = 0x134;
const A_AUX21: c_int = 0x135;
const A_AUX22: c_int = 0x136;
const A_AUX23: c_int = 0x137;
const A_AUX24: c_int = 0x138;
const A_AUX25: c_int = 0x139;
const A_AUX26: c_int = 0x13a;
const A_AUX27: c_int = 0x13b;
const A_AUX28: c_int = 0x13c;
const A_AUX29: c_int = 0x13d;
const A_AUX30: c_int = 0x13e;
const A_AUX31: c_int = 0x13f;

const KEYCATCH_CONSOLE: c_int = 1;
const KEYCATCH_UI: c_int = 2;
const KEYCATCH_MESSAGE: c_int = 4;
const CA_DISCONNECTED: c_int = 0;
const CA_ACTIVE: c_int = 1;
const CA_CINEMATIC: c_int = 2;
const K_CHAR_FLAG: c_int = 0x8000;
const CVAR_ARCHIVE: c_int = 1;

const ERR_DROP: c_int = 0;

// Globals

pub static mut chatField: field_t = field_t {
    buffer: [0; 256],
    cursor: 0,
    scroll: 0,
    widthInChars: 0,
};

pub static mut key_wastab: bool = false;  // Hit tab once already?

pub static mut keymatch_part: [c_char; 256] = [0; 256];
pub static mut keymatch_last: [c_char; 256] = [0; 256];

pub static mut kg: keyGlobals_t = keyGlobals_t {
    keys: [keyCmd_t {
        down: false,
        repeats: 0,
        binding: std::ptr::null_mut(),
    }; 256],
    key_overstrikeMode: false,
    anykeydown: false,
    keyDownCount: 0,
    g_consoleField: field_t {
        buffer: [0; 256],
        cursor: 0,
        scroll: 0,
        widthInChars: 0,
    },
    nextHistoryLine: 0,
    historyLine: 0,
    historyEditLines: [field_t {
        buffer: [0; 256],
        cursor: 0,
        scroll: 0,
        widthInChars: 0,
    }; 32],
};

// do NOT blithely change any of the key names (3rd field) here, since they have to match the key binds
// in the CFG files, they're also prepended with "KEYNAME_" when looking up StripEd references
pub static keynames: [keyname_t; MAX_KEYS] = [
    keyname_t { upper: 0x00, lower: 0x00, name: std::ptr::null(), keynum: A_NULL, printable: false },
    keyname_t { upper: 0x01, lower: 0x01, name: b"SHIFT\0".as_ptr() as *const c_char, keynum: A_SHIFT, printable: false },
    keyname_t { upper: 0x02, lower: 0x02, name: b"CTRL\0".as_ptr() as *const c_char, keynum: A_CTRL, printable: false },
    keyname_t { upper: 0x03, lower: 0x03, name: b"ALT\0".as_ptr() as *const c_char, keynum: A_ALT, printable: false },
    keyname_t { upper: 0x04, lower: 0x04, name: b"CAPSLOCK\0".as_ptr() as *const c_char, keynum: A_CAPSLOCK, printable: false },
    keyname_t { upper: 0x05, lower: 0x05, name: b"KP_NUMLOCK\0".as_ptr() as *const c_char, keynum: A_NUMLOCK, printable: false },
    keyname_t { upper: 0x06, lower: 0x06, name: b"SCROLLLOCK\0".as_ptr() as *const c_char, keynum: A_SCROLLLOCK, printable: false },
    keyname_t { upper: 0x07, lower: 0x07, name: b"PAUSE\0".as_ptr() as *const c_char, keynum: A_PAUSE, printable: false },
    keyname_t { upper: 0x08, lower: 0x08, name: b"BACKSPACE\0".as_ptr() as *const c_char, keynum: A_BACKSPACE, printable: false },
    keyname_t { upper: 0x09, lower: 0x09, name: b"TAB\0".as_ptr() as *const c_char, keynum: A_TAB, printable: false },
    keyname_t { upper: 0x0a, lower: 0x0a, name: b"ENTER\0".as_ptr() as *const c_char, keynum: A_ENTER, printable: false },
    keyname_t { upper: 0x0b, lower: 0x0b, name: b"KP_PLUS\0".as_ptr() as *const c_char, keynum: A_KP_PLUS, printable: false },
    keyname_t { upper: 0x0c, lower: 0x0c, name: b"KP_MINUS\0".as_ptr() as *const c_char, keynum: A_KP_MINUS, printable: false },
    keyname_t { upper: 0x0d, lower: 0x0d, name: b"KP_ENTER\0".as_ptr() as *const c_char, keynum: A_KP_ENTER, printable: false },
    keyname_t { upper: 0x0e, lower: 0x0e, name: b"KP_DEL\0".as_ptr() as *const c_char, keynum: A_KP_PERIOD, printable: false },
    keyname_t { upper: 0x0f, lower: 0x0f, name: std::ptr::null(), keynum: A_PRINTSCREEN, printable: false },
    keyname_t { upper: 0x10, lower: 0x10, name: b"KP_INS\0".as_ptr() as *const c_char, keynum: A_KP_0, printable: false },
    keyname_t { upper: 0x11, lower: 0x11, name: b"KP_END\0".as_ptr() as *const c_char, keynum: A_KP_1, printable: false },
    keyname_t { upper: 0x12, lower: 0x12, name: b"KP_DOWNARROW\0".as_ptr() as *const c_char, keynum: A_KP_2, printable: false },
    keyname_t { upper: 0x13, lower: 0x13, name: b"KP_PGDN\0".as_ptr() as *const c_char, keynum: A_KP_3, printable: false },
    keyname_t { upper: 0x14, lower: 0x14, name: b"KP_LEFTARROW\0".as_ptr() as *const c_char, keynum: A_KP_4, printable: false },
    keyname_t { upper: 0x15, lower: 0x15, name: b"KP_5\0".as_ptr() as *const c_char, keynum: A_KP_5, printable: false },
    keyname_t { upper: 0x16, lower: 0x16, name: b"KP_RIGHTARROW\0".as_ptr() as *const c_char, keynum: A_KP_6, printable: false },
    keyname_t { upper: 0x17, lower: 0x17, name: b"KP_HOME\0".as_ptr() as *const c_char, keynum: A_KP_7, printable: false },
    keyname_t { upper: 0x18, lower: 0x18, name: b"KP_UPARROW\0".as_ptr() as *const c_char, keynum: A_KP_8, printable: false },
    keyname_t { upper: 0x19, lower: 0x19, name: b"KP_PGUP\0".as_ptr() as *const c_char, keynum: A_KP_9, printable: false },
    keyname_t { upper: 0x1a, lower: 0x1a, name: b"CONSOLE\0".as_ptr() as *const c_char, keynum: A_CONSOLE, printable: false },
    keyname_t { upper: 0x1b, lower: 0x1b, name: b"ESCAPE\0".as_ptr() as *const c_char, keynum: A_ESCAPE, printable: false },
    keyname_t { upper: 0x1c, lower: 0x1c, name: b"F1\0".as_ptr() as *const c_char, keynum: A_F1, printable: true },
    keyname_t { upper: 0x1d, lower: 0x1d, name: b"F2\0".as_ptr() as *const c_char, keynum: A_F2, printable: true },
    keyname_t { upper: 0x1e, lower: 0x1e, name: b"F3\0".as_ptr() as *const c_char, keynum: A_F3, printable: true },
    keyname_t { upper: 0x1f, lower: 0x1f, name: b"F4\0".as_ptr() as *const c_char, keynum: A_F4, printable: true },
    keyname_t { upper: 0x20, lower: 0x20, name: b"SPACE\0".as_ptr() as *const c_char, keynum: A_SPACE, printable: false },
    keyname_t { upper: 0x21, lower: 0x21, name: std::ptr::null(), keynum: A_PLING, printable: false },
    keyname_t { upper: 0x22, lower: 0x22, name: std::ptr::null(), keynum: A_DOUBLE_QUOTE, printable: false },
    keyname_t { upper: 0x23, lower: 0x23, name: std::ptr::null(), keynum: A_HASH, printable: false },
    keyname_t { upper: 0x24, lower: 0x24, name: std::ptr::null(), keynum: A_STRING, printable: false },
    keyname_t { upper: 0x25, lower: 0x25, name: std::ptr::null(), keynum: A_PERCENT, printable: false },
    keyname_t { upper: 0x26, lower: 0x26, name: std::ptr::null(), keynum: A_AND, printable: false },
    keyname_t { upper: 0x27, lower: 0x27, name: std::ptr::null(), keynum: A_SINGLE_QUOTE, printable: false },
    keyname_t { upper: 0x28, lower: 0x28, name: std::ptr::null(), keynum: A_OPEN_BRACKET, printable: false },
    keyname_t { upper: 0x29, lower: 0x29, name: std::ptr::null(), keynum: A_CLOSE_BRACKET, printable: false },
    keyname_t { upper: 0x2a, lower: 0x2a, name: std::ptr::null(), keynum: A_STAR, printable: false },
    keyname_t { upper: 0x2b, lower: 0x2b, name: std::ptr::null(), keynum: A_PLUS, printable: false },
    keyname_t { upper: 0x2c, lower: 0x2c, name: std::ptr::null(), keynum: A_COMMA, printable: false },
    keyname_t { upper: 0x2d, lower: 0x2d, name: std::ptr::null(), keynum: A_MINUS, printable: false },
    keyname_t { upper: 0x2e, lower: 0x2e, name: std::ptr::null(), keynum: A_PERIOD, printable: false },
    keyname_t { upper: 0x2f, lower: 0x2f, name: std::ptr::null(), keynum: A_FORWARD_SLASH, printable: false },
    keyname_t { upper: 0x30, lower: 0x30, name: std::ptr::null(), keynum: A_0, printable: false },
    keyname_t { upper: 0x31, lower: 0x31, name: std::ptr::null(), keynum: A_1, printable: false },
    keyname_t { upper: 0x32, lower: 0x32, name: std::ptr::null(), keynum: A_2, printable: false },
    keyname_t { upper: 0x33, lower: 0x33, name: std::ptr::null(), keynum: A_3, printable: false },
    keyname_t { upper: 0x34, lower: 0x34, name: std::ptr::null(), keynum: A_4, printable: false },
    keyname_t { upper: 0x35, lower: 0x35, name: std::ptr::null(), keynum: A_5, printable: false },
    keyname_t { upper: 0x36, lower: 0x36, name: std::ptr::null(), keynum: A_6, printable: false },
    keyname_t { upper: 0x37, lower: 0x37, name: std::ptr::null(), keynum: A_7, printable: false },
    keyname_t { upper: 0x38, lower: 0x38, name: std::ptr::null(), keynum: A_8, printable: false },
    keyname_t { upper: 0x39, lower: 0x39, name: std::ptr::null(), keynum: A_9, printable: false },
    keyname_t { upper: 0x3a, lower: 0x3a, name: std::ptr::null(), keynum: A_COLON, printable: false },
    keyname_t { upper: 0x3b, lower: 0x3b, name: b"SEMICOLON\0".as_ptr() as *const c_char, keynum: A_SEMICOLON, printable: false },
    keyname_t { upper: 0x3c, lower: 0x3c, name: std::ptr::null(), keynum: A_LESSTHAN, printable: false },
    keyname_t { upper: 0x3d, lower: 0x3d, name: std::ptr::null(), keynum: A_EQUALS, printable: false },
    keyname_t { upper: 0x3e, lower: 0x3e, name: std::ptr::null(), keynum: A_GREATERTHAN, printable: false },
    keyname_t { upper: 0x3f, lower: 0x3f, name: std::ptr::null(), keynum: A_QUESTION, printable: false },
    keyname_t { upper: 0x40, lower: 0x40, name: std::ptr::null(), keynum: A_AT, printable: false },
    keyname_t { upper: 0x41, lower: 0x61, name: std::ptr::null(), keynum: A_CAP_A, printable: false },
    keyname_t { upper: 0x42, lower: 0x62, name: std::ptr::null(), keynum: A_CAP_B, printable: false },
    keyname_t { upper: 0x43, lower: 0x63, name: std::ptr::null(), keynum: A_CAP_C, printable: false },
    keyname_t { upper: 0x44, lower: 0x64, name: std::ptr::null(), keynum: A_CAP_D, printable: false },
    keyname_t { upper: 0x45, lower: 0x65, name: std::ptr::null(), keynum: A_CAP_E, printable: false },
    keyname_t { upper: 0x46, lower: 0x66, name: std::ptr::null(), keynum: A_CAP_F, printable: false },
    keyname_t { upper: 0x47, lower: 0x67, name: std::ptr::null(), keynum: A_CAP_G, printable: false },
    keyname_t { upper: 0x48, lower: 0x68, name: std::ptr::null(), keynum: A_CAP_H, printable: false },
    keyname_t { upper: 0x49, lower: 0x69, name: std::ptr::null(), keynum: A_CAP_I, printable: false },
    keyname_t { upper: 0x4a, lower: 0x6a, name: std::ptr::null(), keynum: A_CAP_J, printable: false },
    keyname_t { upper: 0x4b, lower: 0x6b, name: std::ptr::null(), keynum: A_CAP_K, printable: false },
    keyname_t { upper: 0x4c, lower: 0x6c, name: std::ptr::null(), keynum: A_CAP_L, printable: false },
    keyname_t { upper: 0x4d, lower: 0x6d, name: std::ptr::null(), keynum: A_CAP_M, printable: false },
    keyname_t { upper: 0x4e, lower: 0x6e, name: std::ptr::null(), keynum: A_CAP_N, printable: false },
    keyname_t { upper: 0x4f, lower: 0x6f, name: std::ptr::null(), keynum: A_CAP_O, printable: false },
    keyname_t { upper: 0x50, lower: 0x70, name: std::ptr::null(), keynum: A_CAP_P, printable: false },
    keyname_t { upper: 0x51, lower: 0x71, name: std::ptr::null(), keynum: A_CAP_Q, printable: false },
    keyname_t { upper: 0x52, lower: 0x72, name: std::ptr::null(), keynum: A_CAP_R, printable: false },
    keyname_t { upper: 0x53, lower: 0x73, name: std::ptr::null(), keynum: A_CAP_S, printable: false },
    keyname_t { upper: 0x54, lower: 0x74, name: std::ptr::null(), keynum: A_CAP_T, printable: false },
    keyname_t { upper: 0x55, lower: 0x75, name: std::ptr::null(), keynum: A_CAP_U, printable: false },
    keyname_t { upper: 0x56, lower: 0x76, name: std::ptr::null(), keynum: A_CAP_V, printable: false },
    keyname_t { upper: 0x57, lower: 0x77, name: std::ptr::null(), keynum: A_CAP_W, printable: false },
    keyname_t { upper: 0x58, lower: 0x78, name: std::ptr::null(), keynum: A_CAP_X, printable: false },
    keyname_t { upper: 0x59, lower: 0x79, name: std::ptr::null(), keynum: A_CAP_Y, printable: false },
    keyname_t { upper: 0x5a, lower: 0x7a, name: std::ptr::null(), keynum: A_CAP_Z, printable: false },
    keyname_t { upper: 0x5b, lower: 0x5b, name: std::ptr::null(), keynum: A_OPEN_SQUARE, printable: false },
    keyname_t { upper: 0x5c, lower: 0x5c, name: std::ptr::null(), keynum: A_BACKSLASH, printable: false },
    keyname_t { upper: 0x5d, lower: 0x5d, name: std::ptr::null(), keynum: A_CLOSE_SQUARE, printable: false },
    keyname_t { upper: 0x5e, lower: 0x5e, name: std::ptr::null(), keynum: A_CARET, printable: false },
    keyname_t { upper: 0x5f, lower: 0x5f, name: std::ptr::null(), keynum: A_UNDERSCORE, printable: false },
    keyname_t { upper: 0x60, lower: 0x60, name: std::ptr::null(), keynum: A_LEFT_SINGLE_QUOTE, printable: false },
    keyname_t { upper: 0x41, lower: 0x61, name: std::ptr::null(), keynum: A_LOW_A, printable: false },
    keyname_t { upper: 0x42, lower: 0x62, name: std::ptr::null(), keynum: A_LOW_B, printable: false },
    keyname_t { upper: 0x43, lower: 0x63, name: std::ptr::null(), keynum: A_LOW_C, printable: false },
    keyname_t { upper: 0x44, lower: 0x64, name: std::ptr::null(), keynum: A_LOW_D, printable: false },
    keyname_t { upper: 0x45, lower: 0x65, name: std::ptr::null(), keynum: A_LOW_E, printable: false },
    keyname_t { upper: 0x46, lower: 0x66, name: std::ptr::null(), keynum: A_LOW_F, printable: false },
    keyname_t { upper: 0x47, lower: 0x67, name: std::ptr::null(), keynum: A_LOW_G, printable: false },
    keyname_t { upper: 0x48, lower: 0x68, name: std::ptr::null(), keynum: A_LOW_H, printable: false },
    keyname_t { upper: 0x49, lower: 0x69, name: std::ptr::null(), keynum: A_LOW_I, printable: false },
    keyname_t { upper: 0x4a, lower: 0x6a, name: std::ptr::null(), keynum: A_LOW_J, printable: false },
    keyname_t { upper: 0x4b, lower: 0x6b, name: std::ptr::null(), keynum: A_LOW_K, printable: false },
    keyname_t { upper: 0x4c, lower: 0x6c, name: std::ptr::null(), keynum: A_LOW_L, printable: false },
    keyname_t { upper: 0x4d, lower: 0x6d, name: std::ptr::null(), keynum: A_LOW_M, printable: false },
    keyname_t { upper: 0x4e, lower: 0x6e, name: std::ptr::null(), keynum: A_LOW_N, printable: false },
    keyname_t { upper: 0x4f, lower: 0x6f, name: std::ptr::null(), keynum: A_LOW_O, printable: false },
    keyname_t { upper: 0x50, lower: 0x70, name: std::ptr::null(), keynum: A_LOW_P, printable: false },
    keyname_t { upper: 0x51, lower: 0x71, name: std::ptr::null(), keynum: A_LOW_Q, printable: false },
    keyname_t { upper: 0x52, lower: 0x72, name: std::ptr::null(), keynum: A_LOW_R, printable: false },
    keyname_t { upper: 0x53, lower: 0x73, name: std::ptr::null(), keynum: A_LOW_S, printable: false },
    keyname_t { upper: 0x54, lower: 0x74, name: std::ptr::null(), keynum: A_LOW_T, printable: false },
    keyname_t { upper: 0x55, lower: 0x75, name: std::ptr::null(), keynum: A_LOW_U, printable: false },
    keyname_t { upper: 0x56, lower: 0x76, name: std::ptr::null(), keynum: A_LOW_V, printable: false },
    keyname_t { upper: 0x57, lower: 0x77, name: std::ptr::null(), keynum: A_LOW_W, printable: false },
    keyname_t { upper: 0x58, lower: 0x78, name: std::ptr::null(), keynum: A_LOW_X, printable: false },
    keyname_t { upper: 0x59, lower: 0x79, name: std::ptr::null(), keynum: A_LOW_Y, printable: false },
    keyname_t { upper: 0x5a, lower: 0x7a, name: std::ptr::null(), keynum: A_LOW_Z, printable: false },
    keyname_t { upper: 0x7b, lower: 0x7b, name: std::ptr::null(), keynum: A_OPEN_BRACE, printable: false },
    keyname_t { upper: 0x7c, lower: 0x7c, name: std::ptr::null(), keynum: A_BAR, printable: false },
    keyname_t { upper: 0x7d, lower: 0x7d, name: std::ptr::null(), keynum: A_CLOSE_BRACE, printable: false },
    keyname_t { upper: 0x7e, lower: 0x7e, name: std::ptr::null(), keynum: A_TILDE, printable: false },
    keyname_t { upper: 0x7f, lower: 0x7f, name: b"DEL\0".as_ptr() as *const c_char, keynum: A_DELETE, printable: false },
    keyname_t { upper: 0x80, lower: 0x80, name: b"EURO\0".as_ptr() as *const c_char, keynum: A_EURO, printable: false },
    keyname_t { upper: 0x81, lower: 0x81, name: b"SHIFT\0".as_ptr() as *const c_char, keynum: A_SHIFT2, printable: false },
    keyname_t { upper: 0x82, lower: 0x82, name: b"CTRL\0".as_ptr() as *const c_char, keynum: A_CTRL2, printable: false },
    keyname_t { upper: 0x83, lower: 0x83, name: b"ALT\0".as_ptr() as *const c_char, keynum: A_ALT2, printable: false },
    keyname_t { upper: 0x84, lower: 0x84, name: b"F5\0".as_ptr() as *const c_char, keynum: A_F5, printable: true },
    keyname_t { upper: 0x85, lower: 0x85, name: b"F6\0".as_ptr() as *const c_char, keynum: A_F6, printable: true },
    keyname_t { upper: 0x86, lower: 0x86, name: b"F7\0".as_ptr() as *const c_char, keynum: A_F7, printable: true },
    keyname_t { upper: 0x87, lower: 0x87, name: b"F8\0".as_ptr() as *const c_char, keynum: A_F8, printable: true },
    keyname_t { upper: 0x88, lower: 0x88, name: b"CIRCUMFLEX\0".as_ptr() as *const c_char, keynum: A_CIRCUMFLEX, printable: false },
    keyname_t { upper: 0x89, lower: 0x89, name: b"MWHEELUP\0".as_ptr() as *const c_char, keynum: A_MWHEELUP, printable: false },
    keyname_t { upper: 0x8a, lower: 0x9a, name: std::ptr::null(), keynum: A_CAP_SCARON, printable: false }, // ******
    keyname_t { upper: 0x8b, lower: 0x8b, name: b"MWHEELDOWN\0".as_ptr() as *const c_char, keynum: A_MWHEELDOWN, printable: false },
    keyname_t { upper: 0x8c, lower: 0x9c, name: std::ptr::null(), keynum: A_CAP_OE, printable: false }, // ******
    keyname_t { upper: 0x8d, lower: 0x8d, name: b"MOUSE1\0".as_ptr() as *const c_char, keynum: A_MOUSE1, printable: false },
    keyname_t { upper: 0x8e, lower: 0x8e, name: b"MOUSE2\0".as_ptr() as *const c_char, keynum: A_MOUSE2, printable: false },
    keyname_t { upper: 0x8f, lower: 0x8f, name: b"INS\0".as_ptr() as *const c_char, keynum: A_INSERT, printable: false },
    keyname_t { upper: 0x90, lower: 0x90, name: b"HOME\0".as_ptr() as *const c_char, keynum: A_HOME, printable: false },
    keyname_t { upper: 0x91, lower: 0x91, name: b"PGUP\0".as_ptr() as *const c_char, keynum: A_PAGE_UP, printable: false },
    keyname_t { upper: 0x92, lower: 0x92, name: std::ptr::null(), keynum: A_RIGHT_SINGLE_QUOTE, printable: false },
    keyname_t { upper: 0x93, lower: 0x93, name: std::ptr::null(), keynum: A_LEFT_DOUBLE_QUOTE, printable: false },
    keyname_t { upper: 0x94, lower: 0x94, name: std::ptr::null(), keynum: A_RIGHT_DOUBLE_QUOTE, printable: false },
    keyname_t { upper: 0x95, lower: 0x95, name: b"F9\0".as_ptr() as *const c_char, keynum: A_F9, printable: true },
    keyname_t { upper: 0x96, lower: 0x96, name: b"F10\0".as_ptr() as *const c_char, keynum: A_F10, printable: true },
    keyname_t { upper: 0x97, lower: 0x97, name: b"F11\0".as_ptr() as *const c_char, keynum: A_F11, printable: true },
    keyname_t { upper: 0x98, lower: 0x98, name: b"F12\0".as_ptr() as *const c_char, keynum: A_F12, printable: true },
    keyname_t { upper: 0x99, lower: 0x99, name: std::ptr::null(), keynum: A_TRADEMARK, printable: false },
    keyname_t { upper: 0x8a, lower: 0x9a, name: std::ptr::null(), keynum: A_LOW_SCARON, printable: false }, // ******
    keyname_t { upper: 0x9b, lower: 0x9b, name: b"SHIFT_ENTER\0".as_ptr() as *const c_char, keynum: A_ENTER, printable: false },
    keyname_t { upper: 0x8c, lower: 0x9c, name: std::ptr::null(), keynum: A_LOW_OE, printable: false }, // ******
    keyname_t { upper: 0x9d, lower: 0x9d, name: b"END\0".as_ptr() as *const c_char, keynum: A_END, printable: false },
    keyname_t { upper: 0x9e, lower: 0x9e, name: b"PGDN\0".as_ptr() as *const c_char, keynum: A_PAGE_DOWN, printable: false },
    keyname_t { upper: 0x9f, lower: 0xff, name: std::ptr::null(), keynum: A_CAP_YDIERESIS, printable: false }, // ******
    keyname_t { upper: 0xa0, lower: 0x00, name: b"SHIFT_SPACE\0".as_ptr() as *const c_char, keynum: A_SPACE, printable: false },
    keyname_t { upper: 0xa1, lower: 0xa1, name: std::ptr::null(), keynum: A_EXCLAMDOWN, printable: false }, // upside down '!' - undisplayable
    keyname_t { upper: 0xa2, lower: 0xa2, name: std::ptr::null(), keynum: A_CENT, printable: false },
    keyname_t { upper: 0xa3, lower: 0xa3, name: std::ptr::null(), keynum: A_POUND, printable: false },
    keyname_t { upper: 0xa4, lower: 0x00, name: b"SHIFT_KP_ENTER\0".as_ptr() as *const c_char, keynum: A_KP_ENTER, printable: false },
    keyname_t { upper: 0xa5, lower: 0xa5, name: std::ptr::null(), keynum: A_YEN, printable: false },
    keyname_t { upper: 0xa6, lower: 0xa6, name: b"MOUSE3\0".as_ptr() as *const c_char, keynum: A_MOUSE3, printable: false },
    keyname_t { upper: 0xa7, lower: 0xa7, name: b"MOUSE4\0".as_ptr() as *const c_char, keynum: A_MOUSE4, printable: false },
    keyname_t { upper: 0xa8, lower: 0xa8, name: b"MOUSE5\0".as_ptr() as *const c_char, keynum: A_MOUSE5, printable: false },
    keyname_t { upper: 0xa9, lower: 0xa9, name: std::ptr::null(), keynum: A_COPYRIGHT, printable: false },
    keyname_t { upper: 0xaa, lower: 0xaa, name: b"UPARROW\0".as_ptr() as *const c_char, keynum: A_CURSOR_UP, printable: false },
    keyname_t { upper: 0xab, lower: 0xab, name: b"DOWNARROW\0".as_ptr() as *const c_char, keynum: A_CURSOR_DOWN, printable: false },
    keyname_t { upper: 0xac, lower: 0xac, name: b"LEFTARROW\0".as_ptr() as *const c_char, keynum: A_CURSOR_LEFT, printable: false },
    keyname_t { upper: 0xad, lower: 0xad, name: b"RIGHTARROW\0".as_ptr() as *const c_char, keynum: A_CURSOR_RIGHT, printable: false },
    keyname_t { upper: 0xae, lower: 0xae, name: std::ptr::null(), keynum: A_REGISTERED, printable: false },
    keyname_t { upper: 0xaf, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_7, printable: false },
    keyname_t { upper: 0xb0, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_8, printable: false },
    keyname_t { upper: 0xb1, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_9, printable: false },
    keyname_t { upper: 0xb2, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_10, printable: false },
    keyname_t { upper: 0xb3, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_11, printable: false },
    keyname_t { upper: 0xb4, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_12, printable: false },
    keyname_t { upper: 0xb5, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_13, printable: false },
    keyname_t { upper: 0xb6, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_14, printable: false },
    keyname_t { upper: 0xb7, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_15, printable: false },
    keyname_t { upper: 0xb8, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_16, printable: false },
    keyname_t { upper: 0xb9, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_17, printable: false },
    keyname_t { upper: 0xba, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_18, printable: false },
    keyname_t { upper: 0xbb, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_19, printable: false },
    keyname_t { upper: 0xbc, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_20, printable: false },
    keyname_t { upper: 0xbd, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_21, printable: false },
    keyname_t { upper: 0xbe, lower: 0x00, name: std::ptr::null(), keynum: A_UNDEFINED_22, printable: false },
    keyname_t { upper: 0xbf, lower: 0xbf, name: std::ptr::null(), keynum: A_QUESTION_DOWN, printable: false },
    keyname_t { upper: 0xc0, lower: 0xe0, name: std::ptr::null(), keynum: A_CAP_AGRAVE, printable: false },
    keyname_t { upper: 0xc1, lower: 0xe1, name: std::ptr::null(), keynum: A_CAP_AACUTE, printable: false },
    keyname_t { upper: 0xc2, lower: 0xe2, name: std::ptr::null(), keynum: A_CAP_ACIRCUMFLEX, printable: false },
    keyname_t { upper: 0xc3, lower: 0xe3, name: std::ptr::null(), keynum: A_CAP_ATILDE, printable: false },
    keyname_t { upper: 0xc4, lower: 0xe4, name: std::ptr::null(), keynum: A_CAP_ADIERESIS, printable: false },
    keyname_t { upper: 0xc5, lower: 0xe5, name: std::ptr::null(), keynum: A_CAP_ARING, printable: false },
    keyname_t { upper: 0xc6, lower: 0xe6, name: std::ptr::null(), keynum: A_CAP_AE, printable: false },
    keyname_t { upper: 0xc7, lower: 0xe7, name: std::ptr::null(), keynum: A_CAP_CCEDILLA, printable: false },
    keyname_t { upper: 0xc8, lower: 0xe8, name: std::ptr::null(), keynum: A_CAP_EGRAVE, printable: false },
    keyname_t { upper: 0xc9, lower: 0xe9, name: std::ptr::null(), keynum: A_CAP_EACUTE, printable: false },
    keyname_t { upper: 0xca, lower: 0xea, name: std::ptr::null(), keynum: A_CAP_ECIRCUMFLEX, printable: false },
    keyname_t { upper: 0xcb, lower: 0xeb, name: std::ptr::null(), keynum: A_CAP_EDIERESIS, printable: false },
    keyname_t { upper: 0xcc, lower: 0xec, name: std::ptr::null(), keynum: A_CAP_IGRAVE, printable: false },
    keyname_t { upper: 0xcd, lower: 0xed, name: std::ptr::null(), keynum: A_CAP_IACUTE, printable: false },
    keyname_t { upper: 0xce, lower: 0xee, name: std::ptr::null(), keynum: A_CAP_ICIRCUMFLEX, printable: false },
    keyname_t { upper: 0xcf, lower: 0xef, name: std::ptr::null(), keynum: A_CAP_IDIERESIS, printable: false },
    keyname_t { upper: 0xd0, lower: 0xf0, name: std::ptr::null(), keynum: A_CAP_ETH, printable: false },
    keyname_t { upper: 0xd1, lower: 0xf1, name: std::ptr::null(), keynum: A_CAP_NTILDE, printable: false },
    keyname_t { upper: 0xd2, lower: 0xf2, name: std::ptr::null(), keynum: A_CAP_OGRAVE, printable: false },
    keyname_t { upper: 0xd3, lower: 0xf3, name: std::ptr::null(), keynum: A_CAP_OACUTE, printable: false },
    keyname_t { upper: 0xd4, lower: 0xf4, name: std::ptr::null(), keynum: A_CAP_OCIRCUMFLEX, printable: false },
    keyname_t { upper: 0xd5, lower: 0xf5, name: std::ptr::null(), keynum: A_CAP_OTILDE, printable: false },
    keyname_t { upper: 0xd6, lower: 0xf6, name: std::ptr::null(), keynum: A_CAP_ODIERESIS, printable: false },
    keyname_t { upper: 0xd7, lower: 0xf7, name: b"KP_STAR\0".as_ptr() as *const c_char, keynum: A_MULTIPLY, printable: false },
    keyname_t { upper: 0xd8, lower: 0xf8, name: std::ptr::null(), keynum: A_CAP_OSLASH, printable: false },
    keyname_t { upper: 0xd9, lower: 0xf9, name: std::ptr::null(), keynum: A_CAP_UGRAVE, printable: false },
    keyname_t { upper: 0xda, lower: 0xfa, name: std::ptr::null(), keynum: A_CAP_UACUTE, printable: false },
    keyname_t { upper: 0xdb, lower: 0xfb, name: std::ptr::null(), keynum: A_CAP_UCIRCUMFLEX, printable: false },
    keyname_t { upper: 0xdc, lower: 0xfc, name: std::ptr::null(), keynum: A_CAP_UDIERESIS, printable: false },
    keyname_t { upper: 0xdd, lower: 0xfd, name: std::ptr::null(), keynum: A_CAP_YACUTE, printable: false },
    keyname_t { upper: 0xde, lower: 0xfe, name: std::ptr::null(), keynum: A_CAP_THORN, printable: false },
    keyname_t { upper: 0xdf, lower: 0xdf, name: std::ptr::null(), keynum: A_GERMANDBLS, printable: false },
    keyname_t { upper: 0xe0, lower: 0xe0, name: std::ptr::null(), keynum: A_LOW_AGRAVE, printable: false },
    keyname_t { upper: 0xe1, lower: 0xe1, name: std::ptr::null(), keynum: A_LOW_AACUTE, printable: false },
    keyname_t { upper: 0xe2, lower: 0xe2, name: std::ptr::null(), keynum: A_LOW_ACIRCUMFLEX, printable: false },
    keyname_t { upper: 0xe3, lower: 0xe3, name: std::ptr::null(), keynum: A_LOW_ATILDE, printable: false },
    keyname_t { upper: 0xe4, lower: 0xe4, name: std::ptr::null(), keynum: A_LOW_ADIERESIS, printable: false },
    keyname_t { upper: 0xe5, lower: 0xe5, name: std::ptr::null(), keynum: A_LOW_ARING, printable: false },
    keyname_t { upper: 0xe6, lower: 0xe6, name: std::ptr::null(), keynum: A_LOW_AE, printable: false },
    keyname_t { upper: 0xe7, lower: 0xe7, name: std::ptr::null(), keynum: A_LOW_CCEDILLA, printable: false },
    keyname_t { upper: 0xe8, lower: 0xe8, name: std::ptr::null(), keynum: A_LOW_EGRAVE, printable: false },
    keyname_t { upper: 0xe9, lower: 0xe9, name: std::ptr::null(), keynum: A_LOW_EACUTE, printable: false },
    keyname_t { upper: 0xea, lower: 0xea, name: std::ptr::null(), keynum: A_LOW_ECIRCUMFLEX, printable: false },
    keyname_t { upper: 0xeb, lower: 0xeb, name: std::ptr::null(), keynum: A_LOW_EDIERESIS, printable: false },
    keyname_t { upper: 0xec, lower: 0xec, name: std::ptr::null(), keynum: A_LOW_IGRAVE, printable: false },
    keyname_t { upper: 0xed, lower: 0xed, name: std::ptr::null(), keynum: A_LOW_IACUTE, printable: false },
    keyname_t { upper: 0xee, lower: 0xee, name: std::ptr::null(), keynum: A_LOW_ICIRCUMFLEX, printable: false },
    keyname_t { upper: 0xef, lower: 0xef, name: std::ptr::null(), keynum: A_LOW_IDIERESIS, printable: false },
    keyname_t { upper: 0xf0, lower: 0xf0, name: std::ptr::null(), keynum: A_LOW_ETH, printable: false },
    keyname_t { upper: 0xf1, lower: 0xf1, name: std::ptr::null(), keynum: A_LOW_NTILDE, printable: false },
    keyname_t { upper: 0xf2, lower: 0xf2, name: std::ptr::null(), keynum: A_LOW_OGRAVE, printable: false },
    keyname_t { upper: 0xf3, lower: 0xf3, name: std::ptr::null(), keynum: A_LOW_OACUTE, printable: false },
    keyname_t { upper: 0xf4, lower: 0xf4, name: std::ptr::null(), keynum: A_LOW_OCIRCUMFLEX, printable: false },
    keyname_t { upper: 0xf5, lower: 0xf5, name: std::ptr::null(), keynum: A_LOW_OTILDE, printable: false },
    keyname_t { upper: 0xf6, lower: 0xf6, name: std::ptr::null(), keynum: A_LOW_ODIERESIS, printable: false },
    keyname_t { upper: 0xf7, lower: 0xf7, name: b"KP_SLASH\0".as_ptr() as *const c_char, keynum: A_DIVIDE, printable: false },
    keyname_t { upper: 0xf8, lower: 0xf8, name: std::ptr::null(), keynum: A_LOW_OSLASH, printable: false },
    keyname_t { upper: 0xf9, lower: 0xf9, name: std::ptr::null(), keynum: A_LOW_UGRAVE, printable: false },
    keyname_t { upper: 0xfa, lower: 0xfa, name: std::ptr::null(), keynum: A_LOW_UACUTE, printable: false },
    keyname_t { upper: 0xfb, lower: 0xfb, name: std::ptr::null(), keynum: A_LOW_UCIRCUMFLEX, printable: false },
    keyname_t { upper: 0xfc, lower: 0xfc, name: std::ptr::null(), keynum: A_LOW_UDIERESIS, printable: false },
    keyname_t { upper: 0xfd, lower: 0xfd, name: std::ptr::null(), keynum: A_LOW_YACUTE, printable: false },
    keyname_t { upper: 0xfe, lower: 0xfe, name: std::ptr::null(), keynum: A_LOW_THORN, printable: false },
    keyname_t { upper: 0x9f, lower: 0xff, name: std::ptr::null(), keynum: A_LOW_YDIERESIS, printable: false }, // *******
];

static mut g_console_field_width: c_int = 0;

// ==================
// Field_Draw
//
// Handles horizontal scrolling and cursor blinking
// x, y, amd width are in pixels
// ==================
unsafe fn Field_VariableSizeDraw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, size: c_int, showCursor: bool) {
    let mut len: c_int;
    let mut drawLen: c_int;
    let mut prestep: c_int;
    let mut cursorChar: c_int;
    let mut str: [c_char; 4096];
    let mut i: c_int;

    drawLen = (*edit).widthInChars;
    len = strlen((*edit).buffer.as_ptr() as *const c_char) as c_int + 1;

    // guarantee that cursor will be visible
    if len <= drawLen {
        prestep = 0;
    } else {
        if (*edit).scroll + drawLen > len {
            (*edit).scroll = len - drawLen;
            if (*edit).scroll < 0 {
                (*edit).scroll = 0;
            }
        }
        prestep = (*edit).scroll;
        // if ( edit->cursor < len - drawLen ) {
        //     prestep = edit->cursor;	// cursor at start
        // } else {
        //     prestep = len - drawLen;
        // }
    }

    if prestep + drawLen > len {
        drawLen = len - prestep;
    }

    // extract <drawLen> characters from the field at <prestep>
    if drawLen >= MAX_STRING_CHARS as c_int {
        Com_Error(ERR_DROP, b"drawLen >= MAX_STRING_CHARS\0".as_ptr() as *const c_char);
    }
    memcpy(str.as_mut_ptr() as *mut c_void, ((*edit).buffer.as_ptr() as *const c_char as usize + prestep as usize) as *const c_void, drawLen as usize);
    str[drawLen as usize] = 0;

    // draw it
    if size == SMALLCHAR_WIDTH {
        i = 0;
        while i < drawLen - 1 {
            SCR_DrawSmallChar(x + i * SMALLCHAR_WIDTH, y, str[i as usize] as c_int);
            i += 1;
        }
    } else {
        // draw big string with drop shadow
        SCR_DrawBigString(x, y, str.as_ptr(), 1.0);
    }

    // draw the cursor
    if !showCursor {
        return;
    }

    if ((cls.realtime >> 8) & 1) != 0 {
        return;  // off blink
    }

    if kg.key_overstrikeMode {
        cursorChar = 11;
    } else {
        cursorChar = 10;
    }
    if size == SMALLCHAR_WIDTH {
        SCR_DrawSmallChar(x + ((*edit).cursor - prestep) * size, y, cursorChar);
    } else {
        str[0] = cursorChar as c_char;
        str[1] = 0;
        SCR_DrawBigString(x + ((*edit).cursor - prestep) * size, y, str.as_ptr(), 1.0);
    }
}

pub unsafe fn Field_Draw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, showCursor: bool) {
    Field_VariableSizeDraw(edit, x, y, width, SMALLCHAR_WIDTH, showCursor);
}

pub unsafe fn Field_BigDraw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, showCursor: bool) {
    Field_VariableSizeDraw(edit, x, y, width, BIGCHAR_WIDTH, showCursor);
}

// ================
// Field_Paste
// ================
pub unsafe fn Field_Paste(edit: *mut field_t) {
    let mut cbd: *mut c_char;
    let mut pasteLen: c_int;
    let mut i: c_int;

    cbd = Sys_GetClipboardData();

    if cbd.is_null() {
        return;
    }

    // send as if typed, so insert / overstrike works properly
    pasteLen = strlen(cbd) as c_int;
    i = 0;
    while i < pasteLen {
        Field_CharEvent(edit, *cbd.offset(i as isize) as u32);
        i += 1;
    }

    Z_Free(cbd as *mut c_void);
}

// =================
// Field_KeyDownEvent
//
// Performs the basic line editing functions for the console,
// in-game talk, and menu fields
//
// Key events are used for non-printable characters, others are gotten from char events.
// =================
pub unsafe fn Field_KeyDownEvent(edit: *mut field_t, key: c_int) {
    let mut len: c_int;

    // shift-insert is paste
    if (key == A_INSERT) && kg.keys[A_SHIFT as usize].down {
        Field_Paste(edit);
        return;
    }

    len = strlen((*edit).buffer.as_ptr() as *const c_char) as c_int;

    if key == A_DELETE {
        if (*edit).cursor < len {
            memmove(
                ((*edit).buffer.as_mut_ptr() as *mut c_char).offset((*edit).cursor as isize) as *mut c_void,
                ((*edit).buffer.as_ptr() as *const c_char).offset(((*edit).cursor + 1) as isize) as *const c_void,
                (len - (*edit).cursor) as usize,
            );
        }
        return;
    }

    if key == A_CURSOR_RIGHT {
        if (*edit).cursor < len {
            (*edit).cursor += 1;
        }
        if (*edit).cursor >= (*edit).scroll + (*edit).widthInChars && (*edit).cursor <= len {
            (*edit).scroll += 1;
        }
        return;
    }

    if key == A_CURSOR_LEFT {
        if (*edit).cursor > 0 {
            (*edit).cursor -= 1;
        }
        if (*edit).cursor < (*edit).scroll {
            (*edit).scroll -= 1;
        }
        return;
    }

    if key == A_HOME || (keynames[key as usize].lower as c_char as i32 == 'a' as i32 && kg.keys[A_CTRL as usize].down) {
        (*edit).cursor = 0;
        return;
    }

    if key == A_END || (keynames[key as usize].lower as c_char as i32 == 'e' as i32 && kg.keys[A_CTRL as usize].down) {
        (*edit).cursor = len;
        return;
    }

    if key == A_INSERT {
        kg.key_overstrikeMode = !kg.key_overstrikeMode;
        return;
    }
}

// ==================
// Field_CharEvent
// ==================
pub unsafe fn Field_CharEvent(edit: *mut field_t, ch: u32) {
    let mut len: c_int;

    if ch == ('v' as u32 - 'a' as u32 + 1) {  // ctrl-v is paste
        Field_Paste(edit);
        return;
    }

    if ch == ('c' as u32 - 'a' as u32 + 1) {  // ctrl-c clears the field
        Field_Clear(edit);
        return;
    }

    len = strlen((*edit).buffer.as_ptr() as *const c_char) as c_int;

    if ch == ('h' as u32 - 'a' as u32 + 1) {  // ctrl-h is backspace
        if (*edit).cursor > 0 {
            memmove(
                ((*edit).buffer.as_mut_ptr() as *mut c_char).offset(((*edit).cursor - 1) as isize) as *mut c_void,
                ((*edit).buffer.as_ptr() as *const c_char).offset((*edit).cursor as isize) as *const c_void,
                (len + 1 - (*edit).cursor) as usize,
            );
            (*edit).cursor -= 1;
            if (*edit).cursor < (*edit).scroll {
                (*edit).scroll -= 1;
            }
        }
        return;
    }

    if ch == ('a' as u32 - 'a' as u32 + 1) {  // ctrl-a is home
        (*edit).cursor = 0;
        (*edit).scroll = 0;
        return;
    }

    if ch == ('e' as u32 - 'a' as u32 + 1) {  // ctrl-e is end
        (*edit).cursor = len;
        (*edit).scroll = (*edit).cursor - (*edit).widthInChars;
        return;
    }

    // ignore any other non printable chars
    if ch < 32 {
        return;
    }

    if kg.key_overstrikeMode {
        if (*edit).cursor == MAX_EDIT_LINE as c_int - 1 {
            return;
        }
        (*edit).buffer[(*edit).cursor as usize] = ch as c_char;
        (*edit).cursor += 1;
    } else {  // insert mode
        if len == MAX_EDIT_LINE as c_int - 1 {
            return;  // all full
        }
        memmove(
            ((*edit).buffer.as_mut_ptr() as *mut c_char).offset(((*edit).cursor + 1) as isize) as *mut c_void,
            ((*edit).buffer.as_ptr() as *const c_char).offset((*edit).cursor as isize) as *const c_void,
            (len + 1 - (*edit).cursor) as usize,
        );
        (*edit).buffer[(*edit).cursor as usize] = ch as c_char;
        (*edit).cursor += 1;
    }

    if (*edit).cursor >= (*edit).widthInChars {
        (*edit).scroll += 1;
    }

    if (*edit).cursor == len + 1 {
        (*edit).buffer[(*edit).cursor as usize] = 0;
    }
}

// ==================
// Field_Clear
// ==================
pub unsafe fn Field_Clear(edit: *mut field_t) {
    (*edit).buffer[0] = 0;
    (*edit).cursor = 0;
    (*edit).scroll = 0;
}

// =============================================================================
//
// CONSOLE LINE EDITING
//
// ==============================================================================

extern "C" {
    fn Field_CharEvent_Forward(edit: *mut field_t, ch: u32);
}

// ===============
// CompleteCommand
//
// Tab expansion
// ===============
unsafe fn CompleteCommand() {
    let mut cmd: *const c_char;
    let mut edit: *mut field_t;
    static mut checking_cmd: bool = false;

    edit = &mut kg.g_consoleField;

    if key_wastab {
        if checking_cmd {
            cmd = Cmd_CompleteCommandNext(keymatch_part.as_ptr(), keymatch_last.as_ptr());
            if cmd.is_null() {
                checking_cmd = false;  // go to checking variables
                cmd = Cvar_CompleteVariableNext(keymatch_part.as_ptr(), std::ptr::null());
                if cmd.is_null() {
                    checking_cmd = true;  // go back to checking cmd
                    cmd = Cmd_CompleteCommandNext(keymatch_part.as_ptr(), std::ptr::null());
                }
            }
        } else {
            cmd = Cvar_CompleteVariableNext(keymatch_part.as_ptr(), keymatch_last.as_ptr());
            if cmd.is_null() {
                checking_cmd = true;  // go back to checking cmd
                cmd = Cmd_CompleteCommandNext(keymatch_part.as_ptr(), std::ptr::null());
                if cmd.is_null() {
                    checking_cmd = false;  // go to checking variables
                    cmd = Cvar_CompleteVariableNext(keymatch_part.as_ptr(), std::ptr::null());
                }
            }
        }
        if !cmd.is_null() {
            strcpy(keymatch_last.as_mut_ptr(), cmd);
            key_wastab = true;
            Com_sprintf((*edit).buffer.as_mut_ptr(), 256, b"%s \0".as_ptr() as *const c_char, cmd);
            (*edit).cursor = strlen((*edit).buffer.as_ptr() as *const c_char) as c_int;
            return;
        }
    } else {
        strcpy(keymatch_part.as_mut_ptr(), (*edit).buffer.as_ptr() as *const c_char);
        checking_cmd = true;
        cmd = Cmd_CompleteCommand((*edit).buffer.as_ptr() as *const c_char);
        if cmd.is_null() {  // means no cmds, so check cvars only
            checking_cmd = false;
            cmd = Cvar_CompleteVariable((*edit).buffer.as_ptr() as *const c_char);
        }
        if !cmd.is_null() {
            Com_sprintf((*edit).buffer.as_mut_ptr(), 256, b"%s \0".as_ptr() as *const c_char, cmd);
            (*edit).cursor = strlen((*edit).buffer.as_ptr() as *const c_char) as c_int;
            strcpy(keymatch_last.as_mut_ptr(), cmd);
            key_wastab = true;
            return;
        }
    }
}

// ====================
// Console_Key
//
// Handles history and console scrollback
// ====================
pub unsafe fn Console_Key(key: c_int) {
    // ctrl-L clears screen
    if keynames[key as usize].lower as c_char as i32 == 'l' as i32 && kg.keys[A_CTRL as usize].down {
        Cbuf_AddText(b"clear\n\0".as_ptr() as *const c_char);
        key_wastab = false;  // For double tabbing on a cvar
        return;
    }

    // extern qboolean SwallowBadNumLockedKPKey( int iKey );
    // if (SwallowBadNumLockedKPKey(key)){
    //     return;
    // }

    // enter finishes the line
    if key == A_ENTER || key == A_KP_ENTER {
        Cbuf_AddText(kg.g_consoleField.buffer.as_ptr() as *const c_char);  // valid command
        Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
        Com_Printf(b"]\%s\n\0".as_ptr() as *const c_char, kg.g_consoleField.buffer.as_ptr() as *const c_char);

        // copy line to history buffer
        kg.historyEditLines[(kg.nextHistoryLine % COMMAND_HISTORY as c_int) as usize] = kg.g_consoleField;
        kg.nextHistoryLine += 1;
        kg.historyLine = kg.nextHistoryLine;

        Field_Clear(&mut kg.g_consoleField);
        kg.g_consoleField.widthInChars = g_console_field_width;
        if cls.state == CA_DISCONNECTED {
            SCR_UpdateScreen();  // force an update, because the command
        }  // may take some time
        key_wastab = false;  // For double tabbing on a cvar
        return;
    }

    // command completion
    if key == A_TAB {
        CompleteCommand();
        return;
    }

    key_wastab = false;  // For double tabbing on a cvar

    // command history (ctrl-p ctrl-n for unix style)
    if (key == A_CURSOR_UP) || ((keynames[key as usize].lower as c_char as i32 == 'p' as i32) && kg.keys[A_CTRL as usize].down) {
        if kg.nextHistoryLine - kg.historyLine < COMMAND_HISTORY as c_int && kg.historyLine > 0 {
            kg.historyLine -= 1;
        }
        kg.g_consoleField = kg.historyEditLines[(kg.historyLine % COMMAND_HISTORY as c_int) as usize];
        return;
    }

    if (key == A_CURSOR_DOWN) || ((keynames[key as usize].lower as c_char as i32 == 'n' as i32) && kg.keys[A_CTRL as usize].down) {
        if kg.historyLine == kg.nextHistoryLine {
            return;
        }
        kg.historyLine += 1;
        kg.g_consoleField = kg.historyEditLines[(kg.historyLine % COMMAND_HISTORY as c_int) as usize];
        return;
    }

    // console scrolling
    if key == A_PAGE_UP {
        Con_PageUp();
        return;
    }

    if key == A_PAGE_DOWN {
        Con_PageDown();
        return;
    }

    // ctrl-home = top of console
    if key == A_HOME && kg.keys[A_CTRL as usize].down {
        Con_Top();
        return;
    }

    // ctrl-end = bottom of console
    if key == A_END && kg.keys[A_CTRL as usize].down {
        Con_Bottom();
        return;
    }

    // pass to the normal editline routine
    Field_KeyDownEvent(&mut kg.g_consoleField, key);
}

// ============================================================================

// ================
// Message_Key
//
// In game talk message
// ================
pub unsafe fn Message_Key(key: c_int) {
    let mut buffer: [c_char; 4096];

    if key == A_ESCAPE {
        cls.keyCatchers &= !KEYCATCH_MESSAGE;
        Field_Clear(&mut chatField);
        return;
    }

    if key == A_ENTER || key == A_KP_ENTER {
        if chatField.buffer[0] != 0 && cls.state == CA_ACTIVE {
            Com_sprintf(buffer.as_mut_ptr(), 4096, b"say \"%s\"\n\0".as_ptr() as *const c_char, chatField.buffer.as_ptr());
            CL_AddReliableCommand(buffer.as_ptr());
        }
        cls.keyCatchers &= !KEYCATCH_MESSAGE;
        Field_Clear(&mut chatField);
        return;
    }

    Field_KeyDownEvent(&mut chatField, key);
}

// ============================================================================

pub fn Key_GetOverstrikeMode() -> bool {
    unsafe { kg.key_overstrikeMode }
}

pub fn Key_SetOverstrikeMode(state: bool) {
    unsafe {
        kg.key_overstrikeMode = state;
    }
}

// ===================
// Key_IsDown
// ===================
pub fn Key_IsDown(keynum: c_int) -> bool {
    if keynum == -1 {
        return false;
    }

    unsafe { kg.keys[keynames[keynum as usize].upper as usize].down }
}

// ===================
// Key_StringToKeynum
//
// Returns a key number to be used to index keys[] by looking at
// the given string.  Single ascii characters return themselves, while
// the K_* names are matched up.
//
// 0x11 will be interpreted as raw hex, which will allow new controlers
// to be configured even if they don't have defined names.
// ===================
pub unsafe fn Key_StringToKeynum(str: *const c_char) -> c_int {
    let mut i: c_int;

    if str.is_null() || *str == 0 {
        return -1;
    }
    // If single char bind, presume ascii char bind
    if *str.offset(1) == 0 {
        return keynames[(*str as u8) as usize].upper as c_int;
    }

    // scan for a text match
    i = 0;
    while i < MAX_KEYS as c_int {
        if !keynames[i as usize].name.is_null() && stricmp(str, keynames[i as usize].name) == 0 {
            return keynames[i as usize].keynum;
        }
        i += 1;
    }

    // check for hex code
    if *str as u8 == b'0' && *str.offset(1) as u8 == b'x' && strlen(str) == 4 {
        let mut n1: c_int;
        let mut n2: c_int;

        n1 = *str.offset(2) as c_int;
        if n1 >= '0' as c_int && n1 <= '9' as c_int {
            n1 -= '0' as c_int;
        } else if n1 >= 'A' as c_int && n1 <= 'F' as c_int {
            n1 = n1 - 'A' as c_int + 10;
        } else {
            n1 = 0;
        }

        n2 = *str.offset(3) as c_int;
        if n2 >= '0' as c_int && n2 <= '9' as c_int {
            n2 -= '0' as c_int;
        } else if n2 >= 'A' as c_int && n2 <= 'F' as c_int {
            n2 = n2 - 'A' as c_int + 10;
        } else {
            n2 = 0;
        }
        return n1 * 16 + n2;
    }

    return -1;
}

static mut tinyString: [c_char; 16] = [0; 16];

unsafe fn Key_KeynumValid(keynum: c_int) -> *const c_char {
    if keynum == -1 {
        return b"<KEY NOT FOUND>\0".as_ptr() as *const c_char;
    }
    if keynum < 0 || keynum >= MAX_KEYS as c_int {
        return b"<OUT OF RANGE>\0".as_ptr() as *const c_char;
    }
    std::ptr::null()
}

unsafe fn Key_KeyToName(keynum: c_int) -> *const c_char {
    keynames[keynum as usize].name
}

unsafe fn Key_KeyToAscii(keynum: c_int) -> *const c_char {
    if keynames[keynum as usize].lower == 0 {
        return std::ptr::null();
    }
    if keynum == A_SPACE {
        tinyString[0] = A_SHIFT_SPACE as c_char;
    } else if keynum == A_ENTER {
        tinyString[0] = A_SHIFT_ENTER as c_char;
    } else if keynum == A_KP_ENTER {
        tinyString[0] = A_SHIFT_KP_ENTER as c_char;
    } else {
        tinyString[0] = keynames[keynum as usize].upper as c_char;
    }
    tinyString[1] = 0;
    tinyString.as_ptr()
}

unsafe fn Key_KeyToHex(keynum: c_int) -> *const c_char {
    let mut i: c_int;
    let mut j: c_int;

    i = keynum >> 4;
    j = keynum & 15;

    tinyString[0] = '0' as c_char;
    tinyString[1] = 'x' as c_char;
    tinyString[2] = if i > 9 { (i - 10 + 'A' as c_int) as c_char } else { (i + '0' as c_int) as c_char };
    tinyString[3] = if j > 9 { (j - 10 + 'A' as c_int) as c_char } else { (j + '0' as c_int) as c_char };
    tinyString[4] = 0;

    tinyString.as_ptr()
}

// Returns the ascii code of the keynum
pub unsafe fn Key_KeynumToAscii(keynum: c_int) -> *const c_char {
    let mut name: *const c_char;

    name = Key_KeynumValid(keynum);

    // check for printable ascii
    if name.is_null() && keynum > 0 && keynum < 256 {
        name = Key_KeyToAscii(keynum);
    }
    // Check for name (for JOYx and AUXx buttons)
    if name.is_null() {
        name = Key_KeyToName(keynum);
    }
    // Fallback to hex number
    if name.is_null() {
        name = Key_KeyToHex(keynum);
    }
    name
}

// ===================
// Key_KeynumToString
//
// Returns a string (either a single ascii char, a K_* name, or a 0x11 hex string) for the
// given keynum.
// ===================
// Returns a console/config file friendly name for the key
pub unsafe fn Key_KeynumToString(keynum: c_int) -> *const c_char {
    let mut name: *const c_char;

    name = Key_KeynumValid(keynum);

    // Check for friendly name
    if name.is_null() {
        name = Key_KeyToName(keynum);
    }
    // check for printable ascii
    if name.is_null() && keynum > 0 && keynum < 256 {
        name = Key_KeyToAscii(keynum);
    }
    // Fallback to hex number
    if name.is_null() {
        name = Key_KeyToHex(keynum);
    }
    name
}

// ===================
// Key_SetBinding
// ===================
pub unsafe fn Key_SetBinding(keynum: c_int, binding: *const c_char) {
    if keynum == -1 {
        return;
    }

    // free old bindings
    if !kg.keys[keynames[keynum as usize].upper as usize].binding.is_null() {
        Z_Free(kg.keys[keynames[keynum as usize].upper as usize].binding as *mut c_void);
        kg.keys[keynames[keynum as usize].upper as usize].binding = std::ptr::null_mut();
    }

    // allocate memory for new binding
    if !binding.is_null() {
        kg.keys[keynames[keynum as usize].upper as usize].binding = CopyString(binding);
    }

    // consider this like modifying an archived cvar, so the
    // file write will be triggered at the next oportunity
    cvar_modifiedFlags |= CVAR_ARCHIVE;
}

// ===================
// Key_GetBinding
// ===================
pub unsafe fn Key_GetBinding(keynum: c_int) -> *const c_char {
    if keynum == -1 {
        return b"\0".as_ptr() as *const c_char;
    }

    let binding = kg.keys[keynum as usize].binding;
    if binding.is_null() {
        b"\0".as_ptr() as *const c_char
    } else {
        binding as *const c_char
    }
}

// ===================
// Key_Unbind_f
// ===================
pub unsafe fn Key_Unbind_f() {
    let mut b: c_int;

    if Cmd_Argc() != 2 {
        Com_Printf(b"unbind <key> : remove commands from a key\n\0".as_ptr() as *const c_char);
        return;
    }

    b = Key_StringToKeynum(Cmd_Argv(1));
    if b == -1 {
        Com_Printf(b"\"%s\" isn't a valid key\n\0".as_ptr() as *const c_char, Cmd_Argv(1));
        return;
    }

    Key_SetBinding(b, b"\0".as_ptr() as *const c_char);
}

// ===================
// Key_Unbindall_f
// ===================
pub unsafe fn Key_Unbindall_f() {
    let mut i: c_int;

    i = 0;
    while i < MAX_KEYS as c_int {
        if !kg.keys[i as usize].binding.is_null() {
            Key_SetBinding(i, b"\0".as_ptr() as *const c_char);
        }
        i += 1;
    }
}

// ===================
// Key_Bind_f
// ===================
pub unsafe fn Key_Bind_f() {
    let mut i: c_int;
    let mut c: c_int;
    let mut b: c_int;
    let mut cmd: [c_char; 1024] = [0; 1024];

    c = Cmd_Argc();

    if c < 2 {
        Com_Printf(b"bind <key> [command] : attach a command to a key\n\0".as_ptr() as *const c_char);
        return;
    }
    b = Key_StringToKeynum(Cmd_Argv(1));
    if b == -1 {
        Com_Printf(b"\"%s\" isn't a valid key\n\0".as_ptr() as *const c_char, Cmd_Argv(1));
        return;
    }

    if c == 2 {
        if !kg.keys[b as usize].binding.is_null() {
            Com_Printf(b"\"%s\" = \"%s\"\n\0".as_ptr() as *const c_char, Cmd_Argv(1), kg.keys[b as usize].binding);
        } else {
            Com_Printf(b"\"%s\" is not bound\n\0".as_ptr() as *const c_char, Cmd_Argv(1));
        }
        return;
    }

    // copy the rest of the command line
    cmd[0] = 0;  // start out with a null string
    i = 2;
    while i < c {
        strcat(cmd.as_mut_ptr(), Cmd_Argv(i));
        if i != (c - 1) {
            strcat(cmd.as_mut_ptr(), b" \0".as_ptr() as *const c_char);
        }
        i += 1;
    }

    Key_SetBinding(b, cmd.as_ptr());
}

// ============
// Key_WriteBindings
//
// Writes lines containing "bind key value"
// ============
pub unsafe fn Key_WriteBindings(f: usize) {
    let mut i: c_int;

    FS_Printf(f, b"unbindall\n\0".as_ptr() as *const c_char);
    i = 0;
    while i < MAX_KEYS as c_int {
        if !kg.keys[i as usize].binding.is_null() && *kg.keys[i as usize].binding != 0 {
            FS_Printf(f, b"bind %s \"%s\"\n\0".as_ptr() as *const c_char, Key_KeynumToString(i), kg.keys[i as usize].binding);
        }
        i += 1;
    }
}

// ============
// Key_Bindlist_f
//
// ============
pub unsafe fn Key_Bindlist_f() {
    let mut i: c_int;

    i = 0;
    while i < MAX_KEYS as c_int {
        if !kg.keys[i as usize].binding.is_null() && *kg.keys[i as usize].binding != 0 {
            Com_Printf(b"Key : %s (%s) \"%s\"\n\0".as_ptr() as *const c_char, Key_KeynumToAscii(i), Key_KeynumToString(i), kg.keys[i as usize].binding);
        }
        i += 1;
    }
}

// ===================
// CL_InitKeyCommands
// ===================
pub unsafe fn CL_InitKeyCommands() {
    // register our functions
    Cmd_AddCommand(b"bind\0".as_ptr() as *const c_char, Key_Bind_f as *const c_void);
    Cmd_AddCommand(b"unbind\0".as_ptr() as *const c_char, Key_Unbind_f as *const c_void);
    Cmd_AddCommand(b"unbindall\0".as_ptr() as *const c_char, Key_Unbindall_f as *const c_void);
    Cmd_AddCommand(b"bindlist\0".as_ptr() as *const c_char, Key_Bindlist_f as *const c_void);
}

pub unsafe fn CL_ActionEvent(key: c_int, down: bool, time: u32) {
    let mut kb: *const c_char;
    let mut cmd: [c_char; 1024] = [0; 1024];

    // send the bound action
    kb = kg.keys[keynames[key as usize].upper as usize].binding;
    if !kb.is_null() {
        if *kb as u8 == b'+' {
            // button commands add keynum and time as parms so that multiple
            // sources can be discriminated and subframe corrected
            Com_sprintf(cmd.as_mut_ptr(), 1024, b"%s %i %i\n\0".as_ptr() as *const c_char, kb, key, time);
            Cbuf_AddText(cmd.as_ptr());
        } else {
            // down-only command
            Cbuf_AddText(kb);
            Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
        }
    }
}

// ===================
// CL_KeyEvent
//
// Called by the system for both key up and key down events
// ===================
pub unsafe fn CL_KeyEvent(key: c_int, down: bool, time: u32) {
    let mut kb: *const c_char;
    let mut cmd: [c_char; 1024] = [0; 1024];

    // update auto-repeat status and BUTTON_ANY status
    kg.keys[keynames[key as usize].upper as usize].down = down;
    if down {
        kg.keys[keynames[key as usize].upper as usize].repeats += 1;
        if kg.keys[keynames[key as usize].upper as usize].repeats == 1 {
            kg.anykeydown = true;
            kg.keyDownCount += 1;
        }
    } else {
        kg.keys[keynames[key as usize].upper as usize].repeats = 0;
        kg.keyDownCount -= 1;
        if kg.keyDownCount <= 0 {
            kg.anykeydown = false;
            kg.keyDownCount = 0;
        }
    }

    // console key is hardcoded, so the user can never unbind it
    if key == A_CONSOLE {
        if !down {
            return;
        }

        // #ifdef FINAL_BUILD
        //     if (!(cls.keyCatchers & KEYCATCH_CONSOLE) && !kg.keys[A_SHIFT].down) {
        //         return;
        //     }
        // #endif

        Con_ToggleConsole_f();
        return;
    }

    // most keys during demo playback will bring up the menu, but non-ascii
    // keys can still be used for bound actions
    if down && (cls.state == CA_CINEMATIC || CL_IsRunningInGameCinematic()) && cls.keyCatchers == 0 {
        SCR_StopCinematic(true);
        return;
        // Cvar_Set ("nextdemo","");
        // key = K_ESCAPE;
    }

    // escape is always handled special
    if key == A_ESCAPE && down {
        if (cls.keyCatchers & KEYCATCH_MESSAGE) != 0 {
            // clear message mode
            Message_Key(key);
            return;
        }

        if (cls.keyCatchers & KEYCATCH_UI) == 0 {
            if cls.state == CA_ACTIVE {
                UI_SetActiveMenu(b"ingame\0".as_ptr() as *const c_char, std::ptr::null());
            } else {
                CL_Disconnect_f();
                UI_SetActiveMenu(b"mainMenu\0".as_ptr() as *const c_char, std::ptr::null());
            }
            return;
        }

        _UI_KeyEvent(key, down);
        return;
    }

    // key up events only perform actions if the game key binding is
    // a button command (leading + sign).  These will be processed even in
    // console mode and menu mode, to keep the character from continuing
    // an action started before a mode switch.
    if !down {
        kb = kg.keys[keynames[key as usize].upper as usize].binding;
        if !kb.is_null() && *kb as u8 == b'+' {
            // button commands add keynum and time as parms so that multiple
            // sources can be discriminated and subframe corrected
            Com_sprintf(cmd.as_mut_ptr(), 1024, b"-%s %i %i\n\0".as_ptr() as *const c_char, kb.offset(1), key, time);
            Cbuf_AddText(cmd.as_ptr());
        }
        if (cls.keyCatchers & KEYCATCH_UI) != 0 {
            // need UP messages to clear out captures!
            _UI_KeyEvent(key, down);
        }
        return;
    }

    // distribute the key down event to the apropriate handler
    if (cls.keyCatchers & KEYCATCH_CONSOLE) != 0 {
        Console_Key(key);
    } else if (cls.keyCatchers & KEYCATCH_UI) != 0 {
        _UI_KeyEvent(key, down);
    } else if (cls.keyCatchers & KEYCATCH_MESSAGE) != 0 {
        Message_Key(key);
    } else if cls.state == CA_DISCONNECTED {
        Console_Key(key);
    } else {
        CL_ActionEvent(key, true, time);
    }
}

// ===================
// CL_CharEvent
//
// Normal keyboard characters, already shifted / capslocked / etc
// ===================
pub unsafe fn CL_CharEvent(key: c_int) {
    // the console key should never be used as a char
    if key == '`' as c_int || key == '~' as c_int {
        return;
    }

    // distribute the key down event to the apropriate handler
    if (cls.keyCatchers & KEYCATCH_CONSOLE) != 0 {
        Field_CharEvent(&mut kg.g_consoleField, key as u32);
    } else if (cls.keyCatchers & KEYCATCH_UI) != 0 {
        _UI_KeyEvent(key | K_CHAR_FLAG, true);
    } else if (cls.keyCatchers & KEYCATCH_MESSAGE) != 0 {
        Field_CharEvent(&mut chatField, key as u32);
    } else if cls.state == CA_DISCONNECTED {
        Field_CharEvent(&mut kg.g_consoleField, key as u32);
    }
}

// ===================
// Key_ClearStates
// ===================
pub unsafe fn Key_ClearStates() {
    let mut i: c_int;

    kg.anykeydown = false;
    kg.keyDownCount = 0;

    i = 0;
    while i < MAX_KEYS as c_int {
        if kg.keys[i as usize].down {
            CL_KeyEvent(i, false, 0);
        }
        kg.keys[i as usize].down = false;
        kg.keys[i as usize].repeats = 0;
        i += 1;
    }
}
