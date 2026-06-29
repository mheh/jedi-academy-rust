// Copyright (C) 1999-2000 Id Software, Inc.

use core::ffi::{c_int, c_char, c_void};

// allow a lot of command backups for very fast systems
// multiple commands may be combined into a single packet, so this
// needs to be larger than PACKET_BACKUP
pub const CMD_BACKUP: c_int = 64;
pub const CMD_MASK: c_int = CMD_BACKUP - 1;

pub const MAX_ENTITIES_IN_SNAPSHOT: usize = 256;

// snapshots are a view of the server at a given time

// Snapshots are generated at regular time intervals by the server,
// but they may not be sent if a client's rate level is exceeded, or
// they may be dropped by the network.
#[repr(C)]
pub struct snapshot_t {
    pub snapFlags: c_int,           // SNAPFLAG_RATE_DELAYED, etc
    pub ping: c_int,

    pub serverTime: c_int,          // server time the message is valid for (in msec)

    pub areamask: [u8; MAX_MAP_AREA_BYTES],     // portalarea visibility bits

    pub ps: playerState_t,                      // complete information about the current player at this time
    pub vps: playerState_t,                     // vehicle I'm riding's playerstate (if applicable) -rww

    pub numEntities: c_int,                     // all of the entities that need to be presented
    pub entities: [entityState_t; MAX_ENTITIES_IN_SNAPSHOT],    // at the time of this snapshot

    pub numServerCommands: c_int,               // text based server commands to execute when this
    pub serverCommandSequence: c_int,           // snapshot becomes current
}

pub const CGAME_EVENT_NONE: c_int = 0;
pub const CGAME_EVENT_TEAMMENU: c_int = 1;
pub const CGAME_EVENT_SCOREBOARD: c_int = 2;
pub const CGAME_EVENT_EDITHUD: c_int = 3;

// functions imported from the main executable

pub const CGAME_IMPORT_API_VERSION: c_int = 5;

#[repr(C)]
pub enum cgameImport_t {
    CG_PRINT = 0,
    CG_ERROR,
    CG_MILLISECONDS,

    // Also for profiling.. do not use for game related tasks.
    CG_PRECISIONTIMER_START,
    CG_PRECISIONTIMER_END,

    CG_CVAR_REGISTER,
    CG_CVAR_UPDATE,
    CG_CVAR_SET,
    CG_CVAR_VARIABLESTRINGBUFFER,
    CG_CVAR_GETHIDDENVALUE,
    CG_ARGC,
    CG_ARGV,
    CG_ARGS,
    CG_FS_FOPENFILE,
    CG_FS_READ,
    CG_FS_WRITE,
    CG_FS_FCLOSEFILE,
    CG_FS_GETFILELIST,
    CG_SENDCONSOLECOMMAND,
    CG_ADDCOMMAND,
    CG_REMOVECOMMAND,
    CG_SENDCLIENTCOMMAND,
    CG_UPDATESCREEN,
    CG_CM_LOADMAP,
    CG_CM_NUMINLINEMODELS,
    CG_CM_INLINEMODEL,
    CG_CM_TEMPBOXMODEL,
    CG_CM_TEMPCAPSULEMODEL,
    CG_CM_POINTCONTENTS,
    CG_CM_TRANSFORMEDPOINTCONTENTS,
    CG_CM_BOXTRACE,
    CG_CM_CAPSULETRACE,
    CG_CM_TRANSFORMEDBOXTRACE,
    CG_CM_TRANSFORMEDCAPSULETRACE,
    CG_CM_MARKFRAGMENTS,
    CG_S_GETVOICEVOLUME,
    CG_S_MUTESOUND,
    CG_S_STARTSOUND,
    CG_S_STARTLOCALSOUND,
    CG_S_CLEARLOOPINGSOUNDS,
    CG_S_ADDLOOPINGSOUND,
    CG_S_UPDATEENTITYPOSITION,
    CG_S_ADDREALLOOPINGSOUND,
    CG_S_STOPLOOPINGSOUND,
    CG_S_RESPATIALIZE,
    CG_S_SHUTUP,
    CG_S_REGISTERSOUND,
    CG_S_STARTBACKGROUNDTRACK,

    // rww - AS trap implem
    CG_S_UPDATEAMBIENTSET,
    CG_AS_PARSESETS,
    CG_AS_ADDPRECACHEENTRY,
    CG_S_ADDLOCALSET,
    CG_AS_GETBMODELSOUND,

    CG_R_LOADWORLDMAP,
    CG_R_REGISTERMODEL,
    CG_R_REGISTERSKIN,
    CG_R_REGISTERSHADER,
    CG_R_REGISTERSHADERNOMIP,
    CG_R_REGISTERFONT,
    CG_R_FONT_STRLENPIXELS,
    CG_R_FONT_STRLENCHARS,
    CG_R_FONT_STRHEIGHTPIXELS,
    CG_R_FONT_DRAWSTRING,
    CG_LANGUAGE_ISASIAN,
    CG_LANGUAGE_USESSPACES,
    CG_ANYLANGUAGE_READCHARFROMSTRING,

    CGAME_MEMSET = 100,
    CGAME_MEMCPY,
    CGAME_STRNCPY,
    CGAME_SIN,
    CGAME_COS,
    CGAME_ATAN2,
    CGAME_SQRT,
    CGAME_MATRIXMULTIPLY,
    CGAME_ANGLEVECTORS,
    CGAME_PERPENDICULARVECTOR,
    CGAME_FLOOR,
    CGAME_CEIL,

    CGAME_TESTPRINTINT,
    CGAME_TESTPRINTFLOAT,

    CGAME_ACOS,
    CGAME_ASIN,

    CG_R_CLEARSCENE = 200,
    CG_R_CLEARDECALS,
    CG_R_ADDREFENTITYTOSCENE,
    CG_R_ADDPOLYTOSCENE,
    CG_R_ADDPOLYSTOSCENE,
    CG_R_ADDDECALTOSCENE,
    CG_R_LIGHTFORPOINT,
    CG_R_ADDLIGHTTOSCENE,
    CG_R_ADDADDITIVELIGHTTOSCENE,
    CG_R_RENDERSCENE,
    CG_R_SETCOLOR,
    CG_R_DRAWSTRETCHPIC,
    CG_R_MODELBOUNDS,
    CG_R_LERPTAG,
    CG_R_DRAWROTATEPIC,
    CG_R_DRAWROTATEPIC2,
    CG_R_SETRANGEFOG,         // linear fogging, with settable range -rww
    CG_R_SETREFRACTIONPROP,   // set some properties for the draw layer for my refractive effect (here primarily for mod authors) -rww
    CG_R_REMAP_SHADER,
    CG_R_GET_LIGHT_STYLE,
    CG_R_SET_LIGHT_STYLE,
    CG_R_GET_BMODEL_VERTS,
    CG_R_GETDISTANCECULL,

    CG_R_GETREALRES,
    CG_R_AUTOMAPELEVADJ,
    CG_R_INITWIREFRAMEAUTO,

    CG_FX_ADDLINE,

    CG_GETGLCONFIG,
    CG_GETGAMESTATE,
    CG_GETCURRENTSNAPSHOTNUMBER,
    CG_GETSNAPSHOT,
    CG_GETDEFAULTSTATE,
    CG_GETSERVERCOMMAND,
    CG_GETCURRENTCMDNUMBER,
    CG_GETUSERCMD,
    CG_SETUSERCMDVALUE,
    CG_SETCLIENTFORCEANGLE,
    CG_SETCLIENTTURNEXTENT,
    CG_OPENUIMENU,
    CG_TESTPRINTINT,
    CG_TESTPRINTFLOAT,
    CG_MEMORY_REMAINING,
    CG_KEY_ISDOWN,
    CG_KEY_GETCATCHER,
    CG_KEY_SETCATCHER,
    CG_KEY_GETKEY,

    CG_PC_ADD_GLOBAL_DEFINE,
    CG_PC_LOAD_SOURCE,
    CG_PC_FREE_SOURCE,
    CG_PC_READ_TOKEN,
    CG_PC_SOURCE_FILE_AND_LINE,
    CG_PC_LOAD_GLOBAL_DEFINES,
    CG_PC_REMOVE_ALL_GLOBAL_DEFINES,

    CG_S_STOPBACKGROUNDTRACK,
    CG_REAL_TIME,
    CG_SNAPVECTOR,
    CG_CIN_PLAYCINEMATIC,
    CG_CIN_STOPCINEMATIC,
    CG_CIN_RUNCINEMATIC,
    CG_CIN_DRAWCINEMATIC,
    CG_CIN_SETEXTENTS,

    CG_GET_ENTITY_TOKEN,
    CG_R_INPVS,

    CG_FX_REGISTER_EFFECT,
    CG_FX_PLAY_EFFECT,
    CG_FX_PLAY_ENTITY_EFFECT,
    CG_FX_PLAY_EFFECT_ID,
    CG_FX_PLAY_PORTAL_EFFECT_ID,
    CG_FX_PLAY_ENTITY_EFFECT_ID,
    CG_FX_PLAY_BOLTED_EFFECT_ID,
    CG_FX_ADD_SCHEDULED_EFFECTS,
    CG_FX_INIT_SYSTEM,
    CG_FX_SET_REFDEF,
    CG_FX_FREE_SYSTEM,
    CG_FX_ADJUST_TIME,
    CG_FX_DRAW_2D_EFFECTS,
    CG_FX_RESET,
    CG_FX_ADDPOLY,
    CG_FX_ADDBEZIER,
    CG_FX_ADDPRIMITIVE,
    CG_FX_ADDSPRITE,
    CG_FX_ADDELECTRICITY,

    // CG_SP_PRINT,
    CG_SP_GETSTRINGTEXTSTRING,

    CG_ROFF_CLEAN,
    CG_ROFF_UPDATE_ENTITIES,
    CG_ROFF_CACHE,
    CG_ROFF_PLAY,
    CG_ROFF_PURGE_ENT,

    // rww - dynamic vm memory allocation!
    CG_TRUEMALLOC,
    CG_TRUEFREE,

    // Ghoul2 Insert Start
    CG_G2_LISTSURFACES,
    CG_G2_LISTBONES,
    CG_G2_SETMODELS,
    CG_G2_HAVEWEGHOULMODELS,
    CG_G2_GETBOLT,
    CG_G2_GETBOLT_NOREC,
    CG_G2_GETBOLT_NOREC_NOROT,
    CG_G2_INITGHOUL2MODEL,
    CG_G2_SETSKIN,
    CG_G2_COLLISIONDETECT,
    CG_G2_COLLISIONDETECTCACHE,
    CG_G2_CLEANMODELS,
    CG_G2_ANGLEOVERRIDE,
    CG_G2_PLAYANIM,
    CG_G2_GETBONEANIM,
    CG_G2_GETBONEFRAME,         // trimmed down version of GBA, so I don't have to pass all those unused args across the VM-exe border
    CG_G2_GETGLANAME,
    CG_G2_COPYGHOUL2INSTANCE,
    CG_G2_COPYSPECIFICGHOUL2MODEL,
    CG_G2_DUPLICATEGHOUL2INSTANCE,
    CG_G2_HASGHOUL2MODELONINDEX,
    CG_G2_REMOVEGHOUL2MODEL,
    CG_G2_SKINLESSMODEL,
    CG_G2_GETNUMGOREMARKS,
    CG_G2_ADDSKINGORE,
    CG_G2_CLEARSKINGORE,
    CG_G2_SIZE,
    CG_G2_ADDBOLT,
    CG_G2_ATTACHENT,
    CG_G2_SETBOLTON,
    CG_G2_SETROOTSURFACE,
    CG_G2_SETSURFACEONOFF,
    CG_G2_SETNEWORIGIN,
    CG_G2_DOESBONEEXIST,
    CG_G2_GETSURFACERENDERSTATUS,

    CG_G2_GETTIME,
    CG_G2_SETTIME,

    CG_G2_ABSURDSMOOTHING,

    // rww - RAGDOLL_BEGIN
    CG_G2_SETRAGDOLL,
    CG_G2_ANIMATEG2MODELS,
    // rww - RAGDOLL_END

    // additional ragdoll options -rww
    CG_G2_RAGPCJCONSTRAINT,
    CG_G2_RAGPCJGRADIENTSPEED,
    CG_G2_RAGEFFECTORGOAL,
    CG_G2_GETRAGBONEPOS,
    CG_G2_RAGEFFECTORKICK,
    CG_G2_RAGFORCESOLVE,

    // rww - ik move method, allows you to specify a bone and move it to a world point (within joint constraints)
    // by using the majority of gil's existing bone angling stuff from the ragdoll code.
    CG_G2_SETBONEIKSTATE,
    CG_G2_IKMOVE,

    CG_G2_REMOVEBONE,

    CG_G2_ATTACHINSTANCETOENTNUM,
    CG_G2_CLEARATTACHEDINSTANCE,
    CG_G2_CLEANENTATTACHMENTS,
    CG_G2_OVERRIDESERVER,

    CG_G2_GETSURFACENAME,

    CG_SET_SHARED_BUFFER,

    CG_CM_REGISTER_TERRAIN,
    CG_RMG_INIT,
    CG_RE_INIT_RENDERER_TERRAIN,
    CG_R_WEATHER_CONTENTS_OVERRIDE,
    CG_R_WORLDEFFECTCOMMAND,
    // Adding trap to get weather working
    CG_WE_ADDWEATHERZONE,

    // Ghoul2 Insert End
}

// functions exported to the main executable

#[repr(C)]
pub enum cgameExport_t {
    CG_INIT = 0,
    // void CG_Init( int serverMessageNum, int serverCommandSequence, int clientNum )
    // called when the level loads or when the renderer is restarted
    // all media should be registered at this time
    // cgame will display loading status by calling SCR_Update, which
    // will call CG_DrawInformation during the loading process
    // reliableCommandSequence will be 0 on fresh loads, but higher for
    // demos, tourney restarts, or vid_restarts

    CG_SHUTDOWN,
    // void (*CG_Shutdown)( void );
    // oportunity to flush and close any open files

    CG_CONSOLE_COMMAND,
    // qboolean (*CG_ConsoleCommand)( void );
    // a console command has been issued locally that is not recognized by the
    // main game system.
    // use Cmd_Argc() / Cmd_Argv() to read the command, return qfalse if the
    // command is not known to the game

    CG_DRAW_ACTIVE_FRAME,
    // void (*CG_DrawActiveFrame)( int serverTime, stereoFrame_t stereoView, qboolean demoPlayback );
    // Generates and draws a game scene and status information at the given time.
    // If demoPlayback is set, local movement prediction will not be enabled

    CG_CROSSHAIR_PLAYER,
    // int (*CG_CrosshairPlayer)( void );

    CG_LAST_ATTACKER,
    // int (*CG_LastAttacker)( void );

    CG_KEY_EVENT,
    // void	(*CG_KeyEvent)( int key, qboolean down );

    CG_MOUSE_EVENT,
    // void	(*CG_MouseEvent)( int dx, int dy );
    CG_EVENT_HANDLING,
    // void (*CG_EventHandling)(int type);

    CG_POINT_CONTENTS,
    // int	CG_PointContents( const vec3_t point, int passEntityNum );

    CG_GET_LERP_ORIGIN,
    // void CG_LerpOrigin(int num, vec3_t result);

    CG_GET_LERP_DATA,
    CG_GET_GHOUL2,
    CG_GET_MODEL_LIST,

    CG_CALC_LERP_POSITIONS,
    // void CG_CalcEntityLerpPositions(int num);

    CG_TRACE,
    CG_G2TRACE,
    // void CG_Trace( trace_t *result, const vec3_t start, const vec3_t mins, const vec3_t maxs, const vec3_t end,
    // 					 int skipNumber, int mask );

    CG_G2MARK,

    CG_RAG_CALLBACK,

    CG_INCOMING_CONSOLE_COMMAND,

    CG_GET_USEABLE_FORCE,

    CG_GET_ORIGIN,              // int entnum, vec3_t origin
    CG_GET_ANGLES,              // int entnum, vec3_t angle

    CG_GET_ORIGIN_TRAJECTORY,   // int entnum
    CG_GET_ANGLE_TRAJECTORY,    // int entnum

    CG_ROFF_NOTETRACK_CALLBACK, // int entnum, char *notetrack

    CG_IMPACT_MARK,
    // void CG_ImpactMark( qhandle_t markShader, const vec3_t origin, const vec3_t dir,
    // 				   float orientation, float red, float green, float blue, float alpha,
    // 				   qboolean alphaFade, float radius, qboolean temporary )

    CG_MAP_CHANGE,

    CG_AUTOMAP_INPUT,

    CG_MISC_ENT,                // rwwRMG - added

    CG_GET_SORTED_FORCE_POWER,

    CG_FX_CAMERASHAKE,          // mcg post-gold added
}

#[repr(C)]
pub struct autoMapInput_t {
    pub up: f32,
    pub down: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub goToDefaults: c_int,    // qboolean
}

// CG_POINT_CONTENTS
#[repr(C)]
pub struct TCGPointContents {
    pub mPoint: vec3_t,         // input
    pub mPassEntityNum: c_int,  // input
}

// CG_GET_BOLT_POS
#[repr(C)]
pub struct TCGGetBoltData {
    pub mOrigin: vec3_t,        // output
    pub mAngles: vec3_t,        // output
    pub mScale: vec3_t,         // output
    pub mEntityNum: c_int,      // input
}

// CG_IMPACT_MARK
#[repr(C)]
pub struct TCGImpactMark {
    pub mHandle: c_int,
    pub mPoint: vec3_t,
    pub mAngle: vec3_t,
    pub mRotation: f32,
    pub mRed: f32,
    pub mGreen: f32,
    pub mBlue: f32,
    pub mAlphaStart: f32,
    pub mSizeStart: f32,
}

// CG_GET_LERP_ORIGIN
// CG_GET_LERP_ANGLES
// CG_GET_MODEL_SCALE
#[repr(C)]
pub struct TCGVectorData {
    pub mEntityNum: c_int,  // input
    pub mPoint: vec3_t,     // output
}

// CG_TRACE/CG_G2TRACE
#[repr(C)]
pub struct TCGTrace {
    pub mResult: trace_t,       // output
    pub mStart: vec3_t,
    pub mMins: vec3_t,
    pub mMaxs: vec3_t,
    pub mEnd: vec3_t,           // input
    pub mSkipNumber: c_int,
    pub mMask: c_int,           // input
}

// CG_G2MARK
#[repr(C)]
pub struct TCGG2Mark {
    pub shader: c_int,
    pub size: f32,
    pub start: vec3_t,
    pub dir: vec3_t,
}

// CG_INCOMING_CONSOLE_COMMAND
#[repr(C)]
pub struct TCGIncomingConsoleCommand {
    pub conCommand: [c_char; 1024],
}

// CG_FX_CAMERASHAKE
#[repr(C)]
pub struct TCGCameraShake {
    pub mOrigin: vec3_t,        // input
    pub mIntensity: f32,        // input
    pub mRadius: c_int,         // input
    pub mTime: c_int,           // input
}

// CG_MISC_ENT
#[repr(C)]
pub struct TCGMiscEnt {
    pub mModel: [c_char; MAX_QPATH],    // input
    pub mOrigin: vec3_t,
    pub mAngles: vec3_t,
    pub mScale: vec3_t,                 // input
}

#[repr(C)]
pub struct TCGPositionOnBolt {
    pub ent: refEntity_t,       // output
    pub ghoul2: *mut c_void,    // input
    pub modelIndex: c_int,      // input
    pub boltIndex: c_int,       // input
    pub origin: vec3_t,         // input
    pub angles: vec3_t,         // input
    pub modelScale: vec3_t,     // input
}

// ragdoll callback structs -rww
pub const RAG_CALLBACK_NONE: c_int = 0;
pub const RAG_CALLBACK_DEBUGBOX: c_int = 1;

#[repr(C)]
pub struct ragCallbackDebugBox_t {
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub duration: c_int,
}

pub const RAG_CALLBACK_DEBUGLINE: c_int = 2;

#[repr(C)]
pub struct ragCallbackDebugLine_t {
    pub start: vec3_t,
    pub end: vec3_t,
    pub time: c_int,
    pub color: c_int,
    pub radius: c_int,
}

pub const RAG_CALLBACK_BONESNAP: c_int = 3;

#[repr(C)]
pub struct ragCallbackBoneSnap_t {
    pub boneName: [c_char; 128],    // name of the bone in question
    pub entNum: c_int,              // index of entity who owns the bone in question
}

pub const RAG_CALLBACK_BONEIMPACT: c_int = 4;

#[repr(C)]
pub struct ragCallbackBoneImpact_t {
    pub boneName: [c_char; 128],    // name of the bone in question
    pub entNum: c_int,              // index of entity who owns the bone in question
}

pub const RAG_CALLBACK_BONEINSOLID: c_int = 5;

#[repr(C)]
pub struct ragCallbackBoneInSolid_t {
    pub bonePos: vec3_t,            // world coordinate position of the bone
    pub entNum: c_int,              // index of entity who owns the bone in question
    pub solidCount: c_int,          // higher the count, the longer we've been in solid (the worse off we are)
}

pub const RAG_CALLBACK_TRACELINE: c_int = 6;

#[repr(C)]
pub struct ragCallbackTraceLine_t {
    pub tr: trace_t,
    pub start: vec3_t,
    pub end: vec3_t,
    pub mins: vec3_t,
    pub maxs: vec3_t,
    pub ignore: c_int,
    pub mask: c_int,
}

pub const MAX_CG_SHARED_BUFFER_SIZE: c_int = 2048;

// External type stubs: these are declared in other modules
// and assumed to be in scope through imports or crate re-exports
#[allow(non_camel_case_types)]
pub struct playerState_t;

#[allow(non_camel_case_types)]
pub struct entityState_t;

#[allow(non_camel_case_types)]
pub struct trace_t;

#[allow(non_camel_case_types)]
pub struct refEntity_t;

pub type vec3_t = [f32; 3];

// External constants from other headers
pub const MAX_MAP_AREA_BYTES: usize = 32;  // portalarea visibility bits
pub const MAX_QPATH: usize = 64;           // file path length
