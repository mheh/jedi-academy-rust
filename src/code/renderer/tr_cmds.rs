// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_char, c_uint, c_float, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Constants from header files
const RC_END_OF_LIST: c_int = 0;
const RC_DRAW_SURFS: c_int = 1;
const RC_SET_COLOR: c_int = 2;
const RC_STRETCH_PIC: c_int = 3;
const RC_ROTATE_PIC: c_int = 4;
const RC_ROTATE_PIC2: c_int = 5;
const RC_SCISSOR: c_int = 6;
const RC_WORLD_EFFECTS: c_int = 7;
const RC_DRAW_BUFFER: c_int = 8;
const RC_SWAP_BUFFERS: c_int = 9;
const MAX_RENDER_COMMANDS: usize = 16384;
const PRINT_ALL: c_int = 0;
const ERR_FATAL: c_int = 1;
const STEREO_CENTER: c_int = 0;
const STEREO_LEFT: c_int = 1;
const STEREO_RIGHT: c_int = 2;
const RDF_doLAGoggles: c_int = 0x00000040;
const RDF_doFullbright: c_int = 0x00000080;
const MAX_LIGHT_STYLES: usize = 32;
const GL_STENCIL_TEST: c_int = 0x0B90;
const GL_ALWAYS: c_int = 0x0207;
const GL_KEEP: c_int = 0x1E00;
const GL_INCR: c_int = 0x1E02;
const GL_BACK: c_int = 0x0405;
const GL_BACK_LEFT: c_int = 0x0402;
const GL_BACK_RIGHT: c_int = 0x0403;
const GL_NO_ERROR: c_int = 0;

// Stub performance counter structure
#[repr(C)]
pub struct PerformanceCounters {
    pub c_shaders: c_int,
    pub c_surfaces: c_int,
    pub c_leafs: c_int,
    pub c_vertexes: c_int,
    pub c_indexes: c_int,
    pub c_totalIndexes: c_int,
    pub c_overDraw: c_int,
    pub c_sphere_cull_patch_in: c_int,
    pub c_sphere_cull_patch_clip: c_int,
    pub c_sphere_cull_patch_out: c_int,
    pub c_box_cull_patch_in: c_int,
    pub c_box_cull_patch_clip: c_int,
    pub c_box_cull_patch_out: c_int,
    pub c_sphere_cull_md3_in: c_int,
    pub c_sphere_cull_md3_clip: c_int,
    pub c_sphere_cull_md3_out: c_int,
    pub c_box_cull_md3_in: c_int,
    pub c_box_cull_md3_clip: c_int,
    pub c_box_cull_md3_out: c_int,
    pub c_dlightSurfaces: c_int,
    pub c_dlightSurfacesCulled: c_int,
    pub c_dlightVertexes: c_int,
    pub c_dlightIndexes: c_int,
    pub c_flareAdds: c_int,
    pub c_flareTests: c_int,
    pub c_flareRenders: c_int,
    pub msec: c_int,
}

// Stub structures for command types
#[repr(C)]
pub struct DrawSurfsCommand {
    pub commandId: c_int,
    pub drawSurfs: *mut c_void,
    pub numDrawSurfs: c_int,
    pub refdef: RefDef,
    pub viewParms: ViewParms,
}

#[repr(C)]
pub struct SetColorCommand {
    pub commandId: c_int,
    pub color: [c_float; 4],
}

#[repr(C)]
pub struct StretchPicCommand {
    pub commandId: c_int,
    pub shader: *mut c_void,
    pub x: c_float,
    pub y: c_float,
    pub w: c_float,
    pub h: c_float,
    pub s1: c_float,
    pub t1: c_float,
    pub s2: c_float,
    pub t2: c_float,
}

#[repr(C)]
pub struct RotatePicCommand {
    pub commandId: c_int,
    pub shader: *mut c_void,
    pub x: c_float,
    pub y: c_float,
    pub w: c_float,
    pub h: c_float,
    pub s1: c_float,
    pub t1: c_float,
    pub s2: c_float,
    pub t2: c_float,
    pub a: c_float,
}

#[repr(C)]
pub struct ScissorCommand {
    pub commandId: c_int,
    pub x: c_float,
    pub y: c_float,
    pub w: c_float,
    pub h: c_float,
}

#[repr(C)]
pub struct SetModeCommand {
    pub commandId: c_int,
}

#[repr(C)]
pub struct DrawBufferCommand {
    pub commandId: c_int,
    pub buffer: c_int,
}

#[repr(C)]
pub struct SwapBuffersCommand {
    pub commandId: c_int,
}

#[repr(C)]
pub struct RenderCommandList {
    pub cmds: *mut c_void,
    pub used: usize,
}

#[repr(C)]
pub struct CVarData {
    pub integer: c_int,
    pub modified: c_int,
    pub string: *const c_char,
}

#[repr(C)]
pub struct GlConfigData {
    pub colorBits: c_int,
    pub stencilBits: c_int,
    pub depthBits: c_int,
    pub vidWidth: c_int,
    pub vidHeight: c_int,
    pub stereoEnabled: c_int,
}

// External C functions
extern "C" {
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn R_SumOfUsedImages(all: c_int) -> c_float;
    fn VID_Printf(level: c_int, fmt: *const c_char, ...);
    fn RB_ExecuteRenderCommands(cmds: *mut c_void);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn R_GetShaderByHandle(handle: c_int) -> *mut c_void;
    fn ColorBytes4(r: c_float, g: c_float, b: c_float, a: c_float) -> c_int;
    fn random() -> c_float;
    fn Cvar_Set(name: *const c_char, value: *const c_char);
    fn GL_TextureMode(mode: *const c_char);
    fn R_SetColorMappings();
    fn qglGetError() -> c_int;
    fn qglEnable(cap: c_int);
    fn qglDisable(cap: c_int);
    fn qglStencilMask(mask: c_uint);
    fn qglClearStencil(s: c_uint);
    fn qglStencilFunc(func: c_int, ref_: c_int, mask: c_uint);
    fn qglStencilOp(fail: c_int, zfail: c_int, zpass: c_int);
    fn qglBeginFrame() -> c_int;
    fn qglEndFrame();
    fn RE_ProcessDissolve();
    fn R_ToggleSmpFrame();

    // External globals - opaque to this module
    pub static mut tr: TrGlobals;
    pub static mut backEnd: BackEndStruct;
    pub static mut backEndData: *mut BackEndData;
    pub static mut glState: GlStateData;
    pub static mut r_speeds: *mut CVarData;
    pub static mut r_texturebits: *mut CVarData;
    pub static mut glConfig: GlConfigData;
    pub static mut r_skipBackEnd: *mut CVarData;
    pub static mut r_measureOverdraw: *mut CVarData;
    pub static mut r_shadows: *mut CVarData;
    pub static mut r_ignoreGLErrors: *mut CVarData;
    pub static mut r_textureMode: *mut CVarData;
    pub static mut r_ext_texture_filter_anisotropic: *mut CVarData;
    pub static mut r_gamma: *mut CVarData;
    pub static mut r_drawBuffer: *mut CVarData;

    pub static mut styleUpdated: [c_int; MAX_LIGHT_STYLES];
}

// Refdef structure - accessed for rdflags and floatTime
#[repr(C)]
pub struct RefDef {
    pub rdflags: c_int,
    pub floatTime: c_float,
    _pad: [c_char; 256],
}

// ViewParms structure
#[repr(C)]
pub struct ViewParms {
    pub zFar: c_float,
    _pad: [c_char; 256],
}

// Fog parameters and structure
#[repr(C)]
pub struct FogParms {
    pub color: [c_float; 4],
    pub depthForOpaque: c_float,
}

#[repr(C)]
pub struct Fog {
    pub parms: FogParms,
    pub colorInt: c_int,
    pub tcScale: c_float,
}

// World structure - accessed for fogs array and numfogs
#[repr(C)]
pub struct World {
    pub fogs: *mut Fog,
    pub numfogs: c_int,
    _pad: [c_char; 256],
}

// Opaque stub types for external structures
#[repr(C)]
pub struct TrGlobals {
    pub pc: PerformanceCounters,
    pub registered: c_int,
    pub frameCount: c_int,
    pub frameSceneNum: c_int,
    pub frontEndMsec: c_int,
    pub refdef: RefDef,
    pub viewParms: ViewParms,
    pub world: *mut World,
    pub viewCluster: c_int,
    _pad: [c_char; 256],
}

#[repr(C)]
pub struct BackEndStruct {
    pub pc: PerformanceCounters,
    _pad: [c_char; 256],
}

#[repr(C)]
pub struct BackEndData {
    pub commands: RenderCommandList,
    _pad: [c_char; 256],
}

#[repr(C)]
pub struct GlStateData {
    pub finishCalled: c_int,
    _pad: [c_char; 256],
}

// Global variables
pub static mut c_blockedOnRender: c_int = 0;
pub static mut c_blockedOnMain: c_int = 0;

/*
=====================
R_PerformanceCounters
=====================
*/
pub unsafe fn R_PerformanceCounters() {
    #[cfg(not(xbox))]
    {
        if (*addr_of!(r_speeds)).integer == 0 {
            // clear the counters even if we aren't printing
            memset(addr_of_mut!(tr.pc) as *mut c_void, 0, core::mem::size_of::<PerformanceCounters>());
            memset(addr_of_mut!(backEnd.pc) as *mut c_void, 0, core::mem::size_of::<PerformanceCounters>());
            return;
        }

        if (*addr_of!(r_speeds)).integer == 1 {
            let texSize = R_SumOfUsedImages(0) / (8.0 * 1048576.0) * (if (*addr_of!(r_texturebits)).integer != 0 { (*addr_of!(r_texturebits)).integer as c_float } else { (*addr_of!(glConfig)).colorBits as c_float });
            VID_Printf(PRINT_ALL, b"%i/%i shdrs/srfs %i leafs %i vrts %i/%i tris %.2fMB tex %.2f dc\n\0".as_ptr() as *const c_char,
                (*addr_of!(backEnd.pc)).c_shaders, (*addr_of!(backEnd.pc)).c_surfaces, (*addr_of!(tr.pc)).c_leafs, (*addr_of!(backEnd.pc)).c_vertexes,
                (*addr_of!(backEnd.pc)).c_indexes / 3, (*addr_of!(backEnd.pc)).c_totalIndexes / 3,
                texSize, (*addr_of!(backEnd.pc)).c_overDraw as c_float / ((*addr_of!(glConfig)).vidWidth * (*addr_of!(glConfig)).vidHeight) as c_float);
        } else if (*addr_of!(r_speeds)).integer == 2 {
            VID_Printf(PRINT_ALL, b"(patch) %i sin %i sclip  %i sout %i bin %i bclip %i bout\n\0".as_ptr() as *const c_char,
                (*addr_of!(tr.pc)).c_sphere_cull_patch_in, (*addr_of!(tr.pc)).c_sphere_cull_patch_clip, (*addr_of!(tr.pc)).c_sphere_cull_patch_out,
                (*addr_of!(tr.pc)).c_box_cull_patch_in, (*addr_of!(tr.pc)).c_box_cull_patch_clip, (*addr_of!(tr.pc)).c_box_cull_patch_out);
            VID_Printf(PRINT_ALL, b"(md3) %i sin %i sclip  %i sout %i bin %i bclip %i bout\n\0".as_ptr() as *const c_char,
                (*addr_of!(tr.pc)).c_sphere_cull_md3_in, (*addr_of!(tr.pc)).c_sphere_cull_md3_clip, (*addr_of!(tr.pc)).c_sphere_cull_md3_out,
                (*addr_of!(tr.pc)).c_box_cull_md3_in, (*addr_of!(tr.pc)).c_box_cull_md3_clip, (*addr_of!(tr.pc)).c_box_cull_md3_out);
        } else if (*addr_of!(r_speeds)).integer == 3 {
            VID_Printf(PRINT_ALL, b"viewcluster: %i\n\0".as_ptr() as *const c_char, (*addr_of!(tr.viewCluster)));
        } else if (*addr_of!(r_speeds)).integer == 4 {
            if (*addr_of!(backEnd.pc)).c_dlightVertexes != 0 {
                VID_Printf(PRINT_ALL, b"dlight srf:%i  culled:%i  verts:%i  tris:%i\n\0".as_ptr() as *const c_char,
                    (*addr_of!(tr.pc)).c_dlightSurfaces, (*addr_of!(tr.pc)).c_dlightSurfacesCulled,
                    (*addr_of!(backEnd.pc)).c_dlightVertexes, (*addr_of!(backEnd.pc)).c_dlightIndexes / 3);
            }
        } else if (*addr_of!(r_speeds)).integer == 5 {
            VID_Printf(PRINT_ALL, b"zFar: %.0f\n\0".as_ptr() as *const c_char, (*addr_of!(tr.viewParms)).zFar);
        } else if (*addr_of!(r_speeds)).integer == 6 {
            VID_Printf(PRINT_ALL, b"flare adds:%i tests:%i renders:%i\n\0".as_ptr() as *const c_char,
                (*addr_of!(backEnd.pc)).c_flareAdds, (*addr_of!(backEnd.pc)).c_flareTests, (*addr_of!(backEnd.pc)).c_flareRenders);
        } else if (*addr_of!(r_speeds)).integer == 7 {
            let texSize = R_SumOfUsedImages(1) / (1048576.0);
            let backBuff = (*addr_of!(glConfig)).vidWidth as c_float * (*addr_of!(glConfig)).vidHeight as c_float * (*addr_of!(glConfig)).colorBits as c_float / (8.0 * 1024.0 * 1024.0);
            let depthBuff = (*addr_of!(glConfig)).vidWidth as c_float * (*addr_of!(glConfig)).vidHeight as c_float * (*addr_of!(glConfig)).depthBits as c_float / (8.0 * 1024.0 * 1024.0);
            let stencilBuff = (*addr_of!(glConfig)).vidWidth as c_float * (*addr_of!(glConfig)).vidHeight as c_float * (*addr_of!(glConfig)).stencilBits as c_float / (8.0 * 1024.0 * 1024.0);
            VID_Printf(PRINT_ALL, b"Tex MB %.2f + buffers %.2f MB = Total %.2fMB\n\0".as_ptr() as *const c_char,
                texSize, backBuff * 2.0 + depthBuff + stencilBuff, texSize + backBuff * 2.0 + depthBuff + stencilBuff);
        }
    }

    memset(addr_of_mut!(tr.pc) as *mut c_void, 0, core::mem::size_of::<PerformanceCounters>());
    memset(addr_of_mut!(backEnd.pc) as *mut c_void, 0, core::mem::size_of::<PerformanceCounters>());
}

/*
====================
R_InitCommandBuffers
====================
*/
pub fn R_InitCommandBuffers() {
}

/*
====================
R_ShutdownCommandBuffers
====================
*/
pub fn R_ShutdownCommandBuffers() {
}

/*
====================
R_IssueRenderCommands
====================
*/
pub unsafe fn R_IssueRenderCommands(runPerformanceCounters: c_int) {
    let cmdList = &mut (*backEndData).commands;

    // add an end-of-list command
    *(cmdList.cmds.add(cmdList.used) as *mut c_int) = RC_END_OF_LIST;

    // clear it out, in case this is a sync and not a buffer flip
    cmdList.used = 0;

    // at this point, the back end thread is idle, so it is ok
    // to look at it's performance counters
    if runPerformanceCounters != 0 {
        R_PerformanceCounters();
    }

    // actually start the commands going
    if (*addr_of!(r_skipBackEnd)).integer == 0 {
        // let it start on the new batch
        RB_ExecuteRenderCommands(cmdList.cmds);
    }
}

/*
====================
R_SyncRenderThread

Issue any pending commands and wait for them to complete.
After exiting, the render thread will have completed its work
and will remain idle and the main thread is free to issue
OpenGL calls until R_IssueRenderCommands is called.
====================
*/
pub unsafe fn R_SyncRenderThread() {
    #[cfg(not(xbox))]
    {
        if (*addr_of!(tr.registered)) == 0 {
            return;
        }
        R_IssueRenderCommands(0);
    }
}

/*
============
R_GetCommandBuffer

make sure there is enough command space, waiting on the
render thread if needed.
============
*/
pub unsafe fn R_GetCommandBuffer(bytes: c_int) -> *mut c_void {
    let cmdList = &mut (*backEndData).commands;

    // always leave room for the end of list command
    if cmdList.used + (bytes as usize) + 4 > MAX_RENDER_COMMANDS {
        #[cfg(all(debug_assertions, xbox))]
        {
            Com_Printf(b"Command buffer overflow!  Tell Brian.\n\0".as_ptr() as *const c_char);
        }
        if bytes > (MAX_RENDER_COMMANDS - 4) as c_int {
            Com_Error(ERR_FATAL, b"R_GetCommandBuffer: bad size %i\0".as_ptr() as *const c_char, bytes);
        }
        // if we run out of room, just start dropping commands
        return core::ptr::null_mut();
    }

    cmdList.used += bytes as usize;

    cmdList.cmds.add(cmdList.used - (bytes as usize))
}

/*
=============
R_AddDrawSurfCmd

=============
*/
pub unsafe fn R_AddDrawSurfCmd(drawSurfs: *mut c_void, numDrawSurfs: c_int) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<DrawSurfsCommand>() as c_int) as *mut DrawSurfsCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_DRAW_SURFS;

    (*cmd).drawSurfs = drawSurfs;
    (*cmd).numDrawSurfs = numDrawSurfs;

    (*cmd).refdef = tr.refdef;
    (*cmd).viewParms = tr.viewParms;
}

/*
=============
RE_SetColor

Passing NULL will set the color to white
=============
*/
pub unsafe fn RE_SetColor(rgba: *const c_float) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<SetColorCommand>() as c_int) as *mut SetColorCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_SET_COLOR;
    if !rgba.is_null() {
        (*cmd).color[0] = *rgba.add(0);
        (*cmd).color[1] = *rgba.add(1);
        (*cmd).color[2] = *rgba.add(2);
        (*cmd).color[3] = *rgba.add(3);
        return;
    }

    (*cmd).color[0] = 1.0;
    (*cmd).color[1] = 1.0;
    (*cmd).color[2] = 1.0;
    (*cmd).color[3] = 1.0;
}

/*
=============
RE_StretchPic
=============
*/
pub unsafe fn RE_StretchPic(x: c_float, y: c_float, w: c_float, h: c_float,
                            s1: c_float, t1: c_float, s2: c_float, t2: c_float, hShader: c_int) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<StretchPicCommand>() as c_int) as *mut StretchPicCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_STRETCH_PIC;
    (*cmd).shader = R_GetShaderByHandle(hShader);
    (*cmd).x = x;
    (*cmd).y = y;
    (*cmd).w = w;
    (*cmd).h = h;
    (*cmd).s1 = s1;
    (*cmd).t1 = t1;
    (*cmd).s2 = s2;
    (*cmd).t2 = t2;
}

/*
=============
RE_RotatePic
=============
*/
pub unsafe fn RE_RotatePic(x: c_float, y: c_float, w: c_float, h: c_float,
                           s1: c_float, t1: c_float, s2: c_float, t2: c_float, a: c_float, hShader: c_int) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<RotatePicCommand>() as c_int) as *mut RotatePicCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_ROTATE_PIC;
    (*cmd).shader = R_GetShaderByHandle(hShader);
    (*cmd).x = x;
    (*cmd).y = y;
    (*cmd).w = w;
    (*cmd).h = h;
    (*cmd).s1 = s1;
    (*cmd).t1 = t1;
    (*cmd).s2 = s2;
    (*cmd).t2 = t2;
    (*cmd).a = a;
}

/*
=============
RE_RotatePic2
=============
*/
pub unsafe fn RE_RotatePic2(x: c_float, y: c_float, w: c_float, h: c_float,
                            s1: c_float, t1: c_float, s2: c_float, t2: c_float, a: c_float, hShader: c_int) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<RotatePicCommand>() as c_int) as *mut RotatePicCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_ROTATE_PIC2;
    (*cmd).shader = R_GetShaderByHandle(hShader);
    (*cmd).x = x;
    (*cmd).y = y;
    (*cmd).w = w;
    (*cmd).h = h;
    (*cmd).s1 = s1;
    (*cmd).t1 = t1;
    (*cmd).s2 = s2;
    (*cmd).t2 = t2;
    (*cmd).a = a;
}

pub unsafe fn RE_LAGoggles() {
    (*addr_of_mut!(tr.refdef)).rdflags |= (RDF_doLAGoggles | RDF_doFullbright);

    let fog = &mut (*(*addr_of!(tr.world)).fogs).add((*addr_of!(tr.world)).numfogs as usize);

    (*fog).parms.color[0] = 0.75;
    (*fog).parms.color[1] = 0.42 + random() * 0.025;
    (*fog).parms.color[2] = 0.07;
    (*fog).parms.color[3] = 1.0;
    (*fog).parms.depthForOpaque = 10000.0;
    (*fog).colorInt = ColorBytes4((*fog).parms.color[0], (*fog).parms.color[1], (*fog).parms.color[2], (*fog).parms.color[3]);
    (*fog).tcScale = 2.0 / ((*fog).parms.depthForOpaque * (1.0 + ((*addr_of!(tr.refdef)).floatTime).cos() * 0.1));
}

pub unsafe fn RE_RenderWorldEffects() {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<SetModeCommand>() as c_int) as *mut SetModeCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_WORLD_EFFECTS;
}

/*
=============
RE_Scissor
=============
*/
pub unsafe fn RE_Scissor(x: c_float, y: c_float, w: c_float, h: c_float) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<ScissorCommand>() as c_int) as *mut ScissorCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_SCISSOR;
    (*cmd).x = x;
    (*cmd).y = y;
    (*cmd).w = w;
    (*cmd).h = h;
}

/*
====================
RE_BeginFrame

If running in stereo, RE_BeginFrame will be called twice
for each RE_EndFrame
====================
*/
pub unsafe fn RE_BeginFrame(stereoFrame: c_int) {
    if (*addr_of!(tr.registered)) == 0 {
        return;
    }
    (*addr_of_mut!(glState)).finishCalled = 0;

    (*addr_of_mut!(tr.frameCount)) += 1;
    (*addr_of_mut!(tr.frameSceneNum)) = 0;

    //
    // do overdraw measurement
    //
    #[cfg(not(xbox))]
    {
        if (*addr_of!(r_measureOverdraw)).integer != 0 {
            if (*addr_of!(glConfig)).stencilBits < 4 {
                VID_Printf(PRINT_ALL, b"Warning: not enough stencil bits to measure overdraw: %d\n\0".as_ptr() as *const c_char, (*addr_of!(glConfig)).stencilBits);
                Cvar_Set(b"r_measureOverdraw\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
                (*addr_of!(r_measureOverdraw)).modified = 0;
            } else if (*addr_of!(r_shadows)).integer == 2 {
                VID_Printf(PRINT_ALL, b"Warning: stencil shadows and overdraw measurement are mutually exclusive\n\0".as_ptr() as *const c_char);
                Cvar_Set(b"r_measureOverdraw\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
                (*addr_of!(r_measureOverdraw)).modified = 0;
            } else {
                R_SyncRenderThread();
                qglEnable(GL_STENCIL_TEST);
                qglStencilMask(!0u32 as c_uint);
                qglClearStencil(0u32 as c_uint);
                qglStencilFunc(GL_ALWAYS, 0, !0u32 as c_uint);
                qglStencilOp(GL_KEEP, GL_INCR, GL_INCR);
            }
            (*addr_of!(r_measureOverdraw)).modified = 0;
        } else {
            // this is only reached if it was on and is now off
            if (*addr_of!(r_measureOverdraw)).modified != 0 {
                R_SyncRenderThread();
                qglDisable(GL_STENCIL_TEST);
                (*addr_of!(r_measureOverdraw)).modified = 0;
            }
        }
    }

    //
    // texturemode stuff
    //
    if (*addr_of!(r_textureMode)).modified != 0 || (*addr_of!(r_ext_texture_filter_anisotropic)).modified != 0 {
        R_SyncRenderThread();
        GL_TextureMode((*addr_of!(r_textureMode)).string);
        (*addr_of!(r_textureMode)).modified = 0;
        (*addr_of!(r_ext_texture_filter_anisotropic)).modified = 0;
    }

    //
    // gamma stuff
    //
    if (*addr_of!(r_gamma)).modified != 0 {
        (*addr_of!(r_gamma)).modified = 0;

        R_SyncRenderThread();
        R_SetColorMappings();
    }

    // check for errors
    if (*addr_of!(r_ignoreGLErrors)).integer == 0 {
        R_SyncRenderThread();
        let err = qglGetError();
        if err != GL_NO_ERROR {
            Com_Error(ERR_FATAL, b"RE_BeginFrame() - glGetError() failed (0x%x)!\n\0".as_ptr() as *const c_char, err);
        }
    }

    //
    // draw buffer stuff
    //
    let cmd = R_GetCommandBuffer(core::mem::size_of::<DrawBufferCommand>() as c_int) as *mut DrawBufferCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_DRAW_BUFFER;

    if (*addr_of!(glConfig)).stereoEnabled != 0 {
        if stereoFrame == STEREO_LEFT {
            (*cmd).buffer = GL_BACK_LEFT;
        } else if stereoFrame == STEREO_RIGHT {
            (*cmd).buffer = GL_BACK_RIGHT;
        } else {
            Com_Error(ERR_FATAL, b"RE_BeginFrame: Stereo is enabled, but stereoFrame was %i\0".as_ptr() as *const c_char, stereoFrame);
        }
    } else {
        if stereoFrame != STEREO_CENTER {
            Com_Error(ERR_FATAL, b"RE_BeginFrame: Stereo is disabled, but stereoFrame was %i\0".as_ptr() as *const c_char, stereoFrame);
        }
        //		if ( !Q_stricmp( r_drawBuffer->string, "GL_FRONT" ) ) {
        //			cmd->buffer = (int)GL_FRONT;
        //		} else
        {
            (*cmd).buffer = GL_BACK;
        }
    }
}

/*
=============
RE_EndFrame

Returns the number of msec spent in the back end
=============
*/
pub unsafe fn RE_EndFrame(frontEndMsec: *mut c_int, backEndMsec: *mut c_int) {
    if (*addr_of!(tr.registered)) == 0 {
        return;
    }
    let cmd = R_GetCommandBuffer(core::mem::size_of::<SwapBuffersCommand>() as c_int) as *mut SwapBuffersCommand;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_SWAP_BUFFERS;

    #[cfg(xbox)]
    {
        if qglBeginFrame() == 0 {
            return;
        }
    }

    R_IssueRenderCommands(1);

    #[cfg(xbox)]
    {
        RE_ProcessDissolve(); // render the disolve now
        qglEndFrame();
    }

    // use the other buffers next frame, because another CPU
    // may still be rendering into the current ones
    R_ToggleSmpFrame();

    if !frontEndMsec.is_null() {
        *frontEndMsec = (*addr_of!(tr.frontEndMsec));
    }
    (*addr_of_mut!(tr.frontEndMsec)) = 0;
    if !backEndMsec.is_null() {
        *backEndMsec = (*addr_of!(backEnd.pc)).msec;
    }
    (*addr_of_mut!(backEnd.pc)).msec = 0;

    for i in 0..MAX_LIGHT_STYLES {
        styleUpdated[i] = 0;
    }
}
