// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// g_headers.h included in C
// b_local.h included in C

use core::ffi::c_int;

// These define the working combat range for these suckers
const MIN_DISTANCE: f32 = 54.0;
const MIN_DISTANCE_SQR: f32 = MIN_DISTANCE * MIN_DISTANCE;

const MAX_DISTANCE: f32 = 128.0;
const MAX_DISTANCE_SQR: f32 = MAX_DISTANCE * MAX_DISTANCE;

const LSTATE_CLEAR: c_int = 0;
const LSTATE_WAITING: c_int = 1;
const LSTATE_FLEE: c_int = 2;
const LSTATE_BERZERK: c_int = 3;

const HOWLER_RETREAT_DIST: f32 = 300.0;
const HOWLER_PANIC_HEALTH: c_int = 10;

// External function declarations - these are implemented in other modules
extern "C" {
	fn G_UcmdMoveForDir(
		myself: *mut gentity_t,
		cmd: *mut usercmd_t,
		dir: *mut core::ffi::c_float,
	);
	fn G_GetBoltPosition(
		myself: *mut gentity_t,
		boltIndex: c_int,
		pos: *mut core::ffi::c_float,
		modelIndex: c_int,
	);
	fn PM_AnimLength(index: c_int, anim: animNumber_t) -> c_int;
	fn NAV_DirSafe(
		myself: *mut gentity_t,
		dir: *mut core::ffi::c_float,
		dist: core::ffi::c_float,
	) -> qboolean;
	fn G_Knockdown(
		myself: *mut gentity_t,
		attacker: *mut gentity_t,
		pushDir: *const core::ffi::c_float,
		strength: core::ffi::c_float,
		breakSaberLock: qboolean,
	);
	fn NPC_EntRangeFromBolt(targEnt: *mut gentity_t, boltIndex: c_int) -> core::ffi::c_float;
	fn NPC_GetEntsNearBolt(
		radiusEnts: *mut *mut gentity_t,
		radius: core::ffi::c_float,
		boltIndex: c_int,
		boltOrg: *mut core::ffi::c_float,
	) -> c_int;
	fn PM_InKnockDown(ps: *mut playerState_t) -> qboolean;
	fn PM_HasAnimation(ent: *mut gentity_t, animation: c_int) -> qboolean;

	fn G_EffectIndex(name: *const core::ffi::c_char) -> c_int;
	fn G_SoundIndex(name: *const core::ffi::c_char) -> c_int;
	fn va(fmt: *const core::ffi::c_char, ...) -> *mut core::ffi::c_char;
	fn TIMER_Set(ent: *mut gentity_t, label: *const core::ffi::c_char, duration: c_int);
	fn TIMER_Done(ent: *mut gentity_t, label: *const core::ffi::c_char) -> qboolean;
	fn TIMER_Done2(
		ent: *mut gentity_t,
		label: *const core::ffi::c_char,
		remove: qboolean,
	) -> qboolean;
	fn TIMER_Remove(ent: *mut gentity_t, label: *const core::ffi::c_char);
	fn TIMER_Exists(ent: *mut gentity_t, label: *const core::ffi::c_char) -> qboolean;
	fn UpdateGoal() -> qboolean;
	fn NPC_Howler_Move(randomJumpChance: c_int) -> qboolean;
	fn NPC_MoveToGoal(allowGoalChange: qboolean) -> qboolean;
	fn VectorCompare(v1: *const core::ffi::c_float, v2: *const core::ffi::c_float) -> qboolean;
	fn VectorClear(v: *mut core::ffi::c_float);
	fn VectorCopy(src: *const core::ffi::c_float, dst: *mut core::ffi::c_float);
	fn VectorSubtract(va: *const core::ffi::c_float, vb: *const core::ffi::c_float, vc: *mut core::ffi::c_float);
	fn VectorLengthSquared(v: *const core::ffi::c_float) -> core::ffi::c_float;
	fn VectorNormalize(v: *mut core::ffi::c_float) -> core::ffi::c_float;
	fn VectorMA(
		veca: *const core::ffi::c_float,
		scale: core::ffi::c_float,
		vecb: *const core::ffi::c_float,
		vecc: *mut core::ffi::c_float,
	);
	fn VectorScale(v: *const core::ffi::c_float, scale: core::ffi::c_float, out: *mut core::ffi::c_float);
	fn AngleVectors(
		angles: *const core::ffi::c_float,
		forward: *mut core::ffi::c_float,
		right: *mut core::ffi::c_float,
		up: *mut core::ffi::c_float,
	);
	fn Distance(p1: *const core::ffi::c_float, p2: *const core::ffi::c_float) -> core::ffi::c_float;
	fn DistanceSquared(p1: *const core::ffi::c_float, p2: *const core::ffi::c_float) -> core::ffi::c_float;
	fn DistanceHorizontal(p1: *const core::ffi::c_float, p2: *const core::ffi::c_float) -> core::ffi::c_float;
	fn G_SetEnemy(self_: *mut gentity_t, enemy: *mut gentity_t);
	fn NPC_CheckEnemyExt(checkAlerts: qboolean) -> qboolean;
	fn G_Damage(
		targ: *mut gentity_t,
		inflictor: *mut gentity_t,
		attacker: *mut gentity_t,
		dir: *mut core::ffi::c_float,
		point: *const core::ffi::c_float,
		damage: c_int,
		dflags: c_int,
		mod_: c_int,
	);
	fn AddSoundEvent(
		owner: *mut gentity_t,
		pos: *const core::ffi::c_float,
		radius: core::ffi::c_float,
		level: c_int,
		loud: qboolean,
		breakGlass: qboolean,
	);
	fn NPC_SetAnim(
		ent: *mut gentity_t,
		setAnimParts: c_int,
		anim: c_int,
		flags: c_int,
	);
	fn NPC_FaceEntity(ent: *mut gentity_t, dontInterrupt: qboolean);
	fn NPC_UpdateAngles(lookTowardsEnemy: qboolean, updateClient: qboolean);
	fn NPC_TryJump(goal: *mut gentity_t, maxHeight: core::ffi::c_float, maxDist: core::ffi::c_float);
	fn NPC_ClearLOS(ent: *mut gentity_t) -> qboolean;
	fn NPC_FaceEnemy(dontInterrupt: qboolean);
	fn NPC_BSFlee() -> qboolean;
	fn NPC_ValidEnemy(ent: *mut gentity_t) -> qboolean;
	fn NPC_CheckEnemy(
		checkAlerts: qboolean,
		ignoreTeam: qboolean,
		tooFarOk: qboolean,
	) -> *mut gentity_t;
	fn G_PlayEffect(
		index: c_int,
		org: *const core::ffi::c_float,
		ang: *const core::ffi::c_float,
		boltNum: c_int,
		entNum: c_int,
		duration: c_int,
		isRelative: qboolean,
	);
	fn G_StopEffect(index: c_int, org: *const core::ffi::c_float, boltNum: c_int, entNum: c_int);
	fn G_SoundOnEnt(ent: *mut gentity_t, channel: c_int, sound: *const core::ffi::c_char);
	fn CGCam_Shake(intensity: core::ffi::c_float, duration: c_int);
	fn Q_irand(low: c_int, high: c_int) -> c_int;
	#[cfg(not(final_build))]
	fn G_DebugLine(
		start: *const core::ffi::c_float,
		end: *const core::ffi::c_float,
		duration: c_int,
		color: core::ffi::c_uint,
		twoSided: qboolean,
	);

	// Global variables from other modules
	static mut NPC: *mut gentity_t;
	static mut NPCInfo: *mut gNPC_t;
	static mut ucmd: usercmd_t;
	static mut level: level_locals_t;
	static mut player: *mut gentity_t;
	static mut g_entities: *mut gentity_t;
	static mut d_saberCombat: *mut cvar_t;
	static mut g_spskill: *mut cvar_t;
	static mut vec3_origin: [core::ffi::c_float; 3];
}

// Type stubs for external dependencies
#[repr(C)]
pub struct gentity_t {
	// Stub - real definition in other modules
	pub s: entityState_t,
	pub r: entityShared_t,
	pub client: *mut gclient_t,
	pub currentOrigin: [core::ffi::c_float; 3],
	pub currentAngles: [core::ffi::c_float; 3],
	pub inuse: qboolean,
	pub count: c_int,
	pub health: c_int,
	pub max_health: c_int,
	pub NPC: *mut gNPC_t,
	pub enemy: *mut gentity_t,
	pub lastEnemy: *mut gentity_t,
	pub useDebounceTime: c_int,
	pub genericBolt1: c_int,
	pub genericBolt2: c_int,
	pub handLBolt: c_int,
	pub playerModel: c_int,
	pub spawnflags: c_int,
	// ... other fields omitted for stub
}

#[repr(C)]
pub struct entityState_t {
	pub number: c_int,
	pub time: c_int,
	// ... stub
}

#[repr(C)]
pub struct entityShared_t {
	// ... stub
}

#[repr(C)]
pub struct gclient_t {
	pub ps: playerState_t,
	pub NPC_class: c_int,
	pub clientInfo: clientInfo_t,
	// ... stub
}

#[repr(C)]
pub struct playerState_t {
	pub groundEntityNum: c_int,
	pub moveDir: [core::ffi::c_float; 3],
	pub speed: core::ffi::c_float,
	pub viewangles: [core::ffi::c_float; 3],
	pub velocity: [core::ffi::c_float; 3],
	pub legsAnim: c_int,
	pub torsoAnim: c_int,
	pub legsAnimTimer: c_int,
	pub torsoAnimTimer: c_int,
	pub weaponTime: c_int,
	// ... stub
}

#[repr(C)]
pub struct clientInfo_t {
	pub animFileIndex: c_int,
	// ... stub
}

#[repr(C)]
pub struct gNPC_t {
	pub localState: c_int,
	pub stats: npcStats_t,
	pub goalEntity: *mut gentity_t,
	pub goalRadius: core::ffi::c_float,
	pub desiredYaw: core::ffi::c_float,
	pub lockedDesiredYaw: core::ffi::c_float,
	pub lastPathAngles: [core::ffi::c_float; 3],
	pub last_ucmd: usercmd_t,
	pub confusionTime: c_int,
	pub scriptFlags: c_int,
	// ... stub
}

#[repr(C)]
pub struct npcStats_t {
	pub aggression: c_int,
	pub walkSpeed: core::ffi::c_float,
	pub runSpeed: core::ffi::c_float,
	// ... stub
}

#[repr(C)]
pub struct usercmd_t {
	pub buttons: c_int,
	pub forwardmove: core::ffi::c_float,
	pub rightmove: core::ffi::c_float,
	// ... stub
}

#[repr(C)]
pub struct level_locals_t {
	pub time: c_int,
	// ... stub
}

#[repr(C)]
pub struct trace_t {
	pub entityNum: c_int,
	pub endpos: [core::ffi::c_float; 3],
	// ... stub
}

#[repr(C)]
pub struct cvar_t {
	pub integer: c_int,
	// ... stub
}

pub type qboolean = c_int;
pub type animNumber_t = c_int;

const ENTITYNUM_NONE: c_int = 1024;
const ENTITYNUM_WORLD: c_int = 1023;

const BUTTON_WALKING: c_int = 1;

const MASK_SHOT: c_int = 1;

const DAMAGE_NO_KNOCKBACK: c_int = 1;

const MOD_MELEE: c_int = 1;
const MOD_IMPACT: c_int = 2;

const BOTH_GESTURE1: c_int = 1;
const BOTH_ATTACK1: c_int = 2;
const BOTH_ATTACK2: c_int = 3;
const BOTH_MELEE1: c_int = 4;
const BOTH_MELEE2: c_int = 5;
const BOTH_JUMP1: c_int = 6;
const BOTH_INAIR1: c_int = 7;
const BOTH_PAIN1: c_int = 8;
const BOTH_SONICPAIN_START: c_int = 9;
const BOTH_SONICPAIN_HOLD: c_int = 10;

const SETANIM_BOTH: c_int = 0;
const SETANIM_LEGS: c_int = 1;
const SETANIM_TORSO: c_int = 2;
const SETANIM_FLAG_NORMAL: c_int = 0;
const SETANIM_FLAG_OVERRIDE: c_int = 1;
const SETANIM_FLAG_HOLD: c_int = 2;
const SETANIM_FLAG_RESTART: c_int = 4;

const CHAN_VOICE: c_int = 1;

const CLASS_HOWLER: c_int = 1;
const CLASS_RANCOR: c_int = 2;
const CLASS_ATST: c_int = 3;

const AEL_DANGER: c_int = 1;

const YAW: usize = 1;

const SCF_LOOK_FOR_ENEMIES: c_int = 1;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

fn qfalse_cond(b: bool) -> qboolean {
	if b { qtrue } else { qfalse }
}

// Stub for extern gi.trace
extern "C" {
	pub fn gi_trace(
		results: *mut trace_t,
		start: *const core::ffi::c_float,
		mins: *const core::ffi::c_float,
		maxs: *const core::ffi::c_float,
		end: *const core::ffi::c_float,
		passent: c_int,
		contentmask: c_int,
	);
}

static mut Howler_Attack_helper: fn(core::ffi::c_float, qboolean) = |_, _| {};

/*
-------------------------
NPC_Howler_Precache
-------------------------
*/
pub unsafe extern "C" fn NPC_Howler_Precache() {
	let mut i: c_int;
	//G_SoundIndex( "sound/chars/howler/howl.mp3" );
	G_EffectIndex(b"howler/sonic\0".as_ptr() as *const core::ffi::c_char);
	G_SoundIndex(b"sound/chars/howler/howl.mp3\0".as_ptr() as *const core::ffi::c_char);
	i = 1;
	while i < 3 {
		G_SoundIndex(va(
			b"sound/chars/howler/idle_hiss%d.mp3\0".as_ptr() as *const core::ffi::c_char,
			i,
		));
		i += 1;
	}
	i = 1;
	while i < 6 {
		G_SoundIndex(va(
			b"sound/chars/howler/howl_talk%d.mp3\0".as_ptr() as *const core::ffi::c_char,
			i,
		));
		G_SoundIndex(va(
			b"sound/chars/howler/howl_yell%d.mp3\0".as_ptr() as *const core::ffi::c_char,
			i,
		));
		i += 1;
	}
}

pub unsafe extern "C" fn Howler_ClearTimers(self_: *mut gentity_t) {
	//clear all my timers
	TIMER_Set(self_, b"flee\0".as_ptr() as *const core::ffi::c_char, -level.time);
	TIMER_Set(self_, b"retreating\0".as_ptr() as *const core::ffi::c_char, -level.time);
	TIMER_Set(self_, b"standing\0".as_ptr() as *const core::ffi::c_char, -level.time);
	TIMER_Set(self_, b"walking\0".as_ptr() as *const core::ffi::c_char, -level.time);
	TIMER_Set(self_, b"running\0".as_ptr() as *const core::ffi::c_char, -level.time);
	TIMER_Set(self_, b"aggressionDecay\0".as_ptr() as *const core::ffi::c_char, -level.time);
	TIMER_Set(self_, b"speaking\0".as_ptr() as *const core::ffi::c_char, -level.time);
}

static mut NPC_Howler_Move_impl: unsafe extern "C" fn(c_int) -> qboolean = |_| { qfalse };

static unsafe fn NPC_Howler_Move_local(mut randomJumpChance: c_int) -> qboolean {
	if !TIMER_Done(NPC, b"standing\0".as_ptr() as *const core::ffi::c_char) != 0 {
		//standing around
		return qfalse;
	}
	if (*NPC).client.as_ref().unwrap().ps.groundEntityNum == ENTITYNUM_NONE {
		//in air, don't do anything
		return qfalse;
	}
	if ((!(*NPC).enemy.is_null() && TIMER_Done(NPC, b"running\0".as_ptr() as *const core::ffi::c_char) != 0)
		|| !TIMER_Done(NPC, b"walking\0".as_ptr() as *const core::ffi::c_char) != 0)
	{
		ucmd.buttons |= BUTTON_WALKING;
	}
	if ((randomJumpChance == 0 || Q_irand(0, randomJumpChance) != 0) && NPC_MoveToGoal(qtrue) != 0)
	{
		if VectorCompare(
			(*NPC).client.as_ref().unwrap().ps.moveDir.as_ptr(),
			vec3_origin.as_ptr(),
		) != 0
			|| (*NPC).client.as_ref().unwrap().ps.speed == 0.0
		{
			//uh.... wtf?  Got there?
			if !NPCInfo.as_ref().unwrap().goalEntity.is_null() {
				NPC_FaceEntity(NPCInfo.as_ref().unwrap().goalEntity, qfalse);
			} else {
				NPC_UpdateAngles(qfalse, qtrue);
			}
			return qtrue;
		}
		//TEMP: don't want to strafe
		VectorClear((*NPC).client.as_mut().unwrap().ps.moveDir.as_mut_ptr());
		ucmd.rightmove = 0.0;
		//		Com_Printf( "Howler moving %d\n",ucmd.forwardmove );
		//if backing up, go slow...
		if ucmd.forwardmove < 0.0 {
			ucmd.buttons |= BUTTON_WALKING;
			//if ( NPC->client->ps.speed > NPCInfo->stats.walkSpeed )
			{
				//don't walk faster than I'm allowed to
				(*NPC).client.as_mut().unwrap().ps.speed = NPCInfo.as_ref().unwrap().stats.walkSpeed;
			}
		} else {
			if (ucmd.buttons & BUTTON_WALKING) != 0 {
				(*NPC).client.as_mut().unwrap().ps.speed = NPCInfo.as_ref().unwrap().stats.walkSpeed;
			} else {
				(*NPC).client.as_mut().unwrap().ps.speed = NPCInfo.as_ref().unwrap().stats.runSpeed;
			}
		}
		NPCInfo.as_mut().unwrap().lockedDesiredYaw =
			NPCInfo.as_mut().unwrap().desiredYaw =
				NPCInfo.as_ref().unwrap().lastPathAngles[YAW];
		NPC_UpdateAngles(qfalse, qtrue);
	} else if !NPCInfo.as_ref().unwrap().goalEntity.is_null() {
		//couldn't get where we wanted to go, try to jump there
		NPC_FaceEntity(NPCInfo.as_ref().unwrap().goalEntity, qfalse);
		NPC_TryJump(NPCInfo.as_ref().unwrap().goalEntity, 400.0, -256.0);
	}
	return qtrue;
}

/*
-------------------------
Howler_Idle
-------------------------
*/
static unsafe fn Howler_Idle() {}

/*
-------------------------
Howler_Patrol
-------------------------
*/
static unsafe fn Howler_Patrol() {
	NPCInfo.as_mut().unwrap().localState = LSTATE_CLEAR;

	//If we have somewhere to go, then do that
	if UpdateGoal() != 0 {
		NPC_Howler_Move(100);
	}

	let mut dif: [core::ffi::c_float; 3];
	dif = [0.0; 3];
	VectorSubtract(
		(*g_entities).currentOrigin.as_ptr(),
		(*NPC).currentOrigin.as_ptr(),
		dif.as_mut_ptr(),
	);

	if VectorLengthSquared(dif.as_ptr()) < 256.0 * 256.0 {
		G_SetEnemy(NPC, g_entities);
	}

	if NPC_CheckEnemyExt(qtrue) == qfalse {
		Howler_Idle();
		return;
	}

	Howler_Attack(0.0, qtrue);
}

/*
-------------------------
Howler_Move
-------------------------
*/
static unsafe fn Howler_Move(visible: qboolean) -> qboolean {
	if NPCInfo.as_ref().unwrap().localState != LSTATE_WAITING {
		NPCInfo.as_mut().unwrap().goalEntity = (*NPC).enemy;
		NPCInfo.as_mut().unwrap().goalRadius = MAX_DISTANCE; // just get us within combat range
		return NPC_Howler_Move(30);
	}
	return qfalse;
}

//---------------------------------------------------------
static unsafe fn Howler_TryDamage(damage: c_int, tongue: qboolean, knockdown: qboolean) {
	let mut start: [core::ffi::c_float; 3] = [0.0; 3];
	let mut end: [core::ffi::c_float; 3] = [0.0; 3];
	let mut dir: [core::ffi::c_float; 3] = [0.0; 3];
	let mut tr: trace_t = core::mem::zeroed();

	if tongue != 0 {
		G_GetBoltPosition(NPC, (*NPC).genericBolt1, start.as_mut_ptr(), 0);
		G_GetBoltPosition(NPC, (*NPC).genericBolt2, end.as_mut_ptr(), 0);
		VectorSubtract(end.as_ptr(), start.as_ptr(), dir.as_mut_ptr());
		let dist: core::ffi::c_float = VectorNormalize(dir.as_mut_ptr());
		VectorMA(start.as_ptr(), dist + 16.0, dir.as_ptr(), end.as_mut_ptr());
	} else {
		VectorCopy((*NPC).currentOrigin.as_ptr(), start.as_mut_ptr());
		AngleVectors(
			(*NPC).currentAngles.as_ptr(),
			dir.as_mut_ptr(),
			core::ptr::null_mut(),
			core::ptr::null_mut(),
		);
		VectorMA(
			start.as_ptr(),
			MIN_DISTANCE * 2.0,
			dir.as_ptr(),
			end.as_mut_ptr(),
		);
	}

	#[cfg(not(final_build))]
	{
		if (*d_saberCombat).integer > 1 {
			G_DebugLine(
				start.as_ptr(),
				end.as_ptr(),
				1000,
				0x000000ff,
				qtrue,
			);
		}
	}
	// Should probably trace from the mouth, but, ah well.
	gi_trace(
		&mut tr,
		start.as_ptr(),
		vec3_origin.as_ptr(),
		vec3_origin.as_ptr(),
		end.as_ptr(),
		(*NPC).s.number,
		MASK_SHOT,
	);

	if tr.entityNum < ENTITYNUM_WORLD {
		//hit *something*
		let victim: *mut gentity_t = &mut *g_entities.offset(tr.entityNum as isize);
		if victim.as_ref().unwrap().client.is_null()
			|| (*victim.as_ref().unwrap().client).NPC_class != CLASS_HOWLER
		{
			//not another howler

			if knockdown != 0 && !victim.as_ref().unwrap().client.is_null() {
				//only do damage if victim isn't knocked down.  If he isn't, knock him down
				if PM_InKnockDown(&mut (*victim.as_ref().unwrap().client).ps) != 0 {
					return;
				}
			}
			//FIXME: some sort of damage effect (claws and tongue are cutting you... blood?)
			G_Damage(
				victim,
				NPC,
				NPC,
				dir.as_mut_ptr(),
				tr.endpos.as_ptr(),
				damage,
				DAMAGE_NO_KNOCKBACK,
				MOD_MELEE,
			);
			if knockdown != 0 && victim.as_ref().unwrap().health > 0 {
				//victim still alive
				G_Knockdown(
					victim,
					NPC,
					(*NPC).client.as_ref().unwrap().ps.velocity.as_ptr(),
					500.0,
					qfalse,
				);
			}
		}
	}
}

static unsafe fn Howler_Howl() {
	let mut radiusEnts: [*mut gentity_t; 128] = [core::ptr::null_mut(); 128];
	let mut numEnts: c_int;
	let radius: core::ffi::c_float = if ((*NPC).spawnflags & 1) != 0 { 256.0 } else { 128.0 };
	let halfRadSquared: core::ffi::c_float = (radius / 2.0) * (radius / 2.0);
	let radiusSquared: core::ffi::c_float = radius * radius;
	let mut distSq: core::ffi::c_float;
	let mut i: c_int;
	let mut boltOrg: [core::ffi::c_float; 3] = [0.0; 3];

	AddSoundEvent(
		NPC,
		(*NPC).currentOrigin.as_ptr(),
		512.0,
		AEL_DANGER,
		qfalse,
		qtrue,
	);

	numEnts = NPC_GetEntsNearBolt(
		radiusEnts.as_mut_ptr(),
		radius,
		(*NPC).handLBolt,
		boltOrg.as_mut_ptr(),
	);

	i = 0;
	while i < numEnts {
		if !radiusEnts[i as usize].as_ref().unwrap().inuse != 0 {
			i += 1;
			continue;
		}

		if radiusEnts[i as usize] == NPC {
			//Skip the rancor ent
			i += 1;
			continue;
		}

		if radiusEnts[i as usize].as_ref().unwrap().client.is_null() {
			//must be a client
			i += 1;
			continue;
		}

		if (*radiusEnts[i as usize].as_ref().unwrap().client).NPC_class == CLASS_HOWLER {
			//other howlers immune
			i += 1;
			continue;
		}

		distSq = DistanceSquared(
			radiusEnts[i as usize].as_ref().unwrap().currentOrigin.as_ptr(),
			boltOrg.as_ptr(),
		);
		if distSq <= radiusSquared {
			if distSq < halfRadSquared {
				//close enough to do damage, too
				if Q_irand(0, (*g_spskill).integer) != 0 {
					//does no damage on easy, does 1 point every other frame on medium, more often on hard
					G_Damage(
						radiusEnts[i as usize],
						NPC,
						NPC,
						vec3_origin.as_mut_ptr() as *mut _,
						(*NPC).currentOrigin.as_ptr(),
						1,
						DAMAGE_NO_KNOCKBACK,
						MOD_IMPACT,
					);
				}
			}
			if radiusEnts[i as usize].as_ref().unwrap().health > 0
				&& !radiusEnts[i as usize].as_ref().unwrap().client.is_null()
				&& (*radiusEnts[i as usize].as_ref().unwrap().client).NPC_class != CLASS_RANCOR
				&& (*radiusEnts[i as usize].as_ref().unwrap().client).NPC_class != CLASS_ATST
				&& PM_InKnockDown(&mut (*radiusEnts[i as usize].as_ref().unwrap().client).ps) == qfalse
			{
				if PM_HasAnimation(radiusEnts[i as usize], BOTH_SONICPAIN_START) != 0 {
					if (*radiusEnts[i as usize].as_ref().unwrap().client).ps.torsoAnim
						!= BOTH_SONICPAIN_START
						&& (*radiusEnts[i as usize].as_ref().unwrap().client).ps.torsoAnim
							!= BOTH_SONICPAIN_HOLD
					{
						NPC_SetAnim(
							radiusEnts[i as usize],
							SETANIM_LEGS,
							BOTH_SONICPAIN_START,
							SETANIM_FLAG_NORMAL,
						);
						NPC_SetAnim(
							radiusEnts[i as usize],
							SETANIM_TORSO,
							BOTH_SONICPAIN_START,
							SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
						);
						(*radiusEnts[i as usize].as_ref().unwrap().client).ps.torsoAnimTimer += 100;
						(*radiusEnts[i as usize].as_ref().unwrap().client).ps.weaponTime =
							(*radiusEnts[i as usize].as_ref().unwrap().client).ps.torsoAnimTimer;
					} else if (*radiusEnts[i as usize].as_ref().unwrap().client).ps.torsoAnimTimer <= 100 {
						//at the end of the sonic pain start or hold anim
						NPC_SetAnim(
							radiusEnts[i as usize],
							SETANIM_LEGS,
							BOTH_SONICPAIN_HOLD,
							SETANIM_FLAG_NORMAL,
						);
						NPC_SetAnim(
							radiusEnts[i as usize],
							SETANIM_TORSO,
							BOTH_SONICPAIN_HOLD,
							SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
						);
						(*radiusEnts[i as usize].as_ref().unwrap().client).ps.torsoAnimTimer += 100;
						(*radiusEnts[i as usize].as_ref().unwrap().client).ps.weaponTime =
							(*radiusEnts[i as usize].as_ref().unwrap().client).ps.torsoAnimTimer;
					}
				}
				/*
				else if ( distSq < halfRadSquared
					&& radiusEnts[i]->client->ps.groundEntityNum != ENTITYNUM_NONE
					&& !Q_irand( 0, 10 ) )//FIXME: base on skill
				{//within range
					G_Knockdown( radiusEnts[i], NPC, vec3_origin, 500, qfalse );
				}
				*/
			}
		}

		i += 1;
	}

	let playerDist: core::ffi::c_float = NPC_EntRangeFromBolt(player, (*NPC).genericBolt1);
	if playerDist < 256.0 {
		CGCam_Shake(1.0 * playerDist / 128.0, 200);
	}
}

//------------------------------
static unsafe fn Howler_Attack(mut enemyDist: core::ffi::c_float, howl: qboolean) {
	let dmg: c_int = if NPCInfo.as_ref().unwrap().localState == LSTATE_BERZERK { 5 } else { 2 };

	if TIMER_Exists(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char) == 0 {
		let mut attackAnim: c_int = BOTH_GESTURE1;
		// Going to do an attack
		if !(*NPC).enemy.is_null()
			&& !(*NPC).enemy.as_ref().unwrap().client.is_null()
			&& PM_InKnockDown(&mut (*(*NPC).enemy.as_ref().unwrap().client).ps) != 0
			&& enemyDist <= MIN_DISTANCE
		{
			attackAnim = BOTH_ATTACK2;
		} else if Q_irand(0, 4) == 0 || howl != 0 {
			//howl attack
			//G_SoundOnEnt( NPC, CHAN_VOICE, "sound/chars/howler/howl.mp3" );
		} else if enemyDist > MIN_DISTANCE && Q_irand(0, 1) != 0 {
			//lunge attack
			//jump foward
			let mut fwd: [core::ffi::c_float; 3] = [0.0; 3];
			let mut yawAng: [core::ffi::c_float; 3] = [0.0, (*NPC).client.as_ref().unwrap().ps.viewangles[YAW], 0.0];
			AngleVectors(yawAng.as_ptr(), fwd.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut());
			VectorScale(
				fwd.as_ptr(),
				enemyDist * 3.0,
				(*NPC).client.as_mut().unwrap().ps.velocity.as_mut_ptr(),
			);
			(*NPC).client.as_mut().unwrap().ps.velocity[2] = 200.0;
			(*NPC).client.as_mut().unwrap().ps.groundEntityNum = ENTITYNUM_NONE;

			attackAnim = BOTH_ATTACK1;
		} else {
			//tongue attack
			attackAnim = BOTH_ATTACK2;
		}

		NPC_SetAnim(
			NPC,
			SETANIM_BOTH,
			attackAnim,
			SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD | SETANIM_FLAG_RESTART,
		);
		if NPCInfo.as_ref().unwrap().localState == LSTATE_BERZERK {
			//attack again right away
			TIMER_Set(
				NPC,
				b"attacking\0".as_ptr() as *const core::ffi::c_char,
				(*NPC).client.as_ref().unwrap().ps.legsAnimTimer,
			);
		} else {
			TIMER_Set(
				NPC,
				b"attacking\0".as_ptr() as *const core::ffi::c_char,
				(*NPC).client.as_ref().unwrap().ps.legsAnimTimer + Q_irand(0, 1500),
			); //FIXME: base on skill
			TIMER_Set(NPC, b"standing\0".as_ptr() as *const core::ffi::c_char, -level.time);
			TIMER_Set(NPC, b"walking\0".as_ptr() as *const core::ffi::c_char, -level.time);
			TIMER_Set(
				NPC,
				b"running\0".as_ptr() as *const core::ffi::c_char,
				(*NPC).client.as_ref().unwrap().ps.legsAnimTimer + 5000,
			);
		}

		TIMER_Set(
			NPC,
			b"attack_dmg\0".as_ptr() as *const core::ffi::c_char,
			200,
		); // level two damage
	}

	// Need to do delayed damage since the attack animations encapsulate multiple mini-attacks
	match (*NPC).client.as_ref().unwrap().ps.legsAnim {
		BOTH_ATTACK1 | BOTH_MELEE1 => {
			if (*NPC).client.as_ref().unwrap().ps.legsAnimTimer > 650 //more than 13 frames left
				&& PM_AnimLength(
					(*NPC).client.as_ref().unwrap().clientInfo.animFileIndex,
					(*NPC).client.as_ref().unwrap().ps.legsAnim as animNumber_t,
				) - (*NPC).client.as_ref().unwrap().ps.legsAnimTimer
					>= 800
			//at least 16 frames into anim
			{
				Howler_TryDamage(dmg, qfalse, qfalse);
			}
		}
		BOTH_ATTACK2 | BOTH_MELEE2 => {
			if (*NPC).client.as_ref().unwrap().ps.legsAnimTimer > 350 //more than 7 frames left
				&& PM_AnimLength(
					(*NPC).client.as_ref().unwrap().clientInfo.animFileIndex,
					(*NPC).client.as_ref().unwrap().ps.legsAnim as animNumber_t,
				) - (*NPC).client.as_ref().unwrap().ps.legsAnimTimer
					>= 550
			//at least 11 frames into anim
			{
				Howler_TryDamage(dmg, qtrue, qfalse);
			}
		}
		BOTH_GESTURE1 => {
			if (*NPC).client.as_ref().unwrap().ps.legsAnimTimer > 1800 //more than 36 frames left
				&& PM_AnimLength(
					(*NPC).client.as_ref().unwrap().clientInfo.animFileIndex,
					(*NPC).client.as_ref().unwrap().ps.legsAnim as animNumber_t,
				) - (*NPC).client.as_ref().unwrap().ps.legsAnimTimer
					>= 950
			//at least 19 frames into anim
			{
				Howler_Howl();
				if (*NPC).count == 0 {
					G_PlayEffect(
						G_EffectIndex(b"howler/sonic\0".as_ptr() as *const core::ffi::c_char),
						(*NPC).playerModel as *const core::ffi::c_float,
						(*NPC).genericBolt1 as *const core::ffi::c_float,
						(*NPC).genericBolt1,
						(*NPC).s.number,
						4750,
						qtrue,
					);
					G_SoundOnEnt(
						NPC,
						CHAN_VOICE,
						b"sound/chars/howler/howl.mp3\0".as_ptr() as *const core::ffi::c_char,
					);
					(*NPC).count = 1;
				}
			}
		}
		_ => {
			//anims seem to get reset after a load, so just stop attacking and it will restart as needed.
			TIMER_Remove(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char);
		}
	}

	// Just using this to remove the attacking flag at the right time
	TIMER_Done2(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char, qtrue);
}

//----------------------------------
static unsafe fn Howler_Combat() {
	let mut faced: qboolean = qfalse;
	let mut distance: core::ffi::c_float;
	let mut advance: qboolean = qfalse;
	if (*NPC).client.as_ref().unwrap().ps.groundEntityNum == ENTITYNUM_NONE {
		//not on the ground
		if (*NPC).client.as_ref().unwrap().ps.legsAnim == BOTH_JUMP1
			|| (*NPC).client.as_ref().unwrap().ps.legsAnim == BOTH_INAIR1
		{
			//flying through the air with the greatest of ease, etc
			Howler_TryDamage(10, qfalse, qfalse);
		}
	} else {
		//not in air, see if we should attack or advance
		// If we cannot see our target or we have somewhere to go, then do that
		if NPC_ClearLOS((*NPC).enemy) == qfalse {
			//|| UpdateGoal( ))
			NPCInfo.as_mut().unwrap().goalEntity = (*NPC).enemy;
			NPCInfo.as_mut().unwrap().goalRadius = MAX_DISTANCE; // just get us within combat range

			if NPCInfo.as_ref().unwrap().localState == LSTATE_BERZERK {
				NPC_Howler_Move(3);
			} else {
				NPC_Howler_Move(10);
			}
			NPC_UpdateAngles(qfalse, qtrue);
			return;
		}

		distance = DistanceHorizontal((*NPC).currentOrigin.as_ptr(), (*NPC).enemy.as_ref().unwrap().currentOrigin.as_ptr());

		if !(*NPC).enemy.is_null()
			&& !(*NPC).enemy.as_ref().unwrap().client.is_null()
			&& PM_InKnockDown(&mut (*(*NPC).enemy.as_ref().unwrap().client).ps) != 0
		{
			//get really close to knocked down enemies
			advance = qfalse_cond(distance > MIN_DISTANCE);
		} else {
			advance = qfalse_cond(distance > MAX_DISTANCE); //MIN_DISTANCE
		}

		if (advance != 0 || NPCInfo.as_ref().unwrap().localState == LSTATE_WAITING)
			&& TIMER_Done(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char) != 0
		{
			// waiting monsters can't attack
			if TIMER_Done2(NPC, b"takingPain\0".as_ptr() as *const core::ffi::c_char, qtrue) != 0 {
				NPCInfo.as_mut().unwrap().localState = LSTATE_CLEAR;
			} else if TIMER_Done(NPC, b"standing\0".as_ptr() as *const core::ffi::c_char) != 0 {
				faced = Howler_Move(1);
			}
		} else {
			Howler_Attack(distance, qfalse);
		}
	}

	if faced == 0 {
		if TIMER_Done(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char) != 0 {
			// not attacking
			//not standing around
			// Sometimes I have problems with facing the enemy I'm attacking, so force the issue so I don't look dumb
			NPC_FaceEnemy(qtrue);
		} else {
			NPC_UpdateAngles(qfalse, qtrue);
		}
	}
}

/*
-------------------------
NPC_Howler_Pain
-------------------------
*/
pub unsafe extern "C" fn NPC_Howler_Pain(
	self_: *mut gentity_t,
	inflictor: *mut gentity_t,
	other: *mut gentity_t,
	point: *const core::ffi::c_float,
	damage: c_int,
	mod_: c_int,
	hitLoc: c_int,
) {
	if self_.is_null() || (*self_).NPC.is_null() {
		return;
	}

	if (*self_).NPC.as_ref().unwrap().localState != LSTATE_BERZERK {
		//damage >= 10 )
		(*self_).NPC.as_mut().unwrap().stats.aggression += damage;
		(*self_).NPC.as_mut().unwrap().localState = LSTATE_WAITING;

		TIMER_Remove(self_, b"attacking\0".as_ptr() as *const core::ffi::c_char);

		VectorCopy(
			(*self_).NPC.as_ref().unwrap().lastPathAngles.as_ptr(),
			(*self_).s.number as *mut core::ffi::c_float,
		);

		//if ( self->client->ps.legsAnim == BOTH_GESTURE1 )
		{
			G_StopEffect(
				G_EffectIndex(b"howler/sonic\0".as_ptr() as *const core::ffi::c_char),
				(*self_).playerModel as *const core::ffi::c_float,
				(*self_).genericBolt1,
				(*self_).s.number,
			);
		}

		NPC_SetAnim(
			self_,
			SETANIM_BOTH,
			BOTH_PAIN1,
			SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
		);
		TIMER_Set(
			self_,
			b"takingPain\0".as_ptr() as *const core::ffi::c_char,
			(*self_).client.as_ref().unwrap().ps.legsAnimTimer,
		); //2900 );

		if (*self_).health > HOWLER_PANIC_HEALTH {
			//still have some health left
			if Q_irand(0, (*self_).max_health) > (*self_).health {
				//FIXME: or check damage?
				//back off!
				TIMER_Set(self_, b"standing\0".as_ptr() as *const core::ffi::c_char, -level.time);
				TIMER_Set(self_, b"running\0".as_ptr() as *const core::ffi::c_char, -level.time);
				TIMER_Set(self_, b"walking\0".as_ptr() as *const core::ffi::c_char, -level.time);
				TIMER_Set(
					self_,
					b"retreating\0".as_ptr() as *const core::ffi::c_char,
					Q_irand(1000, 5000),
				);
			} else {
				//go after him!
				TIMER_Set(self_, b"standing\0".as_ptr() as *const core::ffi::c_char, -level.time);
				TIMER_Set(
					self_,
					b"running\0".as_ptr() as *const core::ffi::c_char,
					(*self_).client.as_ref().unwrap().ps.legsAnimTimer + Q_irand(3000, 6000),
				);
				TIMER_Set(self_, b"walking\0".as_ptr() as *const core::ffi::c_char, -level.time);
				TIMER_Set(self_, b"retreating\0".as_ptr() as *const core::ffi::c_char, -level.time);
			}
		} else if !(*self_).NPC.is_null() {
			//panic!
			if Q_irand(0, 1) != 0 {
				//berzerk
				(*self_).NPC.as_mut().unwrap().localState = LSTATE_BERZERK;
			} else {
				//flee
				(*self_).NPC.as_mut().unwrap().localState = LSTATE_FLEE;
				TIMER_Set(self_, b"flee\0".as_ptr() as *const core::ffi::c_char, Q_irand(10000, 30000));
			}
		}
	}
}

/*
-------------------------
NPC_BSHowler_Default
-------------------------
*/
pub unsafe extern "C" fn NPC_BSHowler_Default() {
	if (*NPC).client.as_ref().unwrap().ps.legsAnim != BOTH_GESTURE1 {
		(*NPC).count = 0;
	}
	//FIXME: if in jump, do damage in front and maybe knock them down?
	if TIMER_Done(NPC, b"attacking\0".as_ptr() as *const core::ffi::c_char) == 0 {
		if !(*NPC).enemy.is_null() {
			//NPC_FaceEnemy( qfalse );
			Howler_Attack(
				Distance(
					(*NPC).enemy.as_ref().unwrap().currentOrigin.as_ptr(),
					(*NPC).currentOrigin.as_ptr(),
				),
				qfalse,
			);
		} else {
			//NPC_UpdateAngles( qfalse, qtrue );
			Howler_Attack(0.0, qfalse);
		}
		NPC_UpdateAngles(qfalse, qtrue);
		return;
	}

	if !(*NPC).enemy.is_null() {
		if NPCInfo.as_ref().unwrap().stats.aggression > 0 {
			if TIMER_Done(NPC, b"aggressionDecay\0".as_ptr() as *const core::ffi::c_char) != 0 {
				NPCInfo.as_mut().unwrap().stats.aggression -= 1;
				TIMER_Set(NPC, b"aggressionDecay\0".as_ptr() as *const core::ffi::c_char, 500);
			}
		}
		if TIMER_Done(NPC, b"flee\0".as_ptr() as *const core::ffi::c_char) == 0
			&& NPC_BSFlee() != 0
		{
			//this can clear ENEMY
			//successfully trying to run away
			return;
		}
		if (*NPC).enemy.is_null() {
			NPC_UpdateAngles(qfalse, qtrue);
			return;
		}
		if NPCInfo.as_ref().unwrap().localState == LSTATE_FLEE {
			//we were fleeing, now done (either timer ran out or we cannot flee anymore
			if NPC_ClearLOS((*NPC).enemy) != 0 {
				//if enemy is still around, go berzerk
				NPCInfo.as_mut().unwrap().localState = LSTATE_BERZERK;
			} else {
				//otherwise, lick our wounds?
				NPCInfo.as_mut().unwrap().localState = LSTATE_CLEAR;
				TIMER_Set(NPC, b"standing\0".as_ptr() as *const core::ffi::c_char, Q_irand(3000, 10000));
			}
		} else if NPCInfo.as_ref().unwrap().localState == LSTATE_BERZERK {
			//go nuts!
		} else if NPCInfo.as_ref().unwrap().stats.aggression >= Q_irand(75, 125) {
			//that's it, go nuts!
			NPCInfo.as_mut().unwrap().localState = LSTATE_BERZERK;
		} else if TIMER_Done(NPC, b"retreating\0".as_ptr() as *const core::ffi::c_char) == 0 {
			//trying to back off
			NPC_FaceEnemy(qtrue);
			if (*NPC).client.as_ref().unwrap().ps.speed > NPCInfo.as_ref().unwrap().stats.walkSpeed {
				(*NPC).client.as_mut().unwrap().ps.speed = NPCInfo.as_ref().unwrap().stats.walkSpeed;
			}
			ucmd.buttons |= BUTTON_WALKING;
			if Distance(
				(*NPC).enemy.as_ref().unwrap().currentOrigin.as_ptr(),
				(*NPC).currentOrigin.as_ptr(),
			) < HOWLER_RETREAT_DIST
			{
				//enemy is close
				let mut moveDir: [core::ffi::c_float; 3] = [0.0; 3];
				AngleVectors(
					(*NPC).currentAngles.as_ptr(),
					moveDir.as_mut_ptr(),
					core::ptr::null_mut(),
					core::ptr::null_mut(),
				);
				VectorScale(moveDir.as_ptr(), -1.0, moveDir.as_mut_ptr());
				if NAV_DirSafe(NPC, moveDir.as_mut_ptr(), 8.0) == 0 {
					//enemy is backing me up against a wall or ledge!  Start to get really mad!
					NPCInfo.as_mut().unwrap().stats.aggression += 2;
				} else {
					//back off
					ucmd.forwardmove = -127.0;
				}
				//enemy won't leave me alone, get mad...
				NPCInfo.as_mut().unwrap().stats.aggression += 1;
			}
			return;
		} else if TIMER_Done(NPC, b"standing\0".as_ptr() as *const core::ffi::c_char) != 0 {
			//not standing around
			if NPCInfo.as_ref().unwrap().last_ucmd.forwardmove == 0.0
				&& NPCInfo.as_ref().unwrap().last_ucmd.rightmove == 0.0
			{
				//stood last frame
				if TIMER_Done(NPC, b"walking\0".as_ptr() as *const core::ffi::c_char) != 0
					&& TIMER_Done(NPC, b"running\0".as_ptr() as *const core::ffi::c_char) != 0
				{
					//not walking or running
					if Q_irand(0, 2) != 0 {
						//run for a while
						TIMER_Set(
							NPC,
							b"walking\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(4000, 8000),
						);
					} else {
						//walk for a bit
						TIMER_Set(
							NPC,
							b"running\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(2500, 5000),
						);
					}
				}
			} else if (NPCInfo.as_ref().unwrap().last_ucmd.buttons & BUTTON_WALKING) != 0 {
				//walked last frame
				if TIMER_Done(NPC, b"walking\0".as_ptr() as *const core::ffi::c_char) != 0 {
					//just finished walking
					if Q_irand(0, 5) != 0
						|| DistanceSquared(
							(*NPC).enemy.as_ref().unwrap().currentOrigin.as_ptr(),
							(*NPC).currentOrigin.as_ptr(),
						) < MAX_DISTANCE_SQR
					{
						//run for a while
						TIMER_Set(
							NPC,
							b"running\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(4000, 20000),
						);
					} else {
						//stand for a bit
						TIMER_Set(
							NPC,
							b"standing\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(2000, 6000),
						);
					}
				}
			} else {
				//ran last frame
				if TIMER_Done(NPC, b"running\0".as_ptr() as *const core::ffi::c_char) != 0 {
					//just finished running
					if Q_irand(0, 8) != 0
						|| DistanceSquared(
							(*NPC).enemy.as_ref().unwrap().currentOrigin.as_ptr(),
							(*NPC).currentOrigin.as_ptr(),
						) < MAX_DISTANCE_SQR
					{
						//walk for a while
						TIMER_Set(
							NPC,
							b"walking\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(3000, 10000),
						);
					} else {
						//stand for a bit
						TIMER_Set(
							NPC,
							b"standing\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(2000, 6000),
						);
					}
				}
			}
		}
		if NPC_ValidEnemy((*NPC).enemy) == qfalse {
			TIMER_Remove(NPC, b"lookForNewEnemy\0".as_ptr() as *const core::ffi::c_char); //make them look again right now
			if !(*NPC).enemy.as_ref().unwrap().inuse != 0
				|| level.time - (*NPC).enemy.as_ref().unwrap().s.time > Q_irand(10000, 15000)
			{
				//it's been a while since the enemy died, or enemy is completely gone, get bored with him
				(*NPC).enemy = core::ptr::null_mut();
				Howler_Patrol();
				NPC_UpdateAngles(qtrue, qtrue);
				return;
			}
		}
		if TIMER_Done(NPC, b"lookForNewEnemy\0".as_ptr() as *const core::ffi::c_char) != 0 {
			let sav_enemy: *mut gentity_t = (*NPC).enemy; //FIXME: what about NPC->lastEnemy?
			(*NPC).enemy = core::ptr::null_mut();
			let newEnemy: *mut gentity_t = NPC_CheckEnemy(
				NPCInfo.as_ref().unwrap().confusionTime < level.time,
				qfalse,
				qfalse,
			);
			(*NPC).enemy = sav_enemy;
			if !newEnemy.is_null() && newEnemy != sav_enemy {
				//picked up a new enemy!
				(*NPC).lastEnemy = (*NPC).enemy;
				G_SetEnemy(NPC, newEnemy);
				if (*NPC).enemy != (*NPC).lastEnemy {
					//clear this so that we only sniff the player the first time we pick them up
					(*NPC).useDebounceTime = 0;
				}
				//hold this one for at least 5-15 seconds
				TIMER_Set(
					NPC,
					b"lookForNewEnemy\0".as_ptr() as *const core::ffi::c_char,
					Q_irand(5000, 15000),
				);
			} else {
				//look again in 2-5 secs
				TIMER_Set(
					NPC,
					b"lookForNewEnemy\0".as_ptr() as *const core::ffi::c_char,
					Q_irand(2000, 5000),
				);
			}
		}
		Howler_Combat();
		if TIMER_Done(NPC, b"speaking\0".as_ptr() as *const core::ffi::c_char) != 0 {
			if TIMER_Done(NPC, b"standing\0".as_ptr() as *const core::ffi::c_char) == 0
				|| TIMER_Done(NPC, b"retreating\0".as_ptr() as *const core::ffi::c_char) == 0
			{
				G_SoundOnEnt(
					NPC,
					CHAN_VOICE,
					va(
						b"sound/chars/howler/idle_hiss%d.mp3\0".as_ptr() as *const core::ffi::c_char,
						Q_irand(1, 2),
					),
				);
			} else if TIMER_Done(NPC, b"walking\0".as_ptr() as *const core::ffi::c_char) == 0
				|| NPCInfo.as_ref().unwrap().localState == LSTATE_FLEE
			{
				G_SoundOnEnt(
					NPC,
					CHAN_VOICE,
					va(
						b"sound/chars/howler/howl_talk%d.mp3\0".as_ptr() as *const core::ffi::c_char,
						Q_irand(1, 5),
					),
				);
			} else {
				G_SoundOnEnt(
					NPC,
					CHAN_VOICE,
					va(
						b"sound/chars/howler/howl_yell%d.mp3\0".as_ptr() as *const core::ffi::c_char,
						Q_irand(1, 5),
					),
				);
			}
			if NPCInfo.as_ref().unwrap().localState == LSTATE_BERZERK
				|| NPCInfo.as_ref().unwrap().localState == LSTATE_FLEE
			{
				TIMER_Set(
					NPC,
					b"speaking\0".as_ptr() as *const core::ffi::c_char,
					Q_irand(1000, 4000),
				);
			} else {
				TIMER_Set(
					NPC,
					b"speaking\0".as_ptr() as *const core::ffi::c_char,
					Q_irand(3000, 8000),
				);
			}
		}
		return;
	} else {
		if TIMER_Done(NPC, b"speaking\0".as_ptr() as *const core::ffi::c_char) != 0 {
			if Q_irand(0, 3) == 0 {
				G_SoundOnEnt(
					NPC,
					CHAN_VOICE,
					va(
						b"sound/chars/howler/idle_hiss%d.mp3\0".as_ptr() as *const core::ffi::c_char,
						Q_irand(1, 2),
					),
				);
			} else {
				G_SoundOnEnt(
					NPC,
					CHAN_VOICE,
					va(
						b"sound/chars/howler/howl_talk%d.mp3\0".as_ptr() as *const core::ffi::c_char,
						Q_irand(1, 5),
					),
				);
			}
			TIMER_Set(
				NPC,
				b"speaking\0".as_ptr() as *const core::ffi::c_char,
				Q_irand(4000, 12000),
			);
		}
		if NPCInfo.as_ref().unwrap().stats.aggression > 0 {
			if TIMER_Done(NPC, b"aggressionDecay\0".as_ptr() as *const core::ffi::c_char) != 0 {
				NPCInfo.as_mut().unwrap().stats.aggression -= 1;
				TIMER_Set(NPC, b"aggressionDecay\0".as_ptr() as *const core::ffi::c_char, 200);
			}
		}
		if TIMER_Done(NPC, b"standing\0".as_ptr() as *const core::ffi::c_char) != 0 {
			//not standing around
			if NPCInfo.as_ref().unwrap().last_ucmd.forwardmove == 0.0
				&& NPCInfo.as_ref().unwrap().last_ucmd.rightmove == 0.0
			{
				//stood last frame
				if TIMER_Done(NPC, b"walking\0".as_ptr() as *const core::ffi::c_char) != 0
					&& TIMER_Done(NPC, b"running\0".as_ptr() as *const core::ffi::c_char) != 0
				{
					//not walking or running
					if !NPCInfo.as_ref().unwrap().goalEntity.is_null() {
						//have somewhere to go
						if Q_irand(0, 2) != 0 {
							//walk for a while
							TIMER_Set(
								NPC,
								b"walking\0".as_ptr() as *const core::ffi::c_char,
								Q_irand(3000, 10000),
							);
						} else {
							//run for a bit
							TIMER_Set(
								NPC,
								b"running\0".as_ptr() as *const core::ffi::c_char,
								Q_irand(2500, 5000),
							);
						}
					}
				}
			} else if (NPCInfo.as_ref().unwrap().last_ucmd.buttons & BUTTON_WALKING) != 0 {
				//walked last frame
				if TIMER_Done(NPC, b"walking\0".as_ptr() as *const core::ffi::c_char) != 0 {
					//just finished walking
					if Q_irand(0, 3) != 0 {
						//run for a while
						TIMER_Set(
							NPC,
							b"running\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(3000, 6000),
						);
					} else {
						//stand for a bit
						TIMER_Set(
							NPC,
							b"standing\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(2500, 5000),
						);
					}
				}
			} else {
				//ran last frame
				if TIMER_Done(NPC, b"running\0".as_ptr() as *const core::ffi::c_char) != 0 {
					//just finished running
					if Q_irand(0, 2) != 0 {
						//walk for a while
						TIMER_Set(
							NPC,
							b"walking\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(6000, 15000),
						);
					} else {
						//stand for a bit
						TIMER_Set(
							NPC,
							b"standing\0".as_ptr() as *const core::ffi::c_char,
							Q_irand(4000, 6000),
						);
					}
				}
			}
		}
		if (NPCInfo.as_ref().unwrap().scriptFlags & SCF_LOOK_FOR_ENEMIES) != 0 {
			Howler_Patrol();
		} else {
			Howler_Idle();
		}
	}

	NPC_UpdateAngles(qfalse, qtrue);
}
