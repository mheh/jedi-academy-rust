//! Slice of `NPC_stats.c` — the NPC stats / `scripts/NPCs.cfg` parser file.
//! Opened bottom-up at its one self-contained leaf: `NPC_ReactionTime`, a tiny
//! accessor over the `NPCInfo` think global's loaded stats.
//!
//! The parser-layer keystone has landed: the parse-buffer global [`NPCParms`] +
//! the `.npc`-file loader [`NPC_LoadParms`] (over `trap_FS_*`). The bulk consumer
//! `NPC_ParseParms` (~2300 lines of token cases over `NPCParms`/`NPCFile`, the
//! `Q3_*` helpers, `BG_ParseLiteral`, the `gNPC_t` stat setters, and the
//! class/team string tables) is the remaining follow-on work, along with
//! `NPC_PrecacheAnimationCFG` and the `NPC_Set*Stats` family.

#![allow(non_upper_case_globals, non_snake_case)] // C function/global names kept verbatim

use core::ffi::{c_char, c_int, CStr};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use crate::codemp::game::b_public_h::{
    rank_t, BS_ADVANCE_FIGHT, BS_CINEMATIC, BS_DEFAULT, BS_FOLLOW_LEADER, BS_JUMP, BS_NOCLIP,
    BS_REMOVE, BS_SEARCH, BS_SLEEP, BS_WANDER, NUM_BSTATES, RANK_CAPTAIN, RANK_CIVILIAN,
    RANK_COMMANDER, RANK_CREWMAN, RANK_ENSIGN, RANK_LT, RANK_LT_COMM, RANK_LT_JG, SCF_ALT_FIRE,
};
use crate::codemp::game::bg_misc::{BG_FindItemForWeapon, BG_TempAlloc, BG_TempFree};
use crate::codemp::game::bg_panimate::BG_ParseAnimationFile;
use crate::codemp::game::bg_public::{
    animation_t, CROUCH_MAXS_2, DEFAULT_MAXS_2, DEFAULT_MINS_2, EF2_FLYING, STAT_MAX_HEALTH,
    STAT_WEAPONS,
};
use crate::codemp::game::bg_saberLoad::{
    BG_ParseLiteral, TranslateSaberColor, WP_RemoveSaber, WP_SaberParseParms,
};
use crate::codemp::game::bg_saga::{FPTable, WPTable};
use crate::codemp::game::bg_vehicles_h::VH_FIGHTER;
use crate::codemp::game::bg_weapons::weaponData;
use crate::codemp::game::bg_weapons_h::{weapon_t, WP_NONE, WP_NUM_WEAPONS, WP_SABER};
use crate::codemp::game::g_client::SetupGameGhoul2Model;
use crate::codemp::game::g_items::RegisterItem;
use crate::codemp::game::g_local::gentity_t;
use crate::codemp::game::g_main::{Com_Printf, G_Error};
use crate::codemp::game::g_public_h::{
    BSET_ANGER, BSET_ATTACK, BSET_AWAKE, BSET_BLOCKED, BSET_BUMPED, BSET_DEATH, BSET_DELAYED,
    BSET_FFDEATH, BSET_FFIRE, BSET_FLEE, BSET_INVALID, BSET_LOSTENEMY, BSET_PAIN, BSET_SPAWN,
    BSET_STUCK, BSET_USE, BSET_VICTORY,
};
use crate::codemp::game::g_spawn::G_NewString;
use crate::codemp::game::g_utils::{G_ModelIndex, G_SetOrigin, G_SoundIndex};
use crate::codemp::game::npc::NPCInfo;
use crate::codemp::game::npc_spawn::NPC_WeaponsForTeam;
use crate::codemp::game::q_math::Q_irand;
use crate::codemp::game::q_math::{VectorCopy, VectorSet};
use crate::codemp::game::q_shared::{
    va, COM_BeginParseSession, COM_Compress, COM_ParseExt, COM_ParseFloat, COM_ParseInt,
    COM_ParseString, Com_sprintf, GetIDForString, Q_strcat, Q_stricmp, Q_strncpyz,
    SkipBracedSection, SkipRestOfLine, Sz,
};
use crate::codemp::game::q_shared_h::{
    qboolean, saber_colors_t, stringID_table_t, vec3_t, FP_FIRST, FS_READ, MAX_BLADES, MAX_QPATH,
    NUM_FORCE_POWERS, QFALSE, QTRUE, SFL_TWO_HANDED,
};
use crate::codemp::game::teams_h::{
    class_t, npcteam_t, CLASS_ATST, CLASS_BARTENDER, CLASS_BESPIN_COP, CLASS_BOBAFETT, CLASS_CLAW,
    CLASS_COMMANDO, CLASS_DESANN, CLASS_FISH, CLASS_FLIER2, CLASS_GALAK, CLASS_GALAKMECH,
    CLASS_GLIDER, CLASS_GONK, CLASS_GRAN, CLASS_HOWLER, CLASS_IMPERIAL, CLASS_IMPWORKER,
    CLASS_INTERROGATOR, CLASS_JAN, CLASS_JAWA, CLASS_JEDI, CLASS_KYLE, CLASS_LANDO, CLASS_LIZARD,
    CLASS_LUKE, CLASS_MARK1, CLASS_MARK2, CLASS_MINEMONSTER, CLASS_MONMOTHA, CLASS_MORGANKATARN,
    CLASS_MOUSE, CLASS_MURJJ, CLASS_NONE, CLASS_PRISONER, CLASS_PROBE, CLASS_PROTOCOL, CLASS_R2D2,
    CLASS_R5D2, CLASS_RANCOR, CLASS_REBEL, CLASS_REBORN, CLASS_REELO, CLASS_REMOTE, CLASS_RODIAN,
    CLASS_SEEKER, CLASS_SENTRY, CLASS_SHADOWTROOPER, CLASS_STORMTROOPER, CLASS_SWAMP,
    CLASS_SWAMPTROOPER, CLASS_TAVION, CLASS_TRANDOSHAN, CLASS_UGNAUGHT, CLASS_VEHICLE, CLASS_WAMPA,
    CLASS_WEEQUAY, NPCTEAM_ENEMY, NPCTEAM_FREE, NPCTEAM_NEUTRAL, NPCTEAM_PLAYER,
};
use crate::trap;

// g_public.h svFlags (PC) — not yet hoisted into g_public_h.rs; defined locally to keep this
// parallel re-port within npc_stats.rs. Values match refs/raven-jediacademy/codemp/game/g_public.h.
const SVF_NO_BASIC_SOUNDS: c_int = 0x10000000; // No basic sounds
const SVF_NO_COMBAT_SOUNDS: c_int = 0x20000000; // No combat sounds
const SVF_NO_EXTRA_SOUNDS: c_int = 0x40000000; // No extra or jedi sounds

/*
static rank_t TranslateRankName( const char *name )

  Should be used to determine pip bolt-ons
*/
#[allow(dead_code)] // static helper; sole caller NPC_ParseParms is still blocked (NPCs.cfg parser globals)
unsafe fn TranslateRankName(name: *const c_char) -> rank_t {
    if Q_stricmp(name, c"civilian".as_ptr()) == 0 {
        return RANK_CIVILIAN;
    }

    if Q_stricmp(name, c"crewman".as_ptr()) == 0 {
        return RANK_CREWMAN;
    }

    if Q_stricmp(name, c"ensign".as_ptr()) == 0 {
        return RANK_ENSIGN;
    }

    if Q_stricmp(name, c"ltjg".as_ptr()) == 0 {
        return RANK_LT_JG;
    }

    if Q_stricmp(name, c"lt".as_ptr()) == 0 {
        return RANK_LT;
    }

    if Q_stricmp(name, c"ltcomm".as_ptr()) == 0 {
        return RANK_LT_COMM;
    }

    if Q_stricmp(name, c"commander".as_ptr()) == 0 {
        return RANK_COMMANDER;
    }

    if Q_stricmp(name, c"captain".as_ptr()) == 0 {
        return RANK_CAPTAIN;
    }

    RANK_CIVILIAN
}

/*
NPC_ReactionTime
*/
//FIXME use grandom in here
pub unsafe fn NPC_ReactionTime() -> c_int {
    200 * (6 - (*NPCInfo).stats.reactions)
}

pub unsafe fn G_ParseAnimFileSet(
    filename: *const c_char,
    _animCFG: *const c_char,
    animFileIndex: *mut c_int,
) -> qboolean {
    *animFileIndex = BG_ParseAnimationFile(filename, null_mut(), QFALSE);
    //if it's humanoid we should have it cached and return it, if it is not it will be loaded (unless it's also cached already)

    if *animFileIndex == -1 {
        return QFALSE;
    }

    //I guess this isn't really even needed game-side.
    //BG_ParseAnimationSndFile(filename, *animFileIndex);
    QTRUE
}

extern "C" {
    /// libc `size_t strlen(const char *s)` — `bg_lib`'s shim is `Q3_VM`-gated (configured out of
    /// the native build), so we declare the libc symbol directly (the `bg_saberLoad` precedent).
    fn strlen(s: *const c_char) -> usize;
    /// libc `char *strcpy(char *dst, const char *src)` — same `Q3_VM`-gated-shim rationale.
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    /// libc `char *strstr(const char *haystack, const char *needle)` — same rationale.
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    /// libc `int atoi(const char *s)` — same rationale.
    fn atoi(s: *const c_char) -> c_int;
}

/// `void SpewDebugStuffToFile(animation_t *anims)` (NPC_stats.c:403) — a `BG_ParseAnimationFile`
/// debug dump (writes per-anim `frameLerp`s to `file_of_debug_stuff_SP.txt`). Its entire body is
/// inside `#ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING` (NPC_stats.c:402), and that macro is never
/// `#define`d (the `#define` at line 400 is commented out), so in the live build the function body
/// is empty. Ported as an empty body to match. `pub` to avoid a dead-code warning.
pub unsafe fn SpewDebugStuffToFile(_anims: *mut animation_t) {
    // #ifdef CONVENIENT_ANIMATION_FILE_DEBUG_THING … #endif (dead in live build):
    // trap_FS_FOpenFile / loop strcat'ing "%i %i\n" frameLerp per anim / trap_FS_Write / FCloseFile.
}

/// `void NPC_PrecacheAnimationCFG( const char *NPC_type )` (NPC_stats.c:444). The entire body is
/// inside `#if 0 //rwwFIXMEFIXME: Actually precache stuff here.` (NPC_stats.c:446 … #endif:552), so
/// in the live build the function is empty. Ported as an empty body to match. `pub` to avoid a
/// dead-code warning.
pub unsafe fn NPC_PrecacheAnimationCFG(_NPC_type: *const c_char) {
    // #if 0 … #endif (dead in live build): would walk NPCParms for NPC_type's block and, on the
    // `legsmodel`/`playerModel` tokens, call G_ParseAnimFileSet to precache the anim file set.
}

/// `void NPC_PrecacheWeapons( team_t playerTeam, int spawnflags, char *NPCtype )`
/// (NPC_stats.c:556) — registers the items for every weapon in [`NPC_WeaponsForTeam`]'s bitmask
/// (`WP_SABER..WP_NUM_WEAPONS`). The trailing `#if 0` block (NPC_stats.c:569–595, the old
/// SP `FindItemForWeapon`/`CG_RegisterItem*`/ghoul2 weapon-model precache) is carried as comments,
/// not live code. No oracle — fires engine traps via [`RegisterItem`].
pub unsafe fn NPC_PrecacheWeapons(playerTeam: npcteam_t, spawnflags: c_int, NPCtype: *mut c_char) {
    let weapons = NPC_WeaponsForTeam(playerTeam, spawnflags, NPCtype);
    let mut curWeap = WP_SABER;

    while curWeap < WP_NUM_WEAPONS {
        if weapons & (1 << curWeap) != 0 {
            RegisterItem(BG_FindItemForWeapon(curWeap as weapon_t));
        }
        curWeap += 1;
    }

    // #if 0 //rwwFIXMEFIXME: actually precache weapons here
    //	int weapons = NPC_WeaponsForTeam( playerTeam, spawnflags, NPCtype );
    //	gitem_t	*item;
    //	for ( int curWeap = WP_SABER; curWeap < WP_NUM_WEAPONS; curWeap++ )
    //	{
    //		if ( (weapons & ( 1 << curWeap )) )
    //		{
    //			item = FindItemForWeapon( ((weapon_t)(curWeap)) );	//precache the weapon
    //			CG_RegisterItemSounds( (item-bg_itemlist) );
    //			CG_RegisterItemVisuals( (item-bg_itemlist) );
    //			//precache the in-hand/in-world ghoul2 weapon model
    //
    //			char weaponModel[64];
    //
    //			strcpy (weaponModel, weaponData[curWeap].weaponMdl);
    //			if (char *spot = strstr(weaponModel, ".md3") ) {
    //				*spot = 0;
    //				spot = strstr(weaponModel, "_w");//i'm using the in view weapon array instead of scanning the item list, so put the _w back on
    //				if (!spot) {
    //					strcat (weaponModel, "_w");
    //				}
    //				strcat (weaponModel, ".glm");	//and change to ghoul2
    //			}
    //			gi.G2API_PrecacheGhoul2Model( weaponModel ); // correct way is item->world_model
    //		}
    //	}
    // #endif
}

/*
void NPC_Precache ( char *NPCName )

Precaches NPC skins, tgas and md3s.

*/
/// `void NPC_Precache ( gentity_t *spawner )` (NPC_stats.c:604) — a smaller sibling of
/// [`NPC_ParseParms`]: walks the [`NPCParms`] text for `spawner->NPC_type`'s block and, per token,
/// indexes the sound sets ([`G_SoundIndex`]), the player model ([`G_ModelIndex`]), and the
/// starting weapon's item ([`RegisterItem`]/[`BG_FindItemForWeapon`]), then calls
/// [`NPC_PrecacheWeapons`] for the team's possible weapons. The commented-out C
/// (`Q_strncpyz(ri.*ModelName…)`, the SVF sound-flag guards rewritten to `if (1)`, the
/// `CG_RegisterNPC*` SP precache calls) is carried as comments. No oracle — entity/global side
/// effects + engine traps, which the off-engine oracle harness cannot satisfy.
pub unsafe fn NPC_Precache(spawner: *mut gentity_t) {
    let mut playerTeam: npcteam_t = NPCTEAM_FREE;
    let mut value: *const c_char = null_mut();
    let mut p: *const c_char;
    let mut patch: *mut c_char;
    let mut sound: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut md3Model: qboolean = QFALSE;
    let mut playerModel: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut customSkin: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

    if Q_stricmp(c"random".as_ptr(), (*spawner).NPC_type) == 0 {
        //sorry, can't precache a random just yet
        return;
    }
    strcpy(customSkin.as_mut_ptr(), c"default".as_ptr());

    p = addr_of!(NPCParms) as *const c_char;
    COM_BeginParseSession(addr_of!(NPCFile) as *const c_char);

    // look for the right NPC
    while !p.is_null() {
        let token = COM_ParseExt(&mut p, QTRUE);
        if *token == 0 {
            return;
        }

        if Q_stricmp(token, (*spawner).NPC_type) == 0 {
            break;
        }

        SkipBracedSection(&mut p);
    }

    if p.is_null() {
        return;
    }

    if BG_ParseLiteral(&mut p, c"{".as_ptr()) != QFALSE {
        return;
    }

    // parse the NPC info block
    loop {
        let token = COM_ParseExt(&mut p, QTRUE);
        if *token == 0 {
            Com_Printf(&format!(
                "^1ERROR: unexpected EOF while parsing '{}'\n",
                Sz((*spawner).NPC_type)
            ));
            return;
        }

        if Q_stricmp(token, c"}".as_ptr()) == 0 {
            break;
        }

        // headmodel
        if Q_stricmp(token, c"headmodel".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }

            if Q_stricmp(c"none".as_ptr(), value) == 0 {
            } else {
                //Q_strncpyz( ri.headModelName, value, sizeof(ri.headModelName), qtrue);
            }
            md3Model = QTRUE;
            continue;
        }

        // torsomodel
        if Q_stricmp(token, c"torsomodel".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }

            if Q_stricmp(c"none".as_ptr(), value) == 0 {
            } else {
                //Q_strncpyz( ri.torsoModelName, value, sizeof(ri.torsoModelName), qtrue);
            }
            md3Model = QTRUE;
            continue;
        }

        // legsmodel
        if Q_stricmp(token, c"legsmodel".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            //Q_strncpyz( ri.legsModelName, value, sizeof(ri.legsModelName), qtrue);
            md3Model = QTRUE;
            continue;
        }

        // playerModel
        if Q_stricmp(token, c"playerModel".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            Q_strncpyz(
                playerModel.as_mut_ptr(),
                value,
                core::mem::size_of_val(&playerModel) as c_int,
            );
            md3Model = QFALSE;
            continue;
        }

        // customSkin
        if Q_stricmp(token, c"customSkin".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            Q_strncpyz(
                customSkin.as_mut_ptr(),
                value,
                core::mem::size_of_val(&customSkin) as c_int,
            );
            continue;
        }

        // playerTeam
        if Q_stricmp(token, c"playerTeam".as_ptr()) == 0 {
            let mut tk: [c_char; 4096] = [0; 4096]; //rww - hackilicious!

            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            //playerTeam = TranslateTeamName(value);
            Com_sprintf(
                tk.as_mut_ptr(),
                core::mem::size_of_val(&tk) as c_int,
                format_args!("NPC{}", Sz(token)),
            );
            playerTeam = GetIDForString(addr_of!(TeamTable) as *const stringID_table_t, tk.as_ptr())
                as npcteam_t;
            continue;
        }

        // snd
        if Q_stricmp(token, c"snd".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            if ((*spawner).r.svFlags & SVF_NO_BASIC_SOUNDS) == 0 {
                //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                Q_strncpyz(
                    sound.as_mut_ptr(),
                    value,
                    core::mem::size_of_val(&sound) as c_int,
                );
                patch = strstr(sound.as_ptr(), c"/".as_ptr());
                if !patch.is_null() {
                    *patch = 0;
                }
                (*spawner).s.csSounds_Std = G_SoundIndex(&format!("*${}", Sz(sound.as_ptr())));
            }
            continue;
        }

        // sndcombat
        if Q_stricmp(token, c"sndcombat".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            if ((*spawner).r.svFlags & SVF_NO_COMBAT_SOUNDS) == 0 {
                //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                Q_strncpyz(
                    sound.as_mut_ptr(),
                    value,
                    core::mem::size_of_val(&sound) as c_int,
                );
                patch = strstr(sound.as_ptr(), c"/".as_ptr());
                if !patch.is_null() {
                    *patch = 0;
                }
                (*spawner).s.csSounds_Combat = G_SoundIndex(&format!("*${}", Sz(sound.as_ptr())));
            }
            continue;
        }

        // sndextra
        if Q_stricmp(token, c"sndextra".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            if ((*spawner).r.svFlags & SVF_NO_EXTRA_SOUNDS) == 0 {
                //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                Q_strncpyz(
                    sound.as_mut_ptr(),
                    value,
                    core::mem::size_of_val(&sound) as c_int,
                );
                patch = strstr(sound.as_ptr(), c"/".as_ptr());
                if !patch.is_null() {
                    *patch = 0;
                }
                (*spawner).s.csSounds_Extra = G_SoundIndex(&format!("*${}", Sz(sound.as_ptr())));
            }
            continue;
        }

        // sndjedi
        if Q_stricmp(token, c"sndjedi".as_ptr()) == 0 {
            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }
            if ((*spawner).r.svFlags & SVF_NO_EXTRA_SOUNDS) == 0 {
                //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                Q_strncpyz(
                    sound.as_mut_ptr(),
                    value,
                    core::mem::size_of_val(&sound) as c_int,
                );
                patch = strstr(sound.as_ptr(), c"/".as_ptr());
                if !patch.is_null() {
                    *patch = 0;
                }
                (*spawner).s.csSounds_Jedi = G_SoundIndex(&format!("*${}", Sz(sound.as_ptr())));
            }
            continue;
        }

        if Q_stricmp(token, c"weapon".as_ptr()) == 0 {
            let curWeap: c_int;

            if COM_ParseString(&mut p, &mut value) != QFALSE {
                continue;
            }

            curWeap = GetIDForString(addr_of!(WPTable) as *const stringID_table_t, value);

            if curWeap > WP_NONE && curWeap < WP_NUM_WEAPONS {
                RegisterItem(BG_FindItemForWeapon(curWeap as weapon_t));
            }
            continue;
        }
    }

    // If we're not a vehicle, then an error here would be valid...
    if (*spawner).client.is_null() || (*(*spawner).client).NPC_class != CLASS_VEHICLE {
        if md3Model != QFALSE {
            Com_Printf("MD3 model using NPCs are not supported in MP\n");
        } else {
            //if we have a model/skin then index them so they'll be registered immediately
            //when the client gets a configstring update.
            let mut modelName: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];

            Com_sprintf(
                modelName.as_mut_ptr(),
                core::mem::size_of_val(&modelName) as c_int,
                format_args!("models/players/{}/model.glm", Sz(playerModel.as_ptr())),
            );
            if customSkin[0] != 0 {
                //append it after a *
                Q_strcat(
                    modelName.as_mut_ptr(),
                    core::mem::size_of_val(&modelName) as c_int,
                    va(format_args!("*{}", Sz(customSkin.as_ptr()))),
                );
            }

            G_ModelIndex(&Sz(modelName.as_ptr()).to_string());
        }
    }

    //precache this NPC's possible weapons
    NPC_PrecacheWeapons(playerTeam, (*spawner).spawnflags, (*spawner).NPC_type);

    //	CG_RegisterNPCCustomSounds( &ci );
    //	CG_RegisterNPCEffects( playerTeam );
    //rwwFIXMEFIXME: same
    //FIXME: Look for a "sounds" directory and precache death, pain, alert sounds
}

/// `void NPC_BuildRandom( gentity_t *NPC )` (NPC_stats.c:885). Its entire body is inside `#if 0`
/// (NPC_stats.c:884 … #endif:980), so in the live build the function is empty. Ported as an empty
/// body to match. `pub` to avoid a dead-code warning.
pub unsafe fn NPC_BuildRandom(_NPC: *mut gentity_t) {
    // #if 0 … #endif (dead in live build): would randomly assemble a starfleet crewman's
    // head/torso/legs model names by sex/color/head index, set modelScale/rank/team/customBasicSoundDir.
}

/// `#define MAX_NPC_DATA_SIZE 0x20000` (NPC_stats.c:241) — capacity of the concatenated NPC parse
/// buffer ([`NPCParms`]) and the per-file read scratch ([`npcParseBuffer`]).
const MAX_NPC_DATA_SIZE: usize = 0x20000;

/// `char NPCParms[MAX_NPC_DATA_SIZE]` (NPC_stats.c:242) — the concatenated text of every loaded
/// `ext_data/NPCs/*.npc` file, later tokenized per-NPC by `NPC_ParseParms`. Filled by
/// [`NPC_LoadParms`].
pub static mut NPCParms: [c_char; MAX_NPC_DATA_SIZE] = [0; MAX_NPC_DATA_SIZE];

/// `char npcParseBuffer[MAX_NPC_DATA_SIZE]` (NPC_stats.c:3291, non-`_XBOX`) — scratch each `.npc`
/// file is read into before being appended to [`NPCParms`].
static mut npcParseBuffer: [c_char; MAX_NPC_DATA_SIZE] = [0; MAX_NPC_DATA_SIZE];

/// `char NPCFile[MAX_QPATH]` (NPC_stats.c:243) — the current parse-session filename handed to
/// `COM_BeginParseSession` by `NPC_ParseParms` for error messages.
pub static mut NPCFile: [c_char; MAX_QPATH] = [0; MAX_QPATH];

/// `ENUM2STRING`/`stringID_table_t` entry constructor (`{ #x, x }`), matching the `bg_saga` idiom.
const fn s(name: &'static CStr, id: c_int) -> stringID_table_t {
    stringID_table_t {
        name: name.as_ptr(),
        id,
    }
}

/// `stringID_table_t TeamTable[]` (NPC_stats.c:19) — `NPCTEAM_*` names; `NPCTEAM_FREE` stays entry 0.
/// The `{ "", -1 }` row terminates.
pub static mut TeamTable: [stringID_table_t; 5] = [
    s(c"NPCTEAM_FREE", NPCTEAM_FREE),
    s(c"NPCTEAM_PLAYER", NPCTEAM_PLAYER),
    s(c"NPCTEAM_ENEMY", NPCTEAM_ENEMY),
    s(c"NPCTEAM_NEUTRAL", NPCTEAM_NEUTRAL),
    s(c"", -1),
];

/// `stringID_table_t ClassTable[]` (NPC_stats.c:29) — `CLASS_*` names; MUST stay in the same order
/// as the `CLASS_` enum in teams.h. Commented-out source rows (the mid-list `CLASS_RANCOR`,
/// `CLASS_ROCKETTROOPER`, `CLASS_PLAYER`) are omitted exactly as in C. The `{ "", -1 }` row
/// terminates.
pub static mut ClassTable: [stringID_table_t; 57] = [
    s(c"CLASS_NONE", CLASS_NONE),
    s(c"CLASS_ATST", CLASS_ATST),
    s(c"CLASS_BARTENDER", CLASS_BARTENDER),
    s(c"CLASS_BESPIN_COP", CLASS_BESPIN_COP),
    s(c"CLASS_CLAW", CLASS_CLAW),
    s(c"CLASS_COMMANDO", CLASS_COMMANDO),
    s(c"CLASS_DESANN", CLASS_DESANN),
    s(c"CLASS_FISH", CLASS_FISH),
    s(c"CLASS_FLIER2", CLASS_FLIER2),
    s(c"CLASS_GALAK", CLASS_GALAK),
    s(c"CLASS_GLIDER", CLASS_GLIDER),
    s(c"CLASS_GONK", CLASS_GONK),
    s(c"CLASS_GRAN", CLASS_GRAN),
    s(c"CLASS_HOWLER", CLASS_HOWLER),
    s(c"CLASS_IMPERIAL", CLASS_IMPERIAL),
    s(c"CLASS_IMPWORKER", CLASS_IMPWORKER),
    s(c"CLASS_INTERROGATOR", CLASS_INTERROGATOR),
    s(c"CLASS_JAN", CLASS_JAN),
    s(c"CLASS_JEDI", CLASS_JEDI),
    s(c"CLASS_KYLE", CLASS_KYLE),
    s(c"CLASS_LANDO", CLASS_LANDO),
    s(c"CLASS_LIZARD", CLASS_LIZARD),
    s(c"CLASS_LUKE", CLASS_LUKE),
    s(c"CLASS_MARK1", CLASS_MARK1),
    s(c"CLASS_MARK2", CLASS_MARK2),
    s(c"CLASS_GALAKMECH", CLASS_GALAKMECH),
    s(c"CLASS_MINEMONSTER", CLASS_MINEMONSTER),
    s(c"CLASS_MONMOTHA", CLASS_MONMOTHA),
    s(c"CLASS_MORGANKATARN", CLASS_MORGANKATARN),
    s(c"CLASS_MOUSE", CLASS_MOUSE),
    s(c"CLASS_MURJJ", CLASS_MURJJ),
    s(c"CLASS_PRISONER", CLASS_PRISONER),
    s(c"CLASS_PROBE", CLASS_PROBE),
    s(c"CLASS_PROTOCOL", CLASS_PROTOCOL),
    s(c"CLASS_R2D2", CLASS_R2D2),
    s(c"CLASS_R5D2", CLASS_R5D2),
    s(c"CLASS_REBEL", CLASS_REBEL),
    s(c"CLASS_REBORN", CLASS_REBORN),
    s(c"CLASS_REELO", CLASS_REELO),
    s(c"CLASS_REMOTE", CLASS_REMOTE),
    s(c"CLASS_RODIAN", CLASS_RODIAN),
    s(c"CLASS_SEEKER", CLASS_SEEKER),
    s(c"CLASS_SENTRY", CLASS_SENTRY),
    s(c"CLASS_SHADOWTROOPER", CLASS_SHADOWTROOPER),
    s(c"CLASS_STORMTROOPER", CLASS_STORMTROOPER),
    s(c"CLASS_SWAMP", CLASS_SWAMP),
    s(c"CLASS_SWAMPTROOPER", CLASS_SWAMPTROOPER),
    s(c"CLASS_TAVION", CLASS_TAVION),
    s(c"CLASS_TRANDOSHAN", CLASS_TRANDOSHAN),
    s(c"CLASS_UGNAUGHT", CLASS_UGNAUGHT),
    s(c"CLASS_JAWA", CLASS_JAWA),
    s(c"CLASS_WEEQUAY", CLASS_WEEQUAY),
    s(c"CLASS_BOBAFETT", CLASS_BOBAFETT),
    s(c"CLASS_VEHICLE", CLASS_VEHICLE),
    s(c"CLASS_RANCOR", CLASS_RANCOR),
    s(c"CLASS_WAMPA", CLASS_WAMPA),
    s(c"", -1),
];

/// `stringID_table_t BSTable[]` (NPC_stats.c:93) — `BS_*` behavior-state names (the rest are
/// internal-only). The `{ "", -1 }` row terminates.
pub static mut BSTable: [stringID_table_t; 11] = [
    s(c"BS_DEFAULT", BS_DEFAULT),
    s(c"BS_ADVANCE_FIGHT", BS_ADVANCE_FIGHT),
    s(c"BS_SLEEP", BS_SLEEP),
    s(c"BS_FOLLOW_LEADER", BS_FOLLOW_LEADER),
    s(c"BS_JUMP", BS_JUMP),
    s(c"BS_SEARCH", BS_SEARCH),
    s(c"BS_WANDER", BS_WANDER),
    s(c"BS_NOCLIP", BS_NOCLIP),
    s(c"BS_REMOVE", BS_REMOVE),
    s(c"BS_CINEMATIC", BS_CINEMATIC),
    s(c"", -1),
];

/// `stringID_table_t BSETTable[]` (NPC_stats.c:111) — `BSET_*` behavior-set script names. The C
/// `stringIDExpand("", BSET_INVALID)` macro expands to both a `{ "", BSET_INVALID }` and a
/// `{ "BSET_INVALID", BSET_INVALID }` row. The `{ "", -1 }` row terminates.
pub static mut BSETTable: [stringID_table_t; 19] = [
    s(c"BSET_SPAWN", BSET_SPAWN),
    s(c"BSET_USE", BSET_USE),
    s(c"BSET_AWAKE", BSET_AWAKE),
    s(c"BSET_ANGER", BSET_ANGER),
    s(c"BSET_ATTACK", BSET_ATTACK),
    s(c"BSET_VICTORY", BSET_VICTORY),
    s(c"BSET_LOSTENEMY", BSET_LOSTENEMY),
    s(c"BSET_PAIN", BSET_PAIN),
    s(c"BSET_FLEE", BSET_FLEE),
    s(c"BSET_DEATH", BSET_DEATH),
    s(c"BSET_DELAYED", BSET_DELAYED),
    s(c"BSET_BLOCKED", BSET_BLOCKED),
    s(c"BSET_BUMPED", BSET_BUMPED),
    s(c"BSET_STUCK", BSET_STUCK),
    s(c"BSET_FFIRE", BSET_FFIRE),
    s(c"BSET_FFDEATH", BSET_FFDEATH),
    s(c"", BSET_INVALID),
    s(c"BSET_INVALID", BSET_INVALID),
    s(c"", -1),
];

/// `void NPC_LoadParms( void )` (NPC_stats.c:3294) — the NPCs.cfg parser-layer keystone: scans
/// `ext_data/NPCs` for `.npc` files and concatenates each one's text (separated by `"\n"`) into the
/// global [`NPCParms`] block, which `NPC_ParseParms` later tokenizes per-NPC. Each file is read into
/// the [`npcParseBuffer`] scratch, NUL-terminated, then appended; overflowing
/// [`MAX_NPC_DATA_SIZE`] is a fatal [`G_Error`].
///
/// Faithful to the retail `QAGAME` build: the `trap_FS_*` traps drive it (the `BG_VehWeaponLoadParms`/
/// [`WP_SaberLoadParms`](crate::codemp::game::bg_saberLoad) precedent). `va( "ext_data/NPCs/%s", … )`
/// collapses into a Rust `format!` since [`trap::FS_FOpenFile`] takes `&str`; the dead `mainBlockLen`
/// (assigned `len`, never read) is dropped. C's bare `strcat` becomes the bounded [`Q_strcat`]
/// (`MAX_NPC_DATA_SIZE - totallen` headroom) — same result, no overrun. The non-`_XBOX` static
/// `npcParseBuffer` is used directly (no `Z_Malloc`/`Z_Free` game-VM syscall exists). See
/// `DEVIATIONS.md`.
///
/// No oracle — pure engine-trap file I/O (`trap_FS_*`), which the off-engine oracle harness cannot
/// satisfy (the [`WP_SaberLoadParms`](crate::codemp::game::bg_saberLoad) precedent).
pub fn NPC_LoadParms() {
    let mut len: c_int;
    let mut totallen: c_int;
    let fileCnt: c_int;
    let mut npcExtensionListBuf = [0 as c_char; 2048]; // The list of file names read in

    len = 0;

    // SAFETY: single-threaded module; the global `NPCParms` text block and the `npcParseBuffer`
    // read scratch are walked with raw pointers exactly as the C does.
    unsafe {
        //remember where to store the next one
        totallen = len;
        let base = addr_of_mut!(NPCParms) as *mut c_char;
        let mut marker = base.add(totallen as usize);
        *marker = 0;

        let scratch = addr_of_mut!(npcParseBuffer) as *mut c_char;

        //now load in the extra .npc extensions
        fileCnt = trap::FS_GetFileList("ext_data/NPCs", ".npc", &mut npcExtensionListBuf);

        let mut holdChar = npcExtensionListBuf.as_mut_ptr();

        let mut i = 0;
        while i < fileCnt {
            let npcExtFNLen = strlen(holdChar) as c_int;

            let path = format!(
                "ext_data/NPCs/{}",
                CStr::from_ptr(holdChar).to_string_lossy()
            );
            let (l, f) = trap::FS_FOpenFile(&path, FS_READ);
            len = l;

            if len == -1 {
                Com_Printf("error reading file\n");
            } else {
                if totallen + len >= MAX_NPC_DATA_SIZE as c_int {
                    G_Error("NPC extensions (*.npc) are too large");
                }

                let buf = core::slice::from_raw_parts_mut(scratch as *mut u8, len as usize);
                trap::FS_Read(buf, f);
                *scratch.add(len as usize) = 0;

                len = COM_Compress(scratch);

                Q_strcat(marker, MAX_NPC_DATA_SIZE as c_int - totallen, scratch);
                Q_strcat(
                    marker,
                    MAX_NPC_DATA_SIZE as c_int - totallen,
                    c"\n".as_ptr(),
                );
                len += 1;
                trap::FS_FCloseFile(f);

                totallen += len;
                marker = base.add(totallen as usize);
            }

            i += 1;
            holdChar = holdChar.add((npcExtFNLen + 1) as usize);
        }
    }
}

/// `qboolean NPC_ParseParms( const char *NPCName, gentity_t *NPC )` (NPC_stats.c:983) — the
/// `scripts/NPCs.cfg` per-NPC token parser. Finds `NPCName`'s `{ … }` block inside the
/// concatenated [`NPCParms`] text and walks every key/value token, filling in the entity's
/// [`gNPCstats_t`] AI stats, the [`renderInfo_t`] head/torso clamp ranges, model scale, health,
/// team/class (via [`GetIDForString`] over [`TeamTable`]/[`ClassTable`]), sounds, starting
/// weapon + force powers (via [`WPTable`]/[`FPTable`]), and the per-blade saber parameters
/// (colors/lengths/radii via [`WP_SaberParseParms`]/[`TranslateSaberColor`]). Defaults are
/// installed up front, then the `!md3Model` tail registers the Ghoul2 model and precaches.
///
/// No oracle — it writes entity state across the opaque `gentity_t`/`gclient_t`/`gNPC_t`/
/// `gNPCstats_t` graph and fires engine traps ([`trap::LinkEntity`], `G_ModelIndex`,
/// `SetupGameGhoul2Model`, `NPC_Precache`), none of which the off-engine oracle harness can
/// satisfy (the `NPC_LoadParms`/`WP_SaberLoadParms` precedent).
///
/// Deviations (see `DEVIATIONS.md`): `SetupGameGhoul2Model` is an `extern` import of its ported
/// sibling; the tail call to `NPC_Precache` now resolves to the in-file port; `#ifdef _XBOX` blocks omitted (PC/dedicated
/// is the ABI target); libc `strcpy`/`strstr`/`atoi`/`strlen` declared as direct externs (the
/// `Q3_VM`-gated `bg_lib` shims are configured out of the native build); the commented-out C
/// (`NPC_BuildRandom`, `MoveTypeNameToEnum`, `FindItemForWeapon`/`RegisterItem`, dismember-prob
/// assignments, model-name `Q_strncpyz`s, the `cg.saberAnimLevelPending` SP-only line, the
/// flying-vehicle trace) is carried as Rust comments, not live code.
pub unsafe fn NPC_ParseParms(mut NPCName: *const c_char, NPC: *mut gentity_t) -> qboolean {
    let mut value: *const c_char = null_mut();
    let mut p: *const c_char;
    let mut n: c_int = 0;
    let mut f: f32 = 0.0;
    let mut patch: *mut c_char;
    let mut sound: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut playerModel: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut customSkin: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let ri = addr_of_mut!((*(*NPC).client).renderInfo);
    let stats: *mut crate::codemp::game::b_public_h::gNPCstats_t;
    let mut md3Model: qboolean = QTRUE;
    let mut surfOff: [c_char; 1024] = [0; 1024];
    let mut surfOn: [c_char; 1024] = [0; 1024];
    let mut parsingPlayer: qboolean = QFALSE;
    let mut playerMins: vec3_t = [0.0; 3];
    let mut playerMaxs: vec3_t = [0.0; 3];
    let mut npcSaber1: c_int = 0;
    let mut npcSaber2: c_int = 0;

    VectorSet(&mut playerMins, -15.0, -15.0, DEFAULT_MINS_2 as f32);
    VectorSet(&mut playerMaxs, 15.0, 15.0, DEFAULT_MAXS_2 as f32);

    strcpy(customSkin.as_mut_ptr(), c"default".as_ptr());
    if NPCName.is_null() || *NPCName == 0 {
        NPCName = c"Player".as_ptr();
    }

    if (*NPC).s.number == 0 && !(*NPC).client.is_null() {
        //player, only want certain data
        parsingPlayer = QTRUE;
    }

    if !(*NPC).NPC.is_null() {
        stats = addr_of_mut!((*(*NPC).NPC).stats);
        /*
        NPC->NPC->allWeaponOrder[0]	= WP_BRYAR_PISTOL;
        NPC->NPC->allWeaponOrder[1]	= WP_SABER;
        NPC->NPC->allWeaponOrder[2]	= WP_IMOD;
        NPC->NPC->allWeaponOrder[3]	= WP_SCAVENGER_RIFLE;
        NPC->NPC->allWeaponOrder[4]	= WP_TRICORDER;
        NPC->NPC->allWeaponOrder[6]	= WP_NONE;
        NPC->NPC->allWeaponOrder[6]	= WP_NONE;
        NPC->NPC->allWeaponOrder[7]	= WP_NONE;
        */
        // fill in defaults
        (*stats).aggression = 3;
        (*stats).aim = 3;
        (*stats).earshot = 1024.0;
        (*stats).evasion = 3;
        (*stats).hfov = 90;
        (*stats).intelligence = 3;
        (*stats).move_ = 3;
        (*stats).reactions = 3;
        (*stats).vfov = 60;
        (*stats).vigilance = 0.1f32;
        (*stats).visrange = 1024.0;

        (*stats).health = 0;

        (*stats).yawSpeed = 90.0;
        (*stats).walkSpeed = 90;
        (*stats).runSpeed = 300;
        (*stats).acceleration = 15; //Increase/descrease speed this much per frame (20fps)
    } else {
        stats = null_mut();
    }

    //Set defaults
    //FIXME: should probably put default torso and head models, but what about enemies
    //that don't have any- like Stasis?
    //Q_strncpyz( ri->headModelName,	DEFAULT_HEADMODEL,  sizeof(ri->headModelName),	qtrue);
    //Q_strncpyz( ri->torsoModelName, DEFAULT_TORSOMODEL, sizeof(ri->torsoModelName),	qtrue);
    //Q_strncpyz( ri->legsModelName,	DEFAULT_LEGSMODEL,  sizeof(ri->legsModelName),	qtrue);
    //FIXME: should we have one for weapon too?
    surfOff.fill(0);
    surfOn.fill(0);

    /*
    ri->headYawRangeLeft = 50;
    ri->headYawRangeRight = 50;
    ri->headPitchRangeUp = 40;
    ri->headPitchRangeDown = 50;
    ri->torsoYawRangeLeft = 60;
    ri->torsoYawRangeRight = 60;
    ri->torsoPitchRangeUp = 30;
    ri->torsoPitchRangeDown = 70;
    */

    (*ri).headYawRangeLeft = 80;
    (*ri).headYawRangeRight = 80;
    (*ri).headPitchRangeUp = 45;
    (*ri).headPitchRangeDown = 45;
    (*ri).torsoYawRangeLeft = 60;
    (*ri).torsoYawRangeRight = 60;
    (*ri).torsoPitchRangeUp = 30;
    (*ri).torsoPitchRangeDown = 50;

    VectorCopy(&playerMins, &mut (*NPC).r.mins);
    VectorCopy(&playerMaxs, &mut (*NPC).r.maxs);
    (*(*NPC).client).ps.crouchheight = CROUCH_MAXS_2;
    (*(*NPC).client).ps.standheight = DEFAULT_MAXS_2;

    //rwwFIXMEFIXME: ...
    /*
    NPC->client->moveType		= MT_RUNJUMP;

    NPC->client->dismemberProbHead = 100;
    NPC->client->dismemberProbArms = 100;
    NPC->client->dismemberProbHands = 100;
    NPC->client->dismemberProbWaist = 100;
    NPC->client->dismemberProbLegs = 100;

    NPC->s.modelScale[0] = NPC->s.modelScale[1] = NPC->s.modelScale[2] = 1.0f;
    */

    (*(*NPC).client).ps.customRGBA[0] = 255;
    (*(*NPC).client).ps.customRGBA[1] = 255;
    (*(*NPC).client).ps.customRGBA[2] = 255;
    (*(*NPC).client).ps.customRGBA[3] = 255;

    if Q_stricmp(c"random".as_ptr(), NPCName) == 0 {
        //Randomly assemble a starfleet guy
        //NPC_BuildRandom( NPC );
        Com_Printf("RANDOM NPC NOT SUPPORTED IN MP\n");
        return QFALSE;
    } else {
        let mut fp: c_int;

        p = addr_of!(NPCParms) as *const c_char;
        COM_BeginParseSession(addr_of!(NPCFile) as *const c_char);

        // look for the right NPC
        while !p.is_null() {
            let tok = COM_ParseExt(&mut p, QTRUE);
            if *tok == 0 {
                return QFALSE;
            }

            if Q_stricmp(tok, NPCName) == 0 {
                break;
            }

            SkipBracedSection(&mut p);
        }
        if p.is_null() {
            return QFALSE;
        }

        if BG_ParseLiteral(&mut p, c"{".as_ptr()) != QFALSE {
            return QFALSE;
        }

        // parse the NPC info block
        loop {
            let tok = COM_ParseExt(&mut p, QTRUE);
            let token: *const c_char = tok;
            if *token == 0 {
                Com_Printf(&format!(
                    "^1ERROR: unexpected EOF while parsing '{}'\n",
                    Sz(NPCName)
                ));
                return QFALSE;
            }

            if Q_stricmp(token, c"}".as_ptr()) == 0 {
                break;
            }
            //===MODEL PROPERTIES===========================================================
            // custom color
            if Q_stricmp(token, c"customRGBA".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if Q_stricmp(value, c"random".as_ptr()) == 0 {
                    (*(*NPC).client).ps.customRGBA[0] = Q_irand(0, 255);
                    (*(*NPC).client).ps.customRGBA[1] = Q_irand(0, 255);
                    (*(*NPC).client).ps.customRGBA[2] = Q_irand(0, 255);
                    (*(*NPC).client).ps.customRGBA[3] = 255;
                } else {
                    (*(*NPC).client).ps.customRGBA[0] = atoi(value);

                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        continue;
                    }
                    (*(*NPC).client).ps.customRGBA[1] = n;

                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        continue;
                    }
                    (*(*NPC).client).ps.customRGBA[2] = n;

                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        continue;
                    }
                    (*(*NPC).client).ps.customRGBA[3] = n;
                }
                continue;
            }

            // headmodel
            if Q_stricmp(token, c"headmodel".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }

                if Q_stricmp(c"none".as_ptr(), value) == 0 {
                    //Zero the head clamp range so the torso & legs don't lag behind
                    (*ri).headYawRangeLeft = 0;
                    (*ri).headYawRangeRight = 0;
                    (*ri).headPitchRangeUp = 0;
                    (*ri).headPitchRangeDown = 0;
                }
                continue;
            }

            // torsomodel
            if Q_stricmp(token, c"torsomodel".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }

                if Q_stricmp(c"none".as_ptr(), value) == 0 {
                    //Zero the torso clamp range so the legs don't lag behind
                    (*ri).torsoYawRangeLeft = 0;
                    (*ri).torsoYawRangeRight = 0;
                    (*ri).torsoPitchRangeUp = 0;
                    (*ri).torsoPitchRangeDown = 0;
                }
                continue;
            }

            // legsmodel
            if Q_stricmp(token, c"legsmodel".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                /*
                Q_strncpyz( ri->legsModelName, value, sizeof(ri->legsModelName), qtrue);
                //Need to do this here to get the right index
                G_ParseAnimFileSet( ri->legsModelName, ri->legsModelName, &ci->animFileIndex );
                */
                continue;
            }

            // playerModel
            if Q_stricmp(token, c"playerModel".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                Q_strncpyz(
                    playerModel.as_mut_ptr(),
                    value,
                    core::mem::size_of_val(&playerModel) as c_int,
                );
                md3Model = QFALSE;
                continue;
            }

            // customSkin
            if Q_stricmp(token, c"customSkin".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                Q_strncpyz(
                    customSkin.as_mut_ptr(),
                    value,
                    core::mem::size_of_val(&customSkin) as c_int,
                );
                continue;
            }

            // surfOff
            if Q_stricmp(token, c"surfOff".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if surfOff[0] != 0 {
                    Q_strcat(
                        surfOff.as_mut_ptr(),
                        core::mem::size_of_val(&surfOff) as c_int,
                        c",".as_ptr(),
                    );
                    Q_strcat(
                        surfOff.as_mut_ptr(),
                        core::mem::size_of_val(&surfOff) as c_int,
                        value,
                    );
                } else {
                    Q_strncpyz(
                        surfOff.as_mut_ptr(),
                        value,
                        core::mem::size_of_val(&surfOff) as c_int,
                    );
                }
                continue;
            }

            // surfOn
            if Q_stricmp(token, c"surfOn".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if surfOn[0] != 0 {
                    Q_strcat(
                        surfOn.as_mut_ptr(),
                        core::mem::size_of_val(&surfOn) as c_int,
                        c",".as_ptr(),
                    );
                    Q_strcat(
                        surfOn.as_mut_ptr(),
                        core::mem::size_of_val(&surfOn) as c_int,
                        value,
                    );
                } else {
                    Q_strncpyz(
                        surfOn.as_mut_ptr(),
                        value,
                        core::mem::size_of_val(&surfOn) as c_int,
                    );
                }
                continue;
            }

            //headYawRangeLeft
            if Q_stricmp(token, c"headYawRangeLeft".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).headYawRangeLeft = n;
                continue;
            }

            //headYawRangeRight
            if Q_stricmp(token, c"headYawRangeRight".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).headYawRangeRight = n;
                continue;
            }

            //headPitchRangeUp
            if Q_stricmp(token, c"headPitchRangeUp".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).headPitchRangeUp = n;
                continue;
            }

            //headPitchRangeDown
            if Q_stricmp(token, c"headPitchRangeDown".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).headPitchRangeDown = n;
                continue;
            }

            //torsoYawRangeLeft
            if Q_stricmp(token, c"torsoYawRangeLeft".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).torsoYawRangeLeft = n;
                continue;
            }

            //torsoYawRangeRight
            if Q_stricmp(token, c"torsoYawRangeRight".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).torsoYawRangeRight = n;
                continue;
            }

            //torsoPitchRangeUp
            if Q_stricmp(token, c"torsoPitchRangeUp".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).torsoPitchRangeUp = n;
                continue;
            }

            //torsoPitchRangeDown
            if Q_stricmp(token, c"torsoPitchRangeDown".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                (*ri).torsoPitchRangeDown = n;
                continue;
            }

            // Uniform XYZ scale
            if Q_stricmp(token, c"scale".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if n != 100 {
                    (*(*NPC).client).ps.iModelScale = n; //so the client knows
                    if n >= 1024 {
                        Com_Printf("WARNING: MP does not support scaling up to or over 1024%\n");
                        n = 1023;
                    }

                    (*NPC).modelScale[0] = n as f32 / 100.0f32;
                    (*NPC).modelScale[1] = n as f32 / 100.0f32;
                    (*NPC).modelScale[2] = n as f32 / 100.0f32;
                }
                continue;
            }

            //X scale
            if Q_stricmp(token, c"scaleX".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if n != 100 {
                    Com_Printf("MP doesn't support xyz scaling, use 'scale'.\n");
                    //NPC->s.modelScale[0] = n/100.0f;
                }
                continue;
            }

            //Y scale
            if Q_stricmp(token, c"scaleY".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if n != 100 {
                    Com_Printf("MP doesn't support xyz scaling, use 'scale'.\n");
                    //NPC->s.modelScale[1] = n/100.0f;
                }
                continue;
            }

            //Z scale
            if Q_stricmp(token, c"scaleZ".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if n != 100 {
                    Com_Printf("MP doesn't support xyz scaling, use 'scale'.\n");
                    //	NPC->s.modelScale[2] = n/100.0f;
                }
                continue;
            }

            //===AI STATS=====================================================================
            if parsingPlayer == QFALSE {
                // aggression
                if Q_stricmp(token, c"aggression".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 1 || n > 5 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).aggression = n;
                    }
                    continue;
                }

                // aim
                if Q_stricmp(token, c"aim".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 1 || n > 5 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).aim = n;
                    }
                    continue;
                }

                // earshot
                if Q_stricmp(token, c"earshot".as_ptr()) == 0 {
                    if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if f < 0.0f32 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).earshot = f;
                    }
                    continue;
                }

                // evasion
                if Q_stricmp(token, c"evasion".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 1 || n > 5 {
                        Com_Printf(&format!(
                            "^3WARNING: bad {} in NPC '{}'\n",
                            Sz(token),
                            Sz(NPCName)
                        ));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).evasion = n;
                    }
                    continue;
                }

                // hfov
                if Q_stricmp(token, c"hfov".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 30 || n > 180 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).hfov = n; // / 2;	//FIXME: Why was this being done?!
                    }
                    continue;
                }

                // intelligence
                if Q_stricmp(token, c"intelligence".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 1 || n > 5 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).intelligence = n;
                    }
                    continue;
                }

                // move
                if Q_stricmp(token, c"move".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 1 || n > 5 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).move_ = n;
                    }
                    continue;
                }

                // reactions
                if Q_stricmp(token, c"reactions".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 1 || n > 5 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).reactions = n;
                    }
                    continue;
                }

                // shootDistance
                if Q_stricmp(token, c"shootDistance".as_ptr()) == 0 {
                    if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if f < 0.0f32 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).shootDistance = f;
                    }
                    continue;
                }

                // vfov
                if Q_stricmp(token, c"vfov".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 30 || n > 180 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).vfov = n / 2;
                    }
                    continue;
                }

                // vigilance
                if Q_stricmp(token, c"vigilance".as_ptr()) == 0 {
                    if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if f < 0.0f32 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).vigilance = f;
                    }
                    continue;
                }

                // visrange
                if Q_stricmp(token, c"visrange".as_ptr()) == 0 {
                    if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if f < 0.0f32 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).visrange = f;
                    }
                    continue;
                }

                // race
                //		if ( !Q_stricmp( token, "race" ) )
                //		{
                //			if ( COM_ParseString( &p, &value ) )
                //			{
                //				continue;
                //			}
                //			NPC->client->race = TranslateRaceName(value);
                //			continue;
                //		}

                // rank
                if Q_stricmp(token, c"rank".as_ptr()) == 0 {
                    if COM_ParseString(&mut p, &mut value) != QFALSE {
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*(*NPC).NPC).rank = TranslateRankName(value);
                    }
                    continue;
                }
            }

            // health
            if Q_stricmp(token, c"health".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!(
                        "^3WARNING: bad {} in NPC '{}'\n",
                        Sz(token),
                        Sz(NPCName)
                    ));
                    continue;
                }
                if !(*NPC).NPC.is_null() {
                    (*stats).health = n;
                } else if parsingPlayer != QFALSE {
                    (*(*NPC).client).pers.maxHealth = n;
                    (*(*NPC).client).ps.stats[STAT_MAX_HEALTH as usize] = n;
                }
                continue;
            }

            // fullName
            if Q_stricmp(token, c"fullName".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                (*NPC).fullName = G_NewString(value);
                continue;
            }

            // playerTeam
            if Q_stricmp(token, c"playerTeam".as_ptr()) == 0 {
                let mut tk: [c_char; 4096] = [0; 4096]; //rww - hackilicious!

                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                Com_sprintf(
                    tk.as_mut_ptr(),
                    core::mem::size_of_val(&tk) as c_int,
                    format_args!("NPC{}", Sz(token)),
                );
                let team =
                    GetIDForString(addr_of!(TeamTable) as *const stringID_table_t, tk.as_ptr()); //TranslateTeamName(value);
                (*(*NPC).client).playerTeam = team as npcteam_t;
                (*NPC).s.teamowner = team;
                continue;
            }

            // enemyTeam
            if Q_stricmp(token, c"enemyTeam".as_ptr()) == 0 {
                let mut tk: [c_char; 4096] = [0; 4096]; //rww - hackilicious!

                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                Com_sprintf(
                    tk.as_mut_ptr(),
                    core::mem::size_of_val(&tk) as c_int,
                    format_args!("NPC{}", Sz(token)),
                );
                (*(*NPC).client).enemyTeam =
                    GetIDForString(addr_of!(TeamTable) as *const stringID_table_t, tk.as_ptr())
                        as npcteam_t; //TranslateTeamName(value);
                continue;
            }

            // class
            if Q_stricmp(token, c"class".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                (*(*NPC).client).NPC_class =
                    GetIDForString(addr_of!(ClassTable) as *const stringID_table_t, value)
                        as class_t;
                (*NPC).s.NPC_class = (*(*NPC).client).NPC_class; //we actually only need this value now, but at the moment I don't feel like changing the 200+ references to client->NPC_class.

                // No md3's for vehicles.
                if (*(*NPC).client).NPC_class == CLASS_VEHICLE {
                    if (*NPC).m_pVehicle.is_null() {
                        //you didn't spawn this guy right!
                        Com_Printf(&format!("^1ERROR: Tried to spawn a vehicle NPC ({}) without using NPC_Vehicle or 'NPC spawn vehicle <vehiclename>'!!!  Bad, bad, bad!  Shame on you!\n", Sz(NPCName)));
                        return QFALSE;
                    }
                    md3Model = QFALSE;
                }

                continue;
            }

            // dismemberment probability for head
            if Q_stricmp(token, c"dismemberProbHead".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if !(*NPC).NPC.is_null() {
                    //	NPC->client->dismemberProbHead = n;
                    //rwwFIXMEFIXME: support for this?
                }
                continue;
            }

            // dismemberment probability for arms
            if Q_stricmp(token, c"dismemberProbArms".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if !(*NPC).NPC.is_null() {
                    //	NPC->client->dismemberProbArms = n;
                }
                continue;
            }

            // dismemberment probability for hands
            if Q_stricmp(token, c"dismemberProbHands".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if !(*NPC).NPC.is_null() {
                    //	NPC->client->dismemberProbHands = n;
                }
                continue;
            }

            // dismemberment probability for waist
            if Q_stricmp(token, c"dismemberProbWaist".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if !(*NPC).NPC.is_null() {
                    //	NPC->client->dismemberProbWaist = n;
                }
                continue;
            }

            // dismemberment probability for legs
            if Q_stricmp(token, c"dismemberProbLegs".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                if n < 0 {
                    Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                    continue;
                }
                if !(*NPC).NPC.is_null() {
                    //	NPC->client->dismemberProbLegs = n;
                }
                continue;
            }

            //===MOVEMENT STATS============================================================

            if Q_stricmp(token, c"width".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    continue;
                }

                (*NPC).r.mins[0] = -n as f32;
                (*NPC).r.mins[1] = -n as f32;
                (*NPC).r.maxs[0] = n as f32;
                (*NPC).r.maxs[1] = n as f32;
                continue;
            }

            if Q_stricmp(token, c"height".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    continue;
                }
                if (*(*NPC).client).NPC_class == CLASS_VEHICLE
                    && !(*NPC).m_pVehicle.is_null()
                    && !(*(*NPC).m_pVehicle).m_pVehicleInfo.is_null()
                    && (*(*(*NPC).m_pVehicle).m_pVehicleInfo).r#type == VH_FIGHTER
                {
                    //a flying vehicle's origin must be centered in bbox and it should spawn on the ground
                    //trace_t		tr;
                    //vec3_t		bottom;
                    //float		adjust = 32.0f;
                    // C chains `maxs[2] = standheight = (n/2.0f)` through the int `standheight`,
                    // so maxs[2] receives the *truncated* value — preserve that.
                    (*(*NPC).client).ps.standheight = (n as f32 / 2.0f32) as c_int;
                    (*NPC).r.maxs[2] = (*(*NPC).client).ps.standheight as f32;
                    (*NPC).r.mins[2] = -(*NPC).r.maxs[2];
                    (*NPC).s.origin[2] += (DEFAULT_MINS_2 as f32 - (*NPC).r.mins[2]) + 0.125f32;
                    VectorCopy(&(*NPC).s.origin, &mut (*(*NPC).client).ps.origin);
                    VectorCopy(&(*NPC).s.origin, &mut (*NPC).r.currentOrigin);
                    let origin = (*NPC).s.origin;
                    G_SetOrigin(NPC, &origin);
                    trap::LinkEntity(NPC);
                    //now trace down
                    /*
                    VectorCopy( NPC->s.origin, bottom );
                    bottom[2] -= adjust;
                    trap_Trace( &tr, NPC->s.origin, NPC->r.mins, NPC->r.maxs, bottom, NPC->s.number, MASK_NPCSOLID );
                    if ( !tr.allsolid && !tr.startsolid )
                    {
                        G_SetOrigin( NPC, tr.endpos );
                        trap_LinkEntity(NPC);
                    }
                    */
                } else {
                    (*NPC).r.mins[2] = DEFAULT_MINS_2 as f32; //Cannot change
                    (*NPC).r.maxs[2] = (n + DEFAULT_MINS_2) as f32;
                    (*(*NPC).client).ps.standheight = n + DEFAULT_MINS_2;
                }
                (*NPC).radius = n as f32;
                continue;
            }

            if Q_stricmp(token, c"crouchheight".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    continue;
                }

                (*(*NPC).client).ps.crouchheight = n + DEFAULT_MINS_2;
                continue;
            }

            if parsingPlayer == QFALSE {
                if Q_stricmp(token, c"movetype".as_ptr()) == 0 {
                    if COM_ParseString(&mut p, &mut value) != QFALSE {
                        continue;
                    }
                    if Q_stricmp(c"flyswim".as_ptr(), value) == 0 {
                        (*(*NPC).client).ps.eFlags2 |= EF2_FLYING;
                    }
                    //NPC->client->moveType = (movetype_t)MoveTypeNameToEnum(value);
                    //rwwFIXMEFIXME: support for movetypes
                    continue;
                }

                // yawSpeed
                if Q_stricmp(token, c"yawSpeed".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n <= 0 {
                        Com_Printf(&format!("bad {} in NPC '{}'\n", Sz(token), Sz(NPCName)));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).yawSpeed = n as f32;
                    }
                    continue;
                }

                // walkSpeed
                if Q_stricmp(token, c"walkSpeed".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 0 {
                        Com_Printf(&format!(
                            "^3WARNING: bad {} in NPC '{}'\n",
                            Sz(token),
                            Sz(NPCName)
                        ));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).walkSpeed = n;
                    }
                    continue;
                }

                //runSpeed
                if Q_stricmp(token, c"runSpeed".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 0 {
                        Com_Printf(&format!(
                            "^3WARNING: bad {} in NPC '{}'\n",
                            Sz(token),
                            Sz(NPCName)
                        ));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).runSpeed = n;
                    }
                    continue;
                }

                //acceleration
                if Q_stricmp(token, c"acceleration".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < 0 {
                        Com_Printf(&format!(
                            "^3WARNING: bad {} in NPC '{}'\n",
                            Sz(token),
                            Sz(NPCName)
                        ));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*stats).acceleration = n;
                    }
                    continue;
                }
                //sex - skip in MP
                if Q_stricmp(token, c"sex".as_ptr()) == 0 {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //===MISC===============================================================================
                // default behavior
                if Q_stricmp(token, c"behavior".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if n < BS_DEFAULT || n >= NUM_BSTATES {
                        Com_Printf(&format!(
                            "^3WARNING: bad {} in NPC '{}'\n",
                            Sz(token),
                            Sz(NPCName)
                        ));
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        (*(*NPC).NPC).defaultBehavior = n;
                    }
                    continue;
                }
            }

            // snd
            if Q_stricmp(token, c"snd".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if ((*NPC).r.svFlags & SVF_NO_BASIC_SOUNDS) == 0 {
                    //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                    Q_strncpyz(
                        sound.as_mut_ptr(),
                        value,
                        core::mem::size_of_val(&sound) as c_int,
                    );
                    patch = strstr(sound.as_ptr(), c"/".as_ptr());
                    if !patch.is_null() {
                        *patch = 0;
                    }
                    //	ci->customBasicSoundDir = G_NewString( sound );
                    //rwwFIXMEFIXME: Hooray for violating client server rules
                }
                continue;
            }

            // sndcombat
            if Q_stricmp(token, c"sndcombat".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if ((*NPC).r.svFlags & SVF_NO_COMBAT_SOUNDS) == 0 {
                    //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                    Q_strncpyz(
                        sound.as_mut_ptr(),
                        value,
                        core::mem::size_of_val(&sound) as c_int,
                    );
                    patch = strstr(sound.as_ptr(), c"/".as_ptr());
                    if !patch.is_null() {
                        *patch = 0;
                    }
                    //	ci->customCombatSoundDir = G_NewString( sound );
                }
                continue;
            }

            // sndextra
            if Q_stricmp(token, c"sndextra".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if ((*NPC).r.svFlags & SVF_NO_EXTRA_SOUNDS) == 0 {
                    //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                    Q_strncpyz(
                        sound.as_mut_ptr(),
                        value,
                        core::mem::size_of_val(&sound) as c_int,
                    );
                    patch = strstr(sound.as_ptr(), c"/".as_ptr());
                    if !patch.is_null() {
                        *patch = 0;
                    }
                    //	ci->customExtraSoundDir = G_NewString( sound );
                }
                continue;
            }

            // sndjedi
            if Q_stricmp(token, c"sndjedi".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if ((*NPC).r.svFlags & SVF_NO_EXTRA_SOUNDS) == 0 {
                    //FIXME: store this in some sound field or parse in the soundTable like the animTable...
                    Q_strncpyz(
                        sound.as_mut_ptr(),
                        value,
                        core::mem::size_of_val(&sound) as c_int,
                    );
                    patch = strstr(sound.as_ptr(), c"/".as_ptr());
                    if !patch.is_null() {
                        *patch = 0;
                    }
                    //ci->customJediSoundDir = G_NewString( sound );
                }
                continue;
            }

            //New NPC/jedi stats:
            //starting weapon
            if Q_stricmp(token, c"weapon".as_ptr()) == 0 {
                let weap: c_int;

                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                //FIXME: need to precache the weapon, too?  (in above func)
                weap = GetIDForString(addr_of!(WPTable) as *const stringID_table_t, value);
                if weap >= WP_NONE && weap <= WP_NUM_WEAPONS {
                    //*WP_BLASTER_PISTOL*/WP_SABER ) //?!
                    (*(*NPC).client).ps.weapon = weap;
                    (*(*NPC).client).ps.stats[STAT_WEAPONS as usize] |=
                        1 << (*(*NPC).client).ps.weapon;
                    if weap > WP_NONE {
                        //	RegisterItem( FindItemForWeapon( (weapon_t)(NPC->client->ps.weapon) ) );	//precache the weapon
                        (*(*NPC).client).ps.ammo
                            [weaponData[(*(*NPC).client).ps.weapon as usize].ammoIndex as usize] =
                            100; //FIXME: max ammo!
                    }
                }
                continue;
            }

            if parsingPlayer == QFALSE {
                //altFire
                if Q_stricmp(token, c"altFire".as_ptr()) == 0 {
                    if COM_ParseInt(&mut p, &mut n) != QFALSE {
                        SkipRestOfLine(&mut p);
                        continue;
                    }
                    if !(*NPC).NPC.is_null() {
                        if n != 0 {
                            (*(*NPC).NPC).scriptFlags |= SCF_ALT_FIRE;
                        }
                    }
                    continue;
                }
                //Other unique behaviors/numbers that are currently hardcoded?
            }

            //force powers
            fp = GetIDForString(addr_of!(FPTable) as *const stringID_table_t, token);
            if fp >= FP_FIRST && fp < NUM_FORCE_POWERS as c_int {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //FIXME: need to precache the fx, too?  (in above func)
                //cap
                if n > 5 {
                    n = 5;
                } else if n < 0 {
                    n = 0;
                }
                if n != 0 {
                    //set
                    (*(*NPC).client).ps.fd.forcePowersKnown |= 1 << fp;
                } else {
                    //clear
                    (*(*NPC).client).ps.fd.forcePowersKnown &= !(1 << fp);
                }
                (*(*NPC).client).ps.fd.forcePowerLevel[fp as usize] = n;
                continue;
            }

            //max force power
            if Q_stricmp(token, c"forcePowerMax".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                (*(*NPC).client).ps.fd.forcePowerMax = n;
                continue;
            }

            //force regen rate - default is 100ms
            if Q_stricmp(token, c"forceRegenRate".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //NPC->client->ps.forcePowerRegenRate = n;
                //rwwFIXMEFIXME: support this?
                continue;
            }

            //force regen amount - default is 1 (points per second)
            if Q_stricmp(token, c"forceRegenAmount".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //NPC->client->ps.forcePowerRegenAmount = n;
                //rwwFIXMEFIXME: support this?
                continue;
            }

            //have a sabers.cfg and just name your saber in your NPCs.cfg/ICARUS script
            //saber name
            if Q_stricmp(token, c"saber".as_ptr()) == 0 {
                let saberName: *mut c_char;

                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }

                saberName = BG_TempAlloc(4096) as *mut c_char; //G_NewString( value );
                strcpy(saberName, value);

                WP_SaberParseParms(saberName, addr_of_mut!((*(*NPC).client).saber[0]));
                npcSaber1 = G_ModelIndex(&format!("@{}", Sz(saberName)));

                BG_TempFree(4096);
                continue;
            }

            //second saber name
            if Q_stricmp(token, c"saber2".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }

                if (*(*NPC).client).saber[0].saberFlags & SFL_TWO_HANDED == 0 {
                    //can't use a second saber if first one is a two-handed saber...?
                    let saberName: *mut c_char = BG_TempAlloc(4096) as *mut c_char; //G_NewString( value );
                    strcpy(saberName, value);

                    WP_SaberParseParms(saberName, addr_of_mut!((*(*NPC).client).saber[1]));
                    if (*(*NPC).client).saber[1].saberFlags & SFL_TWO_HANDED != 0 {
                        //tsk tsk, can't use a twoHanded saber as second saber
                        WP_RemoveSaber((*(*NPC).client).saber.as_mut_ptr(), 1);
                    } else {
                        //NPC->client->ps.dualSabers = qtrue;
                        npcSaber2 = G_ModelIndex(&format!("@{}", Sz(saberName)));
                    }
                    BG_TempFree(4096);
                }
                continue;
            }

            // saberColor
            if Q_stricmp(token, c"saberColor".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    let color: saber_colors_t = TranslateSaberColor(value);
                    n = 0;
                    while n < MAX_BLADES as c_int {
                        (*(*NPC).client).saber[0].blade[n as usize].color = color;
                        n += 1;
                    }
                }
                continue;
            }

            if Q_stricmp(token, c"saberColor2".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[0].blade[1].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saberColor3".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[0].blade[2].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saberColor4".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[0].blade[3].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saberColor5".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[0].blade[4].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saberColor6".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[0].blade[5].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saberColor7".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[0].blade[6].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saberColor8".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[0].blade[7].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    let color: saber_colors_t = TranslateSaberColor(value);
                    n = 0;
                    while n < MAX_BLADES as c_int {
                        (*(*NPC).client).saber[1].blade[n as usize].color = color;
                        n += 1;
                    }
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color2".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[1].blade[1].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color3".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[1].blade[2].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color4".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[1].blade[3].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color5".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[1].blade[4].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color6".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[1].blade[5].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color7".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[1].blade[6].color = TranslateSaberColor(value);
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Color8".as_ptr()) == 0 {
                if COM_ParseString(&mut p, &mut value) != QFALSE {
                    continue;
                }
                if !(*NPC).client.is_null() {
                    (*(*NPC).client).saber[1].blade[7].color = TranslateSaberColor(value);
                }
                continue;
            }

            //saber length
            if Q_stricmp(token, c"saberLength".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }

                n = 0;
                while n < MAX_BLADES as c_int {
                    (*(*NPC).client).saber[0].blade[n as usize].lengthMax = f;
                    n += 1;
                }
                continue;
            }

            if Q_stricmp(token, c"saberLength2".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[0].blade[1].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saberLength3".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[0].blade[2].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saberLength4".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[0].blade[3].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saberLength5".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[0].blade[4].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saberLength6".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[0].blade[5].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saberLength7".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[0].blade[6].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saberLength8".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[0].blade[7].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Length".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                n = 0;
                while n < MAX_BLADES as c_int {
                    (*(*NPC).client).saber[1].blade[n as usize].lengthMax = f;
                    n += 1;
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Length2".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[1].blade[1].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Length3".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[1].blade[2].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Length4".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[1].blade[3].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Length5".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[1].blade[4].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Length6".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[1].blade[5].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Length7".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[1].blade[6].lengthMax = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Length8".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 4.0f32 {
                    f = 4.0f32;
                }
                (*(*NPC).client).saber[1].blade[7].lengthMax = f;
                continue;
            }

            //saber radius
            if Q_stricmp(token, c"saberRadius".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                n = 0;
                while n < MAX_BLADES as c_int {
                    (*(*NPC).client).saber[0].blade[n as usize].radius = f;
                    n += 1;
                }
                continue;
            }

            if Q_stricmp(token, c"saberRadius2".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[0].blade[1].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saberRadius3".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[0].blade[2].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saberRadius4".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[0].blade[3].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saberRadius5".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[0].blade[4].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saberRadius6".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[0].blade[5].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saberRadius7".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[0].blade[6].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saberRadius8".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[0].blade[7].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Radius".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                n = 0;
                while n < MAX_BLADES as c_int {
                    (*(*NPC).client).saber[1].blade[n as usize].radius = f;
                    n += 1;
                }
                continue;
            }

            if Q_stricmp(token, c"saber2Radius2".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[1].blade[1].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Radius3".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[1].blade[2].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Radius4".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[1].blade[3].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Radius5".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[1].blade[4].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Radius6".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[1].blade[5].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Radius7".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[1].blade[6].radius = f;
                continue;
            }

            if Q_stricmp(token, c"saber2Radius8".as_ptr()) == 0 {
                if COM_ParseFloat(&mut p, &mut f) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if f < 0.25f32 {
                    f = 0.25f32;
                }
                (*(*NPC).client).saber[1].blade[7].radius = f;
                continue;
            }

            //ADD:
            //saber sounds (on, off, loop)
            //loop sound (like Vader's breathing or droid bleeps, etc.)

            //starting saber style
            if Q_stricmp(token, c"saberStyle".as_ptr()) == 0 {
                if COM_ParseInt(&mut p, &mut n) != QFALSE {
                    SkipRestOfLine(&mut p);
                    continue;
                }
                //cap
                if n < 0 {
                    n = 0;
                } else if n > 5 {
                    n = 5;
                }
                (*(*NPC).client).ps.fd.saberAnimLevel = n;
                /*
                if ( parsingPlayer )
                {
                    cg.saberAnimLevelPending = n;
                }
                */
                continue;
            }

            if parsingPlayer == QFALSE {
                Com_Printf(&format!(
                    "WARNING: unknown keyword '{}' while parsing '{}'\n",
                    Sz(token),
                    Sz(NPCName)
                ));
            }
            SkipRestOfLine(&mut p);
        }
    }

    /*
    Ghoul2 Insert Start
    */
    if md3Model == QFALSE {
        let mut setTypeBack: qboolean = QFALSE;

        if npcSaber1 == 0 {
            //use "kyle" for a default then
            npcSaber1 = G_ModelIndex("@Kyle");
            WP_SaberParseParms(c"Kyle".as_ptr(), addr_of_mut!((*(*NPC).client).saber[0]));
        }

        (*NPC).s.npcSaber1 = npcSaber1;
        (*NPC).s.npcSaber2 = npcSaber2;

        if customSkin[0] == 0 {
            strcpy(customSkin.as_mut_ptr(), c"default".as_ptr());
        }

        if !(*NPC).client.is_null() && (*(*NPC).client).NPC_class == CLASS_VEHICLE {
            //vehicles want their names fed in as models
            //we put the $ in front to indicate a name and not a model
            strcpy(
                playerModel.as_mut_ptr(),
                va(format_args!("${}", Sz(NPCName))) as *const c_char,
            );
        }
        // #ifdef _XBOX … ModelMem.SetNPCMode(true) … (omitted — PC/dedicated build)
        SetupGameGhoul2Model(NPC, playerModel.as_mut_ptr(), customSkin.as_mut_ptr());
        // #ifdef _XBOX … ModelMem.SetNPCMode(false) … (omitted — PC/dedicated build)

        if (*NPC).NPC_type.is_null() {
            //just do this for now so NPC_Precache can see the name.
            (*NPC).NPC_type = NPCName as *mut c_char;
            setTypeBack = QTRUE;
        }

        NPC_Precache(NPC); //this will just soundindex some values for sounds on the client,

        if setTypeBack != QFALSE {
            //don't want this being set if we aren't ready yet.
            (*NPC).NPC_type = null_mut();
        }
    } else {
        Com_Printf("MD3 MODEL NPC'S ARE NOT SUPPORTED IN MP!\n");
        return QFALSE;
    }
    /*
    Ghoul2 Insert End
    */
    /*
    if(	NPCsPrecached )
    {//Spawning in after initial precache, our models are precached, we just need to set our clientInfo
        CG_RegisterClientModels( NPC->s.number );
        CG_RegisterNPCCustomSounds( ci );
        CG_RegisterNPCEffects( NPC->client->playerTeam );
    }
    */
    //rwwFIXMEFIXME: Do something here I guess to properly precache stuff.

    QTRUE
}

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::TranslateRankName;
    use crate::codemp::game::b_public_h::{
        RANK_CAPTAIN, RANK_CIVILIAN, RANK_COMMANDER, RANK_CREWMAN, RANK_ENSIGN, RANK_LT,
        RANK_LT_COMM, RANK_LT_JG,
    };
    use crate::oracle::jka_TranslateRankName;

    /// Parity: every rank-name string maps to the same `rank_t` as the authentic C,
    /// including the matched names, an unknown name (falls back to `RANK_CIVILIAN`),
    /// and a case-insensitivity check (`Q_stricmp`).
    #[test]
    fn translate_rank_name_matches_c() {
        unsafe {
            for &name in &[
                c"civilian".as_ptr(),
                c"crewman".as_ptr(),
                c"ensign".as_ptr(),
                c"ltjg".as_ptr(),
                c"lt".as_ptr(),
                c"ltcomm".as_ptr(),
                c"commander".as_ptr(),
                c"captain".as_ptr(),
                c"Captain".as_ptr(),
                c"CIVILIAN".as_ptr(),
                c"bogus".as_ptr(),
                c"".as_ptr(),
            ] {
                assert_eq!(TranslateRankName(name), jka_TranslateRankName(name));
            }

            // Spot-check the exact expected constants too.
            assert_eq!(TranslateRankName(c"civilian".as_ptr()), RANK_CIVILIAN);
            assert_eq!(TranslateRankName(c"crewman".as_ptr()), RANK_CREWMAN);
            assert_eq!(TranslateRankName(c"ensign".as_ptr()), RANK_ENSIGN);
            assert_eq!(TranslateRankName(c"ltjg".as_ptr()), RANK_LT_JG);
            assert_eq!(TranslateRankName(c"lt".as_ptr()), RANK_LT);
            assert_eq!(TranslateRankName(c"ltcomm".as_ptr()), RANK_LT_COMM);
            assert_eq!(TranslateRankName(c"commander".as_ptr()), RANK_COMMANDER);
            assert_eq!(TranslateRankName(c"captain".as_ptr()), RANK_CAPTAIN);
            assert_eq!(TranslateRankName(c"bogus".as_ptr()), RANK_CIVILIAN);
        }
    }
}
