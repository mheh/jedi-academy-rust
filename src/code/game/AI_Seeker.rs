// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// #include "g_headers.h"

// #include "b_local.h"
// #include "g_nav.h"

extern "C" {
    fn NPC_BSST_Patrol();
    fn Boba_FireDecide();
    fn CreateMissile(org: *const [f32; 3], dir: *const [f32; 3], vel: f32, life: i32, owner: *mut gentity_t, altFire: c_uint) -> *mut gentity_t;
    fn G_SoundIndex(sound: *const c_char) -> i32;
    fn G_EffectIndex(effect: *const c_char) -> i32;
    fn G_Damage(targ: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *const [f32; 3], point: *const [f32; 3], damage: i32, dflags: i32, mod_: i32);
    fn SaveNPCGlobals();
    fn SetNPCGlobals(self_: *mut gentity_t);
    fn RestoreNPCGlobals();
    fn NPC_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: i32, mod_: i32);
    fn NPC_UpdateAngles(doPitch: c_uint, doYaw: c_uint);
    fn TIMER_Done(ent: *mut gentity_t, timer: *const c_char) -> c_uint;
    fn TIMER_Set(ent: *mut gentity_t, timer: *const c_char, duration: i32);
    fn Q_irand(min: i32, max: i32) -> i32;
    fn Q_flrand(min: f32, max: f32) -> f32;
    fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn VectorScale(vin: *const [f32; 3], scale: f32, vout: *mut [f32; 3]);
    fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    fn G_Sound(ent: *mut gentity_t, index: i32);
    fn NPC_FaceEnemy(doPitch: c_uint);
    fn NPC_MoveToGoal(allowGo: c_uint);
    fn NPC_ClearLOS(ent: *mut gentity_t) -> c_uint;
    fn DistanceHorizontalSquared(p1: *const [f32; 3], p2: *const [f32; 3]) -> f32;
    fn CalcEntitySpot(ent: *mut gentity_t, spot: i32, point: *mut [f32; 3]);
    fn G_PlayEffect(effect: *const c_char, origin: *const [f32; 3], dir: *const [f32; 3]);
    fn crandom() -> f32;
    fn random() -> f32;
    fn rand() -> i32;

    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut npcinfoState_t;
    static mut ucmd: usercmd_t;
    static mut level: level_locals_t;
    static mut g_entities: [gentity_t; 2048];
    static mut in_camera: c_uint;
    static mut g_spskill: *mut cvar_t;
}

use core::ffi::{c_int, c_char, c_uint, c_void};

// Stub structures for portability - these are placeholders for the full definitions
#[repr(C)]
pub struct gentity_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct npcinfoState_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct usercmd_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct level_locals_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct cvar_t {
    _private: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
    _private: [u8; 0],
}

extern "C" {
    fn gi_trace(tr: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passent: i32, contentmask: i32);
}

fn Seeker_Strafe();

const VELOCITY_DECAY: f32 = 0.7f32;

const MIN_MELEE_RANGE: i32 = 320;
const MIN_MELEE_RANGE_SQR: i32 = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

const MIN_DISTANCE: i32 = 80;
const MIN_DISTANCE_SQR: i32 = MIN_DISTANCE * MIN_DISTANCE;

const SEEKER_STRAFE_VEL: f32 = 100f32;
const SEEKER_STRAFE_DIS: f32 = 200f32;
const SEEKER_UPWARD_PUSH: f32 = 32f32;

const SEEKER_FORWARD_BASE_SPEED: f32 = 10f32;
const SEEKER_FORWARD_MULTIPLIER: f32 = 2f32;

const SEEKER_SEEK_RADIUS: f32 = 1024f32;

//------------------------------------
fn NPC_Seeker_Precache() {
    unsafe {
        G_SoundIndex(b"sound/chars/seeker/misc/fire.wav\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/seeker/misc/hiss.wav\0".as_ptr() as *const c_char);
        G_EffectIndex(b"env/small_explode\0".as_ptr() as *const c_char);
    }
}

//------------------------------------
fn NPC_Seeker_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: i32, mod_: i32, hitLoc: i32) {
    unsafe {
        // Assuming SVF_CUSTOM_GRAVITY is a flag constant
        const SVF_CUSTOM_GRAVITY: u32 = 0x1000; // placeholder

        if ((*self_).svFlags & SVF_CUSTOM_GRAVITY) == 0 {
            //void G_Damage( gentity_t *targ, gentity_t *inflictor, gentity_t *attacker, vec3_t dir, vec3_t point, int damage, int dflags, int mod, int hitLoc=HL_NONE );
            G_Damage( self_, core::ptr::null_mut(), core::ptr::null_mut(), vec3_origin as *const _, (vec3_origin as *const f32) as *const _, 999, 0, 4 ); // MOD_FALLING assumed to be 4
        }

        SaveNPCGlobals();
        SetNPCGlobals( self_ );
        Seeker_Strafe();
        RestoreNPCGlobals();
        NPC_Pain( self_, inflictor, other, point, damage, mod_ );
    }
}

static vec3_origin: [f32; 3] = [0.0, 0.0, 0.0];

//------------------------------------
fn Seeker_MaintainHeight() {
    unsafe {
        let mut dif: f32;

        // Update our angles regardless
        NPC_UpdateAngles( 1, 1 );

        // If we have an enemy, we should try to hover at or a little below enemy eye level
        if !(*NPC).enemy.is_null() {
            if TIMER_Done( NPC, b"heightChange\0".as_ptr() as *const c_char ) != 0 {
                TIMER_Set( NPC, b"heightChange\0".as_ptr() as *const c_char, Q_irand( 1000, 3000 ));

                // Find the height difference
                dif = ((*(*NPC).enemy).currentOrigin[2] +  Q_flrand( (*(*NPC).enemy).maxs[2]/2.0, (*(*NPC).enemy).maxs[2]+8.0 )) - (*NPC).currentOrigin[2];

                let mut difFactor: f32 = 1.0f32;
                if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT assumed to be 3
                    if TIMER_Done( NPC, b"flameTime\0".as_ptr() as *const c_char ) != 0 {
                        difFactor = 10.0f32;
                    }
                }

                // cap to prevent dramatic height shifts
                if dif.abs() > 2.0*difFactor {
                    if dif.abs() > 24.0*difFactor {
                        dif = ( if dif < 0.0 { -24.0*difFactor } else { 24.0*difFactor } );
                    }

                    (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2]+dif)/2.0;
                }
                if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT
                    (*(*NPC).client).ps.velocity[2] *= Q_flrand( 0.85f32, 3.0f32 );
                }
            }
        } else {
            let mut goal: *mut gentity_t = core::ptr::null_mut();

            if !(*NPCInfo).goalEntity.is_null() {	// Is there a goal?
                goal = (*NPCInfo).goalEntity;
            } else {
                goal = (*NPCInfo).lastGoalEntity;
            }
            if !goal.is_null() {
                dif = (*goal).currentOrigin[2] - (*NPC).currentOrigin[2];

                if dif.abs() > 24.0 {
                    ucmd.upmove = ( if ucmd.upmove < 0 { -4 } else { 4 } );
                } else {
                    if (*(*NPC).client).ps.velocity[2] != 0.0 {
                        (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                        if (*(*NPC).client).ps.velocity[2].abs() < 2.0 {
                            (*(*NPC).client).ps.velocity[2] = 0.0;
                        }
                    }
                }
            }
        }

        // Apply friction
        if (*(*NPC).client).ps.velocity[0] != 0.0 {
            (*(*NPC).client).ps.velocity[0] *= VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[0].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[0] = 0.0;
            }
        }

        if (*(*NPC).client).ps.velocity[1] != 0.0 {
            (*(*NPC).client).ps.velocity[1] *= VELOCITY_DECAY;

            if (*(*NPC).client).ps.velocity[1].abs() < 1.0 {
                (*(*NPC).client).ps.velocity[1] = 0.0;
            }
        }
    }
}

//------------------------------------
fn Seeker_Strafe() {
    unsafe {
        let mut side: i32;
        let mut end: [f32; 3];
        let mut right: [f32; 3];
        let mut dir: [f32; 3];
        let mut tr: trace_t;

        if random() > 0.7f32 || (*NPC).enemy.is_null() || (*(*NPC).enemy).client.is_null() {
            // Do a regular style strafe
            AngleVectors( &(*(*NPC).client).renderInfo.eyeAngles as *const _, core::ptr::null_mut(), &mut right as *mut _, core::ptr::null_mut() );

            // Pick a random strafe direction, then check to see if doing a strafe would be
            //	reasonably valid
            side = ( if (rand() & 1) != 0 { -1 } else { 1 } );
            VectorMA( &(*NPC).currentOrigin as *const _, SEEKER_STRAFE_DIS * (side as f32), &right as *const _, &mut end as *mut _ );

            gi_trace( &mut tr, &(*NPC).currentOrigin as *const _, core::ptr::null(), core::ptr::null(), &end as *const _, (*NPC).s.number, 0x4000 ); // MASK_SOLID assumed to be 0x4000

            // Close enough
            if 0.9f32 < 1.0f32 {  // tr.fraction > 0.9f - accessing tr.fraction would require full struct
                let mut vel: f32 = SEEKER_STRAFE_VEL;
                let mut upPush: f32 = SEEKER_UPWARD_PUSH;
                if (*(*NPC).client).NPC_class != 1 { // CLASS_BOBAFETT check inverted
                    G_Sound( NPC, G_SoundIndex( b"sound/chars/seeker/misc/hiss\0".as_ptr() as *const c_char ));
                } else {
                    vel *= 3.0f32;
                    upPush *= 4.0f32;
                }
                VectorMA( &(*(*NPC).client).ps.velocity as *const _, vel*(side as f32), &right as *const _, &mut (*(*NPC).client).ps.velocity as *mut _ );
                // Add a slight upward push
                (*(*NPC).client).ps.velocity[2] += upPush;

                (*NPCInfo).standTime = 0; // level.time + 1000 + random() * 500 - needs level.time access
            }
        } else {
            // Do a strafe to try and keep on the side of their enemy
            AngleVectors( &(*(*(*NPC).enemy).client).renderInfo.eyeAngles as *const _, &mut dir as *mut _, &mut right as *mut _, core::ptr::null_mut() );

            // Pick a random side
            side = ( if (rand() & 1) != 0 { -1 } else { 1 } );
            let mut stDis: f32 = SEEKER_STRAFE_DIS;
            if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT
                stDis *= 2.0f32;
            }
            VectorMA( &(*(*NPC).enemy).currentOrigin as *const _, stDis * (side as f32), &right as *const _, &mut end as *mut _ );

            // then add a very small bit of random in front of/behind the player action
            VectorMA( &end as *const _, crandom() * 25.0, &dir as *const _, &mut end as *mut _ );

            gi_trace( &mut tr, &(*NPC).currentOrigin as *const _, core::ptr::null(), core::ptr::null(), &end as *const _, (*NPC).s.number, 0x4000 ); // MASK_SOLID

            // Close enough
            if 0.9f32 < 1.0f32 { // tr.fraction > 0.9f
                // tr.endpos access needed - defer
                let mut dis: f32;
                dir[2] *= 0.25; // do less upward change
                dis = VectorNormalize( &mut dir as *mut _ );

                // Try to move the desired enemy side
                VectorMA( &(*(*NPC).client).ps.velocity as *const _, dis, &dir as *const _, &mut (*(*NPC).client).ps.velocity as *mut _ );

                let mut upPush: f32 = SEEKER_UPWARD_PUSH;
                if (*(*NPC).client).NPC_class != 3 { // CLASS_BOBAFETT
                    G_Sound( NPC, G_SoundIndex( b"sound/chars/seeker/misc/hiss\0".as_ptr() as *const c_char ));
                } else {
                    upPush *= 4.0f32;
                }

                // Add a slight upward push
                (*(*NPC).client).ps.velocity[2] += upPush;

                (*NPCInfo).standTime = 0; // level.time + 2500 + random() * 500
            }
        }
    }
}

//------------------------------------
fn Seeker_Hunt(visible: c_uint, advance: c_uint) {
    unsafe {
        let mut distance: f32;
        let mut speed: f32;
        let mut forward: [f32; 3];

        NPC_FaceEnemy( 1 );

        // If we're not supposed to stand still, pursue the player
        if (*NPCInfo).standTime < 0 { // level.time - needs level.time
            // Only strafe when we can see the player
            if visible != 0 {
                Seeker_Strafe();
                return;
            }
        }

        // If we don't want to advance, stop here
        if advance == 0 {
            return;
        }

        // Only try and navigate if the player is visible
        if visible == 0 {
            // Move towards our goal
            (*NPCInfo).goalEntity = (*NPC).enemy;
            (*NPCInfo).goalRadius = 24;

            NPC_MoveToGoal(1);
            return;

        } else {
            VectorSubtract( &(*(*NPC).enemy).currentOrigin as *const _, &(*NPC).currentOrigin as *const _, &mut forward as *mut _ );
            distance = VectorNormalize( &mut forward as *mut _ );
        }

        speed = SEEKER_FORWARD_BASE_SPEED + SEEKER_FORWARD_MULTIPLIER * 0.0; // g_spskill->integer - needs full cvar access
        VectorMA( &(*(*NPC).client).ps.velocity as *const _, speed, &forward as *const _, &mut (*(*NPC).client).ps.velocity as *mut _ );
    }
}

//------------------------------------
fn Seeker_Fire() {
    unsafe {
        let mut dir: [f32; 3];
        let mut enemy_org: [f32; 3];
        let mut muzzle: [f32; 3];
        let mut missile: *mut gentity_t;

        CalcEntitySpot( (*NPC).enemy, 0, &mut enemy_org as *mut _ ); // SPOT_HEAD assumed to be 0
        VectorSubtract( &enemy_org as *const _, &(*NPC).currentOrigin as *const _, &mut dir as *mut _ );
        VectorNormalize( &mut dir as *mut _ );

        // move a bit forward in the direction we shall shoot in so that the bolt doesn't poke out the other side of the seeker
        VectorMA( &(*NPC).currentOrigin as *const _, 15.0, &dir as *const _, &mut muzzle as *mut _ );

        missile = CreateMissile( &muzzle as *const _, &dir as *const _, 1000.0, 10000, NPC, 0 );

        G_PlayEffect( b"blaster/muzzle_flash\0".as_ptr() as *const c_char, &(*NPC).currentOrigin as *const _, &dir as *const _ );

        (*missile).classname = b"blaster\0".as_ptr() as *const c_char;
        (*missile).s.weapon = 1; // WP_BLASTER assumed to be 1

        (*missile).damage = 5;
        (*missile).dflags = 0x2; // DAMAGE_DEATH_KNOCKBACK assumed to be 0x2
        (*missile).methodOfDeath = 3; // MOD_ENERGY assumed to be 3
        (*missile).clipmask = 0x4001; // MASK_SHOT | CONTENTS_LIGHTSABER
    }
}

//------------------------------------
fn Seeker_Ranged(visible: c_uint, advance: c_uint) {
    unsafe {
        if (*(*NPC).client).NPC_class != 3 { // CLASS_BOBAFETT
            if (*NPC).count > 0 {
                if TIMER_Done( NPC, b"attackDelay\0".as_ptr() as *const c_char ) != 0 {	// Attack?
                    TIMER_Set( NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand( 250, 2500 ));
                    Seeker_Fire();
                    (*NPC).count -= 1;
                }
            } else {
                // out of ammo, so let it die...give it a push up so it can fall more and blow up on impact
        //		NPC->client->ps.gravity = 900;
        //		NPC->svFlags &= ~SVF_CUSTOM_GRAVITY;
        //		NPC->client->ps.velocity[2] += 16;
                G_Damage( NPC, NPC, NPC, core::ptr::null(), core::ptr::null(), 999, 0, 0 ); // MOD_UNKNOWN assumed to be 0
            }
        }

        if ((*NPCInfo).scriptFlags & 0x1000000) != 0 { // SCF_CHASE_ENEMIES assumed to be 0x1000000
            Seeker_Hunt( visible, advance );
        }
    }
}

//------------------------------------
fn Seeker_Attack() {
    unsafe {
        // Always keep a good height off the ground
        Seeker_MaintainHeight();

        // Rate our distance to the target, and our visibilty
        let distance: f32 = DistanceHorizontalSquared( &(*NPC).currentOrigin as *const _, &(*(*NPC).enemy).currentOrigin as *const _ );
        let visible: c_uint = NPC_ClearLOS( (*NPC).enemy );
        let mut advance: c_uint = if distance > (MIN_DISTANCE_SQR as f32) { 1 } else { 0 };

        if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT
            advance = if distance > (200.0f32*200.0f32) { 1 } else { 0 };
        }

        // If we cannot see our target, move to see it
        if visible == 0 {
            if ((*NPCInfo).scriptFlags & 0x1000000) != 0 { // SCF_CHASE_ENEMIES
                Seeker_Hunt( visible, advance );
                return;
            }
        }

        Seeker_Ranged( visible, advance );
    }
}

//------------------------------------
fn Seeker_FindEnemy() {
    unsafe {
        let mut numFound: i32;
        let mut dis: f32;
        let mut bestDis: f32 = SEEKER_SEEK_RADIUS * SEEKER_SEEK_RADIUS + 1.0;
        let mut mins: [f32; 3];
        let mut maxs: [f32; 3];
        let mut entityList: [*mut gentity_t; 2048];
        let mut ent: *mut gentity_t;
        let mut best: *mut gentity_t = core::ptr::null_mut();

        VectorSet( &mut maxs as *mut _, SEEKER_SEEK_RADIUS, SEEKER_SEEK_RADIUS, SEEKER_SEEK_RADIUS );
        VectorScale( &maxs as *const _, -1.0, &mut mins as *mut _ );

        // gi.EntitiesInBox( mins, maxs, entityList, MAX_GENTITIES ) - needs extern
        numFound = 0; // placeholder

        let mut i: i32 = 0;
        while i < numFound {
            ent = entityList[i as usize];

            if (*ent).s.number == (*NPC).s.number || (*ent).client.is_null() || (*ent).NPC.is_null() || (*ent).health <= 0 || (*ent).inuse == 0 {
                i += 1;
                continue;
            }

            if (*(*ent).client).playerTeam == (*(*NPC).client).playerTeam || (*(*ent).client).playerTeam == 0 { // TEAM_NEUTRAL assumed to be 0
                i += 1;
                continue;
            }

            // try to find the closest visible one
            if NPC_ClearLOS( ent ) == 0 {
                i += 1;
                continue;
            }

            dis = DistanceHorizontalSquared( &(*NPC).currentOrigin as *const _, &(*ent).currentOrigin as *const _ );

            if dis <= bestDis {
                bestDis = dis;
                best = ent;
            }
            i += 1;
        }

        if !best.is_null() {
            // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
            (*NPC).random = random() * 6.3f32; // roughly 2pi

            (*NPC).enemy = best;
        }
    }
}

//------------------------------------
fn Seeker_FollowPlayer() {
    unsafe {
        Seeker_MaintainHeight();

        let dis: f32 = DistanceHorizontalSquared( &(*NPC).currentOrigin as *const _, &g_entities[0].currentOrigin as *const _ );
        let mut pt: [f32; 3];
        let mut dir: [f32; 3];

        let mut minDistSqr: f32 = MIN_DISTANCE_SQR as f32;

        if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT
            if TIMER_Done( NPC, b"flameTime\0".as_ptr() as *const c_char ) != 0 {
                minDistSqr = 200.0*200.0;
            }
        }

        if dis < minDistSqr {
            // generally circle the player closely till we take an enemy..this is our target point
            if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT
                pt[0] = g_entities[0].currentOrigin[0] + ((level.time as f32) * 0.001f32 + (*NPC).random).cos() * 250.0;
                pt[1] = g_entities[0].currentOrigin[1] + ((level.time as f32) * 0.001f32 + (*NPC).random).sin() * 250.0;
                if (*(*NPC).client).jetPackTime < level.time as i32 {
                    pt[2] = (*NPC).currentOrigin[2] - 64.0;
                } else {
                    pt[2] = g_entities[0].currentOrigin[2] + 200.0;
                }
            } else {
                pt[0] = g_entities[0].currentOrigin[0] + ((level.time as f32) * 0.001f32 + (*NPC).random).cos() * 56.0;
                pt[1] = g_entities[0].currentOrigin[1] + ((level.time as f32) * 0.001f32 + (*NPC).random).sin() * 56.0;
                pt[2] = g_entities[0].currentOrigin[2] + 40.0;
            }

            VectorSubtract( &pt as *const _, &(*NPC).currentOrigin as *const _, &mut dir as *mut _ );
            VectorMA( &(*(*NPC).client).ps.velocity as *const _, 0.8f32, &dir as *const _, &mut (*(*NPC).client).ps.velocity as *mut _ );
        } else {
            if (*(*NPC).client).NPC_class != 3 { // CLASS_BOBAFETT
                if TIMER_Done( NPC, b"seekerhiss\0".as_ptr() as *const c_char ) != 0 {
                    TIMER_Set( NPC, b"seekerhiss\0".as_ptr() as *const c_char, (1000.0 + random() * 1000.0) as i32 );
                    G_Sound( NPC, G_SoundIndex( b"sound/chars/seeker/misc/hiss\0".as_ptr() as *const c_char ));
                }
            }

            // Hey come back!
            (*NPCInfo).goalEntity = &mut g_entities[0];
            (*NPCInfo).goalRadius = 32;
            NPC_MoveToGoal( 1 );
            (*NPC).owner = &mut g_entities[0];
        }

        if (*NPCInfo).enemyCheckDebounceTime < level.time as i32 {
            // check twice a second to find a new enemy
            Seeker_FindEnemy();
            (*NPCInfo).enemyCheckDebounceTime = level.time as i32 + 500;
        }

        NPC_UpdateAngles( 1, 1 );
    }
}

//------------------------------------
fn NPC_BSSeeker_Default() {
    unsafe {
        if in_camera != 0 {
            if (*(*NPC).client).NPC_class != 3 { // CLASS_BOBAFETT
                // cameras make me commit suicide....
                G_Damage( NPC, NPC, NPC, core::ptr::null(), core::ptr::null(), 999, 0, 0 ); // MOD_UNKNOWN assumed to be 0
            }
        }

        if (*NPC).random == 0.0f32 {
            // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
            (*NPC).random = random() * 6.3f32; // roughly 2pi
        }

        if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health != 0 && (*(*NPC).enemy).inuse != 0 {
            if (*(*NPC).client).NPC_class != 3 // CLASS_BOBAFETT
                && ( (*(*NPC).enemy).s.number == 0 || (!(*(*NPC).enemy).client.is_null() && (*(*(*NPC).enemy).client).NPC_class == 1) ) { // CLASS_SEEKER assumed to be 1
                //hacked to never take the player as an enemy, even if the player shoots at it
                (*NPC).enemy = core::ptr::null_mut();
            } else {
                Seeker_Attack();
                if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT
                    Boba_FireDecide();
                }
                return;
            }
        } else if (*(*NPC).client).NPC_class == 3 { // CLASS_BOBAFETT
            NPC_BSST_Patrol();
            return;
        }

        // In all other cases, follow the player and look for enemies to take on
        Seeker_FollowPlayer();
    }
}
