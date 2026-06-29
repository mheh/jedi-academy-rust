#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::ffi::{c_char, c_int, c_uint, c_void};

// #include "../game/q_shared.h"
use crate::codemp::game::q_shared_h::*;
// #include "../cgame/tr_types.h"
use crate::codemp::cgame::tr_types_h::*;
// #include "keycodes.h"
use crate::codemp::ui::keycodes_h::*;
// #include "../../ui/menudef.h"
use crate::ui::menudef_h::*;
// #include "../namespace_begin.h" and "../namespace_end.h" (hoisted from mid-file positions)
use crate::codemp::namespace_begin_h::*;
use crate::codemp::namespace_end_h::*;

pub const MAX_MENUNAME: usize      = 32;
pub const MAX_ITEMTEXT: usize      = 64;
pub const MAX_ITEMACTION: usize    = 64;
pub const MAX_MENUDEFFILE: usize   = 4096;
pub const MAX_MENUFILE: usize      = 32768;
pub const MAX_MENUS: usize         = 64;
pub const MAX_MENUITEMS: usize     = 256;
pub const MAX_COLOR_RANGES: usize  = 10;
pub const MAX_OPEN_MENUS: usize    = 16;
pub const MAX_TEXTSCROLL_LINES: usize = 256;

pub const WINDOW_MOUSEOVER: c_int        = 0x00000001; // mouse is over it, non exclusive
pub const WINDOW_HASFOCUS: c_int         = 0x00000002; // has cursor focus, exclusive
pub const WINDOW_VISIBLE: c_int          = 0x00000004; // is visible
pub const WINDOW_INACTIVE: c_int         = 0x00000008; // is visible but grey ( non-active )
pub const WINDOW_DECORATION: c_int       = 0x00000010; // for decoration only, no mouse, keyboard, etc..
pub const WINDOW_FADINGOUT: c_int        = 0x00000020; // fading out, non-active
pub const WINDOW_FADINGIN: c_int         = 0x00000040; // fading in
pub const WINDOW_MOUSEOVERTEXT: c_int    = 0x00000080; // mouse is over it, non exclusive
pub const WINDOW_INTRANSITION: c_int     = 0x00000100; // window is in transition
pub const WINDOW_FORECOLORSET: c_int     = 0x00000200; // forecolor was explicitly set ( used to color alpha images or not )
pub const WINDOW_HORIZONTAL: c_int       = 0x00000400; // for list boxes and sliders, vertical is default this is set of horizontal
pub const WINDOW_LB_LEFTARROW: c_int     = 0x00000800; // mouse is over left/up arrow
pub const WINDOW_LB_RIGHTARROW: c_int    = 0x00001000; // mouse is over right/down arrow
pub const WINDOW_LB_THUMB: c_int         = 0x00002000; // mouse is over thumb
pub const WINDOW_LB_PGUP: c_int          = 0x00004000; // mouse is over page up
pub const WINDOW_LB_PGDN: c_int          = 0x00008000; // mouse is over page down
pub const WINDOW_ORBITING: c_int         = 0x00010000; // item is in orbit
pub const WINDOW_OOB_CLICK: c_int        = 0x00020000; // close on out of bounds click
pub const WINDOW_WRAPPED: c_int          = 0x00040000; // manually wrap text
pub const WINDOW_AUTOWRAPPED: c_int      = 0x00080000; // auto wrap text
pub const WINDOW_FORCED: c_int           = 0x00100000; // forced open
pub const WINDOW_POPUP: c_int            = 0x00200000; // popup
pub const WINDOW_BACKCOLORSET: c_int     = 0x00400000; // backcolor was explicitly set
pub const WINDOW_TIMEDVISIBLE: c_int     = 0x00800000; // visibility timing ( NOT implemented )
pub const WINDOW_PLAYERCOLOR: c_int      = 0x01000000; // hack the forecolor to match ui_char_color_*

//JLF
pub const WINDOW_INTRANSITIONMODEL: c_int = 0x04000000; // delayed script waiting to run


// CGAME cursor type bits
pub const CURSOR_NONE: c_int  = 0x00000001;
pub const CURSOR_ARROW: c_int = 0x00000002;
pub const CURSOR_SIZER: c_int = 0x00000004;

// #ifdef _XBOX mapped to cfg(feature = "xbox"); #ifdef CGAME mapped to cfg(feature = "cgame")
#[cfg(all(feature = "xbox", feature = "cgame"))]
pub const STRING_POOL_SIZE: usize = 32 * 1024;

#[cfg(all(feature = "xbox", not(feature = "cgame")))]
pub const STRING_POOL_SIZE: usize = 128 * 1024;

#[cfg(all(not(feature = "xbox"), feature = "cgame"))]
pub const STRING_POOL_SIZE: usize = 128 * 1024;

#[cfg(all(not(feature = "xbox"), not(feature = "cgame")))]
pub const STRING_POOL_SIZE: usize = 384 * 1024;

pub const MAX_STRING_HANDLES: usize = 4096;
pub const MAX_SCRIPT_ARGS: usize    = 12;
pub const MAX_EDITFIELD: usize      = 256;

pub const ART_FX_BASE:   &str = "menu/art/fx_base";
pub const ART_FX_BLUE:   &str = "menu/art/fx_blue";
pub const ART_FX_CYAN:   &str = "menu/art/fx_cyan";
pub const ART_FX_GREEN:  &str = "menu/art/fx_grn";
pub const ART_FX_RED:    &str = "menu/art/fx_red";
pub const ART_FX_TEAL:   &str = "menu/art/fx_teal";
pub const ART_FX_WHITE:  &str = "menu/art/fx_white";
pub const ART_FX_YELLOW: &str = "menu/art/fx_yel";
pub const ART_FX_ORANGE: &str = "menu/art/fx_orange";
pub const ART_FX_PURPLE: &str = "menu/art/fx_purple";

pub const ASSET_GRADIENTBAR:          &str = "ui/assets/gradientbar2.tga";
pub const ASSET_SCROLLBAR:            &str = "gfx/menus/scrollbar.tga";
pub const ASSET_SCROLLBAR_ARROWDOWN:  &str = "gfx/menus/scrollbar_arrow_dwn_a.tga";
pub const ASSET_SCROLLBAR_ARROWUP:    &str = "gfx/menus/scrollbar_arrow_up_a.tga";
pub const ASSET_SCROLLBAR_ARROWLEFT:  &str = "gfx/menus/scrollbar_arrow_left.tga";
pub const ASSET_SCROLLBAR_ARROWRIGHT: &str = "gfx/menus/scrollbar_arrow_right.tga";
pub const ASSET_SCROLL_THUMB:         &str = "gfx/menus/scrollbar_thumb.tga";
pub const ASSET_SLIDER_BAR:           &str = "menu/new/slider";
pub const ASSET_SLIDER_THUMB:         &str = "menu/new/sliderthumb";
pub const SCROLLBAR_SIZE: f32      = 16.0;
pub const SLIDER_WIDTH: f32        = 96.0;
pub const SLIDER_HEIGHT: f32       = 16.0;
pub const SLIDER_THUMB_WIDTH: f32  = 12.0;
pub const SLIDER_THUMB_HEIGHT: f32 = 20.0;
pub const NUM_CROSSHAIRS: usize    = 9;

#[repr(C)]
pub struct scriptDef_t {
    pub command: *const c_char,
    pub args: [*const c_char; MAX_SCRIPT_ARGS],
}


#[repr(C)]
pub struct rectDef_t {
    pub x: f32, // horiz position
    pub y: f32, // vert position
    pub w: f32, // width
    pub h: f32, // height;
}

pub type Rectangle = rectDef_t;

// FIXME: do something to separate text vs window stuff
#[repr(C)]
pub struct windowDef_t {
    pub rect: Rectangle,              // client coord rectangle
    pub rectClient: Rectangle,        // screen coord rectangle
    pub name: *const c_char,          //
    pub group: *const c_char,         // if it belongs to a group
    pub cinematicName: *const c_char, // cinematic name
    pub cinematic: c_int,             // cinematic handle
    pub style: c_int,                 //
    pub border: c_int,                //
    pub ownerDraw: c_int,             // ownerDraw style
    pub ownerDrawFlags: c_int,        // show flags for ownerdraw items
    pub borderSize: f32,              //
    pub flags: c_int,                 // visible, focus, mouseover, cursor
    pub rectEffects: Rectangle,       // for various effects
    pub rectEffects2: Rectangle,      // for various effects
    pub offsetTime: c_int,            // time based value for various effects
    pub nextTime: c_int,              // time next effect should cycle
    pub foreColor: vec4_t,            // text color
    pub backColor: vec4_t,            // border color
    pub borderColor: vec4_t,          // border color
    pub outlineColor: vec4_t,         // border color
    pub background: qhandle_t,        // background asset
}

pub type Window = windowDef_t;

#[repr(C)]
pub struct colorRangeDef_t {
    pub color: vec4_t,
    pub low: f32,
    pub high: f32,
}

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
pub const MAX_LB_COLUMNS: usize = 16;

#[repr(C)]
pub struct columnInfo_t {
    pub pos: c_int,
    pub width: c_int,
    pub maxChars: c_int,
}

#[repr(C)]
pub struct listBoxDef_t {
    pub startPos: c_int,
    pub endPos: c_int,
    pub drawPadding: c_int,
    pub cursorPos: c_int,
    pub elementWidth: f32,
    pub elementHeight: f32,
    pub elementStyle: c_int,
    pub numColumns: c_int,
    pub columnInfo: [columnInfo_t; MAX_LB_COLUMNS],
    pub doubleClick: *const c_char,
    pub notselectable: qboolean,
    //JLF MPMOVED
    pub scrollhidden: qboolean,
}

#[repr(C)]
pub struct editFieldDef_t {
    pub minVal: f32,         //	edit field limits
    pub maxVal: f32,         //
    pub defVal: f32,         //
    pub range: f32,          //
    pub maxChars: c_int,     // for edit fields
    pub maxPaintChars: c_int, // for edit fields
    pub paintOffset: c_int,  //
}

pub const MAX_MULTI_CVARS: usize = 32;

#[repr(C)]
pub struct multiDef_t {
    pub cvarList: [*const c_char; MAX_MULTI_CVARS],
    pub cvarStr: [*const c_char; MAX_MULTI_CVARS],
    pub cvarValue: [f32; MAX_MULTI_CVARS],
    pub count: c_int,
    pub strDef: qboolean,
}

#[repr(C)]
pub struct modelDef_t {
    pub angle: c_int,
    pub origin: vec3_t,
    pub fov_x: f32,
    pub fov_y: f32,
    pub rotationSpeed: c_int,

    pub g2mins: vec3_t,  //required
    pub g2maxs: vec3_t,  //required
    pub g2scale: vec3_t, //optional
    pub g2skin: c_int,   //optional
    pub g2anim: c_int,   //optional
    //JLF
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

#[repr(C)]
pub struct textScrollDef_t {
    pub startPos: c_int,
    pub endPos: c_int,

    pub lineHeight: f32,
    pub maxLineChars: c_int,
    pub drawPadding: c_int,

    // changed spelling to make them fall out during compile while I made them asian-aware	-Ste
    //
    pub iLineCount: c_int,
    pub pLines: [*const c_char; MAX_TEXTSCROLL_LINES], // can contain NULL ptrs that you should skip over during paint.
}

pub const ITEM_ALIGN_LEFT: c_int   = 0; // left alignment
pub const ITEM_ALIGN_CENTER: c_int = 1; // center alignment
pub const ITEM_ALIGN_RIGHT: c_int  = 2; // right alignment

pub const CVAR_ENABLE: c_int  = 0x00000001;
pub const CVAR_DISABLE: c_int = 0x00000002;
pub const CVAR_SHOW: c_int    = 0x00000004;
pub const CVAR_HIDE: c_int    = 0x00000008;

pub const ITF_G2VALID: c_int     = 0x0001; // indicates whether or not g2 instance is valid.
pub const ITF_ISCHARACTER: c_int = 0x0002; // a character item, uses customRGBA
pub const ITF_ISSABER: c_int     = 0x0004; // first saber item, draws blade
pub const ITF_ISSABER2: c_int    = 0x0008; // second saber item, draws blade

pub const ITF_ISANYSABER: c_int = ITF_ISSABER | ITF_ISSABER2; //either saber

#[repr(C)]
pub struct itemDef_t {
    pub window: Window,                    // common positional, border, style, layout info
    pub textRect: Rectangle,               // rectangle the text ( if any ) consumes
    pub type_: c_int,                      // text, button, radiobutton, checkbox, textfield, listbox, combo  [porting: C 'type' -> 'type_']
    pub alignment: c_int,                  // left center right
    pub textalignment: c_int,              // ( optional ) alignment for text within rect based on text width
    pub textalignx: f32,                   // ( optional ) text alignment x coord
    pub textaligny: f32,                   // ( optional ) text alignment x coord
    pub textscale: f32,                    // scale percentage from 72pts
    pub textStyle: c_int,                  // ( optional ) style, normal and shadowed are it for now
    pub text: *const c_char,               // display text
    pub text2: *const c_char,              // display text, 2nd line
    pub text2alignx: f32,                  // ( optional ) text2 alignment x coord
    pub text2aligny: f32,                  // ( optional ) text2 alignment y coord
    pub parent: *mut c_void,               // menu owner
    pub asset: qhandle_t,                  // handle to asset
    pub ghoul2: *mut c_void,               // ghoul2 instance if available instead of a model.
    pub flags: c_int,                      // flags like g2valid, character, saber, saber2, etc.
    pub mouseEnterText: *const c_char,     // mouse enter script
    pub mouseExitText: *const c_char,      // mouse exit script
    pub mouseEnter: *const c_char,         // mouse enter script
    pub mouseExit: *const c_char,          // mouse exit script
    pub action: *const c_char,             // select script
    //JLFACCEPT MPMOVED
    pub accept: *const c_char,
    //JLFDPADSCRIPT
    pub selectionNext: *const c_char,
    pub selectionPrev: *const c_char,

    pub onFocus: *const c_char,            // select script
    pub leaveFocus: *const c_char,         // select script
    pub cvar: *const c_char,               // associated cvar
    pub cvarTest: *const c_char,           // associated cvar for enable actions
    pub enableCvar: *const c_char,         // enable, disable, show, or hide based on value, this can contain a list
    pub cvarFlags: c_int,                  //	what type of action to take on cvarenables
    pub focusSound: sfxHandle_t,
    pub numColors: c_int,                  // number of color ranges
    pub colorRanges: [colorRangeDef_t; MAX_COLOR_RANGES],
    pub special: f32,                      // used for feeder id's etc.. diff per type
    pub cursorPos: c_int,                  // cursor position in characters
    pub typeData: *mut c_void,             // type specific data ptr's
    pub descText: *const c_char,           //	Description text
    pub appearanceSlot: c_int,             // order of appearance
    pub iMenuFont: c_int,                  // FONT_SMALL,FONT_MEDIUM,FONT_LARGE	// changed from 'font' so I could see what didn't compile, and differentiate between font handles returned from RegisterFont -ste
    pub disabled: qboolean,                // Does this item ignore mouse and keyboard focus
    pub invertYesNo: c_int,
    pub xoffset: c_int,
}

#[repr(C)]
pub struct menuDef_t {
    pub window: Window,
    pub font: *const c_char,               // font
    pub fullScreen: qboolean,              // covers entire screen
    pub itemCount: c_int,                  // number of items;
    pub fontIndex: c_int,                  //
    pub cursorItem: c_int,                 // which item as the cursor
    pub fadeCycle: c_int,                  //
    pub fadeClamp: f32,                    //
    pub fadeAmount: f32,                   //
    pub onOpen: *const c_char,             // run when the menu is first opened
    pub onClose: *const c_char,            // run when the menu is closed
    //JLFACCEPT
    pub onAccept: *const c_char,           // run when menu is closed with acceptance

    pub onESC: *const c_char,              // run when the menu is closed
    pub soundName: *const c_char,          // background loop sound for menu

    pub focusColor: vec4_t,                // focus color for items
    pub disableColor: vec4_t,              // focus color for items
    pub items: [*mut itemDef_t; MAX_MENUITEMS], // items this menu contains
    pub descX: c_int,                      // X position of description
    pub descY: c_int,                      // X position of description
    pub descColor: vec4_t,                 // description text color for items
    pub descAlignment: c_int,             // Description of alignment
    pub descScale: f32,                    // Description scale
    pub appearanceTime: f32,               //	when next item should appear
    pub appearanceCnt: c_int,              //	current item displayed
    pub appearanceIncrement: f32,          //
}

#[repr(C)]
pub struct cachedAssets_t {
    pub fontStr: *const c_char,
    pub cursorStr: *const c_char,
    pub gradientStr: *const c_char,
    pub qhSmallFont: qhandle_t,
    pub qhSmall2Font: qhandle_t,
    pub qhMediumFont: qhandle_t,
    pub qhBigFont: qhandle_t,
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
    pub fadeClamp: f32,
    pub fadeCycle: c_int,
    pub fadeAmount: f32,
    pub shadowX: f32,
    pub shadowY: f32,
    pub shadowColor: vec4_t,
    pub shadowFadeClamp: f32,
    pub fontRegistered: qboolean,

    pub needPass: qhandle_t,
    pub noForce: qhandle_t,
    pub forceRestrict: qhandle_t,
    pub saberOnly: qhandle_t,
    pub trueJedi: qhandle_t,

    pub moveRollSound: sfxHandle_t,
    pub moveJumpSound: sfxHandle_t,
    pub datapadmoveSaberSound1: sfxHandle_t,
    pub datapadmoveSaberSound2: sfxHandle_t,
    pub datapadmoveSaberSound3: sfxHandle_t,
    pub datapadmoveSaberSound4: sfxHandle_t,
    pub datapadmoveSaberSound5: sfxHandle_t,
    pub datapadmoveSaberSound6: sfxHandle_t,

    // player settings
    pub fxBasePic: qhandle_t,
    pub fxPic: [qhandle_t; 7],
    pub crosshairShader: [qhandle_t; NUM_CROSSHAIRS],
}

#[repr(C)]
pub struct commandDef_t {
    pub name: *const c_char,
    pub handler: Option<unsafe extern "C" fn(*mut itemDef_t, *mut *mut c_char) -> qboolean>,
}

#[repr(C)]
pub struct displayContextDef_t {
    pub registerShaderNoMip: Option<unsafe extern "C" fn(*const c_char) -> qhandle_t>,
    pub setColor: Option<unsafe extern "C" fn(*const vec4_t)>,
    pub drawHandlePic: Option<unsafe extern "C" fn(f32, f32, f32, f32, qhandle_t)>,
    pub drawStretchPic: Option<unsafe extern "C" fn(f32, f32, f32, f32, f32, f32, f32, f32, qhandle_t)>,
    pub drawText: Option<unsafe extern "C" fn(f32, f32, f32, *const vec4_t, *const c_char, f32, c_int, c_int, c_int)>,
    pub textWidth: Option<unsafe extern "C" fn(*const c_char, f32, c_int) -> c_int>,
    pub textHeight: Option<unsafe extern "C" fn(*const c_char, f32, c_int) -> c_int>,
    pub registerModel: Option<unsafe extern "C" fn(*const c_char) -> qhandle_t>,
    pub modelBounds: Option<unsafe extern "C" fn(qhandle_t, *mut vec3_t, *mut vec3_t)>,
    pub fillRect: Option<unsafe extern "C" fn(f32, f32, f32, f32, *const vec4_t)>,
    pub drawRect: Option<unsafe extern "C" fn(f32, f32, f32, f32, f32, *const vec4_t)>,
    pub drawSides: Option<unsafe extern "C" fn(f32, f32, f32, f32, f32)>,
    pub drawTopBottom: Option<unsafe extern "C" fn(f32, f32, f32, f32, f32)>,
    pub clearScene: Option<unsafe extern "C" fn()>,
    pub addRefEntityToScene: Option<unsafe extern "C" fn(*const refEntity_t)>,
    pub renderScene: Option<unsafe extern "C" fn(*const refdef_t)>,

    pub RegisterFont: Option<unsafe extern "C" fn(*const c_char) -> qhandle_t>,
    pub Font_StrLenPixels: Option<unsafe extern "C" fn(*const c_char, c_int, f32) -> c_int>,
    pub Font_StrLenChars: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub Font_HeightPixels: Option<unsafe extern "C" fn(c_int, f32) -> c_int>,
    pub Font_DrawString: Option<unsafe extern "C" fn(c_int, c_int, *const c_char, *const f32, c_int, c_int, f32)>,
    pub Language_IsAsian: Option<unsafe extern "C" fn() -> qboolean>,
    pub Language_UsesSpaces: Option<unsafe extern "C" fn() -> qboolean>,
    pub AnyLanguage_ReadCharFromString: Option<unsafe extern "C" fn(*const c_char, *mut c_int, *mut qboolean) -> c_uint>,
    pub ownerDrawItem: Option<unsafe extern "C" fn(f32, f32, f32, f32, f32, f32, c_int, c_int, c_int, f32, f32, *const vec4_t, qhandle_t, c_int, c_int)>,
    pub getValue: Option<unsafe extern "C" fn(c_int) -> f32>,
    pub ownerDrawVisible: Option<unsafe extern "C" fn(c_int) -> qboolean>,
    pub runScript: Option<unsafe extern "C" fn(*mut *mut c_char)>,
    pub deferScript: Option<unsafe extern "C" fn(*mut *mut c_char) -> qboolean>,
    pub getTeamColor: Option<unsafe extern "C" fn(*mut vec4_t)>,
    pub getCVarString: Option<unsafe extern "C" fn(*const c_char, *mut c_char, c_int)>,
    pub getCVarValue: Option<unsafe extern "C" fn(*const c_char) -> f32>,
    pub setCVar: Option<unsafe extern "C" fn(*const c_char, *const c_char)>,
    pub drawTextWithCursor: Option<unsafe extern "C" fn(f32, f32, f32, *const vec4_t, *const c_char, c_int, c_char, c_int, c_int, c_int)>,
    pub setOverstrikeMode: Option<unsafe extern "C" fn(qboolean)>,
    pub getOverstrikeMode: Option<unsafe extern "C" fn() -> qboolean>,
    pub startLocalSound: Option<unsafe extern "C" fn(sfxHandle_t, c_int)>,
    pub ownerDrawHandleKey: Option<unsafe extern "C" fn(c_int, c_int, *mut f32, c_int) -> qboolean>,
    pub feederCount: Option<unsafe extern "C" fn(f32) -> c_int>,
    pub feederItemText: Option<unsafe extern "C" fn(f32, c_int, c_int, *mut qhandle_t, *mut qhandle_t, *mut qhandle_t) -> *const c_char>,
    pub feederItemImage: Option<unsafe extern "C" fn(f32, c_int) -> qhandle_t>,
    pub feederSelection: Option<unsafe extern "C" fn(f32, c_int, *mut itemDef_t) -> qboolean>,
    pub keynumToStringBuf: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
    pub getBindingBuf: Option<unsafe extern "C" fn(c_int, *mut c_char, c_int)>,
    pub setBinding: Option<unsafe extern "C" fn(c_int, *const c_char)>,
    pub executeText: Option<unsafe extern "C" fn(c_int, *const c_char)>,
    pub Error: Option<unsafe extern "C" fn(c_int, *const c_char, ...)>,
    pub Print: Option<unsafe extern "C" fn(*const c_char, ...)>,
    pub Pause: Option<unsafe extern "C" fn(qboolean)>,
    pub ownerDrawWidth: Option<unsafe extern "C" fn(c_int, f32) -> c_int>,
    pub registerSound: Option<unsafe extern "C" fn(*const c_char) -> sfxHandle_t>,
    pub startBackgroundTrack: Option<unsafe extern "C" fn(*const c_char, *const c_char, qboolean)>,
    pub stopBackgroundTrack: Option<unsafe extern "C" fn()>,
    pub playCinematic: Option<unsafe extern "C" fn(*const c_char, f32, f32, f32, f32) -> c_int>,
    pub stopCinematic: Option<unsafe extern "C" fn(c_int)>,
    pub drawCinematic: Option<unsafe extern "C" fn(c_int, f32, f32, f32, f32)>,
    pub runCinematicFrame: Option<unsafe extern "C" fn(c_int)>,

    pub yscale: f32,
    pub xscale: f32,
    pub bias: f32,
    pub realTime: c_int,
    pub frameTime: c_int,
    pub cursorx: c_int,
    pub cursory: c_int,
    pub debug: qboolean,

    pub Assets: cachedAssets_t,

    pub glconfig: glconfig_t,
    pub whiteShader: qhandle_t,
    pub gradientImage: qhandle_t,
    pub cursor: qhandle_t,
    pub FPS: f32,
}

extern "C" {
    pub fn String_Alloc(p: *const c_char) -> *const c_char;
    pub fn String_Init();
    pub fn String_Report();
    pub fn Init_Display(dc: *mut displayContextDef_t);
    pub fn Display_ExpandMacros(buff: *mut c_char);
    pub fn Menu_Init(menu: *mut menuDef_t);
    pub fn Item_Init(item: *mut itemDef_t);
    pub fn Menu_PostParse(menu: *mut menuDef_t);
    pub fn Menu_GetFocused() -> *mut menuDef_t;
    pub fn Menu_HandleKey(menu: *mut menuDef_t, key: c_int, down: qboolean);
    pub fn Menu_HandleMouseMove(menu: *mut menuDef_t, x: f32, y: f32);
    pub fn Menu_ScrollFeeder(menu: *mut menuDef_t, feeder: c_int, down: qboolean);
    pub fn Float_Parse(p: *mut *mut c_char, f: *mut f32) -> qboolean;
    pub fn Color_Parse(p: *mut *mut c_char, c: *mut vec4_t) -> qboolean;
    pub fn Int_Parse(p: *mut *mut c_char, i: *mut c_int) -> qboolean;
    pub fn Rect_Parse(p: *mut *mut c_char, r: *mut rectDef_t) -> qboolean;
    pub fn String_Parse(p: *mut *mut c_char, out: *mut *const c_char) -> qboolean;
    pub fn Script_Parse(p: *mut *mut c_char, out: *mut *const c_char) -> qboolean;
    pub fn PC_Float_Parse(handle: c_int, f: *mut f32) -> qboolean;
    pub fn PC_Color_Parse(handle: c_int, c: *mut vec4_t) -> qboolean;
    pub fn PC_Int_Parse(handle: c_int, i: *mut c_int) -> qboolean;
    pub fn PC_Rect_Parse(handle: c_int, r: *mut rectDef_t) -> qboolean;
    pub fn PC_String_Parse(handle: c_int, out: *mut *const c_char) -> qboolean;
    pub fn PC_Script_Parse(handle: c_int, out: *mut *const c_char) -> qboolean;
    pub fn Menu_Count() -> c_int;
    pub fn Menu_New(handle: c_int);
    pub fn Menu_PaintAll();
    pub fn Menus_ActivateByName(p: *const c_char) -> *mut menuDef_t;
    pub fn Menu_Reset();
    pub fn Menus_AnyFullScreenVisible() -> qboolean;
    pub fn Menus_Activate(menu: *mut menuDef_t);
    pub fn Menu_FindItemByName(menu: *mut menuDef_t, p: *const c_char) -> *mut itemDef_t;

    pub fn Display_GetContext() -> *mut displayContextDef_t;
    pub fn Display_CaptureItem(x: c_int, y: c_int) -> *mut c_void;
    pub fn Display_MouseMove(p: *mut c_void, x: c_int, y: c_int) -> qboolean;
    pub fn Display_CursorType(x: c_int, y: c_int) -> c_int;
    pub fn Display_KeyBindPending() -> qboolean;
    pub fn Menus_OpenByName(p: *const c_char);
    pub fn Menus_FindByName(p: *const c_char) -> *mut menuDef_t;
    pub fn Menus_ShowByName(p: *const c_char);
    pub fn Menus_CloseByName(p: *const c_char);
    pub fn Display_HandleKey(key: c_int, down: qboolean, x: c_int, y: c_int);
    pub fn LerpColor(a: *mut vec4_t, b: *mut vec4_t, c: *mut vec4_t, t: f32);
    pub fn Menus_CloseAll();
    pub fn Menu_Paint(menu: *mut menuDef_t, forcePaint: qboolean);
    pub fn Menu_SetFeederSelection(menu: *mut menuDef_t, feeder: c_int, index: c_int, name: *const c_char);
    pub fn Display_CacheAll();
    pub fn Menu_SetItemBackground(menu: *const menuDef_t, itemName: *const c_char, background: *const c_char);

    pub fn UI_Alloc(size: c_int) -> *mut c_void;
    pub fn UI_InitMemory();
    pub fn UI_OutOfMemory() -> qboolean;

    pub fn Controls_GetConfig();
    pub fn Controls_SetConfig(restart: qboolean);


    pub fn trap_PC_AddGlobalDefine(define: *mut c_char) -> c_int;
    pub fn trap_PC_LoadSource(filename: *const c_char) -> c_int;
    pub fn trap_PC_FreeSource(handle: c_int) -> c_int;
    pub fn trap_PC_ReadToken(handle: c_int, pc_token: *mut pc_token_t) -> c_int;
    pub fn trap_PC_SourceFileAndLine(handle: c_int, filename: *mut c_char, line: *mut c_int) -> c_int;
    pub fn trap_PC_LoadGlobalDefines(filename: *const c_char) -> c_int;
    pub fn trap_PC_RemoveAllGlobalDefines();

    pub fn trap_R_Font_StrLenPixels(text: *const c_char, iFontIndex: c_int, scale: f32) -> c_int;
    pub fn trap_R_Font_StrLenChars(text: *const c_char) -> c_int;
    pub fn trap_R_Font_HeightPixels(iFontIndex: c_int, scale: f32) -> c_int;
    pub fn trap_R_Font_DrawString(ox: c_int, oy: c_int, text: *const c_char, rgba: *const f32, setIndex: c_int, iCharLimit: c_int, scale: f32);
    pub fn trap_Language_IsAsian() -> qboolean;
    pub fn trap_Language_UsesSpaces() -> qboolean;
    pub fn trap_AnyLanguage_ReadCharFromString(psText: *const c_char, piAdvanceCount: *mut c_int, pbIsTrailingPunctuation: *mut qboolean) -> c_uint;

    pub fn trap_SP_GetStringTextString(text: *const c_char, buffer: *mut c_char, bufferLength: c_int) -> c_int;
    pub fn trap_SP_GetNumLanguages() -> c_int;
    pub fn trap_GetLanguageName(languageIndex: c_int, buffer: *mut c_char);

    //these traps must exist both on the cgame and ui
    /*
    Ghoul2 Insert Start
    */
    // UI specific API access
    pub fn trap_G2API_CollisionDetect(collRecMap: *mut CollisionRecord_t, ghoul2: *mut c_void, angles: *const vec3_t, position: *const vec3_t, frameNumber: c_int, entNum: c_int, rayStart: *const vec3_t, rayEnd: *const vec3_t, scale: *const vec3_t, traceFlags: c_int, useLod: c_int, fRadius: f32);
    pub fn trap_G2API_CollisionDetectCache(collRecMap: *mut CollisionRecord_t, ghoul2: *mut c_void, angles: *const vec3_t, position: *const vec3_t, frameNumber: c_int, entNum: c_int, rayStart: *const vec3_t, rayEnd: *const vec3_t, scale: *const vec3_t, traceFlags: c_int, useLod: c_int, fRadius: f32);


    pub fn trap_G2_ListModelSurfaces(ghlInfo: *mut c_void);
    pub fn trap_G2_ListModelBones(ghlInfo: *mut c_void, frame: c_int);
    pub fn trap_G2_SetGhoul2ModelIndexes(ghoul2: *mut c_void, modelList: *mut qhandle_t, skinList: *mut qhandle_t);
    pub fn trap_G2_HaveWeGhoul2Models(ghoul2: *mut c_void) -> qboolean;
    pub fn trap_G2API_GetBoltMatrix(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut mdxaBone_t, angles: *const vec3_t, position: *const vec3_t, frameNum: c_int, modelList: *mut qhandle_t, scale: *const vec3_t) -> qboolean;
    pub fn trap_G2API_GetBoltMatrix_NoReconstruct(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut mdxaBone_t, angles: *const vec3_t, position: *const vec3_t, frameNum: c_int, modelList: *mut qhandle_t, scale: *const vec3_t) -> qboolean;
    pub fn trap_G2API_GetBoltMatrix_NoRecNoRot(ghoul2: *mut c_void, modelIndex: c_int, boltIndex: c_int, matrix: *mut mdxaBone_t, angles: *const vec3_t, position: *const vec3_t, frameNum: c_int, modelList: *mut qhandle_t, scale: *const vec3_t) -> qboolean;
    pub fn trap_G2API_InitGhoul2Model(ghoul2Ptr: *mut *mut c_void, fileName: *const c_char, modelIndex: c_int, customSkin: qhandle_t, customShader: qhandle_t, modelFlags: c_int, lodBias: c_int) -> c_int;
    pub fn trap_G2API_SetSkin(ghoul2: *mut c_void, modelIndex: c_int, customSkin: qhandle_t, renderSkin: qhandle_t) -> qboolean;
    pub fn trap_G2API_AttachG2Model(ghoul2From: *mut c_void, modelIndexFrom: c_int, ghoul2To: *mut c_void, toBoltIndex: c_int, toModel: c_int) -> qboolean;


    pub fn trap_G2API_CopyGhoul2Instance(g2From: *mut c_void, g2To: *mut c_void, modelIndex: c_int) -> c_int;
    pub fn trap_G2API_CopySpecificGhoul2Model(g2From: *mut c_void, modelFrom: c_int, g2To: *mut c_void, modelTo: c_int);
    pub fn trap_G2API_DuplicateGhoul2Instance(g2From: *mut c_void, g2To: *mut *mut c_void);
    pub fn trap_G2API_HasGhoul2ModelOnIndex(ghlInfo: *mut c_void, modelIndex: c_int) -> qboolean;
    pub fn trap_G2API_RemoveGhoul2Model(ghlInfo: *mut c_void, modelIndex: c_int) -> qboolean;

    pub fn trap_G2API_AddBolt(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char) -> c_int;
    //pub fn trap_G2API_RemoveBolt(ghoul2: *mut c_void, index: c_int) -> qboolean;
    pub fn trap_G2API_SetBoltInfo(ghoul2: *mut c_void, modelIndex: c_int, boltInfo: c_int);
    pub fn trap_G2API_CleanGhoul2Models(ghoul2Ptr: *mut *mut c_void);
    pub fn trap_G2API_SetBoneAngles(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, angles: *const vec3_t, flags: c_int, up: c_int, right: c_int, forward: c_int, modelList: *mut qhandle_t, blendTime: c_int, currentTime: c_int) -> qboolean;
    pub fn trap_G2API_GetGLAName(ghoul2: *mut c_void, modelIndex: c_int, fillBuf: *mut c_char);
    pub fn trap_G2API_SetBoneAnim(ghoul2: *mut c_void, modelIndex: c_int, boneName: *const c_char, startFrame: c_int, endFrame: c_int, flags: c_int, animSpeed: f32, currentTime: c_int, setFrame: f32, blendTime: c_int) -> qboolean;
    pub fn trap_G2API_GetBoneAnim(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32, startFrame: *mut c_int, endFrame: *mut c_int, flags: *mut c_int, animSpeed: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> qboolean;
    pub fn trap_G2API_GetBoneFrame(ghoul2: *mut c_void, boneName: *const c_char, currentTime: c_int, currentFrame: *mut f32, modelList: *mut c_int, modelIndex: c_int) -> qboolean;

    pub fn trap_G2API_SetRootSurface(ghoul2: *mut c_void, modelIndex: c_int, surfaceName: *const c_char) -> qboolean;
    pub fn trap_G2API_SetSurfaceOnOff(ghoul2: *mut c_void, surfaceName: *const c_char, flags: c_int) -> qboolean;
    pub fn trap_G2API_SetNewOrigin(ghoul2: *mut c_void, boltIndex: c_int) -> qboolean;

    pub fn trap_G2API_GetTime() -> c_int;
    pub fn trap_G2API_SetTime(time: c_int, clock: c_int);

    pub fn trap_G2API_SetRagDoll(ghoul2: *mut c_void, params: *mut sharedRagDollParams_t);
    pub fn trap_G2API_AnimateG2Models(ghoul2: *mut c_void, time: c_int, params: *mut sharedRagDollUpdateParams_t);

    pub fn trap_G2API_SetBoneIKState(ghoul2: *mut c_void, time: c_int, boneName: *const c_char, ikState: c_int, params: *mut sharedSetBoneIKStateParams_t) -> qboolean;
    pub fn trap_G2API_IKMove(ghoul2: *mut c_void, time: c_int, params: *mut sharedIKMoveParams_t) -> qboolean;

    pub fn trap_G2API_GetSurfaceName(ghoul2: *mut c_void, surfNumber: c_int, modelIndex: c_int, fillBuf: *mut c_char);
}

/*
Ghoul2 Insert End
*/
