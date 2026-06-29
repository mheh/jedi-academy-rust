// leave this line at the top for all g_xxxx.cpp files...
// include "g_headers.h"

// include "g_local.h"
// include "g_functions.h"
// include "g_items.h"
// include "wp_saber.h"

use core::ffi::{c_int, c_char};

// Local stub structures - actual definitions are in ported header files
#[repr(C)]
pub struct gentity_t([u8; 0]);

#[repr(C)]
pub struct playerState_t([u8; 0]);

#[repr(C)]
pub struct gitem_t([u8; 0]);

#[repr(C)]
pub struct cvar_t([u8; 0]);

#[repr(C)]
pub struct saberInfo_t([u8; 0]);

#[repr(C)]
pub struct entityState_s([u8; 0]);

#[repr(C)]
pub struct trajectory_t([u8; 0]);

#[repr(C)]
pub struct trace_t([u8; 0]);

#[repr(C)]
pub struct level_local_t([u8; 0]);

#[repr(C)]
pub struct cg_t([u8; 0]);

#[repr(C)]
pub struct stringID_table_t([u8; 0]);

#[repr(C)]
pub struct ammoData_t([u8; 0]);

#[repr(C)]
pub struct weaponData_t([u8; 0]);

pub type saber_colors_t = c_int;

// Constants matching C defines
const MAX_BACTA_HEAL_AMOUNT: c_int = 25;
const ITMSF_SUSPEND: c_int = 1;
const ITMSF_NOPLAYER: c_int = 2;
const ITMSF_ALLOWNPC: c_int = 4;
const ITMSF_NOTSOLID: c_int = 8;
const ITMSF_VERTICAL: c_int = 16;
const ITMSF_INVISIBLE: c_int = 32;
const ITMSF_NOGLOW: c_int = 64;
const ITMSF_USEPICKUP: c_int = 128;
const ITMSF_STATIONARY: c_int = 2048;
const MAX_ITEMS: usize = 256;

extern "C" {
    pub static mut missionInfo_Updated: c_int;
    pub static mut g_spskill: *mut cvar_t;
    pub static mut g_sex: *mut cvar_t;
    pub static mut g_saberPickuppableDroppedSabers: *mut cvar_t;
    pub static mut g_gravity: *mut cvar_t;
    pub static mut g_saber: *mut cvar_t;
    pub static mut g_timescale: *mut cvar_t;
    pub static mut level: level_local_t;
    pub static mut cg: cg_t;
    pub static mut cg_updatedDataPadForcePower1: cvar_t;
    pub static mut cg_updatedDataPadForcePower2: cvar_t;
    pub static mut cg_updatedDataPadForcePower3: cvar_t;
    pub static mut g_entities: [gentity_t; 2048];
    pub static mut ammoData: [ammoData_t; 32];
    pub static mut weaponData: [weaponData_t; 32];
    pub static mut bg_itemlist: [gitem_t; 256];
    pub static mut bg_numItems: c_int;
    pub static mut saberColorStringForColor: [*const c_char; 16];
    pub static mut delayedShutDown: c_int;

    // External functions from other modules
    pub fn CrystalAmmoSettings(ent: *mut gentity_t);
    pub fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    pub fn PM_InKnockDown(ps: *mut playerState_t) -> c_int;
    pub fn PM_InGetUp(ps: *mut playerState_t) -> c_int;
    pub fn WP_SetSaber(ent: *mut gentity_t, saberNum: c_int, saberName: *mut c_char);
    pub fn WP_RemoveSaber(ent: *mut gentity_t, saberNum: c_int);
    pub fn WP_SaberFallSound(owner: *mut gentity_t, saber: *mut gentity_t);
    pub fn TranslateSaberColor(name: *const c_char) -> saber_colors_t;
    pub fn INV_GoodieKeyGive(target: *mut gentity_t) -> c_int;
    pub fn INV_SecurityKeyGive(target: *mut gentity_t, keyname: *const c_char) -> c_int;
    pub fn G_Spawn() -> *mut gentity_t;
    pub fn G_SetOrigin(ent: *mut gentity_t, origin: *const [f32; 3]);
    pub fn G_SetAngles(ent: *mut gentity_t, angles: *const [f32; 3]);
    pub fn G_NewString(string: *const c_char) -> *mut c_char;
    pub fn FindItemForWeapon(weapon: c_int) -> *mut gitem_t;
    pub fn FinishSpawningItem(ent: *mut gentity_t);
    pub fn WP_SaberParseParms(saberType: *const c_char, saber: *mut saberInfo_t) -> c_int;
    pub fn WP_SaberInitBladeData(ent: *mut gentity_t);
    pub fn WP_SaberFreeStrings(saber: saberInfo_t);
    pub fn NPC_SetAnim(ent: *mut gentity_t, setAnimParts: c_int, anim: c_int, flags: c_int);
    pub fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int);
    pub fn G_UseTargets(ent: *mut gentity_t, other: *mut gentity_t);
    pub fn G_FreeEntity(ent: *mut gentity_t);
    pub fn BG_CanItemBeGrabbed(s: *const entityState_s, ps: *mut playerState_t) -> c_int;
    pub fn VectorCopy(in_: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorClear(v: *mut [f32; 3]);
    pub fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    pub fn VectorScale(in_: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    pub fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn VectorAdd(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    pub fn DotProduct(x: *const [f32; 3], y: *const [f32; 3]) -> f32;
    pub fn crandom() -> f32;
    pub fn random() -> f32;
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn Q_flrand(low: f32, high: f32) -> f32;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8;
    pub fn G_Error(fmt: *const c_char, ...);
    pub fn EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: *mut [f32; 3]);
    pub fn EvaluateTrajectoryDelta(tr: *const trajectory_t, atTime: c_int, result: *mut [f32; 3]);
    pub fn G_RunThink(ent: *mut gentity_t);
    pub fn pitch_roll_for_slope(ent: *mut gentity_t, normal: *const [f32; 3], angles: *mut [f32; 3], useTemp: c_int);
    pub fn G_SetSabersFromCVars(ent: *mut gentity_t);
    pub fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int);
    pub fn GEntity_TouchFunc(ent: *mut gentity_t, other: *mut gentity_t, trace_: *mut trace_t);
    pub fn G_BoundsOverlap(mins1: *const [f32; 3], maxs1: *const [f32; 3], mins2: *const [f32; 3], maxs2: *const [f32; 3]) -> c_int;
    pub fn FindItemForInventory(inv: c_int) -> *mut gitem_t;
    pub fn Player_CacheFromPrevLevel();
    pub fn GetIDForString(table: *mut stringID_table_t, str: *const c_char) -> c_int;
    pub fn CG_ChangeWeapon(num: c_int);
    pub fn G_CreateG2AttachedWeaponModel(ent: *mut gentity_t, model: *const c_char, bolt: c_int, unused: c_int);
    pub fn WP_SaberAddG2SaberModels(ent: *mut gentity_t);
    pub fn G_ModelIndex(name: *const c_char) -> c_int;
    pub fn G_SpawnFloat(key: *const c_char, defaultString: *const c_char, out: *mut f32);
    pub fn va(fmt: *const c_char, ...) -> *mut c_char;
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, sound: *mut c_char);

    // Game interface (gi) syscalls
    pub fn gi_linkentity(ent: *mut gentity_t);
    pub fn gi_unlinkentity(ent: *mut gentity_t);
    pub fn gi_pointcontents(p: *const [f32; 3], passEntityNum: c_int) -> c_int;
    pub fn gi_trace(results: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passEntityNum: c_int, contentmask: c_int);
    pub fn gi_SetConfigstring(index: c_int, val: *const c_char);
    pub fn gi_Printf(fmt: *const c_char, ...);
    pub fn gi_SendServerCommand(clientNum: *mut c_char, fmt: *const c_char, ...);
    pub fn gi_cvar_set(var_name: *const c_char, value: *const c_char);
    pub fn gi_S_StartSound(origin: *const [f32; 3], entitynum: c_int, entchannel: c_int, sfx: c_int);
    pub fn gi_S_RegisterSound(name: *const c_char) -> c_int;
    pub fn gi_G2API_InitGhoul2Model(ghoul2: *mut *mut c_char, modelName: *const c_char, modelIndex: c_int, ...);
    pub fn CG_ItemPickup(itemNum: c_int, bHadItem: c_int);
}

/*

  Items are any object that a player can touch to gain some effect.

  Pickup will return the number of seconds until they should respawn.

  all items should pop when dropped in lava or slime

  Respawnable items don't actually go away when picked up, they are
  just made invisible and untouchable.  This allows them to ride
  movers and respawn apropriately.
*/

//======================================================================

/*
===============
G_InventorySelectable
===============
*/
#[no_mangle]
pub extern "C" fn G_InventorySelectable(index: c_int, other: *mut gentity_t) -> c_int {
    unsafe {
        // Stub implementation - actual structure layout unknown
        0
    }
}

#[no_mangle]
pub extern "C" fn Pickup_Holdable(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    unsafe {
        // Stub implementation - references unknown struct members
        60
    }
}

// ======================================================================
#[no_mangle]
pub extern "C" fn Add_Ammo2(ent: *mut gentity_t, ammoType: c_int, count: c_int) -> c_int {
    unsafe {
        // Stub implementation - references unknown struct members
        1 // qtrue
    }
}

// -------------------------------------------------------
#[no_mangle]
pub extern "C" fn Add_Ammo(ent: *mut gentity_t, weapon: c_int, count: c_int) {
    unsafe {
        // Stub implementation
    }
}

// -------------------------------------------------------
#[no_mangle]
pub extern "C" fn Pickup_Ammo(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    unsafe {
        30
    }
}

// ======================================================================
#[no_mangle]
pub extern "C" fn Add_Batteries(ent: *mut gentity_t, count: *mut c_int) {
    unsafe {
        // Stub implementation
    }
}

// -------------------------------------------------------
#[no_mangle]
pub extern "C" fn Pickup_Battery(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    unsafe {
        30
    }
}

// ======================================================================

#[no_mangle]
pub extern "C" fn G_CopySaberItemValues(pickUpSaber: *mut gentity_t, oldSaber: *mut gentity_t) {
    unsafe {
        // Stub implementation
    }
}

#[no_mangle]
pub extern "C" fn G_DropSaberItem(
    saberType: *const c_char,
    saberColor: saber_colors_t,
    saberPos: *const [f32; 3],
    saberVel: *mut f32,
    saberAngles: *const [f32; 3],
    copySaber: *mut gentity_t,
) -> *mut gentity_t {
    unsafe {
        // turn it into a pick-uppable item!
        // Stub implementation
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn Pickup_Saber(
    self_: *mut gentity_t,
    hadSaber: c_int,
    pickUpSaber: *mut gentity_t,
) -> c_int {
    unsafe {
        // NOTE: loopAnim = saberSolo, alt_fire = saberLeftHand, NPC_type = saberType, NPC_targetname = saberColor
        // Stub implementation
        0
    }
}

#[no_mangle]
pub extern "C" fn Pickup_Weapon(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    unsafe {
        // Stub implementation
        5
    }
}

// ======================================================================

#[no_mangle]
pub extern "C" fn ITM_AddHealth(ent: *mut gentity_t, count: c_int) -> c_int {
    unsafe {
        1 // qtrue
    }
}

#[no_mangle]
pub extern "C" fn Pickup_Health(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    unsafe {
        30
    }
}

// ======================================================================

#[no_mangle]
pub extern "C" fn ITM_AddArmor(ent: *mut gentity_t, count: c_int) -> c_int {
    unsafe {
        1 // qtrue
    }
}

#[no_mangle]
pub extern "C" fn Pickup_Armor(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    unsafe {
        30
    }
}

// ======================================================================

#[no_mangle]
pub extern "C" fn Pickup_Holocron(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    unsafe {
        1
    }
}

// ======================================================================

/*
===============
RespawnItem
===============
*/
#[no_mangle]
pub extern "C" fn RespawnItem(_ent: *mut gentity_t) {}

#[no_mangle]
pub extern "C" fn CheckItemCanBePickedUpByNPC(item: *mut gentity_t, pickerupper: *mut gentity_t) -> c_int {
    unsafe {
        0 // qfalse
    }
}

#[no_mangle]
pub extern "C" fn G_CanPickUpWeapons(other: *mut gentity_t) -> c_int {
    unsafe {
        1 // qtrue
    }
}

/*
===============
Touch_Item
===============
*/
#[no_mangle]
pub extern "C" fn Touch_Item(ent: *mut gentity_t, other: *mut gentity_t, trace_: *mut trace_t) {
    unsafe {
        // Stub implementation
    }
}

// ======================================================================

/*
================
LaunchItem

Spawns an item and tosses it forward
================
*/
#[no_mangle]
pub extern "C" fn LaunchItem(
    item: *mut gitem_t,
    origin: *const [f32; 3],
    velocity: *const [f32; 3],
    target: *mut c_char,
) -> *mut gentity_t {
    unsafe {
        // Stub implementation
        core::ptr::null_mut()
    }
}

/*
================
Drop_Item

Spawns an item and tosses it forward
================
*/
#[no_mangle]
pub extern "C" fn Drop_Item(
    ent: *mut gentity_t,
    item: *mut gitem_t,
    angle: f32,
    copytarget: c_int,
) -> *mut gentity_t {
    unsafe {
        // Stub implementation
        core::ptr::null_mut()
    }
}

/*
================
Use_Item

Respawn the item
================
*/
#[no_mangle]
pub extern "C" fn Use_Item(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    unsafe {
        // Stub implementation
    }
}

// ======================================================================

/*
================
FinishSpawningItem

Traces down to find where an item should rest, instead of letting them
free fall from their spawn points
================
*/
#[no_mangle]
pub extern "C" fn FinishSpawningItem(ent: *mut gentity_t) {
    unsafe {
        // Stub implementation
    }
}

static mut itemRegistered: [u8; MAX_ITEMS + 1] = [0u8; MAX_ITEMS + 1];

/*
==============
ClearRegisteredItems
==============
*/
#[no_mangle]
pub extern "C" fn ClearRegisteredItems() {
    unsafe {
        memset(itemRegistered.as_mut_ptr(), '0' as c_int, bg_numItems as usize);
        itemRegistered[bg_numItems as usize] = 0;

        // these are given in g_client, ClientSpawn(), but MUST be registered HERE, BEFORE cgame starts.
        // RegisterItem( FindItemForWeapon( WP_NONE ) );	//has no item
        RegisterItem(FindItemForInventory(5)); // INV_ELECTROBINOCULARS
        // RegisterItem( FindItemForInventory( INV_BACTA_CANISTER ));
        // saber or baton is cached in SP_info_player_deathmatch now.

        Player_CacheFromPrevLevel(); // reads from transition carry-over;
    }
}

/*
===============
RegisterItem

The item will be added to the precache list
===============
*/
#[no_mangle]
pub extern "C" fn RegisterItem(item: *mut gitem_t) {
    unsafe {
        if item.is_null() {
            G_Error("RegisterItem: NULL\0" as *const c_char);
        }
        // itemRegistered[ item - bg_itemlist ] = '1';
        // gi.SetConfigstring(CS_ITEMS, itemRegistered);	//Write the needed items to a config string
        gi_SetConfigstring(0, itemRegistered.as_ptr() as *const c_char); // CS_ITEMS
    }
}

/*
===============
SaveRegisteredItems

Write the needed items to a config string
so the client will know which ones to precache
===============
*/
#[no_mangle]
pub extern "C" fn SaveRegisteredItems() {
    /*	char	string[MAX_ITEMS+1];
    int		i;
    int		count;

    count = 0;
    for ( i = 0 ; i < bg_numItems ; i++ ) {
        if ( itemRegistered[i] ) {
            count++;
            string[i] = '1';
        } else {
            string[i] = '0';
        }
    }
    string[ bg_numItems ] = 0;

    gi.Printf( "%i items registered\n", count );
    gi.SetConfigstring(CS_ITEMS, string);
*/
    unsafe {
        gi_SetConfigstring(0, itemRegistered.as_ptr() as *const c_char); // CS_ITEMS
    }
}

/*
============
item_spawn_use

 if an item is given a targetname, it will be spawned in when used
============
*/
#[no_mangle]
pub extern "C" fn item_spawn_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
) {
    unsafe {
        // Stub implementation
    }
}

/*
============
G_SpawnItem

Sets the clipping size and plants the object on the floor.

Items can't be immediately dropped to floor, because they might
be on an entity that hasn't spawned yet.
============
*/
#[no_mangle]
pub extern "C" fn G_SpawnItem(ent: *mut gentity_t, item: *mut gitem_t) {
    unsafe {
        // Stub implementation
    }
}

/*
================
G_BounceItem

================
*/
#[no_mangle]
pub extern "C" fn G_BounceItem(ent: *mut gentity_t, trace_: *mut trace_t) {
    unsafe {
        // Stub implementation
    }
}

/*
================
G_RunItem

================
*/
#[no_mangle]
pub extern "C" fn G_RunItem(ent: *mut gentity_t) {
    unsafe {
        // Stub implementation
    }
}

/*
================
ItemUse_Bacta

================
*/
#[no_mangle]
pub extern "C" fn ItemUse_Bacta(ent: *mut gentity_t) {
    unsafe {
        // Stub implementation
    }
}
