// tr_flares.c
//Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::addr_of_mut;

/*
=============================================================================

LIGHT FLARES

A light flare is an effect that takes place inside the eye when bright light
sources are visible.  The size of the flare reletive to the screen is nearly
constant, irrespective of distance, but the intensity should be proportional to the
projected area of the light source.

A surface that has been flagged as having a light flare will calculate the depth
buffer value that it's midpoint should have when the surface is added.

After all opaque surfaces have been rendered, the depth buffer is read back for
each flare in view.  If the point has not been obscured by a closer surface, the
flare should be drawn.

Surfaces that have a repeated texture should never be flagged as flaring, because
there will only be a single flare added at the midpoint of the polygon.

To prevent abrupt popping, the intensity of the flare is interpolated up and
down as it changes visibility.  This involves scene to scene state, unlike almost
all other aspects of the renderer, and is complicated by the fact that a single
frame may have multiple scenes.

RB_RenderFlares() will be called once per view (twice in a mirrored scene, potentially
up to five or more times in a frame with 3D status bar icons).

=============================================================================
*/

// Type aliases
pub type vec3_t = [f32; 3];
pub type vec4_t = [f32; 4];
pub type qboolean = c_int;

const QTRUE: c_int = 1;
const QFALSE: c_int = 0;

// flare states maintain visibility over multiple frames for fading
// layers: view, mirror, menu
#[repr(C)]
pub struct flare_s {
	pub next: *mut flare_s,		// for active chain

	pub addedFrame: c_int,

	pub inPortal: qboolean,				// true if in a portal view of the scene
	pub frameSceneNum: c_int,
	pub surface: *mut c_void,
	pub fogNum: c_int,

	pub fadeTime: c_int,

	pub visible: qboolean,			// state of last test
	pub drawIntensity: f32,		// may be non 0 even if !visible due to fading

	pub windowX: c_int,
	pub windowY: c_int,
	pub eyeZ: f32,

	pub color: vec3_t,
}

pub type flare_t = flare_s;

const MAX_FLARES: usize = 128;

// Local type stubs for structural coherence
#[repr(C)]
pub struct performanceCounters_t {
	pub c_flareAdds: c_int,
	pub c_flareTests: c_int,
	pub c_flareRenders: c_int,
	// Placeholder for other counter fields
}

#[repr(C)]
pub struct orientationr_t {
	pub origin: vec3_t,
	pub axis: [[f32; 3]; 3],
}

#[repr(C)]
pub struct viewParms_t {
	pub viewportX: c_int,
	pub viewportY: c_int,
	pub viewportWidth: c_int,
	pub viewportHeight: c_int,
	pub frameCount: c_int,
	pub frameSceneNum: c_int,
	pub isPortal: qboolean,
	pub ori: orientationr_t,
	pub projectionMatrix: [f32; 16],
	// Placeholder for other fields
}

#[repr(C)]
pub struct refdef_t {
	pub time: c_int,
	pub dlights: *mut dlight_t,
	pub num_dlights: c_int,
	// Placeholder for other fields
}

#[repr(C)]
pub struct backEnd_t {
	pub pc: performanceCounters_t,
	pub ori: orientationr_t,
	pub viewParms: viewParms_t,
	pub refdef: refdef_t,
	// Placeholder for other fields
}

#[repr(C)]
pub struct glState_t {
	pub finishCalled: c_int,
	// Placeholder for actual glState structure
}

#[repr(C)]
pub struct cvar_t {
	pub value: f32,
	pub integer: c_int,
}

#[repr(C)]
pub struct fog_t {
	pub bounds: [[f32; 3]; 2],
	// Placeholder for other fields
}

#[repr(C)]
pub struct worldData_t {
	pub fogs: *mut fog_t,
	pub numfogs: c_int,
	// Placeholder for other fields
}

#[repr(C)]
pub struct shader_t {
	// Placeholder
}

#[repr(C)]
pub struct tr_t {
	pub world: *mut worldData_t,
	pub flareShader: *mut shader_t,
	pub identityLight: f32,
	// Placeholder for other fields
}

#[repr(C)]
pub struct dlight_t {
	pub origin: vec3_t,
	pub transformed: vec3_t,
	pub radius: f32,
	pub color: vec3_t,
}

#[repr(C)]
pub struct drawVert_t {
	pub xyz: vec3_t,
	pub st: [f32; 2],
	pub lightmap: [f32; 2],
	pub normal: vec3_t,
	pub vertexColors: [u8; 4],
}

#[repr(C)]
pub struct tessellator_t {
	pub xyz: [[f32; 4]; 4096],
	pub texCoords: [[[f32; 2]; 2]; 4096],
	pub vertexColors: [[c_int; 4]; 4096],
	pub indexes: [c_int; 6144],
	pub numVertexes: c_int,
	pub numIndexes: c_int,
	// Placeholder for other fields
}

// GL Constants
const GL_DEPTH_COMPONENT: c_int = 0x1902;
const GL_FLOAT: c_int = 0x1406;
const GL_CLIP_PLANE0: c_int = 0x3000;
const GL_PROJECTION: c_int = 0x1701;
const GL_MODELVIEW: c_int = 0x1700;

// External globals
extern "C" {
	pub static mut r_flareStructs: [flare_t; MAX_FLARES];
	pub static mut r_activeFlares: *mut flare_t;
	pub static mut r_inactiveFlares: *mut flare_t;
	pub static mut backEnd: backEnd_t;
	pub static mut glState: glState_t;
	pub static mut tr: tr_t;
	pub static r_flares: *const cvar_t;
	pub static r_flareFade: *const cvar_t;
	pub static r_flareSize: *const cvar_t;
	pub static mut tess: tessellator_t;

	// Functions
	fn Com_Memset(dest: *mut c_void, c: c_int, count: usize);
	fn R_TransformModelToClip(
		src: *const vec3_t,
		modelMatrix: *const f32,
		projectionMatrix: *const f32,
		eye: *mut vec4_t,
		dst: *mut vec4_t,
	);
	fn R_TransformClipToWindow(
		clip: *const vec4_t,
		viewParms: *const viewParms_t,
		normalized: *mut vec4_t,
		window: *mut vec4_t,
	);
	fn VectorCopy(src: vec3_t, dst: *mut vec3_t);
	fn VectorSubtract(a: vec3_t, b: vec3_t, dst: *mut vec3_t);
	fn VectorNormalizeFast(v: *mut vec3_t) -> f32;
	fn DotProduct(a: vec3_t, b: vec3_t) -> f32;
	fn VectorScale(v: vec3_t, scale: f32, dst: *mut vec3_t);
	fn RB_BeginSurface(shader: *mut shader_t, fogNum: c_int);
	fn RB_EndSurface();
	fn qglReadPixels(
		x: c_int,
		y: c_int,
		width: c_int,
		height: c_int,
		format: c_int,
		type_: c_int,
		pixels: *mut c_void,
	);
	fn qglDisable(cap: c_int);
	fn qglPushMatrix();
	fn qglLoadIdentity();
	fn qglMatrixMode(mode: c_int);
	fn qglOrtho(
		left: f32,
		right: f32,
		bottom: f32,
		top: f32,
		zNear: f32,
		zFar: f32,
	);
	fn qglPopMatrix();
}

/*
==================
R_ClearFlares
==================
*/
pub unsafe fn R_ClearFlares() {
	let mut i: c_int;

	Com_Memset(
		addr_of_mut!(r_flareStructs) as *mut c_void,
		0,
		core::mem::size_of_val(&r_flareStructs),
	);
	r_activeFlares = core::ptr::null_mut();
	r_inactiveFlares = core::ptr::null_mut();

	i = 0;
	while i < MAX_FLARES as c_int {
		(*addr_of_mut!(r_flareStructs)[i as usize]).next = r_inactiveFlares;
		r_inactiveFlares = &mut r_flareStructs[i as usize];
		i += 1;
	}
}


/*
==================
RB_AddFlare

This is called at surface tesselation time
==================
*/
pub unsafe fn RB_AddFlare(
	surface: *mut c_void,
	fogNum: c_int,
	point: vec3_t,
	color: vec3_t,
	normal: *mut vec3_t,
) {
	let mut i: c_int;
	let mut f: *mut flare_t;
	let mut oldest: *mut flare_t;
	let mut local: vec3_t;
	let mut d: f32;
	let mut eye: vec4_t = [0.0; 4];
	let mut clip: vec4_t = [0.0; 4];
	let mut normalized: vec4_t = [0.0; 4];
	let mut window: vec4_t = [0.0; 4];

	(*addr_of_mut!(backEnd)).pc.c_flareAdds += 1;

	// if the point is off the screen, don't bother adding it
	// calculate screen coordinates and depth
	R_TransformModelToClip(
		&point,
		(*addr_of_mut!(backEnd)).ori.axis[0].as_ptr(),
		(*addr_of_mut!(backEnd)).viewParms.projectionMatrix.as_ptr(),
		&mut eye,
		&mut clip,
	);

	// check to see if the point is completely off screen
	i = 0;
	while i < 3 {
		if clip[i as usize] >= clip[3] || clip[i as usize] <= -clip[3] {
			return;
		}
		i += 1;
	}

	R_TransformClipToWindow(
		&clip,
		&(*addr_of_mut!(backEnd)).viewParms,
		&mut normalized,
		&mut window,
	);

	if window[0] < 0.0
		|| window[0] >= (*addr_of_mut!(backEnd)).viewParms.viewportWidth as f32
		|| window[1] < 0.0
		|| window[1] >= (*addr_of_mut!(backEnd)).viewParms.viewportHeight as f32
	{
		return;	// shouldn't happen, since we check the clip[] above, except for FP rounding
	}

	// see if a flare with a matching surface, scene, and view exists
	oldest = &mut r_flareStructs[0];
	f = r_activeFlares;
	while !f.is_null() {
		if (*f).surface == surface
			&& (*f).frameSceneNum == (*addr_of_mut!(backEnd)).viewParms.frameSceneNum
			&& (*f).inPortal == (*addr_of_mut!(backEnd)).viewParms.isPortal
		{
			break;
		}
		f = (*f).next;
	}

	// allocate a new one
	if f.is_null() {
		if r_inactiveFlares.is_null() {
			// the list is completely full
			return;
		}
		f = r_inactiveFlares;
		r_inactiveFlares = (*r_inactiveFlares).next;
		(*f).next = r_activeFlares;
		r_activeFlares = f;

		(*f).surface = surface;
		(*f).frameSceneNum = (*addr_of_mut!(backEnd)).viewParms.frameSceneNum;
		(*f).inPortal = (*addr_of_mut!(backEnd)).viewParms.isPortal;
		(*f).addedFrame = -1;
	}

	if (*f).addedFrame != (*addr_of_mut!(backEnd)).viewParms.frameCount - 1 {
		(*f).visible = QFALSE;
		(*f).fadeTime = (*addr_of_mut!(backEnd)).refdef.time - 2000;
	}

	(*f).addedFrame = (*addr_of_mut!(backEnd)).viewParms.frameCount;
	(*f).fogNum = fogNum;

	VectorCopy(color, &mut (*f).color);

	// fade the intensity of the flare down as the
	// light surface turns away from the viewer
	if !normal.is_null() {
		VectorSubtract((*addr_of_mut!(backEnd)).viewParms.ori.origin, point, &mut local);
		VectorNormalizeFast(&mut local);
		d = DotProduct(local, *normal);
		VectorScale((*f).color, d, &mut (*f).color);
	}

	// save info needed to test
	(*f).windowX = (*addr_of_mut!(backEnd)).viewParms.viewportX + window[0] as c_int;
	(*f).windowY = (*addr_of_mut!(backEnd)).viewParms.viewportY + window[1] as c_int;

	(*f).eyeZ = eye[2];
}

/*
==================
RB_AddDlightFlares
==================
*/
pub unsafe fn RB_AddDlightFlares() {
	let mut l: *mut dlight_t;
	let mut i: c_int;
	let mut j: c_int;
	let mut k: c_int;
	let mut fog: *mut fog_t;

	if (*r_flares).integer == 0 {
		return;
	}

	l = (*addr_of_mut!(backEnd)).refdef.dlights;
	fog = (*(*addr_of_mut!(tr)).world).fogs;
	i = 0;
	while i < (*addr_of_mut!(backEnd)).refdef.num_dlights {
		// find which fog volume the light is in
		j = 1;
		while j < (*(*addr_of_mut!(tr)).world).numfogs {
			fog = &mut (*(*addr_of_mut!(tr)).world).fogs.add(j as usize).read();
			k = 0;
			while k < 3 {
				if (*l).origin[k as usize] < (*fog).bounds[0][k as usize]
					|| (*l).origin[k as usize] > (*fog).bounds[1][k as usize]
				{
					break;
				}
				k += 1;
			}
			if k == 3 {
				break;
			}
			j += 1;
		}
		if j == (*(*addr_of_mut!(tr)).world).numfogs {
			j = 0;
		}

		RB_AddFlare(l as *mut c_void, j, (*l).origin, (*l).color, core::ptr::null_mut());
		l = l.add(1);
		i += 1;
	}
}

/*
===============================================================================

FLARE BACK END

===============================================================================
*/

/*
==================
RB_TestFlare
==================
*/
pub unsafe fn RB_TestFlare(f: *mut flare_t) {
	let mut depth: f32;
	let mut visible: qboolean;
	let mut fade: f32;
	let mut screenZ: f32;

	(*addr_of_mut!(backEnd)).pc.c_flareTests += 1;

	// doing a readpixels is as good as doing a glFinish(), so
	// don't bother with another sync
	(*addr_of_mut!(glState)).finishCalled = QFALSE;

	// read back the z buffer contents
	qglReadPixels(
		(*f).windowX,
		(*f).windowY,
		1,
		1,
		GL_DEPTH_COMPONENT,
		GL_FLOAT,
		&mut depth as *mut f32 as *mut c_void,
	);

	screenZ = (*addr_of_mut!(backEnd)).viewParms.projectionMatrix[14]
		/ ((2.0 * depth - 1.0) * (*addr_of_mut!(backEnd)).viewParms.projectionMatrix[11]
			- (*addr_of_mut!(backEnd)).viewParms.projectionMatrix[10]);

	visible = if ((-(*f).eyeZ) - (-screenZ)) < 24.0 { QTRUE } else { QFALSE };

	if visible != 0 {
		if (*f).visible == 0 {
			(*f).visible = QTRUE;
			(*f).fadeTime = (*addr_of_mut!(backEnd)).refdef.time - 1;
		}
		fade = ((((*addr_of_mut!(backEnd)).refdef.time - (*f).fadeTime) as f32) / 1000.0) * (*r_flareFade).value;
	} else {
		if (*f).visible != 0 {
			(*f).visible = QFALSE;
			(*f).fadeTime = (*addr_of_mut!(backEnd)).refdef.time - 1;
		}
		fade = 1.0 - ((((*addr_of_mut!(backEnd)).refdef.time - (*f).fadeTime) as f32) / 1000.0) * (*r_flareFade).value;
	}

	if fade < 0.0 {
		fade = 0.0;
	}
	if fade > 1.0 {
		fade = 1.0;
	}

	(*f).drawIntensity = fade;
}


/*
==================
RB_RenderFlare
==================
*/
pub unsafe fn RB_RenderFlare(f: *mut flare_t) {
	let mut size: f32;
	let mut color: vec3_t;
	let mut iColor: [c_int; 3];

	(*addr_of_mut!(backEnd)).pc.c_flareRenders += 1;

	VectorScale((*f).color, (*f).drawIntensity * (*addr_of_mut!(tr)).identityLight, &mut color);
	iColor[0] = (color[0] * 255.0) as c_int;
	iColor[1] = (color[1] * 255.0) as c_int;
	iColor[2] = (color[2] * 255.0) as c_int;

	size = ((*addr_of_mut!(backEnd)).viewParms.viewportWidth as f32)
		* ((*r_flareSize).value / 640.0 + 8.0 / (-(*f).eyeZ));

	RB_BeginSurface((*addr_of_mut!(tr)).flareShader, (*f).fogNum);

	// FIXME: use quadstamp?
	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][0] = (*f).windowX as f32 - size;
	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][1] = (*f).windowY as f32 - size;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 0.0;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 0.0;
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = iColor[0];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = iColor[1];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = iColor[2];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][3] = 255;
	(*addr_of_mut!(tess)).numVertexes += 1;

	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][0] = (*f).windowX as f32 - size;
	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][1] = (*f).windowY as f32 + size;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 0.0;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 1.0;
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = iColor[0];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = iColor[1];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = iColor[2];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][3] = 255;
	(*addr_of_mut!(tess)).numVertexes += 1;

	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][0] = (*f).windowX as f32 + size;
	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][1] = (*f).windowY as f32 + size;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 1.0;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 1.0;
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = iColor[0];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = iColor[1];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = iColor[2];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][3] = 255;
	(*addr_of_mut!(tess)).numVertexes += 1;

	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][0] = (*f).windowX as f32 + size;
	(*addr_of_mut!(tess)).xyz[(*addr_of_mut!(tess)).numVertexes as usize][1] = (*f).windowY as f32 - size;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][0] = 1.0;
	(*addr_of_mut!(tess)).texCoords[(*addr_of_mut!(tess)).numVertexes as usize][0][1] = 0.0;
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][0] = iColor[0];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][1] = iColor[1];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][2] = iColor[2];
	(*addr_of_mut!(tess)).vertexColors[(*addr_of_mut!(tess)).numVertexes as usize][3] = 255;
	(*addr_of_mut!(tess)).numVertexes += 1;

	(*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 0;
	(*addr_of_mut!(tess)).numIndexes += 1;
	(*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 1;
	(*addr_of_mut!(tess)).numIndexes += 1;
	(*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 2;
	(*addr_of_mut!(tess)).numIndexes += 1;
	(*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 0;
	(*addr_of_mut!(tess)).numIndexes += 1;
	(*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 2;
	(*addr_of_mut!(tess)).numIndexes += 1;
	(*addr_of_mut!(tess)).indexes[(*addr_of_mut!(tess)).numIndexes as usize] = 3;
	(*addr_of_mut!(tess)).numIndexes += 1;

	RB_EndSurface();
}

/*
==================
RB_RenderFlares

Because flares are simulating an occular effect, they should be drawn after
everything (all views) in the entire frame has been drawn.

Because of the way portals use the depth buffer to mark off areas, the
needed information would be lost after each view, so we are forced to draw
flares after each view.

The resulting artifact is that flares in mirrors or portals don't dim properly
when occluded by something in the main view, and portal flares that should
extend past the portal edge will be overwritten.
==================
*/
pub unsafe fn RB_RenderFlares() {
	let mut f: *mut flare_t;
	let mut prev: *mut *mut flare_t;
	let mut draw: qboolean;

	if (*r_flares).integer == 0 {
		return;
	}

//	RB_AddDlightFlares();

	// perform z buffer readback on each flare in this view
	draw = QFALSE;
	prev = &mut r_activeFlares;
	while !(*prev).is_null() {
		f = *prev;
		// throw out any flares that weren't added last frame
		if (*f).addedFrame < (*addr_of_mut!(backEnd)).viewParms.frameCount - 1 {
			*prev = (*f).next;
			(*f).next = r_inactiveFlares;
			r_inactiveFlares = f;
			continue;
		}

		// don't draw any here that aren't from this scene / portal
		(*f).drawIntensity = 0.0;
		if (*f).frameSceneNum == (*addr_of_mut!(backEnd)).viewParms.frameSceneNum
			&& (*f).inPortal == (*addr_of_mut!(backEnd)).viewParms.isPortal
		{
			RB_TestFlare(f);
			if (*f).drawIntensity != 0.0 {
				draw = QTRUE;
			} else {
				// this flare has completely faded out, so remove it from the chain
				*prev = (*f).next;
				(*f).next = r_inactiveFlares;
				r_inactiveFlares = f;
				continue;
			}
		}

		prev = &mut (*f).next;
	}

	if draw == 0 {
		return;		// none visible
	}

	if (*addr_of_mut!(backEnd)).viewParms.isPortal != 0 {
		qglDisable(GL_CLIP_PLANE0);
	}

	qglPushMatrix();
    qglLoadIdentity();
	qglMatrixMode(GL_PROJECTION);
	qglPushMatrix();
    qglLoadIdentity();
	qglOrtho(
		(*addr_of_mut!(backEnd)).viewParms.viewportX as f32,
		((*addr_of_mut!(backEnd)).viewParms.viewportX + (*addr_of_mut!(backEnd)).viewParms.viewportWidth) as f32,
		(*addr_of_mut!(backEnd)).viewParms.viewportY as f32,
		((*addr_of_mut!(backEnd)).viewParms.viewportY + (*addr_of_mut!(backEnd)).viewParms.viewportHeight) as f32,
		-99999.0,
		99999.0,
	);

	f = r_activeFlares;
	while !f.is_null() {
		if (*f).frameSceneNum == (*addr_of_mut!(backEnd)).viewParms.frameSceneNum
			&& (*f).inPortal == (*addr_of_mut!(backEnd)).viewParms.isPortal
			&& (*f).drawIntensity != 0.0
		{
			RB_RenderFlare(f);
		}
		f = (*f).next;
	}

	qglPopMatrix();
	qglMatrixMode(GL_MODELVIEW);
	qglPopMatrix();
}
