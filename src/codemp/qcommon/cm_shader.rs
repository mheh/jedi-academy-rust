//! Mechanical port of `codemp/qcommon/cm_shader.cpp`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use crate::codemp::game::q_shared_h::{qboolean, MAX_QPATH, QTRUE, QFALSE};
use crate::codemp::game::surfaceflags_h::{
    CONTENTS_SOLID, CONTENTS_OPAQUE, CONTENTS_LAVA, CONTENTS_WATER, CONTENTS_FOG,
    CONTENTS_PLAYERCLIP, CONTENTS_MONSTERCLIP, CONTENTS_BOTCLIP, CONTENTS_SHOTCLIP,
    CONTENTS_TRIGGER, CONTENTS_NODROP, CONTENTS_TERRAIN, CONTENTS_LADDER,
    CONTENTS_ABSEIL, CONTENTS_OUTSIDE, CONTENTS_INSIDE, CONTENTS_DETAIL,
    CONTENTS_TRANSLUCENT, SURF_SKY, SURF_SLICK, SURF_NODAMAGE, SURF_NOIMPACT,
    SURF_NOMARKS, SURF_NODRAW, SURF_NOSTEPS, SURF_NODLIGHT, MATERIAL_LAST, MATERIALS,
};
use crate::codemp::qcommon::cm_local_h::{cmg, CCMShader};
use crate::codemp::qcommon::chash_h::{CHash, CHashItem, strcmp_compare};
use crate::codemp::qcommon::tags_h::TAG_SHADERTEXT;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// ====================
// CCMShaderText class
// ====================

pub struct CCMShaderText {
    pub mName: [c_char; MAX_QPATH],
    pub mNext: *mut CCMShaderText,
    pub mData: *const c_char,
}

impl CCMShaderText {
    // Constructors
    pub fn new(name: *const c_char, data: *const c_char) -> Self {
        let mut shader = CCMShaderText {
            mName: [0; MAX_QPATH],
            mNext: ptr::null_mut(),
            mData: data,
        };
        unsafe {
            Q_strncpyz(
                shader.mName.as_mut_ptr(),
                name,
                MAX_QPATH as c_int,
            );
        }
        shader
    }

    // Accessors
    #[inline]
    pub fn GetName(&self) -> *const c_char {
        self.mName.as_ptr()
    }

    #[inline]
    pub fn GetNext(&self) -> *mut CCMShaderText {
        self.mNext
    }

    #[inline]
    pub fn SetNext(&mut self, next: *mut CCMShaderText) {
        self.mNext = next;
    }

    pub fn Destroy(&self) {
        // no-op in Rust (handled by drop)
    }

    pub fn GetData(&self) -> *const c_char {
        self.mData
    }
}

// Implement CHashItem for CCMShaderText
unsafe impl CHashItem for CCMShaderText {
    unsafe fn GetName(item: *mut Self) -> *const c_char {
        (*item).GetName()
    }

    unsafe fn Destroy(_item: *mut Self) {
        // no-op - memory managed elsewhere
    }

    unsafe fn SetNext(item: *mut Self, next: *mut Self) {
        (*item).SetNext(next);
    }

    unsafe fn GetNext(item: *mut Self) -> *mut Self {
        (*item).GetNext()
    }
}

// ====================
// Global variables
// ====================

pub static mut shaderText: *mut c_char = ptr::null_mut();
pub static mut shaderTextTable: CHash<CCMShaderText> = CHash {
    mHashTable: [ptr::null_mut(); 1024],
    mNext: ptr::null_mut(),
    mCount: 0,
    mPrevious: ptr::null_mut(),
    mHash: 0,
    _TCompare: core::marker::PhantomData,
};
pub static mut cmShaderTable: CHash<CCMShader> = CHash {
    mHashTable: [ptr::null_mut(); 1024],
    mNext: ptr::null_mut(),
    mCount: 0,
    mPrevious: ptr::null_mut(),
    mHash: 0,
    _TCompare: core::marker::PhantomData,
};

// External functions (need to be linked)
extern "C" {
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: c_int);
    pub fn COM_ParseExt(data: *mut *const c_char, allowLineBreak: qboolean) -> *const c_char;
    pub fn FS_ListFiles(path: *const c_char, extension: *const c_char, numFiles: *mut c_int) -> *mut *mut c_char;
    pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    pub fn FS_FreeFile(buffer: *mut c_void);
    pub fn FS_FreeFileList(fileList: *mut *mut c_char);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    pub fn SkipWhitespace(p: *const c_char, hasNewLines: *mut qboolean) -> *const c_char;
    pub fn SkipBracedSection(p: *mut *const c_char);
    pub fn SkipRestOfLine(p: *mut *const c_char);
    pub fn Z_Malloc(size: c_int, tag: c_int, quitOnMem: qboolean) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Hunk_Alloc(size: c_int, h_high: c_int) -> *mut c_void;
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn atof(nptr: *const c_char) -> f32;
}

const MAX_SHADER_FILES: c_int = 1024;

/*
====================
CM_CreateShaderTextHash
=====================
*/
pub fn CM_CreateShaderTextHash() {
    let mut p: *const c_char;
    let mut hasNewLines: qboolean;
    let mut token: *const c_char;
    let mut shader: *mut CCMShaderText;

    unsafe {
        p = shaderText;
        // look for label
        while !p.is_null() && *p != 0 {
            p = SkipWhitespace(p, &mut hasNewLines);
            token = COM_ParseExt(&mut p, QTRUE);
            if *token == 0 {
                break;
            }
            shader = Box::into_raw(Box::new(CCMShaderText::new(token, p)));
            shaderTextTable.insert(shader);

            SkipBracedSection(&mut p);
        }
    }
}

/*
====================
CM_LoadShaderFiles

Finds and loads all .shader files, combining them into
a single large text block that can be scanned for shader names
=====================
*/

pub fn CM_LoadShaderFiles() {
    let mut shaderFiles1: *mut *mut c_char;
    let mut numShaders1: c_int = 0;
    #[cfg(not(feature = "FINAL_BUILD"))]
    let mut shaderFiles2: *mut *mut c_char;
    #[cfg(not(feature = "FINAL_BUILD"))]
    let mut numShaders2: c_int = 0;
    let mut buffers: [*mut c_char; 1024] = [ptr::null_mut(); 1024];
    let mut numShaders: c_int;
    let mut i: c_int;
    let mut sum: c_int = 0;

    unsafe {
        // scan for shader files
        shaderFiles1 = FS_ListFiles(
            "shaders\0".as_ptr() as *const c_char,
            ".shader\0".as_ptr() as *const c_char,
            &mut numShaders1,
        );
        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            shaderFiles2 = FS_ListFiles(
                "shaders/test\0".as_ptr() as *const c_char,
                ".shader\0".as_ptr() as *const c_char,
                &mut numShaders2,
            );
        }

        if shaderFiles1.is_null() || numShaders1 == 0 {
            Com_Printf(
                "^3WARNING: no shader files found\n\0".as_ptr() as *const c_char,
            );
            return;
        }

        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            numShaders = numShaders1 + numShaders2;
        }
        #[cfg(feature = "FINAL_BUILD")]
        {
            numShaders = numShaders1;
        }

        if numShaders > MAX_SHADER_FILES {
            numShaders = MAX_SHADER_FILES;
        }

        // load and parse shader files
        i = 0;
        while i < numShaders1 {
            let mut filename: [c_char; MAX_QPATH] = [0; MAX_QPATH];

            Com_sprintf(
                filename.as_mut_ptr(),
                MAX_QPATH as c_int,
                "shaders/%s\0".as_ptr() as *const c_char,
                *shaderFiles1.add(i as usize),
            );
            Com_DPrintf(
                "...loading '%s'\n\0".as_ptr() as *const c_char,
                filename.as_ptr(),
            );
            sum += FS_ReadFile(filename.as_ptr(), &mut (buffers[i as usize]));
            if buffers[i as usize].is_null() {
                Com_Error(
                    0, // ERR_FATAL
                    "Couldn't load %s\0".as_ptr() as *const c_char,
                    filename.as_ptr(),
                );
            }
            i += 1;
        }
        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            while i < numShaders {
                let mut filename: [c_char; MAX_QPATH] = [0; MAX_QPATH];

                Com_sprintf(
                    filename.as_mut_ptr(),
                    MAX_QPATH as c_int,
                    "shaders/test/%s\0".as_ptr() as *const c_char,
                    *shaderFiles2.add((i - numShaders1) as usize),
                );
                Com_DPrintf(
                    "...loading '%s'\n\0".as_ptr() as *const c_char,
                    filename.as_ptr(),
                );
                sum += FS_ReadFile(filename.as_ptr(), &mut (buffers[i as usize]));
                if buffers[i as usize].is_null() {
                    Com_Error(
                        1, // ERR_DROP
                        "Couldn't load %s\0".as_ptr() as *const c_char,
                        filename.as_ptr(),
                    );
                }
                i += 1;
            }
        }

        // build single large buffer
        shaderText = Z_Malloc(
            sum + numShaders * 2,
            TAG_SHADERTEXT,
            QTRUE,
        ) as *mut c_char;

        // free in reverse order, so the temp files are all dumped
        i = numShaders - 1;
        while i >= 0 {
            strcat(shaderText, "\n\0".as_ptr() as *const c_char);
            strcat(shaderText, buffers[i as usize]);
            FS_FreeFile(buffers[i as usize] as *mut c_void);
            i -= 1;
        }

        // free up memory
        FS_FreeFileList(shaderFiles1);
        #[cfg(not(feature = "FINAL_BUILD"))]
        {
            FS_FreeFileList(shaderFiles2);
        }
    }
}

/*
==================
CM_GetShaderText
==================
*/

pub fn CM_GetShaderText(key: *const c_char) -> *const c_char {
    let st: *mut CCMShaderText;

    unsafe {
        st = shaderTextTable.operator_index(key);
        if !st.is_null() {
            return (*st).GetData();
        }
    }
    ptr::null()
}

/*
==================
CM_FreeShaderText
==================
*/

pub fn CM_FreeShaderText() {
    unsafe {
        shaderTextTable.clear();
        if !shaderText.is_null() {
            Z_Free(shaderText as *mut c_void);
            shaderText = ptr::null_mut();
        }
    }
}

/*
==================
CM_LoadShaderText

  Loads in all the .shader files so it can be accessed by the server and the renderer
  Creates a hash table to quickly access the shader text
==================
*/

pub fn CM_LoadShaderText(forceReload: qboolean) {
    unsafe {
        if forceReload != 0 {
            CM_FreeShaderText();
        }
        if !shaderText.is_null() {
            return;
        }
        //	Com_Printf("Loading shader text .....\n");
        CM_LoadShaderFiles();
        CM_CreateShaderTextHash();

        //Com_Printf("..... %d shader definitions loaded\n", shaderTextTable.count());
    }
}

/*
===============
ParseSurfaceParm

surfaceparm <name>
===============
*/

#[repr(C)]
struct infoParm_t {
    name: *const c_char,
    clearSolid: c_int,
    surfaceFlags: c_int,
    contents: c_int,
}

const svInfoParms: [infoParm_t; 26] = [
    // Game surface flags
    infoParm_t {
        name: "sky\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_SKY,
        contents: 0,
    }, // emit light from an environment map
    infoParm_t {
        name: "slick\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_SLICK,
        contents: 0,
    },
    infoParm_t {
        name: "nodamage\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NODAMAGE,
        contents: 0,
    },
    infoParm_t {
        name: "noimpact\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NOIMPACT,
        contents: 0,
    }, // don't make impact explosions or marks
    infoParm_t {
        name: "nomarks\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NOMARKS,
        contents: 0,
    }, // don't make impact marks, but still explode
    infoParm_t {
        name: "nodraw\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NODRAW,
        contents: 0,
    }, // don't generate a drawsurface (or a lightmap)
    infoParm_t {
        name: "nosteps\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NOSTEPS,
        contents: 0,
    },
    infoParm_t {
        name: "nodlight\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NODLIGHT,
        contents: 0,
    }, // don't ever add dynamic lights
    // Game content flags
    infoParm_t {
        name: "nonsolid\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: 0,
    }, // special hack to clear solid flag
    infoParm_t {
        name: "nonopaque\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_OPAQUE,
        surfaceFlags: 0,
        contents: 0,
    }, // special hack to clear opaque flag
    infoParm_t {
        name: "lava\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_LAVA,
    }, // very damaging
    infoParm_t {
        name: "water\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_WATER,
    },
    infoParm_t {
        name: "fog\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_FOG,
    }, // carves surfaces entering
    infoParm_t {
        name: "playerclip\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_PLAYERCLIP,
    },
    infoParm_t {
        name: "monsterclip\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_MONSTERCLIP,
    },
    infoParm_t {
        name: "botclip\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_BOTCLIP,
    }, // for bots
    infoParm_t {
        name: "shotclip\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_SHOTCLIP,
    },
    infoParm_t {
        name: "trigger\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_TRIGGER,
    },
    infoParm_t {
        name: "nodrop\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_NODROP,
    }, // don't drop items or leave bodies (death fog, lava, etc)
    infoParm_t {
        name: "terrain\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_TERRAIN,
    }, // use special terrain collsion
    infoParm_t {
        name: "ladder\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_LADDER,
    }, // climb up in it like water
    infoParm_t {
        name: "abseil\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_ABSEIL,
    }, // can abseil down this brush
    infoParm_t {
        name: "outside\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_OUTSIDE,
    }, // volume is considered to be in the outside (i.e. not indoors)
    infoParm_t {
        name: "inside\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_INSIDE,
    }, // volume is considered to be inside (i.e. indoors)
    infoParm_t {
        name: "detail\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: 0,
        contents: CONTENTS_DETAIL,
    }, // don't include in structural bsp
    infoParm_t {
        name: "trans\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: 0,
        contents: CONTENTS_TRANSLUCENT,
    }, // surface has an alpha component
];

pub fn SV_ParseSurfaceParm(shader: *mut CCMShader, text: *mut *const c_char) {
    let mut token: *const c_char;
    let numsvInfoParms: c_int = 26;
    let mut i: c_int;

    unsafe {
        token = COM_ParseExt(text, QFALSE);
        i = 0;
        while i < numsvInfoParms {
            if Q_stricmp(token, svInfoParms[i as usize].name) == 0 {
                (*shader).surfaceFlags |= svInfoParms[i as usize].surfaceFlags;
                (*shader).contentFlags |= svInfoParms[i as usize].contents;
                (*shader).contentFlags &= svInfoParms[i as usize].clearSolid;
                break;
            }
            i += 1;
        }
    }
}

/*
=================
ParseMaterial
=================
*/

pub fn SV_ParseMaterial(shader: *mut CCMShader, text: *mut *const c_char) {
    let mut token: *const c_char;
    let mut i: c_int;

    unsafe {
        token = COM_ParseExt(text, QFALSE);
        if *token == 0 {
            Com_Printf(
                "^3WARNING: missing material in shader '%s'\n\0".as_ptr() as *const c_char,
                (*shader).shader.as_ptr(),
            );
            return;
        }
        i = 0;
        while i < MATERIAL_LAST {
            let material_name = MATERIALS[i as usize];
            if Q_stricmp(
                token,
                material_name.as_ptr() as *const c_char,
            ) == 0
            {
                (*shader).surfaceFlags |= i;
                break;
            }
            i += 1;
        }
    }
}

/*
===============
ParseVector
===============
*/

fn CM_ParseVector(shader: *mut CCMShader, text: *mut *const c_char, count: c_int, v: *mut f32) -> qboolean {
    let mut token: *const c_char;
    let mut i: c_int;

    unsafe {
        // FIXME: spaces are currently required after parens, should change parseext...
        token = COM_ParseExt(text, QFALSE);
        if strcmp(token, "(\0".as_ptr() as *const c_char) != 0 {
            Com_Printf(
                "^3WARNING: missing parenthesis in shader '%s'\n\0".as_ptr() as *const c_char,
                (*shader).shader.as_ptr(),
            );
            return QFALSE;
        }

        i = 0;
        while i < count {
            token = COM_ParseExt(text, QFALSE);
            if *token == 0 {
                Com_Printf(
                    "^3WARNING: missing vector element in shader '%s'\n\0".as_ptr() as *const c_char,
                    (*shader).shader.as_ptr(),
                );
                return QFALSE;
            }
            *v.add(i as usize) = atof(token);
            i += 1;
        }

        token = COM_ParseExt(text, QFALSE);
        if strcmp(token, ")\0".as_ptr() as *const c_char) != 0 {
            Com_Printf(
                "^3WARNING: missing parenthesis in shader '%s'\n\0".as_ptr() as *const c_char,
                (*shader).shader.as_ptr(),
            );
            return QFALSE;
        }
    }
    QTRUE
}

/*
=================
CM_ParseShader

The current text pointer is at the explicit text definition of the
shader.  Parse it into the global shader variable.

This extracts all the info from the shader required for physics and collision
It is designed to *NOT* load any image files and not require any of the renderer to
be initialised.
=================
*/

fn CM_ParseShader(shader: *mut CCMShader, text: *mut *const c_char) {
    let mut token: *const c_char;

    unsafe {
        token = COM_ParseExt(text, QTRUE);
        if *token != (b'{' as c_char) {
            Com_Printf(
                "^3WARNING: expecting '{', found '%s' instead in shader '%s'\n\0".as_ptr() as *const c_char,
                token,
                (*shader).shader.as_ptr(),
            );
            return;
        }

        loop {
            token = COM_ParseExt(text, QTRUE);
            if *token == 0 {
                Com_Printf(
                    "^3WARNING: no concluding '}' in shader %s\n\0".as_ptr() as *const c_char,
                    (*shader).shader.as_ptr(),
                );
                return;
            }

            // end of shader definition
            if *token == (b'}' as c_char) {
                break;
            }
            // stage definition
            else if *token == (b'{' as c_char) {
                SkipBracedSection(text);
                continue;
            }
            // material deprecated as of 11 Jan 01
            // material undeprecated as of 7 May 01 - q3map_material deprecated
            else if Q_stricmp(token, "material\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(token, "q3map_material\0".as_ptr() as *const c_char) == 0
            {
                SV_ParseMaterial(shader, text);
            }
            // sun parms
            // q3map_sun deprecated as of 11 Jan 01
            else if Q_stricmp(token, "sun\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(token, "q3map_sun\0".as_ptr() as *const c_char) == 0
            {
                //			float	a, b;

                token = COM_ParseExt(text, QFALSE);
                //			shader->sunLight[0] = atof( token );
                token = COM_ParseExt(text, QFALSE);
                //			shader->sunLight[1] = atof( token );
                token = COM_ParseExt(text, QFALSE);
                //			shader->sunLight[2] = atof( token );

                //			VectorNormalize( shader->sunLight );

                token = COM_ParseExt(text, QFALSE);
                //			a = atof( token );
                //			VectorScale( shader->sunLight, a, shader->sunLight);

                token = COM_ParseExt(text, QFALSE);
                //			a = DEG2RAD(atof( token ));

                token = COM_ParseExt(text, QFALSE);
                //			b = DEG2RAD(atof( token ));

                //			shader->sunDirection[0] = cos( a ) * cos( b );
                //			shader->sunDirection[1] = sin( a ) * cos( b );
                //			shader->sunDirection[2] = sin( b );
            } else if Q_stricmp(token, "surfaceParm\0".as_ptr() as *const c_char) == 0 {
                SV_ParseSurfaceParm(shader, text);
                continue;
            } else if Q_stricmp(token, "fogParms\0".as_ptr() as *const c_char) == 0 {
                let mut fogColor: [f32; 3] = [0.0; 3];
                if CM_ParseVector(shader, text, 3, fogColor.as_mut_ptr()) == QFALSE {
                    return;
                }

                token = COM_ParseExt(text, QFALSE);
                if *token == 0 {
                    Com_Printf(
                        "^3WARNING: missing parm for 'fogParms' keyword in shader '%s'\n\0".as_ptr() as *const c_char,
                        (*shader).shader.as_ptr(),
                    );
                    continue;
                }
                //			shader->depthForOpaque = atof( token );

                // skip any old gradient directions
                SkipRestOfLine(text);
                continue;
            }
        }
    }
}

/*
=================
CM_SetupShaderProperties

  Scans thru the shaders loaded for the map, parses the text of that shader and
  extracts the interesting info *WITHOUT* loading up any images or requiring
  the renderer to be active.
=================
*/

pub fn CM_SetupShaderProperties() {
    let mut i: c_int;
    let mut def: *const c_char;
    let mut shader: *mut CCMShader;

    unsafe {
        // Add all basic shaders to the cmShaderTable
        i = 0;
        while i < cmg.numShaders {
            cmShaderTable.insert(CM_GetShaderInfo_by_num(i));
            i += 1;
        }
        // Go through and parse evaluate shader names to shadernums
        i = 0;
        while i < cmg.numShaders {
            shader = CM_GetShaderInfo_by_num(i);
            if !shader.is_null() {
                def = CM_GetShaderText((*shader).shader.as_ptr());
                if !def.is_null() {
                    CM_ParseShader(shader, &mut (def as *mut c_char));
                }
            }
            i += 1;
        }
    }
}

pub fn CM_ShutdownShaderProperties() {
    unsafe {
        if cmShaderTable.count() > 0 {
            //		Com_Printf("Shutting down cmShaderTable .....\n");
            cmShaderTable.clear();
        }
    }
}

pub fn CM_GetShaderInfo(name: *const c_char) -> *mut CCMShader {
    let mut out: *mut CCMShader;
    let mut def: *const c_char;

    unsafe {
        out = cmShaderTable.operator_index(name);
        if !out.is_null() {
            return out;
        }

        // Create a new CCMShader class
        out = Hunk_Alloc(core::mem::size_of::<CCMShader>() as c_int, 0) as *mut CCMShader;
        // Set defaults
        Q_strncpyz(
            (*out).shader.as_mut_ptr(),
            name,
            MAX_QPATH as c_int,
        );
        (*out).contentFlags = CONTENTS_SOLID | CONTENTS_OPAQUE;

        // Parse in any text if it exists
        def = CM_GetShaderText(name);
        if !def.is_null() {
            CM_ParseShader(out, &mut (def as *mut c_char));
        }

        cmShaderTable.insert(out);
        out
    }
}

pub fn CM_GetShaderInfo_by_num(shaderNum: c_int) -> *mut CCMShader {
    let mut out: *mut CCMShader;

    unsafe {
        if shaderNum < 0 || shaderNum >= cmg.numShaders {
            return ptr::null_mut();
        }
        out = (cmg.shaders as *mut CCMShader).add(shaderNum as usize);
        out
    }
}

// end
