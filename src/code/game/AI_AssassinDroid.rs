// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void, c_float};

// Type definitions for structures needed
pub type vec3_t = [f32; 3];
pub type qboolean = c_int;

// Stub type definitions (detailed definitions in game headers)
#[repr(C)]
pub struct playerState_t {
    pub stats: [c_int; 16],
    pub powerups: [c_int; 16],
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub eFlags: c_int,
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct renderInfo_t {
    pub customRGBA: [u8; 4],
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct entityState_t {
    pub eType: c_int,
    pub eFlags: c_int,
    pub powerups: c_int,
    pub weapon: c_int,
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
    pub renderInfo: renderInfo_t,
    pub NPC_class: c_int,
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct gNPC_t {
    pub touchedByPlayer: *mut gentity_t,
    pub enemyLastSeenTime: c_int,
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct gentity_t {
    pub flags: c_int,
    pub health: c_int,
    pub currentOrigin: vec3_t,
    pub client: *mut gclient_t,
    pub enemy: *mut gentity_t,
    pub s: entityState_t,
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct usercmd_t {
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct CGhoul2Info {
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct gameImport_t {
    _dummy: [u8; 0],
}

//custom anims:
    //both_attack1 - running attack
    //both_attack2 - crouched attack
    //both_attack3 - standing attack
    //both_stand1idle1 - idle
    //both_crouch2stand1 - uncrouch
    //both_death4 - running death

const ASSASSIN_SHIELD_SIZE: f32 = 75.0;
const TURN_ON: u32 = 0x00000000;
const TURN_OFF: u32 = 0x00000100;

// Constants
const FL_SHIELDED: c_int = 0x00000001;
const PW_GALAK_SHIELD: c_int = 7;
const PW_SHOCKED: c_int = 9;
const STAT_ARMOR: c_int = 3;
const DAMAGE_NO_KNOCKBACK: c_int = 0x00000008;
const MOD_ELECTROCUTE: c_int = 31;
const Q3_INFINITE: c_int = 16777216;
const HL_NONE: c_int = 0;

// External declarations - game engine functions
extern "C" {
    pub fn G_Damage(targ: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *const vec3_t, point: *const vec3_t, damage: c_int, dflags: c_int, mod_: c_int, hitLoc: c_int);
    pub fn G_Throw(targ: *mut gentity_t, newDir: *const vec3_t, push: f32);
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, out: *mut vec3_t);
    pub fn VectorNormalize(vec: *mut vec3_t) -> f32;
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn TIMER_Done(ent: *mut gentity_t, timer: *const c_char) -> qboolean;
    pub fn TIMER_Set(ent: *mut gentity_t, timer: *const c_char, duration: c_int);

    // Global game state
    pub static mut NPC: *mut gentity_t;
    pub static mut NPCInfo: *mut gNPC_t;
    pub static mut level: level_locals_t;
    pub static mut g_spskill: *mut cvar_t;
    pub static mut gi: gameImport_t;
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    _dummy: [u8; 0],
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_IsOn() -> bool {
    ((*NPC).flags & FL_SHIELDED) != 0
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_TurnOn() {
    if !BubbleShield_IsOn() {
        (*NPC).flags |= FL_SHIELDED;
        (*(*NPC).client).ps.powerups[PW_GALAK_SHIELD as usize] = Q3_INFINITE;
        // gi.G2API_SetSurfaceOnOff( &NPC->ghoul2[NPC->playerModel], "force_shield", TURN_ON );
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_TurnOff() {
    if BubbleShield_IsOn() {
        (*NPC).flags &= !FL_SHIELDED;
        (*(*NPC).client).ps.powerups[PW_GALAK_SHIELD as usize] = 0;
        // gi.G2API_SetSurfaceOnOff( &NPC->ghoul2[NPC->playerModel], "force_shield", TURN_OFF );
    }
}


////////////////////////////////////////////////////////////////////////////////////////
// Push A Particular Ent
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_PushEnt(pushed: *mut gentity_t, smackDir: vec3_t) {
    G_Damage(pushed, NPC, NPC, &smackDir, (*NPC).currentOrigin.as_ptr(), ((*g_spskill).integer+1)*Q_irand( 5, 10), DAMAGE_NO_KNOCKBACK, MOD_ELECTROCUTE, HL_NONE);
    G_Throw(pushed, &smackDir, 10.0);

    // Make Em Electric
    //------------------
    (*pushed).s.powerups |= (1 << PW_SHOCKED);
    if !(*pushed).client.is_null() {
        (*(*pushed).client).ps.powerups[PW_SHOCKED as usize] = level.time + 1000;
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// Go Through All The Ents Within The Radius Of The Shield And Push Them
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_PushRadiusEnts() {
    let mut radiusEnts: [*mut gentity_t; 128] = [core::ptr::null_mut(); 128];
    let radius = ASSASSIN_SHIELD_SIZE;
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut smackDir: vec3_t = [0.0; 3];
    let mut smackDist: f32;

    for i in 0..3 {
        mins[i] = (*NPC).currentOrigin[i] - radius;
        maxs[i] = (*NPC).currentOrigin[i] + radius;
    }

    // PORTING: gi.EntitiesInBox would be called here, but gi needs proper struct definition
    // The function pointer would be: (gi.EntitiesInBox)(mins.as_ptr(), maxs.as_ptr(), radiusEnts.as_mut_ptr(), 128)
    let numEnts: c_int = 0; // PORTING: stub - needs gi.EntitiesInBox to be properly linked

    for entIndex in 0..numEnts {
        // Only Clients
        //--------------
        if radiusEnts[entIndex as usize].is_null() || (*radiusEnts[entIndex as usize]).client.is_null() {
            continue;
        }

        // Don't Push Away Other Assassin Droids
        //---------------------------------------
        if (*(*radiusEnts[entIndex as usize]).client).NPC_class == (*(*NPC).client).NPC_class {
            continue;
        }

        // Should Have Already Pushed The Enemy If He Touched Us
        //-------------------------------------------------------
        if !NPC.is_null() && !NPCInfo.is_null() && !(*NPC).enemy.is_null() {
            if (*NPCInfo).touchedByPlayer == (*NPC).enemy && radiusEnts[entIndex as usize] == (*NPC).enemy {
                continue;
            }
        }

        // Do The Vector Distance Test
        //-----------------------------
        VectorSubtract((*radiusEnts[entIndex as usize]).currentOrigin.as_ptr(), (*NPC).currentOrigin.as_ptr(), &mut smackDir);
        smackDist = VectorNormalize(&mut smackDir);
        if smackDist < radius {
            BubbleShield_PushEnt(radiusEnts[entIndex as usize], smackDir);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
pub unsafe fn BubbleShield_Update() {
    // Shields Go When You Die
    //-------------------------
    if (*NPC).health <= 0 {
        if BubbleShield_IsOn() {
            BubbleShield_TurnOff();
        }
        return;
    }


    // Recharge Shields
    //------------------
    (*(*NPC).client).ps.stats[STAT_ARMOR as usize] += 1;
    if (*(*NPC).client).ps.stats[STAT_ARMOR as usize] > 250 {
        (*(*NPC).client).ps.stats[STAT_ARMOR as usize] = 250;
    }




    // If We Have Enough Armor And Are Not Shooting Right Now, Kick The Shield On
    //----------------------------------------------------------------------------
    if (*(*NPC).client).ps.stats[STAT_ARMOR as usize] > 100 && TIMER_Done(NPC, b"ShieldsDown\0".as_ptr() as *const c_char) != 0 {
        // Check On Timers To Raise And Lower Shields
        //--------------------------------------------
        if (level.time - (*NPCInfo).enemyLastSeenTime) < 1000 && TIMER_Done(NPC, b"ShieldsUp\0".as_ptr() as *const c_char) != 0 {
            TIMER_Set(NPC, b"ShieldsDown\0".as_ptr() as *const c_char, 2000);		// Drop Shields
            TIMER_Set(NPC, b"ShieldsUp\0".as_ptr() as *const c_char, Q_irand(4000, 5000));	// Then Bring Them Back Up For At Least 3 sec
        }

        BubbleShield_TurnOn();
        if BubbleShield_IsOn() {
            // Update Our Shader Value
            //-------------------------
            let rgba_value = ((*(*NPC).client).ps.stats[STAT_ARMOR as usize] - 100) as u8;
            (*(*NPC).client).renderInfo.customRGBA[0] = rgba_value;
            (*(*NPC).client).renderInfo.customRGBA[1] = rgba_value;
            (*(*NPC).client).renderInfo.customRGBA[2] = rgba_value;
            (*(*NPC).client).renderInfo.customRGBA[3] = rgba_value;


            // If Touched By An Enemy, ALWAYS Shove Them
            //-------------------------------------------
            if !(*NPC).enemy.is_null() && (*NPCInfo).touchedByPlayer == (*NPC).enemy {
                let mut dir: vec3_t = [0.0; 3];
                VectorSubtract((*(*NPC).enemy).currentOrigin.as_ptr(), (*NPC).currentOrigin.as_ptr(), &mut dir);
                VectorNormalize(&mut dir);
                BubbleShield_PushEnt((*NPC).enemy, dir);
            }

            // Push Anybody Else Near
            //------------------------
            BubbleShield_PushRadiusEnts();
        }
    }


    // Shields Gone
    //--------------
    else {
        BubbleShield_TurnOff();
    }
}
