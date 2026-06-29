// port of oracle/code/RMG/RM_Terrain.cpp

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use crate::code::server::exe_headers_h::*;
use crate::code::client::client_h::*;
use crate::code::qcommon::cm_local_h::*;
use crate::code::qcommon::cm_landscape_h::*;
use crate::code::renderer::tr_types_h::*;
use crate::code::RMG::rm_headers_h::*;
use crate::code::RMG::genericparser2_h::*;
use crate::code::RMG::RM_Terrain_h::*;

use core::ffi::{c_char, c_int, c_void};

//#include "../qcommon/q_imath.h"

// #pragma optimize("", off)

extern "C" {
    fn R_LoadDataImage(name: *const c_char, pic: *mut *mut u8, width: *mut c_int, height: *mut c_int);
    fn R_InvertImage(data: *mut u8, width: c_int, height: c_int, depth: c_int);
    fn R_Resample(
        source: *mut u8,
        swidth: c_int,
        sheight: c_int,
        dest: *mut u8,
        dwidth: c_int,
        dheight: c_int,
        components: c_int,
    );
    fn RE_GetModelBounds(refEnt: *mut refEntity_t, bounds1: *mut vec3_t, bounds2: *mut vec3_t);
}

static mut rm_landscape: *mut CRMLandScape = core::ptr::null_mut();
static mut origin_land: *mut CCMLandScape = core::ptr::null_mut();

impl CRMLandScape {
    // CRMLandScape::CRMLandScape(void)
    pub unsafe fn new_landscape() -> Self {
        let mut s: Self = core::mem::zeroed();
        s.common = core::ptr::null_mut();
        s.mDensityMap = core::ptr::null_mut();
        s
    }

    pub unsafe fn AddModel(&mut self, height: c_int, mut maxheight: c_int, hd: *const CRandomModel) {
        let mut i: c_int;

        if maxheight > HEIGHT_RESOLUTION {
            maxheight = HEIGHT_RESOLUTION;
        }

        i = height;
        while (*hd).GetModel() && (i < maxheight) {
            self.mHeightDetails[i as usize].AddModel(hd);
            i += 1;
        }
    }

    pub unsafe fn LoadMiscentDef(&mut self, td: *const c_char) {
        let mut miscentDef: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        let mut parse: CGenericParser2 = core::mem::zeroed();
        let mut basegroup: *mut CGPGroup;
        let mut classes: *mut CGPGroup;
        let mut items: *mut CGPGroup;
        let mut model: *mut CGPGroup;
        let mut pair: *mut CGPValue;

        Com_sprintf(
            miscentDef.as_mut_ptr(),
            MAX_QPATH,
            b"ext_data/RMG/%s.miscents\0".as_ptr() as *const c_char,
            Info_ValueForKey(td, b"miscentDef\0".as_ptr() as *const c_char),
        );
        Com_DPrintf(
            b"CG_Terrain: Loading and parsing miscentDef %s.....\n\0".as_ptr() as *const c_char,
            Info_ValueForKey(td, b"miscentDef\0".as_ptr() as *const c_char),
        );

        if Com_ParseTextFile(miscentDef.as_ptr(), &mut parse) == 0 {
            Com_sprintf(
                miscentDef.as_mut_ptr(),
                MAX_QPATH,
                b"ext_data/arioche/%s.miscents\0".as_ptr() as *const c_char,
                Info_ValueForKey(td, b"miscentDef\0".as_ptr() as *const c_char),
            );
            if Com_ParseTextFile(miscentDef.as_ptr(), &mut parse) == 0 {
                Com_Printf(
                    b"Could not open %s\n\0".as_ptr() as *const c_char,
                    miscentDef.as_ptr(),
                );
                return;
            }
        }
        // The whole file....
        basegroup = parse.GetBaseParseGroup();

        // The root { } struct
        classes = (*basegroup).GetSubGroups();
        while !classes.is_null() {
            items = (*classes).GetSubGroups();
            while !items.is_null() {
                if stricmp((*items).GetName(), b"miscent\0".as_ptr() as *const c_char) == 0 {
                    let height: c_int;
                    let maxheight: c_int;

                    // Height must exist - the rest are optional
                    height = atol((*items).FindPairValue(
                        b"height\0".as_ptr() as *const c_char,
                        b"0\0".as_ptr() as *const c_char,
                    ));
                    maxheight = atol((*items).FindPairValue(
                        b"maxheight\0".as_ptr() as *const c_char,
                        b"255\0".as_ptr() as *const c_char,
                    ));

                    model = (*items).GetSubGroups();
                    while !model.is_null() {
                        if stricmp((*model).GetName(), b"model\0".as_ptr() as *const c_char) == 0 {
                            let mut hd: CRandomModel = core::mem::zeroed();

                            // Set defaults
                            hd.SetModel(b"\0".as_ptr() as *const c_char);
                            hd.SetFrequency(1.0f32);
                            hd.SetMinScale(1.0f32);
                            hd.SetMaxScale(1.0f32);

                            pair = (*model).GetPairs();
                            while !pair.is_null() {
                                if stricmp((*pair).GetName(), b"name\0".as_ptr() as *const c_char) == 0 {
                                    hd.SetModel((*pair).GetTopValue());
                                } else if stricmp(
                                    (*pair).GetName(),
                                    b"frequency\0".as_ptr() as *const c_char,
                                ) == 0
                                {
                                    hd.SetFrequency(atof((*pair).GetTopValue()) as f32);
                                } else if stricmp(
                                    (*pair).GetName(),
                                    b"minscale\0".as_ptr() as *const c_char,
                                ) == 0
                                {
                                    hd.SetMinScale(atof((*pair).GetTopValue()) as f32);
                                } else if stricmp(
                                    (*pair).GetName(),
                                    b"maxscale\0".as_ptr() as *const c_char,
                                ) == 0
                                {
                                    hd.SetMaxScale(atof((*pair).GetTopValue()) as f32);
                                }
                                pair = (*pair).GetNext() as *mut CGPValue;
                            }
                            self.AddModel(height, maxheight, &hd);
                        }
                        model = (*model).GetNext() as *mut CGPGroup;
                    }
                }
                items = (*items).GetNext() as *mut CGPGroup;
            }
            classes = (*classes).GetNext() as *mut CGPGroup;
        }
        Com_ParseTextFileDestroy(&mut parse);
    }

    pub unsafe fn CreateRandomDensityMap(
        &mut self,
        density: *mut u8,
        width: c_int,
        height: c_int,
        seed: c_int,
    ) {
        //	int			i, border, inc;
        let mut x: c_int;
        let mut y: c_int;
        let mut count: c_int;
        //	byte		*work, *work2;
        let mut area: *mut CArea;
        let mut derxelSize: vec3_t = core::mem::zeroed();
        let mut pos: vec3_t = core::mem::zeroed();
        let mut dmappos: ivec3_t = core::mem::zeroed();
        let hm_map: *mut u8 = (*self.common).GetHeightMap();
        let hm_width: c_int = (*self.common).GetRealWidth();
        let hm_height: c_int = (*self.common).GetRealHeight();
        let mut xpos: c_int;
        let mut ypos: c_int;
        let mut dx: c_int;
        let mut dy: c_int;
        let mut densityPos: *mut u8 = density;
        let mut foundUneven: bool;

        // Init to linear spread
        memset(density as *mut c_void, 0, (width * height) as usize);

    /*	// Make more prevalent towards the edges
        border = Com_Clamp(6, 12, (width + height) >> 4);

        for(i = 0; i < border; i++)
        {
            inc = (border - i + 1) * 9;

            // Top line
            work = density + i + (i * width);
            for(x = i; x < width - i; x++, work++)
            {
                *work += (byte)common->irand(inc >> 1, inc);
            }

            // Left and right edges
            work = density + i + ((i + 1) * width);
            work2 = density + (width - i) + ((i + 1) * width);
            for(y = i + 1; y < height - i - 2; y++, work += width, work2 += width)
            {
                *work += (byte)common->irand(inc >> 1, inc);
                *work2 += (byte)common->irand(inc >> 1, inc);
            }

            // Bottom line
            work = density + i + ((height - i - 1) * width);
            for(x = i; x < width - i; x++, work++)
            {
                *work += (byte)common->irand(inc >> 1, inc);
            }
        }
    */
        count = 0;

        y = 0;
        while y < height {
            x = 0;
            while x < width {
                xpos = x * hm_width / width;
                ypos = y * hm_height / height;
                ypos = hm_height - ypos - 1;

                if *hm_map.add((ypos * hm_width + xpos) as usize) < 150 {
                    x += 1;
                    densityPos = densityPos.add(1);
                    continue;
                }

                foundUneven = false;
                dx = -4;
                while dx <= 4 && !foundUneven {
                    dy = -4;
                    while dy <= 4 && !foundUneven {
                        if dx == 0 && dy == 0 {
                            dy += 1;
                            continue;
                        }
                        if (xpos + dx) >= 0
                            && (xpos + dx) < hm_width
                            && (ypos + dy) >= 0
                            && (ypos + dy) < hm_height
                        {
                            if *hm_map.add(((ypos + dy) * hm_width + (xpos + dx)) as usize) < 190 {
                                *densityPos = 205;
                                count += 1;
                                foundUneven = true;
                            }
                        }
                        dy += 1;
                    }
                    dx += 1;
                }

                x += 1;
                densityPos = densityPos.add(1);
            }
            y += 1;
        }

    /*	FILE	*FH;

        FH = fopen("c:\o.raw", "wb");
        fwrite(hm_map, 1, common->GetRealWidth() * common->GetRealHeight(), FH);
        fclose(FH);

        FH = fopen("c:\d.raw", "wb");
        fwrite(density, 1, width*height, FH);
        fclose(FH);
    */
        // Reduce severely for any settlements/buildings/objectives
        VectorScale((*self.common).GetSize(), 1.0f32 / width as f32, &mut derxelSize);

        origin_land = self.common;
        area = (*self.common).GetFirstArea();
        while !area.is_null() {
            // Skip group types since they encompass to much open area
            if (*area).GetType() == AT_GROUP {
                area = (*self.common).GetNextArea();
                continue;
            }

            VectorSubtract((*area).GetPosition(), (*self.common).GetMins(), &mut pos);
            VectorInverseScaleVector(pos, derxelSize, &mut dmappos);
            // Damn upside down gensurf
            dmappos[1] = height - dmappos[1];

            count = ((*area).GetRadius() / derxelSize[1]).ceil() as c_int;

            while count > 0 {
                CM_CircularIterate(
                    density,
                    width,
                    height,
                    dmappos[0],
                    dmappos[1],
                    0,
                    count,
                    core::ptr::null_mut(),
                    CG_Decrease,
                );
                count -= 1;
            }
            area = (*self.common).GetNextArea();
        }
    }

    pub unsafe fn LoadDensityMap(&mut self, td: *const c_char) {
        let mut densityMap: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
        let mut imageData: *mut u8;
        let mut iWidth: c_int;
        let mut iHeight: c_int;
        let mut ptr: *mut c_char = core::ptr::null_mut();

        // Fill in with default values
        self.mDensityMap =
            Z_Malloc((*self.common).GetBlockCount(), TAG_R_TERRAIN, qfalse) as *mut u8;
        memset(
            self.mDensityMap as *mut c_void,
            128,
            (*self.common).GetBlockCount() as usize,
        );

        // Load in density map (if any)
        Com_sprintf(
            densityMap.as_mut_ptr(),
            MAX_QPATH,
            b"%s\0".as_ptr() as *const c_char,
            Info_ValueForKey(td, b"densityMap\0".as_ptr() as *const c_char),
        );
        if strlen(densityMap.as_ptr()) != 0 {
            Com_DPrintf(
                b"CG_Terrain: Loading density map %s.....\n\0".as_ptr() as *const c_char,
                densityMap.as_ptr(),
            );
            imageData = core::ptr::null_mut();
            iWidth = 0;
            iHeight = 0;
            R_LoadDataImage(densityMap.as_ptr(), &mut imageData, &mut iWidth, &mut iHeight);
            if !imageData.is_null() {
                if !strstr(densityMap.as_ptr(), b"density_\0".as_ptr() as *const c_char).is_null() {
                    let seed = strtoul(
                        Info_ValueForKey(td, b"seed\0".as_ptr() as *const c_char),
                        &mut ptr,
                        10,
                    ) as c_int;
                    self.CreateRandomDensityMap(imageData, iWidth, iHeight, seed);
                }
                R_Resample(
                    imageData,
                    iWidth,
                    iHeight,
                    self.mDensityMap,
                    (*self.common).GetBlockWidth(),
                    (*self.common).GetBlockHeight(),
                    1,
                );
                R_InvertImage(
                    self.mDensityMap,
                    (*self.common).GetBlockWidth(),
                    (*self.common).GetBlockHeight(),
                    1,
                );
                Z_Free(imageData as *mut c_void);
            }
        }
    }
}

impl CCGHeightDetails {
    pub unsafe fn AddModel(&mut self, hd: *const CRandomModel) {
        if (self.mNumModels as usize) < MAX_RANDOM_MODELS {
            self.mTotalFrequency += (*hd).GetFrequency();
            self.mModels[self.mNumModels as usize] = *hd;
            self.mNumModels += 1;
        }
    }
}

// GetRandomModel is NOT in CCGHeightDetails impl here — it is a method defined
// in this .cpp, so it belongs in this impl block:
impl CCGHeightDetails {
    pub unsafe fn GetRandomModel(&self, land: *mut CCMLandScape) -> *mut CRandomModel {
        let mut seek: c_int;
        let mut i: c_int;

        seek = (*land).irand(0, self.mTotalFrequency as c_int);
        i = 0;
        while i < self.mNumModels {
            seek -= self.mModels[i as usize].GetFrequency() as c_int;
            if seek <= 0 {
                return (self.mModels.as_ptr() as *mut CRandomModel).add(i as usize);
            }
            i += 1;
        }
        debug_assert!(false);
        core::ptr::null_mut()
    }
}

#[cfg(not(feature = "dedicated"))]
impl CRMLandScape {
    pub unsafe fn Sprinkle(&mut self, patch: *mut CCMPatch, hd: *mut CCGHeightDetails, level: c_int) {
        let mut i: c_int;
        let mut count: c_int;
        let px: c_int;
        let py: c_int;
        let density: f32;
        let mut origin: vec3_t = core::mem::zeroed();
        let mut scale: vec3_t = core::mem::zeroed();
        let mut angles: vec3_t = core::mem::zeroed();
        let mut bounds: [vec3_t; 2] = core::mem::zeroed();
        let mut refEnt: refEntity_t = core::mem::zeroed();
        let mut rm: *mut CRandomModel = core::ptr::null_mut();
        let mut area: CArea = core::mem::zeroed();
        //	int				areaTypes[] = { AT_BSP, AT_OBJECTIVE };
        //	TCGMiscEnt		*data = (TCGMiscEnt *)cl.mSharedMemory;
        //	TCGTrace		*td = (TCGTrace *)cl.mSharedMemory;

        //	memset(&refEnt, 0, sizeof(refEntity_t));

        px = (*patch).GetHeightMapX() / (*self.common).GetTerxels();
        py = (*patch).GetHeightMapY() / (*self.common).GetTerxels();
        // Get a number -5.3f to 5.3f
        density =
            (*self.mDensityMap.add((px + (*self.common).GetBlockWidth() * py) as usize) as f32
                - 128.0f32)
                / 24.0f32;
        // ..and multiply that into the count
        count = Round(
            (*self.common).GetPatchScalarSize()
                * (*hd).GetAverageFrequency() as f32
                * (2.0f32).powf(density)
                * 0.001f32,
        );

        i = 0;
        while i < count {
            if (*self.common).irand(0, 10) == 0 {
                let mut temp: vec3_t = core::mem::zeroed();
                let mut tr: trace_t = core::mem::zeroed();
                let average: f32;

                rm = (*hd).GetRandomModel(self.common);

                refEnt.hModel = ((*core::ptr::addr_of!(re)).RegisterModel)((*rm).GetModelName());
                refEnt.frame = 0;
                RE_GetModelBounds(&mut refEnt, &mut bounds[0], &mut bounds[1]);

                // Calculate the scale using some magic to help ensure that the
                // scales are never too different from eachother.  Otherwise you
                // could get an entity that is really small on one axis but huge
                // on another.
                temp[0] = (*self.common).flrand((*rm).GetMinScale(), (*rm).GetMaxScale());
                temp[1] = (*self.common).flrand((*rm).GetMinScale(), (*rm).GetMaxScale());
                temp[2] = (*self.common).flrand((*rm).GetMinScale(), (*rm).GetMaxScale());

                // Average of the three random numbers and divide that by two
                average = ((temp[0] + temp[1] + temp[2]) / 3.0f32) / 2.0f32;

                // Add in half of the other two numbers and then subtract half the average to prevent.
                // any number from going beyond the range. If all three numbers were the same then
                // they would remain unchanged after this calculation.
                scale[0] = temp[0] + (temp[1] + temp[2]) / 2.0f32 - average;
                scale[1] = temp[1] + (temp[0] + temp[2]) / 2.0f32 - average;
                scale[2] = temp[2] + (temp[0] + temp[1]) / 2.0f32 - average;

                angles[0] = 0.0f32;
                angles[1] = (*self.common).flrand(-(M_PI as f32), M_PI as f32);
                angles[2] = 0.0f32;

                VectorCopy((*patch).GetMins(), &mut origin);
                origin[0] += (*self.common).flrand(0.0f32, (*self.common).GetPatchWidth());
                origin[1] += (*self.common).flrand(0.0f32, (*self.common).GetPatchHeight());
                // Get above world height
                let slope: f32 = (*self.common).GetWorldHeight(&mut origin, bounds.as_mut_ptr(), qtrue);

                if slope > 1.33f32 {
                    // spot has too steep of a slope
                    i += 1;
                    continue;
                }
                if origin[2] < (*self.common).GetWaterHeight() {
                    i += 1;
                    continue;
                }
                // very that we aren't dropped too low
                if origin[2] < (*self.common).CalcWorldHeight(level) {
                    i += 1;
                    continue;
                }

                // Hack-ariffic, don't allow them to drop below the big player clip brush.
                if origin[2] < 1280.0f32 {
                    i += 1;
                    continue;
                }
                // FIXME: shouldn't be using a hard-coded 1280 number, only allow to spawn if inside player clip brush?
        //		if( !(CONTENTS_PLAYERCLIP & VM_Call( cgvm, CG_POINT_CONTENTS )) )
        //		{
        //			continue;
        //		}
                // Simple radius check for buildings
    /*			area.Init(origin, VectorLength(bounds[0]));
                if(common->AreaCollision(&area, areaTypes, sizeof(areaTypes) / sizeof(int)))
                {
                    continue;
                }*/
                // Make sure there is no architecture around - doesn't work for ents though =(

                /*
                memset(td, sizeof(*td), 0);
                VectorCopy(origin, td->mStart);
                VectorCopy(bounds[0], td->mMins);
                VectorCopy(bounds[1], td->mMaxs);
                VectorCopy(origin, td->mEnd);
                td->mSkipNumber = -1;
                td->mMask = MASK_PLAYERSOLID;
                */
                SV_Trace(&mut tr, &mut origin, &mut bounds[0], &mut bounds[1], &mut origin, -1, CONTENTS_SOLID | CONTENTS_PLAYERCLIP | CONTENTS_BODY | CONTENTS_TERRAIN);

                /*
                VM_Call( cgvm, CG_TRACE );
                if(td->mResult.surfaceFlags & SURF_NOMISCENTS)
                {
                    continue;
                }
                if(td->mResult.startsolid)
                {
    //				continue;
                }
                */
                if tr.surfaceFlags & SURF_NOMISCENTS != 0 {
                    i += 1;
                    continue;
                }
                if tr.startsolid != 0 {
    //				continue;
                }

                // Get minimum height of area
                (*self.common).GetWorldHeight(&mut origin, bounds.as_mut_ptr(), qfalse);
                // Account for relative origin
                origin[2] -= bounds[0][2] * scale[2];
                origin[2] -= (*self.common).flrand(2.0f32, (bounds[1][2] - bounds[0][2]) / 4.0f32);

                //rwwFIXMEFIXME: Do this properly
                // Spawn the client model
                /*
                strcpy(data->mModel, rm->GetModelName());
                VectorCopy(origin, data->mOrigin);
                VectorCopy(angles, data->mAngles);
                VectorCopy(scale, data->mScale);
                VM_Call( cgvm, CG_MISC_ENT);
                */

                self.mModelCount += 1;
            }
            i += 1;
        }
    }
}

impl CRMLandScape {
    pub unsafe fn SpawnPatchModels(&mut self, patch: *mut CCMPatch) {
        //	Rand_Init(10);
        #[cfg(not(feature = "dedicated"))]
        {
            let mut i: c_int = 0;
            let mut hd: *mut CCGHeightDetails;
            while i < 4 {
                hd = self.mHeightDetails.as_mut_ptr().add((*patch).GetHeight(i) as usize);
                if (*hd).GetNumModels() != 0 {
                    self.Sprinkle(patch, hd, (*patch).GetHeight(i));
                }
                i += 1;
            }
        }
    }
}

impl Drop for CRMLandScape {
    fn drop(&mut self) {
        unsafe {
            if !self.mDensityMap.is_null() {
                Z_Free(self.mDensityMap as *mut c_void);
                self.mDensityMap = core::ptr::null_mut();
            }
        }
    }
}

pub unsafe extern "C" fn CG_Decrease(work: *mut u8, lerp: f32, info: *mut c_int) {
    let val: c_int;

    val = *work as c_int - (*origin_land).irand(2, 5);
    *work = Com_Clamp(1, 255, val) as u8;
}

pub unsafe extern "C" fn SpawnPatchModelsWrapper(patch: *mut CCMPatch, userdata: *mut c_void) {
    let landscape: *mut CRMLandScape = userdata as *mut CRMLandScape;
    (*landscape).SpawnPatchModels(patch);
}

pub unsafe fn RM_CreateRandomModels(terrainId: c_int, terrainInfo: *const c_char) {
    let landscape: *mut CRMLandScape;

    landscape = Box::into_raw(Box::new(CRMLandScape::new_landscape()));
    rm_landscape = landscape;
    (*landscape).SetCommon((*core::ptr::addr_of!(cmg)).landScape);

    Com_DPrintf(b"CG_Terrain: Creating random models.....\n\0".as_ptr() as *const c_char);
    (*landscape).LoadMiscentDef(terrainInfo);
    (*landscape).LoadDensityMap(terrainInfo);
    (*landscape).ClearModelCount();
    CM_TerrainPatchIterate(
        (*landscape).GetCommon(),
        SpawnPatchModelsWrapper,
        landscape as *mut c_void,
    );

    Com_DPrintf(
        b".....%d random client models spawned\n\0".as_ptr() as *const c_char,
        (*landscape).GetModelCount(),
    );
}

pub unsafe fn RM_InitTerrain() {
    rm_landscape = core::ptr::null_mut();
}

pub unsafe fn RM_ShutdownTerrain() {
    let landscape: *mut CRMLandScape;

    landscape = rm_landscape;
    if !landscape.is_null() {
    //			CM_ShutdownTerrain(i);
        drop(Box::from_raw(landscape));
        rm_landscape = core::ptr::null_mut();
    }
}

// end

// #pragma optimize("", on)
