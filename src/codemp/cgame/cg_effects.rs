// Copyright (C) 1999-2000 Id Software, Inc.
//
// cg_effects.c -- these functions generate localentities, usually as a result
// of event processing

use core::ffi::{c_int, c_char};

// Assuming cg_local.h provides types and functions via module
// use crate::codemp::cgame::cg_local::*;

// ==================
// CG_BubbleTrail
//
// Bullets shot underwater
// ==================
pub fn CG_BubbleTrail(start: &[f32; 3], end: &[f32; 3], spacing: f32) {
    let mut move_pos: [f32; 3] = [0.0; 3];
    let mut vec: [f32; 3] = [0.0; 3];
    let mut len: f32;
    let mut i: i32;

    if unsafe { cg_noProjectileTrail.integer } != 0 {
        return;
    }

    VectorCopy(start, &mut move_pos);
    VectorSubtract(end, start, &mut vec);
    len = VectorNormalize(&mut vec);

    // advance a random amount first
    i = (rand() % (spacing as i32)) as i32;
    VectorMA(&move_pos, i as f32, &vec, &mut move_pos);

    VectorScale(&vec, spacing, &mut vec);

    while (i as f32) < len {
        let le: *mut localEntity_t;
        let re: *mut refEntity_t;

        le = CG_AllocLocalEntity();
        unsafe {
            (*le).leFlags = LEF_PUFF_DONT_SCALE;
            (*le).leType = LE_MOVE_SCALE_FADE;
            (*le).startTime = cg.time;
            (*le).endTime = cg.time + 1000 + (random() * 250.0) as i32;
            (*le).lifeRate = 1.0 / (((*le).endTime - (*le).startTime) as f32);

            re = &mut (*le).refEntity;
            (*re).shaderTime = cg.time as f32 / 1000.0;

            (*re).reType = RT_SPRITE;
            (*re).rotation = 0.0;
            (*re).radius = 3.0;
            (*re).customShader = 0; //cgs.media.waterBubbleShader;
            (*re).shaderRGBA[0] = 0xff;
            (*re).shaderRGBA[1] = 0xff;
            (*re).shaderRGBA[2] = 0xff;
            (*re).shaderRGBA[3] = 0xff;

            (*le).color[3] = 1.0;

            (*le).pos.trType = TR_LINEAR;
            (*le).pos.trTime = cg.time;
            VectorCopy(&move_pos, &mut (*le).pos.trBase);
            (*le).pos.trDelta[0] = crandom() * 5.0;
            (*le).pos.trDelta[1] = crandom() * 5.0;
            (*le).pos.trDelta[2] = crandom() * 5.0 + 6.0;

            VectorAdd(&move_pos, &vec, &mut move_pos);
        }

        i = (i as f32 + spacing) as i32;
    }
}

// =====================
// CG_SmokePuff
//
// Adds a smoke puff or blood trail localEntity.
// =====================
pub fn CG_SmokePuff(
    p: &[f32; 3],
    vel: &[f32; 3],
    radius: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    duration: f32,
    startTime: i32,
    fadeInTime: i32,
    leFlags: i32,
    hShader: i32,
) -> *mut localEntity_t {
    static mut seed: i32 = 0x92;
    let le: *mut localEntity_t;
    let re: *mut refEntity_t;
    //	int fadeInTime = startTime + duration / 2;

    le = CG_AllocLocalEntity();
    unsafe {
        (*le).leFlags = leFlags;
        (*le).radius = radius;

        re = &mut (*le).refEntity;
        (*re).rotation = Q_random(&mut seed) * 360.0;
        (*re).radius = radius;
        (*re).shaderTime = startTime as f32 / 1000.0;

        (*le).leType = LE_MOVE_SCALE_FADE;
        (*le).startTime = startTime;
        (*le).fadeInTime = fadeInTime;
        (*le).endTime = startTime + (duration as i32);
        if fadeInTime > startTime {
            (*le).lifeRate = 1.0 / (((*le).endTime - (*le).fadeInTime) as f32);
        } else {
            (*le).lifeRate = 1.0 / (((*le).endTime - (*le).startTime) as f32);
        }
        (*le).color[0] = r;
        (*le).color[1] = g;
        (*le).color[2] = b;
        (*le).color[3] = a;

        (*le).pos.trType = TR_LINEAR;
        (*le).pos.trTime = startTime;
        VectorCopy(vel, &mut (*le).pos.trDelta);
        VectorCopy(p, &mut (*le).pos.trBase);

        VectorCopy(p, &mut (*re).origin);
        (*re).customShader = hShader;

        (*re).shaderRGBA[0] = ((*le).color[0] * 0xff as f32) as u8;
        (*re).shaderRGBA[1] = ((*le).color[1] * 0xff as f32) as u8;
        (*re).shaderRGBA[2] = ((*le).color[2] * 0xff as f32) as u8;
        (*re).shaderRGBA[3] = 0xff;

        (*re).reType = RT_SPRITE;
        (*re).radius = (*le).radius;
    }

    le
}

pub fn CGDEBUG_SaberColor(saberColor: i32) -> i32 {
    match saberColor {
        SABER_RED => 0x000000ff,
        SABER_ORANGE => 0x000088ff,
        SABER_YELLOW => 0x0000ffff,
        SABER_GREEN => 0x0000ff00,
        SABER_BLUE => 0x00ff0000,
        SABER_PURPLE => 0x00ff00ff,
        _ => saberColor,
    }
}

pub fn CG_TestLine(start: &[f32; 3], end: &[f32; 3], time: i32, color: u32, radius: i32) {
    let le: *mut localEntity_t;
    let re: *mut refEntity_t;

    le = CG_AllocLocalEntity();
    unsafe {
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
        (*re).customShader = cgs.media.whiteShader; //trap_R_RegisterShader("textures/colombia/canvas_doublesided");

        (*re).shaderTexCoord[0] = 1.0;
        (*re).shaderTexCoord[1] = 1.0;

        if color == 0 {
            (*re).shaderRGBA[0] = 0xff;
            (*re).shaderRGBA[1] = 0xff;
            (*re).shaderRGBA[2] = 0xff;
            (*re).shaderRGBA[3] = 0xff;
        } else {
            let mut color_val = CGDEBUG_SaberColor(color as i32) as u32;
            (*re).shaderRGBA[0] = (color_val & 0xff) as u8;
            color_val >>= 8;
            (*re).shaderRGBA[1] = (color_val & 0xff) as u8;
            color_val >>= 8;
            (*re).shaderRGBA[2] = (color_val & 0xff) as u8;
            //		color >>= 8;
            //		re->shaderRGBA[3] = color & 0xff;
            (*re).shaderRGBA[3] = 0xff;
        }

        (*le).color[3] = 1.0;

        //(*re).renderfx |= RF_DEPTHHACK;
    }
}

// ==================
// CG_ThrowChunk
// ==================
pub fn CG_ThrowChunk(origin: &[f32; 3], velocity: &[f32; 3], hModel: i32, optionalSound: i32, startalpha: i32) {
    let le: *mut localEntity_t;
    let re: *mut refEntity_t;

    le = CG_AllocLocalEntity();
    unsafe {
        re = &mut (*le).refEntity;

        (*le).leType = LE_FRAGMENT;
        (*le).startTime = cg.time;
        (*le).endTime = (*le).startTime + 5000 + (random() * 3000.0) as i32;

        VectorCopy(origin, &mut (*re).origin);
        AxisCopy(&axisDefault, &mut (*re).axis);
        (*re).hModel = hModel;

        (*le).pos.trType = TR_GRAVITY;
        (*le).angles.trType = TR_GRAVITY;
        VectorCopy(origin, &mut (*le).pos.trBase);
        VectorCopy(velocity, &mut (*le).pos.trDelta);
        VectorSet(&mut (*le).angles.trBase, 20.0, 20.0, 20.0);
        VectorCopy(velocity, &mut (*le).angles.trDelta);
        (*le).pos.trTime = cg.time;
        (*le).angles.trTime = cg.time;

        (*le).leFlags = LEF_TUMBLE;

        (*le).angles.trBase[YAW] = 180.0;

        (*le).bounceFactor = 0.3;
        (*le).bounceSound = optionalSound;

        (*le).forceAlpha = startalpha;
    }
}

//----------------------------
//
// Breaking Glass Technology
//
//----------------------------

// Since we have shared verts when we tesselate the glass sheet, it helps to have a
//	random offset table set up up front.

static mut offX: [[f32; 20]; 20] = [[0.0; 20]; 20];
static mut offZ: [[f32; 20]; 20] = [[0.0; 20]; 20];

const FX_ALPHA_NONLINEAR: u32 = 0x00000004;
const FX_APPLY_PHYSICS: u32 = 0x02000000;
const FX_USE_ALPHA: u32 = 0x08000000;

unsafe fn CG_DoGlassQuad(
    p: &[[f32; 3]; 4],
    uv: &[[f32; 2]; 4],
    stick: bool,
    time: i32,
    dmgDir: &[f32; 3],
) {
    let mut bounce: f32;
    let mut rotDelta: [f32; 3] = [0.0; 3];
    let mut vel: [f32; 3] = [0.0; 3];
    let mut accel: [f32; 3] = [0.0; 3];
    let mut rgb1: [f32; 3] = [0.0; 3];
    let mut apArgs: addpolyArgStruct_t;
    let mut i: i32;
    let mut i_2: i32;

    VectorSet(&mut vel, crandom() * 12.0, crandom() * 12.0, -1.0);

    if !stick {
        // We aren't a motion delayed chunk, so let us move quickly
        VectorMA(&vel, 0.3, dmgDir, &mut vel);
    }

    // Set up acceleration due to gravity, 800 is standard QuakeIII gravity, so let's use something close
    VectorSet(&mut accel, 0.0, 0.0, -(600.0 + random() * 100.0));

    // We are using an additive shader, so let's set the RGB low so we look more like transparent glass
    //	VectorSet( rgb1, 0.1f, 0.1f, 0.1f );
    VectorSet(&mut rgb1, 1.0, 1.0, 1.0);

    // Being glass, we don't want to bounce much
    bounce = random() * 0.2 + 0.15;

    // Set up our random rotate, we only do PITCH and YAW, not ROLL.  This is something like degrees per second
    VectorSet(&mut rotDelta, crandom() * 40.0, crandom() * 40.0, 0.0);

    //In an ideal world, this might actually work.
    /*
    CPoly *pol = FX_AddPoly(p, uv, 4,			// verts, ST, vertCount
            vel, accel,				// motion
            0.15f, 0.0f, 85.0f,		// alpha start, alpha end, alpha parm ( begin alpha fade when 85% of life is complete )
            rgb1, rgb1, 0.0f,		// rgb start, rgb end, rgb parm ( not used )
            rotDelta, bounce, time,	// rotation amount, bounce, and time to delay motion for ( zero if no delay );
            6000,					// life
            cgi_R_RegisterShader( "gfx/misc/test_crackle" ),
            FX_APPLY_PHYSICS | FX_ALPHA_NONLINEAR | FX_USE_ALPHA );

    if ( random() > 0.95f && pol )
    {
        pol->AddFlags( FX_IMPACT_RUNS_FX | FX_KILL_ON_IMPACT );
        pol->SetImpactFxID( theFxScheduler.RegisterEffect( "glass_impact" ));
    }
    */

    //rww - this is dirty.

    i = 0;
    i_2 = 0;

    while i < 4 {
        while i_2 < 3 {
            apArgs.p[i as usize][i_2 as usize] = p[i as usize][i_2 as usize];

            i_2 += 1;
        }

        i_2 = 0;
        i += 1;
    }

    i = 0;
    i_2 = 0;

    while i < 4 {
        while i_2 < 2 {
            apArgs.ev[i as usize][i_2 as usize] = uv[i as usize][i_2 as usize];

            i_2 += 1;
        }

        i_2 = 0;
        i += 1;
    }

    apArgs.numVerts = 4;
    VectorCopy(&vel, &mut apArgs.vel);
    VectorCopy(&accel, &mut apArgs.accel);

    apArgs.alpha1 = 0.15;
    apArgs.alpha2 = 0.0;
    apArgs.alphaParm = 85.0;

    VectorCopy(&rgb1, &mut apArgs.rgb1);
    VectorCopy(&rgb1, &mut apArgs.rgb2);

    apArgs.rgbParm = 0.0;

    VectorCopy(&rotDelta, &mut apArgs.rotationDelta);

    apArgs.bounce = bounce;
    apArgs.motionDelay = time;
    apArgs.killTime = 6000;
    apArgs.shader = cgs.media.glassShardShader;
    apArgs.flags = FX_APPLY_PHYSICS | FX_ALPHA_NONLINEAR | FX_USE_ALPHA;

    trap_FX_AddPoly(&apArgs);
}

unsafe fn CG_CalcBiLerp(verts: &[[f32; 3]; 4], subVerts: &mut [[f32; 3]; 4], uv: &[[f32; 2]; 4]) {
    let mut temp: [f32; 3] = [0.0; 3];

    // Nasty crap
    VectorScale(&verts[0], 1.0 - uv[0][0], &mut subVerts[0]);
    VectorMA(&subVerts[0], uv[0][0], &verts[1], &mut subVerts[0]);
    VectorScale(&subVerts[0], 1.0 - uv[0][1], &mut temp);
    VectorScale(&verts[3], 1.0 - uv[0][0], &mut subVerts[0]);
    VectorMA(&subVerts[0], uv[0][0], &verts[2], &mut subVerts[0]);
    VectorMA(&temp, uv[0][1], &subVerts[0], &mut subVerts[0]);

    VectorScale(&verts[0], 1.0 - uv[1][0], &mut subVerts[1]);
    VectorMA(&subVerts[1], uv[1][0], &verts[1], &mut subVerts[1]);
    VectorScale(&subVerts[1], 1.0 - uv[1][1], &mut temp);
    VectorScale(&verts[3], 1.0 - uv[1][0], &mut subVerts[1]);
    VectorMA(&subVerts[1], uv[1][0], &verts[2], &mut subVerts[1]);
    VectorMA(&temp, uv[1][1], &subVerts[1], &mut subVerts[1]);

    VectorScale(&verts[0], 1.0 - uv[2][0], &mut subVerts[2]);
    VectorMA(&subVerts[2], uv[2][0], &verts[1], &mut subVerts[2]);
    VectorScale(&subVerts[2], 1.0 - uv[2][1], &mut temp);
    VectorScale(&verts[3], 1.0 - uv[2][0], &mut subVerts[2]);
    VectorMA(&subVerts[2], uv[2][0], &verts[2], &mut subVerts[2]);
    VectorMA(&temp, uv[2][1], &subVerts[2], &mut subVerts[2]);

    VectorScale(&verts[0], 1.0 - uv[3][0], &mut subVerts[3]);
    VectorMA(&subVerts[3], uv[3][0], &verts[1], &mut subVerts[3]);
    VectorScale(&subVerts[3], 1.0 - uv[3][1], &mut temp);
    VectorScale(&verts[3], 1.0 - uv[3][0], &mut subVerts[3]);
    VectorMA(&subVerts[3], uv[3][0], &verts[2], &mut subVerts[3]);
    VectorMA(&temp, uv[3][1], &subVerts[3], &mut subVerts[3]);
}
// bilinear
//f(p',q') = (1 - y) · {[(1 - x) · f(p,q)] + [x · f(p,q+1)]} + y · {[(1 - x) · f(p+1,q)] + [x · f(p+1,q+1)]}.

unsafe fn CG_CalcHeightWidth(verts: &[[f32; 3]; 4], height: &mut f32, width: &mut f32) {
    let mut dir1: [f32; 3] = [0.0; 3];
    let mut dir2: [f32; 3] = [0.0; 3];
    let mut cross: [f32; 3] = [0.0; 3];

    VectorSubtract(&verts[3], &verts[0], &mut dir1); // v
    VectorSubtract(&verts[1], &verts[0], &mut dir2); // p-a
    CrossProduct(&dir1, &dir2, &mut cross);
    *width = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    VectorSubtract(&verts[2], &verts[0], &mut dir2); // p-a
    CrossProduct(&dir1, &dir2, &mut cross);
    *width += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    *width *= 0.5;

    VectorSubtract(&verts[1], &verts[0], &mut dir1); // v
    VectorSubtract(&verts[2], &verts[0], &mut dir2); // p-a
    CrossProduct(&dir1, &dir2, &mut cross);
    *height = VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    VectorSubtract(&verts[3], &verts[0], &mut dir2); // p-a
    CrossProduct(&dir1, &dir2, &mut cross);
    *height += VectorNormalize(&mut cross) / VectorNormalize(&mut dir1); // v
    *height *= 0.5;
}
//Consider a line in 3D with position vector "a" and direction vector "v" and
// let "p" be the position vector of an arbitrary point in 3D
//dist = len( crossprod(p-a,v) ) / len(v);

pub fn CG_InitGlass() {
    let mut i: i32;
    let mut t: i32;

    // Build a table first, so that we can do a more unpredictable crack scheme
    //	do it once, up front to save a bit of time.
    for i in 0..20 {
        for t in 0..20 {
            unsafe {
                offX[t as usize][i as usize] = crandom() * 0.03;
                offZ[i as usize][t as usize] = crandom() * 0.03;
            }
        }
    }
}

pub fn Vector2Set(a: &mut [f32; 2], b: f32, c: f32) {
    a[0] = b;
    a[1] = c;
}

const TIME_DECAY_SLOW: f32 = 0.1;
const TIME_DECAY_MED: f32 = 0.04;
const TIME_DECAY_FAST: f32 = 0.009;

unsafe fn CG_DoGlass(
    verts: &[[f32; 3]; 4],
    normal: &[f32; 3],
    dmgPt: &[f32; 3],
    dmgDir: &[f32; 3],
    dmgRadius: f32,
    maxShards: i32,
) {
    let mut i: i32;
    let mut t: i32;
    let mut mxHeight: i32;
    let mut mxWidth: i32;
    let mut height: f32 = 0.0;
    let mut width: f32 = 0.0;
    let mut stepWidth: f32;
    let mut stepHeight: f32;
    let mut timeDecay: f32;
    let mut x: f32;
    let mut z: f32;
    let mut xx: f32;
    let mut zz: f32;
    let mut dif: f32;
    let mut time: i32 = 0;
    let mut glassShards: i32 = 0;
    let mut stick: bool = true;
    let mut subVerts: [[f32; 3]; 4] = [[0.0; 3]; 4];
    let mut biPoints: [[f32; 2]; 4] = [[0.0; 2]; 4];

    // To do a smarter tesselation, we should figure out the relative height and width of the brush face,
    //	then use this to pick a lod value from 1-3 in each axis.  This will give us 1-9 lod levels, which will
    //	hopefully be sufficient.
    CG_CalcHeightWidth(verts, &mut height, &mut width);

    trap_S_StartSound(dmgPt, -1, CHAN_AUTO, trap_S_RegisterSound("sound/effects/glassbreak1.wav"));

    // Pick "LOD" for height
    if height < 100.0 {
        stepHeight = 0.2;
        mxHeight = 5;
        timeDecay = TIME_DECAY_SLOW;
    } else if height > 220.0 {
        stepHeight = 0.05;
        mxHeight = 20;
        timeDecay = TIME_DECAY_FAST;
    } else {
        stepHeight = 0.1;
        mxHeight = 10;
        timeDecay = TIME_DECAY_MED;
    }

    // Pick "LOD" for width
    /*
    if ( width < 100 )
    {
        stepWidth = 0.2f;
        mxWidth = 5;
        timeDecay = ( timeDecay + TIME_DECAY_SLOW ) * 0.5f;
    }
    else if ( width > 220 )
    {
        stepWidth = 0.05f;
        mxWidth = 20;
        timeDecay = ( timeDecay + TIME_DECAY_FAST ) * 0.5f;
    }
    else
    {
        stepWidth = 0.1f;
        mxWidth = 10;
        timeDecay = ( timeDecay + TIME_DECAY_MED ) * 0.5f;
    }
    */

    //Attempt to scale the glass directly to the size of the window

    stepWidth = 0.25 - (width * 0.0002); //(width*0.0005));
    mxWidth = (width * 0.2) as i32;
    timeDecay = (timeDecay + TIME_DECAY_FAST) * 0.5;

    if stepWidth < 0.01 {
        stepWidth = 0.01;
    }
    if mxWidth < 5 {
        mxWidth = 5;
    }

    z = 0.0;
    i = 0;
    while z < 1.0 {
        x = 0.0;
        t = 0;
        while x < 1.0 {
            // This is nasty..
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

            dif = DistanceSquared(&subVerts[0], dmgPt) * timeDecay - random() * 32.0;

            // If we decrease dif, we are increasing the impact area, making it more likely to blow out large holes
            dif -= dmgRadius * dmgRadius;

            if dif > 1.0 {
                stick = true;
                time = (dif + random() * 200.0) as i32;
            } else {
                stick = false;
                time = 0;
            }

            CG_DoGlassQuad(&subVerts, &biPoints, stick, time, dmgDir);
            glassShards += 1;

            if maxShards != 0 && glassShards >= maxShards {
                return;
            }

            x += stepWidth;
            t += 1;
        }

        z += stepHeight;
        i += 1;
    }
}

// ==================
// CG_GlassShatter
// Break glass with fancy method
// ==================
pub fn CG_GlassShatter(entnum: i32, dmgPt: &[f32; 3], dmgDir: &[f32; 3], dmgRadius: f32, maxShards: i32) {
    let mut verts: [[f32; 3]; 4] = [[0.0; 3]; 4];
    let mut normal: [f32; 3] = [0.0; 3];

    unsafe {
        if !cgs.inlineDrawModel[cg_entities[entnum as usize].currentState.modelindex as usize].is_null() {
            trap_R_GetBModelVerts(
                cgs.inlineDrawModel[cg_entities[entnum as usize].currentState.modelindex as usize],
                &mut verts,
                &mut normal,
            );
            CG_DoGlass(&verts, &normal, dmgPt, dmgDir, dmgRadius, maxShards);
        }
        //otherwise something awful has happened.
    }
}

// ==================
// CG_GlassShatter_Old
// Throws glass shards from within a given bounding box in the world
// ==================
pub fn CG_GlassShatter_Old(entnum: i32, org: &[f32; 3], mins: &[f32; 3], maxs: &[f32; 3]) {
    let mut velocity: [f32; 3] = [0.0; 3];
    let mut a: [f32; 3] = [0.0; 3];
    let mut shardorg: [f32; 3] = [0.0; 3];
    let mut dif: [f32; 3] = [0.0; 3];
    let mut difx: [f32; 3] = [0.0; 3];
    let mut windowmass: f32;
    let mut shardsthrow: f32 = 0.0;
    let mut chunkname: [u8; 256] = [0; 256];

    trap_S_StartSound(org, entnum, CHAN_BODY, trap_S_RegisterSound("sound/effects/glassbreak1.wav"));

    VectorSubtract(maxs, mins, &mut a);

    windowmass = VectorLength(&a); //should give us some idea of how big the chunk of glass is

    while shardsthrow < windowmass {
        velocity[0] = crandom() * 150.0;
        velocity[1] = crandom() * 150.0;
        velocity[2] = 150.0 + crandom() * 75.0;

        Com_sprintf(
            &mut chunkname,
            chunkname.len(),
            "models/chunks/glass/glchunks_%i.md3",
            Q_irand(1, 6),
        );
        VectorCopy(org, &mut shardorg);

        dif[0] = (maxs[0] - mins[0]) / 2.0;
        dif[1] = (maxs[1] - mins[1]) / 2.0;
        dif[2] = (maxs[2] - mins[2]) / 2.0;

        if dif[0] < 2.0 {
            dif[0] = 2.0;
        }
        if dif[1] < 2.0 {
            dif[1] = 2.0;
        }
        if dif[2] < 2.0 {
            dif[2] = 2.0;
        }

        difx[0] = Q_irand(1, ((dif[0] * 0.9) * 2.0) as i32) as f32;
        difx[1] = Q_irand(1, ((dif[1] * 0.9) * 2.0) as i32) as f32;
        difx[2] = Q_irand(1, ((dif[2] * 0.9) * 2.0) as i32) as f32;

        if difx[0] > dif[0] {
            shardorg[0] += difx[0] - dif[0];
        } else {
            shardorg[0] -= difx[0];
        }
        if difx[1] > dif[1] {
            shardorg[1] += difx[1] - dif[1];
        } else {
            shardorg[1] -= difx[1];
        }
        if difx[2] > dif[2] {
            shardorg[2] += difx[2] - dif[2];
        } else {
            shardorg[2] -= difx[2];
        }

        //CG_TestLine(org, shardorg, 5000, 0x0000ff, 3);

        CG_ThrowChunk(&shardorg, &velocity, trap_R_RegisterModel(&chunkname), 0, 254);

        shardsthrow += 10.0;
    }
}

// ==================
// CG_CreateDebris
// Throws specified debris from within a given bounding box in the world
// ==================
const DEBRIS_SPECIALCASE_ROCK: i32 = -1;
const DEBRIS_SPECIALCASE_CHUNKS: i32 = -2;
const DEBRIS_SPECIALCASE_WOOD: i32 = -3;
const DEBRIS_SPECIALCASE_GLASS: i32 = -4;

const NUM_DEBRIS_MODELS_GLASS: usize = 8;
const NUM_DEBRIS_MODELS_WOOD: usize = 8;
const NUM_DEBRIS_MODELS_CHUNKS: usize = 3;
const NUM_DEBRIS_MODELS_ROCKS: usize = 4; //12

static mut dbModels_Glass: [i32; NUM_DEBRIS_MODELS_GLASS] = [0; NUM_DEBRIS_MODELS_GLASS];
static mut dbModels_Wood: [i32; NUM_DEBRIS_MODELS_WOOD] = [0; NUM_DEBRIS_MODELS_WOOD];
static mut dbModels_Chunks: [i32; NUM_DEBRIS_MODELS_CHUNKS] = [0; NUM_DEBRIS_MODELS_CHUNKS];
static mut dbModels_Rocks: [i32; NUM_DEBRIS_MODELS_ROCKS] = [0; NUM_DEBRIS_MODELS_ROCKS];

pub fn CG_CreateDebris(
    entnum: i32,
    org: &[f32; 3],
    mins: &[f32; 3],
    maxs: &[f32; 3],
    debrissound: i32,
    debrismodel: i32,
) {
    let mut velocity: [f32; 3] = [0.0; 3];
    let mut a: [f32; 3] = [0.0; 3];
    let mut shardorg: [f32; 3] = [0.0; 3];
    let mut dif: [f32; 3] = [0.0; 3];
    let mut difx: [f32; 3] = [0.0; 3];
    let mut windowmass: f32;
    let mut shardsthrow: f32 = 0.0;
    let mut omodel: i32 = debrismodel;
    let mut debrismodel_var: i32 = debrismodel;

    unsafe {
        if omodel == DEBRIS_SPECIALCASE_GLASS && dbModels_Glass[0] == 0 {
            //glass no longer exists, using it for metal.
            dbModels_Glass[0] = trap_R_RegisterModel("models/chunks/metal/metal1_1.md3");
            dbModels_Glass[1] = trap_R_RegisterModel("models/chunks/metal/metal1_2.md3");
            dbModels_Glass[2] = trap_R_RegisterModel("models/chunks/metal/metal1_3.md3");
            dbModels_Glass[3] = trap_R_RegisterModel("models/chunks/metal/metal1_4.md3");
            dbModels_Glass[4] = trap_R_RegisterModel("models/chunks/metal/metal2_1.md3");
            dbModels_Glass[5] = trap_R_RegisterModel("models/chunks/metal/metal2_2.md3");
            dbModels_Glass[6] = trap_R_RegisterModel("models/chunks/metal/metal2_3.md3");
            dbModels_Glass[7] = trap_R_RegisterModel("models/chunks/metal/metal2_4.md3");
        }
        if omodel == DEBRIS_SPECIALCASE_WOOD && dbModels_Wood[0] == 0 {
            dbModels_Wood[0] = trap_R_RegisterModel("models/chunks/crate/crate1_1.md3");
            dbModels_Wood[1] = trap_R_RegisterModel("models/chunks/crate/crate1_2.md3");
            dbModels_Wood[2] = trap_R_RegisterModel("models/chunks/crate/crate1_3.md3");
            dbModels_Wood[3] = trap_R_RegisterModel("models/chunks/crate/crate1_4.md3");
            dbModels_Wood[4] = trap_R_RegisterModel("models/chunks/crate/crate2_1.md3");
            dbModels_Wood[5] = trap_R_RegisterModel("models/chunks/crate/crate2_2.md3");
            dbModels_Wood[6] = trap_R_RegisterModel("models/chunks/crate/crate2_3.md3");
            dbModels_Wood[7] = trap_R_RegisterModel("models/chunks/crate/crate2_4.md3");
        }
        if omodel == DEBRIS_SPECIALCASE_CHUNKS && dbModels_Chunks[0] == 0 {
            dbModels_Chunks[0] = trap_R_RegisterModel("models/chunks/generic/chunks_1.md3");
            dbModels_Chunks[1] = trap_R_RegisterModel("models/chunks/generic/chunks_2.md3");
        }
        if omodel == DEBRIS_SPECIALCASE_ROCK && dbModels_Rocks[0] == 0 {
            dbModels_Rocks[0] = trap_R_RegisterModel("models/chunks/rock/rock1_1.md3");
            dbModels_Rocks[1] = trap_R_RegisterModel("models/chunks/rock/rock1_2.md3");
            dbModels_Rocks[2] = trap_R_RegisterModel("models/chunks/rock/rock1_3.md3");
            dbModels_Rocks[3] = trap_R_RegisterModel("models/chunks/rock/rock1_4.md3");
            /*
            dbModels_Rocks[4] = trap_R_RegisterModel("models/chunks/rock/rock2_1.md3");
            dbModels_Rocks[5] = trap_R_RegisterModel("models/chunks/rock/rock2_2.md3");
            dbModels_Rocks[6] = trap_R_RegisterModel("models/chunks/rock/rock2_3.md3");
            dbModels_Rocks[7] = trap_R_RegisterModel("models/chunks/rock/rock2_4.md3");
            dbModels_Rocks[8] = trap_R_RegisterModel("models/chunks/rock/rock3_1.md3");
            dbModels_Rocks[9] = trap_R_RegisterModel("models/chunks/rock/rock3_2.md3");
            dbModels_Rocks[10] = trap_R_RegisterModel("models/chunks/rock/rock3_3.md3");
            dbModels_Rocks[11] = trap_R_RegisterModel("models/chunks/rock/rock3_4.md3");
            */
        }

        VectorSubtract(maxs, mins, &mut a);

        windowmass = VectorLength(&a); //should give us some idea of how big the chunk of glass is

        while shardsthrow < windowmass {
            velocity[0] = crandom() * 150.0;
            velocity[1] = crandom() * 150.0;
            velocity[2] = 150.0 + crandom() * 75.0;

            if omodel == DEBRIS_SPECIALCASE_GLASS {
                debrismodel_var = dbModels_Glass[Q_irand(0, (NUM_DEBRIS_MODELS_GLASS - 1) as i32) as usize];
            } else if omodel == DEBRIS_SPECIALCASE_WOOD {
                debrismodel_var = dbModels_Wood[Q_irand(0, (NUM_DEBRIS_MODELS_WOOD - 1) as i32) as usize];
            } else if omodel == DEBRIS_SPECIALCASE_CHUNKS {
                debrismodel_var = dbModels_Chunks[Q_irand(0, (NUM_DEBRIS_MODELS_CHUNKS - 1) as i32) as usize];
            } else if omodel == DEBRIS_SPECIALCASE_ROCK {
                debrismodel_var = dbModels_Rocks[Q_irand(0, (NUM_DEBRIS_MODELS_ROCKS - 1) as i32) as usize];
            }

            VectorCopy(org, &mut shardorg);

            dif[0] = (maxs[0] - mins[0]) / 2.0;
            dif[1] = (maxs[1] - mins[1]) / 2.0;
            dif[2] = (maxs[2] - mins[2]) / 2.0;

            if dif[0] < 2.0 {
                dif[0] = 2.0;
            }
            if dif[1] < 2.0 {
                dif[1] = 2.0;
            }
            if dif[2] < 2.0 {
                dif[2] = 2.0;
            }

            difx[0] = Q_irand(1, ((dif[0] * 0.9) * 2.0) as i32) as f32;
            difx[1] = Q_irand(1, ((dif[1] * 0.9) * 2.0) as i32) as f32;
            difx[2] = Q_irand(1, ((dif[2] * 0.9) * 2.0) as i32) as f32;

            if difx[0] > dif[0] {
                shardorg[0] += difx[0] - dif[0];
            } else {
                shardorg[0] -= difx[0];
            }
            if difx[1] > dif[1] {
                shardorg[1] += difx[1] - dif[1];
            } else {
                shardorg[1] -= difx[1];
            }
            if difx[2] > dif[2] {
                shardorg[2] += difx[2] - dif[2];
            } else {
                shardorg[2] -= difx[2];
            }

            //CG_TestLine(org, shardorg, 5000, 0x0000ff, 3);

            CG_ThrowChunk(&shardorg, &velocity, debrismodel_var, debrissound, 0);

            shardsthrow += 10.0;
        }
    }
}

//==========================================================
//SP-style chunks
//==========================================================

// -------------------------
// CG_ExplosionEffects
//
// Used to find the player and shake the camera if close enough
// intensity ranges from 1 (minor tremble) to 16 (major quake)
// -------------------------

pub fn CG_ExplosionEffects(origin: &[f32; 3], intensity: f32, radius: i32, time: i32) {
    //FIXME: When exactly is the vieworg calculated in relation to the rest of the frame's?

    let mut dir: [f32; 3] = [0.0; 3];
    let dist: f32;
    let intensityScale: f32;
    let realIntensity: f32;

    unsafe {
        VectorSubtract(&cg.refdef.vieworg, origin, &mut dir);
        dist = VectorNormalize(&mut dir);

        //Use the dir to add kick to the explosion

        if dist > (radius as f32) {
            return;
        }

        intensityScale = 1.0 - (dist / (radius as f32));
        realIntensity = intensity * intensityScale;

        CGCam_Shake(realIntensity, time);
    }
}

// -------------------------
// CG_MiscModelExplosion
//
// Adds an explosion to a misc model breakables
// -------------------------

pub fn CG_MiscModelExplosion(mins: &[f32; 3], maxs: &[f32; 3], size: i32, chunkType: i32) {
    let mut ct: i32 = 13;
    let mut r: f32;
    let mut org: [f32; 3] = [0.0; 3];
    let mut mid: [f32; 3] = [0.0; 3];
    let mut dir: [f32; 3] = [0.0; 3];
    let mut effect: Option<&'static str> = None;
    let mut effect2: Option<&'static str> = None;
    let mut eID1: i32;
    let mut eID2: i32 = 0;
    let mut i: i32;

    VectorAdd(mins, maxs, &mut mid);
    VectorScale(&mid, 0.5, &mut mid);

    match chunkType {
        MAT_GLASS => {
            effect = Some("chunks/glassbreak");
            ct = 5;
        }
        MAT_GLASS_METAL => {
            effect = Some("chunks/glassbreak");
            effect2 = Some("chunks/metalexplode");
            ct = 5;
        }
        MAT_ELECTRICAL | MAT_ELEC_METAL => {
            effect = Some("chunks/sparkexplode");
            ct = 5;
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_CRATE1 | MAT_CRATE2 => {
            effect = Some("chunks/metalexplode");
            ct = 2;
        }
        MAT_GRATE1 => {
            effect = Some("chunks/grateexplode");
            ct = 8;
        }
        MAT_ROPE => {
            ct = 20;
            effect = Some("chunks/ropebreak");
        }
        MAT_WHITE_METAL | MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_SNOWY_ROCK => {
            //not sure what this crap is really supposed to be..
            match size {
                2 => {
                    effect = Some("chunks/rockbreaklg");
                }
                _ => {
                    effect = Some("chunks/rockbreakmed");
                }
            }
        }
        _ => {}
    }

    if effect.is_none() {
        return;
    }

    ct += 7 * size;

    // FIXME: real precache .. VERify that these need to be here...don't think they would because the effects should be registered in g_breakable
    //rww - No they don't.. indexed effects gameside get precached on load clientside, as server objects are setup before client asset load time.
    //However, we need to index them, so..
    eID1 = trap_FX_RegisterEffect(effect.unwrap());

    if effect2.is_some() && !effect2.unwrap().is_empty() {
        // FIXME: real precache
        eID2 = trap_FX_RegisterEffect(effect2.unwrap());
    }

    // spawn chunk roughly in the bbox of the thing..
    for i in 0..ct {
        let mut j: i32;
        for j in 0..3 {
            r = random() * 0.8 + 0.1;
            org[j as usize] = r * mins[j as usize] + (1.0 - r) * maxs[j as usize];
        }

        // shoot effect away from center
        VectorSubtract(&org, &mid, &mut dir);
        VectorNormalize(&mut dir);

        if effect2.is_some() && !effect2.unwrap().is_empty() && ((rand() & 1) != 0) {
            trap_FX_PlayEffectID(eID2, &org, &dir, -1, -1);
        } else {
            trap_FX_PlayEffectID(eID1, &org, &dir, -1, -1);
        }
    }
}

// -------------------------
// CG_Chunks
//
// Fun chunk spewer
// -------------------------

pub fn CG_Chunks(
    owner: i32,
    origin: &[f32; 3],
    normal: &[f32; 3],
    mins: &[f32; 3],
    maxs: &[f32; 3],
    speed: f32,
    numChunks: i32,
    chunkType: i32,
    customChunk: i32,
    baseScale: f32,
) {
    let le: *mut localEntity_t;
    let re: *mut refEntity_t;
    let mut dir: [f32; 3] = [0.0; 3];
    let mut i: i32;
    let mut j: i32;
    let mut k: i32;
    let mut chunkModel: i32 = 0;
    let mut bounce: i32 = LEBS_NONE;
    let mut r: f32;
    let mut speedMod: f32 = 1.0;
    let mut chunk: bool = false;

    if chunkType == MAT_NONE {
        // Well, we should do nothing
        return;
    }

    // Set up our chunk sound info...breaking sounds are done here so they are done once on breaking..some return instantly because the chunks are done with effects instead of models
    match chunkType {
        MAT_GLASS => {
            unsafe { trap_S_StartSound(std::ptr::null(), owner, CHAN_BODY, cgs.media.glassChunkSound); }
            return;
        }
        MAT_GRATE1 => {
            unsafe { trap_S_StartSound(std::ptr::null(), owner, CHAN_BODY, cgs.media.grateSound); }
            return;
        }
        MAT_ELECTRICAL => {
            // (sparks)
            unsafe {
                trap_S_StartSound(
                    std::ptr::null(),
                    owner,
                    CHAN_BODY,
                    trap_S_RegisterSound(&format!("sound/ambience/spark{}.wav", Q_irand(1, 6))),
                );
            }
            return;
        }
        MAT_DRK_STONE | MAT_LT_STONE | MAT_GREY_STONE | MAT_WHITE_METAL | MAT_SNOWY_ROCK => {
            // not quite sure what this stuff is supposed to be...it's for Stu
            unsafe { trap_S_StartSound(std::ptr::null(), owner, CHAN_BODY, cgs.media.rockBreakSound); }
            bounce = LEBS_ROCK;
            speedMod = 0.5; // rock blows up less
        }
        MAT_GLASS_METAL => {
            unsafe { trap_S_StartSound(std::ptr::null(), owner, CHAN_BODY, cgs.media.glassChunkSound); } // FIXME: should probably have a custom sound
            bounce = LEBS_METAL;
        }
        MAT_CRATE1 | MAT_CRATE2 => {
            unsafe {
                trap_S_StartSound(
                    std::ptr::null(),
                    owner,
                    CHAN_BODY,
                    cgs.media.crateBreakSound[Q_irand(0, 1) as usize],
                );
            }
        }
        MAT_METAL | MAT_METAL2 | MAT_METAL3 | MAT_ELEC_METAL => {
            // FIXME: maybe have its own sound?
            unsafe { trap_S_StartSound(std::ptr::null(), owner, CHAN_BODY, cgs.media.chunkSound); }
            bounce = LEBS_METAL;
            speedMod = 0.8; // metal blows up a bit more
        }
        MAT_ROPE => {
            //		trap_S_StartSound( NULL, owner, CHAN_BODY, cgi_S_RegisterSound( "" ));  FIXME:  needs a sound
            return;
        }
        _ => {}
    }

    let mut baseScale_var: f32 = baseScale;
    if baseScale_var <= 0.0 {
        baseScale_var = 1.0;
    }

    // Chunks
    for i in 0..numChunks {
        let mut chunk_var: bool = chunk;
        let mut chunkModel_var: i32 = chunkModel;

        if customChunk > 0 {
            // Try to use a custom chunk.
            unsafe {
                if !cgs.gameModels[customChunk as usize].is_null() {
                    chunk_var = true;
                    chunkModel_var = cgs.gameModels[customChunk as usize];
                }
            }
        }

        if !chunk_var {
            // No custom chunk.  Pick a random chunk type at run-time so we don't get the same chunks
            match chunkType {
                MAT_METAL2 => {
                    //bluegrey
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_METAL2 as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_GREY_STONE => {
                    //gray
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_ROCK1 as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_LT_STONE => {
                    //tan
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_ROCK2 as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_DRK_STONE => {
                    //brown
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_ROCK3 as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_SNOWY_ROCK => {
                    //gray & brown
                    if Q_irand(0, 1) != 0 {
                        unsafe {
                            chunkModel_var = cgs.media.chunkModels[CHUNK_ROCK1 as usize][Q_irand(0, 3) as usize];
                        }
                    } else {
                        unsafe {
                            chunkModel_var = cgs.media.chunkModels[CHUNK_ROCK3 as usize][Q_irand(0, 3) as usize];
                        }
                    }
                }
                MAT_WHITE_METAL => {
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_WHITE_METAL as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_CRATE1 => {
                    //yellow multi-colored crate chunks
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_CRATE1 as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_CRATE2 => {
                    //red multi-colored crate chunks
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_CRATE2 as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_ELEC_METAL | MAT_GLASS_METAL | MAT_METAL => {
                    //grey
                    unsafe {
                        chunkModel_var = cgs.media.chunkModels[CHUNK_METAL1 as usize][Q_irand(0, 3) as usize];
                    }
                }
                MAT_METAL3 => {
                    if (rand() & 1) != 0 {
                        unsafe {
                            chunkModel_var = cgs.media.chunkModels[CHUNK_METAL1 as usize][Q_irand(0, 3) as usize];
                        }
                    } else {
                        unsafe {
                            chunkModel_var = cgs.media.chunkModels[CHUNK_METAL2 as usize][Q_irand(0, 3) as usize];
                        }
                    }
                }
                _ => {}
            }
        }

        // It wouldn't look good to throw a bunch of RGB axis models...so make sure we have something to work with.
        if chunkModel_var != 0 {
            le = CG_AllocLocalEntity();
            unsafe {
                re = &mut (*le).refEntity;

                (*re).hModel = chunkModel_var;
                (*le).leType = LE_FRAGMENT;
                (*le).endTime = cg.time + 1300 + (random() * 900.0) as i32;

                // spawn chunk roughly in the bbox of the thing...bias towards center in case thing blowing up doesn't complete fill its bbox.
                for j in 0..3 {
                    r = random() * 0.8 + 0.1;
                    (*re).origin[j as usize] = r * mins[j as usize] + (1.0 - r) * maxs[j as usize];
                }
                VectorCopy(&(*re).origin, &mut (*le).pos.trBase);

                // Move out from center of thing, otherwise you can end up things moving across the brush in an undesirable direction.  Visually looks wrong
                VectorSubtract(&(*re).origin, origin, &mut dir);
                VectorNormalize(&mut dir);
                VectorScale(
                    &dir,
                    flrand(speed * 0.5, speed * 1.25) * speedMod,
                    &mut (*le).pos.trDelta,
                );

                // Angular Velocity
                VectorSet(
                    &mut (*le).angles.trBase,
                    random() * 360.0,
                    random() * 360.0,
                    random() * 360.0,
                );

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
                //(*le).ownerGentNum = owner;
                (*le).leBounceSoundType = bounce;

                // Make sure that we have the desired start size set
                (*le).radius = flrand(baseScale_var * 0.75, baseScale_var * 1.25);
                (*re).nonNormalizedAxes = true;
                AxisCopy(&axisDefault, &mut (*re).axis); // could do an angles to axis, but this is cheaper and works ok
                for k in 0..3 {
                    (*re).modelScale[k as usize] = (*le).radius;
                }
                ScaleModelAxis(re);
                /*
                for( k = 0; k < 3; k++ )
                {
                    VectorScale( re->axis[k], le->radius, re->axis[k] );
                }
                */
            }
        }
    }
}

// ==================
// CG_ScorePlum
// ==================
pub fn CG_ScorePlum(client: i32, org: &[f32; 3], score: i32) {
    let le: *mut localEntity_t;
    let re: *mut refEntity_t;
    let mut angles: [f32; 3] = [0.0; 3];
    static mut lastPos: [f32; 3] = [0.0; 3];

    // only visualize for the client that scored
    unsafe {
        if client != cg.predictedPlayerState.clientNum || cg_scorePlum.integer == 0 {
            return;
        }
    }

    le = CG_AllocLocalEntity();
    unsafe {
        (*le).leFlags = 0;
        (*le).leType = LE_SCOREPLUM;
        (*le).startTime = cg.time;
        (*le).endTime = cg.time + 4000;
        (*le).lifeRate = 1.0 / (((*le).endTime - (*le).startTime) as f32);

        (*le).color[0] = 1.0;
        (*le).color[1] = 1.0;
        (*le).color[2] = 1.0;
        (*le).color[3] = 1.0;
        (*le).radius = score as f32;

        VectorCopy(org, &mut (*le).pos.trBase);
        if org[2] >= lastPos[2] - 20.0 && org[2] <= lastPos[2] + 20.0 {
            (*le).pos.trBase[2] -= 20.0;
        }

        //CG_Printf( "Plum origin %i %i %i -- %i\n", (int)org[0], (int)org[1], (int)org[2], (int)Distance(org, lastPos));
        VectorCopy(org, &mut lastPos);

        re = &mut (*le).refEntity;

        (*re).reType = RT_SPRITE;
        (*re).radius = 16.0;

        VectorClear(&mut angles);
        AnglesToAxis(&angles, &mut (*re).axis);
    }
}

// ====================
// CG_MakeExplosion
// ====================
pub fn CG_MakeExplosion(
    origin: &[f32; 3],
    dir: *const [f32; 3],
    hModel: i32,
    numFrames: i32,
    shader: i32,
    msec: i32,
    isSprite: bool,
    scale: f32,
    flags: i32,
) -> *mut localEntity_t {
    let mut ang: f32 = 0.0;
    let ex: *mut localEntity_t;
    let mut offset: i32;
    let mut tmpVec: [f32; 3] = [0.0; 3];
    let mut newOrigin: [f32; 3] = [0.0; 3];

    if msec <= 0 {
        CG_Error("CG_MakeExplosion: msec = %i", msec);
    }

    // skew the time a bit so they aren't all in sync
    offset = (rand() & 63) as i32;

    ex = CG_AllocLocalEntity();
    unsafe {
        if isSprite {
            (*ex).leType = LE_SPRITE_EXPLOSION;
            (*ex).refEntity.rotation = (rand() % 360) as f32;
            (*ex).radius = scale;
            if !dir.is_null() {
                VectorScale(&*dir, 16.0, &mut tmpVec);
                VectorAdd(&tmpVec, origin, &mut newOrigin);
            } else {
                VectorCopy(origin, &mut newOrigin);
            }
        } else {
            (*ex).leType = LE_EXPLOSION;
            VectorCopy(origin, &mut newOrigin);

            // set axis with random rotate when necessary
            if dir.is_null() {
                AxisClear(&mut (*ex).refEntity.axis);
            } else {
                if (flags & LEF_NO_RANDOM_ROTATE) == 0 {
                    ang = (rand() % 360) as f32;
                }
                VectorCopy(&*dir, &mut (*ex).refEntity.axis[0]);
                RotateAroundDirection(&mut (*ex).refEntity.axis, ang);
            }
        }

        (*ex).startTime = cg.time - offset;
        (*ex).endTime = (*ex).startTime + msec;

        // bias the time so all shader effects start correctly
        (*ex).refEntity.shaderTime = (*ex).startTime as f32 / 1000.0;

        (*ex).refEntity.hModel = hModel;
        (*ex).refEntity.customShader = shader;
        (*ex).lifeRate = (numFrames as f32) / (msec as f32);
        (*ex).leFlags = flags;

        //Scale the explosion
        if scale != 1.0 {
            (*ex).refEntity.nonNormalizedAxes = true;

            VectorScale(&(*ex).refEntity.axis[0], scale, &mut (*ex).refEntity.axis[0]);
            VectorScale(&(*ex).refEntity.axis[1], scale, &mut (*ex).refEntity.axis[1]);
            VectorScale(&(*ex).refEntity.axis[2], scale, &mut (*ex).refEntity.axis[2]);
        }
        // set origin
        VectorCopy(&newOrigin, &mut (*ex).refEntity.origin);
        VectorCopy(&newOrigin, &mut (*ex).refEntity.oldorigin);

        (*ex).color[0] = 1.0;
        (*ex).color[1] = 1.0;
        (*ex).color[2] = 1.0;
    }

    ex
}

// -------------------------
// CG_SurfaceExplosion
//
// Adds an explosion to a surface
// -------------------------

const NUM_SPARKS: i32 = 12;
const NUM_PUFFS: i32 = 1;
const NUM_EXPLOSIONS: i32 = 4;

pub fn CG_SurfaceExplosion(origin: &[f32; 3], normal: &[f32; 3], radius: f32, shake_speed: f32, smoke: bool) {
    let le: *mut localEntity_t;
    //FXTrail			*particle;
    let mut direction: [f32; 3] = [0.0; 3];
    let mut new_org: [f32; 3] = [0.0; 3];
    let mut velocity: [f32; 3] = [0.0, 0.0, 0.0];
    let mut temp_org: [f32; 3] = [0.0; 3];
    let mut temp_vel: [f32; 3] = [0.0; 3];
    let mut scale: f32;
    let mut dscale: f32;
    let mut i: i32;
    let mut numSparks: i32;

    //Sparks
    numSparks = (16.0 + (random() * 16.0)) as i32;

    for i in 0..numSparks {
        scale = 0.25 + (random() * 2.0);
        dscale = -scale * 0.5;

        /*		particle = FX_AddTrail( origin,
                                NULL,
                                NULL,
                                32.0f,
                                -64.0f,
                                scale,
                                -scale,
                                1.0f,
                                0.0f,
                                0.25f,
                                4000.0f,
                                cgs.media.sparkShader,
                                rand() & FXF_BOUNCE);
        if ( particle == NULL )
            return;

        FXE_Spray( normal, 500, 150, 1.0f, 768 + (rand() & 255), (FXPrimitive *) particle );*/
    }

    //Smoke
    //Move this out a little from the impact surface
    unsafe {
        VectorMA(origin, 4.0, normal, &mut new_org);
        VectorSet(&mut velocity, 0.0, 0.0, 16.0);

        for i in 0..4 {
            VectorSet(
                &mut temp_org,
                new_org[0] + (crandom() * 16.0),
                new_org[1] + (crandom() * 16.0),
                new_org[2] + (random() * 4.0),
            );
            VectorSet(
                &mut temp_vel,
                velocity[0] + (crandom() * 8.0),
                velocity[1] + (crandom() * 8.0),
                velocity[2] + (crandom() * 8.0),
            );

            /*		FX_AddSprite(	temp_org,
                                temp_vel,
                                NULL,
                                64.0f + (random() * 32.0f),
                                16.0f,
                                1.0f,
                                0.0f,
                                20.0f + (crandom() * 90.0f),
                                0.5f,
                                1500.0f,
                                cgs.media.smokeShader, FXF_USE_ALPHA_CHAN );*/
        }

        //Core of the explosion

        //Orient the explosions to face the camera
        VectorSubtract(&cg.refdef.vieworg, origin, &mut direction);
        VectorNormalize(&mut direction);

        //Tag the last one with a light
        le = CG_MakeExplosion(
            origin,
            &direction,
            cgs.media.explosionModel,
            6,
            cgs.media.surfaceExplosionShader,
            500,
            false,
            radius * 0.02 + (random() * 0.3),
            0,
        );
        (*le).light = 150;
        VectorSet(&mut (*le).lightColor, 0.9, 0.8, 0.5);

        for i in 0..(NUM_EXPLOSIONS - 1) {
            VectorSet(
                &mut new_org,
                origin[0] + ((16.0 + (crandom() * 8.0)) * crandom()),
                origin[1] + ((16.0 + (crandom() * 8.0)) * crandom()),
                origin[2] + ((16.0 + (crandom() * 8.0)) * crandom()),
            );
            le = CG_MakeExplosion(
                &new_org,
                &direction,
                cgs.media.explosionModel,
                6,
                cgs.media.surfaceExplosionShader,
                300 + (rand() & 99) as i32,
                false,
                radius * 0.05 + (crandom() * 0.3),
                0,
            );
        }

        //Shake the camera
        CG_ExplosionEffects(origin, shake_speed, 350, 750);

        // The level designers wanted to be able to turn the smoke spawners off.  The rationale is that they
        //	want to blow up catwalks and such that fall down...when that happens, it shouldn't really leave a mark
        //	and a smoke spewer at the explosion point...
        if smoke {
            VectorMA(origin, -8.0, normal, &mut temp_org);
            //		FX_AddSpawner( temp_org, normal, NULL, NULL, 100, random()*25.0f, 5000.0f, (void *) CG_SmokeSpawn );

            //Impact mark
            //FIXME: Replace mark
            //CG_ImpactMark( cgs.media.burnMarkShader, origin, normal, random()*360, 1,1,1,1, qfalse, 8, qfalse );
        }
    }
}

// =================
// CG_Bleed
//
// This is the spurt of blood when a character gets hit
// =================
pub fn CG_Bleed(origin: &[f32; 3], entityNum: i32) {
    let ex: *mut localEntity_t;

    unsafe {
        if cg_blood.integer == 0 {
            return;
        }

        ex = CG_AllocLocalEntity();
        (*ex).leType = LE_EXPLOSION;

        (*ex).startTime = cg.time;
        (*ex).endTime = (*ex).startTime + 500;

        VectorCopy(origin, &mut (*ex).refEntity.origin);
        (*ex).refEntity.reType = RT_SPRITE;
        (*ex).refEntity.rotation = (rand() % 360) as f32;
        (*ex).refEntity.radius = 24.0;

        (*ex).refEntity.customShader = 0; //cgs.media.bloodExplosionShader;

        // don't show player's own blood in view
        if entityNum == cg.snap.as_ref().unwrap().ps.clientNum {
            (*ex).refEntity.renderfx |= RF_THIRD_PERSON;
        }
    }
}

// ==================
// CG_LaunchGib
// ==================
pub fn CG_LaunchGib(origin: &[f32; 3], velocity: &[f32; 3], hModel: i32) {
    let le: *mut localEntity_t;
    let re: *mut refEntity_t;

    le = CG_AllocLocalEntity();
    unsafe {
        re = &mut (*le).refEntity;

        (*le).leType = LE_FRAGMENT;
        (*le).startTime = cg.time;
        (*le).endTime = (*le).startTime + 5000 + (random() * 3000.0) as i32;

        VectorCopy(origin, &mut (*re).origin);
        AxisCopy(&axisDefault, &mut (*re).axis);
        (*re).hModel = hModel;

        (*le).pos.trType = TR_GRAVITY;
        VectorCopy(origin, &mut (*le).pos.trBase);
        VectorCopy(velocity, &mut (*le).pos.trDelta);
        (*le).pos.trTime = cg.time;

        (*le).bounceFactor = 0.6;

        (*le).leBounceSoundType = LEBS_BLOOD;
        (*le).leMarkType = LEMT_BLOOD;
    }
}

// Stub declarations for types and externs that should be imported from cg_local module
// These are placeholders indicating dependencies from cg_local.h
pub struct localEntity_t {
    // TODO: define fields
}

pub struct refEntity_t {
    // TODO: define fields
}

pub struct addpolyArgStruct_t {
    // TODO: define fields
}

extern "C" {
    pub static mut cg: CG_t;
    pub static mut cgs: CGameStatic_t;
    pub static mut cg_entities: [centity_t; MAX_GENTITIES];
    pub static mut cg_noProjectileTrail: cvar_t;
    pub static mut cg_scorePlum: cvar_t;
    pub static mut cg_blood: cvar_t;

    pub fn CG_AllocLocalEntity() -> *mut localEntity_t;
    pub fn CG_Error(msg: &str, ...) -> !;
    pub fn VectorCopy(a: &[f32; 3], b: &mut [f32; 3]);
    pub fn VectorSubtract(a: &[f32; 3], b: &[f32; 3], c: &mut [f32; 3]);
    pub fn VectorNormalize(v: &mut [f32; 3]) -> f32;
    pub fn VectorMA(v1: &[f32; 3], scale: f32, v2: &[f32; 3], v3: &mut [f32; 3]);
    pub fn VectorScale(v: &[f32; 3], scale: f32, out: &mut [f32; 3]);
    pub fn VectorAdd(a: &[f32; 3], b: &[f32; 3], c: &mut [f32; 3]);
    pub fn VectorSet(v: &mut [f32; 3], x: f32, y: f32, z: f32);
    pub fn VectorClear(v: &mut [f32; 3]);
    pub fn CrossProduct(v1: &[f32; 3], v2: &[f32; 3], cross: &mut [f32; 3]);
    pub fn DistanceSquared(p1: &[f32; 3], p2: &[f32; 3]) -> f32;
    pub fn VectorLength(v: &[f32; 3]) -> f32;
    pub fn AxisCopy(a: &[[f32; 3]; 3], b: &mut [[f32; 3]; 3]);
    pub fn AxisClear(axis: &mut [[f32; 3]; 3]);
    pub fn AnglesToAxis(angles: &[f32; 3], axis: &mut [[f32; 3]; 3]);
    pub fn RotateAroundDirection(axis: &mut [[f32; 3]; 3], degrees: f32);
    pub fn ScaleModelAxis(re: *mut refEntity_t);
    pub fn rand() -> i32;
    pub fn random() -> f32;
    pub fn crandom() -> f32;
    pub fn Q_random(seed: &mut i32) -> f32;
    pub fn Q_irand(min: i32, max: i32) -> i32;
    pub fn Com_sprintf(str_: &mut [u8], size: usize, fmt: &str, ...);
    pub fn trap_S_StartSound(origin: *const [f32; 3], entitynum: i32, entchannel: i32, sfxHandle: i32);
    pub fn trap_S_RegisterSound(name: &str) -> i32;
    pub fn trap_R_RegisterModel(name: &str) -> i32;
    pub fn trap_R_RegisterShaderNoMip(name: &str) -> i32;
    pub fn trap_R_GetBModelVerts(
        hModel: i32,
        verts: &mut [[f32; 3]; 4],
        normals: &mut [f32; 3],
    );
    pub fn trap_FX_AddPoly(args: *const addpolyArgStruct_t);
    pub fn trap_FX_RegisterEffect(effect: &str) -> i32;
    pub fn trap_FX_PlayEffectID(fxHandle: i32, org: &[f32; 3], dir: &[f32; 3], iLoopTime: i32, iTremorTime: i32);
    pub fn CGCam_Shake(intensity: f32, duration: i32);
    pub fn flrand(min: f32, max: f32) -> f32;
}

// Stub types and constants
pub struct CG_t;
pub struct CGameStatic_t;
pub struct centity_t;
pub struct cvar_t {
    pub integer: i32,
}

const MAX_GENTITIES: usize = 1024;
const LEF_PUFF_DONT_SCALE: i32 = 1;
const LEF_TUMBLE: i32 = 2;
const LEF_NO_RANDOM_ROTATE: i32 = 4;
const LE_MOVE_SCALE_FADE: i32 = 1;
const LE_SPRITE_EXPLOSION: i32 = 2;
const LE_EXPLOSION: i32 = 3;
const LE_FRAGMENT: i32 = 4;
const LE_LINE: i32 = 5;
const LE_SCOREPLUM: i32 = 6;
const TR_LINEAR: i32 = 1;
const TR_GRAVITY: i32 = 2;
const RT_SPRITE: i32 = 1;
const RT_LINE: i32 = 2;
const CHAN_AUTO: i32 = 0;
const CHAN_BODY: i32 = 1;
const LEBS_NONE: i32 = 0;
const LEBS_ROCK: i32 = 1;
const LEBS_METAL: i32 = 2;
const LEBS_BLOOD: i32 = 3;
const LEMT_BLOOD: i32 = 1;
const YAW: usize = 1;
const SABER_RED: i32 = 0;
const SABER_ORANGE: i32 = 1;
const SABER_YELLOW: i32 = 2;
const SABER_GREEN: i32 = 3;
const SABER_BLUE: i32 = 4;
const SABER_PURPLE: i32 = 5;
const MAT_NONE: i32 = 0;
const MAT_GLASS: i32 = 1;
const MAT_GLASS_METAL: i32 = 2;
const MAT_ELECTRICAL: i32 = 3;
const MAT_ELEC_METAL: i32 = 4;
const MAT_METAL: i32 = 5;
const MAT_METAL2: i32 = 6;
const MAT_METAL3: i32 = 7;
const MAT_CRATE1: i32 = 8;
const MAT_CRATE2: i32 = 9;
const MAT_GRATE1: i32 = 10;
const MAT_ROPE: i32 = 11;
const MAT_WHITE_METAL: i32 = 12;
const MAT_DRK_STONE: i32 = 13;
const MAT_LT_STONE: i32 = 14;
const MAT_GREY_STONE: i32 = 15;
const MAT_SNOWY_ROCK: i32 = 16;
const CHUNK_METAL1: i32 = 0;
const CHUNK_METAL2: i32 = 1;
const CHUNK_ROCK1: i32 = 2;
const CHUNK_ROCK2: i32 = 3;
const CHUNK_ROCK3: i32 = 4;
const CHUNK_WHITE_METAL: i32 = 5;
const CHUNK_CRATE1: i32 = 6;
const CHUNK_CRATE2: i32 = 7;
const RF_THIRD_PERSON: i32 = 1;

static mut axisDefault: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
