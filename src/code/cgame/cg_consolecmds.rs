// cg_consolecmds.c -- text commands typed in at the local console, or
// executed by a key binding

// this line must stay at top so the whole PCH thing works...
// #include "cg_headers.h"

// #include "cg_local.h"
// #include "cg_media.h"	//just for cgs....

use core::ffi::{c_char, c_int, c_void};

extern "C" {
    fn CG_TargetCommand_f();
    static mut player_locked: c_int;
    fn CMD_CGCam_Disable();
    fn CG_NextInventory_f();
    fn CG_PrevInventory_f();
    fn CG_NextForcePower_f();
    fn CG_PrevForcePower_f();
    fn CG_Printf(fmt: *const c_char, ...);
    fn CG_Argv(arg: c_int) -> *const c_char;
    static mut cgs: cgs_t;
    static mut cg: cg_t;
    static mut cg_entities: *mut entityState_t;
    static mut in_camera: c_int;
    static mut cg_fov: cvar_t;
    static mut cg_hudFiles: cvar_t;
    static mut cg_zoomFov: f32;
    fn cgi_S_StartSound(
        origin: *mut c_void,
        entityNum: c_int,
        entchannel: c_int,
        sfxHandle: sfxHandle_t,
    );
    fn cgi_FF_Start(effect: sfxHandle_t, clientNum: c_int);
    fn cgi_AddCommand(cmd_name: *const c_char);
    fn cgi_Argc() -> c_int;
    fn CG_TestModel_f();
    fn CG_TestModelNextFrame_f();
    fn CG_TestModelPrevFrame_f();
    fn CG_TestModelNextSkin_f();
    fn CG_TestModelPrevSkin_f();
    fn CG_TestG2Model_f();
    fn CG_TestModelSurfaceOnOff_f();
    fn CG_TestModelSetAnglespre_f();
    fn CG_TestModelSetAnglespost_f();
    fn CG_TestModelAnimate_f();
    fn CG_ListModelBones_f();
    fn CG_ListModelSurfaces_f();
    fn CG_NextWeapon_f();
    fn CG_PrevWeapon_f();
    fn CG_Weapon_f();
    fn CGCam_Enable();
    fn CG_DPNextWeapon_f();
    fn CG_DPPrevWeapon_f();
    fn CG_DPNextInventory_f();
    fn CG_DPPrevInventory_f();
    fn CG_DPNextForcePower_f();
    fn CG_DPPrevForcePower_f();
    fn CG_LoadMenus(filename: *const c_char);
    fn Menu_Reset();
    fn Q_stricmp(s0: *const c_char, s1: *const c_char) -> c_int;
}

pub type qboolean = c_int;
pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub type sfxHandle_t = c_int;

#[repr(C)]
pub struct cvar_t {
    pub string: *const c_char,
    pub integer: c_int,
    pub value: f32,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct refdef_t {
    pub vieworg: [f32; 3],
    // other fields omitted for brevity
}

#[repr(C)]
pub struct media_t {
    pub zoomStart: sfxHandle_t,
    pub zoomEnd: sfxHandle_t,
    pub zoomStartForce: sfxHandle_t,
    pub zoomEndForce: sfxHandle_t,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct cgs_t {
    pub mapname: [c_char; 64],
    pub media: media_t,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct cg_overrides_t {
    pub active: c_int,
    pub fov: f32,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct saber_t {
    // fields omitted for brevity
}

impl saber_t {
    pub fn Active(&self) -> qboolean {
        // stub implementation
        qfalse
    }
}

#[repr(C)]
pub struct playerState_t {
    pub saber: [saber_t; 2],
    pub saberInFlight: qboolean,
    pub stats: [c_int; 16],
    pub clientNum: c_int,
    pub viewEntity: c_int,
    pub batteryCharge: c_int,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct snapshot_t {
    pub ps: playerState_t,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct entityState_s {
    pub eFlags: c_int,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct entityState_t {
    pub currentState: entityState_s,
    // other fields omitted for brevity
}

#[repr(C)]
pub struct cg_t {
    pub refdef: refdef_t,
    pub refdefViewAngles: [f32; 3],
    pub zoomMode: c_int,
    pub zoomLocked: qboolean,
    pub zoomTime: c_int,
    pub snap: *mut snapshot_t,
    pub time: c_int,
    pub overrides: cg_overrides_t,
    // other fields omitted for brevity
}

pub const EF_LOCKED_TO_WEAPON: c_int = 1 << 3;
pub const EF_IN_ATST: c_int = 1 << 4;
pub const STAT_HEALTH: c_int = 0;
pub const CHAN_AUTO: c_int = 0;
pub const CG_OVERRIDE_FOV: c_int = 1 << 2;
pub const YAW: usize = 1;

/*
====================
CG_ColorFromString
====================
*/
/*
static void CG_SetColor_f( void) {

	if (cgi_Argc()==4)
	{
		g_entities[0].client->renderInfo.customRGBA[0] = atoi( CG_Argv(1) );
		g_entities[0].client->renderInfo.customRGBA[1] = atoi( CG_Argv(2) );
		g_entities[0].client->renderInfo.customRGBA[2] = atoi( CG_Argv(3) );
	}
	if (cgi_Argc()==2)
	{
		int val = atoi( CG_Argv(1) );

		if ( val < 1 || val > 7 ) {
			g_entities[0].client->renderInfo.customRGBA[0] = 255;
			g_entities[0].client->renderInfo.customRGBA[1] = 255;
			g_entities[0].client->renderInfo.customRGBA[2] = 255;
			return;
		}
		g_entities[0].client->renderInfo.customRGBA[0]=0;
		g_entities[0].client->renderInfo.customRGBA[1]=0;
		g_entities[0].client->renderInfo.customRGBA[2]=0;

		if ( val & 1 ) {
			g_entities[0].client->renderInfo.customRGBA[2] = 255;
		}
		if ( val & 2 ) {
			g_entities[0].client->renderInfo.customRGBA[1] = 255;
		}
		if ( val & 4 ) {
			g_entities[0].client->renderInfo.customRGBA[0] = 255;
		}
	}
}
*/
/*
=============
CG_Viewpos_f

Debugging command to print the current position
=============
*/
#[allow(non_snake_case)]
pub extern "C" fn CG_Viewpos_f() {
    unsafe {
        let fmt = b"%s (%i %i %i) : %i\n\0".as_ptr() as *const c_char;
        CG_Printf(
            fmt,
            (*core::ptr::addr_of!(cgs)).mapname.as_ptr(),
            (*core::ptr::addr_of!(cg)).refdef.vieworg[0] as c_int,
            (*core::ptr::addr_of!(cg)).refdef.vieworg[1] as c_int,
            (*core::ptr::addr_of!(cg)).refdef.vieworg[2] as c_int,
            (*core::ptr::addr_of!(cg)).refdefViewAngles[YAW] as c_int,
        );
    }
}

#[allow(non_snake_case)]
pub extern "C" fn CG_WriteCam_f() {
    let mut text: [c_char; 1024] = [0; 1024];
    let mut targetname: *const c_char;
    static mut numCams: c_int = 0;

    unsafe {
        numCams += 1;

        targetname = CG_Argv(1);

        if targetname.is_null() || *targetname == 0 {
            targetname = b"nameme!\0".as_ptr() as *const c_char;
        }

        let fmt = b"Camera #%d ('%s') written to: \0".as_ptr() as *const c_char;
        CG_Printf(fmt, numCams, targetname);

        // sprintf call - using a format string
        let sprintf_fmt = b"//entity %d\n{\n\"classname\"\t\"ref_tag\"\n\"targetname\"\t\"%s\"\n\"origin\" \"%i %i %i\"\n\"angles\" \"%i %i %i\"\n\"fov\" \"%i\"\n}\n\0".as_ptr() as *const c_char;
        libc::sprintf(
            text.as_mut_ptr(),
            sprintf_fmt,
            numCams,
            targetname,
            (*core::ptr::addr_of!(cg)).refdef.vieworg[0] as c_int,
            (*core::ptr::addr_of!(cg)).refdef.vieworg[1] as c_int,
            (*core::ptr::addr_of!(cg)).refdef.vieworg[2] as c_int,
            (*core::ptr::addr_of!(cg)).refdefViewAngles[0] as c_int,
            (*core::ptr::addr_of!(cg)).refdefViewAngles[1] as c_int,
            (*core::ptr::addr_of!(cg)).refdefViewAngles[2] as c_int,
            (*core::ptr::addr_of!(cg_fov)).integer,
        );
        // TODO: gi.WriteCam is not defined in C++ - need to call through proper interface
        // gi.WriteCam(text.as_ptr());
    }
}

#[allow(non_snake_case)]
pub extern "C" fn Lock_Disable() {
    unsafe {
        player_locked = qfalse;
    }
}

/*
=============
CG_ToggleBinoculars
=============
*/
#[allow(non_snake_case)]
pub extern "C" fn CG_ToggleBinoculars() {
    unsafe {
        if in_camera != 0 || (*core::ptr::addr_of!(cg)).snap.is_null() {
            return;
        }

        if (*core::ptr::addr_of!(cg)).zoomMode == 0 || (*core::ptr::addr_of!(cg)).zoomMode >= 2 {
            // not zoomed or currently zoomed with the disruptor or LA goggles
            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            if (snap_ref.ps.saber[0].Active() != 0 && snap_ref.ps.saberInFlight != 0)
                || snap_ref.ps.stats[STAT_HEALTH as usize] <= 0
            {
                //can't select binoculars when throwing saber
                //FIXME: indicate this to the player
                return;
            }

            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            if snap_ref.ps.viewEntity != 0
                || ((*cg_entities.add(snap_ref.ps.clientNum as usize))
                    .currentState
                    .eFlags
                    & (EF_LOCKED_TO_WEAPON | EF_IN_ATST))
                    != 0
            {
                // can't zoom when you have a viewEntity or driving an atst or in an emplaced gun
                return;
            }

            (*core::ptr::addr_of_mut!(cg)).zoomMode = 1;
            (*core::ptr::addr_of_mut!(cg)).zoomLocked = qfalse;

            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            if snap_ref.ps.batteryCharge != 0 {
                // when you have batteries, you can actually zoom in
                cg_zoomFov = 40.0f32;
            } else if ((*core::ptr::addr_of!(cg)).overrides.active & CG_OVERRIDE_FOV) != 0 {
                cg_zoomFov = (*core::ptr::addr_of!(cg)).overrides.fov;
            } else {
                cg_zoomFov = (*core::ptr::addr_of!(cg_fov)).value;
            }

            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            cgi_S_StartSound(
                core::ptr::null_mut(),
                snap_ref.ps.clientNum,
                CHAN_AUTO,
                (*core::ptr::addr_of!(cgs)).media.zoomStart,
            );
            #[cfg(feature = "IMMERSION")]
            {
                cgi_FF_Start(
                    (*core::ptr::addr_of!(cgs)).media.zoomStartForce,
                    snap_ref.ps.clientNum,
                );
            }
        } else {
            (*core::ptr::addr_of_mut!(cg)).zoomMode = 0;
            (*core::ptr::addr_of_mut!(cg)).zoomTime = (*core::ptr::addr_of!(cg)).time;
            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            cgi_S_StartSound(
                core::ptr::null_mut(),
                snap_ref.ps.clientNum,
                CHAN_AUTO,
                (*core::ptr::addr_of!(cgs)).media.zoomEnd,
            );
            #[cfg(feature = "IMMERSION")]
            {
                cgi_FF_Start(
                    (*core::ptr::addr_of!(cgs)).media.zoomEndForce,
                    snap_ref.ps.clientNum,
                );
            }
        }
    }
}

/*
=============
CG_ToggleLAGoggles
=============
*/
#[allow(non_snake_case)]
pub extern "C" fn CG_ToggleLAGoggles() {
    unsafe {
        if in_camera != 0 || (*core::ptr::addr_of!(cg)).snap.is_null() {
            return;
        }

        if (*core::ptr::addr_of!(cg)).zoomMode == 0 || (*core::ptr::addr_of!(cg)).zoomMode < 3 {
            // not zoomed or currently zoomed with the disruptor or regular binoculars
            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            if (snap_ref.ps.saber[0].Active() != 0 && snap_ref.ps.saberInFlight != 0)
                || snap_ref.ps.stats[STAT_HEALTH as usize] <= 0
            {
                //can't select binoculars when throwing saber
                //FIXME: indicate this to the player
                return;
            }

            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            if snap_ref.ps.viewEntity != 0
                || ((*cg_entities.add(snap_ref.ps.clientNum as usize))
                    .currentState
                    .eFlags
                    & (EF_LOCKED_TO_WEAPON | EF_IN_ATST))
                    != 0
            {
                // can't zoom when you have a viewEntity or driving an atst or in an emplaced gun
                return;
            }

            (*core::ptr::addr_of_mut!(cg)).zoomMode = 3;
            (*core::ptr::addr_of_mut!(cg)).zoomLocked = qfalse;
            if ((*core::ptr::addr_of!(cg)).overrides.active & CG_OVERRIDE_FOV) != 0 {
                cg_zoomFov = (*core::ptr::addr_of!(cg)).overrides.fov;
            } else {
                cg_zoomFov = (*core::ptr::addr_of!(cg_fov)).value; // does not zoom!!
            }

            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            cgi_S_StartSound(
                core::ptr::null_mut(),
                snap_ref.ps.clientNum,
                CHAN_AUTO,
                (*core::ptr::addr_of!(cgs)).media.zoomStart,
            );
            #[cfg(feature = "IMMERSION")]
            {
                cgi_FF_Start(
                    (*core::ptr::addr_of!(cgs)).media.zoomStartForce,
                    snap_ref.ps.clientNum,
                );
            }
        } else {
            (*core::ptr::addr_of_mut!(cg)).zoomMode = 0;
            (*core::ptr::addr_of_mut!(cg)).zoomTime = (*core::ptr::addr_of!(cg)).time;
            let snap_ref = &*(*core::ptr::addr_of!(cg)).snap;
            cgi_S_StartSound(
                core::ptr::null_mut(),
                snap_ref.ps.clientNum,
                CHAN_AUTO,
                (*core::ptr::addr_of!(cgs)).media.zoomEnd,
            );
            #[cfg(feature = "IMMERSION")]
            {
                cgi_FF_Start(
                    (*core::ptr::addr_of!(cgs)).media.zoomEndForce,
                    snap_ref.ps.clientNum,
                );
            }
        }
    }
}

#[allow(non_snake_case)]
pub extern "C" fn CG_InfoDown_f() {
    //	cg.showInformation = qtrue;
}

#[allow(non_snake_case)]
pub extern "C" fn CG_InfoUp_f() {
    //	cg.showInformation = qfalse;
}

#[repr(C)]
pub struct consoleCommand_t {
    pub cmd: *const c_char,
    pub function: Option<extern "C" fn()>,
}

static COMMANDS: &[consoleCommand_t] = &[
    consoleCommand_t {
        cmd: b"testmodel\0".as_ptr() as *const c_char,
        function: Some(CG_TestModel_f),
    },
    consoleCommand_t {
        cmd: b"nextframe\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelNextFrame_f),
    },
    consoleCommand_t {
        cmd: b"prevframe\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelPrevFrame_f),
    },
    consoleCommand_t {
        cmd: b"nextskin\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelNextSkin_f),
    },
    consoleCommand_t {
        cmd: b"prevskin\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelPrevSkin_f),
    },
    /*
    Ghoul2 Insert Start
    */
    consoleCommand_t {
        cmd: b"testG2Model\0".as_ptr() as *const c_char,
        function: Some(CG_TestG2Model_f),
    },
    consoleCommand_t {
        cmd: b"testsurface\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelSurfaceOnOff_f),
    },
    consoleCommand_t {
        cmd: b"testanglespre\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelSetAnglespre_f),
    },
    consoleCommand_t {
        cmd: b"testanglespost\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelSetAnglespost_f),
    },
    consoleCommand_t {
        cmd: b"testanimate\0".as_ptr() as *const c_char,
        function: Some(CG_TestModelAnimate_f),
    },
    consoleCommand_t {
        cmd: b"testlistbones\0".as_ptr() as *const c_char,
        function: Some(CG_ListModelBones_f),
    },
    consoleCommand_t {
        cmd: b"testlistsurfaces\0".as_ptr() as *const c_char,
        function: Some(CG_ListModelSurfaces_f),
    },
    /*
    Ghoul2 Insert End
    */
    consoleCommand_t {
        cmd: b"viewpos\0".as_ptr() as *const c_char,
        function: Some(CG_Viewpos_f),
    },
    consoleCommand_t {
        cmd: b"writecam\0".as_ptr() as *const c_char,
        function: Some(CG_WriteCam_f),
    },
    consoleCommand_t {
        cmd: b"+info\0".as_ptr() as *const c_char,
        function: Some(CG_InfoDown_f),
    },
    consoleCommand_t {
        cmd: b"-info\0".as_ptr() as *const c_char,
        function: Some(CG_InfoUp_f),
    },
    consoleCommand_t {
        cmd: b"weapnext\0".as_ptr() as *const c_char,
        function: Some(CG_NextWeapon_f),
    },
    consoleCommand_t {
        cmd: b"weapprev\0".as_ptr() as *const c_char,
        function: Some(CG_PrevWeapon_f),
    },
    consoleCommand_t {
        cmd: b"weapon\0".as_ptr() as *const c_char,
        function: Some(CG_Weapon_f),
    },
    consoleCommand_t {
        cmd: b"tcmd\0".as_ptr() as *const c_char,
        function: Some(CG_TargetCommand_f),
    },
    consoleCommand_t {
        cmd: b"cam_disable\0".as_ptr() as *const c_char,
        function: Some(CMD_CGCam_Disable), //gets out of camera mode for debuggin
    },
    consoleCommand_t {
        cmd: b"cam_enable\0".as_ptr() as *const c_char,
        function: Some(CGCam_Enable), //gets into camera mode for precise camera placement
    },
    consoleCommand_t {
        cmd: b"lock_disable\0".as_ptr() as *const c_char,
        function: Some(Lock_Disable), //player can move now
    },
    consoleCommand_t {
        cmd: b"zoom\0".as_ptr() as *const c_char,
        function: Some(CG_ToggleBinoculars),
    },
    consoleCommand_t {
        cmd: b"la_zoom\0".as_ptr() as *const c_char,
        function: Some(CG_ToggleLAGoggles),
    },
    consoleCommand_t {
        cmd: b"invnext\0".as_ptr() as *const c_char,
        function: Some(CG_NextInventory_f),
    },
    consoleCommand_t {
        cmd: b"invprev\0".as_ptr() as *const c_char,
        function: Some(CG_PrevInventory_f),
    },
    consoleCommand_t {
        cmd: b"forcenext\0".as_ptr() as *const c_char,
        function: Some(CG_NextForcePower_f),
    },
    consoleCommand_t {
        cmd: b"forceprev\0".as_ptr() as *const c_char,
        function: Some(CG_PrevForcePower_f),
    },
    consoleCommand_t {
        cmd: b"loadhud\0".as_ptr() as *const c_char,
        function: Some(CG_LoadHud_f),
    },
    consoleCommand_t {
        cmd: b"dpweapnext\0".as_ptr() as *const c_char,
        function: Some(CG_DPNextWeapon_f),
    },
    consoleCommand_t {
        cmd: b"dpweapprev\0".as_ptr() as *const c_char,
        function: Some(CG_DPPrevWeapon_f),
    },
    consoleCommand_t {
        cmd: b"dpinvnext\0".as_ptr() as *const c_char,
        function: Some(CG_DPNextInventory_f),
    },
    consoleCommand_t {
        cmd: b"dpinvprev\0".as_ptr() as *const c_char,
        function: Some(CG_DPPrevInventory_f),
    },
    consoleCommand_t {
        cmd: b"dpforcenext\0".as_ptr() as *const c_char,
        function: Some(CG_DPNextForcePower_f),
    },
    consoleCommand_t {
        cmd: b"dpforceprev\0".as_ptr() as *const c_char,
        function: Some(CG_DPPrevForcePower_f),
    },
    //	{ "color", CG_SetColor_f },
];

//extern menuDef_t *menuScoreboard;

#[allow(non_snake_case)]
pub extern "C" fn CG_LoadHud_f() {
    let mut hudSet: *const c_char;

    //	cgi_UI_String_Init();

    //	cgi_UI_Menu_Reset();

    unsafe {
        hudSet = (*core::ptr::addr_of!(cg_hudFiles)).string;
        if *hudSet == b'\0' as c_char {
            hudSet = b"ui/jahud.txt\0".as_ptr() as *const c_char;
        }

        CG_LoadMenus(hudSet);
    }
    //	menuScoreboard = NULL;
}

/*
=================
CG_ConsoleCommand

The string has been tokenized and can be retrieved with
Cmd_Argc() / Cmd_Argv()
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn CG_ConsoleCommand() -> qboolean {
    let mut cmd: *const c_char;
    let mut i: c_int;

    unsafe {
        cmd = CG_Argv(0);

        i = 0;
        while (i as usize) < COMMANDS.len() {
            if Q_stricmp(cmd, COMMANDS[i as usize].cmd) == 0 {
                if let Some(func) = COMMANDS[i as usize].function {
                    func();
                }
                return qtrue;
            }
            i += 1;
        }
    }

    qfalse
}

/*
=================
CG_InitConsoleCommands

Let the client system know about all of our commands
so it can perform tab completion
=================
*/
#[allow(non_snake_case)]
pub extern "C" fn CG_InitConsoleCommands() {
    let mut i: c_int;

    unsafe {
        i = 0;
        while (i as usize) < COMMANDS.len() {
            cgi_AddCommand(COMMANDS[i as usize].cmd);
            i += 1;
        }
    }
}
