#![allow(non_snake_case)]

// leave this as first line for PCH reasons...
//

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #if !defined(TR_LOCAL_H)
// #include "../renderer/tr_local.h"
// #endif

// #if !defined(G2_H_INC)
// #include "G2.h"
// #endif
// #include "G2_local.h"
// #pragma warning(disable : 4512)		//assignment op could not be genereated

use core::ffi::{c_int, c_char, c_void};
use std::os::raw::c_uint;

// ============================================================================
// LOCAL TYPE STUBS
// These are forward declarations for types defined in external headers.
// ============================================================================

#[repr(C)]
pub struct surfaceInfo_t {
    pub surface: c_int,
    pub offFlags: c_int,
    pub genBarycentricI: f32,
    pub genBarycentricJ: f32,
    pub genPolySurfaceIndex: c_int,
    pub genLod: c_int,
}

pub type surfaceInfo_v = Vec<surfaceInfo_t>;
pub type boneInfo_v = Vec<c_int>;

#[repr(C)]
pub struct mdxmSurfHierarchy_t {
    pub name: [c_char; 64],
    pub flags: c_int,
    pub parentIndex: c_int,
    pub numChildren: c_int,
    pub childIndexes: [c_int; 1],
}

#[repr(C)]
pub struct mdxmHeader_t {
    pub ident: c_int,
    pub version: c_int,
    pub numBones: c_int,
    pub numSurfaces: c_int,
    pub ofsSurfHierarchy: c_int,
}

#[repr(C)]
pub struct mdxmHierarchyOffsets_t {
    pub offsets: [c_int; 1],
}

#[repr(C)]
pub struct mdxmSurface_t {
    pub thisSurfaceIndex: c_int,
}

#[repr(C)]
pub struct model_t {
    pub mdxm: *mut mdxmHeader_t,
    pub mdxa: *mut mdxmHeader_t,
}

#[repr(C)]
pub struct CGhoul2Info {
    pub currentModel: *mut c_void,
    pub animModel: *mut c_void,
    pub mSlist: Vec<surfaceInfo_t>,
    pub mBlist: Vec<c_int>,
    pub mBltlist: Vec<c_int>,
    pub mSurfaceRoot: c_int,
    pub mMeshFrameNum: c_int,
    pub mModelBoltLink: c_int,
}

pub type CGhoul2Info_v = Vec<CGhoul2Info>;

#[repr(C)]
pub struct shader_t {
    pub name: [c_char; 64],
    // ... more fields
}

#[repr(C)]
pub struct surfaceType_t {
    pub name: [c_char; 64],
    pub shader: *mut shader_t,
    // ... more fields
}

#[repr(C)]
pub struct skin_t {
    pub name: [c_char; 64],
    pub numSurfaces: c_int,
    pub surfaces: *mut *mut surfaceType_t,
}

pub type qhandle_t = c_int;
pub type qboolean = c_int;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub const G2SURFACEFLAG_OFF: c_int = 1;
pub const G2SURFACEFLAG_NODESCENDANTS: c_int = 2;
pub const G2SURFACEFLAG_GENERATED: c_int = 4;

pub const TAG_GHOUL2: c_int = 0;

pub const MODEL_SHIFT: c_int = 0;
pub const MODEL_AND: c_int = 0xFFFF;
pub const BOLT_SHIFT: c_int = 16;
pub const BOLT_AND: c_int = 0xFFFF;

// ============================================================================
// EXTERNAL FUNCTION DECLARATIONS
// ============================================================================

extern "C" {
    fn G2_ConstructUsedBoneList(CBL: *mut CConstructBoneList);
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn R_GetSkinByHandle(hSkin: qhandle_t) -> *const skin_t;
    fn Z_Malloc(size: usize, tag: c_int, clear: qboolean) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn G2_DecideTraceLod(ghoul2: *const CGhoul2Info, useLod: c_int) -> c_int;
    fn G2API_RemoveGhoul2Model(g2i: *mut *mut CGhoul2Info_v, index: c_int);
    fn G2_RemoveRedundantBoneOverrides(boneList: *mut boneInfo_v, activeBones: *mut c_int);
    fn G2_RemoveRedundantBolts(
        boltList: *mut Vec<c_int>,
        surfaceList: *mut surfaceInfo_v,
        activeSurfaces: *mut c_int,
        activeBones: *mut c_int,
    );
}

// C++ overload variant: G2_FindSurface(void *mod, int surfaceNum, int unused) -> mdxmSurface_t*
// Declared separately since Rust doesn't support function overloading
extern "C" {
    fn G2_FindSurfaceByNum(mod_: *mut c_void, surfaceNum: c_int, unused: c_int) -> *mut mdxmSurface_t;
}

// ============================================================================
// C++ CLASS TRANSLATION
// ============================================================================

#[repr(C)]
pub struct CConstructBoneList {
    pub surfaceNum: c_int,
    pub boneUsedList: *mut c_int,
    pub rootSList: *mut surfaceInfo_v,
    pub currentModel: *mut model_t,
    pub boneList: *mut boneInfo_v,
}

impl CConstructBoneList {
    pub fn new(
        initsurfaceNum: c_int,
        initboneUsedList: *mut c_int,
        initrootSList: *mut surfaceInfo_v,
        initcurrentModel: *mut model_t,
        initboneList: *mut boneInfo_v,
    ) -> Self {
        CConstructBoneList {
            surfaceNum: initsurfaceNum,
            boneUsedList: initboneUsedList,
            rootSList: initrootSList,
            currentModel: initcurrentModel,
            boneList: initboneList,
        }
    }
}

// =====================================================================================================================
// Surface List handling routines - so entities can determine what surfaces attached to a model are operational or not.

// find a particular surface in the surface override list
pub fn G2_FindOverrideSurface(surfaceNum: c_int, surfaceList: &surfaceInfo_v) -> *mut surfaceInfo_t {
    // look through entire list
    for i in 0..surfaceList.len() {
        if surfaceList[i].surface == surfaceNum {
            return &surfaceList[i] as *const surfaceInfo_t as *mut surfaceInfo_t;
        }
    }
    // didn't find it.
    std::ptr::null_mut()
}

// given a surface name, lets see if it's legal in the model
pub fn G2_IsSurfaceLegal(
    mod_: *mut c_void,
    surfaceName: *const c_char,
    flags: *mut c_int,
) -> c_int {
    // damn include file dependancies
    let mod_m = mod_ as *mut model_t;
    unsafe {
        let mdxm = (*mod_m).mdxm;
        if mdxm.is_null() {
            return -1;
        }

        let mut surf = (mdxm as *mut u8)
            .add((*mdxm).ofsSurfHierarchy as usize)
            as *mut mdxmSurfHierarchy_t;

        for i in 0..(*mdxm).numSurfaces {
            if stricmp(surfaceName, &(*surf).name[0]) == 0 {
                *flags = (*surf).flags;
                return i;
            }
            // find the next surface
            surf = (surf as *mut u8).add(
                std::mem::size_of::<mdxmSurfHierarchy_t>()
                    + ((*surf).numChildren as usize) * std::mem::size_of::<c_int>(),
            ) as *mut mdxmSurfHierarchy_t;
        }
    }
    -1
}

/************************************************************************************************
 * G2_FindSurface
 *    find a surface in a ghoul2 surface override list based on it's name
 *
 * Input
 *    filename of model, surface list of model instance, name of surface, int to be filled in
 * with the index of this surface (defaults to NULL)
 *
 * Output
 *    pointer to surface if successful, false otherwise
 *
 ************************************************************************************************/
pub fn G2_FindSurface(
    ghlInfo: *mut CGhoul2Info,
    slist: &surfaceInfo_v,
    surfaceName: *const c_char,
    surfIndex: *mut c_int,
) -> *mut mdxmSurface_t {
    // find the model we want
    let mod_m = unsafe { (*ghlInfo).currentModel as *mut model_t };
    let surfIndexes = unsafe {
        ((*mod_m).mdxm as *mut u8).add(std::mem::size_of::<mdxmHeader_t>())
            as *mut mdxmHierarchyOffsets_t
    };
    let mut surfInfo: *mut mdxmSurfHierarchy_t;

    // did we find a ghoul 2 model or not?
    unsafe {
        if (*mod_m).mdxm.is_null() {
            assert!(false);
            if !surfIndex.is_null() {
                *surfIndex = -1;
            }
            return std::ptr::null_mut();
        }
    }

    // first find if we already have this surface in the list
    let mut i = (slist.len() as c_int) - 1;
    while i >= 0 {
        if slist[i as usize].surface != 10000 && slist[i as usize].surface != -1 {
            let surf = unsafe { G2_FindSurfaceByNum(mod_m as *mut c_void, slist[i as usize].surface, 0) };
            if !surf.is_null() {
                // back track and get the surfinfo struct for this surface
                unsafe {
                    surfInfo = (surfIndexes as *mut u8)
                        .add((*surfIndexes).offsets[(*surf).thisSurfaceIndex as usize] as usize)
                        as *mut mdxmSurfHierarchy_t;

                    // are these the droids we're looking for?
                    if stricmp(&(*surfInfo).name[0], surfaceName) == 0 {
                        // yup
                        if !surfIndex.is_null() {
                            *surfIndex = i;
                        }
                        return surf;
                    }
                }
            }
        }
        i -= 1;
    }
    // didn't find it
    if !surfIndex.is_null() {
        unsafe {
            *surfIndex = -1;
        }
    }
    std::ptr::null_mut()
}

// set a named surface offFlags - if it doesn't find a surface with this name in the list then it will add one.
pub fn G2_SetSurfaceOnOff(
    ghlInfo: *mut CGhoul2Info,
    slist: &mut surfaceInfo_v,
    surfaceName: *const c_char,
    offFlags: c_int,
) -> qboolean {
    let mut surfIndex: c_int = -1;
    let mut temp_slist_entry: surfaceInfo_t;
    let surf: *mut mdxmSurface_t;
    // find the model we want
    let mod_m = unsafe { (*ghlInfo).currentModel as *mut model_t };

    // did we find a ghoul 2 model or not?
    unsafe {
        if (*mod_m).mdxm.is_null() {
            assert!(false);
            return qfalse;
        }
    }

    // first find if we already have this surface in the list
    surf = G2_FindSurface(ghlInfo, slist, surfaceName, &mut surfIndex);
    if !surf.is_null() {
        // set descendants value

        // slist[surfIndex].offFlags = offFlags;
        // seems to me that we shouldn't overwrite the other flags.
        // the only bit we really care about in the incoming flags is the off bit
        slist[surfIndex as usize].offFlags &= !(G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
        slist[surfIndex as usize].offFlags |= offFlags & (G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
        return qtrue;
    } else {
        // ok, not in the list already - in that case, lets verify this surface exists in the model mesh
        let mut flags: c_int = 0;
        let surfaceNum = G2_IsSurfaceLegal(mod_m as *mut c_void, surfaceName, &mut flags);
        if surfaceNum != -1 {
            let mut newflags = flags;
            // the only bit we really care about in the incoming flags is the off bit
            newflags &= !(G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
            newflags |= offFlags & (G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);

            if newflags != flags {
                // insert here then because it changed, no need to add an override otherwise
                temp_slist_entry.offFlags = newflags;
                temp_slist_entry.surface = surfaceNum;

                slist.push(temp_slist_entry);
            }
            return qtrue;
        }
    }
    qfalse
}

pub fn G2_SetSurfaceOnOffFromSkin(ghlInfo: *mut CGhoul2Info, renderSkin: qhandle_t) {
    let skin = unsafe { R_GetSkinByHandle(renderSkin) };

    unsafe {
        (*ghlInfo).mSlist.clear(); //remove any overrides we had before.
        (*ghlInfo).mMeshFrameNum = 0;

        for j in 0..(*skin).numSurfaces {
            // the names have both been lowercased
            let surface_ptr = *(*skin).surfaces.add(j as usize);
            if surface_ptr.is_null() {
                continue;
            }

            if strcmp(&(*(*surface_ptr).shader).name[0], "*off\0".as_ptr() as *const c_char) == 0 {
                G2_SetSurfaceOnOff(ghlInfo, &mut (*ghlInfo).mSlist, &(*surface_ptr).name[0], G2SURFACEFLAG_OFF);
            } else {
                let mut flags: c_int = 0;
                let surfaceNum = G2_IsSurfaceLegal((*ghlInfo).currentModel, &(*surface_ptr).name[0], &mut flags);
                if (surfaceNum != -1) && (!(flags & G2SURFACEFLAG_OFF) != 0) {
                    //only turn on if it's not an "_off" surface
                    G2_SetSurfaceOnOff(ghlInfo, &mut (*ghlInfo).mSlist, &(*surface_ptr).name[0], 0);
                }
            }
        }
    }
}

// return a named surfaces off flags - should tell you if this surface is on or off.
pub fn G2_IsSurfaceOff(
    ghlInfo: *mut CGhoul2Info,
    slist: &surfaceInfo_v,
    surfaceName: *const c_char,
) -> c_int {
    let mod_m = unsafe { (*ghlInfo).currentModel as *mut model_t };
    let mut surfIndex: c_int = -1;
    let mut surf: *mut mdxmSurface_t = std::ptr::null_mut();

    // did we find a ghoul 2 model or not?
    unsafe {
        if (*mod_m).mdxm.is_null() {
            return 0;
        }
    }

    // first find if we already have this surface in the list
    surf = G2_FindSurface(ghlInfo, slist, surfaceName, &mut surfIndex);
    if !surf.is_null() {
        // set descendants value
        return slist[surfIndex as usize].offFlags;
    }
    // ok, we didn't find it in the surface list. Lets look at the original surface then.

    let mut surface: *mut mdxmSurfHierarchy_t = unsafe {
        let mdxm = (*mod_m).mdxm;
        (mdxm as *mut u8)
            .add((*mdxm).ofsSurfHierarchy as usize)
            as *mut mdxmSurfHierarchy_t
    };

    unsafe {
        let mdxm = (*mod_m).mdxm;
        for _i in 0..(*mdxm).numSurfaces {
            if stricmp(surfaceName, &(*surface).name[0]) == 0 {
                return (*surface).flags;
            }
            // find the next surface
            surface = (surface as *mut u8).add(
                std::mem::size_of::<mdxmSurfHierarchy_t>()
                    + ((*surface).numChildren as usize) * std::mem::size_of::<c_int>(),
            ) as *mut mdxmSurfHierarchy_t;
        }
    }

    assert!(false);
    0
}

pub fn G2_FindRecursiveSurface(
    currentModel: *mut model_t,
    surfaceNum: c_int,
    rootList: &mut surfaceInfo_v,
    activeSurfaces: *mut c_int,
) {
    let surface = unsafe { G2_FindSurfaceByNum(currentModel as *mut c_void, surfaceNum, 0) };

    if surface.is_null() {
        return;
    }

    let surfIndexes = unsafe {
        ((*currentModel).mdxm as *mut u8).add(std::mem::size_of::<mdxmHeader_t>())
            as *mut mdxmHierarchyOffsets_t
    };
    let surfInfo = unsafe {
        (surfIndexes as *mut u8)
            .add((*surfIndexes).offsets[(*surface).thisSurfaceIndex as usize] as usize)
            as *mut mdxmSurfHierarchy_t
    };

    // see if we have an override surface in the surface list
    let surfOverride = G2_FindOverrideSurface(surfaceNum, rootList);

    // really, we should use the default flags for this surface unless it's been overriden
    let mut offFlags = unsafe { (*surfInfo).flags };

    // set the off flags if we have some
    if !surfOverride.is_null() {
        offFlags = unsafe { (*surfOverride).offFlags };
    }

    // if this surface is not off, indicate as such in the active surface list
    if (offFlags & G2SURFACEFLAG_OFF) == 0 {
        unsafe {
            *activeSurfaces.add(surfaceNum as usize) = 1;
        }
    } else
    // if we are turning off all descendants, then stop this recursion now
    if (offFlags & G2SURFACEFLAG_NODESCENDANTS) != 0 {
        return;
    }

    // now recursively call for the children
    unsafe {
        for i in 0..(*surfInfo).numChildren {
            let childSurfaceNum = (*surfInfo).childIndexes[i as usize];
            G2_FindRecursiveSurface(currentModel, childSurfaceNum, rootList, activeSurfaces);
        }
    }
}

pub fn G2_RemoveRedundantGeneratedSurfaces(
    slist: &mut surfaceInfo_v,
    activeSurfaces: *mut c_int,
) {
    // walk the surface list, removing surface overrides or generated surfaces that are pointing at surfaces that aren't active anymore
    let mut i: usize = 0;
    while i < slist.len() {
        if slist[i].surface != -1 {
            // is this a generated surface?
            if slist[i].offFlags & G2SURFACEFLAG_GENERATED != 0 {
                // if it's not in the list, remove it
                unsafe {
                    if *activeSurfaces.add((slist[i].genPolySurfaceIndex & 0xffff) as usize) == 0 {
                        G2_RemoveSurface(slist, i as c_int);
                    }
                }
            }
            // no, so it does point back at a legal surface
            else {
                // if it's not in the list, remove it
                unsafe {
                    if *activeSurfaces.add(slist[i].surface as usize) == 0 {
                        G2_RemoveSurface(slist, i as c_int);
                    }
                }
            }
        }
        i += 1;
    }
}

pub fn G2_SetRootSurface(ghoul2: &mut CGhoul2Info_v, modelIndex: c_int, surfaceName: *const c_char) -> qboolean {
    let mut surf: c_int;
    let mut flags: c_int = 0;
    let mut activeSurfaces: *mut c_int;
    let mut activeBones: *mut c_int;

    assert!(
        !ghoul2[modelIndex as usize].currentModel.is_null()
            && !ghoul2[modelIndex as usize].animModel.is_null()
    );

    let mod_m = ghoul2[modelIndex as usize].currentModel as *mut model_t;
    let mod_a = ghoul2[modelIndex as usize].animModel as *mut model_t;

    // did we find a ghoul 2 model or not?
    unsafe {
        if (*mod_m).mdxm.is_null() {
            return qfalse;
        }
    }

    // first find if we already have this surface in the list
    surf = G2_IsSurfaceLegal(mod_m as *mut c_void, surfaceName, &mut flags);
    if surf != -1 {
        // first see if this ghoul2 model already has this as a root surface
        if ghoul2[modelIndex as usize].mSurfaceRoot == surf {
            return qtrue;
        }

        // set the root surface
        ghoul2[modelIndex as usize].mSurfaceRoot = surf;

        // ok, now the tricky bits.
        // firstly, generate a list of active / on surfaces below the root point

        // gimme some space to put this list into
        unsafe {
            activeSurfaces = Z_Malloc(
                ((*(*mod_m).mdxm).numSurfaces as usize) * 4,
                TAG_GHOUL2,
                qtrue,
            ) as *mut c_int;
            std::ptr::write_bytes(activeSurfaces, 0, ((*(*mod_m).mdxm).numSurfaces as usize) * 4);
            activeBones = Z_Malloc(
                ((*(*mod_a).mdxa).numBones as usize) * 4,
                TAG_GHOUL2,
                qtrue,
            ) as *mut c_int;
            std::ptr::write_bytes(activeBones, 0, ((*(*mod_a).mdxa).numBones as usize) * 4);
        }

        G2_FindRecursiveSurface(
            mod_m,
            surf,
            &mut ghoul2[modelIndex as usize].mSlist,
            activeSurfaces,
        );

        // now generate the used bone list
        let mut CBL = CConstructBoneList::new(
            ghoul2[modelIndex as usize].mSurfaceRoot,
            activeBones,
            &mut ghoul2[modelIndex as usize].mSlist,
            mod_m,
            &mut ghoul2[modelIndex as usize].mBlist,
        );

        unsafe {
            G2_ConstructUsedBoneList(&mut CBL);
        }

        // now remove all procedural or override surfaces that refer to surfaces that arent on this list
        G2_RemoveRedundantGeneratedSurfaces(
            &mut ghoul2[modelIndex as usize].mSlist,
            activeSurfaces,
        );

        // now remove all bones that are pointing at bones that aren't active
        unsafe {
            G2_RemoveRedundantBoneOverrides(&mut ghoul2[modelIndex as usize].mBlist, activeBones);
        }

        // then remove all bolts that point at surfaces or bones that *arent* active.
        unsafe {
            G2_RemoveRedundantBolts(
                &mut ghoul2[modelIndex as usize].mBltlist,
                &mut ghoul2[modelIndex as usize].mSlist,
                activeSurfaces,
                activeBones,
            );
        }

        // then remove all models on this ghoul2 instance that use those bolts that are being removed.
        for i in 0..ghoul2.len() {
            // are we even bolted to anything?
            if ghoul2[i].mModelBoltLink != -1 {
                let boltMod = (ghoul2[i].mModelBoltLink >> MODEL_SHIFT) & MODEL_AND;
                let boltNum = (ghoul2[i].mModelBoltLink >> BOLT_SHIFT) & BOLT_AND;
                // if either the bolt list is too small, or the bolt we are pointing at references nothing, remove this model
                if (ghoul2[boltMod as usize].mBltlist.len() as c_int <= boltNum)
                    || ((ghoul2[boltMod as usize].mBltlist[boltNum as usize] == -1)
                        && (ghoul2[boltMod as usize].mBltlist[boltNum as usize] == -1))
                {
                    let mut g2i = ghoul2 as *mut CGhoul2Info_v;
                    unsafe {
                        G2API_RemoveGhoul2Model(&mut g2i, i as c_int);
                    }
                }
            }
        }
        //No support for this, for now.

        // remember to free what we used
        unsafe {
            Z_Free(activeSurfaces as *mut c_void);
            Z_Free(activeBones as *mut c_void);
        }

        return qtrue;
    }
    /*
    //g2r	if (entstate->ghoul2)
        {
            CGhoul2Info_v &ghoul2 = *((CGhoul2Info_v *)entstate->ghoul2);
            model_t				*mod_m = R_GetModelByHandle(RE_RegisterModel(ghoul2[modelIndex].mFileName));
            model_t				*mod_a = R_GetModelByHandle(mod_m->mdxm->animIndex);
            int					surf;
            int					flags;
            int					*activeSurfaces, *activeBones;

            // did we find a ghoul 2 model or not?
            if (!mod_m->mdxm)
            {
                return qfalse;
            }

             // first find if we already have this surface in the list
            surf = G2_IsSurfaceLegal(mod_m, surfaceName, &flags);
            if (surf != -1)
            {
                // first see if this ghoul2 model already has this as a root surface
                if (ghoul2[modelIndex].mSurfaceRoot == surf)
                {
                    return qtrue;
                }

                // set the root surface
                ghoul2[modelIndex].mSurfaceRoot = surf;

                // ok, now the tricky bits.
                // firstly, generate a list of active / on surfaces below the root point

                // gimme some space to put this list into
                activeSurfaces = (int *)Z_Malloc(mod_m->mdxm->numSurfaces * 4, TAG_GHOUL2, qtrue);
                memset(activeSurfaces, 0, (mod_m->mdxm->numSurfaces * 4));
                activeBones = (int *)Z_Malloc(mod_a->mdxa->numBones * 4, TAG_GHOUL2, qtrue);
                memset(activeBones, 0, (mod_a->mdxa->numBones * 4));

                G2_FindRecursiveSurface(mod_m, surf, ghoul2[modelIndex].mSlist, activeSurfaces);

                // now generate the used bone list
                CConstructBoneList	CBL(ghoul2[modelIndex].mSurfaceRoot,
                                    activeBones,
                                    ghoul2[modelIndex].mSlist,
                                    mod_m,
                                    ghoul2[modelIndex].mBlist);

                G2_ConstructUsedBoneList(CBL);

                // now remove all procedural or override surfaces that refer to surfaces that arent on this list
                G2_RemoveRedundantGeneratedSurfaces(ghoul2[modelIndex].mSlist, activeSurfaces);

                // now remove all bones that are pointing at bones that aren't active
                G2_RemoveRedundantBoneOverrides(ghoul2[modelIndex].mBlist, activeBones);

                // then remove all bolts that point at surfaces or bones that *arent* active.
                G2_RemoveRedundantBolts(ghoul2[modelIndex].mBltlist, ghoul2[modelIndex].mSlist, activeSurfaces, activeBones);

                // then remove all models on this ghoul2 instance that use those bolts that are being removed.
                for (int i=0; i<ghoul2.size(); i++)
                {
                    // are we even bolted to anything?
                    if (ghoul2[i].mModelBoltLink != -1)
                    {
                        int	boltMod = (ghoul2[i].mModelBoltLink >> MODEL_SHIFT) & MODEL_AND;
                        int	boltNum = (ghoul2[i].mModelBoltLink >> BOLT_SHIFT) & BOLT_AND;
                        // if either the bolt list is too small, or the bolt we are pointing at references nothing, remove this model
                        if ((ghoul2[boltMod].mBltlist.size() <= boltNum) ||
                            ((ghoul2[boltMod].mBltlist[boltNum].boneNumber == -1) &&
                             (ghoul2[boltMod].mBltlist[boltNum].surfaceNumber == -1)))
                        {
                            G2API_RemoveGhoul2Model(entstate, i);
                        }
                    }
                }

                // remember to free what we used
                Z_Free(activeSurfaces);
                Z_Free(activeBones);

                return (qtrue);
            }
        }
        assert(0);*/
    qfalse
}

pub fn G2_AddSurface(
    ghoul2: *mut CGhoul2Info,
    surfaceNumber: c_int,
    polyNumber: c_int,
    BarycentricI: f32,
    BarycentricJ: f32,
    lod: c_int,
) -> c_int {
    let mut temp_slist_entry: surfaceInfo_t;

    // decide if LOD is legal
    let lod = unsafe { G2_DecideTraceLod(ghoul2 as *const CGhoul2Info, lod) };

    // first up, see if we have a free one already set up  - look only from the end of the constant surfaces onwards
    unsafe {
        for i in 0..(*ghoul2).mSlist.len() {
            // is the surface count -1? That would indicate it's free
            if (*ghoul2).mSlist[i].surface == -1 {
                (*ghoul2).mSlist[i].offFlags = G2SURFACEFLAG_GENERATED;
                (*ghoul2).mSlist[i].surface = 10000; // no model will ever have 10000 surfaces
                (*ghoul2).mSlist[i].genBarycentricI = BarycentricI;
                (*ghoul2).mSlist[i].genBarycentricJ = BarycentricJ;
                (*ghoul2).mSlist[i].genPolySurfaceIndex = ((polyNumber & 0xffff) << 16) | (surfaceNumber & 0xffff);
                (*ghoul2).mSlist[i].genLod = lod;
                return i as c_int;
            }
        }
    }

    // ok, didn't find one. Better create one

    temp_slist_entry.offFlags = G2SURFACEFLAG_GENERATED;
    temp_slist_entry.surface = 10000;
    temp_slist_entry.genBarycentricI = BarycentricI;
    temp_slist_entry.genBarycentricJ = BarycentricJ;
    temp_slist_entry.genPolySurfaceIndex = ((polyNumber & 0xffff) << 16) | (surfaceNumber & 0xffff);
    temp_slist_entry.genLod = lod;

    unsafe {
        (*ghoul2).mSlist.push(temp_slist_entry);

        return ((*ghoul2).mSlist.len() - 1) as c_int;
    }
}

pub fn G2_RemoveSurface(slist: &mut surfaceInfo_v, index: c_int) -> qboolean {
    // did we find it?
    if index != -1 {
        // set us to be the 'not active' state
        slist[index as usize].surface = -1;

        let mut newSize = slist.len();
        // now look through the list from the back and see if there is a block of -1's we can resize off the end of the list
        let mut i = (slist.len() as c_int) - 1;
        while i > -1 {
            if slist[i as usize].surface == -1 {
                newSize = i as usize;
            }
            // once we hit one that isn't a -1, we are done.
            else {
                break;
            }
            i -= 1;
        }
        // do we need to resize?
        if newSize != slist.len() {
            // yes, so lets do it
            slist.truncate(newSize);
        }

        return qtrue;
    }

    assert!(false);

    // no
    qfalse
}

pub fn G2_GetParentSurface(ghlInfo: *mut CGhoul2Info, index: c_int) -> c_int {
    let mod_m = unsafe { (*ghlInfo).currentModel as *mut model_t };
    let surfIndexes = unsafe {
        ((*mod_m).mdxm as *mut u8).add(std::mem::size_of::<mdxmHeader_t>())
            as *mut mdxmHierarchyOffsets_t
    };

    // walk each surface and see if this index is listed in it's children
    let surf = unsafe { G2_FindSurfaceByNum(mod_m as *mut c_void, index, 0) };
    if surf.is_null() {
        return -1;
    }

    unsafe {
        let surfInfo = (surfIndexes as *mut u8)
            .add((*surfIndexes).offsets[(*surf).thisSurfaceIndex as usize] as usize)
            as *mut mdxmSurfHierarchy_t;

        return (*surfInfo).parentIndex;
    }
}

pub fn G2_GetSurfaceIndex(ghlInfo: *mut CGhoul2Info, surfaceName: *const c_char) -> c_int {
    let mod_m = unsafe { (*ghlInfo).currentModel as *mut model_t };
    let mut flags: c_int = 0;

    G2_IsSurfaceLegal(mod_m as *mut c_void, surfaceName, &mut flags)
}

pub fn G2_IsSurfaceRendered(
    ghlInfo: *mut CGhoul2Info,
    surfaceName: *const c_char,
    slist: &surfaceInfo_v,
) -> c_int {
    let mut flags: c_int = 0; //,  surfFlags = 0;
    let mut surfIndex: c_int = 0;
    assert!(!unsafe { (*ghlInfo).currentModel }.is_null());
    unsafe {
        let mod_m = (*ghlInfo).currentModel as *mut model_t;
        assert!(!(*mod_m).mdxm.is_null());
        if (*mod_m).mdxm.is_null() {
            return -1;
        }
    }

    // now travel up the skeleton to see if any of it's ancestors have a 'no descendants' turned on

    // find the original surface in the surface list
    let mod_m = unsafe { (*ghlInfo).currentModel as *mut model_t };
    let mut surfNum = G2_IsSurfaceLegal(mod_m as *mut c_void, surfaceName, &mut flags);
    if surfNum != -1 {
        //must be legal
        let surfIndexes = unsafe {
            ((*mod_m).mdxm as *mut u8)
                .add(std::mem::size_of::<mdxmHeader_t>())
                as *const mdxmHierarchyOffsets_t
        };
        let mut surfInfo = unsafe {
            (surfIndexes as *const u8)
                .add((*surfIndexes).offsets[surfNum as usize] as usize)
                as *const mdxmSurfHierarchy_t
        };
        surfNum = unsafe { (*surfInfo).parentIndex };
        // walk the surface hierarchy up until we hit the root
        while surfNum != -1 {
            let mut parentFlags: c_int = 0;
            let parentSurfInfo: *const mdxmSurfHierarchy_t;

            parentSurfInfo = unsafe {
                (surfIndexes as *const u8)
                    .add((*surfIndexes).offsets[surfNum as usize] as usize)
                    as *const mdxmSurfHierarchy_t
            };

            // find the original surface in the surface list
            //G2 was bug, above comment was accurate, but we don't want the original flags, we want the parent flags
            unsafe {
                G2_IsSurfaceLegal(
                    mod_m as *mut c_void,
                    &(*parentSurfInfo).name[0],
                    &mut parentFlags,
                );
            }

            // now see if we already have overriden this surface in the slist
            let parentSurf = G2_FindSurface(ghlInfo, slist, unsafe { &(*parentSurfInfo).name[0] }, &mut surfIndex);
            if !parentSurf.is_null() {
                // set descendants value
                parentFlags = slist[surfIndex as usize].offFlags;
            }
            // now we have the parent flags, lets see if any have the 'no descendants' flag set
            if (parentFlags & G2SURFACEFLAG_NODESCENDANTS) != 0 {
                flags |= G2SURFACEFLAG_OFF;
                break;
            }
            // set up scan of next parent
            surfNum = unsafe { (*parentSurfInfo).parentIndex };
        }
    } else {
        return -1;
    }
    if flags == 0 {
        //it's not being overridden by a parent
        // now see if we already have overriden this surface in the slist
        let surf = G2_FindSurface(ghlInfo, slist, surfaceName, &mut surfIndex);
        if !surf.is_null() {
            // set descendants value
            flags = slist[surfIndex as usize].offFlags;
        }
        // ok, at this point in flags we have what this surface is set to, and the index of the surface itself
    }
    flags
}
