// leave this line at the top for all g_xxxx.rs files...
use crate::code::game::g_headers_h::*;
use crate::code::game::Q3_Interface_h::*;
use crate::code::game::g_local_h::*;
use crate::code::game::wp_saber_h::*;

// cg_t is defined in cgame/cg_local.h. g_svcmds.cpp accesses `cg` via the SP cross-module
// linkage (game and cgame in the same process). The Rust port requires an explicit type import.
use crate::code::cgame::cg_local_h::cg_t;

use core::ffi::{c_int, c_char, c_float};

extern "C" {
    fn G_NextTestAxes();
    fn G_ChangePlayerModel(ent: *mut gentity_t, newModel: *const c_char);
    fn G_InitPlayerFromCvars(ent: *mut gentity_t);
    fn Q3_SetViewEntity(entID: c_int, name: *const c_char);
    fn G_ClearViewEntity(ent: *mut gentity_t) -> qboolean;
    fn G_Knockdown(
        self_: *mut gentity_t,
        attacker: *mut gentity_t,
        pushDir: *const vec3_t,
        strength: c_float,
        breakSaberLock: qboolean,
    );

    fn WP_SetSaber(ent: *mut gentity_t, saberNum: c_int, saberName: *mut c_char);
    fn WP_RemoveSaber(ent: *mut gentity_t, saberNum: c_int);
    fn TranslateSaberColor(name: *const c_char) -> saber_colors_t;
    fn WP_SaberBladeUseSecondBladeStyle(saber: *mut saberInfo_t, bladeNum: c_int) -> qboolean;
    fn WP_UseFirstValidSaberStyle(ent: *mut gentity_t, saberAnimLevel: *mut c_int) -> qboolean;

    fn G_SetWeapon(self_: *mut gentity_t, wp: c_int);
    static WPTable: stringID_table_t;

    static mut g_char_model: *mut cvar_t;
    static mut g_char_skin_head: *mut cvar_t;
    static mut g_char_skin_torso: *mut cvar_t;
    static mut g_char_skin_legs: *mut cvar_t;
    static mut g_char_color_red: *mut cvar_t;
    static mut g_char_color_green: *mut cvar_t;
    static mut g_char_color_blue: *mut cvar_t;
    static mut g_saber: *mut cvar_t;
    static mut g_saber2: *mut cvar_t;
    static mut g_saber_color: *mut cvar_t;
    static mut g_saber2_color: *mut cvar_t;
}

//---------------------------
// The following three externs appear just before Svcmd_ExitView_f in the C source.
// g_skippingcin is declared inside the function body in C; Rust requires module scope.
extern "C" {
    fn G_StopCinematicSkip();
    fn G_StartCinematicSkip();
    fn ExitEmplacedWeapon(ent: *mut gentity_t);
    static mut g_skippingcin: *mut cvar_t;
}

// in_camera is defined in cgame/cg_camera.cpp and declared in cgame/cg_camera.h,
// which g_headers.h includes only on Xbox. g_svcmds.cpp uses it without a local extern,
// relying on SP cross-module linkage. Declared explicitly here for Rust.
extern "C" {
    static in_camera: bool;
}

// cg is defined in cgame/cg_main.cpp, declared in cgame/cg_local.h. Same cross-module note
// as in_camera above.
extern "C" {
    static mut cg: cg_t;
}

extern "C" {
    fn PM_SaberInStart(mv: c_int) -> qboolean;
    fn PM_SaberInTransition(mv: c_int) -> qboolean;
    fn PM_SaberInAttack(mv: c_int) -> qboolean;
    fn WP_SaberCanTurnOffSomeBlades(saber: *mut saberInfo_t) -> qboolean;
}

/*
===================
Svcmd_EntityList_f
===================
*/
#[allow(non_snake_case)]
pub unsafe fn Svcmd_EntityList_f() {
    let mut e: c_int;
    let mut check: *mut gentity_t;

    check = core::ptr::addr_of_mut!(g_entities[0]).add(1);
    e = 1;
    while e < (*core::ptr::addr_of!(globals)).num_entities {
        if (*check).inuse == 0 {
            e += 1;
            check = check.add(1);
            continue;
        }
        (gi.Printf)(b"%3i:\0".as_ptr() as *const c_char, e);
        match (*check).s.eType {
            ET_GENERAL => {
                (gi.Printf)(b"ET_GENERAL \0".as_ptr() as *const c_char);
            }
            ET_PLAYER => {
                (gi.Printf)(b"ET_PLAYER  \0".as_ptr() as *const c_char);
            }
            ET_ITEM => {
                (gi.Printf)(b"ET_ITEM    \0".as_ptr() as *const c_char);
            }
            ET_MISSILE => {
                (gi.Printf)(b"ET_MISSILE \0".as_ptr() as *const c_char);
            }
            ET_MOVER => {
                (gi.Printf)(b"ET_MOVER   \0".as_ptr() as *const c_char);
            }
            ET_BEAM => {
                (gi.Printf)(b"ET_BEAM    \0".as_ptr() as *const c_char);
            }
            _ => {
                (gi.Printf)(b"#%i\0".as_ptr() as *const c_char, (*check).s.eType);
            }
        }

        if !(*check).classname.is_null() {
            (gi.Printf)(b"%s\0".as_ptr() as *const c_char, (*check).classname);
        }
        (gi.Printf)(b"\n\0".as_ptr() as *const c_char);
        e += 1;
        check = check.add(1);
    }
}

#[allow(non_snake_case)]
pub unsafe fn ClientForString(s: *const c_char) -> *mut gclient_t {
    let mut cl: *mut gclient_t;
    let mut i: c_int;
    let idnum: c_int;

    // numeric values are just slot numbers
    if *s as u8 >= b'0' && *s as u8 <= b'9' {
        idnum = atoi(s);
        if idnum < 0 || idnum >= (*core::ptr::addr_of!(level)).maxclients {
            Com_Printf(b"Bad client slot: %i\n\0".as_ptr() as *const c_char, idnum);
            return core::ptr::null_mut();
        }

        cl = &mut (*core::ptr::addr_of_mut!(level)).clients[idnum as usize];
        if (*cl).pers.connected == CON_DISCONNECTED {
            (gi.Printf)(b"Client %i is not connected\n\0".as_ptr() as *const c_char, idnum);
            return core::ptr::null_mut();
        }
        return cl;
    }

    // check for a name match
    i = 0;
    while i < (*core::ptr::addr_of!(level)).maxclients {
        cl = &mut (*core::ptr::addr_of_mut!(level)).clients[i as usize];
        if (*cl).pers.connected == CON_DISCONNECTED {
            i += 1;
            continue;
        }
        if Q_stricmp((*cl).pers.netname.as_ptr(), s) == 0 {
            return cl;
        }
        i += 1;
    }

    (gi.Printf)(b"User %s is not on the server\n\0".as_ptr() as *const c_char, s);

    core::ptr::null_mut()
}

//---------------------------
#[allow(non_snake_case)]
unsafe fn Svcmd_ExitView_f() {
    // extern cvar_t *g_skippingcin; -- declared at module scope in Rust (extern items must be at module level)
    static mut exitViewDebounce: c_int = 0;
    if exitViewDebounce > (*core::ptr::addr_of!(level)).time {
        return;
    }
    exitViewDebounce = (*core::ptr::addr_of!(level)).time + 500;
    if in_camera {
        //see if we need to exit an in-game cinematic
        if (*g_skippingcin).integer != 0 {
            // already doing cinematic skip?
            // yes...   so stop skipping...
            G_StopCinematicSkip();
        } else {
            // no... so start skipping...
            G_StartCinematicSkip();
        }
    } else if G_ClearViewEntity(player) == qfalse {
        //didn't exit control of a droid or turret
        //okay, now try exiting emplaced guns or AT-ST's
        if (*player).s.eFlags & EF_LOCKED_TO_WEAPON != 0 {
            //get out of emplaced gun
            ExitEmplacedWeapon(player);
        } else if !(*player).client.is_null()
            && (*(*player).client).NPC_class == CLASS_ATST
        {
            //a player trying to get out of his ATST
            GEntity_UseFunc((*player).activator, player, player);
        }
    }
}

#[allow(non_snake_case)]
pub unsafe fn G_GetSelfForPlayerCmd() -> *mut gentity_t {
    let ent0 = core::ptr::addr_of_mut!(g_entities[0]);
    if (*(*ent0).client).ps.viewEntity > 0
        && (*(*ent0).client).ps.viewEntity < ENTITYNUM_WORLD
        && !(*core::ptr::addr_of_mut!(
            g_entities[(*(*ent0).client).ps.viewEntity as usize]
        ))
        .client
        .is_null()
        && (*core::ptr::addr_of_mut!(
            g_entities[(*(*ent0).client).ps.viewEntity as usize]
        ))
        .s
        .weapon
            == WP_SABER
    {
        //you're controlling another NPC
        let ve = (*(*ent0).client).ps.viewEntity as usize;
        return core::ptr::addr_of_mut!(g_entities[ve]);
    } else {
        return core::ptr::addr_of_mut!(g_entities[0]);
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_Saber_f() {
    let saber = (gi.argv)(1);
    let saber2 = (gi.argv)(2);

    if (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
        || saber.is_null()
        || *saber == 0
    {
        return;
    }

    (gi.cvar_set)(b"g_saber\0".as_ptr() as *const c_char, saber);
    WP_SetSaber(core::ptr::addr_of_mut!(g_entities[0]), 0, saber as *mut c_char);
    if !saber2.is_null()
        && *saber2 != 0
        && ((*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.saber[0].saberFlags
            & SFL_TWO_HANDED)
            == 0
    {
        //want to use a second saber and first one is not twoHanded
        (gi.cvar_set)(b"g_saber2\0".as_ptr() as *const c_char, saber2);
        WP_SetSaber(core::ptr::addr_of_mut!(g_entities[0]), 1, saber2 as *mut c_char);
    } else {
        (gi.cvar_set)(
            b"g_saber2\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
        );
        WP_RemoveSaber(core::ptr::addr_of_mut!(g_entities[0]), 1);
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberBlade_f() {
    if (gi.argc)() < 2 {
        (gi.Printf)(
            b"USAGE: saberblade <sabernum> <bladenum> [0 = off, 1 = on, no arg = toggle]\n\0"
                .as_ptr() as *const c_char,
        );
        return;
    }
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    let sabernum = atoi((gi.argv)(1)) - 1;
    if sabernum < 0 || sabernum > 1 {
        return;
    }
    if sabernum > 0
        && (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.dualSabers == qfalse
    {
        return;
    }
    //FIXME: what if don't even have a single saber at all?
    let bladenum = atoi((gi.argv)(2)) - 1;
    if bladenum < 0
        || bladenum
            >= (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.saber[sabernum as usize]
                .numBlades
    {
        return;
    }
    let turnOn: qboolean;
    if (gi.argc)() > 2 {
        //explicit
        turnOn = if atoi((gi.argv)(3)) != 0 { qtrue } else { qfalse };
    } else {
        //toggle
        turnOn = if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.saber
            [sabernum as usize]
            .blade[bladenum as usize]
            .active
            == qfalse
        {
            qtrue
        } else {
            qfalse
        };
    }

    (*(*core::ptr::addr_of_mut!(g_entities[0])).client)
        .ps
        .SaberBladeActivate(sabernum, bladenum, turnOn);
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberColor_f() {
    //FIXME: just list the colors, each additional listing sets that blade
    let saberNum = atoi((gi.argv)(1));
    let mut color: [*const c_char; MAX_BLADES] = [core::ptr::null(); MAX_BLADES];
    let mut bladeNum: c_int;

    bladeNum = 0;
    while bladeNum < MAX_BLADES as c_int {
        color[bladeNum as usize] = (gi.argv)(2 + bladeNum);
        bladeNum += 1;
    }

    // VALIDSTRING( color ) expands to ((color) && (color)[0]).
    // color is a stack array so the pointer is always non-null; (color)[0] checks color[0] != NULL.
    if color[0].is_null() || saberNum < 1 || saberNum > 2 {
        (gi.Printf)(
            b"Usage:  saberColor <saberNum> <blade1 color> <blade2 color> ... <blade8 color> \n\0"
                .as_ptr() as *const c_char,
        );
        (gi.Printf)(b"valid saberNums:  1 or 2\n\0".as_ptr() as *const c_char);
        (gi.Printf)(
            b"valid colors:  red, orange, yellow, green, blue, and purple\n\0".as_ptr()
                as *const c_char,
        );

        return;
    }
    let saberNum = saberNum - 1;

    let self_ = G_GetSelfForPlayerCmd();

    bladeNum = 0;
    while bladeNum < MAX_BLADES as c_int {
        if color[bladeNum as usize].is_null() || *color[bladeNum as usize] == 0 {
            break;
        } else {
            (*(*self_).client).ps.saber[saberNum as usize].blade[bladeNum as usize].color =
                TranslateSaberColor(color[bladeNum as usize]);
        }
        bladeNum += 1;
    }

    if saberNum == 0 {
        (gi.cvar_set)(b"g_saber_color\0".as_ptr() as *const c_char, color[0]);
    } else if saberNum == 1 {
        (gi.cvar_set)(b"g_saber2_color\0".as_ptr() as *const c_char, color[0]);
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceJump_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    // VALIDSTRING(newVal): ptr non-null && first char non-zero
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current forceJump level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_LEVITATION as usize],
        );
        (gi.Printf)(b"Usage:  setForceJump <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << FP_LEVITATION;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_LEVITATION);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_LEVITATION as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_LEVITATION as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_LEVITATION as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_LEVITATION as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_LEVITATION as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberThrow_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current saberThrow level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_SABERTHROW as usize],
        );
        (gi.Printf)(b"Usage:  setSaberThrow <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << FP_SABERTHROW;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_SABERTHROW);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABERTHROW as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABERTHROW as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SABERTHROW as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABERTHROW as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SABERTHROW as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceHeal_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current forceHeal level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_HEAL as usize],
        );
        (gi.Printf)(b"Usage:  forceHeal <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |= 1 << FP_HEAL;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_HEAL);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_HEAL as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_HEAL as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_HEAL as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_HEAL as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_HEAL as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForcePush_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current forcePush level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_PUSH as usize],
        );
        (gi.Printf)(b"Usage:  forcePush <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |= 1 << FP_PUSH;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_PUSH);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_PUSH as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_PUSH as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_PUSH as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_PUSH as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_PUSH as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForcePull_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current forcePull level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_PULL as usize],
        );
        (gi.Printf)(b"Usage:  forcePull <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |= 1 << FP_PULL;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_PULL);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_PULL as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_PULL as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_PULL as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_PULL as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_PULL as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceSpeed_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current forceSpeed level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_SPEED as usize],
        );
        (gi.Printf)(b"Usage:  forceSpeed <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << FP_SPEED;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_SPEED);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_SPEED as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_SPEED as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SPEED as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SPEED as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SPEED as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceGrip_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current forceGrip level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_GRIP as usize],
        );
        (gi.Printf)(b"Usage:  forceGrip <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |= 1 << FP_GRIP;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_GRIP);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_GRIP as usize] =
        val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel[FP_GRIP as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_GRIP as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_GRIP as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_GRIP as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceLightning_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current forceLightning level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_LIGHTNING as usize],
        );
        (gi.Printf)(b"Usage:  forceLightning <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << FP_LIGHTNING;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_LIGHTNING);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_LIGHTNING as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_LIGHTNING as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_LIGHTNING as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_LIGHTNING as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_LIGHTNING as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_MindTrick_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current mindTrick level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_TELEPATHY as usize],
        );
        (gi.Printf)(b"Usage:  mindTrick <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << FP_TELEPATHY;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_TELEPATHY);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_TELEPATHY as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_TELEPATHY as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_TELEPATHY as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_TELEPATHY as usize]
        > FORCE_LEVEL_4
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_TELEPATHY as usize] = FORCE_LEVEL_4;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberDefense_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current saberDefense level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_SABER_DEFENSE as usize],
        );
        (gi.Printf)(b"Usage:  saberDefense <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << FP_SABER_DEFENSE;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_SABER_DEFENSE);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABER_DEFENSE as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABER_DEFENSE as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SABER_DEFENSE as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABER_DEFENSE as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SABER_DEFENSE as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_SaberOffense_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current saberOffense level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [FP_SABER_OFFENSE as usize],
        );
        (gi.Printf)(b"Usage:  saberOffense <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << FP_SABER_OFFENSE;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << FP_SABER_OFFENSE);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABER_OFFENSE as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABER_OFFENSE as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SABER_OFFENSE as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [FP_SABER_OFFENSE as usize]
        >= SS_NUM_SABER_STYLES
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [FP_SABER_OFFENSE as usize] = SS_NUM_SABER_STYLES - 1;
    }
}

#[allow(non_snake_case)]
unsafe fn Svcmd_ForceSetLevel_f(forcePower: c_int) {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }
    if (*g_cheats).integer == 0 {
        (gi.SendServerCommand)(
            0,
            b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
        );
        return;
    }
    let newVal = (gi.argv)(1);
    if newVal.is_null() || *newVal == 0 {
        (gi.Printf)(
            b"Current force level is %d\n\0".as_ptr() as *const c_char,
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                [forcePower as usize],
        );
        (gi.Printf)(b"Usage:  force <level> (1 - 3)\n\0".as_ptr() as *const c_char);
        return;
    }
    let val = atoi(newVal);
    if val > FORCE_LEVEL_0 {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |=
            1 << forcePower;
    } else {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown &=
            !(1 << forcePower);
    }
    (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [forcePower as usize] = val;
    if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [forcePower as usize]
        < FORCE_LEVEL_0
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [forcePower as usize] = FORCE_LEVEL_0;
    } else if (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
        [forcePower as usize]
        > FORCE_LEVEL_3
    {
        (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
            [forcePower as usize] = FORCE_LEVEL_3;
    }
}

#[allow(non_snake_case)]
pub unsafe fn Svcmd_SaberAttackCycle_f() {
    if core::ptr::addr_of_mut!(g_entities[0]).is_null()
        || (*core::ptr::addr_of_mut!(g_entities[0])).client.is_null()
    {
        return;
    }

    let self_ = G_GetSelfForPlayerCmd();
    if (*self_).s.weapon != WP_SABER {
        // saberAttackCycle button also switches to saber
        (gi.SendConsoleCommand)(b"weapon 1\0".as_ptr() as *const c_char);
        return;
    }

    if (*(*self_).client).ps.dualSabers != qfalse {
        //can't cycle styles with dualSabers, so just toggle second saber on/off
        if WP_SaberCanTurnOffSomeBlades(&mut (*(*self_).client).ps.saber[1]) != qfalse {
            //can turn second saber off
            if (*(*self_).client).ps.saber[1].ActiveManualOnly() != qfalse {
                //turn it off
                let mut skipThisBlade: qboolean;
                let mut bladeNum: c_int = 0;
                while bladeNum < (*(*self_).client).ps.saber[1].numBlades {
                    skipThisBlade = qfalse;
                    if WP_SaberBladeUseSecondBladeStyle(
                        &mut (*(*self_).client).ps.saber[1],
                        bladeNum,
                    ) != qfalse
                    {
                        //check to see if we should check the secondary style's flags
                        if ((*(*self_).client).ps.saber[1].saberFlags2
                            & SFL2_NO_MANUAL_DEACTIVATE2)
                            != 0
                        {
                            skipThisBlade = qtrue;
                        }
                    } else {
                        //use the primary style's flags
                        if ((*(*self_).client).ps.saber[1].saberFlags2
                            & SFL2_NO_MANUAL_DEACTIVATE)
                            != 0
                        {
                            skipThisBlade = qtrue;
                        }
                    }
                    if skipThisBlade == qfalse {
                        (*(*self_).client).ps.saber[1].BladeActivate(bladeNum, qfalse);
                        G_SoundIndexOnEnt(
                            self_,
                            CHAN_WEAPON,
                            (*(*self_).client).ps.saber[1].soundOff,
                        );
                    }
                    bladeNum += 1;
                }
            } else if (*(*self_).client).ps.saber[0].ActiveManualOnly() == qfalse {
                //first one is off, too, so just turn that one on
                if (*(*self_).client).ps.saberInFlight == qfalse {
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
        && WP_SaberCanTurnOffSomeBlades(&mut (*(*self_).client).ps.saber[0]) != qfalse
    //self->client->ps.saber[0].type == SABER_STAFF
    {
        //can't cycle styles with saberstaff, so just toggles saber blades on/off
        if (*(*self_).client).ps.saberInFlight != qfalse {
            //can't turn second blade back on if it's in the air, you naughty boy!
            return;
        }
        /*
        if ( self->client->ps.saber[0].singleBladeStyle == SS_NONE )
        {//can't use just one blade?
            return;
        }
        */
        let mut playedSound: qboolean = qfalse;
        if (*(*self_).client).ps.saber[0].blade[0].active == qfalse {
            //first one is not even on
            //turn only it on
            (*(*self_).client).ps.SaberBladeActivate(0, 0, qtrue);
            return;
        }

        let mut skipThisBlade: qboolean;
        let mut bladeNum: c_int = 1;
        while bladeNum < (*(*self_).client).ps.saber[0].numBlades {
            if (*(*self_).client).ps.saber[0].blade[bladeNum as usize].active == qfalse {
                //extra is off, turn it on
                (*(*self_).client).ps.saber[0].BladeActivate(bladeNum, qtrue);
            } else {
                //turn extra off
                skipThisBlade = qfalse;
                if WP_SaberBladeUseSecondBladeStyle(
                    &mut (*(*self_).client).ps.saber[1],
                    bladeNum,
                ) != qfalse
                {
                    //check to see if we should check the secondary style's flags
                    if ((*(*self_).client).ps.saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE2)
                        != 0
                    {
                        skipThisBlade = qtrue;
                    }
                } else {
                    //use the primary style's flags
                    if ((*(*self_).client).ps.saber[1].saberFlags2 & SFL2_NO_MANUAL_DEACTIVATE)
                        != 0
                    {
                        skipThisBlade = qtrue;
                    }
                }
                if skipThisBlade == qfalse {
                    (*(*self_).client).ps.saber[0].BladeActivate(bladeNum, qfalse);
                    if playedSound == qfalse {
                        G_SoundIndexOnEnt(
                            self_,
                            CHAN_WEAPON,
                            (*(*self_).client).ps.saber[0].soundOff,
                        );
                        playedSound = qtrue;
                    }
                }
            }
            bladeNum += 1;
        }
        return;
    }

    let mut allowedStyles = (*(*self_).client).ps.saberStylesKnown;
    if (*(*self_).client).ps.dualSabers != qfalse
        && (*(*self_).client).ps.saber[0].Active() != qfalse
        && (*(*self_).client).ps.saber[1].Active() != qfalse
    {
        allowedStyles |= 1 << SS_DUAL;
        let mut styleNum: c_int = SS_NONE + 1;
        while styleNum < SS_NUM_SABER_STYLES {
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
            styleNum += 1;
        }
    }

    if allowedStyles == 0 {
        return;
    }

    let mut saberAnimLevel: c_int;
    if (*self_).s.number == 0 {
        saberAnimLevel = (*core::ptr::addr_of_mut!(cg)).saberAnimLevelPending;
    } else {
        saberAnimLevel = (*(*self_).client).ps.saberAnimLevel;
    }
    saberAnimLevel += 1;
    let mut sanityCheck: c_int = 0;
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
        (*core::ptr::addr_of_mut!(cg)).saberAnimLevelPending = saberAnimLevel;
    } else {
        (*(*self_).client).ps.saberAnimLevel = saberAnimLevel;
    }

    #[cfg(not(feature = "final_build"))]
    {
        match saberAnimLevel {
            x if x == SS_FAST => {
                (gi.Printf)(
                    b"%sLightsaber Combat Style: Fast\n\0".as_ptr() as *const c_char,
                    S_COLOR_BLUE,
                );
                //LIGHTSABERCOMBATSTYLE_FAST
            }
            x if x == SS_MEDIUM => {
                (gi.Printf)(
                    b"%sLightsaber Combat Style: Medium\n\0".as_ptr() as *const c_char,
                    S_COLOR_YELLOW,
                );
                //LIGHTSABERCOMBATSTYLE_MEDIUM
            }
            x if x == SS_STRONG => {
                (gi.Printf)(
                    b"%sLightsaber Combat Style: Strong\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                );
                //LIGHTSABERCOMBATSTYLE_STRONG
            }
            x if x == SS_DESANN => {
                (gi.Printf)(
                    b"%sLightsaber Combat Style: Desann\n\0".as_ptr() as *const c_char,
                    S_COLOR_CYAN,
                );
                //LIGHTSABERCOMBATSTYLE_DESANN
            }
            x if x == SS_TAVION => {
                (gi.Printf)(
                    b"%sLightsaber Combat Style: Tavion\n\0".as_ptr() as *const c_char,
                    S_COLOR_MAGENTA,
                );
                //LIGHTSABERCOMBATSTYLE_TAVION
            }
            x if x == SS_DUAL => {
                (gi.Printf)(
                    b"%sLightsaber Combat Style: Dual\n\0".as_ptr() as *const c_char,
                    S_COLOR_MAGENTA,
                );
                //LIGHTSABERCOMBATSTYLE_TAVION
            }
            x if x == SS_STAFF => {
                (gi.Printf)(
                    b"%sLightsaber Combat Style: Staff\n\0".as_ptr() as *const c_char,
                    S_COLOR_MAGENTA,
                );
                //LIGHTSABERCOMBATSTYLE_TAVION
            }
            _ => {}
        }
        //gi.Printf("\n");
    }
}

#[allow(non_snake_case)]
pub unsafe fn G_ReleaseEntity(grabber: *mut gentity_t) -> qboolean {
    if !grabber.is_null()
        && !(*grabber).client.is_null()
        && (*(*grabber).client).ps.heldClient < ENTITYNUM_WORLD
    {
        let heldClient =
            core::ptr::addr_of_mut!(g_entities[(*(*grabber).client).ps.heldClient as usize]);
        (*(*grabber).client).ps.heldClient = ENTITYNUM_NONE;
        if !heldClient.is_null() && !(*heldClient).client.is_null() {
            (*(*heldClient).client).ps.heldByClient = ENTITYNUM_NONE;

            (*heldClient).owner = core::ptr::null_mut();
        }
        return qtrue;
    }
    qfalse
}

#[allow(non_snake_case)]
pub unsafe fn G_GrabEntity(grabber: *mut gentity_t, target: *mut c_char) {
    if grabber.is_null() || (*grabber).client.is_null() {
        return;
    }
    let heldClient = G_Find(
        core::ptr::null_mut(),
        core::mem::offset_of!(gentity_t, targetname) as c_int, // C: FOFS(targetname)
        target as *const c_char,
    );
    if !heldClient.is_null()
        && !(*heldClient).client.is_null()
        && heldClient != grabber
    //don't grab yourself, it's not polite
    {
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
pub unsafe fn ConsoleCommand() -> qboolean {
    let cmd: *const c_char;

    cmd = (gi.argv)(0);

    if Q_stricmp(cmd, b"entitylist\0".as_ptr() as *const c_char) == 0 {
        Svcmd_EntityList_f();
        return qtrue;
    }

    if Q_stricmp(cmd, b"game_memory\0".as_ptr() as *const c_char) == 0 {
        Svcmd_GameMem_f();
        return qtrue;
    }

//	if (Q_stricmp (cmd, "addbot") == 0) {
//		Svcmd_AddBot_f();
//		return qtrue;
//	}

    if Q_stricmp(cmd, b"nav\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        Svcmd_Nav_f();
        return qtrue;
    }

    if Q_stricmp(cmd, b"npc\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        Svcmd_NPC_f();
        return qtrue;
    }

    if Q_stricmp(cmd, b"use\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        Svcmd_Use_f();
        return qtrue;
    }

    if Q_stricmp(cmd, b"ICARUS\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }

        (*Quake3Game()).Svcmd();

        return qtrue;
    }

    if Q_stricmp(cmd, b"saberColor\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        Svcmd_SaberColor_f();
        return qtrue;
    }

    if Q_stricmp(cmd, b"saber\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        Svcmd_Saber_f();
        return qtrue;
    }

    if Q_stricmp(cmd, b"saberblade\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        Svcmd_SaberBlade_f();
        return qtrue;
    }


    if Q_stricmp(cmd, b"setForceJump\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceJump_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setSaberThrow\0".as_ptr() as *const c_char) == 0 {
        Svcmd_SaberThrow_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceHeal\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceHeal_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForcePush\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForcePush_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForcePull\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForcePull_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceSpeed\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceSpeed_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceGrip\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceGrip_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceLightning\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceLightning_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setMindTrick\0".as_ptr() as *const c_char) == 0 {
        Svcmd_MindTrick_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setSaberDefense\0".as_ptr() as *const c_char) == 0 {
        Svcmd_SaberDefense_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setSaberOffense\0".as_ptr() as *const c_char) == 0 {
        Svcmd_SaberOffense_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceRage\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_RAGE);
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceDrain\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_DRAIN);
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceProtect\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_PROTECT);
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceAbsorb\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_ABSORB);
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceSight\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ForceSetLevel_f(FP_SEE);
        return qtrue;
    }
    if Q_stricmp(cmd, b"setForceAll\0".as_ptr() as *const c_char) == 0 {
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
        let mut i: c_int = SS_NONE + 1;
        while i < SS_NUM_SABER_STYLES {
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.saberStylesKnown |= 1 << i;
            i += 1;
        }
        return qtrue;
    }
    if Q_stricmp(cmd, b"saberAttackCycle\0".as_ptr() as *const c_char) == 0 {
        Svcmd_SaberAttackCycle_f();
        return qtrue;
    }
    if Q_stricmp(cmd, b"runscript\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        let cmd2 = (gi.argv)(1);

        if !cmd2.is_null() && *cmd2 != 0 {
            let cmd3 = (gi.argv)(2);
            if !cmd3.is_null() && *cmd3 != 0 {
                let mut found: *mut gentity_t = core::ptr::null_mut();
                found = G_Find(
                    core::ptr::null_mut(),
                    core::mem::offset_of!(gentity_t, targetname) as c_int, // C: FOFS(targetname)
                    cmd2,
                );
                if !found.is_null() {
                    (*Quake3Game()).RunScript(found, cmd3);
                } else {
                    //can't find cmd2
                    (gi.Printf)(
                        b"%srunscript: can't find targetname %s\n\0".as_ptr() as *const c_char,
                        S_COLOR_RED,
                        cmd2,
                    );
                }
            } else {
                (*Quake3Game()).RunScript(core::ptr::addr_of_mut!(g_entities[0]), cmd2);
            }
        } else {
            (gi.Printf)(
                b"%susage: runscript <ent targetname> scriptname\n\0".as_ptr() as *const c_char,
                S_COLOR_RED,
            );
        }
        //FIXME: else warning
        return qtrue;
    }

    if Q_stricmp(cmd, b"playerteam\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        let cmd2 = (gi.argv)(1);
        let mut n: c_int;

        if cmd2.is_null() || *cmd2 == 0 {
            (gi.Printf)(
                b"%s'playerteam' - change player team, requires a team name!\n\0".as_ptr()
                    as *const c_char,
                S_COLOR_RED,
            );
            (gi.Printf)(
                b"%sValid team names are:\n\0".as_ptr() as *const c_char,
                S_COLOR_RED,
            );
            n = TEAM_FREE + 1;
            while n < TEAM_NUM_TEAMS {
                (gi.Printf)(
                    b"%s%s\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                    GetStringForID(core::ptr::addr_of!(TeamTable), n),
                );
                n += 1;
            }
        } else {
            let team: team_t;

            team = GetIDForString(core::ptr::addr_of!(TeamTable), cmd2) as team_t;
            if team == -1 as team_t {
                (gi.Printf)(
                    b"%s'playerteam' unrecognized team name %s!\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                    cmd2,
                );
                (gi.Printf)(
                    b"%sValid team names are:\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                );
                n = TEAM_FREE;
                while n < TEAM_NUM_TEAMS {
                    (gi.Printf)(
                        b"%s%s\n\0".as_ptr() as *const c_char,
                        S_COLOR_RED,
                        GetStringForID(core::ptr::addr_of!(TeamTable), n),
                    );
                    n += 1;
                }
            } else {
                (*(*core::ptr::addr_of_mut!(g_entities[0])).client).playerTeam = team;
                //FIXME: convert Imperial, Malon, Hirogen and Klingon to Scavenger?
            }
        }
        return qtrue;
    }

    if Q_stricmp(cmd, b"control\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        let cmd2 = (gi.argv)(1);
        if cmd2.is_null() || *cmd2 == 0 {
            if G_ClearViewEntity(core::ptr::addr_of_mut!(g_entities[0])) == qfalse {
                (gi.Printf)(
                    b"%scontrol <NPC_targetname>\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                    cmd2,
                );
            }
        } else {
            Q3_SetViewEntity(0, cmd2);
        }
        return qtrue;
    }

    if Q_stricmp(cmd, b"grab\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        let cmd2 = (gi.argv)(1);
        if cmd2.is_null() || *cmd2 == 0 {
            if G_ReleaseEntity(core::ptr::addr_of_mut!(g_entities[0])) == qfalse {
                (gi.Printf)(
                    b"%sgrab <NPC_targetname>\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                    cmd2,
                );
            }
        } else {
            G_GrabEntity(core::ptr::addr_of_mut!(g_entities[0]), cmd2 as *mut c_char);
        }
        return qtrue;
    }

    if Q_stricmp(cmd, b"knockdown\0".as_ptr() as *const c_char) == 0 {
        if (*g_cheats).integer == 0 {
            (gi.SendServerCommand)(
                0,
                b"print \"Cheats are not enabled on this server.\n\"\0".as_ptr() as *const c_char,
            );
            return qfalse;
        }
        G_Knockdown(
            core::ptr::addr_of_mut!(g_entities[0]),
            core::ptr::addr_of_mut!(g_entities[0]),
            core::ptr::addr_of!(vec3_origin),
            300.0f32,
            qtrue,
        );
        return qtrue;
    }

    if Q_stricmp(cmd, b"playerModel\0".as_ptr() as *const c_char) == 0 {
        if (gi.argc)() == 1 {
            (gi.Printf)(
                b"%sUSAGE: playerModel <NPC Name>\n       playerModel <g2model> <skinhead> <skintorso> <skinlower>\n       playerModel player (builds player from customized menu settings)\n\0"
                    .as_ptr() as *const c_char,
                S_COLOR_RED,
            );
            (gi.Printf)(
                b"playerModel = %s \0".as_ptr() as *const c_char,
                va(
                    b"%s %s %s %s\n\0".as_ptr() as *const c_char,
                    (*g_char_model).string,
                    (*g_char_skin_head).string,
                    (*g_char_skin_torso).string,
                    (*g_char_skin_legs).string,
                ),
            );
        } else if (gi.argc)() == 2 {
            G_ChangePlayerModel(core::ptr::addr_of_mut!(g_entities[0]), (gi.argv)(1));
        } else if (gi.argc)() == 5 {
            //instead of setting it directly via a command, we now store it in cvars
            //G_ChangePlayerModel( &g_entities[0], va("%s|%s|%s|%s", gi.argv(1), gi.argv(2), gi.argv(3), gi.argv(4)) );
            (gi.cvar_set)(b"g_char_model\0".as_ptr() as *const c_char, (gi.argv)(1));
            (gi.cvar_set)(b"g_char_skin_head\0".as_ptr() as *const c_char, (gi.argv)(2));
            (gi.cvar_set)(b"g_char_skin_torso\0".as_ptr() as *const c_char, (gi.argv)(3));
            (gi.cvar_set)(b"g_char_skin_legs\0".as_ptr() as *const c_char, (gi.argv)(4));
            G_InitPlayerFromCvars(core::ptr::addr_of_mut!(g_entities[0]));
        }
        return qtrue;
    }

    if Q_stricmp(cmd, b"playerTint\0".as_ptr() as *const c_char) == 0 {
        if (gi.argc)() == 4 {
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).renderInfo.customRGBA[0] =
                atoi((gi.argv)(1));
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).renderInfo.customRGBA[1] =
                atoi((gi.argv)(2));
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).renderInfo.customRGBA[2] =
                atoi((gi.argv)(3));
            (gi.cvar_set)(b"g_char_color_red\0".as_ptr() as *const c_char, (gi.argv)(1));
            (gi.cvar_set)(b"g_char_color_green\0".as_ptr() as *const c_char, (gi.argv)(2));
            (gi.cvar_set)(b"g_char_color_blue\0".as_ptr() as *const c_char, (gi.argv)(3));
        } else {
            (gi.Printf)(
                b"%sUSAGE: playerTint <red 0 - 255> <green 0 - 255> <blue 0 - 255>\n\0".as_ptr()
                    as *const c_char,
                S_COLOR_RED,
            );
            (gi.Printf)(
                b"playerTint = %s\n\0".as_ptr() as *const c_char,
                va(
                    b"%d %d %d\0".as_ptr() as *const c_char,
                    (*g_char_color_red).integer,
                    (*g_char_color_green).integer,
                    (*g_char_color_blue).integer,
                ),
            );
        }
        return qtrue;
    }
    if Q_stricmp(cmd, b"nexttestaxes\0".as_ptr() as *const c_char) == 0 {
        G_NextTestAxes();
    }

    if Q_stricmp(cmd, b"exitview\0".as_ptr() as *const c_char) == 0 {
        Svcmd_ExitView_f();
    }

    if Q_stricmp(cmd, b"iknowkungfu\0".as_ptr() as *const c_char) == 0 {
        (gi.cvar_set)(
            b"g_debugMelee\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
        );
        G_SetWeapon(core::ptr::addr_of_mut!(g_entities[0]), WP_MELEE);
        let mut i: c_int = FP_FIRST;
        while i < NUM_FORCE_POWERS {
            (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowersKnown |= 1 << i;
            if i == FP_TELEPATHY {
                (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                    [i as usize] = FORCE_LEVEL_4;
            } else {
                (*(*core::ptr::addr_of_mut!(g_entities[0])).client).ps.forcePowerLevel
                    [i as usize] = FORCE_LEVEL_3;
            }
            i += 1;
        }
    }

    qfalse
}
