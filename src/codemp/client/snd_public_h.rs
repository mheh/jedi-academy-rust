//! Mechanical port of `codemp/client/snd_public.h`.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::{c_char, c_float, c_int, c_uchar};

use crate::codemp::game::q_shared_h::{byte, qboolean, vec3_t};

// Unported shared typedef dependency from the original sound headers.
pub type sfxHandle_t = c_int;

unsafe extern "C" {
    pub fn S_Init();
    pub fn S_Shutdown();

    // if origin is NULL, the sound will be dynamically sourced from the entity
    pub fn S_AddAmbientLoopingSound(
        origin: *const vec3_t,
        volume: c_uchar,
        sfxHandle: sfxHandle_t,
    );
    pub fn S_StartAmbientSound(
        origin: *const vec3_t,
        entityNum: c_int,
        volume: c_uchar,
        sfxHandle: sfxHandle_t,
    );
    pub fn S_MuteSound(entityNum: c_int, entchannel: c_int);
    pub fn S_StartSound(
        origin: *const vec3_t,
        entnum: c_int,
        entchannel: c_int,
        sfx: sfxHandle_t,
    );
    pub fn S_StartLocalSound(sfx: sfxHandle_t, channelNum: c_int);
    pub fn S_StartLocalLoopingSound(sfx: sfxHandle_t);

    pub fn S_UnCacheDynamicMusic();
    pub fn S_RestartMusic();
    pub fn S_StartBackgroundTrack(
        intro: *const c_char,
        r#loop: *const c_char,
        bCalledByCGameStart: c_int,
    );
    pub fn S_StopBackgroundTrack();
    pub fn S_GetSampleLengthInMilliSeconds(sfxHandle: sfxHandle_t) -> c_float;

    // cinematics and voice-over-network will send raw samples
    // 1.0 volume will be direct output of source samples
    pub fn S_RawSamples(
        samples: c_int,
        rate: c_int,
        width: c_int,
        s_channels: c_int,
        data: *const byte,
        volume: c_float,
        bFirstOrOnlyUpdateThisFrame: c_int,
    );
    // stop all sounds
    pub fn S_StopSounds(); // from snd_dma.cpp
    // stop all sounds and the background track
    pub fn S_StopAllSounds();

    // scan all MP3s in the sound dir and add maxvol info if necessary.
    pub fn S_MP3_CalcVols_f();

    // all continuous looping sounds must be added before calling S_Update
    pub fn S_ClearLoopingSounds();
    pub fn S_StopLoopingSound(entityNum: c_int);
    #[cfg(feature = "xbox")]
    pub fn S_AddLoopingSound(
        entityNum: c_int,
        origin: *const vec3_t,
        velocity: *const vec3_t,
        sfx: sfxHandle_t,
        chan: c_int,
    );
    #[cfg(not(feature = "xbox"))]
    pub fn S_AddLoopingSound(
        entityNum: c_int,
        origin: *const vec3_t,
        velocity: *const vec3_t,
        sfx: sfxHandle_t,
    );

    // recompute the reletive volumes for all running sounds
    // relative to the given entityNum / orientation
    pub fn S_Respatialize(entityNum: c_int, head: *const vec3_t, axis: *mut vec3_t, inwater: c_int);

    // let the sound system know where an entity currently is
    pub fn S_UpdateEntityPosition(entityNum: c_int, origin: *const vec3_t);

    pub fn S_Update();

    pub fn S_DisableSounds();

    pub fn S_BeginRegistration();

    // RegisterSound will allways return a valid sample, even if it
    // has to create a placeholder.  This prevents continuous filesystem
    // checks for missing files
    pub fn S_RegisterSound(sample: *const c_char) -> sfxHandle_t;

    pub static mut s_shutUp: qboolean;

    pub fn S_FreeAllSFXMem();
}
