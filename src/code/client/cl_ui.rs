// leave this as first line for PCH reasons...
//

use core::ffi::{c_char, c_int, c_void};
use core::mem;

// From original includes: client.h, client_ui.h, vmachine.h

extern "C" {
    fn PC_ReadTokenHandle(handle: c_int, pc_token: *mut pc_token_s) -> c_int;
}

extern "C" {
    fn CL_UISystemCalls(args: *mut c_int) -> c_int;
}

// prototypes
// extern qboolean SG_GetSaveImage( const char *psPathlessBaseName, void *pvAddress );
extern "C" {
    fn SG_GetSaveGameComment(psPathlessBaseName: *const c_char, sComment: *mut c_char, sMapName: *mut c_char) -> c_int;
    fn SG_GameAllowedToSaveHere(inCamera: c_int) -> c_int;
    fn SG_StoreSaveGameComment(sComment: *const c_char);
    // extern byte *SCR_GetScreenshot(qboolean *qValid);
}

/*
====================
Helper functions for User Interface
====================
*/

/*
====================
GetClientState
====================
*/
fn GetClientState() -> connstate_t {
    unsafe { cls.state }
}

/*
====================
CL_GetGlConfig
====================
*/
fn UI_GetGlconfig(config: *mut glconfig_t) {
    unsafe {
        *config = cls.glconfig;
    }
}

/*
====================
GetClipboardData
====================
*/
fn GetClipboardData(buf: *mut c_char, buflen: c_int) {
    unsafe {
        let cbd = Sys_GetClipboardData();

        if cbd.is_null() {
            *buf = 0;
            return;
        }

        Q_strncpyz(buf, cbd, buflen);

        Z_Free(cbd as *mut c_void);
    }
}

/*
====================
Key_KeynumToStringBuf
====================
*/
// only ever called by binding-display code, therefore returns non-technical "friendly" names
//	in any language that don't necessarily match those in the config file...
//
fn Key_KeynumToStringBuf(keynum: c_int, buf: *mut c_char, buflen: c_int) {
    unsafe {
        let psKeyName = Key_KeynumToString(keynum/*, qtrue */);

        // see if there's a more friendly (or localised) name...
        //
        let psKeyNameFriendly = SE_GetString(va(b"KEYNAMES_KEYNAME_%s\0".as_ptr() as *const c_char, psKeyName));

        Q_strncpyz(
            buf,
            if !psKeyNameFriendly.is_null() && *psKeyNameFriendly != 0 {
                psKeyNameFriendly
            } else {
                psKeyName
            },
            buflen
        );
    }
}

/*
====================
Key_GetBindingBuf
====================
*/
fn Key_GetBindingBuf(keynum: c_int, buf: *mut c_char, buflen: c_int) {
    unsafe {
        let value = Key_GetBinding(keynum);
        if !value.is_null() {
            Q_strncpyz(buf, value, buflen);
        } else {
            *buf = 0;
        }
    }
}

/*
====================
Key_GetCatcher
====================
*/
fn Key_GetCatcher() -> c_int {
    unsafe { cls.keyCatchers }
}

/*
====================
Key_GetCatcher
====================
*/
fn Key_SetCatcher(catcher: c_int) {
    unsafe {
        cls.keyCatchers = catcher;
    }
}

/*
====================
FloatAsInt
====================
*/
fn FloatAsInt(f: f32) -> c_int {
    let mut temp: c_int = 0;

    unsafe {
        *((&mut temp as *mut c_int) as *mut f32) = f;
    }

    temp
}

fn UI_Cvar_Create(var_name: *const c_char, var_value: *const c_char, flags: c_int) {
    unsafe {
        Cvar_Get(var_name, var_value, flags);
    }
}

fn GetConfigString(index: c_int, buf: *mut c_char, size: c_int) -> c_int {
    unsafe {
        if index < 0 || index >= MAX_CONFIGSTRINGS {
            return 0; // qfalse
        }

        let offset = (*cl.gameState.stringOffsets.as_ptr().add(index as usize));
        if offset == 0 {
            return 0; // qfalse
        }

        Q_strncpyz(buf, (*cl.gameState.stringData).add(offset as usize), size);

        return 1; // qtrue
    }
}

/*
====================
CL_ShutdownUI
====================
*/
fn CL_ShutdownUI() {
    unsafe {
        cls.keyCatchers &= !KEYCATCH_UI;
        cls.uiStarted = 0; // qfalse
    }
}

fn CL_DrawDatapad(HUDType: c_int) {
    unsafe {
        match HUDType {
            DP_HUD => {
                VM_Call(CG_DRAW_DATAPAD_HUD);
            }
            DP_OBJECTIVES => {
                VM_Call(CG_DRAW_DATAPAD_OBJECTIVES);
            }
            DP_WEAPONS => {
                VM_Call(CG_DRAW_DATAPAD_WEAPONS);
            }
            DP_INVENTORY => {
                VM_Call(CG_DRAW_DATAPAD_INVENTORY);
            }
            DP_FORCEPOWERS => {
                VM_Call(CG_DRAW_DATAPAD_FORCEPOWERS);
            }
            _ => {}
        }
    }
}

extern "C" {
    fn UI_Init(apiVersion: c_int, uiimport: *mut uiimport_t, inGameLoad: c_int);
}

/*
====================
CL_InitUI
====================
*/
fn CL_InitUI() {
    let mut uii: uiimport_t = unsafe { mem::zeroed() };

    unsafe {
        uii.Printf = Some(Com_Printf);
        uii.Error = Some(Com_Error);

        uii.Cvar_Set = Some(Cvar_Set);
        uii.Cvar_VariableValue = Some(Cvar_VariableValue);
        uii.Cvar_VariableStringBuffer = Some(Cvar_VariableStringBuffer);
        uii.Cvar_SetValue = Some(Cvar_SetValue);
        uii.Cvar_Reset = Some(Cvar_Reset);
        uii.Cvar_Create = Some(UI_Cvar_Create);
        uii.Cvar_InfoStringBuffer = Some(Cvar_InfoStringBuffer);

        uii.Draw_DataPad = Some(CL_DrawDatapad);

        uii.Argc = Some(Cmd_Argc);
        uii.Argv = Some(Cmd_ArgvBuffer);
        uii.Cmd_TokenizeString = Some(Cmd_TokenizeString);

        uii.Cmd_ExecuteText = Some(Cbuf_ExecuteText);

        uii.FS_FOpenFile = Some(FS_FOpenFileByMode);
        uii.FS_Read = Some(FS_Read);
        uii.FS_Write = Some(FS_Write);
        uii.FS_FCloseFile = Some(FS_FCloseFile);
        uii.FS_GetFileList = Some(FS_GetFileList);
        uii.FS_ReadFile = Some(FS_ReadFile);
        uii.FS_FreeFile = Some(FS_FreeFile);

        uii.R_RegisterModel = Some(re.RegisterModel);
        uii.R_RegisterSkin = Some(re.RegisterSkin);
        uii.R_RegisterShader = Some(re.RegisterShader);
        uii.R_RegisterShaderNoMip = Some(re.RegisterShaderNoMip);
        uii.R_RegisterFont = Some(re.RegisterFont);
        #[cfg(not(target_os = "windows"))]
        {
            uii.R_Font_StrLenPixels = Some(re.Font_StrLenPixels);
            uii.R_Font_HeightPixels = Some(re.Font_HeightPixels);
            uii.R_Font_DrawString = Some(re.Font_DrawString);
        }
        uii.R_Font_StrLenChars = Some(re.R_Font_StrLenChars);
        uii.Language_IsAsian = Some(re.Language_IsAsian);
        uii.Language_UsesSpaces = Some(re.Language_UsesSpaces);
        uii.AnyLanguage_ReadCharFromString = Some(re.AnyLanguage_ReadCharFromString);

        //uii.SG_GetSaveImage = SG_GetSaveImage;
        uii.SG_GetSaveGameComment = Some(SG_GetSaveGameComment);
        uii.SG_StoreSaveGameComment = Some(SG_StoreSaveGameComment);
        uii.SG_GameAllowedToSaveHere = Some(SG_GameAllowedToSaveHere);

        //uii.SCR_GetScreenshot = SCR_GetScreenshot;

        //uii.DrawStretchRaw = re.DrawStretchRaw;
        uii.R_ClearScene = Some(re.ClearScene);
        uii.R_AddRefEntityToScene = Some(re.AddRefEntityToScene);
        uii.R_AddPolyToScene = Some(re.AddPolyToScene);
        uii.R_AddLightToScene = Some(re.AddLightToScene);
        uii.R_RenderScene = Some(re.RenderScene);

        uii.R_ModelBounds = Some(re.ModelBounds);

        uii.R_SetColor = Some(re.SetColor);
        uii.R_DrawStretchPic = Some(re.DrawStretchPic);
        uii.UpdateScreen = Some(SCR_UpdateScreen);

        #[cfg(target_os = "windows")]
        {
            uii.PrecacheScreenshot = Some(SCR_PrecacheScreenshot);
        }

        uii.R_LerpTag = Some(re.LerpTag);

        uii.S_StartLocalLoopingSound = Some(S_StartLocalLoopingSound);
        uii.S_StartLocalSound = Some(S_StartLocalSound);
        uii.S_RegisterSound = Some(S_RegisterSound);

        uii.Key_KeynumToStringBuf = Some(Key_KeynumToStringBuf);
        uii.Key_GetBindingBuf = Some(Key_GetBindingBuf);
        uii.Key_SetBinding = Some(Key_SetBinding);
        uii.Key_IsDown = Some(Key_IsDown);
        uii.Key_GetOverstrikeMode = Some(Key_GetOverstrikeMode);
        uii.Key_SetOverstrikeMode = Some(Key_SetOverstrikeMode);
        uii.Key_ClearStates = Some(Key_ClearStates);
        uii.Key_GetCatcher = Some(Key_GetCatcher);
        uii.Key_SetCatcher = Some(Key_SetCatcher);

        uii.GetClipboardData = Some(GetClipboardData);

        uii.GetClientState = Some(GetClientState);

        uii.GetGlconfig = Some(UI_GetGlconfig);

        uii.GetConfigString = Some(core::mem::transmute::<
            unsafe extern "C" fn(c_int, *mut c_char, c_int) -> c_int,
            unsafe extern "C" fn(c_int, *mut c_char, c_int),
        >(GetConfigString));

        uii.Milliseconds = Some(Sys_Milliseconds);

        let in_game_load = if cls.state > CA_DISCONNECTED && cls.state <= CA_ACTIVE {
            1
        } else {
            0
        };
        UI_Init(UI_API_VERSION, &mut uii, in_game_load);

        //JLF MPSKIPPED
        #[cfg(target_os = "windows")]
        {
            extern "C" {
                fn UpdateDemoTimer();
            }
            UpdateDemoTimer();
        }

        //	uie->UI_Init( UI_API_VERSION, &uii );
    }
}

fn UI_GameCommand() -> c_int {
    unsafe {
        if cls.uiStarted == 0 {
            return 0; // qfalse
        }
        UI_ConsoleCommand()
    }
}

fn CL_GenericMenu_f() {
    unsafe {
        let arg = Cmd_Argv(1);

        if cls.uiStarted != 0 {
            UI_SetActiveMenu(b"ingame\0".as_ptr() as *const c_char, arg);
        }
    }
}

fn CL_EndScreenDissolve_f() {
    unsafe {
        re.InitDissolve(1); // qtrue
    }
}

fn CL_DataPad_f() {
    unsafe {
        if cls.uiStarted != 0 && cls.cgameStarted != 0 && (cls.state == CA_ACTIVE) {
            UI_SetActiveMenu(b"datapad\0".as_ptr() as *const c_char, core::ptr::null());
        }
    }
}

/*
====================
CL_GetGlConfig
====================
*/
fn CL_GetGlconfig(config: *mut glconfig_t) {
    unsafe {
        *config = cls.glconfig;
    }
}

/*
int PC_ReadTokenHandle(int handle, pc_token_t *pc_token);
int PC_SourceFileAndLine(int handle, char *filename, int *line);
*/

/*
====================
CL_UISystemCalls

The ui module is making a system call
====================
*/
static mut uivm: vm_t = unsafe { core::mem::zeroed() };

// #define	VMA(x) ((void*)args[x])
// #define	VMF(x)	((float *)args)[x]

#[no_mangle]
pub extern "C" fn CL_UISystemCalls(args: *mut c_int) -> c_int {
    unsafe {
        match *args {
            UI_ERROR => {
                Com_Error(
                    ERR_DROP,
                    b"%s\0".as_ptr() as *const c_char,
                    *args.add(1) as *const c_char,
                );
                return 0;
            }

            UI_CVAR_REGISTER => {
                Cvar_Register(
                    *args.add(1) as *mut vmCvar_t,
                    *args.add(2) as *const c_char,
                    *args.add(3) as *const c_char,
                    *args.add(4),
                );
                return 0;
            }

            UI_CVAR_SET => {
                Cvar_Set(*args.add(1) as *const c_char, *args.add(2) as *const c_char);
                return 0;
            }

            UI_CVAR_SETVALUE => {
                Cvar_SetValue(
                    *args.add(1) as *const c_char,
                    *(args.add(2) as *const f32),
                );
                return 0;
            }

            UI_CVAR_UPDATE => {
                Cvar_Update(*args.add(1) as *mut vmCvar_t);
                return 0;
            }

            UI_R_REGISTERMODEL => {
                return re.RegisterModel(*args.add(1) as *const c_char);
            }

            UI_R_REGISTERSHADERNOMIP => {
                return re.RegisterShaderNoMip(*args.add(1) as *const c_char);
            }

            UI_GETGLCONFIG => {
                CL_GetGlconfig(*args.add(1) as *mut glconfig_t);
                return 0;
            }

            UI_CMD_EXECUTETEXT => {
                Cbuf_ExecuteText(*args.add(1), *args.add(2) as *const c_char);
                return 0;
            }

            UI_CVAR_VARIABLEVALUE => {
                return FloatAsInt(Cvar_VariableValue(*args.add(1) as *const c_char));
            }

            UI_FS_GETFILELIST => {
                return FS_GetFileList(
                    *args.add(1) as *const c_char,
                    *args.add(2) as *const c_char,
                    *args.add(3) as *mut c_char,
                    *args.add(4),
                );
            }

            UI_KEY_SETCATCHER => {
                Key_SetCatcher(*args.add(1));
                return 0;
            }

            UI_KEY_CLEARSTATES => {
                Key_ClearStates();
                return 0;
            }

            UI_R_SETCOLOR => {
                re.SetColor(*args.add(1) as *const f32);
                return 0;
            }

            UI_R_DRAWSTRETCHPIC => {
                re.DrawStretchPic(
                    *(args.add(1) as *const f32),
                    *(args.add(2) as *const f32),
                    *(args.add(3) as *const f32),
                    *(args.add(4) as *const f32),
                    *(args.add(5) as *const f32),
                    *(args.add(6) as *const f32),
                    *(args.add(7) as *const f32),
                    *(args.add(8) as *const f32),
                    *args.add(9),
                );
                return 0;
            }

            UI_CVAR_VARIABLESTRINGBUFFER => {
                Cvar_VariableStringBuffer(
                    *args.add(1) as *const c_char,
                    *args.add(2) as *mut c_char,
                    *args.add(3),
                );
                return 0;
            }

            UI_R_MODELBOUNDS => {
                re.ModelBounds(
                    *args.add(1),
                    *args.add(2) as *mut f32,
                    *args.add(3) as *mut f32,
                );
                return 0;
            }

            UI_R_CLEARSCENE => {
                re.ClearScene();
                return 0;
            }

            // case UI_KEY_GETOVERSTRIKEMODE:
            //	return Key_GetOverstrikeMode();
            //	return 0;

            // case UI_PC_READ_TOKEN:
            //	return PC_ReadTokenHandle( args[1], VMA(2) );

            // case UI_PC_SOURCE_FILE_AND_LINE:
            //	return PC_SourceFileAndLine( args[1], VMA(2), VMA(3) );

            UI_KEY_GETCATCHER => {
                return Key_GetCatcher();
            }

            UI_MILLISECONDS => {
                return Sys_Milliseconds();
            }

            UI_S_REGISTERSOUND => {
                return S_RegisterSound(*args.add(1) as *const c_char);
            }

            UI_S_STARTLOCALSOUND => {
                S_StartLocalSound(*args.add(1), *args.add(2));
                return 0;
            }

            // case UI_R_REGISTERFONT:
            //	re.RegisterFont( VMA(1), args[2], VMA(3));
            //	return 0;

            UI_CIN_PLAYCINEMATIC => {
                Com_DPrintf(b"UI_CIN_PlayCinematic\n\0".as_ptr() as *const c_char);
                return CIN_PlayCinematic(
                    *args.add(1) as *const c_char,
                    *args.add(2),
                    *args.add(3),
                    *args.add(4),
                    *args.add(5),
                    *args.add(6),
                    *args.add(7) as *const c_char,
                );
            }

            UI_CIN_STOPCINEMATIC => {
                return CIN_StopCinematic(*args.add(1));
            }

            UI_CIN_RUNCINEMATIC => {
                return CIN_RunCinematic(*args.add(1));
            }

            UI_CIN_DRAWCINEMATIC => {
                CIN_DrawCinematic(*args.add(1));
                return 0;
            }

            UI_KEY_SETBINDING => {
                Key_SetBinding(*args.add(1), *args.add(2) as *const c_char);
                return 0;
            }

            UI_KEY_KEYNUMTOSTRINGBUF => {
                Key_KeynumToStringBuf(*args.add(1), *args.add(2) as *mut c_char, *args.add(3));
                return 0;
            }

            UI_CIN_SETEXTENTS => {
                CIN_SetExtents(
                    *args.add(1),
                    *args.add(2),
                    *args.add(3),
                    *args.add(4),
                    *args.add(5),
                );
                return 0;
            }

            UI_KEY_GETBINDINGBUF => {
                Key_GetBindingBuf(*args.add(1), *args.add(2) as *mut c_char, *args.add(3));
                return 0;
            }

            _ => {
                Com_Error(
                    ERR_DROP,
                    b"Bad UI system trap: %i\0".as_ptr() as *const c_char,
                    *args,
                );
            }
        }
    }

    return 0;
}

// External function stubs and declarations
// These would normally come from included headers

extern "C" {
    static mut cls: client_static_t;
    static mut cl: client_t;
    static re: refexport_t;

    fn Sys_GetClipboardData() -> *mut c_char;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn Z_Free(ptr: *mut c_void);
    fn Key_KeynumToString(keynum: c_int) -> *const c_char;
    fn SE_GetString(label: *const c_char) -> *const c_char;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn Key_GetBinding(keynum: c_int) -> *const c_char;
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_Set(var_name: *const c_char, value: *const c_char);
    fn Cvar_VariableValue(var_name: *const c_char) -> f32;
    fn Cvar_VariableStringBuffer(var_name: *const c_char, buffer: *mut c_char, bufsize: c_int);
    fn Cvar_SetValue(var_name: *const c_char, value: f32);
    fn Cvar_Reset(var_name: *const c_char);
    fn Cvar_InfoStringBuffer(bit: c_int, buf: *mut c_char, bufsize: c_int);
    fn Cvar_Register(
        vmCvar: *mut vmCvar_t,
        varName: *const c_char,
        defaultValue: *const c_char,
        flags: c_int,
    );
    fn Cvar_Update(vmCvar: *mut vmCvar_t);
    fn Com_Printf(msg: *const c_char, ...);
    fn Com_Error(level: c_int, error: *const c_char, ...);
    fn Com_DPrintf(msg: *const c_char);
    fn Cmd_Argc() -> c_int;
    fn Cmd_Argv(arg: c_int) -> *const c_char;
    fn Cmd_ArgvBuffer(arg: c_int, buffer: *mut c_char, bufferLength: c_int);
    fn Cmd_TokenizeString(text: *const c_char) -> c_int;
    fn Cbuf_ExecuteText(exec_when: c_int, text: *const c_char);
    fn FS_FOpenFileByMode(
        filename: *const c_char,
        f: *mut *mut c_void,
        mode: c_int,
    ) -> c_int;
    fn FS_Read(buffer: *mut c_void, len: c_int, f: *mut c_void) -> c_int;
    fn FS_Write(buffer: *const c_void, len: c_int, f: *mut c_void) -> c_int;
    fn FS_FCloseFile(f: *mut c_void);
    fn FS_GetFileList(
        path: *const c_char,
        extension: *const c_char,
        listbuf: *mut c_char,
        bufsize: c_int,
    ) -> c_int;
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);
    fn VM_Call(command: c_int, ...) -> c_int;
    fn UI_SetActiveMenu(menu: *const c_char, arg: *const c_char);
    fn UI_ConsoleCommand() -> c_int;
    fn S_StartLocalLoopingSound(sfxHandle: c_int) -> c_int;
    fn S_StartLocalSound(sfxHandle: c_int, channelNum: c_int);
    fn S_RegisterSound(name: *const c_char) -> c_int;
    fn Sys_Milliseconds() -> c_int;
    fn Key_SetBinding(keynum: c_int, binding: *const c_char);
    fn Key_IsDown(keynum: c_int) -> c_int;
    fn Key_GetOverstrikeMode() -> c_int;
    fn Key_SetOverstrikeMode(state: c_int);
    fn Key_ClearStates();
    fn SCR_UpdateScreen();
    fn CIN_PlayCinematic(
        arg0: *const c_char,
        x: c_int,
        y: c_int,
        w: c_int,
        h: c_int,
        systemBitFlags: c_int,
        psDialogName: *const c_char,
    ) -> c_int;
    fn CIN_StopCinematic(handle: c_int) -> c_int;
    fn CIN_RunCinematic(handle: c_int) -> c_int;
    fn CIN_DrawCinematic(handle: c_int);
    fn CIN_SetExtents(handle: c_int, x: c_int, y: c_int, w: c_int, h: c_int);
}

// Type stubs for layout/ABI types
#[repr(C)]
pub struct client_static_t {
    pub state: connstate_t,
    pub keyCatchers: c_int,
    pub uiStarted: c_int,
    pub cgameStarted: c_int,
    pub glconfig: glconfig_t,
    // ... other fields not shown
}

#[repr(C)]
pub struct client_t {
    pub gameState: gameState_t,
    // ... other fields not shown
}

#[repr(C)]
pub struct gameState_t {
    pub stringOffsets: [c_int; 4096], // MAX_CONFIGSTRINGS
    pub stringData: *mut c_char,
    // ... other fields not shown
}

#[repr(C)]
pub struct glconfig_t {
    // placeholder for full glconfig structure
}

#[repr(C)]
pub struct refexport_t {
    pub RegisterModel: unsafe extern "C" fn(*const c_char) -> c_int,
    pub RegisterSkin: unsafe extern "C" fn(*const c_char) -> c_int,
    pub RegisterShader: unsafe extern "C" fn(*const c_char) -> c_int,
    pub RegisterShaderNoMip: unsafe extern "C" fn(*const c_char) -> c_int,
    pub RegisterFont: unsafe extern "C" fn(*const c_char) -> c_int,
    pub Font_StrLenPixels: unsafe extern "C" fn(*const c_char, c_int) -> c_int,
    pub Font_HeightPixels: unsafe extern "C" fn(*const c_char, c_int) -> c_int,
    pub Font_DrawString: unsafe extern "C" fn(c_int, c_int, *const c_char, *const f32, c_int, c_int) -> c_int,
    pub R_Font_StrLenChars: unsafe extern "C" fn(*const c_char) -> c_int,
    pub Language_IsAsian: unsafe extern "C" fn() -> c_int,
    pub Language_UsesSpaces: unsafe extern "C" fn() -> c_int,
    pub AnyLanguage_ReadCharFromString: unsafe extern "C" fn(*const c_char, *mut c_int, *mut c_char) -> c_int,
    pub ClearScene: unsafe extern "C" fn(),
    pub AddRefEntityToScene: unsafe extern "C" fn(*const c_void),
    pub AddPolyToScene: unsafe extern "C" fn(*const c_void),
    pub AddLightToScene: unsafe extern "C" fn(*const c_void),
    pub RenderScene: unsafe extern "C" fn(*const c_void),
    pub ModelBounds: unsafe extern "C" fn(c_int, *mut f32, *mut f32),
    pub SetColor: unsafe extern "C" fn(*const f32),
    pub DrawStretchPic: unsafe extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, c_int),
    pub LerpTag: unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, f32, *mut c_void) -> c_int,
    pub InitDissolve: unsafe extern "C" fn(c_int),
    // ... other fields not shown
}

#[repr(C)]
pub struct uiimport_t {
    pub Printf: Option<unsafe extern "C" fn(*const c_char, ...)>,
    pub Error: Option<unsafe extern "C" fn(c_int, *const c_char, ...)>,
    pub Cvar_Set: Option<unsafe extern "C" fn(*const c_char, *const c_char)>,
    pub Cvar_VariableValue: Option<unsafe extern "C" fn(*const c_char) -> f32>,
    pub Cvar_VariableStringBuffer: Option<unsafe extern "C" fn(*const c_char, *mut c_char, c_int)>,
    pub Cvar_SetValue: Option<unsafe extern "C" fn(*const c_char, f32)>,
    pub Cvar_Reset: Option<unsafe extern "C" fn(*const c_char)>,
    pub Cvar_Create: Option<unsafe extern "C" fn(*const c_char, *const c_char, c_int)>,
    pub Cvar_InfoStringBuffer: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
    pub Draw_DataPad: Option<unsafe extern "C" fn(c_int)>,
    pub Argc: Option<unsafe extern "C" fn() -> c_int>,
    pub Argv: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
    pub Cmd_TokenizeString: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub Cmd_ExecuteText: Option<unsafe extern "C" fn(c_int, *const c_char)>,
    pub FS_FOpenFile: Option<unsafe extern "C" fn(*const c_char, *mut *mut c_void, c_int) -> c_int>,
    pub FS_Read: Option<unsafe extern "C" fn(*mut c_void, c_int, *mut c_void) -> c_int>,
    pub FS_Write: Option<unsafe extern "C" fn(*const c_void, c_int, *mut c_void) -> c_int>,
    pub FS_FCloseFile: Option<unsafe extern "C" fn(*mut c_void)>,
    pub FS_GetFileList: Option<unsafe extern "C" fn(*const c_char, *const c_char, *mut c_char, c_int) -> c_int>,
    pub FS_ReadFile: Option<unsafe extern "C" fn(*const c_char, *mut *mut c_void) -> c_int>,
    pub FS_FreeFile: Option<unsafe extern "C" fn(*mut c_void)>,
    pub R_RegisterModel: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub R_RegisterSkin: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub R_RegisterShader: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub R_RegisterShaderNoMip: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub R_RegisterFont: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub R_Font_StrLenPixels: Option<unsafe extern "C" fn(*const c_char, c_int) -> c_int>,
    pub R_Font_HeightPixels: Option<unsafe extern "C" fn(*const c_char, c_int) -> c_int>,
    pub R_Font_DrawString: Option<unsafe extern "C" fn(c_int, c_int, *const c_char, *const f32, c_int, c_int)>,
    pub R_Font_StrLenChars: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub Language_IsAsian: Option<unsafe extern "C" fn() -> c_int>,
    pub Language_UsesSpaces: Option<unsafe extern "C" fn() -> c_int>,
    pub AnyLanguage_ReadCharFromString: Option<unsafe extern "C" fn(*const c_char, *mut c_int, *mut c_char) -> c_int>,
    pub SG_GetSaveGameComment: Option<unsafe extern "C" fn(*const c_char, *mut c_char, *mut c_char) -> c_int>,
    pub SG_StoreSaveGameComment: Option<unsafe extern "C" fn(*const c_char)>,
    pub SG_GameAllowedToSaveHere: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub R_ClearScene: Option<unsafe extern "C" fn()>,
    pub R_AddRefEntityToScene: Option<unsafe extern "C" fn(*const c_void)>,
    pub R_AddPolyToScene: Option<unsafe extern "C" fn(*const c_void)>,
    pub R_AddLightToScene: Option<unsafe extern "C" fn(*const c_void)>,
    pub R_RenderScene: Option<unsafe extern "C" fn(*const c_void)>,
    pub R_ModelBounds: Option<unsafe extern "C" fn(c_int, *mut f32, *mut f32)>,
    pub R_SetColor: Option<unsafe extern "C" fn(*const f32)>,
    pub R_DrawStretchPic: Option<unsafe extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, c_int)>,
    pub UpdateScreen: Option<unsafe extern "C" fn()>,
    pub PrecacheScreenshot: Option<unsafe extern "C" fn()>,
    pub R_LerpTag: Option<unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, f32, *mut c_void) -> c_int>,
    pub S_StartLocalLoopingSound: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub S_StartLocalSound: Option<unsafe extern "C" fn(c_int, c_int)>,
    pub S_RegisterSound: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub Key_KeynumToStringBuf: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
    pub Key_GetBindingBuf: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
    pub Key_SetBinding: Option<unsafe extern "C" fn(c_int, *const c_char)>,
    pub Key_IsDown: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub Key_GetOverstrikeMode: Option<unsafe extern "C" fn() -> c_int>,
    pub Key_SetOverstrikeMode: Option<unsafe extern "C" fn(c_int)>,
    pub Key_ClearStates: Option<unsafe extern "C" fn()>,
    pub Key_GetCatcher: Option<unsafe extern "C" fn() -> c_int>,
    pub Key_SetCatcher: Option<unsafe extern "C" fn(c_int)>,
    pub GetClipboardData: Option<unsafe extern "C" fn(*mut c_char, c_int)>,
    pub GetClientState: Option<unsafe extern "C" fn() -> c_int>,
    pub GetGlconfig: Option<unsafe extern "C" fn(*mut glconfig_t)>,
    pub GetConfigString: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
    pub Milliseconds: Option<unsafe extern "C" fn() -> c_int>,
}

#[repr(C)]
pub struct vmCvar_t {
    // placeholder
}

#[repr(C)]
pub struct cvar_t {
    // placeholder
}

#[repr(C)]
pub struct pc_token_s {
    // placeholder
}

#[repr(C)]
pub struct vm_t {
    // placeholder - zero-initialize
}

// Type aliases and constants
pub type connstate_t = c_int;

const MAX_CONFIGSTRINGS: c_int = 4096;
const KEYCATCH_UI: c_int = 1;
const DP_HUD: c_int = 0;
const DP_OBJECTIVES: c_int = 1;
const DP_WEAPONS: c_int = 2;
const DP_INVENTORY: c_int = 3;
const DP_FORCEPOWERS: c_int = 4;
const CA_DISCONNECTED: c_int = 0;
const CA_ACTIVE: c_int = 4;
const UI_API_VERSION: c_int = 6;
const UI_ERROR: c_int = 1;
const UI_CVAR_REGISTER: c_int = 2;
const UI_CVAR_SET: c_int = 3;
const UI_CVAR_SETVALUE: c_int = 4;
const UI_CVAR_UPDATE: c_int = 5;
const UI_R_REGISTERMODEL: c_int = 6;
const UI_R_REGISTERSHADERNOMIP: c_int = 7;
const UI_GETGLCONFIG: c_int = 8;
const UI_CMD_EXECUTETEXT: c_int = 9;
const UI_CVAR_VARIABLEVALUE: c_int = 10;
const UI_FS_GETFILELIST: c_int = 11;
const UI_KEY_SETCATCHER: c_int = 12;
const UI_KEY_CLEARSTATES: c_int = 13;
const UI_R_SETCOLOR: c_int = 14;
const UI_R_DRAWSTRETCHPIC: c_int = 15;
const UI_CVAR_VARIABLESTRINGBUFFER: c_int = 16;
const UI_R_MODELBOUNDS: c_int = 17;
const UI_R_CLEARSCENE: c_int = 18;
const UI_KEY_GETCATCHER: c_int = 19;
const UI_MILLISECONDS: c_int = 20;
const UI_S_REGISTERSOUND: c_int = 21;
const UI_S_STARTLOCALSOUND: c_int = 22;
const UI_CIN_PLAYCINEMATIC: c_int = 23;
const UI_CIN_STOPCINEMATIC: c_int = 24;
const UI_CIN_RUNCINEMATIC: c_int = 25;
const UI_CIN_DRAWCINEMATIC: c_int = 26;
const UI_KEY_SETBINDING: c_int = 27;
const UI_KEY_KEYNUMTOSTRINGBUF: c_int = 28;
const UI_CIN_SETEXTENTS: c_int = 29;
const UI_KEY_GETBINDINGBUF: c_int = 30;
const ERR_DROP: c_int = 1;
const CG_DRAW_DATAPAD_HUD: c_int = 1;
const CG_DRAW_DATAPAD_OBJECTIVES: c_int = 2;
const CG_DRAW_DATAPAD_WEAPONS: c_int = 3;
const CG_DRAW_DATAPAD_INVENTORY: c_int = 4;
const CG_DRAW_DATAPAD_FORCEPOWERS: c_int = 5;
