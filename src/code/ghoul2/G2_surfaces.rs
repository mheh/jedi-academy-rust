// leave this as first line for PCH reasons...
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};
use std::ptr;

// Import or define external types that this file depends on
// These would normally come from q_shared.h, tr_local.h, G2.h, etc.
// For now, we declare minimal type stubs for structural coherence

#[repr(C)]
pub struct mdxmHeader_t {
    // Placeholder - actual definition in G2.h
    pub ofsSurfHierarchy: c_int,
    pub numSurfaces: c_int,
}

#[repr(C)]
pub struct mdxmHierarchyOffsets_t {
    // Placeholder - actual definition in header
    pub offsets: [c_int; 1],  // Variable length in reality
}

#[repr(C)]
pub struct mdxmSurface_t {
    // Placeholder - actual definition in header
    pub thisSurfaceIndex: c_int,
}

#[repr(C)]
pub struct mdxmSurfHierarchy_t {
    pub name: [c_char; 64],
    pub flags: c_int,
    pub parentIndex: c_int,
    pub numChildren: c_int,
    pub childIndexes: [c_int; 1],  // Variable length in reality
}

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

#[repr(C)]
pub struct model_t {
    pub mdxm: *mut mdxmHeader_t,
    // Other fields omitted for structural coherence
}

pub type model_s = model_t;

#[repr(C)]
pub struct CGhoul2Info {
    pub mValid: c_int,
    pub aHeader: c_int,
    pub currentModel: *mut model_t,
    pub animModel: c_int,
    pub mSlist: surfaceInfo_v,
    pub mSurfaceRoot: c_int,
    // Other fields omitted
}

pub type CGhoul2Info_v = Vec<CGhoul2Info>;

pub const G2SURFACEFLAG_OFF: c_int = 1;
pub const G2SURFACEFLAG_NODESCENDANTS: c_int = 2;
pub const G2SURFACEFLAG_GENERATED: c_int = 4;

// ported macro: #define G2_MODEL_OK(g) ((g)&&(g)->mValid&&(g)->aHeader&&(g)->currentModel&&(g)->animModel)
#[inline]
fn G2_MODEL_OK(g: *const CGhoul2Info) -> bool {
    !g.is_null() && unsafe {
        (*g).mValid != 0 && (*g).aHeader != 0 && !(*g).currentModel.is_null() && (*g).animModel != 0
    }
}

struct CQuickOverride {
    mOverride: [c_int; 512],
    mAt: [c_int; 512],
    mCurrentTouch: c_int,
}

impl CQuickOverride {
    fn new() -> Self {
        let mut result = CQuickOverride {
            mOverride: [0; 512],
            mAt: [0; 512],
            mCurrentTouch: 1,
        };
        // mCurrentTouch = 1
        // mOverride array is already zero-initialized by [0; 512]
        result
    }

    fn Invalidate(&mut self) {
        self.mCurrentTouch += 1;
    }

    fn Set(&mut self, index: c_int, pos: c_int) {
        if index == 10000 {
            return;
        }
        assert!(index >= 0 && index < 512);
        self.mOverride[index as usize] = self.mCurrentTouch;
        self.mAt[index as usize] = pos;
    }

    fn Test(&self, index: c_int) -> c_int {
        assert!(index >= 0 && index < 512);
        if self.mOverride[index as usize] != self.mCurrentTouch {
            -1
        } else {
            self.mAt[index as usize]
        }
    }
}

static mut QuickOverride: CQuickOverride = CQuickOverride {
    mOverride: [0; 512],
    mAt: [0; 512],
    mCurrentTouch: 1,
};

// find a particular surface in the surface override list
pub fn G2_FindOverrideSurface(
    surfaceNum: c_int,
    surfaceList: &surfaceInfo_v,
) -> *const surfaceInfo_t {
    if surfaceNum < 0 {
        // starting a new lookup
        unsafe {
            QuickOverride.Invalidate();
        }
        let mut i: usize = 0;
        while i < surfaceList.len() {
            if surfaceList[i].surface >= 0 {
                unsafe {
                    QuickOverride.Set(surfaceList[i].surface, i as c_int);
                }
            }
            i += 1;
        }
        return ptr::null();
    }
    let idx = unsafe { QuickOverride.Test(surfaceNum) };
    if idx < 0 {
        let mut i: usize = 0;
        if surfaceNum == 10000 {
            while i < surfaceList.len() {
                if surfaceList[i].surface == surfaceNum {
                    return &surfaceList[i];
                }
                i += 1;
            }
        }
        // Only in debug mode:
        #[cfg(debug_assertions)]
        {
            // look through entire list
            while i < surfaceList.len() {
                if surfaceList[i].surface == surfaceNum {
                    break;
                }
                i += 1;
            }
            // didn't find it.
            assert_eq!(i, surfaceList.len(), "our quickoverride is not working right");
        }
        return ptr::null();
    }
    assert!(idx >= 0 && (idx as usize) < surfaceList.len());
    assert_eq!(surfaceList[idx as usize].surface, surfaceNum);
    &surfaceList[idx as usize]
}

// given a surface name, lets see if it's legal in the model
pub fn G2_IsSurfaceLegal(
    mod_m: *const model_s,
    surfaceName: *const c_char,
    flags: *mut c_int,
) -> c_int {
    assert!(!mod_m.is_null());
    unsafe {
        assert!(!(*mod_m).mdxm.is_null());
        // damn include file dependancies
        let mut surf = ((*(*mod_m).mdxm) as *const u8).add((*(*mod_m).mdxm).ofsSurfHierarchy as usize)
            as *mut mdxmSurfHierarchy_t;

        for i in 0..(*(*mod_m).mdxm).numSurfaces {
            if stricmp(surfaceName, (*surf).name.as_ptr()) == 0 {
                *flags = (*surf).flags;
                return i;
            }
            // find the next surface
            surf = (surf as *mut u8).add(
                std::mem::size_of::<c_int>()
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
    slist: &mut surfaceInfo_v,
    surfaceName: *const c_char,
    surfIndex: *mut c_int,
) -> *const mdxmSurface_t {
    assert!(G2_MODEL_OK(ghlInfo));

    unsafe {
        let surfIndexes = ((*(*(*ghlInfo).currentModel).mdxm) as *const u8)
            .add(std::mem::size_of::<mdxmHeader_t>())
            as *const mdxmHierarchyOffsets_t;

        // first find if we already have this surface in the list
        let mut i = slist.len();
        loop {
            if i == 0 {
                break;
            }
            i -= 1;
            if (slist[i].surface != 10000) && (slist[i].surface != -1) {
                let surf = G2_FindSurface(
                    (*ghlInfo).currentModel as *mut CGhoul2Info,
                    slist,
                    surfaceName,
                    ptr::null_mut(),
                ) as *mut mdxmSurface_t;
                // back track and get the surfinfo struct for this surface
                let surfInfo = ((surfIndexes as *const u8)
                    .add((*surfIndexes).offsets[(*surf).thisSurfaceIndex as usize] as usize))
                    as *const mdxmSurfHierarchy_t;

                // are these the droids we're looking for?
                if stricmp((*surfInfo).name.as_ptr(), surfaceName) == 0 {
                    // yup
                    if !surfIndex.is_null() {
                        *surfIndex = i as c_int;
                    }
                    return surf;
                }
            }
        }
    }
    // didn't find it
    if !surfIndex.is_null() {
        unsafe {
            *surfIndex = -1;
        }
    }
    ptr::null()
}

// set a named surface offFlags - if it doesn't find a surface with this name in the list then it will add one.
pub fn G2_SetSurfaceOnOff(
    ghlInfo: *mut CGhoul2Info,
    surfaceName: *const c_char,
    offFlags: c_int,
) -> bool {
    unsafe {
        let mut surfIndex: c_int = -1;
        let mut temp_slist_entry: surfaceInfo_t = std::mem::zeroed();

        // find the model we want
        // first find if we already have this surface in the list
        let surf = G2_FindSurface(ghlInfo, &mut (*ghlInfo).mSlist, surfaceName, &mut surfIndex);
        if !surf.is_null() {
            // set descendants value

            // slist[surfIndex].offFlags = offFlags;
            // seems to me that we shouldn't overwrite the other flags.
            // the only bit we really care about in the incoming flags is the off bit
            (*ghlInfo).mSlist[surfIndex as usize].offFlags &=
                !(G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
            (*ghlInfo).mSlist[surfIndex as usize].offFlags |=
                offFlags & (G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
            return true;
        } else {
            // ok, not in the list already - in that case, lets verify this surface exists in the model mesh
            let mut flags: c_int = 0;
            let surfaceNum =
                G2_IsSurfaceLegal((*ghlInfo).currentModel, surfaceName, &mut flags);
            if surfaceNum != -1 {
                let mut newflags = flags;
                // the only bit we really care about in the incoming flags is the off bit
                newflags &= !(G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);
                newflags |= offFlags & (G2SURFACEFLAG_OFF | G2SURFACEFLAG_NODESCENDANTS);

                if newflags != flags {
                    // insert here then because it changed, no need to add an override otherwise
                    temp_slist_entry.offFlags = newflags;
                    temp_slist_entry.surface = surfaceNum;

                    (*ghlInfo).mSlist.push_back(temp_slist_entry);
                }
                return true;
            }
        }
    }
    false
}

pub fn G2_FindRecursiveSurface(
    currentModel: *const model_t,
    surfaceNum: c_int,
    rootList: &mut surfaceInfo_v,
    activeSurfaces: *mut c_int,
) {
    assert!(!currentModel.is_null());
    unsafe {
        assert!(!(*currentModel).mdxm.is_null());
        let i: c_int;
        let surface = G2_FindSurface(
            currentModel as *mut CGhoul2Info,
            rootList,
            ptr::null(),
            ptr::null_mut(),
        ) as *const mdxmSurface_t;
        let surfIndexes = (((*currentModel).mdxm) as *const u8)
            .add(std::mem::size_of::<mdxmHeader_t>())
            as *const mdxmHierarchyOffsets_t;
        let surfInfo = ((surfIndexes as *const u8)
            .add((*surfIndexes).offsets[(*surface).thisSurfaceIndex as usize] as usize))
            as *const mdxmSurfHierarchy_t;

        // see if we have an override surface in the surface list
        let surfOverride = G2_FindOverrideSurface(surfaceNum, rootList);

        // really, we should use the default flags for this surface unless it's been overriden
        let mut offFlags = (*surfInfo).flags;

        // set the off flags if we have some
        if !surfOverride.is_null() {
            offFlags = (*surfOverride).offFlags;
        }

        // if this surface is not off, indicate as such in the active surface list
        if !(offFlags & G2SURFACEFLAG_OFF) != 0 {
            *activeSurfaces.add(surfaceNum as usize) = 1;
        } else if (offFlags & G2SURFACEFLAG_NODESCENDANTS) != 0 {
            // if we are turning off all descendants, then stop this recursion now
            return;
        }

        // now recursively call for the children
        for child_idx in 0..(*surfInfo).numChildren {
            let surfaceNum = (*surfInfo).childIndexes[child_idx as usize];
            G2_FindRecursiveSurface(currentModel, surfaceNum, rootList, activeSurfaces);
        }
    }
}

pub fn G2_SetRootSurface(
    ghoul2: &mut CGhoul2Info_v,
    modelIndex: c_int,
    surfaceName: *const c_char,
) -> bool {
    unsafe {
        let mut surf: c_int;
        let mut flags: c_int = 0;
        assert!(modelIndex >= 0 && (modelIndex as usize) < ghoul2.len());
        assert!(!ghoul2[modelIndex as usize].currentModel.is_null());
        assert!(!(*ghoul2[modelIndex as usize].currentModel)
            .mdxm
            .is_null());
        // first find if we already have this surface in the list
        surf = G2_IsSurfaceLegal(
            ghoul2[modelIndex as usize].currentModel,
            surfaceName,
            &mut flags,
        );
        if surf != -1 {
            ghoul2[modelIndex as usize].mSurfaceRoot = surf;
            return true;
        }
        assert!(false, "Surface not found");
        false
    }
}

extern "C" {
    pub fn G2_DecideTraceLod(ghoul2: &CGhoul2Info, useLod: c_int) -> c_int;
}

pub fn G2_AddSurface(
    ghoul2: *mut CGhoul2Info,
    surfaceNumber: c_int,
    polyNumber: c_int,
    BarycentricI: f32,
    BarycentricJ: f32,
    lod: c_int,
) -> c_int {
    unsafe {
        let lod = G2_DecideTraceLod(&*ghoul2, lod);

        // first up, see if we have a free one already set up  - look only from the end of the constant surfaces onwards
        let mut i: usize = 0;
        while i < (*ghoul2).mSlist.len() {
            // is the surface count -1? That would indicate it's free
            if (*ghoul2).mSlist[i].surface == -1 {
                break;
            }
            i += 1;
        }
        if i == (*ghoul2).mSlist.len() {
            (*ghoul2).mSlist.push_back(surfaceInfo_t {
                surface: 0,
                offFlags: 0,
                genBarycentricI: 0.0,
                genBarycentricJ: 0.0,
                genPolySurfaceIndex: 0,
                genLod: 0,
            });
        }
        (*ghoul2).mSlist[i].offFlags = G2SURFACEFLAG_GENERATED;
        (*ghoul2).mSlist[i].surface = 10000; // no model will ever have 10000 surfaces
        (*ghoul2).mSlist[i].genBarycentricI = BarycentricI;
        (*ghoul2).mSlist[i].genBarycentricJ = BarycentricJ;
        (*ghoul2).mSlist[i].genPolySurfaceIndex =
            ((polyNumber & 0xffff) << 16) | (surfaceNumber & 0xffff);
        (*ghoul2).mSlist[i].genLod = lod;
        i as c_int
    }
}

pub fn G2_RemoveSurface(slist: &mut surfaceInfo_v, index: c_int) -> bool {
    if index != -1 {
        slist[index as usize].surface = -1;
        return true;
    }
    assert!(false);
    false
}

pub fn G2_GetParentSurface(ghlInfo: *mut CGhoul2Info, index: c_int) -> c_int {
    unsafe {
        assert!(!(*ghlInfo).currentModel.is_null());
        assert!(!(*(*ghlInfo).currentModel).mdxm.is_null());
        let surfIndexes = (((*(*ghlInfo).currentModel).mdxm) as *const u8)
            .add(std::mem::size_of::<mdxmHeader_t>())
            as *const mdxmHierarchyOffsets_t;

        // walk each surface and see if this index is listed in it's children
        let surf = G2_FindSurface(
            (*ghlInfo).currentModel as *mut CGhoul2Info,
            &mut (*ghlInfo).mSlist,
            ptr::null(),
            ptr::null_mut(),
        ) as *const mdxmSurface_t;
        let surfInfo = ((surfIndexes as *const u8)
            .add((*surfIndexes).offsets[(*surf).thisSurfaceIndex as usize] as usize))
            as *const mdxmSurfHierarchy_t;

        (*surfInfo).parentIndex
    }
}

pub fn G2_GetSurfaceIndex(ghlInfo: *mut CGhoul2Info, surfaceName: *const c_char) -> c_int {
    unsafe {
        let mut flags: c_int = 0;
        assert!(!(*ghlInfo).currentModel.is_null());
        G2_IsSurfaceLegal((*ghlInfo).currentModel, surfaceName, &mut flags)
    }
}

pub fn G2_IsSurfaceRendered(
    ghlInfo: *mut CGhoul2Info,
    surfaceName: *const c_char,
    slist: &mut surfaceInfo_v,
) -> c_int {
    unsafe {
        let mut flags: c_int = 0;
        let mut surfIndex: c_int = 0;
        assert!(!(*ghlInfo).currentModel.is_null());
        assert!(!(*(*ghlInfo).currentModel).mdxm.is_null());
        if (*(*ghlInfo).currentModel).mdxm.is_null() {
            return -1;
        }

        // now travel up the skeleton to see if any of it's ancestors have a 'no descendants' turned on

        // find the original surface in the surface list
        let mut surfNum = G2_IsSurfaceLegal((*ghlInfo).currentModel, surfaceName, &mut flags);
        if surfNum != -1 {
            //must be legal
            let surfIndexes = (((*(*ghlInfo).currentModel).mdxm) as *const u8)
                .add(std::mem::size_of::<mdxmHeader_t>())
                as *const mdxmHierarchyOffsets_t;
            let mut surfInfo = ((surfIndexes as *const u8)
                .add((*surfIndexes).offsets[surfNum as usize] as usize))
                as *const mdxmSurfHierarchy_t;
            surfNum = (*surfInfo).parentIndex;
            // walk the surface hierarchy up until we hit the root
            while surfNum != -1 {
                let parentSurf: *const mdxmSurface_t;
                let mut parentFlags: c_int;
                let parentSurfInfo: *const mdxmSurfHierarchy_t;

                parentSurfInfo = ((surfIndexes as *const u8)
                    .add((*surfIndexes).offsets[surfNum as usize] as usize))
                    as *const mdxmSurfHierarchy_t;

                // find the original surface in the surface list
                // G2 was bug, above comment was accurate, but we don't want the original flags, we want the parent flags
                G2_IsSurfaceLegal(
                    (*ghlInfo).currentModel,
                    (*parentSurfInfo).name.as_ptr(),
                    &mut parentFlags,
                );

                // now see if we already have overriden this surface in the slist
                parentSurf = G2_FindSurface(ghlInfo, slist, (*parentSurfInfo).name.as_ptr(), &mut surfIndex);
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
                surfNum = (*parentSurfInfo).parentIndex;
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
}

// Local stub for stricmp since it's a C standard library function
#[inline]
fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let c1 = (*p1) as u8;
            let c2 = (*p2) as u8;
            let c1_lower = if c1 >= b'A' && c1 <= b'Z' {
                c1 + 32
            } else {
                c1
            };
            let c2_lower = if c2 >= b'A' && c2 <= b'Z' {
                c2 + 32
            } else {
                c2
            };
            if c1_lower != c2_lower {
                return (c1_lower as c_int) - (c2_lower as c_int);
            }
            if c1 == 0 {
                return 0;
            }
            p1 = p1.add(1);
            p2 = p2.add(1);
        }
    }
}
