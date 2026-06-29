// leave this line at the top for all g_xxxx.cpp files...
// g_headers.h

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{null_mut, addr_of_mut};

// Simplified C type stubs for compilation
// These are declared but not defined here; they come from external C linkage
#[repr(C)]
pub struct gentity_t {
    // Placeholder - actual definition in other modules
}

#[repr(C)]
pub struct trace_t {
    // Placeholder - actual definition in other modules
}

#[repr(C)]
pub struct gclient_t {
    // Placeholder - actual definition in other modules
}

#[repr(C)]
pub struct level_t {
    // Placeholder - actual definition in other modules
}

extern "C" {
    // From Q3_Interface.h and g_local.h
    pub static mut level: level_t;
    pub static mut g_entities: gentity_t;
    pub static mut globals: level_t;
    pub static mut cg_entities: [c_void; 0];
    pub static mut gi: GameInterface;
    pub static mut cgs: CGameState;
    pub static mut com_buildScript: *mut cvar_t;

    // Functions
    pub fn G_SetEnemy(self_: *mut gentity_t, enemy: *mut gentity_t);
    pub fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int);
    pub fn G_Find(from: *mut gentity_t, fofs: c_int, match_: *const c_char) -> *mut gentity_t;
    pub fn Touch_Item(ent: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn G_SoundIndex(name: *const c_char) -> c_int;
    pub fn G_UseTargets(self_: *mut gentity_t, other: *mut gentity_t);
    pub fn G_SpawnFloat(key: *const c_char, default: *const c_char, out: *mut f32) -> c_int;
    pub fn G_SpawnString(key: *const c_char, default: *const c_char, out: *mut *mut c_char) -> c_int;
    pub fn crandom() -> f32;
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;
    pub fn AddScore(ent: *mut gentity_t, score: c_int);
    pub fn va(fmt: *const c_char, ...) -> *mut c_char;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn COM_DefaultExtension(path: *mut c_char, maxlen: c_int, ext: *const c_char);
    pub fn G_Error(fmt: *const c_char, ...);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn VectorMA(veca: *const f32, scale: f32, vecb: *const f32, vecc: *mut f32);
    pub fn VectorSubtract(veca: *const f32, vecb: *const f32, out: *mut f32);
    pub fn VectorNormalize(vec: *mut f32) -> f32;
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn G_Damage(
        targ: *mut gentity_t,
        inflictor: *mut gentity_t,
        attacker: *mut gentity_t,
        dir: *const f32,
        point: *const f32,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    );
    pub fn G_UseTargets2(self_: *mut gentity_t, other: *mut gentity_t, target2: *const c_char);
    pub fn GEntity_UseFunc(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn G_SetOrigin(ent: *mut gentity_t, origin: *const f32);
    pub fn G_PickTarget(targetname: *const c_char) -> *mut gentity_t;
    pub fn TeleportPlayer(client: *mut gentity_t, origin: *const f32, angles: *const f32);
    pub fn G_SetMovedir(angles: *const f32, movedir: *mut f32);
    pub fn G_Find(from: *mut gentity_t, fofs: c_int, match_: *const c_char) -> *mut gentity_t;
    pub fn Quake3Game() -> *mut IGameInterface;
    pub fn FindItemForWeapon(wpn: weapon_t) -> *mut gitem_t;
    pub fn cgi_SP_GetStringTextString(
        text: *const c_char,
        buf: *mut c_char,
        buflen: c_int,
    );
    pub fn cgi_S_StopSounds();
    pub fn cgi_S_StartSound(origin: *const f32, entitynum: c_int, entchannel: c_int, sfx: c_int);
    pub fn G_ChangeMap(mapname: *const c_char, spawntarget: *const c_char, hub: c_int);
    pub fn Q3_SetParm(entID: c_int, parmNum: c_int, parmValue: *const c_char);
    pub fn CG_CenterPrint(text: *const c_char, y: f32);
    pub fn G_Sound(ent: *mut gentity_t, index: c_int);
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn VALIDSTRING(ptr: *const c_char) -> c_int;
}

#[repr(C)]
pub struct GameInterface {
    // Placeholder - actual definition elsewhere
}

#[repr(C)]
pub struct CGameState {
    // Placeholder - actual definition elsewhere
}

#[repr(C)]
pub struct cvar_t {
    // Placeholder - actual definition elsewhere
}

#[repr(C)]
pub struct gitem_t {
    // Placeholder - actual definition elsewhere
}

pub type weapon_t = c_int;
pub type fileHandle_t = c_int;

const FOFS: fn() -> c_int = || 0; // Placeholder
const BSET_USE: c_int = 0; // Placeholder
const useF_Use_Target_Give: c_int = 0; // Placeholder
const useF_Use_Target_Delay: c_int = 0; // Placeholder
const useF_Use_Target_Score: c_int = 0; // Placeholder
const useF_Use_Target_Print: c_int = 0; // Placeholder
const useF_Use_Target_Speaker: c_int = 0; // Placeholder
const useF_target_laser_use: c_int = 0; // Placeholder
const useF_target_teleporter_use: c_int = 0; // Placeholder
const useF_target_relay_use: c_int = 0; // Placeholder
const useF_target_kill_use: c_int = 0; // Placeholder
const useF_target_random_use: c_int = 0; // Placeholder
const useF_target_scriptrunner_use: c_int = 0; // Placeholder
const useF_target_gravity_change_use: c_int = 0; // Placeholder
const useF_target_friction_change_use: c_int = 0; // Placeholder
const useF_target_play_music_use: c_int = 0; // Placeholder
const useF_target_autosave_use: c_int = 0; // Placeholder
const useF_target_secret_use: c_int = 0; // Placeholder
const useF_target_counter_use: c_int = 0; // Placeholder
const useF_NULL: c_int = 0; // Placeholder
const thinkF_Think_Target_Delay: c_int = 0; // Placeholder
const thinkF_target_laser_think: c_int = 0; // Placeholder
const thinkF_target_laser_start: c_int = 0; // Placeholder
const thinkF_target_relay_use_go: c_int = 0; // Placeholder
const thinkF_scriptrunner_run: c_int = 0; // Placeholder
const thinkF_target_location_linkup: c_int = 0; // Placeholder
const EV_GENERAL_SOUND: c_int = 0; // Placeholder
const EV_GLOBAL_SOUND: c_int = 0; // Placeholder
const ET_SPEAKER: c_int = 0; // Placeholder
const ET_BEAM: c_int = 0; // Placeholder
const START_TIME_LINK_ENTS: c_int = 0; // Placeholder
const FRAMETIME: c_int = 0; // Placeholder
const CONTENTS_SOLID: c_int = 0; // Placeholder
const CONTENTS_BODY: c_int = 0; // Placeholder
const CONTENTS_CORPSE: c_int = 0; // Placeholder
const DAMAGE_NO_KNOCKBACK: c_int = 0; // Placeholder
const DAMAGE_NO_PROTECTION: c_int = 0; // Placeholder
const MOD_ENERGY: c_int = 0; // Placeholder
const MOD_FALLING: c_int = 0; // Placeholder
const MOD_ELECTROCUTE: c_int = 0; // Placeholder
const MOD_UNKNOWN: c_int = 0; // Placeholder
const PW_SHOCKED: c_int = 0; // Placeholder
const SVF_BROADCAST: c_int = 0; // Placeholder
const SVF_INACTIVE: c_int = 0; // Placeholder
const SVF_CUSTOM_GRAVITY: c_int = 0; // Placeholder
const MAX_QPATH: usize = 256; // Placeholder
const MAX_FILENAME_LENGTH: usize = 256; // Placeholder
const MAX_PARMS: c_int = 16; // Placeholder
const WP_NUM_WEAPONS: c_int = 16; // Placeholder
const SCREEN_HEIGHT: f32 = 480.0; // Placeholder
const CS_MUSIC: c_int = 0; // Placeholder
const FS_READ: c_int = 0; // Placeholder
const CHAN_VOICE: c_int = 0; // Placeholder
const FP_ABSORB: c_int = 0; // Placeholder
const FP_HEAL: c_int = 1; // Placeholder
const FP_TELEPATHY: c_int = 2; // Placeholder
const FP_PROTECT: c_int = 3; // Placeholder
const FP_LEVITATION: c_int = 4; // Placeholder
const FP_PULL: c_int = 5; // Placeholder
const FP_PUSH: c_int = 6; // Placeholder
const FP_SEE: c_int = 7; // Placeholder
const FP_SPEED: c_int = 8; // Placeholder
const FP_SABER_DEFENSE: c_int = 9; // Placeholder
const FP_SABER_OFFENSE: c_int = 10; // Placeholder
const FP_SABERTHROW: c_int = 11; // Placeholder
const FP_DRAIN: c_int = 12; // Placeholder
const FP_GRIP: c_int = 13; // Placeholder
const FP_LIGHTNING: c_int = 14; // Placeholder
const FP_RAGE: c_int = 15; // Placeholder

//==========================================================

/*QUAKED target_give (1 0 0) (-8 -8 -8) (8 8 8)
Gives the activator all the items pointed to.
*/
pub unsafe fn Use_Target_Give(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    let mut t: *mut gentity_t;
    let mut trace: trace_t;

    if (*activator).client.is_null() {
        return;
    }

    if (*ent).target.is_null() {
        return;
    }

    G_ActivateBehavior(ent, BSET_USE);

    memset(&mut trace as *mut _ as *mut c_void, 0, std::mem::size_of::<trace_t>());
    t = null_mut();
    loop {
        t = G_Find(t, FOFS(), (*ent).target);
        if t.is_null() {
            break;
        }
        if (*t).item.is_null() {
            continue;
        }
        Touch_Item(t, activator, &mut trace);

        // make sure it isn't going to respawn or show any events
        (*t).nextthink = 0;
        gi.unlinkentity(t);
    }
}

pub unsafe fn SP_target_give(ent: *mut gentity_t) {
    (*ent).e_UseFunc = useF_Use_Target_Give;
}

//==========================================================

/*QUAKED target_delay (1 0 0) (-8 -8 -8) (8 8 8)
"wait" seconds to pause before firing targets.
"random" delay variance, total delay = delay +/- random seconds
*/
pub unsafe fn Think_Target_Delay(ent: *mut gentity_t) {
    G_UseTargets(ent, (*ent).activator);
}

pub unsafe fn Use_Target_Delay(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(ent, BSET_USE);

    (*ent).nextthink = level.time + (((*ent).wait + (*ent).random * crandom()) * 1000.0) as c_int;
    (*ent).e_ThinkFunc = thinkF_Think_Target_Delay;
    (*ent).activator = activator;
}

pub unsafe fn SP_target_delay(ent: *mut gentity_t) {
    // check delay for backwards compatability
    if G_SpawnFloat("delay\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, &mut (*ent).wait) == 0 {
        G_SpawnFloat("wait\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, &mut (*ent).wait);
    }

    if (*ent).wait == 0.0 {
        (*ent).wait = 1.0;
    }

    (*ent).e_UseFunc = useF_Use_Target_Delay;
}


//==========================================================

/*QUAKED target_score (1 0 0) (-8 -8 -8) (8 8 8)
"count" number of points to add, default 1

The activator is given this many points.
*/
pub unsafe fn Use_Target_Score(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(ent, BSET_USE);

    AddScore(activator, (*ent).count);
}

pub unsafe fn SP_target_score(ent: *mut gentity_t) {
    if (*ent).count == 0 {
        (*ent).count = 1;
    }
    (*ent).e_UseFunc = useF_Use_Target_Score;
}


//==========================================================

/*QUAKED target_print (1 0 0) (-8 -8 -8) (8 8 8)
"message"	text to print
If "private", only the activator gets the message.  If no checks, all clients get the message.
*/
pub unsafe fn Use_Target_Print(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(ent, BSET_USE);

    if !(*activator).client.is_null() {
        gi.SendServerCommand(activator as c_int - g_entities as c_int, "cp \"%s\"\0".as_ptr() as *const c_char, (*ent).message);
    }
}

pub unsafe fn SP_target_print(ent: *mut gentity_t) {
    (*ent).e_UseFunc = useF_Use_Target_Print;
}


//==========================================================


/*QUAKED target_speaker (1 0 0) (-8 -8 -8) (8 8 8) looped-on looped-off global activator
"noise"		wav file to play

"sounds" va() min max, so if your sound string is borgtalk%d.wav, and you set a "sounds" value of 4, it will randomly play borgtalk1.wav - borgtalk4.wav when triggered
to use this, you must store the wav name in "soundGroup", NOT "noise"

A global sound will play full volume throughout the level.
Activator sounds will play on the player that activated the target.
Global and activator sounds can't be combined with looping.
Normal sounds play each time the target is used.
Looped sounds will be toggled by use functions.
Multiple identical looping sounds will just increase volume without any speed cost.
"wait" : Seconds between triggerings, 0 = don't auto trigger
"random"	wait variance, default is 0
*/
pub unsafe fn Use_Target_Speaker(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if (*ent).painDebounceTime > level.time {
        return;
    }

    G_ActivateBehavior(ent, BSET_USE);

    if (*ent).sounds != 0 {
        (*ent).noise_index = G_SoundIndex(va("%.100s%d\0".as_ptr() as *const c_char, (*ent).paintarget, Q_irand(1, (*ent).sounds)));
    }

    if ((*ent).spawnflags & 3) != 0 {	// looping sound toggles
        let mut looper: *mut gentity_t = ent;
        if ((*ent).spawnflags & 8) != 0 {
            looper = activator;
        }
        if (*looper).s.loopSound != 0 {
            (*looper).s.loopSound = 0;	// turn it off
        } else {
            (*looper).s.loopSound = (*ent).noise_index;	// start it
        }
    } else {	// normal sound
        if ((*ent).spawnflags & 8) != 0 {
            G_AddEvent(activator, EV_GENERAL_SOUND, (*ent).noise_index);
        } else if ((*ent).spawnflags & 4) != 0 {
            G_AddEvent(ent, EV_GLOBAL_SOUND, (*ent).noise_index);
        } else {
            G_AddEvent(ent, EV_GENERAL_SOUND, (*ent).noise_index);
        }
    }

    if (*ent).wait < 0.0 {//BYE!
        (*ent).e_UseFunc = useF_NULL;
    } else {
        (*ent).painDebounceTime = level.time + (*ent).wait as c_int;
    }
}

pub unsafe fn SP_target_speaker(ent: *mut gentity_t) {
    let mut buffer: [c_char; MAX_QPATH] = [0; MAX_QPATH];
    let mut s: *mut c_char;
    let mut i: c_int;

    if VALIDSTRING((*ent).soundSet) != 0 {
        VectorCopy((*ent).s.origin.as_ptr(), (*ent).s.pos.trBase.as_mut_ptr());
        gi.linkentity(ent);
        return;
    }

    G_SpawnFloat("wait\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, &mut (*ent).wait);
    G_SpawnFloat("random\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, &mut (*ent).random);

    if (*ent).sounds == 0 {
        if G_SpawnString("noise\0".as_ptr() as *const c_char, "*NOSOUND*\0".as_ptr() as *const c_char, &mut s) == 0 {
            G_Error("target_speaker without a noise key at %s\0".as_ptr() as *const c_char, vtos((*ent).s.origin.as_ptr()));
        }

        Q_strncpyz(buffer.as_mut_ptr(), s, MAX_QPATH as c_int);
        COM_DefaultExtension(buffer.as_mut_ptr(), MAX_QPATH as c_int, ".wav\0".as_ptr() as *const c_char);
        (*ent).noise_index = G_SoundIndex(buffer.as_ptr());
    } else {//Precache all possible sounds
        i = 0;
        while i < (*ent).sounds {
            (*ent).noise_index = G_SoundIndex(va("%.100s%d\0".as_ptr() as *const c_char, (*ent).paintarget, i + 1));
            i += 1;
        }
    }

    // a repeating speaker can be done completely client side
    (*ent).s.eType = ET_SPEAKER;
    (*ent).s.eventParm = (*ent).noise_index;
    (*ent).s.frame = ((*ent).wait * 10.0) as c_int;
    (*ent).s.clientNum = ((*ent).random * 10.0) as c_int;

    (*ent).wait *= 1000.0;

    // check for prestarted looping sound
    if ((*ent).spawnflags & 1) != 0 {
        (*ent).s.loopSound = (*ent).noise_index;
    }

    (*ent).e_UseFunc = useF_Use_Target_Speaker;

    if ((*ent).spawnflags & 4) != 0 {
        (*ent).svFlags |= SVF_BROADCAST;
    }

    VectorCopy((*ent).s.origin.as_ptr(), (*ent).s.pos.trBase.as_mut_ptr());

    // must link the entity so we get areas and clusters so
    // the server can determine who to send updates to
    gi.linkentity(ent);
}



//==========================================================

/*QUAKED target_laser (0 .5 .8) (-8 -8 -8) (8 8 8) START_ON
When triggered, fires a laser.  You can either set a target or a direction.
*/
pub unsafe fn target_laser_think(self_: *mut gentity_t) {
    let mut end: [f32; 3] = [0.0; 3];
    let mut tr: trace_t;
    let mut point: [f32; 3] = [0.0; 3];

    // if pointed at another entity, set movedir to point at it
    if !(*self_).enemy.is_null() {
        VectorMA((*(*self_).enemy).s.origin.as_ptr(), 0.5, (*(*self_).enemy).mins.as_ptr(), point.as_mut_ptr());
        VectorMA(point.as_ptr(), 0.5, (*(*self_).enemy).maxs.as_ptr(), point.as_mut_ptr());
        VectorSubtract(point.as_ptr(), (*self_).s.origin.as_ptr(), (*self_).movedir.as_mut_ptr());
        VectorNormalize((*self_).movedir.as_mut_ptr());
    }

    // fire forward and see what we hit
    VectorMA((*self_).s.origin.as_ptr(), 2048.0, (*self_).movedir.as_ptr(), end.as_mut_ptr());

    gi.trace(&mut tr, (*self_).s.origin.as_ptr(), null_mut(), null_mut(), end.as_ptr(), (*self_).s.number, CONTENTS_SOLID | CONTENTS_BODY | CONTENTS_CORPSE);

    if tr.entityNum != 0 {
        // hurt it if we can
        G_Damage(&mut g_entities as *mut _ as *mut gentity_t, self_, self_, (*self_).movedir.as_ptr(),
            tr.endpos.as_ptr(), (*self_).damage, DAMAGE_NO_KNOCKBACK, MOD_ENERGY);
    }

    VectorCopy(tr.endpos.as_ptr(), (*self_).s.origin2.as_mut_ptr());

    gi.linkentity(self_);
    (*self_).nextthink = level.time + FRAMETIME;
}

pub unsafe fn target_laser_on(self_: *mut gentity_t) {
    if (*self_).activator.is_null() {
        (*self_).activator = self_;
    }
    target_laser_think(self_);
}

pub unsafe fn target_laser_off(self_: *mut gentity_t) {
    gi.unlinkentity(self_);
    (*self_).nextthink = 0;
}

pub unsafe fn target_laser_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(self_, BSET_USE);

    (*self_).activator = activator;
    if (*self_).nextthink > 0 {
        target_laser_off(self_);
    } else {
        target_laser_on(self_);
    }
}

pub unsafe fn target_laser_start(self_: *mut gentity_t) {
    let mut ent: *mut gentity_t;

    (*self_).s.eType = ET_BEAM;

    if !(*self_).target.is_null() {
        ent = G_Find(null_mut(), FOFS(), (*self_).target);
        if ent.is_null() {
            gi.Printf("%s at %s: %s is a bad target\n\0".as_ptr() as *const c_char, (*self_).classname, vtos((*self_).s.origin.as_ptr()), (*self_).target);
        }
        G_SetEnemy(self_, ent);
    } else {
        G_SetMovedir((*self_).s.angles.as_ptr(), (*self_).movedir.as_mut_ptr());
    }

    (*self_).e_UseFunc = useF_target_laser_use;
    (*self_).e_ThinkFunc = thinkF_target_laser_think;

    if (*self_).damage == 0 {
        (*self_).damage = 1;
    }

    if ((*self_).spawnflags & 1) != 0 {
        target_laser_on(self_);
    } else {
        target_laser_off(self_);
    }
}

pub unsafe fn SP_target_laser(self_: *mut gentity_t) {
    // let everything else get spawned before we start firing
    (*self_).e_ThinkFunc = thinkF_target_laser_start;
    (*self_).nextthink = level.time + START_TIME_LINK_ENTS;
}


//==========================================================

pub unsafe fn target_teleporter_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    let mut dest: *mut gentity_t;

    if (*activator).client.is_null() {
        return;
    }

    G_ActivateBehavior(self_, BSET_USE);

    dest = G_PickTarget((*self_).target);
    if dest.is_null() {
        gi.Printf("Couldn't find teleporter destination\n\0".as_ptr() as *const c_char);
        return;
    }

    TeleportPlayer(activator, (*dest).s.origin.as_ptr(), (*dest).s.angles.as_ptr());
}

/*QUAK-ED target_teleporter (1 0 0) (-8 -8 -8) (8 8 8)
The activator will be teleported away.
*/
pub unsafe fn SP_target_teleporter(self_: *mut gentity_t) {
    if (*self_).targetname.is_null() {
        gi.Printf("untargeted %s at %s\n\0".as_ptr() as *const c_char, (*self_).classname, vtos((*self_).s.origin.as_ptr()));
    }

    (*self_).e_UseFunc = useF_target_teleporter_use;
}

//==========================================================


/*QUAKED target_relay (.5 .5 .5) (-8 -8 -8) (8 8 8) RED_ONLY BLUE_ONLY RANDOM x x x x INACTIVE
This doesn't perform any actions except fire its targets.
The activator can be forced to be from a certain team.
if RANDOM is checked, only one of the targets will be fired, not all of them

INACTIVE  Can't be used until activated

  "delay" - Will actually fire this many seconds after being used
  "wait" - Cannot be fired again until this many seconds after the last time it was used
*/
pub unsafe fn target_relay_use_go(self_: *mut gentity_t) {
    G_ActivateBehavior(self_, BSET_USE);

    if ((*self_).spawnflags & 4) != 0 {
        let mut ent: *mut gentity_t;

        ent = G_PickTarget((*self_).target);
        if !ent.is_null() && (*ent).e_UseFunc != useF_NULL {	// e_UseFunc check can be omitted
            GEntity_UseFunc(ent, self_, (*self_).activator);
        }
        return;
    }

    G_UseTargets(self_, (*self_).activator);
}

pub unsafe fn target_relay_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if ((*self_).spawnflags & 1) != 0 && !(*activator).client.is_null() {
        //&& activator->client->ps.persistant[PERS_TEAM] != TEAM_RED ) {
        return;
    }

    if ((*self_).spawnflags & 2) != 0 && !(*activator).client.is_null() {
        //&& activator->client->ps.persistant[PERS_TEAM] != TEAM_BLUE ) {
        return;
    }

    if ((*self_).svFlags & SVF_INACTIVE) != 0 {
        //set by target_deactivate
        return;
    }

    if (*self_).painDebounceTime > level.time {
        return;
    }

    G_SetEnemy(self_, other);
    (*self_).activator = activator;

    if (*self_).delay != 0 {
        (*self_).e_ThinkFunc = thinkF_target_relay_use_go;
        (*self_).nextthink = level.time + (*self_).delay;
        return;
    }

    target_relay_use_go(self_);

    if (*self_).wait < 0.0 {
        (*self_).e_UseFunc = useF_NULL;
    } else {
        (*self_).painDebounceTime = level.time + (*self_).wait as c_int;
    }
}

pub unsafe fn SP_target_relay(self_: *mut gentity_t) {
    (*self_).e_UseFunc = useF_target_relay_use;
    (*self_).wait *= 1000.0;
    (*self_).delay *= 1000.0;
    if ((*self_).spawnflags & 128) != 0 {
        (*self_).svFlags |= SVF_INACTIVE;
    }
}


//==========================================================

/*QUAKED target_kill (.5 .5 .5) (-8 -8 -8) (8 8 8) FALLING ELECTRICAL
Kills the activator.
*/
pub unsafe fn target_kill_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {

    G_ActivateBehavior(self_, BSET_USE);

    if ((*self_).spawnflags & 1) != 0 {
        //falling death
        G_Damage(activator, null_mut(), null_mut(), null_mut(), null_mut(), 100000, DAMAGE_NO_PROTECTION, MOD_FALLING);
        if (*activator).s.number == 0 && (*activator).health <= 0 && true {
            extern "C" {
                pub fn CGCam_Fade(source: *const f32, dest: *const f32, duration: f32);
            }
            let src: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
            let dst: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
            CGCam_Fade(src.as_ptr(), dst.as_ptr(), 10000.0);
        }
    } else if ((*self_).spawnflags & 2) != 0 {
        // electrical
        G_Damage(activator, null_mut(), null_mut(), null_mut(), null_mut(), 100000, DAMAGE_NO_PROTECTION, MOD_ELECTROCUTE);

        if !(*activator).client.is_null() {
            (*(*activator).client).ps.powerups = ((*(*activator).client).ps.powerups | (1 << PW_SHOCKED)) as c_int;
            (*(*activator).client).ps.powerups = level.time + 4000;
        }
    } else {
        G_Damage(activator, null_mut(), null_mut(), null_mut(), null_mut(), 100000, DAMAGE_NO_PROTECTION, MOD_UNKNOWN);
    }
}

pub unsafe fn SP_target_kill(self_: *mut gentity_t) {
    (*self_).e_UseFunc = useF_target_kill_use;
}

/*QUAKED target_position (0 0.5 0) (-4 -4 -4) (4 4 4)
Used as a positional target for in-game calculation, like jumppad targets.
info_notnull does the same thing
*/
pub unsafe fn SP_target_position(self_: *mut gentity_t) {
    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
}

//static -slc
pub unsafe fn target_location_linkup(ent: *mut gentity_t) {
    let mut i: c_int;

    if level.locationLinked != 0 {
        return;
    }

    level.locationLinked = 1; // qtrue

    level.locationHead = null_mut();

    i = 0;
    ent = &mut g_entities as *mut gentity_t;
    while i < globals.num_entities {
        if !(*ent).classname.is_null() && Q_stricmp((*ent).classname, "target_location\0".as_ptr() as *const c_char) == 0 {
            (*ent).nextTrain = level.locationHead;
            level.locationHead = ent;
        }
        i += 1;
        ent = ent.add(1);
    }

    // All linked together now
}

/*QUAKED target_location (0 0.5 0) (-8 -8 -8) (8 8 8)
Set "message" to the name of this location.
Set "count" to 0-7 for color.
0:white 1:red 2:green 3:yellow 4:blue 5:cyan 6:magenta 7:white

Closest target_location in sight used for the location, if none
in site, closest in distance
*/
pub unsafe fn SP_target_location(self_: *mut gentity_t) {
    (*self_).e_ThinkFunc = thinkF_target_location_linkup;
    (*self_).nextthink = level.time + 1000;  // Let them all spawn first

    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
}

//===NEW===================================================================

/*QUAKED target_counter (1.0 0 0) (-4 -4 -4) (4 4 4) x x x x x x x INACTIVE
Acts as an intermediary for an action that takes multiple inputs.

INACTIVE cannot be used until used by a target_activate

target2 - what the counter should fire each time it's incremented and does NOT reach it's count

After the counter has been triggered "count" times (default 2), it will fire all of it's targets and remove itself.

bounceCount - number of times the counter should reset to it's full count when it's done
*/

pub unsafe fn target_counter_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if (*self_).count == 0 {
        return;
    }

    //gi.Printf("target_counter %s used by %s, entnum %d\n", self->targetname, activator->targetname, activator->s.number );
    (*self_).count -= 1;

    if !activator.is_null() {
        (*Quake3Game()).DebugPrint(2, "target_counter %s used by %s (%d/%d)\n\0".as_ptr() as *const c_char, (*self_).targetname, (*activator).targetname, ((*self_).max_health - (*self_).count), (*self_).max_health);
    }

    if (*self_).count != 0 {
        if !(*self_).target2.is_null() {
            //gi.Printf("target_counter %s firing target2 from %s, entnum %d\n", self->targetname, activator->targetname, activator->s.number );
            G_UseTargets2(self_, activator, (*self_).target2);
        }
        return;
    }

    G_ActivateBehavior(self_, BSET_USE);

    if ((*self_).spawnflags & 128) != 0 {
        (*self_).svFlags |= SVF_INACTIVE;
    }

    (*self_).activator = activator;
    G_UseTargets(self_, activator);

    if (*self_).count == 0 {
        if (*self_).bounceCount == 0 {
            return;
        }
        (*self_).count = (*self_).max_health;
        if (*self_).bounceCount > 0 {
            //-1 means bounce back forever
            (*self_).bounceCount -= 1;
        }
    }
}

pub unsafe fn SP_target_counter(self_: *mut gentity_t) {
    (*self_).wait = -1.0;
    if (*self_).count == 0 {
        (*self_).count = 2;
    }
    //if ( self->bounceCount > 0 )//let's always set this anyway
    {//we will reset when we use up our count, remember our initial count
        (*self_).max_health = (*self_).count;
    }

    (*self_).e_UseFunc = useF_target_counter_use;
}

/*QUAKED target_random (.5 .5 .5) (-4 -4 -4) (4 4 4) USEONCE
Randomly fires off only one of it's targets each time used

USEONCE	set to never fire again
*/

pub unsafe fn target_random_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    let mut t_count: c_int = 0;
    let mut pick: c_int;
    let mut t: *mut gentity_t = null_mut();

    //gi.Printf("target_random %s used by %s (entnum %d)\n", self->targetname, activator->targetname, activator->s.number );
    G_ActivateBehavior(self_, BSET_USE);

    if ((*self_).spawnflags & 1) != 0 {
        (*self_).e_UseFunc = useF_NULL;
    }

    loop {
        t = G_Find(t, FOFS(), (*self_).target);
        if t.is_null() {
            break;
        }
        if t != self_ {
            t_count += 1;
        }
    }

    if t_count == 0 {
        return;
    }

    if t_count == 1 {
        G_UseTargets(self_, activator);
        return;
    }

    //FIXME: need a seed
    pick = Q_irand(1, t_count);
    t_count = 0;
    t = null_mut();
    loop {
        t = G_Find(t, FOFS(), (*self_).target);
        if t.is_null() {
            break;
        }
        if t != self_ {
            t_count += 1;
        } else {
            continue;
        }

        if t == self_ {
    //				gi.Printf ("WARNING: Entity used itself.\n");
        } else if t_count == pick {
            if (*t).e_UseFunc != useF_NULL {	// check can be omitted
                GEntity_UseFunc(t, self_, activator);
                return;
            }
        }

        if (*self_).inuse == 0 {
            gi.Printf("entity was removed while using targets\n\0".as_ptr() as *const c_char);
            return;
        }
    }
}

pub unsafe fn SP_target_random(self_: *mut gentity_t) {
    (*self_).e_UseFunc = useF_target_random_use;
}

static mut numNewICARUSEnts: c_int = 0;
pub unsafe fn scriptrunner_run(self_: *mut gentity_t) {
    /*
    if (self->behaviorSet[BSET_USE])
    {
        char	newname[MAX_FILENAME_LENGTH];

        sprintf((char *) &newname, "%s/%s", Q3_SCRIPT_DIR, self->behaviorSet[BSET_USE] );

        ICARUS_RunScript( self, newname );
    }
    */

    if (*self_).count != -1 {
        if (*self_).count <= 0 {
            (*self_).e_UseFunc = useF_NULL;
            (*self_).behaviorSet[0] = null_mut();
            return;
        } else {
            (*self_).count -= 1;
        }
    }

    if !(*self_).behaviorSet[0].is_null() {
        if ((*self_).spawnflags & 1) != 0 {
            if (*self_).activator.is_null() {
                (*Quake3Game()).DebugPrint(0, "target_scriptrunner tried to run on invalid entity!\n\0".as_ptr() as *const c_char);
                return;
            }

            if (*(*self_).activator).m_iIcarusID == -1 {
                //Need to be initialized through ICARUS
                if (*self_).activator.is_null() || (*(*self_).activator).script_targetname.is_null() || *(*(*self_).activator).script_targetname == 0 {
                    //We don't have a script_targetname, so create a new one
                    (*self_).activator = va("newICARUSEnt%d\0".as_ptr() as *const c_char, numNewICARUSEnts) as *mut gentity_t;
                    numNewICARUSEnts += 1;
                }

                if (*Quake3Game()).ValidEntity((*self_).activator) != 0 {
                    (*Quake3Game()).InitEntity((*self_).activator);
                } else {
                    (*Quake3Game()).DebugPrint(0, "target_scriptrunner tried to run on invalid ICARUS activator!\n\0".as_ptr() as *const c_char);
                    return;
                }
            }

            (*Quake3Game()).DebugPrint(2, "target_scriptrunner running %s on activator %s\n\0".as_ptr() as *const c_char, (*self_).behaviorSet[0], (*(*self_).activator).targetname);

            (*Quake3Game()).RunScript((*self_).activator, (*self_).behaviorSet[0]);
        } else {
            if !(*self_).activator.is_null() {
                (*Quake3Game()).DebugPrint(2, "target_scriptrunner %s used by %s\n\0".as_ptr() as *const c_char, (*self_).targetname, (*(*self_).activator).targetname);
            }
            G_ActivateBehavior(self_, BSET_USE);
        }
    }

    if (*self_).wait != 0.0 {
        (*self_).nextthink = level.time + (*self_).wait as c_int;
    }
}

pub unsafe fn target_scriptrunner_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if (*self_).nextthink > level.time {
        return;
    }

    (*self_).activator = activator;
    G_SetEnemy(self_, other);
    if (*self_).delay != 0 {
        //delay before firing scriptrunner
        (*self_).e_ThinkFunc = thinkF_scriptrunner_run;
        (*self_).nextthink = level.time + (*self_).delay;
    } else {
        scriptrunner_run(self_);
    }
}

/*QUAKED target_scriptrunner (1 0 0) (-4 -4 -4) (4 4 4) runonactivator x x x x x x INACTIVE
--- SPAWNFLAGS ---
runonactivator - Will run the script on the entity that used this or tripped the trigger that used this
INACTIVE - start off

----- KEYS ------
Usescript - Script to run when used
count - how many times to run, -1 = infinite.  Default is once
wait - can't be used again in this amount of seconds (Default is 1 second if it's multiple-use)
delay - how long to wait after use to run script

*/
pub unsafe fn SP_target_scriptrunner(self_: *mut gentity_t) {
    if (*self_).behaviorSet[0].is_null() {
        gi.Printf("SP_target_scriptrunner %s has no USESCRIPT\n\0".as_ptr() as *const c_char, (*self_).targetname);
    }
    if ((*self_).spawnflags & 128) != 0 {
        (*self_).svFlags |= SVF_INACTIVE;
    }

    if (*self_).count == 0 {
        (*self_).count = 1;//default 1 use only
    }
    /*
    else if ( !self->wait )
    {
        self->wait = 1;//default wait of 1 sec
    }
    */
    // FIXME: this is a hack... because delay is read in as an int, so I'm bypassing that because it's too late in the project to change it and I want to be able to set less than a second delays
    // no one should be setting a radius on a scriptrunner, if they are this would be bad, take this out for the next project
    (*self_).radius = 0.0;
    G_SpawnFloat("delay\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, &mut (*self_).radius);
    (*self_).delay = ((*self_).radius * 1000.0) as c_int;//sec to ms
    (*self_).wait *= 1000.0;//sec to ms

    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    (*self_).e_UseFunc = useF_target_scriptrunner_use;
}

pub unsafe fn target_gravity_change_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(self_, BSET_USE);

    if ((*self_).spawnflags & 1) != 0 {
        gi.cvar_set("g_gravity\0".as_ptr() as *const c_char, va("%f\0".as_ptr() as *const c_char, (*self_).speed));
    } else if !(*activator).client.is_null() {
        let grav: c_int = (*self_).speed.floor() as c_int;
        /*
        if ( activator->client->ps.gravity != grav )
        {
            gi.Printf("%s gravity changed to %d\n", activator->targetname, grav );
        }
        */
        (*(*activator).client).ps.gravity = grav;
        (*activator).svFlags |= SVF_CUSTOM_GRAVITY;
        //FIXME: need a way to set this back to normal?
    }
}

/*QUAKED target_gravity_change (1 0 0) (-4 -4 -4) (4 4 4) GLOBAL

"gravity" - Normal = 800, Valid range: any

GLOBAL - Apply to entire world, not just the activator
*/
pub unsafe fn SP_target_gravity_change(self_: *mut gentity_t) {
    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    G_SpawnFloat("gravity\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char, &mut (*self_).speed);
    (*self_).e_UseFunc = useF_target_gravity_change_use;
}

pub unsafe fn target_friction_change_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(self_, BSET_USE);

    if ((*self_).spawnflags & 1) != 0 {
        //FIXME - make a global?
        //gi.Cvar_Set("g_friction", va("%d", self->health));
    } else if !(*activator).client.is_null() {
        (*(*activator).client).ps.friction = (*self_).health;
    }
}

/*QUAKED target_friction_change (1 0 0) (-4 -4 -4) (4 4 4)

"friction" Normal = 6, Valid range 0 - 10

*/
pub unsafe fn SP_target_friction_change(self_: *mut gentity_t) {
    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    (*self_).e_UseFunc = useF_target_friction_change_use;
}

pub unsafe fn set_mission_stats_cvars() {
    let mut text: [c_char; 1024] = [0; 1024];

    //we'll assume that the activator is the player
    let client: *const gclient_t = addr_of!(level.clients[0]);

    if client.is_null() {
        return;
    }
    (*cg_entities.as_ptr().add(0) as *mut gentity_t).as_ref().unwrap().client.as_ref().unwrap().sess.missionStats.enemiesKilled;

    gi.cvar_set("ui_stats_enemieskilled\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.enemiesKilled));	//pass this on to the menu

    if (*(*cg_entities.as_ptr().add(0) as *mut gentity_t).client).sess.missionStats.totalSecrets != 0 {
        cgi_SP_GetStringTextString("SP_INGAME_SECRETAREAS_OF\0".as_ptr() as *const c_char, text.as_mut_ptr(), 1024);
        gi.cvar_set("ui_stats_secretsfound\0".as_ptr() as *const c_char, va("%d %s %d\0".as_ptr() as *const c_char,
            (*(*cg_entities.as_ptr().add(0) as *mut gentity_t).client).sess.missionStats.secretsFound,
            text.as_ptr(),
            (*(*cg_entities.as_ptr().add(0) as *mut gentity_t).client).sess.missionStats.totalSecrets));
    } else	{
        // Setting ui_stats_secretsfound to 0 will hide the text on screen
        gi.cvar_set("ui_stats_secretsfound\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char);
    }

    // Find the favorite weapon
    let mut wpn: c_int = 0;
    let mut i: c_int;
    let mut max_wpn: c_int = (*(*cg_entities.as_ptr().add(0) as *mut gentity_t).client).sess.missionStats.weaponUsed[0];
    i = 1;
    while i < WP_NUM_WEAPONS {
        if (*(*cg_entities.as_ptr().add(0) as *mut gentity_t).client).sess.missionStats.weaponUsed[i as usize] > max_wpn {
            max_wpn = (*(*cg_entities.as_ptr().add(0) as *mut gentity_t).client).sess.missionStats.weaponUsed[i as usize];
            wpn = i;
        }
        i += 1;
    }

    if wpn != 0 {
        let wItem: *mut gitem_t = FindItemForWeapon(wpn);
        cgi_SP_GetStringTextString(va("SP_INGAME_%s\0".as_ptr() as *const c_char, (*wItem).classname), text.as_mut_ptr(), 1024);
        gi.cvar_set("ui_stats_fave\0".as_ptr() as *const c_char, va("%s\0".as_ptr() as *const c_char, text.as_ptr()));	//pass this on to the menu
    }

    gi.cvar_set("ui_stats_shots\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.shotsFired));				//pass this on to the menu

    gi.cvar_set("ui_stats_hits\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.hits));						//pass this on to the menu

    let percent: f32 = if (*client).sess.missionStats.shotsFired != 0 {
        100.0 * (*client).sess.missionStats.hits as f32 / (*client).sess.missionStats.shotsFired as f32
    } else {
        0.0
    };
    gi.cvar_set("ui_stats_accuracy\0".as_ptr() as *const c_char, va("%.2f%%\0".as_ptr() as *const c_char, percent));						//pass this on to the menu

    gi.cvar_set("ui_stats_thrown\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.saberThrownCnt));						//pass this on to the menu

    gi.cvar_set("ui_stats_blocks\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.saberBlocksCnt));
    gi.cvar_set("ui_stats_legattacks\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.legAttacksCnt));
    gi.cvar_set("ui_stats_armattacks\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.armAttacksCnt));
    gi.cvar_set("ui_stats_bodyattacks\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.torsoAttacksCnt));

    gi.cvar_set("ui_stats_absorb\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_ABSORB as usize]));
    gi.cvar_set("ui_stats_heal\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_HEAL as usize]));
    gi.cvar_set("ui_stats_mindtrick\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_TELEPATHY as usize]));
    gi.cvar_set("ui_stats_protect\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_PROTECT as usize]));

    gi.cvar_set("ui_stats_jump\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_LEVITATION as usize]));
    gi.cvar_set("ui_stats_pull\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_PULL as usize]));
    gi.cvar_set("ui_stats_push\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_PUSH as usize]));
    gi.cvar_set("ui_stats_sense\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_SEE as usize]));
    gi.cvar_set("ui_stats_speed\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_SPEED as usize]));
    gi.cvar_set("ui_stats_defense\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_SABER_DEFENSE as usize]));
    gi.cvar_set("ui_stats_offense\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_SABER_OFFENSE as usize]));
    gi.cvar_set("ui_stats_throw\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_SABERTHROW as usize]));

    gi.cvar_set("ui_stats_drain\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_DRAIN as usize]));
    gi.cvar_set("ui_stats_grip\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_GRIP as usize]));
    gi.cvar_set("ui_stats_lightning\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_LIGHTNING as usize]));
    gi.cvar_set("ui_stats_rage\0".as_ptr() as *const c_char, va("%d\0".as_ptr() as *const c_char, (*client).sess.missionStats.forceUsed[FP_RAGE as usize]));

}

// #include "..\cgame\cg_media.h"	//access to cgs
extern "C" {
    pub fn G_ChangeMap(mapname: *const c_char, spawntarget: *const c_char, hub: c_int);	//g_utils
}
pub unsafe fn target_level_change_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(self_, BSET_USE);

    if !(*self_).message.is_null() && Q_stricmp("disconnect\0".as_ptr() as *const c_char, (*self_).message) == 0 {
        gi.SendConsoleCommand("disconnect\n\0".as_ptr() as *const c_char);
    } else {
        G_ChangeMap((*self_).message, (*self_).target, (if ((*self_).spawnflags & 1) != 0 { 1 } else { 0 }));
    }
    if (*self_).count >= 0 {
        gi.cvar_set("tier_storyinfo\0".as_ptr() as *const c_char, va("%i\0".as_ptr() as *const c_char, (*self_).count));
        if level.mapname[0] == 't' as u8 as c_char && level.mapname[2] == '_' as u8 as c_char
            && ((level.mapname[1] == '1' as u8 as c_char) || (level.mapname[1] == '2' as u8 as c_char) || (level.mapname[1] == '3' as u8 as c_char)) {
            let mut s: [c_char; 2048] = [0; 2048];
            gi.Cvar_VariableStringBuffer("tiers_complete\0".as_ptr() as *const c_char, s.as_mut_ptr(), 2048);	//get the current list
            if s[0] != 0 {
                gi.cvar_set("tiers_complete\0".as_ptr() as *const c_char, va("%s %s\0".as_ptr() as *const c_char, s.as_ptr(), level.mapname.as_ptr()));	//strcat this level into the existing list
            } else {
                gi.cvar_set("tiers_complete\0".as_ptr() as *const c_char, level.mapname.as_ptr());	//set this level into the list
            }
        }
        if (*self_).noise_index != 0 {
            cgi_S_StopSounds();
            cgi_S_StartSound(null_mut(), 0, CHAN_VOICE, (*cgs.sound_precache.add((*self_).noise_index as usize)));
        }
    }

    set_mission_stats_cvars();

}

/*QUAKED target_level_change (1 0 0) (-4 -4 -4) (4 4 4) HUB NO_STORYSOUND
HUB - Will save the current map's status and load the next map with any saved status it may have
NO_STORYSOUND - will not play storyinfo wav file, even if you '++' or set tier_storyinfo

"mapname" - Name of map to change to or "+menuname" to launch a menu instead
"target" - Name of spawnpoint to start at in the new map
"tier_storyinfo" - integer to set cvar or "++" to just increment
"storyhead"	 - which head to show on menu [luke, kyle, or prot]
"saber_menu" - integer to set cvar for menu
"weapon_menu" - integer to set cvar for ingame weapon menu
*/
pub unsafe fn SP_target_level_change(self_: *mut gentity_t) {
    if (*self_).message.is_null() {
        G_Error("target_level_change with no mapname!\n\0".as_ptr() as *const c_char);
        return;
    }

    let mut s: *mut c_char;
    if G_SpawnString("tier_storyinfo\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, &mut s) != 0 {
        if *s as c_int == '+' as c_int {
            (*self_).noise_index = G_SoundIndex(va("sound/chars/tiervictory/%s.mp3\0".as_ptr() as *const c_char, level.mapname.as_ptr()));
            (*self_).count = gi.Cvar_VariableIntegerValue("tier_storyinfo\0".as_ptr() as *const c_char) + 1;
            G_SoundIndex(va("sound/chars/storyinfo/%d.mp3\0".as_ptr() as *const c_char, (*self_).count));	//cache for menu
        } else {
            (*self_).count = atoi(s);
            if ((*self_).spawnflags & 2) == 0 {
                (*self_).noise_index = G_SoundIndex(va("sound/chars/storyinfo/%d.mp3\0".as_ptr() as *const c_char, (*self_).count));
            }
        }

        if G_SpawnString("storyhead\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, &mut s) != 0 {
            {	//[luke, kyle, or prot]
                gi.cvar_set("storyhead\0".as_ptr() as *const c_char, s);	//pass this on to the menu
            }
        } else {
            {	//show head based on mapname
                gi.cvar_set("storyhead\0".as_ptr() as *const c_char, level.mapname.as_ptr());	//pass this on to the menu
            }
        }
    }
    if G_SpawnString("saber_menu\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, &mut s) != 0 {
        gi.cvar_set("saber_menu\0".as_ptr() as *const c_char, s);	//pass this on to the menu
    }

    if G_SpawnString("weapon_menu\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char, &mut s) != 0 {
        gi.cvar_set("weapon_menu\0".as_ptr() as *const c_char, s);	//pass this on to the menu
    } else {
        gi.cvar_set("weapon_menu\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char);	//pass this on to the menu
    }

    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    (*self_).e_UseFunc = useF_target_level_change_use;
}

/*QUAKED target_change_parm (1 0 0) (-4 -4 -4) (4 4 4)
Copies any parms set on this ent to the entity that  fired the trigger/button/whatever that uses this
parm1
parm2
parm3
parm4
parm5
parm6
parm7
parm8
parm9
parm10
parm11
parm12
parm13
parm14
parm15
parm16
*/
extern "C" {
    pub fn Q3_SetParm(entID: c_int, parmNum: c_int, parmValue: *const c_char);
}
pub unsafe fn target_change_parm_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if activator.is_null() || self_.is_null() {
        return;
    }

    //FIXME: call capyparms
    if !(*self_).parms.is_null() {
        let mut parmNum: c_int = 0;
        while parmNum < MAX_PARMS {
            if !(*(*self_).parms).parm[parmNum as usize].is_null() && *(*(*self_).parms).parm[parmNum as usize] as c_int != 0 {
                Q3_SetParm((*activator).s.number, parmNum, (*(*self_).parms).parm[parmNum as usize]);
            }
            parmNum += 1;
        }
    }
}

pub unsafe fn SP_target_change_parm(self_: *mut gentity_t) {
    if (*self_).parms.is_null() {
        //ERROR!
        return;
    }
    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    (*self_).e_UseFunc = useF_target_change_parm_use;
}

pub unsafe fn target_play_music_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(self_, BSET_USE);
    gi.SetConfigstring(CS_MUSIC, (*self_).message);
}

/*QUAKED target_play_music (1 0 0) (-4 -4 -4) (4 4 4)
target_play_music
Plays the requested music files when this target is used.

"targetname"
"music"		music WAV or MP3 file ( music/introfile.mp3 [optional]  music/loopfile.mp3 )

If an intro file and loop file are specified, the intro plays first, then the looping
portion will start and loop indefinetly.  If no introfile is entered, only the loopfile
will play.
*/
pub unsafe fn SP_target_play_music(self_: *mut gentity_t) {
    let mut s: *mut c_char;
    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    if G_SpawnString("music\0".as_ptr() as *const c_char, "\0".as_ptr() as *const c_char, &mut s) == 0 {
        G_Error("target_play_music without a music key at %s\0".as_ptr() as *const c_char, vtos((*self_).s.origin.as_ptr()));
    }
    (*self_).message = G_NewString(s);
    (*self_).e_UseFunc = useF_target_play_music_use;
    //Precache!
    if (*com_buildScript).integer != 0 {//copy this puppy over
        let mut buffer: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut hFile: fileHandle_t;

        Q_strncpyz(buffer.as_mut_ptr(), s, MAX_QPATH as c_int);
        COM_DefaultExtension(buffer.as_mut_ptr(), MAX_QPATH as c_int, ".mp3\0".as_ptr() as *const c_char);

        gi.FS_FOpenFile(buffer.as_ptr(), &mut hFile, FS_READ);
        if hFile != 0 {
            gi.FS_FCloseFile(hFile);
        }
    }
}

pub unsafe fn target_autosave_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    G_ActivateBehavior(self_, BSET_USE);
    //gi.SendServerCommand( NULL, "cp @SP_INGAME_CHECKPOINT" );
    CG_CenterPrint("@SP_INGAME_CHECKPOINT\0".as_ptr() as *const c_char, SCREEN_HEIGHT * 0.25);	//jump the network

    gi.SendConsoleCommand("wait 2;save auto\n\0".as_ptr() as *const c_char);
}

/*QUAKED target_autosave (1 0 0) (-4 -4 -4) (4 4 4)
Auto save the game in two frames.
Make sure it won't trigger during dialogue or cinematic or it will stutter!
*/
pub unsafe fn SP_target_autosave(self_: *mut gentity_t) {
    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    (*self_).e_UseFunc = useF_target_autosave_use;
}

pub unsafe fn target_secret_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    // we'll assume that the activator is the player
    let client: *mut gclient_t = addr_of_mut!(level.clients[0]) as *mut gclient_t;
    (*client).sess.missionStats.secretsFound += 1;
    if !activator.is_null() {
        G_Sound(activator, (*self_).noise_index);
    } else {
        G_Sound(self_, (*self_).noise_index);
    }
    gi.SendServerCommand(null_mut(), "cp @SP_INGAME_SECRET_AREA\0".as_ptr() as *const c_char);
    assert!((*client).sess.missionStats.totalSecrets != 0);
}

/*QUAKED target_secret (1 0 1) (-4 -4 -4) (4 4 4)
You found a Secret!
"count" - how many secrets on this level,
          if more than one on a level, be sure they all have the same count!
*/
pub unsafe fn SP_target_secret(self_: *mut gentity_t) {
    G_SetOrigin(self_, (*self_).s.origin.as_ptr());
    (*self_).e_UseFunc = useF_target_secret_use;
    (*self_).noise_index = G_SoundIndex("sound/interface/secret_area\0".as_ptr() as *const c_char);
    if (*self_).count != 0 {
        gi.cvar_set("newTotalSecrets\0".as_ptr() as *const c_char, va("%i\0".as_ptr() as *const c_char, (*self_).count));
    }
}

// Additional extern declarations needed for various functions
extern "C" {
    pub fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventParm: c_int);
    pub fn G_NewString(string: *const c_char) -> *mut c_char;
    pub fn vtos(v: *const f32) -> *const c_char;
    pub fn atoi(str: *const c_char) -> c_int;
}
