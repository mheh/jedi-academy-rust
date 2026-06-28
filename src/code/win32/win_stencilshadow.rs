//
//
// win_stencilshadow.cpp
//
// Stencil shadow computation/rendering
//
//

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// vec3_t is [f32; 3]
pub type vec3_t = [f32; 3];

// Constants from shader system
pub const SHADER_MAX_VERTEXES: usize = 1000;
pub const SHADER_MAX_INDEXES: usize = 6000;
pub const MAX_EDGE_DEFS: usize = 16;

// edgeDef_t struct
#[repr(C)]
#[derive(Copy, Clone)]
pub struct edgeDef_t {
	// facing is only one bit, but we can't do better than 4 bytes without
	// packing all the data we need into a single unsigned short, which
	// isn't really worth it. (unless we REALLY need 64k at some point).
	pub i2: i16,
	pub facing: i16,
}

// Local stubs for renderer types
#[repr(C)]
pub struct BackEndState {
	pub ori: frame_t,
	pub currentEntity: *mut backEndEntity_t,
	pub viewParms: viewParms_t,
}

#[repr(C)]
pub struct frame_t {
	pub axis: [[f32; 3]; 3],
}

#[repr(C)]
pub struct backEndEntity_t {
	pub lightDir: [f32; 3],
}

#[repr(C)]
pub struct viewParms_t {
	pub ori: frame_t,
}

// Tessellation data structure
#[repr(C)]
pub struct shaderCommands_t {
	pub xyz: *mut vec3_t,
	pub indexes: *mut i32,
	pub numVertexes: i32,
	pub numIndexes: i32,
}

// Windows/D3D types
pub type DWORD = u32;

// Opaque DirectX types
#[repr(C)]
pub struct IDirect3DDevice8 {
	_private: [u8; 0],
}

#[repr(C)]
pub struct ID3DXMatrixStack {
	_private: [u8; 0],
}

// DirectX enums and constants
pub type D3DPRIMITIVETYPE = u32;
pub type D3DTEXTUREOP = u32;
pub type D3DRENDERSTATETYPE = u32;

pub const D3DTSS_COLOROP: D3DTEXTUREOP = 1;
pub const D3DTOP_DISABLE: D3DTEXTUREOP = 1;
pub const D3DTOP_SELECTARG1: D3DTEXTUREOP = 2;

pub const D3DPT_QUADLIST: D3DPRIMITIVETYPE = 1;
pub const D3DPT_TRIANGLESTRIP: D3DPRIMITIVETYPE = 3;
pub const D3DPT_TRIANGLELIST: D3DPRIMITIVETYPE = 4;

pub const D3DRS_LIGHTING: D3DRENDERSTATETYPE = 137;
pub const D3DRS_FOGENABLE: D3DRENDERSTATETYPE = 28;
pub const D3DRS_SRCBLEND: D3DRENDERSTATETYPE = 19;
pub const D3DRS_DESTBLEND: D3DRENDERSTATETYPE = 20;
pub const D3DRS_ALPHABLENDENABLE: D3DRENDERSTATETYPE = 27;
pub const D3DRS_ZWRITEENABLE: D3DRENDERSTATETYPE = 14;
pub const D3DRS_ZFUNC: D3DRENDERSTATETYPE = 23;
pub const D3DRS_STENCILENABLE: D3DRENDERSTATETYPE = 52;
pub const D3DRS_SHADEMODE: D3DRENDERSTATETYPE = 9;
pub const D3DRS_STENCILFUNC: D3DRENDERSTATETYPE = 54;
pub const D3DRS_STENCILZFAIL: D3DRENDERSTATETYPE = 56;
pub const D3DRS_STENCILFAIL: D3DRENDERSTATETYPE = 55;
pub const D3DRS_STENCILREF: D3DRENDERSTATETYPE = 57;
pub const D3DRS_STENCILMASK: D3DRENDERSTATETYPE = 58;
pub const D3DRS_STENCILWRITEMASK: D3DRENDERSTATETYPE = 59;
pub const D3DRS_STENCILPASS: D3DRENDERSTATETYPE = 60;
pub const D3DRS_ZENABLE: D3DRENDERSTATETYPE = 7;
pub const D3DRS_CULLMODE: D3DRENDERSTATETYPE = 22;
pub const D3DRS_TEXTUREFACTOR: D3DRENDERSTATETYPE = 60;

pub const D3DCMP_ALWAYS: u32 = 7;
pub const D3DCMP_LESS: u32 = 1;
pub const D3DCMP_NOTEQUAL: u32 = 5;

pub const D3DSTENCILOP_KEEP: u32 = 1;
pub const D3DSTENCILOP_INCR: u32 = 3;
pub const D3DSTENCILOP_DECR: u32 = 4;

pub const D3DBLEND_ZERO: u32 = 1;
pub const D3DBLEND_ONE: u32 = 2;
pub const D3DBLEND_SRCALPHA: u32 = 5;
pub const D3DBLEND_INVSRCALPHA: u32 = 6;

pub const D3DSHADE_FLAT: u32 = 1;
pub const D3DSHADE_GOURAUD: u32 = 2;

pub const D3DCULL_CCW: u32 = 3;

pub const D3DTSS_COLORARG1: u32 = 2;
pub const D3DTA_TFACTOR: u32 = 0;
pub const D3DTSS_ALPHAOP: u32 = 3;
pub const D3DTSS_ALPHAARG1: u32 = 4;

pub const D3DFVF_XYZ: u32 = 0x002;
pub const D3DFVF_XYZRHW: u32 = 0x004;

pub const GL_FRONT: i32 = 1028;
pub const GL_BACK: i32 = 1029;

// glwstate_t structure
#[repr(C)]
pub struct glwstate_t {
	pub device: *mut IDirect3DDevice8,
	pub matrixStack: [*mut ID3DXMatrixStack; 6],
	// ... other fields as needed
}

// StencilShadow class in Rust
#[repr(C)]
pub struct StencilShadow {
	pub m_edgeDefs: [[edgeDef_t; MAX_EDGE_DEFS]; SHADER_MAX_VERTEXES],
	pub m_numEdgeDefs: [i32; SHADER_MAX_VERTEXES],
	pub m_facing: [i32; SHADER_MAX_INDEXES / 3],
	pub m_shadowVerts: [vec3_t; SHADER_MAX_VERTEXES * 8],
}

// External C variables and functions
extern "C" {
	pub static mut StencilShadower: StencilShadow;

	pub static mut tess: shaderCommands_t;
	pub static mut backEnd: BackEndState;
	pub static mut glw_state: *mut glwstate_t;

	pub fn Com_Printf(fmt: *const i8, ...);
	pub fn GL_Bind(texture: i32);
	pub fn qglCullFace(mode: i32);
	pub fn VectorCopy(in_: *const c_void, out: *mut c_void);
	pub fn VectorMA(veca: *const f32, scale: f32, vecb: *const f32, vecc: *mut f32);
	pub fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
	pub fn CrossProduct(v1: *const f32, v2: *const f32, cross: *mut f32);
	pub fn DotProduct(v1: *const f32, v2: *const f32) -> f32;
	pub fn VectorNormalize(v: *mut f32) -> f32;
	pub fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void;
}

impl StencilShadow {
	pub fn new() -> Self {
		StencilShadow {
			m_edgeDefs: [[edgeDef_t { i2: 0, facing: 0 }; MAX_EDGE_DEFS]; SHADER_MAX_VERTEXES],
			m_numEdgeDefs: [0; SHADER_MAX_VERTEXES],
			m_facing: [0; SHADER_MAX_INDEXES / 3],
			m_shadowVerts: [[0.0; 3]; SHADER_MAX_VERTEXES * 8],
		}
	}

	pub fn AddEdge(&mut self, i1: c_int, i2: c_int, facing: c_int) {
		let i1 = i1 as usize;
		let i2 = i2 as usize;

		let c = self.m_numEdgeDefs[i1] as usize;
		if c == MAX_EDGE_DEFS {
			unsafe {
				Com_Printf(b"WARNING: MAX_EDGE_DEFS overflow!\n\0".as_ptr() as *const i8);
			}
			return; // overflow
		}
		self.m_edgeDefs[i1][c].i2 = i2 as i16;
		self.m_edgeDefs[i1][c].facing = facing as i16;

		self.m_numEdgeDefs[i1] += 1;
	}

	pub fn RenderEdges(&self) {
		//   int		i;
		//int		c, c2;
		//int		j, k;
		//int		i2;
		//int		c_edges, c_rejected;
		//int		hit[2];

		//// an edge is NOT a silhouette edge if its face doesn't face the light,
		//// or if it has a reverse paired edge that also faces the light.
		//// A well behaved polyhedron would have exactly two faces for each edge,
		//// but lots of models have dangling edges or overfanned edges
		//c_edges = 0;
		//c_rejected = 0;

		//for ( i = 0 ; i < tess.numVertexes ; i++ )
		//{
		//	c = m_numEdgeDefs[ i ];
		//	for ( j = 0 ; j < c ; j++ )
		//	{
		//		if ( !m_edgeDefs[ i ][ j ].facing )
		//		{
		//			continue;
		//		}

		//		hit[0] = 0;
		//		hit[1] = 0;

		//		i2 = m_edgeDefs[ i ][ j ].i2;
		//		c2 = m_numEdgeDefs[ i2 ];
		//		for ( k = 0 ; k < c2 ; k++ )
		//		{
		//			if ( m_edgeDefs[ i2 ][ k ].i2 == i )
		//			{
		//				hit[ m_edgeDefs[ i2 ][ k ].facing ]++;
		//			}
		//		}

		//		// if it doesn't share the edge with another front facing
		//		// triangle, it is a sil edge
		//		if ( hit[ 1 ] == 0 )
		//		{
		//			VectorCopy( tess.xyz[i],					 m_shadowVerts[0] );
		//			VectorCopy( tess.xyz[i + tess.numVertexes],  m_shadowVerts[1] );
		//			VectorCopy( tess.xyz[i2],					 m_shadowVerts[2] );
		//			VectorCopy( tess.xyz[i2 + tess.numVertexes], m_shadowVerts[3] );

		//			c_edges++;

		//			glw_state->device->SetVertexShader( D3DFVF_XYZ );
		//			glw_state->device->DrawPrimitiveUP( D3DPT_TRIANGLESTRIP, 2, m_shadowVerts, sizeof(vec3_t) );
		//		}
		//		else
		//		{
		//			c_rejected++;
		//		}
		//	}
		//}

		let mut i: c_int;
		let mut c: c_int;
		let mut j: c_int;
		let mut i2: c_int;
		let mut c_edges: c_int;
		let mut c_rejected: c_int;
		let mut numTris: c_int;
		let mut o1: c_int;
		let mut o2: c_int;
		let mut o3: c_int;

		// an edge is NOT a silhouette edge if its face doesn't face the light,
		// or if it has a reverse paired edge that also faces the light.
		// A well behaved polyhedron would have exactly two faces for each edge,
		// but lots of models have dangling edges or overfanned edges
		c_edges = 0;
		c_rejected = 0;

		let mut nVerts: i32 = 0;
		let mut numPrims: i32 = 0;

		unsafe {
			i = 0;
			while i < tess.numVertexes {
				c = self.m_numEdgeDefs[i as usize];
				j = 0;
				while j < c {
					if self.m_edgeDefs[i as usize][j as usize].facing == 0 {
						j += 1;
						continue;
					}

					//with this system we can still get edges shared by more than 2 tris which
					//produces artifacts including seeing the shadow through walls. So for now
					//we are going to render all edges even though it is a tiny bit slower. -rww
					i2 = self.m_edgeDefs[i as usize][j as usize].i2 as c_int;
					VectorCopy(
						addr_of!((*tess.xyz.add(i as usize))).cast(),
						addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
					);
					nVerts += 1;
					VectorCopy(
						addr_of!((*tess.xyz.add((i + tess.numVertexes) as usize))).cast(),
						addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
					);
					nVerts += 1;
					VectorCopy(
						addr_of!((*tess.xyz.add((i2 + tess.numVertexes) as usize))).cast(),
						addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
					);
					nVerts += 1;
					VectorCopy(
						addr_of!((*tess.xyz.add(i2 as usize))).cast(),
						addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
					);
					nVerts += 1;
					numPrims += 1;
					j += 1;
				}
				i += 1;
			}

			if numPrims == 0 || nVerts == 0 {
				return;
			}

			(*glw_state).device.as_mut().unwrap().SetTextureStageState(0, D3DTSS_COLOROP, D3DTOP_DISABLE);

			(*glw_state).device.as_mut().unwrap().SetVertexShader(D3DFVF_XYZ);
			(*glw_state).device.as_mut().unwrap().DrawPrimitiveUP(
				D3DPT_QUADLIST,
				numPrims,
				self.m_shadowVerts.as_ptr() as *const c_void,
				core::mem::size_of::<vec3_t>(),
			);

			nVerts = 0;
			numPrims = 0;

			//Carmack Reverse<tm> method requires that volumes
			//be capped properly -rww
			numTris = tess.numIndexes / 3;

			i = 0;
			while i < numTris {
				if self.m_facing[i as usize] == 0 {
					i += 1;
					continue;
				}

				o1 = *tess.indexes.add((i * 3 + 0) as usize);
				o2 = *tess.indexes.add((i * 3 + 1) as usize);
				o3 = *tess.indexes.add((i * 3 + 2) as usize);

				VectorCopy(
					addr_of!((*tess.xyz.add(o1 as usize))).cast(),
					addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
				);
				nVerts += 1;
				VectorCopy(
					addr_of!((*tess.xyz.add(o2 as usize))).cast(),
					addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
				);
				nVerts += 1;
				VectorCopy(
					addr_of!((*tess.xyz.add(o3 as usize))).cast(),
					addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
				);
				nVerts += 1;
				VectorCopy(
					addr_of!((*tess.xyz.add((o3 + tess.numVertexes) as usize))).cast(),
					addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
				);
				nVerts += 1;
				VectorCopy(
					addr_of!((*tess.xyz.add((o2 + tess.numVertexes) as usize))).cast(),
					addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
				);
				nVerts += 1;
				VectorCopy(
					addr_of!((*tess.xyz.add((o1 + tess.numVertexes) as usize))).cast(),
					addr_of_mut!(self.m_shadowVerts[nVerts as usize]).cast(),
				);
				nVerts += 1;
				numPrims += 2;
				i += 1;
			}

			(*glw_state).device.as_mut().unwrap().SetVertexShader(D3DFVF_XYZ);
			(*glw_state).device.as_mut().unwrap().DrawPrimitiveUP(
				D3DPT_TRIANGLELIST,
				numPrims,
				self.m_shadowVerts.as_ptr() as *const c_void,
				core::mem::size_of::<vec3_t>(),
			);
		}
	}

	pub fn BuildFromLight(&mut self, dl: *mut c_void) -> bool {
		// //   int		i;
		// //int		numTris;
		// //vec3_t	lightDir;
		// //D3DXMATRIX matWorldInv;
		// //D3DXVECTOR4 viewLightPos;

		// //// we can only do this if we have enough space in the vertex buffers
		// //if ( tess.numVertexes >= SHADER_MAX_VERTEXES / 2 ) {
		// //	return false;
		// //}

		// //// project vertexes away from light direction
		// //for ( i = 0 ; i < tess.numVertexes ; i++ )
		// //{
		// //	// Get the light direction to the vertex
		// //	VectorCopy( backEnd.currentEntity->lightDir, lightDir );

		// //	VectorMA( tess.xyz[i], -512, lightDir, tess.xyz[i+tess.numVertexes] );
		// //}

		let mut i: c_int;
		let mut numTris: c_int;
		let mut lightDir: vec3_t = [0.0; 3];
		let mut ground: vec3_t = [0.0; 3];
		let mut d: f32;

		// we can only do this if we have enough space in the vertex buffers
		unsafe {
			if tess.numVertexes >= (SHADER_MAX_VERTEXES / 2) as i32 {
				return false;
			}

			//controlled method - try to keep shadows in range so they don't show through so much -rww
			let mut _worldxyz: vec3_t = [0.0; 3];
			let mut _ld: vec3_t = [0.0; 3];
			let mut _groundDist: f32;
			let mut _extlength: f32;

			VectorCopy(
				addr_of!((*backEnd.currentEntity).lightDir).cast(),
				addr_of_mut!(lightDir).cast(),
			);

			ground[0] = backEnd.ori.axis[0][2];
			ground[1] = backEnd.ori.axis[1][2];
			ground[2] = backEnd.ori.axis[2][2];

			d = DotProduct(addr_of!(lightDir).cast(), addr_of!(ground).cast());
			// don't let the shadows get too long or go negative
			if d < 0.5 {
				VectorMA(
					addr_of!(lightDir).cast(),
					0.5 - d,
					addr_of!(ground).cast(),
					addr_of_mut!(lightDir).cast(),
				);
				d = DotProduct(addr_of!(lightDir).cast(), addr_of!(ground).cast());
			}
			d = 1.0 / d;

			lightDir[0] = lightDir[0] * d;
			lightDir[1] = lightDir[1] * d;
			lightDir[2] = lightDir[2] * d;

			VectorNormalize(addr_of_mut!(lightDir).cast());

			//Oh well, just cast them straight down no matter what onto the ground plane.
			//This presents no chance of screwups and still looks better than a stupid
			//shader blob.
			//VectorSet(lightDir, 0.0f, 0.0f, 1.0f);

			// project vertexes away from light direction
			i = 0;
			while i < tess.numVertexes {
				//add or.origin to vert xyz to end up with world oriented coord, then figure
				//out the ground pos for the vert to project the shadow volume to
				//VectorAdd(tess.xyz[i], backEnd.ori.origin, worldxyz);
				//groundDist = worldxyz[2] - backEnd.currentEntity->e.shadowPlane;
				//groundDist += 2.0f; //fudge factor
				//VectorMA( tess.xyz[i], -groundDist, lightDir, tess.xyz[i+tess.numVertexes] );
				VectorMA(
					addr_of!((*tess.xyz.add(i as usize))).cast(),
					-200.0,
					addr_of!(lightDir).cast(),
					addr_of_mut!((*tess.xyz.add((i + tess.numVertexes) as usize))).cast(),
				);
				i += 1;
			}

			// decide which triangles face the light
			memset(
				addr_of_mut!(self.m_numEdgeDefs).cast(),
				0,
				4 * tess.numVertexes as usize,
			);

			numTris = tess.numIndexes / 3;
			i = 0;
			while i < numTris {
				let mut i1: c_int;
				let mut i2: c_int;
				let mut i3: c_int;
				let mut d1: vec3_t = [0.0; 3];
				let mut d2: vec3_t = [0.0; 3];
				let mut normal: vec3_t = [0.0; 3];
				let mut v1: *const f32;
				let mut v2: *const f32;
				let mut v3: *const f32;
				let mut d: f32;

				i1 = *tess.indexes.add((i * 3 + 0) as usize);
				i2 = *tess.indexes.add((i * 3 + 1) as usize);
				i3 = *tess.indexes.add((i * 3 + 2) as usize);

				v1 = addr_of!((*tess.xyz.add(i1 as usize))[0]); // Cast to *const f32
				v2 = addr_of!((*tess.xyz.add(i2 as usize))[0]);
				v3 = addr_of!((*tess.xyz.add(i3 as usize))[0]);

				VectorSubtract(v2, v1, addr_of_mut!(d1).cast());
				VectorSubtract(v3, v1, addr_of_mut!(d2).cast());
				CrossProduct(
					addr_of!(d1).cast(),
					addr_of!(d2).cast(),
					addr_of_mut!(normal).cast(),
				);

				d = DotProduct(addr_of!(normal).cast(), addr_of!(lightDir).cast());
				if d > 0.0 {
					self.m_facing[i as usize] = 1;
				} else {
					self.m_facing[i as usize] = 0;
				}

				// create the edges
				self.AddEdge(i1, i2, self.m_facing[i as usize]);
				self.AddEdge(i2, i3, self.m_facing[i as usize]);
				self.AddEdge(i3, i1, self.m_facing[i as usize]);
				i += 1;
			}

			return true;
		}
	}

	pub fn RenderShadow(&self) {
		let mut lighting: DWORD = 0;
		let mut fog: DWORD = 0;
		let mut srcblend: DWORD = 0;
		let mut destblend: DWORD = 0;
		let mut alphablend: DWORD = 0;
		let mut zwrite: DWORD = 0;
		let mut zfunc: DWORD = 0;

		unsafe {
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_LIGHTING, &mut lighting);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_FOGENABLE, &mut fog);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_SRCBLEND, &mut srcblend);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_DESTBLEND, &mut destblend);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_ALPHABLENDENABLE, &mut alphablend);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_ZWRITEENABLE, &mut zwrite);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_ZFUNC, &mut zfunc);

			GL_Bind(1); // tr.whiteImage - stub call

			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_LIGHTING, 0); // FALSE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_FOGENABLE, 0); // FALSE

			// Disable z-buffer writes (note: z-testing still occurs), and enable the
			// stencil-buffer
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ZWRITEENABLE, 0); // FALSE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILENABLE, 1); // TRUE

			// Don't bother with interpolating color
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_SHADEMODE, D3DSHADE_FLAT);

			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ZFUNC, D3DCMP_LESS);

			// Set up stencil compare function, reference value, and masks.
			// Stencil test passes if ((ref & mask) cmpfn (stencil & mask)) is true.
			// Note: since we set up the stencil-test to always pass, the STENCILFAIL
			// renderstate is really not needed.
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILFUNC, D3DCMP_ALWAYS);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILZFAIL, D3DSTENCILOP_INCR);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILFAIL, D3DSTENCILOP_KEEP);

			// If ztest passes, inc/decrement stencil buffer value
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILREF, 0x1);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILMASK, 0xffffffff);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILWRITEMASK, 0xffffffff);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILPASS, D3DSTENCILOP_KEEP);

			// Make sure that no pixels get drawn to the frame buffer
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ALPHABLENDENABLE, 1); // TRUE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_SRCBLEND, D3DBLEND_ZERO);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_DESTBLEND, D3DBLEND_ONE);

			(*glw_state).device.as_mut().unwrap().SetTransform(
				0, // D3DTS_VIEW - stub
				(&(*(*glw_state).matrixStack[0]).GetTop() as *const _) as *const c_void,
			);

			(*glw_state).device.as_mut().unwrap().SetTexture(0, core::ptr::null_mut());
			(*glw_state).device.as_mut().unwrap().SetTexture(1, core::ptr::null_mut());

			qglCullFace(GL_FRONT);

			// Draw front-side of shadow volume in stencil/z only
			self.RenderEdges();

			// Now reverse cull order so back sides of shadow volume are written.
			qglCullFace(GL_BACK);

			// Decrement stencil buffer value
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILZFAIL, D3DSTENCILOP_DECR);

			// Draw back-side of shadow volume in stencil/z only
			self.RenderEdges();

			// Restore render states
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_SHADEMODE, D3DSHADE_GOURAUD);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILENABLE, 0); // FALSE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_LIGHTING, lighting);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_FOGENABLE, fog);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_SRCBLEND, srcblend);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_DESTBLEND, destblend);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ALPHABLENDENABLE, alphablend);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ZWRITEENABLE, zwrite);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ZFUNC, zfunc);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_CULLMODE, D3DCULL_CCW);
		}
	}

	pub fn FinishShadows(&self) {
		let mut lighting: DWORD = 0;
		let mut fog: DWORD = 0;
		let mut srcblend: DWORD = 0;
		let mut destblend: DWORD = 0;
		let mut alphablend: DWORD = 0;

		unsafe {
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_LIGHTING, &mut lighting);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_FOGENABLE, &mut fog);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_SRCBLEND, &mut srcblend);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_DESTBLEND, &mut destblend);
			(*glw_state).device.as_mut().unwrap().GetRenderState(D3DRS_ALPHABLENDENABLE, &mut alphablend);

			// The stencilbuffer values indicates # of shadows that overlap each pixel.
			// We only want to draw pixels that are in shadow, which was set up in
			// RenderShadow() such that StencilBufferValue >= 1. In the Direct3D API,
			// the stencil test is pseudo coded as:
			//    StencilRef CompFunc StencilBufferValue
			// so we set our renderstates with StencilRef = 1 and CompFunc = LESSEQUAL.
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILENABLE, 1); // TRUE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILREF, 0); //0x1
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILFUNC, D3DCMP_NOTEQUAL); //D3DCMP_LESSEQUAL
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILWRITEMASK, 255);

			// Set renderstates (disable z-buffering and turn on alphablending)
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ZENABLE, 0); // FALSE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ALPHABLENDENABLE, 1); // TRUE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_SRCBLEND, D3DBLEND_SRCALPHA);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA);

			// Set the hardware to draw black, alpha-blending pixels
			(*glw_state).device.as_mut().unwrap().SetTextureStageState(0, D3DTSS_COLOROP, D3DTOP_SELECTARG1);
			(*glw_state).device.as_mut().unwrap().SetTextureStageState(0, D3DTSS_COLORARG1, D3DTA_TFACTOR);
			(*glw_state).device.as_mut().unwrap().SetTextureStageState(0, D3DTSS_ALPHAOP, D3DTOP_SELECTARG1);
			(*glw_state).device.as_mut().unwrap().SetTextureStageState(0, D3DTSS_ALPHAARG1, D3DTA_TFACTOR);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_TEXTUREFACTOR, 0x7f000000);

			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_FOGENABLE, 0); // FALSE

			// Draw the big, darkening square
			let v: [[f32; 4]; 4] = [
				[0.0 - 0.5, 0.0 - 0.5, 0.0, 1.0],
				[640.0 - 0.5, 0.0 - 0.5, 0.0, 1.0],
				[640.0 - 0.5, 480.0 - 0.5, 0.0, 1.0],
				[0.0 - 0.5, 480.0 - 0.5, 0.0, 1.0],
			];

			(*glw_state).device.as_mut().unwrap().SetVertexShader(D3DFVF_XYZRHW);
			(*glw_state).device.as_mut().unwrap().DrawPrimitiveUP(
				D3DPT_QUADLIST,
				1,
				v.as_ptr() as *const c_void,
				core::mem::size_of::<[f32; 4]>(),
			);

			// Restore render states
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ZENABLE, 1); // TRUE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_STENCILENABLE, 0); // FALSE
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_LIGHTING, lighting);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_FOGENABLE, fog);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_SRCBLEND, srcblend);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_DESTBLEND, destblend);
			(*glw_state).device.as_mut().unwrap().SetRenderState(D3DRS_ALPHABLENDENABLE, alphablend);
		}
	}
}

use core::ptr::{addr_of, addr_of_mut};
