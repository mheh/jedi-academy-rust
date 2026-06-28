//! Mechanical port of `code/ff/ff_MultiCompound.cpp`.
//!
//! Porting note: The original file was wrapped in `#ifdef _IMMERSION`, but this
//! code is translated unconditionally since _IMMERSION is not yet a Cargo feature.

#![allow(non_snake_case)]

use std::collections::BTreeSet;
use core::ffi::c_int;

// LOCAL STUB: qboolean type from engine
pub type qboolean = c_int;

// LOCAL STUB: DWORD type
pub type DWORD = c_int;

// LOCAL STUB: MultiEffect from ff_MultiEffect.h
pub struct MultiEffect {
    _private: (),
}

////-------------
///	MultiCompound
//-----------------
//	MultiCompound is a container for MultiEffect pointers.
//	It is not a single, complex effect and should not be treated as such.
//
#[repr(C)]
pub struct MultiCompound {
    // typedef set<MultiEffect*> Set;
    // Set mSet;
    mSet: BTreeSet<*mut MultiEffect>,
}

impl MultiCompound {
    pub fn new() -> Self {
        MultiCompound {
            mSet: BTreeSet::new(),
        }
    }

    pub fn new_with_compound(compound: &BTreeSet<*mut MultiEffect>) -> Self {
        let mut result = MultiCompound {
            mSet: BTreeSet::new(),
        };
        result.Add(compound);
        result
    }

    pub fn GetSet(&mut self) -> &mut BTreeSet<*mut MultiEffect> {
        &mut self.mSet
    }

    ////------------------
    ///	MultiCompound::Add
    //----------------------
    //	Insert a single compound effect if it does not already exist.
    //	Only fails when parameter is NULL.
    //
    pub fn Add(&mut self, effect: *mut MultiEffect) -> qboolean {
        if !effect.is_null() {
            self.mSet.insert(effect);
            1 // qtrue
        } else {
            0 // qfalse
        }
    }

    ////------------------
    ///	MultiCompound::Add
    //----------------------
    //	Merge set of compound effects with current set. NULL pointers are excluded.
    //	Returns false if set contains any NULL pointers.
    //
    pub fn AddSet(&mut self, effect: &BTreeSet<*mut MultiEffect>) -> qboolean {
        let mut result: qboolean = 1; // qtrue

        for itSet in effect.iter() {
            result &= self.Add(*itSet);
        }

        result
    }

    ////--------------------
    ///	MultiCompound::Start
    //------------------------
    //	Analogous to CImmCompoundEffect::Start. Starts all contained compound effects.
    //	Returns false if any effect returns false.
    //
    pub fn Start(&mut self) -> qboolean {
        let mut result: qboolean = 1; // qtrue

        for itSet in self.mSet.iter() {
            // SAFETY: We're dereferencing a raw pointer from the set. The original C++ code
            // assumes these pointers are valid. This is a mechanical port that preserves
            // the original dangerous behavior.
            if !itSet.is_null() {
                unsafe {
                    result &= (*itSet).Start();
                }
            }
        }

        if result != 0 && self.mSet.len() != 0 {
            1 // qtrue
        } else {
            0 // qfalse
        }
    }

    pub fn IsPlaying(&mut self) -> qboolean {
        for itSet in self.mSet.iter() {
            // SAFETY: We're dereferencing a raw pointer from the set. The original C++ code
            // assumes these pointers are valid. This is a mechanical port that preserves
            // the original dangerous behavior.
            if !itSet.is_null() {
                unsafe {
                    if (*itSet).IsPlaying() == 0 {
                        return 0; // qfalse
                    }
                }
            }
        }

        1 // qtrue
    }

    ////----------------------------
    ///	MultiCompound::EnsurePlaying
    //--------------------------------
    //	Starts any contained compound effects if they are not currently playing.
    //	Returns false if any effect returns false or any are playing.
    //
    pub fn EnsurePlaying(&mut self) -> qboolean {
        let mut result: qboolean = 1; // qtrue

        if self.IsPlaying() == 0 {
            for itSet in self.mSet.iter() {
                // SAFETY: We're dereferencing a raw pointer from the set. The original C++ code
                // assumes these pointers are valid. This is a mechanical port that preserves
                // the original dangerous behavior.
                if !itSet.is_null() {
                    unsafe {
                        result &= (*itSet).Start();
                    }
                }
            }
        }

        if result != 0 && self.mSet.len() != 0 {
            1 // qtrue
        } else {
            0 // qfalse
        }
    }

    ////-------------------
    ///	MultiCompound::Stop
    //-----------------------
    //	Analogous to CImmCompoundEffect::Stop. Stops all contained compound effects.
    //	Returns false if any effect returns false.
    //
    pub fn Stop(&mut self) -> qboolean {
        let mut result: qboolean = 1; // qtrue

        for itSet in self.mSet.iter() {
            // SAFETY: We're dereferencing a raw pointer from the set. The original C++ code
            // assumes these pointers are valid. This is a mechanical port that preserves
            // the original dangerous behavior.
            if !itSet.is_null() {
                unsafe {
                    result &= (*itSet).Stop() as qboolean;
                }
            }
        }

        if result != 0 && self.mSet.len() != 0 {
            1 // qtrue
        } else {
            0 // qfalse
        }
    }

    ////-----------------------------
    ///	MultiCompound::ChangeDuration
    //---------------------------------
    //	Changes duration of all compounds.
    //	Returns false if any effect returns false.
    //
    pub fn ChangeDuration(&mut self, Duration: DWORD) -> qboolean {
        let mut result: qboolean = 1; // qtrue

        for itSet in self.mSet.iter() {
            // SAFETY: We're dereferencing a raw pointer from the set. The original C++ code
            // assumes these pointers are valid. This is a mechanical port that preserves
            // the original dangerous behavior.
            if !itSet.is_null() {
                unsafe {
                    result &= (*itSet).ChangeDuration(Duration);
                }
            }
        }

        if result != 0 && self.mSet.len() != 0 {
            1 // qtrue
        } else {
            0 // qfalse
        }
    }

    ////-------------------------
    ///	MultiCompound::ChangeGain
    //-----------------------------
    //	Changes gain of all compounds.
    //	Returns false if any effect returns false.
    //
    pub fn ChangeGain(&mut self, Gain: DWORD) -> qboolean {
        let mut result: qboolean = 1; // qtrue

        for itSet in self.mSet.iter() {
            // SAFETY: We're dereferencing a raw pointer from the set. The original C++ code
            // assumes these pointers are valid. This is a mechanical port that preserves
            // the original dangerous behavior.
            if !itSet.is_null() {
                unsafe {
                    result &= (*itSet).ChangeGain(Gain);
                }
            }
        }

        if result != 0 && self.mSet.len() != 0 {
            1 // qtrue
        } else {
            0 // qfalse
        }
    }

    ////--------------------------
    ///	MultiCompound::operator ==
    //------------------------------
    //	Returns qtrue if the sets are EXACTLY equal, including order. This is not good
    //	in general. (Fix me)
    //
    pub fn Equals(&self, compound: &MultiCompound) -> qboolean {
        let other = &compound.mSet;
        let mut result: qboolean = 0; // qfalse

        if self.mSet.len() == other.len() {
            let mut itSet = self.mSet.iter();
            let mut itOther = other.iter();
            let mut all_equal = true;

            loop {
                match (itSet.next(), itOther.next()) {
                    (None, None) => {
                        result = 1; // qtrue
                        break;
                    }
                    (Some(a), Some(b)) => {
                        if a != b {
                            all_equal = false;
                            break;
                        }
                    }
                    _ => {
                        all_equal = false;
                        break;
                    }
                }
            }

            if all_equal {
                result = 1; // qtrue
            }
        }

        result
    }
}
