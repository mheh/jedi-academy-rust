// tr_sky.c

// leave this as first line for PCH reasons...
//

use core::ffi::c_int;

const SKY_SUBDIVISIONS: usize = 8;
const HALF_SKY_SUBDIVISIONS: usize = SKY_SUBDIVISIONS / 2;

static mut s_cloudTexCoords: [[[[[f32; 2]; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1]; 6] =
    [[[[0.0; 2]; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1]; 6];
static mut s_cloudTexP: [[[[f32; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1]; 6] =
    [[[0.0; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1]; 6];

/*
===================================================================================

POLYGON TO BOX SIDE PROJECTION

===================================================================================
*/

static SKY_CLIP: [[f32; 3]; 6] = [
    [1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, 1.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
];

static mut sky_mins: [[f32; 6]; 2] = [[0.0; 6]; 2];
static mut sky_maxs: [[f32; 6]; 2] = [[0.0; 6]; 2];
static mut sky_min: f32 = 0.0;
static mut sky_max: f32 = 0.0;

/*
================
AddSkyPolygon
================
*/
unsafe fn AddSkyPolygon(nump: c_int, mut vecs: *const f32) {
    let nump = nump as usize;
    let mut i: usize;
    let mut j: usize;
    let mut v: [f32; 3];
    let mut av: [f32; 3];
    let mut s: f32;
    let mut t: f32;
    let mut dv: f32;
    let mut axis: usize;
    let mut vp: *const f32;

    // s = [0]/[2], t = [1]/[2]
    static VEC_TO_ST: [[i32; 3]; 6] = [
        [-2, 3, 1],
        [2, 3, -1],

        [1, 3, 2],
        [-1, 3, -2],

        [-2, -1, 3],
        [-2, 1, -3],

    //	[-1, 2, 3],
    //	[1, 2, -3]
    ];

    // decide which face it maps to
    v = [0.0, 0.0, 0.0];
    vp = vecs;
    i = 0;
    while i < nump {
        v[0] += *vp;
        v[1] += *vp.add(1);
        v[2] += *vp.add(2);
        i += 1;
        vp = vp.add(3);
    }

    av[0] = v[0].abs();
    av[1] = v[1].abs();
    av[2] = v[2].abs();

    if av[0] > av[1] && av[0] > av[2] {
        if v[0] < 0.0 {
            axis = 1;
        } else {
            axis = 0;
        }
    } else if av[1] > av[2] && av[1] > av[0] {
        if v[1] < 0.0 {
            axis = 3;
        } else {
            axis = 2;
        }
    } else {
        if v[2] < 0.0 {
            axis = 5;
        } else {
            axis = 4;
        }
    }

    // project new texture coords
    i = 0;
    while i < nump {
        j = (VEC_TO_ST[axis][2]) as usize;
        if VEC_TO_ST[axis][2] > 0 {
            dv = *vecs.add(j - 1);
        } else {
            dv = -*vecs.add(((-VEC_TO_ST[axis][2]) - 1) as usize);
        }
        if dv < 0.001 {
            i += 1;
            vecs = vecs.add(3);
            continue;	// don't divide by zero
        }

        j = (VEC_TO_ST[axis][0]) as usize;
        if VEC_TO_ST[axis][0] < 0 {
            s = -*vecs.add(((-VEC_TO_ST[axis][0]) - 1) as usize) / dv;
        } else {
            s = *vecs.add(j - 1) / dv;
        }

        j = (VEC_TO_ST[axis][1]) as usize;
        if VEC_TO_ST[axis][1] < 0 {
            t = -*vecs.add(((-VEC_TO_ST[axis][1]) - 1) as usize) / dv;
        } else {
            t = *vecs.add(j - 1) / dv;
        }

        if s < sky_mins[0][axis] {
            sky_mins[0][axis] = s;
        }
        if t < sky_mins[1][axis] {
            sky_mins[1][axis] = t;
        }
        if s > sky_maxs[0][axis] {
            sky_maxs[0][axis] = s;
        }
        if t > sky_maxs[1][axis] {
            sky_maxs[1][axis] = t;
        }

        i += 1;
        vecs = vecs.add(3);
    }
}

const ON_EPSILON: f32 = 0.1;			// point on plane side epsilon
const MAX_CLIP_VERTS: usize = 64;

/*
================
ClipSkyPolygon
================
*/
unsafe fn ClipSkyPolygon(nump: c_int, vecs: *const f32, stage: c_int) {
    let nump = nump as usize;
    let stage = stage as usize;
    let mut norm: *const f32;
    let mut v: *const f32;
    let mut front: bool;
    let mut back: bool;
    let mut d: f32;
    let mut e: f32;
    let mut dists: [f32; MAX_CLIP_VERTS];
    let mut sides: [i32; MAX_CLIP_VERTS];
    let mut newv: [[[f32; 3]; MAX_CLIP_VERTS]; 2];
    let mut newc: [usize; 2];
    let mut i: usize;
    let mut j: usize;

    if nump > MAX_CLIP_VERTS - 2 {
        com_error(ERR_DROP, "ClipSkyPolygon: MAX_CLIP_VERTS\0");
    }
    if stage == 6 {
        // fully clipped, so draw it
        AddSkyPolygon(nump as c_int, vecs);
        return;
    }

    front = false;
    back = false;
    norm = core::ptr::addr_of!(SKY_CLIP[stage][0]);
    v = vecs;
    i = 0;
    while i < nump {
        d = dot_product(v, norm);
        if d > ON_EPSILON {
            front = true;
            sides[i] = SIDE_FRONT;
        } else if d < -ON_EPSILON {
            back = true;
            sides[i] = SIDE_BACK;
        } else {
            sides[i] = SIDE_ON;
        }
        dists[i] = d;
        i += 1;
        v = v.add(3);
    }

    if !front || !back {
        // not clipped
        ClipSkyPolygon(nump as c_int, vecs, (stage + 1) as c_int);
        return;
    }

    // clip it
    sides[i] = sides[0];
    dists[i] = dists[0];
    core::ptr::copy_nonoverlapping(vecs, vecs.add(i * 3) as *mut _, 3);
    newc[0] = 0;
    newc[1] = 0;

    v = vecs;
    i = 0;
    while i < nump {
        match sides[i] {
            SIDE_FRONT => {
                newv[0][newc[0]][0] = *v;
                newv[0][newc[0]][1] = *v.add(1);
                newv[0][newc[0]][2] = *v.add(2);
                newc[0] += 1;
            }
            SIDE_BACK => {
                newv[1][newc[1]][0] = *v;
                newv[1][newc[1]][1] = *v.add(1);
                newv[1][newc[1]][2] = *v.add(2);
                newc[1] += 1;
            }
            SIDE_ON => {
                newv[0][newc[0]][0] = *v;
                newv[0][newc[0]][1] = *v.add(1);
                newv[0][newc[0]][2] = *v.add(2);
                newc[0] += 1;
                newv[1][newc[1]][0] = *v;
                newv[1][newc[1]][1] = *v.add(1);
                newv[1][newc[1]][2] = *v.add(2);
                newc[1] += 1;
            }
            _ => {}
        }

        if sides[i] == SIDE_ON || sides[i + 1] == SIDE_ON || sides[i + 1] == sides[i] {
            i += 1;
            v = v.add(3);
            continue;
        }

        d = dists[i] / (dists[i] - dists[i + 1]);
        j = 0;
        while j < 3 {
            e = *v.add(j) + d * (*v.add(j + 3) - *v.add(j));
            newv[0][newc[0]][j] = e;
            newv[1][newc[1]][j] = e;
            j += 1;
        }
        newc[0] += 1;
        newc[1] += 1;

        i += 1;
        v = v.add(3);
    }

    // continue
    ClipSkyPolygon(newc[0] as c_int, core::ptr::addr_of!(newv[0][0][0]), (stage + 1) as c_int);
    ClipSkyPolygon(newc[1] as c_int, core::ptr::addr_of!(newv[1][0][0]), (stage + 1) as c_int);
}

/*
==============
ClearSkyBox
==============
*/
unsafe fn ClearSkyBox() {
    let mut i: usize;

    i = 0;
    while i < 6 {
        sky_mins[0][i] = MAX_WORLD_COORD;	//9999;
        sky_mins[1][i] = MAX_WORLD_COORD;
        sky_maxs[0][i] = MIN_WORLD_COORD;	//-9999;
        sky_maxs[1][i] = MIN_WORLD_COORD;
        i += 1;
    }
}

/*
================
RB_ClipSkyPolygons
================
*/
pub unsafe fn RB_ClipSkyPolygons(input: *mut shaderCommands_t) {
    let mut p: [[f32; 3]; 5] = [[0.0; 3]; 5];	// need one extra point for clipping
    let mut i: usize;
    let mut j: usize;

    ClearSkyBox();

    i = 0;
    while i < (*input).numIndexes {
        j = 0;
        while j < 3 {
            let idx = *(*input).indexes.add(i + j) as usize;
            p[j][0] = *(*input).xyz.add(idx) - (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[0];
            p[j][1] = *(*input).xyz.add(idx + 1) - (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[1];
            p[j][2] = *(*input).xyz.add(idx + 2) - (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[2];
            j += 1;
        }
        ClipSkyPolygon(3, core::ptr::addr_of!(p[0][0]), 0);
        i += 3;
    }
}

/*
===================================================================================

CLOUD VERTEX GENERATION

===================================================================================
*/

/*
** MakeSkyVec
**
** Parms: s, t range from -1 to 1
*/

unsafe fn MakeSkyVec(s: f32, t: f32, axis: c_int, outSt: *mut [f32; 2], outXYZ: *mut [f32; 3]) {
    let axis = axis as usize;
    // 1 = s, 2 = t, 3 = 2048
    static ST_TO_VEC: [[i32; 3]; 6] = [
        [3, -1, 2],
        [-3, 1, 2],

        [1, 3, 2],
        [-1, -3, 2],

        [-2, -1, 3],		// 0 degrees yaw, look straight up
        [2, -1, -3]		// look straight down
    ];

    let mut b: [f32; 3];
    let mut j: usize;
    let mut k: i32;
    let mut boxSize: f32;

    boxSize = (*core::ptr::addr_of!(backEnd)).viewParms.zFar / 1.75;		// div sqrt(3)
    b[0] = s * boxSize;
    b[1] = t * boxSize;
    b[2] = boxSize;

    j = 0;
    while j < 3 {
        k = ST_TO_VEC[axis][j];
        if k < 0 {
            (*outXYZ)[j] = -b[((-k) - 1) as usize];
        } else {
            (*outXYZ)[j] = b[(k - 1) as usize];
        }
        j += 1;
    }

    // avoid bilerp seam
    let mut s_mut = (s + 1.0) * 0.5;
    let mut t_mut = (t + 1.0) * 0.5;

    if s_mut < sky_min {
        s_mut = sky_min;
    } else if s_mut > sky_max {
        s_mut = sky_max;
    }

    if t_mut < sky_min {
        t_mut = sky_min;
    } else if t_mut > sky_max {
        t_mut = sky_max;
    }

    t_mut = 1.0 - t_mut;

    if !outSt.is_null() {
        (*outSt)[0] = s_mut;
        (*outSt)[1] = t_mut;
    }
}

static mut s_skyPoints: [[[f32; 3]; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1] =
    [[[0.0; 3]; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1];
static mut s_skyTexCoords: [[[f32; 2]; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1] =
    [[[0.0; 2]; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1];

unsafe fn DrawSkySide(image: *mut image_s, mins: *const [c_int; 2], maxs: *const [c_int; 2]) {
    let mut s: c_int;
    let mut t: c_int;

    gl_bind(image);

    #[cfg(target_os = "xbox")]
    {
        let verts = (((*maxs)[0] + HALF_SKY_SUBDIVISIONS as c_int) - ((*mins)[0] + HALF_SKY_SUBDIVISIONS as c_int)) * 2 + 2;
        t = (*mins)[1] + HALF_SKY_SUBDIVISIONS as c_int;
        while t < (*maxs)[1] + HALF_SKY_SUBDIVISIONS as c_int {
            qgl_begin_ext(GL_TRIANGLE_STRIP, verts, 0, 0, verts, 0);

            s = (*mins)[0] + HALF_SKY_SUBDIVISIONS as c_int;
            while s <= (*maxs)[0] + HALF_SKY_SUBDIVISIONS as c_int {
                qgl_tex_coord_2fv(core::ptr::addr_of!(s_skyTexCoords[t as usize][(s) as usize]));
                qgl_vertex_3fv(core::ptr::addr_of!(s_skyPoints[t as usize][(s) as usize]));

                qgl_tex_coord_2fv(core::ptr::addr_of!(s_skyTexCoords[(t + 1) as usize][(s) as usize]));
                qgl_vertex_3fv(core::ptr::addr_of!(s_skyPoints[(t + 1) as usize][(s) as usize]));
                s += 1;
            }

            qgl_end();
            t += 1;
        }
    }

    #[cfg(not(target_os = "xbox"))]
    {
        t = (*mins)[1] + HALF_SKY_SUBDIVISIONS as c_int;
        while t < (*maxs)[1] + HALF_SKY_SUBDIVISIONS as c_int {
            qgl_begin(GL_TRIANGLE_STRIP);

            s = (*mins)[0] + HALF_SKY_SUBDIVISIONS as c_int;
            while s <= (*maxs)[0] + HALF_SKY_SUBDIVISIONS as c_int {
                qgl_tex_coord_2fv(core::ptr::addr_of!(s_skyTexCoords[t as usize][(s) as usize]));
                qgl_vertex_3fv(core::ptr::addr_of!(s_skyPoints[t as usize][(s) as usize]));

                qgl_tex_coord_2fv(core::ptr::addr_of!(s_skyTexCoords[(t + 1) as usize][(s) as usize]));
                qgl_vertex_3fv(core::ptr::addr_of!(s_skyPoints[(t + 1) as usize][(s) as usize]));
                s += 1;
            }

            qgl_end();
            t += 1;
        }
    }
}

unsafe fn DrawSkyBox(shader: *mut shader_t) {
    let mut i: usize;

    sky_min = 0.0;
    sky_max = 1.0;

    // memset s_skyTexCoords to 0
    s_skyTexCoords = [[[0.0; 2]; SKY_SUBDIVISIONS + 1]; SKY_SUBDIVISIONS + 1];

    i = 0;
    while i < 6 {
        let mut sky_mins_subd: [c_int; 2];
        let mut sky_maxs_subd: [c_int; 2];
        let mut s: c_int;
        let mut t: c_int;

        sky_mins[0][i] = (sky_mins[0][i] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_mins[1][i] = (sky_mins[1][i] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[0][i] = (sky_maxs[0][i] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[1][i] = (sky_maxs[1][i] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;

        if (sky_mins[0][i] >= sky_maxs[0][i]) ||
           (sky_mins[1][i] >= sky_maxs[1][i]) {
            i += 1;
            continue;
        }

        sky_mins_subd[0] = (sky_mins[0][i] * HALF_SKY_SUBDIVISIONS as f32) as c_int;
        sky_mins_subd[1] = (sky_mins[1][i] * HALF_SKY_SUBDIVISIONS as f32) as c_int;
        sky_maxs_subd[0] = (sky_maxs[0][i] * HALF_SKY_SUBDIVISIONS as f32) as c_int;
        sky_maxs_subd[1] = (sky_maxs[1][i] * HALF_SKY_SUBDIVISIONS as f32) as c_int;

        if sky_mins_subd[0] < -(HALF_SKY_SUBDIVISIONS as c_int) {
            sky_mins_subd[0] = -(HALF_SKY_SUBDIVISIONS as c_int);
        } else if sky_mins_subd[0] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_mins_subd[0] = HALF_SKY_SUBDIVISIONS as c_int;
        }
        if sky_mins_subd[1] < -(HALF_SKY_SUBDIVISIONS as c_int) {
            sky_mins_subd[1] = -(HALF_SKY_SUBDIVISIONS as c_int);
        } else if sky_mins_subd[1] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_mins_subd[1] = HALF_SKY_SUBDIVISIONS as c_int;
        }

        if sky_maxs_subd[0] < -(HALF_SKY_SUBDIVISIONS as c_int) {
            sky_maxs_subd[0] = -(HALF_SKY_SUBDIVISIONS as c_int);
        } else if sky_maxs_subd[0] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_maxs_subd[0] = HALF_SKY_SUBDIVISIONS as c_int;
        }
        if sky_maxs_subd[1] < -(HALF_SKY_SUBDIVISIONS as c_int) {
            sky_maxs_subd[1] = -(HALF_SKY_SUBDIVISIONS as c_int);
        } else if sky_maxs_subd[1] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_maxs_subd[1] = HALF_SKY_SUBDIVISIONS as c_int;
        }

        //
        // iterate through the subdivisions
        //
        t = sky_mins_subd[1] + HALF_SKY_SUBDIVISIONS as c_int;
        while t <= sky_maxs_subd[1] + HALF_SKY_SUBDIVISIONS as c_int {
            s = sky_mins_subd[0] + HALF_SKY_SUBDIVISIONS as c_int;
            while s <= sky_maxs_subd[0] + HALF_SKY_SUBDIVISIONS as c_int {
                MakeSkyVec(
                    ((s - HALF_SKY_SUBDIVISIONS as c_int) as f32) / (HALF_SKY_SUBDIVISIONS as f32),
                    ((t - HALF_SKY_SUBDIVISIONS as c_int) as f32) / (HALF_SKY_SUBDIVISIONS as f32),
                    i as c_int,
                    core::ptr::addr_of_mut!(s_skyTexCoords[t as usize][s as usize]),
                    core::ptr::addr_of_mut!(s_skyPoints[t as usize][s as usize]),
                );
                s += 1;
            }
            t += 1;
        }

        DrawSkySide(
            (*shader).sky.as_ref().unwrap().outerbox[i],
            core::ptr::addr_of!(sky_mins_subd),
            core::ptr::addr_of!(sky_maxs_subd),
        );
        i += 1;
    }
}

unsafe fn FillCloudySkySide(mins: *const [c_int; 2], maxs: *const [c_int; 2], addIndexes: bool) {
    let mut s: c_int;
    let mut t: c_int;
    let mut vertexStart: c_int = (*core::ptr::addr_of!(tess)).numVertexes as c_int;
    let mut tHeight: c_int;
    let mut sWidth: c_int;

    tHeight = (*maxs)[1] - (*mins)[1] + 1;
    sWidth = (*maxs)[0] - (*mins)[0] + 1;

    t = (*mins)[1] + HALF_SKY_SUBDIVISIONS as c_int;
    while t <= (*maxs)[1] + HALF_SKY_SUBDIVISIONS as c_int {
        s = (*mins)[0] + HALF_SKY_SUBDIVISIONS as c_int;
        while s <= (*maxs)[0] + HALF_SKY_SUBDIVISIONS as c_int {
            let tess_ref = core::ptr::addr_of_mut!(tess);
            let idx = (*tess_ref).numVertexes;
            (*tess_ref).xyz[idx] = s_skyPoints[t as usize][s as usize];
            (*tess_ref).xyz[idx][0] += (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[0];
            (*tess_ref).xyz[idx][1] += (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[1];
            (*tess_ref).xyz[idx][2] += (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[2];
            (*tess_ref).texCoords[idx][0][0] = s_skyTexCoords[t as usize][s as usize][0];
            (*tess_ref).texCoords[idx][0][1] = s_skyTexCoords[t as usize][s as usize][1];

            (*tess_ref).numVertexes += 1;

            if (*tess_ref).numVertexes >= SHADER_MAX_VERTEXES {
                com_error(ERR_DROP, "SHADER_MAX_VERTEXES hit in FillCloudySkySide()\n\0");
            }
            s += 1;
        }
        t += 1;
    }

    // only add indexes for one pass, otherwise it would draw multiple times for each pass
    if addIndexes {
        let mut t_loop: c_int = 0;
        while t_loop < tHeight - 1 {
            let mut s_loop: c_int = 0;
            while s_loop < sWidth - 1 {
                let tess_ref = core::ptr::addr_of_mut!(tess);
                let idx = (*tess_ref).numIndexes;

                (*tess_ref).indexes[idx] = (vertexStart + s_loop + t_loop * sWidth) as u32;
                (*tess_ref).numIndexes += 1;
                (*tess_ref).indexes[(*tess_ref).numIndexes] = (vertexStart + s_loop + (t_loop + 1) * sWidth) as u32;
                (*tess_ref).numIndexes += 1;
                (*tess_ref).indexes[(*tess_ref).numIndexes] = (vertexStart + s_loop + 1 + t_loop * sWidth) as u32;
                (*tess_ref).numIndexes += 1;

                (*tess_ref).indexes[(*tess_ref).numIndexes] = (vertexStart + s_loop + (t_loop + 1) * sWidth) as u32;
                (*tess_ref).numIndexes += 1;
                (*tess_ref).indexes[(*tess_ref).numIndexes] = (vertexStart + s_loop + 1 + (t_loop + 1) * sWidth) as u32;
                (*tess_ref).numIndexes += 1;
                (*tess_ref).indexes[(*tess_ref).numIndexes] = (vertexStart + s_loop + 1 + t_loop * sWidth) as u32;
                (*tess_ref).numIndexes += 1;

                s_loop += 1;
            }
            t_loop += 1;
        }
    }
}

unsafe fn FillCloudBox(shader: *const shader_t, stage: c_int) {
    let mut i: usize;

    i = 0;
    while i < 6 {
        let mut sky_mins_subd: [c_int; 2];
        let mut sky_maxs_subd: [c_int; 2];
        let mut s: c_int;
        let mut t: c_int;
        let mut MIN_T: f32;

        if true { // FIXME? shader->sky->fullClouds
            MIN_T = -(HALF_SKY_SUBDIVISIONS as f32);

            // still don't want to draw the bottom, even if fullClouds
            if i == 5 {
                i += 1;
                continue;
            }
        } else {
            match i {
                0 | 1 | 2 | 3 => MIN_T = -1.0,
                5 => {
                    // don't draw clouds beneath you
                    i += 1;
                    continue;
                }
                4 | _ => MIN_T = -(HALF_SKY_SUBDIVISIONS as f32),
            }
        }

        sky_mins[0][i] = (sky_mins[0][i] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_mins[1][i] = (sky_mins[1][i] * HALF_SKY_SUBDIVISIONS as f32).floor() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[0][i] = (sky_maxs[0][i] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;
        sky_maxs[1][i] = (sky_maxs[1][i] * HALF_SKY_SUBDIVISIONS as f32).ceil() / HALF_SKY_SUBDIVISIONS as f32;

        if (sky_mins[0][i] >= sky_maxs[0][i]) ||
           (sky_mins[1][i] >= sky_maxs[1][i]) {
            i += 1;
            continue;
        }

        sky_mins_subd[0] = myftol(sky_mins[0][i] * HALF_SKY_SUBDIVISIONS as f32);
        sky_mins_subd[1] = myftol(sky_mins[1][i] * HALF_SKY_SUBDIVISIONS as f32);
        sky_maxs_subd[0] = myftol(sky_maxs[0][i] * HALF_SKY_SUBDIVISIONS as f32);
        sky_maxs_subd[1] = myftol(sky_maxs[1][i] * HALF_SKY_SUBDIVISIONS as f32);

        if sky_mins_subd[0] < -(HALF_SKY_SUBDIVISIONS as c_int) {
            sky_mins_subd[0] = -(HALF_SKY_SUBDIVISIONS as c_int);
        } else if sky_mins_subd[0] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_mins_subd[0] = HALF_SKY_SUBDIVISIONS as c_int;
        }
        if (sky_mins_subd[1] as f32) < MIN_T {
            sky_mins_subd[1] = MIN_T as c_int;
        } else if sky_mins_subd[1] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_mins_subd[1] = HALF_SKY_SUBDIVISIONS as c_int;
        }

        if sky_maxs_subd[0] < -(HALF_SKY_SUBDIVISIONS as c_int) {
            sky_maxs_subd[0] = -(HALF_SKY_SUBDIVISIONS as c_int);
        } else if sky_maxs_subd[0] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_maxs_subd[0] = HALF_SKY_SUBDIVISIONS as c_int;
        }
        if (sky_maxs_subd[1] as f32) < MIN_T {
            sky_maxs_subd[1] = MIN_T as c_int;
        } else if sky_maxs_subd[1] > HALF_SKY_SUBDIVISIONS as c_int {
            sky_maxs_subd[1] = HALF_SKY_SUBDIVISIONS as c_int;
        }

        //
        // iterate through the subdivisions
        //
        t = sky_mins_subd[1] + HALF_SKY_SUBDIVISIONS as c_int;
        while t <= sky_maxs_subd[1] + HALF_SKY_SUBDIVISIONS as c_int {
            s = sky_mins_subd[0] + HALF_SKY_SUBDIVISIONS as c_int;
            while s <= sky_maxs_subd[0] + HALF_SKY_SUBDIVISIONS as c_int {
                MakeSkyVec(
                    ((s - HALF_SKY_SUBDIVISIONS as c_int) as f32) / (HALF_SKY_SUBDIVISIONS as f32),
                    ((t - HALF_SKY_SUBDIVISIONS as c_int) as f32) / (HALF_SKY_SUBDIVISIONS as f32),
                    i as c_int,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(s_skyPoints[t as usize][s as usize]),
                );

                s_skyTexCoords[t as usize][s as usize][0] = s_cloudTexCoords[i][t as usize][s as usize][0];
                s_skyTexCoords[t as usize][s as usize][1] = s_cloudTexCoords[i][t as usize][s as usize][1];
                s += 1;
            }
            t += 1;
        }

        // only add indexes for first stage
        FillCloudySkySide(core::ptr::addr_of!(sky_mins_subd), core::ptr::addr_of!(sky_maxs_subd), stage == 0);
        i += 1;
    }
}

/*
** R_BuildCloudData
*/
pub unsafe fn R_BuildCloudData(input: *mut shaderCommands_t) {
    let mut i: usize;
    let mut shader: *mut shader_t;

    shader = (*input).shader;

    assert!(!(*shader).sky.is_none());

    sky_min = 1.0 / 256.0;		// FIXME: not correct?
    sky_max = 255.0 / 256.0;

    // set up for drawing
    (*core::ptr::addr_of_mut!(tess)).numIndexes = 0;
    (*core::ptr::addr_of_mut!(tess)).numVertexes = 0;

    if (*(*shader).sky.as_ref().unwrap()).cloudHeight != 0.0 {
        i = 0;
        while i < (*input).shader.as_ref().unwrap().numUnfoggedPasses as usize {
            FillCloudBox(input.as_ref().unwrap().shader, i as c_int);
            i += 1;
        }
    }
}

/*
** R_InitSkyTexCoords
** Called when a sky shader is parsed
*/
const SQR: fn(f32) -> f32 = |a| a * a;

pub unsafe fn R_InitSkyTexCoords(heightCloud: f32) {
    let mut i: usize;
    let mut s: usize;
    let mut t: usize;
    let mut radiusWorld: f32 = MAX_WORLD_COORD;
    let mut p: f32;
    let mut sRad: f32;
    let mut tRad: f32;
    let mut skyVec: [f32; 3];
    let mut v: [f32; 3];

    // init zfar so MakeSkyVec works even though
    // a world hasn't been bounded
    (*core::ptr::addr_of_mut!(backEnd)).viewParms.zFar = 1024.0;

    i = 0;
    while i < 6 {
        t = 0;
        while t <= SKY_SUBDIVISIONS {
            s = 0;
            while s <= SKY_SUBDIVISIONS {
                // compute vector from view origin to sky side integral point
                MakeSkyVec(
                    ((s as c_int - HALF_SKY_SUBDIVISIONS as c_int) as f32) / (HALF_SKY_SUBDIVISIONS as f32),
                    ((t as c_int - HALF_SKY_SUBDIVISIONS as c_int) as f32) / (HALF_SKY_SUBDIVISIONS as f32),
                    i as c_int,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(skyVec),
                );

                // compute parametric value 'p' that intersects with cloud layer
                p = (1.0 / (2.0 * dot_product(core::ptr::addr_of!(skyVec), core::ptr::addr_of!(skyVec)))) *
                    (-2.0 * skyVec[2] * radiusWorld +
                       2.0 * (SQR(skyVec[2]) * SQR(radiusWorld) +
                             2.0 * SQR(skyVec[0]) * radiusWorld * heightCloud +
                            SQR(skyVec[0]) * SQR(heightCloud) +
                            2.0 * SQR(skyVec[1]) * radiusWorld * heightCloud +
                            SQR(skyVec[1]) * SQR(heightCloud) +
                            2.0 * SQR(skyVec[2]) * radiusWorld * heightCloud +
                            SQR(skyVec[2]) * SQR(heightCloud)).sqrt());

                s_cloudTexP[i][t][s] = p;

                // compute intersection point based on p
                v[0] = skyVec[0] * p;
                v[1] = skyVec[1] * p;
                v[2] = skyVec[2] * p;
                v[2] += radiusWorld;

                // compute vector from world origin to intersection point 'v'
                vector_normalize(core::ptr::addr_of_mut!(v));

                sRad = v[0].acos();
                tRad = v[1].acos();

                s_cloudTexCoords[i][t][s][0] = sRad;
                s_cloudTexCoords[i][t][s][1] = tRad;
                s += 1;
            }
            t += 1;
        }
        i += 1;
    }
}

//======================================================================================

/*
** RB_DrawSun
*/
pub unsafe fn RB_DrawSun() {
    let mut size: f32;
    let mut dist: f32;
    let mut origin: [f32; 3];
    let mut vec1: [f32; 3];
    let mut vec2: [f32; 3];
    let mut temp: [f32; 3];

    if !(*core::ptr::addr_of!(backEnd)).skyRenderedThisView {
        return;
    }
    if (*r_drawSun).is_null() || (*(*r_drawSun)).integer == 0 {
        return;
    }
    qgl_load_matrixf(core::ptr::addr_of!((*core::ptr::addr_of!(backEnd)).viewParms.world.modelMatrix));
    qgl_translatef(
        (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[0],
        (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[1],
        (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[2],
    );

    dist = (*core::ptr::addr_of!(backEnd)).viewParms.zFar / 1.75;		// div sqrt(3)
    size = dist * 0.4;

    origin[0] = (*core::ptr::addr_of!(tr)).sunDirection[0] * dist;
    origin[1] = (*core::ptr::addr_of!(tr)).sunDirection[1] * dist;
    origin[2] = (*core::ptr::addr_of!(tr)).sunDirection[2] * dist;

    perpendicular_vector(core::ptr::addr_of_mut!(vec1), core::ptr::addr_of!((*core::ptr::addr_of!(tr)).sunDirection));
    cross_product(core::ptr::addr_of!((*core::ptr::addr_of!(tr)).sunDirection), core::ptr::addr_of!(vec1), core::ptr::addr_of_mut!(vec2));

    vec1[0] *= size;
    vec1[1] *= size;
    vec1[2] *= size;
    vec2[0] *= size;
    vec2[1] *= size;
    vec2[2] *= size;

    // farthest depth range
    qgl_depth_range(1.0, 1.0);

    // FIXME: use quad stamp
    RB_BeginSurface((*core::ptr::addr_of!(tr)).sunShader, (*core::ptr::addr_of!(tess)).fogNum);

        temp[0] = origin[0];
        temp[1] = origin[1];
        temp[2] = origin[2];
        temp[0] -= vec1[0];
        temp[1] -= vec1[1];
        temp[2] -= vec1[2];
        temp[0] -= vec2[0];
        temp[1] -= vec2[1];
        temp[2] -= vec2[2];
        let tess_ref = core::ptr::addr_of_mut!(tess);
        (*tess_ref).xyz[(*tess_ref).numVertexes][0] = temp[0];
        (*tess_ref).xyz[(*tess_ref).numVertexes][1] = temp[1];
        (*tess_ref).xyz[(*tess_ref).numVertexes][2] = temp[2];
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][0] = 0.0;
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][1] = 0.0;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][0] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][1] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][2] = 255;
        (*tess_ref).numVertexes += 1;

        temp[0] = origin[0];
        temp[1] = origin[1];
        temp[2] = origin[2];
        temp[0] += vec1[0];
        temp[1] += vec1[1];
        temp[2] += vec1[2];
        temp[0] -= vec2[0];
        temp[1] -= vec2[1];
        temp[2] -= vec2[2];
        let tess_ref = core::ptr::addr_of_mut!(tess);
        (*tess_ref).xyz[(*tess_ref).numVertexes][0] = temp[0];
        (*tess_ref).xyz[(*tess_ref).numVertexes][1] = temp[1];
        (*tess_ref).xyz[(*tess_ref).numVertexes][2] = temp[2];
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][0] = 0.0;
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][1] = 1.0;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][0] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][1] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][2] = 255;
        (*tess_ref).numVertexes += 1;

        temp[0] = origin[0];
        temp[1] = origin[1];
        temp[2] = origin[2];
        temp[0] += vec1[0];
        temp[1] += vec1[1];
        temp[2] += vec1[2];
        temp[0] += vec2[0];
        temp[1] += vec2[1];
        temp[2] += vec2[2];
        let tess_ref = core::ptr::addr_of_mut!(tess);
        (*tess_ref).xyz[(*tess_ref).numVertexes][0] = temp[0];
        (*tess_ref).xyz[(*tess_ref).numVertexes][1] = temp[1];
        (*tess_ref).xyz[(*tess_ref).numVertexes][2] = temp[2];
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][0] = 1.0;
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][1] = 1.0;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][0] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][1] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][2] = 255;
        (*tess_ref).numVertexes += 1;

        temp[0] = origin[0];
        temp[1] = origin[1];
        temp[2] = origin[2];
        temp[0] -= vec1[0];
        temp[1] -= vec1[1];
        temp[2] -= vec1[2];
        temp[0] += vec2[0];
        temp[1] += vec2[1];
        temp[2] += vec2[2];
        let tess_ref = core::ptr::addr_of_mut!(tess);
        (*tess_ref).xyz[(*tess_ref).numVertexes][0] = temp[0];
        (*tess_ref).xyz[(*tess_ref).numVertexes][1] = temp[1];
        (*tess_ref).xyz[(*tess_ref).numVertexes][2] = temp[2];
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][0] = 1.0;
        (*tess_ref).texCoords[(*tess_ref).numVertexes][0][1] = 0.0;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][0] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][1] = 255;
        (*tess_ref).vertexColors[(*tess_ref).numVertexes][2] = 255;
        (*tess_ref).numVertexes += 1;

        let tess_ref = core::ptr::addr_of_mut!(tess);
        (*tess_ref).indexes[(*tess_ref).numIndexes] = 0;
        (*tess_ref).numIndexes += 1;
        (*tess_ref).indexes[(*tess_ref).numIndexes] = 1;
        (*tess_ref).numIndexes += 1;
        (*tess_ref).indexes[(*tess_ref).numIndexes] = 2;
        (*tess_ref).numIndexes += 1;
        (*tess_ref).indexes[(*tess_ref).numIndexes] = 0;
        (*tess_ref).numIndexes += 1;
        (*tess_ref).indexes[(*tess_ref).numIndexes] = 2;
        (*tess_ref).numIndexes += 1;
        (*tess_ref).indexes[(*tess_ref).numIndexes] = 3;
        (*tess_ref).numIndexes += 1;

    RB_EndSurface();

    // back to normal depth range
    qgl_depth_range(0.0, 1.0);
}




/*
================
RB_StageIteratorSky

All of the visible sky triangles are in tess

Other things could be stuck in here, like birds in the sky, etc
================
*/
pub unsafe fn RB_StageIteratorSky() {
    if (*r_fastsky).is_null() || (*(*r_fastsky)).integer != 0 {
        return;
    }

    if skyboxportal && !((*core::ptr::addr_of!(backEnd)).refdef.rdflags & RDF_SKYBOXPORTAL) != 0 {
        return;
    }

    // go through all the polygons and project them onto
    // the sky box to see which blocks on each side need
    // to be drawn
    RB_ClipSkyPolygons(core::ptr::addr_of_mut!(tess));

    // r_showsky will let all the sky blocks be drawn in
    // front of everything to allow developers to see how
    // much sky is getting sucked in
    if !(*r_showsky).is_null() && (*(*r_showsky)).integer != 0 {
        qgl_depth_range(0.0, 0.0);
    } else {
        #[cfg(target_os = "xbox")]
        {
            qgl_depth_range(0.99, 1.0);
        }
        #[cfg(not(target_os = "xbox"))]
        {
            qgl_depth_range(1.0, 1.0);
        }
    }

    // draw the outer skybox
    if !(*core::ptr::addr_of!(tess)).shader.is_null() &&
       !(*(*core::ptr::addr_of!(tess)).shader).sky.is_none() &&
       !(*(*(*core::ptr::addr_of!(tess)).shader).sky.as_ref().unwrap()).outerbox[0].is_null() &&
       (*(*(*core::ptr::addr_of!(tess)).shader).sky.as_ref().unwrap()).outerbox[0] != (*core::ptr::addr_of!(tr)).defaultImage {
        qgl_color_3f((*core::ptr::addr_of!(tr)).identityLight, (*core::ptr::addr_of!(tr)).identityLight, (*core::ptr::addr_of!(tr)).identityLight);

        qgl_push_matrix();
        GL_State(0);
        qgl_translatef(
            (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[0],
            (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[1],
            (*core::ptr::addr_of!(backEnd)).viewParms.or.origin[2],
        );

        DrawSkyBox((*core::ptr::addr_of!(tess)).shader);

        qgl_pop_matrix();
    }

    // generate the vertexes for all the clouds, which will be drawn
    // by the generic shader routine
    R_BuildCloudData(core::ptr::addr_of_mut!(tess));

    RB_StageIteratorGeneric();

    // draw the inner skybox


    // back to normal depth range
    qgl_depth_range(0.0, 1.0);

    // note that sky was drawn so we will draw a sun later
    (*core::ptr::addr_of_mut!(backEnd)).skyRenderedThisView = true;
}

// ============================================================================
// Stub declarations for external types and functions
// ============================================================================

// External types from tr_local.h
#[repr(C)]
pub struct image_s {
    // stub - filled from tr_local.h
}

#[repr(C)]
pub struct shader_t {
    sky: Option<Box<shader_sky_t>>,
    numUnfoggedPasses: c_int,
    // ... other fields
}

#[repr(C)]
pub struct shader_sky_t {
    cloudHeight: f32,
    outerbox: [*mut image_s; 6],
}

#[repr(C)]
pub struct shaderCommands_t {
    xyz: *mut [f32; 3],
    texCoords: *mut [[f32; 2]; 1],
    vertexColors: *mut [u8; 3],
    indexes: *mut u32,
    numIndexes: usize,
    numVertexes: usize,
    shader: *mut shader_t,
}

#[repr(C)]
pub struct tess_t {
    xyz: [[f32; 3]; 4096],
    texCoords: [[[f32; 2]; 1]; 4096],
    vertexColors: [[u8; 3]; 4096],
    indexes: [u32; 8192],
    numIndexes: usize,
    numVertexes: usize,
    shader: *mut shader_t,
    fogNum: c_int,
}

#[repr(C)]
pub struct viewParms_s {
    or: orientation_t,
    world: frame_t,
    zFar: f32,
    // ... other fields
}

#[repr(C)]
pub struct orientation_t {
    origin: [f32; 3],
    // ... other fields
}

#[repr(C)]
pub struct frame_t {
    modelMatrix: [f32; 16],
    // ... other fields
}

#[repr(C)]
pub struct backEnd_t {
    viewParms: viewParms_s,
    refdef: refdef_t,
    skyRenderedThisView: bool,
}

#[repr(C)]
pub struct refdef_t {
    rdflags: u32,
    // ... other fields
}

#[repr(C)]
pub struct cvar_s {
    integer: c_int,
}

#[repr(C)]
pub struct trGlobals_t {
    sunDirection: [f32; 3],
    identityLight: f32,
    sunShader: *mut shader_t,
    defaultImage: *mut image_s,
}

// External globals
extern "C" {
    static mut backEnd: backEnd_t;
    static mut tess: tess_t;
    static mut tr: trGlobals_t;
    static r_drawSun: *const cvar_s;
    static r_fastsky: *const cvar_s;
    static r_showsky: *const cvar_s;
}

const MAX_WORLD_COORD: f32 = 128000.0;
const MIN_WORLD_COORD: f32 = -128000.0;
const SHADER_MAX_VERTEXES: usize = 4096;

const SIDE_FRONT: i32 = 0;
const SIDE_BACK: i32 = 1;
const SIDE_ON: i32 = 2;

const GL_TRIANGLE_STRIP: c_int = 5;
const ERR_DROP: c_int = 0;
const RDF_SKYBOXPORTAL: u32 = 0x01;

static mut skyboxportal: bool = false;

// Stub function declarations
fn gl_bind(_image: *mut image_s) {}
fn qgl_begin(_mode: c_int) {}
fn qgl_begin_ext(_mode: c_int, _verts: c_int, _a: c_int, _b: c_int, _c: c_int, _d: c_int) {}
fn qgl_end() {}
fn qgl_tex_coord_2fv(_v: *const [f32; 2]) {}
fn qgl_vertex_3fv(_v: *const [f32; 3]) {}
fn qgl_load_matrixf(_m: *const f32) {}
fn qgl_translatef(_x: f32, _y: f32, _z: f32) {}
fn qgl_depth_range(_near: f32, _far: f32) {}
fn qgl_color_3f(_r: f32, _g: f32, _b: f32) {}
fn qgl_push_matrix() {}
fn qgl_pop_matrix() {}
fn GL_State(_flags: c_int) {}
fn RB_BeginSurface(_shader: *mut shader_t, _fogNum: c_int) {}
fn RB_EndSurface() {}
fn RB_StageIteratorGeneric() {}
fn dot_product(a: *const f32, b: *const f32) -> f32 {
    unsafe {
        *a * *b + *a.add(1) * *b.add(1) + *a.add(2) * *b.add(2)
    }
}
fn vector_normalize(_v: *mut [f32; 3]) {}
fn perpendicular_vector(_dst: *mut [f32; 3], _src: *const [f32; 3]) {}
fn cross_product(_v1: *const [f32; 3], _v2: *const [f32; 3], _cross: *mut [f32; 3]) {}
fn myftol(f: f32) -> c_int {
    f as c_int
}
fn com_error(_level: c_int, _msg: &str) {}
