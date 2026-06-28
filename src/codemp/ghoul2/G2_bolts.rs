// leave this as first line for PCH reasons...
//

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
//
// #if !defined(TR_LOCAL_H)
// 	#include "../renderer/tr_local.h"
// #endif
//
// #if !defined(G2_H_INC)
// 	#include "G2.h"
// #endif
// 	#include "G2_local.h"

use core::ffi::{c_int, c_char, c_void};

//=====================================================================================================================
// Bolt List handling routines - so entities can attach themselves to any part of the model in question

// Given a bone number, see if that bone is already in our bone list
pub fn G2_Find_Bolt_Bone_Num(bltlist: &[boltInfo_t], boneNum: c_int) -> c_int
{
	let mut i: c_int;

	// look through entire list
	i = 0;
	while i < bltlist.len() as c_int
	{
		// if this bone entry has no info in it, bounce over it
		if bltlist[i as usize].boneNumber == -1
		{
			i += 1;
			continue;
		}

		if bltlist[i as usize].boneNumber == boneNum
		{
			return i;
		}
		i += 1;
	}

	// didn't find it
	return -1;
}

// Given a bone number, see if that surface is already in our surfacelist list
pub fn G2_Find_Bolt_Surface_Num(bltlist: &[boltInfo_t], surfaceNum: c_int, flags: c_int) -> c_int
{
	let mut i: c_int;

	// look through entire list
	i = 0;
	while i < bltlist.len() as c_int
	{
		// if this bone entry has no info in it, bounce over it
		if bltlist[i as usize].surfaceNumber == -1
		{
			i += 1;
			continue;
		}

		if (bltlist[i as usize].surfaceNumber == surfaceNum) && ((bltlist[i as usize].surfaceType & flags) == flags)
		{
			return i;
		}
		i += 1;
	}

	// didn't find it
	return -1;
}

//=========================================================================================
//// Public Bolt Routines
pub fn G2_Add_Bolt_Surf_Num(ghlInfo: *mut CGhoul2Info, bltlist: &mut Vec<boltInfo_t>, slist: &[surfaceInfo_t], surfNum: c_int) -> c_int
{
	debug_assert!(unsafe { !ghlInfo.is_null() && (*ghlInfo).mValid != 0 });
	let mut tempBolt: boltInfo_t;
	let mut i: c_int;

	// first up, make sure have a surface first
	if surfNum >= slist.len() as c_int
	{
		return -1;
	}

	 // look through entire list - see if it's already there first
	i = 0;
	while i < bltlist.len() as c_int
	{
		// already there??
		if bltlist[i as usize].surfaceNumber == surfNum
		{
			// increment the usage count
			bltlist[i as usize].boltUsed += 1;
			return i;
		}
		i += 1;
	}

	// we have a surface
	// look through entire list - see if it's already there first
	i = 0;
	while i < bltlist.len() as c_int
	{
		// if this surface entry has info in it, bounce over it
	  	if bltlist[i as usize].boneNumber == -1 && bltlist[i as usize].surfaceNumber == -1
		{
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

pub fn G2_Add_Bolt(ghlInfo: *mut CGhoul2Info, bltlist: &mut Vec<boltInfo_t>, slist: &[surfaceInfo_t], boneName: *const c_char) -> c_int
{
	debug_assert!(unsafe { !ghlInfo.is_null() && (*ghlInfo).mValid != 0 });
	let mod_m: *mut model_t = unsafe { (*ghlInfo).currentModel as *mut model_t };
	let mod_a: *mut model_t = unsafe { (*ghlInfo).animModel as *mut model_t };
	let mut i: c_int;
	let mut x: c_int;
	let mut surfNum: c_int = -1;
	let mut skel: *mut mdxaSkel_t;
	let mut offsets: *mut mdxaSkelOffsets_t;
	let mut surfOffsets: *mut mdxmHierarchyOffsets_t;
	let mut tempBolt: boltInfo_t;
	let mut flags: c_int = 0;

	unsafe {
		surfOffsets = ((*mod_m).mdxm as *mut u8).add(core::mem::size_of::<mdxmHeader_t>()) as *mut mdxmHierarchyOffsets_t;
	}
	// first up, we'll search for that which this bolt names in all the surfaces
	surfNum = G2_IsSurfaceLegal(mod_m as *const c_void, boneName, &mut flags);

	// did we find it as a surface?
	if surfNum != -1
	{
		 // look through entire list - see if it's already there first
		i = 0;
		while i < bltlist.len() as c_int
		{
			// already there??
			if bltlist[i as usize].surfaceNumber == surfNum
			{
				// increment the usage count
				bltlist[i as usize].boltUsed += 1;
				return i;
			}
			i += 1;
		}

		 // look through entire list - see if we can re-use one
		i = 0;
		while i < bltlist.len() as c_int
		{
			// if this surface entry has info in it, bounce over it
		  	if bltlist[i as usize].boneNumber == -1 && bltlist[i as usize].surfaceNumber == -1
			{
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

   	unsafe {
		offsets = ((*mod_a).mdxa as *mut u8).add(core::mem::size_of::<mdxaHeader_t>()) as *mut mdxaSkelOffsets_t;
	}

 	// walk the entire list of bones in the gla file for this model and see if any match the name of the bone we want to find
 	x = 0;
 	while x < unsafe { (*(*mod_a).mdxa).numBones }
 	{
 		unsafe {
 			skel = ((*mod_a).mdxa as *mut u8).add(core::mem::size_of::<mdxaHeader_t>()).add((*offsets).offsets[x as usize] as usize) as *mut mdxaSkel_t;
 		}
 		// if name is the same, we found it
 		if unsafe { stricmp((*skel).name.as_ptr(), boneName) == 0 }
		{
			break;
		}
		x += 1;
	}

	// check to see we did actually make a match with a bone in the model
	if x == unsafe { (*(*mod_a).mdxa).numBones }
	{
		// didn't find it? Error
		//assert(0&&x == mod_a->mdxa->numBones);
#[cfg(debug_assertions)]
		//		Com_Printf("WARNING: %s not found on skeleton\n", boneName);
		return -1;
	}

	// look through entire list - see if it's already there first
	i = 0;
	while i < bltlist.len() as c_int
	{
		// already there??
		if bltlist[i as usize].boneNumber == x
		{
			// increment the usage count
			bltlist[i as usize].boltUsed += 1;
			return i;
		}
		i += 1;
	}

	// look through entire list - see if we can re-use it
	i = 0;
	while i < bltlist.len() as c_int
	{
		// if this bone entry has info in it, bounce over it
		if bltlist[i as usize].boneNumber == -1 && bltlist[i as usize].surfaceNumber == -1
		{
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

// Given a model handle, and a bone name, we want to remove this bone from the bone override list
pub fn G2_Remove_Bolt (bltlist: &mut Vec<boltInfo_t>, index: c_int) -> c_int
{
	// did we find it?
	if index != -1
	{
		bltlist[index as usize].boltUsed -= 1;
		if bltlist[index as usize].boltUsed == 0
		{
			// set this bone to not used
			bltlist[index as usize].boneNumber = -1;
			bltlist[index as usize].surfaceNumber = -1;

			let mut newSize: usize = bltlist.len();
			// now look through the list from the back and see if there is a block of -1's we can resize off the end of the list
			let mut i: i32 = bltlist.len() as i32 - 1;
			while i > -1
			{
				if (bltlist[i as usize].surfaceNumber == -1) && (bltlist[i as usize].boneNumber == -1)
				{
					newSize = i as usize;
				}
				// once we hit one that isn't a -1, we are done.
				else
				{
					break;
				}
				i -= 1;
			}
			// do we need to resize?
			if newSize != bltlist.len()
			{
				// yes, so lets do it
				bltlist.resize(newSize, Default::default());
			}

		}
		return 1;
	}

	debug_assert!(false, "G2_Remove_Bolt: invalid index");

	// no
	return 0;
}

// set the bolt list to all unused so the bone transformation routine ignores it.
pub fn G2_Init_Bolt_List(bltlist: &mut Vec<boltInfo_t>)
{
	bltlist.clear();
}

// remove any bolts that reference original surfaces, generated surfaces, or bones that aren't active anymore
pub fn G2_RemoveRedundantBolts(bltlist: &mut Vec<boltInfo_t>, slist: &[surfaceInfo_t], activeSurfaces: *const c_int, activeBones: *const c_int)
{
	// walk the bolt list
	let mut i: usize = 0;
	while i < bltlist.len()
	{
		// are we using this bolt?
		if (bltlist[i].surfaceNumber != -1) || (bltlist[i].boneNumber != -1)
		{
			// is this referenceing a surface?
			if bltlist[i].surfaceNumber != -1
			{
				// is this bolt looking at a generated surface?
				if bltlist[i].surfaceType != 0
				{
					// yes, so look for it in the surface list
					if G2_FindOverrideSurface(bltlist[i].surfaceNumber, slist) == 0
					{
						// no - we want to remove this bolt, regardless of how many people are using it
						bltlist[i].boltUsed = 1;
						G2_Remove_Bolt(bltlist, i as c_int);
					}
				}
				// no, it's an original, so look for it in the active surfaces list
				{
					unsafe {
						if (*activeSurfaces.add(bltlist[i].surfaceNumber as usize)) == 0
						{
							// no - we want to remove this bolt, regardless of how many people are using it
							bltlist[i].boltUsed = 1;
							G2_Remove_Bolt(bltlist, i as c_int);
						}
					}
				}
			}
			// no, must be looking at a bone then
			else
			{
				// is that bone active then?
				unsafe {
					if (*activeBones.add(bltlist[i].boneNumber as usize)) == 0
					{
						// no - we want to remove this bolt, regardless of how many people are using it
						bltlist[i].boltUsed = 1;
						G2_Remove_Bolt(bltlist, i as c_int);
					}
				}
			}
		}
		i += 1;
	}
}

// === Local type stubs for structural coherence ===

#[repr(C)]
pub struct boltInfo_t {
	pub boneNumber: c_int,
	pub surfaceNumber: c_int,
	pub surfaceType: c_int,
	pub boltUsed: c_int,
}

impl Default for boltInfo_t {
	fn default() -> Self {
		boltInfo_t {
			boneNumber: 0,
			surfaceNumber: 0,
			surfaceType: 0,
			boltUsed: 0,
		}
	}
}

#[repr(C)]
pub struct surfaceInfo_t {
	// Stub - fields not specified in this module
	pub _unused: c_int,
}

#[repr(C)]
pub struct CGhoul2Info {
	pub mValid: c_int,
	pub currentModel: *mut c_void,
	pub animModel: *mut c_void,
	// Other fields omitted
}

#[repr(C)]
pub struct model_t {
	pub mdxm: *mut c_void,
	pub mdxa: *mut mdxaHeader_t,
	// Other fields omitted
}

#[repr(C)]
pub struct mdxaHeader_t {
	pub numBones: c_int,
	// Other fields omitted
}

#[repr(C)]
pub struct mdxmHeader_t {
	// Fields not specified
}

#[repr(C)]
pub struct mdxaSkel_t {
	pub name: [c_char; 64],
	// Other fields omitted
}

#[repr(C)]
pub struct mdxaSkelOffsets_t {
	pub offsets: [c_int; 1000],
	// Fields not specified
}

#[repr(C)]
pub struct mdxmHierarchyOffsets_t {
	// Fields not specified
}

pub const G2SURFACEFLAG_GENERATED: c_int = 1;

// External function declarations
extern "C" {
	pub fn G2_IsSurfaceLegal(mod_m: *const c_void, boneName: *const c_char, flags: *mut c_int) -> c_int;
	pub fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
	pub fn G2_FindOverrideSurface(surfNum: c_int, slist: &[surfaceInfo_t]) -> c_int;
}
