//
// cg_weaponinit.c -- events and effects dealing with weapons

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};
use crate::codemp::game::bg_public::{gitem_t, MAX_ITEM_MODELS, IT_WEAPON, IT_AMMO};
use crate::codemp::game::q_shared_h::{vec3_t, MAX_QPATH, qboolean};

// ============================================================================
// Stubs for external types and functions
// ============================================================================

/// Stub for weaponInfo_t: weapon rendering/sound metadata.
#[repr(C)]
pub struct weaponInfo_t {
    pub registered: qboolean,
    pub weaponModel: c_int,
    pub viewModel: c_int,
    pub weaponMidpoint: vec3_t,
    pub weaponIcon: c_int,
    pub ammoIcon: c_int,
    pub ammoModel: c_int,
    pub flashModel: c_int,
    pub barrelModel: c_int,
    pub handsModel: c_int,
    pub flashDlightColor: vec3_t,
    pub firingSound: c_int,
    pub altFiringSound: c_int,
    pub flashSound: [c_int; 4],
    pub altFlashSound: [c_int; 4],
    pub chargeSound: c_int,
    pub altChargeSound: c_int,
    pub selectSound: c_int,
    pub muzzleEffect: c_int,
    pub altMuzzleEffect: c_int,
    pub missileModel: c_int,
    pub altMissileModel: c_int,
    pub missileSound: c_int,
    pub altMissileSound: c_int,
    pub missileTrailFunc: *const c_void,
    pub altMissileTrailFunc: *const c_void,
    pub missileDlight: c_int,
    pub missileDlightColor: vec3_t,
    pub altMissileDlight: c_int,
    pub altMissileDlightColor: vec3_t,
    pub missileHitSound: c_int,
    pub altMissileHitSound: c_int,
    pub item: *const gitem_t,
}

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 76],
    pub bowcasterShotEffect: c_int,
    pub bowcasterImpactEffect: c_int,
    pub _pad1: [u8; 100],
    pub bryarShotEffect: c_int,
    pub bryarPowerupShotEffect: c_int,
    pub bryarWallImpactEffect: c_int,
    pub bryarWallImpactEffect2: c_int,
    pub bryarWallImpactEffect3: c_int,
    pub bryarFleshImpactEffect: c_int,
    pub bryarDroidImpactEffect: c_int,
    pub _pad2: [u8; 100],
    pub blasterShotEffect: c_int,
    pub blasterWallImpactEffect: c_int,
    pub blasterFleshImpactEffect: c_int,
    pub blasterDroidImpactEffect: c_int,
    pub _pad3: [u8; 100],
    pub disruptorRingsEffect: c_int,
    pub disruptorProjectileEffect: c_int,
    pub disruptorWallImpactEffect: c_int,
    pub disruptorFleshImpactEffect: c_int,
    pub disruptorAltMissEffect: c_int,
    pub disruptorAltHitEffect: c_int,
    pub _pad4: [u8; 100],
    pub repeaterProjectileEffect: c_int,
    pub repeaterAltProjectileEffect: c_int,
    pub repeaterWallImpactEffect: c_int,
    pub repeaterFleshImpactEffect: c_int,
    pub repeaterAltWallImpactEffect: c_int,
    pub _pad5: [u8; 100],
    pub demp2ProjectileEffect: c_int,
    pub demp2WallImpactEffect: c_int,
    pub demp2FleshImpactEffect: c_int,
    pub _pad6: [u8; 100],
    pub flechetteShotEffect: c_int,
    pub flechetteAltShotEffect: c_int,
    pub flechetteWallImpactEffect: c_int,
    pub flechetteFleshImpactEffect: c_int,
    pub _pad7: [u8; 100],
    pub rocketShotEffect: c_int,
    pub rocketExplosionEffect: c_int,
    pub _pad8: [u8; 100],
    pub thermalExplosionEffect: c_int,
    pub thermalShockwaveEffect: c_int,
    pub _pad9: [u8; 100],
    pub tripmineLaserFX: c_int,
    pub tripmineGlowFX: c_int,
    pub _pad10: [u8; 100],
    pub concussionShotEffect: c_int,
    pub concussionImpactEffect: c_int,
}

/// Stub for cgMedia_t: registered shader/model handles.
#[repr(C)]
pub struct cgMedia_t {
    pub _pad0: [u8; 100],
    pub disruptorMask: c_int,
    pub disruptorInsert: c_int,
    pub disruptorLight: c_int,
    pub disruptorInsertTick: c_int,
    pub disruptorChargeShader: c_int,
    pub disruptorZoomLoop: c_int,
    pub _pad1: [u8; 100],
    pub bryarFrontFlash: c_int,
    pub _pad2: [u8; 100],
    pub greenFrontFlash: c_int,
    pub _pad3: [u8; 100],
    pub demp2Shell: c_int,
    pub demp2ShellShader: c_int,
    pub lightningFlash: c_int,
    pub _pad4: [u8; 100],
    pub grenadeBounce1: c_int,
    pub grenadeBounce2: c_int,
}

/// Stub for cgs_t struct (client game static state).
#[repr(C)]
pub struct cgs_t {
    pub _pad0: [u8; 9632],
    pub effects: cgEffects_t,
    pub media: cgMedia_t,
}

extern "C" {
    pub static mut cgs: cgs_t;
    pub static bg_itemlist: [gitem_t; 256];

    fn trap_R_RegisterModel(name: *const c_char) -> c_int;
    fn trap_R_ModelBounds(model: c_int, mins: *mut vec3_t, maxs: *mut vec3_t);
    fn trap_R_RegisterShader(name: *const c_char) -> c_int;
    fn trap_R_RegisterShaderNoMip(name: *const c_char) -> c_int;
    fn trap_S_RegisterSound(name: *const c_char) -> c_int;
    fn trap_FX_RegisterEffect(name: *const c_char) -> c_int;
    fn CG_RegisterItemVisuals(itemNum: c_int);
    fn CG_Error(fmt: *const c_char, ...);
    fn COM_StripExtension(in_: *const c_char, out: *mut c_char);
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
}

// Weapon constants
const WP_SABER: c_int = 1;
const WP_STUN_BATON: c_int = 2;
const WP_MELEE: c_int = 3;
const WP_BRYAR_PISTOL: c_int = 4;
const WP_BRYAR_OLD: c_int = 5;
const WP_BLASTER: c_int = 6;
const WP_DISRUPTOR: c_int = 7;
const WP_BOWCASTER: c_int = 8;
const WP_REPEATER: c_int = 9;
const WP_DEMP2: c_int = 10;
const WP_FLECHETTE: c_int = 11;
const WP_ROCKET_LAUNCHER: c_int = 12;
const WP_THERMAL: c_int = 13;
const WP_TRIP_MINE: c_int = 14;
const WP_DET_PACK: c_int = 15;
const WP_CONCUSSION: c_int = 16;
const WP_EMPLACED_GUN: c_int = 17;
const WP_TURRET: c_int = 18;

// Effect constants
const NULL_SOUND: c_int = -1;
const NULL_HANDLE: c_int = 0;
const NULL_FX: c_int = -1;

// Global weapons array
extern "C" {
    pub static mut cg_weapons: [weaponInfo_t; 128];
}

// Macros and helper functions
#[inline]
unsafe fn MAKERGB(dst: &mut vec3_t, r: f32, g: f32, b: f32) {
    dst[0] = r;
    dst[1] = g;
    dst[2] = b;
}

#[inline]
unsafe fn VectorSet(dst: &mut vec3_t, x: f32, y: f32, z: f32) {
    dst[0] = x;
    dst[1] = y;
    dst[2] = z;
}

/*
=================
CG_RegisterWeapon

The server says this item is used on this level
=================
*/
pub unsafe extern "C" fn CG_RegisterWeapon(weaponNum: c_int) {
    let mut weaponInfo: *mut weaponInfo_t;
    let mut item: *const gitem_t;
    let mut ammo: *const gitem_t;
    let mut path: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    let mut i: c_int;

    weaponInfo = &mut cg_weapons[weaponNum as usize];

    if weaponNum == 0 {
        return;
    }

    if (*weaponInfo).registered != 0 {
        return;
    }

    memset(weaponInfo as *mut c_void, 0, core::mem::size_of::<weaponInfo_t>());
    (*weaponInfo).registered = 1;

    item = &bg_itemlist[1];
    while !(*item).classname.is_null() {
        if (*item).giType == IT_WEAPON && (*item).giTag == weaponNum {
            (*weaponInfo).item = item;
            break;
        }
        item = item.add(1);
    }
    if (*item).classname.is_null() {
        CG_Error(
            b"Couldn't find weapon %i\0".as_ptr() as *const c_char,
            weaponNum,
        );
    }
    // Calculate item index: (item - bg_itemlist)
    let item_index = ((item as usize - addr_of!(bg_itemlist) as usize) / core::mem::size_of::<gitem_t>()) as c_int;
    CG_RegisterItemVisuals(item_index);

    // load cmodel before model so filecache works
    (*weaponInfo).weaponModel = trap_R_RegisterModel((*item).world_model[0]);
    // load in-view model also
    (*weaponInfo).viewModel = trap_R_RegisterModel((*item).view_model);

    // calc midpoint for rotation
    trap_R_ModelBounds((*weaponInfo).weaponModel, &mut mins, &mut maxs);
    i = 0;
    while i < 3 {
        (*weaponInfo).weaponMidpoint[i as usize] = mins[i as usize] + 0.5 * (maxs[i as usize] - mins[i as usize]);
        i += 1;
    }

    (*weaponInfo).weaponIcon = trap_R_RegisterShader((*item).icon);
    (*weaponInfo).ammoIcon = trap_R_RegisterShader((*item).icon);

    ammo = &bg_itemlist[1];
    while !(*ammo).classname.is_null() {
        if (*ammo).giType == IT_AMMO && (*ammo).giTag == weaponNum {
            break;
        }
        ammo = ammo.add(1);
    }
    if !(*ammo).classname.is_null() && !(*ammo).world_model[0].is_null() {
        (*weaponInfo).ammoModel = trap_R_RegisterModel((*ammo).world_model[0]);
    }

    //	strcpy( path, item->view_model );
    //	COM_StripExtension( path, path );
    //	strcat( path, "_flash.md3" );
    (*weaponInfo).flashModel = 0; //trap_R_RegisterModel( path );

    if weaponNum == WP_DISRUPTOR
        || weaponNum == WP_FLECHETTE
        || weaponNum == WP_REPEATER
        || weaponNum == WP_ROCKET_LAUNCHER
    {
        strcpy(path.as_mut_ptr(), (*item).view_model);
        COM_StripExtension(path.as_ptr(), path.as_mut_ptr());
        strcat(path.as_mut_ptr(), b"_barrel.md3\0".as_ptr() as *const c_char);
        (*weaponInfo).barrelModel = trap_R_RegisterModel(path.as_ptr());
    } else if weaponNum == WP_STUN_BATON {
        //only weapon with more than 1 barrel..
        trap_R_RegisterModel(b"models/weapons2/stun_baton/baton_barrel.md3\0".as_ptr() as *const c_char);
        trap_R_RegisterModel(b"models/weapons2/stun_baton/baton_barrel2.md3\0".as_ptr() as *const c_char);
        trap_R_RegisterModel(b"models/weapons2/stun_baton/baton_barrel3.md3\0".as_ptr() as *const c_char);
    } else {
        (*weaponInfo).barrelModel = 0;
    }

    if weaponNum != WP_SABER {
        strcpy(path.as_mut_ptr(), (*item).view_model);
        COM_StripExtension(path.as_ptr(), path.as_mut_ptr());
        strcat(path.as_mut_ptr(), b"_hand.md3\0".as_ptr() as *const c_char);
        (*weaponInfo).handsModel = trap_R_RegisterModel(path.as_ptr());
    } else {
        (*weaponInfo).handsModel = 0;
    }

    //	if ( !weaponInfo->handsModel ) {
    //		weaponInfo->handsModel = trap_R_RegisterModel( "models/weapons2/shotgun/shotgun_hand.md3" );
    //	}

    match weaponNum {
        WP_STUN_BATON | WP_MELEE => {
            /*		MAKERGB( weaponInfo->flashDlightColor, 0.6f, 0.6f, 1.0f );
            weaponInfo->firingSound = trap_S_RegisterSound( "sound/weapons/saber/saberhum.wav" );
    //		weaponInfo->flashSound[0] = trap_S_RegisterSound( "sound/weapons/melee/fstatck.wav" );
    */
            //trap_R_RegisterShader( "gfx/effects/stunPass" );
            trap_FX_RegisterEffect(b"stunBaton/flesh_impact\0".as_ptr() as *const c_char);

            if weaponNum == WP_STUN_BATON {
                trap_S_RegisterSound(b"sound/weapons/baton/idle.wav\0".as_ptr() as *const c_char);
                (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/baton/fire.mp3\0".as_ptr() as *const c_char);
                (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/baton/fire.mp3\0".as_ptr() as *const c_char);
            } else {
                /*
                int j = 0;

                while (j < 4)
                {
                    weaponInfo->flashSound[j] = trap_S_RegisterSound( va("sound/weapons/melee/swing%i", j+1) );
                    weaponInfo->altFlashSound[j] = weaponInfo->flashSound[j];
                    j++;
                }
                */
                //No longer needed, animsound config plays them for us
            }
        }
        WP_SABER => {
            MAKERGB(&mut (*weaponInfo).flashDlightColor, 0.6, 0.6, 1.0);
            (*weaponInfo).firingSound = trap_S_RegisterSound(b"sound/weapons/saber/saberhum1.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = trap_R_RegisterModel(b"models/weapons2/saber/saber_w.glm\0".as_ptr() as *const c_char);
        }

        WP_CONCUSSION => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/concussion/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = NULL_SOUND;
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"concussion/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //weaponInfo->missileDlightColor= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_ConcussionProjectileThink as *const c_void;

            (*weaponInfo).altFlashSound[0] = NULL_SOUND;
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = trap_S_RegisterSound(b"sound/weapons/bryar/altcharge.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"concussion/altmuzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = NULL_HANDLE;
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = FX_ConcussionProjectileThink as *const c_void;

            (*addr_of_mut!(cgs)).effects.disruptorAltMissEffect = trap_FX_RegisterEffect(b"disruptor/alt_miss\0".as_ptr() as *const c_char);

            (*addr_of_mut!(cgs)).effects.concussionShotEffect = trap_FX_RegisterEffect(b"concussion/shot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.concussionImpactEffect = trap_FX_RegisterEffect(b"concussion/explosion\0".as_ptr() as *const c_char);
            trap_R_RegisterShader(b"gfx/effects/blueLine\0".as_ptr() as *const c_char);
            trap_R_RegisterShader(b"gfx/misc/whiteline2\0".as_ptr() as *const c_char);
        }

        WP_BRYAR_PISTOL | WP_BRYAR_OLD => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/bryar/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/bryar/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"bryar/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //weaponInfo->missileDlightColor= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_BryarProjectileThink as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/bryar/alt_fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = trap_S_RegisterSound(b"sound/weapons/bryar/altcharge.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"bryar/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = NULL_HANDLE;
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = FX_BryarAltProjectileThink as *const c_void;

            (*addr_of_mut!(cgs)).effects.bryarShotEffect = trap_FX_RegisterEffect(b"bryar/shot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.bryarPowerupShotEffect = trap_FX_RegisterEffect(b"bryar/crackleShot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.bryarWallImpactEffect = trap_FX_RegisterEffect(b"bryar/wall_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.bryarWallImpactEffect2 = trap_FX_RegisterEffect(b"bryar/wall_impact2\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.bryarWallImpactEffect3 = trap_FX_RegisterEffect(b"bryar/wall_impact3\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.bryarFleshImpactEffect = trap_FX_RegisterEffect(b"bryar/flesh_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.bryarDroidImpactEffect = trap_FX_RegisterEffect(b"bryar/droid_impact\0".as_ptr() as *const c_char);

            (*addr_of_mut!(cgs)).media.bryarFrontFlash = trap_R_RegisterShader(b"gfx/effects/bryarFrontFlash\0".as_ptr() as *const c_char);

            // Note these are temp shared effects
            trap_FX_RegisterEffect(b"blaster/wall_impact.efx\0".as_ptr() as *const c_char);
            trap_FX_RegisterEffect(b"blaster/flesh_impact.efx\0".as_ptr() as *const c_char);
        }

        WP_BLASTER | WP_EMPLACED_GUN => {
            //rww - just use the same as this for now..
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/blaster/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/blaster/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"blaster/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_BlasterProjectileThink as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/blaster/alt_fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = NULL_SOUND;
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"blaster/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = NULL_HANDLE;
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = FX_BlasterProjectileThink as *const c_void;

            trap_FX_RegisterEffect(b"blaster/deflect\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.blasterShotEffect = trap_FX_RegisterEffect(b"blaster/shot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.blasterWallImpactEffect = trap_FX_RegisterEffect(b"blaster/wall_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.blasterFleshImpactEffect = trap_FX_RegisterEffect(b"blaster/flesh_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.blasterDroidImpactEffect = trap_FX_RegisterEffect(b"blaster/droid_impact\0".as_ptr() as *const c_char);
        }

        WP_DISRUPTOR => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/disruptor/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/disruptor/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"disruptor/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = 0 as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/disruptor/alt_fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = trap_S_RegisterSound(b"sound/weapons/disruptor/altCharge.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"disruptor/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = NULL_HANDLE;
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = 0 as *const c_void;

            (*addr_of_mut!(cgs)).effects.disruptorRingsEffect = trap_FX_RegisterEffect(b"disruptor/rings\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.disruptorProjectileEffect = trap_FX_RegisterEffect(b"disruptor/projectile\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.disruptorWallImpactEffect = trap_FX_RegisterEffect(b"disruptor/wall_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.disruptorFleshImpactEffect = trap_FX_RegisterEffect(b"disruptor/flesh_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.disruptorAltMissEffect = trap_FX_RegisterEffect(b"disruptor/alt_miss\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.disruptorAltHitEffect = trap_FX_RegisterEffect(b"disruptor/alt_hit\0".as_ptr() as *const c_char);

            trap_R_RegisterShader(b"gfx/effects/redLine\0".as_ptr() as *const c_char);
            trap_R_RegisterShader(b"gfx/misc/whiteline2\0".as_ptr() as *const c_char);
            trap_R_RegisterShader(b"gfx/effects/smokeTrail\0".as_ptr() as *const c_char);

            trap_S_RegisterSound(b"sound/weapons/disruptor/zoomstart.wav\0".as_ptr() as *const c_char);
            trap_S_RegisterSound(b"sound/weapons/disruptor/zoomend.wav\0".as_ptr() as *const c_char);

            // Disruptor gun zoom interface
            (*addr_of_mut!(cgs)).media.disruptorMask = trap_R_RegisterShader(b"gfx/2d/cropCircle2\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).media.disruptorInsert = trap_R_RegisterShader(b"gfx/2d/cropCircle\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).media.disruptorLight = trap_R_RegisterShader(b"gfx/2d/cropCircleGlow\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).media.disruptorInsertTick = trap_R_RegisterShader(b"gfx/2d/insertTick\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).media.disruptorChargeShader = trap_R_RegisterShaderNoMip(b"gfx/2d/crop_charge\0".as_ptr() as *const c_char);

            (*addr_of_mut!(cgs)).media.disruptorZoomLoop = trap_S_RegisterSound(b"sound/weapons/disruptor/zoomloop.wav\0".as_ptr() as *const c_char);
        }

        WP_BOWCASTER => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/bowcaster/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/bowcaster/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = NULL_SOUND;
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"bowcaster/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = NULL_HANDLE;
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor	= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = FX_BowcasterProjectileThink as *const c_void;

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/bowcaster/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = trap_S_RegisterSound(b"sound/weapons/bowcaster/altcharge.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"bowcaster/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_BowcasterAltProjectileThink as *const c_void;

            (*addr_of_mut!(cgs)).effects.bowcasterShotEffect = trap_FX_RegisterEffect(b"bowcaster/shot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.bowcasterImpactEffect = trap_FX_RegisterEffect(b"bowcaster/explosion\0".as_ptr() as *const c_char);

            trap_FX_RegisterEffect(b"bowcaster/deflect\0".as_ptr() as *const c_char);

            (*addr_of_mut!(cgs)).media.greenFrontFlash = trap_R_RegisterShader(b"gfx/effects/greenFrontFlash\0".as_ptr() as *const c_char);
        }

        WP_REPEATER => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/repeater/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/repeater/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"repeater/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_RepeaterProjectileThink as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/repeater/alt_fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = NULL_SOUND;
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"repeater/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = NULL_HANDLE;
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = FX_RepeaterAltProjectileThink as *const c_void;

            (*addr_of_mut!(cgs)).effects.repeaterProjectileEffect = trap_FX_RegisterEffect(b"repeater/projectile\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.repeaterAltProjectileEffect = trap_FX_RegisterEffect(b"repeater/alt_projectile\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.repeaterWallImpactEffect = trap_FX_RegisterEffect(b"repeater/wall_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.repeaterFleshImpactEffect = trap_FX_RegisterEffect(b"repeater/flesh_impact\0".as_ptr() as *const c_char);
            //cgs.effects.repeaterAltWallImpactEffect	= trap_FX_RegisterEffect( "repeater/alt_wall_impact" );
            (*addr_of_mut!(cgs)).effects.repeaterAltWallImpactEffect = trap_FX_RegisterEffect(b"repeater/concussion\0".as_ptr() as *const c_char);
        }

        WP_DEMP2 => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/demp2/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/demp2/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"demp2/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_DEMP2_ProjectileThink as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/demp2/altfire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = trap_S_RegisterSound(b"sound/weapons/demp2/altCharge.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"demp2/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = NULL_HANDLE;
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = 0 as *const c_void;

            (*addr_of_mut!(cgs)).effects.demp2ProjectileEffect = trap_FX_RegisterEffect(b"demp2/projectile\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.demp2WallImpactEffect = trap_FX_RegisterEffect(b"demp2/wall_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.demp2FleshImpactEffect = trap_FX_RegisterEffect(b"demp2/flesh_impact\0".as_ptr() as *const c_char);

            (*addr_of_mut!(cgs)).media.demp2Shell = trap_R_RegisterModel(b"models/items/sphere.md3\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).media.demp2ShellShader = trap_R_RegisterShader(b"gfx/effects/demp2shell\0".as_ptr() as *const c_char);

            (*addr_of_mut!(cgs)).media.lightningFlash = trap_R_RegisterShader(b"gfx/misc/lightningFlash\0".as_ptr() as *const c_char);
        }

        WP_FLECHETTE => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/flechette/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/flechette/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"flechette/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).missileModel = trap_R_RegisterModel(b"models/weapons2/golan_arms/projectileMain.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_FlechetteProjectileThink as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/flechette/alt_fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = NULL_SOUND;
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"flechette/muzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = trap_R_RegisterModel(b"models/weapons2/golan_arms/projectile.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = FX_FlechetteAltProjectileThink as *const c_void;

            (*addr_of_mut!(cgs)).effects.flechetteShotEffect = trap_FX_RegisterEffect(b"flechette/shot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.flechetteAltShotEffect = trap_FX_RegisterEffect(b"flechette/alt_shot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.flechetteWallImpactEffect = trap_FX_RegisterEffect(b"flechette/wall_impact\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.flechetteFleshImpactEffect = trap_FX_RegisterEffect(b"flechette/flesh_impact\0".as_ptr() as *const c_char);
        }

        WP_ROCKET_LAUNCHER => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/rocket/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/rocket/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = trap_FX_RegisterEffect(b"rocket/muzzle_flash\0".as_ptr() as *const c_char); //trap_FX_RegisterEffect( "rocket/muzzle_flash2" );
            //flash2 still looks crappy with the fx bolt stuff. Because the fx bolt stuff doesn't work entirely right.
            (*weaponInfo).missileModel = trap_R_RegisterModel(b"models/weapons2/merr_sonn/projectile.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).missileSound = trap_S_RegisterSound(b"sound/weapons/rocket/missleloop.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).missileDlight = 125;
            VectorSet(&mut (*weaponInfo).missileDlightColor, 1.0, 1.0, 0.5);
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_RocketProjectileThink as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/rocket/alt_fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = NULL_SOUND;
            (*weaponInfo).altMuzzleEffect = trap_FX_RegisterEffect(b"rocket/altmuzzle_flash\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileModel = trap_R_RegisterModel(b"models/weapons2/merr_sonn/projectile.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileSound = trap_S_RegisterSound(b"sound/weapons/rocket/missleloop.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileDlight = 125;
            VectorSet(&mut (*weaponInfo).altMissileDlightColor, 1.0, 1.0, 0.5);
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = FX_RocketAltProjectileThink as *const c_void;

            (*addr_of_mut!(cgs)).effects.rocketShotEffect = trap_FX_RegisterEffect(b"rocket/shot\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.rocketExplosionEffect = trap_FX_RegisterEffect(b"rocket/explosion\0".as_ptr() as *const c_char);

            trap_R_RegisterShaderNoMip(b"gfx/2d/wedge\0".as_ptr() as *const c_char);
            trap_R_RegisterShaderNoMip(b"gfx/2d/lock\0".as_ptr() as *const c_char);

            trap_S_RegisterSound(b"sound/weapons/rocket/lock.wav\0".as_ptr() as *const c_char);
            trap_S_RegisterSound(b"sound/weapons/rocket/tick.wav\0".as_ptr() as *const c_char);
        }

        WP_THERMAL => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/thermal/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/thermal/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = trap_S_RegisterSound(b"sound/weapons/thermal/charge.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).muzzleEffect = NULL_FX;
            (*weaponInfo).missileModel = trap_R_RegisterModel(b"models/weapons2/thermal/thermal_proj.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = 0 as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/thermal/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = trap_S_RegisterSound(b"sound/weapons/thermal/charge.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altMuzzleEffect = NULL_FX;
            (*weaponInfo).altMissileModel = trap_R_RegisterModel(b"models/weapons2/thermal/thermal_proj.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = 0 as *const c_void;

            (*addr_of_mut!(cgs)).effects.thermalExplosionEffect = trap_FX_RegisterEffect(b"thermal/explosion\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.thermalShockwaveEffect = trap_FX_RegisterEffect(b"thermal/shockwave\0".as_ptr() as *const c_char);

            (*addr_of_mut!(cgs)).media.grenadeBounce1 = trap_S_RegisterSound(b"sound/weapons/thermal/bounce1.wav\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).media.grenadeBounce2 = trap_S_RegisterSound(b"sound/weapons/thermal/bounce2.wav\0".as_ptr() as *const c_char);

            trap_S_RegisterSound(b"sound/weapons/thermal/thermloop.wav\0".as_ptr() as *const c_char);
            trap_S_RegisterSound(b"sound/weapons/thermal/warning.wav\0".as_ptr() as *const c_char);
        }

        WP_TRIP_MINE => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/detpack/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/laser_trap/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = NULL_FX;
            (*weaponInfo).missileModel = 0; //trap_R_RegisterModel( "models/weapons2/laser_trap/laser_trap_w.md3" );
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = 0 as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/laser_trap/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = NULL_SOUND;
            (*weaponInfo).altMuzzleEffect = NULL_FX;
            (*weaponInfo).altMissileModel = 0; //trap_R_RegisterModel( "models/weapons2/laser_trap/laser_trap_w.md3" );
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = 0 as *const c_void;

            (*addr_of_mut!(cgs)).effects.tripmineLaserFX = trap_FX_RegisterEffect(b"tripMine/laserMP.efx\0".as_ptr() as *const c_char);
            (*addr_of_mut!(cgs)).effects.tripmineGlowFX = trap_FX_RegisterEffect(b"tripMine/glowbit.efx\0".as_ptr() as *const c_char);

            trap_FX_RegisterEffect(b"tripMine/explosion\0".as_ptr() as *const c_char);
            // NOTENOTE temp stuff
            trap_S_RegisterSound(b"sound/weapons/laser_trap/stick.wav\0".as_ptr() as *const c_char);
            trap_S_RegisterSound(b"sound/weapons/laser_trap/warning.wav\0".as_ptr() as *const c_char);
        }

        WP_DET_PACK => {
            (*weaponInfo).selectSound = trap_S_RegisterSound(b"sound/weapons/detpack/select.wav\0".as_ptr() as *const c_char);

            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/detpack/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = NULL_FX;
            (*weaponInfo).missileModel = trap_R_RegisterModel(b"models/weapons2/detpack/det_pack.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = 0 as *const c_void;

            (*weaponInfo).altFlashSound[0] = trap_S_RegisterSound(b"sound/weapons/detpack/fire.wav\0".as_ptr() as *const c_char);
            (*weaponInfo).altFiringSound = NULL_SOUND;
            (*weaponInfo).altChargeSound = NULL_SOUND;
            (*weaponInfo).altMuzzleEffect = NULL_FX;
            (*weaponInfo).altMissileModel = trap_R_RegisterModel(b"models/weapons2/detpack/det_pack.md3\0".as_ptr() as *const c_char);
            (*weaponInfo).altMissileSound = NULL_SOUND;
            (*weaponInfo).altMissileDlight = 0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            (*weaponInfo).altMissileHitSound = NULL_SOUND;
            (*weaponInfo).altMissileTrailFunc = 0 as *const c_void;

            trap_R_RegisterModel(b"models/weapons2/detpack/det_pack.md3\0".as_ptr() as *const c_char);
            trap_S_RegisterSound(b"sound/weapons/detpack/stick.wav\0".as_ptr() as *const c_char);
            trap_S_RegisterSound(b"sound/weapons/detpack/warning.wav\0".as_ptr() as *const c_char);
            trap_S_RegisterSound(b"sound/weapons/explosions/explode5.wav\0".as_ptr() as *const c_char);
        }
        WP_TURRET => {
            (*weaponInfo).flashSound[0] = NULL_SOUND;
            (*weaponInfo).firingSound = NULL_SOUND;
            (*weaponInfo).chargeSound = NULL_SOUND;
            (*weaponInfo).muzzleEffect = NULL_HANDLE;
            (*weaponInfo).missileModel = NULL_HANDLE;
            (*weaponInfo).missileSound = NULL_SOUND;
            (*weaponInfo).missileDlight = 0;
            (*weaponInfo).missileHitSound = NULL_SOUND;
            (*weaponInfo).missileTrailFunc = FX_TurretProjectileThink as *const c_void;

            trap_FX_RegisterEffect(b"effects/blaster/wall_impact.efx\0".as_ptr() as *const c_char);
            trap_FX_RegisterEffect(b"effects/blaster/flesh_impact.efx\0".as_ptr() as *const c_char);
        }

        _ => {
            MAKERGB(&mut (*weaponInfo).flashDlightColor, 1.0, 1.0, 1.0);
            (*weaponInfo).flashSound[0] = trap_S_RegisterSound(b"sound/weapons/rocket/rocklf1a.wav\0".as_ptr() as *const c_char);
        }
    }
}

// Stub extern function pointers for weapon trail effects
extern "C" {
    pub fn FX_ConcussionProjectileThink();
    pub fn FX_BryarProjectileThink();
    pub fn FX_BryarAltProjectileThink();
    pub fn FX_BlasterProjectileThink();
    pub fn FX_BowcasterProjectileThink();
    pub fn FX_BowcasterAltProjectileThink();
    pub fn FX_RepeaterProjectileThink();
    pub fn FX_RepeaterAltProjectileThink();
    pub fn FX_DEMP2_ProjectileThink();
    pub fn FX_FlechetteProjectileThink();
    pub fn FX_FlechetteAltProjectileThink();
    pub fn FX_RocketProjectileThink();
    pub fn FX_RocketAltProjectileThink();
    pub fn FX_TurretProjectileThink();
}

// Additional media stubs needed
extern "C" {
    pub static mut cgMedia: cgMedia_t;
}
