// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
// include "g_headers.h"
// include "b_local.h"
// include "g_nav.h"

use core::ffi::{c_int, c_char};

const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;

const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;

const TURN_OFF: c_int = 0x00000100;

const LEFT_ARM_HEALTH: c_int = 40;
const RIGHT_ARM_HEALTH: c_int = 40;
const AMMO_POD_HEALTH: c_int = 40;

const BOWCASTER_VELOCITY: c_int = 1300;
const BOWCASTER_NPC_DAMAGE_EASY: c_int = 12;
const BOWCASTER_NPC_DAMAGE_NORMAL: c_int = 24;
const BOWCASTER_NPC_DAMAGE_HARD: c_int = 36;
const BOWCASTER_SIZE: c_int = 2;
const BOWCASTER_SPLASH_DAMAGE: c_int = 0;
const BOWCASTER_SPLASH_RADIUS: c_int = 0;

//Local state enums
const LSTATE_NONE: c_int = 0;
const LSTATE_ASLEEP: c_int = 1;
const LSTATE_WAKEUP: c_int = 2;
const LSTATE_FIRED0: c_int = 3;
const LSTATE_FIRED1: c_int = 4;
const LSTATE_FIRED2: c_int = 5;
const LSTATE_FIRED3: c_int = 6;
const LSTATE_FIRED4: c_int = 7;

extern "C" {
    fn NPC_CheckPlayerTeamStealth() -> c_int;
    fn CreateMissile(org: *const [f32; 3], dir: *const [f32; 3], vel: f32, life: c_int, owner: *mut gentity_t, altFire: c_int) -> *mut gentity_t;
    fn Mark1_BlasterAttack(advance: c_int);
    fn DeathFX(ent: *mut gentity_t);
    fn FindItemForAmmo(ammo: c_int) -> *mut gitem_t;
    fn G_SoundIndex(name: *const c_char) -> c_int;
    fn G_EffectIndex(name: *const c_char) -> c_int;
    fn RegisterItem(item: *mut gitem_t);
    fn FindItemForWeapon(weapon: c_int) -> *mut gitem_t;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn G_PlayEffect(effect: *const c_char, org: *const [f32; 3], dir: *const [f32; 3]);
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn NPC_BSIdle();
    fn NPC_SetAnim(ent: *mut gentity_t, setAnimParts: c_int, anim: c_int, flags: c_int);
    fn TIMER_Done(ent: *mut gentity_t, timer: *const c_char) -> c_int;
    fn TIMER_Set(ent: *mut gentity_t, timer: *const c_char, duration: c_int);
    fn CalcEntitySpot(ent: *mut gentity_t, spot: c_int, point: *mut [f32; 3]);
    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn vectoangles(vec: *const [f32; 3], angles: *mut [f32; 3]);
    fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    fn G_Sound(ent: *mut gentity_t, index: c_int);
    fn NPC_Pain(ent: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int);
    fn NPC_FaceEnemy(addPainAngle: c_int) -> c_int;
    fn NPC_CheckEnemyExt() -> c_int;
    fn DistanceHorizontalSquared(v1: *const [f32; 3], v2: *const [f32; 3]) -> f32;
    fn NPC_ClearLOS(ent: *mut gentity_t) -> c_int;
    fn NPC_MoveToGoal(allowUse: c_int);
    fn VectorSet(v: *mut [f32; 3], x: f32, y: f32, z: f32);
    fn VectorScale(v: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn G_Damage(target: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, dir: *mut [f32; 3], point: *mut [f32; 3], damage: c_int, dflags: c_int, mod_: c_int);
    fn SaveNPCGlobals();
    fn SetNPCGlobals(ent: *mut gentity_t);
    fn RestoreNPCGlobals();
    fn UpdateGoal() -> c_int;
    fn NPC_UpdateAngles(useSum: c_int, updateClient: c_int);
    fn GEntity_DieFunc(ent: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int);

    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut gNPCInfo_t;
    static mut ucmd: usercmd_t;
    static mut cg: cg_t;
    static mut level: level_locals_t;
}

// Placeholder structs - these would be defined elsewhere in the full codebase
#[repr(C)]
pub struct gentity_t {
    // This is a placeholder; the real struct is much larger
}

#[repr(C)]
pub struct gitem_t {
    // This is a placeholder
}

#[repr(C)]
pub struct mdxaBone_t {
    // This is a placeholder
}

#[repr(C)]
pub struct gNPCInfo_t {
    pub goalEntity: *mut gentity_t,
    pub localState: c_int,
    pub combatMove: c_int,
    pub burstCount: c_int,
    pub scriptFlags: c_int,
    // ... other fields
}

#[repr(C)]
pub struct usercmd_t {
    pub buttons: c_int,
    // ... other fields
}

#[repr(C)]
pub struct cg_t {
    pub time: c_int,
    // ... other fields
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
    // ... other fields
}

/*
-------------------------
NPC_Mark1_Precache
-------------------------
*/
pub extern "C" fn NPC_Mark1_Precache() {
    G_SoundIndex("sound/chars/mark1/misc/mark1_wakeup\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/shutdown\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/walk\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/run\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/death1\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/death2\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/anger\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/mark1_fire\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/mark1_pain\0".as_ptr() as *const c_char);
    G_SoundIndex("sound/chars/mark1/misc/mark1_explo\0".as_ptr() as *const c_char);

    //	G_EffectIndex( "small_chunks");
    G_EffectIndex("env/med_explode2\0".as_ptr() as *const c_char);
    G_EffectIndex("explosions/probeexplosion1\0".as_ptr() as *const c_char);
    G_EffectIndex("blaster/smoke_bolton\0".as_ptr() as *const c_char);
    G_EffectIndex("bryar/muzzle_flash\0".as_ptr() as *const c_char);
    G_EffectIndex("explosions/droidexplosion1\0".as_ptr() as *const c_char);

    RegisterItem(FindItemForAmmo(4)); // AMMO_METAL_BOLTS
    RegisterItem(FindItemForAmmo(1)); // AMMO_BLASTER
    RegisterItem(FindItemForWeapon(6)); // WP_BOWCASTER
    RegisterItem(FindItemForWeapon(1)); // WP_BRYAR_PISTOL
}

/*
-------------------------
NPC_Mark1_Part_Explode
-------------------------
*/
pub extern "C" fn NPC_Mark1_Part_Explode(self_: *mut gentity_t, bolt: c_int) {
    if bolt >= 0 {
        let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
        let mut org: [f32; 3] = [0.0; 3];
        let mut dir: [f32; 3] = [0.0; 3];

        unsafe {
            gi.G2API_GetBoltMatrix((*self_).ghoul2, (*self_).playerModel,
                bolt,
                &mut boltMatrix, (*self_).currentAngles, (*self_).currentOrigin, (cg.time != 0 ? cg.time : level.time),
                core::ptr::null_mut(), (*self_).s.modelScale);

            gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 0, &mut org); // ORIGIN = 0
            gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 2, &mut dir); // NEGATIVE_Y = 2
        }

        G_PlayEffect("env/med_explode2\0".as_ptr() as *const c_char, &org, &dir);
        G_PlayEffect(G_EffectIndex("blaster/smoke_bolton\0".as_ptr() as *const c_char), (*self_).playerModel, bolt, (*self_).s.number, &org);
    }
}

/*
-------------------------
Mark1_Idle
-------------------------
*/
pub extern "C" fn Mark1_Idle() {

    NPC_BSIdle();

    unsafe {
        NPC_SetAnim(NPC, 5, 79, 0); // SETANIM_BOTH=5, BOTH_SLEEP1=79, SETANIM_FLAG_NORMAL=0
    }
}

/*
-------------------------
Mark1Dead_FireRocket
- Shoot the left weapon, the multi-blaster
-------------------------
*/
pub extern "C" fn Mark1Dead_FireRocket() {
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let mut muzzle1: [f32; 3] = [0.0; 3];
    let mut muzzle_dir: [f32; 3] = [0.0; 3];

    let damage: c_int = 50;

    unsafe {
        gi.G2API_GetBoltMatrix((*NPC).ghoul2, (*NPC).playerModel,
            (*NPC).genericBolt5,
            &mut boltMatrix, (*NPC).currentAngles, (*NPC).currentOrigin, (cg.time != 0 ? cg.time : level.time),
            core::ptr::null_mut(), (*NPC).s.modelScale);

        gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 0, &mut muzzle1); // ORIGIN = 0
        gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 2, &mut muzzle_dir); // NEGATIVE_Y = 2

        G_PlayEffect("bryar/muzzle_flash\0".as_ptr() as *const c_char, &muzzle1, &muzzle_dir);

        G_Sound(NPC, G_SoundIndex("sound/chars/mark1/misc/mark1_fire\0".as_ptr() as *const c_char));

        let missile = CreateMissile(&muzzle1, &muzzle_dir, BOWCASTER_VELOCITY as f32, 10000, NPC, 0);

        (*missile).classname = "bowcaster_proj\0".as_ptr() as *const c_char;
        (*missile).s.weapon = 6; // WP_BOWCASTER

        VectorSet(&mut (*missile).maxs, BOWCASTER_SIZE as f32, BOWCASTER_SIZE as f32, BOWCASTER_SIZE as f32);
        VectorScale(&(*missile).maxs, -1.0, &mut (*missile).mins);

        (*missile).damage = damage;
        (*missile).dflags = 1; // DAMAGE_DEATH_KNOCKBACK
        (*missile).methodOfDeath = 10; // MOD_ENERGY
        (*missile).clipmask = 2 | 0x80000000; // MASK_SHOT | CONTENTS_LIGHTSABER
        (*missile).splashDamage = BOWCASTER_SPLASH_DAMAGE;
        (*missile).splashRadius = BOWCASTER_SPLASH_RADIUS;

        // we don't want it to bounce
        (*missile).bounceCount = 0;
    }
}

/*
-------------------------
Mark1Dead_FireBlaster
- Shoot the left weapon, the multi-blaster
-------------------------
*/
pub extern "C" fn Mark1Dead_FireBlaster() {
    let mut muzzle1: [f32; 3] = [0.0; 3];
    let mut muzzle_dir: [f32; 3] = [0.0; 3];
    let mut missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let mut bolt: c_int;

    unsafe {
        bolt = (*NPC).genericBolt1;

        gi.G2API_GetBoltMatrix((*NPC).ghoul2, (*NPC).playerModel,
            bolt,
            &mut boltMatrix, (*NPC).currentAngles, (*NPC).currentOrigin, (cg.time != 0 ? cg.time : level.time),
            core::ptr::null_mut(), (*NPC).s.modelScale);

        gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 0, &mut muzzle1); // ORIGIN = 0
        gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 2, &mut muzzle_dir); // NEGATIVE_Y = 2

        G_PlayEffect("bryar/muzzle_flash\0".as_ptr() as *const c_char, &muzzle1, &muzzle_dir);

        missile = CreateMissile(&muzzle1, &muzzle_dir, 1600.0, 10000, NPC, 0);

        G_Sound(NPC, G_SoundIndex("sound/chars/mark1/misc/mark1_fire\0".as_ptr() as *const c_char));

        (*missile).classname = "bryar_proj\0".as_ptr() as *const c_char;
        (*missile).s.weapon = 1; // WP_BRYAR_PISTOL

        (*missile).damage = 1;
        (*missile).dflags = 1; // DAMAGE_DEATH_KNOCKBACK
        (*missile).methodOfDeath = 10; // MOD_ENERGY
        (*missile).clipmask = 2 | 0x80000000; // MASK_SHOT | CONTENTS_LIGHTSABER
    }
}

/*
-------------------------
Mark1_die
-------------------------
*/
pub extern "C" fn Mark1_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int) {
    /*
    int	anim;

    // Is he dead already?
    anim = self->client->ps.legsAnim;
    if (((anim==BOTH_DEATH1) || (anim==BOTH_DEATH2)) && (self->client->ps.torsoAnimTimer==0))
    {	// This is because self->health keeps getting zeroed out. HL_NONE acts as health in this case.
        self->locationDamage[HL_NONE] += damage;
        if (self->locationDamage[HL_NONE] > 50)
        {
            DeathFX(self);
            self->client->ps.eFlags |= EF_NODRAW;
            self->contents = CONTENTS_CORPSE;
            // G_FreeEntity( self ); // Is this safe?  I can't see why we'd mark it nodraw and then just leave it around??
            self->e_ThinkFunc = thinkF_G_FreeEntity;
            self->nextthink = level.time + FRAMETIME;
        }
        return;
    }
    */

    unsafe {
        G_Sound(self_, G_SoundIndex("sound/chars/mark1/misc/death%d.wav\0".as_ptr() as *const c_char)); // va() would be used in C

        // Choose a death anim
        if Q_irand(1, 10) > 5 {
            NPC_SetAnim(self_, 5, 71, 0x100 | 0x200); // SETANIM_BOTH=5, BOTH_DEATH2=71, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
        } else {
            NPC_SetAnim(self_, 5, 70, 0x100 | 0x200); // SETANIM_BOTH=5, BOTH_DEATH1=70, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
        }
    }
}

/*
-------------------------
Mark1_dying
-------------------------
*/
pub extern "C" fn Mark1_dying(self_: *mut gentity_t) {
    let mut num: c_int;
    let mut newBolt: c_int;

    unsafe {
        if (*self_).client->ps.torsoAnimTimer > 0 {
            if TIMER_Done(self_, "dyingExplosion\0".as_ptr() as *const c_char) != 0 {
                num = Q_irand(1, 3);

                // Find place to generate explosion
                if num == 1 {
                    num = Q_irand(8, 10);
                    newBolt = gi.G2API_AddBolt(&mut (*self_).ghoul2[(*self_).playerModel], va("*flash%d\0".as_ptr() as *const c_char, num));
                    NPC_Mark1_Part_Explode(self_, newBolt);
                } else {
                    num = Q_irand(1, 6);
                    newBolt = gi.G2API_AddBolt(&mut (*self_).ghoul2[(*self_).playerModel], va("*torso_tube%d\0".as_ptr() as *const c_char, num));
                    NPC_Mark1_Part_Explode(self_, newBolt);
                    gi.G2API_SetSurfaceOnOff(&mut (*self_).ghoul2[(*self_).playerModel], va("torso_tube%d\0".as_ptr() as *const c_char, num), TURN_OFF);
                }

                TIMER_Set(self_, "dyingExplosion\0".as_ptr() as *const c_char, Q_irand(300, 1000));
            }

            //		int		dir;
            //		vec3_t	right;

            // Shove to the side
            //		AngleVectors( self->client->renderInfo.eyeAngles, NULL, right, NULL );
            //		VectorMA( self->client->ps.velocity, -80, right, self->client->ps.velocity );

            // See which weapons are there
            // Randomly fire blaster
            if gi.G2API_GetSurfaceRenderStatus(&(*self_).ghoul2[(*self_).playerModel], "l_arm\0".as_ptr() as *const c_char) == 0 {	// Is the blaster still on the model?
                if Q_irand(1, 5) == 1 {
                    SaveNPCGlobals();
                    SetNPCGlobals(self_);
                    Mark1Dead_FireBlaster();
                    RestoreNPCGlobals();
                }
            }

            // Randomly fire rocket
            if gi.G2API_GetSurfaceRenderStatus(&(*self_).ghoul2[(*self_).playerModel], "r_arm\0".as_ptr() as *const c_char) == 0 {	// Is the rocket still on the model?
                if Q_irand(1, 10) == 1 {
                    SaveNPCGlobals();
                    SetNPCGlobals(self_);
                    Mark1Dead_FireRocket();
                    RestoreNPCGlobals();
                }
            }
        }
    }
}

/*
-------------------------
NPC_Mark1_Pain
- look at what was hit and see if it should be removed from the model.
-------------------------
*/
pub extern "C" fn NPC_Mark1_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, other: *mut gentity_t, point: *const [f32; 3], damage: c_int, mod_: c_int, hitLoc: c_int) {
    let mut newBolt: c_int;
    let mut i: c_int;
    let mut chance: c_int;

    unsafe {
        NPC_Pain(self_, inflictor, other, point, damage, mod_);

        G_Sound(self_, G_SoundIndex("sound/chars/mark1/misc/mark1_pain\0".as_ptr() as *const c_char));

        // Hit in the CHEST???
        if hitLoc == 4 { // HL_CHEST = 4
            chance = Q_irand(1, 4);

            if (chance == 1) && (damage > 5) {
                NPC_SetAnim(self_, 5, 35, 0x100 | 0x200); // SETANIM_BOTH=5, BOTH_PAIN1=35, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
            }
        }
        // Hit in the left arm?
        else if (hitLoc == 8) && ((*self_).locationDamage[8] > LEFT_ARM_HEALTH) { // HL_ARM_LT = 8
            if (*self_).locationDamage[hitLoc as usize] >= LEFT_ARM_HEALTH {	// Blow it up?
                newBolt = gi.G2API_AddBolt(&mut (*self_).ghoul2[(*self_).playerModel], "*flash3\0".as_ptr() as *const c_char);
                if newBolt != -1 {
                    NPC_Mark1_Part_Explode(self_, newBolt);
                }

                gi.G2API_SetSurfaceOnOff(&mut (*self_).ghoul2[(*self_).playerModel], "l_arm\0".as_ptr() as *const c_char, TURN_OFF);
            }
        }
        // Hit in the right arm?
        else if (hitLoc == 9) && ((*self_).locationDamage[9] > RIGHT_ARM_HEALTH) { // HL_ARM_RT = 9	// Blow it up?
            if (*self_).locationDamage[hitLoc as usize] >= RIGHT_ARM_HEALTH {
                newBolt = gi.G2API_AddBolt(&mut (*self_).ghoul2[(*self_).playerModel], "*flash4\0".as_ptr() as *const c_char);
                if newBolt != -1 {
                    //				G_PlayEffect( "small_chunks", self->playerModel, self->genericBolt2, self->s.number);
                    NPC_Mark1_Part_Explode(self_, newBolt);
                }

                gi.G2API_SetSurfaceOnOff(&mut (*self_).ghoul2[(*self_).playerModel], "r_arm\0".as_ptr() as *const c_char, TURN_OFF);
            }
        }
        // Check ammo pods
        else {
            i = 0;
            while i < 6 {
                if (hitLoc == 10 + i) && ((*self_).locationDamage[(10 + i) as usize] > AMMO_POD_HEALTH) {	// Blow it up?
                    if (*self_).locationDamage[hitLoc as usize] >= AMMO_POD_HEALTH {
                        newBolt = gi.G2API_AddBolt(&mut (*self_).ghoul2[(*self_).playerModel], va("*torso_tube%d\0".as_ptr() as *const c_char, (i + 1)));
                        if newBolt != -1 {
                            NPC_Mark1_Part_Explode(self_, newBolt);
                        }
                        gi.G2API_SetSurfaceOnOff(&mut (*self_).ghoul2[(*self_).playerModel], va("torso_tube%d\0".as_ptr() as *const c_char, (i + 1)), TURN_OFF);
                        NPC_SetAnim(self_, 5, 35, 0x100 | 0x200); // SETANIM_BOTH=5, BOTH_PAIN1=35, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
                        break;
                    }
                }
                i += 1;
            }
        }

        // Are both guns shot off?
        if (gi.G2API_GetSurfaceRenderStatus(&(*self_).ghoul2[(*self_).playerModel], "l_arm\0".as_ptr() as *const c_char) != 0) &&
           (gi.G2API_GetSurfaceRenderStatus(&(*self_).ghoul2[(*self_).playerModel], "r_arm\0".as_ptr() as *const c_char) != 0) {
            G_Damage(self_, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), (*self_).health, 0, 255); // MOD_UNKNOWN = 255
        }
    }
}

/*
-------------------------
Mark1_Hunt
- look for enemy.
-------------------------`
*/
pub extern "C" fn Mark1_Hunt() {

    unsafe {
        if (*NPCInfo).goalEntity == core::ptr::null_mut() {
            (*NPCInfo).goalEntity = (*NPC).enemy;
        }

        NPC_FaceEnemy(1); // qtrue = 1

        (*NPCInfo).combatMove = 1;
        NPC_MoveToGoal(1);
    }
}

/*
-------------------------
Mark1_FireBlaster
- Shoot the left weapon, the multi-blaster
-------------------------
*/
pub extern "C" fn Mark1_FireBlaster() {
    let mut muzzle1: [f32; 3] = [0.0; 3];
    let mut enemy_org1: [f32; 3] = [0.0; 3];
    let mut delta1: [f32; 3] = [0.0; 3];
    let mut angleToEnemy1: [f32; 3] = [0.0; 3];
    static mut forward: [f32; 3] = [0.0; 3];
    static mut vright: [f32; 3] = [0.0; 3];
    static mut up: [f32; 3] = [0.0; 3];
    static mut muzzle: [f32; 3] = [0.0; 3];
    let mut missile: *mut gentity_t;
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let mut bolt: c_int;

    unsafe {
        // Which muzzle to fire from?
        if ((*NPCInfo).localState <= 3) || ((*NPCInfo).localState == 7) { // LSTATE_FIRED0 = 3, LSTATE_FIRED4 = 7
            (*NPCInfo).localState = 4; // LSTATE_FIRED1
            bolt = (*NPC).genericBolt1;
        } else if (*NPCInfo).localState == 4 { // LSTATE_FIRED1
            (*NPCInfo).localState = 5; // LSTATE_FIRED2
            bolt = (*NPC).genericBolt2;
        } else if (*NPCInfo).localState == 5 { // LSTATE_FIRED2
            (*NPCInfo).localState = 6; // LSTATE_FIRED3
            bolt = (*NPC).genericBolt3;
        } else {
            (*NPCInfo).localState = 7; // LSTATE_FIRED4
            bolt = (*NPC).genericBolt4;
        }

        gi.G2API_GetBoltMatrix((*NPC).ghoul2, (*NPC).playerModel,
            bolt,
            &mut boltMatrix, (*NPC).currentAngles, (*NPC).currentOrigin, (cg.time != 0 ? cg.time : level.time),
            core::ptr::null_mut(), (*NPC).s.modelScale);

        gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 0, &mut muzzle1); // ORIGIN = 0

        if (*NPC).health != 0 {
            CalcEntitySpot((*NPC).enemy, 0, &mut enemy_org1); // SPOT_HEAD = 0
            VectorSubtract(&enemy_org1, &muzzle1, &mut delta1);
            vectoangles(&delta1, &mut angleToEnemy1);
            AngleVectors(&angleToEnemy1, &mut forward, &mut vright, &mut up);
        } else {
            AngleVectors(&(*NPC).currentAngles, &mut forward, &mut vright, &mut up);
        }

        G_PlayEffect("bryar/muzzle_flash\0".as_ptr() as *const c_char, &muzzle1, &forward);

        G_Sound(NPC, G_SoundIndex("sound/chars/mark1/misc/mark1_fire\0".as_ptr() as *const c_char));

        missile = CreateMissile(&muzzle1, &forward, 1600.0, 10000, NPC, 0);

        (*missile).classname = "bryar_proj\0".as_ptr() as *const c_char;
        (*missile).s.weapon = 1; // WP_BRYAR_PISTOL

        (*missile).damage = 1;
        (*missile).dflags = 1; // DAMAGE_DEATH_KNOCKBACK
        (*missile).methodOfDeath = 10; // MOD_ENERGY
        (*missile).clipmask = 2 | 0x80000000; // MASK_SHOT | CONTENTS_LIGHTSABER
    }
}

/*
-------------------------
Mark1_BlasterAttack
-------------------------
*/
pub extern "C" fn Mark1_BlasterAttack(advance: c_int) {
    let mut chance: c_int;

    unsafe {
        if TIMER_Done(NPC, "attackDelay\0".as_ptr() as *const c_char) != 0 {	// Attack?
            chance = Q_irand(1, 5);

            (*NPCInfo).burstCount += 1;

            if (*NPCInfo).burstCount < 3 {	// Too few shots this burst?
                chance = 2;				// Force it to keep firing.
            } else if (*NPCInfo).burstCount > 12 {	// Too many shots fired this burst?
                (*NPCInfo).burstCount = 0;
                chance = 1;				// Force it to stop firing.
            }

            // Stop firing.
            if chance == 1 {
                (*NPCInfo).burstCount = 0;
                TIMER_Set(NPC, "attackDelay\0".as_ptr() as *const c_char, Q_irand(1000, 3000));
                (*NPC).client->ps.torsoAnimTimer = 0;						// Just in case the firing anim is running.
            } else {
                if TIMER_Done(NPC, "attackDelay2\0".as_ptr() as *const c_char) != 0 {	// Can't be shooting every frame.
                    TIMER_Set(NPC, "attackDelay2\0".as_ptr() as *const c_char, Q_irand(50, 50));
                    Mark1_FireBlaster();
                    NPC_SetAnim(NPC, 5, 6, 0x100 | 0x200); // SETANIM_BOTH=5, BOTH_ATTACK1=6, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
                }
                return;
            }
        } else if advance != 0 {
            if (*NPC).client->ps.torsoAnim == 6 { // BOTH_ATTACK1 = 6
                (*NPC).client->ps.torsoAnimTimer = 0;						// Just in case the firing anim is running.
            }
            Mark1_Hunt();
        } else {	// Make sure he's not firing.
            if (*NPC).client->ps.torsoAnim == 6 { // BOTH_ATTACK1 = 6
                (*NPC).client->ps.torsoAnimTimer = 0;						// Just in case the firing anim is running.
            }
        }
    }
}

/*
-------------------------
Mark1_FireRocket
-------------------------
*/
pub extern "C" fn Mark1_FireRocket() {
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let mut muzzle1: [f32; 3] = [0.0; 3];
    let mut enemy_org1: [f32; 3] = [0.0; 3];
    let mut delta1: [f32; 3] = [0.0; 3];
    let mut angleToEnemy1: [f32; 3] = [0.0; 3];
    static mut forward: [f32; 3] = [0.0; 3];
    static mut vright: [f32; 3] = [0.0; 3];
    static mut up: [f32; 3] = [0.0; 3];

    let damage: c_int = 50;

    unsafe {
        gi.G2API_GetBoltMatrix((*NPC).ghoul2, (*NPC).playerModel,
            (*NPC).genericBolt5,
            &mut boltMatrix, (*NPC).currentAngles, (*NPC).currentOrigin, (cg.time != 0 ? cg.time : level.time),
            core::ptr::null_mut(), (*NPC).s.modelScale);

        gi.G2API_GiveMeVectorFromMatrix(boltMatrix, 0, &mut muzzle1); // ORIGIN = 0

        //	G_PlayEffect( "blaster/muzzle_flash", muzzle1 );

        CalcEntitySpot((*NPC).enemy, 0, &mut enemy_org1); // SPOT_HEAD = 0
        VectorSubtract(&enemy_org1, &muzzle1, &mut delta1);
        vectoangles(&delta1, &mut angleToEnemy1);
        AngleVectors(&angleToEnemy1, &mut forward, &mut vright, &mut up);

        G_Sound(NPC, G_SoundIndex("sound/chars/mark1/misc/mark1_fire\0".as_ptr() as *const c_char));

        let missile = CreateMissile(&muzzle1, &forward, BOWCASTER_VELOCITY as f32, 10000, NPC, 0);

        (*missile).classname = "bowcaster_proj\0".as_ptr() as *const c_char;
        (*missile).s.weapon = 6; // WP_BOWCASTER

        VectorSet(&mut (*missile).maxs, BOWCASTER_SIZE as f32, BOWCASTER_SIZE as f32, BOWCASTER_SIZE as f32);
        VectorScale(&(*missile).maxs, -1.0, &mut (*missile).mins);

        (*missile).damage = damage;
        (*missile).dflags = 1; // DAMAGE_DEATH_KNOCKBACK
        (*missile).methodOfDeath = 10; // MOD_ENERGY
        (*missile).clipmask = 2 | 0x80000000; // MASK_SHOT | CONTENTS_LIGHTSABER
        (*missile).splashDamage = BOWCASTER_SPLASH_DAMAGE;
        (*missile).splashRadius = BOWCASTER_SPLASH_RADIUS;

        // we don't want it to bounce
        (*missile).bounceCount = 0;
    }
}

/*
-------------------------
Mark1_RocketAttack
-------------------------
*/
pub extern "C" fn Mark1_RocketAttack(advance: c_int) {
    unsafe {
        if TIMER_Done(NPC, "attackDelay\0".as_ptr() as *const c_char) != 0 {	// Attack?
            TIMER_Set(NPC, "attackDelay\0".as_ptr() as *const c_char, Q_irand(1000, 3000));
            NPC_SetAnim(NPC, 6, 7, 0x100 | 0x200); // SETANIM_TORSO=6, BOTH_ATTACK2=7, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD
            Mark1_FireRocket();
        } else if advance != 0 {
            Mark1_Hunt();
        }
    }
}

/*
-------------------------
Mark1_AttackDecision
-------------------------
*/
pub extern "C" fn Mark1_AttackDecision() {
    let mut blasterTest: c_int;
    let mut rocketTest: c_int;

    unsafe {
        //randomly talk
        if TIMER_Done(NPC, "patrolNoise\0".as_ptr() as *const c_char) != 0 {
            if TIMER_Done(NPC, "angerNoise\0".as_ptr() as *const c_char) != 0 {
                //			G_Sound( NPC, G_SoundIndex(va("sound/chars/mark1/misc/talk%d.wav",	Q_irand(1, 4))));
                TIMER_Set(NPC, "patrolNoise\0".as_ptr() as *const c_char, Q_irand(4000, 10000));
            }
        }

        // Enemy is dead or he has no enemy.
        if ((*(*NPC).enemy).health < 1) || (NPC_CheckEnemyExt() == 0) {
            (*NPC).enemy = core::ptr::null_mut();
            return;
        }

        // Rate our distance to the target and visibility
        let distance: f32 = DistanceHorizontalSquared((*NPC).currentOrigin, (*(*NPC).enemy).currentOrigin);
        let distRate: c_int = if distance > (MIN_MELEE_RANGE_SQR as f32) { 1 } else { 0 }; // DIST_LONG=1, DIST_MELEE=0
        let visible: c_int = NPC_ClearLOS((*NPC).enemy);
        let advance: c_int = if distance > (MIN_DISTANCE_SQR as f32) { 1 } else { 0 };

        // If we cannot see our target, move to see it
        if (visible == 0) || (NPC_FaceEnemy(1) == 0) { // qtrue = 1
            Mark1_Hunt();
            return;
        }

        // See if the side weapons are there
        blasterTest = gi.G2API_GetSurfaceRenderStatus(&(*NPC).ghoul2[(*NPC).playerModel], "l_arm\0".as_ptr() as *const c_char);
        rocketTest = gi.G2API_GetSurfaceRenderStatus(&(*NPC).ghoul2[(*NPC).playerModel], "r_arm\0".as_ptr() as *const c_char);

        // It has both side weapons
        let mut distRate_mut = distRate;
        if (blasterTest == 0) && (rocketTest == 0) {
            ;	// So do nothing.
        } else if blasterTest != 0 {
            distRate_mut = 1; // DIST_LONG
        } else if rocketTest != 0 {
            distRate_mut = 0; // DIST_MELEE
        } else {	// It should never get here, but just in case
            (*NPC).health = 0;
            (*NPC).client->ps.stats[0] = 0; // STAT_HEALTH = 0
            GEntity_DieFunc(NPC, NPC, NPC, 100, 255); // MOD_UNKNOWN = 255
        }

        // We can see enemy so shoot him if timers let you.
        NPC_FaceEnemy(1); // qtrue = 1

        if distRate_mut == 0 { // DIST_MELEE
            Mark1_BlasterAttack(advance);
        } else if distRate_mut == 1 { // DIST_LONG
            Mark1_RocketAttack(advance);
        }
    }
}

/*
-------------------------
Mark1_Patrol
-------------------------
*/
pub extern "C" fn Mark1_Patrol() {
    unsafe {
        if NPC_CheckPlayerTeamStealth() != 0 {
            G_Sound(NPC, G_SoundIndex("sound/chars/mark1/misc/mark1_wakeup\0".as_ptr() as *const c_char));
            NPC_UpdateAngles(1, 1); // qtrue = 1
            return;
        }

        //If we have somewhere to go, then do that
        if (*NPC).enemy == core::ptr::null_mut() {
            if UpdateGoal() != 0 {
                ucmd.buttons |= 0x00000200; // BUTTON_WALKING = 0x00000200
                NPC_MoveToGoal(1); // qtrue = 1
                NPC_UpdateAngles(1, 1); // qtrue = 1
            }

            //randomly talk
            //		if (TIMER_Done(NPC,"patrolNoise"))
            //		{
            //			G_Sound( NPC, G_SoundIndex(va("sound/chars/mark1/misc/talk%d.wav",	Q_irand(1, 4))));
            //
            //			TIMER_Set( NPC, "patrolNoise", Q_irand( 2000, 4000 ) );
            //		}
        }
    }
}


/*
-------------------------
NPC_BSMark1_Default
-------------------------
*/
pub extern "C" fn NPC_BSMark1_Default() {
    //NPC->e_DieFunc = dieF_Mark1_die;

    unsafe {
        if (*NPC).enemy != core::ptr::null_mut() {
            (*NPCInfo).goalEntity = (*NPC).enemy;
            Mark1_AttackDecision();
        } else if ((*NPCInfo).scriptFlags & 0x00010000) != 0 { // SCF_LOOK_FOR_ENEMIES
            Mark1_Patrol();
        } else {
            Mark1_Idle();
        }
    }
}
