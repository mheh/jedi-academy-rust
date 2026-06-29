// Anything above this #include will be ignored by the compiler
//
// WIN_GAMMA.C

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void, c_char};
use core::ptr::addr_of_mut;

// Windows API types
type HDC = *mut c_void;
type HWND = *mut c_void;
type BOOL = c_int;

#[repr(C)]
pub struct OSVERSIONINFO {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
}

const VER_PLATFORM_WIN32_NT: u32 = 2;

// Extract high byte from u16, as in the original HIBYTE macro
#[inline]
fn HIBYTE(w: u16) -> u8 {
    ((w >> 8) & 0xff) as u8
}

// Windows API functions
extern "C" {
    fn GetDC(hWnd: HWND) -> HDC;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
    fn GetDesktopWindow() -> HWND;
    fn GetDeviceGammaRamp(hDC: HDC, lpRamp: *mut [[u16; 256]; 3]) -> BOOL;
    fn SetDeviceGammaRamp(hDC: HDC, lpRamp: *const [[u16; 256]; 3]) -> BOOL;
    fn GetVersionEx(lpVersionInformation: *mut OSVERSIONINFO) -> BOOL;
}

// External engine references
// glConfig, r_ignorehwgamma, glw_state are globals defined in the engine;
// we declare them as extern with minimal #[repr(C)] stubs containing only the fields we need
#[repr(C)]
pub struct glConfig_t {
    pub deviceSupportsGamma: c_int,
    // other fields omitted
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    // other fields omitted
}

#[repr(C)]
pub struct glw_state_t {
    pub hDC: HDC,
    // other fields omitted
}

extern "C" {
    static mut glConfig: glConfig_t;
    static r_ignorehwgamma: *const cvar_t;
    static mut glw_state: glw_state_t;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    static S_COLOR_YELLOW: *const c_char;
}

static mut s_oldHardwareGamma: [[u16; 256]; 3] = [[0; 256]; 3];

/*
** WG_CheckHardwareGamma
**
** Determines if the underlying hardware supports the Win32 gamma correction API.
*/
#[no_mangle]
pub extern "C" fn WG_CheckHardwareGamma() {
    unsafe {
        let mut hDC: HDC;

        glConfig.deviceSupportsGamma = 0; // qfalse

        if (*r_ignorehwgamma).integer == 0 {
            hDC = GetDC(GetDesktopWindow());
            glConfig.deviceSupportsGamma = GetDeviceGammaRamp(hDC, addr_of_mut!(s_oldHardwareGamma));
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
                    Com_Printf(
                        b"%sWARNING: device has broken gamma support, generated gamma.dat\n\0".as_ptr() as *const c_char,
                        S_COLOR_YELLOW,
                    );
                }

                //
                // make sure that we didn't have a prior crash in the game, and if so we need to
                // restore the gamma values to at least a linear value
                //
                if HIBYTE(s_oldHardwareGamma[0][181]) == 255 {
                    let mut g: c_int;

                    Com_Printf(
                        b"%sWARNING: suspicious gamma tables, using linear ramp for restoration\n\0".as_ptr() as *const c_char,
                        S_COLOR_YELLOW,
                    );

                    for g in 0..255 {
                        s_oldHardwareGamma[0][g as usize] = (g << 8) as u16;
                        s_oldHardwareGamma[1][g as usize] = (g << 8) as u16;
                        s_oldHardwareGamma[2][g as usize] = (g << 8) as u16;
                    }
                }
            }
        }
    }
}

/*
void mapGammaMax( void ) {
    int		i, j;
    unsigned short table[3][256];

    // try to figure out what win2k will let us get away with setting
    for ( i = 0 ; i < 256 ; i++ ) {
        if ( i >= 128 ) {
            table[0][i] = table[1][i] = table[2][i] = 0xffff;
        } else {
            table[0][i] = table[1][i] = table[2][i] = i<<9;
        }
    }

    for ( i = 0 ; i < 128 ; i++ ) {
        for ( j = i*2 ; j < 255 ; j++ ) {
            table[0][i] = table[1][i] = table[2][i] = j<<8;
            if ( !SetDeviceGammaRamp( glw_state.hDC, table ) ) {
                break;
            }
        }
        table[0][i] = table[1][i] = table[2][i] = i<<9;
        Com_Printf( "index %i max: %i\n", i, j-1 );
    }
}
*/

/*
** GLimp_SetGamma
**
** This routine should only be called if glConfig.deviceSupportsGamma is TRUE
*/
#[no_mangle]
pub extern "C" fn GLimp_SetGamma(red: *const u8, green: *const u8, blue: *const u8) {
    unsafe {
        let mut table: [[u16; 256]; 3] = [[0; 256]; 3];
        let mut i: c_int;
        let mut j: c_int;
        let mut ret: c_int;
        let mut vinfo: OSVERSIONINFO;

        if glConfig.deviceSupportsGamma == 0 || (*r_ignorehwgamma).integer != 0 || glw_state.hDC.is_null() {
            return;
        }

        //mapGammaMax();

        for i in 0..256 {
            table[0][i as usize] = (((*red.add(i as usize)) as u16) << 8) | (*red.add(i as usize)) as u16;
            table[1][i as usize] = (((*green.add(i as usize)) as u16) << 8) | (*green.add(i as usize)) as u16;
            table[2][i as usize] = (((*blue.add(i as usize)) as u16) << 8) | (*blue.add(i as usize)) as u16;
        }

        // Win2K puts this odd restriction on gamma ramps...
        vinfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFO>() as u32;
        GetVersionEx(&mut vinfo);
        if vinfo.dwMajorVersion == 5 && vinfo.dwPlatformId == VER_PLATFORM_WIN32_NT {
            Com_DPrintf(b"performing W2K gamma clamp.\n\0".as_ptr() as *const c_char);
            for j in 0..3 {
                for i in 0..128 {
                    if table[j as usize][i as usize] > ((128 + i) << 8) as u16 {
                        table[j as usize][i as usize] = ((128 + i) << 8) as u16;
                    }
                }
                if table[j as usize][127] > (254 << 8) as u16 {
                    table[j as usize][127] = (254 << 8) as u16;
                }
            }
        } else {
            Com_DPrintf(b"skipping W2K gamma clamp.\n\0".as_ptr() as *const c_char);
        }

        // enforce constantly increasing
        for j in 0..3 {
            for i in 1..256 {
                if table[j as usize][i as usize] < table[j as usize][(i - 1) as usize] {
                    table[j as usize][i as usize] = table[j as usize][(i - 1) as usize];
                }
            }
        }

        ret = SetDeviceGammaRamp(glw_state.hDC, &table);
        if ret == 0 {
            Com_Printf(b"SetDeviceGammaRamp failed.\n\0".as_ptr() as *const c_char);
        }
    }
}

/*
** WG_RestoreGamma
*/
#[no_mangle]
pub extern "C" fn WG_RestoreGamma() {
    unsafe {
        if glConfig.deviceSupportsGamma != 0 {
            let hDC: HDC;

            hDC = GetDC(GetDesktopWindow());
            SetDeviceGammaRamp(hDC, &s_oldHardwareGamma);
            ReleaseDC(GetDesktopWindow(), hDC);
        }
    }
}
