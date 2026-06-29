// leave this as first line for PCH reasons...
//

/*****************************************************************************
 * name:		cl_cin.c
 *
 * desc:		video and cinematic playback
 *
 * $Archive: /MissionPack/code/client/cl_cin.c $
 * $Author: Ttimo $
 * $Revision: 82 $
 * $Modtime: 4/13/01 4:48p $
 * $Date: 4/13/01 4:48p $
 *
 * cl_glconfig.hwtype trtypes 3dfx/ragepro need 256x256
 *
 *****************************************************************************/

use core::ffi::{c_int, c_uint, c_char, c_void, c_uchar};
use core::mem;
use core::ptr;

#[allow(non_snake_case)]

const MAXSIZE: c_int = 8;
const MINSIZE: c_int = 4;

const DEFAULT_CIN_WIDTH: c_int = 512;
const DEFAULT_CIN_HEIGHT: c_int = 512;

const ROQ_QUAD: c_uint = 0x1000;
const ROQ_QUAD_INFO: c_uint = 0x1001;
const ROQ_CODEBOOK: c_uint = 0x1002;
const ROQ_QUAD_VQ: c_uint = 0x1011;
const ROQ_QUAD_JPEG: c_uint = 0x1012;
const ROQ_QUAD_HANG: c_uint = 0x1013;
const ROQ_PACKET: c_uint = 0x1030;
const ZA_SOUND_MONO: c_uint = 0x1020;
const ZA_SOUND_STEREO: c_uint = 0x1021;

const MAX_VIDEO_HANDLES: usize = 16;

// Stub types for external dependencies
type glconfig_t = c_void;
type fileHandle_t = c_int;
type qboolean = c_uchar;
type e_status = c_int;
type sfxHandle_t = c_int;
type qhandle_t = c_int;
type portable_samplepair_t = c_void;
type refdef_t = c_void;
type polyVert_t = c_void;

extern "C" {
    static glConfig: glconfig_t;
    static mut s_paintedtime: c_int;
    static mut s_rawend: c_int;

    fn S_CIN_StopSound(sfxHandle: sfxHandle_t);
}

static RoQ_init: extern "C" fn() = roq_init_internal;

/******************************************************************************
*
* Class:		trFMV
*
* Description:	RoQ/RnR manipulation routines
*				not entirely complete for first run
*
******************************************************************************/

#[repr(C)]
struct cinematics_t {
    linbuf: [c_uchar; (DEFAULT_CIN_WIDTH as usize) * (DEFAULT_CIN_HEIGHT as usize) * 4 * 2],
    file: [c_uchar; 65536],
    sqrTable: [i16; 256],

    mcomp: [c_uint; 256],
    vq2: [u16; 256*16*4],
    vq4: [u16; 256*64*4],
    vq8: [u16; 256*256*4],

    ROQ_YY_tab: [c_int; 256],
    ROQ_UB_tab: [c_int; 256],
    ROQ_UG_tab: [c_int; 256],
    ROQ_VG_tab: [c_int; 256],
    ROQ_VR_tab: [c_int; 256],

    qStatus: [[*mut c_uchar; 32768]; 2],

    oldXOff: c_int,
    oldYOff: c_int,
    oldysize: c_uint,
    oldxsize: c_uint,
}

#[repr(C)]
struct cin_cache {
    fileName: [c_char; 260], // MAX_OSPATH
    CIN_WIDTH: c_int,
    CIN_HEIGHT: c_int,
    xpos: c_int,
    ypos: c_int,
    width: c_int,
    height: c_int,
    looping: qboolean,
    holdAtEnd: qboolean,
    dirty: qboolean,
    alterGameState: qboolean,
    silent: qboolean,
    shader: qboolean,
    iFile: fileHandle_t,
    status: e_status,
    startTime: c_uint,
    lastTime: c_uint,
    tfps: c_int,
    RoQPlayed: c_int,
    ROQSize: c_int,
    RoQFrameSize: c_uint,
    onQuad: c_int,
    numQuads: c_int,
    samplesPerLine: c_int,
    roq_id: c_uint,
    screenDelta: c_int,

    VQ0: Option<extern "C" fn(*mut c_uchar, *mut c_void)>,
    VQ1: Option<extern "C" fn(*mut c_uchar, *mut c_void)>,
    VQNormal: Option<extern "C" fn(*mut c_uchar, *mut c_void)>,
    VQBuffer: Option<extern "C" fn(*mut c_uchar, *mut c_void)>,

    gray: *mut c_uchar,
    xsize: c_uint,
    ysize: c_uint,
    maxsize: c_uint,
    minsize: c_uint,

    inMemory: qboolean,
    normalBuffer0: c_int,
    roq_flags: c_int,
    roqF0: c_int,
    roqF1: c_int,
    t: [c_int; 2],
    roqFPS: c_int,
    playonwalls: c_int,
    buf: *mut c_uchar,
    drawX: c_int,
    drawY: c_int,
    hSFX: sfxHandle_t,
    hCRAWLTEXT: qhandle_t,
}

static mut cin: cinematics_t = unsafe { mem::zeroed() };
static mut cinTable: [cin_cache; MAX_VIDEO_HANDLES] = unsafe { mem::zeroed() };
static mut currentHandle: c_int = -1;
static mut CL_handle: c_int = -1;
static mut CL_iPlaybackStartTime: c_int = 0;

extern "C" {
    static mut s_soundtime: c_int;
}

pub extern "C" fn CIN_CloseAllVideos() {
    let mut i: c_int = 0;

    while i < MAX_VIDEO_HANDLES as c_int {
        unsafe {
            if cinTable[i as usize].fileName[0] != 0 {
                CIN_StopCinematic(i);
            }
        }
        i += 1;
    }
}

fn CIN_HandleForVideo() -> c_int {
    let mut i: c_int = 0;

    // these end up in scratchImage[NUM_SCRATCH_IMAGES], so MAX_VIDEO_HANDLES should match
    // assert (MAX_VIDEO_HANDLES<=NUM_SCRATCH_IMAGES);

    while i < MAX_VIDEO_HANDLES as c_int {
        unsafe {
            if cinTable[i as usize].fileName[0] == 0 {
                return i;
            }
        }
        i += 1;
    }
    // Com_Error( ERR_DROP, "CIN_HandleForVideo: none free" );
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
fn RllSetupTable() {
    let mut z: c_int = 0;

    unsafe {
        if currentHandle < 0 { return; }

        while z < 128 {
            cin.sqrTable[z as usize] = (z * z) as i16;
            cin.sqrTable[(z + 128) as usize] = -cin.sqrTable[z as usize];
            z += 1;
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
/*
fn RllDecodeMonoToMono(from: *mut c_uchar, to: *mut i16, size: c_uint, signedOutput: c_char, flag: u16) -> c_int {
    let mut z: c_uint = 0;
    let mut prev: c_int = 0;

    unsafe {
        if currentHandle < 0 { return 0; }

        if signedOutput != 0 {
            prev = (flag as c_int) - 0x8000;
        } else {
            prev = flag as c_int;
        }

        while z < size {
            prev = (prev + cin.sqrTable[(*from.add(z as usize)) as usize] as c_int) as c_int;
            *to.add(z as usize) = prev as i16;
            z += 1;
        }
    }
    size as c_int
}
*/

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
extern "C" {
    fn S_GetRawSamplePointer() -> *mut portable_samplepair_t;
    fn S_Update();
    static mut dma: c_void;
    static mut s_volume: *mut c_void;
}

fn RllDecodeMonoToStereo(from: *mut c_uchar, size: c_uint, signedOutput: c_char, flag: u16, bMixedWithCurrentAudio: qboolean) -> c_int {
    let mut z: c_uint = 0;
    let mut prev: c_int = 0;
    let mut dst: c_int = 0;
    let samps: *mut portable_samplepair_t;

    unsafe {
        if currentHandle < 0 { return 0; }

        if signedOutput != 0 {
            prev = (flag as c_int) - 0x8000;
        } else {
            prev = flag as c_int;
        }

        samps = S_GetRawSamplePointer();

        let iVolume: c_int = if bMixedWithCurrentAudio != 0 {
            ((256.0f32 * (if !s_volume.is_null() { 1.0f32 } else { 1.0f32 })) as c_int)
        } else {
            256
        };

        while z < size {
            dst = (s_rawend & (65536 - 1)) as c_int;
            s_rawend += 1;
            prev = (prev + cin.sqrTable[(*from.add(z as usize)) as usize] as c_int) as c_int;
            // samps[dst].left = samps[dst].right = prev*iVolume;
            z += 1;
        }
    }

    size as c_int
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
fn RllDecodeStereoToStereo(from: *mut c_uchar, size: c_uint, signedOutput: c_char, flag: u16, bMixedWithCurrentAudio: qboolean) -> c_int {
    let mut z: c_uint = 0;
    let mut zz: *mut c_uchar;
    let mut prevL: c_int = 0;
    let mut prevR: c_int = 0;
    let mut dst: c_int = 0;
    let samps: *mut portable_samplepair_t;

    unsafe {
        if currentHandle < 0 { return 0; }

        zz = from;

        if signedOutput != 0 {
            prevL = ((flag as c_int) & 0xff00) - 0x8000;
            prevR = (((flag as c_int) & 0x00ff) << 8) - 0x8000;
        } else {
            prevL = (flag as c_int) & 0xff00;
            prevR = ((flag as c_int) & 0x00ff) << 8;
        }

        samps = S_GetRawSamplePointer();

        let iVolume: c_int = if bMixedWithCurrentAudio != 0 {
            ((256.0f32 * (if !s_volume.is_null() { 1.0f32 } else { 1.0f32 })) as c_int)
        } else {
            256
        };

        // TODO: implement proper dma.speed checking
        // For now, simplified to case 22050
        while z < size {
            z = z.wrapping_add(2);
            dst = (s_rawend & (65536 - 1)) as c_int;
            s_rawend += 1;
            prevL = (prevL + cin.sqrTable[(*zz) as usize] as c_int) as c_int;
            zz = zz.add(1);
            prevR = (prevR + cin.sqrTable[(*zz) as usize] as c_int) as c_int;
            zz = zz.add(1);
            // samps[dst].left  = prevL*iVolume;
            // samps[dst].right = prevR*iVolume;
        }
    }

    (size >> 1) as c_int
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
/*
fn RllDecodeStereoToMono(from: *mut c_uchar, to: *mut i16, size: c_uint, signedOutput: c_char, flag: u16) -> c_int {
    let mut z: c_uint = 0;
    let mut prevL: c_int = 0;
    let mut prevR: c_int = 0;

    unsafe {
        if currentHandle < 0 { return 0; }

        if signedOutput != 0 {
            prevL = ((flag as c_int) & 0xff00) - 0x8000;
            prevR = (((flag as c_int) & 0x00ff) << 8) - 0x8000;
        } else {
            prevL = (flag as c_int) & 0xff00;
            prevR = ((flag as c_int) & 0x00ff) << 8;
        }

        while z < size {
            prevL = prevL + cin.sqrTable[(*from.add(z as usize * 2)) as usize] as c_int;
            prevR = prevR + cin.sqrTable[(*from.add(z as usize * 2 + 1)) as usize] as c_int;
            *to.add(z as usize) = ((prevL + prevR) / 2) as i16;
            z += 1;
        }
    }

    size as c_int
}
*/

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

fn move8_32(src: *mut c_uchar, dst: *mut c_uchar, spl: c_int) {
    let mut dsrc: *mut f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *mut f64;
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

fn move4_32(src: *mut c_uchar, dst: *mut c_uchar, spl: c_int) {
    let mut dsrc: *mut f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *mut f64;
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

fn blit8_32(src: *mut c_uchar, dst: *mut c_uchar, spl: c_int) {
    let mut dsrc: *mut f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *mut f64;
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

fn blit4_32(src: *mut c_uchar, dst: *mut c_uchar, spl: c_int) {
    let mut dsrc: *mut f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *mut f64;
        ddst = dst as *mut f64;
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

fn blit2_32(src: *mut c_uchar, dst: *mut c_uchar, spl: c_int) {
    let mut dsrc: *mut f64;
    let mut ddst: *mut f64;
    let mut dspl: c_int;

    unsafe {
        dsrc = src as *mut f64;
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

fn blitVQQuad32fs(status: *mut *mut c_uchar, data: *mut c_uchar) {
    let mut newd: u16 = 0;
    let mut celdata: u16 = 0;
    let mut code: u16 = 0;
    let mut index: c_uint = 0;
    let mut i: c_uint = 0;
    let mut data_mut = data;

    newd = 0;
    celdata = 0;
    index = 0;

    unsafe {
        if currentHandle < 0 { return; }

        loop {
            if newd == 0 {
                newd = 7;
                celdata = (*data_mut as u16) + ((*data_mut.add(1) as u16) << 8);
                data_mut = data_mut.add(2);
            } else {
                newd -= 1;
            }

            code = celdata & 0xc000;
            celdata = celdata << 2;

            match code {
                0x8000 => {
                    // vq code
                    blit8_32(
                        &mut cin.vq8[(*data_mut as usize) * 128] as *mut u16 as *mut c_uchar,
                        *status.add(index as usize),
                        cinTable[currentHandle as usize].samplesPerLine
                    );
                    data_mut = data_mut.add(1);
                    index += 5;
                }
                0xc000 => {
                    // drop
                    index += 1;
                    for _ in 0..4 {
                        if newd == 0 {
                            newd = 7;
                            celdata = (*data_mut as u16) + ((*data_mut.add(1) as u16) << 8);
                            data_mut = data_mut.add(2);
                        } else {
                            newd -= 1;
                        }

                        code = celdata & 0xc000;
                        celdata = celdata << 2;

                        match code {
                            0x8000 => {
                                // 4x4 vq code
                                blit4_32(
                                    &mut cin.vq4[(*data_mut as usize) * 32] as *mut u16 as *mut c_uchar,
                                    *status.add(index as usize),
                                    cinTable[currentHandle as usize].samplesPerLine
                                );
                                data_mut = data_mut.add(1);
                            }
                            0xc000 => {
                                // 2x2 vq code
                                blit2_32(
                                    &mut cin.vq2[(*data_mut as usize) * 8] as *mut u16 as *mut c_uchar,
                                    *status.add(index as usize),
                                    cinTable[currentHandle as usize].samplesPerLine
                                );
                                data_mut = data_mut.add(1);
                                blit2_32(
                                    &mut cin.vq2[(*data_mut as usize) * 8] as *mut u16 as *mut c_uchar,
                                    (*status.add(index as usize)).add(8),
                                    cinTable[currentHandle as usize].samplesPerLine
                                );
                                data_mut = data_mut.add(1);
                                blit2_32(
                                    &mut cin.vq2[(*data_mut as usize) * 8] as *mut u16 as *mut c_uchar,
                                    (*status.add(index as usize)).add((cinTable[currentHandle as usize].samplesPerLine * 2) as usize),
                                    cinTable[currentHandle as usize].samplesPerLine
                                );
                                data_mut = data_mut.add(1);
                                blit2_32(
                                    &mut cin.vq2[(*data_mut as usize) * 8] as *mut u16 as *mut c_uchar,
                                    (*status.add(index as usize)).add(((cinTable[currentHandle as usize].samplesPerLine * 2) + 8) as usize),
                                    cinTable[currentHandle as usize].samplesPerLine
                                );
                                data_mut = data_mut.add(1);
                            }
                            0x4000 => {
                                // motion compensation
                                move4_32(
                                    (*status.add(index as usize)).add(cin.mcomp[*data_mut as usize] as usize),
                                    *status.add(index as usize),
                                    cinTable[currentHandle as usize].samplesPerLine
                                );
                                data_mut = data_mut.add(1);
                            }
                            _ => {}
                        }
                        index += 1;
                    }
                }
                0x4000 => {
                    // motion compensation
                    move8_32(
                        (*status.add(index as usize)).add(cin.mcomp[*data_mut as usize] as usize),
                        *status.add(index as usize),
                        cinTable[currentHandle as usize].samplesPerLine
                    );
                    data_mut = data_mut.add(1);
                    index += 5;
                }
                0x0000 => {
                    index += 5;
                }
                _ => {}
            }

            if (*status.add(index as usize)).is_null() {
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

fn ROQ_GenYUVTables() {
    let mut t_ub: f32;
    let mut t_vr: f32;
    let mut t_ug: f32;
    let mut t_vg: f32;
    let mut i: c_int = 0;

    unsafe {
        if currentHandle < 0 { return; }

        t_ub = (1.77200f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;
        t_vr = (1.40200f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;
        t_ug = (0.34414f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;
        t_vg = (0.71414f32 / 2.0f32) * ((1i32 << 6) as f32) + 0.5f32;

        while i < 256 {
            let x: f32 = ((2 * i) - 255) as f32;

            cin.ROQ_UB_tab[i as usize] = ((t_ub * x) + (1i32 << 5) as f32) as c_int;
            cin.ROQ_VR_tab[i as usize] = ((t_vr * x) + (1i32 << 5) as f32) as c_int;
            cin.ROQ_UG_tab[i as usize] = (-t_ug * x) as c_int;
            cin.ROQ_VG_tab[i as usize] = ((-t_vg * x) + (1i32 << 5) as f32) as c_int;
            cin.ROQ_YY_tab[i as usize] = ((i << 6) | (i >> 2)) as c_int;

            i += 1;
        }
    }
}

// VQ2TO4 and VQ2TO2 macros are implemented inline in decodeCodeBook

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/
/*
fn yuv_to_rgb(y: c_int, u: c_int, v: c_int) -> u16 {
    unsafe {
        let YY: c_int = cin.ROQ_YY_tab[y as usize];
        let mut r: c_int = (YY + cin.ROQ_VR_tab[v as usize]) >> 9;
        let mut g: c_int = (YY + cin.ROQ_UG_tab[u as usize] + cin.ROQ_VG_tab[v as usize]) >> 8;
        let mut b: c_int = (YY + cin.ROQ_UB_tab[u as usize]) >> 9;

        if r < 0 { r = 0; } if g < 0 { g = 0; } if b < 0 { b = 0; }
        if r > 31 { r = 31; } if g > 63 { g = 63; } if b > 31 { b = 31; }

        ((r << 11) + (g << 5) + b) as u16
    }
}
*/

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

fn yuv_to_rgb24(y: c_int, u: c_int, v: c_int) -> c_uint {
    unsafe {
        let YY: c_int = cin.ROQ_YY_tab[y as usize];
        let mut r: c_int = (YY + cin.ROQ_VR_tab[v as usize]) >> 6;
        let mut g: c_int = (YY + cin.ROQ_UG_tab[u as usize] + cin.ROQ_VG_tab[v as usize]) >> 6;
        let mut b: c_int = (YY + cin.ROQ_UB_tab[u as usize]) >> 6;

        if r < 0 { r = 0; } if g < 0 { g = 0; } if b < 0 { b = 0; }
        if r > 255 { r = 255; } if g > 255 { g = 255; } if b > 255 { b = 255; }

        ((r + (g << 8) + (b << 16)) as c_uint)
    }
}

extern "C" {
    fn LittleLong(x: c_uint) -> c_uint;
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

fn decodeCodeBook(input: *mut c_uchar, roq_flags: u16) {
    let mut i: c_int = 0;
    let mut j: c_int = 0;
    let mut two: c_int = 0;
    let mut four: c_int = 0;
    let mut bptr: *mut u16;
    let mut y0: c_int;
    let mut y1: c_int;
    let mut y2: c_int;
    let mut y3: c_int;
    let mut cr: c_int;
    let mut cb: c_int;
    let mut iaptr: *mut c_uint;
    let mut ibptr: *mut c_uint;
    let mut icptr: *mut c_uint;
    let mut idptr: *mut c_uint;
    let mut input_idx: usize = 0;

    unsafe {
        if currentHandle < 0 { return; }

        if roq_flags == 0 {
            two = 256;
            four = 256;
        } else {
            two = (roq_flags as c_int) >> 8;
            if two == 0 { two = 256; }
            four = (roq_flags as c_int) & 0xff;
        }

        four *= 2;

        bptr = cin.vq2.as_mut_ptr();

        // normal height
        ibptr = bptr as *mut c_uint;
        i = 0;
        while i < two {
            y0 = *input.add(input_idx) as c_int;
            input_idx += 1;
            y1 = *input.add(input_idx) as c_int;
            input_idx += 1;
            y2 = *input.add(input_idx) as c_int;
            input_idx += 1;
            y3 = *input.add(input_idx) as c_int;
            input_idx += 1;
            cr = *input.add(input_idx) as c_int;
            input_idx += 1;
            cb = *input.add(input_idx) as c_int;
            input_idx += 1;

            *ibptr.add(0) = yuv_to_rgb24(y0, cr, cb);
            *ibptr.add(1) = yuv_to_rgb24(y1, cr, cb);
            *ibptr.add(2) = yuv_to_rgb24(y2, cr, cb);
            *ibptr.add(3) = yuv_to_rgb24(y3, cr, cb);

            ibptr = ibptr.add(4);
            i += 1;
        }

        icptr = cin.vq4.as_mut_ptr() as *mut c_uint;
        idptr = cin.vq8.as_mut_ptr() as *mut c_uint;

        i = 0;
        while i < four {
            let idx1 = *input.add(input_idx) as usize;
            input_idx += 1;
            let idx2 = *input.add(input_idx) as usize;
            input_idx += 1;

            iaptr = cin.vq2.as_mut_ptr().add(idx1 * 4) as *mut c_uint;
            ibptr = cin.vq2.as_mut_ptr().add(idx2 * 4) as *mut c_uint;

            j = 0;
            while j < 2 {
                // VQ2TO4 macro - expand inline
                *icptr = *iaptr;
                icptr = icptr.add(1);
                *idptr = *iaptr;
                idptr = idptr.add(1);
                *idptr = *iaptr;
                idptr = idptr.add(1);

                *icptr = *iaptr.add(1);
                icptr = icptr.add(1);
                *idptr = *iaptr.add(1);
                idptr = idptr.add(1);
                *idptr = *iaptr.add(1);
                idptr = idptr.add(1);

                *icptr = *ibptr;
                icptr = icptr.add(1);
                *idptr = *ibptr;
                idptr = idptr.add(1);
                *idptr = *ibptr;
                idptr = idptr.add(1);

                *icptr = *ibptr.add(1);
                icptr = icptr.add(1);
                *idptr = *ibptr.add(1);
                idptr = idptr.add(1);
                *idptr = *ibptr.add(1);
                idptr = idptr.add(1);

                *idptr = *iaptr;
                idptr = idptr.add(1);
                *idptr = *iaptr;
                idptr = idptr.add(1);

                *idptr = *iaptr.add(1);
                idptr = idptr.add(1);
                *idptr = *iaptr.add(1);
                idptr = idptr.add(1);

                *idptr = *ibptr;
                idptr = idptr.add(1);
                *idptr = *ibptr;
                idptr = idptr.add(1);

                *idptr = *ibptr.add(1);
                idptr = idptr.add(1);
                *idptr = *ibptr.add(1);
                idptr = idptr.add(1);

                iaptr = iaptr.add(2);
                ibptr = ibptr.add(2);
                j += 1;
            }

            i += 1;
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

fn recurseQuad(startX: c_int, startY: c_int, quadSize: c_int, xOff: c_int, yOff: c_int) {
    let mut scroff: *mut c_uchar;
    let mut bigx: c_int = 0;
    let mut bigy: c_int = 0;
    let mut lowx: c_int = 0;
    let mut lowy: c_int = 0;
    let mut useY: c_int = 0;
    let mut offset: c_int = 0;

    unsafe {
        offset = cinTable[currentHandle as usize].screenDelta;

        lowx = 0;
        lowy = 0;
        bigx = cinTable[currentHandle as usize].xsize as c_int;
        bigy = cinTable[currentHandle as usize].ysize as c_int;

        if bigx > cinTable[currentHandle as usize].CIN_WIDTH { bigx = cinTable[currentHandle as usize].CIN_WIDTH; }
        if bigy > cinTable[currentHandle as usize].CIN_HEIGHT { bigy = cinTable[currentHandle as usize].CIN_HEIGHT; }

        if (startX >= lowx) && (startX + quadSize) <= bigx && (startY + quadSize) <= bigy && (startY >= lowy) && quadSize <= MAXSIZE {
            useY = startY;
            scroff = cin.linbuf.as_mut_ptr().add(
                ((useY + ((cinTable[currentHandle as usize].CIN_HEIGHT - bigy) >> 1) + yOff) as usize * cinTable[currentHandle as usize].samplesPerLine as usize) +
                (((startX + xOff) as usize) * 4)
            );

            cin.qStatus[0][cinTable[currentHandle as usize].onQuad as usize] = scroff;
            cin.qStatus[1][cinTable[currentHandle as usize].onQuad as usize] = scroff.add(offset as usize);
            cinTable[currentHandle as usize].onQuad += 1;
        }

        if quadSize != MINSIZE {
            let quadSize_div = quadSize >> 1;
            recurseQuad(startX, startY, quadSize_div, xOff, yOff);
            recurseQuad(startX + quadSize_div, startY, quadSize_div, xOff, yOff);
            recurseQuad(startX, startY + quadSize_div, quadSize_div, xOff, yOff);
            recurseQuad(startX + quadSize_div, startY + quadSize_div, quadSize_div, xOff, yOff);
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

fn setupQuad(xOff: c_int, yOff: c_int) {
    let mut numQuadCels: c_int = 0;
    let mut i: c_int = 0;
    let mut x: c_int = 0;
    let mut y: c_int = 0;
    let temp: *mut c_uchar;

    unsafe {
        if currentHandle < 0 { return; }

        if xOff == cin.oldXOff && yOff == cin.oldYOff && cinTable[currentHandle as usize].ysize == cin.oldysize && cinTable[currentHandle as usize].xsize == cin.oldxsize {
            return;
        }

        cin.oldXOff = xOff;
        cin.oldYOff = yOff;
        cin.oldysize = cinTable[currentHandle as usize].ysize;
        cin.oldxsize = cinTable[currentHandle as usize].xsize;

        numQuadCels = ((cinTable[currentHandle as usize].CIN_WIDTH * cinTable[currentHandle as usize].CIN_HEIGHT) / 16) as c_int;
        numQuadCels += numQuadCels / 4 + numQuadCels / 16;
        numQuadCels += 64;

        numQuadCels = ((cinTable[currentHandle as usize].xsize as c_int * cinTable[currentHandle as usize].ysize as c_int) / 16);
        numQuadCels += numQuadCels / 4;
        numQuadCels += 64;

        cinTable[currentHandle as usize].onQuad = 0;

        y = 0;
        while (y as c_uint) < cinTable[currentHandle as usize].ysize {
            x = 0;
            while (x as c_uint) < cinTable[currentHandle as usize].xsize {
                recurseQuad(x, y, 16, xOff, yOff);
                x += 16;
            }
            y += 16;
        }

        temp = ptr::null_mut();

        i = numQuadCels - 64;
        while i < numQuadCels {
            cin.qStatus[0][i as usize] = temp;
            cin.qStatus[1][i as usize] = temp;
            i += 1;
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

fn readQuadInfo(qData: *mut c_uchar) {
    unsafe {
        if currentHandle < 0 { return; }

        cinTable[currentHandle as usize].xsize = (*qData as c_uint) + ((*qData.add(1) as c_uint) << 8);
        cinTable[currentHandle as usize].ysize = (*qData.add(2) as c_uint) + ((*qData.add(3) as c_uint) << 8);
        cinTable[currentHandle as usize].maxsize = (*qData.add(4) as c_uint) + ((*qData.add(5) as c_uint) << 8);
        cinTable[currentHandle as usize].minsize = (*qData.add(6) as c_uint) + ((*qData.add(7) as c_uint) << 8);

        cinTable[currentHandle as usize].CIN_HEIGHT = cinTable[currentHandle as usize].ysize as c_int;
        cinTable[currentHandle as usize].CIN_WIDTH = cinTable[currentHandle as usize].xsize as c_int;

        cinTable[currentHandle as usize].samplesPerLine = (cinTable[currentHandle as usize].CIN_WIDTH * 4) as c_int;
        cinTable[currentHandle as usize].screenDelta = (cinTable[currentHandle as usize].CIN_HEIGHT * cinTable[currentHandle as usize].samplesPerLine) as c_int;

        cinTable[currentHandle as usize].VQ0 = cinTable[currentHandle as usize].VQNormal;
        cinTable[currentHandle as usize].VQ1 = cinTable[currentHandle as usize].VQBuffer;

        cinTable[currentHandle as usize].t[0] = (0 - (cin.linbuf.as_ptr() as c_uint)) + (cin.linbuf.as_ptr() as c_uint) + cinTable[currentHandle as usize].screenDelta as c_uint;
        cinTable[currentHandle as usize].t[1] = (0 - ((cin.linbuf.as_ptr() as c_uint) + cinTable[currentHandle as usize].screenDelta as c_uint)) + (cin.linbuf.as_ptr() as c_uint);

        cinTable[currentHandle as usize].drawX = cinTable[currentHandle as usize].CIN_WIDTH;
        cinTable[currentHandle as usize].drawY = cinTable[currentHandle as usize].CIN_HEIGHT;

        // jic the card sucks
        // TODO: glConfig.maxTextureSize check
        /*
        if glConfig.maxTextureSize <= 256 {
            if cinTable[currentHandle as usize].drawX > 256 {
                cinTable[currentHandle as usize].drawX = 256;
            }
            if cinTable[currentHandle as usize].drawY > 256 {
                cinTable[currentHandle as usize].drawY = 256;
            }
            if cinTable[currentHandle as usize].CIN_WIDTH != 256 || cinTable[currentHandle as usize].CIN_HEIGHT != 256 {
                Com_DPrintf("HACK: approxmimating cinematic for Rage Pro or Voodoo\n");
            }
        }
        */
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

fn RoQPrepMcomp(xoff: c_int, yoff: c_int) {
    let mut i: c_int = 0;
    let mut j: c_int = 0;
    let mut x: c_int = 0;
    let mut y: c_int = 0;
    let mut temp: c_int = 0;
    let mut temp2: c_int = 0;

    unsafe {
        if currentHandle < 0 { return; }

        i = cinTable[currentHandle as usize].samplesPerLine;
        j = 4;
        if cinTable[currentHandle as usize].xsize == (cinTable[currentHandle as usize].ysize * 4) {
            j = j + j;
            i = i + i;
        }

        y = 0;
        while y < 16 {
            temp2 = (y + yoff - 8) * i;
            x = 0;
            while x < 16 {
                temp = (x + xoff - 8) * j;
                cin.mcomp[((x * 16) + y) as usize] = (cinTable[currentHandle as usize].normalBuffer0 - (temp2 + temp)) as c_uint;
                x += 1;
            }
            y += 1;
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

fn initRoQ() {
    unsafe {
        if currentHandle < 0 { return; }

        cinTable[currentHandle as usize].VQNormal = Some(blitVQQuad32fs);
        cinTable[currentHandle as usize].VQBuffer = Some(blitVQQuad32fs);
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
fn RoQFetchInterlaced(source: *mut c_int) -> *mut c_uchar {
    let mut x: c_int = 0;
    let mut src: *mut c_int;
    let mut dst: *mut c_int;

    unsafe {
        if currentHandle < 0 { return ptr::null_mut(); }

        src = source;
        dst = cinTable[currentHandle as usize].buf2 as *mut c_int;

        x = 0;
        while x < 256 * 256 {
            *dst = *src;
            dst = dst.add(1);
            src = src.add(2);
            x += 1;
        }
    }

    cinTable[currentHandle as usize].buf2
}
*/

fn RoQReset() {
    extern "C" {
        fn Sys_EndStreamedFile(handle: fileHandle_t);
        fn FS_Seek(handle: fileHandle_t, offset: c_int, origin: c_int) -> c_int;
        fn FS_Read(buffer: *mut c_void, len: c_int, handle: fileHandle_t) -> c_int;
        fn Sys_BeginStreamedFile(handle: fileHandle_t, readAhead: c_int);
    }

    const FS_SEEK_SET: c_int = 0;
    const FMV_LOOPED: e_status = 2;

    unsafe {
        if currentHandle < 0 { return; }

        if cinTable[currentHandle as usize].iFile != 0 {
            Sys_EndStreamedFile(cinTable[currentHandle as usize].iFile);
            FS_Seek(cinTable[currentHandle as usize].iFile, 0, FS_SEEK_SET);
            FS_Read(cin.file.as_mut_ptr() as *mut c_void, 16, cinTable[currentHandle as usize].iFile);
            roq_init_internal();
            Sys_BeginStreamedFile(cinTable[currentHandle as usize].iFile, 0x10000);
            cinTable[currentHandle as usize].status = FMV_LOOPED;
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

fn RoQInterrupt() {
    extern "C" {
        fn Sys_StreamedRead(buffer: *mut c_void, size: c_int, count: c_int, handle: fileHandle_t) -> c_int;
    }

    const FMV_EOF: e_status = 0;
    const FMV_IDLE: e_status = 1;
    const FMV_LOOPED: e_status = 2;
    const FMV_PLAY: e_status = 3;

    let mut framedata: *mut c_uchar;

    unsafe {
        if currentHandle < 0 { return; }

        Sys_StreamedRead(cin.file.as_mut_ptr().add(cinTable[currentHandle as usize].RoQFrameSize as usize + 8) as *mut c_void,
                         1,
                         cinTable[currentHandle as usize].RoQFrameSize as c_int + 8,
                         cinTable[currentHandle as usize].iFile);

        if cinTable[currentHandle as usize].RoQPlayed >= cinTable[currentHandle as usize].ROQSize {
            if cinTable[currentHandle as usize].holdAtEnd == 0 {
                if cinTable[currentHandle as usize].looping != 0 {
                    RoQReset();
                } else {
                    cinTable[currentHandle as usize].status = FMV_EOF;
                    if cinTable[currentHandle as usize].hSFX != 0 && cinTable[currentHandle as usize].looping == 0 {
                        S_CIN_StopSound(cinTable[currentHandle as usize].hSFX);
                    }
                }
            } else {
                cinTable[currentHandle as usize].status = FMV_IDLE;
            }
            return;
        }

        framedata = cin.file.as_mut_ptr();

        // new frame is ready
        'redump: loop {
            match cinTable[currentHandle as usize].roq_id {
                ROQ_QUAD_VQ => {
                    if (cinTable[currentHandle as usize].numQuads & 1) != 0 {
                        cinTable[currentHandle as usize].normalBuffer0 = cinTable[currentHandle as usize].t[1];
                        RoQPrepMcomp(cinTable[currentHandle as usize].roqF0, cinTable[currentHandle as usize].roqF1);
                        if let Some(vq1_fn) = cinTable[currentHandle as usize].VQ1 {
                            vq1_fn(cin.qStatus[1].as_mut_ptr() as *mut c_uchar, framedata as *mut c_void);
                        }
                        cinTable[currentHandle as usize].buf = cin.linbuf.as_mut_ptr().add(cinTable[currentHandle as usize].screenDelta as usize);
                    } else {
                        cinTable[currentHandle as usize].normalBuffer0 = cinTable[currentHandle as usize].t[0];
                        RoQPrepMcomp(cinTable[currentHandle as usize].roqF0, cinTable[currentHandle as usize].roqF1);
                        if let Some(vq0_fn) = cinTable[currentHandle as usize].VQ0 {
                            vq0_fn(cin.qStatus[0].as_mut_ptr() as *mut c_uchar, framedata as *mut c_void);
                        }
                        cinTable[currentHandle as usize].buf = cin.linbuf.as_mut_ptr();
                    }
                    if cinTable[currentHandle as usize].numQuads == 0 {
                        // first frame
                        ptr::copy_nonoverlapping(
                            cin.linbuf.as_ptr().add(cinTable[currentHandle as usize].screenDelta as usize),
                            cin.linbuf.as_mut_ptr(),
                            (cinTable[currentHandle as usize].samplesPerLine * cinTable[currentHandle as usize].ysize) as usize
                        );
                    }
                    cinTable[currentHandle as usize].numQuads += 1;
                    cinTable[currentHandle as usize].dirty = 1;
                }
                ROQ_CODEBOOK => {
                    decodeCodeBook(framedata, cinTable[currentHandle as usize].roq_flags as u16);
                }
                ZA_SOUND_MONO => {
                    if cinTable[currentHandle as usize].silent == 0 {
                        if cinTable[currentHandle as usize].numQuads == -1 {
                            S_Update();
                            s_rawend = s_soundtime;
                            RllDecodeMonoToStereo(framedata, cinTable[currentHandle as usize].RoQFrameSize, 0, cinTable[currentHandle as usize].roq_flags as u16, if cinTable[currentHandle as usize].hSFX != 0 { 1 } else { 0 });
                        } else {
                            if cinTable[currentHandle as usize].hSFX != 0 {
                                S_Update();
                            }
                            RllDecodeMonoToStereo(framedata, cinTable[currentHandle as usize].RoQFrameSize, 0, cinTable[currentHandle as usize].roq_flags as u16, if cinTable[currentHandle as usize].hSFX != 0 { 1 } else { 0 });
                        }
                    }
                }
                ZA_SOUND_STEREO => {
                    if cinTable[currentHandle as usize].silent == 0 {
                        if cinTable[currentHandle as usize].numQuads == -1 {
                            S_Update();
                            s_rawend = s_soundtime;
                            RllDecodeStereoToStereo(framedata, cinTable[currentHandle as usize].RoQFrameSize, 0, cinTable[currentHandle as usize].roq_flags as u16, if cinTable[currentHandle as usize].hSFX != 0 { 1 } else { 0 });
                        } else {
                            if cinTable[currentHandle as usize].hSFX != 0 {
                                S_Update();
                            }
                            RllDecodeStereoToStereo(framedata, cinTable[currentHandle as usize].RoQFrameSize, 0, cinTable[currentHandle as usize].roq_flags as u16, if cinTable[currentHandle as usize].hSFX != 0 { 1 } else { 0 });
                        }
                    }
                }
                ROQ_QUAD_INFO => {
                    if cinTable[currentHandle as usize].numQuads == -1 {
                        readQuadInfo(framedata);
                        setupQuad(0, 0);
                        extern "C" {
                            fn Sys_Milliseconds() -> c_int;
                            static mut com_timescale: *mut c_void;
                        }
                        // cinTable[currentHandle as usize].startTime = cinTable[currentHandle as usize].lastTime = (Sys_Milliseconds() * com_timescale->value);
                    }
                    if cinTable[currentHandle as usize].numQuads != 1 {
                        cinTable[currentHandle as usize].numQuads = 0;
                    }
                }
                ROQ_PACKET => {
                    cinTable[currentHandle as usize].inMemory = cinTable[currentHandle as usize].roq_flags as c_uchar;
                    cinTable[currentHandle as usize].RoQFrameSize = 0;
                }
                ROQ_QUAD_HANG => {
                    cinTable[currentHandle as usize].RoQFrameSize = 0;
                }
                ROQ_QUAD_JPEG => {}
                _ => {
                    cinTable[currentHandle as usize].status = FMV_EOF;
                }
            }

            // read in next frame data
            if cinTable[currentHandle as usize].RoQPlayed >= cinTable[currentHandle as usize].ROQSize {
                if cinTable[currentHandle as usize].holdAtEnd == 0 {
                    if cinTable[currentHandle as usize].looping != 0 {
                        RoQReset();
                    } else {
                        cinTable[currentHandle as usize].status = FMV_EOF;
                    }
                } else {
                    cinTable[currentHandle as usize].status = FMV_IDLE;
                }
                return;
            }

            framedata = framedata.add(cinTable[currentHandle as usize].RoQFrameSize as usize);
            cinTable[currentHandle as usize].roq_id = (*framedata as c_uint) + ((*framedata.add(1) as c_uint) << 8);
            cinTable[currentHandle as usize].RoQFrameSize = (*framedata.add(2) as c_uint) + ((*framedata.add(3) as c_uint) << 8) + ((*framedata.add(4) as c_uint) << 16);
            cinTable[currentHandle as usize].roq_flags = (*framedata.add(6) as c_int) + ((*framedata.add(7) as c_int) << 8);
            cinTable[currentHandle as usize].roqF0 = *framedata.add(7) as i8 as c_int;
            cinTable[currentHandle as usize].roqF1 = *framedata.add(6) as i8 as c_int;

            if cinTable[currentHandle as usize].RoQFrameSize > 65536 || cinTable[currentHandle as usize].roq_id == 0x1084 {
                // Com_DPrintf("roq_size>65536||roq_id==0x1084\n");
                cinTable[currentHandle as usize].status = FMV_EOF;
                if cinTable[currentHandle as usize].looping != 0 {
                    RoQReset();
                }
                return;
            }

            if cinTable[currentHandle as usize].inMemory != 0 && (cinTable[currentHandle as usize].status != FMV_EOF) {
                cinTable[currentHandle as usize].inMemory -= 1;
                framedata = framedata.add(8);
                continue 'redump;
            }

            break 'redump;
        }

        // one more frame hits the dust
        cinTable[currentHandle as usize].RoQPlayed += cinTable[currentHandle as usize].RoQFrameSize as c_int + 8;
    }
}

/******************************************************************************
*
* Function:
*
* Description:
*
******************************************************************************/

fn roq_init_internal() {
    extern "C" {
        fn Sys_Milliseconds() -> c_int;
        static mut com_timescale: *mut c_void;
        fn S_StartLocalSound(sfxHandle: sfxHandle_t, channelIndex: c_int);
    }

    unsafe {
        cinTable[currentHandle as usize].startTime = cinTable[currentHandle as usize].lastTime = Sys_Milliseconds();

        cinTable[currentHandle as usize].RoQPlayed = 24;

        // get frame rate
        cinTable[currentHandle as usize].roqFPS = (cin.file[6] as c_int) + ((cin.file[7] as c_int) << 8);

        if cinTable[currentHandle as usize].roqFPS == 0 {
            cinTable[currentHandle as usize].roqFPS = 30;
        }

        cinTable[currentHandle as usize].numQuads = -1;

        cinTable[currentHandle as usize].roq_id = (cin.file[8] as c_uint) + ((cin.file[9] as c_uint) << 8);
        cinTable[currentHandle as usize].RoQFrameSize = (cin.file[10] as c_uint) + ((cin.file[11] as c_uint) << 8) + ((cin.file[12] as c_uint) << 16);
        cinTable[currentHandle as usize].roq_flags = (cin.file[14] as c_int) + ((cin.file[15] as c_int) << 8);

        if cinTable[currentHandle as usize].RoQFrameSize > 65536 || cinTable[currentHandle as usize].RoQFrameSize == 0 {
            return;
        }

        if cinTable[currentHandle as usize].hSFX != 0 {
            S_StartLocalSound(cinTable[currentHandle as usize].hSFX, 0); // CHAN_AUTO
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

fn RoQShutdown() {
    extern "C" {
        fn Sys_EndStreamedFile(handle: fileHandle_t);
        fn FS_FCloseFile(handle: fileHandle_t);
        fn Com_DPrintf(format: *const c_char, ...);
    }

    const FMV_IDLE: e_status = 1;

    unsafe {
        if cinTable[currentHandle as usize].buf.is_null() {
            if cinTable[currentHandle as usize].iFile != 0 {
                Sys_EndStreamedFile(cinTable[currentHandle as usize].iFile);
                FS_FCloseFile(cinTable[currentHandle as usize].iFile);
                cinTable[currentHandle as usize].iFile = 0;
                if cinTable[currentHandle as usize].hSFX != 0 {
                    S_CIN_StopSound(cinTable[currentHandle as usize].hSFX);
                }
            }
            return;
        }

        if cinTable[currentHandle as usize].status == FMV_IDLE {
            return;
        }

        // Com_DPrintf("finished cinematic\n");
        cinTable[currentHandle as usize].status = FMV_IDLE;

        if cinTable[currentHandle as usize].iFile != 0 {
            Sys_EndStreamedFile(cinTable[currentHandle as usize].iFile);
            FS_FCloseFile(cinTable[currentHandle as usize].iFile);
            cinTable[currentHandle as usize].iFile = 0;
            if cinTable[currentHandle as usize].hSFX != 0 {
                S_CIN_StopSound(cinTable[currentHandle as usize].hSFX);
            }
        }

        if cinTable[currentHandle as usize].alterGameState != 0 {
            // cls.state = CA_DISCONNECTED;
            // TODO: handle alter game state
            /*
            s = Cvar_VariableString("nextmap");
            if s[0] != 0 {
                Cbuf_ExecuteText(EXEC_APPEND, va("%s\n", s));
                Cvar_Set("nextmap", "");
            }
            */
            CL_handle = -1;
        }
        cinTable[currentHandle as usize].fileName[0] = 0;
        currentHandle = -1;
    }
}

pub extern "C" fn CIN_StopCinematic(handle: c_int) -> e_status {
    extern "C" {
        fn Sys_EndStreamedFile(handle: fileHandle_t);
        fn FS_FCloseFile(handle: fileHandle_t);
    }

    const FMV_EOF: e_status = 0;

    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES as c_int || cinTable[handle as usize].status == FMV_EOF {
            return FMV_EOF;
        }
        currentHandle = handle;

        // Com_DPrintf("trFMV::stop(), closing %s\n", cinTable[currentHandle as usize].fileName.as_ptr());

        if cinTable[currentHandle as usize].buf.is_null() {
            if cinTable[currentHandle as usize].iFile != 0 {
                Sys_EndStreamedFile(cinTable[currentHandle as usize].iFile);
                FS_FCloseFile(cinTable[currentHandle as usize].iFile);
                cinTable[currentHandle as usize].iFile = 0;
                cinTable[currentHandle as usize].fileName[0] = 0;
                if cinTable[currentHandle as usize].hSFX != 0 {
                    S_CIN_StopSound(cinTable[currentHandle as usize].hSFX);
                }
            }
            return FMV_EOF;
        }

        if cinTable[currentHandle as usize].alterGameState != 0 {
            // TODO: handle CA_CINEMATIC check
            /*
            if cls.state != CA_CINEMATIC {
                return cinTable[currentHandle as usize].status;
            }
            */
        }
        cinTable[currentHandle as usize].status = FMV_EOF;
        RoQShutdown();

        FMV_EOF
    }
}

pub extern "C" fn CIN_RunCinematic(handle: c_int) -> e_status {
    extern "C" {
        fn Sys_Milliseconds() -> c_int;
        static mut com_timescale: *mut c_void;
    }

    const FMV_EOF: e_status = 0;
    const FMV_IDLE: e_status = 1;
    const FMV_LOOPED: e_status = 2;
    const FMV_PLAY: e_status = 3;

    let mut start: c_uint = 0;
    let mut thisTime: c_int = 0;

    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES as c_int || cinTable[handle as usize].status == FMV_EOF {
            return FMV_EOF;
        }

        if currentHandle != handle {
            currentHandle = handle;
            cinTable[currentHandle as usize].status = FMV_EOF;
            RoQReset();
        }

        if cinTable[handle as usize].playonwalls < -1 {
            return cinTable[handle as usize].status;
        }

        currentHandle = handle;

        if cinTable[currentHandle as usize].alterGameState != 0 {
            // TODO: handle CA_CINEMATIC check
            /*
            if cls.state != CA_CINEMATIC {
                return cinTable[currentHandle as usize].status;
            }
            */
        }

        if cinTable[currentHandle as usize].status == FMV_IDLE {
            return cinTable[currentHandle as usize].status;
        }

        thisTime = Sys_Milliseconds();
        if cinTable[currentHandle as usize].shader != 0 && ((thisTime - cinTable[currentHandle as usize].lastTime) as c_int).abs() > 100 {
            cinTable[currentHandle as usize].startTime = (cinTable[currentHandle as usize].startTime as c_int + thisTime - cinTable[currentHandle as usize].lastTime) as c_uint;
        }
        cinTable[currentHandle as usize].tfps = (((Sys_Milliseconds() - cinTable[currentHandle as usize].startTime as c_int) * cinTable[currentHandle as usize].roqFPS) / 1000) as c_int;

        start = cinTable[currentHandle as usize].startTime;
        while (cinTable[currentHandle as usize].tfps != cinTable[currentHandle as usize].numQuads) && (cinTable[currentHandle as usize].status == FMV_PLAY) {
            RoQInterrupt();
            if start != cinTable[currentHandle as usize].startTime {
                cinTable[currentHandle as usize].tfps = (((Sys_Milliseconds() - cinTable[currentHandle as usize].startTime as c_int) * cinTable[currentHandle as usize].roqFPS) / 1000) as c_int;
                start = cinTable[currentHandle as usize].startTime;
            }
        }

        cinTable[currentHandle as usize].lastTime = thisTime as c_uint;

        if cinTable[currentHandle as usize].status == FMV_LOOPED {
            cinTable[currentHandle as usize].status = FMV_PLAY;
        }

        if cinTable[currentHandle as usize].status == FMV_EOF {
            if cinTable[currentHandle as usize].looping != 0 {
                RoQReset();
            } else {
                RoQShutdown();
                return FMV_IDLE;
            }
        }

        cinTable[currentHandle as usize].status
    }
}

pub extern "C" fn CIN_PlayCinematic(arg: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, systemBits: c_int, psAudioFile: *const c_char) -> c_int {
    extern "C" {
        fn Com_sprintf(dest: *mut c_char, size: usize, format: *const c_char, ...);
        fn FS_FOpenFileRead(qpath: *const c_char, handle: *mut fileHandle_t, uniqueFILE: qboolean) -> c_int;
        fn FS_Read(buffer: *mut c_void, len: c_int, handle: fileHandle_t) -> c_int;
        fn Sys_BeginStreamedFile(handle: fileHandle_t, readAhead: c_int);
        fn S_RegisterSound(name: *const c_char) -> sfxHandle_t;
        fn Con_Close();
        fn S_StopAllSounds();
        fn strlen(s: *const c_char) -> usize;
        fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
        fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
        fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    }

    const CIN_system: c_int = 1;
    const CIN_loop: c_int = 2;
    const CIN_hold: c_int = 4;
    const CIN_silent: c_int = 8;
    const CIN_shader: c_int = 16;
    const FMV_PLAY: e_status = 3;

    let mut RoQID: u16 = 0;
    let mut name: [c_char; 260] = [0; 260];
    let mut i: c_int = 0;

    unsafe {
        // Build the name
        if strstr(arg, b"/" as *const u8 as *const c_char).is_null() && strstr(arg, b"\\" as *const u8 as *const c_char).is_null() {
            Com_sprintf(name.as_mut_ptr(), name.len(), b"video/%s\0" as *const u8 as *const c_char, arg);
        } else {
            Com_sprintf(name.as_mut_ptr(), name.len(), b"%s\0" as *const u8 as *const c_char, arg);
        }

        // COM_DefaultExtension(name, name.len(), ".roq");
        // TODO: Add default extension

        if (systemBits & CIN_system) == 0 {
            i = 0;
            while i < MAX_VIDEO_HANDLES as c_int {
                if strcmp(cinTable[i as usize].fileName.as_ptr(), name.as_ptr()) == 0 {
                    return i;
                }
                i += 1;
            }
        }

        // Com_DPrintf("SCR_PlayCinematic( %s )\n", arg);

        currentHandle = CIN_HandleForVideo();
        if currentHandle == -1 {
            return -1;
        }

        strcpy(cinTable[currentHandle as usize].fileName.as_mut_ptr(), name.as_ptr());

        cinTable[currentHandle as usize].ROQSize = 0;
        cinTable[currentHandle as usize].ROQSize = FS_FOpenFileRead(cinTable[currentHandle as usize].fileName.as_ptr(), &mut cinTable[currentHandle as usize].iFile, 1);

        if cinTable[currentHandle as usize].ROQSize <= 0 {
            // Com_Printf(S_COLOR_RED"ERROR: playCinematic: %s not found!\n", arg);
            cinTable[currentHandle as usize].fileName[0] = 0;
            return -1;
        }

        CIN_SetExtents(currentHandle, x, y, w, h);
        CIN_SetLooping(currentHandle, ((systemBits & CIN_loop) != 0) as c_uchar);

        cinTable[currentHandle as usize].CIN_HEIGHT = DEFAULT_CIN_HEIGHT;
        cinTable[currentHandle as usize].CIN_WIDTH = DEFAULT_CIN_WIDTH;
        cinTable[currentHandle as usize].holdAtEnd = (((systemBits & CIN_hold) != 0) as c_uchar);
        cinTable[currentHandle as usize].alterGameState = (((systemBits & CIN_system) != 0) as c_uchar);
        cinTable[currentHandle as usize].playonwalls = 1;
        cinTable[currentHandle as usize].silent = (((systemBits & CIN_silent) != 0) as c_uchar);
        cinTable[currentHandle as usize].shader = (((systemBits & CIN_shader) != 0) as c_uchar);

        if !psAudioFile.is_null() {
            cinTable[currentHandle as usize].hSFX = S_RegisterSound(psAudioFile);
        } else {
            cinTable[currentHandle as usize].hSFX = 0;
        }
        cinTable[currentHandle as usize].hCRAWLTEXT = 0;

        if cinTable[currentHandle as usize].alterGameState != 0 {
            Con_Close();
            // TODO: Handle UI_Cursor_Show and Menus_CloseAll
        } else {
            // TODO: cl_ingameVideo->integer
            cinTable[currentHandle as usize].playonwalls = 1; // cl_ingameVideo->integer
        }

        initRoQ();

        FS_Read(cin.file.as_mut_ptr() as *mut c_void, 16, cinTable[currentHandle as usize].iFile);

        RoQID = (cin.file[0] as u16) + ((cin.file[1] as u16) << 8);
        if RoQID == 0x1084 {
            roq_init_internal();
            Sys_BeginStreamedFile(cinTable[currentHandle as usize].iFile, 0x10000);

            cinTable[currentHandle as usize].status = FMV_PLAY;
            // Com_DPrintf("trFMV::play(), playing %s\n", arg);

            if cinTable[currentHandle as usize].alterGameState != 0 {
                // cls.state = CA_CINEMATIC;
            }

            Con_Close();

            s_rawend = s_soundtime;

            return currentHandle;
        }

        // Com_DPrintf("trFMV::play(), invalid RoQ ID\n");

        RoQShutdown();
        -1
    }
}

pub extern "C" fn CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int) {
    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES as c_int || cinTable[handle as usize].status == 0 {
            return;
        }
        cinTable[handle as usize].xpos = x;
        cinTable[handle as usize].ypos = y;
        cinTable[handle as usize].width = w;
        cinTable[handle as usize].height = h;
        cinTable[handle as usize].dirty = 1;
    }
}

pub extern "C" fn CIN_SetLooping(handle: c_int, looping: qboolean) {
    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES as c_int || cinTable[handle as usize].status == 0 {
            return;
        }
        cinTable[handle as usize].looping = looping;
    }
}

pub extern "C" fn CIN_DrawCinematic(handle: c_int) {
    extern "C" {
        fn Z_Malloc(size: c_int, tag: c_int, zeroIt: qboolean) -> *mut c_void;
        fn Z_Free(ptr: *mut c_void);
    }

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut w: f32 = 0.0;
    let mut h: f32 = 0.0;

    unsafe {
        if handle < 0 || handle >= MAX_VIDEO_HANDLES as c_int || cinTable[handle as usize].status == 0 {
            return;
        }

        if cinTable[handle as usize].buf.is_null() {
            return;
        }

        x = cinTable[handle as usize].xpos as f32;
        y = cinTable[handle as usize].ypos as f32;
        w = cinTable[handle as usize].width as f32;
        h = cinTable[handle as usize].height as f32;

        if cinTable[handle as usize].dirty != 0 && (cinTable[handle as usize].CIN_WIDTH != cinTable[handle as usize].drawX || cinTable[handle as usize].CIN_HEIGHT != cinTable[handle as usize].drawY) {
            let mut ix: c_int = 0;
            let mut iy: c_int = 0;
            let mut buf2: *mut c_int;
            let mut buf3: *mut c_int;
            let mut xm: c_int = 0;
            let mut ym: c_int = 0;

            xm = cinTable[handle as usize].CIN_WIDTH / 256;
            ym = cinTable[handle as usize].CIN_HEIGHT / 256;

            buf3 = cinTable[handle as usize].buf as *mut c_int;
            buf2 = Z_Malloc((256 * 256 * 4) as c_int, 0, 0) as *mut c_int;

            iy = 0;
            while iy < 256 {
                ix = 0;
                while ix < 256 {
                    *buf2.add(((iy << 8) + ix) as usize) = *buf3.add((((iy * ym) * cinTable[handle as usize].CIN_WIDTH) + (ix * xm)) as usize);
                    ix += 1;
                }
                iy += 1;
            }

            // re.DrawStretchRaw(x, y, w, h, 256, 256, buf2 as *mut c_uchar, handle, 1);
            cinTable[handle as usize].dirty = 0;
            Z_Free(buf2 as *mut c_void);

            return;
        }

        // re.DrawStretchRaw(x, y, w, h, cinTable[handle as usize].drawX as u32, cinTable[handle as usize].drawY as u32, cinTable[handle as usize].buf, handle, cinTable[handle as usize].dirty);
        cinTable[handle as usize].dirty = 0;
    }
}

pub extern "C" fn CIN_UploadCinematic(handle: c_int) {
    unsafe {
        if handle >= 0 && handle < MAX_VIDEO_HANDLES as c_int {
            if cinTable[handle as usize].buf.is_null() {
                return;
            }
            if cinTable[handle as usize].playonwalls <= 0 && cinTable[handle as usize].dirty != 0 {
                if cinTable[handle as usize].playonwalls == 0 {
                    cinTable[handle as usize].playonwalls = -1;
                } else {
                    if cinTable[handle as usize].playonwalls == -1 {
                        cinTable[handle as usize].playonwalls = -2;
                    } else {
                        cinTable[handle as usize].dirty = 0;
                    }
                }
            }
            // re.UploadCinematic(cinTable[handle as usize].drawX as c_uint, cinTable[handle as usize].drawY as c_uint, cinTable[handle as usize].buf, handle, cinTable[handle as usize].dirty);
            // TODO: cl_ingameVideo->integer check
            if false && cinTable[handle as usize].playonwalls == 1 {
                cinTable[handle as usize].playonwalls -= 1;
            }
        }
    }
}

pub extern "C" fn CL_IsRunningInGameCinematic() -> qboolean {
    0 // TODO: qbPlayingInGameCinematic
}

pub extern "C" fn CL_InGameCinematicOnStandBy() -> qboolean {
    0 // TODO: qbInGameCinematicOnStandBy
}
