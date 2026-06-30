#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_variables, unused_mut)]

use crate::code::server::exe_headers_h::*;
use crate::code::qcommon::cm_local_h::*;
use crate::code::qcommon::cm_patch_h::*;
use crate::code::qcommon::cm_landscape_h::*;
use crate::code::game::genericparser2_h::*;
// #include "image.h"
// #include "../qcommon/q_imath.h"
use crate::code::qcommon::cm_terrainmap_h::*;
use crate::code::qcommon::cm_draw_h::*;
use crate::code::png::png_h::*;

static mut TerrainMap: *mut CTerrainMap = core::ptr::null_mut();

extern "C" {
    fn R_CreateAutomapImage(
        name: *const core::ffi::c_char,
        pic: *const byte,
        width: core::ffi::c_int,
        height: core::ffi::c_int,
        mipmap: qboolean,
        allowPicmip: qboolean,
        allowTC: qboolean,
        glWrapClampMode: core::ffi::c_int,
    );
}

// simple function for getting a proper color for a side
#[inline]
unsafe fn SideColor(side: core::ffi::c_int) -> CPixel32 {
    let mut col = CPixel32::new(255, 255, 255);
    match side {
        SIDE_BLUE => {
            col = CPixel32::new(0, 0, 192);
        }
        SIDE_RED => {
            col = CPixel32::new(192, 0, 0);
        }
        _ => {}
    }
    col
}

impl CTerrainMap {
    // Constructor body: CTerrainMap::CTerrainMap(CCMLandScape *landscape) : mLandscape(landscape)
    pub unsafe fn ctor(&mut self, landscape: *mut CCMLandScape) {
        self.mLandscape = landscape;

        self.ApplyBackground();
        self.ApplyHeightmap();

        let mut draw = CDraw32::new();
        draw.SetBuffer(self.mImage.as_mut_ptr() as *mut CPixel32);
        draw.SetBufferSize(TM_WIDTH, TM_HEIGHT, TM_WIDTH);

        // create version with paths and water shown
        let mut x: core::ffi::c_int;
        let mut y: core::ffi::c_int;
        let mut water: core::ffi::c_int;
        let mut land: core::ffi::c_int;

        y = 0;
        while y < TM_HEIGHT {
            x = 0;
            while x < TM_WIDTH {
                let mut cp: CPixel32 = *(self.mBufImage.as_ptr() as *const CPixel32)
                    .add(PIXPOS(x, y, TM_WIDTH) as usize);
                land = CLAMP(((255 - cp.a as core::ffi::c_int) * 2) / 3, 0, 255);
                water = CLAMP(
                    ((*landscape).GetBaseWaterHeight() - cp.a as core::ffi::c_int) * 4,
                    0,
                    255,
                );
                cp.a = 255;

                if x > TM_BORDER
                    && x < (TM_WIDTH - TM_BORDER)
                    && y > TM_BORDER
                    && y < (TM_WIDTH - TM_BORDER)
                {
                    cp = ALPHA_PIX(CPixel32::new(0, 0, 0), cp, land, 256 - land);
                    if water > 0 {
                        cp = ALPHA_PIX(CPixel32::new(0, 0, 255), cp, water, 256 - water);
                    }
                }

                draw.PutPix(x, y, cp);
                x += 1;
            }
            y += 1;
        }

        // Load icons for symbols on map
        let mut format: GLenum = 0;

        #[cfg(feature = "xbox")]
        {
            let mut mipcount: core::ffi::c_int = 0;
            R_LoadImage(
                b"gfx/menus/rmg/start\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymStart as *mut *mut byte,
                &mut self.mSymStartWidth,
                &mut self.mSymStartHeight,
                &mut mipcount,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/end\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymEnd as *mut *mut byte,
                &mut self.mSymEndWidth,
                &mut self.mSymEndHeight,
                &mut mipcount,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/objective\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymObjective as *mut *mut byte,
                &mut self.mSymObjectiveWidth,
                &mut self.mSymObjectiveHeight,
                &mut mipcount,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/building\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymBld as *mut *mut byte,
                &mut self.mSymBldWidth,
                &mut self.mSymBldHeight,
                &mut mipcount,
                &mut format,
            );
        }
        #[cfg(not(feature = "xbox"))]
        {
            R_LoadImage(
                b"gfx/menus/rmg/start\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymStart as *mut *mut byte,
                &mut self.mSymStartWidth,
                &mut self.mSymStartHeight,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/end\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymEnd as *mut *mut byte,
                &mut self.mSymEndWidth,
                &mut self.mSymEndHeight,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/objective\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymObjective as *mut *mut byte,
                &mut self.mSymObjectiveWidth,
                &mut self.mSymObjectiveHeight,
                &mut format,
            );
            R_LoadImage(
                b"gfx/menus/rmg/building\0".as_ptr() as *const core::ffi::c_char,
                &mut self.mSymBld as *mut *mut byte,
                &mut self.mSymBldWidth,
                &mut self.mSymBldHeight,
                &mut format,
            );
        }
    }

    // Destructor body: CTerrainMap::~CTerrainMap()
    pub unsafe fn dtor(&mut self) {
        if !self.mSymStart.is_null() {
            Z_Free(self.mSymStart as *mut core::ffi::c_void);
            self.mSymStart = core::ptr::null_mut();
        }

        if !self.mSymEnd.is_null() {
            Z_Free(self.mSymEnd as *mut core::ffi::c_void);
            self.mSymEnd = core::ptr::null_mut();
        }

        if !self.mSymBld.is_null() {
            Z_Free(self.mSymBld as *mut core::ffi::c_void);
            self.mSymBld = core::ptr::null_mut();
        }

        if !self.mSymObjective.is_null() {
            Z_Free(self.mSymObjective as *mut core::ffi::c_void);
            self.mSymObjective = core::ptr::null_mut();
        }

        CDraw32::CleanUp();
    }

    pub unsafe fn ApplyBackground(&mut self) {
        let mut x: core::ffi::c_int;
        let mut y: core::ffi::c_int;
        let mut outPos: *mut byte;
        let mut xRel: f32;
        let mut yRel: f32;
        let xInc: f32;
        let yInc: f32;
        let mut backgroundImage: *mut byte = core::ptr::null_mut();
        let mut backgroundWidth: core::ffi::c_int = 0;
        let mut backgroundHeight: core::ffi::c_int = 0;
        let backgroundDepth: core::ffi::c_int;
        let mut pos: core::ffi::c_int;
        let mut format: GLenum = 0;

        libc::memset(
            self.mImage.as_mut_ptr() as *mut core::ffi::c_void,
            255,
            core::mem::size_of_val(&self.mBufImage),
        );
        // R_LoadImage("textures\\kamchatka\\ice", &backgroundImage, &backgroundWidth, &backgroundHeight, &format);0
        backgroundDepth = 4;

        #[cfg(feature = "xbox")]
        {
            let mut mipcount: core::ffi::c_int = 0;
            R_LoadImage(
                b"gfx\\menus\\rmg\\01_bg\0".as_ptr() as *const core::ffi::c_char,
                &mut backgroundImage,
                &mut backgroundWidth,
                &mut backgroundHeight,
                &mut mipcount,
                &mut format,
            );
        }
        #[cfg(not(feature = "xbox"))]
        {
            R_LoadImage(
                b"gfx\\menus\\rmg\\01_bg\0".as_ptr() as *const core::ffi::c_char,
                &mut backgroundImage,
                &mut backgroundWidth,
                &mut backgroundHeight,
                &mut format,
            );
        }
        if !backgroundImage.is_null() {
            outPos = self.mBufImage.as_mut_ptr() as *mut byte;
            xInc = backgroundWidth as f32 / TM_WIDTH as f32;
            yInc = backgroundHeight as f32 / TM_HEIGHT as f32;

            yRel = 0.0;
            y = 0;
            while y < TM_HEIGHT {
                xRel = 0.0;
                x = 0;
                while x < TM_WIDTH {
                    pos = ((yRel as core::ffi::c_int * backgroundWidth)
                        + xRel as core::ffi::c_int)
                        * 4;
                    *outPos = *backgroundImage.add(pos as usize);
                    pos += 1;
                    outPos = outPos.add(1);
                    *outPos = *backgroundImage.add(pos as usize);
                    pos += 1;
                    outPos = outPos.add(1);
                    *outPos = *backgroundImage.add(pos as usize);
                    outPos = outPos.add(2);
                    xRel += xInc;
                    x += 1;
                }
                yRel += yInc;
                y += 1;
            }
            Z_Free(backgroundImage as *mut core::ffi::c_void);
        }
    }

    pub unsafe fn ApplyHeightmap(&mut self) {
        let mut x: core::ffi::c_int;
        let mut y: core::ffi::c_int;
        let inPos: *mut byte = (*self.mLandscape).GetHeightMap();
        let width: core::ffi::c_int = (*self.mLandscape).GetRealWidth();
        let height: core::ffi::c_int = (*self.mLandscape).GetRealHeight();
        let mut outPos: *mut byte;
        let mut tempColor: core::ffi::c_uint;
        let mut xRel: f32;
        let mut yRel: f32;
        let xInc: f32;
        let yInc: f32;
        let mut count: core::ffi::c_int;

        outPos = self.mBufImage.as_mut_ptr() as *mut byte;
        outPos = outPos.add(((TM_BORDER * TM_WIDTH + TM_BORDER) as usize) * 4);
        xInc = width as f32 / TM_REAL_WIDTH as f32;
        yInc = height as f32 / TM_REAL_HEIGHT as f32;

        // add in height map as alpha
        yRel = 0.0;
        y = 0;
        while y < TM_REAL_HEIGHT {
            // x is flipped!
            xRel = width as f32;
            x = 0;
            while x < TM_REAL_WIDTH {
                count = 1;
                tempColor = *inPos.add(
                    (yRel as usize * width as usize) + xRel as usize,
                ) as core::ffi::c_uint;
                if yRel >= 1.0 {
                    tempColor += *inPos.add(
                        ((yRel - 0.5) as usize * width as usize) + xRel as usize,
                    ) as core::ffi::c_uint;
                    count += 1;
                }
                if yRel <= height as f32 - 2.0 {
                    tempColor += *inPos.add(
                        ((yRel + 0.5) as usize * width as usize) + xRel as usize,
                    ) as core::ffi::c_uint;
                    count += 1;
                }
                if xRel >= 1.0 {
                    tempColor += *inPos.add(
                        (yRel as usize * width as usize) + (xRel - 0.5) as usize,
                    ) as core::ffi::c_uint;
                    count += 1;
                }
                if xRel <= width as f32 - 2.0 {
                    tempColor += *inPos.add(
                        (yRel as usize * width as usize) + (xRel + 0.5) as usize,
                    ) as core::ffi::c_uint;
                    count += 1;
                }
                tempColor /= count as core::ffi::c_uint;

                *outPos.add(3) = tempColor as byte;
                outPos = outPos.add(4);

                // x is flipped!
                xRel -= xInc;
                x += 1;
            }
            outPos = outPos.add(TM_BORDER as usize * 4 * 2);

            yRel += yInc;
            y += 1;
        }
    }

    // Convert position in game coords to automap coords
    pub unsafe fn ConvertPos(&mut self, x: &mut core::ffi::c_int, y: &mut core::ffi::c_int) {
        *x = ((*x - (*self.mLandscape).GetMins()[0] as core::ffi::c_int)
            / (*self.mLandscape).GetSize()[0] as core::ffi::c_int)
            * TM_REAL_WIDTH as core::ffi::c_int;
        *y = ((*y - (*self.mLandscape).GetMins()[1] as core::ffi::c_int)
            / (*self.mLandscape).GetSize()[1] as core::ffi::c_int)
            * TM_REAL_HEIGHT as core::ffi::c_int;

        // x is flipped!
        *x = TM_REAL_WIDTH as core::ffi::c_int - *x - 1;

        // border
        *x += TM_BORDER as core::ffi::c_int;
        *y += TM_BORDER as core::ffi::c_int;
    }

    pub unsafe fn AddStart(&mut self, mut x: core::ffi::c_int, mut y: core::ffi::c_int, side: core::ffi::c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32::new();
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

    pub unsafe fn AddEnd(&mut self, mut x: core::ffi::c_int, mut y: core::ffi::c_int, side: core::ffi::c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32::new();
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

    pub unsafe fn AddObjective(&mut self, mut x: core::ffi::c_int, mut y: core::ffi::c_int, side: core::ffi::c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32::new();
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

    pub unsafe fn AddBuilding(&mut self, mut x: core::ffi::c_int, mut y: core::ffi::c_int, side: core::ffi::c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32::new();
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

    pub unsafe fn AddNPC(&mut self, mut x: core::ffi::c_int, mut y: core::ffi::c_int, friendly: bool) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32::new();
        if friendly {
            draw.DrawCircle(x, y, 3, CPixel32::new(0, 192, 0), CPixel32::new_rgba(0, 0, 0, 0));
        } else {
            draw.DrawCircle(x, y, 3, CPixel32::new(192, 0, 0), CPixel32::new_rgba(0, 0, 0, 0));
        }
    }

    pub unsafe fn AddNode(&mut self, mut x: core::ffi::c_int, mut y: core::ffi::c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32::new();
        draw.DrawCircle(x, y, 20, CPixel32::new(255, 255, 255), CPixel32::new_rgba(0, 0, 0, 0));
    }

    pub unsafe fn AddWallRect(&mut self, mut x: core::ffi::c_int, mut y: core::ffi::c_int, side: core::ffi::c_int) {
        self.ConvertPos(&mut x, &mut y);

        let mut draw = CDraw32::new();
        match side {
            SIDE_BLUE => {
                draw.DrawBox(x - 1, y - 1, 3, 3, CPixel32::new_rgba(0, 0, 192, 128));
            }
            SIDE_RED => {
                draw.DrawBox(x - 1, y - 1, 3, 3, CPixel32::new_rgba(192, 0, 0, 128));
            }
            _ => {
                draw.DrawBox(x - 1, y - 1, 3, 3, CPixel32::new_rgba(192, 192, 192, 128));
            }
        }
    }

    pub unsafe fn AddPlayer(&mut self, origin: *const f32, angles: *const f32) {
        // draw player start on automap
        let mut draw = CDraw32::new();

        let mut up: vec3_t = [0.0, 0.0, 0.0];
        let mut pt: [vec3_t; 4] = [
            [0.0, 0.0, 0.0],
            [-5.0, -5.0, 0.0],
            [10.0, 0.0, 0.0],
            [-5.0, 5.0, 0.0],
        ];
        let mut p: vec3_t = [0.0, 0.0, 0.0];
        let mut x: core::ffi::c_int;
        let mut y: core::ffi::c_int;
        let mut i: core::ffi::c_int;
        let facing: f32;
        let mut poly: [POINT; 4] = [POINT { x: 0, y: 0 }; 4];

        facing = *angles.add(1);

        up[0] = 0.0;
        up[1] = 0.0;
        up[2] = 1.0;

        x = *origin.add(0) as core::ffi::c_int;
        y = *origin.add(1) as core::ffi::c_int;
        self.ConvertPos(&mut x, &mut y);
        x += 1;
        y += 1;

        i = 0;
        while i < 4 {
            RotatePointAroundVector(
                p.as_mut_ptr(),
                up.as_ptr(),
                pt[i as usize].as_ptr(),
                facing,
            );
            poly[i as usize].x = (-p[0] + x as f32) as core::ffi::c_int;
            poly[i as usize].y = (p[1] + y as f32) as core::ffi::c_int;
            i += 1;
        }

        // draw arrowhead shadow
        draw.DrawPolygon(
            4,
            poly.as_ptr(),
            CPixel32::new_rgba(0, 0, 0, 128),
            CPixel32::new_rgba(0, 0, 0, 128),
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
            poly.as_ptr(),
            CPixel32::new(255, 255, 255),
            CPixel32::new(255, 255, 255),
        );
    }

    pub unsafe fn Upload(&mut self, player_origin: *const f32, player_angles: *const f32) {
        let mut draw = CDraw32::new();

        // copy completed map to mBufImage
        draw.SetBuffer(self.mBufImage.as_mut_ptr() as *mut CPixel32);
        draw.SetBufferSize(TM_WIDTH, TM_HEIGHT, TM_WIDTH);

        draw.Blit(
            0,
            0,
            TM_WIDTH,
            TM_HEIGHT,
            self.mImage.as_mut_ptr() as *mut CPixel32,
            0,
            0,
            TM_WIDTH,
        );

        // now draw player's location on map
        if !player_origin.is_null() {
            self.AddPlayer(player_origin, player_angles);
        }

        draw.SetAlphaBuffer(255);

        R_CreateAutomapImage(
            b"*automap\0".as_ptr() as *const core::ffi::c_char,
            draw.buffer as *const byte,
            TM_WIDTH,
            TM_HEIGHT,
            qfalse,
            qfalse,
            qtrue,
            qfalse,
        );

        draw.SetBuffer(self.mImage.as_mut_ptr() as *mut CPixel32);
    }

    pub unsafe fn SaveImageToDisk(
        &mut self,
        _terrainName: *const core::ffi::c_char,
        _missionName: *const core::ffi::c_char,
        _seed: *const core::ffi::c_char,
    ) {
        //ri.COM_SavePNG(va("save/%s_%s_%s.png", terrainName, missionName, seed),
        //		(unsigned char *)mImage, TM_WIDTH, TM_HEIGHT, 4);
        //rww - Use JPG here? This function seems to be only for debugging anyway.
        // PNG_Save(va("save/%s_%s_%s.png", terrainName, missionName, seed),
        //		(unsigned char *)mImage, TM_WIDTH, TM_HEIGHT, 4);
    }
}

pub unsafe fn CM_TM_Create(landscape: *mut CCMLandScape) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        CM_TM_Free();
    }

    let tm: *mut CTerrainMap = Box::into_raw(Box::<CTerrainMap>::new_zeroed().assume_init());
    (*tm).ctor(landscape);
    *core::ptr::addr_of_mut!(TerrainMap) = tm;
}

pub unsafe fn CM_TM_Free() {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).dtor();
        drop(Box::from_raw(*core::ptr::addr_of!(TerrainMap)));
        *core::ptr::addr_of_mut!(TerrainMap) = core::ptr::null_mut();
    }
}

pub unsafe fn CM_TM_AddStart(x: core::ffi::c_int, y: core::ffi::c_int, side: core::ffi::c_int) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).AddStart(x, y, side);
    }
}

pub unsafe fn CM_TM_AddEnd(x: core::ffi::c_int, y: core::ffi::c_int, side: core::ffi::c_int) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).AddEnd(x, y, side);
    }
}

pub unsafe fn CM_TM_AddObjective(x: core::ffi::c_int, y: core::ffi::c_int, side: core::ffi::c_int) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).AddObjective(x, y, side);
    }
}

pub unsafe fn CM_TM_AddNPC(x: core::ffi::c_int, y: core::ffi::c_int, friendly: bool) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).AddNPC(x, y, friendly);
    }
}

pub unsafe fn CM_TM_AddNode(x: core::ffi::c_int, y: core::ffi::c_int) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).AddNode(x, y);
    }
}

pub unsafe fn CM_TM_AddBuilding(x: core::ffi::c_int, y: core::ffi::c_int, side: core::ffi::c_int) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).AddBuilding(x, y, side);
    }
}

pub unsafe fn CM_TM_AddWallRect(x: core::ffi::c_int, y: core::ffi::c_int, side: core::ffi::c_int) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).AddWallRect(x, y, side);
    }
}

pub unsafe fn CM_TM_Upload(player_origin: *const f32, player_angles: *const f32) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).Upload(player_origin, player_angles);
    }
}

pub unsafe fn CM_TM_SaveImageToDisk(
    terrainName: *const core::ffi::c_char,
    missionName: *const core::ffi::c_char,
    seed: *const core::ffi::c_char,
) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        // write out automap
        (*(*core::ptr::addr_of!(TerrainMap))).SaveImageToDisk(terrainName, missionName, seed);
    }
}

pub unsafe fn CM_TM_ConvertPosition(
    x: &mut core::ffi::c_int,
    y: &mut core::ffi::c_int,
    Width: core::ffi::c_int,
    Height: core::ffi::c_int,
) {
    if !(*core::ptr::addr_of!(TerrainMap)).is_null() {
        (*(*core::ptr::addr_of!(TerrainMap))).ConvertPos(x, y);
        *x = *x * Width / TM_WIDTH as core::ffi::c_int;
        *y = *y * Height / TM_HEIGHT as core::ffi::c_int;
    }
}
