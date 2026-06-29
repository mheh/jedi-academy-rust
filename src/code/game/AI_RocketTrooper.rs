// #[allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// extern declarations for functions from g_headers.h and b_local.h
extern "C" {
    fn PM_FlippingAnim(anim: c_int) -> bool;
    fn NPC_BSST_Patrol();

    fn RT_FlyStart(self_: *mut gentity_t);
    fn Q3_TaskIDPending(ent: *mut gentity_t, taskType: c_int) -> bool;

    fn G_SoundIndex(filename: *const c_char) -> c_int;
    fn G_EffectIndex(filename: *const c_char) -> c_int;

    fn NPC_BehaviorSet_Stormtrooper(bState: c_int);

    fn NPC_ClearLOS(ent: *mut gentity_t) -> bool;

    fn VectorClear(v: *mut [f32; 3]);
    fn DistanceSquared(p1: *const [f32; 3], p2: *const [f32; 3]) -> f32;
    fn VectorSubtract(v1: *const [f32; 3], v2: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    fn DotProduct(v1: *const [f32; 3], v2: *const [f32; 3]) -> f32;

    fn TIMER_Done(ent: *mut gentity_t, timer: *const c_char) -> bool;
    fn TIMER_Set(ent: *mut gentity_t, timer: *const c_char, duration: c_int);

    fn NPC_ShotEntity(ent: *mut gentity_t, impactPos: *mut [f32; 3]) -> c_int;

    fn CalcEntitySpot(ent: *mut gentity_t, spot: c_int, point: *mut [f32; 3]);
    fn VectorCompare(v1: *const [f32; 3], v2: *const [f32; 3]) -> bool;

    fn VectorScale(in_: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn VectorMA(v1: *const [f32; 3], scale: f32, v2: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorCopy(in_: *const [f32; 3], out: *mut [f32; 3]);

    fn DistanceHorizontal(p1: *const [f32; 3], p2: *const [f32; 3]) -> f32;
    fn DistanceHorizontalSquared(p1: *const [f32; 3], p2: *const [f32; 3]) -> f32;

    fn vectoangles(value: *const [f32; 3], angles: *mut [f32; 3]);

    fn UpdateGoal() -> bool;

    fn NPC_UpdateAngles(centerFace: bool, turnCapped: bool);
    fn NPC_FaceEnemy(deadly: bool);
    fn NPC_MoveToGoal(allowPathfinding: bool) -> bool;

    fn WeaponThink(force: bool);

    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn Q_flrand(low: f32, high: f32) -> f32;

    fn G_PlayEffect(fxHandle: c_int, modelindex: c_int, bolt: c_int, entnum: c_int, origin: *const [f32; 3], duration: c_int, loop_: bool);
    fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundfile: *const c_char);
    fn G_StopEffect(effect: *const c_char, modelindex: c_int, bolt: c_int, entnum: c_int);

    fn RT_CheckJump();

    // Used as free function calls within the module
    fn rand() -> c_int;
    fn random() -> f32;
    fn crandom() -> f32;
    fn fabs(x: f32) -> f32;

    // Global struct gi (game interface) - accessed for tracing
    static mut gi: game_import_t;
}

// Forward declarations for types we'll reference but not fully define
#[repr(C)]
pub struct gentity_t {
    // Placeholder - real structure defined elsewhere
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct trace_t {
    // Placeholder - real structure defined elsewhere
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct game_import_t {
    // Placeholder - real structure defined elsewhere
    _opaque: [u8; 0],
}

// Globals from the module (must be declared as static mut and accessed via addr_of!)
extern "C" {
    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut c_void; // npcinfodata_t*
    static mut level: c_void; // level_locals_t
    static mut g_gravity: *mut cvar_t;
    static mut g_entities: *mut gentity_t;
    static mut ucmd: usercmd_t;
}

#[repr(C)]
pub struct cvar_t {
    // Placeholder
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct usercmd_t {
    // Placeholder
    _opaque: [u8; 0],
}

// Macros converted to constants
const VELOCITY_DECAY: f32 = 0.7;

const RT_FLYING_STRAFE_VEL: i32 = 60;
const RT_FLYING_STRAFE_DIS: i32 = 200;
const RT_FLYING_UPWARD_PUSH: i32 = 150;

const RT_FLYING_FORWARD_BASE_SPEED: i32 = 50;
const RT_FLYING_FORWARD_MULTIPLIER: i32 = 10;

// Constants that appear in the code
const ENTITYNUM_NONE: c_int = -1;
const MIN_ROCKET_DIST_SQUARED: f32 = 128.0 * 128.0;
const SPOT_HEAD: c_int = 0; // placeholder
const MASK_SHOT: c_int = 0; // placeholder
const MASK_SOLID: c_int = 0; // placeholder
const BUTTON_ATTACK: c_int = 1; // placeholder
const BUTTON_ALT_ATTACK: c_int = 2; // placeholder
const PMF_TIME_KNOCKBACK: c_int = 1; // placeholder
const SVF_CUSTOM_GRAVITY: c_int = 1; // placeholder
const MT_FLYSWIM: c_int = 1; // placeholder
const MT_RUNJUMP: c_int = 0; // placeholder
const NPCAI_FLY: c_int = 1; // placeholder
const TID_MOVE_NAV: c_int = 0; // placeholder
const CHAN_ITEM: c_int = 0; // placeholder
const Q3_INFINITE: c_int = -1; // placeholder
const RANK_LT: c_int = 0; // placeholder
const WP_NONE: c_int = 0; // placeholder
const WP_ROCKET_LAUNCHER: c_int = 5; // placeholder
const WP_CONCUSSION: c_int = 6; // placeholder
const WP_FLECHETTE: c_int = 7; // placeholder
const WP_REPEATER: c_int = 8; // placeholder
const WP_THERMAL: c_int = 9; // placeholder
const WP_TRIP_MINE: c_int = 10; // placeholder
const WP_DET_PACK: c_int = 11; // placeholder
const WP_EMPLACED_GUN: c_int = 12; // placeholder
const SCF_ALT_FIRE: c_int = 1; // placeholder
const SCF_CHASE_ENEMIES: c_int = 2; // placeholder
const SCF_FIRE_WEAPON: c_int = 4; // placeholder
const SVF_GLASS_BRUSH: c_int = 1; // placeholder
const FP_LEVITATION: c_int = 0; // placeholder
const YAW: usize = 1;
const PITCH: usize = 0;

pub fn RT_Precache() {
    unsafe {
        G_SoundIndex(b"sound/chars/boba/bf_blast-off.wav\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/boba/bf_jetpack_lp.wav\0".as_ptr() as *const c_char);
        G_SoundIndex(b"sound/chars/boba/bf_land.wav\0".as_ptr() as *const c_char);
        G_EffectIndex(b"rockettrooper/flameNEW\0".as_ptr() as *const c_char);
        G_EffectIndex(b"rockettrooper/light_cone\0".as_ptr() as *const c_char); //extern this?  At least use a different one
    }
}

pub fn RT_RunStormtrooperAI() {
    unsafe {
        let mut bState: c_int;
        //Execute our bState
        if (*NPCInfo as *mut c_void as usize) != 0 {
            // tempBehavior access via NPCInfo (placeholder pattern)
            // This is a simplified access pattern - actual structure would need proper offset calculation
            bState = 0; // placeholder
        } else {
            // behaviorState and defaultBehavior access via NPCInfo (placeholder)
            bState = 0; // placeholder
        }
        NPC_BehaviorSet_Stormtrooper(bState);
    }
}

pub fn RT_FireDecide() {
    unsafe {
        let mut enemyLOS: bool = false;
        let mut enemyCS: bool = false;
        let mut enemyInFOV: bool = false;
        //let mut move_: bool = true;
        let mut faceEnemy: bool = false;
        let mut shoot: bool = false;
        let mut hitAlly: bool = false;
        let mut impactPos: [f32; 3] = [0.0; 3];
        let mut enemyDist: f32;

        // Access NPC->client->ps.groundEntityNum, etc. via raw pointer
        // Placeholder: simplified structure access
        let npc_ptr = NPC;
        let enemy_ptr = (*npc_ptr).client as *mut c_void; // placeholder

        if 0 == 1 { // placeholder for groundEntityNum == ENTITYNUM_NONE check
            // take off
            RT_FlyStart(NPC);
        }

        if (*NPC).enemy.is_null() {
            return;
        }

        VectorClear(&mut impactPos);
        enemyDist = DistanceSquared(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);

        let mut enemyDir: [f32; 3] = [0.0; 3];
        let mut shootDir: [f32; 3] = [0.0; 3];
        VectorSubtract(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin, &mut enemyDir);
        VectorNormalize(&mut enemyDir);
        AngleVectors(&(*(*NPC).client).ps.viewangles, &mut shootDir, core::ptr::null_mut(), core::ptr::null_mut());
        let dot: f32 = DotProduct(&enemyDir, &shootDir);
        if dot > 0.5 || (enemyDist * (1.0 - dot)) < 10000.0 {
            //enemy is in front of me or they're very close and not behind me
            enemyInFOV = true;
        }

        if enemyDist < MIN_ROCKET_DIST_SQUARED {
            //enemy within 128
            if ((*(*NPC).client).ps.weapon == WP_FLECHETTE || (*(*NPC).client).ps.weapon == WP_REPEATER) &&
                (*(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE != 0) {
                //shooting an explosive, but enemy too close, switch to primary fire
                // placeholder for scriptFlags access
                //NPCInfo->scriptFlags &= ~SCF_ALT_FIRE;
                //FIXME: we can never go back to alt-fire this way since, after this, we don't know if we were initially supposed to use alt-fire or not...
            }
        }

        //can we see our target?
        if TIMER_Done(NPC, b"nextAttackDelay\0".as_ptr() as *const c_char) && TIMER_Done(NPC, b"flameTime\0".as_ptr() as *const c_char) {
            if NPC_ClearLOS((*NPC).enemy) {
                // NPCInfo->enemyLastSeenTime = level.time;
                enemyLOS = true;

                if (*(*NPC).client).ps.weapon == WP_NONE {
                    enemyCS = false; //not true, but should stop us from firing
                } else {
                    //can we shoot our target?
                    if ((*(*NPC).client).ps.weapon == WP_ROCKET_LAUNCHER
                        || ((*(*NPC).client).ps.weapon == WP_CONCUSSION && !(*(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE) != 0)
                        || ((*(*NPC).client).ps.weapon == WP_FLECHETTE && (*(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE) != 0)) && enemyDist < MIN_ROCKET_DIST_SQUARED {
                        //128*128
                        enemyCS = false; //not true, but should stop us from firing
                        hitAlly = true; //us!
                        //FIXME: if too close, run away!
                    } else if enemyInFOV {
                        //if enemy is FOV, go ahead and check for shooting
                        let hit: c_int = NPC_ShotEntity((*NPC).enemy, &mut impactPos);
                        let hitEnt: *mut gentity_t = &mut *g_entities.offset(hit as isize);

                        if hit == (*(*NPC).enemy).s.number {
                            //can hit enemy or enemy ally or will hit glass or other minor breakable (or in emplaced gun), so shoot anyway
                            enemyCS = true;
                            //NPC_AimAdjust( 2 );//adjust aim better longer we have clear shot at enemy
                            VectorCopy(&(*(*NPC).enemy).currentOrigin, &mut (*NPCInfo as *mut c_void as *mut [f32; 3]);
                        } else if !hitEnt.is_null() {
                            // Continue with hitEnt checks...
                            //Hmm, have to get around this bastard
                            //NPC_AimAdjust( 1 );//adjust aim better longer we can see enemy
                            if !hitEnt.is_null() {
                                //would hit an ally, don't fire!!!
                                hitAlly = true;
                            } else {
                                //Check and see where our shot *would* hit... if it's not close to the enemy (within 256?), then don't fire
                            }
                        } else {
                            enemyCS = false; //not true, but should stop us from firing
                        }
                    } else {
                        enemyCS = false; //not true, but should stop us from firing
                    }
                }
            } else if 0 == 1 { // placeholder for gi.inPVS check
                // NPCInfo->enemyLastSeenTime = level.time;
                faceEnemy = true;
                //NPC_AimAdjust( -1 );//adjust aim worse longer we cannot see enemy
            }

            if (*(*NPC).client).ps.weapon == WP_NONE {
                faceEnemy = false;
                shoot = false;
            } else {
                if enemyLOS {
                    //FIXME: no need to face enemy if we're moving to some other goal and he's too far away to shoot?
                    faceEnemy = true;
                }
                if enemyCS {
                    shoot = true;
                }
            }

            if !enemyCS {
                //if have a clear shot, always try
                //See if we should continue to fire on their last position
                //!TIMER_Done( NPC, "stick" ) ||
                if !hitAlly { //we're not going to hit an ally
                    // && enemyInFOV //enemy is in our FOV //FIXME: or we don't have a clear LOS?
                    // && NPCInfo->enemyLastSeenTime > 0 //we've seen the enemy
                    if 0 == 1 { // placeholder for time check
                        if Q_irand(0, 10) == 0 {
                            //Fire on the last known position
                            let mut muzzle: [f32; 3] = [0.0; 3];
                            let mut dir: [f32; 3] = [0.0; 3];
                            let mut angles: [f32; 3] = [0.0; 3];
                            let mut tooClose: bool = false;
                            let mut tooFar: bool = false;

                            CalcEntitySpot(NPC, SPOT_HEAD, &mut muzzle);
                            if VectorCompare(&impactPos, &[0.0, 0.0, 0.0]) {
                                //never checked ShotEntity this frame, so must do a trace...
                                let mut tr: trace_t = core::mem::zeroed();
                                //let mut mins: [f32; 3] = [-2.0, -2.0, -2.0];
                                //let mut maxs: [f32; 3] = [2.0, 2.0, 2.0];
                                let mut forward: [f32; 3] = [0.0; 3];
                                let mut end: [f32; 3] = [0.0; 3];
                                AngleVectors(&(*(*NPC).client).ps.viewangles, &mut forward, core::ptr::null_mut(), core::ptr::null_mut());
                                VectorMA(&muzzle, 8192.0, &forward, &mut end);
                                gi.trace(&mut tr, &muzzle, &[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0], &end, (*NPC).s.number, MASK_SHOT);
                                VectorCopy(&tr.endpos as *const [f32; 3], &mut impactPos);
                            }

                            //see if impact would be too close to me
                            let mut distThreshold: f32 = 16384.0; /*128*128*/ //default
                            match (*NPC).s.weapon {
                                WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE | WP_DET_PACK => {
                                    distThreshold = 65536.0; /*256*256*/
                                }
                                WP_REPEATER => {
                                    if *(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE != 0 {
                                        distThreshold = 65536.0; /*256*256*/
                                    }
                                }
                                WP_CONCUSSION => {
                                    if !(*(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE) != 0 {
                                        distThreshold = 65536.0; /*256*256*/
                                    }
                                }
                                _ => {}
                            }

                            let dist: f32 = DistanceSquared(&impactPos, &muzzle);

                            if dist < distThreshold {
                                //impact would be too close to me
                                tooClose = true;
                            } else if 0 == 1 { // placeholder for time check
                                //we've haven't seen them in the last 5 seconds
                                //see if it's too far from where he is
                                distThreshold = 65536.0; /*256*256*/ //default
                                match (*NPC).s.weapon {
                                    WP_ROCKET_LAUNCHER | WP_FLECHETTE | WP_THERMAL | WP_TRIP_MINE | WP_DET_PACK => {
                                        distThreshold = 262144.0; /*512*512*/
                                    }
                                    WP_REPEATER => {
                                        if *(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE != 0 {
                                            distThreshold = 262144.0; /*512*512*/
                                        }
                                    }
                                    WP_CONCUSSION => {
                                        if !(*(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE) != 0 {
                                            distThreshold = 262144.0; /*512*512*/
                                        }
                                    }
                                    _ => {}
                                }
                                let dist_last: f32 = DistanceSquared(&impactPos, &*NPCInfo as *const _ as *const [f32; 3]);
                                if dist_last > distThreshold {
                                    //impact would be too far from enemy
                                    tooFar = true;
                                }
                            }

                            if !tooClose && !tooFar {
                                //okay too shoot at last pos
                                VectorSubtract(&*NPCInfo as *const _ as *const [f32; 3], &muzzle, &mut dir);
                                VectorNormalize(&mut dir);
                                vectoangles(&dir, &mut angles);

                                // NPCInfo->desiredYaw = angles[YAW];
                                // NPCInfo->desiredPitch = angles[PITCH];

                                shoot = true;
                                faceEnemy = false;
                            }
                        }
                    }
                }
            }

            //FIXME: don't shoot right away!
            if (*(*NPC).client).fireDelay != 0 {
                if (*NPC).s.weapon == WP_ROCKET_LAUNCHER
                    || ((*NPC).s.weapon == WP_CONCUSSION && !(*(NPCInfo as *mut c_void as *mut i32) & SCF_ALT_FIRE) != 0) {
                    if !enemyLOS || !enemyCS {
                        //cancel it
                        (*(*NPC).client).fireDelay = 0;
                    } else {
                        //delay our next attempt
                        TIMER_Set(NPC, b"nextAttackDelay\0".as_ptr() as *const c_char, Q_irand(1000, 3000)); //FIXME: base on g_spskill
                    }
                }
            } else if shoot {
                //try to shoot if it's time
                if TIMER_Done(NPC, b"nextAttackDelay\0".as_ptr() as *const c_char) {
                    if !(*(NPCInfo as *mut c_void as *mut i32) & SCF_FIRE_WEAPON != 0) { // we've already fired, no need to do it again here
                        WeaponThink(true);
                    }
                    //NASTY
                    let mut altChance: c_int = 6; //FIXME: base on g_spskill
                    if (*NPC).s.weapon == WP_ROCKET_LAUNCHER {
                        if (ucmd.buttons & BUTTON_ATTACK != 0)
                            && Q_irand(0, altChance) == 0 {
                            //every now and then, shoot a homing rocket
                            ucmd.buttons &= !BUTTON_ATTACK;
                            ucmd.buttons |= BUTTON_ALT_ATTACK;
                            (*(*NPC).client).fireDelay = Q_irand(1000, 3000); //FIXME: base on g_spskill
                        }
                    } else if (*NPC).s.weapon == WP_CONCUSSION {
                        if (ucmd.buttons & BUTTON_ATTACK != 0)
                            && Q_irand(0, altChance * 5) != 0 {
                            //fire the beam shot
                            ucmd.buttons &= !BUTTON_ATTACK;
                            ucmd.buttons |= BUTTON_ALT_ATTACK;
                            TIMER_Set(NPC, b"nextAttackDelay\0".as_ptr() as *const c_char, Q_irand(1500, 2500)); //FIXME: base on g_spskill
                        } else {
                            //fire the rocket-like shot
                            TIMER_Set(NPC, b"nextAttackDelay\0".as_ptr() as *const c_char, Q_irand(3000, 5000)); //FIXME: base on g_spskill
                        }
                    }
                }
            }
        }
    }
}

//=====================================================================================
//FLYING behavior
//=====================================================================================
pub fn RT_Flying(self_: *mut gentity_t) -> bool {
    unsafe {
        ((*(*self_).client).moveType == MT_FLYSWIM)
    }
}

pub fn RT_FlyStart(self_: *mut gentity_t) {
    //switch to seeker AI for a while
    unsafe {
        if TIMER_Done(self_, b"jetRecharge\0".as_ptr() as *const c_char)
            && !RT_Flying(self_) {
            (*(*self_).client).ps.gravity = 0;
            (*self_).svFlags |= SVF_CUSTOM_GRAVITY;
            (*(*self_).client).moveType = MT_FLYSWIM;
            //Inform NPC_HandleAIFlags we want to fly
            (*(*self_).NPC).aiFlags |= NPCAI_FLY;
            (*self_).lastInAirTime = 0; // level.time - placeholder

            //start jet effect
            (*(*self_).client).jetPackTime = Q3_INFINITE;
            if (*self_).genericBolt1 != -1 {
                G_PlayEffect(G_EffectIndex(b"rockettrooper/flameNEW\0".as_ptr() as *const c_char), (*self_).playerModel, (*self_).genericBolt1, (*self_).s.number, &(*self_).currentOrigin, Q3_INFINITE, true);
            }
            if (*self_).genericBolt2 != -1 {
                G_PlayEffect(G_EffectIndex(b"rockettrooper/flameNEW\0".as_ptr() as *const c_char), (*self_).playerModel, (*self_).genericBolt2, (*self_).s.number, &(*self_).currentOrigin, Q3_INFINITE, true);
            }

            //take-off sound
            G_SoundOnEnt(self_, CHAN_ITEM, b"sound/chars/boba/bf_blast-off.wav\0".as_ptr() as *const c_char);
            //jet loop sound
            (*self_).s.loopSound = G_SoundIndex(b"sound/chars/boba/bf_jetpack_lp.wav\0".as_ptr() as *const c_char);
            if !(*self_).NPC.is_null() {
                (*self_).count = Q3_INFINITE; // SEEKER shot ammo count
            }
        }
    }
}

pub fn RT_FlyStop(self_: *mut gentity_t) {
    unsafe {
        (*(*self_).client).ps.gravity = 0; // g_gravity->value - placeholder
        (*self_).svFlags &= !SVF_CUSTOM_GRAVITY;
        (*(*self_).client).moveType = MT_RUNJUMP;
        //Stop the effect
        (*(*self_).client).jetPackTime = 0;
        if (*self_).genericBolt1 != -1 {
            G_StopEffect(b"rockettrooper/flameNEW\0".as_ptr() as *const c_char, (*self_).playerModel, (*self_).genericBolt1, (*self_).s.number);
        }
        if (*self_).genericBolt2 != -1 {
            G_StopEffect(b"rockettrooper/flameNEW\0".as_ptr() as *const c_char, (*self_).playerModel, (*self_).genericBolt2, (*self_).s.number);
        }
        //stop jet loop sound
        (*self_).s.loopSound = 0;
        G_SoundOnEnt(self_, CHAN_ITEM, b"sound/chars/boba/bf_land.wav\0".as_ptr() as *const c_char);

        if !(*self_).NPC.is_null() {
            (*self_).count = 0; // SEEKER shot ammo count
            TIMER_Set(self_, b"jetRecharge\0".as_ptr() as *const c_char, Q_irand(1000, 5000));
            TIMER_Set(self_, b"jumpChaseDebounce\0".as_ptr() as *const c_char, Q_irand(500, 2000));
        }
    }
}

pub fn RT_JetPackEffect(duration: c_int) {
    unsafe {
        if (*NPC).genericBolt1 != -1 {
            G_PlayEffect(G_EffectIndex(b"rockettrooper/flameNEW\0".as_ptr() as *const c_char), (*NPC).playerModel, (*NPC).genericBolt1, (*NPC).s.number, &(*NPC).currentOrigin, duration, true);
        }
        if (*NPC).genericBolt2 != -1 {
            G_PlayEffect(G_EffectIndex(b"rockettrooper/flameNEW\0".as_ptr() as *const c_char), (*NPC).playerModel, (*NPC).genericBolt2, (*NPC).s.number, &(*NPC).currentOrigin, duration, true);
        }

        //take-off sound
        G_SoundOnEnt(NPC, CHAN_ITEM, b"sound/chars/boba/bf_blast-off.wav\0".as_ptr() as *const c_char);
    }
}

pub fn RT_Flying_ApplyFriction(frictionScale: f32) {
    unsafe {
        if (*(*NPC).client).ps.velocity[0] != 0.0 {
            (*(*NPC).client).ps.velocity[0] *= VELOCITY_DECAY; ///frictionScale;

            if fabs((*(*NPC).client).ps.velocity[0]) < 1.0 {
                (*(*NPC).client).ps.velocity[0] = 0.0;
            }
        }

        if (*(*NPC).client).ps.velocity[1] != 0.0 {
            (*(*NPC).client).ps.velocity[1] *= VELOCITY_DECAY; ///frictionScale;

            if fabs((*(*NPC).client).ps.velocity[1]) < 1.0 {
                (*(*NPC).client).ps.velocity[1] = 0.0;
            }
        }
    }
}

pub fn RT_Flying_MaintainHeight() {
    unsafe {
        let mut dif: f32 = 0.0;

        // Update our angles regardless
        NPC_UpdateAngles(true, true);

        if (*NPC).forcePushTime > 0 { // level.time - placeholder
            //if being pushed, we don't have control over our movement
            return;
        }

        if ((*(*NPC).client).ps.pm_flags & PMF_TIME_KNOCKBACK) != 0 {
            //don't slow down for a bit
            if (*(*NPC).client).ps.pm_time > 0 {
                VectorScale(&(*(*NPC).client).ps.velocity, 0.9, &mut (*(*NPC).client).ps.velocity);
                return;
            }
        }

        /*
        if ( (NPC->client->ps.eFlags&EF_FORCE_GRIPPED) )
        {
            RT_Flying_ApplyFriction( 3.0f );
            return;
        }
        */
        // If we have an enemy, we should try to hover at or a little below enemy eye level
        if !(*NPC).enemy.is_null()
            && (!Q3_TaskIDPending(NPC, TID_MOVE_NAV) || !(*NPCInfo as *mut c_void as *mut c_void).is_null()) {
            if TIMER_Done(NPC, b"heightChange\0".as_ptr() as *const c_char) {
                TIMER_Set(NPC, b"heightChange\0".as_ptr() as *const c_char, Q_irand(1000, 3000));

                let mut enemyZHeight: f32 = (*(*NPC).enemy).currentOrigin[2];
                if !(*(*NPC).enemy).client.is_null()
                    && (*(*(*NPC).enemy).client).ps.groundEntityNum == ENTITYNUM_NONE
                    && ((*(*(*NPC).enemy).client).ps.forcePowersActive & (1 << FP_LEVITATION)) != 0 {
                    //so we don't go up when they force jump up at us
                    enemyZHeight = (*(*(*NPC).enemy).client).ps.forceJumpZStart;
                }

                // Find the height difference
                dif = (enemyZHeight + Q_flrand((*(*NPC).enemy).maxs[2] / 2.0, (*(*NPC).enemy).maxs[2] + 8.0)) - (*NPC).currentOrigin[2];

                let difFactor: f32 = 10.0;

                // cap to prevent dramatic height shifts
                if fabs(dif) > 2.0 * difFactor {
                    if fabs(dif) > 20.0 * difFactor {
                        dif = if dif < 0.0 { -20.0 * difFactor } else { 20.0 * difFactor };
                    }

                    (*(*NPC).client).ps.velocity[2] = ((*(*NPC).client).ps.velocity[2] + dif) / 2.0;
                }
                (*(*NPC).client).ps.velocity[2] *= Q_flrand(0.85, 1.25);
            } else {
                //don't get too far away from height of enemy...
                let mut enemyZHeight: f32 = (*(*NPC).enemy).currentOrigin[2];
                if !(*(*NPC).enemy).client.is_null()
                    && (*(*(*NPC).enemy).client).ps.groundEntityNum == ENTITYNUM_NONE
                    && ((*(*(*NPC).enemy).client).ps.forcePowersActive & (1 << FP_LEVITATION)) != 0 {
                    //so we don't go up when they force jump up at us
                    enemyZHeight = (*(*(*NPC).enemy).client).ps.forceJumpZStart;
                }
                dif = (*NPC).currentOrigin[2] - (enemyZHeight + 64.0);
                let mut maxHeight: f32 = 200.0;
                let hDist: f32 = DistanceHorizontal(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin);
                if hDist < 512.0 {
                    maxHeight *= hDist / 512.0;
                }
                if dif > maxHeight {
                    if (*(*NPC).client).ps.velocity[2] > 0.0 { //FIXME: or: we can't see him anymore
                        //slow down
                        if (*(*NPC).client).ps.velocity[2] != 0.0 {
                            (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                            if fabs((*(*NPC).client).ps.velocity[2]) < 2.0 {
                                (*(*NPC).client).ps.velocity[2] = 0.0;
                            }
                        }
                    } else {
                        //start coming back down
                        (*(*NPC).client).ps.velocity[2] -= 4.0;
                    }
                } else if dif < -200.0 && (*(*NPC).client).ps.velocity[2] < 0.0 { //we're way below him
                    if (*(*NPC).client).ps.velocity[2] < 0.0 { //FIXME: or: we can't see him anymore
                        //slow down
                        if (*(*NPC).client).ps.velocity[2] != 0.0 {
                            (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                            if fabs((*(*NPC).client).ps.velocity[2]) > -2.0 {
                                (*(*NPC).client).ps.velocity[2] = 0.0;
                            }
                        }
                    } else {
                        //start going back up
                        (*(*NPC).client).ps.velocity[2] += 4.0;
                    }
                }
            }
        } else {
            let mut goal: *mut gentity_t = core::ptr::null_mut();

            if !(*NPCInfo as *mut c_void as *mut c_void).is_null() { // Is there a goal?
                goal = (*NPCInfo as *mut c_void as *mut c_void) as *mut gentity_t;
            } else {
                goal = (*NPCInfo as *mut c_void as *mut c_void) as *mut gentity_t;
            }
            if !goal.is_null() {
                dif = (*goal).currentOrigin[2] - (*NPC).currentOrigin[2];
            } else if VectorCompare(&(*NPC).pos1, &[0.0, 0.0, 0.0]) {
                //have a starting position as a reference point
                dif = (*NPC).pos1[2] - (*NPC).currentOrigin[2];
            }

            if fabs(dif) > 24.0 {
                ucmd.upmove = if ucmd.upmove < 0 { -4 } else { 4 };
            } else {
                if (*(*NPC).client).ps.velocity[2] != 0.0 {
                    (*(*NPC).client).ps.velocity[2] *= VELOCITY_DECAY;

                    if fabs((*(*NPC).client).ps.velocity[2]) < 2.0 {
                        (*(*NPC).client).ps.velocity[2] = 0.0;
                    }
                }
            }
        }

        // Apply friction
        RT_Flying_ApplyFriction(1.0);
    }
}

pub fn RT_Flying_Strafe() {
    unsafe {
        let mut side: i32;
        let mut end: [f32; 3] = [0.0; 3];
        let mut right: [f32; 3] = [0.0; 3];
        let mut dir: [f32; 3] = [0.0; 3];
        let mut tr: trace_t = core::mem::zeroed();

        if random() > 0.7
            || (*NPC).enemy.is_null()
            || (*(*NPC).enemy).client.is_null() {
            // Do a regular style strafe
            AngleVectors(&(*(*NPC).client).renderInfo.eyeAngles, core::ptr::null_mut(), &mut right, core::ptr::null_mut());

            // Pick a random strafe direction, then check to see if doing a strafe would be
            //	reasonably valid
            side = if (rand() & 1) != 0 { -1 } else { 1 };
            VectorMA(&(*NPC).currentOrigin, (RT_FLYING_STRAFE_DIS * side) as f32, &right, &mut end);

            gi.trace(&mut tr, &(*NPC).currentOrigin, core::ptr::null(), core::ptr::null(), &end, (*NPC).s.number, MASK_SOLID);

            // Close enough
            if tr.fraction > 0.9 {
                let vel: f32 = (RT_FLYING_STRAFE_VEL as f32) + Q_flrand(-20.0, 20.0);
                VectorMA(&(*(*NPC).client).ps.velocity, vel * (side as f32), &right, &mut (*(*NPC).client).ps.velocity);
                if Q_irand(0, 3) == 0 {
                    // Add a slight upward push
                    let upPush: f32 = RT_FLYING_UPWARD_PUSH as f32;
                    if (*(*NPC).client).ps.velocity[2] < 300.0 {
                        if (*(*NPC).client).ps.velocity[2] < 300.0 + upPush {
                            (*(*NPC).client).ps.velocity[2] += upPush;
                        } else {
                            (*(*NPC).client).ps.velocity[2] = 300.0;
                        }
                    }
                }

                // NPCInfo->standTime = level.time + 1000 + random() * 500;
            }
        } else {
            // Do a strafe to try and keep on the side of their enemy
            AngleVectors(&(*(*(*NPC).enemy).client).renderInfo.eyeAngles, &mut dir, &mut right, core::ptr::null_mut());

            // Pick a random side
            side = if (rand() & 1) != 0 { -1 } else { 1 };
            let stDis: f32 = (RT_FLYING_STRAFE_DIS as f32) * 2.0;
            VectorMA(&(*(*NPC).enemy).currentOrigin, stDis * (side as f32), &right, &mut end);

            // then add a very small bit of random in front of/behind the player action
            VectorMA(&end, crandom() * 25.0, &dir, &mut end);

            gi.trace(&mut tr, &(*NPC).currentOrigin, core::ptr::null(), core::ptr::null(), &end, (*NPC).s.number, MASK_SOLID);

            // Close enough
            if tr.fraction > 0.9 {
                let vel: f32 = ((RT_FLYING_STRAFE_VEL as f32) * 4.0) + Q_flrand(-20.0, 20.0);
                VectorSubtract(&tr.endpos as *const [f32; 3], &(*NPC).currentOrigin, &mut dir);
                dir[2] *= 0.25; // do less upward change
                let dis: f32 = VectorNormalize(&mut dir);
                let mut final_dis = dis;
                if dis > vel {
                    final_dis = vel;
                }
                // Try to move the desired enemy side
                VectorMA(&(*(*NPC).client).ps.velocity, final_dis, &dir, &mut (*(*NPC).client).ps.velocity);

                if Q_irand(0, 3) == 0 {
                    let upPush: f32 = RT_FLYING_UPWARD_PUSH as f32;
                    // Add a slight upward push
                    if (*(*NPC).client).ps.velocity[2] < 300.0 {
                        if (*(*NPC).client).ps.velocity[2] < 300.0 + upPush {
                            (*(*NPC).client).ps.velocity[2] += upPush;
                        } else {
                            (*(*NPC).client).ps.velocity[2] = 300.0;
                        }
                    } else if (*(*NPC).client).ps.velocity[2] > 300.0 {
                        (*(*NPC).client).ps.velocity[2] = 300.0;
                    }
                }

                // NPCInfo->standTime = level.time + 2500 + random() * 500;
            }
        }
    }
}

pub fn RT_Flying_Hunt(visible: bool, advance: bool) {
    unsafe {
        let mut distance: f32;
        let mut speed: f32;
        let mut forward: [f32; 3] = [0.0; 3];

        if (*NPC).forcePushTime >= 0 { // level.time - placeholder
            //if being pushed, we don't have control over our movement
            (*NPC).delay = 0;
            return;
        }
        NPC_FaceEnemy(true);

        // If we're not supposed to stand still, pursue the player
        if (*NPCInfo as *mut c_void as *mut i32).is_null() { // placeholder for NPCInfo->standTime < level.time
            // Only strafe when we can see the player
            if visible {
                (*NPC).delay = 0;
                RT_Flying_Strafe();
                return;
            }
        }

        // If we don't want to advance, stop here
        if advance {
            // Only try and navigate if the player is visible
            if !visible {
                // Move towards our goal
                // NPCInfo->goalEntity = NPC->enemy;
                // NPCInfo->goalRadius = 24;

                (*NPC).delay = 0;
                NPC_MoveToGoal(true);
                return;
            }
        }
        //else move straight at/away from him
        VectorSubtract(&(*(*NPC).enemy).currentOrigin, &(*NPC).currentOrigin, &mut forward);
        forward[2] *= 0.1;
        distance = VectorNormalize(&mut forward);

        speed = (RT_FLYING_FORWARD_BASE_SPEED as f32) + (RT_FLYING_FORWARD_MULTIPLIER as f32) * 0.0; // g_spskill->integer - placeholder
        if advance && distance < Q_flrand(256.0, 3096.0) {
            (*NPC).delay = 0;
            VectorMA(&(*(*NPC).client).ps.velocity, speed, &forward, &mut (*(*NPC).client).ps.velocity);
        } else if distance < Q_flrand(0.0, 128.0) {
            if (*NPC).health <= 50 {
                //always back off
                (*NPC).delay = 0;
            } else if !TIMER_Done(NPC, b"backoffTime\0".as_ptr() as *const c_char) {
                //still backing off from end of last delay
                (*NPC).delay = 0;
            } else if (*NPC).delay == 0 {
                //start a new delay
                (*NPC).delay = Q_irand(0, 10 + (20 * (2 - 0))); // placeholder for g_spskill->integer
            } else {
                //continue the current delay
                (*NPC).delay -= 1;
            }
            if (*NPC).delay == 0 {
                //delay done, now back off for a few seconds!
                TIMER_Set(NPC, b"backoffTime\0".as_ptr() as *const c_char, Q_irand(2000, 5000));
                VectorMA(&(*(*NPC).client).ps.velocity, speed * -2.0, &forward, &mut (*(*NPC).client).ps.velocity);
            }
        } else {
            (*NPC).delay = 0;
        }
    }
}

pub fn RT_Flying_Ranged(visible: bool, advance: bool) {
    unsafe {
        if *(NPCInfo as *mut c_void as *mut i32) & SCF_CHASE_ENEMIES != 0 {
            RT_Flying_Hunt(visible, advance);
        }
    }
}

pub fn RT_Flying_Attack() {
    // Always keep a good height off the ground
    RT_Flying_MaintainHeight();

    unsafe {
        // Rate our distance to the target, and our visibilty
        let distance: f32 = DistanceHorizontalSquared(&(*NPC).currentOrigin, &(*(*NPC).enemy).currentOrigin);
        let visible: bool = NPC_ClearLOS((*NPC).enemy);
        let advance: bool = (distance > (256.0 * 256.0));

        // If we cannot see our target, move to see it
        if !visible {
            if *(NPCInfo as *mut c_void as *mut i32) & SCF_CHASE_ENEMIES != 0 {
                RT_Flying_Hunt(visible, advance);
                return;
            }
        }

        RT_Flying_Ranged(visible, advance);
    }
}

pub fn RT_Flying_Think() {
    unsafe {
        if Q3_TaskIDPending(NPC, TID_MOVE_NAV)
            && UpdateGoal() {
            //being scripted to go to a certain spot, don't maintain height
            if NPC_MoveToGoal(true) {
                //we could macro-nav to our goal
                if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health != 0 && (*(*NPC).enemy).inuse {
                    NPC_FaceEnemy(true);
                    RT_FireDecide();
                }
            } else {
                //frick, no where to nav to, keep us in the air!
                RT_Flying_MaintainHeight();
            }
            return;
        }

        if (*NPC).random == 0.0 {
            // used to offset seekers around a circle so they don't occupy the same spot.  This is not a fool-proof method.
            (*NPC).random = random() * 6.3; // roughly 2pi
        }

        if !(*NPC).enemy.is_null() && (*(*NPC).enemy).health != 0 && (*(*NPC).enemy).inuse {
            RT_Flying_Attack();
            RT_FireDecide();
            return;
        } else {
            RT_Flying_MaintainHeight();
            RT_RunStormtrooperAI();
            return;
        }
    }
}


//=====================================================================================
//ON GROUND WITH ENEMY behavior
//=====================================================================================


//=====================================================================================
//DEFAULT behavior
//=====================================================================================
pub fn NPC_BSRT_Default() {
    unsafe {
        //FIXME: custom pain and death funcs:
            //pain3 is in air
            //die in air is both_falldeath1
            //attack1 is on ground, attack2 is in air

        //FIXME: this doesn't belong here
        if (*(*NPC).client).ps.groundEntityNum != ENTITYNUM_NONE {
            if (*NPCInfo as *mut c_void as *mut i32).is_null() { // placeholder for rank check
                //officers always stay in the air
                (*(*NPC).client).ps.velocity[2] = Q_irand(50, 125) as f32;
                (*(*NPC).NPC).aiFlags |= NPCAI_FLY; //fixme also, Inform NPC_HandleAIFlags we want to fly
            }
        }

        if RT_Flying(NPC) {
            //FIXME: only officers need do this, right?
            RT_Flying_Think();
        } else if !(*NPC).enemy.is_null() {
            //rocketrooper on ground with enemy
            UpdateGoal();
            RT_RunStormtrooperAI();
            RT_CheckJump();
            //NPC_BSST_Default();//FIXME: add missile avoidance
            //RT_Hunt();//NPC_BehaviorSet_Jedi( bState );
        } else {
            //shouldn't have gotten in here
            RT_RunStormtrooperAI();
            //NPC_BSST_Patrol();
        }
    }
}
