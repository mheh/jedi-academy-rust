// cg_localents.c -- every frame, generate renderer commands for locally
// processed entities, like smoke puffs, gibs, shells, etc.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// Import basic types from codemp's q_shared_h
use crate::codemp::game::q_shared_h::{vec3_t, vec_t, qboolean};

// ============================================================================
// External C type declarations and stubs
// ============================================================================

#[repr(C)]
pub struct localEntity_t {
    pub prev: *mut localEntity_t,
    pub next: *mut localEntity_t,
    pub leType: c_int,
    pub leFlags: c_int,
    pub startTime: c_int,
    pub endTime: c_int,
    pub lifeRate: f32,
    pub pos: trajectory_t,
    pub angles: trajectory_t,
    pub bounceFactor: f32,
    pub color: [f32; 4],
    pub radius: f32,
    pub light: f32,
    pub lightColor: vec3_t,
    pub leBounceSoundType: c_int,
    pub refEntity: refEntity_t,
    pub ownerGentNum: c_int,
}

#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

#[repr(C)]
pub struct refEntity_t {
    pub reType: c_int,
    pub renderfx: c_int,
    pub hModel: c_int,
    pub lightingOrigin: vec3_t,
    pub shadowPlane: f32,
    pub axis: [vec3_t; 3],
    pub nonNormalizedAxes: c_int,
    pub origin: vec3_t,
    pub oldorigin: vec3_t,
    pub frame: c_int,
    pub oldframe: c_int,
    pub backlerp: f32,
    pub surfaceFlags: c_int,
    pub customShader: c_int,
    pub shaderRGBA: [u8; 4],
    pub shaderTexCoord: [f32; 2],
    pub shaderTime: f32,
    pub radius: f32,
    pub rotation: f32,
}

#[repr(C)]
pub struct trace_t {
    pub allsolid: c_int,
    pub startsolid: c_int,
    pub fraction: f32,
    pub endpos: vec3_t,
    pub plane: cplane_t,
    pub surfaceFlags: c_int,
    pub contents: c_int,
    pub entityNum: c_int,
}

#[repr(C)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
    pub pad: [u8; 2],
}

#[repr(C)]
pub struct cg_t {
    pub time: c_int,
    pub frametime: c_int,
    pub refdef: refdef_t,
}

#[repr(C)]
pub struct refdef_t {
    pub vieworg: vec3_t,
    pub viewaxis: [vec3_t; 3],
}

#[repr(C)]
pub struct cgs_t {
    pub media: cgs_media_t,
}

#[repr(C)]
pub struct cgs_media_t {
    pub rockBounceSound: [c_int; 2],
    pub metalBounceSound: [c_int; 2],
}

// Constants
pub const MAX_LOCAL_ENTITIES_XBOX: c_int = 128;
pub const MAX_LOCAL_ENTITIES: c_int = 512;

pub const FRAG_FADE_TIME: c_int = 750;

// Bounce sound types
pub const LEBS_NONE: c_int = 0;
pub const LEBS_METAL: c_int = 1;
pub const LEBS_ROCK: c_int = 2;

// Local entity types
pub const LE_MARK: c_int = 0;
pub const LE_FADE_MODEL: c_int = 1;
pub const LE_FADE_SCALE_MODEL: c_int = 2;
pub const LE_FRAGMENT: c_int = 3;
pub const LE_PUFF: c_int = 4;
pub const LE_FADE_RGB: c_int = 5;
pub const LE_LIGHT: c_int = 6;
pub const LE_LINE: c_int = 7;
pub const LE_QUAD: c_int = 8;
pub const LE_SPRITE: c_int = 9;

// Local entity flags
pub const LEF_PUFF_DONT_SCALE: c_int = 0x0001;
pub const LEF_TUMBLE: c_int = 0x0002;

// Trajectory types
pub const TR_STATIONARY: c_int = 0;
pub const TR_GRAVITY: c_int = 2;

// Render types and flags
pub const RT_MODEL: c_int = 0;
pub const RT_LINE: c_int = 1;
pub const RF_ALPHA_FADE: c_int = 0x0001;
pub const RF_LIGHTING_ORIGIN: c_int = 0x0002;

// Sound channels
pub const CHAN_AUTO: c_int = 0;

// Entity number
pub const ENTITYNUM_WORLD: c_int = 1023;

// Contents flags
pub const CONTENTS_SOLID: c_int = 1;
pub const CONTENTS_NODROP: c_int = 0x00000080;

extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;

    pub static mut axisDefault: [vec3_t; 3];

    // Vector operations
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorScale(src: *const vec3_t, scale: f32, dst: *mut vec3_t);
    pub fn VectorClear(v: *mut vec3_t);
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorMA(base: *const vec3_t, scale: f32, dir: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorLength(v: *const vec3_t) -> f32;
    pub fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn AxisCopy(src: *const [vec3_t; 3], dst: *mut [vec3_t; 3]);
    pub fn AnglesToAxis(angles: *const vec3_t, axis: *mut [vec3_t; 3]);

    // Trajectory operations
    pub fn EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: *mut vec3_t);
    pub fn EvaluateTrajectoryDelta(tr: *const trajectory_t, atTime: c_int, result: *mut vec3_t);

    // Collision
    pub fn CG_Trace(
        result: *mut trace_t,
        start: *const vec3_t,
        mins: *const vec3_t,
        maxs: *const vec3_t,
        end: *const vec3_t,
        passent: c_int,
        contentmask: c_int,
    );
    pub fn cgi_CM_PointContents(p: *const vec3_t, boxnum: c_int) -> c_int;

    // Rendering
    pub fn cgi_R_AddRefEntityToScene(re: *const refEntity_t);
    pub fn cgi_R_AddLightToScene(org: *const vec3_t, radius: f32, r: f32, g: f32, b: f32);
    pub fn cgi_R_AddPolyToScene(hShader: c_int, numVerts: c_int, verts: *const polyVert_t);

    // Sound
    pub fn cgi_S_StartSound(origin: *const vec3_t, entityNum: c_int, entchannel: c_int, sfxHandle: c_int);

    // Random
    pub fn rand() -> c_int;
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;

    // Error handling
    pub fn CG_Error(msg: *const c_char, ...);
}

// Stub types for external functions
#[repr(C)]
pub struct polyVert_t {
    pub xyz: vec3_t,
    pub st: [f32; 2],
    pub modulate: [u8; 4],
}

// Helper to create a zeroed localEntity_t for static initialization
const fn zeroed_local_entity() -> localEntity_t {
    localEntity_t {
        prev: core::ptr::null_mut(),
        next: core::ptr::null_mut(),
        leType: 0,
        leFlags: 0,
        startTime: 0,
        endTime: 0,
        lifeRate: 0.0,
        pos: trajectory_t {
            trType: 0,
            trTime: 0,
            trDuration: 0,
            trBase: [0.0; 3],
            trDelta: [0.0; 3],
        },
        angles: trajectory_t {
            trType: 0,
            trTime: 0,
            trDuration: 0,
            trBase: [0.0; 3],
            trDelta: [0.0; 3],
        },
        bounceFactor: 0.0,
        color: [0.0; 4],
        radius: 0.0,
        light: 0.0,
        lightColor: [0.0; 3],
        leBounceSoundType: 0,
        refEntity: refEntity_t {
            reType: 0,
            renderfx: 0,
            hModel: 0,
            lightingOrigin: [0.0; 3],
            shadowPlane: 0.0,
            axis: [[0.0; 3]; 3],
            nonNormalizedAxes: 0,
            origin: [0.0; 3],
            oldorigin: [0.0; 3],
            frame: 0,
            oldframe: 0,
            backlerp: 0.0,
            surfaceFlags: 0,
            customShader: 0,
            shaderRGBA: [0; 4],
            shaderTexCoord: [0.0; 2],
            shaderTime: 0.0,
            radius: 0.0,
            rotation: 0.0,
        },
        ownerGentNum: 0,
    }
}

// Globals
pub static mut cg_localEntities: [localEntity_t; 512] = [zeroed_local_entity(); 512];

pub static mut cg_activeLocalEntities: localEntity_t = zeroed_local_entity();

pub static mut cg_freeLocalEntities: *mut localEntity_t = core::ptr::null_mut();

/*
===================
CG_InitLocalEntities

This is called at startup and for tournement restarts
===================
*/

pub unsafe fn CG_InitLocalEntities() {
    // memset( cg_localEntities, 0, sizeof( cg_localEntities ) );
    for i in 0..(MAX_LOCAL_ENTITIES as usize) {
        core::ptr::write(
            addr_of_mut!(cg_localEntities[i]),
            localEntity_t {
                prev: core::ptr::null_mut(),
                next: core::ptr::null_mut(),
                leType: 0,
                leFlags: 0,
                startTime: 0,
                endTime: 0,
                lifeRate: 0.0,
                pos: trajectory_t {
                    trType: 0,
                    trTime: 0,
                    trDuration: 0,
                    trBase: [0.0; 3],
                    trDelta: [0.0; 3],
                },
                angles: trajectory_t {
                    trType: 0,
                    trTime: 0,
                    trDuration: 0,
                    trBase: [0.0; 3],
                    trDelta: [0.0; 3],
                },
                bounceFactor: 0.0,
                color: [0.0; 4],
                radius: 0.0,
                light: 0.0,
                lightColor: [0.0; 3],
                leBounceSoundType: 0,
                refEntity: refEntity_t {
                    reType: 0,
                    renderfx: 0,
                    hModel: 0,
                    lightingOrigin: [0.0; 3],
                    shadowPlane: 0.0,
                    axis: [[0.0; 3]; 3],
                    nonNormalizedAxes: 0,
                    origin: [0.0; 3],
                    oldorigin: [0.0; 3],
                    frame: 0,
                    oldframe: 0,
                    backlerp: 0.0,
                    surfaceFlags: 0,
                    customShader: 0,
                    shaderRGBA: [0; 4],
                    shaderTexCoord: [0.0; 2],
                    shaderTime: 0.0,
                    radius: 0.0,
                    rotation: 0.0,
                },
                ownerGentNum: 0,
            },
        );
    }

    (*addr_of_mut!(cg_activeLocalEntities)).next = addr_of_mut!(cg_activeLocalEntities);
    (*addr_of_mut!(cg_activeLocalEntities)).prev = addr_of_mut!(cg_activeLocalEntities);
    cg_freeLocalEntities = addr_of_mut!(cg_localEntities[0]);

    for i in 0..((MAX_LOCAL_ENTITIES - 1) as usize) {
        (*addr_of_mut!(cg_localEntities[i])).next = addr_of_mut!(cg_localEntities[i + 1]);
    }
}

/*
==================
CG_FreeLocalEntity
==================
*/

pub unsafe fn CG_FreeLocalEntity(le: *mut localEntity_t) {
    if (*le).prev.is_null() {
        CG_Error(b"CG_FreeLocalEntity: not active\0".as_ptr() as *const c_char);
    }

    // remove from the doubly linked active list
    (*(*le).prev).next = (*le).next;
    (*(*le).next).prev = (*le).prev;

    // the free list is only singly linked
    (*le).next = cg_freeLocalEntities;
    cg_freeLocalEntities = le;
}

/*
===================
CG_AllocLocalEntity

Will allways succeed, even if it requires freeing an old active entity
===================
*/

pub unsafe fn CG_AllocLocalEntity() -> *mut localEntity_t {
    let mut le: *mut localEntity_t;

    if cg_freeLocalEntities.is_null() {
        // no free entities, so free the one at the end of the chain
        // remove the oldest active entity
        CG_FreeLocalEntity((*addr_of_mut!(cg_activeLocalEntities)).prev);
    }

    le = cg_freeLocalEntities;
    cg_freeLocalEntities = (*cg_freeLocalEntities).next;

    // memset( le, 0, sizeof( *le ) );
    core::ptr::write(
        le,
        localEntity_t {
            prev: core::ptr::null_mut(),
            next: core::ptr::null_mut(),
            leType: 0,
            leFlags: 0,
            startTime: 0,
            endTime: 0,
            lifeRate: 0.0,
            pos: trajectory_t {
                trType: 0,
                trTime: 0,
                trDuration: 0,
                trBase: [0.0; 3],
                trDelta: [0.0; 3],
            },
            angles: trajectory_t {
                trType: 0,
                trTime: 0,
                trDuration: 0,
                trBase: [0.0; 3],
                trDelta: [0.0; 3],
            },
            bounceFactor: 0.0,
            color: [0.0; 4],
            radius: 0.0,
            light: 0.0,
            lightColor: [0.0; 3],
            leBounceSoundType: 0,
            refEntity: refEntity_t {
                reType: 0,
                renderfx: 0,
                hModel: 0,
                lightingOrigin: [0.0; 3],
                shadowPlane: 0.0,
                axis: [[0.0; 3]; 3],
                nonNormalizedAxes: 0,
                origin: [0.0; 3],
                oldorigin: [0.0; 3],
                frame: 0,
                oldframe: 0,
                backlerp: 0.0,
                surfaceFlags: 0,
                customShader: 0,
                shaderRGBA: [0; 4],
                shaderTexCoord: [0.0; 2],
                shaderTime: 0.0,
                radius: 0.0,
                rotation: 0.0,
            },
            ownerGentNum: 0,
        },
    );

    // link into the active list
    (*le).next = (*addr_of_mut!(cg_activeLocalEntities)).next;
    (*le).prev = addr_of_mut!(cg_activeLocalEntities);
    (*(*addr_of_mut!(cg_activeLocalEntities)).next).prev = le;
    (*addr_of_mut!(cg_activeLocalEntities)).next = le;
    (*le).ownerGentNum = -1;
    le
}

/*
====================================================================================

FRAGMENT PROCESSING

A fragment localentity interacts with the environment in some way (hitting walls),
or generates more localentities along a trail.

====================================================================================
*/

/*
================
CG_FragmentBounceSound
================
*/

pub unsafe fn CG_FragmentBounceSound(le: *mut localEntity_t, trace: *const trace_t) {
    // half the fragments will make a bounce sounds
    if (rand() & 1) != 0 {
        let mut s: c_int = 0;

        match (*le).leBounceSoundType {
            LEBS_ROCK => {
                s = cgs.media.rockBounceSound[Q_irand(0, 1) as usize];
            }
            LEBS_METAL => {
                s = cgs.media.metalBounceSound[Q_irand(0, 1) as usize]; // FIXME: make sure that this sound is registered properly...might still be rock bounce sound....
            }
            _ => {}
        }

        if s != 0 {
            cgi_S_StartSound(addr_of!((*trace).endpos), ENTITYNUM_WORLD, CHAN_AUTO, s);
        }

        // bouncers only make the sound once...
        // FIXME: arbitrary...change if it bugs you
        (*le).leBounceSoundType = LEBS_NONE;
    } else if (rand() & 1) != 0 {
        // we may end up bouncing again, but each bounce reduces the chance of playing the sound again or they may make a lot of noise when they settle
        // FIXME: maybe just always do this??
        (*le).leBounceSoundType = LEBS_NONE;
    }
}

/*
================
CG_ReflectVelocity
================
*/

pub unsafe fn CG_ReflectVelocity(le: *mut localEntity_t, trace: *const trace_t) {
    let mut velocity: vec3_t = [0.0; 3];
    let mut dot: f32;
    let mut hitTime: c_int;

    // reflect the velocity on the trace plane
    hitTime = cg.time - cg.frametime + ((cg.frametime as f32) * (*trace).fraction) as c_int;
    EvaluateTrajectoryDelta(addr_of!((*le).pos), hitTime, addr_of_mut!(velocity));
    dot = DotProduct(addr_of!(velocity), addr_of!((*trace).plane.normal));
    VectorMA(
        addr_of!(velocity),
        -2.0 * dot,
        addr_of!((*trace).plane.normal),
        addr_of_mut!((*le).pos.trDelta),
    );

    VectorScale(
        addr_of!((*le).pos.trDelta),
        (*le).bounceFactor,
        addr_of_mut!((*le).pos.trDelta),
    );

    VectorCopy(addr_of!((*trace).endpos), addr_of_mut!((*le).pos.trBase));
    (*le).pos.trTime = cg.time;

    // check for stop, making sure that even on low FPS systems it doesn't bobble
    if (*trace).allsolid != 0
        || ((*trace).plane.normal[2] > 0.0
            && ((*le).pos.trDelta[2] < 40.0
                || (*le).pos.trDelta[2] < -((cg.frametime as f32) * (*le).pos.trDelta[2])))
    {
        (*le).pos.trType = TR_STATIONARY;
    }
}

/*
================
CG_AddFragment
================
*/

pub unsafe fn CG_AddFragment(le: *mut localEntity_t) {
    let mut newOrigin: vec3_t = [0.0; 3];
    let mut trace: trace_t = core::mem::zeroed();
    // used to sink into the ground, but it looks better to maybe just fade them out
    let mut t: c_int;

    t = (*le).endTime - cg.time;

    if t < FRAG_FADE_TIME {
        (*le).refEntity.renderfx |= RF_ALPHA_FADE;
        (*le).refEntity.shaderRGBA[0] = 255;
        (*le).refEntity.shaderRGBA[1] = 255;
        (*le).refEntity.shaderRGBA[2] = 255;
        (*le).refEntity.shaderRGBA[3] = (((t as f32) / (FRAG_FADE_TIME as f32)) * 255.0) as u8;
    }

    if (*le).pos.trType == TR_STATIONARY {
        if (cgi_CM_PointContents(addr_of!((*le).refEntity.origin), 0) & CONTENTS_SOLID) == 0 {
            // thing is no longer in solid, so let gravity take it back
            VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!((*le).pos.trBase));
            VectorClear(addr_of_mut!((*le).pos.trDelta));
            (*le).pos.trTime = cg.time;
            (*le).pos.trType = TR_GRAVITY;
        }

        cgi_R_AddRefEntityToScene(addr_of!((*le).refEntity));

        return;
    }

    // calculate new position
    EvaluateTrajectory(addr_of!((*le).pos), cg.time, addr_of_mut!(newOrigin));

    (*le).refEntity.renderfx |= RF_LIGHTING_ORIGIN;
    VectorCopy(addr_of!(newOrigin), addr_of_mut!((*le).refEntity.lightingOrigin));

    // trace a line from previous position to new position
    CG_Trace(
        addr_of_mut!(trace),
        addr_of!((*le).refEntity.origin),
        core::ptr::null(),
        core::ptr::null(),
        addr_of!(newOrigin),
        (*le).ownerGentNum,
        CONTENTS_SOLID,
    );

    if trace.fraction == 1.0 {
        // still in free fall
        VectorCopy(addr_of!(newOrigin), addr_of_mut!((*le).refEntity.origin));

        if ((*le).leFlags & LEF_TUMBLE) != 0 {
            let mut angles: vec3_t = [0.0; 3];

            EvaluateTrajectory(addr_of!((*le).angles), cg.time, addr_of_mut!(angles));
            AnglesToAxis(addr_of!(angles), addr_of_mut!((*le).refEntity.axis));
            for k in 0..3 {
                VectorScale(
                    addr_of!((*le).refEntity.axis[k]),
                    (*le).radius,
                    addr_of_mut!((*le).refEntity.axis[k]),
                );
            }
        }

        cgi_R_AddRefEntityToScene(addr_of!((*le).refEntity));

        return;
    }

    // if it is in a nodrop zone, remove it
    // this keeps gibs from waiting at the bottom of pits of death
    // and floating levels
    if (cgi_CM_PointContents(addr_of!(trace.endpos), 0) & CONTENTS_NODROP) != 0 {
        CG_FreeLocalEntity(le);
        return;
    }

    // do a bouncy sound
    CG_FragmentBounceSound(le, addr_of!(trace));

    // reflect the velocity on the trace plane
    CG_ReflectVelocity(le, addr_of!(trace));
    //FIXME: if LEF_TUMBLE, change avelocity too?

    cgi_R_AddRefEntityToScene(addr_of!((*le).refEntity));
}

/*
=====================================================================

TRIVIAL LOCAL ENTITIES

These only do simple scaling or modulation before passing to the renderer
=====================================================================
*/

/*
** CG_AddTeleporterEffect
*/

pub unsafe fn CG_AddTeleporterEffect(le: *mut localEntity_t) {
    let re: *mut refEntity_t = addr_of_mut!((*le).refEntity);
    let mut c: f32;

    c = (((*le).endTime - cg.time) as f32) / (((*le).endTime - (*le).startTime) as f32);

    (*re).shaderRGBA[0] = (0xff as f32 * c) as u8;
    (*re).shaderRGBA[1] = (0xff as f32 * c) as u8;
    (*re).shaderRGBA[2] = (0xff as f32 * c) as u8;
    (*re).shaderRGBA[3] = (0xff as f32 * c) as u8;

    cgi_R_AddRefEntityToScene(re);
}

/*
** CG_AddFadeRGB
*/

pub unsafe fn CG_AddFadeRGB(le: *mut localEntity_t) {
    let re: *mut refEntity_t = addr_of_mut!((*le).refEntity);
    let mut c: f32;

    c = (((*le).endTime - cg.time) as f32) * (*le).lifeRate;
    c *= 0xff as f32;

    (*re).shaderRGBA[0] = ((*le).color[0] * c) as u8;
    (*re).shaderRGBA[1] = ((*le).color[1] * c) as u8;
    (*re).shaderRGBA[2] = ((*le).color[2] * c) as u8;
    (*re).shaderRGBA[3] = ((*le).color[3] * c) as u8;

    cgi_R_AddRefEntityToScene(re);
}

/*
==================
CG_AddPuff
==================
*/

pub unsafe fn CG_AddPuff(le: *mut localEntity_t) {
    let re: *mut refEntity_t = addr_of_mut!((*le).refEntity);
    let mut c: f32;
    let mut delta: vec3_t = [0.0; 3];
    let mut len: f32;

    // fade / grow time
    c = (((*le).endTime - cg.time) as f32) / (((*le).endTime - (*le).startTime) as f32);

    (*re).shaderRGBA[0] = ((*le).color[0] * c) as u8;
    (*re).shaderRGBA[1] = ((*le).color[1] * c) as u8;
    (*re).shaderRGBA[2] = ((*le).color[2] * c) as u8;

    if ((*le).leFlags & LEF_PUFF_DONT_SCALE) == 0 {
        (*re).radius = (*le).radius * (1.0 - c) + 8.0;
    }

    EvaluateTrajectory(addr_of!((*le).pos), cg.time, addr_of_mut!((*re).origin));

    // if the view would be "inside" the sprite, kill the sprite
    // so it doesn't add too much overdraw
    VectorSubtract(
        addr_of!((*re).origin),
        addr_of!(cg.refdef.vieworg),
        addr_of_mut!(delta),
    );
    len = VectorLength(addr_of!(delta));
    if len < (*le).radius {
        CG_FreeLocalEntity(le);
        return;
    }

    cgi_R_AddRefEntityToScene(re);
}

/*
================
CG_AddLocalLight
================
*/

pub unsafe fn CG_AddLocalLight(le: *mut localEntity_t) {
    // There should be a light if this is being used, but hey...
    if (*le).light != 0.0 {
        let mut light: f32;

        light = ((cg.time - (*le).startTime) as f32) / (((*le).endTime - (*le).startTime) as f32);

        if light < 0.5 {
            light = 1.0;
        } else {
            light = 1.0 - (light - 0.5) * 2.0;
        }

        light = (*le).light * light;

        cgi_R_AddLightToScene(
            addr_of!((*le).refEntity.origin),
            light,
            (*le).lightColor[0],
            (*le).lightColor[1],
            (*le).lightColor[2],
        );
    }
}

//---------------------------------------------------

pub unsafe fn CG_AddFadeModel(le: *mut localEntity_t) {
    let ent: *mut refEntity_t = addr_of_mut!((*le).refEntity);

    if cg.time < (*le).startTime {
        CG_FreeLocalEntity(le);
        return;
    }

    let frac: f32 = 1.0 - (((cg.time - (*le).startTime) as f32) / (((*le).endTime - (*le).startTime) as f32));

    (*ent).shaderRGBA[0] = ((*le).color[0] * frac) as u8;
    (*ent).shaderRGBA[1] = ((*le).color[1] * frac) as u8;
    (*ent).shaderRGBA[2] = ((*le).color[2] * frac) as u8;
    (*ent).shaderRGBA[3] = ((*le).color[3] * frac) as u8;

    EvaluateTrajectory(addr_of!((*le).pos), cg.time, addr_of_mut!((*ent).origin));

    // add the entity
    cgi_R_AddRefEntityToScene(ent);
}

// NOTE: this is 100% for the demp2 alt-fire effect, so changes to the visual effect will affect game side demp2 code
//---------------------------------------------------

pub unsafe fn CG_AddFadeScaleModel(le: *mut localEntity_t) {
    let ent: *mut refEntity_t = addr_of_mut!((*le).refEntity);

    let mut frac: f32 = ((cg.time - (*le).startTime) as f32) / (((*le).endTime - (*le).startTime) as f32);

    frac = frac * frac * frac; // yes, this is completely ridiculous...but it causes the shell to grow slowly then "explode" at the end

    (*ent).nonNormalizedAxes = 1; // qtrue

    AxisCopy(addr_of!(axisDefault), addr_of_mut!((*ent).axis));

    VectorScale(
        addr_of!((*ent).axis[0]),
        (*le).radius * frac,
        addr_of_mut!((*ent).axis[0]),
    );
    VectorScale(
        addr_of!((*ent).axis[1]),
        (*le).radius * frac,
        addr_of_mut!((*ent).axis[1]),
    );
    VectorScale(
        addr_of!((*ent).axis[2]),
        (*le).radius * 0.5 * frac,
        addr_of_mut!((*ent).axis[2]),
    );

    frac = 1.0 - frac;

    (*ent).shaderRGBA[0] = ((*le).color[0] * frac) as u8;
    (*ent).shaderRGBA[1] = ((*le).color[1] * frac) as u8;
    (*ent).shaderRGBA[2] = ((*le).color[2] * frac) as u8;
    (*ent).shaderRGBA[3] = ((*le).color[3] * frac) as u8;

    // add the entity
    cgi_R_AddRefEntityToScene(ent);
}

// create a quad that doesn't use a refEnt.  Currently only for use with the DebugNav drawing so it doesn't have to use fx
//------------------------------------------

pub unsafe fn CG_AddQuad(le: *mut localEntity_t) {
    let mut verts: [polyVert_t; 4] = core::mem::zeroed();

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[0].xyz));
    verts[0].xyz[0] -= (*le).radius;
    verts[0].xyz[1] -= (*le).radius;
    verts[0].st[0] = 0.0;
    verts[0].st[1] = 0.0;

    for i in 0..4 {
        verts[i].modulate[0] = ((*le).color[0] * 255.0) as u8;
        verts[i].modulate[1] = ((*le).color[1] * 255.0) as u8;
        verts[i].modulate[2] = ((*le).color[2] * 255.0) as u8;
        verts[i].modulate[3] = ((*le).color[3] * 255.0) as u8;
    }

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[1].xyz));
    verts[1].xyz[0] -= (*le).radius;
    verts[1].xyz[1] += (*le).radius;
    verts[1].st[0] = 0.0;
    verts[1].st[1] = 1.0;

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[2].xyz));
    verts[2].xyz[0] += (*le).radius;
    verts[2].xyz[1] += (*le).radius;
    verts[2].st[0] = 1.0;
    verts[2].st[1] = 1.0;

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[3].xyz));
    verts[3].xyz[0] += (*le).radius;
    verts[3].xyz[1] -= (*le).radius;
    verts[3].st[0] = 1.0;
    verts[3].st[1] = 0.0;

    cgi_R_AddPolyToScene((*le).refEntity.customShader, 4, addr_of!(verts[0]));
}

// create a sprite that doesn't use a refEnt.  Currently only for use with the DebugNav drawing so it doesn't have to use fx
//------------------------------------------

pub unsafe fn CG_AddSprite(le: *mut localEntity_t) {
    let mut verts: [polyVert_t; 4] = core::mem::zeroed();

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[0].xyz));
    VectorMA(
        addr_of_mut!(verts[0].xyz),
        -(*le).radius,
        addr_of!(cg.refdef.viewaxis[2]),
        addr_of_mut!(verts[0].xyz),
    );
    VectorMA(
        addr_of_mut!(verts[0].xyz),
        -(*le).radius,
        addr_of!(cg.refdef.viewaxis[1]),
        addr_of_mut!(verts[0].xyz),
    );
    verts[0].st[0] = 0.0;
    verts[0].st[1] = 0.0;

    for i in 0..4 {
        verts[i].modulate[0] = ((*le).color[0] * 255.0) as u8;
        verts[i].modulate[1] = ((*le).color[1] * 255.0) as u8;
        verts[i].modulate[2] = ((*le).color[2] * 255.0) as u8;
        verts[i].modulate[3] = ((*le).color[3] * 255.0) as u8;
    }

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[1].xyz));
    VectorMA(
        addr_of_mut!(verts[1].xyz),
        -(*le).radius,
        addr_of!(cg.refdef.viewaxis[2]),
        addr_of_mut!(verts[1].xyz),
    );
    VectorMA(
        addr_of_mut!(verts[1].xyz),
        (*le).radius,
        addr_of!(cg.refdef.viewaxis[1]),
        addr_of_mut!(verts[1].xyz),
    );
    verts[1].st[0] = 0.0;
    verts[1].st[1] = 1.0;

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[2].xyz));
    VectorMA(
        addr_of_mut!(verts[2].xyz),
        (*le).radius,
        addr_of!(cg.refdef.viewaxis[2]),
        addr_of_mut!(verts[2].xyz),
    );
    VectorMA(
        addr_of_mut!(verts[2].xyz),
        (*le).radius,
        addr_of!(cg.refdef.viewaxis[1]),
        addr_of_mut!(verts[2].xyz),
    );
    verts[2].st[0] = 1.0;
    verts[2].st[1] = 1.0;

    VectorCopy(addr_of!((*le).refEntity.origin), addr_of_mut!(verts[3].xyz));
    VectorMA(
        addr_of_mut!(verts[3].xyz),
        (*le).radius,
        addr_of!(cg.refdef.viewaxis[2]),
        addr_of_mut!(verts[3].xyz),
    );
    VectorMA(
        addr_of_mut!(verts[3].xyz),
        -(*le).radius,
        addr_of!(cg.refdef.viewaxis[1]),
        addr_of_mut!(verts[3].xyz),
    );
    verts[3].st[0] = 1.0;
    verts[3].st[1] = 0.0;

    cgi_R_AddPolyToScene((*le).refEntity.customShader, 4, addr_of!(verts[0]));
}

/*
===================
CG_AddLine

for beams and the like.
===================
*/

pub unsafe fn CG_AddLine(le: *mut localEntity_t) {
    let re: *mut refEntity_t = addr_of_mut!((*le).refEntity);

    (*re).reType = RT_LINE;

    cgi_R_AddRefEntityToScene(re);
}

//==============================================================================

/*
===================
CG_AddLocalEntities

===================
*/

pub unsafe fn CG_AddLocalEntities() {
    let mut le: *mut localEntity_t;
    let mut next: *mut localEntity_t;

    // walk the list backwards, so any new local entities generated
    // (trails, marks, etc) will be present this frame
    le = (*addr_of_mut!(cg_activeLocalEntities)).prev;

    loop {
        if le == addr_of_mut!(cg_activeLocalEntities) {
            break;
        }

        // grab next now, so if the local entity is freed we
        // still have it
        next = (*le).prev;

        if cg.time >= (*le).endTime {
            CG_FreeLocalEntity(le);
            le = next;
            continue;
        }

        match (*le).leType {
            LE_MARK => {}

            LE_FADE_MODEL => {
                CG_AddFadeModel(le);
            }

            LE_FADE_SCALE_MODEL => {
                CG_AddFadeScaleModel(le);
            }

            LE_FRAGMENT => {
                CG_AddFragment(le);
            }

            LE_PUFF => {
                CG_AddPuff(le);
            }

            LE_FADE_RGB => {
                // teleporters, railtrails
                CG_AddFadeRGB(le);
            }

            LE_LIGHT => {
                CG_AddLocalLight(le);
            }

            LE_LINE => {
                // oriented lines for FX
                CG_AddLine(le);
            }

            // Use for debug only
            LE_QUAD => {
                CG_AddQuad(le);
            }

            LE_SPRITE => {
                CG_AddSprite(le);
            }

            _ => {
                CG_Error(b"Bad leType: %i\0".as_ptr() as *const c_char, (*le).leType);
            }
        }

        le = next;
    }
}
