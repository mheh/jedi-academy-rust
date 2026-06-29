// this include must remain at the top of every bg_xxxx CPP file
// #include "common_headers.h"

// included in both game dll and client

// #include "q_shared.h"
// #include "g_local.h"
// #include "bg_public.h"
// #include "g_items.h"
// #include "g_vehicles.h"

use core::ffi::{c_int, c_char};

extern "C" {
    pub static mut weaponData: [weaponData_t; WP_NUM_WEAPONS];
    pub static mut ammoData: [ammoData_t; AMMO_MAX];
}

// Foreign type declarations (to be imported from their defining modules)
#[repr(C)]
pub struct weaponData_t;

#[repr(C)]
pub struct ammoData_t;

#[repr(C)]
pub struct gitem_t;

#[repr(C)]
pub struct trajectory_t;

#[repr(C)]
pub struct playerState_t;

#[repr(C)]
pub struct entityState_t;

#[repr(C)]
pub struct Vehicle_t;

pub type weapon_t = c_int;
pub type ammo_t = c_int;
pub type qboolean = c_int;
pub type vec3_t = [f32; 3];

// Constants from enums/defines
const WP_NUM_WEAPONS: usize = 14; // placeholder
const AMMO_MAX: usize = 14; // placeholder
const ITM_NUM_ITEMS: c_int = 64; // placeholder
const MAX_PS_EVENTS: c_int = 4; // placeholder
const MAX_POWERUPS: c_int = 16; // placeholder
const MAX_BATTERIES: c_int = 99; // placeholder

const STAT_WEAPONS: usize = 4; // placeholder
const STAT_ARMOR: usize = 5; // placeholder
const STAT_MAX_HEALTH: usize = 7; // placeholder
const STAT_HEALTH: usize = 0; // placeholder

const IT_WEAPON: c_int = 1; // placeholder
const IT_AMMO: c_int = 2; // placeholder
const IT_ARMOR: c_int = 3; // placeholder
const IT_HEALTH: c_int = 4; // placeholder
const IT_BATTERY: c_int = 5; // placeholder
const IT_HOLOCRON: c_int = 6; // placeholder
const IT_HOLDABLE: c_int = 7; // placeholder

const AMMO_FORCE: c_int = 0; // placeholder
const AMMO_THERMAL: c_int = 4; // placeholder
const AMMO_DETPACK: c_int = 5; // placeholder
const AMMO_TRIPMINE: c_int = 6; // placeholder

const WP_THERMAL: c_int = 5; // placeholder
const WP_DET_PACK: c_int = 6; // placeholder
const WP_TRIP_MINE: c_int = 7; // placeholder
const WP_SABER: c_int = 1; // placeholder

const INV_ELECTROBINOCULARS: c_int = 0; // placeholder
const INV_SENTRY: c_int = 3; // placeholder

const FP_RAGE: c_int = 0; // placeholder

const ERR_DROP: c_int = 1; // placeholder

const M_PI: f32 = 3.14159265358979323846;
const DEG2RAD_MULT: f32 = M_PI / 180.0;

const TR_STATIONARY: c_int = 0;
const TR_INTERPOLATE: c_int = 1;
const TR_LINEAR: c_int = 2;
const TR_SINE: c_int = 3;
const TR_LINEAR_STOP: c_int = 4;
const TR_NONLINEAR_STOP: c_int = 5;
const TR_GRAVITY: c_int = 6;

const PM_INTERMISSION: c_int = 0;
const PM_SPECTATOR: c_int = 1;

const ET_INVISIBLE: c_int = 0;
const ET_PLAYER: c_int = 1;

const CLASS_VEHICLE: c_int = 0;

const YAW: usize = 2;
const ROLL: usize = 0;

#[define]
const PICKUPSOUND: &str = "sound/weapons/w_pkup.wav";

/*QUAKED weapon_***** ( 0 0 0 ) (-16 -16 -16) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
DO NOT USE THIS CLASS, IT JUST HOLDS GENERAL INFORMATION for weapons, ammo, and item pickups.
The suspended flag will allow items to hang in the air, otherwise they are dropped to the next surface.
The NOPLAYER flag makes it so player cannot pick it up.
The ALLOWNPC flag allows only NPCs to pick it up, too
USEPICKUP - Player must be holding "use" to pick it up
STATIONARY - Cannot be moved around by force push/pull, radius damage, knockback, etc...

If an item is the target of another entity, it will spawn as normal, use INVISIBLE to hide it.

An item fires all of its targets when it is picked up.  If the toucher can't carry it, the targets won't be fired.

"wait"	override the default wait before respawning.  -1 = never respawn automatically, which can be used with targeted spawning.
"random" random number of plus or minus seconds varied from the respawn time
"count" override quantity or duration on most items.
"team" only this team can pick it up
	"player"
	"neutral"
	"enemy"
*/

/*QUAKED weapon_stun_baton (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/stun_baton/baton.md3"
*/
/*QUAKED weapon_saber (.3 .3 1) (-16 -16 -8) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID LEANING INVISIBLE NOGLOW USEPICKUP STATIONARY
SUSPENDED - allow items to hang in the air, otherwise they are dropped to the next surface.
NOPLAYER - makes it so player cannot pick it up.
ALLOWNPC - allows only NPCs to pick it up, too
LEANING - lean back against wall
NOGLOW - No Glow
USEPICKUP - Player must be holding "use" to pick it up
STATIONARY - Cannot be moved around by force push/pull, radius damage, knockback, etc...

model="/models/weapons2/saber/saber_w.md3"
When picked up, will be used as a second saber unless:
	1) It's a two-handed saber
	2) The old saber was two-handed
	3) You set "saberSolo" to "1"
	4) You have 2 sabers and the saber pickup is on your right when you touch it

saberType - entry name from sabers.cfg - which kind of saber this is - use "player" to make it so that the saber will be whatever saber the player is configured to use
saberColor - red, orange, yellow, green, blue, and purple
saberLeftHand - always be added as a left-hand saber
saberSolo - set to "1" and this will be the only saber the person who picks this up will be holding
saberPitch - if set "LEANING" flag, you can specify the exact pitch to lean forward/back
count - how many you can pick up before it's removed (default is 1, -1 is infinite)
*/
/*QUAKED weapon_bryar_pistol (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/briar_pistol/briar_pistol.md3"
*/
/*QUAKED weapon_blaster_pistol (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/blaster_pistol/blaster_pistol.md3"
*/
/*QUAKED weapon_blaster (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/blaster_r/blaster.md3"
*/
/*QUAKED weapon_disruptor (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/disruptor/disruptor.md3"
*/
/*QUAKED weapon_bowcaster (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/bowcaster/bowcaster.md3"
*/
/*QUAKED weapon_repeater (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/heavy_repeater/heavy_repeater.md3"
*/
/*QUAKED weapon_demp2 (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/demp2/demp2.md3"
*/
/*QUAKED weapon_flechette (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/golan_arms/golan_arms.md3"
*/
/*QUAKED weapon_concussion_rifle (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/c_rifle/c_rifle.md3"
*/
/*QUAKED weapon_rocket_launcher (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/merr_sonn/merr_sonn.md3"
*/
/*QUAKED weapon_thermal (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/thermal/thermal.md3"
*/
/*QUAKED weapon_trip_mine (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/laser_trap/laser_trap.md3"
*/
/*QUAKED weapon_det_pack (.3 .3 1) (-16 -16 -2) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
model="/models/weapons2/detpack/det_pack.md3"
*/

/*QUAKED item_seeker (.3 .3 1) (-8 -8 -4) (8 8 16) suspended
30 seconds of seeker drone
*/
/*QUAKED item_bacta (.3 .3 1) (-8 -8 0) (8 8 16) suspended
model="/models/items/bacta.md3"
*/
/*QUAKED item_datapad (.3 .3 1) (-8 -8 0) (8 8 16) suspended
model="/models/items/datapad.md3"
*/
/*QUAKED item_binoculars (.3 .3 1) (-8 -8 0) (8 8 16) suspended
model="/models/items/binoculars.md3"
*/
/*QUAKED item_sentry_gun (.3 .3 1) (-8 -8 0) (8 8 16) suspended
*/
/*QUAKED item_la_goggles (.3 .3 1) (-8 -8 0) (8 8 16) suspended
*/
/*QUAKED ammo_force (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
Ammo for the force.
*/
/*QUAKED ammo_blaster (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
Ammo for the Bryar and Blaster pistols.
*/
/*QUAKED ammo_powercell (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
Ammo for Tenloss Disruptor, Wookie Bowcaster, and the Destructive Electro Magnetic Pulse (demp2 ) guns
*/
/*QUAKED ammo_metallic_bolts (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
Ammo for Imperial Heavy Repeater and the Golan Arms Flechette
*/
/*QUAKED ammo_rockets (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
Ammo for Merr-Sonn portable missile launcher
*/
/*QUAKED ammo_thermal (.3 .5 1) (-16 -16 -0) (16 16 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
Belt of thermal detonators
*/
/*QUAKED ammo_tripmine (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
3 pack of tripmines
*/
/*QUAKED ammo_detpack (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
3 pack of detpacks
*/

/*QUAKED item_medpak_instant (.3 .3 1) (-8 -8 -4) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
*/

/*QUAKED item_shield_sm_instant (.3 .3 1) (-8 -8 -4) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
*/

/*QUAKED item_shield_lrg_instant (.3 .3 1) (-8 -8 -4) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID VERTICAL INVISIBLE NOGLOW USEPICKUP STATIONARY
*/
/*QUAKED item_goodie_key (.3 .3 1) (-8 -8 0) (8 8 16) suspended
*/
/*QUAKED item_security_key (.3 .3 1) (-8 -8 0) (8 8 16) suspended
message - used to differentiate one key from another.
*/
/*QUAKED item_battery (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
model="/models/items/battery.md3"
battery pickup item
*/

/*QUAKED holocron_force_heal (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force heal pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_levitation (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force levitation pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_speed (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force speed pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_push (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force push pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_pull (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force pull pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_telepathy (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force telepathy pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_grip (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force grip pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_lightining (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force lighting pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

/*QUAKED holocron_force_saberthrow (.3 .5 1) (-8 -8 -0) (8 8 16) SUSPEND NOPLAYER ALLOWNPC NOTSOLID
force saberthrow pickup item

"count"     level of force power this holocron gives activator ( range: 0-3, default 1)
*/

extern "C" {
    pub static mut bg_itemlist: [gitem_t; (ITM_NUM_ITEMS as usize) + 1]; // need a null on the end
    pub static mut g_gravity: *mut crate::code::game::cvar_t;
    pub static mut g_entities: *mut crate::code::game::gentity_t;

    fn Com_Error(level: c_int, msg: *const c_char, ...);
    fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    fn VectorClear(v: *mut vec3_t);
    fn VectorMA(veca: *const vec3_t, scale: f32, vecb: *const vec3_t, vecc: *mut vec3_t);
    fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn AngleSubtract(angle1: f32, angle2: f32) -> f32;
    fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    fn DotProduct(v1: *const vec3_t, v2: *const vec3_t) -> f32;
}

// int		bg_numItems = sizeof(bg_itemlist) / sizeof(bg_itemlist[0]) ;
pub const bg_numItems: c_int = ITM_NUM_ITEMS;

/*
===============
FindItemForWeapon

===============
*/
pub unsafe extern "C" fn FindItemForWeapon(weapon: weapon_t) -> *mut gitem_t {
    let mut i: c_int;

    i = 1;
    while i < bg_numItems {
        if unsafe {
            (*core::ptr::addr_of!(bg_itemlist[i as usize])).giType == IT_WEAPON
                && (*core::ptr::addr_of!(bg_itemlist[i as usize])).giTag == weapon
        } {
            return unsafe { core::ptr::addr_of_mut!(bg_itemlist[i as usize]) };
        }
        i += 1;
    }

    Com_Error(ERR_DROP, b"Couldn't find item for weapon %i\0".as_ptr() as *const c_char, weapon);
    core::ptr::null_mut()
}

//----------------------------------------------
pub unsafe extern "C" fn FindItemForInventory(inv: c_int) -> *mut gitem_t {
    let mut i: c_int;
    let mut it: *mut gitem_t;

    // Now just check for any other kind of item.
    i = 1;
    while i < bg_numItems {
        it = unsafe { core::ptr::addr_of_mut!(bg_itemlist[i as usize]) };

        if unsafe { (*it).giType == IT_HOLDABLE } {
            if unsafe { (*it).giTag == inv } {
                return it;
            }
        }
        i += 1;
    }

    Com_Error(
        ERR_DROP,
        b"Couldn't find item for inventory %i\0".as_ptr() as *const c_char,
        inv,
    );
    core::ptr::null_mut()
}

/*
===============
FindItemForWeapon

===============
*/
pub unsafe extern "C" fn FindItemForAmmo(ammo: ammo_t) -> *mut gitem_t {
    let mut i: c_int;

    i = 1;
    while i < bg_numItems {
        if unsafe {
            (*core::ptr::addr_of!(bg_itemlist[i as usize])).giType == IT_AMMO
                && (*core::ptr::addr_of!(bg_itemlist[i as usize])).giTag == ammo
        } {
            return unsafe { core::ptr::addr_of_mut!(bg_itemlist[i as usize]) };
        }
        i += 1;
    }

    Com_Error(
        ERR_DROP,
        b"Couldn't find item for ammo %i\0".as_ptr() as *const c_char,
        ammo,
    );
    core::ptr::null_mut()
}

/*
===============
FindItem

===============
*/
pub unsafe extern "C" fn FindItem(className: *const c_char) -> *mut gitem_t {
    let mut i: c_int;

    i = 1;
    while i < bg_numItems {
        if Q_stricmp(
            unsafe { (*core::ptr::addr_of!(bg_itemlist[i as usize])).classname },
            className,
        ) == 0
        {
            return unsafe { core::ptr::addr_of_mut!(bg_itemlist[i as usize]) };
        }
        i += 1;
    }

    core::ptr::null_mut()
}

/*
================
BG_CanItemBeGrabbed

Returns false if the item should not be picked up.
This needs to be the same for client side prediction and server use.
================
*/
pub unsafe extern "C" fn BG_CanItemBeGrabbed(ent: *const entityState_t, ps: *const playerState_t) -> qboolean {
    let mut item: *mut gitem_t;

    if unsafe { (*ent).modelindex } < 1 || unsafe { (*ent).modelindex } >= bg_numItems {
        Com_Error(
            ERR_DROP,
            b"BG_CanItemBeGrabbed: index out of range\0".as_ptr() as *const c_char,
        );
    }

    item = unsafe { core::ptr::addr_of_mut!(bg_itemlist[(*ent).modelindex as usize]) };

    match unsafe { (*item).giType } {
        x if x == IT_WEAPON => {
            // See if we already have this weapon.
            if unsafe { ((*ps).stats[STAT_WEAPONS] & (1 << (*item).giTag)) == 0 } {
                // Don't have this weapon yet, so pick it up.
                return 1; // qtrue
            } else if unsafe { (*item).giTag == WP_SABER } {
                // always pick up a saber, might be a new one?
                return 1; // qtrue
            }

            // Make sure that we aren't already full on ammo for this weapon
            if unsafe {
                (*ps).ammo[weaponData[(*item).giTag as usize].ammoIndex as usize]
                    >= ammoData[weaponData[(*item).giTag as usize].ammoIndex as usize].max
            } {
                // full, so don't grab the item
                return 0; // qfalse
            }

            return 1; // qtrue - could use more of this type of ammo, so grab the item
        }

        x if x == IT_AMMO => {
            if unsafe { (*item).giTag != AMMO_FORCE } {
                // since the ammo is the weapon in this case, picking up ammo should actually give you the weapon
                match unsafe { (*item).giTag } {
                    x if x == AMMO_THERMAL => {
                        if unsafe { ((*ps).stats[STAT_WEAPONS] & (1 << WP_THERMAL)) == 0 } {
                            return 1; // qtrue
                        }
                    }
                    x if x == AMMO_DETPACK => {
                        if unsafe { ((*ps).stats[STAT_WEAPONS] & (1 << WP_DET_PACK)) == 0 } {
                            return 1; // qtrue
                        }
                    }
                    x if x == AMMO_TRIPMINE => {
                        if unsafe { ((*ps).stats[STAT_WEAPONS] & (1 << WP_TRIP_MINE)) == 0 } {
                            return 1; // qtrue
                        }
                    }
                    _ => {}
                }

                if unsafe { (*ps).ammo[(*item).giTag as usize] >= ammoData[(*item).giTag as usize].max } {
                    // checkme
                    return 0; // qfalse - can't hold any more
                }
            } else {
                if unsafe { (*ps).forcePower >= ammoData[(*item).giTag as usize].max * 2 } {
                    return 0; // qfalse - can't hold any more
                }
            }

            return 1; // qtrue
        }

        x if x == IT_ARMOR => {
            // we also clamp armor to the maxhealth for handicapping
            if unsafe { (*ps).stats[STAT_ARMOR] >= (*ps).stats[STAT_MAX_HEALTH] } {
                return 0; // qfalse
            }
            return 1; // qtrue
        }

        x if x == IT_HEALTH => {
            if unsafe { ((*ps).forcePowersActive & (1 << FP_RAGE)) != 0 } {
                // ragers can't use health
                return 0; // qfalse
            }
            // don't pick up if already at max
            if unsafe { (*ps).stats[STAT_HEALTH] >= (*ps).stats[STAT_MAX_HEALTH] } {
                return 0; // qfalse
            }
            return 1; // qtrue
        }

        x if x == IT_BATTERY => {
            // don't pick up if already at max
            if unsafe { (*ps).batteryCharge >= MAX_BATTERIES } {
                return 0; // qfalse
            }
            return 1; // qtrue
        }

        x if x == IT_HOLOCRON => {
            // pretty lame but for now you can always pick these up
            return 1; // qtrue
        }

        x if x == IT_HOLDABLE => {
            if unsafe { (*item).giTag >= INV_ELECTROBINOCULARS && (*item).giTag <= INV_SENTRY } {
                // hardcoded--can only pick up five of any holdable
                if unsafe { (*ps).inventory[(*item).giTag as usize] >= 5 } {
                    return 0; // qfalse
                }
            }
            return 1; // qtrue
        }

        _ => {}
    }

    0 // qfalse
}

//======================================================================

/*
================
EvaluateTrajectory

================
*/
pub unsafe extern "C" fn EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: *mut vec3_t) {
    let mut deltaTime: f32;
    let mut phase: f32;
    let mut atTime = atTime; // mutable copy for TR_LINEAR_STOP case

    match unsafe { (*tr).trType } {
        x if x == TR_STATIONARY || x == TR_INTERPOLATE => {
            VectorCopy(unsafe { core::ptr::addr_of!((*tr).trBase) }, result);
        }
        x if x == TR_LINEAR => {
            deltaTime = (atTime as f32 - unsafe { (*tr).trTime } as f32) * 0.001; // milliseconds to seconds
            VectorMA(
                unsafe { core::ptr::addr_of!((*tr).trBase) },
                deltaTime,
                unsafe { core::ptr::addr_of!((*tr).trDelta) },
                result,
            );
        }
        x if x == TR_SINE => {
            deltaTime = (atTime as f32 - unsafe { (*tr).trTime } as f32) / unsafe { (*tr).trDuration } as f32;
            phase = (deltaTime * M_PI * 2.0).sin();
            VectorMA(
                unsafe { core::ptr::addr_of!((*tr).trBase) },
                phase,
                unsafe { core::ptr::addr_of!((*tr).trDelta) },
                result,
            );
        }
        x if x == TR_LINEAR_STOP => {
            if atTime > unsafe { (*tr).trTime } + unsafe { (*tr).trDuration } {
                atTime = unsafe { (*tr).trTime } + unsafe { (*tr).trDuration };
            }
            // old totally linear
            deltaTime = (atTime as f32 - unsafe { (*tr).trTime } as f32) * 0.001; // milliseconds to seconds
            if deltaTime < 0.0 {
                // going past the total duration
                deltaTime = 0.0;
            }
            VectorMA(
                unsafe { core::ptr::addr_of!((*tr).trBase) },
                deltaTime,
                unsafe { core::ptr::addr_of!((*tr).trDelta) },
                result,
            );
        }
        x if x == TR_NONLINEAR_STOP => {
            if atTime > unsafe { (*tr).trTime } + unsafe { (*tr).trDuration } {
                atTime = unsafe { (*tr).trTime } + unsafe { (*tr).trDuration };
            }
            // new slow-down at end
            if atTime as f32 - unsafe { (*tr).trTime } as f32 > unsafe { (*tr).trDuration } as f32
                || atTime as f32 - unsafe { (*tr).trTime } as f32 <= 0.0
            {
                deltaTime = 0.0;
            } else {
                // FIXME: maybe scale this somehow?  So that it starts out faster and stops faster?
                deltaTime = unsafe { (*tr).trDuration } as f32
                    * 0.001
                    * ((90.0 - (90.0 * (atTime as f32 - unsafe { (*tr).trTime } as f32) / unsafe { (*tr).trDuration } as f32))
                        * DEG2RAD_MULT)
                        .cos();
            }
            VectorMA(
                unsafe { core::ptr::addr_of!((*tr).trBase) },
                deltaTime,
                unsafe { core::ptr::addr_of!((*tr).trDelta) },
                result,
            );
        }
        x if x == TR_GRAVITY => {
            deltaTime = (atTime as f32 - unsafe { (*tr).trTime } as f32) * 0.001; // milliseconds to seconds
            VectorMA(
                unsafe { core::ptr::addr_of!((*tr).trBase) },
                deltaTime,
                unsafe { core::ptr::addr_of!((*tr).trDelta) },
                result,
            );
            unsafe { (*result)[2] -= 0.5 * (*g_gravity).value * deltaTime * deltaTime }; // DEFAULT_GRAVITY
        }
        _ => {
            Com_Error(
                ERR_DROP,
                b"EvaluateTrajectory: unknown trType: %i\0".as_ptr() as *const c_char,
                unsafe { (*tr).trTime },
            );
        }
    }
}

/*
================
EvaluateTrajectoryDelta

Returns current speed at given time
================
*/
pub unsafe extern "C" fn EvaluateTrajectoryDelta(tr: *const trajectory_t, atTime: c_int, result: *mut vec3_t) {
    let mut deltaTime: f32;
    let mut phase: f32;

    match unsafe { (*tr).trType } {
        x if x == TR_STATIONARY || x == TR_INTERPOLATE => {
            VectorClear(result);
        }
        x if x == TR_LINEAR => {
            VectorCopy(unsafe { core::ptr::addr_of!((*tr).trDelta) }, result);
        }
        x if x == TR_SINE => {
            deltaTime = (atTime as f32 - unsafe { (*tr).trTime } as f32) / unsafe { (*tr).trDuration } as f32;
            phase = (deltaTime * M_PI * 2.0).cos(); // derivative of sin = cos
            phase *= 0.5;
            VectorScale(unsafe { core::ptr::addr_of!((*tr).trDelta) }, phase, result);
        }
        x if x == TR_LINEAR_STOP => {
            if atTime > unsafe { (*tr).trTime } + unsafe { (*tr).trDuration } {
                VectorClear(result);
                return;
            }
            VectorCopy(unsafe { core::ptr::addr_of!((*tr).trDelta) }, result);
        }
        x if x == TR_NONLINEAR_STOP => {
            if atTime as f32 - unsafe { (*tr).trTime } as f32 > unsafe { (*tr).trDuration } as f32
                || atTime as f32 - unsafe { (*tr).trTime } as f32 <= 0.0
            {
                VectorClear(result);
                return;
            }
            deltaTime = unsafe { (*tr).trDuration } as f32
                * 0.001
                * ((90.0 - (90.0 * (atTime as f32 - unsafe { (*tr).trTime } as f32) / unsafe { (*tr).trDuration } as f32))
                    * DEG2RAD_MULT)
                    .cos();
            VectorScale(unsafe { core::ptr::addr_of!((*tr).trDelta) }, deltaTime, result);
        }
        x if x == TR_GRAVITY => {
            deltaTime = (atTime as f32 - unsafe { (*tr).trTime } as f32) * 0.001; // milliseconds to seconds
            VectorCopy(unsafe { core::ptr::addr_of!((*tr).trDelta) }, result);
            unsafe { (*result)[2] -= (*g_gravity).value * deltaTime }; // DEFAULT_GRAVITY
        }
        _ => {
            Com_Error(
                ERR_DROP,
                b"EvaluateTrajectoryDelta: unknown trType: %i\0".as_ptr() as *const c_char,
                unsafe { (*tr).trTime },
            );
        }
    }
}

/*
===============
AddEventToPlayerstate

Handles the sequence numbers
===============
*/
pub unsafe extern "C" fn AddEventToPlayerstate(newEvent: c_int, eventParm: c_int, ps: *mut playerState_t) {
    unsafe {
        (*ps).events[((*ps).eventSequence & (MAX_PS_EVENTS - 1)) as usize] = newEvent;
        (*ps).eventParms[((*ps).eventSequence & (MAX_PS_EVENTS - 1)) as usize] = eventParm;
        (*ps).eventSequence += 1;
    }
}

/*
===============
CurrentPlayerstateEvent

===============
*/
pub unsafe extern "C" fn CurrentPlayerstateEvent(ps: *mut playerState_t) -> c_int {
    unsafe { (*ps).events[(((*ps).eventSequence - 1) & (MAX_PS_EVENTS - 1)) as usize] }
}

/*
========================
PlayerStateToEntityState

This is done after each set of usercmd_t on the server,
and after local prediction on the client
========================
*/
pub unsafe extern "C" fn PlayerStateToEntityState(ps: *mut playerState_t, s: *mut entityState_t) {
    let mut i: c_int;

    if unsafe { (*ps).pm_type } == PM_INTERMISSION || unsafe { (*ps).pm_type } == PM_SPECTATOR {
        unsafe { (*s).eType = ET_INVISIBLE };
    }
    /*else if ( ps->stats[STAT_HEALTH] <= GIB_HEALTH )
    {
        s->eType = ET_INVISIBLE;
    } */
    else {
        unsafe { (*s).eType = ET_PLAYER };
    }

    unsafe { (*s).number = (*ps).clientNum };

    unsafe { (*s).pos.trType = TR_INTERPOLATE };
    VectorCopy(unsafe { core::ptr::addr_of!((*ps).origin) }, unsafe { core::ptr::addr_of_mut!((*s).pos.trBase) });
    // SnapVector( s->pos.trBase );

    unsafe { (*s).apos.trType = TR_INTERPOLATE };
    VectorCopy(unsafe { core::ptr::addr_of!((*ps).viewangles) }, unsafe { core::ptr::addr_of_mut!((*s).apos.trBase) });
    // SnapVector( s->apos.trBase );

    unsafe { (*s).angles2[YAW] = (*ps).movementDir };
    unsafe { (*s).legsAnim = (*ps).legsAnim };
    unsafe { (*s).torsoAnim = (*ps).torsoAnim };
    unsafe { (*s).clientNum = (*ps).clientNum }; // ET_PLAYER looks here instead of at number
                                                   // so corpses can also reference the proper config
    unsafe { (*s).eFlags = (*ps).eFlags };

    // new sabre stuff
    // unsafe { (*s).saberActive = (*ps).SaberActive() }; // WHY is this on the entityState_t, too???
    unsafe { (*s).saberInFlight = (*ps).saberInFlight };

    // NOTE: Although we store this stuff locally on a vehicle, who's to say we
    // can't bring back these variables and fill them at the appropriate time? -Aurelio
    // We need to bring these in from the vehicle NPC.
    if unsafe {
        !(*core::ptr::addr_of!(g_entities[(*ps).clientNum as usize])).client.is_null()
            && (*(*core::ptr::addr_of!(g_entities[(*ps).clientNum as usize])).client).NPC_class == CLASS_VEHICLE
            && !(*core::ptr::addr_of!(g_entities[(*ps).clientNum as usize])).NPC.is_null()
    } {
        let pVeh: *mut Vehicle_t =
            unsafe { (*core::ptr::addr_of!(g_entities[(*ps).clientNum as usize])).m_pVehicle };
        unsafe { (*s).vehicleArmor = (*pVeh).m_iArmor };
        VectorCopy(
            unsafe { core::ptr::addr_of!((*pVeh).m_vOrientation) },
            unsafe { core::ptr::addr_of_mut!((*s).vehicleAngles) },
        );
    }

    unsafe { (*s).weapon = (*ps).weapon };
    unsafe { (*s).groundEntityNum = (*ps).groundEntityNum };

    unsafe { (*s).powerups = 0 };
    i = 0;
    while i < MAX_POWERUPS {
        if unsafe { (*ps).powerups[i as usize] != 0 } {
            unsafe { (*s).powerups |= 1 << i };
        }
        i += 1;
    }
    #[cfg(disable)]
    {
        if unsafe { (*ps).externalEvent } != 0 {
            unsafe { (*s).event = (*ps).externalEvent };
            unsafe { (*s).eventParm = (*ps).externalEventParm };
        } else {
            let mut seq: c_int;

            seq = (unsafe { (*ps).eventSequence } - 1) & (MAX_PS_EVENTS - 1);
            unsafe {
                (*s).event = (*ps).events[seq as usize] | ((unsafe { (*ps).eventSequence } & 3) << 8);
                (*s).eventParm = (*ps).eventParms[seq as usize];
            }
        }

        // show some roll in the body based on velocity and angle
        if unsafe { (*ps).stats[STAT_HEALTH] } > 0 {
            let mut right: vec3_t = [0.0; 3];
            let mut sign: f32;
            let mut side: f32;
            let mut value: f32;

            AngleVectors(unsafe { core::ptr::addr_of!((*ps).viewangles) }, core::ptr::null_mut(), core::ptr::addr_of_mut!(right), core::ptr::null_mut());

            side = DotProduct(unsafe { core::ptr::addr_of!((*ps).velocity) }, unsafe { core::ptr::addr_of!(right) });
            sign = if side < 0.0 { -1.0 } else { 1.0 };
            side = side.abs();

            value = 2.0; // g_rollangle->value;

            if side < 200.0 /* g_rollspeed->value */ {
                side = side * value / 200.0; // g_rollspeed->value;
            } else {
                side = value;
            }

            unsafe { (*s).angles[ROLL] = (side * sign * 4.0) as c_int };
        }
    }
}

/*
============
BG_PlayerTouchesItem

Items can be picked up without actually touching their physical bounds
============
*/
pub unsafe extern "C" fn BG_PlayerTouchesItem(ps: *mut playerState_t, item: *mut entityState_t, atTime: c_int) -> qboolean {
    let mut origin: vec3_t = [0.0; 3];

    EvaluateTrajectory(unsafe { core::ptr::addr_of!((*item).pos) }, atTime, core::ptr::addr_of_mut!(origin));

    // we are ignoring ducked differences here
    if unsafe { (*ps).origin[0] } - origin[0] > 44.0
        || unsafe { (*ps).origin[0] } - origin[0] < -50.0
        || unsafe { (*ps).origin[1] } - origin[1] > 36.0
        || unsafe { (*ps).origin[1] } - origin[1] < -36.0
        || unsafe { (*ps).origin[2] } - origin[2] > 36.0
        || unsafe { (*ps).origin[2] } - origin[2] < -36.0
    {
        return 0; // qfalse
    }

    1 // qtrue
}

/*
=================
BG_EmplacedView

Shared code for emplaced angle gun constriction
=================
*/
pub unsafe extern "C" fn BG_EmplacedView(
    baseAngles: *mut vec3_t,
    angles: *mut vec3_t,
    newYaw: *mut f32,
    constraint: f32,
) -> c_int {
    let mut dif: f32 = AngleSubtract(unsafe { (*baseAngles)[YAW] }, unsafe { (*angles)[YAW] });

    if dif > constraint || dif < -constraint {
        let mut amt: f32;

        if dif > constraint {
            amt = dif - constraint;
            dif = constraint;
        } else if dif < -constraint {
            amt = dif + constraint;
            dif = -constraint;
        } else {
            amt = 0.0;
        }

        unsafe { *newYaw = AngleSubtract(unsafe { (*angles)[YAW] }, -dif) };

        if amt > 1.0 || amt < -1.0 {
            // significant, force the view
            return 2;
        } else {
            // just a little out of range
            return 1;
        }
    }

    0
}
