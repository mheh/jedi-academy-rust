//! Mechanical port of `codemp/client/FxSystem.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_void};
use core::ptr;

use crate::codemp::client::snd_public_h::{sfxHandle_t, S_RegisterSound, S_StartLocalSound, S_StartSound};
use crate::codemp::game::q_math::VectorCopy;
use crate::codemp::game::q_shared_h::{
    byte, qboolean, qhandle_t, trace_t, vec2_t, vec3_t, CHAN_AUTO, ENTITYNUM_NONE, FS_READ,
};
use crate::codemp::qcommon::files_h::cvar_t;
use crate::codemp::qcommon::vm_local_h::vm_t;
use crate::ffi::types::fileHandle_t;

unsafe extern "C" {
    pub static mut fx_debug: *mut cvar_t;

    #[cfg(feature = "sof2dev")]
    pub static mut fx_freeze: *mut cvar_t;

    pub static mut fx_countScale: *mut cvar_t;
    pub static mut fx_nearCull: *mut cvar_t;

    pub static mut theFxHelper: SFxHelper;

    pub static mut cl: clientActive_t;
    pub static mut cgvm: *mut vm_t;
    pub static mut re: refexport_t;

    pub fn FS_FOpenFileByMode(path: *const c_char, fh: *mut fileHandle_t, mode: c_int) -> c_int;
    pub fn FS_Read2(buffer: *mut c_void, len: c_int, f: fileHandle_t);
    pub fn FS_FCloseFile(f: fileHandle_t);

    pub fn VM_Call(vm: *mut vm_t, callNum: c_int, ...) -> c_int;
}

#[repr(C)]
pub struct clientActive_t {
    _prefix: [u8; 0],
    pub mSharedMemory: *mut c_char,
}

#[repr(C)]
pub struct CGhoul2Info_v {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct refdef_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct polyVert_t {
    pub xyz: vec3_t,
    pub st: [c_float; 2],
    pub modulate: [byte; 4],
}

#[repr(C)]
pub struct refEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
}

#[repr(C)]
pub struct miniRefEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
}

pub const CG_TRACE: c_int = 16;
pub const CG_G2TRACE: c_int = 17;
pub const CG_G2MARK: c_int = 18;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TCGTrace {
    pub mResult: trace_t,
    pub mStart: vec3_t,
    pub mMins: vec3_t,
    pub mMaxs: vec3_t,
    pub mEnd: vec3_t,
    pub mSkipNumber: c_int,
    pub mMask: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TCGG2Mark {
    pub shader: c_int,
    pub size: c_float,
    pub start: vec3_t,
    pub dir: vec3_t,
}

#[repr(C)]
pub struct refexport_t {
    pub Shutdown: Option<unsafe extern "C" fn(destroyWindow: qboolean)>,
    pub BeginRegistration: Option<unsafe extern "C" fn(config: *mut c_void)>,
    pub RegisterModel: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub RegisterSkin: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub RegisterShader: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub RegisterShaderNoMip: Option<unsafe extern "C" fn(name: *const c_char) -> qhandle_t>,
    pub ShaderNameFromIndex: Option<unsafe extern "C" fn(index: c_int) -> *const c_char>,
    pub LoadWorld: Option<unsafe extern "C" fn(name: *const c_char)>,
    pub SetWorldVisData: Option<unsafe extern "C" fn(vis: *const byte)>,
    pub EndRegistration: Option<unsafe extern "C" fn()>,
    pub ClearScene: Option<unsafe extern "C" fn()>,
    pub ClearDecals: Option<unsafe extern "C" fn()>,
    pub AddRefEntityToScene: Option<unsafe extern "C" fn(re: *const refEntity_t)>,
    pub AddMiniRefEntityToScene: Option<unsafe extern "C" fn(re: *const miniRefEntity_t)>,
    pub AddPolyToScene:
        Option<unsafe extern "C" fn(hShader: qhandle_t, numVerts: c_int, verts: *const polyVert_t, num: c_int)>,
    pub AddDecalToScene: Option<
        unsafe extern "C" fn(
            shader: qhandle_t,
            origin: *const vec3_t,
            dir: *const vec3_t,
            orientation: c_float,
            r: c_float,
            g: c_float,
            b: c_float,
            a: c_float,
            alphaFade: qboolean,
            radius: c_float,
            temporary: qboolean,
        ),
    >,
    pub LightForPoint: Option<
        unsafe extern "C" fn(
            point: *mut vec3_t,
            ambientLight: *mut vec3_t,
            directedLight: *mut vec3_t,
            lightDir: *mut vec3_t,
        ) -> c_int,
    >,
    #[cfg(not(feature = "vv_lighting"))]
    pub AddLightToScene:
        Option<unsafe extern "C" fn(org: *const vec3_t, intensity: c_float, r: c_float, g: c_float, b: c_float)>,
}

#[inline]
pub fn Vector2Clear(a: &mut vec2_t) {
    a[0] = 0.0;
    a[1] = 0.0;
}

#[inline]
pub fn Vector2Set(a: &mut vec2_t, b: c_float, c: c_float) {
    a[0] = b;
    a[1] = c;
}

#[inline]
pub fn Vector2Copy(src: &vec2_t, dst: &mut vec2_t) {
    dst[0] = src[0];
    dst[1] = src[1];
}

#[inline]
pub fn Vector2MA(src: &vec2_t, m: c_float, v: &vec2_t, dst: &mut vec2_t) {
    dst[0] = src[0] + (m * v[0]);
    dst[1] = src[1] + (m * v[1]);
}

#[inline]
pub fn Vector2Scale(src: &vec2_t, b: c_float, dst: &mut vec2_t) {
    dst[0] = src[0] * b;
    dst[1] = src[1] * b;
}

#[repr(C)]
pub struct SFxHelper {
    pub mTime: c_int,
    pub mOldTime: c_int,
    pub mFrameTime: c_int,
    pub mTimeFrozen: bool,
    pub mRealTime: c_float,
    pub refdef: *mut refdef_t,
    #[cfg(debug_assertions)]
    pub mMainRefs: c_int,
    #[cfg(debug_assertions)]
    pub mMiniRefs: c_int,
}

impl SFxHelper {
    pub unsafe fn new() -> Self {
        todo!("SFxHelper::SFxHelper body is in FxSystem.cpp")
    }

    #[inline]
    pub unsafe fn GetTime(&self) -> c_int {
        self.mTime
    }

    #[inline]
    pub unsafe fn GetFrameTime(&self) -> c_int {
        self.mFrameTime
    }

    pub unsafe fn ReInit(&mut self, _pRefdef: *mut refdef_t) {
        todo!("SFxHelper::ReInit body is in FxSystem.cpp")
    }

    pub unsafe fn AdjustTime(&mut self, _time: c_int) {
        todo!("SFxHelper::AdjustTime body is in FxSystem.cpp")
    }

    pub unsafe fn Print(&mut self, _msg: *const c_char) {
        todo!("SFxHelper::Print body is in FxSystem.cpp; variadic wrapper not represented in Rust method form")
    }

    #[inline]
    pub unsafe fn OpenFile(
        &mut self,
        path: *const c_char,
        fh: *mut fileHandle_t,
        _mode: c_int,
    ) -> c_int {
        FS_FOpenFileByMode(path, fh, FS_READ)
    }

    #[inline]
    pub unsafe fn ReadFile(&mut self, data: *mut c_void, len: c_int, fh: fileHandle_t) -> c_int {
        FS_Read2(data, len, fh);
        1
    }

    #[inline]
    pub unsafe fn CloseFile(&mut self, fh: fileHandle_t) {
        FS_FCloseFile(fh);
    }

    #[inline]
    pub unsafe fn PlaySound(
        &mut self,
        origin: *const vec3_t,
        _entityNum: c_int,
        _entchannel: c_int,
        sfxHandle: sfxHandle_t,
        _volume: c_int,
        _radius: c_int,
    ) {
        //S_StartSound( origin, ENTITYNUM_NONE, CHAN_AUTO, sfxHandle, volume, radius );
        S_StartSound(origin, ENTITYNUM_NONE, CHAN_AUTO, sfxHandle);
    }

    #[inline]
    pub unsafe fn PlayLocalSound(&mut self, sfxHandle: sfxHandle_t, entchannel: c_int) {
        //S_StartSound( origin, ENTITYNUM_NONE, CHAN_AUTO, sfxHandle, volume, radius );
        S_StartLocalSound(sfxHandle, entchannel);
    }

    #[inline]
    pub unsafe fn RegisterSound(&mut self, sound: *const c_char) -> c_int {
        S_RegisterSound(sound)
    }

    #[inline]
    pub unsafe fn Trace(
        &mut self,
        tr: *mut trace_t,
        start: *const vec3_t,
        min: *const vec3_t,
        max: *const vec3_t,
        end: *const vec3_t,
        skipEntNum: c_int,
        flags: c_int,
    ) {
        let td: *mut TCGTrace = cl.mSharedMemory as *mut TCGTrace;
        let mut min = min;
        let mut max = max;
        let vec3_origin: vec3_t = [0.0, 0.0, 0.0];

        if min.is_null() {
            min = &vec3_origin;
        }

        if max.is_null() {
            max = &vec3_origin;
        }

        // Original C calls `memset(td, sizeof(*td), 0)`, which writes zero bytes.
        ptr::write_bytes(td as *mut u8, core::mem::size_of::<TCGTrace>() as u8, 0);
        VectorCopy(&*start, &mut (*td).mStart);
        VectorCopy(&*min, &mut (*td).mMins);
        VectorCopy(&*max, &mut (*td).mMaxs);
        VectorCopy(&*end, &mut (*td).mEnd);
        (*td).mSkipNumber = skipEntNum;
        (*td).mMask = flags;

        VM_Call(cgvm, CG_TRACE);

        *tr = (*td).mResult;
    }

    #[inline]
    pub unsafe fn G2Trace(
        &mut self,
        tr: *mut trace_t,
        start: *const vec3_t,
        min: *const vec3_t,
        max: *const vec3_t,
        end: *const vec3_t,
        skipEntNum: c_int,
        flags: c_int,
    ) {
        let td: *mut TCGTrace = cl.mSharedMemory as *mut TCGTrace;
        let mut min = min;
        let mut max = max;
        let vec3_origin: vec3_t = [0.0, 0.0, 0.0];

        if min.is_null() {
            min = &vec3_origin;
        }

        if max.is_null() {
            max = &vec3_origin;
        }

        // Original C calls `memset(td, sizeof(*td), 0)`, which writes zero bytes.
        ptr::write_bytes(td as *mut u8, core::mem::size_of::<TCGTrace>() as u8, 0);
        VectorCopy(&*start, &mut (*td).mStart);
        VectorCopy(&*min, &mut (*td).mMins);
        VectorCopy(&*max, &mut (*td).mMaxs);
        VectorCopy(&*end, &mut (*td).mEnd);
        (*td).mSkipNumber = skipEntNum;
        (*td).mMask = flags;

        VM_Call(cgvm, CG_G2TRACE);

        *tr = (*td).mResult;
    }

    #[inline]
    pub unsafe fn AddGhoul2Decal(
        &mut self,
        shader: c_int,
        start: *const vec3_t,
        dir: *const vec3_t,
        size: c_float,
    ) {
        let td: *mut TCGG2Mark = cl.mSharedMemory as *mut TCGG2Mark;

        (*td).size = size;
        (*td).shader = shader;
        VectorCopy(&*start, &mut (*td).start);
        VectorCopy(&*dir, &mut (*td).dir);

        VM_Call(cgvm, CG_G2MARK);
    }

    #[inline]
    pub unsafe fn AddFxToScene(&mut self, ent: *mut refEntity_t) {
        #[cfg(debug_assertions)]
        {
            self.mMainRefs += 1;
            assert!(ent.is_null() || (*ent).renderfx >= 0);
        }
        if let Some(AddRefEntityToScene) = re.AddRefEntityToScene {
            AddRefEntityToScene(ent);
        }
    }

    #[inline]
    pub unsafe fn AddMiniFxToScene(&mut self, ent: *mut miniRefEntity_t) {
        #[cfg(debug_assertions)]
        {
            self.mMiniRefs += 1;
            assert!(ent.is_null() || (*ent).renderfx >= 0);
        }
        if let Some(AddMiniRefEntityToScene) = re.AddMiniRefEntityToScene {
            AddMiniRefEntityToScene(ent);
        }
    }

    #[cfg(not(feature = "vv_lighting"))]
    #[inline]
    pub unsafe fn AddLightToScene(
        &mut self,
        org: *const vec3_t,
        radius: c_float,
        red: c_float,
        green: c_float,
        blue: c_float,
    ) {
        if let Some(AddLightToScene) = re.AddLightToScene {
            AddLightToScene(org, radius, red, green, blue);
        }
    }

    #[inline]
    pub unsafe fn RegisterShader(&mut self, shader: *const c_char) -> c_int {
        if let Some(RegisterShader) = re.RegisterShader {
            RegisterShader(shader)
        } else {
            0
        }
    }

    #[inline]
    pub unsafe fn RegisterModel(&mut self, model: *const c_char) -> c_int {
        if let Some(RegisterModel) = re.RegisterModel {
            RegisterModel(model)
        } else {
            0
        }
    }

    #[inline]
    pub unsafe fn AddPolyToScene(&mut self, shader: c_int, count: c_int, verts: *mut polyVert_t) {
        if let Some(AddPolyToScene) = re.AddPolyToScene {
            AddPolyToScene(shader, count, verts, 1);
        }
    }

    #[inline]
    pub unsafe fn AddDecalToScene(
        &mut self,
        shader: qhandle_t,
        origin: *const vec3_t,
        dir: *const vec3_t,
        orientation: c_float,
        r: c_float,
        g: c_float,
        b: c_float,
        a: c_float,
        alphaFade: qboolean,
        radius: c_float,
        temporary: qboolean,
    ) {
        if let Some(AddDecalToScene) = re.AddDecalToScene {
            AddDecalToScene(
                shader,
                origin,
                dir,
                orientation,
                r,
                g,
                b,
                a,
                alphaFade,
                radius,
                temporary,
            );
        }
    }

    pub unsafe fn CameraShake(
        &mut self,
        _origin: *const vec3_t,
        _intensity: c_float,
        _radius: c_int,
        _time: c_int,
    ) {
        todo!("SFxHelper::CameraShake body is in FxSystem.cpp")
    }

    pub unsafe fn GetOriginAxisFromBolt(
        &mut self,
        _pGhoul2: *mut CGhoul2Info_v,
        _mEntNum: c_int,
        _modelNum: c_int,
        _boltNum: c_int,
        _origin: *mut vec3_t,
        _axis: *mut vec3_t,
    ) -> qboolean {
        todo!("SFxHelper::GetOriginAxisFromBolt body is in FxSystem.cpp")
    }
}
