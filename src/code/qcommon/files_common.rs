/*****************************************************************************
 * name:		files.c
 *
 * desc:		handle based filesystem for Quake III Arena
 *
 *
 *****************************************************************************/


// #include "../game/q_shared.h"
// #include "qcommon.h"
// #include "files.h"

#![allow(non_snake_case)]
#![allow(static_mut_refs)]

use core::ffi::{c_char, c_int, c_void};

/*
=============================================================================

QUAKE3 FILESYSTEM

All of Quake's data access is through a hierarchical file system, but the contents of
the file system can be transparently merged from several sources.

A "qpath" is a reference to game file data.  MAX_QPATH is 64 characters, which must include
a terminating zero. "..", "\\", and ":" are explicitly illegal in qpaths to prevent any
references outside the quake directory system.

The "base path" is the path to the directory holding all the game directories and usually
the executable.  It defaults to ".", but can be overridden with a "+set fs_basepath c:\quake3"
command line to allow code debugging in a different directory.  Basepath cannot
be modified at all after startup.  Any files that are created (demos, screenshots,
etc) will be created relative to the base path, so base path should usually be writable.

The "cd path" is the path to an alternate hierarchy that will be searched if a file
is not located in the base path.  A user can do a partial install that copies some
data to a base path created on their hard drive and leave the rest on the cd.  Files
are never writen to the cd path.  It defaults to a value set by the installer, like
"e:\quake3", but it can be overridden with "+set ds_cdpath g:\quake3".

If a user runs the game directly from a CD, the base path would be on the CD.  This
should still function correctly, but all file writes will fail (harmlessly).


The "base game" is the directory under the paths where data comes from by default, and
can be either "base" or "demo".

The "current game" may be the same as the base game, or it may be the name of another
directory under the paths that should be searched for files before looking in the base game.
This is the basis for addons.

Clients automatically set the game directory after receiving a gamestate from a server,
so only servers need to worry about +set fs_game.

No other directories outside of the base game and current game will ever be referenced by
filesystem functions.

To save disk space and speed loading, directory trees can be collapsed into zip files.
The files use a ".pk3" extension to prevent users from unzipping them accidentally, but
otherwise the are simply normal uncompressed zip files.  A game directory can have multiple
zip files of the form "asset0.pk3", "pak1.pk3", etc.  Zip files are searched in decending order
from the highest number to the lowest, and will always take precedence over the filesystem.
This allows a pk3 distributed as a patch to override all existing data.

Because we will have updated executables freely available online, there is no point to
trying to restrict demo / oem versions of the game with code changes.  Demo / oem versions
should be exactly the same executables as release versions, but with different data that
automatically restricts where game media can come from to prevent add-ons from working.

After the paths are initialized, quake will look for the product.txt file.  If not
found and verified, the game will run in restricted mode.  In restricted mode, only
files contained in demo/asset0.pk3 will be available for loading, and only if the zip header is
verified to not have been modified.  A single exception is made for jaconfig.cfg.  Files
can still be written out in restricted mode, so screenshots and demos are allowed.
Restricted mode can be tested by setting "+set fs_restrict 1" on the command line, even
if there is a valid product.txt under the basepath or cdpath.

If not running in restricted mode, and a file is not found in any local filesystem,
an attempt will be made to download it and save it under the base path.

If the "fs_copyfiles" cvar is set to 1, then every time a file is sourced from the cd
path, it will be copied over to the base path.  This is a development aid to help build
test releases and to copy working sets over slow network links.
(If set to 2, copying will only take place if the two filetimes are NOT EQUAL)


The qpath "sound/newstuff/test.wav" would be searched for in the following places:

base path + current game's zip files
base path + current game's directory
cd path + current game's zip files
cd path + current game's directory
base path + base game's zip files
base path + base game's directory
cd path + base game's zip files
cd path + base game's directory
server download, to be written to base path + current game's directory


The filesystem can be safely shutdown and reinitialized with different
basedir / cddir / game combinations, but all other subsystems that rely on it
(sound, video) must also be forced to restart.

Because the same files are loaded by both the clip model (CM_) and renderer (TR_)
subsystems, a simple single-file caching scheme is used.  The CM_ subsystems will
load the file with a request to cache.  Only one file will be kept cached at a time,
so any models that are going to be referenced by both subsystems should alternate
between the CM_ load function and the ref load function.




TODO: A qpath that starts with a leading slash will always refer to the base game, even if another
game is currently active.  This allows character models, skins, and sounds to be downloaded
to a common directory no matter which game is active.


How to prevent downloading zip files?
Pass pk3 file names in systeminfo, and download before FS_Restart()?



Aborting a download disconnects the client from the server.

How to mark files as downloadable?  Commercial add-ons won't be downloadable.

Non-commercial downloads will want to download the entire zip file.
the game would have to be reset to actually read the zip in

Auto-update information

Path separators

Casing

  separate server gamedir and client gamedir, so if the user starts
  a local game after having connected to a network game, it won't stick
  with the network game.

  allow menu options for game selection?

Read / write config to floppy option.

Different version coexistance?

When building a pak file, make sure a jaconfig.cfg isn't present in it,
or configs will never get loaded from disk!

  todo:

  downloading (outside fs?)
  game directory passing and restarting

=============================================================================

*/

// if this is defined, the executable positively won't work with any paks other
// than the demo pak, even if productid is present.  This is only used for our
// last demo release to prevent the mac and linux users from using the demo
// executable with the production windows pak before the mac/linux products
// hit the shelves a little later
//#define	PRE_RELEASE_DEMO

// ============================================================================
// Constants
// ============================================================================

const MAX_QPATH: usize = 64;
const MAX_OSPATH: usize = 260;
const MAX_FILE_HANDLES: usize = 16;
const MAXPRINTMSG: usize = 4096;
const BASEGAME: &[u8] = b"base";
const PATH_SEP: c_char = '/' as c_char;
const ERR_FATAL: c_int = 1;
const ERR_DROP: c_int = 2;

// Type aliases
type qboolean = c_int;
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;
type fileHandle_t = c_int;

// ============================================================================
// Structures (from files.h and cvar.h)
// ============================================================================

// cvar_t from cvar.h - needed to access cvar string field
#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub resetString: *mut c_char,
    pub latchedString: *mut c_char,
    pub flags: c_int,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub next: *mut cvar_t,
}

#[repr(C)]
pub struct fileInPack_s {
    pub name: *mut c_char,
    pub pos: c_int, // file info position in zip
    pub next: *mut fileInPack_s,
}

#[repr(C)]
pub struct pack_t {
    pub pakFilename: [c_char; MAX_OSPATH],
    pub handle: *mut c_void, // unzFile or similar
    pub checksum: c_int,
    pub numfiles: c_int,
    pub hashSize: c_int,
    pub hashTable: *mut *mut fileInPack_s,
    pub buildBuffer: *mut fileInPack_s,
}

#[repr(C)]
pub struct directory_t {
    pub path: [c_char; MAX_OSPATH],
    pub gamedir: [c_char; MAX_OSPATH],
}

#[repr(C)]
pub struct searchpath_s {
    pub next: *mut searchpath_s,
    pub pack: *mut pack_t,
    pub dir: *mut directory_t,
}

pub type searchpath_t = searchpath_s;

#[repr(C)]
union qfile_gut {
    o: *mut c_void, // FILE*
    z: *mut c_void, // unzFile
}

#[repr(C)]
struct qfile_ut {
    file: qfile_gut,
    unique: qboolean,
}

#[repr(C)]
pub struct fileHandleData_t {
    pub handleFiles: qfile_ut,
    pub handleSync: qboolean,
    pub baseOffset: c_int,
    pub fileSize: c_int,
    pub zipFilePos: c_int,
    pub zipFile: qboolean,
    pub name: [c_char; MAX_QPATH],
}

// ============================================================================
// Global Variables
// ============================================================================

pub static mut fs_gamedir: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];	// this will be a single file name with no separators
pub static mut fs_debug: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_basepath: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_cdpath: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_copyfiles: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_gamedirvar: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_restrict: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_searchpaths: *mut searchpath_t = core::ptr::null_mut();
pub static mut fs_readCount: c_int = 0;			// total bytes read
pub static mut fs_loadCount: c_int = 0;			// total files read
pub static mut fs_packFiles: c_int = 0;			// total number of files in packs

static mut initialized: qboolean = qfalse;

// Helper function to create a zero-initialized fileHandleData_t
const fn zeroed_file_handle() -> fileHandleData_t {
    fileHandleData_t {
        handleFiles: qfile_ut {
            file: qfile_gut { o: core::ptr::null_mut() },
            unique: 0,
        },
        handleSync: 0,
        baseOffset: 0,
        fileSize: 0,
        zipFilePos: 0,
        zipFile: 0,
        name: [0; MAX_QPATH],
    }
}

pub static mut fsh: [fileHandleData_t; MAX_FILE_HANDLES] = [zeroed_file_handle(); MAX_FILE_HANDLES];

// ============================================================================
// External C Functions
// ============================================================================

extern "C" {
    fn Com_Error(code: c_int, fmt: *const c_char, ...) -> !;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...);
    fn Com_StartupVariable(variable: *const c_char);
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn Q_islower(c: c_int) -> c_int;
    fn S_ClearSoundBuffer();
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut c_void; // FILE*
    fn fclose(f: *mut c_void) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn fflush(f: *mut c_void) -> c_int;
    fn Sys_Mkdir(path: *const c_char);
    fn strlwr(s: *mut c_char) -> *mut c_char;
    fn strstr(s: *const c_char, substr: *const c_char) -> *mut c_char;
    fn unzClose(file: *mut c_void) -> c_int;
    fn Z_Free(ptr: *mut c_void);
    fn Z_Malloc(size: usize, tag: c_int, clear: qboolean) -> *mut c_void;
    fn Cmd_RemoveCommand(cmd_name: *const c_char);
    fn FS_Startup(gameName: *const c_char);
    fn FS_SetRestrictions();
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_WriteFile(qpath: *const c_char, buffer: *const c_void, size: c_int);
    fn FS_FOpenFileWrite(qpath: *const c_char) -> fileHandle_t;
    fn FS_FCloseFile(f: fileHandle_t);
    fn FS_filelength(f: fileHandle_t) -> c_int;
    fn FS_Write(buffer: *const c_void, len: c_int, h: fileHandle_t);
}

// ============================================================================
// Functions
// ============================================================================

pub fn FS_CheckInit() {
    unsafe {
        if initialized == 0 {
            Com_Error(ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char);
        }
    }
}

/*
==============
FS_Initialized
==============
*/

pub fn FS_Initialized() -> qboolean {
    unsafe {
        return (fs_searchpaths != core::ptr::null_mut()) as c_int;
    }
}

pub fn FS_HandleForFile() -> fileHandle_t {
    let mut i: c_int;

    unsafe {
        for i in 1..(MAX_FILE_HANDLES as c_int) {
            if fsh[i as usize].handleFiles.file.o == core::ptr::null_mut() as *mut c_void {
                return i;
            }
        }

        Com_Printf("FS_HandleForFile: all handles taken:\n\0".as_ptr() as *const c_char);
        for i in 1..(MAX_FILE_HANDLES as c_int) {
            Com_Printf("%d. %s\n\0".as_ptr() as *const c_char, i, fsh[i as usize].name.as_ptr());
        }
        Com_Error(ERR_DROP, "FS_HandleForFile: none free\0".as_ptr() as *const c_char);
        return 0;
    }
}

/*
====================
FS_ReplaceSeparators

Fix things up differently for win/unix/mac
====================
*/
pub fn FS_ReplaceSeparators(path: *mut c_char) {
    let mut s: *mut c_char;

    unsafe {
        s = path;
        while *s != 0 {
            if *s == '/' as c_char || *s == '\\' as c_char {
                *s = PATH_SEP;
            }
            s = s.add(1);
        }
    }
}

/*
===================
FS_BuildOSPath

Qpath may have either forward or backwards slashes
===================
*/

pub fn FS_BuildOSPath(qpath: *const c_char) -> *mut c_char {
    let mut temp: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    static mut ospath: [[c_char; MAX_OSPATH]; 2] = [[0; MAX_OSPATH]; 2];
    static mut toggle: c_int = 0;

    unsafe {
        toggle ^= 1;		// flip-flop to allow two returns without clash

        Com_sprintf(
            temp.as_mut_ptr(),
            temp.len(),
            "/%s/%s\0".as_ptr() as *const c_char,
            (*fs_gamedirvar).string,
            qpath
        );

        FS_ReplaceSeparators(temp.as_mut_ptr());
        Com_sprintf(
            ospath[toggle as usize].as_mut_ptr(),
            ospath[0].len(),
            "%s%s\0".as_ptr() as *const c_char,
            (*fs_basepath).string,
            temp.as_ptr() as *const c_char
        );

        return ospath[toggle as usize].as_mut_ptr();
    }
}

pub fn FS_BuildOSPath_full(base: *const c_char, game: *const c_char, qpath: *const c_char) -> *mut c_char {
    let mut temp: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
    static mut ospath: [[c_char; MAX_OSPATH]; 4] = [[0; MAX_OSPATH]; 4];
    static mut toggle: c_int = 0;

    unsafe {
        toggle = (toggle + 1) & 3;	// allows four returns without clash (increased from 2 during fs_copyfiles 2 enhancement)

        Com_sprintf(temp.as_mut_ptr(), temp.len(), "/%s/%s\0".as_ptr() as *const c_char, game, qpath);
        FS_ReplaceSeparators(temp.as_mut_ptr());
        Com_sprintf(
            ospath[toggle as usize].as_mut_ptr(),
            ospath[0].len(),
            "%s%s\0".as_ptr() as *const c_char,
            base,
            temp.as_ptr() as *const c_char
        );

        return ospath[toggle as usize].as_mut_ptr();
    }
}

/*
============
FS_CreatePath

Creates any directories needed to store the given filename
============
*/
pub fn FS_CreatePath(OSPath: *mut c_char) {
    let mut ofs: *mut c_char;

    unsafe {
        // make absolutely sure that it can't back up the path
        // FIXME: is c: allowed???
        if !strstr(OSPath, "..\0".as_ptr() as *const c_char).is_null()
            || !strstr(OSPath, "::\0".as_ptr() as *const c_char).is_null() {
            Com_Printf("WARNING: refusing to create relative path \"%s\"\n\0".as_ptr() as *const c_char, OSPath);
            return;
        }

        strlwr(OSPath);

        ofs = OSPath.add(1);
        while *ofs != 0 {
            if *ofs == PATH_SEP {
                // create the directory
                *ofs = 0;
                Sys_Mkdir(OSPath);
                *ofs = PATH_SEP;
            }
            ofs = ofs.add(1);
        }
    }
}

/*
===========
FS_SV_FOpenFileRead

===========
*/
pub fn FS_SV_FOpenFileRead(filename: *const c_char, fp: *mut fileHandle_t) -> c_int {
    let mut ospath: *mut c_char;
    let mut f: fileHandle_t;

    unsafe {
        if fs_searchpaths == core::ptr::null_mut() {
            Com_Error(ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char);
        }

        f = FS_HandleForFile();
        fsh[f as usize].zipFile = qfalse;

        Q_strncpyz(fsh[f as usize].name.as_mut_ptr(), filename, MAX_QPATH as c_int);

        // don't let sound stutter
        S_ClearSoundBuffer();

        // #ifdef _XBOX
        // ospath = FS_BuildOSPath(filename);
        // #else
        ospath = FS_BuildOSPath_full((*fs_basepath).string, filename, "\0".as_ptr() as *const c_char);
        // #endif

        // remove trailing slash
        *ospath.add(strlen(ospath).wrapping_sub(1)) = '\0' as c_char;

        if !fs_debug.is_null() && (*fs_debug).integer != 0 {
            Com_Printf("FS_SV_FOpenFileRead: %s\n\0".as_ptr() as *const c_char, ospath);
        }

        fsh[f as usize].handleFiles.file.o = fopen(ospath, "rb\0".as_ptr() as *const c_char);
        fsh[f as usize].handleSync = qfalse;
        if fsh[f as usize].handleFiles.file.o == core::ptr::null_mut() as *mut c_void {
            f = 0;
        }

        *fp = f;
        if f != 0 {
            return FS_filelength(f);
        }
        return 0;
    }
}

/*
===========
FS_FOpenFileAppend

===========
*/
pub fn FS_FOpenFileAppend(filename: *const c_char) -> fileHandle_t {
    let mut ospath: *mut c_char;
    let mut f: fileHandle_t;

    unsafe {
        if fs_searchpaths == core::ptr::null_mut() {
            Com_Error(ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char);
        }

        f = FS_HandleForFile();
        fsh[f as usize].zipFile = qfalse;

        Q_strncpyz(fsh[f as usize].name.as_mut_ptr(), filename, MAX_QPATH as c_int);

        // don't let sound stutter
        S_ClearSoundBuffer();

        // #ifdef _XBOX
        // ospath = FS_BuildOSPath(filename);
        // #else
        ospath = FS_BuildOSPath_full((*fs_basepath).string, fs_gamedir.as_ptr(), filename);
        // #endif

        if !fs_debug.is_null() && (*fs_debug).integer != 0 {
            Com_Printf("FS_FOpenFileAppend: %s\n\0".as_ptr() as *const c_char, ospath);
        }

        FS_CreatePath(ospath);
        fsh[f as usize].handleFiles.file.o = fopen(ospath, "ab\0".as_ptr() as *const c_char);
        fsh[f as usize].handleSync = qfalse;
        if fsh[f as usize].handleFiles.file.o == core::ptr::null_mut() as *mut c_void {
            f = 0;
        }
        return f;
    }
}

/*
===========
FS_FilenameCompare

Ignore case and seprator char distinctions
===========
*/
pub fn FS_FilenameCompare(s1: *const c_char, s2: *const c_char) -> qboolean {
    let mut c1: c_int;
    let mut c2: c_int;
    let mut s1_ptr = s1;
    let mut s2_ptr = s2;

    unsafe {
        loop {
            c1 = *s1_ptr as c_int;
            s1_ptr = s1_ptr.add(1);
            c2 = *s2_ptr as c_int;
            s2_ptr = s2_ptr.add(1);

            if Q_islower(c1) != 0 {
                c1 -= ('a' as c_int) - ('A' as c_int);
            }
            if Q_islower(c2) != 0 {
                c2 -= ('a' as c_int) - ('A' as c_int);
            }

            if c1 == '\\' as c_int || c1 == ':' as c_int {
                c1 = '/' as c_int;
            }
            if c2 == '\\' as c_int || c2 == ':' as c_int {
                c2 = '/' as c_int;
            }

            if c1 != c2 {
                return -1;		// strings not equal
            }
            if c1 == 0 {
                break;
            }
        }
    }

    return 0;		// strings are equal
}

// FS_Printf is a variadic function that's typically called from C code.
// Rust doesn't support C-style variadic functions directly, so this is a structural stub.
// The actual formatting would need to be done via C interop.
// This function is defined in the original C code but rarely used within files_common itself.

/*
============
FS_WriteFile

Filename are relative to the quake search path
============
*/
pub fn FS_WriteFile(qpath: *const c_char, buffer: *const c_void, size: c_int) {
    let mut f: fileHandle_t;

    unsafe {
        if fs_searchpaths == core::ptr::null_mut() {
            Com_Error(ERR_FATAL, "Filesystem call made without initialization\n\0".as_ptr() as *const c_char);
        }

        if qpath == core::ptr::null_mut() as *const c_char || buffer == core::ptr::null_mut() as *const c_void {
            Com_Error(ERR_FATAL, "FS_WriteFile: NULL parameter\0".as_ptr() as *const c_char);
        }

        f = FS_FOpenFileWrite(qpath);
        if f == 0 {
            Com_Printf("Failed to open %s\n\0".as_ptr() as *const c_char, qpath);
            return;
        }

        FS_Write(buffer, size, f);

        FS_FCloseFile(f);
    }
}

/*
================
FS_Shutdown

Frees all resources and closes all files
================
*/
pub fn FS_Shutdown() {
    let mut p: *mut searchpath_t;
    let mut next: *mut searchpath_t;
    let mut i: c_int;

    unsafe {
        for i in 0..(MAX_FILE_HANDLES as c_int) {
            if fsh[i as usize].fileSize != 0 {
                FS_FCloseFile(i);
            }
        }

        // free everything
        p = fs_searchpaths;
        while p != core::ptr::null_mut() {
            next = (*p).next;

            if !(*p).pack.is_null() {
                // #ifndef _XBOX
                unzClose((*(*p).pack).handle);
                // #endif
                Z_Free((*(*p).pack).buildBuffer as *mut c_void);
                Z_Free((*p).pack as *mut c_void);
            }
            if !(*p).dir.is_null() {
                Z_Free((*p).dir as *mut c_void);
            }
            Z_Free(p as *mut c_void);
            p = next;
        }

        // any FS_ calls will now be an error until reinitialized
        fs_searchpaths = core::ptr::null_mut();

        Cmd_RemoveCommand("path\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand("dir\0".as_ptr() as *const c_char);
        Cmd_RemoveCommand("touchFile\0".as_ptr() as *const c_char);

        initialized = qfalse;
    }
}

/*
================
FS_InitFilesystem

Called only at inital startup, not when the filesystem
is resetting due to a game change
================
*/
pub fn FS_InitFilesystem() {
    // allow command line parms to override our defaults
    // we don't have to specially handle this, because normal command
    // line variable sets happen before the filesystem
    // has been initialized
    //
    // UPDATE: BTO (VV)
    // we have to specially handle this, because normal command
    // line variable sets don't happen until after the filesystem
    // has already been initialized

    unsafe {
        Com_StartupVariable("fs_cdpath\0".as_ptr() as *const c_char);
        Com_StartupVariable("fs_basepath\0".as_ptr() as *const c_char);
        Com_StartupVariable("fs_game\0".as_ptr() as *const c_char);
        Com_StartupVariable("fs_copyfiles\0".as_ptr() as *const c_char);
        Com_StartupVariable("fs_restrict\0".as_ptr() as *const c_char);

        // try to start up normally
        FS_Startup(BASEGAME.as_ptr() as *const c_char);
        initialized = qtrue;

        // see if we are going to allow add-ons
        FS_SetRestrictions();

        // if we can't find default.cfg, assume that the paths are
        // busted and error out now, rather than getting an unreadable
        // graphics screen when the font fails to load
        if FS_ReadFile("default.cfg\0".as_ptr() as *const c_char, core::ptr::null_mut()) <= 0 {
            Com_Error(ERR_FATAL, "Couldn't load default.cfg\0".as_ptr() as *const c_char);
        }
    }
}

pub fn FS_Flush(f: fileHandle_t) {
    unsafe {
        fflush(fsh[f as usize].handleFiles.file.o);
    }
}
