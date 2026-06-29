// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

use core::ffi::{c_int, c_char, c_void};

// Type aliases to match C types
#[allow(non_camel_case_types)]
pub type qboolean = c_int;

#[allow(non_camel_case_types)]
pub type vec3_t = [f32; 3];

#[allow(non_camel_case_types)]
pub type mdxaBone_t = [f32; 16];

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(C)]
pub enum distance_e {
    DIST_MELEE = 0,
    DIST_LONG = 1,
}

// Game entity type - minimal definition for fields used in this file
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub ghoul2: [*mut c_void; 16], // G2 model array (array of void pointers)
    pub playerModel: c_int,
    pub currentOrigin: vec3_t,
    pub currentAngles: vec3_t,
    pub enemy: *mut gentity_t,
    pub locationDamage: [c_int; 8],
    pub genericBolt1: c_int,
    pub genericBolt2: c_int,
    // ... more fields exist but not used in this file
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub modelScale: vec3_t,
    // ... more fields not needed for this file
}

const MIN_MELEE_RANGE: c_int = 640;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const TURN_OFF: c_int = 0x00000100; //G2SURFACEFLAG_NODESCENDANTS

const LEFT_ARM_HEALTH: c_int = 40;
const RIGHT_ARM_HEALTH: c_int = 40;

// NPC info structure - minimal definition for fields used in this file
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct npc_t {
    pub goalEntity: *mut gentity_t,
    pub combatMove: c_int,
    pub scriptFlags: c_int,
    // ... other fields not needed for this file
}

// External engine interface and functions
extern "C" {
    pub static mut NPCInfo: *mut npc_t;
    pub static mut NPC: *mut gentity_t;
    pub static mut ucmd: ucmd_t;
    pub static mut cg: cg_t;
    pub static mut level: level_t;

    pub fn G_SoundIndex(name: *const c_char) -> c_int;
    pub fn G_SoundOnEnt(
        self_: *mut gentity_t,
        channel: c_int,
        sound: *const c_char,
    );
    pub fn RegisterItem(item: *const c_void);
    pub fn FindItemForWeapon(weapon: c_int) -> *const c_void;
    pub fn G_EffectIndex(name: *const c_char) -> c_int;
    pub fn G_PlayEffect(
        fx: *const c_char,
        model: c_int,
        bolt: c_int,
        entity_num: c_int,
        origin: *const vec3_t,
    );
    pub fn G_PlayEffect_alt(fx: *const c_char, org: *const vec3_t, dir: *const vec3_t);

    pub fn TIMER_Done(ent: *mut gentity_t, label: *const c_char) -> qboolean;
    pub fn TIMER_Set(ent: *mut gentity_t, label: *const c_char, duration: c_int);
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;

    pub fn NPC_MoveToGoal(urgent: qboolean);
    pub fn NPC_FaceEnemy(force: qboolean);
    pub fn NPC_ClearLOS(ent: *const gentity_t) -> qboolean;
    pub fn NPC_ChangeWeapon(weapon: c_int);
    pub fn NPC_CheckEnemyExt() -> qboolean;
    pub fn NPC_CheckPlayerTeamStealth() -> qboolean;
    pub fn UpdateGoal() -> qboolean;
    pub fn NPC_UpdateAngles(allow_look: qboolean, allow_turn: qboolean);
    pub fn NPC_BSIdle();
    pub fn NPC_SetAnim(
        ent: *mut gentity_t,
        set_flags: c_int,
        anim: c_int,
        flags: c_int,
    );
    pub fn NPC_Pain(
        self_: *mut gentity_t,
        inflictor: *mut gentity_t,
        other: *mut gentity_t,
        point: *const vec3_t,
        damage: c_int,
        mod_: c_int,
    );

    pub fn DistanceHorizontalSquared(
        point1: *const vec3_t,
        point2: *const vec3_t,
    ) -> f32;
}

// Game interface struct for G2API functions
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct gameImport_t {
    // Stub - only declare the functions we need below
}

// G2 API function types
pub type G2API_GetBoltMatrix_t = unsafe extern "C" fn(
    ghoul2: *mut *mut c_void,
    model_index: c_int,
    bolt_index: c_int,
    bolt_matrix: *mut mdxaBone_t,
    angles: *const vec3_t,
    origin: *const vec3_t,
    time: c_int,
    model_list: *const c_void,
    model_scale: *const vec3_t,
);

pub type G2API_GiveMeVectorFromMatrix_t = unsafe extern "C" fn(
    matrix: mdxaBone_t,
    flags: c_int,
    vec: *mut vec3_t,
);

pub type G2API_AddBolt_t = unsafe extern "C" fn(
    ghoul2: *mut *mut c_void,
    surface_name: *const c_char,
) -> c_int;

pub type G2API_SetSurfaceOnOff_t = unsafe extern "C" fn(
    ghoul2: *mut *mut c_void,
    surface_name: *const c_char,
    flags: c_int,
);

pub type G2API_GetSurfaceRenderStatus_t = unsafe extern "C" fn(
    ghoul2: *mut *mut c_void,
    surface_name: *const c_char,
) -> c_int;

// Global game interface instance
extern "C" {
    pub static gi: GameInterface;
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct GameInterface {
    pub G2API_GetBoltMatrix: G2API_GetBoltMatrix_t,
    pub G2API_GiveMeVectorFromMatrix: G2API_GiveMeVectorFromMatrix_t,
    pub G2API_AddBolt: G2API_AddBolt_t,
    pub G2API_SetSurfaceOnOff: G2API_SetSurfaceOnOff_t,
    pub G2API_GetSurfaceRenderStatus: G2API_GetSurfaceRenderStatus_t,
    // ... more fields for other engine functions (truncated for this file)
}

// Stub types for engine structures
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct ucmd_t {
    pub buttons: c_int,
    // ... other fields not needed for this file
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct cg_t {
    pub time: c_int,
    // ... other fields not needed for this file
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct level_t {
    pub time: c_int,
    // ... other fields not needed for this file
}

// ORIGIN and directional constants for G2API
const ORIGIN: c_int = 1;
const NEGATIVE_Y: c_int = 2;

// Weapon and animation constants
const WP_NONE: c_int = 0;
const WP_ATST_MAIN: c_int = 1;
const WP_ATST_SIDE: c_int = 2;
const WP_BOWCASTER: c_int = 3;
const WP_ROCKET_LAUNCHER: c_int = 4;

const SETANIM_BOTH: c_int = 0;
const BOTH_STAND1: c_int = 1;
const SETANIM_FLAG_NORMAL: c_int = 0;

const BUTTON_ATTACK: c_int = 1;
const BUTTON_ALT_ATTACK: c_int = 2;
const BUTTON_WALKING: c_int = 4;

const CHAN_LESS_ATTEN: c_int = 1;

const SCF_CHASE_ENEMIES: c_int = 0x00000001;
const SCF_LOOK_FOR_ENEMIES: c_int = 0x00000002;

const HL_ARM_LT: c_int = 2;
const HL_ARM_RT: c_int = 3;

/*
-------------------------
NPC_ATST_Precache
-------------------------
*/
pub unsafe fn NPC_ATST_Precache() {
    G_SoundIndex(b"sound/chars/atst/atst_damaged1\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/atst/atst_damaged2\0".as_ptr() as *const c_char);

    RegisterItem(FindItemForWeapon(WP_ATST_MAIN)); //precache the weapon
    RegisterItem(FindItemForWeapon(WP_BOWCASTER)); //precache the weapon
    RegisterItem(FindItemForWeapon(WP_ROCKET_LAUNCHER)); //precache the weapon

    G_EffectIndex(b"env/med_explode2\0".as_ptr() as *const c_char);
    //	G_EffectIndex( "smaller_chunks" );
    G_EffectIndex(b"blaster/smoke_bolton\0".as_ptr() as *const c_char);
    G_EffectIndex(b"explosions/droidexplosion1\0".as_ptr() as *const c_char);
}

//-----------------------------------------------------------------
unsafe fn ATST_PlayEffect(self_: *mut gentity_t, boltID: c_int, fx: *const c_char) {
    if boltID >= 0 && !fx.is_null() && *fx != 0 {
        let mut boltMatrix: mdxaBone_t = [0.0; 16];
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        (gi.G2API_GetBoltMatrix)(
            &mut (*self_).ghoul2 as *mut [*mut c_void; 16] as *mut *mut c_void,
            (*self_).playerModel,
            boltID,
            &mut boltMatrix,
            &(*self_).currentAngles,
            &(*self_).currentOrigin,
            if cg.time != 0 { cg.time } else { level.time },
            core::ptr::null(),
            &(*self_).s.modelScale,
        );

        (gi.G2API_GiveMeVectorFromMatrix)(boltMatrix, ORIGIN, &mut org);
        (gi.G2API_GiveMeVectorFromMatrix)(boltMatrix, NEGATIVE_Y, &mut dir);

        G_PlayEffect_alt(fx, &org, &dir);
    }
}

/*
-------------------------
G_ATSTCheckPain

Called by NPC's and player in an ATST
-------------------------
*/

pub unsafe fn G_ATSTCheckPain(
    self_: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    let newBolt: c_int;

    if rand() & 1 != 0 {
        G_SoundOnEnt(
            self_,
            CHAN_LESS_ATTEN,
            b"sound/chars/atst/atst_damaged1\0".as_ptr() as *const c_char,
        );
    } else {
        G_SoundOnEnt(
            self_,
            CHAN_LESS_ATTEN,
            b"sound/chars/atst/atst_damaged2\0".as_ptr() as *const c_char,
        );
    }

    if hitLoc == HL_ARM_LT && (*self_).locationDamage[HL_ARM_LT as usize] > LEFT_ARM_HEALTH {
        if (*self_).locationDamage[hitLoc as usize] >= LEFT_ARM_HEALTH {
            // Blow it up?
            newBolt = (gi.G2API_AddBolt)(
                &mut (*self_).ghoul2 as *mut [*mut c_void; 16] as *mut *mut c_void,
                b"*flash3\0".as_ptr() as *const c_char,
            );
            if newBolt != -1 {
                //				G_PlayEffect( "small_chunks", self->playerModel, self->genericBolt1, self->s.number);
                ATST_PlayEffect(
                    self_,
                    (*self_).genericBolt1,
                    b"env/med_explode2\0".as_ptr() as *const c_char,
                );
                G_PlayEffect(
                    b"blaster/smoke_bolton\0".as_ptr() as *const c_char,
                    (*self_).playerModel,
                    newBolt,
                    (*self_).s.number,
                    point,
                );
            }

            (gi.G2API_SetSurfaceOnOff)(
                &mut (*self_).ghoul2 as *mut [*mut c_void; 16] as *mut *mut c_void,
                b"head_light_blaster_cann\0".as_ptr() as *const c_char,
                TURN_OFF,
            );
        }
    } else if hitLoc == HL_ARM_RT
        && (*self_).locationDamage[HL_ARM_RT as usize] > RIGHT_ARM_HEALTH
    {
        // Blow it up?
        if (*self_).locationDamage[hitLoc as usize] >= RIGHT_ARM_HEALTH {
            newBolt = (gi.G2API_AddBolt)(
                &mut (*self_).ghoul2 as *mut [*mut c_void; 16] as *mut *mut c_void,
                b"*flash4\0".as_ptr() as *const c_char,
            );
            if newBolt != -1 {
                //				G_PlayEffect( "small_chunks", self->playerModel, self->genericBolt2, self->s.number);
                ATST_PlayEffect(
                    self_,
                    (*self_).genericBolt2,
                    b"env/med_explode2\0".as_ptr() as *const c_char,
                );
                G_PlayEffect(
                    b"blaster/smoke_bolton\0".as_ptr() as *const c_char,
                    (*self_).playerModel,
                    newBolt,
                    (*self_).s.number,
                    point,
                );
            }

            (gi.G2API_SetSurfaceOnOff)(
                &mut (*self_).ghoul2 as *mut [*mut c_void; 16] as *mut *mut c_void,
                b"head_concussion_charger\0".as_ptr() as *const c_char,
                TURN_OFF,
            );
        }
    }
}
/*
-------------------------
NPC_ATST_Pain
-------------------------
*/
pub unsafe fn NPC_ATST_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    G_ATSTCheckPain(self_, other, point, damage, mod_, hitLoc);
    NPC_Pain(self_, inflictor, other, point, damage, mod_);
}

/*
-------------------------
ATST_Hunt
-------------------------`
*/
pub unsafe fn ATST_Hunt(visible: qboolean, advance: qboolean) {
    if (*NPCInfo).goalEntity.is_null() {
        //hunt
        (*NPCInfo).goalEntity = (*NPC).enemy;
    }

    (*NPCInfo).combatMove = 1; // qtrue

    NPC_MoveToGoal(1); // qtrue
}

/*
-------------------------
ATST_Ranged
-------------------------
*/
pub unsafe fn ATST_Ranged(visible: qboolean, advance: qboolean, altAttack: qboolean) {
    if TIMER_Done(NPC, b"atkDelay\0".as_ptr() as *const c_char) != 0 && visible != 0 {
        // Attack?
        TIMER_Set(
            NPC,
            b"atkDelay\0".as_ptr() as *const c_char,
            Q_irand(500, 3000),
        );

        if altAttack != 0 {
            ucmd.buttons |= BUTTON_ATTACK | BUTTON_ALT_ATTACK;
        } else {
            ucmd.buttons |= BUTTON_ATTACK;
        }
    }

    if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
        ATST_Hunt(visible, advance);
    }
}

/*
-------------------------
ATST_Attack
-------------------------
*/
pub unsafe fn ATST_Attack() {
    let mut altAttack: qboolean = 0; // qfalse
    let mut blasterTest: c_int;
    let mut chargerTest: c_int;
    let weapon: c_int;

    if NPC_CheckEnemyExt() == 0 {
        //!NPC->enemy )//
        (*NPC).enemy = core::ptr::null_mut();
        return;
    }

    NPC_FaceEnemy(1); // qtrue

    // Rate our distance to the target, and our visibilty
    let distance: f32 = DistanceHorizontalSquared(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);
    let distance_int: c_int = distance as c_int;
    let distRate: distance_e = if distance_int > MIN_MELEE_RANGE_SQR {
        distance_e::DIST_LONG
    } else {
        distance_e::DIST_MELEE
    };
    let visible: qboolean = NPC_ClearLOS(NPC);
    let advance: qboolean = if distance_int > MIN_DISTANCE_SQR { 1 } else { 0 }; // qboolean cast

    // If we cannot see our target, move to see it
    if visible == 0 {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            ATST_Hunt(visible, advance);
            return;
        }
    }

    // Decide what type of attack to do
    match distRate {
        distance_e::DIST_MELEE => {
            NPC_ChangeWeapon(WP_ATST_MAIN);
        }

        distance_e::DIST_LONG => {
            NPC_ChangeWeapon(WP_ATST_SIDE);

            // See if the side weapons are there
            blasterTest = (gi.G2API_GetSurfaceRenderStatus)(
                &mut (*NPC).ghoul2 as *mut [*mut c_void; 16] as *mut *mut c_void,
                b"head_light_blaster_cann\0".as_ptr() as *const c_char,
            );
            chargerTest = (gi.G2API_GetSurfaceRenderStatus)(
                &mut (*NPC).ghoul2 as *mut [*mut c_void; 16] as *mut *mut c_void,
                b"head_concussion_charger\0".as_ptr() as *const c_char,
            );

            // It has both side weapons
            if (blasterTest & TURN_OFF) == 0 && (chargerTest & TURN_OFF) == 0 {
                let weapon: c_int = Q_irand(0, 1); // 0 is blaster, 1 is charger (ALT SIDE)

                if weapon != 0 {
                    // Fire charger
                    altAttack = 1; // qtrue
                } else {
                    altAttack = 0; // qfalse
                }
            } else if (blasterTest & TURN_OFF) == 0 {
                // Blaster is on
                altAttack = 0; // qfalse
            } else if (chargerTest & TURN_OFF) == 0 {
                // Blaster is on
                altAttack = 1; // qtrue
            } else {
                NPC_ChangeWeapon(WP_NONE);
            }
        }
    }

    NPC_FaceEnemy(1); // qtrue

    ATST_Ranged(visible, advance, altAttack);
}

/*
-------------------------
ATST_Patrol
-------------------------
*/
pub unsafe fn ATST_Patrol() {
    if NPC_CheckPlayerTeamStealth() != 0 {
        NPC_UpdateAngles(1, 1); // qtrue, qtrue
        return;
    }

    //If we have somewhere to go, then do that
    if (*NPC).enemy.is_null() {
        if UpdateGoal() != 0 {
            ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(1); // qtrue
            NPC_UpdateAngles(1, 1); // qtrue, qtrue
        }
    }
}

/*
-------------------------
ATST_Idle
-------------------------
*/
pub unsafe fn ATST_Idle() {
    NPC_BSIdle();

    NPC_SetAnim(NPC, SETANIM_BOTH, BOTH_STAND1, SETANIM_FLAG_NORMAL);
}

/*
-------------------------
NPC_BSDroid_Default
-------------------------
*/
pub unsafe fn NPC_BSATST_Default() {
    if !(*NPC).enemy.is_null() {
        if (*NPCInfo).scriptFlags & SCF_CHASE_ENEMIES != 0 {
            (*NPCInfo).goalEntity = (*NPC).enemy;
        }
        ATST_Attack();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        ATST_Patrol();
    } else {
        ATST_Idle();
    }
}

// Stub for rand() function (typically from C stdlib)
extern "C" {
    pub fn rand() -> c_int;
}


// Stub fields for gentity_t (game entity)
// These would normally be defined in the game engine headers
// Accessing through raw pointers/offsets in actual implementation
// For this faithful port, we note the field access patterns:
// - ghoul2: model animation system
// - playerModel: model index
// - currentAngles, currentOrigin: position
// - s.modelScale: scale vector
// - locationDamage: per-location damage array
// - genericBolt1, genericBolt2: bolt indices
// - enemy: pointer to enemy entity
// - s.number: entity number
