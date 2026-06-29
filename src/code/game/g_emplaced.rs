// leave this line at the top for all g_xxxx.cpp files...

use core::ffi::c_int;

// These are external types and functions from the game engine
// We don't fully define them here, just declare them for linking
extern "C" {
    // From g_local.h / game engine
    type gentity_t;
    type mdxaBone_t;
    type vec3_t;
    type trace_t;
    type usercmd_t;

    // Global state
    static mut cg: cg_t;
    static mut level: level_t;

    // Game interface
    static mut gi: game_import_t;

    // External functions
    fn G_IsRidingVehicle(pEnt: *mut gentity_t) -> *mut gentity_t;
    fn VectorCopy(from: *const f32, to: *mut f32);
    fn VectorSet(v: *mut f32, x: f32, y: f32, z: f32);
    fn VectorClear(v: *mut f32);
    fn VectorAdd(a: *const f32, b: *const f32, c: *mut f32);
    fn VectorSubtract(a: *const f32, b: *const f32, c: *mut f32);
    fn VectorNormalize(v: *mut f32) -> f32;
    fn VectorNormalize2(v: *const f32, out: *mut f32) -> f32;
    fn VectorMA(a: *const f32, scale: f32, b: *const f32, c: *mut f32);
    fn VectorCompare(a: *const f32, b: *const f32) -> i32;
    fn DotProduct(a: *const f32, b: *const f32) -> f32;
    fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);

    fn G_SetOrigin(ent: *mut gentity_t, origin: *const f32);
    fn G_SetAngles(ent: *mut gentity_t, angles: *const f32);
    fn G_Spawn() -> *mut gentity_t;
    fn G_FreeEntity(ent: *mut gentity_t);
    fn G_PlayEffect(effect: *const u8, origin: *const f32);
    fn G_EffectIndex(name: *const u8) -> c_int;
    fn G_SoundIndex(name: *const u8) -> c_int;
    fn G_Sound(ent: *mut gentity_t, index: c_int);
    fn G_ModelIndex(name: *const u8) -> c_int;
    fn G_RadiusDamage(
        origin: *const f32,
        inflictor: *mut gentity_t,
        damage: c_int,
        radius: c_int,
        ignore: *mut gentity_t,
        mod_: c_int,
    );
    fn G_UseTargets(ent: *mut gentity_t, activator: *mut gentity_t);
    fn G_UseTargets2(ent: *mut gentity_t, activator: *mut gentity_t, target: *const u8);
    fn G_ActivateBehavior(ent: *mut gentity_t, bset: c_int);
    fn G_SpawnInt(key: *const u8, defvalue: *const u8, value: *mut c_int);
    fn G_SpawnFloat(key: *const u8, defvalue: *const u8, value: *mut f32);
    fn G_RemoveWeaponModels(ent: *mut gentity_t);
    fn G_CreateG2AttachedWeaponModel(ent: *mut gentity_t, model: *const u8, bolt: c_int, tag: c_int);
    fn RegisterItem(item: *mut c_int);
    fn FindItemForWeapon(weapon: c_int) -> *mut c_int;
    fn Add_Ammo(ent: *mut gentity_t, weapon: c_int, count: c_int);
    fn NPC_SetAnim(
        ent: *mut gentity_t,
        setanim_type: c_int,
        anim: c_int,
        flags: c_int,
    );
    fn SetClientViewAngle(ent: *mut gentity_t, angle: *const f32);
    fn ChangeWeapon(ent: *mut gentity_t, newWeapon: c_int);
    fn CG_ChangeWeapon(num: c_int);
    fn WP_SaberAddG2SaberModels(ent: *mut gentity_t);
    fn CG_CenterPrint(msg: *const u8, y: f32);
}

// Emplaced gun spawnflags
const EMPLACED_INACTIVE: c_int = 1;
const EMPLACED_FACING: c_int = 2;
const EMPLACED_VULNERABLE: c_int = 4;
const EWEB_INVULNERABLE: c_int = 4;
const EMPLACED_PLAYERUSE: c_int = 8;

// External constants and types not fully defined
const MAX_CLIENTS: c_int = 64;
const STEPSIZE: f32 = 18.0;
const BUTTON_USE: c_int = 1;
const BUTTON_ATTACK: c_int = 1;
const BUTTON_ALT_ATTACK: c_int = 2;

// External types we'll use as opaque pointers where needed
// These would be defined in headers we haven't ported yet
#[allow(non_camel_case_types)]
type client_t = core::ffi::c_void;
#[allow(non_camel_case_types)]
type playerState_t = core::ffi::c_void;
#[allow(non_camel_case_types)]
type cg_t = core::ffi::c_void;
#[allow(non_camel_case_types)]
type level_t = core::ffi::c_void;
#[allow(non_camel_case_types)]
type game_import_t = core::ffi::c_void;

// Phantom type markers for alignment/safety
struct ORIGIN;
struct NEGATIVE_Y;
struct ORIGIN_marker;
const ORIGIN: c_int = 0;
const NEGATIVE_Y: c_int = 1;

const BONE_ANGLES_POSTMULT: c_int = 0;
const POSITIVE_X: c_int = 1;
const POSITIVE_Y: c_int = 2;
const POSITIVE_Z: c_int = 4;
const NEGATIVE_X: c_int = 8;
const NEGATIVE_Y_CONST: c_int = 16;

const SETANIM_LEGS: c_int = 0;
const SETANIM_FLAG_OVERRIDE: c_int = 1;
const SETANIM_FLAG_HOLD: c_int = 2;

const BOTH_STRAFE_RIGHT1: c_int = 0;
const BOTH_STRAFE_LEFT1: c_int = 1;

const SVF_PLAYER_USABLE: c_int = 1;
const SVF_INACTIVE: c_int = 2;
const SVF_ANIMATING: c_int = 4;
const SVF_NONNPC_ENEMY: c_int = 8;

const CONTENTS_BODY: c_int = 1;
const CONTENTS_MONSTERCLIP: c_int = 2;
const CONTENTS_PLAYERCLIP: c_int = 4;

const FL_GODMODE: c_int = 1;

const STAT_HEALTH: c_int = 0;
const STAT_WEAPONS: c_int = 1;

const EF_LOCKED_TO_WEAPON: c_int = 1;

const PMF_DUCKED: c_int = 1;
const PMF_TIME_NOFRICTION: c_int = 2;
const PMF_TIME_KNOCKBACK: c_int = 4;

const MOD_UNKNOWN: c_int = 0;

const DAMAGE_CUSTOM_HUD: c_int = 1;

const WP_NONE: c_int = 0;
const WP_SABER: c_int = 1;
const WP_EMPLACED_GUN: c_int = 2;

const BSET_USE: c_int = 0;
const BSET_PAIN: c_int = 1;
const BSET_DEATH: c_int = 2;

const TURN_OFF: c_int = 0x00000100; // G2SURFACEFLAG_NODESCENDANTS

const YAW: usize = 0;
const PITCH: usize = 1;
const ROLL: usize = 2;

const S_COLOR_RED: &[u8] = b"^1";

const TEAM_FREE: c_int = 0;

const FF_CHANNEL_TOUCH: c_int = 0;

// Stub implementations for missing functions
fn crandom() -> f32 {
    // Placeholder for crandom() - returns random value between -1 and 1
    0.0
}

fn vtos(v: *const f32) -> *const u8 {
    b"<vector>\0".as_ptr()
}

fn rand() -> c_int {
    0
}

//lock the owner into place relative to the cannon pos
unsafe fn EWebPositionUser(owner: *mut gentity_t, eweb: *mut gentity_t) {
    let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
    let mut p: [f32; 3] = [0.0; 3];
    let mut p2: [f32; 3] = [0.0; 3];
    let mut d: [f32; 3] = [0.0; 3];
    let mut tr: trace_t = core::mem::zeroed();
    let mut traceOver: bool = true;

    if (*owner).s.number < MAX_CLIENTS {
        //extra checks
        gi.trace(
            &mut tr,
            (*owner).currentOrigin,
            (*owner).mins,
            (*owner).maxs,
            (*owner).currentOrigin,
            (*owner).s.number,
            (*owner).clipmask,
        );
        if tr.startsolid || tr.allsolid {
            //crap, they're already in solid somehow, don't bother tracing over
            traceOver = false;
        }
    }
    if traceOver {
        //trace up
        VectorCopy((*owner).currentOrigin, p2.as_mut_ptr());
        p2[2] += STEPSIZE;
        gi.trace(
            &mut tr,
            (*owner).currentOrigin,
            (*owner).mins,
            (*owner).maxs,
            p2.as_ptr(),
            (*owner).s.number,
            (*owner).clipmask,
        );
        if !tr.startsolid && !tr.allsolid {
            VectorCopy(tr.endpos, p2.as_mut_ptr());
        } else {
            VectorCopy((*owner).currentOrigin, p2.as_mut_ptr());
        }
    }
    //trace over
    gi.G2API_GetBoltMatrix(
        (*eweb).ghoul2,
        0,
        (*eweb).headBolt,
        &mut boltMatrix,
        (*eweb).s.apos.trBase,
        (*eweb).currentOrigin,
        if cg.time != 0 { cg.time } else { level.time },
        core::ptr::null_mut(),
        (*eweb).s.modelScale,
    );
    gi.G2API_GiveMeVectorFromMatrix(boltMatrix, ORIGIN, p.as_mut_ptr());
    gi.G2API_GiveMeVectorFromMatrix(boltMatrix, NEGATIVE_Y, d.as_mut_ptr());
    d[2] = 0.0;
    VectorNormalize(d.as_mut_ptr());
    VectorMA(p.as_ptr(), -44.0, d.as_ptr(), p.as_mut_ptr());
    if !traceOver {
        VectorCopy(p.as_ptr(), tr.endpos);
        tr.allsolid = false;
        tr.startsolid = false;
    } else {
        p[2] = p2[2];
        if (*owner).s.number < MAX_CLIENTS {
            //extra checks
            //just see if end point is not in solid
            gi.trace(
                &mut tr,
                p.as_ptr(),
                (*owner).mins,
                (*owner).maxs,
                p.as_ptr(),
                (*owner).s.number,
                (*owner).clipmask,
            );
            if tr.startsolid || tr.allsolid {
                //would be in solid there, so just trace over, I guess?
                gi.trace(
                    &mut tr,
                    p2.as_ptr(),
                    (*owner).mins,
                    (*owner).maxs,
                    p.as_ptr(),
                    (*owner).s.number,
                    (*owner).clipmask,
                );
            }
        } else {
            //trace over
            gi.trace(
                &mut tr,
                p2.as_ptr(),
                (*owner).mins,
                (*owner).maxs,
                p.as_ptr(),
                (*owner).s.number,
                (*owner).clipmask,
            );
        }
    }
    if !tr.startsolid && !tr.allsolid {
        //trace down
        VectorCopy(tr.endpos, p.as_mut_ptr());
        VectorCopy(p.as_ptr(), p2.as_mut_ptr());
        p2[2] -= STEPSIZE;
        gi.trace(
            &mut tr,
            p.as_ptr(),
            (*owner).mins,
            (*owner).maxs,
            p2.as_ptr(),
            (*owner).s.number,
            (*owner).clipmask,
        );

        if !tr.startsolid && !tr.allsolid {
            //&& tr.fraction == 1.0f)
            // all clear, we can move there
            let mut moveDir: [f32; 3] = [0.0; 3];
            let mut moveDist: f32 = 0.0;
            VectorCopy(tr.endpos, p.as_mut_ptr());
            VectorSubtract(p.as_ptr(), (*eweb).pos4, moveDir.as_mut_ptr());
            moveDist = VectorNormalize(moveDir.as_mut_ptr());
            if moveDist > 4.0 {
                //moved past the threshold from last position
                let mut oRight: [f32; 3] = [0.0; 3];
                let mut strafeAnim: c_int = 0;

                VectorCopy(p.as_ptr(), (*eweb).pos4); //update the position
                //find out what direction he moved in
                AngleVectors(
                    (*owner).currentAngles,
                    core::ptr::null_mut(),
                    oRight.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
                if DotProduct(moveDir.as_ptr(), oRight.as_ptr()) > 0.0 {
                    //moved to his right, play right strafe
                    strafeAnim = BOTH_STRAFE_RIGHT1;
                } else {
                    //moved left, play left strafe
                    strafeAnim = BOTH_STRAFE_LEFT1;
                }
                NPC_SetAnim(
                    owner,
                    SETANIM_LEGS,
                    strafeAnim,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }

            G_SetOrigin(owner, p.as_ptr());
            VectorCopy(p.as_ptr(), (*(*owner).client).ps.origin);
            gi.linkentity(owner);
        }
    }
    //FIXME: IK the hands to the handles of the gun?
}

//===============================================
//End E-Web
//===============================================

//----------------------------------------------------------

//===============================================
//Emplaced Gun
//===============================================

/*QUAKED emplaced_eweb (0 0 1) (-12 -12 -24) (12 12 24) INACTIVE FACING INVULNERABLE PLAYERUSE

 INACTIVE cannot be used until used by a target_activate
 FACING - player must be facing relatively in the same direction as the gun in order to use it
 VULNERABLE - allow the gun to take damage
 PLAYERUSE - only the player makes it run its usescript

 count - how much ammo to give this gun ( default 999 )
 health - how much damage the gun can take before it blows ( default 250 )
 delay - ONLY AFFECTS NPCs - time between shots ( default 200 on hardest setting )
 wait - ONLY AFFECTS NPCs - time between bursts ( default 800 on hardest setting )
 splashdamage - how much damage a blowing up gun deals ( default 80 )
 splashradius - radius for exploding damage ( default 128 )

 scripts:
	will run usescript, painscript and deathscript
*/
//----------------------------------------------------------
unsafe fn eweb_pain(
    slf: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _point: *const f32,
    _damage: c_int,
    _mod_: c_int,
    _hitLoc: c_int,
) {
    if (*slf).health <= 0 {
        // play pain effect?
    } else {
        if !(*slf).paintarget.is_null() {
            G_UseTargets2(slf, (*slf).activator, (*slf).paintarget);
        }

        // Don't do script if dead
        G_ActivateBehavior(slf, BSET_PAIN);
    }
}
//----------------------------------------------------------
unsafe fn eweb_die(
    slf: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
    _dFlags: c_int,
    _hitLoc: c_int,
) {
    let mut org: [f32; 3] = [0.0; 3];

    // turn off any firing animations it may have been doing
    (*slf).s.frame = 0;
    (*slf).startFrame = 0;
    (*slf).endFrame = 0;
    (*slf).svFlags &= !(SVF_ANIMATING | SVF_PLAYER_USABLE);

    (*slf).health = 0;
    //	(*slf).s.weapon = WP_EMPLACED_GUN; // we need to be able to switch back to the old weapon

    (*slf).takedamage = false;
    (*slf).lastEnemy = attacker;

    if !(*slf).activator.is_null() && !(*(*slf).activator).client.is_null() {
        if !(*(*slf).activator).NPC.is_null() {
            let mut right: [f32; 3] = [0.0; 3];

            // radius damage seems to throw them, but add an extra bit to throw them away from the weapon
            AngleVectors(
                (*slf).currentAngles,
                core::ptr::null_mut(),
                right.as_mut_ptr(),
                core::ptr::null_mut(),
            );
            VectorMA(
                (*(*(*slf).activator).client).ps.velocity,
                140.0,
                right.as_ptr(),
                (*(*(*slf).activator).client).ps.velocity,
            );
            (*(*(*slf).activator).client).ps.velocity[2] = -100.0;

            // kill them
            (*(*slf).activator).health = 0;
            (*(*(*(*slf).activator).client).ps.stats)[STAT_HEALTH as usize] = 0;
        }

        // kill the players emplaced ammo, cheesy way to keep the gun from firing
        (*(*(*(*slf).activator).client).ps.ammo)[weaponData[WP_EMPLACED_GUN as usize].ammoIndex]
            = 0;
    }

    (*slf).e_PainFunc = core::ptr::null_mut();

    if !(*slf).target.is_null() {
        G_UseTargets(slf, attacker);
    }

    G_RadiusDamage(
        (*slf).currentOrigin,
        slf,
        (*slf).splashDamage,
        (*slf).splashRadius,
        slf,
        MOD_UNKNOWN,
    );

    VectorCopy((*slf).currentOrigin, org.as_mut_ptr());
    org[2] += 20.0;

    G_PlayEffect(b"emplaced/explode\0".as_ptr(), org.as_ptr());

    // Turn the top of the eweb off.
    gi.G2API_SetSurfaceOnOff(
        &mut (*slf).ghoul2[(*slf).playerModel as usize],
        b"eweb_damage\0".as_ptr(),
        TURN_OFF,
    );

    // create some persistent smoke by using a dynamically created fx runner
    let ent = G_Spawn();

    if !ent.is_null() {
        (*ent).delay = 200;
        (*ent).random = 100.0;

        (*ent).fxID = G_EffectIndex(b"emplaced/dead_smoke\0".as_ptr());

        (*ent).e_ThinkFunc = Some(fx_runner_think);
        (*ent).nextthink = level.time + 50;

        // move up above the gun origin
        VectorCopy((*slf).currentOrigin, org.as_mut_ptr());
        org[2] += 35.0;
        G_SetOrigin(ent, org.as_ptr());
        VectorCopy(org.as_ptr(), (*ent).s.origin);

        VectorSet((*ent).s.angles, -90.0, 0.0, 0.0); // up
        G_SetAngles(ent, (*ent).s.angles);

        gi.linkentity(ent);
    }

    G_ActivateBehavior(slf, BSET_DEATH);
}

unsafe fn eweb_can_be_used(
    slf: *mut gentity_t,
    other: *mut gentity_t,
    activator: *mut gentity_t,
) -> bool {
    if (*slf).health <= 0 {
        // can't use a dead gun.
        return false;
    }

    if (*slf).svFlags & SVF_INACTIVE != 0 {
        return false; // can't use inactive gun
    }

    if (*activator).client.is_null() {
        return false; // only a client can use it.
    }

    if !(*slf).activator.is_null() {
        // someone is already in the gun.
        return false;
    }

    if !other.is_null()
        && !(*other).client.is_null()
        && !G_IsRidingVehicle(other).is_null()
    {
        //can't use eweb when on a vehicle
        return false;
    }

    if !activator.is_null()
        && !(*activator).client.is_null()
        && !G_IsRidingVehicle(activator).is_null()
    {
        //can't use eweb when on a vehicle
        return false;
    }

    if !activator.is_null()
        && !(*activator).client.is_null()
        && ((*(*activator).client).ps.pm_flags & PMF_DUCKED) != 0
    {
        //stand up, ya cowardly varmint!
        return false;
    }

    if !activator.is_null() && (*activator).health <= 0 {
        //dead men ain't got no more use fer guns...
        return false;
    }

    let mut fwd1: [f32; 3] = [0.0; 3];
    let mut fwd2: [f32; 3] = [0.0; 3];
    let mut facingAngles: [f32; 3] = [0.0; 3];

    VectorAdd((*slf).s.angles, (*slf).pos1, facingAngles.as_mut_ptr());
    if (*activator).s.number < MAX_CLIENTS {
        //player must be facing general direction of the turret head
        // Let's get some direction vectors for the users
        AngleVectors(
            (*(*activator).client).ps.viewangles,
            fwd1.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        fwd1[2] = 0.0;

        // Get the gun's direction vector
        AngleVectors(
            facingAngles.as_ptr(),
            fwd2.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        fwd2[2] = 0.0;

        let dot = DotProduct(fwd1.as_ptr(), fwd2.as_ptr());

        // Must be reasonably facing the way the gun points ( 90 degrees or so ), otherwise we don't allow to use it.
        if dot < 0.75 {
            return false;
        }
    }

    if (*slf).delay + 500 < level.time {
        return true;
    }
    false
}

unsafe fn eweb_use(slf: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    if !eweb_can_be_used(slf, other, activator) {
        return;
    }

    let oldWeapon = (*activator).s.weapon;

    if oldWeapon == WP_SABER {
        (*slf).alt_fire = (*(*activator).client).ps.SaberActive();
    }

    // swap the users weapon with the emplaced gun and add the ammo the gun has to the player
    (*activator).client = &mut (*activator).client as *mut _;
    (*(*activator).client).ps.weapon = (*slf).s.weapon;
    Add_Ammo(activator, WP_EMPLACED_GUN, (*slf).count);
    (*(*(*activator).client).ps.stats)[STAT_WEAPONS as usize] |= (1 << WP_EMPLACED_GUN);

    // Allow us to point from one to the other
    (*activator).owner = slf; // kind of dumb, but when we are locked to the weapon, we are owned by it.
    (*slf).activator = activator;

    G_RemoveWeaponModels(activator);

    if !(*activator).NPC.is_null() {
        ChangeWeapon(activator, WP_EMPLACED_GUN);
    } else if (*activator).s.number == 0 {
        // we don't want for it to draw the weapon select stuff
        cg.weaponSelect = WP_EMPLACED_GUN;
        CG_CenterPrint(b"@SP_INGAME_EXIT_VIEW\0".as_ptr(), 570.0);
    }

    VectorCopy(
        (*activator).currentOrigin,
        (*slf).pos4,
    ); //keep this around so we know when to make them play the strafe anim

    // the gun will track which weapon we used to have
    (*slf).s.weapon = oldWeapon;

    // Lock the player
    (*(*activator).client).ps.eFlags |= EF_LOCKED_TO_WEAPON;
    (*activator).owner = slf; // kind of dumb, but when we are locked to the weapon, we are owned by it.
    (*slf).activator = activator;
    (*slf).delay = level.time; // can't disconnect from the thing for half a second

    // Let the gun be considered an enemy
    //Ugh, so much AI code seems to assume enemies are clients, maybe this shouldn't be on, but it's too late in the game to change it now without knowing what side-effects this will have
    (*slf).svFlags |= SVF_NONNPC_ENEMY;
    (*slf).noDamageTeam = (*(*activator).client).playerTeam;

    //FIXME: should really wait a bit after spawn and get this just once?
    (*slf).waypoint = NAV::GetNearestNode(slf);
    #[cfg(debug_assertions)]
    {
        if (*slf).waypoint == -1 {
            gi.Printf(
                b"%sERROR: no waypoint for emplaced_gun %s at %s\n\0".as_ptr() as *const u8,
                S_COLOR_RED.as_ptr(),
                (*slf).targetname,
                vtos((*slf).currentOrigin),
            );
        }
    }

    G_Sound(slf, G_SoundIndex(b"sound/weapons/eweb/eweb_mount.mp3\0".as_ptr()));
    #[cfg(feature = "_IMMERSION")]
    {
        G_Force(
            slf,
            G_ForceIndex(
                b"fffx/weapons/emplaced/emplaced_mount\0".as_ptr(),
                FF_CHANNEL_TOUCH,
            ),
        );
    }

    if ((*slf).spawnflags & EMPLACED_PLAYERUSE) == 0 || (*activator).s.number == 0 {
        //player-only usescript or any usescript
        // Run use script
        G_ActivateBehavior(slf, BSET_USE);
    }
}

//----------------------------------------------------------
unsafe fn SP_emplaced_eweb(ent: *mut gentity_t) {
    let name = b"models/map_objects/hoth/eweb_model.glm\0";

    (*ent).svFlags |= SVF_PLAYER_USABLE;
    (*ent).contents = CONTENTS_BODY;

    if (*ent).spawnflags & EMPLACED_INACTIVE != 0 {
        (*ent).svFlags |= SVF_INACTIVE;
    }

    VectorSet((*ent).mins, -12.0, -12.0, -24.0);
    VectorSet((*ent).maxs, 12.0, 12.0, 24.0);

    (*ent).takedamage = true;

    if ((*ent).spawnflags & EWEB_INVULNERABLE) != 0 {
        (*ent).flags |= FL_GODMODE;
    }

    (*ent).s.radius = 80;
    (*ent).spawnflags |= 4; // deadsolid

    //(*ent).e_ThinkFunc = None;
    (*ent).e_PainFunc = Some(eweb_pain as _);
    (*ent).e_DieFunc = Some(eweb_die as _);

    G_EffectIndex(b"emplaced/explode\0".as_ptr());
    G_EffectIndex(b"emplaced/dead_smoke\0".as_ptr());

    G_SoundIndex(b"sound/weapons/eweb/eweb_aim.wav\0".as_ptr());
    G_SoundIndex(b"sound/weapons/eweb/eweb_dismount.mp3\0".as_ptr());
    //G_SoundIndex( b"sound/weapons/eweb/eweb_empty.wav\0".as_ptr());
    G_SoundIndex(b"sound/weapons/eweb/eweb_fire.wav\0".as_ptr());
    G_SoundIndex(b"sound/weapons/eweb/eweb_hitplayer.wav\0".as_ptr());
    G_SoundIndex(b"sound/weapons/eweb/eweb_hitsurface.wav\0".as_ptr());
    //G_SoundIndex( b"sound/weapons/eweb/eweb_load.wav\0".as_ptr());
    G_SoundIndex(b"sound/weapons/eweb/eweb_mount.mp3\0".as_ptr());

    // Set up our defaults and override with custom amounts as necessary
    G_SpawnInt(b"count\0".as_ptr(), b"999\0".as_ptr(), &mut (*ent).count);
    G_SpawnInt(b"health\0".as_ptr(), b"250\0".as_ptr(), &mut (*ent).health);
    G_SpawnInt(b"splashDamage\0".as_ptr(), b"40\0".as_ptr(), &mut (*ent).splashDamage);
    G_SpawnInt(b"splashRadius\0".as_ptr(), b"100\0".as_ptr(), &mut (*ent).splashRadius);
    G_SpawnFloat(b"delay\0".as_ptr(), b"200\0".as_ptr(), &mut (*ent).random); // NOTE: spawning into a different field!!
    G_SpawnFloat(b"wait\0".as_ptr(), b"800\0".as_ptr(), &mut (*ent).wait);

    (*ent).max_health = (*ent).health;
    (*ent).dflags |= DAMAGE_CUSTOM_HUD; // dumb, but we draw a custom hud

    (*ent).s.modelindex = G_ModelIndex(name.as_ptr());
    (*ent).playerModel =
        gi.G2API_InitGhoul2Model((*ent).ghoul2, name.as_ptr(), (*ent).s.modelindex);

    // Activate our tags and bones
    (*ent).handLBolt = gi.G2API_AddBolt(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"*cannonflash\0".as_ptr()); //muzzle bolt
    (*ent).headBolt = gi.G2API_AddBolt(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"cannon_Xrot\0".as_ptr()); //for placing the owner relative to rotation
    (*ent).rootBone = gi.G2API_GetBoneIndex(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"model_root\0".as_ptr(), true);
    (*ent).lowerLumbarBone = gi.G2API_GetBoneIndex(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"cannon_Yrot\0".as_ptr(), true);
    (*ent).upperLumbarBone = gi.G2API_GetBoneIndex(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"cannon_Xrot\0".as_ptr(), true);
    gi.G2API_SetBoneAnglesIndex(
        &mut (*ent).ghoul2[(*ent).playerModel as usize],
        (*ent).lowerLumbarBone,
        vec3_origin,
        BONE_ANGLES_POSTMULT,
        POSITIVE_Z,
        NEGATIVE_X,
        NEGATIVE_Y_CONST,
        core::ptr::null_mut(),
    );
    gi.G2API_SetBoneAnglesIndex(
        &mut (*ent).ghoul2[(*ent).playerModel as usize],
        (*ent).upperLumbarBone,
        vec3_origin,
        BONE_ANGLES_POSTMULT,
        POSITIVE_Z,
        NEGATIVE_X,
        NEGATIVE_Y_CONST,
        core::ptr::null_mut(),
    );
    //gi.G2API_SetBoneAngles( &(*ent).ghoul2[0], b"cannon_Yrot\0".as_ptr(), vec3_origin, BONE_ANGLES_POSTMULT, POSITIVE_Y, POSITIVE_Z, POSITIVE_X, NULL);
    //set the constraints for this guy as an emplaced weapon, and his constraint angles
    //(*ent).s.origin2[0] = 60.0f; //60 degrees in either direction

    RegisterItem(FindItemForWeapon(WP_EMPLACED_GUN));
    (*ent).s.weapon = WP_EMPLACED_GUN;

    G_SetOrigin(ent, (*ent).s.origin);
    G_SetAngles(ent, (*ent).s.angles);
    VectorCopy((*ent).s.angles, (*ent).lastAngles);

    // store base angles for later
    VectorClear((*ent).pos1);

    (*ent).e_UseFunc = Some(eweb_use as _);
    (*ent).bounceCount = 1; //to distinguish it from the emplaced gun

    gi.linkentity(ent);
}

/*QUAKED emplaced_gun (0 0 1) (-24 -24 0) (24 24 64) INACTIVE x VULNERABLE PLAYERUSE

 INACTIVE cannot be used until used by a target_activate
 VULNERABLE - allow the gun to take damage
 PLAYERUSE - only the player makes it run its usescript

 count - how much ammo to give this gun ( default 999 )
 health - how much damage the gun can take before it blows ( default 250 )
 delay - ONLY AFFECTS NPCs - time between shots ( default 200 on hardest setting )
 wait - ONLY AFFECTS NPCs - time between bursts ( default 800 on hardest setting )
 splashdamage - how much damage a blowing up gun deals ( default 80 )
 splashradius - radius for exploding damage ( default 128 )

 scripts:
	will run usescript, painscript and deathscript
*/

//----------------------------------------------------------
unsafe fn emplaced_gun_use(slf: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    let mut fwd1: [f32; 3] = [0.0; 3];
    let mut fwd2: [f32; 3] = [0.0; 3];

    if (*slf).health <= 0 {
        // can't use a dead gun.
        return;
    }

    if (*slf).svFlags & SVF_INACTIVE != 0 {
        return; // can't use inactive gun
    }

    if (*activator).client.is_null() {
        return; // only a client can use it.
    }

    if !(*slf).activator.is_null() {
        // someone is already in the gun.
        return;
    }

    if !other.is_null()
        && !(*other).client.is_null()
        && !G_IsRidingVehicle(other).is_null()
    {
        //can't use eweb when on a vehicle
        return;
    }

    if !activator.is_null()
        && !(*activator).client.is_null()
        && !G_IsRidingVehicle(activator).is_null()
    {
        //can't use eweb when on a vehicle
        return;
    }

    // We'll just let the designers duke this one out....I mean, as to whether they even want to limit such a thing.
    if (*slf).spawnflags & EMPLACED_FACING != 0 {
        // Let's get some direction vectors for the users
        AngleVectors(
            (*(*activator).client).ps.viewangles,
            fwd1.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        // Get the guns direction vector
        AngleVectors(
            (*slf).pos1,
            fwd2.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        let dot = DotProduct(fwd1.as_ptr(), fwd2.as_ptr());

        // Must be reasonably facing the way the gun points ( 90 degrees or so ), otherwise we don't allow to use it.
        if dot < 0.0 {
            return;
        }
    }

    // don't allow using it again for half a second
    if (*slf).delay + 500 < level.time {
        let oldWeapon = (*activator).s.weapon;

        if oldWeapon == WP_SABER {
            (*slf).alt_fire = (*(*activator).client).ps.SaberActive();
        }

        // swap the users weapon with the emplaced gun and add the ammo the gun has to the player
        (*(*activator).client).ps.weapon = (*slf).s.weapon;
        Add_Ammo(activator, WP_EMPLACED_GUN, (*slf).count);
        (*(*(*activator).client).ps.stats)[STAT_WEAPONS as usize] |= (1 << WP_EMPLACED_GUN);

        // Allow us to point from one to the other
        (*activator).owner = slf; // kind of dumb, but when we are locked to the weapon, we are owned by it.
        (*slf).activator = activator;

        G_RemoveWeaponModels(activator);

        if !(*activator).NPC.is_null() {
            ChangeWeapon(activator, WP_EMPLACED_GUN);
        } else if (*activator).s.number == 0 {
            // we don't want for it to draw the weapon select stuff
            cg.weaponSelect = WP_EMPLACED_GUN;
            CG_CenterPrint(b"@SP_INGAME_EXIT_VIEW\0".as_ptr(), 570.0);
        }
        // Since we move the activator inside of the gun, we reserve a solid spot where they were standing in order to be able to get back out without being in solid
        if !(*slf).nextTrain.is_null() {
            //you never know
            G_FreeEntity((*slf).nextTrain);
        }
        (*slf).nextTrain = G_Spawn();
        //(*slf).nextTrain->classname = b"emp_placeholder\0";
        (*(*slf).nextTrain).contents = CONTENTS_MONSTERCLIP | CONTENTS_PLAYERCLIP; //hmm... playerclip too now that we're doing it for NPCs?
        G_SetOrigin((*slf).nextTrain, (*(*activator).client).ps.origin);
        VectorCopy((*activator).mins, (*(*slf).nextTrain).mins);
        VectorCopy((*activator).maxs, (*(*slf).nextTrain).maxs);
        gi.linkentity((*slf).nextTrain);

        //need to inflate the activator's mins/maxs since the gunsit anim puts them outside of their bbox
        VectorSet((*activator).mins, -24.0, -24.0, -24.0);
        VectorSet((*activator).maxs, 24.0, 24.0, 40.0);

        // Move the activator into the center of the gun.  For NPC's the only way the can get out of the gun is to die.
        VectorCopy((*slf).s.origin, (*(*activator).client).ps.origin);
        (*(*activator).client).ps.origin[2] += 30.0; // move them up so they aren't standing in the floor
        gi.linkentity(activator);

        // the gun will track which weapon we used to have
        (*slf).s.weapon = oldWeapon;

        // Lock the player
        (*(*activator).client).ps.eFlags |= EF_LOCKED_TO_WEAPON;
        (*activator).owner = slf; // kind of dumb, but when we are locked to the weapon, we are owned by it.
        (*slf).activator = activator;
        (*slf).delay = level.time; // can't disconnect from the thing for half a second

        // Let the gun be considered an enemy
        //Ugh, so much AI code seems to assume enemies are clients, maybe this shouldn't be on, but it's too late in the game to change it now without knowing what side-effects this will have
        (*slf).svFlags |= SVF_NONNPC_ENEMY;
        (*slf).noDamageTeam = (*(*activator).client).playerTeam;

        // FIXME: don't do this, we'll try and actually put the player in this beast
        // move the player to the center of the gun
        //		(*activator).contents = 0;
        //		VectorCopy( (*slf).currentOrigin, (*(*activator).client).ps.origin );

        SetClientViewAngle(activator, (*slf).pos1);

        //FIXME: should really wait a bit after spawn and get this just once?
        (*slf).waypoint = NAV::GetNearestNode(slf);
        #[cfg(debug_assertions)]
        {
            if (*slf).waypoint == -1 {
                gi.Printf(
                    b"%sERROR: no waypoint for emplaced_gun %s at %s\n\0".as_ptr() as *const u8,
                    S_COLOR_RED.as_ptr(),
                    (*slf).targetname,
                    vtos((*slf).currentOrigin),
                );
            }
        }

        G_Sound(slf, G_SoundIndex(b"sound/weapons/emplaced/emplaced_mount.mp3\0".as_ptr()));
        #[cfg(feature = "_IMMERSION")]
        {
            G_Force(
                slf,
                G_ForceIndex(
                    b"fffx/weapons/emplaced/emplaced_mount\0".as_ptr(),
                    FF_CHANNEL_TOUCH,
                ),
            );
        }

        if ((*slf).spawnflags & EMPLACED_PLAYERUSE) == 0 || (*activator).s.number == 0 {
            //player-only usescript or any usescript
            // Run use script
            G_ActivateBehavior(slf, BSET_USE);
        }
    }
}

//----------------------------------------------------------
unsafe fn emplaced_gun_pain(
    slf: *mut gentity_t,
    _inflictor: *mut gentity_t,
    _attacker: *mut gentity_t,
    _point: *const f32,
    _damage: c_int,
    _mod_: c_int,
    _hitLoc: c_int,
) {
    if (*slf).health <= 0 {
        // play pain effect?
    } else {
        if !(*slf).paintarget.is_null() {
            G_UseTargets2(slf, (*slf).activator, (*slf).paintarget);
        }

        // Don't do script if dead
        G_ActivateBehavior(slf, BSET_PAIN);
    }
}

//----------------------------------------------------------
unsafe fn emplaced_blow(ent: *mut gentity_t) {
    (*ent).e_DieFunc = None;
    emplaced_gun_die(ent, (*ent).lastEnemy, (*ent).lastEnemy, 0, MOD_UNKNOWN, 0, 0);
}

//----------------------------------------------------------
unsafe fn emplaced_gun_die(
    slf: *mut gentity_t,
    _inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    _damage: c_int,
    _mod_: c_int,
    _dFlags: c_int,
    _hitLoc: c_int,
) {
    let mut org: [f32; 3] = [0.0; 3];

    // turn off any firing animations it may have been doing
    (*slf).s.frame = 0;
    (*slf).startFrame = 0;
    (*slf).endFrame = 0;
    (*slf).svFlags &= !SVF_ANIMATING;

    (*slf).health = 0;
    //	(*slf).s.weapon = WP_EMPLACED_GUN; // we need to be able to switch back to the old weapon

    (*slf).takedamage = false;
    (*slf).lastEnemy = attacker;

    // we defer explosion so the player has time to get out
    if (*slf).e_DieFunc.is_some() {
        (*slf).e_ThinkFunc = Some(emplaced_blow as _);
        (*slf).nextthink = level.time + 3000; // don't blow for a couple of seconds
        return;
    }

    if !(*slf).activator.is_null() && !(*(*slf).activator).client.is_null() {
        if !(*(*slf).activator).NPC.is_null() {
            let mut right: [f32; 3] = [0.0; 3];

            // radius damage seems to throw them, but add an extra bit to throw them away from the weapon
            AngleVectors(
                (*slf).currentAngles,
                core::ptr::null_mut(),
                right.as_mut_ptr(),
                core::ptr::null_mut(),
            );
            VectorMA(
                (*(*(*slf).activator).client).ps.velocity,
                140.0,
                right.as_ptr(),
                (*(*(*slf).activator).client).ps.velocity,
            );
            (*(*(*slf).activator).client).ps.velocity[2] = -100.0;

            // kill them
            (*(*slf).activator).health = 0;
            (*(*(*(*slf).activator).client).ps.stats)[STAT_HEALTH as usize] = 0;
        }

        // kill the players emplaced ammo, cheesy way to keep the gun from firing
        (*(*(*(*slf).activator).client).ps.ammo)[weaponData[WP_EMPLACED_GUN as usize].ammoIndex]
            = 0;
    }

    (*slf).e_PainFunc = None;
    (*slf).e_ThinkFunc = None;

    if !(*slf).target.is_null() {
        G_UseTargets(slf, attacker);
    }

    G_RadiusDamage(
        (*slf).currentOrigin,
        slf,
        (*slf).splashDamage,
        (*slf).splashRadius,
        slf,
        MOD_UNKNOWN,
    );

    // when the gun is dead, add some ugliness to it.
    let mut ugly: [f32; 3] = [0.0; 3];

    ugly[YAW] = 4.0;
    ugly[PITCH] = (*slf).lastAngles[PITCH] * 0.8 + crandom() * 6.0;
    ugly[ROLL] = crandom() * 7.0;
    gi.G2API_SetBoneAnglesIndex(
        &mut (*slf).ghoul2[(*slf).playerModel as usize],
        (*slf).lowerLumbarBone,
        ugly.as_ptr(),
        BONE_ANGLES_POSTMULT,
        POSITIVE_Y,
        POSITIVE_Z,
        POSITIVE_X,
        core::ptr::null_mut(),
    );

    VectorCopy((*slf).currentOrigin, org.as_mut_ptr());
    org[2] += 20.0;

    G_PlayEffect(b"emplaced/explode\0".as_ptr(), org.as_ptr());

    // create some persistent smoke by using a dynamically created fx runner
    let ent = G_Spawn();

    if !ent.is_null() {
        (*ent).delay = 200;
        (*ent).random = 100.0;

        (*ent).fxID = G_EffectIndex(b"emplaced/dead_smoke\0".as_ptr());

        (*ent).e_ThinkFunc = Some(fx_runner_think);
        (*ent).nextthink = level.time + 50;

        // move up above the gun origin
        VectorCopy((*slf).currentOrigin, org.as_mut_ptr());
        org[2] += 35.0;
        G_SetOrigin(ent, org.as_ptr());
        VectorCopy(org.as_ptr(), (*ent).s.origin);

        VectorSet((*ent).s.angles, -90.0, 0.0, 0.0); // up
        G_SetAngles(ent, (*ent).s.angles);

        gi.linkentity(ent);
    }

    G_ActivateBehavior(slf, BSET_DEATH);
}

//----------------------------------------------------------
unsafe fn SP_emplaced_gun(ent: *mut gentity_t) {
    let name = b"models/map_objects/imp_mine/turret_chair.glm\0";

    (*ent).svFlags |= SVF_PLAYER_USABLE;
    (*ent).contents = CONTENTS_BODY; //CONTENTS_SHOTCLIP|CONTENTS_PLAYERCLIP|CONTENTS_MONSTERCLIP;//CONTENTS_SOLID;

    if (*ent).spawnflags & EMPLACED_INACTIVE != 0 {
        (*ent).svFlags |= SVF_INACTIVE;
    }

    VectorSet((*ent).mins, -30.0, -30.0, -5.0);
    VectorSet((*ent).maxs, 30.0, 30.0, 60.0);

    (*ent).takedamage = true;

    if ((*ent).spawnflags & EMPLACED_VULNERABLE) == 0 {
        (*ent).flags |= FL_GODMODE;
    }

    (*ent).s.radius = 110;
    (*ent).spawnflags |= 4; // deadsolid

    //(*ent).e_ThinkFunc = None;
    (*ent).e_PainFunc = Some(emplaced_gun_pain as _);
    (*ent).e_DieFunc = Some(emplaced_gun_die as _);

    G_EffectIndex(b"emplaced/explode\0".as_ptr());
    G_EffectIndex(b"emplaced/dead_smoke\0".as_ptr());

    G_SoundIndex(b"sound/weapons/emplaced/emplaced_mount.mp3\0".as_ptr());
    G_SoundIndex(b"sound/weapons/emplaced/emplaced_dismount.mp3\0".as_ptr());
    G_SoundIndex(b"sound/weapons/emplaced/emplaced_move_lp.wav\0".as_ptr());

    // Set up our defaults and override with custom amounts as necessary
    G_SpawnInt(b"count\0".as_ptr(), b"999\0".as_ptr(), &mut (*ent).count);
    G_SpawnInt(b"health\0".as_ptr(), b"250\0".as_ptr(), &mut (*ent).health);
    G_SpawnInt(b"splashDamage\0".as_ptr(), b"80\0".as_ptr(), &mut (*ent).splashDamage);
    G_SpawnInt(b"splashRadius\0".as_ptr(), b"128\0".as_ptr(), &mut (*ent).splashRadius);
    G_SpawnFloat(b"delay\0".as_ptr(), b"200\0".as_ptr(), &mut (*ent).random); // NOTE: spawning into a different field!!
    G_SpawnFloat(b"wait\0".as_ptr(), b"800\0".as_ptr(), &mut (*ent).wait);

    (*ent).max_health = (*ent).health;
    (*ent).dflags |= DAMAGE_CUSTOM_HUD; // dumb, but we draw a custom hud

    (*ent).s.modelindex = G_ModelIndex(name.as_ptr());
    (*ent).playerModel =
        gi.G2API_InitGhoul2Model((*ent).ghoul2, name.as_ptr(), (*ent).s.modelindex);

    // Activate our tags and bones
    (*ent).headBolt = gi.G2API_AddBolt(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"*seat\0".as_ptr());
    (*ent).handLBolt = gi.G2API_AddBolt(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"*flash01\0".as_ptr());
    (*ent).handRBolt = gi.G2API_AddBolt(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"*flash02\0".as_ptr());
    (*ent).rootBone = gi.G2API_GetBoneIndex(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"base_bone\0".as_ptr(), true);
    (*ent).lowerLumbarBone = gi.G2API_GetBoneIndex(&mut (*ent).ghoul2[(*ent).playerModel as usize], b"swivel_bone\0".as_ptr(), true);
    gi.G2API_SetBoneAnglesIndex(
        &mut (*ent).ghoul2[(*ent).playerModel as usize],
        (*ent).lowerLumbarBone,
        vec3_origin,
        BONE_ANGLES_POSTMULT,
        POSITIVE_Y,
        POSITIVE_Z,
        POSITIVE_X,
        core::ptr::null_mut(),
    );

    RegisterItem(FindItemForWeapon(WP_EMPLACED_GUN));
    (*ent).s.weapon = WP_EMPLACED_GUN;

    G_SetOrigin(ent, (*ent).s.origin);
    G_SetAngles(ent, (*ent).s.angles);
    VectorCopy((*ent).s.angles, (*ent).lastAngles);

    // store base angles for later
    VectorCopy((*ent).s.angles, (*ent).pos1);

    (*ent).e_UseFunc = Some(emplaced_gun_use as _);
    (*ent).bounceCount = 0; //to distinguish it from the eweb

    gi.linkentity(ent);
}

//====================================================
//General Emplaced Weapon Funcs called in g_active.cpp
//====================================================

unsafe fn G_UpdateEmplacedWeaponData(ent: *mut gentity_t) {
    if !ent.is_null() && !(*ent).owner.is_null() && (*ent).health > 0 {
        let chair = (*ent).owner;
        if (*chair).e_UseFunc == Some(emplaced_gun_use as _) {
            //yeah, crappy way to check this, but...
            //one that you sit in
            //take the emplaced gun's waypoint as your own
            (*ent).waypoint = (*chair).waypoint;

            //update the actual origin of the sitter
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut chairAng: [f32; 3] = [0.0, (*(*ent).client).ps.viewangles[YAW], 0.0];

            // Getting the seat bolt here
            gi.G2API_GetBoltMatrix(
                (*chair).ghoul2,
                (*chair).playerModel,
                (*chair).headBolt,
                &mut boltMatrix,
                chairAng.as_ptr(),
                (*chair).currentOrigin,
                if cg.time != 0 { cg.time } else { level.time },
                core::ptr::null_mut(),
                (*chair).s.modelScale,
            );
            // Storing ent position, bolt position, and bolt axis
            gi.G2API_GiveMeVectorFromMatrix(boltMatrix, ORIGIN, (*(*ent).client).ps.origin);
            gi.linkentity(ent);
        } else if (*chair).e_UseFunc == Some(eweb_use as _) {
            //yeah, crappy way to check this, but...
            //standing at an E-Web
            EWebPositionUser(ent, chair);
        }
    }
}

unsafe fn ExitEmplacedWeapon(ent: *mut gentity_t) {
    // requesting to unlock from the weapon
    // We'll leave the gun pointed in the direction it was last facing, though we'll cut out the pitch
    if !(*ent).client.is_null() {
        // if we are the player we will have put down a brush that blocks NPCs so that we have a clear spot to get back out.
        //gentity_t *place = G_Find( NULL, FOFS(classname), b"emp_placeholder\0".as_ptr());

        if (*ent).health > 0 {
            //he's still alive, and we have a placeholder, so put him back
            if !(*(*ent).owner).nextTrain.is_null() {
                // reset the players position
                VectorCopy(
                    (*(*(*ent).owner).nextTrain).currentOrigin,
                    (*(*ent).client).ps.origin,
                );
                //reset ent's size to normal
                VectorCopy(
                    (*(*(*ent).owner).nextTrain).mins,
                    (*ent).mins,
                );
                VectorCopy(
                    (*(*(*ent).owner).nextTrain).maxs,
                    (*ent).maxs,
                );
                //free the placeholder
                G_FreeEntity((*(*ent).owner).nextTrain);
                //re-link the ent
                gi.linkentity(ent);
            } else if (*(*ent).owner).e_UseFunc == Some(eweb_use as _) {
                //yeah, crappy way to check this, but...
                // so give 'em a push away from us
                let mut backDir: [f32; 3] = [0.0; 3];
                let mut start: [f32; 3] = [0.0; 3];
                let mut end: [f32; 3] = [0.0; 3];
                let mut trace: trace_t = core::mem::zeroed();
                let eweb = (*ent).owner;
                let mut curRadius: f32 = 0.0;
                let minRadius: f32;
                let mut maxRadius: f32;
                let mut safeExit: bool = false;

                VectorSubtract((*ent).currentOrigin, (*eweb).currentOrigin, backDir.as_mut_ptr());
                backDir[2] = 0.0;
                minRadius = VectorNormalize(backDir.as_mut_ptr()) - 8.0;

                maxRadius = ((*ent).maxs[0] + (*ent).maxs[1]) * 0.5;
                maxRadius += ((*eweb).maxs[0] + (*eweb).maxs[1]) * 0.5;
                maxRadius *= 1.5;

                if minRadius >= maxRadius - 1.0 {
                    maxRadius = minRadius + 8.0;
                }

                (*ent).owner = core::ptr::null_mut(); //so his trace hits me

                while curRadius <= maxRadius {
                    VectorMA(
                        (*ent).currentOrigin,
                        curRadius,
                        backDir.as_ptr(),
                        start.as_mut_ptr(),
                    );
                    //make sure they're not in the ground
                    VectorCopy(start.as_ptr(), end.as_mut_ptr());
                    start[2] += 18.0;
                    end[2] -= 18.0;
                    gi.trace(
                        &mut trace,
                        start.as_ptr(),
                        (*ent).mins,
                        (*ent).maxs,
                        end.as_ptr(),
                        (*ent).s.number,
                        (*ent).clipmask,
                    );
                    if !trace.allsolid && !trace.startsolid {
                        G_SetOrigin(ent, trace.endpos);
                        gi.linkentity(ent);
                        safeExit = true;
                        break;
                    }
                    curRadius += 4.0;
                }
                //Hmm... otherwise, don't allow them to get off?
                (*ent).owner = eweb;
                if !safeExit {
                    //don't try again for a second
                    (*(*ent).owner).delay = level.time + 500;
                    return;
                }
            }
        } else if (*ent).health <= 0 {
            // dead, so give 'em a push out of the chair
            let mut dir: [f32; 3] = [0.0; 3];
            AngleVectors(
                (*(*ent).owner).s.angles,
                core::ptr::null_mut(),
                dir.as_mut_ptr(),
                core::ptr::null_mut(),
            );

            if rand() & 1 != 0 {
                VectorScale(dir.as_ptr(), -1.0, dir.as_mut_ptr());
            }

            VectorMA(
                (*(*ent).client).ps.velocity,
                75.0,
                dir.as_ptr(),
                (*(*ent).client).ps.velocity,
            );
        }
        //don't let them move towards me for a couple frames so they don't step back into me while I'm becoming solid to them
        if (*ent).s.number < MAX_CLIENTS {
            if (*(*ent).client).ps.pm_time < 100 {
                (*(*ent).client).ps.pm_time = 100;
            }
            (*(*ent).client).ps.pm_flags |= PMF_TIME_NOFRICTION | PMF_TIME_KNOCKBACK;
        }

        if (*(*ent).owner).bounceCount == 0 {
            //not an EWeb - the overridden bone angles will remember the angle we left it at
            VectorCopy((*(*ent).client).ps.viewangles, (*(*ent).owner).s.angles);
            (*(*ent).owner).s.angles[PITCH] = 0.0;
            G_SetAngles((*ent).owner, (*(*ent).owner).s.angles);
            VectorCopy((*(*ent).owner).s.angles, (*(*ent).owner).pos1);
        }
    }

    // Remove the emplaced gun from our inventory
    (*(*ent).client).ps.stats[STAT_WEAPONS as usize] &= !(1 << WP_EMPLACED_GUN);

    if (*ent).health <= 0 {
        //when die, don't set weapon back on when ejected from emplaced/eweb
        //empty hands
        (*(*ent).client).ps.weapon = WP_NONE;
        if !(*ent).NPC.is_null() {
            ChangeWeapon(ent, (*(*ent).client).ps.weapon); // should be OK actually.
        } else {
            CG_ChangeWeapon((*(*ent).client).ps.weapon);
        }
        if (*ent).s.number < MAX_CLIENTS {
            gi.cvar_set(b"cg_thirdperson\0".as_ptr(), b"1\0".as_ptr());
        }
    } else {
        // when we lock or unlock from the the gun, we get our old weapon back
        (*(*ent).client).ps.weapon = (*(*(*ent).owner).s.weapon);

        if !(*ent).NPC.is_null() {
            //BTW, if a saber-using NPC ever gets off of an emplaced gun/eweb, this will not work, look at NPC_ChangeWeapon for the proper way
            ChangeWeapon(ent, (*(*ent).client).ps.weapon);
        } else {
            G_RemoveWeaponModels(ent);
            CG_ChangeWeapon((*(*ent).client).ps.weapon);
            if (*(*ent).client).ps.weapon == WP_SABER {
                WP_SaberAddG2SaberModels(ent);
            } else {
                G_CreateG2AttachedWeaponModel(
                    ent,
                    weaponData[(*(*ent).client).ps.weapon as usize].weaponMdl,
                    (*ent).handRBolt,
                    0,
                );
            }

            if (*ent).s.number < MAX_CLIENTS {
                if (*(*ent).client).ps.weapon == WP_SABER {
                    gi.cvar_set(b"cg_thirdperson\0".as_ptr(), b"1\0".as_ptr());
                } else if (*(*ent).client).ps.weapon != WP_SABER && cg_gunAutoFirst.integer != 0
                {
                    gi.cvar_set(b"cg_thirdperson\0".as_ptr(), b"0\0".as_ptr());
                }
            }
        }

        if (*(*ent).client).ps.weapon == WP_SABER {
            if (*(*ent).owner).alt_fire != 0 {
                (*(*ent).client).ps.SaberActivate();
            } else {
                (*(*ent).client).ps.SaberDeactivate();
            }
        }
    }
    //set the emplaced gun/eweb's weapon back to the emplaced gun
    (*(*(*ent).owner).s.weapon) = WP_EMPLACED_GUN;
    //	gi.G2API_DetachG2Model( &(*ent).ghoul2[(*ent).playerModel] );

    (*ent).s.eFlags &= !EF_LOCKED_TO_WEAPON;
    (*(*ent).client).ps.eFlags &= !EF_LOCKED_TO_WEAPON;

    (*(*ent).owner).noDamageTeam = TEAM_FREE;
    (*(*ent).owner).svFlags &= !SVF_NONNPC_ENEMY;
    (*(*ent).owner).delay = level.time;
    (*(*ent).owner).activator = core::ptr::null_mut();

    if (*ent).NPC.is_null() {
        // by keeping the owner, a dead npc can be pushed out of the chair without colliding with it
        (*ent).owner = core::ptr::null_mut();
    }
}

unsafe fn RunEmplacedWeapon(ent: *mut gentity_t, ucmd: *mut *mut usercmd_t) {
    if ((**ucmd).buttons & BUTTON_USE != 0 || (**ucmd).forwardmove < 0 || (**ucmd).upmove > 0)
        && !(*ent).owner.is_null()
        && (*(*ent).owner).delay + 500 < level.time
    {
        (*(*ent).owner).s.loopSound = 0;

        if (*(*ent).owner).e_UseFunc == Some(eweb_use as _) {
            //yeah, crappy way to check this, but...
            G_Sound(ent, G_SoundIndex(b"sound/weapons/eweb/eweb_dismount.mp3\0".as_ptr()));
        } else {
            G_Sound(ent, G_SoundIndex(b"sound/weapons/emplaced/emplaced_dismount.mp3\0".as_ptr()));
        }
        #[cfg(feature = "_IMMERSION")]
        {
            G_Force(
                ent,
                G_ForceIndex(
                    b"fffx/weapons/emplaced/emplaced_dismount\0".as_ptr(),
                    FF_CHANNEL_TOUCH,
                ),
            );
        }

        ExitEmplacedWeapon(ent);
        (**ucmd).buttons &= !BUTTON_USE;
        if (**ucmd).upmove > 0 {
            //don't actually jump
            (**ucmd).upmove = 0;
        }
    } else {
        // this is a crappy way to put sounds on a moving eweb....
        if !(*ent).owner.is_null()
            && (*(*ent).owner).e_UseFunc == Some(eweb_use as _)
        {
            //yeah, crappy way to check this, but...
            if !VectorCompare((*(*ent).client).ps.viewangles, (*(*ent).owner).movedir) != 0 {
                (*(*ent).owner).s.loopSound = G_SoundIndex(b"sound/weapons/eweb/eweb_aim.wav\0".as_ptr());
                (*(*ent).owner).fly_sound_debounce_time = level.time;
            } else {
                if (*(*ent).owner).fly_sound_debounce_time + 100 <= level.time {
                    (*(*ent).owner).s.loopSound = 0;
                }
            }

            VectorCopy(
                (*(*ent).client).ps.viewangles,
                (*(*ent).owner).movedir,
            );
        }

        // don't allow movement, weapon switching, and most kinds of button presses
        (**ucmd).forwardmove = 0;
        (**ucmd).rightmove = 0;
        (**ucmd).upmove = 0;
        (**ucmd).buttons &= BUTTON_ATTACK | BUTTON_ALT_ATTACK;

        (**ucmd).weapon = (*(*ent).client).ps.weapon; //WP_EMPLACED_GUN;

        if (*ent).health <= 0 {
            ExitEmplacedWeapon(ent);
        }
    }
}

// Stub functions for references that we don't have full definitions for
fn fx_runner_think(_ent: *mut gentity_t) {
    // Placeholder implementation
}

// Stubs for external variables and types we reference
static mut vec3_origin: [f32; 3] = [0.0, 0.0, 0.0];
static mut weaponData: [WeaponData; 3] = [
    WeaponData { ammoIndex: 0, weaponMdl: core::ptr::null() },
    WeaponData { ammoIndex: 0, weaponMdl: core::ptr::null() },
    WeaponData { ammoIndex: 0, weaponMdl: core::ptr::null() },
];
static mut cg_gunAutoFirst: CVarStatic = CVarStatic { integer: 0 };

#[repr(C)]
struct WeaponData {
    ammoIndex: c_int,
    weaponMdl: *const u8,
}

#[repr(C)]
struct CVarStatic {
    integer: c_int,
}

// Stub for G_Force and G_ForceIndex
extern "C" {
    fn G_Force(ent: *mut gentity_t, force: c_int);
    fn G_ForceIndex(name: *const u8, channel: c_int) -> c_int;
}

// Namespace stub for NAV
mod NAV {
    use core::ffi::c_int;

    pub unsafe fn GetNearestNode(ent: *mut crate::gentity_t) -> c_int {
        // Placeholder
        0
    }
}

// Scale vector function stub
unsafe fn VectorScale(a: *const f32, scale: f32, b: *mut f32) {
    // Placeholder
}
