#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void, c_long};
use core::ptr::{addr_of_mut, null_mut};

// Type stubs from other headers (qcommon/q_shared.h, etc.)
pub type byte = u8;
pub type vec_t = f32;
pub type vec3_t = [f32; 3];

// Forward declaration stub for CCMLandScape (opaque)
#[repr(C)]
pub struct CCMLandScape {
    _opaque: [u8; 0],
}

// CPixel32 - 32-bit pixel with RGBA components
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CPixel32 {
    pub r: byte,
    pub g: byte,
    pub b: byte,
    pub a: byte,
}

impl CPixel32 {
    #[inline]
    pub const fn new(R: byte, G: byte, B: byte, A: byte) -> Self {
        CPixel32 {
            r: R,
            g: G,
            b: B,
            a: A,
        }
    }
}

// POINT - Windows API compatible point type
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct POINT {
    pub x: c_long,
    pub y: c_long,
}

// CDraw32 - Opaque drawing context
#[repr(C)]
pub struct CDraw32 {
    _opaque: [u8; 0],
}

impl CDraw32 {
    #[inline]
    pub fn new() -> Self {
        CDraw32 { _opaque: [] }
    }

    // Stub implementations - these would call into the renderer
    pub fn SetBuffer(&self, _buffer: *mut CPixel32) {}
    pub fn SetBufferSize(&self, _width: c_int, _height: c_int, _stride: c_int) {}
    pub fn PutPix(&self, _x: c_int, _y: c_int, _cp: CPixel32) {}
    pub fn BlitColor(&self, _x: c_int, _y: c_int, _width: c_int, _height: c_int,
                      _src: *mut CPixel32, _srcX: c_int, _srcY: c_int, _srcStride: c_int,
                      _col: CPixel32) {}
    pub fn DrawCircle(&self, _x: c_int, _y: c_int, _radius: c_int,
                       _outer: CPixel32, _inner: CPixel32) {}
    pub fn DrawBox(&self, _x: c_int, _y: c_int, _width: c_int, _height: c_int,
                    _col: CPixel32) {}
    pub fn DrawPolygon(&self, _count: c_int, _poly: *const POINT,
                       _outer: CPixel32, _inner: CPixel32) {}
    pub fn SetAlphaBuffer(&self, _alpha: c_int) {}
    pub fn CleanUp() {}
    pub fn buffer(&self) -> *mut u8 {
        null_mut()
    }
}

// Constants
pub const TM_WIDTH: usize = 512;
pub const TM_HEIGHT: usize = 512;
pub const TM_BORDER: usize = 16;
pub const TM_REAL_WIDTH: usize = TM_WIDTH - TM_BORDER - TM_BORDER;
pub const TM_REAL_HEIGHT: usize = TM_HEIGHT - TM_BORDER - TM_BORDER;

pub const SIDE_NONE: c_int = 0;
pub const SIDE_BLUE: c_int = 1;
pub const SIDE_RED: c_int = 2;

// Macros from cm_draw.h
#[inline]
pub const fn PIXPOS(x: usize, y: usize, stride: usize) -> usize {
    ((y * stride) + x)
}

#[inline]
pub const fn CLAMP(v: i32, l: i32, h: i32) -> i32 {
    if v < l {
        l
    } else if v > h {
        h
    } else {
        v
    }
}

#[inline]
pub fn ALPHA_PIX(x: CPixel32, y: CPixel32, alpha: i32, inv_alpha: i32) -> CPixel32 {
    CPixel32 {
        r: (((x.r as i32 * alpha + y.r as i32 * inv_alpha) >> 8) & 0xff) as byte,
        g: (((x.g as i32 * alpha + y.g as i32 * inv_alpha) >> 8) & 0xff) as byte,
        b: (((x.b as i32 * alpha + y.b as i32 * inv_alpha) >> 8) & 0xff) as byte,
        a: y.a,
    }
}

// External C functions
extern "C" {
    fn R_LoadImage(
        name: *const c_char,
        pic: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
        format: *mut c_int,
    );

    #[cfg(target_os = "windows")]
    fn R_LoadImage_Xbox(
        name: *const c_char,
        pic: *mut *mut byte,
        width: *mut c_int,
        height: *mut c_int,
        mipcount: *mut c_int,
        format: *mut c_int,
    );

    fn Z_Free(ptr: *mut c_void);

    fn R_CreateAutomapImage(
        name: *const c_char,
        pic: *const byte,
        width: c_int,
        height: c_int,
        mipmap: c_int,
        allowPicmip: c_int,
        allowTC: c_int,
        glWrapClampMode: c_int,
    );

    pub fn RotatePointAroundVector(dst: *mut f32, dir: *const f32, point: *const f32, degrees: f32);
}

// simple function for getting a proper color for a side
#[inline]
fn SideColor(side: c_int) -> CPixel32 {
    let mut col = CPixel32::new(255, 255, 255, 255);
    match side {
        SIDE_BLUE => {
            col = CPixel32::new(0, 0, 192, 255);
        }
        SIDE_RED => {
            col = CPixel32::new(192, 0, 0, 255);
        }
        _ => {}
    }
    col
}

// CTerrainMap class
#[repr(C)]
pub struct CTerrainMap {
    pub mImage: [[[byte; 4]; TM_WIDTH]; TM_HEIGHT],      // image to output
    pub mBufImage: [[[byte; 4]; TM_WIDTH]; TM_HEIGHT],   // src data for image, color and bump

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
    pub fn new(landscape: *mut CCMLandScape) -> Self {
        let mut tm = CTerrainMap {
            mImage: [[[0u8; 4]; TM_WIDTH]; TM_HEIGHT],
            mBufImage: [[[0u8; 4]; TM_WIDTH]; TM_HEIGHT],
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
        };

        tm.ApplyBackground();
        tm.ApplyHeightmap();

        let draw = CDraw32::new();
        draw.SetBuffer(tm.mImage.as_mut_ptr() as *mut CPixel32);
        draw.SetBufferSize(TM_WIDTH as c_int, TM_HEIGHT as c_int, TM_WIDTH as c_int);

        // create version with paths and water shown
        for y in 0..TM_HEIGHT {
            for x in 0..TM_WIDTH {
                let cp = (tm.mBufImage[y][x][0], tm.mBufImage[y][x][1], tm.mBufImage[y][x][2], tm.mBufImage[y][x][3]);
                let mut cp_pixel = CPixel32::new(cp.0, cp.1, cp.2, 255);
                let land = CLAMP((((255 - cp.3 as i32) * 2) / 3), 0, 255) as byte;
                let water = if !landscape.is_null() {
                    CLAMP((255 - cp.3 as i32) * 4, 0, 255) as byte  // Note: landscape->GetBaseWaterHeight() called, but stubbed
                } else {
                    0
                };

                if x > TM_BORDER && x < (TM_WIDTH - TM_BORDER) &&
                   y > TM_BORDER && y < (TM_WIDTH - TM_BORDER) {
                    cp_pixel = ALPHA_PIX(CPixel32::new(0, 0, 0, 255), cp_pixel, land as i32, 256 - land as i32);
                    if water > 0 {
                        cp_pixel = ALPHA_PIX(CPixel32::new(0, 0, 255, 255), cp_pixel, water as i32, 256 - water as i32);
                    }
                }

                draw.PutPix(x as c_int, y as c_int, cp_pixel);
            }
        }

        // Load icons for symbols on map
        let mut format: c_int = 0;

        #[cfg(target_os = "windows")]
        {
            let mut mipcount: c_int = 0;
            unsafe {
                R_LoadImage_Xbox(
                    b"gfx/menus/rmg/start\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymStart),
                    addr_of_mut!(tm.mSymStartWidth),
                    addr_of_mut!(tm.mSymStartHeight),
                    addr_of_mut!(mipcount),
                    addr_of_mut!(format),
                );
                R_LoadImage_Xbox(
                    b"gfx/menus/rmg/end\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymEnd),
                    addr_of_mut!(tm.mSymEndWidth),
                    addr_of_mut!(tm.mSymEndHeight),
                    addr_of_mut!(mipcount),
                    addr_of_mut!(format),
                );
                R_LoadImage_Xbox(
                    b"gfx/menus/rmg/objective\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymObjective),
                    addr_of_mut!(tm.mSymObjectiveWidth),
                    addr_of_mut!(tm.mSymObjectiveHeight),
                    addr_of_mut!(mipcount),
                    addr_of_mut!(format),
                );
                R_LoadImage_Xbox(
                    b"gfx/menus/rmg/building\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymBld),
                    addr_of_mut!(tm.mSymBldWidth),
                    addr_of_mut!(tm.mSymBldHeight),
                    addr_of_mut!(mipcount),
                    addr_of_mut!(format),
                );
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            unsafe {
                R_LoadImage(
                    b"gfx/menus/rmg/start\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymStart),
                    addr_of_mut!(tm.mSymStartWidth),
                    addr_of_mut!(tm.mSymStartHeight),
                    addr_of_mut!(format),
                );
                R_LoadImage(
                    b"gfx/menus/rmg/end\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymEnd),
                    addr_of_mut!(tm.mSymEndWidth),
                    addr_of_mut!(tm.mSymEndHeight),
                    addr_of_mut!(format),
                );
                R_LoadImage(
                    b"gfx/menus/rmg/objective\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymObjective),
                    addr_of_mut!(tm.mSymObjectiveWidth),
                    addr_of_mut!(tm.mSymObjectiveHeight),
                    addr_of_mut!(format),
                );
                R_LoadImage(
                    b"gfx/menus/rmg/building\0".as_ptr() as *const c_char,
                    addr_of_mut!(tm.mSymBld),
                    addr_of_mut!(tm.mSymBldWidth),
                    addr_of_mut!(tm.mSymBldHeight),
                    addr_of_mut!(format),
                );
            }
        }

        tm
    }

    pub fn drop(&mut self) {
        if !self.mSymStart.is_null() {
            unsafe {
                Z_Free(self.mSymStart as *mut c_void);
            }
            self.mSymStart = null_mut();
        }

        if !self.mSymEnd.is_null() {
            unsafe {
                Z_Free(self.mSymEnd as *mut c_void);
            }
            self.mSymEnd = null_mut();
        }

        if !self.mSymBld.is_null() {
            unsafe {
                Z_Free(self.mSymBld as *mut c_void);
            }
            self.mSymBld = null_mut();
        }

        if !self.mSymObjective.is_null() {
            unsafe {
                Z_Free(self.mSymObjective as *mut c_void);
            }
            self.mSymObjective = null_mut();
        }

        CDraw32::CleanUp();
    }

    fn ApplyBackground(&mut self) {
        let mut backgroundImage: *mut byte = null_mut();
        let mut backgroundWidth: c_int = 0;
        let mut backgroundHeight: c_int = 0;
        let mut backgroundDepth: c_int;
        let mut format: c_int = 0;

        // memset(mImage, 255, sizeof(mBufImage));
        for y in 0..TM_HEIGHT {
            for x in 0..TM_WIDTH {
                self.mImage[y][x][0] = 255;
                self.mImage[y][x][1] = 255;
                self.mImage[y][x][2] = 255;
                self.mImage[y][x][3] = 255;
            }
        }

        backgroundDepth = 4;

        #[cfg(target_os = "windows")]
        {
            let mut mipcount: c_int = 0;
            unsafe {
                R_LoadImage_Xbox(
                    b"gfx\\menus\\rmg\\01_bg\0".as_ptr() as *const c_char,
                    addr_of_mut!(backgroundImage),
                    addr_of_mut!(backgroundWidth),
                    addr_of_mut!(backgroundHeight),
                    addr_of_mut!(mipcount),
                    addr_of_mut!(format),
                );
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            unsafe {
                R_LoadImage(
                    b"gfx\\menus\\rmg\\01_bg\0".as_ptr() as *const c_char,
                    addr_of_mut!(backgroundImage),
                    addr_of_mut!(backgroundWidth),
                    addr_of_mut!(backgroundHeight),
                    addr_of_mut!(format),
                );
            }
        }

        if !backgroundImage.is_null() {
            let xInc: f32 = (backgroundWidth as f32) / (TM_WIDTH as f32);
            let yInc: f32 = (backgroundHeight as f32) / (TM_HEIGHT as f32);

            let mut yRel: f32 = 0.0;
            for y in 0..TM_HEIGHT {
                let mut xRel: f32 = 0.0;
                for x in 0..TM_WIDTH {
                    let pos: usize = (((yRel as usize) * (backgroundWidth as usize)) + (xRel as usize)) * 4;
                    if pos + 2 < (backgroundWidth as usize * backgroundHeight as usize * 4) {
                        unsafe {
                            self.mImage[y][x][0] = *backgroundImage.add(pos);
                            self.mImage[y][x][1] = *backgroundImage.add(pos + 1);
                            self.mImage[y][x][2] = *backgroundImage.add(pos + 2);
                            self.mImage[y][x][3] = 255;
                        }
                    }
                    xRel += xInc;
                }
                yRel += yInc;
            }
            unsafe {
                Z_Free(backgroundImage as *mut c_void);
            }
        }
    }

    fn ApplyHeightmap(&mut self) {
        if self.mLandscape.is_null() {
            return;
        }

        // Stub: Can't call mLandscape methods without full implementation
        // Original code calls:
        // mLandscape->GetHeightMap()
        // mLandscape->GetRealWidth()
        // mLandscape->GetRealHeight()
        // mLandscape->GetBaseWaterHeight()
    }

    // Convert position in game coords to automap coords
    pub fn ConvertPos(&self, x: *mut c_int, y: *mut c_int) {
        if self.mLandscape.is_null() || x.is_null() || y.is_null() {
            return;
        }

        unsafe {
            // Original code accesses mLandscape->GetMins() and mLandscape->GetSize()
            // Stub out for now since CCMLandScape is opaque
            // *x = ((*x - mLandscape->GetMins()[0]) / mLandscape->GetSize()[0]) * TM_REAL_WIDTH;
            // *y = ((*y - mLandscape->GetMins()[1]) / mLandscape->GetSize()[1]) * TM_REAL_HEIGHT;

            // x is flipped!
            *x = (TM_REAL_WIDTH as c_int) - *x - 1;

            // border
            *x += TM_BORDER as c_int;
            *y += TM_BORDER as c_int;
        }
    }

    pub fn AddStart(&self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));

        let draw = CDraw32::new();
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

    pub fn AddEnd(&self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));

        let draw = CDraw32::new();
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

    pub fn AddObjective(&self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));

        let draw = CDraw32::new();
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

    pub fn AddBuilding(&self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));

        let draw = CDraw32::new();
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

    pub fn AddNPC(&self, mut x: c_int, mut y: c_int, friendly: bool) {
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));

        let draw = CDraw32::new();
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

    pub fn AddNode(&self, mut x: c_int, mut y: c_int) {
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));

        let draw = CDraw32::new();
        draw.DrawCircle(
            x,
            y,
            20,
            CPixel32::new(255, 255, 255, 255),
            CPixel32::new(0, 0, 0, 0),
        );
    }

    pub fn AddWallRect(&self, mut x: c_int, mut y: c_int, side: c_int) {
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));

        let draw = CDraw32::new();
        match side {
            SIDE_BLUE => {
                draw.DrawBox(
                    x - 1,
                    y - 1,
                    3,
                    3,
                    CPixel32::new(0, 0, 192, 128),
                );
            }
            SIDE_RED => {
                draw.DrawBox(
                    x - 1,
                    y - 1,
                    3,
                    3,
                    CPixel32::new(192, 0, 0, 128),
                );
            }
            _ => {
                draw.DrawBox(
                    x - 1,
                    y - 1,
                    3,
                    3,
                    CPixel32::new(192, 192, 192, 128),
                );
            }
        }
    }

    pub fn AddPlayer(&self, origin: &vec3_t, angles: &vec3_t) {
        // draw player start on automap
        let draw = CDraw32::new();

        let mut up: vec3_t = [0.0, 0.0, 1.0];
        let pt: [vec3_t; 4] = [
            [0.0, 0.0, 0.0],
            [-5.0, -5.0, 0.0],
            [10.0, 0.0, 0.0],
            [-5.0, 5.0, 0.0],
        ];

        let facing = angles[1];

        let mut x: c_int = origin[0] as c_int;
        let mut y: c_int = origin[1] as c_int;
        self.ConvertPos(addr_of_mut!(x), addr_of_mut!(y));
        x += 1;
        y += 1;

        let mut poly: [POINT; 4] = [
            POINT { x: 0, y: 0 },
            POINT { x: 0, y: 0 },
            POINT { x: 0, y: 0 },
            POINT { x: 0, y: 0 },
        ];

        for i in 0..4 {
            let mut p: vec3_t = [0.0, 0.0, 0.0];
            unsafe {
                RotatePointAroundVector(
                    p.as_mut_ptr(),
                    up.as_ptr(),
                    pt[i].as_ptr(),
                    facing,
                );
            }
            poly[i].x = (-p[0] + x as f32) as c_long;
            poly[i].y = (p[1] + y as f32) as c_long;
        }

        // draw arrowhead shadow
        draw.DrawPolygon(
            4,
            poly.as_ptr(),
            CPixel32::new(0, 0, 0, 128),
            CPixel32::new(0, 0, 0, 128),
        );

        // draw arrowhead
        for i in 0..4 {
            poly[i].x -= 1;
            poly[i].y -= 1;
        }
        draw.DrawPolygon(
            4,
            poly.as_ptr(),
            CPixel32::new(255, 255, 255, 255),
            CPixel32::new(255, 255, 255, 255),
        );
    }

    pub fn Upload(&mut self, player_origin: *const vec3_t, player_angles: *const vec3_t) {
        let draw = CDraw32::new();

        // copy completed map to mBufImage
        draw.SetBuffer(self.mBufImage.as_mut_ptr() as *mut CPixel32);
        draw.SetBufferSize(TM_WIDTH as c_int, TM_HEIGHT as c_int, TM_WIDTH as c_int);

        draw.BlitColor(
            0,
            0,
            TM_WIDTH as c_int,
            TM_HEIGHT as c_int,
            self.mImage.as_mut_ptr() as *mut CPixel32,
            0,
            0,
            TM_WIDTH as c_int,
            CPixel32::new(255, 255, 255, 255),
        );

        // now draw player's location on map
        if !player_origin.is_null() && !player_angles.is_null() {
            unsafe {
                self.AddPlayer(&*player_origin, &*player_angles);
            }
        }

        draw.SetAlphaBuffer(255);

        unsafe {
            R_CreateAutomapImage(
                b"*automap\0".as_ptr() as *const c_char,
                draw.buffer() as *const byte,
                TM_WIDTH as c_int,
                TM_HEIGHT as c_int,
                0, // qfalse
                0, // qfalse
                1, // qtrue
                0, // qfalse
            );
        }

        draw.SetBuffer(self.mImage.as_mut_ptr() as *mut CPixel32);
    }

    pub fn SaveImageToDisk(
        &self,
        _terrainName: *const c_char,
        _missionName: *const c_char,
        _seed: *const c_char,
    ) {
        // TODO: PNG_Save implementation
        // ri.COM_SavePNG(va("save/%s_%s_%s.png", terrainName, missionName, seed),
        //		(unsigned char *)mImage, TM_WIDTH, TM_HEIGHT, 4);
        // rww - Use JPG here? This function seems to be only for debugging anyway.
        // PNG_Save(va("save/%s_%s_%s.png", terrainName, missionName, seed),
        //		(unsigned char *)mImage, TM_WIDTH, TM_HEIGHT, 4);
    }
}

// Global TerrainMap pointer
static mut TerrainMap: *mut CTerrainMap = null_mut();

// C wrapper functions

pub extern "C" fn CM_TM_Create(landscape: *mut CCMLandScape) {
    unsafe {
        if !TerrainMap.is_null() {
            CM_TM_Free();
        }

        let tm = Box::new(CTerrainMap::new(landscape));
        TerrainMap = Box::into_raw(tm);
    }
}

pub extern "C" fn CM_TM_Free() {
    unsafe {
        if !TerrainMap.is_null() {
            let mut tm = Box::from_raw(TerrainMap);
            tm.drop();
            TerrainMap = null_mut();
        }
    }
}

pub extern "C" fn CM_TM_AddStart(x: c_int, y: c_int, side: c_int) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).AddStart(x, y, side);
        }
    }
}

pub extern "C" fn CM_TM_AddEnd(x: c_int, y: c_int, side: c_int) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).AddEnd(x, y, side);
        }
    }
}

pub extern "C" fn CM_TM_AddObjective(x: c_int, y: c_int, side: c_int) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).AddObjective(x, y, side);
        }
    }
}

pub extern "C" fn CM_TM_AddNPC(x: c_int, y: c_int, friendly: bool) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).AddNPC(x, y, friendly);
        }
    }
}

pub extern "C" fn CM_TM_AddNode(x: c_int, y: c_int) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).AddNode(x, y);
        }
    }
}

pub extern "C" fn CM_TM_AddBuilding(x: c_int, y: c_int, side: c_int) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).AddBuilding(x, y, side);
        }
    }
}

pub extern "C" fn CM_TM_AddWallRect(x: c_int, y: c_int, side: c_int) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).AddWallRect(x, y, side);
        }
    }
}

pub extern "C" fn CM_TM_Upload(player_origin: *const vec3_t, player_angles: *const vec3_t) {
    unsafe {
        if !TerrainMap.is_null() {
            (*TerrainMap).Upload(player_origin, player_angles);
        }
    }
}

pub extern "C" fn CM_TM_SaveImageToDisk(
    terrainName: *const c_char,
    missionName: *const c_char,
    seed: *const c_char,
) {
    unsafe {
        if !TerrainMap.is_null() {
            // write out automap
            (*TerrainMap).SaveImageToDisk(terrainName, missionName, seed);
        }
    }
}

pub extern "C" fn CM_TM_ConvertPosition(
    x: *mut c_int,
    y: *mut c_int,
    Width: c_int,
    Height: c_int,
) {
    unsafe {
        if !TerrainMap.is_null() && !x.is_null() && !y.is_null() {
            (*TerrainMap).ConvertPos(x, y);
            *x = *x * Width / TM_WIDTH as c_int;
            *y = *y * Height / TM_HEIGHT as c_int;
        }
    }
}
