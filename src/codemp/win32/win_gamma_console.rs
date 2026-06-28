/*
** WIN_GAMMA.C
*/
// leave this as first line for PCH reasons...
//
//#include "../server/exe_headers.h"

// #include <assert.h>
// #include "../game/q_shared.h"
// #include "../renderer/tr_local.h"
// #include "../qcommon/qcommon.h"
// #include "win_local.h"

// #if defined(_XBOX)
// #include "glw_win_dx8.h"
// #endif

#![allow(non_snake_case)]

use core::ffi::c_int;

// External declarations from included modules
extern "C" {
    static qtrue: c_int;
    static mut glConfig: glconfig_t;
}

#[cfg(feature = "xbox")]
extern "C" {
    static mut glw_state: *mut glw_state_t;
}

// Type definitions for external dependencies
#[repr(C)]
pub struct glconfig_t {
    pub deviceSupportsGamma: c_int,
}

#[repr(C)]
#[cfg(feature = "xbox")]
pub struct glw_state_t {
    pub device: *mut core::ffi::c_void,
}

#[repr(C)]
#[cfg(feature = "xbox")]
pub struct D3DGAMMARAMP {
    pub red: [c_int; 256],
    pub green: [c_int; 256],
    pub blue: [c_int; 256],
}

/*
** WG_CheckHardwareGamma
**
** Determines if the underlying hardware supports the Win32 gamma correction API.
*/
pub fn WG_CheckHardwareGamma() {
    unsafe {
        glConfig.deviceSupportsGamma = qtrue;
    }
}

/*
** GLimp_SetGamma
**
** This routine should only be called if glConfig.deviceSupportsGamma is TRUE
*/
pub fn GLimp_SetGamma(g: f32) {
    #[cfg(feature = "gamecube")]
    {
        // GXGamma gamma = GX_GM_1_0;
        // if (g >= 2.2f)
        // {
        //     gamma = GX_GM_2_2;
        // }
        // else if (g >= 1.7f)
        // {
        //     gamma = GX_GM_1_7;
        // }
        // GXSetDispCopyGamma(gamma);
    }
    #[cfg(feature = "xbox")]
    unsafe {
        const maxval: c_int = 255;

        let mut ramp: D3DGAMMARAMP = core::mem::zeroed();
        for i in 0..256 {
            let inf: c_int;
            if g == 1.0 {
                inf = (maxval as f32 * i as f32 / 255.0) as c_int;
            } else {
                inf = (maxval as f32 * (i as f32 / 255.0).powf(1.0 / g) + 0.5) as c_int;
            }
            let inf = if inf < 0 {
                0
            } else if inf > maxval {
                maxval
            } else {
                inf
            };
            ramp.red[i] = inf;
            ramp.green[i] = inf;
            ramp.blue[i] = inf;
        }
        // glw_state->device->SetGammaRamp(D3DSGR_CALIBRATE, &ramp);
    }
}
