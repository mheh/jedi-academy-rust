//! `bg_vehicles.h` — the vehicle data layer shared by both the server game and
//! client game modules (the "BG" = both-games layer).
//!
//! This header defines the full `Vehicle_t` (instance) and `vehicleInfo_t`
//! (shared definition) structs whose forward declaration (`typedef struct
//! Vehicle_s Vehicle_t;`) lived **opaque** in [`g_public_h`] until now — porting it
//! here gives `gentity_t::m_pVehicle` (and the BG `bgEntity_t::m_pVehicle`) a real
//! target, and unblocks `bgEntity_t`/`pmove_t` in [`bg_public`].
//!
//! Mutual recursion: `Vehicle_t`/`vehicleInfo_t` reference `bgEntity_t` only through
//! pointers, and `bgEntity_t` (bg_public.h) references `Vehicle_t` only through a
//! pointer. The C breaks the cycle with the two forward `typedef`s at the top of this
//! header (`Vehicle_s`/`bgEntity_s`); Rust breaks it by letting the sibling modules
//! `use` each other (pointers need no size), so the real `bgEntity_t` lives in
//! [`bg_public`] and is imported here. See `DEVIATIONS.md`.
//!
//! `Vehicle_t`/`vehicleInfo_t` carry pointers (and `vehicleInfo_t` carries C function
//! pointers), so their layout is arch-dependent: the size/offset asserts are gated
//! `#[cfg(target_pointer_width = "64")]` and validated against a host-64-bit C oracle,
//! the same pointer-prefix hazard navigated for `g_local.h`'s `gentity_t`. `Vehicle_t`
//! additionally embeds a C `long` (`m_ulFlags`), so its numeric asserts are narrowed
//! further to LP64 (`not(target_os = "windows")`) — Windows LLP64 has a 4-byte `long`
//! and would false-fail the LP64-calibrated values (EXTERN_C_AUDIT.md section D).
//!
//! Not ported here, per the header-port convention: the `extern` data tables
//! (`VehicleTable`, `g_vehWeaponInfo`/`numVehicleWeapons`, `g_vehicleInfo`/
//! `numVehicles`), the `BG_VehicleGetIndex` prototype, and the `VWFOFS`/`VFOFS`
//! offsetof macros — all land when their defining `.c` (`bg_vehicleLoad.c`) is ported.
//!
//! [`g_public_h`]: crate::codemp::game::g_public_h
//! [`bg_public`]: crate::codemp::game::bg_public

#![allow(non_camel_case_types, non_snake_case)]

use crate::codemp::game::bg_public::bgEntity_t;
use crate::codemp::game::q_shared_h::{qboolean, trace_t, usercmd_t, vec3_t, vec_t};
use core::ffi::{c_char, c_int, c_ulong};

// `typedef struct Vehicle_s Vehicle_t;` / `typedef struct bgEntity_s bgEntity_t;`
// (bg_vehicles.h:6-7) — the two forward declarations that break the Vehicle_t <->
// bgEntity_t pointer cycle. In Rust the real types are the structs below
// (`Vehicle_t`) and in `bg_public` (`bgEntity_t`); no separate forward decl is needed.

/// `vehicleType_t` (bg_vehicles.h) — what kind of vehicle. Ported as a C-`int`-width
/// alias + consts (faithful to the C enum's `int` storage), per the `g_public_h`
/// enum convention.
pub type vehicleType_t = c_int;
pub const VH_NONE: vehicleType_t = 0; //0 just in case anyone confuses VH_NONE and VEHICLE_NONE below
pub const VH_WALKER: vehicleType_t = 1; //something you ride inside of, it walks like you, like an AT-ST
pub const VH_FIGHTER: vehicleType_t = 2; //something you fly inside of, like an X-Wing or TIE fighter
pub const VH_SPEEDER: vehicleType_t = 3; //something you ride on that hovers, like a speeder or swoop
pub const VH_ANIMAL: vehicleType_t = 4; //animal you ride on top of that walks, like a tauntaun
pub const VH_FLIER: vehicleType_t = 5; //animal you ride on top of that flies, like a giant mynoc?
pub const VH_NUM_VEHICLES: vehicleType_t = 6;

/// `EWeaponPose` (bg_vehicles.h).
pub type EWeaponPose = c_int;
pub const WPOSE_NONE: EWeaponPose = 0;
pub const WPOSE_BLASTER: EWeaponPose = 1;
pub const WPOSE_SABERLEFT: EWeaponPose = 2;
pub const WPOSE_SABERRIGHT: EWeaponPose = 3;

// extern stringID_table_t VehicleTable[VH_NUM_VEHICLES+1]; -- extern data table,
// lands with bg_vehicleLoad.c.

//===========================================================================================================
//START VEHICLE WEAPONS
//===========================================================================================================
/// `vehWeaponInfo_t` (bg_vehicles.h) — a vehicle weapon's stats. Pointer-bearing
/// (`name`), so the layout is arch-dependent (asserts gated to 64-bit).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct vehWeaponInfo_t {
    //*** IMPORTANT!!! *** the number of variables in the vehWeaponStats_t struct (including all elements of arrays) must be reflected by NUM_VWEAP_PARMS!!!
    //*** IMPORTANT!!! *** vWeapFields table correponds to this structure!
    pub name: *mut c_char,
    pub bIsProjectile: qboolean, //traceline or entity?
    pub bHasGravity: qboolean,   //if a projectile, drops
    pub bIonWeapon: qboolean,    //disables ship shields and sends them out of control
    pub bSaberBlockable: qboolean, //lightsabers can deflect this projectile
    pub iMuzzleFX: c_int,        //index of Muzzle Effect
    pub iModel: c_int,           //handle to the model used by this projectile
    pub iShotFX: c_int,          //index of Shot Effect
    pub iImpactFX: c_int,        //index of Impact Effect
    pub iG2MarkShaderHandle: c_int, //index of shader to use for G2 marks made on other models when hit by this projectile
    pub fG2MarkSize: f32,        //size (diameter) of the ghoul2 mark
    pub iLoopSound: c_int,       //index of loopSound
    pub fSpeed: f32,             //speed of projectile/range of traceline
    pub fHoming: f32,            //0.0 = not homing, 0.5 = half vel to targ, half cur vel, 1.0 = all vel to targ
    pub fHomingFOV: f32,         //missile will lose lock on if DotProduct of missile direction and direction to target ever drops below this (-1 to 1, -1 = never lose target, 0 = lose if ship gets behind missile, 1 = pretty much will lose it's target right away)
    pub iLockOnTime: c_int,      //0 = no lock time needed, else # of ms needed to lock on
    pub iDamage: c_int,          //damage done when traceline or projectile directly hits target
    pub iSplashDamage: c_int,    //damage done to ents in splashRadius of end of traceline or projectile origin on impact
    pub fSplashRadius: f32,      //radius that ent must be in to take splashDamage (linear fall-off)
    pub iAmmoPerShot: c_int,     //how much "ammo" each shot takes
    pub iHealth: c_int,          //if non-zero, projectile can be shot, takes this much damage before being destroyed
    pub fWidth: f32,             //width of traceline or bounding box of projecile (non-rotating!)
    pub fHeight: f32,            //height of traceline or bounding box of projecile (non-rotating!)
    pub iLifeTime: c_int,        //removes itself after this amount of time
    pub bExplodeOnExpire: qboolean, //when iLifeTime is up, explodes rather than simply removing itself
}
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, name) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<vehWeaponInfo_t>() == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, bIsProjectile) == 8);

//NOTE: this MUST stay up to date with the number of variables in the vehFields table!!!
/// `NUM_VWEAP_PARMS` (bg_vehicles.h).
pub const NUM_VWEAP_PARMS: c_int = 25;

// #define VWFOFS(x) ((int)&(((vehWeaponInfo_t *)0)->x)) -- offsetof macro for the
// vehWeaponInfo field-parser table; lands with bg_vehicleLoad.c.

/// `MAX_VEH_WEAPONS` (bg_vehicles.h) — sigh... no more than 16 different vehicle weapons.
pub const MAX_VEH_WEAPONS: usize = 16;
/// `VEH_WEAPON_BASE` (bg_vehicles.h).
pub const VEH_WEAPON_BASE: c_int = 0;
/// `VEH_WEAPON_NONE` (bg_vehicles.h).
pub const VEH_WEAPON_NONE: c_int = -1;

// extern vehWeaponInfo_t g_vehWeaponInfo[MAX_VEH_WEAPONS]; extern int numVehicleWeapons;
// -- extern data tables, land with bg_vehicleLoad.c.

//===========================================================================================================
//END VEHICLE WEAPONS
//===========================================================================================================

/// `MAX_VEHICLE_MUZZLES` (bg_vehicles.h).
pub const MAX_VEHICLE_MUZZLES: usize = 12;
/// `MAX_VEHICLE_EXHAUSTS` (bg_vehicles.h).
pub const MAX_VEHICLE_EXHAUSTS: usize = 12;
/// `MAX_VEHICLE_WEAPONS` (bg_vehicles.h).
pub const MAX_VEHICLE_WEAPONS: usize = 2;
/// `MAX_VEHICLE_TURRETS` (bg_vehicles.h).
pub const MAX_VEHICLE_TURRETS: usize = 2;
/// `MAX_VEHICLE_TURRET_MUZZLES` (bg_vehicles.h).
pub const MAX_VEHICLE_TURRET_MUZZLES: usize = 2;

/// `turretStats_t` (bg_vehicles.h) — a vehicle turret's stats. Pointer-bearing
/// (`yawBone`/`pitchBone`/`gunnerViewTag`), so the layout is arch-dependent.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct turretStats_t {
    pub iWeapon: c_int,         //what vehWeaponInfo index to use
    pub iDelay: c_int,          //delay between turret muzzle shots
    pub iAmmoMax: c_int,        //how much ammo it has
    pub iAmmoRechargeMS: c_int, //how many MS between every point of recharged ammo
    pub yawBone: *mut c_char,   //bone on ship that this turret uses to yaw
    pub pitchBone: *mut c_char, //bone on ship that this turret uses to pitch
    pub yawAxis: c_int,         //axis on yawBone to which we should to apply the yaw angles
    pub pitchAxis: c_int,       //axis on pitchBone to which we should to apply the pitch angles
    pub yawClampLeft: f32,      //how far the turret is allowed to turn left
    pub yawClampRight: f32,     //how far the turret is allowed to turn right
    pub pitchClampUp: f32,      //how far the turret is allowed to title up
    pub pitchClampDown: f32,    //how far the turret is allowed to tilt down
    pub iMuzzle: [c_int; MAX_VEHICLE_TURRET_MUZZLES], //iMuzzle-1 = index of ship's muzzle to fire this turret's 1st and 2nd shots from
    pub gunnerViewTag: *mut c_char, //Where to put the view origin of the gunner (name)
    pub fTurnSpeed: f32,        //how quickly the turret can turn
    pub bAI: qboolean,          //whether or not the turret auto-targets enemies when it's not manned
    pub bAILead: qboolean,      //whether
    pub fAIRange: f32,          //how far away the AI will look for enemies
    pub passengerNum: c_int,    //which passenger, if any, has control of this turret (overrides AI)
}
const _: () = assert!(core::mem::offset_of!(turretStats_t, iWeapon) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<turretStats_t>() == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, yawBone) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, iMuzzle) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, gunnerViewTag) == 64);

/// `vehWeaponStats_t` (bg_vehicles.h) — a vehicle's per-weapon configured stats.
/// Pointer-free; identical on 32/64-bit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct vehWeaponStats_t {
    //*** IMPORTANT!!! *** See note at top of next structure!!! ***
    // Weapon stuff.
    pub ID: c_int, //index into the weapon data
    // The delay between shots for each weapon.
    pub delay: c_int,
    // Whether or not all the muzzles for each weapon can be linked together (linked delay = weapon delay * number of muzzles linked!)
    pub linkable: c_int,
    // Whether or not to auto-aim the projectiles/tracelines at the thing under the crosshair when we fire
    pub aimCorrect: qboolean,
    //maximum ammo
    pub ammoMax: c_int,
    //ammo recharge rate - milliseconds per unit (minimum of 100, which is 10 ammo per second)
    pub ammoRechargeMS: c_int,
    //sound to play when out of ammo (plays default "no ammo" sound if none specified)
    pub soundNoAmmo: c_int,
}
const _: () = assert!(core::mem::size_of::<vehWeaponStats_t>() == 28);
const _: () = assert!(core::mem::align_of::<vehWeaponStats_t>() == 4);

/// `vehicleInfo_t` (bg_vehicles.h) — a vehicle *type*'s shared definition (read from
/// the `.veh` file) plus the C-function-pointer table that simulates C++ inheritance
/// for vehicles. Pointer-bearing (and fn-pointer-bearing) => arch-dependent layout.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct vehicleInfo_t {
    //*** IMPORTANT!!! *** vehFields table correponds to this structure!
    pub name: *mut c_char, //unique name of the vehicle

    //general data
    pub r#type: vehicleType_t, //what kind of vehicle
    pub numHands: c_int, //if 2 hands, no weapons, if 1 hand, can use 1-handed weapons, if 0 hands, can use 2-handed weapons
    pub lookPitch: f32,  //How far you can look up and down off the forward of the vehicle
    pub lookYaw: f32,    //How far you can look left and right off the forward of the vehicle
    pub length: f32,     //how long it is - used for body length traces when turning/moving?
    pub width: f32,      //how wide it is - used for body length traces when turning/moving?
    pub height: f32,     //how tall it is - used for body length traces when turning/moving?
    pub centerOfGravity: vec3_t, //offset from origin: {forward, right, up} as a modifier on that dimension (-1.0f is all the way back, 1.0f is all the way forward)

    //speed stats
    pub speedMax: f32,       //top speed
    pub turboSpeed: f32,     //turbo speed
    pub speedMin: f32,       //if < 0, can go in reverse
    pub speedIdle: f32,      //what speed it drifts to when no accel/decel input is given
    pub accelIdle: f32,      //if speedIdle > 0, how quickly it goes up to that speed
    pub acceleration: f32,   //when pressing on accelerator
    pub decelIdle: f32,      //when giving no input, how quickly it drops to speedIdle
    pub throttleSticks: f32, //if true, speed stays at whatever you accel/decel to, unless you turbo or brake
    pub strafePerc: f32, //multiplier on current speed for strafing.  If 1.0f, you can strafe at the same speed as you're going forward, 0.5 is half, 0 is no strafing

    //handling stats
    pub bankingSpeed: f32,        //how quickly it pitches and rolls (not under player control)
    pub rollLimit: f32,           //how far it can roll to either side
    pub pitchLimit: f32,          //how far it can roll forward or backward
    pub braking: f32,             //when pressing on decelerator
    pub mouseYaw: f32,            // The mouse yaw override.
    pub mousePitch: f32,          // The mouse pitch override.
    pub turningSpeed: f32,        //how quickly you can turn
    pub turnWhenStopped: qboolean, //whether or not you can turn when not moving
    pub traction: f32,            //how much your command input affects velocity
    pub friction: f32,            //how much velocity is cut on its own
    pub maxSlope: f32,            //the max slope that it can go up with control
    pub speedDependantTurning: qboolean, //vehicle turns faster the faster it's going

    //durability stats
    pub mass: c_int,        //for momentum and impact force (player mass is 10)
    pub armor: c_int,       //total points of damage it can take
    pub shields: c_int,     //energy shield damage points
    pub shieldRechargeMS: c_int, //energy shield milliseconds per point recharged
    pub toughness: f32, //modifies incoming damage, 1.0 is normal, 0.5 is half, etc.  Simulates being made of tougher materials/construction
    pub malfunctionArmorLevel: c_int, //when armor drops to or below this point, start malfunctioning
    pub surfDestruction: c_int, //can parts of this thing be torn off on impact? -rww

    //individual "area" health -rww
    pub health_front: c_int,
    pub health_back: c_int,
    pub health_right: c_int,
    pub health_left: c_int,

    //visuals & sounds
    pub model: *mut c_char, //what model to use - if make it an NPC's primary model, don't need this?
    pub skin: *mut c_char,  //what skin to use - if make it an NPC's primary model, don't need this?
    pub g2radius: c_int,    //render radius for the ghoul2 model
    pub riderAnim: c_int,   //what animation the rider uses
    pub radarIconHandle: c_int, //what icon to show on radar in MP
    pub dmgIndicFrameHandle: c_int, //what image to use for the frame of the damage indicator
    pub dmgIndicShieldHandle: c_int, //what image to use for the shield of the damage indicator
    pub dmgIndicBackgroundHandle: c_int, //what image to use for the background of the damage indicator
    pub iconFrontHandle: c_int, //what image to use for the front of the ship on the damage indicator
    pub iconBackHandle: c_int, //what image to use for the back of the ship on the damage indicator
    pub iconRightHandle: c_int, //what image to use for the right of the ship on the damage indicator
    pub iconLeftHandle: c_int, //what image to use for the left of the ship on the damage indicator
    pub crosshairShaderHandle: c_int, //what image to use for the left of the ship on the damage indicator
    pub shieldShaderHandle: c_int, //What shader to use when drawing the shield shell
    pub droidNPC: *mut c_char, //NPC to attach to *droidunit tag (if it exists in the model)

    pub soundOn: c_int,          //sound to play when get on it
    pub soundTakeOff: c_int,     //sound to play when ship takes off
    pub soundEngineStart: c_int, //sound to play when ship's thrusters first activate
    pub soundLoop: c_int,        //sound to loop while riding it
    pub soundSpin: c_int,        //sound to loop while spiraling out of control
    pub soundTurbo: c_int,       //sound to play when turbo/afterburner kicks in
    pub soundHyper: c_int,       //sound to play when ship lands
    pub soundLand: c_int,        //sound to play when ship lands
    pub soundOff: c_int,         //sound to play when get off
    pub soundFlyBy: c_int,       //sound to play when they buzz you
    pub soundFlyBy2: c_int,      //alternate sound to play when they buzz you
    pub soundShift1: c_int,      //sound to play when accelerating
    pub soundShift2: c_int,      //sound to play when accelerating
    pub soundShift3: c_int,      //sound to play when decelerating
    pub soundShift4: c_int,      //sound to play when decelerating

    pub iExhaustFX: c_int,    //exhaust effect, played from "*exhaust" bolt(s)
    pub iTurboFX: c_int,      //turbo exhaust effect, played from "*exhaust" bolt(s) when ship is in "turbo" mode
    pub iTurboStartFX: c_int, //turbo begin effect, played from "*exhaust" bolts when "turbo" mode begins
    pub iTrailFX: c_int,      //trail effect, played from "*trail" bolt(s)
    pub iImpactFX: c_int,     //impact effect, for when it bumps into something
    pub iExplodeFX: c_int,    //explosion effect, for when it blows up (should have the sound built into explosion effect)
    pub iWakeFX: c_int,       //effect it makes when going across water
    pub iDmgFX: c_int,        //effect to play on damage from a weapon or something
    pub iInjureFX: c_int,
    pub iNoseFX: c_int,  //effect for nose piece flying away when blown off
    pub iLWingFX: c_int, //effect for left wing piece flying away when blown off
    pub iRWingFX: c_int, //effect for right wing piece flying away when blown off

    //Weapon stats
    pub weapon: [vehWeaponStats_t; MAX_VEHICLE_WEAPONS],

    // Which weapon a muzzle fires (has to match one of the weapons this vehicle has). So 1 would be weapon 1,
    // 2 would be weapon 2 and so on.
    pub weapMuzzle: [c_int; MAX_VEHICLE_MUZZLES],

    //turrets (if any) on the vehicle
    pub turret: [turretStats_t; MAX_VEHICLE_TURRETS],

    // The max height before this ship (?) starts (auto)landing.
    pub landingHeight: f32,

    //other misc stats
    pub gravity: c_int,       //normal is 800
    pub hoverHeight: f32,     //if 0, it's a ground vehicle
    pub hoverStrength: f32,   //how hard it pushes off ground when less than hover height... causes "bounce", like shocks
    pub waterProof: qboolean, //can drive underwater if it has to
    pub bouyancy: f32,        //when in water, how high it floats (1 is neutral bouyancy)
    pub fuelMax: c_int,       //how much fuel it can hold (capacity)
    pub fuelRate: c_int,      //how quickly is uses up fuel
    pub turboDuration: c_int, //how long turbo lasts
    pub turboRecharge: c_int, //how long turbo takes to recharge
    pub visibility: c_int,    //for sight alerts
    pub loudness: c_int,      //for sound alerts
    pub explosionRadius: f32, //range of explosion
    pub explosionDamage: c_int, //damage of explosion

    pub maxPassengers: c_int, // The max number of passengers this vehicle may have (Default = 0).
    pub hideRider: qboolean,  // rider (and passengers?) should not be drawn
    pub killRiderOnDeath: qboolean, //if rider is on vehicle when it dies, they should die
    pub flammable: qboolean,  //whether or not the vehicle should catch on fire before it explodes
    pub explosionDelay: c_int, //how long the vehicle should be on fire/dying before it explodes
    //camera stuff
    pub cameraOverride: qboolean, //whether or not to use all of the following 3rd person camera override values
    pub cameraRange: f32,         //how far back the camera should be - normal is 80
    pub cameraVertOffset: f32,    //how high over the vehicle origin the camera should be - normal is 16
    pub cameraHorzOffset: f32, //how far to left/right (negative/positive) of of the vehicle origin the camera should be - normal is 0
    pub cameraPitchOffset: f32, //a modifier on the camera's pitch (up/down angle) to the vehicle - normal is 0
    pub cameraFOV: f32,        //third person camera FOV, default is 80
    pub cameraAlpha: f32, //fade out the vehicle to this alpha (0.1-1.0f) if it's in the way of the crosshair
    pub cameraPitchDependantVertOffset: qboolean, //use the hacky AT-ST pitch dependant vertical offset

    //NOTE: some info on what vehicle weapon to use?  Like ATST or TIE bomber or TIE fighter or X-Wing...?

    //===VEH_PARM_MAX========================================================================
    //*** IMPORTANT!!! *** vehFields table correponds to this structure!

    //THE FOLLOWING FIELDS are not in the vehFields table because they are internal variables, not read in from the .veh file
    pub modelIndex: c_int, //set internally, not until this vehicle is spawned into the level

    // NOTE: Please note that most of this stuff has been converted from C++ classes to generic C.
    // This part of the structure is used to simulate inheritance for vehicles. The basic idea is that all vehicle use
    // this vehicle interface since they declare their own functions and assign the function pointer to the
    // corresponding function. Meanwhile, the base logic can still call the appropriate functions. In C++ talk all
    // of these functions (pointers) are pure virtuals and this is an abstract base class (although it cannot be
    // inherited from, only contained and reimplemented (through an object and a setup function respectively)). -AReis

    // Makes sure that the vehicle is properly animated.
    pub AnimateVehicle: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // Makes sure that the rider's in this vehicle are properly animated.
    pub AnimateRiders: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // Determine whether this entity is able to board this vehicle or not.
    pub ValidateBoard: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean>,

    // Set the parent entity of this Vehicle NPC.
    pub SetParent: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pParentEntity: *mut bgEntity_t)>,

    // Add a pilot to the vehicle.
    pub SetPilot: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pPilot: *mut bgEntity_t)>,

    // Add a passenger to the vehicle (false if we're full).
    pub AddPassenger: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t) -> qboolean>,

    // Animate the vehicle and it's riders.
    pub Animate: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // Board this Vehicle (get on). The first entity to board an empty vehicle becomes the Pilot.
    pub Board: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t) -> qboolean>,

    // Eject an entity from the vehicle.
    pub Eject: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t, forceEject: qboolean) -> qboolean>,

    // Eject all the inhabitants of this vehicle.
    pub EjectAll: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t) -> qboolean>,

    // Start a delay until the vehicle dies.
    pub StartDeathDelay: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, iDelayTime: c_int)>,

    // Update death sequence.
    pub DeathUpdate: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // Register all the assets used by this vehicle.
    pub RegisterAssets: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // Initialize the vehicle (should be called by Spawn?).
    pub Initialize: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t) -> qboolean>,

    // Like a think or move command, this updates various vehicle properties.
    pub Update: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pUcmd: *const usercmd_t) -> qboolean>,

    // Update the properties of a Rider (that may reflect what happens to the vehicle).
    //
    //	[return]		bool			True if still in vehicle, false if otherwise.
    pub UpdateRider: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pRider: *mut bgEntity_t, pUcmd: *mut usercmd_t) -> qboolean>,

    // ProcessMoveCommands the Vehicle.
    pub ProcessMoveCommands: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // ProcessOrientCommands the Vehicle.
    pub ProcessOrientCommands: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // Attachs all the riders of this vehicle to their appropriate position/tag (*driver, *pass1, *pass2, whatever...).
    pub AttachRiders: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t)>,

    // Make someone invisible and un-collidable.
    pub Ghost: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t)>,

    // Make someone visible and collidable.
    pub UnGhost: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t, pEnt: *mut bgEntity_t)>,

    // Get the pilot of this vehicle.
    pub GetPilot: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t) -> *const bgEntity_t>,

    // Whether this vehicle is currently inhabited (by anyone) or not.
    pub Inhabited: Option<unsafe extern "C" fn(pVeh: *mut Vehicle_t) -> qboolean>,
}
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, name) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<vehicleInfo_t>() == 952);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, r#type) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, model) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, weapon) == 356);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, turret) == 464);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, modelIndex) == 764);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehicleInfo_t, AnimateVehicle) == 768);

// #define VFOFS(x) ((int)&(((vehicleInfo_t *)0)->x)) -- offsetof macro for the
// vehFields parser table; lands with bg_vehicleLoad.c.

/// `MAX_VEHICLES` (bg_vehicles.h) — sigh... no more than 64 individual vehicles.
pub const MAX_VEHICLES: usize = 16;
/// `VEHICLE_BASE` (bg_vehicles.h).
pub const VEHICLE_BASE: c_int = 0;
/// `VEHICLE_NONE` (bg_vehicles.h).
pub const VEHICLE_NONE: c_int = -1;

// extern vehicleInfo_t g_vehicleInfo[MAX_VEHICLES]; extern int numVehicles; -- extern
// data tables, land with bg_vehicleLoad.c.

pub const VEH_DEFAULT_SPEED_MAX: f32 = 800.0;
pub const VEH_DEFAULT_ACCEL: f32 = 10.0;
pub const VEH_DEFAULT_DECEL: f32 = 10.0;
pub const VEH_DEFAULT_STRAFE_PERC: f32 = 0.5;
pub const VEH_DEFAULT_BANKING_SPEED: f32 = 0.5;
pub const VEH_DEFAULT_ROLL_LIMIT: f32 = 60.0;
pub const VEH_DEFAULT_PITCH_LIMIT: f32 = 90.0;
pub const VEH_DEFAULT_BRAKING: f32 = 10.0;
pub const VEH_DEFAULT_TURNING_SPEED: f32 = 1.0;
pub const VEH_DEFAULT_TRACTION: f32 = 8.0;
pub const VEH_DEFAULT_FRICTION: f32 = 1.0;
pub const VEH_DEFAULT_MAX_SLOPE: f32 = 0.85;
pub const VEH_DEFAULT_MASS: c_int = 200;
pub const VEH_DEFAULT_MAX_ARMOR: c_int = 200;
pub const VEH_DEFAULT_TOUGHNESS: f32 = 2.5;
pub const VEH_DEFAULT_GRAVITY: c_int = 800;
pub const VEH_DEFAULT_HOVER_HEIGHT: f32 = 64.0;
pub const VEH_DEFAULT_HOVER_STRENGTH: f32 = 10.0;
pub const VEH_DEFAULT_VISIBILITY: c_int = 0;
pub const VEH_DEFAULT_LOUDNESS: c_int = 0;
pub const VEH_DEFAULT_EXP_RAD: f32 = 400.0;
pub const VEH_DEFAULT_EXP_DMG: c_int = 1000;
pub const VEH_MAX_PASSENGERS: usize = 10;

pub const MAX_STRAFE_TIME: f32 = 2000.0; //FIXME: extern?
pub const MIN_LANDING_SPEED: c_int = 200; //equal to or less than this and close to ground = auto-slow-down to land
pub const MIN_LANDING_SLOPE: f32 = 0.8; //must be pretty flat to land on the surf

pub const VEH_MOUNT_THROW_LEFT: c_int = -5;
pub const VEH_MOUNT_THROW_RIGHT: c_int = -6;

// Eject directions. NOTE: a `typedef enum { ... };` with no tag and no typedef name in
// the C -- it only introduces these enumerator constants, so they are ported as plain
// `c_int` consts (there is no named type).
pub const VEH_EJECT_LEFT: c_int = 0;
pub const VEH_EJECT_RIGHT: c_int = 1;
pub const VEH_EJECT_FRONT: c_int = 2;
pub const VEH_EJECT_REAR: c_int = 3;
pub const VEH_EJECT_TOP: c_int = 4;
pub const VEH_EJECT_BOTTOM: c_int = 5;

/// `vehFlags_t` (bg_vehicles.h) — flags that describe the vehicle's behavior. (Stored
/// in `Vehicle_t::m_ulFlags`, an `unsigned long`.)
pub type vehFlags_t = c_int;
pub const VEH_NONE: vehFlags_t = 0;
pub const VEH_FLYING: vehFlags_t = 0x00000001;
pub const VEH_CRASHING: vehFlags_t = 0x00000002;
pub const VEH_LANDING: vehFlags_t = 0x00000004;
pub const VEH_BUCKING: vehFlags_t = 0x00000010;
pub const VEH_WINGSOPEN: vehFlags_t = 0x00000020;
pub const VEH_GEARSOPEN: vehFlags_t = 0x00000040;
pub const VEH_SLIDEBREAKING: vehFlags_t = 0x00000080;
pub const VEH_SPINNING: vehFlags_t = 0x00000100;
pub const VEH_OUTOFCONTROL: vehFlags_t = 0x00000200;
pub const VEH_SABERINLEFTHAND: vehFlags_t = 0x00000400;

//defines for impact damage surface stuff
pub const SHIPSURF_FRONT: c_int = 0;
pub const SHIPSURF_BACK: c_int = 1;
pub const SHIPSURF_RIGHT: c_int = 2;
pub const SHIPSURF_LEFT: c_int = 3;

pub const SHIPSURF_DAMAGE_FRONT_LIGHT: c_int = 0;
pub const SHIPSURF_DAMAGE_BACK_LIGHT: c_int = 1;
pub const SHIPSURF_DAMAGE_RIGHT_LIGHT: c_int = 2;
pub const SHIPSURF_DAMAGE_LEFT_LIGHT: c_int = 3;
pub const SHIPSURF_DAMAGE_FRONT_HEAVY: c_int = 4;
pub const SHIPSURF_DAMAGE_BACK_HEAVY: c_int = 5;
pub const SHIPSURF_DAMAGE_RIGHT_HEAVY: c_int = 6;
pub const SHIPSURF_DAMAGE_LEFT_HEAVY: c_int = 7;

//generic part bits
pub const SHIPSURF_BROKEN_A: c_int = 1 << 0; //gear 1
pub const SHIPSURF_BROKEN_B: c_int = 1 << 1; //gear 1
pub const SHIPSURF_BROKEN_C: c_int = 1 << 2; //wing 1
pub const SHIPSURF_BROKEN_D: c_int = 1 << 3; //wing 2
pub const SHIPSURF_BROKEN_E: c_int = 1 << 4; //wing 3
pub const SHIPSURF_BROKEN_F: c_int = 1 << 5; //wing 4
pub const SHIPSURF_BROKEN_G: c_int = 1 << 6; //front

/// `vehWeaponStatus_t` (bg_vehicles.h) — runtime per-weapon firing state. Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct vehWeaponStatus_t {
    //linked firing mode
    pub linked: qboolean, //weapon 1's muzzles are in linked firing mode
    //current weapon ammo
    pub ammo: c_int,
    //debouncer for ammo recharge
    pub lastAmmoInc: c_int,
    //which muzzle will fire next
    pub nextMuzzle: c_int,
}
const _: () = assert!(core::mem::size_of::<vehWeaponStatus_t>() == 16);
const _: () = assert!(core::mem::align_of::<vehWeaponStatus_t>() == 4);

/// `vehTurretStatus_t` (bg_vehicles.h) — runtime per-turret firing state. Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct vehTurretStatus_t {
    //current weapon ammo
    pub ammo: c_int,
    //debouncer for ammo recharge
    pub lastAmmoInc: c_int,
    //which muzzle will fire next
    pub nextMuzzle: c_int,
    //which entity they're after
    pub enemyEntNum: c_int,
    //how long to hold on to our current enemy
    pub enemyHoldTime: c_int,
}
const _: () = assert!(core::mem::size_of::<vehTurretStatus_t>() == 20);
const _: () = assert!(core::mem::align_of::<vehTurretStatus_t>() == 4);

// This is the implementation of the vehicle interface and any of the other variables needed. This
// is what actually represents a vehicle. -AReis.
/// `Vehicle_s` / `Vehicle_t` (bg_vehicles.h) — a live vehicle instance. The full
/// struct whose forward declaration was opaque in [`g_public_h`]. Pointer-bearing
/// (and embeds `unsigned long m_ulFlags`, also 64-bit on the LP64 host) =>
/// arch-dependent layout. (`g_public_h::Vehicle_t` re-exports this type.)
///
/// [`g_public_h`]: crate::codemp::game::g_public_h
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vehicle_t {
    // The entity who pilots/drives this vehicle.
    // NOTE: This is redundant (since m_pParentEntity->owner _should_ be the pilot). This makes things clearer though.
    pub m_pPilot: *mut bgEntity_t,

    pub m_iPilotTime: c_int,      //if spawnflag to die without pilot and this < level.time then die.
    pub m_iPilotLastIndex: c_int, //index to last pilot
    pub m_bHasHadPilot: qboolean, //qtrue once the vehicle gets its first pilot

    // The passengers of this vehicle.
    //bgEntity_t **m_ppPassengers;
    pub m_ppPassengers: [*mut bgEntity_t; VEH_MAX_PASSENGERS],

    //the droid unit NPC for this vehicle, if any
    pub m_pDroidUnit: *mut bgEntity_t,

    // The number of passengers currently in this vehicle.
    pub m_iNumPassengers: c_int,

    // The entity from which this NPC comes from.
    pub m_pParentEntity: *mut bgEntity_t,

    // If not zero, how long to wait before we can do anything with the vehicle (we're getting on still).
    // -1 = board from left, -2 = board from right, -3 = jump/quick board.  -4 & -5 = throw off existing pilot
    pub m_iBoarding: c_int,

    // Used to check if we've just started the boarding process
    pub m_bWasBoarding: qboolean,

    // The speed the vehicle maintains while boarding occurs (often zero)
    pub m_vBoardingVelocity: vec3_t,

    // Time modifier (must only be used in ProcessMoveCommands() and ProcessOrientCommands() and is updated in Update()).
    pub m_fTimeModifier: f32,

    // Ghoul2 Animation info.
    //int m_iDriverTag;
    pub m_iLeftExhaustTag: c_int,
    pub m_iRightExhaustTag: c_int,
    pub m_iGun1Tag: c_int,
    pub m_iGun1Bone: c_int,
    pub m_iLeftWingBone: c_int,
    pub m_iRightWingBone: c_int,

    pub m_iExhaustTag: [c_int; MAX_VEHICLE_EXHAUSTS],
    pub m_iMuzzleTag: [c_int; MAX_VEHICLE_MUZZLES],
    pub m_iDroidUnitTag: c_int,
    pub m_iGunnerViewTag: [c_int; MAX_VEHICLE_TURRETS], //Where to put the view origin of the gunner (index)

    //this stuff is a little bit different from SP, because I am lazy -rww
    pub m_iMuzzleTime: [c_int; MAX_VEHICLE_MUZZLES],
    // These are updated every frame and represent the current position and direction for the specific muzzle.
    pub m_vMuzzlePos: [vec3_t; MAX_VEHICLE_MUZZLES],
    pub m_vMuzzleDir: [vec3_t; MAX_VEHICLE_MUZZLES],

    // This is how long to wait before being able to fire a specific muzzle again. This is based on the firing rate
    // so that a firing rate of 10 rounds/sec would make this value initially 100 miliseconds.
    pub m_iMuzzleWait: [c_int; MAX_VEHICLE_MUZZLES],

    // The user commands structure.
    pub m_ucmd: usercmd_t,

    // The direction an entity will eject from the vehicle towards.
    pub m_EjectDir: c_int,

    // Flags that describe the vehicles behavior.
    pub m_ulFlags: c_ulong,

    // NOTE: Vehicle Type ID, Orientation, and Armor MUST be transmitted over the net.

    // The ID of the type of vehicle this is.
    pub m_iVehicleTypeID: c_int,

    // Current angles of this vehicle.
    //vec3_t		m_vOrientation;
    pub m_vOrientation: *mut vec_t,
    //Yeah, since we use the SP code for vehicles, I want to use this value, but I'm going
    //to make it a pointer to a vec3_t in the playerstate for prediction's sake. -rww

    // How long you have strafed left or right (increments every frame that you strafe to right, decrements every frame you strafe left)
    pub m_fStrafeTime: c_int,

    // Previous angles of this vehicle.
    pub m_vPrevOrientation: vec3_t,

    // Previous viewangles of the rider
    pub m_vPrevRiderViewAngles: vec3_t,

    // When control is lost on a speeder, current angular velocity is stored here and applied until landing
    pub m_vAngularVelocity: f32,

    pub m_vFullAngleVelocity: vec3_t,

    // Current armor and shields of your vehicle (explodes if armor to 0).
    pub m_iArmor: c_int,   //hull strength - STAT_HEALTH on NPC
    pub m_iShields: c_int, //energy shielding - STAT_ARMOR on NPC

    //mp-specific
    pub m_iHitDebounce: c_int,

    // Timer for all cgame-FX...? ex: exhaust?
    pub m_iLastFXTime: c_int,

    // When to die.
    pub m_iDieTime: c_int,

    // This pointer is to a valid VehicleInfo (which could be an animal, speeder, fighter, whatever). This
    // contains the functions actually used to do things to this specific kind of vehicle as well as shared
    // information (max speed, type, etc...).
    pub m_pVehicleInfo: *mut vehicleInfo_t,

    // This trace tells us if we're within landing height.
    pub m_LandTrace: trace_t,

    // TEMP: The wing angles (used to animate it).
    pub m_vWingAngles: vec3_t,

    //amount of damage done last impact
    pub m_iLastImpactDmg: c_int,

    //bitflag of surfaces that have broken off
    pub m_iRemovedSurfaces: c_int,

    pub m_iDmgEffectTime: c_int,

    // the last time this vehicle fired a turbo burst
    pub m_iTurboTime: c_int,

    //how long it should drop like a rock for after freed from SUSPEND
    pub m_iDropTime: c_int,

    pub m_iSoundDebounceTimer: c_int,

    //last time we incremented the shields
    pub lastShieldInc: c_int,

    //so we don't hold it down and toggle it back and forth
    pub linkWeaponToggleHeld: qboolean,

    //info about our weapons (linked, ammo, etc.)
    pub weaponStatus: [vehWeaponStatus_t; MAX_VEHICLE_WEAPONS],
    pub turretStatus: [vehTurretStatus_t; MAX_VEHICLE_TURRETS],

    //the guy who was previously the pilot
    pub m_pOldPilot: *mut bgEntity_t,
}
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pPilot) == 0); // arch-independent anchor
// Vehicle_t is the ONLY struct in this file embedding a C `long` (`m_ulFlags`,
// `c_ulong`). On LP64 (the dev host + x86_64-unknown-linux-gnu) long is 8 bytes;
// on Windows LLP64 (x86_64-pc-windows-msvc) it is 4, which shrinks the struct and
// shifts every field from `m_ulFlags` onward. These byte values were calibrated
// against the LP64 C oracle, so the numeric self-check is gated to LP64 — a plain
// `target_pointer_width = "64"` also admitted 64-bit Windows and false-failed there
// (E0080). The 32-bit targets are excluded by the pointer-width clause. The struct
// itself stays ABI-correct on every platform because `c_ulong` already tracks the
// platform's `long` width. See EXTERN_C_AUDIT.md section D.
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::size_of::<Vehicle_t>() == 976);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_ucmd) == 668);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_ulFlags) == 704);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_vOrientation) == 720);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pVehicleInfo) == 792);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_LandTrace) == 800);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, weaponStatus) == 892);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, turretStatus) == 924);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
const _: () = assert!(core::mem::offset_of!(Vehicle_t, m_pOldPilot) == 968);

// extern int BG_VehicleGetIndex( const char *vehicleName ); -- function prototype,
// lands with bg_vehicleLoad.c.

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::*;
    use core::mem::{offset_of, size_of};

    /// Parity: the standalone bg_vehicles.h structs match the authentic C `sizeof` /
    /// `offsetof` at the host (64-bit) layout. The two pointer-bearing structs
    /// (`vehWeaponInfo_t`/`turretStats_t`) validate the post-pointer offsets; the
    /// pointer-free stats/status structs are arch-independent.
    #[test]
    fn bg_vehicles_standalone_layout_matches_c() {
        unsafe {
            assert_eq!(
                size_of::<vehWeaponInfo_t>(),
                jka_bv_sizeof_vehWeaponInfo_t()
            );
            assert_eq!(
                offset_of!(vehWeaponInfo_t, bIsProjectile),
                jka_bv_off_vwi_bIsProjectile()
            );

            assert_eq!(size_of::<turretStats_t>(), jka_bv_sizeof_turretStats_t());
            assert_eq!(offset_of!(turretStats_t, yawBone), jka_bv_off_ts_yawBone());
            assert_eq!(offset_of!(turretStats_t, iMuzzle), jka_bv_off_ts_iMuzzle());
            assert_eq!(
                offset_of!(turretStats_t, gunnerViewTag),
                jka_bv_off_ts_gunnerViewTag()
            );

            assert_eq!(
                size_of::<vehWeaponStats_t>(),
                jka_bv_sizeof_vehWeaponStats_t()
            );
            assert_eq!(
                size_of::<vehWeaponStatus_t>(),
                jka_bv_sizeof_vehWeaponStatus_t()
            );
            assert_eq!(
                size_of::<vehTurretStatus_t>(),
                jka_bv_sizeof_vehTurretStatus_t()
            );
        }
    }

    /// Parity: the pointer cluster (`vehicleInfo_t` with its fn-pointer table, and
    /// `Vehicle_t`) matches the authentic C `sizeof`/`offsetof` at the host (64-bit)
    /// layout. (`bgEntity_t`, the third cluster member, is tested in `bg_public`.)
    #[test]
    fn bg_vehicles_cluster_layout_matches_c() {
        unsafe {
            assert_eq!(size_of::<vehicleInfo_t>(), jka_bv_sizeof_vehicleInfo_t());
            assert_eq!(offset_of!(vehicleInfo_t, r#type), jka_bv_off_vi_type());
            assert_eq!(offset_of!(vehicleInfo_t, model), jka_bv_off_vi_model());
            assert_eq!(offset_of!(vehicleInfo_t, weapon), jka_bv_off_vi_weapon());
            assert_eq!(offset_of!(vehicleInfo_t, turret), jka_bv_off_vi_turret());
            assert_eq!(
                offset_of!(vehicleInfo_t, modelIndex),
                jka_bv_off_vi_modelIndex()
            );
            assert_eq!(
                offset_of!(vehicleInfo_t, AnimateVehicle),
                jka_bv_off_vi_AnimateVehicle()
            );

            assert_eq!(size_of::<Vehicle_t>(), jka_bv_sizeof_Vehicle_t());
            assert_eq!(offset_of!(Vehicle_t, m_ucmd), jka_bv_off_veh_m_ucmd());
            assert_eq!(offset_of!(Vehicle_t, m_ulFlags), jka_bv_off_veh_m_ulFlags());
            assert_eq!(
                offset_of!(Vehicle_t, m_vOrientation),
                jka_bv_off_veh_m_vOrientation()
            );
            assert_eq!(
                offset_of!(Vehicle_t, m_pVehicleInfo),
                jka_bv_off_veh_m_pVehicleInfo()
            );
            assert_eq!(
                offset_of!(Vehicle_t, m_LandTrace),
                jka_bv_off_veh_m_LandTrace()
            );
            assert_eq!(
                offset_of!(Vehicle_t, weaponStatus),
                jka_bv_off_veh_weaponStatus()
            );
            assert_eq!(
                offset_of!(Vehicle_t, turretStatus),
                jka_bv_off_veh_turretStatus()
            );
            assert_eq!(
                offset_of!(Vehicle_t, m_pOldPilot),
                jka_bv_off_veh_m_pOldPilot()
            );
        }
    }
}
