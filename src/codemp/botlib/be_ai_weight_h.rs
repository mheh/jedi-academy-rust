
/*****************************************************************************
 * name:		be_ai_weight.h
 *
 * desc:		fuzzy weights
 *
 * $Archive: /source/code/botlib/be_ai_weight.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

use core::ffi::{c_char, c_int};

pub const WT_BALANCE: c_int = 1;
pub const MAX_WEIGHTS: c_int = 128;
pub const MAX_QPATH: usize = 64;

//fuzzy seperator
#[repr(C)]
pub struct fuzzyseperator_s {
    pub index: c_int,
    pub value: c_int,
    pub r#type: c_int,
    pub weight: f32,
    pub minweight: f32,
    pub maxweight: f32,
    pub child: *mut fuzzyseperator_s,
    pub next: *mut fuzzyseperator_s,
}

pub type fuzzyseperator_t = fuzzyseperator_s;

//fuzzy weight
#[repr(C)]
pub struct weight_s {
    pub name: *mut c_char,
    pub firstseperator: *mut fuzzyseperator_s,
}

pub type weight_t = weight_s;

//weight configuration
#[repr(C)]
pub struct weightconfig_s {
    pub numweights: c_int,
    pub weights: [weight_s; 128],
    pub filename: [c_char; 64],
}

pub type weightconfig_t = weightconfig_s;

//reads a weight configuration
extern "C" {
    pub fn ReadWeightConfig(filename: *mut c_char) -> *mut weightconfig_t;
    //free a weight configuration
    pub fn FreeWeightConfig(config: *mut weightconfig_t);
    //writes a weight configuration, returns true if successfull
    pub fn WriteWeightConfig(filename: *mut c_char, config: *mut weightconfig_t) -> c_int;
    //find the fuzzy weight with the given name
    pub fn FindFuzzyWeight(wc: *mut weightconfig_t, name: *mut c_char) -> c_int;
    //returns the fuzzy weight for the given inventory and weight
    pub fn FuzzyWeight(inventory: *mut c_int, wc: *mut weightconfig_t, weightnum: c_int) -> f32;
    pub fn FuzzyWeightUndecided(inventory: *mut c_int, wc: *mut weightconfig_t, weightnum: c_int) -> f32;
    //scales the weight with the given name
    pub fn ScaleWeight(config: *mut weightconfig_t, name: *mut c_char, scale: f32);
    //scale the balance range
    pub fn ScaleBalanceRange(config: *mut weightconfig_t, scale: f32);
    //evolves the weight configuration
    pub fn EvolveWeightConfig(config: *mut weightconfig_t);
    //interbreed the weight configurations and stores the interbreeded one in configout
    pub fn InterbreedWeightConfigs(config1: *mut weightconfig_t, config2: *mut weightconfig_t, configout: *mut weightconfig_t);
    //frees cached weight configurations
    pub fn BotShutdownWeights();
}
