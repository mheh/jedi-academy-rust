// Port of oracle/code/cgame/cg_media.h
//
// cg_media.h carries no #include directives of its own; it relies on the
// including translation unit having already pulled in q_shared.h, cg_local.h,
// etc.  Accordingly this module declares NO use imports derived from includes.
// External types (vec4_t, qhandle_t, sfxHandle_t, ffHandle_t, fxHandle_t,
// gameState_t, glconfig_t, clientInfo_t, vec3_t) and constants
// (MAX_QPATH, MAX_MODELS, MAX_SOUNDS, MAX_FORCES, MAX_CHARSKINS,
// MAX_SUBMODELS, MAX_CLIENTS) are trusted to be in scope from the crate graph.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::{c_char, c_int};

pub const NUM_CROSSHAIRS: usize = 9;

#[repr(C)]
pub enum footstep_t {
    FOOTSTEP_STONEWALK,
    FOOTSTEP_STONERUN,
    FOOTSTEP_METALWALK,
    FOOTSTEP_METALRUN,
    FOOTSTEP_PIPEWALK,
    FOOTSTEP_PIPERUN,
    FOOTSTEP_SPLASH,
    FOOTSTEP_WADE,
    FOOTSTEP_SWIM,
    FOOTSTEP_SNOWWALK,
    FOOTSTEP_SNOWRUN,
    FOOTSTEP_SANDWALK,
    FOOTSTEP_SANDRUN,
    FOOTSTEP_GRASSWALK,
    FOOTSTEP_GRASSRUN,
    FOOTSTEP_DIRTWALK,
    FOOTSTEP_DIRTRUN,
    FOOTSTEP_MUDWALK,
    FOOTSTEP_MUDRUN,
    FOOTSTEP_GRAVELWALK,
    FOOTSTEP_GRAVELRUN,
    FOOTSTEP_RUGWALK,
    FOOTSTEP_RUGRUN,
    FOOTSTEP_WOODWALK,
    FOOTSTEP_WOODRUN,

    FOOTSTEP_TOTAL,
}

pub const ICON_WEAPONS: c_int = 0;
pub const ICON_FORCE: c_int = 1;
pub const ICON_INVENTORY: c_int = 2;

pub const MAX_HUD_TICS: usize = 4;


#[repr(C)]
pub struct HUDMenuItem_s
{
    pub menuName: *mut c_char,
    pub itemName: *mut c_char,
    pub xPos: c_int,
    pub yPos: c_int,
    pub width: c_int,
    pub height: c_int,
    pub color: vec4_t,
    pub background: qhandle_t,
}
pub type HUDMenuItem_t = HUDMenuItem_s;

extern "C" {
    pub static mut healthTics: [HUDMenuItem_t; 0];
    pub static mut armorTics: [HUDMenuItem_t; 0];
    pub static mut ammoTics: [HUDMenuItem_t; 0];
    pub static mut forceTics: [HUDMenuItem_t; 0];
    pub static mut otherHUDBits: [HUDMenuItem_t; 0];
}


#[repr(C)]
pub enum otherhudbits_t
{
    OHB_HEALTHAMOUNT = 0,
    OHB_ARMORAMOUNT,
    OHB_FORCEAMOUNT,
    OHB_AMMOAMOUNT,
    OHB_SABERSTYLE_STRONG,
    OHB_SABERSTYLE_MEDIUM,
    OHB_SABERSTYLE_FAST,
    OHB_SCANLINE_LEFT,
    OHB_SCANLINE_RIGHT,
    OHB_FRAME_LEFT,
    OHB_FRAME_RIGHT,
    OHB_MAX,
}

pub const NUM_CHUNK_MODELS: usize = 4;

// Porting note: C source has an anonymous `typedef enum { ... };` with no
// type alias name.  Synthetic name `chunk_type_e` was added to satisfy Rust
// syntax.  Variants are re-exported at module scope with `pub use
// chunk_type_e::*` to match C's global-scope enum constant semantics.
#[repr(C)]
pub enum chunk_type_e
{
    CHUNK_METAL1 = 0,
    CHUNK_METAL2,
    CHUNK_ROCK1,
    CHUNK_ROCK2,
    CHUNK_ROCK3,
    CHUNK_CRATE1,
    CHUNK_CRATE2,
    CHUNK_WHITE_METAL,
    NUM_CHUNK_TYPES,
}
pub use chunk_type_e::*;

// all of the model, shader, and sound references that are
// loaded at gamestate time are stored in cgMedia_t
// Other media that can be tied to clients, weapons, or items are
// stored in the clientInfo_t, itemInfo_t, weaponInfo_t, and powerupInfo_t
#[repr(C)]
pub struct cgMedia_t {
    pub charsetShader: qhandle_t,
    pub whiteShader: qhandle_t,

    pub crosshairShader: [qhandle_t; NUM_CROSSHAIRS],
    pub backTileShader: qhandle_t,
//	pub noammoShader: qhandle_t,

    pub numberShaders: [qhandle_t; 11],
    pub smallnumberShaders: [qhandle_t; 11],
    pub chunkyNumberShaders: [qhandle_t; 11],

    pub loadTick: qhandle_t,
    pub loadTickCap: qhandle_t,

    //			HUD artwork
    pub currentBackground: c_int,
    pub weaponbox: qhandle_t,
    pub weaponIconBackground: qhandle_t,
    pub forceIconBackground: qhandle_t,
    pub inventoryIconBackground: qhandle_t,
    pub turretComputerOverlayShader: qhandle_t,
    pub turretCrossHairShader: qhandle_t,

//Chunks
    pub chunkModels: [[qhandle_t; 4]; chunk_type_e::NUM_CHUNK_TYPES as usize],
    pub chunkSound: sfxHandle_t,
    pub grateSound: sfxHandle_t,
    pub rockBreakSound: sfxHandle_t,
    pub rockBounceSound: [sfxHandle_t; 2],
    pub metalBounceSound: [sfxHandle_t; 2],
    pub glassChunkSound: sfxHandle_t,
    pub crateBreakSound: [sfxHandle_t; 2],

    // Saber shaders
    //-----------------------------
    pub forceCoronaShader: qhandle_t,
    pub saberBlurShader: qhandle_t,
    pub swordTrailShader: qhandle_t,
    pub yellowDroppedSaberShader: qhandle_t, // glow

    pub redSaberGlowShader: qhandle_t,
    pub redSaberCoreShader: qhandle_t,
    pub orangeSaberGlowShader: qhandle_t,
    pub orangeSaberCoreShader: qhandle_t,
    pub yellowSaberGlowShader: qhandle_t,
    pub yellowSaberCoreShader: qhandle_t,
    pub greenSaberGlowShader: qhandle_t,
    pub greenSaberCoreShader: qhandle_t,
    pub blueSaberGlowShader: qhandle_t,
    pub blueSaberCoreShader: qhandle_t,
    pub purpleSaberGlowShader: qhandle_t,
    pub purpleSaberCoreShader: qhandle_t,

    pub explosionModel: qhandle_t,
    pub surfaceExplosionShader: qhandle_t,

    pub halfShieldModel: qhandle_t,

    pub solidWhiteShader: qhandle_t,
    pub electricBodyShader: qhandle_t,
    pub electricBody2Shader: qhandle_t,
    pub refractShader: qhandle_t,
    pub boltShader: qhandle_t,

    // Disruptor zoom graphics
    pub disruptorMask: qhandle_t,
    pub disruptorInsert: qhandle_t,
    pub disruptorLight: qhandle_t,
    pub disruptorInsertTick: qhandle_t,

    // Binocular graphics
    pub binocularCircle: qhandle_t,
    pub binocularMask: qhandle_t,
    pub binocularArrow: qhandle_t,
    pub binocularTri: qhandle_t,
    pub binocularStatic: qhandle_t,
    pub binocularOverlay: qhandle_t,

    // LA Goggles graphics
    pub laGogglesStatic: qhandle_t,
    pub laGogglesMask: qhandle_t,
    pub laGogglesSideBit: qhandle_t,
    pub laGogglesBracket: qhandle_t,
    pub laGogglesArrow: qhandle_t,

    // wall mark shaders
    pub scavMarkShader: qhandle_t,
    pub rivetMarkShader: qhandle_t,

    pub shadowMarkShader: qhandle_t,
    pub wakeMarkShader: qhandle_t,
    pub fsrMarkShader: qhandle_t,
    pub fslMarkShader: qhandle_t,
    pub fshrMarkShader: qhandle_t,
    pub fshlMarkShader: qhandle_t,

    pub damageBlendBlobShader: qhandle_t,

    // fonts...
    //
    pub qhFontSmall: qhandle_t,
    pub qhFontMedium: qhandle_t,

    // special effects models / etc.
    pub personalShieldShader: qhandle_t,
    pub cloakedShader: qhandle_t,

    // Interface media
    pub ammoslider: qhandle_t,
    pub emplacedHealthBarShader: qhandle_t,

    pub dataPadFrame: qhandle_t,
    pub DPForcePowerOverlay: qhandle_t,

    pub bdecal_burnmark1: qhandle_t,
    pub bdecal_saberglowmark: qhandle_t,

    pub messageLitOn: qhandle_t,
    pub messageLitOff: qhandle_t,
    pub messageObjCircle: qhandle_t,

    pub batteryChargeShader: qhandle_t,
    pub useableHint: qhandle_t,

    pub levelLoad: qhandle_t,

    //new stuff for Jedi Academy
    //force power icons
//	pub forcePowerIcons: [qhandle_t; NUM_FORCE_POWERS],
    pub rageRecShader: qhandle_t,
    pub playerShieldDamage: qhandle_t,
    pub forceSightBubble: qhandle_t,
    pub forceShell: qhandle_t,
    pub sightShell: qhandle_t,
    pub drainShader: qhandle_t,

    // sounds
    pub disintegrateSound: sfxHandle_t,
    pub disintegrate2Sound: sfxHandle_t,
    pub disintegrate3Sound: sfxHandle_t,

    pub grenadeBounce1: sfxHandle_t,
    pub grenadeBounce2: sfxHandle_t,

    pub flechetteStickSound: sfxHandle_t,
    pub detPackStickSound: sfxHandle_t,
    pub tripMineStickSound: sfxHandle_t,

    pub selectSound: sfxHandle_t,
    pub selectSound2: sfxHandle_t,
    pub overchargeSlowSound: sfxHandle_t,
    pub overchargeFastSound: sfxHandle_t,
    pub overchargeLoopSound: sfxHandle_t,
    pub overchargeEndSound: sfxHandle_t,

//	pub useNothingSound: sfxHandle_t,
    pub footsteps: [[sfxHandle_t; 4]; footstep_t::FOOTSTEP_TOTAL as usize],

//	pub talkSound: sfxHandle_t,
    pub noAmmoSound: sfxHandle_t,

    pub landSound: sfxHandle_t,
    pub rollSound: sfxHandle_t,
    pub messageLitSound: sfxHandle_t,

    pub batteryChargeSound: sfxHandle_t,

    pub watrInSound: sfxHandle_t,
    pub watrOutSound: sfxHandle_t,
    pub watrUnSound: sfxHandle_t,

    pub lavaInSound: sfxHandle_t,
    pub lavaOutSound: sfxHandle_t,
    pub lavaUnSound: sfxHandle_t,

    pub noforceSound: sfxHandle_t,

    // Zoom
    pub zoomStart: sfxHandle_t,
    pub zoomLoop: sfxHandle_t,
    pub zoomEnd: sfxHandle_t,
    pub disruptorZoomLoop: sfxHandle_t,

    //new stuff for Jedi Academy
    pub drainSound: sfxHandle_t,

    //force feedback stuff
    #[cfg(feature = "immersion")]
    pub grenadeBounce1Force: ffHandle_t,
    #[cfg(feature = "immersion")]
    pub grenadeBounce2Force: ffHandle_t,

    #[cfg(feature = "immersion")]
    pub selectForce: ffHandle_t,

    #[cfg(feature = "immersion")]
    pub footstepForces: [[ffHandle_t; 4]; footstep_t::FOOTSTEP_TOTAL as usize],

    #[cfg(feature = "immersion")]
    pub noAmmoForce: ffHandle_t,

    #[cfg(feature = "immersion")]
    pub landForce: ffHandle_t,
    #[cfg(feature = "immersion")]
    pub messageLitForce: ffHandle_t,

    #[cfg(feature = "immersion")]
    pub watrInForce: ffHandle_t,
    #[cfg(feature = "immersion")]
    pub watrOutForce: ffHandle_t,
    #[cfg(feature = "immersion")]
    pub watrUnForce: ffHandle_t,

    #[cfg(feature = "immersion")]
    pub zoomStartForce: ffHandle_t,
    #[cfg(feature = "immersion")]
    pub zoomLoopForce: ffHandle_t,
    #[cfg(feature = "immersion")]
    pub zoomEndForce: ffHandle_t,
    #[cfg(feature = "immersion")]
    pub disruptorZoomLoopForce: ffHandle_t,
    // #endif // _IMMERSION
}


// Stored FX handles
//--------------------
#[repr(C)]
pub struct cgEffects_t
{
    // BRYAR PISTOL
    pub bryarShotEffect: fxHandle_t,
    pub bryarPowerupShotEffect: fxHandle_t,
    pub bryarWallImpactEffect: fxHandle_t,
    pub bryarWallImpactEffect2: fxHandle_t,
    pub bryarWallImpactEffect3: fxHandle_t,
    pub bryarFleshImpactEffect: fxHandle_t,

    // BLASTER
    pub blasterShotEffect: fxHandle_t,
    pub blasterOverchargeEffect: fxHandle_t,
    pub blasterWallImpactEffect: fxHandle_t,
    pub blasterFleshImpactEffect: fxHandle_t,

    // BOWCASTER
    pub bowcasterShotEffect: fxHandle_t,
    pub bowcasterBounceEffect: fxHandle_t,
    pub bowcasterImpactEffect: fxHandle_t,

    // FLECHETTE
    pub flechetteShotEffect: fxHandle_t,
    pub flechetteAltShotEffect: fxHandle_t,
    pub flechetteShotDeathEffect: fxHandle_t,
    pub flechetteFleshImpactEffect: fxHandle_t,
    pub flechetteRicochetEffect: fxHandle_t,

    //FORCE
    pub forceConfusion: fxHandle_t,
    pub forceLightning: fxHandle_t,
    pub forceLightningWide: fxHandle_t,
    //pub forceInvincibility: fxHandle_t,
    pub forceHeal: fxHandle_t,

    //new stuff for Jedi Academy
    pub forceDrain: fxHandle_t,
    pub forceDrainWide: fxHandle_t,
    pub forceDrained: fxHandle_t,

    //footstep effects
    pub footstepMud: fxHandle_t,
    pub footstepSand: fxHandle_t,
    pub footstepSnow: fxHandle_t,
    pub footstepGravel: fxHandle_t,
    //landing effects
    pub landingMud: fxHandle_t,
    pub landingSand: fxHandle_t,
    pub landingDirt: fxHandle_t,
    pub landingSnow: fxHandle_t,
    pub landingGravel: fxHandle_t,
}


// The client game static (cgs) structure hold everything
// loaded or calculated from the gamestate.  It will NOT
// be cleared when a tournement restart is done, allowing
// all clients to begin playing instantly
pub const STRIPED_LEVELNAME_VARIATIONS: usize = 3;	// sigh, to cope with levels that use text from >1 SP file (plus 1 for common)

#[repr(C)]
pub struct cgs_t {
    pub gameState: gameState_t,			// gamestate from server
    pub glconfig: glconfig_t,			// rendering configuration

    pub serverCommandSequence: c_int,	// reliable command stream counter

    // parsed from serverinfo
    pub dmflags: c_int,
    pub teamflags: c_int,
    pub timelimit: c_int,
    pub maxclients: c_int,
    pub mapname: [c_char; MAX_QPATH],
    pub stripLevelName: [[c_char; MAX_QPATH]; STRIPED_LEVELNAME_VARIATIONS],

    //
    // locally derived information from gamestate
    //
    pub model_draw: [qhandle_t; MAX_MODELS],
    pub sound_precache: [sfxHandle_t; MAX_SOUNDS],
    #[cfg(feature = "immersion")]
    pub force_precache: [ffHandle_t; MAX_FORCES],
    // Ghoul2 start
    pub skins: [qhandle_t; MAX_CHARSKINS],

    // Ghoul2 end

    pub numInlineModels: c_int,
    pub inlineDrawModel: [qhandle_t; MAX_SUBMODELS],
    pub inlineModelMidpoints: [vec3_t; MAX_SUBMODELS],

    pub clientinfo: [clientInfo_t; MAX_CLIENTS],

    // media
    pub media: cgMedia_t,

    // effects
    pub effects: cgEffects_t,
}

extern "C" {
    pub static mut cgs: cgs_t;
}
