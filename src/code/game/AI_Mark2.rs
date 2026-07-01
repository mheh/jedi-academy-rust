// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

#![allow(non_snake_case)]

use crate::code::game::g_headers_h::*;
use crate::code::game::b_local_h::*;
use crate::code::game::g_nav_h::*;

use core::ffi::{c_char, c_float, c_int};
use core::ptr::{addr_of, addr_of_mut, null_mut};

//#define AMMO_POD_HEALTH				40
const AMMO_POD_HEALTH: c_int = 1;
const TURN_OFF: c_int = 0x00000100;

const VELOCITY_DECAY: c_float = 0.25;
const MAX_DISTANCE: c_int = 256;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;
const MIN_DISTANCE: c_int = 24;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

extern "C" {
    fn FindItemForAmmo(ammo: ammo_t) -> *mut gitem_t;
}

//Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_DROPPINGDOWN: c_int = 1;
const LSTATE_DOWN: c_int = 2;
const LSTATE_RISINGUP: c_int = 3;

extern "C" {
    fn CreateMissile(
        org: *mut vec3_t,
        dir: *mut vec3_t,
        vel: c_float,
        life: c_int,
        owner: *mut gentity_t,
        altFire: qboolean,
    ) -> *mut gentity_t;
}

pub unsafe extern "C" fn NPC_Mark2_Precache() {
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char); // blows up on death
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_pain\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_fire\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_move_lp\0".as_ptr() as *const c_char);

    G_EffectIndex(b"explosions/droidexplosion1\0".as_ptr() as *const c_char);
    G_EffectIndex(b"env/med_explode2\0".as_ptr() as *const c_char);
    G_EffectIndex(b"blaster/smoke_bolton\0".as_ptr() as *const c_char);
    G_EffectIndex(b"bryar/muzzle_flash\0".as_ptr() as *const c_char);

    RegisterItem(FindItemForWeapon(WP_BRYAR_PISTOL));
    RegisterItem(FindItemForAmmo(AMMO_METAL_BOLTS));
    RegisterItem(FindItemForAmmo(AMMO_POWERCELL));
    RegisterItem(FindItemForAmmo(AMMO_BLASTER));
}

/*
-------------------------
NPC_Mark2_Part_Explode
-------------------------
*/
pub unsafe extern "C" fn NPC_Mark2_Part_Explode(self_: *mut gentity_t, bolt: c_int) {
    if bolt >= 0 {
        let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        ((*addr_of!(gi)).G2API_GetBoltMatrix)(
            (*self_).ghoul2,
            (*self_).playerModel,
            bolt,
            addr_of_mut!(boltMatrix),
            (*self_).currentAngles,
            (*self_).currentOrigin,
            if (*addr_of!(cg)).time != 0 { (*addr_of!(cg)).time } else { (*addr_of!(level)).time },
            null_mut(),
            (*self_).s.modelScale,
        );

        ((*addr_of!(gi)).G2API_GiveMeVectorFromMatrix)(boltMatrix, ORIGIN, org.as_mut_ptr());
        ((*addr_of!(gi)).G2API_GiveMeVectorFromMatrix)(boltMatrix, NEGATIVE_Y, dir.as_mut_ptr());

        G_PlayEffect(b"env/med_explode2\0".as_ptr() as *const c_char, org.as_mut_ptr(), dir.as_mut_ptr());
        // C++ overload: G_PlayEffect( int fxID, int ghoul2Model, int boltIndex, int entNum, vec3_t origin )
        G_PlayEffect(
            G_EffectIndex(b"blaster/smoke_bolton\0".as_ptr() as *const c_char),
            (*self_).playerModel,
            bolt,
            (*self_).s.number,
            org.as_mut_ptr(),
        );
    }

    (*self_).count += 1; // Count of pods blown off
}

/*
-------------------------
NPC_Mark2_Pain
- look at what was hit and see if it should be removed from the model.
-------------------------
*/
pub unsafe extern "C" fn NPC_Mark2_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const vec3_t,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    let mut newBolt: c_int;
    let mut i: c_int;

    NPC_Pain(self_, inflictor, other, point, damage, mod_);

    i = 0;
    while i < 3 {
        if (hitLoc == HL_GENERIC1 + i)
            && ((*self_).locationDamage[(HL_GENERIC1 + i) as usize] > AMMO_POD_HEALTH)
        // Blow it up?
        {
            if (*self_).locationDamage[hitLoc as usize] >= AMMO_POD_HEALTH {
                newBolt = ((*addr_of!(gi)).G2API_AddBolt)(
                    addr_of_mut!((*self_).ghoul2[(*self_).playerModel as usize]),
                    va(b"torso_canister%d\0".as_ptr() as *const c_char, i + 1),
                );
                if newBolt != -1 {
                    NPC_Mark2_Part_Explode(self_, newBolt);
                }
                ((*addr_of!(gi)).G2API_SetSurfaceOnOff)(
                    addr_of_mut!((*self_).ghoul2[(*self_).playerModel as usize]),
                    va(b"torso_canister%d\0".as_ptr() as *const c_char, i + 1),
                    TURN_OFF,
                );
                break;
            }
        }
        i += 1;
    }

    G_Sound(self_, G_SoundIndex(b"sound/chars/mark2/misc/mark2_pain\0".as_ptr() as *const c_char));

    // If any pods were blown off, kill him
    if (*self_).count > 0 {
        G_Damage(
            self_,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            (*self_).health,
            DAMAGE_NO_PROTECTION,
            MOD_UNKNOWN,
        );
    }
}

/*
-------------------------
Mark2_Hunt
-------------------------
*/
pub unsafe extern "C" fn Mark2_Hunt() {
    if (*NPCInfo).goalEntity.is_null() {
        (*NPCInfo).goalEntity = (*NPC).enemy;
    }

    // Turn toward him before moving towards him.
    NPC_FaceEnemy(qtrue);

    (*NPCInfo).combatMove = qtrue;
    NPC_MoveToGoal(qtrue);
}

/*
-------------------------
Mark2_FireBlaster
-------------------------
*/
pub unsafe extern "C" fn Mark2_FireBlaster(advance: qboolean) {
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut delta1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    static mut forward: [f32; 3] = [0.0; 3];
    static mut vright: [f32; 3] = [0.0; 3];
    static mut up: [f32; 3] = [0.0; 3];
    static mut muzzle: [f32; 3] = [0.0; 3];
    let missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

    ((*addr_of!(gi)).G2API_GetBoltMatrix)(
        (*NPC).ghoul2,
        (*NPC).playerModel,
        (*NPC).genericBolt1,
        addr_of_mut!(boltMatrix),
        (*NPC).currentAngles,
        (*NPC).currentOrigin,
        if (*addr_of!(cg)).time != 0 { (*addr_of!(cg)).time } else { (*addr_of!(level)).time },
        null_mut(),
        (*NPC).s.modelScale,
    );

    ((*addr_of!(gi)).G2API_GiveMeVectorFromMatrix)(boltMatrix, ORIGIN, muzzle1.as_mut_ptr());

    if (*NPC).health != 0 {
        CalcEntitySpot((*NPC).enemy, SPOT_HEAD, enemy_org1.as_mut_ptr());
        VectorSubtract(enemy_org1.as_ptr(), muzzle1.as_ptr(), delta1.as_mut_ptr());
        vectoangles(delta1.as_ptr(), angleToEnemy1.as_mut_ptr());
        AngleVectors(
            angleToEnemy1.as_ptr(),
            forward.as_mut_ptr(),
            vright.as_mut_ptr(),
            up.as_mut_ptr(),
        );
    } else {
        AngleVectors(
            (*NPC).currentAngles.as_ptr(),
            forward.as_mut_ptr(),
            vright.as_mut_ptr(),
            up.as_mut_ptr(),
        );
    }

    G_PlayEffect(b"bryar/muzzle_flash\0".as_ptr() as *const c_char, muzzle1.as_mut_ptr(), forward.as_mut_ptr());

    G_Sound(NPC, G_SoundIndex(b"sound/chars/mark2/misc/mark2_fire\0".as_ptr() as *const c_char));

    missile = CreateMissile(addr_of_mut!(muzzle1), addr_of_mut!(forward) as *mut vec3_t, 1600.0, 10000, NPC, qfalse);

    (*missile).classname = b"bryar_proj\0".as_ptr() as *const c_char;
    (*missile).s.weapon = WP_BRYAR_PISTOL;

    (*missile).damage = 1;
    (*missile).dflags = DAMAGE_DEATH_KNOCKBACK;
    (*missile).methodOfDeath = MOD_ENERGY;
    (*missile).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
Mark2_BlasterAttack
-------------------------
*/
pub unsafe extern "C" fn Mark2_BlasterAttack(advance: qboolean) {
    if TIMER_Done(NPC, b"attackDelay\0".as_ptr() as *const c_char) != 0 {
        // Attack?
        if (*NPCInfo).localState == LSTATE_NONE {
            // He's up so shoot less often.
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(500, 2000));
        } else {
            TIMER_Set(NPC, b"attackDelay\0".as_ptr() as *const c_char, Q_irand(100, 500));
        }
        Mark2_FireBlaster(advance);
        return;
    } else if advance != 0 {
        Mark2_Hunt();
    }
}

/*
-------------------------
Mark2_AttackDecision
-------------------------
*/
pub unsafe extern "C" fn Mark2_AttackDecision() {
    NPC_FaceEnemy(qtrue);

    let distance: c_float =
        DistanceHorizontalSquared((*NPC).currentOrigin.as_ptr(), (*(*NPC).enemy).currentOrigin.as_ptr())
            as c_int as c_float;
    let visible: qboolean = NPC_ClearLOS((*NPC).enemy);
    let advance: qboolean = (distance > MIN_DISTANCE_SQR as c_float) as qboolean;

    // He's been ordered to get up
    if (*NPCInfo).localState == LSTATE_RISINGUP {
        (*NPC).flags &= !FL_SHIELDED;
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_RUN1START,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        if (*(*NPC).client).ps.legsAnimTimer == 0
            && (*(*NPC).client).ps.torsoAnim == BOTH_RUN1START
        {
            (*NPCInfo).localState = LSTATE_NONE; // He's up again.
        }
        return;
    }

    // If we cannot see our target, move to see it
    if visible == 0 || NPC_FaceEnemy(qtrue) == 0 {
        // If he's going down or is down, make him get up
        if (*NPCInfo).localState == LSTATE_DOWN || (*NPCInfo).localState == LSTATE_DROPPINGDOWN {
            if TIMER_Done(NPC, b"downTime\0".as_ptr() as *const c_char) != 0 {
                // Down being down?? (The delay is so he doesn't pop up and down when the player goes in and out of range)
                (*NPCInfo).localState = LSTATE_RISINGUP;
                NPC_SetAnim(
                    NPC,
                    SETANIM_BOTH,
                    BOTH_RUN1STOP,
                    SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
                );
                TIMER_Set(NPC, b"runTime\0".as_ptr() as *const c_char, Q_irand(3000, 8000)); // So he runs for a while before testing to see if he should drop down.
            }
        } else {
            Mark2_Hunt();
        }
        return;
    }

    // He's down but he could advance if he wants to.
    if advance != 0
        && TIMER_Done(NPC, b"downTime\0".as_ptr() as *const c_char) != 0
        && (*NPCInfo).localState == LSTATE_DOWN
    {
        (*NPCInfo).localState = LSTATE_RISINGUP;
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_RUN1STOP,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        TIMER_Set(NPC, b"runTime\0".as_ptr() as *const c_char, Q_irand(3000, 8000)); // So he runs for a while before testing to see if he should drop down.
    }

    NPC_FaceEnemy(qtrue);

    // Dropping down to shoot
    if (*NPCInfo).localState == LSTATE_DROPPINGDOWN {
        NPC_SetAnim(
            NPC,
            SETANIM_BOTH,
            BOTH_RUN1STOP,
            SETANIM_FLAG_HOLD | SETANIM_FLAG_OVERRIDE,
        );
        TIMER_Set(NPC, b"downTime\0".as_ptr() as *const c_char, Q_irand(3000, 9000));

        if (*(*NPC).client).ps.legsAnimTimer == 0
            && (*(*NPC).client).ps.torsoAnim == BOTH_RUN1STOP
        {
            (*NPC).flags |= FL_SHIELDED;
            (*NPCInfo).localState = LSTATE_DOWN;
        }
    }
    // He's down and shooting
    else if (*NPCInfo).localState == LSTATE_DOWN {
        //		NPC->flags |= FL_SHIELDED;//only damagable by lightsabers and missiles

        Mark2_BlasterAttack(qfalse);
    } else if TIMER_Done(NPC, b"runTime\0".as_ptr() as *const c_char) != 0 {
        // Lowering down to attack. But only if he's done running at you.
        (*NPCInfo).localState = LSTATE_DROPPINGDOWN;
    } else if advance != 0 {
        // We can see enemy so shoot him if timer lets you.
        Mark2_BlasterAttack(advance);
    }
}


/*
-------------------------
Mark2_Patrol
-------------------------
*/
pub unsafe extern "C" fn Mark2_Patrol() {
    if NPC_CheckPlayerTeamStealth() != 0 {
        //		G_Sound( NPC, G_SoundIndex("sound/chars/mark1/misc/anger.wav"));
        NPC_UpdateAngles(qtrue, qtrue);
        return;
    }

    //If we have somewhere to go, then do that
    if (*NPC).enemy.is_null() {
        if UpdateGoal() != 0 {
            (*addr_of_mut!(ucmd)).buttons |= BUTTON_WALKING;
            NPC_MoveToGoal(qtrue);
            NPC_UpdateAngles(qtrue, qtrue);
        }

        //randomly talk
        if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
            //			G_Sound( NPC, G_SoundIndex(va("sound/chars/mark1/misc/talk%d.wav",	Q_irand(1, 4))));

            TIMER_Set(NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand(2000, 4000));
        }
    }
}

/*
-------------------------
Mark2_Idle
-------------------------
*/
pub unsafe extern "C" fn Mark2_Idle() {
    NPC_BSIdle();
}

/*
-------------------------
NPC_BSMark2_Default
-------------------------
*/
pub unsafe extern "C" fn NPC_BSMark2_Default() {
    if !(*NPC).enemy.is_null() {
        (*NPCInfo).goalEntity = (*NPC).enemy;
        Mark2_AttackDecision();
    } else if (*NPCInfo).scriptFlags & SCF_LOOK_FOR_ENEMIES != 0 {
        Mark2_Patrol();
    } else {
        Mark2_Idle();
    }
}
