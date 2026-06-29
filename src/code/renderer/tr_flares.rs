// tr_flares.c

use core::ffi::c_int;
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

// Local type stubs - these will be replaced when full dependencies are ported
type qboolean = c_int;
type vec3_t = [f32; 3];
type vec4_t = [f32; 4];

extern "C" {
	static mut backEnd: BackEnd;
	static mut tr: Tr;
	static mut glState: GlState;
	static mut tess: Tess;
	static r_flares: *mut CVar;
	static r_flareFade: *mut CVar;
	static r_flareSize: *mut CVar;

	fn R_TransformModelToClip(
		src: *const vec3_t,
		modelMatrix: *const f32,
		projectionMatrix: *const f32,
		eye: *mut vec4_t,
		clip: *mut vec4_t,
	);
	fn R_TransformClipToWindow(
		clip: *const vec4_t,
		viewParms: *const ViewParms,
		normalized: *mut vec4_t,
		window: *mut vec4_t,
	);
	fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
	fn VectorSubtract(va: *const vec3_t, vb: *const vec3_t, out: *mut vec3_t);
	fn VectorNormalizeFast(v: *mut vec3_t) -> f32;
	fn DotProduct(a: *const vec3_t, b: *const vec3_t) -> f32;
	fn VectorScale(v: *const vec3_t, scale: f32, out: *mut vec3_t);
	fn RB_BeginSurface(shader: *mut Shader, fogNum: c_int);
	fn RB_EndSurface();
	fn memset(s: *mut core::ffi::c_void, c: c_int, n: usize) -> *mut core::ffi::c_void;
	fn qglReadPixels(
		x: c_int,
		y: c_int,
		width: c_int,
		height: c_int,
		format: c_int,
		typ: c_int,
		pixels: *mut f32,
	);
	fn qglDisable(cap: c_int);
	fn qglPushMatrix();
	fn qglLoadIdentity();
	fn qglMatrixMode(mode: c_int);
	fn qglOrtho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32);
	fn qglPopMatrix();
}

// Stub types for external dependencies
struct BackEnd {
	// Placeholder - will be defined when tr_backend_h.rs is ported
}

struct Tr {
	// Placeholder - will be defined when tr_h.rs is ported
}

struct GlState {
	// Placeholder - will be defined when gl_h.rs is ported
}

struct Tess {
	// Placeholder - will be defined when tr_local_h.rs is ported
}

struct CVar {
	// Placeholder - will be defined when cvar_h.rs is ported
}

struct Shader {
	// Placeholder - will be defined when tr_local_h.rs is ported
}

struct ViewParms {
	// Placeholder - will be defined when tr_local_h.rs is ported
}

// flare states maintain visibility over multiple frames for fading
// layers: view, mirror, menu
#[repr(C)]
pub struct flare_s {
	pub next: *mut flare_s,		// for active chain

	pub addedFrame: c_int,

	pub inPortal: qboolean,				// true if in a portal view of the scene
	pub frameSceneNum: c_int,
	pub surface: *mut core::ffi::c_void,
	pub fogNum: c_int,

	pub fadeTime: c_int,

	pub visible: qboolean,			// state of last test
	pub drawIntensity: f32,		// may be non 0 even if !visible due to fading
	pub lightScale: f32,
	pub windowX: c_int,
	pub windowY: c_int,
	pub eyeZ: f32,

	pub color: vec3_t,
}

pub type flare_t = flare_s;

const MAX_FLARES: usize = 128;

pub static mut r_flareStructs: [flare_t; MAX_FLARES] = [flare_t {
	next: core::ptr::null_mut(),
	addedFrame: 0,
	inPortal: 0,
	frameSceneNum: 0,
	surface: core::ptr::null_mut(),
	fogNum: 0,
	fadeTime: 0,
	visible: 0,
	drawIntensity: 0.0,
	lightScale: 0.0,
	windowX: 0,
	windowY: 0,
	eyeZ: 0.0,
	color: [0.0; 3],
}; MAX_FLARES];
pub static mut r_activeFlares: *mut flare_t = core::ptr::null_mut();
pub static mut r_inactiveFlares: *mut flare_t = core::ptr::null_mut();

/*
==================
R_ClearFlares
==================
*/
pub fn R_ClearFlares() {
	let i: c_int;

	unsafe {
		memset(
			addr_of_mut!(r_flareStructs) as *mut core::ffi::c_void,
			0,
			core::mem::size_of_val(&r_flareStructs),
		);
		r_activeFlares = core::ptr::null_mut();
		r_inactiveFlares = core::ptr::null_mut();

		for i in 0..MAX_FLARES as c_int {
			r_flareStructs[i as usize].next = r_inactiveFlares;
			r_inactiveFlares = &mut r_flareStructs[i as usize];
		}
	}
}


/*
==================
RB_AddFlare

This is called at surface tesselation time
==================
*/
pub fn RB_AddFlare(
	surface: *mut core::ffi::c_void,
	fogNum: c_int,
	point: *const vec3_t,
	color: *const vec3_t,
	normal: *const vec3_t,
	lightScale: f32,
) {
	let i: c_int;
	let f: *mut flare_t;
	let oldest: *mut flare_t;
	let local: vec3_t;
	let d: f32;
	let eye: vec4_t;
	let clip: vec4_t;
	let normalized: vec4_t;
	let window: vec4_t;

	unsafe {
		(*addr_of_mut!(backEnd)).pc.c_flareAdds = (*addr_of_mut!(backEnd)).pc.c_flareAdds.wrapping_add(1);

		// if the point is off the screen, don't bother adding it
		// calculate screen coordinates and depth
		R_TransformModelToClip(
			point,
			(*addr_of_mut!(backEnd)).or.modelMatrix.as_ptr(),
			(*addr_of_mut!(backEnd)).viewParms.projectionMatrix.as_ptr(),
			&mut eye as *mut vec4_t,
			&mut clip as *mut vec4_t,
		);

		// check to see if the point is completely off screen
		for i in 0..3 {
			if clip[i] >= clip[3] || clip[i] <= -clip[3] {
				return;
			}
		}

		R_TransformClipToWindow(
			&clip as *const vec4_t,
			&(*addr_of_mut!(backEnd)).viewParms as *const ViewParms,
			&mut normalized as *mut vec4_t,
			&mut window as *mut vec4_t,
		);

		if window[0] < 0.0
			|| window[0] >= (*addr_of_mut!(backEnd)).viewParms.viewportWidth as f32
			|| window[1] < 0.0
			|| window[1] >= (*addr_of_mut!(backEnd)).viewParms.viewportHeight as f32
		{
			return;	// shouldn't happen, since we check the clip[] above, except for FP rounding
		}

		// see if a flare with a matching surface, scene, and view exists
		oldest = r_flareStructs.as_mut_ptr();
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
			(*f).visible = 0;
			(*f).fadeTime = (*addr_of_mut!(backEnd)).refdef.time - 2000;
		}

		(*f).addedFrame = (*addr_of_mut!(backEnd)).viewParms.frameCount;
		(*f).fogNum = fogNum;
		(*f).lightScale = lightScale;

		VectorCopy(color, &mut (*f).color as *mut vec3_t);

		// fade the intensity of the flare down as the
		// light surface turns away from the viewer
		if !normal.is_null() {
			VectorSubtract(
				&(*addr_of_mut!(backEnd)).viewParms.or.origin as *const vec3_t,
				point,
				&mut local as *mut vec3_t,
			);
			VectorNormalizeFast(&mut local as *mut vec3_t);
			d = DotProduct(&local as *const vec3_t, normal);
			VectorScale(
				&(*f).color as *const vec3_t,
				d,
				&mut (*f).color as *mut vec3_t,
			);
		}

		// save info needed to test
		(*f).windowX = (*addr_of_mut!(backEnd)).viewParms.viewportX + window[0] as c_int;
		(*f).windowY = (*addr_of_mut!(backEnd)).viewParms.viewportY + window[1] as c_int;

		(*f).eyeZ = eye[2];
	}
}

/*
==================
RB_AddDlightFlares
==================
*/
pub fn RB_AddDlightFlares() {
	let l: *mut DLight;
	let i: c_int;
	let j: c_int;
	let k: c_int;
	let fog: *mut Fog;

	unsafe {
		if (*r_flares).integer == 0 {
			return;
		}

		l = (*addr_of_mut!(backEnd)).refdef.dlights.as_mut_ptr();
		fog = (*tr.world).fogs.as_mut_ptr();
		for i in 0..(*addr_of_mut!(backEnd)).refdef.num_dlights {
			// find which fog volume the light is in
			for j in 1..(*(*tr.world)).numfogs {
				fog = &mut (*(*tr.world)).fogs[j as usize];
				for k in 0..3 {
					if (*l).origin[k as usize] < (*fog).bounds[0][k as usize]
						|| (*l).origin[k as usize] > (*fog).bounds[1][k as usize]
					{
						break;
					}
				}
				if k == 3 {
					break;
				}
			}
			if j == (*(*tr.world)).numfogs {
				j = 0;
			}

			RB_AddFlare(
				l as *mut core::ffi::c_void,
				j,
				&(*l).origin as *const vec3_t,
				&(*l).color as *const vec3_t,
				core::ptr::null(),
				1.0,
			);

			l = l.offset(1);
		}
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
pub fn RB_TestFlare(f: *mut flare_t) {
	let depth: f32;
	let visible: qboolean;
	let fade: f32;
	let screenZ: f32;

	unsafe {
		(*addr_of_mut!(backEnd)).pc.c_flareTests =
			(*addr_of_mut!(backEnd)).pc.c_flareTests.wrapping_add(1);

		// doing a readpixels is as good as doing a glFinish(), so
		// don't bother with another sync
		(*addr_of_mut!(glState)).finishCalled = 0;

		// read back the z buffer contents
		qglReadPixels(
			(*f).windowX,
			(*f).windowY,
			1,
			1,
			0x1902, // GL_DEPTH_COMPONENT
			0x1406, // GL_FLOAT
			&mut depth as *mut f32,
		);

		screenZ = (*addr_of_mut!(backEnd)).viewParms.projectionMatrix[14]
			/ ((2.0 * depth - 1.0) * (*addr_of_mut!(backEnd)).viewParms.projectionMatrix[11]
				- (*addr_of_mut!(backEnd)).viewParms.projectionMatrix[10]);

		visible = if (-(*f).eyeZ - (-screenZ)) < 24.0 { 1 } else { 0 };

		if visible != 0 {
			if (*f).visible == 0 {
				(*f).visible = 1;
				(*f).fadeTime = (*addr_of_mut!(backEnd)).refdef.time - 1;
			}
			fade = (((*addr_of_mut!(backEnd)).refdef.time - (*f).fadeTime) as f32 / 1000.0)
				* (*r_flareFade).value;
		} else {
			if (*f).visible != 0 {
				(*f).visible = 0;
				(*f).fadeTime = (*addr_of_mut!(backEnd)).refdef.time - 1;
			}
			fade = 1.0 - ((((*addr_of_mut!(backEnd)).refdef.time - (*f).fadeTime) as f32 / 1000.0)
				* (*r_flareFade).value);
		}

		if fade < 0.0 {
			fade = 0.0;
		}
		if fade > 1.0 {
			fade = 1.0;
		}

		(*f).drawIntensity = fade;
	}
}


/*
==================
RB_RenderFlare
==================
*/
pub fn RB_RenderFlare(f: *mut flare_t) {
	let size: f32;
	let color: vec3_t;
	let iColor: [c_int; 3];

	unsafe {
		(*addr_of_mut!(backEnd)).pc.c_flareRenders =
			(*addr_of_mut!(backEnd)).pc.c_flareRenders.wrapping_add(1);

		VectorScale(
			&(*f).color as *const vec3_t,
			(*f).drawIntensity * (*tr.identityLight),
			&mut color as *mut vec3_t,
		);
		iColor[0] = (color[0] * 255.0) as c_int;
		iColor[1] = (color[1] * 255.0) as c_int;
		iColor[2] = (color[2] * 255.0) as c_int;

		size = (*f).lightScale
			* (*addr_of_mut!(backEnd)).viewParms.viewportWidth as f32
			* ((*r_flareSize).value / 640.0 + 8.0 / -(*f).eyeZ);

		RB_BeginSurface((*tr.flareShader), (*f).fogNum);

		// FIXME: use quadstamp?
		tess.xyz[tess.numVertexes][0] = (*f).windowX as f32 - size;
		tess.xyz[tess.numVertexes][1] = (*f).windowY as f32 - size;
		tess.texCoords[tess.numVertexes][0][0] = 0.0;
		tess.texCoords[tess.numVertexes][0][1] = 0.0;
		tess.vertexColors[tess.numVertexes][0] = iColor[0];
		tess.vertexColors[tess.numVertexes][1] = iColor[1];
		tess.vertexColors[tess.numVertexes][2] = iColor[2];
		tess.vertexColors[tess.numVertexes][3] = 255;
		tess.numVertexes += 1;

		tess.xyz[tess.numVertexes][0] = (*f).windowX as f32 - size;
		tess.xyz[tess.numVertexes][1] = (*f).windowY as f32 + size;
		tess.texCoords[tess.numVertexes][0][0] = 0.0;
		tess.texCoords[tess.numVertexes][0][1] = 1.0;
		tess.vertexColors[tess.numVertexes][0] = iColor[0];
		tess.vertexColors[tess.numVertexes][1] = iColor[1];
		tess.vertexColors[tess.numVertexes][2] = iColor[2];
		tess.vertexColors[tess.numVertexes][3] = 255;
		tess.numVertexes += 1;

		tess.xyz[tess.numVertexes][0] = (*f).windowX as f32 + size;
		tess.xyz[tess.numVertexes][1] = (*f).windowY as f32 + size;
		tess.texCoords[tess.numVertexes][0][0] = 1.0;
		tess.texCoords[tess.numVertexes][0][1] = 1.0;
		tess.vertexColors[tess.numVertexes][0] = iColor[0];
		tess.vertexColors[tess.numVertexes][1] = iColor[1];
		tess.vertexColors[tess.numVertexes][2] = iColor[2];
		tess.vertexColors[tess.numVertexes][3] = 255;
		tess.numVertexes += 1;

		tess.xyz[tess.numVertexes][0] = (*f).windowX as f32 + size;
		tess.xyz[tess.numVertexes][1] = (*f).windowY as f32 - size;
		tess.texCoords[tess.numVertexes][0][0] = 1.0;
		tess.texCoords[tess.numVertexes][0][1] = 0.0;
		tess.vertexColors[tess.numVertexes][0] = iColor[0];
		tess.vertexColors[tess.numVertexes][1] = iColor[1];
		tess.vertexColors[tess.numVertexes][2] = iColor[2];
		tess.vertexColors[tess.numVertexes][3] = 255;
		tess.numVertexes += 1;

		tess.indexes[tess.numIndexes] = 0;
		tess.numIndexes += 1;
		tess.indexes[tess.numIndexes] = 1;
		tess.numIndexes += 1;
		tess.indexes[tess.numIndexes] = 2;
		tess.numIndexes += 1;
		tess.indexes[tess.numIndexes] = 0;
		tess.numIndexes += 1;
		tess.indexes[tess.numIndexes] = 2;
		tess.numIndexes += 1;
		tess.indexes[tess.numIndexes] = 3;
		tess.numIndexes += 1;

		RB_EndSurface();
	}
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
pub fn RB_RenderFlares() {
	let f: *mut flare_t;
	let prev: *mut *mut flare_t;
	let draw: qboolean;

	unsafe {
		if (*r_flares).integer == 0 {
			return;
		}

		//	RB_AddDlightFlares();

		// perform z buffer readback on each flare in this view
		draw = 0;
		prev = &mut r_activeFlares;
		while {
			f = *prev;
			!f.is_null()
		} {
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
					draw = 1;
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
			qglDisable(0x3000); // GL_CLIP_PLANE0
		}

		qglPushMatrix();
		qglLoadIdentity();
		qglMatrixMode(0x1701); // GL_PROJECTION
		qglPushMatrix();
		qglLoadIdentity();
		qglOrtho(
			(*addr_of_mut!(backEnd)).viewParms.viewportX as f32,
			((*addr_of_mut!(backEnd)).viewParms.viewportX + (*addr_of_mut!(backEnd)).viewParms.viewportWidth)
				as f32,
			(*addr_of_mut!(backEnd)).viewParms.viewportY as f32,
			((*addr_of_mut!(backEnd)).viewParms.viewportY + (*addr_of_mut!(backEnd)).viewParms.viewportHeight)
				as f32,
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
		qglMatrixMode(0x1700); // GL_MODELVIEW
		qglPopMatrix();
	}
}

// Stub types for external dependencies
struct DLight {
	// Placeholder - will be defined when tr_local_h.rs is ported
	origin: vec3_t,
	color: vec3_t,
}

struct Fog {
	// Placeholder - will be defined when tr_local_h.rs is ported
	bounds: [[f32; 3]; 2],
}
