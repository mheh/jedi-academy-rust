// IcarusInterface.h: ICARUS Interface header file.
// -Date: ~October, 2002
// -Created by: Mike Crowns and Aurelio Reis.
// -Description: The new interface between a Game Engine and the Icarus Scripting Language.
// An Interface is an Abstract Base Class with pure virtual members that MUST be implemented
// in order for the compile to succeed. Because of this, all needed functionality can be
// added without compromising other core systems.
// -Usage: To use the new Icarus Interface, two classes must be derived. The first is the
// actual Icarus Interface class which contains all relevent functionality to the scripting
// system. The second is the Game Interface which is very much more broad and thus implemented
// by the user. Icarus functions by calling the Game Interface to do certain tasks for it. This
// is why the Game Interface is required to have certain functions implemented.

#![allow(non_snake_case)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_ulong, c_uint, c_void};

// The basic Icarus Interface ABC.
pub trait IIcarusInterface {
    const ICARUS_INVALID: c_int = 0;

    // Get a static singleton instance (of a specific flavor).
    // must be implemented along with concrete class
    fn GetIcarus(flavor: c_int, constructIfNecessary: bool) -> *mut dyn IIcarusInterface;
    // Destroy the static singleton instance.
    fn DestroyIcarus();

    fn GetFlavor(&self) -> c_int;

    // Save all Icarus states.
    fn Save(&self) -> c_int;
    // Load all Icarus states.
    fn Load(&self) -> c_int;

    // Execute a script.
    fn Run(&mut self, icarusID: c_int, buffer: *mut c_char, length: i64) -> c_int;
    // Delete an Icarus ID from the list (making the ID Invalid on the other end through reference).
    fn DeleteIcarusID(&mut self, icarusID: &mut c_int);
    // Get an Icarus ID.
    fn GetIcarusID(&self, gameID: c_int) -> c_int;
    // Update all internal Icarus structures.
    fn Update(&mut self, icarusID: c_int) -> c_int;
    // Whether a Icarus is running or not.
    fn IsRunning(&self, icarusID: c_int) -> c_int;
    // Tells Icarus a task is completed.
    fn Completed(&mut self, icarusID: c_int, taskID: c_int);
    // Precache a Script in memory.
    fn Precache(&mut self, buffer: *mut c_char, length: i64);
}

// Description: The Game Interface is used by the Icarus Interface to access specific
// data or initiate certain things within the engine being used. It is made to be
// as generic as possible to allow any engine to derive it's own interface for use.
// Created: 10/08/02 by Aurelio Reis.

// For system-wide prints
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum e_DebugPrintLevel {
    WL_ERROR = 1,
    WL_WARNING = 2,
    WL_VERBOSE = 3,
    WL_DEBUG = 4,
}

pub trait IGameInterface {
    // How many flavors are needed.
    fn get_s_IcarusFlavorsNeeded() -> c_int;

    // Get a static singleton instance (of a specific flavor).
    fn GetGame(flavor: c_int) -> *mut dyn IGameInterface;

    // Destroy the static singleton instance (NOTE: Destroy the Game Interface BEFORE the Icarus Interface).
    fn Destroy();

    // General
    // Load a script File into the destination buffer. If the script has already been loaded
    // NOTE: This is what was called before:
    // /*
    // // Description	: Reads in a file and attaches the script directory properly
    // extern int ICARUS_GetScript( const char *name, char **buf );	//g_icarus.cpp
    // static int Q3_ReadScript( const char *name, void **buf )
    // {
    //     return ICARUS_GetScript( va( "%s/%s", Q3_SCRIPT_DIR, name ), (char**)buf );	//get a (hopefully) cached file
    // }
    // */
    fn GetFlavor(&self) -> c_int;

    fn LoadFile(&mut self, name: *const c_char, buf: *mut *mut c_void) -> c_int;
    fn CenterPrint(&mut self, format: *const c_char);
    fn DebugPrint(&mut self, level: e_DebugPrintLevel, fmt: *const c_char);
    // Gets the current time
    fn GetTime(&self) -> c_uint;
    fn PlaySound(&mut self, taskID: c_int, gameID: c_int, name: *const c_char, channel: *const c_char) -> c_int;
    fn Lerp2Pos(&mut self, taskID: c_int, gameID: c_int, origin: *mut [f32; 3], angles: *mut [f32; 3], duration: f32);
    fn Lerp2Angles(&mut self, taskID: c_int, gameID: c_int, angles: *mut [f32; 3], duration: f32);
    fn GetTag(&mut self, gameID: c_int, name: *const c_char, lookup: c_int, info: *mut [f32; 3]) -> c_int;
    fn Set(&mut self, taskID: c_int, gameID: c_int, type_name: *const c_char, data: *const c_char);
    fn Use(&mut self, gameID: c_int, name: *const c_char);
    fn Activate(&mut self, gameID: c_int, name: *const c_char);
    fn Deactivate(&mut self, gameID: c_int, name: *const c_char);
    fn Kill(&mut self, gameID: c_int, name: *const c_char);
    fn Remove(&mut self, gameID: c_int, name: *const c_char);
    fn Random(&self, min: f32, max: f32) -> f32;
    fn Play(&mut self, taskID: c_int, gameID: c_int, r#type: *const c_char, name: *const c_char);

    // Camera functions
    fn CameraPan(&mut self, angles: *mut [f32; 3], dir: *mut [f32; 3], duration: f32);
    fn CameraMove(&mut self, origin: *mut [f32; 3], duration: f32);
    fn CameraZoom(&mut self, fov: f32, duration: f32);
    fn CameraRoll(&mut self, angle: f32, duration: f32);
    fn CameraFollow(&mut self, name: *const c_char, speed: f32, initLerp: f32);
    fn CameraTrack(&mut self, name: *const c_char, speed: f32, initLerp: f32);
    fn CameraDistance(&mut self, dist: f32, initLerp: f32);
    fn CameraFade(&mut self, sr: f32, sg: f32, sb: f32, sa: f32, dr: f32, dg: f32, db: f32, da: f32, duration: f32);
    fn CameraPath(&mut self, name: *const c_char);
    fn CameraEnable(&mut self);
    fn CameraDisable(&mut self);
    fn CameraShake(&mut self, intensity: f32, duration: c_int);

    fn GetFloat(&mut self, gameID: c_int, name: *const c_char, value: *mut f32) -> c_int;
    // Should be float return type?
    fn GetVector(&mut self, gameID: c_int, name: *const c_char, value: *mut [f32; 3]) -> c_int;
    fn GetString(&mut self, gameID: c_int, name: *const c_char, value: *mut *mut c_char) -> c_int;

    fn Evaluate(&mut self, p1Type: c_int, p1: *const c_char, p2Type: c_int, p2: *const c_char, operatorType: c_int) -> c_int;

    fn DeclareVariable(&mut self, r#type: c_int, name: *const c_char);
    fn FreeVariable(&mut self, name: *const c_char);

    // Save / Load functions

    fn WriteSaveData(&mut self, chid: c_ulong, data: *mut c_void, length: c_int) -> c_int;
    fn ReadSaveData(&mut self, chid: c_ulong, address: *mut c_void, length: c_int, addressptr: *mut *mut c_void) -> c_int;
    fn LinkGame(&mut self, gameID: c_int, icarusID: c_int) -> c_int;

    // Access functions

    fn CreateIcarus(&mut self, gameID: c_int) -> c_int;
    // Polls the engine for the sequencer of the entity matching the name passed
    fn GetByName(&self, name: *const c_char) -> c_int;
    // (g_entities[m_ownerID].svFlags&SVF_ICARUS_FREEZE)
    // return -1 indicates invalid
    fn IsFrozen(&self, gameID: c_int) -> c_int;
    fn Free(&mut self, data: *mut c_void);
    fn Malloc(&mut self, size: c_int) -> *mut c_void;
    fn MaxFloat(&self) -> f32;

    // Script precache functions.
    // G_LoadRoff
    fn PrecacheRoff(&mut self, name: *const c_char);
    // must strip extension COM_StripExtension()
    fn PrecacheScript(&mut self, name: *const c_char);
    // G_SoundIndex
    fn PrecacheSound(&mut self, name: *const c_char);
    fn PrecacheFromSet(&mut self, setname: *const c_char, filename: *const c_char);
}
