//! Inventory, model, and weapon index macros from `inv.h`.

use core::ffi::c_int;

pub const INVENTORY_NONE: c_int = 0;
// pickups
pub const INVENTORY_ARMOR: c_int = 1;
pub const INVENTORY_HEALTH: c_int = 2;
// items
pub const INVENTORY_SEEKER: c_int = 3;
pub const INVENTORY_MEDPAC: c_int = 4;
pub const INVENTORY_DATAPAD: c_int = 5;
pub const INVENTORY_BINOCULARS: c_int = 6;
pub const INVENTORY_SENTRY_GUN: c_int = 7;
pub const INVENTORY_GOGGLES: c_int = 8;
// weapons
pub const INVENTORY_STUN_BATON: c_int = 9;
pub const INVENTORY_SABER: c_int = 10;
pub const INVENTORY_BRYAR_PISTOL: c_int = 11;
pub const INVENTORY_BLASTER: c_int = 12;
pub const INVENTORY_DISRUPTOR: c_int = 13;
pub const INVENTORY_BOWCASTER: c_int = 14;
pub const INVENTORY_REPEATER: c_int = 15;
pub const INVENTORY_DEMP2: c_int = 16;
pub const INVENTORY_FLECHETTE: c_int = 17;
pub const INVENTORY_ROCKET_LAUNCHER: c_int = 18;
pub const INVENTORY_THERMAL: c_int = 19;
pub const INVENTORY_TRIP_MINE: c_int = 20;
pub const INVENTORY_DET_PACK: c_int = 21;
// ammo
pub const INVENTORY_AMMO_FORCE: c_int = 22;
pub const INVENTORY_AMMO_BLASTER: c_int = 23;
pub const INVENTORY_AMMO_BOLTS: c_int = 24;
pub const INVENTORY_AMMO_ROCKETS: c_int = 25;
// powerups
pub const INVENTORY_REDFLAG: c_int = 26;
pub const INVENTORY_BLUEFLAG: c_int = 27;
pub const INVENTORY_SCOUT: c_int = 28;
pub const INVENTORY_GUARD: c_int = 29;
pub const INVENTORY_DOUBLER: c_int = 30;
pub const INVENTORY_AMMOREGEN: c_int = 31;
pub const INVENTORY_NEUTRALFLAG: c_int = 32;
pub const INVENTORY_REDCUBE: c_int = 33;
pub const INVENTORY_BLUECUBE: c_int = 34;

// enemy stuff
pub const ENEMY_HORIZONTAL_DIST: c_int = 200;
pub const ENEMY_HEIGHT: c_int = 201;
pub const NUM_VISIBLE_ENEMIES: c_int = 202;
pub const NUM_VISIBLE_TEAMMATES: c_int = 203;

// NOTENOTE Update this so that it is in sync.
// item numbers (make sure they are in sync with bg_itemlist in bg_misc.c)
// pickups
pub const MODELINDEX_ARMOR: c_int = 1;
pub const MODELINDEX_HEALTH: c_int = 2;
// items
pub const MODELINDEX_SEEKER: c_int = 3;
pub const MODELINDEX_MEDPAC: c_int = 4;
pub const MODELINDEX_DATAPAD: c_int = 5;
pub const MODELINDEX_BINOCULARS: c_int = 6;
pub const MODELINDEX_SENTRY_GUN: c_int = 7;
pub const MODELINDEX_GOGGLES: c_int = 8;
// weapons
pub const MODELINDEX_STUN_BATON: c_int = 9;
pub const MODELINDEX_SABER: c_int = 10;
pub const MODELINDEX_BRYAR_PISTOL: c_int = 11;
pub const MODELINDEX_BLASTER: c_int = 12;
pub const MODELINDEX_DISRUPTOR: c_int = 13;
pub const MODELINDEX_BOWCASTER: c_int = 14;
pub const MODELINDEX_REPEATER: c_int = 15;
pub const MODELINDEX_DEMP2: c_int = 16;
pub const MODELINDEX_FLECHETTE: c_int = 17;
pub const MODELINDEX_ROCKET_LAUNCHER: c_int = 18;
pub const MODELINDEX_THERMAL: c_int = 19;
pub const MODELINDEX_TRIP_MINE: c_int = 20;
pub const MODELINDEX_DET_PACK: c_int = 21;
// ammo
pub const MODELINDEX_AMMO_FORCE: c_int = 22;
pub const MODELINDEX_AMMO_BLASTER: c_int = 23;
pub const MODELINDEX_AMMO_BOLTS: c_int = 24;
pub const MODELINDEX_AMMO_ROCKETS: c_int = 25;
// powerups
pub const MODELINDEX_REDFLAG: c_int = 26;
pub const MODELINDEX_BLUEFLAG: c_int = 27;
pub const MODELINDEX_SCOUT: c_int = 28;
pub const MODELINDEX_GUARD: c_int = 29;
pub const MODELINDEX_DOUBLER: c_int = 30;
pub const MODELINDEX_AMMOREGEN: c_int = 31;
pub const MODELINDEX_NEUTRALFLAG: c_int = 32;
pub const MODELINDEX_REDCUBE: c_int = 33;
pub const MODELINDEX_BLUECUBE: c_int = 34;

//
pub const WEAPONINDEX_STUN_BATON: c_int = 1;
pub const WEAPONINDEX_SABER: c_int = 2;
pub const WEAPONINDEX_BRYAR_PISTOL: c_int = 3;
pub const WEAPONINDEX_BLASTER: c_int = 4;
pub const WEAPONINDEX_DISRUPTOR: c_int = 5;
pub const WEAPONINDEX_BOWCASTER: c_int = 6;
pub const WEAPONINDEX_REPEATER: c_int = 7;
pub const WEAPONINDEX_DEMP2: c_int = 8;
pub const WEAPONINDEX_FLECHETTE: c_int = 9;
pub const WEAPONINDEX_ROCKET_LAUNCHER: c_int = 10;
pub const WEAPONINDEX_THERMAL: c_int = 11;
pub const WEAPONINDEX_TRIP_MINE: c_int = 12;
pub const WEAPONINDEX_DET_PACK: c_int = 13;

