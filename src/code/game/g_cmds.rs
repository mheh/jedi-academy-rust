// leave this line at the top for all g_xxxx.cpp files...

use core::ffi::{c_int, c_char, c_void};

// Stubs for includes that would map to other modules
// These are declared but implementations are in other modules

extern "C" {
    static mut in_camera: bool;
    static SaberStyleTable: [stringID_table_t; 0];

    fn ForceHeal(self_: *mut gentity_t);
    fn ForceGrip(self_: *mut gentity_t);
    fn ForceTelepathy(self_: *mut gentity_t);
    fn ForceRage(self_: *mut gentity_t);
    fn ForceProtect(self_: *mut gentity_t);
    fn ForceAbsorb(self_: *mut gentity_t);
    fn ForceSeeing(self_: *mut gentity_t);
    fn G_CreateG2AttachedWeaponModel(ent: *mut gentity_t, psWeaponModel: *const c_char, boltNum: c_int, weaponNum: c_int);
    fn G_StartMatrixEffect(ent: *mut gentity_t, meFlags: c_int, length: c_int, timeScale: f32, spinTime: c_int);
    fn ItemUse_Bacta(ent: *mut gentity_t);
    fn G_GetSelfForPlayerCmd() -> *mut gentity_t;
    fn ForceThrow(ent: *mut gentity_t, pull: bool);
    fn ForceSpeed(ent: *mut gentity_t);
    fn G_Find(ent: *mut gentity_t, fieldofs: c_int, match_: *const c_char) -> *mut gentity_t;
    fn SP_fx_runner(ent: *mut gentity_t);
    fn G_Spawn() -> *mut gentity_t;
    fn G_FreeEntity(ent: *mut gentity_t);
    fn G_SetOrigin(ent: *mut gentity_t, origin: *const [f32; 3]);
    fn G_SetAngles(ent: *mut gentity_t, angles: *const [f32; 3]);
    fn G_SoundIndex(sound: *const c_char) -> c_int;
    fn G_Sound(ent: *mut gentity_t, soundIndex: c_int);
    fn G_AddEvent(ent: *mut gentity_t, event: c_int, eventparm: c_int);
    fn G_CallSpawn(ent: *mut gentity_t) -> bool;
    fn G_NewString(string: *const c_char) -> *mut c_char;
    fn G_DropSaberItem(saberType: *const c_char, saberColor: c_int, origin: *const [f32; 3], velocity: *const [f32; 3], angles: *const [f32; 3]) -> *mut gentity_t;
    fn G_CheckPlayerDarkSide() -> bool;
    fn WP_RemoveSaber(ent: *mut gentity_t, saberNum: c_int);
    fn CG_ChangeWeapon(num: c_int);
    fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    fn player_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, meansOfDeath: c_int);
    fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    fn VectorMA(veca: *const [f32; 3], scale: f32, vecb: *const [f32; 3], vecc: *mut [f32; 3]);
    fn VectorCopy(in_: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    fn VectorClear(v: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn FindItem(name: *const c_char) -> *mut gitem_t;
    fn G_SpawnItem(ent: *mut gentity_t, it: *mut gitem_t);
    fn FinishSpawningItem(ent: *mut gentity_t);
    fn Touch_Item(ent: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    fn TeleportPlayer(ent: *mut gentity_t, origin: *const [f32; 3], angles: *const [f32; 3]);
    fn NPC_SetAnim(ent: *mut gentity_t, setAnimParts: c_int, anim: c_int, flags: c_int);
    fn G_SpeechEvent(self_: *mut gentity_t, event: c_int);
    fn place_portable_assault_sentry(self_: *mut gentity_t, origin: *const [f32; 3], dir: *const [f32; 3]) -> bool;
    fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, soundfile: *const c_char);
    fn GetIDForString(table: *const stringID_table_t, string: *const c_char) -> c_int;
    fn vtos(v: *const [f32; 3]) -> *const c_char;

    // Game interface
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize);
    fn tolower(c: c_int) -> c_int;
    fn atoi(nptr: *const c_char) -> c_int;
    fn atof(nptr: *const c_char) -> f32;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    static mut g_entities: [gentity_t; 0];
    static mut level: level_locals_t;
    static g_cheats: *mut cvar_t;
    static cg: cg_t;
    static g_saberPickuppableDroppedSabers: *mut cvar_t;

    fn gi_argc() -> c_int;
    fn gi_argv(i: c_int) -> *const c_char;
    fn gi_SendServerCommand(clientNum: c_int, fmt: *const c_char, ...);
    fn gi_Printf(fmt: *const c_char, ...);
    fn gi_SetConfigstring(index: c_int, val: *const c_char);
    fn gi_linkentity(ent: *mut gentity_t);
    fn gi_trace(results: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passent: c_int, contentmask: c_int);
    fn gi_FlushCamFile();
}

#[repr(C)]
pub struct gentity_t {
    // Stub - full definition would be in g_local.h
}

#[repr(C)]
pub struct gclient_t {
    // Stub
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
    pub maxclients: c_int,
    pub clients: *mut gclient_t,
    pub num_entities: c_int,
}

#[repr(C)]
pub struct cvar_t {
    // Stub
}

#[repr(C)]
pub struct gitem_t {
    // Stub
}

#[repr(C)]
pub struct trace_t {
    // Stub
}

#[repr(C)]
pub struct stringID_table_t {
    // Stub
}

#[repr(C)]
pub struct cg_t {
    pub inventorySelect: c_int,
    pub saberAnimLevelPending: c_int,
}

// Constants and macros would be defined here
const MAX_STRING_CHARS: usize = 1024;
const MAX_BATTERIES: c_int = 100;
const FORCE_POWER_MAX: c_int = 100;
const AMMO_MAX: c_int = 16;
const Q3_INFINITE: c_int = 1000000;

// Flags
const FL_GODMODE: c_int = 0x00000010;
const FL_UNDYING: c_int = 0x00000020;
const FL_NOTARGET: c_int = 0x00000040;

// Stat indices
const STAT_HEALTH: c_int = 0;
const STAT_MAX_HEALTH: c_int = 1;
const STAT_WEAPONS: c_int = 2;
const STAT_ARMOR: c_int = 3;
const STAT_ITEMS: c_int = 4;

// Powerups
const PW_BATTLESUIT: c_int = 3;
const PW_SEEKER: c_int = 8;

// Inventory
const INV_ELECTROBINOCULARS: c_int = 0;
const INV_BACTA_CANISTER: c_int = 1;
const INV_SEEKER: c_int = 2;
const INV_LIGHTAMP_GOGGLES: c_int = 3;
const INV_SENTRY: c_int = 4;
const INV_GOODIE_KEY: c_int = 5;
const INV_SECURITY_KEY: c_int = 6;

// Weapons
const WP_NONE: c_int = 0;
const WP_SABER: c_int = 1;
const WP_MELEE: c_int = 2;
const WP_NUM_WEAPONS: c_int = 16;

// Connection states
const CON_CONNECTED: c_int = 2;

// Saber styles
const SS_NONE: c_int = -1;
const SS_FAST: c_int = 0;
const SS_MEDIUM: c_int = 1;
const SS_STRONG: c_int = 2;
const SS_DESANN: c_int = 3;
const SS_TAVION: c_int = 4;
const SS_DUAL: c_int = 5;
const SS_STAFF: c_int = 6;

// Entity numbers
const ENTITYNUM_NONE: c_int = 1023;

// Animation settings
const BOTH_ENGAGETAUNT: c_int = 0;
const BOTH_GESTURE1: c_int = 1;
const BOTH_DUAL_TAUNT: c_int = 2;
const BOTH_STAFF_TAUNT: c_int = 3;
const BOTH_BOW: c_int = 4;
const BOTH_MEDITATE: c_int = 5;
const BOTH_SHOWOFF_FAST: c_int = 6;
const BOTH_SHOWOFF_MEDIUM: c_int = 7;
const BOTH_SHOWOFF_STRONG: c_int = 8;
const BOTH_SHOWOFF_DUAL: c_int = 9;
const BOTH_SHOWOFF_STAFF: c_int = 10;
const BOTH_VICTORY_FAST: c_int = 11;
const BOTH_VICTORY_MEDIUM: c_int = 12;
const BOTH_VICTORY_STRONG: c_int = 13;
const BOTH_VICTORY_DUAL: c_int = 14;
const BOTH_VICTORY_STAFF: c_int = 15;

// Events
const EV_USE_INV_BINOCULARS: c_int = 0;
const EV_USE_INV_LIGHTAMP_GOGGLES: c_int = 1;
const EV_USE_INV_SENTRY: c_int = 2;
const EV_ANGER1: c_int = 100;
const EV_ANGER3: c_int = 102;
const EV_TAUNT1: c_int = 103;
const EV_TAUNT3: c_int = 105;
const EV_DEFLECT1: c_int = 106;
const EV_DEFLECT3: c_int = 108;
const EV_GLOAT1: c_int = 109;
const EV_GLOAT3: c_int = 111;
const EV_VICTORY1: c_int = 112;
const EV_VICTORY3: c_int = 114;

// Damage means
const MOD_SUICIDE: c_int = 7;

// Mask
const MASK_PLAYERSOLID: c_int = 0x00000001;

// Channel
const CHAN_VOICE: c_int = 0;

// Animation flags
const SETANIM_TORSO: c_int = 1;
const SETANIM_BOTH: c_int = 3;
const SETANIM_FLAG_OVERRIDE: c_int = 0x0100;
const SETANIM_FLAG_HOLD: c_int = 0x0200;

// Field offsets for FOFS
const FOFS_classname: c_int = 0;
const FOFS_targetname: c_int = 1;

// Color codes
const S_COLOR_GREEN: &[u8; 1] = b"";
const S_COLOR_CYAN: &[u8; 1] = b"";

// Constants for setViewPos
const YAW: usize = 1;
const CS_MUSIC: c_int = 1;

// Ammo data
#[repr(C)]
pub struct ammo_data_t {
    pub max: c_int,
}

extern "C" {
    static ammoData: [ammo_data_t; AMMO_MAX as usize];
}

// LOCAL stubs for Q functions
#[inline]
pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe { strcmp(s1, s2) }
}

#[inline]
pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: usize) -> c_int {
    unsafe {
        for i in 0..n {
            let c1 = *s1.add(i) as c_int;
            let c2 = *s2.add(i) as c_int;
            if c1 != c2 {
                return c1 - c2;
            }
            if c1 == 0 {
                return 0;
            }
        }
        0
    }
}

#[inline]
pub fn Q_irand(min: c_int, max: c_int) -> c_int {
    // stub - would use random number generator
    min
}

#[inline]
pub fn PInUse(i: c_int) -> bool {
    // stub
    false
}

pub struct gi_t;

impl gi_t {
    #[inline]
    pub fn argc() -> c_int {
        unsafe { gi_argc() }
    }

    #[inline]
    pub fn argv(i: c_int) -> *const c_char {
        unsafe { gi_argv(i) }
    }

    #[inline]
    pub fn SendServerCommand(clientNum: c_int, fmt: *const c_char) {
        unsafe { gi_SendServerCommand(clientNum, fmt) }
    }

    #[inline]
    pub fn Printf(fmt: *const c_char) {
        unsafe { gi_Printf(fmt) }
    }

    #[inline]
    pub fn SetConfigstring(index: c_int, val: *const c_char) {
        unsafe { gi_SetConfigstring(index, val) }
    }

    #[inline]
    pub fn linkentity(ent: *mut gentity_t) {
        unsafe { gi_linkentity(ent) }
    }

    #[inline]
    pub fn trace(results: *mut trace_t, start: *const [f32; 3], mins: *const [f32; 3], maxs: *const [f32; 3], end: *const [f32; 3], passent: c_int, contentmask: c_int) {
        unsafe { gi_trace(results, start, mins, maxs, end, passent, contentmask) }
    }

    #[inline]
    pub fn FlushCamFile() {
        unsafe { gi_FlushCamFile() }
    }
}

pub const gi: gi_t = gi_t;

/*
==================
CheatsOk
==================
*/
pub fn CheatsOk(ent: *mut gentity_t) -> bool {
    unsafe {
        if (*g_cheats).integer == 0 {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char);
            return false;
        }
        if (*ent).health <= 0 {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"You must be alive to use this command.\n\"\0".as_ptr() as *const c_char);
            return false;
        }
        true
    }
}


/*
==================
ConcatArgs
==================
*/
pub fn ConcatArgs(start: c_int) -> *mut c_char {
    static mut line: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
    let mut i: c_int;
    let mut c: c_int;
    let mut tlen: usize;
    let mut len: usize = 0;
    let mut arg: *const c_char;

    len = 0;
    c = gi.argc();
    i = start;
    while i < c {
        arg = gi.argv(i);
        tlen = unsafe { strlen(arg) };
        if len + tlen >= MAX_STRING_CHARS - 1 {
            break;
        }
        unsafe {
            memcpy(line.as_mut_ptr().add(len) as *mut c_void, arg as *const c_void, tlen);
        }
        len += tlen;
        if i != c - 1 {
            unsafe {
                line[len] = ' ' as c_char;
            }
            len += 1;
        }
        i += 1;
    }

    unsafe {
        line[len] = 0;
        line.as_mut_ptr()
    }
}

/*
==================
SanitizeString

Remove case and control characters
==================
*/
pub fn SanitizeString(in_: *mut c_char, out: *mut c_char) {
    unsafe {
        let mut in_ptr = in_;
        let mut out_ptr = out;
        while *in_ptr != 0 {
            if *in_ptr == 27 {
                in_ptr = in_ptr.add(2);		// skip color code
                continue;
            }
            if (*in_ptr as u8) < 32 {
                in_ptr = in_ptr.add(1);
                continue;
            }
            *out_ptr = tolower(*in_ptr as c_int) as c_char;
            out_ptr = out_ptr.add(1);
            in_ptr = in_ptr.add(1);
        }
        *out_ptr = 0;
    }
}

/*
==================
ClientNumberFromString

Returns a player number for either a number or name string
Returns -1 if invalid
==================
*/
pub fn ClientNumberFromString(to: *mut gentity_t, s: *const c_char) -> c_int {
    let mut cl: *mut gclient_t;
    let mut idnum: c_int;
    let mut s2: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
    let mut n2: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

    unsafe {
        // numeric values are just slot numbers
        if *s as u8 >= b'0' && *s as u8 <= b'9' {
            idnum = atoi(s);
            if idnum < 0 || idnum >= (*level).maxclients {
                gi.SendServerCommand((*to).s.number - (&g_entities[0]).s.number, b"print \"Bad client slot: %i\n\"\0".as_ptr() as *const c_char);
                return -1;
            }

            cl = &mut (*level).clients[idnum as usize];
            if (*cl).pers.connected != CON_CONNECTED {
                gi.SendServerCommand((*to).s.number - (&g_entities[0]).s.number, b"print \"Client %i is not active\n\"\0".as_ptr() as *const c_char);
                return -1;
            }
            return idnum;
        }

        // check for a name match
        SanitizeString(s as *mut c_char, s2.as_mut_ptr());
        idnum = 0;
        cl = &mut (*level).clients[0];
        while idnum < (*level).maxclients {
            if (*cl).pers.connected != CON_CONNECTED {
                idnum += 1;
                cl = cl.add(1);
                continue;
            }
            SanitizeString((*cl).pers.netname, n2.as_mut_ptr());
            if strcmp(n2.as_ptr(), s2.as_ptr()) == 0 {
                return idnum;
            }
            idnum += 1;
            cl = cl.add(1);
        }

        gi.SendServerCommand((*to).s.number - (&g_entities[0]).s.number, b"print \"User %s is not on the server\n\"\0".as_ptr() as *const c_char);
        -1
    }
}

/*
==================
Cmd_Give_f

Give items to a client
==================
*/
pub fn Cmd_Give_f(ent: *mut gentity_t) {
    let mut name: *const c_char;
    let mut it: *mut gitem_t;
    let mut i: c_int;
    let mut give_all: bool;

    unsafe {
        if !CheatsOk(ent) {
            return;
        }

        name = ConcatArgs(1);

        if Q_stricmp(name, b"all\0".as_ptr() as *const c_char) == 0 {
            give_all = true;
        } else {
            give_all = false;
        }

        if give_all || Q_stricmp(name, b"force\0".as_ptr() as *const c_char) == 0 {
            if !(*ent).client.is_null() {
                (*(*ent).client).ps.forcePower = FORCE_POWER_MAX;
            }
            if !give_all {
                return;
            }
        }

        if give_all || Q_stricmp(gi.argv(1), b"health\0".as_ptr() as *const c_char) == 0 {
            if gi.argc() == 3 {
                (*ent).health = atoi(gi.argv(2));
                if (*ent).health > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] {
                    (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
                }
            } else {
                (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
            }
            if !give_all {
                return;
            }
        }

        /*	if (give_all || Q_stricmp(name, "inventory") == 0)
        {
            // Huh?  Was doing a INV_MAX+1 which was wrong because then you'd actually have every inventory item including INV_MAX
            ent->client->ps.stats[STAT_ITEMS] = (1 << (INV_MAX)) - ( 1 << INV_ELECTROBINOCULARS );

            ent->client->ps.inventory[INV_ELECTROBINOCULARS] = 1;
            //ent->client->ps.inventory[INV_BACTA_CANISTER] = 5;
            //ent->client->ps.inventory[INV_SEEKER] = 5;
            ent->client->ps.inventory[INV_LIGHTAMP_GOGGLES] = 1;
            //ent->client->ps.inventory[INV_SENTRY] = 5;
            //ent->client->ps.inventory[INV_GOODIE_KEY] = 5;
            //ent->client->ps.inventory[INV_SECURITY_KEY] = 5;

            if (!give_all)
            {
                return;
            }
        }
        */
        if give_all || Q_stricmp(name, b"weapons\0".as_ptr() as *const c_char) == 0 {
            (*(*ent).client).ps.stats[STAT_WEAPONS as usize] = (1 << (WP_MELEE)) - (1 << WP_NONE);
            if !give_all {
                return;
            }
        }

        if !give_all && Q_stricmp(gi.argv(1), b"weaponnum\0".as_ptr() as *const c_char) == 0 {
            (*(*ent).client).ps.stats[STAT_WEAPONS as usize] |= 1 << atoi(gi.argv(2));
            return;
        }

        if Q_stricmp(name, b"eweaps\0".as_ptr() as *const c_char) == 0 {	//for developing, gives you all the weapons, including enemy
            (*(*ent).client).ps.stats[STAT_WEAPONS as usize] = ((1 as c_int) << WP_NUM_WEAPONS) as u32 as c_int - (1 << WP_NONE) as u32 as c_int; // NOTE: this wasn't giving the last weapon in the list
            if !give_all {
                return;
            }
        }

        if give_all || Q_stricmp(name, b"ammo\0".as_ptr() as *const c_char) == 0 {
            i = 0;
            while i < AMMO_MAX {
                (*(*ent).client).ps.ammo[i as usize] = ammoData[i as usize].max;
                i += 1;
            }
            if !give_all {
                return;
            }
        }

        if give_all || Q_stricmp(gi.argv(1), b"batteries\0".as_ptr() as *const c_char) == 0 {
            if gi.argc() == 3 {
                (*(*ent).client).ps.batteryCharge = atoi(gi.argv(2));
            } else {
                (*(*ent).client).ps.batteryCharge = MAX_BATTERIES;
            }

            if !give_all {
                return;
            }
        }

        if give_all || Q_stricmp(gi.argv(1), b"armor\0".as_ptr() as *const c_char) == 0 {
            if gi.argc() == 3 {
                (*(*ent).client).ps.stats[STAT_ARMOR as usize] = atoi(gi.argv(2));
            } else {
                (*(*ent).client).ps.stats[STAT_ARMOR as usize] = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
            }

            if (*(*ent).client).ps.stats[STAT_ARMOR as usize] > 0 {
                (*(*ent).client).ps.powerups[PW_BATTLESUIT as usize] = Q3_INFINITE;
            } else {
                (*(*ent).client).ps.powerups[PW_BATTLESUIT as usize] = 0;
            }

            if !give_all {
                return;
            }
        }

        // spawn a specific item right on the player
        if !give_all {
            let mut it_ent: *mut gentity_t;
            let mut trace: trace_t = core::mem::zeroed();
            it = FindItem(name);
            if it.is_null() {
                name = gi.argv(1);
                it = FindItem(name);
                if it.is_null() {
                    gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"unknown item\n\"\0".as_ptr() as *const c_char);
                    return;
                }
            }

            it_ent = G_Spawn();
            VectorCopy(&(*ent).currentOrigin, &mut (*it_ent).s.origin);
            (*it_ent).classname = G_NewString((*it).classname);
            G_SpawnItem(it_ent, it);
            FinishSpawningItem(it_ent);
            memset(&mut trace as *mut trace_t as *mut c_void, 0, core::mem::size_of::<trace_t>());
            Touch_Item(it_ent, ent, &mut trace);
            if (*it_ent).inuse {
                G_FreeEntity(it_ent);
            }
        }
    }
}

//------------------
pub fn Cmd_Fx(ent: *mut gentity_t) {
    let mut dir: [f32; 3] = [0.0; 3];
    let mut fx_ent: *mut gentity_t = core::ptr::null_mut();

    unsafe {
        if Q_stricmp(gi.argv(1), b"play\0".as_ptr() as *const c_char) == 0 {
            if gi.argc() == 3 {
                // I guess, only allow one active at a time
                loop {
                    fx_ent = G_Find(fx_ent, FOFS_classname, b"cmd_fx\0".as_ptr() as *const c_char);
                    if fx_ent.is_null() {
                        break;
                    }
                    G_FreeEntity(fx_ent);
                }

                fx_ent = G_Spawn();

                (*fx_ent).fxFile = gi.argv(2);

                // Move out in front of the person spawning the effect
                AngleVectors(&(*ent).currentAngles, &mut dir, core::ptr::null_mut(), core::ptr::null_mut());
                VectorMA(&(*ent).currentOrigin, 32.0, &dir, &mut (*fx_ent).s.origin);

                SP_fx_runner(fx_ent);
                (*fx_ent).delay = 2000;			// adjusting delay
                (*fx_ent).classname = b"cmd_fx\0".as_ptr() as *const c_char;	//	and classname

                return;
            }
        } else if Q_stricmp(gi.argv(1), b"stop\0".as_ptr() as *const c_char) == 0 {
            loop {
                fx_ent = G_Find(fx_ent, FOFS_classname, b"cmd_fx\0".as_ptr() as *const c_char);
                if fx_ent.is_null() {
                    break;
                }
                G_FreeEntity(fx_ent);
            }

            return;
        } else if Q_stricmp(gi.argv(1), b"delay\0".as_ptr() as *const c_char) == 0 {
            loop {
                fx_ent = G_Find(fx_ent, FOFS_classname, b"cmd_fx\0".as_ptr() as *const c_char);
                if fx_ent.is_null() {
                    break;
                }
                if gi.argc() == 3 {
                    (*fx_ent).delay = atoi(gi.argv(2));
                } else {
                    gi.Printf(b"FX: current delay is: %i\n\0".as_ptr() as *const c_char);
                }

                return;
            }
        } else if Q_stricmp(gi.argv(1), b"random\0".as_ptr() as *const c_char) == 0 {
            loop {
                fx_ent = G_Find(fx_ent, FOFS_classname, b"cmd_fx\0".as_ptr() as *const c_char);
                if fx_ent.is_null() {
                    break;
                }
                if gi.argc() == 3 {
                    (*fx_ent).random = atoi(gi.argv(2)) as f32;
                } else {
                    gi.Printf(b"FX: current random is: %6.2f\n\0".as_ptr() as *const c_char);
                }

                return;
            }
        } else if Q_stricmp(gi.argv(1), b"origin\0".as_ptr() as *const c_char) == 0 {
            loop {
                fx_ent = G_Find(fx_ent, FOFS_classname, b"cmd_fx\0".as_ptr() as *const c_char);
                if fx_ent.is_null() {
                    break;
                }
                if gi.argc() == 5 {
                    (*fx_ent).s.origin[0] = atof(gi.argv(2));
                    (*fx_ent).s.origin[1] = atof(gi.argv(3));
                    (*fx_ent).s.origin[2] = atof(gi.argv(4));

                    G_SetOrigin(fx_ent, &(*fx_ent).s.origin);
                } else {
                    gi.Printf(b"FX: current origin is: <%6.2f %6.2f %6.2f>\n\0".as_ptr() as *const c_char);
                }

                return;
            }
        } else if Q_stricmp(gi.argv(1), b"dir\0".as_ptr() as *const c_char) == 0 {
            loop {
                fx_ent = G_Find(fx_ent, FOFS_classname, b"cmd_fx\0".as_ptr() as *const c_char);
                if fx_ent.is_null() {
                    break;
                }
                if gi.argc() == 5 {
                    (*fx_ent).s.angles[0] = atof(gi.argv(2));
                    (*fx_ent).s.angles[1] = atof(gi.argv(3));
                    (*fx_ent).s.angles[2] = atof(gi.argv(4));

                    if VectorNormalize(&mut (*fx_ent).s.angles) == 0.0 {
                        // must have been zero length
                        (*fx_ent).s.angles[2] = 1.0;
                    }
                } else {
                    gi.Printf(b"FX: current dir is: <%6.2f %6.2f %6.2f>\n\0".as_ptr() as *const c_char);
                }

                return;
            }
        }

        gi.Printf(b"Fx--------------------------------------------------------\n\0".as_ptr() as *const c_char);
        gi.Printf(b"commands:              sample usage:\n\0".as_ptr() as *const c_char);
        gi.Printf(b"----------------------------------------------------------\n\0".as_ptr() as *const c_char);
        gi.Printf(b"fx play <filename>     fx play sparks, fx play env/fire\n\0".as_ptr() as *const c_char);
        gi.Printf(b"fx stop                fx stop\n\0".as_ptr() as *const c_char);
        gi.Printf(b"fx delay <#>           fx delay 1000\n\0".as_ptr() as *const c_char);
        gi.Printf(b"fx random <#>          fx random 200\n\0".as_ptr() as *const c_char);
        gi.Printf(b"fx origin <#><#><#>    fx origin 10 20 30\n\0".as_ptr() as *const c_char);
        gi.Printf(b"fx dir <#><#><#>       fx dir 0 0 -1\n\n\0".as_ptr() as *const c_char);
    }
}

/*
==================
Cmd_God_f

Sets client to godmode

argv(0) god
==================
*/
pub fn Cmd_God_f(ent: *mut gentity_t) {
    let mut msg: *const c_char;

    unsafe {
        if !CheatsOk(ent) {
            return;
        }

        (*ent).flags ^= FL_GODMODE as u32;
        if ((*ent).flags & (FL_GODMODE as u32)) == 0 {
            msg = b"godmode OFF\n\0".as_ptr() as *const c_char;
        } else {
            msg = b"godmode ON\n\0".as_ptr() as *const c_char;
        }

        gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"%s\"\0".as_ptr() as *const c_char);
    }
}

/*
==================
Cmd_Undying_f

Sets client to undead mode

argv(0) undying
==================
*/
pub fn Cmd_Undying_f(ent: *mut gentity_t) {
    let mut msg: *const c_char;

    unsafe {
        if !CheatsOk(ent) {
            return;
        }

        (*ent).flags ^= FL_UNDYING as u32;
        if ((*ent).flags & (FL_UNDYING as u32)) == 0 {
            msg = b"undead mode OFF\n\0".as_ptr() as *const c_char;
        } else {
            let mut max: c_int;
            let mut cmd: *const c_char;

            cmd = gi.argv(1);
            if !cmd.is_null() && atoi(cmd) != 0 {
                max = atoi(cmd);
            } else {
                max = 999;
            }

            (*ent).health = (*ent).max_health = max;

            msg = b"undead mode ON\n\0".as_ptr() as *const c_char;

            if !(*ent).client.is_null() {
                (*(*ent).client).ps.stats[STAT_HEALTH as usize] = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] = 999;
            }
        }

        gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"%s\"\0".as_ptr() as *const c_char);
    }
}

/*
==================
Cmd_Notarget_f

Sets client to notarget

argv(0) notarget
==================
*/
pub fn Cmd_Notarget_f(ent: *mut gentity_t) {
    let mut msg: *const c_char;

    unsafe {
        if !CheatsOk(ent) {
            return;
        }

        (*ent).flags ^= FL_NOTARGET as u32;
        if ((*ent).flags & (FL_NOTARGET as u32)) == 0 {
            msg = b"notarget OFF\n\0".as_ptr() as *const c_char;
        } else {
            msg = b"notarget ON\n\0".as_ptr() as *const c_char;
        }

        gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"%s\"\0".as_ptr() as *const c_char);
    }
}


/*
==================
Cmd_Noclip_f

argv(0) noclip
==================
*/
pub fn Cmd_Noclip_f(ent: *mut gentity_t) {
    let mut msg: *const c_char;

    unsafe {
        if !CheatsOk(ent) {
            return;
        }

        if (*(*ent).client).noclip != 0 {
            msg = b"noclip OFF\n\0".as_ptr() as *const c_char;
        } else {
            msg = b"noclip ON\n\0".as_ptr() as *const c_char;
        }
        (*(*ent).client).noclip ^= 1;

        gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"%s\"\0".as_ptr() as *const c_char);
    }
}


/*
==================
Cmd_LevelShot_f

This is just to help generate the level pictures
for the menus.  It goes to the intermission immediately
and sends over a command to the client to resize the view,
hide the scoreboard, and take a special screenshot
==================
*/
pub fn Cmd_LevelShot_f(ent: *mut gentity_t) {
    unsafe {
        if !CheatsOk(ent) {
            return;
        }

        gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"clientLevelShot\0".as_ptr() as *const c_char);
    }
}


/*
=================
Cmd_Kill_f
=================
*/
pub fn Cmd_Kill_f(ent: *mut gentity_t) {
    unsafe {
        if (level.time - (*(*ent).client).respawnTime) < 5000 {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"cp @SP_INGAME_ONE_KILL_PER_5_SECONDS\0".as_ptr() as *const c_char);
            return;
        }
        (*ent).flags &= !(FL_GODMODE as u32);
        (*(*ent).client).ps.stats[STAT_HEALTH as usize] = (*ent).health = 0;
        player_die(ent, ent, ent, 100000, MOD_SUICIDE);
    }
}


/*
==================
Cmd_Where_f
==================
*/
pub fn Cmd_Where_f(ent: *mut gentity_t) {
    let mut s: *const c_char = unsafe { gi.argv(1) };
    let len = unsafe { strlen(s) };
    let mut check: *mut gentity_t;
    let mut i: c_int;

    unsafe {
        if gi.argc() < 2 {
            gi.Printf(b"usage: where classname\n\0".as_ptr() as *const c_char);
            return;
        }
        i = 0;
        while i < (*level).num_entities {
            if !PInUse(i) {
                i += 1;
                continue;
            }
            check = &mut g_entities[i as usize];
            if Q_stricmpn(s, (*check).classname, len) == 0 {
                gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"%s %s\n\"\0".as_ptr() as *const c_char);
            }
            i += 1;
        }
    }
}


/*
-------------------------
UserSpawn
-------------------------
*/

pub fn UserSpawn(ent: *mut gentity_t, name: *const c_char) {
    let mut origin: [f32; 3] = [0.0; 3];
    let mut vf: [f32; 3] = [0.0; 3];
    let mut angles: [f32; 3] = [0.0; 3];
    let mut ent2: *mut gentity_t;

    unsafe {
        //Spawn the ent
        ent2 = G_Spawn();
        (*ent2).classname = G_NewString(name);

        //TODO: This should ultimately make sure this is a safe spawn!

        //Spawn the entity and place it there
        VectorSet(&mut angles, 0.0, (*ent).s.apos.trBase[YAW], 0.0);
        AngleVectors(&angles, &mut vf, core::ptr::null_mut(), core::ptr::null_mut());
        VectorMA(&(*ent).s.pos.trBase, 96.0, &vf, &mut origin);	//FIXME: Find the radius size of the object, and push out 32 + radius

        origin[2] += 8.0;
        VectorCopy(&origin, &mut (*ent2).s.pos.trBase);
        VectorCopy(&origin, &mut (*ent2).s.origin);
        VectorCopy(&(*ent).s.apos.trBase, &mut (*ent2).s.angles);

        gi.linkentity(ent2);

        //Find a valid spawning spot
        if G_CallSpawn(ent2) == false {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Failed to spawn '%s'\n\"\0".as_ptr() as *const c_char);
            G_FreeEntity(ent2);
            return;
        }
    }
}

/*
-------------------------
Cmd_Spawn
-------------------------
*/

pub fn Cmd_Spawn(ent: *mut gentity_t) {
    let mut name: *const c_char;

    unsafe {
        name = ConcatArgs(1);

        gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Spawning '%s'\n\"\0".as_ptr() as *const c_char);

        UserSpawn(ent, name);
    }
}

/*
=================
Cmd_SetViewpos_f
=================
*/
pub fn Cmd_SetViewpos_f(ent: *mut gentity_t) {
    let mut origin: [f32; 3] = [0.0; 3];
    let mut angles: [f32; 3] = [0.0; 3];
    let mut i: c_int;

    unsafe {
        if (*g_cheats).integer == 0 {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char);
            return;
        }
        if gi.argc() != 5 {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"usage: setviewpos x y z yaw\n\"\0".as_ptr() as *const c_char);
            return;
        }

        VectorClear(&mut angles);
        i = 0;
        while i < 3 {
            origin[i as usize] = atof(gi.argv(i + 1));
            i += 1;
        }
        origin[2] -= 25.0;	//acount for eye height from viewpos cmd

        angles[YAW] = atof(gi.argv(4));

        TeleportPlayer(ent, &origin, &angles);
    }
}



/*
=================
Cmd_SetObjective_f
=================
*/
pub fn Cmd_SetObjective_f(ent: *mut gentity_t) {
    let mut objectiveI: c_int;
    let mut status: c_int;
    let mut displayStatus: c_int;

    unsafe {
        if gi.argc() == 2 {
            objectiveI = atoi(gi.argv(1));
            gi.Printf(b"objective #%d  display status=%d, status=%d\n\0".as_ptr() as *const c_char);
            return;
        }
        if gi.argc() != 4 {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"usage: setobjective <objective #>  <display status> <status>\n\"\0".as_ptr() as *const c_char);
            return;
        }

        if !CheatsOk(ent) {
            return;
        }

        objectiveI = atoi(gi.argv(1));
        displayStatus = atoi(gi.argv(2));
        status = atoi(gi.argv(3));

        (*(*ent).client).sess.mission_objectives[objectiveI as usize].display = displayStatus;
        (*(*ent).client).sess.mission_objectives[objectiveI as usize].status = status;
        G_CheckPlayerDarkSide();
    }
}

/*
=================
Cmd_ViewObjective_f
=================
*/
pub fn Cmd_ViewObjective_f(ent: *mut gentity_t) {
    let mut objectiveI: c_int;

    unsafe {
        if gi.argc() != 2 {
            gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"usage: viewobjective <objective #>\n\"\0".as_ptr() as *const c_char);
            return;
        }

        objectiveI = atoi(gi.argv(1));

        gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Objective %d   Display Status(1=show): %d  Status:%d\n\"\0".as_ptr() as *const c_char);
    }
}


/*
================
Cmd_UseElectrobinoculars_f
================
*/
pub fn Cmd_UseElectrobinoculars_f(ent: *mut gentity_t) {
    unsafe {
        if (*ent).health < 1 || in_camera {
            return;
        }

        if (*(*ent).client).ps.inventory[INV_ELECTROBINOCULARS as usize] <= 0 {
            // have none to place...play sound?
            return;
        }

        G_AddEvent(ent, EV_USE_INV_BINOCULARS, 0);
    }
}

/*
================
Cmd_UseBacta_f
================
*/
pub fn Cmd_UseBacta_f(ent: *mut gentity_t) {
    unsafe {
        if (*ent).health < 1 || in_camera {
            return;
        }

        ItemUse_Bacta(ent);
    }
}

//----------------------------------------------------------------------------------
pub fn PickSeekerSpawnPoint(org: *const [f32; 3], fwd: *const [f32; 3], right: *const [f32; 3], skip: c_int, spot: *mut [f32; 3]) -> bool {
    let mut mins: [f32; 3] = [0.0; 3];
    let mut maxs: [f32; 3] = [0.0; 3];
    let mut forward: [f32; 3] = [0.0; 3];
    let mut end: [f32; 3] = [0.0; 3];
    let mut tr: trace_t = unsafe { core::mem::zeroed() };

    unsafe {
        VectorSet(&mut maxs, -8.0, -8.0, -24.0); // ?? size
        VectorSet(&mut maxs, 8.0, 8.0, 8.0);

        VectorCopy(fwd, &mut forward);

        // to the front and side a bit
        forward[2] = 0.3; // start up a bit

        VectorMA(org, 48.0, &forward, &mut end);
        VectorMA(&end, -8.0, right, &mut end);

        gi.trace(&mut tr, org, &mins, &maxs, &end, skip, MASK_PLAYERSOLID);

        if !tr.startsolid && !tr.allsolid && tr.fraction >= 1.0 {
            VectorCopy(&tr.endpos, spot);
            return true;
        }

        // side
        VectorMA(org, 48.0, right, &mut end);

        gi.trace(&mut tr, org, &mins, &maxs, &end, skip, MASK_PLAYERSOLID);

        if !tr.startsolid && !tr.allsolid && tr.fraction >= 1.0 {
            VectorCopy(&tr.endpos, spot);
            return true;
        }

        // other side
        VectorMA(org, -48.0, right, &mut end);

        gi.trace(&mut tr, org, &mins, &maxs, &end, skip, MASK_PLAYERSOLID);

        if !tr.startsolid && !tr.allsolid && tr.fraction >= 1.0 {
            VectorCopy(&tr.endpos, spot);
            return true;
        }

        // behind
        VectorMA(org, -48.0, fwd, &mut end);

        gi.trace(&mut tr, org, &mins, &maxs, &end, skip, MASK_PLAYERSOLID);

        if !tr.startsolid && !tr.allsolid && tr.fraction >= 1.0 {
            VectorCopy(&tr.endpos, spot);
            return true;
        }

        false
    }
}

/*
================
Cmd_UseSeeker_f
================
*/
pub fn Cmd_UseSeeker_f(ent: *mut gentity_t) {
    unsafe {
        if (*ent).health < 1 || in_camera {
            return;
        }

        // don't use them if we don't have any...also don't use them if one is already going
        if !(*ent).client.is_null() && (*(*ent).client).ps.inventory[INV_SEEKER as usize] > 0 && level.time > (*(*ent).client).ps.powerups[PW_SEEKER as usize] {
            let mut tent: *mut gentity_t = G_Spawn();

            if !tent.is_null() {
                let mut fwd: [f32; 3] = [0.0; 3];
                let mut right: [f32; 3] = [0.0; 3];
                let mut spot: [f32; 3] = [0.0; 3];

                AngleVectors(&(*(*ent).client).ps.viewangles, &mut fwd, &mut right, core::ptr::null_mut());

                VectorCopy(&(*ent).currentOrigin, &mut spot); // does nothing really, just initialize the goods...

                if PickSeekerSpawnPoint(&(*ent).currentOrigin, &fwd, &right, (*ent).s.number, &mut spot) {
                    VectorCopy(&spot, &mut (*tent).s.origin);
                    G_SetOrigin(tent, &spot);
                    G_SetAngles(tent, &(*ent).currentAngles);

                    SP_NPC_Droid_Seeker(tent);
                    G_Sound(tent, G_SoundIndex(b"sound/chars/seeker/misc/hiss\0".as_ptr() as *const c_char));

                    // make sure that we even have some
                    (*(*ent).client).ps.inventory[INV_SEEKER as usize] -= 1;
                    (*(*ent).client).ps.powerups[PW_SEEKER as usize] = level.time + 1000;// can only drop one every second..maybe this is annoying?
                }
            }
        }
    }
}

/*
================
Cmd_UseGoggles_f
================
*/
pub fn Cmd_UseGoggles_f(ent: *mut gentity_t) {
    unsafe {
        if (*ent).health < 1 || in_camera {
            return;
        }

        if !(*ent).client.is_null() && (*(*ent).client).ps.inventory[INV_LIGHTAMP_GOGGLES as usize] > 0 {
            G_AddEvent(ent, EV_USE_INV_LIGHTAMP_GOGGLES, 0);
        }
    }
}

/*
================
Cmd_UseSentry_f
================
*/
pub fn Cmd_UseSentry_f(ent: *mut gentity_t) {
    unsafe {
        if (*ent).health < 1 || in_camera {
            return;
        }

        if (*(*ent).client).ps.inventory[INV_SENTRY as usize] <= 0 {
            // have none to place...play sound?
            return;
        }

        if place_portable_assault_sentry(ent, &(*ent).currentOrigin, &(*(*ent).client).ps.viewangles) {
            (*(*ent).client).ps.inventory[INV_SENTRY as usize] -= 1;
            G_AddEvent(ent, EV_USE_INV_SENTRY, 0);
        } else {
            // couldn't be placed....play a notification sound!!
        }
    }
}

/*
================
Cmd_UseInventory_f
================
*/
pub fn Cmd_UseInventory_f(ent: *mut gentity_t) {
    unsafe {
        match cg.inventorySelect {
            INV_ELECTROBINOCULARS => {
                Cmd_UseElectrobinoculars_f(ent);
                return;
            }
            //case INV_BACTA_CANISTER :
            //	Cmd_UseBacta_f(ent);
            //	return;
            INV_SEEKER => {
                Cmd_UseSeeker_f(ent);
                return;
            }
            INV_LIGHTAMP_GOGGLES => {
                Cmd_UseGoggles_f(ent);
                return;
            }
            INV_SENTRY => {
                Cmd_UseSentry_f(ent);
                return;
            }
            _ => {
                return;
            }
        }
    }
}

pub fn Cmd_FlushCamFile_f(ent: *mut gentity_t) {
    unsafe {
        gi.FlushCamFile();
    }
}

pub fn G_Taunt(ent: *mut gentity_t) {
    unsafe {
        if !(*ent).client.is_null() {
            if (*(*ent).client).ps.weapon == WP_SABER
                && ((*(*ent).client).ps.saberAnimLevel == SS_STAFF //ent->client->ps.saber[0].type == SABER_STAFF
                    || (*(*ent).client).ps.dualSabers != 0) {
                (*(*ent).client).ps.taunting = level.time + 100;
                //make sure all sabers are on
                (*(*ent).client).ps.SaberActivate();
            } else {
                (*(*ent).client).ps.taunting = level.time + 100;
            }
        }
    }
}

pub fn G_Victory(ent: *mut gentity_t) {
    unsafe {
        if (*ent).health > 0 {
            //say something and put away saber
            G_SoundOnEnt(ent, CHAN_VOICE, b"sound/chars/kyle/misc/taunt1.wav\0".as_ptr() as *const c_char);
            if !(*ent).client.is_null() {
                (*(*ent).client).ps.SaberDeactivate();
            }
        }
    }
}

#[repr(C)]
pub enum taunt_type_t {
    TAUNT_TAUNT = 0,
    TAUNT_BOW,
    TAUNT_MEDITATE,
    TAUNT_FLOURISH,
    TAUNT_GLOAT,
}

pub fn G_TauntSound(ent: *mut gentity_t, taunt: c_int) {
    unsafe {
        match taunt {
            0 => { // TAUNT_TAUNT
                if Q_irand(0, 1) != 0 {
                    G_SpeechEvent(ent, Q_irand(EV_ANGER1, EV_ANGER3));
                } else {
                    G_SpeechEvent(ent, Q_irand(EV_TAUNT1, EV_TAUNT3));
                }
            }
            1 => { // TAUNT_BOW
            }
            2 => { // TAUNT_MEDITATE
            }
            3 => { // TAUNT_FLOURISH
                if Q_irand(0, 1) != 0 {
                    G_SpeechEvent(ent, Q_irand(EV_DEFLECT1, EV_DEFLECT3));
                } else {
                    G_SpeechEvent(ent, Q_irand(EV_GLOAT1, EV_GLOAT3));
                }
            }
            4 => { // TAUNT_GLOAT
                G_SpeechEvent(ent, Q_irand(EV_VICTORY1, EV_VICTORY3));
            }
            _ => {}
        }
    }
}

pub fn G_SetTauntAnim(ent: *mut gentity_t, taunt: c_int) {
    unsafe {
        if ent.is_null() || (*ent).client.is_null() {
            return;
        }
        if (*(*ent).client).ps.torsoAnimTimer == 0
            && (*(*ent).client).ps.legsAnimTimer == 0
            && (*(*ent).client).ps.weaponTime == 0
            && (*(*ent).client).ps.saberLockTime < level.time {
            let mut anim: c_int = -1;
            match taunt {
                0 => { // TAUNT_TAUNT
                    if (*(*ent).client).ps.weapon != WP_SABER {
                        anim = BOTH_ENGAGETAUNT;
                    } else if (*(*ent).client).ps.saber[0].tauntAnim != -1 {
                        anim = (*(*ent).client).ps.saber[0].tauntAnim;
                    } else if (*(*ent).client).ps.dualSabers != 0
                        && (*(*ent).client).ps.saber[1].tauntAnim != -1 {
                        anim = (*(*ent).client).ps.saber[1].tauntAnim;
                    } else {
                        match (*(*ent).client).ps.saberAnimLevel {
                            SS_FAST | SS_TAVION => {
                                if (*(*ent).client).ps.saber[1].Active() {
                                    //turn off second saber
                                    G_Sound(ent, (*(*ent).client).ps.saber[1].soundOff);
                                } else if (*(*ent).client).ps.saber[0].Active() {
                                    //turn off first
                                    G_Sound(ent, (*(*ent).client).ps.saber[0].soundOff);
                                }
                                (*(*ent).client).ps.SaberDeactivate();
                                anim = BOTH_GESTURE1;
                            }
                            SS_MEDIUM | SS_STRONG | SS_DESANN => {
                                anim = BOTH_ENGAGETAUNT;
                            }
                            SS_DUAL => {
                                (*(*ent).client).ps.SaberActivate();
                                anim = BOTH_DUAL_TAUNT;
                            }
                            SS_STAFF => {
                                (*(*ent).client).ps.SaberActivate();
                                anim = BOTH_STAFF_TAUNT;
                            }
                            _ => {}
                        }
                    }
                }
                1 => { // TAUNT_BOW
                    if (*(*ent).client).ps.saber[0].bowAnim != -1 {
                        anim = (*(*ent).client).ps.saber[0].bowAnim;
                    } else if (*(*ent).client).ps.dualSabers != 0
                        && (*(*ent).client).ps.saber[1].bowAnim != -1 {
                        anim = (*(*ent).client).ps.saber[1].bowAnim;
                    } else {
                        anim = BOTH_BOW;
                    }
                    if (*(*ent).client).ps.saber[1].Active() {
                        //turn off second saber
                        G_Sound(ent, (*(*ent).client).ps.saber[1].soundOff);
                    } else if (*(*ent).client).ps.saber[0].Active() {
                        //turn off first
                        G_Sound(ent, (*(*ent).client).ps.saber[0].soundOff);
                    }
                    (*(*ent).client).ps.SaberDeactivate();
                }
                2 => { // TAUNT_MEDITATE
                    if (*(*ent).client).ps.saber[0].meditateAnim != -1 {
                        anim = (*(*ent).client).ps.saber[0].meditateAnim;
                    } else if (*(*ent).client).ps.dualSabers != 0
                        && (*(*ent).client).ps.saber[1].meditateAnim != -1 {
                        anim = (*(*ent).client).ps.saber[1].meditateAnim;
                    } else {
                        anim = BOTH_MEDITATE;
                    }
                    if (*(*ent).client).ps.saber[1].Active() {
                        //turn off second saber
                        G_Sound(ent, (*(*ent).client).ps.saber[1].soundOff);
                    } else if (*(*ent).client).ps.saber[0].Active() {
                        //turn off first
                        G_Sound(ent, (*(*ent).client).ps.saber[0].soundOff);
                    }
                    (*(*ent).client).ps.SaberDeactivate();
                }
                3 => { // TAUNT_FLOURISH
                    if (*(*ent).client).ps.weapon == WP_SABER {
                        (*(*ent).client).ps.SaberActivate();
                        if (*(*ent).client).ps.saber[0].flourishAnim != -1 {
                            anim = (*(*ent).client).ps.saber[0].flourishAnim;
                        } else if (*(*ent).client).ps.dualSabers != 0
                            && (*(*ent).client).ps.saber[1].flourishAnim != -1 {
                            anim = (*(*ent).client).ps.saber[1].flourishAnim;
                        } else {
                            match (*(*ent).client).ps.saberAnimLevel {
                                SS_FAST | SS_TAVION => {
                                    anim = BOTH_SHOWOFF_FAST;
                                }
                                SS_MEDIUM => {
                                    anim = BOTH_SHOWOFF_MEDIUM;
                                }
                                SS_STRONG | SS_DESANN => {
                                    anim = BOTH_SHOWOFF_STRONG;
                                }
                                SS_DUAL => {
                                    anim = BOTH_SHOWOFF_DUAL;
                                }
                                SS_STAFF => {
                                    anim = BOTH_SHOWOFF_STAFF;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                4 => { // TAUNT_GLOAT
                    if (*(*ent).client).ps.saber[0].gloatAnim != -1 {
                        anim = (*(*ent).client).ps.saber[0].gloatAnim;
                    } else if (*(*ent).client).ps.dualSabers != 0
                        && (*(*ent).client).ps.saber[1].gloatAnim != -1 {
                        anim = (*(*ent).client).ps.saber[1].gloatAnim;
                    } else {
                        match (*(*ent).client).ps.saberAnimLevel {
                            SS_FAST | SS_TAVION => {
                                anim = BOTH_VICTORY_FAST;
                            }
                            SS_MEDIUM => {
                                anim = BOTH_VICTORY_MEDIUM;
                            }
                            SS_STRONG | SS_DESANN => {
                                (*(*ent).client).ps.SaberActivate();
                                anim = BOTH_VICTORY_STRONG;
                            }
                            SS_DUAL => {
                                (*(*ent).client).ps.SaberActivate();
                                anim = BOTH_VICTORY_DUAL;
                            }
                            SS_STAFF => {
                                (*(*ent).client).ps.SaberActivate();
                                anim = BOTH_VICTORY_STAFF;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            if anim != -1 {
                if (*(*ent).client).ps.groundEntityNum != ENTITYNUM_NONE {
                    let mut parts: c_int = SETANIM_TORSO;
                    if anim != BOTH_ENGAGETAUNT {
                        parts = SETANIM_BOTH;
                        VectorClear(&mut (*(*ent).client).ps.velocity);
                    }
                    NPC_SetAnim(ent, parts, anim, (SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD));
                }
                if taunt != 2 && taunt != 1 {
                    //no sound for meditate or bow
                    G_TauntSound(ent, taunt);
                }
            }
        }
    }
}

pub fn Cmd_SaberDrop_f(ent: *mut gentity_t, saberNum: c_int) {
    unsafe {
        if saberNum < 0 {
            return;
        }
        if saberNum > 1 {
            return;
        }
        if ent.is_null() || (*ent).client.is_null() {
            return;
        }
        if (*ent).weaponModel[saberNum as usize] <= 0 {
            return;
        }

        if (*(*ent).client).ps.weapon != WP_SABER {
            return;
        }

        if (*(*ent).client).ps.weaponTime > 0 {
            return;
        }

        if (*(*ent).client).ps.saberMove != 0 // LS_READY
            && (*(*ent).client).ps.saberMove != 1 // LS_PUTAWAY
            && (*(*ent).client).ps.saberMove != 2 // LS_DRAW
            && (*(*ent).client).ps.saberMove != -1 { // LS_NONE
            return;
        }

        if !(*g_saberPickuppableDroppedSabers).integer != 0 {
            return;
        }

        if (*(*ent).client).ps.saber[saberNum as usize].name.is_null()
            || *(*(*ent).client).ps.saber[saberNum as usize].name == 0 {
            return;
        }

        //have a valid string to use for saberType

        //turn it into a pick-uppable item!
        if !G_DropSaberItem(
                (*(*ent).client).ps.saber[saberNum as usize].name,
                (*(*ent).client).ps.saber[saberNum as usize].blade[0].color,
                if saberNum == 0 {
                    &(*(*ent).client).renderInfo.handRPoint
                } else {
                    &(*(*ent).client).renderInfo.handLPoint
                },
                &(*(*ent).client).ps.velocity,
                &(*ent).currentAngles
            ).is_null() {
            //dropped it
            WP_RemoveSaber(ent, saberNum);
        }

        if (*ent).weaponModel[0] <= 0
            && (*ent).weaponModel[1] <= 0 {
            //no sabers left!
            //remove saber from inventory
            (*(*ent).client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_SABER);
            //change weapons
            if (*ent).s.number < 64 {  // MAX_CLIENTS
                //player
                CG_ChangeWeapon(WP_NONE);
            } else {
                ChangeWeapon(ent, WP_NONE);
            }
            (*(*ent).client).ps.weapon = WP_NONE;
        }
    }
}

/*
=================
ClientCommand
=================
*/
pub fn ClientCommand(clientNum: c_int) {
    let mut ent: *mut gentity_t;
    let mut cmd: *const c_char;

    unsafe {
        ent = &mut g_entities[clientNum as usize];
        if (*ent).client.is_null() {
            return;		// not fully in game yet
        }

        cmd = gi.argv(0);

        if Q_stricmp(cmd, b"spawn\0".as_ptr() as *const c_char) == 0 {
            Cmd_Spawn(ent);
            return;
        }

        if Q_stricmp(cmd, b"give\0".as_ptr() as *const c_char) == 0 {
            Cmd_Give_f(ent);
        } else if Q_stricmp(cmd, b"god\0".as_ptr() as *const c_char) == 0 {
            Cmd_God_f(ent);
        } else if Q_stricmp(cmd, b"undying\0".as_ptr() as *const c_char) == 0 {
            Cmd_Undying_f(ent);
        } else if Q_stricmp(cmd, b"notarget\0".as_ptr() as *const c_char) == 0 {
            Cmd_Notarget_f(ent);
        } else if Q_stricmp(cmd, b"noclip\0".as_ptr() as *const c_char) == 0 {
            Cmd_Noclip_f(ent);
        } else if Q_stricmp(cmd, b"kill\0".as_ptr() as *const c_char) == 0 {
            if !CheatsOk(ent) {
                return;
            }
            Cmd_Kill_f(ent);
        } else if Q_stricmp(cmd, b"levelshot\0".as_ptr() as *const c_char) == 0 {
            Cmd_LevelShot_f(ent);
        } else if Q_stricmp(cmd, b"where\0".as_ptr() as *const c_char) == 0 {
            Cmd_Where_f(ent);
        } else if Q_stricmp(cmd, b"setviewpos\0".as_ptr() as *const c_char) == 0 {
            Cmd_SetViewpos_f(ent);
        } else if Q_stricmp(cmd, b"setobjective\0".as_ptr() as *const c_char) == 0 {
            Cmd_SetObjective_f(ent);
        } else if Q_stricmp(cmd, b"viewobjective\0".as_ptr() as *const c_char) == 0 {
            Cmd_ViewObjective_f(ent);
        } else if Q_stricmp(cmd, b"force_throw\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceThrow(ent, false);
        } else if Q_stricmp(cmd, b"force_pull\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceThrow(ent, true);
        } else if Q_stricmp(cmd, b"force_speed\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceSpeed(ent);
        } else if Q_stricmp(cmd, b"force_heal\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceHeal(ent);
        } else if Q_stricmp(cmd, b"force_grip\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceGrip(ent);
        } else if Q_stricmp(cmd, b"force_distract\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceTelepathy(ent);
        } else if Q_stricmp(cmd, b"force_rage\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceRage(ent);
        } else if Q_stricmp(cmd, b"force_protect\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceProtect(ent);
        } else if Q_stricmp(cmd, b"force_absorb\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceAbsorb(ent);
        } else if Q_stricmp(cmd, b"force_sight\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            ForceSeeing(ent);
        } else if Q_stricmp(cmd, b"addsaberstyle\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            if ent.is_null() || (*ent).client.is_null() {
                //wtf?
                return;
            }
            if gi.argc() < 2 {
                gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"usage: addsaberstyle <saber style>\n\"\0".as_ptr() as *const c_char);
                gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Valid styles: SS_FAST, SS_MEDIUM, SS_STRONG, SS_DESANN, SS_TAVION, SS_DUAL and SS_STAFF\n\"\0".as_ptr() as *const c_char);
                return;
            }

            let addStyle = GetIDForString(&SaberStyleTable[0], gi.argv(1));
            if addStyle > SS_NONE && addStyle < SS_STAFF {
                (*(*ent).client).ps.saberStylesKnown |= 1 << addStyle;
            }
        } else if Q_stricmp(cmd, b"setsaberstyle\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            if ent.is_null() || (*ent).client.is_null() {
                //wtf?
                return;
            }
            if gi.argc() < 2 {
                gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"usage: setsaberstyle <saber style>\n\"\0".as_ptr() as *const c_char);
                gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Valid styles: SS_FAST, SS_MEDIUM, SS_STRONG, SS_DESANN, SS_TAVION, SS_DUAL and SS_STAFF\n\"\0".as_ptr() as *const c_char);
                return;
            }

            let setStyle = GetIDForString(&SaberStyleTable[0], gi.argv(1));
            if setStyle > SS_NONE && setStyle < SS_STAFF {
                (*(*ent).client).ps.saberStylesKnown = 1 << setStyle;
                cg.saberAnimLevelPending = (*(*ent).client).ps.saberAnimLevel = setStyle;
            }
        } else if Q_stricmp(cmd, b"taunt\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            //		G_Taunt( ent );
            G_SetTauntAnim(ent, 0); // TAUNT_TAUNT
        } else if Q_stricmp(cmd, b"bow\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            G_SetTauntAnim(ent, 1); // TAUNT_BOW
        } else if Q_stricmp(cmd, b"meditate\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            G_SetTauntAnim(ent, 2); // TAUNT_MEDITATE
        } else if Q_stricmp(cmd, b"flourish\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            G_SetTauntAnim(ent, 3); // TAUNT_FLOURISH
        } else if Q_stricmp(cmd, b"gloat\0".as_ptr() as *const c_char) == 0 {
            ent = G_GetSelfForPlayerCmd();
            G_SetTauntAnim(ent, 4); // TAUNT_GLOAT
        }
        /*
        else if (Q_stricmp (cmd, "drive") == 0)
        {
            if ( !CheatsOk( ent ) )
            {
                return;
            }
            if ( gi.argc() < 2 )
            {
                gi.SendServerCommand( ent-g_entities, va("print \"usage: drive <vehicle name>\n\""));
                gi.SendServerCommand( ent-g_entities, va("print \"Vehicles will be in vehicles.cfg, try using 'speeder' for now\n\""));
                return;
            }
            G_DriveVehicle( ent, NULL, gi.argv(1) );
        }
        */
        else if Q_stricmp(cmd, b"NPCdrive\0".as_ptr() as *const c_char) == 0 {
            if !CheatsOk(ent) {
                return;
            }
            if gi.argc() < 3 {
                gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"usage: drive <NPC_targetname> <vehicle name>\n\"\0".as_ptr() as *const c_char);
                gi.SendServerCommand((*ent).s.number - (&g_entities[0]).s.number, b"print \"Vehicles will be in vehicles.cfg, try using 'speeder' for now\n\"\0".as_ptr() as *const c_char);
                return;
            }
            let found = G_Find(core::ptr::null_mut(), FOFS_targetname, gi.argv(1));
            if !found.is_null() && (*found).health > 0 && !(*found).client.is_null() {
                // TEMPORARY! BRING BACK LATER!!!
                //G_DriveVehicle( found, NULL, gi.argv(2) );
            }
        } else if Q_stricmp(cmd, b"thereisnospoon\0".as_ptr() as *const c_char) == 0 {
            G_StartMatrixEffect(ent, 0, 0, 0.0, 0);
        } else if Q_stricmp(cmd, b"use_electrobinoculars\0".as_ptr() as *const c_char) == 0 {
            Cmd_UseElectrobinoculars_f(ent);
        } else if Q_stricmp(cmd, b"use_bacta\0".as_ptr() as *const c_char) == 0 {
            Cmd_UseBacta_f(ent);
        } else if Q_stricmp(cmd, b"use_seeker\0".as_ptr() as *const c_char) == 0 {
            Cmd_UseSeeker_f(ent);
        } else if Q_stricmp(cmd, b"use_lightamp_goggles\0".as_ptr() as *const c_char) == 0 {
            Cmd_UseGoggles_f(ent);
        } else if Q_stricmp(cmd, b"use_sentry\0".as_ptr() as *const c_char) == 0 {
            Cmd_UseSentry_f(ent);
        } else if Q_stricmp(cmd, b"fx\0".as_ptr() as *const c_char) == 0 {
            Cmd_Fx(ent);
        } else if Q_stricmp(cmd, b"invuse\0".as_ptr() as *const c_char) == 0 {
            Cmd_UseInventory_f(ent);
        } else if Q_stricmp(cmd, b"playmusic\0".as_ptr() as *const c_char) == 0 {
            let cmd2 = gi.argv(1);
            if !cmd2.is_null() {
                gi.SetConfigstring(CS_MUSIC, cmd2);
            }
        } else if Q_stricmp(cmd, b"flushcam\0".as_ptr() as *const c_char) == 0 {
            Cmd_FlushCamFile_f(ent);
        } else if Q_stricmp(cmd, b"dropsaber\0".as_ptr() as *const c_char) == 0 {
            let cmd2 = gi.argv(1);
            let mut saberNum: c_int = 2;//by default, drop both
            if !cmd2.is_null() && *cmd2 as u8 != 0 {
                saberNum = atoi(cmd2);
            }
            if saberNum > 1 {
                //drop both
                Cmd_SaberDrop_f(ent, 1);
                Cmd_SaberDrop_f(ent, 0);
            } else {
                //drop either left or right
                Cmd_SaberDrop_f(ent, saberNum);
            }
        }
    }
}
