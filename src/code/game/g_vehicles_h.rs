#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::c_int;
use core::ffi::c_char;
use core::ffi::c_void;

// External types from q_shared.h and g_public.h
pub struct gentity_t;
pub struct usercmd_t;
pub struct trace_t;
pub struct stringID_table_t;

pub type qboolean = bool;
pub type vec3_t = [f32; 3];

#[repr(C)]
#[derive(Copy, Clone)]
pub enum vehicleType_t {
    VH_NONE = 0,
    VH_WALKER,		//something you ride inside of, it walks like you, like an AT-ST
    VH_FIGHTER,		//something you fly inside of, like an X-Wing or TIE fighter
    VH_SPEEDER,		//something you ride on that hovers, like a speeder or swoop
    VH_ANIMAL,		//animal you ride on top of that walks, like a tauntaun
    VH_FLIER,		//animal you ride on top of that flies, like a giant mynoc?
    VH_NUM_VEHICLES,
}

#[repr(C)]
pub enum EWeaponPose {
    WPOSE_NONE	= 0,
    WPOSE_BLASTER,
    WPOSE_SABERLEFT,
    WPOSE_SABERRIGHT,
}

extern "C" {
    pub static mut VehicleTable: *mut stringID_table_t;
}

pub const NO_PILOT_DIE_TIME: c_int = 10000;

//===========================================================================================================
//START VEHICLE WEAPONS
//===========================================================================================================
#[repr(C)]
pub struct vehWeaponInfo_t {
//*** IMPORTANT!!! *** the number of variables in the vehWeaponStats_t struct (including all elements of arrays) must be reflected by NUM_VWEAP_PARMS!!!
//*** IMPORTANT!!! *** vWeapFields table correponds to this structure!
    pub name: *mut c_char,
    pub bIsProjectile: qboolean,	//traceline or entity?
    pub bHasGravity: qboolean,	//if a projectile, drops
    pub bIonWeapon: qboolean,//disables ship shields and sends them out of control
    pub bSaberBlockable: qboolean,//lightsabers can deflect this projectile
    pub iMuzzleFX: c_int,	//index of Muzzle Effect
    pub iModel: c_int,		//handle to the model used by this projectile
    pub iShotFX: c_int,	//index of Shot Effect
    pub iImpactFX: c_int,	//index of Impact Effect
    pub iG2MarkShaderHandle: c_int,	//index of shader to use for G2 marks made on other models when hit by this projectile
    pub fG2MarkSize: f32,//size (diameter) of the ghoul2 mark
    pub iLoopSound: c_int,	//index of loopSound
    pub fSpeed: f32,		//speed of projectile/range of traceline
    pub fHoming: f32,		//0.0 = not homing, 0.5 = half vel to targ, half cur vel, 1.0 = all vel to targ
    pub fHomingFOV: f32,
    pub iLockOnTime: c_int,	//0 = no lock time needed, else # of ms needed to lock on
    pub iDamage: c_int,		//damage done when traceline or projectile directly hits target
    pub iSplashDamage: c_int,//damage done to ents in splashRadius of end of traceline or projectile origin on impact
    pub fSplashRadius: f32,//radius that ent must be in to take splashDamage (linear fall-off)
    pub iAmmoPerShot: c_int,	//how much "ammo" each shot takes
    pub iHealth: c_int,		//if non-zero, projectile can be shot, takes this much damage before being destroyed
    pub fWidth: f32,		//width of traceline or bounding box of projecile (non-rotating!)
    pub fHeight: f32,		//height of traceline or bounding box of projecile (non-rotating!)
    pub iLifeTime: c_int,	//removes itself after this amount of time
    pub bExplodeOnExpire: qboolean,	//when iLifeTime is up, explodes rather than simply removing itself
}
//NOTE: this MUST stay up to date with the number of variables in the vehFields table!!!
pub const NUM_VWEAP_PARMS: c_int = 25;

//macro: VWFOFS(x) -> ((int)&(((vehWeaponInfo_t *)0)->x))
//NOTE: Field offset macro - would require std::mem::offset_of! or manual offset computation in Rust

pub const MAX_VEH_WEAPONS: c_int = 16;	//sigh... no more than 16 different vehicle weapons
pub const VEH_WEAPON_BASE: c_int = 0;
pub const VEH_WEAPON_NONE: c_int = -1;

extern "C" {
    pub static mut g_vehWeaponInfo: [vehWeaponInfo_t; 16];
    pub static mut numVehicleWeapons: c_int;
}

//===========================================================================================================
//END VEHICLE WEAPONS
//===========================================================================================================

// The maximum number of muzzles a vehicle may have.
pub const MAX_VEHICLE_MUZZLES: c_int = 10;

// The maximum number of exhausts a vehicle may have.
pub const MAX_VEHICLE_EXHAUSTS: c_int = 4;

// The maxiumum number of different weapons a vehicle may have
pub const MAX_VEHICLE_WEAPONS: c_int = 2;
pub const MAX_VEHICLE_TURRETS: c_int = 2;
pub const MAX_VEHICLE_TURRET_MUZZLES: c_int = 2;

#[repr(C)]
pub struct turretStats_t {
    pub iWeapon: c_int,	//what vehWeaponInfo index to use
    pub iDelay: c_int,		//delay between turret muzzle shots
    pub iAmmoMax: c_int,	//how much ammo it has
    pub iAmmoRechargeMS: c_int,	//how many MS between every point of recharged ammo
    pub yawBone: *mut c_char,	//bone on ship that this turret uses to yaw
    pub pitchBone: *mut c_char,	//bone on ship that this turret uses to pitch
    pub yawAxis: c_int,	//axis on yawBone to which we should to apply the yaw angles
    pub pitchAxis: c_int,	//axis on pitchBone to which we should to apply the pitch angles
    pub yawClampLeft: f32,	//how far the turret is allowed to turn left
    pub yawClampRight: f32,	//how far the turret is allowed to turn right
    pub pitchClampUp: f32,	//how far the turret is allowed to title up
    pub pitchClampDown: f32, //how far the turret is allowed to tilt down
    pub iMuzzle: [c_int; 2],//iMuzzle-1 = index of ship's muzzle to fire this turret's 1st and 2nd shots from
    pub gunnerViewTag: *mut c_char,//Where to put the view origin of the gunner (name)
    pub fTurnSpeed: f32,	//how quickly the turret can turn
    pub bAI: qboolean,	//whether or not the turret auto-targets enemies when it's not manned
    pub bAILead: qboolean,//whether
    pub fAIRange: f32,	//how far away the AI will look for enemies
    pub passengerNum: c_int,//which passenger, if any, has control of this turret (overrides AI)
}

#[repr(C)]
pub struct vehWeaponStats_t {
//*** IMPORTANT!!! *** See note at top of next structure!!! ***
    // Weapon stuff.
    pub ID: c_int,//index into the weapon data
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

#[repr(C)]
pub struct vehicleInfo_t {
//*** IMPORTANT!!! *** vehFields table correponds to this structure!
    pub name: *mut c_char,	//unique name of the vehicle

    //general data
    pub r#type: vehicleType_t,	//what kind of vehicle
    pub numHands: c_int,	//if 2 hands, no weapons, if 1 hand, can use 1-handed weapons, if 0 hands, can use 2-handed weapons
    pub lookPitch: f32,	//How far you can look up and down off the forward of the vehicle
    pub lookYaw: f32,	//How far you can look left and right off the forward of the vehicle
    pub length: f32,		//how long it is - used for body length traces when turning/moving?
    pub width: f32,		//how wide it is - used for body length traces when turning/moving?
    pub height: f32,		//how tall it is - used for body length traces when turning/moving?
    pub centerOfGravity: vec3_t,//offset from origin: {forward, right, up} as a modifier on that dimension (-1.0f is all the way back, 1.0f is all the way forward)

    //speed stats
    pub speedMax: f32,		//top speed
    pub turboSpeed: f32,		//turbo speed
    pub speedMin: f32,		//if < 0, can go in reverse
    pub speedIdle: f32,		//what speed it drifts to when no accel/decel input is given
    pub accelIdle: f32,		//if speedIdle > 0, how quickly it goes up to that speed
    pub acceleration: f32,	//when pressing on accelerator
    pub decelIdle: f32,		//when giving no input, how quickly it drops to speedIdle
    pub throttleSticks: f32,	//if true, speed stays at whatever you accel/decel to, unless you turbo or brake
    pub strafePerc: f32,		//multiplier on current speed for strafing.  If 1.0f, you can strafe at the same speed as you're going forward, 0.5 is half, 0 is no strafing

    //handling stats
    pub bankingSpeed: f32,	//how quickly it pitches and rolls (not under player control)
    pub rollLimit: f32,		//how far it can roll to either side
    pub pitchLimit: f32,		//how far it can roll forward or backward
    pub braking: f32,		//when pressing on decelerator
    pub mouseYaw: f32,		// The mouse yaw override.
    pub mousePitch: f32,		// The mouse pitch override.
    pub turningSpeed: f32,	//how quickly you can turn
    pub turnWhenStopped: qboolean,//whether or not you can turn when not moving
    pub traction: f32,		//how much your command input affects velocity
    pub friction: f32,		//how much velocity is cut on its own
    pub maxSlope: f32,		//the max slope that it can go up with control
    pub speedDependantTurning: qboolean,//vehicle turns faster the faster it's going

    //durability stats
    pub mass: c_int,			//for momentum and impact force (player mass is 10)
    pub armor: c_int,			//total points of damage it can take
    pub shields: c_int,		//energy shield damage points
    pub shieldRechargeMS: c_int,//energy shield milliseconds per point recharged
    pub toughness: f32,		//modifies incoming damage, 1.0 is normal, 0.5 is half, etc.  Simulates being made of tougher materials/construction
    pub malfunctionArmorLevel: c_int,//when armor drops to or below this point, start malfunctioning
    pub surfDestruction: c_int, //can parts of this thing be torn off on impact? -rww

    //individual "area" health -rww
    pub health_front: c_int,
    pub health_back: c_int,
    pub health_right: c_int,
    pub health_left: c_int,

    //visuals & sounds
    pub model: *mut c_char,			//what model to use - if make it an NPC's primary model, don't need this?
    pub skin: *mut c_char,			//what skin to use - if make it an NPC's primary model, don't need this?
    pub g2radius: c_int,		//render radius for the ghoul2 model
    pub riderAnim: c_int,		//what animation the rider uses
    pub radarIconHandle: c_int,//what icon to show on radar in MP
    pub droidNPC: *mut c_char,		//NPC to attach to *droidunit tag (if it exists in the model)

    pub soundOn: c_int,		//sound to play when get on it
    pub soundOff: c_int,		//sound to play when get off
    pub soundLoop: c_int,		//sound to loop while riding it
    pub soundTakeOff: c_int,	//sound to play when ship takes off
    pub soundEngineStart: c_int,//sound to play when ship's thrusters first activate
    pub soundSpin: c_int,		//sound to loop while spiraling out of control
    pub soundTurbo: c_int,		//sound to play when turbo/afterburner kicks in
    pub soundHyper: c_int,		//sound to play when ship lands
    pub soundLand: c_int,		//sound to play when ship lands
    pub soundFlyBy: c_int,		//sound to play when they buzz you
    pub soundFlyBy2: c_int,	//alternate sound to play when they buzz you
    pub soundShift1: c_int,	//sound to play when accelerating
    pub soundShift2: c_int,	//sound to play when accelerating
    pub soundShift3: c_int,	//sound to play when decelerating
    pub soundShift4: c_int,	//sound to play when decelerating

    pub iExhaustFX: c_int,		//exhaust effect, played from "*exhaust" bolt(s)
    pub iTurboFX: c_int,		//turbo exhaust effect, played from "*exhaust" bolt(s) when ship is in "turbo" mode
    pub iTurboStartFX: c_int,	//turbo begin effect, played from "*exhaust" bolts when "turbo" mode begins
    pub iTrailFX: c_int,		//trail effect, played from "*trail" bolt(s)
    pub iImpactFX: c_int,		//impact effect, for when it bumps into something
    pub iExplodeFX: c_int,		//explosion effect, for when it blows up (should have the sound built into explosion effect)
    pub iWakeFX: c_int,		//effect it makes when going across water
    pub iDmgFX: c_int,			//effect to play on damage from a weapon or something
    pub iArmorLowFX: c_int,	//played when armor is less than 30% of full
    pub iArmorGoneFX: c_int,	//played when on armor is completely gone

    //Weapon stats
    pub weapon: [vehWeaponStats_t; 2],

    // Which weapon a muzzle fires (has to match one of the weapons this vehicle has). So 1 would be weapon 1,
    // 2 would be weapon 2 and so on.
    pub weapMuzzle: [c_int; 10],

    //turrets (if any) on the vehicle
    pub turret: [turretStats_t; 2],

    // The max height before this ship (?) starts (auto)landing.
    pub landingHeight: f32,

    //other misc stats
    pub gravity: c_int,		//normal is 800
    pub hoverHeight: f32,	//if 0, it's a ground vehicle
    pub hoverStrength: f32,	//how hard it pushes off ground when less than hover height... causes "bounce", like shocks
    pub waterProof: qboolean,		//can drive underwater if it has to
    pub bouyancy: f32,		//when in water, how high it floats (1 is neutral bouyancy)
    pub fuelMax: c_int,		//how much fuel it can hold (capacity)
    pub fuelRate: c_int,		//how quickly is uses up fuel
    pub turboDuration: c_int,	//how long turbo lasts
    pub turboRecharge: c_int,	//how long turbo takes to recharge
    pub visibility: c_int,		//for sight alerts
    pub loudness: c_int,		//for sound alerts
    pub explosionRadius: f32,//range of explosion
    pub explosionDamage: c_int,//damage of explosion

    pub maxPassengers: c_int,	// The max number of passengers this vehicle may have (Default = 0).
    pub hideRider: qboolean,		// rider (and passengers?) should not be drawn
    pub killRiderOnDeath: qboolean,//if rider is on vehicle when it dies, they should die
    pub flammable: qboolean,		//whether or not the vehicle should catch on fire before it explodes
    pub explosionDelay: c_int,	//how long the vehicle should be on fire/dying before it explodes
    //camera stuff
    pub cameraOverride: qboolean,	//whether or not to use all of the following 3rd person camera override values
    pub cameraRange: f32,	//how far back the camera should be - normal is 80
    pub cameraVertOffset: f32,//how high over the vehicle origin the camera should be - normal is 16
    pub cameraHorzOffset: f32,//how far to left/right (negative/positive) of of the vehicle origin the camera should be - normal is 0
    pub cameraPitchOffset: f32,//a modifier on the camera's pitch (up/down angle) to the vehicle - normal is 0
    pub cameraFOV: f32,		//third person camera FOV, default is 80
    pub cameraAlpha: f32,	//fade out the vehicle to this alpha (0.1-1.0f) if it's in the way of the crosshair
    pub cameraPitchDependantVertOffset: qboolean,//use the hacky AT-ST pitch dependant vertical offset

    //NOTE: some info on what vehicle weapon to use?  Like ATST or TIE bomber or TIE fighter or X-Wing...?

//===VEH_PARM_MAX========================================================================
//*** IMPORTANT!!! *** vehFields table correponds to this structure!

//THE FOLLOWING FIELDS are not in the vehFields table because they are internal variables, not read in from the .veh file
    pub modelIndex: c_int,		//set internally, not until this vehicle is spawned into the level

    // NOTE: Please note that most of this stuff has been converted from C++ classes to generic C.
    // This part of the structure is used to simulate inheritance for vehicles. The basic idea is that all vehicle use
    // this vehicle interface since they declare their own functions and assign the function pointer to the
    // corresponding function. Meanwhile, the base logic can still call the appropriate functions. In C++ talk all
    // of these functions (pointers) are pure virtuals and this is an abstract base class (although it cannot be
    // inherited from, only contained and reimplemented (through an object and a setup function respectively)). -AReis

    // Makes sure that the vehicle is properly animated.
    pub AnimateVehicle: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // Makes sure that the rider's in this vehicle are properly animated.
    pub AnimateRiders: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // Determine whether this entity is able to board this vehicle or not.
    pub ValidateBoard: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t) -> bool>,

    // Set the parent entity of this Vehicle NPC.
    pub SetParent: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t)>,

    // Add a pilot to the vehicle.
    pub SetPilot: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t)>,

    // Add a passenger to the vehicle (false if we're full).
    pub AddPassenger: Option<unsafe extern "C" fn(*mut Vehicle_t) -> bool>,

    // Animate the vehicle and it's riders.
    pub Animate: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // Board this Vehicle (get on). The first entity to board an empty vehicle becomes the Pilot.
    pub Board: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t) -> bool>,

    // Eject an entity from the vehicle.
    pub Eject: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t, qboolean) -> bool>,

    // Eject all the inhabitants of this vehicle.
    pub EjectAll: Option<unsafe extern "C" fn(*mut Vehicle_t) -> bool>,

    // Start a delay until the vehicle dies.
    pub StartDeathDelay: Option<unsafe extern "C" fn(*mut Vehicle_t, c_int)>,

    // Update death sequence.
    pub DeathUpdate: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // Register all the assets used by this vehicle.
    pub RegisterAssets: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // Initialize the vehicle (should be called by Spawn?).
    pub Initialize: Option<unsafe extern "C" fn(*mut Vehicle_t) -> bool>,

    // Like a think or move command, this updates various vehicle properties.
    pub Update: Option<unsafe extern "C" fn(*mut Vehicle_t, *const usercmd_t) -> bool>,

    // Update the properties of a Rider (that may reflect what happens to the vehicle).
    //
    //	[return]		bool			True if still in vehicle, false if otherwise.
    pub UpdateRider: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t, *mut usercmd_t) -> bool>,

    // ProcessMoveCommands the Vehicle.
    pub ProcessMoveCommands: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // ProcessOrientCommands the Vehicle.
    pub ProcessOrientCommands: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // Attachs all the riders of this vehicle to their appropriate position/tag (*driver, *pass1, *pass2, whatever...).
    pub AttachRiders: Option<unsafe extern "C" fn(*mut Vehicle_t)>,

    // Make someone invisible and un-collidable.
    pub Ghost: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t)>,

    // Make someone visible and collidable.
    pub UnGhost: Option<unsafe extern "C" fn(*mut Vehicle_t, *mut gentity_t)>,

    // Get the pilot of this vehicle.
    pub GetPilot: Option<unsafe extern "C" fn(*mut Vehicle_t) -> *const gentity_t>,

    // Whether this vehicle is currently inhabited (by anyone) or not.
    pub Inhabited: Option<unsafe extern "C" fn(*mut Vehicle_t) -> bool>,
}

//NOTE: macro VFOFS(x) -> ((int)&(((vehicleInfo_t *)0)->x))
//NOTE: Field offset macro - would require std::mem::offset_of! or manual offset computation in Rust

pub const MAX_VEHICLES: c_int = 16;	//sigh... no more than 64 individual vehicles
extern "C" {
    pub static mut g_vehicleInfo: [vehicleInfo_t; 16];
    pub static mut numVehicles: c_int;
}

// Load the function pointers for a vehicle into this shared vehicle info structure.
extern "C" {
    pub fn G_SetSpeederVehicleFunctions(pVehInfo: *mut vehicleInfo_t);
    pub fn G_SetAnimalVehicleFunctions(pVehInfo: *mut vehicleInfo_t);
    pub fn G_SetFighterVehicleFunctions(pVehInfo: *mut vehicleInfo_t);
    pub fn G_SetWalkerVehicleFunctions(pVehInfo: *mut vehicleInfo_t);
}

// Setup the shared functions (one's that all vehicles would generally use).
extern "C" {
    pub fn G_SetSharedVehicleFunctions(pVehInfo: *mut vehicleInfo_t);
}

// Create/Allocate a new Animal Vehicle (initializing it as well).
extern "C" {
    pub fn G_CreateSpeederNPC(pVeh: *mut *mut Vehicle_t, strType: *const c_char);
    pub fn G_CreateAnimalNPC(pVeh: *mut *mut Vehicle_t, strType: *const c_char);
    pub fn G_CreateFighterNPC(pVeh: *mut *mut Vehicle_t, strType: *const c_char);
    pub fn G_CreateWalkerNPC(pVeh: *mut *mut Vehicle_t, strType: *const c_char);
}

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
pub const VEH_MAX_PASSENGERS: c_int = 10;

pub const VEH_MOUNT_THROW_LEFT: c_int = -5;
pub const VEH_MOUNT_THROW_RIGHT: c_int = -6;

pub const MAX_STRAFE_TIME: f32 = 2000.0;//FIXME: extern?
pub const MIN_LANDING_SPEED: c_int = 200;//equal to or less than this and close to ground = auto-slow-down to land
pub const MIN_LANDING_SLOPE: f32 = 0.8;//must be pretty flat to land on the surf

pub const VEHICLE_BASE: c_int = 0;
pub const VEHICLE_NONE: c_int = -1;

pub mod VEH_EJECT {
    pub const VEH_EJECT_LEFT: c_int = 0;
    pub const VEH_EJECT_RIGHT: c_int = 1;
    pub const VEH_EJECT_FRONT: c_int = 2;
    pub const VEH_EJECT_REAR: c_int = 3;
    pub const VEH_EJECT_TOP: c_int = 4;
    pub const VEH_EJECT_BOTTOM: c_int = 5;
}

// Vehicle flags.
pub mod VEH_FLAGS {
    use core::ffi::c_int;
    pub const VEH_NONE: c_int = 0;
    pub const VEH_FLYING: c_int = 0x00000001;
    pub const VEH_CRASHING: c_int = 0x00000002;
    pub const VEH_LANDING: c_int = 0x00000004;
    pub const VEH_BUCKING: c_int = 0x00000010;
    pub const VEH_WINGSOPEN: c_int = 0x00000020;
    pub const VEH_GEARSOPEN: c_int = 0x00000040;
    pub const VEH_SLIDEBREAKING: c_int = 0x00000080;
    pub const VEH_SPINNING: c_int = 0x00000100;
    pub const VEH_OUTOFCONTROL: c_int = 0x00000200;
    pub const VEH_SABERINLEFTHAND: c_int = 0x00000400;
    pub const VEH_STRAFERAM: c_int = 0x00000800;
    pub const VEH_ACCELERATORON: c_int = 0x00001000;
    pub const VEH_ARMORLOW: c_int = 0x00002000;
    pub const VEH_ARMORGONE: c_int = 0x00004000;
}
//externed functions
//extern void G_DriveVehicle( gentity_t *ent, gentity_t *vehEnt, char *vehicleName );
/*extern void G_VehicleStartExplosionDelay( gentity_t *self );
extern void VehicleExplosionDelay( gentity_t *self );
extern void G_VehicleRegisterAssets( int vehicleIndex );
extern void G_DriveATST( gentity_t *ent, gentity_t *atst );
extern void G_VehicleInitialize( gentity_t *vehEnt );*/
extern "C" {
    pub fn G_VehicleSpawn(self_: *mut gentity_t);
}

// A vehicle weapon muzzle.
#[repr(C)]
pub struct Muzzle {
    // These are updated every frame and represent the current position and direction for the specific muzzle.
    pub m_vMuzzlePos: vec3_t,
    pub m_vMuzzleDir: vec3_t,

    // This is how long to wait before being able to fire a specific muzzle again. This is based on the firing rate
    // so that a firing rate of 10 rounds/sec would make this value initially 100 miliseconds.
    pub m_iMuzzleWait: c_int,

    // whether this Muzzle was just fired or not (reset at muzzle flash code).
    pub m_bFired: bool,
}

//defines for impact damage surface stuff
pub const SHIPSURF_FRONT: c_int = 1;
pub const SHIPSURF_BACK: c_int = 2;
pub const SHIPSURF_RIGHT: c_int = 3;
pub const SHIPSURF_LEFT: c_int = 4;

pub const SHIPSURF_DAMAGE_FRONT_LIGHT: c_int = 1;
pub const SHIPSURF_DAMAGE_BACK_LIGHT: c_int = 2;
pub const SHIPSURF_DAMAGE_RIGHT_LIGHT: c_int = 3;
pub const SHIPSURF_DAMAGE_LEFT_LIGHT: c_int = 4;
pub const SHIPSURF_DAMAGE_FRONT_HEAVY: c_int = 5;
pub const SHIPSURF_DAMAGE_BACK_HEAVY: c_int = 6;
pub const SHIPSURF_DAMAGE_RIGHT_HEAVY: c_int = 7;
pub const SHIPSURF_DAMAGE_LEFT_HEAVY: c_int = 8;

//generic part bits
pub const SHIPSURF_BROKEN_A: c_int = 1 << 0; //gear 1
pub const SHIPSURF_BROKEN_B: c_int = 1 << 1; //gear 1
pub const SHIPSURF_BROKEN_C: c_int = 1 << 2; //wing 1
pub const SHIPSURF_BROKEN_D: c_int = 1 << 3; //wing 2
pub const SHIPSURF_BROKEN_E: c_int = 1 << 4; //wing 3
pub const SHIPSURF_BROKEN_F: c_int = 1 << 5; //wing 4

#[repr(C)]
pub struct vehWeaponStatus_t {
    //linked firing mode
    pub linked: qboolean,//weapon 1's muzzles are in linked firing mode
    //current weapon ammo
    pub ammo: c_int,
    //debouncer for ammo recharge
    pub lastAmmoInc: c_int,
    //which muzzle will fire next
    pub nextMuzzle: c_int,
}

#[repr(C)]
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

// This is the implementation of the vehicle interface and any of the other variables needed. This
// is what actually represents a vehicle. -AReis.
// !!!!!!!!!!!!!!!!!! loadsave affecting structure !!!!!!!!!!!!!!!!!!!!!!!
#[repr(C)]
pub struct Vehicle_t {
    // The entity who pilots/drives this vehicle.
    // NOTE: This is redundant (since m_pParentEntity->owner _should_ be the pilot). This makes things clearer though.
    pub m_pPilot: *mut gentity_t,

    pub m_iPilotTime: c_int, //if spawnflag to die without pilot and this < level.time then die.
    pub m_bHasHadPilot: qboolean, //qtrue once the vehicle gets its first pilot

    //the droid unit NPC for this vehicle, if any
    pub m_pDroidUnit: *mut gentity_t,

    // The entity from which this NPC comes from.
    pub m_pParentEntity: *mut gentity_t,

    // If not zero, how long to wait before we can do anything with the vehicle (we're getting on still).
    // -1 = board from left, -2 = board from right, -3 = jump/quick board.  -4 & -5 = throw off existing pilot
    pub m_iBoarding: c_int,

    // Used to check if we've just started the boarding process
    pub m_bWasBoarding: bool,

    // The speed the vehicle maintains while boarding occurs (often zero)
    pub m_vBoardingVelocity: vec3_t,

    // Time modifier (must only be used in ProcessMoveCommands() and ProcessOrientCommands() and is updated in Update()).
    pub m_fTimeModifier: f32,

    // Ghoul2 Animation info.
    // NOTE: Since each vehicle has their own model instance, these bolts must be local to each vehicle as well.
    pub m_iLeftWingBone: c_int,
    pub m_iRightWingBone: c_int,
    //int m_iDriverTag;
    pub m_iExhaustTag: [c_int; 4],
    pub m_iMuzzleTag: [c_int; 10],
    pub m_iDroidUnitTag: c_int,
    pub m_iGunnerViewTag: [c_int; 2],//Where to put the view origin of the gunner (index)

    // This vehicles weapon muzzles.
    pub m_Muzzles: [Muzzle; 10],

    // The user commands structure.
    pub m_ucmd: usercmd_t,

    // The direction an entity will eject from the vehicle towards.
    pub m_EjectDir: c_int,

    // Flags that describe the vehicles behavior.
    pub m_ulFlags: c_int,

    // NOTE: Vehicle Type ID, Orientation, and Armor MUST be transmitted over the net.

    // Current angles of this vehicle.
    pub m_vOrientation: vec3_t,

    // How long you have strafed left or right (increments every frame that you strafe to right, decrements every frame you strafe left)
    pub m_fStrafeTime: c_int,

    // Previous angles of this vehicle.
    pub m_vPrevOrientation: vec3_t,

    // When control is lost on a speeder, current angular velocity is stored here and applied until landing
    pub m_vAngularVelocity: f32,

    pub m_vFullAngleVelocity: vec3_t,

    // Current armor and shields of your vehicle (explodes if armor to 0).
    pub m_iArmor: c_int,	//hull strength - STAT_HEALTH on NPC
    pub m_iShields: c_int,	//energy shielding - STAT_ARMOR on NPC

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

    //bitflag of surfaces that have broken off
    pub m_iRemovedSurfaces: c_int,

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
    pub weaponStatus: [vehWeaponStatus_t; 2],
    pub turretStatus: [vehTurretStatus_t; 2],

    //the guy who was previously the pilot
    pub m_pOldPilot: *mut gentity_t,

    // don't need these in mp
    pub m_safeJumpMountTime: c_int,
    pub m_safeJumpMountRightDot: f32,
}

extern "C" {
    pub fn BG_VehicleGetIndex(vehicleName: *const c_char) -> c_int;
}
