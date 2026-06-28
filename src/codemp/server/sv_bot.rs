// sv_bot.c
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "server.h"
// #include "../game/botlib.h"

use core::ffi::{c_int, c_char, c_void};
use core::mem;

// Stub includes/imports - these would come from linked modules/extern blocks
// For now, we declare externs as needed

#[repr(C)]
pub struct bot_debugpoly_s {
    pub inuse: c_int,
    pub color: c_int,
    pub numPoints: c_int,
    pub points: [[f32; 3]; 128],
}

pub type bot_debugpoly_t = bot_debugpoly_s;

pub static mut debugpolygons: *mut bot_debugpoly_t = core::ptr::null_mut();
pub static mut bot_maxdebugpolys: c_int = 0;

extern "C" {
    pub static mut botlib_export: *mut botlib_export_t;
    pub static mut bot_enable: c_int;
}

pub static mut gWPNum: c_int = 0;
pub static mut gWPArray: [*mut wpobject_t; 64] = [core::ptr::null_mut(); 64]; // MAX_WPARRAY_SIZE

#[repr(C)]
pub struct botlib_export_t {
    // Stub: actual fields depend on botlib.h
}

#[repr(C)]
pub struct wpobject_t {
    // Stub: actual fields depend on context
    pub inuse: c_int,
    pub origin: [f32; 3],
    pub neighbornum: c_int,
    pub neighbors: [neighbor_t; 256], // MAX_NEIGHBOR_SIZE
}

#[repr(C)]
pub struct neighbor_t {
    pub num: c_int,
    pub forceJumpTo: c_int,
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: cplane_t,
    pub surfaceFlags: c_int,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct cplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
}

#[repr(C)]
pub struct bsp_trace_t {
    pub allsolid: c_int, // qboolean
    pub startsolid: c_int, // qboolean
    pub fraction: f32,
    pub endpos: [f32; 3],
    pub plane: bsp_plane_t,
    pub surface: bsp_surface_t,
    pub ent: c_int,
    pub exp_dist: f32,
    pub sidenum: c_int,
    pub contents: c_int,
}

#[repr(C)]
pub struct bsp_plane_t {
    pub normal: [f32; 3],
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
}

#[repr(C)]
pub struct bsp_surface_t {
    pub value: c_int,
}

#[repr(C)]
pub struct clipHandle_t(c_int);

#[repr(C)]
pub struct client_t {
    // Stub fields
    pub state: c_int,
    pub gentity: *mut gentity_t,
    pub lastPacketTime: c_int,
    pub netchan: netchan_t,
    pub rate: c_int,
    pub name: [c_char; 32],
    pub reliableAcknowledge: c_int,
    pub reliableSequence: c_int,
    pub reliableCommands: [[c_char; 1024]; 128],
    pub frames: [clientSnapshot_t; 4],
}

#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub r: gentityRef_t,
}

#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
}

#[repr(C)]
pub struct gentityRef_t {
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
    pub svFlags: c_int,
}

#[repr(C)]
pub struct netchan_t {
    pub remoteAddress: netadr_t,
    pub outgoingSequence: c_int,
}

#[repr(C)]
pub struct netadr_t {
    pub type_: c_int,
}

#[repr(C)]
pub struct clientSnapshot_t {
    pub num_entities: c_int,
    pub first_entity: c_int,
}

extern "C" {
    pub static mut svs: serverStatic_t;
    pub static mut sv_maxclients: *mut cvar_t;
    pub static mut gvm: *mut c_void;

    pub fn SV_Trace(
        results: *mut trace_t,
        start: [f32; 3],
        mins: *const [f32; 3],
        maxs: *const [f32; 3],
        end: [f32; 3],
        passent: c_int,
        contentmask: c_int,
        skip1: c_int,
        skip2: c_int,
        skip3: c_int,
    );

    pub fn SV_ClipToEntity(
        results: *mut trace_t,
        start: [f32; 3],
        mins: *const [f32; 3],
        maxs: *const [f32; 3],
        end: [f32; 3],
        entnum: c_int,
        contentmask: c_int,
        skip: c_int,
    );

    pub fn SV_PointContents(point: [f32; 3], passent: c_int) -> c_int;
    pub fn SV_inPVS(p1: [f32; 3], p2: [f32; 3]) -> c_int;
    pub fn CM_EntityString() -> *mut c_char;
    pub fn CM_InlineModel(num: c_int) -> clipHandle_t;
    pub fn CM_ModelBounds(model: clipHandle_t, mins: *mut [f32; 3], maxs: *mut [f32; 3]);

    pub fn RadiusFromBounds(mins: [f32; 3], maxs: [f32; 3]) -> f32;

    pub fn VectorSubtract(a: [f32; 3], b: [f32; 3], out: *mut [f32; 3]);
    pub fn VectorLength(v: [f32; 3]) -> f32;
    pub fn VectorCopy(src: [f32; 3], dst: *mut [f32; 3]);
    pub fn VectorClear(v: *mut [f32; 3]);
    pub fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    pub fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    pub fn VectorMA(v: *mut [f32; 3], scale: f32, dir: [f32; 3], out: *mut [f32; 3]);
    pub fn DotProduct(a: [f32; 3], b: [f32; 3]) -> f32;
    pub fn CrossProduct(a: [f32; 3], b: [f32; 3], out: *mut [f32; 3]);

    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Memcpy(dst: *mut c_void, src: *const c_void, count: usize);

    pub fn Cvar_Get(var_name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    pub fn Cvar_VariableIntegerValue(var_name: *const c_char) -> c_int;

    pub fn Z_Malloc(size: usize, tag: c_int, qb: c_int) -> *mut c_void;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Z_MemSize(tag: c_int) -> usize;

    pub fn Hunk_CheckMark() -> c_int;
    pub fn Hunk_Alloc(size: usize, h: c_int) -> *mut c_void;

    pub fn FS_FOpenFileByMode(
        filename: *const c_char,
        handle: *mut *mut c_void,
        mode: c_int,
    ) -> c_int;
    pub fn FS_Read2(buffer: *mut c_void, len: c_int, f: *mut c_void) -> c_int;
    pub fn FS_Write(buffer: *const c_void, len: c_int, f: *mut c_void) -> c_int;
    pub fn FS_FCloseFile(f: *mut c_void);
    pub fn FS_Seek(f: *mut c_void, offset: c_int, origin: c_int) -> c_int;

    pub fn SV_GentityNum(num: c_int) -> *mut gentity_t;
    pub fn SV_ExecuteClientCommand(client: *mut client_t, command: *const c_char, clientOK: c_int);

    pub fn VM_Call(vm: *mut c_void, callnum: c_int, ...) -> c_int;

    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);

    pub fn GetBotLibAPI(
        apiVersion: c_int,
        import: *mut botlib_import_t,
    ) -> *mut botlib_export_t;
}

#[repr(C)]
pub struct serverStatic_t {
    pub clients: *mut client_t,
    pub time: c_int,
    pub numSnapshotEntities: c_int,
    pub snapshotEntities: *mut entitySnapshot_t,
}

#[repr(C)]
pub struct entitySnapshot_t {
    pub number: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    pub string: *mut c_char,
}

pub type va_list = *mut c_void;

#[repr(C)]
pub struct botlib_import_t {
    pub Print: Option<extern "C" fn(c_int, *mut c_char, ...)>,
    pub Trace: Option<extern "C" fn(*mut bsp_trace_t, [f32; 3], [f32; 3], [f32; 3], [f32; 3], c_int, c_int)>,
    pub EntityTrace: Option<extern "C" fn(*mut bsp_trace_t, [f32; 3], [f32; 3], [f32; 3], [f32; 3], c_int, c_int)>,
    pub PointContents: Option<extern "C" fn([f32; 3]) -> c_int>,
    pub inPVS: Option<extern "C" fn([f32; 3], [f32; 3]) -> c_int>,
    pub BSPEntityData: Option<extern "C" fn() -> *mut c_char>,
    pub BSPModelMinsMaxsOrigin: Option<extern "C" fn(c_int, [f32; 3], *mut [f32; 3], *mut [f32; 3], *mut [f32; 3])>,
    pub BotClientCommand: Option<extern "C" fn(c_int, *mut c_char)>,
    pub GetMemory: Option<extern "C" fn(c_int) -> *mut c_void>,
    pub FreeMemory: Option<extern "C" fn(*mut c_void)>,
    pub AvailableMemory: Option<extern "C" fn() -> c_int>,
    pub HunkAlloc: Option<extern "C" fn(c_int) -> *mut c_void>,
    pub FS_FOpenFile: Option<extern "C" fn(*const c_char, *mut *mut c_void, c_int) -> c_int>,
    pub FS_Read: Option<extern "C" fn(*mut c_void, c_int, *mut c_void) -> c_int>,
    pub FS_Write: Option<extern "C" fn(*const c_void, c_int, *mut c_void) -> c_int>,
    pub FS_FCloseFile: Option<extern "C" fn(*mut c_void)>,
    pub FS_Seek: Option<extern "C" fn(*mut c_void, c_int, c_int) -> c_int>,
    pub DebugLineCreate: Option<extern "C" fn() -> c_int>,
    pub DebugLineDelete: Option<extern "C" fn(c_int)>,
    pub DebugLineShow: Option<extern "C" fn(c_int, [f32; 3], [f32; 3], c_int)>,
    pub DebugPolygonCreate: Option<extern "C" fn(c_int, c_int, *mut [f32; 3]) -> c_int>,
    pub DebugPolygonDelete: Option<extern "C" fn(c_int)>,
}

// Constants (from the original file's context)
const MAX_WPARRAY_SIZE: usize = 64;
const MAX_NEIGHBOR_LINK_DISTANCE: c_int = 300;
const DEFAULT_GRID_SPACING: c_int = 100;
const MAX_NEIGHBOR_SIZE: usize = 256;
const MASK_SOLID: c_int = 1;
const ENTITYNUM_NONE: c_int = 1024;
const CS_FREE: c_int = 0;
const CS_ACTIVE: c_int = 1;
const NA_BOT: c_int = 3;
const BUTTON_ATTACK: c_int = 1;
const ERR_DROP: c_int = 1;
const CVAR_CHEAT: c_int = 16;
const TAG_BOTGAME: c_int = 11;
const TAG_BOTLIB: c_int = 12;
const PRT_MESSAGE: c_int = 0;
const PRT_WARNING: c_int = 1;
const PRT_ERROR: c_int = 2;
const PRT_FATAL: c_int = 3;
const PRT_EXIT: c_int = 4;
const BOTLIB_API_VERSION: c_int = 2;
const MAX_RELIABLE_COMMANDS: c_int = 64;
const PACKET_MASK: c_int = 63;
const SVF_BOT: c_int = 1;

const S_COLOR_RED: &[u8] = b"^1";
const S_COLOR_YELLOW: &[u8] = b"^3";

const qfalse: c_int = 0;
const qtrue: c_int = 1;

// Local stub for BotVMShift - would be defined elsewhere
extern "C" {
    pub fn BotVMShift(ptr: c_int) -> *mut c_void;
}

fn NotWithinRange(base: c_int, extent: c_int) -> c_int {
    if extent > base && base + 5 >= extent {
        return 0;
    }

    if extent < base && base - 5 <= extent {
        return 0;
    }

    return 1;
}

pub fn SV_OrgVisibleBox(
    org1: [f32; 3],
    mins: [f32; 3],
    maxs: [f32; 3],
    org2: [f32; 3],
    ignore: c_int,
    rmg: c_int,
) -> c_int {
    let mut tr: trace_t = unsafe { mem::zeroed() };

    unsafe {
        if rmg != 0 {
            SV_Trace(
                &mut tr,
                org1,
                core::ptr::null(),
                core::ptr::null(),
                org2,
                ignore,
                MASK_SOLID,
                0,
                0,
                10,
            );
        } else {
            SV_Trace(
                &mut tr,
                org1,
                &mins,
                &maxs,
                org2,
                ignore,
                MASK_SOLID,
                0,
                0,
                10,
            );
        }
    }

    if tr.fraction == 1.0 && tr.startsolid == 0 && tr.allsolid == 0 {
        return 1;
    }

    return 0;
}

pub fn SV_BotWaypointReception(wpnum: c_int, wps: *mut *mut wpobject_t) {
    let mut i: c_int = 0;

    unsafe {
        gWPNum = wpnum;

        while i < gWPNum {
            gWPArray[i as usize] = BotVMShift(*wps.offset(i as isize) as c_int) as *mut wpobject_t;
            i += 1;
        }
    }
}

/*
==================
SV_BotCalculatePaths
==================
*/
pub fn SV_BotCalculatePaths(rmg: c_int) {
    let mut i: c_int;
    let mut c: c_int;
    let mut forceJumpable: c_int;
    let mut maxNeighborDist: c_int = MAX_NEIGHBOR_LINK_DISTANCE;
    let mut nLDist: f32;
    let mut a: [f32; 3];
    let mut mins: [f32; 3];
    let mut maxs: [f32; 3];

    unsafe {
        if gWPNum == 0 {
            return;
        }

        if rmg != 0 {
            maxNeighborDist = DEFAULT_GRID_SPACING + (DEFAULT_GRID_SPACING as f32 * 0.5) as c_int;
        }

        mins[0] = -15.0;
        mins[1] = -15.0;
        mins[2] = -15.0; //-1
        maxs[0] = 15.0;
        maxs[1] = 15.0;
        maxs[2] = 15.0; //1

        //now clear out all the neighbor data before we recalculate
        i = 0;

        while i < gWPNum {
            if !gWPArray[i as usize].is_null()
                && (*gWPArray[i as usize]).inuse != 0
                && (*gWPArray[i as usize]).neighbornum != 0
            {
                while (*gWPArray[i as usize]).neighbornum >= 0 {
                    (*gWPArray[i as usize]).neighbors
                        [(*gWPArray[i as usize]).neighbornum as usize]
                        .num = 0;
                    (*gWPArray[i as usize]).neighbors
                        [(*gWPArray[i as usize]).neighbornum as usize]
                        .forceJumpTo = 0;
                    (*gWPArray[i as usize]).neighbornum -= 1;
                }
                (*gWPArray[i as usize]).neighbornum = 0;
            }

            i += 1;
        }

        i = 0;

        while i < gWPNum {
            if !gWPArray[i as usize].is_null() && (*gWPArray[i as usize]).inuse != 0 {
                c = 0;

                while c < gWPNum {
                    if !gWPArray[c as usize].is_null()
                        && (*gWPArray[c as usize]).inuse != 0
                        && i != c
                        && NotWithinRange(i, c) != 0
                    {
                        let origin_i = (*gWPArray[i as usize]).origin;
                        let origin_c = (*gWPArray[c as usize]).origin;
                        a = [
                            origin_i[0] - origin_c[0],
                            origin_i[1] - origin_c[1],
                            origin_i[2] - origin_c[2],
                        ];

                        nLDist = VectorLength(a);
                        forceJumpable = qfalse; //CanForceJumpTo(i, c, nLDist);

                        if ((nLDist < maxNeighborDist as f32 || forceJumpable != 0)
                            && ((origin_i[2] as c_int) == (origin_c[2] as c_int) || forceJumpable != 0)
                            && (SV_OrgVisibleBox(origin_i, mins, maxs, origin_c, ENTITYNUM_NONE, rmg) != 0
                                || forceJumpable != 0))
                        {
                            (*gWPArray[i as usize]).neighbors
                                [(*gWPArray[i as usize]).neighbornum as usize]
                                .num = c;
                            if forceJumpable != 0
                                && ((origin_i[2] as c_int) != (origin_c[2] as c_int)
                                    || nLDist < maxNeighborDist as f32)
                            {
                                (*gWPArray[i as usize]).neighbors
                                    [(*gWPArray[i as usize]).neighbornum as usize]
                                    .forceJumpTo = 999; //forceJumpable; //FJSR
                            } else {
                                (*gWPArray[i as usize]).neighbors
                                    [(*gWPArray[i as usize]).neighbornum as usize]
                                    .forceJumpTo = 0;
                            }
                            (*gWPArray[i as usize]).neighbornum += 1;
                        }

                        if (*gWPArray[i as usize]).neighbornum >= MAX_NEIGHBOR_SIZE as c_int {
                            break;
                        }
                    }
                    c += 1;
                }
            }
            i += 1;
        }
    }
}

/*
==================
SV_BotAllocateClient
==================
*/
pub fn SV_BotAllocateClient() -> c_int {
    let mut i: c_int;
    let mut cl: *mut client_t;

    unsafe {
        // find a client slot
        i = 0;
        cl = svs.clients;
        while i < (*sv_maxclients).integer {
            if (*cl).state == CS_FREE {
                break;
            }
            cl = cl.offset(1);
            i += 1;
        }

        if i == (*sv_maxclients).integer {
            return -1;
        }

        (*cl).gentity = SV_GentityNum(i);
        (*(*cl).gentity).s.number = i;
        (*cl).state = CS_ACTIVE;
        (*cl).lastPacketTime = svs.time;
        (*cl).netchan.remoteAddress.type_ = NA_BOT;
        (*cl).rate = 16384;

        return i;
    }
}

/*
==================
SV_BotFreeClient
==================
*/
pub fn SV_BotFreeClient(clientNum: c_int) {
    let mut cl: *mut client_t;

    unsafe {
        if clientNum < 0 || clientNum >= (*sv_maxclients).integer {
            Com_Error(
                ERR_DROP,
                b"SV_BotFreeClient: bad clientNum: %i\0".as_ptr() as *const c_char,
                clientNum,
            );
        }
        cl = &mut (*svs.clients.offset(clientNum as isize));
        (*cl).state = CS_FREE;
        (*cl).name[0] = 0;
        if !(*cl).gentity.is_null() {
            (*(*cl).gentity).r.svFlags &= !SVF_BOT;
        }
    }
}

/*
==================
BotDrawDebugPolygons
==================
*/
pub fn BotDrawDebugPolygons(
    drawPoly: Option<extern "C" fn(c_int, c_int, *mut f32)>,
    value: c_int,
) {
    static mut bot_debug: *mut cvar_t = core::ptr::null_mut();
    static mut bot_groundonly: *mut cvar_t = core::ptr::null_mut();
    static mut bot_reachability: *mut cvar_t = core::ptr::null_mut();
    static mut bot_highlightarea: *mut cvar_t = core::ptr::null_mut();

    let mut poly: *mut bot_debugpoly_t;
    let mut i: c_int;
    let mut parm0: c_int;

    unsafe {
        if debugpolygons.is_null() {
            return;
        }
        //bot debugging
        if bot_debug.is_null() {
            bot_debug = Cvar_Get(b"bot_debug\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        }
        //
        if bot_enable != 0 && (*bot_debug).integer != 0 {
            //show reachabilities
            if bot_reachability.is_null() {
                bot_reachability = Cvar_Get(
                    b"bot_reachability\0".as_ptr() as *const c_char,
                    b"0\0".as_ptr() as *const c_char,
                    0,
                );
            }
            //show ground faces only
            if bot_groundonly.is_null() {
                bot_groundonly = Cvar_Get(
                    b"bot_groundonly\0".as_ptr() as *const c_char,
                    b"1\0".as_ptr() as *const c_char,
                    0,
                );
            }
            //get the hightlight area
            if bot_highlightarea.is_null() {
                bot_highlightarea = Cvar_Get(
                    b"bot_highlightarea\0".as_ptr() as *const c_char,
                    b"0\0".as_ptr() as *const c_char,
                    0,
                );
            }
            //
            parm0 = 0;
            if (*svs.clients).lastPacketTime & BUTTON_ATTACK != 0 {
                parm0 |= 1;
            }
            if (*bot_reachability).integer != 0 {
                parm0 |= 2;
            }
            if (*bot_groundonly).integer != 0 {
                parm0 |= 4;
            }
            (*botlib_export)
                .BotLibVarSet(
                    b"bot_highlightarea\0".as_ptr() as *const c_char,
                    (*bot_highlightarea).string,
                );
            (*botlib_export).Test(
                parm0,
                core::ptr::null_mut(),
                (*(*svs.clients).gentity).r.currentOrigin,
                (*(*svs.clients).gentity).r.currentAngles,
            );
        } //end if
          //draw all debug polys
        i = 0;
        while i < bot_maxdebugpolys {
            poly = &mut (*debugpolygons).offset(i as isize);
            if (*poly).inuse == 0 {
                i += 1;
                continue;
            }
            if let Some(f) = drawPoly {
                f(
                    (*poly).color,
                    (*poly).numPoints,
                    (*poly).points.as_mut_ptr() as *mut f32,
                );
            }
            //Com_Printf("poly %i, numpoints = %d\n", i, poly->numPoints);
            i += 1;
        }
    }
}

/*
==================
BotImport_Print
==================
*/
pub extern "C" fn BotImport_Print(type_: c_int, fmt: *mut c_char, ...) {
    let mut str_buf: [u8; 2048] = [0; 2048];
    let mut ap: va_list = core::ptr::null_mut();

    // Stub: actual vsprintf would require C va_list handling
    // For now, just forward to Com_Printf with the format

    unsafe {
        match type_ {
            PRT_MESSAGE => {
                Com_Printf(b"%s\0".as_ptr() as *const c_char, &str_buf as *const u8);
            }
            PRT_WARNING => {
                Com_Printf(
                    b"^3Warning: %s\0".as_ptr() as *const c_char,
                    &str_buf as *const u8,
                );
            }
            PRT_ERROR => {
                Com_Printf(
                    b"^1Error: %s\0".as_ptr() as *const c_char,
                    &str_buf as *const u8,
                );
            }
            PRT_FATAL => {
                Com_Printf(
                    b"^1Fatal: %s\0".as_ptr() as *const c_char,
                    &str_buf as *const u8,
                );
            }
            PRT_EXIT => {
                Com_Error(
                    ERR_DROP,
                    b"^1Exit: %s\0".as_ptr() as *const c_char,
                    &str_buf as *const u8,
                );
            }
            _ => {
                Com_Printf(b"unknown print type\n\0".as_ptr() as *const c_char);
            }
        }
    }
}

/*
==================
BotImport_Trace
==================
*/
pub extern "C" fn BotImport_Trace(
    bsptrace: *mut bsp_trace_t,
    start: [f32; 3],
    mins: [f32; 3],
    maxs: [f32; 3],
    end: [f32; 3],
    passent: c_int,
    contentmask: c_int,
) {
    let mut trace: trace_t = unsafe { mem::zeroed() };

    unsafe {
        SV_Trace(
            &mut trace,
            start,
            &mins,
            &maxs,
            end,
            passent,
            contentmask,
            qfalse,
            0,
            10,
        );
        //copy the trace information
        (*bsptrace).allsolid = trace.allsolid as c_int;
        (*bsptrace).startsolid = trace.startsolid as c_int;
        (*bsptrace).fraction = trace.fraction;
        (*bsptrace).endpos = trace.endpos;
        (*bsptrace).plane.dist = trace.plane.dist;
        (*bsptrace).plane.normal = trace.plane.normal;
        (*bsptrace).plane.signbits = trace.plane.signbits;
        (*bsptrace).plane.type_ = trace.plane.type_;
        (*bsptrace).surface.value = trace.surfaceFlags;
        (*bsptrace).ent = trace.entityNum;
        (*bsptrace).exp_dist = 0.0;
        (*bsptrace).sidenum = 0;
        (*bsptrace).contents = 0;
    }
}

/*
==================
BotImport_EntityTrace
==================
*/
pub extern "C" fn BotImport_EntityTrace(
    bsptrace: *mut bsp_trace_t,
    start: [f32; 3],
    mins: [f32; 3],
    maxs: [f32; 3],
    end: [f32; 3],
    entnum: c_int,
    contentmask: c_int,
) {
    let mut trace: trace_t = unsafe { mem::zeroed() };

    unsafe {
        SV_ClipToEntity(&mut trace, start, &mins, &maxs, end, entnum, contentmask, qfalse);
        //copy the trace information
        (*bsptrace).allsolid = trace.allsolid as c_int;
        (*bsptrace).startsolid = trace.startsolid as c_int;
        (*bsptrace).fraction = trace.fraction;
        (*bsptrace).endpos = trace.endpos;
        (*bsptrace).plane.dist = trace.plane.dist;
        (*bsptrace).plane.normal = trace.plane.normal;
        (*bsptrace).plane.signbits = trace.plane.signbits;
        (*bsptrace).plane.type_ = trace.plane.type_;
        (*bsptrace).surface.value = trace.surfaceFlags;
        (*bsptrace).ent = trace.entityNum;
        (*bsptrace).exp_dist = 0.0;
        (*bsptrace).sidenum = 0;
        (*bsptrace).contents = 0;
    }
}

/*
==================
BotImport_PointContents
==================
*/
pub extern "C" fn BotImport_PointContents(point: [f32; 3]) -> c_int {
    unsafe { SV_PointContents(point, -1) }
}

/*
==================
BotImport_inPVS
==================
*/
pub extern "C" fn BotImport_inPVS(p1: [f32; 3], p2: [f32; 3]) -> c_int {
    unsafe { SV_inPVS(p1, p2) }
}

/*
==================
BotImport_BSPEntityData
==================
*/
pub extern "C" fn BotImport_BSPEntityData() -> *mut c_char {
    unsafe { CM_EntityString() }
}

/*
==================
BotImport_BSPModelMinsMaxsOrigin
==================
*/
pub extern "C" fn BotImport_BSPModelMinsMaxsOrigin(
    modelnum: c_int,
    angles: [f32; 3],
    outmins: *mut [f32; 3],
    outmaxs: *mut [f32; 3],
    origin: *mut [f32; 3],
) {
    let mut h: clipHandle_t;
    let mut mins: [f32; 3];
    let mut maxs: [f32; 3];
    let mut max: f32;
    let mut i: c_int;

    unsafe {
        h = CM_InlineModel(modelnum);
        CM_ModelBounds(h, &mut mins, &mut maxs);
        //if the model is rotated
        if angles[0] != 0.0 || angles[1] != 0.0 || angles[2] != 0.0 {
            // expand for rotation

            max = RadiusFromBounds(mins, maxs);
            i = 0;
            while i < 3 {
                mins[i as usize] = -max;
                maxs[i as usize] = max;
                i += 1;
            }
        }
        if !outmins.is_null() {
            VectorCopy(mins, outmins);
        }
        if !outmaxs.is_null() {
            VectorCopy(maxs, outmaxs);
        }
        if !origin.is_null() {
            VectorClear(origin);
        }
    }
}

/*
==================
BotImport_GetMemoryGame
==================
*/
// #ifndef _XBOX	// These are unused, I want the tag back
pub extern "C" fn Bot_GetMemoryGame(size: c_int) -> *mut c_void {
    unsafe { Z_Malloc(size as usize, TAG_BOTGAME, qtrue) }
}

/*
==================
BotImport_FreeMemoryGame
==================
*/
pub extern "C" fn Bot_FreeMemoryGame(ptr: *mut c_void) {
    unsafe {
        Z_Free(ptr);
    }
}
// #endif
/*
==================
BotImport_GetMemory
==================
*/
pub extern "C" fn BotImport_GetMemory(size: c_int) -> *mut c_void {
    unsafe { Z_Malloc(size as usize, TAG_BOTLIB, qtrue) }
}

/*
==================
BotImport_FreeMemory
==================
*/
pub extern "C" fn BotImport_FreeMemory(ptr: *mut c_void) {
    unsafe {
        Z_Free(ptr);
    }
}

/*
=================
BotImport_HunkAlloc
=================
*/
pub extern "C" fn BotImport_HunkAlloc(size: c_int) -> *mut c_void {
    unsafe {
        if Hunk_CheckMark() != 0 {
            Com_Error(
                ERR_DROP,
                b"SV_Bot_HunkAlloc: Alloc with marks already set\n\0".as_ptr() as *const c_char,
            );
        }
        Hunk_Alloc(size as usize, 1) // h_high
    }
}

/*
==================
BotImport_DebugPolygonCreate
==================
*/
pub extern "C" fn BotImport_DebugPolygonCreate(
    color: c_int,
    numPoints: c_int,
    points: *mut [f32; 3],
) -> c_int {
    let mut poly: *mut bot_debugpoly_t;
    let mut i: c_int;

    unsafe {
        if debugpolygons.is_null() {
            return 0;
        }

        i = 1;
        while i < bot_maxdebugpolys {
            if (*debugpolygons.offset(i as isize)).inuse == 0 {
                break;
            }
            i += 1;
        }
        if i >= bot_maxdebugpolys {
            return 0;
        }
        poly = &mut (*debugpolygons).offset(i as isize);
        (*poly).inuse = qtrue;
        (*poly).color = color;
        (*poly).numPoints = numPoints;
        Com_Memcpy(
            &mut (*poly).points as *mut _ as *mut c_void,
            points as *const c_void,
            (numPoints as usize) * mem::size_of::<[f32; 3]>(),
        );
        //
        return i;
    }
}

/*
==================
BotImport_DebugPolygonShow
==================
*/
pub extern "C" fn BotImport_DebugPolygonShow(
    id: c_int,
    color: c_int,
    numPoints: c_int,
    points: *mut [f32; 3],
) {
    let mut poly: *mut bot_debugpoly_t;

    unsafe {
        if debugpolygons.is_null() {
            return;
        }
        poly = &mut (*debugpolygons).offset(id as isize);
        (*poly).inuse = qtrue;
        (*poly).color = color;
        (*poly).numPoints = numPoints;
        Com_Memcpy(
            &mut (*poly).points as *mut _ as *mut c_void,
            points as *const c_void,
            (numPoints as usize) * mem::size_of::<[f32; 3]>(),
        );
    }
}

/*
==================
BotImport_DebugPolygonDelete
==================
*/
pub extern "C" fn BotImport_DebugPolygonDelete(id: c_int) {
    unsafe {
        if debugpolygons.is_null() {
            return;
        }
        (*debugpolygons.offset(id as isize)).inuse = qfalse;
    }
}

/*
==================
BotImport_DebugLineCreate
==================
*/
pub extern "C" fn BotImport_DebugLineCreate() -> c_int {
    let points: [[f32; 3]; 1] = [[0.0, 0.0, 0.0]];
    BotImport_DebugPolygonCreate(0, 0, &points[0] as *const _ as *mut _)
}

/*
==================
BotImport_DebugLineDelete
==================
*/
pub extern "C" fn BotImport_DebugLineDelete(line: c_int) {
    BotImport_DebugPolygonDelete(line);
}

/*
==================
BotImport_DebugLineShow
==================
*/
pub extern "C" fn BotImport_DebugLineShow(line: c_int, start: [f32; 3], end: [f32; 3], color: c_int) {
    let mut points: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ];
    let mut dir: [f32; 3];
    let mut cross: [f32; 3];
    let up: [f32; 3] = [0.0, 0.0, 1.0];
    let mut dot: f32;

    points[0] = start;
    points[1] = start;
    //points[1][2] -= 2;
    points[2] = end;
    //points[2][2] -= 2;
    points[3] = end;

    unsafe {
        dir = [
            end[0] - start[0],
            end[1] - start[1],
            end[2] - start[2],
        ];
        VectorNormalize(&mut dir);
        dot = DotProduct(dir, up);
        if dot > 0.99 || dot < -0.99 {
            cross = [1.0, 0.0, 0.0];
        } else {
            CrossProduct(dir, up, &mut cross);
        }

        VectorNormalize(&mut cross);

        VectorMA(&mut points[0], 2.0, cross, &mut points[0]);
        VectorMA(&mut points[1], -2.0, cross, &mut points[1]);
        VectorMA(&mut points[2], -2.0, cross, &mut points[2]);
        VectorMA(&mut points[3], 2.0, cross, &mut points[3]);

        BotImport_DebugPolygonShow(line, color, 4, &mut points[0]);
    }
}

/*
==================
SV_BotClientCommand
==================
*/
pub extern "C" fn BotClientCommand(client: c_int, command: *mut c_char) {
    unsafe {
        SV_ExecuteClientCommand(&mut *svs.clients.offset(client as isize), command, qtrue);
    }
}

/*
==================
SV_BotFrame
==================
*/
pub fn SV_BotFrame(time: c_int) {
    unsafe {
        if bot_enable == 0 {
            return;
        }
        //NOTE: maybe the game is already shutdown
        if gvm.is_null() {
            return;
        }
        VM_Call(gvm, 1, time); // BOTAI_START_FRAME
    }
}

/*
===============
SV_BotLibSetup
===============
*/
pub fn SV_BotLibSetup() -> c_int {
    unsafe {
        if bot_enable == 0 {
            return 0;
        }

        if botlib_export.is_null() {
            Com_Printf(b"^1Error: SV_BotLibSetup without SV_BotInitBotLib\n\0".as_ptr() as *const c_char);
            return -1;
        }

        // Stub call - actual function pointer table would be used
        return 0;
    }
}

/*
===============
SV_ShutdownBotLib

Called when either the entire server is being killed, or
it is changing to a different game directory.
===============
*/
pub fn SV_BotLibShutdown() -> c_int {
    unsafe {
        if botlib_export.is_null() {
            return -1;
        }

        // Stub call - actual function pointer table would be used
        return 0;
    }
}

/*
==================
SV_BotInitCvars
==================
*/
pub fn SV_BotInitCvars() {
    unsafe {
        Cvar_Get(b"bot_enable\0".as_ptr() as *const c_char, b"1\0".as_ptr() as *const c_char, 0); //enable the bot
        Cvar_Get(
            b"bot_developer\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //bot developer mode
        Cvar_Get(
            b"bot_debug\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //enable bot debugging
        Cvar_Get(
            b"bot_maxdebugpolys\0".as_ptr() as *const c_char,
            b"2\0".as_ptr() as *const c_char,
            0,
        ); //maximum number of debug polys
        Cvar_Get(
            b"bot_groundonly\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            0,
        ); //only show ground faces of areas
        Cvar_Get(
            b"bot_reachability\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //show all reachabilities to other areas
        Cvar_Get(
            b"bot_visualizejumppads\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //show jumppads
        Cvar_Get(
            b"bot_forceclustering\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //force cluster calculations
        Cvar_Get(
            b"bot_forcereachability\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //force reachability calculations
        Cvar_Get(
            b"bot_forcewrite\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //force writing aas file
        Cvar_Get(
            b"bot_aasoptimize\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //no aas file optimisation
        Cvar_Get(
            b"bot_saveroutingcache\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //save routing cache
        Cvar_Get(
            b"bot_thinktime\0".as_ptr() as *const c_char,
            b"100\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //msec the bots thinks
        Cvar_Get(
            b"bot_reloadcharacters\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //reload the bot characters each time
        Cvar_Get(
            b"bot_testichat\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //test ichats
        Cvar_Get(
            b"bot_testrchat\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //test rchats
        Cvar_Get(
            b"bot_testsolid\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //test for solid areas
        Cvar_Get(
            b"bot_testclusters\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //test the AAS clusters
        Cvar_Get(
            b"bot_fastchat\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //fast chatting bots
        Cvar_Get(
            b"bot_nochat\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //disable chats
        Cvar_Get(
            b"bot_pause\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //pause the bots thinking
        Cvar_Get(
            b"bot_report\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //get a full report in ctf
        Cvar_Get(
            b"bot_grapple\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //enable grapple
        Cvar_Get(
            b"bot_rocketjump\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            0,
        ); //enable rocket jumping
        Cvar_Get(
            b"bot_challenge\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //challenging bot
        Cvar_Get(
            b"bot_minplayers\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            0,
        ); //minimum players in a team or the game
        Cvar_Get(
            b"bot_interbreedchar\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //bot character used for interbreeding
        Cvar_Get(
            b"bot_interbreedbots\0".as_ptr() as *const c_char,
            b"10\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //number of bots used for interbreeding
        Cvar_Get(
            b"bot_interbreedcycle\0".as_ptr() as *const c_char,
            b"20\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //bot interbreeding cycle
        Cvar_Get(
            b"bot_interbreedwrite\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_CHEAT,
        ); //write interbreeded bots to this file
    }
}

// there's no such thing as this now, since the zone is unlimited, but I have to provide something
//	so it doesn't run out of control alloc-wise (since the bot code calls this in a while() loop to free
//	up bot mem until zone has > 1MB available again. So, simulate a reasonable limit...
//
fn bot_Z_AvailableMemory() -> c_int {
    const iMaxBOTLIBMem: usize = 8 * 1024 * 1024;
    unsafe { (iMaxBOTLIBMem - Z_MemSize(TAG_BOTLIB)) as c_int }
}

/*
==================
SV_BotInitBotLib
==================
*/
pub fn SV_BotInitBotLib() {
    let mut botlib_import: botlib_import_t = unsafe { mem::zeroed() };

    unsafe {
        if !debugpolygons.is_null() {
            Z_Free(debugpolygons as *mut c_void);
        }
        bot_maxdebugpolys = Cvar_VariableIntegerValue(b"bot_maxdebugpolys\0".as_ptr() as *const c_char);
        debugpolygons = Z_Malloc(
            (mem::size_of::<bot_debugpoly_t>() * (bot_maxdebugpolys as usize)),
            TAG_BOTLIB,
            qtrue,
        ) as *mut bot_debugpoly_t;

        botlib_import.Print = Some(BotImport_Print);
        botlib_import.Trace = Some(BotImport_Trace);
        botlib_import.EntityTrace = Some(BotImport_EntityTrace);
        botlib_import.PointContents = Some(BotImport_PointContents);
        botlib_import.inPVS = Some(BotImport_inPVS);
        botlib_import.BSPEntityData = Some(BotImport_BSPEntityData);
        botlib_import.BSPModelMinsMaxsOrigin = Some(BotImport_BSPModelMinsMaxsOrigin);
        botlib_import.BotClientCommand = Some(BotClientCommand);

        //memory management
        botlib_import.GetMemory = Some(BotImport_GetMemory);
        botlib_import.FreeMemory = Some(BotImport_FreeMemory);
        botlib_import.AvailableMemory = Some(|| bot_Z_AvailableMemory());
        botlib_import.HunkAlloc = Some(BotImport_HunkAlloc);

        // file system access
        botlib_import.FS_FOpenFile = Some(FS_FOpenFileByMode);
        botlib_import.FS_Read = Some(FS_Read2);
        botlib_import.FS_Write = Some(FS_Write);
        botlib_import.FS_FCloseFile = Some(FS_FCloseFile);
        botlib_import.FS_Seek = Some(FS_Seek);

        //debug lines
        botlib_import.DebugLineCreate = Some(BotImport_DebugLineCreate);
        botlib_import.DebugLineDelete = Some(BotImport_DebugLineDelete);
        botlib_import.DebugLineShow = Some(BotImport_DebugLineShow);

        //debug polygons
        botlib_import.DebugPolygonCreate = Some(BotImport_DebugPolygonCreate);
        botlib_import.DebugPolygonDelete = Some(BotImport_DebugPolygonDelete);

        botlib_export = GetBotLibAPI(BOTLIB_API_VERSION, &mut botlib_import);
        assert!(!botlib_export.is_null()); // bk001129 - somehow we end up with a zero import.
    }
}

//
//  * * * BOT AI CODE IS BELOW THIS POINT * * *
//

/*
==================
SV_BotGetConsoleMessage
==================
*/
pub fn SV_BotGetConsoleMessage(client: c_int, buf: *mut c_char, size: c_int) -> c_int {
    let mut cl: *mut client_t;
    let mut index: c_int;

    unsafe {
        cl = &mut *svs.clients.offset(client as isize);
        (*cl).lastPacketTime = svs.time;

        if (*cl).reliableAcknowledge == (*cl).reliableSequence {
            return qfalse;
        }

        (*cl).reliableAcknowledge += 1;
        index = (*cl).reliableAcknowledge & (MAX_RELIABLE_COMMANDS as c_int - 1);

        if (*cl).reliableCommands[index as usize][0] == 0 {
            return qfalse;
        }

        Q_strncpyz(buf, &(*cl).reliableCommands[index as usize][0], size);
        return qtrue;
    }
}

// #if 0
// /*
// ==================
// EntityInPVS
// ==================
// */
// int EntityInPVS( int client, int entityNum ) {
// 	client_t			*cl;
// 	clientSnapshot_t	*frame;
// 	int					i;
//
// 	cl = &svs.clients[client];
// 	frame = &cl->frames[cl->netchan.outgoingSequence & PACKET_MASK];
// 	for ( i = 0; i < frame->num_entities; i++ )	{
// 		if ( svs.snapshotEntities[(frame->first_entity + i) % svs.numSnapshotEntities].number == entityNum ) {
// 			return qtrue;
// 		}
// 	}
// 	return qfalse;
// }
// #endif

/*
==================
SV_BotGetSnapshotEntity
==================
*/
pub fn SV_BotGetSnapshotEntity(client: c_int, sequence: c_int) -> c_int {
    let mut cl: *mut client_t;
    let mut frame: *mut clientSnapshot_t;

    unsafe {
        cl = &mut *svs.clients.offset(client as isize);
        frame = &mut (*cl).frames[((*cl).netchan.outgoingSequence & PACKET_MASK) as usize];
        if sequence < 0 || sequence >= (*frame).num_entities {
            return -1;
        }
        return (*svs.snapshotEntities.offset(((*frame).first_entity + sequence) as isize % svs.numSnapshotEntities as isize)).number;
    }
}
