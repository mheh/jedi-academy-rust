// g_weaponLoad.cpp
// fills in memory struct with ext_dat\weapons.dat

// this is excluded from PCH usage 'cos it looks kinda scary to me, being game and ui.... -Ste

use core::ffi::{c_char, c_int, c_float, c_void};

#[allow(non_snake_case)]

// ONLY DO THIS ON THE GAME SIDE
// #include "g_local.h"

// Stub types - these would be defined in g_local.h
#[repr(C)]
pub struct centity_t;

#[repr(C)]
pub struct weaponInfo_s;

#[repr(C)]
pub struct weaponData_t;

#[repr(C)]
pub struct ammoData_t;

#[repr(C)]
pub struct gameinfo_import_t;

extern "C" {
    pub static mut gi: gameinfo_import_t;
    pub static mut weaponData: [weaponData_t; 0];
    pub static mut ammoData: [ammoData_t; 0];
}

#[repr(C)]
pub struct func_t {
    pub name: *const c_char,
    pub func: Option<unsafe extern "C" fn(*mut centity_t, *const weaponInfo_s)>,
}

// Bryar
extern "C" {
    pub fn FX_BryarProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BryarAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Blaster
    pub fn FX_BlasterProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_BlasterAltFireThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Bowcaster
    pub fn FX_BowcasterProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Heavy Repeater
    pub fn FX_RepeaterProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_RepeaterAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // DEMP2
    pub fn FX_DEMP2_ProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_DEMP2_AltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Golan Arms Flechette
    pub fn FX_FlechetteProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_FlechetteAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Personal Rocket Launcher
    pub fn FX_RocketProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_RocketAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Concussion Rifle
    pub fn FX_ConcProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Emplaced weapon
    pub fn FX_EmplacedProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Turret weapon
    pub fn FX_TurretProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // ATST Main weapon
    pub fn FX_ATSTMainProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // ATST Side weapons
    pub fn FX_ATSTSideMainProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);
    pub fn FX_ATSTSideAltProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Tusken projectile
    pub fn FX_TuskenShotProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    // Noghri projectile
    pub fn FX_NoghriShotProjectileThink(cent: *mut centity_t, weapon: *const weaponInfo_s);

    pub fn COM_ParseInt(data: *mut *const c_char, i: *mut c_int) -> c_int;
    pub fn COM_ParseString(data: *mut *const c_char, s: *mut *const c_char) -> c_int;
    pub fn COM_ParseFloat(data: *mut *const c_char, f: *mut c_float) -> c_int;
    pub fn COM_ParseExt(data: *mut *const c_char, allow_nl: c_int) -> *const c_char;
    pub fn COM_BeginParseSession();
    pub fn SkipRestOfLine(data: *mut *const c_char);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn strlen(s: *const c_char) -> usize;
    pub fn G_EffectIndex(name: *const c_char) -> c_int;
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
    pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    pub fn FS_FreeFile(buf: *mut c_void);
}

// Table used to attach an extern missile function string to the actual cgame function
#[allow(non_upper_case_globals)]
static mut funcs: [func_t; 22] = [
    func_t { name: b"bryar_func\0".as_ptr() as *const c_char, func: Some(FX_BryarProjectileThink) },
    func_t { name: b"bryar_alt_func\0".as_ptr() as *const c_char, func: Some(FX_BryarAltProjectileThink) },
    func_t { name: b"blaster_func\0".as_ptr() as *const c_char, func: Some(FX_BlasterProjectileThink) },
    func_t { name: b"blaster_alt_func\0".as_ptr() as *const c_char, func: Some(FX_BlasterAltFireThink) },
    func_t { name: b"bowcaster_func\0".as_ptr() as *const c_char, func: Some(FX_BowcasterProjectileThink) },
    func_t { name: b"repeater_func\0".as_ptr() as *const c_char, func: Some(FX_RepeaterProjectileThink) },
    func_t { name: b"repeater_alt_func\0".as_ptr() as *const c_char, func: Some(FX_RepeaterAltProjectileThink) },
    func_t { name: b"demp2_func\0".as_ptr() as *const c_char, func: Some(FX_DEMP2_ProjectileThink) },
    func_t { name: b"demp2_alt_func\0".as_ptr() as *const c_char, func: Some(FX_DEMP2_AltProjectileThink) },
    func_t { name: b"flechette_func\0".as_ptr() as *const c_char, func: Some(FX_FlechetteProjectileThink) },
    func_t { name: b"flechette_alt_func\0".as_ptr() as *const c_char, func: Some(FX_FlechetteAltProjectileThink) },
    func_t { name: b"rocket_func\0".as_ptr() as *const c_char, func: Some(FX_RocketProjectileThink) },
    func_t { name: b"rocket_alt_func\0".as_ptr() as *const c_char, func: Some(FX_RocketAltProjectileThink) },
    func_t { name: b"conc_func\0".as_ptr() as *const c_char, func: Some(FX_ConcProjectileThink) },
    func_t { name: b"emplaced_func\0".as_ptr() as *const c_char, func: Some(FX_EmplacedProjectileThink) },
    func_t { name: b"turret_func\0".as_ptr() as *const c_char, func: Some(FX_TurretProjectileThink) },
    func_t { name: b"atstmain_func\0".as_ptr() as *const c_char, func: Some(FX_ATSTMainProjectileThink) },
    func_t { name: b"atst_side_alt_func\0".as_ptr() as *const c_char, func: Some(FX_ATSTSideAltProjectileThink) },
    func_t { name: b"atst_side_main_func\0".as_ptr() as *const c_char, func: Some(FX_ATSTSideMainProjectileThink) },
    func_t { name: b"tusk_shot_func\0".as_ptr() as *const c_char, func: Some(FX_TuskenShotProjectileThink) },
    func_t { name: b"noghri_shot_func\0".as_ptr() as *const c_char, func: Some(FX_NoghriShotProjectileThink) },
    func_t { name: core::ptr::null(), func: None },
];

#[repr(C)]
struct wpnParms_s {
    pub weaponNum: c_int,  // Current weapon number
    pub ammoNum: c_int,
}

#[allow(non_upper_case_globals)]
static mut wpnParms: wpnParms_s = wpnParms_s {
    weaponNum: 0,
    ammoNum: 0,
};

const ERR_FATAL: c_int = 3;

//qboolean COM_ParseInt( char **data, int *i );
//qboolean COM_ParseString( char **data, char **s );
//qboolean COM_ParseFloat( char **data, float *f );

#[repr(C)]
struct wpnParms_t {
    pub parmName: *const c_char,
    pub func: Option<extern "C" fn(*mut *const c_char)>,
}

pub fn WPN_WeaponType(holdBuf: *mut *const c_char) {
    let mut weaponNum: c_int = 0;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    // FIXME : put this in an array (maybe a weaponDataInternal array???)
    if Q_stricmp(tokenStr, b"WP_NONE\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 0; // WP_NONE
    } else if Q_stricmp(tokenStr, b"WP_SABER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 1; // WP_SABER
    } else if Q_stricmp(tokenStr, b"WP_BLASTER_PISTOL\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 2; // WP_BLASTER_PISTOL
    } else if Q_stricmp(tokenStr, b"WP_BRYAR_PISTOL\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 3; // WP_BRYAR_PISTOL
    } else if Q_stricmp(tokenStr, b"WP_BLASTER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 4; // WP_BLASTER
    } else if Q_stricmp(tokenStr, b"WP_DISRUPTOR\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 5; // WP_DISRUPTOR
    } else if Q_stricmp(tokenStr, b"WP_BOWCASTER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 6; // WP_BOWCASTER
    } else if Q_stricmp(tokenStr, b"WP_REPEATER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 7; // WP_REPEATER
    } else if Q_stricmp(tokenStr, b"WP_DEMP2\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 8; // WP_DEMP2
    } else if Q_stricmp(tokenStr, b"WP_FLECHETTE\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 9; // WP_FLECHETTE
    } else if Q_stricmp(tokenStr, b"WP_ROCKET_LAUNCHER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 10; // WP_ROCKET_LAUNCHER
    } else if Q_stricmp(tokenStr, b"WP_CONCUSSION\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 11; // WP_CONCUSSION
    } else if Q_stricmp(tokenStr, b"WP_THERMAL\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 12; // WP_THERMAL
    } else if Q_stricmp(tokenStr, b"WP_TRIP_MINE\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 13; // WP_TRIP_MINE
    } else if Q_stricmp(tokenStr, b"WP_DET_PACK\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 14; // WP_DET_PACK
    } else if Q_stricmp(tokenStr, b"WP_STUN_BATON\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 15; // WP_STUN_BATON
    } else if Q_stricmp(tokenStr, b"WP_BOT_LASER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 16; // WP_BOT_LASER
    } else if Q_stricmp(tokenStr, b"WP_EMPLACED_GUN\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 17; // WP_EMPLACED_GUN
    } else if Q_stricmp(tokenStr, b"WP_MELEE\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 18; // WP_MELEE
    } else if Q_stricmp(tokenStr, b"WP_TURRET\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 19; // WP_TURRET
    } else if Q_stricmp(tokenStr, b"WP_ATST_MAIN\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 20; // WP_ATST_MAIN
    } else if Q_stricmp(tokenStr, b"WP_ATST_SIDE\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 21; // WP_ATST_SIDE
    } else if Q_stricmp(tokenStr, b"WP_TIE_FIGHTER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 22; // WP_TIE_FIGHTER
    } else if Q_stricmp(tokenStr, b"WP_RAPID_FIRE_CONC\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 23; // WP_RAPID_FIRE_CONC
    } else if Q_stricmp(tokenStr, b"WP_JAWA\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 24; // WP_JAWA
    } else if Q_stricmp(tokenStr, b"WP_TUSKEN_RIFLE\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 25; // WP_TUSKEN_RIFLE
    } else if Q_stricmp(tokenStr, b"WP_TUSKEN_STAFF\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 26; // WP_TUSKEN_STAFF
    } else if Q_stricmp(tokenStr, b"WP_SCEPTER\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 27; // WP_SCEPTER
    } else if Q_stricmp(tokenStr, b"WP_NOGHRI_STICK\0".as_ptr() as *const c_char) == 0 {
        weaponNum = 28; // WP_NOGHRI_STICK
    } else {
        weaponNum = 0;
        // WARNING: bad weapontype in external weapon data '%s'
    }

    unsafe {
        wpnParms.weaponNum = weaponNum;
    }
}

//--------------------------------------------
pub fn WPN_WeaponClass(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 32 {
        len = 32;
        // WARNING: weaponclass too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_WeaponModel(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: weaponMdl too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_WeaponIcon(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: weaponIcon too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AmmoType(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt >= 10 {
        // WARNING: bad Ammotype in external weapon data '%d'
        return;
    }

    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().ammoIndex = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_AmmoLowCnt(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 200 {
        // FIXME :What are the right values?
        // WARNING: bad Ammolowcount in external weapon data '%d'
        return;
    }

    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().ammoLow = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_FiringSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: firingSnd too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AltFiringSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: altFiringSnd too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_StopSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: stopSnd too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

/*
//--------------------------------------------
pub fn WPN_FlashSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: flashSnd too long in external WEAPONS.DAT '%s'
    }

    Q_strncpyz(weaponData[wpnParms.weaponNum].flashSnd, tokenStr, len);
}

//--------------------------------------------
pub fn WPN_AltFlashSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: altFlashSnd too long in external WEAPONS.DAT '%s'
    }

    Q_strncpyz(weaponData[wpnParms.weaponNum].altFlashSnd, tokenStr, len);
}
*/

//--------------------------------------------
pub fn WPN_ChargeSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: chargeSnd too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AltChargeSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: altChargeSnd too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_SelectSnd(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;

    if len > 64 {
        len = 64;
        // WARNING: selectSnd too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//#ifdef _IMMERSION

//--------------------------------------------
pub fn WPN_FiringFrc(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;

    if len > 64 {
        len = 64;
        // WARNING: firingFrc too long in external WEAPONS.DAT '%s'
    }

    #[cfg(feature = "immersion")]
    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AltFiringFrc(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;

    if len > 64 {
        len = 64;
        // WARNING: altFiringFrc too long in external WEAPONS.DAT '%s'
    }

    #[cfg(feature = "immersion")]
    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_ChargeFrc(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;

    if len > 64 {
        len = 64;
        // WARNING: chargeFrc too long in external WEAPONS.DAT '%s'
    }

    #[cfg(feature = "immersion")]
    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AltChargeFrc(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;

    if len > 64 {
        len = 64;
        // WARNING: altChargeFrc too long in external WEAPONS.DAT '%s'
    }

    #[cfg(feature = "immersion")]
    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_StopFrc(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;

    if len > 64 {
        len = 64;
        // WARNING: stopFrc too long in external WEAPONS.DAT '%s'
    }

    #[cfg(feature = "immersion")]
    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_SelectFrc(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;

    if len > 64 {
        len = 64;
        // WARNING: selectFrc too long in external WEAPONS.DAT '%s'
    }

    #[cfg(feature = "immersion")]
    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//#endif // _IMMERSION

//--------------------------------------------
pub fn WPN_FireTime(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 10000 {
        // FIXME :What are the right values?
        // WARNING: bad Firetime in external weapon data '%d'
        return;
    }
    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().fireTime = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_Range(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 10000 {
        // FIXME :What are the right values?
        // WARNING: bad Range in external weapon data '%d'
        return;
    }

    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().range = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_EnergyPerShot(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 1000 {
        // FIXME :What are the right values?
        // WARNING: bad EnergyPerShot in external weapon data '%d'
        return;
    }
    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().energyPerShot = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_AltFireTime(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 10000 {
        // FIXME :What are the right values?
        // WARNING: bad altFireTime in external weapon data '%d'
        return;
    }
    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().altFireTime = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_AltRange(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 10000 {
        // FIXME :What are the right values?
        // WARNING: bad AltRange in external weapon data '%d'
        return;
    }

    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().altRange = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_AltEnergyPerShot(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 1000 {
        // FIXME :What are the right values?
        // WARNING: bad AltEnergyPerShot in external weapon data '%d'
        return;
    }
    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().altEnergyPerShot = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_Ammo(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    if Q_stricmp(tokenStr, b"AMMO_NONE\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 0; } // AMMO_NONE
    } else if Q_stricmp(tokenStr, b"AMMO_FORCE\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 1; } // AMMO_FORCE
    } else if Q_stricmp(tokenStr, b"AMMO_BLASTER\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 2; } // AMMO_BLASTER
    } else if Q_stricmp(tokenStr, b"AMMO_POWERCELL\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 3; } // AMMO_POWERCELL
    } else if Q_stricmp(tokenStr, b"AMMO_METAL_BOLTS\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 4; } // AMMO_METAL_BOLTS
    } else if Q_stricmp(tokenStr, b"AMMO_ROCKETS\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 5; } // AMMO_ROCKETS
    } else if Q_stricmp(tokenStr, b"AMMO_EMPLACED\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 6; } // AMMO_EMPLACED
    } else if Q_stricmp(tokenStr, b"AMMO_THERMAL\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 7; } // AMMO_THERMAL
    } else if Q_stricmp(tokenStr, b"AMMO_TRIPMINE\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 8; } // AMMO_TRIPMINE
    } else if Q_stricmp(tokenStr, b"AMMO_DETPACK\0".as_ptr() as *const c_char) == 0 {
        unsafe { wpnParms.ammoNum = 9; } // AMMO_DETPACK
    } else {
        // WARNING: bad ammotype in external weapon data '%s'
        unsafe { wpnParms.ammoNum = 0; }
    }
}

//--------------------------------------------
pub fn WPN_AmmoIcon(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();
    let mut len: c_int;

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: ammoicon too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(ammoData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AmmoMax(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 1000 {
        // WARNING: bad Ammo Max in external weapon data '%d'
        return;
    }
    unsafe {
        (*core::ptr::addr_of_mut!(ammoData)).as_mut_ptr().as_mut().unwrap().max = tokenInt;
    }
}

//--------------------------------------------
pub fn WPN_BarrelCount(holdBuf: *mut *const c_char) {
    let mut tokenInt: c_int = 0;

    if COM_ParseInt(holdBuf, &mut tokenInt) != 0 {
        SkipRestOfLine(holdBuf);
        return;
    }

    if tokenInt < 0 || tokenInt > 4 {
        // WARNING: bad Range in external weapon data '%d'
        return;
    }

    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().numBarrels = tokenInt;
    }
}

//--------------------------------------------
fn WP_ParseWeaponParms(holdBuf: *mut *const c_char) {
    let mut token: *const c_char;
    let mut i: usize;

    loop {
        token = COM_ParseExt(holdBuf, 1);

        if Q_stricmp(token, b"}\0".as_ptr() as *const c_char) == 0 {
            // End of data for this weapon
            break;
        }

        // Loop through possible parameters
        i = 0;
        while i < WPN_PARM_MAX {
            if Q_stricmp(token, unsafe { WpnParms[i].parmName }) == 0 {
                if let Some(func) = unsafe { WpnParms[i].func } {
                    func(holdBuf);
                }
                break;
            }
            i += 1;
        }

        if i < WPN_PARM_MAX {
            // Find parameter???
            continue;
        }
        Com_Error(ERR_FATAL, b"bad parameter in external weapon data '%s'\n\0".as_ptr() as *const c_char, token);
    }
}

//--------------------------------------------
pub fn WPN_MissileName(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: MissileName too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AltMissileName(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: AltMissileName too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_MissileHitSound(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: MissileHitSound too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AltMissileHitSound(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: AltMissileHitSound too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_MissileSound(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: MissileSound too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_AltMissileSound(holdBuf: *mut *const c_char) {
    let mut len: c_int;
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    len = strlen(tokenStr) as c_int;
    len += 1;
    if len > 64 {
        len = 64;
        // WARNING: AltMissileSound too long in external WEAPONS.DAT '%s'
    }

    unsafe {
        Q_strncpyz(
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
            tokenStr,
            len,
        );
    }
}

//--------------------------------------------
pub fn WPN_MissileLightColor(holdBuf: *mut *const c_char) {
    let mut i: c_int;
    let mut tokenFlt: c_float = 0.0;

    i = 0;
    while i < 3 {
        if COM_ParseFloat(holdBuf, &mut tokenFlt) != 0 {
            SkipRestOfLine(holdBuf);
            i += 1;
            continue;
        }

        if tokenFlt < 0.0 || tokenFlt > 1.0 {
            // WARNING: bad missilelightcolor in external weapon data '%f'
            i += 1;
            continue;
        }
        unsafe {
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().missileDlightColor[i as usize] = tokenFlt;
        }
        i += 1;
    }
}

//--------------------------------------------
pub fn WPN_AltMissileLightColor(holdBuf: *mut *const c_char) {
    let mut i: c_int;
    let mut tokenFlt: c_float = 0.0;

    i = 0;
    while i < 3 {
        if COM_ParseFloat(holdBuf, &mut tokenFlt) != 0 {
            SkipRestOfLine(holdBuf);
            i += 1;
            continue;
        }

        if tokenFlt < 0.0 || tokenFlt > 1.0 {
            // WARNING: bad altmissilelightcolor in external weapon data '%f'
            i += 1;
            continue;
        }
        unsafe {
            (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().alt_missileDlightColor[i as usize] = tokenFlt;
        }
        i += 1;
    }
}

//--------------------------------------------
pub fn WPN_MissileLight(holdBuf: *mut *const c_char) {
    let mut tokenFlt: c_float = 0.0;

    if COM_ParseFloat(holdBuf, &mut tokenFlt) != 0 {
        SkipRestOfLine(holdBuf);
    }

    if tokenFlt < 0.0 || tokenFlt > 255.0 {
        // FIXME :What are the right values?
        // WARNING: bad missilelight in external weapon data '%f'
    }
    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().missileDlight = tokenFlt;
    }
}

//--------------------------------------------
pub fn WPN_AltMissileLight(holdBuf: *mut *const c_char) {
    let mut tokenFlt: c_float = 0.0;

    if COM_ParseFloat(holdBuf, &mut tokenFlt) != 0 {
        SkipRestOfLine(holdBuf);
    }

    if tokenFlt < 0.0 || tokenFlt > 255.0 {
        // FIXME :What are the right values?
        // WARNING: bad altmissilelight in external weapon data '%f'
    }
    unsafe {
        (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr().as_mut().unwrap().alt_missileDlight = tokenFlt;
    }
}

//--------------------------------------------
pub fn WPN_FuncName(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }
    // ONLY DO THIS ON THE GAME SIDE
    #[cfg(not(feature = "ui_dll"))]
    {
        let len = strlen(tokenStr) as c_int + 1;

        if len > 64 {
            // WARNING: FuncName '%s' too long in external WEAPONS.DAT
        }

        unsafe {
            let mut s = funcs.as_mut_ptr();
            while !(*s).name.is_null() {
                if Q_stricmp((*s).name, tokenStr) == 0 {
                    // found it
                    // Assign function pointer
                    return;
                }
                s = s.add(1);
            }
        }
        // WARNING: FuncName '%s' in external WEAPONS.DAT does not exist
    }
}

//--------------------------------------------
pub fn WPN_AltFuncName(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }

    // ONLY DO THIS ON THE GAME SIDE
    #[cfg(not(feature = "ui_dll"))]
    {
        let len = strlen(tokenStr) as c_int + 1;
        if len > 64 {
            // WARNING: AltFuncName '%s' too long in external WEAPONS.DAT
        }

        unsafe {
            let mut s = funcs.as_mut_ptr();
            while !(*s).name.is_null() {
                if Q_stricmp((*s).name, tokenStr) == 0 {
                    // found it
                    // Assign alt function pointer
                    return;
                }
                s = s.add(1);
            }
        }
        // WARNING: AltFuncName %s in external WEAPONS.DAT does not exist
    }
}

//--------------------------------------------
pub fn WPN_MuzzleEffect(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }
    // ONLY DO THIS ON THE GAME SIDE
    #[cfg(not(feature = "ui_dll"))]
    {
        let len = strlen(tokenStr) as c_int + 1;

        if len > 64 {
            // WARNING: MuzzleEffect '%s' too long in external WEAPONS.DAT
        }

        unsafe {
            G_EffectIndex(tokenStr);
            Q_strncpyz(
                (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
                tokenStr,
                len,
            );
        }
    }
}

//--------------------------------------------
pub fn WPN_AltMuzzleEffect(holdBuf: *mut *const c_char) {
    let mut tokenStr: *const c_char = core::ptr::null();

    if COM_ParseString(holdBuf, &mut tokenStr) != 0 {
        return;
    }
    // ONLY DO THIS ON THE GAME SIDE
    #[cfg(not(feature = "ui_dll"))]
    {
        let len = strlen(tokenStr) as c_int + 1;

        if len > 64 {
            // WARNING: AltMuzzleEffect '%s' too long in external WEAPONS.DAT
        }

        unsafe {
            G_EffectIndex(tokenStr);
            Q_strncpyz(
                (*core::ptr::addr_of_mut!(weaponData)).as_mut_ptr() as *mut c_char,
                tokenStr,
                len,
            );
        }
    }
}

//--------------------------------------------
#[allow(non_upper_case_globals)]
static mut WpnParms: [wpnParms_t; 47] = [
    wpnParms_t { parmName: b"ammo\0".as_ptr() as *const c_char, func: Some(WPN_Ammo) },
    wpnParms_t { parmName: b"ammoicon\0".as_ptr() as *const c_char, func: Some(WPN_AmmoIcon) },
    wpnParms_t { parmName: b"ammomax\0".as_ptr() as *const c_char, func: Some(WPN_AmmoMax) },
    wpnParms_t { parmName: b"ammolowcount\0".as_ptr() as *const c_char, func: Some(WPN_AmmoLowCnt) }, //weapons
    wpnParms_t { parmName: b"ammotype\0".as_ptr() as *const c_char, func: Some(WPN_AmmoType) },
    wpnParms_t { parmName: b"energypershot\0".as_ptr() as *const c_char, func: Some(WPN_EnergyPerShot) },
    wpnParms_t { parmName: b"fireTime\0".as_ptr() as *const c_char, func: Some(WPN_FireTime) },
    wpnParms_t { parmName: b"firingsound\0".as_ptr() as *const c_char, func: Some(WPN_FiringSnd) },
    wpnParms_t { parmName: b"altfiringsound\0".as_ptr() as *const c_char, func: Some(WPN_AltFiringSnd) },
    wpnParms_t { parmName: b"stopsound\0".as_ptr() as *const c_char, func: Some(WPN_StopSnd) },
    wpnParms_t { parmName: b"chargesound\0".as_ptr() as *const c_char, func: Some(WPN_ChargeSnd) },
    wpnParms_t { parmName: b"altchargesound\0".as_ptr() as *const c_char, func: Some(WPN_AltChargeSnd) },
    wpnParms_t { parmName: b"selectsound\0".as_ptr() as *const c_char, func: Some(WPN_SelectSnd) },
    wpnParms_t { parmName: b"range\0".as_ptr() as *const c_char, func: Some(WPN_Range) },
    wpnParms_t { parmName: b"weaponclass\0".as_ptr() as *const c_char, func: Some(WPN_WeaponClass) },
    wpnParms_t { parmName: b"weaponicon\0".as_ptr() as *const c_char, func: Some(WPN_WeaponIcon) },
    wpnParms_t { parmName: b"weaponmodel\0".as_ptr() as *const c_char, func: Some(WPN_WeaponModel) },
    wpnParms_t { parmName: b"weapontype\0".as_ptr() as *const c_char, func: Some(WPN_WeaponType) },
    wpnParms_t { parmName: b"altenergypershot\0".as_ptr() as *const c_char, func: Some(WPN_AltEnergyPerShot) },
    wpnParms_t { parmName: b"altfireTime\0".as_ptr() as *const c_char, func: Some(WPN_AltFireTime) },
    wpnParms_t { parmName: b"altrange\0".as_ptr() as *const c_char, func: Some(WPN_AltRange) },
    wpnParms_t { parmName: b"barrelcount\0".as_ptr() as *const c_char, func: Some(WPN_BarrelCount) },
    wpnParms_t { parmName: b"missileModel\0".as_ptr() as *const c_char, func: Some(WPN_MissileName) },
    wpnParms_t { parmName: b"altmissileModel\0".as_ptr() as *const c_char, func: Some(WPN_AltMissileName) },
    wpnParms_t { parmName: b"missileSound\0".as_ptr() as *const c_char, func: Some(WPN_MissileSound) },
    wpnParms_t { parmName: b"altmissileSound\0".as_ptr() as *const c_char, func: Some(WPN_AltMissileSound) },
    wpnParms_t { parmName: b"missileLight\0".as_ptr() as *const c_char, func: Some(WPN_MissileLight) },
    wpnParms_t { parmName: b"altmissileLight\0".as_ptr() as *const c_char, func: Some(WPN_AltMissileLight) },
    wpnParms_t { parmName: b"missileLightColor\0".as_ptr() as *const c_char, func: Some(WPN_MissileLightColor) },
    wpnParms_t { parmName: b"altmissileLightColor\0".as_ptr() as *const c_char, func: Some(WPN_AltMissileLightColor) },
    wpnParms_t { parmName: b"missileFuncName\0".as_ptr() as *const c_char, func: Some(WPN_FuncName) },
    wpnParms_t { parmName: b"altmissileFuncName\0".as_ptr() as *const c_char, func: Some(WPN_AltFuncName) },
    wpnParms_t { parmName: b"missileHitSound\0".as_ptr() as *const c_char, func: Some(WPN_MissileHitSound) },
    wpnParms_t { parmName: b"altmissileHitSound\0".as_ptr() as *const c_char, func: Some(WPN_AltMissileHitSound) },
    wpnParms_t { parmName: b"muzzleEffect\0".as_ptr() as *const c_char, func: Some(WPN_MuzzleEffect) },
    wpnParms_t { parmName: b"altmuzzleEffect\0".as_ptr() as *const c_char, func: Some(WPN_AltMuzzleEffect) },
    //#ifdef _IMMERSION
    wpnParms_t { parmName: b"firingForce\0".as_ptr() as *const c_char, func: Some(WPN_FiringFrc) },
    wpnParms_t { parmName: b"altFiringForce\0".as_ptr() as *const c_char, func: Some(WPN_AltFiringFrc) },
    wpnParms_t { parmName: b"chargeForce\0".as_ptr() as *const c_char, func: Some(WPN_ChargeFrc) },
    wpnParms_t { parmName: b"altChargeForce\0".as_ptr() as *const c_char, func: Some(WPN_AltChargeFrc) },
    wpnParms_t { parmName: b"stopForce\0".as_ptr() as *const c_char, func: Some(WPN_StopFrc) },
    wpnParms_t { parmName: b"selectForce\0".as_ptr() as *const c_char, func: Some(WPN_SelectFrc) },
    //#endif // _IMMERSION
];

const WPN_PARM_MAX: usize = 47; // sizeof(WpnParms) / sizeof(WpnParms[0])

//--------------------------------------------
fn WP_ParseParms(buffer: *const c_char) {
    let mut holdBuf = buffer;
    let mut token: *const c_char;

    COM_BeginParseSession();

    loop {
        token = COM_ParseExt(&mut holdBuf, 1);

        if Q_stricmp(token, b"{\0".as_ptr() as *const c_char) == 0 {
            WP_ParseWeaponParms(&mut holdBuf);
        }

        if token.is_null() {
            break;
        }
    }
}

//--------------------------------------------
pub fn WP_LoadWeaponParms() {
    let mut buffer: *mut c_char = core::ptr::null_mut();
    let len: c_int;

    len = unsafe { FS_ReadFile(b"ext_data/weapons.dat\0".as_ptr() as *const c_char, &mut buffer as *mut _ as *mut *mut c_void) };

    if len == -1 {
        Com_Error(ERR_FATAL, b"Cannot find ext_data/weapons.dat!\n\0".as_ptr() as *const c_char);
    }

    // initialise the data area
    unsafe {
        core::ptr::write_bytes(
            weaponData.as_mut_ptr() as *mut c_void,
            0,
            (35 * core::mem::size_of::<weaponData_t>()) as usize,
        );
    }

    unsafe {
        WP_ParseParms(buffer);
    }

    unsafe {
        FS_FreeFile(buffer as *mut c_void); //let go of the buffer
    }
}
