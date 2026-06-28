//! `tags.h`
//!
//! Do NOT include-protect this file, or add any fields or labels, because it's
//! included within enums and tables.
//!
//! These macro args get "TAG_" prepended on them for enum purposes, and appear
//! as literal strings for "meminfo" command.

#![allow(non_camel_case_types)]

use core::ffi::c_int;

macro_rules! tags_h_non_xbox {
    ($TAGDEF:ident) => {
        $TAGDEF!(ALL),
        $TAGDEF!(BOTLIB),
        $TAGDEF!(CLIENTS),
        $TAGDEF!(BOTGAME),
        $TAGDEF!(DOWNLOAD),
        $TAGDEF!(GENERAL),
        $TAGDEF!(CLIPBOARD),
        $TAGDEF!(SND_MP3STREAMHDR),
        $TAGDEF!(SND_DYNAMICMUSIC),
        $TAGDEF!(BSP_DISKIMAGE),
        $TAGDEF!(VM),
        $TAGDEF!(SPECIAL_MEM_TEST),
        $TAGDEF!(HUNK_MARK1),
        $TAGDEF!(HUNK_MARK2),
        $TAGDEF!(EVENT),
        $TAGDEF!(FILESYS),
        $TAGDEF!(GHOUL2),
        $TAGDEF!(GHOUL2_GORE),
        $TAGDEF!(LISTFILES),
        $TAGDEF!(AMBIENTSET),
        $TAGDEF!(STATIC),
        $TAGDEF!(SMALL),
        $TAGDEF!(MODEL_MD3),
        $TAGDEF!(MODEL_GLM),
        $TAGDEF!(MODEL_GLA),
        $TAGDEF!(ICARUS),
        $TAGDEF!(ICARUS2),
        $TAGDEF!(ICARUS3),
        $TAGDEF!(ICARUS4),
        $TAGDEF!(ICARUS5),
        $TAGDEF!(SHADERTEXT),
        $TAGDEF!(SND_RAWDATA),
        $TAGDEF!(TEMP_WORKSPACE),
        $TAGDEF!(TEMP_PNG),
        $TAGDEF!(TEXTPOOL),
        $TAGDEF!(IMAGE_T),
        $TAGDEF!(INFLATE),
        $TAGDEF!(DEFLATE),
        $TAGDEF!(BSP),
        $TAGDEF!(GRIDMESH),
        $TAGDEF!(POINTCACHE),
        $TAGDEF!(TERRAIN),
        $TAGDEF!(R_TERRAIN),
        $TAGDEF!(RESAMPLE),
        $TAGDEF!(CM_TERRAIN),
        $TAGDEF!(CM_TERRAIN_TEMP),
        $TAGDEF!(TEMP_IMAGE),
        $TAGDEF!(VM_ALLOCATED),
        $TAGDEF!(TEMP_HUNKALLOC),
        $TAGDEF!(COUNT)
    };
}

pub(crate) use tags_h_non_xbox;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum memtag_e {
    TAG_ALL,
    TAG_BOTLIB,
    TAG_CLIENTS, // Memory used for client info
    TAG_BOTGAME,
    TAG_DOWNLOAD,          // used by the downloading system
    TAG_GENERAL,
    TAG_CLIPBOARD,
    TAG_SND_MP3STREAMHDR,  // specific MP3 struct for decoding (about 18..22K each?), not the actual MP3 binary
    TAG_SND_DYNAMICMUSIC,  // in-mem MP3 files
    TAG_BSP_DISKIMAGE,     // temp during loading, to save both server and renderer fread()ing the same file. Only used if not low physical memory (currently 96MB)
    TAG_VM,                // stuff for VM, may be zapped later?
    TAG_SPECIAL_MEM_TEST,  // special usage for testing z_malloc recover only
    TAG_HUNK_MARK1,        // hunk allocations before the mark is set
    TAG_HUNK_MARK2,        // hunk allocations after the mark is set
    TAG_EVENT,
    TAG_FILESYS,           // general filesystem usage
    TAG_GHOUL2,            // Ghoul2 stuff
    TAG_GHOUL2_GORE,       // Ghoul2 gore stuff
    TAG_LISTFILES,         // for "*.blah" lists
    TAG_AMBIENTSET,
    TAG_STATIC,            // special usage for 1-byte allocations from 0..9 to avoid CopyString() slowdowns during cvar value copies
    TAG_SMALL,             // used by S_Malloc, but probably more of a hint now. Will be dumped later
    TAG_MODEL_MD3,         // specific model types' disk images
    TAG_MODEL_GLM,         //       "
    TAG_MODEL_GLA,         //       "
    TAG_ICARUS,            // Memory used internally by the Icarus scripting system
    // sorry, I don't want to have to keep adding these and recompiling, so there may be more than I need
    TAG_ICARUS2,           // for debugging mem leaks in icarus -rww
    TAG_ICARUS3,           // for debugging mem leaks in icarus -rww
    TAG_ICARUS4,           // for debugging mem leaks in icarus -rww
    TAG_ICARUS5,           // for debugging mem leaks in icarus -rww
    TAG_SHADERTEXT,
    TAG_SND_RAWDATA,       // raw sound data, either MP3 or WAV
    TAG_TEMP_WORKSPACE,    // anything like file loading or image workspace that's only temporary
    TAG_TEMP_PNG,          // image workspace that's only temporary
    TAG_TEXTPOOL,          // for some special text-pool class thingy
    TAG_IMAGE_T,           // an image_t struct (no longer on the hunk because of cached texture stuff)
    TAG_INFLATE,           // Temp memory used by zlib32
    TAG_DEFLATE,           // Temp memory used by zlib32//  TAGDEF(SOUNDPOOL), pool of mem for the sound system
    TAG_BSP,               // guess.
    TAG_GRIDMESH,          // some specific temp workspace that only seems to be in the MP codebase

    // rwwRMG - following:
    TAG_POINTCACHE,        // weather system
    TAG_TERRAIN,           // RMG terrain management
    TAG_R_TERRAIN,         // terrain renderer
    TAG_RESAMPLE,          // terrain heightmap resampling (I think)
    TAG_CM_TERRAIN,        // common terrain data management
    TAG_CM_TERRAIN_TEMP,   // temporary terrain allocations
    TAG_TEMP_IMAGE,        // temporary allocations for image manipulation

    TAG_VM_ALLOCATED,      // allocated by game or cgame via memory shifting

    TAG_TEMP_HUNKALLOC,
    TAG_COUNT,
}

pub type memtag_t = c_int;

pub const TAG_ALL: memtag_t = memtag_e::TAG_ALL as memtag_t;
pub const TAG_BOTLIB: memtag_t = memtag_e::TAG_BOTLIB as memtag_t;
pub const TAG_CLIENTS: memtag_t = memtag_e::TAG_CLIENTS as memtag_t;
pub const TAG_BOTGAME: memtag_t = memtag_e::TAG_BOTGAME as memtag_t;
pub const TAG_DOWNLOAD: memtag_t = memtag_e::TAG_DOWNLOAD as memtag_t;
pub const TAG_GENERAL: memtag_t = memtag_e::TAG_GENERAL as memtag_t;
pub const TAG_CLIPBOARD: memtag_t = memtag_e::TAG_CLIPBOARD as memtag_t;
pub const TAG_SND_MP3STREAMHDR: memtag_t = memtag_e::TAG_SND_MP3STREAMHDR as memtag_t;
pub const TAG_SND_DYNAMICMUSIC: memtag_t = memtag_e::TAG_SND_DYNAMICMUSIC as memtag_t;
pub const TAG_BSP_DISKIMAGE: memtag_t = memtag_e::TAG_BSP_DISKIMAGE as memtag_t;
pub const TAG_VM: memtag_t = memtag_e::TAG_VM as memtag_t;
pub const TAG_SPECIAL_MEM_TEST: memtag_t = memtag_e::TAG_SPECIAL_MEM_TEST as memtag_t;
pub const TAG_HUNK_MARK1: memtag_t = memtag_e::TAG_HUNK_MARK1 as memtag_t;
pub const TAG_HUNK_MARK2: memtag_t = memtag_e::TAG_HUNK_MARK2 as memtag_t;
pub const TAG_EVENT: memtag_t = memtag_e::TAG_EVENT as memtag_t;
pub const TAG_FILESYS: memtag_t = memtag_e::TAG_FILESYS as memtag_t;
pub const TAG_GHOUL2: memtag_t = memtag_e::TAG_GHOUL2 as memtag_t;
pub const TAG_GHOUL2_GORE: memtag_t = memtag_e::TAG_GHOUL2_GORE as memtag_t;
pub const TAG_LISTFILES: memtag_t = memtag_e::TAG_LISTFILES as memtag_t;
pub const TAG_AMBIENTSET: memtag_t = memtag_e::TAG_AMBIENTSET as memtag_t;
pub const TAG_STATIC: memtag_t = memtag_e::TAG_STATIC as memtag_t;
pub const TAG_SMALL: memtag_t = memtag_e::TAG_SMALL as memtag_t;
pub const TAG_MODEL_MD3: memtag_t = memtag_e::TAG_MODEL_MD3 as memtag_t;
pub const TAG_MODEL_GLM: memtag_t = memtag_e::TAG_MODEL_GLM as memtag_t;
pub const TAG_MODEL_GLA: memtag_t = memtag_e::TAG_MODEL_GLA as memtag_t;
pub const TAG_ICARUS: memtag_t = memtag_e::TAG_ICARUS as memtag_t;
pub const TAG_ICARUS2: memtag_t = memtag_e::TAG_ICARUS2 as memtag_t;
pub const TAG_ICARUS3: memtag_t = memtag_e::TAG_ICARUS3 as memtag_t;
pub const TAG_ICARUS4: memtag_t = memtag_e::TAG_ICARUS4 as memtag_t;
pub const TAG_ICARUS5: memtag_t = memtag_e::TAG_ICARUS5 as memtag_t;
pub const TAG_SHADERTEXT: memtag_t = memtag_e::TAG_SHADERTEXT as memtag_t;
pub const TAG_SND_RAWDATA: memtag_t = memtag_e::TAG_SND_RAWDATA as memtag_t;
pub const TAG_TEMP_WORKSPACE: memtag_t = memtag_e::TAG_TEMP_WORKSPACE as memtag_t;
pub const TAG_TEMP_PNG: memtag_t = memtag_e::TAG_TEMP_PNG as memtag_t;
pub const TAG_TEXTPOOL: memtag_t = memtag_e::TAG_TEXTPOOL as memtag_t;
pub const TAG_IMAGE_T: memtag_t = memtag_e::TAG_IMAGE_T as memtag_t;
pub const TAG_INFLATE: memtag_t = memtag_e::TAG_INFLATE as memtag_t;
pub const TAG_DEFLATE: memtag_t = memtag_e::TAG_DEFLATE as memtag_t;
pub const TAG_BSP: memtag_t = memtag_e::TAG_BSP as memtag_t;
pub const TAG_GRIDMESH: memtag_t = memtag_e::TAG_GRIDMESH as memtag_t;
pub const TAG_POINTCACHE: memtag_t = memtag_e::TAG_POINTCACHE as memtag_t;
pub const TAG_TERRAIN: memtag_t = memtag_e::TAG_TERRAIN as memtag_t;
pub const TAG_R_TERRAIN: memtag_t = memtag_e::TAG_R_TERRAIN as memtag_t;
pub const TAG_RESAMPLE: memtag_t = memtag_e::TAG_RESAMPLE as memtag_t;
pub const TAG_CM_TERRAIN: memtag_t = memtag_e::TAG_CM_TERRAIN as memtag_t;
pub const TAG_CM_TERRAIN_TEMP: memtag_t = memtag_e::TAG_CM_TERRAIN_TEMP as memtag_t;
pub const TAG_TEMP_IMAGE: memtag_t = memtag_e::TAG_TEMP_IMAGE as memtag_t;
pub const TAG_VM_ALLOCATED: memtag_t = memtag_e::TAG_VM_ALLOCATED as memtag_t;
pub const TAG_TEMP_HUNKALLOC: memtag_t = memtag_e::TAG_TEMP_HUNKALLOC as memtag_t;
pub const TAG_COUNT: memtag_t = memtag_e::TAG_COUNT as memtag_t;

pub const TAG_NAMES: [&str; TAG_COUNT as usize + 1] = [
    "ALL",
    "BOTLIB",
    "CLIENTS",
    "BOTGAME",
    "DOWNLOAD",
    "GENERAL",
    "CLIPBOARD",
    "SND_MP3STREAMHDR",
    "SND_DYNAMICMUSIC",
    "BSP_DISKIMAGE",
    "VM",
    "SPECIAL_MEM_TEST",
    "HUNK_MARK1",
    "HUNK_MARK2",
    "EVENT",
    "FILESYS",
    "GHOUL2",
    "GHOUL2_GORE",
    "LISTFILES",
    "AMBIENTSET",
    "STATIC",
    "SMALL",
    "MODEL_MD3",
    "MODEL_GLM",
    "MODEL_GLA",
    "ICARUS",
    "ICARUS2",
    "ICARUS3",
    "ICARUS4",
    "ICARUS5",
    "SHADERTEXT",
    "SND_RAWDATA",
    "TEMP_WORKSPACE",
    "TEMP_PNG",
    "TEXTPOOL",
    "IMAGE_T",
    "INFLATE",
    "DEFLATE",
    "BSP",
    "GRIDMESH",
    "POINTCACHE",
    "TERRAIN",
    "R_TERRAIN",
    "RESAMPLE",
    "CM_TERRAIN",
    "CM_TERRAIN_TEMP",
    "TEMP_IMAGE",
    "VM_ALLOCATED",
    "TEMP_HUNKALLOC",
    "COUNT",
];

//////////////// eof //////////////
