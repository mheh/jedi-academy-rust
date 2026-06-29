// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// #include "g_local.h"
// #include "g_functions.h"
// #include "g_items.h"
// #include "wp_saber.h"

#![allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    dead_code,
    unused_variables,
    unused_imports,
    unused_mut,
    unused_assignments,
    clippy::all
)]

use crate::code::game::g_headers_h::*;
use crate::code::game::g_local_h::*;
use crate::code::game::g_functions_h::*;
use crate::code::game::g_items_h::*;
use crate::code::game::wp_saber_h::*;
use core::ffi::{c_char, c_int};
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    static mut missionInfo_Updated: qboolean;
    fn CrystalAmmoSettings(ent: *mut gentity_t);
    fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    fn PM_InKnockDown(ps: *mut playerState_t) -> qboolean;
    fn PM_InGetUp(ps: *mut playerState_t) -> qboolean;
    fn WP_SetSaber(ent: *mut gentity_t, saberNum: c_int, saberName: *mut c_char);
    fn WP_RemoveSaber(ent: *mut gentity_t, saberNum: c_int);
    fn WP_SaberFallSound(owner: *mut gentity_t, saber: *mut gentity_t);
    fn TranslateSaberColor(name: *const c_char) -> saber_colors_t;
    static mut g_spskill: *mut cvar_t;
    static mut g_sex: *mut cvar_t;
    static mut g_saberPickuppableDroppedSabers: *mut cvar_t;
    // From inline extern declarations in function bodies (lifted to module level):
    fn INV_GoodieKeyGive(target: *mut gentity_t) -> qboolean;
    fn INV_SecurityKeyGive(target: *mut gentity_t, keyname: *const c_char) -> qboolean;
    fn G_SetSabersFromCVars(ent: *mut gentity_t);
    static mut g_timescale: *mut cvar_t;
    fn CG_ItemPickup(itemNum: c_int, bHadItem: qboolean);
    static mut delayedShutDown: c_int;
    static mut g_saber: *mut cvar_t;
    fn CG_ChangeWeapon(num: c_int);
    fn Player_CacheFromPrevLevel();
}

const MAX_BACTA_HEAL_AMOUNT: c_int = 25;

/*

  Items are any object that a player can touch to gain some effect.

  Pickup will return the number of seconds until they should respawn.

  all items should pop when dropped in lava or slime

  Respawnable items don't actually go away when picked up, they are
  just made invisible and untouchable.  This allows them to ride
  movers and respawn apropriately.
*/

// Item Spawn flags
const ITMSF_SUSPEND: c_int = 1;
const ITMSF_NOPLAYER: c_int = 2;
const ITMSF_ALLOWNPC: c_int = 4;
const ITMSF_NOTSOLID: c_int = 8;
const ITMSF_VERTICAL: c_int = 16;
const ITMSF_INVISIBLE: c_int = 32;
const ITMSF_NOGLOW: c_int = 64;
const ITMSF_USEPICKUP: c_int = 128;
const ITMSF_STATIONARY: c_int = 2048;

//======================================================================

/*
===============
G_InventorySelectable
===============
*/
pub unsafe fn G_InventorySelectable(index: c_int, other: *mut gentity_t) -> qboolean {
    if (*(*other).client).ps.inventory[index as usize] != 0 {
        return qtrue;
    }

    return qfalse;
}

// INV_GoodieKeyGive and INV_SecurityKeyGive are extern "C" (declared above, originally
// inline in this function body in C; lifted to module-level extern block in Rust).
pub unsafe fn Pickup_Holdable(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let mut i: c_int;
    let mut original: c_int;

    (*(*other).client).ps.stats[STAT_ITEMS as usize] |= 1 << (*(*ent).item).giTag;

    if (*(*ent).item).giTag == INV_SECURITY_KEY {
        //give the key
        //FIXME: temp message
        gi.SendServerCommand(0, b"cp @SP_INGAME_YOU_TOOK_SECURITY_KEY\0".as_ptr() as *const c_char);
        INV_SecurityKeyGive(other, (*ent).message);
    } else if (*(*ent).item).giTag == INV_GOODIE_KEY {
        //give the key
        //FIXME: temp message
        gi.SendServerCommand(0, b"cp @SP_INGAME_YOU_TOOK_SUPPLY_KEY\0".as_ptr() as *const c_char);
        INV_GoodieKeyGive(other);
    } else {
        // Picking up a normal item?
        (*(*other).client).ps.inventory[(*(*ent).item).giTag as usize] += 1;
    }
    // Got a security key

    // Set the inventory select, just in case it hasn't
    original = cg.inventorySelect;
    i = 0;
    while i < INV_MAX {
        if (cg.inventorySelect < INV_ELECTROBINOCULARS) || (cg.inventorySelect >= INV_MAX) {
            cg.inventorySelect = INV_MAX - 1;
        }

        if G_InventorySelectable(cg.inventorySelect, other) != 0 {
            return 60;
        }
        cg.inventorySelect += 1;
        i += 1;
    }

    cg.inventorySelect = original;

    return 60;
}


//======================================================================
pub unsafe fn Add_Ammo2(ent: *mut gentity_t, ammoType: c_int, count: c_int) -> c_int {

    if ammoType != AMMO_FORCE {
        (*(*ent).client).ps.ammo[ammoType as usize] += count;

        // since the ammo is the weapon in this case, picking up ammo should actually give you the weapon
        match ammoType {
            _ if ammoType == AMMO_THERMAL => {
                (*(*ent).client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_THERMAL;
            }
            _ if ammoType == AMMO_DETPACK => {
                (*(*ent).client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_DET_PACK;
            }
            _ if ammoType == AMMO_TRIPMINE => {
                (*(*ent).client).ps.stats[STAT_WEAPONS as usize] |= 1 << WP_TRIP_MINE;
            }
            _ => {}
        }

        if (*(*ent).client).ps.ammo[ammoType as usize] > ammoData[ammoType as usize].max {
            (*(*ent).client).ps.ammo[ammoType as usize] = ammoData[ammoType as usize].max;
            return qfalse;
        }
    } else {
        if (*(*ent).client).ps.forcePower >= ammoData[ammoType as usize].max {
            //if have full force, just get 25 extra per crystal
            (*(*ent).client).ps.forcePower += 25;
        } else {
            //else if don't have full charge, give full amount, up to max + 25
            (*(*ent).client).ps.forcePower += count;
            if (*(*ent).client).ps.forcePower >= ammoData[ammoType as usize].max + 25 {
                //cap at max + 25
                (*(*ent).client).ps.forcePower = ammoData[ammoType as usize].max + 25;
            }
        }

        if (*(*ent).client).ps.forcePower >= ammoData[ammoType as usize].max * 2 {
            //always cap at twice a full charge
            (*(*ent).client).ps.forcePower = ammoData[ammoType as usize].max * 2;
            return qfalse; // can't hold any more
        }
    }
    return qtrue;
}

//-------------------------------------------------------
pub unsafe fn Add_Ammo(ent: *mut gentity_t, weapon: c_int, count: c_int) {
    Add_Ammo2(ent, weaponData[weapon as usize].ammoIndex, count);
}

//-------------------------------------------------------
pub unsafe fn Pickup_Ammo(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let mut quantity: c_int;

    if (*ent).count != 0 {
        quantity = (*ent).count;
    } else {
        quantity = (*(*ent).item).quantity;
    }

    Add_Ammo2(other, (*(*ent).item).giTag, quantity);

    return 30;
}

//======================================================================
pub unsafe fn Add_Batteries(ent: *mut gentity_t, count: *mut c_int) {
    if !(*ent).client.is_null()
        && (*(*ent).client).ps.batteryCharge < MAX_BATTERIES
        && *count != 0
    {
        if *count + (*(*ent).client).ps.batteryCharge > MAX_BATTERIES {
            // steal what we need, then leave the rest for later
            *count -= MAX_BATTERIES - (*(*ent).client).ps.batteryCharge;
            (*(*ent).client).ps.batteryCharge = MAX_BATTERIES;
        } else {
            // just drain all of the batteries
            (*(*ent).client).ps.batteryCharge += *count;
            *count = 0;
        }

        G_AddEvent(ent, EV_BATTERIES_CHARGED, 0);
    }
}

//-------------------------------------------------------
pub unsafe fn Pickup_Battery(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let mut quantity: c_int;

    if (*ent).count != 0 {
        quantity = (*ent).count;
    } else {
        quantity = (*(*ent).item).quantity;
    }

    // There may be some left over in quantity if the player is close to full, but with pickup items, this amount will just be lost
    Add_Batteries(other, addr_of_mut!(quantity));

    return 30;
}

//======================================================================

pub unsafe fn G_CopySaberItemValues(pickUpSaber: *mut gentity_t, oldSaber: *mut gentity_t) {
    if !oldSaber.is_null() && !pickUpSaber.is_null() {
        (*oldSaber).spawnflags = (*pickUpSaber).spawnflags;
        (*oldSaber).random = (*pickUpSaber).random;
        (*oldSaber).flags = (*pickUpSaber).flags;
    }
}

pub unsafe fn G_DropSaberItem(
    saberType: *const c_char,
    saberColor: saber_colors_t,
    saberPos: *mut vec3_t,
    saberVel: *mut vec3_t,
    saberAngles: *mut vec3_t,
    copySaber: *mut gentity_t,
) -> *mut gentity_t {
    //turn it into a pick-uppable item!
    let mut newItem: *mut gentity_t = core::ptr::null_mut();
    if !saberType.is_null() && *saberType != 0 {
        //have a valid string to use for saberType
        newItem = G_Spawn();
        if !newItem.is_null() {
            (*newItem).classname = G_NewString(b"weapon_saber\0".as_ptr() as *const c_char);
            VectorCopy(saberPos, (*newItem).s.origin.as_mut_ptr() as *mut vec3_t);
            G_SetOrigin(newItem, (*newItem).s.origin.as_ptr() as *const vec3_t);
            VectorCopy(saberAngles, (*newItem).s.angles.as_mut_ptr() as *mut vec3_t);
            G_SetAngles(newItem, (*newItem).s.angles.as_ptr() as *const vec3_t);
            (*newItem).spawnflags = 128; /*ITMSF_USEPICKUP*/
            (*newItem).spawnflags |= 64; /*ITMSF_NOGLOW*/
            (*newItem).NPC_type = G_NewString(saberType); //saberType
            //FIXME: transfer per-blade color somehow?
            (*newItem).NPC_targetname = saberColorStringForColor[saberColor as usize];
            (*newItem).count = 1;
            (*newItem).flags = FL_DROPPED_ITEM;
            G_SpawnItem(newItem, FindItemForWeapon(WP_SABER));
            (*newItem).s.pos.trType = TR_GRAVITY;
            (*newItem).s.pos.trTime = level.time;
            VectorCopy(saberVel, (*newItem).s.pos.trDelta.as_mut_ptr() as *mut vec3_t);
            //newItem->s.eFlags |= EF_BOUNCE_HALF;
            //copy some values from another saber, if provided:
            G_CopySaberItemValues(copySaber, newItem);
            //don't *think* about calling FinishSpawningItem, just do it!
            (*newItem).e_ThinkFunc = thinkF_NULL;
            (*newItem).nextthink = -1;
            FinishSpawningItem(newItem);
            (*newItem).delay = level.time + 500; //so you can't pick it back up right away
        }
    }
    return newItem;
}

// G_SetSabersFromCVars is extern "C" (originally inline in this function body in C; lifted above).
pub unsafe fn Pickup_Saber(
    self_: *mut gentity_t,
    hadSaber: qboolean,
    pickUpSaber: *mut gentity_t,
) -> qboolean {
    //NOTE: loopAnim = saberSolo, alt_fire = saberLeftHand, NPC_type = saberType, NPC_targetname = saberColor
    let mut foundIt: qboolean = qfalse;

    if pickUpSaber.is_null() || self_.is_null() || (*self_).client.is_null() {
        return qfalse;
    }

    //G_RemoveWeaponModels( ent );//???
    if Q_stricmp(b"player\0".as_ptr() as *const c_char, (*pickUpSaber).NPC_type) == 0 {
        //"player" means use cvar info
        G_SetSabersFromCVars(self_);
        foundIt = qtrue;
    } else {
        let mut newSaber: saberInfo_t = core::mem::zeroed();
        let mut swapSabers: qboolean = qfalse;

        if (*(*self_).client).ps.weapon == WP_SABER
            && (*(*self_).client).ps.weaponTime > 0
        {
            //can't pick up a new saber while the old one is busy (also helps to work as a debouncer so you don't swap out sabers rapidly when touching more than one at a time)
            return qfalse;
        }

        if (*pickUpSaber).count == 1 && (*g_saberPickuppableDroppedSabers).integer != 0 {
            swapSabers = qtrue;
        }

        if WP_SaberParseParms((*pickUpSaber).NPC_type, addr_of_mut!(newSaber)) != 0 {
            //successfully found a saber .sab entry to use
            let mut saberNum: c_int = 0;
            let mut removeLeftSaber: qboolean = qfalse;
            if (*pickUpSaber).alt_fire != 0 {
                //always go in the left hand
                if hadSaber == 0 {
                    //can't have a saber only in your left hand!
                    return qfalse;
                }
                saberNum = 1;
                //just in case...
                removeLeftSaber = qtrue;
            } else if hadSaber == 0 {
                //don't have a saber at all yet, put it in our right hand
                saberNum = 0;
                //just in case...
                removeLeftSaber = qtrue;
            } else if (*pickUpSaber).loopAnim != 0 //only supposed to use this one saber when grab this pickup
                || (newSaber.saberFlags & SFL_TWO_HANDED) != 0 //new saber is two-handed
                || (hadSaber != 0 && ((*(*self_).client).ps.saber[0].saberFlags & SFL_TWO_HANDED) != 0)
            //old saber is two-handed
            {
                //replace the old right-hand saber and remove the left hand one
                saberNum = 0;
                removeLeftSaber = qtrue;
            } else {
                //have, at least, a saber in our right hand and the new one could go in either left or right hand
                if (*(*self_).client).ps.dualSabers != 0 {
                    //I already have 2 sabers
                    let mut dir2Saber: vec3_t = [0.0; 3];
                    let mut rightDir: vec3_t = [0.0; 3];
                    //to determine which one to replace, see which side of me it's on
                    VectorSubtract(
                        (*pickUpSaber).currentOrigin.as_ptr() as *const vec3_t,
                        (*self_).currentOrigin.as_ptr() as *const vec3_t,
                        addr_of_mut!(dir2Saber),
                    );
                    dir2Saber[2] = 0.0;
                    AngleVectors(
                        (*self_).currentAngles.as_ptr() as *const vec3_t,
                        core::ptr::null_mut(),
                        addr_of_mut!(rightDir),
                        core::ptr::null_mut(),
                    );
                    rightDir[2] = 0.0;
                    if DotProduct(
                        addr_of!(rightDir) as *const vec3_t,
                        addr_of!(dir2Saber) as *const vec3_t,
                    ) > 0.0
                    {
                        saberNum = 0;
                    } else {
                        saberNum = 1;
                        //just in case...
                        removeLeftSaber = qtrue;
                    }
                } else {
                    //just add it as a second saber
                    saberNum = 1;
                    //just in case...
                    removeLeftSaber = qtrue;
                }
            }
            if saberNum == 0 {
                //want to reach out with right hand
                if (*(*self_).client).ps.torsoAnim == BOTH_BUTTON_HOLD {
                    //but only if already playing the pickup with left hand anim...
                    NPC_SetAnim(
                        self_,
                        SETANIM_TORSO,
                        BOTH_SABERPULL,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                }
                if swapSabers != 0 {
                    //drop first one where the one we're picking up is
                    G_DropSaberItem(
                        (*(*self_).client).ps.saber[saberNum as usize].name,
                        (*(*self_).client).ps.saber[saberNum as usize].blade[0].color,
                        (*pickUpSaber).currentOrigin.as_mut_ptr() as *mut vec3_t,
                        addr_of_mut!(vec3_origin) as *mut vec3_t,
                        (*pickUpSaber).currentAngles.as_mut_ptr() as *mut vec3_t,
                        pickUpSaber,
                    );
                    if removeLeftSaber != 0 {
                        //drop other one at my origin
                        G_DropSaberItem(
                            (*(*self_).client).ps.saber[1].name,
                            (*(*self_).client).ps.saber[1].blade[0].color,
                            (*self_).currentOrigin.as_mut_ptr() as *mut vec3_t,
                            addr_of_mut!(vec3_origin) as *mut vec3_t,
                            (*self_).currentAngles.as_mut_ptr() as *mut vec3_t,
                            pickUpSaber,
                        );
                    }
                }
            } else {
                if swapSabers != 0 {
                    G_DropSaberItem(
                        (*(*self_).client).ps.saber[saberNum as usize].name,
                        (*(*self_).client).ps.saber[saberNum as usize].blade[0].color,
                        (*pickUpSaber).currentOrigin.as_mut_ptr() as *mut vec3_t,
                        addr_of_mut!(vec3_origin) as *mut vec3_t,
                        (*pickUpSaber).currentAngles.as_mut_ptr() as *mut vec3_t,
                        pickUpSaber,
                    );
                }
            }
            if removeLeftSaber != 0 {
                WP_RemoveSaber(self_, 1);
            }
            WP_SetSaber(self_, saberNum, (*pickUpSaber).NPC_type);
            WP_SaberInitBladeData(self_);
            if (*(*self_).client).ps.saber[saberNum as usize].stylesLearned != 0 {
                (*(*self_).client).ps.saberStylesKnown |=
                    (*(*self_).client).ps.saber[saberNum as usize].stylesLearned;
            }
            if (*(*self_).client).ps.saber[saberNum as usize].singleBladeStyle != 0 {
                (*(*self_).client).ps.saberStylesKnown |=
                    (*(*self_).client).ps.saber[saberNum as usize].singleBladeStyle;
            }
            if !(*pickUpSaber).NPC_targetname.is_null() {
                //NPC_targetname = saberColor
                let saber_color: saber_colors_t =
                    TranslateSaberColor((*pickUpSaber).NPC_targetname);
                for bladeNum in 0..MAX_BLADES as usize {
                    (*(*self_).client).ps.saber[saberNum as usize].blade[bladeNum].color =
                        saber_color;
                }
            }
            if (*(*self_).client).ps.torsoAnim == BOTH_BUTTON_HOLD
                || (*(*self_).client).ps.torsoAnim == BOTH_SABERPULL
            {
                //don't let them attack right away, force them to finish the anim
                (*(*self_).client).ps.weaponTime =
                    (*(*self_).client).ps.torsoAnimTimer;
            }
            foundIt = qtrue;
        }
        WP_SaberFreeStrings(newSaber);
    }
    return foundIt;
}

// CG_ChangeWeapon is extern "C" (originally inline in this function body in C; lifted above).
pub unsafe fn Pickup_Weapon(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let mut quantity: c_int;
    let mut hadWeapon: qboolean = qfalse;

    /*
    if ( ent->count || (ent->activator && !ent->activator->s.number) )
    {
        quantity = ent->count;
    }
    else
    {
        quantity = ent->item->quantity;
    }
    */

    // dropped items are always picked up
    if (*ent).flags & FL_DROPPED_ITEM != 0 {
        quantity = (*ent).count;
    } else {
        //wasn't dropped
        let q = (*(*ent).item).quantity;
        quantity = if q != 0 { q } else { 50 };
    }

    // add the weapon
    if (*(*other).client).ps.stats[STAT_WEAPONS as usize] & (1 << (*(*ent).item).giTag) != 0 {
        hadWeapon = qtrue;
    }
    (*(*other).client).ps.stats[STAT_WEAPONS as usize] |= 1 << (*(*ent).item).giTag;

    if (*(*ent).item).giTag == WP_SABER
        && (hadWeapon == 0 || !(*ent).NPC_type.is_null())
    {
        //didn't have a saber or it is specifying a certain kind of saber to use
        if Pickup_Saber(other, hadWeapon, ent) == 0 {
            return 0;
        }
    }

    if (*other).s.number != 0 {
        //NPC
        if (*other).s.weapon == WP_NONE || (*(*ent).item).giTag == WP_SABER {
            //NPC with no weapon picked up a weapon, change to this weapon
            //FIXME: clear/set the alt-fire flag based on the picked up weapon and my class?
            (*(*other).client).ps.weapon = (*(*ent).item).giTag;
            (*(*other).client).ps.weaponstate = WEAPON_RAISING;
            ChangeWeapon(other, (*(*ent).item).giTag);
            if (*(*ent).item).giTag == WP_SABER {
                (*(*other).client).ps.SaberActivate();
                WP_SaberAddG2SaberModels(other);
            } else {
                G_CreateG2AttachedWeaponModel(
                    other,
                    weaponData[(*(*ent).item).giTag as usize].weaponMdl,
                    (*other).handRBolt,
                    0,
                );
            }
        }
    }
    if (*(*ent).item).giTag == WP_SABER {
        //picked up a saber
        if (*other).s.weapon != WP_SABER {
            //player picking up saber
            (*(*other).client).ps.weapon = WP_SABER;
            (*(*other).client).ps.weaponstate = WEAPON_RAISING;
            if (*other).s.number < MAX_CLIENTS {
                //make sure the cgame-side knows this
                CG_ChangeWeapon(WP_SABER);
            } else {
                //make sure the cgame-side knows this
                ChangeWeapon(other, WP_SABER);
            }
        }
        if (*(*other).client).ps.SaberActive() == 0 {
            //turn it/them on!
            (*(*other).client).ps.SaberActivate();
        }
    }

    if quantity != 0 {
        // Give ammo
        Add_Ammo(other, (*(*ent).item).giTag, quantity);
    }
    return 5;
}


//======================================================================

pub unsafe fn ITM_AddHealth(ent: *mut gentity_t, count: c_int) -> c_int {

    (*ent).health += count;

    if (*ent).health > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] {
        // Past max health
        (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];

        return qfalse;
    }

    return qtrue;
}

pub unsafe fn Pickup_Health(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let mut max: c_int;
    let mut quantity: c_int;

    max = (*(*other).client).ps.stats[STAT_MAX_HEALTH as usize];

    if (*ent).count != 0 {
        quantity = (*ent).count;
    } else {
        quantity = (*(*ent).item).quantity;
    }

    (*other).health += quantity;

    if (*other).health > max {
        (*other).health = max;
    }

    if (*(*ent).item).giTag == 100 {
        // mega health respawns slow
        return 120;
    }

    return 30;
}

//======================================================================

pub unsafe fn ITM_AddArmor(ent: *mut gentity_t, count: c_int) -> c_int {

    (*(*ent).client).ps.stats[STAT_ARMOR as usize] += count;

    if (*(*ent).client).ps.stats[STAT_ARMOR as usize]
        > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize]
    {
        (*(*ent).client).ps.stats[STAT_ARMOR as usize] =
            (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
        return qfalse;
    }

    return qtrue;
}


pub unsafe fn Pickup_Armor(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {

    // make sure that the shield effect is on
    (*(*other).client).ps.powerups[PW_BATTLESUIT as usize] = Q3_INFINITE;

    (*(*other).client).ps.stats[STAT_ARMOR as usize] += (*(*ent).item).quantity;
    if (*(*other).client).ps.stats[STAT_ARMOR as usize]
        > (*(*other).client).ps.stats[STAT_MAX_HEALTH as usize]
    {
        (*(*other).client).ps.stats[STAT_ARMOR as usize] =
            (*(*other).client).ps.stats[STAT_MAX_HEALTH as usize];
    }

    return 30;
}



//======================================================================

pub unsafe fn Pickup_Holocron(ent: *mut gentity_t, other: *mut gentity_t) -> c_int {
    let forcePower: c_int = (*(*ent).item).giTag;
    let forceLevel: c_int = (*ent).count;
    // check if out of range
    if forceLevel < 0 || forceLevel >= NUM_FORCE_POWER_LEVELS {
        gi.Printf(
            b" Pickup_Holocron : count %d not in valid range\n\0".as_ptr() as *const c_char,
            forceLevel,
        );
        return 1;
    }

    // don't pick up if already known AND your level is higher than pickup level
    if (*(*other).client).ps.forcePowersKnown & (1 << forcePower) != 0 {
        //don't pickup if item is lower than current level
        if (*(*other).client).ps.forcePowerLevel[forcePower as usize] >= forceLevel {
            return 1;
        }
    }

    (*(*other).client).ps.forcePowerLevel[forcePower as usize] = forceLevel;
    (*(*other).client).ps.forcePowersKnown |= 1 << forcePower;

    missionInfo_Updated = qtrue; // Activate flashing text
    gi.cvar_set(
        b"cg_updatedDataPadForcePower1\0".as_ptr() as *const c_char,
        va(b"%d\0".as_ptr() as *const c_char, forcePower + 1),
    ); // The +1 is offset in the print routine.
    cg_updatedDataPadForcePower1.integer = forcePower + 1;
    gi.cvar_set(
        b"cg_updatedDataPadForcePower2\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
    ); // The +1 is offset in the print routine.
    cg_updatedDataPadForcePower2.integer = 0;
    gi.cvar_set(
        b"cg_updatedDataPadForcePower3\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
    ); // The +1 is offset in the print routine.
    cg_updatedDataPadForcePower3.integer = 0;

    return 1;
}


//======================================================================

/*
===============
RespawnItem
===============
*/
pub unsafe fn RespawnItem(ent: *mut gentity_t) {
}


pub unsafe fn CheckItemCanBePickedUpByNPC(
    item: *mut gentity_t,
    pickerupper: *mut gentity_t,
) -> qboolean {
    if ((*item).flags & FL_DROPPED_ITEM) != 0
        && (*item).activator != addr_of_mut!(g_entities[0])
        && (*pickerupper).s.number != 0
        && (*pickerupper).s.weapon == WP_NONE
        && !(*pickerupper).enemy.is_null()
        && (*pickerupper).painDebounceTime < level.time
        && !(*pickerupper).NPC.is_null()
        && (*(*pickerupper).NPC).surrenderTime < level.time //not surrendering
        && ((*(*pickerupper).NPC).scriptFlags & SCF_FORCED_MARCH) == 0 //not being forced to march
        && (*(*item).item).giTag != INV_SECURITY_KEY
    {
        //non-player, in combat, picking up a dropped item that does NOT belong to the player and it *not* a security key
        if level.time - (*item).s.time < 3000 {
            //was 5000
            return qfalse;
        }
        return qtrue;
    }
    return qfalse;
}

pub unsafe fn G_CanPickUpWeapons(other: *mut gentity_t) -> qboolean {
    if other.is_null() || (*other).client.is_null() {
        return qfalse;
    }
    match (*(*other).client).NPC_class {
        _ if (*(*other).client).NPC_class == CLASS_ATST => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_GONK => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_MARK1 => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_MARK2 => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_MOUSE => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_PROBE => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_PROTOCOL => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_R2D2 => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_R5D2 => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_SEEKER => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_REMOTE => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_RANCOR => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_WAMPA => { return qfalse; }
        _ if (*(*other).client).NPC_class == CLASS_JAWA => { return qfalse; } //FIXME: in some cases it's okay?
        _ if (*(*other).client).NPC_class == CLASS_UGNAUGHT => { return qfalse; } //FIXME: in some cases it's okay?
        _ if (*(*other).client).NPC_class == CLASS_SENTRY => { return qfalse; }
        _ => {}
    }
    return qtrue;
}
/*
===============
Touch_Item
===============
*/
// g_timescale is extern "C" (originally inline in this function body in C; lifted above).
// CG_ItemPickup is extern "C" (originally inline in this function body in C; lifted above).
pub unsafe fn Touch_Item(ent: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    let mut respawn: c_int = 0;

    if (*other).client.is_null() {
        return;
    }
    if (*other).health < 1 {
        return; // dead people can't pickup
    }

    if (*(*other).client).ps.pm_time > 0 {
        //cant pick up when out of control
        return;
    }

    // NPCs can pick it up
    if ((*ent).spawnflags & ITMSF_ALLOWNPC) != 0 && ((*other).s.number == 0) {
        return;
    }

    // Players cannot pick it up
    if ((*ent).spawnflags & ITMSF_NOPLAYER) != 0 && ((*other).s.number != 0) {
        return;
    }

    if (*ent).noDamageTeam != TEAM_FREE
        && (*(*other).client).playerTeam != (*ent).noDamageTeam
    {
        //only one team can pick it up
        return;
    }

    if G_CanPickUpWeapons(other) == 0 {
        //FIXME: some flag would be better
        //droids can't pick up items/weapons!
        return;
    }

    //FIXME: need to make them run toward a dropped weapon when fleeing without one?
    //FIXME: need to make them come out of flee mode when pick up their old weapon?
    if CheckItemCanBePickedUpByNPC(ent, other) != 0 {
        if !(*other).NPC.is_null()
            && !(*(*other).NPC).goalEntity.is_null()
            && (*(*other).NPC).goalEntity == ent
        {
            //they were running to pick me up, they did, so clear goal
            (*(*other).NPC).goalEntity = core::ptr::null_mut();
            (*(*other).NPC).squadState = SQUAD_STAND_AND_SHOOT;
            (*NPCInfo).tempBehavior = BS_DEFAULT;
            TIMER_Set(other, b"flee\0".as_ptr() as *const c_char, -1);
        } else {
            return;
        }
    } else if ((*ent).spawnflags & ITMSF_ALLOWNPC) == 0 {
        // NPCs cannot pick it up
        if (*other).s.number != 0 {
            // Not the player?
            return;
        }
    }

    // the same pickup rules are used for client side and server side
    if BG_CanItemBeGrabbed(addr_of!((*ent).s), addr_of_mut!((*(*other).client).ps)) == 0 {
        return;
    }

    if !(*other).client.is_null() {
        if ((*(*other).client).ps.eFlags & EF_FORCE_GRIPPED) != 0
            || ((*(*other).client).ps.eFlags & EF_FORCE_DRAINED) != 0
        {
            //can't pick up anything while being gripped
            return;
        }
        if PM_InKnockDown(addr_of_mut!((*(*other).client).ps)) != 0
            && PM_InGetUp(addr_of_mut!((*(*other).client).ps)) == 0
        {
            //can't pick up while in a knockdown
            return;
        }
    }
    if (*ent).item.is_null() {
        //not an item!
        gi.Printf(
            b"Touch_Item: %s is not an item!\n\0".as_ptr() as *const c_char,
            (*ent).classname,
        );
        return;
    }

    if (*(*ent).item).giType == IT_WEAPON && (*(*ent).item).giTag == WP_SABER {
        //a saber
        if (*ent).delay > level.time {
            //just picked it up, don't pick up again right away
            return;
        }
    }

    if (*other).s.number < MAX_CLIENTS && ((*ent).spawnflags & ITMSF_USEPICKUP) != 0 {
        //only if player is holing use button
        if ((*(*other).client).usercmd.buttons & BUTTON_USE) == 0 {
            //not holding use?
            return;
        }
    }

    let mut bHadWeapon: qboolean = qfalse;
    // call the item-specific pickup function
    match (*(*ent).item).giType {
        _ if (*(*ent).item).giType == IT_WEAPON => {
            if !(*other).NPC.is_null() && (*other).s.weapon == WP_NONE {
                //Make them duck and sit here for a few seconds
                let pickUpTime: c_int = Q_irand(1000, 3000);
                TIMER_Set(other, b"duck\0".as_ptr() as *const c_char, pickUpTime);
                TIMER_Set(other, b"roamTime\0".as_ptr() as *const c_char, pickUpTime);
                TIMER_Set(other, b"stick\0".as_ptr() as *const c_char, pickUpTime);
                TIMER_Set(other, b"verifyCP\0".as_ptr() as *const c_char, pickUpTime);
                TIMER_Set(other, b"attackDelay\0".as_ptr() as *const c_char, 600);
                respawn = 0;
            }
            if (*(*other).client).ps.stats[STAT_WEAPONS as usize]
                & (1 << (*(*ent).item).giTag)
                != 0
            {
                bHadWeapon = qtrue;
            }
            respawn = Pickup_Weapon(ent, other);
        }
        _ if (*(*ent).item).giType == IT_AMMO => {
            respawn = Pickup_Ammo(ent, other);
        }
        _ if (*(*ent).item).giType == IT_ARMOR => {
            respawn = Pickup_Armor(ent, other);
        }
        _ if (*(*ent).item).giType == IT_HEALTH => {
            respawn = Pickup_Health(ent, other);
        }
        _ if (*(*ent).item).giType == IT_HOLDABLE => {
            respawn = Pickup_Holdable(ent, other);
        }
        _ if (*(*ent).item).giType == IT_BATTERY => {
            respawn = Pickup_Battery(ent, other);
        }
        _ if (*(*ent).item).giType == IT_HOLOCRON => {
            respawn = Pickup_Holocron(ent, other);
        }
        _ => {
            return;
        }
    }

    if respawn == 0 {
        return;
    }

    // play the normal pickup sound
    if (*other).s.number == 0 && (*g_timescale).value < 1.0_f32 {
        //SIGH... with timescale on, you lose events left and right
        // but we're SP so we'll cheat
        cgi_S_StartSound(
            core::ptr::null(),
            (*other).s.number,
            CHAN_AUTO,
            cgi_S_RegisterSound((*(*ent).item).pickup_sound),
        );
        // show icon and name on status bar
        CG_ItemPickup((*ent).s.modelindex, bHadWeapon);
    } else {
        if bHadWeapon != 0 {
            G_AddEvent(other, EV_ITEM_PICKUP, -(*ent).s.modelindex);
        } else {
            G_AddEvent(other, EV_ITEM_PICKUP, (*ent).s.modelindex);
        }
    }

    // fire item targets
    G_UseTargets(ent, other);

    if (*(*ent).item).giType == IT_WEAPON && (*(*ent).item).giTag == WP_SABER {
        //a saber that was picked up
        if (*ent).count < 0 {
            //infinite supply
            (*ent).delay = level.time + 500;
            return;
        }
        (*ent).count -= 1;
        if (*ent).count > 0 {
            //still have more to pick up
            (*ent).delay = level.time + 500;
            return;
        }
    }
    // wait of -1 will not respawn
    //	if ( ent->wait == -1 )
    {
        //why not just remove me?
        G_FreeEntity(ent);
        /*
        //NOTE: used to do this:  (for respawning?)
        ent->svFlags |= SVF_NOCLIENT;
        ent->s.eFlags |= EF_NODRAW;
        ent->contents = 0;
        ent->unlinkAfterEvent = qtrue;
        */
        return;
    }
}


//======================================================================

/*
================
LaunchItem

Spawns an item and tosses it forward
================
*/
pub unsafe fn LaunchItem(
    item: *const gitem_t,
    origin: *const vec3_t,
    velocity: *const vec3_t,
    target: *mut c_char,
) -> *mut gentity_t {
    let mut dropped: *mut gentity_t;

    dropped = G_Spawn();

    (*dropped).s.eType = ET_ITEM;
    (*dropped).s.modelindex = item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as c_int; // store item number in modelindex
    (*dropped).s.modelindex2 = 1; // This is non-zero is it's a dropped item

    (*dropped).classname = G_NewString((*item).classname); //copy it so it can be freed safely
    (*dropped).item = item as *mut gitem_t;

    // try using the "correct" mins/maxs first
    VectorSet(
        (*dropped).mins.as_mut_ptr() as *mut vec3_t,
        (*item).mins[0],
        (*item).mins[1],
        (*item).mins[2],
    );
    VectorSet(
        (*dropped).maxs.as_mut_ptr() as *mut vec3_t,
        (*item).maxs[0],
        (*item).maxs[1],
        (*item).maxs[2],
    );

    if ((*dropped).mins[0] == 0.0 && (*dropped).mins[1] == 0.0 && (*dropped).mins[2] == 0.0)
        && ((*dropped).maxs[0] == 0.0 && (*dropped).maxs[1] == 0.0 && (*dropped).maxs[2] == 0.0)
    {
        VectorSet(
            (*dropped).maxs.as_mut_ptr() as *mut vec3_t,
            ITEM_RADIUS,
            ITEM_RADIUS,
            ITEM_RADIUS,
        );
        VectorScale(
            (*dropped).maxs.as_ptr() as *const vec3_t,
            -1.0,
            (*dropped).mins.as_mut_ptr() as *mut vec3_t,
        );
    }

    (*dropped).contents = CONTENTS_TRIGGER | CONTENTS_ITEM; //CONTENTS_TRIGGER;//not CONTENTS_BODY for dropped items, don't need to ID them

    if !target.is_null() && *target != 0 {
        (*dropped).target = G_NewString(target as *const c_char);
    } else {
        // if not targeting something, auto-remove after 30 seconds
        // only if it's NOT a security or goodie key
        if (*(*dropped).item).giTag != INV_SECURITY_KEY {
            (*dropped).e_ThinkFunc = thinkF_G_FreeEntity;
            (*dropped).nextthink = level.time + 30000;
        }

        if (*(*dropped).item).giType == IT_AMMO && (*(*dropped).item).giTag == AMMO_FORCE {
            (*dropped).nextthink = -1;
            (*dropped).e_ThinkFunc = thinkF_NULL;
        }
    }

    (*dropped).e_TouchFunc = touchF_Touch_Item;

    if (*item).giType == IT_WEAPON {
        // give weapon items zero pitch, a random yaw, and rolled onto their sides...but would be bad to do this for a bowcaster
        if (*item).giTag != WP_BOWCASTER
            && (*item).giTag != WP_THERMAL
            && (*item).giTag != WP_TRIP_MINE
            && (*item).giTag != WP_DET_PACK
        {
            VectorSet(
                (*dropped).s.angles.as_mut_ptr() as *mut vec3_t,
                0.0,
                crandom() * 180.0,
                90.0_f32,
            );
            G_SetAngles(dropped, (*dropped).s.angles.as_ptr() as *const vec3_t);
        }
    }

    G_SetOrigin(dropped, origin);
    (*dropped).s.pos.trType = TR_GRAVITY;
    (*dropped).s.pos.trTime = level.time;
    VectorCopy(velocity, (*dropped).s.pos.trDelta.as_mut_ptr() as *mut vec3_t);

    (*dropped).s.eFlags |= EF_BOUNCE_HALF;

    (*dropped).flags = FL_DROPPED_ITEM;

    gi.linkentity(dropped);

    return dropped;
}

/*
================
Drop_Item

Spawns an item and tosses it forward
================
*/
pub unsafe fn Drop_Item(
    ent: *mut gentity_t,
    item: *mut gitem_t,
    angle: f32,
    copytarget: qboolean,
) -> *mut gentity_t {
    let mut dropped: *mut gentity_t = core::ptr::null_mut();
    let mut velocity: vec3_t = [0.0; 3];
    let mut angles: vec3_t = [0.0; 3];

    VectorCopy(
        (*ent).s.apos.trBase.as_ptr() as *const vec3_t,
        addr_of_mut!(angles),
    );
    angles[YAW as usize] += angle;
    angles[PITCH as usize] = 0.0; // always forward

    AngleVectors(
        addr_of!(angles),
        addr_of_mut!(velocity),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    VectorScale(addr_of!(velocity), 150.0, addr_of_mut!(velocity));
    velocity[2] += 200.0 + crandom() * 50.0;

    if copytarget != 0 {
        dropped = LaunchItem(item, (*ent).s.pos.trBase.as_ptr() as *const vec3_t, addr_of!(velocity), (*ent).opentarget);
    } else {
        dropped = LaunchItem(item, (*ent).s.pos.trBase.as_ptr() as *const vec3_t, addr_of!(velocity), core::ptr::null_mut());
    }

    (*dropped).activator = ent; //so we know who we belonged to so they can pick it back up later
    (*dropped).s.time = level.time; //mark this time so we aren't picked up instantly by the guy who dropped us
    return dropped;
}


/*
================
Use_Item

Respawn the item
================
*/
pub unsafe fn Use_Item(ent: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if ((*ent).svFlags & SVF_PLAYER_USABLE) != 0 && !other.is_null() && (*other).s.number == 0 {
        //used directly by the player, pick me up
        if ((*ent).spawnflags & ITMSF_USEPICKUP) != 0 {
            //player has to be touching me and hit use to pick it up, so don't allow this
            if G_BoundsOverlap(
                (*ent).absmin.as_ptr() as *const vec3_t,
                (*ent).absmax.as_ptr() as *const vec3_t,
                (*other).absmin.as_ptr() as *const vec3_t,
                (*other).absmax.as_ptr() as *const vec3_t,
            ) == 0
            {
                //not touching
                return;
            }
        }
        GEntity_TouchFunc(ent, other, core::ptr::null_mut());
    } else {
        //use me
        if ((*ent).spawnflags & 32) != 0 {
            // invisible
            // If it was invisible, first use makes it visible....
            (*ent).s.eFlags &= !EF_NODRAW;
            (*ent).contents = CONTENTS_TRIGGER | CONTENTS_ITEM;

            (*ent).spawnflags &= !32;
            return;
        }

        G_ActivateBehavior(ent, BSET_USE);
        RespawnItem(ent);
    }
}

//======================================================================

/*
================
FinishSpawningItem

Traces down to find where an item should rest, instead of letting them
free fall from their spawn points
================
*/
// delayedShutDown is extern "C" (originally inline in this function body in C; lifted above).
// g_saber is extern "C" (originally inline in this function body in C; lifted above).
pub unsafe fn FinishSpawningItem(ent: *mut gentity_t) {
    let mut tr: trace_t = core::mem::zeroed();
    let mut dest: vec3_t = [0.0; 3];
    let mut item: *const gitem_t;
    let mut itemNum: c_int;

    itemNum = 1;
    item = bg_itemlist.as_ptr().add(1);
    while !(*item).classname.is_null() {
        if strcmp((*item).classname, (*ent).classname) == 0 {
            break;
        }
        item = item.add(1);
        itemNum += 1;
    }

    // Set bounding box for item
    VectorSet(
        (*ent).mins.as_mut_ptr() as *mut vec3_t,
        (*item).mins[0],
        (*item).mins[1],
        (*item).mins[2],
    );
    VectorSet(
        (*ent).maxs.as_mut_ptr() as *mut vec3_t,
        (*item).maxs[0],
        (*item).maxs[1],
        (*item).maxs[2],
    );

    if ((*ent).mins[0] == 0.0 && (*ent).mins[1] == 0.0 && (*ent).mins[2] == 0.0)
        && ((*ent).maxs[0] == 0.0 && (*ent).maxs[1] == 0.0 && (*ent).maxs[2] == 0.0)
    {
        VectorSet(
            (*ent).mins.as_mut_ptr() as *mut vec3_t,
            -ITEM_RADIUS,
            -ITEM_RADIUS,
            -2.0,
        ); //to match the comments in the items.dat file!
        VectorSet(
            (*ent).maxs.as_mut_ptr() as *mut vec3_t,
            ITEM_RADIUS,
            ITEM_RADIUS,
            ITEM_RADIUS,
        );
    }

    if (*item).quantity != 0 && (*item).giType == IT_AMMO {
        (*ent).count = (*item).quantity;
    }

    if (*item).quantity != 0 && (*item).giType == IT_BATTERY {
        (*ent).count = (*item).quantity;
    }

    (*ent).s.radius = 20.0;
    VectorSet(
        (*ent).s.modelScale.as_mut_ptr() as *mut vec3_t,
        1.0_f32,
        1.0_f32,
        1.0_f32,
    );

    if (*(*ent).item).giType == IT_WEAPON
        && (*(*ent).item).giTag == WP_SABER
        && !(*ent).NPC_type.is_null()
        && *(*ent).NPC_type != 0
    {
        let mut itemSaber: saberInfo_t = core::mem::zeroed();
        if Q_stricmp(b"player\0".as_ptr() as *const c_char, (*ent).NPC_type) == 0
            && !(*g_saber).string.is_null()
            && *(*g_saber).string != 0
            && Q_stricmp(b"none\0".as_ptr() as *const c_char, (*g_saber).string) != 0
            && Q_stricmp(b"NULL\0".as_ptr() as *const c_char, (*g_saber).string) != 0
        {
            //player's saber
            WP_SaberParseParms((*g_saber).string, addr_of_mut!(itemSaber));
        } else {
            //specific saber
            WP_SaberParseParms((*ent).NPC_type, addr_of_mut!(itemSaber));
        }
        //NOTE:  should I keep this string around for any reason?  Will I ever need it later?
        //ent->??? = G_NewString( itemSaber.model );
        gi.G2API_InitGhoul2Model(
            (*ent).ghoul2,
            itemSaber.model,
            G_ModelIndex(itemSaber.model),
        );
        WP_SaberFreeStrings(itemSaber);
    } else {
        gi.G2API_InitGhoul2Model(
            (*ent).ghoul2,
            (*(*ent).item).world_model,
            G_ModelIndex((*(*ent).item).world_model),
        );
    }

    // Set crystal ammo amount based on skill level
    /* if ((itemNum == ITM_AMMO_CRYSTAL_BORG) ||
            (itemNum == ITM_AMMO_CRYSTAL_DN) ||
            (itemNum == ITM_AMMO_CRYSTAL_FORGE) ||
            (itemNum == ITM_AMMO_CRYSTAL_SCAVENGER) ||
            (itemNum == ITM_AMMO_CRYSTAL_STASIS))
        {
            CrystalAmmoSettings(ent);
        }
    */
    (*ent).s.eType = ET_ITEM;
    (*ent).s.modelindex = (*ent)
        .item
        .offset_from(addr_of!(bg_itemlist) as *const gitem_t)
        as c_int; // store item number in modelindex
    (*ent).s.modelindex2 = 0; // zero indicates this isn't a dropped item

    (*ent).contents = CONTENTS_TRIGGER | CONTENTS_ITEM; //CONTENTS_BODY;//CONTENTS_TRIGGER|
    (*ent).e_TouchFunc = touchF_Touch_Item;
    // useing an item causes it to respawn
    (*ent).e_UseFunc = useF_Use_Item;
    (*ent).svFlags |= SVF_PLAYER_USABLE; //so player can pick it up

    // Hang in air?
    (*ent).s.origin[2] += 1.0; //just to get it off the damn ground because coplanar = insolid
    if ((*ent).spawnflags & ITMSF_SUSPEND) != 0 || ((*ent).flags & FL_DROPPED_ITEM) != 0 {
        // suspended
        G_SetOrigin(ent, (*ent).s.origin.as_ptr() as *const vec3_t);
    } else {
        // drop to floor
        dest[0] = (*ent).s.origin[0];
        dest[1] = (*ent).s.origin[1];
        dest[2] = MIN_WORLD_COORD;
        gi.trace(
            addr_of_mut!(tr),
            (*ent).s.origin.as_ptr() as *const vec3_t,
            (*ent).mins.as_ptr() as *const vec3_t,
            (*ent).maxs.as_ptr() as *const vec3_t,
            addr_of!(dest),
            (*ent).s.number,
            MASK_SOLID | CONTENTS_PLAYERCLIP,
        );
        if tr.startsolid != 0 {
            if !addr_of_mut!(g_entities[tr.entityNum as usize]).is_null() {
                gi.Printf(
                    // S_COLOR_RED expanded as "^1" (porting note: C adjacent-string-literal concat)
                    b"^1FinishSpawningItem: removing %s startsolid at %s (in a %s)\n\0"
                        .as_ptr() as *const c_char,
                    (*ent).classname,
                    vtos((*ent).s.origin.as_ptr() as *const vec3_t),
                    g_entities[tr.entityNum as usize].classname,
                );
            } else {
                gi.Printf(
                    // S_COLOR_RED expanded as "^1" (porting note: C adjacent-string-literal concat)
                    b"^1FinishSpawningItem: removing %s startsolid at %s (in a %s)\n\0"
                        .as_ptr() as *const c_char,
                    (*ent).classname,
                    vtos((*ent).s.origin.as_ptr() as *const vec3_t),
                );
            }
            debug_assert!(false, "item starting in solid");
            if g_entities[ENTITYNUM_WORLD as usize].s.radius == 0.0 {
                //not a region
                delayedShutDown = level.time + 100;
            }
            G_FreeEntity(ent);
            return;
        }

        // allow to ride movers
        (*ent).s.groundEntityNum = tr.entityNum;

        G_SetOrigin(ent, tr.endpos.as_ptr() as *const vec3_t);
    }

    /* ? don't need this
        // team slaves and targeted items aren't present at start
        if ( ( ent->flags & FL_TEAMSLAVE ) || ent->targetname ) {
            ent->s.eFlags |= EF_NODRAW;
            ent->contents = 0;
            return;
        }
    */
    if ((*ent).spawnflags & ITMSF_INVISIBLE) != 0 {
        // invisible
        (*ent).s.eFlags |= EF_NODRAW;
        (*ent).contents = 0;
    }

    if ((*ent).spawnflags & ITMSF_NOTSOLID) != 0 {
        // not solid
        (*ent).contents = 0;
    }

    if ((*ent).spawnflags & ITMSF_STATIONARY) != 0 {
        //can't be pushed around
        (*ent).flags |= FL_NO_KNOCKBACK;
    }

    if ((*ent).flags & FL_DROPPED_ITEM) != 0 {
        //go away after 30 seconds
        (*ent).e_ThinkFunc = thinkF_G_FreeEntity;
        (*ent).nextthink = level.time + 30000;
    }

    gi.linkentity(ent);
}


pub static mut itemRegistered: [u8; MAX_ITEMS as usize + 1] = [0u8; MAX_ITEMS as usize + 1];


/*
==============
ClearRegisteredItems
==============
*/
pub unsafe fn ClearRegisteredItems() {
    core::ptr::write_bytes(
        addr_of_mut!(itemRegistered) as *mut u8,
        b'0',
        bg_numItems as usize,
    );
    itemRegistered[bg_numItems as usize] = 0;

    //these are given in g_client, ClientSpawn(), but MUST be registered HERE, BEFORE cgame starts.
    //RegisterItem( FindItemForWeapon( WP_NONE ) );	//has no item
    RegisterItem(FindItemForInventory(INV_ELECTROBINOCULARS));
    //RegisterItem( FindItemForInventory( INV_BACTA_CANISTER ));
    // saber or baton is cached in SP_info_player_deathmatch now.

    // Player_CacheFromPrevLevel is extern "C" (originally inline in this function body in C; lifted above).
    Player_CacheFromPrevLevel(); //reads from transition carry-over;
}

/*
===============
RegisterItem

The item will be added to the precache list
===============
*/
pub unsafe fn RegisterItem(item: *mut gitem_t) {
    if item.is_null() {
        G_Error(b"RegisterItem: NULL\0".as_ptr() as *const c_char);
    }
    itemRegistered[item.offset_from(addr_of!(bg_itemlist) as *const gitem_t) as usize] = b'1';
    gi.SetConfigstring(CS_ITEMS, addr_of!(itemRegistered) as *const c_char); //Write the needed items to a config string
}


/*
===============
SaveRegisteredItems

Write the needed items to a config string
so the client will know which ones to precache
===============
*/
pub unsafe fn SaveRegisteredItems() {
    /*	char	string[MAX_ITEMS+1];
        int		i;
        int		count;

        count = 0;
        for ( i = 0 ; i < bg_numItems ; i++ ) {
            if ( itemRegistered[i] ) {
                count++;
                string[i] = '1';
            } else {
                string[i] = '0';
            }
        }
        string[ bg_numItems ] = 0;

        gi.Printf( "%i items registered\n", count );
        gi.SetConfigstring(CS_ITEMS, string);
    */
    gi.SetConfigstring(CS_ITEMS, addr_of!(itemRegistered) as *const c_char);
}

/*
============
item_spawn_use

 if an item is given a targetname, it will be spawned in when used
============
*/
pub unsafe fn item_spawn_use(
    self_: *mut gentity_t,
    _other: *mut gentity_t,
    _activator: *mut gentity_t,
)
//-----------------------------------------------------------------------------
{
    (*self_).nextthink = level.time + 50;
    (*self_).e_ThinkFunc = thinkF_FinishSpawningItem;
    // I could be fancy and add a count or something like that to be able to spawn the item numerous times...
    (*self_).e_UseFunc = useF_NULL;
}

/*
============
G_SpawnItem

Sets the clipping size and plants the object on the floor.

Items can't be immediately dropped to floor, because they might
be on an entity that hasn't spawned yet.
============
*/
pub unsafe fn G_SpawnItem(ent: *mut gentity_t, item: *mut gitem_t) {
    G_SpawnFloat(
        b"random\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        addr_of_mut!((*ent).random),
    );
    G_SpawnFloat(
        b"wait\0".as_ptr() as *const c_char,
        b"0\0".as_ptr() as *const c_char,
        addr_of_mut!((*ent).wait),
    );

    RegisterItem(item);
    (*ent).item = item;

    // targetname indicates they want to spawn it later
    if !(*ent).targetname.is_null() {
        (*ent).e_UseFunc = useF_item_spawn_use;
    } else {
        // some movers spawn on the second frame, so delay item
        // spawns until the third frame so they can ride trains
        (*ent).nextthink = level.time + START_TIME_MOVERS_SPAWNED + 50;
        (*ent).e_ThinkFunc = thinkF_FinishSpawningItem;
    }

    (*ent).physicsBounce = 0.50; // items are bouncy

    // Set a default infoString text color
    // NOTE: if we want to do cool cross-hair colors for items, we can just modify this, but for now, don't do it
    VectorSet(
        (*ent).startRGBA.as_mut_ptr() as *mut vec3_t,
        1.0_f32,
        1.0_f32,
        1.0_f32,
    );

    if !(*ent).team.is_null() && *(*ent).team != 0 {
        (*ent).noDamageTeam =
            GetIDForString(addr_of!(TeamTable), (*ent).team) as team_t;
        if (*ent).noDamageTeam == TEAM_FREE {
            G_Error(
                b"team name %s not recognized\n\0".as_ptr() as *const c_char,
                (*ent).team,
            );
        }
    }

    if !(*ent).item.is_null()
        && (*(*ent).item).giType == IT_WEAPON
        && (*(*ent).item).giTag == WP_SABER
    {
        //weapon_saber item
        if (*ent).count == 0 {
            //can only pick up once
            (*ent).count = 1;
        }
    }
    (*ent).team = core::ptr::null_mut();
}


/*
================
G_BounceItem

================
*/
pub unsafe fn G_BounceItem(ent: *mut gentity_t, trace: *mut trace_t) {
    let mut velocity: vec3_t = [0.0; 3];
    let mut dot: f32;
    let mut hitTime: c_int;
    let mut droppedSaber: qboolean = qtrue;

    if !(*ent).item.is_null()
        && (*(*ent).item).giType == IT_WEAPON
        && (*(*ent).item).giTag == WP_SABER
        && ((*ent).flags & FL_DROPPED_ITEM) != 0
    {
        droppedSaber = qtrue;
    }

    // reflect the velocity on the trace plane
    hitTime = level.previousTime
        + ((level.time - level.previousTime) as f32 * (*trace).fraction) as c_int;
    EvaluateTrajectoryDelta(
        addr_of!((*ent).s.pos),
        hitTime,
        addr_of_mut!(velocity),
    );
    dot = DotProduct(
        addr_of!(velocity),
        addr_of!((*trace).plane.normal) as *const vec3_t,
    );
    VectorMA(
        addr_of!(velocity),
        -2.0 * dot,
        addr_of!((*trace).plane.normal) as *const vec3_t,
        (*ent).s.pos.trDelta.as_mut_ptr() as *mut vec3_t,
    );

    // cut the velocity to keep from bouncing forever
    VectorScale(
        (*ent).s.pos.trDelta.as_ptr() as *const vec3_t,
        (*ent).physicsBounce,
        (*ent).s.pos.trDelta.as_mut_ptr() as *mut vec3_t,
    );

    if droppedSaber != 0 {
        //a dropped saber item
        //FIXME: use NPC_type (as saberType) to get proper bounce sound?
        WP_SaberFallSound(core::ptr::null_mut(), ent);
    }

    // check for stop
    if (*trace).plane.normal[2] > 0.0 && (*ent).s.pos.trDelta[2] < 40.0 {
        //stop
        G_SetOrigin(ent, (*trace).endpos.as_ptr() as *const vec3_t);
        (*ent).s.groundEntityNum = (*trace).entityNum;
        if droppedSaber != 0 {
            //a dropped saber item
            //stop rotation
            VectorClear((*ent).s.apos.trDelta.as_mut_ptr() as *mut vec3_t);
            (*ent).currentAngles[PITCH as usize] = SABER_PITCH_HACK;
            (*ent).currentAngles[ROLL as usize] = 0.0;
            if !(*ent).NPC_type.is_null() && *(*ent).NPC_type != 0 {
                //we have a valid saber for this
                let mut saber: saberInfo_t = core::mem::zeroed();
                if WP_SaberParseParms((*ent).NPC_type, addr_of_mut!(saber)) != 0 {
                    if (saber.saberFlags & SFL_BOLT_TO_WRIST) != 0 {
                        (*ent).currentAngles[PITCH as usize] = 0.0;
                    }
                }
            }
            pitch_roll_for_slope(
                ent,
                (*trace).plane.normal.as_ptr() as *const vec3_t,
                (*ent).currentAngles.as_mut_ptr() as *mut vec3_t,
                qtrue,
            );
            G_SetAngles(ent, (*ent).currentAngles.as_ptr() as *const vec3_t);
        }
        return;
    }
    //bounce
    if droppedSaber != 0 {
        //a dropped saber item
        //change rotation
        VectorCopy(
            (*ent).currentAngles.as_ptr() as *const vec3_t,
            (*ent).s.apos.trBase.as_mut_ptr() as *mut vec3_t,
        );
        (*ent).s.apos.trType = TR_LINEAR;
        (*ent).s.apos.trTime = level.time;
        VectorSet(
            (*ent).s.apos.trDelta.as_mut_ptr() as *mut vec3_t,
            Q_irand(-300, 300) as f32,
            Q_irand(-300, 300) as f32,
            Q_irand(-300, 300) as f32,
        );
    }

    VectorAdd(
        (*ent).currentOrigin.as_ptr() as *const vec3_t,
        (*trace).plane.normal.as_ptr() as *const vec3_t,
        (*ent).currentOrigin.as_mut_ptr() as *mut vec3_t,
    );
    VectorCopy(
        (*ent).currentOrigin.as_ptr() as *const vec3_t,
        (*ent).s.pos.trBase.as_mut_ptr() as *mut vec3_t,
    );
    (*ent).s.pos.trTime = level.time;
}


/*
================
G_RunItem

================
*/
pub unsafe fn G_RunItem(ent: *mut gentity_t) {
    let mut origin: vec3_t = [0.0; 3];
    let mut tr: trace_t = core::mem::zeroed();
    let mut contents: c_int;
    let mut mask: c_int;

    // if groundentity has been set to -1, it may have been pushed off an edge
    if (*ent).s.groundEntityNum == ENTITYNUM_NONE {
        if (*ent).s.pos.trType != TR_GRAVITY {
            (*ent).s.pos.trType = TR_GRAVITY;
            (*ent).s.pos.trTime = level.time;
        }
    }

    if (*ent).s.pos.trType == TR_STATIONARY {
        // check think function
        G_RunThink(ent);
        if (*g_gravity).value == 0.0 {
            (*ent).s.pos.trType = TR_GRAVITY;
            (*ent).s.pos.trTime = level.time;
            (*ent).s.pos.trDelta[0] += crandom() * 40.0_f32; // I dunno, just do this??
            (*ent).s.pos.trDelta[1] += crandom() * 40.0_f32;
            (*ent).s.pos.trDelta[2] += random() * 20.0_f32;
        } else if ((*ent).flags & FL_DROPPED_ITEM) != 0
            && !(*ent).item.is_null()
            && (*(*ent).item).giType == IT_WEAPON
            && (*(*ent).item).giTag == WP_SABER
        {
            //a dropped saber item, check below, just in case
            let mut ignore: c_int = ENTITYNUM_NONE;
            if (*ent).clipmask != 0 {
                mask = (*ent).clipmask;
            } else {
                mask = MASK_SOLID | CONTENTS_PLAYERCLIP; //shouldn't be able to get anywhere player can't
            }
            if !(*ent).owner.is_null() {
                ignore = (*(*ent).owner).s.number;
            } else if !(*ent).activator.is_null() {
                ignore = (*(*ent).activator).s.number;
            }
            origin[0] = (*ent).currentOrigin[0];
            origin[1] = (*ent).currentOrigin[1];
            origin[2] = (*ent).currentOrigin[2] - 1.0;
            gi.trace(
                addr_of_mut!(tr),
                (*ent).currentOrigin.as_ptr() as *const vec3_t,
                (*ent).mins.as_ptr() as *const vec3_t,
                (*ent).maxs.as_ptr() as *const vec3_t,
                addr_of!(origin),
                ignore,
                mask,
            );
            if tr.allsolid == 0 && tr.startsolid == 0 && tr.fraction > 0.001_f32 {
                //wha?  fall....
                (*ent).s.pos.trType = TR_GRAVITY;
                (*ent).s.pos.trTime = level.time;
            }
        }
        return;
    }

    // get current position
    EvaluateTrajectory(addr_of!((*ent).s.pos), level.time, addr_of_mut!(origin));
    if (*ent).s.apos.trType != TR_STATIONARY {
        EvaluateTrajectory(
            addr_of!((*ent).s.apos),
            level.time,
            (*ent).currentAngles.as_mut_ptr() as *mut vec3_t,
        );
        G_SetAngles(ent, (*ent).currentAngles.as_ptr() as *const vec3_t);
    }

    // trace a line from the previous position to the current position
    if (*ent).clipmask != 0 {
        mask = (*ent).clipmask;
    } else {
        mask = MASK_SOLID | CONTENTS_PLAYERCLIP; //shouldn't be able to get anywhere player can't
    }

    let mut ignore: c_int = ENTITYNUM_NONE;
    if !(*ent).owner.is_null() {
        ignore = (*(*ent).owner).s.number;
    } else if !(*ent).activator.is_null() {
        ignore = (*(*ent).activator).s.number;
    }
    gi.trace(
        addr_of_mut!(tr),
        (*ent).currentOrigin.as_ptr() as *const vec3_t,
        (*ent).mins.as_ptr() as *const vec3_t,
        (*ent).maxs.as_ptr() as *const vec3_t,
        addr_of!(origin),
        ignore,
        mask,
    );

    VectorCopy(
        tr.endpos.as_ptr() as *const vec3_t,
        (*ent).currentOrigin.as_mut_ptr() as *mut vec3_t,
    );

    if tr.startsolid != 0 {
        tr.fraction = 0.0;
    }

    gi.linkentity(ent); // FIXME: avoid this for stationary?

    // check think function
    G_RunThink(ent);

    if tr.fraction == 1.0 {
        if (*g_gravity).value <= 0.0 {
            if (*ent).s.apos.trType != TR_LINEAR {
                VectorCopy(
                    (*ent).currentAngles.as_ptr() as *const vec3_t,
                    (*ent).s.apos.trBase.as_mut_ptr() as *mut vec3_t,
                );
                (*ent).s.apos.trType = TR_LINEAR;
                (*ent).s.apos.trDelta[1] = Q_flrand(-300.0, 300.0);
                (*ent).s.apos.trDelta[0] = Q_flrand(-10.0, 10.0);
                (*ent).s.apos.trDelta[2] = Q_flrand(-10.0, 10.0);
                (*ent).s.apos.trTime = level.time;
            }
        }
        //friction in zero-G
        if (*g_gravity).value == 0.0 {
            let friction: f32 = 0.975_f32;
            /*friction -= ent->mass/1000.0f;
            if ( friction < 0.1 )
            {
                friction = 0.1f;
            }
            */
            VectorScale(
                (*ent).s.pos.trDelta.as_ptr() as *const vec3_t,
                friction,
                (*ent).s.pos.trDelta.as_mut_ptr() as *mut vec3_t,
            );
            VectorCopy(
                (*ent).currentOrigin.as_ptr() as *const vec3_t,
                (*ent).s.pos.trBase.as_mut_ptr() as *mut vec3_t,
            );
            (*ent).s.pos.trTime = level.time;
        }
        return;
    }

    // if it is in a nodrop volume, remove it
    contents = gi.pointcontents((*ent).currentOrigin.as_ptr() as *const vec3_t, -1);
    if (contents & CONTENTS_NODROP) != 0 {
        G_FreeEntity(ent);
        return;
    }

    if tr.startsolid == 0 {
        G_BounceItem(ent, addr_of_mut!(tr));
    }
}

/*
================
ItemUse_Bacta

================
*/
pub unsafe fn ItemUse_Bacta(ent: *mut gentity_t) {
    if ent.is_null() || (*ent).client.is_null() {
        return;
    }

    if (*ent).health >= (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize]
        || (*(*ent).client).ps.inventory[INV_BACTA_CANISTER as usize] == 0
    {
        return;
    }

    (*ent).health += MAX_BACTA_HEAL_AMOUNT;

    if (*ent).health > (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize] {
        (*ent).health = (*(*ent).client).ps.stats[STAT_MAX_HEALTH as usize];
    }

    (*(*ent).client).ps.inventory[INV_BACTA_CANISTER as usize] -= 1;

    G_SoundOnEnt(
        ent,
        CHAN_VOICE,
        va(
            b"sound/weapons/force/heal%d_%c.mp3\0".as_ptr() as *const c_char,
            Q_irand(1, 4),
            *(*g_sex).string.add(0) as c_int,
        ),
    );
}
