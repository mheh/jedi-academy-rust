// cg_event.c -- handle entity events at snapshot or playerstate transitions

// this line must stay at top so the whole PCH thing works...
use super::cg_headers;
use super::cg_media;
use super::FxScheduler_h;
use crate::code::game::anims;

#[cfg(target_os = "xbox")]
use crate::code::client::fffx;

#[cfg(feature = "_IMMERSION")]
use crate::code::ff::ff;

use core::ffi::{c_int, c_char, c_void};

// extern functions
extern "C" {
    fn CG_TryPlayCustomSound(
        origin: *mut [f32; 3],
        entityNum: c_int,
        channel: c_int,
        soundName: *const c_char,
        customSoundSet: c_int,
    ) -> u32;
    fn FX_KothosBeam(start: *mut [f32; 3], end: *mut [f32; 3]);
}

//==========================================================================

pub fn CG_IsFemale(infostring: *const c_char) -> u32 {
    let mut sex: *mut c_char;

    sex = Info_ValueForKey(infostring, "s\0".as_ptr() as *const c_char);
    if unsafe { *sex } as c_char == b'f' as c_char || unsafe { *sex } as c_char == b'F' as c_char {
        return 1; // qtrue
    }
    return 0; // qfalse
}

pub fn CG_PlaceString(rank: c_int) -> *const c_char {
    static mut str: [c_char; 64] = [0; 64];
    let mut s: *const c_char;
    let mut t: *const c_char;
    let mut rank_mut = rank;

    if (rank_mut & 0x80000000) != 0 {
        rank_mut &= !0x80000000; // RANK_TIED_FLAG
        t = "Tied for \0".as_ptr() as *const c_char;
    } else {
        t = "\0".as_ptr() as *const c_char;
    }

    if rank_mut == 1 {
        s = "\x03341st\x0337\0".as_ptr() as *const c_char; // draw in blue
    } else if rank_mut == 2 {
        s = "\x03312nd\x0337\0".as_ptr() as *const c_char; // draw in red
    } else if rank_mut == 3 {
        s = "\x03333rd\x0337\0".as_ptr() as *const c_char; // draw in yellow
    } else if rank_mut == 11 {
        s = "11th\0".as_ptr() as *const c_char;
    } else if rank_mut == 12 {
        s = "12th\0".as_ptr() as *const c_char;
    } else if rank_mut == 13 {
        s = "13th\0".as_ptr() as *const c_char;
    } else if rank_mut % 10 == 1 {
        s = va("%ist\0".as_ptr() as *const c_char, rank_mut);
    } else if rank_mut % 10 == 2 {
        s = va("%ind\0".as_ptr() as *const c_char, rank_mut);
    } else if rank_mut % 10 == 3 {
        s = va("%ird\0".as_ptr() as *const c_char, rank_mut);
    } else {
        s = va("%ith\0".as_ptr() as *const c_char, rank_mut);
    }

    Com_sprintf(
        unsafe { str.as_mut_ptr() as *mut c_char },
        core::mem::size_of_val(&str) as c_int,
        "%s%s\0".as_ptr() as *const c_char,
        t,
        s,
    );
    unsafe { str.as_ptr() as *const c_char }
}

/*
================
CG_ItemPickup

A new item was picked up this frame
================
*/
pub fn CG_ItemPickup(itemNum: c_int, bHadItem: u32) {
    unsafe {
        cg.itemPickup = itemNum;
        cg.itemPickupTime = cg.time;
        cg.itemPickupBlendTime = cg.time;

        if !bg_itemlist[itemNum as usize].classname.is_null()
            && !(*bg_itemlist[itemNum as usize].classname).is_null()
        {
            let mut text: [c_char; 1024] = [0; 1024];
            let mut data: [c_char; 1024] = [0; 1024];
            if cgi_SP_GetStringTextString(
                "SP_INGAME_PICKUPLINE\0".as_ptr() as *const c_char,
                text.as_mut_ptr(),
                core::mem::size_of_val(&text) as c_int,
            ) != 0
            {
                if cgi_SP_GetStringTextString(
                    va(
                        "SP_INGAME_%s\0".as_ptr() as *const c_char,
                        *bg_itemlist[itemNum as usize].classname,
                    ),
                    data.as_mut_ptr(),
                    core::mem::size_of_val(&data) as c_int,
                ) != 0
                {
                    //				Com_Printf("%s %s\n", text, data );
                    cgi_Cvar_Set(
                        "cg_WeaponPickupText\0".as_ptr() as *const c_char,
                        va("%s %s\n\0".as_ptr() as *const c_char, text.as_ptr(), data.as_ptr()),
                    );
                    cg.weaponPickupTextTime = cg.time + 5000;
                }
            }
        }

        // see if it should be the grabbed weapon
        if (*bg_itemlist[itemNum as usize]).giType == 3 {
            // IT_WEAPON
            let nCurWpn = cg.predicted_player_state.weapon;
            let nNewWpn = (*bg_itemlist[itemNum as usize]).giTag;

            if nCurWpn == 14 || bHadItem != 0 {
                // WP_SABER
                //never switch away from the saber!
                return;
            }

            // kef -- check cg_autoswitch...
            //
            // 0 == no switching
            // 1 == automatically switch to best SAFE weapon
            // 2 == automatically switch to best weapon, safe or otherwise
            //
            // NOTE: automatically switching to any weapon you pick up is stupid and annoying and we won't do it.
            //

            if nNewWpn == 14 {
                // WP_SABER
                //always switch to saber
                SetWeaponSelectTime();
                cg.weaponSelect = nNewWpn;
            } else if cg_autoswitch.integer == 0 {
                // don't switch
            } else if cg_autoswitch.integer == 1 {
                // safe switching
                if (nNewWpn > nCurWpn)
                    && (nNewWpn != 13) // WP_DET_PACK
                    && (nNewWpn != 8)  // WP_TRIP_MINE
                    && (nNewWpn != 9)  // WP_THERMAL
                    && (nNewWpn != 5)  // WP_ROCKET_LAUNCHER
                    && (nNewWpn != 4)
                {
                    // WP_CONCUSSION
                    // switch to new wpn
                    //				cg.weaponSelectTime = cg.time;
                    SetWeaponSelectTime();
                    cg.weaponSelect = nNewWpn;
                }
            } else if cg_autoswitch.integer == 2 {
                // best
                if nNewWpn > nCurWpn {
                    // switch to new wpn
                    //				cg.weaponSelectTime = cg.time;
                    SetWeaponSelectTime();
                    cg.weaponSelect = nNewWpn;
                }
            }
        }
    }
}

/*
===============
UseItem
===============
*/
extern "C" {
    fn CG_ToggleBinoculars();
    fn CG_ToggleLAGoggles();
}

pub fn UseItem(itemNum: c_int) {
    let mut cent: *mut centity_t;

    unsafe {
        cent = &mut cg_entities[(*cg.snap).ps.clientNum as usize];

        match itemNum {
            1 => {
                // INV_ELECTROBINOCULARS
                CG_ToggleBinoculars();
            }
            3 => {
                // INV_LIGHTAMP_GOGGLES
                CG_ToggleLAGoggles();
            }
            11 => {
                // INV_GOODIE_KEY
                if (*(*cent).gent).client != core::ptr::null_mut()
                    && (*(*(*cent).gent).client).ps.inventory[11] > 0
                {
                    (*(*(*cent).gent).client).ps.inventory[11] -= 1;
                }
            }
            12 => {
                // INV_SECURITY_KEY
                if (*(*cent).gent).client != core::ptr::null_mut()
                    && (*(*(*cent).gent).client).ps.inventory[12] > 0
                {
                    (*(*(*cent).gent).client).ps.inventory[12] -= 1;
                }
            }
            _ => {}
        }
    }
}

/*
===============
CG_UseForce
===============
*/
pub fn CG_UseForce(cent: *mut centity_t) {
    //FIXME: sound or graphic change or something?
    //actual force power action is on game/pm side
}

/*
===============
CG_UseItem
===============
*/
pub fn CG_UseItem(cent: *mut centity_t) {
    let mut itemNum: c_int;
    let mut es: *mut entityState_t;

    unsafe {
        es = &mut (*cent).currentState;

        itemNum = cg.inventorySelect;
        if itemNum < 0 || itemNum > 24 {
            // INV_MAX
            itemNum = 0;
        }

        // print a message if the local player
        if (*es).number == (*cg.snap).ps.clientNum {
            if itemNum == 0 {
                //			CG_CenterPrint( "No item to use", SCREEN_HEIGHT * 0.30, BIGCHAR_WIDTH );
            } else {
                //			item = BG_FindItemForHoldable( itemNum );
                //			CG_CenterPrint( va("Use %s", item->pickup_name), SCREEN_HEIGHT * 0.30, BIGCHAR_WIDTH );
            }
        }

        UseItem(itemNum);
    }
}

#[cfg(feature = "_IMMERSION")]
pub fn CG_ConfigForce(index: c_int, name: &mut *const c_char, channel: &mut c_int) -> u32 {
    let mut result = 0u32;
    let configstring = CG_ConfigString(100 + index); // CS_FORCES

    if !configstring.is_null() && unsafe { sscanf(configstring, "%d\0".as_ptr() as *const c_char, channel) == 1 } {
        result = 1;
    }

    if result != 0 {
        unsafe {
            *name = strchr(configstring, b',' as c_int);
            if !(*name).is_null() {
                *name = (*name).offset(1);
            }
            result = if *name as usize != 1 { 1 } else { 0 };
        }
    }

    return result;
}

/*
==============
CG_EntityEvent

An entity has an event value
==============
*/
macro_rules! DEBUGNAME {
    ($x:expr) => {
        if unsafe { cg_debugEvents.integer != 0 } {
            unsafe {
                let msg = concat!($x, "\n\0");
                CG_Printf(msg.as_ptr() as *const c_char);
            }
        }
    };
}

pub fn CG_EntityEvent(cent: *mut centity_t, position: *mut [f32; 3]) {
    let mut es: *mut entityState_t;
    let mut event: c_int;
    let mut axis: [[f32; 3]; 3] = [[0.0; 3]; 3];
    let mut s: *const c_char;
    let mut s2: *const c_char;
    let mut clientNum: c_int;
    //clientInfo_t	*ci;

    unsafe {
        es = &mut (*cent).currentState;
        event = (*es).event & !0x1f; // ~EV_EVENT_BITS

        if cg_debugEvents.integer != 0 {
            CG_Printf("ent:%3i  event:%3i \0".as_ptr() as *const c_char, (*es).number, event);
        }

        if event == 0 {
            DEBUGNAME!("ZEROEVENT");
            return;
        }

        if (*cent).gent.is_null() {
            //|| !cent->gent->client )
            return;
        }

        //ci = &cent->gent->client->clientInfo;
        clientNum = (*(*cent).gent).s.number;

        match event {
            //
            // movement generated events
            //
            /*	case EV_FOOTSTEP:
                    DEBUGNAME("EV_FOOTSTEP");
                    if (cg_footsteps.integer) {
                        if ( cent->gent && cent->gent->s.number == 0 && !cg.renderingThirdPerson )//!cg_thirdPerson.integer )
                        {//Everyone else has keyframed footsteps in animevents.cfg
            #ifdef _IMMERSION
                            int index = rand()&3;
                            cgi_S_StartSound (NULL, es->number, CHAN_BODY, cgs.media.footsteps[ FOOTSTEP_NORMAL ][index] );
                            cgi_FF_Start( cgs.media.footstepForces[ FOOTSTEP_NORMAL ][ index ], es->number );
            #else
                            cgi_S_StartSound (NULL, es->number, CHAN_BODY,
                                cgs.media.footsteps[ FOOTSTEP_NORMAL ][rand()&3] );
            #endif // _IMMERSION
                        }
                    }
                    break;
                case EV_FOOTSTEP_METAL:
                    DEBUGNAME("EV_FOOTSTEP_METAL");
                    if (cg_footsteps.integer)
                    {
                        if ( cent->gent && cent->gent->s.number == 0 && !cg.renderingThirdPerson )
                        {//Everyone else has keyframed footsteps in animevents.cfg
            #ifdef _IMMERSION
                            int index = rand()&3;
                            cgi_S_StartSound (NULL, es->number, CHAN_BODY, cgs.media.footsteps[ FOOTSTEP_METAL ][index] );
                            cgi_FF_Start( cgs.media.footstepForces[ FOOTSTEP_METAL ][ index ], es->number );
            #else
                            cgi_S_StartSound (NULL, es->number, CHAN_BODY, cgs.media.footsteps[ FOOTSTEP_METAL ][rand()&3] );
            #endif // _IMMERSION
                        }
                    }
                    break;
            */
            5 => {
                // EV_FOOTSPLASH
                DEBUGNAME!("EV_FOOTSPLASH");
                if cg_footsteps.integer != 0 {
                    #[cfg(feature = "_IMMERSION")]
                    {
                        let index = Q_irand(0, 3);
                        cgi_S_StartSound(
                            core::ptr::null_mut(),
                            (*es).number,
                            0, // CHAN_BODY
                            cgs.media.footsteps[3][index as usize], // FOOTSTEP_SPLASH
                        );
                        cgi_FF_Start(cgs.media.footstepForces[3][index as usize], (*es).number);
                    }
                    #[cfg(not(feature = "_IMMERSION"))]
                    {
                        cgi_S_StartSound(
                            core::ptr::null_mut(),
                            (*es).number,
                            0, // CHAN_BODY
                            cgs.media.footsteps[3][Q_irand(0, 3) as usize],
                        );
                    }
                }
            }
            6 => {
                // EV_FOOTWADE
                DEBUGNAME!("EV_FOOTWADE");
                if cg_footsteps.integer != 0 {
                    #[cfg(feature = "_IMMERSION")]
                    {
                        let index = Q_irand(0, 3);
                        cgi_S_StartSound(
                            core::ptr::null_mut(),
                            (*es).number,
                            0, // CHAN_BODY
                            cgs.media.footsteps[4][index as usize], // FOOTSTEP_WADE
                        );
                        cgi_FF_Start(cgs.media.footstepForces[4][index as usize], (*es).number);
                    }
                    #[cfg(not(feature = "_IMMERSION"))]
                    {
                        cgi_S_StartSound(
                            core::ptr::null_mut(),
                            (*es).number,
                            0, // CHAN_BODY
                            cgs.media.footsteps[4][Q_irand(0, 3) as usize],
                        );
                    }
                }
            }
            7 => {
                // EV_SWIM
                DEBUGNAME!("EV_SWIM");
                if cg_footsteps.integer != 0 {
                    #[cfg(feature = "_IMMERSION")]
                    {
                        let index = Q_irand(0, 3);
                        cgi_S_StartSound(
                            core::ptr::null_mut(),
                            (*es).number,
                            0, // CHAN_BODY
                            cgs.media.footsteps[5][index as usize], // FOOTSTEP_SWIM
                        );
                        cgi_FF_Start(cgs.media.footstepForces[5][index as usize], (*es).number);
                    }
                    #[cfg(not(feature = "_IMMERSION"))]
                    {
                        cgi_S_StartSound(
                            core::ptr::null_mut(),
                            (*es).number,
                            0, // CHAN_BODY
                            cgs.media.footsteps[5][Q_irand(0, 3) as usize],
                        );
                    }
                }
            }

            8 => {
                // EV_FALL_SHORT
                DEBUGNAME!("EV_FALL_SHORT");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.landSound,
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(cgs.media.landForce, (*es).number);
                }
                if clientNum == cg.predicted_player_state.clientNum {
                    // smooth landing z changes
                    cg.landChange = -8;
                    cg.landTime = cg.time;
                }
                //FIXME: maybe kick up some dust?
            }
            9 => {
                // EV_FALL_MEDIUM
                DEBUGNAME!("EV_FALL_MEDIUM");
                // use normal pain sound -
                if g_entities[(*es).number as usize].health <= 0 {
                    //dead
                    cgi_S_StartSound(
                        core::ptr::null_mut(),
                        (*es).number,
                        3, // CHAN_AUTO
                        cgs.media.landSound,
                    );
                    #[cfg(feature = "_IMMERSION")]
                    {
                        cgi_FF_Start(cgs.media.landForce, (*es).number);
                    }
                } else if g_entities[(*es).number as usize].s.weapon == 14 // WP_SABER
                    || (g_entities[(*es).number as usize].client != core::ptr::null_mut()
                        && ((*g_entities[(*es).number as usize].client).ps.forcePowersKnown
                            & (1 << 3)) != 0) // FP_LEVITATION
                {
                    //jedi or someone who has force jump (so probably took no damage)
                    CG_TryPlayCustomSound(
                        core::ptr::null_mut(),
                        (*es).number,
                        0, // CHAN_BODY
                        "*land1.wav\0".as_ptr() as *const c_char,
                        0, // CS_BASIC
                    );
                } else {
                    //still alive
                    CG_TryPlayCustomSound(
                        core::ptr::null_mut(),
                        (*es).number,
                        0, // CHAN_BODY
                        "*pain100.wav\0".as_ptr() as *const c_char,
                        0, // CS_BASIC
                    );
                }
                if clientNum == cg.predicted_player_state.clientNum {
                    // smooth landing z changes
                    cg.landChange = -16;
                    cg.landTime = cg.time;
                }
                #[cfg(target_os = "xbox")]
                {
                    cgi_FF_StartFX(fffx_FallingMedium);
                }
                //FIXME: maybe kick up some dust?
            }
            10 => {
                // EV_FALL_FAR
                DEBUGNAME!("EV_FALL_FAR");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    0, // CHAN_BODY
                    "*land1.wav\0".as_ptr() as *const c_char,
                    0, // CS_BASIC
                );
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.landSound,
                );
                (*cent).pe.painTime = cg.time; // don't play a pain sound right after this
                if clientNum == cg.predicted_player_state.clientNum {
                    // smooth landing z changes
                    cg.landChange = -24;
                    cg.landTime = cg.time;
                }
                #[cfg(target_os = "xbox")]
                {
                    cgi_FF_StartFX(fffx_FallingFar);
                }
                //FIXME: maybe kick up some dust?
            }

            11 | 12 | 13 | 14 => {
                // EV_STEP_4 | EV_STEP_8 | EV_STEP_12 | EV_STEP_16	// smooth out step up transitions
                DEBUGNAME!("EV_STEP");
                let mut oldStep: f32;
                let delta: c_int;
                let step: c_int;

                if clientNum != cg.predicted_player_state.clientNum {
                    break;
                }
                // if we are interpolating, we don't need to smooth steps
                if cg_timescale.value >= 1.0f32 {
                    break;
                }
                // check for stepping up before a previous step is completed
                delta = cg.time - cg.stepTime;
                if delta < 66 {
                    // STEP_TIME
                    oldStep = cg.stepChange * (66 - delta) as f32 / 66.0;
                } else {
                    oldStep = 0.0;
                }

                // add this amount
                step = 4 * (event - 11 + 1);
                cg.stepChange = oldStep + step as f32;
                if cg.stepChange > 22.0 {
                    // MAX_STEP_CHANGE
                    cg.stepChange = 22.0;
                }
                cg.stepTime = cg.time;
            }

            15 => {
                // EV_JUMP
                DEBUGNAME!("EV_JUMP");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    "*jump1.wav\0".as_ptr() as *const c_char,
                    0, // CS_BASIC
                ); //CHAN_VOICE
            }

            16 => {
                // EV_ROLL
                DEBUGNAME!("EV_ROLL");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    "*jump1.wav\0".as_ptr() as *const c_char,
                    0, // CS_BASIC
                ); //CHAN_VOICE
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    0, // CHAN_BODY
                    cgs.media.rollSound,
                ); //CHAN_AUTO
                   //FIXME: need some sort of body impact on ground sound and maybe kick up some dust?
            }

            17 => {
                // EV_LAVA_TOUCH
                DEBUGNAME!("EV_LAVA_TOUCH");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.lavaInSound,
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(cgs.media.watrInSound, (*es).number);
                }
            }

            18 => {
                // EV_LAVA_LEAVE
                DEBUGNAME!("EV_LAVA_LEAVE");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.lavaOutSound,
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(cgs.media.watrOutSound, (*es).number);
                }
            }

            19 => {
                // EV_LAVA_UNDER
                DEBUGNAME!("EV_LAVA_UNDER");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.lavaUnSound,
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(cgs.media.watrUnSound, (*es).number);
                }
            }

            20 => {
                // EV_WATER_TOUCH
                DEBUGNAME!("EV_WATER_TOUCH");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.watrInSound,
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(cgs.media.watrInSound, (*es).number);
                }
            }

            21 => {
                // EV_WATER_LEAVE
                DEBUGNAME!("EV_WATER_LEAVE");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.watrOutSound,
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(cgs.media.watrOutSound, (*es).number);
                }
            }

            22 => {
                // EV_WATER_UNDER
                DEBUGNAME!("EV_WATER_UNDER");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.watrUnSound,
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(cgs.media.watrUnSound, (*es).number);
                }
            }

            23 => {
                // EV_WATER_CLEAR
                DEBUGNAME!("EV_WATER_CLEAR");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    "*gasp.wav\0".as_ptr() as *const c_char,
                    0, // CS_BASIC
                );
            }

            24 | 25 => {
                // EV_WATER_GURP1 | EV_WATER_GURP2
                DEBUGNAME!("EV_WATER_GURPx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    va(
                        "*gurp%d.wav\0".as_ptr() as *const c_char,
                        event - 24 + 1,
                    ),
                    0, // CS_BASIC
                );
            }

            26 => {
                // EV_WATER_DROWN
                DEBUGNAME!("EV_WATER_DROWN");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    "*drown.wav\0".as_ptr() as *const c_char,
                    0, // CS_BASIC
                );
            }

            27 => {
                // EV_ITEM_PICKUP
                DEBUGNAME!("EV_ITEM_PICKUP");
                let mut item: *mut gitem_t;
                let mut index: c_int;
                let mut bHadItem: u32 = 0;

                index = (*es).eventParm; // player predicted

                if (index as i8) < 0 {
                    index = -(index as i8) as c_int;
                    bHadItem = 1; // qtrue
                }

                if index >= bg_numItems {
                    break;
                }
                item = &mut bg_itemlist[index as usize];
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgi_S_RegisterSound((*item).pickup_sound),
                );
                #[cfg(feature = "_IMMERSION")]
                {
                    cgi_FF_Start(
                        cgi_FF_Register((*item).pickup_force, 0), // FF_CHANNEL_TOUCH
                        (*es).number,
                    );
                }

                // show icon and name on status bar
                if (*es).number == (*cg.snap).ps.clientNum {
                    CG_ItemPickup(index, bHadItem);
                }
            }

            //
            // weapon events
            //
            28 => {
                // EV_NOAMMO
                DEBUGNAME!("EV_NOAMMO");
                //cgi_S_StartSound (NULL, es->number, CHAN_AUTO, cgs.media.noAmmoSound );
                if (*es).number == (*cg.snap).ps.clientNum {
                    CG_OutOfAmmoChange();
                }
            }
            29 => {
                // EV_CHANGE_WEAPON
                DEBUGNAME!("EV_CHANGE_WEAPON");
                if (*es).weapon == 14 {
                    // WP_SABER
                    /*
                    if ( !cent->gent || !cent->gent->client || (cent->currentState.saberInFlight == qfalse && cent->currentState.saberActive == qtrue) )
                    {
                        cgi_S_StartSound (NULL, es->number, CHAN_AUTO, cgi_S_RegisterSound( "sound/weapons/saber/saberoffquick.wav" ) );
                    }
                    */
                    if (*cent).gent != core::ptr::null_mut() && (*(*cent).gent).client != core::ptr::null_mut() {
                        //if ( cent->gent->client->ps.saberInFlight )
                        {
                            //if it's not in flight or lying around, turn it off!
                            (*cent).currentState.saberActive = 0; // qfalse
                        }
                    }
                }

                // FIXME: if it happens that you don't want the saber to play the switch sounds, feel free to modify this bit.
                if !weaponData[cg.weaponSelect as usize].selectSnd[0].is_null() {
                    // custom select sound
                    cgi_S_StartSound(
                        core::ptr::null_mut(),
                        (*es).number,
                        3, // CHAN_AUTO
                        cgi_S_RegisterSound(weaponData[cg.weaponSelect as usize].selectSnd),
                    );
                    #[cfg(feature = "_IMMERSION")]
                    {
                        cgi_FF_Start(
                            cgi_FF_Register(
                                weaponData[cg.weaponSelect as usize].selectFrc,
                                1, // FF_CHANNEL_WEAPON
                            ),
                            (*es).number,
                        );
                    }
                } else {
                    // generic sound
                    cgi_S_StartSound(
                        core::ptr::null_mut(),
                        (*es).number,
                        3, // CHAN_AUTO
                        cgs.media.selectSound,
                    );
                    #[cfg(feature = "_IMMERSION")]
                    {
                        cgi_FF_Start(cgs.media.selectForce, (*es).number);
                    }
                }
            }

            30 => {
                // EV_FIRE_WEAPON
                DEBUGNAME!("EV_FIRE_WEAPON");
                CG_FireWeapon(cent, 0); // qfalse
            }

            31 => {
                // EV_ALT_FIRE
                DEBUGNAME!("EV_ALT_FIRE");
                CG_FireWeapon(cent, 1); // qtrue
            }

            32 => {
                // EV_DISRUPTOR_MAIN_SHOT
                DEBUGNAME!("EV_DISRUPTOR_MAIN_SHOT");
                FX_DisruptorMainShot((*cent).currentState.origin2.as_mut_ptr(), (*cent).lerpOrigin.as_mut_ptr());
            }

            33 => {
                // EV_DISRUPTOR_SNIPER_SHOT
                DEBUGNAME!("EV_DISRUPTOR_SNIPER_SHOT");
                FX_DisruptorAltShot(
                    (*cent).currentState.origin2.as_mut_ptr(),
                    (*cent).lerpOrigin.as_mut_ptr(),
                    (*(*cent).gent).alt_fire,
                );
            }

            34 => {
                // EV_DISRUPTOR_SNIPER_MISS
                DEBUGNAME!("EV_DISRUPTOR_SNIPER_MISS");
                FX_DisruptorAltMiss((*cent).lerpOrigin.as_mut_ptr(), (*(*cent).gent).pos1.as_mut_ptr());
            }

            35 => {
                // EV_DEMP2_ALT_IMPACT
                FX_DEMP2_AltDetonate((*cent).lerpOrigin.as_mut_ptr(), (*es).eventParm);
            }

            36 => {
                // EV_CONC_ALT_SHOT
                DEBUGNAME!("EV_CONC_ALT_SHOT");
                FX_ConcAltShot(
                    (*cent).currentState.origin2.as_mut_ptr(),
                    (*cent).lerpOrigin.as_mut_ptr(),
                );
            }

            37 => {
                // EV_CONC_ALT_MISS
                DEBUGNAME!("EV_CONC_ALT_MISS");
                FX_ConcAltMiss((*cent).lerpOrigin.as_mut_ptr(), (*(*cent).gent).pos1.as_mut_ptr());
            }

            //	case EV_POWERUP_SEEKER_FIRE:
            //		DEBUGNAME("EV_POWERUP_SEEKER_FIRE");
            //		CG_FireSeeker( cent );
            //		break;

            38 => {
                // EV_POWERUP_BATTLESUIT
                DEBUGNAME!("EV_POWERUP_BATTLESUIT");
                if (*es).number == (*cg.snap).ps.clientNum {
                    cg.powerupActive = 3; // PW_BATTLESUIT
                    cg.powerupTime = cg.time;
                }
                //cgi_S_StartSound (NULL, es->number, CHAN_ITEM, cgs.media.invulnoProtectSound );
            }

            39 => {
                // EV_KOTHOS_BEAM
                DEBUGNAME!("EV_KOTHOS_BEAM");
                if Q_irand(0, 1) != 0 {
                    FX_KothosBeam(
                        (*cg_entities[(*cent).currentState.otherEntityNum as usize].gent)
                            .client
                            .as_mut()
                            .unwrap()
                            .renderInfo
                            .handRPoint
                            .as_mut_ptr(),
                        cg_entities[(*cent).currentState.otherEntityNum2 as usize]
                            .lerpOrigin
                            .as_mut_ptr(),
                    );
                } else {
                    FX_KothosBeam(
                        (*cg_entities[(*cent).currentState.otherEntityNum as usize].gent)
                            .client
                            .as_mut()
                            .unwrap()
                            .renderInfo
                            .handLPoint
                            .as_mut_ptr(),
                        cg_entities[(*cent).currentState.otherEntityNum2 as usize]
                            .lerpOrigin
                            .as_mut_ptr(),
                    );
                }
            }
            //=================================================================

            //
            // other events
            //
            40 => {
                // EV_REPLICATOR
                DEBUGNAME!("EV_REPLICATOR");
                //		FX_Replicator( cent, position );
            }

            41 => {
                // EV_BATTERIES_CHARGED
                cg.batteryChargeTime = cg.time + 3000;
                cgi_S_StartSound(
                    vec3_origin.as_mut_ptr(),
                    (*es).number,
                    3, // CHAN_AUTO
                    cgs.media.batteryChargeSound,
                );
            }

            42 => {
                // EV_DISINTEGRATION
                DEBUGNAME!("EV_DISINTEGRATION");
                let mut makeNotSolid: u32 = 0; // qfalse
                let disintPW = (*es).eventParm as c_int;
                let mut disintEffect: c_int = 0;
                let mut disintLength: c_int = 0;
                let mut disintSound1: u32 = 0; // qhandle_t
                let mut disintSound2: u32 = 0;
                let mut disintSound3: u32 = 0;

                match disintPW {
                    18 => {
                        // PW_DISRUPTION // sniper rifle
                        disintEffect = 64; //ef_DISINTEGRATION//ef_
                        disintSound1 = cgs.media.disintegrateSound; //with scream
                        disintSound2 = cgs.media.disintegrate2Sound; //no scream
                        disintSound3 = cgs.media.disintegrate3Sound; //with inhuman scream
                        disintLength = 2000;
                        makeNotSolid = 1; // qtrue
                    }
                    /*			case PW_SHOCKED:// arc welder
                        disintEffect = EF_DISINT_1;//ef_
                        disintSound1 = NULL;//with scream
                        disintSound2 = NULL;//no scream
                        disintSound3 = NULL;//with inhuman scream
                        disintLength = 4000;
                        break;
                    */
                    _ => {
                        break;
                    }
                }

                if (*cent).gent != core::ptr::null_mut() && (*(*cent).gent).owner != core::ptr::null_mut() {
                    (*(*(*cent).gent).owner).fx_time = cg.time;
                    if (*(*(*cent).gent).owner).client != core::ptr::null_mut() {
                        if disintSound1 != 0 && disintSound2 != 0 {
                            //play an extra sound
                            /*
                            if ( cent->gent->owner->client->playerTeam == TEAM_STARFLEET ||
                                    cent->gent->owner->client->playerTeam == TEAM_SCAVENGERS ||
                                    cent->gent->owner->client->playerTeam == TEAM_MALON ||
                                    cent->gent->owner->client->playerTeam == TEAM_IMPERIAL ||
                                    cent->gent->owner->client->playerTeam == TEAM_HIROGEN ||
                                    cent->gent->owner->client->playerTeam == TEAM_DISGUISE ||
                                    cent->gent->owner->client->playerTeam == TEAM_KLINGON )
                            */
                            // listed all the non-humanoids, because there's a lot more humanoids
                            let npc_class = (*(*(*cent).gent).owner).client.as_mut().unwrap().NPC_class;
                            if npc_class != 9 // CLASS_ATST
                                && npc_class != 8 // CLASS_GONK
                                && npc_class != 10 // CLASS_INTERROGATOR
                                && npc_class != 11 // CLASS_MARK1
                                && npc_class != 12 // CLASS_MARK2
                                && npc_class != 13 // CLASS_MOUSE
                                && npc_class != 14 // CLASS_PROBE
                                && npc_class != 15 // CLASS_PROTOCOL
                                && npc_class != 16 // CLASS_R2D2
                                && npc_class != 17 // CLASS_R5D2
                                && npc_class != 18 // CLASS_SEEKER
                                && npc_class != 19
                            {
                                // CLASS_SENTRY//Only the humanoids scream
                                cgi_S_StartSound(
                                    core::ptr::null_mut(),
                                    (*(*(*cent).gent).owner).s.number,
                                    1, // CHAN_VOICE
                                    disintSound1,
                                );
                            }
                            // no more forge or 8472
                            //	else if ( cent->gent->owner->client->playerTeam == TEAM_FORGE ||
                            //			cent->gent->owner->client->playerTeam == TEAM_8472 )
                            //	{
                            //		cgi_S_StartSound ( NULL, cent->gent->s.number, CHAN_VOICE, disintSound3 );
                            //	}
                            else {
                                cgi_S_StartSound(
                                    core::ptr::null_mut(),
                                    (*(*cent).gent).s.number,
                                    3, // CHAN_AUTO
                                    disintSound2,
                                );
                            }
                        }
                        (*(*(*cent).gent).owner).s.powerups |= 1 << disintPW;
                        (*(*(*cent).gent).owner).client.as_mut().unwrap().ps.powerups[disintPW as usize] =
                            cg.time + disintLength;

                        // Things that are being disintegrated should probably not be solid...
                        if makeNotSolid != 0
                            && (*(*(*cent).gent).owner).client.as_mut().unwrap().playerTeam != 0
                        {
                            // TEAM_NEUTRAL
                            (*(*(*cent).gent).owner).contents = 0; // CONTENTS_NONE
                        }
                    } else {
                        (*(*(*cent).gent).owner).s.eFlags = disintEffect as u32; //FIXME: |= ?
                        (*(*(*cent).gent).owner).delay = cg.time + disintLength;
                    }
                }
            }

            // This does not necessarily have to be from a grenade...
            43 => {
                // EV_GRENADE_BOUNCE
                DEBUGNAME!("EV_GRENADE_BOUNCE");
                CG_BounceEffect(cent, (*es).weapon, position, (*(*cent).gent).pos1.as_mut_ptr());
            }

            //
            // missile impacts
            //

            44 => {
                // EV_MISSILE_STICK
                DEBUGNAME!("EV_MISSILE_STICK");
                CG_MissileStick(cent, (*es).weapon, position);
            }

            45 => {
                // EV_MISSILE_HIT
                DEBUGNAME!("EV_MISSILE_HIT");
                CG_MissileHitPlayer(
                    cent,
                    (*es).weapon,
                    position,
                    (*(*cent).gent).pos1.as_mut_ptr(),
                    (*(*cent).gent).alt_fire,
                );
            }

            46 => {
                // EV_MISSILE_MISS
                DEBUGNAME!("EV_MISSILE_MISS");
                CG_MissileHitWall(
                    cent,
                    (*es).weapon,
                    position,
                    (*(*cent).gent).pos1.as_mut_ptr(),
                    (*(*cent).gent).alt_fire,
                );
            }

            47 => {
                // EV_BMODEL_SOUND
                DEBUGNAME!("EV_BMODEL_SOUND");
                cgi_S_StartSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    3, // CHAN_AUTO
                    (*es).eventParm as u32,
                );
            }

            48 => {
                // EV_GENERAL_SOUND
                DEBUGNAME!("EV_GENERAL_SOUND");
                if cgs.sound_precache[(*es).eventParm as usize] != 0 {
                    cgi_S_StartSound(
                        core::ptr::null_mut(),
                        (*es).number,
                        3, // CHAN_AUTO
                        cgs.sound_precache[(*es).eventParm as usize],
                    );
                } else {
                    s = CG_ConfigString(0x100 + (*es).eventParm); // CS_SOUNDS
                    CG_TryPlayCustomSound(
                        core::ptr::null_mut(),
                        (*es).number,
                        3, // CHAN_AUTO
                        s,
                        0, // CS_BASIC
                    );
                }
            }

            49 => {
                // EV_GLOBAL_SOUND	// play from the player's head so it never diminishes
                DEBUGNAME!("EV_GLOBAL_SOUND");
                if cgs.sound_precache[(*es).eventParm as usize] != 0 {
                    cgi_S_StartSound(
                        core::ptr::null_mut(),
                        (*cg.snap).ps.clientNum,
                        3, // CHAN_AUTO
                        cgs.sound_precache[(*es).eventParm as usize],
                    );
                } else {
                    s = CG_ConfigString(0x100 + (*es).eventParm); // CS_SOUNDS
                    CG_TryPlayCustomSound(
                        core::ptr::null_mut(),
                        (*cg.snap).ps.clientNum,
                        3, // CHAN_AUTO
                        s,
                        0, // CS_BASIC
                    );
                }
            }

            #[cfg(feature = "_IMMERSION")]
            50 => {
                // EV_ENTITY_FORCE				// Plays force on entity
                DEBUGNAME!("EV_ENTITY_FORCE");
                if cgs.force_precache[(*es).eventParm as usize] == 0 {
                    let mut name: *const c_char = core::ptr::null();
                    let mut channel: c_int = 0;
                    if CG_ConfigForce((*es).eventParm, &mut name, &mut channel) != 0 {
                        cgs.force_precache[(*es).eventParm as usize] = cgi_FF_Register(name, channel);
                    }
                }
                cgi_FF_Start(cgs.force_precache[(*es).eventParm as usize], (*es).number);
            }

            #[cfg(feature = "_IMMERSION")]
            51 | 52 => {
                // EV_GLOBAL_FORCE | EV_AREA_FORCE
                DEBUGNAME!("EV_AREA_FORCE"); // Plays force for anyone
                if cgs.force_precache[(*es).eventParm as usize] == 0 {
                    let mut name: *const c_char = core::ptr::null();
                    let mut channel: c_int = 0;
                    if CG_ConfigForce((*es).eventParm, &mut name, &mut channel) != 0 {
                        cgs.force_precache[(*es).eventParm as usize] = cgi_FF_Register(name, channel);
                    }
                }
                cgi_FF_Start(cgs.force_precache[(*es).eventParm as usize], (*es).number);
            }

            #[cfg(feature = "_IMMERSION")]
            53 => {
                // EV_FORCE_STOP
                DEBUGNAME!("EV_FORCE_STOP");
                if (*es).eventParm < 0 {
                    cgi_FF_StopAll();
                } else if (*es).eventParm < 32 && cgs.force_precache[(*es).eventParm as usize] != 0 {
                    // MAX_FORCES
                    cgi_FF_Stop(cgs.force_precache[(*es).eventParm as usize], (*es).number);
                }
            }

            54 => {
                // EV_DRUGGED
                DEBUGNAME!("EV_DRUGGED");
                if (*cent).gent != core::ptr::null_mut()
                    && (*(*cent).gent).owner != core::ptr::null_mut()
                    && (*(*(*cent).gent).owner).s.number == 0
                {
                    // Only allow setting up the wonky vision on the player..do it for 10 seconds...must be synchronized with calcs done in cg_view.  Just search for cg.wonkyTime to find 'em.
                    cg.wonkyTime = cg.time + 10000;
                }
            }

            55 => {
                // EV_PAIN
                let mut snd: *const c_char;
                let health = (*es).eventParm;

                if (*cent).gent != core::ptr::null_mut()
                    && (*(*cent).gent).NPC != core::ptr::null_mut()
                    && ((*(*(*cent).gent).NPC).aiFlags & 0x00001000) != 0
                {
                    // NPCAI_DIE_ON_IMPACT
                    return;
                }
                //FIXME: don't do this if we're falling to our deaths...
                DEBUGNAME!("EV_PAIN");
                // don't do more than two pain sounds a second
                if cg.time - (*cent).pe.painTime < 500 {
                    return;
                }

                if health < 25 {
                    snd = "*pain100.wav\0".as_ptr() as *const c_char;
                } else if health < 50 {
                    snd = "*pain75.wav\0".as_ptr() as *const c_char;
                } else if health < 75 {
                    snd = "*pain50.wav\0".as_ptr() as *const c_char;
                } else {
                    snd = "*pain25.wav\0".as_ptr() as *const c_char;
                }
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    snd,
                    0, // CS_BASIC
                );

                // save pain time for programitic twitch animation
                (*cent).pe.painTime = cg.time;
                (*cent).pe.painDirection ^= 1;
            }

            56 | 57 | 58 => {
                // EV_DEATH1 | EV_DEATH2 | EV_DEATH3
                DEBUGNAME!("EV_DEATHx");
                /*
                if ( cent->gent && cent->gent->NPC && (cent->gent->NPC->aiFlags & NPCAI_DIE_ON_IMPACT) )
                {
                    return;
                }
                */
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*death%i.wav\0".as_ptr() as *const c_char,
                        event - 56 + 1,
                    ),
                    0, // CS_BASIC
                );
            }

            // Called by the FxRunner entity...usually for Environmental FX Events
            59 => {
                // EV_PLAY_EFFECT
                DEBUGNAME!("EV_PLAY_EFFECT");
                let portalEnt = !!(*es).isPortalEnt; //the fxrunner spawning this effect is within a skyportal, so only render this effect within that portal.

                s = CG_ConfigString(200 + (*es).eventParm); // CS_EFFECTS
                // Ghoul2 Insert Start
                if (*es).boltInfo != 0 {
                    let isRelative = !!(*es).weapon;
                    theFxScheduler.PlayEffect(
                        s,
                        (*cent).lerpOrigin.as_mut_ptr(),
                        axis.as_mut_ptr(),
                        (*es).boltInfo,
                        -1,
                        portalEnt,
                        (*es).loopSound,
                        isRelative,
                    ); //loopSound 0 = not looping, 1 for infinite, else duration
                } else {
                    VectorCopy((*(*cent).gent).pos3.as_ptr(), &mut axis[0]);
                    VectorCopy((*(*cent).gent).pos4.as_ptr(), &mut axis[1]);
                    CrossProduct(axis[0].as_ptr(), axis[1].as_ptr(), &mut axis[2]);

                    // the entNum the effect may be attached to
                    #[cfg(feature = "_IMMERSION")]
                    {
                        if (*es).saberActive != 0 {
                            theFxScheduler.PlayEffect(
                                s,
                                (*cent).lerpOrigin.as_mut_ptr(),
                                axis.as_mut_ptr(),
                                -1,
                                0, // FF_CLIENT( es->otherEntityNum ),
                                portalEnt,
                            );
                        } else if (*es).otherEntityNum != 0 {
                            theFxScheduler.PlayEffect(
                                s,
                                (*cent).lerpOrigin.as_mut_ptr(),
                                axis.as_mut_ptr(),
                                -1,
                                (*es).otherEntityNum as c_int,
                                portalEnt,
                            );
                        } else {
                            theFxScheduler.PlayEffect(
                                s,
                                (*cent).lerpOrigin.as_mut_ptr(),
                                axis.as_mut_ptr(),
                                -1,
                                -1,
                                portalEnt,
                            );
                        }
                    }
                    #[cfg(not(feature = "_IMMERSION"))]
                    {
                        if (*es).otherEntityNum != 0 {
                            theFxScheduler.PlayEffect(
                                s,
                                (*cent).lerpOrigin.as_mut_ptr(),
                                axis.as_mut_ptr(),
                                -1,
                                (*es).otherEntityNum as c_int,
                                portalEnt,
                            );
                        } else {
                            theFxScheduler.PlayEffect(
                                s,
                                (*cent).lerpOrigin.as_mut_ptr(),
                                axis.as_mut_ptr(),
                                -1,
                                -1,
                                portalEnt,
                            );
                        }
                    }
                }
                // Ghoul2 Insert End
            }

            // play an effect bolted onto a muzzle
            60 => {
                // EV_PLAY_MUZZLE_EFFECT
                DEBUGNAME!("EV_PLAY_MUZZLE_EFFECT");
                s = CG_ConfigString(200 + (*es).eventParm); // CS_EFFECTS

                theFxScheduler.PlayEffect_Muzzle(s, (*es).otherEntityNum);
            }

            61 => {
                // EV_STOP_EFFECT
                DEBUGNAME!("EV_STOP_EFFECT");
                let mut portalEnt = false;

                if (*es).isPortalEnt != 0 {
                    //the fxrunner spawning this effect is within a skyportal, so only render this effect within that portal.
                    portalEnt = true;
                }

                s = CG_ConfigString(200 + (*es).eventParm); // CS_EFFECTS
                if (*es).boltInfo != 0 {
                    theFxScheduler.StopEffect(s, (*es).boltInfo, portalEnt);
                }
            }

            62 => {
                // EV_TARGET_BEAM_DRAW
                DEBUGNAME!("EV_TARGET_BEAM_DRAW");
                if (*cent).gent != core::ptr::null_mut() {
                    s = CG_ConfigString(200 + (*es).eventParm); // CS_EFFECTS

                    if !s.is_null() && *s as c_char != 0 {
                        if (*(*cent).gent).delay != 0 {
                            s2 = CG_ConfigString(200 + (*(*cent).gent).delay as c_int); // CS_EFFECTS
                        } else {
                            s2 = core::ptr::null();
                        }

                        CG_DrawTargetBeam(
                            (*cent).lerpOrigin.as_mut_ptr(),
                            (*(*cent).gent).s.origin2.as_mut_ptr(),
                            (*(*cent).gent).pos1.as_mut_ptr(),
                            s,
                            s2,
                        );
                    }
                    /*			else
                        {
                            int gack = 0; // this is bad if it get's here
                        }
                    */
                }
            }

            63 | 64 | 65 => {
                // EV_ANGER1 | EV_ANGER2 | EV_ANGER3	//Say when acquire an enemy when didn't have one before
                DEBUGNAME!("EV_ANGERx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*anger%i.wav\0".as_ptr() as *const c_char,
                        event - 63 + 1,
                    ),
                    1, // CS_COMBAT
                );
            }

            66 | 67 | 68 => {
                // EV_VICTORY1 | EV_VICTORY2 | EV_VICTORY3	//Say when killed an enemy
                DEBUGNAME!("EV_VICTORYx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*victory%i.wav\0".as_ptr() as *const c_char,
                        event - 66 + 1,
                    ),
                    1, // CS_COMBAT
                );
            }

            69 | 70 | 71 => {
                // EV_CONFUSE1 | EV_CONFUSE2 | EV_CONFUSE3	//Say when confused
                DEBUGNAME!("EV_CONFUSEDx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*confuse%i.wav\0".as_ptr() as *const c_char,
                        event - 69 + 1,
                    ),
                    1, // CS_COMBAT
                );
            }

            72 | 73 | 74 => {
                // EV_PUSHED1 | EV_PUSHED2 | EV_PUSHED3	//Say when pushed
                DEBUGNAME!("EV_PUSHEDx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*pushed%i.wav\0".as_ptr() as *const c_char,
                        event - 72 + 1,
                    ),
                    1, // CS_COMBAT
                );
            }

            75 | 76 | 77 => {
                // EV_CHOKE1 | EV_CHOKE2 | EV_CHOKE3	//Say when choking
                DEBUGNAME!("EV_CHOKEx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*choke%i.wav\0".as_ptr() as *const c_char,
                        event - 75 + 1,
                    ),
                    1, // CS_COMBAT
                );
            }

            78 => {
                // EV_FFWARN	//Warn ally to stop shooting you
                DEBUGNAME!("EV_FFWARN");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    "*ffwarn.wav\0".as_ptr() as *const c_char,
                    1, // CS_COMBAT
                );
            }

            79 => {
                // EV_FFTURN	//Turn on ally after being shot by them
                DEBUGNAME!("EV_FFTURN");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    "*ffturn.wav\0".as_ptr() as *const c_char,
                    1, // CS_COMBAT
                );
            }

            //extra sounds for ST
            80 | 81 | 82 => {
                // EV_CHASE1 | EV_CHASE2 | EV_CHASE3
                DEBUGNAME!("EV_CHASEx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*chase%i.wav\0".as_ptr() as *const c_char,
                        event - 80 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            83 | 84 | 85 | 86 | 87 => {
                // EV_COVER1..EV_COVER5
                DEBUGNAME!("EV_COVERx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*cover%i.wav\0".as_ptr() as *const c_char,
                        event - 83 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            88 | 89 | 90 | 91 | 92 => {
                // EV_DETECTED1..EV_DETECTED5
                DEBUGNAME!("EV_DETECTEDx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*detected%i.wav\0".as_ptr() as *const c_char,
                        event - 88 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            93 | 94 | 95 | 96 => {
                // EV_GIVEUP1..EV_GIVEUP4
                DEBUGNAME!("EV_GIVEUPx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*giveup%i.wav\0".as_ptr() as *const c_char,
                        event - 93 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            97 | 98 => {
                // EV_LOOK1 | EV_LOOK2
                DEBUGNAME!("EV_LOOKx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*look%i.wav\0".as_ptr() as *const c_char,
                        event - 97 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            99 => {
                // EV_LOST1
                DEBUGNAME!("EV_LOST1");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    "*lost1.wav\0".as_ptr() as *const c_char,
                    2, // CS_EXTRA
                );
            }
            100 | 101 => {
                // EV_OUTFLANK1 | EV_OUTFLANK2
                DEBUGNAME!("EV_OUTFLANKx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*outflank%i.wav\0".as_ptr() as *const c_char,
                        event - 100 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            102 | 103 | 104 => {
                // EV_ESCAPING1 | EV_ESCAPING2 | EV_ESCAPING3
                DEBUGNAME!("EV_ESCAPINGx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*escaping%i.wav\0".as_ptr() as *const c_char,
                        event - 102 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            105 | 106 | 107 => {
                // EV_SIGHT1 | EV_SIGHT2 | EV_SIGHT3
                DEBUGNAME!("EV_SIGHTx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*sight%i.wav\0".as_ptr() as *const c_char,
                        event - 105 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            108 | 109 | 110 => {
                // EV_SOUND1 | EV_SOUND2 | EV_SOUND3
                DEBUGNAME!("EV_SOUNDx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*sound%i.wav\0".as_ptr() as *const c_char,
                        event - 108 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            111 | 112 | 113 | 114 | 115 => {
                // EV_SUSPICIOUS1..EV_SUSPICIOUS5
                DEBUGNAME!("EV_SUSPICIOUSx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*suspicious%i.wav\0".as_ptr() as *const c_char,
                        event - 111 + 1,
                    ),
                    2, // CS_EXTRA
                );
            }
            //extra sounds for Jedi
            116 | 117 | 118 => {
                // EV_COMBAT1 | EV_COMBAT2 | EV_COMBAT3
                DEBUGNAME!("EV_COMBATx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*combat%i.wav\0".as_ptr() as *const c_char,
                        event - 116 + 1,
                    ),
                    3, // CS_JEDI
                );
            }
            119 | 120 | 121 => {
                // EV_JDETECTED1 | EV_JDETECTED2 | EV_JDETECTED3
                DEBUGNAME!("EV_JDETECTEDx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*jdetected%i.wav\0".as_ptr() as *const c_char,
                        event - 119 + 1,
                    ),
                    3, // CS_JEDI
                );
            }
            122 | 123 | 124 => {
                // EV_TAUNT1 | EV_TAUNT2 | EV_TAUNT3
                DEBUGNAME!("EV_TAUNTx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*taunt%i.wav\0".as_ptr() as *const c_char,
                        event - 122 + 1,
                    ),
                    3, // CS_JEDI
                );
            }
            125 | 126 | 127 => {
                // EV_JCHASE1 | EV_JCHASE2 | EV_JCHASE3
                DEBUGNAME!("EV_JCHASEx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*jchase%i.wav\0".as_ptr() as *const c_char,
                        event - 125 + 1,
                    ),
                    3, // CS_JEDI
                );
            }
            128 | 129 | 130 => {
                // EV_JLOST1 | EV_JLOST2 | EV_JLOST3
                DEBUGNAME!("EV_JLOSTx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*jlost%i.wav\0".as_ptr() as *const c_char,
                        event - 128 + 1,
                    ),
                    3, // CS_JEDI
                );
            }
            131 | 132 | 133 => {
                // EV_DEFLECT1 | EV_DEFLECT2 | EV_DEFLECT3
                DEBUGNAME!("EV_DEFLECTx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*deflect%i.wav\0".as_ptr() as *const c_char,
                        event - 131 + 1,
                    ),
                    3, // CS_JEDI
                );
            }
            134 | 135 | 136 => {
                // EV_GLOAT1 | EV_GLOAT2 | EV_GLOAT3
                DEBUGNAME!("EV_GLOATx");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    va(
                        "*gloat%i.wav\0".as_ptr() as *const c_char,
                        event - 134 + 1,
                    ),
                    3, // CS_JEDI
                );
            }
            137 => {
                // EV_PUSHFAIL
                DEBUGNAME!("EV_PUSHFAIL");
                CG_TryPlayCustomSound(
                    core::ptr::null_mut(),
                    (*es).number,
                    1, // CHAN_VOICE
                    "*pushfail.wav\0".as_ptr() as *const c_char,
                    3, // CS_JEDI
                );
            }

            138 => {
                // EV_USE_FORCE
                DEBUGNAME!("EV_USE_FORCEITEM");
                CG_UseForce(cent);
            }

            139 => {
                // EV_USE_ITEM
                DEBUGNAME!("EV_USE_ITEM");
                CG_UseItem(cent);
            }

            140 => {
                // EV_USE_INV_BINOCULARS
                DEBUGNAME!("EV_USE_INV_BINOCULARS");
                UseItem(1); // INV_ELECTROBINOCULARS
            }

            141 => {
                // EV_USE_INV_BACTA
                DEBUGNAME!("EV_USE_INV_BACTA");
                UseItem(2); // INV_BACTA_CANISTER
            }

            142 => {
                // EV_USE_INV_SEEKER
                DEBUGNAME!("EV_USE_INV_SEEKER");
                UseItem(4); // INV_SEEKER
            }

            143 => {
                // EV_USE_INV_LIGHTAMP_GOGGLES
                DEBUGNAME!("EV_USE_INV_LIGHTAMP_GOGGLES");
                UseItem(3); // INV_LIGHTAMP_GOGGLES
            }

            144 => {
                // EV_USE_INV_SENTRY
                DEBUGNAME!("EV_USE_INV_SENTRY");
                UseItem(10); // INV_SENTRY
            }

            145 => {
                // EV_DEBUG_LINE
                DEBUGNAME!("EV_DEBUG_LINE");
                CG_TestLine(
                    position,
                    (*es).origin2.as_mut_ptr(),
                    (*es).time,
                    (*es).time2 as u32,
                    (*es).weapon,
                );
            }

            _ => {
                DEBUGNAME!("UNKNOWN");
                CG_Error("Unknown event: %i\0".as_ptr() as *const c_char, event);
            }
        }
    }
}

/*
==============
CG_CheckEvents

==============
*/
pub fn CG_CheckEvents(cent: *mut centity_t) {
    unsafe {
        // check for event-only entities
        if (*cent).currentState.eType > 32 {
            // ET_EVENTS
            if (*cent).previousEvent != 0 {
                return; // already fired
            }
            (*cent).previousEvent = 1;

            (*cent).currentState.event = (*cent).currentState.eType - 32; // ET_EVENTS
        } else {
            // check for events riding with another entity
            if (*cent).currentState.event == (*cent).previousEvent {
                return;
            }
            (*cent).previousEvent = (*cent).currentState.event;
            if ((*cent).currentState.event & !0x1f) == 0 {
                // ~EV_EVENT_BITS
                return;
            }
        }

        // calculate the position at exactly the frame time
        EvaluateTrajectory(
            &(*cent).currentState.pos,
            (*cg.snap).serverTime,
            (*cent).lerpOrigin.as_mut_ptr(),
        );
        CG_SetEntitySoundPosition(cent);

        CG_EntityEvent(cent, (*cent).lerpOrigin.as_mut_ptr());
    }
}

// LOCAL STUBS - unresolved dependencies

// Type stubs - these should be defined elsewhere
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: u32,
    pub pos: c_void, // trajectory_t - placeholder
    pub event: c_int,
    pub eventParm: c_int,
    pub weapon: c_int,
    pub clientNum: c_int,
    pub origin2: [f32; 3],
    pub saberActive: u32,
    pub otherEntityNum: c_int,
    pub otherEntityNum2: c_int,
    pub boltInfo: c_int,
    pub loopSound: c_int,
    pub isPortalEnt: u32,
    pub time: c_int,
    pub time2: c_int,
    // ... other fields as needed
}

#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub lerpOrigin: [f32; 3],
    pub previousEvent: c_int,
    pub gent: *mut c_void,
    pub pe: PEState,
}

#[repr(C)]
pub struct PEState {
    pub painTime: c_int,
    pub painDirection: c_int,
}

#[repr(C)]
pub struct gitem_t {
    pub classname: *const c_char,
    pub giType: c_int,
    pub giTag: c_int,
    pub pickup_sound: *const c_char,
    pub pickup_force: *const c_char,
}

// Global stubs
pub static mut cg: CGame = CGame {
    itemPickup: 0,
    itemPickupTime: 0,
    itemPickupBlendTime: 0,
    snap: core::ptr::null_mut(),
    predicted_player_state: playerState_t {
        clientNum: 0,
        weapon: 0,
    },
    time: 0,
    weaponPickupTextTime: 0,
    weaponSelect: 0,
    autoswitch: cvar_t { integer: 0 },
    renderingThirdPerson: 0,
    landChange: 0,
    landTime: 0,
    stepChange: 0.0,
    stepTime: 0,
    powerupActive: 0,
    powerupTime: 0,
    batteryChargeTime: 0,
    wonkyTime: 0,
    inventorySelect: 0,
};

#[repr(C)]
pub struct CGame {
    pub itemPickup: c_int,
    pub itemPickupTime: c_int,
    pub itemPickupBlendTime: c_int,
    pub snap: *mut c_void,
    pub predicted_player_state: playerState_t,
    pub time: c_int,
    pub weaponPickupTextTime: c_int,
    pub weaponSelect: c_int,
    pub autoswitch: cvar_t,
    pub renderingThirdPerson: c_int,
    pub landChange: c_int,
    pub landTime: c_int,
    pub stepChange: f32,
    pub stepTime: c_int,
    pub powerupActive: c_int,
    pub powerupTime: c_int,
    pub batteryChargeTime: c_int,
    pub wonkyTime: c_int,
    pub inventorySelect: c_int,
}

#[repr(C)]
pub struct playerState_t {
    pub clientNum: c_int,
    pub weapon: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
}

pub static mut cgs: CGServer = CGServer {
    media: CMediaStuff {
        footsteps: [[0u32; 4]; 6],
        footstepForces: [[0u32; 4]; 6],
        landSound: 0,
        landForce: 0,
        rollSound: 0,
        lavaInSound: 0,
        lavaOutSound: 0,
        lavaUnSound: 0,
        watrInSound: 0,
        watrOutSound: 0,
        watrUnSound: 0,
        selectSound: 0,
        selectForce: 0,
        disintegrateSound: 0,
        disintegrate2Sound: 0,
        disintegrate3Sound: 0,
        batteryChargeSound: 0,
    },
    sound_precache: [0; 256],
    force_precache: [0; 32],
};

#[repr(C)]
pub struct CGServer {
    pub media: CMediaStuff,
    pub sound_precache: [u32; 256],
    pub force_precache: [u32; 32],
}

#[repr(C)]
pub struct CMediaStuff {
    pub footsteps: [[u32; 4]; 6],
    pub footstepForces: [[u32; 4]; 6],
    pub landSound: u32,
    pub landForce: u32,
    pub rollSound: u32,
    pub lavaInSound: u32,
    pub lavaOutSound: u32,
    pub lavaUnSound: u32,
    pub watrInSound: u32,
    pub watrOutSound: u32,
    pub watrUnSound: u32,
    pub selectSound: u32,
    pub selectForce: u32,
    pub disintegrateSound: u32,
    pub disintegrate2Sound: u32,
    pub disintegrate3Sound: u32,
    pub batteryChargeSound: u32,
}

pub static mut bg_itemlist: [gitem_t; 32] = [gitem_t {
    classname: core::ptr::null(),
    giType: 0,
    giTag: 0,
    pickup_sound: core::ptr::null(),
    pickup_force: core::ptr::null(),
}; 32];

pub static mut bg_numItems: c_int = 0;
pub static mut cg_entities: [centity_t; 2048] = [centity_t {
    currentState: entityState_t {
        number: 0,
        eType: 0,
        eFlags: 0,
        pos: core::ptr::null(),
        event: 0,
        eventParm: 0,
        weapon: 0,
        clientNum: 0,
        origin2: [0.0; 3],
        saberActive: 0,
        otherEntityNum: 0,
        otherEntityNum2: 0,
        boltInfo: 0,
        loopSound: 0,
        isPortalEnt: 0,
        time: 0,
        time2: 0,
    },
    lerpOrigin: [0.0; 3],
    previousEvent: 0,
    gent: core::ptr::null_mut(),
    pe: PEState {
        painTime: 0,
        painDirection: 0,
    },
}; 2048];

pub static mut g_entities: [*mut c_void; 2048] = [core::ptr::null_mut(); 2048]; // placeholder

pub static mut cg_autoswitch: cvar_t = cvar_t { integer: 0 };
pub static mut cg_footsteps: cvar_t = cvar_t { integer: 0 };
pub static mut cg_timescale: cvar_t = cvar_t { integer: 0 };
pub static mut cg_debugEvents: cvar_t = cvar_t { integer: 0 };

pub static vec3_origin: [f32; 3] = [0.0; 3];

pub static mut weaponData: [WeaponData; 16] = [WeaponData {
    selectSnd: core::ptr::null(),
    selectFrc: core::ptr::null(),
}; 16];

#[repr(C)]
pub struct WeaponData {
    pub selectSnd: *const c_char,
    pub selectFrc: *const c_char,
}

// extern "C" function stubs
extern "C" {
    pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char;
    pub fn Com_sprintf(buffer: *mut c_char, bufsize: c_int, fmt: *const c_char, ...);
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
    pub fn cgi_SP_GetStringTextString(
        reference: *const c_char,
        buffer: *mut c_char,
        bufsize: c_int,
    ) -> c_int;
    pub fn cgi_Cvar_Set(var_name: *const c_char, value: *const c_char);
    pub fn SetWeaponSelectTime();
    pub fn CG_OutOfAmmoChange();
    pub fn cgi_S_StartSound(
        origin: *mut [f32; 3],
        entityNum: c_int,
        channel: c_int,
        soundHandle: u32,
    );
    pub fn cgi_S_RegisterSound(name: *const c_char) -> u32;
    pub fn cgi_FF_Start(handle: u32, entityNum: c_int);
    pub fn cgi_FF_Register(name: *const c_char, channel: c_int) -> u32;
    pub fn cgi_FF_StopAll();
    pub fn cgi_FF_Stop(handle: u32, entityNum: c_int);
    pub fn cgi_FF_StartFX(handle: u32);
    pub fn CG_Printf(fmt: *const c_char, ...);
    pub fn CG_ConfigString(index: c_int) -> *const c_char;
    pub fn FX_DisruptorMainShot(start: *mut [f32; 3], end: *mut [f32; 3]);
    pub fn FX_DisruptorAltShot(start: *mut [f32; 3], end: *mut [f32; 3], alt_fire: c_int);
    pub fn FX_DisruptorAltMiss(start: *mut [f32; 3], end: *mut [f32; 3]);
    pub fn FX_DEMP2_AltDetonate(origin: *mut [f32; 3], parm: c_int);
    pub fn FX_ConcAltShot(start: *mut [f32; 3], end: *mut [f32; 3]);
    pub fn FX_ConcAltMiss(start: *mut [f32; 3], end: *mut [f32; 3]);
    pub fn CG_FireWeapon(cent: *mut centity_t, alt: u32);
    pub fn CG_BounceEffect(
        cent: *mut centity_t,
        weapon: c_int,
        origin: *mut [f32; 3],
        otherOrigin: *mut [f32; 3],
    );
    pub fn CG_MissileStick(cent: *mut centity_t, weapon: c_int, origin: *mut [f32; 3]);
    pub fn CG_MissileHitPlayer(
        cent: *mut centity_t,
        weapon: c_int,
        origin: *mut [f32; 3],
        otherOrigin: *mut [f32; 3],
        alt: c_int,
    );
    pub fn CG_MissileHitWall(
        cent: *mut centity_t,
        weapon: c_int,
        origin: *mut [f32; 3],
        otherOrigin: *mut [f32; 3],
        alt: c_int,
    );
    pub fn CG_DrawTargetBeam(
        start: *mut [f32; 3],
        end: *mut [f32; 3],
        other: *mut [f32; 3],
        s1: *const c_char,
        s2: *const c_char,
    );
    pub fn CG_Error(fmt: *const c_char, ...);
    pub fn CG_TestLine(
        start: *mut [f32; 3],
        end: *mut [f32; 3],
        time: c_int,
        time2: u32,
        weapon: c_int,
    );
    pub fn EvaluateTrajectory(pos: *const c_void, time: c_int, result: *mut [f32; 3]);
    pub fn CG_SetEntitySoundPosition(cent: *mut centity_t);
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;
    pub fn sscanf(s: *const c_char, fmt: *const c_char, ...) -> c_int;
    pub fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn VectorCopy(src: *const [f32; 3], dst: &mut [f32; 3]);
    pub fn CrossProduct(a: *const [f32; 3], b: *const [f32; 3], dst: &mut [f32; 3]);
}

pub struct FxSchedulerType;

pub static mut theFxScheduler: FxSchedulerType = FxSchedulerType;

impl FxSchedulerType {
    pub fn PlayEffect(
        &mut self,
        name: *const c_char,
        origin: *mut [f32; 3],
        axis: *mut [[f32; 3]; 3],
        boltInfo: c_int,
        entNum: c_int,
        portalEnt: bool,
        loopSound: c_int,
        isRelative: bool,
    ) {
        // stub
    }

    pub fn PlayEffect_Muzzle(&mut self, name: *const c_char, muzzle: c_int) {
        // overload stub
    }

    pub fn StopEffect(&mut self, name: *const c_char, boltInfo: c_int, portalEnt: bool) {
        // stub
    }
}

pub const STEP_TIME: c_int = 66;
pub const MAX_STEP_CHANGE: f32 = 22.0;
pub const RANK_TIED_FLAG: c_int = 0x80000000;
pub const ET_EVENTS: c_int = 32;
pub const EV_EVENT_BITS: c_int = 0x1f;

#[cfg(target_os = "xbox")]
pub static fffx_FallingMedium: u32 = 0;
#[cfg(target_os = "xbox")]
pub static fffx_FallingFar: u32 = 0;
