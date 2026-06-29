// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_float, c_void};

// //#define AMMO_POD_HEALTH				40
const AMMO_POD_HEALTH: c_int = 1;
const TURN_OFF: c_int = 0x00000100;

const VELOCITY_DECAY: c_float = 0.25;
const MAX_DISTANCE: c_int = 256;
const MAX_DISTANCE_SQR: c_int = MAX_DISTANCE * MAX_DISTANCE;
const MIN_DISTANCE: c_int = 24;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

// Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_DROPPINGDOWN: c_int = 1;
const LSTATE_DOWN: c_int = 2;
const LSTATE_RISINGUP: c_int = 3;

extern "C" {
    fn FindItemForAmmo(ammo: c_int) -> *mut c_void;

    fn G_SoundIndex(name: *const c_char) -> c_int;
    fn G_EffectIndex(name: *const c_char) -> c_int;
    fn RegisterItem(item: *mut c_void) -> ();
    fn FindItemForWeapon(weapon: c_int) -> *mut c_void;
    fn G_PlayEffect(name: *const c_char, origin: *const c_float, dir: *const c_float) -> ();
    fn G_Sound(ent: *mut c_void, sound: c_int) -> ();
    fn NPC_Pain(
        self_: *mut c_void,
        inflictor: *mut c_void,
        other: *mut c_void,
        point: *const c_float,
        damage: c_int,
        mod_: c_int,
    ) -> ();
    fn CreateMissile(
        org: *const c_float,
        dir: *const c_float,
        vel: c_float,
        life: c_int,
        owner: *mut gentity_t,
        altFire: c_int,
    ) -> *mut gentity_t;
    fn G_Damage(
        target: *mut c_void,
        inflictor: *mut c_void,
        attacker: *mut c_void,
        dir: *const c_float,
        point: *const c_float,
        damage: c_int,
        dflags: c_int,
        mod_: c_int,
    ) -> ();
    fn NPC_FaceEnemy(doPitch: c_int) -> c_int;
    fn NPC_MoveToGoal(clearGoal: c_int) -> ();
    fn TIMER_Done(ent: *mut c_void, label: *const c_char) -> c_int;
    fn TIMER_Set(ent: *mut c_void, label: *const c_char, duration: c_int) -> ();
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn DistanceHorizontalSquared(p1: *const c_float, p2: *const c_float) -> c_float;
    fn NPC_ClearLOS(ent: *mut c_void) -> c_int;
    fn CalcEntitySpot(ent: *mut c_void, spot: c_int, point: *mut c_float) -> ();
    fn VectorSubtract(veca: *const c_float, vecb: *const c_float, out: *mut c_float) -> ();
    fn vectoangles(value1: *const c_float, angles: *mut c_float) -> ();
    fn AngleVectors(angles: *const c_float, forward: *mut c_float, right: *mut c_float, up: *mut c_float) -> ();
    fn NPC_SetAnim(ent: *mut c_void, setanim_level: c_int, anim: c_int, flags: c_int) -> ();
    fn UpdateGoal() -> c_int;
    fn NPC_UpdateAngles(doPitch: c_int, doYaw: c_int) -> ();
    fn NPC_BSIdle() -> ();
    fn NPC_CheckPlayerTeamStealth() -> c_int;
}

// Opaque struct types from headers - declarations needed for member access
#[repr(C)]
pub struct gentity_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct mdxaBone_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct gitem_t {
    _opaque: [u8; 0],
}

// Global references
extern "C" {
    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut c_void;
    static mut ucmd: c_void;
    static cg: c_void;
    static level: c_void;
}

/*
-------------------------
NPC_Mark2_Precache
-------------------------
*/
pub unsafe extern "C" fn NPC_Mark2_Precache() {
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_explo\0".as_ptr() as *const c_char); // blows up on death
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_pain\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_fire\0".as_ptr() as *const c_char);
    G_SoundIndex(b"sound/chars/mark2/misc/mark2_move_lp\0".as_ptr() as *const c_char);

    G_EffectIndex(b"explosions/droidexplosion1\0".as_ptr() as *const c_char);
    G_EffectIndex(b"env/med_explode2\0".as_ptr() as *const c_char);
    G_EffectIndex(b"blaster/smoke_bolton\0".as_ptr() as *const c_char);
    G_EffectIndex(b"bryar/muzzle_flash\0".as_ptr() as *const c_char);

    RegisterItem(FindItemForWeapon(4 as c_int)); // WP_BRYAR_PISTOL
    RegisterItem(FindItemForAmmo(10 as c_int)); // AMMO_METAL_BOLTS
    RegisterItem(FindItemForAmmo(3 as c_int));  // AMMO_POWERCELL
    RegisterItem(FindItemForAmmo(2 as c_int));  // AMMO_BLASTER
}

/*
-------------------------
NPC_Mark2_Part_Explode
-------------------------
*/
pub unsafe extern "C" fn NPC_Mark2_Part_Explode(self_: *mut gentity_t, bolt: c_int) {
    if bolt >= 0 {
        // mdxaBone_t	boltMatrix;
        // vec3_t		org, dir;

        // gi.G2API_GetBoltMatrix( self->ghoul2, self->playerModel,
        // 					bolt,
        // 					&boltMatrix, self->currentAngles, self->currentOrigin, (cg.time?cg.time:level.time),
        // 					NULL, self->s.modelScale );

        // gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, org );
        // gi.G2API_GiveMeVectorFromMatrix( boltMatrix, NEGATIVE_Y, dir );

        // G_PlayEffect( "env/med_explode2", org, dir );
        // G_PlayEffect( G_EffectIndex("blaster/smoke_bolton"), self->playerModel, bolt, self->s.number, org);

        // self->count++;	// Count of pods blown off
    }
}

/*
-------------------------
NPC_Mark2_Pain
- look at what was hit and see if it should be removed from the model.
-------------------------
*/
pub unsafe extern "C" fn NPC_Mark2_Pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    other: *mut gentity_t,
    point: *const c_float,
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    // int newBolt,i;

    NPC_Pain(self_ as *mut c_void, inflictor as *mut c_void, other as *mut c_void, point, damage, mod_);

    // for (i=0;i<3;i++)
    // {
    // 	if ((hitLoc==HL_GENERIC1+i) && (self->locationDamage[HL_GENERIC1+i] > AMMO_POD_HEALTH))	// Blow it up?
    // 	{
    // 		if (self->locationDamage[hitLoc] >= AMMO_POD_HEALTH)
    // 		{
    // 			newBolt = gi.G2API_AddBolt( &self->ghoul2[self->playerModel], va("torso_canister%d",(i+1)) );
    // 			if ( newBolt != -1 )
    // 			{
    // 				NPC_Mark2_Part_Explode(self,newBolt);
    // 			}
    // 			gi.G2API_SetSurfaceOnOff( &self->ghoul2[self->playerModel], va("torso_canister%d",(i+1)), TURN_OFF );
    // 			break;
    // 		}
    // 	}
    // }

    // G_Sound( self, G_SoundIndex( "sound/chars/mark2/misc/mark2_pain" ));

    // // If any pods were blown off, kill him
    // if (self->count > 0)
    // {
    // 	G_Damage( self, NULL, NULL, NULL, NULL, self->health, DAMAGE_NO_PROTECTION, MOD_UNKNOWN );
    // }
}

/*
-------------------------
Mark2_Hunt
-------------------------
*/
pub unsafe extern "C" fn Mark2_Hunt() {
    // if ( NPCInfo->goalEntity == NULL )
    // {
    // 	NPCInfo->goalEntity = NPC->enemy;
    // }

    // // Turn toward him before moving towards him.
    NPC_FaceEnemy(1 as c_int);

    // NPCInfo->combatMove = qtrue;
    // NPC_MoveToGoal( qtrue );
}

/*
-------------------------
Mark2_FireBlaster
-------------------------
*/
pub unsafe extern "C" fn Mark2_FireBlaster(advance: c_int) {
    // vec3_t	muzzle1,enemy_org1,delta1,angleToEnemy1;
    let mut muzzle1: [c_float; 3] = [0.0; 3];
    let mut enemy_org1: [c_float; 3] = [0.0; 3];
    let mut delta1: [c_float; 3] = [0.0; 3];
    let mut angleToEnemy1: [c_float; 3] = [0.0; 3];

    // static	vec3_t	forward, vright, up;
    // static	vec3_t	muzzle;
    let mut forward: [c_float; 3] = [0.0; 3];
    let mut vright: [c_float; 3] = [0.0; 3];
    let mut up: [c_float; 3] = [0.0; 3];
    let mut muzzle: [c_float; 3] = [0.0; 3];

    // gentity_t	*missile;
    let missile: *mut gentity_t;
    // mdxaBone_t	boltMatrix;
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();

    // gi.G2API_GetBoltMatrix( NPC->ghoul2, NPC->playerModel,
    // 				NPC->genericBolt1,
    // 				&boltMatrix, NPC->currentAngles, NPC->currentOrigin, (cg.time?cg.time:level.time),
    // 				NULL, NPC->s.modelScale );

    // gi.G2API_GiveMeVectorFromMatrix( boltMatrix, ORIGIN, muzzle1 );

    // if (NPC->health)
    // {
    // 	CalcEntitySpot( NPC->enemy, SPOT_HEAD, enemy_org1 );
    // 	VectorSubtract (enemy_org1, muzzle1, delta1);
    // 	vectoangles ( delta1, angleToEnemy1 );
    // 	AngleVectors (angleToEnemy1, forward, vright, up);
    // }
    // else
    // {
    // 	AngleVectors (NPC->currentAngles, forward, vright, up);
    // }

    // G_PlayEffect( "bryar/muzzle_flash", muzzle1, forward );

    G_Sound(NPC as *mut c_void, G_SoundIndex(b"sound/chars/mark2/misc/mark2_fire\0".as_ptr() as *const c_char));

    missile = CreateMissile(muzzle1.as_ptr(), forward.as_ptr(), 1600.0, 10000, NPC, 0);

    // missile->classname = "bryar_proj";
    // missile->s.weapon = WP_BRYAR_PISTOL;

    // missile->damage = 1;
    // missile->dflags = DAMAGE_DEATH_KNOCKBACK;
    // missile->methodOfDeath = MOD_ENERGY;
    // missile->clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/*
-------------------------
Mark2_BlasterAttack
-------------------------
*/
pub unsafe extern "C" fn Mark2_BlasterAttack(advance: c_int) {
    if TIMER_Done(NPC as *mut c_void, b"attackDelay\0".as_ptr() as *const c_char) != 0 {	// Attack?
        // if (NPCInfo->localState == LSTATE_NONE)	// He's up so shoot less often.
        // {
        // 	TIMER_Set( NPC, "attackDelay", Q_irand( 500, 2000) );
        // }
        // else
        // {
        // 	TIMER_Set( NPC, "attackDelay", Q_irand( 100, 500) );
        // }
        Mark2_FireBlaster(advance);
        return;
    } else if advance != 0 {
        Mark2_Hunt();
    }
}

/*
-------------------------
Mark2_AttackDecision
-------------------------
*/
pub unsafe extern "C" fn Mark2_AttackDecision() {
    NPC_FaceEnemy(1 as c_int);

    // float		distance	= (int) DistanceHorizontalSquared( NPC->currentOrigin, NPC->enemy->currentOrigin );
    // qboolean	visible		= NPC_ClearLOS( NPC->enemy );
    // qboolean	advance		= (qboolean)(distance > MIN_DISTANCE_SQR);

    // // He's been ordered to get up
    // if (NPCInfo->localState == LSTATE_RISINGUP)
    // {
    // 	NPC->flags &= ~FL_SHIELDED;
    // 	NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_RUN1START, SETANIM_FLAG_HOLD|SETANIM_FLAG_OVERRIDE );
    // 	if ((NPC->client->ps.legsAnimTimer==0) &&
    // 		NPC->client->ps.torsoAnim == BOTH_RUN1START )
    // 	{
    // 		NPCInfo->localState = LSTATE_NONE;	// He's up again.
    // 	}
    // 	return;
    // }

    // // If we cannot see our target, move to see it
    // if ((!visible) || (!NPC_FaceEnemy(qtrue)))
    // {
    // 	// If he's going down or is down, make him get up
    // 	if ((NPCInfo->localState == LSTATE_DOWN) || (NPCInfo->localState == LSTATE_DROPPINGDOWN))
    // 	{
    // 		if ( TIMER_Done( NPC, "downTime" ) )	// Down being down?? (The delay is so he doesn't pop up and down when the player goes in and out of range)
    // 		{
    // 			NPCInfo->localState = LSTATE_RISINGUP;
    // 			NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_RUN1STOP, SETANIM_FLAG_HOLD|SETANIM_FLAG_OVERRIDE );
    // 			TIMER_Set( NPC, "runTime", Q_irand( 3000, 8000) );	// So he runs for a while before testing to see if he should drop down.
    // 		}
    // 	}
    // 	else
    // 	{
    // 		Mark2_Hunt();
    // 	}
    // 	return;
    // }

    // // He's down but he could advance if he wants to.
    // if ((advance) && (TIMER_Done( NPC, "downTime" )) && (NPCInfo->localState == LSTATE_DOWN))
    // {
    // 	NPCInfo->localState = LSTATE_RISINGUP;
    // 	NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_RUN1STOP, SETANIM_FLAG_HOLD|SETANIM_FLAG_OVERRIDE );
    // 	TIMER_Set( NPC, "runTime", Q_irand( 3000, 8000) );	// So he runs for a while before testing to see if he should drop down.
    // }

    // NPC_FaceEnemy( qtrue );

    // // Dropping down to shoot
    // if (NPCInfo->localState == LSTATE_DROPPINGDOWN)
    // {
    // 	NPC_SetAnim( NPC, SETANIM_BOTH, BOTH_RUN1STOP, SETANIM_FLAG_HOLD|SETANIM_FLAG_OVERRIDE );
    // 	TIMER_Set( NPC, "downTime", Q_irand( 3000, 9000) );

    // 	if ((NPC->client->ps.legsAnimTimer==0) && NPC->client->ps.torsoAnim == BOTH_RUN1STOP )
    // 	{
    // 		NPC->flags |= FL_SHIELDED;
    // 		NPCInfo->localState = LSTATE_DOWN;
    // 	}
    // }
    // // He's down and shooting
    // else if (NPCInfo->localState == LSTATE_DOWN)
    // {
    // //		NPC->flags |= FL_SHIELDED;//only damagable by lightsabers and missiles

    // 	Mark2_BlasterAttack(qfalse);
    // }
    // else if (TIMER_Done( NPC, "runTime" ))	// Lowering down to attack. But only if he's done running at you.
    // {
    // 	NPCInfo->localState = LSTATE_DROPPINGDOWN;
    // }
    // else if (advance)
    // {
    // 	// We can see enemy so shoot him if timer lets you.
    // 	Mark2_BlasterAttack(advance);
    // }
}

/*
-------------------------
Mark2_Patrol
-------------------------
*/
pub unsafe extern "C" fn Mark2_Patrol() {
    if NPC_CheckPlayerTeamStealth() != 0 {
        // //		G_Sound( NPC, G_SoundIndex("sound/chars/mark1/misc/anger.wav"));
        NPC_UpdateAngles(1 as c_int, 1 as c_int);
        return;
    }

    // //If we have somewhere to go, then do that
    // if (!NPC->enemy)
    // {
    // 	if ( UpdateGoal() )
    // 	{
    // 		ucmd.buttons |= BUTTON_WALKING;
    // 		NPC_MoveToGoal( qtrue );
    // 		NPC_UpdateAngles( qtrue, qtrue );
    // 	}

    // 	//randomly talk
    // 	if (TIMER_Done(NPC,"patrolNoise"))
    // 	{
    // //			G_Sound( NPC, G_SoundIndex(va("sound/chars/mark1/misc/talk%d.wav",	Q_irand(1, 4))));

    // 		TIMER_Set( NPC, "patrolNoise", Q_irand( 2000, 4000 ) );
    // 	}
    // }
}

/*
-------------------------
Mark2_Idle
-------------------------
*/
pub unsafe extern "C" fn Mark2_Idle() {
    NPC_BSIdle();
}

/*
-------------------------
NPC_BSMark2_Default
-------------------------
*/
pub unsafe extern "C" fn NPC_BSMark2_Default() {
    // if ( NPC->enemy )
    // {
    // 	NPCInfo->goalEntity = NPC->enemy;
    // 	Mark2_AttackDecision();
    // }
    // else if ( NPCInfo->scriptFlags & SCF_LOOK_FOR_ENEMIES )
    // {
    // 	Mark2_Patrol();
    // }
    // else
    // {
    // 	Mark2_Idle();
    // }
}
