//bg_vehicleLoad.c

#[cfg(feature = "jk2")]
//SP does not have this preprocessor for game like MP does
#[cfg(all(feature = "jk2", not(feature = "jk2mp")))]
const _JK2MP: () = ();

#[cfg(feature = "jk2mp")]
mod _jk2mp_impl {
    use core::ffi::{c_int, c_char, c_void};

    //Could use strap stuff but I don't particularly care at the moment anyway.
    extern "C" {
        pub fn trap_FS_FOpenFile(qpath: *const c_char, f: *mut c_int, mode: c_int) -> c_int;
        pub fn trap_FS_Read(buffer: *mut c_void, len: c_int, f: c_int);
        pub fn trap_FS_Write(buffer: *const c_void, len: c_int, f: c_int);
        pub fn trap_FS_FCloseFile(f: c_int);
        pub fn trap_FS_GetFileList(path: *const c_char, extension: *const c_char, listbuf: *mut c_char, bufsize: c_int) -> c_int;
    }
}

#[cfg(not(feature = "jk2mp"))]
mod _not_jk2mp_impl {
    // When not in MP mode, we don't declare the trap functions here
    // they are expected to come from g_local.h via other means
}

#[cfg(feature = "jk2mp")]
use _jk2mp_impl::*;

#[cfg(all(not(feature = "jk2mp"), feature = "qagame"))]
const QAGAME: () = ();

#[cfg(feature = "jk2mp")]
#[cfg(all(not(feature = "qagame"), not(feature = "cgame")))]
const WE_ARE_IN_THE_UI: () = ();

#[cfg(not(feature = "jk2mp"))]
mod _ratl_string {
    // This is a stub for ratl::string_vs used in SP mode
    // Full implementation would be in a separate module
}

use core::ffi::{c_int, c_char, c_void};

// These buffers are filled in with the same contents and then just read from in
// a few places. We only need one copy on Xbox.
const MAX_VEH_WEAPON_DATA_SIZE: usize = 0x20000;
const MAX_VEHICLE_DATA_SIZE: usize = 0x80000;

#[cfg(any(not(cfg(feature = "xbox")), feature = "qagame"))]
pub static mut VehWeaponParms: [c_char; MAX_VEH_WEAPON_DATA_SIZE] = [0; MAX_VEH_WEAPON_DATA_SIZE];
#[cfg(any(not(cfg(feature = "xbox")), feature = "qagame"))]
pub static mut VehicleParms: [c_char; MAX_VEHICLE_DATA_SIZE] = [0; MAX_VEHICLE_DATA_SIZE];

#[cfg(any(not(cfg(feature = "xbox")), feature = "qagame"))]
pub fn BG_ClearVehicleParseParms() {
    //You can't strcat to these forever without clearing them!
    unsafe {
        VehWeaponParms[0] = 0;
        VehicleParms[0] = 0;
    }
}

#[cfg(all(feature = "xbox", not(feature = "qagame")))]
extern "C" {
    pub static VehWeaponParms: [c_char; MAX_VEH_WEAPON_DATA_SIZE];
    pub static VehicleParms: [c_char; MAX_VEHICLE_DATA_SIZE];
}

#[cfg(feature = "jk2mp")]
// ... namespace declarations would go here in actual C code
// For Rust, module organization handles this differently
#[cfg(not(feature = "WE_ARE_IN_THE_UI"))]
extern "C" {
    //These funcs are actually shared in both projects
    pub fn G_SetAnimalVehicleFunctions(pVehInfo: *mut crate::vehicleInfo_t);
    pub fn G_SetSpeederVehicleFunctions(pVehInfo: *mut crate::vehicleInfo_t);
    pub fn G_SetWalkerVehicleFunctions(pVehInfo: *mut crate::vehicleInfo_t);
    pub fn G_SetFighterVehicleFunctions(pVehInfo: *mut crate::vehicleInfo_t);
}

pub struct vehWeaponInfo_t {
    // Stub - actual definition should come from bg_vehicles.h
}

pub struct vehicleInfo_t {
    // Stub - actual definition should come from bg_vehicles.h
}

pub static mut g_vehWeaponInfo: [vehWeaponInfo_t; MAX_VEH_WEAPONS] = unsafe { core::mem::zeroed() };
pub static mut numVehicleWeapons: c_int = 1; //first one is null/default

pub static mut g_vehicleInfo: [vehicleInfo_t; MAX_VEHICLES] = unsafe { core::mem::zeroed() };
pub static mut numVehicles: c_int = 0; //first one is null/default

pub fn BG_VehicleLoadParms();

#[repr(C)]
#[derive(Copy, Clone)]
pub enum vehFieldType_t {
    VF_IGNORE = 0,
    VF_INT = 1,
    VF_FLOAT = 2,
    VF_LSTRING = 3,   // string on disk, pointer in memory, TAG_LEVEL
    VF_VECTOR = 4,
    VF_BOOL = 5,
    VF_VEHTYPE = 6,
    VF_ANIM = 7,
    VF_WEAPON = 8,    // take string, resolve into index into VehWeaponParms
    VF_MODEL = 9,     // take the string, get the G_ModelIndex
    VF_MODEL_CLIENT = 10,   // (cgame only) take the string, get the G_ModelIndex
    VF_EFFECT = 11,   // take the string, get the G_EffectIndex
    VF_EFFECT_CLIENT = 12,  // (cgame only) take the string, get the index
    VF_SHADER = 13,   // (cgame only) take the string, call trap_R_RegisterShader
    VF_SHADER_NOMIP = 14,   // (cgame only) take the string, call trap_R_RegisterShaderNoMip
    VF_SOUND = 15,    // take the string, get the G_SoundIndex
    VF_SOUND_CLIENT = 16,   // (cgame only) take the string, get the index
}

#[repr(C)]
pub struct vehField_t {
    pub name: *const c_char,
    pub ofs: c_int,
    pub type_: vehFieldType_t,
}

pub static vehWeaponFields: [vehField_t; NUM_VWEAP_PARMS] = [
    vehField_t { name: b"name\0".as_ptr() as _, ofs: VWFOFS_name, type_: vehFieldType_t::VF_LSTRING }, //unique name of the vehicle
    vehField_t { name: b"projectile\0".as_ptr() as _, ofs: VWFOFS_bIsProjectile, type_: vehFieldType_t::VF_BOOL }, //traceline or entity?
    vehField_t { name: b"hasGravity\0".as_ptr() as _, ofs: VWFOFS_bHasGravity, type_: vehFieldType_t::VF_BOOL }, //if a projectile, drops
    vehField_t { name: b"ionWeapon\0".as_ptr() as _, ofs: VWFOFS_bIonWeapon, type_: vehFieldType_t::VF_BOOL }, //disables ship shields and sends them out of control
    vehField_t { name: b"saberBlockable\0".as_ptr() as _, ofs: VWFOFS_bSaberBlockable, type_: vehFieldType_t::VF_BOOL }, //lightsabers can deflect this projectile
    vehField_t { name: b"muzzleFX\0".as_ptr() as _, ofs: VWFOFS_iMuzzleFX, type_: vehFieldType_t::VF_EFFECT_CLIENT }, //index of Muzzle Effect
    vehField_t { name: b"model\0".as_ptr() as _, ofs: VWFOFS_iModel, type_: vehFieldType_t::VF_MODEL_CLIENT }, //handle to the model used by this projectile
    vehField_t { name: b"shotFX\0".as_ptr() as _, ofs: VWFOFS_iShotFX, type_: vehFieldType_t::VF_EFFECT_CLIENT }, //index of Shot Effect
    vehField_t { name: b"impactFX\0".as_ptr() as _, ofs: VWFOFS_iImpactFX, type_: vehFieldType_t::VF_EFFECT_CLIENT }, //index of Impact Effect
    vehField_t { name: b"g2MarkShader\0".as_ptr() as _, ofs: VWFOFS_iG2MarkShaderHandle, type_: vehFieldType_t::VF_SHADER }, //index of shader to use for G2 marks made on other models when hit by this projectile
    vehField_t { name: b"g2MarkSize\0".as_ptr() as _, ofs: VWFOFS_fG2MarkSize, type_: vehFieldType_t::VF_FLOAT }, //size (diameter) of the ghoul2 mark
    vehField_t { name: b"loopSound\0".as_ptr() as _, ofs: VWFOFS_iLoopSound, type_: vehFieldType_t::VF_SOUND_CLIENT }, //index of loopSound
    vehField_t { name: b"speed\0".as_ptr() as _, ofs: VWFOFS_fSpeed, type_: vehFieldType_t::VF_FLOAT }, //speed of projectile/range of traceline
    vehField_t { name: b"homing\0".as_ptr() as _, ofs: VWFOFS_fHoming, type_: vehFieldType_t::VF_FLOAT }, //0.0 = not homing, 0.5 = half vel to targ, half cur vel, 1.0 = all vel to targ
    vehField_t { name: b"homingFOV\0".as_ptr() as _, ofs: VWFOFS_fHomingFOV, type_: vehFieldType_t::VF_FLOAT }, //missile will lose lock on if DotProduct of missile direction and direction to target ever drops below this (-1 to 1, -1 = never lose target, 0 = lose if ship gets behind missile, 1 = pretty much will lose it's target right away)
    vehField_t { name: b"lockOnTime\0".as_ptr() as _, ofs: VWFOFS_iLockOnTime, type_: vehFieldType_t::VF_INT }, //0 = no lock time needed, else # of ms needed to lock on
    vehField_t { name: b"damage\0".as_ptr() as _, ofs: VWFOFS_iDamage, type_: vehFieldType_t::VF_INT }, //damage done when traceline or projectile directly hits target
    vehField_t { name: b"splashDamage\0".as_ptr() as _, ofs: VWFOFS_iSplashDamage, type_: vehFieldType_t::VF_INT }, //damage done to ents in splashRadius of end of traceline or projectile origin on impact
    vehField_t { name: b"splashRadius\0".as_ptr() as _, ofs: VWFOFS_fSplashRadius, type_: vehFieldType_t::VF_FLOAT }, //radius that ent must be in to take splashDamage (linear fall-off)
    vehField_t { name: b"ammoPerShot\0".as_ptr() as _, ofs: VWFOFS_iAmmoPerShot, type_: vehFieldType_t::VF_INT }, //how much "ammo" each shot takes
    vehField_t { name: b"health\0".as_ptr() as _, ofs: VWFOFS_iHealth, type_: vehFieldType_t::VF_INT }, //if non-zero, projectile can be shot, takes this much damage before being destroyed
    vehField_t { name: b"width\0".as_ptr() as _, ofs: VWFOFS_fWidth, type_: vehFieldType_t::VF_FLOAT }, //width of traceline or bounding box of projecile (non-rotating!)
    vehField_t { name: b"height\0".as_ptr() as _, ofs: VWFOFS_fHeight, type_: vehFieldType_t::VF_FLOAT }, //height of traceline or bounding box of projecile (non-rotating!)
    vehField_t { name: b"lifetime\0".as_ptr() as _, ofs: VWFOFS_iLifeTime, type_: vehFieldType_t::VF_INT }, //removes itself after this amount of time
    vehField_t { name: b"explodeOnExpire\0".as_ptr() as _, ofs: VWFOFS_bExplodeOnExpire, type_: vehFieldType_t::VF_BOOL }, //when iLifeTime is up, explodes rather than simply removing itself
];

pub static BG_ParseVehWeaponParm: unsafe extern "C" fn(*mut vehWeaponInfo_t, *mut c_char, *mut c_char) -> c_int;

pub fn BG_ParseVehWeaponParm(
    vehWeapon: *mut vehWeaponInfo_t,
    parmName: *mut c_char,
    pValue: *mut c_char,
) -> c_int {
    unsafe {
        let mut i: c_int;
        let mut vec: [f32; 3] = [0.0; 3];
        let b = vehWeapon as *mut u8;
        let mut _iFieldsRead: c_int = 0;
        let mut vehType: vehicleType_t;
        let mut value: [c_char; 1024] = [0; 1024];

        Q_strncpyz(value.as_mut_ptr(), pValue, core::mem::size_of_val(&value) as c_int);

        // Loop through possible parameters
        i = 0;
        while i < NUM_VWEAP_PARMS as c_int {
            if !vehWeaponFields[i as usize].name.is_null()
                && Q_stricmp(
                    vehWeaponFields[i as usize].name,
                    parmName,
                ) == 0
            {
                // found it
                match vehWeaponFields[i as usize].type_ {
                    vehFieldType_t::VF_INT => {
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                            atoi(value.as_ptr());
                    }
                    vehFieldType_t::VF_FLOAT => {
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut f32) =
                            atof(value.as_ptr());
                    }
                    vehFieldType_t::VF_LSTRING => {
                        // string on disk, pointer in memory, TAG_LEVEL
                        if (*(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut *mut c_char))
                            .is_null()
                        {
                            //just use 1024 bytes in case we want to write over the string
                            #[cfg(feature = "jk2mp")]
                            {
                                *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut *mut c_char) =
                                    BG_Alloc(1024) as *mut c_char; //(char *)BG_Alloc(strlen(value));
                                strcpy(
                                    *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut *mut c_char),
                                    value.as_ptr(),
                                );
                            }
                            #[cfg(not(feature = "jk2mp"))]
                            {
                                *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut *mut c_char) =
                                    G_NewString(value.as_ptr());
                            }
                        }
                    }
                    vehFieldType_t::VF_VECTOR => {
                        _iFieldsRead = sscanf(
                            value.as_ptr(),
                            b"%f %f %f\0".as_ptr() as _,
                            &mut vec[0] as *mut f32,
                            &mut vec[1] as *mut f32,
                            &mut vec[2] as *mut f32,
                        );
                        assert!(_iFieldsRead == 3);
                        if _iFieldsRead != 3 {
                            Com_Printf(
                                b"BG_ParseVehWeaponParm: VEC3 sscanf() failed to read 3 floats ('angle' key bug?)\n\0"
                                    .as_ptr() as _,
                            );
                        }
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut f32) = vec[0];
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut f32).add(1) = vec[1];
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut f32).add(2) = vec[2];
                    }
                    vehFieldType_t::VF_BOOL => {
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                            (atof(value.as_ptr()) != 0.0) as c_int;
                    }
                    vehFieldType_t::VF_VEHTYPE => {
                        vehType = GetIDForString(VehicleTable.as_ptr(), value.as_ptr()) as vehicleType_t;
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut vehicleType_t) = vehType;
                    }
                    vehFieldType_t::VF_ANIM => {
                        let anim = GetIDForString(animTable.as_ptr(), value.as_ptr());
                        *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) = anim;
                    }
                    vehFieldType_t::VF_WEAPON => {
                        // take string, resolve into index into VehWeaponParms
                        //*(int *)(b+vehWeaponFields[i].ofs) = VEH_VehWeaponIndexForName( value );
                    }
                    vehFieldType_t::VF_MODEL => {
                        // take the string, get the G_ModelIndex
                        #[cfg(feature = "qagame")]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                G_ModelIndex(value.as_ptr());
                        }
                        #[cfg(not(feature = "qagame"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_R_RegisterModel(value.as_ptr());
                        }
                    }
                    vehFieldType_t::VF_MODEL_CLIENT => {
                        // (MP cgame only) take the string, get the G_ModelIndex
                        #[cfg(not(feature = "jk2mp"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                G_ModelIndex(value.as_ptr());
                        }
                        #[cfg(all(feature = "jk2mp", feature = "qagame"))]
                        {
                            //*(int *)(b+vehWeaponFields[i].ofs) = G_ModelIndex( value );
                        }
                        #[cfg(all(feature = "jk2mp", not(feature = "qagame")))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_R_RegisterModel(value.as_ptr());
                        }
                    }
                    vehFieldType_t::VF_EFFECT => {
                        // take the string, get the G_EffectIndex
                        #[cfg(feature = "qagame")]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                G_EffectIndex(value.as_ptr());
                        }
                        #[cfg(feature = "cgame")]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_FX_RegisterEffect(value.as_ptr());
                        }
                    }
                    vehFieldType_t::VF_EFFECT_CLIENT => {
                        // (MP cgame only) take the string, get the index
                        #[cfg(not(feature = "jk2mp"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                G_EffectIndex(value.as_ptr());
                        }
                        #[cfg(all(feature = "jk2mp", feature = "qagame"))]
                        {
                            //*(int *)(b+vehWeaponFields[i].ofs) = G_EffectIndex( value );
                        }
                        #[cfg(all(feature = "jk2mp", feature = "cgame"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_FX_RegisterEffect(value.as_ptr());
                        }
                    }
                    vehFieldType_t::VF_SHADER => {
                        // (cgame only) take the string, call trap_R_RegisterShader
                        #[cfg(feature = "WE_ARE_IN_THE_UI")]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_R_RegisterShaderNoMip(value.as_ptr());
                        }
                        #[cfg(all(not(feature = "WE_ARE_IN_THE_UI"), feature = "cgame"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_R_RegisterShader(value.as_ptr());
                        }
                    }
                    vehFieldType_t::VF_SHADER_NOMIP => {
                        // (cgame only) take the string, call trap_R_RegisterShaderNoMip
                        #[cfg(not(feature = "qagame"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_R_RegisterShaderNoMip(value.as_ptr());
                        }
                    }
                    vehFieldType_t::VF_SOUND => {
                        // take the string, get the G_SoundIndex
                        #[cfg(feature = "qagame")]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                G_SoundIndex(value.as_ptr());
                        }
                        #[cfg(not(feature = "qagame"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_S_RegisterSound(value.as_ptr());
                        }
                    }
                    vehFieldType_t::VF_SOUND_CLIENT => {
                        // (MP cgame only) take the string, get the index
                        #[cfg(not(feature = "jk2mp"))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                G_SoundIndex(value.as_ptr());
                        }
                        #[cfg(all(feature = "jk2mp", feature = "qagame"))]
                        {
                            //*(int *)(b+vehWeaponFields[i].ofs) = G_SoundIndex( value );
                        }
                        #[cfg(all(feature = "jk2mp", not(feature = "qagame")))]
                        {
                            *(b.add(vehWeaponFields[i as usize].ofs as usize) as *mut c_int) =
                                trap_S_RegisterSound(value.as_ptr());
                        }
                    }
                }
                break;
            }
            i += 1;
        }
        if i == NUM_VWEAP_PARMS as c_int {
            return 0; // qfalse
        } else {
            return 1; // qtrue
        }
    }
}

pub fn VEH_LoadVehWeapon(vehWeaponName: *const c_char) -> c_int {
    //load up specified vehWeapon and save in array: g_vehWeaponInfo
    unsafe {
        let token: *const c_char;
        let parmName: [c_char; 128] = [0; 128]; //we'll assume that no parm name is longer than 128
        let value: *mut c_char;
        let p: *mut *const c_char;
        let vehWeapon: *mut vehWeaponInfo_t = core::ptr::null_mut();

        //BG_VehWeaponSetDefaults( &g_vehWeaponInfo[0] );//set the first vehicle to default data

        //try to parse data out
        let mut p_val = addr_of_mut!(VehWeaponParms)[0];

        #[cfg(feature = "jk2mp")]
        {
            COM_BeginParseSession(b"vehWeapons\0".as_ptr() as _);
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            COM_BeginParseSession(core::ptr::null());
        }

        *(&mut g_vehWeaponInfo[numVehicleWeapons as usize] as *mut vehWeaponInfo_t) = vehWeapon;
        // look for the right vehicle weapon
        while !p_val.is_null() {
            token = COM_ParseExt(&mut p_val, 1);
            if *token == 0 {
                return 0; // qfalse
            }

            if Q_stricmp(token, b"weapon_name_placeholder\0".as_ptr() as _) == 0 {
                break;
            }

            SkipBracedSection(&mut p_val);
        }
        if p_val.is_null() {
            return 0; // qfalse
        }

        token = COM_ParseExt(&mut p_val, 1);
        if *token == 0 {
            //barf
            return VEH_WEAPON_NONE;
        }

        if Q_stricmp(token, b"{\0".as_ptr() as _) != 0 {
            return VEH_WEAPON_NONE;
        }

        // parse the vehWeapon info block
        loop {
            SkipRestOfLine(&mut p_val);
            token = COM_ParseExt(&mut p_val, 1);
            if *token == 0 {
                Com_Printf(
                    b"ERROR: unexpected EOF while parsing Vehicle Weapon '%s'\n\0".as_ptr() as _,
                    vehWeaponName,
                );
                return VEH_WEAPON_NONE;
            }

            if Q_stricmp(token, b"}\0".as_ptr() as _) == 0 {
                break;
            }
            Q_strncpyz(parmName.as_ptr() as _, token, 128);
            value = COM_ParseExt(&mut p_val, 1) as *mut c_char;
            if value.is_null() || *value == 0 {
                Com_Printf(
                    b"ERROR: Vehicle Weapon token '%s' has no value!\n\0".as_ptr() as _,
                    parmName.as_ptr(),
                );
            } else {
                if BG_ParseVehWeaponParm(vehWeapon, parmName.as_ptr() as *mut c_char, value) == 0 {
                    Com_Printf(
                        b"ERROR: Unknown Vehicle Weapon key/value pair '%s','%s'!\n\0".as_ptr() as _,
                        parmName.as_ptr(),
                        value,
                    );
                }
            }
        }

        if (*vehWeapon).fHoming != 0.0 {
            //all lock-on weapons use these 2 sounds
            #[cfg(feature = "qagame")]
            {
                //Hmm, no need fo have server register this, is there?
                //G_SoundIndex( "sound/weapons/torpedo/tick.wav" );
                //G_SoundIndex( "sound/weapons/torpedo/lock.wav" );
            }
            #[cfg(feature = "cgame")]
            {
                trap_S_RegisterSound(b"sound/vehicles/weapons/common/tick.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/weapons/common/lock.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/common/lockalarm1.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/common/lockalarm2.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/common/lockalarm3.wav\0".as_ptr() as _);
            }
            #[cfg(all(not(feature = "qagame"), not(feature = "cgame")))]
            {
                trap_S_RegisterSound(b"sound/vehicles/weapons/common/tick.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/weapons/common/lock.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/common/lockalarm1.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/common/lockalarm2.wav\0".as_ptr() as _);
                trap_S_RegisterSound(b"sound/vehicles/common/lockalarm3.wav\0".as_ptr() as _);
            }
        }
        numVehicleWeapons += 1;
        return numVehicleWeapons - 1;
    }
}

pub fn VEH_VehWeaponIndexForName(vehWeaponName: *const c_char) -> c_int {
    unsafe {
        let mut vw: c_int;
        if vehWeaponName.is_null() || *vehWeaponName == 0 {
            Com_Printf(b"ERROR: Trying to read Vehicle Weapon with no name!\n\0".as_ptr() as _);
            return VEH_WEAPON_NONE;
        }
        vw = VEH_WEAPON_BASE;
        while vw < numVehicleWeapons {
            if !g_vehWeaponInfo[vw as usize].name.is_null()
                && Q_stricmp(&g_vehWeaponInfo[vw as usize].name, vehWeaponName) == 0
            {
                //already loaded this one
                return vw;
            }
            vw += 1;
        }
        //haven't loaded it yet
        if vw >= MAX_VEH_WEAPONS as c_int {
            //no more room!
            Com_Printf(
                b"ERROR: Too many Vehicle Weapons (max 16), aborting load on %s!\n\0".as_ptr() as _,
                vehWeaponName,
            );
            return VEH_WEAPON_NONE;
        }
        //we have room for another one, load it up and return the index
        //HMM... should we not even load the .vwp file until we want to?
        vw = VEH_LoadVehWeapon(vehWeaponName);
        if vw == VEH_WEAPON_NONE {
            Com_Printf(
                b"ERROR: Could not find Vehicle Weapon %s!\n\0".as_ptr() as _,
                vehWeaponName,
            );
        }
        return vw;
    }
}

// Stub - field offset constants that would normally be generated from offsetof macro
const VFOFS_name: c_int = 0;
const VWFOFS_name: c_int = 0;
const VWFOFS_bIsProjectile: c_int = 0;
const VWFOFS_bHasGravity: c_int = 0;
const VWFOFS_bIonWeapon: c_int = 0;
const VWFOFS_bSaberBlockable: c_int = 0;
const VWFOFS_iMuzzleFX: c_int = 0;
const VWFOFS_iModel: c_int = 0;
const VWFOFS_iShotFX: c_int = 0;
const VWFOFS_iImpactFX: c_int = 0;
const VWFOFS_iG2MarkShaderHandle: c_int = 0;
const VWFOFS_fG2MarkSize: c_int = 0;
const VWFOFS_iLoopSound: c_int = 0;
const VWFOFS_fSpeed: c_int = 0;
const VWFOFS_fHoming: c_int = 0;
const VWFOFS_fHomingFOV: c_int = 0;
const VWFOFS_iLockOnTime: c_int = 0;
const VWFOFS_iDamage: c_int = 0;
const VWFOFS_iSplashDamage: c_int = 0;
const VWFOFS_fSplashRadius: c_int = 0;
const VWFOFS_iAmmoPerShot: c_int = 0;
const VWFOFS_iHealth: c_int = 0;
const VWFOFS_fWidth: c_int = 0;
const VWFOFS_fHeight: c_int = 0;
const VWFOFS_iLifeTime: c_int = 0;
const VWFOFS_bExplodeOnExpire: c_int = 0;

// Stubs for external functions and types
extern "C" {
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, len: c_int);
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn atoi(s: *const c_char) -> c_int;
    pub fn atof(s: *const c_char) -> f64;
    pub fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn GetIDForString(table: *const c_void, string: *const c_char) -> c_int;
    pub fn SkipBracedSection(p: *mut *const c_char);
    pub fn SkipRestOfLine(p: *mut *const c_char);
    pub fn COM_ParseExt(p: *mut *const c_char, allowLineComments: c_int) -> *const c_char;
    pub fn COM_BeginParseSession(name: *const c_char);
    pub fn BG_Alloc(size: usize) -> *mut c_void;
    pub fn BG_TempAlloc(size: usize) -> *mut c_void;
    pub fn BG_TempFree(size: usize);
    pub fn G_NewString(string: *const c_char) -> *mut c_char;
    pub fn G_ModelIndex(name: *const c_char) -> c_int;
    pub fn G_SoundIndex(name: *const c_char) -> c_int;
    pub fn G_EffectIndex(name: *const c_char) -> c_int;
    pub fn G_SkinIndex(name: *const c_char) -> c_int;
    pub fn trap_R_RegisterModel(name: *const c_char) -> c_int;
    pub fn trap_R_RegisterSkin(name: *const c_char) -> c_int;
    pub fn trap_R_RegisterShader(name: *const c_char) -> c_int;
    pub fn trap_R_RegisterShaderNoMip(name: *const c_char) -> c_int;
    pub fn trap_FX_RegisterEffect(file: *const c_char) -> c_int;
    pub fn trap_S_RegisterSound(sample: *const c_char) -> c_int;
    pub fn va(fmt: *const c_char, ...) -> *mut c_char;
}

// Stub types
type vehicleType_t = c_int;
type stringID_table_t = c_void;

const NUM_VWEAP_PARMS: usize = 25;
const MAX_VEH_WEAPONS: usize = 16;
const MAX_VEHICLES: usize = 64;
const VEH_WEAPON_NONE: c_int = 0;
const VEH_WEAPON_BASE: c_int = 1;
const VEHICLE_NONE: c_int = 0;
const VEHICLE_BASE: c_int = 1;
const VH_NUM_VEHICLES: usize = 6;

extern "C" {
    pub static animTable: [stringID_table_t; MAX_ANIMATIONS + 1];
}

const MAX_ANIMATIONS: usize = 1024;

#[cfg(feature = "qagame")]
extern "C" {
    pub fn G_SetSharedVehicleFunctions(pVehInfo: *mut vehicleInfo_t);
}

static VehicleTable: [stringID_table_t; VH_NUM_VEHICLES + 1] = unsafe { core::mem::zeroed() };

// Setup the shared functions (one's that all vehicles would generally use).
pub fn BG_SetSharedVehicleFunctions(pVehInfo: *mut vehicleInfo_t) {
    #[cfg(feature = "qagame")]
    {
        //only do the whole thing if we're on game
        unsafe {
            G_SetSharedVehicleFunctions(pVehInfo);
        }
    }

    #[cfg(not(feature = "WE_ARE_IN_THE_UI"))]
    {
        unsafe {
            let type_ = (*pVehInfo).type_;
            match type_ {
                3 => {
                    // VH_SPEEDER
                    G_SetSpeederVehicleFunctions(pVehInfo);
                }
                4 => {
                    // VH_ANIMAL
                    G_SetAnimalVehicleFunctions(pVehInfo);
                }
                2 => {
                    // VH_FIGHTER
                    G_SetFighterVehicleFunctions(pVehInfo);
                }
                1 => {
                    // VH_WALKER
                    G_SetWalkerVehicleFunctions(pVehInfo);
                }
                _ => {}
            }
        }
    }
}

pub fn BG_VehicleSetDefaults(vehicle: *mut vehicleInfo_t) {
    unsafe {
        core::ptr::write_bytes(vehicle, 0, core::mem::size_of::<vehicleInfo_t>());
    }
    /*
    #if _JK2MP
        if (!vehicle->name)
        {
            vehicle->name = (char *)BG_Alloc(1024);
        }
        strcpy(vehicle->name, "default");
    #else
        vehicle->name = G_NewString( "default" );
    #endif

        //general data
        vehicle->type = VH_SPEEDER;				//what kind of vehicle
        //FIXME: no saber or weapons if numHands = 2, should switch to speeder weapon, no attack anim on player
        vehicle->numHands = 0;					//if 2 hands, no weapons, if 1 hand, can use 1-handed weapons, if 0 hands, can use 2-handed weapons
        vehicle->lookPitch = 0;				//How far you can look up and down off the forward of the vehicle
        vehicle->lookYaw = 5;					//How far you can look left and right off the forward of the vehicle
        vehicle->length = 0;					//how long it is - used for body length traces when turning/moving?
        vehicle->width = 0;						//how wide it is - used for body length traces when turning/moving?
        vehicle->height = 0;					//how tall it is - used for body length traces when turning/moving?
        VectorClear( vehicle->centerOfGravity );//offset from origin: {forward, right, up} as a modifier on that dimension (-1.0f is all the way back, 1.0f is all the way forward)

        //speed stats - note: these are DESIRED speed, not actual current speed/velocity
        vehicle->speedMax = VEH_DEFAULT_SPEED_MAX;	//top speed
        vehicle->turboSpeed = 0;					//turboBoost
        vehicle->speedMin = 0;						//if < 0, can go in reverse
        vehicle->speedIdle = 0;						//what speed it drifts to when no accel/decel input is given
        vehicle->accelIdle = 0;						//if speedIdle > 0, how quickly it goes up to that speed
        vehicle->acceleration = VEH_DEFAULT_ACCEL;	//when pressing on accelerator (1/2 this when going in reverse)
        vehicle->decelIdle = VEH_DEFAULT_DECEL;		//when giving no input, how quickly it desired speed drops to speedIdle
        vehicle->strafePerc = VEH_DEFAULT_STRAFE_PERC;//multiplier on current speed for strafing.  If 1.0f, you can strafe at the same speed as you're going forward, 0.5 is half, 0 is no strafing

        //handling stats
        vehicle->bankingSpeed = VEH_DEFAULT_BANKING_SPEED;	//how quickly it pitches and rolls (not under player control)
        vehicle->rollLimit = VEH_DEFAULT_ROLL_LIMIT;		//how far it can roll to either side
        vehicle->pitchLimit = VEH_DEFAULT_PITCH_LIMIT;		//how far it can pitch forward or backward
        vehicle->braking = VEH_DEFAULT_BRAKING;				//when pressing on decelerator (backwards)
        vehicle->turningSpeed = VEH_DEFAULT_TURNING_SPEED;	//how quickly you can turn
        vehicle->turnWhenStopped = qfalse;					//whether or not you can turn when not moving
        vehicle->traction = VEH_DEFAULT_TRACTION;			//how much your command input affects velocity
        vehicle->friction = VEH_DEFAULT_FRICTION;			//how much velocity is cut on its own
        vehicle->maxSlope = VEH_DEFAULT_MAX_SLOPE;			//the max slope that it can go up with control

        //durability stats
        vehicle->mass = VEH_DEFAULT_MASS;			//for momentum and impact force (player mass is 10)
        vehicle->armor = VEH_DEFAULT_MAX_ARMOR;		//total points of damage it can take
        vehicle->toughness = VEH_DEFAULT_TOUGHNESS;	//modifies incoming damage, 1.0 is normal, 0.5 is half, etc.  Simulates being made of tougher materials/construction
        vehicle->malfunctionArmorLevel = 0;			//when armor drops to or below this point, start malfunctioning

        //visuals & sounds
        //vehicle->model = "models/map_objects/ships/swoop.md3";	//what model to use - if make it an NPC's primary model, don't need this?
        if (!vehicle->model)
        {
            vehicle->model = (char *)BG_Alloc(1024);
        }
        strcpy(vehicle->model, "models/map_objects/ships/swoop.md3");

        vehicle->modelIndex = 0;							//set internally, not until this vehicle is spawned into the level
        vehicle->skin = NULL;								//what skin to use - if make it an NPC's primary model, don't need this?
        vehicle->riderAnim = BOTH_GUNSIT1;					//what animation the rider uses

        vehicle->soundOn = NULL;							//sound to play when get on it
        vehicle->soundLoop = NULL;							//sound to loop while riding it
        vehicle->soundOff = NULL;							//sound to play when get off
        vehicle->exhaustFX = NULL;							//exhaust effect, played from "*exhaust" bolt(s)
        vehicle->trailFX = NULL;							//trail effect, played from "*trail" bolt(s)
        vehicle->impactFX = NULL;							//explosion effect, for when it blows up (should have the sound built into explosion effect)
        vehicle->explodeFX = NULL;							//explosion effect, for when it blows up (should have the sound built into explosion effect)
        vehicle->wakeFX = NULL;								//effect itmakes when going across water

        //other misc stats
        vehicle->gravity = VEH_DEFAULT_GRAVITY;				//normal is 800
        vehicle->hoverHeight = 0;//VEH_DEFAULT_HOVER_HEIGHT;	//if 0, it's a ground vehicle
        vehicle->hoverStrength = 0;//VEH_DEFAULT_HOVER_STRENGTH;//how hard it pushes off ground when less than hover height... causes "bounce", like shocks
        vehicle->waterProof = qtrue;						//can drive underwater if it has to
        vehicle->bouyancy = 1.0f;							//when in water, how high it floats (1 is neutral bouyancy)
        vehicle->fuelMax = 1000;							//how much fuel it can hold (capacity)
        vehicle->fuelRate = 1;								//how quickly is uses up fuel
        vehicle->visibility = VEH_DEFAULT_VISIBILITY;		//radius for sight alerts
        vehicle->loudness = VEH_DEFAULT_LOUDNESS;			//radius for sound alerts
        vehicle->explosionRadius = VEH_DEFAULT_EXP_RAD;
        vehicle->explosionDamage = VEH_DEFAULT_EXP_DMG;
        vehicle->maxPassengers = 0;

        //new stuff
        vehicle->hideRider = qfalse;						// rider (and passengers?) should not be drawn
        vehicle->killRiderOnDeath = qfalse;					//if rider is on vehicle when it dies, they should die
        vehicle->flammable = qfalse;						//whether or not the vehicle should catch on fire before it explodes
        vehicle->explosionDelay = 0;						//how long the vehicle should be on fire/dying before it explodes
        //camera stuff
        vehicle->cameraOverride = qfalse;					//whether or not to use all of the following 3rd person camera override values
        vehicle->cameraRange = 0.0f;						//how far back the camera should be - normal is 80
        vehicle->cameraVertOffset = 0.0f;					//how high over the vehicle origin the camera should be - normal is 16
        vehicle->cameraHorzOffset = 0.0f;					//how far to left/right (negative/positive) of of the vehicle origin the camera should be - normal is 0
        vehicle->cameraPitchOffset = 0.0f;					//a modifier on the camera's pitch (up/down angle) to the vehicle - normal is 0
        vehicle->cameraFOV = 0.0f;							//third person camera FOV, default is 80
        vehicle->cameraAlpha = qfalse;						//fade out the vehicle if it's in the way of the crosshair
    */
}

pub fn BG_VehicleClampData(vehicle: *mut vehicleInfo_t) {
    //sanity check and clamp the vehicle's data
    unsafe {
        let mut i: c_int;

        i = 0;
        while i < 3 {
            if (*vehicle).centerOfGravity[i as usize] > 1.0 {
                (*vehicle).centerOfGravity[i as usize] = 1.0;
            } else if (*vehicle).centerOfGravity[i as usize] < -1.0 {
                (*vehicle).centerOfGravity[i as usize] = -1.0;
            }
            i += 1;
        }

        // Validate passenger max.
        if (*vehicle).maxPassengers > 8 {
            // VEH_MAX_PASSENGERS
            (*vehicle).maxPassengers = 8;
        } else if (*vehicle).maxPassengers < 0 {
            (*vehicle).maxPassengers = 0;
        }
    }
}

pub fn BG_ParseVehicleParm(
    vehicle: *mut vehicleInfo_t,
    parmName: *mut c_char,
    pValue: *mut c_char,
) -> c_int {
    // Stub function - implemented similarly to BG_ParseVehWeaponParm but for vehicles
    0 // qfalse
}

pub fn VEH_LoadVehicle(vehicleName: *const c_char) -> c_int {
    //load up specified vehicle and save in array: g_vehicleInfo
    0 // Stub
}

pub fn VEH_VehicleIndexForName(vehicleName: *const c_char) -> c_int {
    unsafe {
        let mut v: c_int;
        if vehicleName.is_null() || *vehicleName == 0 {
            Com_Printf(b"ERROR: Trying to read Vehicle with no name!\n\0".as_ptr() as _);
            return VEHICLE_NONE;
        }
        v = VEHICLE_BASE;
        while v < numVehicles {
            if !g_vehicleInfo[v as usize].name.is_null()
                && Q_stricmp(
                    &g_vehicleInfo[v as usize].name,
                    vehicleName,
                ) == 0
            {
                //already loaded this one
                return v;
            }
            v += 1;
        }
        //haven't loaded it yet
        if v >= MAX_VEHICLES as c_int {
            //no more room!
            Com_Printf(
                b"ERROR: Too many Vehicles (max 64), aborting load on %s!\n\0".as_ptr() as _,
                vehicleName,
            );
            return VEHICLE_NONE;
        }
        //we have room for another one, load it up and return the index
        //HMM... should we not even load the .veh file until we want to?
        v = VEH_LoadVehicle(vehicleName);
        if v == VEHICLE_NONE {
            Com_Printf(
                b"ERROR: Could not find Vehicle %s!\n\0".as_ptr() as _,
                vehicleName,
            );
        }
        return v;
    }
}

pub fn BG_VehWeaponLoadParms() {
    unsafe {
        let mut len: c_int;
        let mut totallen: c_int;
        let mut vehExtFNLen: c_int;
        let mut mainBlockLen: c_int;
        let mut fileCnt: c_int;
        let mut i: c_int;
        let mut holdChar: *mut c_char;
        let mut marker: *mut c_char;
        let vehWeaponExtensionListBuf: [c_char; 2048] = [0; 2048]; //	The list of file names read in
        let mut f: c_int;
        let mut tempReadBuffer: *mut c_char;

        len = 0;

        //remember where to store the next one
        totallen = mainBlockLen = len;
        marker = addr_of_mut!(VehWeaponParms)[totallen as usize];
        *marker = 0;

        //now load in the extra .veh extensions
        #[cfg(feature = "jk2mp")]
        {
            fileCnt = trap_FS_GetFileList(
                b"ext_data/vehicles/weapons\0".as_ptr() as _,
                b".vwp\0".as_ptr() as _,
                vehWeaponExtensionListBuf.as_ptr() as *mut c_char,
                core::mem::size_of_val(&vehWeaponExtensionListBuf) as c_int,
            );
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            fileCnt = gi_FS_GetFileList(
                b"ext_data/vehicles/weapons\0".as_ptr() as _,
                b".vwp\0".as_ptr() as _,
                vehWeaponExtensionListBuf.as_ptr() as *mut c_char,
                core::mem::size_of_val(&vehWeaponExtensionListBuf) as c_int,
            );
        }

        holdChar = vehWeaponExtensionListBuf.as_ptr() as *mut c_char;

        #[cfg(feature = "jk2mp")]
        {
            tempReadBuffer = BG_TempAlloc(MAX_VEH_WEAPON_DATA_SIZE) as *mut c_char;
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            tempReadBuffer = gi_Malloc(MAX_VEH_WEAPON_DATA_SIZE as c_int, 0, 1) as *mut c_char;
        }

        // NOTE: Not use TempAlloc anymore...
        //Make ABSOLUTELY CERTAIN that BG_Alloc/etc. is not used before
        //the subsequent BG_TempFree or the pool will be screwed.

        i = 0;
        while i < fileCnt {
            vehExtFNLen = strlen(holdChar) as c_int;

            //		Com_Printf( "Parsing %s\n", holdChar );

            #[cfg(feature = "jk2mp")]
            {
                len = trap_FS_FOpenFile(
                    va(b"ext_data/vehicles/weapons/%s\0".as_ptr() as _, holdChar),
                    &mut f,
                    0, // FS_READ
                );
            }
            #[cfg(not(feature = "jk2mp"))]
            {
                //		len = gi.FS_ReadFile( va( "ext_data/vehicles/weapons/%s", holdChar), (void **) &buffer );
                len = gi_FS_FOpenFile(
                    va(b"ext_data/vehicles/weapons/%s\0".as_ptr() as _, holdChar),
                    &mut f,
                    0, // FS_READ
                );
            }

            if len == -1 {
                Com_Printf(b"error reading file\n\0".as_ptr() as _);
            } else {
                #[cfg(feature = "jk2mp")]
                {
                    trap_FS_Read(tempReadBuffer as *mut c_void, len, f);
                    *tempReadBuffer.add(len as usize) = 0;
                }
                #[cfg(not(feature = "jk2mp"))]
                {
                    gi_FS_Read(tempReadBuffer as *mut c_void, len, f);
                    *tempReadBuffer.add(len as usize) = 0;
                }

                // Don't let it end on a } because that should be a stand-alone token.
                if totallen != 0 && *marker.offset(-1) == b'}' as c_char {
                    strcat(marker, b" \0".as_ptr() as _);
                    totallen += 1;
                    marker = marker.offset(1);
                }

                if totallen + len >= MAX_VEH_WEAPON_DATA_SIZE as c_int {
                    Com_Error(
                        2, // ERR_DROP
                        b"Vehicle Weapon extensions (*.vwp) are too large\0".as_ptr() as _,
                    );
                }
                strcat(marker, tempReadBuffer);
                #[cfg(feature = "jk2mp")]
                {
                    trap_FS_FCloseFile(f);
                }
                #[cfg(not(feature = "jk2mp"))]
                {
                    gi_FS_FCloseFile(f);
                }

                totallen += len;
                marker = addr_of_mut!(VehWeaponParms)[totallen as usize];
            }
            i += 1;
            holdChar = holdChar.offset((vehExtFNLen + 1) as isize);
        }

        #[cfg(feature = "jk2mp")]
        {
            BG_TempFree(MAX_VEH_WEAPON_DATA_SIZE as c_int);
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            gi_Free(tempReadBuffer as *mut c_void);
            tempReadBuffer = core::ptr::null_mut();
        }
    }
}

pub fn BG_VehicleLoadParms() {
    //HMM... only do this if there's a vehicle on the level?
    unsafe {
        let mut len: c_int;
        let mut totallen: c_int;
        let mut vehExtFNLen: c_int;
        let mut mainBlockLen: c_int;
        let mut fileCnt: c_int;
        let mut i: c_int;
        //	const char	*filename = "ext_data/vehicles.dat";
        let mut holdChar: *mut c_char;
        let mut marker: *mut c_char;
        let vehExtensionListBuf: [c_char; 2048] = [0; 2048]; //	The list of file names read in
        let mut f: c_int;
        let mut tempReadBuffer: *mut c_char;

        len = 0;

        //remember where to store the next one
        totallen = mainBlockLen = len;
        marker = addr_of_mut!(VehicleParms)[totallen as usize];
        *marker = 0;

        //now load in the extra .veh extensions
        #[cfg(feature = "jk2mp")]
        {
            fileCnt = trap_FS_GetFileList(
                b"ext_data/vehicles\0".as_ptr() as _,
                b".veh\0".as_ptr() as _,
                vehExtensionListBuf.as_ptr() as *mut c_char,
                core::mem::size_of_val(&vehExtensionListBuf) as c_int,
            );
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            fileCnt = gi_FS_GetFileList(
                b"ext_data/vehicles\0".as_ptr() as _,
                b".veh\0".as_ptr() as _,
                vehExtensionListBuf.as_ptr() as *mut c_char,
                core::mem::size_of_val(&vehExtensionListBuf) as c_int,
            );
        }

        holdChar = vehExtensionListBuf.as_ptr() as *mut c_char;

        #[cfg(feature = "jk2mp")]
        {
            tempReadBuffer = BG_TempAlloc(MAX_VEHICLE_DATA_SIZE) as *mut c_char;
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            tempReadBuffer = gi_Malloc(MAX_VEHICLE_DATA_SIZE as c_int, 0, 1) as *mut c_char;
        }

        // NOTE: Not use TempAlloc anymore...
        //Make ABSOLUTELY CERTAIN that BG_Alloc/etc. is not used before
        //the subsequent BG_TempFree or the pool will be screwed.

        i = 0;
        while i < fileCnt {
            vehExtFNLen = strlen(holdChar) as c_int;

            //		Com_Printf( "Parsing %s\n", holdChar );

            #[cfg(feature = "jk2mp")]
            {
                len = trap_FS_FOpenFile(
                    va(b"ext_data/vehicles/%s\0".as_ptr() as _, holdChar),
                    &mut f,
                    0, // FS_READ
                );
            }
            #[cfg(not(feature = "jk2mp"))]
            {
                //		len = gi.FS_ReadFile( va( "ext_data/vehicles/%s", holdChar), (void **) &buffer );
                len = gi_FS_FOpenFile(
                    va(b"ext_data/vehicles/%s\0".as_ptr() as _, holdChar),
                    &mut f,
                    0, // FS_READ
                );
            }

            if len == -1 {
                Com_Printf(b"error reading file\n\0".as_ptr() as _);
            } else {
                #[cfg(feature = "jk2mp")]
                {
                    trap_FS_Read(tempReadBuffer as *mut c_void, len, f);
                    *tempReadBuffer.add(len as usize) = 0;
                }
                #[cfg(not(feature = "jk2mp"))]
                {
                    gi_FS_Read(tempReadBuffer as *mut c_void, len, f);
                    *tempReadBuffer.add(len as usize) = 0;
                }

                // Don't let it end on a } because that should be a stand-alone token.
                if totallen != 0 && *marker.offset(-1) == b'}' as c_char {
                    strcat(marker, b" \0".as_ptr() as _);
                    totallen += 1;
                    marker = marker.offset(1);
                }

                if totallen + len >= MAX_VEHICLE_DATA_SIZE as c_int {
                    Com_Error(
                        2, // ERR_DROP
                        b"Vehicle extensions (*.veh) are too large\0".as_ptr() as _,
                    );
                }
                strcat(marker, tempReadBuffer);
                #[cfg(feature = "jk2mp")]
                {
                    trap_FS_FCloseFile(f);
                }
                #[cfg(not(feature = "jk2mp"))]
                {
                    gi_FS_FCloseFile(f);
                }

                totallen += len;
                marker = addr_of_mut!(VehicleParms)[totallen as usize];
            }
            i += 1;
            holdChar = holdChar.offset((vehExtFNLen + 1) as isize);
        }

        #[cfg(feature = "jk2mp")]
        {
            BG_TempFree(MAX_VEHICLE_DATA_SIZE as c_int);
        }
        #[cfg(not(feature = "jk2mp"))]
        {
            gi_Free(tempReadBuffer as *mut c_void);
            tempReadBuffer = core::ptr::null_mut();
        }

        numVehicles = 1; //first one is null/default
        //set the first vehicle to default data
        BG_VehicleSetDefaults(&mut g_vehicleInfo[VEHICLE_BASE as usize]);
        //sanity check and clamp the vehicle's data
        BG_VehicleClampData(&mut g_vehicleInfo[VEHICLE_BASE as usize]);
        // Setup the shared function pointers.
        BG_SetSharedVehicleFunctions(&mut g_vehicleInfo[VEHICLE_BASE as usize]);

        //Load the Vehicle Weapons data, too
        BG_VehWeaponLoadParms();
    }
}

pub fn BG_VehicleGetIndex(vehicleName: *const c_char) -> c_int {
    VEH_VehicleIndexForName(vehicleName)
}

//We get the vehicle name passed in as modelname
//with a $ in front of it.
//we are expected to then get the model for the
//vehicle and stomp over modelname with it.
pub fn BG_GetVehicleModelName(modelname: *mut c_char) {
    unsafe {
        let vehName = modelname.offset(1);
        let vIndex = BG_VehicleGetIndex(vehName);
        assert!(*modelname == b'$' as c_char);

        if vIndex == VEHICLE_NONE {
            Com_Error(
                2, // ERR_DROP
                b"BG_GetVehicleModelName:  couldn't find vehicle %s\0".as_ptr() as _,
                vehName,
            );
        }

        strcpy(modelname, g_vehicleInfo[vIndex as usize].model);
    }
}

pub fn BG_GetVehicleSkinName(skinname: *mut c_char) {
    unsafe {
        let vehName = skinname.offset(1);
        let vIndex = BG_VehicleGetIndex(vehName);
        assert!(*skinname == b'$' as c_char);

        if vIndex == VEHICLE_NONE {
            Com_Error(
                2, // ERR_DROP
                b"BG_GetVehicleSkinName:  couldn't find vehicle %s\0".as_ptr() as _,
                vehName,
            );
        }

        if g_vehicleInfo[vIndex as usize].skin.is_null()
            || *g_vehicleInfo[vIndex as usize].skin == 0
        {
            *skinname = 0;
        } else {
            strcpy(skinname, g_vehicleInfo[vIndex as usize].skin);
        }
    }
}

#[cfg(all(feature = "jk2mp", not(feature = "WE_ARE_IN_THE_UI")))]
extern "C" {
    //so cgame can assign the function pointer for the vehicle attachment without having to
    //bother with all the other funcs that don't really exist cgame-side.
    pub fn BG_GetTime() -> c_int;
    pub fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char)
        -> c_int;
    pub fn trap_G2API_GetBoltMatrix(
        ghoul2: *mut c_void,
        modelIndex: c_int,
        boltIndex: c_int,
        matrix: *mut c_void,
        angles: *const f32,
        position: *const f32,
        frameNum: c_int,
        modelList: *mut c_void,
        scale: *const f32,
    ) -> c_int;
    pub fn BG_GiveMeVectorFromMatrix(matrix: *mut c_void, mode: c_int, vec: *mut f32);
}

#[cfg(all(feature = "jk2mp", not(feature = "WE_ARE_IN_THE_UI")))]
#[repr(C)]
pub struct mdxaBone_t {
    // Stub - actual definition should be in mdx format headers
    _data: [u8; 128], // Placeholder size
}

#[cfg(all(feature = "jk2mp", not(feature = "WE_ARE_IN_THE_UI")))]
#[repr(C)]
pub struct bgEntity_t {
    // Stub - actual definition should be in bg_public.h
    pub playerState: *mut c_void,
    pub ghoul2: *mut c_void,
    pub modelScale: [f32; 3],
}

#[cfg(all(feature = "jk2mp", not(feature = "WE_ARE_IN_THE_UI")))]
#[repr(C)]
pub struct Vehicle_t {
    // Stub - actual definition should be in bg_vehicles.h
    pub m_pPilot: *mut bgEntity_t,
    pub m_pParentEntity: *mut bgEntity_t,
}

#[cfg(all(feature = "jk2mp", not(feature = "WE_ARE_IN_THE_UI")))]
pub fn AttachRidersGeneric(pVeh: *mut Vehicle_t) {
    unsafe {
        // If we have a pilot, attach him to the driver tag.
        if !(*pVeh).m_pPilot.is_null() {
            let mut boltMatrix: mdxaBone_t = core::mem::zeroed();
            let mut yawOnlyAngles: [f32; 3] = [0.0; 3];
            let parent = (*pVeh).m_pParentEntity;
            let pilot = (*pVeh).m_pPilot;
            let crotchBolt = trap_G2API_AddBolt((*parent).ghoul2, 0, b"*driver\0".as_ptr() as _);

            assert!(!(*parent).playerState.is_null());

            yawOnlyAngles[0] = 0.0;
            yawOnlyAngles[1] = *((*parent).playerState as *const f32).add(1); // YAW = 1 (assuming playerState layout)
            yawOnlyAngles[2] = 0.0;

            // Get the driver tag.
            trap_G2API_GetBoltMatrix(
                (*parent).ghoul2,
                0,
                crotchBolt,
                &mut boltMatrix as *mut mdxaBone_t as *mut c_void,
                yawOnlyAngles.as_ptr(),
                *((*parent).playerState as *const f32) as *const f32, // origin
                BG_GetTime(),
                core::ptr::null_mut(),
                (*parent).modelScale.as_ptr(),
            );
            BG_GiveMeVectorFromMatrix(
                &mut boltMatrix as *mut mdxaBone_t as *mut c_void,
                0, // ORIGIN
                *((*pilot).playerState as *mut f32),
            );
        }
    }
}

// Stub functions for gi. namespace items (single-player)
#[cfg(not(feature = "jk2mp"))]
extern "C" {
    pub fn gi_FS_GetFileList(
        path: *const c_char,
        extension: *const c_char,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int;
    pub fn gi_FS_FOpenFile(filename: *const c_char, f: *mut c_int, mode: c_int) -> c_int;
    pub fn gi_FS_Read(buffer: *mut c_void, len: c_int, f: c_int);
    pub fn gi_FS_FCloseFile(f: c_int);
    pub fn gi_Malloc(size: c_int, tag: c_int, bZeromem: c_int) -> *mut c_void;
    pub fn gi_Free(buffer: *mut c_void);
}

#[cfg(not(feature = "jk2mp"))]
use {gi_FS_GetFileList, gi_FS_FOpenFile, gi_FS_Read, gi_FS_FCloseFile, gi_Malloc, gi_Free};
