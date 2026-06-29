// Filename:-	sv_savegame.cpp
//
// leave this as first line for PCH reasons...
//

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types, dead_code,
         unused_mut, unused_variables, unused_assignments, clippy::all)]

use crate::code::server::exe_headers_h::*;
use crate::code::server::server_h::*;
use crate::code::game::statindex_h::*;
use crate::code::game::weapons_h::*;
use crate::code::game::g_items_h::*;
use core::ffi::{c_char, c_int, c_long, c_ulong, c_uint, c_void};
use core::ptr::{addr_of, addr_of_mut};

// a little naughty, since these are in the renderer, but I need access to them for savegames, so...
//
extern "C" {
    pub fn Decompress_JPG(
        filename: *const c_char,
        pJPGData: *mut byte,
        pic: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
    );
    pub fn Compress_JPG(
        pOutputSize: *mut c_int,
        quality: c_int,
        image_width: c_int,
        image_height: c_int,
        image_buffer: *mut byte,
        bInvertDuringCompression: qboolean,
    ) -> *mut byte;
}

const JPEG_IMAGE_QUALITY: c_int = 95;

// //#define USE_LAST_SAVE_FROM_THIS_MAP	// enable this if you want to use the last explicity-loaded savegame from this map
// 					 					//	when respawning after dying, else it'll just load "auto" regardless
// 										//	(EF1 behaviour). I should maybe time/date check them though?

// #pragma warning(disable : 4786)  // identifier was truncated (STL crap)
// #pragma warning(disable : 4710)  // function was not inlined (STL crap)
// #pragma warning(disable : 4512)  // yet more STL drivel...

// using namespace std;  (C++ STL — no Rust equivalent needed)

// #ifdef _XBOX
// #include <stdlib.h>
// //support for mbstowcs
#[cfg(feature = "xbox")]
pub static mut sg_Handle: HANDLE = 0 as HANDLE;
#[cfg(feature = "xbox")]
const SG_BUFFERSIZE: usize = 8192;
#[cfg(feature = "xbox")]
pub static mut sg_Buffer: [byte; 8192 /* SG_BUFFERSIZE */] = [0; 8192];
#[cfg(feature = "xbox")]
pub static mut sg_BufferSize: c_int = 0;
// //used for save game reading
#[cfg(feature = "xbox")]
pub static mut sg_CurrentBufferPos: c_int = 0;

// #define filepathlength 120
#[cfg(feature = "xbox")]
const filepathlength: usize = 120;

// struct XValidationHeader
// {
//     // Length of the file, including header, in bytes
//     DWORD dwFileLength;
//
//     // File signature (secure hash of file data)
//     XCALCSIG_SIGNATURE Signature;
// };
#[cfg(feature = "xbox")]
#[repr(C)]
pub struct XValidationHeader {
    // Length of the file, including header, in bytes
    pub dwFileLength: u32, /* DWORD */
    // File signature (secure hash of file data)
    pub Signature: XCALCSIG_SIGNATURE,
}

// //validation header going into file and coming out of file
#[cfg(feature = "xbox")]
pub static mut sg_validationHeader: XValidationHeader = XValidationHeader {
    dwFileLength: 0,
    Signature: unsafe { core::mem::zeroed() },
};
// //validation header calculated on file read to test against file
#[cfg(feature = "xbox")]
pub static mut sg_validationHeaderRead: XValidationHeader = XValidationHeader {
    dwFileLength: 0,
    Signature: unsafe { core::mem::zeroed() },
};

// //signature handle
#[cfg(feature = "xbox")]
pub static mut sg_sigHandle: HANDLE = 0 as HANDLE;
// #endif

// //#define SG_PROFILE	// enable for debug save stats if you want

// #ifdef SG_PROFILE
#[cfg(feature = "sg_profile")]
struct CChid {
    m_iCount: c_int,
    m_iSize: c_int,
}

#[cfg(feature = "sg_profile")]
impl CChid {
    fn new() -> Self {
        CChid {
            m_iCount: 0,
            m_iSize: 0,
        }
    }
    fn Add(&mut self, iLength: c_int) {
        self.m_iCount += 1;
        self.m_iSize += iLength;
    }
    fn GetCount(&self) -> c_int {
        self.m_iCount
    }
    fn GetSize(&self) -> c_int {
        self.m_iSize
    }
}

// typedef map<unsigned long, CChid> CChidInfo_t;
#[cfg(feature = "sg_profile")]
static mut save_info: Option<std::collections::BTreeMap<c_ulong, CChid>> = None;
// #endif

static mut saveGameComment: [c_char; iSG_COMMENT_SIZE] = [0; iSG_COMMENT_SIZE];

pub static mut giSaveGameVersion: c_int = 0; // filled in when a savegame file is opened
pub static mut fhSaveGame: fileHandle_t = 0;
pub static mut eSavedGameJustLoaded: SavedGameJustLoaded_e = eNO;
pub static mut qbSGReadIsTestOnly: qboolean = qfalse; // this MUST be left in this state
pub static mut sLastSaveFileLoaded: [c_char; MAX_QPATH] = [0; MAX_QPATH];

// #define iSG_MAPCMD_SIZE MAX_QPATH
const iSG_MAPCMD_SIZE: usize = MAX_QPATH as usize;

// #ifndef LPCSTR
type LPCSTR = *const c_char;
// #endif

// type time_t: on Windows (MSVC target), time_t is 64-bit
type time_t = i64;

extern "C" {
    // from <time.h>
    fn time(tloc: *mut time_t) -> time_t;

    // write all CVAR_SAVEGAME cvars — I know this is really unpleasant, but I need access for scanning/writing latched cvars during save games
    pub static mut cvar_vars: *mut cvar_t;

    // from sv_ccmds.cpp: used in SV_LoadGame_f and SG_ReadSavegame
    pub static mut qbLoadTransition: qboolean;

    // from sv_player.cpp / game module
    fn SV_Player_EndOfLevelSave();

    // from scr_scrn.cpp (xbox only)
    #[cfg(feature = "xbox")]
    fn SCR_PrecacheScreenshot();
}

pub unsafe fn SG_GetChidText(chid: c_ulong) -> LPCSTR {
    static mut chidtext: [c_char; 5] = [0; 5];
    // Write chid as big-endian bytes into chidtext so it prints as the 4-char tag
    *(addr_of_mut!(chidtext) as *mut c_ulong) = BigLong(chid);
    (*addr_of_mut!(chidtext))[4] = 0;
    addr_of!(chidtext) as LPCSTR
}

unsafe fn GetString_FailedToOpenSaveGame(
    psFilename: *const c_char,
    bOpen: qboolean,
) -> *const c_char {
    static mut sTemp: [c_char; 256] = [0; 256];

    strcpy(addr_of_mut!(sTemp) as *mut c_char, S_COLOR_RED);

    let psReference: *const c_char = if bOpen != qfalse {
        b"MENUS_FAILED_TO_OPEN_SAVEGAME\0".as_ptr() as *const c_char
    } else {
        b"MENUS3_FAILED_TO_CREATE_SAVEGAME\0".as_ptr() as *const c_char
    };
    Q_strncpyz(
        (addr_of_mut!(sTemp) as *mut c_char).add(strlen(addr_of!(sTemp) as *const c_char)),
        va(SE_GetString(psReference), psFilename),
        core::mem::size_of::<[c_char; 256]>() as c_int,
    );
    strcat(addr_of_mut!(sTemp) as *mut c_char, b"\n\0".as_ptr() as *const c_char);
    addr_of!(sTemp) as *const c_char
}

// (copes with up to 8 ptr returns at once)
//
unsafe fn SG_AddSavePath(psPathlessBaseName: LPCSTR) -> LPCSTR {
    static mut sSaveName: [[c_char; MAX_OSPATH as usize]; 8] =
        [[0; MAX_OSPATH as usize]; 8];
    static mut i: c_int = 0;

    *addr_of_mut!(i) = (*addr_of!(i) + 1) & 7;
    let idx = *addr_of!(i) as usize;

    if !psPathlessBaseName.is_null() {
        let mut p: *mut c_char = strchr(psPathlessBaseName, b'/' as c_int);
        if !p.is_null() {
            while !p.is_null() {
                *p = b'_' as c_char;
                p = strchr(p, b'/' as c_int);
            }
        }
    }
    Com_sprintf(
        (*addr_of_mut!(sSaveName))[idx].as_mut_ptr(),
        MAX_OSPATH,
        b"saves/%s.sav\0".as_ptr() as *const c_char,
        psPathlessBaseName,
    );
    (*addr_of!(sSaveName))[idx].as_ptr()
}

pub unsafe fn SG_WipeSavegame(psPathlessBaseName: LPCSTR) {
    #[cfg(not(feature = "xbox"))]
    {
        let psLocalFilename: LPCSTR = SG_AddSavePath(psPathlessBaseName);
        FS_DeleteUserGenFile(psLocalFilename);
    }
    #[cfg(feature = "xbox")]
    {
        let mut namebuffer: [u16; filepathlength] = [0; filepathlength];
        mbstowcs(namebuffer.as_mut_ptr(), psPathlessBaseName, filepathlength);
        // kill the whole directory
        // remove it
        XDeleteSaveGame(b"U:\\\0".as_ptr() as *const c_char, namebuffer.as_ptr());
    }
}

unsafe fn SG_Move(
    psPathlessBaseName_Src: LPCSTR,
    psPathlessBaseName_Dst: LPCSTR,
) -> qboolean {
    #[cfg(not(feature = "xbox"))]
    {
        let psLocalFilename_Src: LPCSTR = SG_AddSavePath(psPathlessBaseName_Src);
        let psLocalFilename_Dst: LPCSTR = SG_AddSavePath(psPathlessBaseName_Dst);

        let qbCopyWentOk: qboolean = FS_MoveUserGenFile(psLocalFilename_Src, psLocalFilename_Dst);

        if qbCopyWentOk == qfalse {
            Com_Printf(
                b"%sError during savegame-rename. Check \"%s\" for write-protect or disk full!\n\0"
                    .as_ptr() as *const c_char,
                S_COLOR_RED,
                psLocalFilename_Dst,
            );
            return qfalse;
        }

        return qtrue;
    }
    #[cfg(feature = "xbox")]
    {
        let mut psLocalFilenameSrc: [c_char; filepathlength] = [0; filepathlength];
        let mut psLocalFilenameDest: [c_char; filepathlength] = [0; filepathlength];
        let mut widecharstring: [u16; filepathlength] = [0; filepathlength];
        mbstowcs(widecharstring.as_mut_ptr(), psPathlessBaseName_Dst, filepathlength);

        if ERROR_SUCCESS
            != XCreateSaveGame(
                b"U:\\\0".as_ptr() as *const c_char,
                widecharstring.as_ptr(),
                OPEN_ALWAYS,
                0,
                psLocalFilenameDest.as_mut_ptr(),
                filepathlength,
            )
        {
            return qfalse;
        }
        mbstowcs(widecharstring.as_mut_ptr(), psPathlessBaseName_Src, filepathlength);
        if ERROR_SUCCESS
            != XCreateSaveGame(
                b"U:\\\0".as_ptr() as *const c_char,
                widecharstring.as_ptr(),
                OPEN_ALWAYS,
                0,
                psLocalFilenameSrc.as_mut_ptr(),
                filepathlength,
            )
        {
            return qfalse;
        }

        Q_strcat(
            psLocalFilenameDest.as_mut_ptr(),
            filepathlength as c_int,
            b"JK3SG.xsv\0".as_ptr() as *const c_char,
        );
        Q_strcat(
            psLocalFilenameSrc.as_mut_ptr(),
            filepathlength as c_int,
            b"JK3SG.xsv\0".as_ptr() as *const c_char,
        );

        CopyFile(
            psLocalFilenameSrc.as_ptr(),
            psLocalFilenameDest.as_ptr(),
            false as i32,
        );

        return qtrue;
    }
}

/* JLFSAVEGAME used to find if there is a file on the xbox */
#[cfg(feature = "xbox")]
pub unsafe fn SG_Exists(psPathlessBaseName: LPCSTR) -> qboolean {
    let mut psLocalFilename: [c_char; filepathlength] = [0; filepathlength];
    let mut widecharstring: [u16; filepathlength] = [0; filepathlength];
    mbstowcs(widecharstring.as_mut_ptr(), psPathlessBaseName, filepathlength);
    if ERROR_SUCCESS
        != XCreateSaveGame(
            b"U:\\\0".as_ptr() as *const c_char,
            widecharstring.as_ptr(),
            CREATE_NEW,
            0,
            psLocalFilename.as_mut_ptr(),
            filepathlength,
        )
    {
        return qtrue;
    }
    if ERROR_SUCCESS
        == XDeleteSaveGame(
            b"U:\\\0".as_ptr() as *const c_char,
            widecharstring.as_ptr(),
        )
    {
        return qfalse;
    }
    assert!(false);
    qfalse
}

pub static mut gbSGWriteFailed: qboolean = qfalse;

unsafe fn SG_Create(psPathlessBaseName: LPCSTR) -> qboolean {
    *addr_of_mut!(gbSGWriteFailed) = qfalse;

    #[cfg(feature = "xbox")]
    {
        let mut psLocalFilename: [c_char; filepathlength] = [0; filepathlength];
        let mut psScreenshotFilename: [c_char; filepathlength] = [0; filepathlength];
        let mut widecharstring: [u16; filepathlength] = [0; filepathlength];
        mbstowcs(widecharstring.as_mut_ptr(), psPathlessBaseName, filepathlength);
        if ERROR_SUCCESS
            != XCreateSaveGame(
                b"U:\\\0".as_ptr() as *const c_char,
                widecharstring.as_ptr(),
                OPEN_ALWAYS,
                0,
                psLocalFilename.as_mut_ptr(),
                filepathlength,
            )
        {
            return qfalse;
        }

        // create the path for the screenshot file
        strcpy(
            psScreenshotFilename.as_mut_ptr(),
            psLocalFilename.as_ptr(),
        );
        Q_strcat(
            psScreenshotFilename.as_mut_ptr(),
            filepathlength as c_int,
            b"saveimage.xbx\0".as_ptr() as *const c_char,
        );

        // create the path for the savegame
        Q_strcat(
            psLocalFilename.as_mut_ptr(),
            filepathlength as c_int,
            b"JK3SG.xsv\0".as_ptr() as *const c_char,
        );

        *addr_of_mut!(sg_Handle) = CreateFile(
            psLocalFilename.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ,
            0 as *mut core::ffi::c_void,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            0 as HANDLE,
        );
        // clear the buffer
        *addr_of_mut!(sg_BufferSize) = 0;

        let mut bytesWritten: u32 /* DWORD */ = 0;
        // save spot for validation
        WriteFile(
            *addr_of!(sg_Handle),
            addr_of!(sg_validationHeader) as *const c_void,
            core::mem::size_of::<XValidationHeader>() as u32,
            &mut bytesWritten,
            core::ptr::null_mut(),
        );
        // start the validation key creation
        // Start the signature hash
        *addr_of_mut!(sg_sigHandle) = XCalculateSignatureBegin(0);
        if *addr_of!(sg_sigHandle) == INVALID_HANDLE_VALUE {
            return qfalse; // returning FALSE as qboolean
        }

        // attempt to copy the last screenshot to the save game directory
        if CopyFile(
            b"u:\\saveimage.xbx\0".as_ptr() as *const c_char,
            psScreenshotFilename.as_ptr(),
            0, /* FALSE */
        ) == 0
        {
            CopyFile(
                b"d:\\base\\media\\defaultsaveimage.xbx\0".as_ptr() as *const c_char,
                psScreenshotFilename.as_ptr(),
                0, /* FALSE */
            );
        }

        if *addr_of!(sg_Handle) == 0 as HANDLE {
            Com_Printf(GetString_FailedToOpenSaveGame(psLocalFilename.as_ptr(), qfalse));
            // S_COLOR_RED "Failed to create new savegame file \"%s\"\n", psLocalFilename
            return qfalse;
        }
    }
    #[cfg(not(feature = "xbox"))]
    {
        SG_WipeSavegame(psPathlessBaseName);
        let psLocalFilename: LPCSTR = SG_AddSavePath(psPathlessBaseName);
        *addr_of_mut!(fhSaveGame) = FS_FOpenFileWrite(psLocalFilename);

        if *addr_of!(fhSaveGame) == 0 {
            Com_Printf(GetString_FailedToOpenSaveGame(psLocalFilename, qfalse));
            // S_COLOR_RED "Failed to create new savegame file \"%s\"\n", psLocalFilename
            return qfalse;
        }
    }

    #[cfg(feature = "sg_profile")]
    {
        assert!(get_save_info().is_empty());
    }

    *addr_of_mut!(giSaveGameVersion) = iSAVEGAME_VERSION;
    SG_Append(
        0x5F564552 as c_ulong, /* '_VER' */
        addr_of_mut!(giSaveGameVersion) as *const c_void,
        core::mem::size_of::<c_int>() as c_int,
    );

    qtrue
}

// called from the ERR_DROP stuff just in case the error occured during loading of a saved game, because if
//	we didn't do this then we'd run out of quake file handles after the 8th load fail...
//
pub unsafe fn SG_Shutdown() {
    if *addr_of!(fhSaveGame) != 0 {
        FS_FCloseFile(*addr_of!(fhSaveGame));
        *addr_of_mut!(fhSaveGame) = 0; // NULL
    }

    *addr_of_mut!(eSavedGameJustLoaded) = eNO; // important to do this if we ERR_DROP during loading, else next map you load after
                                               //	a bad save-file you'll arrive at dead :-)

    // and this bit stops people messing up the laoder by repeatedly stabbing at the load key during loads...
    //
    // extern qboolean gbAlreadyDoingLoad; (defined in this file at module level)
    *addr_of_mut!(gbAlreadyDoingLoad) = qfalse;
}

#[cfg(feature = "xbox")]
pub unsafe fn SG_CloseWrite() -> qboolean {
    let mut bytesWritten: u32 /* DWORD */ = 0;
    // clear the buffer to the file
    if WriteFile(
        *addr_of!(sg_Handle),
        addr_of!(sg_Buffer) as *const c_void,
        *addr_of!(sg_BufferSize) as u32,
        &mut bytesWritten,
        core::ptr::null_mut(),
    ) == 0
    {
        return qfalse;
    }
    // get the length of the file
    let filelength: u32 = GetFileSize(*addr_of!(sg_Handle), core::ptr::null_mut());
    // create the validation code
    (*addr_of_mut!(sg_validationHeader)).dwFileLength = filelength;
    // Release signature resources
    let dwSuccess: u32 /* DWORD */ =
        XCalculateSignatureEnd(*addr_of!(sg_sigHandle), addr_of_mut!((*addr_of_mut!(sg_validationHeader)).Signature));
    assert!(dwSuccess == ERROR_SUCCESS);
    // seek to the first of the file
    SG_Seek(0 as fileHandle_t, 0, FS_SEEK_SET);
    // SetFilePointer(sg_Handle,0,0,FILE_BEGIN);
    // write the validation codes
    WriteFile(
        *addr_of!(sg_Handle),
        addr_of!(sg_validationHeader) as *const c_void,
        core::mem::size_of::<XValidationHeader>() as u32,
        &mut bytesWritten,
        core::ptr::null_mut(),
    );
    SG_Close()
}

pub unsafe fn SG_Close() -> qboolean {
    #[cfg(feature = "xbox")]
    {
        CloseHandle(*addr_of!(sg_Handle));
        *addr_of_mut!(sg_Handle) = 0 as HANDLE; // NULL
    }
    #[cfg(not(feature = "xbox"))]
    {
        assert!(*addr_of!(fhSaveGame) != 0);
        FS_FCloseFile(*addr_of!(fhSaveGame));
    }
    *addr_of_mut!(fhSaveGame) = 0; // NULL

    #[cfg(feature = "sg_profile")]
    {
        if (*sv_testsave).integer == 0 {
            let mut iCount: c_int = 0;
            let mut iSize: c_int = 0;

            Com_DPrintf(
                b"%s================================\n\0".as_ptr() as *const c_char,
                S_COLOR_CYAN,
            );
            Com_DPrintf(
                b"%sCHID   Count      Size\n\n\0".as_ptr() as *const c_char,
                S_COLOR_WHITE,
            );
            for (k, v) in get_save_info().iter() {
                Com_DPrintf(
                    b"%s   %5d  %8d\n\0".as_ptr() as *const c_char,
                    SG_GetChidText(*k),
                    v.GetCount(),
                    v.GetSize(),
                );
                iCount += v.GetCount();
                iSize += v.GetSize();
            }
            Com_DPrintf(
                b"\n%s%d chunks making %d bytes\n\0".as_ptr() as *const c_char,
                S_COLOR_WHITE,
                iCount,
                iSize,
            );
            Com_DPrintf(
                b"%s================================\n\0".as_ptr() as *const c_char,
                S_COLOR_CYAN,
            );
            get_save_info().clear();
        }
    }

    CompressMem_FreeScratchBuffer();
    qtrue
}

pub unsafe fn SG_Open(psPathlessBaseName: LPCSTR) -> qboolean {
    //	if ( fhSaveGame )		// hmmm...
    //	{						//
    //		SG_Close();			//
    //	}						//
    assert!(*addr_of!(fhSaveGame) == 0); // I'd rather know about this
    if psPathlessBaseName.is_null() {
        return qfalse;
    }
    // JLFSAVEGAME

    #[cfg(feature = "xbox")]
    {
        let mut saveGameName: [u16; filepathlength] = [0; filepathlength];
        let mut directoryInfo: [c_char; filepathlength] = [0; filepathlength];
        let mut psLocalFilename: [c_char; filepathlength] = [0; filepathlength];
        let mut bytesRead: u32 /* DWORD */ = 0;

        mbstowcs(saveGameName.as_mut_ptr(), psPathlessBaseName, filepathlength);

        XCreateSaveGame(
            b"U:\\\0".as_ptr() as *const c_char,
            saveGameName.as_ptr(),
            OPEN_ALWAYS,
            0,
            directoryInfo.as_mut_ptr(),
            filepathlength,
        );

        strcpy(psLocalFilename.as_mut_ptr(), directoryInfo.as_ptr());
        strcat(
            psLocalFilename.as_mut_ptr(),
            b"JK3SG.xsv\0".as_ptr() as *const c_char,
        );

        *addr_of_mut!(sg_Handle) = 0 as HANDLE; // NULL
        *addr_of_mut!(sg_Handle) = CreateFile(
            psLocalFilename.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            0 as *mut c_void,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            0 as HANDLE,
        );

        if *addr_of!(sg_Handle) == 0 as HANDLE {
            //		Com_Printf(S_COLOR_RED "Failed to open savegame file %s\n", psLocalFilename);
            Com_DPrintf(GetString_FailedToOpenSaveGame(psLocalFilename.as_ptr(), qtrue));
            return qfalse;
        }

        // read the validation header
        if ReadFile(
            *addr_of!(sg_Handle),
            addr_of_mut!(sg_validationHeader) as *mut c_void,
            core::mem::size_of::<XValidationHeader>() as u32,
            &mut bytesRead,
            core::ptr::null_mut(),
        ) == 0
        {
            SG_Close();
            Com_Printf(
                b"%sFile \"%s\" has no sig\0".as_ptr() as *const c_char,
                S_COLOR_RED,
            );
            return qfalse;
        }
        // initialize buffer data
        *addr_of_mut!(sg_BufferSize) = 0;
        *addr_of_mut!(sg_CurrentBufferPos) = 0;
    }
    #[cfg(not(feature = "xbox"))]
    {
        let psLocalFilename: LPCSTR = SG_AddSavePath(psPathlessBaseName);
        FS_FOpenFileRead(psLocalFilename, addr_of_mut!(fhSaveGame), qtrue); // qtrue = dup handle, so I can close it ok later
        if *addr_of!(fhSaveGame) == 0 {
            //		Com_Printf(S_COLOR_RED "Failed to open savegame file %s\n", psLocalFilename);
            Com_DPrintf(GetString_FailedToOpenSaveGame(psLocalFilename, qtrue));
            return qfalse;
        }
    }

    *addr_of_mut!(giSaveGameVersion) = -1; // jic
    SG_Read(
        0x5F564552 as c_ulong, /* '_VER' */
        addr_of_mut!(giSaveGameVersion) as *mut c_void,
        core::mem::size_of::<c_int>() as c_int,
        core::ptr::null_mut(),
    );
    if *addr_of!(giSaveGameVersion) != iSAVEGAME_VERSION {
        SG_Close();
        Com_Printf(
            b"%sFile \"%s\" has version # %d (expecting %d)\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
            psPathlessBaseName,
            *addr_of!(giSaveGameVersion),
            iSAVEGAME_VERSION,
        );
        return qfalse;
    }

    qtrue
}

// you should only call this when you know you've successfully opened a savegame, and you want to query for
//	whether it's an old (street-copy) version, or a new (expansion-pack) version
//
pub unsafe fn SG_Version() -> c_int {
    *addr_of!(giSaveGameVersion)
}

pub unsafe fn SV_WipeGame_f() {
    if Cmd_Argc() != 2 {
        Com_Printf(
            b"%sUSAGE: wipe <name>\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
        );
        return;
    }
    if stricmp(Cmd_Argv(1), b"auto\0".as_ptr() as *const c_char) == 0 {
        Com_Printf(
            b"%sCan't wipe 'auto'\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
        );
        return;
    }
    SG_WipeSavegame(Cmd_Argv(1));
    //	Com_Printf("%s has been wiped\n", Cmd_Argv(1));	// wurde gel�scht in german, but we've only got one string
    //	Com_Printf("Ok\n"); // no localization of this
}

/*
// Store given string in saveGameComment for later use when game is
// actually saved
*/
pub unsafe fn SG_StoreSaveGameComment(sComment: *const c_char) {
    memmove(
        addr_of_mut!(saveGameComment) as *mut c_void,
        sComment as *const c_void,
        iSG_COMMENT_SIZE,
    );
}

pub unsafe fn SV_TryLoadTransition(mapname: *const c_char) -> qboolean {
    let psFilename: *mut c_char = va(b"hub/%s\0".as_ptr() as *const c_char, mapname);

    Com_Printf(
        b"%sRestoring game \"%s\"...\n\0".as_ptr() as *const c_char,
        S_COLOR_CYAN,
        psFilename,
    );

    if SG_ReadSavegame(psFilename) == qfalse {
        // couldn't load a savegame
        return qfalse;
    }
    Com_Printf(
        b"%s%s.\n\0".as_ptr() as *const c_char,
        S_COLOR_CYAN,
        SE_GetString(b"MENUS_DONE\0".as_ptr() as *const c_char),
    );

    qtrue
}

pub static mut gbAlreadyDoingLoad: qboolean = qfalse;

pub unsafe fn SV_LoadGame_f() {
    if *addr_of!(gbAlreadyDoingLoad) != qfalse {
        Com_DPrintf(
            b"( Already loading, ignoring extra 'load' commands... )\n\0".as_ptr() as *const c_char,
        );
        return;
    }

    //	// check server is running
    //	//
    //	if ( !com_sv_running->integer )
    //	{
    //		Com_Printf( "Server is not running\n" );
    //		return;
    //	}

    if Cmd_Argc() != 2 {
        Com_Printf(b"USAGE: load <filename>\n\0".as_ptr() as *const c_char);
        return;
    }

    let psFilename: *const c_char = Cmd_Argv(1);
    if !strstr(psFilename, b"..\0".as_ptr() as *const c_char).is_null()
        || !strstr(psFilename, b"/\0".as_ptr() as *const c_char).is_null()
        || !strstr(psFilename, b"\\\0".as_ptr() as *const c_char).is_null()
    {
        Com_Printf(
            b"%sBad loadgame name.\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
        );
        return;
    }

    #[cfg(not(feature = "xbox"))] // VVFIXME : Part of super-bootleg SG hackery
    {
        if stricmp(psFilename, b"current\0".as_ptr() as *const c_char) == 0 {
            Com_Printf(
                b"%sCan't load from \"current\"\n\0".as_ptr() as *const c_char,
                S_COLOR_RED,
            );
            return;
        }
    }

    // special case, if doing a respawn then check that the available auto-save (if any) is from the same map
    //	as we're currently on (if in a map at all), if so, load that "auto", else re-load the last-loaded file...
    //
    let mut psFilename: *const c_char = psFilename; // rebind as mutable for reassignment
    if stricmp(psFilename, b"*respawn\0".as_ptr() as *const c_char) == 0 {
        psFilename = b"auto\0".as_ptr() as *const c_char; // default to standard respawn behaviour

        // see if there's a last-loaded file to even check against as regards loading...
        //
        if *addr_of!(sLastSaveFileLoaded)[0] != 0 {
            let psServerInfo: LPCSTR = (*addr_of!(sv)).configstrings[CS_SERVERINFO as usize];
            let psMapName: LPCSTR =
                Info_ValueForKey(psServerInfo, b"mapname\0".as_ptr() as *const c_char);

            let psMapNameOfAutoSave: *mut c_char =
                SG_GetSaveGameMapName(b"auto\0".as_ptr() as *const c_char);

            if Q_stricmp(psMapName, b"_brig\0".as_ptr() as *const c_char) == 0 {
                // if you're in the brig and there is no autosave, load the last loaded savegame
                if psMapNameOfAutoSave.is_null() {
                    psFilename = addr_of!(sLastSaveFileLoaded) as *const c_char;
                }
            } else {
                #[cfg(feature = "use_last_save_from_this_map")]
                {
                    // if the map name within the name of the last save file we explicitly loaded is the same
                    //	as the current map, then use that...
                    //
                    let psMapNameOfLastSaveFileLoaded: *mut c_char =
                        SG_GetSaveGameMapName(addr_of!(sLastSaveFileLoaded) as *const c_char);

                    if Q_stricmp(psMapName, psMapNameOfLastSaveFileLoaded) == 0 {
                        psFilename = addr_of!(sLastSaveFileLoaded) as *const c_char;
                    } else if !(psMapName != core::ptr::null()
                        && psMapNameOfAutoSave != core::ptr::null_mut()
                        && Q_stricmp(psMapName, psMapNameOfAutoSave) == 0)
                    {
                        // either there's no auto file, or it's from a different map to the one we've just died on...
                        //
                        psFilename = addr_of!(sLastSaveFileLoaded) as *const c_char;
                    }
                }
                #[cfg(not(feature = "use_last_save_from_this_map"))]
                {
                    if !(psMapName != core::ptr::null()
                        && psMapNameOfAutoSave != core::ptr::null_mut()
                        && Q_stricmp(psMapName, psMapNameOfAutoSave) == 0)
                    {
                        // either there's no auto file, or it's from a different map to the one we've just died on...
                        //
                        psFilename = addr_of!(sLastSaveFileLoaded) as *const c_char;
                    }
                }
            }
        }
        // default will continue to load auto
    }
    Com_Printf(
        b"%s%s\n\0".as_ptr() as *const c_char,
        S_COLOR_CYAN,
        va(
            SE_GetString(b"MENUS_LOADING_MAPNAME\0".as_ptr() as *const c_char),
            psFilename,
        ),
    );

    *addr_of_mut!(gbAlreadyDoingLoad) = qtrue;
    if SG_ReadSavegame(psFilename) == qfalse {
        *addr_of_mut!(gbAlreadyDoingLoad) = qfalse; //	do NOT do this here now, need to wait until client spawn, unless the load failed.
    } else {
        Com_Printf(
            b"%s%s.\n\0".as_ptr() as *const c_char,
            S_COLOR_CYAN,
            SE_GetString(b"MENUS_DONE\0".as_ptr() as *const c_char),
        );
    }
}

// extern qboolean SG_GameAllowedToSaveHere(qboolean inCamera); (defined below)

// JLF notes
//	save game will be in charge of creating a new directory
pub unsafe fn SV_SaveGame_f() {
    // check server is running
    //
    if (*com_sv_running).integer == 0 {
        Com_Printf(
            b"%sServer is not running\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
        );
        return;
    }

    if (*addr_of!(sv)).state != SS_GAME {
        Com_Printf(
            b"%sYou must be in a game to save.\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
        );
        return;
    }

    // check args...
    //
    if Cmd_Argc() != 2 {
        Com_Printf(b"USAGE: \"save <filename>\"\n\0".as_ptr() as *const c_char);
        return;
    }

    if (*addr_of!(svs))
        .clients[0]
        .frames[(*addr_of!(svs)).clients[0].netchan.outgoingSequence as usize
            & PACKET_MASK as usize]
        .ps
        .stats[STAT_HEALTH as usize]
        <= 0
    {
        Com_Printf(
            b"%s\n%s\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
            SE_GetString(b"SP_INGAME_CANT_SAVE_DEAD\0".as_ptr() as *const c_char),
        );
        return;
    }

    // this check catches deaths even the instant you die, like during a slo-mo death!
    let svent: *mut gentity_t = SV_GentityNum(0);
    if (*(*svent).client).stats[STAT_HEALTH as usize] <= 0 {
        Com_Printf(
            b"%s\n%s\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
            SE_GetString(b"SP_INGAME_CANT_SAVE_DEAD\0".as_ptr() as *const c_char),
        );
        return;
    }

    let psFilename: *mut c_char = Cmd_Argv(1) as *mut c_char;

    if stricmp(psFilename, b"current\0".as_ptr() as *const c_char) == 0 {
        Com_Printf(
            b"%sCan't save to 'current'\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
        );
        return;
    }

    if !strstr(psFilename, b"..\0".as_ptr() as *const c_char).is_null()
        || !strstr(psFilename, b"/\0".as_ptr() as *const c_char).is_null()
        || !strstr(psFilename, b"\\\0".as_ptr() as *const c_char).is_null()
    {
        Com_Printf(
            b"%sBad savegame name.\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
        );
        return;
    }

    if SG_GameAllowedToSaveHere(qfalse) == qfalse {
        // full check
        return; // this prevents people saving via quick-save now during cinematics.
    }

    if stricmp(psFilename, b"auto\0".as_ptr() as *const c_char) == 0 {
        #[cfg(feature = "xbox")]
        {
            // extern void	SCR_PrecacheScreenshot();  //scr_scrn.cpp
            SCR_PrecacheScreenshot();
        }
        SG_StoreSaveGameComment(b"\0".as_ptr() as *const c_char); // clear previous comment/description, which will force time/date comment.
    }

    Com_Printf(
        b"%s%s \"%s\"...\n\0".as_ptr() as *const c_char,
        S_COLOR_CYAN,
        SE_GetString(b"CON_TEXT_SAVING_GAME\0".as_ptr() as *const c_char),
        psFilename,
    );

    if SG_WriteSavegame(psFilename, qfalse) != qfalse {
        Com_Printf(
            b"%s%s.\n\0".as_ptr() as *const c_char,
            S_COLOR_CYAN,
            SE_GetString(b"MENUS_DONE\0".as_ptr() as *const c_char),
        );
    } else {
        Com_Printf(
            b"%s%s.\n\0".as_ptr() as *const c_char,
            S_COLOR_RED,
            SE_GetString(b"MENUS_FAILED_TO_OPEN_SAVEGAME\0".as_ptr() as *const c_char),
        );
    }
}

// ---------------
unsafe fn WriteGame(autosave: qboolean) {
    SG_Append(
        0x47414D45 as c_ulong, /* 'GAME' */
        &autosave as *const qboolean as *const c_void,
        core::mem::size_of::<qboolean>() as c_int,
    );

    if autosave != qfalse {
        // write out player ammo level, health, etc...
        //
        // extern void SV_Player_EndOfLevelSave(void); (declared at top)
        SV_Player_EndOfLevelSave(); // this sets up the various cvars needed, so we can then write them to disk
        //
        let mut s: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];

        // write health/armour etc...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        Cvar_VariableStringBuffer(
            sCVARNAME_PLAYERSAVE,
            s.as_mut_ptr(),
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );
        SG_Append(
            0x43565356 as c_ulong, /* 'CVSV' */
            s.as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );

        // write ammo...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        Cvar_VariableStringBuffer(
            b"playerammo\0".as_ptr() as *const c_char,
            s.as_mut_ptr(),
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );
        SG_Append(
            0x414D4D4F as c_ulong, /* 'AMMO' */
            s.as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );

        // write inventory...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        Cvar_VariableStringBuffer(
            b"playerinv\0".as_ptr() as *const c_char,
            s.as_mut_ptr(),
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );
        SG_Append(
            0x49565459 as c_ulong, /* 'IVTY' */
            s.as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );

        // the new JK2 stuff - force powers, etc...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        Cvar_VariableStringBuffer(
            b"playerfplvl\0".as_ptr() as *const c_char,
            s.as_mut_ptr(),
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );
        SG_Append(
            0x46504C56 as c_ulong, /* 'FPLV' */
            s.as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
        );
    }
}

unsafe fn ReadGame() -> qboolean {
    let mut qbAutoSave: qboolean = qfalse;
    SG_Read(
        0x47414D45 as c_ulong, /* 'GAME' */
        &mut qbAutoSave as *mut qboolean as *mut c_void,
        core::mem::size_of::<qboolean>() as c_int,
        core::ptr::null_mut(),
    );

    if qbAutoSave != qfalse {
        let mut s: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];

        // read health/armour etc...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        SG_Read(
            0x43565356 as c_ulong, /* 'CVSV' */
            s.as_mut_ptr() as *mut c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
            core::ptr::null_mut(),
        );
        Cvar_Set(sCVARNAME_PLAYERSAVE, s.as_ptr());

        // read ammo...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        SG_Read(
            0x414D4D4F as c_ulong, /* 'AMMO' */
            s.as_mut_ptr() as *mut c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
            core::ptr::null_mut(),
        );
        Cvar_Set(b"playerammo\0".as_ptr() as *const c_char, s.as_ptr());

        // read inventory...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        SG_Read(
            0x49565459 as c_ulong, /* 'IVTY' */
            s.as_mut_ptr() as *mut c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
            core::ptr::null_mut(),
        );
        Cvar_Set(b"playerinv\0".as_ptr() as *const c_char, s.as_ptr());

        // read force powers...
        //
        memmove(
            s.as_mut_ptr() as *mut c_void,
            [0u8; MAX_STRING_CHARS as usize].as_ptr() as *const c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>(),
        );
        SG_Read(
            0x46504C56 as c_ulong, /* 'FPLV' */
            s.as_mut_ptr() as *mut c_void,
            core::mem::size_of::<[c_char; MAX_STRING_CHARS as usize]>() as c_int,
            core::ptr::null_mut(),
        );
        Cvar_Set(b"playerfplvl\0".as_ptr() as *const c_char, s.as_ptr());
    }

    qbAutoSave
}

// ---------------  (end of ReadGame)

// write all CVAR_SAVEGAME cvars
// these will be things like model, name, ...
//
// extern  cvar_t	*cvar_vars; (declared at top as extern "C" static)

pub unsafe fn SG_WriteCvars() {
    let mut var: *mut cvar_t;
    let mut iCount: c_int = 0;

    // count the cvars...
    //
    var = *addr_of!(cvar_vars);
    while !var.is_null() {
        if (*var).flags & CVAR_SAVEGAME == 0 {
            var = (*var).next;
            continue;
        }
        iCount += 1;
        var = (*var).next;
    }

    // store count...
    //
    SG_Append(
        0x4356434E as c_ulong, /* 'CVCN' */
        &iCount as *const c_int as *const c_void,
        core::mem::size_of::<c_int>() as c_int,
    );

    // write 'em...
    //
    var = *addr_of!(cvar_vars);
    while !var.is_null() {
        if (*var).flags & CVAR_SAVEGAME == 0 {
            var = (*var).next;
            continue;
        }
        SG_Append(
            0x43564152 as c_ulong, /* 'CVAR' */
            (*var).name as *const c_void,
            strlen((*var).name) as c_int + 1,
        );
        SG_Append(
            0x56414C55 as c_ulong, /* 'VALU' */
            (*var).string as *const c_void,
            strlen((*var).string) as c_int + 1,
        );
        var = (*var).next;
    }
}

pub unsafe fn SG_ReadCvars() {
    let mut iCount: c_int = 0;
    let mut psName: *mut c_char;
    let mut psValue: *mut c_char;

    SG_Read(
        0x4356434E as c_ulong, /* 'CVCN' */
        &mut iCount as *mut c_int as *mut c_void,
        core::mem::size_of::<c_int>() as c_int,
        core::ptr::null_mut(),
    );

    let mut _i: c_int = 0;
    while _i < iCount {
        psName = core::ptr::null_mut();
        psValue = core::ptr::null_mut();
        SG_Read(
            0x43564152 as c_ulong, /* 'CVAR' */
            core::ptr::null_mut(),
            0,
            &mut (psName as *mut c_void) as *mut *mut c_void,
        );
        SG_Read(
            0x56414C55 as c_ulong, /* 'VALU' */
            core::ptr::null_mut(),
            0,
            &mut (psValue as *mut c_void) as *mut *mut c_void,
        );

        Cvar_Set(psName, psValue);

        Z_Free(psName as *mut c_void);
        Z_Free(psValue as *mut c_void);
        _i += 1;
    }
}

pub unsafe fn SG_WriteServerConfigStrings() {
    let mut iCount: c_int = 0;
    let mut i: c_int; // not in FOR statement in case compiler goes weird by reg-optimising it then failing to get the address later

    // count how many non-blank server strings there are...
    //
    i = 0;
    while i < MAX_CONFIGSTRINGS {
        if i != CS_SYSTEMINFO {
            if !(*addr_of!(sv)).configstrings[i as usize].is_null()
                && strlen((*addr_of!(sv)).configstrings[i as usize]) != 0
            {
                // paranoia... <g>
                iCount += 1;
            }
        }
        i += 1;
    }

    SG_Append(
        0x4353434E as c_ulong, /* 'CSCN' */
        &iCount as *const c_int as *const c_void,
        core::mem::size_of::<c_int>() as c_int,
    );

    // now write 'em...
    //
    i = 0;
    while i < MAX_CONFIGSTRINGS {
        if i != CS_SYSTEMINFO {
            if !(*addr_of!(sv)).configstrings[i as usize].is_null()
                && strlen((*addr_of!(sv)).configstrings[i as usize]) != 0
            {
                SG_Append(
                    0x4353494E as c_ulong, /* 'CSIN' */
                    &i as *const c_int as *const c_void,
                    core::mem::size_of::<c_int>() as c_int,
                );
                SG_Append(
                    0x43534441 as c_ulong, /* 'CSDA' */
                    (*addr_of!(sv)).configstrings[i as usize] as *const c_void,
                    strlen((*addr_of!(sv)).configstrings[i as usize]) as c_int + 1,
                );
            }
        }
        i += 1;
    }
}

pub unsafe fn SG_ReadServerConfigStrings() {
    // trash the whole table...
    //
    let mut i: c_int = 0;
    while i < MAX_CONFIGSTRINGS {
        if i != CS_SYSTEMINFO {
            if !(*addr_of!(sv)).configstrings[i as usize].is_null() {
                Z_Free((*addr_of!(sv)).configstrings[i as usize] as *mut c_void);
            }
            (*addr_of_mut!(sv)).configstrings[i as usize] =
                CopyString(b"\0".as_ptr() as *const c_char);
        }
        i += 1;
    }

    // now read the replacement ones...
    //
    let mut iCount: c_int = 0;

    SG_Read(
        0x4353434E as c_ulong, /* 'CSCN' */
        &mut iCount as *mut c_int as *mut c_void,
        core::mem::size_of::<c_int>() as c_int,
        core::ptr::null_mut(),
    );

    Com_DPrintf(
        b"Reading %d configstrings...\n\0".as_ptr() as *const c_char,
        iCount,
    );

    i = 0;
    while i < iCount {
        let mut iIndex: c_int = 0;
        let mut psName: *mut c_char = core::ptr::null_mut();

        SG_Read(
            0x4353494E as c_ulong, /* 'CSIN' */
            &mut iIndex as *mut c_int as *mut c_void,
            core::mem::size_of::<c_int>() as c_int,
            core::ptr::null_mut(),
        );
        SG_Read(
            0x43534441 as c_ulong, /* 'CSDA' */
            core::ptr::null_mut(),
            0,
            &mut (psName as *mut c_void) as *mut *mut c_void,
        );

        Com_DPrintf(
            b"Cfg str %d = %s\n\0".as_ptr() as *const c_char,
            iIndex,
            psName,
        );

        // sv.configstrings[iIndex] = psName;
        SV_SetConfigstring(iIndex, psName);
        Z_Free(psName as *mut c_void);

        i += 1;
    }
}

unsafe fn SG_WriteComment(qbAutosave: qboolean, psMapName: LPCSTR) {
    let mut sComment: [c_char; iSG_COMMENT_SIZE] = [0; iSG_COMMENT_SIZE];

    if qbAutosave != qfalse || *addr_of!(saveGameComment)[0] == 0 {
        Com_sprintf(
            sComment.as_mut_ptr(),
            core::mem::size_of::<[c_char; iSG_COMMENT_SIZE]>() as c_int,
            b"---> %s\0".as_ptr() as *const c_char,
            psMapName,
        );
    } else {
        strcpy(
            sComment.as_mut_ptr(),
            addr_of!(saveGameComment) as *const c_char,
        );
    }

    SG_Append(
        0x434F4D4D as c_ulong, /* 'COMM' */
        sComment.as_ptr() as *const c_void,
        core::mem::size_of::<[c_char; iSG_COMMENT_SIZE]>() as c_int,
    );

    // Add Date/Time/Map stamp
    let mut now: time_t = 0;
    time(&mut now);
    SG_Append(
        0x434D544D as c_ulong, /* 'CMTM' */
        &now as *const time_t as *const c_void,
        core::mem::size_of::<time_t>() as c_int,
    );

    Com_DPrintf(
        b"Saving: current (%s)\n\0".as_ptr() as *const c_char,
        sComment.as_ptr(),
    );
}

// Test to see if the given file name is in the save game directory
// then grab the comment if it's there
//
pub unsafe fn SG_GetSaveGameComment(
    psPathlessBaseName: *const c_char,
    sComment: *mut c_char,
    sMapName: *mut c_char,
) -> c_int {
    let mut ret: c_int = 0;
    let mut tFileTime: time_t = 0;

    *addr_of_mut!(qbSGReadIsTestOnly) = qtrue; // do NOT leave this in this state

    if SG_Open(psPathlessBaseName) == qfalse {
        *addr_of_mut!(qbSGReadIsTestOnly) = qfalse;
        return 0;
    }

    if SG_Read(
        0x434F4D4D as c_ulong, /* 'COMM' */
        sComment as *mut c_void,
        iSG_COMMENT_SIZE as c_int,
        core::ptr::null_mut(),
    ) != 0
    {
        if SG_Read(
            0x434D544D as c_ulong, /* 'CMTM' */
            &mut tFileTime as *mut time_t as *mut c_void,
            core::mem::size_of::<time_t>() as c_int,
            core::ptr::null_mut(),
        ) != 0
        {
            // read
            if SG_Read(
                0x4D50434D as c_ulong, /* 'MPCM' */
                sMapName as *mut c_void,
                iSG_MAPCMD_SIZE as c_int,
                core::ptr::null_mut(),
            ) != 0
            {
                // read
                ret = tFileTime as c_int;
            }
        }
    }
    *addr_of_mut!(qbSGReadIsTestOnly) = qfalse;

    if SG_Close() == qfalse {
        return 0;
    }
    ret
}

// read the mapname field from the supplied savegame file
//
// returns NULL if not found
//
unsafe fn SG_GetSaveGameMapName(psPathlessBaseName: *const c_char) -> *mut c_char {
    static mut sMapName: [c_char; iSG_MAPCMD_SIZE] = [0; iSG_MAPCMD_SIZE];
    let mut psReturn: *mut c_char = core::ptr::null_mut();
    if SG_GetSaveGameComment(
        psPathlessBaseName,
        core::ptr::null_mut(),
        addr_of_mut!(sMapName) as *mut c_char,
    ) != 0
    {
        psReturn = addr_of_mut!(sMapName) as *mut c_char;
    }

    psReturn
}

// pass in qtrue to set as loading screen, else pass in pvDest to read it into there...
//
/*
static qboolean SG_ReadScreenshot(qboolean qbSetAsLoadingScreen, void *pvDest = NULL);
static qboolean SG_ReadScreenshot(qboolean qbSetAsLoadingScreen, void *pvDest)
{
#ifdef _XBOX
	return qfalse;
#else
	qboolean bReturn = qfalse;

	// get JPG screenshot data length...
	//
	int iScreenShotLength = 0;
	SG_Read('SHLN', &iScreenShotLength, sizeof(iScreenShotLength));
	//
	// alloc enough space plus extra 4K for sloppy JPG-decode reader to not do memory access violation...
	//
	byte *pJPGData = (byte *) Z_Malloc(iScreenShotLength + 4096,TAG_TEMP_SAVEGAME_WORKSPACE, qfalse);
	//
	// now read the JPG data...
	//
	SG_Read('SHOT', pJPGData, iScreenShotLength, 0);
	//
	// decompress JPG data...
	//
	byte *pDecompressedPic = NULL;
	int iWidth, iHeight;
	Decompress_JPG( "[savegame]", pJPGData, &pDecompressedPic, &iWidth, &iHeight );
	//
	// if the loaded image is the same size as the game is expecting, then copy it to supplied arg (if present)...
	//
	if (iWidth == SG_SCR_WIDTH && iHeight == SG_SCR_HEIGHT)
	{
		bReturn = qtrue;

		if (pvDest)
		{
			memcpy(pvDest, pDecompressedPic, SG_SCR_WIDTH * SG_SCR_HEIGHT * 4);
		}

		if (qbSetAsLoadingScreen)
		{
			SCR_SetScreenshot((byte *)pDecompressedPic, SG_SCR_WIDTH, SG_SCR_HEIGHT);
		}
	}

	Z_Free( pJPGData );
	Z_Free( pDecompressedPic );

	return bReturn;
#endif
}
// Gets the savegame screenshot
//
qboolean SG_GetSaveImage( const char *psPathlessBaseName, void *pvAddress )
{
	if(!psPathlessBaseName)
	{
		return qfalse;
	}
//JLFSAVEGAME
#if 0
	unsigned short saveGameName[filepathlength];
	char directoryInfo[filepathlength];
	char psLocalFilename[filepathlength];
	DWORD bytesRead;

	mbstowcs(saveGameName, psPathlessBaseName,filepathlength);

	XCreateSaveGame("U:\\", saveGameName, OPEN_ALWAYS, 0,directoryInfo, filepathlength);

	strcpy (psLocalFilename , directoryInfo);
	strcat (psLocalFilename , "saveimage.xbx");


	sg_Handle = NULL;
	sg_Handle = CreateFile(psLocalFilename, GENERIC_READ, FILE_SHARE_READ, 0,
		OPEN_EXISTING,	FILE_ATTRIBUTE_NORMAL, 0);

	if (!sg_Handle)
		return qfalse;



#else

	if (!SG_Open(psPathlessBaseName))
	{
		return qfalse;
	}

	SG_Read('COMM', NULL, 0, NULL);	// skip
	SG_Read('CMTM', NULL, sizeof( time_t ));

	qboolean bGotSaveImage = SG_ReadScreenshot(qfalse, pvAddress);

	SG_Close();
#endif
	return bGotSaveImage;
}


static void SG_WriteScreenshot(qboolean qbAutosave, LPCSTR psMapName)
{
#ifndef _XBOX
	byte *pbRawScreenShot = NULL;

	if( qbAutosave )
	{
		// try to read a levelshot (any valid TGA/JPG etc named the same as the map)...
		//
		int iWidth = SG_SCR_WIDTH;
		int iHeight= SG_SCR_HEIGHT;
		byte	byBlank[SG_SCR_WIDTH * SG_SCR_HEIGHT * 4] = {0};

		pbRawScreenShot = SCR_TempRawImage_ReadFromFile(va("levelshots/%s.tga",psMapName), &iWidth, &iHeight, byBlank, qtrue);	// qtrue = vert flip
	}

	if (!pbRawScreenShot)
	{
		pbRawScreenShot = SCR_GetScreenshot(0);
	}


	int iJPGDataSize = 0;
	byte *pJPGData = Compress_JPG(&iJPGDataSize, JPEG_IMAGE_QUALITY, SG_SCR_WIDTH, SG_SCR_HEIGHT, pbRawScreenShot, qfalse);
	SG_Append('SHLN', &iJPGDataSize, sizeof(iJPGDataSize));
	SG_Append('SHOT', pJPGData, iJPGDataSize);
	Z_Free(pJPGData);
	SCR_TempRawImage_CleanUp();
#endif
}
*/

pub unsafe fn SG_GameAllowedToSaveHere(inCamera: qboolean) -> qboolean {
    if inCamera == qfalse {
        if com_sv_running.is_null() || (*com_sv_running).integer == 0 {
            return qfalse; //		Com_Printf( S_COLOR_RED "Server is not running\n" );
        }

        if CL_IsRunningInGameCinematic() != qfalse {
            return qfalse; // nope, not during a video
        }

        if (*addr_of!(sv)).state != SS_GAME {
            return qfalse; //		Com_Printf (S_COLOR_RED "You must be in a game to save.\n");
        }

        // No savegames from "_" maps
        if sv_mapname.is_null()
            || (!(*sv_mapname).string.is_null() && *(*sv_mapname).string == b'_' as c_char)
        {
            return qfalse; //		Com_Printf (S_COLOR_RED "Cannot save on holodeck or brig.\n");
        }

        if (*addr_of!(svs))
            .clients[0]
            .frames[(*addr_of!(svs)).clients[0].netchan.outgoingSequence as usize
                & PACKET_MASK as usize]
            .ps
            .stats[STAT_HEALTH as usize]
            <= 0
        {
            return qfalse; //		Com_Printf (S_COLOR_RED "\nCan't savegame while dead!\n");
        }
    }
    if ge.is_null() {
        return inCamera; // only happens when called to test if inCamera
    }

    (*ge).GameAllowedToSaveHere()
}

pub unsafe fn SG_WriteSavegame(
    psPathlessBaseName: *const c_char,
    qbAutosave: qboolean,
) -> qboolean {
    if qbAutosave == qfalse && SG_GameAllowedToSaveHere(qfalse) == qfalse {
        // full check
        return qfalse; // this prevents people saving via quick-save now during cinematics
    }

    let iPrevTestSave: c_int = (*sv_testsave).integer;
    (*sv_testsave).integer = 0;

    // Write out server data...
    //
    let psServerInfo: LPCSTR = (*addr_of!(sv)).configstrings[CS_SERVERINFO as usize];
    let psMapName: LPCSTR =
        Info_ValueForKey(psServerInfo, b"mapname\0".as_ptr() as *const c_char);
    // JLF
    #[cfg(feature = "xbox")]
    {
        let mut mapname: [c_char; filepathlength] = [0; filepathlength];
        let mut numberedmapname: [c_char; filepathlength] = [0; filepathlength];
        let mut mapnumber: c_int = 0;
        let mut numberbuffer: [c_char; 10] = [0; 10];
        if strcmp(b"JKSG3\0".as_ptr() as *const c_char, psPathlessBaseName) == 0 {
            // strcpy(mapname, psMapName);
            strcpy(mapname.as_mut_ptr(), psPathlessBaseName);
            strcpy(numberedmapname.as_mut_ptr(), mapname.as_ptr());
        } else {
            strcpy(mapname.as_mut_ptr(), psMapName);
            strcpy(numberedmapname.as_mut_ptr(), psPathlessBaseName);
        }
        while qbAutosave == qfalse && SG_Exists(numberedmapname.as_ptr()) != qfalse {
            strcpy(numberedmapname.as_mut_ptr(), mapname.as_ptr());
            Com_sprintf(
                numberbuffer.as_mut_ptr(),
                core::mem::size_of::<[c_char; 10]>() as c_int,
                b"_%02i\0".as_ptr() as *const c_char,
                mapnumber,
            );
            strcat(numberedmapname.as_mut_ptr(), numberbuffer.as_ptr());
            mapnumber += 1;
        }
        SG_Create(numberedmapname.as_ptr());
    }
    #[cfg(not(feature = "xbox"))]
    {
        if strcmp(b"quick\0".as_ptr() as *const c_char, psPathlessBaseName) == 0 {
            SG_StoreSaveGameComment(va(
                b"--> %s <--\0".as_ptr() as *const c_char,
                psMapName,
            ));
        }

        if SG_Create(b"current\0".as_ptr() as *const c_char) == qfalse {
            Com_Printf(GetString_FailedToOpenSaveGame(
                b"current\0".as_ptr() as *const c_char,
                qfalse,
            )); // S_COLOR_RED "Failed to create savegame\n"
            SG_WipeSavegame(b"current\0".as_ptr() as *const c_char);
            (*sv_testsave).integer = iPrevTestSave;
            return qfalse;
        }
    }
    // END JLF

    let mut sMapCmd: [c_char; iSG_MAPCMD_SIZE] = [0; iSG_MAPCMD_SIZE];
    strcpy(sMapCmd.as_mut_ptr(), psMapName); // need as array rather than ptr because const strlen needed for MPCM chunk

    SG_WriteComment(qbAutosave, sMapCmd.as_ptr());
    //	SG_WriteScreenshot(qbAutosave, sMapCmd);
    SG_Append(
        0x4D50434D as c_ulong, /* 'MPCM' */
        sMapCmd.as_ptr() as *const c_void,
        core::mem::size_of::<[c_char; iSG_MAPCMD_SIZE]>() as c_int,
    );
    SG_WriteCvars();

    WriteGame(qbAutosave);

    // Write out all the level data...
    //
    if qbAutosave == qfalse {
        SG_Append(
            0x54494D45 as c_ulong, /* 'TIME' */
            &(*addr_of!(sv)).time as *const c_int as *const c_void,
            core::mem::size_of::<c_int>() as c_int,
        );
        SG_Append(
            0x54494D52 as c_ulong, /* 'TIMR' */
            &(*addr_of!(sv)).timeResidual as *const c_int as *const c_void,
            core::mem::size_of::<c_int>() as c_int,
        );
        CM_WritePortalState();
        SG_WriteServerConfigStrings();
    }
    (*ge).WriteLevel(qbAutosave); // always done now, but ent saver only does player if auto
    #[cfg(feature = "xbox")]
    {
        SG_CloseWrite();
    }
    #[cfg(not(feature = "xbox"))]
    {
        SG_Close();
    }
    if *addr_of!(gbSGWriteFailed) != qfalse {
        Com_Printf(GetString_FailedToOpenSaveGame(
            b"current\0".as_ptr() as *const c_char,
            qfalse,
        )); // S_COLOR_RED "Failed to write savegame!\n"
        SG_WipeSavegame(b"current\0".as_ptr() as *const c_char);
        (*sv_testsave).integer = iPrevTestSave;
        return qfalse;
    }

    SG_Move(
        b"current\0".as_ptr() as *const c_char,
        psPathlessBaseName,
    );

    (*sv_testsave).integer = iPrevTestSave;
    qtrue
}

pub unsafe fn SG_ReadSavegame(psPathlessBaseName: *const c_char) -> qboolean {
    let mut sComment: [c_char; iSG_COMMENT_SIZE] = [0; iSG_COMMENT_SIZE];
    let mut sMapCmd: [c_char; iSG_MAPCMD_SIZE] = [0; iSG_MAPCMD_SIZE];
    let mut qbAutosave: qboolean = qfalse;

    let iPrevTestSave: c_int = (*sv_testsave).integer;
    (*sv_testsave).integer = 0;

    if SG_Open(psPathlessBaseName) == qfalse {
        Com_Printf(GetString_FailedToOpenSaveGame(psPathlessBaseName, qtrue));
        // S_COLOR_RED "Failed to open savegame \"%s\"\n", psPathlessBaseName
        (*sv_testsave).integer = iPrevTestSave;
        return qfalse;
    }

    // this check isn't really necessary, but it reminds me that these two strings may actually be the same physical one.
    //
    if psPathlessBaseName != addr_of!(sLastSaveFileLoaded) as *const c_char {
        Q_strncpyz(
            addr_of_mut!(sLastSaveFileLoaded) as *mut c_char,
            psPathlessBaseName,
            core::mem::size_of::<[c_char; MAX_QPATH as usize]>() as c_int,
        );
    }

    // Read in all the server data...
    //
    SG_Read(
        0x434F4D4D as c_ulong, /* 'COMM' */
        sComment.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<[c_char; iSG_COMMENT_SIZE]>() as c_int,
        core::ptr::null_mut(),
    );
    Com_DPrintf(
        b"Reading: %s\n\0".as_ptr() as *const c_char,
        sComment.as_ptr(),
    );
    SG_Read(
        0x434D544D as c_ulong, /* 'CMTM' */
        core::ptr::null_mut(),
        core::mem::size_of::<time_t>() as c_int,
        core::ptr::null_mut(),
    );

    //	SG_ReadScreenshot(qtrue);	// qboolean qbSetAsLoadingScreen
    SG_Read(
        0x4D50434D as c_ulong, /* 'MPCM' */
        sMapCmd.as_mut_ptr() as *mut c_void,
        core::mem::size_of::<[c_char; iSG_MAPCMD_SIZE]>() as c_int,
        core::ptr::null_mut(),
    );
    SG_ReadCvars();

    // read game state
    qbAutosave = ReadGame();
    *addr_of_mut!(eSavedGameJustLoaded) = if qbAutosave != qfalse { eAUTO } else { eFULL };

    SV_SpawnServer(
        sMapCmd.as_ptr(),
        eForceReload_NOTHING,
        (*addr_of!(eSavedGameJustLoaded) != eFULL) as qboolean,
    ); // note that this also trashes the whole G_Alloc pool as well (of course)

    // read in all the level data...
    //
    if qbAutosave == qfalse {
        SG_Read(
            0x54494D45 as c_ulong, /* 'TIME' */
            &mut (*addr_of_mut!(sv)).time as *mut c_int as *mut c_void,
            core::mem::size_of::<c_int>() as c_int,
            core::ptr::null_mut(),
        );
        SG_Read(
            0x54494D52 as c_ulong, /* 'TIMR' */
            &mut (*addr_of_mut!(sv)).timeResidual as *mut c_int as *mut c_void,
            core::mem::size_of::<c_int>() as c_int,
            core::ptr::null_mut(),
        );
        CM_ReadPortalState();
        SG_ReadServerConfigStrings();
    }
    (*ge).ReadLevel(qbAutosave, *addr_of!(qbLoadTransition)); // always done now, but ent reader only does player if auto

    if SG_Close() == qfalse {
        Com_Printf(GetString_FailedToOpenSaveGame(psPathlessBaseName, qfalse));
        // S_COLOR_RED "Failed to close savegame\n"
        (*sv_testsave).integer = iPrevTestSave;
        return qfalse;
    }

    (*sv_testsave).integer = iPrevTestSave;
    qtrue
}

pub unsafe fn Compress_RLE(pIn: *const byte, iLength: c_int, pOut: *mut byte) -> c_int {
    let mut iCount: c_int = 0;
    let mut iOutIndex: c_int = 0;

    while iCount < iLength {
        let mut iIndex: c_int = iCount;
        let b: byte = *pIn.add(iIndex as usize);
        iIndex += 1;

        while iIndex < iLength
            && iIndex - iCount < 127
            && *pIn.add(iIndex as usize) == b
        {
            iIndex += 1;
        }

        if iIndex - iCount == 1 {
            while iIndex < iLength
                && iIndex - iCount < 127
                && (*pIn.add(iIndex as usize) != *pIn.add((iIndex - 1) as usize)
                    || (iIndex > 1
                        && *pIn.add(iIndex as usize) != *pIn.add((iIndex - 2) as usize)))
            {
                iIndex += 1;
            }
            while iIndex < iLength
                && *pIn.add(iIndex as usize) == *pIn.add((iIndex - 1) as usize)
            {
                iIndex -= 1;
            }
            *pOut.add(iOutIndex as usize) = (iCount - iIndex) as u8; /* (unsigned char)(iCount-iIndex) */
            iOutIndex += 1;
            let mut i: c_int = iCount;
            while i < iIndex {
                *pOut.add(iOutIndex as usize) = *pIn.add(i as usize);
                iOutIndex += 1;
                i += 1;
            }
        } else {
            *pOut.add(iOutIndex as usize) = (iIndex - iCount) as u8; /* (unsigned char)(iIndex-iCount) */
            iOutIndex += 1;
            *pOut.add(iOutIndex as usize) = b;
            iOutIndex += 1;
        }
        iCount = iIndex;
    }
    iOutIndex
}

pub unsafe fn DeCompress_RLE(
    pOut: *mut byte,
    pIn: *const byte,
    iDecompressedBytesRemaining: c_int,
) {
    let mut count: i8; /* signed char */
    let mut pIn: *const byte = pIn;
    let mut pOut: *mut byte = pOut;
    let mut iDecompressedBytesRemaining: c_int = iDecompressedBytesRemaining;

    while iDecompressedBytesRemaining > 0 {
        count = *pIn as i8;
        pIn = pIn.add(1);
        if count > 0 {
            let fill: byte = *pIn;
            pIn = pIn.add(1);
            core::ptr::write_bytes(pOut, fill, count as usize);
        } else if count < 0 {
            count = (-(count as i32)) as i8; /* (signed char) -count */
            core::ptr::copy_nonoverlapping(pIn, pOut, count as usize);
            pIn = pIn.add(count as usize);
        }
        pOut = pOut.add(count.unsigned_abs() as usize);
        iDecompressedBytesRemaining -= count.unsigned_abs() as c_int;
    }
}

// simulate decompression over original data (but don't actually do it), to test de/compress validity...
//
pub unsafe fn Verify_RLE(
    pOut: *const byte,
    pIn: *const byte,
    iDecompressedBytesRemaining: c_int,
) -> qboolean {
    let mut count: i8; /* signed char */
    let pOutEnd: *const byte = pOut.add(iDecompressedBytesRemaining as usize);
    let mut pOut: *const byte = pOut;
    let mut pIn: *const byte = pIn;
    let mut iDecompressedBytesRemaining: c_int = iDecompressedBytesRemaining;

    while iDecompressedBytesRemaining > 0 {
        if pOut >= pOutEnd {
            return qfalse;
        }
        count = *pIn as i8;
        pIn = pIn.add(1);
        if count > 0 {
            // memset(pOut,*pIn++,count);
            let iMemSetByte: c_int = *pIn as c_int;
            pIn = pIn.add(1);
            let mut i: c_int = 0;
            while i < count as c_int {
                if *pOut.add(i as usize) != iMemSetByte as byte {
                    return qfalse;
                }
                i += 1;
            }
        } else if count < 0 {
            count = (-(count as i32)) as i8; /* (signed char) -count */
            //			memcpy(pOut,pIn,count);
            if memcmp(
                pOut as *const c_void,
                pIn as *const c_void,
                count as usize,
            ) != 0
            {
                return qfalse;
            }
            pIn = pIn.add(count as usize);
        }
        pOut = pOut.add(count.unsigned_abs() as usize);
        iDecompressedBytesRemaining -= count.unsigned_abs() as c_int;
    }

    if pOut != pOutEnd {
        return qfalse;
    }

    qtrue
}

pub static mut gpbCompBlock: *mut byte = core::ptr::null_mut();
pub static mut giCompBlockSize: c_int = 0;

unsafe fn CompressMem_FreeScratchBuffer() {
    if !(*addr_of!(gpbCompBlock)).is_null() {
        Z_Free(*addr_of!(gpbCompBlock) as *mut c_void);
        *addr_of_mut!(gpbCompBlock) = core::ptr::null_mut();
    }
    *addr_of_mut!(giCompBlockSize) = 0;
}

unsafe fn CompressMem_AllocScratchBuffer(iSize: c_int) -> *mut byte {
    // only alloc new buffer if we need more than the existing one...
    //
    if *addr_of!(giCompBlockSize) < iSize {
        CompressMem_FreeScratchBuffer();

        *addr_of_mut!(gpbCompBlock) =
            Z_Malloc(iSize, TAG_TEMP_WORKSPACE, qfalse) as *mut byte;
        *addr_of_mut!(giCompBlockSize) = iSize;
    }

    *addr_of!(gpbCompBlock)
}

// returns -1 for compression-not-worth-it, else compressed length...
//
pub unsafe fn CompressMem(pbData: *mut byte, iLength: c_int, pbOut: &mut *mut byte) -> c_int {
    if (*sv_compress_saved_games).integer == 0 {
        return -1;
    }

    // malloc enough to cope with uncompressable data (it'll never grow to 2* size, so)...
    //
    *pbOut = CompressMem_AllocScratchBuffer(iLength * 2);
    //
    // compress it...
    //
    let iOutputLength: c_int = Compress_RLE(pbData, iLength, *pbOut);
    //
    // worth compressing?...
    //
    if iOutputLength >= iLength {
        return -1;
    }
    //
    // compression code works? (I'd hope this is always the case, but for safety)...
    //
    if Verify_RLE(pbData, *pbOut, iLength) == qfalse {
        return -1;
    }

    iOutputLength
}

// #ifdef _XBOX // function for xbox
/*
int SG_Write(const void * chid, const int bytesize, fileHandle_t fhSG)
{
	DWORD bytesWritten;
	int currentsize;


	if (!WriteFile(sg_Handle,                    // handle to file
							chid,                // data buffer
							bytesize,     // number of bytes to write
							&bytesWritten,  // number of bytes written
							NULL        // overlapped buffer
							))
		{
			return 0;
		}

		return bytesWritten;
}
*/

#[cfg(feature = "xbox")]
pub unsafe fn SG_Write(chid: *const c_void, bytesize: c_int, fhSG: fileHandle_t) -> c_int {
    let mut bytesWritten: u32 /* DWORD */ = 0;

    if *addr_of!(sg_BufferSize) + bytesize >= SG_BUFFERSIZE as c_int {
        if WriteFile(
            *addr_of!(sg_Handle),
            addr_of!(sg_Buffer) as *const c_void,
            *addr_of!(sg_BufferSize) as u32,
            &mut bytesWritten,
            core::ptr::null_mut(),
        ) == 0
        // return bytesWritten;
        // else
        {
            return 0;
        }

        let _dwSuccess: u32 /* DWORD */ = XCalculateSignatureUpdate(
            *addr_of!(sg_sigHandle),
            addr_of!(sg_Buffer) as *const u8,
            *addr_of!(sg_BufferSize) as u32,
        );
        *addr_of_mut!(sg_BufferSize) = 0;
    }
    if bytesize >= SG_BUFFERSIZE as c_int {
        if WriteFile(
            *addr_of!(sg_Handle),
            chid,
            bytesize as u32,
            &mut bytesWritten,
            core::ptr::null_mut(),
        ) == 0
        {
            return 0;
        }
        let _dwSuccess: u32 /* DWORD */ = XCalculateSignatureUpdate(
            *addr_of!(sg_sigHandle),
            chid as *const u8,
            bytesize as u32,
        );
        *addr_of_mut!(sg_BufferSize) = 0;
    } else {
        let tempptr: *mut byte =
            addr_of_mut!(sg_Buffer).cast::<byte>().add(*addr_of!(sg_BufferSize) as usize);
        core::ptr::copy_nonoverlapping(chid as *const byte, tempptr, bytesize as usize);
        *addr_of_mut!(sg_BufferSize) += bytesize;
    }
    bytesize
}

// #else
// pass through function
#[cfg(not(feature = "xbox"))]
pub unsafe fn SG_Write(chid: *const c_void, bytesize: c_int, fhSaveGame: fileHandle_t) -> c_int {
    FS_Write(chid, bytesize, fhSaveGame)
}
// #endif

pub unsafe fn SG_Append(chid: c_ulong, pvData: *const c_void, mut iLength: c_int) -> qboolean {
    let mut uiCksum: c_uint;
    let mut uiSaved: c_uint;

    #[cfg(debug_assertions)]
    {
        let mut i: c_int;
        let mut pTest: *mut c_ulong;

        pTest = pvData as *mut c_ulong;
        i = 0;
        while i < iLength / 4 {
            assert!(*pTest != 0xfeeefeee);
            assert!(*pTest != 0xcdcdcdcd);
            assert!(*pTest != 0xdddddddd);
            pTest = pTest.add(1);
            i += 1;
        }
    }

    Com_DPrintf(
        b"Attempting write of chunk %s length %d\n\0".as_ptr() as *const c_char,
        SG_GetChidText(chid),
        iLength,
    );

    // only write data out if we're not in test mode....
    //
    if (*sv_testsave).integer == 0 {
        uiCksum = Com_BlockChecksum(pvData, iLength) as c_uint;

        uiSaved = SG_Write(
            &chid as *const c_ulong as *const c_void,
            core::mem::size_of::<c_ulong>() as c_int,
            *addr_of!(fhSaveGame),
        ) as c_uint;

        let mut pbCompressedData: *mut byte = core::ptr::null_mut();
        let iCompressedLength: c_int =
            CompressMem(pvData as *mut byte, iLength, &mut pbCompressedData);
        if iCompressedLength != -1 {
            // compressed...  (write length field out as -ve)
            //
            iLength = -iLength;
            uiSaved += SG_Write(
                &iLength as *const c_int as *const c_void,
                core::mem::size_of::<c_int>() as c_int,
                *addr_of!(fhSaveGame),
            ) as c_uint;
            iLength = -iLength;
            //
            // [compressed length]
            //
            uiSaved += SG_Write(
                &iCompressedLength as *const c_int as *const c_void,
                core::mem::size_of::<c_int>() as c_int,
                *addr_of!(fhSaveGame),
            ) as c_uint;
            //
            // compressed data...
            //
            uiSaved += SG_Write(
                pbCompressedData as *const c_void,
                iCompressedLength,
                *addr_of!(fhSaveGame),
            ) as c_uint;
            //
            // CRC...
            //
            uiSaved += SG_Write(
                &uiCksum as *const c_uint as *const c_void,
                core::mem::size_of::<c_uint>() as c_int,
                *addr_of!(fhSaveGame),
            ) as c_uint;

            if uiSaved
                != (core::mem::size_of::<c_ulong>()
                    + core::mem::size_of::<c_int>()
                    + core::mem::size_of::<c_uint>()
                    + core::mem::size_of::<c_int>()
                    + iCompressedLength as usize) as c_uint
            {
                Com_Printf(
                    b"%sFailed to write %s chunk\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                    SG_GetChidText(chid),
                );
                *addr_of_mut!(gbSGWriteFailed) = qtrue;
                return qfalse;
            }
        } else {
            // uncompressed...
            //
            uiSaved += SG_Write(
                &iLength as *const c_int as *const c_void,
                core::mem::size_of::<c_int>() as c_int,
                *addr_of!(fhSaveGame),
            ) as c_uint;
            //
            // uncompressed data...
            //
            uiSaved += SG_Write(pvData, iLength, *addr_of!(fhSaveGame)) as c_uint;
            //
            // CRC...
            //
            uiSaved += SG_Write(
                &uiCksum as *const c_uint as *const c_void,
                core::mem::size_of::<c_int>() as c_int,
                *addr_of!(fhSaveGame),
            ) as c_uint;

            if uiSaved
                != (core::mem::size_of::<c_ulong>()
                    + core::mem::size_of::<c_int>()
                    + core::mem::size_of::<c_uint>()
                    + iLength as usize) as c_uint
            {
                Com_Printf(
                    b"%sFailed to write %s chunk\n\0".as_ptr() as *const c_char,
                    S_COLOR_RED,
                    SG_GetChidText(chid),
                );
                *addr_of_mut!(gbSGWriteFailed) = qtrue;
                return qfalse;
            }
        }

        #[cfg(feature = "sg_profile")]
        {
            get_save_info()
                .entry(chid)
                .or_insert_with(CChid::new)
                .Add(iLength);
        }
    }

    qtrue
}

// #ifdef _XBOX // function for xbox
// SG_ReadBytes replaces FS_Read. I was going to use SG_Read but it is already in use
/*
int SG_ReadBytes(void * chid, int bytesize, fileHandle_t fhSG)
{
	byte* bufferptr;
	unsigned char* destptr;
	DWORD retBytesRead=0;
	DWORD bytesRead =0;
	int segmentLength;


	//bufferptr = (byte*)chid;
	//destptr = NULL;

	if (ReadFile(sg_Handle,                    // handle to file
				chid,                // data buffer
				bytesize,     // number of bytes to write
				&bytesRead,  // number of bytes written
				NULL        // overlapped buffer
				))
		{	return bytesRead;
		}
		else
		{
			return 0;
		}

	return retBytesRead;
}
*/

#[cfg(feature = "xbox")]
pub unsafe fn SG_ReadBytes(chid: *mut c_void, bytesize: c_int, fhSG: fileHandle_t) -> c_int {
    let mut bufferptr: *mut byte;
    let mut destptr: *mut u8; /* unsigned char* */
    let mut retBytesRead: u32 /* DWORD */ = 0;
    let mut bytesRead: u32 /* DWORD */ = 0;
    let mut segmentLength: c_int;
    let mut bytesize: c_int = bytesize;

    // bufferptr = (byte*)chid;
    // destptr = NULL;

    if bytesize < (*addr_of!(sg_BufferSize) - *addr_of!(sg_CurrentBufferPos)) {
        bufferptr = addr_of_mut!(sg_Buffer)
            .cast::<byte>()
            .add(*addr_of!(sg_CurrentBufferPos) as usize);
        core::ptr::copy_nonoverlapping(bufferptr, chid as *mut byte, bytesize as usize);
        *addr_of_mut!(sg_CurrentBufferPos) += bytesize;
        retBytesRead = bytesize as u32;
    } else {
        destptr = chid as *mut u8;

        while bytesize > 0 {
            bufferptr = addr_of_mut!(sg_Buffer)
                .cast::<byte>()
                .add(*addr_of!(sg_CurrentBufferPos) as usize);
            segmentLength = *addr_of!(sg_BufferSize) - *addr_of!(sg_CurrentBufferPos);
            if segmentLength <= bytesize {
                core::ptr::copy_nonoverlapping(bufferptr, destptr, segmentLength as usize);
                destptr = destptr.add(segmentLength as usize);
                retBytesRead += segmentLength as u32;
                bytesize -= segmentLength;
                *addr_of_mut!(sg_CurrentBufferPos) += segmentLength;
            } else {
                core::ptr::copy_nonoverlapping(bufferptr, destptr, bytesize as usize);
                destptr = destptr.add(bytesize as usize);
                retBytesRead += bytesize as u32;
                *addr_of_mut!(sg_CurrentBufferPos) += bytesize;
                bytesize -= bytesize;
            }

            if *addr_of!(sg_BufferSize) - *addr_of!(sg_CurrentBufferPos) <= 0 && bytesize > 0 {
                if ReadFile(
                    *addr_of!(sg_Handle),
                    addr_of_mut!(sg_Buffer) as *mut c_void,
                    SG_BUFFERSIZE as u32,
                    &mut bytesRead,
                    core::ptr::null_mut(),
                ) != 0
                {
                    *addr_of_mut!(sg_BufferSize) = bytesRead as c_int;
                    *addr_of_mut!(sg_CurrentBufferPos) = 0;
                    bufferptr = addr_of_mut!(sg_Buffer).cast::<byte>();
                    // sig processing
                } else {
                    return 0;
                }
            }
        }
    }
    retBytesRead as c_int
}

// handle offset origin
// fhSaveGame not used (use global variable)
#[cfg(feature = "xbox")]
pub unsafe fn SG_Seek(fhSaveGame: fileHandle_t, offset: c_long, origin: c_int) -> c_int {
    match origin {
        FS_SEEK_CUR => SetFilePointer(
            *addr_of!(sg_Handle), // handle to file
            offset as i32,        // bytes to move pointer
            core::ptr::null_mut(), // bytes to move pointer
            FILE_CURRENT,         // starting point
        ) as c_int,
        FS_SEEK_END => SetFilePointer(
            *addr_of!(sg_Handle), // handle to file
            offset as i32,        // bytes to move pointer
            core::ptr::null_mut(), // bytes to move pointer
            FILE_END,             // starting point
        ) as c_int,
        _ => {
            // FS_SEEK_SET:
            SetFilePointer(
                *addr_of!(sg_Handle), // handle to file
                offset as i32,        // bytes to move pointer
                core::ptr::null_mut(), // bytes to move pointer
                FILE_BEGIN,           // starting point
            ) as c_int
        }
    }
}

// #else
// pass through function
#[cfg(not(feature = "xbox"))]
pub unsafe fn SG_ReadBytes(chid: *mut c_void, bytesize: c_int, fhSaveGame: fileHandle_t) -> c_int {
    FS_Read(chid, bytesize, fhSaveGame)
}

#[cfg(not(feature = "xbox"))]
pub unsafe fn SG_Seek(fhSaveGame: fileHandle_t, offset: c_long, origin: c_int) -> c_int {
    FS_Seek(fhSaveGame, offset, origin)
}
// #endif

// Pass in pvAddress (or NULL if you want memory to be allocated)
//	if pvAddress==NULL && ppvAddressPtr == NULL then the block is discarded/skipped.
//
// If iLength==0 then it counts as a query, else it must match the size found in the file
//
// function doesn't return if error (uses ERR_DROP), unless "qbSGReadIsTestOnly == qtrue", then NZ return = success
//
unsafe fn SG_Read_Actual(
    chid: c_ulong,
    mut pvAddress: *mut c_void,
    mut iLength: c_int,
    ppvAddressPtr: *mut *mut c_void,
    bChunkIsOptional: qboolean,
) -> c_int {
    let mut uiLoadedCksum: c_uint = 0;
    let mut uiCksum: c_uint;
    let mut uiLoadedLength: c_uint = 0;
    let mut ulLoadedChid: c_ulong = 0;
    let mut uiLoaded: c_uint;
    let mut sChidText1: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut sChidText2: [c_char; MAX_QPATH as usize] = [0; MAX_QPATH as usize];
    let mut qbTransient: qboolean = qfalse;

    Com_DPrintf(
        b"Attempting read of chunk %s length %d\n\0".as_ptr() as *const c_char,
        SG_GetChidText(chid),
        iLength,
    );

    // Load in chid and length...
    //
    uiLoaded = SG_ReadBytes(
        &mut ulLoadedChid as *mut c_ulong as *mut c_void,
        core::mem::size_of::<c_ulong>() as c_int,
        *addr_of!(fhSaveGame),
    ) as c_uint;
    uiLoaded += SG_ReadBytes(
        &mut uiLoadedLength as *mut c_uint as *mut c_void,
        core::mem::size_of::<c_uint>() as c_int,
        *addr_of!(fhSaveGame),
    ) as c_uint;

    let bBlockIsCompressed: qboolean =
        if (uiLoadedLength as i32) < 0 { qtrue } else { qfalse };
    if bBlockIsCompressed != qfalse {
        uiLoadedLength = (-(uiLoadedLength as i32)) as c_uint;
    }

    // Make sure we are loading the correct chunk...
    //
    if ulLoadedChid != chid {
        if bChunkIsOptional != qfalse {
            SG_Seek(*addr_of!(fhSaveGame), -(uiLoaded as c_long), FS_SEEK_CUR);
            return 0;
        }

        strcpy(sChidText1.as_mut_ptr(), SG_GetChidText(ulLoadedChid));
        strcpy(sChidText2.as_mut_ptr(), SG_GetChidText(chid));
        if *addr_of!(qbSGReadIsTestOnly) == qfalse {
            Com_Error(
                ERR_DROP,
                b"Loaded chunk ID (%s) does not match requested chunk ID (%s)\0".as_ptr()
                    as *const c_char,
                sChidText1.as_ptr(),
                sChidText2.as_ptr(),
            );
        }
        return 0;
    }

    // Find length of chunk and make sure it matches the requested length...
    //
    if iLength != 0 {
        // .. but only if there was one specified
        if iLength != uiLoadedLength as c_int {
            if *addr_of!(qbSGReadIsTestOnly) == qfalse {
                Com_Error(
                    ERR_DROP,
                    b"Loaded chunk (%s) has different length than requested\0".as_ptr()
                        as *const c_char,
                    SG_GetChidText(chid),
                );
            }
            return 0;
        }
    }
    iLength = uiLoadedLength as c_int; // for retval

    // alloc?...
    //
    if pvAddress.is_null() {
        pvAddress = Z_Malloc(iLength, TAG_SAVEGAME, qfalse) as *mut c_void;
        //
        // Pass load address back...
        //
        if !ppvAddressPtr.is_null() {
            *ppvAddressPtr = pvAddress;
        } else {
            qbTransient = qtrue; // if no passback addr, mark block for skipping
        }
    }

    // Load in data and magic number...
    //
    let mut uiCompressedLength: c_uint = 0;
    if bBlockIsCompressed != qfalse {
        //
        // read compressed data length...
        //
        uiLoaded += SG_ReadBytes(
            &mut uiCompressedLength as *mut c_uint as *mut c_void,
            core::mem::size_of::<c_uint>() as c_int,
            *addr_of!(fhSaveGame),
        ) as c_uint;
        //
        // alloc space...
        //
        let pTempRLEData: *mut byte =
            Z_Malloc(uiCompressedLength as c_int, TAG_SAVEGAME, qfalse) as *mut byte;
        //
        // read compressed data...
        //
        uiLoaded += SG_ReadBytes(
            pTempRLEData as *mut c_void,
            uiCompressedLength as c_int,
            *addr_of!(fhSaveGame),
        ) as c_uint;
        //
        // decompress it...
        //
        DeCompress_RLE(pvAddress as *mut byte, pTempRLEData, iLength);
        //
        // free workspace...
        //
        Z_Free(pTempRLEData as *mut c_void);
    } else {
        uiLoaded +=
            SG_ReadBytes(pvAddress, iLength, *addr_of!(fhSaveGame)) as c_uint;
    }
    // Get checksum...
    //
    uiLoaded += SG_ReadBytes(
        &mut uiLoadedCksum as *mut c_uint as *mut c_void,
        core::mem::size_of::<c_uint>() as c_int,
        *addr_of!(fhSaveGame),
    ) as c_uint;

    // Make sure the checksums match...
    //
    uiCksum = Com_BlockChecksum(pvAddress, iLength) as c_uint;
    if uiLoadedCksum != uiCksum {
        if *addr_of!(qbSGReadIsTestOnly) == qfalse {
            Com_Error(
                ERR_DROP,
                b"Failed checksum check for chunk\0".as_ptr() as *const c_char,
                SG_GetChidText(chid),
            );
        } else if qbTransient != qfalse {
            Z_Free(pvAddress);
        }
        return 0;
    }

    // Make sure we didn't encounter any read errors...
    // size_t
    if uiLoaded
        != (core::mem::size_of::<c_ulong>()
            + core::mem::size_of::<c_uint>()
            + core::mem::size_of::<c_uint>()
            + if bBlockIsCompressed != qfalse {
                core::mem::size_of::<c_uint>()
            } else {
                0
            }
            + if bBlockIsCompressed != qfalse {
                uiCompressedLength as usize
            } else {
                iLength as usize
            }) as c_uint
    {
        if *addr_of!(qbSGReadIsTestOnly) == qfalse {
            Com_Error(
                ERR_DROP,
                b"Error during loading chunk %s\0".as_ptr() as *const c_char,
                SG_GetChidText(chid),
            );
        } else if qbTransient != qfalse {
            Z_Free(pvAddress);
        }
        return 0;
    }

    // If we are skipping the chunk, then free the memory...
    //
    if qbTransient != qfalse {
        Z_Free(pvAddress);
    }

    iLength
}

pub unsafe fn SG_Read(
    chid: c_ulong,
    pvAddress: *mut c_void,
    iLength: c_int,
    ppvAddressPtr: *mut *mut c_void, /* = NULL */
) -> c_int {
    SG_Read_Actual(chid, pvAddress, iLength, ppvAddressPtr, qfalse) // qboolean bChunkIsOptional
}

pub unsafe fn SG_ReadOptional(
    chid: c_ulong,
    pvAddress: *mut c_void,
    iLength: c_int,
    ppvAddressPtr: *mut *mut c_void, /* = NULL */
) -> c_int {
    SG_Read_Actual(chid, pvAddress, iLength, ppvAddressPtr, qtrue) // qboolean bChunkIsOptional
}

pub unsafe fn SG_TestSave() {
    if (*sv_testsave).integer != 0 && (*addr_of!(sv)).state == SS_GAME {
        WriteGame(false as qboolean);
        (*ge).WriteLevel(false as qboolean);
    }
}

////////////////// eof ////////////////////

// Helper for sg_profile BTreeMap access
#[cfg(feature = "sg_profile")]
unsafe fn get_save_info() -> &'static mut std::collections::BTreeMap<c_ulong, CChid> {
    if (*addr_of!(save_info)).is_none() {
        *addr_of_mut!(save_info) = Some(std::collections::BTreeMap::new());
    }
    (*addr_of_mut!(save_info)).as_mut().unwrap()
}
