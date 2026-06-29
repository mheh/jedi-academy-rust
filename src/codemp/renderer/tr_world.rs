//Anything above this #include will be ignored by the compiler

use core::ffi::{c_int, c_char, c_void};
use crate::codemp::qcommon::exe_headers_h::*;
use crate::codemp::renderer::tr_local_h::*;

#[cfg(feature = "vv_lighting")]
use crate::codemp::renderer::tr_lightmanager_h::*;

#[cfg(target_os = "windows")]  // _XBOX placeholder
static mut lookingForWorstLeaf: bool = false;

#[cfg(not(target_os = "windows"))]
#[inline]
unsafe fn Q_CastShort2Float(f: *mut f32, s: *const i16) {
	*f = *s as f32;
}

#[cfg(target_os = "windows")]
unsafe fn GetCoordsForLeaf(leafNum: c_int, coords: &mut [f32; 3]) -> bool {
	let face: *mut srfSurfaceFace_t;
	let surf: *mut msurface_t;
	let mut i: c_int;

	i = 0;
	while i < (*tr.world).leafs[leafNum as usize].nummarksurfaces {
		surf = *(((*tr.world).marksurfaces as *mut *mut msurface_t).add(((*tr.world).leafs[leafNum as usize].firstMarkSurfNum + i) as usize));

		if (*surf).data.is_null() || *(*surf).data != SF_FACE as c_int {
			i += 1;
			continue;
		}

		face = (*surf).data as *mut srfSurfaceFace_t;
		Q_CastShort2Float(&mut coords[0], ((*face).srfPoints as *const i16).add(0));
		Q_CastShort2Float(&mut coords[1], ((*face).srfPoints as *const i16).add(1));
		Q_CastShort2Float(&mut coords[2], ((*face).srfPoints as *const i16).add(2));
		return true;
	}

	return false;
}

/*
=================
R_CullTriSurf

Returns true if the grid is completely culled away.
Also sets the clipped hint bit in tess
=================
*/
unsafe fn R_CullTriSurf(cv: *mut srfTriangles_t) -> qboolean {
	let boxCull: c_int;

	boxCull = R_CullLocalBox((*cv).bounds);

	if boxCull == CULL_OUT as c_int {
		return qtrue;
	}
	return qfalse;
}

/*
=================
R_CullGrid

Returns true if the grid is completely culled away.
Also sets the clipped hint bit in tess
=================
*/
unsafe fn R_CullGrid(cv: *mut srfGridMesh_t) -> qboolean {
	let boxCull: c_int;
	let sphereCull: c_int;

	if (*r_nocurves).integer != 0 {
		return qtrue;
	}

	if (*tr).currentEntityNum != TR_WORLDENT as c_int {
		sphereCull = R_CullLocalPointAndRadius((*cv).localOrigin, (*cv).meshRadius);
	} else {
		sphereCull = R_CullPointAndRadius((*cv).localOrigin, (*cv).meshRadius);
	}
	boxCull = CULL_OUT as c_int;

	// check for trivial reject
	if sphereCull == CULL_OUT as c_int
	{
		(*tr).pc.c_sphere_cull_patch_out += 1;
		return qtrue;
	}
	// check bounding box if necessary
	else if sphereCull == CULL_CLIP as c_int
	{
		(*tr).pc.c_sphere_cull_patch_clip += 1;

		boxCull = R_CullLocalBox((*cv).meshBounds);

		if boxCull == CULL_OUT as c_int
		{
			(*tr).pc.c_box_cull_patch_out += 1;
			return qtrue;
		}
		else if boxCull == CULL_IN as c_int
		{
			(*tr).pc.c_box_cull_patch_in += 1;
		}
		else
		{
			(*tr).pc.c_box_cull_patch_clip += 1;
		}
	}
	else
	{
		(*tr).pc.c_sphere_cull_patch_in += 1;
	}

	return qfalse;
}

/*
================
R_CullSurface

Tries to back face cull surfaces before they are lighted or
added to the sorting list.

This will also allow mirrors on both sides of a model without recursion.
================
*/
unsafe fn R_CullSurface(surface: *mut surfaceType_t, shader: *mut shader_t) -> qboolean {
	let sface: *mut srfSurfaceFace_t;
	let mut d: f32;

	if (*r_nocull).integer != 0 {
		return qfalse;
	}

	if *surface == SF_FACE as c_int {
		return R_CullGrid((surface) as *mut srfGridMesh_t);
	}

	if *surface == SF_TRIANGLES as c_int {
		return R_CullTriSurf((surface) as *mut srfTriangles_t);
	}

	if *surface != SF_FACE as c_int {
		return qfalse;
	}

	if (*shader).cullType == CT_TWO_SIDED as c_int {
		return qfalse;
	}

	// face culling
	if (*r_facePlaneCull).integer == 0 {
		return qfalse;
	}

	sface = (surface) as *mut srfSurfaceFace_t;

	if (*r_cullRoofFaces).integer != 0
	{ //Very slow, but this is only intended for taking shots for automap images.
		if (*sface).plane.normal[2] > 0.0f && (*sface).numPoints > 0
		{ //it's facing up I guess
			static mut i: c_int = 0;
			static mut tr_local: trace_t = core::mem::zeroed();
			static mut basePoint: [f32; 3] = [0.0; 3];
			static mut endPoint: [f32; 3] = [0.0; 3];
			static mut nNormal: [f32; 3] = [0.0; 3];
			static mut v: [f32; 3] = [0.0; 3];

			//The fact that this point is in the middle of the array has no relation to the
			//orientation in the surface outline.
			#[cfg(target_os = "windows")]
			{
				Q_CastShort2Float(&mut basePoint[0], ((*sface).srfPoints as *const i16).add(((*sface).numPoints / 2 + 0) as usize));
				Q_CastShort2Float(&mut basePoint[1], ((*sface).srfPoints as *const i16).add(((*sface).numPoints / 2 + 1) as usize));
				Q_CastShort2Float(&mut basePoint[2], ((*sface).srfPoints as *const i16).add(((*sface).numPoints / 2 + 2) as usize));
			}
			#[cfg(not(target_os = "windows"))]
			{
				basePoint[0] = (*sface).points[((*sface).numPoints / 2) as usize][0];
				basePoint[1] = (*sface).points[((*sface).numPoints / 2) as usize][1];
				basePoint[2] = (*sface).points[((*sface).numPoints / 2) as usize][2];
			}
			basePoint[2] += 2.0f;

			//the endpoint will be 8192 units from the chosen point
			//in the direction of the surface normal

			//just go straight up I guess, for now (slight hack)
			VectorSet(&mut nNormal, 0.0f, 0.0f, 1.0f);
			VectorMA(&basePoint, 8192.0f, &nNormal, &mut endPoint);

			CM_BoxTrace(&mut tr_local, &basePoint, &endPoint, core::ptr::null_mut(), core::ptr::null_mut(), 0, (CONTENTS_SOLID | CONTENTS_TERRAIN) as c_int, qfalse);

			if !tr_local.startsolid && !tr_local.allsolid &&
				(tr_local.fraction == 1.0f || (tr_local.surfaceFlags & SURF_NOIMPACT as c_int) != 0)
			{ //either hit nothing or sky, so this surface is near the top of the level I guess. Or the floor of a really tall room, but if that's the case we're just screwed.
				VectorSubtract(&basePoint, &tr_local.endpos, &mut v);
				if tr_local.fraction == 1.0f || VectorLength(&v) < (*r_roofCullCeilDist).value
				{ //ignore it if it's not close to the top, unless it just hit nothing
					//Let's try to dig back into the brush based on the negative direction of the plane,
					//and if we pop out on the other side we'll see if it's ground or not.
					i = 4;
					VectorCopy(&(*sface).plane.normal, &mut nNormal);
					VectorInverse(&mut nNormal);

					while i < 4096
					{
						VectorMA(&basePoint, i as f32, &nNormal, &mut endPoint);
						CM_BoxTrace(&mut tr_local, &endPoint, &endPoint, core::ptr::null_mut(), core::ptr::null_mut(), 0, (CONTENTS_SOLID | CONTENTS_TERRAIN) as c_int, qfalse);
						if !tr_local.startsolid && !tr_local.allsolid && tr_local.fraction == 1.0f
						{ //in the clear
							break;
						}
						i += 1;
					}
					if i < 4096
					{ //Make sure we got into clearance
						VectorCopy(&endPoint, &mut basePoint);
						basePoint[2] -= 2.0f;

						//just go straight down I guess, for now (slight hack)
						VectorSet(&mut nNormal, 0.0f, 0.0f, -1.0f);
						VectorMA(&basePoint, 4096.0f, &nNormal, &mut endPoint);

						//trace a second time from the clear point in the inverse normal direction of the surface.
						//If we hit something within a set amount of units, we will assume it's a bridge type object
						//and leave it to be drawn. Otherwise we will assume it is a roof or other obstruction and
						//cull it out.
						CM_BoxTrace(&mut tr_local, &basePoint, &endPoint, core::ptr::null_mut(), core::ptr::null_mut(), 0, (CONTENTS_SOLID | CONTENTS_TERRAIN) as c_int, qfalse);

						if !tr_local.startsolid && !tr_local.allsolid &&
							(tr_local.fraction != 1.0f && (tr_local.surfaceFlags & SURF_NOIMPACT as c_int) == 0)
						{ //if we hit nothing or a noimpact going down then this is probably "ground".
							VectorSubtract(&basePoint, &tr_local.endpos, &mut endPoint);
							if VectorLength(&endPoint) > (*r_roofCullCeilDist).value
							{ //128 (by default) is our maximum tolerance, above that will be removed
								return qtrue;
							}
						}
					}
				}
			}
		}
	}

	d = DotProduct(&(*tr).ori.viewOrigin, &(*sface).plane.normal);

	// don't cull exactly on the plane, because there are levels of rounding
	// through the BSP, ICD, and hardware that may cause pixel gaps if an
	// epsilon isn't allowed here
	if (*shader).cullType == CT_FRONT_SIDED as c_int {
		if d < (*sface).plane.dist - 8.0f {
			return qtrue;
		}
	} else {
		if d > (*sface).plane.dist + 8.0f {
			return qtrue;
		}
	}

	return qfalse;
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightFace(face: *mut srfSurfaceFace_t, dlightBits: c_int) -> c_int {
	let mut d: f32;
	let mut i: c_int;
	let mut dl: *mut dlight_t;
	let mut dlightBits_mut: c_int = dlightBits;

	i = 0;
	while i < (*tr).refdef.num_dlights {
		if (dlightBits_mut & (1 << i)) == 0 {
			i += 1;
			continue;
		}
		dl = &mut (*tr).refdef.dlights[i as usize];
		d = DotProduct(&(*dl).origin, &(*face).plane.normal) - (*face).plane.dist;
		if !VectorCompare(&(*face).plane.normal, &vec3_origin) && (d < -(*dl).radius || d > (*dl).radius) {
			// dlight doesn't reach the plane
			dlightBits_mut &= !(1 << i);
		}
		i += 1;
	}

	if dlightBits_mut == 0 {
		(*tr).pc.c_dlightSurfacesCulled += 1;
	}

	(*face).dlightBits = dlightBits_mut;
	return dlightBits_mut;
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightGrid(grid: *mut srfGridMesh_t, dlightBits: c_int) -> c_int {
	let mut i: c_int;
	let mut dl: *mut dlight_t;
	let mut dlightBits_mut: c_int = dlightBits;

	i = 0;
	while i < (*tr).refdef.num_dlights {
		if (dlightBits_mut & (1 << i)) == 0 {
			i += 1;
			continue;
		}
		dl = &mut (*tr).refdef.dlights[i as usize];
		if (*dl).origin[0] - (*dl).radius > (*grid).meshBounds[1][0]
			|| (*dl).origin[0] + (*dl).radius < (*grid).meshBounds[0][0]
			|| (*dl).origin[1] - (*dl).radius > (*grid).meshBounds[1][1]
			|| (*dl).origin[1] + (*dl).radius < (*grid).meshBounds[0][1]
			|| (*dl).origin[2] - (*dl).radius > (*grid).meshBounds[1][2]
			|| (*dl).origin[2] + (*dl).radius < (*grid).meshBounds[0][2] {
			// dlight doesn't reach the bounds
			dlightBits_mut &= !(1 << i);
		}
		i += 1;
	}

	if dlightBits_mut == 0 {
		(*tr).pc.c_dlightSurfacesCulled += 1;
	}

	(*grid).dlightBits = dlightBits_mut;
	return dlightBits_mut;
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightTrisurf(surf: *mut srfTriangles_t, dlightBits: c_int) -> c_int {
	// FIXME: more dlight culling to trisurfs...
	(*surf).dlightBits = dlightBits;
	return dlightBits;
}

/*
====================
R_DlightSurface

The given surface is going to be drawn, and it touches a leaf
that is touched by one or more dlights, so try to throw out
more dlights if possible.
====================
*/
#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_DlightSurface(surf: *mut msurface_t, dlightBits: c_int) -> c_int {
	let mut dlightBits_local: c_int = dlightBits;
	if *(*surf).data == SF_FACE as c_int {
		dlightBits_local = R_DlightFace(((*surf).data) as *mut srfSurfaceFace_t, dlightBits_local);
	} else if *(*surf).data == SF_GRID as c_int {
		dlightBits_local = R_DlightGrid(((*surf).data) as *mut srfGridMesh_t, dlightBits_local);
	} else if *(*surf).data == SF_TRIANGLES as c_int {
		dlightBits_local = R_DlightTrisurf(((*surf).data) as *mut srfTriangles_t, dlightBits_local);
	} else {
		dlightBits_local = 0;
	}

	if dlightBits_local != 0 {
		(*tr).pc.c_dlightSurfaces += 1;
	}

	return dlightBits_local;
}

#[cfg(feature = "alt_automap_method")]
static mut tr_drawingAutoMap: bool = false;
static mut g_playerHeight: f32 = 0.0f;

/*
======================
R_AddWorldSurface
======================
*/
#[cfg(feature = "vv_lighting")]
pub unsafe fn R_AddWorldSurface(surf: *mut msurface_t, dlightBits: c_int, noViewCount: qboolean) {
	if noViewCount == 0 {
		if (*surf).viewCount == (*tr).viewCount {
			// already in this view, but lets make sure all the dlight bits are set
			if *(*surf).data == SF_FACE as c_int {
				let face = (*surf).data as *mut srfSurfaceFace_t;
				(*face).dlightBits |= dlightBits;
			}
			else if *(*surf).data == SF_GRID as c_int {
				let grid = (*surf).data as *mut srfGridMesh_t;
				(*grid).dlightBits |= dlightBits;
			}
			else if *(*surf).data == SF_TRIANGLES as c_int {
				let tri = (*surf).data as *mut srfTriangles_t;
				(*tri).dlightBits |= dlightBits;
			}
			return;
		}
		(*surf).viewCount = (*tr).viewCount;
		// FIXME: bmodel fog?
	}

	/*
	if (r_shadows->integer == 2)
	{
		dlightBits = R_DlightSurface( surf, dlightBits );
		//dlightBits = ( dlightBits != 0 );
		R_AddDrawSurf( surf->data, tr.shadowShader, surf->fogIndex, dlightBits );
	}
	*/
	//world shadows?

	// try to cull before dlighting or adding
	if R_CullSurface((*surf).data, (*surf).shader) != 0 {
		return;
	}

	// check for dlighting
	if dlightBits != 0 {
		let mut dlightBits_mut = dlightBits;
		dlightBits_mut = VVLightMan.R_DlightSurface(surf, dlightBits_mut);
		dlightBits_mut = if dlightBits_mut != 0 { 1 } else { 0 };
		R_AddDrawSurf((*surf).data, (*surf).shader, (*surf).fogIndex, dlightBits_mut);
	} else {
		R_AddDrawSurf((*surf).data, (*surf).shader, (*surf).fogIndex, dlightBits);
	}
}

#[cfg(not(feature = "vv_lighting"))]
unsafe fn R_AddWorldSurface(surf: *mut msurface_t, dlightBits: c_int, noViewCount: qboolean) {
	if noViewCount == 0 {
		if (*surf).viewCount == (*tr).viewCount {
			// already in this view, but lets make sure all the dlight bits are set
			if *(*surf).data == SF_FACE as c_int {
				let face = (*surf).data as *mut srfSurfaceFace_t;
				(*face).dlightBits |= dlightBits;
			}
			else if *(*surf).data == SF_GRID as c_int {
				let grid = (*surf).data as *mut srfGridMesh_t;
				(*grid).dlightBits |= dlightBits;
			}
			else if *(*surf).data == SF_TRIANGLES as c_int {
				let tri = (*surf).data as *mut srfTriangles_t;
				(*tri).dlightBits |= dlightBits;
			}
			return;
		}
		(*surf).viewCount = (*tr).viewCount;
		// FIXME: bmodel fog?
	}

	/*
	if (r_shadows->integer == 2)
	{
		dlightBits = R_DlightSurface( surf, dlightBits );
		//dlightBits = ( dlightBits != 0 );
		R_AddDrawSurf( surf->data, tr.shadowShader, surf->fogIndex, dlightBits );
	}
	*/
	//world shadows?

	// try to cull before dlighting or adding
	#[cfg(feature = "alt_automap_method")]
	{
		if !tr_drawingAutoMap && R_CullSurface((*surf).data, (*surf).shader) != 0 {
			return;
		}
	}
	#[cfg(not(feature = "alt_automap_method"))]
	{
		if R_CullSurface((*surf).data, (*surf).shader) != 0 {
			return;
		}
	}

	// check for dlighting
	if dlightBits != 0 {
		let mut dlightBits_mut = dlightBits;
		dlightBits_mut = R_DlightSurface(surf, dlightBits_mut);
		dlightBits_mut = if dlightBits_mut != 0 { 1 } else { 0 };
		R_AddDrawSurf((*surf).data, (*surf).shader, (*surf).fogIndex, dlightBits_mut);
	} else {
		#[cfg(feature = "alt_automap_method")]
		{
			if tr_drawingAutoMap {
				if *(*surf).data == SF_FACE as c_int {
					let mut completelyTransparent: bool = true;
					let mut i: c_int = 0;
					let face: *mut srfSurfaceFace_t = (*surf).data as *mut srfSurfaceFace_t;
					let indices: *mut u8 = ((face as *mut u8).add((*face).ofsIndices as usize)) as *mut u8;
					let mut point: *mut f32;
					let mut color: [f32; 3] = [0.0; 3];
					let mut alpha: f32;
					let mut e: f32;
					let mut polyStarted: bool = false;

					while i < (*face).numIndices {
						point = &mut (*face).points[*indices.add(i as usize) as usize][0];

						//base the color on the elevation... for now, just check the first point height
						if *point.add(2) < g_playerHeight {
							e = *point.add(2) - g_playerHeight;
						} else {
							e = g_playerHeight - *point.add(2);
						}
						if e < 0.0f {
							e = -e;
						}

						//set alpha and color based on relative height of point
						alpha = e / 256.0f;
						e /= 512.0f;

						//cap color
						if e > 1.0f {
							e = 1.0f;
						}
						else if e < 0.0f {
							e = 0.0f;
						}
						VectorSet(&mut color, e, 1.0f - e, 0.0f);

						//cap alpha
						if alpha > 1.0f {
							alpha = 1.0f;
						}
						else if alpha < 0.0f {
							alpha = 0.0f;
						}

						if alpha != 1.0f {
							// this point is not entirely alpha'd out, so still draw the surface
							completelyTransparent = false;
						}

						if !completelyTransparent {
							if !polyStarted {
								qglBegin(GL_POLYGON);
								polyStarted = true;
							}

							qglColor4f(color[0], color[1], color[2], 1.0f - alpha);
							qglVertex3f(*point, *point, *point.add(2));
						}

						i += 1;
					}

					if polyStarted {
						qglEnd();
					}
				}
			} else {
				R_AddDrawSurf((*surf).data, (*surf).shader, (*surf).fogIndex, dlightBits);
			}
		}
		#[cfg(not(feature = "alt_automap_method"))]
		{
			R_AddDrawSurf((*surf).data, (*surf).shader, (*surf).fogIndex, dlightBits);
		}
	}
}

/*
=============================================================

	BRUSH MODELS

=============================================================
*/

/*
=================
R_AddBrushModelSurfaces
=================
*/
pub unsafe fn R_AddBrushModelSurfaces(ent: *mut trRefEntity_t) {
	let bmodel: *mut bmodel_t;
	let clip: c_int;
	let pModel: *mut model_t;
	let mut i: c_int;

	pModel = R_GetModelByHandle((*ent).e.hModel);

	bmodel = (*pModel).bmodel;

	clip = R_CullLocalBox((*bmodel).bounds);
	if clip == CULL_OUT as c_int {
		return;
	}

	if (*pModel).bspInstance != 0
	{ //rwwRMG - added
		#[cfg(feature = "vv_lighting")]
		{
			VVLightMan.R_SetupEntityLighting(&mut (*tr).refdef, ent);
		}
		#[cfg(not(feature = "vv_lighting"))]
		{
			R_SetupEntityLighting(&mut (*tr).refdef, ent);
		}
	}

	//rww - Take this into account later?
//	if (!com_RMG || !com_RMG->integer)
//	{	// don't dlight bmodels on rmg, as multiple copies of the same instance will light up
	#[cfg(feature = "vv_lighting")]
	{
		VVLightMan.R_DlightBmodel(bmodel, false);
	}
	#[cfg(not(feature = "vv_lighting"))]
	{
		R_DlightBmodel(bmodel, false);
	}
//	}
//	else
//	{
//		R_DlightBmodel( bmodel, true );
//	}

	i = 0;
	while i < (*bmodel).numSurfaces {
		R_AddWorldSurface((*bmodel).firstSurface.add(i as usize) as *mut msurface_t, (*tr).currentEntity.dlightBits, qtrue);
		i += 1;
	}
}

fn GetQuadArea(v1: &[f32; 3], v2: &[f32; 3], v3: &[f32; 3], v4: &[f32; 3]) -> f32
{
	let mut vec1: [f32; 3] = [0.0; 3];
	let mut vec2: [f32; 3] = [0.0; 3];
	let mut dis1: [f32; 3] = [0.0; 3];
	let mut dis2: [f32; 3] = [0.0; 3];

	// Get area of tri1
	VectorSubtract(v1, v2, &mut vec1);
	VectorSubtract(v1, v4, &mut vec2);
	CrossProduct(&vec1, &vec2, &mut dis1);
	VectorScale(&mut dis1, 0.25f);

	// Get area of tri2
	VectorSubtract(v3, v2, &mut vec1);
	VectorSubtract(v3, v4, &mut vec2);
	CrossProduct(&vec1, &vec2, &mut dis2);
	VectorScale(&mut dis2, 0.25f);

	// Return addition of disSqr of each tri area
	return (dis1[0] * dis1[0] + dis1[1] * dis1[1] + dis1[2] * dis1[2] +
				dis2[0] * dis2[0] + dis2[1] * dis2[1] + dis2[2] * dis2[2]);
}

#[cfg(target_os = "windows")]
fn GetQuadArea_Short(v1: &[u16; 3], v2: &[u16; 3], v3: &[u16; 3], v4: &[u16; 3]) -> f32
{
	let mut fv1: [f32; 3] = [0.0; 3];
	let mut fv2: [f32; 3] = [0.0; 3];
	let mut fv3: [f32; 3] = [0.0; 3];
	let mut fv4: [f32; 3] = [0.0; 3];

	for i in 0..3 {
		unsafe {
			Q_CastShort2Float(&mut fv1[i], &v1[i] as *const u16 as *const i16);
			Q_CastShort2Float(&mut fv2[i], &v2[i] as *const u16 as *const i16);
			Q_CastShort2Float(&mut fv3[i], &v3[i] as *const u16 as *const i16);
			Q_CastShort2Float(&mut fv4[i], &v4[i] as *const u16 as *const i16);
		}
	}

	return GetQuadArea(&fv1, &fv2, &fv3, &fv4);
}

pub unsafe fn RE_GetBModelVerts(bmodelIndex: c_int, verts: *mut [f32; 3], normal: &[f32; 3])
{
	let mut surfs: *mut msurface_t;
	let mut face: *mut srfSurfaceFace_t;
	let bmodel: *mut bmodel_t;
	let pModel: *mut model_t;
	let mut i: c_int;
	//	Not sure if we really need to track the best two candidates
	let mut maxDist: [c_int; 2] = [0, 0];
	let mut maxIndx: [c_int; 2] = [0, 0];
	let mut dist: c_int = 0;
	let mut dot1: f32;
	let mut dot2: f32;

	pModel = R_GetModelByHandle(bmodelIndex);
	bmodel = (*pModel).bmodel;

	// Loop through all surfaces on the brush and find the best two candidates
	i = 0;
	while i < (*bmodel).numSurfaces {
		surfs = (*bmodel).firstSurface.add(i as usize) as *mut msurface_t;
		face = ((*surfs).data) as *mut srfSurfaceFace_t;

		// It seems that the safest way to handle this is by finding the area of the faces
		#[cfg(target_os = "windows")]
		{
			let nextSurfPoint = NEXT_SURFPOINT((*face).flags);
			let v1ptr = (*face).srfPoints as *const u16;
			let v2ptr = ((*face).srfPoints + nextSurfPoint) as *const u16;
			let v3ptr = ((*face).srfPoints + nextSurfPoint * 2) as *const u16;
			let v4ptr = ((*face).srfPoints + nextSurfPoint * 3) as *const u16;
			dist = GetQuadArea_Short(&*(v1ptr as *const [u16; 3]), &*(v2ptr as *const [u16; 3]), &*(v3ptr as *const [u16; 3]), &*(v4ptr as *const [u16; 3])) as c_int;
		}
		#[cfg(not(target_os = "windows"))]
		{
			dist = GetQuadArea(&(*face).points[0], &(*face).points[1], &(*face).points[2], &(*face).points[3]) as c_int;
		}

		// Check against the highest max
		if dist > maxDist[0] {
			// Shuffle our current maxes down
			maxDist[1] = maxDist[0];
			maxIndx[1] = maxIndx[0];

			maxDist[0] = dist;
			maxIndx[0] = i;
		}
		// Check against the second highest max
		else if dist >= maxDist[1] {
			// just stomp the old
			maxDist[1] = dist;
			maxIndx[1] = i;
		}
		i += 1;
	}

	// Hopefully we've found two best case candidates.  Now we should see which of these faces the viewer
	surfs = (*bmodel).firstSurface.add(maxIndx[0] as usize) as *mut msurface_t;
	face = ((*surfs).data) as *mut srfSurfaceFace_t;
	dot1 = DotProduct(&(*face).plane.normal, &(*tr).refdef.viewaxis[0]);

	surfs = (*bmodel).firstSurface.add(maxIndx[1] as usize) as *mut msurface_t;
	face = ((*surfs).data) as *mut srfSurfaceFace_t;
	dot2 = DotProduct(&(*face).plane.normal, &(*tr).refdef.viewaxis[0]);

	let mut idx: c_int;
	if dot2 < dot1 && dot2 < 0.0f {
		idx = maxIndx[1]; // use the second face
	}
	else if dot1 < dot2 && dot1 < 0.0f {
		idx = maxIndx[0]; // use the first face
	}
	else {
		// Possibly only have one face, so may as well use the first face, which also should be the best one
		//i = rand() & 1; // ugh, we don't know which to use.  I'd hope this would never happen
		idx = maxIndx[0]; // use the first face
	}

	surfs = (*bmodel).firstSurface.add(idx as usize) as *mut msurface_t;
	face = ((*surfs).data) as *mut srfSurfaceFace_t;

	#[cfg(target_os = "windows")]
	{
		let nextSurfPoint = NEXT_SURFPOINT((*face).flags);
		for t in 0..4 {
			Q_CastShort2Float(&mut (*verts)[t][0], ((*face).srfPoints + nextSurfPoint * t + 0) as *const i16);
			Q_CastShort2Float(&mut (*verts)[t][1], ((*face).srfPoints + nextSurfPoint * t + 1) as *const i16);
			Q_CastShort2Float(&mut (*verts)[t][2], ((*face).srfPoints + nextSurfPoint * t + 2) as *const i16);
		}
	}
	#[cfg(not(target_os = "windows"))]
	{
		for t in 0..4 {
			VectorCopy(&(*face).points[t], &mut (*verts)[t]);
		}
	}
}

/*
=============================================================

	WORLD MODEL

=============================================================
*/

/*
=============================================================
WIREFRAME AUTOMAP GENERATION SYSTEM - BEGIN
=============================================================
*/
#[cfg(not(feature = "alt_automap_method"))]
#[repr(C)]
struct wireframeSurfPoint_s {
	xyz: [f32; 3],
	alpha: f32,
	color: [f32; 3],
}

#[cfg(not(feature = "alt_automap_method"))]
type wireframeSurfPoint_t = wireframeSurfPoint_s;

#[cfg(not(feature = "alt_automap_method"))]
#[repr(C)]
struct wireframeMapSurf_s {
	completelyTransparent: bool,
	numPoints: c_int,
	points: *mut wireframeSurfPoint_t,
	next: *mut wireframeMapSurf_s,
}

#[cfg(not(feature = "alt_automap_method"))]
type wireframeMapSurf_t = wireframeMapSurf_s;

#[cfg(not(feature = "alt_automap_method"))]
#[repr(C)]
struct wireframeMap_s {
	surfs: *mut wireframeMapSurf_t,
}

#[cfg(not(feature = "alt_automap_method"))]
type wireframeMap_t = wireframeMap_s;

#[cfg(not(feature = "alt_automap_method"))]
static mut g_autoMapFrame: wireframeMap_t = wireframeMap_t { surfs: core::ptr::null_mut() };

#[cfg(not(feature = "alt_automap_method"))]
static mut g_autoMapNextFree: *mut *mut wireframeMapSurf_t = core::ptr::null_mut();

#[cfg(not(feature = "alt_automap_method"))]
static mut g_autoMapValid: bool = false; //set to true of g_autoMapFrame is valid.

//get the next available wireframe automap surface. -rww
#[cfg(not(feature = "alt_automap_method"))]
#[inline]
unsafe fn R_GetNewWireframeMapSurf() -> *mut wireframeMapSurf_t
{
	let mut next: *mut *mut wireframeMapSurf_t = &mut g_autoMapFrame.surfs;

	if !g_autoMapNextFree.is_null() {
		// save us the time of going through the entire linked list from root
		next = g_autoMapNextFree;
	}

	while !(*next).is_null() {
		// iterate through until we find the next unused one
		next = &mut (**next).next;
	}

	//allocate memory for it and pass it back
	*next = Z_Malloc(core::mem::size_of::<wireframeMapSurf_t>() as c_int, TAG_ALL as c_int, qtrue) as *mut wireframeMapSurf_t;
	g_autoMapNextFree = &mut (**next).next;
	return *next;
}

//evaluate a surface, see if it is valid for being part of the
//wireframe map render. -rww
#[cfg(all(not(feature = "alt_automap_method"), target_os = "windows"))]
#[inline]
unsafe fn R_EvaluateWireframeSurf(surf: *mut msurface_t)
{
	if *(*surf).data == SF_FACE as c_int {
		let face: *mut srfSurfaceFace_t = (*surf).data as *mut srfSurfaceFace_t;
		let numPoints: c_int = (*face).numPoints;
		let indices: *mut u8 = ((face as *mut u8).add((*face).ofsIndices as usize)) as *mut u8;

		if numPoints > 0 {
			// we can add it
			let mut i: c_int = 0;
			let nextSurf: *mut wireframeMapSurf_t = R_GetNewWireframeMapSurf();

			//now go through the indices and add a point for each
			(*nextSurf).points = Z_Malloc((core::mem::size_of::<wireframeSurfPoint_t>() as c_int) * (*face).numIndices, TAG_ALL as c_int, qtrue) as *mut wireframeSurfPoint_t;
			(*nextSurf).numPoints = (*face).numIndices;
			while i < (*face).numIndices {
				let mut point: [f32; 3] = [0.0; 3];
				Q_CastShort2Float(&mut point[0], ((*face).srfPoints as *const i16).add(*indices.add(i as usize) as usize + 0));
				Q_CastShort2Float(&mut point[1], ((*face).srfPoints as *const i16).add(*indices.add(i as usize) as usize + 1));
				Q_CastShort2Float(&mut point[2], ((*face).srfPoints as *const i16).add(*indices.add(i as usize) as usize + 2));
				VectorCopy(&point, &mut (*(*nextSurf).points.add(i as usize)).xyz);

				i += 1;
			}
		}
	}
	else if *(*surf).data == SF_TRIANGLES as c_int {
		//srfTriangles_t *surfTri = (srfTriangles_t *)surf->data;
		return; //not handled
	}
	else if *(*surf).data == SF_GRID as c_int {
		//srfGridMesh_t *gridMesh = (srfGridMesh_t *)surf->data;
		return; //not handled
	}
	else {
		// ...unknown type?
		return;
	}
}

#[cfg(all(not(feature = "alt_automap_method"), not(target_os = "windows")))]
#[inline]
unsafe fn R_EvaluateWireframeSurf(surf: *mut msurface_t)
{
	if *(*surf).data == SF_FACE as c_int {
		let face: *mut srfSurfaceFace_t = (*surf).data as *mut srfSurfaceFace_t;
		let points: *mut f32 = &mut (*face).points[0][0];
		let numPoints: c_int = (*face).numIndices;
		let indices: *mut c_int = ((surf as *mut u8).add((*face).ofsIndices as usize)) as *mut c_int;
		//byte *indices = (byte *)(face + face->ofsIndices);

		if !points.is_null() && numPoints > 0 {
			// we can add it
			let mut i: c_int = 0;
			let nextSurf: *mut wireframeMapSurf_t = R_GetNewWireframeMapSurf();

			//now go through the indices and add a point for each
			(*nextSurf).points = Z_Malloc((core::mem::size_of::<wireframeSurfPoint_t>() as c_int) * (*face).numIndices, TAG_ALL as c_int, qtrue) as *mut wireframeSurfPoint_t;
			(*nextSurf).numPoints = (*face).numIndices;
			while i < (*face).numIndices {
				let points_i: *mut f32 = &mut (*face).points[*indices.add(i as usize) as usize][0];
				VectorCopy(&*(points_i as *const [f32; 3]), &mut (*(*nextSurf).points.add(i as usize)).xyz);

				i += 1;
			}
		}
	}
	else if *(*surf).data == SF_TRIANGLES as c_int {
		//srfTriangles_t *surfTri = (srfTriangles_t *)surf->data;
		return; //not handled
	}
	else if *(*surf).data == SF_GRID as c_int {
		//srfGridMesh_t *gridMesh = (srfGridMesh_t *)surf->data;
		return; //not handled
	}
	else {
		// ...unknown type?
		return;
	}
}

//see if any surfaces on the node are facing opposite directions
//using plane normals. -rww
#[cfg(not(feature = "alt_automap_method"))]
#[inline]
unsafe fn R_NodeHasOppositeFaces(node: *mut mnode_t) -> bool
{
	let mut c: c_int;
	let mut d: c_int;
	let mut surf: *mut msurface_t;
	let mut surf2: *mut msurface_t;
	let mut mark: *mut *mut msurface_t;
	let mut mark2: *mut *mut msurface_t;
	let mut face: *mut srfSurfaceFace_t;
	let mut face2: *mut srfSurfaceFace_t;
	let mut normalDif: [f32; 3] = [0.0; 3];

	#[cfg(target_os = "windows")]
	{
		let leaf: *mut mleaf_s = node as *mut mleaf_s;
		mark = ((*tr.world).marksurfaces as *mut *mut msurface_t).add((*leaf).firstMarkSurfNum as usize);
		c = (*leaf).nummarksurfaces;
	}
	#[cfg(not(target_os = "windows"))]
	{
		mark = (*node).firstmarksurface;
		c = (*node).nummarksurfaces;
	}

	while c != 0 {
		c -= 1;
		surf = *mark;

		if *(*surf).data != SF_FACE as c_int {
			// if this node is not entirely comprised of faces, I guess we shouldn't check it?
			return false;
		}

		face = (*surf).data as *mut srfSurfaceFace_t;

		//go through other surfs and compare against this surf
		#[cfg(target_os = "windows")]
		{
			let leaf: *mut mleaf_s = node as *mut mleaf_s;
			d = (*leaf).nummarksurfaces;
			mark2 = ((*tr.world).marksurfaces as *mut *mut msurface_t).add((*leaf).firstMarkSurfNum as usize);
		}
		#[cfg(not(target_os = "windows"))]
		{
			d = (*node).nummarksurfaces;
			mark2 = (*node).firstmarksurface;
		}

		while d != 0 {
			d -= 1;
			surf2 = *mark2;

			if *(*surf2).data != SF_FACE as c_int {
				return false;
			}
			face2 = (*surf2).data as *mut srfSurfaceFace_t;
			//see if this normal has a drastic angular change
			VectorSubtract(&(*face).plane.normal, &(*face2).plane.normal, &mut normalDif);
			if VectorLength(&normalDif) >= 1.8f {
				return true;
			}

			mark2 = mark2.add(1);
		}
		mark = mark.add(1);
	}

	return false;
}

//recursively called for each node to go through the surfaces on that
//node and generate the wireframe map. -rww
#[cfg(not(feature = "alt_automap_method"))]
#[inline]
unsafe fn R_RecursiveWireframeSurf(node: *mut mnode_t)
{
	let mut c: c_int;
	let mut surf: *mut msurface_t;
	let mut mark: *mut *mut msurface_t;

	if node.is_null() {
		return;
	}

	loop {
		if node.is_null() || (*node).visframe != (*tr).visCount {
			// not valid, stop this chain of recursion
			return;
		}

		if (*node).contents != -1 {
			break;
		}

		R_RecursiveWireframeSurf((*node).children[0]);

		node = (*node).children[1];
	}

	// add the individual surfaces
	#[cfg(target_os = "windows")]
	{
		let leaf: *mut mleaf_s = node as *mut mleaf_s;
		mark = ((*tr.world).marksurfaces as *mut *mut msurface_t).add((*leaf).firstMarkSurfNum as usize);
		c = (*leaf).nummarksurfaces;
	}
	#[cfg(not(target_os = "windows"))]
	{
		mark = (*node).firstmarksurface;
		c = (*node).nummarksurfaces;
	}

	while c != 0 {
		c -= 1;
		// the surface may have already been added if it
		// spans multiple leafs
		surf = *mark;
		R_EvaluateWireframeSurf(surf);
		mark = mark.add(1);
	}
}

//generates a wireframe model of the map for the automap view -rww
#[cfg(not(feature = "alt_automap_method"))]
unsafe fn R_GenerateWireframeMap(baseNode: *mut mnode_t)
{
	let mut i: c_int;

	//initialize data to all 0
	g_autoMapFrame.surfs = core::ptr::null_mut();

	//take the hit for this frame, mark all of these things as visible
	//so we know which are valid for automap generation, but only the
	//ones that are facing outside the world! (well, ideally.)
	i = 0;
	while i < (*tr.world).numnodes {
		if (*tr.world).nodes[i as usize].contents != CONTENTS_SOLID as c_int {
			//if (!R_NodeHasOppositeFaces(&tr.world->nodes[i]))
			{
				(*tr.world).nodes[i as usize].visframe = (*tr).visCount;
			}
		}
		i += 1;
	}

	//now start the recursive evaluation
	R_RecursiveWireframeSurf(baseNode);
}

//clear out the wireframe map data -rww
#[cfg(not(feature = "alt_automap_method"))]
pub unsafe fn R_DestroyWireframeMap()
{
	let mut next: *mut wireframeMapSurf_t;
	let mut last: *mut wireframeMapSurf_t;

	if !g_autoMapValid {
		// not valid to begin with
		return;
	}

	next = g_autoMapFrame.surfs;
	while !next.is_null() {
		//free memory allocated for points on this surface
		Z_Free(next as *mut c_void);

		//get the next surface
		last = next;
		next = (*next).next;

		//free memory for this surface
		Z_Free(last as *mut c_void);
	}

	//invalidate everything
	g_autoMapFrame.surfs = core::ptr::null_mut();
	g_autoMapValid = false;
	g_autoMapNextFree = core::ptr::null_mut();
}

//save 3d automap data to file -rww
#[cfg(not(feature = "alt_automap_method"))]
pub unsafe fn R_WriteWireframeMapToFile() -> qboolean
{
	let f: fileHandle_t;
	let mut requiredSize: c_int = 0;
	let mut surf: *mut wireframeMapSurf_t = g_autoMapFrame.surfs;
	let out: *mut u8;
	let rOut: *mut u8;

	//let's go through and see how much space we're going to need to stuff all this
	//data into
	while !surf.is_null() {
		//memory for each point
		requiredSize += (core::mem::size_of::<wireframeSurfPoint_t>() as c_int) * (*surf).numPoints;

		//memory for numPoints
		requiredSize += core::mem::size_of::<c_int>() as c_int;

		surf = (*surf).next;
	}

	if requiredSize <= 0 {
		// nothing to do..?
		return qfalse;
	}


	f = FS_FOpenFileWrite(b"blahblah.bla\0".as_ptr() as *const c_char);
	if f == 0 {
		// can't create?
		return qfalse;
	}

	//allocate the memory we will need
	out = Z_Malloc(requiredSize as c_int, TAG_ALL as c_int, qtrue) as *mut u8;
	rOut = out;

	//now go through and put the data into the memory
	surf = g_autoMapFrame.surfs;
	while !surf.is_null() {
		core::ptr::copy_nonoverlapping(surf as *const u8, out, ((core::mem::size_of::<wireframeSurfPoint_t>() as c_int) * (*surf).numPoints + core::mem::size_of::<c_int>() as c_int) as usize);

		//memory for each point
		out = out.add((core::mem::size_of::<wireframeSurfPoint_t>() as c_int * (*surf).numPoints) as usize);

		//memory for numPoints
		out = out.add(core::mem::size_of::<c_int>());

		surf = (*surf).next;
	}

	//now write the buffer, and close
	FS_Write(rOut as *const c_void, requiredSize as c_int, f);
	Z_Free(rOut as *mut c_void);
	FS_FCloseFile(f);

	return qtrue;
}

//load 3d automap data from file -rww
#[cfg(not(feature = "alt_automap_method"))]
pub unsafe fn R_GetWireframeMapFromFile() -> qboolean
{
	let mut surfs: *mut wireframeMapSurf_t;
	let rSurfs: *mut wireframeMapSurf_t;
	let mut newSurf: *mut wireframeMapSurf_t;
	let f: fileHandle_t;
	let mut i: c_int = 0;
	let len: c_int;
	let mut stepBytes: c_int;

	len = FS_FOpenFileRead(b"blahblah.bla\0".as_ptr() as *const c_char, &mut core::ptr::null_mut(), qfalse);
	if len <= 0 {
		// it doesn't exist
		return qfalse;
	}

	surfs = Z_Malloc(len, TAG_ALL as c_int, qtrue) as *mut wireframeMapSurf_t;
	rSurfs = surfs;
	FS_Read(surfs as *mut c_void, len, f);

	while i < len {
		newSurf = R_GetNewWireframeMapSurf();
		(*newSurf).points = Z_Malloc((core::mem::size_of::<wireframeSurfPoint_t>() as c_int) * (*surfs).numPoints, TAG_ALL as c_int, qtrue) as *mut wireframeSurfPoint_t;

		//copy the surf data into the new surf
		//note - the surfs->points pointer is NOT pointing to valid memory, a pointer to that
		//pointer is actually what we want to use as the location of the point offsets.
		core::ptr::copy_nonoverlapping(&(*surfs).points as *const *mut wireframeSurfPoint_t as *const u8, (*newSurf).points as *mut u8, (core::mem::size_of::<wireframeSurfPoint_t>() as c_int * (*surfs).numPoints) as usize);
		(*newSurf).numPoints = (*surfs).numPoints;

		//the size of the point data, plus an int (the number of points)
		stepBytes = (core::mem::size_of::<wireframeSurfPoint_t>() as c_int * (*surfs).numPoints) + core::mem::size_of::<c_int>() as c_int;
		i += stepBytes;

		//increment the pointer to the start of the next surface
		surfs = ((surfs as *mut u8).add(stepBytes as usize)) as *mut wireframeMapSurf_t;
	}

	//it should end up being equal, if not something was wrong with this file.
	debug_assert_eq!(i, len);

	FS_FCloseFile(f);
	Z_Free(rSurfs as *mut c_void);
	return qtrue;
}

//create everything, after destroying any existing data -rww
#[cfg(not(feature = "alt_automap_method"))]
pub unsafe fn R_InitializeWireframeAutomap() -> qboolean
{
	if !r_autoMapDisable.is_null() && (*r_autoMapDisable).integer != 0 {
		return qfalse;
	}

	if !(*tr).world.is_null() && !(*(*tr).world).nodes.is_null() {
		R_DestroyWireframeMap();
		R_GenerateWireframeMap((*(*tr).world).nodes);
		g_autoMapValid = true;
	}

	return if g_autoMapValid { qtrue } else { qfalse };
}

/*
=============================================================
WIREFRAME AUTOMAP GENERATION SYSTEM - END
=============================================================
*/

pub unsafe fn R_AutomapElevationAdjustment(newHeight: f32)
{
	g_playerHeight = newHeight;
}

#[cfg(feature = "alt_automap_method")]
//adjust the player height for gradient elevation colors -rww
pub unsafe fn R_InitializeWireframeAutomap() -> qboolean
{ //yoink
	return qtrue;
}

//draw the automap with the given transformation matrix -rww
const QUADINFINITY: f32 = 16777216.0f;
static mut g_lastHeight: f32 = 0.0f;
static mut g_lastHeightValid: bool = false;
extern "C" {
	fn R_RecursiveWorldNode(node: *mut mnode_t, planeBits: c_int, dlightBits: c_int);
}

pub unsafe fn R_DrawWireframeAutomap(data: *const c_void) -> *const c_void
{
	let cmd: *const drawBufferCommand_t = data as *const drawBufferCommand_t;
	let mut e: f32 = 0.0f;
	let mut alpha: f32;
	let mut s: *mut wireframeMapSurf_t;
	#[cfg(not(feature = "alt_automap_method"))]
	let mut i: c_int;

	#[cfg(not(feature = "alt_automap_method"))]
	{
		s = g_autoMapFrame.surfs;
	}
	#[cfg(feature = "alt_automap_method")]
	{
		s = core::ptr::null_mut();
	}

	if r_autoMap.is_null() || (*r_autoMap).integer == 0 {
		return (cmd as *mut drawBufferCommand_t).add(1) as *const c_void;
	}

	#[cfg(not(feature = "alt_automap_method"))]
	{
		if !g_autoMapValid {
			// data is not valid, don't draw
			return (cmd as *mut drawBufferCommand_t).add(1) as *const c_void;
		}
	}

	//disable 2d texturing
	qglDisable(GL_TEXTURE_2D);

	//now draw the backdrop
	{
		alpha = 1.0f;
		GL_State(0);
	}
	//black
	qglColor4f(0.0f, 0.0f, 0.0f, alpha);

	//draw a black backdrop
	qglPushMatrix();
	qglLoadIdentity(); //get the ident matrix

	qglBegin(GL_QUADS);
	qglVertex3f(-QUADINFINITY, QUADINFINITY, -((*backEnd).viewParms.zFar - 1.0f));
	qglVertex3f(QUADINFINITY, QUADINFINITY, -((*backEnd).viewParms.zFar - 1.0f));
	qglVertex3f(QUADINFINITY, -QUADINFINITY, -((*backEnd).viewParms.zFar - 1.0f));
	qglVertex3f(-QUADINFINITY, -QUADINFINITY, -((*backEnd).viewParms.zFar - 1.0f));
	qglEnd();

	//pop back the viewmatrix
	qglPopMatrix();


	//set the mode to line draw
	if (*r_autoMap).integer == 2 {
		// line mode
		GL_State((GLS_POLYMODE_LINE | GLS_SRCBLEND_SRC_ALPHA | GLS_DSTBLEND_SRC_COLOR | GLS_DEPTHMASK_TRUE) as c_int);
	}
	else {
		// fill mode
		GL_State(GLS_DEPTHMASK_TRUE as c_int);
	}

	//set culling
	GL_Cull(CT_TWO_SIDED as c_int);

	#[cfg(not(feature = "alt_automap_method"))]
	{
		//Draw the triangles
		while !s.is_null() {
			//first, loop through and set the alpha on every point for this surf.
			//if the alpha ends up being completely transparent for every point, we don't even
			//need to draw it
			if g_playerHeight != g_lastHeight || !g_lastHeightValid {
				// only do this if we need to
				i = 0;
				(*s).completelyTransparent = true;
				while i < (*s).numPoints {
					//base the color on the elevation... for now, just check the first point height
					if (*s).points[i as usize].xyz[2] < g_playerHeight {
						e = (*s).points[i as usize].xyz[2] - g_playerHeight;
					}
					else {
						e = g_playerHeight - (*s).points[i as usize].xyz[2];
					}
					if e < 0.0f {
						e = -e;
					}

					if (*r_autoMap).integer != 2 {
						// fill mode
						if (*s).points[i as usize].xyz[2] > (g_playerHeight + 64.0f) {
							(*s).points[i as usize].alpha = 1.0f;
						}
						else {
							(*s).points[i as usize].alpha = e / 256.0f;
						}
					}
					else {
						//set alpha and color based on relative height of point
						(*s).points[i as usize].alpha = e / 256.0f;
					}
					e /= 512.0f;

					//cap color
					if e > 1.0f {
						e = 1.0f;
					}
					else if e < 0.0f {
						e = 0.0f;
					}
					VectorSet(&mut (*s).points[i as usize].color, e, 1.0f - e, 0.0f);

					//cap alpha
					if (*s).points[i as usize].alpha > 1.0f {
						(*s).points[i as usize].alpha = 1.0f;
					}
					else if (*s).points[i as usize].alpha < 0.0f {
						(*s).points[i as usize].alpha = 0.0f;
					}

					if (*s).points[i as usize].alpha != 1.0f {
						// this point is not entirely alpha'd out, so still draw the surface
						(*s).completelyTransparent = false;
					}

					i += 1;
				}
			}

			if (*s).completelyTransparent {
				s = (*s).next;
				continue;
			}

			i = 0;
			qglBegin(GL_TRIANGLES);
			while i < (*s).numPoints {
				if (*r_autoMap).integer == 2 || (*s).numPoints < 3 {
					// line mode or not enough verts on surface
					qglColor4f((*s).points[i as usize].color[0], (*s).points[i as usize].color[1], (*s).points[i as usize].color[2], (*s).points[i as usize].alpha);
				}
				else {
					// fill mode
					let mut planeNormal: [f32; 3] = [0.0; 3];
					let fAlpha: f32 = (*s).points[i as usize].alpha;
					planeNormal[0] = (*s).points[0].xyz[1] * ((*s).points[1].xyz[2] - (*s).points[2].xyz[2]) + (*s).points[1].xyz[1] * ((*s).points[2].xyz[2] - (*s).points[0].xyz[2]) + (*s).points[2].xyz[1] * ((*s).points[0].xyz[2] - (*s).points[1].xyz[2]);
					planeNormal[1] = (*s).points[0].xyz[2] * ((*s).points[1].xyz[0] - (*s).points[2].xyz[0]) + (*s).points[1].xyz[2] * ((*s).points[2].xyz[0] - (*s).points[0].xyz[0]) + (*s).points[2].xyz[2] * ((*s).points[0].xyz[0] - (*s).points[1].xyz[0]);
					planeNormal[2] = (*s).points[0].xyz[0] * ((*s).points[1].xyz[1] - (*s).points[2].xyz[1]) + (*s).points[1].xyz[0] * ((*s).points[2].xyz[1] - (*s).points[0].xyz[1]) + (*s).points[2].xyz[0] * ((*s).points[0].xyz[1] - (*s).points[1].xyz[1]);

					if planeNormal[0] < 0.0f { planeNormal[0] = -planeNormal[0]; }
					if planeNormal[1] < 0.0f { planeNormal[1] = -planeNormal[1]; }
					if planeNormal[2] < 0.0f { planeNormal[2] = -planeNormal[2]; }

					//qglColor4f(planeNormal[0], planeNormal[1], planeNormal[2], fAlpha);
					qglColor4f((*s).points[i as usize].color[0], (*s).points[i as usize].color[1], 1.0f - planeNormal[2], fAlpha);
				}
				qglVertex3f((*s).points[i as usize].xyz[0], (*s).points[i as usize].xyz[1], (*s).points[i as usize].xyz[2]);
				i += 1;
			}
			qglEnd();
			s = (*s).next;
		}
	}
	#[cfg(feature = "alt_automap_method")]
	{
		tr_drawingAutoMap = true;
		R_RecursiveWorldNode((*tr).world.nodes, 15, 0);
		tr_drawingAutoMap = false;
	}

	g_lastHeight = g_playerHeight;
	g_lastHeightValid = true;

	//reenable 2d texturing
	qglEnable(GL_TEXTURE_2D);

	//white color/full alpha
	qglColor4f(1.0f, 1.0f, 1.0f, 1.0f);

	return (cmd as *mut drawBufferCommand_t).add(1) as *const c_void;
}


/*
================
R_RecursiveWorldNode
================
*/
#[cfg(not(feature = "vv_lighting"))]
pub unsafe fn R_RecursiveWorldNode_Def(node: *mut mnode_t, mut planeBits: c_int, mut dlightBits: c_int) {

	loop {
		let mut newDlights: [c_int; 2] = [0, 0];

		#[cfg(feature = "alt_automap_method")]
		{
			if tr_drawingAutoMap {
				(*node).visframe = (*tr).visCount;
			}
		}

		// if the node wasn't marked as potentially visible, exit
		if (*node).visframe != (*tr).visCount {
			return;
		}

		// if the bounding volume is outside the frustum, nothing
		// inside can be visible OPTIMIZE: don't do this all the way to leafs?

		#[cfg(feature = "alt_automap_method")]
		{
			if (*r_nocull).integer != 1 && !tr_drawingAutoMap {
				let mut r: c_int;

				if (planeBits & 1) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[0]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !1;			// all descendants will also be in front
					}
				}

				if (planeBits & 2) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[1]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !2;			// all descendants will also be in front
					}
				}

				if (planeBits & 4) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[2]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !4;			// all descendants will also be in front
					}
				}

				if (planeBits & 8) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[3]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !8;			// all descendants will also be in front
					}
				}
			}
		}
		#[cfg(not(feature = "alt_automap_method"))]
		{
			if (*r_nocull).integer != 1 {
				let mut r: c_int;

				if (planeBits & 1) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[0]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !1;			// all descendants will also be in front
					}
				}

				if (planeBits & 2) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[1]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !2;			// all descendants will also be in front
					}
				}

				if (planeBits & 4) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[2]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !4;			// all descendants will also be in front
					}
				}

				if (planeBits & 8) != 0 {
					r = BoxOnPlaneSide((*node).mins, (*node).maxs, &(*tr).viewParms.frustum[3]);
					if r == 2 {
						return;						// culled
					}
					if r == 1 {
						planeBits &= !8;			// all descendants will also be in front
					}
				}
			}
		}

		if (*node).contents != -1 {
			break;
		}

		// node is just a decision point, so go down both sides
		// since we don't care about sort orders, just go positive to negative

		// determine which dlights are needed
		if (*r_nocull).integer != 2 {
			newDlights[0] = 0;
			newDlights[1] = 0;
			if dlightBits != 0 {
				let mut i: c_int;
				i = 0;
				while i < (*tr).refdef.num_dlights {
					let dl: *mut dlight_t;
					let dist: f32;

					if (dlightBits & (1 << i)) != 0 {
						dl = &mut (*tr).refdef.dlights[i as usize];
						dist = DotProduct(&(*dl).origin, &(*node).plane.normal) - (*node).plane.dist;

						if dist > -(*dl).radius {
							newDlights[0] |= (1 << i);
						}
						if dist < (*dl).radius {
							newDlights[1] |= (1 << i);
						}
					}
					i += 1;
				}
			}
		}
		else {
			newDlights[0] = dlightBits;
			newDlights[1] = dlightBits;
		}

		// recurse down the children, front side first
		R_RecursiveWorldNode_Def((*node).children[0], planeBits, newDlights[0]);

		// tail recurse
		node = (*node).children[1];
		dlightBits = newDlights[1];
	}

	{
		// leaf node, so add mark surfaces
		let mut c: c_int;
		let mut surf: *mut msurface_t;
		let mut mark: *mut *mut msurface_t;

		(*tr).pc.c_leafs += 1;

		// add to z buffer bounds
		if (*node).mins[0] < (*tr).viewParms.visBounds[0][0] {
			(*tr).viewParms.visBounds[0][0] = (*node).mins[0];
		}
		if (*node).mins[1] < (*tr).viewParms.visBounds[0][1] {
			(*tr).viewParms.visBounds[0][1] = (*node).mins[1];
		}
		if (*node).mins[2] < (*tr).viewParms.visBounds[0][2] {
			(*tr).viewParms.visBounds[0][2] = (*node).mins[2];
		}

		if (*node).maxs[0] > (*tr).viewParms.visBounds[1][0] {
			(*tr).viewParms.visBounds[1][0] = (*node).maxs[0];
		}
		if (*node).maxs[1] > (*tr).viewParms.visBounds[1][1] {
			(*tr).viewParms.visBounds[1][1] = (*node).maxs[1];
		}
		if (*node).maxs[2] > (*tr).viewParms.visBounds[1][2] {
			(*tr).viewParms.visBounds[1][2] = (*node).maxs[2];
		}

		// add the individual surfaces
		mark = (*node).firstmarksurface;
		c = (*node).nummarksurfaces;
		while c != 0 {
			c -= 1;
			// the surface may have already been added if it
			// spans multiple leafs
			surf = *mark;
			R_AddWorldSurface(surf, dlightBits, qfalse);
			mark = mark.add(1);
		}
	}
}

/*
===============
R_PointInLeaf
===============
*/
unsafe fn R_PointInLeaf(p: &[f32; 3]) -> *mut mnode_t {
	let mut node: *mut mnode_t;
	let mut d: f32;
	let plane: *mut cplane_t;

	if (*tr).world.is_null() {
		Com_Error(ERR_DROP as c_int, b"R_PointInLeaf: bad model\0".as_ptr() as *const c_char);
	}

	node = (*(*tr).world).nodes;
	loop {
		if (*node).contents != -1 {
			break;
		}
		#[cfg(target_os = "windows")]
		{
			plane = ((*(*tr).world).planes as *mut cplane_t).add((*node).planeNum as usize);
		}
		#[cfg(not(target_os = "windows"))]
		{
			plane = (*node).plane;
		}
		d = DotProduct(p, &(*plane).normal) - (*plane).dist;
		if d > 0.0f {
			node = (*node).children[0];
		} else {
			node = (*node).children[1];
		}
	}

	return node;
}

/*
==============
R_ClusterPVS
==============
*/
unsafe fn R_ClusterPVS(cluster: c_int) -> *const u8 {
	if (*tr).world.is_null() || (*(*tr).world).vis.is_null() || cluster < 0 || cluster >= (*(*tr).world).numClusters {
		return (*(*tr).world).novis;
	}

	#[cfg(target_os = "windows")]
	{
		return (*(*(*tr).world).vis).Decompress((cluster * (*(*tr).world).clusterBytes) as c_int, (*(*tr).world).numClusters);
	}
	#[cfg(not(target_os = "windows"))]
	{
		return ((*(*tr).world).vis as *const u8).add((cluster * (*(*tr).world).clusterBytes) as usize);
	}
}

/*
=================
R_inPVS
=================
*/
pub unsafe fn R_inPVS(p1: &[f32; 3], p2: &[f32; 3], mask: *mut u8) -> qboolean {
	let mut leafnum: c_int;
	let mut cluster: c_int;
	let mut area1: c_int;
	let mut area2: c_int;
	let mut mask_local: *mut u8;

	leafnum = CM_PointLeafnum(p1);
	cluster = CM_LeafCluster(leafnum);
	area1 = CM_LeafArea(leafnum);

	//agh, the damn snapshot mask doesn't work for this
	mask_local = CM_ClusterPVS(cluster) as *mut u8;

	leafnum = CM_PointLeafnum(p2);
	cluster = CM_LeafCluster(leafnum);
	area2 = CM_LeafArea(leafnum);
	if !mask_local.is_null() && ((*mask_local.add(cluster as usize >> 3) & (1 << (cluster & 7))) == 0) {
		return qfalse;
	}
	//this doesn't freakin work
//	if (!CM_AreasConnected (area1, area2))
//		return qfalse;		// a door blocks sight
	return qtrue;
}

/*
===============
R_MarkLeaves

Mark the leaves and nodes that are in the PVS for the current
cluster
===============
*/
#[cfg(target_os = "windows")]
pub unsafe fn R_MarkLeaves(leafOverride: *mut mleaf_s) {
	let vis: *const u8;
	let mut leaf: *mut mleaf_s;
	let mut parent: *mut mnode_s;
	let mut i: c_int;
	let mut cluster: c_int;

	// lockpvs lets designers walk around to determine the
	// extent of the current pvs
	if (*r_lockpvs).integer != 0 {
		return;
	}

	// current viewcluster
	if leafOverride.is_null() {
		leaf = R_PointInLeaf(&(*tr).viewParms.pvsOrigin) as *mut mleaf_s;
	} else {
		leaf = leafOverride;
	}
	cluster = (*leaf).cluster;

	debug_assert_ne!((*leaf).contents, -1);

	// if the cluster is the same and the area visibility matrix
	// hasn't changed, we don't need to mark everything again

	if (*tr).viewCluster == cluster && (*tr).refdef.areamaskModified == 0 {
		return;
	}

	(*tr).visCount += 1;
	(*tr).viewCluster = cluster;

	if (*r_novis).integer != 0 || (*tr).viewCluster == -1 {
		i = 0;
		while i < (*(*tr).world).numnodes {
			if (*(*tr).world).nodes[i as usize].contents != CONTENTS_SOLID as c_int {
				(*(*tr).world).nodes[i as usize].visframe = (*tr).visCount;
			}
			i += 1;
		}
		return;
	}

	vis = R_ClusterPVS((*tr).viewCluster);

	i = 0;
	leaf = (*(*tr).world).leafs;
	while i < (*(*tr).world).numleafs {
		cluster = (*leaf).cluster;
		if cluster < 0 || cluster >= (*(*tr).world).numClusters {
			i += 1;
			leaf = leaf.add(1);
			continue;
		}

		// check general pvs
		if (*(vis.add(cluster as usize >> 3)) & (1 << (cluster & 7))) == 0 {
			i += 1;
			leaf = leaf.add(1);
			continue;
		}

		// check for door connection
		if !lookingForWorstLeaf && ((*(*tr).refdef.areamask.add((*leaf).area as usize >> 3)) & (1 << ((*leaf).area & 7))) != 0 {
			i += 1;
			leaf = leaf.add(1);
			continue;		// not visible
		}

		parent = leaf as *mut mnode_s;
		debug_assert_ne!((*leaf).contents, -1);
		loop {
			if (*parent).visframe == (*tr).visCount {
				break;
			}
			(*parent).visframe = (*tr).visCount;
			parent = (*parent).parent;
			if parent.is_null() {
				break;
			}
		}
		i += 1;
		leaf = leaf.add(1);
	}
}

#[cfg(not(target_os = "windows"))]
unsafe fn R_MarkLeaves() {
	let vis: *const u8;
	let mut leaf: *mut mnode_t;
	let mut parent: *mut mnode_t;
	let mut i: c_int;
	let mut cluster: c_int;

	// lockpvs lets designers walk around to determine the
	// extent of the current pvs
	if (*r_lockpvs).integer != 0 {
		return;
	}

	// current viewcluster
	leaf = R_PointInLeaf(&(*tr).viewParms.pvsOrigin);
	cluster = (*leaf).cluster;

	// if the cluster is the same and the area visibility matrix
	// hasn't changed, we don't need to mark everything again

	// if r_showcluster was just turned on, remark everything
	if (*tr).viewCluster == cluster && (*tr).refdef.areamaskModified == 0
		&& (*r_showcluster).modified == 0 {
		return;
	}

	if (*r_showcluster).modified != 0 || (*r_showcluster).integer != 0 {
		(*r_showcluster).modified = qfalse;
		if (*r_showcluster).integer != 0 {
			Com_Printf(b"cluster:%i  area:%i\n\0".as_ptr() as *const c_char, cluster, (*leaf).area);
		}
	}

	(*tr).visCount += 1;
	(*tr).viewCluster = cluster;

	if (*r_novis).integer != 0 || (*tr).viewCluster == -1 {
		i = 0;
		while i < (*(*tr).world).numnodes {
			if (*(*tr).world).nodes[i as usize].contents != CONTENTS_SOLID as c_int {
				(*(*tr).world).nodes[i as usize].visframe = (*tr).visCount;
			}
			i += 1;
		}
		return;
	}

	vis = R_ClusterPVS((*tr).viewCluster);

	i = 0;
	leaf = (*(*tr).world).nodes;
	while i < (*(*tr).world).numnodes {
		cluster = (*leaf).cluster;
		if cluster < 0 || cluster >= (*(*tr).world).numClusters {
			i += 1;
			leaf = leaf.add(1);
			continue;
		}

		// check general pvs
		if (*(vis.add(cluster as usize >> 3)) & (1 << (cluster & 7))) == 0 {
			i += 1;
			leaf = leaf.add(1);
			continue;
		}

		// check for door connection
		if (*((*tr).refdef.areamask.add((*leaf).area as usize >> 3)) & (1 << ((*leaf).area & 7))) != 0 {
			i += 1;
			leaf = leaf.add(1);
			continue;		// not visible
		}

		parent = leaf;
		loop {
			if (*parent).visframe == (*tr).visCount {
				break;
			}
			(*parent).visframe = (*tr).visCount;
			parent = (*parent).parent;
			if parent.is_null() {
				break;
			}
		}
		i += 1;
		leaf = leaf.add(1);
	}
}

/*
=============
R_AddWorldSurfaces
=============
*/
#[cfg(target_os = "windows")]
pub unsafe fn R_AddWorldSurfaces() {
	if (*r_drawworld).integer == 0 {
		return;
	}

	if ((*tr).refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
		return;
	}

	(*tr).currentEntityNum = TR_WORLDENT as c_int;
	(*tr).shiftedEntityNum = (*tr).currentEntityNum << QSORT_ENTITYNUM_SHIFT;

	// clear out the visible min/max
	ClearBounds(&mut (*tr).viewParms.visBounds[0], &mut (*tr).viewParms.visBounds[1]);

	// perform frustum culling and add all the potentially visible surfaces
	if VVLightMan.num_dlights > MAX_DLIGHTS as c_int {
		VVLightMan.num_dlights = MAX_DLIGHTS as c_int;
	}

	VVLightMan.R_RecursiveWorldNode((*(*tr).world).nodes, 15, (1 << VVLightMan.num_dlights) - 1);
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn R_AddWorldSurfaces() {
	if (*r_drawworld).integer == 0 {
		return;
	}

	if ((*tr).refdef.rdflags & RDF_NOWORLDMODEL) != 0 {
		return;
	}

	(*tr).currentEntityNum = TR_WORLDENT as c_int;
	(*tr).shiftedEntityNum = (*tr).currentEntityNum << QSORT_ENTITYNUM_SHIFT;

	// determine which leaves are in the PVS / areamask
	R_MarkLeaves();

	// clear out the visible min/max
	ClearBounds(&mut (*tr).viewParms.visBounds[0], &mut (*tr).viewParms.visBounds[1]);

	// perform frustum culling and add all the potentially visible surfaces
	if (*tr).refdef.num_dlights > 32 {
		(*tr).refdef.num_dlights = 32;
	}

	R_RecursiveWorldNode_Def((*(*tr).world).nodes, 15, (1 << (*tr).refdef.num_dlights) - 1);
}
