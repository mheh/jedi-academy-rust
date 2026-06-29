// Filename: tags.h

// do NOT include-protect this file, or add any fields or labels, because it's included within enums and tables
//
// these macro args get "TAG_" prepended on them for enum purposes, and appear as literal strings for "meminfo" command

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Tag {
	ALL,
	HUNKALLOC,					// mem that was formerly from the hunk AFTER the SetMark (ie discarded during vid_reset)
	HUNKMISCMODELS,			// sub-hunk alloc to track misc models
	FILESYS,					// general filesystem usage
	EVENT,
	CLIPBOARD,
	LISTFILES,					// for "*.blah" lists
	AMBIENTSET,
	G_ALLOC,					// used by G_Alloc()
	CLIENTS,					// Memory used for client info
	STATIC,						// special usage for 1-byte allocations from 0..9 to avoid CopyString() slowdowns during cvar value copies
	SMALL,						// used by S_Malloc, but probably more of a hint now. Will be dumped later
	MODEL,						// general model usage), includes header-struct-only stuff like 'model_t'
	MODEL_MD3,					// specific model types' disk images
	MODEL_GLM,					//	   "
	MODEL_GLA,					//	   "
	ICARUS,						// Memory used internally by the Icarus scripting system
	IMAGE_T,					// an image_t struct (no longer on the hunk because of cached texture stuff)
	TEMP_WORKSPACE,				// anything like file loading or image workspace that's only temporary
	TEMP_TGA,					// image workspace that's only temporary
	TEMP_JPG,					// image workspace that's only temporary
	TEMP_PNG,					// image workspace that's only temporary
	SND_MP3STREAMHDR,			// specific MP3 struct for decoding (about 18..22K each?), not the actual MP3 binary
	SND_DYNAMICMUSIC,			// in-mem MP3 files
	SND_RAWDATA,				// raw sound data, either MP3 or WAV
	GHOUL2,						// Ghoul2 stuff
	BSP,						// guess.
	BSP_DISKIMAGE,				// temp during loading, to save both server and renderer fread()ing the same file. Only used if not low physical memory (currently 96MB)
	GP2,						// generic parser 2
	SPECIAL_MEM_TEST,			// special usage in one function only!!!!!!
	ANIMATION_CFG,				// may as well keep this seperate / readable

	SAVEGAME,					// used for allocating chunks during savegame file read
	SHADERTEXT,					// used by cm_shader stuff
	CM_TERRAIN,					// terrain
	R_TERRAIN,					// renderer side of terrain
	INFLATE,				// Temp memory used by zlib32
	DEFLATE,				// Temp memory used by zlib32
	POINTCACHE,					// weather effects
	NEWDEL,
	#[cfg(feature = "xbox")]
	UI_ALLOC,
	#[cfg(feature = "xbox")]
	BINK,
	COUNT
}

//////////////// eof //////////////
