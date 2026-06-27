//! `bg_vehicleLoad.c` — the vehicle (`.veh`) and vehicle-weapon (`.vwp`) definition
//! parser, shared by game and client. Reads the asset text into `g_vehicleInfo[]` /
//! `g_vehWeaponInfo[]` via the field-descriptor tables below.
//!
//! **Build config resolved to the retail MP game module** (`_JK2MP` defined, `QAGAME`
//! defined by the build, `CGAME`/UI undefined): the SP-only `Ratl/string_vs.h` and the
//! UI-only `ui_local.h` includes are `#ifdef`-excluded; the `QAGAME` branches select
//! `G_ModelIndex`/`G_SoundIndex`/`G_EffectIndex` (all ported in `g_utils`) over the
//! cgame `trap_R_Register*`/`trap_S_RegisterSound`/`trap_FX_RegisterEffect` traps.
//!
//! **Status: data layer + the `.vwp` *vehicle-weapon* parse path.** This file ports the
//! self-contained foundation — the data buffers, the `g_veh*Info` arrays, the `BG_Clear*`
//! resets, the `vehFieldType_t`/`vehField_t` field-descriptor types, both descriptor tables
//! (`vehWeaponFields` and the 175-row `vehicleFields`, whose ~64 nested-array rows use the
//! `weap_ofs`/`turret_ofs` offset helpers and are element-wise oracle-verified), the
//! `VehicleTable` string-ID table, and the `.vwp` file concatenator `BG_VehWeaponLoadParms`
//! — **plus** the now-landed `.vwp` weapon parse path: `BG_ParseVehWeaponParm` (:176),
//! `VEH_LoadVehWeapon` (:318), and `VEH_VehWeaponIndexForName` (:422).
//!
//! Also landed: the standalone `.veh` helpers `BG_VehicleSetDefaults` (:718),
//! `BG_VehicleClampData` (:821), and the `.veh` field parser `BG_ParseVehicleParm` (:848,
//! decode-identical to `BG_ParseVehWeaponParm`, preserving the C `VF_VECTOR` offset bug).
//!
//! Also landed: the self-contained ghoul2 leaf `AttachRidersGeneric` (:1651) — it snaps the
//! pilot onto the parent's `*driver` bolt and only touches already-ported ghoul2 traps +
//! `BG_GetTime`/`BG_GiveMeVectorFromMatrix`, so it is not actually blocked on the vehicle
//! load chain (despite being grouped with it in the C file).
//!
//! Also landed (cycle 97): the `.veh` *vehicle* load/lookup chain —
//! `BG_SetSharedVehicleFunctions` (the keystone, now that `G_SetSharedVehicleFunctions` and
//! the four per-type `G_Set{Speeder,Animal,Fighter,Walker}VehicleFunctions` setters exist),
//! `BG_VehicleLoadParms`, `VEH_LoadVehicle`, `VEH_VehicleIndexForName`, `BG_VehicleGetIndex`,
//! and `BG_GetVehicleModelName`/`BG_GetVehicleSkinName`. The file is now complete.

#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use crate::codemp::cgame::animtable::animTable;
use crate::codemp::game::bg_lib::sscanf;
use crate::codemp::game::bg_vehicles_h::{
    turretStats_t, vehWeaponInfo_t, vehWeaponStats_t, vehicleInfo_t, vehicleType_t, MAX_VEHICLES,
    MAX_VEH_WEAPONS, NUM_VWEAP_PARMS, VEHICLE_BASE, VEHICLE_NONE, VEH_MAX_PASSENGERS,
    VEH_WEAPON_BASE, VEH_WEAPON_NONE, VH_ANIMAL, VH_FIGHTER, VH_FLIER, VH_NONE, VH_NUM_VEHICLES,
    VH_SPEEDER, VH_WALKER,
};
use crate::codemp::game::g_vehicles::G_SetSharedVehicleFunctions;
use crate::codemp::game::speedernpc::G_SetSpeederVehicleFunctions;
use crate::codemp::game::animalnpc::G_SetAnimalVehicleFunctions;
use crate::codemp::game::fighternpc::G_SetFighterVehicleFunctions;
use crate::codemp::game::walkernpc::G_SetWalkerVehicleFunctions;
use crate::codemp::game::bg_misc::{BG_Alloc, BG_TempAlloc, BG_TempFree};
use crate::codemp::game::g_main::{Com_Error, Com_Printf};
use crate::codemp::game::g_utils::{G_EffectIndex, G_ModelIndex, G_SoundIndex};
use crate::codemp::game::q_shared::{
    GetIDForString, Q_strncpyz, Q_stricmp, Sz, COM_BeginParseSession, COM_ParseExt,
    SkipBracedSection, SkipRestOfLine,
};
use crate::codemp::game::q_shared_h::{
    mdxaBone_t, qboolean, stringID_table_t, vec3_t, ERR_DROP, FS_READ, ORIGIN, YAW,
};
use crate::codemp::game::bg_public::{bgEntity_t, BG_GiveMeVectorFromMatrix};
use crate::codemp::game::bg_vehicles_h::Vehicle_t;
use crate::codemp::game::g_main::BG_GetTime;
use crate::codemp::game::q_math::VectorSet;
use crate::ffi::types::{QFALSE, QTRUE};
use crate::trap;
use core::ffi::{c_char, c_int, CStr};
#[cfg(feature = "vm")]
use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr::{addr_of, addr_of_mut};

// C `strlen`/`strcat`/`strcpy`/`atoi`/`atof` (libc): the `_JK2MP`/non-`Q3_VM` build
// links these from the CRT rather than the `Q3_VM` bg_lib shims, so they come in via
// `extern "C"` (mirrors `bg_misc.rs`'s `BG_ParseField`) instead of the
// `#[cfg(feature = "vm")]`-gated copies.
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atoi(s: *const c_char) -> c_int;
    fn atof(s: *const c_char) -> f64;
}

// These buffers are filled in with the same contents and then just read from in
// a few places. We only need one copy on Xbox.
/// `MAX_VEH_WEAPON_DATA_SIZE` (bg_vehicleLoad.c).
pub const MAX_VEH_WEAPON_DATA_SIZE: usize = 0x20000;
/// `MAX_VEHICLE_DATA_SIZE` (bg_vehicleLoad.c).
pub const MAX_VEHICLE_DATA_SIZE: usize = 0x80000;

// `#if !defined(_XBOX) || defined(QAGAME)` -> QAGAME, so these are defined (not extern).
pub static mut VehWeaponParms: [c_char; MAX_VEH_WEAPON_DATA_SIZE] = [0; MAX_VEH_WEAPON_DATA_SIZE];
pub static mut VehicleParms: [c_char; MAX_VEHICLE_DATA_SIZE] = [0; MAX_VEHICLE_DATA_SIZE];

/// `BG_ClearVehicleParseParms` (bg_vehicleLoad.c).
pub fn BG_ClearVehicleParseParms() {
    //You can't strcat to these forever without clearing them!
    unsafe {
        *(addr_of_mut!(VehWeaponParms) as *mut c_char) = 0;
        *(addr_of_mut!(VehicleParms) as *mut c_char) = 0;
    }
}

pub static mut g_vehWeaponInfo: [vehWeaponInfo_t; MAX_VEH_WEAPONS] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
pub static mut numVehicleWeapons: c_int = 1; //first one is null/default

pub static mut g_vehicleInfo: [vehicleInfo_t; MAX_VEHICLES] =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
pub static mut numVehicles: c_int = 0; //first one is null/default

// NOTE: original Raven (Xbox) `bg_vehicleLoad.c` defined `BG_ClearVehicleLoadInfo`
// here, but the PC source removed it entirely (no in-module caller; the engine
// cleared this state itself). The PC re-port therefore drops it.

/// `vehFieldType_t` (bg_vehicleLoad.c) — how a parsed `.veh`/`.vwp` value is stored.
pub type vehFieldType_t = c_int;
pub const VF_IGNORE: vehFieldType_t = 0;
pub const VF_INT: vehFieldType_t = 1;
pub const VF_FLOAT: vehFieldType_t = 2;
pub const VF_LSTRING: vehFieldType_t = 3; // string on disk, pointer in memory, TAG_LEVEL
pub const VF_VECTOR: vehFieldType_t = 4;
pub const VF_BOOL: vehFieldType_t = 5;
pub const VF_VEHTYPE: vehFieldType_t = 6;
pub const VF_ANIM: vehFieldType_t = 7;
pub const VF_WEAPON: vehFieldType_t = 8; // take string, resolve into index into VehWeaponParms
pub const VF_MODEL: vehFieldType_t = 9; // take the string, get the G_ModelIndex
pub const VF_MODEL_CLIENT: vehFieldType_t = 10; // (cgame only) take the string, get the G_ModelIndex
pub const VF_EFFECT: vehFieldType_t = 11; // take the string, get the G_EffectIndex
pub const VF_EFFECT_CLIENT: vehFieldType_t = 12; // (cgame only) take the string, get the index
pub const VF_SHADER: vehFieldType_t = 13; // (cgame only) take the string, call trap_R_RegisterShader
pub const VF_SHADER_NOMIP: vehFieldType_t = 14; // (cgame only) take the string, call trap_R_RegisterShaderNoMip
pub const VF_SOUND: vehFieldType_t = 15; // take the string, get the G_SoundIndex
pub const VF_SOUND_CLIENT: vehFieldType_t = 16; // (cgame only) take the string, get the index

/// `vehField_t` (bg_vehicleLoad.c) — one field-descriptor row: parm name, byte offset
/// into the target struct, and how to interpret the value. Pointer-bearing (`name`);
/// `type` is a Rust keyword -> `r#type`. (`char *name` -> `*const c_char`, the
/// `cvarTable_t` precedent: the table never writes through these name pointers.)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct vehField_t {
    pub name: *const c_char,
    pub ofs: c_int,
    pub r#type: vehFieldType_t,
}
const _: () = assert!(core::mem::offset_of!(vehField_t, name) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<vehField_t>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehField_t, ofs) == 8);

/// Builds one `vehField_t` row, mirroring the C `{ "name", VWFOFS(field), VF_* }`
/// aggregate (the `cv()`/`wd()` precedent). The `VWFOFS`/`VFOFS` offsetof macros
/// become `offset_of!`; the offsets are correct by construction since the target
/// struct layouts are C-oracle-verified (see `bg_vehicles_h`).
const fn vf(name: &'static CStr, ofs: usize, r#type: vehFieldType_t) -> vehField_t {
    vehField_t {
        name: name.as_ptr(),
        ofs: ofs as c_int,
        r#type,
    }
}

/// `vehWeaponFields[NUM_VWEAP_PARMS]` (bg_vehicleLoad.c) — the `.vwp` field-parser
/// table for `vehWeaponInfo_t` (all flat fields). Read-only descriptors, but the C is
/// a mutable global and `vehField_t` carries a raw pointer (`!Sync`), so `static mut`.
pub static mut vehWeaponFields: [vehField_t; NUM_VWEAP_PARMS as usize] = [
    vf(c"name", offset_of!(vehWeaponInfo_t, name), VF_LSTRING), //unique name of the vehicle
    vf(c"projectile", offset_of!(vehWeaponInfo_t, bIsProjectile), VF_BOOL), //traceline or entity?
    vf(c"hasGravity", offset_of!(vehWeaponInfo_t, bHasGravity), VF_BOOL), //if a projectile, drops
    vf(c"ionWeapon", offset_of!(vehWeaponInfo_t, bIonWeapon), VF_BOOL), //disables ship shields and sends them out of control
    vf(c"saberBlockable", offset_of!(vehWeaponInfo_t, bSaberBlockable), VF_BOOL), //lightsabers can deflect this projectile
    vf(c"muzzleFX", offset_of!(vehWeaponInfo_t, iMuzzleFX), VF_EFFECT_CLIENT), //index of Muzzle Effect
    vf(c"model", offset_of!(vehWeaponInfo_t, iModel), VF_MODEL_CLIENT), //handle to the model used by this projectile
    vf(c"shotFX", offset_of!(vehWeaponInfo_t, iShotFX), VF_EFFECT_CLIENT), //index of Shot Effect
    vf(c"impactFX", offset_of!(vehWeaponInfo_t, iImpactFX), VF_EFFECT_CLIENT), //index of Impact Effect
    vf(c"g2MarkShader", offset_of!(vehWeaponInfo_t, iG2MarkShaderHandle), VF_SHADER), //index of shader to use for G2 marks made on other models when hit by this projectile
    vf(c"g2MarkSize", offset_of!(vehWeaponInfo_t, fG2MarkSize), VF_FLOAT), //size (diameter) of the ghoul2 mark
    vf(c"loopSound", offset_of!(vehWeaponInfo_t, iLoopSound), VF_SOUND_CLIENT), //index of loopSound
    vf(c"speed", offset_of!(vehWeaponInfo_t, fSpeed), VF_FLOAT), //speed of projectile/range of traceline
    vf(c"homing", offset_of!(vehWeaponInfo_t, fHoming), VF_FLOAT), //0.0 = not homing, 0.5 = half vel to targ, half cur vel, 1.0 = all vel to targ
    vf(c"homingFOV", offset_of!(vehWeaponInfo_t, fHomingFOV), VF_FLOAT), //missile will lose lock on if DotProduct of missile direction and direction to target ever drops below this (-1 to 1, -1 = never lose target, 0 = lose if ship gets behind missile, 1 = pretty much will lose it's target right away)
    vf(c"lockOnTime", offset_of!(vehWeaponInfo_t, iLockOnTime), VF_INT), //0 = no lock time needed, else # of ms needed to lock on
    vf(c"damage", offset_of!(vehWeaponInfo_t, iDamage), VF_INT), //damage done when traceline or projectile directly hits target
    vf(c"splashDamage", offset_of!(vehWeaponInfo_t, iSplashDamage), VF_INT), //damage done to ents in splashRadius of end of traceline or projectile origin on impact
    vf(c"splashRadius", offset_of!(vehWeaponInfo_t, fSplashRadius), VF_FLOAT), //radius that ent must be in to take splashDamage (linear fall-off)
    vf(c"ammoPerShot", offset_of!(vehWeaponInfo_t, iAmmoPerShot), VF_INT), //how much "ammo" each shot takes
    vf(c"health", offset_of!(vehWeaponInfo_t, iHealth), VF_INT), //if non-zero, projectile can be shot, takes this much damage before being destroyed
    vf(c"width", offset_of!(vehWeaponInfo_t, fWidth), VF_FLOAT), //width of traceline or bounding box of projecile (non-rotating!)
    vf(c"height", offset_of!(vehWeaponInfo_t, fHeight), VF_FLOAT), //height of traceline or bounding box of projecile (non-rotating!)
    vf(c"lifetime", offset_of!(vehWeaponInfo_t, iLifeTime), VF_INT), //removes itself after this amount of time
    vf(c"explodeOnExpire", offset_of!(vehWeaponInfo_t, bExplodeOnExpire), VF_BOOL), //when iLifeTime is up, explodes rather than simply removing itself
];

// The C `VFOFS(x)` offsetof macro for nested-array fields (`weapon[i].field`,
// `weapMuzzle[i]`, `turret[i].field[j]`) expands to `offsetof(vehicleInfo_t,
// weapon[i].field)`, which `offset_of!` cannot spell directly. These const helpers
// rebuild that address arithmetic from the array-base offset, the element stride, and
// the in-element field offset -- all correct by construction over the C-oracle-verified
// `vehicleInfo_t`/`vehWeaponStats_t`/`turretStats_t` layouts (and the whole offset
// column is element-wise oracle-verified below; see the test).

/// `VFOFS(weapon[i].field)` -- `weapon` is `[vehWeaponStats_t; MAX_VEHICLE_WEAPONS]`.
const fn weap_ofs(i: usize, field_ofs: usize) -> usize {
    offset_of!(vehicleInfo_t, weapon) + i * size_of::<vehWeaponStats_t>() + field_ofs
}
/// `VFOFS(weapMuzzle[i])` -- `weapMuzzle` is `[c_int; MAX_VEHICLE_MUZZLES]`.
const fn muzzle_ofs(i: usize) -> usize {
    offset_of!(vehicleInfo_t, weapMuzzle) + i * size_of::<c_int>()
}
/// `VFOFS(turret[i].field)` -- `turret` is `[turretStats_t; MAX_VEHICLE_TURRETS]`.
const fn turret_ofs(i: usize, field_ofs: usize) -> usize {
    offset_of!(vehicleInfo_t, turret) + i * size_of::<turretStats_t>() + field_ofs
}
/// `VFOFS(turret[i].iMuzzle[j])` -- `iMuzzle` is a `[c_int; _]` inside `turretStats_t`.
const fn turret_muzzle_ofs(i: usize, j: usize) -> usize {
    turret_ofs(i, offset_of!(turretStats_t, iMuzzle)) + j * size_of::<c_int>()
}

/// The terminating `{0, -1, VF_INT}` row: a NULL name and an `ofs` of -1.
const VEHFIELD_TERMINATOR: vehField_t = vehField_t {
    name: core::ptr::null(),
    ofs: -1,
    r#type: VF_INT,
};

/// `vehicleFields[]` (bg_vehicleLoad.c:454) -- the `.veh` field-parser table for
/// `vehicleInfo_t`, in the `_JK2MP` (retail MP) configuration: the `#ifdef _JK2MP`
/// shader-icon / area-health / wing-FX rows are included; the `#else` SP-only rows
/// (`radarIcon` `VF_IGNORE`, `armorLowFX`/`armorGoneFX`) are excluded. Read-only
/// descriptors, but the C is a mutable global and `vehField_t` carries a raw pointer
/// (`!Sync`), so `static mut` (like `vehWeaponFields`).
pub static mut vehicleFields: [vehField_t; 175] = [
    vf(c"name", offset_of!(vehicleInfo_t, name), VF_LSTRING), //unique name of the vehicle
    //general data
    vf(c"type", offset_of!(vehicleInfo_t, r#type), VF_VEHTYPE), //what kind of vehicle
    vf(c"numHands", offset_of!(vehicleInfo_t, numHands), VF_INT), //if 2 hands, no weapons, if 1 hand, can use 1-handed weapons, if 0 hands, can use 2-handed weapons
    vf(c"lookPitch", offset_of!(vehicleInfo_t, lookPitch), VF_FLOAT), //How far you can look up and down off the forward of the vehicle
    vf(c"lookYaw", offset_of!(vehicleInfo_t, lookYaw), VF_FLOAT), //How far you can look left and right off the forward of the vehicle
    vf(c"length", offset_of!(vehicleInfo_t, length), VF_FLOAT), //how long it is - used for body length traces when turning/moving?
    vf(c"width", offset_of!(vehicleInfo_t, width), VF_FLOAT), //how wide it is - used for body length traces when turning/moving?
    vf(c"height", offset_of!(vehicleInfo_t, height), VF_FLOAT), //how tall it is - used for body length traces when turning/moving?
    vf(c"centerOfGravity", offset_of!(vehicleInfo_t, centerOfGravity), VF_VECTOR), //offset from origin: {forward, right, up} as a modifier on that dimension (-1.0f is all the way back, 1.0f is all the way forward)
    //speed stats
    vf(c"speedMax", offset_of!(vehicleInfo_t, speedMax), VF_FLOAT), //top speed
    vf(c"turboSpeed", offset_of!(vehicleInfo_t, turboSpeed), VF_FLOAT), //turbo speed
    vf(c"speedMin", offset_of!(vehicleInfo_t, speedMin), VF_FLOAT), //if < 0, can go in reverse
    vf(c"speedIdle", offset_of!(vehicleInfo_t, speedIdle), VF_FLOAT), //what speed it drifts to when no accel/decel input is given
    vf(c"accelIdle", offset_of!(vehicleInfo_t, accelIdle), VF_FLOAT), //if speedIdle > 0, how quickly it goes up to that speed
    vf(c"acceleration", offset_of!(vehicleInfo_t, acceleration), VF_FLOAT), //when pressing on accelerator
    vf(c"decelIdle", offset_of!(vehicleInfo_t, decelIdle), VF_FLOAT), //when giving no input, how quickly it drops to speedIdle
    vf(c"throttleSticks", offset_of!(vehicleInfo_t, throttleSticks), VF_BOOL), //if true, speed stays at whatever you accel/decel to, unless you turbo or brake
    vf(c"strafePerc", offset_of!(vehicleInfo_t, strafePerc), VF_FLOAT), //multiplier on current speed for strafing.  If 1.0f, you can strafe at the same speed as you're going forward, 0.5 is half, 0 is no strafing
    //handling stats
    vf(c"bankingSpeed", offset_of!(vehicleInfo_t, bankingSpeed), VF_FLOAT), //how quickly it pitches and rolls (not under player control)
    vf(c"pitchLimit", offset_of!(vehicleInfo_t, pitchLimit), VF_FLOAT), //how far it can roll forward or backward
    vf(c"rollLimit", offset_of!(vehicleInfo_t, rollLimit), VF_FLOAT), //how far it can roll to either side
    vf(c"braking", offset_of!(vehicleInfo_t, braking), VF_FLOAT), //when pressing on decelerator
    vf(c"mouseYaw", offset_of!(vehicleInfo_t, mouseYaw), VF_FLOAT), // The mouse yaw override.
    vf(c"mousePitch", offset_of!(vehicleInfo_t, mousePitch), VF_FLOAT), // The mouse yaw override.
    vf(c"turningSpeed", offset_of!(vehicleInfo_t, turningSpeed), VF_FLOAT), //how quickly you can turn
    vf(c"turnWhenStopped", offset_of!(vehicleInfo_t, turnWhenStopped), VF_BOOL), //whether or not you can turn when not moving
    vf(c"traction", offset_of!(vehicleInfo_t, traction), VF_FLOAT), //how much your command input affects velocity
    vf(c"friction", offset_of!(vehicleInfo_t, friction), VF_FLOAT), //how much velocity is cut on its own
    vf(c"maxSlope", offset_of!(vehicleInfo_t, maxSlope), VF_FLOAT), //the max slope that it can go up with control
    vf(c"speedDependantTurning", offset_of!(vehicleInfo_t, speedDependantTurning), VF_BOOL), //vehicle turns faster the faster it's going
    //durability stats
    vf(c"mass", offset_of!(vehicleInfo_t, mass), VF_INT), //for momentum and impact force (player mass is 10)
    vf(c"armor", offset_of!(vehicleInfo_t, armor), VF_INT), //total points of damage it can take
    vf(c"shields", offset_of!(vehicleInfo_t, shields), VF_INT), //energy shield damage points
    vf(c"shieldRechargeMS", offset_of!(vehicleInfo_t, shieldRechargeMS), VF_INT), //energy shield milliseconds per point recharged
    vf(c"toughness", offset_of!(vehicleInfo_t, toughness), VF_FLOAT), //modifies incoming damage, 1.0 is normal, 0.5 is half, etc.  Simulates being made of tougher materials/construction
    vf(c"malfunctionArmorLevel", offset_of!(vehicleInfo_t, malfunctionArmorLevel), VF_INT), //when armor drops to or below this point, start malfunctioning
    vf(c"surfDestruction", offset_of!(vehicleInfo_t, surfDestruction), VF_INT),
    //visuals & sounds
    vf(c"model", offset_of!(vehicleInfo_t, model), VF_LSTRING), //what model to use - if make it an NPC's primary model, don't need this?
    vf(c"skin", offset_of!(vehicleInfo_t, skin), VF_LSTRING), //what skin to use - if make it an NPC's primary model, don't need this?
    vf(c"g2radius", offset_of!(vehicleInfo_t, g2radius), VF_INT), //render radius (really diameter, but...) for the ghoul2 model
    vf(c"riderAnim", offset_of!(vehicleInfo_t, riderAnim), VF_ANIM), //what animation the rider uses
    vf(c"droidNPC", offset_of!(vehicleInfo_t, droidNPC), VF_LSTRING), //NPC to attach to *droidunit tag (if it exists in the model)
    // #ifdef _JK2MP
    vf(c"radarIcon", offset_of!(vehicleInfo_t, radarIconHandle), VF_SHADER_NOMIP), //what icon to show on radar in MP
    vf(c"dmgIndicFrame", offset_of!(vehicleInfo_t, dmgIndicFrameHandle), VF_SHADER_NOMIP), //what image to use for the frame of the damage indicator
    vf(c"dmgIndicShield", offset_of!(vehicleInfo_t, dmgIndicShieldHandle), VF_SHADER_NOMIP), //what image to use for the shield of the damage indicator
    vf(c"dmgIndicBackground", offset_of!(vehicleInfo_t, dmgIndicBackgroundHandle), VF_SHADER_NOMIP), //what image to use for the background of the damage indicator
    vf(c"icon_front", offset_of!(vehicleInfo_t, iconFrontHandle), VF_SHADER_NOMIP), //what image to use for the front of the ship on the damage indicator
    vf(c"icon_back", offset_of!(vehicleInfo_t, iconBackHandle), VF_SHADER_NOMIP), //what image to use for the back of the ship on the damage indicator
    vf(c"icon_right", offset_of!(vehicleInfo_t, iconRightHandle), VF_SHADER_NOMIP), //what image to use for the right of the ship on the damage indicator
    vf(c"icon_left", offset_of!(vehicleInfo_t, iconLeftHandle), VF_SHADER_NOMIP), //what image to use for the left of the ship on the damage indicator
    vf(c"crosshairShader", offset_of!(vehicleInfo_t, crosshairShaderHandle), VF_SHADER_NOMIP), //what image to use as the crosshair
    vf(c"shieldShader", offset_of!(vehicleInfo_t, shieldShaderHandle), VF_SHADER), //What shader to use when drawing the shield shell
    //individual "area" health -rww
    vf(c"health_front", offset_of!(vehicleInfo_t, health_front), VF_INT),
    vf(c"health_back", offset_of!(vehicleInfo_t, health_back), VF_INT),
    vf(c"health_right", offset_of!(vehicleInfo_t, health_right), VF_INT),
    vf(c"health_left", offset_of!(vehicleInfo_t, health_left), VF_INT),
    // #endif
    vf(c"soundOn", offset_of!(vehicleInfo_t, soundOn), VF_SOUND), //sound to play when get on it
    vf(c"soundOff", offset_of!(vehicleInfo_t, soundOff), VF_SOUND), //sound to play when get off
    vf(c"soundLoop", offset_of!(vehicleInfo_t, soundLoop), VF_SOUND), //sound to loop while riding it
    vf(c"soundTakeOff", offset_of!(vehicleInfo_t, soundTakeOff), VF_SOUND), //sound to play when ship takes off
    vf(c"soundEngineStart", offset_of!(vehicleInfo_t, soundEngineStart), VF_SOUND_CLIENT), //sound to play when ship's thrusters first activate
    vf(c"soundSpin", offset_of!(vehicleInfo_t, soundSpin), VF_SOUND), //sound to loop while spiraling out of control
    vf(c"soundTurbo", offset_of!(vehicleInfo_t, soundTurbo), VF_SOUND), //sound to play when turbo/afterburner kicks in
    vf(c"soundHyper", offset_of!(vehicleInfo_t, soundHyper), VF_SOUND_CLIENT), //sound to play when hits hyperspace
    vf(c"soundLand", offset_of!(vehicleInfo_t, soundLand), VF_SOUND), //sound to play when ship lands
    vf(c"soundFlyBy", offset_of!(vehicleInfo_t, soundFlyBy), VF_SOUND_CLIENT), //sound to play when they buzz you
    vf(c"soundFlyBy2", offset_of!(vehicleInfo_t, soundFlyBy2), VF_SOUND_CLIENT), //alternate sound to play when they buzz you
    vf(c"soundShift1", offset_of!(vehicleInfo_t, soundShift1), VF_SOUND), //sound to play when changing speeds
    vf(c"soundShift2", offset_of!(vehicleInfo_t, soundShift2), VF_SOUND), //sound to play when changing speeds
    vf(c"soundShift3", offset_of!(vehicleInfo_t, soundShift3), VF_SOUND), //sound to play when changing speeds
    vf(c"soundShift4", offset_of!(vehicleInfo_t, soundShift4), VF_SOUND), //sound to play when changing speeds
    vf(c"exhaustFX", offset_of!(vehicleInfo_t, iExhaustFX), VF_EFFECT_CLIENT), //exhaust effect, played from "*exhaust" bolt(s)
    vf(c"turboFX", offset_of!(vehicleInfo_t, iTurboFX), VF_EFFECT_CLIENT), //turbo exhaust effect, played from "*exhaust" bolt(s) when ship is in "turbo" mode
    vf(c"turboStartFX", offset_of!(vehicleInfo_t, iTurboStartFX), VF_EFFECT), //turbo start effect, played from "*exhaust" bolt(s) when ship is in "turbo" mode
    vf(c"trailFX", offset_of!(vehicleInfo_t, iTrailFX), VF_EFFECT_CLIENT), //trail effect, played from "*trail" bolt(s)
    vf(c"impactFX", offset_of!(vehicleInfo_t, iImpactFX), VF_EFFECT_CLIENT), //impact effect, for when it bumps into something
    vf(c"explodeFX", offset_of!(vehicleInfo_t, iExplodeFX), VF_EFFECT), //explosion effect, for when it blows up (should have the sound built into explosion effect)
    vf(c"wakeFX", offset_of!(vehicleInfo_t, iWakeFX), VF_EFFECT_CLIENT), //effect it makes when going across water
    vf(c"dmgFX", offset_of!(vehicleInfo_t, iDmgFX), VF_EFFECT_CLIENT), //effect to play on damage from a weapon or something
    // #ifdef _JK2MP
    vf(c"injureFX", offset_of!(vehicleInfo_t, iInjureFX), VF_EFFECT_CLIENT), //effect to play on partially damaged ship surface
    vf(c"noseFX", offset_of!(vehicleInfo_t, iNoseFX), VF_EFFECT_CLIENT), //effect for nose piece flying away when blown off
    vf(c"lwingFX", offset_of!(vehicleInfo_t, iLWingFX), VF_EFFECT_CLIENT), //effect for left wing piece flying away when blown off
    vf(c"rwingFX", offset_of!(vehicleInfo_t, iRWingFX), VF_EFFECT_CLIENT), //effect for right wing piece flying away when blown off
    // #endif
    // Weapon stuff:
    vf(c"weap1", weap_ofs(0, offset_of!(vehWeaponStats_t, ID)), VF_WEAPON), //weapon used when press fire
    vf(c"weap2", weap_ofs(1, offset_of!(vehWeaponStats_t, ID)), VF_WEAPON), //weapon used when press alt-fire
    // The delay between shots for this weapon.
    vf(c"weap1Delay", weap_ofs(0, offset_of!(vehWeaponStats_t, delay)), VF_INT),
    vf(c"weap2Delay", weap_ofs(1, offset_of!(vehWeaponStats_t, delay)), VF_INT),
    // Whether or not all the muzzles for each weapon can be linked together (linked delay = weapon delay * number of muzzles linked!)
    vf(c"weap1Link", weap_ofs(0, offset_of!(vehWeaponStats_t, linkable)), VF_INT),
    vf(c"weap2Link", weap_ofs(1, offset_of!(vehWeaponStats_t, linkable)), VF_INT),
    // Whether or not to auto-aim the projectiles at the thing under the crosshair when we fire
    vf(c"weap1Aim", weap_ofs(0, offset_of!(vehWeaponStats_t, aimCorrect)), VF_BOOL),
    vf(c"weap2Aim", weap_ofs(1, offset_of!(vehWeaponStats_t, aimCorrect)), VF_BOOL),
    //maximum ammo
    vf(c"weap1AmmoMax", weap_ofs(0, offset_of!(vehWeaponStats_t, ammoMax)), VF_INT),
    vf(c"weap2AmmoMax", weap_ofs(1, offset_of!(vehWeaponStats_t, ammoMax)), VF_INT),
    //ammo recharge rate - milliseconds per unit (minimum of 100, which is 10 ammo per second)
    vf(c"weap1AmmoRechargeMS", weap_ofs(0, offset_of!(vehWeaponStats_t, ammoRechargeMS)), VF_INT),
    vf(c"weap2AmmoRechargeMS", weap_ofs(1, offset_of!(vehWeaponStats_t, ammoRechargeMS)), VF_INT),
    //sound to play when out of ammo (plays default "no ammo" sound if none specified)
    vf(c"weap1SoundNoAmmo", weap_ofs(0, offset_of!(vehWeaponStats_t, soundNoAmmo)), VF_SOUND_CLIENT), //sound to play when try to fire weapon 1 with no ammo
    vf(c"weap2SoundNoAmmo", weap_ofs(1, offset_of!(vehWeaponStats_t, soundNoAmmo)), VF_SOUND_CLIENT), //sound to play when try to fire weapon 2 with no ammo
    // Which weapon a muzzle fires (has to match one of the weapons this vehicle has).
    vf(c"weapMuzzle1", muzzle_ofs(0), VF_WEAPON),
    vf(c"weapMuzzle2", muzzle_ofs(1), VF_WEAPON),
    vf(c"weapMuzzle3", muzzle_ofs(2), VF_WEAPON),
    vf(c"weapMuzzle4", muzzle_ofs(3), VF_WEAPON),
    vf(c"weapMuzzle5", muzzle_ofs(4), VF_WEAPON),
    vf(c"weapMuzzle6", muzzle_ofs(5), VF_WEAPON),
    vf(c"weapMuzzle7", muzzle_ofs(6), VF_WEAPON),
    vf(c"weapMuzzle8", muzzle_ofs(7), VF_WEAPON),
    vf(c"weapMuzzle9", muzzle_ofs(8), VF_WEAPON),
    vf(c"weapMuzzle10", muzzle_ofs(9), VF_WEAPON),
    // The max height before this ship (?) starts (auto)landing.
    vf(c"landingHeight", offset_of!(vehicleInfo_t, landingHeight), VF_FLOAT),
    //other misc stats
    vf(c"gravity", offset_of!(vehicleInfo_t, gravity), VF_INT), //normal is 800
    vf(c"hoverHeight", offset_of!(vehicleInfo_t, hoverHeight), VF_FLOAT), //if 0, it's a ground vehicle
    vf(c"hoverStrength", offset_of!(vehicleInfo_t, hoverStrength), VF_FLOAT), //how hard it pushes off ground when less than hover height... causes "bounce", like shocks
    vf(c"waterProof", offset_of!(vehicleInfo_t, waterProof), VF_BOOL), //can drive underwater if it has to
    vf(c"bouyancy", offset_of!(vehicleInfo_t, bouyancy), VF_FLOAT), //when in water, how high it floats (1 is neutral bouyancy)
    vf(c"fuelMax", offset_of!(vehicleInfo_t, fuelMax), VF_INT), //how much fuel it can hold (capacity)
    vf(c"fuelRate", offset_of!(vehicleInfo_t, fuelRate), VF_INT), //how quickly is uses up fuel
    vf(c"turboDuration", offset_of!(vehicleInfo_t, turboDuration), VF_INT), //how long turbo lasts
    vf(c"turboRecharge", offset_of!(vehicleInfo_t, turboRecharge), VF_INT), //how long turbo takes to recharge
    vf(c"visibility", offset_of!(vehicleInfo_t, visibility), VF_INT), //for sight alerts
    vf(c"loudness", offset_of!(vehicleInfo_t, loudness), VF_INT), //for sound alerts
    vf(c"explosionRadius", offset_of!(vehicleInfo_t, explosionRadius), VF_FLOAT), //range of explosion
    vf(c"explosionDamage", offset_of!(vehicleInfo_t, explosionDamage), VF_INT), //damage of explosion
    //new stuff
    vf(c"maxPassengers", offset_of!(vehicleInfo_t, maxPassengers), VF_INT), // The max number of passengers this vehicle may have (Default = 0).
    vf(c"hideRider", offset_of!(vehicleInfo_t, hideRider), VF_BOOL), // rider (and passengers?) should not be drawn
    vf(c"killRiderOnDeath", offset_of!(vehicleInfo_t, killRiderOnDeath), VF_BOOL), //if rider is on vehicle when it dies, they should die
    vf(c"flammable", offset_of!(vehicleInfo_t, flammable), VF_BOOL), //whether or not the vehicle should catch on fire before it explodes
    vf(c"explosionDelay", offset_of!(vehicleInfo_t, explosionDelay), VF_INT), //how long the vehicle should be on fire/dying before it explodes
    //camera stuff
    vf(c"cameraOverride", offset_of!(vehicleInfo_t, cameraOverride), VF_BOOL), //override the third person camera with the below values - normal is 0 (off)
    vf(c"cameraRange", offset_of!(vehicleInfo_t, cameraRange), VF_FLOAT), //how far back the camera should be - normal is 80
    vf(c"cameraVertOffset", offset_of!(vehicleInfo_t, cameraVertOffset), VF_FLOAT), //how high over the vehicle origin the camera should be - normal is 16
    vf(c"cameraHorzOffset", offset_of!(vehicleInfo_t, cameraHorzOffset), VF_FLOAT), //how far to left/right (negative/positive) of of the vehicle origin the camera should be - normal is 0
    vf(c"cameraPitchOffset", offset_of!(vehicleInfo_t, cameraPitchOffset), VF_FLOAT), //a modifier on the camera's pitch (up/down angle) to the vehicle - normal is 0
    vf(c"cameraFOV", offset_of!(vehicleInfo_t, cameraFOV), VF_FLOAT), //third person camera FOV, default is 80
    vf(c"cameraAlpha", offset_of!(vehicleInfo_t, cameraAlpha), VF_FLOAT), //fade out the vehicle to this alpha (0.1-1.0f) if it's in the way of the crosshair
    vf(c"cameraPitchDependantVertOffset", offset_of!(vehicleInfo_t, cameraPitchDependantVertOffset), VF_BOOL), //use the hacky AT-ST pitch dependant vertical offset
    //===TURRETS===========================================================================
    //Turret 1
    vf(c"turret1Weap", turret_ofs(0, offset_of!(turretStats_t, iWeapon)), VF_WEAPON),
    vf(c"turret1Delay", turret_ofs(0, offset_of!(turretStats_t, iDelay)), VF_INT),
    vf(c"turret1AmmoMax", turret_ofs(0, offset_of!(turretStats_t, iAmmoMax)), VF_INT),
    vf(c"turret1AmmoRechargeMS", turret_ofs(0, offset_of!(turretStats_t, iAmmoRechargeMS)), VF_INT),
    vf(c"turret1YawBone", turret_ofs(0, offset_of!(turretStats_t, yawBone)), VF_LSTRING),
    vf(c"turret1PitchBone", turret_ofs(0, offset_of!(turretStats_t, pitchBone)), VF_LSTRING),
    vf(c"turret1YawAxis", turret_ofs(0, offset_of!(turretStats_t, yawAxis)), VF_INT),
    vf(c"turret1PitchAxis", turret_ofs(0, offset_of!(turretStats_t, pitchAxis)), VF_INT),
    vf(c"turret1ClampYawL", turret_ofs(0, offset_of!(turretStats_t, yawClampLeft)), VF_FLOAT), //how far the turret is allowed to turn left
    vf(c"turret1ClampYawR", turret_ofs(0, offset_of!(turretStats_t, yawClampRight)), VF_FLOAT), //how far the turret is allowed to turn right
    vf(c"turret1ClampPitchU", turret_ofs(0, offset_of!(turretStats_t, pitchClampUp)), VF_FLOAT), //how far the turret is allowed to title up
    vf(c"turret1ClampPitchD", turret_ofs(0, offset_of!(turretStats_t, pitchClampDown)), VF_FLOAT), //how far the turret is allowed to tilt down
    vf(c"turret1Muzzle1", turret_muzzle_ofs(0, 0), VF_INT),
    vf(c"turret1Muzzle2", turret_muzzle_ofs(0, 1), VF_INT),
    vf(c"turret1TurnSpeed", turret_ofs(0, offset_of!(turretStats_t, fTurnSpeed)), VF_FLOAT),
    vf(c"turret1AI", turret_ofs(0, offset_of!(turretStats_t, bAI)), VF_BOOL),
    vf(c"turret1AILead", turret_ofs(0, offset_of!(turretStats_t, bAILead)), VF_BOOL),
    vf(c"turret1AIRange", turret_ofs(0, offset_of!(turretStats_t, fAIRange)), VF_FLOAT),
    vf(c"turret1PassengerNum", turret_ofs(0, offset_of!(turretStats_t, passengerNum)), VF_INT), //which number passenger can control this turret
    vf(c"turret1GunnerViewTag", turret_ofs(0, offset_of!(turretStats_t, gunnerViewTag)), VF_LSTRING),
    //Turret 2
    vf(c"turret2Weap", turret_ofs(1, offset_of!(turretStats_t, iWeapon)), VF_WEAPON),
    vf(c"turret2Delay", turret_ofs(1, offset_of!(turretStats_t, iDelay)), VF_INT),
    vf(c"turret2AmmoMax", turret_ofs(1, offset_of!(turretStats_t, iAmmoMax)), VF_INT),
    vf(c"turret2AmmoRechargeMS", turret_ofs(1, offset_of!(turretStats_t, iAmmoRechargeMS)), VF_INT),
    vf(c"turret2YawBone", turret_ofs(1, offset_of!(turretStats_t, yawBone)), VF_LSTRING),
    vf(c"turret2PitchBone", turret_ofs(1, offset_of!(turretStats_t, pitchBone)), VF_LSTRING),
    vf(c"turret2YawAxis", turret_ofs(1, offset_of!(turretStats_t, yawAxis)), VF_INT),
    vf(c"turret2PitchAxis", turret_ofs(1, offset_of!(turretStats_t, pitchAxis)), VF_INT),
    vf(c"turret2ClampYawL", turret_ofs(1, offset_of!(turretStats_t, yawClampLeft)), VF_FLOAT), //how far the turret is allowed to turn left
    vf(c"turret2ClampYawR", turret_ofs(1, offset_of!(turretStats_t, yawClampRight)), VF_FLOAT), //how far the turret is allowed to turn right
    vf(c"turret2ClampPitchU", turret_ofs(1, offset_of!(turretStats_t, pitchClampUp)), VF_FLOAT), //how far the turret is allowed to title up
    vf(c"turret2ClampPitchD", turret_ofs(1, offset_of!(turretStats_t, pitchClampDown)), VF_FLOAT), //how far the turret is allowed to tilt down
    vf(c"turret2Muzzle1", turret_muzzle_ofs(1, 0), VF_INT),
    vf(c"turret2Muzzle2", turret_muzzle_ofs(1, 1), VF_INT),
    vf(c"turret2TurnSpeed", turret_ofs(1, offset_of!(turretStats_t, fTurnSpeed)), VF_FLOAT),
    vf(c"turret2AI", turret_ofs(1, offset_of!(turretStats_t, bAI)), VF_BOOL),
    vf(c"turret2AILead", turret_ofs(1, offset_of!(turretStats_t, bAILead)), VF_BOOL),
    vf(c"turret2AIRange", turret_ofs(1, offset_of!(turretStats_t, fAIRange)), VF_FLOAT),
    vf(c"turret2PassengerNum", turret_ofs(1, offset_of!(turretStats_t, passengerNum)), VF_INT), //which number passenger can control this turret
    vf(c"turret2GunnerViewTag", turret_ofs(1, offset_of!(turretStats_t, gunnerViewTag)), VF_LSTRING),
    //===END TURRETS===========================================================================
    //terminating entry
    VEHFIELD_TERMINATOR,
];

/// Builds one `stringID_table_t` row from the C `ENUM2STRING(arg)` macro
/// (`#arg, arg`): the stringized enumerator name paired with its value.
const fn enum2string(name: &'static CStr, id: vehicleType_t) -> stringID_table_t {
    stringID_table_t { name: name.as_ptr(), id }
}

/// `stringID_table_t VehicleTable[VH_NUM_VEHICLES+1]` (bg_vehicleLoad.c:680) -- the
/// vehicle-type name table for the `VF_VEHTYPE` lookup (`GetIDForString( VehicleTable,
/// value )`). The C global is non-const and `stringID_table_t` carries a raw
/// `*const c_char` (`!Sync`), so `static mut` (the `vehWeaponFields`/`vehicleFields`
/// precedent; read via `addr_of!`). The trailing `{ 0, -1 }` is the table terminator.
pub static mut VehicleTable: [stringID_table_t; (VH_NUM_VEHICLES + 1) as usize] = [
    enum2string(c"VH_NONE", VH_NONE),
    enum2string(c"VH_WALKER", VH_WALKER), //something you ride inside of, it walks like you, like an AT-ST
    enum2string(c"VH_FIGHTER", VH_FIGHTER), //something you fly inside of, like an X-Wing or TIE fighter
    enum2string(c"VH_SPEEDER", VH_SPEEDER), //something you ride on that hovers, like a speeder or swoop
    enum2string(c"VH_ANIMAL", VH_ANIMAL), //animal you ride on top of that walks, like a tauntaun
    enum2string(c"VH_FLIER", VH_FLIER),   //animal you ride on top of that flies, like a giant mynoc?
    stringID_table_t { name: core::ptr::null(), id: -1 }, // 0, -1
];

/// `BG_SetSharedVehicleFunctions( vehicleInfo_t *pVehInfo )` (bg_vehicleLoad.c:692) — install
/// the function-pointer table on a vehicle record. Build config = retail `QAGAME` (and
/// `WE_ARE_IN_THE_UI` undefined): so first call the server-side
/// [`G_SetSharedVehicleFunctions`] (the `#ifdef QAGAME` block), then dispatch on
/// `pVehInfo->type` to the per-type setter (the `#ifndef WE_ARE_IN_THE_UI` switch). The
/// `VH_FLIER`/`VH_NONE`/default cases have no setter in C (fall through), matching the
/// Rust `_ => {}`.
///
/// No oracle — assigns engine fn-pointers / touches the opaque vehicle struct, not pure
/// computation.
///
/// # Safety
/// `pVehInfo` must point at a valid, writable [`vehicleInfo_t`].
pub unsafe fn BG_SetSharedVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    //only do the whole thing if we're on game
    G_SetSharedVehicleFunctions(pVehInfo);

    match (*pVehInfo).r#type {
        VH_SPEEDER => {
            G_SetSpeederVehicleFunctions(pVehInfo);
        }
        VH_ANIMAL => {
            G_SetAnimalVehicleFunctions(pVehInfo);
        }
        VH_FIGHTER => {
            G_SetFighterVehicleFunctions(pVehInfo);
        }
        VH_WALKER => {
            G_SetWalkerVehicleFunctions(pVehInfo);
        }
        _ => {}
    }
}

/// `BG_VehWeaponLoadParms` (bg_vehicleLoad.c:1404) -- concatenate every
/// `ext_data/vehicles/weapons/*.vwp` file into the global `VehWeaponParms` text
/// block, separating `}`-terminated chunks with a space so adjacent tokens never
/// merge. The `.vwp` text is later tokenized by `BG_ParseVehWeaponParm` (not yet
/// ported).
///
/// Faithful to the `_JK2MP` + `QAGAME` build: `trap_FS_GetFileList` / the
/// `trap_FS_*` file traps and `BG_TempAlloc`/`BG_TempFree` (not the SP `gi.*`).
/// The C `va( "ext_data/vehicles/weapons/%s", holdChar )` collapses into a Rust
/// `format!` since [`trap::FS_FOpenFile`] already takes `&str` and re-`CString`s
/// it. `mainBlockLen` is dropped: dead in C (assigned `len`, never read).
///
/// No oracle test -- the body is pure engine-trap I/O (`trap_FS_*`), which the
/// off-engine oracle harness cannot satisfy (cf. `BG_ModelCache`).
pub fn BG_VehWeaponLoadParms() {
    let mut len: c_int;
    let mut totallen: c_int;
    let fileCnt: c_int;
    let mut vehWeaponExtensionListBuf = [0 as c_char; 2048];

    len = 0;

    // remember where to store the next one
    totallen = len;

    // SAFETY: single-threaded module; the global `VehWeaponParms` text block and
    // the temp read buffer are walked with raw pointers exactly as the C does.
    unsafe {
        let base = addr_of_mut!(VehWeaponParms) as *mut c_char;
        let mut marker = base.add(totallen as usize);
        *marker = 0;

        // now load in the extra .veh extensions
        fileCnt = trap::FS_GetFileList(
            "ext_data/vehicles/weapons",
            ".vwp",
            &mut vehWeaponExtensionListBuf,
        );

        let mut holdChar = vehWeaponExtensionListBuf.as_mut_ptr();

        let tempReadBuffer = BG_TempAlloc(MAX_VEH_WEAPON_DATA_SIZE as c_int) as *mut c_char;

        // NOTE: Not use TempAlloc anymore...
        // Make ABSOLUTELY CERTAIN that BG_Alloc/etc. is not used before
        // the subsequent BG_TempFree or the pool will be screwed.

        let mut i = 0;
        while i < fileCnt {
            let vehExtFNLen = strlen(holdChar) as c_int;

            //		Com_Printf( "Parsing %s\n", holdChar );

            let path = format!(
                "ext_data/vehicles/weapons/{}",
                CStr::from_ptr(holdChar).to_string_lossy()
            );
            let (l, f) = trap::FS_FOpenFile(&path, FS_READ);
            len = l;

            if len == -1 {
                Com_Printf("error reading file\n");
            } else {
                let buf = core::slice::from_raw_parts_mut(tempReadBuffer as *mut u8, len as usize);
                trap::FS_Read(buf, f);
                *tempReadBuffer.add(len as usize) = 0;

                // Don't let it end on a } because that should be a stand-alone token.
                if totallen != 0 && *marker.offset(-1) == b'}' as c_char {
                    strcat(marker, c" ".as_ptr());
                    totallen += 1;
                    marker = marker.add(1);
                }

                if totallen + len >= MAX_VEH_WEAPON_DATA_SIZE as c_int {
                    Com_Error(ERR_DROP, "Vehicle Weapon extensions (*.vwp) are too large");
                }
                strcat(marker, tempReadBuffer);
                trap::FS_FCloseFile(f);

                totallen += len;
                marker = base.add(totallen as usize);
            }

            i += 1;
            holdChar = holdChar.add((vehExtFNLen + 1) as usize);
        }

        BG_TempFree(MAX_VEH_WEAPON_DATA_SIZE as c_int);
    }
}

/// `BG_VehicleLoadParms` (bg_vehicleLoad.c:1495) -- concatenate every
/// `ext_data/vehicles/*.veh` file into the global `VehicleParms` text block (mirroring
/// [`BG_VehWeaponLoadParms`] over the `.veh`/`VehicleParms`/`MAX_VEHICLE_DATA_SIZE`
/// trio), then prime slot [`VEHICLE_BASE`]: set `numVehicles = 1`, default + clamp +
/// install the shared funcs on `g_vehicleInfo[VEHICLE_BASE]`, and finally load the
/// vehicle-weapon parms too.
///
/// Faithful to the `_JK2MP` + `QAGAME` build (the `trap_FS_*` file traps and
/// `BG_TempAlloc`/`BG_TempFree`, not the SP `gi.*`); `mainBlockLen` is dropped (dead in C,
/// assigned `len` and never read). No oracle -- pure engine-trap I/O plus the priming
/// calls, which the off-engine oracle harness cannot satisfy.
pub fn BG_VehicleLoadParms() {
    //HMM... only do this if there's a vehicle on the level?
    let mut len: c_int;
    let mut totallen: c_int;
    let fileCnt: c_int;
    let mut vehExtensionListBuf = [0 as c_char; 2048];

    len = 0;

    //remember where to store the next one
    totallen = len;

    // SAFETY: single-threaded module; the global `VehicleParms` text block and the temp
    // read buffer are walked with raw pointers exactly as the C does.
    unsafe {
        let base = addr_of_mut!(VehicleParms) as *mut c_char;
        let mut marker = base.add(totallen as usize);
        *marker = 0;

        //now load in the extra .veh extensions
        fileCnt = trap::FS_GetFileList("ext_data/vehicles", ".veh", &mut vehExtensionListBuf);

        let mut holdChar = vehExtensionListBuf.as_mut_ptr();

        let tempReadBuffer = BG_TempAlloc(MAX_VEHICLE_DATA_SIZE as c_int) as *mut c_char;

        // NOTE: Not use TempAlloc anymore...
        //Make ABSOLUTELY CERTAIN that BG_Alloc/etc. is not used before
        //the subsequent BG_TempFree or the pool will be screwed.

        let mut i = 0;
        while i < fileCnt {
            let vehExtFNLen = strlen(holdChar) as c_int;

            //		Com_Printf( "Parsing %s\n", holdChar );

            let path = format!(
                "ext_data/vehicles/{}",
                CStr::from_ptr(holdChar).to_string_lossy()
            );
            let (l, f) = trap::FS_FOpenFile(&path, FS_READ);
            len = l;

            if len == -1 {
                Com_Printf("error reading file\n");
            } else {
                let buf = core::slice::from_raw_parts_mut(tempReadBuffer as *mut u8, len as usize);
                trap::FS_Read(buf, f);
                *tempReadBuffer.add(len as usize) = 0;

                // Don't let it end on a } because that should be a stand-alone token.
                if totallen != 0 && *marker.offset(-1) == b'}' as c_char {
                    strcat(marker, c" ".as_ptr());
                    totallen += 1;
                    marker = marker.add(1);
                }

                if totallen + len >= MAX_VEHICLE_DATA_SIZE as c_int {
                    Com_Error(ERR_DROP, "Vehicle extensions (*.veh) are too large");
                }
                strcat(marker, tempReadBuffer);
                trap::FS_FCloseFile(f);

                totallen += len;
                marker = base.add(totallen as usize);
            }

            i += 1;
            holdChar = holdChar.add((vehExtFNLen + 1) as usize);
        }

        BG_TempFree(MAX_VEHICLE_DATA_SIZE as c_int);

        *addr_of_mut!(numVehicles) = 1; //first one is null/default
        //set the first vehicle to default data
        BG_VehicleSetDefaults(
            (addr_of_mut!(g_vehicleInfo) as *mut vehicleInfo_t).add(VEHICLE_BASE as usize),
        );
        //sanity check and clamp the vehicle's data
        BG_VehicleClampData(
            (addr_of_mut!(g_vehicleInfo) as *mut vehicleInfo_t).add(VEHICLE_BASE as usize),
        );
        // Setup the shared function pointers.
        BG_SetSharedVehicleFunctions(
            (addr_of_mut!(g_vehicleInfo) as *mut vehicleInfo_t).add(VEHICLE_BASE as usize),
        );
    }

    //Load the Vehicle Weapons data, too
    BG_VehWeaponLoadParms();
}

/// `BG_ParseVehWeaponParm( vehWeaponInfo_t *vehWeapon, char *parmName, char *pValue )`
/// (bg_vehicleLoad.c:176) — set one `.vwp` field on `vehWeapon` from a `parmName`/`pValue`
/// text pair. Scans `vehWeaponFields[]` for a case-insensitive name match, then decodes
/// `pValue` per the field's [`vehFieldType_t`] and writes it at `f->ofs` bytes. Returns
/// `qtrue` on a recognized parm, `qfalse` if no field matched (or the type fell through to
/// the `default` arm). The `.vwp` (`vehWeaponFields`) table only exercises the
/// `VF_LSTRING`/`VF_BOOL`/`VF_FLOAT`/`VF_INT` (+ no-op client) arms; the rest of the switch
/// is ported faithfully for completeness.
///
/// **Build config = retail `QAGAME`** (`_JK2MP`, no `CGAME`/UI): `VF_LSTRING` allocates via
/// [`BG_Alloc`] + `strcpy`; `VF_MODEL`/`VF_EFFECT`/`VF_SOUND` route to
/// [`G_ModelIndex`]/[`G_EffectIndex`]/[`G_SoundIndex`]; the `_CLIENT` variants, `VF_WEAPON`,
/// `VF_SHADER`, and `VF_SHADER_NOMIP` are commented-out / `#ifndef QAGAME` in C, so they are
/// no-ops here. `atoi`/`atof` are the platform libc; `sscanf` is the
/// [`bg_lib::sscanf`](crate::codemp::game::bg_lib::sscanf) cfg-split (slice form under `vm`,
/// variadic libc native) — the [`BG_ParseField`](crate::codemp::game::bg_misc::BG_ParseField)
/// precedent.
///
/// # Safety
/// `vehWeapon` must point at a valid [`vehWeaponInfo_t`]; `parmName`/`pValue` valid
/// NUL-terminated C strings — the contract the `.vwp` parser ([`VEH_LoadVehWeapon`]) upholds.
pub unsafe fn BG_ParseVehWeaponParm(
    vehWeapon: *mut vehWeaponInfo_t,
    parmName: *const c_char,
    pValue: *const c_char,
) -> qboolean {
    let b = vehWeapon as *mut u8;
    let mut value = [0 as c_char; 1024];

    Q_strncpyz(value.as_mut_ptr(), pValue, value.len() as c_int);

    // Loop through possible parameters
    let mut i: c_int = 0;
    while i < NUM_VWEAP_PARMS {
        let f = &*(addr_of!(vehWeaponFields) as *const vehField_t).add(i as usize);
        if !f.name.is_null() && Q_stricmp(f.name, parmName) == 0 {
            // found it
            match f.r#type {
                VF_INT => {
                    *(b.add(f.ofs as usize) as *mut c_int) = atoi(value.as_ptr());
                }
                VF_FLOAT => {
                    *(b.add(f.ofs as usize) as *mut f32) = atof(value.as_ptr()) as f32;
                }
                VF_LSTRING => {
                    // string on disk, pointer in memory, TAG_LEVEL
                    if (*(b.add(f.ofs as usize) as *mut *mut c_char)).is_null() {
                        //just use 1024 bytes in case we want to write over the string
                        let p = BG_Alloc(1024) as *mut c_char; //BG_Alloc(strlen(value))
                        *(b.add(f.ofs as usize) as *mut *mut c_char) = p;
                        strcpy(p, value.as_ptr());
                    }
                }
                VF_VECTOR => {
                    let mut vec: vec3_t = [0.0; 3];
                    // NOTE: the `vm` sscanf shim always returns 0 (it never increments the
                    // match count — see DEVIATIONS.md), so the `!=3` warning below would
                    // fire under the `vm` build; the `.vwp` (vehWeaponFields) table has no
                    // VF_VECTOR row, so this arm is unreached in practice.
                    #[cfg(feature = "vm")]
                    let _iFieldsRead = sscanf(
                        value.as_ptr(),
                        c"%f %f %f".as_ptr(),
                        &[
                            addr_of_mut!(vec[0]) as *mut c_void,
                            addr_of_mut!(vec[1]) as *mut c_void,
                            addr_of_mut!(vec[2]) as *mut c_void,
                        ],
                    );
                    #[cfg(not(feature = "vm"))]
                    let _iFieldsRead = sscanf(
                        value.as_ptr(),
                        c"%f %f %f".as_ptr(),
                        addr_of_mut!(vec[0]),
                        addr_of_mut!(vec[1]),
                        addr_of_mut!(vec[2]),
                    );
                    // assert(_iFieldsRead==3) -- NDEBUG, compiled out in the retail build.
                    if _iFieldsRead != 3 {
                        Com_Printf("^3BG_ParseVehWeaponParm: VEC3 sscanf() failed to read 3 floats ('angle' key bug?)\n");
                    }
                    let p = b.add(f.ofs as usize) as *mut f32;
                    *p.add(0) = vec[0];
                    *p.add(1) = vec[1];
                    *p.add(2) = vec[2];
                }
                VF_BOOL => {
                    *(b.add(f.ofs as usize) as *mut qboolean) =
                        (atof(value.as_ptr()) != 0.0) as qboolean;
                }
                VF_VEHTYPE => {
                    let vehType =
                        GetIDForString(addr_of!(VehicleTable) as *const stringID_table_t, value.as_ptr())
                            as vehicleType_t;
                    *(b.add(f.ofs as usize) as *mut vehicleType_t) = vehType;
                }
                VF_ANIM => {
                    let anim = GetIDForString(
                        addr_of!(animTable) as *const stringID_table_t,
                        value.as_ptr(),
                    );
                    *(b.add(f.ofs as usize) as *mut c_int) = anim;
                }
                VF_MODEL => {
                    // take the string, get the G_ModelIndex
                    *(b.add(f.ofs as usize) as *mut c_int) =
                        G_ModelIndex(&CStr::from_ptr(value.as_ptr()).to_string_lossy());
                }
                VF_EFFECT => {
                    // take the string, get the G_EffectIndex
                    *(b.add(f.ofs as usize) as *mut c_int) =
                        G_EffectIndex(&CStr::from_ptr(value.as_ptr()).to_string_lossy());
                }
                VF_SOUND => {
                    // take the string, get the G_SoundIndex
                    *(b.add(f.ofs as usize) as *mut c_int) =
                        G_SoundIndex(&CStr::from_ptr(value.as_ptr()).to_string_lossy());
                }
                // QAGAME no-ops: VF_WEAPON (commented in C), the `_CLIENT` variants and
                // VF_SHADER/VF_SHADER_NOMIP are `#ifndef QAGAME` / commented-out, so the
                // server build does nothing for these.
                VF_WEAPON | VF_MODEL_CLIENT | VF_EFFECT_CLIENT | VF_SHADER | VF_SHADER_NOMIP
                | VF_SOUND_CLIENT => {}
                // default (incl. VF_IGNORE): unknown type
                _ => return QFALSE,
            }
            break;
        }
        i += 1;
    }
    if i == NUM_VWEAP_PARMS {
        QFALSE
    } else {
        QTRUE
    }
}

/// `int VEH_LoadVehWeapon( const char *vehWeaponName )` (bg_vehicleLoad.c:318) — load the
/// named vehicle weapon out of the concatenated `VehWeaponParms` text into the next
/// `g_vehWeaponInfo[]` slot and return its index (post-incrementing `numVehicleWeapons`).
/// Token-scans the buffer with [`COM_ParseExt`]/[`SkipBracedSection`] for the matching
/// `{ … }` block, then feeds each key/value pair to [`BG_ParseVehWeaponParm`]. Returns
/// `qfalse` (0) if the name/opening-brace isn't found, [`VEH_WEAPON_NONE`] on a malformed
/// block.
///
/// The `if ( vehWeapon->fHoming )` lock-on-sound registration is `CGAME`/UI only; under
/// `QAGAME` its body is empty (the server need not register these), so it is a no-op here.
pub unsafe fn VEH_LoadVehWeapon(vehWeaponName: *const c_char) -> c_int {
    //load up specified vehWeapon and save in array: g_vehWeaponInfo
    let mut token: *const c_char;
    let mut parmName = [0 as c_char; 128]; //we'll assume that no parm name is longer than 128
    let mut value: *mut c_char;

    //BG_VehWeaponSetDefaults( &g_vehWeaponInfo[0] );//set the first vehicle to default data

    //try to parse data out
    let mut p = addr_of!(VehWeaponParms) as *const c_char;

    COM_BeginParseSession(c"vehWeapons".as_ptr());

    let vehWeapon =
        (addr_of_mut!(g_vehWeaponInfo) as *mut vehWeaponInfo_t).add(numVehicleWeapons as usize);
    // look for the right vehicle weapon
    while !p.is_null() {
        token = COM_ParseExt(&mut p, QTRUE);
        if *token == 0 {
            return QFALSE;
        }

        if Q_stricmp(token, vehWeaponName) == 0 {
            break;
        }

        SkipBracedSection(&mut p);
    }
    if p.is_null() {
        return QFALSE;
    }

    token = COM_ParseExt(&mut p, QTRUE);
    if *token == 0 {
        //barf
        return VEH_WEAPON_NONE;
    }

    if Q_stricmp(token, c"{".as_ptr()) != 0 {
        return VEH_WEAPON_NONE;
    }

    // parse the vehWeapon info block
    loop {
        SkipRestOfLine(&mut p);
        token = COM_ParseExt(&mut p, QTRUE);
        if *token == 0 {
            Com_Printf(&format!(
                "^1ERROR: unexpected EOF while parsing Vehicle Weapon '{}'\n",
                Sz(vehWeaponName)
            ));
            return VEH_WEAPON_NONE;
        }

        if Q_stricmp(token, c"}".as_ptr()) == 0 {
            break;
        }
        Q_strncpyz(parmName.as_mut_ptr(), token, parmName.len() as c_int);
        value = COM_ParseExt(&mut p, QTRUE);
        if value.is_null() || *value == 0 {
            Com_Printf(&format!(
                "^1ERROR: Vehicle Weapon token '{}' has no value!\n",
                Sz(parmName.as_ptr())
            ));
        } else if BG_ParseVehWeaponParm(vehWeapon, parmName.as_ptr(), value) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle Weapon key/value pair '{}','{}'!\n",
                Sz(parmName.as_ptr()),
                Sz(value)
            ));
        }
    }
    if (*vehWeapon).fHoming != 0.0 {
        //all lock-on weapons use these 2 sounds
        // QAGAME: no need to have the server register these (CGAME/UI only).
    }
    let ret = numVehicleWeapons;
    *addr_of_mut!(numVehicleWeapons) += 1;
    ret
}

/// `int VEH_VehWeaponIndexForName( const char *vehWeaponName )` (bg_vehicleLoad.c:422) —
/// resolve a vehicle-weapon name to its `g_vehWeaponInfo[]` index, loading it on first use
/// via [`VEH_LoadVehWeapon`]. Returns the existing index if already loaded,
/// [`VEH_WEAPON_NONE`] on a missing name / overflow (max [`MAX_VEH_WEAPONS`]) / load failure.
///
/// # Safety
/// `vehWeaponName` may be null/empty (handled); otherwise a valid NUL-terminated C string.
pub unsafe fn VEH_VehWeaponIndexForName(vehWeaponName: *const c_char) -> c_int {
    if vehWeaponName.is_null() || *vehWeaponName == 0 {
        Com_Printf("^1ERROR: Trying to read Vehicle Weapon with no name!\n");
        return VEH_WEAPON_NONE;
    }
    let mut vw = VEH_WEAPON_BASE;
    while vw < numVehicleWeapons {
        let vwInfo = &*(addr_of!(g_vehWeaponInfo) as *const vehWeaponInfo_t).add(vw as usize);
        if !vwInfo.name.is_null() && Q_stricmp(vwInfo.name, vehWeaponName) == 0 {
            //already loaded this one
            return vw;
        }
        vw += 1;
    }
    //haven't loaded it yet
    if vw >= MAX_VEH_WEAPONS as c_int {
        //no more room!
        Com_Printf(&format!(
            "^1ERROR: Too many Vehicle Weapons (max 16), aborting load on {}!\n",
            Sz(vehWeaponName)
        ));
        return VEH_WEAPON_NONE;
    }
    //we have room for another one, load it up and return the index
    //HMM... should we not even load the .vwp file until we want to?
    vw = VEH_LoadVehWeapon(vehWeaponName);
    if vw == VEH_WEAPON_NONE {
        Com_Printf(&format!(
            "^1ERROR: Could not find Vehicle Weapon {}!\n",
            Sz(vehWeaponName)
        ));
    }
    vw
}

/// `int VEH_LoadVehicle( const char *vehicleName )` (bg_vehicleLoad.c:992) — load the named
/// vehicle out of the concatenated `VehicleParms` text into the next `g_vehicleInfo[]` slot
/// and return its index (post-incrementing `numVehicles`). Loads the parms first if no
/// vehicles are loaded yet, token-scans for the matching `{ … }` block with
/// [`COM_ParseExt`]/[`SkipBracedSection`], then feeds each key/value to
/// [`BG_ParseVehicleParm`] — except the `weap1`/`weap2`/`weapMuzzle1..10` keys, which are
/// stashed and re-parsed *after* the block so the inner text parser isn't re-entered while
/// this one is mid-parse. Returns [`VEHICLE_NONE`] on a missing name / malformed block.
///
/// Build config = `_JK2MP` + `QAGAME`: `COM_BeginParseSession("vehicles")`; the area-health
/// defaults are `#ifdef _JK2MP`; `modelIndex` comes from [`G_ModelIndex`]
/// (`va("models/players/%s/model.glm", …)`). The skin handling is a **no-op in our build**:
/// the SP `#ifndef _JK2MP` block (`ratl::string_vs`/`gi.RE_RegisterSkin`/`G_SkinIndex`) and
/// the MP `#else / #ifndef QAGAME` (`trap_R_RegisterSkin`) block are both `#ifdef`-excluded
/// under `_JK2MP`+`QAGAME` — so nothing registers the skin here (`G_SkinIndex` lives only in
/// that dead SP branch and is intentionally not ported). The misc-effect `G_EffectIndex`/
/// `G_SoundIndex` priming calls are the `#ifdef QAGAME` arms.
///
/// No oracle — text parse + engine traps + the opaque vehicle struct.
///
/// # Safety
/// `vehicleName` a valid NUL-terminated C string.
pub unsafe fn VEH_LoadVehicle(vehicleName: *const c_char) -> c_int {
    //load up specified vehicle and save in array: g_vehicleInfo
    let mut token: *const c_char;
    //we'll assume that no parm name is longer than 128
    let mut parmName = [0 as c_char; 128];
    let mut weap1 = [0 as c_char; 128];
    let mut weap2 = [0 as c_char; 128];
    let mut weapMuzzle1 = [0 as c_char; 128];
    let mut weapMuzzle2 = [0 as c_char; 128];
    let mut weapMuzzle3 = [0 as c_char; 128];
    let mut weapMuzzle4 = [0 as c_char; 128];
    let mut weapMuzzle5 = [0 as c_char; 128];
    let mut weapMuzzle6 = [0 as c_char; 128];
    let mut weapMuzzle7 = [0 as c_char; 128];
    let mut weapMuzzle8 = [0 as c_char; 128];
    let mut weapMuzzle9 = [0 as c_char; 128];
    let mut weapMuzzle10 = [0 as c_char; 128];
    let mut value: *mut c_char;

    // Load the vehicle parms if no vehicles have been loaded yet.
    if numVehicles == 0 {
        BG_VehicleLoadParms();
    }

    //try to parse data out
    let mut p = addr_of!(VehicleParms) as *const c_char;

    COM_BeginParseSession(c"vehicles".as_ptr());

    let vehicle = (addr_of_mut!(g_vehicleInfo) as *mut vehicleInfo_t).add(numVehicles as usize);
    // look for the right vehicle
    while !p.is_null() {
        token = COM_ParseExt(&mut p, QTRUE);
        if *token == 0 {
            return VEHICLE_NONE;
        }

        if Q_stricmp(token, vehicleName) == 0 {
            break;
        }

        SkipBracedSection(&mut p);
    }

    if p.is_null() {
        return VEHICLE_NONE;
    }

    token = COM_ParseExt(&mut p, QTRUE);
    if *token == 0 {
        //barf
        return VEHICLE_NONE;
    }

    if Q_stricmp(token, c"{".as_ptr()) != 0 {
        return VEHICLE_NONE;
    }

    BG_VehicleSetDefaults(vehicle);
    // parse the vehicle info block
    loop {
        SkipRestOfLine(&mut p);
        token = COM_ParseExt(&mut p, QTRUE);
        if *token == 0 {
            Com_Printf(&format!(
                "^1ERROR: unexpected EOF while parsing Vehicle '{}'\n",
                Sz(vehicleName)
            ));
            return VEHICLE_NONE;
        }

        if Q_stricmp(token, c"}".as_ptr()) == 0 {
            break;
        }
        Q_strncpyz(parmName.as_mut_ptr(), token, parmName.len() as c_int);
        value = COM_ParseExt(&mut p, QTRUE);
        if value.is_null() || *value == 0 {
            Com_Printf(&format!(
                "^1ERROR: Vehicle token '{}' has no value!\n",
                Sz(parmName.as_ptr())
            ));
        } else if Q_stricmp(c"weap1".as_ptr(), parmName.as_ptr()) == 0 {
            //hmm, store this off because we don't want to call another one of these text parsing routines while we're in the middle of one...
            Q_strncpyz(weap1.as_mut_ptr(), value, weap1.len() as c_int);
        } else if Q_stricmp(c"weap2".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weap2.as_mut_ptr(), value, weap2.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle1".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle1.as_mut_ptr(), value, weapMuzzle1.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle2".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle2.as_mut_ptr(), value, weapMuzzle2.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle3".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle3.as_mut_ptr(), value, weapMuzzle3.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle4".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle4.as_mut_ptr(), value, weapMuzzle4.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle5".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle5.as_mut_ptr(), value, weapMuzzle5.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle6".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle6.as_mut_ptr(), value, weapMuzzle6.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle7".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle7.as_mut_ptr(), value, weapMuzzle7.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle8".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle8.as_mut_ptr(), value, weapMuzzle8.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle9".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle9.as_mut_ptr(), value, weapMuzzle9.len() as c_int);
        } else if Q_stricmp(c"weapMuzzle10".as_ptr(), parmName.as_ptr()) == 0 {
            Q_strncpyz(weapMuzzle10.as_mut_ptr(), value, weapMuzzle10.len() as c_int);
        } else if BG_ParseVehicleParm(vehicle, parmName.as_ptr(), value) == QFALSE {
            // #ifndef FINAL_BUILD
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair '{}', '{}'!\n",
                Sz(parmName.as_ptr()),
                Sz(value)
            ));
            // #endif
        }
    }
    //NOW: if we have any weapons, go ahead and load them
    if weap1[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weap1".as_ptr(), weap1.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weap1', '{}'!\n",
                Sz(weap1.as_ptr())
            ));
        }
    }
    if weap2[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weap2".as_ptr(), weap2.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weap2', '{}'!\n",
                Sz(weap2.as_ptr())
            ));
        }
    }
    if weapMuzzle1[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle1".as_ptr(), weapMuzzle1.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle1', '{}'!\n",
                Sz(weapMuzzle1.as_ptr())
            ));
        }
    }
    if weapMuzzle2[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle2".as_ptr(), weapMuzzle2.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle2', '{}'!\n",
                Sz(weapMuzzle2.as_ptr())
            ));
        }
    }
    if weapMuzzle3[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle3".as_ptr(), weapMuzzle3.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle3', '{}'!\n",
                Sz(weapMuzzle3.as_ptr())
            ));
        }
    }
    if weapMuzzle4[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle4".as_ptr(), weapMuzzle4.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle4', '{}'!\n",
                Sz(weapMuzzle4.as_ptr())
            ));
        }
    }
    if weapMuzzle5[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle5".as_ptr(), weapMuzzle5.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle5', '{}'!\n",
                Sz(weapMuzzle5.as_ptr())
            ));
        }
    }
    if weapMuzzle6[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle6".as_ptr(), weapMuzzle6.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle6', '{}'!\n",
                Sz(weapMuzzle6.as_ptr())
            ));
        }
    }
    if weapMuzzle7[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle7".as_ptr(), weapMuzzle7.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle7', '{}'!\n",
                Sz(weapMuzzle7.as_ptr())
            ));
        }
    }
    if weapMuzzle8[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle8".as_ptr(), weapMuzzle8.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle8', '{}'!\n",
                Sz(weapMuzzle8.as_ptr())
            ));
        }
    }
    if weapMuzzle9[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle9".as_ptr(), weapMuzzle9.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle9', '{}'!\n",
                Sz(weapMuzzle9.as_ptr())
            ));
        }
    }
    if weapMuzzle10[0] != 0 {
        if BG_ParseVehicleParm(vehicle, c"weapMuzzle10".as_ptr(), weapMuzzle10.as_ptr()) == QFALSE {
            Com_Printf(&format!(
                "^1ERROR: Unknown Vehicle key/value pair 'weapMuzzle10', '{}'!\n",
                Sz(weapMuzzle10.as_ptr())
            ));
        }
    }

    // #ifdef _JK2MP
    //let's give these guys some defaults
    if (*vehicle).health_front == 0 {
        (*vehicle).health_front = (*vehicle).armor / 4;
    }
    if (*vehicle).health_back == 0 {
        (*vehicle).health_back = (*vehicle).armor / 4;
    }
    if (*vehicle).health_right == 0 {
        (*vehicle).health_right = (*vehicle).armor / 4;
    }
    if (*vehicle).health_left == 0 {
        (*vehicle).health_left = (*vehicle).armor / 4;
    }
    // #endif

    if !(*vehicle).model.is_null() {
        // QAGAME
        (*vehicle).modelIndex = G_ModelIndex(&format!(
            "models/players/{}/model.glm",
            CStr::from_ptr((*vehicle).model).to_string_lossy()
        ));
    }

    // SP `#ifndef _JK2MP` skin block (ratl::string_vs / gi.RE_RegisterSkin / G_SkinIndex) and
    // the MP `#else / #ifndef QAGAME` skin block (trap_R_RegisterSkin) are BOTH cfg'd out under
    // _JK2MP + QAGAME, so skin handling is a no-op in this build. (G_SkinIndex lives only in
    // that dead SP branch and is intentionally not ported.)

    //sanity check and clamp the vehicle's data
    BG_VehicleClampData(vehicle);
    // Setup the shared function pointers.
    BG_SetSharedVehicleFunctions(vehicle);
    //misc effects... FIXME: not even used in MP, are they?
    if (*vehicle).explosionDamage != 0 {
        // QAGAME
        G_EffectIndex("ships/ship_explosion_mark");
    }
    if (*vehicle).flammable != 0 {
        // QAGAME
        G_SoundIndex("sound/vehicles/common/fire_lp.wav");
    }

    if (*vehicle).hoverHeight > 0.0 {
        // QAGAME
        G_EffectIndex("ships/swoop_dust");
    }

    // QAGAME
    G_EffectIndex("volumetric/black_smoke");
    G_EffectIndex("ships/fire");
    G_SoundIndex("sound/vehicles/common/release.wav");

    let ret = numVehicles;
    *addr_of_mut!(numVehicles) += 1;
    ret
}

/// `int VEH_VehicleIndexForName( const char *vehicleName )` (bg_vehicleLoad.c:1372) — resolve
/// a vehicle name to its `g_vehicleInfo[]` index, loading it on first use via
/// [`VEH_LoadVehicle`]. Returns the existing index if already loaded, [`VEHICLE_NONE`] on a
/// missing name / overflow (max [`MAX_VEHICLES`]) / load failure.
///
/// No oracle — touches the global vehicle array + drives the trap-backed loader.
///
/// # Safety
/// `vehicleName` may be null/empty (handled); otherwise a valid NUL-terminated C string.
pub unsafe fn VEH_VehicleIndexForName(vehicleName: *const c_char) -> c_int {
    if vehicleName.is_null() || *vehicleName == 0 {
        Com_Printf("^1ERROR: Trying to read Vehicle with no name!\n");
        return VEHICLE_NONE;
    }
    let mut v = VEHICLE_BASE;
    while v < numVehicles {
        let vInfo = &*(addr_of!(g_vehicleInfo) as *const vehicleInfo_t).add(v as usize);
        if !vInfo.name.is_null() && Q_stricmp(vInfo.name, vehicleName) == 0 {
            //already loaded this one
            return v;
        }
        v += 1;
    }
    //haven't loaded it yet
    if v >= MAX_VEHICLES as c_int {
        //no more room!
        Com_Printf(&format!(
            "^1ERROR: Too many Vehicles (max 64), aborting load on {}!\n",
            Sz(vehicleName)
        ));
        return VEHICLE_NONE;
    }
    //we have room for another one, load it up and return the index
    //HMM... should we not even load the .veh file until we want to?
    v = VEH_LoadVehicle(vehicleName);
    if v == VEHICLE_NONE {
        Com_Printf(&format!(
            "^1ERROR: Could not find Vehicle {}!\n",
            Sz(vehicleName)
        ));
    }
    v
}

/// `int BG_VehicleGetIndex( const char *vehicleName )` (bg_vehicleLoad.c:1598) — one-liner
/// wrapper over [`VEH_VehicleIndexForName`]. No oracle.
///
/// # Safety
/// `vehicleName` a valid NUL-terminated C string (forwarded unchanged).
pub unsafe fn BG_VehicleGetIndex(vehicleName: *const c_char) -> c_int {
    VEH_VehicleIndexForName(vehicleName)
}

//We get the vehicle name passed in as modelname
//with a $ in front of it.
//we are expected to then get the model for the
//vehicle and stomp over modelname with it.
/// `void BG_GetVehicleModelName( char *modelname )` (bg_vehicleLoad.c:1607) — `modelname`
/// arrives as `$<vehName>`; resolve the vehicle via [`BG_VehicleGetIndex`] and overwrite
/// `modelname` in place with its `model` string. [`Com_Error`]s on [`VEHICLE_NONE`]. The C
/// `assert(modelname[0]=='$')` is NDEBUG-compiled-out in the retail build (dropped). No oracle.
///
/// # Safety
/// `modelname` must be a writable, NUL-terminated `$`-prefixed C string buffer large enough
/// to hold the resolved model name.
pub unsafe fn BG_GetVehicleModelName(modelname: *mut c_char) {
    let vehName = modelname.add(1);
    let vIndex = BG_VehicleGetIndex(vehName);
    // assert(modelname[0] == '$') -- NDEBUG, compiled out in the retail build.

    if vIndex == VEHICLE_NONE {
        Com_Error(
            ERR_DROP,
            &format!(
                "BG_GetVehicleModelName:  couldn't find vehicle {}",
                Sz(vehName)
            ),
        );
    }

    strcpy(
        modelname,
        (*(addr_of!(g_vehicleInfo) as *const vehicleInfo_t).add(vIndex as usize)).model,
    );
}

/// `void BG_GetVehicleSkinName( char *skinname )` (bg_vehicleLoad.c:1621) — same shape as
/// [`BG_GetVehicleModelName`] but for the `skin` string: resolve the `$`-prefixed vehicle,
/// then overwrite `skinname` with its `skin` (or set `skinname[0]=0` when the vehicle has no
/// skin). [`Com_Error`]s on [`VEHICLE_NONE`]; the `assert` is NDEBUG-dropped. No oracle.
///
/// # Safety
/// `skinname` must be a writable, NUL-terminated `$`-prefixed C string buffer large enough to
/// hold the resolved skin name.
pub unsafe fn BG_GetVehicleSkinName(skinname: *mut c_char) {
    let vehName = skinname.add(1);
    let vIndex = BG_VehicleGetIndex(vehName);
    // assert(skinname[0] == '$') -- NDEBUG, compiled out in the retail build.

    if vIndex == VEHICLE_NONE {
        Com_Error(
            ERR_DROP,
            &format!(
                "BG_GetVehicleSkinName:  couldn't find vehicle {}",
                Sz(vehName)
            ),
        );
    }

    let vInfo = &*(addr_of!(g_vehicleInfo) as *const vehicleInfo_t).add(vIndex as usize);
    if vInfo.skin.is_null() || *vInfo.skin == 0 {
        *skinname = 0;
    } else {
        strcpy(skinname, vInfo.skin);
    }
}

/// `BG_VehicleSetDefaults( vehicleInfo_t *vehicle )` (bg_vehicleLoad.c:718) — reset a
/// vehicle record to defaults. In the original the entire field-by-field default block
/// (the `vehicle->type = VH_SPEEDER; …` body) is inside a `/* … */` comment, so the
/// live function is just the leading `memset(vehicle, 0, sizeof(vehicleInfo_t))`.
///
/// # Safety
/// `vehicle` must point at a valid, writable [`vehicleInfo_t`].
pub unsafe fn BG_VehicleSetDefaults(vehicle: *mut vehicleInfo_t) {
    core::ptr::write_bytes(vehicle as *mut u8, 0, size_of::<vehicleInfo_t>());
    // The remainder of the C body (the `vehicle->speedMax = VEH_DEFAULT_SPEED_MAX;` …
    // default assignments) is entirely inside a `/* … */` block comment in the original,
    // so the memset above is the whole function.
}

/// `BG_VehicleClampData( vehicleInfo_t *vehicle )` (bg_vehicleLoad.c:821) — sanity-check
/// and clamp a parsed vehicle's data: `centerOfGravity` to `[-1, 1]` per axis and
/// `maxPassengers` to `[0, VEH_MAX_PASSENGERS]`.
///
/// # Safety
/// `vehicle` must point at a valid, writable [`vehicleInfo_t`].
pub unsafe fn BG_VehicleClampData(vehicle: *mut vehicleInfo_t) {
    //sanity check and clamp the vehicle's data
    for i in 0..3 {
        if (*vehicle).centerOfGravity[i] > 1.0 {
            (*vehicle).centerOfGravity[i] = 1.0;
        } else if (*vehicle).centerOfGravity[i] < -1.0 {
            (*vehicle).centerOfGravity[i] = -1.0;
        }
    }

    // Validate passenger max.
    if (*vehicle).maxPassengers > VEH_MAX_PASSENGERS as c_int {
        (*vehicle).maxPassengers = VEH_MAX_PASSENGERS as c_int;
    } else if (*vehicle).maxPassengers < 0 {
        (*vehicle).maxPassengers = 0;
    }
}

/// `BG_ParseVehicleParm( vehicleInfo_t *vehicle, char *parmName, char *pValue )`
/// (bg_vehicleLoad.c:848) — set one `.veh` field on `vehicle` from a `parmName`/`pValue`
/// text pair. Scans the global [`vehicleFields`] table (terminated by `ofs == -1`) for a
/// case-insensitive name match, then decodes `pValue` per the field's [`vehFieldType_t`]
/// and writes it at `f->ofs` bytes. Returns `qtrue` on a recognized parm, `qfalse` if no
/// field matched.
///
/// The value-decode switch is **character-identical** to the oracle-verified
/// [`BG_ParseVehWeaponParm`] (only the table iterated differs); see that function for the
/// build-config notes (retail `QAGAME`: `VF_LSTRING` → [`BG_Alloc`]; `VF_MODEL`/`VF_EFFECT`/
/// `VF_SOUND` → [`G_ModelIndex`]/[`G_EffectIndex`]/[`G_SoundIndex`]; the `_CLIENT`/`VF_SHADER`/
/// `VF_SHADER_NOMIP` arms are `#ifndef QAGAME` no-ops). `VF_LSTRING` here allocates **128**
/// bytes (the vehicle parser), not the weapon parser's 1024.
///
/// **Faithful C bug preserved (bg_vehicleLoad.c:894):** the `VF_VECTOR` arm writes the
/// parsed vec3 through `vehWeaponFields[i].ofs` — the *vehicle-weapon* table indexed by the
/// *vehicle*-table loop counter `i` — rather than `vehicleFields[i].ofs`. It is **benign by
/// coincidence**: the lone `VF_VECTOR` row is `centerOfGravity` at `i == 8`, and both
/// `vehWeaponFields[8]` (`iImpactFX`) and `vehicleFields[8]` (`centerOfGravity`) sit at byte
/// offset 36, so the vector still lands exactly on `centerOfGravity`. Reproduced verbatim
/// anyway. See `DEVIATIONS.md`.
///
/// # Safety
/// `vehicle` must point at a valid [`vehicleInfo_t`]; `parmName`/`pValue` valid
/// NUL-terminated C strings — the contract the `.veh` parser ([`VEH_LoadVehicle`]) upholds.
pub unsafe fn BG_ParseVehicleParm(
    vehicle: *mut vehicleInfo_t,
    parmName: *const c_char,
    pValue: *const c_char,
) -> qboolean {
    let b = vehicle as *mut u8;
    let mut value = [0 as c_char; 1024];

    Q_strncpyz(value.as_mut_ptr(), pValue, value.len() as c_int);

    let fields = addr_of!(vehicleFields) as *const vehField_t;
    let vehWeapFields = addr_of!(vehWeaponFields) as *const vehField_t;

    // Loop through possible parameters
    let mut i: usize = 0;
    while (*fields.add(i)).ofs != -1 {
        let f = &*fields.add(i);
        if Q_stricmp(f.name, parmName) == 0 {
            // found it
            match f.r#type {
                VF_IGNORE => {}
                VF_INT => {
                    *(b.add(f.ofs as usize) as *mut c_int) = atoi(value.as_ptr());
                }
                VF_FLOAT => {
                    *(b.add(f.ofs as usize) as *mut f32) = atof(value.as_ptr()) as f32;
                }
                VF_LSTRING => {
                    // string on disk, pointer in memory, TAG_LEVEL
                    if (*(b.add(f.ofs as usize) as *mut *mut c_char)).is_null() {
                        //just use 128 bytes in case we want to write over the string
                        let p = BG_Alloc(128) as *mut c_char; //BG_Alloc(strlen(value))
                        *(b.add(f.ofs as usize) as *mut *mut c_char) = p;
                        strcpy(p, value.as_ptr());
                    }
                }
                VF_VECTOR => {
                    let mut vec: vec3_t = [0.0; 3];
                    #[cfg(feature = "vm")]
                    let _iFieldsRead = sscanf(
                        value.as_ptr(),
                        c"%f %f %f".as_ptr(),
                        &[
                            addr_of_mut!(vec[0]) as *mut c_void,
                            addr_of_mut!(vec[1]) as *mut c_void,
                            addr_of_mut!(vec[2]) as *mut c_void,
                        ],
                    );
                    #[cfg(not(feature = "vm"))]
                    let _iFieldsRead = sscanf(
                        value.as_ptr(),
                        c"%f %f %f".as_ptr(),
                        addr_of_mut!(vec[0]),
                        addr_of_mut!(vec[1]),
                        addr_of_mut!(vec[2]),
                    );
                    // assert(_iFieldsRead==3) -- NDEBUG, compiled out in the retail build.
                    if _iFieldsRead != 3 {
                        Com_Printf("^3BG_ParseVehicleParm: VEC3 sscanf() failed to read 3 floats ('angle' key bug?)\n");
                    }
                    // FAITHFUL C BUG (bg_vehicleLoad.c:894-896): the store uses
                    // `vehWeaponFields[i].ofs`, NOT `vehicleFields[i].ofs`. See the doc comment.
                    let wf = &*vehWeapFields.add(i);
                    let p = b.add(wf.ofs as usize) as *mut f32;
                    *p.add(0) = vec[0];
                    *p.add(1) = vec[1];
                    *p.add(2) = vec[2];
                }
                VF_BOOL => {
                    *(b.add(f.ofs as usize) as *mut qboolean) =
                        (atof(value.as_ptr()) != 0.0) as qboolean;
                }
                VF_VEHTYPE => {
                    let vehType = GetIDForString(
                        addr_of!(VehicleTable) as *const stringID_table_t,
                        value.as_ptr(),
                    ) as vehicleType_t;
                    *(b.add(f.ofs as usize) as *mut vehicleType_t) = vehType;
                }
                VF_ANIM => {
                    let anim = GetIDForString(
                        addr_of!(animTable) as *const stringID_table_t,
                        value.as_ptr(),
                    );
                    *(b.add(f.ofs as usize) as *mut c_int) = anim;
                }
                VF_WEAPON => {
                    // take string, resolve into index into VehWeaponParms
                    *(b.add(f.ofs as usize) as *mut c_int) =
                        VEH_VehWeaponIndexForName(value.as_ptr());
                }
                VF_MODEL => {
                    // take the string, get the G_ModelIndex
                    *(b.add(f.ofs as usize) as *mut c_int) =
                        G_ModelIndex(&CStr::from_ptr(value.as_ptr()).to_string_lossy());
                }
                VF_EFFECT => {
                    // take the string, get the G_EffectIndex
                    *(b.add(f.ofs as usize) as *mut c_int) =
                        G_EffectIndex(&CStr::from_ptr(value.as_ptr()).to_string_lossy());
                }
                VF_SOUND => {
                    // take the string, get the G_SoundIndex
                    *(b.add(f.ofs as usize) as *mut c_int) =
                        G_SoundIndex(&CStr::from_ptr(value.as_ptr()).to_string_lossy());
                }
                // QAGAME no-ops: the `_CLIENT` variants and VF_SHADER/VF_SHADER_NOMIP are
                // `#ifndef QAGAME` / commented-out, so the server build does nothing here.
                VF_MODEL_CLIENT | VF_EFFECT_CLIENT | VF_SHADER | VF_SHADER_NOMIP
                | VF_SOUND_CLIENT => {}
                // default: unknown type
                _ => return QFALSE,
            }
            break;
        }
        i += 1;
    }
    if (*fields.add(i)).ofs == -1 {
        QFALSE
    } else {
        QTRUE
    }
}

// so cgame can assign the function pointer for the vehicle attachment without having to
// bother with all the other funcs that don't really exist cgame-side.
/// `void AttachRidersGeneric( Vehicle_t *pVeh )` (bg_vehicleLoad.c:1651). Snaps the
/// vehicle's pilot onto the parent's `*driver` bolt tag: adds the bolt, reads the bolt
/// matrix off the parent's ghoul2 instance with a yaw-only angle, and pulls the origin
/// out of it into the pilot's player-state origin. No oracle — calls ghoul2 traps + reads
/// the `level`-backed `BG_GetTime`.
///
/// `_JK2MP && !WE_ARE_IN_THE_UI` build path (the retail MP game module). The shared
/// `bgEntity_t` accessors (`playerState`/`ghoul2`/`modelScale`) match the C `extern`
/// forward-decls of `BG_GetTime`/`trap_G2API_AddBolt`/`trap_G2API_GetBoltMatrix` above it.
///
/// # Safety
/// `pVeh` must point to a valid `Vehicle_t`. When `m_pPilot` is non-null, both `m_pPilot`
/// and `m_pParentEntity` (and their `playerState`/`ghoul2`) must be valid.
pub unsafe fn AttachRidersGeneric(pVeh: *mut Vehicle_t) {
    // If we have a pilot, attach him to the driver tag.
    if !(*pVeh).m_pPilot.is_null() {
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut yawOnlyAngles: vec3_t = [0.0; 3];
        let parent: *mut bgEntity_t = (*pVeh).m_pParentEntity;
        let pilot: *mut bgEntity_t = (*pVeh).m_pPilot;
        let crotchBolt: c_int = trap::G2API_AddBolt((*parent).ghoul2, 0, "*driver");

        debug_assert!(!(*parent).playerState.is_null());

        VectorSet(
            &mut yawOnlyAngles,
            0.0,
            (*(*parent).playerState).viewangles[YAW],
            0.0,
        );

        // Get the driver tag.
        trap::G2API_GetBoltMatrix(
            (*parent).ghoul2,
            0,
            crotchBolt,
            &mut boltMatrix,
            &yawOnlyAngles,
            &(*(*parent).playerState).origin,
            BG_GetTime(),
            core::ptr::null_mut(),
            &(*parent).modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN, &mut (*(*pilot).playerState).origin);
    }
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::codemp::game::g_mem::POOL_LOCK;
    use crate::oracle::{
        self, jka_BG_ParseVehWeaponParm, jka_BG_VehicleClampData, jka_vehicleFields_count,
        jka_vehicleFields_offsets,
    };
    use core::ptr::addr_of;
    use std::ffi::CString;

    /// Reset the module's vehicle-load globals to their initial state. The PC source
    /// removed `BG_ClearVehicleLoadInfo` (the engine cleared this state itself), so the
    /// tests inline the equivalent reset locally.
    ///
    /// SAFETY: callers hold `POOL_LOCK`; raw-pointer writes avoid `static_mut_refs`.
    unsafe fn reset_vehicle_load_globals() {
        *addr_of_mut!(numVehicleWeapons) = 1;
        *addr_of_mut!(numVehicles) = 0;
        core::ptr::write_bytes(
            addr_of_mut!(g_vehWeaponInfo) as *mut u8,
            0,
            size_of::<[vehWeaponInfo_t; MAX_VEH_WEAPONS]>(),
        );
        core::ptr::write_bytes(
            addr_of_mut!(g_vehicleInfo) as *mut u8,
            0,
            size_of::<[vehicleInfo_t; MAX_VEHICLES]>(),
        );
    }

    /// Element-wise parity of the `vehicleFields[]` offset column against an
    /// independent C transcription built with the real `VFOFS()` macro -- catches an
    /// index/field/arithmetic slip in the nested-array helpers on either side.
    #[test]
    fn vehicle_fields_offsets_match_c() {
        // SAFETY: `vehicleFields` is a read-only static table; raw-pointer reads avoid
        // forming a reference to the `static mut` (no `static_mut_refs`).
        unsafe {
            let n = jka_vehicleFields_count() as usize;
            let rust = &*addr_of!(vehicleFields);
            assert_eq!(n, rust.len(), "vehicleFields row count");
            let c = core::slice::from_raw_parts(jka_vehicleFields_offsets(), n);
            for i in 0..n {
                assert_eq!(rust[i].ofs, c[i], "vehicleFields[{i}].ofs");
            }
        }
    }

    /// Drives the Rust `VehicleTable` through the authentic C `GetIDForString` /
    /// `GetStringForID` (oracle, real q_shared.c) -- confirms each `ENUM2STRING` row
    /// pairs the right name with the right id, and that the `{ 0, -1 }` terminator
    /// bounds the scan as C expects.
    #[test]
    fn vehicle_table_lookups_match_c() {
        // `addr_of!` takes the address of the `static mut` without forming a reference
        // to it (no `static_mut_refs`); creating the raw pointer is safe, so no `unsafe`.
        let tbl = addr_of!(VehicleTable) as *const stringID_table_t;

        // name -> id, including a miss and a case-fold (GetIDForString uses Q_stricmp).
        let cases: &[(&CStr, vehicleType_t)] = &[
            (c"VH_NONE", VH_NONE),
            (c"VH_WALKER", VH_WALKER),
            (c"VH_FIGHTER", VH_FIGHTER),
            (c"VH_SPEEDER", VH_SPEEDER),
            (c"VH_ANIMAL", VH_ANIMAL),
            (c"VH_FLIER", VH_FLIER),
            (c"vh_flier", VH_FLIER),
            (c"VH_MISSING", -1),
            (c"", -1),
        ];
        for &(name, expect) in cases {
            let o = unsafe { oracle::GetIDForString(tbl, name.as_ptr()) };
            assert_eq!(o, expect, "GetIDForString({name:?})");
        }

        // id -> name, round-tripped back through GetIDForString to compare values
        // (the returned pointers index the same table, so they match by construction;
        // re-looking-up the C string is the meaningful check).
        for id in [VH_NONE, VH_WALKER, VH_FIGHTER, VH_SPEEDER, VH_ANIMAL, VH_FLIER] {
            let s = unsafe { oracle::GetStringForID(tbl, id) };
            assert!(!s.is_null(), "GetStringForID({id}) null");
            let back = unsafe { oracle::GetIDForString(tbl, s) };
            assert_eq!(back, id, "GetStringForID({id}) round-trip");
        }
    }

    /// Parity for `BG_ParseVehWeaponParm`: drive every field type the `.vwp`
    /// (`vehWeaponFields[]`) table actually exercises -- `VF_LSTRING` (`name`), `VF_BOOL`
    /// (`projectile`/`hasGravity`/`explodeOnExpire`), `VF_FLOAT` (`speed`/`g2MarkSize`),
    /// `VF_INT` (`damage`/`lockOnTime`) -- plus the QAGAME no-op arms (`VF_EFFECT_CLIENT`
    /// `muzzleFX`, `VF_MODEL_CLIENT` `model`, `VF_SHADER` `g2MarkShader`, `VF_SOUND_CLIENT`
    /// `loopSound`) and an unknown key, through the same parm stream into a Rust and a
    /// verbatim-C `vehWeaponInfo_t`, then compare. The `name` `VF_LSTRING` slot holds a
    /// freshly-allocated `char *` (a different address per side), so its pointer is compared
    /// by pointed-to content; everything else byte-for-byte. The return value (`qtrue`
    /// match / `qfalse` miss) is checked per call. Values are exactly `f32`-representable so
    /// the `vm` `atof` shim agrees with libc.
    #[test]
    fn veh_weapon_parm_matches_oracle() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // (key, value, expected return qtrue=1 / qfalse=0)
        let kvs: &[(&str, &str, c_int)] = &[
            ("name", "concussion", QTRUE),
            ("projectile", "1", QTRUE),
            ("hasGravity", "0", QTRUE),
            ("explodeOnExpire", "1", QTRUE),
            ("speed", "8000", QTRUE),
            ("g2MarkSize", "1.5", QTRUE),
            ("damage", "-100", QTRUE),
            ("lockOnTime", "2500", QTRUE),
            ("muzzleFX", "effect/path", QTRUE),   // VF_EFFECT_CLIENT -> no-op (QAGAME)
            ("model", "models/foo.md3", QTRUE),    // VF_MODEL_CLIENT  -> no-op
            ("g2MarkShader", "gfx/mark", QTRUE),   // VF_SHADER        -> no-op
            ("loopSound", "sound/loop.wav", QTRUE), // VF_SOUND_CLIENT -> no-op
            ("nosuchkey", "x", QFALSE),            // unmatched -> qfalse, struct untouched
        ];

        // SAFETY: identical `vehWeaponInfo_t` layout (104 bytes, 64-bit asserted) both
        // sides; the C oracle takes the struct as a raw byte pointer.
        unsafe {
            let mut rust_vw: vehWeaponInfo_t = core::mem::zeroed();
            let mut orac_vw: vehWeaponInfo_t = core::mem::zeroed();
            let sz = size_of::<vehWeaponInfo_t>();

            for &(k, v, expect) in kvs {
                let key = CString::new(k).unwrap();
                let val = CString::new(v).unwrap();
                let r = BG_ParseVehWeaponParm(
                    addr_of_mut!(rust_vw),
                    key.as_ptr(),
                    val.as_ptr(),
                );
                let o = jka_BG_ParseVehWeaponParm(
                    addr_of_mut!(orac_vw) as *mut u8,
                    key.as_ptr(),
                    val.as_ptr(),
                );
                assert_eq!(r, o, "return mismatch for key '{k}'");
                assert_eq!(r, expect, "unexpected return for key '{k}'");
            }

            // Everything past the `name` pointer (bytes 8..) is byte-exact.
            let rb = core::slice::from_raw_parts(addr_of!(rust_vw) as *const u8, sz);
            let ob = core::slice::from_raw_parts(addr_of!(orac_vw) as *const u8, sz);
            assert_eq!(rb[8..], ob[8..], "vehWeaponInfo_t scalar region");

            // The `name` VF_LSTRING slot: compare the strings the two pointers point at.
            assert!(!rust_vw.name.is_null() && !orac_vw.name.is_null(), "name allocated");
            assert_eq!(
                CStr::from_ptr(rust_vw.name).to_bytes(),
                CStr::from_ptr(orac_vw.name).to_bytes(),
                "name VF_LSTRING content",
            );
        }
    }

    /// Behavioral round-trip for `VEH_LoadVehWeapon` / `VEH_VehWeaponIndexForName` -- the
    /// COM-parse glue over the global `VehWeaponParms` buffer + `g_vehWeaponInfo[]` /
    /// `numVehicleWeapons` state (no separate C oracle; the field decode it delegates to is
    /// oracle-verified above, and `COM_ParseExt`/`SkipBracedSection`/`SkipRestOfLine` are
    /// oracle-verified in `q_shared`). Loads two weapons out of a synthetic two-block
    /// buffer, checks the returned indices, the dedup-by-name fast path, parsed field
    /// values, and the null-name guard.
    #[test]
    fn veh_load_vehweapon_roundtrip() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let parms = b"\
concussion\n\
{\n\
name concussion\n\
projectile 1\n\
speed 8000\n\
damage 100\n\
}\n\
laser\n\
{\n\
name laser\n\
speed 16000\n\
}\n\0";

        // SAFETY: single-threaded under POOL_LOCK; resets and drives the module globals
        // exactly as the engine's vehicle-weapon load path does.
        unsafe {
            reset_vehicle_load_globals(); // numVehicleWeapons = 1, zero g_vehWeaponInfo
            let dst = addr_of_mut!(VehWeaponParms) as *mut u8;
            core::ptr::copy_nonoverlapping(parms.as_ptr(), dst, parms.len());

            // First reference loads into slot 1 (slot 0 is the null/default).
            let i_laser = VEH_VehWeaponIndexForName(c"laser".as_ptr());
            assert_eq!(i_laser, 1, "laser loaded at index 1");
            assert_eq!(*addr_of!(numVehicleWeapons), 2, "numVehicleWeapons after first load");
            let laser = &*(addr_of!(g_vehWeaponInfo) as *const vehWeaponInfo_t).add(1);
            assert_eq!(CStr::from_ptr(laser.name).to_bytes(), b"laser", "laser name");
            assert_eq!(laser.fSpeed, 16000.0, "laser speed");

            // Re-reference hits the dedup-by-name fast path (no new load).
            let i_again = VEH_VehWeaponIndexForName(c"laser".as_ptr());
            assert_eq!(i_again, 1, "laser dedup returns same index");
            assert_eq!(*addr_of!(numVehicleWeapons), 2, "no extra load on dedup");

            // A second distinct weapon loads into slot 2, decoded by BG_ParseVehWeaponParm.
            let i_conc = VEH_VehWeaponIndexForName(c"concussion".as_ptr());
            assert_eq!(i_conc, 2, "concussion loaded at index 2");
            assert_eq!(*addr_of!(numVehicleWeapons), 3, "numVehicleWeapons after second load");
            let conc = &*(addr_of!(g_vehWeaponInfo) as *const vehWeaponInfo_t).add(2);
            assert_eq!(CStr::from_ptr(conc.name).to_bytes(), b"concussion", "concussion name");
            assert_eq!(conc.bIsProjectile, QTRUE, "concussion projectile");
            assert_eq!(conc.fSpeed, 8000.0, "concussion speed");
            assert_eq!(conc.iDamage, 100, "concussion damage");

            // (The null-name / not-found guards print via Com_Printf -> engine syscall,
            // which the unit-test harness has no dllEntry pointer for, so they are not
            // exercised here -- that surface is trap I/O, outside the oracle scope.)

            reset_vehicle_load_globals(); // leave the globals clean for other tests
        }
    }

    /// `BG_VehicleSetDefaults` is a bare `memset(vehicle, 0, sizeof(vehicleInfo_t))` (the
    /// rest of the C body is block-commented), so the behavior is "zero the whole record."
    /// Dirty a struct, reset it, confirm every byte is zero.
    #[test]
    fn veh_set_defaults_zeroes() {
        // SAFETY: a local `vehicleInfo_t` written through raw pointers; `BG_VehicleSetDefaults`
        // only memsets it.
        unsafe {
            let mut v: vehicleInfo_t = core::mem::zeroed();
            // Scribble non-zero bytes across the whole record.
            let sz = size_of::<vehicleInfo_t>();
            core::ptr::write_bytes(addr_of_mut!(v) as *mut u8, 0xAB, sz);

            BG_VehicleSetDefaults(addr_of_mut!(v));

            let bytes = core::slice::from_raw_parts(addr_of!(v) as *const u8, sz);
            assert!(bytes.iter().all(|&b| b == 0), "vehicleInfo_t fully zeroed");
        }
    }

    /// Bit-exact parity for `BG_VehicleClampData` against the verbatim C
    /// (`jka_BG_VehicleClampData`): run both on identical `vehicleInfo_t` bytes across a
    /// spread of `centerOfGravity` / `maxPassengers` inputs (over/under both bounds and
    /// in-range), then compare the whole struct byte-for-byte.
    #[test]
    fn veh_clamp_data_matches_oracle() {
        // (centerOfGravity, maxPassengers): exercise both clamp directions + pass-through.
        let cases: &[([f32; 3], c_int)] = &[
            ([2.0, -3.0, 0.5], 15),    // cog: high/low/in; pass: over max -> 10
            ([-1.5, 1.25, 0.0], -5),   // pass: negative -> 0
            ([1.0, -1.0, 0.999], 7),   // boundary + in-range pass-through
            ([0.0, 0.0, 0.0], 0),      // all no-ops
        ];

        // SAFETY: identical `vehicleInfo_t` layout (64-bit asserted in bg_vehicles_h) both
        // sides; the C oracle takes the struct as a raw byte pointer. Both structs start
        // fully zeroed, so the function-pointer tail compares equal.
        unsafe {
            let sz = size_of::<vehicleInfo_t>();
            for &(cog, maxp) in cases {
                let mut rust_v: vehicleInfo_t = core::mem::zeroed();
                let mut orac_v: vehicleInfo_t = core::mem::zeroed();
                rust_v.centerOfGravity = cog;
                rust_v.maxPassengers = maxp;
                orac_v.centerOfGravity = cog;
                orac_v.maxPassengers = maxp;

                BG_VehicleClampData(addr_of_mut!(rust_v));
                jka_BG_VehicleClampData(addr_of_mut!(orac_v) as *mut u8);

                let rb = core::slice::from_raw_parts(addr_of!(rust_v) as *const u8, sz);
                let ob = core::slice::from_raw_parts(addr_of!(orac_v) as *const u8, sz);
                assert_eq!(rb, ob, "BG_VehicleClampData bytes for cog={cog:?} maxp={maxp}");
            }
        }
    }

    /// Behavioral coverage for `BG_ParseVehicleParm` -- the parser-specific surface beyond
    /// the value-decode switch (which is character-identical to the oracle-verified
    /// [`BG_ParseVehWeaponParm`], and whose `vehicleFields[]` offsets are independently
    /// oracle-verified in `vehicle_fields_offsets_match_c`). Drives one representative key
    /// per deterministic arm into a zeroed `vehicleInfo_t`, asserts the decoded value at the
    /// right field, checks the `ofs == -1` loop termination (unknown key -> `qfalse`), and
    /// pins the faithful (benign) `VF_VECTOR` offset bug.
    #[test]
    fn veh_parse_vehicle_parm() {
        let _guard = POOL_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        // SAFETY: a local `vehicleInfo_t` driven through the same parser the `.veh` loader
        // uses; `BG_Alloc` (VF_LSTRING) is serialized by POOL_LOCK.
        unsafe {
            let mut v: vehicleInfo_t = core::mem::zeroed();
            let parse = |v: *mut vehicleInfo_t, k: &CStr, val: &CStr| -> c_int {
                BG_ParseVehicleParm(v, k.as_ptr(), val.as_ptr())
            };

            // VF_INT
            assert_eq!(parse(addr_of_mut!(v), c"mass", c"200"), QTRUE, "mass return");
            assert_eq!(v.mass, 200, "VF_INT mass");
            // VF_FLOAT (exactly f32-representable so the vm atof shim agrees)
            assert_eq!(parse(addr_of_mut!(v), c"speedMax", c"800.5"), QTRUE, "speedMax return");
            assert_eq!(v.speedMax, 800.5, "VF_FLOAT speedMax");
            // VF_BOOL
            assert_eq!(parse(addr_of_mut!(v), c"turnWhenStopped", c"1"), QTRUE, "bool return");
            assert_eq!(v.turnWhenStopped, QTRUE, "VF_BOOL turnWhenStopped");
            // VF_VEHTYPE (GetIDForString over VehicleTable, itself oracle-verified)
            assert_eq!(parse(addr_of_mut!(v), c"type", c"VH_FIGHTER"), QTRUE, "vehtype return");
            assert_eq!(v.r#type, VH_FIGHTER, "VF_VEHTYPE type");
            // VF_LSTRING (BG_Alloc'd copy)
            assert_eq!(parse(addr_of_mut!(v), c"name", c"swoop"), QTRUE, "name return");
            assert!(!v.name.is_null(), "VF_LSTRING allocated");
            assert_eq!(CStr::from_ptr(v.name).to_bytes(), b"swoop", "VF_LSTRING name");

            // VF_VECTOR: the faithful bug stores via vehWeaponFields[8].ofs == 36, which
            // equals vehicleFields[8].ofs (centerOfGravity) == 36 -- so it lands correctly.
            // Values are exactly f32-representable.
            assert_eq!(
                parse(addr_of_mut!(v), c"centerOfGravity", c"0.5 0.25 -0.75"),
                QTRUE,
                "vector return"
            );
            assert_eq!(v.centerOfGravity, [0.5, 0.25, -0.75], "VF_VECTOR centerOfGravity (benign bug)");

            // Loop terminates on ofs == -1: an unknown key returns qfalse.
            assert_eq!(parse(addr_of_mut!(v), c"nosuchkey", c"x"), QFALSE, "unknown key -> qfalse");
        }
    }
}
