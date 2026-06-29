//g_itemLoad.rs
//reads in ext_data\items.dat to bg_itemlist[]

// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// #include "g_local.h"
// #include "g_items.h"

const PICKUPSOUND: &[u8] = b"sound/weapons/w_pkup.wav\0";
#[cfg(feature = "_IMMERSION")]
const PICKUPFORCE: &[u8] = b"fffx/weapons/w_pkup\0";

// qboolean COM_ParseInt( char **data, int *i );
// qboolean COM_ParseString( char **data, char **s );
// qboolean COM_ParseFloat( char **data, float *f );

// Stub types from other modules - full definitions in respective header ports
#[repr(C)]
pub struct gitem_t {
    // Stub: actual layout defined in g_items_h.rs
}

#[repr(C)]
pub struct itemType_t {
    // Stub: actual definition in g_items_h.rs
}

extern "C" {
    pub static mut bg_itemlist: [gitem_t; 0];

    // External parsing and utility functions from common module
    fn COM_ParseString(data: *mut *const c_char, s: *mut *const c_char) -> c_int;
    fn COM_ParseInt(data: *mut *const c_char, i: *mut c_int) -> c_int;
    fn COM_ParseExt(data: *mut *const c_char, allowLineBreak: c_int) -> *const c_char;
    fn COM_BeginParseSession();

    // String and memory functions
    fn strlen(s: *const c_char) -> usize;
    fn G_NewString(string: *const c_char) -> *const c_char;

    // Error/output functions
    fn G_Error(fmt: *const c_char, ...);

    // File system functions
    pub struct fileHandle_t {
        // Stub
    }

    // Engine interface - gi
    pub static gi: GameImport_t;
}

#[repr(C)]
pub struct GameImport_t {
    // Stub - actual definition in other module
}

impl GameImport_t {
    unsafe fn Printf(&self, fmt: *const c_char, ...) {
        // Stub - actual call through engine interface
    }

    unsafe fn FS_ReadFile(&self, qpath: *const c_char, buffer: *mut *mut c_void) -> c_int {
        // Stub
        0
    }

    unsafe fn FS_FreeFile(&self, buffer: *mut c_void) {
        // Stub
    }
}

// External constants/enums - stubs for item types and weapon types
const ITM_NONE: c_int = 0;
const ITM_STUN_BATON_PICKUP: c_int = 1;
const ITM_SABER_PICKUP: c_int = 2;
const ITM_BRYAR_PISTOL_PICKUP: c_int = 3;
const ITM_BLASTER_PICKUP: c_int = 4;
const ITM_DISRUPTOR_PICKUP: c_int = 5;
const ITM_BOWCASTER_PICKUP: c_int = 6;
const ITM_REPEATER_PICKUP: c_int = 7;
const ITM_DEMP2_PICKUP: c_int = 8;
const ITM_FLECHETTE_PICKUP: c_int = 9;
const ITM_ROCKET_LAUNCHER_PICKUP: c_int = 10;
const ITM_THERMAL_DET_PICKUP: c_int = 11;
const ITM_TRIP_MINE_PICKUP: c_int = 12;
const ITM_DET_PACK_PICKUP: c_int = 13;
const ITM_BOT_LASER_PICKUP: c_int = 14;
const ITM_EMPLACED_GUN_PICKUP: c_int = 15;
const ITM_TURRET_PICKUP: c_int = 16;
const ITM_MELEE: c_int = 17;
const ITM_ATST_MAIN_PICKUP: c_int = 18;
const ITM_ATST_SIDE_PICKUP: c_int = 19;
const ITM_TIE_FIGHTER_PICKUP: c_int = 20;
const ITM_RAPID_FIRE_CONC_PICKUP: c_int = 21;
const ITM_JAWA_PICKUP: c_int = 22;
const ITM_TUSKEN_RIFLE_PICKUP: c_int = 23;
const ITM_TUSKEN_STAFF_PICKUP: c_int = 24;
const ITM_SCEPTER_PICKUP: c_int = 25;
const ITM_NOGHRI_STICK_PICKUP: c_int = 26;
const ITM_AMMO_FORCE_PICKUP: c_int = 27;
const ITM_AMMO_BLASTER_PICKUP: c_int = 28;
const ITM_AMMO_POWERCELL_PICKUP: c_int = 29;
const ITM_AMMO_METAL_BOLTS_PICKUP: c_int = 30;
const ITM_AMMO_ROCKETS_PICKUP: c_int = 31;
const ITM_AMMO_EMPLACED_PICKUP: c_int = 32;
const ITM_AMMO_THERMAL_PICKUP: c_int = 33;
const ITM_AMMO_TRIPMINE_PICKUP: c_int = 34;
const ITM_AMMO_DETPACK_PICKUP: c_int = 35;
const ITM_FORCE_HEAL_PICKUP: c_int = 36;
const ITM_FORCE_LEVITATION_PICKUP: c_int = 37;
const ITM_FORCE_SPEED_PICKUP: c_int = 38;
const ITM_FORCE_PUSH_PICKUP: c_int = 39;
const ITM_FORCE_PULL_PICKUP: c_int = 40;
const ITM_FORCE_TELEPATHY_PICKUP: c_int = 41;
const ITM_FORCE_GRIP_PICKUP: c_int = 42;
const ITM_FORCE_LIGHTNING_PICKUP: c_int = 43;
const ITM_FORCE_SABERTHROW_PICKUP: c_int = 44;
const ITM_BATTERY_PICKUP: c_int = 45;
const ITM_SEEKER_PICKUP: c_int = 46;
const ITM_SHIELD_PICKUP: c_int = 47;
const ITM_BACTA_PICKUP: c_int = 48;
const ITM_DATAPAD_PICKUP: c_int = 49;
const ITM_BINOCULARS_PICKUP: c_int = 50;
const ITM_SENTRY_GUN_PICKUP: c_int = 51;
const ITM_LA_GOGGLES_PICKUP: c_int = 52;
const ITM_BLASTER_PISTOL_PICKUP: c_int = 53;
const ITM_CONCUSSION_RIFLE_PICKUP: c_int = 54;
const ITM_MEDPAK_PICKUP: c_int = 55;
const ITM_SHIELD_SM_PICKUP: c_int = 56;
const ITM_SHIELD_LRG_PICKUP: c_int = 57;
const ITM_GOODIE_KEY_PICKUP: c_int = 58;
const ITM_SECURITY_KEY_PICKUP: c_int = 59;

const WP_NONE: c_int = 0;
const WP_STUN_BATON: c_int = 1;
const WP_SABER: c_int = 2;
const WP_BLASTER_PISTOL: c_int = 3;
const WP_BRYAR_PISTOL: c_int = 4;
const WP_BLASTER: c_int = 5;
const WP_DISRUPTOR: c_int = 6;
const WP_BOWCASTER: c_int = 7;
const WP_REPEATER: c_int = 8;
const WP_DEMP2: c_int = 9;
const WP_FLECHETTE: c_int = 10;
const WP_ROCKET_LAUNCHER: c_int = 11;
const WP_CONCUSSION: c_int = 12;
const WP_THERMAL: c_int = 13;
const WP_TRIP_MINE: c_int = 14;
const WP_DET_PACK: c_int = 15;
const WP_BOT_LASER: c_int = 16;
const WP_EMPLACED_GUN: c_int = 17;
const WP_MELEE: c_int = 18;
const WP_TURRET: c_int = 19;
const WP_ATST_MAIN: c_int = 20;
const WP_ATST_SIDE: c_int = 21;
const WP_TIE_FIGHTER: c_int = 22;
const WP_RAPID_FIRE_CONC: c_int = 23;
const WP_JAWA: c_int = 24;
const WP_TUSKEN_RIFLE: c_int = 25;
const WP_TUSKEN_STAFF: c_int = 26;
const WP_SCEPTER: c_int = 27;
const WP_NOGHRI_STICK: c_int = 28;

const AMMO_FORCE: c_int = 29;
const AMMO_BLASTER: c_int = 30;
const AMMO_POWERCELL: c_int = 31;
const AMMO_METAL_BOLTS: c_int = 32;
const AMMO_ROCKETS: c_int = 33;
const AMMO_EMPLACED: c_int = 34;
const AMMO_THERMAL: c_int = 35;
const AMMO_TRIPMINE: c_int = 36;
const AMMO_DETPACK: c_int = 37;

const FP_HEAL: c_int = 38;
const FP_LEVITATION: c_int = 39;
const FP_SPEED: c_int = 40;
const FP_PUSH: c_int = 41;
const FP_PULL: c_int = 42;
const FP_TELEPATHY: c_int = 43;
const FP_GRIP: c_int = 44;
const FP_LIGHTNING: c_int = 45;
const FP_SABERTHROW: c_int = 46;

const INV_SEEKER: c_int = 47;
const INV_BACTA_CANISTER: c_int = 48;
const INV_ELECTROBINOCULARS: c_int = 49;
const INV_SENTRY: c_int = 50;
const INV_LIGHTAMP_GOGGLES: c_int = 51;
const INV_GOODIE_KEY: c_int = 52;
const INV_SECURITY_KEY: c_int = 53;

const IT_BAD: c_int = 0;
const IT_WEAPON: c_int = 1;
const IT_AMMO: c_int = 2;
const IT_ARMOR: c_int = 3;
const IT_HEALTH: c_int = 4;
const IT_HOLDABLE: c_int = 5;
const IT_BATTERY: c_int = 6;
const IT_HOLOCRON: c_int = 7;

const FINAL_BUILD: bool = false; // Stub - actual value from build config

#[repr(C)]
pub struct ItemParms {
    itemNum: c_int,
}

static mut itemParms: ItemParms = ItemParms { itemNum: 0 };

// Function pointer type for item parameter handlers
type ItemParseFunc = fn(*mut *const c_char);

#[repr(C)]
pub struct itemParms_t {
    parmName: *const c_char,
    func: ItemParseFunc,
}

// Forward declarations
fn IT_ClassName(holdBuf: *mut *const c_char);
fn IT_Count(holdBuf: *mut *const c_char);
fn IT_Icon(holdBuf: *mut *const c_char);
fn IT_Min(holdBuf: *mut *const c_char);
fn IT_Max(holdBuf: *mut *const c_char);
fn IT_Name(holdBuf: *mut *const c_char);
fn IT_PickupSound(holdBuf: *mut *const c_char);
fn IT_Tag(holdBuf: *mut *const c_char);
fn IT_Type(holdBuf: *mut *const c_char);
fn IT_WorldModel(holdBuf: *mut *const c_char);
#[cfg(feature = "_IMMERSION")]
fn IT_PickupForce(holdBuf: *mut *const c_char);

#[cfg(feature = "_IMMERSION")]
const IT_PARM_MAX: usize = 11;
#[cfg(not(feature = "_IMMERSION"))]
const IT_PARM_MAX: usize = 10;

static ITEM_PARMS: [itemParms_t; 10] = [
    itemParms_t {
        parmName: b"itemname\0".as_ptr() as *const c_char,
        func: IT_Name,
    },
    itemParms_t {
        parmName: b"classname\0".as_ptr() as *const c_char,
        func: IT_ClassName,
    },
    itemParms_t {
        parmName: b"count\0".as_ptr() as *const c_char,
        func: IT_Count,
    },
    itemParms_t {
        parmName: b"icon\0".as_ptr() as *const c_char,
        func: IT_Icon,
    },
    itemParms_t {
        parmName: b"min\0".as_ptr() as *const c_char,
        func: IT_Min,
    },
    itemParms_t {
        parmName: b"max\0".as_ptr() as *const c_char,
        func: IT_Max,
    },
    itemParms_t {
        parmName: b"pickupsound\0".as_ptr() as *const c_char,
        func: IT_PickupSound,
    },
    itemParms_t {
        parmName: b"tag\0".as_ptr() as *const c_char,
        func: IT_Tag,
    },
    itemParms_t {
        parmName: b"type\0".as_ptr() as *const c_char,
        func: IT_Type,
    },
    itemParms_t {
        parmName: b"worldmodel\0".as_ptr() as *const c_char,
        func: IT_WorldModel,
    },
    // #[cfg(feature = "_IMMERSION")]
    // itemParms_t {
    //     parmName: b"pickupforce\0".as_ptr() as *const c_char,
    //     func: IT_PickupForce,
    // },
];

fn IT_SetDefaults() {
    unsafe {
        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].mins[0] = -16;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].mins[1] = -16;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].mins[2] = -2;

        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].maxs[0] = 16;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].maxs[1] = 16;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].maxs[2] = 16;

        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].pickup_sound = PICKUPSOUND.as_ptr() as *const c_char; //give it a default sound
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].precaches = core::ptr::null_mut();
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].sounds = core::ptr::null_mut();
        #[cfg(feature = "_IMMERSION")]
        {
            (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].pickup_force = PICKUPFORCE.as_ptr() as *const c_char;
            (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].forces = core::ptr::null_mut();
        }
    }
}

fn IT_Name(holdBuf: *mut *const c_char) {
    unsafe {
        let mut itemNum: c_int;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        if Q_stricmp(tokenStr, b"ITM_NONE\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_NONE;
        } else if Q_stricmp(tokenStr, b"ITM_STUN_BATON_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_STUN_BATON_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SABER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SABER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BRYAR_PISTOL_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BRYAR_PISTOL_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BLASTER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BLASTER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_DISRUPTOR_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_DISRUPTOR_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BOWCASTER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BOWCASTER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_REPEATER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_REPEATER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_DEMP2_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_DEMP2_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FLECHETTE_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FLECHETTE_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_ROCKET_LAUNCHER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_ROCKET_LAUNCHER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_THERMAL_DET_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_THERMAL_DET_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_TRIP_MINE_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_TRIP_MINE_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_DET_PACK_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_DET_PACK_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BOT_LASER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BOT_LASER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_EMPLACED_GUN_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_EMPLACED_GUN_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_TURRET_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_TURRET_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_MELEE\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_MELEE;
        } else if Q_stricmp(tokenStr, b"ITM_ATST_MAIN_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_ATST_MAIN_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_ATST_SIDE_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_ATST_SIDE_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_TIE_FIGHTER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_TIE_FIGHTER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_RAPID_FIRE_CONC_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_RAPID_FIRE_CONC_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_JAWA_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_JAWA_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_TUSKEN_RIFLE_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_TUSKEN_RIFLE_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_TUSKEN_STAFF_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_TUSKEN_STAFF_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SCEPTER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SCEPTER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_NOGHRI_STICK_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_NOGHRI_STICK_PICKUP;
        }
        //ammo
        else if Q_stricmp(tokenStr, b"ITM_AMMO_FORCE_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_FORCE_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_BLASTER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_BLASTER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_POWERCELL_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_POWERCELL_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_METAL_BOLTS_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_METAL_BOLTS_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_ROCKETS_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_ROCKETS_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_EMPLACED_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_EMPLACED_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_THERMAL_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_THERMAL_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_TRIPMINE_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_TRIPMINE_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_AMMO_DETPACK_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_AMMO_DETPACK_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_HEAL_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_HEAL_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_LEVITATION_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_LEVITATION_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_SPEED_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_SPEED_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_PUSH_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_PUSH_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_PULL_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_PULL_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_TELEPATHY_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_TELEPATHY_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_GRIP_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_GRIP_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_LIGHTNING_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_LIGHTNING_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_FORCE_SABERTHROW_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_FORCE_SABERTHROW_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BATTERY_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BATTERY_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SEEKER_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SEEKER_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SHIELD_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SHIELD_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BACTA_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BACTA_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_DATAPAD_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_DATAPAD_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BINOCULARS_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BINOCULARS_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SENTRY_GUN_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SENTRY_GUN_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_LA_GOGGLES_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_LA_GOGGLES_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_BLASTER_PISTOL_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_BLASTER_PISTOL_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_CONCUSSION_RIFLE_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_CONCUSSION_RIFLE_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_MEDPAK_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_MEDPAK_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SHIELD_SM_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SHIELD_SM_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SHIELD_LRG_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SHIELD_LRG_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_GOODIE_KEY_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_GOODIE_KEY_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SECURITY_KEY_PICKUP\0".as_ptr() as *const c_char) == 0 {
            itemNum = ITM_SECURITY_KEY_PICKUP;
        } else {
            itemNum = 0;
            gi.Printf(b"WARNING: bad itemname in external item data '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        (*core::ptr::addr_of_mut!(itemParms)).itemNum = itemNum;
        //	++bg_numItems;

        IT_SetDefaults();
    }
}

fn IT_ClassName(holdBuf: *mut *const c_char) {
    unsafe {
        let mut len: usize;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        len = strlen(tokenStr);
        len += 1;
        if len > 32 {
            len = 32;
            gi.Printf(b"WARNING: weaponclass too long in external ITEMS.DAT '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].classname = G_NewString(tokenStr);

        //	Q_strncpyz(bg_itemlist[itemParms.itemNum].classname,tokenStr,len);
    }
}

fn IT_WorldModel(holdBuf: *mut *const c_char) {
    unsafe {
        let mut len: usize;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        len = strlen(tokenStr);
        len += 1;
        if len > 64 {
            len = 64;
            gi.Printf(b"WARNING: world model too long in external ITEMS.DAT '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].world_model = G_NewString(tokenStr);

        //	Q_strncpyz(bg_itemlist[itemParms.itemNum].world_model[0],tokenStr,len);
    }
}

fn IT_Tag(holdBuf: *mut *const c_char) {
    unsafe {
        let mut tag: c_int;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        if Q_stricmp(tokenStr, b"WP_NONE\0".as_ptr() as *const c_char) == 0 {
            tag = WP_NONE;
        } else if Q_stricmp(tokenStr, b"WP_STUN_BATON\0".as_ptr() as *const c_char) == 0 {
            tag = WP_STUN_BATON;
        } else if Q_stricmp(tokenStr, b"WP_SABER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_SABER;
        } else if Q_stricmp(tokenStr, b"WP_BLASTER_PISTOL\0".as_ptr() as *const c_char) == 0 {
            tag = WP_BLASTER_PISTOL;
        } else if Q_stricmp(tokenStr, b"WP_BRYAR_PISTOL\0".as_ptr() as *const c_char) == 0 {
            tag = WP_BRYAR_PISTOL;
        } else if Q_stricmp(tokenStr, b"WP_BLASTER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_BLASTER;
        } else if Q_stricmp(tokenStr, b"WP_DISRUPTOR\0".as_ptr() as *const c_char) == 0 {
            tag = WP_DISRUPTOR;
        } else if Q_stricmp(tokenStr, b"WP_BOWCASTER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_BOWCASTER;
        } else if Q_stricmp(tokenStr, b"WP_REPEATER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_REPEATER;
        } else if Q_stricmp(tokenStr, b"WP_DEMP2\0".as_ptr() as *const c_char) == 0 {
            tag = WP_DEMP2;
        } else if Q_stricmp(tokenStr, b"WP_FLECHETTE\0".as_ptr() as *const c_char) == 0 {
            tag = WP_FLECHETTE;
        } else if Q_stricmp(tokenStr, b"WP_ROCKET_LAUNCHER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_ROCKET_LAUNCHER;
        } else if Q_stricmp(tokenStr, b"WP_CONCUSSION\0".as_ptr() as *const c_char) == 0 {
            tag = WP_CONCUSSION;
        } else if Q_stricmp(tokenStr, b"WP_THERMAL\0".as_ptr() as *const c_char) == 0 {
            tag = WP_THERMAL;
        } else if Q_stricmp(tokenStr, b"WP_TRIP_MINE\0".as_ptr() as *const c_char) == 0 {
            tag = WP_TRIP_MINE;
        } else if Q_stricmp(tokenStr, b"WP_DET_PACK\0".as_ptr() as *const c_char) == 0 {
            tag = WP_DET_PACK;
        }
        //	else if (!Q_stricmp(tokenStr,"WP_TRICORDER"))
        //		tag = WP_TRICORDER;
        else if Q_stricmp(tokenStr, b"WP_BOT_LASER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_BOT_LASER;
        } else if Q_stricmp(tokenStr, b"WP_EMPLACED_GUN\0".as_ptr() as *const c_char) == 0 {
            tag = WP_EMPLACED_GUN;
        } else if Q_stricmp(tokenStr, b"WP_MELEE\0".as_ptr() as *const c_char) == 0 {
            tag = WP_MELEE;
        } else if Q_stricmp(tokenStr, b"WP_TURRET\0".as_ptr() as *const c_char) == 0 {
            tag = WP_TURRET;
        } else if Q_stricmp(tokenStr, b"WP_ATST_MAIN\0".as_ptr() as *const c_char) == 0 {
            tag = WP_ATST_MAIN;
        } else if Q_stricmp(tokenStr, b"WP_ATST_SIDE\0".as_ptr() as *const c_char) == 0 {
            tag = WP_ATST_SIDE;
        } else if Q_stricmp(tokenStr, b"WP_TIE_FIGHTER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_TIE_FIGHTER;
        } else if Q_stricmp(tokenStr, b"WP_RAPID_FIRE_CONC\0".as_ptr() as *const c_char) == 0 {
            tag = WP_RAPID_FIRE_CONC;
        } else if Q_stricmp(tokenStr, b"WP_BLASTER_PISTOL\0".as_ptr() as *const c_char) == 0 {
            tag = WP_BLASTER_PISTOL;
        } else if Q_stricmp(tokenStr, b"WP_JAWA\0".as_ptr() as *const c_char) == 0 {
            tag = WP_JAWA;
        } else if Q_stricmp(tokenStr, b"WP_TUSKEN_RIFLE\0".as_ptr() as *const c_char) == 0 {
            tag = WP_TUSKEN_RIFLE;
        } else if Q_stricmp(tokenStr, b"WP_TUSKEN_STAFF\0".as_ptr() as *const c_char) == 0 {
            tag = WP_TUSKEN_STAFF;
        } else if Q_stricmp(tokenStr, b"WP_SCEPTER\0".as_ptr() as *const c_char) == 0 {
            tag = WP_SCEPTER;
        } else if Q_stricmp(tokenStr, b"WP_NOGHRI_STICK\0".as_ptr() as *const c_char) == 0 {
            tag = WP_NOGHRI_STICK;
        } else if Q_stricmp(tokenStr, b"AMMO_FORCE\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_FORCE;
        } else if Q_stricmp(tokenStr, b"AMMO_BLASTER\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_BLASTER;
        } else if Q_stricmp(tokenStr, b"AMMO_POWERCELL\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_POWERCELL;
        } else if Q_stricmp(tokenStr, b"AMMO_METAL_BOLTS\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_METAL_BOLTS;
        } else if Q_stricmp(tokenStr, b"AMMO_ROCKETS\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_ROCKETS;
        } else if Q_stricmp(tokenStr, b"AMMO_EMPLACED\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_EMPLACED;
        } else if Q_stricmp(tokenStr, b"AMMO_THERMAL\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_THERMAL;
        } else if Q_stricmp(tokenStr, b"AMMO_TRIPMINE\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_TRIPMINE;
        } else if Q_stricmp(tokenStr, b"AMMO_DETPACK\0".as_ptr() as *const c_char) == 0 {
            tag = AMMO_DETPACK;
        } else if Q_stricmp(tokenStr, b"FP_HEAL\0".as_ptr() as *const c_char) == 0 {
            tag = FP_HEAL;
        } else if Q_stricmp(tokenStr, b"FP_LEVITATION\0".as_ptr() as *const c_char) == 0 {
            tag = FP_LEVITATION;
        } else if Q_stricmp(tokenStr, b"FP_SPEED\0".as_ptr() as *const c_char) == 0 {
            tag = FP_SPEED;
        } else if Q_stricmp(tokenStr, b"FP_PUSH\0".as_ptr() as *const c_char) == 0 {
            tag = FP_PUSH;
        } else if Q_stricmp(tokenStr, b"FP_PULL\0".as_ptr() as *const c_char) == 0 {
            tag = FP_PULL;
        } else if Q_stricmp(tokenStr, b"FP_TELEPATHY\0".as_ptr() as *const c_char) == 0 {
            tag = FP_TELEPATHY;
        } else if Q_stricmp(tokenStr, b"FP_GRIP\0".as_ptr() as *const c_char) == 0 {
            tag = FP_GRIP;
        } else if Q_stricmp(tokenStr, b"FP_LIGHTNING\0".as_ptr() as *const c_char) == 0 {
            tag = FP_LIGHTNING;
        } else if Q_stricmp(tokenStr, b"FP_SABERTHROW\0".as_ptr() as *const c_char) == 0 {
            tag = FP_SABERTHROW;
        } else if Q_stricmp(tokenStr, b"ITM_BATTERY_PICKUP\0".as_ptr() as *const c_char) == 0 {
            tag = ITM_BATTERY_PICKUP;
        } else if Q_stricmp(tokenStr, b"INV_SEEKER\0".as_ptr() as *const c_char) == 0 {
            tag = INV_SEEKER;
        } else if Q_stricmp(tokenStr, b"ITM_SHIELD_PICKUP\0".as_ptr() as *const c_char) == 0 {
            tag = ITM_SHIELD_PICKUP;
        } else if Q_stricmp(tokenStr, b"INV_BACTA_CANISTER\0".as_ptr() as *const c_char) == 0 {
            tag = INV_BACTA_CANISTER;
        } else if Q_stricmp(tokenStr, b"ITM_DATAPAD_PICKUP\0".as_ptr() as *const c_char) == 0 {
            tag = ITM_DATAPAD_PICKUP;
        } else if Q_stricmp(tokenStr, b"INV_ELECTROBINOCULARS\0".as_ptr() as *const c_char) == 0 {
            tag = INV_ELECTROBINOCULARS;
        } else if Q_stricmp(tokenStr, b"INV_SENTRY\0".as_ptr() as *const c_char) == 0 {
            tag = INV_SENTRY;
        } else if Q_stricmp(tokenStr, b"INV_LIGHTAMP_GOGGLES\0".as_ptr() as *const c_char) == 0 {
            tag = INV_LIGHTAMP_GOGGLES;
        } else if Q_stricmp(tokenStr, b"INV_GOODIE_KEY\0".as_ptr() as *const c_char) == 0 {
            tag = INV_GOODIE_KEY;
        } else if Q_stricmp(tokenStr, b"INV_SECURITY_KEY\0".as_ptr() as *const c_char) == 0 {
            tag = INV_SECURITY_KEY;
        } else if Q_stricmp(tokenStr, b"ITM_MEDPAK_PICKUP\0".as_ptr() as *const c_char) == 0 {
            tag = ITM_MEDPAK_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SHIELD_SM_PICKUP\0".as_ptr() as *const c_char) == 0 {
            tag = ITM_SHIELD_SM_PICKUP;
        } else if Q_stricmp(tokenStr, b"ITM_SHIELD_LRG_PICKUP\0".as_ptr() as *const c_char) == 0 {
            tag = ITM_SHIELD_LRG_PICKUP;
        } else {
            tag = WP_BRYAR_PISTOL;
            //This error was slipping through too much, causing runaway exceptions and shutting down, so now it's a real error when not in Final
            #[cfg(not(FINAL_BUILD))]
            G_Error(b"ERROR: bad tagname in external item data '%s'\n\0".as_ptr() as *const c_char, tokenStr);
            #[cfg(FINAL_BUILD)]
            gi.Printf(b"WARNING: bad tagname in external item data '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].giTag = tag;
    }
}

fn IT_Type(holdBuf: *mut *const c_char) {
    unsafe {
        let mut item_type: c_int;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        if Q_stricmp(tokenStr, b"IT_BAD\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_BAD;
        } else if Q_stricmp(tokenStr, b"IT_WEAPON\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_WEAPON;
        } else if Q_stricmp(tokenStr, b"IT_AMMO\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_AMMO;
        } else if Q_stricmp(tokenStr, b"IT_ARMOR\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_ARMOR;
        } else if Q_stricmp(tokenStr, b"IT_HEALTH\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_HEALTH;
        } else if Q_stricmp(tokenStr, b"IT_HOLDABLE\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_HOLDABLE;
        } else if Q_stricmp(tokenStr, b"IT_BATTERY\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_BATTERY;
        } else if Q_stricmp(tokenStr, b"IT_HOLOCRON\0".as_ptr() as *const c_char) == 0 {
            item_type = IT_HOLOCRON;
        } else {
            item_type = IT_BAD;
            gi.Printf(b"WARNING: bad itemname in external item data '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].giType = item_type;
    }
}

fn IT_Icon(holdBuf: *mut *const c_char) {
    unsafe {
        let mut len: usize;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        len = strlen(tokenStr);
        len += 1;
        if len > 32 {
            len = 32;
            gi.Printf(b"WARNING: icon too long in external ITEMS.DAT '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].icon = G_NewString(tokenStr);
    }
}

fn IT_Count(holdBuf: *mut *const c_char) {
    unsafe {
        let mut tokenInt: c_int;

        if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
            SkipRestOfLine(holdBuf);
            return;
        }

        if (tokenInt < 0) || (tokenInt > 1000) {
            // FIXME :What are the right values?
            gi.Printf(b"WARNING: bad Count in external item data '%d'\n\0".as_ptr() as *const c_char, tokenInt);
            return;
        }
        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].quantity = tokenInt;
    }
}

fn IT_Min(holdBuf: *mut *const c_char) {
    unsafe {
        let mut tokenInt: c_int;
        let mut i: c_int;

        i = 0;
        while i < 3 {
            if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
                SkipRestOfLine(holdBuf);
                return;
            }

            let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
            (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].mins[i as usize] = tokenInt;
            i += 1;
        }
    }
}

fn IT_Max(holdBuf: *mut *const c_char) {
    unsafe {
        let mut tokenInt: c_int;
        let mut i: c_int;

        i = 0;
        while i < 3 {
            if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
                SkipRestOfLine(holdBuf);
                return;
            }

            let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
            (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].maxs[i as usize] = tokenInt;
            i += 1;
        }
    }
}

fn IT_PickupSound(holdBuf: *mut *const c_char) {
    unsafe {
        let mut len: usize;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        len = strlen(tokenStr);
        len += 1;
        if len > 32 {
            len = 32;
            gi.Printf(b"WARNING: Pickup Sound too long in external ITEMS.DAT '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].pickup_sound = G_NewString(tokenStr);
    }
}

#[cfg(feature = "_IMMERSION")]
fn IT_PickupForce(holdBuf: *mut *const c_char) {
    unsafe {
        let mut len: usize;
        let mut tokenStr: *const c_char;

        if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
            return;
        }

        len = strlen(tokenStr);
        len += 1;
        if len > 32 {
            len = 32;
            gi.Printf(b"WARNING: Pickup Force too long in external ITEMS.DAT '%s'\n\0".as_ptr() as *const c_char, tokenStr);
        }

        let item_num = (*core::ptr::addr_of_mut!(itemParms)).itemNum as usize;
        (*core::ptr::addr_of_mut!(bg_itemlist))[item_num].pickup_force = G_NewString(tokenStr);
    }
}

fn IT_ParseWeaponParms(holdBuf: *mut *const c_char) {
    unsafe {
        let mut token: *const c_char;
        let mut i: usize;

        loop {
            token = COM_ParseExt(holdBuf, 1);

            if Q_stricmp(token, b"}\0".as_ptr() as *const c_char) == 0 {
                // End of data for this weapon
                break;
            }

            // Loop through possible parameters
            i = 0;
            while i < IT_PARM_MAX {
                if Q_stricmp(token, ITEM_PARMS[i].parmName) == 0 {
                    (ITEM_PARMS[i].func)(holdBuf);
                    break;
                }
                i += 1;
            }

            if i < IT_PARM_MAX {
                // Find parameter???
                continue;
            }

            gi.Printf(b"bad parameter in external weapon data '%s'\n\0".as_ptr() as *const c_char, token);
            SkipRestOfLine(holdBuf);
        }
    }
}

fn IT_ParseParms(buffer: *const c_char) {
    unsafe {
        let mut holdBuf: *const c_char = buffer;
        let mut token: *const c_char;

        //	bg_numItems = 0;
        COM_BeginParseSession();

        loop {
            token = COM_ParseExt(&mut holdBuf, 1);

            if token.is_null() {
                break;
            }

            if Q_stricmp(token, b"{\0".as_ptr() as *const c_char) == 0 {
                IT_ParseWeaponParms(&mut holdBuf);
            }
        }

        //	--bg_numItems;
    }
}

pub fn IT_LoadItemParms() {
    unsafe {
        let mut buffer: *mut c_char = core::ptr::null_mut();
        let len: c_int;

        len = gi.FS_ReadFile(b"ext_data/items.dat\0".as_ptr() as *const c_char, &mut (buffer as *mut c_void));

        IT_ParseParms(buffer as *const c_char);

        gi.FS_FreeFile(buffer as *mut c_void); //let go of the buffer
    }
}

// Stub functions - these should be implemented in appropriate modules
fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    // Stub: case-insensitive string compare
    0
}

fn SkipRestOfLine(data: *mut *const c_char) {
    // Stub: skip to end of current line in parser
}
