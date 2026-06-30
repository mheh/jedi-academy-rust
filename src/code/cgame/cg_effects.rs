// cg_effects.c -- these functions generate localentities

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

use crate::code::cgame::cg_headers::*;
use crate::code::cgame::cg_media_h::*;
use crate::code::cgame::FxScheduler_h::*;
use crate::code::game::q_shared::*;

extern "C" {
    pub static mut cg: cg_t;
    pub static mut cgs: cgs_t;

    // Vector operations
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorScale(src: *const vec3_t, scale: f32, dst: *mut vec3_t);
    pub fn VectorSubtract(a: *const vec3_t, b: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorAdd(a: *const vec3_t, b: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorMA(base: *const vec3_t, scale: f32, dir: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn Vector2Set(v: *mut vec2_t, x: f32, y: f32);
    pub fn VectorSet(v: *mut vec3_t, x: f32, y: f32, z: f32);
    pub fn DistanceSquared(a: *const vec3_t, b: *const vec3_t) -> f32;
    pub fn CrossProduct(a: *const vec3_t, b: *const vec3_t, dst: *mut vec3_t);
    pub fn AxisCopy(src: *const [vec3_t; 3], dst: *mut [vec3_t; 3]);
    pub fn AxisClear(axis: *mut [vec3_t; 3]);
    pub fn RotateAroundDirection(axis: *mut [vec3_t; 3], angle: f32);
    pub fn AnglesToAxis(angles: *const vec3_t, axis: *mut [vec3_t; 3]);
    pub fn MakeNormalVectors(forward: *const vec3_t, right: *mut vec3_t, up: *mut vec3_t);

    // Local entity management
    pub fn CG_AllocLocalEntity() -> *mut localEntity_t;

    // Sound
    pub fn cgi_S_StartSound(origin: *const vec3_t, entity_num: c_int, channel: c_int, sound_handle: c_int);
    pub fn cgi_S_RegisterSound(name: *const c_char) -> c_int;

    // Rendering
    pub fn cgi_R_RegisterShader(name: *const c_char) -> c_int;
    pub fn cgi_R_RegisterModel(name: *const c_char) -> c_int;
    pub fn cgi_R_AddRefEntityToScene(re: *const refEntity_t);

    // Camera effects
    pub fn CGCam_Shake(intensity: f32, time: c_int);

    // Error handling
    pub fn CG_Error(msg: *const c_char, ...);

    // Configuration strings
    pub fn CG_ConfigString(index: c_int) -> *const c_char;

    // Random functions
    pub fn rand() -> c_int;
    pub fn random() -> f32;
    pub fn crandom() -> f32;
    pub fn Q_irand(low: c_int, high: c_int) -> c_int;
    pub fn Q_flrand(low: f32, high: f32) -> f32;

    // FX system
    pub fn FX_AddPoly(
        verts: *const [vec3_t; 4],
        st: *const [vec2_t; 4],
        vert_count: c_int,
        vel: *const vec3_t,
        accel: *const vec3_t,
        alpha_start: f32,
        alpha_end: f32,
        alpha_parm: f32,
        rgb_start: *const vec3_t,
        rgb_end: *const vec3_t,
        rgb_parm: f32,
        rot_delta: *const vec3_t,
        bounce: f32,
        time: c_int,
        life: c_int,
        shader: c_int,
        flags: c_int,
    ) -> *mut CPoly;

    // FxScheduler - opaque C++ object with extern C++ methods
    pub static mut theFxScheduler: FxScheduler;

    // FxScheduler method-like functions (extern C++ calling conventions)
    pub fn FxScheduler_RegisterEffect(sched: *mut FxScheduler, effect_name: *const c_char);
    pub fn FxScheduler_PlayEffect_3(sched: *mut FxScheduler, effect_name: *const c_char, origin: *const vec3_t, dir: *const vec3_t);
    pub fn FxScheduler_PlayEffect_5(
        sched: *mut FxScheduler,
        effect_name: *const c_char,
        origin: *const vec3_t,
        dir: *const vec3_t,
        bolt_info: c_int,
        ent_num: c_int,
    );
    pub fn FxScheduler_PlayEffect_7(
        sched: *mut FxScheduler,
        effect_name: *const c_char,
        origin: *const vec3_t,
        dir: *const vec3_t,
        bolt_info: c_int,
        ent_num: c_int,
        isrelative: bool,
        loop_time: c_int,
    );
    pub fn FxScheduler_PlayEffect_id(sched: *mut FxScheduler, effect_id: c_int, origin: *const vec3_t, dir: *const vec3_t);
    pub fn FxScheduler_GetEffectCopy(sched: *mut FxScheduler, effect_name: *const c_char, handle: *mut c_int) -> *mut SEffectTemplate;
    pub fn FxScheduler_GetPrimitiveCopy(sched: *mut FxScheduler, temp: *mut SEffectTemplate, prim_name: *const c_char) -> *mut CPrimitiveTemplate;
}

// ============================================================================
// Static arrays for glass breaking
// ============================================================================

static mut offX: [[f32; 20]; 20] = [[0.0; 20]; 20];
static mut offZ: [[f32; 20]; 20] = [[0.0; 20]; 20];

// ============================================================================
// Functions
// ============================================================================

/*
====================
CG_AddTempLight
====================
*/
pub unsafe fn CG_AddTempLight(origin: *const vec3_t, scale: f32, color: *const vec3_t, msec: c_int) -> *mut localEntity_t {
    let ex: *mut localEntity_t;

    if msec <= 0 {
        CG_Error(b"CG_AddTempLight: msec = %i\0".as_ptr() as *const c_char, msec);
    }

    ex = CG_AllocLocalEntity();

    (*ex).leType = LE_LIGHT;

    (*ex).startTime = cg.time;
    (*ex).endTime = (*ex).startTime + msec;

    // set origin
    VectorCopy(origin, &mut (*ex).refEntity.origin);
    VectorCopy(origin, &mut (*ex).refEntity.oldorigin);

    VectorCopy(color, &mut (*ex).lightColor);
    (*ex).light = scale;

    ex
}

/*
-------------------------
CG_ExplosionEffects

Used to find the player and shake the camera if close enough
intensity ranges from 1 (minor tremble) to 16 (major quake)
-------------------------
*/

pub unsafe fn CG_ExplosionEffects(origin: *const vec3_t, intensity: f32, radius: c_int, time: c_int) {
    //FIXME: When exactly is the vieworg calculated in relation to the rest of the frame?s

    let mut dir: vec3_t = [0.0; 3];
    let mut dist: f32;
    let intensityScale: f32;
    let realIntensity: f32;

    VectorSubtract(&cg.refdef.vieworg, origin, &mut dir);
    dist = VectorNormalize(&mut dir);

    //Use the dir to add kick to the explosion

    if dist > radius as f32 {
        return;
    }

    intensityScale = 1.0 - (dist / radius as f32);
    realIntensity = intensity * intensityScale;

    CGCam_Shake(realIntensity, time);
}

/*
-------------------------
CG_MiscModelExplosion

Adds an explosion to a misc model breakables
-------------------------
*/

pub unsafe fn CG_MiscModelExplosion(mins: *const vec3_t, maxs: *const vec3_t, size: c_int, chunkType: c_int) {
    let mut ct: c_int = 13;
    let mut r: f32;
    let mut org: vec3_t = [0.0; 3];
    let mut mid: vec3_t = [0.0; 3];
    let mut dir: vec3_t = [0.0; 3];
    let mut effect: *const c_char = core::ptr::null();
    let mut effect2: *const c_char = core::ptr::null();

    VectorAdd(mins, maxs, &mut mid);
    VectorScale(&mid, 0.5, &mut mid);

    match chunkType {
        MAT_GLASS => {
            effect = b"chunks/glassbreak\0".as_ptr() as *const c_char;
            ct = 5;
        }
        MAT_GLASS_METAL => {
            effect = b"chunks/glassbreak\0".as_ptr() as *const c_char;
            effect2 = b"chunks/metalexplode\0".as_ptr() as *const c_char;
            ct = 5;
        }
        MAT_ELECTRICAL | MAT_ELEC_METAL => {
            effect = b"chunks/sparkexplode\0".as_ptr() as *const c_char;
            ct = 5;
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_CRATE1 | MAT_CRATE2 => {
            effect = b"chunks/metalexplode\0".as_ptr() as *const c_char;
            ct = 2;
        }
        MAT_GRATE1 => {
            effect = b"chunks/grateexplode\0".as_ptr() as *const c_char;
            ct = 8;
        }
        MAT_ROPE => {
            ct = 20;
            effect = b"chunks/ropebreak\0".as_ptr() as *const c_char;
        }
        MAT_WHITE_METAL | MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE => {
            //not sure what this crap is really supposed to be..
            match size {
                2 => {
                    effect = b"chunks/rockbreaklg\0".as_ptr() as *const c_char;
                }
                1 | _ => {
                    effect = b"chunks/rockbreakmed\0".as_ptr() as *const c_char;
                }
            }
        }
        _ => {}
    }

    if effect.is_null() {
        return;
    }

    ct += 7 * size;

    // FIXME: real precache .. VERify that these need to be here...don't think they would because the effects should be registered in g_breakable
    FxScheduler_RegisterEffect(addr_of_mut!(theFxScheduler), effect);

    if !effect2.is_null() {
        // FIXME: real precache
        FxScheduler_RegisterEffect(addr_of_mut!(theFxScheduler), effect2);
    }

    // spawn chunk roughly in the bbox of the thing..
    let mut i: c_int = 0;
    while i < ct {
        let mut j: c_int = 0;
        while j < 3 {
            r = random() * 0.8 + 0.1;
            org[j as usize] = r * (*mins)[j as usize] + (1.0 - r) * (*maxs)[j as usize];
            j += 1;
        }

        // shoot effect away from center
        VectorSubtract(&org, &mid, &mut dir);
        VectorNormalize(&mut dir);

        if !effect2.is_null() && (rand() & 1) != 0 {
            FxScheduler_PlayEffect_3(addr_of_mut!(theFxScheduler), effect2, &org, &dir);
        } else {
            FxScheduler_PlayEffect_3(addr_of_mut!(theFxScheduler), effect, &org, &dir);
        }

        i += 1;
    }
}

/*
-------------------------
CG_Chunks

Fun chunk spewer
-------------------------
*/

pub unsafe fn CG_Chunks(
    owner: c_int,
    origin: *const vec3_t,
    normal: *const vec3_t,
    mins: *const vec3_t,
    maxs: *const vec3_t,
    speed: f32,
    numChunks: c_int,
    chunkType: c_int,
    customChunk: c_int,
    mut baseScale: f32,
    customSound: c_int,
) {
    let mut le: *mut localEntity_t;
    let mut re: *mut refEntity_t;
    let mut dir: vec3_t = [0.0; 3];
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut chunkModel: c_int = 0;
    let mut bounce: c_int = LEBS_NONE;
    let mut r: f32;
    let mut speedMod: f32 = 1.0;
    let mut chunk: bool = false;

    if chunkType == MAT_NONE {
        // Well, we should do nothing
        return;
    }

    if customSound != 0 {
        if cgs.sound_precache[customSound as usize] != 0 {
            cgi_S_StartSound(core::ptr::null(), owner, CHAN_BODY, cgs.sound_precache[customSound as usize]);
        }
    }
    // Set up our chunk sound info...breaking sounds are done here so they are done once on breaking..some return instantly because the chunks are done with effects instead of models
    match chunkType {
        MAT_GLASS => {
            if customSound == 0 {
                cgi_S_StartSound(core::ptr::null(), owner, CHAN_BODY, cgs.media.glassChunkSound);
            }
            return;
        }
        MAT_GRATE1 => {
            if customSound == 0 {
                cgi_S_StartSound(core::ptr::null(), owner, CHAN_BODY, cgs.media.grateSound);
            }
            return;
        }
        MAT_ELECTRICAL => {
            // (sparks)
            if customSound == 0 {
                cgi_S_StartSound(
                    core::ptr::null(),
                    owner,
                    CHAN_BODY,
                    cgi_S_RegisterSound(b"sound/ambience/spark1.wav\0".as_ptr() as *const c_char),
                );
            }
            return;
        }
        MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_WHITE_METAL => {
            // not quite sure what this stuff is supposed to be...it's for Stu
            if customSound == 0 {
                cgi_S_StartSound(core::ptr::null(), owner, CHAN_BODY, cgs.media.rockBreakSound);
                bounce = LEBS_ROCK;
            }
            speedMod = 0.5; // rock blows up less
        }
        MAT_GLASS_METAL => {
            if customSound == 0 {
                cgi_S_StartSound(core::ptr::null(), owner, CHAN_BODY, cgs.media.glassChunkSound); // FIXME: should probably have a custom sound
                bounce = LEBS_METAL;
            }
        }
        MAT_CRATE1 | MAT_CRATE2 => {
            if customSound == 0 {
                cgi_S_StartSound(core::ptr::null(), owner, CHAN_BODY, cgs.media.crateBreakSound[Q_irand(0, 1) as usize]);
            }
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_ELEC_METAL => {
            // FIXME: maybe have its own sound?
            if customSound == 0 {
                cgi_S_StartSound(core::ptr::null(), owner, CHAN_BODY, cgs.media.chunkSound);
                bounce = LEBS_METAL;
            }
            speedMod = 0.8; // metal blows up a bit more
        }
        MAT_ROPE => {
            /*
            if ( !customSound )
            {
                cgi_S_StartSound( NULL, owner, CHAN_BODY, cgi_S_RegisterSound( "" ));  FIXME:  needs a sound
            }
            */
            return;
        }
        _ => {}
    }

    if baseScale <= 0.0 {
        baseScale = 1.0;
    }

    // Chunks
    i = 0;
    while i < numChunks {
        if customChunk > 0 {
            // Try to use a custom chunk.
            if cgs.model_draw[customChunk as usize] != 0 {
                chunk = true;
                chunkModel = cgs.model_draw[customChunk as usize];
            }
        }

        if !chunk {
            // No custom chunk.  Pick a random chunk type at run-time so we don't get the same chunks
            match chunkType {
                MAT_METAL2 => {
                    //bluegrey
                    chunkModel = cgs.media.chunkModels[0][Q_irand(0, 3) as usize];
                }
                MAT_GREY_STONE => {
                    //gray
                    chunkModel = cgs.media.chunkModels[CHUNK_ROCK1 as usize][Q_irand(0, 3) as usize];
                }
                MAT_LT_STONE => {
                    //tan
                    chunkModel = cgs.media.chunkModels[CHUNK_ROCK2 as usize][Q_irand(0, 3) as usize];
                }
                MAT_DRK_STONE => {
                    //brown
                    chunkModel = cgs.media.chunkModels[CHUNK_ROCK3 as usize][Q_irand(0, 3) as usize];
                }
                MAT_WHITE_METAL => {
                    chunkModel = cgs.media.chunkModels[CHUNK_WHITE_METAL as usize][Q_irand(0, 3) as usize];
                }
                MAT_CRATE1 => {
                    //yellow multi-colored crate chunks
                    chunkModel = cgs.media.chunkModels[CHUNK_CRATE1 as usize][Q_irand(0, 3) as usize];
                }
                MAT_CRATE2 => {
                    //red multi-colored crate chunks
                    chunkModel = cgs.media.chunkModels[CHUNK_CRATE2 as usize][Q_irand(0, 3) as usize];
                }
                MAT_ELEC_METAL | MAT_GLASS_METAL | MAT_METAL => {
                    //grey
                    chunkModel = cgs.media.chunkModels[CHUNK_METAL1 as usize][Q_irand(0, 3) as usize];
                }
                MAT_METAL3 => {
                    if rand() & 1 != 0 {
                        chunkModel = cgs.media.chunkModels[CHUNK_METAL1 as usize][Q_irand(0, 3) as usize];
                    } else {
                        chunkModel = cgs.media.chunkModels[CHUNK_METAL2 as usize][Q_irand(0, 3) as usize];
                    }
                }
                _ => {}
            }
        }

        // It wouldn't look good to throw a bunch of RGB axis models...so make sure we have something to work with.
        if chunkModel != 0 {
            le = CG_AllocLocalEntity();
            re = &mut (*le).refEntity;

            (*re).hModel = chunkModel;
            (*le).leType = LE_FRAGMENT;
            (*le).endTime = cg.time + 1300 + (random() * 900.0) as c_int;

            // spawn chunk roughly in the bbox of the thing...bias towards center in case thing blowing up doesn't complete fill its bbox.
            j = 0;
            while j < 3 {
                r = random() * 0.8 + 0.1;
                (*re).origin[j as usize] = r * (*mins)[j as usize] + (1.0 - r) * (*maxs)[j as usize];
                j += 1;
            }
            VectorCopy(&(*re).origin, &mut (*le).pos.trBase);

            // Move out from center of thing, otherwise you can end up things moving across the brush in an undesirable direction.  Visually looks wrong
            VectorSubtract(&(*re).origin, origin, &mut dir);
            VectorNormalize(&mut dir);
            VectorScale(&dir, Q_flrand(speed * 0.5, speed * 1.25) * speedMod, &mut (*le).pos.trDelta);

            // Angular Velocity
            VectorSet(&mut (*le).angles.trBase, random() * 360.0, random() * 360.0, random() * 360.0);

            (*le).angles.trDelta[0] = crandom();
            (*le).angles.trDelta[1] = crandom();
            (*le).angles.trDelta[2] = 0.0; // don't do roll

            VectorScale(&(*le).angles.trDelta, random() * 600.0 + 200.0, &mut (*le).angles.trDelta);

            (*le).pos.trType = TR_GRAVITY;
            (*le).angles.trType = TR_LINEAR;
            (*le).pos.trTime = cg.time;
            (*le).angles.trTime = cg.time;
            (*le).bounceFactor = 0.2 + random() * 0.2;
            (*le).leFlags |= LEF_TUMBLE;
            (*le).ownerGentNum = owner;
            (*le).leBounceSoundType = bounce;

            // Make sure that we have the desired start size set
            (*le).radius = Q_flrand(baseScale * 0.75, baseScale * 1.25);
            (*re).nonNormalizedAxes = 1;
            AxisCopy(&axisDefault, &mut (*re).axis); // could do an angles to axis, but this is cheaper and works ok
            k = 0;
            while k < 3 {
                VectorScale(&(*re).axis[k as usize], (*le).radius, &mut (*re).axis[k as usize]);
                k += 1;
            }
        }

        i += 1;
    }
}

pub unsafe fn CG_TestLine(start: *const vec3_t, end: *const vec3_t, time: c_int, color: u32, radius: c_int) {
    let le: *mut localEntity_t;
    let re: *mut refEntity_t;

    le = CG_AllocLocalEntity();
    (*le).leType = LE_LINE;
    (*le).startTime = cg.time;
    (*le).endTime = cg.time + time;
    (*le).lifeRate = 1.0 / (((*le).endTime - (*le).startTime) as f32);

    re = &mut (*le).refEntity;
    VectorCopy(start, &mut (*re).origin);
    VectorCopy(end, &mut (*re).oldorigin);
    (*re).shaderTime = cg.time as f32 / 1000.0;

    (*re).reType = RT_LINE;
    (*re).radius = 0.5 * (radius as f32);
    (*re).customShader = cgs.media.whiteShader; //trap_R_RegisterShaderNoMip("textures/colombia/canvas_doublesided");

    (*re).shaderTexCoord[0] = 1.0;
    (*re).shaderTexCoord[1] = 1.0;

    if color == 0 {
        (*re).shaderRGBA[0] = 0xff;
        (*re).shaderRGBA[1] = 0xff;
        (*re).shaderRGBA[2] = 0xff;
        (*re).shaderRGBA[3] = 0xff;
    } else {
        let mut c = color;
        (*re).shaderRGBA[0] = (c & 0xff) as u8;
        c >>= 8;
        (*re).shaderRGBA[1] = (c & 0xff) as u8;
        c >>= 8;
        (*re).shaderRGBA[2] = (c & 0xff) as u8;
        //		c >>= 8;
        //		(*re).shaderRGBA[3] = (c & 0xff) as u8;
        (*re).shaderRGBA[3] = 0xff;
    }

    (*le).color[3] = 1.0;
}

//----------------------------
//
// Breaking Glass Technology
//
//----------------------------

// Since we have shared verts when we tesselate the glass sheet, it helps to have a
//	random offset table set up up front...so that we can have more random looking breaks.

unsafe fn CG_DoGlassQuad(p: *const [vec3_t; 4], uv: *const [vec2_t; 4], stick: bool, time: c_int, dmgDir: *const vec3_t) {
    let mut bounce: f32;
    let mut rotDelta: vec3_t = [0.0; 3];
    let mut vel: vec3_t = [0.0; 3];
    let mut accel: vec3_t = [0.0; 3];
    let mut rgb1: vec3_t = [0.0; 3];

    VectorSet(&mut vel, crandom() * 12.0, crandom() * 12.0, -1.0);

    if !stick {
        // We aren't a motion delayed chunk, so let us move quickly
        VectorMA(&vel, 0.3, dmgDir, &mut vel);
    }

    // Set up acceleration due to gravity, 800 is standard QuakeIII gravity, so let's use something close
    VectorSet(&mut accel, 0.0, 0.0, -(600.0 + random() * 100.0));

    VectorSet(&mut rgb1, 1.0, 1.0, 1.0);

    // Being glass, we don't want to bounce much
    bounce = random() * 0.2 + 0.15;

    // Set up our random rotate, we only do PITCH and YAW, not ROLL.  This is something like degrees per second
    VectorSet(&mut rotDelta, crandom() * 40.0, crandom() * 40.0, 0.0);

    let pol = FX_AddPoly(
        p,
        uv,
        4, // verts, ST, vertCount
        &vel,
        &accel,
        // motion
        0.15,
        0.0,
        85.0, // alpha start, alpha end, alpha parm ( begin alpha fade when 85% of life is complete )
        &rgb1,
        &rgb1,
        0.0, // rgb start, rgb end, rgb parm ( not used )
        &rotDelta,
        bounce,
        time, // rotation amount, bounce, and time to delay motion for ( zero if no delay );
        3500 + (random() * 1000.0) as c_int, // life
        cgi_R_RegisterShader(b"gfx/misc/test_crackle\0".as_ptr() as *const c_char),
        FX_APPLY_PHYSICS | FX_ALPHA_NONLINEAR | FX_USE_ALPHA,
    );

    if random() > 0.95 && !pol.is_null() {
        // (*pol).AddFlags(FX_IMPACT_RUNS_FX | FX_KILL_ON_IMPACT);
        // (*pol).SetImpactFxID(theFxScheduler.RegisterEffect(b"misc/glass_impact\0".as_ptr() as *const c_char));
    }
}

fn CG_CalcBiLerp(verts: *const [vec3_t; 4], subVerts: *mut [vec3_t; 4], uv: *const [vec2_t; 4]) {
    unsafe {
        let mut temp: vec3_t = [0.0; 3];

        // Nasty crap
        VectorScale(&(*verts)[0], 1.0 - (*uv)[0][0], &mut (*subVerts)[0]);
        VectorMA(&(*subVerts)[0], (*uv)[0][0], &(*verts)[1], &mut (*subVerts)[0]);
        VectorScale(&(*subVerts)[0], 1.0 - (*uv)[0][1], &mut temp);
        VectorScale(&(*verts)[3], 1.0 - (*uv)[0][0], &mut (*subVerts)[0]);
        VectorMA(&(*subVerts)[0], (*uv)[0][0], &(*verts)[2], &mut (*subVerts)[0]);
        VectorMA(&temp, (*uv)[0][1], &(*subVerts)[0], &mut (*subVerts)[0]);

        VectorScale(&(*verts)[0], 1.0 - (*uv)[1][0], &mut (*subVerts)[1]);
        VectorMA(&(*subVerts)[1], (*uv)[1][0], &(*verts)[1], &mut (*subVerts)[1]);
        VectorScale(&(*subVerts)[1], 1.0 - (*uv)[1][1], &mut temp);
        VectorScale(&(*verts)[3], 1.0 - (*uv)[1][0], &mut (*subVerts)[1]);
        VectorMA(&(*subVerts)[1], (*uv)[1][0], &(*verts)[2], &mut (*subVerts)[1]);
        VectorMA(&temp, (*uv)[1][1], &(*subVerts)[1], &mut (*subVerts)[1]);

        VectorScale(&(*verts)[0], 1.0 - (*uv)[2][0], &mut (*subVerts)[2]);
        VectorMA(&(*subVerts)[2], (*uv)[2][0], &(*verts)[1], &mut (*subVerts)[2]);
        VectorScale(&(*subVerts)[2], 1.0 - (*uv)[2][1], &mut temp);
        VectorScale(&(*verts)[3], 1.0 - (*uv)[2][0], &mut (*subVerts)[2]);
        VectorMA(&(*subVerts)[2], (*uv)[2][0], &(*verts)[2], &mut (*subVerts)[2]);
        VectorMA(&temp, (*uv)[2][1], &(*subVerts)[2], &mut (*subVerts)[2]);

        VectorScale(&(*verts)[0], 1.0 - (*uv)[3][0], &mut (*subVerts)[3]);
        VectorMA(&(*subVerts)[3], (*uv)[3][0], &(*verts)[1], &mut (*subVerts)[3]);
        VectorScale(&(*subVerts)[3], 1.0 - (*uv)[3][1], &mut temp);
        VectorScale(&(*verts)[3], 1.0 - (*uv)[3][0], &mut (*subVerts)[3]);
        VectorMA(&(*subVerts)[3], (*uv)[3][0], &(*verts)[2], &mut (*subVerts)[3]);
        VectorMA(&temp, (*uv)[3][1], &(*subVerts)[3], &mut (*subVerts)[3]);
    }
}
// bilinear
//f(p',q') = (1 - y) · {[(1 - x) · f(p,q)] + [x · f(p,q+1)]} + y · {[(1 - x) · f(p+1,q)] + [x · f(p+1,q+1)]}.

fn CG_CalcHeightWidth(verts: *const [vec3_t; 4], height: *mut f32, width: *mut f32) {
    unsafe {
        let mut dir1: vec3_t = [0.0; 3];
        let mut dir2: vec3_t = [0.0; 3];
        let mut cross: vec3_t = [0.0; 3];

        VectorSubtract(&(*verts)[3], &(*verts)[0], &mut dir1); // v
        VectorSubtract(&(*verts)[1], &(*verts)[0], &mut dir2); // p-a
        CrossProduct(&dir1, &dir2, &mut cross);
        *width = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
        VectorSubtract(&(*verts)[2], &(*verts)[0], &mut dir2); // p-a
        CrossProduct(&dir1, &dir2, &mut cross);
        *width += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
        *width *= 0.5;

        VectorSubtract(&(*verts)[1], &(*verts)[0], &mut dir1); // v
        VectorSubtract(&(*verts)[2], &(*verts)[0], &mut dir2); // p-a
        CrossProduct(&dir1, &dir2, &mut cross);
        *height = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
        VectorSubtract(&(*verts)[3], &(*verts)[0], &mut dir2); // p-a
        CrossProduct(&dir1, &dir2, &mut cross);
        *height += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
        *height *= 0.5;
    }
}
//Consider a line in 3D with position vector "a" and direction vector "v" and
// let "p" be the position vector of an arbitrary point in 3D
//dist = len( crossprod(p-a,v) ) / len(v);

pub unsafe fn CG_InitGlass() {
    let mut i: c_int;
    let mut t: c_int;

    // Build a table first, so that we can do a more unpredictable crack scheme
    //	do it once, up front to save a bit of time.
    i = 0;
    while i < 20 {
        t = 0;
        while t < 20 {
            offX[t as usize][i as usize] = crandom() * 0.03;
            offZ[i as usize][t as usize] = crandom() * 0.03;
            t += 1;
        }
        i += 1;
    }
}

const TIME_DECAY_SLOW: f32 = 0.1;
const TIME_DECAY_MED: f32 = 0.04;
const TIME_DECAY_FAST: f32 = 0.009;

pub unsafe fn CG_DoGlass(verts: *const [vec3_t; 4], normal: *const vec3_t, dmgPt: *const vec3_t, dmgDir: *const vec3_t, dmgRadius: f32) {
    let mut i: c_int;
    let mut t: c_int;
    let mut mxHeight: c_int;
    let mut mxWidth: c_int;
    let mut height: f32 = 0.0;
    let mut width: f32 = 0.0;
    let mut stepWidth: f32;
    let mut stepHeight: f32;
    let mut timeDecay: f32;
    let mut x: f32;
    let mut z: f32;
    let mut xx: f32;
    let mut zz: f32;
    let mut time: c_int = 0;
    let mut stick: bool = true;
    let mut subVerts: [vec3_t; 4] = [[0.0; 3]; 4];
    let mut biPoints: [vec2_t; 4] = [[0.0; 2]; 4];

    // To do a smarter tesselation, we should figure out the relative height and width of the brush face,
    //	then use this to pick a lod value from 1-3 in each axis.  This will give us 1-9 lod levels, which will
    //	hopefully be sufficient.
    CG_CalcHeightWidth(verts, &mut height, &mut width);

    cgi_S_StartSound(
        dmgPt,
        -1,
        CHAN_AUTO,
        cgi_S_RegisterSound(b"sound/effects/glassbreak1.wav\0".as_ptr() as *const c_char),
    );

    // Pick "LOD" for height
    if height < 100.0 {
        stepHeight = 0.2;
        mxHeight = 5;
        timeDecay = TIME_DECAY_SLOW;
    }
    /*	else if ( height > 220 ) // was originally mxHeight = 20....but removing this whole section because it causes huge number of chunks...which is bad
    {
        stepHeight = 0.075;
        mxHeight = 15;
        timeDecay = TIME_DECAY_FAST;
    }*/
    else {
        stepHeight = 0.1;
        mxHeight = 10;
        timeDecay = TIME_DECAY_MED;
    }

    // Pick "LOD" for width
    if width < 100.0 {
        stepWidth = 0.2;
        mxWidth = 5;
        timeDecay = (timeDecay + TIME_DECAY_SLOW) * 0.5;
    }
    /*	else if ( width > 220 ) // don't do this because it causes too much chug with large glass panes...especially when more than one pane can be broken at a time
    {
        stepWidth = 0.075;
        mxWidth = 15;
        timeDecay = ( timeDecay + TIME_DECAY_FAST ) * 0.5;
    }*/
    else {
        stepWidth = 0.1;
        mxWidth = 10;
        timeDecay = (timeDecay + TIME_DECAY_MED) * 0.5;
    }

    z = 0.0;
    i = 0;
    while z < 1.0 {
        x = 0.0;
        t = 0;
        while x < 1.0 {
            // This is nasty..we do this because we don't want to add a random offset on the edge of the glass brush
            //	...but we do in the center, otherwise the breaking scheme looks way too orderly
            if t > 0 && t < mxWidth {
                xx = x - offX[i as usize][t as usize];
            } else {
                xx = x;
            }

            if i > 0 && i < mxHeight {
                zz = z - offZ[t as usize][i as usize];
            } else {
                zz = z;
            }

            Vector2Set(&mut biPoints[0], xx, zz);

            if t + 1 > 0 && t + 1 < mxWidth {
                xx = x - offX[i as usize][(t + 1) as usize];
            } else {
                xx = x;
            }

            if i > 0 && i < mxHeight {
                zz = z - offZ[(t + 1) as usize][i as usize];
            } else {
                zz = z;
            }

            Vector2Set(&mut biPoints[1], xx + stepWidth, zz);

            if t + 1 > 0 && t + 1 < mxWidth {
                xx = x - offX[(i + 1) as usize][(t + 1) as usize];
            } else {
                xx = x;
            }

            if i + 1 > 0 && i + 1 < mxHeight {
                zz = z - offZ[(t + 1) as usize][(i + 1) as usize];
            } else {
                zz = z;
            }

            Vector2Set(&mut biPoints[2], xx + stepWidth, zz + stepHeight);

            if t > 0 && t < mxWidth {
                xx = x - offX[(i + 1) as usize][t as usize];
            } else {
                xx = x;
            }

            if i + 1 > 0 && i + 1 < mxHeight {
                zz = z - offZ[t as usize][(i + 1) as usize];
            } else {
                zz = z;
            }

            Vector2Set(&mut biPoints[3], xx, zz + stepHeight);

            CG_CalcBiLerp(verts, &mut subVerts, &biPoints);

            let dif = DistanceSquared(&subVerts[0], dmgPt) * timeDecay - random() * 32.0;

            // If we decrease dif, we are increasing the impact area, making it more likely to blow out large holes
            let dif = dif - dmgRadius * dmgRadius;

            if dif > 1.0 {
                stick = true;
                time = (dif + random() * 200.0) as c_int;
            } else {
                stick = false;
                time = 0;
            }

            CG_DoGlassQuad(&subVerts, &biPoints, stick, time, dmgDir);

            x += stepWidth;
            t += 1;
        }
        z += stepHeight;
        i += 1;
    }
}

/*
=================
CG_Seeker
=================
*/
/*void CG_Seeker( centity_t *cent )
{
    refEntity_t	re;

    vec3_t	seekerOrg, viewAng;
    float	angle, c;

    // must match cg_effects ( CG_Seeker ) & g_weapon ( SeekerAcquiresTarget ) & cg_weapons ( CG_FireSeeker )
    angle = cg.time * 0.004f;
    c = cos( angle );

    seekerOrg[0] = cent->lerpOrigin[0] + 18 * c;
    seekerOrg[1] = cent->lerpOrigin[1] + 18 * sin(angle);
    seekerOrg[2] = cent->lerpOrigin[2] + cg.predicted_player_state.viewheight + 8 + (3 * cos(cg.time * 0.001));

    memset( &re, 0, sizeof( re ) );

    re.reType = RT_MODEL;
    VectorCopy( seekerOrg, re.origin);
    re.hModel = cgi_R_RegisterModel( "models/items/remote.md3" );

    VectorCopy( cent->lerpAngles, viewAng ); // so the seeker faces the same direction the player is
    viewAng[PITCH] = -90; // but, we don't want the seeker facing up or down, always horizontal
    viewAng[YAW] += c * 15.f;

    AnglesToAxis( viewAng, re.axis );
    VectorScale( re.axis[0], 0.5f, re.axis[0] );
    VectorScale( re.axis[1], 0.5f, re.axis[1] );
    VectorScale( re.axis[2], 0.5f, re.axis[2] );
    re.nonNormalizedAxes = qtrue;

    cgi_R_AddRefEntityToScene( &re );
}
*/

//------------------------------------------------------------------------------------------

pub unsafe fn CG_DrawTargetBeam(
    start: *const vec3_t,
    end: *const vec3_t,
    norm: *const vec3_t,
    beamFx: *const c_char,
    impactFx: *const c_char,
) {
    let mut handle: c_int = 0;
    let mut dir: vec3_t = [0.0; 3];
    let mut temp: *mut SEffectTemplate;

    // overriding the effect, so give us a copy first
    temp = FxScheduler_GetEffectCopy(addr_of_mut!(theFxScheduler), beamFx, &mut handle);

    VectorSubtract(start, end, &mut dir);
    VectorNormalize(&mut dir);

    if !temp.is_null() {
        // have a copy, so get the line element out of there
        let prim = FxScheduler_GetPrimitiveCopy(addr_of_mut!(theFxScheduler), temp, b"beam\0".as_ptr() as *const c_char);

        if !prim.is_null() {
            // we have the primitive, so modify the endpoint
            // prim->mOrigin2X.SetRange(end[0], end[0]);
            // prim->mOrigin2Y.SetRange(end[1], end[1]);
            // prim->mOrigin2Z.SetRange(end[2], end[2]);

            // have a copy, so get the line element out of there
            let prim2 = FxScheduler_GetPrimitiveCopy(addr_of_mut!(theFxScheduler), temp, b"glow\0".as_ptr() as *const c_char);

            // glow is not required
            if !prim2.is_null() {
                // we have the primitive, so modify the endpoint
                // prim2->mOrigin2X.SetRange(end[0], end[0]);
                // prim2->mOrigin2Y.SetRange(end[1], end[1]);
                // prim2->mOrigin2Z.SetRange(end[2], end[2]);
            }

            // play the modified effect
            FxScheduler_PlayEffect_id(addr_of_mut!(theFxScheduler), handle, start, &dir);
        }
    }

    if !impactFx.is_null() {
        FxScheduler_PlayEffect_3(addr_of_mut!(theFxScheduler), impactFx, end, norm);
    }
}

pub unsafe fn CG_PlayEffectBolted(
    fxName: *const c_char,
    modelIndex: c_int,
    boltIndex: c_int,
    entNum: c_int,
    origin: *const vec3_t,
    iLoopTime: c_int,
    isRelative: bool,
) {
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3]; //FIXME: shouldn't this be initialized to something?  It isn't in the EV_PLAY_EFFECT call... irrelevant?
    let mut boltInfo: c_int = 0;

    //pack the data into boltInfo as if we were going to send it over the network
    gi.G2API_AttachEnt(&mut boltInfo, &mut (*g_entities[entNum as usize]).ghoul2[modelIndex as usize], boltIndex, entNum, modelIndex);
    //send direcly to FX scheduler
    FxScheduler_PlayEffect_7(
        addr_of_mut!(theFxScheduler),
        fxName,
        origin,
        axis.as_ptr(),
        boltInfo,
        -1,
        isRelative,
        iLoopTime,
    ); //iLoopTime 0 = not looping, 1 for infinite, else duration
}

pub unsafe fn CG_PlayEffectIDBolted(
    fxID: c_int,
    modelIndex: c_int,
    boltIndex: c_int,
    entNum: c_int,
    origin: *const vec3_t,
    iLoopTime: c_int,
    isRelative: bool,
) {
    let fxName = CG_ConfigString(/* CS_EFFECTS +*/ fxID);
    CG_PlayEffectBolted(fxName, modelIndex, boltIndex, entNum, origin, iLoopTime, isRelative);
}

pub unsafe fn CG_PlayEffectOnEnt(fxName: *const c_char, clientNum: c_int, origin: *const vec3_t, fwd: *const vec3_t) {
    let mut temp: vec3_t = [0.0; 3];
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];

    // Assume angles, we'll do a cross product to finish up
    VectorCopy(fwd, &mut axis[0]);
    MakeNormalVectors(fwd, &mut axis[1], &mut temp);
    CrossProduct(&axis[0], &axis[1], &mut axis[2]);
    //call FX scheduler directly
    FxScheduler_PlayEffect_5(addr_of_mut!(theFxScheduler), fxName, origin, axis.as_ptr(), -1, clientNum);
}

pub unsafe fn CG_PlayEffectIDOnEnt(fxID: c_int, clientNum: c_int, origin: *const vec3_t, fwd: *const vec3_t) {
    let fxName = CG_ConfigString(/* CS_EFFECTS +*/ fxID);
    CG_PlayEffectOnEnt(fxName, clientNum, origin, fwd);
}

pub unsafe fn CG_PlayEffect(fxName: *const c_char, origin: *const vec3_t, fwd: *const vec3_t) {
    let mut temp: vec3_t = [0.0; 3];
    let mut axis: [vec3_t; 3] = [[0.0; 3]; 3];

    // Assume angles, we'll do a cross product to finish up
    VectorCopy(fwd, &mut axis[0]);
    MakeNormalVectors(fwd, &mut axis[1], &mut temp);
    CrossProduct(&axis[0], &axis[1], &mut axis[2]);
    //call FX scheduler directly
    FxScheduler_PlayEffect_5(addr_of_mut!(theFxScheduler), fxName, origin, axis.as_ptr(), -1, -1);
}

pub unsafe fn CG_PlayEffectID(fxID: c_int, origin: *const vec3_t, fwd: *const vec3_t) {
    let fxName = CG_ConfigString(/* CS_EFFECTS +*/ fxID);
    CG_PlayEffect(fxName, origin, fwd);
}
