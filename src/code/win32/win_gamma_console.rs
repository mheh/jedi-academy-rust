/*
** WIN_GAMMA.C
*/
// leave this as first line for PCH reasons...
//
//#include "../server/exe_headers.h"

use core::ffi::c_int;

// Stubs for cross-module types and globals
// These are defined elsewhere in the codebase

extern "C" {
    static mut glConfig: GlConfig;
}

#[cfg(any(target_env = "xbox"))]
extern "C" {
    static mut glw_state: *mut GlwState;
}

#[repr(C)]
pub struct GlConfig {
    pub deviceSupportsGamma: c_int,
    // ... other fields omitted for this module's scope
}

#[cfg(any(target_env = "xbox"))]
#[repr(C)]
pub struct GlwState {
    pub device: *mut GlDevice,
    // ... other fields omitted for this module's scope
}

#[cfg(any(target_env = "xbox"))]
#[repr(C)]
pub struct GlDevice;

#[cfg(any(target_env = "xbox"))]
#[repr(C)]
pub struct D3DGAMMARAMP {
    pub red: [c_int; 256],
    pub green: [c_int; 256],
    pub blue: [c_int; 256],
}

// Platform-specific constants and functions
#[cfg(any(target_env = "gamecube"))]
pub type GXGamma = c_int;

#[cfg(any(target_env = "gamecube"))]
pub const GX_GM_1_0: c_int = 0;

#[cfg(any(target_env = "gamecube"))]
pub const GX_GM_1_7: c_int = 1;

#[cfg(any(target_env = "gamecube"))]
pub const GX_GM_2_2: c_int = 2;

#[cfg(any(target_env = "gamecube"))]
extern "C" {
    pub fn GXSetDispCopyGamma(gamma: c_int);
}

#[cfg(any(target_env = "xbox"))]
extern "C" {
    pub fn SetGammaRamp(device: *mut GlDevice, flags: c_int, ramp: *const D3DGAMMARAMP);
}

// Local helper for pow() - call from libm
extern "C" {
    pub fn pow(x: f64, y: f64) -> f64;
}

/*
** WG_CheckHardwareGamma
**
** Determines if the underlying hardware supports the Win32 gamma correction API.
*/
pub fn WG_CheckHardwareGamma() {
    unsafe {
        (*core::ptr::addr_of_mut!(glConfig)).deviceSupportsGamma = 1; // qtrue
    }
}

/*
** GLimp_SetGamma
**
** This routine should only be called if glConfig.deviceSupportsGamma is TRUE
*/
pub fn GLimp_SetGamma(g: f32) {
    #[cfg(any(target_env = "gamecube"))]
    {
        let mut gamma: c_int = GX_GM_1_0;
        if g >= 2.2f {
            gamma = GX_GM_2_2;
        } else if g >= 1.7f {
            gamma = GX_GM_1_7;
        }
        unsafe { GXSetDispCopyGamma(gamma); }
    }

    #[cfg(any(target_env = "xbox"))]
    {
        const MAXVAL: c_int = 255;

        let mut ramp: D3DGAMMARAMP = unsafe { core::mem::zeroed() };

        for i in 0..256 {
            let mut inf: c_int;
            if g == 1.0f {
                inf = ((MAXVAL as f32) * (i as f32) / 255.0f) as c_int;
            } else {
                inf = ((MAXVAL as f32) * (unsafe { pow((i as f64) / 255.0f64, 1.0f64 / (g as f64)) } as f32) + 0.5f) as c_int;
            }
            if inf < 0 {
                inf = 0;
            }
            if inf > MAXVAL {
                inf = MAXVAL;
            }
            ramp.red[i] = inf;
            ramp.green[i] = inf;
            ramp.blue[i] = inf;
        }

        unsafe {
            // Note: D3DSGR_CALIBRATE = 1 on Xbox
            // Original: glw_state->device->SetGammaRamp(D3DSGR_CALIBRATE, &ramp);
            let state = &*glw_state;
            SetGammaRamp(state.device, 1, &ramp);
        }
    }
}
