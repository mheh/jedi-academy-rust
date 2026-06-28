// Copyright (C) 1999-2000 Id Software, Inc.
//

// cg_localents.c -- every frame, generate renderer commands for locally
// processed entities, like smoke puffs, gibs, shells, etc.

use core::ffi::c_int;
use crate::codemp::cgame::cg_local_h::*;
use core::ptr::{addr_of, addr_of_mut};

const MAX_LOCAL_ENTITIES: usize = 512;

// Global arrays - zero-initialized in BSS semantics (per C)
// These will be initialized by CG_InitLocalEntities at startup
#[allow(non_upper_case_globals)]
pub static mut cg_localEntities: [localEntity_t; MAX_LOCAL_ENTITIES] = unsafe { core::mem::zeroed() };

#[allow(non_upper_case_globals)]
pub static mut cg_activeLocalEntities: localEntity_t = unsafe { core::mem::zeroed() };

#[allow(non_upper_case_globals)]
pub static mut cg_freeLocalEntities: *mut localEntity_t = core::ptr::null_mut();

/*
===================
CG_InitLocalEntities

This is called at startup and for tournement restarts
===================
*/
pub fn CG_InitLocalEntities() {
	let mut i: c_int = 0;

	unsafe {
		core::ptr::write_bytes(
			addr_of_mut!(cg_localEntities) as *mut u8,
			0,
			core::mem::size_of_val(&cg_localEntities),
		);
		cg_activeLocalEntities.next = addr_of_mut!(cg_activeLocalEntities);
		cg_activeLocalEntities.prev = addr_of_mut!(cg_activeLocalEntities);
		cg_freeLocalEntities = addr_of_mut!(cg_localEntities[0]);
		while i < (MAX_LOCAL_ENTITIES as c_int) - 1 {
			cg_localEntities[i as usize].next = addr_of_mut!(cg_localEntities[(i + 1) as usize]);
			i += 1;
		}
	}
}


/*
==================
CG_FreeLocalEntity
==================
*/
pub fn CG_FreeLocalEntity(le: *mut localEntity_t) {
	unsafe {
		if (*le).prev.is_null() {
			CG_Error(b"CG_FreeLocalEntity: not active\0".as_ptr() as *const i8);
		}

		// remove from the doubly linked active list
		(*(*le).prev).next = (*le).next;
		(*(*le).next).prev = (*le).prev;

		// the free list is only singly linked
		(*le).next = cg_freeLocalEntities;
		cg_freeLocalEntities = le;
	}
}

/*
===================
CG_AllocLocalEntity

Will allways succeed, even if it requires freeing an old active entity
===================
*/
pub fn CG_AllocLocalEntity() -> *mut localEntity_t {
	let mut le: *mut localEntity_t;

	unsafe {
		if cg_freeLocalEntities.is_null() {
			// no free entities, so free the one at the end of the chain
			// remove the oldest active entity
			CG_FreeLocalEntity(cg_activeLocalEntities.prev);
		}

		le = cg_freeLocalEntities;
		cg_freeLocalEntities = (*cg_freeLocalEntities).next;

		core::ptr::write_bytes(
			le as *mut u8,
			0,
			core::mem::size_of::<localEntity_t>(),
		);

		// link into the active list
		(*le).next = cg_activeLocalEntities.next;
		(*le).prev = addr_of_mut!(cg_activeLocalEntities);
		(*cg_activeLocalEntities.next).prev = le;
		cg_activeLocalEntities.next = le;
	}
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
CG_BloodTrail

Leave expanding blood puffs behind gibs
================
*/
pub fn CG_BloodTrail(le: *mut localEntity_t) {
	let mut t: c_int;
	let mut t2: c_int;
	let step: c_int = 150;
	let mut newOrigin: vec3_t = [0.0; 3];
	let mut blood: *mut localEntity_t;

	unsafe {
		t = step * (((cg.time - cg.frametime + step) / step) as c_int);
		t2 = step * ((cg.time / step) as c_int);

		while t <= t2 {
			BG_EvaluateTrajectory(addr_of!((*le).pos), t, newOrigin.as_mut_ptr());

			blood = CG_SmokePuff(
				newOrigin.as_ptr(),
				vec3_origin.as_ptr(),
				20, // radius
				1.0,
				1.0,
				1.0,
				1.0, // color
				2000, // trailTime
				t,    // startTime
				0,    // fadeInTime
				0,    // flags
				0,    /*cgs.media.bloodTrailShader*/
			);
			// use the optimized version
			(*blood).leType = LE_FALL_SCALE_FADE;
			// drop a total of 40 units over its lifetime
			(*blood).pos.trDelta[2] = 40.0;
			t += step;
		}
	}
}


/*
================
CG_FragmentBounceMark
================
*/
pub fn CG_FragmentBounceMark(le: *mut localEntity_t, trace: *mut trace_t) {
	let mut radius: c_int;

	unsafe {
		if (*le).leMarkType == LEMT_BLOOD {
			radius = 16 + (rand() & 31);
			//		CG_ImpactMark( cgs.media.bloodMarkShader, trace->endpos, trace->plane.normal, random()*360,
			//			1,1,1,1, qtrue, radius, qfalse );
		} else if (*le).leMarkType == LEMT_BURN {
			radius = 8 + (rand() & 15);
			//		CG_ImpactMark( cgs.media.burnMarkShader, trace->endpos, trace->plane.normal, random()*360,
			//			1,1,1,1, qtrue, radius, qfalse );
		}

		// don't allow a fragment to make multiple marks, or they
		// pile up while settling
		(*le).leMarkType = LEMT_NONE;
	}
}

/*
================
CG_FragmentBounceSound
================
*/
pub fn CG_FragmentBounceSound(le: *mut localEntity_t, trace: *mut trace_t) {
	unsafe {
		// half the fragments will make a bounce sounds
		if rand() & 1 != 0 {
			let mut s: sfxHandle_t = 0;

			match (*le).leBounceSoundType {
				LEBS_ROCK => {
					s = cgs.media.rockBounceSound[Q_irand(0, 1) as usize];
				}
				LEBS_METAL => {
					s = cgs.media.metalBounceSound[Q_irand(0, 1) as usize]; // FIXME: make sure that this sound is registered properly...might still be rock bounce sound....
				}
				_ => {
					return;
				}
			}

			if s != 0 {
				trap_S_StartSound((*trace).endpos.as_ptr(), ENTITYNUM_WORLD, CHAN_AUTO, s);
			}

			// bouncers only make the sound once...
			// FIXME: arbitrary...change if it bugs you
			(*le).leBounceSoundType = LEBS_NONE;
		} else if rand() & 1 != 0 {
			// we may end up bouncing again, but each bounce reduces the chance of playing the sound again or they may make a lot of noise when they settle
			// FIXME: maybe just always do this??
			(*le).leBounceSoundType = LEBS_NONE;
		}
	}
}


/*
================
CG_ReflectVelocity
================
*/
pub fn CG_ReflectVelocity(le: *mut localEntity_t, trace: *mut trace_t) {
	let mut velocity: vec3_t = [0.0; 3];
	let mut dot: f32;
	let mut hitTime: c_int;

	unsafe {
		// reflect the velocity on the trace plane
		hitTime = cg.time - cg.frametime + (cg.frametime as f32 * (*trace).fraction) as c_int;
		BG_EvaluateTrajectoryDelta(addr_of!((*le).pos), hitTime, velocity.as_mut_ptr());
		dot = DotProduct(velocity.as_ptr(), (*trace).plane.normal.as_ptr());
		VectorMA(
			velocity.as_ptr(),
			-2.0 * dot,
			(*trace).plane.normal.as_ptr(),
			(*le).pos.trDelta.as_mut_ptr(),
		);

		VectorScale((*le).pos.trDelta.as_ptr(), (*le).bounceFactor, (*le).pos.trDelta.as_mut_ptr());

		VectorCopy((*trace).endpos.as_ptr(), (*le).pos.trBase.as_mut_ptr());
		(*le).pos.trTime = cg.time;

		// check for stop, making sure that even on low FPS systems it doesn't bobble
		if (*trace).allsolid
			|| ((*trace).plane.normal[2] > 0.0
				&& ((*le).pos.trDelta[2] < 40.0
					|| (*le).pos.trDelta[2] < -(cg.frametime as f32) * (*le).pos.trDelta[2]))
		{
			(*le).pos.trType = TR_STATIONARY;
		} else {
		}
	}
}

/*
================
CG_AddFragment
================
*/
pub fn CG_AddFragment(le: *mut localEntity_t) {
	let mut newOrigin: vec3_t = [0.0; 3];
	let mut trace: trace_t;

	unsafe {
		if (*le).forceAlpha != 0 {
			(*le).refEntity.renderfx |= RF_FORCE_ENT_ALPHA;
			(*le).refEntity.shaderRGBA[3] = (*le).forceAlpha;
		}

		if (*le).pos.trType == TR_STATIONARY {
			// sink into the ground if near the removal time
			let mut t: c_int;
			let mut t_e: f32;

			t = (*le).endTime - cg.time;
			if t < (SINK_TIME as c_int * 2) {
				(*le).refEntity.renderfx |= RF_FORCE_ENT_ALPHA;
				t_e = ((*le).endTime - cg.time) as f32 / (SINK_TIME * 2.0);
				t_e = ((t_e) * 255.0) as i32 as f32;

				if t_e > 255.0 {
					t_e = 255.0;
				}
				if t_e < 1.0 {
					t_e = 1.0;
				}

				if (*le).refEntity.shaderRGBA[3] != 0 && t_e > (*le).refEntity.shaderRGBA[3] as f32 {
					t_e = (*le).refEntity.shaderRGBA[3] as f32;
				}

				(*le).refEntity.shaderRGBA[3] = t_e as u8;

				trap_R_AddRefEntityToScene(addr_of_mut!((*le).refEntity));
			} else {
				trap_R_AddRefEntityToScene(addr_of_mut!((*le).refEntity));
			}

			return;
		}

		// calculate new position
		BG_EvaluateTrajectory(addr_of!((*le).pos), cg.time, newOrigin.as_mut_ptr());

		// trace a line from previous position to new position
		CG_Trace(
			addr_of_mut!(trace),
			(*le).refEntity.origin.as_ptr(),
			core::ptr::null(),
			core::ptr::null(),
			newOrigin.as_ptr(),
			-1,
			CONTENTS_SOLID,
		);
		if trace.fraction == 1.0 {
			// still in free fall
			VectorCopy(newOrigin.as_ptr(), (*le).refEntity.origin.as_mut_ptr());

			if (*le).leFlags & LEF_TUMBLE != 0 {
				let mut angles: vec3_t = [0.0; 3];

				BG_EvaluateTrajectory(addr_of!((*le).angles), cg.time, angles.as_mut_ptr());
				AnglesToAxis(angles.as_ptr(), (*le).refEntity.axis.as_mut_ptr());
				ScaleModelAxis(addr_of_mut!((*le).refEntity));
			}

			trap_R_AddRefEntityToScene(addr_of_mut!((*le).refEntity));

			// add a blood trail
			if (*le).leBounceSoundType == LEBS_BLOOD {
				CG_BloodTrail(le);
			}

			return;
		}

		// if it is in a nodrop zone, remove it
		// this keeps gibs from waiting at the bottom of pits of death
		// and floating levels
		if trap_CM_PointContents(trace.endpos.as_ptr(), 0) & CONTENTS_NODROP != 0 {
			CG_FreeLocalEntity(le);
			return;
		}

		if !trace.startsolid {
			// leave a mark
			CG_FragmentBounceMark(le, addr_of_mut!(trace));

			// do a bouncy sound
			CG_FragmentBounceSound(le, addr_of_mut!(trace));

			if (*le).bounceSound != 0 {
				// specified bounce sound (debris)
				trap_S_StartSound((*le).pos.trBase.as_ptr(), ENTITYNUM_WORLD, CHAN_AUTO, (*le).bounceSound);
			}

			// reflect the velocity on the trace plane
			CG_ReflectVelocity(le, addr_of_mut!(trace));

			trap_R_AddRefEntityToScene(addr_of_mut!((*le).refEntity));
		}
	}
}

/*
=====================================================================

TRIVIAL LOCAL ENTITIES

These only do simple scaling or modulation before passing to the renderer
=====================================================================
*/

/*
====================
CG_AddFadeRGB
====================
*/
pub fn CG_AddFadeRGB(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;
	let mut c: f32;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		c = ((*le).endTime - cg.time) as f32 * (*le).lifeRate;
		c *= 0xff as f32;

		(*re).shaderRGBA[0] = ((*le).color[0] * c) as u8;
		(*re).shaderRGBA[1] = ((*le).color[1] * c) as u8;
		(*re).shaderRGBA[2] = ((*le).color[2] * c) as u8;
		(*re).shaderRGBA[3] = ((*le).color[3] * c) as u8;

		trap_R_AddRefEntityToScene(re);
	}
}

pub fn CG_AddFadeScaleModel(le: *mut localEntity_t) {
	let mut ent: *mut refEntity_t;
	let mut frac: f32;

	unsafe {
		ent = addr_of_mut!((*le).refEntity);

		frac = (cg.time - (*le).startTime) as f32 / (((*le).endTime - (*le).startTime) as f32);

		frac = frac * frac * frac; // yes, this is completely ridiculous...but it causes the shell to grow slowly then "explode" at the end

		(*ent).nonNormalizedAxes = qtrue;

		AxisCopy(axisDefault.as_ptr(), (*ent).axis.as_mut_ptr());

		VectorScale((*ent).axis[0].as_ptr(), (*le).radius * frac, (*ent).axis[0].as_mut_ptr());
		VectorScale((*ent).axis[1].as_ptr(), (*le).radius * frac, (*ent).axis[1].as_mut_ptr());
		VectorScale(
			(*ent).axis[2].as_ptr(),
			(*le).radius * 0.5 * frac,
			(*ent).axis[2].as_mut_ptr(),
		);

		frac = 1.0 - frac;

		(*ent).shaderRGBA[0] = ((*le).color[0] * frac) as u8;
		(*ent).shaderRGBA[1] = ((*le).color[1] * frac) as u8;
		(*ent).shaderRGBA[2] = ((*le).color[2] * frac) as u8;
		(*ent).shaderRGBA[3] = ((*le).color[3] * frac) as u8;

		// add the entity
		trap_R_AddRefEntityToScene(ent);
	}
}

/*
==================
CG_AddMoveScaleFade
==================
*/
pub fn CG_AddMoveScaleFade(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;
	let mut c: f32;
	let mut delta: vec3_t = [0.0; 3];
	let mut len: f32;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		if (*le).fadeInTime > (*le).startTime && cg.time < (*le).fadeInTime {
			// fade / grow time
			c = 1.0 - ((*le).fadeInTime - cg.time) as f32 / ((*le).fadeInTime - (*le).startTime) as f32;
		} else {
			// fade / grow time
			c = ((*le).endTime - cg.time) as f32 * (*le).lifeRate;
		}

		(*re).shaderRGBA[3] = ((0xff as f32) * c * (*le).color[3]) as u8;

		if (*le).leFlags & LEF_PUFF_DONT_SCALE == 0 {
			(*re).radius = (*le).radius * (1.0 - c) + 8.0;
		}

		BG_EvaluateTrajectory(addr_of!((*le).pos), cg.time, (*re).origin.as_mut_ptr());

		// if the view would be "inside" the sprite, kill the sprite
		// so it doesn't add too much overdraw
		VectorSubtract((*re).origin.as_ptr(), cg.refdef.vieworg.as_ptr(), delta.as_mut_ptr());
		len = VectorLength(delta.as_ptr());
		if len < (*le).radius {
			CG_FreeLocalEntity(le);
			return;
		}

		trap_R_AddRefEntityToScene(re);
	}
}

/*
==================
CG_AddPuff
==================
*/
pub fn CG_AddPuff(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;
	let mut c: f32;
	let mut delta: vec3_t = [0.0; 3];
	let mut len: f32;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		// fade / grow time
		c = ((*le).endTime - cg.time) as f32 / ((*le).endTime - (*le).startTime) as f32;

		(*re).shaderRGBA[0] = ((*le).color[0] * c) as u8;
		(*re).shaderRGBA[1] = ((*le).color[1] * c) as u8;
		(*re).shaderRGBA[2] = ((*le).color[2] * c) as u8;

		if (*le).leFlags & LEF_PUFF_DONT_SCALE == 0 {
			(*re).radius = (*le).radius * (1.0 - c) + 8.0;
		}

		BG_EvaluateTrajectory(addr_of!((*le).pos), cg.time, (*re).origin.as_mut_ptr());

		// if the view would be "inside" the sprite, kill the sprite
		// so it doesn't add too much overdraw
		VectorSubtract((*re).origin.as_ptr(), cg.refdef.vieworg.as_ptr(), delta.as_mut_ptr());
		len = VectorLength(delta.as_ptr());
		if len < (*le).radius {
			CG_FreeLocalEntity(le);
			return;
		}

		trap_R_AddRefEntityToScene(re);
	}
}

/*
===================
CG_AddScaleFade

For rocket smokes that hang in place, fade out, and are
removed if the view passes through them.
There are often many of these, so it needs to be simple.
===================
*/
pub fn CG_AddScaleFade(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;
	let mut c: f32;
	let mut delta: vec3_t = [0.0; 3];
	let mut len: f32;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		// fade / grow time
		c = ((*le).endTime - cg.time) as f32 * (*le).lifeRate;

		(*re).shaderRGBA[3] = ((0xff as f32) * c * (*le).color[3]) as u8;
		(*re).radius = (*le).radius * (1.0 - c) + 8.0;

		// if the view would be "inside" the sprite, kill the sprite
		// so it doesn't add too much overdraw
		VectorSubtract((*re).origin.as_ptr(), cg.refdef.vieworg.as_ptr(), delta.as_mut_ptr());
		len = VectorLength(delta.as_ptr());
		if len < (*le).radius {
			CG_FreeLocalEntity(le);
			return;
		}

		trap_R_AddRefEntityToScene(re);
	}
}


/*
=================
CG_AddFallScaleFade

This is just an optimized CG_AddMoveScaleFade
For blood mists that drift down, fade out, and are
removed if the view passes through them.
There are often 100+ of these, so it needs to be simple.
=================
*/
pub fn CG_AddFallScaleFade(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;
	let mut c: f32;
	let mut delta: vec3_t = [0.0; 3];
	let mut len: f32;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		// fade time
		c = ((*le).endTime - cg.time) as f32 * (*le).lifeRate;

		(*re).shaderRGBA[3] = ((0xff as f32) * c * (*le).color[3]) as u8;

		(*re).origin[2] = (*le).pos.trBase[2] - (1.0 - c) * (*le).pos.trDelta[2];

		(*re).radius = (*le).radius * (1.0 - c) + 16.0;

		// if the view would be "inside" the sprite, kill the sprite
		// so it doesn't add too much overdraw
		VectorSubtract((*re).origin.as_ptr(), cg.refdef.vieworg.as_ptr(), delta.as_mut_ptr());
		len = VectorLength(delta.as_ptr());
		if len < (*le).radius {
			CG_FreeLocalEntity(le);
			return;
		}

		trap_R_AddRefEntityToScene(re);
	}
}



/*
================
CG_AddExplosion
================
*/
pub fn CG_AddExplosion(ex: *mut localEntity_t) {
	let mut ent: *mut refEntity_t;

	unsafe {
		ent = addr_of_mut!((*ex).refEntity);

		// add the entity
		trap_R_AddRefEntityToScene(ent);

		// add the dlight
		if (*ex).light != 0.0 {
			let mut light: f32;

			light = (cg.time - (*ex).startTime) as f32 / ((*ex).endTime - (*ex).startTime) as f32;
			if light < 0.5 {
				light = 1.0;
			} else {
				light = 1.0 - (light - 0.5) * 2.0;
			}
			light = (*ex).light * light;
			trap_R_AddLightToScene(
				(*ent).origin.as_ptr(),
				light,
				(*ex).lightColor[0],
				(*ex).lightColor[1],
				(*ex).lightColor[2],
			);
		}
	}
}

/*
================
CG_AddSpriteExplosion
================
*/
pub fn CG_AddSpriteExplosion(le: *mut localEntity_t) {
	let mut re: refEntity_t;
	let mut c: f32;

	unsafe {
		re = (*le).refEntity;

		c = ((*le).endTime - cg.time) as f32 / ((*le).endTime - (*le).startTime) as f32;
		if c > 1.0 {
			c = 1.0; // can happen during connection problems
		}

		re.shaderRGBA[0] = 0xff;
		re.shaderRGBA[1] = 0xff;
		re.shaderRGBA[2] = 0xff;
		re.shaderRGBA[3] = ((0xff as f32) * c * 0.33) as u8;

		re.reType = RT_SPRITE;
		re.radius = 42.0 * (1.0 - c) + 30.0;

		trap_R_AddRefEntityToScene(addr_of_mut!(re));

		// add the dlight
		if (*le).light != 0.0 {
			let mut light: f32;

			light = (cg.time - (*le).startTime) as f32 / ((*le).endTime - (*le).startTime) as f32;
			if light < 0.5 {
				light = 1.0;
			} else {
				light = 1.0 - (light - 0.5) * 2.0;
			}
			light = (*le).light * light;
			trap_R_AddLightToScene(
				re.origin.as_ptr(),
				light,
				(*le).lightColor[0],
				(*le).lightColor[1],
				(*le).lightColor[2],
			);
		}
	}
}


/*
===================
CG_AddRefEntity
===================
*/
pub fn CG_AddRefEntity(le: *mut localEntity_t) {
	unsafe {
		if (*le).endTime < cg.time {
			CG_FreeLocalEntity(le);
			return;
		}
		trap_R_AddRefEntityToScene(addr_of_mut!((*le).refEntity));
	}
}

/*
===================
CG_AddScorePlum
===================
*/
const NUMBER_SIZE: c_int = 8;

pub fn CG_AddScorePlum(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;
	let mut origin: vec3_t = [0.0; 3];
	let mut delta: vec3_t = [0.0; 3];
	let mut dir: vec3_t = [0.0; 3];
	let mut vec: vec3_t = [0.0; 3];
	let up: vec3_t = [0.0, 0.0, 1.0];
	let mut c: f32;
	let mut len: f32;
	let mut i: c_int;
	let mut score: c_int;
	let mut digits: [c_int; 10] = [0; 10];
	let mut numdigits: c_int;
	let mut negative: c_int;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		c = ((*le).endTime - cg.time) as f32 * (*le).lifeRate;

		score = (*le).radius as c_int;
		if score < 0 {
			(*re).shaderRGBA[0] = 0xff;
			(*re).shaderRGBA[1] = 0x11;
			(*re).shaderRGBA[2] = 0x11;
		} else {
			(*re).shaderRGBA[0] = 0xff;
			(*re).shaderRGBA[1] = 0xff;
			(*re).shaderRGBA[2] = 0xff;
			if score >= 50 {
				(*re).shaderRGBA[1] = 0;
			} else if score >= 20 {
				(*re).shaderRGBA[0] = 0;
				(*re).shaderRGBA[1] = 0;
			} else if score >= 10 {
				(*re).shaderRGBA[2] = 0;
			} else if score >= 2 {
				(*re).shaderRGBA[0] = 0;
				(*re).shaderRGBA[2] = 0;
			}
		}
		if c < 0.25 {
			(*re).shaderRGBA[3] = ((0xff as f32) * 4.0 * c) as u8;
		} else {
			(*re).shaderRGBA[3] = 0xff;
		}

		(*re).radius = NUMBER_SIZE as f32 / 2.0;

		VectorCopy((*le).pos.trBase.as_ptr(), origin.as_mut_ptr());
		origin[2] += 110.0 - c * 100.0;

		VectorSubtract(cg.refdef.vieworg.as_ptr(), origin.as_ptr(), dir.as_mut_ptr());
		CrossProduct(dir.as_ptr(), up.as_ptr(), vec.as_mut_ptr());
		VectorNormalize(vec.as_mut_ptr());

		VectorMA(
			origin.as_ptr(),
			-10.0 + 20.0 * (c * 2.0 * core::f32::consts::PI).sin(),
			vec.as_ptr(),
			origin.as_mut_ptr(),
		);

		// if the view would be "inside" the sprite, kill the sprite
		// so it doesn't add too much overdraw
		VectorSubtract(origin.as_ptr(), cg.refdef.vieworg.as_ptr(), delta.as_mut_ptr());
		len = VectorLength(delta.as_ptr());
		if len < 20.0 {
			CG_FreeLocalEntity(le);
			return;
		}

		negative = qfalse;
		if score < 0 {
			negative = qtrue;
			score = -score;
		}

		numdigits = 0;
		loop {
			digits[numdigits as usize] = score % 10;
			score = score / 10;
			numdigits += 1;
			if numdigits != 0 && score == 0 {
				break;
			}
		}

		if negative != 0 {
			digits[numdigits as usize] = 10;
			numdigits += 1;
		}

		i = 0;
		while i < numdigits {
			VectorMA(
				origin.as_ptr(),
				((numdigits as f32 / 2.0) - i as f32) * NUMBER_SIZE as f32,
				vec.as_ptr(),
				(*re).origin.as_mut_ptr(),
			);
			(*re).customShader = cgs.media.numberShaders[digits[(numdigits - 1 - i) as usize] as usize];
			trap_R_AddRefEntityToScene(re);
			i += 1;
		}
	}
}

/*
===================
CG_AddOLine

For forcefields/other rectangular things
===================
*/
pub fn CG_AddOLine(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;
	let mut frac: f32;
	let mut alpha: f32;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		frac = (cg.time - (*le).startTime) as f32 / ((*le).endTime - (*le).startTime) as f32;
		if frac > 1.0 {
			frac = 1.0; // can happen during connection problems
		} else if frac < 0.0 {
			frac = 0.0;
		}

		// Use the liferate to set the scale over time.
		(*re).data.line.width = (*le).data.line.width + ((*le).data.line.dwidth * frac);
		if (*re).data.line.width <= 0.0 {
			CG_FreeLocalEntity(le);
			return;
		}

		// We will assume here that we want additive transparency effects.
		alpha = (*le).alpha + ((*le).dalpha * frac);
		(*re).shaderRGBA[0] = ((0xff as f32) * alpha) as u8;
		(*re).shaderRGBA[1] = ((0xff as f32) * alpha) as u8;
		(*re).shaderRGBA[2] = ((0xff as f32) * alpha) as u8;
		(*re).shaderRGBA[3] = ((0xff as f32) * alpha) as u8; // Yes, we could apply c to this too, but fading the color is better for lines.

		(*re).shaderTexCoord[0] = 1;
		(*re).shaderTexCoord[1] = 1;

		(*re).rotation = 90;

		(*re).reType = RT_ORIENTEDLINE;

		trap_R_AddRefEntityToScene(re);
	}
}

/*
===================
CG_AddLine

for beams and the like.
===================
*/
pub fn CG_AddLine(le: *mut localEntity_t) {
	let mut re: *mut refEntity_t;

	unsafe {
		re = addr_of_mut!((*le).refEntity);

		(*re).reType = RT_LINE;

		trap_R_AddRefEntityToScene(re);
	}
}

//==============================================================================

/*
===================
CG_AddLocalEntities

===================
*/
pub fn CG_AddLocalEntities() {
	let mut le: *mut localEntity_t;
	let mut next: *mut localEntity_t;

	unsafe {
		// walk the list backwards, so any new local entities generated
		// (trails, marks, etc) will be present this frame
		le = cg_activeLocalEntities.prev;
		loop {
			if le == addr_of_mut!(cg_activeLocalEntities) {
				break;
			}
			// grab next now, so if the local entity is freed we
			// still have it
			next = (*le).prev;

			if cg.time >= (*le).endTime {
				CG_FreeLocalEntity(le);
			} else {
				match (*le).leType {
					LE_MARK => {}

					LE_SPRITE_EXPLOSION => {
						CG_AddSpriteExplosion(le);
					}

					LE_EXPLOSION => {
						CG_AddExplosion(le);
					}

					LE_FADE_SCALE_MODEL => {
						CG_AddFadeScaleModel(le);
					}

					LE_FRAGMENT => {
						// gibs and brass
						CG_AddFragment(le);
					}

					LE_PUFF => {
						CG_AddPuff(le);
					}

					LE_MOVE_SCALE_FADE => {
						// water bubbles
						CG_AddMoveScaleFade(le);
					}

					LE_FADE_RGB => {
						// teleporters, railtrails
						CG_AddFadeRGB(le);
					}

					LE_FALL_SCALE_FADE => {
						// gib blood trails
						CG_AddFallScaleFade(le);
					}

					LE_SCALE_FADE => {
						// rocket trails
						CG_AddScaleFade(le);
					}

					LE_SCOREPLUM => {
						CG_AddScorePlum(le);
					}

					LE_OLINE => {
						CG_AddOLine(le);
					}

					LE_SHOWREFENTITY => {
						CG_AddRefEntity(le);
					}

					LE_LINE => {
						// oriented lines for FX
						CG_AddLine(le);
					}

					_ => {
						CG_Error(b"Bad leType: %i\0".as_ptr() as *const i8, (*le).leType);
					}
				}
			}
			le = next;
		}
	}
}

// External function declarations needed from other modules
// Porting note: CG_Error is variadic in C; we wrap calls to pass formatted strings
extern "C" {
	#[allow(improper_ctypes)]
	pub fn CG_Error(msg: *const i8, ...);

	pub fn CG_Trace(
		result: *mut trace_t,
		start: *const f32,
		mins: *const f32,
		maxs: *const f32,
		end: *const f32,
		passent: c_int,
		contentmask: c_int,
	);

	pub fn CG_SmokePuff(
		origin: *const f32,
		velocity: *const f32,
		radius: c_int,
		r: f32,
		g: f32,
		b: f32,
		a: f32,
		duration: c_int,
		startTime: c_int,
		fadeInTime: c_int,
		flags: c_int,
		shader: c_int,
	) -> *mut localEntity_t;

	pub fn BG_EvaluateTrajectory(tr: *const trajectory_t, atTime: c_int, result: *mut f32);
	pub fn BG_EvaluateTrajectoryDelta(tr: *const trajectory_t, atTime: c_int, result: *mut f32);
	pub fn DotProduct(a: *const f32, b: *const f32) -> f32;
	pub fn VectorMA(v: *const f32, scale: f32, dir: *const f32, out: *mut f32);
	pub fn VectorScale(v: *const f32, scale: f32, out: *mut f32);
	pub fn VectorCopy(src: *const f32, dst: *mut f32);
	pub fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
	pub fn VectorLength(v: *const f32) -> f32;
	pub fn VectorNormalize(v: *mut f32) -> f32;
	pub fn CrossProduct(a: *const f32, b: *const f32, out: *mut f32);
	pub fn AnglesToAxis(angles: *const f32, axis: *mut f32);
	pub fn AxisCopy(src: *const f32, dst: *mut f32);
	pub fn ScaleModelAxis(re: *mut refEntity_t);
	pub fn Q_irand(min: c_int, max: c_int) -> c_int;
	pub fn rand() -> c_int;
	pub fn trap_R_AddRefEntityToScene(re: *const refEntity_t);
	pub fn trap_R_AddLightToScene(origin: *const f32, intensity: f32, r: f32, g: f32, b: f32);
	pub fn trap_S_StartSound(origin: *const f32, entityNum: c_int, entchannel: c_int, sfxHandle: sfxHandle_t);
	pub fn trap_CM_PointContents(p: *const f32, model: c_int) -> c_int;

	#[allow(non_upper_case_globals)]
	pub static mut cg: cg_t;
	#[allow(non_upper_case_globals)]
	pub static mut cgs: cgs_t;
	#[allow(non_upper_case_globals)]
	pub static axisDefault: [vec3_t; 3];
	#[allow(non_upper_case_globals)]
	pub static vec3_origin: vec3_t;
}
