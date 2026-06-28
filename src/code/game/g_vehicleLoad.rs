//g_vehicleLoad.cpp
// leave this line at the top for all NPC_xxxx.cpp files...
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// ============================================================================
// External types and functions (from headers)
// ============================================================================

// Forward declarations from g_headers.h, q_shared.h, anims.h, g_vehicles.h
extern "C" {
    pub fn G_ParseLiteral(data: *const *const c_char, string: *const c_char) -> c_int;
    pub fn G_CreateG2AttachedWeaponModel(
        ent: *mut c_void,
        weaponModel: *const c_char,
        boltNum: c_int,
        weaponNum: c_int,
    );

    pub fn COM_ParseExt(holdBuf: *const *const c_char, allowLineBreaks: c_int) -> *const c_char;
    pub fn COM_ParseString(holdBuf: *const *const c_char, value: *const *const c_char) -> c_int;
    pub fn COM_BeginParseSession();

    pub fn atoi(str: *const c_char) -> c_int;
    pub fn atof(str: *const c_char) -> f32;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;

    pub fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;

    pub fn G_NewString(string: *const c_char) -> *mut c_char;
    pub fn G_Error(error: *const c_char, ...);
    pub fn GetIDForString(table: *const stringID_table_t, str: *const c_char) -> c_int;

    pub static mut gi: gameImport_t;
    pub static animTable: *const c_void;
}

// ============================================================================
// Type definitions and constants (from g_vehicles.h, q_shared.h)
// ============================================================================

pub const MAX_VEHICLES: usize = 256;
pub const VEH_MAX_PASSENGERS: c_int = 0;
pub const VEH_PARM_MAX: usize = 128;
pub const VH_NUM_VEHICLES: usize = 5;

// Vehicle types (from g_vehicles.h)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum vehicleType_t {
    VH_WALKER = 0,
    VH_FIGHTER = 1,
    VH_SPEEDER = 2,
    VH_ANIMAL = 3,
    VH_FLIER = 4,
}

// Field types for vehicle parsing
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum vehFieldType_t {
    VF_INT = 0,
    VF_FLOAT = 1,
    VF_LSTRING = 2,  // string on disk, pointer in memory, TAG_LEVEL
    VF_VECTOR = 3,
    VF_BOOL = 4,
    VF_VEHTYPE = 5,
    VF_ANIM = 6,
}

// Vehicle field descriptor
#[repr(C)]
pub struct vehField_t {
    pub name: *const c_char,
    pub ofs: c_int,
    pub type_: vehFieldType_t,
}

// String ID table entry
#[repr(C)]
pub struct stringID_table_t {
    pub string: *const c_char,
    pub id: c_int,
}

// Vehicle info structure (from g_vehicles.h - partial definition for offset purposes)
#[repr(C)]
pub struct vehicleInfo_t {
    // This is a placeholder - the full definition would come from g_vehicles.h
    // For now, we preserve the structure as it appears in the offset table
    pub _data: [u8; 1024], // Placeholder - actual size depends on g_vehicles.h
}

// Game import structure (partial)
#[repr(C)]
pub struct gameImport_t {
    pub Printf: Option<fn(*const c_char, ...)>,
    pub FS_GetFileList: Option<fn(*const c_char, *const c_char, *mut c_char, c_int) -> c_int>,
    pub FS_ReadFile: Option<fn(*const c_char, *const *mut c_void) -> c_int>,
    pub FS_FreeFile: Option<fn(*mut c_void)>,
    // ... more fields as needed
}

// ============================================================================
// Macros
// ============================================================================

// VFOFS macro - calculates offset of field within vehicleInfo_t structure
macro_rules! VFOFS {
    ($field:ident) => {
        core::mem::offset_of!(vehicleInfo_t, $field) as c_int
    };
}

// ENUM2STRING macro - creates stringID_table_t entries
macro_rules! ENUM2STRING {
    ($enum_val:ident) => {
        stringID_table_t {
            string: concat!(stringify!($enum_val), "\0").as_ptr() as *const c_char,
            id: $enum_val as c_int,
        }
    };
}

// Color codes for printing
const S_COLOR_RED: &[u8] = b"^1";
const S_COLOR_YELLOW: &[u8] = b"^3";

// ============================================================================
// Global variables
// ============================================================================

pub static mut g_vehicleInfo: [vehicleInfo_t; MAX_VEHICLES] = [vehicleInfo_t { _data: [0; 1024] }; MAX_VEHICLES];
pub static mut numVehicles: c_int = 1; // first one is null/default

// ============================================================================
// Vehicle field definitions table
// ============================================================================

pub static vehFields: [vehField_t; VEH_PARM_MAX] = [
    // name field at offset 0
    vehField_t {
        name: b"name\0".as_ptr() as *const c_char,
        ofs: VFOFS!(name) as c_int,
        type_: vehFieldType_t::VF_LSTRING,
    }, // unique name of the vehicle

    // general data
    vehField_t {
        name: b"type\0".as_ptr() as *const c_char,
        ofs: VFOFS!(type_) as c_int,
        type_: vehFieldType_t::VF_VEHTYPE,
    }, // what kind of vehicle

    vehField_t {
        name: b"numHands\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // if 2 hands, no weapons, if 1 hand, can use 1-handed weapons, if 0 hands, can use 2-handed weapons
    vehField_t {
        name: b"lookPitch\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // How far you can look up and down off the forward of the vehicle
    vehField_t {
        name: b"lookYaw\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // How far you can look left and right off the forward of the vehicle
    vehField_t {
        name: b"length\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how long it is - used for body length traces when turning/moving?
    vehField_t {
        name: b"width\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how wide it is - used for body length traces when turning/moving?
    vehField_t {
        name: b"height\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how tall it is - used for body length traces when turning/moving?
    vehField_t {
        name: b"centerOfGravity\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_VECTOR,
    }, // offset from origin: {forward, right, up} as a modifier on that dimension (-1.0f is all the way back, 1.0f is all the way forward)

    // speed stats
    vehField_t {
        name: b"speedMax\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // top speed
    vehField_t {
        name: b"turboSpeed\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // turbo speed
    vehField_t {
        name: b"speedMin\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // if < 0, can go in reverse
    vehField_t {
        name: b"speedIdle\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // what speed it drifts to when no accel/decel input is given
    vehField_t {
        name: b"accelIdle\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // if speedIdle > 0, how quickly it goes up to that speed
    vehField_t {
        name: b"acceleration\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // when pressing on accelerator
    vehField_t {
        name: b"decelIdle\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // when giving no input, how quickly it drops to speedIdle
    vehField_t {
        name: b"strafePerc\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // multiplier on current speed for strafing.  If 1.0f, you can strafe at the same speed as you're going forward, 0.5 is half, 0 is no strafing

    // handling stats
    vehField_t {
        name: b"bankingSpeed\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how quickly it pitches and rolls (not under player control)
    vehField_t {
        name: b"pitchLimit\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how far it can roll forward or backward
    vehField_t {
        name: b"rollLimit\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how far it can roll to either side
    vehField_t {
        name: b"braking\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // when pressing on decelerator
    vehField_t {
        name: b"turningSpeed\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how quickly you can turn
    vehField_t {
        name: b"turnWhenStopped\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_BOOL,
    }, // whether or not you can turn when not moving
    vehField_t {
        name: b"traction\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how much your command input affects velocity
    vehField_t {
        name: b"friction\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how much velocity is cut on its own
    vehField_t {
        name: b"maxSlope\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // the max slope that it can go up with control

    // durability stats
    vehField_t {
        name: b"mass\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // for momentum and impact force (player mass is 10)
    vehField_t {
        name: b"armor\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // total points of damage it can take
    vehField_t {
        name: b"toughness\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // modifies incoming damage, 1.0 is normal, 0.5 is half, etc.  Simulates being made of tougher materials/construction
    vehField_t {
        name: b"malfunctionArmorLevel\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // when armor drops to or below this point, start malfunctioning

    // visuals & sounds
    vehField_t {
        name: b"model\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // what model to use - if make it an NPC's primary model, don't need this?
    vehField_t {
        name: b"skin\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // what skin to use - if make it an NPC's primary model, don't need this?
    vehField_t {
        name: b"riderAnim\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_ANIM,
    }, // what animation the rider uses
    vehField_t {
        name: b"gunswivelBone\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // gun swivel bones
    vehField_t {
        name: b"lFinBone\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // left fin bone
    vehField_t {
        name: b"rFinBone\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // right fin bone
    vehField_t {
        name: b"lExhaustTag\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // left exhaust tag
    vehField_t {
        name: b"rExhaustTag\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // right exhaust tag

    vehField_t {
        name: b"soundOn\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // sound to play when get on it
    vehField_t {
        name: b"soundLoop\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // sound to loop while riding it
    vehField_t {
        name: b"soundOff\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // sound to play when get off
    vehField_t {
        name: b"exhaustFX\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // exhaust effect, played from "*exhaust" bolt(s)
    vehField_t {
        name: b"trailFX\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // trail effect, played from "*trail" bolt(s)
    vehField_t {
        name: b"impactFX\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // impact effect, for when it bumps into something
    vehField_t {
        name: b"explodeFX\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // explosion effect, for when it blows up (should have the sound built into explosion effect)
    vehField_t {
        name: b"wakeFX\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_LSTRING,
    }, // effect it makes when going across water

    // other misc stats
    vehField_t {
        name: b"gravity\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // normal is 800
    vehField_t {
        name: b"hoverHeight\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // if 0, it's a ground vehicle
    vehField_t {
        name: b"hoverStrength\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how hard it pushes off ground when less than hover height... causes "bounce", like shocks
    vehField_t {
        name: b"waterProof\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_BOOL,
    }, // can drive underwater if it has to
    vehField_t {
        name: b"bouyancy\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // when in water, how high it floats (1 is neutral bouyancy)
    vehField_t {
        name: b"fuelMax\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // how much fuel it can hold (capacity)
    vehField_t {
        name: b"fuelRate\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // how quickly is uses up fuel
    vehField_t {
        name: b"visibility\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // for sight alerts
    vehField_t {
        name: b"loudness\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // for sound alerts
    vehField_t {
        name: b"explosionRadius\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // range of explosion
    vehField_t {
        name: b"explosionDamage\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // damage of explosion

    // new stuff
    vehField_t {
        name: b"maxPassengers\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // The max number of passengers this vehicle may have (Default = 0).
    vehField_t {
        name: b"hideRider\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_BOOL,
    }, // rider (and passengers?) should not be drawn
    vehField_t {
        name: b"killRiderOnDeath\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_BOOL,
    }, // if rider is on vehicle when it dies, they should die
    vehField_t {
        name: b"flammable\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_BOOL,
    }, // whether or not the vehicle should catch on fire before it explodes
    vehField_t {
        name: b"explosionDelay\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    }, // how long the vehicle should be on fire/dying before it explodes
    // camera stuff
    vehField_t {
        name: b"cameraOverride\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_BOOL,
    }, // override the third person camera with the below values - normal is 0 (off)
    vehField_t {
        name: b"cameraRange\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how far back the camera should be - normal is 80
    vehField_t {
        name: b"cameraVertOffset\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how high over the vehicle origin the camera should be - normal is 16
    vehField_t {
        name: b"cameraHorzOffset\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // how far to left/right (negative/positive) of of the vehicle origin the camera should be - normal is 0
    vehField_t {
        name: b"cameraPitchOffset\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // a modifier on the camera's pitch (up/down angle) to the vehicle - normal is 0
    vehField_t {
        name: b"cameraFOV\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_FLOAT,
    }, // third person camera FOV, default is 80
    vehField_t {
        name: b"cameraAlpha\0".as_ptr() as *const c_char,
        ofs: 0,
        type_: vehFieldType_t::VF_BOOL,
    }, // fade out the vehicle if it's in the way of the crosshair

    // Padding to reach VEH_PARM_MAX
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
    vehField_t {
        name: core::ptr::null(),
        ofs: 0,
        type_: vehFieldType_t::VF_INT,
    },
];

pub static VehicleTable: [stringID_table_t; VH_NUM_VEHICLES + 1] = [
    stringID_table_t {
        string: b"VH_WALKER\0".as_ptr() as *const c_char,
        id: 0,
    }, // something you ride inside of, it walks like you, like an AT-ST
    stringID_table_t {
        string: b"VH_FIGHTER\0".as_ptr() as *const c_char,
        id: 1,
    }, // something you fly inside of, like an X-Wing or TIE fighter
    stringID_table_t {
        string: b"VH_SPEEDER\0".as_ptr() as *const c_char,
        id: 2,
    }, // something you ride on that hovers, like a speeder or swoop
    stringID_table_t {
        string: b"VH_ANIMAL\0".as_ptr() as *const c_char,
        id: 3,
    }, // animal you ride on top of that walks, like a tauntaun
    stringID_table_t {
        string: b"VH_FLIER\0".as_ptr() as *const c_char,
        id: 4,
    }, // animal you ride on top of that flies, like a giant mynoc?
    stringID_table_t {
        string: b"\0".as_ptr() as *const c_char,
        id: -1,
    },
];

// ============================================================================
// Functions
// ============================================================================

pub fn G_VehicleSetDefaults(vehicle: *mut vehicleInfo_t) {
    if vehicle.is_null() {
        return;
    }

    // vehicle->name = "default"; // unique name of the vehicle
    /*
    //general data
    vehicle->type = VH_SPEEDER;				//what kind of vehicle
    //FIXME: no saber or weapons if numHands = 2, should switch to speeder weapon, no attack anim on player
    vehicle->numHands = 2;					//if 2 hands, no weapons, if 1 hand, can use 1-handed weapons, if 0 hands, can use 2-handed weapons
    vehicle->lookPitch = 35;				//How far you can look up and down off the forward of the vehicle
    vehicle->lookYaw = 5;					//How far you can look left and right off the forward of the vehicle
    vehicle->length = 0;					//how long it is - used for body length traces when turning/moving?
    vehicle->width = 0;						//how wide it is - used for body length traces when turning/moving?
    vehicle->height = 0;					//how tall it is - used for body length traces when turning/moving?
    VectorClear( vehicle->centerOfGravity );//offset from origin: {forward, right, up} as a modifier on that dimension (-1.0f is all the way back, 1.0f is all the way forward)

    //speed stats - note: these are DESIRED speed, not actual current speed/velocity
    vehicle->speedMax = VEH_DEFAULT_SPEED_MAX;	//top speed
    vehicle->turboSpeed = 0;					//turboBoost
    vehicle->speedMin = 0;						//if < 0, can go in reverse
    vehicle->speedIdle = 0;						//what speed it drifts to when no accel/decel input is given
    vehicle->accelIdle = 0;						//if speedIdle > 0, how quickly it goes up to that speed
    vehicle->acceleration = VEH_DEFAULT_ACCEL;	//when pressing on accelerator (1/2 this when going in reverse)
    vehicle->decelIdle = VEH_DEFAULT_DECEL;		//when giving no input, how quickly it desired speed drops to speedIdle
    vehicle->strafePerc = VEH_DEFAULT_STRAFE_PERC;//multiplier on current speed for strafing.  If 1.0f, you can strafe at the same speed as you're going forward, 0.5 is half, 0 is no strafing

    //handling stats
    vehicle->bankingSpeed = VEH_DEFAULT_BANKING_SPEED;	//how quickly it pitches and rolls (not under player control)
    vehicle->rollLimit = VEH_DEFAULT_ROLL_LIMIT;		//how far it can roll to either side
    vehicle->pitchLimit = VEH_DEFAULT_PITCH_LIMIT;		//how far it can pitch forward or backward
    vehicle->braking = VEH_DEFAULT_BRAKING;				//when pressing on decelerator (backwards)
    vehicle->turningSpeed = VEH_DEFAULT_TURNING_SPEED;	//how quickly you can turn
    vehicle->turnWhenStopped = qfalse;					//whether or not you can turn when not moving
    vehicle->traction = VEH_DEFAULT_TRACTION;			//how much your command input affects velocity
    vehicle->friction = VEH_DEFAULT_FRICTION;			//how much velocity is cut on its own
    vehicle->maxSlope = VEH_DEFAULT_MAX_SLOPE;			//the max slope that it can go up with control

    //durability stats
    vehicle->mass = VEH_DEFAULT_MASS;			//for momentum and impact force (player mass is 10)
    vehicle->armor = VEH_DEFAULT_MAX_ARMOR;		//total points of damage it can take
    vehicle->toughness = VEH_DEFAULT_TOUGHNESS;	//modifies incoming damage, 1.0 is normal, 0.5 is half, etc.  Simulates being made of tougher materials/construction
    vehicle->malfunctionArmorLevel = 0;			//when armor drops to or below this point, start malfunctioning

    //visuals & sounds
    vehicle->model = "swoop";								//what model to use - if make it an NPC's primary model, don't need this?
    vehicle->modelIndex = 0;							//set internally, not until this vehicle is spawned into the level
    vehicle->skin = NULL;								//what skin to use - if make it an NPC's primary model, don't need this?
    vehicle->riderAnim = BOTH_GUNSIT1;					//what animation the rider uses
    vehicle->gunswivelBone = NULL;						//gun swivel bones
    vehicle->lFinBone = NULL;							//left fin bone
    vehicle->rFinBone = NULL;							//right fin bone
    vehicle->lExhaustTag = NULL;						//left exhaust tag
    vehicle->rExhaustTag = NULL;						//right exhaust tag

    vehicle->soundOn = NULL;							//sound to play when get on it
    vehicle->soundLoop = NULL;							//sound to loop while riding it
    vehicle->soundOff = NULL;							//sound to play when get off
    vehicle->exhaustFX = NULL;							//exhaust effect, played from "*exhaust" bolt(s)
    vehicle->trailFX = NULL;							//trail effect, played from "*trail" bolt(s)
    vehicle->impactFX = NULL;							//explosion effect, for when it blows up (should have the sound built into explosion effect)
    vehicle->explodeFX = NULL;							//explosion effect, for when it blows up (should have the sound built into explosion effect)
    vehicle->wakeFX = NULL;								//effect itmakes when going across water

    //other misc stats
    vehicle->gravity = VEH_DEFAULT_GRAVITY;				//normal is 800
    vehicle->hoverHeight = 0;	//if 0, it's a ground vehicle
    vehicle->hoverStrength = 0;//how hard it pushes off ground when less than hover height... causes "bounce", like shocks
    vehicle->waterProof = qtrue;						//can drive underwater if it has to
    vehicle->bouyancy = 1.0f;							//when in water, how high it floats (1 is neutral bouyancy)
    vehicle->fuelMax = 1000;							//how much fuel it can hold (capacity)
    vehicle->fuelRate = 1;								//how quickly is uses up fuel
    vehicle->visibility = VEH_DEFAULT_VISIBILITY;		//radius for sight alerts
    vehicle->loudness = VEH_DEFAULT_LOUDNESS;			//radius for sound alerts
    vehicle->explosionRadius = VEH_DEFAULT_EXP_RAD;
    vehicle->explosionDamage = VEH_DEFAULT_EXP_DMG;

    //new stuff
    vehicle->maxPassengers = 0;
    vehicle->hideRider = qfalse;						// rider (and passengers?) should not be drawn
    vehicle->killRiderOnDeath = qfalse;					//if rider is on vehicle when it dies, they should die
    vehicle->flammable = qfalse;						//whether or not the vehicle should catch on fire before it explodes
    vehicle->explosionDelay = 0;						//how long the vehicle should be on fire/dying before it explodes
    //camera stuff
    vehicle->cameraOverride = qfalse;					//whether or not to use all of the following 3rd person camera override values
    vehicle->cameraRange = 0.0f;						//how far back the camera should be - normal is 80
    vehicle->cameraVertOffset = 0.0f;					//how high over the vehicle origin the camera should be - normal is 16
    vehicle->cameraHorzOffset = 0.0f;					//how far to left/right (negative/positive) of of the vehicle origin the camera should be - normal is 0
    vehicle->cameraPitchOffset = 0.0f;					//a modifier on the camera's pitch (up/down angle) to the vehicle - normal is 0
    vehicle->cameraFOV = 0.0f;							//third person camera FOV, default is 80
    vehicle->cameraAlpha = qfalse;						//fade out the vehicle if it's in the way of the crosshair
    */
}

pub fn G_VehicleClampData(vehicle: *mut vehicleInfo_t) {
    // sanity check and clamp the vehicle's data
    if vehicle.is_null() {
        return;
    }

    let mut i: c_int;

    for i in 0..3 {
        let center_of_gravity_ptr = unsafe { (vehicle as *mut u8).add(0) as *mut f32 };
        let cog_val = unsafe { *center_of_gravity_ptr.add(i as usize) };

        if cog_val > 1.0f32 {
            unsafe { *center_of_gravity_ptr.add(i as usize) = 1.0f32 };
        } else if cog_val < -1.0f32 {
            unsafe { *center_of_gravity_ptr.add(i as usize) = -1.0f32 };
        }
    }

    // Validate passenger max.
    // vehicle->maxPassengers handling would go here, but requires knowing exact field offset
}

static mut G_ParseVehicleParms_line_298_assert: bool = false;

fn G_ParseVehicleParms(vehicle: *mut vehicleInfo_t, holdBuf: *const *const c_char) {
    if vehicle.is_null() || holdBuf.is_null() {
        return;
    }

    let mut token: *const c_char;
    let mut value: *const c_char;
    let mut i: c_int;
    let mut vec: [f32; 3] = [0.0; 3];
    let b = vehicle as *mut u8;
    let mut _iFieldsRead: c_int = 0;
    let mut vehType: vehicleType_t;

    unsafe {
        while !(*holdBuf).is_null() {
            token = COM_ParseExt(holdBuf, 1);
            if (*token) == 0 as c_char {
                if let Some(printf) = gi.Printf {
                    printf(
                        b"ERROR: unexpected EOF while parsing vehicles!\n\0".as_ptr() as *const c_char,
                    );
                }
                return;
            }

            if (*token as c_char) as u8 == b'}' as u8 {
                // End of data for this vehicle
                break;
            }

            // Loop through possible parameters
            i = 0;
            while i < VEH_PARM_MAX as c_int {
                if !vehFields[i as usize].name.is_null()
                    && Q_stricmp(vehFields[i as usize].name, token) == 0
                {
                    // found it
                    if COM_ParseString(holdBuf, &mut value) != 0 {
                        i += 1;
                        continue;
                    }

                    match vehFields[i as usize].type_ {
                        vehFieldType_t::VF_INT => {
                            *(b.add(vehFields[i as usize].ofs as usize) as *mut c_int) =
                                atoi(value);
                        }
                        vehFieldType_t::VF_FLOAT => {
                            *(b.add(vehFields[i as usize].ofs as usize) as *mut f32) =
                                atof(value);
                        }
                        vehFieldType_t::VF_LSTRING => {
                            // string on disk, pointer in memory, TAG_LEVEL
                            *(b.add(vehFields[i as usize].ofs as usize) as *mut *mut c_char) =
                                G_NewString(value);
                        }
                        vehFieldType_t::VF_VECTOR => {
                            _iFieldsRead = sscanf(
                                value,
                                b"%f %f %f\0".as_ptr() as *const c_char,
                                &mut vec[0] as *mut f32,
                                &mut vec[1] as *mut f32,
                                &mut vec[2] as *mut f32,
                            );
                            assert!(_iFieldsRead == 3);
                            if _iFieldsRead != 3 {
                                if let Some(printf) = gi.Printf {
                                    printf(
                                        b"G_ParseVehicleParms: VEC3 sscanf() failed to read 3 floats ('angle' key bug?)\n\0"
                                            .as_ptr() as *const c_char,
                                    );
                                }
                            }
                            let offset = b.add(vehFields[i as usize].ofs as usize) as *mut f32;
                            *offset = vec[0];
                            *offset.add(1) = vec[1];
                            *offset.add(2) = vec[2];
                        }
                        vehFieldType_t::VF_BOOL => {
                            *(b.add(vehFields[i as usize].ofs as usize) as *mut c_int) =
                                if atof(value) != 0.0f32 { 1 } else { 0 };
                        }
                        vehFieldType_t::VF_VEHTYPE => {
                            vehType = GetIDForString(
                                addr_of!(VehicleTable) as *const stringID_table_t,
                                value,
                            ) as vehicleType_t;
                            *(b.add(vehFields[i as usize].ofs as usize) as *mut vehicleType_t) =
                                vehType;
                        }
                        vehFieldType_t::VF_ANIM => {
                            let anim = GetIDForString(animTable as *const stringID_table_t, value);
                            *(b.add(vehFields[i as usize].ofs as usize) as *mut c_int) = anim;
                        }
                    }
                    break;
                }
                i += 1;
            }
        }
    }
}

// Stub function for Q_stricmp
fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if s1.is_null() || s2.is_null() {
        return if s1 == s2 { 0 } else { 1 };
    }
    unsafe {
        let mut i = 0;
        loop {
            let c1 = *s1.add(i) as u8;
            let c2 = *s2.add(i) as u8;
            if c1 == 0 && c2 == 0 {
                return 0;
            }
            if c1.to_ascii_lowercase() != c2.to_ascii_lowercase() {
                return if c1 < c2 { -1 } else { 1 };
            }
            i += 1;
        }
    }
}

fn G_VehicleStoreParms(p: *const c_char) {
    // load up all into a table: g_vehicleInfo
    if p.is_null() {
        return;
    }

    let mut token: *const c_char;
    let mut vehicle: *mut vehicleInfo_t;

    unsafe {
        ////////////////// HERE //////////////////////
        // The first vehicle just contains all the base level (not 'overridden') function calls.
        // G_SetSharedVehicleFunctions( &g_vehicleInfo[0] );
        numVehicles = 1;

        // try to parse data out
        COM_BeginParseSession();

        let mut p_mut = p;
        // look for an open brace
        while !p_mut.is_null() {
            token = COM_ParseExt(&mut p_mut, 1);
            if *token == 0 as c_char {
                // barf
                return;
            }

            if *token as u8 == b'{' as u8 {
                // found one, parse out the goodies
                if numVehicles >= MAX_VEHICLES as c_int {
                    // sorry, no more vehicle slots!
                    if let Some(printf) = gi.Printf {
                        printf(
                            b"Too many vehicles in *.veh (limit %d)\n\0".as_ptr() as *const c_char,
                            MAX_VEHICLES as c_int,
                        );
                    }
                    break;
                }
                // token = token;
                vehicle = &mut g_vehicleInfo[numVehicles as usize];
                numVehicles += 1;
                G_VehicleSetDefaults(vehicle);
                G_ParseVehicleParms(vehicle, &mut p_mut);
                // sanity check and clamp the vehicle's data
                G_VehicleClampData(vehicle);
                ////////////////// HERE //////////////////////
                // Setup the shared function pointers.
                // G_SetSharedVehicleFunctions( vehicle );
                // match vehicle->type
                // {
                //     case VH_SPEEDER:
                //         G_SetSpeederVehicleFunctions( vehicle );
                //         break;
                //     case VH_ANIMAL:
                //         G_SetAnimalVehicleFunctions( vehicle );
                //         break;
                //     case VH_FIGHTER:
                //         G_SetFighterVehicleFunctions( vehicle );
                //         break;
                //     case VH_WALKER:
                //         G_SetAnimalVehicleFunctions( vehicle );
                //         break;
                // }
            }
        }
    }
}

pub fn G_VehicleLoadParms() {
    // HMM... only do this if there's a vehicle on the level?
    let mut len: c_int;
    let mut totallen: c_int;
    let mut vehExtFNLen: usize;
    let mut fileCnt: c_int;
    let mut i: c_int;
    let mut buffer: *mut c_char;
    let mut holdChar: *mut c_char;
    let mut marker: *mut c_char;

    let vehExtensionListBuf: [c_char; 2048] = [0; 2048]; // The list of file names read in

    const MAX_VEHICLE_DATA_SIZE: usize = 0x20000;
    let VehicleParms: [c_char; MAX_VEHICLE_DATA_SIZE] = [0; MAX_VEHICLE_DATA_SIZE];

    // gi.Printf( "Parsing *.veh vehicle definitions\n" );

    // set where to store the first one
    unsafe {
        totallen = 0;
        marker = addr_of_mut!(VehicleParms) as *mut c_char;

        // now load in the .veh vehicle definitions
        fileCnt = if let Some(fs_get_file_list) = gi.FS_GetFileList {
            fs_get_file_list(
                b"ext_data/vehicles\0".as_ptr() as *const c_char,
                b".veh\0".as_ptr() as *const c_char,
                addr_of_mut!(vehExtensionListBuf) as *mut c_char,
                2048,
            )
        } else {
            0
        };

        holdChar = addr_of_mut!(vehExtensionListBuf) as *mut c_char;
        i = 0;
        while i < fileCnt {
            vehExtFNLen = strlen(holdChar);

            // gi.Printf( "Parsing %s\n", holdChar );

            len = if let Some(fs_read_file) = gi.FS_ReadFile {
                // Need to construct the path: va( "ext_data/vehicles/%s", holdChar)
                // For now, use a simplified version
                fs_read_file(
                    b"ext_data/vehicles/\0".as_ptr() as *const c_char,
                    &mut buffer as *const *mut c_void as *const *mut c_void,
                )
            } else {
                -1
            };

            if len == -1 {
                if let Some(printf) = gi.Printf {
                    printf(
                        b"G_VehicleLoadParms: error reading file %s\n\0".as_ptr() as *const c_char,
                        holdChar,
                    );
                }
            } else {
                if totallen != 0 && *(marker as *const c_char).offset(-1) as u8 == b'}' as u8 {
                    // don't let it end on a } because that should be a stand-alone token
                    strcat(marker, b" \0".as_ptr() as *const c_char);
                    totallen += 1;
                    marker = marker.add(1);
                }
                if totallen + len >= MAX_VEHICLE_DATA_SIZE as c_int {
                    G_Error(
                        b"G_VehicleLoadParms: ran out of space before reading %s\n(you must make the .npc files smaller)\0"
                            .as_ptr() as *const c_char,
                        holdChar,
                    );
                }
                strcat(marker, buffer);
                if let Some(fs_free_file) = gi.FS_FreeFile {
                    fs_free_file(buffer as *mut c_void);
                }

                totallen += len;
                marker = marker.add(len as usize);
            }
            i += 1;
            holdChar = holdChar.add(vehExtFNLen + 1);
        }
        G_VehicleStoreParms(addr_of!(VehicleParms) as *const c_char);
    }
}
