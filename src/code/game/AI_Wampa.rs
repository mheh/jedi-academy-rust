// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// #include "g_headers.h"
// #include "b_local.h"

use core::ffi::{c_int, c_void};

// These define the working combat range for these suckers
const MIN_DISTANCE: c_int = 48;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const MAX_DISTANCE: c_int = 1024;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;

static mut enemyDist: f32 = 0.0;

// Stub type declarations for FFI
#[repr(C)]
pub struct gentity_t {
    _marker: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
    _marker: [u8; 0],
}

#[repr(C)]
pub struct cvar_t {
    _marker: [u8; 0],
}

#[repr(C)]
pub struct playerState_t {
    _marker: [u8; 0],
}

extern "C" {
    pub fn NAV_CheckAhead(
        selff: *mut gentity_t,
        end: *const [f32; 3],
        trace: *mut trace_t,
        clipmask: c_int,
    ) -> bool;
    pub fn PM_AnimLength(index: c_int, anim: c_int) -> c_int;
    pub static mut g_dismemberment: *mut cvar_t;

    pub fn G_SoundIndex(name: *const u8) -> c_int;
    pub fn UpdateGoal() -> bool;
    pub fn NPC_MoveToGoal(visible: bool) -> bool;
    pub fn NPC_CheckEnemyExt(distanceCheck: bool) -> bool;
    pub fn Wampa_Idle();
    pub fn NPC_SetAnim(
        ent: *mut gentity_t,
        setAnimParts: c_int,
        anim: c_int,
        flags: c_int,
    );
    pub fn TIMER_Set(ent: *mut gentity_t, label: *const u8, duration: c_int);
    pub fn TIMER_Done(ent: *const gentity_t, label: *const u8) -> bool;
    pub fn TIMER_Done2(
        ent: *const gentity_t,
        label: *const u8,
        remove: bool,
    ) -> bool;
    pub fn TIMER_Exists(ent: *const gentity_t, label: *const u8) -> bool;
    pub fn TIMER_Remove(ent: *mut gentity_t, label: *const u8);
    pub fn NPC_ClearLOS(ent: *mut gentity_t) -> bool;
    pub fn NPC_FaceEnemy(addPainAnim: bool);
    pub fn Distance(start: *const [f32; 3], end: *const [f32; 3]) -> f32;
    pub fn DistanceSquared(start: *const [f32; 3], end: *const [f32; 3]) -> f32;
    pub fn G_Knockdown(
        selff: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const [f32; 3],
        strength: f32,
        breakSaberLock: bool,
    );
    pub fn G_DoDismemberment(
        selff: *mut gentity_t,
        point: *const [f32; 3],
        mod_: c_int,
        damage: c_int,
        hitLoc: c_int,
        force: bool,
    ) -> bool;
    pub fn NPC_GetEntsNearBolt(
        radiusEnts: *mut *mut gentity_t,
        radius: f32,
        boltIndex: c_int,
        boltOrg: *mut [f32; 3],
    ) -> c_int;
    pub fn G_Damage(
        victim: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        dir: *const [f32; 3],
        point: *const [f32; 3],
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    );
    pub fn G_Throw(victim: *mut gentity_t, dir: *const [f32; 3], strength: c_int);
    pub fn G_Sound(ent: *mut gentity_t, index: c_int);
    pub fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundPath: *const u8);
    pub fn NPC_CheckEnemyExt(distanceCheck: bool) -> bool;
    pub fn NPC_ValidEnemy(ent: *const gentity_t) -> bool;
    pub fn NPC_CheckEnemy(
        IgnoreTeam: bool,
        checkSightlineToBody: bool,
        setBusy: bool,
    ) -> *mut gentity_t;
    pub fn G_SetEnemy(ent: *mut gentity_t, enemy: *mut gentity_t);
    pub fn NPC_UpdateAngles(addPainAnim: bool, lookAtEnemy: bool);
    pub fn NPC_EnemyRangeFromBolt(boltIndex: c_int) -> f32;
    pub fn AngleVectors(
        angles: *const [f32; 3],
        forward: *mut [f32; 3],
        right: *mut [f32; 3],
        up: *mut [f32; 3],
    );
    pub fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorScale(src: *const [f32; 3], scale: f32, dst: *mut [f32; 3]);
    pub fn AngleNormalize180(angle: f32) -> f32;
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn Q_flrand(low: f32, high: f32) -> f32;
    pub fn Q_random() -> f32;
    pub fn InFOV(
        spot: *const [f32; 3],
        from: *const [f32; 3],
        angles: *const [f32; 3],
        fovXY: c_int,
        fovZ: c_int,
    ) -> bool;
    pub fn SetClientViewAngle(ent: *mut gentity_t, angle: *const [f32; 3]);
    pub fn fabs(x: f32) -> f32;

    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut c_void; // Opaque for now
    pub static mut ucmd: c_void; // Opaque for now
    pub static mut level: c_void; // Opaque for now
    pub static mut gi: c_void; // Opaque for now
}

/*
-------------------------
NPC_Wampa_Precache
-------------------------
*/
#[allow(non_snake_case)]
pub extern "C" fn NPC_Wampa_Precache() {
    /*
    int i;
    for ( i = 1; i < 4; i ++ )
    {
        G_SoundIndex( va("sound/chars/wampa/growl%d.wav", i) );
    }
    for ( i = 1; i < 3; i ++ )
    {
        G_SoundIndex( va("sound/chars/wampa/snort%d.wav", i) );
    }
    */
    unsafe {
        G_SoundIndex(b"sound/chars/rancor/swipehit.wav\0".as_ptr());
        //G_SoundIndex( "sound/chars/wampa/chomp.wav" );
    }
}

/*
-------------------------
Wampa_Idle
-------------------------
*/
#[allow(non_snake_case)]
pub extern "C" fn Wampa_Idle() {
    unsafe {
        // NPCInfo->localState = LSTATE_CLEAR;
        // This requires proper accessor; stub for now
    }

    //If we have somewhere to go, then do that
    unsafe {
        if UpdateGoal() {
            // ucmd.buttons &= ~BUTTON_WALKING;
            // This requires proper accessor; stub for now
            NPC_MoveToGoal(true);
        }
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Wampa_CheckRoar(selff: *mut gentity_t) -> bool {
    unsafe {
        // if ( self->wait < level.time )
        // {
        //     self->wait = level.time + Q_irand( 5000, 20000 );
        //     NPC_SetAnim( self, SETANIM_BOTH, Q_irand(BOTH_GESTURE1,BOTH_GESTURE2), (SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD) );
        //     TIMER_Set( self, "rageTime", self->client->ps.legsAnimTimer );
        //     return qtrue;
        // }
        // return qfalse;
        // This requires access to self fields; stub for now
        false
    }
}

/*
-------------------------
Wampa_Patrol
-------------------------
*/
#[allow(non_snake_case)]
pub extern "C" fn Wampa_Patrol() {
    unsafe {
        // NPCInfo->localState = LSTATE_CLEAR;

        //If we have somewhere to go, then do that
        if UpdateGoal() {
            // ucmd.buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(true);
        }

        if !NPC_CheckEnemyExt(true) {
            Wampa_Idle();
            return;
        }
        Wampa_CheckRoar(NPC);
        TIMER_Set(NPC, b"lookForNewEnemy\0".as_ptr() as *const u8, Q_irand(5000, 15000));
    }
}

/*
-------------------------
Wampa_Move
-------------------------
*/
#[allow(non_snake_case)]
pub extern "C" fn Wampa_Move(visible: bool) {
    unsafe {
        // if ( NPCInfo->localState != LSTATE_WAITING )
        // {
        //     NPCInfo->goalEntity = NPC->enemy;

        //     trace_t	trace;
        //     if ( !NAV_CheckAhead( NPC, NPCInfo->goalEntity->currentOrigin, trace, (NPC->clipmask|CONTENTS_BOTCLIP) ) )
        //     {
        //         if ( !NPC_MoveToGoal( qfalse ) )
        //         {
        //             STEER::Activate(NPC);
        //             STEER::Seek(NPC, NPCInfo->goalEntity->currentOrigin);
        //             STEER::AvoidCollisions(NPC);
        //             STEER::DeActivate(NPC, &ucmd);
        //         }
        //     }
        //     NPCInfo->goalRadius = MIN_DISTANCE;//MAX_DISTANCE;	// just get us within combat range

        //     if ( NPC->enemy )
        //     {//pick correct movement speed and anim
        //         //run by default
        //         ucmd.buttons &= ~BUTTON_WALKING;
        //         if ( !TIMER_Done( NPC, "runfar" )
        //             || !TIMER_Done( NPC, "runclose" ) )
        //         {//keep running with this anim & speed for a bit
        //         }
        //         else if ( !TIMER_Done( NPC, "walk" ) )
        //         {//keep walking for a bit
        //             ucmd.buttons |= BUTTON_WALKING;
        //         }
        //         else if ( visible && enemyDist > 350 && NPCInfo->stats.runSpeed == 200 )//180 )
        //         {//fast run, all fours
        //             //BOTH_RUN1
        //             NPCInfo->stats.runSpeed = 300;
        //             TIMER_Set( NPC, "runfar", Q_irand( 4000, 8000 ) );
        //             if ( NPC->client->ps.legsAnim == BOTH_RUN2 )
        //             {
        //                 NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_RUN2TORUN1, SETANIM_FLAG_HOLD );
        //             }
        //         }
        //         else if ( enemyDist > 200 && NPCInfo->stats.runSpeed == 300 )
        //         {//slow run, upright
        //             //BOTH_RUN2
        //             NPCInfo->stats.runSpeed = 200;//180;
        //             TIMER_Set( NPC, "runclose", Q_irand( 5000, 10000 ) );
        //             if ( NPC->client->ps.legsAnim == BOTH_RUN1 )
        //             {
        //                 NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_RUN1TORUN2, SETANIM_FLAG_HOLD );
        //             }
        //         }
        //         else if ( enemyDist < 100 )
        //         {//walk
        //             NPCInfo->stats.runSpeed = 200;//180;
        //             ucmd.buttons |= BUTTON_WALKING;
        //             TIMER_Set( NPC, "walk", Q_irand( 6000, 12000 ) );
        //         }
        //     }
        // }
        // This requires access to complex structures; stub for now
    }
}

//---------------------------------------------------------
#[allow(non_snake_case)]
pub extern "C" fn Wampa_Slash(boltIndex: c_int, backhand: bool) {
    let mut radiusEnts: [*mut gentity_t; 128] = [std::ptr::null_mut(); 128];
    let radius = 88.0_f32;
    let radiusSquared = radius * radius;
    let mut boltOrg = [0.0_f32; 3];

    unsafe {
        let numEnts = NPC_GetEntsNearBolt(
            radiusEnts.as_mut_ptr(),
            radius,
            boltIndex,
            &mut boltOrg,
        );

        for i in 0..numEnts as usize {
            if (*radiusEnts[i]).inuse == 0 {
                continue;
            }

            if radiusEnts[i] == NPC {
                //Skip the wampa ent
                continue;
            }

            if (*radiusEnts[i]).client == std::ptr::null_mut() {
                //must be a client
                continue;
            }

            if DistanceSquared(&(*radiusEnts[i]).currentOrigin, &boltOrg) <= radiusSquared {
                //smack
                let damage = if backhand {
                    Q_irand(10, 15)
                } else {
                    Q_irand(20, 30)
                };
                G_Damage(
                    radiusEnts[i],
                    NPC,
                    NPC,
                    &[0.0, 0.0, 0.0],
                    &(*radiusEnts[i]).currentOrigin,
                    damage,
                    if backhand { 0 } else { 0 }, // DAMAGE_NO_KNOCKBACK
                    0, // MOD_MELEE
                );
                if backhand {
                    //actually push the enemy
                    let mut pushDir = [0.0_f32; 3];
                    let mut angs = [0.0_f32; 3];
                    VectorCopy(&(*(*radiusEnts[i]).client).ps.viewangles, &mut angs);
                    angs[1] += Q_flrand(25.0, 50.0); // YAW
                    angs[0] = Q_flrand(-25.0, -15.0); // PITCH
                    AngleVectors(&angs, &mut pushDir, std::ptr::null_mut(), std::ptr::null_mut());
                    if (*(*radiusEnts[i]).client).NPC_class != 24 // CLASS_WAMPA
                        && (*(*radiusEnts[i]).client).NPC_class != 25 // CLASS_RANCOR
                        && (*(*radiusEnts[i]).client).NPC_class != 22 // CLASS_ATST
                        && ((*radiusEnts[i]).flags & 0x1) == 0
                    {
                        // FL_NO_KNOCKBACK
                        G_Throw(radiusEnts[i], &pushDir, 65);
                        if (*radiusEnts[i]).health > 0 && Q_irand(0, 1) != 0 {
                            //do pain on enemy
                            G_Knockdown(radiusEnts[i], NPC, &pushDir, 300.0, true);
                        }
                    }
                } else if (*radiusEnts[i]).health <= 0 && (*radiusEnts[i]).client != std::ptr::null_mut()
                {
                    //killed them, chance of dismembering
                    if Q_irand(0, 1) == 0 {
                        //bite something off
                        let mut hitLoc = 3; // HL_WAIST
                        if (*g_dismemberment).integer < 4 {
                            hitLoc = Q_irand(9, 15); // HL_BACK_RT to HL_HAND_LT
                        } else {
                            hitLoc = Q_irand(3, 0); // HL_WAIST to HL_HEAD
                        }
                        if hitLoc == 0 {
                            // HL_HEAD
                            NPC_SetAnim(
                                radiusEnts[i],
                                20, // SETANIM_BOTH
                                160, // BOTH_DEATH17
                                0x2 | 0x1, // SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                            );
                        } else if hitLoc == 3 {
                            // HL_WAIST
                            NPC_SetAnim(
                                radiusEnts[i],
                                20, // SETANIM_BOTH
                                145, // BOTH_DEATHBACKWARD2
                                0x2 | 0x1, // SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                            );
                        }
                        (*(*radiusEnts[i]).client).dismembered = 0;
                        //FIXME: the limb should just disappear, cuz I ate it
                        G_DoDismemberment(
                            radiusEnts[i],
                            &(*radiusEnts[i]).currentOrigin,
                            0, // MOD_SABER
                            1000,
                            hitLoc,
                            true,
                        );
                    }
                } else if Q_irand(0, 3) == 0 && (*radiusEnts[i]).health > 0 {
                    //one out of every 4 normal hits does a knockdown, too
                    let mut pushDir = [0.0_f32; 3];
                    let mut angs = [0.0_f32; 3];
                    VectorCopy(&(*(*radiusEnts[i]).client).ps.viewangles, &mut angs);
                    angs[1] += Q_flrand(25.0, 50.0); // YAW
                    angs[0] = Q_flrand(-25.0, -15.0); // PITCH
                    AngleVectors(&angs, &mut pushDir, std::ptr::null_mut(), std::ptr::null_mut());
                    G_Knockdown(radiusEnts[i], NPC, &pushDir, 35.0, true);
                }
                G_Sound(radiusEnts[i], G_SoundIndex(b"sound/chars/rancor/swipehit.wav\0".as_ptr()));
            }
        }
    }
}

//------------------------------
#[allow(non_snake_case)]
pub extern "C" fn Wampa_Attack(distance: f32, doCharge: bool) {
    unsafe {
        if !TIMER_Exists(NPC, b"attacking\0".as_ptr() as *const u8) {
            if Q_irand(0, 3) == 0 && !doCharge {
                //double slash
                NPC_SetAnim(NPC, 20, 23, 0x2 | 0x1); // SETANIM_BOTH, BOTH_ATTACK1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                TIMER_Set(NPC, b"attack_dmg\0".as_ptr() as *const u8, 750);
            } else if doCharge || (distance > 270.0 && distance < 430.0 && Q_irand(0, 1) != 0) {
                //leap
                NPC_SetAnim(NPC, 20, 24, 0x2 | 0x1); // SETANIM_BOTH, BOTH_ATTACK2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                TIMER_Set(NPC, b"attack_dmg\0".as_ptr() as *const u8, 500);
                let mut yawAng = [0.0_f32; 3];
                yawAng[1] = (*NPC).client.cast::<*mut playerState_t>() as *const _ as c_int as f32; // viewangles[YAW]
                let mut fwd = [0.0_f32; 3];
                AngleVectors(&yawAng, &mut fwd, std::ptr::null_mut(), std::ptr::null_mut());
                VectorScale(&fwd, distance * 1.5, &mut (*(*NPC).client.cast::<playerState_t>()).velocity);
                (*(*NPC).client.cast::<playerState_t>()).velocity[2] = 150.0;
                // NPC->client->ps.groundEntityNum = ENTITYNUM_NONE;
            } else if distance < 100.0 {
                //grab
                NPC_SetAnim(NPC, 20, 58, 0x2 | 0x1); // SETANIM_BOTH, BOTH_HOLD_START, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                // NPC->client->ps.legsAnimTimer += 200;
                TIMER_Set(NPC, b"attack_dmg\0".as_ptr() as *const u8, 250);
            } else {
                //backhand
                NPC_SetAnim(NPC, 20, 25, 0x2 | 0x1); // SETANIM_BOTH, BOTH_ATTACK3, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                TIMER_Set(NPC, b"attack_dmg\0".as_ptr() as *const u8, 250);
            }

            // TIMER_Set( NPC, "attacking", NPC->client->ps.legsAnimTimer + random() * 200 );
            TIMER_Set(
                NPC,
                b"attacking\0".as_ptr() as *const u8,
                0, // Stub: placeholder
            );
            //allow us to re-evaluate our running speed/anim
            TIMER_Set(NPC, b"runfar\0".as_ptr() as *const u8, -1);
            TIMER_Set(NPC, b"runclose\0".as_ptr() as *const u8, -1);
            TIMER_Set(NPC, b"walk\0".as_ptr() as *const u8, -1);
        }

        // Need to do delayed damage since the attack animations encapsulate multiple mini-attacks

        if TIMER_Done2(NPC, b"attack_dmg\0".as_ptr() as *const u8, true) {
            // switch ( NPC->client->ps.legsAnim )
            // {
            // case BOTH_ATTACK1:
            Wampa_Slash((*NPC).handRBolt, false);
            //do second hit
            TIMER_Set(NPC, b"attack_dmg2\0".as_ptr() as *const u8, 100);
            // break;
            // case BOTH_ATTACK2:
            Wampa_Slash((*NPC).handRBolt, false);
            TIMER_Set(NPC, b"attack_dmg2\0".as_ptr() as *const u8, 100);
            // break;
            // case BOTH_ATTACK3:
            Wampa_Slash((*NPC).handLBolt, true);
            // break;
            // }
        } else if TIMER_Done2(NPC, b"attack_dmg2\0".as_ptr() as *const u8, true) {
            // switch ( NPC->client->ps.legsAnim )
            // {
            // case BOTH_ATTACK1:
            Wampa_Slash((*NPC).handLBolt, false);
            // break;
            // case BOTH_ATTACK2:
            Wampa_Slash((*NPC).handLBolt, false);
            // break;
            // }
        }

        // Just using this to remove the attacking flag at the right time
        TIMER_Done2(NPC, b"attacking\0".as_ptr() as *const u8, true);

        // if ( NPC->client->ps.legsAnim == BOTH_ATTACK1 && distance > (NPC->maxs[0]+MIN_DISTANCE) )
        // {//okay to keep moving
        //     ucmd.buttons |= BUTTON_WALKING;
        //     Wampa_Move( 1 );
        // }
    }
}

//----------------------------------
#[allow(non_snake_case)]
pub extern "C" fn Wampa_Combat() {
    unsafe {
        // If we cannot see our target or we have somewhere to go, then do that
        if !NPC_ClearLOS((*NPC).enemy) {
            if Q_irand(0, 10) == 0 {
                if Wampa_CheckRoar(NPC) {
                    return;
                }
            }
            // NPCInfo->combatMove = qtrue;
            // NPCInfo->goalEntity = NPC->enemy;
            // NPCInfo->goalRadius = MIN_DISTANCE;//MAX_DISTANCE;	// just get us within combat range

            Wampa_Move(false);
            return;
        }
        /*
        else if ( UpdateGoal() )
        {
            NPCInfo->combatMove = qtrue;
            NPCInfo->goalEntity = NPC->enemy;
            NPCInfo->goalRadius = MIN_DISTANCE;//MAX_DISTANCE;	// just get us within combat range

            Wampa_Move( 1 );
            return;
        }*/

        // Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
        //FIXME: always seems to face off to the left or right?!!!!
        NPC_FaceEnemy(true);

        let distance = Distance(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);
        enemyDist = distance;

        let advance = distance > ((*NPC).maxs[0] + MIN_DISTANCE as f32);
        let mut doCharge = false;

        if advance {
            //have to get closer
            let mut yawOnlyAngles = [0.0_f32; 3];
            yawOnlyAngles[1] = (*NPC).currentAngles[1]; // YAW
            if (*(*NPC).enemy).health > 0//enemy still alive
                && fabs(distance - 350.0) <= 80.0 //enemy anywhere from 270 to 430 away
                && InFOV(
                    &(*(*NPC).enemy).currentOrigin,
                    &(*NPC).currentOrigin,
                    &yawOnlyAngles,
                    20,
                    20,
                )
            //enemy generally in front
            {
                //10% chance of doing charge anim
                if Q_irand(0, 6) == 0 {
                    //go for the charge
                    doCharge = true;
                    // advance = false;
                }
            }
        }

        if (advance || false) && TIMER_Done(NPC, b"attacking\0".as_ptr() as *const u8)
        // waiting monsters can't attack
        {
            if TIMER_Done2(NPC, b"takingPain\0".as_ptr() as *const u8, true) {
                // NPCInfo->localState = LSTATE_CLEAR;
            } else {
                Wampa_Move(true);
            }
        } else {
            if Q_irand(0, 15) == 0 {
                //FIXME: only do this if we just damaged them or vice-versa?
                if Wampa_CheckRoar(NPC) {
                    return;
                }
            }
            Wampa_Attack(distance, doCharge);
        }
    }
}

/*
-------------------------
NPC_Wampa_Pain
-------------------------
*/
#[allow(non_snake_case)]
pub extern "C" fn NPC_Wampa_Pain(
    selff: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const [f32; 3],
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    unsafe {
        let mut hitByWampa = false;
        if (*selff).count != 0 {
            //FIXME: need pain anim
            NPC_SetAnim(selff, 20, 37, 0x2 | 0x1); // SETANIM_BOTH, BOTH_STAND2TO1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
            TIMER_Set(selff, b"takingPain\0".as_ptr() as *const u8, 0); // Stub
            TIMER_Set(
                selff,
                b"attacking\0".as_ptr() as *const u8,
                0, // -level.time
            );
            return;
        }
        if !other.is_null()
            && !(*other).client.is_null()
            && (*(*other).client).NPC_class == 24
        {
            // CLASS_WAMPA
            hitByWampa = true;
        }
        if !other.is_null()
            && (*other).inuse != 0
            && other != (*selff).enemy
            && ((*other).flags & 0x2000) == 0
        {
            // FL_NOTARGET
            if ((*other).s.number == 0 && Q_irand(0, 3) == 0)
                || (*selff).enemy.is_null()
                || (*(*selff).enemy).health == 0
                || (!(*selff).enemy.is_null()
                    && !(*(*selff).enemy).client.is_null()
                    && (*(*(*selff).enemy).client).NPC_class == 24)
                // CLASS_WAMPA
                || (!Q_irand(0, 4) != 0
                    && DistanceSquared(&(*other).currentOrigin, &(*selff).currentOrigin)
                        < DistanceSquared(&(*(*selff).enemy).currentOrigin, &(*selff).currentOrigin))
            {
                //if my enemy is dead (or attacked by player) and I'm not still holding/eating someone, turn on the attacker
                //FIXME: if can't nav to my enemy, take this guy if I can nav to him
                (*selff).lastEnemy = other;
                G_SetEnemy(selff, other);
                if (*selff).enemy != (*selff).lastEnemy {
                    //clear this so that we only sniff the player the first time we pick them up
                    (*selff).useDebounceTime = 0;
                }
                TIMER_Set(
                    selff,
                    b"lookForNewEnemy\0".as_ptr() as *const u8,
                    Q_irand(5000, 15000),
                );
                if hitByWampa {
                    //stay mad at this Wampa for 2-5 secs before looking for other enemies
                    TIMER_Set(
                        selff,
                        b"wampaInfight\0".as_ptr() as *const u8,
                        Q_irand(2000, 5000),
                    );
                }
            }
        }
        if (hitByWampa || Q_irand(0, 100) < damage)
            //hit by wampa, hit while holding live victim, or took a lot of damage
            && (*selff).client.cast::<playerState_t>() as c_int != 0 // ps.legsAnim != BOTH_GESTURE1
            && (*selff).client.cast::<playerState_t>() as c_int != 0 // ps.legsAnim != BOTH_GESTURE2
            && TIMER_Done(selff, b"takingPain\0".as_ptr() as *const u8)
        {
            if !Wampa_CheckRoar(selff) {
                if (*selff).client.cast::<playerState_t>() as c_int != 23 // ps.legsAnim != BOTH_ATTACK1
                    && (*selff).client.cast::<playerState_t>() as c_int != 24 // ps.legsAnim != BOTH_ATTACK2
                    && (*selff).client.cast::<playerState_t>() as c_int != 25
                {
                    // ps.legsAnim != BOTH_ATTACK3
                    //cant interrupt one of the big attack anims
                    if (*selff).health > 100 || hitByWampa {
                        TIMER_Remove(selff, b"attacking\0".as_ptr() as *const u8);

                        // VectorCopy( self->NPC->lastPathAngles, self->s.angles );

                        if Q_irand(0, 1) == 0 {
                            NPC_SetAnim(selff, 20, 35, 0x2 | 0x1); // SETANIM_BOTH, BOTH_PAIN2, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                        } else {
                            NPC_SetAnim(selff, 20, 34, 0x2 | 0x1); // SETANIM_BOTH, BOTH_PAIN1, SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD
                        }
                        TIMER_Set(
                            selff,
                            b"takingPain\0".as_ptr() as *const u8,
                            0, // Stub: self->client->ps.legsAnimTimer+Q_irand(0, 500*(2-g_spskill->integer))
                        );
                        TIMER_Set(
                            selff,
                            b"attacking\0".as_ptr() as *const u8,
                            0, // -level.time
                        );
                        //allow us to re-evaluate our running speed/anim
                        TIMER_Set(selff, b"runfar\0".as_ptr() as *const u8, -1);
                        TIMER_Set(selff, b"runclose\0".as_ptr() as *const u8, -1);
                        TIMER_Set(selff, b"walk\0".as_ptr() as *const u8, -1);

                        // if ( self->NPC )
                        // {
                        //     self->NPC->localState = LSTATE_WAITING;
                        // }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Wampa_DropVictim(selff: *mut gentity_t) {
    unsafe {
        //FIXME: if Wampa dies, it should drop its victim.
        //FIXME: if Wampa is removed, it must remove its victim.
        //FIXME: if in BOTH_HOLD_DROP, throw them a little, too?
        if (*selff).health > 0 {
            NPC_SetAnim(selff, 20, 37, 0x2 | 0x1); // SETANIM_BOTH, BOTH_STAND2TO1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
        }
        TIMER_Set(selff, b"attacking\0".as_ptr() as *const u8, 0); // -level.time
        if !(*selff).activator.is_null() {
            if !(*(*selff).activator).client.is_null() {
                (*(*(*selff).activator).client).ps.eFlags &= !0x4; // EF_HELD_BY_WAMPA
            }
            (*(*selff).activator).activator = std::ptr::null_mut();
            NPC_SetAnim(
                (*selff).activator,
                20,
                74, // SETANIM_BOTH, BOTH_RELEASED
                0x2 | 0x1,
            ); // SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
            // (*(*selff).activator).client->ps.legsAnimTimer += 500;
            // (*(*selff).activator).client->ps.weaponTime = (*(*selff).activator).client->ps.torsoAnimTimer = (*(*selff).activator).client->ps.legsAnimTimer;
            if (*(*selff).activator).health > 0 {
                // if ( self->activator->NPC )
                // {//start thinking again
                //     self->activator->NPC->nextBStateThink = level.time;
                // }
                if !(*(*selff).activator).client.is_null() && (*(*selff).activator).s.number < 64 {
                    // MAX_CLIENTS
                    let mut vicAngles = [30.0_f32, AngleNormalize180((*(*selff).client).ps.viewangles[1] + 180.0), 0.0_f32];
                    SetClientViewAngle((*selff).activator, &vicAngles);
                }
            } else {
                if (*selff).enemy == (*selff).activator {
                    (*selff).enemy = std::ptr::null_mut();
                }
                // (*selff).activator->clipmask &= !CONTENTS_BODY;
            }
            (*selff).activator = std::ptr::null_mut();
        }
        (*selff).count = 0; //drop him
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Wampa_CheckDropVictim(selff: *mut gentity_t, excludeMe: bool) -> bool {
    unsafe {
        if selff.is_null() || (*selff).activator.is_null() {
            return true;
        }
        let mut mins = [
            (*(*selff).activator).mins[0] - 1.0,
            (*(*selff).activator).mins[1] - 1.0,
            0.0,
        ];
        let mut maxs = [
            (*(*selff).activator).maxs[0] + 1.0,
            (*(*selff).activator).maxs[1] + 1.0,
            1.0,
        ];
        let mut start = [
            (*(*selff).activator).currentOrigin[0],
            (*(*selff).activator).currentOrigin[1],
            (*(*selff).activator).absmin[2],
        ];
        let mut end = [
            (*(*selff).activator).currentOrigin[0],
            (*(*selff).activator).currentOrigin[1],
            (*(*selff).activator).absmax[2] - 1.0,
        ];
        let mut trace: trace_t = std::mem::zeroed();
        if excludeMe {
            // gi.unlinkentity( self );
        }
        // gi.trace( &trace, start, mins, maxs, end, self->activator->s.number, self->activator->clipmask );
        if excludeMe {
            // gi.linkentity( self );
        }
        // if ( !trace.allsolid && !trace.startsolid && trace.fraction >= 1.0f )
        // {
        //     Wampa_DropVictim( self );
        //     return qtrue;
        // }
        if excludeMe {
            //victim stuck in wall
            // if ( self->NPC )
            // {//turn
            //     self->NPC->desiredYaw += Q_irand( -30, 30 );
            //     self->NPC->lockedDesiredYaw = self->NPC->desiredYaw;
            // }
        }
        false
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Wampa_TryGrab() -> bool {
    let radius = 64.0_f32;

    unsafe {
        if NPC.is_null()
            || (*NPC).enemy.is_null()
            || !(*(*NPC).enemy).client.is_null()
            || (*(*NPC).enemy).health <= 0
        {
            return false;
        }

        let enemyDist = NPC_EnemyRangeFromBolt((*NPC).handRBolt);
        if enemyDist <= radius
            && (*NPC).count == 0 //don't have one in hand already
            && (*(*(*NPC).enemy).client).NPC_class != 25 // CLASS_RANCOR
            && (*(*(*NPC).enemy).client).NPC_class != 19 // CLASS_GALAKMECH
            && (*(*(*NPC).enemy).client).NPC_class != 22 // CLASS_ATST
            && (*(*(*NPC).enemy).client).NPC_class != 20 // CLASS_GONK
            && (*(*(*NPC).enemy).client).NPC_class != 21 // CLASS_R2D2
            && (*(*(*NPC).enemy).client).NPC_class != 17 // CLASS_R5D2
            && (*(*(*NPC).enemy).client).NPC_class != 14 // CLASS_MARK1
            && (*(*(*NPC).enemy).client).NPC_class != 15 // CLASS_MARK2
            && (*(*(*NPC).enemy).client).NPC_class != 16 // CLASS_MOUSE
            && (*(*(*NPC).enemy).client).NPC_class != 13 // CLASS_PROBE
            && (*(*(*NPC).enemy).client).NPC_class != 10 // CLASS_SEEKER
            && (*(*(*NPC).enemy).client).NPC_class != 11 // CLASS_REMOTE
            && (*(*(*NPC).enemy).client).NPC_class != 12 // CLASS_SENTRY
            && (*(*(*NPC).enemy).client).NPC_class != 9 // CLASS_INTERROGATOR
            && (*(*(*NPC).enemy).client).NPC_class != 26
        {
            // CLASS_VEHICLE
            //grab
            (*NPC).enemy = (*NPC).enemy; //make him my new best friend
            (*(*(*NPC).enemy).client).ps.eFlags |= 0x4; // EF_HELD_BY_WAMPA
            //FIXME: this makes it so that the victim can't hit us with shots!  Just use activator or something
            (*(*NPC).enemy).activator = *NPC; // kind of dumb, but when we are locked to the Rancor, we are owned by it.
            (*NPC).activator = (*NPC).enemy; //remember him
            (*NPC).count = 1; //in my hand
            //wait to attack
            TIMER_Set(
                NPC,
                b"attacking\0".as_ptr() as *const u8,
                0, // NPC->client->ps.legsAnimTimer + Q_irand(500, 2500)
            );
            NPC_SetAnim(
                (*NPC).enemy,
                20,
                51, // SETANIM_BOTH, BOTH_GRABBED
                0x2 | 0x1,
            ); // SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
            NPC_SetAnim(NPC, 20, 55, 0x2 | 0x1); // SETANIM_BOTH, BOTH_HOLD_END, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
            TIMER_Set(NPC, b"takingPain\0".as_ptr() as *const u8, 0); // -level.time
            return true;
        } else if enemyDist < radius * 2.0 {
            //smack
            G_Sound(
                (*NPC).enemy,
                G_SoundIndex(b"sound/chars/rancor/swipehit.wav\0".as_ptr()),
            );
            //actually push the enemy
            let mut pushDir = [0.0_f32; 3];
            let mut angs = [0.0_f32; 3];
            VectorCopy(&(*(*NPC).client).ps.viewangles, &mut angs);
            angs[1] += Q_flrand(25.0, 50.0); // YAW
            angs[0] = Q_flrand(-25.0, -15.0); // PITCH
            AngleVectors(&angs, &mut pushDir, std::ptr::null_mut(), std::ptr::null_mut());
            if (*(*(*NPC).enemy).client).NPC_class != 25 // CLASS_RANCOR
                && (*(*(*NPC).enemy).client).NPC_class != 22 // CLASS_ATST
                && ((*(*NPC).enemy).flags & 0x1) == 0
            {
                // FL_NO_KNOCKBACK
                G_Throw((*NPC).enemy, &pushDir, Q_irand(30, 70));
                if (*(*NPC).enemy).health > 0 {
                    //do pain on enemy
                    G_Knockdown((*NPC).enemy, NPC, &pushDir, 300.0, true);
                }
            }
        }
        false
    }
}

/*
-------------------------
NPC_BSWampa_Default
-------------------------
*/
#[allow(non_snake_case)]
pub extern "C" fn NPC_BSWampa_Default() {
    //NORMAL ANIMS
    //	stand1 = normal stand
    //	walk1 = normal, non-angry walk
    //	walk2 = injured
    //	run1 = far away run
    //	run2 = close run
    //VICTIM ANIMS
    //	grabswipe = melee1 - sweep out and grab
    //	stand2 attack = attack4 - while holding victim, swipe at him
    //	walk3_drag = walk5 - walk with drag
    //	stand2 = hold victim
    //	stand2to1 = drop victim
    unsafe {
        if (*NPC).client.cast::<playerState_t>() as c_int == 58 {
            // ps.legsAnim == BOTH_HOLD_START
            NPC_FaceEnemy(true);
            // if ( NPC->client->ps.legsAnimTimer < 200 )
            // {//see if he's there to grab
            if !Wampa_TryGrab() {
                NPC_SetAnim(NPC, 20, 52, 0x2 | 0x1); // SETANIM_BOTH, BOTH_HOLD_MISS, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
            }
            // }
            return;
        }

        if (*NPC).count != 0 {
            if (*NPC).activator.is_null() || (*(*NPC).activator).client.is_null() {
                //wtf?
                (*NPC).count = 0;
                (*NPC).activator = std::ptr::null_mut();
            } else {
                if (*NPC).client.cast::<playerState_t>() as c_int == 60 {
                    // ps.legsAnim == BOTH_HOLD_DROP
                    // if ( NPC->client->ps.legsAnimTimer < PM_AnimLength(NPC->client->clientInfo.animFileIndex, (animNumber_t)NPC->client->ps.legsAnim)-500 )
                    // {//at least half a second into the anim
                    if Wampa_CheckDropVictim(NPC, false) {
                        TIMER_Set(
                            NPC,
                            b"attacking\0".as_ptr() as *const u8,
                            1000 + (Q_irand(500, 1000) * (3 - 2)), // (3-g_spskill->integer)
                        );
                    }
                    // }
                } else if !TIMER_Done(NPC, b"takingPain\0".as_ptr() as *const u8) {
                    Wampa_CheckDropVictim(NPC, false);
                } else if (*(*NPC).activator).health <= 0 {
                    if TIMER_Done(NPC, b"sniffCorpse\0".as_ptr() as *const u8) {
                        Wampa_CheckDropVictim(NPC, false);
                    }
                } else if (*NPC).useDebounceTime >= 0 && !(*NPC).activator.is_null()
                {
                    //just sniffing the guy
                    if (*NPC).useDebounceTime <= 0 + 100 && (*NPC).client.cast::<playerState_t>() as c_int != 60
                    {
                        // ps.legsAnim != BOTH_HOLD_DROP
                        //just about done, drop him
                        NPC_SetAnim(NPC, 20, 60, 0x2 | 0x1); // SETANIM_BOTH, BOTH_HOLD_DROP, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
                        TIMER_Set(
                            NPC,
                            b"attacking\0".as_ptr() as *const u8,
                            0, // NPC->client->ps.legsAnimTimer+500
                        );
                    }
                } else {
                    if (*NPC).useDebounceTime == 0 && !(*NPC).activator.is_null() && (*(*NPC).activator).s.number < 64
                    {
                        // MAX_CLIENTS
                        //first time I pick the player, just sniff them
                        if TIMER_Done(NPC, b"attacking\0".as_ptr() as *const u8) {
                            //ready to attack
                            NPC_SetAnim(NPC, 20, 54, 0x2 | 0x1); // SETANIM_BOTH, BOTH_HOLD_SNIFF, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
                            (*NPC).useDebounceTime = 0 + 0 + Q_irand(500, 2000); // level.time + NPC->client->ps.legsAnimTimer
                        }
                    } else {
                        if TIMER_Done(NPC, b"attacking\0".as_ptr() as *const u8) {
                            //ready to attack
                            NPC_SetAnim(
                                NPC,
                                20,
                                56, // SETANIM_BOTH, BOTH_HOLD_ATTACK/*BOTH_ATTACK4*/
                                0x2 | 0x1,
                            ); // SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
                            TIMER_Set(NPC, b"grabAttackDamage\0".as_ptr() as *const u8, 1400);
                            TIMER_Set(
                                NPC,
                                b"attacking\0".as_ptr() as *const u8,
                                0, // NPC->client->ps.legsAnimTimer+Q_irand(3000,10000)
                            );
                        }

                        if (*NPC).client.cast::<playerState_t>() as c_int == 56 {
                            // ps.legsAnim == BOTH_HOLD_ATTACK
                            // if ( NPC->client->ps.legsAnimTimer )
                            // {
                            if TIMER_Done2(NPC, b"grabAttackDamage\0".as_ptr() as *const u8, true) {
                                G_Sound(
                                    (*NPC).activator,
                                    G_SoundIndex(b"sound/chars/rancor/swipehit.wav\0".as_ptr()),
                                );
                                G_Damage(
                                    (*NPC).activator,
                                    NPC,
                                    NPC,
                                    &[0.0, 0.0, 0.0],
                                    &(*(*NPC).activator).currentOrigin,
                                    Q_irand(25, 40),
                                    0x1 | 0x2, // DAMAGE_NO_KNOCKBACK|DAMAGE_NO_ARMOR
                                    0, // MOD_MELEE
                                );
                                if (*(*NPC).activator).health <= 0 {
                                    //killed them, chance of dismembering
                                    let mut hitLoc = 3; // HL_WAIST
                                    if (*g_dismemberment).integer < 4 {
                                        hitLoc = Q_irand(9, 15); // HL_BACK_RT to HL_HAND_LT
                                    } else {
                                        hitLoc = Q_irand(3, 0); // HL_WAIST to HL_HEAD
                                    }
                                    (*(*(*NPC).activator).client).dismembered = 0;
                                    //FIXME: the limb should just disappear, cuz I ate it
                                    G_DoDismemberment(
                                        (*NPC).activator,
                                        &(*(*NPC).activator).currentOrigin,
                                        0, // MOD_SABER
                                        1000,
                                        hitLoc,
                                        true,
                                    );
                                    TIMER_Set(
                                        NPC,
                                        b"sniffCorpse\0".as_ptr() as *const u8,
                                        Q_irand(2000, 5000),
                                    );
                                }
                                NPC_SetAnim(
                                    (*NPC).activator,
                                    20,
                                    87, // SETANIM_BOTH, BOTH_HANG_PAIN
                                    0x2 | 0x1,
                                ); // SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
                            }
                            // }
                            // else
                            // {
                            //     NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_HOLD_IDLE/*BOTH_ATTACK4*/, SETANIM_FLAG_NORMAL );
                            // }
                        } else if (*NPC).client.cast::<playerState_t>() as c_int == 37 && false
                        {
                            // ps.legsAnim == BOTH_STAND2TO1 && !NPC->client->ps.legsAnimTimer
                            NPC_SetAnim(NPC, 20, 57, 0x4); // SETANIM_BOTH, BOTH_HOLD_IDLE, SETANIM_FLAG_NORMAL
                        }
                    }
                }
            }

            NPC_UpdateAngles(true, true);
            return;
        }

        // if ( NPCInfo->localState == LSTATE_WAITING
        //     && TIMER_Done2( NPC, "takingPain", qtrue ) )
        // {//was not doing anything because we were taking pain, but pain is done now, so clear it...
        //     NPCInfo->localState = LSTATE_CLEAR;
        // }

        if !TIMER_Done(NPC, b"rageTime\0".as_ptr() as *const u8) {
            //do nothing but roar first time we see an enemy
            NPC_FaceEnemy(true);
            return;
        }
        if !(*NPC).enemy.is_null() {
            if !(*(*NPC).enemy).client.is_null() //enemy is a client
                && ((*(*(*NPC).enemy).client).NPC_class == 7 || (*(*(*NPC).enemy).client).NPC_class == 8) //enemy is a lowly jawa or ugnaught
                && (*(*NPC).enemy).enemy != NPC //enemy's enemy is not me
                && ((*(*NPC).enemy).enemy.is_null()
                    || (*(*(*(*NPC).enemy).enemy).client).is_null()
                    || (*(*(*(*(*NPC).enemy).enemy).client).NPC_class != 25)
            {
                // enemy's enemy is not a client or is not a rancor (which is scarier than me)
                //they should be scared of ME and no-one else
                G_SetEnemy((*NPC).enemy, NPC);
            }
            if !TIMER_Done(NPC, b"attacking\0".as_ptr() as *const u8) {
                //in middle of attack
                //face enemy
                NPC_FaceEnemy(true);
                //continue attack logic
                enemyDist = Distance(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);
                Wampa_Attack(enemyDist, false);
                return;
            } else {
                if TIMER_Done(NPC, b"angrynoise\0".as_ptr() as *const u8) {
                    G_SoundOnEnt(
                        NPC,
                        0, // CHAN_AUTO
                        b"sound/chars/wampa/misc/anger1.wav\0".as_ptr(),
                    );

                    TIMER_Set(NPC, b"angrynoise\0".as_ptr() as *const u8, Q_irand(5000, 10000));
                }
                //else, if he's in our hand, we eat, else if he's on the ground, we keep attacking his dead body for a while
                if !(*(*NPC).enemy).client.is_null()
                    && (*(*(*NPC).enemy).client).NPC_class == 24
                {
                    // CLASS_WAMPA
                    //got mad at another Wampa, look for a valid enemy
                    if TIMER_Done(NPC, b"wampaInfight\0".as_ptr() as *const u8) {
                        NPC_CheckEnemyExt(true);
                    }
                } else {
                    if !NPC_ValidEnemy((*NPC).enemy) {
                        TIMER_Remove(NPC, b"lookForNewEnemy\0".as_ptr() as *const u8); //make them look again right now
                        if !(*(*NPC).enemy).inuse != 0
                            || 0 - (*(*NPC).enemy).s.time > Q_irand(10000, 15000)
                        {
                            // level.time - NPC->enemy->s.time
                            //it's been a while since the enemy died, or enemy is completely gone, get bored with him
                            (*NPC).enemy = std::ptr::null_mut();
                            Wampa_Patrol();
                            NPC_UpdateAngles(true, true);
                            return;
                        }
                    }
                    if TIMER_Done(NPC, b"lookForNewEnemy\0".as_ptr() as *const u8) {
                        let sav_enemy = (*NPC).enemy; //FIXME: what about NPC->lastEnemy?
                        (*NPC).enemy = std::ptr::null_mut();
                        let newEnemy = NPC_CheckEnemy(false, false, false); // NPCInfo->confusionTime < level.time
                        (*NPC).enemy = sav_enemy;
                        if !newEnemy.is_null() && newEnemy != sav_enemy {
                            //picked up a new enemy!
                            (*NPC).lastEnemy = (*NPC).enemy;
                            G_SetEnemy(NPC, newEnemy);
                            if (*NPC).enemy != (*NPC).lastEnemy {
                                //clear this so that we only sniff the player the first time we pick them up
                                (*NPC).useDebounceTime = 0;
                            }
                            //hold this one for at least 5-15 seconds
                            TIMER_Set(
                                NPC,
                                b"lookForNewEnemy\0".as_ptr() as *const u8,
                                Q_irand(5000, 15000),
                            );
                        } else {
                            //look again in 2-5 secs
                            TIMER_Set(
                                NPC,
                                b"lookForNewEnemy\0".as_ptr() as *const u8,
                                Q_irand(2000, 5000),
                            );
                        }
                    }
                }
                Wampa_Combat();
                return;
            }
        } else {
            if TIMER_Done(NPC, b"idlenoise\0".as_ptr() as *const u8) {
                G_SoundOnEnt(NPC, 0, b"sound/chars/wampa/misc/anger3.wav\0".as_ptr()); // CHAN_AUTO

                TIMER_Set(NPC, b"idlenoise\0".as_ptr() as *const u8, Q_irand(2000, 4000));
            }
            // if ( NPCInfo->scriptFlags & SCF_LOOK_FOR_ENEMIES )
            // {
            //     Wampa_Patrol();
            // }
            // else
            // {
            //     Wampa_Idle();
            // }
        }

        NPC_UpdateAngles(true, true);
    }
}
