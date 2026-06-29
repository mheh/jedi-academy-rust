// leave this line at the top for all g_xxxx.cpp files...
// (C header: g_headers.h)

//g_inventory.cpp

// (C headers: g_local.h)

use core::ffi::{c_int, c_char};

// ============================================================================
// Type definitions
// ============================================================================

pub type qboolean = c_int;
pub type vec3_t = [f32; 3];

// Constants from g_items.h
const INV_ELECTROBINOCULARS: c_int = 0;
const INV_BACTA_CANISTER: c_int = 1;
const INV_SEEKER: c_int = 2;
const INV_LIGHTAMP_GOGGLES: c_int = 3;
const INV_SENTRY: c_int = 4;
const INV_GOODIE_KEY: c_int = 5;
const INV_SECURITY_KEY: c_int = 6;

// Constants from q_shared.h
const MAX_INVENTORY: usize = 15;
const MAX_SECURITY_KEYS: usize = 5;
const MAX_SECURITY_KEY_MESSSAGE: usize = 24;

// qboolean values
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// ============================================================================
// Structure definitions for playerState_t and related types
// ============================================================================

#[repr(C)]
pub struct playerState_s {
    pub commandTime: c_int,
    pub pm_type: c_int,
    pub bobCycle: c_int,
    pub pm_flags: c_int,
    pub pm_time: c_int,
    pub origin: vec3_t,
    pub velocity: vec3_t,
    pub weaponTime: c_int,
    pub weaponChargeTime: c_int,
    pub rechargeTime: c_int,
    pub gravity: c_int,
    pub leanofs: c_int,
    pub friction: c_int,
    pub speed: c_int,
    pub delta_angles: [c_int; 3],
    pub groundEntityNum: c_int,
    pub legsAnim: c_int,
    pub legsAnimTimer: c_int,
    pub torsoAnim: c_int,
    pub torsoAnimTimer: c_int,
    pub movementDir: c_int,
    pub eFlags: c_int,
    pub eventSequence: c_int,
    pub events: [c_int; 4],
    pub eventParms: [c_int; 4],
    pub externalEvent: c_int,
    pub externalEventParm: c_int,
    pub externalEventTime: c_int,
    pub clientNum: c_int,
    pub weapon: c_int,
    pub weaponstate: c_int,
    pub batteryCharge: c_int,
    pub viewangles: vec3_t,
    pub legsYaw: f32,
    pub viewheight: c_int,
    pub damageEvent: c_int,
    pub damageYaw: c_int,
    pub damagePitch: c_int,
    pub damageCount: c_int,
    pub stats: [c_int; 16],
    pub persistant: [c_int; 16],
    pub powerups: [c_int; 16],
    pub ammo: [c_int; 16],
    pub inventory: [c_int; MAX_INVENTORY],
    pub security_key_message: [[c_char; MAX_SECURITY_KEY_MESSSAGE]; MAX_SECURITY_KEYS],
    // (remaining fields omitted - this module only accesses inventory and security_key_message)
}
pub type playerState_t = playerState_s;

#[repr(C)]
pub struct gclient_s {
    pub ps: playerState_t,
    // (remaining fields are opaque to this module)
}
pub type gclient_t = gclient_s;

#[repr(C)]
pub struct entityState_s {
    pub _data: [u8; 532],
}
pub type entityState_t = entityState_s;

#[repr(C)]
pub struct gentity_s {
    pub s: entityState_t,
    pub client: *mut gclient_t,
}
pub type gentity_t = gentity_s;

// ============================================================================
// External function declarations
// ============================================================================

extern "C" {
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int, bBarfIfTooLong: qboolean);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// ============================================================================
// Functions
// ============================================================================

/*
================
Goodie Keys
================
*/
#[allow(non_snake_case)]
pub unsafe fn INV_GoodieKeyGive(target: *mut gentity_t) -> qboolean {
    if target.is_null() || (*target).client.is_null() {
        return qfalse;
    }

    (*(*target).client).ps.inventory[INV_GOODIE_KEY as usize] += 1;
    return qtrue;
}

#[allow(non_snake_case)]
pub unsafe fn INV_GoodieKeyTake(target: *mut gentity_t) -> qboolean {
    if target.is_null() || (*target).client.is_null() {
        return qfalse;
    }

    if (*(*target).client).ps.inventory[INV_GOODIE_KEY as usize] != 0 {
        (*(*target).client).ps.inventory[INV_GOODIE_KEY as usize] -= 1;
        return qtrue;
    }

    //had no keys
    return qfalse;
}

#[allow(non_snake_case)]
pub unsafe fn INV_GoodieKeyCheck(target: *mut gentity_t) -> c_int {
    if target.is_null() || (*target).client.is_null() {
        return qfalse;
    }

    if (*(*target).client).ps.inventory[INV_GOODIE_KEY as usize] != 0 {
        //found a key
        return INV_GOODIE_KEY;
    }

    //no keys
    return qfalse;
}

/*
================
Security Keys
================
*/
#[allow(non_snake_case)]
pub unsafe fn INV_SecurityKeyGive(target: *mut gentity_t, keyname: *const c_char) -> qboolean {
    if target.is_null() || keyname.is_null() || (*target).client.is_null() {
        return qfalse;
    }

    for i in 0..=4 {
        if (*(*target).client).ps.security_key_message[i][0] == 0 as c_char {
            //fill in the first empty slot we find with this key
            (*(*target).client).ps.inventory[INV_SECURITY_KEY as usize] += 1; // He got the key
            Q_strncpyz(
                core::ptr::addr_of_mut!((*(*target).client).ps.security_key_message[i][0]),
                keyname,
                MAX_SECURITY_KEY_MESSSAGE as c_int,
                qtrue,
            );
            return qtrue;
        }
    }
    //couldn't find an empty slot
    return qfalse;
}

#[allow(non_snake_case)]
pub unsafe fn INV_SecurityKeyTake(target: *mut gentity_t, keyname: *mut c_char) {
    if target.is_null() || keyname.is_null() || (*target).client.is_null() {
        return;
    }

    for i in 0..=4 {
        if (*(*target).client).ps.security_key_message[i][0] != 0 as c_char {
            if Q_stricmp(
                keyname,
                (*(*target).client).ps.security_key_message[i].as_ptr(),
            ) == 0
            {
                (*(*target).client).ps.inventory[INV_SECURITY_KEY as usize] -= 1; // Take the key
                (*(*target).client).ps.security_key_message[i][0] = 0 as c_char;
                return;
            }
        }
        /*
        //don't do this because we could have removed one that's between 2 valid ones
        else
        {
            break;
        }
        */
    }
}

#[allow(non_snake_case)]
pub unsafe fn INV_SecurityKeyCheck(target: *mut gentity_t, keyname: *mut c_char) -> qboolean {
    if target.is_null() || keyname.is_null() || (*target).client.is_null() {
        return qfalse;
    }

    for i in 0..=4 {
        if (*(*target).client).ps.inventory[INV_SECURITY_KEY as usize] != 0
            && (*(*target).client).ps.security_key_message[i][0] != 0 as c_char
        {
            if Q_stricmp(
                keyname,
                (*(*target).client).ps.security_key_message[i].as_ptr(),
            ) == 0
            {
                return qtrue;
            }
        }
        /*
        //don't do this because we could have removed one that's between 2 valid ones
        else
        {
            break;
        }
        */
    }

    return qfalse;
}
