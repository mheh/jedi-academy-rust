/*****************************************************************************
 * name:		files.c
 *
 * desc:		handle based filesystem for Quake III Arena
 *
 *****************************************************************************/

//Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"

// #include "../client/client.h"
// #include "../zlib32/zip.h"
// #include "unzip.h"
// #include "files.h"

// #include <windows.h> //rww - included to make fs_copyfiles 2 related functions happy.
// #include "platform.h"

use core::ffi::{c_char, c_int, c_void};

/*
=============================================================================

QUAKE3 FILESYSTEM

All of Quake's data access is through a hierarchical file system, but the contents of
the file system can be transparently merged from several sources.

A "qpath" is a reference to game file data.  MAX_ZPATH is 256 characters, which must include
a terminating zero. "..", "\\", and ":" are explicitly illegal in qpaths to prevent any
references outside the quake directory system.

The "base path" is the path to the directory holding all the game directories and usually
the executable.  It defaults to ".", but can be overridden with a "+set fs_basepath c:\quake3"
command line to allow code debugging in a different directory.  Basepath cannot
be modified at all after startup.  Any files that are created (demos, screenshots,
etc) will be created reletive to the base path, so base path should usually be writable.

The "cd path" is the path to an alternate hierarchy that will be searched if a file
is not located in the base path.  A user can do a partial install that copies some
data to a base path created on their hard drive and leave the rest on the cd.  Files
are never writen to the cd path.  It defaults to a value set by the installer, like
"e:\quake3", but it can be overridden with "+set ds_cdpath g:\quake3".

If a user runs the game directly from a CD, the base path would be on the CD.  This
should still function correctly, but all file writes will fail (harmlessly).

The "home path" is the path used for all write access. On win32 systems we have "base path"
== "home path", but on *nix systems the base installation is usually readonly, and
"home path" points to ~/.q3a or similar

The user can also install custom mods and content in "home path", so it should be searched
along with "home path" and "cd path" for game content.


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
zip files of the form "pak0.pk3", "pak1.pk3", etc.  Zip files are searched in decending order
from the highest number to the lowest, and will always take precedence over the filesystem.
This allows a pk3 distributed as a patch to override all existing data.

Because we will have updated executables freely available online, there is no point to
trying to restrict demo / oem versions of the game with code changes.  Demo / oem versions
should be exactly the same executables as release versions, but with different data that
automatically restricts where game media can come from to prevent add-ons from working.

After the paths are initialized, quake will look for the product.txt file.  If not
found and verified, the game will run in restricted mode.  In restricted mode, only
files contained in demoq3/pak0.pk3 will be available for loading, and only if the zip header is
verified to not have been modified.  A single exception is made for jampconfig.cfg.  Files
can still be written out in restricted mode, so screenshots and demos are allowed.
Restricted mode can be tested by setting "+set fs_restrict 1" on the command line, even
if there is a valid product.txt under the basepath or cdpath.

If not running in restricted mode, and a file is not found in any local filesystem,
an attempt will be made to download it and save it under the base path.

If the "fs_copyfiles" cvar is set to 1, then every time a file is sourced from the cd
path, it will be copied over to the base path.  This is a development aid to help build
test releases and to copy working sets over slow network links.
(If set to 2, copying will only take place if the two filetimes are NOT EQUAL)

File search order: when FS_FOpenFileRead gets called it will go through the fs_searchpaths
structure and stop on the first successful hit. fs_searchpaths is built with successive
calls to FS_AddGameDirectory

Additionaly, we search in several subdirectories:
current game is the current mode
base game is a variable to allow mods based on other mods
(such as base + missionpack content combination in a mod for instance)
BASEGAME is the hardcoded base game ("base")

e.g. the qpath "sound/newstuff/test.wav" would be searched for in the following places:

home path + current game's zip files
home path + current game's directory
base path + current game's zip files
base path + current game's directory
cd path + current game's zip files
cd path + current game's directory

home path + base game's zip file
home path + base game's directory
base path + base game's zip file
base path + base game's directory
cd path + base game's zip file
cd path + base game's directory

home path + BASEGAME's zip file
home path + BASEGAME's directory
base path + BASEGAME's zip file
base path + BASEGAME's directory
cd path + BASEGAME's zip file
cd path + BASEGAME's directory

server download, to be written to home path + current game's directory


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

When building a pak file, make sure a jampconfig.cfg isn't present in it,
or configs will never get loaded from disk!

  todo:

  downloading (outside fs?)
  game directory passing and restarting

=============================================================================

*/

// Constants and type stubs for structural coherence
const MAX_OSPATH: usize = 256;
const MAX_FILE_HANDLES: usize = 64;
const MAX_SEARCH_PATHS: usize = 4096;
const BASEGAME: &str = "base";
const MAXPRINTMSG: usize = 4096;

// ERR_* constants for Com_Error
const ERR_FATAL: c_int = 1;
const ERR_DROP: c_int = 2;

type fileHandle_t = c_int;
type qboolean = c_int;

// Stub structures for unported dependencies
#[repr(C)]
pub struct unzFile {
    _opaque: *mut c_void,
}

#[repr(C)]
pub struct pack_t {
    pub handle: *mut unzFile,
    pub buildBuffer: *mut c_void,
}

#[repr(C)]
pub struct directory_t {
    _opaque: *mut c_void,
}

#[repr(C)]
pub struct fileHandleData_t {
    pub handleFiles: fileHandleFiles_t,
    pub fileSize: c_int,
}

#[repr(C)]
pub struct fileHandleFiles_t {
    pub file: fileHandle_c_t,
}

#[repr(C)]
pub struct fileHandle_c_t {
    pub o: *mut c_void,
}

#[repr(C)]
pub struct searchpath_t {
    pub next: *mut searchpath_t,
    pub pack: *mut pack_t,
    pub dir: *mut directory_t,
}

#[repr(C)]
pub struct cvar_t {
    pub string: *mut c_char,
}

// Globals
pub static mut fs_gamedir: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];	// this will be a single file name with no separators
pub static mut fs_debug: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_homepath: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_basepath: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_basegame: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_cdpath: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_copyfiles: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_gamedirvar: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_restrict: *mut cvar_t = core::ptr::null_mut();
pub static mut fs_dirbeforepak: *mut cvar_t = core::ptr::null_mut(); //rww - when building search path, keep directories at top and insert pk3's under them
pub static mut fs_searchpaths: *mut searchpath_t = core::ptr::null_mut();
pub static mut fs_readCount: c_int = 0;			// total bytes read
pub static mut fs_loadCount: c_int = 0;			// total files read
pub static mut fs_loadStack: c_int = 0;			// total files in memory
pub static mut fs_packFiles: c_int = 0;			// total number of files in packs

pub static mut fs_fakeChkSum: c_int = 0;
pub static mut fs_checksumFeed: c_int = 0;

pub static mut fsh: [fileHandleData_t; MAX_FILE_HANDLES] = [fileHandleData_t {
    handleFiles: fileHandleFiles_t {
        file: fileHandle_c_t {
            o: core::ptr::null_mut(),
        }
    },
    fileSize: 0,
}; MAX_FILE_HANDLES];


// never load anything from pk3 files that are not present at the server when pure
pub static mut fs_numServerPaks: c_int = 0;
pub static mut fs_serverPaks: [c_int; MAX_SEARCH_PATHS] = [0; MAX_SEARCH_PATHS];				// checksums
pub static mut fs_serverPakNames: [*mut c_char; MAX_SEARCH_PATHS] = [core::ptr::null_mut(); MAX_SEARCH_PATHS];			// pk3 names

// only used for autodownload, to make sure the client has at least
// all the pk3 files that are referenced at the server side
pub static mut fs_numServerReferencedPaks: c_int = 0;
pub static mut fs_serverReferencedPaks: [c_int; MAX_SEARCH_PATHS] = [0; MAX_SEARCH_PATHS];			// checksums
pub static mut fs_serverReferencedPakNames: [*mut c_char; MAX_SEARCH_PATHS] = [core::ptr::null_mut(); MAX_SEARCH_PATHS];		// pk3 names

// last valid game folder used
pub static mut lastValidBase: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
pub static mut lastValidGame: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];

#[cfg(feature = "FS_MISSING")]
pub static mut missingFiles: *mut core::ffi::c_FILE = core::ptr::null_mut();

pub static mut initialized: qboolean = 0; // qfalse

// Extern C declarations for functions from other modules
extern "C" {
    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_sprintf(dest: *mut c_char, destsize: c_int, fmt: *const c_char, ...);
    pub fn Com_StartupVariable(match_: *const c_char);
    pub fn FS_Startup(gameName: *const c_char);
    pub fn FS_SetRestrictions();
    pub fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    pub fn FS_FOpenFileWrite(filename: *const c_char) -> fileHandle_t;
    pub fn FS_Write(buffer: *const c_void, len: c_int, h: fileHandle_t);
    pub fn FS_FCloseFile(f: fileHandle_t);
    pub fn unzClose(file: *mut unzFile) -> c_int;
    pub fn Z_Free(ptr: *mut c_void);
    pub fn Cmd_RemoveCommand(cmd_name: *const c_char);
    pub fn strlen(s: *const c_char) -> usize;
    pub fn vsprintf(dest: *mut c_char, fmt: *const c_char, arg: *mut core::ffi::va_list) -> c_int;
}

/*
	Extra utility for checking that FS is up and running
*/
pub fn FS_CheckInit() {
	unsafe {
		if initialized == 0 {
			Com_Error( ERR_FATAL, b"Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
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
		if (*core::ptr::addr_of!(fs_searchpaths)).is_null() {
			return 0; // qfalse
		} else {
			return 1; // qtrue
		}
	}
}

/*
=================
FS_LoadStack
return load stack
=================
*/
pub fn FS_LoadStack() -> c_int {
	unsafe {
		return *core::ptr::addr_of!(fs_loadStack);
	}
}

pub fn FS_HandleForFile() -> fileHandle_t {
	let mut i: c_int;

	unsafe {
		for i in 1..MAX_FILE_HANDLES as c_int {
			if (*core::ptr::addr_of!(fsh[i as usize]).handleFiles.file.o).is_null() {
				return i;
			}
		}
	}
	unsafe {
		Com_Error( ERR_DROP, b"FS_HandleForFile: none free\0".as_ptr() as *const c_char );
	}
	return 0;
}

/*
====================
FS_ReplaceSeparators

Fix things up differently for win/unix/mac
====================
*/
pub fn FS_ReplaceSeparators( path: *mut c_char ) {
	let mut s: *mut c_char;

	unsafe {
		s = path;
		while *s != 0 {
			if *s == b'/' as c_char || *s == b'\\' as c_char {
				*s = core::ffi::PATH_SEP as c_char;
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
pub fn FS_BuildOSPath( qpath: *const c_char ) -> *mut c_char {
	let mut temp: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
	static mut ospath: [[c_char; MAX_OSPATH]; 2] = [[0; MAX_OSPATH]; 2];
	static mut toggle: c_int = 0;

	unsafe {
		toggle ^= 1;		// flip-flop to allow two returns without clash

		// Fix for filenames that are given to FS with a leading "/" (/botfiles/Foo)
		let mut qpath_adjusted = qpath;
		if *qpath == b'\\' as c_char || *qpath == b'/' as c_char {
			qpath_adjusted = qpath.add(1);
		}

		// FIXME VVFIXME Holy crap this is wrong.
		//	Com_sprintf( temp, sizeof(temp), "/%s/%s", fs_gamedirvar->string, qpath );
		Com_sprintf( temp.as_mut_ptr(), MAX_OSPATH as c_int, b"/%s/%s\0".as_ptr() as *const c_char, b"base\0".as_ptr() as *const c_char, qpath_adjusted );

		FS_ReplaceSeparators( temp.as_mut_ptr() );
		Com_sprintf( ospath[toggle as usize].as_mut_ptr(), MAX_OSPATH as c_int, b"%s%s\0".as_ptr() as *const c_char,
			(*core::ptr::addr_of!(fs_basepath)).string, temp.as_ptr() );

		return ospath[toggle as usize].as_mut_ptr();
	}
}

pub fn FS_BuildOSPath_3( base: *const c_char, game: *const c_char, qpath: *const c_char ) -> *mut c_char {
	let mut temp: [c_char; MAX_OSPATH] = [0; MAX_OSPATH];
	static mut ospath: [[c_char; MAX_OSPATH]; 4] = [[0; MAX_OSPATH]; 4];
	static mut toggle: c_int = 0;

	unsafe {
		//pre-fs_cf2
		//toggle ^= 1;		// flip-flop to allow two returns without clash
		//post-fs_cf2
		toggle = (toggle + 1) & 3;	// allows four returns without clash (increased from 2 during fs_copyfiles 2 enhancement)

		let mut game_to_use = game;
		if game.is_null() || *game == 0 {
			game_to_use = core::ptr::addr_of!(fs_gamedir) as *const c_char;
		}

		Com_sprintf( temp.as_mut_ptr(), MAX_OSPATH as c_int, b"/%s/%s\0".as_ptr() as *const c_char, game_to_use, qpath );
		FS_ReplaceSeparators( temp.as_mut_ptr() );
		Com_sprintf( ospath[toggle as usize].as_mut_ptr(), MAX_OSPATH as c_int, b"%s%s\0".as_ptr() as *const c_char, base, temp.as_ptr() );

		return ospath[toggle as usize].as_mut_ptr();
	}
}

/*
===========
FS_FilenameCompare

Ignore case and seprator char distinctions
===========
*/
pub fn FS_FilenameCompare( s1: *const c_char, s2: *const c_char ) -> qboolean {
	let mut c1: c_int;
	let mut c2: c_int;
	let mut s1_ptr = s1;
	let mut s2_ptr = s2;

	unsafe {
		loop {
			c1 = *s1_ptr as c_int;
			c2 = *s2_ptr as c_int;
			s1_ptr = s1_ptr.add(1);
			s2_ptr = s2_ptr.add(1);

			if c1 >= b'a' as c_int && c1 <= b'z' as c_int {
				c1 -= (b'a' as c_int - b'A' as c_int);
			}
			if c2 >= b'a' as c_int && c2 <= b'z' as c_int {
				c2 -= (b'a' as c_int - b'A' as c_int);
			}

			if c1 == b'\\' as c_int || c1 == b':' as c_int {
				c1 = b'/' as c_int;
			}
			if c2 == b'\\' as c_int || c2 == b':' as c_int {
				c2 = b'/' as c_int;
			}

			if c1 != c2 {
				return -1i32;		// strings not equal
			}
			if c1 == 0 {
				break;
			}
		}
	}

	return 0i32;		// strings are equal
}

#[allow(non_snake_case)]
pub fn FS_Printf( h: fileHandle_t, fmt: *const c_char, args: ... ) {
	let mut msg: [c_char; MAXPRINTMSG] = [0; MAXPRINTMSG];
	let mut argptr: core::ffi::va_list;

	unsafe {
		core::ffi::va_start(&mut argptr, fmt);
		vsprintf (msg.as_mut_ptr(), fmt, &mut argptr);
		core::ffi::va_end(&mut argptr);

		FS_Write(msg.as_ptr() as *const c_void, strlen(msg.as_ptr()) as c_int, h);
	}
}

/*
======================================================================================

CONVENIENCE FUNCTIONS FOR ENTIRE FILES

======================================================================================
*/

/*
============
FS_WriteFile

Filename are reletive to the quake search path
============
*/
pub fn FS_WriteFile( qpath: *const c_char, buffer: *const c_void, size: c_int ) {
	let mut f: fileHandle_t;

	unsafe {
		if (*core::ptr::addr_of!(fs_searchpaths)).is_null() {
			Com_Error( ERR_FATAL, b"Filesystem call made without initialization\n\0".as_ptr() as *const c_char );
		}

		if qpath.is_null() || buffer.is_null() {
			Com_Error( ERR_FATAL, b"FS_WriteFile: NULL parameter\0".as_ptr() as *const c_char );
		}

		f = FS_FOpenFileWrite( qpath );
		if f == 0 {
			Com_Printf( b"Failed to open %s\n\0".as_ptr() as *const c_char, qpath );
			return;
		}

		FS_Write( buffer, size, f );

		FS_FCloseFile( f );
	}
}

/*
================
FS_Shutdown

Frees all resources and closes all files
================
*/
pub fn FS_Shutdown( closemfp: qboolean ) {
	let mut p: *mut searchpath_t;
	let mut next: *mut searchpath_t;
	let mut i: c_int;

	unsafe {
		for i in 0..MAX_FILE_HANDLES as c_int {
			if (*core::ptr::addr_of!(fsh[i as usize])).fileSize != 0 {
				FS_FCloseFile(i);
			}
		}

		// free everything
		p = *core::ptr::addr_of!(fs_searchpaths);
		while !p.is_null() {
			next = (*p).next;

			if !(*p).pack.is_null() {
	#[cfg(not(target_os = "xbox"))]
				{
					unzClose((*(*p).pack).handle);
				}
				Z_Free( (*(*p).pack).buildBuffer );
				Z_Free( (*p).pack as *mut c_void );
			}
			if !(*p).dir.is_null() {
				Z_Free( (*p).dir as *mut c_void );
			}
			Z_Free( p as *mut c_void );
			p = next;
		}

		// any FS_ calls will now be an error until reinitialized
		*core::ptr::addr_of_mut!(fs_searchpaths) = core::ptr::null_mut();

		Cmd_RemoveCommand( b"path\0".as_ptr() as *const c_char );
		Cmd_RemoveCommand( b"dir\0".as_ptr() as *const c_char );
		Cmd_RemoveCommand( b"fdir\0".as_ptr() as *const c_char );
		Cmd_RemoveCommand( b"touchFile\0".as_ptr() as *const c_char );

		#[cfg(feature = "FS_MISSING")]
		{
			if closemfp != 0 {
				// fclose(missingFiles); - can't call fclose directly, need libc binding
			}
		}
	}
}

/*
================
FS_InitFilesystem

Called only at inital startup, not when the filesystem
is resetting due to a game change
================
*/
pub fn FS_InitFilesystem( ) {
	unsafe {
		// allow command line parms to override our defaults
		// we have to specially handle this, because normal command
		// line variable sets don't happen until after the filesystem
		// has already been initialized
		Com_StartupVariable( b"fs_cdpath\0".as_ptr() as *const c_char );
		Com_StartupVariable( b"fs_basepath\0".as_ptr() as *const c_char );
		Com_StartupVariable( b"fs_homepath\0".as_ptr() as *const c_char );
		Com_StartupVariable( b"fs_game\0".as_ptr() as *const c_char );
		Com_StartupVariable( b"fs_copyfiles\0".as_ptr() as *const c_char );
		Com_StartupVariable( b"fs_restrict\0".as_ptr() as *const c_char );

		// try to start up normally
		FS_Startup( b"base\0".as_ptr() as *const c_char );
		*core::ptr::addr_of_mut!(initialized) = 1; // qtrue

		// see if we are going to allow add-ons
		FS_SetRestrictions();

		// if we can't find default.cfg, assume that the paths are
		// busted and error out now, rather than getting an unreadable
		// graphics screen when the font fails to load
		if FS_ReadFile( b"mpdefault.cfg\0".as_ptr() as *const c_char, core::ptr::null_mut() ) <= 0 {
			Com_Error( ERR_FATAL, b"Couldn't load mpdefault.cfg\0".as_ptr() as *const c_char );
			// bk001208 - SafeMode see below, FIXME?
		}

		Q_strncpyz(core::ptr::addr_of_mut!(lastValidBase) as *mut c_char, (*core::ptr::addr_of!(fs_basepath)).string, MAX_OSPATH as c_int);
		Q_strncpyz(core::ptr::addr_of_mut!(lastValidGame) as *mut c_char, (*core::ptr::addr_of!(fs_gamedirvar)).string, MAX_OSPATH as c_int);

		  // bk001208 - SafeMode see below, FIXME?
	}
}
