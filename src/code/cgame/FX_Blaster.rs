//! Mechanical port of `code/cgame/FX_Blaster.cpp`.

// Blaster Weapon

// this line must stay at top so the whole PCH thing works...

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use core::ffi::c_int;

use crate::code::game::q_shared_h::vec3_t;

// ============================================================================
// External functions
// ============================================================================

extern "C" {
    /// Compute forward, right, and up vectors from angles.
    pub fn AngleVectors(
        angles: *const vec3_t,
        forward: *mut vec3_t,
        right: *mut vec3_t,
        up: *mut vec3_t,
    );

    /// Normalize the input vector and stores result; returns magnitude of original vector.
    pub fn VectorNormalize2(v: *const vec3_t, out: *mut vec3_t) -> f32;

    /// Scale input vector by scalar and stores result.
    pub fn VectorScale(i: *const vec3_t, scale: f32, o: *mut vec3_t);

    /// Add a Ghoul2 mark (decal) to an entity.
    /// Signature: void CG_AddGhoul2Mark(int type, float size, vec3_t hitloc, vec3_t hitdirection,
    ///     int entnum, vec3_t entposition, float entangle, CGhoul2Info_v &ghoul2, vec3_t modelScale,
    ///     int lifeTime = 0, int firstModel = 0, vec3_t uaxis = 0);
    pub fn CG_AddGhoul2Mark(
        r#type: c_int,
        size: f32,
        hitloc: *const vec3_t,
        hitdirection: *const vec3_t,
        entnum: c_int,
        entposition: *const vec3_t,
        entangle: f32,
        ghoul2: *mut CGhoul2Info_v,
        modelScale: *const vec3_t,
        lifeTime: c_int,
        firstModel: c_int,
        uaxis: *const vec3_t,
    );

    /// Return a random float between min and max (inclusive of min, exclusive of max).
    pub fn Q_flrand(min: f32, max: f32) -> f32;

    /// Return a random integer between min and max (inclusive).
    pub fn Q_irand(min: c_int, max: c_int) -> c_int;

    /// Global effect scheduler instance (theFxScheduler C++ object).
    /// This is a wrapper to CFxScheduler::PlayEffect for string-based effect names.
    pub fn FX_PlayEffect(name: *const u8, origin: *const vec3_t, normal: *const vec3_t);
}

// ============================================================================
// Stubs for cgame types
// ============================================================================

/// Stub for trajectory_t: a trajectory with delta vector.
#[repr(C)]
pub struct trajectory_t {
    pub trType: c_int,
    pub trTime: c_int,
    pub trDuration: c_int,
    pub trBase: vec3_t,
    pub trDelta: vec3_t,
}

/// Stub for entityState_t: minimal definition for necessary fields.
#[repr(C)]
pub struct entityState_t {
    pub number: c_int,
    pub eType: c_int,
    pub eFlags: c_int,
    pub pos: trajectory_t,
    pub apos: trajectory_t,
    pub _pad0: [u8; 32], // Padding to angles
    pub angles: vec3_t,
    pub _pad1: [u8; 80], // Padding to modelScale
    pub modelScale: vec3_t,
    // ... rest of fields omitted
}

/// Stub for gentity_t: game entity.
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub _pad0: [u8; 512], // Padding to client
    pub client: *mut gclient_t,
    pub _pad1: [u8; 16], // Padding to owner
    pub owner: *mut gentity_t,
    pub _pad2: [u8; 64], // Padding to ghoul2
    pub ghoul2: CGhoul2Info_v,
    // ... rest of fields omitted
}

/// Stub for gclient_t: client-specific game entity data.
#[repr(C)]
pub struct gclient_t {
    pub ps: playerState_t,
    pub _pad0: [u8; 256], // Padding to renderInfo
    pub renderInfo: clientRenderInfo_t,
    // ... rest of fields omitted
}

/// Stub for playerState_t: player state data.
#[repr(C)]
pub struct playerState_t {
    pub origin: vec3_t,
    // ... rest of fields omitted
}

/// Stub for clientRenderInfo_t: client rendering info.
#[repr(C)]
pub struct clientRenderInfo_t {
    pub legsYaw: f32,
    // ... rest of fields omitted
}

/// Stub for CGhoul2Info_v: Ghoul2 model info vector.
#[repr(C)]
pub struct CGhoul2Info_v {
    pub _data: [u8; 576], // Placeholder - opaque to us
}

impl CGhoul2Info_v {
    /// Get the size of the ghoul2 info vector.
    pub fn size(&self) -> usize {
        // This is a stub implementation that calls into C++
        // The actual ghoul2 info is opaque in the port
        unsafe { CGhoul2Info_v_size(core::ptr::addr_of!(*self)) }
    }
}

extern "C" {
    /// Get the size of a Ghoul2 info vector (C++ wrapper).
    pub fn CGhoul2Info_v_size(ghoul2: *const CGhoul2Info_v) -> usize;
}

/// Stub for centity_t: client entity with position and rendering state.
#[repr(C)]
pub struct centity_t {
    pub currentState: entityState_t,
    pub _pad0: [u8; 8],
    pub lerpOrigin: vec3_t,
    pub _pad1: [u8; 44],
    pub gent: *mut gentity_t,
}

/// Stub for weaponInfo_s: weapon metadata.
#[repr(C)]
pub struct weaponInfo_s {
    // Fields not needed for FX_Blaster
}

// ============================================================================
// Global client game state
// ============================================================================

/// Stub for cg_t struct to allow access to cg.time.
#[repr(C)]
pub struct cg_t {
    pub _pad: [u8; 316],
    pub time: c_int,
}

extern "C" {
    /// Global client game state, declared in cg_main.cpp.
    pub static cg: cg_t;
}

// ============================================================================
// Global cgs state (cgame static)
// ============================================================================

/// Stub for cgEffects_t: effect handle table.
#[repr(C)]
pub struct cgEffects_t {
    pub _pad0: [u8; 344], // Padding to blasterShotEffect (after bowcaster effects)
    pub _bowcasterShotEffect: c_int,
    pub _bowcasterBounceEffect: c_int,
    pub _bowcasterImpactEffect: c_int,
    pub _pad1: [u8; 4], // Padding to blasterShotEffect
    pub blasterShotEffect: c_int,
    pub blasterWallImpactEffect: c_int,
    pub blasterFleshImpactEffect: c_int,
    // ... rest of fields omitted
}

/// Stub for cgMedia_t: media resources.
#[repr(C)]
pub struct cgMedia_t {
    pub bdecal_burnmark1: c_int,
    pub _pad0: [u8; 5340], // Placeholder for rest of fields
    // ... rest of fields omitted
}

/// Stub for cgs_t struct (client game static state).
#[repr(C)]
pub struct cgs_t {
    pub _pad0: [u8; 9424], // Padding to media
    pub media: cgMedia_t,
    pub effects: cgEffects_t,
}

extern "C" {
    /// Global client game static state, declared in cg_main.cpp.
    pub static cgs: cgs_t;
}

// ============================================================================
// External C++ object and member function for theFxScheduler
// ============================================================================

/// Opaque struct for CFxScheduler C++ class.
#[repr(C)]
pub struct CFxScheduler {
    _unused: [u8; 0],
}

extern "C" {
    /// Global instance of the effect scheduler (defined in FxScheduler.cpp).
    /// This is a C++ global object; accessing its methods requires proper C++ name mangling
    /// or linking against a C wrapper function.
    pub static theFxScheduler: CFxScheduler;
}

extern "C" {
    /// Play an effect by ID at the given origin with a forward direction vector.
    /// This corresponds to CFxScheduler::PlayEffect(int id, vec3_t org, vec3_t fwd, bool isPortal=false).
    /// Uses the GCC/Clang C++ name mangling for the method signature.
    /// On MSVC, the link_name would differ and may need adjustment.
    #[link_name = "_ZN11CFxScheduler9PlayEffectEiA3_fS1_b"]
    pub fn CFxScheduler_PlayEffect_v1(
        this: *const CFxScheduler,
        id: c_int,
        origin: *const vec3_t,
        forward: *const vec3_t,
        isPortal: c_int,
    );
}

// ============================================================================
// Constants
// ============================================================================

const EF_USE_ANGLEDELTA: c_int = 0x00002000;

/*
-------------------------
FX_BlasterProjectileThink
-------------------------
*/

/// Project the Blaster projectile effect along its trajectory.
///
/// # Safety
/// `cent` and `weapon` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_BlasterProjectileThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    let mut forward: vec3_t = [0.0; 3];

    if (*cent).currentState.eFlags & EF_USE_ANGLEDELTA != 0 {
        AngleVectors(
            core::ptr::addr_of!((*cent).currentState.angles),
            core::ptr::addr_of_mut!(forward),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    } else {
        if VectorNormalize2(
            core::ptr::addr_of!((*(*cent).gent).s.pos.trDelta),
            core::ptr::addr_of_mut!(forward),
        ) == 0.0f32
        {
            if VectorNormalize2(
                core::ptr::addr_of!((*cent).currentState.pos.trDelta),
                core::ptr::addr_of_mut!(forward),
            ) == 0.0f32
            {
                forward[2] = 1.0f32;
            }
        }
    }

    // hack the scale of the forward vector if we were just fired or bounced...this will shorten up the tail for a split second so tails don't clip so harshly
    let mut dif = cg.time - (*(*cent).gent).s.pos.trTime;

    if dif < 75 {
        if dif < 0 {
            dif = 0;
        }

        let scale = (dif as f32 / 75.0f32) * 0.95f32 + 0.05f32;

        VectorScale(
            core::ptr::addr_of!(forward),
            scale,
            core::ptr::addr_of_mut!(forward),
        );
    }

    if !(*cent).gent.is_null()
        && !(*(*cent).gent).owner.is_null()
        && (*(*(*cent).gent).owner).s.number > 0
    {
        FX_PlayEffect(
            b"blaster/NPCshot\0".as_ptr(),
            core::ptr::addr_of!((*cent).lerpOrigin),
            core::ptr::addr_of!(forward),
        );
    } else {
        CFxScheduler_PlayEffect_v1(
            core::ptr::addr_of!(theFxScheduler),
            cgs.effects.blasterShotEffect,
            core::ptr::addr_of!((*cent).lerpOrigin),
            core::ptr::addr_of!(forward),
            0, // isPortal = false
        );
    }
}

/*
-------------------------
FX_BlasterAltFireThink
-------------------------
*/

/// Play the Blaster alt fire effect (delegates to projectile think).
///
/// # Safety
/// `cent` and `weapon` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_BlasterAltFireThink(
    cent: *const centity_t,
    weapon: *const weaponInfo_s,
) {
    FX_BlasterProjectileThink(cent, weapon);
}

/*
-------------------------
FX_BlasterWeaponHitWall
-------------------------
*/

/// Play the Blaster wall impact effect.
///
/// # Safety
/// `origin` and `normal` must be valid vec3_t pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_BlasterWeaponHitWall(origin: *const vec3_t, normal: *const vec3_t) {
    CFxScheduler_PlayEffect_v1(
        core::ptr::addr_of!(theFxScheduler),
        cgs.effects.blasterWallImpactEffect,
        origin,
        normal,
        0, // isPortal = false
    );
}

/*
-------------------------
FX_BlasterWeaponHitPlayer
-------------------------
*/

/// Play the Blaster player impact effect, with optional Ghoul2 mark.
///
/// # Safety
/// `hit`, `origin`, and `normal` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn FX_BlasterWeaponHitPlayer(
    hit: *mut gentity_t,
    origin: *const vec3_t,
    normal: *const vec3_t,
    humanoid: c_int,
) {
    //temporary? just testing out the damage skin stuff -rww
    if !hit.is_null() && !(*hit).client.is_null() && (*hit).ghoul2.size() > 0 {
        CG_AddGhoul2Mark(
            cgs.media.bdecal_burnmark1,
            Q_flrand(3.5, 4.0),
            origin,
            normal,
            (*hit).s.number,
            core::ptr::addr_of!((*(*hit).client).ps.origin),
            (*(*hit).client).renderInfo.legsYaw,
            core::ptr::addr_of_mut!((*hit).ghoul2),
            core::ptr::addr_of!((*hit).s.modelScale),
            Q_irand(10000, 13000),
            0,
            core::ptr::null(),
        );
    }

    CFxScheduler_PlayEffect_v1(
        core::ptr::addr_of!(theFxScheduler),
        cgs.effects.blasterFleshImpactEffect,
        origin,
        normal,
        0, // isPortal = false
    );
}
