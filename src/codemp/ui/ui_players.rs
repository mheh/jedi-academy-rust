// Copyright (C) 1999-2000 Id Software, Inc.
//
// ui_players.c

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use crate::codemp::game::q_shared_h::{
    vec3_t, vec4_t, qboolean, qhandle_t, sfxHandle_t, fileHandle_t, clipHandle_t,
    orientation_t, animation_t, MAX_QPATH, MAX_TOTALANIMATIONS,
};

// Forward declarations for types from ui_local.h
#[repr(C)]
#[derive(Copy, Clone)]
pub struct lerpFrame_t {
    pub oldFrame: c_int,
    pub oldFrameTime: c_int,       // time when ->oldFrame was exactly on
    pub frame: c_int,
    pub frameTime: c_int,          // time when ->frame will be exactly on
    pub backlerp: f32,
    pub yawAngle: f32,
    pub yawing: qboolean,
    pub pitchAngle: f32,
    pub pitching: qboolean,
    pub animationNumber: c_int,
    pub animation: *mut animation_t,
    pub animationTime: c_int,      // time when the first frame of the animation will be exact
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct playerInfo_t {
    // model info
    pub legsModel: qhandle_t,
    pub legsSkin: qhandle_t,
    pub legs: lerpFrame_t,
    pub torsoModel: qhandle_t,
    pub torsoSkin: qhandle_t,
    pub torso: lerpFrame_t,
    pub headModel: qhandle_t,
    pub headSkin: qhandle_t,
    pub animations: [animation_t; MAX_TOTALANIMATIONS],
    pub weaponModel: qhandle_t,
    pub barrelModel: qhandle_t,
    pub flashModel: qhandle_t,
    pub flashDlightColor: vec3_t,
    pub muzzleFlashTime: c_int,
    // currently in use drawing parms
    pub viewAngles: vec3_t,
    pub moveAngles: vec3_t,
    pub currentWeapon: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    // animation vars
    pub weapon: c_int,
    pub lastWeapon: c_int,
    pub pendingWeapon: c_int,
    pub weaponTimer: c_int,
    pub pendingLegsAnim: c_int,
    pub torsoAnimationTimer: c_int,
    pub pendingTorsoAnim: c_int,
    pub legsAnimationTimer: c_int,
    pub chat: qboolean,
    pub newModel: qboolean,
    pub barrelSpinning: qboolean,
    pub barrelAngle: f32,
    pub barrelTime: c_int,
    pub realWeapon: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct refdef_t {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub fov_x: f32,
    pub fov_y: f32,
    pub vieworg: vec3_t,
    pub viewangles: vec3_t,
    pub viewaxis: [vec3_t; 3],     // transformation matrix
    pub viewContents: c_int,        // world contents at vieworg
    pub time: c_int,                // time in milliseconds for shader effects and other time dependent rendering issues
    pub rdflags: c_int,             // RDF_NOWORLDMODEL, etc
    pub areamask: [u8; 32],         // 1 bits will prevent the associated area from rendering at all
    pub text: [[c_char; 32]; 8],    // text messages for deform text shaders
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct refEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
    pub hModel: qhandle_t,
    pub axis: [vec3_t; 3],         // rotation vectors
    pub nonNormalizedAxes: qboolean,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub customShader: qhandle_t,
    pub shaderRGBA: [u8; 4],
    pub shaderTexCoord: [f32; 2],
    pub radius: f32,
    pub rotation: f32,
    pub shaderTime: f32,
    pub frame: c_int,
    pub lightingOrigin: vec3_t,
    pub shadowPlane: f32,
    pub oldframe: c_int,
    pub backlerp: f32,
    pub skinNum: c_int,
    pub customSkin: qhandle_t,
    pub endTime: f32,
    pub saberLength: f32,
    pub angles: vec3_t,
    pub modelScale: vec3_t,
    pub ghoul2: *mut c_void,
}

#[derive(Copy, Clone)]
pub struct uiInfo_t {
    pub uiDC: displayContextDef_t,
}

#[derive(Copy, Clone)]
pub struct displayContextDef_t {
    pub frameTime: c_int,
}

// Macros for animation constants
const UI_TIMER_GESTURE: c_int = 2300;
const UI_TIMER_JUMP: c_int = 1000;
const UI_TIMER_LAND: c_int = 130;
const UI_TIMER_WEAPON_SWITCH: c_int = 300;
const UI_TIMER_ATTACK: c_int = 500;
const UI_TIMER_MUZZLE_FLASH: c_int = 20;
const UI_TIMER_WEAPON_DELAY: c_int = 250;

const JUMP_HEIGHT: f32 = 56.0;

const SWINGSPEED: f32 = 0.3;

const SPIN_SPEED: f32 = 0.9;
const COAST_TIME: c_int = 1000;

// Animation toggle bit for legacy animation system
const ANIM_TOGGLEBIT: c_int = 0x01000000;

// Weapon constants (from bg_weapons.h equivalent)
const WP_NONE: c_int = 0;
const WP_STUN_BATON: c_int = 1;
const WP_SABER: c_int = 2;
const WP_BRYAR_PISTOL: c_int = 3;
const WP_BLASTER: c_int = 4;
const WP_DISRUPTOR: c_int = 5;
const WP_BOWCASTER: c_int = 6;
const WP_REPEATER: c_int = 7;
const WP_DEMP2: c_int = 8;
const WP_FLECHETTE: c_int = 9;
const WP_ROCKET_LAUNCHER: c_int = 10;
const WP_THERMAL: c_int = 11;
const WP_TRIP_MINE: c_int = 12;
const WP_DET_PACK: c_int = 13;

// Animation constants from anims.h
const BOTH_JUMP1: c_int = 26;
const BOTH_LAND1: c_int = 27;
const BOTH_GESTURE1: c_int = 123;
const BOTH_ATTACK3: c_int = 142;
const BOTH_A1_T__B_: c_int = 145;
const BOTH_DEATH1: c_int = 0;
const BOTH_STAND2: c_int = 133;
const TORSO_WEAPONREADY3: c_int = 214;
const TORSO_DROPWEAP1: c_int = 215;
const TORSO_RAISEWEAP1: c_int = 216;

// Angle indices
const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

// Render effect flags from tr_types.h
const RF_LIGHTING_ORIGIN: c_int = 0x00080;
const RF_NOSHADOW: c_int = 0x00040;

// Render type flags
const RT_SPRITE: c_int = 2;

// Render def flags
const RDF_NOWORLDMODEL: c_int = 1;

// Channel constants
const CHAN_LOCAL: c_int = 1;

// Static globals
static mut dp_realtime: c_int = 0;
static mut jumpHeight: f32 = 0.0;
pub static mut weaponChangeSound: sfxHandle_t = 0;

// External trap functions
extern "C" {
    pub fn trap_R_RegisterModel(name: *const c_char) -> qhandle_t;
    pub fn trap_R_RegisterSkin(name: *const c_char) -> qhandle_t;
    pub fn trap_R_RegisterShaderNoMip(name: *const c_char) -> qhandle_t;
    pub fn trap_CM_LerpTag(
        tag: *mut orientation_t,
        mod_: clipHandle_t,
        startFrame: c_int,
        endFrame: c_int,
        frac: f32,
        tagName: *const c_char,
    ) -> c_int;
    pub fn trap_Error(string: *const c_char);
    pub fn trap_R_AddRefEntityToScene(re: *const refEntity_t);
    pub fn trap_R_ClearScene();
    pub fn trap_R_RenderScene(fd: *const refdef_t);
    pub fn trap_R_AddLightToScene(org: *const vec3_t, intensity: f32, r: f32, g: f32, b: f32);
    pub fn trap_S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int);
    pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut fileHandle_t, mode: c_int) -> c_int;
    pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: fileHandle_t);
    pub fn trap_FS_FCloseFile(f: fileHandle_t);
    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    fn strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: c_int);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn atoi(nptr: *const c_char) -> c_int;
    fn atof(nptr: *const c_char) -> f32;
    fn fabs(x: f32) -> f32;
    fn sin(x: f32) -> f32;
    fn tan(x: f32) -> f32;
    fn atan2(y: f32, x: f32) -> f32;
    pub fn AngleMod(anum: f32) -> f32;
    pub fn AngleSubtract(angle1: f32, angle2: f32) -> f32;
    pub fn AnglesSubtract(a1: *const vec3_t, a2: *const vec3_t, a3: *mut vec3_t);
    pub fn AnglesToAxis(angles: *const vec3_t, axis: *mut [vec3_t; 3]);
    pub fn AngleVectors(angles: *const vec3_t, forward: *mut vec3_t, right: *mut vec3_t, up: *mut vec3_t);
    pub fn VectorMA(veca: *const vec3_t, scale: f32, vecb: *const vec3_t, vecc: *mut vec3_t);
    pub fn MatrixMultiply(a: *const [vec3_t; 3], b: *const [vec3_t; 3], c: *mut [vec3_t; 3]);
    pub fn AxisClear(axis: *mut [vec3_t; 3]);
    pub fn _VectorCopy(input: *const vec3_t, output: *mut vec3_t);
    pub fn Q_fabs(x: f32) -> f32;
    pub fn va(fmt: *const c_char, ...) -> *const c_char;
}

pub extern "C" fn UI_PlayerInfo_SetWeapon(pi: *mut playerInfo_t, weaponNum: c_int) {
    let mut item: *const gitem_t;
    let mut path: [c_char; MAX_QPATH] = [0; MAX_QPATH];

    unsafe {
        (*pi).currentWeapon = weaponNum;
        loop {
            (*pi).realWeapon = weaponNum;
            (*pi).weaponModel = 0;
            (*pi).barrelModel = 0;
            (*pi).flashModel = 0;

            if weaponNum == WP_NONE {
                return;
            }

            item = core::ptr::addr_of!(bg_itemlist) as *const gitem_t;
            item = item.add(1);
            loop {
                if (*item).classname.is_null() {
                    break;
                }
                if (*item).giType != IT_WEAPON {
                    item = item.add(1);
                    continue;
                }
                if (*item).giTag == weaponNum {
                    break;
                }
                item = item.add(1);
            }

            if !(*item).classname.is_null() {
                (*pi).weaponModel = trap_R_RegisterModel((*item).world_model[0]);
            }

            if (*pi).weaponModel == 0 {
                if weaponNum == WP_BRYAR_PISTOL {
                    let mut weaponNum = WP_NONE;
                    continue;
                }
                let mut weaponNum = WP_BRYAR_PISTOL;
                continue;
            }
            // commented out section from original C
            // if ( weaponNum == WP_MACHINEGUN || weaponNum == WP_BFG ) {
            //     strcpy( path, item->world_model[0] );
            //     COM_StripExtension( path, path );
            //     strcat( path, "_barrel.md3" );
            //     pi->barrelModel = trap_R_RegisterModel( path );
            // }

            strcpy(path.as_mut_ptr(), (*item).world_model[0]);
            COM_StripExtension(path.as_mut_ptr(), path.as_mut_ptr());
            strcat(path.as_mut_ptr(), "_flash.md3\0".as_ptr() as *const c_char);
            (*pi).flashModel = trap_R_RegisterModel(path.as_ptr());

            match weaponNum {
                WP_STUN_BATON | WP_SABER => {
                    // MAKERGB( pi->flashDlightColor, 0.6f, 0.6f, 1 );
                    (*pi).flashDlightColor[0] = 0.6;
                    (*pi).flashDlightColor[1] = 0.6;
                    (*pi).flashDlightColor[2] = 1.0;
                }
                WP_BRYAR_PISTOL | WP_BLASTER | WP_DISRUPTOR | WP_BOWCASTER | WP_REPEATER
                | WP_DEMP2 | WP_FLECHETTE | WP_ROCKET_LAUNCHER | WP_THERMAL | WP_TRIP_MINE
                | WP_DET_PACK => {
                    // MAKERGB( pi->flashDlightColor, 1, 1, 0 );
                    (*pi).flashDlightColor[0] = 1.0;
                    (*pi).flashDlightColor[1] = 1.0;
                    (*pi).flashDlightColor[2] = 0.0;
                }
                _ => {
                    // MAKERGB( pi->flashDlightColor, 1, 1, 1 );
                    (*pi).flashDlightColor[0] = 1.0;
                    (*pi).flashDlightColor[1] = 1.0;
                    (*pi).flashDlightColor[2] = 1.0;
                }
            }

            break;
        }
    }
}

pub extern "C" fn UI_ForceLegsAnim(pi: *mut playerInfo_t, anim: c_int) {
    unsafe {
        (*pi).legsAnim = (((*pi).legsAnim & ANIM_TOGGLEBIT) ^ ANIM_TOGGLEBIT) | anim;

        if anim == BOTH_JUMP1 {
            (*pi).legsAnimationTimer = UI_TIMER_JUMP;
        }
    }
}

pub extern "C" fn UI_SetLegsAnim(pi: *mut playerInfo_t, anim: c_int) {
    unsafe {
        let mut anim = anim;
        if (*pi).pendingLegsAnim != 0 {
            anim = (*pi).pendingLegsAnim;
            (*pi).pendingLegsAnim = 0;
        }
        UI_ForceLegsAnim(pi, anim);
    }
}

pub extern "C" fn UI_ForceTorsoAnim(pi: *mut playerInfo_t, anim: c_int) {
    unsafe {
        (*pi).torsoAnim = (((*pi).torsoAnim & ANIM_TOGGLEBIT) ^ ANIM_TOGGLEBIT) | anim;

        if anim == BOTH_GESTURE1 {
            (*pi).torsoAnimationTimer = UI_TIMER_GESTURE;
        }

        if anim == BOTH_ATTACK3 || anim == BOTH_A1_T__B_ {
            (*pi).torsoAnimationTimer = UI_TIMER_ATTACK;
        }
    }
}

pub extern "C" fn UI_SetTorsoAnim(pi: *mut playerInfo_t, anim: c_int) {
    unsafe {
        let mut anim = anim;
        if (*pi).pendingTorsoAnim != 0 {
            anim = (*pi).pendingTorsoAnim;
            (*pi).pendingTorsoAnim = 0;
        }

        UI_ForceTorsoAnim(pi, anim);
    }
}

pub extern "C" fn UI_TorsoSequencing(pi: *mut playerInfo_t) {
    unsafe {
        let currentAnim = (*pi).torsoAnim & !ANIM_TOGGLEBIT;

        if (*pi).weapon != (*pi).currentWeapon {
            if currentAnim != TORSO_DROPWEAP1 {
                (*pi).torsoAnimationTimer = UI_TIMER_WEAPON_SWITCH;
                UI_ForceTorsoAnim(pi, TORSO_DROPWEAP1);
            }
        }

        if (*pi).torsoAnimationTimer > 0 {
            return;
        }

        if currentAnim == BOTH_GESTURE1 {
            UI_SetTorsoAnim(pi, TORSO_WEAPONREADY3);
            return;
        }

        if currentAnim == BOTH_ATTACK3 || currentAnim == BOTH_A1_T__B_ {
            UI_SetTorsoAnim(pi, TORSO_WEAPONREADY3);
            return;
        }

        if currentAnim == TORSO_DROPWEAP1 {
            UI_PlayerInfo_SetWeapon(pi, (*pi).weapon);
            (*pi).torsoAnimationTimer = UI_TIMER_WEAPON_SWITCH;
            UI_ForceTorsoAnim(pi, TORSO_RAISEWEAP1);
            return;
        }

        if currentAnim == TORSO_RAISEWEAP1 {
            UI_SetTorsoAnim(pi, TORSO_WEAPONREADY3);
            return;
        }
    }
}

pub extern "C" fn UI_LegsSequencing(pi: *mut playerInfo_t) {
    unsafe {
        let currentAnim = (*pi).legsAnim & !ANIM_TOGGLEBIT;

        if (*pi).legsAnimationTimer > 0 {
            if currentAnim == BOTH_JUMP1 {
                jumpHeight = JUMP_HEIGHT
                    * sin(std::f32::consts::PI * (UI_TIMER_JUMP - (*pi).legsAnimationTimer) as f32
                        / UI_TIMER_JUMP as f32);
            }
            return;
        }

        if currentAnim == BOTH_JUMP1 {
            UI_ForceLegsAnim(pi, BOTH_LAND1);
            (*pi).legsAnimationTimer = UI_TIMER_LAND;
            jumpHeight = 0.0;
            return;
        }

        if currentAnim == BOTH_LAND1 {
            UI_SetLegsAnim(pi, TORSO_WEAPONREADY3);
            return;
        }
    }
}

pub extern "C" fn UI_PositionEntityOnTag(
    entity: *mut refEntity_t,
    parent: *const refEntity_t,
    parentModel: clipHandle_t,
    tagName: *mut c_char,
) {
    unsafe {
        let mut lerped: orientation_t = core::mem::zeroed();

        // lerp the tag
        trap_CM_LerpTag(
            core::ptr::addr_of_mut!(lerped),
            parentModel,
            (*parent).oldframe,
            (*parent).frame,
            1.0 - (*parent).backlerp,
            tagName,
        );

        // FIXME: allow origin offsets along tag?
        _VectorCopy(core::ptr::addr_of!((*parent).origin), core::ptr::addr_of_mut!((*entity).origin));
        for i in 0..3 {
            VectorMA(
                core::ptr::addr_of!((*entity).origin),
                lerped.origin[i],
                core::ptr::addr_of!((*parent).axis[i]),
                core::ptr::addr_of_mut!((*entity).origin),
            );
        }

        // cast away const because of compiler problems
        MatrixMultiply(
            core::ptr::addr_of!(lerped.axis),
            core::ptr::addr_of!((*((parent as *const refEntity_t) as *mut refEntity_t)).axis),
            core::ptr::addr_of_mut!((*entity).axis),
        );
        (*entity).backlerp = (*parent).backlerp;
    }
}

pub extern "C" fn UI_PositionRotatedEntityOnTag(
    entity: *mut refEntity_t,
    parent: *const refEntity_t,
    parentModel: clipHandle_t,
    tagName: *mut c_char,
) {
    unsafe {
        let mut lerped: orientation_t = core::mem::zeroed();
        let mut tempAxis: [vec3_t; 3] = [[0.0; 3]; 3];

        // lerp the tag
        trap_CM_LerpTag(
            core::ptr::addr_of_mut!(lerped),
            parentModel,
            (*parent).oldframe,
            (*parent).frame,
            1.0 - (*parent).backlerp,
            tagName,
        );

        // FIXME: allow origin offsets along tag?
        _VectorCopy(core::ptr::addr_of!((*parent).origin), core::ptr::addr_of_mut!((*entity).origin));
        for i in 0..3 {
            VectorMA(
                core::ptr::addr_of!((*entity).origin),
                lerped.origin[i],
                core::ptr::addr_of!((*parent).axis[i]),
                core::ptr::addr_of_mut!((*entity).origin),
            );
        }

        // cast away const because of compiler problems
        MatrixMultiply(
            core::ptr::addr_of!((*entity).axis),
            core::ptr::addr_of!((*((parent as *const refEntity_t) as *mut refEntity_t)).axis),
            core::ptr::addr_of_mut!(tempAxis),
        );
        MatrixMultiply(
            core::ptr::addr_of!(lerped.axis),
            core::ptr::addr_of!(tempAxis),
            core::ptr::addr_of_mut!((*entity).axis),
        );
    }
}

pub extern "C" fn UI_SetLerpFrameAnimation(
    ci: *mut playerInfo_t,
    lf: *mut lerpFrame_t,
    newAnimation: c_int,
) {
    unsafe {
        (*lf).animationNumber = newAnimation;
        let newAnimation = newAnimation & !ANIM_TOGGLEBIT;

        if newAnimation < 0 || newAnimation >= MAX_TOTALANIMATIONS as c_int {
            let msg = va("Bad animation number: %i\0".as_ptr() as *const c_char, newAnimation);
            trap_Error(msg);
        }

        let anim = core::ptr::addr_of_mut!((*ci).animations[newAnimation as usize]);

        (*lf).animation = anim;
        (*lf).animationTime = (*lf).frameTime + (*anim).initialLerp as c_int;
    }
}

pub extern "C" fn UI_RunLerpFrame(
    ci: *mut playerInfo_t,
    lf: *mut lerpFrame_t,
    newAnimation: c_int,
) {
    unsafe {
        let mut f: c_int;
        let anim: *mut animation_t;

        // see if the animation sequence is switching
        if newAnimation != (*lf).animationNumber || (*lf).animation.is_null() {
            UI_SetLerpFrameAnimation(ci, lf, newAnimation);
        }

        // if we have passed the current frame, move it to
        // oldFrame and calculate a new frame
        if dp_realtime >= (*lf).frameTime {
            (*lf).oldFrame = (*lf).frame;
            (*lf).oldFrameTime = (*lf).frameTime;

            // get the next frame based on the animation
            anim = (*lf).animation;
            if dp_realtime < (*lf).animationTime {
                (*lf).frameTime = (*lf).animationTime;        // initial lerp
            } else {
                (*lf).frameTime = (*lf).oldFrameTime + (*anim).frameLerp as c_int;
            }
            f = ((*lf).frameTime - (*lf).animationTime) / (*anim).frameLerp as c_int;
            if f >= (*anim).numFrames as c_int {
                f -= (*anim).numFrames as c_int;
                if (*anim).loopFrames != 0 {
                    f %= (*anim).loopFrames as c_int;
                    f += (*anim).numFrames as c_int - (*anim).loopFrames as c_int;
                } else {
                    f = (*anim).numFrames as c_int - 1;
                    // the animation is stuck at the end, so it
                    // can immediately transition to another sequence
                    (*lf).frameTime = dp_realtime;
                }
            }
            (*lf).frame = (*anim).firstFrame as c_int + f;
            if dp_realtime > (*lf).frameTime {
                (*lf).frameTime = dp_realtime;
            }
        }

        if (*lf).frameTime > dp_realtime + 200 {
            (*lf).frameTime = dp_realtime;
        }

        if (*lf).oldFrameTime > dp_realtime {
            (*lf).oldFrameTime = dp_realtime;
        }
        // calculate current lerp value
        if (*lf).frameTime == (*lf).oldFrameTime {
            (*lf).backlerp = 0.0;
        } else {
            (*lf).backlerp = 1.0
                - (dp_realtime - (*lf).oldFrameTime) as f32 / ((*lf).frameTime - (*lf).oldFrameTime)
                    as f32;
        }
    }
}

pub extern "C" fn UI_PlayerAnimation(
    pi: *mut playerInfo_t,
    legsOld: *mut c_int,
    legs: *mut c_int,
    legsBackLerp: *mut f32,
    torsoOld: *mut c_int,
    torso: *mut c_int,
    torsoBackLerp: *mut f32,
) {
    unsafe {
        // legs animation
        (*pi).legsAnimationTimer -= uiInfo.uiDC.frameTime;
        if (*pi).legsAnimationTimer < 0 {
            (*pi).legsAnimationTimer = 0;
        }

        UI_LegsSequencing(pi);

        if (*pi).legs.yawing != 0
            && ((*pi).legsAnim & !ANIM_TOGGLEBIT) == TORSO_WEAPONREADY3
        {
            UI_RunLerpFrame(pi, core::ptr::addr_of_mut!((*pi).legs), TORSO_WEAPONREADY3);
        } else {
            UI_RunLerpFrame(pi, core::ptr::addr_of_mut!((*pi).legs), (*pi).legsAnim);
        }
        *legsOld = (*pi).legs.oldFrame;
        *legs = (*pi).legs.frame;
        *legsBackLerp = (*pi).legs.backlerp;

        // torso animation
        (*pi).torsoAnimationTimer -= uiInfo.uiDC.frameTime;
        if (*pi).torsoAnimationTimer < 0 {
            (*pi).torsoAnimationTimer = 0;
        }

        UI_TorsoSequencing(pi);

        UI_RunLerpFrame(pi, core::ptr::addr_of_mut!((*pi).torso), (*pi).torsoAnim);
        *torsoOld = (*pi).torso.oldFrame;
        *torso = (*pi).torso.frame;
        *torsoBackLerp = (*pi).torso.backlerp;
    }
}

pub extern "C" fn UI_SwingAngles(
    destination: f32,
    swingTolerance: f32,
    clampTolerance: f32,
    speed: f32,
    angle: *mut f32,
    swinging: *mut qboolean,
) {
    unsafe {
        let mut swing: f32;
        let mut move_: f32;
        let mut scale: f32;

        if *swinging == 0 {
            // see if a swing should be started
            swing = AngleSubtract(*angle, destination);
            if swing > swingTolerance || swing < -swingTolerance {
                *swinging = 1;
            }
        }

        if *swinging == 0 {
            return;
        }

        // modify the speed depending on the delta
        // so it doesn't seem so linear
        swing = AngleSubtract(destination, *angle);
        scale = fabs(swing);
        if scale < swingTolerance * 0.5 {
            scale = 0.5;
        } else if scale < swingTolerance {
            scale = 1.0;
        } else {
            scale = 2.0;
        }

        // swing towards the destination angle
        if swing >= 0.0 {
            move_ = uiInfo.uiDC.frameTime as f32 * scale * speed;
            if move_ >= swing {
                move_ = swing;
                *swinging = 0;
            }
            *angle = AngleMod(*angle + move_);
        } else if swing < 0.0 {
            move_ = uiInfo.uiDC.frameTime as f32 * scale * -speed;
            if move_ <= swing {
                move_ = swing;
                *swinging = 0;
            }
            *angle = AngleMod(*angle + move_);
        }

        // clamp to no more than tolerance
        swing = AngleSubtract(destination, *angle);
        if swing > clampTolerance {
            *angle = AngleMod(destination - (clampTolerance - 1.0));
        } else if swing < -clampTolerance {
            *angle = AngleMod(destination + (clampTolerance - 1.0));
        }
    }
}

pub extern "C" fn UI_MovedirAdjustment(pi: *const playerInfo_t) -> f32 {
    unsafe {
        let mut relativeAngles: vec3_t = [0.0; 3];
        let mut moveVector: vec3_t = [0.0; 3];

        // VectorSubtract( pi->viewAngles, pi->moveAngles, relativeAngles );
        relativeAngles[0] = (*pi).viewAngles[0] - (*pi).moveAngles[0];
        relativeAngles[1] = (*pi).viewAngles[1] - (*pi).moveAngles[1];
        relativeAngles[2] = (*pi).viewAngles[2] - (*pi).moveAngles[2];

        AngleVectors(
            core::ptr::addr_of!(relativeAngles),
            core::ptr::addr_of_mut!(moveVector),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if Q_fabs(moveVector[0]) < 0.01 {
            moveVector[0] = 0.0;
        }
        if Q_fabs(moveVector[1]) < 0.01 {
            moveVector[1] = 0.0;
        }

        if moveVector[1] == 0.0 && moveVector[0] > 0.0 {
            return 0.0;
        }
        if moveVector[1] < 0.0 && moveVector[0] > 0.0 {
            return 22.0;
        }
        if moveVector[1] < 0.0 && moveVector[0] == 0.0 {
            return 45.0;
        }
        if moveVector[1] < 0.0 && moveVector[0] < 0.0 {
            return -22.0;
        }
        if moveVector[1] == 0.0 && moveVector[0] < 0.0 {
            return 0.0;
        }
        if moveVector[1] > 0.0 && moveVector[0] < 0.0 {
            return 22.0;
        }
        if moveVector[1] > 0.0 && moveVector[0] == 0.0 {
            return -45.0;
        }

        return -22.0;
    }
}

pub extern "C" fn UI_PlayerAngles(
    pi: *mut playerInfo_t,
    legs: *mut [vec3_t; 3],
    torso: *mut [vec3_t; 3],
    head: *mut [vec3_t; 3],
) {
    unsafe {
        let mut legsAngles: vec3_t = [0.0; 3];
        let mut torsoAngles: vec3_t = [0.0; 3];
        let mut headAngles: vec3_t = [0.0; 3];
        let mut dest: f32;
        let mut adjust: f32;

        _VectorCopy(
            core::ptr::addr_of!((*pi).viewAngles),
            core::ptr::addr_of_mut!(headAngles),
        );
        headAngles[YAW] = AngleMod(headAngles[YAW]);
        // VectorClear( legsAngles );
        legsAngles[0] = 0.0;
        legsAngles[1] = 0.0;
        legsAngles[2] = 0.0;
        // VectorClear( torsoAngles );
        torsoAngles[0] = 0.0;
        torsoAngles[1] = 0.0;
        torsoAngles[2] = 0.0;

        // --------- yaw -------------

        // allow yaw to drift a bit
        if ((*pi).legsAnim & !ANIM_TOGGLEBIT) != TORSO_WEAPONREADY3
            || ((*pi).torsoAnim & !ANIM_TOGGLEBIT) != TORSO_WEAPONREADY3
        {
            // if not standing still, always point all in the same direction
            (*pi).torso.yawing = 1;      // always center
            (*pi).torso.pitching = 1;    // always center
            (*pi).legs.yawing = 1;       // always center
        }

        // adjust legs for movement dir
        adjust = UI_MovedirAdjustment(pi);
        legsAngles[YAW] = headAngles[YAW] + adjust;
        torsoAngles[YAW] = headAngles[YAW] + 0.25 * adjust;

        // torso
        UI_SwingAngles(
            torsoAngles[YAW],
            25.0,
            90.0,
            SWINGSPEED,
            core::ptr::addr_of_mut!((*pi).torso.yawAngle),
            core::ptr::addr_of_mut!((*pi).torso.yawing),
        );
        UI_SwingAngles(
            legsAngles[YAW],
            40.0,
            90.0,
            SWINGSPEED,
            core::ptr::addr_of_mut!((*pi).legs.yawAngle),
            core::ptr::addr_of_mut!((*pi).legs.yawing),
        );

        torsoAngles[YAW] = (*pi).torso.yawAngle;
        legsAngles[YAW] = (*pi).legs.yawAngle;

        // --------- pitch -------------

        // only show a fraction of the pitch angle in the torso
        if headAngles[PITCH] > 180.0 {
            dest = (-360.0 + headAngles[PITCH]) * 0.75;
        } else {
            dest = headAngles[PITCH] * 0.75;
        }
        UI_SwingAngles(
            dest,
            15.0,
            30.0,
            0.1,
            core::ptr::addr_of_mut!((*pi).torso.pitchAngle),
            core::ptr::addr_of_mut!((*pi).torso.pitching),
        );
        torsoAngles[PITCH] = (*pi).torso.pitchAngle;

        // pull the angles back out of the hierarchial chain
        AnglesSubtract(
            core::ptr::addr_of!(headAngles),
            core::ptr::addr_of!(torsoAngles),
            core::ptr::addr_of_mut!(headAngles),
        );
        AnglesSubtract(
            core::ptr::addr_of!(torsoAngles),
            core::ptr::addr_of!(legsAngles),
            core::ptr::addr_of_mut!(torsoAngles),
        );
        AnglesToAxis(core::ptr::addr_of!(legsAngles), legs);
        AnglesToAxis(core::ptr::addr_of!(torsoAngles), torso);
        AnglesToAxis(core::ptr::addr_of!(headAngles), head);
    }
}

pub extern "C" fn UI_PlayerFloatSprite(
    pi: *const playerInfo_t,
    origin: *const vec3_t,
    shader: qhandle_t,
) {
    unsafe {
        let mut ent: refEntity_t = core::mem::zeroed();

        memset(
            core::ptr::addr_of_mut!(ent) as *mut c_void,
            0,
            core::mem::size_of::<refEntity_t>(),
        );
        _VectorCopy(origin, core::ptr::addr_of_mut!(ent.origin));
        ent.origin[2] += 48.0;
        ent.reType = RT_SPRITE;
        ent.customShader = shader;
        ent.radius = 10.0;
        ent.renderfx = 0;
        trap_R_AddRefEntityToScene(core::ptr::addr_of!(ent));
    }
}

pub extern "C" fn UI_MachinegunSpinAngle(pi: *const playerInfo_t) -> f32 {
    unsafe {
        let mut delta: c_int;
        let mut angle: f32;
        let mut speed: f32;
        let mut torsoAnim: c_int;

        delta = dp_realtime - (*pi).barrelTime;
        if (*pi).barrelSpinning != 0 {
            angle = (*pi).barrelAngle + delta as f32 * SPIN_SPEED;
        } else {
            if delta > COAST_TIME {
                delta = COAST_TIME;
            }

            speed = 0.5 * (SPIN_SPEED + (COAST_TIME - delta) as f32 / COAST_TIME as f32);
            angle = (*pi).barrelAngle + delta as f32 * speed;
        }

        torsoAnim = (*pi).torsoAnim & !ANIM_TOGGLEBIT;
        if torsoAnim == BOTH_A1_T__B_ {
            torsoAnim = BOTH_ATTACK3;
        }
        if ((*pi).barrelSpinning != 0) != (torsoAnim == BOTH_ATTACK3) {
            (*pi).barrelTime = dp_realtime;
            (*pi).barrelAngle = AngleMod(angle);
            (*pi).barrelSpinning = if torsoAnim == BOTH_ATTACK3 { 1 } else { 0 };
        }

        return angle;
    }
}

pub extern "C" fn UI_DrawPlayer(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    pi: *mut playerInfo_t,
    time: c_int,
) {
    unsafe {
        let mut refdef: refdef_t = core::mem::zeroed();
        let mut legs: refEntity_t = core::mem::zeroed();
        let mut torso: refEntity_t = core::mem::zeroed();
        let mut head: refEntity_t = core::mem::zeroed();
        let mut gun: refEntity_t = core::mem::zeroed();
        let mut flash: refEntity_t = core::mem::zeroed();
        let mut origin: vec3_t = [0.0; 3];
        let mut renderfx: c_int;
        let mins: vec3_t = [-16.0, -16.0, -24.0];
        let maxs: vec3_t = [16.0, 16.0, 32.0];
        let mut len: f32;
        let mut xx: f32;

        if (*pi).legsModel == 0
            || (*pi).torsoModel == 0
            || (*pi).headModel == 0
            || (*pi).animations[0].numFrames == 0
        {
            return;
        }

        // this allows the ui to cache the player model on the main menu
        if w == 0.0 || h == 0.0 {
            return;
        }

        dp_realtime = time;

        if (*pi).pendingWeapon != -1 && dp_realtime > (*pi).weaponTimer {
            (*pi).weapon = (*pi).pendingWeapon;
            (*pi).lastWeapon = (*pi).pendingWeapon;
            (*pi).pendingWeapon = -1;
            (*pi).weaponTimer = 0;
            if (*pi).currentWeapon != (*pi).weapon {
                trap_S_StartLocalSound(weaponChangeSound, CHAN_LOCAL);
            }
        }

        let y = y - jumpHeight;

        memset(
            core::ptr::addr_of_mut!(refdef) as *mut c_void,
            0,
            core::mem::size_of::<refdef_t>(),
        );
        memset(
            core::ptr::addr_of_mut!(legs) as *mut c_void,
            0,
            core::mem::size_of::<refEntity_t>(),
        );
        memset(
            core::ptr::addr_of_mut!(torso) as *mut c_void,
            0,
            core::mem::size_of::<refEntity_t>(),
        );
        memset(
            core::ptr::addr_of_mut!(head) as *mut c_void,
            0,
            core::mem::size_of::<refEntity_t>(),
        );

        refdef.rdflags = RDF_NOWORLDMODEL;

        AxisClear(core::ptr::addr_of_mut!(refdef.viewaxis));

        refdef.x = x as c_int;
        refdef.y = y as c_int;
        refdef.width = w as c_int;
        refdef.height = h as c_int;

        refdef.fov_x = w / 640.0 * 90.0;
        xx = w / tan(refdef.fov_x / 360.0 * std::f32::consts::PI);
        refdef.fov_y = atan2(h, xx);
        refdef.fov_y *= 360.0 / std::f32::consts::PI;

        // calculate distance so the player nearly fills the box
        len = 0.7 * (maxs[2] - mins[2]);
        origin[0] = len / tan((refdef.fov_x * std::f32::consts::PI / 180.0) * 0.5);
        origin[1] = 0.5 * (mins[1] + maxs[1]);
        origin[2] = -0.5 * (mins[2] + maxs[2]);

        refdef.time = dp_realtime;

        trap_R_ClearScene();

        // get the rotation information
        UI_PlayerAngles(
            pi,
            core::ptr::addr_of_mut!(legs.axis),
            core::ptr::addr_of_mut!(torso.axis),
            core::ptr::addr_of_mut!(head.axis),
        );

        // get the animation state (after rotation, to allow feet shuffle)
        UI_PlayerAnimation(
            pi,
            core::ptr::addr_of_mut!(legs.oldframe),
            core::ptr::addr_of_mut!(legs.frame),
            core::ptr::addr_of_mut!(legs.backlerp),
            core::ptr::addr_of_mut!(torso.oldframe),
            core::ptr::addr_of_mut!(torso.frame),
            core::ptr::addr_of_mut!(torso.backlerp),
        );

        renderfx = RF_LIGHTING_ORIGIN | RF_NOSHADOW;

        //
        // add the legs
        //
        legs.hModel = (*pi).legsModel;
        legs.customSkin = (*pi).legsSkin;

        _VectorCopy(core::ptr::addr_of!(origin), core::ptr::addr_of_mut!(legs.origin));

        _VectorCopy(core::ptr::addr_of!(origin), core::ptr::addr_of_mut!(legs.lightingOrigin));
        legs.renderfx = renderfx;
        _VectorCopy(core::ptr::addr_of!(legs.origin), core::ptr::addr_of_mut!(legs.oldorigin));

        trap_R_AddRefEntityToScene(core::ptr::addr_of!(legs));

        if legs.hModel == 0 {
            return;
        }

        //
        // add the torso
        //
        torso.hModel = (*pi).torsoModel;
        if torso.hModel == 0 {
            return;
        }

        torso.customSkin = (*pi).torsoSkin;

        _VectorCopy(core::ptr::addr_of!(origin), core::ptr::addr_of_mut!(torso.lightingOrigin));

        UI_PositionRotatedEntityOnTag(
            core::ptr::addr_of_mut!(torso),
            core::ptr::addr_of!(legs),
            (*pi).legsModel,
            "tag_torso\0".as_ptr() as *mut c_char,
        );

        torso.renderfx = renderfx;

        trap_R_AddRefEntityToScene(core::ptr::addr_of!(torso));

        //
        // add the head
        //
        head.hModel = (*pi).headModel;
        if head.hModel == 0 {
            return;
        }
        head.customSkin = (*pi).headSkin;

        _VectorCopy(core::ptr::addr_of!(origin), core::ptr::addr_of_mut!(head.lightingOrigin));

        UI_PositionRotatedEntityOnTag(
            core::ptr::addr_of_mut!(head),
            core::ptr::addr_of!(torso),
            (*pi).torsoModel,
            "tag_head\0".as_ptr() as *mut c_char,
        );

        head.renderfx = renderfx;

        trap_R_AddRefEntityToScene(core::ptr::addr_of!(head));

        //
        // add the gun
        //
        if (*pi).currentWeapon != WP_NONE {
            memset(
                core::ptr::addr_of_mut!(gun) as *mut c_void,
                0,
                core::mem::size_of::<refEntity_t>(),
            );
            gun.hModel = (*pi).weaponModel;
            _VectorCopy(core::ptr::addr_of!(origin), core::ptr::addr_of_mut!(gun.lightingOrigin));
            UI_PositionEntityOnTag(
                core::ptr::addr_of_mut!(gun),
                core::ptr::addr_of!(torso),
                (*pi).torsoModel,
                "tag_weapon\0".as_ptr() as *mut c_char,
            );
            gun.renderfx = renderfx;
            trap_R_AddRefEntityToScene(core::ptr::addr_of!(gun));
        }

        //
        // add the spinning barrel
        //
        // commented out in original C
        // if ( pi->realWeapon == WP_MACHINEGUN || pi->realWeapon == WP_BFG ) {
        //     vec3_t angles;
        //     memset( &barrel, 0, sizeof(barrel) );
        //     VectorCopy( origin, barrel.lightingOrigin );
        //     barrel.renderfx = renderfx;
        //     barrel.hModel = pi->barrelModel;
        //     angles[YAW] = 0;
        //     angles[PITCH] = 0;
        //     angles[ROLL] = UI_MachinegunSpinAngle( pi );
        //     if(pi->realWeapon == WP_BFG ) {
        //         angles[PITCH] = angles[ROLL];
        //         angles[ROLL] = 0;
        //     }
        //     AnglesToAxis( angles, barrel.axis );
        //     UI_PositionRotatedEntityOnTag( &barrel, &gun, pi->weaponModel, "tag_barrel");
        //     trap_R_AddRefEntityToScene( &barrel );
        // }

        //
        // add muzzle flash
        //
        if dp_realtime <= (*pi).muzzleFlashTime {
            if (*pi).flashModel != 0 {
                memset(
                    core::ptr::addr_of_mut!(flash) as *mut c_void,
                    0,
                    core::mem::size_of::<refEntity_t>(),
                );
                flash.hModel = (*pi).flashModel;
                _VectorCopy(core::ptr::addr_of!(origin), core::ptr::addr_of_mut!(flash.lightingOrigin));
                UI_PositionEntityOnTag(
                    core::ptr::addr_of_mut!(flash),
                    core::ptr::addr_of!(gun),
                    (*pi).weaponModel,
                    "tag_flash\0".as_ptr() as *mut c_char,
                );
                flash.renderfx = renderfx;
                trap_R_AddRefEntityToScene(core::ptr::addr_of!(flash));
            }

            // make a dlight for the flash
            if (*pi).flashDlightColor[0] != 0.0
                || (*pi).flashDlightColor[1] != 0.0
                || (*pi).flashDlightColor[2] != 0.0
            {
                trap_R_AddLightToScene(
                    core::ptr::addr_of!(flash.origin),
                    200.0 + ((rand() & 31) as f32),
                    (*pi).flashDlightColor[0],
                    (*pi).flashDlightColor[1],
                    (*pi).flashDlightColor[2],
                );
            }
        }

        //
        // add the chat icon
        //
        if (*pi).chat != 0 {
            UI_PlayerFloatSprite(
                pi,
                core::ptr::addr_of!(origin),
                trap_R_RegisterShaderNoMip("sprites/balloon3\0".as_ptr() as *const c_char),
            );
        }

        //
        // add an accent light
        //
        origin[0] -= 100.0;     // + = behind, - = in front
        origin[1] += 100.0;     // + = left, - = right
        origin[2] += 100.0;     // + = above, - = below
        trap_R_AddLightToScene(
            core::ptr::addr_of!(origin),
            500.0,
            1.0,
            1.0,
            1.0,
        );

        origin[0] -= 100.0;
        origin[1] -= 100.0;
        origin[2] -= 100.0;
        trap_R_AddLightToScene(
            core::ptr::addr_of!(origin),
            500.0,
            1.0,
            0.0,
            0.0,
        );

        trap_R_RenderScene(core::ptr::addr_of!(refdef));
    }
}

pub extern "C" fn UI_FileExists(filename: *const c_char) -> qboolean {
    unsafe {
        let mut len: c_int;
        let mut f: fileHandle_t = 0;

        len = trap_FS_FOpenFile(filename, core::ptr::addr_of_mut!(f), FS_READ);
        if len > 0 {
            trap_FS_FCloseFile(f);
            return 1;
        }
        return 0;
    }
}

pub extern "C" fn UI_FindClientHeadFile(
    filename: *mut c_char,
    length: c_int,
    teamName: *const c_char,
    headModelName: *const c_char,
    headSkinName: *const c_char,
    base: *const c_char,
    ext: *const c_char,
) -> qboolean {
    unsafe {
        let mut team: *const c_char = "default\0".as_ptr() as *const c_char;
        let mut headsFolder: *const c_char;
        let mut i: c_int;

        if (*headModelName) as u8 == b'*' {
            headsFolder = "heads/\0".as_ptr() as *const c_char;
            headModelName = headModelName.add(1);
        } else {
            headsFolder = "\0".as_ptr() as *const c_char;
        }

        loop {
            i = 0;
            while i < 2 {
                if i == 0 && !teamName.is_null() && *teamName != 0 {
                    Com_sprintf(
                        filename,
                        length,
                        "models/players/%s%s/%s/%s%s_%s.%s\0".as_ptr() as *const c_char,
                        headsFolder,
                        headModelName,
                        headSkinName,
                        teamName,
                        base,
                        team,
                        ext,
                    );
                } else {
                    Com_sprintf(
                        filename,
                        length,
                        "models/players/%s%s/%s/%s_%s.%s\0".as_ptr() as *const c_char,
                        headsFolder,
                        headModelName,
                        headSkinName,
                        base,
                        team,
                        ext,
                    );
                }
                if UI_FileExists(filename) != 0 {
                    return 1;
                }
                if i == 0 && !teamName.is_null() && *teamName != 0 {
                    Com_sprintf(
                        filename,
                        length,
                        "models/players/%s%s/%s%s_%s.%s\0".as_ptr() as *const c_char,
                        headsFolder,
                        headModelName,
                        teamName,
                        base,
                        headSkinName,
                        ext,
                    );
                } else {
                    Com_sprintf(
                        filename,
                        length,
                        "models/players/%s%s/%s_%s.%s\0".as_ptr() as *const c_char,
                        headsFolder,
                        headModelName,
                        base,
                        headSkinName,
                        ext,
                    );
                }
                if UI_FileExists(filename) != 0 {
                    return 1;
                }
                if teamName.is_null() || *teamName == 0 {
                    break;
                }
                i += 1;
            }
            // if tried the heads folder first
            if *headsFolder as u8 != 0 {
                break;
            }
            headsFolder = "heads/\0".as_ptr() as *const c_char;
        }

        return 0;
    }
}

pub extern "C" fn UI_RegisterClientSkin(
    pi: *mut playerInfo_t,
    modelName: *const c_char,
    skinName: *const c_char,
    headModelName: *const c_char,
    headSkinName: *const c_char,
    teamName: *const c_char,
) -> qboolean {
    unsafe {
        let mut filename: [c_char; MAX_QPATH * 2] = [0; MAX_QPATH * 2];

        if !teamName.is_null() && *teamName != 0 {
            Com_sprintf(
                filename.as_mut_ptr(),
                (MAX_QPATH * 2) as c_int,
                "models/players/%s/%s/lower_%s.skin\0".as_ptr() as *const c_char,
                modelName,
                teamName,
                skinName,
            );
        } else {
            Com_sprintf(
                filename.as_mut_ptr(),
                (MAX_QPATH * 2) as c_int,
                "models/players/%s/lower_%s.skin\0".as_ptr() as *const c_char,
                modelName,
                skinName,
            );
        }
        (*pi).legsSkin = trap_R_RegisterSkin(filename.as_ptr());
        if (*pi).legsSkin == 0 {
            if !teamName.is_null() && *teamName != 0 {
                Com_sprintf(
                    filename.as_mut_ptr(),
                    (MAX_QPATH * 2) as c_int,
                    "models/players/characters/%s/%s/lower_%s.skin\0".as_ptr() as *const c_char,
                    modelName,
                    teamName,
                    skinName,
                );
            } else {
                Com_sprintf(
                    filename.as_mut_ptr(),
                    (MAX_QPATH * 2) as c_int,
                    "models/players/characters/%s/lower_%s.skin\0".as_ptr() as *const c_char,
                    modelName,
                    skinName,
                );
            }
            (*pi).legsSkin = trap_R_RegisterSkin(filename.as_ptr());
        }

        if !teamName.is_null() && *teamName != 0 {
            Com_sprintf(
                filename.as_mut_ptr(),
                (MAX_QPATH * 2) as c_int,
                "models/players/%s/%s/upper_%s.skin\0".as_ptr() as *const c_char,
                modelName,
                teamName,
                skinName,
            );
        } else {
            Com_sprintf(
                filename.as_mut_ptr(),
                (MAX_QPATH * 2) as c_int,
                "models/players/%s/upper_%s.skin\0".as_ptr() as *const c_char,
                modelName,
                skinName,
            );
        }
        (*pi).torsoSkin = trap_R_RegisterSkin(filename.as_ptr());
        if (*pi).torsoSkin == 0 {
            if !teamName.is_null() && *teamName != 0 {
                Com_sprintf(
                    filename.as_mut_ptr(),
                    (MAX_QPATH * 2) as c_int,
                    "models/players/characters/%s/%s/upper_%s.skin\0".as_ptr() as *const c_char,
                    modelName,
                    teamName,
                    skinName,
                );
            } else {
                Com_sprintf(
                    filename.as_mut_ptr(),
                    (MAX_QPATH * 2) as c_int,
                    "models/players/characters/%s/upper_%s.skin\0".as_ptr() as *const c_char,
                    modelName,
                    skinName,
                );
            }
            (*pi).torsoSkin = trap_R_RegisterSkin(filename.as_ptr());
        }

        if UI_FindClientHeadFile(
            filename.as_mut_ptr(),
            (MAX_QPATH * 2) as c_int,
            teamName,
            headModelName,
            headSkinName,
            "head\0".as_ptr() as *const c_char,
            "skin\0".as_ptr() as *const c_char,
        ) != 0
        {
            (*pi).headSkin = trap_R_RegisterSkin(filename.as_ptr());
        }

        if (*pi).legsSkin == 0 || (*pi).torsoSkin == 0 || (*pi).headSkin == 0 {
            return 0;
        }

        return 1;
    }
}

pub extern "C" fn UI_ParseAnimationFile(
    filename: *const c_char,
    animations: *mut animation_t,
) -> qboolean {
    unsafe {
        let mut text_p: *mut c_char;
        let mut prev: *mut c_char;
        let mut len: c_int;
        let mut i: c_int;
        let mut token: *const c_char;
        let mut fps: f32;
        let mut skip: c_int = 0;
        let mut text: [c_char; 20000] = [0; 20000];
        let mut f: fileHandle_t = 0;

        memset(
            animations as *mut c_void,
            0,
            (core::mem::size_of::<animation_t>() * MAX_TOTALANIMATIONS),
        );

        // load the file
        len = trap_FS_FOpenFile(filename, core::ptr::addr_of_mut!(f), FS_READ);
        if len <= 0 {
            return 0;
        }
        if len >= ((20000 - 1) as c_int) {
            Com_Printf("File %s too long\n\0".as_ptr() as *const c_char, filename);
            return 0;
        }
        trap_FS_Read(text.as_mut_ptr() as *mut c_void, len, f);
        text[len as usize] = 0;
        trap_FS_FCloseFile(f);

        COM_Compress(text.as_mut_ptr());

        // parse the text
        text_p = text.as_mut_ptr();
        skip = 0;

        // read optional parameters
        loop {
            prev = text_p;
            token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
            if token.is_null() || *token == 0 {
                break;
            }
            if Q_stricmp(token, "footsteps\0".as_ptr() as *const c_char) == 0 {
                token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
                if token.is_null() || *token == 0 {
                    break;
                }
                continue;
            } else if Q_stricmp(token, "headoffset\0".as_ptr() as *const c_char) == 0 {
                i = 0;
                while i < 3 {
                    token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
                    if token.is_null() || *token == 0 {
                        break;
                    }
                    i += 1;
                }
                continue;
            } else if Q_stricmp(token, "sex\0".as_ptr() as *const c_char) == 0 {
                token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
                if token.is_null() || *token == 0 {
                    break;
                }
                continue;
            }

            // if it is a number, start parsing animations
            if *token as u8 >= b'0' && *token as u8 <= b'9' {
                text_p = prev;
                break;
            }

            Com_Printf(
                "unknown token '%s' is %s\n\0".as_ptr() as *const c_char,
                token,
                filename,
            );
        }

        // read information for each frame
        i = 0;
        while i < MAX_TOTALANIMATIONS as c_int {
            token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
            if token.is_null() || *token == 0 {
                break;
            }
            (*animations.add(i as usize)).firstFrame = atoi(token) as u16;
            // leg only frames are adjusted to not count the upper body only frames
            if i == BOTH_CROUCH1WALK {
                skip = (*animations.add(BOTH_CROUCH1WALK as usize)).firstFrame as c_int
                    - (*animations.add(BOTH_GESTURE1 as usize)).firstFrame as c_int;
            }
            if i >= BOTH_CROUCH1WALK {
                (*animations.add(i as usize)).firstFrame =
                    ((*animations.add(i as usize)).firstFrame as c_int - skip) as u16;
            }

            token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
            if token.is_null() || *token == 0 {
                break;
            }
            (*animations.add(i as usize)).numFrames = atoi(token) as u16;

            token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
            if token.is_null() || *token == 0 {
                break;
            }
            (*animations.add(i as usize)).loopFrames = atoi(token) as i8;

            token = COM_Parse(core::ptr::addr_of_mut!(text_p) as *mut *const c_char);
            if token.is_null() || *token == 0 {
                break;
            }
            fps = atof(token);
            if fps == 0.0 {
                fps = 1.0;
            }
            (*animations.add(i as usize)).frameLerp = (1000 / fps) as i16;
            (*animations.add(i as usize)).initialLerp = (1000 / fps) as i16;

            i += 1;
        }

        if i != MAX_TOTALANIMATIONS as c_int {
            Com_Printf(
                "Error parsing animation file: %s\0".as_ptr() as *const c_char,
                filename,
            );
            return 0;
        }

        return 1;
    }
}

pub extern "C" fn UI_RegisterClientModelname(
    pi: *mut playerInfo_t,
    modelSkinName: *const c_char,
    headModelSkinName: *const c_char,
    teamName: *const c_char,
) -> qboolean {
    unsafe {
        let mut modelName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut skinName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut headModelName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut headSkinName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut filename: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut slash: *mut c_char;

        (*pi).torsoModel = 0;
        (*pi).headModel = 0;

        if *modelSkinName == 0 {
            return 0;
        }

        Q_strncpyz(modelName.as_mut_ptr(), modelSkinName, MAX_QPATH as c_int);

        slash = strchr(modelName.as_ptr(), '/' as c_int);
        if slash.is_null() {
            // modelName did not include a skin name
            Q_strncpyz(skinName.as_mut_ptr(), "default\0".as_ptr() as *const c_char, MAX_QPATH as c_int);
        } else {
            Q_strncpyz(skinName.as_mut_ptr(), slash.add(1), MAX_QPATH as c_int);
            *slash = 0;
        }

        Q_strncpyz(headModelName.as_mut_ptr(), headModelSkinName, MAX_QPATH as c_int);
        slash = strchr(headModelName.as_ptr(), '/' as c_int);
        if slash.is_null() {
            // modelName did not include a skin name
            Q_strncpyz(headSkinName.as_mut_ptr(), "default\0".as_ptr() as *const c_char, MAX_QPATH as c_int);
        } else {
            Q_strncpyz(headSkinName.as_mut_ptr(), slash.add(1), MAX_QPATH as c_int);
            *slash = 0;
        }

        // load cmodels before models so filecache works

        Com_sprintf(
            filename.as_mut_ptr(),
            MAX_QPATH as c_int,
            "models/players/%s/lower.md3\0".as_ptr() as *const c_char,
            modelName.as_ptr(),
        );
        (*pi).legsModel = trap_R_RegisterModel(filename.as_ptr());
        if (*pi).legsModel == 0 {
            Com_sprintf(
                filename.as_mut_ptr(),
                MAX_QPATH as c_int,
                "models/players/characters/%s/lower.md3\0".as_ptr() as *const c_char,
                modelName.as_ptr(),
            );
            (*pi).legsModel = trap_R_RegisterModel(filename.as_ptr());
            if (*pi).legsModel == 0 {
                Com_Printf(
                    "Failed to load model file %s\n\0".as_ptr() as *const c_char,
                    filename.as_ptr(),
                );
                return 0;
            }
        }

        Com_sprintf(
            filename.as_mut_ptr(),
            MAX_QPATH as c_int,
            "models/players/%s/upper.md3\0".as_ptr() as *const c_char,
            modelName.as_ptr(),
        );
        (*pi).torsoModel = trap_R_RegisterModel(filename.as_ptr());
        if (*pi).torsoModel == 0 {
            Com_sprintf(
                filename.as_mut_ptr(),
                MAX_QPATH as c_int,
                "models/players/characters/%s/upper.md3\0".as_ptr() as *const c_char,
                modelName.as_ptr(),
            );
            (*pi).torsoModel = trap_R_RegisterModel(filename.as_ptr());
            if (*pi).torsoModel == 0 {
                Com_Printf(
                    "Failed to load model file %s\n\0".as_ptr() as *const c_char,
                    filename.as_ptr(),
                );
                return 0;
            }
        }

        if !headModelName.as_ptr().is_null() && *headModelName.as_ptr() as u8 == b'*' {
            Com_sprintf(
                filename.as_mut_ptr(),
                MAX_QPATH as c_int,
                "models/players/heads/%s/%s.md3\0".as_ptr() as *const c_char,
                headModelName.as_ptr().add(1),
                headModelName.as_ptr().add(1),
            );
        } else {
            Com_sprintf(
                filename.as_mut_ptr(),
                MAX_QPATH as c_int,
                "models/players/%s/head.md3\0".as_ptr() as *const c_char,
                headModelName.as_ptr(),
            );
        }
        (*pi).headModel = trap_R_RegisterModel(filename.as_ptr());
        if (*pi).headModel == 0 && *headModelName.as_ptr() as u8 != b'*' {
            Com_sprintf(
                filename.as_mut_ptr(),
                MAX_QPATH as c_int,
                "models/players/heads/%s/%s.md3\0".as_ptr() as *const c_char,
                headModelName.as_ptr(),
                headModelName.as_ptr(),
            );
            (*pi).headModel = trap_R_RegisterModel(filename.as_ptr());
        }

        if (*pi).headModel == 0 {
            Com_Printf(
                "Failed to load model file %s\n\0".as_ptr() as *const c_char,
                filename.as_ptr(),
            );
            return 0;
        }

        // if any skins failed to load, fall back to default
        if UI_RegisterClientSkin(
            pi,
            modelName.as_ptr(),
            skinName.as_ptr(),
            headModelName.as_ptr(),
            headSkinName.as_ptr(),
            teamName,
        ) == 0
        {
            if UI_RegisterClientSkin(
                pi,
                modelName.as_ptr(),
                "default\0".as_ptr() as *const c_char,
                headModelName.as_ptr(),
                "default\0".as_ptr() as *const c_char,
                teamName,
            ) == 0
            {
                Com_Printf(
                    "Failed to load skin file: %s : %s\n\0".as_ptr() as *const c_char,
                    modelName.as_ptr(),
                    skinName.as_ptr(),
                );
                return 0;
            }
        }

        // load the animations
        Com_sprintf(
            filename.as_mut_ptr(),
            MAX_QPATH as c_int,
            "models/players/%s/animation.cfg\0".as_ptr() as *const c_char,
            modelName.as_ptr(),
        );
        if UI_ParseAnimationFile(filename.as_ptr(), (*pi).animations.as_mut_ptr()) == 0 {
            Com_sprintf(
                filename.as_mut_ptr(),
                MAX_QPATH as c_int,
                "models/players/characters/%s/animation.cfg\0".as_ptr() as *const c_char,
                modelName.as_ptr(),
            );
            if UI_ParseAnimationFile(filename.as_ptr(), (*pi).animations.as_mut_ptr()) == 0 {
                Com_Printf(
                    "Failed to load animation file %s\n\0".as_ptr() as *const c_char,
                    filename.as_ptr(),
                );
                return 0;
            }
        }

        return 1;
    }
}

pub extern "C" fn UI_PlayerInfo_SetModel(
    pi: *mut playerInfo_t,
    model: *const c_char,
    headmodel: *const c_char,
    teamName: *mut c_char,
) {
    unsafe {
        memset(
            pi as *mut c_void,
            0,
            core::mem::size_of::<playerInfo_t>(),
        );
        UI_RegisterClientModelname(pi, model, headmodel, teamName);
        (*pi).weapon = WP_BRYAR_PISTOL;
        (*pi).currentWeapon = (*pi).weapon;
        (*pi).lastWeapon = (*pi).weapon;
        (*pi).pendingWeapon = -1;
        (*pi).weaponTimer = 0;
        (*pi).chat = 0;
        (*pi).newModel = 1;
        UI_PlayerInfo_SetWeapon(pi, (*pi).weapon);
    }
}

pub extern "C" fn UI_PlayerInfo_SetInfo(
    pi: *mut playerInfo_t,
    legsAnim: c_int,
    torsoAnim: c_int,
    viewAngles: *const vec3_t,
    moveAngles: *const vec3_t,
    weaponNumber: c_int,
    chat: qboolean,
) {
    unsafe {
        let mut currentAnim: c_int;
        let mut weaponNum: c_int;

        (*pi).chat = chat;

        // view angles
        _VectorCopy(viewAngles, core::ptr::addr_of_mut!((*pi).viewAngles));

        // move angles
        _VectorCopy(moveAngles, core::ptr::addr_of_mut!((*pi).moveAngles));

        if (*pi).newModel != 0 {
            (*pi).newModel = 0;

            jumpHeight = 0.0;
            (*pi).pendingLegsAnim = 0;
            UI_ForceLegsAnim(pi, legsAnim);
            (*pi).legs.yawAngle = (*viewAngles)[YAW];
            (*pi).legs.yawing = 0;

            (*pi).pendingTorsoAnim = 0;
            UI_ForceTorsoAnim(pi, torsoAnim);
            (*pi).torso.yawAngle = (*viewAngles)[YAW];
            (*pi).torso.yawing = 0;

            if weaponNumber != -1 {
                (*pi).weapon = weaponNumber;
                (*pi).currentWeapon = weaponNumber;
                (*pi).lastWeapon = weaponNumber;
                (*pi).pendingWeapon = -1;
                (*pi).weaponTimer = 0;
                UI_PlayerInfo_SetWeapon(pi, (*pi).weapon);
            }

            return;
        }

        // weapon
        if weaponNumber == -1 {
            (*pi).pendingWeapon = -1;
            (*pi).weaponTimer = 0;
        } else if weaponNumber != WP_NONE {
            (*pi).pendingWeapon = weaponNumber;
            (*pi).weaponTimer = dp_realtime + UI_TIMER_WEAPON_DELAY;
        }
        weaponNum = (*pi).lastWeapon;
        (*pi).weapon = weaponNum;

        if torsoAnim == BOTH_DEATH1 || legsAnim == BOTH_DEATH1 {
            let torsoAnim = BOTH_DEATH1;
            let legsAnim = BOTH_DEATH1;
            (*pi).weapon = WP_NONE;
            (*pi).currentWeapon = WP_NONE;
            UI_PlayerInfo_SetWeapon(pi, (*pi).weapon);

            jumpHeight = 0.0;
            (*pi).pendingLegsAnim = 0;
            UI_ForceLegsAnim(pi, legsAnim);

            (*pi).pendingTorsoAnim = 0;
            UI_ForceTorsoAnim(pi, torsoAnim);

            return;
        }

        // leg animation
        currentAnim = (*pi).legsAnim & !ANIM_TOGGLEBIT;
        if legsAnim != BOTH_JUMP1
            && (currentAnim == BOTH_JUMP1 || currentAnim == BOTH_LAND1)
        {
            (*pi).pendingLegsAnim = legsAnim;
        } else if legsAnim != currentAnim {
            jumpHeight = 0.0;
            (*pi).pendingLegsAnim = 0;
            UI_ForceLegsAnim(pi, legsAnim);
        }

        // torso animation
        let mut torsoAnim = torsoAnim;
        if torsoAnim == TORSO_WEAPONREADY3 || torsoAnim == BOTH_STAND2 {
            if weaponNum == WP_NONE || weaponNum == WP_SABER {
                torsoAnim = BOTH_STAND2;
            } else {
                torsoAnim = TORSO_WEAPONREADY3;
            }
        }

        if torsoAnim == BOTH_ATTACK3 || torsoAnim == BOTH_A1_T__B_ {
            if weaponNum == WP_NONE || weaponNum == WP_SABER {
                torsoAnim = BOTH_A1_T__B_;
            } else {
                torsoAnim = BOTH_ATTACK3;
            }
            (*pi).muzzleFlashTime = dp_realtime + UI_TIMER_MUZZLE_FLASH;
            // FIXME play firing sound here
        }

        currentAnim = (*pi).torsoAnim & !ANIM_TOGGLEBIT;

        if weaponNum != (*pi).currentWeapon
            || currentAnim == TORSO_RAISEWEAP1
            || currentAnim == TORSO_DROPWEAP1
        {
            (*pi).pendingTorsoAnim = torsoAnim;
        } else if (currentAnim == BOTH_GESTURE1 || currentAnim == BOTH_ATTACK3)
            && (torsoAnim != currentAnim)
        {
            (*pi).pendingTorsoAnim = torsoAnim;
        } else if torsoAnim != currentAnim {
            (*pi).pendingTorsoAnim = 0;
            UI_ForceTorsoAnim(pi, torsoAnim);
        }
    }
}

// Stub types and globals for unresolved dependencies
pub struct gitem_t {
    pub classname: *const c_char,
    pub giType: c_int,
    pub giTag: c_int,
    pub world_model: [*const c_char; 2],
}

const IT_WEAPON: c_int = 1;
const BOTH_CROUCH1WALK: c_int = 125;
const FS_READ: c_int = 0;
const DEG2RAD: f32 = std::f32::consts::PI / 180.0;

static bg_itemlist: [gitem_t; 1] = [gitem_t {
    classname: core::ptr::null(),
    giType: 0,
    giTag: 0,
    world_model: [core::ptr::null(); 2],
}];

static mut uiInfo: uiInfo_t = uiInfo_t {
    uiDC: displayContextDef_t { frameTime: 0 },
};

// Stub external functions for unresolved dependencies
extern "C" {
    pub fn COM_StripExtension(in_: *mut c_char, out: *mut c_char);
    pub fn COM_Compress(data: *mut c_char);
    pub fn COM_Parse(data_p: *mut *const c_char) -> *const c_char;
    pub fn rand() -> c_int;
}
