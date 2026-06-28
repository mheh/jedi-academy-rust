/*
Copyright (c) 1997-2000 Immersion Corporation

Permission to use, copy, modify, distribute, and sell this
software and its documentation may be granted without fee;
interested parties are encouraged to request permission from
    Immersion Corporation
    2158 Paragon Drive
    San Jose, CA 95131
    408-467-1900

IMMERSION DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS.
IN NO EVENT SHALL IMMERSION BE LIABLE FOR ANY SPECIAL, INDIRECT OR
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

FILE:		FeelitAPI.h

PURPOSE:	Feelit API

STARTED:	09/08/97

NOTES/REVISIONS:

23-May-2000   MPR     Added support for querying the number of effects that can be
                      downloaded to a device (FEELIT_PROPNUMEFFECTS and FEELIT_PROP_NUMEFFECTS).
                      Added a new device subtype, FEELIT_DEVICETYPEMOUSE_VIBRATION_FF.
                      Removed FEELIT_DEVICETYPE_HID.

06-Jun-2000   MPR     Added FEELIT_PROP_DEVICEID.

08-Jun-2000   MPR     Added MAKE_FEELIT_DEVICE_TYPE_DWORD.
*/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_char, c_void};

// FEELIT_VERSION
pub const FEELIT_VERSION: u32 = 0x0103;

/****************************************************************************
 *
 *      Class IDs
 *
 ****************************************************************************/

// DEFINE_GUID(CLSID_Feelit,		0x5959df60,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const CLSID_Feelit: GUID = GUID {
    Data1: 0x5959df60,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(CLSID_FeelitDevice,	0x5959df61,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const CLSID_FeelitDevice: GUID = GUID {
    Data1: 0x5959df61,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};


/****************************************************************************
 *
 *      Interfaces
 *
 ****************************************************************************/

// DEFINE_GUID(IID_IFeelit,		0x5959df62,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const IID_IFeelit: GUID = GUID {
    Data1: 0x5959df62,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(IID_IFeelitDevice,	0x5959df63,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const IID_IFeelitDevice: GUID = GUID {
    Data1: 0x5959df63,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(IID_IFeelitEffect,	0x5959df64,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const IID_IFeelitEffect: GUID = GUID {
    Data1: 0x5959df64,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(IID_IFeelitConfig,  0x900c39e0,0xcc5c,0x11d2,0x8c,0x5d,0x00,0x10,0x5a,0x17,0x8a,0xd1);
pub const IID_IFeelitConfig: GUID = GUID {
    Data1: 0x900c39e0,
    Data2: 0xcc5c,
    Data3: 0x11d2,
    Data4: [0x8c, 0x5d, 0x00, 0x10, 0x5a, 0x17, 0x8a, 0xd1],
};


/****************************************************************************
 *
 *      Predefined object types
 *
 ****************************************************************************/

// DEFINE_GUID(GUID_Feelit_XAxis,   0x5959df65,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_XAxis: GUID = GUID {
    Data1: 0x5959df65,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_YAxis,   0x5959df66,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_YAxis: GUID = GUID {
    Data1: 0x5959df66,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_ZAxis,   0x5959df67,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_ZAxis: GUID = GUID {
    Data1: 0x5959df67,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_RxAxis,  0x5959df68,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_RxAxis: GUID = GUID {
    Data1: 0x5959df68,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_RyAxis,  0x5959df69,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_RyAxis: GUID = GUID {
    Data1: 0x5959df69,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_RzAxis,  0x5959df6a,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_RzAxis: GUID = GUID {
    Data1: 0x5959df6a,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Slider,  0x5959df6b,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Slider: GUID = GUID {
    Data1: 0x5959df6b,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Button,  0x5959df6c,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Button: GUID = GUID {
    Data1: 0x5959df6c,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Key,     0x5959df6d,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Key: GUID = GUID {
    Data1: 0x5959df6d,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_POV,     0x5959df6e,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_POV: GUID = GUID {
    Data1: 0x5959df6e,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Unknown, 0x5959df6f,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Unknown: GUID = GUID {
    Data1: 0x5959df6f,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};


/****************************************************************************
 *
 *      Predefined Product GUIDs
 *
 ****************************************************************************/

// DEFINE_GUID(GUID_Feelit_Mouse,   0x99bb5400,0x2b94,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Mouse: GUID = GUID {
    Data1: 0x99bb5400,
    Data2: 0x2b94,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

/****************************************************************************
 *
 *      Force feedback effects
 *
 ****************************************************************************/


// Constant Force
// DEFINE_GUID(GUID_Feelit_ConstantForce,0x5959df71,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_ConstantForce: GUID = GUID {
    Data1: 0x5959df71,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// Ramp Force
// DEFINE_GUID(GUID_Feelit_RampForce,	0x5959df72,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_RampForce: GUID = GUID {
    Data1: 0x5959df72,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// Periodic Effects
// DEFINE_GUID(GUID_Feelit_Square,      0x5959df73,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Square: GUID = GUID {
    Data1: 0x5959df73,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Sine,        0x5959df74,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Sine: GUID = GUID {
    Data1: 0x5959df74,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Triangle,    0x5959df75,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Triangle: GUID = GUID {
    Data1: 0x5959df75,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_SawtoothUp,	0x5959df76,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_SawtoothUp: GUID = GUID {
    Data1: 0x5959df76,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_SawtoothDown,0x5959df77,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_SawtoothDown: GUID = GUID {
    Data1: 0x5959df77,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};


// Conditions
// DEFINE_GUID(GUID_Feelit_Spring,      0x5959df78,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Spring: GUID = GUID {
    Data1: 0x5959df78,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_DeviceSpring,0x5959df83,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_DeviceSpring: GUID = GUID {
    Data1: 0x5959df83,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Damper,      0x5959df79,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Damper: GUID = GUID {
    Data1: 0x5959df79,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Inertia,     0x5959df7a,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Inertia: GUID = GUID {
    Data1: 0x5959df7a,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Friction,    0x5959df7b,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Friction: GUID = GUID {
    Data1: 0x5959df7b,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Texture,		0x5959df7c,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Texture: GUID = GUID {
    Data1: 0x5959df7c,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Grid,		0x5959df7d,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Grid: GUID = GUID {
    Data1: 0x5959df7d,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// Enclosures
// DEFINE_GUID(GUID_Feelit_Enclosure,	0x5959df7f,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Enclosure: GUID = GUID {
    Data1: 0x5959df7f,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// DEFINE_GUID(GUID_Feelit_Ellipse,	    0x5959df82,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_Ellipse: GUID = GUID {
    Data1: 0x5959df82,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

// Custom Force
// DEFINE_GUID(GUID_Feelit_CustomForce, 0x5959df7e,0x2911,0x11d1,0xb0,0x49,0x00,0x20,0xaf,0x30,0x26,0x9a);
pub const GUID_Feelit_CustomForce: GUID = GUID {
    Data1: 0x5959df7e,
    Data2: 0x2911,
    Data3: 0x11d1,
    Data4: [0xb0, 0x49, 0x00, 0x20, 0xaf, 0x30, 0x26, 0x9a],
};

/****************************************************************************
 *
 *      Interfaces and Structures...
 *
 ****************************************************************************/


/****************************************************************************
 *
 *      IFeelitEffect
 *
 ****************************************************************************/

pub const FEELIT_FEFFECTTYPE_ALL: u32 = 0x00000000;

pub const FEELIT_FEFFECTTYPE_CONSTANTFORCE: u32 = 0x00000001;
pub const FEELIT_FEFFECTTYPE_RAMPFORCE: u32 = 0x00000002;
pub const FEELIT_FEFFECTTYPE_PERIODIC: u32 = 0x00000003;
pub const FEELIT_FEFFECTTYPE_CONDITION: u32 = 0x00000004;
pub const FEELIT_FEFFECTTYPE_ENCLOSURE: u32 = 0x00000005;
pub const FEELIT_FEFFECTTYPE_ELLIPSE: u32 = 0x00000006;
pub const FEELIT_FEFFECTTYPE_TEXTURE: u32 = 0x00000007;
pub const FEELIT_FEFFECTTYPE_CUSTOMFORCE: u32 = 0x000000F0;
pub const FEELIT_FEFFECTTYPE_HARDWARE: u32 = 0x000000FF;

pub const FEELIT_FEFFECTTYPE_FFATTACK: u32 = 0x00000200;
pub const FEELIT_FEFFECTTYPE_FFFADE: u32 = 0x00000400;
pub const FEELIT_FEFFECTTYPE_SATURATION: u32 = 0x00000800;
pub const FEELIT_FEFFECTTYPE_POSNEGCOEFFICIENTS: u32 = 0x00001000;
pub const FEELIT_FEFFECTTYPE_POSNEGSATURATION: u32 = 0x00002000;
pub const FEELIT_FEFFECTTYPE_DEADBAND: u32 = 0x00004000;

// #define FEELIT_EFFECTTYPE_GETTYPE(n)             LOBYTE(n)
pub const fn FEELIT_EFFECTTYPE_GETTYPE(n: u32) -> u32 {
    n & 0xFF
}

// #define FEELIT_EFFECTTYPE_GETFLAGS(n)            ( n & 0xFFFFFF00 )
pub const fn FEELIT_EFFECTTYPE_GETFLAGS(n: u32) -> u32 {
    n & 0xFFFFFF00
}

pub const FEELIT_DEGREES: u32 = 100;
pub const FEELIT_FFNOMINALMAX: u32 = 10000;
pub const FEELIT_SECONDS: u32 = 1000000;

// typedef struct FEELIT_CONSTANTFORCE {
//     LONG  lMagnitude;	/* Magnitude of the effect, in the range -10000 to 10000 */
// } FEELIT_CONSTANTFORCE, *LPFEELIT_CONSTANTFORCE;
#[repr(C)]
pub struct FEELIT_CONSTANTFORCE {
    pub lMagnitude: i32, // Magnitude of the effect, in the range -10000 to 10000
}

// typedef struct FEELIT_RAMPFORCE {
//     LONG  lStart;	/* Magnitude at start of effect. Range -10000 to 10000 */
//     LONG  lEnd;		/* Magnitude at end of effect. Range -10000 to 10000 */
// } FEELIT_RAMPFORCE, *LPFEELIT_RAMPFORCE;
#[repr(C)]
pub struct FEELIT_RAMPFORCE {
    pub lStart: i32, // Magnitude at start of effect. Range -10000 to 10000
    pub lEnd: i32,   // Magnitude at end of effect. Range -10000 to 10000
}

// typedef struct FEELIT_PERIODIC {
//     DWORD dwMagnitude;  /* Magnitude of the effect, in the range 0 to 10000 */
//     LONG  lOffset;		/* Force will be gen'd in range lOffset - dwMagnitude to lOffset + dwMagnitude */
//     DWORD dwPhase;		/* Position in cycle at wich playback begins. Range 0 - 35,999 */
//     DWORD dwPeriod;		/* Period (length of one cycle) of the effect in microseconds */
// } FEELIT_PERIODIC, *LPFEELIT_PERIODIC;
#[repr(C)]
pub struct FEELIT_PERIODIC {
    pub dwMagnitude: u32, // Magnitude of the effect, in the range 0 to 10000
    pub lOffset: i32,     // Force will be gen'd in range lOffset - dwMagnitude to lOffset + dwMagnitude
    pub dwPhase: u32,     // Position in cycle at wich playback begins. Range 0 - 35,999
    pub dwPeriod: u32,    // Period (length of one cycle) of the effect in microseconds
}

// typedef struct FEELIT_CONDITION {
//     LONG  lCenter;				/* Center-point in screen coords. Axis depends on that in FEELIT_EFFECT */
//     LONG  lPositiveCoefficient;	/* Coef. on pos. side of the offset. Range -10000 to 10000 */
//     LONG  lNegativeCoefficient;	/* Coef. on neg. side of the offset. Range -10000 to 10000 */
//     DWORD dwPositiveSaturation; /* Max force output on pos. side of offset. Range 0 to 10000 */
//     DWORD dwNegativeSaturation;	/* Max force output on neg. side of offset. Range 0 to 10000 */
//     LONG  lDeadBand;			/* Region around lOffset where condition is not active. Range 0 to 10000 */
// } FEELIT_CONDITION, *LPFEELIT_CONDITION;
#[repr(C)]
pub struct FEELIT_CONDITION {
    pub lCenter: i32,                // Center-point in screen coords. Axis depends on that in FEELIT_EFFECT
    pub lPositiveCoefficient: i32,   // Coef. on pos. side of the offset. Range -10000 to 10000
    pub lNegativeCoefficient: i32,   // Coef. on neg. side of the offset. Range -10000 to 10000
    pub dwPositiveSaturation: u32,   // Max force output on pos. side of offset. Range 0 to 10000
    pub dwNegativeSaturation: u32,   // Max force output on neg. side of offset. Range 0 to 10000
    pub lDeadBand: i32,              // Region around lOffset where condition is not active. Range 0 to 10000
}

// typedef struct FEELIT_TEXTURE {
//     DWORD dwSize;			    /* sizeof(FEELIT_TEXTURE)   */
//     LONG lOffset;				/* Offset in screen coords of first texture from left or top edge */
//     LONG lPosBumpMag;			/* Magnitude of bumps felt when mouse travels in positive direction */
//     DWORD dwPosBumpWidth;		/* Width of bumps felt when mouse travels in positive direction */
//     DWORD dwPosBumpSpacing;		/* Center-to-Center spacing of bumps felt when mouse travels in positive direction */
//     LONG lNegBumpMag;	        /* Magnitude of bumps felt when mouse travels in negative direction */
//     DWORD dwNegBumpWidth;		/* Width of bumps felt when mouse travels in negative direction */
//     DWORD dwNegBumpSpacing;		/* Center-to-Center spacing of bumps felt when mouse travels in negative direction */
// } FEELIT_TEXTURE, *LPFEELIT_TEXTURE;
#[repr(C)]
pub struct FEELIT_TEXTURE {
    pub dwSize: u32,              // sizeof(FEELIT_TEXTURE)
    pub lOffset: i32,             // Offset in screen coords of first texture from left or top edge
    pub lPosBumpMag: i32,         // Magnitude of bumps felt when mouse travels in positive direction
    pub dwPosBumpWidth: u32,      // Width of bumps felt when mouse travels in positive direction
    pub dwPosBumpSpacing: u32,    // Center-to-Center spacing of bumps felt when mouse travels in positive direction
    pub lNegBumpMag: i32,         // Magnitude of bumps felt when mouse travels in negative direction
    pub dwNegBumpWidth: u32,      // Width of bumps felt when mouse travels in negative direction
    pub dwNegBumpSpacing: u32,    // Center-to-Center spacing of bumps felt when mouse travels in negative direction
}

// typedef struct FEELIT_CUSTOMFORCE {
//     DWORD cChannels;		/* No. of channels (axes) affected by this force */
//     DWORD dwSamplePeriod;	/* Sample period in microseconds */
//     DWORD cSamples;			/* Total number of samples in the rglForceData */
//     LPLONG rglForceData;	/* long[cSamples]. Array of force values. Channels are interleaved */
// } FEELIT_CUSTOMFORCE, *LPFEELIT_CUSTOMFORCE;
#[repr(C)]
pub struct FEELIT_CUSTOMFORCE {
    pub cChannels: u32,           // No. of channels (axes) affected by this force
    pub dwSamplePeriod: u32,      // Sample period in microseconds
    pub cSamples: u32,            // Total number of samples in the rglForceData
    pub rglForceData: *mut i32,   // long[cSamples]. Array of force values. Channels are interleaved
}

// typedef struct FEELIT_ENVELOPE {
//     DWORD dwSize;			/* sizeof(FEELIT_ENVELOPE)   */
//     DWORD dwAttackLevel;	/* Ampl. for start of env., rel. to baseline. Range 0 to 10000 */
//     DWORD dwAttackTime;     /* Time, in microseconds, to reach sustain level */
//     DWORD dwFadeLevel;		/* Ampl. for end of env., rel. to baseline. Range 0 to 10000 */
//     DWORD dwFadeTime;       /* Time, in microseconds, to reach fade level */
// } FEELIT_ENVELOPE, *LPFEELIT_ENVELOPE;
#[repr(C)]
pub struct FEELIT_ENVELOPE {
    pub dwSize: u32,       // sizeof(FEELIT_ENVELOPE)
    pub dwAttackLevel: u32, // Ampl. for start of env., rel. to baseline. Range 0 to 10000
    pub dwAttackTime: u32,  // Time, in microseconds, to reach sustain level
    pub dwFadeLevel: u32,   // Ampl. for end of env., rel. to baseline. Range 0 to 10000
    pub dwFadeTime: u32,    // Time, in microseconds, to reach fade level
}

// typedef struct FEELIT_EFFECT {
//     DWORD dwSize;                   /* sizeof(FEELIT_EFFECT) */
// 	GUID guidEffect;			    /* Effect Identifier    */
// 	DWORD dwFlags;                  /* FEELIT_FEFFECT_*      */
//     DWORD dwDuration;               /* Microseconds         */
//     DWORD dwSamplePeriod;           /* RESERVED             */
//     DWORD dwGain;					/* RESERVED             */
//     DWORD dwTriggerButton;          /* RESERVED             */
//     DWORD dwTriggerRepeatInterval;  /* RESERVED             */
//     DWORD cAxes;                    /* Number of axes       */
//     LPDWORD rgdwAxes;               /* Array of axes        */
//     LPLONG rglDirection;            /* Array of directions  */
//     LPFEELIT_ENVELOPE lpEnvelope;   /* Optional             */
//     DWORD cbTypeSpecificParams;     /* Size of params       */
//     LPVOID lpvTypeSpecificParams;   /* Pointer to params    */
// 	 DWORD dwStartDelay;             /* Microseconds delay    */
// } FEELIT_EFFECT, *LPFEELIT_EFFECT;
#[repr(C)]
pub struct FEELIT_EFFECT {
    pub dwSize: u32,                    // sizeof(FEELIT_EFFECT)
    pub guidEffect: GUID,               // Effect Identifier
    pub dwFlags: u32,                   // FEELIT_FEFFECT_*
    pub dwDuration: u32,                // Microseconds
    pub dwSamplePeriod: u32,            // RESERVED
    pub dwGain: u32,                    // RESERVED
    pub dwTriggerButton: u32,           // RESERVED
    pub dwTriggerRepeatInterval: u32,   // RESERVED
    pub cAxes: u32,                     // Number of axes
    pub rgdwAxes: *mut u32,             // Array of axes
    pub rglDirection: *mut i32,         // Array of directions
    pub lpEnvelope: *mut FEELIT_ENVELOPE, // Optional
    pub cbTypeSpecificParams: u32,      // Size of params
    pub lpvTypeSpecificParams: *mut c_void, // Pointer to params
    pub dwStartDelay: u32,              // Microseconds delay
}


// Effect Flags
pub const FEELIT_FEFFECT_OBJECTIDS: u32 = 0x00000001;
pub const FEELIT_FEFFECT_OBJECTOFFSETS: u32 = 0x00000002;
pub const FEELIT_FEFFECT_CARTESIAN: u32 = 0x00000010;
pub const FEELIT_FEFFECT_POLAR: u32 = 0x00000020;
pub const FEELIT_FEFFECT_SPHERICAL: u32 = 0x00000040;

// Parameter Flags
pub const FEELIT_FPARAM_DURATION: u32 = 0x00000001;
pub const FEELIT_FPARAM_SAMPLEPERIOD: u32 = 0x00000002;
pub const FEELIT_FPARAM_GAIN: u32 = 0x00000004;
pub const FEELIT_FPARAM_TRIGGERBUTTON: u32 = 0x00000008;
pub const FEELIT_FPARAM_TRIGGERREPEATINTERVAL: u32 = 0x00000010;
pub const FEELIT_FPARAM_AXES: u32 = 0x00000020;
pub const FEELIT_FPARAM_DIRECTION: u32 = 0x00000040;
pub const FEELIT_FPARAM_ENVELOPE: u32 = 0x00000080;
pub const FEELIT_FPARAM_TYPESPECIFICPARAMS: u32 = 0x00000100;
pub const FEELIT_FPARAM_STARTDELAY: u32 = 0x00000200;
pub const FEELIT_FPARAM_ALLPARAMS: u32 = 0x000003FF;
pub const FEELIT_FPARAM_START: u32 = 0x20000000;
pub const FEELIT_FPARAM_NORESTART: u32 = 0x40000000;
pub const FEELIT_FPARAM_NODOWNLOAD: u32 = 0x80000000;

pub const FEELIT_PARAM_NOTRIGGER: u32 = 0xFFFFFFFF;

// Start Flags
pub const FEELIT_FSTART_SOLO: u32 = 0x00000001;
pub const FEELIT_FSTART_NODOWNLOAD: u32 = 0x80000000;

// Status Flags
pub const FEELIT_FSTATUS_PLAYING: u32 = 0x00000001;
pub const FEELIT_FSTATUS_EMULATED: u32 = 0x00000002;

// Stiffness Mask Flags
pub const FEELIT_FSTIFF_NONE: u32 = 0x00000000;
pub const FEELIT_FSTIFF_OUTERLEFTWALL: u32 = 0x00000001;
pub const FEELIT_FSTIFF_INNERLEFTWALL: u32 = 0x00000002;
pub const FEELIT_FSTIFF_INNERRIGHTWALL: u32 = 0x00000004;
pub const FEELIT_FSTIFF_OUTERRIGHTWALL: u32 = 0x00000008;
pub const FEELIT_FSTIFF_OUTERTOPWALL: u32 = 0x00000010;
pub const FEELIT_FSTIFF_INNERTOPWALL: u32 = 0x00000020;
pub const FEELIT_FSTIFF_INNERBOTTOMWALL: u32 = 0x00000040;
pub const FEELIT_FSTIFF_OUTERBOTTOMWALL: u32 = 0x00000080;
pub const FEELIT_FSTIFF_OUTERANYWALL: u32 = (FEELIT_FSTIFF_OUTERTOPWALL | FEELIT_FSTIFF_OUTERBOTTOMWALL | FEELIT_FSTIFF_OUTERLEFTWALL | FEELIT_FSTIFF_OUTERRIGHTWALL);
pub const FEELIT_FSTIFF_INBOUNDANYWALL: u32 = FEELIT_FSTIFF_OUTERANYWALL;
pub const FEELIT_FSTIFF_INNERANYWALL: u32 = (FEELIT_FSTIFF_INNERTOPWALL | FEELIT_FSTIFF_INNERBOTTOMWALL | FEELIT_FSTIFF_INNERLEFTWALL | FEELIT_FSTIFF_INNERRIGHTWALL);
pub const FEELIT_FSTIFF_OUTBOUNDANYWALL: u32 = FEELIT_FSTIFF_INNERANYWALL;
pub const FEELIT_FSTIFF_ANYWALL: u32 = (FEELIT_FSTIFF_OUTERANYWALL | FEELIT_FSTIFF_INNERANYWALL);

// Clipping Mask Flags
pub const FEELIT_FCLIP_NONE: u32 = 0x00000000;
pub const FEELIT_FCLIP_OUTERLEFTWALL: u32 = 0x00000001;
pub const FEELIT_FCLIP_INNERLEFTWALL: u32 = 0x00000002;
pub const FEELIT_FCLIP_INNERRIGHTWALL: u32 = 0x00000004;
pub const FEELIT_FCLIP_OUTERRIGHTWALL: u32 = 0x00000008;
pub const FEELIT_FCLIP_OUTERTOPWALL: u32 = 0x00000010;
pub const FEELIT_FCLIP_INNERTOPWALL: u32 = 0x00000020;
pub const FEELIT_FCLIP_INNERBOTTOMWALL: u32 = 0x00000040;
pub const FEELIT_FCLIP_OUTERBOTTOMWALL: u32 = 0x00000080;
pub const FEELIT_FCLIP_OUTERANYWALL: u32 = (FEELIT_FCLIP_OUTERTOPWALL | FEELIT_FCLIP_OUTERBOTTOMWALL | FEELIT_FCLIP_OUTERLEFTWALL | FEELIT_FCLIP_OUTERRIGHTWALL);
pub const FEELIT_FCLIP_INNERANYWALL: u32 = (FEELIT_FCLIP_INNERTOPWALL | FEELIT_FCLIP_INNERBOTTOMWALL | FEELIT_FCLIP_INNERLEFTWALL | FEELIT_FCLIP_INNERRIGHTWALL);
pub const FEELIT_FCLIP_ANYWALL: u32 = (FEELIT_FCLIP_OUTERANYWALL | FEELIT_FCLIP_INNERANYWALL);

// typedef struct FEELIT_EFFESCAPE {
//     DWORD   dwSize;			/* sizeof( FEELIT_EFFESCAPE ) */
//     DWORD   dwCommand;		/* Driver-specific command number */
//     LPVOID  lpvInBuffer;	/* Buffer containing data required to perform the operation */
//     DWORD   cbInBuffer;		/* Size, in bytes, of lpvInBuffer */
//     LPVOID  lpvOutBuffer;	/* Buffer in which the operation's output data is returned */
//     DWORD   cbOutBuffer;	/* Size, in bytes, of lpvOutBuffer */
// } FEELIT_EFFESCAPE, *LPFEELIT_EFFESCAPE;
#[repr(C)]
pub struct FEELIT_EFFESCAPE {
    pub dwSize: u32,          // sizeof( FEELIT_EFFESCAPE )
    pub dwCommand: u32,       // Driver-specific command number
    pub lpvInBuffer: *mut c_void, // Buffer containing data required to perform the operation
    pub cbInBuffer: u32,      // Size, in bytes, of lpvInBuffer
    pub lpvOutBuffer: *mut c_void, // Buffer in which the operation's output data is returned
    pub cbOutBuffer: u32,     // Size, in bytes, of lpvOutBuffer
}


// IFeelitEffect interface type (stub - COM interfaces not directly representable in Rust)
pub type IFeelitEffect = c_void;
pub type LPIFEELIT_EFFECT = *mut IFeelitEffect;

pub const FEELIT_FEFFECTTYPE_ALL_CONST: u32 = FEELIT_FEFFECTTYPE_ALL;

// FEELIT_ENCLOSURE
// typedef struct FEELIT_ENCLOSURE {
//     DWORD dwSize;                       /* sizeof(FEELIT_ENCLOSURE) */
//     RECT rectBoundary;                  /* Rectangle defining the boundaries of the effect, in screen coords */
//     DWORD dwTopAndBottomWallThickness;  /* Thickness (pixels) of top/bottom walls. Must be < rectOutside.Height()/2 */
//     DWORD dwLeftAndRightWallThickness;  /* Thickness (pixels) of left/right walls. Must be < rectOutside.Width()/2 */
//     LONG lTopAndBottomWallStiffness;    /* Stiffness of horizontal borders */
//     LONG lLeftAndRightWallStiffness;    /* Stiffness of vertical borders */
//     DWORD dwStiffnessMask;              /* Borders where stiffness is turned on (FEELIT_FSTIFF*) */
//     DWORD dwClippingMask;               /* Borders where clipping is turned on  (FEELIT_FCLIP*) */
//     DWORD dwTopAndBottomWallSaturation; /* Saturation level of spring effect for top/bottom borders */
//     DWORD dwLeftAndRightWallSaturation; /* Saturation level of spring effect for left/right borders */
//     LPIFEELIT_EFFECT piInsideEffect;    /* Interface pointer to effect active in inner area of the enclosure */
// } FEELIT_ENCLOSURE, *LPFEELIT_ENCLOSURE;
#[repr(C)]
pub struct FEELIT_ENCLOSURE {
    pub dwSize: u32,                       // sizeof(FEELIT_ENCLOSURE)
    pub rectBoundary: RECT,                // Rectangle defining the boundaries of the effect, in screen coords
    pub dwTopAndBottomWallThickness: u32,  // Thickness (pixels) of top/bottom walls. Must be < rectOutside.Height()/2
    pub dwLeftAndRightWallThickness: u32,  // Thickness (pixels) of left/right walls. Must be < rectOutside.Width()/2
    pub lTopAndBottomWallStiffness: i32,   // Stiffness of horizontal borders
    pub lLeftAndRightWallStiffness: i32,   // Stiffness of vertical borders
    pub dwStiffnessMask: u32,              // Borders where stiffness is turned on (FEELIT_FSTIFF*)
    pub dwClippingMask: u32,               // Borders where clipping is turned on  (FEELIT_FCLIP*)
    pub dwTopAndBottomWallSaturation: u32, // Saturation level of spring effect for top/bottom borders
    pub dwLeftAndRightWallSaturation: u32, // Saturation level of spring effect for left/right borders
    pub piInsideEffect: LPIFEELIT_EFFECT,  // Interface pointer to effect active in inner area of the enclosure
}

// typedef struct FEELIT_ELLIPSE {
//     DWORD dwSize;                       /* sizeof(FEELIT_ELLIPSE) */
//     RECT  rectBoundary;                 /* Rectangle which circumscribes the ellipse (screen coords) */
//     DWORD dwWallThickness;              /* Thickness (pixels) of ellipse wall at its thickest point */
//     LONG lStiffness;                    /* Stiffness of ellipse borders */
//     DWORD dwStiffnessMask;              /* Borders where stiffness is turned on (FEELIT_FSTIFF*) */
//     DWORD dwClippingMask;               /* Borders where clipping is turned on  (FEELIT_FCLIP*) */
//     DWORD dwSaturation;                 /* Saturation level of spring effect for ellipse borders */
//     LPIFEELIT_EFFECT piInsideEffect;    /* Interface pointer to effect active in inner area of the ellipse */
// } FEELIT_ELLIPSE, *LPFEELIT_ELLIPSE;
#[repr(C)]
pub struct FEELIT_ELLIPSE {
    pub dwSize: u32,              // sizeof(FEELIT_ELLIPSE)
    pub rectBoundary: RECT,       // Rectangle which circumscribes the ellipse (screen coords)
    pub dwWallThickness: u32,     // Thickness (pixels) of ellipse wall at its thickest point
    pub lStiffness: i32,          // Stiffness of ellipse borders
    pub dwStiffnessMask: u32,     // Borders where stiffness is turned on (FEELIT_FSTIFF*)
    pub dwClippingMask: u32,      // Borders where clipping is turned on  (FEELIT_FCLIP*)
    pub dwSaturation: u32,        // Saturation level of spring effect for ellipse borders
    pub piInsideEffect: LPIFEELIT_EFFECT, // Interface pointer to effect active in inner area of the ellipse
}


/****************************************************************************
 *
 *      IFeelitDevice
 *
 ****************************************************************************/

// Device types
pub const FEELIT_DEVICETYPE_DEVICE: u32 = 1;
pub const FEELIT_DEVICETYPE_MOUSE: u32 = 2;

// Device subtypes
pub const FEELIT_DEVICETYPEMOUSE_UNKNOWN: u32 = 1;
pub const FEELIT_DEVICETYPEMOUSE_TRADITIONAL_FF: u32 = 2;
pub const FEELIT_DEVICETYPEMOUSE_VIBRATION_FF: u32 = 3;

// Device type macros
pub const fn GET_FEELIT_DEVICE_TYPE(dwDevType: u32) -> u32 {
    dwDevType & 0xFF
}

pub const fn GET_FEELIT_DEVICE_SUBTYPE(dwDevType: u32) -> u32 {
    (dwDevType >> 8) & 0xFF
}

pub const fn MAKE_FEELIT_DEVICE_TYPE_DWORD(ucDevType: u8, ucDevSubtype: u8) -> u32 {
    (((ucDevSubtype as u32) << 8) | (ucDevType as u32))
}

// typedef struct FEELIT_DEVCAPS {
//     DWORD   dwSize;					/* sizeof( FEELIT_DEVCAPS ) */
//     DWORD   dwFlags;				/* FEELIT_FDEVCAPS_* */
//     DWORD   dwDevType;				/* FEELIT_DEVICETYPE* */
//     DWORD   dwAxes;					/* No. of axes available on the device */
//     DWORD   dwButtons;				/* No. of buttons available on the device */
//     DWORD   dwPOVs;					/* No. of point-of-view controllers on the device */
//     DWORD   dwFFSamplePeriod;		/* Min. time btwn playback of consec. raw force commands */
//     DWORD   dwFFMinTimeResolution;	/* Min. time, in microseconds, the device can resolve */
//     DWORD   dwFirmwareRevision;		/* Firmware revision number of the device */
//     DWORD   dwHardwareRevision;		/* Hardware revision number of the device */
//     DWORD   dwFFDriverVersion;		/* Version number of the device driver */
// } FEELIT_DEVCAPS, *LPFEELIT_DEVCAPS;
#[repr(C)]
pub struct FEELIT_DEVCAPS {
    pub dwSize: u32,               // sizeof( FEELIT_DEVCAPS )
    pub dwFlags: u32,              // FEELIT_FDEVCAPS_*
    pub dwDevType: u32,            // FEELIT_DEVICETYPE*
    pub dwAxes: u32,               // No. of axes available on the device
    pub dwButtons: u32,            // No. of buttons available on the device
    pub dwPOVs: u32,               // No. of point-of-view controllers on the device
    pub dwFFSamplePeriod: u32,     // Min. time btwn playback of consec. raw force commands
    pub dwFFMinTimeResolution: u32, // Min. time, in microseconds, the device can resolve
    pub dwFirmwareRevision: u32,   // Firmware revision number of the device
    pub dwHardwareRevision: u32,   // Hardware revision number of the device
    pub dwFFDriverVersion: u32,    // Version number of the device driver
}

// Device capabilities flags
pub const FEELIT_FDEVCAPS_ATTACHED: u32 = 0x00000001;
pub const FEELIT_FDEVCAPS_POLLEDDEVICE: u32 = 0x00000002;
pub const FEELIT_FDEVCAPS_EMULATED: u32 = 0x00000004;
pub const FEELIT_FDEVCAPS_POLLEDDATAFORMAT: u32 = 0x00000008;
pub const FEELIT_FDEVCAPS_FORCEFEEDBACK: u32 = 0x00000100;
pub const FEELIT_FDEVCAPS_FFATTACK: u32 = 0x00000200;
pub const FEELIT_FDEVCAPS_FFFADE: u32 = 0x00000400;
pub const FEELIT_FDEVCAPS_SATURATION: u32 = 0x00000800;
pub const FEELIT_FDEVCAPS_POSNEGCOEFFICIENTS: u32 = 0x00001000;
pub const FEELIT_FDEVCAPS_POSNEGSATURATION: u32 = 0x00002000;
pub const FEELIT_FDEVCAPS_DEADBAND: u32 = 0x00004000;


// Data Format Type Flags

pub const FEELIT_FOBJDATAFMT_ALL: u32 = 0x00000000;

pub const FEELIT_FOBJDATAFMT_RELAXIS: u32 = 0x00000001;
pub const FEELIT_FOBJDATAFMT_ABSAXIS: u32 = 0x00000002;
pub const FEELIT_FOBJDATAFMT_AXIS: u32 = 0x00000003;

pub const FEELIT_FOBJDATAFMT_PSHBUTTON: u32 = 0x00000004;
pub const FEELIT_FOBJDATAFMT_TGLBUTTON: u32 = 0x00000008;
pub const FEELIT_FOBJDATAFMT_BUTTON: u32 = 0x0000000C;

pub const FEELIT_FOBJDATAFMT_POV: u32 = 0x00000010;

pub const FEELIT_FOBJDATAFMT_COLLECTION: u32 = 0x00000040;
pub const FEELIT_FOBJDATAFMT_NODATA: u32 = 0x00000080;
pub const FEELIT_FOBJDATAFMT_FFACTUATOR: u32 = 0x01000000;
pub const FEELIT_FOBJDATAFMT_FFEFFECTTRIGGER: u32 = 0x02000000;
pub const FEELIT_FOBJDATAFMT_NOCOLLECTION: u32 = 0x00FFFF00;

pub const FEELIT_FOBJDATAFMT_ANYINSTANCE: u32 = 0x00FFFF00;
pub const FEELIT_FOBJDATAFMT_INSTANCEMASK: u32 = FEELIT_FOBJDATAFMT_ANYINSTANCE;


// Data Format Type Macros
pub const fn FEELIT_OBJDATAFMT_MAKEINSTANCE(n: u16) -> u32 {
    (n as u32) << 8
}

pub const fn FEELIT_OBJDATAFMT_GETTYPE(n: u32) -> u32 {
    n & 0xFF
}

pub const fn FEELIT_OBJDATAFMT_GETINSTANCE(n: u32) -> u32 {
    (n >> 8) & 0xFFFF
}

pub const fn FEELIT_OBJDATAFMT_ENUMCOLLECTION(n: u16) -> u32 {
    (n as u32) << 8
}


// typedef struct _FEELIT_OBJECTDATAFORMAT {
//     const GUID *pguid;	/* Unique ID for the axis, button, or other input source. */
//     DWORD   dwOfs;		/* Offset in data packet where input source data is stored */
//     DWORD   dwType;		/* Device type describing the object. (FEELIT_FOBJDATAFMT_*) */
//     DWORD   dwFlags;	/* Aspect flags.  Zero or more of FEELIT_FDEVOBJINST_ASPECT* */
// } FEELIT_OBJECTDATAFORMAT, *LPFEELIT_OBJECTDATAFORMAT;
#[repr(C)]
pub struct FEELIT_OBJECTDATAFORMAT {
    pub pguid: *const GUID, // Unique ID for the axis, button, or other input source.
    pub dwOfs: u32,         // Offset in data packet where input source data is stored
    pub dwType: u32,        // Device type describing the object. (FEELIT_FOBJDATAFMT_*)
    pub dwFlags: u32,       // Aspect flags.  Zero or more of FEELIT_FDEVOBJINST_ASPECT*
}

// typedef struct _FEELIT_DATAFORMAT {
//     DWORD   dwSize;		/* sizeof( FEELIT_DATAFORMAT ) */
//     DWORD   dwObjSize;	/* sizeof( FEELIT_OBJECTDATAFORMAT ) */
//     DWORD   dwFlags;	/* One of FEELIT_FDATAFORMAT_* */
//     DWORD   dwDataSize;	/* Size, in bytes, of data packet returned by the device */
//     DWORD   dwNumObjs;	/* Number of object in the rgodf array */
//     LPFEELIT_OBJECTDATAFORMAT rgodf;	/* Ptr to array of FEELIT_OBJECTDATAFORMAT */
// } FEELIT_DATAFORMAT, *LPFEELIT_DATAFORMAT;
#[repr(C)]
pub struct FEELIT_DATAFORMAT {
    pub dwSize: u32,        // sizeof( FEELIT_DATAFORMAT )
    pub dwObjSize: u32,     // sizeof( FEELIT_OBJECTDATAFORMAT )
    pub dwFlags: u32,       // One of FEELIT_FDATAFORMAT_*
    pub dwDataSize: u32,    // Size, in bytes, of data packet returned by the device
    pub dwNumObjs: u32,     // Number of object in the rgodf array
    pub rgodf: *mut FEELIT_OBJECTDATAFORMAT, // Ptr to array of FEELIT_OBJECTDATAFORMAT
}

// Data Format Flags
pub const FEELIT_FDATAFORMAT_ABSAXIS: u32 = 0x00000001;
pub const FEELIT_FDATAFORMAT_RELAXIS: u32 = 0x00000002;

// Predefined Data Formats
extern "C" {
    pub static c_dfFeelitMouse: FEELIT_DATAFORMAT;
}

// typedef struct FEELIT_DEVICEOBJECTINSTANCE {
//     DWORD   dwSize;					/* sizeof( FEELIT_DEVICEOBJECTINSTANCE ) */
//     GUID    guidType;				/* Optional unique ID indicating object type */
//     DWORD   dwOfs;					/* Offset within data format for data from this object */
//     DWORD   dwType;					/* Device type describing the object. (FEELIT_FOBJDATAFMT_*) */
//     DWORD   dwFlags;				/* Zero or more of FEELIT_FDEVOBJINST_* */
//     CHAR    tszName[MAX_PATH];		/* Name of object (e.g. "X-Axis") */
//     DWORD   dwFFMaxForce;			/* Mag. of max force created by actuator for this object */
//     DWORD   dwFFForceResolution;	/* Force resolution of the actuator for this object */
//     WORD    wCollectionNumber;		/* RESERVED */
//     WORD    wDesignatorIndex;		/* RESERVED */
//     WORD    wUsagePage;				/* HID usage page associated with the object */
//     WORD    wUsage;					/* HID usage associated with the object */
//     DWORD   dwDimension;			/* Dimensional units in which object's value is reported */
//     WORD    wExponent;				/* Exponent to associate with the demension */
//     WORD    wReserved;
// } FEELIT_DEVICEOBJECTINSTANCE, *LPFEELIT_DEVICEOBJECTINSTANCE;

pub const MAX_PATH: usize = 260;

#[repr(C)]
pub struct FEELIT_DEVICEOBJECTINSTANCE {
    pub dwSize: u32,                    // sizeof( FEELIT_DEVICEOBJECTINSTANCE )
    pub guidType: GUID,                 // Optional unique ID indicating object type
    pub dwOfs: u32,                     // Offset within data format for data from this object
    pub dwType: u32,                    // Device type describing the object. (FEELIT_FOBJDATAFMT_*)
    pub dwFlags: u32,                   // Zero or more of FEELIT_FDEVOBJINST_*
    pub tszName: [c_char; MAX_PATH],    // Name of object (e.g. "X-Axis")
    pub dwFFMaxForce: u32,              // Mag. of max force created by actuator for this object
    pub dwFFForceResolution: u32,       // Force resolution of the actuator for this object
    pub wCollectionNumber: u16,         // RESERVED
    pub wDesignatorIndex: u16,          // RESERVED
    pub wUsagePage: u16,                // HID usage page associated with the object
    pub wUsage: u16,                    // HID usage associated with the object
    pub dwDimension: u32,               // Dimensional units in which object's value is reported
    pub wExponent: u16,                 // Exponent to associate with the demension
    pub wReserved: u16,
}

// typedef BOOL (FAR PASCAL * LPFEELIT_ENUMDEVICEOBJECTSCALLBACK)(LPCFEELIT_DEVICEOBJECTINSTANCE, LPVOID);
pub type LPFEELIT_ENUMDEVICEOBJECTSCALLBACK = extern "C" fn(*const FEELIT_DEVICEOBJECTINSTANCE, *mut c_void) -> bool;

// Device Object Instance Flags
pub const FEELIT_FDEVOBJINST_FFACTUATOR: u32 = 0x00000001;
pub const FEELIT_FDEVOBJINST_FFEFFECTTRIGGER: u32 = 0x00000002;
pub const FEELIT_FDEVOBJINST_POLLED: u32 = 0x00008000;
pub const FEELIT_FDEVOBJINST_ASPECTPOSITION: u32 = 0x00000100;
pub const FEELIT_FDEVOBJINST_ASPECTVELOCITY: u32 = 0x00000200;
pub const FEELIT_FDEVOBJINST_ASPECTACCEL: u32 = 0x00000300;
pub const FEELIT_FDEVOBJINST_ASPECTFORCE: u32 = 0x00000400;
pub const FEELIT_FDEVOBJINST_ASPECTMASK: u32 = 0x00000F00;

// typedef struct FEELIT_PROPHEADER {
//     DWORD   dwSize;			/* Size of enclosing struct, to which this struct is header */
//     DWORD   dwHeaderSize;	/* sizeof ( FEELIT_PROPHEADER ) */
//     DWORD   dwObj;			/* Object for which the property is to be accessed */
//     DWORD   dwHow;			/* Specifies how dwObj is interpreted. ( FEELIT_FPROPHEADER_* ) */
// } FEELIT_PROPHEADER, *LPFEELIT_PROPHEADER;
#[repr(C)]
pub struct FEELIT_PROPHEADER {
    pub dwSize: u32,       // Size of enclosing struct, to which this struct is header
    pub dwHeaderSize: u32, // sizeof ( FEELIT_PROPHEADER )
    pub dwObj: u32,        // Object for which the property is to be accessed
    pub dwHow: u32,        // Specifies how dwObj is interpreted. ( FEELIT_FPROPHEADER_* )
}

// Prop header flags
pub const FEELIT_FPROPHEADER_DEVICE: u32 = 0;
pub const FEELIT_FPROPHEADER_BYOFFSET: u32 = 1;
pub const FEELIT_FPROPHEADER_BYID: u32 = 2;

// typedef struct FEELIT_PROPDWORD {
//     FEELIT_PROPHEADER feelitph;	/* Feelit property header struct */
//     DWORD   dwData;				/* Property-specific value being retrieved */
// } FEELIT_PROPDWORD, *LPFEELIT_PROPDWORD;
#[repr(C)]
pub struct FEELIT_PROPDWORD {
    pub feelitph: FEELIT_PROPHEADER, // Feelit property header struct
    pub dwData: u32,                 // Property-specific value being retrieved
}

// typedef struct FEELIT_PROPRANGE {
//     FEELIT_PROPHEADER feelitph;	/* Feelit property header struct */
//     LONG    lMin;				/* Lower limit of range, inclusive */
//     LONG    lMax;				/* Upper limit of range, inclusive */
// } FEELIT_PROPRANGE, *LPFEELIT_PROPRANGE;
#[repr(C)]
pub struct FEELIT_PROPRANGE {
    pub feelitph: FEELIT_PROPHEADER, // Feelit property header struct
    pub lMin: i32,                   // Lower limit of range, inclusive
    pub lMax: i32,                   // Upper limit of range, inclusive
}

pub const FEELIT_PROPRANGE_NOMIN: i32 = 0x80000000i32;
pub const FEELIT_PROPRANGE_NOMAX: i32 = 0x7FFFFFFFi32;


// #define MAKE_FEELIT_PROP(prop)    (*(const GUID *)(prop))  -- C++ version
// #define MAKE_FEELIT_PROP(prop)    ((REFGUID)(prop))         -- C version

pub const FEELIT_PROP_BUFFERSIZE: u32 = 1;
pub const FEELIT_PROP_AXISMODE: u32 = 2;
pub const FEELIT_PROP_GRANULARITY: u32 = 3;
pub const FEELIT_PROP_RANGE: u32 = 4;
pub const FEELIT_PROP_DEADZONE: u32 = 5;
pub const FEELIT_PROP_SATURATION: u32 = 6;
pub const FEELIT_PROP_FFGAIN: u32 = 7;
pub const FEELIT_PROP_FFLOAD: u32 = 8;
pub const FEELIT_PROP_AUTOCENTER: u32 = 9;
pub const FEELIT_PROP_CALIBRATIONMODE: u32 = 10;
pub const FEELIT_PROP_DEVICEGAIN: u32 = 11;
pub const FEELIT_PROP_BALLISTICS: u32 = 12;
pub const FEELIT_PROP_SCREENSIZE: u32 = 13;
pub const FEELIT_PROP_ABSOLUTEMODE: u32 = 14;
pub const FEELIT_PROP_DEVICEMODE: u32 = 15;
pub const FEELIT_PROP_NUMEFFECTS: u32 = 16;
pub const FEELIT_PROP_DEVICEID: u32 = 17;

pub const FEELIT_PROPAXISMODE_ABS: u32 = 0;
pub const FEELIT_PROPAXISMODE_REL: u32 = 1;

pub const FEELIT_PROPAUTOCENTER_OFF: u32 = 0;
pub const FEELIT_PROPAUTOCENTER_ON: u32 = 1;

pub const FEELIT_PROPCALIBRATIONMODE_COOKED: u32 = 0;
pub const FEELIT_PROPCALIBRATIONMODE_RAW: u32 = 1;

// Device configuration/control, for use by control panels
// typedef struct FEELIT_PROPBALLISTICS {
//     FEELIT_PROPHEADER feelitph;	/* Feelit property header struct */
//     INT   Sensitivity;			/* Property-specific value */
//     INT   LowThreshhold;		/* Property-specific value */
//     INT   HighThreshhold;		/* Property-specific value */
// } FEELIT_PROPBALLISTICS, *LPFEELIT_PROPBALLISTICS;
#[repr(C)]
pub struct FEELIT_PROPBALLISTICS {
    pub feelitph: FEELIT_PROPHEADER, // Feelit property header struct
    pub Sensitivity: i32,            // Property-specific value
    pub LowThreshhold: i32,          // Property-specific value
    pub HighThreshhold: i32,         // Property-specific value
}

// typedef struct FEELIT_PROPSCREENSIZE {
//     FEELIT_PROPHEADER feelitph;	/* Feelit property header struct */
//     DWORD   dwXScreenSize;		/* Max X screen coord value */
//     DWORD   dwYScreenSize;		/* Max Y screen coord value */
// } FEELIT_PROPSCREENSIZE, *LPFEELIT_PROPSCREENSIZE;
#[repr(C)]
pub struct FEELIT_PROPSCREENSIZE {
    pub feelitph: FEELIT_PROPHEADER, // Feelit property header struct
    pub dwXScreenSize: u32,          // Max X screen coord value
    pub dwYScreenSize: u32,          // Max Y screen coord value
}

// typedef struct FEELIT_PROPABSOLUTEMODE {
// 	FEELIT_PROPHEADER feelitph;	/* Feelit property header struct */
// 	BOOL bAbsMode;				/* TRUE for Absolute mode FALSE for Relative mode */
// } FEELIT_PROPABSOLUTEMODE, *LPFEELIT_PROPABSOLUTEMODE;
#[repr(C)]
pub struct FEELIT_PROPABSOLUTEMODE {
    pub feelitph: FEELIT_PROPHEADER, // Feelit property header struct
    pub bAbsMode: bool,              // TRUE for Absolute mode FALSE for Relative mode
}

pub const FEELIT_PROPDEVICEMODE_MOUSE: u32 = 1;
pub const FEELIT_PROPDEVICEMODE_JOYSTICK: u32 = 2;

// Number of effects.  This is the number of effects (not the number of effect types)
// that can be downloaded to a device.  dwTotalEffects includes any caching cababilities
// of the driver.  dwHardwareEffects is strictly the number of effects that can be
// stored in the device hardware.  Pay attention to dwHardwareEffects only if you're
// worried about optimizing for speed.
// typedef struct FEELIT_PROPNUMEFFECTS {
//     FEELIT_PROPHEADER feelitph;	/* Feelit property header struct */
//     DWORD   dwTotalEffects;	    /* Total number of effects a device driver can support */
//     DWORD   dwHardwareEffects;	/* Number of effects the device hardware can hold */
// } FEELIT_PROPNUMEFFECTS, *LPFEELIT_PROPNUMEFFECTS;
#[repr(C)]
pub struct FEELIT_PROPNUMEFFECTS {
    pub feelitph: FEELIT_PROPHEADER, // Feelit property header struct
    pub dwTotalEffects: u32,         // Total number of effects a device driver can support
    pub dwHardwareEffects: u32,      // Number of effects the device hardware can hold
}


// typedef struct FEELIT_DEVICEOBJECTDATA {
//     DWORD   dwOfs;			/* Offset into current data format of this data's object */
//     DWORD   dwData;			/* Data obtained from the device */
//     DWORD   dwTimeStamp;	/* Tick count, in milliseconds, at which event was generated */
//     DWORD   dwSequence;		/* Sequence number for this event */
// } FEELIT_DEVICEOBJECTDATA, *LPFEELIT_DEVICEOBJECTDATA;
#[repr(C)]
pub struct FEELIT_DEVICEOBJECTDATA {
    pub dwOfs: u32,      // Offset into current data format of this data's object
    pub dwData: u32,     // Data obtained from the device
    pub dwTimeStamp: u32, // Tick count, in milliseconds, at which event was generated
    pub dwSequence: u32,  // Sequence number for this event
}

// #define FEELIT_SEQUENCE_COMPARE(dwSequence1, cmp, dwSequence2) \
//                         ((int)((dwSequence1) - (dwSequence2)) cmp 0)

// GetDeviceData Flags
pub const FEELIT_FGETDEVDATA_PEEK: u32 = 0x00000001;

// Cooperative Level Flags
pub const FEELIT_FCOOPLEVEL_EXCLUSIVE: u32 = 0x00000001;
pub const FEELIT_FCOOPLEVEL_NONEXCLUSIVE: u32 = 0x00000002;
pub const FEELIT_FCOOPLEVEL_FOREGROUND: u32 = 0x00000004;
pub const FEELIT_FCOOPLEVEL_BACKGROUND: u32 = 0x00000008;

// typedef struct FEELIT_DEVICEINSTANCE {
//     DWORD   dwSize;						/* sizeof ( FEELIT_DEVICEINSTANCE ) */
//     GUID    guidInstance;				/* Unique id for instance of device */
//     GUID    guidProduct;				/* Unique id for the product */
//     DWORD   dwDevType;					/* Device type (FEELIT_DEVICETYPE*) */
//     CHAR    tszInstanceName[MAX_PATH];	/* Friendly name for the instance (e.g. "Feelit Mouse 1") */
//     CHAR    tszProductName[MAX_PATH];	/* Friendly name for the product (e.g. "Feelit Mouse") */
//     GUID    guidFFDriver;				/* Unique id for the driver being used for force feedback */
//     WORD    wUsagePage;					/* HID usage page code (if the device driver is a HID device) */
//     WORD    wUsage;						/* HID usage code (if the device driver is a HID device) */
// } FEELIT_DEVICEINSTANCE, *LPFEELIT_DEVICEINSTANCE;
#[repr(C)]
pub struct FEELIT_DEVICEINSTANCE {
    pub dwSize: u32,                   // sizeof ( FEELIT_DEVICEINSTANCE )
    pub guidInstance: GUID,            // Unique id for instance of device
    pub guidProduct: GUID,             // Unique id for the product
    pub dwDevType: u32,                // Device type (FEELIT_DEVICETYPE*)
    pub tszInstanceName: [c_char; MAX_PATH], // Friendly name for the instance (e.g. "Feelit Mouse 1")
    pub tszProductName: [c_char; MAX_PATH],  // Friendly name for the product (e.g. "Feelit Mouse")
    pub guidFFDriver: GUID,            // Unique id for the driver being used for force feedback
    pub wUsagePage: u16,               // HID usage page code (if the device driver is a HID device)
    pub wUsage: u16,                   // HID usage code (if the device driver is a HID device)
}

pub const FEELIT_FCOMMAND_RESET: u32 = 0x00000001;
pub const FEELIT_FCOMMAND_STOPALL: u32 = 0x00000002;
pub const FEELIT_FCOMMAND_PAUSE: u32 = 0x00000004;
pub const FEELIT_FCOMMAND_CONTINUE: u32 = 0x00000008;
pub const FEELIT_FCOMMAND_SETACTUATORSON: u32 = 0x00000010;
pub const FEELIT_FCOMMAND_SETACTUATORSOFF: u32 = 0x00000020;

pub const FEELIT_FDEVICESTATE_EMPTY: u32 = 0x00000001;
pub const FEELIT_FDEVICESTATE_STOPPED: u32 = 0x00000002;
pub const FEELIT_FDEVICESTATE_PAUSED: u32 = 0x00000004;
pub const FEELIT_FDEVICESTATE_ACTUATORSON: u32 = 0x00000010;
pub const FEELIT_FDEVICESTATE_ACTUATORSOFF: u32 = 0x00000020;
pub const FEELIT_FDEVICESTATE_POWERON: u32 = 0x00000040;
pub const FEELIT_FDEVICESTATE_POWEROFF: u32 = 0x00000080;
pub const FEELIT_FDEVICESTATE_SAFETYSWITCHON: u32 = 0x00000100;
pub const FEELIT_FDEVICESTATE_SAFETYSWITCHOFF: u32 = 0x00000200;
pub const FEELIT_FDEVICESTATE_USERFFSWITCHON: u32 = 0x00000400;
pub const FEELIT_FDEVICESTATE_USERFFSWITCHOFF: u32 = 0x00000800;
pub const FEELIT_FDEVICESTATE_DEVICELOST: u32 = 0x80000000;

// typedef struct FEELIT_EFFECTINFO {
//     DWORD   dwSize;				/* sizeof( FEELIT_EFFECTINFO ) */
//     GUID    guid;				/* Unique ID of the effect */
//     DWORD   dwEffType;			/* Zero or more of FEELIT_FEFFECTTYPE_* */
//     DWORD   dwStaticParams;		/* All params supported. Zero or more of FEELIT_FPARAM_* */
//     DWORD   dwDynamicParams;	/* Params modifiable while effect playing. (FEELIT_FPARAM_*) */
//     CHAR    tszName[MAX_PATH];	/* Name of effect (e.g. "Enclosure" ) */
// } FEELIT_EFFECTINFO, *LPFEELIT_EFFECTINFO;
#[repr(C)]
pub struct FEELIT_EFFECTINFO {
    pub dwSize: u32,           // sizeof( FEELIT_EFFECTINFO )
    pub guid: GUID,            // Unique ID of the effect
    pub dwEffType: u32,        // Zero or more of FEELIT_FEFFECTTYPE_*
    pub dwStaticParams: u32,   // All params supported. Zero or more of FEELIT_FPARAM_*
    pub dwDynamicParams: u32,  // Params modifiable while effect playing. (FEELIT_FPARAM_*)
    pub tszName: [c_char; MAX_PATH], // Name of effect (e.g. "Enclosure" )
}

// typedef BOOL (FAR PASCAL * LPFEELIT_ENUMEFFECTSCALLBACK)(LPCFEELIT_EFFECTINFO, LPVOID);
pub type LPFEELIT_ENUMEFFECTSCALLBACK = extern "C" fn(*const FEELIT_EFFECTINFO, *mut c_void) -> bool;

// typedef BOOL (FAR PASCAL * LPFEELIT_ENUMCREATEDEFFECTOBJECTSCALLBACK)(LPFEELIT_EFFECT, LPVOID);
pub type LPFEELIT_ENUMCREATEDEFFECTOBJECTSCALLBACK = extern "C" fn(*mut FEELIT_EFFECT, *mut c_void) -> bool;


/*
                Feelit Events

Feelit events are defined using a FEELIT_EVENT struct.  They are created by
passing the struct to CreateFeelitEvent, which returns an HFEELITEVENT handle.
Feelit notifies clients that Feelit Event has triggered by sending a message to
the window handle associated with the event.  Window handles are associated with
events using the hWndEventHandler param. of the FEELIT_EVENT struct. The window
message that Feelit sends to notify of an event, contains information in the
WPARAM and LPARAM as described below.

DURING INITIALIZATION:
const UINT g_wmFeelitEvent = RegisterWindowMessage( FEELIT_EVENT_MSG_STRING );

IN MESSAGE LOOP:
if ( msgID == g_wmFeelitEvent )
{
    WORD wRef = LOWORD(wParam);				// 16-bit app-defined event id
    WORD wfTriggers = HIWORD(wParam);		// Trigger Flags
    short xForce = (short) LOWORD(lParam);	// Force applied along X-axis
    short yForce = (short) HIWORD(lParam);	// Force applied along Y-axis
}

*/

pub const FEELIT_EVENT_MSG_STRING: &str = "FEELIT_EVENT_MSG";

pub type HFEELITEVENT = *mut c_void;

// typedef struct FEELIT_EVENT {
//    DWORD dwSize;                   /* sizeof(FEELIT_EVENT) */
//    HWND hWndEventHandler;			/* Handle of window to which event msgs are sent */
//    WORD wRef;						/* 16-bit app-defined value to identify the event to the app */
//    DWORD dwEventTriggerMask;		/* Mask specifying events which trigger the callback  (FEELIT_FTRIG*) */
//    LPIFEELIT_EFFECT	piEffect;		/* Effect, if any, that this event is associated with */
// } FEELIT_EVENT, *LPFEELIT_EVENT;
#[repr(C)]
pub struct FEELIT_EVENT {
    pub dwSize: u32,           // sizeof(FEELIT_EVENT)
    pub hWndEventHandler: HWND, // Handle of window to which event msgs are sent
    pub wRef: u16,             // 16-bit app-defined value to identify the event to the app
    pub dwEventTriggerMask: u32, // Mask specifying events which trigger the callback  (FEELIT_FTRIG*)
    pub piEffect: LPIFEELIT_EFFECT, // Effect, if any, that this event is associated with
}


// Event Trigger Flags

pub const FEELIT_FTRIG_NONE: u32 = 0x00000000;
pub const FEELIT_FTRIG_ENTER: u32 = 0x00000001;
pub const FEELIT_FTRIG_EXIT: u32 = 0x00000002;
pub const FEELIT_FTRIG_OUTER: u32 = 0x00000004;
pub const FEELIT_FTRIG_INBOUND: u32 = FEELIT_FTRIG_OUTER;
pub const FEELIT_FTRIG_INNER: u32 = 0x00000008;
pub const FEELIT_FTRIG_OUTBOUND: u32 = FEELIT_FTRIG_INNER;
pub const FEELIT_FTRIG_TOPWALL: u32 = 0x00000010;
pub const FEELIT_FTRIG_BOTTOMWALL: u32 = 0x00000020;
pub const FEELIT_FTRIG_LEFTWALL: u32 = 0x00000040;
pub const FEELIT_FTRIG_RIGHTWALL: u32 = 0x00000080;
pub const FEELIT_FTRIG_ANYWALL: u32 = (FEELIT_FTRIG_TOPWALL | FEELIT_FTRIG_BOTTOMWALL | FEELIT_FTRIG_LEFTWALL | FEELIT_FTRIG_RIGHTWALL);
pub const FEELIT_FTRIG_ONENTERANY: u32 = (FEELIT_FTRIG_ENTER | FEELIT_FTRIG_ANYWALL);
pub const FEELIT_FTRIG_ONEXITANY: u32 = (FEELIT_FTRIG_EXIT | FEELIT_FTRIG_ANYWALL);
pub const FEELIT_FTRIG_ONOUTERANY: u32 = (FEELIT_FTRIG_OUTER | FEELIT_FTRIG_ANYWALL);
pub const FEELIT_FTRIG_ONINBOUNDANY: u32 = FEELIT_FTRIG_ONOUTERANY;
pub const FEELIT_FTRIG_ONINNERANY: u32 = (FEELIT_FTRIG_INNER | FEELIT_FTRIG_ANYWALL);
pub const FEELIT_FTRIG_ONOUTBOUNDANY: u32 = FEELIT_FTRIG_ONINNERANY;
pub const FEELIT_FTRIG_ONANYENCLOSURE: u32 = (FEELIT_FTRIG_ONENTERANY | FEELIT_FTRIG_ONEXITANY | FEELIT_FTRIG_ONOUTERANY | FEELIT_FTRIG_ONINNERANY);

pub const FEELIT_FTRIG_ONSCROLL: u32 = 0x0000100;
pub const FEELIT_FTRIG_ONEFFECTCOMPLETION: u32 = 0x0000200;

// IFeelitDevice interface type (stub - COM interfaces not directly representable in Rust)
pub type IFeelitDevice = c_void;
pub type LPIFEELIT_DEVICE = *mut IFeelitDevice;

// IFeelit interface type (stub - COM interfaces not directly representable in Rust)
pub type IFeelit = c_void;
pub type LPIFEELIT = *mut IFeelit;

pub const FEELIT_ENUM_STOP: u32 = 0;
pub const FEELIT_ENUM_CONTINUE: u32 = 1;

// typedef BOOL (FAR PASCAL * LPFEELIT_ENUMDEVICESCALLBACK)(LPCFEELIT_DEVICEINSTANCE, LPVOID);
pub type LPFEELIT_ENUMDEVICESCALLBACK = extern "C" fn(*const FEELIT_DEVICEINSTANCE, *mut c_void) -> bool;

pub const FEELIT_FENUMDEV_ALLDEVICES: u32 = 0x00000000;
pub const FEELIT_FENUMDEV_ATTACHEDONLY: u32 = 0x00000001;
pub const FEELIT_FENUMDEV_FORCEFEEDBACK: u32 = 0x00000100;


/****************************************************************************
 *
 *      Mouse State
 *
 ****************************************************************************/

// typedef struct _FEELIT_MOUSESTATE {
//     LONG    lXpos;
//     LONG    lYpos;
//     LONG    lZpos;
//     LONG    lXforce;
//     LONG    lYforce;
//     LONG    lZforce;
//     BYTE    rgbButtons[4];
// } FEELIT_MOUSESTATE, *LPFEELIT_MOUSESTATE;
#[repr(C)]
pub struct FEELIT_MOUSESTATE {
    pub lXpos: i32,
    pub lYpos: i32,
    pub lZpos: i32,
    pub lXforce: i32,
    pub lYforce: i32,
    pub lZforce: i32,
    pub rgbButtons: [u8; 4],
}

// #define FEELIT_MOUSEOFFSET_XAXIS		FIELD_OFFSET(FEELIT_MOUSESTATE, lXpos)
// #define FEELIT_MOUSEOFFSET_YAXIS     FIELD_OFFSET(FEELIT_MOUSESTATE, lYpos)
// #define FEELIT_MOUSEOFFSET_ZAXIS     FIELD_OFFSET(FEELIT_MOUSESTATE, lZpos)
// #define FEELIT_MOUSEOFFSET_XFORCE    FIELD_OFFSET(FEELIT_MOUSESTATE, lXforce)
// #define FEELIT_MOUSEOFFSET_YFORCE    FIELD_OFFSET(FEELIT_MOUSESTATE, lYforce)
// #define FEELIT_MOUSEOFFSET_ZFORCE    FIELD_OFFSET(FEELIT_MOUSESTATE, lZforce)
// #define FEELIT_MOUSEOFFSET_BUTTON0	(FIELD_OFFSET(FEELIT_MOUSESTATE, rgbButtons) + 0)
// #define FEELIT_MOUSEOFFSET_BUTTON1	(FIELD_OFFSET(FEELIT_MOUSESTATE, rgbButtons) + 1)
// #define FEELIT_MOUSEOFFSET_BUTTON2	(FIELD_OFFSET(FEELIT_MOUSESTATE, rgbButtons) + 2)
// #define FEELIT_MOUSEOFFSET_BUTTON3	(FIELD_OFFSET(FEELIT_MOUSESTATE, rgbButtons) + 3)

// Computed offsets for FEELIT_MOUSESTATE fields (6 i32 + [u8; 4])
pub const FEELIT_MOUSEOFFSET_XAXIS: usize = 0;
pub const FEELIT_MOUSEOFFSET_YAXIS: usize = 4;
pub const FEELIT_MOUSEOFFSET_ZAXIS: usize = 8;
pub const FEELIT_MOUSEOFFSET_XFORCE: usize = 12;
pub const FEELIT_MOUSEOFFSET_YFORCE: usize = 16;
pub const FEELIT_MOUSEOFFSET_ZFORCE: usize = 20;
pub const FEELIT_MOUSEOFFSET_BUTTON0: usize = 24;
pub const FEELIT_MOUSEOFFSET_BUTTON1: usize = 25;
pub const FEELIT_MOUSEOFFSET_BUTTON2: usize = 26;
pub const FEELIT_MOUSEOFFSET_BUTTON3: usize = 27;


/****************************************************************************
 *
 *      Return Codes
 *
 ****************************************************************************/

/*
 *  The operation completed successfully.
 */
pub const FEELIT_RESULT_OK: i32 = 0; // S_OK equivalent

/*
 *  The device exists but is not currently attached.
 */
pub const FEELIT_RESULT_NOTATTACHED: i32 = 1; // S_FALSE equivalent

/*
 *  The device buffer overflowed.  Some input was lost.
 */
pub const FEELIT_RESULT_BUFFEROVERFLOW: i32 = 1; // S_FALSE equivalent

/*
 *  The change in device properties had no effect.
 */
pub const FEELIT_RESULT_PROPNOEFFECT: i32 = 1; // S_FALSE equivalent

/*
 *  The operation had no effect.
 */
pub const FEELIT_RESULT_NOEFFECT: i32 = 1; // S_FALSE equivalent

/*
 *  The device is a polled device.  As a result, device buffering
 *  will not collect any data and event notifications will not be
 *  signalled until GetDeviceState is called.
 */
pub const FEELIT_RESULT_POLLEDDEVICE: i32 = 0x00000002;

/*
 *  The parameters of the effect were successfully updated by
 *  IFeelitEffect::SetParameters, but the effect was not
 *  downloaded because the device is not exclusively acquired
 *  or because the FEELIT_FPARAM_NODOWNLOAD flag was passed.
 */
pub const FEELIT_RESULT_DOWNLOADSKIPPED: i32 = 0x00000003;

/*
 *  The parameters of the effect were successfully updated by
 *  IFeelitEffect::SetParameters, but in order to change
 *  the parameters, the effect needed to be restarted.
 */
pub const FEELIT_RESULT_EFFECTRESTARTED: i32 = 0x00000004;

/*
 *  The parameters of the effect were successfully updated by
 *  IFeelitEffect::SetParameters, but some of them were
 *  beyond the capabilities of the device and were truncated.
 */
pub const FEELIT_RESULT_TRUNCATED: i32 = 0x00000008;

/*
 *  Equal to FEELIT_RESULT_EFFECTRESTARTED | FEELIT_RESULT_TRUNCATED.
 */
pub const FEELIT_RESULT_TRUNCATEDANDRESTARTED: i32 = 0x0000000C;

/*
 *  The application requires a newer version of Feelit.
 */
pub const FEELIT_ERROR_OLDFEELITVERSION: i32 = 0x80000000; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_OLD_WIN_VERSION)

/*
 *  The application was written for an unsupported prerelease version
 *  of Feelit.
 */
pub const FEELIT_ERROR_BETAFEELITVERSION: i32 = 0x80000000; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_RMODE_APP)

/*
 *  The object could not be created due to an incompatible driver version
 *  or mismatched or incomplete driver components.
 */
pub const FEELIT_ERROR_BADDRIVERVER: i32 = 0x80000000; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_BAD_DRIVER_LEVEL)

/*
 * The device or device instance or effect is not registered with Feelit.
 */
pub const FEELIT_ERROR_DEVICENOTREG: i32 = 0x80040155; // REGDB_E_CLASSNOTREG

/*
 * The requested object does not exist.
 */
pub const FEELIT_ERROR_NOTFOUND: i32 = 0x80070002; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_FILE_NOT_FOUND)

/*
 * The requested object does not exist.
 */
pub const FEELIT_ERROR_OBJECTNOTFOUND: i32 = 0x80070002; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_FILE_NOT_FOUND)

/*
 * An invalid parameter was passed to the returning function,
 * or the object was not in a state that admitted the function
 * to be called.
 */
pub const FEELIT_ERROR_INVALIDPARAM: i32 = 0x80070057; // E_INVALIDARG

/*
 * The specified interface is not supported by the object
 */
pub const FEELIT_ERROR_NOINTERFACE: i32 = 0x80004002; // E_NOINTERFACE

/*
 * An undetermined error occured inside the Feelit subsystem
 */
pub const FEELIT_ERROR_GENERIC: i32 = 0x80004005; // E_FAIL

/*
 * The Feelit subsystem couldn't allocate sufficient memory to complete the
 * caller's request.
 */
pub const FEELIT_ERROR_OUTOFMEMORY: i32 = 0x8007000E; // E_OUTOFMEMORY

/*
 * The function called is not supported at this time
 */
pub const FEELIT_ERROR_UNSUPPORTED: i32 = 0x80004001; // E_NOTIMPL

/*
 * This object has not been initialized
 */
pub const FEELIT_ERROR_NOTINITIALIZED: i32 = 0x80040200; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_NOT_READY)

/*
 * This object is already initialized
 */
pub const FEELIT_ERROR_ALREADYINITIALIZED: i32 = 0x80040201; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_ALREADY_INITIALIZED)

/*
 * This object does not support aggregation
 */
pub const FEELIT_ERROR_NOAGGREGATION: i32 = 0x80040110; // CLASS_E_NOAGGREGATION

/*
 * Another app has a higher priority level, preventing this call from
 * succeeding.
 */
pub const FEELIT_ERROR_OTHERAPPHASPRIO: i32 = 0x80070005; // E_ACCESSDENIED

/*
 * Access to the device has been lost.  It must be re-acquired.
 */
pub const FEELIT_ERROR_INPUTLOST: i32 = 0x8007001E; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_READ_FAULT)

/*
 * The operation cannot be performed while the device is acquired.
 */
pub const FEELIT_ERROR_ACQUIRED: i32 = 0x80070AA; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_BUSY)

/*
 * The operation cannot be performed unless the device is acquired.
 */
pub const FEELIT_ERROR_NOTACQUIRED: i32 = 0x8007000C; // MAKE_HRESULT(SEVERITY_ERROR, FACILITY_WIN32, ERROR_INVALID_ACCESS)

/*
 * The specified property cannot be changed.
 */
pub const FEELIT_ERROR_READONLY: i32 = 0x80070005; // E_ACCESSDENIED

/*
 * The device already has an event notification associated with it.
 */
pub const FEELIT_ERROR_HANDLEEXISTS: i32 = 0x80070005; // E_ACCESSDENIED

/*
 * Data is not yet available.
 */
pub const E_PENDING: i32 = 0x80070007;

/*
 * Unable to perform the requested operation because the user
 * does not have sufficient privileges.
 */
pub const FEELIT_ERROR_INSUFFICIENTPRIVS: i32 = 0x80040200;

/*
 * The device is full.
 */
pub const FEELIT_ERROR_DEVICEFULL: i32 = 0x80040201;

/*
 * Not all the requested information fit into the buffer.
 */
pub const FEELIT_ERROR_MOREDATA: i32 = 0x80040202;

/*
 * The effect is not downloaded.
 */
pub const FEELIT_ERROR_NOTDOWNLOADED: i32 = 0x80040203;

/*
 *  The device cannot be reinitialized because there are still effects
 *  attached to it.
 */
pub const FEELIT_ERROR_HASEFFECTS: i32 = 0x80040204;

/*
 *  The operation cannot be performed unless the device is acquired
 *  in FEELIT_FCOOPLEVEL_EXCLUSIVE mode.
 */
pub const FEELIT_ERROR_NOTEXCLUSIVEACQUIRED: i32 = 0x80040205;

/*
 *  The effect could not be downloaded because essential information
 *  is missing.  For example, no axes have been associated with the
 *  effect, or no type-specific information has been created.
 */
pub const FEELIT_ERROR_INCOMPLETEEFFECT: i32 = 0x80040206;

/*
 *  Attempted to read buffered device data from a device that is
 *  not buffered.
 */
pub const FEELIT_ERROR_NOTBUFFERED: i32 = 0x80040207;

/*
 *  An attempt was made to modify parameters of an effect while it is
 *  playing.  Not all hardware devices support altering the parameters
 *  of an effect while it is playing.
 */
pub const FEELIT_ERROR_EFFECTPLAYING: i32 = 0x80040208;

/*
 *  An internal error occurred (inside the API or the driver)
 */
pub const FEELIT_ERROR_INTERNAL: i32 = 0x80040209;

/*
 *  Effect set referenced by a command is not the active set
 */
pub const FEELIT_ERROR_INACTIVE: i32 = 0x8004020A;

// Windows types needed for this header
pub type HWND = *mut c_void;

// GUID structure - Windows COM GUID
#[repr(C)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

// RECT structure
#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

extern "C" {
    // extern HRESULT WINAPI FeelitCreateA(HINSTANCE hinst, DWORD dwVersion, LPIFEELIT *ppFeelit, LPUNKNOWN punkOuter);
    pub fn FeelitCreateA(
        hinst: *mut c_void,
        dwVersion: u32,
        ppFeelit: *mut *mut IFeelit,
        punkOuter: *mut c_void,
    ) -> i32;
}

// #define FeelitCreate FeelitCreateA
// In Rust, FeelitCreate is equivalent to FeelitCreateA; use FeelitCreateA directly or create a wrapper
