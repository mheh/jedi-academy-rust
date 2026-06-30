// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
#![allow(non_snake_case)]

use crate::code::game::g_headers::*;    // g_headers.h
use crate::code::game::b_local_h::*;    // b_local.h
use core::ffi::{c_int, c_char};
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

// Constants
const TURN_OFF: u32 = 0x00000100;

// Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_BACKINGUP: c_int = 1;
const LSTATE_SPINNING: c_int = 2;
const LSTATE_PAIN: c_int = 3;
const LSTATE_DROP: c_int = 4;

// External declarations - game engine functions
extern "C" {
    fn NPC_GetPainChance(self_: *mut gentity_t, damage: c_int) -> f32;
    fn TIMER_Done(ent: *mut gentity_t, timer: *const c_char) -> i32;
    fn TIMER_Set(ent: *mut gentity_t, timer: *const c_char, duration: c_int);
    fn AngleNormalize360(angle: f32) -> f32;
    fn AngleDelta(angle1: f32, angle2: f32) -> f32;
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn random() -> f32;
    fn crandom() -> f32;
    fn NPC_SetAnim(ent: *mut gentity_t, setanim_type: c_int, anim: c_int, flags: c_int);
    fn NPC_MoveToGoal(allowWalk: i32) -> i32;
    fn UpdateGoal() -> i32;
    fn NPC_UpdateAngles(reset: i32, useDesired: i32);
    fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundfile: *const c_char);
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn G_PlayEffect(effect: *const c_char, origin: *const [f32; 3], dir: *const [f32; 3]);
    fn G_SoundIndex(soundfile: *const c_char) -> c_int;
    fn G_EffectIndex(effectfile: *const c_char) -> c_int;
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn VectorMA(start: *const [f32; 3], scale: f32, dir: *const [f32; 3], out: *mut [f32; 3]);
    fn NPC_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int);

    // Global game state (accessed via addr_of! / addr_of_mut!)
    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut gNPC_t;
    pub static mut level: level_locals_t;
    pub static mut ucmd: usercmd_t;
    pub static mut gi: gameImport_t;
}

/*
-------------------------
R2D2_PartsMove
-------------------------
*/
pub unsafe fn R2D2_PartsMove() {
    // Front 'eye' lense
    if TIMER_Done(NPC, b"eyeDelay\0".as_ptr() as *const c_char) != 0 {
        (*NPC).pos1[1] = AngleNormalize360((*NPC).pos1[1]);

        (*NPC).pos1[0] += Q_irand(-20, 20) as f32;	// Roll
        (*NPC).pos1[1] = Q_irand(-20, 20) as f32;
        (*NPC).pos1[2] = Q_irand(-20, 20) as f32;

        if (*NPC).genericBone1 != std::ptr::null_mut() {
            // gi.G2API_SetBoneAnglesIndex( &NPC->ghoul2[NPC->playerModel], NPC->genericBone1, NPC->pos1, BONE_ANGLES_POSTMULT, POSITIVE_X, NEGATIVE_Y, NEGATIVE_Z, NULL );
        }
        TIMER_Set( NPC, b"eyeDelay\0".as_ptr() as *const c_char, Q_irand( 100, 1000 ) );
    }
}

/*
-------------------------
NPC_BSDroid_Idle
-------------------------
*/
pub unsafe fn Droid_Idle() {
    //	VectorCopy( NPCInfo->investigateGoal, lookPos );

    //	NPC_FacePosition( lookPos );
}

/*
-------------------------
R2D2_TurnAnims
-------------------------
*/
pub unsafe fn R2D2_TurnAnims() {
    let mut turndelta: f32;
    let mut anim: c_int;

    turndelta = AngleDelta((*NPC).currentAngles[1 as usize], (*NPCInfo).desiredYaw);

    if (turndelta.abs() > 20.0) && (((*(*NPC).client).NPC_class == 3 /* CLASS_R2D2 */) || ((*(*NPC).client).NPC_class == 4 /* CLASS_R5D2 */)) {
        anim = (*(*NPC).client).ps.legsAnim;
        if turndelta < 0.0 {
            if anim != 24 /* BOTH_TURN_LEFT1 */ {
                NPC_SetAnim( NPC, 3 /* SETANIM_BOTH */, 24 /* BOTH_TURN_LEFT1 */, (1 << 6) /* SETANIM_FLAG_OVERRIDE */ | (1 << 7) /* SETANIM_FLAG_HOLD */ );
            }
        } else {
            if anim != 25 /* BOTH_TURN_RIGHT1 */ {
                NPC_SetAnim( NPC, 3 /* SETANIM_BOTH */, 25 /* BOTH_TURN_RIGHT1 */, (1 << 6) | (1 << 7) );
            }
        }
    } else {
        NPC_SetAnim( NPC, 3 /* SETANIM_BOTH */, 26 /* BOTH_RUN1 */, (1 << 6) | (1 << 7) );
    }
}

/*
-------------------------
Droid_Patrol
-------------------------
*/
pub unsafe fn Droid_Patrol() {

    (*NPC).pos1[1] = AngleNormalize360( (*NPC).pos1[1]);

    if !NPC.is_null() && !(*NPC).client.is_null() && (*(*NPC).client).NPC_class != 6 /* CLASS_GONK */ {
        R2D2_PartsMove();		// Get his eye moving.
        R2D2_TurnAnims();
    }

    //If we have somewhere to go, then do that
    if UpdateGoal() != 0 {
        (*addr_of_mut!(ucmd)).buttons |= 1 << 3; /* BUTTON_WALKING */
        NPC_MoveToGoal( 1 );

        if !NPC.is_null() && !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == 5 /* CLASS_MOUSE */ {
            (*NPCInfo).desiredYaw += (((*addr_of!(level)).time as f32) * 0.5).sin() * 25.0; // Weaves side to side a little

            if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
                G_SoundOnEnt( NPC, 0 /* CHAN_AUTO */, va(b"sound/chars/mouse/misc/mousego%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)) );

                TIMER_Set( NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand( 2000, 4000 ) );
            }
        } else if !NPC.is_null() && !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == 3 /* CLASS_R2D2 */ {
            if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
                G_SoundOnEnt( NPC, 0 /* CHAN_AUTO */, va(b"sound/chars/r2d2/misc/r2d2talk0%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 3)) );

                TIMER_Set( NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand( 2000, 4000 ) );
            }
        } else if !NPC.is_null() && !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == 4 /* CLASS_R5D2 */ {
            if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
                G_SoundOnEnt( NPC, 0 /* CHAN_AUTO */, va(b"sound/chars/r5d2/misc/r5talk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 4)) );

                TIMER_Set( NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand( 2000, 4000 ) );
            }
        }
        if !NPC.is_null() && !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == 6 /* CLASS_GONK */ {
            if TIMER_Done(NPC, b"patrolNoise\0".as_ptr() as *const c_char) != 0 {
                G_SoundOnEnt( NPC, 0 /* CHAN_AUTO */, va(b"sound/chars/gonk/misc/gonktalk%d.wav\0".as_ptr() as *const c_char, Q_irand(1, 2)) );

                TIMER_Set( NPC, b"patrolNoise\0".as_ptr() as *const c_char, Q_irand( 2000, 4000 ) );
            }
        }
        //		else
        //		{
        //			R5D2_LookAround();
        //		}
    }

    NPC_UpdateAngles( 1, 1 );

}

/*
-------------------------
Droid_Run
-------------------------
*/
pub unsafe fn Droid_Run() {
    R2D2_PartsMove();

    if (*NPCInfo).localState == LSTATE_BACKINGUP {
        (*addr_of_mut!(ucmd)).forwardmove = -127;
        (*NPCInfo).desiredYaw += 5.0;

        (*NPCInfo).localState = LSTATE_NONE;	// So he doesn't constantly backup.
    } else {
        (*addr_of_mut!(ucmd)).forwardmove = 64;
        //If we have somewhere to go, then do that
        if UpdateGoal() != 0 {
            if NPC_MoveToGoal( 0 ) != 0 {
                (*NPCInfo).desiredYaw += (((*addr_of!(level)).time as f32) * 0.5).sin() * 5.0; // Weaves side to side a little
            }
        }
    }

    NPC_UpdateAngles( 1, 1 );
}

/*
-------------------------
void Droid_Spin( void )
-------------------------
*/
pub unsafe fn Droid_Spin() {
    let mut dir: [f32; 3] = [0.0, 0.0, 1.0];

    R2D2_TurnAnims();


    // Head is gone, spin and spark
    if (*(*NPC).client).NPC_class == 4 /* CLASS_R5D2 */ {
        // No head?
        if (*NPC).ghoul2_render_status[(*NPC).playerModel as usize] != 0 {
            if TIMER_Done(NPC, b"smoke\0".as_ptr() as *const c_char) != 0 && TIMER_Done(NPC, b"droidsmoketotal\0".as_ptr() as *const c_char) == 0 {
                TIMER_Set( NPC, b"smoke\0".as_ptr() as *const c_char, 100);
                G_PlayEffect( b"volumetric/droid_smoke\0".as_ptr() as *const c_char, &(*NPC).currentOrigin as *const [f32; 3], &dir as *const [f32; 3]);
            }

            if TIMER_Done(NPC, b"droidspark\0".as_ptr() as *const c_char) != 0 {
                TIMER_Set( NPC, b"droidspark\0".as_ptr() as *const c_char, Q_irand(100,500));
                G_PlayEffect( b"sparks/spark\0".as_ptr() as *const c_char, &(*NPC).currentOrigin as *const [f32; 3], &dir as *const [f32; 3]);
            }

            (*addr_of_mut!(ucmd)).forwardmove = Q_irand( -64, 64);

            if TIMER_Done(NPC, b"roam\0".as_ptr() as *const c_char) != 0 {
                TIMER_Set( NPC, b"roam\0".as_ptr() as *const c_char, Q_irand( 250, 1000 ) );
                (*NPCInfo).desiredYaw = Q_irand( 0, 360 ) as f32; // Go in random directions
            }
        } else {
            if TIMER_Done(NPC, b"roam\0".as_ptr() as *const c_char) != 0 {
                (*NPCInfo).localState = LSTATE_NONE;
            } else {
                (*NPCInfo).desiredYaw = AngleNormalize360((*NPCInfo).desiredYaw + 40.0); // Spin around
            }
        }
    } else {
        if TIMER_Done(NPC, b"roam\0".as_ptr() as *const c_char) != 0 {
            (*NPCInfo).localState = LSTATE_NONE;
        } else {
            (*NPCInfo).desiredYaw = AngleNormalize360((*NPCInfo).desiredYaw + 40.0); // Spin around
        }
    }

    NPC_UpdateAngles( 1, 1 );
}

/*
-------------------------
NPC_BSDroid_Pain
-------------------------
*/
pub unsafe fn NPC_Droid_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int, hitLoc: c_int) {
    let mut anim: c_int;
    let mut pain_chance: f32;

    if !(*self_).NPC.is_null() && (*(*self_).NPC).ignorePain != 0 {
        return;
    }
    VectorCopy( &(*(*self_).NPC).lastPathAngles as *const [f32; 3], &mut (*self_).s.angles as *mut [f32; 3] );

    if (*(*self_).client).NPC_class == 4 /* CLASS_R5D2 */ {
        pain_chance = NPC_GetPainChance( self_, damage );

        // Put it in pain
        if mod_ == 15 /* MOD_DEMP2 */ || mod_ == 16 /* MOD_DEMP2_ALT */ || random() < pain_chance {	// Spin around in pain? Demp2 always does this
            // Health is between 0-30 or was hit by a DEMP2 so pop his head
            if (*self_).health < 30 || mod_ == 15 /* MOD_DEMP2 */ || mod_ == 16 /* MOD_DEMP2_ALT */ {
                if ((*self_).spawnflags & 2) == 0 {	// Doesn't have to ALWAYSDIE
                    if ((*(*self_).NPC).localState != LSTATE_SPINNING) &&
                        ((*self_).ghoul2_render_status[(*self_).playerModel as usize] == 0) {
                        // gi.G2API_SetSurfaceOnOff( &self->ghoul2[self->playerModel], "head", TURN_OFF );

                        //						G_PlayEffect( "small_chunks" , self->currentOrigin );
                        G_PlayEffect( b"chunks/r5d2head\0".as_ptr() as *const c_char, &(*self_).currentOrigin as *const [f32; 3], std::ptr::null());

                        (*self_).s.powerups |= ( 1 << 6 /* PW_SHOCKED */ );
                        (*(*self_).client).ps.powerups[6 /* PW_SHOCKED */] = (*addr_of!(level)).time + 3000;

                        TIMER_Set( self_, b"droidsmoketotal\0".as_ptr() as *const c_char, 5000);
                        TIMER_Set( self_, b"droidspark\0".as_ptr() as *const c_char, 100);
                        (*(*self_).NPC).localState = LSTATE_SPINNING;
                    }
                }
            }
            // Just give him normal pain for a little while
            else {
                anim = (*(*self_).client).ps.legsAnim;

                if anim == 27 /* BOTH_STAND2 */ {	// On two legs?
                    anim = 20; /* BOTH_PAIN1 */
                } else {						// On three legs
                    anim = 21; /* BOTH_PAIN2 */
                }

                NPC_SetAnim( self_, 3 /* SETANIM_BOTH */, anim, (1 << 6) | (1 << 7) );

                // Spin around in pain
                (*(*self_).NPC).localState = LSTATE_SPINNING;
                TIMER_Set( self_, b"roam\0".as_ptr() as *const c_char, Q_irand(1000,2000));
            }
        }
    } else if (*(*self_).client).NPC_class == 5 /* CLASS_MOUSE */ {
        if mod_ == 15 /* MOD_DEMP2 */ || mod_ == 16 /* MOD_DEMP2_ALT */ {
            (*(*self_).NPC).localState = LSTATE_SPINNING;
            (*self_).s.powerups |= ( 1 << 6 /* PW_SHOCKED */ );
            (*(*self_).client).ps.powerups[6 /* PW_SHOCKED */] = (*addr_of!(level)).time + 3000;
        } else {
            (*(*self_).NPC).localState = LSTATE_BACKINGUP;
        }

        (*(*self_).NPC).scriptFlags &= !(1 << 10); /* SCF_LOOK_FOR_ENEMIES */
    } else if ((*(*self_).client).NPC_class == 3 /* CLASS_R2D2 */) {

        pain_chance = NPC_GetPainChance( self_, damage );

        if mod_ == 15 /* MOD_DEMP2 */ || mod_ == 16 /* MOD_DEMP2_ALT */ || random() < pain_chance {	// Spin around in pain? Demp2 always does this
            anim = (*(*self_).client).ps.legsAnim;

            if anim == 27 /* BOTH_STAND2 */ {	// On two legs?
                anim = 20; /* BOTH_PAIN1 */
            } else {						// On three legs
                anim = 21; /* BOTH_PAIN2 */
            }

            NPC_SetAnim( self_, 3 /* SETANIM_BOTH */, anim, (1 << 6) | (1 << 7) );

            // Spin around in pain
            (*(*self_).NPC).localState = LSTATE_SPINNING;
            TIMER_Set( self_, b"roam\0".as_ptr() as *const c_char, Q_irand(1000,2000));
        }
    } else if (*(*self_).client).NPC_class == 2 /* CLASS_INTERROGATOR */ && ( mod_ == 15 /* MOD_DEMP2 */ || mod_ == 16 /* MOD_DEMP2_ALT */ ) && !other.is_null() {
        let mut dir: [f32; 3] = [0.0; 3];

        VectorSubtract( &(*self_).currentOrigin as *const [f32; 3], &(*other).currentOrigin as *const [f32; 3], &mut dir as *mut [f32; 3] );
        VectorNormalize( &mut dir as *mut [f32; 3] );

        VectorMA( &(*(*self_).client).ps.velocity as *const [f32; 3], 550.0, &dir as *const [f32; 3], &mut (*(*self_).client).ps.velocity as *mut [f32; 3] );
        (*(*self_).client).ps.velocity[2] -= 127.0;
    }

    NPC_Pain( self_, inflictor, other, point, damage, mod_);
}


/*
-------------------------
Droid_Pain
-------------------------
*/
pub unsafe fn Droid_Pain() {
    if TIMER_Done(NPC, b"droidpain\0".as_ptr() as *const c_char) != 0 {	//He's done jumping around
        (*NPCInfo).localState = LSTATE_NONE;
    }
}

/*
-------------------------
NPC_Mouse_Precache
-------------------------
*/
pub unsafe fn NPC_Mouse_Precache() {
    let mut i: c_int;

    for i in 1..4 {
        G_SoundIndex( va( b"sound/chars/mouse/misc/mousego%d.wav\0".as_ptr() as *const c_char, i ) );
    }

    G_EffectIndex( b"env/small_explode\0".as_ptr() as *const c_char );
    G_SoundIndex( b"sound/chars/mouse/misc/death1\0".as_ptr() as *const c_char );
    G_SoundIndex( b"sound/chars/mouse/misc/mouse_lp\0".as_ptr() as *const c_char );
}

/*
-------------------------
NPC_R5D2_Precache
-------------------------
*/
pub unsafe fn NPC_R5D2_Precache() {
    for i in 1..5 {
        G_SoundIndex( va( b"sound/chars/r5d2/misc/r5talk%d.wav\0".as_ptr() as *const c_char, i ) );
    }
    G_SoundIndex( b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char ); // ??
    G_SoundIndex( b"sound/chars/r2d2/misc/r2_move_lp2.wav\0".as_ptr() as *const c_char );
    G_EffectIndex( b"env/med_explode\0".as_ptr() as *const c_char);
    G_EffectIndex( b"volumetric/droid_smoke\0".as_ptr() as *const c_char );
    G_EffectIndex( b"chunks/r5d2head\0".as_ptr() as *const c_char);
}

/*
-------------------------
NPC_R2D2_Precache
-------------------------
*/
pub unsafe fn NPC_R2D2_Precache() {
    for i in 1..4 {
        G_SoundIndex( va( b"sound/chars/r2d2/misc/r2d2talk0%d.wav\0".as_ptr() as *const c_char, i ) );
    }
    G_SoundIndex( b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char ); // ??
    G_SoundIndex( b"sound/chars/r2d2/misc/r2_move_lp.wav\0".as_ptr() as *const c_char );
    G_EffectIndex( b"env/med_explode\0".as_ptr() as *const c_char);
}

/*
-------------------------
NPC_Gonk_Precache
-------------------------
*/
pub unsafe fn NPC_Gonk_Precache() {
    G_SoundIndex(b"sound/chars/gonk/misc/gonktalk1.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/gonk/misc/gonktalk2.wav\0".as_ptr() as *const c_char);

    G_SoundIndex(b"sound/chars/gonk/misc/death1.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/gonk/misc/death2.wav\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/gonk/misc/death3.wav\0".as_ptr() as *const c_char);

    G_EffectIndex( b"env/med_explode\0".as_ptr() as *const c_char);
}

/*
-------------------------
NPC_Protocol_Precache
-------------------------
*/
pub unsafe fn NPC_Protocol_Precache() {
    G_SoundIndex( b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char );
    G_EffectIndex( b"env/med_explode\0".as_ptr() as *const c_char);
}

/*
static void R5D2_OffsetLook( float offset, vec3_t out )
{
    vec3_t	angles, forward, temp;

    GetAnglesForDirection( NPC->currentOrigin, NPCInfo->investigateGoal, angles );
    angles[YAW] += offset;
    AngleVectors( angles, forward, NULL, NULL );
    VectorMA( NPC->currentOrigin, 64, forward, out );

    CalcEntitySpot( NPC, SPOT_HEAD, temp );
    out[2] = temp[2];
}
*/

/*
-------------------------
R5D2_LookAround
-------------------------
*/
/*
static void R5D2_LookAround( void )
{
    vec3_t	lookPos;
    float	perc = (float) ( level.time - NPCInfo->pauseTime ) / (float) NPCInfo->investigateDebounceTime;

    //Keep looking at the spot
    if ( perc < 0.25 )
    {
        VectorCopy( NPCInfo->investigateGoal, lookPos );
    }
    else if ( perc < 0.5f )		//Look up but straight ahead
    {
        R5D2_OffsetLook( 0.0f, lookPos );
    }
    else if ( perc < 0.75f )	//Look right
    {
        R5D2_OffsetLook( 45.0f, lookPos );
    }
    else	//Look left
    {
        R5D2_OffsetLook( -45.0f, lookPos );
    }

    NPC_FacePosition( lookPos );
}

*/

/*
-------------------------
NPC_BSDroid_Default
-------------------------
*/
pub unsafe fn NPC_BSDroid_Default() {

    if (*NPCInfo).localState == LSTATE_SPINNING {
        Droid_Spin();
    } else if (*NPCInfo).localState == LSTATE_PAIN {
        Droid_Pain();
    } else if (*NPCInfo).localState == LSTATE_DROP {
        NPC_UpdateAngles( 1, 1 );
        (*addr_of_mut!(ucmd)).upmove = (crandom() * 64.0) as c_int;
    } else if ((*NPCInfo).scriptFlags & (1 << 10)) != 0 { /* SCF_LOOK_FOR_ENEMIES */
        Droid_Patrol();
    } else {
        Droid_Run();
    }
}
