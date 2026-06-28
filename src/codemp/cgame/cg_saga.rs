// Copyright (C) 2000-2002 Raven Software, Inc.
//
/*****************************************************************************
 * name:		cg_siege.c
 *
 * desc:		Clientgame-side module for Siege gametype.
 *
 * $Author: osman $
 * $Revision: 1.5 $
 *
 *****************************************************************************/

use core::ffi::{c_char, c_int};

// ============================================================================
// Externs and forward declarations
// ============================================================================

// extern void CG_LoadCISounds(clientInfo_t *ci, qboolean modelloaded); //cg_players.c
extern "C" {
    fn CG_LoadCISounds(ci: *mut crate::codemp::cgame::cg_local_h::clientInfo_t, modelloaded: c_int);
}

extern "C" {
    fn CG_DrawSiegeMessage(str: *const c_char, objectiveScreen: c_int);
    fn CG_DrawSiegeMessageNonMenu(str: *const c_char);
    fn CG_SiegeBriefingDisplay(team: c_int, dontshow: c_int);
}

// ============================================================================
// Global variables
// ============================================================================

pub static mut cgSiegeRoundState: c_int = 0;
pub static mut cgSiegeRoundTime: c_int = 0;

static mut team1: [c_char; 512] = [0; 512];
static mut team2: [c_char; 512] = [0; 512];

pub static mut team1Timed: c_int = 0;
pub static mut team2Timed: c_int = 0;

pub static mut cgSiegeTeam1PlShader: c_int = 0;
pub static mut cgSiegeTeam2PlShader: c_int = 0;

const MAX_SIEGE_INFO_SIZE: usize = 40960;

static mut cgParseObjectives: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];

pub static mut cg_siegeExtendedData: [crate::codemp::cgame::cg_local_h::siegeExtended_t; 64] =
    [crate::codemp::cgame::cg_local_h::siegeExtended_t {
        health: 0,
        maxhealth: 0,
        weapon: 0,
        ammo: 0,
        lastUpdated: 0,
    }; 64];

// ============================================================================
// Forward declarations (local functions)
// ============================================================================

fn CG_SiegeObjectiveBuffer(team: c_int, objective: c_int) -> *const c_char;

// ============================================================================
// Implementations
// ============================================================================

pub fn CG_PrecacheSiegeObjectiveAssetsForTeam(myTeam: c_int) {
    let mut teamstr: [c_char; 64] = [0; 64];
    let mut objstr: [c_char; 256] = [0; 256];
    let mut foundobjective: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];

    if unsafe { crate::codemp::cgame::cg_local_h::siege_valid == 0 } {
        unsafe {
            crate::codemp::cgame::cg_strap::CG_Error(
                b"Siege data does not exist on client!\n\0".as_ptr() as *const c_char
            );
        }
        return;
    }

    unsafe {
        if myTeam == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team1.as_ptr(),
            );
        } else {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team2.as_ptr(),
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            teamstr.as_ptr(),
            cgParseObjectives.as_mut_ptr(),
        ) != 0
        {
            let mut i: c_int = 1;
            while i < 32 {
                // eh, just try 32 I guess
                crate::codemp::qcommon::q_shared_h::Com_sprintf(
                    objstr.as_mut_ptr(),
                    core::mem::size_of_val(&objstr) as c_int,
                    b"Objective%i\0".as_ptr() as *const c_char,
                    i,
                );

                if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
                    cgParseObjectives.as_mut_ptr(),
                    objstr.as_ptr(),
                    foundobjective.as_mut_ptr(),
                ) != 0
                {
                    let mut str: [c_char; 256] = [0; 256];

                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"sound_team1\0".as_ptr() as *const c_char,
                        str.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_S_RegisterSound(str.as_ptr());
                    }
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"sound_team2\0".as_ptr() as *const c_char,
                        str.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_S_RegisterSound(str.as_ptr());
                    }
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"objgfx\0".as_ptr() as *const c_char,
                        str.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_R_RegisterShaderNoMip(str.as_ptr());
                    }
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"mapicon\0".as_ptr() as *const c_char,
                        str.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_R_RegisterShaderNoMip(str.as_ptr());
                    }
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"litmapicon\0".as_ptr() as *const c_char,
                        str.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_R_RegisterShaderNoMip(str.as_ptr());
                    }
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"donemapicon\0".as_ptr() as *const c_char,
                        str.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_R_RegisterShaderNoMip(str.as_ptr());
                    }
                } else {
                    // no more
                    break;
                }
                i += 1;
            }
        }
    }
}

pub fn CG_PrecachePlayersForSiegeTeam(team: c_int) {
    let mut stm: *mut crate::codemp::game::bg_saga_h::siegeTeam_t;
    let mut i: c_int = 0;

    unsafe {
        stm = crate::codemp::game::bg_saga_h::BG_SiegeFindThemeForTeam(team);

        if stm.is_null() {
            // invalid team/no theme for team?
            return;
        }

        while i < (*stm).numClasses {
            let scl: *mut crate::codemp::game::bg_saga_h::siegeClass_t = (*stm).classes[i as usize];

            if (*scl).forcedModel[0] != 0 {
                let mut fake: crate::codemp::cgame::cg_local_h::clientInfo_t =
                    core::mem::zeroed();

                let model_path = crate::codemp::qcommon::q_shared_h::va(
                    b"models/players/%s/model.glm\0".as_ptr() as *const c_char,
                    (*scl).forcedModel.as_ptr(),
                );
                crate::codemp::cgame::cg_syscalls::trap_R_RegisterModel(model_path);

                if (*scl).forcedSkin[0] != 0 {
                    let skin_path = crate::codemp::qcommon::q_shared_h::va(
                        b"models/players/%s/model_%s.skin\0".as_ptr() as *const c_char,
                        (*scl).forcedModel.as_ptr(),
                        (*scl).forcedSkin.as_ptr(),
                    );
                    crate::codemp::cgame::cg_syscalls::trap_R_RegisterSkin(skin_path);
                    crate::codemp::qcommon::q_shared_h::strcpy(
                        fake.skinName.as_mut_ptr(),
                        (*scl).forcedSkin.as_ptr(),
                    );
                } else {
                    crate::codemp::qcommon::q_shared_h::strcpy(
                        fake.skinName.as_mut_ptr(),
                        b"default\0".as_ptr() as *const c_char,
                    );
                }

                // precache the sounds for the model...
                CG_LoadCISounds(
                    &mut fake,
                    1, // qtrue
                );
            }

            i += 1;
        }
    }
}

pub fn CG_InitSiegeMode() {
    let mut levelname: [c_char; 256] = [0; 256];
    let mut btime: [c_char; 1024] = [0; 1024];
    let mut teams: [c_char; 2048] = [0; 2048];
    let mut teamInfo: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];
    let mut len: c_int = 0;
    let mut i: c_int = 0;
    let mut j: c_int = 0;
    let mut cl: *mut crate::codemp::game::bg_saga_h::siegeClass_t;
    let mut sTeam: *mut crate::codemp::game::bg_saga_h::siegeTeam_t;
    let mut f: crate::codemp::cgame::cg_local_h::fileHandle_t = 0;
    let mut teamIcon: [c_char; 128] = [0; 128];

    unsafe {
        if crate::codemp::cgame::cg_local_h::cgs.gametype
            != crate::codemp::qcommon::qcommon_h::GT_SIEGE
        {
            goto_failure();
        }

        crate::codemp::qcommon::q_shared_h::Com_sprintf(
            levelname.as_mut_ptr(),
            core::mem::size_of_val(&levelname) as c_int,
            b"%s\0\0".as_ptr() as *const c_char,
            crate::codemp::cgame::cg_local_h::cgs.mapname.as_ptr(),
        );

        i = crate::codemp::qcommon::q_shared_h::strlen(levelname.as_ptr()) as c_int - 1;

        while i > 0 && levelname[i as usize] != 0 && levelname[i as usize] as u8 != b'.' {
            i -= 1;
        }

        if i == 0 {
            goto_failure();
        }

        levelname[i as usize] = 0; // kill the ".bsp"

        crate::codemp::qcommon::q_shared_h::Com_sprintf(
            levelname.as_mut_ptr(),
            core::mem::size_of_val(&levelname) as c_int,
            b"%s.siege\0\0".as_ptr() as *const c_char,
            levelname.as_ptr(),
        );

        if levelname.as_ptr().is_null() || levelname[0] == 0 {
            goto_failure();
        }

        len = crate::codemp::cgame::cg_syscalls::trap_FS_FOpenFile(
            levelname.as_ptr(),
            &mut f,
            crate::codemp::qcommon::qcommon_h::FS_READ,
        );

        if f == 0 || len >= MAX_SIEGE_INFO_SIZE as c_int {
            goto_failure();
        }

        crate::codemp::cgame::cg_syscalls::trap_FS_Read(
            crate::codemp::cgame::cg_local_h::siege_info,
            len,
            f,
        );

        crate::codemp::cgame::cg_syscalls::trap_FS_FCloseFile(f);

        crate::codemp::cgame::cg_local_h::siege_valid = 1;

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            b"Teams\0".as_ptr() as *const c_char,
            teams.as_mut_ptr(),
        ) != 0
        {
            let mut buf: [c_char; 1024] = [0; 1024];

            crate::codemp::cgame::cg_syscalls::trap_Cvar_VariableStringBuffer(
                b"cg_siegeTeam1\0".as_ptr() as *const c_char,
                buf.as_mut_ptr(),
                1024,
            );
            if buf[0] != 0 && crate::codemp::qcommon::q_shared_h::Q_stricmp(
                buf.as_ptr(),
                b"none\0".as_ptr() as *const c_char,
            ) != 0
            {
                crate::codemp::qcommon::q_shared_h::strcpy(team1.as_mut_ptr(), buf.as_ptr());
            } else {
                crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                    teams.as_ptr(),
                    b"team1\0".as_ptr() as *const c_char,
                    team1.as_mut_ptr(),
                );
            }

            if team1[0] as u8 == b'@' {
                // it's a damn stringed reference.
                let mut b: [c_char; 256] = [0; 256];
                crate::codemp::cgame::cg_syscalls::trap_SP_GetStringTextString(
                    team1.as_ptr().offset(1),
                    b.as_mut_ptr(),
                    256,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    b"cg_siegeTeam1Name\0".as_ptr() as *const c_char,
                    b.as_ptr(),
                );
            } else {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    b"cg_siegeTeam1Name\0".as_ptr() as *const c_char,
                    team1.as_ptr(),
                );
            }

            crate::codemp::cgame::cg_syscalls::trap_Cvar_VariableStringBuffer(
                b"cg_siegeTeam2\0".as_ptr() as *const c_char,
                buf.as_mut_ptr(),
                1024,
            );
            if buf[0] != 0 && crate::codemp::qcommon::q_shared_h::Q_stricmp(
                buf.as_ptr(),
                b"none\0".as_ptr() as *const c_char,
            ) != 0
            {
                crate::codemp::qcommon::q_shared_h::strcpy(team2.as_mut_ptr(), buf.as_ptr());
            } else {
                crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                    teams.as_ptr(),
                    b"team2\0".as_ptr() as *const c_char,
                    team2.as_mut_ptr(),
                );
            }

            if team2[0] as u8 == b'@' {
                // it's a damn stringed reference.
                let mut b: [c_char; 256] = [0; 256];
                crate::codemp::cgame::cg_syscalls::trap_SP_GetStringTextString(
                    team2.as_ptr().offset(1),
                    b.as_mut_ptr(),
                    256,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    b"cg_siegeTeam2Name\0".as_ptr() as *const c_char,
                    b.as_ptr(),
                );
            } else {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    b"cg_siegeTeam2Name\0".as_ptr() as *const c_char,
                    team2.as_ptr(),
                );
            }
        } else {
            crate::codemp::cgame::cg_strap::CG_Error(
                b"Siege teams not defined\0".as_ptr() as *const c_char
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            team1.as_ptr(),
            teamInfo.as_mut_ptr(),
        ) != 0
        {
            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"TeamIcon\0".as_ptr() as *const c_char,
                teamIcon.as_mut_ptr(),
            ) != 0
            {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    b"team1_icon\0".as_ptr() as *const c_char,
                    teamIcon.as_ptr(),
                );
            }

            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"Timed\0".as_ptr() as *const c_char,
                btime.as_mut_ptr(),
            ) != 0
            {
                team1Timed = crate::codemp::qcommon::q_shared_h::atoi(btime.as_ptr()) * 1000;
                CG_SetSiegeTimerCvar(team1Timed);
            } else {
                team1Timed = 0;
            }
        } else {
            crate::codemp::cgame::cg_strap::CG_Error(
                b"No team entry for '%s'\n\0".as_ptr() as *const c_char,
                team1.as_ptr(),
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
            crate::codemp::cgame::cg_local_h::siege_info,
            b"mapgraphic\0".as_ptr() as *const c_char,
            teamInfo.as_mut_ptr(),
        ) != 0
        {
            crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                b"siege_mapgraphic\0".as_ptr() as *const c_char,
                teamInfo.as_ptr(),
            );
        } else {
            crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                b"siege_mapgraphic\0".as_ptr() as *const c_char,
                b"gfx/mplevels/siege1_hoth\0".as_ptr() as *const c_char,
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
            crate::codemp::cgame::cg_local_h::siege_info,
            b"missionname\0".as_ptr() as *const c_char,
            teamInfo.as_mut_ptr(),
        ) != 0
        {
            crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                b"siege_missionname\0".as_ptr() as *const c_char,
                teamInfo.as_ptr(),
            );
        } else {
            crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                b"siege_missionname\0".as_ptr() as *const c_char,
                b" \0".as_ptr() as *const c_char,
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            team2.as_ptr(),
            teamInfo.as_mut_ptr(),
        ) != 0
        {
            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"TeamIcon\0".as_ptr() as *const c_char,
                teamIcon.as_mut_ptr(),
            ) != 0
            {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    b"team2_icon\0".as_ptr() as *const c_char,
                    teamIcon.as_ptr(),
                );
            }

            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"Timed\0".as_ptr() as *const c_char,
                btime.as_mut_ptr(),
            ) != 0
            {
                team2Timed = crate::codemp::qcommon::q_shared_h::atoi(btime.as_ptr()) * 1000;
                CG_SetSiegeTimerCvar(team2Timed);
            } else {
                team2Timed = 0;
            }
        } else {
            crate::codemp::cgame::cg_strap::CG_Error(
                b"No team entry for '%s'\n\0".as_ptr() as *const c_char,
                team2.as_ptr(),
            );
        }

        // Load the player class types
        crate::codemp::game::bg_saga_h::BG_SiegeLoadClasses(core::ptr::null_mut());

        if crate::codemp::game::bg_saga_h::bgNumSiegeClasses == 0 {
            // We didn't find any?!
            crate::codemp::cgame::cg_strap::CG_Error(
                b"Couldn't find any player classes for Siege\0".as_ptr() as *const c_char
            );
        }

        // Now load the teams since we have class data.
        crate::codemp::game::bg_saga_h::BG_SiegeLoadTeams();

        if crate::codemp::game::bg_saga_h::bgNumSiegeTeams == 0 {
            // React same as with classes.
            crate::codemp::cgame::cg_strap::CG_Error(
                b"Couldn't find any player teams for Siege\0".as_ptr() as *const c_char
            );
        }

        // Get and set the team themes for each team. This will control which classes can be
        // used on each team.
        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            team1.as_ptr(),
            teamInfo.as_mut_ptr(),
        ) != 0
        {
            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"UseTeam\0".as_ptr() as *const c_char,
                btime.as_mut_ptr(),
            ) != 0
            {
                crate::codemp::game::bg_saga_h::BG_SiegeSetTeamTheme(
                    crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1,
                    btime.as_ptr(),
                );
            }
            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"FriendlyShader\0".as_ptr() as *const c_char,
                btime.as_mut_ptr(),
            ) != 0
            {
                cgSiegeTeam1PlShader = crate::codemp::cgame::cg_syscalls::trap_R_RegisterShaderNoMip(btime.as_ptr());
            } else {
                cgSiegeTeam1PlShader = 0;
            }
        }
        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            team2.as_ptr(),
            teamInfo.as_mut_ptr(),
        ) != 0
        {
            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"UseTeam\0".as_ptr() as *const c_char,
                btime.as_mut_ptr(),
            ) != 0
            {
                crate::codemp::game::bg_saga_h::BG_SiegeSetTeamTheme(
                    crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2,
                    btime.as_ptr(),
                );
            }
            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                teamInfo.as_ptr(),
                b"FriendlyShader\0".as_ptr() as *const c_char,
                btime.as_mut_ptr(),
            ) != 0
            {
                cgSiegeTeam2PlShader = crate::codemp::cgame::cg_syscalls::trap_R_RegisterShaderNoMip(btime.as_ptr());
            } else {
                cgSiegeTeam2PlShader = 0;
            }
        }

        // Now go through the classes used by the loaded teams and try to precache
        // any forced models or forced skins.
        i = crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1;

        while i <= crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2 {
            j = 0;
            sTeam = crate::codemp::game::bg_saga_h::BG_SiegeFindThemeForTeam(i);

            if sTeam.is_null() {
                i += 1;
                continue;
            }

            // Get custom team shaders while we're at it.
            if i == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
                cgSiegeTeam1PlShader = (*sTeam).friendlyShader;
            } else if i == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2 {
                cgSiegeTeam2PlShader = (*sTeam).friendlyShader;
            }

            while j < (*sTeam).numClasses {
                cl = (*sTeam).classes[j as usize];

                if (*cl).forcedModel[0] != 0 {
                    // This class has a forced model, so precache it.
                    let model_path = crate::codemp::qcommon::q_shared_h::va(
                        b"models/players/%s/model.glm\0".as_ptr() as *const c_char,
                        (*cl).forcedModel.as_ptr(),
                    );
                    crate::codemp::cgame::cg_syscalls::trap_R_RegisterModel(model_path);

                    if (*cl).forcedSkin[0] != 0 {
                        // also has a forced skin, precache it.
                        let mut useSkinName: *const c_char;

                        if !crate::codemp::qcommon::q_shared_h::strchr(
                            (*cl).forcedSkin.as_ptr(),
                            b'|' as c_int,
                        )
                        .is_null()
                        {
                            // three part skin
                            useSkinName = crate::codemp::qcommon::q_shared_h::va(
                                b"models/players/%s/|%s\0".as_ptr() as *const c_char,
                                (*cl).forcedModel.as_ptr(),
                                (*cl).forcedSkin.as_ptr(),
                            );
                        } else {
                            useSkinName = crate::codemp::qcommon::q_shared_h::va(
                                b"models/players/%s/model_%s.skin\0".as_ptr() as *const c_char,
                                (*cl).forcedModel.as_ptr(),
                                (*cl).forcedSkin.as_ptr(),
                            );
                        }

                        crate::codemp::cgame::cg_syscalls::trap_R_RegisterSkin(useSkinName);
                    }
                }

                j += 1;
            }
            i += 1;
        }

        // precache saber data for classes that use sabers on both teams
        crate::codemp::game::bg_saga_h::BG_PrecacheSabersForSiegeTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1);
        crate::codemp::game::bg_saga_h::BG_PrecacheSabersForSiegeTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2);

        CG_PrecachePlayersForSiegeTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1);
        CG_PrecachePlayersForSiegeTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2);

        CG_PrecachePlayersForSiegeTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1);
        CG_PrecachePlayersForSiegeTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2);

        CG_PrecacheSiegeObjectiveAssetsForTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1);
        CG_PrecacheSiegeObjectiveAssetsForTeam(crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2);

        return;
    }
}

#[inline(always)]
fn goto_failure() {
    unsafe {
        crate::codemp::cgame::cg_local_h::siege_valid = 0;
    }
}

#[inline]
fn CG_SiegeObjectiveBuffer(team: c_int, objective: c_int) -> *const c_char {
    static mut buf: [c_char; 8192] = [0; 8192];
    let mut teamstr: [c_char; 1024] = [0; 1024];

    unsafe {
        if team == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team1.as_ptr(),
            );
        } else {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team2.as_ptr(),
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            teamstr.as_ptr(),
            cgParseObjectives.as_mut_ptr(),
        ) != 0
        {
            // found the team group
            if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
                cgParseObjectives.as_mut_ptr(),
                crate::codemp::qcommon::q_shared_h::va(
                    b"Objective%i\0".as_ptr() as *const c_char,
                    objective,
                ),
                buf.as_mut_ptr(),
            ) != 0
            {
                // found the objective group
                return buf.as_ptr();
            }
        }

        core::ptr::null()
    }
}

pub fn CG_ParseSiegeObjectiveStatus(str: *const c_char) {
    let mut i: c_int = 0;
    let mut team: c_int = crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1;
    let mut cvarName: *const c_char;
    let mut s: *const c_char;
    let mut objectiveNum: c_int = 0;

    unsafe {
        if str.is_null() || *str == 0 {
            return;
        }

        while *str.offset(i as isize) != 0 {
            if *str.offset(i as isize) as u8 == b'|' {
                // switch over to team2, this is the next section
                team = crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2;
                objectiveNum = 0;
            } else if *str.offset(i as isize) as u8 == b'-' {
                objectiveNum += 1;
                i += 1;

                cvarName = crate::codemp::qcommon::q_shared_h::va(
                    b"team%i_objective%i\0".as_ptr() as *const c_char,
                    team,
                    objectiveNum,
                );
                if *str.offset(i as isize) as u8 == b'1' {
                    // it's completed
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, b"1\0".as_ptr() as *const c_char);
                } else {
                    // otherwise assume it is not
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, b"0\0".as_ptr() as *const c_char);
                }

                s = CG_SiegeObjectiveBuffer(team, objectiveNum);
                if !s.is_null() && *s != 0 {
                    // now set the description and graphic cvars to by read by the menu
                    let mut buffer: [c_char; 8192] = [0; 8192];

                    cvarName = crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_longdesc\0".as_ptr() as *const c_char,
                        team,
                        objectiveNum,
                    );
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        s,
                        b"objdesc\0".as_ptr() as *const c_char,
                        buffer.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, buffer.as_ptr());
                    } else {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, b"UNSPECIFIED\0".as_ptr() as *const c_char);
                    }

                    cvarName = crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_gfx\0".as_ptr() as *const c_char,
                        team,
                        objectiveNum,
                    );
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        s,
                        b"objgfx\0".as_ptr() as *const c_char,
                        buffer.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, buffer.as_ptr());
                    } else {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, b"UNSPECIFIED\0".as_ptr() as *const c_char);
                    }

                    cvarName = crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_mapicon\0".as_ptr() as *const c_char,
                        team,
                        objectiveNum,
                    );
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        s,
                        b"mapicon\0".as_ptr() as *const c_char,
                        buffer.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, buffer.as_ptr());
                    } else {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, b"UNSPECIFIED\0".as_ptr() as *const c_char);
                    }

                    cvarName = crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_litmapicon\0".as_ptr() as *const c_char,
                        team,
                        objectiveNum,
                    );
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        s,
                        b"litmapicon\0".as_ptr() as *const c_char,
                        buffer.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, buffer.as_ptr());
                    } else {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, b"UNSPECIFIED\0".as_ptr() as *const c_char);
                    }

                    cvarName = crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_donemapicon\0".as_ptr() as *const c_char,
                        team,
                        objectiveNum,
                    );
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        s,
                        b"donemapicon\0".as_ptr() as *const c_char,
                        buffer.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, buffer.as_ptr());
                    } else {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, b"UNSPECIFIED\0".as_ptr() as *const c_char);
                    }

                    cvarName = crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_mappos\0".as_ptr() as *const c_char,
                        team,
                        objectiveNum,
                    );
                    if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        s,
                        b"mappos\0".as_ptr() as *const c_char,
                        buffer.as_mut_ptr(),
                    ) != 0
                    {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(cvarName, buffer.as_ptr());
                    } else {
                        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                            cvarName,
                            b"0 0 32 32\0".as_ptr() as *const c_char,
                        );
                    }
                }
            }
            i += 1;
        }

        if crate::codemp::cgame::cg_local_h::cg.predictedPlayerState.persistant
            [crate::codemp::qcommon::qcommon_h::PERS_TEAM as usize]
            != crate::codemp::qcommon::qcommon_h::TEAM_SPECTATOR
        {
            // update menu cvars
            CG_SiegeBriefingDisplay(
                crate::codemp::cgame::cg_local_h::cg.predictedPlayerState.persistant
                    [crate::codemp::qcommon::qcommon_h::PERS_TEAM as usize],
                1,
            );
        }
    }
}

pub fn CG_SiegeRoundOver(ent: *mut crate::codemp::cgame::cg_local_h::centity_t, won: c_int) {
    let mut myTeam: c_int;
    let mut teamstr: [c_char; 64] = [0; 64];
    let mut appstring: [c_char; 1024] = [0; 1024];
    let mut soundstr: [c_char; 1024] = [0; 1024];
    let mut success: c_int = 0;
    let mut ps: *mut crate::codemp::qcommon::q_shared_h::playerState_t;

    unsafe {
        if crate::codemp::cgame::cg_local_h::siege_valid == 0 {
            crate::codemp::cgame::cg_strap::CG_Error(
                b"ERROR: Siege data does not exist on client!\n\0".as_ptr() as *const c_char
            );
            return;
        }

        if !crate::codemp::cgame::cg_local_h::cg.snap.is_null() {
            // this should always be true, if it isn't though use the predicted ps as a fallback
            ps = &mut (*crate::codemp::cgame::cg_local_h::cg.snap).ps;
        } else {
            ps = &mut crate::codemp::cgame::cg_local_h::cg.predictedPlayerState;
        }

        if ps.is_null() {
            assert!(false);
            return;
        }

        myTeam = (*ps).persistant[crate::codemp::qcommon::qcommon_h::PERS_TEAM as usize];

        if myTeam == crate::codemp::qcommon::qcommon_h::TEAM_SPECTATOR {
            return;
        }

        if myTeam == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team1.as_ptr(),
            );
        } else {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team2.as_ptr(),
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            teamstr.as_ptr(),
            cgParseObjectives.as_mut_ptr(),
        ) != 0
        {
            if won == myTeam {
                success = crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                    cgParseObjectives.as_ptr(),
                    b"wonround\0".as_ptr() as *const c_char,
                    appstring.as_mut_ptr(),
                );
            } else {
                success = crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                    cgParseObjectives.as_ptr(),
                    b"lostround\0".as_ptr() as *const c_char,
                    appstring.as_mut_ptr(),
                );
            }

            if success != 0 {
                CG_DrawSiegeMessage(appstring.as_ptr(), 0);
            }

            appstring[0] = 0;
            soundstr[0] = 0;

            if myTeam == won {
                crate::codemp::qcommon::q_shared_h::Com_sprintf(
                    teamstr.as_mut_ptr(),
                    core::mem::size_of_val(&teamstr) as c_int,
                    b"roundover_sound_wewon\0".as_ptr() as *const c_char,
                );
            } else {
                crate::codemp::qcommon::q_shared_h::Com_sprintf(
                    teamstr.as_mut_ptr(),
                    core::mem::size_of_val(&teamstr) as c_int,
                    b"roundover_sound_welost\0".as_ptr() as *const c_char,
                );
            }

            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                cgParseObjectives.as_ptr(),
                teamstr.as_ptr(),
                appstring.as_mut_ptr(),
            ) != 0
            {
                crate::codemp::qcommon::q_shared_h::Com_sprintf(
                    soundstr.as_mut_ptr(),
                    core::mem::size_of_val(&soundstr) as c_int,
                    appstring.as_ptr(),
                );
            }
            /*
            else
            {
                if (myTeam != won)
                {
                    Com_sprintf(soundstr, sizeof(soundstr), DEFAULT_LOSE_ROUND);
                }
                else
                {
                    Com_sprintf(soundstr, sizeof(soundstr), DEFAULT_WIN_ROUND);
                }
            }
            */

            if soundstr[0] != 0 {
                crate::codemp::cgame::cg_syscalls::trap_S_StartLocalSound(
                    crate::codemp::cgame::cg_syscalls::trap_S_RegisterSound(soundstr.as_ptr()),
                    crate::codemp::qcommon::qcommon_h::CHAN_ANNOUNCER,
                );
            }
        }
    }
}

pub fn CG_SiegeGetObjectiveDescription(
    team: c_int,
    objective: c_int,
    buffer: *mut c_char,
) {
    let mut teamstr: [c_char; 1024] = [0; 1024];
    let mut objectiveStr: [c_char; 8192] = [0; 8192];

    unsafe {
        *buffer = 0; // set to 0 ahead of time in case we fail to find the objective group/name

        if team == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team1.as_ptr(),
            );
        } else {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team2.as_ptr(),
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            teamstr.as_ptr(),
            cgParseObjectives.as_mut_ptr(),
        ) != 0
        {
            // found the team group
            if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
                cgParseObjectives.as_mut_ptr(),
                crate::codemp::qcommon::q_shared_h::va(
                    b"Objective%i\0".as_ptr() as *const c_char,
                    objective,
                ),
                objectiveStr.as_mut_ptr(),
            ) != 0
            {
                // found the objective group
                // Parse the name right into the buffer.
                crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                    objectiveStr.as_ptr(),
                    b"goalname\0".as_ptr() as *const c_char,
                    buffer,
                );
            }
        }
    }
}

pub fn CG_SiegeGetObjectiveFinal(team: c_int, objective: c_int) -> c_int {
    let mut finalStr: [c_char; 64] = [0; 64];
    let mut teamstr: [c_char; 1024] = [0; 1024];
    let mut objectiveStr: [c_char; 8192] = [0; 8192];

    unsafe {
        if team == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team1.as_ptr(),
            );
        } else {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team2.as_ptr(),
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            teamstr.as_ptr(),
            cgParseObjectives.as_mut_ptr(),
        ) != 0
        {
            // found the team group
            if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
                cgParseObjectives.as_mut_ptr(),
                crate::codemp::qcommon::q_shared_h::va(
                    b"Objective%i\0".as_ptr() as *const c_char,
                    objective,
                ),
                objectiveStr.as_mut_ptr(),
            ) != 0
            {
                // found the objective group
                // Parse the name right into the buffer.
                crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                    objectiveStr.as_ptr(),
                    b"final\0".as_ptr() as *const c_char,
                    finalStr.as_mut_ptr(),
                );
                return crate::codemp::qcommon::q_shared_h::atoi(finalStr.as_ptr());
            }
        }
    }
    0
}

pub fn CG_SiegeBriefingDisplay(team: c_int, dontshow: c_int) {
    let mut teamstr: [c_char; 64] = [0; 64];
    let mut briefing: [c_char; 8192] = [0; 8192];
    let mut properValue: [c_char; 1024] = [0; 1024];
    let mut objectiveDesc: [c_char; 1024] = [0; 1024];
    let mut i: c_int = 1;
    let mut useTeam: c_int = team;
    let mut primary: c_int = 0;

    unsafe {
        if crate::codemp::cgame::cg_local_h::siege_valid == 0 {
            return;
        }

        if team == crate::codemp::qcommon::qcommon_h::TEAM_SPECTATOR {
            return;
        }

        if team == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team1.as_ptr(),
            );
        } else {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team2.as_ptr(),
            );
        }

        if useTeam != crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1
            && useTeam != crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2
        {
            // This shouldn't be happening. But just fall back to team 2 anyway.
            useTeam = crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM2;
        }

        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
            crate::codemp::qcommon::q_shared_h::va(b"siege_primobj_inuse\0".as_ptr() as *const c_char),
            b"0\0".as_ptr() as *const c_char,
        );

        while i < 16 {
            // do up to 16 objectives I suppose
            // Get the value for this objective on this team
            // Now set the cvar for the menu to display.

            // primary = (CG_SiegeGetObjectiveFinal(useTeam, i)>-1)?qtrue:qfalse;
            primary = if CG_SiegeGetObjectiveFinal(useTeam, i) > 0 { 1 } else { 0 };

            properValue[0] = 0;
            crate::codemp::cgame::cg_syscalls::trap_Cvar_VariableStringBuffer(
                crate::codemp::qcommon::q_shared_h::va(
                    b"team%i_objective%i\0".as_ptr() as *const c_char,
                    useTeam,
                    i,
                ),
                properValue.as_mut_ptr(),
                1024,
            );
            if primary != 0 {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(b"siege_primobj\0".as_ptr() as *const c_char),
                    properValue.as_ptr(),
                );
            } else {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i\0".as_ptr() as *const c_char,
                        i,
                    ),
                    properValue.as_ptr(),
                );
            }

            // Now set the long desc cvar for the menu to display.
            properValue[0] = 0;
            crate::codemp::cgame::cg_syscalls::trap_Cvar_VariableStringBuffer(
                crate::codemp::qcommon::q_shared_h::va(
                    b"team%i_objective%i_longdesc\0".as_ptr() as *const c_char,
                    useTeam,
                    i,
                ),
                properValue.as_mut_ptr(),
                1024,
            );
            if primary != 0 {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(b"siege_primobj_longdesc\0".as_ptr() as *const c_char),
                    properValue.as_ptr(),
                );
            } else {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_longdesc\0".as_ptr() as *const c_char,
                        i,
                    ),
                    properValue.as_ptr(),
                );
            }

            // Now set the gfx cvar for the menu to display.
            properValue[0] = 0;
            crate::codemp::cgame::cg_syscalls::trap_Cvar_VariableStringBuffer(
                crate::codemp::qcommon::q_shared_h::va(
                    b"team%i_objective%i_gfx\0".as_ptr() as *const c_char,
                    useTeam,
                    i,
                ),
                properValue.as_mut_ptr(),
                1024,
            );
            if primary != 0 {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(b"siege_primobj_gfx\0".as_ptr() as *const c_char),
                    properValue.as_ptr(),
                );
            } else {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_gfx\0".as_ptr() as *const c_char,
                        i,
                    ),
                    properValue.as_ptr(),
                );
            }

            // Now set the mapicon cvar for the menu to display.
            properValue[0] = 0;
            crate::codemp::cgame::cg_syscalls::trap_Cvar_VariableStringBuffer(
                crate::codemp::qcommon::q_shared_h::va(
                    b"team%i_objective%i_mapicon\0".as_ptr() as *const c_char,
                    useTeam,
                    i,
                ),
                properValue.as_mut_ptr(),
                1024,
            );
            if primary != 0 {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(b"siege_primobj_mapicon\0".as_ptr() as *const c_char),
                    properValue.as_ptr(),
                );
            } else {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_mapicon\0".as_ptr() as *const c_char,
                        i,
                    ),
                    properValue.as_ptr(),
                );
            }

            // Now set the mappos cvar for the menu to display.
            properValue[0] = 0;
            crate::codemp::cgame::cg_syscalls::trap_Cvar_VariableStringBuffer(
                crate::codemp::qcommon::q_shared_h::va(
                    b"team%i_objective%i_mappos\0".as_ptr() as *const c_char,
                    useTeam,
                    i,
                ),
                properValue.as_mut_ptr(),
                1024,
            );
            if primary != 0 {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(b"siege_primobj_mappos\0".as_ptr() as *const c_char),
                    properValue.as_ptr(),
                );
            } else {
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_mappos\0".as_ptr() as *const c_char,
                        i,
                    ),
                    properValue.as_ptr(),
                );
            }

            // Now set the description cvar for the objective
            CG_SiegeGetObjectiveDescription(useTeam, i, objectiveDesc.as_mut_ptr());

            if objectiveDesc[0] != 0 {
                // found a valid objective description
                if primary != 0 {
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                        crate::codemp::qcommon::q_shared_h::va(b"siege_primobj_desc\0".as_ptr() as *const c_char),
                        objectiveDesc.as_ptr(),
                    );
                    // this one is marked not in use because it gets primobj
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                        crate::codemp::qcommon::q_shared_h::va(
                            b"siege_objective%i_inuse\0".as_ptr() as *const c_char,
                            i,
                        ),
                        b"0\0".as_ptr() as *const c_char,
                    );
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                        crate::codemp::qcommon::q_shared_h::va(b"siege_primobj_inuse\0".as_ptr() as *const c_char),
                        b"1\0".as_ptr() as *const c_char,
                    );

                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                        crate::codemp::qcommon::q_shared_h::va(
                            b"team%i_objective%i_inuse\0".as_ptr() as *const c_char,
                            useTeam,
                            i,
                        ),
                        b"1\0".as_ptr() as *const c_char,
                    );
                } else {
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                        crate::codemp::qcommon::q_shared_h::va(
                            b"siege_objective%i_desc\0".as_ptr() as *const c_char,
                            i,
                        ),
                        objectiveDesc.as_ptr(),
                    );
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                        crate::codemp::qcommon::q_shared_h::va(
                            b"siege_objective%i_inuse\0".as_ptr() as *const c_char,
                            i,
                        ),
                        b"2\0".as_ptr() as *const c_char,
                    );
                    crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                        crate::codemp::qcommon::q_shared_h::va(
                            b"team%i_objective%i_inuse\0".as_ptr() as *const c_char,
                            useTeam,
                            i,
                        ),
                        b"2\0".as_ptr() as *const c_char,
                    );
                }
            } else {
                // didn't find one, so set the "inuse" cvar to 0 for the objective and mark it non-complete.
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_inuse\0".as_ptr() as *const c_char,
                        i,
                    ),
                    b"0\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i\0".as_ptr() as *const c_char,
                        i,
                    ),
                    b"0\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_inuse\0".as_ptr() as *const c_char,
                        useTeam,
                        i,
                    ),
                    b"0\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i\0".as_ptr() as *const c_char,
                        useTeam,
                        i,
                    ),
                    b"0\0".as_ptr() as *const c_char,
                );

                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_mappos\0".as_ptr() as *const c_char,
                        i,
                    ),
                    b"\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_mappos\0".as_ptr() as *const c_char,
                        useTeam,
                        i,
                    ),
                    b"\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_gfx\0".as_ptr() as *const c_char,
                        i,
                    ),
                    b"\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_gfx\0".as_ptr() as *const c_char,
                        useTeam,
                        i,
                    ),
                    b"\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"siege_objective%i_mapicon\0".as_ptr() as *const c_char,
                        i,
                    ),
                    b"\0".as_ptr() as *const c_char,
                );
                crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
                    crate::codemp::qcommon::q_shared_h::va(
                        b"team%i_objective%i_mapicon\0".as_ptr() as *const c_char,
                        useTeam,
                        i,
                    ),
                    b"\0".as_ptr() as *const c_char,
                );
            }

            i += 1;
        }

        if dontshow != 0 {
            return;
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            teamstr.as_ptr(),
            cgParseObjectives.as_mut_ptr(),
        ) != 0
        {
            if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                cgParseObjectives.as_ptr(),
                b"briefing\0".as_ptr() as *const c_char,
                briefing.as_mut_ptr(),
            ) != 0
            {
                CG_DrawSiegeMessage(briefing.as_ptr(), 1);
            }
        }
    }
}

pub fn CG_SiegeObjectiveCompleted(
    ent: *mut crate::codemp::cgame::cg_local_h::centity_t,
    won: c_int,
    objectivenum: c_int,
) {
    let mut myTeam: c_int;
    let mut teamstr: [c_char; 64] = [0; 64];
    let mut objstr: [c_char; 256] = [0; 256];
    let mut foundobjective: [c_char; MAX_SIEGE_INFO_SIZE] = [0; MAX_SIEGE_INFO_SIZE];
    let mut appstring: [c_char; 1024] = [0; 1024];
    let mut soundstr: [c_char; 1024] = [0; 1024];
    let mut success: c_int = 0;
    let mut ps: *mut crate::codemp::qcommon::q_shared_h::playerState_t;

    unsafe {
        if crate::codemp::cgame::cg_local_h::siege_valid == 0 {
            crate::codemp::cgame::cg_strap::CG_Error(
                b"Siege data does not exist on client!\n\0".as_ptr() as *const c_char
            );
            return;
        }

        if !crate::codemp::cgame::cg_local_h::cg.snap.is_null() {
            // this should always be true, if it isn't though use the predicted ps as a fallback
            ps = &mut (*crate::codemp::cgame::cg_local_h::cg.snap).ps;
        } else {
            ps = &mut crate::codemp::cgame::cg_local_h::cg.predictedPlayerState;
        }

        if ps.is_null() {
            assert!(false);
            return;
        }

        myTeam = (*ps).persistant[crate::codemp::qcommon::qcommon_h::PERS_TEAM as usize];

        if myTeam == crate::codemp::qcommon::qcommon_h::TEAM_SPECTATOR {
            return;
        }

        if won == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team1.as_ptr(),
            );
        } else {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                teamstr.as_mut_ptr(),
                core::mem::size_of_val(&teamstr) as c_int,
                team2.as_ptr(),
            );
        }

        if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
            crate::codemp::cgame::cg_local_h::siege_info,
            teamstr.as_ptr(),
            cgParseObjectives.as_mut_ptr(),
        ) != 0
        {
            crate::codemp::qcommon::q_shared_h::Com_sprintf(
                objstr.as_mut_ptr(),
                core::mem::size_of_val(&objstr) as c_int,
                b"Objective%i\0".as_ptr() as *const c_char,
                objectivenum,
            );

            if crate::codemp::game::bg_saga_h::BG_SiegeGetValueGroup(
                cgParseObjectives.as_mut_ptr(),
                objstr.as_ptr(),
                foundobjective.as_mut_ptr(),
            ) != 0
            {
                if myTeam == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
                    success = crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"message_team1\0".as_ptr() as *const c_char,
                        appstring.as_mut_ptr(),
                    );
                } else {
                    success = crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                        foundobjective.as_ptr(),
                        b"message_team2\0".as_ptr() as *const c_char,
                        appstring.as_mut_ptr(),
                    );
                }

                if success != 0 {
                    CG_DrawSiegeMessageNonMenu(appstring.as_ptr());
                }

                appstring[0] = 0;
                soundstr[0] = 0;

                if myTeam == crate::codemp::qcommon::qcommon_h::SIEGETEAM_TEAM1 {
                    crate::codemp::qcommon::q_shared_h::Com_sprintf(
                        teamstr.as_mut_ptr(),
                        core::mem::size_of_val(&teamstr) as c_int,
                        b"sound_team1\0".as_ptr() as *const c_char,
                    );
                } else {
                    crate::codemp::qcommon::q_shared_h::Com_sprintf(
                        teamstr.as_mut_ptr(),
                        core::mem::size_of_val(&teamstr) as c_int,
                        b"sound_team2\0".as_ptr() as *const c_char,
                    );
                }

                if crate::codemp::game::bg_saga_h::BG_SiegeGetPairedValue(
                    foundobjective.as_ptr(),
                    teamstr.as_ptr(),
                    appstring.as_mut_ptr(),
                ) != 0
                {
                    crate::codemp::qcommon::q_shared_h::Com_sprintf(
                        soundstr.as_mut_ptr(),
                        core::mem::size_of_val(&soundstr) as c_int,
                        appstring.as_ptr(),
                    );
                }
                /*
                else
                {
                    if (myTeam != won)
                    {
                        Com_sprintf(soundstr, sizeof(soundstr), DEFAULT_LOSE_OBJECTIVE);
                    }
                    else
                    {
                        Com_sprintf(soundstr, sizeof(soundstr), DEFAULT_WIN_OBJECTIVE);
                    }
                }
                */

                if soundstr[0] != 0 {
                    crate::codemp::cgame::cg_syscalls::trap_S_StartLocalSound(
                        crate::codemp::cgame::cg_syscalls::trap_S_RegisterSound(soundstr.as_ptr()),
                        crate::codemp::qcommon::qcommon_h::CHAN_ANNOUNCER,
                    );
                }
            }
        }
    }
}

// parse a single extended siege data entry
pub fn CG_ParseSiegeExtendedDataEntry(conStr: *const c_char) {
    let mut s: [c_char; 1024] = [0; 1024];
    let mut str: *const c_char = conStr;
    let mut argParses: c_int = 0;
    let mut i: c_int = 0;
    let mut maxAmmo: c_int = 0;
    let mut clNum: c_int = -1;
    let mut health: c_int = 1;
    let mut maxhealth: c_int = 1;
    let mut ammo: c_int = 1;
    let mut cent: *mut crate::codemp::cgame::cg_local_h::centity_t;

    unsafe {
        if conStr.is_null() || *conStr == 0 {
            return;
        }

        while *str != 0 && argParses < 4 {
            i = 0;
            while *str != 0 && *str as u8 != b'|' {
                s[i as usize] = *str;
                i += 1;
                str = str.offset(1);
            }
            s[i as usize] = 0;
            match argParses {
                0 => clNum = crate::codemp::qcommon::q_shared_h::atoi(s.as_ptr()),
                1 => health = crate::codemp::qcommon::q_shared_h::atoi(s.as_ptr()),
                2 => maxhealth = crate::codemp::qcommon::q_shared_h::atoi(s.as_ptr()),
                3 => ammo = crate::codemp::qcommon::q_shared_h::atoi(s.as_ptr()),
                _ => {}
            }
            argParses += 1;
            str = str.offset(1);
        }

        if clNum < 0 || clNum >= 64 {
            return;
        }

        cg_siegeExtendedData[clNum as usize].health = health;
        cg_siegeExtendedData[clNum as usize].maxhealth = maxhealth;
        cg_siegeExtendedData[clNum as usize].ammo = ammo;

        cent = &mut crate::codemp::cgame::cg_local_h::cg_entities[clNum as usize];

        maxAmmo = crate::codemp::game::bg_public_h::ammoData
            [crate::codemp::game::bg_public_h::weaponData[(*cent).currentState.weapon as usize]
                .ammoIndex as usize]
                .max;
        if ((*cent).currentState.eFlags & crate::codemp::qcommon::qcommon_h::EF_DOUBLE_AMMO) != 0 {
            maxAmmo = (maxAmmo as f32 * 2.0f32) as c_int;
        }
        if ammo >= 0 && ammo <= maxAmmo {
            // assure the weapon number is valid and not over max
            // keep the weapon so if it changes before our next ext data update we'll know
            // that the ammo is not applicable.
            cg_siegeExtendedData[clNum as usize].weapon = (*cent).currentState.weapon;
        } else {
            // not valid? Oh well, just invalidate the weapon too then so we don't display ammo
            cg_siegeExtendedData[clNum as usize].weapon = -1;
        }

        cg_siegeExtendedData[clNum as usize].lastUpdated = crate::codemp::cgame::cg_local_h::cg.time;
    }
}

// parse incoming siege data, see counterpart in g_saga.c
pub fn CG_ParseSiegeExtendedData() {
    let mut numEntries: c_int = crate::codemp::cgame::cg_syscalls::trap_Argc();
    let mut i: c_int = 0;

    unsafe {
        if numEntries < 1 {
            assert!(false, "Bad numEntries for sxd");
            return;
        }

        while i < numEntries {
            CG_ParseSiegeExtendedDataEntry(crate::codemp::cgame::cg_syscalls::CG_Argv(i + 1));
            i += 1;
        }
    }
}

pub fn CG_SetSiegeTimerCvar(msec: c_int) {
    let mut seconds: c_int = msec / 1000;
    let mut mins: c_int = seconds / 60;
    seconds -= mins * 60;
    let mut tens: c_int = seconds / 10;
    seconds -= tens * 10;

    unsafe {
        crate::codemp::cgame::cg_syscalls::trap_Cvar_Set(
            b"ui_siegeTimer\0".as_ptr() as *const c_char,
            crate::codemp::qcommon::q_shared_h::va(
                b"%i:%i%i\0".as_ptr() as *const c_char,
                mins,
                tens,
                seconds,
            ),
        );
    }
}
