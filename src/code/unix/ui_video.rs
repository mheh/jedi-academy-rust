#![allow(non_snake_case, non_camel_case_types)]

use core::ffi::{c_int, c_char, c_void};

// Type definitions - stubs for external dependencies
pub type qboolean = c_int;
pub type sfxHandle_t = c_int;

const BIGCHAR_WIDTH: c_int = 8;
const BIGCHAR_HEIGHT: c_int = 8;
const SCREEN_WIDTH: c_int = 640;

const QMF_GRAYED: c_int = 0x00002000;
const QMF_BLINK: c_int = 0x00000001;

const MTYPE_SPINCONTROL: c_int = 3;
const MTYPE_RADIOBUTTON: c_int = 5;
const MTYPE_SLIDER: c_int = 1;
const MTYPE_ACTION: c_int = 2;

const K_ENTER: c_int = 13;
const K_ESCAPE: c_int = 27;

// Struct definitions - local stubs for UI menu types
#[repr(C)]
pub struct menucommon_s {
    pub type_: c_int,
    pub name: *const c_char,
    pub id: c_int,
    pub x: c_int,
    pub y: c_int,
    pub left: c_int,
    pub top: c_int,
    pub right: c_int,
    pub bottom: c_int,
    pub parent: *mut c_void,
    pub menuPosition: c_int,
    pub flags: c_int,
    pub callback: *mut c_void,
    pub statusbar: *mut c_void,
    pub ownerdraw: *mut c_void,
}

#[repr(C)]
pub struct menulist_s {
    pub generic: menucommon_s,
    pub oldvalue: c_int,
    pub curvalue: c_int,
    pub numitems: c_int,
    pub top: c_int,
    pub itemnames: *const *const c_char,
    pub width: c_int,
    pub height: c_int,
    pub columns: c_int,
    pub seperation: c_int,
}

#[repr(C)]
pub struct menuslider_s {
    pub generic: menucommon_s,
    pub minvalue: f32,
    pub maxvalue: f32,
    pub curvalue: f32,
    pub range: f32,
}

#[repr(C)]
pub struct menuaction_s {
    pub generic: menucommon_s,
}

#[repr(C)]
pub struct menuframework_s {
    pub cursor: c_int,
    pub cursor_prev: c_int,
    pub nitems: c_int,
    pub items: [*mut c_void; 256],
    pub draw: *const c_void,
    pub key: *const c_void,
    pub wrapAround: qboolean,
    pub fullscreen: qboolean,
    pub showlogo: qboolean,
    pub x: c_int,
    pub y: c_int,
}

// Extern function declarations
extern "C" {
    fn UI_ForceMenuOff();
    fn Cvar_VariableValue(name: *const c_char) -> f32;
    fn Cvar_VariableString(name: *const c_char) -> *const c_char;
    fn Cvar_SetValue(name: *const c_char, value: f32);
    fn Cvar_Set(name: *const c_char, value: *const c_char);
    fn Q_stricmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn UI_PushMenu(draw: *const c_void, key: *const c_void);
    fn UI_PopMenu();
    fn Menu_AddItem(menu: *mut menuframework_s, item: *mut c_void);
    fn Menu_AdjustCursor(menu: *mut menuframework_s, dir: c_int);
    fn Menu_Draw(menu: *mut menuframework_s);
    fn Menu_SelectItem(m: *mut menuframework_s) -> qboolean;
    fn Menu_Center(menu: *mut menuframework_s);
    fn Default_MenuKey(m: *mut menuframework_s, key: c_int) -> *const c_char;
    fn CL_Vid_Restart_f();
    fn SCR_DrawBigStringColor(x: c_int, y: c_int, text: *const c_char, color: *const f32);
}

static mut s_driver_names: [*const c_char; 4] = [
    b"[default OpenGL]\0" as *const c_char,
    b"[Voodoo OpenGL ]\0" as *const c_char,
    b"[Custom       ]\0" as *const c_char,
    core::ptr::null(),
];

static mut s_drivers: [*const c_char; 4] = [
    b"opengl32\0" as *const c_char,  // OPENGL_DRIVER_NAME
    b"3dfxvgl\0" as *const c_char,   // _3DFX_DRIVER_NAME
    b"\0" as *const c_char,
    core::ptr::null(),
];

// ====================================================================
// MENU INTERACTION
// ====================================================================

static mut s_menu: menuframework_s = menuframework_s {
    cursor: 0,
    cursor_prev: 0,
    nitems: 0,
    items: [core::ptr::null_mut(); 256],
    draw: core::ptr::null(),
    key: core::ptr::null(),
    wrapAround: 0,
    fullscreen: 0,
    showlogo: 0,
    x: 0,
    y: 0,
};

static mut s_graphics_options_list: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_mode_list: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_driver_list: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_tq_slider: menuslider_s = menuslider_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    minvalue: 0.0,
    maxvalue: 0.0,
    curvalue: 0.0,
    range: 0.0,
};

static mut s_fs_box: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_lighting_box: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_allow_extensions_box: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_texturebits_box: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_colordepth_list: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_geometry_box: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_filter_box: menulist_s = menulist_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
    oldvalue: 0,
    curvalue: 0,
    numitems: 0,
    top: 0,
    itemnames: core::ptr::null(),
    width: 0,
    height: 0,
    columns: 0,
    seperation: 0,
};

static mut s_driverinfo_action: menuaction_s = menuaction_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
};

static mut s_apply_action: menuaction_s = menuaction_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
};

static mut s_defaults_action: menuaction_s = menuaction_s {
    generic: menucommon_s {
        type_: 0,
        name: core::ptr::null(),
        id: 0,
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        parent: core::ptr::null_mut(),
        menuPosition: 0,
        flags: 0,
        callback: core::ptr::null_mut(),
        statusbar: core::ptr::null_mut(),
        ownerdraw: core::ptr::null_mut(),
    },
};

#[repr(C)]
struct InitialVideoOptions_s {
    mode: c_int,
    fullscreen: qboolean,
    tq: c_int,
    lighting: c_int,
    colordepth: c_int,
    texturebits: c_int,
    geometry: c_int,
    filter: c_int,
    driver: c_int,
    extensions: qboolean,
}

static mut s_ivo: InitialVideoOptions_s = InitialVideoOptions_s {
    mode: 0,
    fullscreen: 0,
    tq: 0,
    lighting: 0,
    colordepth: 0,
    texturebits: 0,
    geometry: 0,
    filter: 0,
    driver: 0,
    extensions: 0,
};

static mut s_ivo_templates: [InitialVideoOptions_s; 5] = [
    InitialVideoOptions_s {
        mode: 4,
        fullscreen: 1,
        tq: 2,
        lighting: 0,
        colordepth: 2,
        texturebits: 2,
        geometry: 1,
        filter: 1,
        driver: 0,
        extensions: 1, // JDC: this was tq 3
    },
    InitialVideoOptions_s {
        mode: 3,
        fullscreen: 1,
        tq: 2,
        lighting: 0,
        colordepth: 0,
        texturebits: 0,
        geometry: 1,
        filter: 0,
        driver: 0,
        extensions: 1,
    },
    InitialVideoOptions_s {
        mode: 2,
        fullscreen: 1,
        tq: 1,
        lighting: 0,
        colordepth: 1,
        texturebits: 0,
        geometry: 0,
        filter: 0,
        driver: 0,
        extensions: 1,
    },
    InitialVideoOptions_s {
        mode: 1,
        fullscreen: 1,
        tq: 1,
        lighting: 1,
        colordepth: 1,
        texturebits: 0,
        geometry: 0,
        filter: 0,
        driver: 0,
        extensions: 1,
    },
    InitialVideoOptions_s {
        mode: 3,
        fullscreen: 1,
        tq: 1,
        lighting: 0,
        colordepth: 0,
        texturebits: 0,
        geometry: 1,
        filter: 0,
        driver: 0,
        extensions: 1,
    },
];

const NUM_IVO_TEMPLATES: usize = 5; // sizeof(s_ivo_templates) / sizeof(s_ivo_templates[0])

unsafe fn DrvInfo_MenuDraw();
unsafe fn DrvInfo_MenuKey(key: c_int) -> *const c_char;

unsafe fn GetInitialVideoVars() {
    s_ivo.colordepth = s_colordepth_list.curvalue;
    s_ivo.driver = s_driver_list.curvalue;
    s_ivo.mode = s_mode_list.curvalue;
    s_ivo.fullscreen = s_fs_box.curvalue;
    s_ivo.extensions = s_allow_extensions_box.curvalue;
    s_ivo.tq = s_tq_slider.curvalue as c_int;
    s_ivo.lighting = s_lighting_box.curvalue;
    s_ivo.geometry = s_geometry_box.curvalue;
    s_ivo.filter = s_filter_box.curvalue;
    s_ivo.texturebits = s_texturebits_box.curvalue;
}

unsafe fn CheckConfigVsTemplates() {
    for i in 0..NUM_IVO_TEMPLATES {
        if s_driver_list.curvalue != 1 {
            if s_ivo_templates[i].colordepth != s_colordepth_list.curvalue {
                continue;
            }
        }
        // #if 0
        // if ( s_ivo_templates[i].driver != s_driver_list.curvalue )
        //     continue;
        // #endif
        if s_ivo_templates[i].mode != s_mode_list.curvalue {
            continue;
        }
        if s_driver_list.curvalue != 1 {
            if s_ivo_templates[i].fullscreen != s_fs_box.curvalue {
                continue;
            }
        }
        if s_ivo_templates[i].tq != s_tq_slider.curvalue as c_int {
            continue;
        }
        if s_ivo_templates[i].lighting != s_lighting_box.curvalue {
            continue;
        }
        if s_ivo_templates[i].geometry != s_geometry_box.curvalue {
            continue;
        }
        if s_ivo_templates[i].filter != s_filter_box.curvalue {
            continue;
        }
        // if ( s_ivo_templates[i].texturebits != s_texturebits_box.curvalue )
        //     continue;
        s_graphics_options_list.curvalue = i as c_int;
        return;
    }
    s_graphics_options_list.curvalue = 4;
}

unsafe fn UpdateMenuItemValues() {
    if s_driver_list.curvalue == 1 {
        s_fs_box.curvalue = 1;
        s_fs_box.generic.flags = QMF_GRAYED;
        s_colordepth_list.curvalue = 1;
    } else {
        s_fs_box.generic.flags = 0;
    }

    if s_fs_box.curvalue == 0 || s_driver_list.curvalue == 1 {
        s_colordepth_list.curvalue = 0;
        s_colordepth_list.generic.flags = QMF_GRAYED;
    } else {
        s_colordepth_list.generic.flags = 0;
    }

    if s_allow_extensions_box.curvalue == 0 {
        if s_texturebits_box.curvalue == 0 {
            s_texturebits_box.curvalue = 1;
        }
    }

    s_apply_action.generic.flags = QMF_GRAYED;

    if s_ivo.mode != s_mode_list.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.fullscreen != s_fs_box.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.extensions != s_allow_extensions_box.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.tq != s_tq_slider.curvalue as c_int {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.lighting != s_lighting_box.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.colordepth != s_colordepth_list.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.driver != s_driver_list.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.texturebits != s_texturebits_box.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.geometry != s_geometry_box.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }
    if s_ivo.filter != s_filter_box.curvalue {
        s_apply_action.generic.flags = QMF_BLINK;
    }

    CheckConfigVsTemplates();
}

unsafe fn SetMenuItemValues() {
    s_mode_list.curvalue = Cvar_VariableValue(b"r_mode\0" as *const c_char) as c_int;
    s_fs_box.curvalue = Cvar_VariableValue(b"r_fullscreen\0" as *const c_char) as c_int;
    s_allow_extensions_box.curvalue = Cvar_VariableValue(b"r_allowExtensions\0" as *const c_char) as c_int;
    s_tq_slider.curvalue = 3.0 - Cvar_VariableValue(b"r_picmip\0" as *const c_char);
    if s_tq_slider.curvalue < 0.0 {
        s_tq_slider.curvalue = 0.0;
    } else if s_tq_slider.curvalue > 3.0 {
        s_tq_slider.curvalue = 3.0;
    }

    s_lighting_box.curvalue = if Cvar_VariableValue(b"r_vertexLight\0" as *const c_char) != 0.0 { 1 } else { 0 };
    match Cvar_VariableValue(b"r_texturebits\0" as *const c_char) as c_int {
        0 => {
            s_texturebits_box.curvalue = 0;
        }
        16 => {
            s_texturebits_box.curvalue = 1;
        }
        32 => {
            s_texturebits_box.curvalue = 2;
        }
        _ => {
            s_texturebits_box.curvalue = 0;
        }
    }

    if Q_stricmp(
        Cvar_VariableString(b"r_textureMode\0" as *const c_char),
        b"GL_LINEAR_MIPMAP_NEAREST\0" as *const c_char,
    ) == 0
    {
        s_filter_box.curvalue = 0;
    } else {
        s_filter_box.curvalue = 1;
    }

    if Cvar_VariableValue(b"r_subdivisions\0" as *const c_char) == 999.0
        || Cvar_VariableValue(b"r_lodBias\0" as *const c_char) > 0.0
    {
        s_geometry_box.curvalue = 0;
    } else {
        s_geometry_box.curvalue = 1;
    }

    match Cvar_VariableValue(b"r_colorbits\0" as *const c_char) as c_int {
        0 => {
            s_colordepth_list.curvalue = 0;
        }
        16 => {
            s_colordepth_list.curvalue = 1;
        }
        32 => {
            s_colordepth_list.curvalue = 2;
        }
        _ => {
            s_colordepth_list.curvalue = 0;
        }
    }

    if s_fs_box.curvalue == 0 {
        s_colordepth_list.curvalue = 0;
    }
    if s_driver_list.curvalue == 1 {
        s_colordepth_list.curvalue = 1;
    }
}

unsafe extern "C" fn FullscreenCallback(_s: *mut c_void) {}

unsafe extern "C" fn ModeCallback(_s: *mut c_void) {
    // clamp 3dfx video modes
    if s_driver_list.curvalue == 1 {
        if s_mode_list.curvalue < 2 {
            s_mode_list.curvalue = 2;
        } else if s_mode_list.curvalue > 6 {
            s_mode_list.curvalue = 6;
        }
    }
}

unsafe extern "C" fn GraphicsOptionsCallback(_s: *mut c_void) {
    let ivo = &s_ivo_templates[s_graphics_options_list.curvalue as usize];

    s_mode_list.curvalue = ivo.mode;
    s_tq_slider.curvalue = ivo.tq as f32;
    s_lighting_box.curvalue = ivo.lighting;
    s_colordepth_list.curvalue = ivo.colordepth;
    s_texturebits_box.curvalue = ivo.texturebits;
    s_geometry_box.curvalue = ivo.geometry;
    s_filter_box.curvalue = ivo.filter;
    s_fs_box.curvalue = ivo.fullscreen;
}

unsafe extern "C" fn TextureDetailCallback(_s: *mut c_void) {}

unsafe extern "C" fn TextureQualityCallback(_s: *mut c_void) {}

unsafe extern "C" fn ExtensionsCallback(_s: *mut c_void) {}

unsafe extern "C" fn ColorDepthCallback(_s: *mut c_void) {}

unsafe extern "C" fn DriverInfoCallback(_s: *mut c_void) {
    UI_PushMenu(DrvInfo_MenuDraw as *const c_void, DrvInfo_MenuKey as *const c_void);
}

unsafe extern "C" fn LightingCallback(_s: *mut c_void) {}

unsafe extern "C" fn ApplyChanges(_unused: *mut c_void) {
    match s_texturebits_box.curvalue {
        0 => {
            Cvar_SetValue(b"r_texturebits\0" as *const c_char, 0.0);
            Cvar_SetValue(b"r_ext_compress_textures\0" as *const c_char, 1.0);
        }
        1 => {
            Cvar_SetValue(b"r_texturebits\0" as *const c_char, 16.0);
            Cvar_SetValue(b"r_ext_compress_textures\0" as *const c_char, 0.0);
        }
        2 => {
            Cvar_SetValue(b"r_texturebits\0" as *const c_char, 32.0);
            Cvar_SetValue(b"r_ext_compress_textures\0" as *const c_char, 0.0);
        }
        _ => {}
    }
    Cvar_SetValue(b"r_picmip\0" as *const c_char, (3 - s_tq_slider.curvalue as c_int) as f32);
    Cvar_SetValue(b"r_allowExtensions\0" as *const c_char, s_allow_extensions_box.curvalue as f32);
    Cvar_SetValue(b"r_mode\0" as *const c_char, s_mode_list.curvalue as f32);
    Cvar_SetValue(b"r_fullscreen\0" as *const c_char, s_fs_box.curvalue as f32);
    if *s_drivers[s_driver_list.curvalue as usize] != 0 {
        Cvar_Set(b"r_glDriver\0" as *const c_char, s_drivers[s_driver_list.curvalue as usize]);
    }
    match s_colordepth_list.curvalue {
        0 => {
            Cvar_SetValue(b"r_colorbits\0" as *const c_char, 0.0);
            Cvar_SetValue(b"r_depthbits\0" as *const c_char, 0.0);
            Cvar_SetValue(b"r_stencilbits\0" as *const c_char, 0.0);
        }
        1 => {
            Cvar_SetValue(b"r_colorbits\0" as *const c_char, 16.0);
            Cvar_SetValue(b"r_depthbits\0" as *const c_char, 16.0);
            Cvar_SetValue(b"r_stencilbits\0" as *const c_char, 0.0);
        }
        2 => {
            Cvar_SetValue(b"r_colorbits\0" as *const c_char, 32.0);
            Cvar_SetValue(b"r_depthbits\0" as *const c_char, 24.0);
        }
        _ => {}
    }
    Cvar_SetValue(b"r_vertexLight\0" as *const c_char, s_lighting_box.curvalue as f32);

    if s_geometry_box.curvalue != 0 {
        Cvar_SetValue(b"r_lodBias\0" as *const c_char, 0.0);
        Cvar_SetValue(b"r_subdivisions\0" as *const c_char, 4.0);
    } else {
        Cvar_SetValue(b"r_lodBias\0" as *const c_char, 1.0);
        Cvar_SetValue(b"r_subdivisions\0" as *const c_char, 999.0);
    }

    if s_filter_box.curvalue != 0 {
        Cvar_Set(b"r_textureMode\0" as *const c_char, b"GL_LINEAR_MIPMAP_LINEAR\0" as *const c_char);
    } else {
        Cvar_Set(b"r_textureMode\0" as *const c_char, b"GL_LINEAR_MIPMAP_NEAREST\0" as *const c_char);
    }

    UI_ForceMenuOff();

    CL_Vid_Restart_f();

    VID_MenuInit();

    // s_fs_box.curvalue = Cvar_VariableValue( "r_fullscreen" );
}

// /*
// ** VID_MenuInit
// */
pub unsafe extern "C" fn VID_MenuInit() {
    let tq_names: [*const c_char; 4] = [
        b"compressed\0" as *const c_char,
        b"16-bit\0" as *const c_char,
        b"32-bit\0" as *const c_char,
        core::ptr::null(),
    ];

    let s_graphics_options_names: [*const c_char; 6] = [
        b"high quality\0" as *const c_char,
        b"normal\0" as *const c_char,
        b"fast\0" as *const c_char,
        b"fastest\0" as *const c_char,
        b"custom\0" as *const c_char,
        core::ptr::null(),
    ];

    let lighting_names: [*const c_char; 3] = [
        b"lightmap\0" as *const c_char,
        b"vertex\0" as *const c_char,
        core::ptr::null(),
    ];

    let colordepth_names: [*const c_char; 4] = [
        b"default\0" as *const c_char,
        b"16-bit\0" as *const c_char,
        b"32-bit\0" as *const c_char,
        core::ptr::null(),
    ];

    let resolutions: [*const c_char; 13] = [
        b"[320 240  ]\0" as *const c_char,
        b"[400 300  ]\0" as *const c_char,
        b"[512 384  ]\0" as *const c_char,
        b"[640 480  ]\0" as *const c_char,
        b"[800 600  ]\0" as *const c_char,
        b"[960 720  ]\0" as *const c_char,
        b"[1024 768 ]\0" as *const c_char,
        b"[1152 864 ]\0" as *const c_char,
        b"[1280 960 ]\0" as *const c_char,
        b"[1600 1200]\0" as *const c_char,
        b"[2048 1536]\0" as *const c_char,
        b"[856 480 W]\0" as *const c_char,
        core::ptr::null(),
    ];

    let filter_names: [*const c_char; 3] = [
        b"bilinear\0" as *const c_char,
        b"trilinear\0" as *const c_char,
        core::ptr::null(),
    ];

    let quality_names: [*const c_char; 3] = [
        b"low\0" as *const c_char,
        b"high\0" as *const c_char,
        core::ptr::null(),
    ];

    let enabled_names: [*const c_char; 3] = [
        b"disabled\0" as *const c_char,
        b"enabled\0" as *const c_char,
        core::ptr::null(),
    ];

    let mut y: c_int = 0;
    let mut i: c_int;

    s_menu.x = (SCREEN_WIDTH as f32 * 0.50) as c_int;
    s_menu.nitems = 0;
    s_menu.wrapAround = 1;

    s_graphics_options_list.generic.type_ = MTYPE_SPINCONTROL;
    s_graphics_options_list.generic.name = b"graphics mode\0" as *const c_char;
    s_graphics_options_list.generic.x = 0;
    s_graphics_options_list.generic.y = y;
    s_graphics_options_list.generic.callback = GraphicsOptionsCallback as *mut c_void;
    s_graphics_options_list.itemnames = &s_graphics_options_names[0];

    s_driver_list.generic.type_ = MTYPE_SPINCONTROL;
    s_driver_list.generic.name = b"driver\0" as *const c_char;
    s_driver_list.generic.x = 0;
    y += 18;
    s_driver_list.generic.y = y;

    let p = Cvar_VariableString(b"r_glDriver\0" as *const c_char);
    i = 0;
    while !s_drivers[i as usize].is_null() {
        if strcmp(s_drivers[i as usize], p) == 0 {
            break;
        }
        i += 1;
    }
    if s_drivers[i as usize].is_null() {
        i -= 1; // go back one, to default 'custom'
    }
    s_driver_list.curvalue = i;

    s_driver_list.itemnames = &s_driver_names[0];

    // references/modifies "r_allowExtensions"
    s_allow_extensions_box.generic.type_ = MTYPE_SPINCONTROL;
    s_allow_extensions_box.generic.x = 0;
    y += 18;
    s_allow_extensions_box.generic.y = y;
    s_allow_extensions_box.generic.name = b"OpenGL extensions\0" as *const c_char;
    s_allow_extensions_box.generic.callback = ExtensionsCallback as *mut c_void;
    s_allow_extensions_box.itemnames = &enabled_names[0];

    // references/modifies "r_mode"
    s_mode_list.generic.type_ = MTYPE_SPINCONTROL;
    s_mode_list.generic.name = b"video mode\0" as *const c_char;
    s_mode_list.generic.x = 0;
    y += 36;
    s_mode_list.generic.y = y;
    s_mode_list.itemnames = &resolutions[0];
    s_mode_list.generic.callback = ModeCallback as *mut c_void;

    // references "r_colorbits"
    s_colordepth_list.generic.type_ = MTYPE_SPINCONTROL;
    s_colordepth_list.generic.name = b"color depth\0" as *const c_char;
    s_colordepth_list.generic.x = 0;
    y += 18;
    s_colordepth_list.generic.y = y;
    s_colordepth_list.itemnames = &colordepth_names[0];
    s_colordepth_list.generic.callback = ColorDepthCallback as *mut c_void;

    // references/modifies "r_fullscreen"
    s_fs_box.generic.type_ = MTYPE_RADIOBUTTON;
    s_fs_box.generic.x = 0;
    y += 18;
    s_fs_box.generic.y = y;
    s_fs_box.generic.name = b"fullscreen\0" as *const c_char;
    s_fs_box.generic.callback = FullscreenCallback as *mut c_void;

    // references/modifies "r_vertexLight"
    s_lighting_box.generic.type_ = MTYPE_SPINCONTROL;
    s_lighting_box.generic.x = 0;
    y += 18;
    s_lighting_box.generic.y = y;
    s_lighting_box.generic.name = b"lighting\0" as *const c_char;
    s_lighting_box.itemnames = &lighting_names[0];
    s_lighting_box.generic.callback = LightingCallback as *mut c_void;

    // references/modifies "r_lodBias" & "subdivisions"
    s_geometry_box.generic.type_ = MTYPE_SPINCONTROL;
    s_geometry_box.generic.x = 0;
    y += 18;
    s_geometry_box.generic.y = y;
    s_geometry_box.generic.name = b"geometric detail\0" as *const c_char;
    s_geometry_box.itemnames = &quality_names[0];

    // references/modifies "r_picmip"
    s_tq_slider.generic.type_ = MTYPE_SLIDER;
    s_tq_slider.generic.x = 0;
    y += 18;
    s_tq_slider.generic.y = y;
    s_tq_slider.generic.name = b"texture detail\0" as *const c_char;
    s_tq_slider.generic.callback = TextureDetailCallback as *mut c_void;
    s_tq_slider.minvalue = 0.0;
    s_tq_slider.maxvalue = 3.0;

    // references/modifies "r_textureBits"
    s_texturebits_box.generic.type_ = MTYPE_SPINCONTROL;
    s_texturebits_box.generic.x = 0;
    y += 18;
    s_texturebits_box.generic.y = y;
    s_texturebits_box.generic.name = b"texture quality\0" as *const c_char;
    s_texturebits_box.generic.callback = TextureQualityCallback as *mut c_void;
    s_texturebits_box.itemnames = &tq_names[0];

    // references/modifies "r_textureMode"
    s_filter_box.generic.type_ = MTYPE_SPINCONTROL;
    s_filter_box.generic.x = 0;
    y += 18;
    s_filter_box.generic.y = y;
    s_filter_box.generic.name = b"texture filter\0" as *const c_char;
    s_filter_box.itemnames = &filter_names[0];

    s_driverinfo_action.generic.type_ = MTYPE_ACTION;
    s_driverinfo_action.generic.name = b"driver information\0" as *const c_char;
    s_driverinfo_action.generic.x = 0;
    y += 36;
    s_driverinfo_action.generic.y = y;
    s_driverinfo_action.generic.callback = DriverInfoCallback as *mut c_void;

    s_apply_action.generic.type_ = MTYPE_ACTION;
    s_apply_action.generic.name = b"apply\0" as *const c_char;
    s_apply_action.generic.x = 0;
    y += 36;
    s_apply_action.generic.y = y;
    s_apply_action.generic.callback = ApplyChanges as *mut c_void;
    s_apply_action.generic.flags = QMF_GRAYED;

    SetMenuItemValues();
    GetInitialVideoVars();

    Menu_AddItem(&mut s_menu, &mut s_graphics_options_list as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_driver_list as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_allow_extensions_box as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_mode_list as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_colordepth_list as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_fs_box as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_lighting_box as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_geometry_box as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_tq_slider as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_texturebits_box as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_filter_box as *mut _ as *mut c_void);

    Menu_AddItem(&mut s_menu, &mut s_driverinfo_action as *mut _ as *mut c_void);
    Menu_AddItem(&mut s_menu, &mut s_apply_action as *mut _ as *mut c_void);

    Menu_Center(&mut s_menu);
    s_menu.y -= 6;
}

// /*
// ================
// VID_MenuDraw
// ================
// */
pub unsafe extern "C" fn VID_MenuDraw() {
    UpdateMenuItemValues();
    Menu_AdjustCursor(&mut s_menu, 1);
    Menu_Draw(&mut s_menu);
}

// /*
// ================
// VID_MenuKey
// ================
// */
pub unsafe extern "C" fn VID_MenuKey(key: c_int) -> *const c_char {
    let m: *mut menuframework_s = &mut s_menu;
    // let sound: *const c_char = b"sound/misc/menu1.wav\0" as *const c_char;

    if key == K_ENTER {
        if Menu_SelectItem(m) == 0 {
            ApplyChanges(core::ptr::null_mut());
        }
        return core::ptr::null();
    }
    Default_MenuKey(m, key)
}

unsafe fn DrvInfo_MenuDraw() {
    let labelColor: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    let textColor: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let mut i: c_int = 14;
    let mut extensionsString: [c_char; 1024] = [0; 1024];
    let mut eptr: *mut c_char = &mut extensionsString[0];

    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        BIGCHAR_HEIGHT * 3,
        b"VENDOR:\0" as *const c_char,
        &labelColor[0],
    );
    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        BIGCHAR_HEIGHT * 4,
        Cvar_VariableString(b"gl_vendor\0" as *const c_char),
        &textColor[0],
    );
    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        (BIGCHAR_HEIGHT as f32 * 5.5) as c_int,
        b"VERSION:\0" as *const c_char,
        &labelColor[0],
    );
    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        (BIGCHAR_HEIGHT as f32 * 6.5) as c_int,
        Cvar_VariableString(b"gl_version\0" as *const c_char),
        &textColor[0],
    );
    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        BIGCHAR_HEIGHT * 8,
        b"RENDERER:\0" as *const c_char,
        &labelColor[0],
    );
    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        BIGCHAR_HEIGHT * 9,
        Cvar_VariableString(b"gl_renderer\0" as *const c_char),
        &textColor[0],
    );
    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        (BIGCHAR_HEIGHT as f32 * 10.5) as c_int,
        b"PIXELFORMAT:\0" as *const c_char,
        &labelColor[0],
    );
    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        (BIGCHAR_HEIGHT as f32 * 11.5) as c_int,
        Cvar_VariableString(b"gl_pixelformat\0" as *const c_char),
        &textColor[0],
    );

    SCR_DrawBigStringColor(
        BIGCHAR_WIDTH * 4,
        BIGCHAR_HEIGHT * 13,
        b"EXTENSIONS:\0" as *const c_char,
        &labelColor[0],
    );
    strcpy(eptr, Cvar_VariableString(b"gl_extensions\0" as *const c_char));
    while i < 25 && *eptr != 0 {
        while *eptr != 0 {
            let mut buf: [c_char; 2] = [b' ' as c_char, 0];
            let mut j: c_int = BIGCHAR_WIDTH * 6;

            while *eptr != 0 && *eptr != (b' ' as c_char) {
                buf[0] = *eptr;
                SCR_DrawBigStringColor(j, i * BIGCHAR_HEIGHT, &buf[0], &textColor[0]);
                j += BIGCHAR_WIDTH;
                eptr = eptr.add(1);
            }

            i += 1;

            while *eptr != 0 && *eptr == (b' ' as c_char) {
                eptr = eptr.add(1);
            }
        }
    }
}

unsafe fn DrvInfo_MenuKey(key: c_int) -> *const c_char {
    if key == K_ESCAPE {
        UI_PopMenu();
    }
    core::ptr::null()
}
