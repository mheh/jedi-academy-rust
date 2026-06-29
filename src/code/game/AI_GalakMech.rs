// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use std::ptr::addr_of_mut;

////////////////////////////////////////////////////////////////////////////////////////
// RAVEN SOFTWARE - STAR WARS: JK III
//  (c) 2002 Activision
//
// April 3, 2003 - This file has been commandeered for use by AI vehicle pilots.
//
////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////
// Includes
////////////////////////////////////////////////////////////////////////////////////////
// Included in this translation:
// #include "g_headers.h"
// #include "b_local.h"
// #include "anims.h"
// #include "g_navigator.h"
// #include "g_Vehicles.h"
// #include "..\Ratl\vector_vs.h"

////////////////////////////////////////////////////////////////////////////////////////
// Defines
////////////////////////////////////////////////////////////////////////////////////////
const MAX_VEHICLES_REGISTERED: usize = 100;

const ATTACK_FWD: f32 = 0.95f32;
const ATTACK_SIDE: f32 = 0.20f32;
const AIM_SIDE: f32 = 0.60f32;
const FUTURE_PRED_DIST: f32 = 20.0f32;
const FUTURE_SIDE_DIST: f32 = 60.0f32;
const ATTACK_FLANK_SLOWING: f32 = 1000.0f32;
const RAM_DIST: f32 = 150.0f32;
const MIN_STAY_VIEWABLE_TIME: c_int = 20000;

////////////////////////////////////////////////////////////////////////////////////////
// Local stub for ratl::vector_vs<gentity_t*, MAX_VEHICLES_REGISTERED>
// Mirrors the fixed-size vector behavior used by the original C++ code
////////////////////////////////////////////////////////////////////////////////////////
struct VectorVs<T: Copy, const N: usize> {
    data: [Option<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> VectorVs<T, N> {
    fn new() -> Self {
        VectorVs {
            data: [None; N],
            len: 0,
        }
    }

    fn clear(&mut self) {
        for i in 0..self.len {
            self.data[i] = None;
        }
        self.len = 0;
    }

    fn push_back(&mut self, item: T) {
        if self.len < N {
            self.data[self.len] = Some(item);
            self.len += 1;
        }
    }

    fn full(&self) -> bool {
        self.len >= N
    }

    fn empty(&self) -> bool {
        self.len == 0
    }

    fn size(&self) -> usize {
        self.len
    }
}

impl<T: Copy, const N: usize> core::ops::Index<usize> for VectorVs<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if let Some(item) = self.data[index] {
            unsafe { &*((&item) as *const T) }
        } else {
            panic!("Index out of bounds or None value")
        }
    }
}

impl<T: Copy, const N: usize> core::ops::IndexMut<usize> for VectorVs<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(item) = &mut self.data[index] {
            unsafe { &mut *(item as *mut T) }
        } else {
            panic!("Index out of bounds or None value")
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Type stubs for entities and game structures
////////////////////////////////////////////////////////////////////////////////////////

pub type qboolean = c_int;
pub type vec3_t = [f32; 3];

#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
    pub pad: [u8; 2],
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: qboolean,
    pub startsolid: qboolean,
    pub fraction: f32,
    pub endpos: vec3_t,
    pub plane: cplane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
    pub localTime: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub commandTime: c_int,
    pub pm_type: c_int,
    pub bobCycle: c_int,
    pub pm_flags: c_int,
    pub pm_time: c_int,
    pub origin: vec3_t,
    pub velocity: vec3_t,
    pub weaponTime: c_int,
    pub gravity: c_int,
    pub speed: c_int,
    pub delta_angles: [c_int; 3],
    pub groundEntityNum: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub movementDir: c_int,
    pub grapplePoint: vec3_t,
    pub eFlags: c_int,
    pub eventSequence: c_int,
    pub events: [c_int; 2],
    pub eventParms: [c_int; 2],
    pub externalEvent: c_int,
    pub externalEventParm: c_int,
    pub clientNum: c_int,
    pub weapon: c_int,
    pub weaponstate: c_int,
    pub viewangles: vec3_t,
    pub viewheight: c_int,
    pub damageEvent: c_int,
    pub damageYaw: c_int,
    pub damagePitch: c_int,
    pub damageCount: c_int,
    pub generic1: c_int,
    pub loopSound: c_int,
    pub jumppad_ent: c_int,
    pub pm_qbKey: c_int,
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
}

#[repr(C)]
pub struct gNPC_t {
    pub greetEnt: *mut gentity_t,
    pub confusionTime: c_int,
    pub charmedTime: c_int,
    pub lastAvoidSteerSideDebouncer: c_int,
    pub desiredPitch: f32,
    pub desiredYaw: f32,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub pos: [f32; 3],
    pub angles: [f32; 3],
    pub otherEntityNum: c_int,
    pub otherEntityNum2: c_int,
    pub groundEntityNum: c_int,
    pub constantLight: c_int,
    pub loopSound: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    pub clientNum: c_int,
    pub frame: c_int,
    pub solid: c_int,
    pub event: c_int,
    pub eventParm: c_int,
    pub powerups: c_int,
    pub weapon: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub generic1: c_int,
    pub m_iVehicleNum: c_int,
}

#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub inuse: c_int,
    pub linkcount: c_int,
    pub client: *mut gclient_t,
    pub NPC: *mut gNPC_t,
    pub owner: *mut gentity_t,
    pub m_pVehicle: *mut Vehicle_t,
    pub currentOrigin: vec3_t,
    pub currentAngles: vec3_t,
    pub pos3: vec3_t,
    pub health: c_int,
    pub e_ThinkFunc: c_int,
    pub nextthink: c_int,
    pub resultspeed: f32,
}

#[repr(C)]
pub struct vehicleInfo_t {
    pub type_: c_int,
    pub soundFlyBy: c_int,
    pub soundFlyBy2: c_int,
    pub speedMax: f32,
}

#[repr(C)]
pub struct Vehicle_t {
    pub m_pVehicleInfo: *mut vehicleInfo_t,
    pub m_pParentEntity: *mut gentity_t,
    pub m_iTurboTime: c_int,
    pub m_ulFlags: c_int,
}

// Enum stub for Side values used in LRTest
#[repr(C)]
pub enum ESide {
    Side_Left = 0,
    Side_Right = 1,
}

// CVec3 vector class stub
#[repr(C)]
pub struct CVec3 {
    pub v: [f32; 3],
}

impl CVec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        CVec3 { v: [x, y, z] }
    }

    pub fn ScaleAdd(&mut self, dir: &CVec3, scale: f32) {
        self.v[0] += dir.v[0] * scale;
        self.v[1] += dir.v[1] * scale;
        self.v[2] += dir.v[2] * scale;
    }

    pub fn SafeNorm(&mut self) -> f32 {
        let len = (self.v[0] * self.v[0] + self.v[1] * self.v[1] + self.v[2] * self.v[2]).sqrt();
        if len > 0.0f32 {
            self.v[0] /= len;
            self.v[1] /= len;
            self.v[2] /= len;
        }
        len
    }

    pub fn Dot(&self, other: &CVec3) -> f32 {
        self.v[0] * other.v[0] + self.v[1] * other.v[1] + self.v[2] * other.v[2]
    }

    pub fn LRTest(&self, pos1: &CVec3, pos2: &CVec3) -> ESide {
        let cross_x = (pos2.v[0] - pos1.v[0]) * (self.v[1] - pos1.v[1]) - (pos2.v[1] - pos1.v[1]) * (self.v[0] - pos1.v[0]);
        if cross_x > 0.0f32 {
            ESide::Side_Right
        } else {
            ESide::Side_Left
        }
    }

    pub fn VecToAng(&self) {
        // Converts vector to angles - stub implementation
    }
}

impl core::ops::Index<usize> for CVec3 {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        &self.v[i]
    }
}

impl core::ops::IndexMut<usize> for CVec3 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        &mut self.v[i]
    }
}

impl core::ops::AddAssign for CVec3 {
    fn add_assign(&mut self, rhs: CVec3) {
        self.v[0] += rhs.v[0];
        self.v[1] += rhs.v[1];
        self.v[2] += rhs.v[2];
    }
}

impl core::ops::SubAssign for CVec3 {
    fn sub_assign(&mut self, rhs: CVec3) {
        self.v[0] -= rhs.v[0];
        self.v[1] -= rhs.v[1];
        self.v[2] -= rhs.v[2];
    }
}

impl core::ops::MulAssign<f32> for CVec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.v[0] *= rhs;
        self.v[1] *= rhs;
        self.v[2] *= rhs;
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Externs
////////////////////////////////////////////////////////////////////////////////////////
extern "C" {
    pub fn G_IsRidingVehicle(ent: *mut gentity_t) -> *mut Vehicle_t;
    pub fn G_SoundAtSpot(org: vec3_t, soundIndex: c_int, broadcast: qboolean);
    pub fn VEH_StartStrafeRam(pVeh: *mut Vehicle_t, Right: bool) -> bool;

    // Global data assumed to be defined elsewhere
    pub static mut g_entities: [gentity_t; 1024];
    pub static mut player: *mut gentity_t;
    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut gNPC_t;
    pub static mut ucmd: usercmd_t;
    pub static mut level: level_t;

    // External functions
    pub fn VectorCopy(src: vec3_t, dst: *mut vec3_t);
    pub fn AngleVectors(angles: vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut c_void);
    pub fn VectorMA(src: vec3_t, scale: f32, dir: vec3_t, dst: *mut vec3_t);
    pub fn VectorLength(v: vec3_t) -> f32;
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn VectorScale(src: vec3_t, scale: f32, dst: *mut vec3_t);
    pub fn Distance(src: vec3_t, dst: vec3_t) -> f32;
    pub fn DotProduct(src: vec3_t, dst: vec3_t) -> f32;
    pub fn fabsf(x: f32) -> f32;
    pub fn Q_flrand(min: f32, max: f32) -> f32;
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;
    pub fn AngleNormalize360(angle: f32) -> f32;
    pub fn TIMER_Set(ent: *mut gentity_t, label: *const u8, duration: c_int);
    pub fn TIMER_Done(ent: *mut gentity_t, label: *const u8) -> bool;
    pub fn TIMER_Exists(ent: *mut gentity_t, label: *const u8) -> bool;
    pub fn NPC_Use(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn NPC_ChangeWeapon(wp: c_int);
    pub fn NPC_UpdateAngles(doPitch: qboolean, doYaw: qboolean);
    pub fn WeaponThink(alt: bool);
    pub fn G_Sound(ent: *mut gentity_t, soundIndex: c_int);

    // namespace stubs
    pub fn STEER_Activate(ent: *mut gentity_t);
    pub fn STEER_Reached(ent: *mut gentity_t, goal: *mut gentity_t, dist: f32) -> bool;
    pub fn STEER_Persue(ent: *mut gentity_t, goal: *mut gentity_t, attackDist: f32, a: f32, b: f32, c: f32, d: bool);
    pub fn STEER_Stop(ent: *mut gentity_t);
    pub fn STEER_AvoidCollisions(ent: *mut gentity_t);
    pub fn STEER_DeActivate(ent: *mut gentity_t, ucmd: *mut usercmd_t);
    pub fn NAV_InSameRegion(ent1: *mut gentity_t, ent2: *mut gentity_t) -> bool;
    pub fn NAV_OnNeighboringPoints(ent1: *mut gentity_t, ent2: *mut gentity_t) -> bool;
    pub fn NAV_GoTo(ent: *mut gentity_t, goal: *mut gentity_t) -> bool;
}

#[repr(C)]
pub struct usercmd_t {
    pub serverTime: c_int,
    pub buttons: c_int,
    pub weapon: u8,
    pub forwardmove: i8,
    pub rightmove: i8,
    pub upmove: i8,
}

#[repr(C)]
pub struct level_t {
    pub time: c_int,
}

const ENTITYNUM_WORLD: c_int = 1023;

const BUTTON_ATTACK: c_int = 1;
const BUTTON_ALT_ATTACK: c_int = 2;
const BUTTON_VEH_SPEED: c_int = 4;

const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

const WP_NONE: c_int = 0;
const WP_BLASTER: c_int = 1;

const VH_SPEEDER: c_int = 0;
const VEH_OUTOFCONTROL: c_int = 1;
const VEH_SLIDEBREAKING: c_int = 2;
const VEH_STRAFERAM: c_int = 4;

const MASK_SHOT: c_int = 0;

const thinkF_G_FreeEntity: c_int = 0;

const EDGE_IMPACT_SAFE: c_int = 0;
const EDGE_IMPACT_POSSIBLE: c_int = 1;

////////////////////////////////////////////////////////////////////////////////////////
// Globals
////////////////////////////////////////////////////////////////////////////////////////
static mut mPilotViewTrace: trace_t = trace_t {
    allsolid: 0,
    startsolid: 0,
    fraction: 0.0f32,
    endpos: [0.0f32; 3],
    plane: cplane_t {
        normal: [0.0f32; 3],
        dist: 0.0f32,
        type_: 0,
        signbits: 0,
        pad: [0; 2],
    },
    surfaceFlags: 0,
    contents: 0,
    entityNum: 0,
    localTime: 0,
};

static mut mPilotViewTraceCount: c_int = 0;
static mut mActivePilotCount: c_int = 0;
static mut mRegistered: VectorVs<*mut gentity_t, MAX_VEHICLES_REGISTERED> = VectorVs {
    data: [None; MAX_VEHICLES_REGISTERED],
    len: 0,
};

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_Reset() {
    unsafe {
        mPilotViewTraceCount = 0;
        mActivePilotCount = 0;
        addr_of_mut!(mRegistered).as_mut().unwrap().clear();
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_ActivePilotCount() -> c_int {
    unsafe { mActivePilotCount }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_Update() {
    unsafe {
        mActivePilotCount = 0;
        addr_of_mut!(mRegistered).as_mut().unwrap().clear();

        let mut i = 0;
        while i < ENTITYNUM_WORLD {
            if g_entities[i as usize].inuse != 0
                && !g_entities[i as usize].client.is_null()
                && !g_entities[i as usize].NPC.is_null()
                && !(*g_entities[i as usize].NPC).greetEnt.is_null()
                && (*(*g_entities[i as usize].NPC).greetEnt).owner == addr_of_mut!(g_entities[i as usize])
            {
                mActivePilotCount += 1;
            }
            if g_entities[i as usize].inuse != 0
                && !g_entities[i as usize].client.is_null()
                && !g_entities[i as usize].m_pVehicle.is_null()
                && g_entities[i as usize].owner.is_null()
                && g_entities[i as usize].health > 0
                && (*(*g_entities[i as usize].m_pVehicle).m_pVehicleInfo).type_ == VH_SPEEDER
                && !addr_of_mut!(mRegistered).as_mut().unwrap().full()
            {
                addr_of_mut!(mRegistered).as_mut().unwrap().push_back(addr_of_mut!(g_entities[i as usize]));
            }

            i += 1;
        }

        if !player.is_null()
            && (*player).inuse != 0
            && TIMER_Done(player, b"FlybySoundArchitectureDebounce\0".as_ptr())
        {
            TIMER_Set(player, b"FlybySoundArchitectureDebounce\0".as_ptr(), 300);

            let pVeh = G_IsRidingVehicle(player);

            if !pVeh.is_null()
                && ((*(*pVeh).m_pVehicleInfo).soundFlyBy != 0 || (*(*pVeh).m_pVehicleInfo).soundFlyBy2 != 0)
                && VectorLength((*(*pVeh).m_pParentEntity).client.as_ref().unwrap().ps.velocity) > 500.0f32
            {
                let mut projectedPosition: vec3_t = [0.0f32; 3];
                let mut projectedDirection: vec3_t = [0.0f32; 3];
                let mut projectedRight: vec3_t = [0.0f32; 3];
                let mut anglesNoRoll: vec3_t = [0.0f32; 3];

                VectorCopy((*(*pVeh).m_pParentEntity).currentAngles, addr_of_mut!(anglesNoRoll));
                anglesNoRoll[2] = 0.0f32;
                AngleVectors(anglesNoRoll, addr_of_mut!(projectedDirection), addr_of_mut!(projectedRight), std::ptr::null_mut());

                VectorMA(
                    (*player).currentOrigin,
                    1.2f32,
                    (*(*pVeh).m_pParentEntity).client.as_ref().unwrap().ps.velocity,
                    addr_of_mut!(projectedPosition),
                );
                VectorMA(
                    projectedPosition,
                    Q_flrand(-200.0f32, 200.0f32),
                    projectedRight,
                    addr_of_mut!(projectedPosition),
                );

                gi_trace(
                    addr_of_mut!(mPilotViewTrace),
                    (*player).currentOrigin,
                    [0.0f32; 3],
                    [0.0f32; 3],
                    projectedPosition,
                    (*player).s.number,
                    MASK_SHOT,
                );

                if mPilotViewTrace.allsolid == 0
                    && mPilotViewTrace.startsolid == 0
                    && mPilotViewTrace.fraction < 0.99f32
                    && mPilotViewTrace.plane.normal[2] < 0.5f32
                    && DotProduct(projectedDirection, mPilotViewTrace.plane.normal) < -0.5f32
                {
                    TIMER_Set(player, b"FlybySoundArchitectureDebounce\0".as_ptr(), Q_irand(1000, 2000));

                    let mut soundFlyBy = (*(*pVeh).m_pVehicleInfo).soundFlyBy;
                    if (*(*pVeh).m_pVehicleInfo).soundFlyBy2 != 0 && (soundFlyBy == 0 || Q_irand(0, 1) == 0) {
                        soundFlyBy = (*(*pVeh).m_pVehicleInfo).soundFlyBy2;
                    }
                    G_SoundAtSpot(mPilotViewTrace.endpos, soundFlyBy, 1);
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_AnyVehiclesRegistered() -> bool {
    unsafe { !addr_of_mut!(mRegistered).as_ref().unwrap().empty() }
}

////////////////////////////////////////////////////////////////////////////////////////
// Vehicle Registration
//
// Any vehicles that can be ridden by NPCs should be registered here
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Vehicle_Register(_ent: *mut gentity_t) {
}

////////////////////////////////////////////////////////////////////////////////////////
// Vehicle Remove From The List Of Valid
////////////////////////////////////////////////////////////////////////////////////////
pub fn Vehicle_Remove(_ent: *mut gentity_t) {
}

////////////////////////////////////////////////////////////////////////////////////////
// Vehicle_Find
//
// Will look through all registered vehicles and choose the closest one that the given
// entity can get to.
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Vehicle_Find(ent: *mut gentity_t) -> *mut gentity_t {
    unsafe {
        let mut closest: *mut gentity_t = std::ptr::null_mut();
        let mut closestDist: f32 = 0.0f32;
        let mut curDist: f32;

        let registered = addr_of_mut!(mRegistered).as_ref().unwrap();
        let mut i = 0;
        while i < registered.size() {
            if (*registered[i]).owner.is_null() {
                curDist = Distance((*registered[i]).currentOrigin, (*ent).currentOrigin);
                if curDist < 1000.0f32 && (closest.is_null() || curDist < closestDist) {
                    if NAV_InSameRegion(ent, registered[i]) {
                        closest = registered[i];
                        closestDist = curDist;
                    }
                }
            }
            i += 1;
        }

        closest
    }
}

pub fn Pilot_Update_Enemy();
pub fn Pilot_Steer_Vehicle();
pub fn Pilot_Goto_Vehicle();

////////////////////////////////////////////////////////////////////////////////////////
// Pilot_MasterUpdate() - Master think function for Pilot NPCs
//
// Will return true if the character is either driving a vehicle or on his way to get
// onto one.
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_MasterUpdate() -> bool {
    unsafe {
        if NPC.is_null() || (*NPC).NPC.is_null() {
            return false;
        }

        if (*NPC).enemy.is_null() {
            // If Still On A Vehicle, Jump Off
            //---------------------------------
            if !(*NPCInfo).greetEnt.is_null() {
                ucmd.upmove = 128;

                if !(*NPCInfo).greetEnt.is_null()
                    && !(*(*NPCInfo).greetEnt).m_pVehicle.is_null()
                    && level.time < (*NPCInfo).confusionTime
                {
                    let pVeh = (*(*NPCInfo).greetEnt).m_pVehicle;
                    if ((*pVeh).m_ulFlags & VEH_OUTOFCONTROL) == 0 {
                        let parent = (*pVeh).m_pParentEntity;
                        let CurSpeed = VectorLength((*(*parent).client).ps.velocity);
                        (*(*pVeh).m_pVehicleInfo).StartDeathDelay(pVeh, 10000);
                        (*pVeh).m_ulFlags |= VEH_OUTOFCONTROL;
                        VectorScale((*(*parent).client).ps.velocity, 1.25f32, addr_of_mut!((*parent).pos3));
                        if CurSpeed < (*(*pVeh).m_pVehicleInfo).speedMax {
                            VectorNormalize(addr_of_mut!((*parent).pos3));
                            if fabsf((*parent).pos3[2]) < 0.25f32 {
                                VectorScale((*parent).pos3, (*(*pVeh).m_pVehicleInfo).speedMax * 1.25f32, addr_of_mut!((*parent).pos3));
                            } else {
                                VectorScale((*(*parent).client).ps.velocity, 1.25f32, addr_of_mut!((*parent).pos3));
                            }
                        }
                    }
                }

                if (*(*NPCInfo).greetEnt).owner == NPC {
                    return true;
                }
                (*NPCInfo).greetEnt = std::ptr::null_mut();
            }

            // Otherwise Nothing To See Here
            //-------------------------------
            return false;
        }

        // If We Already Have A Target Vehicle, Make Sure It Is Still Valid
        //------------------------------------------------------------------
        if !(*NPCInfo).greetEnt.is_null() {
            if (*(*NPCInfo).greetEnt).inuse == 0
                || (*(*NPCInfo).greetEnt).m_pVehicle.is_null()
                || (*(*(*NPCInfo).greetEnt).m_pVehicle).m_pVehicleInfo.is_null()
            {
                (*NPCInfo).greetEnt = Vehicle_Find(NPC);
            } else {
                if !(*(*NPCInfo).greetEnt).owner.is_null() && (*(*NPCInfo).greetEnt).owner != NPC {
                    (*NPCInfo).greetEnt = Vehicle_Find(NPC);
                }
            }
        }

        // If We Have An Enemy, Try To Find A Vehicle Nearby
        //---------------------------------------------------
        else {
            (*NPCInfo).greetEnt = Vehicle_Find(NPC);
        }

        // If No Vehicle Available, Continue As Usual
        //--------------------------------------------
        if (*NPCInfo).greetEnt.is_null() {
            return false;
        }

        if (*(*NPCInfo).greetEnt).owner == NPC {
            Pilot_Steer_Vehicle();
        } else {
            Pilot_Goto_Vehicle();
        }

        Pilot_Update_Enemy();
        true
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_Update_Enemy() {
    unsafe {
        if !TIMER_Exists(NPC, b"PilotRemoveTime\0".as_ptr()) {
            TIMER_Set(NPC, b"PilotRemoveTime\0".as_ptr(), MIN_STAY_VIEWABLE_TIME);
        }

        if TIMER_Done(NPC, b"NextPilotCheckEnemyTime\0".as_ptr()) {
            TIMER_Set(NPC, b"NextPilotCheckEnemyTime\0".as_ptr(), Q_irand(1000, 2000));
            if !(*NPC).enemy.is_null() && Distance((*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin) > 1000.0f32 {
                mPilotViewTraceCount += 1;
                gi_trace(
                    addr_of_mut!(mPilotViewTrace),
                    (*NPC).currentOrigin,
                    [0.0f32; 3],
                    [0.0f32; 3],
                    (*(*NPC).enemy).currentOrigin,
                    (*NPC).s.number,
                    MASK_SHOT,
                );

                if mPilotViewTrace.allsolid == 0
                    && mPilotViewTrace.startsolid == 0
                    && (mPilotViewTrace.entityNum == (*(*NPC).enemy).s.number
                        || mPilotViewTrace.entityNum == (*(*NPC).enemy).s.m_iVehicleNum)
                {
                    TIMER_Set(NPC, b"PilotRemoveTime\0".as_ptr(), MIN_STAY_VIEWABLE_TIME);
                }
            } else {
                TIMER_Set(NPC, b"PilotRemoveTime\0".as_ptr(), MIN_STAY_VIEWABLE_TIME);
            }
        }

        if TIMER_Done(NPC, b"PilotRemoveTime\0".as_ptr()) {
            if !(*NPCInfo).greetEnt.is_null() && (*(*NPCInfo).greetEnt).owner == NPC {
                (*(*NPCInfo).greetEnt).e_ThinkFunc = thinkF_G_FreeEntity;
                (*(*NPCInfo).greetEnt).nextthink = level.time;
            }
            (*NPC).e_ThinkFunc = thinkF_G_FreeEntity;
            (*NPC).nextthink = level.time;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_Goto_Vehicle() {
    unsafe {
        STEER_Activate(NPC);
        {
            if STEER_Reached(NPC, (*NPCInfo).greetEnt, 80.0f32) {
                NPC_Use((*NPCInfo).greetEnt, NPC, NPC);
            } else if NAV_OnNeighboringPoints(NPC, (*NPCInfo).greetEnt) {
                STEER_Persue(NPC, (*NPCInfo).greetEnt, 50.0f32, 0.0f32, 30.0f32, 0.0f32, true);
            } else {
                if !NAV_GoTo(NPC, (*NPCInfo).greetEnt) {
                    STEER_Stop(NPC);
                }
            }
        }
        STEER_AvoidCollisions(NPC);
        STEER_DeActivate(NPC, addr_of_mut!(ucmd));
        NPC_UpdateAngles(1, 1);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub fn Pilot_Steer_Vehicle() {
    unsafe {
        if NPC.is_null() || (*NPC).enemy.is_null() || (*(*NPC).enemy).client.is_null() {
            return;
        }

        // SETUP
        //=======
        // Setup Actor Data
        //------------------
        let mut ActorPos = CVec3 {
            v: (*NPC).currentOrigin,
        };
        let mut ActorAngles = CVec3 {
            v: (*NPC).currentAngles,
        };
        ActorAngles.v[2] = 0.0f32;
        let ActorVeh = (*NPCInfo).greetEnt.as_ref().unwrap().m_pVehicle;
        let ActorInTurbo = (*ActorVeh).m_iTurboTime > level.time;
        let ActorSpeed = if !ActorVeh.is_null() {
            VectorLength((*(*ActorVeh).m_pParentEntity).client.as_ref().unwrap().ps.velocity)
        } else {
            (*NPC).client.as_ref().unwrap().ps.speed as f32
        };

        // If my vehicle is spinning out of control, just hold on, we're going to die!!!!!
        //---------------------------------------------------------------------------------
        if !ActorVeh.is_null() && ((*ActorVeh).m_ulFlags & VEH_OUTOFCONTROL) != 0 {
            if (*NPC).client.as_ref().unwrap().ps.weapon != WP_NONE as u8 {
                NPC_ChangeWeapon(WP_NONE);
            }
            ucmd.buttons &= !BUTTON_ATTACK;
            ucmd.buttons &= !BUTTON_ALT_ATTACK;
            return;
        }

        let mut ActorDirection = CVec3 { v: [0.0f32; 3] };
        AngleVectors(ActorAngles.v, addr_of_mut!(ActorDirection.v), std::ptr::null_mut(), std::ptr::null_mut());

        let mut ActorFuturePos = ActorPos;
        ActorFuturePos.ScaleAdd(&ActorDirection, FUTURE_PRED_DIST);

        let mut ActorDoTurbo = false;
        let mut ActorAccelerate = false;
        let mut ActorAimAtTarget = true;
        let ActorYawOffset = 0.0f32;

        // Setup Enemy Data
        //------------------
        let mut EnemyPos = CVec3 {
            v: (*(*NPC).enemy).currentOrigin,
        };
        let mut EnemyAngles = CVec3 {
            v: (*(*NPC).enemy).currentAngles,
        };
        EnemyAngles.v[2] = 0.0f32;
        let EnemyVeh = if (*(*NPC).enemy).s.m_iVehicleNum != 0 {
            g_entities[(*(*NPC).enemy).s.m_iVehicleNum as usize].m_pVehicle
        } else {
            std::ptr::null_mut()
        };
        let EnemyInTurbo = !EnemyVeh.is_null() && (*EnemyVeh).m_iTurboTime > level.time;
        let EnemySpeed = if !EnemyVeh.is_null() {
            (*EnemyVeh).m_pParentEntity.as_ref().unwrap().client.as_ref().unwrap().ps.speed as f32
        } else {
            (*(*NPC).enemy).resultspeed
        };
        let EnemySlideBreak = !EnemyVeh.is_null()
            && (((*EnemyVeh).m_ulFlags & VEH_SLIDEBREAKING) != 0 || ((*EnemyVeh).m_ulFlags & VEH_STRAFERAM) != 0);
        let EnemyDead = (*(*NPC).enemy).health <= 0;

        let ActorFlank = (*NPCInfo).lastAvoidSteerSideDebouncer > level.time && !EnemyVeh.is_null() && EnemySpeed > 10.0f32;

        let mut EnemyDirection = CVec3 { v: [0.0f32; 3] };
        let mut EnemyRight = CVec3 { v: [0.0f32; 3] };
        AngleVectors(EnemyAngles.v, addr_of_mut!(EnemyDirection.v), addr_of_mut!(EnemyRight.v), std::ptr::null_mut());

        let mut EnemyFuturePos = EnemyPos;
        EnemyFuturePos.ScaleAdd(&EnemyDirection, FUTURE_PRED_DIST);

        let EnemySide = ActorPos.LRTest(&EnemyPos, &EnemyFuturePos);
        let mut EnemyFlankPos = EnemyFuturePos;
        let flank_dist = match EnemySide {
            ESide::Side_Right => FUTURE_SIDE_DIST,
            ESide::Side_Left => -FUTURE_SIDE_DIST,
        };
        EnemyFlankPos.ScaleAdd(&EnemyRight, flank_dist);

        // Setup Move And Aim Directions
        //-------------------------------
        let mut MoveDirection = if ActorFlank {
            EnemyFlankPos
        } else {
            EnemyFuturePos
        };
        MoveDirection -= ActorPos;
        let MoveDistance = MoveDirection.SafeNorm();
        let MoveAccuracy = MoveDirection.Dot(&ActorDirection);

        let mut AimDirection = EnemyPos;
        AimDirection -= ActorPos;
        let AimDistance = AimDirection.SafeNorm();
        let AimAccuracy = AimDirection.Dot(&ActorDirection);

        if !ActorFlank && TIMER_Done(NPC, b"FlankAttackCheck\0".as_ptr()) {
            TIMER_Set(NPC, b"FlankAttackCheck\0".as_ptr(), Q_irand(1000, 3000));
            if MoveDistance < 4000.0f32 && Q_irand(0, 1) == 0 {
                (*NPCInfo).lastAvoidSteerSideDebouncer = level.time + Q_irand(8000, 14000);
            }
        }

        // Fly By Sounds
        //---------------
        if ((*ActorVeh).as_ref().unwrap().m_pVehicleInfo.as_ref().unwrap().soundFlyBy != 0
            || (*ActorVeh).as_ref().unwrap().m_pVehicleInfo.as_ref().unwrap().soundFlyBy2 != 0)
            && !EnemyVeh.is_null()
            && MoveDistance < 800.0f32
            && ActorSpeed > 500.0f32
            && TIMER_Done(NPC, b"FlybySoundDebouncer\0".as_ptr())
        {
            if EnemySpeed < 100.0f32
                || (ActorDirection.Dot(&EnemyDirection) * (MoveDistance / 800.0f32)) < -0.5f32
            {
                TIMER_Set(NPC, b"FlybySoundDebouncer\0".as_ptr(), 2000);
                let mut soundFlyBy = (*ActorVeh).m_pVehicleInfo.as_ref().unwrap().soundFlyBy;
                if (*ActorVeh).m_pVehicleInfo.as_ref().unwrap().soundFlyBy2 != 0
                    && (soundFlyBy == 0 || Q_irand(0, 1) == 0)
                {
                    soundFlyBy = (*ActorVeh).m_pVehicleInfo.as_ref().unwrap().soundFlyBy2;
                }
                G_Sound((*ActorVeh).m_pParentEntity, soundFlyBy);
            }
        }

        // FLY PAST BEHAVIOR
        //===================
        if EnemySlideBreak || !TIMER_Done(NPC, b"MinHoldDirectionTime\0".as_ptr()) {
            if TIMER_Done(NPC, b"MinHoldDirectionTime\0".as_ptr()) {
                TIMER_Set(NPC, b"MinHoldDirectionTime\0".as_ptr(), 500); // Hold For At Least 500 ms
            }
            ActorAccelerate = true; // Go
            ActorAimAtTarget = false; // Don't Alter Our Aim Direction
            ucmd.buttons &= !BUTTON_VEH_SPEED; // Let Normal Vehicle Controls Go
        }
        // FLANKING BEHAVIOR
        //===================
        else if ActorFlank {
            ActorAccelerate = true;
            ActorDoTurbo = MoveDistance > 2500.0f32 || EnemyInTurbo;
            ucmd.buttons |= BUTTON_VEH_SPEED; // Tells PMove to use the ps.speed we calculate here, not the one from g_vehicles.c

            // For Flanking, We Calculate The Speed By Hand, Rather Than Using Pure Accelerate / No Accelerate Functionality
            //---------------------------------------------------------------------------------------------------------------
            (*NPC).client.as_mut().unwrap().ps.speed = ((*ActorVeh)
                .m_pVehicleInfo
                .as_ref()
                .unwrap()
                .speedMax
                * if ActorInTurbo { 1.35f32 } else { 1.15f32 }) as c_int;

            // If In Slowing Distance, Scale Down The Speed As We Approach Our Move Target
            //-----------------------------------------------------------------------------
            if MoveDistance < ATTACK_FLANK_SLOWING {
                (*NPC).client.as_mut().unwrap().ps.speed =
                    ((*NPC).client.as_ref().unwrap().ps.speed as f32 * (MoveDistance / ATTACK_FLANK_SLOWING)) as c_int;
                (*NPC).client.as_mut().unwrap().ps.speed =
                    ((*NPC).client.as_ref().unwrap().ps.speed as f32 + EnemySpeed) as c_int;

                // Match Enemy Speed
                //-------------------
                if (*NPC).client.as_ref().unwrap().ps.speed as f32 < 5.0f32 && EnemySpeed < 5.0f32 {
                    (*NPC).client.as_mut().unwrap().ps.speed = EnemySpeed as c_int;
                }

                // Extra Slow Down When Out In Front
                //-----------------------------------
                if MoveAccuracy < 0.0f32 {
                    (*NPC).client.as_mut().unwrap().ps.speed =
                        ((*NPC).client.as_ref().unwrap().ps.speed as f32 * (MoveAccuracy + 1.0f32)) as c_int;
                }

                MoveDirection *= MoveDistance / ATTACK_FLANK_SLOWING;
                EnemyDirection *= 1.0f32 - (MoveDistance / ATTACK_FLANK_SLOWING);
                MoveDirection += EnemyDirection;

                if TIMER_Done(NPC, b"RamCheck\0".as_ptr()) {
                    TIMER_Set(NPC, b"RamCheck\0".as_ptr(), Q_irand(1000, 3000));
                    if MoveDistance < RAM_DIST && Q_irand(0, 2) == 0 {
                        let side_right = match EnemySide {
                            ESide::Side_Left => false,
                            ESide::Side_Right => true,
                        };
                        VEH_StartStrafeRam(ActorVeh, side_right);
                    }
                }
            }
        }
        // NORMAL CHASE BEHAVIOR
        //=======================
        else {
            if EnemyVeh.is_null() && AimAccuracy > 0.99f32 && MoveDistance < 500.0f32 && !EnemyDead {
                ActorAccelerate = true;
                ActorDoTurbo = false;
            } else {
                ActorAccelerate = (MoveDistance > 500.0f32 && EnemySpeed > 20.0f32) || MoveDistance > 1000.0f32;
                ActorDoTurbo = MoveDistance > 3000.0f32 && EnemySpeed > 20.0f32;
            }
            ucmd.buttons &= !BUTTON_VEH_SPEED;
        }

        // APPLY RESULTS
        //=======================
        // Decide Turbo
        //--------------
        if ActorDoTurbo || ActorInTurbo {
            ucmd.buttons |= BUTTON_ALT_ATTACK;
        } else {
            ucmd.buttons &= !BUTTON_ALT_ATTACK;
        }

        // Decide Acceleration
        //---------------------
        ucmd.forwardmove = if ActorAccelerate { 127 } else { 0 };

        // Decide To Shoot
        //-----------------
        ucmd.buttons &= !BUTTON_ATTACK;
        ucmd.rightmove = 0;
        if AimDistance < 2000.0f32 && !EnemyDead {
            // If Doing A Ram Attack
            //-----------------------
            if ActorYawOffset != 0.0f32 {
                if (*NPC).client.as_ref().unwrap().ps.weapon != WP_NONE as u8 {
                    NPC_ChangeWeapon(WP_NONE);
                }
                ucmd.buttons &= !BUTTON_ATTACK;
            } else if AimAccuracy > ATTACK_FWD {
                if (*NPC).client.as_ref().unwrap().ps.weapon != WP_NONE as u8 {
                    NPC_ChangeWeapon(WP_NONE);
                }
                ucmd.buttons |= BUTTON_ATTACK;
            } else if AimAccuracy < AIM_SIDE && AimAccuracy > -AIM_SIDE {
                if (*NPC).client.as_ref().unwrap().ps.weapon != WP_BLASTER as u8 {
                    NPC_ChangeWeapon(WP_BLASTER);
                }

                if AimAccuracy < ATTACK_SIDE && AimAccuracy > -ATTACK_SIDE {
                    ucmd.buttons |= BUTTON_ATTACK;

                    WeaponThink(true);
                }
                ucmd.rightmove = match EnemySide {
                    ESide::Side_Left => 127,
                    ESide::Side_Right => -127,
                };
            } else {
                if (*NPC).client.as_ref().unwrap().ps.weapon != WP_NONE as u8 {
                    NPC_ChangeWeapon(WP_NONE);
                }
            }
        } else {
            if (*NPC).client.as_ref().unwrap().ps.weapon != WP_NONE as u8 {
                NPC_ChangeWeapon(WP_NONE);
            }
        }

        // Aim At Target
        //---------------
        if ActorAimAtTarget {
            MoveDirection.VecToAng();
            (*NPCInfo).desiredPitch = AngleNormalize360(MoveDirection.v[PITCH]);
            (*NPCInfo).desiredYaw = AngleNormalize360(MoveDirection.v[YAW] + ActorYawOffset);
        }
        NPC_UpdateAngles(1, 1);
    }
}

// Stub wrapper for gi.trace to match the interface
fn gi_trace(
    _trace: *mut trace_t,
    _start: vec3_t,
    _mins: vec3_t,
    _maxs: vec3_t,
    _end: vec3_t,
    _passent: c_int,
    _contentmask: c_int,
) {
    // This would call the actual game engine trace function
}
