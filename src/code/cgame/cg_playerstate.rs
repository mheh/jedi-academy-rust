// cg_playerstate.c -- this file acts on changes in a new playerState_t
// With normal play, this will be done after local prediction, but when
// following another player or playing back a demo, it will be checked
// when the snapshot transitions like all the other entities

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"
// #include "cg_media.h"

use core::ffi::c_int;
use core::ptr::{addr_of, addr_of_mut};

// Forward declarations and external references needed for this module
// These would normally come from headers included in the C version

extern "C" {
    // External function declarations for vector operations
    pub fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    pub fn VectorSubtract(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    pub fn DotProduct(x: *const [f32; 3], y: *const [f32; 3]) -> f32;
    pub fn VectorLength(v: *const [f32; 3]) -> f32;

    // External entity and event functions
    pub fn CG_EntityEvent(cent: *mut centity_t, lerpOrigin: *const [f32; 3]);

    // External sound and force feedback functions
    pub fn cgi_S_StartLocalSound(sfxHandle: c_int, channelNum: c_int);
    pub fn cgi_FF_Start(sfxHandle: c_int, entNum: c_int);
    pub fn cgi_FF_Register(name: *const u8, channel: c_int) -> c_int;
    pub fn cgi_FF_Xbox_Damage(damage: c_int, x: f32);

    // Global game state references
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;
    pub static mut cg_entities: [centity_t; 512];  // MAX_GENTITIES typically 512
    pub static mut weaponData: [weaponData_t; 16]; // WP_NUM_WEAPONS typically
}

// Stub function declaration for SetWeaponSelectTime which is likely defined in another module
extern "C" {
    pub fn SetWeaponSelectTime();
}

// Placeholder type declarations for structures not fully defined in this file
// These are declared minimally to allow compilation; full definitions would be in headers

#[repr(C)]
pub struct cg_t {
    // Minimal placeholder - actual structure would be much larger
    pub snap: *mut snapshot_t,
    pub weaponSelect: c_int,
    pub lowAmmoWarning: c_int,
    pub damageX: f32,
    pub damageY: f32,
    pub v_dmg_roll: f32,
    pub v_dmg_pitch: f32,
    pub v_dmg_time: c_int,
    pub damageValue: f32,
    pub damageTime: c_int,
    pub time: c_int,
    pub thisFrameTeleport: u32,
    pub nextFrameTeleport: u32,
    pub duckChange: c_int,
    pub duckTime: c_int,
    pub refdef: refdef_t,
}

#[repr(C)]
pub struct cgs_t {
    // Minimal placeholder
    pub media: media_t,
    pub timelimit: c_int,
    pub timelimitWarnings: c_int,
}

#[repr(C)]
pub struct snapshot_t {
    pub ps: playerState_t,
    pub serverTime: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub commandTime: c_int,
    pub pm_type: u8,
    pub bobCycle: u8,
    pub pm_flags: u16,
    pub pm_time: c_int,
    pub origin: [f32; 3],
    pub velocity: [f32; 3],
    pub weaponTime: c_int,
    pub gravity: c_int,
    pub speed: c_int,
    pub delta_angles: [i16; 3],
    pub ground_entity_num: u32,
    pub legsTimer: c_int,
    pub legsAnim: c_int,
    pub torsoTimer: c_int,
    pub torsoAnim: c_int,
    pub movementDir: u8,
    pub eFlags: u32,
    pub eventSequence: c_int,
    pub events: [u32; 4], // MAX_PS_EVENTS = 4
    pub eventParms: [u32; 4], // MAX_PS_EVENTS = 4
    pub externalEvent: c_int,
    pub externalEventParm: u8,
    pub clientNum: u8,
    pub weapon: u8,
    pub weaponstate: u8,
    pub viewangles: [f32; 3],
    pub viewheight: c_int,
    pub damageEvent: u8,
    pub damageYaw: u8,
    pub damagePitch: u8,
    pub damageCount: u8,
    pub stats: [c_int; 16],
    pub persistant: [c_int; 16],
    pub misc: [c_int; 16],
    pub ammo: [c_int; 16],
}

#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub lerpOrigin: [f32; 3],
}

#[repr(C)]
pub struct entityState_t {
    pub event: c_int,
    pub eventParm: u8,
}

#[repr(C)]
pub struct refdef_t {
    pub viewaxis: [[f32; 3]; 3],
}

#[repr(C)]
pub struct media_t {
    pub noAmmoSound: c_int,
    pub noAmmoForce: c_int,
}

#[repr(C)]
pub struct weaponData_t {
    pub ammoIndex: c_int,
    pub ammoLow: c_int,
}

// Constants (typically from headers)
const MAX_PS_EVENTS: c_int = 4;
const WP_NONE: c_int = 0;
const WP_SABER: c_int = 1;
const WP_NUM_WEAPONS: c_int = 15;
const STAT_HEALTH: usize = 0;
const STAT_WEAPONS: usize = 1;
const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;
const EF_TELEPORT_BIT: u32 = 0x0001;
const PERS_SPAWN_COUNT: usize = 0;
const PERS_HITS: usize = 2;
const PERS_SCORE: usize = 1;
const PERS_REWARD_COUNT: usize = 3;
const PERS_REWARD: usize = 4;
const DAMAGE_TIME: c_int = 500;
const CHAN_LOCAL_SOUND: c_int = 1;
const FF_CLIENT_LOCAL: c_int = 0;
const FF_CHANNEL_DAMAGE: c_int = 1;

const REWARD_IMPRESSIVE: c_int = 1;
const REWARD_EXCELLENT: c_int = 2;
const REWARD_DENIED: c_int = 3;

// Immersion force feedback conditionally compiled
#[cfg(feature = "immersion")]
const _IMMERSION: bool = true;
#[cfg(not(feature = "immersion"))]
const _IMMERSION: bool = false;

// Xbox conditionally compiled
#[cfg(feature = "xbox")]
const _XBOX: bool = true;
#[cfg(not(feature = "xbox"))]
const _XBOX: bool = false;

// ===============
// CG_CheckAmmo
//
// If the ammo has gone low enough to generate the warning, play a sound
// ===============
pub fn CG_CheckAmmo() {
    //	int		i;
    //	int		weapons;

    #[cfg(never)]
    {
        // see about how many seconds of ammo we have remaining
        // let weapons = unsafe { (*addr_of_mut!(cg)).snap.read().ps.stats[STAT_WEAPONS as usize] };
        // let mut total = 0;

        // for ( i = WP_SABER; i < WP_NUM_WEAPONS  i++ )
        // {
        //     if ( ! ( weapons & ( 1 << i ) ) )
        //         continue;
        //
        //     /*
        //     switch ( i )
        //     {
        //     case WP_ROCKET_LAUNCHER:
        //     case WP_GRENADE_LAUNCHER:
        //     case WP_RAILGUN:
        //     case WP_SHOTGUN:
        //         total += cg.snap->ps.ammo[i] * 1000;
        //         break;
        //     default:
        //         total += cg.snap->ps.ammo[i] * 200;
        //         break;
        //     }
        //     */
        //
        //     if ( total >= 5000 )
        //     {
        //         cg.lowAmmoWarning = 0;
        //         return;
        //     }
        // }
    }

    unsafe {
        // Don't bother drawing the ammo warning when have no weapon selected
        if (*addr_of_mut!(cg)).weaponSelect == WP_NONE {
            return;
        }

        let total = (*addr_of_mut!(cg))
            .snap
            .read()
            .ps
            .ammo[weaponData[(*addr_of_mut!(cg)).weaponSelect as usize].ammoIndex as usize];

        if total > weaponData[(*addr_of_mut!(cg)).weaponSelect as usize].ammoLow {
            // Low on ammo?
            (*addr_of_mut!(cg)).lowAmmoWarning = 0;
            return;
        }

        let previous = (*addr_of_mut!(cg)).lowAmmoWarning;

        if total == 0 {
            // We're completely freak'in out!
            (*addr_of_mut!(cg)).lowAmmoWarning = 2;
        } else {
            // Got a little left
            (*addr_of_mut!(cg)).lowAmmoWarning = 1;
        }

        // play a sound on transitions
        if (*addr_of_mut!(cg)).lowAmmoWarning != previous {
            cgi_S_StartLocalSound((*addr_of_mut!(cgs)).media.noAmmoSound, CHAN_LOCAL_SOUND); //"sound/weapons/noammo.wav"
            if _IMMERSION {
                cgi_FF_Start((*addr_of_mut!(cgs)).media.noAmmoForce, FF_CLIENT_LOCAL);
            }
        }
    }
}

// ===============
// CG_DamageFeedback
// ===============
pub fn CG_DamageFeedback(yawByte: u8, pitchByte: u8, damage: c_int) {
    let mut left: f32 = 0.0;
    let mut front: f32 = 0.0;
    let mut up: f32 = 0.0;
    let mut kick: f32;
    let health: c_int;
    let mut scale: f32;
    let mut dir: [f32; 3] = [0.0; 3];
    let mut angles: [f32; 3] = [0.0; 3];
    let mut dist: f32;
    let mut yaw: f32 = 0.0;
    let mut pitch: f32 = 0.0;

    //FIXME: Based on MOD, do different kinds of damage effects,
    //		for example, Borg damage could progressively tint screen green and raise FOV?

    unsafe {
        // the lower on health you are, the greater the view kick will be
        health = (*addr_of_mut!(cg))
            .snap
            .read()
            .ps
            .stats[STAT_HEALTH];
        if health < 40 {
            scale = 1.0;
        } else {
            scale = 40.0 / health as f32;
        }
        kick = damage as f32 * scale;

        if kick < 5.0 {
            kick = 5.0;
        }
        if kick > 10.0 {
            kick = 10.0;
        }

        // if yaw and pitch are both 255, make the damage always centered (falling, etc)
        if yawByte == 255 && pitchByte == 255 {
            (*addr_of_mut!(cg)).damageX = 0.0;
            (*addr_of_mut!(cg)).damageY = 0.0;
            (*addr_of_mut!(cg)).v_dmg_roll = 0.0;
            (*addr_of_mut!(cg)).v_dmg_pitch = -kick;
        } else {
            // positional
            pitch = (pitchByte as f32) / 255.0 * 360.0;
            yaw = (yawByte as f32) / 255.0 * 360.0;

            angles[PITCH] = pitch;
            angles[YAW] = yaw;
            angles[ROLL] = 0.0;

            AngleVectors(&angles, &mut dir, core::ptr::null_mut(), core::ptr::null_mut());
            VectorSubtract(&[0.0, 0.0, 0.0], &dir, &mut dir);

            front = DotProduct(&dir, &(*addr_of_mut!(cg)).refdef.viewaxis[0]);
            left = DotProduct(&dir, &(*addr_of_mut!(cg)).refdef.viewaxis[1]);
            up = DotProduct(&dir, &(*addr_of_mut!(cg)).refdef.viewaxis[2]);

            dir[0] = front;
            dir[1] = left;
            dir[2] = 0.0;
            dist = VectorLength(&dir);
            if dist < 0.1 {
                dist = 0.1;
            }

            (*addr_of_mut!(cg)).v_dmg_roll = kick * left;

            (*addr_of_mut!(cg)).v_dmg_pitch = -kick * front;

            if front <= 0.1 {
                front = 0.1;
            }
            (*addr_of_mut!(cg)).damageX = -left / front;
            (*addr_of_mut!(cg)).damageY = up / dist;
        }

        // clamp the position
        if (*addr_of_mut!(cg)).damageX > 1.0 {
            (*addr_of_mut!(cg)).damageX = 1.0;
        }
        if (*addr_of_mut!(cg)).damageX < -1.0 {
            (*addr_of_mut!(cg)).damageX = -1.0;
        }

        if (*addr_of_mut!(cg)).damageY > 1.0 {
            (*addr_of_mut!(cg)).damageY = 1.0;
        }
        if (*addr_of_mut!(cg)).damageY < -1.0 {
            (*addr_of_mut!(cg)).damageY = -1.0;
        }

        // don't let the screen flashes vary as much
        if kick > 10.0 {
            kick = 10.0;
        }
        (*addr_of_mut!(cg)).damageValue = kick;
        (*addr_of_mut!(cg)).v_dmg_time = (*addr_of_mut!(cg)).time + DAMAGE_TIME;
        (*addr_of_mut!(cg)).damageTime = (*addr_of_mut!(cg)).snap.read().serverTime;
        if _IMMERSION {
            cgi_FF_Start(
                cgi_FF_Register(b"fffx/player/damage\0".as_ptr(), FF_CHANNEL_DAMAGE),
                (*addr_of_mut!(cg)).snap.read().ps.clientNum as c_int,
            );
        }

        if _XBOX {
            cgi_FF_Xbox_Damage(damage, (*addr_of_mut!(cg)).damageX);
        }
    }
}

// ================
// CG_Respawn
//
// A respawn happened this snapshot
// ================
pub fn CG_Respawn() {
    unsafe {
        // no error decay on player movement
        (*addr_of_mut!(cg)).thisFrameTeleport = 1;

        // display weapons available
        //	cg.weaponSelectTime = cg.time;
        SetWeaponSelectTime();

        // select the weapon the server says we are using
        if (*addr_of_mut!(cg)).snap.read().ps.weapon != 0 {
            (*addr_of_mut!(cg)).weaponSelect = (*addr_of_mut!(cg)).snap.read().ps.weapon as c_int;
        }
    }
}

// ===============
// CG_CheckPlayerstateEvents
//
// ===============
pub fn CG_CheckPlayerstateEvents(ps: *mut playerState_t, ops: *mut playerState_t) {
    #[cfg(never)]
    {
        // if ( ps->externalEvent && ps->externalEvent != ops->externalEvent ) {
        //     cent = &cg_entities[ ps->clientNum ];
        //     cent->currentState.event = ps->externalEvent;
        //     cent->currentState.eventParm = ps->externalEventParm;
        //     CG_EntityEvent( cent, cent->lerpOrigin );
        // }
    }

    unsafe {
        let mut i = (*ps).eventSequence - MAX_PS_EVENTS;
        while i < (*ps).eventSequence {
            let idx = (i & (MAX_PS_EVENTS - 1)) as usize;
            if (*ps).events[idx] != (*ops).events[idx] || i >= (*ops).eventSequence {
                let event = (*ps).events[idx];

                let cent = addr_of_mut!(cg_entities[(*ps).clientNum as usize]);
                (*cent).currentState.event = event as c_int;
                (*cent).currentState.eventParm = (*ps).eventParms[idx] as u8;
                CG_EntityEvent(cent, addr_of!((*cent).lerpOrigin));
            }
            i += 1;
        }
    }
}

// ==================
// CG_CheckLocalSounds
// ==================
/*
void CG_CheckLocalSounds( playerState_t *ps, playerState_t *ops ) {
    const char *s;

    // hit changes
    if ( ps->persistant[PERS_HITS] > ops->persistant[PERS_HITS] ) {
        cgi_S_StartLocalSound( "sound/feedback/hit.wav" );
    } else if ( ps->persistant[PERS_HITS] < ops->persistant[PERS_HITS] ) {
        cgi_S_StartLocalSound( "sound/feedback/hit_teammate.wav" );
    }

    // score up / down changes
    if ( ps->persistant[PERS_SCORE] > ops->persistant[PERS_SCORE] ) {
        cgi_S_StartLocalSound( "sound/feedback/scoreup.wav" );
    } else if ( ps->persistant[PERS_SCORE] < ops->persistant[PERS_SCORE] ) {
        cgi_S_StartLocalSound( "sound/feedback/scoredown.wav" );
    }

    // reward sounds
    if ( ps->persistant[PERS_REWARD_COUNT] > ops->persistant[PERS_REWARD_COUNT] ) {
        switch ( ps->persistant[PERS_REWARD] ) {
        case REWARD_IMPRESSIVE:
            cgi_S_StartLocalSound( "sound/feedback/impressive.wav" );
            break;
        case REWARD_EXCELLENT:
            cgi_S_StartLocalSound( "sound/feedback/excellent.wav" );
            break;
        case REWARD_DENIED:
            cgi_S_StartLocalSound( "sound/feedback/denied.wav" );
            break;

        default:
            CG_Error( "Bad reward_t" );
        }
    }

    // timelimit warnings
    if ( cgs.timelimit > 0 ) {
        if ( cgs.timelimit > 5 && !( cg.timelimitWarnings & 1 ) && cg.time > (cgs.timelimit - 5) * 60 * 1000 ) {
            cg.timelimitWarnings |= 1;
            cgi_S_StartLocalSound( "sound/feedback/5_minute.wav" );
        }
        if ( !( cg.timelimitWarnings & 2 ) && cg.time > (cgs.timelimit - 1) * 60 * 1000 ) {
            cg.timelimitWarnings |= 2;
            cgi_S_StartLocalSound( "sound/feedback/1_minute.wav" );
        }
        if ( !( cg.timelimitWarnings & 4 ) && cg.time > ( cgs.timelimit * 60 + 2 ) * 1000 ) {
            cg.timelimitWarnings |= 4;
            cgi_S_StartLocalSound( "sound/feedback/sudden_death.wav" );
        }
    }
}
*/

// ===============
// CG_TransitionPlayerState
//
// ===============
pub fn CG_TransitionPlayerState(ps: *mut playerState_t, ops: *mut playerState_t) {
    unsafe {
        // teleporting
        if ((*ps).eFlags ^ (*ops).eFlags) & EF_TELEPORT_BIT != 0 {
            (*addr_of_mut!(cg)).thisFrameTeleport = 1;
        } else {
            (*addr_of_mut!(cg)).thisFrameTeleport = 0;
        }

        // check for changing follow mode
        if (*ps).clientNum != (*ops).clientNum {
            (*addr_of_mut!(cg)).thisFrameTeleport = 1;
            // make sure we don't get any unwanted transition effects
            *ops = *ps;
        }

        // damage events (player is getting wounded)
        if (*ps).damageEvent != (*ops).damageEvent && (*ps).damageCount != 0 {
            CG_DamageFeedback((*ps).damageYaw, (*ps).damagePitch, (*ps).damageCount as c_int);
        }

        // respawning
        if (*ps).persistant[PERS_SPAWN_COUNT] != (*ops).persistant[PERS_SPAWN_COUNT] {
            CG_Respawn();
        }

        // check for going low on ammo
        CG_CheckAmmo();

        // run events
        CG_CheckPlayerstateEvents(ps, ops);

        // smooth the ducking viewheight change
        if (*ps).viewheight != (*ops).viewheight {
            if (*addr_of_mut!(cg)).nextFrameTeleport == 0 {
                //when we crouch/uncrouch in mid-air, our viewhieght doesn't actually change in
                //absolute world coordinates, just locally.
                (*addr_of_mut!(cg)).duckChange = (*ps).viewheight - (*ops).viewheight;
                (*addr_of_mut!(cg)).duckTime = (*addr_of_mut!(cg)).time;
            }
        }
    }
}
