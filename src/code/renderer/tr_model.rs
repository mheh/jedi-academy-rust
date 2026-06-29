// tr_models.c -- model loading and caching

// leave this as first line for PCH reasons...
//
// #include "../server/exe_headers.h"
// #include "tr_local.h"
// #include "MatComp.h"
// #include "../qcommon/sstring.h"

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut, null_mut};

#[allow(non_snake_case)]
mod tr_model {
    use super::*;
    use std::collections::HashMap;

    macro_rules! LL {
        ($x:expr) => {
            $x = LittleLong($x)
        };
    }

    // Forward declarations / stubs (will be defined later in extern block)

    /*
    Ghoul2 Insert Start
    */

    #[repr(C)]
    struct modelHash_s {
        name: [c_char; 260], // MAX_QPATH
        handle: c_int,
        next: *mut modelHash_s,
    }

    type modelHash_t = modelHash_s;

    const FILE_HASH_SIZE: usize = 1024;
    static mut mhHashTable: [*mut modelHash_t; FILE_HASH_SIZE] = [null_mut(); FILE_HASH_SIZE];

    /*
    Ghoul2 Insert End
    */

    // This stuff looks a bit messy, but it's kept here as black box, and nothing appears in any .H files for other
    //	modules to worry about. I may make another module for this sometime.
    //

    type StringOffsetAndShaderIndexDest_t = (c_int, c_int);

    #[repr(C)]
    struct CachedEndianedModelBinary_s {
        pModelDiskImage: *mut c_void,
        iAllocSize: c_int, // may be useful for mem-query, but I don't actually need it
        ShaderRegisterData: Vec<StringOffsetAndShaderIndexDest_t>,
        iLastLevelUsedOn: c_int,
    }

    impl CachedEndianedModelBinary_s {
        fn new() -> Self {
            CachedEndianedModelBinary_s {
                pModelDiskImage: null_mut(),
                iAllocSize: 0,
                ShaderRegisterData: Vec::new(),
                iLastLevelUsedOn: -1,
            }
        }
    }

    type CachedEndianedModelBinary_t = CachedEndianedModelBinary_s;
    type CachedModels_t = HashMap<String, CachedEndianedModelBinary_t>;

    static mut CachedModels: *mut CachedModels_t = null_mut(); // the important cache item.

    fn RE_RegisterModels_StoreShaderRequest(
        psModelFileName: *const c_char,
        psShaderName: *const c_char,
        piShaderIndexPoke: *const c_int,
    ) {
        unsafe {
            let mut sModelName: [c_char; 260] = [0; 260]; // MAX_QPATH

            Q_strncpyz(
                &mut sModelName as *mut c_char,
                psModelFileName,
                260,
            );
            Q_strlwr(&mut sModelName as *mut c_char);

            let model_name_str =
                std::ffi::CStr::from_ptr(&sModelName as *const c_char)
                    .to_string_lossy()
                    .into_owned();

            if let Some(cached_models) = (*CachedModels).get_mut(&model_name_str) {
                let ModelBin = cached_models;

                if ModelBin.pModelDiskImage.is_null() {
                    assert_eq!(0, 1); // should never happen, means that we're being called on a model that wasn't loaded
                } else {
                    let iNameOffset = (psShaderName as *const u8 as usize
                        - ModelBin.pModelDiskImage as *const u8 as usize)
                        as c_int;
                    let iPokeOffset =
                        (piShaderIndexPoke as *const u8 as usize
                            - ModelBin.pModelDiskImage as *const u8 as usize)
                            as c_int;

                    ModelBin
                        .ShaderRegisterData
                        .push((iNameOffset, iPokeOffset));
                }
            }
        }
    }

    static FakeGLAFile: &[u8] = &[
        0x32, 0x4C, 0x47, 0x41, 0x06, 0x00, 0x00, 0x00, 0x2A, 0x64, 0x65, 0x66, 0x61, 0x75, 0x6C,
        0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x80, 0x3F, 0x01, 0x00, 0x00, 0x00, 0x14, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x18,
        0x01, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x26, 0x01, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
        0x4D, 0x6F, 0x64, 0x56, 0x69, 0x65, 0x77, 0x20, 0x69, 0x6E, 0x74, 0x65, 0x72, 0x6E, 0x61,
        0x6C, 0x20, 0x64, 0x65, 0x66, 0x61, 0x75, 0x6C, 0x74, 0x00, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD,
        0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD,
        0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD, 0xCD,
        0xCD, 0xCD, 0xCD, 0xCD, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x80,
        0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFD, 0xBF, 0xFE, 0x7F,
        0xFE, 0x7F, 0xFE, 0x7F, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80,
    ];

    // returns qtrue if loaded, and sets the supplied qbool to true if it was from cache (instead of disk)
    //   (which we need to know to avoid LittleLong()ing everything again (well, the Mac needs to know anyway)...
    //
    fn RE_RegisterModels_GetDiskFile(
        psModelFileName: *const c_char,
        ppvBuffer: *mut *mut c_void,
        pqbAlreadyCached: *mut c_int,
    ) -> c_int {
        unsafe {
            let mut sModelName: [c_char; 260] = [0; 260]; // MAX_QPATH

            Q_strncpyz(
                &mut sModelName as *mut c_char,
                psModelFileName,
                260,
            );
            Q_strlwr(&mut sModelName as *mut c_char);

            let model_name_str =
                std::ffi::CStr::from_ptr(&sModelName as *const c_char)
                    .to_string_lossy()
                    .into_owned();

            if let Some(ModelBin) = (*CachedModels).get(&model_name_str) {
                if ModelBin.pModelDiskImage.is_null() {
                    // didn't have it cached, so try the disk...
                    //

                    // special case intercept first...
                    //
                    let default_gla_str = std::ffi::CStr::from_bytes_with_nul(b"default.gla.gla\0").unwrap();

                    if strcmp(psModelFileName, default_gla_str.as_ptr()) == 0 {
                        // return fake params as though it was found on disk...
                        //
                        let pvFakeGLAFile = Z_Malloc(FakeGLAFile.len() as c_int, 20, 0); // TAG_FILESYS, qfalse
                        memcpy(
                            pvFakeGLAFile,
                            FakeGLAFile.as_ptr() as *const c_void,
                            FakeGLAFile.len(),
                        );
                        *ppvBuffer = pvFakeGLAFile;
                        *pqbAlreadyCached = 0; // qfalse
                        return 1; // qtrue
                    }

                    FS_ReadFile(&sModelName as *const c_char, ppvBuffer);
                    *pqbAlreadyCached = 0; // qfalse

                    let bSuccess = if (*ppvBuffer).is_null() { 0 } else { 1 };

                    return bSuccess;
                } else {
                    *ppvBuffer = ModelBin.pModelDiskImage;
                    *pqbAlreadyCached = 1; // qtrue
                    return 1; // qtrue
                }
            }

            0
        }
    }

    // if return == true, no further action needed by the caller...
    //
    fn RE_RegisterModels_Malloc(
        iSize: c_int,
        pvDiskBufferIfJustLoaded: *mut c_void,
        psModelFileName: *const c_char,
        pqbAlreadyFound: *mut c_int,
        eTag: c_int,
    ) -> *mut c_void {
        unsafe {
            let mut sModelName: [c_char; 260] = [0; 260]; // MAX_QPATH

            Q_strncpyz(
                &mut sModelName as *mut c_char,
                psModelFileName,
                260,
            );
            Q_strlwr(&mut sModelName as *mut c_char);

            let model_name_str =
                std::ffi::CStr::from_ptr(&sModelName as *const c_char)
                    .to_string_lossy()
                    .into_owned();

            let models_ref = &mut *CachedModels;
            let ModelBin = models_ref
                .entry(model_name_str.clone())
                .or_insert_with(CachedEndianedModelBinary_s::new);

            if ModelBin.pModelDiskImage.is_null() {
                // ... then this entry has only just been created, ie we need to load it fully...
                //
                // new, instead of doing a Z_Malloc and assigning that we just morph the disk buffer alloc
                //	then don't thrown it away on return - cuts down on mem overhead
                //
                // ... groan, but not if doing a limb hierarchy creation (some VV stuff?), in which case it's NULL
                //
                let pvDiskBufferIfJustLoaded_mut = if !pvDiskBufferIfJustLoaded.is_null() {
                    Z_MorphMallocTag(pvDiskBufferIfJustLoaded, eTag);
                    pvDiskBufferIfJustLoaded
                } else {
                    Z_Malloc(iSize, eTag, 0) // qfalse
                };

                ModelBin.pModelDiskImage = pvDiskBufferIfJustLoaded_mut;
                ModelBin.iAllocSize = iSize;
                *pqbAlreadyFound = 0; // qfalse
            } else {
                // if we already had this model entry, then re-register all the shaders it wanted...
                //
                let iEntries = ModelBin.ShaderRegisterData.len();
                for i in 0..iEntries {
                    let iShaderNameOffset = ModelBin.ShaderRegisterData[i].0;
                    let iShaderPokeOffset = ModelBin.ShaderRegisterData[i].1;

                    let psShaderName = ((ModelBin.pModelDiskImage as *const u8).add(iShaderNameOffset as usize))
                        as *const c_char;
                    let piShaderPokePtr =
                        ((ModelBin.pModelDiskImage as *mut u8).add(iShaderPokeOffset as usize)) as *mut c_int;

                    let sh = R_FindShader(psShaderName, 0, 0, 1); // lightmapsNone, stylesDefault, qtrue

                    if (*sh).defaultShader != 0 {
                        *piShaderPokePtr = 0;
                    } else {
                        *piShaderPokePtr = (*sh).index;
                    }
                }
                *pqbAlreadyFound = 1; // qtrue - tell caller not to re-Endian or re-Shader this binary
            }

            ModelBin.iLastLevelUsedOn = RE_RegisterMedia_GetLevel();

            ModelBin.pModelDiskImage
        }
    }

    // dump any models not being used by this level if we're running low on memory...
    //
    fn GetModelDataAllocSize() -> c_int {
        unsafe {
            Z_MemSize(10) + // TAG_MODEL_MD3
            Z_MemSize(11) + // TAG_MODEL_GLM
            Z_MemSize(12)   // TAG_MODEL_GLA
        }
    }

    extern "C" {
        static r_modelpoolmegs: *mut c_void; // cvar_t
    }

    //
    // return qtrue if at least one cached model was freed (which tells z_malloc()-fail recovery code to try again)
    //
    static mut gbInsideRegisterModel: c_int = 0;

    fn RE_RegisterModels_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: c_int) -> c_int {
        unsafe {
            let mut bAtLeastoneModelFreed = 0; // qfalse

            if gbInsideRegisterModel != 0 {
                Com_DPrintf("(Inside RE_RegisterModel (z_malloc recovery?), exiting...\n\0".as_ptr() as *const c_char);
            } else {
                let iLoadedModelBytes = GetModelDataAllocSize();
                let iMaxModelBytes = if !r_modelpoolmegs.is_null() {
                    // assume integer field at some offset in cvar_t
                    let cvar_integer = *(r_modelpoolmegs as *const c_int);
                    cvar_integer * 1024 * 1024
                } else {
                    0
                };

                let models_ref = &mut *CachedModels;
                let keys_to_remove: Vec<String> = models_ref
                    .iter()
                    .filter_map(|(k, v)| {
                        let bDeleteThis = if bDeleteEverythingNotUsedThisLevel != 0 {
                            v.iLastLevelUsedOn != RE_RegisterMedia_GetLevel()
                        } else {
                            v.iLastLevelUsedOn < RE_RegisterMedia_GetLevel()
                        };

                        if bDeleteThis && !v.pModelDiskImage.is_null() {
                            Some(k.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                for key in keys_to_remove {
                    if let Some(CachedModel) = models_ref.get(&key) {
                        if !CachedModel.pModelDiskImage.is_null() {
                            Z_Free(CachedModel.pModelDiskImage);
                            bAtLeastoneModelFreed = 1; // qtrue
                        }
                    }
                    models_ref.remove(&key);
                }
            }

            bAtLeastoneModelFreed
        }
    }

    fn RE_RegisterModels_Info_f() {
        unsafe {
            let mut iTotalBytes = 0;
            if CachedModels.is_null() {
                Com_Printf(
                    "%d bytes total (%.2fMB)\n\0".as_ptr() as *const c_char,
                    iTotalBytes,
                );
                return;
            }

            let models_ref = &*CachedModels;
            let iModels = models_ref.len() as c_int;
            let mut iModel = 0;

            for (_, CachedModel) in models_ref.iter() {
                VID_Printf(
                    0, // PRINT_ALL
                    "%d/%d: \"%s\" (%d bytes)\0".as_ptr() as *const c_char,
                    iModel,
                    iModels,
                    CachedModel.pModelDiskImage as *const c_char,
                    CachedModel.iAllocSize,
                );

                // #ifdef _DEBUG
                VID_Printf(
                    0, // PRINT_ALL
                    ", lvl %d\n\0".as_ptr() as *const c_char,
                    CachedModel.iLastLevelUsedOn,
                );
                // #endif

                iTotalBytes += CachedModel.iAllocSize;
                iModel += 1;
            }

            VID_Printf(
                0, // PRINT_ALL
                "%d bytes total (%.2fMB)\n\0".as_ptr() as *const c_char,
                iTotalBytes,
            );
        }
    }

    fn RE_RegisterModels_DeleteAll() {
        unsafe {
            if CachedModels.is_null() {
                return;
            }

            let models_ref = &mut *CachedModels;
            let keys: Vec<String> = models_ref.keys().cloned().collect();

            for key in keys {
                if let Some(CachedModel) = models_ref.get(&key) {
                    if !CachedModel.pModelDiskImage.is_null() {
                        Z_Free(CachedModel.pModelDiskImage);
                    }
                }
                models_ref.remove(&key);
            }

            RE_AnimationCFGs_DeleteAll();
        }
    }

    static mut giRegisterMedia_CurrentLevel: c_int = 0;
    static mut gbAllowScreenDissolve: c_int = 1; // qtrue

    //
    // param "bAllowScreenDissolve" is just a convenient way of getting hold of a bool which can be checked by the code that
    //	issues the InitDissolve command later in RE_RegisterMedia_LevelLoadEnd()
    //
    static mut sPrevMapName: [c_char; 260] = [0; 260]; // MAX_QPATH

    fn RE_RegisterMedia_LevelLoadBegin(
        psMapName: *const c_char,
        eForceReload: c_int,
        bAllowScreenDissolve: c_int,
    ) {
        unsafe {
            gbAllowScreenDissolve = bAllowScreenDissolve;

            // tr.numBSPModels = 0;
            // (assuming tr is a global, would need to set it here)

            // for development purposes we may want to ditch certain media just before loading a map...
            //
            match eForceReload {
                0 => { // eForceReload_BSP
                    CM_DeleteCachedMap(1); // qtrue
                    R_Images_DeleteLightMaps();
                }
                1 => { // eForceReload_MODELS
                    RE_RegisterModels_DeleteAll();
                }
                2 => { // eForceReload_ALL
                    // BSP...
                    //
                    CM_DeleteCachedMap(1); // qtrue
                    R_Images_DeleteLightMaps();
                    //
                    // models...
                    //
                    RE_RegisterModels_DeleteAll();
                }
                _ => {}
            }

            // at some stage I'll probably want to put some special logic here, like not incrementing the level number
            //	when going into a map like "brig" or something, so returning to the previous level doesn't require an
            //	asset reload etc, but for now...
            //
            // only bump level number if we're not on the same level.
            //	Note that this will hide uncached models, which is perhaps a bad thing?...
            //
            if Q_stricmp(psMapName, addr_of_mut!(sPrevMapName).cast()) != 0 {
                Q_strncpyz(addr_of_mut!(sPrevMapName).cast(), psMapName, 260);
                giRegisterMedia_CurrentLevel += 1;
            }
        }
    }

    fn RE_RegisterMedia_GetLevel() -> c_int {
        unsafe { giRegisterMedia_CurrentLevel }
    }

    extern "C" {
        fn SND_RegisterAudio_LevelLoadEnd(bDeleteEverythingNotUsedThisLevel: c_int) -> c_int;
    }

    fn RE_RegisterMedia_LevelLoadEnd() {
        unsafe {
            RE_RegisterModels_LevelLoadEnd(0); // qfalse
            RE_RegisterImages_LevelLoadEnd();
            SND_RegisterAudio_LevelLoadEnd(0); // qfalse

            if gbAllowScreenDissolve != 0 {
                RE_InitDissolve(0); // qfalse
            }

            S_RestartMusic();

            // extern qboolean gbAlreadyDoingLoad;
            // gbAlreadyDoingLoad = qfalse;
        }
    }

    /*
    ** R_GetModelByHandle
    */
    fn R_GetModelByHandle(index: c_int) -> *mut model_t {
        unsafe {
            // out of range gets the default model
            if index < 1 || index >= tr_numModels {
                return tr_models[0];
            }

            tr_models[index as usize]
        }
    }

    //===============================================================================

    /*
    ** R_AllocModel
    */
    fn R_AllocModel() -> *mut model_t {
        unsafe {
            if tr_numModels == 4096 { // MAX_MOD_KNOWN
                return null_mut();
            }

            let mod_ = Hunk_Alloc(
                std::mem::size_of::<model_t>() as c_int,
                1, // qtrue
            ) as *mut model_t;

            (*mod_).index = tr_numModels;
            tr_models[tr_numModels as usize] = mod_;
            tr_numModels += 1;

            mod_
        }
    }

    /*
    Ghoul2 Insert Start
    */

    /*
    ================
    return a hash value for the filename
    ================
    */
    fn generateHashValue(fname: *const c_char, size: c_int) -> c_int {
        unsafe {
            let mut i = 0;
            let mut hash = 0i64;

            while *fname.add(i) != 0 {
                let letter = tolower(*fname.add(i) as u8 as c_int) as u8 as c_char;
                if letter as u8 == b'.' {
                    break; // don't include extension
                }
                if letter as u8 == b'\\' {
                    hash += '/' as i64 * (i as i64 + 119);
                } else {
                    hash += (letter as i64) * (i as i64 + 119);
                }
                i += 1;
            }
            (hash & ((size - 1) as i64)) as c_int
        }
    }

    fn RE_InsertModelIntoHash(name: *const c_char, mod_: *mut model_t) {
        unsafe {
            let hash = generateHashValue(name, FILE_HASH_SIZE as c_int) as usize;

            // insert this file into the hash table so we can look it up faster later
            let mh = Hunk_Alloc(std::mem::size_of::<modelHash_t>() as c_int, 1) // qtrue
                as *mut modelHash_t;

            (*mh).next = *addr_of_mut!(mhHashTable[hash]);
            (*mh).handle = (*mod_).index;
            strcpy(&mut (*mh).name as *mut c_char, name);
            *addr_of_mut!(mhHashTable[hash]) = mh;
        }
    }
    /*
    Ghoul2 Insert End
    */

    /*
    ====================
    RE_RegisterModel

    Loads in a model for the given name

    Zero will be returned if the model fails to load.
    An entry will be retained for failed models as an
    optimization to prevent disk rescanning if they are
    asked for again.
    ====================
    */
    fn RE_RegisterModel_Actual(name: *const c_char) -> c_int {
        unsafe {
            let mut mod_: *mut model_t;
            let mut buf: *mut u32;
            let mut ident: u32;
            let mut loaded: c_int;
            let mut numLoaded: c_int;

            if name.is_null() || *name == 0 {
                VID_Printf(1, "RE_RegisterModel: NULL name\n\0".as_ptr() as *const c_char); // PRINT_WARNING
                return 0;
            }

            if strlen(name) >= 260 { // MAX_QPATH
                VID_Printf(2, "Model name exceeds MAX_QPATH\n\0".as_ptr() as *const c_char); // PRINT_DEVELOPER
                return 0;
            }

            /*
            Ghoul2 Insert Start
            */
            //	if (!tr.registered) {
            //		VID_Printf( PRINT_WARNING, "RE_RegisterModel (%s) called before ready!\n",name );
            //		return 0;
            //	}
            //
            // search the currently loaded models
            //

            let hash_val = generateHashValue(name, FILE_HASH_SIZE as c_int);

            //
            // see if the model is already loaded
            //
            let mut mh_iter = *addr_of_mut!(mhHashTable[hash_val as usize]);
            while !mh_iter.is_null() {
                if Q_stricmp(addr_of!((*mh_iter).name).cast(), name) == 0 {
                    if (*tr_models[(*mh_iter).handle as usize]).model_type == 0 { // MOD_BAD
                        return 0;
                    }
                    return (*mh_iter).handle;
                }
                mh_iter = (*mh_iter).next;
            }

            /*
            Ghoul2 Insert End
            */

            if *name as u8 == b'#' {
                let mut temp: [c_char; 260] = [0; 260]; // MAX_QPATH

                // tr.numBSPModels++;
                // #ifndef DEDICATED
                // RE_LoadWorldMap_Actual(va("maps/%s.bsp", name + 1), tr.bspModels[tr.numBSPModels - 1], tr.numBSPModels);
                // #endif
                // Com_sprintf(temp, MAX_QPATH, "*%d-0", tr.numBSPModels);
                let hash_map = generateHashValue(&temp as *const c_char, FILE_HASH_SIZE as c_int);
                let mut mh_search = *addr_of_mut!(mhHashTable[hash_map as usize]);
                while !mh_search.is_null() {
                    if Q_stricmp(addr_of!((*mh_search).name).cast(), &temp as *const c_char) == 0 {
                        return (*mh_search).handle;
                    }
                    mh_search = (*mh_search).next;
                }

                return 0;
            }

            // allocate a new model_t

            mod_ = R_AllocModel();
            if mod_.is_null() {
                VID_Printf(1, "RE_RegisterModel: R_AllocModel() failed for '%s'\n\0".as_ptr() as *const c_char, name); // PRINT_WARNING
                return 0;
            }

            // only set the name after the model has been successfully loaded
            Q_strncpyz(addr_of_mut!((*mod_).name).cast(), name, 260); // sizeof( mod->name )

            // make sure the render thread is stopped
            //R_SyncRenderThread();

            let iLODStart = if strstr(name, ".md3\0".as_ptr() as *const c_char).is_null() {
                0
            } else {
                3 // MD3_MAX_LODS-1 (this loads the md3s in reverse so they can be biased)
            };
            (*mod_).numLods = 0;

            //
            // load the files
            //
            numLoaded = 0;

            let mut lod_iter = iLODStart;
            let mut fail_load = false;

            while lod_iter >= 0 && !fail_load {
                let mut filename: [c_char; 1024] = [0; 1024];

                strcpy(&mut filename as *mut c_char, name);

                if lod_iter != 0 {
                    let mut namebuf: [c_char; 80] = [0; 80];

                    if !strrchr(&filename as *const c_char, b'.' as c_int).is_null() {
                        *strrchr(&filename as *const c_char, b'.' as c_int) = 0;
                    }
                    sprintf(&mut namebuf as *mut c_char, "_%d.md3\0".as_ptr() as *const c_char, lod_iter);
                    strcat(&mut filename as *mut c_char, &namebuf as *const c_char);
                }

                let mut bAlreadyCached = 0; // qfalse
                if RE_RegisterModels_GetDiskFile(
                    &filename as *const c_char,
                    &mut buf as *mut *mut c_void as *mut *mut u32,
                    &mut bAlreadyCached,
                ) == 0
                {
                    if numLoaded != 0 {
                        //we loaded one already, but a higher LOD is missing!
                        Com_Error(
                            2, // ERR_DROP
                            "R_LoadMD3: %s has LOD %d but is missing LOD %d ('%s')!\0".as_ptr()
                                as *const c_char,
                            addr_of!((*mod_).name) as *const c_char,
                            lod_iter + 1,
                            lod_iter,
                            &filename as *const c_char,
                        );
                    }
                    lod_iter -= 1;
                    continue;
                }

                //loadmodel = mod_; // this seems to be fairly pointless

                // important that from now on we pass 'filename' instead of 'name' to all model load functions,
                //	because 'filename' accounts for any LOD mangling etc so guarantees unique lookups for yet more
                //	internal caching...
                //
                ident = *(buf as *const u32);
                if bAlreadyCached == 0 {
                    ident = LittleLong(ident);
                }

                match ident {
                    // if you add any new types of model load in this switch-case, tell me,
                    //	or copy what I've done with the cache scheme (-ste).
                    //
                    0x4158444D => {
                        // MDXA_IDENT
                        loaded = R_LoadMDXA(
                            mod_,
                            buf as *mut c_void,
                            &filename as *const c_char,
                            bAlreadyCached,
                        );
                    }
                    0x4D58444D => {
                        // MDXM_IDENT
                        loaded = R_LoadMDXM(
                            mod_,
                            buf as *mut c_void,
                            &filename as *const c_char,
                            bAlreadyCached,
                        );
                    }
                    0x3344444D => {
                        // MD3_IDENT
                        loaded = R_LoadMD3(
                            mod_,
                            lod_iter,
                            buf as *mut c_void,
                            &filename as *const c_char,
                            &mut bAlreadyCached,
                        );
                    }
                    _ => {
                        VID_Printf(
                            1,
                            "RE_RegisterModel: unknown fileid for %s\n\0".as_ptr()
                                as *const c_char,
                            &filename as *const c_char,
                        );
                        // PRINT_WARNING
                        fail_load = true;
                        break;
                    }
                }

                if bAlreadyCached == 0 {
                    // important to check!!
                    FS_FreeFile(buf as *mut c_void);
                }

                if loaded == 0 {
                    if lod_iter == 0 {
                        VID_Printf(
                            1,
                            "RE_RegisterModel: cannot load %s\n\0".as_ptr() as *const c_char,
                            &filename as *const c_char,
                        );
                        // PRINT_WARNING
                        fail_load = true;
                        break;
                    } else {
                        break;
                    }
                } else {
                    (*mod_).numLods += 1;
                    numLoaded += 1;
                    // if we have a valid model and are biased
                    // so that we won't see any higher detail ones,
                    // stop loading them
                    if lod_iter <= r_lodbias {
                        // ->integer
                        break;
                    }
                }

                lod_iter -= 1;
            }

            if numLoaded != 0 && !fail_load {
                // duplicate into higher lod spots that weren't
                // loaded, in case the user changes r_lodbias on the fly
                let mut lod_dup = lod_iter - 1;
                while lod_dup >= 0 {
                    (*mod_).numLods += 1;
                    (*mod_).md3[lod_dup as usize] = (*mod_).md3[(lod_dup + 1) as usize];
                    lod_dup -= 1;
                }
                /*
                Ghoul2 Insert Start
                */

                RE_InsertModelIntoHash(name, mod_);
                return (*mod_).index;
                /*
                Ghoul2 Insert End
                */
            }

            // fail: we still keep the model_t around, so if the model name is asked for
            // again, we won't bother scanning the filesystem
            (*mod_).model_type = 0; // MOD_BAD
            RE_InsertModelIntoHash(name, mod_);
            0
        }
    }

    // wrapper function needed to avoid problems with mid-function returns so I can safely use this bool to tell the
    //	z_malloc-fail recovery code whether it's safe to ditch any model caches...
    //
    fn RE_RegisterModel(name: *const c_char) -> c_int {
        unsafe {
            gbInsideRegisterModel = 1; // qtrue

            let q = RE_RegisterModel_Actual(name);

            let name_len = strlen(name);
            if name_len >= 4 {
                let ext_ptr = name.add(name_len - 4);
                if stricmp(ext_ptr, ".gla\0".as_ptr() as *const c_char) != 0 {
                    gbInsideRegisterModel = 0; // qfalse
                }
            }

            q
        }
    }

    /*
    =================
    R_LoadMD3
    =================
    */
    fn R_LoadMD3(
        mod_: *mut model_t,
        lod: c_int,
        buffer: *mut c_void,
        mod_name: *const c_char,
        bAlreadyCached: *mut c_int,
    ) -> c_int {
        unsafe {
            let pinmodel = buffer as *mut md3Header_t;
            //
            // read some fields from the binary, but only LittleLong() them when we know this wasn't an already-cached model...
            //
            let mut version = (*pinmodel).version;
            let mut size = (*pinmodel).ofsEnd;

            if *bAlreadyCached == 0 {
                version = LittleLong(version);
                size = LittleLong(size);
            }

            if version != 15 { // MD3_VERSION
                VID_Printf(
                    1,
                    "R_LoadMD3: %s has wrong version (%i should be %i)\n\0".as_ptr()
                        as *const c_char,
                    mod_name,
                    version,
                    15, // MD3_VERSION
                );
                return 0; // qfalse
            }

            (*mod_).model_type = 2; // MOD_MESH
            (*mod_).dataSize += size;

            let mut bAlreadyFound = 0; // qfalse
            (*mod_).md3[lod as usize] = RE_RegisterModels_Malloc(
                size,
                buffer,
                mod_name,
                &mut bAlreadyFound,
                10, // TAG_MODEL_MD3
            ) as *mut md3Header_t;

            assert_eq!(*bAlreadyCached, bAlreadyFound);

            if bAlreadyFound == 0 {
                // horrible new hackery, if !bAlreadyFound then we've just done a tag-morph, so we need to set the
                //	bool reference passed into this function to true, to tell the caller NOT to do an FS_Freefile since
                //	we've hijacked that memory block...
                //
                // Aaaargh. Kill me now...
                //
                *bAlreadyCached = 1; // qtrue
                assert!((*mod_).md3[lod as usize] as *const u8 == buffer as *const u8);
                //		memcpy( mod->md3[lod], buffer, size );	// and don't do this now, since it's the same thing

                LL!((*(*mod_).md3[lod as usize]).ident);
                LL!((*(*mod_).md3[lod as usize]).version);
                LL!((*(*mod_).md3[lod as usize]).numFrames);
                LL!((*(*mod_).md3[lod as usize]).numTags);
                LL!((*(*mod_).md3[lod as usize]).numSurfaces);
                LL!((*(*mod_).md3[lod as usize]).ofsFrames);
                LL!((*(*mod_).md3[lod as usize]).ofsTags);
                LL!((*(*mod_).md3[lod as usize]).ofsSurfaces);
                LL!((*(*mod_).md3[lod as usize]).ofsEnd);
            }

            if (*(*mod_).md3[lod as usize]).numFrames < 1 {
                VID_Printf(
                    1,
                    "R_LoadMD3: %s has no frames\n\0".as_ptr() as *const c_char,
                    mod_name,
                );
                return 0; // qfalse
            }

            if bAlreadyFound != 0 {
                return 1; // qtrue - All done. Stop, go no further, do not pass Go...
            }

            // #ifndef _M_IX86
            //	//
            //	// optimisation, we don't bother doing this for standard intel case since our data's already in that format...
            //	//
            //
            //	// swap all the frames
            //    frame = (md3Frame_t *) ( (byte *)mod->md3[lod] + mod->md3[lod]->ofsFrames );
            //    for ( i = 0 ; i < mod->md3[lod]->numFrames ; i++, frame++) {
            //    	frame->radius = LittleFloat( frame->radius );
            //        for ( j = 0 ; j < 3 ; j++ ) {
            //            frame->bounds[0][j] = LittleFloat( frame->bounds[0][j] );
            //            frame->bounds[1][j] = LittleFloat( frame->bounds[1][j] );
            //	    	frame->localOrigin[j] = LittleFloat( frame->localOrigin[j] );
            //        }
            //	}
            //
            //	// swap all the tags
            //    tag = (md3Tag_t *) ( (byte *)mod->md3[lod] + mod->md3[lod]->ofsTags );
            //    for ( i = 0 ; i < mod->md3[lod]->numTags * mod->md3[lod]->numFrames ; i++, tag++) {
            //        for ( j = 0 ; j < 3 ; j++ ) {
            //			tag->origin[j] = LittleFloat( tag->origin[j] );
            //			tag->axis[0][j] = LittleFloat( tag->axis[0][j] );
            //			tag->axis[1][j] = LittleFloat( tag->axis[1][j] );
            //			tag->axis[2][j] = LittleFloat( tag->axis[2][j] );
            //        }
            //	}
            // #endif

            // swap all the surfaces
            let mut surf = ((*(*mod_).md3[lod as usize]).ofsSurfaces as *const u8).add((*mod_).md3[lod as usize] as usize) as *mut md3Surface_t;
            let mut i = 0;
            while i < (*(*mod_).md3[lod as usize]).numSurfaces {
                LL!((*surf).flags);
                LL!((*surf).numFrames);
                LL!((*surf).numShaders);
                LL!((*surf).numTriangles);
                LL!((*surf).ofsTriangles);
                LL!((*surf).numVerts);
                LL!((*surf).ofsShaders);
                LL!((*surf).ofsSt);
                LL!((*surf).ofsXyzNormals);
                LL!((*surf).ofsEnd);

                if (*surf).numVerts > 4000 { // SHADER_MAX_VERTEXES
                    Com_Error(
                        2, // ERR_DROP
                        "R_LoadMD3: %s has more than %i verts on a surface (%i)\0".as_ptr()
                            as *const c_char,
                        mod_name,
                        4000, // SHADER_MAX_VERTEXES
                        (*surf).numVerts,
                    );
                }
                if (*surf).numTriangles * 3 > 10000 { // SHADER_MAX_INDEXES
                    Com_Error(
                        2, // ERR_DROP
                        "R_LoadMD3: %s has more than %i triangles on a surface (%i)\0".as_ptr()
                            as *const c_char,
                        mod_name,
                        10000 / 3, // SHADER_MAX_INDEXES / 3
                        (*surf).numTriangles,
                    );
                }

                // change to surface identifier
                (*surf).ident = 1; // SF_MD3

                // lowercase the surface name so skin compares are faster
                Q_strlwr(addr_of_mut!((*surf).name).cast());

                // strip off a trailing _1 or _2
                // this is a crutch for q3data being a mess
                let mut j = strlen(addr_of!((*surf).name).cast());
                if j > 2 && (*addr_of!((*surf).name).offset(j as isize - 2)) as u8 == b'_' {
                    *addr_of_mut!((*surf).name).offset(j as isize - 2) = 0;
                }

                // register the shaders
                let mut shader = ((*surf).ofsShaders as *const u8).add(surf as usize) as *mut md3Shader_t;
                j = 0;
                while j < (*surf).numShaders {
                    let sh = R_FindShader(
                        addr_of!((*shader).name).cast(),
                        0, // lightmapsNone
                        0, // stylesDefault
                        1, // qtrue
                    );
                    if (*sh).defaultShader != 0 {
                        (*shader).shaderIndex = 0;
                    } else {
                        (*shader).shaderIndex = (*sh).index;
                    }
                    RE_RegisterModels_StoreShaderRequest(
                        mod_name,
                        addr_of!((*shader).name).cast(),
                        addr_of!((*shader).shaderIndex).cast(),
                    );
                    shader = shader.add(1);
                    j += 1;
                }

                // #ifndef _M_IX86
                //	//
                //	// optimisation, we don't bother doing this for standard intel case since our data's already in that format...
                //	//
                //
                //		// swap all the triangles
                //		tri = (md3Triangle_t *) ( (byte *)surf + surf->ofsTriangles );
                //		for ( j = 0 ; j < surf->numTriangles ; j++, tri++ ) {
                //			LL(tri->indexes[0]);
                //			LL(tri->indexes[1]);
                //			LL(tri->indexes[2]);
                //		}
                //
                //		// swap all the ST
                //        st = (md3St_t *) ( (byte *)surf + surf->ofsSt );
                //        for ( j = 0 ; j < surf->numVerts ; j++, st++ ) {
                //            st->st[0] = LittleFloat( st->st[0] );
                //            st->st[1] = LittleFloat( st->st[1] );
                //        }
                //
                //		// swap all the XyzNormals
                //        xyz = (md3XyzNormal_t *) ( (byte *)surf + surf->ofsXyzNormals );
                //        for ( j = 0 ; j < surf->numVerts * surf->numFrames ; j++, xyz++ )
                //		{
                //            xyz->xyz[0] = LittleShort( xyz->xyz[0] );
                //            xyz->xyz[1] = LittleShort( xyz->xyz[1] );
                //            xyz->xyz[2] = LittleShort( xyz->xyz[2] );
                //
                //            xyz->normal = LittleShort( xyz->normal );
                //        }
                // #endif

                // find the next surface
                surf = ((*surf).ofsEnd as *const u8).add(surf as usize) as *mut md3Surface_t;
                i += 1;
            }

            1 // qtrue
        }
    }

    //=============================================================================

    extern "C" {
        fn ShaderTableCleanup();
        fn CM_LoadShaderText(forceReload: c_int);
        fn CM_SetupShaderProperties();
    }

    /*
    ** RE_BeginRegistration
    */
    fn RE_BeginRegistration(glconfigOut: *mut glconfig_t) {
        unsafe {
            // #ifndef _XBOX
            ShaderTableCleanup();
            // #endif
            Hunk_ClearToMark();

            R_Init();
            *glconfigOut = glConfig;

            // tr.viewCluster = -1;		// force markleafs to regenerate
            // RE_ClearScene();
            // tr.registered = qtrue;

            R_SyncRenderThread();
        }
    }

    //=============================================================================

    /*
    ===============
    R_ModelInit
    ===============
    */
    fn R_ModelInit() {
        unsafe {
            // #ifdef _XBOX
            //	// Sorry Raven, but static maps == fragmentation
            //	if (!CachedModels)
            //	{
            //		CachedModels = new CachedModels_t;
            //	}
            // #else
            if CachedModels.is_null() {
                let cached = Box::into_raw(Box::new(HashMap::new()));
                CachedModels = cached;
            }
            // #endif

            // leave a space for NULL model
            tr_numModels = 0;

            let mod_ = R_AllocModel();
            (*mod_).model_type = 0; // MOD_BAD
            /*
            Ghoul2 Insert Start
            */

            for i in 0..FILE_HASH_SIZE {
                *addr_of_mut!(mhHashTable[i]) = null_mut();
            }
            /*
            Ghoul2 Insert End
            */
        }
    }

    /*
    ================
    R_Modellist_f
    ================
    */
    fn R_Modellist_f() {
        unsafe {
            let mut total = 0;
            for i in 1..tr_numModels as usize {
                let mod_ = tr_models[i];
                match (*mod_).model_type {
                    0 => {
                        // MOD_BAD
                        VID_Printf(
                            0, // PRINT_ALL
                            "MOD_BAD  :      %s\n\0".as_ptr() as *const c_char,
                            addr_of!((*mod_).name) as *const c_char,
                        );
                    }
                    1 => {
                        // MOD_BRUSH
                        VID_Printf(
                            0, // PRINT_ALL
                            "%8i : (%i) %s\n\0".as_ptr() as *const c_char,
                            (*mod_).dataSize,
                            (*mod_).numLods,
                            addr_of!((*mod_).name) as *const c_char,
                        );
                    }
                    3 => {
                        // MOD_MDXA
                        VID_Printf(
                            0, // PRINT_ALL
                            "%8i : (%i) %s\n\0".as_ptr() as *const c_char,
                            (*mod_).dataSize,
                            (*mod_).numLods,
                            addr_of!((*mod_).name) as *const c_char,
                        );
                    }
                    4 => {
                        // MOD_MDXM
                        VID_Printf(
                            0, // PRINT_ALL
                            "%8i : (%i) %s\n\0".as_ptr() as *const c_char,
                            (*mod_).dataSize,
                            (*mod_).numLods,
                            addr_of!((*mod_).name) as *const c_char,
                        );
                    }
                    2 => {
                        // MOD_MESH
                        let mut lods = 1;
                        for j in 1..4 { // MD3_MAX_LODS
                            if !(*mod_).md3[j].is_null() && (*mod_).md3[j] != (*mod_).md3[j - 1] {
                                lods += 1;
                            }
                        }
                        VID_Printf(
                            0, // PRINT_ALL
                            "%8i : (%i) %s\n\0".as_ptr() as *const c_char,
                            (*mod_).dataSize,
                            lods,
                            addr_of!((*mod_).name) as *const c_char,
                        );
                    }
                    _ => {
                        assert!(false);
                        VID_Printf(
                            0, // PRINT_ALL
                            "UNKNOWN  :      %s\n\0".as_ptr() as *const c_char,
                            addr_of!((*mod_).name) as *const c_char,
                        );
                    }
                }
                total += (*mod_).dataSize;
            }
            VID_Printf(
                0, // PRINT_ALL
                "%8i : Total models\n\0".as_ptr() as *const c_char,
                total,
            );

            /*	this doesn't work with the new hunks
            	if ( tr.world ) {
            		VID_Printf( PRINT_ALL, "%8i : %s\n", tr.world->dataSize, tr.world->name );
            	} */
        }
    }

    //=============================================================================

    /*
    ================
    R_GetTag for MD3s
    ================
    */
    fn R_GetTag(mod_: *mut md3Header_t, mut frame: c_int, tagName: *const c_char) -> *mut md3Tag_t {
        unsafe {
            if frame >= (*mod_).numFrames {
                // it is possible to have a bad frame while changing models, so don't error
                frame = (*mod_).numFrames - 1;
            }

            let mut tag = (((*mod_).ofsTags as *const u8).add(mod_ as usize) as *const md3Tag_t)
                .add((frame * (*mod_).numTags) as usize) as *mut md3Tag_t;
            for _i in 0..(*mod_).numTags {
                if strcmp(addr_of!((*tag).name).cast(), tagName) == 0 {
                    return tag; // found it
                }
                tag = tag.add(1);
            }

            null_mut()
        }
    }

    /*
    ================
    R_LerpTag
    ================
    */
    fn R_LerpTag(
        tag: *mut orientation_t,
        handle: c_int,
        startFrame: c_int,
        endFrame: c_int,
        frac: f32,
        tagName: *const c_char,
    ) {
        unsafe {
            let model = R_GetModelByHandle(handle);
            let start: *mut md3Tag_t;
            let finish: *mut md3Tag_t;

            if !(*model).md3[0].is_null() {
                start = R_GetTag((*model).md3[0], startFrame, tagName);
                finish = R_GetTag((*model).md3[0], endFrame, tagName);
            } else {
                AxisClear(addr_of_mut!((*tag).axis).cast());
                VectorClear(addr_of_mut!((*tag).origin).cast());
                return;
            }

            if start.is_null() || finish.is_null() {
                AxisClear(addr_of_mut!((*tag).axis).cast());
                VectorClear(addr_of_mut!((*tag).origin).cast());
                return;
            }

            let frontLerp = frac;
            let backLerp = 1.0 - frac;

            for i in 0..3 {
                (*tag).origin[i] = (*start).origin[i] * backLerp + (*finish).origin[i] * frontLerp;
                (*tag).axis[0][i] = (*start).axis[0][i] * backLerp + (*finish).axis[0][i] * frontLerp;
                (*tag).axis[1][i] = (*start).axis[1][i] * backLerp + (*finish).axis[1][i] * frontLerp;
                (*tag).axis[2][i] = (*start).axis[2][i] * backLerp + (*finish).axis[2][i] * frontLerp;
            }
            VectorNormalize(addr_of_mut!((*tag).axis[0]).cast());
            VectorNormalize(addr_of_mut!((*tag).axis[1]).cast());
            VectorNormalize(addr_of_mut!((*tag).axis[2]).cast());
        }
    }

    /*
    ====================
    R_ModelBounds
    ====================
    */
    fn R_ModelBounds(handle: c_int, mins: *mut [f32; 3], maxs: *mut [f32; 3]) {
        unsafe {
            let model = R_GetModelByHandle(handle);

            if !(*model).bmodel.is_null() {
                VectorCopy(
                    addr_of!((*(*model).bmodel).bounds[0]).cast(),
                    mins as *mut c_void,
                );
                VectorCopy(
                    addr_of!((*(*model).bmodel).bounds[1]).cast(),
                    maxs as *mut c_void,
                );
                return;
            }

            if !(*model).md3[0].is_null() {
                let header = (*model).md3[0];
                let frame = ((*header).ofsFrames as *const u8).add(header as usize) as *mut md3Frame_t;

                VectorCopy(
                    addr_of!((*frame).bounds[0]).cast(),
                    mins as *mut c_void,
                );
                VectorCopy(
                    addr_of!((*frame).bounds[1]).cast(),
                    maxs as *mut c_void,
                );
            } else {
                VectorClear(mins as *mut c_void);
                VectorClear(maxs as *mut c_void);
                return;
            }
        }
    }

    // #ifdef _XBOX
    // void R_ModelFree(void)
    // {
    //	if (CachedModels)
    //	{
    //		RE_RegisterModels_DeleteAll();
    //		delete CachedModels;
    //		CachedModels = NULL;
    //	}
    // }
    // #endif

    // ==================== STUB IMPLEMENTATIONS ====================

    // Forward declared functions that need stubs for structural coherence

    // Stubs for external globals
    static mut tr_numModels: c_int = 0;
    static mut tr_models: [*mut model_t; 4096] = [null_mut(); 4096]; // MAX_MOD_KNOWN
    static mut r_lodbias: c_int = 0;
    static glConfig: glconfig_t = glconfig_t { _dummy: [0; 1] };
    static sDEFAULT_GLA_NAME: &[u8] = b"default.gla\0";

    extern "C" {

        fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
        fn Q_strlwr(string: *mut c_char);
        fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
        fn Z_Malloc(size: c_int, tag: c_int, zero: c_int) -> *mut c_void;
        fn Z_Free(ptr: *mut c_void);
        fn Z_MorphMallocTag(ptr: *mut c_void, tag: c_int);
        fn Z_MemSize(tag: c_int) -> c_int;
        fn FS_ReadFile(name: *const c_char, buffer: *mut *mut c_void) -> c_int;
        fn FS_FreeFile(buffer: *mut c_void);
        fn VID_Printf(level: c_int, format: *const c_char, ...);
        fn Com_Printf(format: *const c_char, ...);
        fn Com_DPrintf(format: *const c_char, ...);
        fn Com_Error(level: c_int, format: *const c_char, ...);
        fn Com_sprintf(dest: *mut c_char, destsize: c_int, format: *const c_char, ...);
        fn R_FindShader(
            name: *const c_char,
            lightmapindex: c_int,
            stylesindex: c_int,
            force: c_int,
        ) -> *mut shader_t;
        fn R_LoadMDXA(
            mod_: *mut model_t,
            buf: *mut c_void,
            filename: *const c_char,
            bAlreadyCached: c_int,
        ) -> c_int;
        fn R_LoadMDXM(
            mod_: *mut model_t,
            buf: *mut c_void,
            filename: *const c_char,
            bAlreadyCached: c_int,
        ) -> c_int;
        fn CM_DeleteCachedMap(all: c_int);
        fn R_Images_DeleteLightMaps();
        fn R_Images_LevelLoadEnd();
        fn RE_InitDissolve(refract: c_int);
        fn S_RestartMusic();
        fn RE_AnimationCFGs_DeleteAll();
        fn R_Init();
        fn R_SyncRenderThread();
        fn Hunk_Alloc(size: c_int, zero: c_int) -> *mut c_void;
        fn Hunk_ClearToMark();
        fn LittleLong(l: u32) -> u32;
        fn LittleFloat(f: f32) -> f32;
        fn LittleShort(s: u16) -> u16;
        fn tolower(c: c_int) -> c_int;
        fn strlen(s: *const c_char) -> usize;
        fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
        fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
        fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
        fn strstr(s1: *const c_char, s2: *const c_char) -> *mut c_char;
        fn strrchr(s: *const c_char, c: c_int) -> *mut c_char;
        fn sprintf(str: *mut c_char, format: *const c_char, ...) -> c_int;
        fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
        fn AxisClear(a: *mut c_void);
        fn VectorClear(a: *mut c_void);
        fn VectorCopy(a: *const c_void, b: *mut c_void);
        fn VectorNormalize(v: *mut c_void);
        fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

        fn RE_RegisterImages_LevelLoadEnd();
    }

    #[repr(C)]
    struct model_t {
        name: [c_char; 260], // MAX_QPATH
        model_type: c_int,
        index: c_int,
        dataSize: c_int,
        bmodel: *mut c_void, // bmodel_t
        md3: [*mut md3Header_t; 4], // MD3_MAX_LODS
        numLods: c_int,
        // more fields...
    }

    #[repr(C)]
    struct md3Header_t {
        ident: u32,
        version: u32,
        name: [c_char; 260], // MAX_QPATH
        flags: u32,
        numFrames: c_int,
        numTags: c_int,
        numSurfaces: c_int,
        numSkins: c_int,
        ofsFrames: u32,
        ofsTags: u32,
        ofsSurfaces: u32,
        ofsEnd: u32,
    }

    #[repr(C)]
    struct md3Frame_t {
        bounds: [[f32; 3]; 2],
        localOrigin: [f32; 3],
        radius: f32,
        name: [c_char; 16],
    }

    #[repr(C)]
    struct md3Surface_t {
        ident: u32,
        name: [c_char; 260], // MAX_QPATH
        flags: u32,
        numFrames: c_int,
        numShaders: c_int,
        numVerts: c_int,
        numTriangles: c_int,
        ofsTriangles: u32,
        ofsShaders: u32,
        ofsSt: u32,
        ofsXyzNormals: u32,
        ofsEnd: u32,
    }

    #[repr(C)]
    struct md3Shader_t {
        name: [c_char; 260], // MAX_QPATH
        shaderIndex: c_int,
    }

    #[repr(C)]
    struct md3Triangle_t {
        indexes: [c_int; 3],
    }

    #[repr(C)]
    struct md3St_t {
        st: [f32; 2],
    }

    #[repr(C)]
    struct md3XyzNormal_t {
        xyz: [i16; 3],
        normal: i16,
    }

    #[repr(C)]
    struct md3Tag_t {
        name: [c_char; 260], // MAX_QPATH
        origin: [f32; 3],
        axis: [[f32; 3]; 3],
    }

    #[repr(C)]
    struct shader_t {
        name: [c_char; 260], // MAX_QPATH
        lightmapindex: c_int,
        // ... more fields
        defaultShader: c_int,
        index: c_int,
        // ... more fields
    }

    #[repr(C)]
    struct orientation_t {
        origin: [f32; 3],
        axis: [[f32; 3]; 3],
    }

    #[repr(C)]
    struct glconfig_t {
        // stub
        _dummy: [u8; 1],
    }
}

// Export module functions if needed
pub use tr_model::*;
