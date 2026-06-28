// defines to setup the

use core::ffi::{c_char, c_int, c_void};

// Note: Following types and forward declarations are resolved from ghoul2_shared.h
// and related headers. These represent opaque C++ class pointers.
pub enum CGhoul2Info {}
pub enum surfaceInfo_v {}
pub enum CGhoul2Info_v {}
pub enum boneInfo_v {}
pub enum qhandle_t {}
pub enum qboolean {}
pub enum Eorientations {}
pub enum mdxaBone_t {}
pub enum CollisionRecord_t {}
pub enum SSkinGoreData {}
pub enum CMiniHeap {}
pub enum CRagDollUpdateParams {}
pub enum CRagDollParams {}
pub enum sharedSetBoneIKStateParams_t {}
pub enum sharedIKMoveParams_t {}

// Aliases for C vector types
pub type vec3_t = [f32; 3];

// hack for smoothing during ugly situations. forgive me.
pub const GHOUL2_CRAZY_SMOOTH: c_int = 0x2000;

// internal surface calls  G2_surfaces.cpp
extern "C" {
    pub fn G2_SetSurfaceOnOff(
        ghlInfo: *mut CGhoul2Info,
        slist: *mut surfaceInfo_v,
        surfaceName: *const c_char,
        offFlags: c_int,
    ) -> c_int;

    pub fn G2_IsSurfaceOff(
        ghlInfo: *mut CGhoul2Info,
        slist: *mut surfaceInfo_v,
        surfaceName: *const c_char,
    ) -> c_int;

    pub fn G2_SetRootSurface(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        surfaceName: *const c_char,
    ) -> c_int;

    pub fn G2_AddSurface(
        ghoul2: *mut CGhoul2Info,
        surfaceNumber: c_int,
        polyNumber: c_int,
        BarycentricI: f32,
        BarycentricJ: f32,
        lod: c_int,
    ) -> c_int;

    pub fn G2_RemoveSurface(
        slist: *mut surfaceInfo_v,
        index: c_int,
    ) -> c_int;

    pub fn G2_FindOverrideSurface(
        surfaceNum: c_int,
        surfaceList: *mut surfaceInfo_v,
    ) -> *mut c_void;

    pub fn G2_IsSurfaceLegal(
        mod_: *mut c_void,
        surfaceName: *const c_char,
        flags: *mut c_int,
    ) -> c_int;

    pub fn G2_GetParentSurface(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
    ) -> c_int;

    pub fn G2_GetSurfaceIndex(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const c_char,
    ) -> c_int;

    pub fn G2_IsSurfaceRendered(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const c_char,
        slist: *mut surfaceInfo_v,
    ) -> c_int;
}

// internal bone calls - G2_Bones.cpp
extern "C" {
    pub fn G2_Set_Bone_Angles(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
        angles: *const f32,
        flags: c_int,
        up: Eorientations,
        left: Eorientations,
        forward: Eorientations,
        modelList: *mut qhandle_t,
        modelIndex: c_int,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2_Remove_Bone(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2_Set_Bone_Anim(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
        startFrame: c_int,
        endFrame: c_int,
        flags: c_int,
        animSpeed: f32,
        currentTime: c_int,
        setFrame: f32,
        blendTime: c_int,
    ) -> c_int;

    pub fn G2_Get_Bone_Anim(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
        currentTime: c_int,
        currentFrame: *mut f32,
        startFrame: *mut c_int,
        endFrame: *mut c_int,
        flags: *mut c_int,
        retAnimSpeed: *mut f32,
        modelList: *mut qhandle_t,
        modelIndex: c_int,
    ) -> c_int;

    pub fn G2_Get_Bone_Anim_Range(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
        startFrame: *mut c_int,
        endFrame: *mut c_int,
    ) -> c_int;

    pub fn G2_Pause_Bone_Anim(
        ghlInfo: *mut CGhoul2Info,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2_IsPaused(
        fileName: *const c_char,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2_Stop_Bone_Anim(
        fileName: *const c_char,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2_Stop_Bone_Angles(
        fileName: *const c_char,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
    ) -> c_int;

    // RAGDOLL_BEGIN
    pub fn G2_Animate_Bone_List(
        ghoul2: *mut CGhoul2Info_v,
        currentTime: c_int,
        index: c_int,
        params: *mut CRagDollUpdateParams,
    );
    // RAGDOLL_END

    pub fn G2_Init_Bone_List(blist: *mut boneInfo_v);

    pub fn G2_Find_Bone_In_List(
        blist: *mut boneInfo_v,
        boneNum: c_int,
    ) -> c_int;

    pub fn G2_RemoveRedundantBoneOverrides(
        blist: *mut boneInfo_v,
        activeBones: *mut c_int,
    );

    pub fn G2_Set_Bone_Angles_Matrix(
        fileName: *const c_char,
        blist: *mut boneInfo_v,
        boneName: *const c_char,
        matrix: *const mdxaBone_t,
        flags: c_int,
        modelList: *mut qhandle_t,
        modelIndex: c_int,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2_Get_Bone_Index(
        ghoul2: *mut CGhoul2Info,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2_Set_Bone_Angles_Index(
        blist: *mut boneInfo_v,
        index: c_int,
        angles: *const f32,
        flags: c_int,
        yaw: Eorientations,
        pitch: Eorientations,
        roll: Eorientations,
        modelList: *mut qhandle_t,
        modelIndex: c_int,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2_Set_Bone_Angles_Matrix_Index(
        blist: *mut boneInfo_v,
        index: c_int,
        matrix: *const mdxaBone_t,
        flags: c_int,
        modelList: *mut qhandle_t,
        modelIndex: c_int,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2_Stop_Bone_Anim_Index(
        blist: *mut boneInfo_v,
        index: c_int,
    ) -> c_int;

    pub fn G2_Stop_Bone_Angles_Index(
        blist: *mut boneInfo_v,
        index: c_int,
    ) -> c_int;

    pub fn G2_Set_Bone_Anim_Index(
        blist: *mut boneInfo_v,
        index: c_int,
        startFrame: c_int,
        endFrame: c_int,
        flags: c_int,
        animSpeed: f32,
        currentTime: c_int,
        setFrame: f32,
        blendTime: c_int,
        numFrames: c_int,
    ) -> c_int;

    pub fn G2_Get_Bone_Anim_Index(
        blist: *mut boneInfo_v,
        index: c_int,
        currentTime: c_int,
        currentFrame: *mut f32,
        startFrame: *mut c_int,
        endFrame: *mut c_int,
        flags: *mut c_int,
        retAnimSpeed: *mut f32,
        modelList: *mut qhandle_t,
        modelIndex: c_int,
    ) -> c_int;
}

// misc functions G2_misc.cpp
extern "C" {
    pub fn G2_List_Model_Surfaces(fileName: *const c_char);

    pub fn G2_List_Model_Bones(
        fileName: *const c_char,
        frame: c_int,
    );

    pub fn G2_GetAnimFileName(
        fileName: *const c_char,
        filename: *mut *mut c_char,
    ) -> c_int;

    #[cfg(feature = "g2_gore")]
    pub fn G2_TraceModels(
        ghoul2: *mut CGhoul2Info_v,
        rayStart: *const vec3_t,
        rayEnd: *const vec3_t,
        collRecMap: *mut CollisionRecord_t,
        entNum: c_int,
        traceFlags: c_int,
        useLod: c_int,
        fRadius: f32,
        ssize: f32,
        tsize: f32,
        theta: f32,
        shader: c_int,
        gore: *mut SSkinGoreData,
        skipIfLODNotMatch: c_int,
    );

    #[cfg(not(feature = "g2_gore"))]
    pub fn G2_TraceModels(
        ghoul2: *mut CGhoul2Info_v,
        rayStart: *const vec3_t,
        rayEnd: *const vec3_t,
        collRecMap: *mut CollisionRecord_t,
        entNum: c_int,
        traceFlags: c_int,
        useLod: c_int,
        fRadius: f32,
    );

    pub fn TransformAndTranslatePoint(
        in_: *const vec3_t,
        out: *mut vec3_t,
        mat: *mut mdxaBone_t,
    );

    #[cfg(feature = "g2_gore")]
    pub fn G2_TransformModel(
        ghoul2: *mut CGhoul2Info_v,
        frameNum: c_int,
        scale: *const vec3_t,
        G2VertSpace: *mut CMiniHeap,
        useLod: c_int,
        ApplyGore: bool,
    );

    #[cfg(not(feature = "g2_gore"))]
    pub fn G2_TransformModel(
        ghoul2: *mut CGhoul2Info_v,
        frameNum: c_int,
        scale: *const vec3_t,
        G2VertSpace: *mut CMiniHeap,
        useLod: c_int,
    );

    pub fn G2_GenerateWorldMatrix(
        angles: *const vec3_t,
        origin: *const vec3_t,
    );

    pub fn TransformPoint(
        in_: *const vec3_t,
        out: *mut vec3_t,
        mat: *mut mdxaBone_t,
    );

    pub fn Inverse_Matrix(
        src: *mut mdxaBone_t,
        dest: *mut mdxaBone_t,
    );

    pub fn G2_FindSurface(
        mod_: *mut c_void,
        index: c_int,
        lod: c_int,
    ) -> *mut c_void;

    pub fn G2_SaveGhoul2Models(
        ghoul2: *mut CGhoul2Info_v,
        buffer: *mut *mut c_char,
        size: *mut c_int,
    ) -> c_int;

    pub fn G2_LoadGhoul2Model(
        ghoul2: *mut CGhoul2Info_v,
        buffer: *mut c_char,
    );
}

// internal bolt calls. G2_bolts.cpp
extern "C" {
    pub fn G2_Add_Bolt(
        ghlInfo: *mut CGhoul2Info,
        bltlist: *mut boltInfo_v,
        slist: *mut surfaceInfo_v,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2_Remove_Bolt(
        bltlist: *mut boltInfo_v,
        index: c_int,
    ) -> c_int;

    pub fn G2_Init_Bolt_List(bltlist: *mut boltInfo_v);

    pub fn G2_Find_Bolt_Bone_Num(
        bltlist: *mut boltInfo_v,
        boneNum: c_int,
    ) -> c_int;

    pub fn G2_Find_Bolt_Surface_Num(
        bltlist: *mut boltInfo_v,
        surfaceNum: c_int,
        flags: c_int,
    ) -> c_int;

    pub fn G2_Add_Bolt_Surf_Num(
        ghlInfo: *mut CGhoul2Info,
        bltlist: *mut boltInfo_v,
        slist: *mut surfaceInfo_v,
        surfNum: c_int,
    ) -> c_int;

    pub fn G2_RemoveRedundantBolts(
        bltlist: *mut boltInfo_v,
        slist: *mut surfaceInfo_v,
        activeSurfaces: *mut c_int,
        activeBones: *mut c_int,
    );
}

pub enum boltInfo_v {}

// API calls - G2_API.cpp
extern "C" {
    pub fn G2API_SetTime(
        currentTime: c_int,
        clock: c_int,
    );

    pub fn G2API_GetTime(argTime: c_int) -> c_int;

    pub fn G2API_PrecacheGhoul2Model(fileName: *const c_char) -> qhandle_t;

    pub fn G2API_InitGhoul2Model(
        ghoul2Ptr: *mut *mut CGhoul2Info_v,
        fileName: *const c_char,
        modelIndex: c_int,
        customSkin: qhandle_t,
        customShader: qhandle_t,
        modelFlags: c_int,
        lodBias: c_int,
    ) -> c_int;

    pub fn G2API_SetLodBias(
        ghlInfo: *mut CGhoul2Info,
        lodBias: c_int,
    ) -> c_int;

    pub fn G2API_SetSkin(
        ghlInfo: *mut CGhoul2Info,
        customSkin: qhandle_t,
        renderSkin: qhandle_t,
    ) -> c_int;

    pub fn G2API_SetShader(
        ghlInfo: *mut CGhoul2Info,
        customShader: qhandle_t,
    ) -> c_int;

    pub fn G2API_HasGhoul2ModelOnIndex(
        ghlRemove: *mut *mut CGhoul2Info_v,
        modelIndex: c_int,
    ) -> c_int;

    pub fn G2API_RemoveGhoul2Model(
        ghlRemove: *mut *mut CGhoul2Info_v,
        modelIndex: c_int,
    ) -> c_int;

    pub fn G2API_RemoveGhoul2Models(
        ghlRemove: *mut *mut CGhoul2Info_v,
    ) -> c_int;

    pub fn G2API_SetSurfaceOnOff(
        ghoul2: *mut CGhoul2Info_v,
        surfaceName: *const c_char,
        flags: c_int,
    ) -> c_int;

    pub fn G2API_GetSurfaceOnOff(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const c_char,
    ) -> c_int;

    pub fn G2API_SetRootSurface(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        surfaceName: *const c_char,
    ) -> c_int;

    pub fn G2API_RemoveSurface(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
    ) -> c_int;

    pub fn G2API_AddSurface(
        ghlInfo: *mut CGhoul2Info,
        surfaceNumber: c_int,
        polyNumber: c_int,
        BarycentricI: f32,
        BarycentricJ: f32,
        lod: c_int,
    ) -> c_int;

    pub fn G2API_SetBoneAnim(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        boneName: *const c_char,
        startFrame: c_int,
        endFrame: c_int,
        flags: c_int,
        animSpeed: f32,
        currentTime: c_int,
        setFrame: f32,
        blendTime: c_int,
    ) -> c_int;

    pub fn G2API_GetBoneAnim(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
        currentTime: c_int,
        currentFrame: *mut f32,
        startFrame: *mut c_int,
        endFrame: *mut c_int,
        flags: *mut c_int,
        animSpeed: *mut f32,
        modelList: *mut qhandle_t,
    ) -> c_int;

    pub fn G2API_GetAnimRange(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
        startFrame: *mut c_int,
        endFrame: *mut c_int,
    ) -> c_int;

    pub fn G2API_PauseBoneAnim(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2API_IsPaused(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2API_StopBoneAnim(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2API_SetBoneAngles(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        boneName: *const c_char,
        angles: *const vec3_t,
        flags: c_int,
        up: Eorientations,
        left: Eorientations,
        forward: Eorientations,
        modelList: *mut qhandle_t,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2API_StopBoneAngles(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2API_RemoveBone(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2API_AnimateG2Models(
        ghoul2: *mut CGhoul2Info_v,
        speedVar: f32,
    );

    pub fn G2API_RemoveBolt(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
    ) -> c_int;

    pub fn G2API_AddBolt(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2API_AddBoltSurfNum(
        ghlInfo: *mut CGhoul2Info,
        surfIndex: c_int,
    ) -> c_int;

    pub fn G2API_SetBoltInfo(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        boltInfo: c_int,
    );

    pub fn G2API_AttachG2Model(
        ghoul2From: *mut CGhoul2Info_v,
        modelFrom: c_int,
        ghoul2To: *mut CGhoul2Info_v,
        toBoltIndex: c_int,
        toModel: c_int,
    ) -> c_int;

    pub fn G2API_DetachG2Model(ghlInfo: *mut CGhoul2Info) -> c_int;

    pub fn G2API_AttachEnt(
        boltInfo: *mut c_int,
        ghlInfoTo: *mut CGhoul2Info,
        toBoltIndex: c_int,
        entNum: c_int,
        toModelNum: c_int,
    ) -> c_int;

    pub fn G2API_DetachEnt(boltInfo: *mut c_int);

    pub fn G2API_GetBoltMatrix(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut mdxaBone_t,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNum: c_int,
        modelList: *mut qhandle_t,
        scale: *const vec3_t,
    ) -> c_int;

    pub fn G2API_ListSurfaces(ghlInfo: *mut CGhoul2Info);

    pub fn G2API_ListBones(
        ghlInfo: *mut CGhoul2Info,
        frame: c_int,
    );

    pub fn G2API_HaveWeGhoul2Models(ghoul2: *mut CGhoul2Info_v) -> c_int;

    pub fn G2API_SetGhoul2ModelIndexes(
        ghoul2: *mut CGhoul2Info_v,
        modelList: *mut qhandle_t,
        skinList: *mut qhandle_t,
    );

    pub fn G2API_SetGhoul2ModelFlags(
        ghlInfo: *mut CGhoul2Info,
        flags: c_int,
    ) -> c_int;

    pub fn G2API_GetGhoul2ModelFlags(ghlInfo: *mut CGhoul2Info) -> c_int;

    pub fn G2API_GetAnimFileName(
        ghlInfo: *mut CGhoul2Info,
        filename: *mut *mut c_char,
    ) -> c_int;

    pub fn G2API_CollisionDetect(
        collRecMap: *mut CollisionRecord_t,
        ghoul2: *mut CGhoul2Info_v,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNumber: c_int,
        entNum: c_int,
        rayStart: *const vec3_t,
        rayEnd: *const vec3_t,
        scale: *const vec3_t,
        G2VertSpace: *mut CMiniHeap,
        traceFlags: c_int,
        useLod: c_int,
        fRadius: f32,
    );

    pub fn G2API_CollisionDetectCache(
        collRecMap: *mut CollisionRecord_t,
        ghoul2: *mut CGhoul2Info_v,
        angles: *const vec3_t,
        position: *const vec3_t,
        frameNumber: c_int,
        entNum: c_int,
        rayStart: *const vec3_t,
        rayEnd: *const vec3_t,
        scale: *const vec3_t,
        G2VertSpace: *mut CMiniHeap,
        traceFlags: c_int,
        useLod: c_int,
        fRadius: f32,
    );

    pub fn G2API_GiveMeVectorFromMatrix(
        boltMatrix: *mut mdxaBone_t,
        flags: Eorientations,
        vec: *mut vec3_t,
    );

    pub fn G2API_CopyGhoul2Instance(
        g2From: *mut CGhoul2Info_v,
        g2To: *mut CGhoul2Info_v,
        modelIndex: c_int,
    ) -> c_int;

    pub fn G2API_CleanGhoul2Models(ghoul2Ptr: *mut *mut CGhoul2Info_v);

    pub fn G2API_GetParentSurface(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
    ) -> c_int;

    pub fn G2API_GetSurfaceIndex(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const c_char,
    ) -> c_int;

    pub fn G2API_GetSurfaceName(
        ghlInfo: *mut CGhoul2Info,
        surfNumber: c_int,
    ) -> *mut c_char;

    pub fn G2API_GetGLAName(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
    ) -> *mut c_char;

    pub fn G2API_SetBoneAnglesMatrix(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
        matrix: *const mdxaBone_t,
        flags: c_int,
        modelList: *mut qhandle_t,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2API_SetNewOrigin(
        ghoul2: *mut CGhoul2Info_v,
        boltIndex: c_int,
    ) -> c_int;

    pub fn G2API_GetBoneIndex(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2API_StopBoneAnglesIndex(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
    ) -> c_int;

    pub fn G2API_StopBoneAnimIndex(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
    ) -> c_int;

    pub fn G2API_SetBoneAnglesIndex(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
        angles: *const vec3_t,
        flags: c_int,
        yaw: c_int,
        pitch: c_int,
        roll: c_int,
        modelList: *mut qhandle_t,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2API_SetBoneAnglesMatrixIndex(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
        matrix: *const mdxaBone_t,
        flags: c_int,
        modelList: *mut qhandle_t,
        blendTime: c_int,
        currentTime: c_int,
    ) -> c_int;

    pub fn G2API_DoesBoneExist(
        ghlInfo: *mut CGhoul2Info,
        boneName: *const c_char,
    ) -> c_int;

    pub fn G2API_SetBoneAnimIndex(
        ghlInfo: *mut CGhoul2Info,
        index: c_int,
        startFrame: c_int,
        endFrame: c_int,
        flags: c_int,
        animSpeed: f32,
        currentTime: c_int,
        setFrame: f32,
        blendTime: c_int,
    ) -> c_int;

    pub fn G2API_SaveGhoul2Models(
        ghoul2: *mut CGhoul2Info_v,
        buffer: *mut *mut c_char,
        size: *mut c_int,
    ) -> c_int;

    pub fn G2API_LoadGhoul2Models(
        ghoul2: *mut CGhoul2Info_v,
        buffer: *mut c_char,
    );

    pub fn G2API_LoadSaveCodeDestructGhoul2Info(
        ghoul2: *mut CGhoul2Info_v,
    );

    pub fn G2API_FreeSaveBuffer(buffer: *mut c_char);

    pub fn G2API_GetAnimFileNameIndex(modelIndex: qhandle_t) -> *mut c_char;

    pub fn G2API_GetSurfaceRenderStatus(
        ghlInfo: *mut CGhoul2Info,
        surfaceName: *const c_char,
    ) -> c_int;

    pub fn G2API_CopySpecificG2Model(
        ghoul2From: *mut CGhoul2Info_v,
        modelFrom: c_int,
        ghoul2To: *mut CGhoul2Info_v,
        modelTo: c_int,
    );

    pub fn G2API_DuplicateGhoul2Instance(
        g2From: *mut CGhoul2Info_v,
        g2To: *mut *mut CGhoul2Info_v,
    );

    pub fn G2API_SetBoltInfo(
        ghoul2: *mut CGhoul2Info_v,
        modelIndex: c_int,
        boltInfo: c_int,
    );

    pub fn G2API_AbsurdSmoothing(
        ghoul2: *mut CGhoul2Info_v,
        status: c_int,
    );

    pub fn G2API_SetRagDoll(
        ghoul2: *mut CGhoul2Info_v,
        parms: *mut CRagDollParams,
    );

    pub fn G2API_ResetRagDoll(ghoul2: *mut CGhoul2Info_v);

    pub fn G2API_AnimateG2Models_RagDoll(
        ghoul2: *mut CGhoul2Info_v,
        AcurrentTime: c_int,
        params: *mut CRagDollUpdateParams,
    );

    pub fn G2API_RagPCJConstraint(
        ghoul2: *mut CGhoul2Info_v,
        boneName: *const c_char,
        min: *mut vec3_t,
        max: *mut vec3_t,
    ) -> c_int;

    pub fn G2API_RagPCJGradientSpeed(
        ghoul2: *mut CGhoul2Info_v,
        boneName: *const c_char,
        speed: f32,
    ) -> c_int;

    pub fn G2API_RagEffectorGoal(
        ghoul2: *mut CGhoul2Info_v,
        boneName: *const c_char,
        pos: *mut vec3_t,
    ) -> c_int;

    pub fn G2API_GetRagBonePos(
        ghoul2: *mut CGhoul2Info_v,
        boneName: *const c_char,
        pos: *mut vec3_t,
        entAngles: *mut vec3_t,
        entPos: *mut vec3_t,
        entScale: *mut vec3_t,
    ) -> c_int;

    pub fn G2API_RagEffectorKick(
        ghoul2: *mut CGhoul2Info_v,
        boneName: *const c_char,
        velocity: *mut vec3_t,
    ) -> c_int;

    pub fn G2API_RagForceSolve(
        ghoul2: *mut CGhoul2Info_v,
        force: c_int,
    ) -> c_int;

    pub fn G2API_SetBoneIKState(
        ghoul2: *mut CGhoul2Info_v,
        time: c_int,
        boneName: *const c_char,
        ikState: c_int,
        params: *mut sharedSetBoneIKStateParams_t,
    ) -> c_int;

    pub fn G2API_IKMove(
        ghoul2: *mut CGhoul2Info_v,
        time: c_int,
        params: *mut sharedIKMoveParams_t,
    ) -> c_int;

    pub fn G2API_AttachInstanceToEntNum(
        ghoul2: *mut CGhoul2Info_v,
        entityNum: c_int,
        server: c_int,
    );

    pub fn G2API_ClearAttachedInstance(entityNum: c_int);

    pub fn G2API_CleanEntAttachments();

    pub fn G2API_OverrideServerWithClientData(
        serverInstance: *mut CGhoul2Info,
    ) -> c_int;

    // From tr_ghoul2.cpp
    pub fn G2_ConstructGhoulSkeleton(
        ghoul2: *mut CGhoul2Info_v,
        frameNum: c_int,
        checkForNewOrigin: bool,
        scale: *const vec3_t,
    );

    pub fn G2API_SkinlessModel(g2: *mut CGhoul2Info) -> c_int;

    #[cfg(feature = "g2_gore")]
    pub fn G2API_GetNumGoreMarks(g2: *mut CGhoul2Info) -> c_int;

    #[cfg(feature = "g2_gore")]
    pub fn G2API_AddSkinGore(
        ghoul2: *mut CGhoul2Info_v,
        gore: *mut SSkinGoreData,
    );

    #[cfg(feature = "g2_gore")]
    pub fn G2API_ClearSkinGore(ghoul2: *mut CGhoul2Info_v);

    pub fn G2API_Ghoul2Size(ghoul2: *mut CGhoul2Info_v) -> c_int;
}

pub static mut gG2_GBMNoReconstruct: c_int = 0;
pub static mut gG2_GBMUseSPMethod: c_int = 0;
