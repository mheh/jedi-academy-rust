/*****************************************************************************
 * name:		be_ai_weap.c
 *
 * desc:		weapon AI
 *
 * $Archive: /MissionPack/code/botlib/be_ai_weap.c $
 * $Author: Ttimo $
 * $Revision: 6 $
 * $Modtime: 4/13/01 4:45p $
 * $Date: 4/13/01 4:45p $
 *
 *****************************************************************************/

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void};
use core::ptr::{addr_of, addr_of_mut};

// === Local Type Stubs ===
// These types are defined in other modules but declared here for structural coherence

#[repr(C)]
pub struct projectileinfo_t {
    pub name: [c_char; 64],        // typical MAX_QPATH
    pub model: [c_char; 64],       // typical MAX_QPATH
    pub flags: c_int,
    pub gravity: f32,
    pub damage: c_int,
    pub radius: f32,
    pub visdamage: c_int,
    pub damagetype: c_int,
    pub healthinc: c_int,
    pub push: f32,
    pub detonation: f32,
    pub bounce: f32,
    pub bouncefric: f32,
    pub bouncestop: f32,
}

#[repr(C)]
pub struct weaponinfo_t {
    pub number: c_int,
    pub name: [c_char; 64],        // typical MAX_QPATH
    pub level: c_int,
    pub model: [c_char; 64],       // typical MAX_QPATH
    pub weaponindex: c_int,
    pub flags: c_int,
    pub projectile: [c_char; 64],  // typical MAX_QPATH
    pub numprojectiles: c_int,
    pub hspread: f32,
    pub vspread: f32,
    pub speed: f32,
    pub acceleration: f32,
    pub recoil: [f32; 3],
    pub offset: [f32; 3],
    pub angleoffset: [f32; 3],
    pub extrazvelocity: f32,
    pub ammoamount: c_int,
    pub ammoindex: c_int,
    pub activate: f32,
    pub reload: f32,
    pub spinup: f32,
    pub spindown: f32,
    pub valid: c_int,
    pub proj: projectileinfo_t,
}

#[repr(C)]
pub struct fielddef_t {
    pub name: *const c_char,
    pub offset: c_int,
    pub type_: c_int,
    pub size: c_int,
}

#[repr(C)]
pub struct structdef_t {
    pub size: usize,
    pub fields: *const fielddef_t,
}

#[repr(C)]
pub struct token_t {
    // Stub: fields needed from l_script.h
    pub string: [c_char; 1024],  // token string
}

#[repr(C)]
pub struct weightconfig_s {
    // Stub: actual definition from be_ai_weight.h
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct botimport_t {
    // Stub: actual definition from be_interface.h
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct weaponconfig_s {
    pub numweapons: c_int,
    pub numprojectiles: c_int,
    pub projectileinfo: *mut projectileinfo_t,
    pub weaponinfo: *mut weaponinfo_t,
}

pub type weaponconfig_t = weaponconfig_s;

#[repr(C)]
pub struct bot_weaponstate_s {
    pub weaponweightconfig: *mut weightconfig_s,  // weapon weight configuration
    pub weaponweightindex: *mut c_int,            // weapon weight index
}

pub type bot_weaponstate_t = bot_weaponstate_s;

// === External C Functions ===

extern "C" {
    fn Com_Memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void;
    fn Com_Memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;

    fn LibVarValue(name: *const c_char, default: *const c_char) -> f64;
    fn LibVarSet(name: *const c_char, value: *const c_char);
    fn LibVarString(name: *const c_char, default: *const c_char) -> *const c_char;

    fn Log_FileStruct() -> *mut c_void;
    fn Log_Flush();

    fn GetClearedHunkMemory(size: usize) -> *mut c_void;
    fn GetClearedMemory(size: usize) -> *mut c_void;
    fn FreeMemory(ptr: *mut c_void);

    fn PC_SetBaseFolder(folder: *const c_char);
    fn PC_ReadToken(source: *mut c_void, token: *mut token_t) -> c_int;

    fn LoadSourceFile(name: *const c_char) -> *mut c_void;
    fn FreeSource(source: *mut c_void);

    fn ReadStructure(source: *mut c_void, def: *const structdef_t, ptr: *mut c_char) -> c_int;
    fn WriteStructure(fp: *mut c_void, def: *const structdef_t, ptr: *const c_char);

    fn ReadWeightConfig(name: *const c_char) -> *mut weightconfig_s;
    fn FreeWeightConfig(config: *mut weightconfig_s);
    fn FuzzyWeight(inventory: *const c_int, config: *const weightconfig_s, index: c_int) -> f32;
    fn FindFuzzyWeight(config: *const weightconfig_s, name: *const c_char) -> c_int;

    pub static botimport: botimport_t;
}

// === Local Constants and Stubs ===

const MAX_PATH: usize = 256;
const MAX_CLIENTS: usize = 64;

// Stub constants - actual values would come from headers
const BOTFILESBASEFOLDER: &[u8] = b"botfiles\0";

// Field type constants (stubs - from l_struct.h)
const FT_INT: c_int = 1;
const FT_FLOAT: c_int = 2;
const FT_STRING: c_int = 4;
const FT_ARRAY: c_int = 8;

// Bot library error codes (from botlib.h)
const BLERR_NOERROR: c_int = 0;
const BLERR_CANNOTLOADWEAPONWEIGHTS: c_int = 1;
const BLERR_CANNOTLOADWEAPONCONFIG: c_int = 2;

// Boolean constants (from q_shared.h)
const qtrue: c_int = 1;
const qfalse: c_int = 0;

// === Stub function that is never called in this module ===
// Since botimport is external and has opaque type, we provide a workaround
fn botimport_Print(level: c_int, format: *const c_char, args: ...) {
    // This would be: unsafe { (*addr_of!(botimport)).Print(level, format, ...) }
    // But we can't call it without knowing the actual function pointer layout
}

// === Macro translations ===

// structure field offsets
// Note: These macros are used to initialize fielddef_t arrays.
// The actual offset values need to match the C struct layout.
// Since we don't have the struct definitions, these are stubbed with 0.
// They will be computed correctly when the actual struct headers are ported.
#[inline(always)]
fn WEAPON_OFS(_field: &str) -> c_int {
    // Placeholder - would compute (int)&(((weaponinfo_t *)0)->field)
    // Actual offsets depend on weaponinfo_t layout
    0
}

#[inline(always)]
fn PROJECTILE_OFS(_field: &str) -> c_int {
    // Placeholder - would compute (int)&(((projectileinfo_t *)0)->field)
    // Actual offsets depend on projectileinfo_t layout
    0
}

// === Global Variables ===

//weapon definition // bk001212 - static
static WEAPONINFO_FIELDS: &[fielddef_t] = &[
    fielddef_t {
        name: b"number\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("number"),
        type_: FT_INT,
        size: 0,
    }, //weapon number
    fielddef_t {
        name: b"name\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("name"),
        type_: FT_STRING,
        size: 0,
    }, //name of the weapon
    fielddef_t {
        name: b"level\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("level"),
        type_: FT_INT,
        size: 0,
    },
    fielddef_t {
        name: b"model\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("model"),
        type_: FT_STRING,
        size: 0,
    }, //model of the weapon
    fielddef_t {
        name: b"weaponindex\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("weaponindex"),
        type_: FT_INT,
        size: 0,
    }, //index of weapon in inventory
    fielddef_t {
        name: b"flags\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("flags"),
        type_: FT_INT,
        size: 0,
    }, //special flags
    fielddef_t {
        name: b"projectile\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("projectile"),
        type_: FT_STRING,
        size: 0,
    }, //projectile used by the weapon
    fielddef_t {
        name: b"numprojectiles\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("numprojectiles"),
        type_: FT_INT,
        size: 0,
    }, //number of projectiles
    fielddef_t {
        name: b"hspread\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("hspread"),
        type_: FT_FLOAT,
        size: 0,
    }, //horizontal spread of projectiles (degrees from middle)
    fielddef_t {
        name: b"vspread\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("vspread"),
        type_: FT_FLOAT,
        size: 0,
    }, //vertical spread of projectiles (degrees from middle)
    fielddef_t {
        name: b"speed\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("speed"),
        type_: FT_FLOAT,
        size: 0,
    }, //speed of the projectile (0 = instant hit)
    fielddef_t {
        name: b"acceleration\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("acceleration"),
        type_: FT_FLOAT,
        size: 0,
    }, //"acceleration" * time (in seconds) + "speed" = projectile speed
    fielddef_t {
        name: b"recoil\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("recoil"),
        type_: FT_FLOAT | FT_ARRAY,
        size: 3,
    }, //amount of recoil the player gets from the weapon
    fielddef_t {
        name: b"offset\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("offset"),
        type_: FT_FLOAT | FT_ARRAY,
        size: 3,
    }, //projectile start offset relative to eye and view angles
    fielddef_t {
        name: b"angleoffset\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("angleoffset"),
        type_: FT_FLOAT | FT_ARRAY,
        size: 3,
    }, //offset of the shoot angles relative to the view angles
    fielddef_t {
        name: b"extrazvelocity\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("extrazvelocity"),
        type_: FT_FLOAT,
        size: 0,
    }, //extra z velocity the projectile gets
    fielddef_t {
        name: b"ammoamount\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("ammoamount"),
        type_: FT_INT,
        size: 0,
    }, //ammo amount used per shot
    fielddef_t {
        name: b"ammoindex\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("ammoindex"),
        type_: FT_INT,
        size: 0,
    }, //index of ammo in inventory
    fielddef_t {
        name: b"activate\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("activate"),
        type_: FT_FLOAT,
        size: 0,
    }, //time it takes to select the weapon
    fielddef_t {
        name: b"reload\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("reload"),
        type_: FT_FLOAT,
        size: 0,
    }, //time it takes to reload the weapon
    fielddef_t {
        name: b"spinup\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("spinup"),
        type_: FT_FLOAT,
        size: 0,
    }, //time it takes before first shot
    fielddef_t {
        name: b"spindown\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("spindown"),
        type_: FT_FLOAT,
        size: 0,
    }, //time it takes before weapon stops firing
    fielddef_t {
        name: std::ptr::null(),
        offset: 0,
        type_: 0,
        size: 0,
    },
];

//projectile definition
static PROJECTILEINFO_FIELDS: &[fielddef_t] = &[
    fielddef_t {
        name: b"name\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("name"),
        type_: FT_STRING,
        size: 0,
    }, //name of the projectile
    fielddef_t {
        name: b"model\0".as_ptr() as *const c_char,
        offset: WEAPON_OFS("model"),
        type_: FT_STRING,
        size: 0,
    }, //model of the projectile
    fielddef_t {
        name: b"flags\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("flags"),
        type_: FT_INT,
        size: 0,
    }, //special flags
    fielddef_t {
        name: b"gravity\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("gravity"),
        type_: FT_FLOAT,
        size: 0,
    }, //amount of gravity applied to the projectile [0,1]
    fielddef_t {
        name: b"damage\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("damage"),
        type_: FT_INT,
        size: 0,
    }, //damage of the projectile
    fielddef_t {
        name: b"radius\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("radius"),
        type_: FT_FLOAT,
        size: 0,
    }, //radius of damage
    fielddef_t {
        name: b"visdamage\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("visdamage"),
        type_: FT_INT,
        size: 0,
    }, //damage of the projectile to visible entities
    fielddef_t {
        name: b"damagetype\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("damagetype"),
        type_: FT_INT,
        size: 0,
    }, //type of damage (combination of the DAMAGETYPE_? flags)
    fielddef_t {
        name: b"healthinc\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("healthinc"),
        type_: FT_INT,
        size: 0,
    }, //health increase the owner gets
    fielddef_t {
        name: b"push\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("push"),
        type_: FT_FLOAT,
        size: 0,
    }, //amount a player is pushed away from the projectile impact
    fielddef_t {
        name: b"detonation\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("detonation"),
        type_: FT_FLOAT,
        size: 0,
    }, //time before projectile explodes after fire pressed
    fielddef_t {
        name: b"bounce\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("bounce"),
        type_: FT_FLOAT,
        size: 0,
    }, //amount the projectile bounces
    fielddef_t {
        name: b"bouncefric\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("bouncefric"),
        type_: FT_FLOAT,
        size: 0,
    }, //amount the bounce decreases per bounce
    fielddef_t {
        name: b"bouncestop\0".as_ptr() as *const c_char,
        offset: PROJECTILE_OFS("bouncestop"),
        type_: FT_FLOAT,
        size: 0,
    }, //minimum bounce value before bouncing stops
    //recurive projectile definition??
    fielddef_t {
        name: std::ptr::null(),
        offset: 0,
        type_: 0,
        size: 0,
    },
];

// Note: These are initialized with field pointers to the static arrays
// The actual offset values in the fields will need to be computed when struct layouts are known
static WEAPONINFO_STRUCT: structdef_t = structdef_t {
    size: std::mem::size_of::<weaponinfo_t>(),
    fields: WEAPONINFO_FIELDS.as_ptr(),
};

static PROJECTILEINFO_STRUCT: structdef_t = structdef_t {
    size: std::mem::size_of::<projectileinfo_t>(),
    fields: PROJECTILEINFO_FIELDS.as_ptr(),
};

static mut BOTWEAPONSTATES: [*mut bot_weaponstate_t; MAX_CLIENTS + 1] = [std::ptr::null_mut(); MAX_CLIENTS + 1];
static mut WEAPONCONFIG: *mut weaponconfig_t = std::ptr::null_mut();

//========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//========================================================================
pub fn BotValidWeaponNumber(weaponnum: c_int) -> c_int {
    unsafe {
        if weaponnum <= 0 || weaponnum > (*WEAPONCONFIG).numweapons {
            // botimport.Print(PRT_ERROR, "weapon number out of range\n");
            return qfalse;
        } //end if
    }
    return qtrue;
} //end of the function BotValidWeaponNumber
  //========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //========================================================================
pub fn BotWeaponStateFromHandle(handle: c_int) -> *mut bot_weaponstate_t {
    unsafe {
        if handle <= 0 || handle > MAX_CLIENTS as c_int {
            // botimport.Print(PRT_FATAL, "move state handle %d out of range\n", handle);
            return std::ptr::null_mut();
        } //end if
        if BOTWEAPONSTATES[handle as usize].is_null() {
            // botimport.Print(PRT_FATAL, "invalid move state %d\n", handle);
            return std::ptr::null_mut();
        } //end if
        return BOTWEAPONSTATES[handle as usize];
    }
} //end of the function BotWeaponStateFromHandle
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
#[cfg(DEBUG_AI_WEAP)]
pub fn DumpWeaponConfig(wc: *mut weaponconfig_t) {
    unsafe {
        let fp = Log_FileStruct();
        if fp.is_null() {
            return;
        }
        for i in 0..(*wc).numprojectiles {
            WriteStructure(
                fp,
                addr_of!(PROJECTILEINFO_STRUCT),
                (*wc).projectileinfo.add(i as usize) as *const c_char,
            );
            Log_Flush();
        } //end for
        for i in 0..(*wc).numweapons {
            WriteStructure(
                fp,
                addr_of!(WEAPONINFO_STRUCT),
                (*wc).weaponinfo.add(i as usize) as *const c_char,
            );
            Log_Flush();
        } //end for
    }
} //end of the function DumpWeaponConfig
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn LoadWeaponConfig(filename: *const c_char) -> *mut weaponconfig_t {
    unsafe {
        let mut max_weaponinfo: c_int;
        let mut max_projectileinfo: c_int;
        let mut token: token_t = std::mem::zeroed();
        let mut path: [c_char; MAX_PATH] = [0; MAX_PATH];
        let mut i: c_int;
        let mut j: c_int;
        let source: *mut c_void;
        let wc: *mut weaponconfig_t;
        let mut weaponinfo: weaponinfo_t = std::mem::zeroed();

        max_weaponinfo = LibVarValue(b"max_weaponinfo\0".as_ptr() as *const c_char, b"32\0".as_ptr() as *const c_char) as c_int;
        if max_weaponinfo < 0 {
            // botimport.Print(PRT_ERROR, "max_weaponinfo = %d\n", max_weaponinfo);
            max_weaponinfo = 32;
            LibVarSet(b"max_weaponinfo\0".as_ptr() as *const c_char, b"32\0".as_ptr() as *const c_char);
        } //end if
        max_projectileinfo = LibVarValue(b"max_projectileinfo\0".as_ptr() as *const c_char, b"32\0".as_ptr() as *const c_char) as c_int;
        if max_projectileinfo < 0 {
            // botimport.Print(PRT_ERROR, "max_projectileinfo = %d\n", max_projectileinfo);
            max_projectileinfo = 32;
            LibVarSet(b"max_projectileinfo\0".as_ptr() as *const c_char, b"32\0".as_ptr() as *const c_char);
        } //end if

        // strncpy(path, filename, MAX_PATH);
        let filename_str = std::ffi::CStr::from_ptr(filename);
        let filename_bytes = filename_str.to_bytes();
        let copy_len = if filename_bytes.len() < MAX_PATH { filename_bytes.len() } else { MAX_PATH - 1 };
        for i in 0..copy_len {
            path[i] = filename_bytes[i] as c_char;
        }
        if copy_len < MAX_PATH {
            path[copy_len] = 0;
        }

        PC_SetBaseFolder(BOTFILESBASEFOLDER.as_ptr() as *const c_char);
        source = LoadSourceFile(path.as_ptr());
        if source.is_null() {
            // botimport.Print(PRT_ERROR, "counldn't load %s\n", path);
            return std::ptr::null_mut();
        } //end if

        //initialize weapon config
        wc = GetClearedHunkMemory(
            std::mem::size_of::<weaponconfig_t>()
                + (max_weaponinfo as usize) * std::mem::size_of::<weaponinfo_t>()
                + (max_projectileinfo as usize) * std::mem::size_of::<projectileinfo_t>(),
        ) as *mut weaponconfig_t;

        (*wc).weaponinfo = (wc as *mut c_char).add(std::mem::size_of::<weaponconfig_t>()) as *mut weaponinfo_t;
        (*wc).projectileinfo = ((*wc).weaponinfo as *mut c_char)
            .add((max_weaponinfo as usize) * std::mem::size_of::<weaponinfo_t>())
            as *mut projectileinfo_t;
        (*wc).numweapons = max_weaponinfo;
        (*wc).numprojectiles = 0;

        //parse the source file
        while PC_ReadToken(source, addr_of_mut!(token)) != 0 {
            let token_string = token.string.as_ptr();
            if std::ffi::CStr::from_ptr(token_string).to_bytes() == b"weaponinfo" {
                Com_Memset(addr_of_mut!(weaponinfo) as *mut c_void, 0, std::mem::size_of::<weaponinfo_t>());
                if ReadStructure(source, addr_of!(WEAPONINFO_STRUCT), addr_of_mut!(weaponinfo) as *mut c_char) == 0 {
                    FreeMemory(wc as *mut c_void);
                    FreeSource(source);
                    return std::ptr::null_mut();
                } //end if
                if weaponinfo.number < 0 || weaponinfo.number >= max_weaponinfo {
                    // botimport.Print(PRT_ERROR, "weapon info number %d out of range in %s\n", weaponinfo.number, path);
                    FreeMemory(wc as *mut c_void);
                    FreeSource(source);
                    return std::ptr::null_mut();
                } //end if
                Com_Memcpy(
                    (*wc).weaponinfo.add(weaponinfo.number as usize) as *mut c_void,
                    addr_of!(weaponinfo) as *const c_void,
                    std::mem::size_of::<weaponinfo_t>(),
                );
                (*(*wc).weaponinfo.add(weaponinfo.number as usize)).valid = qtrue;
            } //end if
            else if std::ffi::CStr::from_ptr(token_string).to_bytes() == b"projectileinfo" {
                if (*wc).numprojectiles >= max_projectileinfo {
                    // botimport.Print(PRT_ERROR, "more than %d projectiles defined in %s\n", max_projectileinfo, path);
                    FreeMemory(wc as *mut c_void);
                    FreeSource(source);
                    return std::ptr::null_mut();
                } //end if
                Com_Memset(
                    (*wc).projectileinfo.add((*wc).numprojectiles as usize) as *mut c_void,
                    0,
                    std::mem::size_of::<projectileinfo_t>(),
                );
                if ReadStructure(
                    source,
                    addr_of!(PROJECTILEINFO_STRUCT),
                    (*wc).projectileinfo.add((*wc).numprojectiles as usize) as *mut c_char,
                ) == 0
                {
                    FreeMemory(wc as *mut c_void);
                    FreeSource(source);
                    return std::ptr::null_mut();
                } //end if
                (*wc).numprojectiles += 1;
            } //end if
            else {
                // botimport.Print(PRT_ERROR, "unknown definition %s in %s\n", token.string, path);
                FreeMemory(wc as *mut c_void);
                FreeSource(source);
                return std::ptr::null_mut();
            } //end else
        } //end while
        FreeSource(source);

        //fix up weapons
        i = 0;
        while i < (*wc).numweapons {
            if (*(*wc).weaponinfo.add(i as usize)).valid == 0 {
                i += 1;
                continue;
            }
            if (*(*wc).weaponinfo.add(i as usize)).name[0] == 0 {
                // botimport.Print(PRT_ERROR, "weapon %d has no name in %s\n", i, path);
                FreeMemory(wc as *mut c_void);
                return std::ptr::null_mut();
            } //end if
            if (*(*wc).weaponinfo.add(i as usize)).projectile[0] == 0 {
                // botimport.Print(PRT_ERROR, "weapon %s has no projectile in %s\n", wc->weaponinfo[i].name, path);
                FreeMemory(wc as *mut c_void);
                return std::ptr::null_mut();
            } //end if
            //find the projectile info and copy it to the weapon info
            j = 0;
            while j < (*wc).numprojectiles {
                let proj_name = std::ffi::CStr::from_ptr((*(*wc).projectileinfo.add(j as usize)).name.as_ptr());
                let weap_proj = std::ffi::CStr::from_ptr((*(*wc).weaponinfo.add(i as usize)).projectile.as_ptr());
                if proj_name.to_bytes() == weap_proj.to_bytes() {
                    Com_Memcpy(
                        addr_of_mut!((*(*wc).weaponinfo.add(i as usize)).proj) as *mut c_void,
                        addr_of!((*(*wc).projectileinfo.add(j as usize))) as *const c_void,
                        std::mem::size_of::<projectileinfo_t>(),
                    );
                    break;
                } //end if
                j += 1;
            } //end for
            if j == (*wc).numprojectiles {
                // botimport.Print(PRT_ERROR, "weapon %s uses undefined projectile in %s\n", wc->weaponinfo[i].name, path);
                FreeMemory(wc as *mut c_void);
                return std::ptr::null_mut();
            } //end if
            i += 1;
        } //end for

        // if (!wc->numweapons) botimport.Print(PRT_WARNING, "no weapon info loaded\n");
        // botimport.Print(PRT_MESSAGE, "loaded %s\n", path);
        return wc;
    }
} //end of the function LoadWeaponConfig
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn WeaponWeightIndex(
    wwc: *mut weightconfig_s,
    wc: *mut weaponconfig_t,
) -> *mut c_int {
    unsafe {
        let index: *mut c_int;
        let mut i: c_int;

        //initialize item weight index
        index = GetClearedMemory((std::mem::size_of::<c_int>() * ((*wc).numweapons as usize))) as *mut c_int;

        i = 0;
        while i < (*wc).numweapons {
            *index.add(i as usize) = FindFuzzyWeight(wwc, (*(*wc).weaponinfo.add(i as usize)).name.as_ptr());
            i += 1;
        } //end for
        return index;
    }
} //end of the function WeaponWeightIndex
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn BotFreeWeaponWeights(weaponstate: c_int) {
    unsafe {
        let ws: *mut bot_weaponstate_t;

        ws = BotWeaponStateFromHandle(weaponstate);
        if ws.is_null() {
            return;
        }
        if !(*ws).weaponweightconfig.is_null() {
            FreeWeightConfig((*ws).weaponweightconfig);
        }
        if !(*ws).weaponweightindex.is_null() {
            FreeMemory((*ws).weaponweightindex as *mut c_void);
        }
    }
} //end of the function BotFreeWeaponWeights
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn BotLoadWeaponWeights(weaponstate: c_int, filename: *const c_char) -> c_int {
    unsafe {
        let ws: *mut bot_weaponstate_t;

        ws = BotWeaponStateFromHandle(weaponstate);
        if ws.is_null() {
            return BLERR_CANNOTLOADWEAPONWEIGHTS;
        }
        BotFreeWeaponWeights(weaponstate);
        //
        (*ws).weaponweightconfig = ReadWeightConfig(filename);
        if (*ws).weaponweightconfig.is_null() {
            // botimport.Print(PRT_FATAL, "couldn't load weapon config %s\n", filename);
            return BLERR_CANNOTLOADWEAPONWEIGHTS;
        } //end if
        if WEAPONCONFIG.is_null() {
            return BLERR_CANNOTLOADWEAPONCONFIG;
        }
        (*ws).weaponweightindex = WeaponWeightIndex((*ws).weaponweightconfig, WEAPONCONFIG);
        return BLERR_NOERROR;
    }
} //end of the function BotLoadWeaponWeights
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn BotGetWeaponInfo(
    weaponstate: c_int,
    weapon: c_int,
    weaponinfo: *mut weaponinfo_t,
) {
    unsafe {
        let ws: *mut bot_weaponstate_t;

        if BotValidWeaponNumber(weapon) == 0 {
            return;
        }
        ws = BotWeaponStateFromHandle(weaponstate);
        if ws.is_null() {
            return;
        }
        if WEAPONCONFIG.is_null() {
            return;
        }
        Com_Memcpy(
            weaponinfo as *mut c_void,
            (*WEAPONCONFIG).weaponinfo.add(weapon as usize) as *const c_void,
            std::mem::size_of::<weaponinfo_t>(),
        );
    }
} //end of the function BotGetWeaponInfo
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn BotChooseBestFightWeapon(weaponstate: c_int, inventory: *const c_int) -> c_int {
    unsafe {
        let mut i: c_int;
        let mut index: c_int;
        let mut bestweapon: c_int;
        let mut weight: f32;
        let mut bestweight: f32;
        let wc: *mut weaponconfig_t;
        let ws: *mut bot_weaponstate_t;

        ws = BotWeaponStateFromHandle(weaponstate);
        if ws.is_null() {
            return 0;
        }
        wc = WEAPONCONFIG;
        if WEAPONCONFIG.is_null() {
            return 0;
        }

        //if the bot has no weapon weight configuration
        if (*ws).weaponweightconfig.is_null() {
            return 0;
        }

        bestweight = 0.0;
        bestweapon = 0;
        i = 0;
        while i < (*wc).numweapons {
            if (*(*wc).weaponinfo.add(i as usize)).valid == 0 {
                i += 1;
                continue;
            }
            index = *(*ws).weaponweightindex.add(i as usize);
            if index < 0 {
                i += 1;
                continue;
            }
            weight = FuzzyWeight(inventory, (*ws).weaponweightconfig, index);
            if weight > bestweight {
                bestweight = weight;
                bestweapon = i;
            } //end if
            i += 1;
        } //end for
        return bestweapon;
    }
} //end of the function BotChooseBestFightWeapon
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn BotResetWeaponState(weaponstate: c_int) {
    unsafe {
        let weaponweightconfig: *mut weightconfig_s;
        let weaponweightindex: *mut c_int;
        let ws: *mut bot_weaponstate_t;

        ws = BotWeaponStateFromHandle(weaponstate);
        if ws.is_null() {
            return;
        }
        weaponweightconfig = (*ws).weaponweightconfig;
        weaponweightindex = (*ws).weaponweightindex;

        //Com_Memset(ws, 0, sizeof(bot_weaponstate_t));
        (*ws).weaponweightconfig = weaponweightconfig;
        (*ws).weaponweightindex = weaponweightindex;
    }
} //end of the function BotResetWeaponState
  //========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //========================================================================
pub fn BotAllocWeaponState() -> c_int {
    unsafe {
        let mut i: c_int;

        i = 1;
        while i <= MAX_CLIENTS as c_int {
            if BOTWEAPONSTATES[i as usize].is_null() {
                BOTWEAPONSTATES[i as usize] = GetClearedMemory(std::mem::size_of::<bot_weaponstate_t>()) as *mut bot_weaponstate_t;
                return i;
            } //end if
            i += 1;
        } //end for
        return 0;
    }
} //end of the function BotAllocWeaponState
  //========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //========================================================================
pub fn BotFreeWeaponState(handle: c_int) {
    unsafe {
        if handle <= 0 || handle > MAX_CLIENTS as c_int {
            // botimport.Print(PRT_FATAL, "move state handle %d out of range\n", handle);
            return;
        } //end if
        if BOTWEAPONSTATES[handle as usize].is_null() {
            // botimport.Print(PRT_FATAL, "invalid move state %d\n", handle);
            return;
        } //end if
        BotFreeWeaponWeights(handle);
        FreeMemory(BOTWEAPONSTATES[handle as usize] as *mut c_void);
        BOTWEAPONSTATES[handle as usize] = std::ptr::null_mut();
    }
} //end of the function BotFreeWeaponState
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn BotSetupWeaponAI() -> c_int {
    unsafe {
        let file: *const c_char;

        file = LibVarString(b"weaponconfig\0".as_ptr() as *const c_char, b"weapons.c\0".as_ptr() as *const c_char);
        WEAPONCONFIG = LoadWeaponConfig(file);
        if WEAPONCONFIG.is_null() {
            // botimport.Print(PRT_FATAL, "couldn't load the weapon config\n");
            return BLERR_CANNOTLOADWEAPONCONFIG;
        } //end if

        #[cfg(DEBUG_AI_WEAP)]
        DumpWeaponConfig(WEAPONCONFIG);

        //
        return BLERR_NOERROR;
    }
} //end of the function BotSetupWeaponAI
  //===========================================================================
  //
  // Parameter:				-
  // Returns:					-
  // Changes Globals:		-
  //===========================================================================
pub fn BotShutdownWeaponAI() {
    unsafe {
        let mut i: c_int;

        if !WEAPONCONFIG.is_null() {
            FreeMemory(WEAPONCONFIG as *mut c_void);
        }
        WEAPONCONFIG = std::ptr::null_mut();

        i = 1;
        while i <= MAX_CLIENTS as c_int {
            if !BOTWEAPONSTATES[i as usize].is_null() {
                BotFreeWeaponState(i);
            } //end if
            i += 1;
        } //end for
    }
} //end of the function BotShutdownWeaponAI
