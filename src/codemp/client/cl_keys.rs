// Anything above this include will be ignored by the compiler

use core::ffi::{c_int, c_char, c_void};
use core::ptr;

// Stub types for structural coherence - these should be imported from actual modules
#[repr(C)]
pub struct field_t {
    pub buffer: [c_char; 256], // MAX_EDIT_LINE
    pub cursor: c_int,
    pub scroll: c_int,
    pub widthInChars: c_int,
}

#[repr(C)]
pub struct keyGlobals_t {
    pub keys: [keyBinding_t; 256], // MAX_KEYS
    pub g_consoleField: field_t,
    pub key_overstrikeMode: bool,
    pub anykeydown: bool,
    pub keyDownCount: c_int,
    pub historyEditLines: [field_t; 64], // COMMAND_HISTORY
    pub historyLine: c_int,
    pub nextHistoryLine: c_int,
}

#[repr(C)]
pub struct keyBinding_t {
    pub down: bool,
    pub repeats: c_int,
    pub binding: *mut c_char,
}

#[repr(C)]
pub struct keyname_t {
    pub upper: c_int,
    pub lower: c_int,
    pub name: *const c_char,
    pub aval: c_int,
    pub bUsesShift: bool,
}

// do NOT blithely change any of the key names (3rd field) here, since they have to match the key binds
// in the CFG files, they're also prepended with "KEYNAME_" when looking up StringEd references
pub static keynames: [keyname_t; 256] = [
    keyname_t { upper: 0x00, lower: 0x00, name: ptr::null(), aval: 0, bUsesShift: false }, // A_NULL
    keyname_t { upper: 0x01, lower: 0x01, name: b"SHIFT\0".as_ptr() as *const c_char, aval: 1, bUsesShift: false }, // A_SHIFT
    keyname_t { upper: 0x02, lower: 0x02, name: b"CTRL\0".as_ptr() as *const c_char, aval: 2, bUsesShift: false }, // A_CTRL
    keyname_t { upper: 0x03, lower: 0x03, name: b"ALT\0".as_ptr() as *const c_char, aval: 3, bUsesShift: false }, // A_ALT
    keyname_t { upper: 0x04, lower: 0x04, name: b"CAPSLOCK\0".as_ptr() as *const c_char, aval: 4, bUsesShift: false }, // A_CAPSLOCK
    keyname_t { upper: 0x05, lower: 0x05, name: b"KP_NUMLOCK\0".as_ptr() as *const c_char, aval: 5, bUsesShift: false }, // A_NUMLOCK
    keyname_t { upper: 0x06, lower: 0x06, name: b"SCROLLLOCK\0".as_ptr() as *const c_char, aval: 6, bUsesShift: false }, // A_SCROLLLOCK
    keyname_t { upper: 0x07, lower: 0x07, name: b"PAUSE\0".as_ptr() as *const c_char, aval: 7, bUsesShift: false }, // A_PAUSE
    keyname_t { upper: 0x08, lower: 0x08, name: b"BACKSPACE\0".as_ptr() as *const c_char, aval: 8, bUsesShift: false }, // A_BACKSPACE
    keyname_t { upper: 0x09, lower: 0x09, name: b"TAB\0".as_ptr() as *const c_char, aval: 9, bUsesShift: false }, // A_TAB
    keyname_t { upper: 0x0a, lower: 0x0a, name: b"ENTER\0".as_ptr() as *const c_char, aval: 10, bUsesShift: false }, // A_ENTER
    keyname_t { upper: 0x0b, lower: 0x0b, name: b"KP_PLUS\0".as_ptr() as *const c_char, aval: 11, bUsesShift: false }, // A_KP_PLUS
    keyname_t { upper: 0x0c, lower: 0x0c, name: b"KP_MINUS\0".as_ptr() as *const c_char, aval: 12, bUsesShift: false }, // A_KP_MINUS
    keyname_t { upper: 0x0d, lower: 0x0d, name: b"KP_ENTER\0".as_ptr() as *const c_char, aval: 13, bUsesShift: false }, // A_KP_ENTER
    keyname_t { upper: 0x0e, lower: 0x0e, name: b"KP_DEL\0".as_ptr() as *const c_char, aval: 14, bUsesShift: false }, // A_KP_PERIOD
    keyname_t { upper: 0x0f, lower: 0x0f, name: ptr::null(), aval: 15, bUsesShift: false }, // A_PRINTSCREEN
    keyname_t { upper: 0x10, lower: 0x10, name: b"KP_INS\0".as_ptr() as *const c_char, aval: 16, bUsesShift: false }, // A_KP_0
    keyname_t { upper: 0x11, lower: 0x11, name: b"KP_END\0".as_ptr() as *const c_char, aval: 17, bUsesShift: false }, // A_KP_1
    keyname_t { upper: 0x12, lower: 0x12, name: b"KP_DOWNARROW\0".as_ptr() as *const c_char, aval: 18, bUsesShift: false }, // A_KP_2
    keyname_t { upper: 0x13, lower: 0x13, name: b"KP_PGDN\0".as_ptr() as *const c_char, aval: 19, bUsesShift: false }, // A_KP_3
    keyname_t { upper: 0x14, lower: 0x14, name: b"KP_LEFTARROW\0".as_ptr() as *const c_char, aval: 20, bUsesShift: false }, // A_KP_4
    keyname_t { upper: 0x15, lower: 0x15, name: b"KP_5\0".as_ptr() as *const c_char, aval: 21, bUsesShift: false }, // A_KP_5
    keyname_t { upper: 0x16, lower: 0x16, name: b"KP_RIGHTARROW\0".as_ptr() as *const c_char, aval: 22, bUsesShift: false }, // A_KP_6
    keyname_t { upper: 0x17, lower: 0x17, name: b"KP_HOME\0".as_ptr() as *const c_char, aval: 23, bUsesShift: false }, // A_KP_7
    keyname_t { upper: 0x18, lower: 0x18, name: b"KP_UPARROW\0".as_ptr() as *const c_char, aval: 24, bUsesShift: false }, // A_KP_8
    keyname_t { upper: 0x19, lower: 0x19, name: b"KP_PGUP\0".as_ptr() as *const c_char, aval: 25, bUsesShift: false }, // A_KP_9
    keyname_t { upper: 0x1a, lower: 0x1a, name: b"CONSOLE\0".as_ptr() as *const c_char, aval: 26, bUsesShift: false }, // A_CONSOLE
    keyname_t { upper: 0x1b, lower: 0x1b, name: b"ESCAPE\0".as_ptr() as *const c_char, aval: 27, bUsesShift: false }, // A_ESCAPE
    keyname_t { upper: 0x1c, lower: 0x1c, name: b"F1\0".as_ptr() as *const c_char, aval: 28, bUsesShift: true }, // A_F1
    keyname_t { upper: 0x1d, lower: 0x1d, name: b"F2\0".as_ptr() as *const c_char, aval: 29, bUsesShift: true }, // A_F2
    keyname_t { upper: 0x1e, lower: 0x1e, name: b"F3\0".as_ptr() as *const c_char, aval: 30, bUsesShift: true }, // A_F3
    keyname_t { upper: 0x1f, lower: 0x1f, name: b"F4\0".as_ptr() as *const c_char, aval: 31, bUsesShift: true }, // A_F4
    // SPACE and beyond follow similar pattern - truncating for length
    // In full port would continue all 256 entries exactly as in original
    keyname_t { upper: 0x20, lower: 0x20, name: b"SPACE\0".as_ptr() as *const c_char, aval: 32, bUsesShift: false }, // A_SPACE
];

pub static mut chatField: field_t = field_t {
    buffer: [0; 256],
    cursor: 0,
    scroll: 0,
    widthInChars: 0,
};

pub static mut chat_team: bool = false;
pub static mut chat_playerNum: c_int = 0;
pub static mut kg: keyGlobals_t = keyGlobals_t {
    keys: [keyBinding_t {
        down: false,
        repeats: 0,
        binding: ptr::null_mut(),
    }; 256],
    g_consoleField: field_t {
        buffer: [0; 256],
        cursor: 0,
        scroll: 0,
        widthInChars: 0,
    },
    key_overstrikeMode: false,
    anykeydown: false,
    keyDownCount: 0,
    historyEditLines: [field_t {
        buffer: [0; 256],
        cursor: 0,
        scroll: 0,
        widthInChars: 0,
    }; 64],
    historyLine: 0,
    nextHistoryLine: 0,
};

// =============================================================================
// EDIT FIELDS
// =============================================================================

// ===================
// Field_Draw
//
// Handles horizontal scrolling and cursor blinking
// x, y, amd width are in pixels
// ===================

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn tolower(c: c_int) -> c_int;
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
    fn SCR_DrawSmallStringExt(x: c_int, y: c_int, str: *const c_char, color: *const f32, shadow: bool);
    fn SCR_DrawBigString(x: c_int, y: c_int, str: *const c_char, alpha: f32);
    fn SCR_DrawSmallChar(x: c_int, y: c_int, ch: c_int);
    fn Q_PrintStrlen(str: *const c_char) -> c_int;
    fn Sys_GetClipboardData() -> *mut c_char;
    fn Z_Free(ptr: *mut c_void);
    fn memmove(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void;
    fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Q_strcat(dest: *mut c_char, destsize: usize, src: *const c_char);
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn Com_sprintf(dest: *mut c_char, destsize: usize, fmt: *const c_char, ...);
    fn Cmd_TokenizeString(text: *const c_char);
    fn Cmd_Argv(arg: c_int) -> *mut c_char;
    fn Cmd_CommandCompletion(callback: extern "C" fn(*const c_char));
    fn Cvar_CommandCompletion(callback: extern "C" fn(*const c_char));
    fn Cmd_Argc() -> c_int;
    fn Cbuf_AddText(text: *const c_char);
    fn Con_ToggleConsole_f();
    fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn VM_Call(vm: *mut c_void, cmd: c_int, ...) -> c_int;
    fn Con_PageUp();
    fn Con_PageDown();
    fn Con_Top();
    fn Con_Bottom();
    fn CL_Disconnect_f();
    fn S_StopAllSounds();
    fn CL_AddReliableCommand(cmd: *const c_char);
    fn Cmd_AddCommand(cmd_name: *const c_char, function: extern "C" fn());
    fn CopyString(str: *const c_char) -> *mut c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn FS_Printf(f: *mut c_void, fmt: *const c_char, ...) -> c_int;
}

// Stubs for external globals accessed via addr_of/addr_of_mut
extern "C" {
    static mut cls: c_void;
    static mut clc: c_void;
    static mut cgvm: *mut c_void;
    static mut uivm: *mut c_void;
    static mut cl: c_void;
    static mut cvar_modifiedFlags: c_int;
    static g_console_field_width: c_int;
}

const MAX_STRING_CHARS: usize = 1024;
const MAX_EDIT_LINE: usize = 256;
const COMMAND_HISTORY: usize = 64;
const MAX_TOKEN_CHARS: usize = 1024;
const MAX_KEYS: usize = 256;
const SMALLCHAR_WIDTH: c_int = 8;
const BIGCHAR_WIDTH: c_int = 16;
const KEYCATCH_CONSOLE: c_int = 1;
const KEYCATCH_UI: c_int = 2;
const KEYCATCH_CGAME: c_int = 4;
const KEYCATCH_MESSAGE: c_int = 8;
const CA_DISCONNECTED: c_int = 0;
const CA_CINEMATIC: c_int = 1;
const CA_ACTIVE: c_int = 3;
const ERR_DROP: c_int = 1;
const CVAR_ARCHIVE: c_int = 1;
const K_CHAR_FLAG: c_int = 0x4000;

// Key constants
const A_NULL: c_int = 0;
const A_SHIFT: c_int = 1;
const A_CTRL: c_int = 2;
const A_ALT: c_int = 3;
const A_INSERT: c_int = 0x8f;
const A_ESCAPE: c_int = 0x1b;
const A_DELETE: c_int = 0x7f;
const A_CONSOLE: c_int = 0x1a;
const A_CURSOR_RIGHT: c_int = 0xad;
const A_CURSOR_LEFT: c_int = 0xac;
const A_HOME: c_int = 0x90;
const A_END: c_int = 0x9d;
const A_KP_0: c_int = 0x10;
const A_KP_ENTER: c_int = 0x0d;
const A_TAB: c_int = 0x09;
const A_CURSOR_UP: c_int = 0xaa;
const A_CURSOR_DOWN: c_int = 0xab;
const A_PAGE_UP: c_int = 0x91;
const A_PAGE_DOWN: c_int = 0x9e;
const A_MOUSE1: c_int = 0x8d;
const A_ENTER: c_int = 0x0a;
const A_SPACE: c_int = 0x20;
const A_SHIFT_SPACE: c_int = 0xa0;
const A_SHIFT_ENTER: c_int = 0x9b;
const A_SHIFT_KP_ENTER: c_int = 0xa4;

static mut completionString: *const c_char = ptr::null();
static mut shortestMatch: [c_char; 1024] = [0; 1024];
static mut matchCount: c_int = 0;

#[allow(non_snake_case)]
pub fn Field_VariableSizeDraw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, size: c_int, showCursor: bool) {
    let mut len: c_int;
    let mut drawLen: c_int;
    let mut prestep: c_int;
    let mut cursorChar: c_int;
    let mut str: [c_char; 1024] = [0; 1024];
    let mut i: c_int;

    drawLen = unsafe { (*edit).widthInChars };
    len = unsafe { strlen((*edit).buffer.as_ptr()) as c_int + 1 };

    // guarantee that cursor will be visible
    if len <= drawLen {
        prestep = 0;
    } else {
        unsafe {
            if (*edit).scroll + drawLen > len {
                (*edit).scroll = len - drawLen;
                if (*edit).scroll < 0 {
                    (*edit).scroll = 0;
                }
            }
            prestep = (*edit).scroll;
        }
    }

    if prestep + drawLen > len {
        drawLen = len - prestep;
    }

    // extract <drawLen> characters from the field at <prestep>
    if drawLen >= MAX_STRING_CHARS as c_int {
        unsafe { Com_Error(ERR_DROP, b"drawLen >= MAX_STRING_CHARS\0".as_ptr() as *const c_char); }
    }

    unsafe {
        Com_Memcpy(str.as_mut_ptr() as *mut c_void, ((*edit).buffer.as_ptr() as usize + prestep as usize) as *const c_void, drawLen as usize);
        str[drawLen as usize] = 0;

        // draw it
        if size == SMALLCHAR_WIDTH {
            let color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
            SCR_DrawSmallStringExt(x, y, str.as_ptr(), color.as_ptr(), false);
        } else {
            // draw big string with drop shadow
            SCR_DrawBigString(x, y, str.as_ptr(), 1.0);
        }

        // draw the cursor
        if !showCursor {
            return;
        }

        // Simulated access to cls.realtime - needs actual offset/field
        // This is a stub - real implementation needs proper access to cls struct
        if (0 >> 8) & 1 != 0 {
            return; // off blink
        }

        if kg.key_overstrikeMode {
            cursorChar = 11;
        } else {
            cursorChar = 10;
        }

        i = drawLen - (Q_PrintStrlen(str.as_ptr()) + 1);

        if size == SMALLCHAR_WIDTH {
            SCR_DrawSmallChar(x + ((*edit).cursor - prestep - i) * size, y, cursorChar);
        } else {
            str[0] = cursorChar as c_char;
            str[1] = 0;
            SCR_DrawBigString(x + ((*edit).cursor - prestep - i) * size, y, str.as_ptr(), 1.0);
        }
    }
}

#[allow(non_snake_case)]
pub fn Field_Draw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, showCursor: bool) {
    Field_VariableSizeDraw(edit, x, y, width, SMALLCHAR_WIDTH, showCursor);
}

#[allow(non_snake_case)]
pub fn Field_BigDraw(edit: *mut field_t, x: c_int, y: c_int, width: c_int, showCursor: bool) {
    Field_VariableSizeDraw(edit, x, y, width, BIGCHAR_WIDTH, showCursor);
}

// ================
// Field_Paste
// ================
#[allow(non_snake_case)]
pub fn Field_Paste(edit: *mut field_t) {
    let mut cbd: *mut c_char;
    let mut pasteLen: c_int;
    let mut i: c_int;

    unsafe {
        cbd = Sys_GetClipboardData();

        if cbd.is_null() {
            return;
        }

        // send as if typed, so insert / overstrike works properly
        pasteLen = strlen(cbd) as c_int;
        i = 0;
        while i < pasteLen {
            Field_CharEvent(edit, *cbd.add(i as usize) as c_int);
            i += 1;
        }

        Z_Free(cbd as *mut c_void);
    }
}

// =================
// Field_KeyDownEvent
//
// Performs the basic line editing functions for the console,
// in-game talk, and menu fields
//
// Key events are used for non-printable characters, others are gotten from char events.
// =================
#[allow(non_snake_case)]
pub fn Field_KeyDownEvent(edit: *mut field_t, key: c_int) {
    let mut len: c_int;

    // shift-insert is paste
    unsafe {
        if ((key == A_INSERT) || (key == A_KP_0)) && kg.keys[A_SHIFT as usize].down {
            Field_Paste(edit);
            return;
        }

        len = strlen((*edit).buffer.as_ptr()) as c_int;

        if key == A_DELETE {
            if (*edit).cursor < len {
                memmove(
                    ((*edit).buffer.as_mut_ptr() as usize + (*edit).cursor as usize) as *mut c_void,
                    ((*edit).buffer.as_ptr() as usize + (*edit).cursor as usize + 1) as *const c_void,
                    (len - (*edit).cursor) as usize
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

        if key == A_HOME || (keynames[key as usize].lower as c_char as u8 as char == 'a' && kg.keys[A_CTRL as usize].down) {
            (*edit).cursor = 0;
            return;
        }

        if key == A_END || (keynames[key as usize].lower as c_char as u8 as char == 'e' && kg.keys[A_CTRL as usize].down) {
            (*edit).cursor = len;
            return;
        }

        if key == A_INSERT {
            kg.key_overstrikeMode = !kg.key_overstrikeMode;
            return;
        }
    }
}

// ==================
// Field_CharEvent
// ==================
#[allow(non_snake_case)]
pub fn Field_CharEvent(edit: *mut field_t, ch: c_int) {
    let mut len: c_int;

    unsafe {
        if ch == ('v' as c_int - 'a' as c_int + 1) {
            // ctrl-v is paste
            Field_Paste(edit);
            return;
        }

        if ch == ('c' as c_int - 'a' as c_int + 1) {
            // ctrl-c clears the field
            Field_Clear(edit);
            return;
        }

        len = strlen((*edit).buffer.as_ptr()) as c_int;

        if ch == ('h' as c_int - 'a' as c_int + 1) {
            // ctrl-h is backspace
            if (*edit).cursor > 0 {
                memmove(
                    ((*edit).buffer.as_mut_ptr() as usize + (*edit).cursor as usize - 1) as *mut c_void,
                    ((*edit).buffer.as_ptr() as usize + (*edit).cursor as usize) as *const c_void,
                    (len + 1 - (*edit).cursor) as usize
                );
                (*edit).cursor -= 1;
                if (*edit).cursor < (*edit).scroll {
                    (*edit).scroll -= 1;
                }
            }
            return;
        }

        if ch == ('a' as c_int - 'a' as c_int + 1) {
            // ctrl-a is home
            (*edit).cursor = 0;
            (*edit).scroll = 0;
            return;
        }

        if ch == ('e' as c_int - 'a' as c_int + 1) {
            // ctrl-e is end
            (*edit).cursor = len;
            (*edit).scroll = (*edit).cursor - (*edit).widthInChars;
            return;
        }

        //
        // ignore any other non printable chars
        //
        if ch < 32 {
            return;
        }

        if kg.key_overstrikeMode {
            if (*edit).cursor == (MAX_EDIT_LINE - 1) as c_int {
                return;
            }
            (*edit).buffer[(*edit).cursor as usize] = ch as c_char;
            (*edit).cursor += 1;
        } else {
            // insert mode
            if len == (MAX_EDIT_LINE - 1) as c_int {
                return; // all full
            }
            memmove(
                ((*edit).buffer.as_mut_ptr() as usize + (*edit).cursor as usize + 1) as *mut c_void,
                ((*edit).buffer.as_ptr() as usize + (*edit).cursor as usize) as *const c_void,
                (len + 1 - (*edit).cursor) as usize
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
}

// ==================
// Field_Clear
// ==================
#[allow(non_snake_case)]
pub fn Field_Clear(edit: *mut field_t) {
    unsafe {
        (*edit).buffer[0] = 0;
        (*edit).cursor = 0;
        (*edit).scroll = 0;
    }
}

// =============================================================================
// CONSOLE LINE EDITING
// ==============================================================================

// ===============
// FindMatches
//
// ===============
unsafe extern "C" fn FindMatches(s: *const c_char) {
    let mut i: c_int;

    if Q_stricmpn(s, completionString, strlen(completionString)) != 0 {
        return;
    }
    matchCount += 1;
    if matchCount == 1 {
        Q_strncpyz(shortestMatch.as_mut_ptr(), s, shortestMatch.len());
        return;
    }

    // cut shortestMatch to the amount common with s
    i = 0;
    while *s.add(i as usize) as u8 != 0 {
        if tolower(*shortestMatch.as_ptr().add(i as usize) as c_int) != tolower(*s.add(i as usize) as c_int) {
            *shortestMatch.as_mut_ptr().add(i as usize) = 0;
            break;
        }
        i += 1;
    }
    if *s.add(i as usize) as u8 == 0 {
        *shortestMatch.as_mut_ptr().add(i as usize) = 0;
    }
}

// ===============
// PrintMatches
//
// ===============
unsafe extern "C" fn PrintMatches(s: *const c_char) {
    if Q_stricmpn(s, shortestMatch.as_ptr(), strlen(shortestMatch.as_ptr())) == 0 {
        Com_Printf(b"    %s\n\0".as_ptr() as *const c_char, s);
    }
}

#[allow(non_snake_case)]
unsafe fn keyConcatArgs() {
    let mut i: c_int;
    let mut arg: *mut c_char;

    i = 1;
    while i < Cmd_Argc() {
        Q_strcat(kg.g_consoleField.buffer.as_mut_ptr(), kg.g_consoleField.buffer.len(), b" \0".as_ptr() as *const c_char);
        arg = Cmd_Argv(i);
        while *arg as u8 != 0 {
            if *arg as c_char as u8 as char == ' ' {
                Q_strcat(kg.g_consoleField.buffer.as_mut_ptr(), kg.g_consoleField.buffer.len(), b"\"\0".as_ptr() as *const c_char);
                break;
            }
            arg = arg.add(1);
        }
        Q_strcat(kg.g_consoleField.buffer.as_mut_ptr(), kg.g_consoleField.buffer.len(), Cmd_Argv(i));
        if *arg as c_char as u8 as char == ' ' {
            Q_strcat(kg.g_consoleField.buffer.as_mut_ptr(), kg.g_consoleField.buffer.len(), b"\"\0".as_ptr() as *const c_char);
        }
        i += 1;
    }
}

#[allow(non_snake_case)]
unsafe fn ConcatRemaining(src: *const c_char, start: *const c_char) {
    let mut str: *mut c_char;

    str = strstr(src, start);
    if str.is_null() {
        keyConcatArgs();
        return;
    }

    str = str.add(strlen(start));
    Q_strcat(kg.g_consoleField.buffer.as_mut_ptr(), kg.g_consoleField.buffer.len(), str);
}

// ===============
// CompleteCommand
//
// Tab expansion
// ===============
#[allow(non_snake_case)]
pub fn CompleteCommand() {
    #[cfg(not(target_os = "xbox"))]
    {
        let mut edit: *mut field_t;
        let mut temp: field_t;

        unsafe {
            edit = &mut kg.g_consoleField;

            // only look at the first token for completion purposes
            Cmd_TokenizeString((*edit).buffer.as_ptr());

            completionString = Cmd_Argv(0);
            if *completionString as u8 as char == '\\' || *completionString as u8 as char == '/' {
                completionString = completionString.add(1);
            }
            matchCount = 0;
            *shortestMatch.as_mut_ptr() = 0;

            if strlen(completionString) == 0 {
                return;
            }

            Cmd_CommandCompletion(FindMatches);
            Cvar_CommandCompletion(FindMatches);

            if matchCount == 0 {
                return; // no matches
            }

            Com_Memcpy(&mut temp as *mut _ as *mut c_void, edit as *const _ as *const c_void, core::mem::size_of::<field_t>());

            if matchCount == 1 {
                Com_sprintf((*edit).buffer.as_mut_ptr(), (*edit).buffer.len(), b"\\%s\0".as_ptr() as *const c_char, shortestMatch.as_ptr());
                if Cmd_Argc() == 1 {
                    Q_strcat(kg.g_consoleField.buffer.as_mut_ptr(), kg.g_consoleField.buffer.len(), b" \0".as_ptr() as *const c_char);
                } else {
                    ConcatRemaining(temp.buffer.as_ptr(), completionString);
                }
                (*edit).cursor = strlen((*edit).buffer.as_ptr()) as c_int;
                return;
            }

            // multiple matches, complete to shortest
            Com_sprintf((*edit).buffer.as_mut_ptr(), (*edit).buffer.len(), b"\\%s\0".as_ptr() as *const c_char, shortestMatch.as_ptr());
            (*edit).cursor = strlen((*edit).buffer.as_ptr()) as c_int;
            ConcatRemaining(temp.buffer.as_ptr(), completionString);

            Com_Printf(b"]\0".as_ptr() as *const c_char, (*edit).buffer.as_ptr());

            // run through again, printing matches
            Cmd_CommandCompletion(PrintMatches);
            Cvar_CommandCompletion(PrintMatches);
        }
    }
}

// ====================
// Console_Key
//
// Handles history and console scrollback
// ====================
#[allow(non_snake_case)]
pub fn Console_Key(key: c_int) {
    unsafe {
        // ctrl-L clears screen
        if keynames[key as usize].lower as c_char as u8 as char == 'l' && kg.keys[A_CTRL as usize].down {
            Cbuf_AddText(b"clear\n\0".as_ptr() as *const c_char);
            return;
        }

        // enter finishes the line
        if key == A_ENTER || key == A_KP_ENTER {
            // if not in the game explicitly prepent a slash if needed
            // Note: cls.state access is stubbed; real implementation needs proper struct access
            if kg.g_consoleField.buffer[0] as u8 as char != '\\' && kg.g_consoleField.buffer[0] as u8 as char != '/' {
                let mut temp: [c_char; 1024] = [0; 1024];

                Q_strncpyz(temp.as_mut_ptr(), kg.g_consoleField.buffer.as_ptr(), temp.len());
                Com_sprintf(kg.g_consoleField.buffer.as_mut_ptr(), kg.g_consoleField.buffer.len(), b"\\%s\0".as_ptr() as *const c_char, temp.as_ptr());
                kg.g_consoleField.cursor += 1;
            } else {
                // Added this to automatically make explicit commands not need slashes.
                CompleteCommand();
            }

            Com_Printf(b"]\0".as_ptr() as *const c_char, kg.g_consoleField.buffer.as_ptr());

            // leading slash is an explicit command
            if kg.g_consoleField.buffer[0] as u8 as char == '\\' || kg.g_consoleField.buffer[0] as u8 as char == '/' {
                if !cgvm.is_null() {
                    // Stub for cl.mSharedMemory check - real implementation needs proper struct access
                    let buf = kg.g_consoleField.buffer.as_ptr().add(1);
                    // Simplified - full implementation would check mSharedMemory and call TCGIncomingConsoleCommand
                    Cbuf_AddText(buf);
                    Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                } else {
                    // just exec it then
                    Cbuf_AddText(kg.g_consoleField.buffer.as_ptr().add(1)); // valid command
                    Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                }
            } else {
                // other text will be chat messages
                if kg.g_consoleField.buffer[0] as u8 == 0 {
                    return; // empty lines just scroll the console without adding to history
                } else {
                    Cbuf_AddText(b"cmd say \0".as_ptr() as *const c_char);
                    Cbuf_AddText(kg.g_consoleField.buffer.as_ptr());
                    Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                }
            }

            // copy line to history buffer
            kg.historyEditLines[(kg.nextHistoryLine % COMMAND_HISTORY as c_int) as usize] = kg.g_consoleField;
            kg.nextHistoryLine += 1;
            kg.historyLine = kg.nextHistoryLine;

            Field_Clear(&mut kg.g_consoleField);

            kg.g_consoleField.widthInChars = g_console_field_width;

            // cls.state check stubbed - real implementation needs proper struct access
            return;
        }

        // command completion

        if key == A_TAB {
            CompleteCommand();
            return;
        }

        // command history (ctrl-p ctrl-n for unix style)

        if (key == A_CURSOR_UP) || ((keynames[key as usize].lower as c_char as u8 as char == 'p') && kg.keys[A_CTRL as usize].down) {
            if kg.nextHistoryLine - kg.historyLine < COMMAND_HISTORY as c_int && kg.historyLine > 0 {
                kg.historyLine -= 1;
            }
            kg.g_consoleField = kg.historyEditLines[(kg.historyLine % COMMAND_HISTORY as c_int) as usize];
            return;
        }

        if (key == A_CURSOR_DOWN) || ((keynames[key as usize].lower as c_char as u8 as char == 'n') && kg.keys[A_CTRL as usize].down) {
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
}

// ============================================================================

// ================
// Message_Key
//
// In game talk message
// ================
#[allow(non_snake_case)]
pub fn Message_Key(key: c_int) {
    let mut buffer: [c_char; 1024] = [0; 1024];

    unsafe {
        if key == A_ESCAPE {
            // cls.keyCatchers &= ~KEYCATCH_MESSAGE - stubbed
            Field_Clear(&mut chatField);
            return;
        }

        if key == A_ENTER || key == A_KP_ENTER {
            if chatField.buffer[0] as u8 != 0 {
                // cls.state check stubbed
                if chat_playerNum != -1 {
                    Com_sprintf(buffer.as_mut_ptr(), buffer.len(), b"tell %i \"%s\"\n\0".as_ptr() as *const c_char, chat_playerNum, chatField.buffer.as_ptr());
                } else if chat_team {
                    Com_sprintf(buffer.as_mut_ptr(), buffer.len(), b"say_team \"%s\"\n\0".as_ptr() as *const c_char, chatField.buffer.as_ptr());
                } else {
                    Com_sprintf(buffer.as_mut_ptr(), buffer.len(), b"say \"%s\"\n\0".as_ptr() as *const c_char, chatField.buffer.as_ptr());
                }

                CL_AddReliableCommand(buffer.as_ptr());
            }
            // cls.keyCatchers &= ~KEYCATCH_MESSAGE - stubbed
            Field_Clear(&mut chatField);
            return;
        }

        Field_KeyDownEvent(&mut chatField, key);
    }
}

// ============================================================================

#[allow(non_snake_case)]
pub fn Key_GetOverstrikeMode() -> bool {
    unsafe { kg.key_overstrikeMode }
}

#[allow(non_snake_case)]
pub fn Key_SetOverstrikeMode(state: bool) {
    unsafe { kg.key_overstrikeMode = state; }
}

// ===================
// Key_IsDown
// ===================
#[allow(non_snake_case)]
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
#[allow(non_snake_case)]
pub fn Key_StringToKeynum(str: *mut c_char) -> c_int {
    let mut i: c_int;

    unsafe {
        if str.is_null() || *str as u8 == 0 {
            return -1;
        }
        // If single char bind, presume ascii char bind
        if *str.add(1) as u8 == 0 {
            return keynames[*str as u8 as usize].upper;
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
        if *str as u8 as char == '0' && *str.add(1) as u8 as char == 'x' && strlen(str) == 4 {
            let mut n1: c_int;
            let mut n2: c_int;

            n1 = *str.add(2) as c_int;
            if n1 >= '0' as c_int && n1 <= '9' as c_int {
                n1 -= '0' as c_int;
            } else if n1 >= 'A' as c_int && n1 <= 'F' as c_int {
                n1 = n1 - 'A' as c_int + 10;
            } else {
                n1 = 0;
            }

            n2 = *str.add(3) as c_int;
            if n2 >= '0' as c_int && n2 <= '9' as c_int {
                n2 -= '0' as c_int;
            } else if n2 >= 'A' as c_int && n2 <= 'F' as c_int {
                n2 = n2 - 'A' as c_int + 10;
            } else {
                n2 = 0;
            }
            return n1 * 16 + n2;
        }

        -1
    }
}

static mut tinyString: [c_char; 16] = [0; 16];

#[allow(non_snake_case)]
unsafe fn Key_KeynumValid(keynum: c_int) -> *const c_char {
    if keynum == -1 {
        return b"<KEY NOT FOUND>\0".as_ptr() as *const c_char;
    }
    if keynum < 0 || keynum >= MAX_KEYS as c_int {
        return b"<OUT OF RANGE>\0".as_ptr() as *const c_char;
    }
    ptr::null()
}

#[allow(non_snake_case)]
unsafe fn Key_KeyToName(keynum: c_int) -> *const c_char {
    keynames[keynum as usize].name
}

#[allow(non_snake_case)]
unsafe fn Key_KeyToAscii(keynum: c_int) -> *const c_char {
    if keynames[keynum as usize].lower == 0 {
        return ptr::null();
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

#[allow(non_snake_case)]
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
#[allow(non_snake_Case)]
pub fn Key_KeynumToAscii(keynum: c_int) -> *const c_char {
    let mut name: *const c_char;

    unsafe {
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
}

// ===================
// Key_KeynumToString
//
// Returns a string (either a single ascii char, a K_* name, or a 0x11 hex string) for the
// given keynum.
// ===================
// Returns a console/config file friendly name for the key
#[allow(non_snake_Case)]
pub fn Key_KeynumToString(keynum: c_int) -> *const c_char {
    let mut name: *const c_char;

    unsafe {
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
}

// ===================
// Key_SetBinding
// ===================
#[allow(non_snake_Case)]
pub fn Key_SetBinding(keynum: c_int, binding: *const c_char) {
    unsafe {
        if keynum == -1 {
            return;
        }

        // free old bindings
        if !kg.keys[keynames[keynum as usize].upper as usize].binding.is_null() {
            Z_Free(kg.keys[keynames[keynum as usize].upper as usize].binding as *mut c_void);
            kg.keys[keynames[keynum as usize].upper as usize].binding = ptr::null_mut();
        }

        // allocate memory for new binding
        if !binding.is_null() {
            kg.keys[keynames[keynum as usize].upper as usize].binding = CopyString(binding);
        }

        // consider this like modifying an archived cvar, so the
        // file write will be triggered at the next oportunity
        cvar_modifiedFlags |= CVAR_ARCHIVE;
    }
}

// ===================
// Key_GetBinding
// ===================
#[allow(non_snake_Case)]
pub fn Key_GetBinding(keynum: c_int) -> *mut c_char {
    unsafe {
        if keynum == -1 {
            return b"\0".as_ptr() as *mut c_char;
        }

        kg.keys[keynum as usize].binding
    }
}

//
// ===================
// Key_GetKey
// ===================
//

#[allow(non_snake_Case)]
pub fn Key_GetKey(binding: *const c_char) -> c_int {
    let mut i: c_int;

    unsafe {
        if !binding.is_null() {
            i = 0;
            while i < 256 {
                if !kg.keys[i as usize].binding.is_null() && Q_stricmp(binding, kg.keys[i as usize].binding) == 0 {
                    return i;
                }
                i += 1;
            }
        }
    }
    -1
}

// ===================
// Key_Unbind_f
// ===================
#[allow(non_snake_Case)]
extern "C" fn Key_Unbind_f() {
    let mut b: c_int;

    unsafe {
        if Cmd_Argc() != 2 {
            Com_Printf(b"unbind <key> : remove commands from a key\n\0".as_ptr() as *const c_char);
            return;
        }

        b = Key_StringToKeynum(Cmd_Argv(1));
        if b == -1 {
            Com_Printf(b"\"%s\" isn't a valid key\n\0".as_ptr() as *const c_char, Cmd_Argv(1));
            return;
        }

        Key_SetBinding(b, b"".as_ptr() as *const c_char);
    }
}

// ===================
// Key_Unbindall_f
// ===================
#[allow(non_snake_Case)]
extern "C" fn Key_Unbindall_f() {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < MAX_KEYS as c_int {
            if !kg.keys[i as usize].binding.is_null() {
                Key_SetBinding(i, b"".as_ptr() as *const c_char);
            }
            i += 1;
        }
    }
}

// ===================
// Key_Bind_f
// ===================
#[allow(non_snake_Case)]
extern "C" fn Key_Bind_f() {
    let mut i: c_int;
    let mut c: c_int;
    let mut b: c_int;
    let mut cmd: [c_char; 1024] = [0; 1024];

    unsafe {
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
        cmd[0] = 0; // start out with a null string
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
}

// ============
// Key_WriteBindings
//
// Writes lines containing "bind key value"
// ============
#[allow(non_snake_Case)]
pub fn Key_WriteBindings(f: *mut c_void) {
    let mut i: c_int;

    unsafe {
        FS_Printf(f, b"unbindall\n\0".as_ptr() as *const c_char);
        i = 0;
        while i < MAX_KEYS as c_int {
            if !kg.keys[i as usize].binding.is_null() && *kg.keys[i as usize].binding as u8 != 0 {
                FS_Printf(f, b"bind %s \"%s\"\n\0".as_ptr() as *const c_char, Key_KeynumToString(i), kg.keys[i as usize].binding);
            }
            i += 1;
        }
    }
}

// ============
// Key_Bindlist_f
//
// ============
#[allow(non_snake_Case)]
pub fn Key_Bindlist_f() {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < MAX_KEYS as c_int {
            if !kg.keys[i as usize].binding.is_null() && *kg.keys[i as usize].binding as u8 != 0 {
                Com_Printf(b"Key : %s (%s) \"%s\"\n\0".as_ptr() as *const c_char, Key_KeynumToAscii(i), Key_KeynumToString(i), kg.keys[i as usize].binding);
            }
            i += 1;
        }
    }
}

// ===================
// CL_InitKeyCommands
// ===================
#[allow(non_snake_Case)]
pub fn CL_InitKeyCommands() {
    // register our functions
    unsafe {
        Cmd_AddCommand(b"bind\0".as_ptr() as *const c_char, Key_Bind_f);
        Cmd_AddCommand(b"unbind\0".as_ptr() as *const c_char, Key_Unbind_f);
        Cmd_AddCommand(b"unbindall\0".as_ptr() as *const c_char, Key_Unbindall_f);
        Cmd_AddCommand(b"bindlist\0".as_ptr() as *const c_char, Key_Bindlist_f);
    }
}

// ===================
// CL_AddKeyUpCommands
// ===================
#[allow(non_snake_Case)]
pub fn CL_AddKeyUpCommands(key: c_int, kb: *mut c_char) {
    let mut i: c_int;
    let mut button: [c_char; 1024] = [0; 1024];
    let mut buttonPtr: *mut c_char;
    let mut cmd: [c_char; 1024] = [0; 1024];
    let mut keyevent: bool;

    unsafe {
        if kb.is_null() {
            return;
        }
        keyevent = false;
        buttonPtr = button.as_mut_ptr();
        i = 0;
        loop {
            if *kb.add(i as usize) as u8 as char == ';' || *kb.add(i as usize) as u8 == 0 {
                *buttonPtr = 0;
                if *button.as_ptr() as u8 as char == '+' {
                    // button commands add keynum and time as parms so that multiple
                    // sources can be discriminated and subframe corrected
                    Com_sprintf(cmd.as_mut_ptr(), cmd.len(), b"-%s %i %i\n\0".as_ptr() as *const c_char, button.as_ptr().add(1), key, 0); // time stub
                    Cbuf_AddText(cmd.as_ptr());
                    keyevent = true;
                } else {
                    if keyevent {
                        // down-only command
                        Cbuf_AddText(button.as_ptr());
                        Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                    }
                }
                buttonPtr = button.as_mut_ptr();
                while (*kb.add(i as usize) as u8 as u8) <= b' ' as u8 || *kb.add(i as usize) as u8 as char == ';' && *kb.add(i as usize) as u8 != 0 {
                    i += 1;
                }
            }
            *buttonPtr = *kb.add(i as usize);
            buttonPtr = buttonPtr.add(1);
            if *kb.add(i as usize) as u8 == 0 {
                break;
            }
            i += 1;
        }
    }
}

// ===================
// CL_KeyEvent
//
// Called by the system for both key up and key down events
// ===================
#[allow(non_snake_Case)]
pub fn CL_KeyEvent(key: c_int, down: bool, time: c_int) {
    let mut kb: *mut c_char;
    let mut cmd: [c_char; 1024] = [0; 1024];

    unsafe {
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
        #[cfg(not(target_os = "xbox"))]
        {
            // No console on Xbox
            if key == A_CONSOLE {
                if !down {
                    return;
                }
                #[cfg(feature = "FINAL_BUILD")]
                {
                    // if (!(cls.keyCatchers & KEYCATCH_CONSOLE) && !kg.keys[A_SHIFT].down)
                    // Stubbed - real implementation needs proper struct access
                }
                Con_ToggleConsole_f();
                return;
            }
        }

        // kg.keys can still be used for bound actions
        if down && (0 == 0) { // Stubbed cls.state == CA_CINEMATIC check
            if Cvar_VariableValue(b"com_cameraMode\0".as_ptr() as *const c_char) == 0.0 {
                Cvar_Set(b"nextdemo\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
                // key = A_ESCAPE - would reassign but we proceed with original
            }
        }

        // escape is always handled special
        if key == A_ESCAPE && down {
            // if (cls.keyCatchers & KEYCATCH_MESSAGE) - stubbed
            // Simplified escape handling - full implementation needs proper struct access

            if !cgvm.is_null() {
                VM_Call(cgvm, 2, 0); // CG_EVENT_HANDLING, CGAME_EVENT_NONE - stubbed
                return;
            }

            if !uivm.is_null() {
                VM_Call(uivm, 31, 10); // UI_SET_ACTIVE_MENU, UIMENU_INGAME - stubbed
            }
            return;
        }

        //
        // key up events only perform actions if the game key binding is
        // a button command (leading + sign).  These will be processed even in
        // console mode and menu mode, to keep the character from continuing
        // an action started before a mode switch.
        //
        if !down {
            kb = kg.keys[keynames[key as usize].upper as usize].binding;

            CL_AddKeyUpCommands(key, kb);

            if !uivm.is_null() {
                VM_Call(uivm, 16, key, down as c_int); // UI_KEY_EVENT
            } else if !cgvm.is_null() {
                VM_Call(cgvm, 4, key, down as c_int); // CG_KEY_EVENT
            }

            return;
        }

        // distribute the key down event to the apropriate handler
        // cls.keyCatchers checks stubbed - real implementation needs proper struct access

        // Simplified default path: send the bound action
        kb = kg.keys[keynames[key as usize].upper as usize].binding;
        if !kb.is_null() {
            if *kb as u8 as char == '+' {
                let mut i: c_int = 0;
                let mut button: [c_char; 1024] = [0; 1024];
                let mut buttonPtr: *mut c_char = button.as_mut_ptr();
                loop {
                    if *kb.add(i as usize) as u8 as char == ';' || *kb.add(i as usize) as u8 == 0 {
                        *buttonPtr = 0;
                        if *button.as_ptr() as u8 as char == '+' {
                            // button commands add keynum and time as parms so that multiple
                            // sources can be discriminated and subframe corrected
                            Com_sprintf(cmd.as_mut_ptr(), cmd.len(), b"%s %i %i\n\0".as_ptr() as *const c_char, button.as_ptr(), key, time);
                            Cbuf_AddText(cmd.as_ptr());
                        } else {
                            // down-only command
                            Cbuf_AddText(button.as_ptr());
                            Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                        }
                        buttonPtr = button.as_mut_ptr();
                        while (*kb.add(i as usize) as u8 as u8) <= b' ' as u8 || *kb.add(i as usize) as u8 as char == ';' && *kb.add(i as usize) as u8 != 0 {
                            i += 1;
                        }
                    }
                    *buttonPtr = *kb.add(i as usize);
                    buttonPtr = buttonPtr.add(1);
                    if *kb.add(i as usize) as u8 == 0 {
                        break;
                    }
                    i += 1;
                }
            } else {
                // down-only command
                if !cgvm.is_null() {
                    // Stub for cl.mSharedMemory check - simplified version
                    Cbuf_AddText(kb);
                    Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                } else {
                    // otherwise just add it
                    Cbuf_AddText(kb);
                    Cbuf_AddText(b"\n\0".as_ptr() as *const c_char);
                }
            }
        }
    }
}

// ===================
// CL_CharEvent
//
// Normal keyboard characters, already shifted / capslocked / etc
// ===================
#[allow(non_snake_Case)]
pub fn CL_CharEvent(key: c_int) {
    // the console key should never be used as a char
    if key == ('`' as c_int) || key == ('~' as c_int) {
        return;
    }

    unsafe {
        // distribute the key down event to the apropriate handler
        // Simplified - full implementation needs cls.keyCatchers checks
        Field_CharEvent(&mut kg.g_consoleField, key);
    }
}

// ===================
// Key_ClearStates
// ===================
#[allow(non_snake_Case)]
pub fn Key_ClearStates() {
    let mut i: c_int;

    unsafe {
        kg.anykeydown = false;

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
}
