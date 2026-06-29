#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Stub imports for types defined elsewhere in the codebase
pub use crate::qcommon::q_shared::{qhandle_t, sfxHandle_t, qboolean, vec3_t, vec4_t};

// Types that may need to be imported from other modules
// (stubs if not yet ported)
pub type CGhoul2Info = c_void;
pub type CGhoul2Info_v = c_void;
pub type mdxaBone_t = c_void;
pub type refEntity_t = c_void;
pub type refdef_t = c_void;
pub type glconfig_t = c_void;
pub type Eorientations = c_int;

// Conditional feature-based type stubs
#[cfg(feature = "immersion")]
pub type ffHandle_t = c_int;

pub const MAX_TOKENLENGTH: c_int = 1024;
pub const MAX_OPEN_MENUS: c_int = 16;
pub const MAX_TEXTSCROLL_LINES: c_int = 256;

pub const MAX_EDITFIELD: c_int = 256;

// token types
pub const TT_STRING: c_int = 1;				// string
pub const TT_LITERAL: c_int = 2;			// literal
pub const TT_NUMBER: c_int = 3;				// number
pub const TT_NAME: c_int = 4;				// name
pub const TT_PUNCTUATION: c_int = 5;		// punctuation

pub const SLIDER_WIDTH: f32 = 128.0;
pub const SLIDER_HEIGHT: f32 = 16.0;
pub const SLIDER_THUMB_WIDTH: f32 = 12.0;
pub const SLIDER_THUMB_HEIGHT: f32 = 16.0;
pub const SCROLLBAR_SIZE: f32 = 16.0;

#[repr(C)]
pub struct pc_token_s {
	pub type_: c_int,
	pub subtype: c_int,
	pub intvalue: c_int,
	pub floatvalue: f32,
	pub string: [c_char; MAX_TOKENLENGTH as usize],
}

pub type pc_token_t = pc_token_s;


// FIXME: combine flags into bitfields to save space
// FIXME: consolidate all of the common stuff in one structure for menus and items
// THINKABOUTME: is there any compelling reason not to have items contain items
// and do away with a menu per say.. major issue is not being able to dynamically allocate
// and destroy stuff.. Another point to consider is adding an alloc free call for vm's and have
// the engine just allocate the pool for it based on a cvar
// many of the vars are re-used for different item types, as such they are not always named appropriately
// the benefits of c++ in DOOM will greatly help crap like this
// FIXME: need to put a type ptr that points to specific type info per type
//
pub const MAX_LB_COLUMNS: c_int = 16;

#[repr(C)]
pub struct columnInfo_s {
	pub pos: c_int,
	pub width: c_int,
	pub maxChars: c_int,
}

pub type columnInfo_t = columnInfo_s;

#[repr(C)]
pub struct listBoxDef_s {
	pub startPos: c_int,
	pub endPos: c_int,
	pub drawPadding: c_int,
	pub cursorPos: c_int,
	pub elementWidth: f32,
	pub elementHeight: f32,
	pub elementStyle: c_int,
	pub numColumns: c_int,
	pub columnInfo: [columnInfo_t; MAX_LB_COLUMNS as usize],
	pub doubleClick: *const c_char,
	pub notselectable: qboolean,
	//JLF MPMOVED
	pub scrollhidden: qboolean,
}

pub type listBoxDef_t = listBoxDef_s;


#[repr(C)]
pub struct editFieldDef_s {
	pub minVal: f32,						//	edit field limits
	pub maxVal: f32,						//
	pub defVal: f32,						//
	pub range: f32,						//
	pub maxChars: c_int,					// for edit fields
	pub maxPaintChars: c_int,				// for edit fields
	pub paintOffset: c_int,				//
}

pub type editFieldDef_t = editFieldDef_s;

pub const MAX_MULTI_CVARS: c_int = 64;//32

#[repr(C)]
pub struct multiDef_s {
	pub cvarList: [*const c_char; MAX_MULTI_CVARS as usize],
	pub cvarStr: [*const c_char; MAX_MULTI_CVARS as usize],
	pub cvarValue: [f32; MAX_MULTI_CVARS as usize],
	pub count: c_int,
	pub strDef: qboolean,
}

pub type multiDef_t = multiDef_s;

pub const CVAR_ENABLE: c_int = 0x00000001;
pub const CVAR_DISABLE: c_int = 0x00000002;
pub const CVAR_SHOW: c_int = 0x00000004;
pub const CVAR_HIDE: c_int = 0x00000008;
pub const CVAR_SUBSTRING: c_int = 0x00000010;	//when using enable or disable, just check for strstr instead of ==


#[cfg(target_os = "windows")]
pub const STRING_POOL_SIZE: c_int = 64*1024;

#[cfg(all(target_os = "windows", feature = "cgame"))]
pub const STRING_POOL_SIZE: c_int = 128*1024;

#[cfg(all(target_os = "windows", not(feature = "cgame")))]
pub const STRING_POOL_SIZE: c_int = 384*1024;

#[cfg(not(target_os = "windows"))]
pub const STRING_POOL_SIZE: c_int = if cfg!(feature = "cgame") { 128*1024 } else { 384*1024 };

pub const NUM_CROSSHAIRS: c_int = 9;

#[repr(C)]
pub struct cachedAssets_t {
	pub qhMediumFont: qhandle_t,
	pub cursor: qhandle_t,
	pub gradientBar: qhandle_t,
	pub scrollBarArrowUp: qhandle_t,
	pub scrollBarArrowDown: qhandle_t,
	pub scrollBarArrowLeft: qhandle_t,
	pub scrollBarArrowRight: qhandle_t,
	pub scrollBar: qhandle_t,
	pub scrollBarThumb: qhandle_t,
	pub buttonMiddle: qhandle_t,
	pub buttonInside: qhandle_t,
	pub solidBox: qhandle_t,
	pub sliderBar: qhandle_t,
	pub sliderThumb: qhandle_t,
	pub menuEnterSound: sfxHandle_t,
	pub menuExitSound: sfxHandle_t,
	pub menuBuzzSound: sfxHandle_t,
	pub itemFocusSound: sfxHandle_t,
	pub forceChosenSound: sfxHandle_t,
	pub forceUnchosenSound: sfxHandle_t,
	pub datapadmoveRollSound: sfxHandle_t,
	pub datapadmoveJumpSound: sfxHandle_t,
	pub datapadmoveSaberSound1: sfxHandle_t,
	pub datapadmoveSaberSound2: sfxHandle_t,
	pub datapadmoveSaberSound3: sfxHandle_t,
	pub datapadmoveSaberSound4: sfxHandle_t,
	pub datapadmoveSaberSound5: sfxHandle_t,
	pub datapadmoveSaberSound6: sfxHandle_t,

	pub nullSound: sfxHandle_t,

	#[cfg(feature = "immersion")]
	pub menuEnterForce: ffHandle_t,
	#[cfg(feature = "immersion")]
	pub menuExitForce: ffHandle_t,
	#[cfg(feature = "immersion")]
	pub menuBuzzForce: ffHandle_t,
	#[cfg(feature = "immersion")]
	pub itemFocusForce: ffHandle_t,
	pub fadeClamp: f32,
	pub fadeCycle: c_int,
	pub fadeAmount: f32,
	pub shadowX: f32,
	pub shadowY: f32,
	pub shadowColor: vec4_t,
	pub shadowFadeClamp: f32,
	pub fontRegistered: qboolean,

	  // player settings
	//	pub fxBasePic: qhandle_t,
	//	pub fxPic: [qhandle_t; 7],
	pub crosshairShader: [qhandle_t; NUM_CROSSHAIRS as usize],

}

pub struct itemDef_s;

pub type displayContextDefFn_addRefEntityToScene = extern "C" fn(*const refEntity_t);
pub type displayContextDefFn_clearScene = extern "C" fn();
pub type displayContextDefFn_drawHandlePic = extern "C" fn(f32, f32, f32, f32, qhandle_t);
pub type displayContextDefFn_drawRect = extern "C" fn(f32, f32, f32, f32, f32, *const vec4_t);
pub type displayContextDefFn_drawSides = extern "C" fn(f32, f32, f32, f32, f32);
pub type displayContextDefFn_drawText = extern "C" fn(f32, f32, f32, vec4_t, *const c_char, c_int, c_int, c_int);
pub type displayContextDefFn_drawTextWithCursor = extern "C" fn(f32, f32, f32, vec4_t, *const c_char, c_int, c_char, c_int, c_int, c_int);
pub type displayContextDefFn_drawTopBottom = extern "C" fn(f32, f32, f32, f32, f32);
pub type displayContextDefFn_executeText = extern "C" fn(c_int, *const c_char);
pub type displayContextDefFn_feederCount = extern "C" fn(f32) -> c_int;
pub type displayContextDefFn_feederSelection = extern "C" fn(f32, c_int, *mut itemDef_s);
pub type displayContextDefFn_fillRect = extern "C" fn(f32, f32, f32, f32, *const vec4_t);
pub type displayContextDefFn_getBindingBuf = extern "C" fn(c_int, *mut c_char, c_int);
pub type displayContextDefFn_getCVarString = extern "C" fn(*const c_char, *mut c_char, c_int);
pub type displayContextDefFn_getCVarValue = extern "C" fn(*const c_char) -> f32;
pub type displayContextDefFn_getOverstrikeMode = extern "C" fn() -> qboolean;
pub type displayContextDefFn_getValue = extern "C" fn(c_int) -> f32;
pub type displayContextDefFn_keynumToStringBuf = extern "C" fn(c_int, *mut c_char, c_int);
pub type displayContextDefFn_modelBounds = extern "C" fn(qhandle_t, *mut vec3_t, *mut vec3_t);
pub type displayContextDefFn_ownerDrawHandleKey = extern "C" fn(c_int, c_int, *mut f32, c_int) -> qboolean;
pub type displayContextDefFn_ownerDrawItem = extern "C" fn(f32, f32, f32, f32, f32, f32, c_int, c_int, c_int, f32, f32, vec4_t, qhandle_t, c_int, c_int);
pub type displayContextDefFn_ownerDrawVisible = extern "C" fn(c_int) -> qboolean;
pub type displayContextDefFn_ownerDrawWidth = extern "C" fn(c_int, f32) -> c_int;
pub type displayContextDefFn_Pause = extern "C" fn(qboolean);
pub type displayContextDefFn_Print = extern "C" fn(*const c_char);
pub type displayContextDefFn_registerFont = extern "C" fn(*const c_char) -> c_int;
pub type displayContextDefFn_registerModel = extern "C" fn(*const c_char) -> qhandle_t;
pub type displayContextDefFn_registerShaderNoMip = extern "C" fn(*const c_char) -> qhandle_t;
pub type displayContextDefFn_registerSound = extern "C" fn(*const c_char, qboolean) -> sfxHandle_t;
pub type displayContextDefFn_renderScene = extern "C" fn(*const refdef_t);
pub type displayContextDefFn_runScript = extern "C" fn(*const *const c_char) -> qboolean;
pub type displayContextDefFn_deferScript = extern "C" fn(*const *const c_char) -> qboolean;
pub type displayContextDefFn_setBinding = extern "C" fn(c_int, *const c_char);
pub type displayContextDefFn_setColor = extern "C" fn(*const vec4_t);
pub type displayContextDefFn_setCVar = extern "C" fn(*const c_char, *const c_char);
pub type displayContextDefFn_setOverstrikeMode = extern "C" fn(qboolean);
pub type displayContextDefFn_startLocalSound = extern "C" fn(sfxHandle_t, c_int);
pub type displayContextDefFn_stopCinematic = extern "C" fn(c_int);
pub type displayContextDefFn_textHeight = extern "C" fn(*const c_char, f32, c_int) -> c_int;
pub type displayContextDefFn_textWidth = extern "C" fn(*const c_char, f32, c_int) -> c_int;
pub type displayContextDefFn_feederItemImage = extern "C" fn(f32, c_int) -> qhandle_t;
pub type displayContextDefFn_feederItemText = extern "C" fn(f32, c_int, c_int, *mut qhandle_t) -> *const c_char;
pub type displayContextDefFn_registerSkin = extern "C" fn(*const c_char) -> qhandle_t;

#[repr(C)]
pub struct displayContextDef_t {
	pub addRefEntityToScene: *const displayContextDefFn_addRefEntityToScene,
	pub clearScene: *const displayContextDefFn_clearScene,
	pub drawHandlePic: *const displayContextDefFn_drawHandlePic,
	pub drawRect: *const displayContextDefFn_drawRect,
	pub drawSides: *const displayContextDefFn_drawSides,
	pub drawText: *const displayContextDefFn_drawText,
	pub drawTextWithCursor: *const displayContextDefFn_drawTextWithCursor,
	pub drawTopBottom: *const displayContextDefFn_drawTopBottom,
	pub executeText: *const displayContextDefFn_executeText,
	pub feederCount: *const displayContextDefFn_feederCount,
	pub feederSelection: *const displayContextDefFn_feederSelection,
	pub fillRect: *const displayContextDefFn_fillRect,
	pub getBindingBuf: *const displayContextDefFn_getBindingBuf,
	pub getCVarString: *const displayContextDefFn_getCVarString,
	pub getCVarValue: *const displayContextDefFn_getCVarValue,
	pub getOverstrikeMode: *const displayContextDefFn_getOverstrikeMode,
	pub getValue: *const displayContextDefFn_getValue,
	pub keynumToStringBuf: *const displayContextDefFn_keynumToStringBuf,
	pub modelBounds: *const displayContextDefFn_modelBounds,
	pub ownerDrawHandleKey: *const displayContextDefFn_ownerDrawHandleKey,
	pub ownerDrawItem: *const displayContextDefFn_ownerDrawItem,
	pub ownerDrawVisible: *const displayContextDefFn_ownerDrawVisible,
	pub ownerDrawWidth: *const displayContextDefFn_ownerDrawWidth,
	pub Pause: *const displayContextDefFn_Pause,
	pub Print: *const displayContextDefFn_Print,
	pub registerFont: *const displayContextDefFn_registerFont,
	pub registerModel: *const displayContextDefFn_registerModel,
	pub registerShaderNoMip: *const displayContextDefFn_registerShaderNoMip,
	pub registerSound: *const displayContextDefFn_registerSound,
	pub renderScene: *const displayContextDefFn_renderScene,
	pub runScript: *const displayContextDefFn_runScript,
	pub deferScript: *const displayContextDefFn_deferScript,
	pub setBinding: *const displayContextDefFn_setBinding,
	pub setColor: *const displayContextDefFn_setColor,
	pub setCVar: *const displayContextDefFn_setCVar,
	pub setOverstrikeMode: *const displayContextDefFn_setOverstrikeMode,
	pub startLocalSound: *const displayContextDefFn_startLocalSound,
	pub stopCinematic: *const displayContextDefFn_stopCinematic,
	pub textHeight: *const displayContextDefFn_textHeight,
	pub textWidth: *const displayContextDefFn_textWidth,
	pub feederItemImage: *const displayContextDefFn_feederItemImage,
	pub feederItemText: *const displayContextDefFn_feederItemText,
	pub registerSkin: *const displayContextDefFn_registerSkin,

	//rww - ghoul2 stuff. Add whatever you need here, remember to set it in _UI_Init or it will crash when you try to use it.
	#[cfg(target_os = "windows")]
	// No default arguments on function pointers
	pub g2_SetSkin: *const extern "C" fn(*mut CGhoul2Info, qhandle_t, qhandle_t) -> qboolean,
	#[cfg(target_os = "windows")]
	pub g2_SetBoneAnim: *const extern "C" fn(*mut CGhoul2Info, *const c_char, c_int, c_int,
					  c_int, f32, c_int, f32, c_int) -> qboolean,
	#[cfg(not(target_os = "windows"))]
	pub g2_SetSkin: *const extern "C" fn(*mut CGhoul2Info, qhandle_t, qhandle_t) -> qboolean,
	#[cfg(not(target_os = "windows"))]
	pub g2_SetBoneAnim: *const extern "C" fn(*mut CGhoul2Info, *const c_char, c_int, c_int,
					  c_int, f32, c_int, f32, c_int) -> qboolean,
	pub g2_RemoveGhoul2Model: *const extern "C" fn(*mut CGhoul2Info_v, c_int) -> qboolean,
	pub g2_InitGhoul2Model: *const extern "C" fn(*mut CGhoul2Info_v, *const c_char, c_int, qhandle_t, qhandle_t, c_int, c_int) -> c_int,
	pub g2_CleanGhoul2Models: *const extern "C" fn(*mut CGhoul2Info_v),
	pub g2_AddBolt: *const extern "C" fn(*mut CGhoul2Info, *const c_char) -> c_int,
	pub g2_GetBoltMatrix: *const extern "C" fn(*mut CGhoul2Info_v, c_int, c_int, *mut mdxaBone_t,
									*const vec3_t, *const vec3_t, c_int, *mut qhandle_t, *const vec3_t) -> qboolean,
	pub g2_GiveMeVectorFromMatrix: *const extern "C" fn(*mut mdxaBone_t, Eorientations, *mut vec3_t),

	//Utility functions that don't immediately redirect to ghoul2 functions
	pub g2hilev_SetAnim: *const extern "C" fn(*mut CGhoul2Info, *const c_char, c_int, qboolean) -> c_int,

	#[cfg(feature = "immersion")]
	pub registerForce: *const extern "C" fn(*const c_char, c_int) -> ffHandle_t,
	#[cfg(feature = "immersion")]
	pub startForce: *const extern "C" fn(ffHandle_t),

	pub yscale: f32,
	pub xscale: f32,
	pub bias: f32,
	pub realTime: c_int,
	pub frameTime: c_int,
	pub cursorShow: qboolean,
	pub cursorx: c_int,
	pub cursory: c_int,
	pub debug: qboolean,

	pub Assets: cachedAssets_t,

	pub glconfig: glconfig_t,
	pub whiteShader: qhandle_t,
	pub gradientImage: qhandle_t,
	pub FPS: f32,
}

extern "C" {
	pub fn UI_InitMemory();
}


pub const MAX_COLOR_RANGES: c_int = 10;
pub const MAX_MENUITEMS: c_int = 150;
pub const MAX_MENUS: c_int = 64;



pub const WINDOW_MOUSEOVER: c_int = 0x00000001;	// mouse is over it, non exclusive
pub const WINDOW_HASFOCUS: c_int = 0x00000002;			// has cursor focus, exclusive
pub const WINDOW_VISIBLE: c_int = 0x00000004;			// is visible
pub const WINDOW_INACTIVE: c_int = 0x00000008;			// is visible but grey ( non-active )
pub const WINDOW_DECORATION: c_int = 0x00000010;		// for decoration only, no mouse, keyboard, etc..
pub const WINDOW_FADINGOUT: c_int = 0x00000020;		// fading out, non-active
pub const WINDOW_FADINGIN: c_int = 0x00000040;		// fading in
pub const WINDOW_MOUSEOVERTEXT: c_int = 0x00000080;	// mouse is over it, non exclusive
pub const WINDOW_INTRANSITION: c_int = 0x00000100;		// window is in transition
pub const WINDOW_FORECOLORSET: c_int = 0x00000200;		// forecolor was explicitly set ( used to color alpha images or not )
pub const WINDOW_HORIZONTAL: c_int = 0x00000400;		// for list boxes and sliders, vertical is default this is set of horizontal
pub const WINDOW_LB_LEFTARROW: c_int = 0x00000800;		// mouse is over left/up arrow
pub const WINDOW_LB_RIGHTARROW: c_int = 0x00001000;	// mouse is over right/down arrow
pub const WINDOW_LB_THUMB: c_int = 0x00002000;			// mouse is over thumb
pub const WINDOW_LB_PGUP: c_int = 0x00004000;			// mouse is over page up
pub const WINDOW_LB_PGDN: c_int = 0x00008000;			// mouse is over page down
pub const WINDOW_ORBITING: c_int = 0x00010000;			// item is in orbit
pub const WINDOW_OOB_CLICK: c_int = 0x00020000;		// close on out of bounds click
pub const WINDOW_WRAPPED: c_int = 0x00040000;			// manually wrap text
pub const WINDOW_AUTOWRAPPED: c_int = 0x00080000;		// auto wrap text
pub const WINDOW_FORCED: c_int = 0x00100000;			// forced open
pub const WINDOW_POPUP: c_int = 0x00200000;			// popup
pub const WINDOW_BACKCOLORSET: c_int = 0x00400000;		// backcolor was explicitly set
pub const WINDOW_TIMEDVISIBLE: c_int = 0x00800000;		// visibility timing ( NOT implemented )
pub const WINDOW_PLAYERCOLOR: c_int = 0x01000000;		// hack the forecolor to match ui_char_color_*
pub const WINDOW_SCRIPTWAITING: c_int = 0x02000000;	// delayed script waiting to run
//JLF MPMOVED
pub const WINDOW_INTRANSITIONMODEL: c_int = 0x04000000;	// delayed script waiting to run
pub const WINDOW_IGNORE_ESCAPE: c_int = 0x08000000;	// ignore normal closeall menus escape functionality

#[repr(C)]
pub struct rectDef_t {
	pub x: f32,							// horiz position
	pub y: f32,							// vert position
	pub w: f32,							// width
	pub h: f32,							// height;
}

pub type UIRectangle = rectDef_t;

// FIXME: do something to separate text vs window stuff
#[repr(C)]
pub struct windowDef_t {
	pub rect: UIRectangle,						// client coord rectangle
	pub rectClient: UIRectangle,					// screen coord rectangle
	pub name: *mut c_char,						//
	pub group: *mut c_char,						// if it belongs to a group
	pub cinematicName: *const c_char,				// cinematic name
	pub cinematic: c_int,					// cinematic handle
	pub style: c_int,                      //
	pub border: c_int,                     //
	pub ownerDraw: c_int,					// ownerDraw style
	pub ownerDrawFlags: c_int,				// show flags for ownerdraw items
	pub borderSize: f32,					//
	pub flags: c_int,						// visible, focus, mouseover, cursor
	pub rectEffects: UIRectangle,				// for various effects
	pub rectEffects2: UIRectangle,				// for various effects
	pub offsetTime: c_int,					// time based value for various effects
	pub nextTime: c_int,                   // time next effect should cycle
	pub delayTime: c_int,					// time when delay expires
	pub delayedScript: *mut c_char,				// points into another script's text while delaying
	pub foreColor: vec4_t,					// text color
	pub backColor: vec4_t,					// border color
	pub borderColor: vec4_t,				// border color
	pub outlineColor: vec4_t,				// border color
	pub background: qhandle_t,					// background asset
}

pub type Window = windowDef_t;

#[repr(C)]
pub struct colorRangeDef_t {
	pub color: vec4_t,						//
	pub low: f32,						//
	pub high: f32,						//
}

#[repr(C)]
pub struct modelDef_s {
	pub angle: c_int,
	pub origin: vec3_t,
	pub fov_x: f32,
	pub fov_y: f32,
	pub rotationSpeed: c_int,

	pub g2mins: vec3_t, //required
	pub g2maxs: vec3_t, //required
	pub g2skin: c_int, //optional
	pub g2anim: c_int, //optional
	//JLF MPMOVED
	//Transition extras
	pub g2mins2: vec3_t,
	pub g2maxs2: vec3_t,
	pub g2minsEffect: vec3_t,
	pub g2maxsEffect: vec3_t,
	pub fov_x2: f32,
	pub fov_y2: f32,
	pub fov_Effectx: f32,
	pub fov_Effecty: f32,
}

pub type modelDef_t = modelDef_s;

pub const ITF_G2VALID: c_int = 0x0001;					// indicates whether or not g2 instance is valid.
pub const ITF_ISCHARACTER: c_int = 0x0002;					// a character item, uses customRGBA
pub const ITF_ISSABER: c_int = 0x0004;					// first saber item, draws blade
pub const ITF_ISSABER2: c_int = 0x0008;					// second saber item, draws blade

pub const ITF_ISANYSABER: c_int = ITF_ISSABER | ITF_ISSABER2;	//either saber

#[repr(C)]
pub struct itemDef_s {
	pub window: Window,						// common positional, border, style, layout info
	pub textRect: UIRectangle,					// rectangle the text ( if any ) consumes
	pub type_: c_int,						// text, button, radiobutton, checkbox, textfield, listbox, combo
	pub alignment: c_int,					// left center right
	pub textalignment: c_int,				// ( optional ) alignment for text within rect based on text width
	pub textalignx: f32,					// ( optional ) text alignment x coord
	pub textaligny: f32,					// ( optional ) text alignment y coord
	pub text2alignx: f32,				// ( optional ) text2 alignment x coord
	pub text2aligny: f32,				// ( optional ) text2 alignment y coord
	pub textscale: f32,					// scale percentage from 72pts
	pub textStyle: c_int,					// ( optional ) style, normal and shadowed are it for now
	pub text: *mut c_char,						// display text
	pub text2: *mut c_char,						// display text2
	pub descText: *mut c_char,					//	Description text
	pub parent: *mut c_void,					// menu owner
	pub asset: qhandle_t,						// handle to asset
	pub ghoul2: CGhoul2Info_v,					// ghoul2 instance if available instead of a model.
	pub flags: c_int,						// flags like g2valid, character, saber, saber2, etc.
	pub mouseEnterText: *const c_char,			// mouse enter script
	pub mouseExitText: *const c_char,				// mouse exit script
	pub mouseEnter: *const c_char,				// mouse enter script
	pub mouseExit: *const c_char,					// mouse exit script
	pub action: *const c_char,					// select script
	//JLFACCEPT MPMOVED
	pub accept: *const c_char,
	//JLFDPADSCRIPT MPMOVED
	pub selectionNext: *const c_char,
	pub selectionPrev: *const c_char,

	pub onFocus: *const c_char,					// select script
	pub leaveFocus: *const c_char,				// select script
	pub cvar: *const c_char,						// associated cvar
	pub cvarTest: *const c_char,					// associated cvar for enable actions
	pub enableCvar: *const c_char,				// enable, disable, show, or hide based on value, this can contain a list
	pub cvarFlags: c_int,					//	what type of action to take on cvarenables
	pub focusSound: sfxHandle_t,					//
	#[cfg(feature = "immersion")]
	pub focusForce: ffHandle_t,
	pub numColors: c_int,					// number of color ranges
	pub colorRanges: [colorRangeDef_t; MAX_COLOR_RANGES as usize],
	pub special: f32,					// used for feeder id's etc.. diff per type
	pub cursorPos: c_int,					// cursor position in characters
	pub typeData: *mut c_void,					// type specific data ptr's
	pub appearanceSlot: c_int,				// order of appearance
	pub value: c_int,						// used by ITEM_TYPE_MULTI that aren't linked to a particular cvar.
	pub font: c_int,						// FONT_SMALL,FONT_MEDIUM,FONT_LARGE
	pub invertYesNo: c_int,
	pub xoffset: c_int,

}

#[repr(C)]
pub struct menuDef_t {
	pub window: Window,
	pub font: *const c_char,						// font
	pub fullScreen: qboolean,					// covers entire screen
	pub itemCount: c_int,					// number of items;
	pub fontIndex: c_int,					//
	pub cursorItem: c_int,					// which item as the cursor
	pub fadeCycle: c_int,					//
	pub fadeClamp: f32,					//
	pub fadeAmount: f32,					//
	pub onOpen: *const c_char,					// run when the menu is first opened
	pub onClose: *const c_char,					// run when the menu is closed
	//JLFACCEPT MPMOVED
	pub onAccept: *const c_char,					// run when menu is closed with acceptance

	pub onESC: *const c_char,						// run when the menu is closed
	pub soundName: *const c_char,					// background loop sound for menu

	pub focusColor: vec4_t,					// focus color for items
	pub disableColor: vec4_t,				// focus color for items
	pub items: [*mut itemDef_t; MAX_MENUITEMS as usize],		// items this menu contains
	pub appearanceTime: f32,				//	when next item should appear
	pub appearanceCnt: c_int,				//	current item displayed
	pub appearanceIncrement: f32,		//
	pub descX: c_int,						// X position of description
	pub descY: c_int,						// X position of description
	pub descColor: vec4_t,					// description text color for items
	pub descAlignment: c_int,				// Description of alignment
	pub descScale: f32,					// Description scale
	pub descTextStyle: c_int,					// ( optional ) style, normal and shadowed are it for now


}

#[repr(C)]
pub struct textScrollDef_s
{
	pub startPos: c_int,
	pub endPos: c_int,

	pub lineHeight: f32,
	pub maxLineChars: c_int,
	pub drawPadding: c_int,

	// changed spelling to make them fall out during compile while I made them asian-aware	-Ste
	//
	pub iLineCount: c_int,
	pub pLines: [*const c_char; MAX_TEXTSCROLL_LINES as usize],	// can contain NULL ptrs that you should skip over during paint.

}

pub type textScrollDef_t = textScrollDef_s;

#[repr(C)]
pub struct commandDef_t
{
	pub name: *const c_char,
	pub handler: *const extern "C" fn(*mut itemDef_t, *const *const c_char) -> qboolean,

}

extern "C" {
	pub fn Menu_GetFocused() -> *mut menuDef_t;

	pub fn Controls_GetConfig();
	pub fn Controls_SetConfig(restart: qboolean);
	pub fn Display_KeyBindPending() -> qboolean;
	pub fn Display_MouseMove(p: *mut c_void, x: c_int, y: c_int) -> qboolean;
	pub fn Display_VisibleMenuCount() -> c_int;
	pub fn Int_Parse(p: *const *const c_char, i: *mut c_int) -> qboolean;
	pub fn Init_Display(dc: *mut displayContextDef_t);
	pub fn Menus_Activate(menu: *mut menuDef_t);
	pub fn Menus_ActivateByName(p: *const c_char) -> *mut menuDef_t;
	pub fn Menus_AnyFullScreenVisible() -> qboolean;
	pub fn Menus_CloseAll();
	pub fn Menu_Count() -> c_int;
	pub fn Menu_FindItemByName(menu: *mut menuDef_t, p: *const c_char) -> *mut itemDef_t;
	pub fn Menu_HandleKey(menu: *mut menuDef_t, key: c_int, down: qboolean);
	pub fn Menu_New(buffer: *mut c_char);
	pub fn Menus_OpenByName(p: *const c_char);
	pub fn Menu_PaintAll();
	pub fn Menu_Reset();
	pub fn PC_EndParseSession(buffer: *mut c_char);
	pub fn PC_Float_Parse(handle: c_int, f: *mut f32) -> qboolean;
	pub fn PC_ParseString(tempStr: *const *const c_char) -> qboolean;
	pub fn PC_ParseStringMem(out: *const *const c_char) -> qboolean;
	pub fn PC_ParseWarning(message: *const c_char);
	pub fn PC_String_Parse(handle: c_int, out: *const *const c_char) -> qboolean;
	#[cfg(target_os = "windows")]
	pub fn PC_StartParseSession(fileName: *const c_char, buffer: *const *mut c_char, nested: bool) -> c_int;
	#[cfg(not(target_os = "windows"))]
	pub fn PC_StartParseSession(fileName: *const c_char, buffer: *const *mut c_char) -> c_int;
	pub fn PC_ParseExt() -> *mut c_char;
	pub fn PC_ParseInt(number: *mut c_int) -> qboolean;
	pub fn PC_ParseFloat(number: *mut f32) -> qboolean;
	pub fn PC_ParseColor(c: *mut vec4_t) -> qboolean;
	pub fn String_Alloc(p: *const c_char) -> *const c_char;
	pub fn String_Init();
	pub fn String_Parse(p: *const *const c_char, out: *const *const c_char) -> qboolean;
	pub fn String_Report();
	pub fn UI_Cursor_Show(flag: qboolean);
	pub fn Menu_GetMatchingItemByNumber(menu: *mut menuDef_t, index: c_int, name: *const c_char) -> *mut itemDef_t;
}

pub static mut DC: *mut displayContextDef_t = std::ptr::null_mut();
