// leave this line at the top for all g_xxxx.rs files...
// g_headers.h included via module system
// Q3_Interface.h
// g_local.h
// wp_saber.h

use core::ffi::{c_int, c_char};

// ============================================================================
// Extern declarations
// ============================================================================

extern "C" {
    fn G_NextTestAxes();
    fn G_ChangePlayerModel(ent: *mut gentity_t, newModel: *const c_char);
    fn G_InitPlayerFromCvars(ent: *mut gentity_t);
    fn Q3_SetViewEntity(entID: c_int, name: *const c_char);
    fn G_ClearViewEntity(ent: *mut gentity_t) -> qboolean;
    fn G_Knockdown(
        this: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const vec3_t,
        strength: f32,
        breakSaberLock: qboolean,
    );
    fn WP_SetSaber(ent: *mut gentity_t, saberNum: c_int, saberName: *mut c_char);
    fn WP_RemoveSaber(ent: *mut gentity_t, saberNum: c_int);
    fn TranslateSaberColor(name: *const c_char) -> saber_colors_t;
    fn WP_SaberBladeUseSecondBladeStyle(saber: *mut saberInfo_t, bladeNum: c_int) -> qboolean;
    fn WP_UseFirstValidSaberStyle(ent: *mut gentity_t, saberAnimLevel: *mut c_int) -> qboolean;
    fn G_SetWeapon(this: *mut gentity_t, wp: c_int);
    fn G_Find(
        from: *mut gentity_t,
        fieldofs: usize,
        match_str: *const c_char,
    ) -> *mut gentity_t;
    fn G_SoundIndexOnEnt(ent: *mut gentity_t, channel: c_int, soundName: *const c_char);
    fn GEntity_UseFunc(
        ent: *mut gentity_t,
        other: *mut gentity_t,
        activator: *mut gentity_t,
    );
    fn ExitEmplacedWeapon(ent: *mut gentity_t);
    fn G_StopCinematicSkip();
    fn G_StartCinematicSkip();
    fn Quake3Game() -> *mut cQ3_GameInterface;
    fn Svcmd_GameMem_f();
    fn Svcmd_Nav_f();
    fn Svcmd_NPC_f();
    fn Svcmd_Use_f();
    fn GetStringForID(table: *const stringID_table_t, id: c_int) -> *const c_char;
    fn GetIDForString(table: *const stringID_table_t, str: *const c_char) -> c_int;

    pub static mut g_entities: gentity_t;
    pub static mut level: level_t;
    pub static mut globals: game_globals_t;
    pub static mut player: *mut gentity_t;
    pub static mut in_camera: qboolean;
    pub static mut g_cheats: *mut cvar_t;
    pub static mut g_char_model: *mut cvar_t;
    pub static mut g_char_skin_head: *mut cvar_t;
    pub static mut g_char_skin_torso: *mut cvar_t;
    pub static mut g_char_skin_legs: *mut cvar_t;
    pub static mut g_char_color_red: *mut cvar_t;
    pub static mut g_char_color_green: *mut cvar_t;
    pub static mut g_char_color_blue: *mut cvar_t;
    pub static mut g_saber: *mut cvar_t;
    pub static mut g_saber2: *mut cvar_t;
    pub static mut g_saber_color: *mut cvar_t;
    pub static mut g_saber2_color: *mut cvar_t;
    pub static mut g_skippingcin: *mut cvar_t;
    pub static mut cg: cg_t;

    static WPTable: stringID_table_t;
    static TeamTable: stringID_table_t;

    fn Q_stricmp(s0: *const c_char, s1: *const c_char) -> c_int;
    fn atoi(str: *const c_char) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;

    // Game interface functions
    static gi: game_interface_t;
}

// ============================================================================
// Type stubs and externs for dependencies
// ============================================================================

#[repr(C)]
pub struct gentity_t {
    // Stub: actual fields defined elsewhere
    _data: [u8; 0],
}

#[repr(C)]
pub struct gclient_t {
    // Stub
    _data: [u8; 0],
}

#[repr(C)]
pub struct cvar_t {
    // Stub
    _data: [u8; 0],
}

type qboolean = c_int;
type vec3_t = [f32; 3];
type saber_colors_t = c_int;

#[repr(C)]
pub struct saberInfo_t {
    // Stub
    _data: [u8; 0],
}

#[repr(C)]
pub struct stringID_table_t {
    // Stub
    _data: [u8; 0],
}

#[repr(C)]
pub struct level_t {
    // Stub
    _data: [u8; 0],
}

#[repr(C)]
pub struct game_globals_t {
    // Stub
    _data: [u8; 0],
}

#[repr(C)]
pub struct game_interface_t {
    // Stub
    _data: [u8; 0],
}

#[repr(C)]
pub struct cg_t {
    // Stub
    _data: [u8; 0],
}

#[repr(C)]
pub struct cQ3_GameInterface {
    // Stub
    _data: [u8; 0],
}

const ENTITYNUM_WORLD: c_int = 0x7FFF;
const ENTITYNUM_NONE: c_int = 0x7FFE;
const MAX_BLADES: usize = 8;
const EF_LOCKED_TO_WEAPON: c_int = 0x00000020;
const CLASS_ATST: c_int = 0;
const WP_SABER: c_int = 1;
const WP_MELEE: c_int = 11;
const SS_NONE: c_int = -1;
const SS_FAST: c_int = 0;
const SS_MEDIUM: c_int = 1;
const SS_STRONG: c_int = 2;
const SS_DESANN: c_int = 3;
const SS_TAVION: c_int = 4;
const SS_DUAL: c_int = 5;
const SS_STAFF: c_int = 6;
const SS_NUM_SABER_STYLES: c_int = 7;
const FP_LEVITATION: c_int = 2;
const FP_SABERTHROW: c_int = 8;
const FP_HEAL: c_int = 1;
const FP_PUSH: c_int = 4;
const FP_PULL: c_int = 5;
const FP_SPEED: c_int = 6;
const FP_GRIP: c_int = 7;
const FP_LIGHTNING: c_int = 9;
const FP_TELEPATHY: c_int = 10;
const FP_SABER_DEFENSE: c_int = 11;
const FP_SABER_OFFENSE: c_int = 12;
const FP_RAGE: c_int = 13;
const FP_DRAIN: c_int = 14;
const FP_PROTECT: c_int = 15;
const FP_ABSORB: c_int = 16;
const FP_SEE: c_int = 17;
const FP_FIRST: c_int = 0;
const NUM_FORCE_POWERS: c_int = 19;
const FORCE_LEVEL_0: c_int = 0;
const FORCE_LEVEL_1: c_int = 1;
const FORCE_LEVEL_2: c_int = 2;
const FORCE_LEVEL_3: c_int = 3;
const FORCE_LEVEL_4: c_int = 4;
const CHAN_WEAPON: c_int = 0;
const CON_DISCONNECTED: c_int = 0;
const ET_GENERAL: c_int = 0;
const ET_PLAYER: c_int = 1;
const ET_ITEM: c_int = 2;
const ET_MISSILE: c_int = 3;
const ET_MOVER: c_int = 4;
const ET_BEAM: c_int = 5;
const TEAM_FREE: c_int = 0;
const TEAM_NUM_TEAMS: c_int = 3;
const SFL_TWO_HANDED: c_int = 0x00000001;
const SFL2_NO_MANUAL_DEACTIVATE: c_int = 0x00000001;
const SFL2_NO_MANUAL_DEACTIVATE2: c_int = 0x00000002;

const S_COLOR_RED: &[u8] = b"^1";
const S_COLOR_YELLOW: &[u8] = b"^3";
const S_COLOR_BLUE: &[u8] = b"^4";
const S_COLOR_CYAN: &[u8] = b"^5";
const S_COLOR_MAGENTA: &[u8] = b"^6";

const FOFS: usize = 0; // Stub for offsetof

const VALIDSTRING_MACRO: usize = 0; // placeholder

// VALIDSTRING macro stub - checks if string pointer is non-NULL and first char is not 0
#[inline]
fn VALIDSTRING(s: *const c_char) -> bool {
    if s.is_null() {
        return false;
    }
    unsafe { *s != 0 }
}

// ============================================================================
// Functions
// ============================================================================

//===================
//Svcmd_EntityList_f
//===================
#[allow(non_snake_case)]
unsafe fn Svcmd_EntityList_f() {
    let mut check = core::ptr::addr_of_mut!(g_entities).add(1);
    for e in 1..(*core::ptr::addr_of!(globals)).num_entities {
        if (*check).inuse == 0 {
            check = check.add(1);
            continue;
        }
        (*gi.Printf)("%3i:" as *const c_char, e);
        match (*check).s.eType {
            ET_GENERAL => {
                (*gi.Printf)("ET_GENERAL " as *const c_char);
            }
            ET_PLAYER => {
                (*gi.Printf)("ET_PLAYER  " as *const c_char);
            }
            ET_ITEM => {
                (*gi.Printf)("ET_ITEM    " as *const c_char);
            }
            ET_MISSILE => {
                (*gi.Printf)("ET_MISSILE " as *const c_char);
            }
            ET_MOVER => {
                (*gi.Printf)("ET_MOVER   " as *const c_char);
            }
            ET_BEAM => {
                (*gi.Printf)("ET_BEAM    " as *const c_char);
            }
            _ => {
                (*gi.Printf)("#%i" as *const c_char, (*check).s.eType);
            }
        }

        if !(*check).classname.is_null() {
            (*gi.Printf)("%s" as *const c_char, (*check).classname);
        }
        (*gi.Printf)("\n" as *const c_char);
        check = check.add(1);
    }
}

#[allow(non_snake_case)]
unsafe fn ClientForString(s: *const c_char) -> *mut gclient_t {
    let mut cl: *mut gclient_t;
    let mut i: c_int;
    let mut idnum: c_int;

    // numeric values are just slot numbers
    if *s as u8 >= b'0' && *s as u8 <= b'9' {
        idnum = atoi(s);
        if idnum < 0 || idnum >= (*core::ptr::addr_of!(level)).maxclients {
            (*gi.Printf)("Bad client slot: %i\n" as *const c_char, idnum);
            return core::ptr::null_mut();
        }

        cl = core::ptr::addr_of_mut!((*core::ptr::addr_of!(level)).clients)
            .add(idnum as usize) as *mut gclient_t;
        if (*cl).pers.connected == CON_DISCONNECTED {
            (*gi.Printf)("Client %i is not connected\n" as *const c_char, idnum);
            return core::ptr::null_mut();
        }
        return cl;
    }

    // check for a name match
    i = 0;
    while i < (*core::ptr::addr_of!(level)).maxclients {
        cl = core::ptr::addr_of_mut!((*core::ptr::addr_of!(level)).clients)
            .add(i as usize) as *mut gclient_t;
        if (*cl).pers.connected == CON_DISCONNECTED {
            i += 1;
            continue;
        }
        if Q_stricmp((*cl).pers.netname, s) == 0 {
            return cl;
        }
        i += 1;
    }

    (*gi.Printf)("User %s is not on the server\n" as *const c_char, s);

    core::ptr::null_mut()
}

//---------------------------
#[allow(non_snake_case)]
unsafe fn Svcmd_ExitView_f() {
    static mut exitViewDebounce: c_int = 0;
    if exitViewDebounce > (*core::ptr::addr_of!(level)).time {
        return;
    }
    exitViewDebounce = (*core::ptr::addr_of!(level)).time + 500;
    if in_camera != 0 {
        //see if we need to exit an in-game cinematic
        if (*g_skippingcin).integer != 0 {
            // already doing cinematic skip?
            // yes...   so stop skipping...
            G_StopCinematicSkip();
        } else {
            // no... so start skipping...
            G_StartCinematicSkip();
        }
    } else if G_ClearViewEntity(player) == 0 {
        //didn't exit control of a droid or turret
        //okay, now try exiting emplaced guns or AT-ST's
        if (*player).s.eFlags & EF_LOCKED_TO_WEAPON != 0 {
            //get out of emplaced gun
            ExitEmplacedWeapon(player);
        } else if (*player).client != core::ptr::null_mut()
            && (*(*player).client).NPC_class == CLASS_ATST
        {
            //a player trying to get out of his ATST
            GEntity_UseFunc((*player).activator, player, player);
        }
    }
}

#[allow(non_snake_case)]
unsafe fn G_GetSelfForPlayerCmd() -> *mut gentity_t {
    let player_ent = core::ptr::addr_of_mut!(g_entities);
    let player_client = (*player_ent).client;
    if !player_client.is_null() {
        let view_entity = (*player_client).ps.viewEntity;
        if view_entity > 0
            && view_entity < ENTITYNUM_WORLD
            && !core::ptr::addr_of!(g_entities)
                .add(view_entity as usize)
                .read()
                .client
                .is_null()
            && core::ptr::addr_of!(g_entities)
                .add(view_entity as usize)
                .read()
                .s
                .weapon
                == WP_SABER
        {
            //you're controlling another NPC
            return core::ptr::addr_of_mut!(g_entities).add(view_entity as usize);
        }
    }
    core::ptr::addr_of_mut!(g_entities)
}

#[allow(non_snake_case)]
unsafe fn Svcmd_Saber_f() {
    let saber = (*gi.argv)(1);
    let saber2 = (*gi.argv)(2);

    if core::ptr::addr_of_mut!(g_entities).read().client.is_null() || saber.is_null()
        || *saber == 0
    {
        return;
    }

    (*gi.cvar_set)("g_saber" as *const c_char, saber);
    WP_SetSaber(core::ptr::addr_of_mut!(g_entities), 0, saber);
    if !saber2.is_null()
        && *saber2 != 0
        && ((*(*core::ptr::addr_of_mut!(g_entities).read().client).ps.saber[0]).saberFlags
            & SFL_TWO_HANDED)
            == 0
    {
        //want to use a second saber and first one is not twoHanded
        (*gi.cvar_set)("g_saber2" as *const c_char, saber2);
        WP_SetSaber(core::ptr::addr_of_mut!(g_entities), 1, saber2);
    } else {
        (*gi.cvar_set)("g_saber2" as *const c_char, "" as *const c_char);
        WP_RemoveSaber(core::ptr::addr_of_mut!(g_entities), 1);
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberBlade_f() {
    if (*gi.argc)() < 2 {
        (*gi.Printf)(
            "USAGE: saberblade <sabernum> <bladenum> [0 = off, 1 = on, no arg = toggle]\n"
                as *const c_char,
        );
        return;
    }
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    let sabernum = atoi((*gi.argv)(1)) - 1;
    if sabernum < 0 || sabernum > 1 {
        return;
    }
    if sabernum > 0
        && (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).dualSabers == 0
    {
        return;
    }
    //FIXME: what if don't even have a single saber at all?
    let bladenum = atoi((*gi.argv)(2)) - 1;
    if bladenum < 0
        || bladenum
            >= (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps.saber[sabernum as usize])
                .numBlades
    {
        return;
    }
    let turnOn: qboolean;
    if (*gi.argc)() > 2 {
        //explicit
        turnOn = if atoi((*gi.argv)(3)) != 0 { 1 } else { 0 };
    } else {
        //toggle
        turnOn = if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps.saber
            [sabernum as usize])
            .blade[bladenum as usize]
            .active
            == 0
        {
            1
        } else {
            0
        };
    }

    (*(*core::ptr::addr_of_mut!(g_entities).read().client)
        .ps)
        .SaberBladeActivate(sabernum, bladenum, turnOn);
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberColor_f() {
    //FIXME: just list the colors, each additional listing sets that blade
    let saberNum = atoi((*gi.argv)(1));
    let mut color: [*const c_char; MAX_BLADES] = [core::ptr::null(); MAX_BLADES];
    let bladeNum: c_int;

    for bladeNum in 0..MAX_BLADES {
        color[bladeNum] = (*gi.argv)(2 + bladeNum as c_int);
    }

    if !VALIDSTRING(color[0] as *const c_char) || saberNum < 1 || saberNum > 2 {
        (*gi.Printf)(
            "Usage:  saberColor <saberNum> <blade1 color> <blade2 color> ... <blade8 color> \n"
                as *const c_char,
        );
        (*gi.Printf)("valid saberNums:  1 or 2\n" as *const c_char);
        (*gi.Printf)("valid colors:  red, orange, yellow, green, blue, and purple\n" as *const c_char);

        return;
    }
    let saberNum = saberNum - 1;

    let self_ = G_GetSelfForPlayerCmd();

    for bladeNum in 0..MAX_BLADES {
        if color[bladeNum].is_null() || *color[bladeNum] == 0 {
            break;
        } else {
            (*(*self_).client).ps.saber[saberNum as usize].blade[bladeNum].color =
                TranslateSaberColor(color[bladeNum]);
        }
    }

    if saberNum == 0 {
        (*gi.cvar_set)("g_saber_color" as *const c_char, color[0]);
    } else if saberNum == 1 {
        (*gi.cvar_set)("g_saber2_color" as *const c_char, color[0]);
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceJump_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current forceJump level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_LEVITATION as usize],
        );
        (*gi.Printf)("Usage:  setForceJump <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_LEVITATION;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_LEVITATION);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LEVITATION
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LEVITATION
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LEVITATION
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_LEVITATION as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LEVITATION
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberThrow_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current saberThrow level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_SABERTHROW as usize],
        );
        (*gi.Printf)("Usage:  setSaberThrow <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_SABERTHROW;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_SABERTHROW);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABERTHROW
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABERTHROW
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABERTHROW
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_SABERTHROW as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABERTHROW
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceHeal_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current forceHeal level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_HEAL as usize],
        );
        (*gi.Printf)("Usage:  forceHeal <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_HEAL;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_HEAL);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_HEAL as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_HEAL
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_HEAL
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_HEAL
        as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_HEAL
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForcePush_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current forcePush level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_PUSH as usize],
        );
        (*gi.Printf)("Usage:  forcePush <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_PUSH;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_PUSH);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PUSH as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PUSH
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PUSH
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PUSH
        as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PUSH
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForcePull_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current forcePull level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_PULL as usize],
        );
        (*gi.Printf)("Usage:  forcePull <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_PULL;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_PULL);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PULL as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PULL
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PULL
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PULL
        as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_PULL
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceSpeed_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current forceSpeed level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_SPEED as usize],
        );
        (*gi.Printf)("Usage:  forceSpeed <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_SPEED;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_SPEED);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SPEED
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SPEED
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SPEED
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_SPEED as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SPEED
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceGrip_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current forceGrip level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_GRIP as usize],
        );
        (*gi.Printf)("Usage:  forceGrip <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_GRIP;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_GRIP);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_GRIP as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_GRIP
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_GRIP
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_GRIP
        as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_GRIP
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceLightning_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current forceLightning level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_LIGHTNING as usize],
        );
        (*gi.Printf)("Usage:  forceLightning <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_LIGHTNING;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_LIGHTNING);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LIGHTNING
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LIGHTNING
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LIGHTNING
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_LIGHTNING as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_LIGHTNING
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_MindTrick_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current mindTrick level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_TELEPATHY as usize],
        );
        (*gi.Printf)("Usage:  mindTrick <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_TELEPATHY;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_TELEPATHY);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_TELEPATHY
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_TELEPATHY
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_TELEPATHY
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_TELEPATHY as usize]
        > FORCE_LEVEL_4
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_TELEPATHY
            as usize] = FORCE_LEVEL_4;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberDefense_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current saberDefense level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_SABER_DEFENSE as usize],
        );
        (*gi.Printf)("Usage:  saberDefense <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_SABER_DEFENSE;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_SABER_DEFENSE);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABER_DEFENSE
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_SABER_DEFENSE as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABER_DEFENSE
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_SABER_DEFENSE as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABER_DEFENSE
            as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberOffense_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current saberOffense level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[FP_SABER_OFFENSE as usize],
        );
        (*gi.Printf)("Usage:  saberOffense <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << FP_SABER_OFFENSE;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << FP_SABER_OFFENSE);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABER_OFFENSE
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_SABER_OFFENSE as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABER_OFFENSE
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[FP_SABER_OFFENSE as usize]
        >= SS_NUM_SABER_STYLES
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[FP_SABER_OFFENSE
            as usize] = SS_NUM_SABER_STYLES - 1;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceSetLevel_f(forcePower: c_int) {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (*gi.SendServerCommand)(
            0,
            "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
        );
        return;
    }
    let newVal = (*gi.argv)(1);
    if !VALIDSTRING(newVal) {
        (*gi.Printf)(
            "Current force level is %d\n" as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
                .forcePowerLevel[forcePower as usize],
        );
        (*gi.Printf)("Usage:  force <level> (1 - 3)\n" as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown |=
            1 << forcePower;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowersKnown &=
            !(1 << forcePower);
    }
    (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[forcePower
        as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[forcePower
        as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[forcePower
            as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps)
        .forcePowerLevel[forcePower as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities).read().client).ps).forcePowerLevel[forcePower
            as usize] = FORCE_LEVEL_3;
    }
}

extern "C" {
    fn PM_SaberInStart(mv: c_int) -> qboolean;
    fn PM_SaberInTransition(mv: c_int) -> qboolean;
    fn PM_SaberInAttack(mv: c_int) -> qboolean;
    fn WP_SaberCanTurnOffSomeBlades(saber: *mut saberInfo_t) -> qboolean;
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberAttackCycle_f() {
    if core::ptr::addr_of_mut!(g_entities).is_null()
        || core::ptr::addr_of_mut!(g_entities)
            .read()
            .client
            .is_null()
    {
        return;
    }

    let self_ = G_GetSelfForPlayerCmd();
    if (*self_).s.weapon != WP_SABER {
        // saberAttackCycle button also switches to saber
        (*gi.SendConsoleCommand)("weapon 1" as *const c_char);
        return;
    }

    if (*(*self_).client).ps.dualSabers != 0 {
        //can't cycle styles with dualSabers, so just toggle second saber on/off
        if WP_SaberCanTurnOffSomeBlades(&mut (*(*self_).client).ps.saber[1]) != 0 {
            //can turn second saber off
            if (*(*self_).client).ps.saber[1].ActiveManualOnly() != 0 {
                //turn it off
                let mut skipThisBlade: qboolean;
                for bladeNum in 0..(*(*self_).client).ps.saber[1].numBlades {
                    skipThisBlade = 0;
                    if WP_SaberBladeUseSecondBladeStyle(&mut (*(*self_).client).ps.saber[1], bladeNum as c_int) != 0 {
                        //check to see if we should check the secondary style's flags
                        if ((*(*self_).client).ps.saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE2)
                            != 0
                        {
                            skipThisBlade = 1;
                        }
                    } else {
                        //use the primary style's flags
                        if ((*(*self_).client).ps.saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE)
                            != 0
                        {
                            skipThisBlade = 1;
                        }
                    }
                    if skipThisBlade == 0 {
                        (*(*self_).client)
                            .ps
                            .saber[1]
                            .BladeActivate(bladeNum as c_int, 0);
                        G_SoundIndexOnEnt(
                            self_,
                            CHAN_WEAPON,
                            (*(*self_).client).ps.saber[1].soundOff,
                        );
                    }
                }
            } else if (*(*self_).client).ps.saber[0].ActiveManualOnly() == 0 {
                //first one is off, too, so just turn that one on
                if (*(*self_).client).ps.saberInFlight == 0 {
                    //but only if it's in your hand!
                    (*(*self_).client).ps.saber[0].Activate();
                }
            } else {
                //turn on the second one
                (*(*self_).client).ps.saber[1].Activate();
            }
            return;
        }
    } else if (*(*self_).client).ps.saber[0].numBlades > 1
        && WP_SaberCanTurnOffSomeBlades(&mut (*(*self_).client).ps.saber[0]) != 0
    {
        //can't cycle styles with saberstaff, so just toggles saber blades on/off
        if (*(*self_).client).ps.saberInFlight != 0 {
            //can't turn second blade back on if it's in the air, you naughty boy!
            return;
        }
        /*
        if ( self->client->ps.saber[0].singleBladeStyle == SS_NONE )
        {//can't use just one blade?
            return;
        }
        */
        let mut playedSound: qboolean = 0;
        if (*(*self_).client).ps.saber[0].blade[0].active == 0 {
            //first one is not even on
            //turn only it on
            (*(*self_).client)
                .ps
                .SaberBladeActivate(0, 0, 1);
            return;
        }

        let mut skipThisBlade: qboolean;
        for bladeNum in 1..(*(*self_).client).ps.saber[0].numBlades {
            if (*(*self_).client).ps.saber[0].blade[bladeNum].active == 0 {
                //extra is off, turn it on
                (*(*self_).client)
                    .ps
                    .saber[0]
                    .BladeActivate(bladeNum as c_int, 1);
            } else {
                //turn extra off
                skipThisBlade = 0;
                if WP_SaberBladeUseSecondBladeStyle(&mut (*(*self_).client).ps.saber[1], bladeNum as c_int) != 0 {
                    //check to see if we should check the secondary style's flags
                    if ((*(*self_).client).ps.saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE2)
                        != 0
                    {
                        skipThisBlade = 1;
                    }
                } else {
                    //use the primary style's flags
                    if ((*(*self_).client).ps.saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE)
                        != 0
                    {
                        skipThisBlade = 1;
                    }
                }
                if skipThisBlade == 0 {
                    (*(*self_).client)
                        .ps
                        .saber[0]
                        .BladeActivate(bladeNum as c_int, 0);
                    if playedSound == 0 {
                        G_SoundIndexOnEnt(
                            self_,
                            CHAN_WEAPON,
                            (*(*self_).client).ps.saber[0].soundOff,
                        );
                        playedSound = 1;
                    }
                }
            }
        }
        return;
    }

    let mut allowedStyles = (*(*self_).client).ps.saberStylesKnown;
    if (*(*self_).client).ps.dualSabers != 0
        && (*(*self_).client).ps.saber[0].Active() != 0
        && (*(*self_).client).ps.saber[1].Active() != 0
    {
        allowedStyles |= 1 << SS_DUAL;
        for styleNum in (SS_NONE + 1)..SS_NUM_SABER_STYLES {
            if styleNum == SS_TAVION
                && (((*(*self_).client).ps.saber[0].stylesLearned & (1 << SS_TAVION))
                    | ((*(*self_).client).ps.saber[1].stylesLearned & (1 << SS_TAVION)))
                    != 0
                && ((*(*self_).client).ps.saber[0].stylesForbidden & (1 << SS_TAVION)) == 0
                && ((*(*self_).client).ps.saber[1].stylesForbidden & (1 << SS_TAVION)) == 0
            {
                //if have both sabers on, allow tavion only if one of our sabers specifically wanted to use it... (unless specifically forbidden)
            } else if styleNum == SS_DUAL
                && ((*(*self_).client).ps.saber[0].stylesForbidden & (1 << SS_DUAL)) == 0
                && ((*(*self_).client).ps.saber[1].stylesForbidden & (1 << SS_DUAL)) == 0
            {
                //if have both sabers on, only dual style is allowed (unless specifically forbidden)
            } else {
                allowedStyles &= !(1 << styleNum);
            }
        }
    }

    if allowedStyles == 0 {
        return;
    }

    let mut saberAnimLevel: c_int;
    if (*self_).s.number == 0 {
        saberAnimLevel = cg.saberAnimLevelPending;
    } else {
        saberAnimLevel = (*(*self_).client).ps.saberAnimLevel;
    }
    saberAnimLevel += 1;
    let mut sanityCheck = 0;
    while (*(*self_).client).ps.saberAnimLevel != saberAnimLevel
        && (allowedStyles & (1 << saberAnimLevel)) == 0
        && sanityCheck < SS_NUM_SABER_STYLES + 1
    {
        saberAnimLevel += 1;
        if saberAnimLevel > SS_STAFF {
            saberAnimLevel = SS_FAST;
        }
        sanityCheck += 1;
    }

    if (allowedStyles & (1 << saberAnimLevel)) == 0 {
        return;
    }

    WP_UseFirstValidSaberStyle(self_, &mut saberAnimLevel);
    if (*self_).s.number == 0 {
        cg.saberAnimLevelPending = saberAnimLevel;
    } else {
        (*(*self_).client).ps.saberAnimLevel = saberAnimLevel;
    }

    #[cfg(not(feature = "FINAL_BUILD"))]
    {
        match saberAnimLevel {
            SS_FAST => {
                (*gi.Printf)("^4Lightsaber Combat Style: Fast\n" as *const c_char);
                //LIGHTSABERCOMBATSTYLE_FAST
            }
            SS_MEDIUM => {
                (*gi.Printf)("^3Lightsaber Combat Style: Medium\n" as *const c_char);
                //LIGHTSABERCOMBATSTYLE_MEDIUM
            }
            SS_STRONG => {
                (*gi.Printf)("^1Lightsaber Combat Style: Strong\n" as *const c_char);
                //LIGHTSABERCOMBATSTYLE_STRONG
            }
            SS_DESANN => {
                (*gi.Printf)("^5Lightsaber Combat Style: Desann\n" as *const c_char);
                //LIGHTSABERCOMBATSTYLE_DESANN
            }
            SS_TAVION => {
                (*gi.Printf)("^6Lightsaber Combat Style: Tavion\n" as *const c_char);
                //LIGHTSABERCOMBATSTYLE_TAVION
            }
            SS_DUAL => {
                (*gi.Printf)("^6Lightsaber Combat Style: Dual\n" as *const c_char);
                //LIGHTSABERCOMBATSTYLE_TAVION
            }
            SS_STAFF => {
                (*gi.Printf)("^6Lightsaber Combat Style: Staff\n" as *const c_char);
                //LIGHTSABERCOMBATSTYLE_TAVION
            }
            _ => {}
        }
        //gi.Printf("\n");
    }
}

#[allow(non_snake_case)]
unsafe fn G_ReleaseEntity(grabber: *mut gentity_t) -> qboolean {
    if !grabber.is_null()
        && !(*grabber).client.is_null()
        && (*(*grabber).client).ps.heldClient < ENTITYNUM_WORLD
    {
        let heldClient = &mut *core::ptr::addr_of_mut!(g_entities)
            .add((*(*grabber).client).ps.heldClient as usize);
        (*(*grabber).client).ps.heldClient = ENTITYNUM_NONE;
        if !heldClient.is_null() && !(*heldClient).client.is_null() {
            (*(*heldClient).client).ps.heldByClient = ENTITYNUM_NONE;

            (*heldClient).owner = core::ptr::null_mut();
        }
        return 1;
    }
    return 0;
}

#[allow(non_snake_case)]
unsafe fn G_GrabEntity(grabber: *mut gentity_t, target: *const c_char) {
    if grabber.is_null() || (*grabber).client.is_null() {
        return;
    }
    let mut heldClient = G_Find(core::ptr::null_mut(), FOFS, target);
    if !heldClient.is_null()
        && !(*heldClient).client.is_null()
        && heldClient != grabber
    {
        //don't grab yourself, it's not polite
        //found him
        (*(*grabber).client).ps.heldClient = (*heldClient).s.number;
        (*(*heldClient).client).ps.heldByClient = (*grabber).s.number;

        (*heldClient).owner = grabber;
    }
}

/*
=================
ConsoleCommand
// these are added in cg_main, CG_Init so they tab-complete
=================
*/
#[allow(non_snake_case)]
pub unsafe extern "C" fn ConsoleCommand() -> qboolean {
    let cmd = (*gi.argv)(0);

    if Q_stricmp(cmd, "entitylist" as *const c_char) == 0 {
        Svcmd_EntityList_f();
        return 1;
    }

    if Q_stricmp(cmd, "game_memory" as *const c_char) == 0 {
        Svcmd_GameMem_f();
        return 1;
    }

    //	if (Q_stricmp (cmd, "addbot") == 0) {
    //		Svcmd_AddBot_f();
    //		return qtrue;
    //	}

    if Q_stricmp(cmd, "nav" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        Svcmd_Nav_f();
        return 1;
    }

    if Q_stricmp(cmd, "npc" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        Svcmd_NPC_f();
        return 1;
    }

    if Q_stricmp(cmd, "use" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        Svcmd_Use_f();
        return 1;
    }

    if Q_stricmp(cmd, "ICARUS" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }

        (*Quake3Game()).Svcmd();

        return 1;
    }

    if Q_stricmp(cmd, "saberColor" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        Svcmd_SaberColor_f();
        return 1;
    }

    if Q_stricmp(cmd, "saber" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        Svcmd_Saber_f();
        return 1;
    }

    if Q_stricmp(cmd, "saberblade" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        Svcmd_SaberBlade_f();
        return 1;
    }

    if Q_stricmp(cmd, "setForceJump" as *const c_char) == 0 {
        Svcmd_ForceJump_f();
        return 1;
    }
    if Q_stricmp(cmd, "setSaberThrow" as *const c_char) == 0 {
        Svcmd_SaberThrow_f();
        return 1;
    }
    if Q_stricmp(cmd, "setForceHeal" as *const c_char) == 0 {
        Svcmd_ForceHeal_f();
        return 1;
    }
    if Q_stricmp(cmd, "setForcePush" as *const c_char) == 0 {
        Svcmd_ForcePush_f();
        return 1;
    }
    if Q_stricmp(cmd, "setForcePull" as *const c_char) == 0 {
        Svcmd_ForcePull_f();
        return 1;
    }
    if Q_stricmp(cmd, "setForceSpeed" as *const c_char) == 0 {
        Svcmd_ForceSpeed_f();
        return 1;
    }
    if Q_stricmp(cmd, "setForceGrip" as *const c_char) == 0 {
        Svcmd_ForceGrip_f();
        return 1;
    }
    if Q_stricmp(cmd, "setForceLightning" as *const c_char) == 0 {
        Svcmd_ForceLightning_f();
        return 1;
    }
    if Q_stricmp(cmd, "setMindTrick" as *const c_char) == 0 {
        Svcmd_MindTrick_f();
        return 1;
    }
    if Q_stricmp(cmd, "setSaberDefense" as *const c_char) == 0 {
        Svcmd_SaberDefense_f();
        return 1;
    }
    if Q_stricmp(cmd, "setSaberOffense" as *const c_char) == 0 {
        Svcmd_SaberOffense_f();
        return 1;
    }
    if Q_stricmp(cmd, "setForceRage" as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_RAGE);
        return 1;
    }
    if Q_stricmp(cmd, "setForceDrain" as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_DRAIN);
        return 1;
    }
    if Q_stricmp(cmd, "setForceProtect" as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_PROTECT);
        return 1;
    }
    if Q_stricmp(cmd, "setForceAbsorb" as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_ABSORB);
        return 1;
    }
    if Q_stricmp(cmd, "setForceSight" as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_SEE);
        return 1;
    }
    if Q_stricmp(cmd, "setForceAll" as *const c_char) == 0 {
        Svcmd_ForceJump_f();
        Svcmd_SaberThrow_f();
        Svcmd_ForceHeal_f();
        Svcmd_ForcePush_f();
        Svcmd_ForcePull_f();
        Svcmd_ForceSpeed_f();
        Svcmd_ForceGrip_f();
        Svcmd_ForceLightning_f();
        Svcmd_MindTrick_f();
        Svcmd_SaberDefense_f();
        Svcmd_SaberOffense_f();
        Svcmd_ForceSetLevel_f(FP_RAGE);
        Svcmd_ForceSetLevel_f(FP_DRAIN);
        Svcmd_ForceSetLevel_f(FP_PROTECT);
        Svcmd_ForceSetLevel_f(FP_ABSORB);
        Svcmd_ForceSetLevel_f(FP_SEE);
        for i in (SS_NONE + 1)..SS_NUM_SABER_STYLES {
            (*core::ptr::addr_of_mut!(g_entities).read().client).ps.saberStylesKnown |= 1 << i;
        }
        return 1;
    }
    if Q_stricmp(cmd, "saberAttackCycle" as *const c_char) == 0 {
        Svcmd_SaberAttackCycle_f();
        return 1;
    }
    if Q_stricmp(cmd, "runscript" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        let cmd2 = (*gi.argv)(1);

        if !cmd2.is_null() && *cmd2 != 0 {
            let cmd3 = (*gi.argv)(2);
            if !cmd3.is_null() && *cmd3 != 0 {
                let mut found = G_Find(
                    core::ptr::null_mut(),
                    FOFS,
                    cmd2,
                );
                if !found.is_null() {
                    (*Quake3Game()).RunScript(found, cmd3);
                } else {
                    //can't find cmd2
                    (*gi.Printf)(
                        "^1runscript: can't find targetname %s\n" as *const c_char,
                        cmd2,
                    );
                }
            } else {
                (*Quake3Game()).RunScript(core::ptr::addr_of_mut!(g_entities), cmd2);
            }
        } else {
            (*gi.Printf)("^1usage: runscript <ent targetname> scriptname\n" as *const c_char);
        }
        //FIXME: else warning
        return 1;
    }

    if Q_stricmp(cmd, "playerteam" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        let cmd2 = (*gi.argv)(1);
        let mut n: c_int;

        if *cmd2 == 0 || cmd2[0 as usize] as c_char == 0 {
            (*gi.Printf)("^1'playerteam' - change player team, requires a team name!\n" as *const c_char);
            (*gi.Printf)("^1Valid team names are:\n" as *const c_char);
            n = TEAM_FREE + 1;
            while n < TEAM_NUM_TEAMS {
                (*gi.Printf)(
                    "^1%s\n" as *const c_char,
                    GetStringForID(&TeamTable, n),
                );
                n += 1;
            }
        } else {
            let team: c_int;

            team = GetIDForString(&TeamTable, cmd2);
            if team == -1 {
                (*gi.Printf)(
                    "^1'playerteam' unrecognized team name %s!\n" as *const c_char,
                    cmd2,
                );
                (*gi.Printf)("^1Valid team names are:\n" as *const c_char);
                n = TEAM_FREE;
                while n < TEAM_NUM_TEAMS {
                    (*gi.Printf)(
                        "^1%s\n" as *const c_char,
                        GetStringForID(&TeamTable, n),
                    );
                    n += 1;
                }
            } else {
                (*core::ptr::addr_of_mut!(g_entities).read().client).playerTeam = team;
                //FIXME: convert Imperial, Malon, Hirogen and Klingon to Scavenger?
            }
        }
        return 1;
    }

    if Q_stricmp(cmd, "control" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        let cmd2 = (*gi.argv)(1);
        if *cmd2 == 0 || cmd2[0 as usize] as c_char == 0 {
            if G_ClearViewEntity(core::ptr::addr_of_mut!(g_entities)) == 0 {
                (*gi.Printf)("^1control <NPC_targetname>\n" as *const c_char, cmd2);
            }
        } else {
            Q3_SetViewEntity(0, cmd2);
        }
        return 1;
    }

    if Q_stricmp(cmd, "grab" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        let cmd2 = (*gi.argv)(1);
        if *cmd2 == 0 || cmd2[0 as usize] as c_char == 0 {
            if G_ReleaseEntity(core::ptr::addr_of_mut!(g_entities)) == 0 {
                (*gi.Printf)("^1grab <NPC_targetname>\n" as *const c_char, cmd2);
            }
        } else {
            G_GrabEntity(core::ptr::addr_of_mut!(g_entities), cmd2);
        }
        return 1;
    }

    if Q_stricmp(cmd, "knockdown" as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (*gi.SendServerCommand)(
                0,
                "print \"Cheats are not enabled on this server.\n\"" as *const c_char,
            );
            return 0;
        }
        let vec3_origin = [0.0f32; 3];
        G_Knockdown(
            core::ptr::addr_of_mut!(g_entities),
            core::ptr::addr_of_mut!(g_entities),
            &vec3_origin,
            300.0,
            1,
        );
        return 1;
    }

    if Q_stricmp(cmd, "playerModel" as *const c_char) == 0 {
        if (*gi.argc)() == 1 {
            (*gi.Printf)(
                "^1USAGE: playerModel <NPC Name>\n       playerModel <g2model> <skinhead> <skintorso> <skinlower>\n       playerModel player (builds player from customized menu settings)\n" as *const c_char,
            );
            (*gi.Printf)(
                "playerModel = %s " as *const c_char,
                va(
                    "%s %s %s %s\n" as *const c_char,
                    (*g_char_model).string,
                    (*g_char_skin_head).string,
                    (*g_char_skin_torso).string,
                    (*g_char_skin_legs).string,
                ),
            );
        } else if (*gi.argc)() == 2 {
            G_ChangePlayerModel(
                core::ptr::addr_of_mut!(g_entities),
                (*gi.argv)(1),
            );
        } else if (*gi.argc)() == 5 {
            //instead of setting it directly via a command, we now store it in cvars
            //G_ChangePlayerModel( &g_entities[0], va("%s|%s|%s|%s", gi.argv(1), gi.argv(2), gi.argv(3), gi.argv(4)) );
            (*gi.cvar_set)("g_char_model" as *const c_char, (*gi.argv)(1));
            (*gi.cvar_set)("g_char_skin_head" as *const c_char, (*gi.argv)(2));
            (*gi.cvar_set)("g_char_skin_torso" as *const c_char, (*gi.argv)(3));
            (*gi.cvar_set)("g_char_skin_legs" as *const c_char, (*gi.argv)(4));
            G_InitPlayerFromCvars(core::ptr::addr_of_mut!(g_entities));
        }
        return 1;
    }

    if Q_stricmp(cmd, "playerTint" as *const c_char) == 0 {
        if (*gi.argc)() == 4 {
            (*core::ptr::addr_of_mut!(g_entities).read().client).renderInfo.customRGBA[0] =
                atoi((*gi.argv)(1));
            (*core::ptr::addr_of_mut!(g_entities).read().client).renderInfo.customRGBA[1] =
                atoi((*gi.argv)(2));
            (*core::ptr::addr_of_mut!(g_entities).read().client).renderInfo.customRGBA[2] =
                atoi((*gi.argv)(3));
            (*gi.cvar_set)("g_char_color_red" as *const c_char, (*gi.argv)(1));
            (*gi.cvar_set)("g_char_color_green" as *const c_char, (*gi.argv)(2));
            (*gi.cvar_set)("g_char_color_blue" as *const c_char, (*gi.argv)(3));
        } else {
            (*gi.Printf)(
                "^1USAGE: playerTint <red 0 - 255> <green 0 - 255> <blue 0 - 255>\n" as *const c_char,
            );
            (*gi.Printf)(
                "playerTint = %s\n" as *const c_char,
                va(
                    "%d %d %d" as *const c_char,
                    (*g_char_color_red).integer,
                    (*g_char_color_green).integer,
                    (*g_char_color_blue).integer,
                ),
            );
        }
        return 1;
    }
    if Q_stricmp(cmd, "nexttestaxes" as *const c_char) == 0 {
        G_NextTestAxes();
    }

    if Q_stricmp(cmd, "exitview" as *const c_char) == 0 {
        Svcmd_ExitView_f();
    }

    if Q_stricmp(cmd, "iknowkungfu" as *const c_char) == 0 {
        (*gi.cvar_set)("g_debugMelee" as *const c_char, "1" as *const c_char);
        G_SetWeapon(core::ptr::addr_of_mut!(g_entities), WP_MELEE);
        for i in FP_FIRST..NUM_FORCE_POWERS {
            (*core::ptr::addr_of_mut!(g_entities).read().client).ps.forcePowersKnown |= 1 << i;
            if i == FP_TELEPATHY {
                (*core::ptr::addr_of_mut!(g_entities).read().client).ps.forcePowerLevel[i as usize] =
                    FORCE_LEVEL_4;
            } else {
                (*core::ptr::addr_of_mut!(g_entities).read().client).ps.forcePowerLevel[i as usize] =
                    FORCE_LEVEL_3;
            }
        }
    }

    return 0;
}
