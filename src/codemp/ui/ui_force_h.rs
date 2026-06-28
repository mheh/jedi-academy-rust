//! Mechanical port of `codemp/ui/ui_force.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{vec_t, NUM_FORCE_POWERS, NUM_SABER_COLORS};
use crate::ffi::types::{qboolean, vmCvar_t};
use core::ffi::{c_char, c_int, c_void};

pub const NUM_FORCE_STAR_IMAGES: c_int = 9;
pub const FORCE_NONJEDI: c_int = 0;
pub const FORCE_JEDI: c_int = 1;

// Header-local opaque stub for ui_shared.h's `rectDef_t`, not yet ported in `src/`.
pub type rectDef_t = c_void;

unsafe extern "C" {
    pub static mut uiForceSide: c_int;
    pub static mut uiJediNonJedi: c_int;
    pub static mut uiForceRank: c_int;
    pub static mut uiMaxRank: c_int;
    pub static mut uiForceUsed: c_int;
    pub static mut uiForceAvailable: c_int;
    pub static mut gTouchedForce: qboolean;
    pub static mut uiForcePowersDisabled: [qboolean; NUM_FORCE_POWERS];
    pub static mut uiForcePowersRank: [c_int; NUM_FORCE_POWERS];
    pub static mut uiForcePowerDarkLight: [c_int; NUM_FORCE_POWERS];
    pub static mut uiSaberColorShaders: [c_int; NUM_SABER_COLORS as usize];
    // Dots above or equal to a given rank carry a certain color.
    pub static mut ui_freeSaber: vmCvar_t;
    pub static mut ui_forcePowerDisable: vmCvar_t;

    pub fn UI_InitForceShaders();
    pub fn UI_ReadLegalForce();
    pub fn UI_DrawTotalForceStars(
        rect: *mut rectDef_t,
        scale: f32,
        color: *mut vec_t,
        textStyle: c_int,
    );
    pub fn UI_DrawForceStars(
        rect: *mut rectDef_t,
        scale: f32,
        color: *mut vec_t,
        textStyle: c_int,
        findex: c_int,
        val: c_int,
        min: c_int,
        max: c_int,
    );
    pub fn UI_UpdateClientForcePowers(teamArg: *const c_char);
    pub fn UI_SaveForceTemplate();
    pub fn UI_UpdateForcePowers();
    pub fn UI_SkinColor_HandleKey(
        flags: c_int,
        special: *mut f32,
        key: c_int,
        num: c_int,
        min: c_int,
        max: c_int,
        r#type: c_int,
    ) -> qboolean;
    pub fn UI_ForceSide_HandleKey(
        flags: c_int,
        special: *mut f32,
        key: c_int,
        num: c_int,
        min: c_int,
        max: c_int,
        r#type: c_int,
    ) -> qboolean;
    pub fn UI_JediNonJedi_HandleKey(
        flags: c_int,
        special: *mut f32,
        key: c_int,
        num: c_int,
        min: c_int,
        max: c_int,
        r#type: c_int,
    ) -> qboolean;
    pub fn UI_ForceMaxRank_HandleKey(
        flags: c_int,
        special: *mut f32,
        key: c_int,
        num: c_int,
        min: c_int,
        max: c_int,
        r#type: c_int,
    ) -> qboolean;
    pub fn UI_ForcePowerRank_HandleKey(
        flags: c_int,
        special: *mut f32,
        key: c_int,
        num: c_int,
        min: c_int,
        max: c_int,
        r#type: c_int,
    ) -> qboolean;
    pub fn UI_ForceConfigHandle(oldindex: c_int, newindex: c_int);
}
