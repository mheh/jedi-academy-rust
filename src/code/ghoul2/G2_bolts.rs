// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};
use core::mem::size_of;
use core::ptr::addr_of_mut;

// External C functions
extern "C" {
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn G2_Bolt_Not_Found(boneName: *const c_char, modName: *const c_char);
}

// G2_MODEL_OK macro: ((g)&&(g)->mValid&&(g)->aHeader&&(g)->currentModel&&(g)->animModel)
#[inline]
fn G2_MODEL_OK(g: *const CGhoul2Info) -> bool {
    !g.is_null() && unsafe {
        let g_ref = &*g;
        g_ref.mValid && !g_ref.aHeader.is_null() && !g_ref.currentModel.is_null() && !g_ref.animModel.is_null()
    }
}

//=====================================================================================================================
// Bolt List handling routines - so entities can attach themselves to any part of the model in question

// Given a bone number, see if that bone is already in our bone list
pub fn G2_Find_Bolt_Bone_Num(bltlist: &Vec<boltInfo_t>, boneNum: c_int) -> c_int {
    let mut i: c_int = 0;

    // look through entire list
    while i < (bltlist.len() as c_int) {
        if bltlist[i as usize].boneNumber == boneNum {
            return i;
        }
        i += 1;
    }

    // didn't find it
    return -1;
}

// Given a bone number, see if that surface is already in our surfacelist list
pub fn G2_Find_Bolt_Surface_Num(bltlist: &Vec<boltInfo_t>, surfaceNum: c_int, flags: c_int) -> c_int {
    let mut i: c_int = 0;

    // look through entire list
    while i < (bltlist.len() as c_int) {
        if (bltlist[i as usize].surfaceNumber == surfaceNum) && ((bltlist[i as usize].surfaceType & flags) == flags) {
            return i;
        }
        i += 1;
    }

    // didn't find it
    return -1;
}

//=========================================================================================
//// Public Bolt Routines
pub fn G2_Add_Bolt_Surf_Num(
    ghlInfo: *mut CGhoul2Info,
    bltlist: &mut Vec<boltInfo_t>,
    slist: &Vec<surfaceInfo_t>,
    surfNum: c_int,
) -> c_int {
    assert!(!ghlInfo.is_null() && unsafe { (*ghlInfo).mValid });
    let mut tempBolt = boltInfo_t::default();
    let mut i: c_int = 0;

    assert!(surfNum >= 0 && (surfNum as usize) < slist.len());
    // ensure surface num is valid
    if (surfNum as usize) >= slist.len() {
        return -1;
    }

    // look through entire list - see if it's already there first
    i = 0;
    while (i as usize) < bltlist.len() {
        // already there??
        if bltlist[i as usize].surfaceNumber == surfNum {
            // increment the usage count
            bltlist[i as usize].boltUsed += 1;
            return i;
        }
        i += 1;
    }

    // we have a surface
    // look through entire list - see if it's already there first
    i = 0;
    while (i as usize) < bltlist.len() {
        // if this surface entry has info in it, bounce over it
        if bltlist[i as usize].boneNumber == -1 && bltlist[i as usize].surfaceNumber == -1 {
            // if we found an entry that had a -1 for the bone / surface number, then we hit a surface / bone slot that was empty
            bltlist[i as usize].surfaceNumber = surfNum;
            bltlist[i as usize].surfaceType = G2SURFACEFLAG_GENERATED;
            bltlist[i as usize].boltUsed = 1;
            return i;
        }
        i += 1;
    }

    // ok, we didn't find an existing surface of that name, or an empty slot. Lets add an entry
    tempBolt.surfaceNumber = surfNum;
    tempBolt.surfaceType = G2SURFACEFLAG_GENERATED;
    tempBolt.boneNumber = -1;
    tempBolt.boltUsed = 1;
    bltlist.push_back(tempBolt);
    return (bltlist.len() - 1) as c_int;
}

pub fn G2_Add_Bolt(
    ghlInfo: *mut CGhoul2Info,
    bltlist: &mut Vec<boltInfo_t>,
    slist: &Vec<surfaceInfo_t>,
    boneName: *const c_char,
) -> c_int {
    assert!(!ghlInfo.is_null() && unsafe { (*ghlInfo).mValid });
    let mut i: c_int = 0;
    let mut x: c_int = 0;
    let mut surfNum: c_int = -1;
    let mut skel: *mut mdxaSkel_t;
    let mut offsets: *mut mdxaSkelOffsets_t;
    let mut tempBolt = boltInfo_t::default();
    let mut flags: c_int = 0;

    assert!(G2_MODEL_OK(ghlInfo));

    unsafe {
        let _surfOffsets = ((*(*ghlInfo).currentModel).mdxm as *mut u8)
            .add(size_of::<mdxmHeader_t>()) as *mut mdxmHierarchyOffsets_t;
        // first up, we'll search for that which this bolt names in all the surfaces
        surfNum = G2_IsSurfaceLegal((*ghlInfo).currentModel, boneName, addr_of_mut!(flags));

        // did we find it as a surface?
        if surfNum != -1 {
            // look through entire list - see if it's already there first
            i = 0;
            while (i as usize) < bltlist.len() {
                // already there??
                if bltlist[i as usize].surfaceNumber == surfNum {
                    // increment the usage count
                    bltlist[i as usize].boltUsed += 1;
                    return i;
                }
                i += 1;
            }

            // look through entire list - see if we can re-use one
            i = 0;
            while (i as usize) < bltlist.len() {
                // if this surface entry has info in it, bounce over it
                if bltlist[i as usize].boneNumber == -1 && bltlist[i as usize].surfaceNumber == -1 {
                    // if we found an entry that had a -1 for the bone / surface number, then we hit a surface / bone slot that was empty
                    bltlist[i as usize].surfaceNumber = surfNum;
                    bltlist[i as usize].boltUsed = 1;
                    bltlist[i as usize].surfaceType = 0;
                    return i;
                }
                i += 1;
            }

            // ok, we didn't find an existing surface of that name, or an empty slot. Lets add an entry
            tempBolt.surfaceNumber = surfNum;
            tempBolt.boneNumber = -1;
            tempBolt.boltUsed = 1;
            tempBolt.surfaceType = 0;
            bltlist.push_back(tempBolt);
            return (bltlist.len() - 1) as c_int;
        }

        // no, check to see if it's a bone then

        offsets = ((*ghlInfo).aHeader as *mut u8).add(size_of::<mdxaHeader_t>()) as *mut mdxaSkelOffsets_t;

        // walk the entire list of bones in the gla file for this model and see if any match the name of the bone we want to find
        x = 0;
        while x < (*(*ghlInfo).aHeader).numBones {
            skel = ((*ghlInfo).aHeader as *mut u8)
                .add(size_of::<mdxaHeader_t>())
                .add((*offsets).offsets[x as usize] as usize) as *mut mdxaSkel_t;
            // if name is the same, we found it
            if stricmp((*skel).name, boneName) == 0 {
                break;
            }
            x += 1;
        }

        // check to see we did actually make a match with a bone in the model
        if x == (*(*ghlInfo).aHeader).numBones {
            // didn't find it? Error
            //assert(0&&x == mod_a->mdxa->numBones);
            #[cfg(debug_assertions)]
            {
                G2_Bolt_Not_Found(boneName, (*ghlInfo).mFileName);
            }
            return -1;
        }

        // look through entire list - see if it's already there first
        i = 0;
        while (i as usize) < bltlist.len() {
            // already there??
            if bltlist[i as usize].boneNumber == x {
                // increment the usage count
                bltlist[i as usize].boltUsed += 1;
                return i;
            }
            i += 1;
        }

        // look through entire list - see if we can re-use it
        i = 0;
        while (i as usize) < bltlist.len() {
            // if this bone entry has info in it, bounce over it
            if bltlist[i as usize].boneNumber == -1 && bltlist[i as usize].surfaceNumber == -1 {
                // if we found an entry that had a -1 for the bonenumber, then we hit a bone slot that was empty
                bltlist[i as usize].boneNumber = x;
                bltlist[i as usize].boltUsed = 1;
                bltlist[i as usize].surfaceType = 0;
                return i;
            }
            i += 1;
        }

        // ok, we didn't find an existing bone of that name, or an empty slot. Lets add an entry
        tempBolt.boneNumber = x;
        tempBolt.surfaceNumber = -1;
        tempBolt.boltUsed = 1;
        tempBolt.surfaceType = 0;
        bltlist.push_back(tempBolt);
        return (bltlist.len() - 1) as c_int;
    }
}

// Given a model handle, and a bone name, we want to remove this bone from the bone override list
pub fn G2_Remove_Bolt(bltlist: &mut Vec<boltInfo_t>, index: c_int) -> qboolean {
    assert!(index >= 0 && (index as usize) < bltlist.len());
    // did we find it?
    if index != -1 {
        bltlist[index as usize].boltUsed -= 1;
        if bltlist[index as usize].boltUsed == 0 {
            // set this bone to not used
            bltlist[index as usize].boneNumber = -1;
            bltlist[index as usize].surfaceNumber = -1;
        }
        return qtrue;
    }
    return qfalse;
}

// set the bolt list to all unused so the bone transformation routine ignores it.
pub fn G2_Init_Bolt_List(bltlist: &mut Vec<boltInfo_t>) {
    bltlist.clear();
}
