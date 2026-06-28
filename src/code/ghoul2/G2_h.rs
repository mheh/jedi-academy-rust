#![allow(non_snake_case)]

use core::ffi::c_void;

// Forward declarations of external types
// These types are defined in other modules and are imported as needed
pub struct CMiniHeap;

pub struct CRagDollUpdateParams;

pub struct model_s;

// Stubs for types used in function signatures - definitions in other modules
pub struct CGhoul2Info;
pub struct CGhoul2Info_v;
pub struct boneInfo_v;
pub struct surfaceInfo_v;
pub struct surfaceInfo_t;
pub struct boltInfo_v;
pub struct CCollisionRecord;
pub struct mdxaBone_t;
pub struct SSkinGoreData;
pub struct boneInfo_t;

// Type aliases for C integer types
pub type qboolean = i32;
pub type qhandle_t = i32;
pub type vec3_t = [f32; 3];

pub type Eorientations = i32;
pub type EG2_Collision = i32;

// defines to setup the
pub const ENTITY_WIDTH: u32 = 12;
pub const MODEL_WIDTH: u32 = 10;
pub const BOLT_WIDTH: u32 = 10;

pub const MODEL_AND: u32 = ((1 << MODEL_WIDTH) - 1);
pub const BOLT_AND: u32 = ((1 << BOLT_WIDTH) - 1);
pub const ENTITY_AND: u32 = ((1 << ENTITY_WIDTH) - 1);

pub const BOLT_SHIFT: u32 = 0;
pub const MODEL_SHIFT: u32 = (BOLT_SHIFT + BOLT_WIDTH);
pub const ENTITY_SHIFT: u32 = (MODEL_SHIFT + MODEL_WIDTH);

//rww - RAGDOLL_BEGIN
//rww - RAGDOLL_END

// internal surface calls  G2_surfaces.cpp
extern "C" {
    pub fn G2_SetSurfaceOnOff(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const i8,
        offFlags: i32,
    ) -> qboolean;

    pub fn G2_SetRootSurface(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: i32,
        surfaceName: *const i8,
    ) -> qboolean;

    pub fn G2_AddSurface(
        ghoul2: *mut CGhoul2Info,
        surfaceNumber: i32,
        polyNumber: i32,
        BarycentricI: f32,
        BarycentricJ: f32,
        lod: i32,
    ) -> i32;

    pub fn G2_RemoveSurface(slist: *mut surfaceInfo_v, index: i32) -> qboolean;

    pub fn G2_FindOverrideSurface(
        surfaceNum: i32,
        surfaceList: *const surfaceInfo_v,
    ) -> *const surfaceInfo_t;

    pub fn G2_IsSurfaceLegal(
        model: *const model_s,
        surfaceName: *const i8,
        flags: *mut i32,
    ) -> i32;

    pub fn G2_GetParentSurface(ghlInfo: *mut CGhoul2Info, index: i32) -> i32;

    pub fn G2_GetSurfaceIndex(ghlInfo: *mut CGhoul2Info, surfaceName: *const i8) -> i32;

    pub fn G2_IsSurfaceRendered(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const i8,
        slist: *mut surfaceInfo_v,
    ) -> i32;
}

// internal bone calls - G2_Bones.cpp
extern "C" {
    pub fn G2_Set_Bone_Angles(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
        angles: *const f32,
        flags: i32,
        up: Eorientations,
        left: Eorientations,
        forward: Eorientations,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2_Remove_Bone(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
    ) -> qboolean;

    pub fn G2_Remove_Bone_Index(blist: *mut boneInfo_v, index: i32) -> qboolean;

    pub fn G2_Set_Bone_Anim(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
        startFrame: i32,
        endFrame: i32,
        flags: i32,
        animSpeed: f32,
        currentTime: i32,
        setFrame: f32,
        blendTime: i32,
    ) -> qboolean;

    pub fn G2_Get_Bone_Anim(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
        currentTime: i32,
        currentFrame: *mut f32,
        startFrame: *mut i32,
        endFrame: *mut i32,
        flags: *mut i32,
        retAnimSpeed: *mut f32,
    ) -> qboolean;

    pub fn G2_Get_Bone_Anim_Range(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
        startFrame: *mut i32,
        endFrame: *mut i32,
    ) -> qboolean;

    pub fn G2_Get_Bone_Anim_Range_Index(
        blist: *mut boneInfo_v,
        boneIndex: i32,
        startFrame: *mut i32,
        endFrame: *mut i32,
    ) -> qboolean;

    pub fn G2_Pause_Bone_Anim(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2_Pause_Bone_Anim_Index(
        blist: *mut boneInfo_v,
        boneIndex: i32,
        currentTime: i32,
        numFrames: i32,
    ) -> qboolean;

    pub fn G2_IsPaused(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
    ) -> qboolean;

    pub fn G2_Stop_Bone_Anim(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
    ) -> qboolean;

    pub fn G2_Stop_Bone_Angles(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
    ) -> qboolean;

    //rww - RAGDOLL_BEGIN
    pub fn G2_Animate_Bone_List(
        ghoul2: *mut CGhoul2Info_v,
        currentTime: i32,
        index: i32,
        params: *mut CRagDollUpdateParams,
    );
    //rww - RAGDOLL_END

    pub fn G2_Init_Bone_List(blist: *mut boneInfo_v);

    pub fn G2_Find_Bone_In_List(blist: *mut boneInfo_v, boneNum: i32) -> i32;

    pub fn G2_Set_Bone_Angles_Matrix(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const i8,
        matrix: *const mdxaBone_t,
        flags: i32,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2_Get_Bone_Index(
        ghoul2: *mut CGhoul2Info,
        boneName: *const i8,
        bAddIfNotFound: qboolean,
    ) -> i32;

    pub fn G2_Set_Bone_Angles_Index(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        index: i32,
        angles: *const f32,
        flags: i32,
        yaw: Eorientations,
        pitch: Eorientations,
        roll: Eorientations,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2_Set_Bone_Angles_Matrix_Index(
        blist: *mut boneInfo_v,
        index: i32,
        matrix: *const mdxaBone_t,
        flags: i32,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2_Stop_Bone_Anim_Index(blist: *mut boneInfo_v, index: i32) -> qboolean;

    pub fn G2_Stop_Bone_Angles_Index(blist: *mut boneInfo_v, index: i32) -> qboolean;

    pub fn G2_Set_Bone_Anim_Index(
        blist: *mut boneInfo_v,
        index: i32,
        startFrame: i32,
        endFrame: i32,
        flags: i32,
        animSpeed: f32,
        currentTime: i32,
        setFrame: f32,
        blendTime: i32,
        numFrames: i32,
    ) -> qboolean;

    pub fn G2_Get_Bone_Anim_Index(
        blist: *mut boneInfo_v,
        index: i32,
        currentTime: i32,
        currentFrame: *mut f32,
        startFrame: *mut i32,
        endFrame: *mut i32,
        flags: *mut i32,
        retAnimSpeed: *mut f32,
        numFrames: i32,
    ) -> qboolean;
}

// misc functions G2_misc.cpp
extern "C" {
    pub fn G2_List_Model_Surfaces(fileName: *const i8);

    pub fn G2_List_Model_Bones(fileName: *const i8, frame: i32);

    pub fn G2_GetAnimFileName(fileName: *const i8, filename: *mut *mut i8) -> qboolean;

    #[cfg(feature = "G2_GORE")]
    pub fn G2_TraceModels(
        ghoul2: *mut CGhoul2Info_v,
        rayStart: *const f32,
        rayEnd: *const f32,
        collRecMap: *mut CCollisionRecord,
        entNum: i32,
        eG2TraceType: EG2_Collision,
        useLod: i32,
        fRadius: f32,
        ssize: f32,
        tsize: f32,
        theta: f32,
        shader: i32,
        gore: *mut SSkinGoreData,
        skipIfLODNotMatch: qboolean,
    );

    #[cfg(not(feature = "G2_GORE"))]
    pub fn G2_TraceModels(
        ghoul2: *mut CGhoul2Info_v,
        rayStart: *const f32,
        rayEnd: *const f32,
        collRecMap: *mut CCollisionRecord,
        entNum: i32,
        eG2TraceType: EG2_Collision,
        useLod: i32,
        fRadius: f32,
    );

    pub fn TransformAndTranslatePoint(
        in_: *const f32,
        out: *mut f32,
        mat: *mut mdxaBone_t,
    );

    #[cfg(feature = "G2_GORE")]
    pub fn G2_TransformModel(
        ghoul2: *mut CGhoul2Info_v,
        frameNum: i32,
        scale: *const f32,
        G2VertSpace: *mut CMiniHeap,
        useLod: i32,
        ApplyGore: bool,
        gore: *mut SSkinGoreData,
    );

    #[cfg(not(feature = "G2_GORE"))]
    pub fn G2_TransformModel(
        ghoul2: *mut CGhoul2Info_v,
        frameNum: i32,
        scale: *const f32,
        G2VertSpace: *mut CMiniHeap,
        useLod: i32,
    );

    pub fn G2_GenerateWorldMatrix(angles: *const f32, origin: *const f32);

    pub fn TransformPoint(in_: *const f32, out: *mut f32, mat: *mut mdxaBone_t);

    pub fn Inverse_Matrix(src: *mut mdxaBone_t, dest: *mut mdxaBone_t);

    pub fn G2_FindSurface(model: *const model_s, index: i32, lod: i32) -> *mut c_void;

    pub fn G2_SaveGhoul2Models(ghoul2: *mut CGhoul2Info_v);

    pub fn G2_LoadGhoul2Model(ghoul2: *mut CGhoul2Info_v, buffer: *mut i8);
}

// internal bolt calls. G2_bolts.cpp
extern "C" {
    pub fn G2_Add_Bolt(
        ghlInfo: *mut CGhoul2Info,
        bltlist: *mut boltInfo_v,
        slist: *mut surfaceInfo_v,
        boneName: *const i8,
    ) -> i32;

    pub fn G2_Remove_Bolt(bltlist: *mut boltInfo_v, index: i32) -> qboolean;

    pub fn G2_Init_Bolt_List(bltlist: *mut boltInfo_v);

    pub fn G2_Find_Bolt_Bone_Num(bltlist: *mut boltInfo_v, boneNum: i32) -> i32;

    pub fn G2_Find_Bolt_Surface_Num(
        bltlist: *mut boltInfo_v,
        surfaceNum: i32,
        flags: i32,
    ) -> i32;

    pub fn G2_Add_Bolt_Surf_Num(
        ghlInfo: *mut CGhoul2Info,
        bltlist: *mut boltInfo_v,
        slist: *mut surfaceInfo_v,
        surfNum: i32,
    ) -> i32;
}

// API calls - G2_API.cpp
extern "C" {
    pub fn G2API_PrecacheGhoul2Model(fileName: *const i8) -> qhandle_t;

    pub fn G2API_InitGhoul2Model(
        ghoul2: *mut CGhoul2Info_v,
        fileName: *const i8,
        modelIndex: i32,
        customSkin: qhandle_t,
        customShader: qhandle_t,
        modelFlags: i32,
        lodBias: i32,
    ) -> i32;

    pub fn G2API_SetLodBias(ghlInfo: *mut CGhoul2Info, lodBias: i32) -> qboolean;

    pub fn G2API_SetSkin(
        ghlInfo: *mut CGhoul2Info,
        customSkin: qhandle_t,
        renderSkin: qhandle_t,
    ) -> qboolean;

    pub fn G2API_SetShader(ghlInfo: *mut CGhoul2Info, customShader: qhandle_t) -> qboolean;

    pub fn G2API_RemoveGhoul2Model(ghlInfo: *mut CGhoul2Info_v, modelIndex: i32) -> qboolean;

    pub fn G2API_SetSurfaceOnOff(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const i8,
        flags: i32,
    ) -> qboolean;

    pub fn G2API_SetRootSurface(
        ghlInfo: *mut CGhoul2Info_v,
        modelIndex: i32,
        surfaceName: *const i8,
    ) -> qboolean;

    pub fn G2API_RemoveSurface(ghlInfo: *mut CGhoul2Info, index: i32) -> qboolean;

    pub fn G2API_AddSurface(
        ghlInfo: *mut CGhoul2Info,
        surfaceNumber: i32,
        polyNumber: i32,
        BarycentricI: f32,
        BarycentricJ: f32,
        lod: i32,
    ) -> i32;

    pub fn G2API_SetBoneAnim(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const i8,
        startFrame: i32,
        endFrame: i32,
        flags: i32,
        animSpeed: f32,
        currentTime: i32,
        setFrame: f32,
        blendTime: i32,
    ) -> qboolean;

    pub fn G2API_GetBoneAnim(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const i8,
        currentTime: i32,
        currentFrame: *mut f32,
        startFrame: *mut i32,
        endFrame: *mut i32,
        flags: *mut i32,
        animSpeed: *mut f32,
        modelList: *mut qhandle_t,
    ) -> qboolean;

    pub fn G2API_GetBoneAnimIndex(
        ghlInfo: *mut CGhoul2Info,
        iBoneIndex: i32,
        currentTime: i32,
        currentFrame: *mut f32,
        startFrame: *mut i32,
        endFrame: *mut i32,
        flags: *mut i32,
        animSpeed: *mut f32,
        modelList: *mut qhandle_t,
    ) -> qboolean;

    pub fn G2API_GetAnimRange(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const i8,
        startFrame: *mut i32,
        endFrame: *mut i32,
    ) -> qboolean;

    pub fn G2API_GetAnimRangeIndex(
        ghlInfo: *mut CGhoul2Info,
        boneIndex: i32,
        startFrame: *mut i32,
        endFrame: *mut i32,
    ) -> qboolean;

    pub fn G2API_PauseBoneAnim(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const i8,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2API_PauseBoneAnimIndex(
        ghlInfo: *mut CGhoul2Info,
        boneIndex: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2API_IsPaused(ghlInfo: *mut CGhoul2Info, boneName: *const i8) -> qboolean;

    pub fn G2API_StopBoneAnim(ghlInfo: *mut CGhoul2Info, boneName: *const i8) -> qboolean;

    pub fn G2API_SetBoneAngles(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const i8,
        angles: *const f32,
        flags: i32,
        up: Eorientations,
        right: Eorientations,
        forward: Eorientations,
        modelList: *mut qhandle_t,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2API_StopBoneAngles(ghlInfo: *mut CGhoul2Info, boneName: *const i8) -> qboolean;

    pub fn G2API_RemoveBone(ghlInfo: *mut CGhoul2Info, boneName: *const i8) -> qboolean;

    pub fn G2API_RemoveBolt(ghlInfo: *mut CGhoul2Info, index: i32) -> qboolean;

    pub fn G2API_AddBolt(ghlInfo: *mut CGhoul2Info, boneName: *const i8) -> i32;

    pub fn G2API_AddBoltSurfNum(ghlInfo: *mut CGhoul2Info, surfIndex: i32) -> i32;

    pub fn G2API_AttachG2Model(
        ghlInfo: *mut CGhoul2Info,
        ghlInfoTo: *mut CGhoul2Info,
        toBoltIndex: i32,
        toModel: i32,
    ) -> qboolean;

    pub fn G2API_DetachG2Model(ghlInfo: *mut CGhoul2Info) -> qboolean;

    pub fn G2API_AttachEnt(
        boltInfo: *mut i32,
        ghlInfoTo: *mut CGhoul2Info,
        toBoltIndex: i32,
        entNum: i32,
        toModelNum: i32,
    ) -> qboolean;

    pub fn G2API_DetachEnt(boltInfo: *mut i32);

    pub fn G2API_GetBoltMatrix(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: i32,
        boltIndex: i32,
        matrix: *mut mdxaBone_t,
        angles: *const f32,
        position: *const f32,
        frameNum: i32,
        modelList: *mut qhandle_t,
        scale: *const f32,
    ) -> qboolean;

    pub fn G2API_ListSurfaces(ghlInfo: *mut CGhoul2Info);

    pub fn G2API_ListBones(ghlInfo: *mut CGhoul2Info, frame: i32);

    pub fn G2API_HaveWeGhoul2Models(ghoul2: *mut CGhoul2Info_v) -> qboolean;

    pub fn G2API_SetGhoul2ModelIndexes(
        ghoul2: *mut CGhoul2Info_v,
        modelList: *mut qhandle_t,
        skinList: *mut qhandle_t,
    );

    pub fn G2API_SetGhoul2ModelFlags(ghlInfo: *mut CGhoul2Info, flags: i32) -> qboolean;

    pub fn G2API_GetGhoul2ModelFlags(ghlInfo: *mut CGhoul2Info) -> i32;

    pub fn G2API_GetAnimFileName(ghlInfo: *mut CGhoul2Info, filename: *mut *mut i8) -> qboolean;

    pub fn G2API_CollisionDetect(
        collRecMap: *mut CCollisionRecord,
        ghoul2: *mut CGhoul2Info_v,
        angles: *const f32,
        position: *const f32,
        frameNumber: i32,
        entNum: i32,
        rayStart: *const f32,
        rayEnd: *const f32,
        scale: *const f32,
        G2VertSpace: *mut CMiniHeap,
        eG2TraceType: EG2_Collision,
        useLod: i32,
        fRadius: f32,
    );

    pub fn G2API_GiveMeVectorFromMatrix(
        boltMatrix: *mut mdxaBone_t,
        flags: Eorientations,
        vec: *mut f32,
    );

    pub fn G2API_CopyGhoul2Instance(
        Ghoul2From: *mut CGhoul2Info_v,
        Ghoul2To: *mut CGhoul2Info_v,
        modelIndex: i32,
    );

    pub fn G2API_CleanGhoul2Models(ghoul2: *mut CGhoul2Info_v);

    pub fn G2API_GetParentSurface(ghlInfo: *mut CGhoul2Info, index: i32) -> i32;

    pub fn G2API_GetSurfaceIndex(ghlInfo: *mut CGhoul2Info, surfaceName: *const i8) -> i32;

    pub fn G2API_GetSurfaceName(ghlInfo: *mut CGhoul2Info, surfNumber: i32) -> *mut i8;

    pub fn G2API_GetGLAName(ghlInfo: *mut CGhoul2Info) -> *mut i8;

    pub fn G2API_SetBoneAnglesMatrix(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const i8,
        matrix: *const mdxaBone_t,
        flags: i32,
        modelList: *mut qhandle_t,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2API_SetNewOrigin(ghlInfo: *mut CGhoul2Info, boltIndex: i32) -> qboolean;

    pub fn G2API_GetBoneIndex(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const i8,
        bAddIfNotFound: qboolean,
    ) -> i32;

    pub fn G2API_StopBoneAnglesIndex(ghlInfo: *mut CGhoul2Info, index: i32) -> qboolean;

    pub fn G2API_StopBoneAnimIndex(ghlInfo: *mut CGhoul2Info, index: i32) -> qboolean;

    pub fn G2API_SetBoneAnglesIndex(
        ghlInfo: *mut CGhoul2Info,
        index: i32,
        angles: *const f32,
        flags: i32,
        yaw: Eorientations,
        pitch: Eorientations,
        roll: Eorientations,
        modelList: *mut qhandle_t,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2API_SetBoneAnglesMatrixIndex(
        ghlInfo: *mut CGhoul2Info,
        index: i32,
        matrix: *const mdxaBone_t,
        flags: i32,
        modelList: *mut qhandle_t,
        blendTime: i32,
        currentTime: i32,
    ) -> qboolean;

    pub fn G2API_SetBoneAnimIndex(
        ghlInfo: *mut CGhoul2Info,
        index: i32,
        startFrame: i32,
        endFrame: i32,
        flags: i32,
        animSpeed: f32,
        currentTime: i32,
        setFrame: f32,
        blendTime: i32,
    ) -> qboolean;

    pub fn G2API_SetAnimIndex(ghlInfo: *mut CGhoul2Info, index: i32) -> qboolean;

    pub fn G2API_GetAnimIndex(ghlInfo: *mut CGhoul2Info) -> i32;

    pub fn G2API_SaveGhoul2Models(ghoul2: *mut CGhoul2Info_v);

    pub fn G2API_LoadGhoul2Models(ghoul2: *mut CGhoul2Info_v, buffer: *mut i8);

    pub fn G2API_LoadSaveCodeDestructGhoul2Info(ghoul2: *mut CGhoul2Info_v);

    pub fn G2API_GetAnimFileNameIndex(modelIndex: qhandle_t) -> *mut i8;

    pub fn G2API_GetAnimFileInternalNameIndex(modelIndex: qhandle_t) -> *mut i8;

    pub fn G2API_GetSurfaceRenderStatus(ghlInfo: *mut CGhoul2Info, surfaceName: *const i8)
        -> i32;
}

// From tr_ghoul2.cpp
extern "C" {
    pub fn G2_ConstructGhoulSkeleton(
        ghoul2: *mut CGhoul2Info_v,
        frameNum: i32,
        checkForNewOrigin: bool,
        scale: *const f32,
    );

    pub fn G2_GetBoltMatrixLow(
        ghoul2: *mut CGhoul2Info,
        boltNum: i32,
        scale: *const f32,
        retMatrix: *mut mdxaBone_t,
    );

    pub fn G2_TimingModel(
        bone: *mut boneInfo_t,
        time: i32,
        numFramesInFile: i32,
        currentFrame: *mut i32,
        newFrame: *mut i32,
        lerp: *mut f32,
    );

    // PORTING NOTE: C++ overloads G2_SetupModelPointers(CGhoul2Info_v &) and
    // G2_SetupModelPointers(CGhoul2Info *) cannot both have the same name in Rust extern "C".
    // This version corresponds to the CGhoul2Info_v overload (returns true if any model is properly set up).
    pub fn G2_SetupModelPointers(ghoul2: *mut CGhoul2Info_v) -> bool;

    // PORTING NOTE: This version corresponds to the CGhoul2Info pointer overload
    // (returns true if the model is properly set up).
    pub fn G2_SetupModelPointers_single(ghlInfo: *mut CGhoul2Info) -> bool;
}

//#ifdef _G2_GORE	// These exist regardless, non-gore versions are empty
extern "C" {
    pub fn G2API_AddSkinGore(ghoul2: *mut CGhoul2Info_v, gore: *mut SSkinGoreData);

    pub fn G2API_ClearSkinGore(ghoul2: *mut CGhoul2Info_v);
}
//#endif
