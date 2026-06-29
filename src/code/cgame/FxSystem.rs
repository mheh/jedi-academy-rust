// this include must remain at the top of every FXxxxx.CPP file
// #include "common_headers.h"

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

use crate::code::cgame::FxSystem_h::*;

// ============================================================================
// External C functions and types
// ============================================================================

extern "C" {
    // #if !defined(FX_SCHEDULER_H_INC)
    // #include "FxScheduler.h"
    // #endif
    // (FxScheduler types assumed to be defined elsewhere)

    // #include "cg_media.h"	//for cgs.model_draw for G2
    pub static cgs: cgs_t;

    // extern vmCvar_t	fx_debug;
    // extern vmCvar_t	fx_freeze;
    // (already declared in FxSystem_h)

    // extern void CG_ExplosionEffects( vec3_t origin, float intensity, int radius, int time );
    pub fn CG_ExplosionEffects(origin: *const vec3_t, intensity: f32, radius: c_int, time: c_int);
}

// File I/O functions
extern "C" {
    pub fn cgi_FS_FOpenFile(qpath: *const c_char, f: *mut fileHandle_t, mode: c_int) -> c_int;
    pub fn cgi_FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t) -> c_int;
    pub fn cgi_FS_FCloseFile(f: fileHandle_t);
}

// Sound functions
extern "C" {
    pub fn cgi_S_StartSound(origin: *const vec3_t, entityNum: c_int, entchannel: c_int, sfx: sfxHandle_t);
    pub fn cgi_S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int);
    pub fn cgi_S_RegisterSound(name: *const c_char) -> sfxHandle_t;
}

// Render functions
extern "C" {
    pub fn cgi_R_AddRefEntityToScene(ent: *const refEntity_t);
    pub fn cgi_R_RegisterShader(name: *const c_char) -> c_int;
    pub fn cgi_R_RegisterModel(name: *const c_char) -> c_int;
    pub fn cgi_R_AddLightToScene(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32);
    pub fn cgi_R_AddPolyToScene(shader: c_int, numVerts: c_int, verts: *const polyVert_t);
}

// Trace and G2 functions
extern "C" {
    pub fn CG_Trace(result: *mut trace_t, start: *const vec3_t, mins: *const vec3_t, maxs: *const vec3_t,
                    end: *const vec3_t, skipNumber: c_int, mask: c_int);

    // gi interface
    pub fn gi_trace(result: *mut trace_t, start: *const c_void, mins: *const c_void, maxs: *const c_void,
                    end: *const vec3_t, skipEntNum: c_int, mask: c_int, flag: c_int);
    pub fn gi_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelNum: c_int, boltNum: c_int,
                                  boltMatrix: *mut mdxaBone_t, angles: *const vec3_t,
                                  origin: *const vec3_t, time: c_int, model_draw: *mut c_void,
                                  modelScale: f32) -> c_int;
}

// Force Feedback functions
extern "C" {
    pub fn cgi_FF_Register(name: *const c_char, channel: c_int) -> ffHandle_t;
    pub fn cgi_FF_Start(ff: ffHandle_t, entityNum: c_int);
}

// Printf function
extern "C" {
    pub fn gi_Printf(msg: *const c_char);
}

// ============================================================================
// Type stubs
// ============================================================================

#[repr(C)]
pub struct cgs_t {
    pub _pad: [u8; 344], // Padding to model_draw
    pub model_draw: *mut c_void,
    // ... rest of fields omitted
}

#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct centity_t {
    pub _pad0: [u8; 136],
    pub snapShotTime: c_int,
    pub _pad1: [u8; 4],
    pub currentState: entityState_t,
    pub _pad2: [u8; 28],
    pub lerpAngles: vec3_t,
    pub _pad3: [u8; 16],
    pub lerpOrigin: vec3_t,
    pub _pad4: [u8; 44],
    pub renderAngles: vec3_t,
    pub _pad5: [u8; 8],
    pub gent: *mut gentity_t,
}

#[repr(C)]
pub struct entityState_t {
    pub eType: c_int,
    pub _pad0: [u8; 40],
    pub modelScale: f32,
}

#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub _pad0: [u8; 256],
    pub m_iVehicleNum: c_int,
    pub _pad1: [u8; 4],
    pub m_pVehicle: *mut Vehicle,
    pub _pad2: [u8; 400],
    pub ghoul2: *mut c_void,
}

#[repr(C)]
pub struct vehicleInfo_t {
    pub _pad0: [u8; 0],
    pub r#type: c_int,
    // ... rest of fields omitted
}

#[repr(C)]
pub struct Vehicle {
    pub _pad0: [u8; 0],
    pub m_pVehicleInfo: *mut vehicleInfo_t,
    // ... rest of fields omitted
}

// ============================================================================
// Stuff for the FxHelper
// ============================================================================

// //------------------------------------------------------
// void SFxHelper::Init()
// {
// 	mTime = 0;
// }
pub unsafe extern "C" fn SFxHelper_Init(this: *mut SFxHelper) {
    (*this).mTime = 0;
}

// //------------------------------------------------------
// void SFxHelper::Print( const char *msg, ... )
// {
// #ifndef FINAL_BUILD
//
// 	va_list		argptr;
// 	char		text[1024];
//
// 	va_start( argptr, msg );
// 	vsprintf( text, msg, argptr );
// 	va_end( argptr );
//
// 	gi.Printf( text );
//
// #endif
// }
pub unsafe extern "C" fn SFxHelper_Print(_this: *mut SFxHelper, msg: *const c_char, _args: ...) {
    // In FINAL_BUILD this is compiled out. For the port, we skip implementation
    // as vsprintf and va_list are C varargs that are handled differently in Rust
}

// //------------------------------------------------------
// void SFxHelper::AdjustTime( int frameTime )
// {
// 	if ( fx_freeze.integer || ( frameTime <= 0 ))
// 	{
// 		// Allow no time progression when we are paused.
// 		mFrameTime = 0;
// 		mFloatFrameTime = 0.0f;
// 	}
// 	else
// 	{
// 		if ( !cg_paused.integer )
// 		{
// 			if ( frameTime > 300 ) // hack for returning from paused and time bursts
// 			{
// 				frameTime = 300;
// 			}
//
// 			mFrameTime = frameTime;
// 			mFloatFrameTime = mFrameTime * 0.001f;
// 			mTime += mFrameTime;
// 		}
// 	}
// }
pub unsafe extern "C" fn SFxHelper_AdjustTime(this: *mut SFxHelper, mut frameTime: c_int) {
    extern "C" {
        pub static mut fx_freeze: vmCvar_t;
        pub static mut cg_paused: vmCvar_t;
    }

    if fx_freeze as c_int != 0 || frameTime <= 0 {
        // Allow no time progression when we are paused.
        (*this).mFrameTime = 0;
        (*this).mFloatFrameTime = 0.0f32;
    } else {
        if (cg_paused as c_int) == 0 {
            if frameTime > 300 {
                // hack for returning from paused and time bursts
                frameTime = 300;
            }

            (*this).mFrameTime = frameTime;
            (*this).mFloatFrameTime = (*this).mFrameTime as f32 * 0.001f32;
            (*this).mTime += (*this).mFrameTime;
        }
    }
}

// //------------------------------------------------------
// int SFxHelper::OpenFile( const char *file, fileHandle_t *fh, int mode )
// {
// //	char path[256];
//
// //	sprintf( path, "%s/%s", FX_FILE_PATH, file );
// 	return cgi_FS_FOpenFile( file, fh, FS_READ );
// }
pub unsafe extern "C" fn SFxHelper_OpenFile(_this: *mut SFxHelper, file: *const c_char, fh: *mut fileHandle_t, _mode: c_int) -> c_int {
    // const FS_READ = 0; (assumed)
    cgi_FS_FOpenFile(file, fh, 0)
}

// //------------------------------------------------------
// int SFxHelper::ReadFile( void *data, int len, fileHandle_t fh )
// {
// 	return cgi_FS_Read( data, len, fh );
// }
pub unsafe extern "C" fn SFxHelper_ReadFile(_this: *mut SFxHelper, data: *mut c_void, len: c_int, fh: fileHandle_t) -> c_int {
    cgi_FS_Read(data, len, fh)
}

// //------------------------------------------------------
// void SFxHelper::CloseFile( fileHandle_t fh )
// {
// 	cgi_FS_FCloseFile( fh );
// }
pub unsafe extern "C" fn SFxHelper_CloseFile(_this: *mut SFxHelper, fh: fileHandle_t) {
    cgi_FS_FCloseFile(fh);
}

// //------------------------------------------------------
// void SFxHelper::PlaySound( const vec3_t org, int entityNum, int entchannel, int sfxHandle )
// {
// 	cgi_S_StartSound( org, entityNum, entchannel, sfxHandle );
// }
pub unsafe extern "C" fn SFxHelper_PlaySound(_this: *mut SFxHelper, org: *const vec3_t, entityNum: c_int, entchannel: c_int, sfxHandle: sfxHandle_t) {
    cgi_S_StartSound(org, entityNum, entchannel, sfxHandle);
}

// //------------------------------------------------------
// void SFxHelper::PlayLocalSound( int sfxHandle, int channelNum )
// {
// 	cgi_S_StartLocalSound(sfxHandle, channelNum);
// }
pub unsafe extern "C" fn SFxHelper_PlayLocalSound(_this: *mut SFxHelper, sfxHandle: sfxHandle_t, channelNum: c_int) {
    cgi_S_StartLocalSound(sfxHandle, channelNum);
}

// //------------------------------------------------------
// void SFxHelper::Trace( trace_t *tr, vec3_t start, vec3_t min, vec3_t max,
// 						vec3_t end, int skipEntNum, int flags )
// {
// 	CG_Trace( tr, start, min, max, end, skipEntNum, flags );
// }
pub unsafe extern "C" fn SFxHelper_Trace(_this: *mut SFxHelper, tr: *mut trace_t, start: *const vec3_t, min: *const vec3_t, max: *const vec3_t,
                                        end: *const vec3_t, skipEntNum: c_int, flags: c_int) {
    CG_Trace(tr, start, min, max, end, skipEntNum, flags);
}

// void SFxHelper::G2Trace( trace_t *tr, vec3_t start, vec3_t min, vec3_t max,
// 						vec3_t end, int skipEntNum, int flags )
// {
// 	//CG_Trace( tr, start, min, max, end, skipEntNum, flags, G2_COLLIDE );
// 	gi.trace(tr, start, NULL, NULL, end, skipEntNum, flags, G2_COLLIDE);
// }
pub unsafe extern "C" fn SFxHelper_G2Trace(_this: *mut SFxHelper, tr: *mut trace_t, start: *const vec3_t, _min: *const vec3_t, _max: *const vec3_t,
                                          end: *const vec3_t, skipEntNum: c_int, flags: c_int) {
    // const G2_COLLIDE = 1; (assumed)
    gi_trace(tr, core::ptr::null(), core::ptr::null(), core::ptr::null(), end, skipEntNum, flags, 1);
}

// //------------------------------------------------------
// void SFxHelper::AddFxToScene( refEntity_t *ent )
// {
// 	cgi_R_AddRefEntityToScene( ent );
// }
pub unsafe extern "C" fn SFxHelper_AddFxToScene(_this: *mut SFxHelper, ent: *const refEntity_t) {
    cgi_R_AddRefEntityToScene(ent);
}

// //------------------------------------------------------
// int SFxHelper::RegisterShader( const char *shader )
// {
// 	return cgi_R_RegisterShader( shader );
// }
pub unsafe extern "C" fn SFxHelper_RegisterShader(_this: *mut SFxHelper, shader: *const c_char) -> c_int {
    cgi_R_RegisterShader(shader)
}

// //------------------------------------------------------
// int SFxHelper::RegisterSound( const char *sound )
// {
// 	return cgi_S_RegisterSound( sound );
// }
pub unsafe extern "C" fn SFxHelper_RegisterSound(_this: *mut SFxHelper, sound: *const c_char) -> c_int {
    cgi_S_RegisterSound(sound)
}

// //------------------------------------------------------
// int SFxHelper::RegisterModel( const char *model )
// {
// 	return cgi_R_RegisterModel( model );
// }
pub unsafe extern "C" fn SFxHelper_RegisterModel(_this: *mut SFxHelper, model: *const c_char) -> c_int {
    cgi_R_RegisterModel(model)
}

// //------------------------------------------------------
// void SFxHelper::AddLightToScene( vec3_t org, float radius, float red, float green, float blue )
// {
// 	cgi_R_AddLightToScene( org, radius, red, green, blue );
// }
pub unsafe extern "C" fn SFxHelper_AddLightToScene(_this: *mut SFxHelper, org: *const vec3_t, radius: f32, red: f32, green: f32, blue: f32) {
    cgi_R_AddLightToScene(org, radius, red, green, blue);
}

// //------------------------------------------------------
// void SFxHelper::AddPolyToScene( int shader, int count, polyVert_t *verts )
// {
// 	cgi_R_AddPolyToScene( shader, count, verts );
// }
pub unsafe extern "C" fn SFxHelper_AddPolyToScene(_this: *mut SFxHelper, shader: c_int, count: c_int, verts: *const polyVert_t) {
    cgi_R_AddPolyToScene(shader, count, verts);
}

// //------------------------------------------------------
// void SFxHelper::CameraShake( vec3_t origin, float intensity, int radius, int time )
// {
// 	CG_ExplosionEffects( origin, intensity, radius, time );
// }
pub unsafe extern "C" fn SFxHelper_CameraShake(_this: *mut SFxHelper, origin: *const vec3_t, intensity: f32, radius: c_int, time: c_int) {
    CG_ExplosionEffects(origin, intensity, radius, time);
}

// //------------------------------------------------------
// int SFxHelper::GetOriginAxisFromBolt(const centity_t &cent, int modelNum, int boltNum, vec3_t /*out*/origin, vec3_t /*out*/axis[3])
// {
// 	if ((cg.time-cent.snapShotTime) > 200)
// 	{ //you were added more than 200ms ago, so I say you are no longer valid/in our snapshot.
// 		return 0;
// 	}
//
// 	int doesBoltExist;
// 	mdxaBone_t 	boltMatrix;
// 	vec3_t	G2Angles = {cent.lerpAngles[0] , cent.lerpAngles[1], cent.lerpAngles[2]};
// 	if ( cent.currentState.eType == ET_PLAYER )
// 	{//players use cent.renderAngles
// 		VectorCopy( cent.renderAngles, G2Angles );
//
// 		if ( cent.gent //has a game entity
// 			&& cent.gent->s.m_iVehicleNum != 0 //in a vehicle
// 			&& cent.gent->m_pVehicle //have a valid vehicle pointer
// 			&& cent.gent->m_pVehicle->m_pVehicleInfo->type != VH_FIGHTER //it's not a fighter
// 			&& cent.gent->m_pVehicle->m_pVehicleInfo->type != VH_SPEEDER //not a speeder
// 			)
// 		{
// 			G2Angles[PITCH]=0;
// 			G2Angles[ROLL] =0;
// 		}
// 	}
//
// 	// go away and get me the bolt position for this frame please
// 	doesBoltExist = gi.G2API_GetBoltMatrix(cent.gent->ghoul2, modelNum,
// 		boltNum, &boltMatrix, G2Angles,
// 		cent.lerpOrigin, cg.time, cgs.model_draw,
// 		cent.currentState.modelScale);
// 	// set up the axis and origin we need for the actual effect spawning
// 	origin[0] = boltMatrix.matrix[0][3];
// 	origin[1] = boltMatrix.matrix[1][3];
// 	origin[2] = boltMatrix.matrix[2][3];
//
// 	axis[1][0] = boltMatrix.matrix[0][0];
// 	axis[1][1] = boltMatrix.matrix[1][0];
// 	axis[1][2] = boltMatrix.matrix[2][0];
//
// 	axis[0][0] = boltMatrix.matrix[0][1];
// 	axis[0][1] = boltMatrix.matrix[1][1];
// 	axis[0][2] = boltMatrix.matrix[2][1];
//
// 	axis[2][0] = boltMatrix.matrix[0][2];
// 	axis[2][1] = boltMatrix.matrix[1][2];
// 	axis[2][2] = boltMatrix.matrix[2][2];
// 	return doesBoltExist;
// }
pub unsafe extern "C" fn SFxHelper_GetOriginAxisFromBolt(_this: *mut SFxHelper, cent: *const centity_t, modelNum: c_int, boltNum: c_int,
                                                        origin: *mut vec3_t, axis: *mut *mut vec3_t) -> c_int {
    extern "C" {
        pub static cg: cg_t;
    }

    #[repr(C)]
    pub struct cg_t {
        pub _pad: [u8; 316],
        pub time: c_int,
    }

    const ET_PLAYER: c_int = 1;
    const PITCH: usize = 0;
    const ROLL: usize = 2;
    const VH_FIGHTER: c_int = 0;
    const VH_SPEEDER: c_int = 1;

    if (cg.time - (*cent).snapShotTime) > 200 {
        // you were added more than 200ms ago, so I say you are no longer valid/in our snapshot.
        return 0;
    }

    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut G2Angles: vec3_t = [(*cent).lerpAngles[0], (*cent).lerpAngles[1], (*cent).lerpAngles[2]];

    if (*cent).currentState.eType == ET_PLAYER {
        // players use cent.renderAngles
        G2Angles[0] = (*cent).renderAngles[0];
        G2Angles[1] = (*cent).renderAngles[1];
        G2Angles[2] = (*cent).renderAngles[2];

        if !(*cent).gent.is_null() {
            let gent = &*(*cent).gent;
            if gent.m_iVehicleNum != 0 && !gent.m_pVehicle.is_null() {
                let vehicle = &*gent.m_pVehicle;
                if !vehicle.m_pVehicleInfo.is_null() {
                    let vehicle_info = &*vehicle.m_pVehicleInfo;
                    if vehicle_info.r#type != VH_FIGHTER && vehicle_info.r#type != VH_SPEEDER {
                        G2Angles[PITCH] = 0.0f32;
                        G2Angles[ROLL] = 0.0f32;
                    }
                }
            }
        }
    }

    // go away and get me the bolt position for this frame please
    let doesBoltExist = gi_G2API_GetBoltMatrix(
        (*cent).gent as *mut c_void,
        modelNum,
        boltNum,
        &mut boltMatrix,
        &G2Angles,
        &(*cent).lerpOrigin,
        cg.time,
        cgs.model_draw,
        (*cent).currentState.modelScale,
    );

    // set up the axis and origin we need for the actual effect spawning
    (*origin)[0] = boltMatrix.matrix[0][3];
    (*origin)[1] = boltMatrix.matrix[1][3];
    (*origin)[2] = boltMatrix.matrix[2][3];

    (*(*axis.add(1)))[0] = boltMatrix.matrix[0][0];
    (*(*axis.add(1)))[1] = boltMatrix.matrix[1][0];
    (*(*axis.add(1)))[2] = boltMatrix.matrix[2][0];

    (*(*axis))[0] = boltMatrix.matrix[0][1];
    (*(*axis))[1] = boltMatrix.matrix[1][1];
    (*(*axis))[2] = boltMatrix.matrix[2][1];

    (*(*axis.add(2)))[0] = boltMatrix.matrix[0][2];
    (*(*axis.add(2)))[1] = boltMatrix.matrix[1][2];
    (*(*axis.add(2)))[2] = boltMatrix.matrix[2][2];

    doesBoltExist
}

// #ifdef _IMMERSION
// //------------------------------------------------------
// ffHandle_t SFxHelper::RegisterForce( const char *force, int channel )
// {
// 	return cgi_FF_Register( force, channel );
// }
//
// //------------------------------------------------------
// void SFxHelper::PlayForce( int entityNum, ffHandle_t ff )
// {
// 	cgi_FF_Start( ff, entityNum );
// }
// #endif // _IMMERSION

pub unsafe extern "C" fn SFxHelper_RegisterForce(_this: *mut SFxHelper, force: *const c_char, channel: c_int) -> ffHandle_t {
    cgi_FF_Register(force, channel)
}

pub unsafe extern "C" fn SFxHelper_PlayForce(_this: *mut SFxHelper, entityNum: c_int, ff: ffHandle_t) {
    cgi_FF_Start(ff, entityNum);
}
