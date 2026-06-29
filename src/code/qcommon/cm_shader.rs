#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::ptr;

// Forward declarations and external functions
extern "C" {
    pub fn SkipWhitespace(data: *const c_char, hasNewLines: *mut c_int) -> *const c_char;
    pub fn COM_ParseExt(text: *mut *const c_char, allowLineBreaks: c_int) -> *const c_char;
    pub fn SkipBracedSection(text: *mut *const c_char);
    pub fn Com_Printf(format: *const c_char, ...);
    pub fn Com_sprintf(dest: *mut c_char, size: usize, format: *const c_char, ...);
    pub fn Com_DPrintf(format: *const c_char, ...);
    pub fn Com_Error(level: c_int, format: *const c_char, ...);
    pub fn FS_ListFiles(
        path: *const c_char,
        extension: *const c_char,
        numFiles: *mut c_int,
    ) -> *mut *mut c_char;
    pub fn FS_ReadFile(filename: *const c_char, buffer: *mut *mut c_void);
    pub fn FS_FreeFile(buffer: *mut c_void);
    pub fn FS_FreeFileList(fileList: *mut *mut c_char);
    pub fn COM_Compress(data: *mut c_char) -> c_int;
    pub fn Z_Malloc(size: c_int, tag: c_int, clear: c_int) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn SkipRestOfLine(text: *mut *const c_char);
}

// Stub external types/globals needed for structural coherence
extern "C" {
    pub static mut cmg: CMGameStub;
}

#[repr(C)]
pub struct CMGameStub {
    pub numShaders: c_int,
    pub shaders: *mut CCMShader,
}

// CCMShader type - used but defined elsewhere
#[repr(C)]
pub struct CCMShader {
    pub shader: [c_char; 64], // MAX_QPATH
    pub surfaceFlags: c_int,
    pub contentFlags: c_int,
    // other fields omitted for stub
}

pub extern "C" fn CM_GetShaderInfo_by_name(name: *const c_char) -> *mut CCMShader;
pub extern "C" fn CM_GetShaderInfo_by_num(shaderNum: c_int) -> *mut CCMShader;

// CCMShaderText class translated to struct
#[repr(C)]
struct CCMShaderText {
    mName: [c_char; 64],      // MAX_QPATH
    mNext: *mut CCMShaderText,
    mData: *const c_char,
}

impl CCMShaderText {
    // Constructor
    fn new(name: *const c_char, data: *const c_char) -> *mut Self {
        let shader = unsafe {
            let ptr = libc::malloc(std::mem::size_of::<CCMShaderText>()) as *mut CCMShaderText;
            (*ptr).mNext = ptr::null_mut();
            (*ptr).mData = data;
            Q_strncpyz((*ptr).mName.as_mut_ptr(), name, 64);
            ptr
        };
        shader
    }

    // Accessors
    fn GetName(&self) -> *const c_char {
        self.mName.as_ptr()
    }

    fn GetNext(&self) -> *mut CCMShaderText {
        self.mNext
    }

    fn SetNext(&mut self, next: *mut CCMShaderText) {
        self.mNext = next;
    }

    fn Destroy(&mut self) {
        unsafe {
            libc::free(self as *mut _ as *mut c_void);
        }
    }

    fn GetData(&self) -> *const c_char {
        self.mData
    }
}

// Global variables - hash table implementations are stubbed as simple structures
// CHash<CCMShaderText> shaderTextTable;
// CHash<CCMShader> cmShaderTable;
pub static mut shaderText: *mut c_char = ptr::null_mut();

// Stub hash table types - actual implementation would need CHash
pub struct CHashShaderText {
    // Stub implementation
}

pub struct CHashCMShader {
    // Stub implementation
}

static mut shaderTextTable: CHashShaderText = CHashShaderText {};
static mut cmShaderTable: CHashCMShader = CHashCMShader {};

// rwwFIXMEFIXME: Called at RE_BeginRegistration because Hunk_Clear
// destroys the memory cmShaderTable is on. This is a temp solution
// I guess.
pub fn ShaderTableCleanup() {
    // cmShaderTable.clear();
}

/*
====================
CM_CreateShaderTextHash
=====================
*/
pub fn CM_CreateShaderTextHash() {
    let mut p: *const c_char;
    let mut hasNewLines: c_int = 0;
    let token: *const c_char;
    let shader: *mut CCMShaderText;

    unsafe {
        p = shaderText;
        // look for label
        while !p.is_null() {
            p = SkipWhitespace(p, &mut hasNewLines);
            token = COM_ParseExt(&mut (p as *mut c_char) as *mut *mut c_char, 1);
            if *token == 0 {
                break;
            }
            shader = CCMShaderText::new(token, p);
            // shaderTextTable.insert(shader);

            SkipBracedSection(&mut (p as *mut c_char) as *mut *mut c_char);
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
const MAX_SHADER_FILES: c_int = 1024;

pub fn CM_LoadShaderFiles() {
    let shaderFiles1: *mut *mut c_char;
    let mut numShaders1: c_int = 0;
    let mut buffers: [*mut c_char; 1024] = [ptr::null_mut(); 1024];
    let mut numShaders: c_int;
    let mut i: c_int;
    let mut sum: c_int = 0;

    unsafe {
        // scan for shader files
        shaderFiles1 = FS_ListFiles(
            b"shaders\0".as_ptr() as *const c_char,
            b".shader\0".as_ptr() as *const c_char,
            &mut numShaders1,
        );

        if shaderFiles1.is_null() || numShaders1 == 0 {
            Com_Printf(b"WARNING: no shader files found\n\0".as_ptr() as *const c_char);
            return;
        }

        numShaders = numShaders1;
        if numShaders > MAX_SHADER_FILES {
            numShaders = MAX_SHADER_FILES;
        }

        // load and parse shader files
        for i in 0..numShaders1 {
            let mut filename: [c_char; 64] = [0; 64]; // MAX_QPATH

            Com_sprintf(
                filename.as_mut_ptr(),
                64,
                b"shaders/%s\0".as_ptr() as *const c_char,
                *shaderFiles1.add(i as usize),
            );
            Com_DPrintf(b"...loading '%s'\n\0".as_ptr() as *const c_char, filename.as_ptr());
            FS_ReadFile(filename.as_ptr(), &mut (buffers[i as usize] as *mut c_void));
            if buffers[i as usize].is_null() {
                Com_Error(
                    1, // ERR_DROP
                    b"Couldn't load %s\0".as_ptr() as *const c_char,
                    filename.as_ptr(),
                );
            }
            sum += COM_Compress(buffers[i as usize]);
        }

        // build single large buffer
        shaderText = Z_Malloc(sum + numShaders * 2, 0, 1) as *mut c_char; // TAG_SHADERTEXT

        // free in reverse order, so the temp files are all dumped
        let mut i = numShaders - 1;
        loop {
            if i < 0 {
                break;
            }
            // strcat( shaderText, "\n" );
            // strcat( shaderText, buffers[i] );
            FS_FreeFile(buffers[i as usize] as *mut c_void);
            i -= 1;
        }

        // free up memory
        FS_FreeFileList(shaderFiles1);
    }
}

/*
==================
CM_GetShaderText
==================
*/

pub fn CM_GetShaderText(key: *const c_char) -> *const c_char {
    // CCMShaderText *st;
    //
    // st = shaderTextTable[key];
    // if(st)
    // {
    //     return(st->GetData());
    // }
    // return(NULL);
    ptr::null()
}

/*
==================
CM_FreeShaderText
==================
*/

pub fn CM_FreeShaderText() {
    // shaderTextTable.clear();
    unsafe {
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

pub fn CM_LoadShaderText(forceReload: bool) {
    unsafe {
        if forceReload {
            CM_FreeShaderText();
        }
        if !shaderText.is_null() {
            return;
        }
        Com_Printf(b"Loading shader text .....\n\0".as_ptr() as *const c_char);
        CM_LoadShaderFiles();
        CM_CreateShaderTextHash();

        // Com_Printf("..... %d shader definitions loaded\n", shaderTextTable.count());
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

// Constants for shader info parms
const CONTENTS_SOLID: c_int = 1 << 0;
const CONTENTS_OPAQUE: c_int = 1 << 9;
const CONTENTS_LAVA: c_int = 1 << 3;
const CONTENTS_SLIME: c_int = 1 << 2;
const CONTENTS_WATER: c_int = 1 << 4;
const CONTENTS_FOG: c_int = 1 << 5;
const CONTENTS_SHOTCLIP: c_int = 1 << 11;
const CONTENTS_PLAYERCLIP: c_int = 1 << 10;
const CONTENTS_MONSTERCLIP: c_int = 1 << 12;
const CONTENTS_BOTCLIP: c_int = 1 << 13;
const CONTENTS_TRIGGER: c_int = 1 << 14;
const CONTENTS_NODROP: c_int = 1 << 15;
const CONTENTS_TERRAIN: c_int = 1 << 16;
const CONTENTS_LADDER: c_int = 1 << 17;
const CONTENTS_ABSEIL: c_int = 1 << 18;
const CONTENTS_OUTSIDE: c_int = 1 << 19;
const CONTENTS_INSIDE: c_int = 1 << 20;
const CONTENTS_DETAIL: c_int = 1 << 21;
const CONTENTS_TRANSLUCENT: c_int = 1 << 22;

const SURF_SKY: c_int = 1 << 0;
const SURF_SLICK: c_int = 1 << 1;
const SURF_NODAMAGE: c_int = 1 << 2;
const SURF_NOIMPACT: c_int = 1 << 3;
const SURF_NOMARKS: c_int = 1 << 4;
const SURF_NODRAW: c_int = 1 << 5;
const SURF_NOSTEPS: c_int = 1 << 6;
const SURF_NODLIGHT: c_int = 1 << 7;
const SURF_METALSTEPS: c_int = 1 << 8;
const SURF_NOMISCENTS: c_int = 1 << 9;
const SURF_FORCEFIELD: c_int = 1 << 10;
const SURF_FORCESIGHT: c_int = 1 << 11;

static svInfoParms: &[infoParm_t] = &[
    // Game content Flags
    infoParm_t {
        name: b"nonsolid\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: 0,
    }, // special hack to clear solid flag
    infoParm_t {
        name: b"nonopaque\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_OPAQUE,
        surfaceFlags: 0,
        contents: 0,
    }, // special hack to clear opaque flag
    infoParm_t {
        name: b"lava\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_LAVA,
    }, // very damaging
    infoParm_t {
        name: b"slime\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_SLIME,
    }, // mildly damaging
    infoParm_t {
        name: b"water\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_WATER,
    },
    infoParm_t {
        name: b"fog\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_FOG,
    }, // carves surfaces entering
    infoParm_t {
        name: b"shotclip\0".as_ptr() as *const c_char,
        clearSolid: !CONTENTS_SOLID,
        surfaceFlags: 0,
        contents: CONTENTS_SHOTCLIP,
    }, /* block shots, but not people */
    infoParm_t {
        name: b"playerclip\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_PLAYERCLIP,
    }, /* block only the player */
    infoParm_t {
        name: b"monsterclip\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_MONSTERCLIP,
    },
    infoParm_t {
        name: b"botclip\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_BOTCLIP,
    }, /* NPC do not enter */
    infoParm_t {
        name: b"trigger\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_TRIGGER,
    },
    infoParm_t {
        name: b"nodrop\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_NODROP,
    }, // don't drop items or leave bodies (death fog, lava, etc)
    infoParm_t {
        name: b"terrain\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_TERRAIN,
    }, /* use special terrain collsion */
    infoParm_t {
        name: b"ladder\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_LADDER,
    }, // climb up in it like water
    infoParm_t {
        name: b"abseil\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_ABSEIL,
    }, // can abseil down this brush
    infoParm_t {
        name: b"outside\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_OUTSIDE,
    }, // volume is considered to be in the outside (i.e. not indoors)
    infoParm_t {
        name: b"inside\0".as_ptr() as *const c_char,
        clearSolid: !(CONTENTS_SOLID | CONTENTS_OPAQUE),
        surfaceFlags: 0,
        contents: CONTENTS_INSIDE,
    }, // volume is considered to be inside (i.e. indoors)
    infoParm_t {
        name: b"detail\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: 0,
        contents: CONTENTS_DETAIL,
    }, // don't include in structural bsp
    infoParm_t {
        name: b"trans\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: 0,
        contents: CONTENTS_TRANSLUCENT,
    }, // surface has an alpha component
    /* Game surface flags */
    infoParm_t {
        name: b"sky\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_SKY,
        contents: 0,
    }, /* emit light from an environment map */
    infoParm_t {
        name: b"slick\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_SLICK,
        contents: 0,
    },
    infoParm_t {
        name: b"nodamage\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NODAMAGE,
        contents: 0,
    },
    infoParm_t {
        name: b"noimpact\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NOIMPACT,
        contents: 0,
    }, /* don't make impact explosions or marks */
    infoParm_t {
        name: b"nomarks\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NOMARKS,
        contents: 0,
    }, /* don't make impact marks, but still explode */
    infoParm_t {
        name: b"nodraw\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NODRAW,
        contents: 0,
    }, /* don't generate a drawsurface (or a lightmap) */
    infoParm_t {
        name: b"nosteps\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NOSTEPS,
        contents: 0,
    },
    infoParm_t {
        name: b"nodlight\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NODLIGHT,
        contents: 0,
    }, /* don't ever add dynamic lights */
    infoParm_t {
        name: b"metalsteps\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_METALSTEPS,
        contents: 0,
    },
    infoParm_t {
        name: b"nomiscents\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_NOMISCENTS,
        contents: 0,
    }, /* No misc ents on this surface */
    infoParm_t {
        name: b"forcefield\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_FORCEFIELD,
        contents: 0,
    },
    infoParm_t {
        name: b"forcesight\0".as_ptr() as *const c_char,
        clearSolid: -1,
        surfaceFlags: SURF_FORCESIGHT,
        contents: 0,
    }, // only visible with force sight
];

pub fn SV_ParseSurfaceParm(shader: *mut CCMShader, text: *mut *const c_char) {
    let token: *const c_char;
    let numsvInfoParms: c_int = svInfoParms.len() as c_int;
    let mut i: c_int;

    unsafe {
        token = COM_ParseExt(text, 0);
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
const MATERIAL_LAST: c_int = 11; // Stub value

const MATERIALS: &str = ""; // Stub - would need actual material strings

const MATERIAL_MASK: c_int = 0xF; // Stub

pub fn SV_ParseMaterial(shader: *mut CCMShader, text: *mut *const c_char) {
    let token: *const c_char;
    let mut i: c_int;

    unsafe {
        token = COM_ParseExt(text, 0);
        if *token == 0 {
            Com_Printf(
                b"WARNING: missing material in shader '%s'\n\0".as_ptr() as *const c_char,
                (*shader).shader.as_ptr(),
            );
            return;
        }
        i = 0;
        while i < MATERIAL_LAST {
            // Material names would be indexed here, but we're stubbing
            // if ( !Q_stricmp( token, svMaterialNames[i] ) )
            // {
            //     shader->surfaceFlags &= ~MATERIAL_MASK;
            //     shader->surfaceFlags |= i;
            //     break;
            // }
            i += 1;
        }
    }
}

/*
===============
ParseVector
===============
*/
pub fn CM_ParseVector(
    shader: *mut CCMShader,
    text: *mut *const c_char,
    count: c_int,
    v: *mut f32,
) -> c_int {
    let token: *const c_char;
    let mut i: c_int;

    unsafe {
        // FIXME: spaces are currently required after parens, should change parseext...
        token = COM_ParseExt(text, 0);
        if token as *const u8 as u8 != b'(' {
            Com_Printf(
                b"WARNING: missing parenthesis in shader '%s'\n\0".as_ptr() as *const c_char,
                (*shader).shader.as_ptr(),
            );
            return 0; // qfalse
        }

        i = 0;
        while i < count {
            token = COM_ParseExt(text, 0);
            if *token == 0 {
                Com_Printf(
                    b"WARNING: missing vector element in shader '%s'\n\0".as_ptr() as *const c_char,
                    (*shader).shader.as_ptr(),
                );
                return 0; // qfalse
            }
            *v.add(i as usize) = libc::atof(token);
            i += 1;
        }

        token = COM_ParseExt(text, 0);
        if token as *const u8 as u8 != b')' {
            Com_Printf(
                b"WARNING: missing parenthesis in shader '%s'\n\0".as_ptr() as *const c_char,
                (*shader).shader.as_ptr(),
            );
            return 0; // qfalse
        }
        return 1; // qtrue
    }
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
pub fn CM_ParseShader(shader: *mut CCMShader, text: *mut *const c_char) {
    let mut token: *const c_char;

    unsafe {
        token = COM_ParseExt(text, 1);
        if *token as u8 != b'{' {
            Com_Printf(
                b"WARNING: expecting '{', found '%s' instead in shader '%s'\n\0".as_ptr()
                    as *const c_char,
                token,
                (*shader).shader.as_ptr(),
            );
            return;
        }

        loop {
            token = COM_ParseExt(text, 1);
            if *token == 0 {
                Com_Printf(
                    b"WARNING: no concluding '}' in shader %s\n\0".as_ptr() as *const c_char,
                    (*shader).shader.as_ptr(),
                );
                return;
            }

            // end of shader definition
            if *token as u8 == b'}' {
                break;
            }
            // stage definition
            else if *token as u8 == b'{' {
                SkipBracedSection(text);
                continue;
            }
            // material deprecated as of 11 Jan 01
            // material undeprecated as of 7 May 01 - q3map_material deprecated
            else if Q_stricmp(token, b"material\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(token, b"q3map_material\0".as_ptr() as *const c_char) == 0
            {
                SV_ParseMaterial(shader, text);
            }
            // sun parms
            // q3map_sun deprecated as of 11 Jan 01
            else if Q_stricmp(token, b"sun\0".as_ptr() as *const c_char) == 0
                || Q_stricmp(token, b"q3map_sun\0".as_ptr() as *const c_char) == 0
            {
                //			float	a, b;

                token = COM_ParseExt(text, 0);
                //			shader->sunLight[0] = atof( token );
                token = COM_ParseExt(text, 0);
                //			shader->sunLight[1] = atof( token );
                token = COM_ParseExt(text, 0);
                //			shader->sunLight[2] = atof( token );

                //			VectorNormalize( shader->sunLight );

                token = COM_ParseExt(text, 0);
                //			a = atof( token );
                //			VectorScale( shader->sunLight, a, shader->sunLight);

                token = COM_ParseExt(text, 0);
                //			a = DEG2RAD(atof( token ));

                token = COM_ParseExt(text, 0);
                //			b = DEG2RAD(atof( token ));

                //			shader->sunDirection[0] = cos( a ) * cos( b );
                //			shader->sunDirection[1] = sin( a ) * cos( b );
                //			shader->sunDirection[2] = sin( b );
            } else if Q_stricmp(token, b"surfaceParm\0".as_ptr() as *const c_char) == 0 {
                SV_ParseSurfaceParm(shader, text);
                continue;
            } else if Q_stricmp(token, b"fogParms\0".as_ptr() as *const c_char) == 0 {
                let mut fogColor: [f32; 3] = [0.0; 3];
                if CM_ParseVector(shader, text, 3, fogColor.as_mut_ptr()) == 0 {
                    return;
                }

                token = COM_ParseExt(text, 0);
                if *token == 0 {
                    Com_Printf(
                        b"WARNING: missing parm for 'fogParms' keyword in shader '%s'\n\0"
                            .as_ptr() as *const c_char,
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
        return;
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
    let def: *const c_char;
    let shader: *mut CCMShader;

    unsafe {
        // Add all basic shaders to the cmShaderTable
        i = 0;
        while i < cmg.numShaders {
            // cmShaderTable.insert(CM_GetShaderInfo(i));
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
        // if(cmShaderTable.count())
        // {
        Com_Printf(b"Shutting down cmShaderTable .....\n\0".as_ptr() as *const c_char);
        // cmShaderTable.clear();
        // }
    }
}

pub fn CM_GetShaderInfo_by_name(name: *const c_char) -> *mut CCMShader {
    let out: *mut CCMShader;
    let def: *const c_char;

    unsafe {
        // out = cmShaderTable[name];
        // if(out)
        // {
        //     return(out);
        // }

        // Create a new CCMShader class
        //out = (CCMShader *)Hunk_Alloc( sizeof( CCMShader ), h_high );
        // out = (CCMShader *)Hunk_Alloc( sizeof( CCMShader ), qtrue );
        // // Set defaults
        // Q_strncpyz(out->shader, name, MAX_QPATH);
        // out->contentFlags = CONTENTS_SOLID | CONTENTS_OPAQUE;

        // // Parse in any text if it exists
        // def = CM_GetShaderText(name);
        // if(def)
        // {
        //     CM_ParseShader(out, &def);
        // }

        // cmShaderTable.insert(out);
        // return(out);
        ptr::null_mut()
    }
}

pub fn CM_GetShaderInfo_by_num(shaderNum: c_int) -> *mut CCMShader {
    let out: *mut CCMShader;

    unsafe {
        if (shaderNum < 0) || (shaderNum >= cmg.numShaders) {
            return ptr::null_mut();
        }
        out = cmg.shaders.add(shaderNum as usize);
        return out;
    }
}

// end
