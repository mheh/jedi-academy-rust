// this include must remain at the top of every CPP file

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
#![allow(unused_variables, dead_code, unused_unsafe, unused_mut, unused_imports)]
#![allow(clippy::all)]

// #if !defined(FX_SCHEDULER_H_INC)
//     #include "FxScheduler.h"
// #endif
use crate::code::cgame::common_headers_h::*;
use crate::code::cgame::FxScheduler_h::*;

// #if !defined(GHOUL2_SHARED_H_INC)
//     #include "..\game\ghoul2_shared.h"    //for CGhoul2Info_v
// #endif
use crate::code::game::ghoul2_shared_h::*;

// #if !defined(G2_H_INC)
//     #include "../ghoul2/G2.h"
// #endif
use crate::code::ghoul2::G2_h::*;

// #if !defined(__Q_SHARED_H)
//     #include "../game/q_shared.h"
// #endif
use crate::code::game::q_shared_h::*;

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// Porting note: In C++, `CFxScheduler theFxScheduler;` is the global definition that
// calls the constructor.  In the Rust port the global is already declared and initialised
// in FxScheduler_h as `Option<Box<CFxScheduler>>`.  We import it here via the glob above.

// don't even ask,. it's to do with loadsave...
//
pub static mut g_vstrEffectsNeededPerSlot: Vec<fxString_t> = Vec::new();
pub static mut gLoopedEffectArray: [SLoopedEffect; MAX_LOOPED_FX] = [SLoopedEffect {
    mId: 0,
    mBoltInfo: 0,
    mNextTime: 0,
    mLoopStopTime: 0,
    mPortalEffect: false,
    mIsRelative: false,
}; MAX_LOOPED_FX];

// CFxScheduler constructor (C++: CFxScheduler::CFxScheduler())
// Porting note: zero-initialisation of mEffectTemplates and mLoopedEffectArray is
// already handled by CFxScheduler::new() in FxScheduler_h.  This impl block provides
// the additional per-method bodies that the header stubs out.
impl CFxScheduler {
    // CFxScheduler::CFxScheduler()
    pub unsafe fn CFxScheduler_ctor(&mut self) {
        // memset( &mEffectTemplates, 0, sizeof( mEffectTemplates ));
        core::ptr::write_bytes(addr_of_mut!(self.mEffectTemplates), 0u8, 1);
        // memset( &mLoopedEffectArray, 0, sizeof( mLoopedEffectArray ));
        core::ptr::write_bytes(addr_of_mut!(self.mLoopedEffectArray), 0u8, 1);
    }

    pub unsafe fn FX_CopeWithAnyLoadedSaveGames(&mut self) {
        if !(*addr_of!(g_vstrEffectsNeededPerSlot)).is_empty() {
            // memcpy( mLoopedEffectArray, gLoopedEffectArray, sizeof(mLoopedEffectArray) );
            core::ptr::copy_nonoverlapping(
                addr_of!(gLoopedEffectArray),
                addr_of_mut!(self.mLoopedEffectArray),
                1,
            );
            // assert( g_vstrEffectsNeededPerSlot.size() == MAX_LOOPED_FX );
            debug_assert!((*addr_of!(g_vstrEffectsNeededPerSlot)).len() == MAX_LOOPED_FX);

            for iFX in 0..(*addr_of!(g_vstrEffectsNeededPerSlot)).len() {
                let psFX_Filename: *const c_char =
                    (*addr_of!(g_vstrEffectsNeededPerSlot))[iFX].c_str();
                if *psFX_Filename != 0 {
                    // register it...
                    //
                    self.mLoopedEffectArray[iFX].mId =
                        self.RegisterEffect(psFX_Filename, false);
                    //
                    // cope with any relative stop time...
                    //
                    if self.mLoopedEffectArray[iFX].mLoopStopTime != 0 {
                        self.mLoopedEffectArray[iFX].mLoopStopTime -=
                            self.mLoopedEffectArray[iFX].mNextTime;
                    }
                    //
                    // and finally reset the time to be the newly-zeroed game time...
                    //
                    self.mLoopedEffectArray[iFX].mNextTime = 0; // otherwise it won't process until game time catches up
                } else {
                    self.mLoopedEffectArray[iFX].mId = 0;
                }
            }

            (*addr_of_mut!(g_vstrEffectsNeededPerSlot)).clear();
        }
    }

    pub unsafe fn LoadSave_Read(&mut self) {
        self.Clean(true, 0); // need to get rid of old pre-cache handles, or it thinks it has some older effects when it doesn't
        (*addr_of_mut!(g_vstrEffectsNeededPerSlot)).clear(); // jic
        (*addr_of_mut!(gi)).ReadFromSaveGame(
            i32::from_be_bytes(*b"FXLE") as c_int,
            addr_of_mut!(gLoopedEffectArray) as *mut c_void,
            core::mem::size_of_val(&*addr_of!(gLoopedEffectArray)) as c_int,
        );
        //
        // now read in and re-register the effects we need for those structs...
        //
        for iFX in 0..MAX_LOOPED_FX {
            let mut sFX_Filename: [c_char; MAX_QPATH] = core::mem::zeroed();
            (*addr_of_mut!(gi)).ReadFromSaveGame(
                i32::from_be_bytes(*b"FXFN") as c_int,
                sFX_Filename.as_mut_ptr() as *mut c_void,
                core::mem::size_of_val(&sFX_Filename) as c_int,
            );
            (*addr_of_mut!(g_vstrEffectsNeededPerSlot))
                .push(fxString_t::new_from_str(sFX_Filename.as_ptr()));
        }
    }

    pub unsafe fn LoadSave_Write(&mut self) {
        // bsave the data we need...
        //
        (*addr_of_mut!(gi)).AppendToSaveGame(
            i32::from_be_bytes(*b"FXLE") as c_int,
            addr_of_mut!(self.mLoopedEffectArray) as *mut c_void,
            core::mem::size_of_val(&self.mLoopedEffectArray) as c_int,
        );
        //
        // then cope with the fact that the mID field in each struct of the array we've just saved will not
        //  necessarily point at the same thing when reloading, so save out the actual fx filename strings they
        //  need for re-registration...
        //
        // since this is only for savegames, and I've got < 2 hours to finish this and test it I'm going to be lazy
        //  with the ondisk data... (besides, the RLE compression will kill most of this anyway)
        //
        for iFX in 0..MAX_LOOPED_FX {
            let mut sFX_Filename: [c_char; MAX_QPATH] = core::mem::zeroed();
            // instead of "sFX_Filename[0]=0;" so RLE will squash whole array to nothing, not just stop at '\0' then have old crap after it to compress
            core::ptr::write_bytes(
                sFX_Filename.as_mut_ptr(),
                0u8,
                core::mem::size_of_val(&sFX_Filename),
            );

            let iID: c_int = self.mLoopedEffectArray[iFX].mId;
            if iID != 0 {
                // now we need to look up what string this represents, unfortunately the existing
                //  lookup table is backwards (keywise) for our needs, so parse the whole thing...
                //
                for (key, val) in self.mEffectIDs.iter() {
                    if *val == iID {
                        Q_strncpyz(
                            sFX_Filename.as_mut_ptr(),
                            key.c_str(),
                            core::mem::size_of_val(&sFX_Filename) as c_int,
                        );
                        break;
                    }
                }
            }

            // write out this string...
            //
            (*addr_of_mut!(gi)).AppendToSaveGame(
                i32::from_be_bytes(*b"FXFN") as c_int,
                sFX_Filename.as_mut_ptr() as *mut c_void,
                core::mem::size_of_val(&sFX_Filename) as c_int,
            );
        }
    }
}

//-----------------------------------------------------------
impl CMediaHandles {
    pub unsafe fn assign(&mut self, that: &CMediaHandles) {
        self.mMediaList.clear();

        for i in 0..that.mMediaList.len() {
            self.mMediaList.push(that.mMediaList[i]);
        }
    }
}

//------------------------------------------------------
impl SEffectTemplate {
    pub unsafe fn assign(&mut self, that: &SEffectTemplate) {
        self.mCopy = true;

        // strcpy( mEffectName, that.mEffectName );
        // mEffectName is [u8; MAX_QPATH]; copy byte-by-byte
        core::ptr::copy_nonoverlapping(
            that.mEffectName.as_ptr(),
            self.mEffectName.as_mut_ptr(),
            MAX_QPATH,
        );

        self.mPrimitiveCount = that.mPrimitiveCount;

        for i in 0..self.mPrimitiveCount as usize {
            self.mPrimitives[i] =
                Box::into_raw(Box::new(CPrimitiveTemplate::new()));
            // *(mPrimitives[i]) = *(that.mPrimitives[i]);
            // C++ compiler-generated copy assignment; uses field-wise copy including CMediaHandles::operator=.
            // Porting note: CPrimitiveTemplate must implement Clone for this; trusting import provides it.
            *self.mPrimitives[i] = (*that.mPrimitives[i]).clone();
            // Mark use as a copy so that we know that we should be chucked when used up
            (*self.mPrimitives[i]).mCopy = true;
        }
    }
}

impl CFxScheduler {
    pub unsafe fn ScheduleLoopedEffect(
        &mut self,
        id: c_int,
        boltInfo: c_int,
        isPortal: bool,
        iLoopTime: c_int,
        isRelative: bool,
    ) -> c_int {
        let mut i: usize;

        debug_assert!(id != 0);
        debug_assert!(boltInfo != -1);

        // see if it's already playing so we can just update it
        i = 0;
        loop {
            if i >= MAX_LOOPED_FX {
                break;
            }
            if self.mLoopedEffectArray[i].mId == id
                && self.mLoopedEffectArray[i].mBoltInfo == boltInfo
                && self.mLoopedEffectArray[i].mPortalEffect == isPortal
            {
                #[cfg(debug_assertions)]
                {
                    // theFxHelper.Print( "CFxScheduler::ScheduleLoopedEffect- updating %s\n", mEffectTemplates[id].mEffectName);
                }
                break;
            }
            i += 1;
        }

        if i == MAX_LOOPED_FX {
            // didn't find it existing, so find a free spot
            i = 0;
            loop {
                if i >= MAX_LOOPED_FX {
                    break;
                }
                if self.mLoopedEffectArray[i].mId == 0 {
                    break;
                }
                i += 1;
            }
        }

        if i == MAX_LOOPED_FX {
            //bad
            debug_assert!(i != MAX_LOOPED_FX);
            theFxHelper.Print(
                b"CFxScheduler::AddLoopedEffect- No Free Slots available for %d\n\0".as_ptr()
                    as *const c_char,
                (*addr_of!(self.mEffectTemplates))[id as usize].mEffectName.as_ptr(),
            );
            return -1;
        }
        self.mLoopedEffectArray[i].mId = id;
        self.mLoopedEffectArray[i].mBoltInfo = boltInfo;
        self.mLoopedEffectArray[i].mPortalEffect = isPortal;
        self.mLoopedEffectArray[i].mIsRelative = isRelative;
        self.mLoopedEffectArray[i].mNextTime =
            theFxHelper.mTime + self.mEffectTemplates[id as usize].mRepeatDelay;
        self.mLoopedEffectArray[i].mLoopStopTime = if iLoopTime == 1 {
            0
        } else {
            theFxHelper.mTime + iLoopTime
        };
        i as c_int
    }

    pub unsafe fn StopEffect(
        &mut self,
        file: *const c_char,
        boltInfo: c_int,
        isPortal: bool,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = core::mem::zeroed();

        // Get an extenstion stripped version of the file
        COM_StripExtension(file, sfile.as_mut_ptr());
        let sfile_key = fxString_t::new_from_str(sfile.as_ptr());
        let id: c_int = *self.mEffectIDs.entry(sfile_key).or_insert(0);
        #[cfg(not(feature = "final_build"))]
        {
            if id == 0 {
                theFxHelper.Print(
                    b"CFxScheduler::StopEffect- unregistered/non-existent effect: %s\n\0"
                        .as_ptr() as *const c_char,
                    sfile.as_ptr(),
                );
                return;
            }
        }

        for i in 0..MAX_LOOPED_FX {
            if self.mLoopedEffectArray[i].mId == id
                && self.mLoopedEffectArray[i].mBoltInfo == boltInfo
                && self.mLoopedEffectArray[i].mPortalEffect == isPortal
            {
                // memset( &mLoopedEffectArray[i], 0, sizeof(mLoopedEffectArray[i]) );
                core::ptr::write_bytes(
                    addr_of_mut!(self.mLoopedEffectArray[i]),
                    0u8,
                    1,
                );
                return;
            }
        }
        #[cfg(feature = "debug_fx")]
        {
            theFxHelper.Print(
                b"CFxScheduler::StopEffect- (%s) is not looping!\n\0".as_ptr() as *const c_char,
                file,
            );
        }
    }

    pub unsafe fn AddLoopedEffects(&mut self) {
        for i in 0..MAX_LOOPED_FX {
            if self.mLoopedEffectArray[i].mId != 0
                && self.mLoopedEffectArray[i].mNextTime < theFxHelper.mTime
            {
                let entNum: c_int = (self.mLoopedEffectArray[i].mBoltInfo >> ENTITY_SHIFT)
                    & ENTITY_AND;
                if (*cg_entities.as_ptr().offset(entNum as isize))
                    .gent
                    .as_ref()
                    .map_or(false, |g| g.inuse)
                {
                    // only play the looped effect when the ent is still inUse....
                    let eid = self.mLoopedEffectArray[i].mId;
                    let boltInfo = self.mLoopedEffectArray[i].mBoltInfo;
                    let isPortal = self.mLoopedEffectArray[i].mPortalEffect;
                    let isRelative = self.mLoopedEffectArray[i].mIsRelative;
                    let lerpOrigin = (*cg_entities.as_ptr().offset(entNum as isize)).lerpOrigin;
                    // PlayEffect( mLoopedEffectArray[i].mId, cg_entities[entNum].lerpOrigin, 0, mLoopedEffectArray[i].mBoltInfo, -1, mLoopedEffectArray[i].mPortalEffect, false,  mLoopedEffectArray[i].mIsRelative );
                    // very important to send FALSE looptime to not recursively add me!
                    self.PlayEffect_Full(
                        eid,
                        lerpOrigin,
                        null_mut(),
                        boltInfo,
                        -1,
                        isPortal,
                        0, // false => 0 = not looping, so we don't recursively add
                        isRelative,
                    );
                    self.mLoopedEffectArray[i].mNextTime = theFxHelper.mTime
                        + self.mEffectTemplates[self.mLoopedEffectArray[i].mId as usize]
                            .mRepeatDelay;
                } else {
                    theFxHelper.Print(
                        b"CFxScheduler::AddLoopedEffects- entity was removed without stopping any looping fx it owned.\0"
                            .as_ptr() as *const c_char,
                    );
                    // memset( &mLoopedEffectArray[i], 0, sizeof(mLoopedEffectArray[i]) );
                    core::ptr::write_bytes(addr_of_mut!(self.mLoopedEffectArray[i]), 0u8, 1);
                    continue;
                }
                if self.mLoopedEffectArray[i].mLoopStopTime != 0
                    && self.mLoopedEffectArray[i].mLoopStopTime < theFxHelper.mTime
                {
                    // time's up
                    //kill this entry
                    // memset( &mLoopedEffectArray[i], 0, sizeof(mLoopedEffectArray[i]) );
                    core::ptr::write_bytes(addr_of_mut!(self.mLoopedEffectArray[i]), 0u8, 1);
                }
            }
        }
    }

    //------------------------------------------------------
    // Clean
    //  Free up any memory we've allocated so we aren't leaking memory
    //
    // Input:
    //  Whether to clean everything or just stop the playing (active) effects
    //
    // Return:
    //  None
    //
    //------------------------------------------------------
    pub unsafe fn Clean(&mut self, bRemoveTemplates: bool, idToPreserve: c_int) {
        // Ditch any scheduled effects
        // C++ pattern: iterate with iterator, delete *itr, erase from list.
        // In Rust the LinkedList stores SScheduledEffect by value; clearing drops them all.
        // The raw mpTemplate pointers inside are NOT freed here (same as C++ delete on the list node).
        self.mFxSchedule.clear();

        if bRemoveTemplates {
            // Ditch any effect templates
            for i in 1..FX_MAX_EFFECTS {
                if i == idToPreserve as usize {
                    continue;
                }

                if self.mEffectTemplates[i].mInUse {
                    // Ditch the primitives
                    for j in 0..self.mEffectTemplates[i].mPrimitiveCount as usize {
                        drop(Box::from_raw(self.mEffectTemplates[i].mPrimitives[j]));
                    }
                }

                self.mEffectTemplates[i].mInUse = false;
            }

            if idToPreserve == 0 {
                self.mEffectIDs.clear();
            } else {
                // Clear the effect names, but first get the name of the effect to preserve,
                // and restore it after clearing.
                let mut str_key = fxString_t::new();

                for (key, val) in self.mEffectIDs.iter() {
                    if *val == idToPreserve {
                        str_key.operator_assign(key);
                        break;
                    }
                }

                self.mEffectIDs.clear();

                self.mEffectIDs.insert(str_key, idToPreserve);
            }
        }
    }

    //------------------------------------------------------
    // RegisterEffect
    //  Attempt to open the specified effect file, if
    //  file read succeeds, parse the file.
    //
    // Input:
    //  path or filename to open
    //
    // Return:
    //  int handle to the effect
    //------------------------------------------------------
    pub unsafe fn RegisterEffect(
        &mut self,
        file: *const c_char,
        bHasCorrectPath: bool,
    ) -> c_int {
        // Dealing with file names:
        // File names can come from two places - the editor, in which case we should use the given
        // path as is, and the effect file, in which case we should add the correct path and extension.
        // In either case we create a stripped file name to use for naming effects.
        //

        let mut sfile: [c_char; MAX_QPATH] = core::mem::zeroed();

        // Get an extension stripped version of the file
        if bHasCorrectPath {
            let mut last: *const c_char = file;
            let mut p: *const c_char = file;

            while *p != 0 {
                if *p == b'/' as c_char || *p == b'\\' as c_char {
                    last = p.offset(1);
                }
                p = p.offset(1);
            }

            COM_StripExtension(last, sfile.as_mut_ptr());
        } else {
            COM_StripExtension(file, sfile.as_mut_ptr());
        }

        // see if the specified file is already registered.  If it is, just return the id of that file
        let sfile_key = fxString_t::new_from_str(sfile.as_ptr());

        if let Some(&existing_id) = self.mEffectIDs.get(&sfile_key) {
            return existing_id;
        }

        let mut len: c_int = 0;
        let mut fh: fileHandle_t = core::mem::zeroed();
        let mut temp: [c_char; MAX_QPATH] = core::mem::zeroed();
        let pfile: *const c_char;

        if bHasCorrectPath {
            pfile = file;
        } else {
            // Add on our extension and prepend the file with the default path
            // sprintf( temp, "%s/%s.efx", FX_FILE_PATH, sfile );
            // FX_FILE_PATH is a &str; use its bytes as C string for sprintf.
            // Porting note: FX_FILE_PATH = "effects" (no null); using Q_snprintf for safety.
            Q_snprintf(
                temp.as_mut_ptr(),
                core::mem::size_of_val(&temp) as c_int,
                b"%s/%s.efx\0".as_ptr() as *const c_char,
                b"effects\0".as_ptr() as *const c_char,
                sfile.as_ptr(),
            );
            pfile = temp.as_ptr();
        }

        len = theFxHelper.OpenFile(pfile, &mut fh, FS_READ);

        if len < 0 {
            theFxHelper.Print(
                b"RegisterEffect: failed to load: %s\n\0".as_ptr() as *const c_char,
                pfile,
            );
            return 0;
        }

        if len == 0 {
            theFxHelper.Print(
                b"RegisterEffect: INVALID file: %s\n\0".as_ptr() as *const c_char,
                pfile,
            );
            theFxHelper.CloseFile(fh);
            return 0;
        }

        // Allocate enough space to hold the file
        // This should be flagged temp, but it seems ok as is.
        // data = new char[len+1];
        let mut data: Box<[u8]> = vec![0u8; (len + 1) as usize].into_boxed_slice();
        let data_ptr: *mut c_char = data.as_mut_ptr() as *mut c_char;

        // Get the goods and ensure Null termination
        theFxHelper.ReadFile(data_ptr as *mut c_void, len, fh);
        *data_ptr.offset(len as isize) = 0;
        let mut bufParse: *mut c_char = data_ptr;

        // Let the generic parser process the whole file
        let mut parser: CGenericParser2 = CGenericParser2::new();
        parser.Parse(&mut bufParse);

        theFxHelper.CloseFile(fh);

        // Delete our temp copy of the file (handled by Box drop above)

        // Lets convert the effect file into something that we can work with
        self.ParseEffect(sfile.as_ptr(), parser.GetBaseParseGroup())
    }

    //------------------------------------------------------
    // ParseEffect
    //  Starts at ground zero, using each group header to
    //  determine which kind of effect we are working with.
    //  Then we call the appropriate function to parse the
    //  specified effect group.
    //
    // Input:
    //  base group, essentially the whole files contents
    //
    // Return:
    //  int handle of the effect
    //------------------------------------------------------
    pub unsafe fn ParseEffect(&mut self, file: *const c_char, base: *mut CGPGroup) -> c_int {
        let primitiveGroup: *mut CGPGroup;
        let prim: *mut CPrimitiveTemplate;
        let grpName: *const c_char;
        let mut effect: *mut SEffectTemplate = null_mut();
        let mut type_: EPrimType;
        let mut handle: c_int = 0;
        let pair: *mut CGPValue;

        effect = self.GetNewEffectTemplate(&mut handle, file);

        if handle == 0 || effect.is_null() {
            // failure
            return 0;
        }

        let pair_check = (*base).GetPairs();
        if !pair_check.is_null() {
            let pair = pair_check;
            grpName = (*pair).GetName();
            if stricmp(grpName, b"repeatDelay\0".as_ptr() as *const c_char) == 0 {
                (*effect).mRepeatDelay = atoi((*pair).GetTopValue());
            } else {
                //unknown
            }
        }

        let mut primitiveGroup = (*base).GetSubGroups();

        while !primitiveGroup.is_null() {
            grpName = (*primitiveGroup).GetName();

            // Huge stricmp lists suxor
            type_ = if stricmp(grpName, b"particle\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Particle
            } else if stricmp(grpName, b"line\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Line
            } else if stricmp(grpName, b"tail\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Tail
            } else if stricmp(grpName, b"sound\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Sound
            } else if stricmp(grpName, b"cylinder\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Cylinder
            } else if stricmp(grpName, b"electricity\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Electricity
            } else if stricmp(grpName, b"emitter\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Emitter
            } else if stricmp(grpName, b"decal\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Decal
            } else if stricmp(grpName, b"orientedparticle\0".as_ptr() as *const c_char) == 0 {
                EPrimType::OrientedParticle
            } else if stricmp(grpName, b"fxrunner\0".as_ptr() as *const c_char) == 0 {
                EPrimType::FxRunner
            } else if stricmp(grpName, b"light\0".as_ptr() as *const c_char) == 0 {
                EPrimType::Light
            } else if stricmp(grpName, b"cameraShake\0".as_ptr() as *const c_char) == 0 {
                EPrimType::CameraShake
            } else if stricmp(grpName, b"flash\0".as_ptr() as *const c_char) == 0 {
                EPrimType::ScreenFlash
            } else {
                EPrimType::None
            };
#[cfg(feature = "immersion")]
            // #ifdef _IMMERSION: "forcefeedback" group maps to PRIMTYPE_FORCE. Hoisted out of the
            // if/else chain above (a #[cfg] attribute cannot gate an `else if` arm); the group-name
            // matches are mutually exclusive, so testing it here is behavior-identical to the C order.
            if stricmp(grpName, b"forcefeedback\0".as_ptr() as *const c_char) == 0 {
                type_ = EPrimType::Force;
            }

            if type_ != EPrimType::None {
                let prim = Box::into_raw(Box::new(CPrimitiveTemplate::new()));

                (*prim).mType = type_;
                (*prim).ParsePrimitive(primitiveGroup);

                // Add our primitive template to the effect list
                self.AddPrimitiveToEffect(effect, prim);
            }

            primitiveGroup = (*primitiveGroup).GetNext() as *mut CGPGroup;
        }

        handle
    }

    //------------------------------------------------------
    // AddPrimitiveToEffect
    //  Takes a primitive and attaches it to the effect.
    //
    // Input:
    //  Effect template that we tack the primitive on to
    //  Primitive to add to the effect template
    //
    // Return:
    //  None
    //------------------------------------------------------
    pub unsafe fn AddPrimitiveToEffect(
        &mut self,
        fx: *mut SEffectTemplate,
        prim: *mut CPrimitiveTemplate,
    ) {
        let ct = (*fx).mPrimitiveCount as usize;

        if ct >= FX_MAX_EFFECT_COMPONENTS {
            theFxHelper.Print(
                b"FxScheduler:  Error--too many primitives in an effect\n\0".as_ptr()
                    as *const c_char,
            );
        } else {
            (*fx).mPrimitives[ct] = prim;
            (*fx).mPrimitiveCount += 1;
        }
    }

    //------------------------------------------------------
    // GetNewEffectTemplate
    //  Finds an unused effect template and returns it to the
    //  caller.
    //
    // Input:
    //  pointer to an id that will be filled in,
    //  file name-- should be NULL when requesting a copy
    //
    // Return:
    //  the id of the added effect template
    //------------------------------------------------------
    pub unsafe fn GetNewEffectTemplate(
        &mut self,
        id: *mut c_int,
        file: *const c_char,
    ) -> *mut SEffectTemplate {
        // wanted zero to be a bogus effect ID, so we just skip it.
        for i in 1..FX_MAX_EFFECTS {
            let effect: *mut SEffectTemplate = &mut self.mEffectTemplates[i];

            if !(*effect).mInUse {
                *id = i as c_int;
                // memset( effect, 0, sizeof( SEffectTemplate ));
                core::ptr::write_bytes(effect, 0u8, 1);

                // If we are a copy, we really won't have a name that we care about saving for later
                if !file.is_null() {
                    let file_key = fxString_t::new_from_str(file);
                    self.mEffectIDs.insert(file_key, i as c_int);
                    // strcpy( effect->mEffectName, file );
                    // mEffectName is [u8; MAX_QPATH]; use Q_strncpyz (from q_shared imports)
                    Q_strncpyz(
                        (*effect).mEffectName.as_mut_ptr() as *mut c_char,
                        file,
                        MAX_QPATH as c_int,
                    );
                }

                (*effect).mInUse = true;
                (*effect).mRepeatDelay = 300;
                return effect;
            }
        }

        theFxHelper.Print(
            b"FxScheduler:  Error--reached max effects\n\0".as_ptr() as *const c_char,
        );
        *id = 0;
        null_mut()
    }

    //------------------------------------------------------
    // GetEffectCopy (from file name)
    //  Returns a copy of the desired effect so that it can
    //  easily be modified run-time.
    //
    // Input:
    //  file-- the name of the effect file that you want a copy of
    //  newHandle-- will actually be the returned handle to the new effect
    //              you have to hold onto this if you intend to call it again
    //
    // Return:
    //  the pointer to the copy
    //------------------------------------------------------
    pub unsafe fn GetEffectCopyFromFile(
        &mut self,
        file: *const c_char,
        newHandle: *mut c_int,
    ) -> *mut SEffectTemplate {
        let file_key = fxString_t::new_from_str(file);
        let fxHandle: c_int = *self.mEffectIDs.entry(file_key).or_insert(0);
        self.GetEffectCopy(fxHandle, newHandle)
    }

    //------------------------------------------------------
    // GetEffectCopy (from handle)
    //  Returns a copy of the desired effect so that it can
    //  easily be modified run-time.
    //
    // Input:
    //  fxHandle-- the handle to the effect that you want a copy of
    //  newHandle-- will actually be the returned handle to the new effect
    //              you have to hold onto this if you intend to call it again
    //
    // Return:
    //  the pointer to the copy
    //------------------------------------------------------
    pub unsafe fn GetEffectCopy(
        &mut self,
        fxHandle: c_int,
        newHandle: *mut c_int,
    ) -> *mut SEffectTemplate {
        if fxHandle < 1
            || fxHandle >= FX_MAX_EFFECTS as c_int
            || !self.mEffectTemplates[fxHandle as usize].mInUse
        {
            // Didn't even request a valid effect to copy!!!
            theFxHelper.Print(
                b"FxScheduler: Bad effect file copy request\n\0".as_ptr() as *const c_char,
            );

            *newHandle = 0;
            return null_mut();
        }

        // never get a copy when time is frozen
        if fx_freeze.integer != 0 {
            return null_mut();
        }

        // Copies shouldn't have names, otherwise they could trash our stl map used for getting ID from name
        let copy: *mut SEffectTemplate = self.GetNewEffectTemplate(newHandle, null_mut());

        if !copy.is_null() && *newHandle != 0 {
            // do the effect copy and mark us as what we are
            (*copy).assign(&self.mEffectTemplates[fxHandle as usize]);
            (*copy).mCopy = true;

            // the user had better hold onto this handle if they ever hope to call this effect.
            return copy;
        }

        // No space left to return an effect
        *newHandle = 0;
        null_mut()
    }

    //------------------------------------------------------
    // GetPrimitiveCopy
    //  Helper function that returns a copy of the desired primitive
    //
    // Input:
    //  fxHandle - the pointer to the effect copy you want to override
    //  componentName - name of the component to find
    //
    // Return:
    //  the pointer to the desired primitive
    //------------------------------------------------------
    pub unsafe fn GetPrimitiveCopy(
        &mut self,
        effectCopy: *mut SEffectTemplate,
        componentName: *const c_char,
    ) -> *mut CPrimitiveTemplate {
        if effectCopy.is_null() || !(*effectCopy).mInUse {
            return null_mut();
        }

        for i in 0..(*effectCopy).mPrimitiveCount as usize {
            // stricmp( effectCopy->mPrimitives[i]->mName, componentName )
            if stricmp(
                (*(*effectCopy).mPrimitives[i]).mName.as_ptr() as *const c_char,
                componentName,
            ) == 0
            {
                // we found a match, so return it
                return (*effectCopy).mPrimitives[i];
            }
        }

        // bah, no good.
        null_mut()
    }
}

//------------------------------------------------------
unsafe fn ReportPlayEffectError(id: c_int) {
    #[cfg(debug_assertions)]
    {
        theFxHelper.Print(
            b"CFxScheduler::PlayEffect called with invalid effect ID: %i\n\0".as_ptr()
                as *const c_char,
            id,
        );
    }
}

impl CFxScheduler {
    //------------------------------------------------------
    // PlayEffect
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Applies a default up
    //  axis.
    //
    // Input:
    //  Effect file id and the origin
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn PlayEffect_Org(
        &mut self,
        id: c_int,
        origin: vec3_t,
        isPortal: bool,
    ) {
        let mut axis: [vec3_t; 3] = core::mem::zeroed();

        VectorSet(axis[0].as_mut_ptr(), 0.0, 0.0, 1.0);
        VectorSet(axis[1].as_mut_ptr(), 1.0, 0.0, 0.0);
        VectorSet(axis[2].as_mut_ptr(), 0.0, 1.0, 0.0);

        self.PlayEffect_Full(id, origin, axis.as_mut_ptr() as *mut vec3_t, -1, -1, isPortal, 0, false);
    }

    //------------------------------------------------------
    // PlayEffect
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Takes a fwd vector
    //  and builds a right and up vector
    //
    // Input:
    //  Effect file id, the origin, and a fwd vector
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn PlayEffect_OrgFwd(
        &mut self,
        id: c_int,
        origin: vec3_t,
        forward: vec3_t,
        isPortal: bool,
    ) {
        let mut axis: [vec3_t; 3] = core::mem::zeroed();

        // Take the forward vector and create two arbitrary but perpendicular vectors
        VectorCopy(forward.as_ptr(), axis[0].as_mut_ptr());
        MakeNormalVectors(forward.as_ptr(), axis[1].as_mut_ptr(), axis[2].as_mut_ptr());

        self.PlayEffect_Full(id, origin, axis.as_mut_ptr() as *mut vec3_t, -1, -1, isPortal, 0, false);
    }

    // #ifdef _IMMERSION
    //------------------------------------------------------
    // PlayEffect
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Takes a fwd vector
    //  and builds a right and up vector
    //
    // Input:
    //  Effect file id, the origin, a fwd vector, and clientNum
    //
    // Return:
    //  none
    //------------------------------------------------------
    #[cfg(feature = "immersion")]
    pub unsafe fn PlayEffect_ClientIDOrgFwd(
        &mut self,
        id: c_int,
        clientNum: c_int,
        origin: vec3_t,
        forward: vec3_t,
        isPortal: bool,
    ) {
        let mut axis: [vec3_t; 3] = core::mem::zeroed();

        // Take the forward vector and create two arbitrary but perpendicular vectors
        VectorCopy(forward.as_ptr(), axis[0].as_mut_ptr());
        MakeNormalVectors(forward.as_ptr(), axis[1].as_mut_ptr(), axis[2].as_mut_ptr());

        self.PlayEffect_Full(id, origin, axis.as_mut_ptr() as *mut vec3_t, -1, clientNum, isPortal, 0, false);
    }

    //------------------------------------------------------
    // PlayEffect
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Takes a forward vector
    //  and uses this to complete the axis field.
    //
    // Input:
    //  Effect file name, the origin, and a forward vector
    //
    // Return:
    //  none
    //------------------------------------------------------
    #[cfg(feature = "immersion")]
    pub unsafe fn PlayEffect_FileClientIDOrgFwd(
        &mut self,
        file: *const c_char,
        clientNum: c_int,
        origin: vec3_t,
        forward: vec3_t,
        isPortal: bool,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = core::mem::zeroed();

        // Get an extenstion stripped version of the file
        COM_StripExtension(file, sfile.as_mut_ptr());

        let sfile_key = fxString_t::new_from_str(sfile.as_ptr());
        let id: c_int = *self.mEffectIDs.entry(sfile_key.clone()).or_insert(0);
        self.PlayEffect_ClientIDOrgFwd(id, clientNum, origin, forward, isPortal);

        #[cfg(not(feature = "final_build"))]
        {
            let sfile_key2 = fxString_t::new_from_str(sfile.as_ptr());
            if *self.mEffectIDs.entry(sfile_key2).or_insert(0) == 0 {
                theFxHelper.Print(
                    b"CFxScheduler::PlayEffect unregistered/non-existent effect: %s\n\0".as_ptr()
                        as *const c_char,
                    file,
                );
            }
        }
    }
    // #endif // _IMMERSION

    //------------------------------------------------------
    // PlayEffect
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Uses the specified axis
    //
    // Input:
    //  Effect file name, the origin, and axis.
    //  Optional boltInfo (defaults to -1)
    //  Optional entity number to be used by a cheap entity origin bolt (defaults to -1)
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn PlayEffect_FileFull(
        &mut self,
        file: *const c_char,
        origin: vec3_t,
        axis: *mut vec3_t,
        boltInfo: c_int,
        entNum: c_int,
        isPortal: bool,
        iLoopTime: c_int,
        isRelative: bool,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = core::mem::zeroed();

        // Get an extenstion stripped version of the file
        COM_StripExtension(file, sfile.as_mut_ptr());

        // This is a horribly dumb thing to have to do, but QuakeIII might not have calc'd the lerpOrigin
        //  for the entity we may be trying to bolt onto.  We like having the correct origin, so we are
        //  forced to call this function....
        if entNum > -1 {
            CG_CalcEntityLerpPositions(
                cg_entities.as_mut_ptr().offset(entNum as isize),
            );
        }

        #[cfg(not(feature = "final_build"))]
        {
            let sfile_key = fxString_t::new_from_str(sfile.as_ptr());
            if *self.mEffectIDs.entry(sfile_key).or_insert(0) == 0 {
                theFxHelper.Print(
                    b"CFxScheduler::PlayEffect unregistered/non-existent effect: %s\n\0".as_ptr()
                        as *const c_char,
                    sfile.as_ptr(),
                );
            }
        }

        let sfile_key2 = fxString_t::new_from_str(sfile.as_ptr());
        let effect_id: c_int = *self.mEffectIDs.entry(sfile_key2).or_insert(0);
        self.PlayEffect_Full(effect_id, origin, axis, boltInfo, entNum, isPortal, iLoopTime, isRelative);
    }

    //------------------------------------------------------
    // PlayEffect (file + clientID variant)
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Uses the specified axis
    //
    // Input:
    //  Effect file name, the origin, and axis.
    //  Optional boltInfo (defaults to -1)
    //  Optional entity number to be used by a cheap entity origin bolt (defaults to -1)
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn PlayEffect_FileClientID(
        &mut self,
        file: *const c_char,
        clientID: c_int,
        isPortal: bool,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = core::mem::zeroed();
        let id: c_int;

        // Get an extenstion stripped version of the file
        COM_StripExtension(file, sfile.as_mut_ptr());
        let sfile_key = fxString_t::new_from_str(sfile.as_ptr());
        id = *self.mEffectIDs.entry(sfile_key).or_insert(0);

        #[cfg(not(feature = "final_build"))]
        {
            if id == 0 {
                theFxHelper.Print(
                    b"CFxScheduler::PlayEffect unregistered/non-existent effect: %s\n\0".as_ptr()
                        as *const c_char,
                    file,
                );
            }
        }

        let mut fx: *mut SEffectTemplate;
        let mut prim: *mut CPrimitiveTemplate;
        let mut count: c_int = 0;
        let mut delay: c_int = 0;
        let mut factor: f32 = 0.0;

        if id < 1 || id >= FX_MAX_EFFECTS as c_int || !self.mEffectTemplates[id as usize].mInUse {
            // Now you've done it!
            ReportPlayEffectError(id);
            return;
        }

        // Don't bother scheduling the effect if the system is currently frozen

        // Get the effect.
        fx = &mut self.mEffectTemplates[id as usize];

        // Loop through the primitives and schedule each bit
        for i in 0..(*fx).mPrimitiveCount as usize {
            prim = (*fx).mPrimitives[i];

            count = (*prim).mSpawnCount.GetRoundedVal();

            if (*prim).mCopy {
                // If we are a copy, we need to store a "how many references count" so that we
                //  can keep the primitive template around for the correct amount of time.
                (*prim).mRefCount = count;
            }

            if (*prim).mSpawnFlags & FX_EVEN_DISTRIBUTION != 0 {
                factor = ((*prim).mSpawnDelay.GetMax() - (*prim).mSpawnDelay.GetMin()).abs()
                    / count as f32;
            }

            // Schedule the random number of bits
            for t in 0..count as usize {
                if (*prim).mSpawnFlags & FX_EVEN_DISTRIBUTION != 0 {
                    delay = (t as f32 * factor) as c_int;
                } else {
                    delay = (*prim).mSpawnDelay.GetValRand() as c_int;
                }

                // if the delay is so small, we may as well just create this bit right now
                if delay < 1 && !isPortal {
                    self.CreateEffectClientOnly(prim, clientID, -delay);
                } else {
                    // We have to create a new scheduled effect so that we can create it at a later point
                    //  you should avoid this because it's much more expensive
                    // Porting note: C++ allocated SScheduledEffect* on heap; Rust LinkedList stores by value.
                    self.mFxSchedule.push_front(SScheduledEffect {
                        mpTemplate: prim,
                        mStartTime: theFxHelper.mTime + delay,
                        mModelNum: 0,
                        mBoltNum: 0,
                        mEntNum: 0,
                        mClientID: clientID as i16,
                        mPortalEffect: isPortal,
                        mIsRelative: false,
                        mOrigin: [0.0; 3],
                        mAxis: [[0.0; 3]; 3],
                    });
                }
            }
        }

        // We track effect templates and primitive templates separately.
        if (*fx).mCopy {
            // We don't use dynamic memory allocation, so just mark us as dead
            (*fx).mInUse = false;
        }
    }

    //------------------------------------------------------
    // PlayEffect (int id + origin + axis -- main implementation)
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Uses the specified axis
    //
    // Input:
    //  Effect id, the origin, and axis.
    //  Optional boltInfo (defaults to -1)
    //  Optional entity number to be used by a cheap entity origin bolt (defaults to -1)
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn PlayEffect_Full(
        &mut self,
        id: c_int,
        origin: vec3_t,
        axis: *mut vec3_t, // vec3_t axis[3] -- can be null (e.g. from AddLoopedEffects)
        boltInfo: c_int,
        entNum: c_int,
        isPortal: bool,
        iLoopTime: c_int,
        isRelative: bool,
    ) {
        let mut fx: *mut SEffectTemplate;
        let mut prim: *mut CPrimitiveTemplate;
        let mut count: c_int;
        let mut delay: c_int;
        let mut factor: f32 = 0.0;
        let mut forceScheduling: bool = false;

        if id < 1 || id >= FX_MAX_EFFECTS as c_int || !self.mEffectTemplates[id as usize].mInUse {
            // Now you've done it!
            ReportPlayEffectError(id);
            return;
        }

        // Don't bother scheduling the effect if the system is currently frozen
        if fx_freeze.integer != 0 {
            return;
        }

        let mut modelNum: c_int = 0;
        let mut boltNum: c_int = -1;
        let mut entityNum: c_int = entNum;

        #[cfg(feature = "immersion")]
        {
            // HACKHACKHACK (negative if effect plays uncentered on an entity)
            entityNum = if entNum < -1 {
                FF_CLIENT(entNum) // decode -2 as entNum=0, -3 as entNum=1, ...
            } else {
                entNum // default
            };
        }

        if boltInfo > 0 {
            // extract the wraith ID from the bolt info
            modelNum = (boltInfo >> MODEL_SHIFT) & MODEL_AND;
            boltNum = (boltInfo >> BOLT_SHIFT) & BOLT_AND;
            entityNum = (boltInfo >> ENTITY_SHIFT) & ENTITY_AND;

            // We always force ghoul bolted objects to be scheduled so that they don't play right away.
            forceScheduling = true;

            if iLoopTime != 0 {
                //0 = not looping, 1 for infinite, else duration
                //store off the id to reschedule every frame
                self.ScheduleLoopedEffect(id, boltInfo, isPortal, iLoopTime, isRelative);
            }
        }

        // Get the effect.
        fx = &mut self.mEffectTemplates[id as usize];

        // Loop through the primitives and schedule each bit
        for i in 0..(*fx).mPrimitiveCount as usize {
            prim = (*fx).mPrimitives[i];

            if (*prim).mCullRange != 0 {
                if DistanceSquared(origin.as_ptr(), (*addr_of!(cg)).refdef.vieworg.as_ptr())
                    > (*prim).mCullRange as f32
                {
                    // cull range has already been squared
                    // is too far away, so don't add this primitive group
                    continue;
                }
            }

            count = (*prim).mSpawnCount.GetRoundedVal();

            if (*prim).mCopy {
                // If we are a copy, we need to store a "how many references count" so that we
                //  can keep the primitive template around for the correct amount of time.
                (*prim).mRefCount = count;
            }

            if (*prim).mSpawnFlags & FX_EVEN_DISTRIBUTION != 0 {
                factor = ((*prim).mSpawnDelay.GetMax() - (*prim).mSpawnDelay.GetMin()).abs()
                    / count as f32;
            }

            // Schedule the random number of bits
            for t in 0..count as usize {
                if (*prim).mSpawnFlags & FX_EVEN_DISTRIBUTION != 0 {
                    delay = (t as f32 * factor) as c_int;
                } else {
                    delay = (*prim).mSpawnDelay.GetValRand() as c_int;
                }

                // if the delay is so small, we may as well just create this bit right now
                if delay < 1 && !forceScheduling && !isPortal {
                    #[cfg(feature = "immersion")]
                    let immersion_cond = boltInfo == -1 && entNum > -1;
                    #[cfg(not(feature = "immersion"))]
                    let immersion_cond = boltInfo == -1 && entNum != -1;

                    if immersion_cond {
                        // Find out where the entity currently is
                        self.CreateEffectFull(
                            prim,
                            (*cg_entities.as_ptr().offset(entNum as isize)).lerpOrigin,
                            axis,
                            -delay,
                            -1,
                            -1,
                            -1,
                        );
                    } else {
                        self.CreateEffectFull(prim, origin, axis, -delay, -1, -1, -1);
                    }
                } else {
                    // We have to create a new scheduled effect so that we can create it at a later point
                    //  you should avoid this because it's much more expensive
                    let mut sfx_val = SScheduledEffect {
                        mpTemplate: prim,
                        mStartTime: theFxHelper.mTime + delay,
                        mModelNum: 0,
                        mBoltNum: 0,
                        mEntNum: entityNum as i16,
                        mClientID: -1,
                        mPortalEffect: isPortal,
                        mIsRelative: isRelative,
                        mOrigin: [0.0; 3],
                        mAxis: [[0.0; 3]; 3],
                    };
                    // sfx->mIsRelative = isRelative;
                    // sfx->mEntNum = entityNum; -- already set above
                    // sfx->mPortalEffect = isPortal; -- already set above

                    if boltInfo == -1 {
                        #[cfg(feature = "immersion")]
                        let no_ent = entNum <= -1;
                        #[cfg(not(feature = "immersion"))]
                        let no_ent = entNum == -1;

                        if no_ent {
                            // we aren't bolting, so make sure the spawn system knows this by putting -1's in these fields
                            sfx_val.mBoltNum = u8::MAX; // -1 as u8 (0xFF)
                            sfx_val.mModelNum = 0;

                            if !axis.is_null() {
                                if !origin.as_ptr().is_null() {
                                    VectorCopy(origin.as_ptr(), sfx_val.mOrigin.as_mut_ptr());
                                } else {
                                    VectorClear(sfx_val.mOrigin.as_mut_ptr());
                                }
                                AxisCopy(axis, sfx_val.mAxis.as_mut_ptr() as *mut vec3_t);
                            } else {
                                VectorClear(sfx_val.mOrigin.as_mut_ptr());
                            }
                        } else {
                            // we are doing bolting onto the origin of the entity, so use a cheaper method
                            sfx_val.mBoltNum = u8::MAX; // -1 as u8
                            sfx_val.mModelNum = 0;

                            if !axis.is_null() {
                                AxisCopy(axis, sfx_val.mAxis.as_mut_ptr() as *mut vec3_t);
                            }
                        }
                    } else {
                        // we are bolting, so store the extra info
                        sfx_val.mBoltNum = boltNum as u8;
                        sfx_val.mModelNum = modelNum as u8;

                        // Also, the ghoul bolt may not be around yet, so delay the creation one frame
                        sfx_val.mStartTime += 1;
                    }

                    self.mFxSchedule.push_front(sfx_val);
                }
            }
        }

        // We track effect templates and primitive templates separately.
        if (*fx).mCopy {
            // We don't use dynamic memory allocation, so just mark us as dead
            (*fx).mInUse = false;
        }
    }

    //------------------------------------------------------
    // PlayEffect (file + origin simple version)
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Applies a default up
    //  axis.
    //
    // Input:
    //  Effect file name and the origin
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn PlayEffect_FileOrg(
        &mut self,
        file: *const c_char,
        origin: vec3_t,
        isPortal: bool,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = core::mem::zeroed();

        // Get an extenstion stripped version of the file
        COM_StripExtension(file, sfile.as_mut_ptr());

        let sfile_key = fxString_t::new_from_str(sfile.as_ptr());
        let id: c_int = *self.mEffectIDs.entry(sfile_key).or_insert(0);
        self.PlayEffect_Org(id, origin, isPortal);

        #[cfg(not(feature = "final_build"))]
        {
            let sfile_key2 = fxString_t::new_from_str(sfile.as_ptr());
            if *self.mEffectIDs.entry(sfile_key2).or_insert(0) == 0 {
                theFxHelper.Print(
                    b"CFxScheduler::PlayEffect unregistered/non-existent effect: %s\n\0".as_ptr()
                        as *const c_char,
                    file,
                );
            }
        }
    }

    //------------------------------------------------------
    // PlayEffect (file + origin + forward)
    //  Handles scheduling an effect so all the components
    //  happen at the specified time.  Takes a forward vector
    //  and uses this to complete the axis field.
    //
    // Input:
    //  Effect file name, the origin, and a forward vector
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn PlayEffect_FileOrgFwd(
        &mut self,
        file: *const c_char,
        origin: vec3_t,
        forward: vec3_t,
        isPortal: bool,
    ) {
        let mut sfile: [c_char; MAX_QPATH] = core::mem::zeroed();

        // Get an extenstion stripped version of the file
        COM_StripExtension(file, sfile.as_mut_ptr());

        let sfile_key = fxString_t::new_from_str(sfile.as_ptr());
        let id: c_int = *self.mEffectIDs.entry(sfile_key).or_insert(0);
        self.PlayEffect_OrgFwd(id, origin, forward, isPortal);

        #[cfg(not(feature = "final_build"))]
        {
            let sfile_key2 = fxString_t::new_from_str(sfile.as_ptr());
            if *self.mEffectIDs.entry(sfile_key2).or_insert(0) == 0 {
                theFxHelper.Print(
                    b"CFxScheduler::PlayEffect unregistered/non-existent effect: %s\n\0".as_ptr()
                        as *const c_char,
                    file,
                );
            }
        }
    }

    //------------------------------------------------------
    // AddScheduledEffects
    //  Handles determining if a scheduled effect should
    //  be created or not.  If it should it handles converting
    //  the template effect into a real one.
    //
    // Input:
    //  boolean portal (true when adding effects to be drawn in the skyportal)
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn AddScheduledEffects(&mut self, portal: bool) {
        let mut origin: vec3_t = core::mem::zeroed();
        let mut axis: [vec3_t; 3] = core::mem::zeroed();
        let mut oldEntNum: c_int = -1;
        let mut oldBoltIndex: c_int = -1;
        let mut oldModelNum: c_int = -1;
        let mut doesBoltExist: qboolean = qfalse;

        if portal {
            *addr_of_mut!(gEffectsInPortal) = true;
        } else {
            self.AddLoopedEffects();
        }

        // C++ pattern: iterate forward through linked list, conditionally erase elements.
        // Porting note: In C++ mFxSchedule is list<SScheduledEffect*>; in Rust it's
        // LinkedList<SScheduledEffect> (by value).  We rebuild the list, processing and
        // discarding due items, retaining the rest.
        let mut remaining: std::collections::LinkedList<SScheduledEffect> =
            std::collections::LinkedList::new();

        while let Some(sfx) = self.mFxSchedule.pop_front() {
            if portal == sfx.mPortalEffect {
                if sfx.mStartTime <= theFxHelper.mTime {
                    // *(*itr) <= theFxHelper.mTime
                    if sfx.mClientID >= 0 {
                        self.CreateEffectClientOnly(
                            sfx.mpTemplate,
                            sfx.mClientID as c_int,
                            theFxHelper.mTime - sfx.mStartTime,
                        );
                    } else if sfx.mBoltNum == u8::MAX {
                        // mBoltNum == -1 (stored as u8::MAX)
                        // normal effect
                        #[cfg(feature = "immersion")]
                        {
                            let ent_num = sfx.mEntNum as c_int;
                            let hit_ent_num: c_int = if ent_num < -1 {
                                FF_CLIENT(ent_num)
                            } else {
                                ent_num
                            };
                            let eff_origin = if ent_num >= 0 {
                                (*cg_entities.as_ptr().offset(ent_num as isize)).lerpOrigin
                            } else {
                                sfx.mOrigin
                            };
                            self.CreateEffectFull(
                                sfx.mpTemplate,
                                eff_origin,
                                sfx.mAxis.as_ptr() as *mut vec3_t,
                                theFxHelper.mTime - sfx.mStartTime,
                                hit_ent_num,
                                -1,
                                -1,
                            );
                        }
                        #[cfg(not(feature = "immersion"))]
                        {
                            if sfx.mEntNum != -1 {
                                // Find out where the entity currently is
                                self.CreateEffectFull(
                                    sfx.mpTemplate,
                                    (*cg_entities
                                        .as_ptr()
                                        .offset(sfx.mEntNum as isize))
                                    .lerpOrigin,
                                    sfx.mAxis.as_ptr() as *mut vec3_t,
                                    theFxHelper.mTime - sfx.mStartTime,
                                    -1,
                                    -1,
                                    -1,
                                );
                            } else {
                                self.CreateEffectFull(
                                    sfx.mpTemplate,
                                    sfx.mOrigin,
                                    sfx.mAxis.as_ptr() as *mut vec3_t,
                                    theFxHelper.mTime - sfx.mStartTime,
                                    -1,
                                    -1,
                                    -1,
                                );
                            }
                        }
                    } else {
                        // bolted on effect

                        // do we need to go and re-get the bolt matrix again? Since it takes time lets try to do it only once
                        if sfx.mModelNum as c_int != oldModelNum
                            || sfx.mEntNum as c_int != oldEntNum
                            || sfx.mBoltNum as c_int != oldBoltIndex
                        {
                            let cent = &*cg_entities.as_ptr().offset(sfx.mEntNum as isize);
                            if (*cent.gent).ghoul2.IsValid() {
                                let model_n = sfx.mModelNum as usize;
                                if model_n < (*cent.gent).ghoul2.size() as usize {
                                    if (*cent.gent).ghoul2.get(model_n).mModelindex >= 0 {
                                        doesBoltExist = theFxHelper.GetOriginAxisFromBolt(
                                            cent,
                                            sfx.mModelNum as c_int,
                                            sfx.mBoltNum as c_int,
                                            origin.as_mut_ptr(),
                                            axis.as_mut_ptr() as *mut vec3_t,
                                        );
                                    }
                                }
                            }

                            oldModelNum = sfx.mModelNum as c_int;
                            oldEntNum = sfx.mEntNum as c_int;
                            oldBoltIndex = sfx.mBoltNum as c_int;
                        }

                        // only do this if we found the bolt
                        if doesBoltExist != qfalse {
                            if sfx.mIsRelative {
                                self.CreateEffectFull(
                                    sfx.mpTemplate,
                                    vec3_origin,
                                    axis.as_mut_ptr() as *mut vec3_t,
                                    0,
                                    sfx.mEntNum as c_int,
                                    sfx.mModelNum as c_int,
                                    sfx.mBoltNum as c_int,
                                );
                            } else {
                                self.CreateEffectFull(
                                    sfx.mpTemplate,
                                    origin,
                                    axis.as_mut_ptr() as *mut vec3_t,
                                    theFxHelper.mTime - sfx.mStartTime,
                                    -1,
                                    -1,
                                    -1,
                                );
                            }
                        }
                    }

                    // Get 'em out of there.
                    // (sfx is consumed/dropped here -- equivalent to delete *itr + erase)
                } else {
                    // not yet due -- keep it
                    remaining.push_back(sfx);
                }
            } else {
                // portal flag doesn't match -- keep it
                remaining.push_back(sfx);
            }
        }

        self.mFxSchedule = remaining;

        // Add all active effects into the scene
        FX_Add(portal);

        *addr_of_mut!(gEffectsInPortal) = false;
    }
}

pub static mut gEffectsInPortal: bool = false; // this is just because I don't want to have to add an mPortalEffect field to every actual effect.

impl CFxScheduler {
    //------------------------------------------------------
    // CreateEffect (simple clientID-based version)
    //  Creates the specified fx taking into account the
    //  multitude of different ways it could be spawned.
    //
    // Input:
    //  template used to build the effect, desired effect origin,
    //  desired orientation and how late the effect is so that
    //  it can be moved to the correct spot
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn CreateEffectClientOnly(
        &mut self,
        fx: *mut CPrimitiveTemplate,
        clientID: c_int,
        delay: c_int,
    ) {
        let mut sRGB: vec3_t = core::mem::zeroed();
        let mut eRGB: vec3_t = core::mem::zeroed();
        let mut vel: vec3_t = core::mem::zeroed();
        let mut accel: vec3_t = core::mem::zeroed();
        let mut org: vec3_t = core::mem::zeroed();
        let mut org2: vec3_t = core::mem::zeroed();
        let mut flags: c_int = 0;

        // Origin calculations -- completely ignores most things
        //-------------------------------------
        VectorSet(
            org.as_mut_ptr(),
            (*fx).mOrigin1X.GetValRand(),
            (*fx).mOrigin1Y.GetValRand(),
            (*fx).mOrigin1Z.GetValRand(),
        );
        VectorSet(
            org2.as_mut_ptr(),
            (*fx).mOrigin2X.GetValRand(),
            (*fx).mOrigin2Y.GetValRand(),
            (*fx).mOrigin2Z.GetValRand(),
        );

        // handle RGB color
        if (*fx).mSpawnFlags & FX_RGB_COMPONENT_INTERP != 0 {
            let perc: f32 = random();

            VectorSet(
                sRGB.as_mut_ptr(),
                (*fx).mRedStart.GetVal(perc),
                (*fx).mGreenStart.GetVal(perc),
                (*fx).mBlueStart.GetVal(perc),
            );
            VectorSet(
                eRGB.as_mut_ptr(),
                (*fx).mRedEnd.GetVal(perc),
                (*fx).mGreenEnd.GetVal(perc),
                (*fx).mBlueEnd.GetVal(perc),
            );
        } else {
            VectorSet(
                sRGB.as_mut_ptr(),
                (*fx).mRedStart.GetValRand(),
                (*fx).mGreenStart.GetValRand(),
                (*fx).mBlueStart.GetValRand(),
            );
            VectorSet(
                eRGB.as_mut_ptr(),
                (*fx).mRedEnd.GetValRand(),
                (*fx).mGreenEnd.GetValRand(),
                (*fx).mBlueEnd.GetValRand(),
            );
        }

        // NOTE: This completely disregards a few specialty flags.
        VectorSet(
            vel.as_mut_ptr(),
            (*fx).mVelX.GetValRand(),
            (*fx).mVelY.GetValRand(),
            (*fx).mVelZ.GetValRand(),
        );
        VectorSet(
            accel.as_mut_ptr(),
            (*fx).mAccelX.GetValRand(),
            (*fx).mAccelY.GetValRand(),
            (*fx).mAccelZ.GetValRand(),
        );

        // If depth hack ISN'T already on, then turn it on.  Otherwise, we treat a pre-existing depth_hack flag as NOT being depth_hack.
        //  This is done because muzzle flash fx files are shared amongst all shooters, but for the player we need to do depth hack in first person....
        if (*fx).mFlags & FX_DEPTH_HACK == 0 && (*addr_of!(cg)).renderingThirdPerson == 0 {
            // hack!
            flags = (*fx).mFlags | FX_RELATIVE | FX_DEPTH_HACK;
        } else {
            flags = ((*fx).mFlags | FX_RELATIVE) & !FX_DEPTH_HACK;
        }

        // We only support particles for now
        //------------------------
        match (*fx).mType {
            //---------
            // Particle
            //---------
            EPrimType::Particle => {
                FX_AddParticle(
                    clientID,
                    org.as_mut_ptr(),
                    vel.as_mut_ptr(),
                    accel.as_mut_ptr(),
                    (*fx).mGravity.GetValRand(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mRotation.GetValRand(),
                    (*fx).mRotationDelta.GetValRand(),
                    (*fx).mMin.as_ptr(),
                    (*fx).mMax.as_ptr(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mDeathFxHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    flags,
                );
            }

            //---------
            // Line
            //---------
            EPrimType::Line => {
                FX_AddLine(
                    clientID,
                    org.as_mut_ptr(),
                    org2.as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    flags,
                );
            }

            //---------
            // Tail
            //---------
            EPrimType::Tail => {
                FX_AddTail(
                    clientID,
                    org.as_mut_ptr(),
                    vel.as_mut_ptr(),
                    accel.as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mLengthStart.GetValRand(),
                    (*fx).mLengthEnd.GetValRand(),
                    (*fx).mLengthParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mMin.as_ptr(),
                    (*fx).mMax.as_ptr(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mDeathFxHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    flags,
                );
            }

            //---------
            // Sound
            //---------
            EPrimType::Sound => {
                if *addr_of!(gEffectsInPortal) {
                    // could orient this anyway for panning, but eh. It's going to appear to the player in the sky the same place no matter what, so just make it a local sound.
                    theFxHelper.PlayLocalSound((*fx).mMediaHandles.GetHandle(), CHAN_AUTO);
                } else {
                    // bolted sounds actually play on the client....
                    theFxHelper.PlaySound(
                        null_mut(),
                        clientID,
                        CHAN_WEAPON,
                        (*fx).mMediaHandles.GetHandle(),
                    );
                }
            }

            // #ifdef _IMMERSION
            //---------
            // Force
            //---------
            #[cfg(feature = "immersion")]
            EPrimType::Force => {
                // Analogous to Sound (same assumption defined in RegisterForce)
                theFxHelper.PlayForce(clientID, (*fx).mMediaHandles.GetHandle());
            }
            // #endif // _IMMERSION

            //---------
            // Light
            //---------
            EPrimType::Light => {
                // don't much care if the light stays bolted...so just add it.
                if clientID >= 0 && clientID < ENTITYNUM_WORLD {
                    // ..um, ok.....
                    let cent: *mut centity_t =
                        cg_entities.as_mut_ptr().offset(clientID as isize);

                    if !cent.is_null()
                        && !(*cent).gent.is_null()
                        && !(*(*cent).gent).client.is_null()
                    {
                        FX_AddLight(
                            (*(*(*cent).gent).client).renderInfo.muzzlePoint.as_ptr(),
                            (*fx).mSizeStart.GetValRand(),
                            (*fx).mSizeEnd.GetValRand(),
                            (*fx).mSizeParm.GetValRand(),
                            sRGB.as_mut_ptr(),
                            eRGB.as_mut_ptr(),
                            (*fx).mRGBParm.GetValRand(),
                            (*fx).mLife.GetValRand(),
                            (*fx).mFlags,
                        );
                    }
                }
            }

            //---------
            // CameraShake
            //---------
            EPrimType::CameraShake => {
                if clientID >= 0 && clientID < ENTITYNUM_WORLD {
                    // ..um, ok.....
                    let cent: *mut centity_t =
                        cg_entities.as_mut_ptr().offset(clientID as isize);

                    if !cent.is_null()
                        && !(*cent).gent.is_null()
                        && !(*(*cent).gent).client.is_null()
                    {
                        theFxHelper.CameraShake(
                            (*(*cent).gent).currentOrigin.as_ptr(),
                            (*fx).mElasticity.GetValRand(),
                            (*fx).mRadius.GetValRand(),
                            (*fx).mLife.GetValRand(),
                        );
                    }
                }
            }

            _ => {}
        }

        // Track when we need to clean ourselves up if we are a copy
        if (*fx).mCopy {
            (*fx).mRefCount -= 1;

            if (*fx).mRefCount <= 0 {
                drop(Box::from_raw(fx));
            }
        }
    }

    //------------------------------------------------------
    // CreateEffect (full origin+axis version)
    //  Creates the specified fx taking into account the
    //  multitude of different ways it could be spawned.
    //
    // Input:
    //  template used to build the effect, desired effect origin,
    //  desired orientation and how late the effect is so that
    //  it can be moved to the correct spot
    //
    // Return:
    //  none
    //------------------------------------------------------
    pub unsafe fn CreateEffectFull(
        &mut self,
        fx: *mut CPrimitiveTemplate,
        origin: vec3_t,
        axis: *mut vec3_t, // vec3_t axis[3]
        lateTime: c_int,
        clientID: c_int,
        modelNum: c_int,
        boltNum: c_int,
    ) {
        let mut org: vec3_t = core::mem::zeroed();
        let mut org2: vec3_t = core::mem::zeroed();
        let mut temp: vec3_t = core::mem::zeroed();
        let mut vel: vec3_t = core::mem::zeroed();
        let mut accel: vec3_t = core::mem::zeroed();
        let mut sRGB: vec3_t = core::mem::zeroed();
        let mut eRGB: vec3_t = core::mem::zeroed();
        let mut ang: vec3_t = core::mem::zeroed();
        let mut angDelta: vec3_t = core::mem::zeroed();
        let mut ax: [vec3_t; 3] = core::mem::zeroed();
        let mut tr: trace_t = core::mem::zeroed();
        let emitterModel: c_int;

        // We may modify the axis, so make a work copy
        if !axis.is_null() {
            AxisCopy(axis, ax.as_mut_ptr() as *mut vec3_t);
        }

        let mut flags: c_int = (*fx).mFlags;
        if clientID >= 0 && modelNum >= 0 && boltNum >= 0 {
            //since you passed in these values, mark as relative to use them
            flags |= FX_RELATIVE;
        }

        if (*fx).mSpawnFlags & FX_RAND_ROT_AROUND_FWD != 0 {
            // Save ax[1] before rotating into it, because C++ passed axis[1] (the un-modified
            // source copy) as the point-to-rotate argument, avoiding aliasing.
            let ax1_orig: vec3_t = ax[1];
            RotatePointAroundVector(
                ax[1].as_mut_ptr(),
                ax[0].as_ptr(),
                ax1_orig.as_ptr(), // original axis[1]
                random() * 360.0,
            );
            CrossProduct(ax[0].as_ptr(), ax[1].as_ptr(), ax[2].as_mut_ptr());
        }

        // Origin calculations
        //-------------------------------------
        if (*fx).mSpawnFlags & FX_CHEAP_ORG_CALC != 0 || flags & FX_RELATIVE != 0 {
            // let's take the easy way out
            VectorSet(
                org.as_mut_ptr(),
                (*fx).mOrigin1X.GetValRand(),
                (*fx).mOrigin1Y.GetValRand(),
                (*fx).mOrigin1Z.GetValRand(),
            );
        } else {
            // time for some extra work
            VectorScale(ax[0].as_ptr(), (*fx).mOrigin1X.GetValRand(), org.as_mut_ptr());
            VectorMA(
                org.as_ptr(),
                (*fx).mOrigin1Y.GetValRand(),
                ax[1].as_ptr(),
                org.as_mut_ptr(),
            );
            VectorMA(
                org.as_ptr(),
                (*fx).mOrigin1Z.GetValRand(),
                ax[2].as_ptr(),
                org.as_mut_ptr(),
            );
        }

        // We always add our calculated offset to the passed in origin...
        VectorAdd(org.as_ptr(), origin.as_ptr(), org.as_mut_ptr());

        // Now, we may need to calc a point on a sphere/ellipsoid/cylinder/disk and add that to it
        //----------------------------------------------------------------
        if (*fx).mSpawnFlags & FX_ORG_ON_SPHERE != 0 {
            let x: f32;
            let y: f32;
            let width: f32;
            let height: f32;

            x = DEG2RAD(random() * 360.0);
            y = DEG2RAD(random() * 180.0);

            width = (*fx).mRadius.GetValRand();
            height = (*fx).mHeight.GetValRand();

            // calculate point on ellipse
            // sinx * siny, cosx * siny, cosy
            VectorSet(
                temp.as_mut_ptr(),
                x.sin() * width * y.sin(),
                x.cos() * width * y.sin(),
                y.cos() * height,
            );
            VectorAdd(org.as_ptr(), temp.as_ptr(), org.as_mut_ptr());

            if (*fx).mSpawnFlags & FX_AXIS_FROM_SPHERE != 0 {
                // well, we will now override the axis at the users request
                VectorNormalize2(temp.as_ptr(), ax[0].as_mut_ptr());
                MakeNormalVectors(ax[0].as_ptr(), ax[1].as_mut_ptr(), ax[2].as_mut_ptr());
            }
        } else if (*fx).mSpawnFlags & FX_ORG_ON_CYLINDER != 0 {
            let mut pt: vec3_t = core::mem::zeroed();

            // set up our point, then rotate around the current direction to.  Make unrotated cylinder centered around 0,0,0
            VectorScale(ax[1].as_ptr(), (*fx).mRadius.GetValRand(), pt.as_mut_ptr());
            VectorMA(
                pt.as_ptr(),
                crandom() * 0.5 * (*fx).mHeight.GetValRand(),
                ax[0].as_ptr(),
                pt.as_mut_ptr(),
            );
            RotatePointAroundVector(
                temp.as_mut_ptr(),
                ax[0].as_ptr(),
                pt.as_ptr(),
                random() * 360.0,
            );

            VectorAdd(org.as_ptr(), temp.as_ptr(), org.as_mut_ptr());

            if (*fx).mSpawnFlags & FX_AXIS_FROM_SPHERE != 0 {
                let mut up: vec3_t = [0.0, 0.0, 1.0];

                // well, we will now override the axis at the users request
                VectorNormalize2(temp.as_ptr(), ax[0].as_mut_ptr());

                if ax[0][2] == 1.0 {
                    // readjust up
                    VectorSet(up.as_mut_ptr(), 0.0, 1.0, 0.0);
                }

                CrossProduct(up.as_ptr(), ax[0].as_ptr(), ax[1].as_mut_ptr());
                CrossProduct(ax[0].as_ptr(), ax[1].as_ptr(), ax[2].as_mut_ptr());
            }
        }

        // There are only a few types that really use velocity and acceleration, so do extra work for those types
        //--------------------------------------------------------------------------------------------------------
        if (*fx).mType == EPrimType::Particle
            || (*fx).mType == EPrimType::OrientedParticle
            || (*fx).mType == EPrimType::Tail
            || (*fx).mType == EPrimType::Emitter
        {
            // Velocity calculations
            //-------------------------------------
            if (*fx).mSpawnFlags & FX_VEL_IS_ABSOLUTE != 0 || flags & FX_RELATIVE != 0 {
                VectorSet(
                    vel.as_mut_ptr(),
                    (*fx).mVelX.GetValRand(),
                    (*fx).mVelY.GetValRand(),
                    (*fx).mVelZ.GetValRand(),
                );
            } else {
                // bah, do some extra work to coerce it
                VectorScale(ax[0].as_ptr(), (*fx).mVelX.GetValRand(), vel.as_mut_ptr());
                VectorMA(
                    vel.as_ptr(),
                    (*fx).mVelY.GetValRand(),
                    ax[1].as_ptr(),
                    vel.as_mut_ptr(),
                );
                VectorMA(
                    vel.as_ptr(),
                    (*fx).mVelZ.GetValRand(),
                    ax[2].as_ptr(),
                    vel.as_mut_ptr(),
                );
            }

            // Acceleration calculations
            //-------------------------------------
            if (*fx).mSpawnFlags & FX_ACCEL_IS_ABSOLUTE != 0 || flags & FX_RELATIVE != 0 {
                VectorSet(
                    accel.as_mut_ptr(),
                    (*fx).mAccelX.GetValRand(),
                    (*fx).mAccelY.GetValRand(),
                    (*fx).mAccelZ.GetValRand(),
                );
            } else {
                VectorScale(ax[0].as_ptr(), (*fx).mAccelX.GetValRand(), accel.as_mut_ptr());
                VectorMA(
                    accel.as_ptr(),
                    (*fx).mAccelY.GetValRand(),
                    ax[1].as_ptr(),
                    accel.as_mut_ptr(),
                );
                VectorMA(
                    accel.as_ptr(),
                    (*fx).mAccelZ.GetValRand(),
                    ax[2].as_ptr(),
                    accel.as_mut_ptr(),
                );
            }

            // Gravity is completely decoupled from acceleration since it is __always__ absolute
            // NOTE: I only effect Z ( up/down in the Quake world )
            accel[2] += (*fx).mGravity.GetValRand();

            // There may be a lag between when the effect should be created and when it actually gets created.
            //  Since we know what the discrepancy is, we can attempt to compensate...
            if lateTime > 0 {
                // Calc the time differences
                let ftime: f32 = lateTime as f32 * 0.001;
                let time2: f32 = ftime * ftime * 0.5;

                VectorMA(vel.as_ptr(), ftime, accel.as_ptr(), vel.as_mut_ptr());

                // Predict the new position
                for i in 0..3usize {
                    org[i] = org[i] + ftime * vel[i] + time2 * vel[i];
                }
            }
        } // end moving types

        // Line type primitives work with an origin2, so do the extra work for them
        //--------------------------------------------------------------------------
        if (*fx).mType == EPrimType::Line || (*fx).mType == EPrimType::Electricity {
            // We may have to do a trace to find our endpoint
            if (*fx).mSpawnFlags & FX_ORG2_FROM_TRACE != 0 {
                VectorMA(
                    org.as_ptr(),
                    FX_MAX_TRACE_DIST as f32,
                    ax[0].as_ptr(),
                    temp.as_mut_ptr(),
                );

                if (*fx).mSpawnFlags & FX_ORG2_IS_OFFSET != 0 {
                    // add a random flair to the endpoint...note: org2 will have to be pretty large to affect this much
                    // we also do this pre-trace as opposed to post trace since we may have to render an impact effect
                    //  and we will want the normal at the exact endpos...
                    if (*fx).mSpawnFlags & FX_CHEAP_ORG2_CALC != 0 || flags & FX_RELATIVE != 0 {
                        VectorSet(
                            org2.as_mut_ptr(),
                            (*fx).mOrigin2X.GetValRand(),
                            (*fx).mOrigin2Y.GetValRand(),
                            (*fx).mOrigin2Z.GetValRand(),
                        );
                        VectorAdd(org2.as_ptr(), temp.as_ptr(), temp.as_mut_ptr());
                    } else {
                        // I can only imagine a few cases where you might want to do this...
                        VectorMA(
                            temp.as_ptr(),
                            (*fx).mOrigin2X.GetValRand(),
                            ax[0].as_ptr(),
                            temp.as_mut_ptr(),
                        );
                        VectorMA(
                            temp.as_ptr(),
                            (*fx).mOrigin2Y.GetValRand(),
                            ax[1].as_ptr(),
                            temp.as_mut_ptr(),
                        );
                        VectorMA(
                            temp.as_ptr(),
                            (*fx).mOrigin2Z.GetValRand(),
                            ax[2].as_ptr(),
                            temp.as_mut_ptr(),
                        );
                    }
                }

                theFxHelper.Trace(
                    &mut tr,
                    org.as_ptr(),
                    null_mut(),
                    null_mut(),
                    temp.as_ptr(),
                    -1,
                    CONTENTS_SOLID | CONTENTS_SHOTCLIP,
                ); //MASK_SHOT );

                if tr.startsolid != 0 || tr.allsolid != 0 {
                    VectorCopy(org.as_ptr(), org2.as_mut_ptr()); // this is not a very good solution
                } else {
                    VectorCopy(tr.endpos.as_ptr(), org2.as_mut_ptr());
                }

                if (*fx).mSpawnFlags & FX_TRACE_IMPACT_FX != 0 {
                    self.PlayEffect_OrgFwd(
                        (*fx).mImpactFxHandles.GetHandle(),
                        org2,
                        tr.plane.normal,
                        false,
                    );
                }
            } else {
                if (*fx).mSpawnFlags & FX_CHEAP_ORG2_CALC != 0 || flags & FX_RELATIVE != 0 {
                    VectorSet(
                        org2.as_mut_ptr(),
                        (*fx).mOrigin2X.GetValRand(),
                        (*fx).mOrigin2Y.GetValRand(),
                        (*fx).mOrigin2Z.GetValRand(),
                    );
                } else {
                    VectorScale(
                        ax[0].as_ptr(),
                        (*fx).mOrigin2X.GetValRand(),
                        org2.as_mut_ptr(),
                    );
                    VectorMA(
                        org2.as_ptr(),
                        (*fx).mOrigin2Y.GetValRand(),
                        ax[1].as_ptr(),
                        org2.as_mut_ptr(),
                    );
                    VectorMA(
                        org2.as_ptr(),
                        (*fx).mOrigin2Z.GetValRand(),
                        ax[2].as_ptr(),
                        org2.as_mut_ptr(),
                    );

                    VectorAdd(org2.as_ptr(), origin.as_ptr(), org2.as_mut_ptr());
                }
            }
        } // end special org2 types

        // handle RGB color, but only for types that will use it
        //---------------------------------------------------------------------------
        // #ifdef _IMMERSION
        // if ( fx->mType != Sound && fx->mType != FxRunner && fx->mType != CameraShake && fx->mType != Force )
        // #else
        // if ( fx->mType != Sound && fx->mType != FxRunner && fx->mType != CameraShake )
        // #endif // _IMMERSION
        let needs_rgb = (*fx).mType != EPrimType::Sound
            && (*fx).mType != EPrimType::FxRunner
            && (*fx).mType != EPrimType::CameraShake
            && {
                #[cfg(feature = "immersion")]
                { (*fx).mType != EPrimType::Force }
                #[cfg(not(feature = "immersion"))]
                { true }
            };

        if needs_rgb {
            if (*fx).mSpawnFlags & FX_RGB_COMPONENT_INTERP != 0 {
                let perc: f32 = random();

                VectorSet(
                    sRGB.as_mut_ptr(),
                    (*fx).mRedStart.GetVal(perc),
                    (*fx).mGreenStart.GetVal(perc),
                    (*fx).mBlueStart.GetVal(perc),
                );
                VectorSet(
                    eRGB.as_mut_ptr(),
                    (*fx).mRedEnd.GetVal(perc),
                    (*fx).mGreenEnd.GetVal(perc),
                    (*fx).mBlueEnd.GetVal(perc),
                );
            } else {
                VectorSet(
                    sRGB.as_mut_ptr(),
                    (*fx).mRedStart.GetValRand(),
                    (*fx).mGreenStart.GetValRand(),
                    (*fx).mBlueStart.GetValRand(),
                );
                VectorSet(
                    eRGB.as_mut_ptr(),
                    (*fx).mRedEnd.GetValRand(),
                    (*fx).mGreenEnd.GetValRand(),
                    (*fx).mBlueEnd.GetValRand(),
                );
            }
        }

        // Now create the appropriate effect entity
        //------------------------
        match (*fx).mType {
            //---------
            // Particle
            //---------
            EPrimType::Particle => {
                FX_AddParticle(
                    clientID,
                    org.as_mut_ptr(),
                    vel.as_mut_ptr(),
                    accel.as_mut_ptr(),
                    (*fx).mGravity.GetValRand(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mRotation.GetValRand(),
                    (*fx).mRotationDelta.GetValRand(),
                    (*fx).mMin.as_ptr(),
                    (*fx).mMax.as_ptr(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mDeathFxHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    flags,
                    modelNum,
                    boltNum,
                );
            }

            //---------
            // Line
            //---------
            EPrimType::Line => {
                FX_AddLine(
                    clientID,
                    org.as_mut_ptr(),
                    org2.as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    flags,
                    modelNum,
                    boltNum,
                );
            }

            //---------
            // Tail
            //---------
            EPrimType::Tail => {
                FX_AddTail(
                    clientID,
                    org.as_mut_ptr(),
                    vel.as_mut_ptr(),
                    accel.as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mLengthStart.GetValRand(),
                    (*fx).mLengthEnd.GetValRand(),
                    (*fx).mLengthParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mMin.as_ptr(),
                    (*fx).mMax.as_ptr(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mDeathFxHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    flags,
                    modelNum,
                    boltNum,
                );
            }

            //----------------
            // Electricity
            //----------------
            EPrimType::Electricity => {
                FX_AddElectricity(
                    clientID,
                    org.as_mut_ptr(),
                    org2.as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    flags,
                    modelNum,
                    boltNum,
                );
            }

            //---------
            // Cylinder
            //---------
            EPrimType::Cylinder => {
                FX_AddCylinder(
                    clientID,
                    org.as_mut_ptr(),
                    ax[0].as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mSize2Start.GetValRand(),
                    (*fx).mSize2End.GetValRand(),
                    (*fx).mSize2Parm.GetValRand(),
                    (*fx).mLengthStart.GetValRand(),
                    (*fx).mLengthEnd.GetValRand(),
                    (*fx).mLengthParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    flags,
                    modelNum,
                    boltNum,
                );
            }

            //---------
            // Emitter
            //---------
            EPrimType::Emitter => {
                // for chunk angles, you don't really need much control over the end result...you just want variation..
                VectorSet(
                    ang.as_mut_ptr(),
                    (*fx).mAngle1.GetValRand(),
                    (*fx).mAngle2.GetValRand(),
                    (*fx).mAngle3.GetValRand(),
                );

                vectoangles(ax[0].as_ptr(), temp.as_mut_ptr());
                VectorAdd(ang.as_ptr(), temp.as_ptr(), ang.as_mut_ptr());

                VectorSet(
                    angDelta.as_mut_ptr(),
                    (*fx).mAngle1Delta.GetValRand(),
                    (*fx).mAngle2Delta.GetValRand(),
                    (*fx).mAngle3Delta.GetValRand(),
                );

                emitterModel = (*fx).mMediaHandles.GetHandle();

                FX_AddEmitter(
                    org.as_mut_ptr(),
                    vel.as_mut_ptr(),
                    accel.as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    ang.as_mut_ptr(),
                    angDelta.as_mut_ptr(),
                    (*fx).mMin.as_ptr(),
                    (*fx).mMax.as_ptr(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mDeathFxHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    (*fx).mEmitterFxHandles.GetHandle(),
                    (*fx).mDensity.GetValRand(),
                    (*fx).mVariance.GetValRand(),
                    (*fx).mLife.GetValRand(),
                    emitterModel,
                    flags,
                );
            }

            //---------
            // Decal
            //---------
            EPrimType::Decal => {
                // I'm calling this function ( at least for now ) because it handles projecting
                //  the decal mark onto the surfaces properly.  This is especially important for large marks.
                // The downside is that it's much less flexible....
                CG_ImpactMark(
                    (*fx).mMediaHandles.GetHandle(),
                    org.as_ptr(),
                    ax[0].as_ptr(),
                    (*fx).mRotation.GetValRand(),
                    sRGB[0],
                    sRGB[1],
                    sRGB[2],
                    (*fx).mAlphaStart.GetValRand(),
                    qtrue,
                    (*fx).mSizeStart.GetValRand(),
                    qfalse,
                );

                if (*fx).mFlags & FX_GHOUL2_DECALS != 0 {
                    let mut tr2: trace_t = core::mem::zeroed();
                    let mut end: vec3_t = core::mem::zeroed();

                    VectorMA(org.as_ptr(), 64.0, ax[0].as_ptr(), end.as_mut_ptr());

                    theFxHelper.G2Trace(
                        &mut tr2,
                        org.as_ptr(),
                        null_mut(),
                        null_mut(),
                        end.as_ptr(),
                        ENTITYNUM_NONE,
                        MASK_PLAYERSOLID,
                    );

                    if tr2.entityNum < ENTITYNUM_WORLD
                        && (*g_entities.as_ptr().offset(tr2.entityNum as isize))
                            .ghoul2
                            .size()
                            > 0
                    {
                        let ent: *mut gentity_t =
                            g_entities.as_mut_ptr().offset(tr2.entityNum as isize);

                        if !ent.is_null() {
                            let mut entOrg: vec3_t = core::mem::zeroed();
                            let mut hitDir: vec3_t = core::mem::zeroed();
                            let entYaw: f32;
                            let firstModel: f32 = 0.0;
                            if (*ent).s.eFlags & EF_NODRAW == 0 {
                                //not drawn, no marks
                                if !(*ent).client.is_null() {
                                    VectorCopy(
                                        (*(*ent).client).ps.origin.as_ptr(),
                                        entOrg.as_mut_ptr(),
                                    );
                                } else {
                                    VectorCopy(
                                        (*ent).currentOrigin.as_ptr(),
                                        entOrg.as_mut_ptr(),
                                    );
                                }
                                if !(*ent).client.is_null() {
                                    entYaw = (*(*ent).client).ps.viewangles[YAW as usize];
                                } else {
                                    entYaw = (*ent).currentAngles[YAW as usize];
                                }
                                //if ( VectorCompare( tr.plane.normal, vec3_origin ) )
                                {
                                    //hunh, no plane?  Use trace dir
                                    VectorCopy(ax[0].as_ptr(), hitDir.as_mut_ptr());
                                }
                                /*
                                else
                                {
                                    VectorCopy( tr.plane.normal, hitDir );
                                }
                                */

                                CG_AddGhoul2Mark(
                                    (*fx).mMediaHandles.GetHandle(),
                                    (*fx).mSizeStart.GetValRand(),
                                    tr2.endpos.as_ptr(),
                                    tr2.plane.normal.as_ptr(),
                                    tr2.entityNum,
                                    entOrg.as_ptr(),
                                    entYaw,
                                    addr_of_mut!((*ent).ghoul2),
                                    (*ent).s.modelScale.as_ptr(),
                                    Q_irand(40000, 60000),
                                    firstModel as c_int,
                                );
                            }
                        }
                    }
                }
            }

            //-------------------
            // OrientedParticle
            //-------------------
            EPrimType::OrientedParticle => {
                FX_AddOrientedParticle(
                    clientID,
                    org.as_mut_ptr(),
                    ax[0].as_mut_ptr(),
                    vel.as_mut_ptr(),
                    accel.as_mut_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    (*fx).mAlphaStart.GetValRand(),
                    (*fx).mAlphaEnd.GetValRand(),
                    (*fx).mAlphaParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mRotation.GetValRand(),
                    (*fx).mRotationDelta.GetValRand(),
                    (*fx).mMin.as_ptr(),
                    (*fx).mMax.as_ptr(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mDeathFxHandles.GetHandle(),
                    (*fx).mImpactFxHandles.GetHandle(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    flags,
                    modelNum,
                    boltNum,
                );
            }

            //---------
            // Sound
            //---------
            EPrimType::Sound => {
                if *addr_of!(gEffectsInPortal) {
                    // could orient this anyway for panning, but eh. It's going to appear to the player in the sky the same place no matter what, so just make it a local sound.
                    theFxHelper.PlayLocalSound((*fx).mMediaHandles.GetHandle(), CHAN_AUTO);
                } else if (*fx).mSpawnFlags & FX_SND_LESS_ATTENUATION != 0 {
                    theFxHelper.PlaySound(
                        org.as_ptr(),
                        ENTITYNUM_NONE,
                        CHAN_LESS_ATTEN,
                        (*fx).mMediaHandles.GetHandle(),
                    );
                } else {
                    theFxHelper.PlaySound(
                        org.as_ptr(),
                        ENTITYNUM_NONE,
                        CHAN_AUTO,
                        (*fx).mMediaHandles.GetHandle(),
                    );
                }
            }

            // #ifdef _IMMERSION
            //---------
            // Force
            //---------
            #[cfg(feature = "immersion")]
            EPrimType::Force => {
                if clientID > -1 {
                    // Fix me: Allow or abolish FF_LOCAL_CLIENT?
                    theFxHelper.PlayForce(clientID, (*fx).mMediaHandles.GetHandle());
                }
            }
            // #endif // _IMMERSION

            //---------
            // FxRunner
            //---------
            EPrimType::FxRunner => {
                self.PlayEffect_Full(
                    (*fx).mPlayFxHandles.GetHandle(),
                    org,
                    ax.as_mut_ptr() as *mut vec3_t,
                    -1,
                    -1,
                    false,
                    0,
                    false,
                );
            }

            //---------
            // Light
            //---------
            EPrimType::Light => {
                FX_AddLight(
                    org.as_ptr(),
                    (*fx).mSizeStart.GetValRand(),
                    (*fx).mSizeEnd.GetValRand(),
                    (*fx).mSizeParm.GetValRand(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mFlags,
                );
            }

            //---------
            // CameraShake
            //---------
            EPrimType::CameraShake => {
                // It calculates how intense the shake should be based on how close you are to the origin you pass in here
                //  elasticity is actually the intensity...radius is the distance in which the shake will have some effect
                //  life is how long the effect lasts.
                theFxHelper.CameraShake(
                    org.as_ptr(),
                    (*fx).mElasticity.GetValRand(),
                    (*fx).mRadius.GetValRand(),
                    (*fx).mLife.GetValRand(),
                );
            }

            //--------------
            // ScreenFlash
            //--------------
            EPrimType::ScreenFlash => {
                FX_AddFlash(
                    org.as_ptr(),
                    sRGB.as_mut_ptr(),
                    eRGB.as_mut_ptr(),
                    (*fx).mRGBParm.GetValRand(),
                    (*fx).mLife.GetValRand(),
                    (*fx).mMediaHandles.GetHandle(),
                    (*fx).mFlags,
                );
            }

            _ => {}
        }

        // Track when we need to clean ourselves up if we are a copy
        if (*fx).mCopy {
            (*fx).mRefCount -= 1;

            if (*fx).mRefCount <= 0 {
                drop(Box::from_raw(fx));
            }
        }
    }
}

// for loadsave...
//
pub unsafe fn FX_Read() {
    let sched = (*addr_of_mut!(theFxScheduler)).as_mut().unwrap();
    sched.LoadSave_Read();
}

// for loadsave...
//
pub unsafe fn FX_Write() {
    let sched = (*addr_of_mut!(theFxScheduler)).as_mut().unwrap();
    sched.LoadSave_Write();
}

pub unsafe fn FX_CopeWithAnyLoadedSaveGames() {
    let sched = (*addr_of_mut!(theFxScheduler)).as_mut().unwrap();
    sched.FX_CopeWithAnyLoadedSaveGames();
}
