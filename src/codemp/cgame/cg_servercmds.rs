// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_servercmds.c -- reliably sequenced text commands sent by the server
// these are processed at snapshot transition time, so there will definately
// be a valid snapshot this frame

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_char, c_void};

// Imports mirroring oracle #includes
use crate::codemp::cgame::cg_local_h::*;   // cg_local.h -> cg_t, cgs_t, centity_t, clientInfo_t, animation_t, cvar_t, qboolean, sfxHandle_t (via chain)
use crate::codemp::ui::menudef_h::*;        // ../../ui/menudef.h (trust-import; module not yet on disk)
use crate::codemp::cgame::cg_lights_h::*;  // cg_lights.h
use crate::codemp::ghoul2::g2_h::*;        // ..\ghoul2\g2.h
use crate::codemp::ui::ui_public_h::*;     // ../ui/ui_public.h

// ============================================================================
// Extern declarations for stubs and cross-module references
// ============================================================================

extern "C" {
    fn atoi(s: *const c_char) -> c_int;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strchr(s: *const c_char, c: c_int) -> *const c_char;
}

// Extern references to globals and functions from other modules
extern "C" {
    static mut cg: cg_t;
    static mut cgs: cgs_t;
    static mut cg_entities: [centity_t; 2048];  // MAX_GENTITIES assumed to be 2048

    static mut numSortedTeamPlayers: c_int;
    static mut sortedTeamPlayers: [c_int; 64];  // MAX_CLIENTS assumed to be 64

    static mut cgSiegeRoundState: c_int;
    static mut cgSiegeRoundTime: c_int;
    static mut cg_beatingSiegeTime: c_int;
    static mut cg_siegeWinTeam: c_int;

    static cg_customSoundNames: [*const c_char; 256];  // MAX_CUSTOM_SOUNDS
    static cg_customCombatSoundNames: [*const c_char; 256];  // MAX_CUSTOM_COMBAT_SOUNDS
    static cg_customExtraSoundNames: [*const c_char; 256];  // MAX_CUSTOM_EXTRA_SOUNDS
    static cg_customJediSoundNames: [*const c_char; 256];  // MAX_CUSTOM_JEDI_SOUNDS
    static cg_customDuelSoundNames: [*const c_char; 256];  // MAX_CUSTOM_DUEL_SOUNDS
    static bg_customSiegeSoundNames: [*const c_char; 256];  // MAX_CUSTOM_SIEGE_SOUNDS

    static cg_showmiss: cvar_t;
    static cg_teamChatsOnly: cvar_t;

    // Function declarations
    fn CG_Argv(arg: c_int) -> *const c_char;
    fn CG_ConfigString(index: c_int) -> *const c_char;
    fn CG_SetScoreSelection(p: *const c_void);
    fn CG_Printf(fmt: *const c_char, ...);
    fn CG_StartMusic(force: qboolean);
    fn CG_ParseServerinfo();
    fn CG_ParseWarmup();
    fn CG_SetLightstyle(num: c_int);
    fn CG_NewClientInfo(clientNum: c_int, forceinfo: qboolean);
    fn CG_BuildSpectatorString();
    fn CG_HandleAppendedSkin(modelName: *mut c_char) -> c_int;
    fn CG_CacheG2AnimInfo(modelName: *mut c_char);
    fn CG_InitLocalEntities();
    fn CG_InitMarkPolys();
    fn CG_ClearParticles();
    fn CG_KillCEntityInstances();
    fn CG_RemoveChatEscapeChar(text: *mut c_char);
    fn CG_GetStringEdString(table: *const c_char, key: *const c_char) -> *const c_char;
    fn CG_CenterPrint(text: *const c_char, y: c_int, charWidth: c_int);
    fn CG_ChatBox_AddString(chatStr: *mut c_char);
    fn CG_LoadDeferredPlayers();
    fn CG_DestroyNPCClient(npcClient: *mut *const c_void);
    fn CG_ReattachLimb(cent: *mut centity_t);
    fn CG_S_StopLoopingSound(entnum: c_int, channel: c_int);
    fn CG_G2WeaponInstance(cent: *mut centity_t, weapon: c_int) -> *const c_void;
    fn CG_SiegeBriefingDisplay(team: c_int, dontshow: c_int);
    fn CG_ParseSiegeExtendedData();
    fn CG_ParseSiegeObjectiveStatus(str: *const c_char);
    fn CG_ParseWeatherEffect(str: *const c_char);
    fn CG_ParseSiegeState(str: *const c_char);
    fn CG_SetSiegeTimerCvar(time: c_int);
    fn CG_ShaderStateChanged();
    fn BG_InDeathAnim(anim: c_int) -> qboolean;

    fn trap_Cvar_GetHiddenVarValue(name: *const c_char) -> c_int;
    fn trap_Cvar_Set(name: *const c_char, value: *const c_char);
    fn trap_Cvar_GetValue(name: *const c_char) -> c_int;
    fn trap_Argc() -> c_int;
    fn trap_GetGameState(gameState: *mut c_void);
    fn trap_R_ClearDecals();
    fn trap_R_RegisterModel(name: *const c_char) -> c_int;
    fn trap_S_RegisterSound(name: *const c_char) -> c_int;
    fn trap_S_StartLocalSound(sfx: c_int, channel: c_int);
    fn trap_S_ShutUp(mute: qboolean);
    fn trap_S_ClearLoopingSounds();
    fn trap_FX_RegisterEffect(name: *const c_char) -> c_int;
    fn trap_R_RemapShader(oldShader: *const c_char, newShader: *const c_char, timeOffset: *const c_char);
    fn trap_G2_HaveWeGhoul2Models(ghoul2: *const c_void) -> qboolean;
    fn trap_G2API_CleanGhoul2Models(ghoul2: *mut *const c_void);
    fn trap_G2API_SetRagDoll(ghoul2: *const c_void, ragdoll: *const c_void);
    fn trap_G2API_DuplicateGhoul2Instance(source: *const c_void, dest: *mut *const c_void);
    fn trap_G2API_RemoveGhoul2Model(ghoul2: *mut *const c_void, index: c_int);
    fn trap_G2API_HasGhoul2ModelOnIndex(ghoul2: *mut *const c_void, index: c_int) -> qboolean;
    fn trap_G2API_CopySpecificGhoul2Model(source: *const c_void, sourceIndex: c_int, dest: *const c_void, destIndex: c_int);
    fn trap_G2API_SetBoneAnim(ghoul2: *const c_void, modelIndex: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int, flags: c_int, animSpeed: f32, currentTime: c_int, ramLevel: c_int, blendTime: c_int);
    fn trap_G2API_ClearSkinGore(ghoul2: *const c_void);
    fn trap_Key_GetCatcher() -> c_int;
    fn trap_OpenUIMenu(menu: c_int);
    fn trap_SP_Print(clientNum: c_int, text: *const c_char);
    fn trap_SP_GetStringTextString(reference: *const c_char, buf: *mut c_char, bufsize: c_int);
    fn trap_SendConsoleCommand(text: *const c_char);
    fn trap_GetServerCommand(sequence: c_int) -> qboolean;

    fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *const c_char;
    fn Q_strncpyz(dst: *mut c_char, src: *const c_char, len: usize);
    fn Q_strcat(dst: *mut c_char, size: c_int, src: *const c_char) -> *mut c_char;
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    fn Com_Printf(fmt: *const c_char, ...);
    fn va(fmt: *const c_char, ...) -> *const c_char;
}

const GT_CTF: c_int = 4;
const GT_CTY: c_int = 5;
const GT_SIEGE: c_int = 8;
const GT_POWERDUEL: c_int = 9;
const GT_DUEL: c_int = 3;

const CS_SERVERINFO: c_int = 0;
const CS_WARMUP: c_int = 3;
const CS_SCORES1: c_int = 6;
const CS_SCORES2: c_int = 7;
const CS_LEVEL_START_TIME: c_int = 8;
const CS_FLAGSTATUS: c_int = 10;
const CS_CLIENT_JEDIMASTER: c_int = 20;
const CS_CLIENT_DUELWINNER: c_int = 21;
const CS_CLIENT_DUELISTS: c_int = 22;
const CS_CLIENT_DUELHEALTHS: c_int = 23;
const CS_VOTE_TIME: c_int = 26;
const CS_VOTE_YES: c_int = 27;
const CS_VOTE_NO: c_int = 28;
const CS_VOTE_STRING: c_int = 29;
const CS_TEAMVOTE_TIME: c_int = 30;
const CS_TEAMVOTE_YES: c_int = 32;
const CS_TEAMVOTE_NO: c_int = 34;
const CS_TEAMVOTE_STRING: c_int = 36;
const CS_INTERMISSION: c_int = 39;
const CS_MUSIC: c_int = 2;
const CS_MODELS: c_int = 40;
const CS_SOUNDS: c_int = 100;
const CS_EFFECTS: c_int = 200;
const CS_TERRAINS: c_int = 300;
const CS_SHADERSTATE: c_int = 350;
const CS_LIGHT_STYLES: c_int = 400;
const CS_SIEGE_STATE: c_int = 500;
const CS_SIEGE_WINTEAM: c_int = 501;
const CS_SIEGE_OBJECTIVES: c_int = 502;
const CS_SIEGE_TIMEOVERRIDE: c_int = 503;
const CS_PLAYERS: c_int = 600;
const CS_CHARSKINS: c_int = 700;
const CS_G2BONES: c_int = 800;

const KEYCATCH_UI: c_int = 0x00000010;

const UIMENU_CLASSSEL: c_int = 2;
const UIMENU_PLAYERCONFIG: c_int = 3;

const CHAN_LOCAL_SOUND: c_int = 0;
const CHAN_ANNOUNCER: c_int = 4;

const SCREEN_HEIGHT: c_int = 480;
const BIGCHAR_WIDTH: c_int = 16;
const GIANTCHAR_WIDTH: c_int = 32;

const BONE_ANIM_OVERRIDE_FREEZE: c_int = 0x20000000;

const WP_BRYAR_PISTOL: c_int = 1;

const MAX_STRINGED_SV_STRING: usize = 1024;

// ============================================================================
// CG_ParseScores
// =================
// =================
unsafe fn CG_ParseScores() {
    let mut i: c_int = 0;
    let mut powerups: c_int = 0;
    let mut readScores: c_int = 0;

    (*std::ptr::addr_of_mut!(cg)).numScores = atoi(CG_Argv(1));

    readScores = (*std::ptr::addr_of_mut!(cg)).numScores;

    if readScores > MAX_CLIENT_SCORE_SEND as c_int
    {
        readScores = MAX_CLIENT_SCORE_SEND as c_int;
    }

    if (*std::ptr::addr_of_mut!(cg)).numScores > MAX_CLIENTS as c_int {
        (*std::ptr::addr_of_mut!(cg)).numScores = MAX_CLIENTS as c_int;
    }

    (*std::ptr::addr_of_mut!(cg)).numScores = readScores;

    (*std::ptr::addr_of_mut!(cgs)).teamScores[0] = atoi(CG_Argv(2));
    (*std::ptr::addr_of_mut!(cgs)).teamScores[1] = atoi(CG_Argv(3));

    memset(std::ptr::addr_of_mut!((*std::ptr::addr_of_mut!(cgs)).scores) as *mut c_void, 0, std::mem::size_of_val(&(*std::ptr::addr_of_mut!(cgs)).scores));
    i = 0;
    while i < readScores {
        //
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].client = atoi(CG_Argv(i * 14 + 4));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].score = atoi(CG_Argv(i * 14 + 5));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].ping = atoi(CG_Argv(i * 14 + 6));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].time = atoi(CG_Argv(i * 14 + 7));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].scoreFlags = atoi(CG_Argv(i * 14 + 8));
        powerups = atoi(CG_Argv(i * 14 + 9));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].accuracy = atoi(CG_Argv(i * 14 + 10));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].impressiveCount = atoi(CG_Argv(i * 14 + 11));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].excellentCount = atoi(CG_Argv(i * 14 + 12));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].guantletCount = atoi(CG_Argv(i * 14 + 13));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].defendCount = atoi(CG_Argv(i * 14 + 14));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].assistCount = atoi(CG_Argv(i * 14 + 15));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].perfect = atoi(CG_Argv(i * 14 + 16));
        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].captures = atoi(CG_Argv(i * 14 + 17));

        if (*std::ptr::addr_of_mut!(cg)).scores[i as usize].client < 0 || (*std::ptr::addr_of_mut!(cg)).scores[i as usize].client >= MAX_CLIENTS as c_int {
            (*std::ptr::addr_of_mut!(cg)).scores[i as usize].client = 0;
        }
        (*std::ptr::addr_of_mut!(cgs)).clientinfo[(*std::ptr::addr_of_mut!(cg)).scores[i as usize].client as usize].score = (*std::ptr::addr_of_mut!(cg)).scores[i as usize].score;
        (*std::ptr::addr_of_mut!(cgs)).clientinfo[(*std::ptr::addr_of_mut!(cg)).scores[i as usize].client as usize].powerups = powerups;

        (*std::ptr::addr_of_mut!(cg)).scores[i as usize].team = (*std::ptr::addr_of_mut!(cgs)).clientinfo[(*std::ptr::addr_of_mut!(cg)).scores[i as usize].client as usize].team;
        i += 1;
    }
    CG_SetScoreSelection(std::ptr::null());
}

/*
=================
CG_ParseTeamInfo

=================
*/
unsafe fn CG_ParseTeamInfo() {
    let mut i: c_int = 0;
    let mut client: c_int = 0;

    numSortedTeamPlayers = atoi(CG_Argv(1));

    i = 0;
    while i < numSortedTeamPlayers {
        client = atoi(CG_Argv(i * 6 + 2));

        sortedTeamPlayers[i as usize] = client;

        (*std::ptr::addr_of_mut!(cgs)).clientinfo[client as usize].location = atoi(CG_Argv(i * 6 + 3));
        (*std::ptr::addr_of_mut!(cgs)).clientinfo[client as usize].health = atoi(CG_Argv(i * 6 + 4));
        (*std::ptr::addr_of_mut!(cgs)).clientinfo[client as usize].armor = atoi(CG_Argv(i * 6 + 5));
        (*std::ptr::addr_of_mut!(cgs)).clientinfo[client as usize].curWeapon = atoi(CG_Argv(i * 6 + 6));
        (*std::ptr::addr_of_mut!(cgs)).clientinfo[client as usize].powerups = atoi(CG_Argv(i * 6 + 7));
        i += 1;
    }
}


/*
================
CG_ParseServerinfo

This is called explicitly when the gamestate is first received,
and whenever the server updates any serverinfo flagged cvars
================
*/
pub unsafe fn CG_ParseServerinfo() {
    let mut info: *const c_char = 0 as *const c_char;
    let mut tinfo: *const c_char = 0 as *const c_char;
    let mut mapname: *mut c_char = 0 as *mut c_char;

    info = CG_ConfigString(CS_SERVERINFO);

    (*std::ptr::addr_of_mut!(cgs)).debugMelee = atoi(Info_ValueForKey(info, b"g_debugMelee\0".as_ptr() as *const c_char)); //trap_Cvar_GetHiddenVarValue("g_iknowkungfu");
    (*std::ptr::addr_of_mut!(cgs)).stepSlideFix = atoi(Info_ValueForKey(info, b"g_stepSlideFix\0".as_ptr() as *const c_char));

    (*std::ptr::addr_of_mut!(cgs)).noSpecMove = atoi(Info_ValueForKey(info, b"g_noSpecMove\0".as_ptr() as *const c_char));

    trap_Cvar_Set(b"bg_fighterAltControl\0".as_ptr() as *const c_char, Info_ValueForKey(info, b"bg_fighterAltControl\0".as_ptr() as *const c_char));

    (*std::ptr::addr_of_mut!(cgs)).siegeTeamSwitch = atoi(Info_ValueForKey(info, b"g_siegeTeamSwitch\0".as_ptr() as *const c_char));

    (*std::ptr::addr_of_mut!(cgs)).showDuelHealths = atoi(Info_ValueForKey(info, b"g_showDuelHealths\0".as_ptr() as *const c_char));

    (*std::ptr::addr_of_mut!(cgs)).gametype = atoi(Info_ValueForKey(info, b"g_gametype\0".as_ptr() as *const c_char));
    trap_Cvar_Set(b"g_gametype\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).gametype));
    (*std::ptr::addr_of_mut!(cgs)).needpass = atoi(Info_ValueForKey(info, b"needpass\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).jediVmerc = atoi(Info_ValueForKey(info, b"g_jediVmerc\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).wDisable = atoi(Info_ValueForKey(info, b"wdisable\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).fDisable = atoi(Info_ValueForKey(info, b"fdisable\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).dmflags = atoi(Info_ValueForKey(info, b"dmflags\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).teamflags = atoi(Info_ValueForKey(info, b"teamflags\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).fraglimit = atoi(Info_ValueForKey(info, b"fraglimit\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).duel_fraglimit = atoi(Info_ValueForKey(info, b"duel_fraglimit\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).capturelimit = atoi(Info_ValueForKey(info, b"capturelimit\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).timelimit = atoi(Info_ValueForKey(info, b"timelimit\0".as_ptr() as *const c_char));
    (*std::ptr::addr_of_mut!(cgs)).maxclients = atoi(Info_ValueForKey(info, b"sv_maxclients\0".as_ptr() as *const c_char));
    mapname = Info_ValueForKey(info, b"mapname\0".as_ptr() as *const c_char) as *mut c_char;

    //rww - You must do this one here, Info_ValueForKey always uses the same memory pointer.
    trap_Cvar_Set(b"ui_about_mapname\0".as_ptr() as *const c_char, mapname);

    Com_sprintf(std::ptr::addr_of_mut!((*std::ptr::addr_of_mut!(cgs)).mapname[0]) as *mut c_char, std::mem::size_of_val(&(*std::ptr::addr_of_mut!(cgs)).mapname) as c_int, b"maps/%s.bsp\0".as_ptr() as *const c_char, mapname);
//	Q_strncpyz( cgs.redTeam, Info_ValueForKey( info, "g_redTeam" ), sizeof(cgs.redTeam) );
//	trap_Cvar_Set("g_redTeam", cgs.redTeam);
//	Q_strncpyz( cgs.blueTeam, Info_ValueForKey( info, "g_blueTeam" ), sizeof(cgs.blueTeam) );
//	trap_Cvar_Set("g_blueTeam", cgs.blueTeam);

    trap_Cvar_Set(b"ui_about_gametype\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).gametype));
    trap_Cvar_Set(b"ui_about_fraglimit\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).fraglimit));
    trap_Cvar_Set(b"ui_about_duellimit\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).duel_fraglimit));
    trap_Cvar_Set(b"ui_about_capturelimit\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).capturelimit));
    trap_Cvar_Set(b"ui_about_timelimit\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).timelimit));
    trap_Cvar_Set(b"ui_about_maxclients\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).maxclients));
    trap_Cvar_Set(b"ui_about_dmflags\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, (*std::ptr::addr_of_mut!(cgs)).dmflags));
    trap_Cvar_Set(b"ui_about_hostname\0".as_ptr() as *const c_char, Info_ValueForKey(info, b"sv_hostname\0".as_ptr() as *const c_char));
    trap_Cvar_Set(b"ui_about_needpass\0".as_ptr() as *const c_char, Info_ValueForKey(info, b"g_needpass\0".as_ptr() as *const c_char));
    trap_Cvar_Set(b"ui_about_botminplayers\0".as_ptr() as *const c_char, Info_ValueForKey(info, b"bot_minplayers\0".as_ptr() as *const c_char));

    //Set the siege teams based on what the server has for overrides.
    trap_Cvar_Set(b"cg_siegeTeam1\0".as_ptr() as *const c_char, Info_ValueForKey(info, b"g_siegeTeam1\0".as_ptr() as *const c_char));
    trap_Cvar_Set(b"cg_siegeTeam2\0".as_ptr() as *const c_char, Info_ValueForKey(info, b"g_siegeTeam2\0".as_ptr() as *const c_char));

    tinfo = CG_ConfigString(CS_TERRAINS + 1);
    if tinfo.is_null() || *tinfo == 0
    {
        (*std::ptr::addr_of_mut!(cg)).mInRMG = 0; // qfalse
    }
    else
    {
        let mut weather: c_int = 0;

        (*std::ptr::addr_of_mut!(cg)).mInRMG = 1; // qtrue
        trap_Cvar_Set(b"RMG\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char);

        weather = atoi(Info_ValueForKey(info, b"RMG_weather\0".as_ptr() as *const c_char));

        trap_Cvar_Set(b"RMG_weather\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, weather));

        if weather == 1 || weather == 2
        {
            (*std::ptr::addr_of_mut!(cg)).mRMGWeather = 1; // qtrue
        }
        else
        {
            (*std::ptr::addr_of_mut!(cg)).mRMGWeather = 0; // qfalse
        }
    }
}

/*
==================
CG_ParseWarmup
==================
*/
unsafe fn CG_ParseWarmup() {
    let mut info: *const c_char = 0 as *const c_char;
    let mut warmup: c_int = 0;

    info = CG_ConfigString(CS_WARMUP);

    warmup = atoi(info);
    (*std::ptr::addr_of_mut!(cg)).warmupCount = -1;

    (*std::ptr::addr_of_mut!(cg)).warmup = warmup;
}

/*
================
CG_SetConfigValues

Called on load to set the initial values from configure strings
================
*/
pub unsafe fn CG_SetConfigValues()
{
    let mut s: *const c_char = 0 as *const c_char;
    let mut str: *const c_char = 0 as *const c_char;

    (*std::ptr::addr_of_mut!(cgs)).scores1 = atoi(CG_ConfigString(CS_SCORES1));
    (*std::ptr::addr_of_mut!(cgs)).scores2 = atoi(CG_ConfigString(CS_SCORES2));
    (*std::ptr::addr_of_mut!(cgs)).levelStartTime = atoi(CG_ConfigString(CS_LEVEL_START_TIME));
    if (*std::ptr::addr_of_mut!(cgs)).gametype == GT_CTF || (*std::ptr::addr_of_mut!(cgs)).gametype == GT_CTY {
        s = CG_ConfigString(CS_FLAGSTATUS);
        (*std::ptr::addr_of_mut!(cgs)).redflag = *s.offset(0) - b'0' as c_char as c_int;
        (*std::ptr::addr_of_mut!(cgs)).blueflag = *s.offset(1) - b'0' as c_char as c_int;
    }
    (*std::ptr::addr_of_mut!(cg)).warmup = atoi(CG_ConfigString(CS_WARMUP));

    // Track who the jedi master is
    (*std::ptr::addr_of_mut!(cgs)).jediMaster = atoi(CG_ConfigString(CS_CLIENT_JEDIMASTER));
    (*std::ptr::addr_of_mut!(cgs)).duelWinner = atoi(CG_ConfigString(CS_CLIENT_DUELWINNER));

    str = CG_ConfigString(CS_CLIENT_DUELISTS);

    if !str.is_null() && *str != 0
    {
        let mut buf: [c_char; 64] = [0; 64];
        let mut c: c_int = 0;
        let mut i: c_int = 0;

        while *str.offset(i as isize) != 0 && *str.offset(i as isize) != b'|' as c_char
        {
            buf[c as usize] = *str.offset(i as isize);
            c += 1;
            i += 1;
        }
        buf[c as usize] = 0;

        (*std::ptr::addr_of_mut!(cgs)).duelist1 = atoi(buf.as_ptr());
        c = 0;

        i += 1;
        while *str.offset(i as isize) != 0
        {
            buf[c as usize] = *str.offset(i as isize);
            c += 1;
            i += 1;
        }
        buf[c as usize] = 0;

        (*std::ptr::addr_of_mut!(cgs)).duelist2 = atoi(buf.as_ptr());
    }
}

/*
=====================
CG_ShaderStateChanged
=====================
*/
pub unsafe fn CG_ShaderStateChanged() {
    let mut originalShader: [c_char; 256] = [0; 256];
    let mut newShader: [c_char; 256] = [0; 256];
    let mut timeOffset: [c_char; 16] = [0; 16];
    let mut o: *const c_char = 0 as *const c_char;
    let mut n: *const c_char = 0 as *const c_char;
    let mut t: *const c_char = 0 as *const c_char;

    o = CG_ConfigString(CS_SHADERSTATE);
    while !o.is_null() && *o != 0 {
        n = strstr(o, b"=\0".as_ptr() as *const c_char);
        if !n.is_null() && *n != 0 {
            strncpy(originalShader.as_mut_ptr(), o, (n as usize) - (o as usize));
            originalShader[(n as usize) - (o as usize)] = 0;
            n = n.offset(1);
            t = strstr(n, b":\0".as_ptr() as *const c_char);
            if !t.is_null() && *t != 0 {
                strncpy(newShader.as_mut_ptr(), n, (t as usize) - (n as usize));
                newShader[(t as usize) - (n as usize)] = 0;
            } else {
                break;
            }
            t = t.offset(1);
            o = strstr(t, b"@\0".as_ptr() as *const c_char);
            if !o.is_null() {
                strncpy(timeOffset.as_mut_ptr(), t, (o as usize) - (t as usize));
                timeOffset[(o as usize) - (t as usize)] = 0;
                o = o.offset(1);
                trap_R_RemapShader(originalShader.as_ptr(), newShader.as_ptr(), timeOffset.as_ptr());
            }
        } else {
            break;
        }
    }
}

unsafe fn GetCustomSoundForType(setType: c_int, index: c_int) -> *const c_char
{
    match setType
    {
    1 =>
        cg_customSoundNames[index as usize],
    2 =>
        cg_customCombatSoundNames[index as usize],
    3 =>
        cg_customExtraSoundNames[index as usize],
    4 =>
        cg_customJediSoundNames[index as usize],
    5 =>
        bg_customSiegeSoundNames[index as usize],
    6 =>
        cg_customDuelSoundNames[index as usize],
    _ => {
        debug_assert!(false, "Invalid setType");
        std::ptr::null()
    }
    }
}

fn SetCustomSoundForType(ci: *mut clientInfo_t, setType: c_int, index: c_int, sfx: sfxHandle_t)
{
    match setType
    {
    1 => {
        unsafe { (*ci).sounds[index as usize] = sfx; }
    }
    2 => {
        unsafe { (*ci).combatSounds[index as usize] = sfx; }
    }
    3 => {
        unsafe { (*ci).extraSounds[index as usize] = sfx; }
    }
    4 => {
        unsafe { (*ci).jediSounds[index as usize] = sfx; }
    }
    5 => {
        unsafe { (*ci).siegeSounds[index as usize] = sfx; }
    }
    6 => {
        unsafe { (*ci).duelSounds[index as usize] = sfx; }
    }
    _ => {
        debug_assert!(false, "Invalid setType");
    }
    }
}

unsafe fn CG_RegisterCustomSounds(ci: *mut clientInfo_t, setType: c_int, psDir: *const c_char)
{
    let mut iTableEntries: c_int = 0;
    let mut i: c_int = 0;

    match setType
    {
    1 =>
        iTableEntries = MAX_CUSTOM_SOUNDS as c_int,
    2 =>
        iTableEntries = MAX_CUSTOM_COMBAT_SOUNDS as c_int,
    3 =>
        iTableEntries = MAX_CUSTOM_EXTRA_SOUNDS as c_int,
    4 =>
        iTableEntries = MAX_CUSTOM_JEDI_SOUNDS as c_int,
    5 =>
        iTableEntries = MAX_CUSTOM_SIEGE_SOUNDS as c_int,
    _ => {
        debug_assert!(false, "Invalid setType");
        return;
    }
    }

    i = 0;
    while i < iTableEntries {
        let mut hSFX: sfxHandle_t = 0;
        let mut s: *const c_char = GetCustomSoundForType(setType, i);

        if s.is_null() {
            break;
        }

        s = s.offset(1);
        hSFX = trap_S_RegisterSound(va(b"sound/chars/%s/misc/%s\0".as_ptr() as *const c_char, psDir, s));

        if hSFX == 0 {
            let mut modifiedSound: [c_char; 256] = [0; 256];
            let mut p: *const c_char = 0 as *const c_char;

            strcpy(modifiedSound.as_mut_ptr(), s);
            p = strchr(modifiedSound.as_ptr(), b'.' as c_int);

            if !p.is_null() {
                let mut testNumber: [c_char; 2] = [0; 2];
                p = p.offset(-1);

                //before we destroy it.. we want to see if this is actually a number.
                //If it isn't a number then don't try decrementing and registering as
                //it will only cause a disk hit (we don't try precaching such files)
                testNumber[0] = *p;
                testNumber[1] = 0;
                if atoi(testNumber.as_ptr()) != 0 {
                    *(p as *mut c_char) = 0;

                    strcat(modifiedSound.as_mut_ptr(), b"1.wav\0".as_ptr() as *const c_char);

                    hSFX = trap_S_RegisterSound(va(b"sound/chars/%s/misc/%s\0".as_ptr() as *const c_char, psDir, modifiedSound.as_ptr()));
                }
            }
        }

        SetCustomSoundForType(ci, setType, i, hSFX);
        i += 1;
    }
}

pub unsafe fn CG_PrecacheNPCSounds(str: *const c_char)
{
    let mut sEnd: [c_char; 256] = [0; 256];
    let mut pEnd: [c_char; 256] = [0; 256];
    let mut i: c_int = 0;
    let mut j: c_int = 0;
    let mut k: c_int = 0;

    k = 2;

    while *str.offset(k as isize) != 0 {
        pEnd[(k-2) as usize] = *str.offset(k as isize);
        k += 1;
    }
    pEnd[(k-2) as usize] = 0;

    i = 0;
    while i < 4 { //4 types
      //It would be better if we knew what type this actually was (extra, combat, jedi, etc).
      //But that would require extra configstring indexing and that is a bad thing.

        j = 0;
        while j < MAX_CUSTOM_SOUNDS as c_int {
            let mut s: *const c_char = GetCustomSoundForType(i+1, j);

            if !s.is_null() && *s != 0 {
              //whatever it is, try registering it under this folder.
                k = 1;
                while *s.offset(k as isize) != 0 {
                    sEnd[(k-1) as usize] = *s.offset(k as isize);
                    k += 1;
                }
                sEnd[(k-1) as usize] = 0;

                trap_S_ShutUp(1); // qtrue
                trap_S_RegisterSound(va(b"sound/chars/%s/misc/%s\0".as_ptr() as *const c_char, pEnd.as_ptr(), sEnd.as_ptr()));
                trap_S_ShutUp(0); // qfalse
            }
            else {
              //move onto the next set
                break;
            }

            j += 1;
        }

        j = 0;
        i += 1;
    }
}

pub unsafe fn CG_HandleNPCSounds(cent: *mut centity_t)
{
    if (*cent).npcClient.is_null() {
        return;
    }

    //standard
    if (*cent).currentState.csSounds_Std != 0 {
        let mut s: *const c_char = CG_ConfigString(CS_SOUNDS + (*cent).currentState.csSounds_Std);

        if !s.is_null() && *s != 0 {
            let mut sEnd: [c_char; 256] = [0; 256];
            let mut i: c_int = 2;
            let mut j: c_int = 0;

            //Parse past the initial "*" which indicates this is a custom sound, and the $ which indicates
            //it is an NPC custom sound dir.
            while *s.offset(i as isize) != 0 {
                sEnd[j as usize] = *s.offset(i as isize);
                j += 1;
                i += 1;
            }
            sEnd[j as usize] = 0;

            CG_RegisterCustomSounds((*cent).npcClient as *mut clientInfo_t, 1, sEnd.as_ptr());
        }
    }
    else {
        memset(&mut (*(*cent).npcClient).sounds as *mut c_void, 0, std::mem::size_of_val(&(*(*cent).npcClient).sounds));
    }

    //combat
    if (*cent).currentState.csSounds_Combat != 0 {
        let mut s: *const c_char = CG_ConfigString(CS_SOUNDS + (*cent).currentState.csSounds_Combat);

        if !s.is_null() && *s != 0 {
            let mut sEnd: [c_char; 256] = [0; 256];
            let mut i: c_int = 2;
            let mut j: c_int = 0;

            //Parse past the initial "*" which indicates this is a custom sound, and the $ which indicates
            //it is an NPC custom sound dir.
            while *s.offset(i as isize) != 0 {
                sEnd[j as usize] = *s.offset(i as isize);
                j += 1;
                i += 1;
            }
            sEnd[j as usize] = 0;

            CG_RegisterCustomSounds((*cent).npcClient as *mut clientInfo_t, 2, sEnd.as_ptr());
        }
    }
    else {
        memset(&mut (*(*cent).npcClient).combatSounds as *mut c_void, 0, std::mem::size_of_val(&(*(*cent).npcClient).combatSounds));
    }

    //extra
    if (*cent).currentState.csSounds_Extra != 0 {
        let mut s: *const c_char = CG_ConfigString(CS_SOUNDS + (*cent).currentState.csSounds_Extra);

        if !s.is_null() && *s != 0 {
            let mut sEnd: [c_char; 256] = [0; 256];
            let mut i: c_int = 2;
            let mut j: c_int = 0;

            //Parse past the initial "*" which indicates this is a custom sound, and the $ which indicates
            //it is an NPC custom sound dir.
            while *s.offset(i as isize) != 0 {
                sEnd[j as usize] = *s.offset(i as isize);
                j += 1;
                i += 1;
            }
            sEnd[j as usize] = 0;

            CG_RegisterCustomSounds((*cent).npcClient as *mut clientInfo_t, 3, sEnd.as_ptr());
        }
    }
    else {
        memset(&mut (*(*cent).npcClient).extraSounds as *mut c_void, 0, std::mem::size_of_val(&(*(*cent).npcClient).extraSounds));
    }

    //jedi
    if (*cent).currentState.csSounds_Jedi != 0 {
        let mut s: *const c_char = CG_ConfigString(CS_SOUNDS + (*cent).currentState.csSounds_Jedi);

        if !s.is_null() && *s != 0 {
            let mut sEnd: [c_char; 256] = [0; 256];
            let mut i: c_int = 2;
            let mut j: c_int = 0;

            //Parse past the initial "*" which indicates this is a custom sound, and the $ which indicates
            //it is an NPC custom sound dir.
            while *s.offset(i as isize) != 0 {
                sEnd[j as usize] = *s.offset(i as isize);
                j += 1;
                i += 1;
            }
            sEnd[j as usize] = 0;

            CG_RegisterCustomSounds((*cent).npcClient as *mut clientInfo_t, 4, sEnd.as_ptr());
        }
    }
    else {
        memset(&mut (*(*cent).npcClient).jediSounds as *mut c_void, 0, std::mem::size_of_val(&(*(*cent).npcClient).jediSounds));
    }
}

extern "C" {
    fn CG_HandleAppendedSkin(modelName: *mut c_char) -> c_int;
    fn CG_CacheG2AnimInfo(modelName: *mut c_char);
}

// nmckenzie: DUEL_HEALTH - fixme - we could really clean this up immensely with some helper functions.
unsafe fn SetDuelistHealthsFromConfigString(str: *const c_char) {
    let mut buf: [c_char; 64] = [0; 64];
    let mut c: c_int = 0;
    let mut i: c_int = 0;

    while *str.offset(i as isize) != 0 && *str.offset(i as isize) != b'|' as c_char {
        buf[c as usize] = *str.offset(i as isize);
        c += 1;
        i += 1;
    }
    buf[c as usize] = 0;

    (*std::ptr::addr_of_mut!(cgs)).duelist1health = atoi(buf.as_ptr());

    c = 0;
    i += 1;
    while *str.offset(i as isize) != 0 && *str.offset(i as isize) != b'|' as c_char {
        buf[c as usize] = *str.offset(i as isize);
        c += 1;
        i += 1;
    }
    buf[c as usize] = 0;

    (*std::ptr::addr_of_mut!(cgs)).duelist2health = atoi(buf.as_ptr());

    c = 0;
    i += 1;
    if *str.offset(i as isize) == b'!' as c_char {
        // we only have 2 duelists, apparently.
        (*std::ptr::addr_of_mut!(cgs)).duelist3health = -1;
        return;
    }

    while *str.offset(i as isize) != 0 && *str.offset(i as isize) != b'|' as c_char {
        buf[c as usize] = *str.offset(i as isize);
        c += 1;
        i += 1;
    }
    buf[c as usize] = 0;

    (*std::ptr::addr_of_mut!(cgs)).duelist3health = atoi(buf.as_ptr());
}

/*
================
CG_ConfigStringModified

================
*/
unsafe fn CG_ConfigStringModified() {
    let mut str: *const c_char = 0 as *const c_char;
    let mut num: c_int = 0;

    num = atoi(CG_Argv(1));

    // get the gamestate from the client system, which will have the
    // new configstring already integrated
    trap_GetGameState(std::ptr::addr_of_mut!((*std::ptr::addr_of_mut!(cgs)).gameState) as *mut c_void);

    // look up the individual string that was modified
    str = CG_ConfigString(num);

    // do something with it if necessary
    if num == CS_MUSIC {
        CG_StartMusic(1); // qtrue
    } else if num == CS_SERVERINFO {
        CG_ParseServerinfo();
    } else if num == CS_WARMUP {
        CG_ParseWarmup();
    } else if num == CS_SCORES1 {
        (*std::ptr::addr_of_mut!(cgs)).scores1 = atoi(str);
    } else if num == CS_SCORES2 {
        (*std::ptr::addr_of_mut!(cgs)).scores2 = atoi(str);
    } else if num == CS_CLIENT_JEDIMASTER {
        (*std::ptr::addr_of_mut!(cgs)).jediMaster = atoi(str);
    }
    else if num == CS_CLIENT_DUELWINNER {
        (*std::ptr::addr_of_mut!(cgs)).duelWinner = atoi(str);
    }
    else if num == CS_CLIENT_DUELISTS {
        let mut buf: [c_char; 64] = [0; 64];
        let mut c: c_int = 0;
        let mut i: c_int = 0;

        while *str.offset(i as isize) != 0 && *str.offset(i as isize) != b'|' as c_char {
            buf[c as usize] = *str.offset(i as isize);
            c += 1;
            i += 1;
        }
        buf[c as usize] = 0;

        (*std::ptr::addr_of_mut!(cgs)).duelist1 = atoi(buf.as_ptr());
        c = 0;

        i += 1;
        while *str.offset(i as isize) != 0 && *str.offset(i as isize) != b'|' as c_char {
            buf[c as usize] = *str.offset(i as isize);
            c += 1;
            i += 1;
        }
        buf[c as usize] = 0;

        (*std::ptr::addr_of_mut!(cgs)).duelist2 = atoi(buf.as_ptr());

        if *str.offset(i as isize) != 0 {
            c = 0;
            i += 1;

            while *str.offset(i as isize) != 0 {
                buf[c as usize] = *str.offset(i as isize);
                c += 1;
                i += 1;
            }
            buf[c as usize] = 0;

            (*std::ptr::addr_of_mut!(cgs)).duelist3 = atoi(buf.as_ptr());
        }
    }
    else if num == CS_CLIENT_DUELHEALTHS {
        // nmckenzie: DUEL_HEALTH
        SetDuelistHealthsFromConfigString(str);
    }
    else if num == CS_LEVEL_START_TIME {
        (*std::ptr::addr_of_mut!(cgs)).levelStartTime = atoi(str);
    } else if num == CS_VOTE_TIME {
        (*std::ptr::addr_of_mut!(cgs)).voteTime = atoi(str);
        (*std::ptr::addr_of_mut!(cgs)).voteModified = 1; // qtrue
    } else if num == CS_VOTE_YES {
        (*std::ptr::addr_of_mut!(cgs)).voteYes = atoi(str);
        (*std::ptr::addr_of_mut!(cgs)).voteModified = 1; // qtrue
    } else if num == CS_VOTE_NO {
        (*std::ptr::addr_of_mut!(cgs)).voteNo = atoi(str);
        (*std::ptr::addr_of_mut!(cgs)).voteModified = 1; // qtrue
    } else if num == CS_VOTE_STRING {
        Q_strncpyz(std::ptr::addr_of_mut!((*std::ptr::addr_of_mut!(cgs)).voteString[0]) as *mut c_char, str, std::mem::size_of_val(&(*std::ptr::addr_of_mut!(cgs)).voteString));
    } else if num >= CS_TEAMVOTE_TIME && num <= CS_TEAMVOTE_TIME + 1 {
        (*std::ptr::addr_of_mut!(cgs)).teamVoteTime[(num-CS_TEAMVOTE_TIME) as usize] = atoi(str);
        (*std::ptr::addr_of_mut!(cgs)).teamVoteModified[(num-CS_TEAMVOTE_TIME) as usize] = 1; // qtrue
    } else if num >= CS_TEAMVOTE_YES && num <= CS_TEAMVOTE_YES + 1 {
        (*std::ptr::addr_of_mut!(cgs)).teamVoteYes[(num-CS_TEAMVOTE_YES) as usize] = atoi(str);
        (*std::ptr::addr_of_mut!(cgs)).teamVoteModified[(num-CS_TEAMVOTE_YES) as usize] = 1; // qtrue
    } else if num >= CS_TEAMVOTE_NO && num <= CS_TEAMVOTE_NO + 1 {
        (*std::ptr::addr_of_mut!(cgs)).teamVoteNo[(num-CS_TEAMVOTE_NO) as usize] = atoi(str);
        (*std::ptr::addr_of_mut!(cgs)).teamVoteModified[(num-CS_TEAMVOTE_NO) as usize] = 1; // qtrue
    } else if num >= CS_TEAMVOTE_STRING && num <= CS_TEAMVOTE_STRING + 1 {
        Q_strncpyz(std::ptr::addr_of_mut!((*std::ptr::addr_of_mut!(cgs)).teamVoteString[(num-CS_TEAMVOTE_STRING) as usize][0]) as *mut c_char, str, std::mem::size_of_val(&(*std::ptr::addr_of_mut!(cgs)).teamVoteString));
    } else if num == CS_INTERMISSION {
        (*std::ptr::addr_of_mut!(cg)).intermissionStarted = atoi(str);
    } else if num >= CS_MODELS && num < CS_MODELS+MAX_MODELS as c_int {
        let mut modelName: [c_char; 256] = [0; 256];
        strcpy(modelName.as_mut_ptr(), str);
        if !strstr(modelName.as_ptr(), b".glm\0".as_ptr() as *const c_char).is_null() || modelName[0] as u8 == b'$' {
            //Check to see if it has a custom skin attached.
            CG_HandleAppendedSkin(modelName.as_mut_ptr());
            CG_CacheG2AnimInfo(modelName.as_mut_ptr());
        }

        if modelName[0] as u8 != b'$' && modelName[0] as u8 != b'@' {
            //don't register vehicle names and saber names as models.
            (*std::ptr::addr_of_mut!(cgs)).gameModels[(num-CS_MODELS) as usize] = trap_R_RegisterModel(modelName.as_ptr());
        }
        else {
            (*std::ptr::addr_of_mut!(cgs)).gameModels[(num-CS_MODELS) as usize] = 0;
        }
// GHOUL2 Insert start
        /*
    } else if ( num >= CS_CHARSKINS && num < CS_CHARSKINS+MAX_CHARSKINS ) {
        cgs.skins[ num-CS_CHARSKINS ] = trap_R_RegisterSkin( str );
        */
        //rww - removed and replaced with CS_G2BONES
// Ghoul2 Insert end
    } else if num >= CS_SOUNDS && num < CS_SOUNDS+MAX_SOUNDS as c_int {
        if *str as u8 != b'*' {
            // player specific sounds don't register here
            (*std::ptr::addr_of_mut!(cgs)).gameSounds[(num-CS_SOUNDS) as usize] = trap_S_RegisterSound(str);
        }
        else if *str.offset(1) as u8 == b'$' {
            //an NPC soundset
            CG_PrecacheNPCSounds(str);
        }
    } else if num >= CS_EFFECTS && num < CS_EFFECTS+MAX_FX as c_int {
        if *str as u8 == b'*' {
            //it's a special global weather effect
            CG_ParseWeatherEffect(str);
            (*std::ptr::addr_of_mut!(cgs)).gameEffects[(num-CS_EFFECTS) as usize] = 0;
        }
        else {
            (*std::ptr::addr_of_mut!(cgs)).gameEffects[(num-CS_EFFECTS) as usize] = trap_FX_RegisterEffect(str);
        }
    }
    else if num >= CS_SIEGE_STATE && num < CS_SIEGE_STATE+1 {
        if *str != 0 {
            CG_ParseSiegeState(str);
        }
    }
    else if num >= CS_SIEGE_WINTEAM && num < CS_SIEGE_WINTEAM+1 {
        if *str != 0 {
            cg_siegeWinTeam = atoi(str);
        }
    }
    else if num >= CS_SIEGE_OBJECTIVES && num < CS_SIEGE_OBJECTIVES+1 {
        CG_ParseSiegeObjectiveStatus(str);
    }
    else if num >= CS_SIEGE_TIMEOVERRIDE && num < CS_SIEGE_TIMEOVERRIDE+1 {
        cg_beatingSiegeTime = atoi(str);
        CG_SetSiegeTimerCvar(cg_beatingSiegeTime);
    }
    else if num >= CS_PLAYERS && num < CS_PLAYERS+MAX_CLIENTS as c_int {
        CG_NewClientInfo(num - CS_PLAYERS, 1); // qtrue
        CG_BuildSpectatorString();
    } else if num == CS_FLAGSTATUS {
        if (*std::ptr::addr_of_mut!(cgs)).gametype == GT_CTF || (*std::ptr::addr_of_mut!(cgs)).gametype == GT_CTY {
            // format is rb where its red/blue, 0 is at base, 1 is taken, 2 is dropped
            (*std::ptr::addr_of_mut!(cgs)).redflag = *str as u8 as c_int - b'0' as c_int;
            (*std::ptr::addr_of_mut!(cgs)).blueflag = *str.offset(1) as u8 as c_int - b'0' as c_int;
        }
    }
    else if num == CS_SHADERSTATE {
        CG_ShaderStateChanged();
    }
    else if num >= CS_LIGHT_STYLES && num < CS_LIGHT_STYLES + (MAX_LIGHT_STYLES as c_int * 3) {
        CG_SetLightstyle(num - CS_LIGHT_STYLES);
    }
}

//frees all ghoul2 stuff and npc stuff from a centity -rww
pub unsafe fn CG_KillCEntityG2(entNum: c_int)
{
    let mut j: c_int = 0;
    let mut ci: *mut clientInfo_t = std::ptr::null_mut();
    let mut cent: *mut centity_t = &mut cg_entities[entNum as usize];

    if (entNum as usize) < MAX_CLIENTS {
        ci = &mut (*std::ptr::addr_of_mut!(cgs)).clientinfo[entNum as usize];
    }
    else {
        ci = (*cent).npcClient as *mut clientInfo_t;
    }

    if !ci.is_null() {
        if ci == (*cent).npcClient as *mut clientInfo_t {
            //never going to be != cent->ghoul2, unless cent->ghoul2 has already been removed (and then this ptr is not valid)
            (*ci).ghoul2Model = std::ptr::null();
        }
        else if (*ci).ghoul2Model == (*cent).ghoul2 {
            (*ci).ghoul2Model = std::ptr::null();
        }
        else if !(*ci).ghoul2Model.is_null() && trap_G2_HaveWeGhoul2Models((*ci).ghoul2Model) != 0 {
            trap_G2API_CleanGhoul2Models(&mut (*ci).ghoul2Model);
            (*ci).ghoul2Model = std::ptr::null();
        }

        //Clean up any weapon instances for custom saber stuff
        j = 0;
        while j < 2 { // MAX_SABERS
            if !(*ci).ghoul2Weapons[j as usize].is_null() && trap_G2_HaveWeGhoul2Models((*ci).ghoul2Weapons[j as usize]) != 0 {
                trap_G2API_CleanGhoul2Models(&mut (*ci).ghoul2Weapons[j as usize]);
                (*ci).ghoul2Weapons[j as usize] = std::ptr::null();
            }

            j += 1;
        }
    }

    if !(*cent).ghoul2.is_null() && trap_G2_HaveWeGhoul2Models((*cent).ghoul2) != 0 {
        trap_G2API_CleanGhoul2Models(&mut (*cent).ghoul2);
        (*cent).ghoul2 = std::ptr::null();
    }

    if !(*cent).grip_arm.is_null() && trap_G2_HaveWeGhoul2Models((*cent).grip_arm) != 0 {
        trap_G2API_CleanGhoul2Models(&mut (*cent).grip_arm);
        (*cent).grip_arm = std::ptr::null();
    }

    if !(*cent).frame_hold.is_null() && trap_G2_HaveWeGhoul2Models((*cent).frame_hold) != 0 {
        trap_G2API_CleanGhoul2Models(&mut (*cent).frame_hold);
        (*cent).frame_hold = std::ptr::null();
    }

    if !(*cent).npcClient.is_null() {
        CG_DestroyNPCClient(&mut (*cent).npcClient);
    }

    (*cent).isRagging = 0; // qfalse; //just in case.
    (*cent).ikStatus = 0; // qfalse

    (*cent).localAnimIndex = 0;
}

pub unsafe fn CG_KillCEntityInstances()
{
    let mut i: c_int = 0;
    let mut cent: *mut centity_t = 0 as *mut centity_t;

    i = 0;
    while i < MAX_GENTITIES as c_int {
        cent = &mut cg_entities[i as usize];

        if i >= MAX_CLIENTS as c_int && (*cent).currentState.number == i {
            //do not clear G2 instances on client ents, they are constant
            CG_KillCEntityG2(i);
        }

        (*cent).bolt1 = 0;
        (*cent).bolt2 = 0;
        (*cent).bolt3 = 0;
        (*cent).bolt4 = 0;

        (*cent).bodyHeight = 0;//SABER_LENGTH_MAX;
        //cent->saberExtendTime = 0;

        (*cent).boltInfo = 0;

        (*cent).frame_minus1_refreshed = 0;
        (*cent).frame_minus2_refreshed = 0;
        (*cent).dustTrailTime = 0;
        (*cent).ghoul2weapon = std::ptr::null();
        //cent->torsoBolt = 0;
        (*cent).trailTime = 0;
        (*cent).frame_hold_time = 0;
        (*cent).frame_hold_refreshed = 0;
        (*cent).trickAlpha = 0;
        (*cent).trickAlphaTime = 0;
        // VectorClear((*cent).turAngles);  -- stub for now
        (*cent).weapon = 0;
        (*cent).teamPowerEffectTime = 0;
        (*cent).teamPowerType = 0;
        (*cent).numLoopingSounds = 0;

        (*cent).localAnimIndex = 0;

        i += 1;
    }
}

/*
===============
CG_MapRestart

The server has issued a map_restart, so the next snapshot
is completely new and should not be interpolated to.

A tournement restart will clear everything, but doesn't
require a reload of all the media
===============
*/
unsafe fn CG_MapRestart() {
    if cg_showmiss.integer != 0 {
        CG_Printf(b"CG_MapRestart\n\0".as_ptr() as *const c_char);
    }

    trap_R_ClearDecals();
    //FIXME: trap_FX_Reset?

    CG_InitLocalEntities();
    CG_InitMarkPolys();
    CG_ClearParticles();
    CG_KillCEntityInstances();

    // make sure the "3 frags left" warnings play again
    (*std::ptr::addr_of_mut!(cg)).fraglimitWarnings = 0;

    (*std::ptr::addr_of_mut!(cg)).timelimitWarnings = 0;

    (*std::ptr::addr_of_mut!(cg)).intermissionStarted = 0; // qfalse

    (*std::ptr::addr_of_mut!(cgs)).voteTime = 0;

    (*std::ptr::addr_of_mut!(cg)).mapRestart = 1; // qtrue

    CG_StartMusic(1); // qtrue

    trap_S_ClearLoopingSounds();

    // we really should clear more parts of cg here and stop sounds

    // play the "fight" sound if this is a restart without warmup
    if (*std::ptr::addr_of_mut!(cg)).warmup == 0 && (*std::ptr::addr_of_mut!(cgs)).gametype != GT_SIEGE && (*std::ptr::addr_of_mut!(cgs)).gametype != GT_POWERDUEL {
        trap_S_StartLocalSound((*std::ptr::addr_of_mut!(cgs)).media.countFightSound, CHAN_ANNOUNCER);
        CG_CenterPrint(CG_GetStringEdString(b"MP_SVGAME\0".as_ptr() as *const c_char, b"BEGIN_DUEL\0".as_ptr() as *const c_char), 120, GIANTCHAR_WIDTH*2);
    }
    /*
    if (cg_singlePlayerActive.integer) {
        trap_Cvar_Set("ui_matchStartTime", va("%i", cg.time));
        if (cg_recordSPDemo.integer && cg_recordSPDemoName.string && *cg_recordSPDemoName.string) {
            trap_SendConsoleCommand(va("set g_synchronousclients 1 ; record %s \n", cg_recordSPDemoName.string));
        }
    }
    */
    trap_Cvar_Set(b"cg_thirdPerson\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
}

/*
=================
CG_RemoveChatEscapeChar
=================
*/
unsafe fn CG_RemoveChatEscapeChar_local(text: *mut c_char) {
    let mut i: c_int = 0;
    let mut l: c_int = 0;

    l = 0;
    while *text.offset(i as isize) != 0 {
        if *text.offset(i as isize) as u8 == 0x19 {
            i += 1;
            continue;
        }
        *text.offset(l as isize) = *text.offset(i as isize);
        l += 1;
        i += 1;
    }
    *text.offset(l as isize) = 0;
}

unsafe fn CG_CheckSVStringEdRef(buf: *mut c_char, str: *const c_char)
{
    //I don't really like doing this. But it utilizes the system that was already in place.
    let mut i: c_int = 0;
    let mut b: c_int = 0;
    let mut strLen: c_int = 0;
    let mut gotStrip: qboolean = 0; // qfalse

    if str.is_null() || *str == 0 {
        if !str.is_null() {
            strcpy(buf, str);
        }
        return;
    }

    strcpy(buf, str);

    strLen = strlen(str) as c_int;

    if strLen >= MAX_STRINGED_SV_STRING as c_int {
        return;
    }

    i = 0;
    while i < strLen && *str.offset(i as isize) != 0 {
        gotStrip = 0; // qfalse

        if *str.offset(i as isize) as u8 == b'@' && (i+1) < strLen {
            if *str.offset((i+1) as isize) as u8 == b'@' && (i+2) < strLen {
                if *str.offset((i+2) as isize) as u8 == b'@' && (i+3) < strLen {
                    //@@@ should mean to insert a StringEd reference here, so insert it into buf at the current place
                    let mut stringRef: [c_char; 1024] = [0; 1024];
                    let mut r: c_int = 0;

                    while i < strLen && *str.offset(i as isize) as u8 == b'@' {
                        i += 1;
                    }

                    while i < strLen && *str.offset(i as isize) != 0 && *str.offset(i as isize) as u8 != b' ' && *str.offset(i as isize) as u8 != b':' && *str.offset(i as isize) as u8 != b'.' && *str.offset(i as isize) as u8 != b'\n' {
                        stringRef[r as usize] = *str.offset(i as isize);
                        r += 1;
                        i += 1;
                    }
                    stringRef[r as usize] = 0;

                    *buf.offset(b as isize) = 0;
                    Q_strcat(buf, MAX_STRINGED_SV_STRING as c_int, CG_GetStringEdString(b"MP_SVGAME\0".as_ptr() as *const c_char, stringRef.as_ptr()));
                    b = strlen(buf) as c_int;
                }
            }
        }

        if gotStrip == 0 {
            *buf.offset(b as isize) = *str.offset(i as isize);
            b += 1;
        }
        i += 1;
    }

    *buf.offset(b as isize) = 0;
}

unsafe fn CG_BodyQueueCopy(cent: *mut centity_t, clientNum: c_int, knownWeapon: c_int)
{
    let mut source: *mut centity_t = 0 as *mut centity_t;
    let mut anim: *const animation_t = 0 as *const animation_t;
    let mut animSpeed: f32 = 0.0;
    let mut flags: c_int = BONE_ANIM_OVERRIDE_FREEZE;
    let mut ci: *mut clientInfo_t = 0 as *mut clientInfo_t;

    if !(*cent).ghoul2.is_null() {
        trap_G2API_CleanGhoul2Models(&mut (*cent).ghoul2);
    }

    if clientNum < 0 || clientNum >= MAX_CLIENTS as c_int {
        return;
    }

    source = &mut cg_entities[clientNum as usize];
    ci = &mut (*std::ptr::addr_of_mut!(cgs)).clientinfo[clientNum as usize];

    if source.is_null() {
        return;
    }

    if (*source).ghoul2.is_null() {
        return;
    }

    (*cent).isRagging = 0; // qfalse; //reset in case it's still set from another body that was in this cent slot.
    (*cent).ownerRagging = (*source).isRagging; //if the owner was in ragdoll state, then we want to go into it too right away.

/* #if 0
    VectorCopy(source->lerpOriginOffset, cent->lerpOriginOffset);
#endif */

    (*cent).bodyFadeTime = 0;
    (*cent).bodyHeight = 0;

    (*cent).dustTrailTime = (*source).dustTrailTime;

    trap_G2API_DuplicateGhoul2Instance((*source).ghoul2, &mut (*cent).ghoul2);

    if (*source).isRagging != 0 {
        //just reset it now.
        (*source).isRagging = 0; // qfalse
        trap_G2API_SetRagDoll((*source).ghoul2, std::ptr::null()); //calling with null parms resets to no ragdoll.
    }

    //either force the weapon from when we died or remove it if it was a dropped weapon
    if knownWeapon > WP_BRYAR_PISTOL && trap_G2API_HasGhoul2ModelOnIndex(&mut (*cent).ghoul2, 1) != 0 {
        trap_G2API_RemoveGhoul2Model(&mut (*cent).ghoul2, 1);
    }
    else if trap_G2API_HasGhoul2ModelOnIndex(&mut (*cent).ghoul2, 1) != 0 {
        trap_G2API_CopySpecificGhoul2Model(CG_G2WeaponInstance(cent, knownWeapon), 0, (*cent).ghoul2, 1);
    }

    if (*cent).ownerRagging == 0 {
        let mut aNum: c_int = 0;
        let mut eFrame: c_int = 0;
        let mut fallBack: qboolean = 0; // qfalse

        //anim = &bgAllAnims[cent->localAnimIndex].anims[ cent->currentState.torsoAnim ];
        if BG_InDeathAnim((*source).currentState.torsoAnim) == 0 {
            //then just snap the corpse into a default
            anim = &(*std::ptr::addr_of!(bgAllAnims[(*source).localAnimIndex as usize])).anims[1]; // BOTH_DEAD1 = 1
            fallBack = 1; // qtrue
        }
        else {
            anim = &(*std::ptr::addr_of!(bgAllAnims[(*source).localAnimIndex as usize])).anims[(*source).currentState.torsoAnim as usize];
        }
        animSpeed = 50.0f / (*anim).frameLerp;

        if fallBack == 0 {
            //this will just set us to the last frame of the animation, in theory
            aNum = (*std::ptr::addr_of!(cgs)).clientinfo[(*source).currentState.number as usize].frame + 1;

            while aNum >= (*anim).firstFrame + (*anim).numFrames {
                aNum -= 1;
            }

            if aNum < (*anim).firstFrame - 1 {
                //wrong animation...?
                aNum = ((*anim).firstFrame + (*anim).numFrames) - 1;
            }
        }
        else {
            aNum = (*anim).firstFrame;
        }

        eFrame = (*anim).firstFrame + (*anim).numFrames;

        //if (!cgs.clientinfo[source->currentState.number].frame || (cent->currentState.torsoAnim) != (source->currentState.torsoAnim) )
        //{
        //	aNum = (anim->firstFrame+anim->numFrames)-1;
        //}

        trap_G2API_SetBoneAnim((*cent).ghoul2, 0, b"upper_lumbar\0".as_ptr() as *const c_char, aNum, eFrame, flags, animSpeed, (*std::ptr::addr_of!(cg)).time, -1, 150);
        trap_G2API_SetBoneAnim((*cent).ghoul2, 0, b"model_root\0".as_ptr() as *const c_char, aNum, eFrame, flags, animSpeed, (*std::ptr::addr_of!(cg)).time, -1, 150);
        trap_G2API_SetBoneAnim((*cent).ghoul2, 0, b"Motion\0".as_ptr() as *const c_char, aNum, eFrame, flags, animSpeed, (*std::ptr::addr_of!(cg)).time, -1, 150);
    }

    //After we create the bodyqueue, regenerate any limbs on the real instance
    if (*source).torsoBolt != 0 {
        CG_ReattachLimb(source);
    }
}

extern "C" {
    static bgAllAnims: [c_void; 0];  // Stub
}

/*
=================
CG_ServerCommand

The string has been tokenized and can be retrieved with
Cmd_Argc() / Cmd_Argv()
=================
*/
unsafe fn CG_ServerCommand() {
    let mut cmd: *const c_char = 0 as *const c_char;
    let mut text: [c_char; 150] = [0; 150];
    let mut IRCG: qboolean = 0; // qfalse

    cmd = CG_Argv(0);

    if *cmd == 0 {
        // server claimed the command
        return;
    }

/* #if 0
    // never seems to get used -Ste
    if ( !strcmp( cmd, "spd" ) )
    {
        const char *ID;
        int holdInt,count,i;
        char string[1204];

        count = trap_Argc();

        ID =  CG_Argv(1);
        holdInt = atoi(ID);

        memset( &string, 0, sizeof( string ) );

        Com_sprintf( string,sizeof(string)," \"%s\"", (const char *) CG_Argv(2));

        for (i=3;i<count;i++)
        {
            Com_sprintf( string,sizeof(string)," %s \"%s\"", string, (const char *) CG_Argv(i));
        }

        trap_SP_Print(holdInt, (byte *)string);
        return;
    }
#endif */

    if Q_stricmp(cmd, b"sxd\0".as_ptr() as *const c_char) == 0 {
        //siege extended data, contains extra info certain classes may want to know about other clients
        CG_ParseSiegeExtendedData();
        return;
    }

    if Q_stricmp(cmd, b"sb\0".as_ptr() as *const c_char) == 0 {
        //siege briefing display
        CG_SiegeBriefingDisplay(atoi(CG_Argv(1)), 0);
        return;
    }

    if Q_stricmp(cmd, b"scl\0".as_ptr() as *const c_char) == 0 {
        //if (!( trap_Key_GetCatcher() & KEYCATCH_UI ))
        //Well, I want it to come up even if the briefing display is up.
        {
            trap_OpenUIMenu(UIMENU_CLASSSEL); //UIMENU_CLASSSEL
        }
        return;
    }

    if Q_stricmp(cmd, b"spc\0".as_ptr() as *const c_char) == 0 {
        trap_Cvar_Set(b"ui_myteam\0".as_ptr() as *const c_char, b"3\0".as_ptr() as *const c_char);
        trap_OpenUIMenu(UIMENU_PLAYERCONFIG); //UIMENU_CLASSSEL
        return;
    }

    if Q_stricmp(cmd, b"nfr\0".as_ptr() as *const c_char) == 0 {
        //"nfr" == "new force rank" (want a short string)
        let mut doMenu: c_int = 0;
        let mut setTeam: c_int = 0;
        let mut newRank: c_int = 0;

        if trap_Argc() < 3 {
            Com_Printf(b"WARNING: Invalid newForceRank string\n\0".as_ptr() as *const c_char);
            return;
        }

        newRank = atoi(CG_Argv(1));
        doMenu = atoi(CG_Argv(2));
        setTeam = atoi(CG_Argv(3));

        trap_Cvar_Set(b"ui_rankChange\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, newRank));

        trap_Cvar_Set(b"ui_myteam\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, setTeam));

        if (trap_Key_GetCatcher() & KEYCATCH_UI) == 0 && doMenu != 0 {
            trap_OpenUIMenu(UIMENU_PLAYERCONFIG);
        }

        return;
    }

    if Q_stricmp(cmd, b"kg2\0".as_ptr() as *const c_char) == 0 {
        //Kill a ghoul2 instance in this slot.
          //If it has been occupied since this message was sent somehow, the worst that can (should) happen
          //is the instance will have to reinit with its current info.
        let mut indexNum: c_int = 0;
        let mut argNum: c_int = trap_Argc();
        let mut i: c_int = 1;

        if argNum < 1 {
            return;
        }

        while i < argNum {
            indexNum = atoi(CG_Argv(i));

            if !cg_entities[indexNum as usize].ghoul2.is_null() && trap_G2_HaveWeGhoul2Models(cg_entities[indexNum as usize].ghoul2) != 0 {
                if (indexNum as usize) < MAX_CLIENTS {
                    //You try to do very bad thing!
                    Com_Printf(b"WARNING: Tried to kill a client ghoul2 instance with a kg2 command!\n\0".as_ptr() as *const c_char);
                    return;
                }

                CG_KillCEntityG2(indexNum);
            }

            i += 1;
        }

        return;
    }

    if Q_stricmp(cmd, b"kls\0".as_ptr() as *const c_char) == 0 {
        //kill looping sounds
        let mut indexNum: c_int = 0;
        let mut argNum: c_int = trap_Argc();
        let mut clent: *mut centity_t = std::ptr::null_mut();
        let mut trackerent: *mut centity_t = std::ptr::null_mut();

        if argNum < 1 {
            debug_assert!(false);
            return;
        }

        indexNum = atoi(CG_Argv(1));

        if indexNum != -1 {
            clent = &mut cg_entities[indexNum as usize];
        }

        if argNum >= 2 {
            indexNum = atoi(CG_Argv(2));

            if indexNum != -1 {
                trackerent = &mut cg_entities[indexNum as usize];
            }
        }

        if !clent.is_null() {
            CG_S_StopLoopingSound((*clent).currentState.number, -1);
        }
        if !trackerent.is_null() {
            CG_S_StopLoopingSound((*trackerent).currentState.number, -1);
        }

        return;
    }

    if Q_stricmp(cmd, b"ircg\0".as_ptr() as *const c_char) == 0 {
        //this means param 2 is the body index and we want to copy to bodyqueue on it
        IRCG = 1; // qtrue
    }

    if Q_stricmp(cmd, b"rcg\0".as_ptr() as *const c_char) == 0 || IRCG != 0 {
        //rcg - Restore Client Ghoul (make sure limbs are reattached and ragdoll state is reset - this must be done reliably)
        let mut indexNum: c_int = 0;
        let mut argNum: c_int = trap_Argc();
        let mut clent: *mut centity_t = 0 as *mut centity_t;

        if argNum < 1 {
            debug_assert!(false);
            return;
        }

        indexNum = atoi(CG_Argv(1));
        if indexNum < 0 || indexNum >= MAX_CLIENTS as c_int {
            debug_assert!(false);
            return;
        }

        clent = &mut cg_entities[indexNum as usize];

        //assert(clent->ghoul2);
        if (*clent).ghoul2.is_null() {
            //this can happen while connecting as a client
            return;
        }

        if !trap_G2_HaveWeGhoul2Models((*clent).ghoul2) != 0 {
            debug_assert!(false, "Tried to reset state on a bad instance. Crash is inevitable.");
        }

        if IRCG != 0 {
            let mut bodyIndex: c_int = 0;
            let mut weaponIndex: c_int = 0;
            let mut side: c_int = 0;
            let mut body: *mut centity_t = 0 as *mut centity_t;

            debug_assert!(argNum >= 3);
            bodyIndex = atoi(CG_Argv(2));
            weaponIndex = atoi(CG_Argv(3));
            side = atoi(CG_Argv(4));

            body = &mut cg_entities[bodyIndex as usize];

            if side != 0 {
                (*body).teamPowerType = 1; // qtrue; //light side
            }
            else {
                (*body).teamPowerType = 0; // qfalse; //dark side
            }

            CG_BodyQueueCopy(body, (*clent).currentState.number, weaponIndex);
        }

        //reattach any missing limbs
        if (*clent).torsoBolt != 0 {
            CG_ReattachLimb(clent);
        }

        //make sure ragdoll state is reset
        if (*clent).isRagging != 0 {
            (*clent).isRagging = 0; // qfalse
            trap_G2API_SetRagDoll((*clent).ghoul2, std::ptr::null()); //calling with null parms resets to no ragdoll.
        }

        //clear all the decals as well
        trap_G2API_ClearSkinGore((*clent).ghoul2);

        (*clent).weapon = 0;
        (*clent).ghoul2weapon = std::ptr::null(); //force a weapon reinit

        return;
    }

    if Q_stricmp(cmd, b"cp\0".as_ptr() as *const c_char) == 0 {
        let mut strEd: [c_char; 1024] = [0; 1024];
        CG_CheckSVStringEdRef(strEd.as_mut_ptr(), CG_Argv(1));
        CG_CenterPrint(strEd.as_ptr(), SCREEN_HEIGHT * 30 / 100, BIGCHAR_WIDTH);
        return;
    }

    if Q_stricmp(cmd, b"cps\0".as_ptr() as *const c_char) == 0 {
        let mut strEd: [c_char; 1024] = [0; 1024];
        let mut x: *const c_char = CG_Argv(1);
        if *x as u8 == b'@' {
            x = x.offset(1);
        }
        trap_SP_GetStringTextString(x, strEd.as_mut_ptr(), MAX_STRINGED_SV_STRING as c_int);
        CG_CenterPrint(strEd.as_ptr(), SCREEN_HEIGHT * 20 / 100, BIGCHAR_WIDTH);
        return;
    }

    if Q_stricmp(cmd, b"cs\0".as_ptr() as *const c_char) == 0 {
        CG_ConfigStringModified();
        return;
    }

    if Q_stricmp(cmd, b"print\0".as_ptr() as *const c_char) == 0 {
        let mut strEd: [c_char; 1024] = [0; 1024];
        CG_CheckSVStringEdRef(strEd.as_mut_ptr(), CG_Argv(1));
        CG_Printf(b"%s\0".as_ptr() as *const c_char, strEd.as_ptr());
        return;
    }

    if Q_stricmp(cmd, b"chat\0".as_ptr() as *const c_char) == 0 {
        if cg_teamChatsOnly.integer == 0 {
            trap_S_StartLocalSound((*std::ptr::addr_of!(cgs)).media.talkSound, CHAN_LOCAL_SOUND);
            Q_strncpyz(text.as_mut_ptr(), CG_Argv(1), MAX_SAY_TEXT);
            CG_RemoveChatEscapeChar(text.as_mut_ptr());
            CG_ChatBox_AddString(text.as_mut_ptr());
            CG_Printf(b"*%s\n\0".as_ptr() as *const c_char, text.as_ptr());
        }
        return;
    }

    if Q_stricmp(cmd, b"tchat\0".as_ptr() as *const c_char) == 0 {
        trap_S_StartLocalSound((*std::ptr::addr_of!(cgs)).media.talkSound, CHAN_LOCAL_SOUND);
        Q_strncpyz(text.as_mut_ptr(), CG_Argv(1), MAX_SAY_TEXT);
        CG_RemoveChatEscapeChar(text.as_mut_ptr());
        CG_ChatBox_AddString(text.as_mut_ptr());
        CG_Printf(b"*%s\n\0".as_ptr() as *const c_char, text.as_ptr());

        return;
    }

    //chat with location, possibly localized.
    if Q_stricmp(cmd, b"lchat\0".as_ptr() as *const c_char) == 0 {
        if cg_teamChatsOnly.integer == 0 {
            let mut name: [c_char; 1024] = [0; 1024];
            let mut loc: [c_char; 1024] = [0; 1024];
            let mut color: [c_char; 8] = [0; 8];
            let mut message: [c_char; 1024] = [0; 1024];

            if trap_Argc() < 4 {
                return;
            }

            strcpy(name.as_mut_ptr(), CG_Argv(1));
            strcpy(loc.as_mut_ptr(), CG_Argv(2));
            strcpy(color.as_mut_ptr(), CG_Argv(3));
            strcpy(message.as_mut_ptr(), CG_Argv(4));

            if *loc.as_ptr() as u8 == b'@' {
                //get localized text
                trap_SP_GetStringTextString(loc.as_ptr().offset(1), loc.as_mut_ptr(), MAX_STRING_CHARS as c_int);
            }

            trap_S_StartLocalSound((*std::ptr::addr_of!(cgs)).media.talkSound, CHAN_LOCAL_SOUND);
            //Q_strncpyz( text, CG_Argv(1), MAX_SAY_TEXT );
            Com_sprintf(text.as_mut_ptr(), MAX_SAY_TEXT as c_int, b"%s<%s>^%s%s\0".as_ptr() as *const c_char, name.as_ptr(), loc.as_ptr(), color.as_ptr(), message.as_ptr());
            CG_RemoveChatEscapeChar(text.as_mut_ptr());
            CG_ChatBox_AddString(text.as_mut_ptr());
            CG_Printf(b"*%s\n\0".as_ptr() as *const c_char, text.as_ptr());
        }
        return;
    }
    if Q_stricmp(cmd, b"ltchat\0".as_ptr() as *const c_char) == 0 {
        let mut name: [c_char; 1024] = [0; 1024];
        let mut loc: [c_char; 1024] = [0; 1024];
        let mut color: [c_char; 8] = [0; 8];
        let mut message: [c_char; 1024] = [0; 1024];

        if trap_Argc() < 4 {
            return;
        }

        strcpy(name.as_mut_ptr(), CG_Argv(1));
        strcpy(loc.as_mut_ptr(), CG_Argv(2));
        strcpy(color.as_mut_ptr(), CG_Argv(3));
        strcpy(message.as_mut_ptr(), CG_Argv(4));

        if *loc.as_ptr() as u8 == b'@' {
            //get localized text
            trap_SP_GetStringTextString(loc.as_ptr().offset(1), loc.as_mut_ptr(), MAX_STRING_CHARS as c_int);
        }

        trap_S_StartLocalSound((*std::ptr::addr_of!(cgs)).media.talkSound, CHAN_LOCAL_SOUND);
        //Q_strncpyz( text, CG_Argv(1), MAX_SAY_TEXT );
        Com_sprintf(text.as_mut_ptr(), MAX_SAY_TEXT as c_int, b"%s<%s> ^%s%s\0".as_ptr() as *const c_char, name.as_ptr(), loc.as_ptr(), color.as_ptr(), message.as_ptr());
        CG_RemoveChatEscapeChar(text.as_mut_ptr());
        CG_ChatBox_AddString(text.as_mut_ptr());
        CG_Printf(b"*%s\n\0".as_ptr() as *const c_char, text.as_ptr());

        return;
    }

    if Q_stricmp(cmd, b"scores\0".as_ptr() as *const c_char) == 0 {
        CG_ParseScores();
        return;
    }

    if Q_stricmp(cmd, b"tinfo\0".as_ptr() as *const c_char) == 0 {
        CG_ParseTeamInfo();
        return;
    }

    if Q_stricmp(cmd, b"map_restart\0".as_ptr() as *const c_char) == 0 {
        CG_MapRestart();
        return;
    }

    if Q_stricmp(cmd, b"remapShader\0".as_ptr() as *const c_char) == 0 {
        if trap_Argc() == 4 {
            trap_R_RemapShader(CG_Argv(1), CG_Argv(2), CG_Argv(3));
        }
    }

    // loaddeferred can be both a servercmd and a consolecmd
    if Q_stricmp(cmd, b"loaddefered\0".as_ptr() as *const c_char) == 0 {
        // FIXME: spelled wrong, but not changing for demo
        CG_LoadDeferredPlayers();
        return;
    }

    // clientLevelShot is sent before taking a special screenshot for
    // the menu system during development
    if Q_stricmp(cmd, b"clientLevelShot\0".as_ptr() as *const c_char) == 0 {
        (*std::ptr::addr_of_mut!(cg)).levelShot = 1; // qtrue
        return;
    }

    CG_Printf(b"Unknown client game command: %s\n\0".as_ptr() as *const c_char, cmd);
}


/*
====================
CG_ExecuteNewServerCommands

Execute all of the server commands that were received along
with this this snapshot.
====================
*/
pub unsafe fn CG_ExecuteNewServerCommands(latestSequence: c_int) {
    while (*std::ptr::addr_of_mut!(cgs)).serverCommandSequence < latestSequence {
        if trap_GetServerCommand((*std::ptr::addr_of_mut!(cgs)).serverCommandSequence + 1) != 0 {
            (*std::ptr::addr_of_mut!(cgs)).serverCommandSequence += 1;
            CG_ServerCommand();
        } else {
            (*std::ptr::addr_of_mut!(cgs)).serverCommandSequence += 1;
        }
    }
}
