//NPC_reactions.cpp

// leave this line at the top for all NPC_xxxx.cpp files...
#![allow(non_snake_case, non_upper_case_globals, unused_imports, dead_code)]

use core::ffi::{c_int, c_char, c_void};

// Stub includes - these would be full modules in a complete port
// For now we declare types minimally to maintain structural validity

// Forward declarations and stubs for external types/functions
// (These would be properly defined in their respective modules)

extern "C" {
    static mut g_spskill: *mut c_void;  // cvar_t *
    static mut teamLastEnemyTime: [c_int; 32];  // Estimated team count
    static mut stop_icarus: c_int;  // qboolean
    static mut killPlayerTimer: c_int;

    static mut player: *mut c_void;  // gentity_t *
    static mut NPC: *mut c_void;      // gentity_t *
    static mut NPCInfo: *mut c_void;  // gNPC_t *
    static mut level: c_void;

    fn SaveNPCGlobals();
    fn SetNPCGlobals(self_: *mut c_void);
    fn RestoreNPCGlobals();

    fn G_CheckForStrongAttackMomentum(self_: *mut c_void) -> c_int;
    fn G_AddVoiceEvent(self_: *mut c_void, event: c_int, speakDebounceTime: c_int);
    fn PM_AnimLength(index: c_int, anim: c_int) -> c_int;
    fn cgi_S_StartSound(origin: *const [f32; 3], entityNum: c_int, entchannel: c_int, sfx: c_int);
    fn Q3_TaskIDPending(ent: *mut c_void, taskType: c_int) -> c_int;
    fn PM_PickAnim(self_: *mut c_void, minAnim: c_int, maxAnim: c_int) -> c_int;
    fn NPC_CheckLookTarget(self_: *mut c_void) -> c_int;
    fn NPC_SetLookTarget(self_: *mut c_void, entNum: c_int, clearTime: c_int);
    fn Jedi_WaitingAmbush(self_: *mut c_void) -> c_int;
    fn Jedi_Ambush(self_: *mut c_void);
    fn G_EntIsBreakable(entityNum: c_int, breaker: *mut c_void) -> c_int;

    fn PM_SaberInSpecialAttack(anim: c_int) -> c_int;
    fn PM_SpinningSaberAnim(anim: c_int) -> c_int;
    fn PM_SpinningAnim(anim: c_int) -> c_int;
    fn PM_InKnockDown(ps: *mut c_void) -> c_int;
    fn PM_CrouchAnim(anim: c_int) -> c_int;
    fn PM_FlippingAnim(anim: c_int) -> c_int;
    fn PM_RollingAnim(anim: c_int) -> c_int;
    fn PM_InCartwheel(anim: c_int) -> c_int;

    fn G_SetEnemy(self_: *mut c_void, enemy: *mut c_void);
    fn G_ClearEnemy(self_: *mut c_void);
    fn G_AddEvent(self_: *mut c_void, event: c_int, eventParm: c_int);
    fn G_PickPainAnim(self_: *mut c_void, point: *const [f32; 3], damage: c_int, hitLoc: c_int) -> c_int;
    fn NPC_SetAnim(self_: *mut c_void, parts: c_int, anim: c_int, flags: c_int);
    fn G_ValidEnemy(self_: *mut c_void, enemy: *mut c_void) -> c_int;
    fn Boba_Pain(self_: *mut c_void, inflictor: *mut c_void, damage: c_int, mod_: c_int);
    fn G_UseTargets2(self_: *mut c_void, other: *mut c_void, target: *const c_char);
    fn G_ActivateBehavior(self_: *mut c_void, bset: c_int) -> c_int;
    fn INV_SecurityKeyGive(target: *mut c_void, keyname: *const c_char) -> c_int;
    fn INV_GoodieKeyGive(target: *mut c_void) -> c_int;
    fn FindItemForInventory(id: c_int) -> *mut c_void;
    fn G_Sound(ent: *mut c_void, index: c_int);
    fn G_SoundIndex(name: *const c_char) -> c_int;
    fn G_SoundOnEnt(ent: *mut c_void, channel: c_int, name: *const c_char);
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn Q_stricmp(s0: *const c_char, s1: *const c_char) -> c_int;
    fn random() -> f32;

    // For va() macro-like function
    fn va(fmt: *const c_char, ...) -> *const c_char;

    // Timer macros as functions
    fn TIMER_Done(ent: *mut c_void, label: *const c_char) -> c_int;
    fn TIMER_Set(ent: *mut c_void, label: *const c_char, duration: c_int);
}

// Global variables
pub static mut g_crosshairEntDist: f32 = f32::INFINITY;  // Q3_INFINITE
pub static mut g_crosshairSameEntTime: c_int = 0;
pub static mut g_crosshairEntNum: c_int = -1;  // ENTITYNUM_NONE
pub static mut g_crosshairEntTime: c_int = 0;

// Constants
const MIN_PAIN_TIME: c_int = 200;
const Q3_INFINITE: f32 = f32::INFINITY;
const ENTITYNUM_NONE: c_int = -1;
const MOD_SABER: c_int = 1;
const MOD_ELECTROCUTE: c_int = 26;
const MOD_MELEE: c_int = 18;
const MOD_GAS: c_int = 25;
const MOD_CRUSH: c_int = 27;
const PW_GALAK_SHIELD: c_int = 18;
const PW_SHOCKED: c_int = 19;
const SVF_LOCKEDENEMY: c_int = 32;
const SVF_IGNORE_ENEMIES: c_int = 64;
const SVF_ICARUS_FREEZE: c_int = 128;
const SVF_NO_COMBAT_SOUNDS: c_int = 256;
const SVF_NONNPC_ENEMY: c_int = 512;
const FL_NOTARGET: c_int = 0x00000001;
const NPCAI_DIE_ON_IMPACT: c_int = 0x00000001;
const NPCAI_TOUCHED_GOAL: c_int = 0x00000002;
const NPCAI_KNEEL: c_int = 0x00000004;
const WP_SABER: c_int = 1;
const WP_THERMAL: c_int = 15;
const CLASS_VEHICLE: c_int = 0;
const CLASS_GALAKMECH: c_int = 1;
const CLASS_PROTOCOL: c_int = 2;
const CLASS_DESANN: c_int = 3;
const CLASS_JAN: c_int = 4;
const CLASS_LANDO: c_int = 5;
const CLASS_LUKE: c_int = 6;
const CLASS_JEDI: c_int = 7;
const CLASS_KYLE: c_int = 8;
const CLASS_PRISONER: c_int = 9;
const CLASS_REBEL: c_int = 10;
const CLASS_BESPIN_COP: c_int = 11;
const CLASS_R2D2: c_int = 12;
const CLASS_R5D2: c_int = 13;
const CLASS_MOUSE: c_int = 14;
const CLASS_GONK: c_int = 15;
const CLASS_JAWA: c_int = 16;
const CLASS_RANCOR: c_int = 17;
const CLASS_BOBAFETT: c_int = 18;
const TEAM_FREE: c_int = 0;
const TEAM_PLAYER: c_int = 1;
const TEAM_NEUTRAL: c_int = 3;
const TEAM_DISGUISE: c_int = 4;
const TEAM_BORG: c_int = 5;
const SS_FAST: c_int = 1;
const LS_READY: c_int = 0;
const EF_FORCE_GRIPPED: c_int = 0x00000001;
const EF_FORCE_DRAINED: c_int = 0x00000002;
const EF_FORCE_VISIBLE: c_int = 0x00000004;
const EV_PAIN: c_int = 1;
const EV_CHOKE1: c_int = 2;
const EV_CHOKE3: c_int = 4;
const EV_CHASE1: c_int = 5;
const EV_CHASE3: c_int = 7;
const EV_OUTFLANK1: c_int = 8;
const EV_OUTFLANK2: c_int = 9;
const EV_COVER1: c_int = 10;
const EV_COVER5: c_int = 14;
const EV_SUSPICIOUS4: c_int = 15;
const EV_SOUND1: c_int = 16;
const EV_SOUND3: c_int = 18;
const EV_CONFUSE1: c_int = 19;
const EV_CONFUSE3: c_int = 21;
const EV_SIGHT2: c_int = 22;
const EV_SIGHT1: c_int = 23;
const EV_SIGHT3: c_int = 24;
const EV_GIVEUP3: c_int = 25;
const EV_GIVEUP4: c_int = 26;
const EV_JDETECTED1: c_int = 27;
const EV_JDETECTED2: c_int = 28;
const EV_DETECTED1: c_int = 29;
const EV_DETECTED5: c_int = 33;
const EV_LOST1: c_int = 34;
const EV_ESCAPING2: c_int = 35;
const EV_FFWARN: c_int = 36;
const EV_FFTURN: c_int = 37;
const EV_ITEM_PICKUP: c_int = 38;
const EV_RESPOND1: c_int = 39;
const EV_RESPOND3: c_int = 41;
const EV_BUSY1: c_int = 42;
const EV_BUSY3: c_int = 44;
const HL_GENERIC1: c_int = 0;
const SETANIM_BOTH: c_int = 0;
const SETANIM_LEGS: c_int = 1;
const SETANIM_FLAG_OVERRIDE: c_int = 0x00000001;
const SETANIM_FLAG_HOLD: c_int = 0x00000002;
const CHAN_VOICE: c_int = 0;
const BSET_PAIN: c_int = 0;
const BSET_FLEE: c_int = 1;
const BSET_USE: c_int = 2;
const BSET_FFIRE: c_int = 3;
const SCF_NO_COMBAT_TALK: c_int = 0x00000001;
const SCF_DONT_FIRE: c_int = 0x00000002;
const SCF_CROUCHED: c_int = 0x00000004;
const SCF_WALKING: c_int = 0x00000008;
const SCF_NO_MIND_TRICK: c_int = 0x00000010;
const SCF_FORCED_MARCH: c_int = 0x00000020;
const SCF_CHASE_ENEMIES: c_int = 0x00000040;
const SCF_LOOK_FOR_ENEMIES: c_int = 0x00000080;
const SCF_NO_RESPONSE: c_int = 0x00000100;
const BS_DEFAULT: c_int = 0;
const BS_HUNT_AND_KILL: c_int = 1;
const BS_MEDIC_HIDE: c_int = 2;
const RANK_CAPTAIN: c_int = 10;
const TID_CHAN_VOICE: c_int = 1;
const INV_GOODIE_KEY: c_int = 0;
const INV_SECURITY_KEY: c_int = 1;
const PM_DEAD: c_int = 1;
const MAX_CLIENTS: c_int = 2;
const useF_emplaced_gun_use: c_int = 1;
const useF_eweb_use: c_int = 2;
const MAX_BATTERIES: c_int = 5;

/*
-------------------------
NPC_CheckAttacker
-------------------------
*/

unsafe fn NPC_CheckAttacker(mut other: *mut c_void, mod_: c_int)
{
	//FIXME: I don't see anything in here that would stop teammates from taking a teammate
	//			as an enemy.  Ideally, there would be code before this to prevent that from
	//			happening, but that is presumptuous.

	//valid ent - FIXME: a VALIDENT macro would be nice here
	if other.is_null()
	{
		return;
	}

	if other == NPC
	{
		return;
	}

	// !other->inuse - cannot check here due to offset unknown

	//Don't take a target that doesn't want to be
	// if ( other->flags & FL_NOTARGET )
	//	return;

	// if ( NPC->svFlags & SVF_LOCKEDENEMY )
	// {//IF LOCKED, CANNOT CHANGE ENEMY!!!!!
	//	return;
	// }

	//If we haven't taken a target, just get mad
	// if ( NPC->enemy == NULL )//was using "other", fixed to NPC
	// {
	//	G_SetEnemy( NPC, other );
	//	return;
	// }

	//we have an enemy, see if he's dead
	// if ( NPC->enemy->health <= 0 )
	// {
	//	G_ClearEnemy( NPC );
	//	G_SetEnemy( NPC, other );
	//	return;
	// }

	//Don't take the same enemy again
	// if ( other == NPC->enemy )
	//	return;

	// if ( NPC->client->ps.weapon == WP_SABER )
	// {//I'm a jedi
	//	if ( mod == MOD_SABER )
	//	{//I was hit by a saber  FIXME: what if this was a thrown saber?
	//		//always switch to this enemy if I'm a jedi and hit by another saber
	//		G_ClearEnemy( NPC );
	//		G_SetEnemy( NPC, other );
	//		return;
	//	}
	// }
	//Special case player interactions
	// if ( other == &g_entities[0] )
	// {
	//	//Account for the skill level to skew the results
	//	float	luckThreshold;

	//	switch ( g_spskill->integer )
	//	{
	//	//Easiest difficulty, mild chance of picking up the player
	//	case 0:
	//		luckThreshold = 0.9f;
	//		break;

	//	//Medium difficulty, half-half chance of picking up the player
	//	case 1:
	//		luckThreshold = 0.5f;
	//		break;

	//	//Hardest difficulty, always turn on attacking player
	//	case 2:
	//	default:
	//		luckThreshold = 0.0f;
	//		break;
	//	}

	//	//Randomly pick up the target
	//	if ( random() > luckThreshold )
	//	{
	//		G_ClearEnemy( other );
	//		other->enemy = NPC;
	//	}

	//	return;
	// }
}

unsafe fn NPC_SetPainEvent(self_: *mut c_void)
{
	// if ( !self->NPC || !(self->NPC->aiFlags&NPCAI_DIE_ON_IMPACT) )
	// {
	// no more borg
	//	if( self->client->playerTeam != TEAM_BORG )
	//	{
	//		if ( !Q3_TaskIDPending( self, TID_CHAN_VOICE ) )
	//		{
	//			G_AddEvent( self, EV_PAIN, floor((float)self->health/self->max_health*100.0f) );
	//		}
	//	}
	// }
}

/*
-------------------------
NPC_GetPainChance
-------------------------
*/

unsafe fn NPC_GetPainChance(self_: *mut c_void, damage: c_int) -> f32
{
	// if ( !self->enemy )
	// {//surprised, always take pain
	//	return 1.0f;
	// }

	// if ( damage > self->max_health/2.0f )
	// {
	//	return 1.0f;
	// }

	// let pain_chance = (self->max_health-self->health) as f32 / (self->max_health as f32 * 2.0f) + damage as f32 / (self->max_health as f32 / 2.0f);
	// switch ( g_spskill->integer )
	// {
	// case 0:	//easy
	//	//return 0.75f;
	//	break;

	// case 1://med
	//	pain_chance *= 0.5f;
	//	//return 0.35f;
	//	break;

	// case 2://hard
	// default:
	//	pain_chance *= 0.1f;
	//	//return 0.05f;
	//	break;
	// }
	//Com_Printf( "%s: %4.2f\n", self->NPC_type, pain_chance );
	// return pain_chance;

	1.0f  // stub return
}

/*
-------------------------
NPC_ChoosePainAnimation
-------------------------
*/

unsafe fn NPC_ChoosePainAnimation(
	self_: *mut c_void,
	other: *mut c_void,
	point: *const [f32; 3],
	damage: c_int,
	mod_: c_int,
	hitLoc: c_int,
	voiceEvent: c_int
)
{
	//If we've already taken pain, then don't take it again
	// if ( level.time < self->painDebounceTime && mod != MOD_ELECTROCUTE && mod != MOD_MELEE )
	// {//FIXME: if hit while recoving from losing a saber lock, we should still play a pain anim?
	//	return;
	// }

	// let mut pain_anim: c_int = -1;
	// let mut pain_chance: f32;

	// if ( self->s.weapon == WP_THERMAL && self->client->fireDelay > 0 )
	// {//don't interrupt thermal throwing anim
	//	return;
	// }
	// else if (self->client->ps.powerups[PW_GALAK_SHIELD])
	// {
	//	return;
	// }
	// else if ( self->client->NPC_class == CLASS_GALAKMECH )
	// {
	//	if ( hitLoc == HL_GENERIC1 )
	//	{//hit the antenna!
	//		pain_chance = 1.0f;
	//		self->s.powerups |= ( 1 << PW_SHOCKED );
	//		self->client->ps.powerups[PW_SHOCKED] = level.time + Q_irand( 500, 2500 );
	//	}
	//	else if ( self->client->ps.powerups[PW_GALAK_SHIELD] )
	//	{//shield up
	//		return;
	//	}
	//	else if ( self->health > 200 && damage < 100 )
	//	{//have a *lot* of health
	//		pain_chance = 0.05f;
	//	}
	//	else
	//	{//the lower my health and greater the damage, the more likely I am to play a pain anim
	//		pain_chance = (200.0f-self->health as f32)/100.0f + damage as f32/50.0f;
	//	}
	// }
	// else if ( self->client && self->client->playerTeam == TEAM_PLAYER && other && !other->s.number )
	// {//ally shot by player always complains
	//	pain_chance = 1.1f;
	// }
	// else
	// {
	//	if ( other && other->s.weapon == WP_SABER || mod == MOD_ELECTROCUTE || mod == MOD_CRUSH/*FIXME:MOD_FORCE_GRIP*/ )
	//	{
	//		if ( self->client->ps.weapon == WP_SABER
	//			&& other->s.number < MAX_CLIENTS )
	//		{//hmm, shouldn't *always* react to damage from player if I have a saber
	//			pain_chance = 1.05f - ((self->NPC->rank)/(f32)RANK_CAPTAIN);
	//		}
	//		else
	//		{
	//			pain_chance = 1.0f;//always take pain from saber
	//		}
	//	}
	//	else if ( mod == MOD_GAS )
	//	{
	//		pain_chance = 1.0f;
	//	}
	//	else if ( mod == MOD_MELEE )
	//	{//higher in rank (skill) we are, less likely we are to be fazed by a punch
	//		pain_chance = 1.0f - ((RANK_CAPTAIN-self->NPC->rank) as f32/(RANK_CAPTAIN as f32));
	//	}
	//	else if ( self->client->NPC_class == CLASS_PROTOCOL )
	//	{
	//		pain_chance = 1.0f;
	//	}
	//	else
	//	{
	//		pain_chance = NPC_GetPainChance( self, damage );
	//	}
	//	if ( self->client->NPC_class == CLASS_DESANN )
	//	{
	//		pain_chance *= 0.5f;
	//	}
	// }

	//See if we're going to flinch
	// if ( random() < pain_chance )
	// {
	//	//Pick and play our animation
	//	if ( (self->client->ps.eFlags&EF_FORCE_GRIPPED) )
	//	{
	//		G_AddVoiceEvent( self, Q_irand(EV_CHOKE1, EV_CHOKE3), 0 );
	//	}
	//	else if ( mod == MOD_GAS )
	//	{
	//		//SIGH... because our choke sounds are inappropriately long, I have to debounce them in code!
	//		if ( TIMER_Done( self, "gasChokeSound" ) )
	//		{
	//			TIMER_Set( self, "gasChokeSound", Q_irand( 1000, 2000 ) );
	//			G_AddVoiceEvent( self, Q_irand(EV_CHOKE1, EV_CHOKE3), 0 );
	//		}
	//	}
	//	else if ( (self->client->ps.eFlags&EF_FORCE_DRAINED) )
	//	{
	//		NPC_SetPainEvent( self );
	//	}
	//	else
	//	{//not being force-gripped or force-drained
	//		if ( G_CheckForStrongAttackMomentum( self )
	//			|| PM_SpinningAnim( self->client->ps.legsAnim )
	//			|| PM_SaberInSpecialAttack( self->client->ps.torsoAnim )
	//			|| PM_InKnockDown( &self->client->ps )
	//			|| PM_RollingAnim( self->client->ps.legsAnim )
	//			|| (PM_FlippingAnim( self->client->ps.legsAnim )&&!PM_InCartwheel( self->client->ps.legsAnim )) )
	//		{//strong attacks, rolls, knockdowns, flips and spins cannot be interrupted by pain
	//		}
	//		else
	//		{//play an anim
	//			if ( self->client->NPC_class == CLASS_GALAKMECH )
	//			{//only has 1 for now
	//				//FIXME: never plays this, it seems...
	//				pain_anim = BOTH_PAIN1;
	//			}
	//			else if ( mod == MOD_MELEE )
	//			{
	//				pain_anim = PM_PickAnim( self, BOTH_PAIN2, BOTH_PAIN3 );
	//			}
	//			else if ( self->s.weapon == WP_SABER )
	//			{//temp HACK: these are the only 2 pain anims that look good when holding a saber
	//				pain_anim = PM_PickAnim( self, BOTH_PAIN2, BOTH_PAIN3 );
	//			}
	//			else if ( mod != MOD_ELECTROCUTE )
	//			{
	//				pain_anim = G_PickPainAnim( self, point, damage, hitLoc );
	//			}

	//			if ( pain_anim == -1 )
	//			{
	//				pain_anim = PM_PickAnim( self, BOTH_PAIN1, BOTH_PAIN18 );
	//			}
	//			self->client->ps.saberAnimLevel = SS_FAST;//next attack must be a quick attack
	//			self->client->ps.saberMove = LS_READY;//don't finish whatever saber move you may have been in
	//			let parts = SETANIM_BOTH;
	//			if ( PM_CrouchAnim( self->client->ps.legsAnim ) || PM_InCartwheel( self->client->ps.legsAnim ) )
	//			{
	//				parts = SETANIM_LEGS;
	//			}
	//			self->NPC->aiFlags &= ~NPCAI_KNEEL;
	//			NPC_SetAnim( self, parts, pain_anim, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD );
	//		}
	//		if ( voiceEvent != -1 )
	//		{
	//			G_AddVoiceEvent( self, voiceEvent, Q_irand( 2000, 4000 ) );
	//		}
	//		else
	//		{
	//			NPC_SetPainEvent( self );
	//		}
	//	}
	//
	//	//Setup the timing for it
	//	if ( mod == MOD_ELECTROCUTE )
	//	{
	//		self->painDebounceTime = level.time + 4000;
	//	}
	//	self->painDebounceTime = level.time + PM_AnimLength( self->client->clientInfo.animFileIndex, (animNumber_t) pain_anim );
	//	self->client->fireDelay = 0;
	// }
}

unsafe fn G_CheckControlledTurretEnemy(self_: *mut c_void, mut enemy: *mut c_void, validate: c_int) -> *mut c_void
{
	// if ( enemy->e_UseFunc == useF_emplaced_gun_use
	//	|| enemy->e_UseFunc == useF_eweb_use )
	// {
	//	if ( enemy->activator
	//		&& enemy->activator->client )
	//	{//return the controller of the eweb/emplaced gun
	//		if (validate==qfalse || !self->client || G_ValidEnemy(self, enemy))
	//		{
	//			return enemy->activator;
	//		}
	//	}
	//	return core::ptr::null_mut();
	// }
	enemy
}

/*
===============
NPC_Pain
===============
*/
unsafe fn NPC_Pain(
	self_: *mut c_void,
	inflictor: *mut c_void,
	other: *mut c_void,
	point: *const [f32; 3],
	damage: c_int,
	mod_: c_int,
	hitLoc: c_int
)
{
	// let mut otherTeam: c_int = TEAM_FREE;
	// let mut voiceEvent: c_int = -1;

	// if ( self->NPC == NULL )
	//	return;

	// if ( other == NULL )
	//	return;

	// //or just remove ->pain in player_die?
	// if ( self->client->ps.pm_type == PM_DEAD )
	//	return;

	// if ( other == self )
	//	return;

	// other = G_CheckControlledTurretEnemy(self, other, 0);
	// if (!other)
	// {
	//	return;
	// }

	// //MCG: Ignore damage from your own team for now
	// if ( other->client )
	// {
	//	otherTeam = other->client->playerTeam;
	// //	if ( otherTeam == TEAM_DISGUISE )
	// //	{
	// //		otherTeam = TEAM_PLAYER;
	// //	}
	// }

	// if ( self->client->playerTeam
	//	&& other->client
	//	&& otherTeam == self->client->playerTeam
	//	&& (!player->client->ps.viewEntity || other->s.number != player->client->ps.viewEntity))
	// {//hit by a teammate
	//	if ( other != self->enemy && self != other->enemy )
	//	{//we weren't already enemies
	//		if ( self->enemy || other->enemy
	//			|| (other->s.number&&other->s.number!=player->client->ps.viewEntity)
	//			/*|| (!other->s.number&&Q_irand( 0, 3 ))*/ )
	//		{//if one of us actually has an enemy already, it's okay, just an accident OR wasn't hit by player or someone controlled by player OR player hit ally and didn't get 25% chance of getting mad (FIXME:accumulate anger+base on diff?)
	//			//FIXME: player should have to do a certain amount of damage to ally or hit them several times to make them mad
	//			//Still run pain and flee scripts
	//			if ( self->client && self->NPC )
	//			{//Run any pain instructions
	//				if ( self->health <= (self->max_health/3) && G_ActivateBehavior(self, BSET_FLEE) )
	//				{
	//
	//				}
	//				else// if( VALIDSTRING( self->behaviorSet[BSET_PAIN] ) )
	//				{
	//					G_ActivateBehavior(self, BSET_PAIN);
	//				}
	//			}
	//			if ( damage != -1 )
	//			{//-1 == don't play pain anim
	//				//Set our proper pain animation
	//				if ( Q_irand( 0, 1 ) )
	//				{
	//					NPC_ChoosePainAnimation( self, other, point, damage, mod, hitLoc, EV_FFWARN );
	//				}
	//				else
	//				{
	//					NPC_ChoosePainAnimation( self, other, point, damage, mod, hitLoc );
	//				}
	//			}
	//			return;
	//		}
	//		else if ( self->NPC && !other->s.number )//should be assumed, but...
	//		{//dammit, stop that!
	//			if ( self->NPC->charmedTime > level.time )
	//			{//mindtricked
	//				return;
	//			}
	//			else if ( self->NPC->ffireCount < 3+((2-g_spskill->integer)*2) )
	//			{//not mad enough yet
	//				//Com_Printf( "chck: %d < %d\n", self->NPC->ffireCount, 3+((2-g_spskill->integer)*2) );
	//				if ( damage != -1 )
	//				{//-1 == don't play pain anim
	//					//Set our proper pain animation
	//					if ( Q_irand( 0, 1 ) )
	//					{
	//						NPC_ChoosePainAnimation( self, other, point, damage, mod, hitLoc, EV_FFWARN );
	//					}
	//					else
	//					{
	//						NPC_ChoosePainAnimation( self, other, point, damage, mod, hitLoc );
	//					}
	//				}
	//				return;
	//			}
	//			else if ( G_ActivateBehavior( self, BSET_FFIRE ) )
	//			{//we have a specific script to run, so do that instead
	//				return;
	//			}
	//			else
	//			{//okay, we're going to turn on our ally, we need to set and lock our enemy and put ourselves in a bstate that lets us attack him (and clear any flags that would stop us)
	//				self->NPC->blockedSpeechDebounceTime = 0;
	//				voiceEvent = EV_FFTURN;
	//				self->NPC->behaviorState = self->NPC->tempBehavior = self->NPC->defaultBehavior = BS_DEFAULT;
	//				other->flags &= ~FL_NOTARGET;
	//				self->svFlags &= ~(SVF_IGNORE_ENEMIES|SVF_ICARUS_FREEZE|SVF_NO_COMBAT_SOUNDS);
	//				G_SetEnemy( self, other );
	//				self->svFlags |= SVF_LOCKEDENEMY;
	//				self->NPC->scriptFlags &= ~(SCF_DONT_FIRE|SCF_CROUCHED|SCF_WALKING|SCF_NO_COMBAT_TALK|SCF_FORCED_MARCH);
	//				self->NPC->scriptFlags |= (SCF_CHASE_ENEMIES|SCF_NO_MIND_TRICK);
	//				//NOTE: we also stop ICARUS altogether
	//				stop_icarus = qtrue;
	//				if ( !killPlayerTimer )
	//				{
	//					killPlayerTimer = level.time + 10000;
	//				}
	//			}
	//		}
	//	}
	// }

	// SaveNPCGlobals();
	// SetNPCGlobals( self );

	// //Do extra bits
	// if ( NPCInfo->ignorePain == qfalse )
	// {
	//	NPCInfo->confusionTime = 0;//clear any charm or confusion, regardless
	//	if ( NPC->ghoul2.size() && NPC->headBolt != -1 )
	//	{
	//		G_StopEffect("force/confusion", NPC->playerModel, NPC->headBolt, NPC->s.number );
	//	}
	//	if ( damage != -1 )
	//	{//-1 == don't play pain anim
	//		//Set our proper pain animation
	//		NPC_ChoosePainAnimation( self, other, point, damage, mod, hitLoc, voiceEvent );
	//	}
	//	//Check to take a new enemy
	//	if ( NPC->enemy != other && NPC != other )
	//	{//not already mad at them
	//		//if it's an eweb or emplaced gun, get mad at the owner, not the gun
	//		NPC_CheckAttacker( other, mod );
	//	}
	// }

	// //Attempt to run any pain instructions
	// if ( self->client && self->NPC )
	// {
	//	//FIXME: This needs better heuristics perhaps
	//	if(self->health <= (self->max_health/3) && G_ActivateBehavior(self, BSET_FLEE) )
	//	{
	//	}
	//	else //if( VALIDSTRING( self->behaviorSet[BSET_PAIN] ) )
	//	{
	//		G_ActivateBehavior(self, BSET_PAIN);
	//	}
	// }

	// //Attempt to fire any paintargets we might have
	// if( self->paintarget && self->paintarget[0] )
	// {
	//	G_UseTargets2(self, other, self->paintarget);
	// }

	// if (self->client && self->client->NPC_class==CLASS_BOBAFETT)
	// {
	//	Boba_Pain( self, inflictor, damage, mod);
	// }


	// RestoreNPCGlobals();
}

/*
-------------------------
NPC_Touch
-------------------------
*/
unsafe fn NPC_Touch(self_: *mut c_void, other: *mut c_void, trace: *mut c_void)
{
	// if(!self->NPC)
	//	return;

	// SaveNPCGlobals();
	// SetNPCGlobals( self );

	// if ( self->message && self->health <= 0 )
	// {//I am dead and carrying a key
	//	if ( other && player && player->health > 0 && other == player )
	//	{//player touched me
	//		let text: *const c_char;
	//		let mut keyTaken: c_int;
	//		//give him my key
	//		if ( Q_stricmp( "goodie", self->message ) == 0 )
	//		{//a goodie key
	//			if ( (keyTaken = INV_GoodieKeyGive( other )) == 1 )
	//			{
	//				text = "cp @SP_INGAME_TOOK_IMPERIAL_GOODIE_KEY";
	//				G_AddEvent( other, EV_ITEM_PICKUP, (FindItemForInventory( INV_GOODIE_KEY )-bg_itemlist) as c_int );
	//			}
	//			else
	//			{
	//				text = "cp @SP_INGAME_CANT_CARRY_GOODIE_KEY";
	//			}
	//		}
	//		else
	//		{//a named security key
	//			if ( (keyTaken = INV_SecurityKeyGive( player, self->message )) == 1 )
	//			{
	//				text = "cp @SP_INGAME_TOOK_IMPERIAL_SECURITY_KEY";
	//				G_AddEvent( other, EV_ITEM_PICKUP, (FindItemForInventory( INV_SECURITY_KEY )-bg_itemlist) as c_int );
	//			}
	//			else
	//			{
	//				text = "cp @SP_INGAME_CANT_CARRY_SECURITY_KEY";
	//			}
	//		}
	//		if ( keyTaken )
	//		{//remove my key
	//			gi.G2API_SetSurfaceOnOff( &self->ghoul2[self->playerModel], "l_arm_key", 0x00000002 );
	//			self->message = core::ptr::null();
	//			self->client->ps.eFlags &= ~EF_FORCE_VISIBLE;	//remove sight flag
	//			G_Sound( player, G_SoundIndex( "sound/weapons/key_pkup.wav" ) );
	//		}
	//		gi.SendServerCommand( NULL, text );
	//	}
	// }

	// if ( other->client )
	// {//FIXME:  if pushing against another bot, both ucmd.rightmove = 127???
	//	//Except if not facing one another...
	//	if ( other->health > 0 )
	//	{
	//		NPCInfo->touchedByPlayer = other;
	//	}

	//	if ( other == NPCInfo->goalEntity )
	//	{
	//		NPCInfo->aiFlags |= NPCAI_TOUCHED_GOAL;
	//	}

	//	if( !(self->svFlags&SVF_LOCKEDENEMY) && !(self->svFlags&SVF_IGNORE_ENEMIES) && !(other->flags & FL_NOTARGET) )
	//	{
	//		if ( self->client->enemyTeam )
	//		{//See if we bumped into an enemy
	//			if ( other->client->playerTeam == self->client->enemyTeam )
	//			{//bumped into an enemy
	//				if( NPCInfo->behaviorState != BS_HUNT_AND_KILL && !NPCInfo->tempBehavior )
	//				{//MCG - Begin: checking specific BS mode here, this is bad, a HACK
	//					//FIXME: not medics?
	//					if ( NPC->enemy != other )
	//					{//not already mad at them
	//						G_SetEnemy( NPC, other );
	//					}
	//		//				NPCInfo->tempBehavior = BS_HUNT_AND_KILL;
	//					}
	//				}
	//			}
	//		}
	//	}

	//	//FIXME: do this if player is moving toward me and with a certain dist?
	//	/*
	//	if ( other->s.number == 0 && self->client->playerTeam == other->client->playerTeam )
	//	{
	//		VectorAdd( self->client->pushVec, other->client->ps.velocity, self->client->pushVec );
	//	}
	//	*/
	// }
	// else
	// {//FIXME: check for SVF_NONNPC_ENEMY flag here?
	//	if ( other->health > 0 )
	//	{
	//		if ( NPC->enemy == other && (other->svFlags&SVF_NONNPC_ENEMY) )
	//		{
	//			NPCInfo->touchedByPlayer = other;
	//		}
	//	}

	//	if ( other == NPCInfo->goalEntity )
	//	{
	//		NPCInfo->aiFlags |= NPCAI_TOUCHED_GOAL;
	//	}
	// }

	// if ( NPC->client->NPC_class == CLASS_RANCOR )
	// {//rancor
	//	if ( NPCInfo->blockedEntity != other && TIMER_Done(NPC, "blockedEntityIgnore"))
	//	{//blocked
	//		//if ( G_EntIsBreakable( other->s.number, NPC ) )
	//		{//bumped into another breakable, so take that one instead?
	//			NPCInfo->blockedEntity = other;//???
	//		}
	//	}
	// }

	// RestoreNPCGlobals();
}

/*
-------------------------
NPC_TempLookTarget
-------------------------
*/

unsafe fn NPC_TempLookTarget(self_: *mut c_void, mut lookEntNum: c_int, mut minLookTime: c_int, mut maxLookTime: c_int)
{
	// if ( !self->client )
	// {
	//	return;
	// }

	// if ( !minLookTime )
	// {
	//	minLookTime = 1000;
	// }

	// if ( !maxLookTime )
	// {
	//	maxLookTime = 1000;
	// }

	// if ( !NPC_CheckLookTarget( self ) )
	// {//Not already looking at something else
	//	//Look at him for 1 to 3 seconds
	//	NPC_SetLookTarget( self, lookEntNum, level.time + Q_irand( minLookTime, maxLookTime ) );
	// }
}

unsafe fn NPC_Respond(self_: *mut c_void, userNum: c_int)
{
	// let mut event: c_int = -1;
	// /*

	// if ( Q_irand( 0, 1 ) )
	// {
	//	event = Q_irand(EV_RESPOND1, EV_RESPOND3);
	// }
	// else
	// {
	//	event = Q_irand(EV_BUSY1, EV_BUSY3);
	// }
	// */

	// if ( !Q_irand( 0, 1 ) )
	// {//set looktarget to them for a second or two
	//	NPC_TempLookTarget( self, userNum, 1000, 3000 );
	// }

	// //some last-minute hacked in responses
	// match ( self->client->NPC_class )
	// {
	// case CLASS_JAN:
	//	if ( self->enemy )
	//	{
	//		if ( !Q_irand( 0, 2 ) )
	//		{
	//			event = Q_irand( EV_CHASE1, EV_CHASE3 );
	//		}
	//		else if ( Q_irand( 0, 1 ) )
	//		{
	//			event = Q_irand( EV_OUTFLANK1, EV_OUTFLANK2 );
	//		}
	//		else
	//		{
	//			event = Q_irand( EV_COVER1, EV_COVER5 );
	//		}
	//	}
	//	else if ( !Q_irand( 0, 2 ) )
	//	{
	//		event = EV_SUSPICIOUS4;
	//	}
	//	else if ( !Q_irand( 0, 1 ) )
	//	{
	//		event = EV_SOUND1;
	//	}
	//	else
	//	{
	//		event = EV_CONFUSE1;
	//	}
	//	break;
	// case CLASS_LANDO:
	//	if ( self->enemy )
	//	{
	//		if ( !Q_irand( 0, 2 ) )
	//		{
	//			event = Q_irand( EV_CHASE1, EV_CHASE3 );
	//		}
	//		else if ( Q_irand( 0, 1 ) )
	//		{
	//			event = Q_irand( EV_OUTFLANK1, EV_OUTFLANK2 );
	//		}
	//		else
	//		{
	//			event = Q_irand( EV_COVER1, EV_COVER5 );
	//		}
	//	}
	//	else if ( !Q_irand( 0, 6 ) )
	//	{
	//		event = EV_SIGHT2;
	//	}
	//	else if ( !Q_irand( 0, 5 ) )
	//	{
	//		event = EV_GIVEUP4;
	//	}
	//	else if ( Q_irand( 0, 4 ) > 1 )
	//	{
	//		event = Q_irand( EV_SOUND1, EV_SOUND3 );
	//	}
	//	else
	//	{
	//		event = Q_irand( EV_JDETECTED1, EV_JDETECTED2 );
	//	}
	//	break;
	// case CLASS_LUKE:
	//	if ( self->enemy )
	//	{
	//		event = EV_COVER1;
	//	}
	//	else
	//	{
	//		event = Q_irand( EV_SOUND1, EV_SOUND3 );
	//	}
	//	break;
	// case CLASS_JEDI:
	// case CLASS_KYLE:
	//	if ( !self->enemy )
	//	{
	//		/*
	//		if ( !(self->svFlags&SVF_IGNORE_ENEMIES)
	//			&& (self->NPC->scriptFlags&SCF_LOOK_FOR_ENEMIES)
	//			&& self->client->enemyTeam == TEAM_ENEMY )
	//		{
	//			event = Q_irand( EV_ANGER1, EV_ANGER3 );
	//		}
	//		else
	//		*/
	//		{
	//			event = Q_irand( EV_CONFUSE1, EV_CONFUSE3 );
	//		}
	//	}
	//	break;
	// case CLASS_PRISONER:
	//	if ( self->enemy )
	//	{
	//		if ( Q_irand( 0, 1 ) )
	//		{
	//			event = Q_irand( EV_CHASE1, EV_CHASE3 );
	//		}
	//		else
	//		{
	//			event = Q_irand( EV_OUTFLANK1, EV_OUTFLANK2 );
	//		}
	//	}
	//	else
	//	{
	//		event = Q_irand( EV_SOUND1, EV_SOUND3 );
	//	}
	//	break;
	// case CLASS_REBEL:
	//	if ( self->enemy )
	//	{
	//		if ( !Q_irand( 0, 2 ) )
	//		{
	//			event = Q_irand( EV_CHASE1, EV_CHASE3 );
	//		}
	//		else
	//		{
	//			event = Q_irand( EV_DETECTED1, EV_DETECTED5 );
	//		}
	//	}
	//	else
	//	{
	//		event = Q_irand( EV_SOUND1, EV_SOUND3 );
	//	}
	//	break;
	// case CLASS_BESPIN_COP:
	//	if ( !Q_stricmp( "bespincop", self->NPC_type ) )
	//	{//variant 1
	//		if ( self->enemy )
	//		{
	//			if ( Q_irand( 0, 9 ) > 6 )
	//			{
	//				event = Q_irand( EV_CHASE1, EV_CHASE3 );
	//			}
	//			else if ( Q_irand( 0, 6 ) > 4 )
	//			{
	//				event = Q_irand( EV_OUTFLANK1, EV_OUTFLANK2 );
	//			}
	//			else
	//			{
	//				event = Q_irand( EV_COVER1, EV_COVER5 );
	//			}
	//		}
	//		else if ( !Q_irand( 0, 3 ) )
	//		{
	//			event = Q_irand( EV_SIGHT2, EV_SIGHT3 );
	//		}
	//		else if ( !Q_irand( 0, 1 ) )
	//		{
	//			event = Q_irand( EV_SOUND1, EV_SOUND3 );
	//		}
	//		else if ( !Q_irand( 0, 2 ) )
	//		{
	//			event = EV_LOST1;
	//		}
	//		else if ( !Q_irand( 0, 1 ) )
	//		{
	//			event = EV_ESCAPING2;
	//		}
	//		else
	//		{
	//			event = EV_GIVEUP4;
	//		}
	//	}
	//	else
	//	{//variant2
	//		if ( self->enemy )
	//		{
	//			if ( Q_irand( 0, 9 ) > 6 )
	//			{
	//				event = Q_irand( EV_CHASE1, EV_CHASE3 );
	//			}
	//			else if ( Q_irand( 0, 6 ) > 4 )
	//			{
	//				event = Q_irand( EV_OUTFLANK1, EV_OUTFLANK2 );
	//			}
	//			else
	//			{
	//				event = Q_irand( EV_COVER1, EV_COVER5 );
	//			}
	//		}
	//		else if ( !Q_irand( 0, 3 ) )
	//		{
	//			event = Q_irand( EV_SIGHT1, EV_SIGHT2 );
	//		}
	//		else if ( !Q_irand( 0, 1 ) )
	//		{
	//			event = Q_irand( EV_SOUND1, EV_SOUND3 );
	//		}
	//		else if ( !Q_irand( 0, 2 ) )
	//		{
	//			event = EV_LOST1;
	//		}
	//		else if ( !Q_irand( 0, 1 ) )
	//		{
	//			event = EV_GIVEUP3;
	//		}
	//		else
	//		{
	//			event = EV_CONFUSE1;
	//		}
	//	}
	//	break;
	// case CLASS_R2D2:				// droid
	//	G_Sound(self, G_SoundIndex(va("sound/chars/r2d2/misc/r2d2talk0%d.wav",Q_irand(1, 3))));
	//	break;
	// case CLASS_R5D2:				// droid
	//	G_Sound(self, G_SoundIndex(va("sound/chars/r5d2/misc/r5talk%d.wav",Q_irand(1, 4))));
	//	break;
	// case CLASS_MOUSE:				// droid
	//	G_Sound(self, G_SoundIndex(va("sound/chars/mouse/misc/mousego%d.wav",Q_irand(1, 3))));
	//	break;
	// case CLASS_GONK:				// droid
	//	G_Sound(self, G_SoundIndex(va("sound/chars/gonk/misc/gonktalk%d.wav",Q_irand(1, 2))));
	//	break;
	// case CLASS_JAWA:
	//	G_SoundOnEnt(self, CHAN_VOICE, va("sound/chars/jawa/misc/chatter%d.wav",Q_irand(1, 6)) );
	//	if ( self->NPC )
	//	{
	//		self->NPC->blockedSpeechDebounceTime = level.time + 2000;
	//	}
	//	break;
	// }

	// if ( event != -1 )
	// {
	//	//hack here because we reuse some "combat" and "extra" sounds
	//	let addFlag = (self->NPC->scriptFlags&SCF_NO_COMBAT_TALK);
	//	self->NPC->scriptFlags &= ~SCF_NO_COMBAT_TALK;

	//	G_AddVoiceEvent( self, event, 3000 );

	//	if ( addFlag )
	//	{
	//		self->NPC->scriptFlags |= SCF_NO_COMBAT_TALK;
	//	}
	// }
}

/*
-------------------------
NPC_UseResponse
-------------------------
*/

unsafe fn NPC_UseResponse(self_: *mut c_void, user: *mut c_void, useWhenDone: c_int)
{
	// if ( !self->NPC || !self->client )
	// {
	//	return;
	// }

	// if ( user->s.number != 0 )
	// {//not used by the player
	//	if ( useWhenDone )
	//	{
	//		G_ActivateBehavior( self, BSET_USE );
	//	}
	//	return;
	// }

	// if ( user->client && self->client->playerTeam != user->client->playerTeam && self->client->playerTeam != TEAM_NEUTRAL )
	// {//only those on the same team react
	//	if ( useWhenDone )
	//	{
	//		G_ActivateBehavior( self, BSET_USE );
	//	}
	//	return;
	// }

	// if ( self->NPC->blockedSpeechDebounceTime > level.time )
	// {//I'm not responding right now
	//	return;
	// }

	// if ( gi.VoiceVolume[self->s.number] )
	// {//I'm talking already
	//	if ( !useWhenDone )
	//	{//you're not trying to use me
	//		return;
	//	}
	// }

	// if ( useWhenDone )
	// {
	//	G_ActivateBehavior( self, BSET_USE );
	// }
	// else
	// {
	//	NPC_Respond( self, user->s.number );
	// }
}

/*
-------------------------
NPC_Use
-------------------------
*/
unsafe fn NPC_Use(self_: *mut c_void, other: *mut c_void, activator: *mut c_void)
{
	// if (self->client->ps.pm_type == PM_DEAD)
	// {//or just remove ->pain in player_die?
	//	return;
	// }

	// SaveNPCGlobals();
	// SetNPCGlobals( self );

	// if(self->client && self->NPC)
	// {
	//	// If this is a vehicle, let the other guy board it. Added 12/14/02 by AReis.
	//	if ( self->client->NPC_class == CLASS_VEHICLE )
	//	{
	//		let pVeh: *mut Vehicle_t = self->m_pVehicle;

	//		if ( pVeh && pVeh->m_pVehicleInfo && other && other->client )
	//		{//safety
	//			//if I used myself, eject everyone on me
	//			if ( other == self )
	//			{
	//				pVeh->m_pVehicleInfo->EjectAll( pVeh );
	//			}
	//			// If other is already riding this vehicle (self), eject him.
	//			else if ( other->owner == self )
	//			{
	//				pVeh->m_pVehicleInfo->Eject( pVeh, other, 0 );
	//			}
	//			// Otherwise board this vehicle.
	//			else
	//			{
	//				pVeh->m_pVehicleInfo->Board( pVeh, other );
	//			}
	//		}
	//	}
	//	else if ( Jedi_WaitingAmbush( NPC ) )
	//	{
	//		Jedi_Ambush( NPC );
	//	}
	//	//Run any use instructions
	//	if ( activator && activator->s.number == 0 && self->client->NPC_class == CLASS_GONK )
	//	{
	//		// must be using the gonk, so attempt to give battery power.
	//		// NOTE: this will steal up to MAX_BATTERIES for the activator, leaving the residual on the gonk for potential later use.
	//		Add_Batteries( activator, &self->client->ps.batteryCharge );
	//	}
	//	// Not using MEDICs anymore
	// /*
	//	if ( self->NPC->behaviorState == BS_MEDIC_HIDE && activator->client )
	//	{//Heal me NOW, dammit!
	//		if ( activator->health < activator->max_health )
	//		{//person needs help
	//			if ( self->NPC->eventualGoal != activator )
	//			{//not my current patient already
	//				NPC_TakePatient( activator );
	//				G_ActivateBehavior( self, BSET_USE );
	//			}
	//		}
	//		else if ( !self->enemy && activator->s.number == 0 && !gi.VoiceVolume[self->s.number] && !(self->NPC->scriptFlags&SCF_NO_RESPONSE) )
	//		{//I don't have an enemy and I'm not talking and I was used by the player
	//			NPC_UseResponse( self, other, 0 );
	//		}
	//	}
	// */
	// //		else if ( self->behaviorSet[BSET_USE] )

	//	if ( self->behaviorSet[BSET_USE] )
	//	{
	//		NPC_UseResponse( self, other, 1 );
	//	}
	// //		else if ( isMedic( self ) )
	// //		{//Heal me NOW, dammit!
	// //			NPC_TakePatient( activator );
	// //		}
	//	else if ( !self->enemy
	//		//&& self->client->NPC_class == CLASS_VEHICLE
	//		&& activator->s.number == 0
	//		&& !gi.VoiceVolume[self->s.number]
	//		&& !(self->NPC->scriptFlags&SCF_NO_RESPONSE) )
	//	{//I don't have an enemy and I'm not talking and I was used by the player
	//		NPC_UseResponse( self, other, 0 );
	//	}
	// }

	// RestoreNPCGlobals();
}

unsafe fn NPC_CheckPlayerAim()
{
	//FIXME: need appropriate dialogue
	/*
	let player = &g_entities[0];

	if ( player && player->client && player->client->ps.weapon > (int)(WP_NONE) && player->client->ps.weapon < (int)(WP_TRICORDER) )
	{//player has a weapon ready
		if ( g_crosshairEntNum == NPC->s.number && level.time - g_crosshairEntTime < 200
			&& g_crosshairSameEntTime >= 3000 && g_crosshairEntDist < 256 )
		{//if the player holds the crosshair on you for a few seconds
			//ask them what the fuck they're doing
			G_AddVoiceEvent( NPC, Q_irand( EV_FF_1A, EV_FF_1C ), 0 );
		}
	}
	*/
}

unsafe fn NPC_CheckAllClear()
{
	//FIXME: need to make this happen only once after losing enemies, not over and over again
	/*
	if ( NPC->client && !NPC->enemy && level.time - teamLastEnemyTime[NPC->client->playerTeam] > 10000 )
	{//Team hasn't seen an enemy in 10 seconds
		if ( !Q_irand( 0, 2 ) )
		{
			G_AddVoiceEvent( NPC, Q_irand(EV_SETTLE1, EV_SETTLE3), 3000 );
		}
	}
	*/
}
