// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "tr_local.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_uint, c_void};
use core::ptr::addr_of_mut;

// External types and functions - these need to be imported or declared from tr_local and related modules
// For now, declaring as extern "C" stubs for structural coherence

extern "C" {
    // Global variables from tr_local
    pub static mut tr: tr_t;
    pub static mut backEnd: backEnd_t;
    pub static mut backEndData: *mut backEndData_t;
    pub static mut glState: glState_t;
    pub static mut glConfig: glConfig_t;

    // Cvars
    pub static mut r_speeds: *const cvar_t;
    pub static mut r_texturebits: *const cvar_t;
    pub static mut r_skipBackEnd: *const cvar_t;
    pub static mut r_measureOverdraw: *const cvar_t;
    pub static mut r_shadows: *const cvar_t;
    pub static mut r_textureMode: *const cvar_t;
    pub static mut r_ext_texture_filter_anisotropic: *const cvar_t;
    pub static mut r_gamma: *const cvar_t;
    pub static mut r_ignoreGLErrors: *const cvar_t;
    pub static mut r_drawBuffer: *const cvar_t;

    // Functions
    fn Com_Memset(dest: *mut c_void, c: c_int, count: usize);
    fn Com_Printf(fmt: *const u8, ...);
    fn Com_Error(code: c_int, fmt: *const u8, ...);
    fn R_SumOfUsedImages(countPixels: c_int) -> c_int;
    fn RB_ExecuteRenderCommands(cmds: *const u8);
    fn R_GetShaderByHandle(hShader: c_int) -> *mut shader_t;
    fn Cvar_Set(var_name: *const u8, value: *const u8);
    fn GL_TextureMode(string: *const u8);
    fn R_SetColorMappings();
    fn qglGetError() -> c_int;
    fn qglEnable(cap: c_int);
    fn qglStencilMask(mask: c_uint);
    fn qglClearStencil(s: c_uint);
    fn qglStencilFunc(func: c_int, ref_: c_int, mask: c_uint);
    fn qglStencilOp(fail: c_int, zfail: c_int, zpass: c_int);
    fn qglDisable(cap: c_int);
    fn R_ToggleSmpFrame();
    #[cfg(target_os = "xbox")]
    fn qglBeginFrame() -> c_int;
    #[cfg(target_os = "xbox")]
    fn qglEndFrame();
}

// Constants - render command IDs
const RC_END_OF_LIST: c_int = 0;
const RC_DRAW_SURFS: c_int = 1;
const RC_SET_COLOR: c_int = 2;
const RC_STRETCH_PIC: c_int = 3;
const RC_ROTATE_PIC: c_int = 4;
const RC_ROTATE_PIC2: c_int = 5;
const RC_WORLD_EFFECTS: c_int = 6;
const RC_AUTO_MAP: c_int = 7;
const RC_DRAW_BUFFER: c_int = 8;
const RC_SWAP_BUFFERS: c_int = 9;

// GL Constants
const GL_NO_ERROR: c_int = 0;
const GL_STENCIL_TEST: c_int = 0x0B90;
const GL_BACK_LEFT: c_int = 0x0402;
const GL_BACK_RIGHT: c_int = 0x0403;
const GL_BACK: c_int = 0x0405;
const GL_FRONT: c_int = 0x0404;
const GL_ALWAYS: c_int = 0x0207;
const GL_KEEP: c_int = 0x1E00;
const GL_INCR: c_int = 0x1E02;

// Error codes
const ERR_FATAL: c_int = 1;

// Stereo frame enum
#[repr(C)]
pub enum stereoFrame_t {
    STEREO_CENTER = 0,
    STEREO_LEFT = 1,
    STEREO_RIGHT = 2,
}

// Stub types for structural coherence
#[repr(C)]
pub struct tr_t {
    pub registered: c_int,
    pub pc: performanceCounters_t,
    pub frameCount: c_int,
    pub frameSceneNum: c_int,
    pub refdef: refdef_t,
    pub viewParms: viewParms_t,
    pub viewCluster: c_int,
    pub c_dlightSurfaces: c_int,
    pub c_dlightSurfacesCulled: c_int,
    pub c_leafs: c_int,
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
    pub viewParms_zFar: f32,
    pub frontEndMsec: c_int,
    // Additional fields as needed
}

#[repr(C)]
pub struct backEnd_t {
    pub pc: performanceCounters_t,
    pub c_shaders: c_int,
    pub c_surfaces: c_int,
    pub c_vertexes: c_int,
    pub c_indexes: c_int,
    pub c_totalIndexes: c_int,
    pub c_overDraw: c_int,
    pub c_dlightVertexes: c_int,
    pub c_dlightIndexes: c_int,
    pub c_flareAdds: c_int,
    pub c_flareTests: c_int,
    pub c_flareRenders: c_int,
    // Additional fields as needed
}

#[repr(C)]
pub struct performanceCounters_t {
    pub msec: c_int,
    // Placeholder for other counter fields
}

#[repr(C)]
pub struct backEndData_t {
    pub commands: renderCommandList_t,
    // Additional fields as needed
}

#[repr(C)]
pub struct glState_t {
    pub finishCalled: c_int,
    // Placeholder for actual glState structure
}

#[repr(C)]
pub struct glConfig_t {
    pub colorBits: c_int,
    pub depthBits: c_int,
    pub stencilBits: c_int,
    pub vidWidth: c_int,
    pub vidHeight: c_int,
    pub stereoEnabled: c_int,
    // Placeholder for other glConfig fields
}

#[repr(C)]
pub struct cvar_t {
    pub integer: c_int,
    pub string: *const u8,
    pub modified: c_int,
    // Placeholder for actual cvar structure
}

#[repr(C)]
pub struct shader_t {
    // Placeholder for actual shader structure
}

#[repr(C)]
pub struct renderCommandList_t {
    pub cmds: *mut u8,
    pub used: c_int,
    // Additional fields as needed
}

#[repr(C)]
pub struct drawSurf_t {
    // Placeholder for actual drawSurf structure
}

#[repr(C)]
pub struct drawSurfsCommand_t {
    pub commandId: c_int,
    pub drawSurfs: *mut drawSurf_t,
    pub numDrawSurfs: c_int,
    pub refdef: refdef_t,
    pub viewParms: viewParms_t,
}

#[repr(C)]
pub struct setColorCommand_t {
    pub commandId: c_int,
    pub color: [f32; 4],
}

#[repr(C)]
pub struct stretchPicCommand_t {
    pub commandId: c_int,
    pub shader: *mut shader_t,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub s1: f32,
    pub t1: f32,
    pub s2: f32,
    pub t2: f32,
}

#[repr(C)]
pub struct rotatePicCommand_t {
    pub commandId: c_int,
    pub shader: *mut shader_t,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub s1: f32,
    pub t1: f32,
    pub s2: f32,
    pub t2: f32,
    pub a: f32,
}

#[repr(C)]
pub struct drawBufferCommand_t {
    pub commandId: c_int,
    pub buffer: c_int,
}

#[repr(C)]
pub struct swapBuffersCommand_t {
    pub commandId: c_int,
}

#[repr(C)]
pub struct refdef_t {
    // Placeholder for actual refdef structure
}

#[repr(C)]
pub struct viewParms_t {
    // Placeholder for actual viewParms structure
}

const MAX_RENDER_COMMANDS: usize = 0x40000;

/*
=====================
R_PerformanceCounters
=====================
*/
#[allow(unused_unsafe)]
pub unsafe fn R_PerformanceCounters() {
    #[cfg(not(target_os = "xbox"))]
    {
        if (*r_speeds).integer == 0 {
            // clear the counters even if we aren't printing
            Com_Memset(
                addr_of_mut!(tr) as *mut c_void,
                0,
                core::mem::size_of_val(&tr),
            );
            Com_Memset(
                addr_of_mut!(backEnd) as *mut c_void,
                0,
                core::mem::size_of_val(&backEnd),
            );
            return;
        }

        if (*r_speeds).integer == 1 {
            let texSize = R_SumOfUsedImages(0) as f32 / (8.0 * 1048576.0_f32)
                * (if (*r_texturebits).integer != 0 {
                    (*r_texturebits).integer
                } else {
                    (*glConfig).colorBits
                }) as f32;
            Com_Printf(
                b"%i/%i shdrs/srfs %i leafs %i vrts %i/%i tris %.2fMB tex %.2f dc\n\0".as_ptr(),
                (*backEnd).c_shaders,
                (*backEnd).c_surfaces,
                (*tr).c_leafs,
                (*backEnd).c_vertexes,
                (*backEnd).c_indexes / 3,
                (*backEnd).c_totalIndexes / 3,
                texSize,
                (*backEnd).c_overDraw as f32
                    / ((*glConfig).vidWidth as f32 * (*glConfig).vidHeight as f32),
            );
        } else if (*r_speeds).integer == 2 {
            Com_Printf(
                b"(patch) %i sin %i sclip  %i sout %i bin %i bclip %i bout\n\0".as_ptr(),
                (*tr).c_sphere_cull_patch_in,
                (*tr).c_sphere_cull_patch_clip,
                (*tr).c_sphere_cull_patch_out,
                (*tr).c_box_cull_patch_in,
                (*tr).c_box_cull_patch_clip,
                (*tr).c_box_cull_patch_out,
            );
            Com_Printf(
                b"(md3) %i sin %i sclip  %i sout %i bin %i bclip %i bout\n\0".as_ptr(),
                (*tr).c_sphere_cull_md3_in,
                (*tr).c_sphere_cull_md3_clip,
                (*tr).c_sphere_cull_md3_out,
                (*tr).c_box_cull_md3_in,
                (*tr).c_box_cull_md3_clip,
                (*tr).c_box_cull_md3_out,
            );
        } else if (*r_speeds).integer == 3 {
            Com_Printf(b"viewcluster: %i\n\0".as_ptr(), (*tr).viewCluster);
        } else if (*r_speeds).integer == 4 {
            if (*backEnd).c_dlightVertexes != 0 {
                Com_Printf(
                    b"dlight srf:%i  culled:%i  verts:%i  tris:%i\n\0".as_ptr(),
                    (*tr).c_dlightSurfaces,
                    (*tr).c_dlightSurfacesCulled,
                    (*backEnd).c_dlightVertexes,
                    (*backEnd).c_dlightIndexes / 3,
                );
            }
        } else if (*r_speeds).integer == 5 {
            Com_Printf(b"zFar: %.0f\n\0".as_ptr(), (*tr).viewParms_zFar);
        } else if (*r_speeds).integer == 6 {
            Com_Printf(
                b"flare adds:%i tests:%i renders:%i\n\0".as_ptr(),
                (*backEnd).c_flareAdds,
                (*backEnd).c_flareTests,
                (*backEnd).c_flareRenders,
            );
        } else if (*r_speeds).integer == 7 {
            let texSize = R_SumOfUsedImages(1) as f32 / (1048576.0_f32);
            let backBuff = (*glConfig).vidWidth as f32
                * (*glConfig).vidHeight as f32
                * (*glConfig).colorBits as f32
                / (8.0_f32 * 1024.0_f32 * 1024.0_f32);
            let depthBuff = (*glConfig).vidWidth as f32
                * (*glConfig).vidHeight as f32
                * (*glConfig).depthBits as f32
                / (8.0_f32 * 1024.0_f32 * 1024.0_f32);
            let stencilBuff = (*glConfig).vidWidth as f32
                * (*glConfig).vidHeight as f32
                * (*glConfig).stencilBits as f32
                / (8.0_f32 * 1024.0_f32 * 1024.0_f32);
            Com_Printf(
                b"Tex MB %.2f + buffers %.2f MB = Total %.2fMB\n\0".as_ptr(),
                texSize,
                backBuff * 2.0 + depthBuff + stencilBuff,
                texSize + backBuff * 2.0 + depthBuff + stencilBuff,
            );
        }
    }

    Com_Memset(
        addr_of_mut!(tr) as *mut c_void,
        0,
        core::mem::size_of_val(&tr),
    );
    Com_Memset(
        addr_of_mut!(backEnd) as *mut c_void,
        0,
        core::mem::size_of_val(&backEnd),
    );
}

/*
====================
R_InitCommandBuffers
====================
*/
pub fn R_InitCommandBuffers() {}

/*
====================
R_ShutdownCommandBuffers
====================
*/
pub fn R_ShutdownCommandBuffers() {}

/*
====================
R_IssueRenderCommands
====================
*/
pub unsafe fn R_IssueRenderCommands(runPerformanceCounters: c_int) {
    let cmdList = &mut (*backEndData).commands;

    assert!(!cmdList.cmds.is_null()); // bk001205
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
    if (*r_skipBackEnd).integer == 0 {
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
    #[cfg(not(target_os = "xbox"))]
    {
        if (*addr_of_mut!(tr)).registered == 0 {
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
    if (cmdList.used + bytes + 4) as usize > MAX_RENDER_COMMANDS {
        #[cfg(all(debug_assertions, target_os = "xbox"))]
        {
            Com_Printf(b"\x1b[31mCommand buffer overflow!  Tell Brian.\n\0".as_ptr());
        }
        if bytes as usize > MAX_RENDER_COMMANDS - 4 {
            Com_Error(
                ERR_FATAL,
                b"R_GetCommandBuffer: bad size %i\0".as_ptr(),
                bytes,
            );
        }
        // if we run out of room, just start dropping commands
        return core::ptr::null_mut();
    }

    let result = cmdList.cmds.add(cmdList.used as usize) as *mut c_void;
    cmdList.used += bytes;

    result
}

/*
=============
R_AddDrawSurfCmd

=============
*/
pub unsafe fn R_AddDrawSurfCmd(drawSurfs: *mut drawSurf_t, numDrawSurfs: c_int) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<drawSurfsCommand_t>() as c_int)
        as *mut drawSurfsCommand_t;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_DRAW_SURFS;

    (*cmd).drawSurfs = drawSurfs;
    (*cmd).numDrawSurfs = numDrawSurfs;

    (*cmd).refdef = (*addr_of_mut!(tr)).refdef;
    (*cmd).viewParms = (*addr_of_mut!(tr)).viewParms;
}

/*
=============
RE_SetColor

Passing NULL will set the color to white
=============
*/
pub unsafe fn RE_SetColor(rgba: *const f32) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<setColorCommand_t>() as c_int)
        as *mut setColorCommand_t;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_SET_COLOR;
    let rgba = if rgba.is_null() {
        static colorWhite: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        colorWhite.as_ptr()
    } else {
        rgba
    };

    (*cmd).color[0] = *rgba;
    (*cmd).color[1] = *rgba.add(1);
    (*cmd).color[2] = *rgba.add(2);
    (*cmd).color[3] = *rgba.add(3);
}

/*
=============
RE_StretchPic
=============
*/
pub unsafe fn RE_StretchPic(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    hShader: c_int,
) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<stretchPicCommand_t>() as c_int)
        as *mut stretchPicCommand_t;
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
pub unsafe fn RE_RotatePic(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    a: f32,
    hShader: c_int,
) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<rotatePicCommand_t>() as c_int)
        as *mut rotatePicCommand_t;
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
pub unsafe fn RE_RotatePic2(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    s1: f32,
    t1: f32,
    s2: f32,
    t2: f32,
    a: f32,
    hShader: c_int,
) {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<rotatePicCommand_t>() as c_int)
        as *mut rotatePicCommand_t;
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

pub unsafe fn RE_RenderWorldEffects() {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<drawBufferCommand_t>() as c_int)
        as *mut drawBufferCommand_t;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_WORLD_EFFECTS;
}

pub unsafe fn RE_RenderAutoMap() {
    let cmd = R_GetCommandBuffer(core::mem::size_of::<drawBufferCommand_t>() as c_int)
        as *mut drawBufferCommand_t;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_AUTO_MAP;
}

/*
====================
RE_BeginFrame

If running in stereo, RE_BeginFrame will be called twice
for each RE_EndFrame
====================
*/
pub unsafe fn RE_BeginFrame(stereoFrame: stereoFrame_t) {
    if (*addr_of_mut!(tr)).registered == 0 {
        return;
    }
    (*addr_of_mut!(glState)).finishCalled = 0;

    (*addr_of_mut!(tr)).frameCount += 1;
    (*addr_of_mut!(tr)).frameSceneNum = 0;

    //
    // do overdraw measurement
    //
    #[cfg(not(target_os = "xbox"))]
    {
        if (*r_measureOverdraw).integer != 0 {
            if (*glConfig).stencilBits < 4 {
                Com_Printf(
                    b"Warning: not enough stencil bits to measure overdraw: %d\n\0".as_ptr(),
                    (*glConfig).stencilBits,
                );
                Cvar_Set(b"r_measureOverdraw\0".as_ptr(), b"0\0".as_ptr());
                (*r_measureOverdraw).modified = 0;
            } else if (*r_shadows).integer == 2 {
                Com_Printf(
                    b"Warning: stencil shadows and overdraw measurement are mutually exclusive\n\0"
                        .as_ptr(),
                );
                Cvar_Set(b"r_measureOverdraw\0".as_ptr(), b"0\0".as_ptr());
                (*r_measureOverdraw).modified = 0;
            } else {
                R_SyncRenderThread();
                qglEnable(GL_STENCIL_TEST);
                qglStencilMask(!0u32 as c_uint);
                qglClearStencil(0u32 as c_uint);
                qglStencilFunc(GL_ALWAYS, 0, !0u32 as c_uint);
                qglStencilOp(GL_KEEP, GL_INCR, GL_INCR);
            }
            (*r_measureOverdraw).modified = 0;
        } else {
            // this is only reached if it was on and is now off
            if (*r_measureOverdraw).modified != 0 {
                R_SyncRenderThread();
                qglDisable(GL_STENCIL_TEST);
            }
            (*r_measureOverdraw).modified = 0;
        }
    }

    //
    // texturemode stuff
    //
    if (*r_textureMode).modified != 0 || (*r_ext_texture_filter_anisotropic).modified != 0 {
        R_SyncRenderThread();
        GL_TextureMode((*r_textureMode).string as *const u8);
        (*r_textureMode).modified = 0;
        (*r_ext_texture_filter_anisotropic).modified = 0;
    }

    //
    // gamma stuff
    //
    if (*r_gamma).modified != 0 {
        (*r_gamma).modified = 0;

        R_SyncRenderThread();
        R_SetColorMappings();
    }

    // check for errors
    if (*r_ignoreGLErrors).integer == 0 {
        R_SyncRenderThread();
        let err = qglGetError();
        if err != GL_NO_ERROR {
            Com_Error(
                ERR_FATAL,
                b"RE_BeginFrame() - glGetError() failed (0x%x)!\n\0".as_ptr(),
                err,
            );
        }
    }

    //
    // draw buffer stuff
    //
    let cmd = R_GetCommandBuffer(core::mem::size_of::<drawBufferCommand_t>() as c_int)
        as *mut drawBufferCommand_t;
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_DRAW_BUFFER;

    if (*glConfig).stereoEnabled != 0 {
        match stereoFrame {
            stereoFrame_t::STEREO_LEFT => {
                (*cmd).buffer = GL_BACK_LEFT;
            }
            stereoFrame_t::STEREO_RIGHT => {
                (*cmd).buffer = GL_BACK_RIGHT;
            }
            _ => {
                Com_Error(
                    ERR_FATAL,
                    b"RE_BeginFrame: Stereo is enabled, but stereoFrame was %i\0".as_ptr(),
                    stereoFrame as c_int,
                );
            }
        }
    } else {
        if (stereoFrame as c_int) != (stereoFrame_t::STEREO_CENTER as c_int) {
            Com_Error(
                ERR_FATAL,
                b"RE_BeginFrame: Stereo is disabled, but stereoFrame was %i\0".as_ptr(),
                stereoFrame as c_int,
            );
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
    let cmd = R_GetCommandBuffer(core::mem::size_of::<swapBuffersCommand_t>() as c_int)
        as *mut swapBuffersCommand_t;

    if (*addr_of_mut!(tr)).registered == 0 {
        return;
    }
    if cmd.is_null() {
        return;
    }
    (*cmd).commandId = RC_SWAP_BUFFERS;

    #[cfg(target_os = "xbox")]
    {
        if qglBeginFrame() == 0 {
            return;
        }
    }

    R_IssueRenderCommands(1);

    #[cfg(target_os = "xbox")]
    {
        qglEndFrame();
    }

    // use the other buffers next frame, because another CPU
    // may still be rendering into the current ones
    R_ToggleSmpFrame();

    if !frontEndMsec.is_null() {
        *frontEndMsec = (*addr_of_mut!(tr)).frontEndMsec;
    }
    (*addr_of_mut!(tr)).frontEndMsec = 0;
    if !backEndMsec.is_null() {
        *backEndMsec = (*addr_of_mut!(backEnd)).pc.msec;
    }
    (*addr_of_mut!(backEnd)).pc.msec = 0;
}
