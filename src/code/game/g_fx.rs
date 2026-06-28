// leave this line at the top for all g_xxxx.cpp files...

use core::ffi::{c_char, c_int, c_void};
use core::ptr::addr_of_mut;

// Declare external functions from the engine/other modules
extern "C" {
    fn G_FindConfigstringIndex(
        name: *const c_char,
        start: c_int,
        max: c_int,
        create: c_int,
    ) -> c_int;
    fn EvaluateTrajectory(
        traj: *const c_void,
        time: c_int,
        result: *mut c_void,
    );
    fn G_AddEvent(ent: *mut c_void, event: c_int, eventParm: c_int);
    fn AngleVectors(
        angles: *const c_void,
        forward: *mut c_void,
        right: *mut c_void,
        up: *mut c_void,
    );
    fn MakeNormalVectors(
        forward: *const c_void,
        right: *mut c_void,
        up: *mut c_void,
    );
    fn G_RadiusDamage(
        origin: *const c_void,
        inflictor: *mut c_void,
        damage: c_int,
        radius: f32,
        attacker: *mut c_void,
        mod_: c_int,
    );
    fn G_UseTargets2(
        ent: *mut c_void,
        activator: *mut c_void,
        target: *const c_char,
    );
    fn CAS_GetBModelSound(
        soundSet: *const c_char,
        index: c_int,
    ) -> c_int;
    fn G_Find(
        from: *mut c_void,
        fieldofs: c_int,
        match_: *const c_char,
    ) -> *mut c_void;
    fn FOFS(field: *const c_char) -> c_int;
    fn Com_Printf(format: *const c_char, ...);
    fn VectorSubtract(a: *const c_void, b: *const c_void, result: *mut c_void);
    fn VectorNormalize(v: *mut c_void) -> f32;
    fn vectoangles(vec: *const c_void, angles: *mut c_void);
    fn G_SetAngles(ent: *mut c_void, angles: *const c_void);
    fn G_SpawnInt(key: *const c_char, def: *const c_char, value: *mut c_int) -> c_int;
    fn G_SpawnFloat(
        key: *const c_char,
        def: *const c_char,
        value: *mut f32,
    ) -> c_int;
    fn G_SpawnAngleHack(
        key: *const c_char,
        def: *const c_char,
        value: *mut c_void,
    ) -> c_int;
    fn VectorSet(v: *mut c_void, x: f32, y: f32, z: f32);
    fn G_EffectIndex(file: *const c_char) -> c_int;
    fn G_FreeEntity(ent: *mut c_void);
    fn G_SetOrigin(ent: *mut c_void, origin: *const c_void);
    fn VectorScale(v: *const c_void, scale: f32, result: *mut c_void);
    fn gi_linkentity(ent: *mut c_void);
    fn gi_cvar(var_name: *const c_char, def: *const c_char, flags: c_int) -> *const c_void;
    fn gi_Printf(format: *const c_char, ...);
    fn vtos(v: *const c_void) -> *const c_char;
    fn random() -> f32;
    fn Q_irand(min: c_int, max: c_int) -> c_int;
    fn Q_flrand(min: f32, max: f32) -> f32;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn G_SpawnString(
        key: *const c_char,
        def: *const c_char,
        value: *mut *const c_char,
    ) -> c_int;
    fn G_Spawn() -> *mut c_void;
    fn G_SetEnemy(ent: *mut c_void, enemy: *mut c_void);
    fn G_ModelIndex(model: *const c_char) -> c_int;
    fn G_SoundIndex(sound: *const c_char) -> c_int;
    fn G_Sound(ent: *mut c_void, soundIndex: c_int);
    fn G_PlayEffect(effect: *const c_char, origin: *const c_void, angles: *const c_void);
    fn G_SoundAtSpot(org: *const c_void, soundIndex: c_int, broadcast: c_int);
    fn gi_trace(
        results: *mut c_void,
        start: *const c_void,
        mins: *const c_void,
        maxs: *const c_void,
        end: *const c_void,
        passent: c_int,
        contentmask: c_int,
        g2pass: c_int,
        chunkSize: c_int,
    );
    fn G_SpawnField(index: c_int, key: *mut *const c_char, value: *mut *const c_char) -> c_int;
    fn G_NewString(s: *const c_char) -> *const c_char;
    fn G_Damage(
        target: *mut c_void,
        inflictor: *mut c_void,
        attacker: *mut c_void,
        dir: *const c_void,
        point: *const c_void,
        damage: c_int,
        flags: c_int,
        mod_: c_int,
    );
    fn gi_SetBrushModel(ent: *mut c_void, model: *const c_char);
    fn va(format: *const c_char, ...) -> *const c_char;
    fn gi_WE_SetTempGlobalFogColor(color: *const c_void);
    fn gi_WE_IsOutside(pos: *const c_void) -> c_int;
    fn VectorClear(v: *mut c_void);
    fn VectorMA(v1: *const c_void, scale: f32, v2: *const c_void, result: *mut c_void);
    fn VectorCopy(src: *const c_void, dst: *mut c_void);
}

// Declare global engine references
extern "C" {
    static mut level: c_void;
    static mut player: *mut c_void;
    static mut g_entities: *mut c_void;
    static BMS_START: c_int;
    static BMS_MID: c_int;
    static BMS_END: c_int;
}

const FX_ENT_RADIUS: c_int = 32;
const FX_RUNNER_RESERVED: c_int = 0x800000;

// QUAKED fx_runner (0 0 1) (-8 -8 -8) (8 8 8) STARTOFF ONESHOT DAMAGE
// Runs the specified effect, can also be targeted at an info_notnull to orient the effect
//
//	STARTOFF - effect starts off, toggles on/off when used
//	ONESHOT - effect fires only when used
//	DAMAGE - does radius damage around effect every "delay" milliseonds
//
//	"fxFile" - name of the effect file to play
//	"target" - direction to aim the effect in, otherwise defaults to up
//	"target2" - uses its target2 when the fx gets triggered
//	"delay"  - how often to call the effect, don't over-do this ( default 200 )
//	"random" - random amount of time to add to delay, ( default 0, 200 = 0ms to 200ms )
//	"splashRadius" - only works when damage is checked ( default 16 )
//	"splashDamage" - only works when damage is checked ( default 5 )
//	"soundset"	- bmodel set to use, plays start sound when toggled on, loop sound while on ( doesn't play on a oneshot), and a stop sound when turned off

// ----------------------------------------------------------

#[no_mangle]
pub extern "C" fn fx_runner_think(ent: *mut c_void) {
    unsafe {
        let mut temp: [f32; 3] = [0.0; 3];

        EvaluateTrajectory((*(&level as *const c_void as *const gentity_pos_s)).pos as *const c_void as *const c_void, (*(&level as *const c_void as *const gentity_level_s)).time, (*ent as *mut gentity_t).currentOrigin.as_mut_ptr() as *mut c_void);
        EvaluateTrajectory((*(&level as *const c_void as *const gentity_pos_s)).apos as *const c_void as *const c_void, (*(&level as *const c_void as *const gentity_level_s)).time, (*ent as *mut gentity_t).currentAngles.as_mut_ptr() as *mut c_void);

        // call the effect with the desired position and orientation
        G_AddEvent(ent, 16, (*ent as *mut gentity_t).fxID);

        // Assume angles, we'll do a cross product on the other end to finish up
        AngleVectors((*ent as *mut gentity_t).currentAngles.as_ptr() as *const c_void, (*ent as *mut gentity_t).pos3.as_mut_ptr() as *mut c_void, core::ptr::null_mut(), core::ptr::null_mut());
        MakeNormalVectors((*ent as *mut gentity_t).pos3.as_ptr() as *const c_void, (*ent as *mut gentity_t).pos4.as_mut_ptr() as *mut c_void, temp.as_mut_ptr() as *mut c_void); // there IS a reason this is done...it's so that it doesn't break every effect in the game...

        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + (*ent as *mut gentity_t).delay + (random() * (*ent as *mut gentity_t).random) as i32;

        if ((*ent as *mut gentity_t).spawnflags & 4) != 0 {
            // damage
            G_RadiusDamage((*ent as *mut gentity_t).currentOrigin.as_ptr() as *const c_void, ent, (*ent as *mut gentity_t).splashDamage, (*ent as *mut gentity_t).splashRadius, ent, 13);
        }

        if !(*ent as *mut gentity_t).target2.is_null() {
            // let our target know that we have spawned an effect
            G_UseTargets2(ent, ent, (*ent as *mut gentity_t).target2);
        }

        if ((*ent as *mut gentity_t).spawnflags & 2) == 0 && (*ent as *mut gentity_t).s.loopSound == 0 {
            // NOT ONESHOT...this is an assy thing to do
            if !(*ent as *mut gentity_t).soundSet.is_null() && !(*(*ent as *mut gentity_t).soundSet).is_null() {
                let loop_sound = CAS_GetBModelSound((*ent as *mut gentity_t).soundSet, BMS_MID);
                (*ent as *mut gentity_t).s.loopSound = if loop_sound < 0 { 0 } else { loop_sound };
            }
        }
    }
}

// ----------------------------------------------------------

#[no_mangle]
pub extern "C" fn fx_runner_use(self_: *mut c_void, other: *mut c_void, activator: *mut c_void) {
    unsafe {
        if ((*self_ as *mut gentity_t).s.isPortalEnt) != 0 {
            //rww - mark it as broadcast upon first use if it's within the area of a skyportal
            (*self_ as *mut gentity_t).svFlags |= 32;
        }

        if ((*self_ as *mut gentity_t).spawnflags & 2) != 0 {
            // ONESHOT
            // call the effect with the desired position and orientation, as a safety thing,
            //	make sure we aren't thinking at all.
            fx_runner_think(self_);
            (*self_ as *mut gentity_t).nextthink = -1;

            if !(*self_ as *mut gentity_t).target2.is_null() {
                // let our target know that we have spawned an effect
                G_UseTargets2(self_, self_, (*self_ as *mut gentity_t).target2);
            }

            if !(*self_ as *mut gentity_t).soundSet.is_null() && !(*(*self_ as *mut gentity_t).soundSet).is_null() {
                G_AddEvent(self_, 12, CAS_GetBModelSound((*self_ as *mut gentity_t).soundSet, BMS_START));
            }
        } else {
            // ensure we are working with the right think function
            (*self_ as *mut gentity_t).e_ThinkFunc = 1; // thinkF_fx_runner_think

            // toggle our state
            if (*self_ as *mut gentity_t).nextthink == -1 {
                // NOTE: we fire the effect immediately on use, the fx_runner_think func will set
                //	up the nextthink time.
                fx_runner_think(self_);

                if !(*self_ as *mut gentity_t).soundSet.is_null() && !(*(*self_ as *mut gentity_t).soundSet).is_null() {
                    G_AddEvent(self_, 12, CAS_GetBModelSound((*self_ as *mut gentity_t).soundSet, BMS_START));
                    let loop_sound = CAS_GetBModelSound((*self_ as *mut gentity_t).soundSet, BMS_MID);
                    (*self_ as *mut gentity_t).s.loopSound = if loop_sound < 0 { 0 } else { loop_sound };
                }
            } else {
                // turn off for now
                (*self_ as *mut gentity_t).nextthink = -1;

                if !(*self_ as *mut gentity_t).soundSet.is_null() && !(*(*self_ as *mut gentity_t).soundSet).is_null() {
                    G_AddEvent(self_, 12, CAS_GetBModelSound((*self_ as *mut gentity_t).soundSet, BMS_END));
                    (*self_ as *mut gentity_t).s.loopSound = 0;
                }
            }
        }
    }
}

// ----------------------------------------------------------

#[no_mangle]
pub extern "C" fn fx_runner_link(ent: *mut c_void) {
    unsafe {
        let mut dir: [f32; 3] = [0.0; 3];

        if !(*ent as *mut gentity_t).target.is_null() {
            // try to use the target to override the orientation
            let mut target: *mut gentity_t = core::ptr::null_mut();

            target = G_Find(target as *mut c_void, FOFS(b"targetname\0".as_ptr() as *const c_char), (*ent as *mut gentity_t).target) as *mut gentity_t;

            if target.is_null() {
                // Bah, no good, dump a warning, but continue on and use the UP vector
                Com_Printf(b"fx_runner_link: target specified but not found: %s\n\0".as_ptr() as *const c_char, (*ent as *mut gentity_t).target);
                Com_Printf(b"  -assuming UP orientation.\n\0".as_ptr() as *const c_char);
            } else {
                // Our target is valid so let's override the default UP vector
                VectorSubtract((*target).s.origin.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void, dir.as_mut_ptr() as *mut c_void);
                VectorNormalize(dir.as_mut_ptr() as *mut c_void);
                vectoangles(dir.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.angles.as_mut_ptr() as *mut c_void);
            }
        }

        // don't really do anything with this right now other than do a check to warn the designers if the target2 is bogus
        if !(*ent as *mut gentity_t).target2.is_null() {
            let mut target: *mut gentity_t = core::ptr::null_mut();

            target = G_Find(target as *mut c_void, FOFS(b"targetname\0".as_ptr() as *const c_char), (*ent as *mut gentity_t).target2) as *mut gentity_t;

            if target.is_null() {
                // Target2 is bogus, but we can still continue
                Com_Printf(b"fx_runner_link: target2 was specified but is not valid: %s\n\0".as_ptr() as *const c_char, (*ent as *mut gentity_t).target2);
            }
        }

        G_SetAngles(ent, (*ent as *mut gentity_t).s.angles.as_ptr() as *const c_void);

        if ((*ent as *mut gentity_t).spawnflags & 1) != 0 || ((*ent as *mut gentity_t).spawnflags & 2) != 0 {
            // STARTOFF || ONESHOT
            // We won't even consider thinking until we are used
            (*ent as *mut gentity_t).nextthink = -1;
        } else {
            if !(*ent as *mut gentity_t).soundSet.is_null() && !(*(*ent as *mut gentity_t).soundSet).is_null() {
                let loop_sound = CAS_GetBModelSound((*ent as *mut gentity_t).soundSet, BMS_MID);
                (*ent as *mut gentity_t).s.loopSound = if loop_sound < 0 { 0 } else { loop_sound };
            }

            // Let's get to work right now!
            (*ent as *mut gentity_t).e_ThinkFunc = 1; // thinkF_fx_runner_think
            (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 200; // wait a small bit, then start working
        }

        // make us useable if we can be targeted
        if !(*ent as *mut gentity_t).targetname.is_null() {
            (*ent as *mut gentity_t).e_UseFunc = 1; // useF_fx_runner_use
        }
    }
}

// ----------------------------------------------------------

#[no_mangle]
pub extern "C" fn SP_fx_runner(ent: *mut c_void) {
    unsafe {
        // Get our defaults
        G_SpawnInt(b"delay\0".as_ptr() as *const c_char, b"200\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).delay);
        G_SpawnFloat(b"random\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).random);
        G_SpawnInt(b"splashRadius\0".as_ptr() as *const c_char, b"16\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).splashRadius);
        G_SpawnInt(b"splashDamage\0".as_ptr() as *const c_char, b"5\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).splashDamage);

        if G_SpawnAngleHack(b"angle\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, (*ent as *mut gentity_t).s.angles.as_mut_ptr() as *mut c_void) == 0 {
            // didn't have angles, so give us the default of up
            VectorSet((*ent as *mut gentity_t).s.angles.as_mut_ptr() as *mut c_void, -90.0, 0.0, 0.0);
        }

        if (*ent as *mut gentity_t).fxFile.is_null() {
            gi_Printf(b"\x1b[31mERROR: fx_runner %s at %s has no fxFile specified\n\0".as_ptr() as *const c_char, (*ent as *mut gentity_t).targetname, vtos((*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void));
            G_FreeEntity(ent);
            return;
        }

        // Try and associate an effect file, unfortunately we won't know if this worked or not
        //	until the CGAME trys to register it...
        (*ent as *mut gentity_t).fxID = G_EffectIndex((*ent as *mut gentity_t).fxFile);

        (*ent as *mut gentity_t).s.eType = 3; // ET_MOVER

        // Give us a bit of time to spawn in the other entities, since we may have to target one of 'em
        (*ent as *mut gentity_t).e_ThinkFunc = 2; // thinkF_fx_runner_link
        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 400;

        // Save our position and link us up!
        G_SetOrigin(ent, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void);

        VectorSet((*ent as *mut gentity_t).maxs.as_mut_ptr() as *mut c_void, FX_ENT_RADIUS as f32, FX_ENT_RADIUS as f32, FX_ENT_RADIUS as f32);
        VectorScale((*ent as *mut gentity_t).maxs.as_ptr() as *const c_void, -1.0, (*ent as *mut gentity_t).mins.as_mut_ptr() as *mut c_void);

        gi_linkentity(ent);
    }
}

// QUAKED fx_snow (1 0 0) (-16 -16 -16) (16 16 16) LIGHT MEDIUM HEAVY MISTY_FOG
// This world effect will spawn snow globally into the level.

#[no_mangle]
pub extern "C" fn SP_CreateSnow(ent: *mut c_void) {
    unsafe {
        let r_weatherScale = gi_cvar(b"r_weatherScale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 2); // CVAR_ARCHIVE
        if (*(r_weatherScale as *const cvar_t)).value == 0.0 {
            return;
        }

        // Different Types Of Rain
        //-------------------------
        if ((*ent as *mut gentity_t).spawnflags & 1) != 0 {
            G_FindConfigstringIndex(b"lightsnow\0".as_ptr() as *const c_char, 10, 8, 1);
        } else if ((*ent as *mut gentity_t).spawnflags & 2) != 0 {
            G_FindConfigstringIndex(b"snow\0".as_ptr() as *const c_char, 10, 8, 1);
        } else if ((*ent as *mut gentity_t).spawnflags & 4) != 0 {
            G_FindConfigstringIndex(b"heavysnow\0".as_ptr() as *const c_char, 10, 8, 1);
        } else {
            G_FindConfigstringIndex(b"snow\0".as_ptr() as *const c_char, 10, 8, 1);
            G_FindConfigstringIndex(b"fog\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // MISTY FOG
        //===========
        if ((*ent as *mut gentity_t).spawnflags & 8) != 0 {
            G_FindConfigstringIndex(b"fog\0".as_ptr() as *const c_char, 10, 8, 1);
        }
    }
}

// QUAKED fx_wind (0 .5 .8) (-16 -16 -16) (16 16 16) NORMAL CONSTANT GUSTING SWIRLING x  FOG LIGHT_FOG
// Generates global wind forces
//
// NORMAL    creates a random light global wind
// CONSTANT  forces all wind to go in a specified direction
// GUSTING   causes random gusts of wind
// SWIRLING  causes random swirls of wind
//
// "angles" the direction for constant wind
// "speed"  the speed for constant wind

#[no_mangle]
pub extern "C" fn SP_CreateWind(ent: *mut c_void) {
    unsafe {
        let r_weatherScale = gi_cvar(b"r_weatherScale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 2); // CVAR_ARCHIVE
        if (*(r_weatherScale as *const cvar_t)).value <= 0.0 {
            return;
        }

        let mut temp: [c_char; 256] = [0; 256];

        // Normal Wind
        //-------------
        if ((*ent as *mut gentity_t).spawnflags & 1) != 0 {
            G_FindConfigstringIndex(b"wind\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // Constant Wind
        //---------------
        if ((*ent as *mut gentity_t).spawnflags & 2) != 0 {
            let mut windDir: [f32; 3] = [0.0; 3];
            AngleVectors((*ent as *mut gentity_t).s.angles.as_ptr() as *const c_void, windDir.as_mut_ptr() as *mut c_void, core::ptr::null_mut(), core::ptr::null_mut());
            G_SpawnFloat(b"speed\0".as_ptr() as *const c_char, b"500\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).speed);
            VectorScale(windDir.as_ptr() as *const c_void, (*ent as *mut gentity_t).speed, windDir.as_mut_ptr() as *mut c_void);

            // sprintf( temp, "constantwind ( %f %f %f )", windDir[0], windDir[1], windDir[2] );
            // G_FindConfigstringIndex(temp, CS_WORLD_FX, MAX_WORLD_FX, qtrue);
        }

        // Gusting Wind
        //--------------
        if ((*ent as *mut gentity_t).spawnflags & 4) != 0 {
            G_FindConfigstringIndex(b"gustingwind\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // Swirling Wind
        //---------------
        if ((*ent as *mut gentity_t).spawnflags & 8) != 0 {
            G_FindConfigstringIndex(b"swirlingwind\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // MISTY FOG
        //===========
        if ((*ent as *mut gentity_t).spawnflags & 32) != 0 {
            G_FindConfigstringIndex(b"fog\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // MISTY FOG
        //===========
        if ((*ent as *mut gentity_t).spawnflags & 64) != 0 {
            G_FindConfigstringIndex(b"light_fog\0".as_ptr() as *const c_char, 10, 8, 1);
        }
    }
}

// QUAKED fx_wind_zone (0 .5 .8) ?  Creates a constant wind in a local area
// Generates local wind forces
//
// "angles" the direction for constant wind
// "speed"  the speed for constant wind

#[no_mangle]
pub extern "C" fn SP_CreateWindZone(ent: *mut c_void) {
    unsafe {
        let r_weatherScale = gi_cvar(b"r_weatherScale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 2); // CVAR_ARCHIVE
        if (*(r_weatherScale as *const cvar_t)).value <= 0.0 {
            return;
        }

        gi_SetBrushModel(ent, (*ent as *mut gentity_t).model);

        let mut windDir: [f32; 3] = [0.0; 3];
        AngleVectors((*ent as *mut gentity_t).s.angles.as_ptr() as *const c_void, windDir.as_mut_ptr() as *mut c_void, core::ptr::null_mut(), core::ptr::null_mut());
        G_SpawnFloat(b"speed\0".as_ptr() as *const c_char, b"500\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).speed);
        VectorScale(windDir.as_ptr() as *const c_void, (*ent as *mut gentity_t).speed, windDir.as_mut_ptr() as *mut c_void);

        let mut temp: [c_char; 256] = [0; 256];
        // sprintf( temp, "windzone ( %f %f %f ) ( %f %f %f ) ( %f %f %f )",
        // 	ent->mins[0], ent->mins[1], ent->mins[2],
        // 	ent->maxs[0], ent->maxs[1], ent->maxs[2],
        // 	windDir[0],	  windDir[1],   windDir[2]
        // 	);
        // G_FindConfigstringIndex(temp, CS_WORLD_FX, MAX_WORLD_FX, qtrue);
    }
}

// QUAKED fx_rain (1 0 0) (-16 -16 -16) (16 16 16) LIGHT MEDIUM HEAVY ACID OUTSIDE_SHAKE MISTY_FOG
// This world effect will spawn rain globally into the level.
//
// LIGHT   create light drizzle
// MEDIUM  create average medium rain
// HEAVY   create heavy downpour (with fog and lightning automatically)
// ACID    create acid rain
//
// OUTSIDE_SHAKE  will cause the camera to shake slightly whenever outside
// MISTY_FOG      causes clouds of misty fog to float through the level
// LIGHTNING      causes random bursts of lightning and thunder in the level
//
// The following fields are for lightning:
// "flashcolor"    "200 200 200" (r g b) (values 0.0-255.0)
// "flashdelay"    "12000" maximum time delay between lightning strikes
// "chanceflicker" "2"  1 in 2 chance of flickering fog
// "chancesound"   "3"  1 in 3 chance of playing a sound
// "chanceeffect"  "4"  1 in 4 chance of playing the effect

//----------------------------------------------------------

#[no_mangle]
pub extern "C" fn fx_rain_think(ent: *mut c_void) {
    unsafe {
        if !player.is_null() {
            if (*ent as *mut gentity_t).count != 0 {
                (*ent as *mut gentity_t).count -= 1;
                if (*ent as *mut gentity_t).count == 0 || ((*ent as *mut gentity_t).count % 2) == 0 {
                    gi_WE_SetTempGlobalFogColor((*ent as *mut gentity_t).pos2.as_ptr() as *const c_void); // Turn Off
                    if (*ent as *mut gentity_t).count == 0 {
                        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(1000, 12000);
                    } else if (*ent as *mut gentity_t).count == 2 {
                        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(150, 450);
                    } else {
                        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(50, 150);
                    }
                } else {
                    gi_WE_SetTempGlobalFogColor((*ent as *mut gentity_t).pos3.as_ptr() as *const c_void); // Turn On
                    (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50;
                }
            } else if gi_WE_IsOutside((*(*player)).currentOrigin.as_ptr() as *const c_void) != 0 {
                let mut effectPos: [f32; 3] = [0.0; 3];
                let mut effectDir: [f32; 3] = [0.0; 3];
                VectorClear(effectDir.as_mut_ptr() as *mut c_void);
                effectDir[0] += Q_flrand(-1.0, 1.0);
                effectDir[1] += Q_flrand(-1.0, 1.0);

                let playEffect: c_int = if Q_irand(1, (*ent as *mut gentity_t).aimDebounceTime) == 1 { 1 } else { 0 };
                let playFlicker: c_int = if Q_irand(1, (*ent as *mut gentity_t).attackDebounceTime) == 1 { 1 } else { 0 };
                let playSound: c_int = if (playEffect != 0) || (playFlicker != 0) || Q_irand(1, (*ent as *mut gentity_t).pushDebounceTime) == 1 { 1 } else { 0 };

                // Play The Sound
                //----------------
                if (playSound != 0) && (playEffect == 0) {
                    VectorMA((*(*player)).currentOrigin.as_ptr() as *const c_void, 250.0, effectDir.as_ptr() as *const c_void, effectPos.as_mut_ptr() as *mut c_void);
                    G_SoundAtSpot(effectPos.as_ptr() as *const c_void, G_SoundIndex(va(b"sound/ambience/thunder%d\0".as_ptr() as *const c_char, Q_irand(1, 4))), 1);
                }

                // Play The Effect
                //-----------------
                if playEffect != 0 {
                    VectorMA((*(*player)).currentOrigin.as_ptr() as *const c_void, 400.0, effectDir.as_ptr() as *const c_void, effectPos.as_mut_ptr() as *mut c_void);
                    if playSound != 0 {
                        G_Sound(*player, G_SoundIndex(va(b"sound/ambience/thunder_close%d\0".as_ptr() as *const c_char, Q_irand(1, 2))));
                    }

                    // Raise It Up Into The Sky
                    //--------------------------
                    effectPos[2] += Q_flrand(600.0, 1000.0);

                    VectorClear(effectDir.as_mut_ptr() as *mut c_void);
                    effectDir[2] = -1.0;

                    G_PlayEffect(b"env/huge_lightning\0".as_ptr() as *const c_char, effectPos.as_ptr() as *const c_void, effectDir.as_ptr() as *const c_void);
                    (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(100, 200);
                }

                // Change The Fog Color
                //----------------------
                if playFlicker != 0 {
                    (*ent as *mut gentity_t).count = Q_irand(1, 4) * 2;
                    (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50;
                    gi_WE_SetTempGlobalFogColor((*ent as *mut gentity_t).pos3.as_ptr() as *const c_void);
                } else {
                    (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(1000, (*ent as *mut gentity_t).delay);
                }
            } else {
                (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(1000, (*ent as *mut gentity_t).delay);
            }
        } else {
            (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(1000, (*ent as *mut gentity_t).delay);
        }
    }
}

#[no_mangle]
pub extern "C" fn SP_CreateRain(ent: *mut c_void) {
    unsafe {
        // Different Types Of Rain
        //-------------------------
        if ((*ent as *mut gentity_t).spawnflags & 1) != 0 {
            G_FindConfigstringIndex(b"lightrain\0".as_ptr() as *const c_char, 10, 8, 1);
        } else if ((*ent as *mut gentity_t).spawnflags & 2) != 0 {
            G_FindConfigstringIndex(b"rain\0".as_ptr() as *const c_char, 10, 8, 1);
        } else if ((*ent as *mut gentity_t).spawnflags & 4) != 0 {
            G_FindConfigstringIndex(b"heavyrain\0".as_ptr() as *const c_char, 10, 8, 1);

            // Automatically Get Heavy Fog
            //-----------------------------
            G_FindConfigstringIndex(b"heavyrainfog\0".as_ptr() as *const c_char, 10, 8, 1);

            // Automatically Get Lightning & Thunder
            //---------------------------------------
            (*ent as *mut gentity_t).spawnflags |= 64;
        } else if ((*ent as *mut gentity_t).spawnflags & 8) != 0 {
            G_EffectIndex(b"world/acid_fizz\0".as_ptr() as *const c_char);
            G_FindConfigstringIndex(b"acidrain\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // OUTSIDE SHAKE
        //===============
        if ((*ent as *mut gentity_t).spawnflags & 16) != 0 {
            G_FindConfigstringIndex(b"outsideShake\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // MISTY FOG
        //===========
        if ((*ent as *mut gentity_t).spawnflags & 32) != 0 {
            G_FindConfigstringIndex(b"fog\0".as_ptr() as *const c_char, 10, 8, 1);
        }

        // LIGHTNING
        //===========
        if ((*ent as *mut gentity_t).spawnflags & 64) != 0 {
            G_SoundIndex(b"sound/ambience/thunder1\0".as_ptr() as *const c_char);
            G_SoundIndex(b"sound/ambience/thunder2\0".as_ptr() as *const c_char);
            G_SoundIndex(b"sound/ambience/thunder3\0".as_ptr() as *const c_char);
            G_SoundIndex(b"sound/ambience/thunder4\0".as_ptr() as *const c_char);
            G_SoundIndex(b"sound/ambience/thunder_close1\0".as_ptr() as *const c_char);
            G_SoundIndex(b"sound/ambience/thunder_close2\0".as_ptr() as *const c_char);
            G_EffectIndex(b"env/huge_lightning\0".as_ptr() as *const c_char);
            (*ent as *mut gentity_t).e_ThinkFunc = 3; // thinkF_fx_rain_think
            (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + Q_irand(4000, 8000);

            if G_SpawnFloat(b"flashcolor\0".as_ptr() as *const c_char, b"200 200 200\0".as_ptr() as *const c_char, (*ent as *mut gentity_t).pos3.as_mut_ptr() as *mut f32) == 0 {
                VectorSet((*ent as *mut gentity_t).pos3.as_mut_ptr() as *mut c_void, 200.0, 200.0, 200.0);
            }
            VectorClear((*ent as *mut gentity_t).pos2.as_mut_ptr() as *mut c_void); // the "off" color

            G_SpawnInt(b"flashdelay\0".as_ptr() as *const c_char, b"12000\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).delay);
            G_SpawnInt(b"chanceflicker\0".as_ptr() as *const c_char, b"2\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).attackDebounceTime);
            G_SpawnInt(b"chancesound\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).pushDebounceTime);
            G_SpawnInt(b"chanceeffect\0".as_ptr() as *const c_char, b"4\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).aimDebounceTime);
        }
    }
}

// Added by Aurelio Reis on 10/20/02.
// QUAKED fx_puff (1 0 0) (-16 -16 -16) (16 16 16)
// This world effect will spawn a puff system globally into the level.
// Enter any valid puff command as a key and value to setup the puff
// system properties.
//
// "count"			The number of puffs/particles (default of 1000).
//
// "whichsystem"	Which puff system to use (currently 0 and 1. Default 0).
//
// // Apply a default puff system.
// default  <value>
// Current defaults are "snowstorm", "sandstorm", "foggy", and "smokey"
//
// // Set the color of the particles (0-1.0).
// color  ( <red>, <green>, <blue> )
// default ( 0.5, 0.5, 0.5 )
//
// // Set the alpha (transparency) value for the particles (0-1.0).
// alpha  <value>
// default 0.5
//
// // Set which texture to use for the particles (make sure to include full path).
// texture  <textures/texture.tga>
// default gfx/effects/alpha_smoke2b.tga
//
// // Set the size of particles (from center, like a radius) (MIN 4, MAX 2048).
// size  <value>
// default 100
//
// // Whether the saber should flicker and spark or not (0 false, 1 true).
// sabersparks   <value>
// default 0
//
// // Set texture filtering mode (0 = Bilinear(default), 1 = Nearest(less quality).
// filtermode  <value>
// default 0
//
// // Set the alpha blending mode (0 = src, src-1, 1 = one, one (additive)).
// blendmode  <value>
// default 0
//
// // How much to rotate particles per second (in degree's).
// rotate ( <min>, <max> )
// default ( 0, 0 )
//
// // Set the area around the player the puffs cover:
// spread ( minX minY minZ ) ( maxX maxY maxZ )
// default: ( -600 -600 -500 ) ( 600 600 550 )
//
// // Set the random range that sets the speed the puffs fall:
// velocity ( minX minY minZ ) ( maxX maxY maxZ )
// default: ( -15 -15 -20 ) ( 15 15 -70 )
//
// // Set an area of puff blowing:
// wind ( windOriginX windOriginY windOriginZ ) ( windVelocityX windVelocityY windVelocityZ ) ( sizeX sizeY sizeZ  )
//
// // Set puff blowing data:
// blowing duration <int>
// blowing low <int>
//		default: 3
// blowing velocity ( min max )
//		default: ( 30 70 )
// blowing size ( minX minY minZ )
//		default: ( 1000 300 300 )

//----------------------------------------------------------

#[no_mangle]
pub extern "C" fn SP_CreatePuffSystem(ent: *mut c_void) {
    unsafe {
        let mut temp: [c_char; 128] = [0; 128];

        // Initialize the puff system to either 1000 particles or whatever they choose.
        G_SpawnInt(b"count\0".as_ptr() as *const c_char, b"1000\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).count);
        let r_weatherScale = gi_cvar(b"r_weatherScale\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 2); // CVAR_ARCHIVE

        // See which puff system to use.
        let mut iPuffSystem: c_int = 0;
        let mut iVal: c_int = 0;
        if G_SpawnInt(b"whichsystem\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut iVal) != 0 {
            iPuffSystem = iVal;
            if iPuffSystem < 0 || iPuffSystem > 1 {
                iPuffSystem = 0;
                //ri.Error( ERR_DROP, "Weather Effect: Invalid value for whichsystem key" );
                Com_Printf(b"Weather Effect: Invalid value for whichsystem key\n\0".as_ptr() as *const c_char);
            }
        }

        if (*(r_weatherScale as *const cvar_t)).value > 0.0 {
            // sprintf( temp, "puff%i init %i", iPuffSystem, (int)( ent->count * r_weatherScale->value ));
            // G_FindConfigstringIndex( temp, CS_WORLD_FX, MAX_WORLD_FX, qtrue );
        }

        // See whether we should have the saber spark from the puff system.
        iVal = 0;
        G_SpawnInt(b"sabersparks\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut iVal);
        if iVal == 1 {
            // level.worldFlags |= WF_PUFFING;
        } else {
            // level.worldFlags &= ~WF_PUFFING;
        }

        // Go through all the fields and assign the values to the created puff system now.
        let mut i: c_int = 0;
        while i < 20 {
            let mut key: *const c_char = core::ptr::null();
            let mut value: *const c_char = core::ptr::null();
            // Fetch a field.
            if G_SpawnField(i, &mut key as *mut *const c_char, &mut value as *mut *const c_char) == 0 {
                i += 1;
                continue;
            }

            // Make sure we don't get key's that are worthless.
            if Q_stricmp(key, b"origin\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(key, b"classname\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(key, b"count\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(key, b"targetname\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(key, b"sabersparks\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(key, b"whichsystem\0".as_ptr() as *const c_char) == 0
            {
                i += 1;
                continue;
            }

            // Send the command.
            // _snprintf( temp, 128, "puff%i %s %s", iPuffSystem, key, value );
            // G_FindConfigstringIndex( temp, CS_WORLD_FX, MAX_WORLD_FX, qtrue );
            i += 1;
        }
    }
}

// Don't use this! Too powerful! - Aurelio
// NOTEINUSE! fx_command (1 0 0) (-16 -16 -16) (16 16 16)
// //This effect allows you to issue console commands from within the world editor.
// //Use the variables c00 to c99 to issue a maximum of 100 console commands.
//
// //example: c00 r_showtris 1
//
// // COMMENTED OUT - original code was:
// /*void SP_Command( gentity_t *ent )
// {
// 	char *strCommand;
//
// 	// Go through all the commands.
// 	for ( int i = 0; i < 100; i++ )
// 	{
// 		strCommand = NULL;
//
// 		// Fetch a command.
// 		G_SpawnString( va("c%02d", i), NULL, &strCommand );
//
// 		// If it's valid, issue it.
// 		if ( strCommand && strCommand[0] )
// 		{
// 			gi.SendConsoleCommand( strCommand );
// 		}
// 	}
// }*/

// -----------------
// Explosion Trail
// -----------------

//----------------------------------------------------------

#[no_mangle]
pub extern "C" fn fx_explosion_trail_think(ent: *mut c_void) {
    unsafe {
        let mut origin: [f32; 3] = [0.0; 3];
        let mut tr: trace_t = core::mem::zeroed();

        if ((*ent as *mut gentity_t).spawnflags & 1) != 0 {
            // gravity
            (*ent as *mut gentity_t).s.pos.trType = 1; // TR_GRAVITY
        } else {
            (*ent as *mut gentity_t).s.pos.trType = 0; // TR_LINEAR
        }

        EvaluateTrajectory(&(*ent as *mut gentity_t).s.pos as *const c_void as *const c_void, (*(&level as *const c_void as *const gentity_level_s)).time, origin.as_mut_ptr() as *mut c_void);

        gi_trace(&mut tr as *mut trace_t as *mut c_void, (*ent as *mut gentity_t).currentOrigin.as_ptr() as *const c_void, core::ptr::null(), core::ptr::null(), origin.as_ptr() as *const c_void,
            if !(*ent as *mut gentity_t).owner.is_null() { (*((*ent as *mut gentity_t).owner as *const gentity_t)).s.number } else { 2047 },
            (*ent as *mut gentity_t).clipmask, 0, 10);

        if tr.fraction < 1.0 {
            // never explode or bounce on sky
            if (tr.surfaceFlags & 0x4000) == 0 {
                if ((*ent as *mut gentity_t).splashDamage != 0) && ((*ent as *mut gentity_t).splashRadius != 0) {
                    G_RadiusDamage(tr.endpos.as_ptr() as *const c_void, ent, (*ent as *mut gentity_t).splashDamage, (*ent as *mut gentity_t).splashRadius, ent, 8); // MOD_EXPLOSIVE_SPLASH
                }
            }

            if !(*ent as *mut gentity_t).cameraGroup.is_null() {
                // fxFile2....in other words, impact fx
                G_PlayEffect((*ent as *mut gentity_t).cameraGroup, tr.endpos.as_ptr() as *const c_void, tr.plane.normal.as_ptr() as *const c_void);
            }

            if !(*ent as *mut gentity_t).soundSet.is_null() && !(*(*ent as *mut gentity_t).soundSet).is_null() {
                G_AddEvent(ent, 12, CAS_GetBModelSound((*ent as *mut gentity_t).soundSet, BMS_END));
            }

            G_FreeEntity(ent);
            return;
        }

        G_RadiusDamage(origin.as_ptr() as *const c_void, ent, (*ent as *mut gentity_t).damage, (*ent as *mut gentity_t).radius, ent, 8); // MOD_EXPLOSIVE_SPLASH

        // call the effect with the desired position and orientation
        G_PlayEffect((*ent as *mut gentity_t).fxID as *const c_char as *const c_void as *const c_char, origin.as_ptr() as *const c_void, (*ent as *mut gentity_t).currentAngles.as_ptr() as *const c_void);

        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50;
        gi_linkentity(ent);
    }
}

//----------------------------------------------------------

#[no_mangle]
pub extern "C" fn fx_explosion_trail_use(
    self_: *mut c_void,
    other: *mut c_void,
    activator: *mut c_void,
) {
    unsafe {
        let missile = G_Spawn();

        // We aren't a missile in the truest sense, rather we just move through the world and spawn effects
        if !missile.is_null() {
            (*missile as *mut gentity_t).classname = b"fx_exp_trail\0".as_ptr() as *const c_char;

            (*missile as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50;
            (*missile as *mut gentity_t).e_ThinkFunc = 4; // thinkF_fx_explosion_trail_think

            (*missile as *mut gentity_t).s.eType = 3; // ET_MOVER

            (*missile as *mut gentity_t).owner = self_;

            (*missile as *mut gentity_t).s.modelindex2 = (*self_ as *mut gentity_t).s.modelindex2;

            (*missile as *mut gentity_t).s.pos.trTime = (*(&level as *const c_void as *const gentity_level_s)).time;
            G_SetOrigin(missile, (*self_ as *mut gentity_t).currentOrigin.as_ptr() as *const c_void);
            if ((*self_ as *mut gentity_t).spawnflags & 1) != 0 {
                // gravity
                (*missile as *mut gentity_t).s.pos.trType = 1; // TR_GRAVITY
            } else {
                (*missile as *mut gentity_t).s.pos.trType = 0; // TR_LINEAR
            }

            (*missile as *mut gentity_t).spawnflags = (*self_ as *mut gentity_t).spawnflags;

            G_SetAngles(missile, (*self_ as *mut gentity_t).currentAngles.as_ptr() as *const c_void);
            VectorScale((*self_ as *mut gentity_t).currentAngles.as_ptr() as *const c_void, (*self_ as *mut gentity_t).speed, (*missile as *mut gentity_t).s.pos.trDelta.as_mut_ptr() as *mut c_void);
            (*missile as *mut gentity_t).s.pos.trTime = (*(&level as *const c_void as *const gentity_level_s)).time;
            (*missile as *mut gentity_t).radius = (*self_ as *mut gentity_t).radius;
            (*missile as *mut gentity_t).damage = (*self_ as *mut gentity_t).damage;
            (*missile as *mut gentity_t).splashDamage = (*self_ as *mut gentity_t).splashDamage;
            (*missile as *mut gentity_t).splashRadius = (*self_ as *mut gentity_t).splashRadius;
            (*missile as *mut gentity_t).fxID = (*self_ as *mut gentity_t).fxID;
            (*missile as *mut gentity_t).cameraGroup = (*self_ as *mut gentity_t).cameraGroup; //fxfile2

            (*missile as *mut gentity_t).clipmask = 24; // MASK_SHOT

            gi_linkentity(missile);

            if !(*self_ as *mut gentity_t).soundSet.is_null() && !(*(*self_ as *mut gentity_t).soundSet).is_null() {
                G_AddEvent(self_, 12, CAS_GetBModelSound((*self_ as *mut gentity_t).soundSet, BMS_START));
                (*missile as *mut gentity_t).s.loopSound = CAS_GetBModelSound((*self_ as *mut gentity_t).soundSet, BMS_MID);
                (*missile as *mut gentity_t).soundSet = G_NewString((*self_ as *mut gentity_t).soundSet); //get my own copy so i can free it when i die

                if (*missile as *mut gentity_t).s.loopSound < 0 {
                    (*missile as *mut gentity_t).s.loopSound = 0;
                }
            }
        }
    }
}

//----------------------------------------------------------

#[no_mangle]
pub extern "C" fn fx_explosion_trail_link(ent: *mut c_void) {
    unsafe {
        let mut dir: [f32; 3] = [0.0; 3];
        let mut target: *mut gentity_t = core::ptr::null_mut();

        // we ony activate when used
        (*ent as *mut gentity_t).e_UseFunc = 5; // useF_fx_explosion_trail_use

        if !(*ent as *mut gentity_t).target.is_null() {
            // try to use the target to override the orientation
            target = G_Find(target as *mut c_void, FOFS(b"targetname\0".as_ptr() as *const c_char), (*ent as *mut gentity_t).target) as *mut gentity_t;

            if target.is_null() {
                gi_Printf(b"\x1b[31mERROR: fx_explosion_trail %s could not find target %s\n\0".as_ptr() as *const c_char, (*ent as *mut gentity_t).targetname, (*ent as *mut gentity_t).target);
                G_FreeEntity(ent);
                return;
            }

            // Our target is valid so lets use that
            VectorSubtract((*target).s.origin.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void, dir.as_mut_ptr() as *mut c_void);
            VectorNormalize(dir.as_mut_ptr() as *mut c_void);
        } else {
            // we are assuming that we have angles, but there are no checks to verify this
            AngleVectors((*ent as *mut gentity_t).s.angles.as_ptr() as *const c_void, dir.as_mut_ptr() as *mut c_void, core::ptr::null_mut(), core::ptr::null_mut());
        }

        // NOTE: this really isn't an angle, but rather an orientation vector
        G_SetAngles(ent, dir.as_ptr() as *const c_void);
    }
}

// QUAKED fx_explosion_trail (0 0 1) (-8 -8 -8) (8 8 8) GRAVITY
// Creates an explosion type trail using the specified effect file, damaging things as it moves through the environment
// Can also be used for something like a meteor, just add an impact effect ( fxFile2 ) and a splashDamage and splashRadius
//
//   GRAVITY - object uses gravity instead of linear motion
//
//   "fxFile" - name of the effect to play for the trail ( default "env/exp_trail_comp" )
//   "fxFile2" - effect file to play on impact
//
//   "model" - model to attach to the trail
//
//   "target" - direction to aim the trail in, required unless you specify angles
//   "targetname" - (required) trail effect spawns only when used.
//   "speed" - velocity through the world, ( default 350 )
//
//   "radius" - damage radius around trail as it travels through the world ( default 128 )
//   "damage" - radius damage ( default 128 )
//   "splashDamage" - damage when thing impacts ( default 0 )
//   "splashRadius" - damage radius on impact ( default 0 )
//   "soundset" - soundset to use, start sound plays when explosion trail starts, loop sound plays on explosion trail, end sound plays when it impacts

//----------------------------------------------------------

#[no_mangle]
pub extern "C" fn SP_fx_explosion_trail(ent: *mut c_void) {
    unsafe {
        // We have to be useable, otherwise we won't spawn in
        if (*ent as *mut gentity_t).targetname.is_null() {
            gi_Printf(b"\x1b[31mERROR: fx_explosion_trail at %s has no targetname specified\n\0".as_ptr() as *const c_char, vtos((*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void));
            G_FreeEntity(ent);
            return;
        }

        // Get our defaults
        G_SpawnString(b"fxFile\0".as_ptr() as *const c_char, b"env/exp_trail_comp\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).fxFile);
        G_SpawnInt(b"damage\0".as_ptr() as *const c_char, b"128\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).damage);
        G_SpawnFloat(b"radius\0".as_ptr() as *const c_char, b"128\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).radius);
        G_SpawnFloat(b"speed\0".as_ptr() as *const c_char, b"350\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).speed);

        // Try to associate an effect file, unfortunately we won't know if this worked or not until the CGAME trys to register it...
        (*ent as *mut gentity_t).fxID = G_EffectIndex((*ent as *mut gentity_t).fxFile);

        if !(*ent as *mut gentity_t).cameraGroup.is_null() {
            G_EffectIndex((*ent as *mut gentity_t).cameraGroup);
        }

        if !(*ent as *mut gentity_t).model.is_null() {
            (*ent as *mut gentity_t).s.modelindex2 = G_ModelIndex((*ent as *mut gentity_t).model);
        }

        // Give us a bit of time to spawn in the other entities, since we may have to target one of 'em
        (*ent as *mut gentity_t).e_ThinkFunc = 6; // thinkF_fx_explosion_trail_link
        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 500;

        // Save our position and link us up!
        G_SetOrigin(ent, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void);

        VectorSet((*ent as *mut gentity_t).maxs.as_mut_ptr() as *mut c_void, FX_ENT_RADIUS as f32, FX_ENT_RADIUS as f32, FX_ENT_RADIUS as f32);
        VectorScale((*ent as *mut gentity_t).maxs.as_ptr() as *const c_void, -1.0, (*ent as *mut gentity_t).mins.as_mut_ptr() as *mut c_void);

        gi_linkentity(ent);
    }
}

// //------------------------------------------

#[no_mangle]
pub extern "C" fn fx_target_beam_set_debounce(self_: *mut c_void) {
    unsafe {
        let frametime = 50; // FRAMETIME
        if (*self_ as *mut gentity_t).wait >= frametime {
            (*self_ as *mut gentity_t).attackDebounceTime = (*(&level as *const c_void as *const gentity_level_s)).time + (*self_ as *mut gentity_t).wait + Q_irand(-(*self_ as *mut gentity_t).random as i32, (*self_ as *mut gentity_t).random as i32);
        } else if (*self_ as *mut gentity_t).wait < 0 {
            (*self_ as *mut gentity_t).e_UseFunc = 0; // useF_NULL
        } else {
            (*self_ as *mut gentity_t).attackDebounceTime = (*(&level as *const c_void as *const gentity_level_s)).time + frametime + Q_irand(-(*self_ as *mut gentity_t).random as i32, (*self_ as *mut gentity_t).random as i32);
        }
    }
}

//------------------------------------------

#[no_mangle]
pub extern "C" fn fx_target_beam_fire(ent: *mut c_void) {
    unsafe {
        let mut trace: trace_t = core::mem::zeroed();
        let mut dir: [f32; 3] = [0.0; 3];
        let mut org: [f32; 3] = [0.0; 3];
        let mut end: [f32; 3] = [0.0; 3];
        let mut ignore: c_int;
        let mut open: c_int;

        if (*ent as *mut gentity_t).enemy.is_null() || (*(*ent as *mut gentity_t).enemy as *const gentity_t).inuse == 0 {
            //info_null most likely
            ignore = (*ent as *mut gentity_t).s.number;
            (*ent as *mut gentity_t).enemy = core::ptr::null_mut();
            VectorCopy((*ent as *mut gentity_t).s.origin2.as_ptr() as *const c_void, org.as_mut_ptr() as *mut c_void);
        } else {
            ignore = (*(*ent as *mut gentity_t).enemy as *const gentity_t).s.number;
            VectorCopy((*(*ent as *mut gentity_t).enemy as *const gentity_t).currentOrigin.as_ptr() as *const c_void, org.as_mut_ptr() as *mut c_void);
        }

        VectorCopy(org.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.origin2.as_mut_ptr() as *mut c_void);
        VectorSubtract(org.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void, dir.as_mut_ptr() as *mut c_void);
        VectorNormalize(dir.as_mut_ptr() as *mut c_void);

        gi_trace(&mut trace as *mut trace_t as *mut c_void, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void, core::ptr::null(), core::ptr::null(), org.as_ptr() as *const c_void, 2047, 24, 0, 0); //ignore
        if ((*ent as *mut gentity_t).spawnflags & 2) != 0 {
            open = 1;
            VectorCopy(org.as_ptr() as *const c_void, end.as_mut_ptr() as *mut c_void);
        } else {
            open = 0;
            VectorCopy(trace.endpos.as_ptr() as *const c_void, end.as_mut_ptr() as *mut c_void);
        }

        if trace.fraction < 1.0 {
            if trace.entityNum < 2047 {
                let victim = &mut g_entities as *mut c_void as *mut gentity_t;
                let victim = &mut *victim.add(trace.entityNum as usize);
                if !victim.is_null() && (*victim).takedamage != 0 {
                    if ((*ent as *mut gentity_t).spawnflags & 4) != 0 {
                        // NO_KNOCKBACK
                        G_Damage(victim as *mut c_void, ent, (*ent as *mut gentity_t).activator, dir.as_ptr() as *const c_void, trace.endpos.as_ptr() as *const c_void, (*ent as *mut gentity_t).damage, 4, 13); // DAMAGE_NO_KNOCKBACK, MOD_UNKNOWN
                    } else {
                        G_Damage(victim as *mut c_void, ent, (*ent as *mut gentity_t).activator, dir.as_ptr() as *const c_void, trace.endpos.as_ptr() as *const c_void, (*ent as *mut gentity_t).damage, 0, 13); // MOD_UNKNOWN
                    }
                }
            }
        }

        G_AddEvent(ent, 22, (*ent as *mut gentity_t).fxID); // EV_TARGET_BEAM_DRAW
        VectorCopy(end.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.origin2.as_mut_ptr() as *mut c_void);

        if open != 0 {
            VectorScale(dir.as_ptr() as *const c_void, -1.0, (*ent as *mut gentity_t).pos1.as_mut_ptr() as *mut c_void);
        } else {
            VectorCopy(trace.plane.normal.as_ptr() as *const c_void, (*ent as *mut gentity_t).pos1.as_mut_ptr() as *mut c_void);
        }

        (*ent as *mut gentity_t).e_ThinkFunc = 7; // thinkF_fx_target_beam_think
        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50; // FRAMETIME
    }
}

//------------------------------------------

#[no_mangle]
pub extern "C" fn fx_target_beam_fire_start(self_: *mut c_void) {
    unsafe {
        fx_target_beam_set_debounce(self_);
        (*self_ as *mut gentity_t).e_ThinkFunc = 7; // thinkF_fx_target_beam_think
        (*self_ as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50; // FRAMETIME
        (*self_ as *mut gentity_t).painDebounceTime = (*(&level as *const c_void as *const gentity_level_s)).time + (*self_ as *mut gentity_t).speed as i32 + Q_irand(-500, 500);
        fx_target_beam_fire(self_);
    }
}

//------------------------------------------

#[no_mangle]
pub extern "C" fn fx_target_beam_use(
    self_: *mut c_void,
    other: *mut c_void,
    activator: *mut c_void,
) {
    unsafe {
        if ((*self_ as *mut gentity_t).spawnflags & 8) != 0 {
            // one shot
            fx_target_beam_fire(self_);
            (*self_ as *mut gentity_t).e_ThinkFunc = 0; // thinkF_NULL
        } else if (*self_ as *mut gentity_t).e_ThinkFunc == 0 {
            (*self_ as *mut gentity_t).e_ThinkFunc = 7; // thinkF_fx_target_beam_think
            (*self_ as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50;
        } else {
            (*self_ as *mut gentity_t).e_ThinkFunc = 0; // thinkF_NULL
        }

        (*self_ as *mut gentity_t).activator = activator;
    }
}

//------------------------------------------

#[no_mangle]
pub extern "C" fn fx_target_beam_think(ent: *mut c_void) {
    unsafe {
        if (*ent as *mut gentity_t).attackDebounceTime > (*(&level as *const c_void as *const gentity_level_s)).time {
            (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50; // FRAMETIME
            return;
        }

        fx_target_beam_fire_start(ent);
    }
}

//------------------------------------------

#[no_mangle]
pub extern "C" fn fx_target_beam_link(ent: *mut c_void) {
    unsafe {
        let mut target: *mut gentity_t = core::ptr::null_mut();
        let mut dir: [f32; 3] = [0.0; 3];
        let mut len: f32;

        target = G_Find(target as *mut c_void, FOFS(b"targetname\0".as_ptr() as *const c_char), (*ent as *mut gentity_t).target) as *mut gentity_t;

        if target.is_null() {
            Com_Printf(b"bolt_link: unable to find target %s\n\0".as_ptr() as *const c_char, (*ent as *mut gentity_t).target);
            G_FreeEntity(ent);
            return;
        }

        (*ent as *mut gentity_t).attackDebounceTime = (*(&level as *const c_void as *const gentity_level_s)).time;

        if !(*target).classname.is_null() && Q_stricmp(b"info_null\0".as_ptr() as *const c_char, (*target).classname) != 0 {
            //don't want to set enemy to something that's going to free itself... actually, this could be bad in other ways, too... ent pointer could be freed up and re-used by the time we check it next
            G_SetEnemy(ent, target as *mut c_void);
        }
        VectorSubtract((*target).s.origin.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void, dir.as_mut_ptr() as *mut c_void); //er, does it ever use dir?
        len = VectorNormalize(dir.as_mut_ptr() as *mut c_void); //er, does it use len or dir?
        vectoangles(dir.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.angles.as_mut_ptr() as *mut c_void); //er, does it use s.angles?

        VectorCopy((*target).s.origin.as_ptr() as *const c_void, (*ent as *mut gentity_t).s.origin2.as_mut_ptr() as *mut c_void);

        if ((*ent as *mut gentity_t).spawnflags & 1) != 0 {
            // Do nothing
            (*ent as *mut gentity_t).e_ThinkFunc = 0; // thinkF_NULL
        } else {
            if ((*ent as *mut gentity_t).spawnflags & 8) == 0 {
                // one_shot, only calls when used
                // switch think functions to avoid doing the bolt_link every time
                (*ent as *mut gentity_t).e_ThinkFunc = 7; // thinkF_fx_target_beam_think
                (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 50; // FRAMETIME
            }
        }

        (*ent as *mut gentity_t).e_UseFunc = 8; // useF_fx_target_beam_use
        gi_linkentity(ent);
    }
}

// QUAKED fx_target_beam (1 0.5 0.5) (-8 -8 -8) (8 8 8) STARTOFF OPEN NO_KNOCKBACK ONE_SHOT NO_IMPACT
//  Emits specified effect file, doing damage if required
//
// STARTOFF - must be used before it's on
// OPEN - will draw all the way to the target, regardless of where the trace hits
// NO_KNOCKBACK - beam damage does no knockback
//
//  "fxFile" - Effect used to draw the beam, ( default "env/targ_beam" )
//  "fxFile2" - Effect used for the beam impact effect, ( default "env/targ_beam_impact" )
//  "targetname" - Fires only when used
//  "duration" - How many seconds each burst lasts, -1 will make it stay on forever
//  "wait" - If always on, how long to wait between blasts, in MILLISECONDS - default/min is 100 (1 frame at 10 fps), -1 means it will never fire again
//  "random" - random amount of seconds added to/subtracted from "wait" each firing
//  "damage" - How much damage to inflict PER FRAME (so probably want it kind of low), default is none
//  "target" - ent to point at- you MUST have this.  This can be anything you want, including a moving ent - for static beams, just use info_null

//------------------------------------------

#[no_mangle]
pub extern "C" fn SP_fx_target_beam(ent: *mut c_void) {
    unsafe {
        G_SetOrigin(ent, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void);

        (*ent as *mut gentity_t).speed *= 1000.0;
        (*ent as *mut gentity_t).wait *= 1000.0;
        (*ent as *mut gentity_t).random *= 1000.0;

        if (*ent as *mut gentity_t).speed < 50.0 {
            // FRAMETIME
            (*ent as *mut gentity_t).speed = 50.0;
        }

        G_SpawnInt(b"damage\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).damage);
        G_SpawnString(b"fxFile\0".as_ptr() as *const c_char, b"env/targ_beam\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).fxFile);

        if ((*ent as *mut gentity_t).spawnflags & 16) != 0 {
            // NO_IMPACT FX
            (*ent as *mut gentity_t).delay = 0;
        } else {
            G_SpawnString(b"fxFile2\0".as_ptr() as *const c_char, b"env/targ_beam_impact\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).cameraGroup);
            (*ent as *mut gentity_t).delay = G_EffectIndex((*ent as *mut gentity_t).cameraGroup);
        }

        (*ent as *mut gentity_t).fxID = G_EffectIndex((*ent as *mut gentity_t).fxFile);

        (*ent as *mut gentity_t).activator = ent;
        (*ent as *mut gentity_t).owner = core::ptr::null_mut();

        (*ent as *mut gentity_t).e_ThinkFunc = 9; // thinkF_fx_target_beam_link
        (*ent as *mut gentity_t).nextthink = (*(&level as *const c_void as *const gentity_level_s)).time + 100; // START_TIME_LINK_ENTS

        VectorSet((*ent as *mut gentity_t).maxs.as_mut_ptr() as *mut c_void, FX_ENT_RADIUS as f32, FX_ENT_RADIUS as f32, FX_ENT_RADIUS as f32);
        VectorScale((*ent as *mut gentity_t).maxs.as_ptr() as *const c_void, -1.0, (*ent as *mut gentity_t).mins.as_mut_ptr() as *mut c_void);

        gi_linkentity(ent);
    }
}

// QUAKED fx_cloudlayer (1 0.3 0.5) (-8 -8 -8) (8 8 8) TUBE ALT
//
//   Creates a scalable scrolling cloud layer, mostly for bespin undercity but could be used other places
//
//   TUBE - creates cloud layer with tube opening in the middle, must an INNER radius also
//   ALT - uses slightly different shader, good if using two layers sort of close together
//
// "radius" - outer radius of cloud layer, (default 2048)
// "random" - inner radius of cloud layer, (default 128) only works for TUBE type
// "wait" - adds curvature as it moves out to the edge of the layer.  ( default 0 ), 1 = small up, 3 = up more, -1 = small down, -3 = down more, etc.

#[no_mangle]
pub extern "C" fn SP_fx_cloudlayer(ent: *mut c_void) {
    unsafe {
        // HACK: this effect is never played, rather it just caches the shaders I need cgame side
        G_EffectIndex(b"world/haze_cache\0".as_ptr() as *const c_char);

        G_SpawnFloat(b"radius\0".as_ptr() as *const c_char, b"2048\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).radius);
        G_SpawnFloat(b"random\0".as_ptr() as *const c_char, b"128\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).random);
        G_SpawnFloat(b"wait\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut (*ent as *mut gentity_t).wait);

        (*ent as *mut gentity_t).s.eType = 19; // ET_CLOUD

        G_SetOrigin(ent, (*ent as *mut gentity_t).s.origin.as_ptr() as *const c_void);

        (*ent as *mut gentity_t).contents = 0;
        VectorSet((*ent as *mut gentity_t).maxs.as_mut_ptr() as *mut c_void, 200.0, 200.0, 200.0);
        VectorScale((*ent as *mut gentity_t).maxs.as_ptr() as *const c_void, -1.0, (*ent as *mut gentity_t).mins.as_mut_ptr() as *mut c_void);

        gi_linkentity(ent);
    }
}

// Stub structures for type definitions
#[repr(C)]
pub struct gentity_t {
    // Placeholder - actual structure defined elsewhere
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct cvar_t {
    pub name: *const c_char,
    pub string: *const c_char,
    pub resetString: *const c_char,
    pub latchedString: *const c_char,
    pub flags: c_int,
    pub modified: c_int,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *const cvar_t,
    pub prev: *const cvar_t,
}

#[repr(C)]
pub struct gentity_level_s {
    pub time: c_int,
    // ... rest of level structure
}

#[repr(C)]
pub struct gentity_pos_s {
    pub pos: c_void,
    pub apos: c_void,
    // ... rest of position structure
}

#[repr(C)]
pub struct trace_t {
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: plane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct plane_t {
    pub normal: [f32; 3],
    pub dist: f32,
}
