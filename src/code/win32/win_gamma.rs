/*
** WIN_GAMMA.C
*/
// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_void};

// Helper macro for extracting high byte (equivalent to Windows HIBYTE)
#[inline]
const fn HIBYTE(w: u16) -> u16 {
    w >> 8
}

// Windows API declarations
extern "C" {
    fn GetDC(hwnd: *mut c_void) -> *mut c_void;
    fn GetDesktopWindow() -> *mut c_void;
    fn GetDeviceGammaRamp(hdc: *mut c_void, lpramp: *mut [[u16; 256]; 3]) -> c_int;
    fn ReleaseDC(hwnd: *mut c_void, hdc: *mut c_void) -> c_int;
    fn SetDeviceGammaRamp(hdc: *mut c_void, lpramp: *const [[u16; 256]; 3]) -> c_int;
    fn GetVersionEx(lpversioninfo: *mut OSVERSIONINFO) -> c_int;
}

// Windows OSVERSIONINFO structure
#[repr(C)]
#[allow(non_snake_case)]
pub struct OSVERSIONINFO {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
}

// Windows platform constant
const VER_PLATFORM_WIN32_NT: u32 = 2;

// External engine declarations
extern "C" {
    static mut glConfig: glConfig_t;
    static mut r_ignorehwgamma: cvar_t;
    static mut glw_state: glw_state_t;

    fn VID_Printf(print_level: c_int, msg: *const i8, ...);
    fn Com_DPrintf(msg: *const i8, ...);
    fn Com_Printf(msg: *const i8, ...);
}

// Engine type stubs for external globals
#[repr(C)]
#[allow(non_snake_case)]
pub struct glConfig_t {
    pub deviceSupportsGamma: c_int,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct cvar_t {
    pub integer: c_int,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct glw_state_t {
    pub hDC: *mut c_void,
}

const PRINT_WARNING: c_int = 2;

static mut s_oldHardwareGamma: [[u16; 256]; 3] = [[0; 256]; 3];

/*
** WG_CheckHardwareGamma
**
** Determines if the underlying hardware supports the Win32 gamma correction API.
*/
#[allow(non_snake_case)]
pub unsafe fn WG_CheckHardwareGamma() {
    let hDC: *mut c_void;

    glConfig.deviceSupportsGamma = 0; // qfalse

    if r_ignorehwgamma.integer == 0 {
        hDC = GetDC(GetDesktopWindow());
        glConfig.deviceSupportsGamma = GetDeviceGammaRamp(hDC, core::ptr::addr_of_mut!(s_oldHardwareGamma));
        ReleaseDC(GetDesktopWindow(), hDC);

        if glConfig.deviceSupportsGamma != 0 {
            //
            // do a sanity check on the gamma values
            //
            if (HIBYTE(s_oldHardwareGamma[0][255]) <= HIBYTE(s_oldHardwareGamma[0][0]))
                || (HIBYTE(s_oldHardwareGamma[1][255]) <= HIBYTE(s_oldHardwareGamma[1][0]))
                || (HIBYTE(s_oldHardwareGamma[2][255]) <= HIBYTE(s_oldHardwareGamma[2][0]))
            {
                glConfig.deviceSupportsGamma = 0; // qfalse
                VID_Printf(PRINT_WARNING, c"WARNING: device has broken gamma support, generated gamma.dat\n".as_ptr());
            }

            //
            // make sure that we didn't have a prior crash in the game, and if so we need to
            // restore the gamma values to at least a linear value
            //
            if HIBYTE(s_oldHardwareGamma[0][181]) == 255 {
                let mut g: c_int;

                VID_Printf(PRINT_WARNING, c"WARNING: suspicious gamma tables, using linear ramp for restoration\n".as_ptr());

                g = 0;
                while g < 255 {
                    s_oldHardwareGamma[0][g as usize] = (g << 8) as u16;
                    s_oldHardwareGamma[1][g as usize] = (g << 8) as u16;
                    s_oldHardwareGamma[2][g as usize] = (g << 8) as u16;
                    g += 1;
                }
            }
        }
    }
}

/*
** GLimp_SetGamma
**
** This routine should only be called if glConfig.deviceSupportsGamma is TRUE
*/
#[allow(non_snake_case)]
pub unsafe fn GLimp_SetGamma(red: *mut [u8; 256], green: *mut [u8; 256], blue: *mut [u8; 256]) {
    let mut table: [[u16; 256]; 3] = [[0; 256]; 3];
    let mut i: c_int;
    let mut j: c_int;
    let ret: c_int;
    let mut vinfo: OSVERSIONINFO;

    if glConfig.deviceSupportsGamma == 0 || r_ignorehwgamma.integer != 0 || glw_state.hDC.is_null() {
        return;
    }

    //mapGammaMax();

    i = 0;
    while i < 256 {
        table[0][i as usize] = (((*red)[i as usize] as u16) << 8) | (*red)[i as usize] as u16;
        table[1][i as usize] = (((*green)[i as usize] as u16) << 8) | (*green)[i as usize] as u16;
        table[2][i as usize] = (((*blue)[i as usize] as u16) << 8) | (*blue)[i as usize] as u16;
        i += 1;
    }

    // Win2K puts this odd restriction on gamma ramps...
    vinfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFO>() as u32;
    GetVersionEx(core::ptr::addr_of_mut!(vinfo));
    if vinfo.dwMajorVersion == 5 && vinfo.dwPlatformId == VER_PLATFORM_WIN32_NT {
        Com_DPrintf(c"performing W2K gamma clamp.\n".as_ptr());
        j = 0;
        while j < 3 {
            i = 0;
            while i < 128 {
                if table[j as usize][i as usize] > ((128 + i) << 8) as u16 {
                    table[j as usize][i as usize] = ((128 + i) << 8) as u16;
                }
                i += 1;
            }
            if table[j as usize][127] > (254 << 8) as u16 {
                table[j as usize][127] = (254 << 8) as u16;
            }
            j += 1;
        }
    } else {
        Com_DPrintf(c"skipping W2K gamma clamp.\n".as_ptr());
    }

    // enforce constantly increasing
    j = 0;
    while j < 3 {
        i = 1;
        while i < 256 {
            if table[j as usize][i as usize] < table[j as usize][(i - 1) as usize] {
                table[j as usize][i as usize] = table[j as usize][(i - 1) as usize];
            }
            i += 1;
        }
        j += 1;
    }

    let ret = SetDeviceGammaRamp(glw_state.hDC, core::ptr::addr_of!(table));
    if ret == 0 {
        Com_Printf(c"SetDeviceGammaRamp failed.\n".as_ptr());
    }
}

/*
** WG_RestoreGamma
*/
#[allow(non_snake_case)]
pub unsafe fn WG_RestoreGamma() {
    if glConfig.deviceSupportsGamma != 0 {
        let hDC: *mut c_void;

        hDC = GetDC(GetDesktopWindow());
        SetDeviceGammaRamp(hDC, core::ptr::addr_of!(s_oldHardwareGamma));
        ReleaseDC(GetDesktopWindow(), hDC);
    }
}
