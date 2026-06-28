
/*****************************************************************************
 * name:		be_aas_bspq3.c
 *
 * desc:		BSP, Environment Sampling
 *
 * $Archive: /MissionPack/code/botlib/be_aas_bspq3.c $
 * $Author: Ttimo $
 * $Revision: 5 $
 * $Modtime: 4/22/01 8:52a $
 * $Date: 4/22/01 8:52a $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char, c_void};
use std::ffi::CStr;
use std::ptr;
use std::ptr::{addr_of_mut, addr_of};

// Forward declarations of types from other modules
// These would be defined in: q_shared.h, botlib.h, be_aas.h, be_aas_funcs.h, be_aas_def.h, aasfile.h, l_*.h
extern "C" {
    pub static mut botimport: botlib_import_t;

    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, count: usize);
    fn Com_Memset(dest: *mut c_void, c: c_int, count: usize);
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn atof(nptr: *const c_char) -> f64;
    fn atoi(nptr: *const c_char) -> c_int;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;

    pub fn FreeMemory(ptr: *mut c_void);
    pub fn GetClearedHunkMemory(size: usize) -> *mut c_void;
    pub fn GetHunkMemory(size: usize) -> *mut c_void;
    pub fn LoadScriptMemory(data: *mut c_char, size: c_int, name: *const c_char) -> *mut script_t;
    pub fn SetScriptFlags(script: *mut script_t, flags: c_int);
    pub fn PS_ReadToken(script: *mut script_t, token: *mut token_t) -> c_int;
    pub fn FreeScript(script: *mut script_t);
    pub fn ScriptError(script: *mut script_t, format: *const c_char, ...);
    pub fn PS_ExpectTokenType(script: *mut script_t, type_: c_int, subtype: c_int, token: *mut token_t) -> c_int;
    pub fn StripDoubleQuotes(string: *mut c_char);
    pub fn VectorClear(vec: *mut f32);
}

// Type stubs for external types
// Porting note: Print is a varargs function pointer in C; Rust function pointers cannot express varargs
// We store it as a raw void pointer and cast to appropriate signatures for each call
pub type PrintFn2 = unsafe extern "C" fn(c_int, *const c_char) -> ();
pub type PrintFn3 = unsafe extern "C" fn(c_int, *const c_char, *const c_char) -> ();

#[repr(C)]
pub struct botlib_import_t {
    pub Print: *const c_void, // void (*)(int, const char*, ...)
    pub Trace: unsafe extern "C" fn(*mut bsp_trace_t, *mut f32, *mut f32, *mut f32, *mut f32, c_int, c_int) -> (),
    pub PointContents: unsafe extern "C" fn(*mut f32) -> c_int,
    pub EntityTrace: unsafe extern "C" fn(*mut bsp_trace_t, *mut f32, *mut f32, *mut f32, *mut f32, c_int, c_int) -> (),
    pub inPVS: unsafe extern "C" fn(*mut f32, *mut f32) -> c_int,
    pub BSPModelMinsMaxsOrigin: unsafe extern "C" fn(c_int, *mut f32, *mut f32, *mut f32, *mut f32) -> (),
    pub BSPEntityData: unsafe extern "C" fn() -> *const c_char,
}

#[repr(C)]
pub struct bsp_trace_t {
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: plane_t,
    pub surface: surf_t,
    pub contents: c_int,
    pub ent: c_int,
}

#[repr(C)]
pub struct plane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: c_int,
}

#[repr(C)]
pub struct surf_t {
    pub name: *const c_char,
    pub flags: c_int,
    pub value: c_int,
}

#[repr(C)]
pub struct script_t;

#[repr(C)]
pub struct token_t {
    pub type_: c_int,
    pub subtype: c_int,
    pub intvalue: c_int,
    pub floatvalue: f32,
    pub string: [c_char; 1024],
}

// Defines from q_shared.h and elsewhere
const PRT_MESSAGE: c_int = 1;
const TT_STRING: c_int = 1;

const SCFL_NOSTRINGWHITESPACES: c_int = 0x00000001;
const SCFL_NOSTRINGESCAPECHARS: c_int = 0x00000002;

const BLERR_NOERROR: c_int = 0;

// Content flags (from q_shared.h)
const CONTENTS_SOLID: c_int = 0x00000001;
const CONTENTS_WINDOW: c_int = 0x00000002;
const CONTENTS_AUX: c_int = 0x00000004;
const CONTENTS_LAVA: c_int = 0x00000008;
const CONTENTS_SLIME: c_int = 0x00000010;
const CONTENTS_WATER: c_int = 0x00000020;
const CONTENTS_MIST: c_int = 0x00000040;
const LAST_VISIBLE_CONTENTS: c_int = 0x00000040;
const CONTENTS_AREAPORTAL: c_int = 0x00008000;
const CONTENTS_PLAYERCLIP: c_int = 0x00010000;
const CONTENTS_MONSTERCLIP: c_int = 0x00020000;
const CONTENTS_CURRENT_0: c_int = 0x00040000;
const CONTENTS_CURRENT_90: c_int = 0x00080000;
const CONTENTS_CURRENT_180: c_int = 0x00100000;
const CONTENTS_CURRENT_270: c_int = 0x00200000;
const CONTENTS_CURRENT_UP: c_int = 0x00400000;
const CONTENTS_CURRENT_DOWN: c_int = 0x00800000;
const CONTENTS_ORIGIN: c_int = 0x01000000;
const CONTENTS_MONSTER: c_int = 0x02000000;
const CONTENTS_DEADMONSTER: c_int = 0x04000000;
const CONTENTS_DETAIL: c_int = 0x08000000;
const CONTENTS_TRANSLUCENT: c_int = 0x10000000;
const CONTENTS_LADDER: c_int = 0x20000000;

//#define TRACE_DEBUG

const ON_EPSILON: f32 = 0.005;
//#define DEG2RAD( a ) (( a * M_PI ) / 180.0F)

const MAX_BSPENTITIES: usize = 2048;
const MAX_EPAIRKEY: usize = 256;

#[repr(C)]
pub struct rgb_s
{
    pub red: c_int,
    pub green: c_int,
    pub blue: c_int,
}
pub type rgb_t = rgb_s;

//bsp entity epair
#[repr(C)]
pub struct bsp_epair_s
{
    pub key: *mut c_char,
    pub value: *mut c_char,
    pub next: *mut bsp_epair_s,
}
pub type bsp_epair_t = bsp_epair_s;

//bsp data entity
#[repr(C)]
pub struct bsp_entity_s
{
    pub epairs: *mut bsp_epair_t,
}
pub type bsp_entity_t = bsp_entity_s;

//id Sofware BSP data
#[repr(C)]
pub struct bsp_s
{
    //true when bsp file is loaded
    pub loaded: c_int,
    //entity data
    pub entdatasize: c_int,
    pub dentdata: *mut c_char,
    //bsp entities
    pub numentities: c_int,
    pub entities: [bsp_entity_t; MAX_BSPENTITIES],
}
pub type bsp_t = bsp_s;

//global bsp
pub static mut bspworld: bsp_t = bsp_t {
    loaded: 0,
    entdatasize: 0,
    dentdata: 0 as *mut c_char,
    numentities: 0,
    entities: [bsp_entity_t {
        epairs: 0 as *mut bsp_epair_t,
    }; MAX_BSPENTITIES],
};


#[cfg(feature = "bsp_debug")]
#[repr(C)]
pub struct cname_s
{
    pub value: c_int,
    pub name: *const c_char,
}

#[cfg(feature = "bsp_debug")]
pub type cname_t = cname_s;

#[cfg(feature = "bsp_debug")]
pub static CONTENTNAMES: &[cname_t] = &[
    cname_t { value: CONTENTS_SOLID, name: b"CONTENTS_SOLID\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_WINDOW, name: b"CONTENTS_WINDOW\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_AUX, name: b"CONTENTS_AUX\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_LAVA, name: b"CONTENTS_LAVA\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_SLIME, name: b"CONTENTS_SLIME\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_WATER, name: b"CONTENTS_WATER\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_MIST, name: b"CONTENTS_MIST\0" as *const u8 as *const c_char },
    cname_t { value: LAST_VISIBLE_CONTENTS, name: b"LAST_VISIBLE_CONTENTS\0" as *const u8 as *const c_char },

    cname_t { value: CONTENTS_AREAPORTAL, name: b"CONTENTS_AREAPORTAL\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_PLAYERCLIP, name: b"CONTENTS_PLAYERCLIP\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_MONSTERCLIP, name: b"CONTENTS_MONSTERCLIP\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_CURRENT_0, name: b"CONTENTS_CURRENT_0\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_CURRENT_90, name: b"CONTENTS_CURRENT_90\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_CURRENT_180, name: b"CONTENTS_CURRENT_180\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_CURRENT_270, name: b"CONTENTS_CURRENT_270\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_CURRENT_UP, name: b"CONTENTS_CURRENT_UP\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_CURRENT_DOWN, name: b"CONTENTS_CURRENT_DOWN\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_ORIGIN, name: b"CONTENTS_ORIGIN\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_MONSTER, name: b"CONTENTS_MONSTER\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_DEADMONSTER, name: b"CONTENTS_DEADMONSTER\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_DETAIL, name: b"CONTENTS_DETAIL\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_TRANSLUCENT, name: b"CONTENTS_TRANSLUCENT\0" as *const u8 as *const c_char },
    cname_t { value: CONTENTS_LADDER, name: b"CONTENTS_LADDER\0" as *const u8 as *const c_char },
    cname_t { value: 0, name: ptr::null() },
];

#[cfg(feature = "bsp_debug")]
pub unsafe fn PrintContents(mut contents: c_int)
{
    let mut i: c_int = 0;

    while CONTENTNAMES[i as usize].value != 0
    {
        if contents & CONTENTNAMES[i as usize].value != 0
        {
            let print_fn: PrintFn3 = core::mem::transmute(botimport.Print);
            print_fn(PRT_MESSAGE, b"%s\n\0" as *const u8 as *const c_char, CONTENTNAMES[i as usize].name);
        } //end if
        i += 1;
    } //end for
} //end of the function PrintContents

//===========================================================================
// traces axial boxes of any size through the world
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_Trace(start: *mut f32, mins: *mut f32, maxs: *mut f32, end: *mut f32, passent: c_int, contentmask: c_int) -> bsp_trace_t
{
    let mut bsptrace: bsp_trace_t = core::mem::zeroed();
    (botimport.Trace)(&mut bsptrace, start, mins, maxs, end, passent, contentmask);
    return bsptrace;
} //end of the function AAS_Trace
//===========================================================================
// returns the contents at the given point
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_PointContents(point: *mut f32) -> c_int
{
    return (botimport.PointContents)(point);
} //end of the function AAS_PointContents
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_EntityCollision(entnum: c_int,
                        start: *mut f32, boxmins: *mut f32, boxmaxs: *mut f32, end: *mut f32,
                                    contentmask: c_int, trace: *mut bsp_trace_t) -> c_int
{
    let mut enttrace: bsp_trace_t = core::mem::zeroed();

    (botimport.EntityTrace)(&mut enttrace, start, boxmins, boxmaxs, end, entnum, contentmask);
    if enttrace.fraction < (*trace).fraction
    {
        Com_Memcpy(trace as *mut c_void, &mut enttrace as *mut bsp_trace_t as *const c_void, core::mem::size_of::<bsp_trace_t>());
        return 1; // qtrue
    } //end if
    return 0; // qfalse
} //end of the function AAS_EntityCollision
//===========================================================================
// returns true if in Potentially Hearable Set
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_inPVS(p1: *mut f32, p2: *mut f32) -> c_int
{
    return (botimport.inPVS)(p1, p2);
} //end of the function AAS_InPVS
//===========================================================================
// returns true if in Potentially Visible Set
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_inPHS(p1: *mut f32, p2: *mut f32) -> c_int
{
    return 1; // qtrue
} //end of the function AAS_inPHS
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_BSPModelMinsMaxsOrigin(modelnum: c_int, angles: *mut f32, mins: *mut f32, maxs: *mut f32, origin: *mut f32)
{
    (botimport.BSPModelMinsMaxsOrigin)(modelnum, angles, mins, maxs, origin);
} //end of the function AAS_BSPModelMinsMaxs
//===========================================================================
// unlinks the entity from all leaves
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_UnlinkFromBSPLeaves(leaves: *mut bsp_link_t)
{
} //end of the function AAS_UnlinkFromBSPLeaves
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_BSPLinkEntity(absmins: *mut f32, absmaxs: *mut f32, entnum: c_int, modelnum: c_int) -> *mut bsp_link_t
{
    return ptr::null_mut();
} //end of the function AAS_BSPLinkEntity
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_BoxEntities(absmins: *mut f32, absmaxs: *mut f32, list: *mut c_int, maxcount: c_int) -> c_int
{
    return 0;
} //end of the function AAS_BoxEntities
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_NextBSPEntity(mut ent: c_int) -> c_int
{
    ent += 1;
    if ent >= 1 && ent < bspworld.numentities { return ent; }
    return 0;
} //end of the function AAS_NextBSPEntity
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_BSPEntityInRange(ent: c_int) -> c_int
{
    if ent <= 0 || ent >= bspworld.numentities
    {
        let print_fn: PrintFn2 = core::mem::transmute(botimport.Print);
        print_fn(PRT_MESSAGE, b"bsp entity out of range\n\0" as *const u8 as *const c_char);
        return 0; // qfalse
    } //end if
    return 1; // qtrue
} //end of the function AAS_BSPEntityInRange
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_ValueForBSPEpairKey(ent: c_int, key: *const c_char, value: *mut c_char, size: c_int) -> c_int
{
    let mut epair: *mut bsp_epair_t;

    *value = 0 as c_char;
    if AAS_BSPEntityInRange(ent) == 0 { return 0; } // qfalse
    epair = bspworld.entities[ent as usize].epairs;
    while epair != ptr::null_mut()
    {
        if strcmp((*epair).key, key) == 0
        {
            strncpy(value, (*epair).value, (size - 1) as usize);
            *value.offset((size - 1) as isize) = 0 as c_char;
            return 1; // qtrue
        } //end if
        epair = (*epair).next;
    } //end for
    return 0; // qfalse
} //end of the function AAS_FindBSPEpair
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_VectorForBSPEpairKey(ent: c_int, key: *const c_char, v: *mut f32) -> c_int
{
    let mut buf: [c_char; 256] = [0; 256];
    let mut v1: f64 = 0.0;
    let mut v2: f64 = 0.0;
    let mut v3: f64 = 0.0;

    VectorClear(v);
    if AAS_ValueForBSPEpairKey(ent, key, buf.as_mut_ptr(), 256) == 0 { return 0; } // qfalse
    //scanf into doubles, then assign, so it is vec_t size independent
    v1 = 0.0;
    v2 = 0.0;
    v3 = 0.0;
    sscanf(buf.as_ptr(), b"%lf %lf %lf\0" as *const u8 as *const c_char, &mut v1, &mut v2, &mut v3);
    *v.offset(0) = v1 as f32;
    *v.offset(1) = v2 as f32;
    *v.offset(2) = v3 as f32;
    return 1; // qtrue
} //end of the function AAS_VectorForBSPEpairKey
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_FloatForBSPEpairKey(ent: c_int, key: *const c_char, value: *mut f32) -> c_int
{
    let mut buf: [c_char; 256] = [0; 256];

    *value = 0.0;
    if AAS_ValueForBSPEpairKey(ent, key, buf.as_mut_ptr(), 256) == 0 { return 0; } // qfalse
    *value = atof(buf.as_ptr()) as f32;
    return 1; // qtrue
} //end of the function AAS_FloatForBSPEpairKey
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_IntForBSPEpairKey(ent: c_int, key: *const c_char, value: *mut c_int) -> c_int
{
    let mut buf: [c_char; 256] = [0; 256];

    *value = 0;
    if AAS_ValueForBSPEpairKey(ent, key, buf.as_mut_ptr(), 256) == 0 { return 0; } // qfalse
    *value = atoi(buf.as_ptr());
    return 1; // qtrue
} //end of the function AAS_IntForBSPEpairKey
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_FreeBSPEntities()
{
    let mut i: c_int = 1;
    let mut ent: *mut bsp_entity_t;
    let mut epair: *mut bsp_epair_t;
    let mut nextepair: *mut bsp_epair_t;

    while i < bspworld.numentities
    {
        ent = &mut bspworld.entities[i as usize];
        epair = (*ent).epairs;
        while epair != ptr::null_mut()
        {
            nextepair = (*epair).next;
            //
            if (*epair).key != ptr::null_mut() { FreeMemory((*epair).key as *mut c_void); }
            if (*epair).value != ptr::null_mut() { FreeMemory((*epair).value as *mut c_void); }
            FreeMemory(epair as *mut c_void);
            epair = nextepair;
        } //end for
        i += 1;
    } //end for
    bspworld.numentities = 0;
} //end of the function AAS_FreeBSPEntities
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_ParseBSPEntities()
{
    let mut script: *mut script_t;
    let mut token: token_t = core::mem::zeroed();
    let mut ent: *mut bsp_entity_t;
    let mut epair: *mut bsp_epair_t;

    script = LoadScriptMemory(bspworld.dentdata, bspworld.entdatasize, b"entdata\0" as *const u8 as *const c_char);
    SetScriptFlags(script, SCFL_NOSTRINGWHITESPACES|SCFL_NOSTRINGESCAPECHARS);//SCFL_PRIMITIVE);

    bspworld.numentities = 1;

    while PS_ReadToken(script, &mut token) != 0
    {
        if strcmp(token.string.as_ptr(), b"{\0" as *const u8 as *const c_char) != 0
        {
            ScriptError(script, b"invalid %s\n\0" as *const u8 as *const c_char, token.string.as_ptr());
            AAS_FreeBSPEntities();
            FreeScript(script);
            return;
        } //end if
        if bspworld.numentities >= MAX_BSPENTITIES as c_int
        {
            let print_fn: PrintFn2 = core::mem::transmute(botimport.Print);
            print_fn(PRT_MESSAGE, b"too many entities in BSP file\n\0" as *const u8 as *const c_char);
            break;
        } //end if
        ent = &mut bspworld.entities[bspworld.numentities as usize];
        bspworld.numentities += 1;
        (*ent).epairs = ptr::null_mut();
        while PS_ReadToken(script, &mut token) != 0
        {
            if strcmp(token.string.as_ptr(), b"}\0" as *const u8 as *const c_char) == 0 { break; }
            epair = GetClearedHunkMemory(core::mem::size_of::<bsp_epair_t>()) as *mut bsp_epair_t;
            (*epair).next = (*ent).epairs;
            (*ent).epairs = epair;
            if token.type_ != TT_STRING
            {
                ScriptError(script, b"invalid %s\n\0" as *const u8 as *const c_char, token.string.as_ptr());
                AAS_FreeBSPEntities();
                FreeScript(script);
                return;
            } //end if
            StripDoubleQuotes(token.string.as_mut_ptr());
            (*epair).key = GetHunkMemory(strlen(token.string.as_ptr()) + 1) as *mut c_char;
            strcpy((*epair).key, token.string.as_ptr());
            if PS_ExpectTokenType(script, TT_STRING, 0, &mut token) == 0
            {
                AAS_FreeBSPEntities();
                FreeScript(script);
                return;
            } //end if
            StripDoubleQuotes(token.string.as_mut_ptr());
            (*epair).value = GetHunkMemory(strlen(token.string.as_ptr()) + 1) as *mut c_char;
            strcpy((*epair).value, token.string.as_ptr());
        } //end while
        if strcmp(token.string.as_ptr(), b"}\0" as *const u8 as *const c_char) != 0
        {
            ScriptError(script, b"missing }\n\0" as *const u8 as *const c_char);
            AAS_FreeBSPEntities();
            FreeScript(script);
            return;
        } //end if
    } //end while
    FreeScript(script);
} //end of the function AAS_ParseBSPEntities
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_BSPTraceLight(start: *mut f32, end: *mut f32, endpos: *mut f32, red: *mut c_int, green: *mut c_int, blue: *mut c_int) -> c_int
{
    return 0;
} //end of the function AAS_BSPTraceLight
//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_DumpBSPData()
{
    AAS_FreeBSPEntities();

    if bspworld.dentdata != ptr::null_mut() { FreeMemory(bspworld.dentdata as *mut c_void); }
    bspworld.dentdata = ptr::null_mut();
    bspworld.entdatasize = 0;
    //
    bspworld.loaded = 0; // qfalse
    Com_Memset(addr_of_mut!(bspworld) as *mut c_void, 0, core::mem::size_of::<bsp_t>());
} //end of the function AAS_DumpBSPData
//===========================================================================
// load an bsp file
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe fn AAS_LoadBSPFile() -> c_int
{
    AAS_DumpBSPData();
    bspworld.entdatasize = (strlen((botimport.BSPEntityData)()) + 1) as c_int;
    bspworld.dentdata = GetClearedHunkMemory(bspworld.entdatasize as usize) as *mut c_char;
    Com_Memcpy(bspworld.dentdata as *mut c_void, (botimport.BSPEntityData)() as *const c_void, bspworld.entdatasize as usize);
    AAS_ParseBSPEntities();
    bspworld.loaded = 1; // qtrue
    return BLERR_NOERROR;
} //end of the function AAS_LoadBSPFile

// Forward declaration for bsp_link_t stub type (used in unused function parameters)
#[repr(C)]
pub struct bsp_link_t;
