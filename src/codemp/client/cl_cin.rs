//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// /*****************************************************************************
//  * name:		cl_cin.c
//  *
//  * desc:		video and cinematic playback
//  *
//  * $Archive: /MissionPack/code/client/cl_cin.c $
//  * $Author: osman $
//  * $Revision: 1.4 $
//  * $Modtime: 6/12/01 10:36a $
//  * $Date: 2003/03/15 23:43:59 $
//  *
//  * cl_glconfig.hwtype trtypes 3dfx/ragepro need 256x256
//  *
//  *****************************************************************************/

// #include "client.h"
// #include "snd_local.h"

use core::ffi::{c_int, c_char, c_void};

const MAXSIZE: c_int = 8;
const MINSIZE: c_int = 4;

const DEFAULT_CIN_WIDTH: c_int = 512;
const DEFAULT_CIN_HEIGHT: c_int = 512;

const ROQ_QUAD: c_int = 0x1000;
const ROQ_QUAD_INFO: c_int = 0x1001;
const ROQ_CODEBOOK: c_int = 0x1002;
const ROQ_QUAD_VQ: c_int = 0x1011;
const ROQ_QUAD_JPEG: c_int = 0x1012;
const ROQ_QUAD_HANG: c_int = 0x1013;
const ROQ_PACKET: c_int = 0x1030;
const ZA_SOUND_MONO: c_int = 0x1020;
const ZA_SOUND_STEREO: c_int = 0x1021;

const MAX_VIDEO_HANDLES: c_int = 16;

extern "C" {
    static mut glConfig: glconfig_t;
    static mut s_paintedtime: c_int;
    static mut s_rawend: c_int;
    static mut s_soundtime: c_int;
    static mut s_volume: *mut cvar_t;
    static mut uivm: *mut vm_t;
    static mut cls: client_state_t;
    static mut cl_inGameVideo: *mut cvar_t;
    static mut com_timescale: *mut cvar_t;

    fn Com_Error(level: c_int, msg: *const c_char, ...);
    fn Com_DPrintf(msg: *const c_char, ...);
    fn Com_Printf(msg: *const c_char, ...);
    fn Com_Memset(ptr: *mut c_void, c: c_int, n: usize);
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, n: usize);
    fn Com_sprintf(dest: *mut c_char, destsize: usize, fmt: *const c_char, ...);
    fn Sys_EndStreamedFile(f: fileHandle_t);
    fn Sys_BeginStreamedFile(f: fileHandle_t, bufferSize: c_int);
    fn Sys_StreamedRead(buffer: *mut c_void, size: usize, count: usize, f: fileHandle_t) -> usize;
    fn Sys_Milliseconds() -> c_int;
    fn FS_Seek(f: fileHandle_t, offset: c_int, origin: c_int);
    fn FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int;
    fn FS_FOpenFileRead(filename: *const c_char, file: *mut fileHandle_t, uniqueFILE: u32) -> c_int;
    fn FS_FCloseFile(f: fileHandle_t);
    fn Cvar_VariableString(var_name: *const c_char) -> *const c_char;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn VM_Call(vm: *mut vm_t, ...);
    fn S_RawSamples(samples: c_int, rate: c_int, width: c_int, channels: c_int, data: *const c_char, volume: f32, soundpan: c_int);
    fn S_Update();
    fn S_StopAllSounds();
    fn LittleLong(l: c_int) -> c_int;
    fn Hunk_AllocateTempMemory(size: usize) -> *mut c_void;
    fn Hunk_FreeTempMemory(ptr: *mut c_void);
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn COM_DefaultExtension(path: *mut c_char, pathlen: usize, extension: *const c_char);
    fn Con_Close();

    // RE (render engine) interface
    fn re_DrawStretchRaw(x: f32, y: f32, w: f32, h: f32, cols: c_int, rows: c_int, data: *const c_char, client: c_int, dirty: bool);
    fn re_UploadCinematic(cols: c_int, rows: c_int, data: *const c_char, client: c_int, dirty: bool);
}

// Stub types for external dependencies
#[repr(C)]
pub struct glconfig_t {
    _unused: [u8; 1],
}

#[repr(C)]
pub struct cvar_t {
    _unused: [u8; 1],
}

#[repr(C)]
pub struct vm_t {
    _unused: [u8; 1],
}

#[repr(C)]
pub struct client_state_t {
    _unused: [u8; 1],
}

pub type fileHandle_t = c_int;
pub type qboolean = c_int;

const ERR_DROP: c_int = 1;
const FS_SEEK_SET: c_int = 0;
const EXEC_APPEND: c_int = 0;
const CIN_system: c_int = 1;
const CIN_loop: c_int = 2;
const CIN_hold: c_int = 4;
const CIN_silent: c_int = 8;
const CIN_shader: c_int = 16;
const MAX_OSPATH: usize = 256;
const SCREEN_WIDTH: c_int = 640;
const SCREEN_HEIGHT: c_int = 480;
const CA_DISCONNECTED: c_int = 0;
const CA_CINEMATIC: c_int = 1;
const UI_SET_ACTIVE_MENU: c_int = 1;
const UIMENU_NONE: c_int = 0;
const qtrue: c_int = 1;
const qfalse: c_int = 0;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum e_status {
    FMV_EOF = 0,
    FMV_PLAY = 1,
    FMV_LOOPED = 2,
    FMV_IDLE = 3,
}

static mut ROQ_YY_tab: [i32; 256] = [0; 256];
static mut ROQ_UB_tab: [i32; 256] = [0; 256];
static mut ROQ_UG_tab: [i32; 256] = [0; 256];
static mut ROQ_VG_tab: [i32; 256] = [0; 256];
static mut ROQ_VR_tab: [i32; 256] = [0; 256];
static mut vq2: [u16; 256 * 16 * 4] = [0; 256 * 16 * 4];
static mut vq4: [u16; 256 * 64 * 4] = [0; 256 * 64 * 4];
static mut vq8: [u16; 256 * 256 * 4] = [0; 256 * 256 * 4];

#[repr(C)]
pub struct cinematics_t {
    pub linbuf: [u8; DEFAULT_CIN_WIDTH as usize * DEFAULT_CIN_HEIGHT as usize * 4 * 2],
    pub file: [u8; 65536],
    pub sqrTable: [i16; 256],

    pub mcomp: [u32; 256],
    pub qStatus: [[*mut u8; 32768]; 2],

    pub oldXOff: i32,
    pub oldYOff: i32,
    pub oldysize: u32,
    pub oldxsize: u32,
}

#[repr(C)]
pub struct cin_cache {
    pub fileName: [c_char; MAX_OSPATH],
    pub CIN_WIDTH: c_int,
    pub CIN_HEIGHT: c_int,
    pub xpos: c_int,
    pub ypos: c_int,
    pub width: c_int,
    pub height: c_int,
    pub looping: c_int,
    pub holdAtEnd: c_int,
    pub dirty: c_int,
    pub alterGameState: c_int,
    pub silent: c_int,
    pub shader: c_int,
    pub iFile: fileHandle_t,
    pub status: e_status,
    pub startTime: u32,
    pub lastTime: u32,
    pub tfps: i32,
    pub RoQPlayed: i32,
    pub ROQSize: i32,
    pub RoQFrameSize: u32,
    pub onQuad: i32,
    pub numQuads: i32,
    pub samplesPerLine: i32,
    pub roq_id: u32,
    pub screenDelta: i32,

    pub VQ0: Option<extern "C" fn(*mut u8, *mut c_void)>,
    pub VQ1: Option<extern "C" fn(*mut u8, *mut c_void)>,
    pub VQNormal: Option<extern "C" fn(*mut u8, *mut c_void)>,
    pub VQBuffer: Option<extern "C" fn(*mut u8, *mut c_void)>,

    pub gray: *mut u8,
    pub xsize: u32,
    pub ysize: u32,
    pub maxsize: u32,
    pub minsize: u32,

    pub inMemory: c_int,
    pub normalBuffer0: i32,
    pub roq_flags: i32,
    pub roqF0: i32,
    pub roqF1: i32,
    pub t: [i32; 2],
    pub roqFPS: i32,
    pub playonwalls: c_int,
    pub buf: *mut u8,
    pub drawX: i32,
    pub drawY: i32,
}

static mut cin: cinematics_t = cinematics_t {
    linbuf: [0; DEFAULT_CIN_WIDTH as usize * DEFAULT_CIN_HEIGHT as usize * 4 * 2],
    file: [0; 65536],
    sqrTable: [0; 256],
    mcomp: [0; 256],
    qStatus: [[core::ptr::null_mut(); 32768]; 2],
    oldXOff: 0,
    oldYOff: 0,
    oldysize: 0,
    oldxsize: 0,
};

static mut cinTable: [cin_cache; MAX_VIDEO_HANDLES as usize] = [
    cin_cache {
        fileName: [0; MAX_OSPATH],
        CIN_WIDTH: 0,
        CIN_HEIGHT: 0,
        xpos: 0,
        ypos: 0,
        width: 0,
        height: 0,
        looping: 0,
        holdAtEnd: 0,
        dirty: 0,
        alterGameState: 0,
        silent: 0,
        shader: 0,
        iFile: 0,
        status: e_status::FMV_EOF,
        startTime: 0,
        lastTime: 0,
        tfps: 0,
        RoQPlayed: 0,
        ROQSize: 0,
        RoQFrameSize: 0,
        onQuad: 0,
        numQuads: 0,
        samplesPerLine: 0,
        roq_id: 0,
        screenDelta: 0,
        VQ0: None,
        VQ1: None,
        VQNormal: None,
        VQBuffer: None,
        gray: core::ptr::null_mut(),
        xsize: 0,
        ysize: 0,
        maxsize: 0,
        minsize: 0,
        inMemory: 0,
        normalBuffer0: 0,
        roq_flags: 0,
        roqF0: 0,
        roqF1: 0,
        t: [0; 2],
        roqFPS: 0,
        playonwalls: 0,
        buf: core::ptr::null_mut(),
        drawX: 0,
        drawY: 0,
    };
    MAX_VIDEO_HANDLES as usize
];

static mut currentHandle: c_int = -1;
static mut CL_handle: c_int = -1;

pub fn CIN_CloseAllVideos() {
    let mut i: c_int;

    for i in 0..MAX_VIDEO_HANDLES {
        unsafe {
            if (*core::ptr::addr_of_mut!(cinTable[i as usize])).fileName[0] != 0 {
                CIN_StopCinematic(i);
            }
        }
    }
}

static fn CIN_HandleForVideo() -> c_int {
    let mut i: c_int;

    for i in 0..MAX_VIDEO_HANDLES {
        unsafe {
            if (*core::ptr::addr_of_mut!(cinTable[i as usize])).fileName[0] == 0 {
                return i;
            }
        }
    }
    unsafe {
        Com_Error(ERR_DROP, b"CIN_HandleForVideo: none free\0".as_ptr() as *const c_char);
    }
    -1
}

//-----------------------------------------------------------------------------
// RllSetupTable
//
// Allocates and initializes the square table.
//
// Parameters:	None
//
// Returns:		Nothing
//-----------------------------------------------------------------------------
static fn RllSetupTable() {
    let mut z: c_int;

    unsafe {
        for z in 0..128 {
            (*core::ptr::addr_of_mut!(cin)).sqrTable[z as usize] = (z * z) as i16;
            (*core::ptr::addr_of_mut!(cin)).sqrTable[(z + 128) as usize] = -(*core::ptr::addr_of_mut!(cin)).sqrTable[z as usize];
        }
    }
}

//-----------------------------------------------------------------------------
// RllDecodeMonoToMono
//
// Decode mono source data into a mono buffer.
//
// Parameters:	from -> buffer holding encoded data
//				to ->	buffer to hold decoded data
//				size =	number of bytes of input (= # of shorts of output)
//				signedOutput = 0 for unsigned output, non-zero for signed output
//				flag = flags from asset header
//
// Returns:		Number of samples placed in output buffer
//-----------------------------------------------------------------------------
static fn RllDecodeMonoToMono(from: *const u8, to: *mut i16, size: u32, signedOutput: c_char, flag: u16) -> i32 {
    let mut z: u32;
    let mut prev: i32;

    unsafe {
        if signedOutput != 0 {
            prev = (flag as i32) - 0x8000;
        } else {
            prev = flag as i32;
        }

        for z in 0..size {
            prev = prev + (*core::ptr::addr_of_mut!(cin)).sqrTable[*from.add(z as usize) as usize] as i32;
            *to.add(z as usize) = prev as i16;
        }
    }
    size as i32
}

//-----------------------------------------------------------------------------
// RllDecodeMonoToStereo
//
// Decode mono source data into a stereo buffer. Output is 4 times the number
// of bytes in the input.
//
// Parameters:	from -> buffer holding encoded data
//				to ->	buffer to hold decoded data
//				size =	number of bytes of input (= 1/4 # of bytes of output)
//				signedOutput = 0 for unsigned output, non-zero for signed output
//				flag = flags from asset header
//
// Returns:		Number of samples placed in output buffer
//-----------------------------------------------------------------------------
static fn RllDecodeMonoToStereo(from: *const u8, to: *mut i16, size: u32, signedOutput: c_char, flag: u16) -> i32 {
    let mut z: u32;
    let mut prev: i32;

    unsafe {
        if signedOutput != 0 {
            prev = (flag as i32) - 0x8000;
        } else {
            prev = flag as i32;
        }

        for z in 0..size {
            prev = prev + (*core::ptr::addr_of_mut!(cin)).sqrTable[*from.add(z as usize) as usize] as i32;
            *to.add((z * 2 + 0) as usize) = prev as i16;
            *to.add((z * 2 + 1) as usize) = prev as i16;
        }
    }

    size as i32
}

//-----------------------------------------------------------------------------
// RllDecodeStereoToStereo
//
// Decode stereo source data into a stereo buffer.
//
// Parameters:	from -> buffer holding encoded data
//				to ->	buffer to hold decoded data
//				size =	number of bytes of input (= 1/2 # of bytes of output)
//				signedOutput = 0 for unsigned output, non-zero for signed output
//				flag = flags from asset header
//
// Returns:		Number of samples placed in output buffer
//-----------------------------------------------------------------------------
static fn RllDecodeStereoToStereo(from: *const u8, to: *mut i16, size: u32, signedOutput: c_char, flag: u16) -> i32 {
    let mut z: u32;
    let mut zz: *const u8 = from;
    let mut prevL: i32;
    let mut prevR: i32;

    unsafe {
        if signedOutput != 0 {
            prevL = ((flag as i32) & 0xff00) - 0x8000;
            prevR = ((((flag as i32) & 0x00ff) << 8) - 0x8000);
        } else {
            prevL = (flag as i32) & 0xff00;
            prevR = ((flag as i32) & 0x00ff) << 8;
        }

        z = 0;
        while z < size {
            prevL = prevL + (*core::ptr::addr_of_mut!(cin)).sqrTable[*zz as usize] as i32;
            zz = zz.add(1);
            prevR = prevR + (*core::ptr::addr_of_mut!(cin)).sqrTable[*zz as usize] as i32;
            zz = zz.add(1);
            *to.add(z as usize) = prevL as i16;
            *to.add((z + 1) as usize) = prevR as i16;
            z += 2;
        }
    }

    (size >> 1) as i32
}

//-----------------------------------------------------------------------------
// RllDecodeStereoToMono
//
// Decode stereo source data into a mono buffer.
//
// Parameters:	from -> buffer holding encoded data
//				to ->	buffer to hold decoded data
//				size =	number of bytes of input (= # of bytes of output)
//				signedOutput = 0 for unsigned output, non-zero for signed output
//				flag = flags from asset header
//
// Returns:		Number of samples placed in output buffer
//-----------------------------------------------------------------------------
static fn RllDecodeStereoToMono(from: *const u8, to: *mut i16, size: u32, signedOutput: c_char, flag: u16) -> i32 {
    let mut z: u32;
    let mut prevL: i32;
    let mut prevR: i32;

    unsafe {
        if signedOutput != 0 {
            prevL = ((flag as i32) & 0xff00) - 0x8000;
            prevR = ((((flag as i32) & 0x00ff) << 8) - 0x8000);
        } else {
            prevL = (flag as i32) & 0xff00;
            prevR = ((flag as i32) & 0x00ff) << 8;
        }

        for z in 0..size {
            prevL = prevL + (*core::ptr::addr_of_mut!(cin)).sqrTable[*from.add((z * 2) as usize) as usize] as i32;
            prevR = prevR + (*core::ptr::addr_of_mut!(cin)).sqrTable[*from.add((z * 2 + 1) as usize) as usize] as i32;
            *to.add(z as usize) = ((prevL + prevR) / 2) as i16;
        }
    }

    size as i32
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn move8_32(src: *const u8, dst: *mut u8, spl: c_int) {
    let mut dsrc: *const f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *const f64;
        ddst = dst as *mut f64;
        dspl = spl >> 3;

        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn move4_32(src: *const u8, dst: *mut u8, spl: c_int) {
    let mut dsrc: *const f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *const f64;
        ddst = dst as *mut f64;
        dspl = spl >> 3;

        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        dsrc = dsrc.add(dspl as usize);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn blit8_32(src: *const u8, dst: *mut u8, spl: c_int) {
    let mut dsrc: *const f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *const f64;
        ddst = dst as *mut f64;
        dspl = spl >> 3;

        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(4);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(4);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(4);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(4);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(4);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(4);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
        dsrc = dsrc.add(4);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        *ddst.add(2) = *dsrc.add(2);
        *ddst.add(3) = *dsrc.add(3);
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/
#[allow(non_snake_case)]
type movs = f64;

static fn blit4_32(src: *const u8, dst: *mut u8, spl: c_int) {
    let mut dsrc: *const movs;
    let mut ddst: *mut movs;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *const movs;
        ddst = dst as *mut movs;
        dspl = spl >> 3;

        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        dsrc = dsrc.add(2);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        dsrc = dsrc.add(2);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
        dsrc = dsrc.add(2);
        ddst = ddst.add(dspl as usize);
        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(1) = *dsrc.add(1);
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn blit2_32(src: *const u8, dst: *mut u8, spl: c_int) {
    let mut dsrc: *const f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *const f64;
        ddst = dst as *mut f64;
        dspl = spl >> 3;

        *ddst.add(0) = *dsrc.add(0);
        *ddst.add(dspl as usize) = *dsrc.add(1);
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn blitVQQuad32fs(status: *mut *mut u8, data: *mut *const u8) {
    let mut newd: u16;
    let mut celdata: u16;
    let mut code: u16;
    let mut index: u32;
    let mut i: u32;
    let mut spl: c_int;

    unsafe {
        newd = 0;
        celdata = 0;
        index = 0;

        spl = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).samplesPerLine;

        loop {
            if newd == 0 {
                newd = 7;
                celdata = (**data) as u16 + ((**data.add(1)) as u16) * 256;
                *data = data.add(2);
            } else {
                newd -= 1;
            }

            code = (celdata & 0xc000) as u16;
            celdata <<= 2;

            match code {
                0x8000 => {
                    // vq code
                    blit8_32(
                        &vq8[(**data as usize) * 128] as *const u16 as *const u8,
                        *status,
                        spl,
                    );
                    *data = data.add(1);
                    index += 5;
                }
                0xc000 => {
                    // drop
                    index += 1;
                    for i in 0..4 {
                        if newd == 0 {
                            newd = 7;
                            celdata = (**data) as u16 + (**data.add(1) as u16) * 256;
                            *data = data.add(2);
                        } else {
                            newd -= 1;
                        }

                        code = (celdata & 0xc000) as u16;
                        celdata <<= 2;

                        match code {
                            0x8000 => {
                                // 4x4 vq code
                                blit4_32(
                                    &vq4[(**data as usize) * 32] as *const u16 as *const u8,
                                    *status.add(index as usize),
                                    spl,
                                );
                                *data = data.add(1);
                            }
                            0xc000 => {
                                // 2x2 vq code
                                blit2_32(
                                    &vq2[(**data as usize) * 8] as *const u16 as *const u8,
                                    *status.add(index as usize),
                                    spl,
                                );
                                *data = data.add(1);
                                blit2_32(
                                    &vq2[(**data as usize) * 8] as *const u16 as *const u8,
                                    (*status.add(index as usize)).add(8),
                                    spl,
                                );
                                *data = data.add(1);
                                blit2_32(
                                    &vq2[(**data as usize) * 8] as *const u16 as *const u8,
                                    (*status.add(index as usize)).add((spl * 2) as usize),
                                    spl,
                                );
                                *data = data.add(1);
                                blit2_32(
                                    &vq2[(**data as usize) * 8] as *const u16 as *const u8,
                                    (*status.add(index as usize)).add((spl * 2 + 8) as usize),
                                    spl,
                                );
                                *data = data.add(1);
                            }
                            0x4000 => {
                                // motion compensation
                                move4_32(
                                    (*status.add(index as usize)).add((*core::ptr::addr_of_mut!(cin)).mcomp[**data as usize] as usize),
                                    *status.add(index as usize),
                                    spl,
                                );
                                *data = data.add(1);
                            }
                            _ => {}
                        }
                        index += 1;
                    }
                }
                0x4000 => {
                    // motion compensation
                    move8_32(
                        (*status.add(index as usize)).add((*core::ptr::addr_of_mut!(cin)).mcomp[**data as usize] as usize),
                        *status.add(index as usize),
                        spl,
                    );
                    *data = data.add(1);
                    index += 5;
                }
                0x0000 => {
                    index += 5;
                }
                _ => {}
            }

            if *status.add(index as usize).is_null() {
                break;
            }
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn ROQ_GenYUVTables() {
    let mut t_ub: f32;
    let mut t_vr: f32;
    let mut t_ug: f32;
    let mut t_vg: f32;
    let mut i: i32;

    t_ub = (1.77200f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;
    t_vr = (1.40200f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;
    t_ug = (0.34414f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;
    t_vg = (0.71414f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;

    unsafe {
        for i in 0..256 {
            let x: f32 = (2 * i - 255) as f32;

            ROQ_UB_tab[i as usize] = ((t_ub * x) + ((1i32 << 5) as f32)) as i32;
            ROQ_VR_tab[i as usize] = ((t_vr * x) + ((1i32 << 5) as f32)) as i32;
            ROQ_UG_tab[i as usize] = ((-t_ug * x)) as i32;
            ROQ_VG_tab[i as usize] = ((-t_vg * x) + ((1i32 << 5) as f32)) as i32;
            ROQ_YY_tab[i as usize] = ((i << 6) | (i >> 2)) as i32;
        }
    }
}

#[allow(non_snake_case)]
macro_rules! VQ2TO4 {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {{
        let mut a = $a;
        let mut b = $b;
        let mut c = $c;
        let mut d = $d;
        unsafe {
            *c = *a;
            c = c.add(1);
            *d = *a;
            d = d.add(1);
            *d = *a;
            d = d.add(1);
            *c = *a.add(1);
            c = c.add(1);
            *d = *a.add(1);
            d = d.add(1);
            *d = *a.add(1);
            d = d.add(1);
            *c = *b;
            c = c.add(1);
            *d = *b;
            d = d.add(1);
            *d = *b;
            d = d.add(1);
            *c = *b.add(1);
            c = c.add(1);
            *d = *b.add(1);
            d = d.add(1);
            *d = *b.add(1);
            d = d.add(1);
            *d = *a;
            d = d.add(1);
            *d = *a;
            d = d.add(1);
            *d = *a.add(1);
            d = d.add(1);
            *d = *a.add(1);
            d = d.add(1);
            *d = *b;
            d = d.add(1);
            *d = *b;
            d = d.add(1);
            *d = *b.add(1);
            d = d.add(1);
            *d = *b.add(1);
            d = d.add(1);
            a = a.add(2);
            b = b.add(2);
        }
    }};
}

#[allow(non_snake_case)]
macro_rules! VQ2TO2 {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {{
        let mut a = $a;
        let mut b = $b;
        let mut c = $c;
        let mut d = $d;
        unsafe {
            *c = *a;
            c = c.add(1);
            *d = *a;
            d = d.add(1);
            *d = *a;
            d = d.add(1);
            *c = *b;
            c = c.add(1);
            *d = *b;
            d = d.add(1);
            *d = *b;
            d = d.add(1);
            *d = *a;
            d = d.add(1);
            *d = *a;
            d = d.add(1);
            *d = *b;
            d = d.add(1);
            *d = *b;
            d = d.add(1);
            a = a.add(1);
            b = b.add(1);
        }
    }};
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/
#[cfg(target_os = "macos")]
#[inline]
static fn yuv_to_rgb24(y: i32, u: i32, v: i32) -> u32 {
    let mut r: i32;
    let mut g: i32;
    let mut b: i32;
    let mut YY: i32;

    unsafe {
        YY = ROQ_YY_tab[y as usize];
    }

    r = unsafe { (YY + ROQ_VR_tab[v as usize]) >> 6 };
    g = unsafe { (YY + ROQ_UG_tab[u as usize] + ROQ_VG_tab[v as usize]) >> 6 };
    b = unsafe { (YY + ROQ_UB_tab[u as usize]) >> 6 };

    if r < 0 {
        r = 0;
    }
    if g < 0 {
        g = 0;
    }
    if b < 0 {
        b = 0;
    }
    if r > 255 {
        r = 255;
    }
    if g > 255 {
        g = 255;
    }
    if b > 255 {
        b = 255;
    }

    (((r << 24) | (g << 16) | (b << 8)) | 255) as u32
}

#[cfg(not(target_os = "macos"))]
static fn yuv_to_rgb24(y: i32, u: i32, v: i32) -> u32 {
    let mut r: i32;
    let mut g: i32;
    let mut b: i32;

    unsafe {
        let YY: i32 = ROQ_YY_tab[y as usize];

        r = (YY + ROQ_VR_tab[v as usize]) >> 6;
        g = (YY + ROQ_UG_tab[u as usize] + ROQ_VG_tab[v as usize]) >> 6;
        b = (YY + ROQ_UB_tab[u as usize]) >> 6;
    }

    if r < 0 {
        r = 0;
    }
    if g < 0 {
        g = 0;
    }
    if b < 0 {
        b = 0;
    }
    if r > 255 {
        r = 255;
    }
    if g > 255 {
        g = 255;
    }
    if b > 255 {
        b = 255;
    }

    unsafe { LittleLong((r | (g << 8) | (b << 16) | (255 << 24)) as c_int) as u32 }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn decodeCodeBook(input: *mut u8, roq_flags: u16) {
    let mut i: i32;
    let mut j: i32;
    let mut two: i32;
    let mut four: i32;
    let mut bptr: *mut u16;
    let mut y0: i32;
    let mut y1: i32;
    let mut y2: i32;
    let mut y3: i32;
    let mut cr: i32;
    let mut cb: i32;
    let mut ibptr: *mut u32;
    let mut icptr: *mut u32;
    let mut idptr: *mut u32;

    unsafe {
        if roq_flags == 0 {
            two = 256;
            four = 256;
        } else {
            two = (roq_flags >> 8) as i32;
            if two == 0 {
                two = 256;
            }
            four = (roq_flags & 0xff) as i32;
        }

        four *= 2;

        bptr = vq2.as_mut_ptr();

        //
        // normal height
        //
        ibptr = bptr as *mut u32;
        for i in 0..two {
            y0 = *input as i32;
            input = input.add(1);
            y1 = *input as i32;
            input = input.add(1);
            y2 = *input as i32;
            input = input.add(1);
            y3 = *input as i32;
            input = input.add(1);
            cr = *input as i32;
            input = input.add(1);
            cb = *input as i32;
            input = input.add(1);
            *ibptr = yuv_to_rgb24(y0, cr, cb);
            ibptr = ibptr.add(1);
            *ibptr = yuv_to_rgb24(y1, cr, cb);
            ibptr = ibptr.add(1);
            *ibptr = yuv_to_rgb24(y2, cr, cb);
            ibptr = ibptr.add(1);
            *ibptr = yuv_to_rgb24(y3, cr, cb);
            ibptr = ibptr.add(1);
        }

        icptr = vq4.as_mut_ptr() as *mut u32;
        idptr = vq8.as_mut_ptr() as *mut u32;

        for i in 0..four {
            let iaptr: *mut u32 = (vq2.as_mut_ptr() as *mut u32).add((*input as usize) * 4);
            input = input.add(1);
            let ibptr2: *mut u32 = (vq2.as_mut_ptr() as *mut u32).add((*input as usize) * 4);
            input = input.add(1);
            for j in 0..2 {
                VQ2TO4!(iaptr, ibptr2, icptr, idptr);
            }
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn recurseQuad(startX: i32, startY: i32, quadSize: i32, xOff: i32, yOff: i32) {
    let mut scroff: *mut u8;
    let mut bigx: i32;
    let mut bigy: i32;
    let mut lowx: i32;
    let mut lowy: i32;
    let mut useY: i32;
    let mut offset: i32;

    unsafe {
        offset = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).screenDelta;

        lowx = 0;
        lowy = 0;
        bigx = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize as i32;
        bigy = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize as i32;

        if bigx > (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH {
            bigx = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH;
        }
        if bigy > (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT {
            bigy = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT;
        }

        if (startX >= lowx)
            && (startX + quadSize) <= (bigx)
            && (startY + quadSize) <= (bigy)
            && (startY >= lowy)
            && quadSize <= MAXSIZE
        {
            useY = startY;
            scroff = (*core::ptr::addr_of_mut!(cin)).linbuf.as_mut_ptr().add(
                (useY
                    + ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT
                        - bigy)
                        >> 1
                    + yOff) as usize
                    * (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).samplesPerLine as usize
                    + ((startX + xOff) as usize * 4),
            );

            (*core::ptr::addr_of_mut!(cin)).qStatus[0][(*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).onQuad as usize] =
                scroff;
            (*core::ptr::addr_of_mut!(cin)).qStatus[1][(*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).onQuad as usize] =
                scroff.add(offset as usize);
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).onQuad += 1;
        }

        if quadSize != MINSIZE {
            let newQuadSize = quadSize >> 1;
            recurseQuad(startX, startY, newQuadSize, xOff, yOff);
            recurseQuad(startX + newQuadSize, startY, newQuadSize, xOff, yOff);
            recurseQuad(startX, startY + newQuadSize, newQuadSize, xOff, yOff);
            recurseQuad(startX + newQuadSize, startY + newQuadSize, newQuadSize, xOff, yOff);
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn setupQuad(xOff: i32, yOff: i32) {
    let mut numQuadCels: i32;
    let mut i: i32;
    let mut x: i32;
    let mut y: i32;
    let mut temp: *mut u8;

    unsafe {
        if xOff == (*core::ptr::addr_of_mut!(cin)).oldXOff
            && yOff == (*core::ptr::addr_of_mut!(cin)).oldYOff
            && (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize == (*core::ptr::addr_of_mut!(cin)).oldysize
            && (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize == (*core::ptr::addr_of_mut!(cin)).oldxsize
        {
            return;
        }

        (*core::ptr::addr_of_mut!(cin)).oldXOff = xOff;
        (*core::ptr::addr_of_mut!(cin)).oldYOff = yOff;
        (*core::ptr::addr_of_mut!(cin)).oldysize = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize;
        (*core::ptr::addr_of_mut!(cin)).oldxsize = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize;

        numQuadCels = (((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH
            * (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT)
            / (16)) as i32;
        numQuadCels += numQuadCels / 4 + numQuadCels / 16;
        numQuadCels += 64;

        numQuadCels = (((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize
            as i32
            * (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize as i32)
            / (16)) as i32;
        numQuadCels += numQuadCels / 4;
        numQuadCels += 64;

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).onQuad = 0;

        let mut y = 0i32;
        while y < (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize as i32 {
            let mut x = 0i32;
            while x < (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize as i32 {
                recurseQuad(x, y, 16, xOff, yOff);
                x += 16;
            }
            y += 16;
        }

        temp = core::ptr::null_mut();

        for i in (numQuadCels - 64)..numQuadCels {
            (*core::ptr::addr_of_mut!(cin)).qStatus[0][i as usize] = temp;
            (*core::ptr::addr_of_mut!(cin)).qStatus[1][i as usize] = temp;
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn readQuadInfo(qData: *const u8) {
    unsafe {
        if *core::ptr::addr_of_mut!(currentHandle) < 0 {
            return;
        }

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize =
            (*qData as u32) + ((*qData.add(1) as u32) * 256);
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize =
            (*qData.add(2) as u32) + ((*qData.add(3) as u32) * 256);
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).maxsize =
            (*qData.add(4) as u32) + ((*qData.add(5) as u32) * 256);
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).minsize =
            (*qData.add(6) as u32) + ((*qData.add(7) as u32) * 256);

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT =
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize as c_int;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH =
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize as c_int;

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).samplesPerLine =
            ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH as c_int) * 4;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).screenDelta =
            ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT as c_int)
                * (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).samplesPerLine;

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQ0 =
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQNormal;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQ1 =
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQBuffer;

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).t[0] =
            (0 - ((*core::ptr::addr_of_mut!(cin)).linbuf.as_ptr() as u32)) + ((*core::ptr::addr_of_mut!(cin)).linbuf.as_ptr() as u32)
                + (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).screenDelta as u32;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).t[1] = (0
            - (((*core::ptr::addr_of_mut!(cin)).linbuf.as_ptr() as u32)
                + (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).screenDelta as u32))
            + ((*core::ptr::addr_of_mut!(cin)).linbuf.as_ptr() as u32)) as i32;

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawX =
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawY =
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT;
        // jic the card sucks
        if (*glConfig).maxTextureSize <= 256 {
            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawX > 256 {
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawX = 256;
            }
            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawY > 256 {
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawY = 256;
            }
            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH != 256
                || (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT != 256
            {
                Com_Printf(b"HACK: approxmimating cinematic for Rage Pro or Voodoo\n\0".as_ptr() as *const c_char);
            }
        }
        #[cfg(target_os = "macos")]
        {
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawX = 256;
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).drawX = 256;
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn RoQPrepMcomp(xoff: i32, yoff: i32) {
    let mut i: i32;
    let mut j: i32;
    let mut x: i32;
    let mut y: i32;
    let mut temp: i32;
    let mut temp2: i32;

    unsafe {
        i = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).samplesPerLine;
        j = 4;
        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).xsize
            == ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize * 4)
        {
            j = j + j;
            i = i + i;
        }

        for y in 0..16 {
            temp2 = (y + yoff - 8) * i;
            for x in 0..16 {
                temp = (x + xoff - 8) * j;
                (*core::ptr::addr_of_mut!(cin)).mcomp[((x * 16) + y) as usize] =
                    ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).normalBuffer0
                        - (temp2 + temp)) as u32;
            }
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn initRoQ() {
    unsafe {
        if *core::ptr::addr_of_mut!(currentHandle) < 0 {
            return;
        }

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQNormal =
            Some(blitVQQuad32fs as extern "C" fn(*mut u8, *mut c_void));
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQBuffer =
            Some(blitVQQuad32fs as extern "C" fn(*mut u8, *mut c_void));
        ROQ_GenYUVTables();
        RllSetupTable();
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/
/*
static byte* RoQFetchInterlaced( byte *source ) {
    int x, *src, *dst;

    if (currentHandle < 0) return NULL;

    src = (int *)source;
    dst = (int *)cinTable[currentHandle].buf2;

    for(x=0;x<256*256;x++) {
        *dst = *src;
        dst++; src += 2;
    }
    return cinTable[currentHandle].buf2;
}
*/

static fn RoQReset() {
    unsafe {
        if *core::ptr::addr_of_mut!(currentHandle) < 0 {
            return;
        }

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile != 0 {
            Sys_EndStreamedFile((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile);
            FS_Seek((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile, 0, FS_SEEK_SET);
            FS_Read(
                (*core::ptr::addr_of_mut!(cin)).file.as_mut_ptr() as *mut c_void,
                16,
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile,
            );
            RoQ_init();
            // let the background thread start reading ahead
            Sys_BeginStreamedFile((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile, 0x10000);
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_LOOPED;
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn RoQInterrupt() {
    let mut framedata: *mut u8;
    let mut sbuf: [i16; 32768] = [0; 32768];
    let mut ssize: c_int;

    unsafe {
        if *core::ptr::addr_of_mut!(currentHandle) < 0 {
            return;
        }

        Sys_StreamedRead(
            (*core::ptr::addr_of_mut!(cin)).file.as_mut_ptr() as *mut c_void,
            ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize + 8) as usize,
            1,
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile,
        );

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQPlayed
            >= (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ROQSize
        {
            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).holdAtEnd == qfalse {
                if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).looping != 0 {
                    RoQReset();
                } else {
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_EOF;
                }
            } else {
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_IDLE;
            }
            return;
        }

        framedata = (*core::ptr::addr_of_mut!(cin)).file.as_mut_ptr();
        //
        // new frame is ready
        //
        'redump: loop {
            match (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_id {
                ROQ_QUAD_VQ as u32 => {
                    if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads & 1 != 0 {
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).normalBuffer0 =
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).t[1];
                        RoQPrepMcomp(
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqF0,
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqF1,
                        );
                        if let Some(vq1) = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQ1 {
                            vq1(
                                (*core::ptr::addr_of_mut!(cin)).qStatus[1].as_mut_ptr() as *mut u8,
                                framedata as *mut c_void,
                            );
                        }
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).buf =
                            (*core::ptr::addr_of_mut!(cin)).linbuf.as_mut_ptr().add(
                                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).screenDelta as usize,
                            );
                    } else {
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).normalBuffer0 =
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).t[0];
                        RoQPrepMcomp(
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqF0,
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqF1,
                        );
                        if let Some(vq0) = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).VQ0 {
                            vq0(
                                (*core::ptr::addr_of_mut!(cin)).qStatus[0].as_mut_ptr() as *mut u8,
                                framedata as *mut c_void,
                            );
                        }
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).buf =
                            (*core::ptr::addr_of_mut!(cin)).linbuf.as_mut_ptr();
                    }
                    if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads == 0 {
                        // first frame
                        Com_Memcpy(
                            (*core::ptr::addr_of_mut!(cin)).linbuf.as_mut_ptr().add(
                                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).screenDelta as usize,
                            ) as *mut c_void,
                            (*core::ptr::addr_of_mut!(cin)).linbuf.as_ptr() as *const c_void,
                            ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).samplesPerLine
                                * (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ysize as c_int)
                                as usize,
                        );
                    }
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads += 1;
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).dirty = qtrue;
                }
                ROQ_CODEBOOK as u32 => {
                    decodeCodeBook(framedata, (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_flags as u16);
                }
                ZA_SOUND_MONO as u32 => {
                    if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).silent == 0 {
                        ssize = RllDecodeMonoToStereo(
                            framedata,
                            sbuf.as_mut_ptr(),
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize,
                            0,
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_flags as u16,
                        );
                        S_RawSamples(
                            ssize,
                            22050,
                            2,
                            1,
                            sbuf.as_ptr() as *const c_char,
                            (*s_volume).value,
                            1,
                        );
                    }
                }
                ZA_SOUND_STEREO as u32 => {
                    if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).silent == 0 {
                        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads == -1 {
                            S_Update();
                            s_rawend = s_soundtime;
                        }
                        ssize = RllDecodeStereoToStereo(
                            framedata,
                            sbuf.as_mut_ptr(),
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize,
                            0,
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_flags as u16,
                        );
                        S_RawSamples(
                            ssize,
                            22050,
                            2,
                            2,
                            sbuf.as_ptr() as *const c_char,
                            (*s_volume).value,
                            1,
                        );
                    }
                }
                ROQ_QUAD_INFO as u32 => {
                    if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads == -1 {
                        readQuadInfo(framedata);
                        setupQuad(0, 0);
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime =
                            (Sys_Milliseconds() as f32 * (*com_timescale).value) as u32;
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).lastTime =
                            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime;
                    }
                    if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads != 1 {
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads = 0;
                    }
                }
                ROQ_PACKET as u32 => {
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).inMemory =
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_flags;
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize = 0;
                }
                ROQ_QUAD_HANG as u32 => {
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize = 0;
                }
                ROQ_QUAD_JPEG as u32 => {}
                _ => {
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_EOF;
                }
            }

            //
            // read in next frame data
            //
            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQPlayed
                >= (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ROQSize
            {
                if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).holdAtEnd == qfalse {
                    if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).looping != 0 {
                        RoQReset();
                    } else {
                        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_EOF;
                    }
                } else {
                    (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_IDLE;
                }
                return;
            }

            framedata = framedata.add((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize as usize);
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_id =
                (*framedata) as u32 + ((*framedata.add(1)) as u32) * 256;
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize =
                ((*framedata.add(2)) as u32) + ((*framedata.add(3)) as u32) * 256 + ((*framedata.add(4)) as u32) * 65536;
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_flags =
                ((*framedata.add(6)) as i32) + ((*framedata.add(7)) as i32) * 256;
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqF0 =
                (*framedata.add(7)) as c_char as i32;
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqF1 =
                (*framedata.add(6)) as c_char as i32;

            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize > 65536
                || (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_id == 0x1084
            {
                Com_DPrintf(b"roq_size>65536||roq_id==0x1084\n\0".as_ptr() as *const c_char);
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_EOF;
                if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).looping != 0 {
                    RoQReset();
                }
                return;
            }
            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).inMemory != 0
                && ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status as u32) != (e_status::FMV_EOF as u32)
            {
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).inMemory =
                    ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).inMemory - 1) as c_int;
                framedata = framedata.add(8);
                break 'redump;
            }
            //
            // one more frame hits the dust
            //
            //	assert(cinTable[currentHandle].RoQFrameSize <= 65536);
            //	r = Sys_StreamedRead( cin.file, cinTable[currentHandle].RoQFrameSize+8, 1, cinTable[currentHandle].iFile );
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQPlayed +=
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize as i32 + 8;

            break;
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn RoQ_init() {
    unsafe {
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime =
            (Sys_Milliseconds() as f32 * (*com_timescale).value) as u32;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).lastTime =
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime;

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQPlayed = 24;

        /*	get frame rate */
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqFPS =
            ((*core::ptr::addr_of_mut!(cin)).file[6] as i32) + ((*core::ptr::addr_of_mut!(cin)).file[7] as i32) * 256;

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqFPS == 0 {
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqFPS = 30;
        }

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads = -1;

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_id =
            ((*core::ptr::addr_of_mut!(cin)).file[8] as u32) + ((*core::ptr::addr_of_mut!(cin)).file[9] as u32) * 256;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize =
            ((*core::ptr::addr_of_mut!(cin)).file[10] as u32) + ((*core::ptr::addr_of_mut!(cin)).file[11] as u32) * 256
                + ((*core::ptr::addr_of_mut!(cin)).file[12] as u32) * 65536;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roq_flags =
            ((*core::ptr::addr_of_mut!(cin)).file[14] as i32) + ((*core::ptr::addr_of_mut!(cin)).file[15] as i32) * 256;

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize > 65536
            || (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).RoQFrameSize == 0
        {
            return;
        }
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

static fn RoQShutdown() {
    let mut s: *const c_char;

    unsafe {
        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).buf.is_null() {
            return;
        }

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status as u32 == (e_status::FMV_IDLE as u32) {
            return;
        }
        Com_DPrintf(b"finished cinematic\n\0".as_ptr() as *const c_char);
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_IDLE;

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile != 0 {
            Sys_EndStreamedFile((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile);
            FS_FCloseFile((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile);
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile = 0;
        }

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).alterGameState != 0 {
            cls.state = CA_DISCONNECTED;
            // we can't just do a vstr nextmap, because
            // if we are aborting the intro cinematic with
            // a devmap command, nextmap would be valid by
            // the time it was referenced
            s = Cvar_VariableString(b"nextmap\0".as_ptr() as *const c_char);
            if *s != 0 {
                let mut va_buf: [c_char; 512] = [0; 512];
                // Approximation of va() function
                let fmt = b"%s\n\0".as_ptr() as *const c_char;
                Com_sprintf(va_buf.as_mut_ptr(), 512, fmt, s);
                Cbuf_ExecuteText(EXEC_APPEND, va_buf.as_ptr());
                Cvar_Set(b"nextmap\0".as_ptr() as *const c_char, b"\0".as_ptr() as *const c_char);
            }
            CL_handle = -1;
        }
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).fileName[0] = 0;
        *core::ptr::addr_of_mut!(currentHandle) = -1;
    }
}

/*
==================
SCR_StopCinematic
==================
*/
pub fn CIN_StopCinematic(handle: c_int) -> e_status {
    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES || (*core::ptr::addr_of_mut!(cinTable[handle as usize])).status as u32 == (e_status::FMV_EOF as u32) {
            return e_status::FMV_EOF;
        }
        *core::ptr::addr_of_mut!(currentHandle) = handle;

        Com_DPrintf(
            b"trFMV::stop(), closing %s\n\0".as_ptr() as *const c_char,
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).fileName.as_ptr(),
        );

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).buf.is_null() {
            return e_status::FMV_EOF;
        }

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).alterGameState != 0 {
            if cls.state != CA_CINEMATIC {
                return (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status;
            }
        }
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_EOF;
        RoQShutdown();

        e_status::FMV_EOF
    }
}

/*
==================
SCR_RunCinematic

Fetch and decompress the pending frame
==================
*/

pub fn CIN_RunCinematic(handle: c_int) -> e_status {
    // bk001204 - init
    let mut start: c_int = 0;
    let mut thisTime: c_int = 0;

    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES || (*core::ptr::addr_of_mut!(cinTable[handle as usize])).status as u32 == (e_status::FMV_EOF as u32) {
            return e_status::FMV_EOF;
        }

        if *core::ptr::addr_of_mut!(currentHandle) != handle {
            *core::ptr::addr_of_mut!(currentHandle) = handle;
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_EOF;
            RoQReset();
        }

        if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls < -1 {
            return (*core::ptr::addr_of_mut!(cinTable[handle as usize])).status;
        }

        *core::ptr::addr_of_mut!(currentHandle) = handle;

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).alterGameState != 0 {
            if cls.state != CA_CINEMATIC {
                return (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status;
            }
        }

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status as u32 == (e_status::FMV_IDLE as u32) {
            return (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status;
        }

        thisTime = (Sys_Milliseconds() as f32 * (*com_timescale).value) as c_int;
        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).shader != 0
            && (((thisTime - (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).lastTime as c_int).abs()) > 100)
        {
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime =
                ((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime as c_int
                    + (thisTime - (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).lastTime as c_int))
                    as u32;
        }
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).tfps =
            (((Sys_Milliseconds() as f32 * (*com_timescale).value) as c_int
                - (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime as c_int)
                * (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqFPS)
                / 1000;

        start = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime as c_int;
        while (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).tfps
            != (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).numQuads
            && (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status as u32 == (e_status::FMV_PLAY as u32)
        {
            RoQInterrupt();
            if start != (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime as c_int {
                (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).tfps =
                    (((Sys_Milliseconds() as f32 * (*com_timescale).value) as c_int
                        - (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime as c_int)
                        * (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).roqFPS)
                        / 1000;
                start = (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).startTime as c_int;
            }
        }

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).lastTime = thisTime as u32;

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status as u32 == (e_status::FMV_LOOPED as u32) {
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_PLAY;
        }

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status as u32 == (e_status::FMV_EOF as u32) {
            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).looping != 0 {
                RoQReset();
            } else {
                RoQShutdown();
            }
        }

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status
    }
}

/*
==================
CL_PlayCinematic

==================
*/
pub fn CIN_PlayCinematic(arg: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, systemBits: c_int) -> c_int {
    let mut RoQID: u16;
    let mut name: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    let mut i: c_int;

    unsafe {
        if strstr(arg, b"/\0".as_ptr() as *const c_char).is_null()
            && strstr(arg, b"\\\0".as_ptr() as *const c_char).is_null()
        {
            Com_sprintf(name.as_mut_ptr(), MAX_OSPATH, b"video/%s\0".as_ptr() as *const c_char, arg);
        } else {
            Com_sprintf(name.as_mut_ptr(), MAX_OSPATH, b"%s\0".as_ptr() as *const c_char, arg);
        }
        COM_DefaultExtension(name.as_mut_ptr(), MAX_OSPATH, b".roq\0".as_ptr() as *const c_char);

        if (systemBits & CIN_system) == 0 {
            for i in 0..MAX_VIDEO_HANDLES {
                if strcmp((*core::ptr::addr_of_mut!(cinTable[i as usize])).fileName.as_ptr(), name.as_ptr()) == 0 {
                    return i;
                }
            }
        }

        Com_DPrintf(b"SCR_PlayCinematic( %s )\n\0".as_ptr() as *const c_char, arg);

        Com_Memset(core::ptr::addr_of_mut!(cin) as *mut c_void, 0, core::mem::size_of::<cinematics_t>());
        *core::ptr::addr_of_mut!(currentHandle) = CIN_HandleForVideo();

        strcpy(
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).fileName.as_mut_ptr(),
            name.as_ptr(),
        );

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ROQSize = 0;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ROQSize = FS_FOpenFileRead(
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).fileName.as_ptr(),
            &mut (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile,
            1,
        );

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).ROQSize <= 0 {
            Com_DPrintf(b"cinematic failed to open %s\n\0".as_ptr() as *const c_char, arg);
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).fileName[0] = 0;
            return -1;
        }

        CIN_SetExtents(*core::ptr::addr_of_mut!(currentHandle), x, y, w, h);
        CIN_SetLooping(
            *core::ptr::addr_of_mut!(currentHandle),
            if (systemBits & CIN_loop) != 0 { qtrue } else { qfalse },
        );

        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_HEIGHT = DEFAULT_CIN_HEIGHT;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).CIN_WIDTH = DEFAULT_CIN_WIDTH;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).holdAtEnd =
            if (systemBits & CIN_hold) != 0 { qtrue } else { qfalse };
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).alterGameState =
            if (systemBits & CIN_system) != 0 { qtrue } else { qfalse };
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).playonwalls = 1;
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).silent =
            if (systemBits & CIN_silent) != 0 { qtrue } else { qfalse };
        (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).shader =
            if (systemBits & CIN_shader) != 0 { qtrue } else { qfalse };

        if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).alterGameState != 0 {
            // close the menu
            if !uivm.is_null() {
                VM_Call(uivm, UI_SET_ACTIVE_MENU, UIMENU_NONE);
            }
        } else {
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).playonwalls = (*cl_inGameVideo).integer;
        }

        initRoQ();

        FS_Read(
            (*core::ptr::addr_of_mut!(cin)).file.as_mut_ptr() as *mut c_void,
            16,
            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile,
        );

        RoQID = ((*core::ptr::addr_of_mut!(cin)).file[0] as u16) + ((*core::ptr::addr_of_mut!(cin)).file[1] as u16) * 256;
        if RoQID == 0x1084 {
            RoQ_init();
            //		FS_Read (cin.file, cinTable[currentHandle].RoQFrameSize+8, cinTable[currentHandle].iFile);
            // let the background thread start reading ahead
            Sys_BeginStreamedFile((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).iFile, 0x10000);

            (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status = e_status::FMV_PLAY;
            Com_DPrintf(b"trFMV::play(), playing %s\n\0".as_ptr() as *const c_char, arg);

            if (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).alterGameState != 0 {
                cls.state = CA_CINEMATIC;
            }

            Con_Close();

            s_rawend = s_soundtime;

            return *core::ptr::addr_of_mut!(currentHandle);
        }
        Com_DPrintf(b"trFMV::play(), invalid RoQ ID\n\0".as_ptr() as *const c_char);

        RoQShutdown();
        -1
    }
}

pub fn CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES || (*core::ptr::addr_of_mut!(cinTable[handle as usize])).status as u32 == (e_status::FMV_EOF as u32) {
            return;
        }
        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).xpos = x;
        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).ypos = y;
        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).width = w;
        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).height = h;
        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty = qtrue;
    }
}

pub fn CIN_SetLooping(handle: c_int, loop_: c_int) {
    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES || (*core::ptr::addr_of_mut!(cinTable[handle as usize])).status as u32 == (e_status::FMV_EOF as u32) {
            return;
        }
        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).looping = loop_;
    }
}

/*
==================
SCR_DrawCinematic

==================
*/
pub fn CIN_DrawCinematic(handle: c_int) {
    let mut x: f32;
    let mut y: f32;
    let mut w: f32;
    let mut h: f32;

    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES || (*core::ptr::addr_of_mut!(cinTable[handle as usize])).status as u32 == (e_status::FMV_EOF as u32) {
            return;
        }

        if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).buf.is_null() {
            return;
        }

        x = (*core::ptr::addr_of_mut!(cinTable[handle as usize])).xpos as f32;
        y = (*core::ptr::addr_of_mut!(cinTable[handle as usize])).ypos as f32;
        w = (*core::ptr::addr_of_mut!(cinTable[handle as usize])).width as f32;
        h = (*core::ptr::addr_of_mut!(cinTable[handle as usize])).height as f32;

        if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty != 0
            && ((*core::ptr::addr_of_mut!(cinTable[handle as usize])).CIN_WIDTH
                != (*core::ptr::addr_of_mut!(cinTable[handle as usize])).drawX
                || (*core::ptr::addr_of_mut!(cinTable[handle as usize])).CIN_HEIGHT
                    != (*core::ptr::addr_of_mut!(cinTable[handle as usize])).drawY)
        {
            let mut ix: c_int;
            let mut iy: c_int;
            let mut buf2: *mut i32;
            let mut buf3: *mut i32;
            let mut xm: c_int;
            let mut ym: c_int;
            let mut ll: c_int;

            xm = (*core::ptr::addr_of_mut!(cinTable[handle as usize])).CIN_WIDTH / 256;
            ym = (*core::ptr::addr_of_mut!(cinTable[handle as usize])).CIN_HEIGHT / 256;
            ll = 8;
            if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).CIN_WIDTH == 512 {
                ll = 9;
            }

            buf3 = (*core::ptr::addr_of_mut!(cinTable[handle as usize])).buf as *mut i32;
            buf2 = Hunk_AllocateTempMemory(256 * 256 * 4) as *mut i32;

            if xm == 2 && ym == 2 {
                let mut bc2: *mut u8;
                let mut bc3: *mut u8;
                let mut ic: c_int;
                let mut iiy: c_int;

                bc2 = buf2 as *mut u8;
                bc3 = buf3 as *mut u8;
                for iy in 0..256 {
                    iiy = iy << 12;
                    let mut ix = 0;
                    while ix < 2048 {
                        for ic in ix..(ix + 4) {
                            *bc2 = ((*bc3.add((iiy + ic) as usize) as c_int
                                + *bc3.add((iiy + 4 + ic) as usize) as c_int
                                + *bc3.add((iiy + 2048 + ic) as usize) as c_int
                                + *bc3.add((iiy + 2048 + 4 + ic) as usize) as c_int)
                                >> 2) as u8;
                            bc2 = bc2.add(1);
                        }
                        ix += 8;
                    }
                }
            } else if xm == 2 && ym == 1 {
                let mut bc2: *mut u8;
                let mut bc3: *mut u8;
                let mut ic: c_int;
                let mut iiy: c_int;

                bc2 = buf2 as *mut u8;
                bc3 = buf3 as *mut u8;
                for iy in 0..256 {
                    iiy = iy << 11;
                    let mut ix = 0;
                    while ix < 2048 {
                        for ic in ix..(ix + 4) {
                            *bc2 = ((*bc3.add((iiy + ic) as usize) as c_int + *bc3.add((iiy + 4 + ic) as usize) as c_int) >> 1) as u8;
                            bc2 = bc2.add(1);
                        }
                        ix += 8;
                    }
                }
            } else {
                for iy in 0..256 {
                    for ix in 0..256 {
                        *buf2.add(((iy << 8) + ix) as usize) =
                            *buf3.add((((iy * ym) << ll) + (ix * xm)) as usize);
                    }
                }
            }
            re_DrawStretchRaw(x, y, w, h, 256, 256, buf2 as *const c_char, handle, true);
            (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty = qfalse;
            Hunk_FreeTempMemory(buf2 as *mut c_void);
            return;
        }

        re_DrawStretchRaw(
            x,
            y,
            w,
            h,
            (*core::ptr::addr_of_mut!(cinTable[handle as usize])).drawX,
            (*core::ptr::addr_of_mut!(cinTable[handle as usize])).drawY,
            (*core::ptr::addr_of_mut!(cinTable[handle as usize])).buf as *const c_char,
            handle,
            (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty != 0,
        );
        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty = qfalse;
    }
}

pub fn CL_PlayCinematic_f() {
    let mut arg: *const c_char;
    let mut s: *const c_char;
    let mut bits: c_int = CIN_system;

    unsafe {
        Com_DPrintf(b"CL_PlayCinematic_f\n\0".as_ptr() as *const c_char);
        if cls.state == CA_CINEMATIC {
            SCR_StopCinematic();
        }

        arg = Cmd_Argv(1);
        s = Cmd_Argv(2);

        if ((s != core::ptr::null()) && (*s as c_char == '1' as c_char))
            || (Q_stricmp(arg, b"demoend.roq\0".as_ptr() as *const c_char) == 0)
            || (Q_stricmp(arg, b"end.roq\0".as_ptr() as *const c_char) == 0)
        {
            bits |= CIN_hold;
        }
        if (s != core::ptr::null()) && (*s as c_char == '2' as c_char) {
            bits |= CIN_loop;
        }

        S_StopAllSounds();

        CL_handle = CIN_PlayCinematic(arg, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, bits);
        if CL_handle >= 0 {
            loop {
                SCR_RunCinematic();
                if !((*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).buf.is_null()
                    && (*core::ptr::addr_of_mut!(cinTable[*core::ptr::addr_of_mut!(currentHandle) as usize])).status as u32
                        == (e_status::FMV_PLAY as u32))
                {
                    break;
                }
            }
        } else {
            let mut va_buf: [c_char; 512] = [0; 512];
            Com_sprintf(
                va_buf.as_mut_ptr(),
                512,
                b"%s%s%s\n\0".as_ptr() as *const c_char,
                b"\x1b[31m\0".as_ptr() as *const c_char,
                b"PlayCinematic(): Failed to open \"\0".as_ptr() as *const c_char,
                arg,
            );
            Com_Printf(va_buf.as_ptr());
        }
    }
}

pub fn SCR_DrawCinematic() {
    unsafe {
        if CL_handle >= 0 && CL_handle < MAX_VIDEO_HANDLES {
            CIN_DrawCinematic(CL_handle);
        }
    }
}

pub fn SCR_RunCinematic() {
    unsafe {
        if CL_handle >= 0 && CL_handle < MAX_VIDEO_HANDLES {
            CIN_RunCinematic(CL_handle);
        }
    }
}

pub fn SCR_StopCinematic() {
    unsafe {
        if CL_handle >= 0 && CL_handle < MAX_VIDEO_HANDLES {
            CIN_StopCinematic(CL_handle);
            S_StopAllSounds();
            CL_handle = -1;
        }
    }
}

pub fn CIN_UploadCinematic(handle: c_int) {
    unsafe {
        if handle >= 0 && handle < MAX_VIDEO_HANDLES {
            if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).buf.is_null() {
                return;
            }
            if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls <= 0
                && (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty != 0
            {
                if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls == 0 {
                    (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls = -1;
                } else {
                    if (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls == -1 {
                        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls = -2;
                    } else {
                        (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty = qfalse;
                    }
                }
            }
            re_UploadCinematic(
                (*core::ptr::addr_of_mut!(cinTable[handle as usize])).drawX,
                (*core::ptr::addr_of_mut!(cinTable[handle as usize])).drawY,
                (*core::ptr::addr_of_mut!(cinTable[handle as usize])).buf as *const c_char,
                handle,
                (*core::ptr::addr_of_mut!(cinTable[handle as usize])).dirty != 0,
            );
            if (*cl_inGameVideo).integer == 0 && (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls == 1 {
                (*core::ptr::addr_of_mut!(cinTable[handle as usize])).playonwalls -= 1;
            }
        }
    }
}
