// Disruptor Weapon

use super::cg_local::*;
use super::fx_local::*;

/*
---------------------------
FX_DisruptorMainShot
---------------------------
*/
static WHITE: [f32; 3] = [1.0f32, 1.0f32, 1.0f32];

pub fn FX_DisruptorMainShot(start: [f32; 3], end: [f32; 3]) {
    //	vec3_t	dir;
    //	float	len;

    trap_FX_AddLine(
        start,
        end,
        0.1f32,
        6.0f32,
        0.0f32,
        1.0f32,
        0.0f32,
        0.0f32,
        WHITE,
        WHITE,
        0.0f32,
        150,
        trap_R_RegisterShader(b"gfx/effects/redLine\0".as_ptr() as *const i8),
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
    );

    //	VectorSubtract( end, start, dir );
    //	len = VectorNormalize( dir );

    //	FX_AddCylinder( start, dir, 5.0f, 5.0f, 0.0f,
    //								5.0f, 5.0f, 0.0f,
    //								len, len, 0.0f,
    //								1.0f, 1.0f, 0.0f,
    //								WHITE, WHITE, 0.0f,
    //								400, cgi_R_RegisterShader( "gfx/effects/spiral" ), 0 );
}

/*
---------------------------
FX_DisruptorAltShot
---------------------------
*/
pub fn FX_DisruptorAltShot(start: [f32; 3], end: [f32; 3], fullCharge: i32) {
    trap_FX_AddLine(
        start,
        end,
        0.1f32,
        10.0f32,
        0.0f32,
        1.0f32,
        0.0f32,
        0.0f32,
        WHITE,
        WHITE,
        0.0f32,
        175,
        trap_R_RegisterShader(b"gfx/effects/redLine\0".as_ptr() as *const i8),
        FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
    );

    if fullCharge != 0 {
        let YELLER: [f32; 3] = [0.8f32, 0.7f32, 0.0f32];

        // add some beef
        trap_FX_AddLine(
            start,
            end,
            0.1f32,
            7.0f32,
            0.0f32,
            1.0f32,
            0.0f32,
            0.0f32,
            YELLER,
            YELLER,
            0.0f32,
            150,
            trap_R_RegisterShader(b"gfx/misc/whiteline2\0".as_ptr() as *const i8),
            FX_SIZE_LINEAR | FX_ALPHA_LINEAR,
        );
    }
}

/*
---------------------------
FX_DisruptorAltMiss
---------------------------
*/
#[allow(non_upper_case_globals)]
const FX_ALPHA_WAVE: i32 = 0x00000008;

pub fn FX_DisruptorAltMiss(origin: [f32; 3], normal: [f32; 3]) {
    let mut pos: [f32; 3] = [0.0f32; 3];
    let mut c1: [f32; 3] = [0.0f32; 3];
    let mut c2: [f32; 3] = [0.0f32; 3];
    let mut b: addbezierArgStruct_t = Default::default();

    VectorMA(origin, 4.0f32, normal, &mut c1);
    VectorCopy(c1, &mut c2);
    c1[2] += 4.0f32;
    c2[2] += 12.0f32;

    VectorAdd(origin, normal, &mut pos);
    pos[2] += 28.0f32;

    /*
    FX_AddBezier( origin, pos, c1, vec3_origin, c2, vec3_origin, 6.0f, 6.0f, 0.0f, 0.0f, 0.2f, 0.5f,
    WHITE, WHITE, 0.0f, 4000, trap_R_RegisterShader( "gfx/effects/smokeTrail" ), FX_ALPHA_WAVE );
    */

    VectorCopy(origin, &mut b.start);
    VectorCopy(pos, &mut b.end);
    VectorCopy(c1, &mut b.control1);
    VectorCopy(vec3_origin, &mut b.control1Vel);
    VectorCopy(c2, &mut b.control2);
    VectorCopy(vec3_origin, &mut b.control2Vel);

    b.size1 = 6.0f32;
    b.size2 = 6.0f32;
    b.sizeParm = 0.0f32;
    b.alpha1 = 0.0f32;
    b.alpha2 = 0.2f32;
    b.alphaParm = 0.5f32;

    VectorCopy(WHITE, &mut b.sRGB);
    VectorCopy(WHITE, &mut b.eRGB);

    b.rgbParm = 0.0f32;
    b.killTime = 4000;
    b.shader = trap_R_RegisterShader(b"gfx/effects/smokeTrail\0".as_ptr() as *const i8);
    b.flags = FX_ALPHA_WAVE;

    trap_FX_AddBezier(&mut b);

    trap_FX_PlayEffectID(cgs.effects.disruptorAltMissEffect, origin, normal, -1, -1);
}

/*
---------------------------
FX_DisruptorAltHit
---------------------------
*/

pub fn FX_DisruptorAltHit(origin: [f32; 3], normal: [f32; 3]) {
    trap_FX_PlayEffectID(cgs.effects.disruptorAltHitEffect, origin, normal, -1, -1);
}

/*
---------------------------
FX_DisruptorHitWall
---------------------------
*/

pub fn FX_DisruptorHitWall(origin: [f32; 3], normal: [f32; 3]) {
    trap_FX_PlayEffectID(cgs.effects.disruptorWallImpactEffect, origin, normal, -1, -1);
}

/*
---------------------------
FX_DisruptorHitPlayer
---------------------------
*/

pub fn FX_DisruptorHitPlayer(origin: [f32; 3], normal: [f32; 3], humanoid: i32) {
    trap_FX_PlayEffectID(cgs.effects.disruptorFleshImpactEffect, origin, normal, -1, -1);
}
