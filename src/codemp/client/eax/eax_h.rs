/*******************************************************************\
*                                                                   *
*  EAX.H - Environmental Audio Extensions version 4.0               *
*          for OpenAL and DirectSound3D                             *
*                                                                   *
*          File revision 1.6.3 (21 February 2003)                   *
*          EAX 4.0 API Spec version 1.6                             *
*                                                                   *
\*******************************************************************/

use core::ffi::{c_int, c_ulong};

#[repr(C)]
pub struct GUID {
    pub Data1: c_ulong,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

// EAX Unified Interface (using Direct X 7) {4FF53B81-1CE0-11d3-AAB8-00A0C95949D5}
pub const CLSID_EAXDirectSound: GUID = GUID {
    Data1: 0x4ff53b81,
    Data2: 0x1ce0,
    Data3: 0x11d3,
    Data4: [0xaa, 0xb8, 0x00, 0xa0, 0xc9, 0x59, 0x49, 0xd5],
};

// EAX Unified Interface (using Direct X 8) {CA503B60-B176-11d4-A094-D0C0BF3A560C}
pub const CLSID_EAXDirectSound8: GUID = GUID {
    Data1: 0xca503b60,
    Data2: 0xb176,
    Data3: 0x11d4,
    Data4: [0xa0, 0x94, 0xd0, 0xc0, 0xbf, 0x3a, 0x56, 0x0c],
};

////////////////////////////////////////////////////////////////////////////
// Constants

pub const EAX_MAX_FXSLOTS: usize = 4;
pub const EAX_MAX_ACTIVE_FXSLOTS: usize = 2;

// The EAX_NULL_GUID is used by EAXFXSLOT_LOADEFFECT, EAXCONTEXT_PRIMARYFXSLOTID
// and EAXSOURCE_ACTIVEFXSLOTID

// {00000000-0000-0000-0000-000000000000}
pub const EAX_NULL_GUID: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
};

// The EAX_PrimaryFXSlotID GUID is used by EAXSOURCE_ACTIVEFXSLOTID

// {F317866D-924C-450C-861B-E6DAA25E7C20}
pub const EAX_PrimaryFXSlotID: GUID = GUID {
    Data1: 0xf317866d,
    Data2: 0x924c,
    Data3: 0x450c,
    Data4: [0x86, 0x1b, 0xe6, 0xda, 0xa2, 0x5e, 0x7c, 0x20],
};

////////////////////////////////////////////////////////////////////////////
// Structures

// Use this structure for EAXCONTEXT_ALL property.
#[repr(C)]
pub struct EAXCONTEXTPROPERTIES {
    pub guidPrimaryFXSlotID: GUID,
    pub flDistanceFactor: f32,
    pub flAirAbsorptionHF: f32,
    pub flHFReference: f32,
}

// Use this structure for EAXSOURCE_ALLPARAMETERS
// - all levels are hundredths of decibels
// - all delays are in seconds
//
// NOTE: This structure may change in future EAX versions.
//       It is recommended to initialize fields by name:
//              myBuffer.lDirect = 0;
//              myBuffer.lDirectHF = -200;
//              ...
//              myBuffer.dwFlags = myFlags /* see EAXSOURCEFLAGS below */ ;
//       instead of:
//              myBuffer = { 0, -200, ... , 0x00000003 };
//
#[repr(C)]
pub struct EAXSOURCEPROPERTIES {
    pub lDirect: c_int,                 // direct path level (at low and mid frequencies)
    pub lDirectHF: c_int,               // relative direct path level at high frequencies
    pub lRoom: c_int,                   // room effect level (at low and mid frequencies)
    pub lRoomHF: c_int,                 // relative room effect level at high frequencies
    pub lObstruction: c_int,            // main obstruction control (attenuation at high frequencies)
    pub flObstructionLFRatio: f32,    // obstruction low-frequency level re. main control
    pub lOcclusion: c_int,              // main occlusion control (attenuation at high frequencies)
    pub flOcclusionLFRatio: f32,      // occlusion low-frequency level re. main control
    pub flOcclusionRoomRatio: f32,    // relative occlusion control for room effect
    pub flOcclusionDirectRatio: f32,  // relative occlusion control for direct path
    pub lExclusion: c_int,              // main exlusion control (attenuation at high frequencies)
    pub flExclusionLFRatio: f32,      // exclusion low-frequency level re. main control
    pub lOutsideVolumeHF: c_int,        // outside sound cone level at high frequencies
    pub flDopplerFactor: f32,         // like DS3D flDopplerFactor but per source
    pub flRolloffFactor: f32,         // like DS3D flRolloffFactor but per source
    pub flRoomRolloffFactor: f32,     // like DS3D flRolloffFactor but for room effect
    pub flAirAbsorptionFactor: f32,   // multiplies EAXREVERB_AIRABSORPTIONHF
    pub ulFlags: c_ulong,                 // modifies the behavior of properties
}

// Use this structure for EAXSOURCE_ALLSENDPARAMETERS
// - all levels are hundredths of decibels
//
#[repr(C)]
pub struct EAXSOURCEALLSENDPROPERTIES {
    pub guidReceivingFXSlotID: GUID,
    pub lSend: c_int,                   // send level (at low and mid frequencies)
    pub lSendHF: c_int,                 // relative send level at high frequencies
    pub lOcclusion: c_int,
    pub flOcclusionLFRatio: f32,
    pub flOcclusionRoomRatio: f32,
    pub flOcclusionDirectRatio: f32,
    pub lExclusion: c_int,
    pub flExclusionLFRatio: f32,
}

// Use this structure for EAXSOURCE_ACTIVEFXSLOTID
#[repr(C)]
pub struct EAXACTIVEFXSLOTS {
    pub guidActiveFXSlots: [GUID; EAX_MAX_ACTIVE_FXSLOTS],
}

// Use this structure for EAXSOURCE_OBSTRUCTIONPARAMETERS property.
#[repr(C)]
pub struct EAXOBSTRUCTIONPROPERTIES {
    pub lObstruction: c_int,
    pub flObstructionLFRatio: f32,
}

// Use this structure for EAXSOURCE_OCCLUSIONPARAMETERS property.
#[repr(C)]
pub struct EAXOCCLUSIONPROPERTIES {
    pub lOcclusion: c_int,
    pub flOcclusionLFRatio: f32,
    pub flOcclusionRoomRatio: f32,
    pub flOcclusionDirectRatio: f32,
}

// Use this structure for EAXSOURCE_EXCLUSIONPARAMETERS property.
#[repr(C)]
pub struct EAXEXCLUSIONPROPERTIES {
    pub lExclusion: c_int,
    pub flExclusionLFRatio: f32,
}

// Use this structure for EAXSOURCE_SENDPARAMETERS properties.
#[repr(C)]
pub struct EAXSOURCESENDPROPERTIES {
    pub guidReceivingFXSlotID: GUID,
    pub lSend: c_int,
    pub lSendHF: c_int,
}

// Use this structure for EAXSOURCE_OCCLUSIONSENDPARAMETERS
#[repr(C)]
pub struct EAXSOURCEOCCLUSIONSENDPROPERTIES {
    pub guidReceivingFXSlotID: GUID,
    pub lOcclusion: c_int,
    pub flOcclusionLFRatio: f32,
    pub flOcclusionRoomRatio: f32,
    pub flOcclusionDirectRatio: f32,
}

// Use this structure for EAXSOURCE_EXCLUSIONSENDPARAMETERS
#[repr(C)]
pub struct EAXSOURCEEXCLUSIONSENDPROPERTIES {
    pub guidReceivingFXSlotID: GUID,
    pub lExclusion: c_int,
    pub flExclusionLFRatio: f32,
}

// Use this structure for EAXFXSLOT_ALLPARAMETERS
// - all levels are hundredths of decibels
//
// NOTE: This structure may change in future EAX versions.
//       It is recommended to initialize fields by name:
//              myFXSlot.guidLoadEffect = EAX_REVERB_EFFECT;
//              myFXSlot.lVolume = 0;
//              myFXSlot.lLock = 1;
//              myFXSlot.ulFlags = myFlags /* see EAXFXSLOTFLAGS below */ ;
//       instead of:
//              myFXSlot = { EAX_REVERB_EFFECT, 0, 1, 0x00000001 };
//
#[repr(C)]
pub struct EAXFXSLOTPROPERTIES {
    pub guidLoadEffect: GUID,
    pub lVolume: c_int,
    pub lLock: c_int,
    pub ulFlags: c_ulong,
}

// Use this structure for EAXREVERB_REFLECTIONSPAN and EAXREVERB_REVERBPAN properties.
#[repr(C)]
pub struct EAXVECTOR {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

////////////////////////////////////////////////////////////////////////////
// Error Codes

pub const EAX_OK: i32 = 0;
pub const EAXERR_INVALID_OPERATION: i32 = -1;
pub const EAXERR_INVALID_VALUE: i32 = -2;
pub const EAXERR_NO_EFFECT_LOADED: i32 = -3;
pub const EAXERR_UNKNOWN_EFFECT: i32 = -4;

////////////////////////////////////////////////////////////////////////////
// Context Object

// {1D4870AD-0DEF-43c0-A40C-523632296342}
pub const EAXPROPERTYID_EAX40_Context: GUID = GUID {
    Data1: 0x1d4870ad,
    Data2: 0x0def,
    Data3: 0x43c0,
    Data4: [0xa4, 0x0c, 0x52, 0x36, 0x32, 0x29, 0x63, 0x42],
};

// For compatibility with future EAX versions:
pub const EAXPROPERTYID_EAX_Context: GUID = EAXPROPERTYID_EAX40_Context;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXCONTEXT_PROPERTY {
    EAXCONTEXT_NONE = 0,
    EAXCONTEXT_ALLPARAMETERS,
    EAXCONTEXT_PRIMARYFXSLOTID,
    EAXCONTEXT_DISTANCEFACTOR,
    EAXCONTEXT_AIRABSORPTIONHF,
    EAXCONTEXT_HFREFERENCE,
    EAXCONTEXT_LASTERROR,
}

// OR these flags with property id
pub const EAXCONTEXT_PARAMETER_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXCONTEXT_PARAMETER_DEFER: u32 = 0x80000000; // changes take effect later
pub const EAXCONTEXT_PARAMETER_COMMITDEFERREDSETTINGS: u32 = EAXCONTEXT_NONE as u32 | EAXCONTEXT_PARAMETER_IMMEDIATE;

// EAX Context property ranges and defaults:
pub const EAXCONTEXT_DEFAULTPRIMARYFXSLOTID: GUID = EAXPROPERTYID_EAX40_FXSlot0;

pub const EAXCONTEXT_MINDISTANCEFACTOR: f32 = f32::MIN_POSITIVE;
pub const EAXCONTEXT_MAXDISTANCEFACTOR: f32 = f32::MAX;
pub const EAXCONTEXT_DEFAULTDISTANCEFACTOR: f32 = 1.0f32;

pub const EAXCONTEXT_MINAIRABSORPTIONHF: f32 = -100.0f32;
pub const EAXCONTEXT_MAXAIRABSORPTIONHF: f32 = 0.0f32;
pub const EAXCONTEXT_DEFAULTAIRABSORPTIONHF: f32 = -5.0f32;

pub const EAXCONTEXT_MINHFREFERENCE: f32 = 1000.0f32;
pub const EAXCONTEXT_MAXHFREFERENCE: f32 = 20000.0f32;
pub const EAXCONTEXT_DEFAULTHFREFERENCE: f32 = 5000.0f32;

pub const EAXCONTEXT_DEFAULTLASTERROR: i32 = EAX_OK;

////////////////////////////////////////////////////////////////////////////
// Effect Slot Objects

// {C4D79F1E-F1AC-436b-A81D-A738E7045469}
pub const EAXPROPERTYID_EAX40_FXSlot0: GUID = GUID {
    Data1: 0xc4d79f1e,
    Data2: 0xf1ac,
    Data3: 0x436b,
    Data4: [0xa8, 0x1d, 0xa7, 0x38, 0xe7, 0x04, 0x54, 0x69],
};

// {08C00E96-74BE-4491-93AA-E8AD35A49117}
pub const EAXPROPERTYID_EAX40_FXSlot1: GUID = GUID {
    Data1: 0x08c00e96,
    Data2: 0x74be,
    Data3: 0x4491,
    Data4: [0x93, 0xaa, 0xe8, 0xad, 0x35, 0xa4, 0x91, 0x17],
};

// {1D433B88-F0F6-4637-919F-60E7E06B5EDD}
pub const EAXPROPERTYID_EAX40_FXSlot2: GUID = GUID {
    Data1: 0x1d433b88,
    Data2: 0xf0f6,
    Data3: 0x4637,
    Data4: [0x91, 0x9f, 0x60, 0xe7, 0xe0, 0x6b, 0x5e, 0xdd],
};

// {EFFF08EA-C7D8-44ab-93AD-6DBD5F910064}
pub const EAXPROPERTYID_EAX40_FXSlot3: GUID = GUID {
    Data1: 0xefff08ea,
    Data2: 0xc7d8,
    Data3: 0x44ab,
    Data4: [0x93, 0xad, 0x6d, 0xbd, 0x5f, 0x91, 0x00, 0x64],
};

// For compatibility with future EAX versions:
pub const EAXPROPERTYID_EAX_FXSlot0: GUID = EAXPROPERTYID_EAX40_FXSlot0;
pub const EAXPROPERTYID_EAX_FXSlot1: GUID = EAXPROPERTYID_EAX40_FXSlot1;
pub const EAXPROPERTYID_EAX_FXSlot2: GUID = EAXPROPERTYID_EAX40_FXSlot2;
pub const EAXPROPERTYID_EAX_FXSlot3: GUID = EAXPROPERTYID_EAX40_FXSlot3;

// FXSlot object properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXFXSLOT_PROPERTY {
    EAXFXSLOT_PARAMETER = 0, // range 0-0x40 reserved for loaded effect parameters
    EAXFXSLOT_NONE = 0x10000,
    EAXFXSLOT_ALLPARAMETERS,
    EAXFXSLOT_LOADEFFECT,
    EAXFXSLOT_VOLUME,
    EAXFXSLOT_LOCK,
    EAXFXSLOT_FLAGS,
}

// Note: The number and order of flags may change in future EAX versions.
//       To insure future compatibility, use flag defines as follows:
//              myFlags = EAXFXSLOTFLAGS_ENVIRONMENT;
//       instead of:
//              myFlags = 0x00000001;
//
pub const EAXFXSLOTFLAGS_ENVIRONMENT: u32 = 0x00000001;
pub const EAXFXSLOTFLAGS_RESERVED: u32 = 0xFFFFFFFE; // reserved future use

// EAX Effect Slot property ranges and defaults:
pub const EAXFXSLOT_MINVOLUME: c_int = -10000;
pub const EAXFXSLOT_MAXVOLUME: c_int = 0;
pub const EAXFXSLOT_DEFAULTVOLUME: c_int = 0;

pub const EAXFXSLOT_MINLOCK: c_int = 0;
pub const EAXFXSLOT_MAXLOCK: c_int = 1;

pub const EAXFXSLOT_UNLOCKED: u32 = 0;
pub const EAXFXSLOT_LOCKED: u32 = 1;

pub const EAXFXSLOT_DEFAULTFLAGS: u32 = EAXFXSLOTFLAGS_ENVIRONMENT;

////////////////////////////////////////////////////////////////////////////
// Source Object

// {1B86B823-22DF-4eae-8B3C-1278CE544227}
pub const EAXPROPERTYID_EAX40_Source: GUID = GUID {
    Data1: 0x1b86b823,
    Data2: 0x22df,
    Data3: 0x4eae,
    Data4: [0x8b, 0x3c, 0x12, 0x78, 0xce, 0x54, 0x42, 0x27],
};

// For compatibility with future EAX versions:
pub const EAXPROPERTYID_EAX_Source: GUID = EAXPROPERTYID_EAX40_Source;

// Source object properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXSOURCE_PROPERTY {
    EAXSOURCE_NONE = 0,
    EAXSOURCE_ALLPARAMETERS,
    EAXSOURCE_OBSTRUCTIONPARAMETERS,
    EAXSOURCE_OCCLUSIONPARAMETERS,
    EAXSOURCE_EXCLUSIONPARAMETERS,
    EAXSOURCE_DIRECT,
    EAXSOURCE_DIRECTHF,
    EAXSOURCE_ROOM,
    EAXSOURCE_ROOMHF,
    EAXSOURCE_OBSTRUCTION,
    EAXSOURCE_OBSTRUCTIONLFRATIO,
    EAXSOURCE_OCCLUSION,
    EAXSOURCE_OCCLUSIONLFRATIO,
    EAXSOURCE_OCCLUSIONROOMRATIO,
    EAXSOURCE_OCCLUSIONDIRECTRATIO,
    EAXSOURCE_EXCLUSION,
    EAXSOURCE_EXCLUSIONLFRATIO,
    EAXSOURCE_OUTSIDEVOLUMEHF,
    EAXSOURCE_DOPPLERFACTOR,
    EAXSOURCE_ROLLOFFFACTOR,
    EAXSOURCE_ROOMROLLOFFFACTOR,
    EAXSOURCE_AIRABSORPTIONFACTOR,
    EAXSOURCE_FLAGS,
    EAXSOURCE_SENDPARAMETERS,
    EAXSOURCE_ALLSENDPARAMETERS,
    EAXSOURCE_OCCLUSIONSENDPARAMETERS,
    EAXSOURCE_EXCLUSIONSENDPARAMETERS,
    EAXSOURCE_ACTIVEFXSLOTID,
}

// OR these flags with property id
pub const EAXSOURCE_PARAMETER_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXSOURCE_PARAMETER_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXSOURCE_PARAMETER_COMMITDEFERREDSETTINGS: u32 = EAXSOURCE_NONE as u32 | EAXSOURCE_PARAMETER_IMMEDIATE;

// Used by EAXSOURCE_FLAGS for EAXSOURCEFLAGS_xxxAUTO
//    TRUE:    value is computed automatically - property is an offset
//    FALSE:   value is used directly
//
// Note: The number and order of flags may change in future EAX versions.
//       To insure future compatibility, use flag defines as follows:
//              myFlags = EAXSOURCE_DIRECTHFAUTO | EAXSOURCE_ROOMAUTO;
//       instead of:
//              myFlags = 0x00000003;
//
pub const EAXSOURCEFLAGS_DIRECTHFAUTO: u32 = 0x00000001; // relates to EAXSOURCE_DIRECTHF
pub const EAXSOURCEFLAGS_ROOMAUTO: u32 = 0x00000002; // relates to EAXSOURCE_ROOM
pub const EAXSOURCEFLAGS_ROOMHFAUTO: u32 = 0x00000004; // relates to EAXSOURCE_ROOMHF
pub const EAXSOURCEFLAGS_RESERVED: u32 = 0xFFFFFFF8; // reserved future use

// EAX Source property ranges and defaults:
pub const EAXSOURCE_MINSEND: c_int = -10000;
pub const EAXSOURCE_MAXSEND: c_int = 0;
pub const EAXSOURCE_DEFAULTSEND: c_int = 0;

pub const EAXSOURCE_MINSENDHF: c_int = -10000;
pub const EAXSOURCE_MAXSENDHF: c_int = 0;
pub const EAXSOURCE_DEFAULTSENDHF: c_int = 0;

pub const EAXSOURCE_MINDIRECT: c_int = -10000;
pub const EAXSOURCE_MAXDIRECT: c_int = 1000;
pub const EAXSOURCE_DEFAULTDIRECT: c_int = 0;

pub const EAXSOURCE_MINDIRECTHF: c_int = -10000;
pub const EAXSOURCE_MAXDIRECTHF: c_int = 0;
pub const EAXSOURCE_DEFAULTDIRECTHF: c_int = 0;

pub const EAXSOURCE_MINROOM: c_int = -10000;
pub const EAXSOURCE_MAXROOM: c_int = 1000;
pub const EAXSOURCE_DEFAULTROOM: c_int = 0;

pub const EAXSOURCE_MINROOMHF: c_int = -10000;
pub const EAXSOURCE_MAXROOMHF: c_int = 0;
pub const EAXSOURCE_DEFAULTROOMHF: c_int = 0;

pub const EAXSOURCE_MINOBSTRUCTION: c_int = -10000;
pub const EAXSOURCE_MAXOBSTRUCTION: c_int = 0;
pub const EAXSOURCE_DEFAULTOBSTRUCTION: c_int = 0;

pub const EAXSOURCE_MINOBSTRUCTIONLFRATIO: f32 = 0.0f32;
pub const EAXSOURCE_MAXOBSTRUCTIONLFRATIO: f32 = 1.0f32;
pub const EAXSOURCE_DEFAULTOBSTRUCTIONLFRATIO: f32 = 0.0f32;

pub const EAXSOURCE_MINOCCLUSION: c_int = -10000;
pub const EAXSOURCE_MAXOCCLUSION: c_int = 0;
pub const EAXSOURCE_DEFAULTOCCLUSION: c_int = 0;

pub const EAXSOURCE_MINOCCLUSIONLFRATIO: f32 = 0.0f32;
pub const EAXSOURCE_MAXOCCLUSIONLFRATIO: f32 = 1.0f32;
pub const EAXSOURCE_DEFAULTOCCLUSIONLFRATIO: f32 = 0.25f32;

pub const EAXSOURCE_MINOCCLUSIONROOMRATIO: f32 = 0.0f32;
pub const EAXSOURCE_MAXOCCLUSIONROOMRATIO: f32 = 10.0f32;
pub const EAXSOURCE_DEFAULTOCCLUSIONROOMRATIO: f32 = 1.5f32;

pub const EAXSOURCE_MINOCCLUSIONDIRECTRATIO: f32 = 0.0f32;
pub const EAXSOURCE_MAXOCCLUSIONDIRECTRATIO: f32 = 10.0f32;
pub const EAXSOURCE_DEFAULTOCCLUSIONDIRECTRATIO: f32 = 1.0f32;

pub const EAXSOURCE_MINEXCLUSION: c_int = -10000;
pub const EAXSOURCE_MAXEXCLUSION: c_int = 0;
pub const EAXSOURCE_DEFAULTEXCLUSION: c_int = 0;

pub const EAXSOURCE_MINEXCLUSIONLFRATIO: f32 = 0.0f32;
pub const EAXSOURCE_MAXEXCLUSIONLFRATIO: f32 = 1.0f32;
pub const EAXSOURCE_DEFAULTEXCLUSIONLFRATIO: f32 = 1.0f32;

pub const EAXSOURCE_MINOUTSIDEVOLUMEHF: c_int = -10000;
pub const EAXSOURCE_MAXOUTSIDEVOLUMEHF: c_int = 0;
pub const EAXSOURCE_DEFAULTOUTSIDEVOLUMEHF: c_int = 0;

pub const EAXSOURCE_MINDOPPLERFACTOR: f32 = 0.0f32;
pub const EAXSOURCE_MAXDOPPLERFACTOR: f32 = 10.0f32;
pub const EAXSOURCE_DEFAULTDOPPLERFACTOR: f32 = 1.0f32;

pub const EAXSOURCE_MINROLLOFFFACTOR: f32 = 0.0f32;
pub const EAXSOURCE_MAXROLLOFFFACTOR: f32 = 10.0f32;
pub const EAXSOURCE_DEFAULTROLLOFFFACTOR: f32 = 0.0f32;

pub const EAXSOURCE_MINROOMROLLOFFFACTOR: f32 = 0.0f32;
pub const EAXSOURCE_MAXROOMROLLOFFFACTOR: f32 = 10.0f32;
pub const EAXSOURCE_DEFAULTROOMROLLOFFFACTOR: f32 = 0.0f32;

pub const EAXSOURCE_MINAIRABSORPTIONFACTOR: f32 = 0.0f32;
pub const EAXSOURCE_MAXAIRABSORPTIONFACTOR: f32 = 10.0f32;
pub const EAXSOURCE_DEFAULTAIRABSORPTIONFACTOR: f32 = 0.0f32;

pub const EAXSOURCE_DEFAULTFLAGS: u32 = EAXSOURCEFLAGS_DIRECTHFAUTO |
                                        EAXSOURCEFLAGS_ROOMAUTO |
                                        EAXSOURCEFLAGS_ROOMHFAUTO;

pub const EAXSOURCE_DEFAULTACTIVEFXSLOTID: [GUID; 2] = [
    EAX_NULL_GUID,
    EAX_PrimaryFXSlotID,
];

////////////////////////////////////////////////////////////////////////////
// Reverb Effect

// EAX REVERB {0CF95C8F-A3CC-4849-B0B6-832ECC1822DF}
pub const EAX_REVERB_EFFECT: GUID = GUID {
    Data1: 0x0cf95c8f,
    Data2: 0xa3cc,
    Data3: 0x4849,
    Data4: [0xb0, 0xb6, 0x83, 0x2e, 0xcc, 0x18, 0x22, 0xdf],
};

// Reverb effect properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXREVERB_PROPERTY {
    EAXREVERB_NONE = 0,
    EAXREVERB_ALLPARAMETERS,
    EAXREVERB_ENVIRONMENT,
    EAXREVERB_ENVIRONMENTSIZE,
    EAXREVERB_ENVIRONMENTDIFFUSION,
    EAXREVERB_ROOM,
    EAXREVERB_ROOMHF,
    EAXREVERB_ROOMLF,
    EAXREVERB_DECAYTIME,
    EAXREVERB_DECAYHFRATIO,
    EAXREVERB_DECAYLFRATIO,
    EAXREVERB_REFLECTIONS,
    EAXREVERB_REFLECTIONSDELAY,
    EAXREVERB_REFLECTIONSPAN,
    EAXREVERB_REVERB,
    EAXREVERB_REVERBDELAY,
    EAXREVERB_REVERBPAN,
    EAXREVERB_ECHOTIME,
    EAXREVERB_ECHODEPTH,
    EAXREVERB_MODULATIONTIME,
    EAXREVERB_MODULATIONDEPTH,
    EAXREVERB_AIRABSORPTIONHF,
    EAXREVERB_HFREFERENCE,
    EAXREVERB_LFREFERENCE,
    EAXREVERB_ROOMROLLOFFFACTOR,
    EAXREVERB_FLAGS,
}

// OR these flags with property id
pub const EAXREVERB_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXREVERB_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXREVERB_COMMITDEFERREDSETTINGS: u32 = EAXREVERB_NONE as u32 | EAXREVERB_IMMEDIATE;

// used by EAXREVERB_ENVIRONMENT
pub const EAX_ENVIRONMENT_GENERIC: u32 = 0;
pub const EAX_ENVIRONMENT_PADDEDCELL: u32 = 1;
pub const EAX_ENVIRONMENT_ROOM: u32 = 2;
pub const EAX_ENVIRONMENT_BATHROOM: u32 = 3;
pub const EAX_ENVIRONMENT_LIVINGROOM: u32 = 4;
pub const EAX_ENVIRONMENT_STONEROOM: u32 = 5;
pub const EAX_ENVIRONMENT_AUDITORIUM: u32 = 6;
pub const EAX_ENVIRONMENT_CONCERTHALL: u32 = 7;
pub const EAX_ENVIRONMENT_CAVE: u32 = 8;
pub const EAX_ENVIRONMENT_ARENA: u32 = 9;
pub const EAX_ENVIRONMENT_HANGAR: u32 = 10;
pub const EAX_ENVIRONMENT_CARPETEDHALLWAY: u32 = 11;
pub const EAX_ENVIRONMENT_HALLWAY: u32 = 12;
pub const EAX_ENVIRONMENT_STONECORRIDOR: u32 = 13;
pub const EAX_ENVIRONMENT_ALLEY: u32 = 14;
pub const EAX_ENVIRONMENT_FOREST: u32 = 15;
pub const EAX_ENVIRONMENT_CITY: u32 = 16;
pub const EAX_ENVIRONMENT_MOUNTAINS: u32 = 17;
pub const EAX_ENVIRONMENT_QUARRY: u32 = 18;
pub const EAX_ENVIRONMENT_PLAIN: u32 = 19;
pub const EAX_ENVIRONMENT_PARKINGLOT: u32 = 20;
pub const EAX_ENVIRONMENT_SEWERPIPE: u32 = 21;
pub const EAX_ENVIRONMENT_UNDERWATER: u32 = 22;
pub const EAX_ENVIRONMENT_DRUGGED: u32 = 23;
pub const EAX_ENVIRONMENT_DIZZY: u32 = 24;
pub const EAX_ENVIRONMENT_PSYCHOTIC: u32 = 25;

pub const EAX_ENVIRONMENT_UNDEFINED: u32 = 26;

pub const EAX_ENVIRONMENT_COUNT: u32 = 27;

// Used by EAXREVERB_FLAGS
//
// Note: The number and order of flags may change in future EAX versions.
//       It is recommended to use the flag defines as follows:
//              myFlags = EAXREVERBFLAGS_DECAYTIMESCALE | EAXREVERBFLAGS_REVERBSCALE;
//       instead of:
//              myFlags = 0x00000009;
//
// These flags determine what properties are affected by environment size.
pub const EAXREVERBFLAGS_DECAYTIMESCALE: u32 = 0x00000001; // reverberation decay time
pub const EAXREVERBFLAGS_REFLECTIONSSCALE: u32 = 0x00000002; // reflection level
pub const EAXREVERBFLAGS_REFLECTIONSDELAYSCALE: u32 = 0x00000004; // initial reflection delay time
pub const EAXREVERBFLAGS_REVERBSCALE: u32 = 0x00000008; // reflections level
pub const EAXREVERBFLAGS_REVERBDELAYSCALE: u32 = 0x00000010; // late reverberation delay time
pub const EAXREVERBFLAGS_ECHOTIMESCALE: u32 = 0x00000040; // echo time
pub const EAXREVERBFLAGS_MODULATIONTIMESCALE: u32 = 0x00000080; // modulation time
// This flag limits high-frequency decay time according to air absorption.
pub const EAXREVERBFLAGS_DECAYHFLIMIT: u32 = 0x00000020;
pub const EAXREVERBFLAGS_RESERVED: u32 = 0xFFFFFF00; // reserved future use

// Use this structure for EAXREVERB_ALLPARAMETERS
// - all levels are hundredths of decibels
// - all times and delays are in seconds
//
// NOTE: This structure may change in future EAX versions.
//       It is recommended to initialize fields by name:
//              myReverb.lRoom = -1000;
//              myReverb.lRoomHF = -100;
//              ...
//              myReverb.dwFlags = myFlags /* see EAXREVERBFLAGS below */ ;
//       instead of:
//              myReverb = { -1000, -100, ... , 0x00000009 };
//       If you want to save and load presets in binary form, you
//       should define your own structure to insure future compatibility.
//
#[repr(C)]
pub struct EAXREVERBPROPERTIES {
    pub ulEnvironment: c_ulong,   // sets all reverb properties
    pub flEnvironmentSize: f32,       // environment size in meters
    pub flEnvironmentDiffusion: f32,  // environment diffusion
    pub lRoom: c_int,                    // room effect level (at mid frequencies)
    pub lRoomHF: c_int,                  // relative room effect level at high frequencies
    pub lRoomLF: c_int,                  // relative room effect level at low frequencies
    pub flDecayTime: f32,             // reverberation decay time at mid frequencies
    pub flDecayHFRatio: f32,          // high-frequency to mid-frequency decay time ratio
    pub flDecayLFRatio: f32,          // low-frequency to mid-frequency decay time ratio
    pub lReflections: c_int,             // early reflections level relative to room effect
    pub flReflectionsDelay: f32,      // initial reflection delay time
    pub vReflectionsPan: EAXVECTOR,     // early reflections panning vector
    pub lReverb: c_int,                  // late reverberation level relative to room effect
    pub flReverbDelay: f32,           // late reverberation delay time relative to initial reflection
    pub vReverbPan: EAXVECTOR,          // late reverberation panning vector
    pub flEchoTime: f32,              // echo time
    pub flEchoDepth: f32,             // echo depth
    pub flModulationTime: f32,        // modulation time
    pub flModulationDepth: f32,       // modulation depth
    pub flAirAbsorptionHF: f32,       // change in level per meter at high frequencies
    pub flHFReference: f32,           // reference high frequency
    pub flLFReference: f32,           // reference low frequency
    pub flRoomRolloffFactor: f32,     // like DS3D flRolloffFactor but for room effect
    pub ulFlags: c_ulong,         // modifies the behavior of properties
}

// Property ranges and defaults:
pub const EAXREVERB_MINENVIRONMENT: c_int = 0;
pub const EAXREVERB_MAXENVIRONMENT: c_int = (EAX_ENVIRONMENT_COUNT as c_int) - 1;
pub const EAXREVERB_DEFAULTENVIRONMENT: u32 = EAX_ENVIRONMENT_GENERIC;

pub const EAXREVERB_MINENVIRONMENTSIZE: f32 = 1.0f32;
pub const EAXREVERB_MAXENVIRONMENTSIZE: f32 = 100.0f32;
pub const EAXREVERB_DEFAULTENVIRONMENTSIZE: f32 = 7.5f32;

pub const EAXREVERB_MINENVIRONMENTDIFFUSION: f32 = 0.0f32;
pub const EAXREVERB_MAXENVIRONMENTDIFFUSION: f32 = 1.0f32;
pub const EAXREVERB_DEFAULTENVIRONMENTDIFFUSION: f32 = 1.0f32;

pub const EAXREVERB_MINROOM: c_int = -10000;
pub const EAXREVERB_MAXROOM: c_int = 0;
pub const EAXREVERB_DEFAULTROOM: c_int = -1000;

pub const EAXREVERB_MINROOMHF: c_int = -10000;
pub const EAXREVERB_MAXROOMHF: c_int = 0;
pub const EAXREVERB_DEFAULTROOMHF: c_int = -100;

pub const EAXREVERB_MINROOMLF: c_int = -10000;
pub const EAXREVERB_MAXROOMLF: c_int = 0;
pub const EAXREVERB_DEFAULTROOMLF: c_int = 0;

pub const EAXREVERB_MINDECAYTIME: f32 = 0.1f32;
pub const EAXREVERB_MAXDECAYTIME: f32 = 20.0f32;
pub const EAXREVERB_DEFAULTDECAYTIME: f32 = 1.49f32;

pub const EAXREVERB_MINDECAYHFRATIO: f32 = 0.1f32;
pub const EAXREVERB_MAXDECAYHFRATIO: f32 = 2.0f32;
pub const EAXREVERB_DEFAULTDECAYHFRATIO: f32 = 0.83f32;

pub const EAXREVERB_MINDECAYLFRATIO: f32 = 0.1f32;
pub const EAXREVERB_MAXDECAYLFRATIO: f32 = 2.0f32;
pub const EAXREVERB_DEFAULTDECAYLFRATIO: f32 = 1.00f32;

pub const EAXREVERB_MINREFLECTIONS: c_int = -10000;
pub const EAXREVERB_MAXREFLECTIONS: c_int = 1000;
pub const EAXREVERB_DEFAULTREFLECTIONS: c_int = -2602;

pub const EAXREVERB_MINREFLECTIONSDELAY: f32 = 0.0f32;
pub const EAXREVERB_MAXREFLECTIONSDELAY: f32 = 0.3f32;
pub const EAXREVERB_DEFAULTREFLECTIONSDELAY: f32 = 0.007f32;

pub const EAXREVERB_DEFAULTREFLECTIONSPAN: EAXVECTOR = EAXVECTOR { x: 0.0f32, y: 0.0f32, z: 0.0f32 };

pub const EAXREVERB_MINREVERB: c_int = -10000;
pub const EAXREVERB_MAXREVERB: c_int = 2000;
pub const EAXREVERB_DEFAULTREVERB: c_int = 200;

pub const EAXREVERB_MINREVERBDELAY: f32 = 0.0f32;
pub const EAXREVERB_MAXREVERBDELAY: f32 = 0.1f32;
pub const EAXREVERB_DEFAULTREVERBDELAY: f32 = 0.011f32;

pub const EAXREVERB_DEFAULTREVERBPAN: EAXVECTOR = EAXVECTOR { x: 0.0f32, y: 0.0f32, z: 0.0f32 };

pub const EAXREVERB_MINECHOTIME: f32 = 0.075f32;
pub const EAXREVERB_MAXECHOTIME: f32 = 0.25f32;
pub const EAXREVERB_DEFAULTECHOTIME: f32 = 0.25f32;

pub const EAXREVERB_MINECHODEPTH: f32 = 0.0f32;
pub const EAXREVERB_MAXECHODEPTH: f32 = 1.0f32;
pub const EAXREVERB_DEFAULTECHODEPTH: f32 = 0.0f32;

pub const EAXREVERB_MINMODULATIONTIME: f32 = 0.04f32;
pub const EAXREVERB_MAXMODULATIONTIME: f32 = 4.0f32;
pub const EAXREVERB_DEFAULTMODULATIONTIME: f32 = 0.25f32;

pub const EAXREVERB_MINMODULATIONDEPTH: f32 = 0.0f32;
pub const EAXREVERB_MAXMODULATIONDEPTH: f32 = 1.0f32;
pub const EAXREVERB_DEFAULTMODULATIONDEPTH: f32 = 0.0f32;

pub const EAXREVERB_MINAIRABSORPTIONHF: f32 = -100.0f32;
pub const EAXREVERB_MAXAIRABSORPTIONHF: f32 = 0.0f32;
pub const EAXREVERB_DEFAULTAIRABSORPTIONHF: f32 = -5.0f32;

pub const EAXREVERB_MINHFREFERENCE: f32 = 1000.0f32;
pub const EAXREVERB_MAXHFREFERENCE: f32 = 20000.0f32;
pub const EAXREVERB_DEFAULTHFREFERENCE: f32 = 5000.0f32;

pub const EAXREVERB_MINLFREFERENCE: f32 = 20.0f32;
pub const EAXREVERB_MAXLFREFERENCE: f32 = 1000.0f32;
pub const EAXREVERB_DEFAULTLFREFERENCE: f32 = 250.0f32;

pub const EAXREVERB_MINROOMROLLOFFFACTOR: f32 = 0.0f32;
pub const EAXREVERB_MAXROOMROLLOFFFACTOR: f32 = 10.0f32;
pub const EAXREVERB_DEFAULTROOMROLLOFFFACTOR: f32 = 0.0f32;

pub const EAXREVERB_DEFAULTFLAGS: u32 = EAXREVERBFLAGS_DECAYTIMESCALE |
                                         EAXREVERBFLAGS_REFLECTIONSSCALE |
                                         EAXREVERBFLAGS_REFLECTIONSDELAYSCALE |
                                         EAXREVERBFLAGS_REVERBSCALE |
                                         EAXREVERBFLAGS_REVERBDELAYSCALE |
                                         EAXREVERBFLAGS_DECAYHFLIMIT;

////////////////////////////////////////////////////////////////////////////
// AGC Compressor Effect

// EAX AGC COMPRESSOR {BFB7A01E-7825-4039-927F-3AABDA0C560}

pub const EAX_AGCCOMPRESSOR_EFFECT: GUID = GUID {
    Data1: 0xbfb7a01e,
    Data2: 0x7825,
    Data3: 0x4039,
    Data4: [0x92, 0x7f, 0x03, 0xaa, 0xbd, 0xa0, 0xc5, 0x60],
};

// AGC Compressor properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXAGCCOMPRESSOR_PROPERTY {
    EAXAGCCOMPRESSOR_NONE = 0,
    EAXAGCCOMPRESSOR_ALLPARAMETERS,
    EAXAGCCOMPRESSOR_ONOFF,
}

// OR these flags with property id
pub const EAXAGCCOMPRESSOR_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXAGCCOMPRESSOR_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXAGCCOMPRESSOR_COMMITDEFERREDSETTINGS: u32 = EAXAGCCOMPRESSOR_NONE as u32 |
                                                 EAXAGCCOMPRESSOR_IMMEDIATE;

// Use this structure for EAXAGCCOMPRESSOR_ALLPARAMETERS
#[repr(C)]
pub struct EAXAGCCOMPRESSORPROPERTIES {
    pub ulOnOff: c_ulong,   // Switch Compressor on or off
}

// Property ranges and defaults:

pub const EAXAGCCOMPRESSOR_MINONOFF: c_int = 0;
pub const EAXAGCCOMPRESSOR_MAXONOFF: c_int = 1;
pub const EAXAGCCOMPRESSOR_DEFAULTONOFF: c_int = 1;

////////////////////////////////////////////////////////////////////////////
// Autowah Effect

// EAX AUTOWAH {EC3130C0-AC7A-11D2-88DD-A024D13CE1}
pub const EAX_AUTOWAH_EFFECT: GUID = GUID {
    Data1: 0xec3130c0,
    Data2: 0xac7a,
    Data3: 0x11d2,
    Data4: [0x88, 0xdd, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Autowah properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXAUTOWAH_PROPERTY {
    EAXAUTOWAH_NONE = 0,
    EAXAUTOWAH_ALLPARAMETERS,
    EAXAUTOWAH_ATTACKTIME,
    EAXAUTOWAH_RELEASETIME,
    EAXAUTOWAH_RESONANCE,
    EAXAUTOWAH_PEAKLEVEL,
}

// OR these flags with property id
pub const EAXAUTOWAH_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXAUTOWAH_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXAUTOWAH_COMMITDEFERREDSETTINGS: u32 = EAXAUTOWAH_NONE as u32 |
                                           EAXAUTOWAH_IMMEDIATE;

// Use this structure for EAXAUTOWAH_ALLPARAMETERS
#[repr(C)]
pub struct EAXAUTOWAHPROPERTIES {
    pub flAttackTime: f32,                // Attack time (seconds)
    pub flReleaseTime: f32,          // Release time (seconds)
    pub lResonance: c_int,             // Resonance (mB)
    pub lPeakLevel: c_int,             // Peak level (mB)
}

// Property ranges and defaults:

pub const EAXAUTOWAH_MINATTACKTIME: f32 = 0.0001f32;
pub const EAXAUTOWAH_MAXATTACKTIME: f32 = 1.0f32;
pub const EAXAUTOWAH_DEFAULTATTACKTIME: f32 = 0.06f32;

pub const EAXAUTOWAH_MINRELEASETIME: f32 = 0.0001f32;
pub const EAXAUTOWAH_MAXRELEASETIME: f32 = 1.0f32;
pub const EAXAUTOWAH_DEFAULTRELEASETIME: f32 = 0.06f32;

pub const EAXAUTOWAH_MINRESONANCE: c_int = 600;
pub const EAXAUTOWAH_MAXRESONANCE: c_int = 6000;
pub const EAXAUTOWAH_DEFAULTRESONANCE: c_int = 6000;

pub const EAXAUTOWAH_MINPEAKLEVEL: c_int = -9000;
pub const EAXAUTOWAH_MAXPEAKLEVEL: c_int = 9000;
pub const EAXAUTOWAH_DEFAULTPEAKLEVEL: c_int = 2100;

////////////////////////////////////////////////////////////////////////////
// Chorus Effect

// EAX CHORUS {DE6D6FE0-AC79-11D2-88DD-A024D13CE1}

pub const EAX_CHORUS_EFFECT: GUID = GUID {
    Data1: 0xde6d6fe0,
    Data2: 0xac79,
    Data3: 0x11d2,
    Data4: [0x88, 0xdd, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Chorus properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXCHORUS_PROPERTY {
    EAXCHORUS_NONE = 0,
    EAXCHORUS_ALLPARAMETERS,
    EAXCHORUS_WAVEFORM,
    EAXCHORUS_PHASE,
    EAXCHORUS_RATE,
    EAXCHORUS_DEPTH,
    EAXCHORUS_FEEDBACK,
    EAXCHORUS_DELAY,
}

// OR these flags with property id
pub const EAXCHORUS_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXCHORUS_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXCHORUS_COMMITDEFERREDSETTINGS: u32 = EAXCHORUS_NONE as u32 |
                                          EAXCHORUS_IMMEDIATE;

// used by EAXCHORUS_WAVEFORM
pub const EAX_CHORUS_SINUSOID: u32 = 0;
pub const EAX_CHORUS_TRIANGLE: u32 = 1;

// Use this structure for EAXCHORUS_ALLPARAMETERS
#[repr(C)]
pub struct EAXCHORUSPROPERTIES {
    pub ulWaveform: c_ulong,      // Waveform selector - see enum above
    pub lPhase: c_int,         // Phase (Degrees)
    pub flRate: f32,         // Rate (Hz)
    pub flDepth: f32,        // Depth (0 to 1)
    pub flFeedback: f32,     // Feedback (-1 to 1)
    pub flDelay: f32,        // Delay (seconds)
}

// Property ranges and defaults:

pub const EAXCHORUS_MINWAVEFORM: c_int = 0;
pub const EAXCHORUS_MAXWAVEFORM: c_int = 1;
pub const EAXCHORUS_DEFAULTWAVEFORM: c_int = 1;

pub const EAXCHORUS_MINPHASE: c_int = -180;
pub const EAXCHORUS_MAXPHASE: c_int = 180;
pub const EAXCHORUS_DEFAULTPHASE: c_int = 90;

pub const EAXCHORUS_MINRATE: f32 = 0.0f32;
pub const EAXCHORUS_MAXRATE: f32 = 10.0f32;
pub const EAXCHORUS_DEFAULTRATE: f32 = 1.1f32;

pub const EAXCHORUS_MINDEPTH: f32 = 0.0f32;
pub const EAXCHORUS_MAXDEPTH: f32 = 1.0f32;
pub const EAXCHORUS_DEFAULTDEPTH: f32 = 0.1f32;

pub const EAXCHORUS_MINFEEDBACK: f32 = -1.0f32;
pub const EAXCHORUS_MAXFEEDBACK: f32 = 1.0f32;
pub const EAXCHORUS_DEFAULTFEEDBACK: f32 = 0.25f32;

pub const EAXCHORUS_MINDELAY: f32 = 0.0f32;
pub const EAXCHORUS_MAXDELAY: f32 = 0.016f32;
pub const EAXCHORUS_DEFAULTDELAY: f32 = 0.016f32;

////////////////////////////////////////////////////////////////////////////
// Distortion Effect

// EAX DISTORTION {975A4CE0-AC7E-11D2-88DD-A024D13CE1}

pub const EAX_DISTORTION_EFFECT: GUID = GUID {
    Data1: 0x975a4ce0,
    Data2: 0xac7e,
    Data3: 0x11d2,
    Data4: [0x88, 0xdd, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Distortion properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXDISTORTION_PROPERTY {
    EAXDISTORTION_NONE = 0,
    EAXDISTORTION_ALLPARAMETERS,
    EAXDISTORTION_EDGE,
    EAXDISTORTION_GAIN,
    EAXDISTORTION_LOWPASSCUTOFF,
    EAXDISTORTION_EQCENTER,
    EAXDISTORTION_EQBANDWIDTH,
}

// OR these flags with property id
pub const EAXDISTORTION_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXDISTORTION_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXDISTORTION_COMMITDEFERREDSETTINGS: u32 = EAXDISTORTION_NONE as u32 |
                                              EAXDISTORTION_IMMEDIATE;

// Use this structure for EAXDISTORTION_ALLPARAMETERS
#[repr(C)]
pub struct EAXDISTORTIONPROPERTIES {
    pub flEdge: f32,             // Controls the shape of the distortion (0 to 1)
    pub lGain: c_int,              // Controls the post distortion gain (mB)
    pub flLowPassCutOff: f32,    // Controls the cut-off of the filter pre-distortion (Hz)
    pub flEQCenter: f32,         // Controls the center frequency of the EQ post-distortion (Hz)
    pub flEQBandwidth: f32,      // Controls the bandwidth of the EQ post-distortion (Hz)
}

// Property ranges and defaults:

pub const EAXDISTORTION_MINEDGE: f32 = 0.0f32;
pub const EAXDISTORTION_MAXEDGE: f32 = 1.0f32;
pub const EAXDISTORTION_DEFAULTEDGE: f32 = 0.2f32;

pub const EAXDISTORTION_MINGAIN: c_int = -6000;
pub const EAXDISTORTION_MAXGAIN: c_int = 0;
pub const EAXDISTORTION_DEFAULTGAIN: c_int = -2600;

pub const EAXDISTORTION_MINLOWPASSCUTOFF: f32 = 80.0f32;
pub const EAXDISTORTION_MAXLOWPASSCUTOFF: f32 = 24000.0f32;
pub const EAXDISTORTION_DEFAULTLOWPASSCUTOFF: f32 = 8000.0f32;

pub const EAXDISTORTION_MINEQCENTER: f32 = 80.0f32;
pub const EAXDISTORTION_MAXEQCENTER: f32 = 24000.0f32;
pub const EAXDISTORTION_DEFAULTEQCENTER: f32 = 3600.0f32;

pub const EAXDISTORTION_MINEQBANDWIDTH: f32 = 80.0f32;
pub const EAXDISTORTION_MAXEQBANDWIDTH: f32 = 24000.0f32;
pub const EAXDISTORTION_DEFAULTEQBANDWIDTH: f32 = 3600.0f32;

////////////////////////////////////////////////////////////////////////////
// Echo Effect

// EAX ECHO {E9F1BC0-AC82-11D2-88DD-A024D13CE1}

pub const EAX_ECHO_EFFECT: GUID = GUID {
    Data1: 0x0e9f1bc0,
    Data2: 0xac82,
    Data3: 0x11d2,
    Data4: [0x88, 0xdd, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Echo properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXECHO_PROPERTY {
    EAXECHO_NONE = 0,
    EAXECHO_ALLPARAMETERS,
    EAXECHO_DELAY,
    EAXECHO_LRDELAY,
    EAXECHO_DAMPING,
    EAXECHO_FEEDBACK,
    EAXECHO_SPREAD,
}

// OR these flags with property id
pub const EAXECHO_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXECHO_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXECHO_COMMITDEFERREDSETTINGS: u32 = EAXECHO_NONE as u32 |
                                        EAXECHO_IMMEDIATE;

// Use this structure for EAXECHO_ALLPARAMETERS
#[repr(C)]
pub struct EAXECHOPROPERTIES {
    pub flDelay: f32,            // Controls the initial delay time (seconds)
    pub flLRDelay: f32,          // Controls the delay time between the first and second taps (seconds)
    pub flDamping: f32,          // Controls a low-pass filter that dampens the echoes (0 to 1)
    pub flFeedback: f32,         // Controls the duration of echo repetition (0 to 1)
    pub flSpread: f32,           // Controls the left-right spread of the echoes
}

// Property ranges and defaults:

pub const EAXECHO_MINDAMPING: f32 = 0.0f32;
pub const EAXECHO_MAXDAMPING: f32 = 0.99f32;
pub const EAXECHO_DEFAULTDAMPING: f32 = 0.5f32;

pub const EAXECHO_MINDELAY: f32 = 0.0f32;
pub const EAXECHO_MAXDELAY: f32 = 0.207f32;
pub const EAXECHO_DEFAULTDELAY: f32 = 0.1f32;

pub const EAXECHO_MINLRDELAY: f32 = 0.0f32;
pub const EAXECHO_MAXLRDELAY: f32 = 0.404f32;
pub const EAXECHO_DEFAULTLRDELAY: f32 = 0.1f32;

pub const EAXECHO_MINFEEDBACK: f32 = 0.0f32;
pub const EAXECHO_MAXFEEDBACK: f32 = 1.0f32;
pub const EAXECHO_DEFAULTFEEDBACK: f32 = 0.5f32;

pub const EAXECHO_MINSPREAD: f32 = -1.0f32;
pub const EAXECHO_MAXSPREAD: f32 = 1.0f32;
pub const EAXECHO_DEFAULTSPREAD: f32 = -1.0f32;

////////////////////////////////////////////////////////////////////////////
// Equalizer Effect

// EAX EQUALIZER {65F94CE0-9793-11D3-939D-C0F02DD6F0}

pub const EAX_EQUALIZER_EFFECT: GUID = GUID {
    Data1: 0x65f94ce0,
    Data2: 0x9793,
    Data3: 0x11d3,
    Data4: [0x93, 0x9d, 0x00, 0xc0, 0xf0, 0x2d, 0xd6, 0xf0],
};

// Equalizer properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXEQUALIZER_PROPERTY {
    EAXEQUALIZER_NONE = 0,
    EAXEQUALIZER_ALLPARAMETERS,
    EAXEQUALIZER_LOWGAIN,
    EAXEQUALIZER_LOWCUTOFF,
    EAXEQUALIZER_MID1GAIN,
    EAXEQUALIZER_MID1CENTER,
    EAXEQUALIZER_MID1WIDTH,
    EAXEQUALIZER_MID2GAIN,
    EAXEQUALIZER_MID2CENTER,
    EAXEQUALIZER_MID2WIDTH,
    EAXEQUALIZER_HIGHGAIN,
    EAXEQUALIZER_HIGHCUTOFF,
}

// OR these flags with property id
pub const EAXEQUALIZER_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXEQUALIZER_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXEQUALIZER_COMMITDEFERREDSETTINGS: u32 = EAXEQUALIZER_NONE as u32 |
                                             EAXEQUALIZER_IMMEDIATE;

// Use this structure for EAXEQUALIZER_ALLPARAMETERS
#[repr(C)]
pub struct EAXEQUALIZERPROPERTIES {
    pub lLowGain: c_int,           // (mB)
    pub flLowCutOff: f32,        // (Hz)
    pub lMid1Gain: c_int,          // (mB)
    pub flMid1Center: f32,       // (Hz)
    pub flMid1Width: f32,        // (octaves)
    pub lMid2Gain: c_int,          // (mB)
    pub flMid2Center: f32,       // (Hz)
    pub flMid2Width: f32,        // (octaves)
    pub lHighGain: c_int,          // (mB)
    pub flHighCutOff: f32,       // (Hz)
}

// Property ranges and defaults:

pub const EAXEQUALIZER_MINLOWGAIN: c_int = -1800;
pub const EAXEQUALIZER_MAXLOWGAIN: c_int = 1800;
pub const EAXEQUALIZER_DEFAULTLOWGAIN: c_int = 0;

pub const EAXEQUALIZER_MINLOWCUTOFF: f32 = 50.0f32;
pub const EAXEQUALIZER_MAXLOWCUTOFF: f32 = 800.0f32;
pub const EAXEQUALIZER_DEFAULTLOWCUTOFF: f32 = 200.0f32;

pub const EAXEQUALIZER_MINMID1GAIN: c_int = -1800;
pub const EAXEQUALIZER_MAXMID1GAIN: c_int = 1800;
pub const EAXEQUALIZER_DEFAULTMID1GAIN: c_int = 0;

pub const EAXEQUALIZER_MINMID1CENTER: f32 = 200.0f32;
pub const EAXEQUALIZER_MAXMID1CENTER: f32 = 3000.0f32;
pub const EAXEQUALIZER_DEFAULTMID1CENTER: f32 = 500.0f32;

pub const EAXEQUALIZER_MINMID1WIDTH: f32 = 0.01f32;
pub const EAXEQUALIZER_MAXMID1WIDTH: f32 = 1.0f32;
pub const EAXEQUALIZER_DEFAULTMID1WIDTH: f32 = 1.0f32;

pub const EAXEQUALIZER_MINMID2GAIN: c_int = -1800;
pub const EAXEQUALIZER_MAXMID2GAIN: c_int = 1800;
pub const EAXEQUALIZER_DEFAULTMID2GAIN: c_int = 0;

pub const EAXEQUALIZER_MINMID2CENTER: f32 = 1000.0f32;
pub const EAXEQUALIZER_MAXMID2CENTER: f32 = 8000.0f32;
pub const EAXEQUALIZER_DEFAULTMID2CENTER: f32 = 3000.0f32;

pub const EAXEQUALIZER_MINMID2WIDTH: f32 = 0.01f32;
pub const EAXEQUALIZER_MAXMID2WIDTH: f32 = 1.0f32;
pub const EAXEQUALIZER_DEFAULTMID2WIDTH: f32 = 1.0f32;

pub const EAXEQUALIZER_MINHIGHGAIN: c_int = -1800;
pub const EAXEQUALIZER_MAXHIGHGAIN: c_int = 1800;
pub const EAXEQUALIZER_DEFAULTHIGHGAIN: c_int = 0;

pub const EAXEQUALIZER_MINHIGHCUTOFF: f32 = 4000.0f32;
pub const EAXEQUALIZER_MAXHIGHCUTOFF: f32 = 16000.0f32;
pub const EAXEQUALIZER_DEFAULTHIGHCUTOFF: f32 = 6000.0f32;

////////////////////////////////////////////////////////////////////////////
// Flanger Effect

// EAX FLANGER {A70007C0-7D2-11D3-9B1E-A024D13CE1}

pub const EAX_FLANGER_EFFECT: GUID = GUID {
    Data1: 0xa70007c0,
    Data2: 0x07d2,
    Data3: 0x11d3,
    Data4: [0x9b, 0x1e, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Flanger properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXFLANGER_PROPERTY {
    EAXFLANGER_NONE = 0,
    EAXFLANGER_ALLPARAMETERS,
    EAXFLANGER_WAVEFORM,
    EAXFLANGER_PHASE,
    EAXFLANGER_RATE,
    EAXFLANGER_DEPTH,
    EAXFLANGER_FEEDBACK,
    EAXFLANGER_DELAY,
}

// OR these flags with property id
pub const EAXFLANGER_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXFLANGER_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXFLANGER_COMMITDEFERREDSETTINGS: u32 = EAXFLANGER_NONE as u32 |
                                           EAXFLANGER_IMMEDIATE;

// used by EAXFLANGER_WAVEFORM
pub const EAX_FLANGER_SINUSOID: u32 = 0;
pub const EAX_FLANGER_TRIANGLE: u32 = 1;

// Use this structure for EAXFLANGER_ALLPARAMETERS
#[repr(C)]
pub struct EAXFLANGERPROPERTIES {
    pub ulWaveform: c_ulong,  // Waveform selector - see enum above
    pub lPhase: c_int,     // Phase (Degrees)
    pub flRate: f32,     // Rate (Hz)
    pub flDepth: f32,    // Depth (0 to 1)
    pub flFeedback: f32, // Feedback (0 to 1)
    pub flDelay: f32,    // Delay (seconds)
}

// Property ranges and defaults:

pub const EAXFLANGER_MINWAVEFORM: c_int = 0;
pub const EAXFLANGER_MAXWAVEFORM: c_int = 1;
pub const EAXFLANGER_DEFAULTWAVEFORM: c_int = 1;

pub const EAXFLANGER_MINPHASE: c_int = -180;
pub const EAXFLANGER_MAXPHASE: c_int = 180;
pub const EAXFLANGER_DEFAULTPHASE: c_int = 0;

pub const EAXFLANGER_MINRATE: f32 = 0.0f32;
pub const EAXFLANGER_MAXRATE: f32 = 10.0f32;
pub const EAXFLANGER_DEFAULTRATE: f32 = 0.27f32;

pub const EAXFLANGER_MINDEPTH: f32 = 0.0f32;
pub const EAXFLANGER_MAXDEPTH: f32 = 1.0f32;
pub const EAXFLANGER_DEFAULTDEPTH: f32 = 1.0f32;

pub const EAXFLANGER_MINFEEDBACK: f32 = -1.0f32;
pub const EAXFLANGER_MAXFEEDBACK: f32 = 1.0f32;
pub const EAXFLANGER_DEFAULTFEEDBACK: f32 = -0.5f32;

pub const EAXFLANGER_MINDELAY: f32 = 0.0f32;
pub const EAXFLANGER_MAXDELAY: f32 = 0.004f32;
pub const EAXFLANGER_DEFAULTDELAY: f32 = 0.002f32;

////////////////////////////////////////////////////////////////////////////
// Frequency Shifter Effect

// EAX FREQUENCY SHIFTER {DC3E1880-9212-11D3-939D-C0F02DD6F0}

pub const EAX_FREQUENCYSHIFTER_EFFECT: GUID = GUID {
    Data1: 0xdc3e1880,
    Data2: 0x9212,
    Data3: 0x11d3,
    Data4: [0x93, 0x9d, 0x00, 0xc0, 0xf0, 0x2d, 0xd6, 0xf0],
};

// Frequency Shifter properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXFREQUENCYSHIFTER_PROPERTY {
    EAXFREQUENCYSHIFTER_NONE = 0,
    EAXFREQUENCYSHIFTER_ALLPARAMETERS,
    EAXFREQUENCYSHIFTER_FREQUENCY,
    EAXFREQUENCYSHIFTER_LEFTDIRECTION,
    EAXFREQUENCYSHIFTER_RIGHTDIRECTION,
}

// OR these flags with property id
pub const EAXFREQUENCYSHIFTER_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXFREQUENCYSHIFTER_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXFREQUENCYSHIFTER_COMMITDEFERREDSETTINGS: u32 = EAXFREQUENCYSHIFTER_NONE as u32 |
                                                    EAXFREQUENCYSHIFTER_IMMEDIATE;

// used by EAXFREQUENCYSHIFTER_LEFTDIRECTION and EAXFREQUENCYSHIFTER_RIGHTDIRECTION
pub const EAX_FREQUENCYSHIFTER_DOWN: u32 = 0;
pub const EAX_FREQUENCYSHIFTER_UP: u32 = 1;
pub const EAX_FREQUENCYSHIFTER_OFF: u32 = 2;

// Use this structure for EAXFREQUENCYSHIFTER_ALLPARAMETERS
#[repr(C)]
pub struct EAXFREQUENCYSHIFTERPROPERTIES {
    pub flFrequency: f32,        // (Hz)
    pub ulLeftDirection: c_ulong,     // see enum above
    pub ulRightDirection: c_ulong,    // see enum above
}

// Property ranges and defaults:

pub const EAXFREQUENCYSHIFTER_MINFREQUENCY: f32 = 0.0f32;
pub const EAXFREQUENCYSHIFTER_MAXFREQUENCY: f32 = 24000.0f32;
pub const EAXFREQUENCYSHIFTER_DEFAULTFREQUENCY: f32 = 0.0f32;

pub const EAXFREQUENCYSHIFTER_MINLEFTDIRECTION: c_int = 0;
pub const EAXFREQUENCYSHIFTER_MAXLEFTDIRECTION: c_int = 2;
pub const EAXFREQUENCYSHIFTER_DEFAULTLEFTDIRECTION: c_int = 0;

pub const EAXFREQUENCYSHIFTER_MINRIGHTDIRECTION: c_int = 0;
pub const EAXFREQUENCYSHIFTER_MAXRIGHTDIRECTION: c_int = 2;
pub const EAXFREQUENCYSHIFTER_DEFAULTRIGHTDIRECTION: c_int = 0;

////////////////////////////////////////////////////////////////////////////
// Vocal Morpher Effect

// EAX VOCAL MORPHER {E41CF10C-3383-11D2-88DD-A024D13CE1}

pub const EAX_VOCALMORPHER_EFFECT: GUID = GUID {
    Data1: 0xe41cf10c,
    Data2: 0x3383,
    Data3: 0x11d2,
    Data4: [0x88, 0xdd, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Vocal Morpher properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXVOCALMORPHER_PROPERTY {
    EAXVOCALMORPHER_NONE = 0,
    EAXVOCALMORPHER_ALLPARAMETERS,
    EAXVOCALMORPHER_PHONEMEA,
    EAXVOCALMORPHER_PHONEMEACOARSETUNING,
    EAXVOCALMORPHER_PHONEMEB,
    EAXVOCALMORPHER_PHONEMEBCOARSETUNING,
    EAXVOCALMORPHER_WAVEFORM,
    EAXVOCALMORPHER_RATE,
}

// OR these flags with property id
pub const EAXVOCALMORPHER_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXVOCALMORPHER_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXVOCALMORPHER_COMMITDEFERREDSETTINGS: u32 = EAXVOCALMORPHER_NONE as u32 |
                                                EAXVOCALMORPHER_IMMEDIATE;

// used by EAXVOCALMORPHER_PHONEMEA and EAXVOCALMORPHER_PHONEMEB
pub const A: u32 = 0;
pub const E: u32 = 1;
pub const I: u32 = 2;
pub const O: u32 = 3;
pub const U: u32 = 4;
pub const AA: u32 = 5;
pub const AE: u32 = 6;
pub const AH: u32 = 7;
pub const AO: u32 = 8;
pub const EH: u32 = 9;
pub const ER: u32 = 10;
pub const IH: u32 = 11;
pub const IY: u32 = 12;
pub const UH: u32 = 13;
pub const UW: u32 = 14;
pub const B: u32 = 15;
pub const D: u32 = 16;
pub const F: u32 = 17;
pub const G: u32 = 18;
pub const J: u32 = 19;
pub const K: u32 = 20;
pub const L: u32 = 21;
pub const M: u32 = 22;
pub const N: u32 = 23;
pub const P: u32 = 24;
pub const R: u32 = 25;
pub const S: u32 = 26;
pub const T: u32 = 27;
pub const V: u32 = 28;
pub const Z: u32 = 29;

// used by EAXVOCALMORPHER_WAVEFORM
pub const EAX_VOCALMORPHER_SINUSOID: u32 = 0;
pub const EAX_VOCALMORPHER_TRIANGLE: u32 = 1;
pub const EAX_VOCALMORPHER_SAWTOOTH: u32 = 2;

// Use this structure for EAXVOCALMORPHER_ALLPARAMETERS
#[repr(C)]
pub struct EAXVOCALMORPHERPROPERTIES {
    pub ulPhonemeA: c_ulong,              // see enum above
    pub lPhonemeACoarseTuning: c_int,  // (semitones)
    pub ulPhonemeB: c_ulong,              // see enum above
    pub lPhonemeBCoarseTuning: c_int,  // (semitones)
    pub ulWaveform: c_ulong,              // Waveform selector - see enum above
    pub flRate: f32,                 // (Hz)
}

// Property ranges and defaults:

pub const EAXVOCALMORPHER_MINPHONEMEA: c_int = 0;
pub const EAXVOCALMORPHER_MAXPHONEMEA: c_int = 29;
pub const EAXVOCALMORPHER_DEFAULTPHONEMEA: c_int = 0;

pub const EAXVOCALMORPHER_MINPHONEMEACOARSETUNING: c_int = -24;
pub const EAXVOCALMORPHER_MAXPHONEMEACOARSETUNING: c_int = 24;
pub const EAXVOCALMORPHER_DEFAULTPHONEMEACOARSETUNING: c_int = 0;

pub const EAXVOCALMORPHER_MINPHONEMEB: c_int = 0;
pub const EAXVOCALMORPHER_MAXPHONEMEB: c_int = 29;
pub const EAXVOCALMORPHER_DEFAULTPHONEMEB: c_int = 10;

pub const EAXVOCALMORPHER_MINPHONEMEBCOARSETUNING: c_int = -24;
pub const EAXVOCALMORPHER_MAXPHONEMEBCOARSETUNING: c_int = 24;
pub const EAXVOCALMORPHER_DEFAULTPHONEMEBCOARSETUNING: c_int = 0;

pub const EAXVOCALMORPHER_MINWAVEFORM: c_int = 0;
pub const EAXVOCALMORPHER_MAXWAVEFORM: c_int = 2;
pub const EAXVOCALMORPHER_DEFAULTWAVEFORM: c_int = 0;

pub const EAXVOCALMORPHER_MINRATE: f32 = 0.0f32;
pub const EAXVOCALMORPHER_MAXRATE: f32 = 10.0f32;
pub const EAXVOCALMORPHER_DEFAULTRATE: f32 = 1.41f32;

////////////////////////////////////////////////////////////////////////////
// Pitch Shifter Effect

// EAX PITCH SHIFTER {E7905100-AFB2-11D2-88DD-A024D13CE1}

pub const EAX_PITCHSHIFTER_EFFECT: GUID = GUID {
    Data1: 0xe7905100,
    Data2: 0xafb2,
    Data3: 0x11d2,
    Data4: [0x88, 0xdd, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Pitch Shifter properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXPITCHSHIFTER_PROPERTY {
    EAXPITCHSHIFTER_NONE = 0,
    EAXPITCHSHIFTER_ALLPARAMETERS,
    EAXPITCHSHIFTER_COARSETUNE,
    EAXPITCHSHIFTER_FINETUNE,
}

// OR these flags with property id
pub const EAXPITCHSHIFTER_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXPITCHSHIFTER_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXPITCHSHIFTER_COMMITDEFERREDSETTINGS: u32 = EAXPITCHSHIFTER_NONE as u32 |
                                                EAXPITCHSHIFTER_IMMEDIATE;

// Use this structure for EAXPITCHSHIFTER_ALLPARAMETERS
#[repr(C)]
pub struct EAXPITCHSHIFTERPROPERTIES {
    pub lCoarseTune: c_int,    // Amount of pitch shift (semitones)
    pub lFineTune: c_int,      // Amount of pitch shift (cents)
}

// Property ranges and defaults:

pub const EAXPITCHSHIFTER_MINCOARSETUNE: c_int = -12;
pub const EAXPITCHSHIFTER_MAXCOARSETUNE: c_int = 12;
pub const EAXPITCHSHIFTER_DEFAULTCOARSETUNE: c_int = 12;

pub const EAXPITCHSHIFTER_MINFINETUNE: c_int = -50;
pub const EAXPITCHSHIFTER_MAXFINETUNE: c_int = 50;
pub const EAXPITCHSHIFTER_DEFAULTFINETUNE: c_int = 0;

////////////////////////////////////////////////////////////////////////////
// Ring Modulator Effect

// EAX RING MODULATOR {B89FE60-AFB5-11D2-88DD-A024D13CE1}

pub const EAX_RINGMODULATOR_EFFECT: GUID = GUID {
    Data1: 0x0b89fe60,
    Data2: 0xafb5,
    Data3: 0x11d2,
    Data4: [0x88, 0xdd, 0x00, 0xa0, 0x24, 0xd1, 0x3c, 0xe1],
};

// Ring Modulator properties
#[repr(C)]
#[derive(Clone, Copy)]
pub enum EAXRINGMODULATOR_PROPERTY {
    EAXRINGMODULATOR_NONE = 0,
    EAXRINGMODULATOR_ALLPARAMETERS,
    EAXRINGMODULATOR_FREQUENCY,
    EAXRINGMODULATOR_HIGHPASSCUTOFF,
    EAXRINGMODULATOR_WAVEFORM,
}

// OR these flags with property id
pub const EAXRINGMODULATOR_IMMEDIATE: u32 = 0x00000000; // changes take effect immediately
pub const EAXRINGMODULATOR_DEFERRED: u32 = 0x80000000; // changes take effect later
pub const EAXRINGMODULATOR_COMMITDEFERREDSETTINGS: u32 = EAXRINGMODULATOR_NONE as u32 |
                                                 EAXRINGMODULATOR_IMMEDIATE;

// used by EAXRINGMODULATOR_WAVEFORM
pub const EAX_RINGMODULATOR_SINUSOID: u32 = 0;
pub const EAX_RINGMODULATOR_SAWTOOTH: u32 = 1;
pub const EAX_RINGMODULATOR_SQUARE: u32 = 2;

// Use this structure for EAXRINGMODULATOR_ALLPARAMETERS
#[repr(C)]
pub struct EAXRINGMODULATORPROPERTIES {
    pub flFrequency: f32,        // Frequency of modulation (Hz)
    pub flHighPassCutOff: f32,   // Cut-off frequency of high-pass filter (Hz)
    pub ulWaveform: c_ulong,          // Waveform selector - see enum above
}

// Property ranges and defaults:

pub const EAXRINGMODULATOR_MINFREQUENCY: f32 = 0.0f32;
pub const EAXRINGMODULATOR_MAXFREQUENCY: f32 = 8000.0f32;
pub const EAXRINGMODULATOR_DEFAULTFREQUENCY: f32 = 440.0f32;

pub const EAXRINGMODULATOR_MINHIGHPASSCUTOFF: f32 = 0.0f32;
pub const EAXRINGMODULATOR_MAXHIGHPASSCUTOFF: f32 = 24000.0f32;
pub const EAXRINGMODULATOR_DEFAULTHIGHPASSCUTOFF: f32 = 800.0f32;

pub const EAXRINGMODULATOR_MINWAVEFORM: c_int = 0;
pub const EAXRINGMODULATOR_MAXWAVEFORM: c_int = 2;
pub const EAXRINGMODULATOR_DEFAULTWAVEFORM: c_int = 0;

////////////////////////////////////////////////////////////////////////////
