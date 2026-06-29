//! `cm_terrainmap.cpp` — terrain map automap image generation and management.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::game::q_shared_h::{byte, vec3_t, qboolean, QFALSE, QTRUE};
use crate::codemp::qcommon::cm_draw_h::{
    CPixel32, CDraw32, PIXPOS, CLAMP, ALPHA_PIX,
};
use crate::codemp::qcommon::cm_landscape_h::CCMLandScape;
use core::ffi::{c_char, c_int, c_long, c_void};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// Constants from cm_terrainmap.h
#[cfg(target_os = "xbox")]
pub const TM_WIDTH: usize = 64;
#[cfg(target_os = "xbox")]
pub const TM_HEIGHT: usize = 64;
#[cfg(target_os = "xbox")]
pub const TM_BORDER: usize = 4;

#[cfg(not(target_os = "xbox"))]
pub const TM_WIDTH: usize = 512;
#[cfg(not(target_os = "xbox"))]
pub const TM_HEIGHT: usize = 512;
#[cfg(not(target_os = "xbox"))]
pub const TM_BORDER: usize = 16;

pub const TM_REAL_WIDTH: usize = TM_WIDTH - TM_BORDER - TM_BORDER;
pub const TM_REAL_HEIGHT: usize = TM_HEIGHT - TM_BORDER - TM_BORDER;

// Side color constants
pub const SIDE_NONE: c_int = 0;
pub const SIDE_BLUE: c_int = 1;
pub const SIDE_RED: c_int = 2;

// Extern functions from renderer and memory allocators
extern "C" {
    #[cfg(target_os = "xbox")]
    fn R_LoadImage(
        shortname: *const c_char,
        pic: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
        mipcount: *mut c_int,
        format: *mut c_int,
    );

    #[cfg(not(target_os = "xbox"))]
    fn R_LoadImage(
        name: *const c_char,
        pic: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
        format: *mut c_int,
    );

    fn R_CreateAutomapImage(
        name: *const c_char,
        pic: *const byte,
        width: c_int,
        height: c_int,
        mipmap: qboolean,
        allowPicmip: qboolean,
        allowTC: qboolean,
        glWrapClampMode: c_int,
    );

    fn Z_Free(ptr: *mut c_void);
    fn va(fmt: *const c_char, ...) -> *mut c_char;
    fn PNG_Save(
        filename: *const c_char,
        data: *const byte,
        width: c_int,
        height: c_int,
        depth: c_int,
    );
    fn RotatePointAroundVector(
        dst: *mut vec3_t,
        axis: *const vec3_t,
        src: *const vec3_t,
        angle: f32,
    );
}

// Hack. This shouldn't be here, but it's easier than including tr_local.h
pub type GLenum = c_int;

// simple function for getting a proper color for a side
#[inline]
fn SideColor(side: c_int) -> CPixel32 {
    let mut col = CPixel32::new(255, 255, 255, 255);
    match side {
        SIDE_BLUE => col = CPixel32::new(0, 0, 192, 255),
        SIDE_RED => col = CPixel32::new(192, 0, 0, 255),
        _ => {}
    }
    col
}

#[repr(C)]
pub struct CTerrainMap {
    // image to output
    pub mImage: *mut byte,
    // src data for image, color and bump
    pub mBufImage: *mut byte,

    pub mSymBld: *mut byte,
    pub mSymBldWidth: c_int,
    pub mSymBldHeight: c_int,

    pub mSymStart: *mut byte,
    pub mSymStartWidth: c_int,
    pub mSymStartHeight: c_int,

    pub mSymEnd: *mut byte,
    pub mSymEndWidth: c_int,
    pub mSymEndHeight: c_int,

    pub mSymObjective: *mut byte,
    pub mSymObjectiveWidth: c_int,
    pub mSymObjectiveHeight: c_int,

    pub mLandscape: *mut CCMLandScape,
}

impl CTerrainMap {
    pub unsafe fn new(landscape: *mut CCMLandScape) -> *mut CTerrainMap {
        let terrain_map = Box::new(CTerrainMap {
            mImage: Box::into_raw(vec![0u8; TM_HEIGHT * TM_WIDTH * 4].into_boxed_slice()) as *mut byte,
            mBufImage: Box::into_raw(vec![0u8; TM_HEIGHT * TM_WIDTH * 4].into_boxed_slice()) as *mut byte,
            mSymBld: null_mut(),
            mSymBldWidth: 0,
            mSymBldHeight: 0,
            mSymStart: null_mut(),
            mSymStartWidth: 0,
            mSymStartHeight: 0,
            mSymEnd: null_mut(),
            mSymEndWidth: 0,
            mSymEndHeight: 0,
            mSymObjective: null_mut(),
            mSymObjectiveWidth: 0,
            mSymObjectiveHeight: 0,
            mLandscape: landscape,
        });
        let ptr = Box::into_raw(terrain_map);

        (*ptr).ApplyBackground();
        (*ptr).ApplyHeightmap();

        let mut draw = CDraw32 {};
        draw.SetBuffer((*ptr).mImage as *mut CPixel32);
        draw.SetBufferSize(TM_WIDTH as c_long, TM_HEIGHT as c_long, TM_WIDTH as c_long);

        // create version with paths and water shown
        let mut x: c_int;
        let mut y: c_int;
        let mut water: c_int;
        let mut land: c_int;

        y = 0;
        while y < TM_HEIGHT as c_int {
            x = 0;
            while x < TM_WIDTH as c_int {
                let cp_idx = PIXPOS(x as c_long, y as c_long, TM_WIDTH as c_long) as usize;
                let cp = ((*ptr).mBufImage as *mut CPixel32).add(cp_idx).read();
                land = CLAMP(((255 - cp.a as c_long) * 2) / 3, 0, 255);
                water = CLAMP(
                    ((*(*ptr).mLandscape).GetBaseWaterHeight() - cp.a as c_int) as c_long * 4,
                    0,
                    255,
                );
                let mut cp = cp;
                cp.a = 255;

                if x > TM_BORDER as c_int
                    && x < (TM_WIDTH as c_int - TM_BORDER as c_int)
                    && y > TM_BORDER as c_int
                    && y < (TM_WIDTH as c_int - TM_BORDER as c_int)
                {
                    cp = ALPHA_PIX(
                        CPixel32::new(0, 0, 0, 255),
                        cp,
                        land,
                        256 - land,
                    );
                    if water > 0 {
                        cp = ALPHA_PIX(
                            CPixel32::new(0, 0, 255, 255),
                            cp,
                            water,
                            256 - water,
                        );
                    }
                }

                draw.PutPix(x as c_long, y as c_long, cp);
                x += 1;
            }
            y += 1;
        }

        // Load icons for symbols on map
        #[cfg(target_os = "xbox")]
        {
            let mut mipcount: c_int = 0;
            let mut format: GLenum = 0;

            R_LoadImage(
                b"gfx/menus/rmg/start\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymStart,
                &mut (*ptr).mSymStartWidth,
                &mut (*ptr).mSymStartHeight,
                &mut mipcount,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/end\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymEnd,
                &mut (*ptr).mSymEndWidth,
                &mut (*ptr).mSymEndHeight,
                &mut mipcount,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/objective\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymObjective,
                &mut (*ptr).mSymObjectiveWidth,
                &mut (*ptr).mSymObjectiveHeight,
                &mut mipcount,
                &mut format,
            );

            R_LoadImage(
                b"gfx/menus/rmg/building\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymBld,
                &mut (*ptr).mSymBldWidth,
                &mut (*ptr).mSymBldHeight,
                &mut mipcount,
                &mut format,
            );
        }

        #[cfg(not(target_os = "xbox"))]
        {
            let mut format: GLenum = 0;

            R_LoadImage(
                b"gfx/menus/rmg/start\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymStart,
                &mut (*ptr).mSymStartWidth,
                &mut (*ptr).mSymStartHeight,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/end\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymEnd,
                &mut (*ptr).mSymEndWidth,
                &mut (*ptr).mSymEndHeight,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/objective\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymObjective,
                &mut (*ptr).mSymObjectiveWidth,
                &mut (*ptr).mSymObjectiveHeight,
                &mut format,
            );

            R_LoadImage(
                b"gfx/menus/rmg/building\0".as_ptr() as *const c_char,
                &mut (*ptr).mSymBld,
                &mut (*ptr).mSymBldWidth,
                &mut (*ptr).mSymBldHeight,
                &mut format,
            );
        }

        ptr
    }

    pub unsafe fn drop(&mut self) {
        if !self.mSymStart.is_null() {
            Z_Free(self.mSymStart as *mut c_void);
            self.mSymStart = null_mut();
        }

        if !self.mSymEnd.is_null() {
            Z_Free(self.mSymEnd as *mut c_void);
            self.mSymEnd = null_mut();
        }

        if !self.mSymBld.is_null() {
            Z_Free(self.mSymBld as *mut c_void);
            self.mSymBld = null_mut();
        }

        if !self.mSymObjective.is_null() {
            Z_Free(self.mSymObjective as *mut c_void);
            self.mSymObjective = null_mut();
        }

        CDraw32::CleanUp();
    }

    pub unsafe fn ApplyBackground(&mut self) {
        let mut x: c_int;
        let mut y: c_int;
        let mut outPos: *mut byte;
        let mut xRel: f32;
        let mut yRel: f32;
        let mut xInc: f32;
        let mut yInc: f32;
        let mut backgroundImage: *mut byte;
        let mut backgroundWidth: c_int = 0;
        let mut backgroundHeight: c_int = 0;
        let mut backgroundDepth: c_int;
        let mut pos: c_int;
        let mut format: GLenum = 0;

        // memset(mImage, 255, sizeof(mBufImage));
        let buf_len = TM_HEIGHT * TM_WIDTH * 4;
        core::ptr::write_bytes(self.mImage, 255, buf_len);

        // R_LoadImage("textures\\kamchatka\\ice", &backgroundImage, &backgroundWidth, &backgroundHeight, &format);0
        backgroundDepth = 4;
        #[cfg(target_os = "xbox")]
        {
            let mut mipcount: c_int = 0;
            R_LoadImage(
                b"gfx\\menus\\rmg\\01_bg\0".as_ptr() as *const c_char,
                &mut backgroundImage,
                &mut backgroundWidth,
                &mut backgroundHeight,
                &mut mipcount,
                &mut format,
            );
        }

        #[cfg(not(target_os = "xbox"))]
        {
            R_LoadImage(
                b"gfx\\menus\\rmg\\01_bg\0".as_ptr() as *const c_char,
                &mut backgroundImage,
                &mut backgroundWidth,
                &mut backgroundHeight,
                &mut format,
            );
        }

        if !backgroundImage.is_null() {
            outPos = self.mBufImage;
            xInc = backgroundWidth as f32 / TM_WIDTH as f32;
            yInc = backgroundHeight as f32 / TM_HEIGHT as f32;

            yRel = 0.0;
            y = 0;
            while y < TM_HEIGHT as c_int {
                xRel = 0.0;
                x = 0;
                while x < TM_WIDTH as c_int {
                    pos = ((yRel as c_int * backgroundWidth) + xRel as c_int) * 4;
                    *outPos = *backgroundImage.add(pos as usize);
                    outPos = outPos.add(1);
                    *outPos = *backgroundImage.add((pos + 1) as usize);
                    outPos = outPos.add(1);
                    *outPos = *backgroundImage.add((pos + 2) as usize);
                    outPos = outPos.add(2);
                    xRel += xInc;
                    x += 1;
                }
                yRel += yInc;
                y += 1;
            }
            Z_Free(backgroundImage as *mut c_void);
        }
    }

    pub unsafe fn ApplyHeightmap(&mut self) {
        let mut x: c_int;
        let mut y: c_int;
        let inPos: *mut byte = (*self.mLandscape).GetHeightMap();
        let width: c_int = (*self.mLandscape).GetRealWidth();
        let height: c_int = (*self.mLandscape).GetRealHeight();
        let mut outPos: *mut byte;
        let mut tempColor: c_int;
        let mut xRel: f32;
        let mut yRel: f32;
        let xInc: f32;
        let yInc: f32;
        let mut count: c_int;

        outPos = self.mBufImage;
        outPos = outPos.add(((TM_BORDER * TM_WIDTH) + TM_BORDER) * 4);
        xInc = width as f32 / TM_REAL_WIDTH as f32;
        yInc = height as f32 / TM_REAL_HEIGHT as f32;

        // add in height map as alpha
        yRel = 0.0;
        y = 0;
        while y < TM_REAL_HEIGHT as c_int {
            // x is flipped!
            xRel = width as f32;
            x = 0;
            while x < TM_REAL_WIDTH as c_int {
                count = 1;
                tempColor = *inPos.add(((yRel as c_int * width) + xRel as c_int) as usize) as c_int;
                if yRel >= 1.0 {
                    tempColor += *inPos.add(
                        (((yRel - 0.5) as c_int * width) + xRel as c_int) as usize,
                    ) as c_int;
                    count += 1;
                }
                if yRel <= (height - 2) as f32 {
                    tempColor += *inPos.add(
                        (((yRel + 0.5) as c_int * width) + xRel as c_int) as usize,
                    ) as c_int;
                    count += 1;
                }
                if xRel >= 1.0 {
                    tempColor += *inPos.add(
                        ((yRel as c_int * width) + (xRel - 0.5) as c_int) as usize,
                    ) as c_int;
                    count += 1;
                }
                if xRel <= (width - 2) as f32 {
                    tempColor += *inPos.add(
                        ((yRel as c_int * width) + (xRel + 0.5) as c_int) as usize,
                    ) as c_int;
                    count += 1;
                }
                tempColor /= count;

                *outPos.add(3) = tempColor as byte;
                outPos = outPos.add(4);

                // x is flipped!
                xRel -= xInc;
                x += 1;
            }
            outPos = outPos.add(TM_BORDER * 4 * 2);

            yRel += yInc;
            y += 1;
        }
    }

    // Convert position in game coords to automap coords
    pub unsafe fn ConvertPos(&self, x: &mut c_int, y: &mut c_int) {
        *x = (((*x) - (*(*self.mLandscape).GetMins())[0] as c_int)
            / (*(*self.mLandscape).GetSize())[0] as c_int)
            * TM_REAL_WIDTH as c_int;
        *y = (((*y) - (*(*self.mLandscape).GetMins())[1] as c_int)
            / (*(*self.mLandscape).GetSize())[1] as c_int)
            * TM_REAL_HEIGHT as c_int;

        // x is flipped!
        *x = TM_REAL_WIDTH as c_int - *x - 1;

        // border
        *x += TM_BORDER as c_int;
        *y += TM_BORDER as c_int;
    }

    pub unsafe fn AddStart(&mut self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32 {};
        draw.BlitColor(
            x - self.mSymStartWidth / 2,
            y - self.mSymStartHeight / 2,
            self.mSymStartWidth,
            self.mSymStartHeight,
            self.mSymStart as *mut CPixel32,
            0,
            0,
            self.mSymStartWidth,
            SideColor(side),
        );
    }

    pub unsafe fn AddEnd(&mut self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32 {};
        draw.BlitColor(
            x - self.mSymEndWidth / 2,
            y - self.mSymEndHeight / 2,
            self.mSymEndWidth,
            self.mSymEndHeight,
            self.mSymEnd as *mut CPixel32,
            0,
            0,
            self.mSymEndWidth,
            SideColor(side),
        );
    }

    pub unsafe fn AddObjective(&mut self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32 {};
        draw.BlitColor(
            x - self.mSymObjectiveWidth / 2,
            y - self.mSymObjectiveHeight / 2,
            self.mSymObjectiveWidth,
            self.mSymObjectiveHeight,
            self.mSymObjective as *mut CPixel32,
            0,
            0,
            self.mSymObjectiveWidth,
            SideColor(side),
        );
    }

    pub unsafe fn AddBuilding(&mut self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32 {};
        draw.BlitColor(
            x - self.mSymBldWidth / 2,
            y - self.mSymBldHeight / 2,
            self.mSymBldWidth,
            self.mSymBldHeight,
            self.mSymBld as *mut CPixel32,
            0,
            0,
            self.mSymBldWidth,
            SideColor(side),
        );
    }

    pub unsafe fn AddNPC(&mut self, mut x: c_int, mut y: c_int, friendly: bool) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32 {};
        if friendly {
            draw.DrawCircle(
                x,
                y,
                3,
                CPixel32::new(0, 192, 0, 255),
                CPixel32::new(0, 0, 0, 0),
            );
        } else {
            draw.DrawCircle(
                x,
                y,
                3,
                CPixel32::new(192, 0, 0, 255),
                CPixel32::new(0, 0, 0, 0),
            );
        }
    }

    pub unsafe fn AddNode(&mut self, mut x: c_int, mut y: c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32 {};
        draw.DrawCircle(
            x,
            y,
            20,
            CPixel32::new(255, 255, 255, 255),
            CPixel32::new(0, 0, 0, 0),
        );
    }

    pub unsafe fn AddWallRect(&mut self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32 {};
        match side {
            SIDE_BLUE => {
                draw.DrawBox(x - 1, y - 1, 3, 3, CPixel32::new(0, 0, 192, 128));
            }
            SIDE_RED => {
                draw.DrawBox(x - 1, y - 1, 3, 3, CPixel32::new(192, 0, 0, 128));
            }
            _ => {
                draw.DrawBox(x - 1, y - 1, 3, 3, CPixel32::new(192, 192, 192, 128));
            }
        }
    }

    pub unsafe fn AddPlayer(&mut self, origin: vec3_t, angles: vec3_t) {
        // draw player start on automap
        let mut draw = CDraw32 {};

        let mut up: vec3_t = [0.0, 0.0, 0.0];
        let pt: [vec3_t; 4] = [
            [0.0, 0.0, 0.0],
            [-5.0, -5.0, 0.0],
            [10.0, 0.0, 0.0],
            [-5.0, 5.0, 0.0],
        ];
        let mut p: vec3_t = [0.0, 0.0, 0.0];
        let mut x: c_int;
        let mut y: c_int;
        let mut i: c_int;
        let facing: f32;
        let mut poly: [crate::codemp::qcommon::cm_draw_h::POINT; 4] = Default::default();

        facing = angles[1];

        up[0] = 0.0;
        up[1] = 0.0;
        up[2] = 1.0;

        x = origin[0] as c_int;
        y = origin[1] as c_int;
        self.ConvertPos(&mut x, &mut y);
        x += 1;
        y += 1;

        i = 0;
        while i < 4 {
            RotatePointAroundVector(&mut p, &up, &pt[i as usize], facing);
            poly[i as usize].x = (-p[0] + x as f32) as c_long;
            poly[i as usize].y = (p[1] + y as f32) as c_long;
            i += 1;
        }

        // draw arrowhead shadow
        draw.DrawPolygon(
            4,
            poly.as_mut_ptr(),
            CPixel32::new(0, 0, 0, 128),
            CPixel32::new(0, 0, 0, 128),
        );

        // draw arrowhead
        i = 0;
        while i < 4 {
            poly[i as usize].x -= 1;
            poly[i as usize].y -= 1;
            i += 1;
        }
        draw.DrawPolygon(
            4,
            poly.as_mut_ptr(),
            CPixel32::new(255, 255, 255, 255),
            CPixel32::new(255, 255, 255, 255),
        );
    }

    pub unsafe fn Upload(&mut self, player_origin: vec3_t, player_angles: vec3_t) {
        let mut draw = CDraw32 {};

        // copy completed map to mBufImage
        draw.SetBuffer(self.mBufImage as *mut CPixel32);
        draw.SetBufferSize(TM_WIDTH as c_long, TM_HEIGHT as c_long, TM_WIDTH as c_long);

        draw.Blit(
            0,
            0,
            TM_WIDTH as c_long,
            TM_HEIGHT as c_long,
            self.mImage as *mut CPixel32,
            0,
            0,
            TM_WIDTH as c_long,
        );

        // now draw player's location on map
        if !player_origin.as_ptr().is_null() {
            self.AddPlayer(player_origin, player_angles);
        }

        draw.SetAlphaBuffer(255);

        R_CreateAutomapImage(
            b"*automap\0".as_ptr() as *const c_char,
            draw.buffer as *const byte,
            TM_WIDTH as c_int,
            TM_HEIGHT as c_int,
            QFALSE,
            QFALSE,
            QTRUE,
            QFALSE,
        );

        draw.SetBuffer(self.mImage as *mut CPixel32);
    }

    pub unsafe fn SaveImageToDisk(
        &self,
        terrainName: *const c_char,
        missionName: *const c_char,
        seed: *const c_char,
    ) {
        // ri.COM_SavePNG(va("save/%s_%s_%s.png", terrainName, missionName, seed),
        //		(unsigned char *)mImage, TM_WIDTH, TM_HEIGHT, 4);
        PNG_Save(
            va(
                b"save/%s_%s_%s.png\0".as_ptr() as *const c_char,
                terrainName,
                missionName,
                seed,
            ),
            self.mImage as *const byte,
            TM_WIDTH as c_int,
            TM_HEIGHT as c_int,
            4,
        );
    }
}

static mut TerrainMap: *mut CTerrainMap = null_mut();

pub unsafe fn CM_TM_Create(landscape: *mut CCMLandScape) {
    if !TerrainMap.is_null() {
        CM_TM_Free();
    }

    TerrainMap = CTerrainMap::new(landscape);
}

pub unsafe fn CM_TM_Free() {
    if !TerrainMap.is_null() {
        (*TerrainMap).drop();
        // Free the allocation
        let _ = Box::from_raw(TerrainMap);
        TerrainMap = null_mut();
    }
}

pub unsafe fn CM_TM_AddStart(x: c_int, y: c_int, side: c_int) {
    if !TerrainMap.is_null() {
        (*TerrainMap).AddStart(x, y, side);
    }
}

pub unsafe fn CM_TM_AddEnd(x: c_int, y: c_int, side: c_int) {
    if !TerrainMap.is_null() {
        (*TerrainMap).AddEnd(x, y, side);
    }
}

pub unsafe fn CM_TM_AddObjective(x: c_int, y: c_int, side: c_int) {
    if !TerrainMap.is_null() {
        (*TerrainMap).AddObjective(x, y, side);
    }
}

pub unsafe fn CM_TM_AddNPC(x: c_int, y: c_int, friendly: bool) {
    if !TerrainMap.is_null() {
        (*TerrainMap).AddNPC(x, y, friendly);
    }
}

pub unsafe fn CM_TM_AddNode(x: c_int, y: c_int) {
    if !TerrainMap.is_null() {
        (*TerrainMap).AddNode(x, y);
    }
}

pub unsafe fn CM_TM_AddBuilding(x: c_int, y: c_int, side: c_int) {
    if !TerrainMap.is_null() {
        (*TerrainMap).AddBuilding(x, y, side);
    }
}

pub unsafe fn CM_TM_AddWallRect(x: c_int, y: c_int, side: c_int) {
    if !TerrainMap.is_null() {
        (*TerrainMap).AddWallRect(x, y, side);
    }
}

pub unsafe fn CM_TM_Upload(player_origin: vec3_t, player_angles: vec3_t) {
    if !TerrainMap.is_null() {
        (*TerrainMap).Upload(player_origin, player_angles);
    }
}

pub unsafe fn CM_TM_SaveImageToDisk(
    terrainName: *const c_char,
    missionName: *const c_char,
    seed: *const c_char,
) {
    if !TerrainMap.is_null() {
        // write out automap
        (*TerrainMap).SaveImageToDisk(terrainName, missionName, seed);
    }
}

pub unsafe fn CM_TM_ConvertPosition(
    x: &mut c_int,
    y: &mut c_int,
    Width: c_int,
    Height: c_int,
) {
    if !TerrainMap.is_null() {
        (*TerrainMap).ConvertPos(x, y);
        *x = *x * Width / TM_WIDTH as c_int;
        *y = *y * Height / TM_HEIGHT as c_int;
    }
}
