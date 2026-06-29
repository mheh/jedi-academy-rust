/*
** QGL.H
*/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::*;

/* Type definitions */
pub type GLenum = c_uint;
pub type GLboolean = c_uchar;
pub type GLbitfield = c_uint;
pub type GLbyte = c_schar;
pub type GLshort = c_short;
pub type GLint = c_int;
pub type GLsizei = c_int;
pub type GLubyte = c_uchar;
pub type GLushort = c_ushort;
pub type GLuint = c_uint;
pub type GLfloat = c_float;
pub type GLclampf = c_float;
pub type GLdouble = c_double;
pub type GLclampd = c_double;
pub type GLvoid = c_void;

/* APIENTRY is empty in this port */

pub const GL_ACCUM: u32 = 0x0100;
pub const GL_LOAD: u32 = 0x0101;
pub const GL_RETURN: u32 = 0x0102;
pub const GL_MULT: u32 = 0x0103;
pub const GL_ADD: u32 = 0x0104;

/* AlphaFunction */
pub const GL_NEVER: u32 = 0x0200;
pub const GL_LESS: u32 = 0x0201;
pub const GL_EQUAL: u32 = 0x0202;
pub const GL_LEQUAL: u32 = 0x0203;
pub const GL_GREATER: u32 = 0x0204;
pub const GL_NOTEQUAL: u32 = 0x0205;
pub const GL_GEQUAL: u32 = 0x0206;
pub const GL_ALWAYS: u32 = 0x0207;

/* AttribMask */
pub const GL_CURRENT_BIT: u32 = 0x00000001;
pub const GL_POINT_BIT: u32 = 0x00000002;
pub const GL_LINE_BIT: u32 = 0x00000004;
pub const GL_POLYGON_BIT: u32 = 0x00000008;
pub const GL_POLYGON_STIPPLE_BIT: u32 = 0x00000010;
pub const GL_PIXEL_MODE_BIT: u32 = 0x00000020;
pub const GL_LIGHTING_BIT: u32 = 0x00000040;
pub const GL_FOG_BIT: u32 = 0x00000080;
pub const GL_DEPTH_BUFFER_BIT: u32 = 0x00000100;
pub const GL_ACCUM_BUFFER_BIT: u32 = 0x00000200;
pub const GL_STENCIL_BUFFER_BIT: u32 = 0x00000400;
pub const GL_VIEWPORT_BIT: u32 = 0x00000800;
pub const GL_TRANSFORM_BIT: u32 = 0x00001000;
pub const GL_ENABLE_BIT: u32 = 0x00002000;
pub const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
pub const GL_HINT_BIT: u32 = 0x00008000;
pub const GL_EVAL_BIT: u32 = 0x00010000;
pub const GL_LIST_BIT: u32 = 0x00020000;
pub const GL_TEXTURE_BIT: u32 = 0x00040000;
pub const GL_SCISSOR_BIT: u32 = 0x00080000;
pub const GL_ALL_ATTRIB_BITS: u32 = 0x000fffff;

/* BeginMode */
pub const GL_POINTS: u32 = 0x0000;
pub const GL_LINES: u32 = 0x0001;
pub const GL_LINE_LOOP: u32 = 0x0002;
pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_TRIANGLES: u32 = 0x0004;
pub const GL_TRIANGLE_STRIP: u32 = 0x0005;
pub const GL_TRIANGLE_FAN: u32 = 0x0006;
pub const GL_QUADS: u32 = 0x0007;
pub const GL_QUAD_STRIP: u32 = 0x0008;
pub const GL_POLYGON: u32 = 0x0009;

/* BlendingFactorDest */
pub const GL_ZERO: u32 = 0;
pub const GL_ONE: u32 = 1;
pub const GL_SRC_COLOR: u32 = 0x0300;
pub const GL_ONE_MINUS_SRC_COLOR: u32 = 0x0301;
pub const GL_SRC_ALPHA: u32 = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
pub const GL_DST_ALPHA: u32 = 0x0304;
pub const GL_ONE_MINUS_DST_ALPHA: u32 = 0x0305;

/* BlendingFactorSrc */
/*      GL_ZERO */
/*      GL_ONE */
pub const GL_DST_COLOR: u32 = 0x0306;
pub const GL_ONE_MINUS_DST_COLOR: u32 = 0x0307;
pub const GL_SRC_ALPHA_SATURATE: u32 = 0x0308;

/* Boolean */
pub const GL_TRUE: u32 = 1;
pub const GL_FALSE: u32 = 0;

/* ClipPlaneName */
pub const GL_CLIP_PLANE0: u32 = 0x3000;
pub const GL_CLIP_PLANE1: u32 = 0x3001;
pub const GL_CLIP_PLANE2: u32 = 0x3002;
pub const GL_CLIP_PLANE3: u32 = 0x3003;
pub const GL_CLIP_PLANE4: u32 = 0x3004;
pub const GL_CLIP_PLANE5: u32 = 0x3005;

/* DataType */
pub const GL_BYTE: u32 = 0x1400;
pub const GL_UNSIGNED_BYTE: u32 = 0x1401;
pub const GL_SHORT: u32 = 0x1402;
pub const GL_UNSIGNED_SHORT: u32 = 0x1403;
pub const GL_INT: u32 = 0x1404;
pub const GL_UNSIGNED_INT: u32 = 0x1405;
pub const GL_FLOAT: u32 = 0x1406;
pub const GL_2_BYTES: u32 = 0x1407;
pub const GL_3_BYTES: u32 = 0x1408;
pub const GL_4_BYTES: u32 = 0x1409;
pub const GL_DOUBLE: u32 = 0x140A;

/* DrawBufferMode */
pub const GL_NONE: u32 = 0;
pub const GL_FRONT_LEFT: u32 = 0x0400;
pub const GL_FRONT_RIGHT: u32 = 0x0401;
pub const GL_BACK_LEFT: u32 = 0x0402;
pub const GL_BACK_RIGHT: u32 = 0x0403;
pub const GL_FRONT: u32 = 0x0404;
pub const GL_BACK: u32 = 0x0405;
pub const GL_LEFT: u32 = 0x0406;
pub const GL_RIGHT: u32 = 0x0407;
pub const GL_FRONT_AND_BACK: u32 = 0x0408;
pub const GL_AUX0: u32 = 0x0409;
pub const GL_AUX1: u32 = 0x040A;
pub const GL_AUX2: u32 = 0x040B;
pub const GL_AUX3: u32 = 0x040C;

/* ErrorCode */
pub const GL_NO_ERROR: u32 = 0;
pub const GL_INVALID_ENUM: u32 = 0x0500;
pub const GL_INVALID_VALUE: u32 = 0x0501;
pub const GL_INVALID_OPERATION: u32 = 0x0502;
pub const GL_STACK_OVERFLOW: u32 = 0x0503;
pub const GL_STACK_UNDERFLOW: u32 = 0x0504;
pub const GL_OUT_OF_MEMORY: u32 = 0x0505;

/* FeedBackMode */
pub const GL_2D: u32 = 0x0600;
pub const GL_3D: u32 = 0x0601;
pub const GL_3D_COLOR: u32 = 0x0602;
pub const GL_3D_COLOR_TEXTURE: u32 = 0x0603;
pub const GL_4D_COLOR_TEXTURE: u32 = 0x0604;

/* FeedBackToken */
pub const GL_PASS_THROUGH_TOKEN: u32 = 0x0700;
pub const GL_POINT_TOKEN: u32 = 0x0701;
pub const GL_LINE_TOKEN: u32 = 0x0702;
pub const GL_POLYGON_TOKEN: u32 = 0x0703;
pub const GL_BITMAP_TOKEN: u32 = 0x0704;
pub const GL_DRAW_PIXEL_TOKEN: u32 = 0x0705;
pub const GL_COPY_PIXEL_TOKEN: u32 = 0x0706;
pub const GL_LINE_RESET_TOKEN: u32 = 0x0707;

/* FogMode */
/*      GL_LINEAR */
pub const GL_EXP: u32 = 0x0800;
pub const GL_EXP2: u32 = 0x0801;

/* FrontFaceDirection */
pub const GL_CW: u32 = 0x0900;
pub const GL_CCW: u32 = 0x0901;

/* GetMapTarget */
pub const GL_COEFF: u32 = 0x0A00;
pub const GL_ORDER: u32 = 0x0A01;
pub const GL_DOMAIN: u32 = 0x0A02;

/* GetTarget */
pub const GL_CURRENT_COLOR: u32 = 0x0B00;
pub const GL_CURRENT_INDEX: u32 = 0x0B01;
pub const GL_CURRENT_NORMAL: u32 = 0x0B02;
pub const GL_CURRENT_TEXTURE_COORDS: u32 = 0x0B03;
pub const GL_CURRENT_RASTER_COLOR: u32 = 0x0B04;
pub const GL_CURRENT_RASTER_INDEX: u32 = 0x0B05;
pub const GL_CURRENT_RASTER_TEXTURE_COORDS: u32 = 0x0B06;
pub const GL_CURRENT_RASTER_POSITION: u32 = 0x0B07;
pub const GL_CURRENT_RASTER_POSITION_VALID: u32 = 0x0B08;
pub const GL_CURRENT_RASTER_DISTANCE: u32 = 0x0B09;
pub const GL_POINT_SMOOTH: u32 = 0x0B10;
pub const GL_POINT_SIZE: u32 = 0x0B11;
pub const GL_POINT_SIZE_RANGE: u32 = 0x0B12;
pub const GL_POINT_SIZE_GRANULARITY: u32 = 0x0B13;
pub const GL_LINE_SMOOTH: u32 = 0x0B20;
pub const GL_LINE_WIDTH: u32 = 0x0B21;
pub const GL_LINE_WIDTH_RANGE: u32 = 0x0B22;
pub const GL_LINE_WIDTH_GRANULARITY: u32 = 0x0B23;
pub const GL_LINE_STIPPLE: u32 = 0x0B24;
pub const GL_LINE_STIPPLE_PATTERN: u32 = 0x0B25;
pub const GL_LINE_STIPPLE_REPEAT: u32 = 0x0B26;
pub const GL_LIST_MODE: u32 = 0x0B30;
pub const GL_MAX_LIST_NESTING: u32 = 0x0B31;
pub const GL_LIST_BASE: u32 = 0x0B32;
pub const GL_LIST_INDEX: u32 = 0x0B33;
pub const GL_POLYGON_MODE: u32 = 0x0B40;
pub const GL_POLYGON_SMOOTH: u32 = 0x0B41;
pub const GL_POLYGON_STIPPLE: u32 = 0x0B42;
pub const GL_EDGE_FLAG: u32 = 0x0B43;
pub const GL_CULL_FACE: u32 = 0x0B44;
pub const GL_CULL_FACE_MODE: u32 = 0x0B45;
pub const GL_FRONT_FACE: u32 = 0x0B46;
pub const GL_LIGHTING: u32 = 0x0B50;
pub const GL_LIGHT_MODEL_LOCAL_VIEWER: u32 = 0x0B51;
pub const GL_LIGHT_MODEL_TWO_SIDE: u32 = 0x0B52;
pub const GL_LIGHT_MODEL_AMBIENT: u32 = 0x0B53;
pub const GL_SHADE_MODEL: u32 = 0x0B54;
pub const GL_COLOR_MATERIAL_FACE: u32 = 0x0B55;
pub const GL_COLOR_MATERIAL_PARAMETER: u32 = 0x0B56;
pub const GL_COLOR_MATERIAL: u32 = 0x0B57;
pub const GL_FOG: u32 = 0x0B60;
pub const GL_FOG_INDEX: u32 = 0x0B61;
pub const GL_FOG_DENSITY: u32 = 0x0B62;
pub const GL_FOG_START: u32 = 0x0B63;
pub const GL_FOG_END: u32 = 0x0B64;
pub const GL_FOG_MODE: u32 = 0x0B65;
pub const GL_FOG_COLOR: u32 = 0x0B66;
pub const GL_DEPTH_RANGE: u32 = 0x0B70;
pub const GL_DEPTH_TEST: u32 = 0x0B71;
pub const GL_DEPTH_WRITEMASK: u32 = 0x0B72;
pub const GL_DEPTH_CLEAR_VALUE: u32 = 0x0B73;
pub const GL_DEPTH_FUNC: u32 = 0x0B74;
pub const GL_ACCUM_CLEAR_VALUE: u32 = 0x0B80;
pub const GL_STENCIL_TEST: u32 = 0x0B90;
pub const GL_STENCIL_CLEAR_VALUE: u32 = 0x0B91;
pub const GL_STENCIL_FUNC: u32 = 0x0B92;
pub const GL_STENCIL_VALUE_MASK: u32 = 0x0B93;
pub const GL_STENCIL_FAIL: u32 = 0x0B94;
pub const GL_STENCIL_PASS_DEPTH_FAIL: u32 = 0x0B95;
pub const GL_STENCIL_PASS_DEPTH_PASS: u32 = 0x0B96;
pub const GL_STENCIL_REF: u32 = 0x0B97;
pub const GL_STENCIL_WRITEMASK: u32 = 0x0B98;
pub const GL_MATRIX_MODE: u32 = 0x0BA0;
pub const GL_NORMALIZE: u32 = 0x0BA1;
pub const GL_VIEWPORT: u32 = 0x0BA2;
pub const GL_MODELVIEW_STACK_DEPTH: u32 = 0x0BA3;
pub const GL_PROJECTION_STACK_DEPTH: u32 = 0x0BA4;
pub const GL_TEXTURE_STACK_DEPTH: u32 = 0x0BA5;
pub const GL_MODELVIEW_MATRIX: u32 = 0x0BA6;
pub const GL_PROJECTION_MATRIX: u32 = 0x0BA7;
pub const GL_TEXTURE_MATRIX: u32 = 0x0BA8;
pub const GL_ATTRIB_STACK_DEPTH: u32 = 0x0BB0;
pub const GL_CLIENT_ATTRIB_STACK_DEPTH: u32 = 0x0BB1;
pub const GL_ALPHA_TEST: u32 = 0x0BC0;
pub const GL_ALPHA_TEST_FUNC: u32 = 0x0BC1;
pub const GL_ALPHA_TEST_REF: u32 = 0x0BC2;
pub const GL_DITHER: u32 = 0x0BD0;
pub const GL_BLEND_DST: u32 = 0x0BE0;
pub const GL_BLEND_SRC: u32 = 0x0BE1;
pub const GL_BLEND: u32 = 0x0BE2;
pub const GL_LOGIC_OP_MODE: u32 = 0x0BF0;
pub const GL_INDEX_LOGIC_OP: u32 = 0x0BF1;
pub const GL_COLOR_LOGIC_OP: u32 = 0x0BF2;
pub const GL_AUX_BUFFERS: u32 = 0x0C00;
pub const GL_DRAW_BUFFER: u32 = 0x0C01;
pub const GL_READ_BUFFER: u32 = 0x0C02;
pub const GL_SCISSOR_BOX: u32 = 0x0C10;
pub const GL_SCISSOR_TEST: u32 = 0x0C11;
pub const GL_INDEX_CLEAR_VALUE: u32 = 0x0C20;
pub const GL_INDEX_WRITEMASK: u32 = 0x0C21;
pub const GL_COLOR_CLEAR_VALUE: u32 = 0x0C22;
pub const GL_COLOR_WRITEMASK: u32 = 0x0C23;
pub const GL_INDEX_MODE: u32 = 0x0C30;
pub const GL_RGBA_MODE: u32 = 0x0C31;
pub const GL_DOUBLEBUFFER: u32 = 0x0C32;
pub const GL_STEREO: u32 = 0x0C33;
pub const GL_RENDER_MODE: u32 = 0x0C40;
pub const GL_PERSPECTIVE_CORRECTION_HINT: u32 = 0x0C50;
pub const GL_POINT_SMOOTH_HINT: u32 = 0x0C51;
pub const GL_LINE_SMOOTH_HINT: u32 = 0x0C52;
pub const GL_POLYGON_SMOOTH_HINT: u32 = 0x0C53;
pub const GL_FOG_HINT: u32 = 0x0C54;
pub const GL_TEXTURE_GEN_S: u32 = 0x0C60;
pub const GL_TEXTURE_GEN_T: u32 = 0x0C61;
pub const GL_TEXTURE_GEN_R: u32 = 0x0C62;
pub const GL_TEXTURE_GEN_Q: u32 = 0x0C63;
pub const GL_PIXEL_MAP_I_TO_I: u32 = 0x0C70;
pub const GL_PIXEL_MAP_S_TO_S: u32 = 0x0C71;
pub const GL_PIXEL_MAP_I_TO_R: u32 = 0x0C72;
pub const GL_PIXEL_MAP_I_TO_G: u32 = 0x0C73;
pub const GL_PIXEL_MAP_I_TO_B: u32 = 0x0C74;
pub const GL_PIXEL_MAP_I_TO_A: u32 = 0x0C75;
pub const GL_PIXEL_MAP_R_TO_R: u32 = 0x0C76;
pub const GL_PIXEL_MAP_G_TO_G: u32 = 0x0C77;
pub const GL_PIXEL_MAP_B_TO_B: u32 = 0x0C78;
pub const GL_PIXEL_MAP_A_TO_A: u32 = 0x0C79;
pub const GL_PIXEL_MAP_I_TO_I_SIZE: u32 = 0x0CB0;
pub const GL_PIXEL_MAP_S_TO_S_SIZE: u32 = 0x0CB1;
pub const GL_PIXEL_MAP_I_TO_R_SIZE: u32 = 0x0CB2;
pub const GL_PIXEL_MAP_I_TO_G_SIZE: u32 = 0x0CB3;
pub const GL_PIXEL_MAP_I_TO_B_SIZE: u32 = 0x0CB4;
pub const GL_PIXEL_MAP_I_TO_A_SIZE: u32 = 0x0CB5;
pub const GL_PIXEL_MAP_R_TO_R_SIZE: u32 = 0x0CB6;
pub const GL_PIXEL_MAP_G_TO_G_SIZE: u32 = 0x0CB7;
pub const GL_PIXEL_MAP_B_TO_B_SIZE: u32 = 0x0CB8;
pub const GL_PIXEL_MAP_A_TO_A_SIZE: u32 = 0x0CB9;
pub const GL_UNPACK_SWAP_BYTES: u32 = 0x0CF0;
pub const GL_UNPACK_LSB_FIRST: u32 = 0x0CF1;
pub const GL_UNPACK_ROW_LENGTH: u32 = 0x0CF2;
pub const GL_UNPACK_SKIP_ROWS: u32 = 0x0CF3;
pub const GL_UNPACK_SKIP_PIXELS: u32 = 0x0CF4;
pub const GL_UNPACK_ALIGNMENT: u32 = 0x0CF5;
pub const GL_PACK_SWAP_BYTES: u32 = 0x0D00;
pub const GL_PACK_LSB_FIRST: u32 = 0x0D01;
pub const GL_PACK_ROW_LENGTH: u32 = 0x0D02;
pub const GL_PACK_SKIP_ROWS: u32 = 0x0D03;
pub const GL_PACK_SKIP_PIXELS: u32 = 0x0D04;
pub const GL_PACK_ALIGNMENT: u32 = 0x0D05;
pub const GL_MAP_COLOR: u32 = 0x0D10;
pub const GL_MAP_STENCIL: u32 = 0x0D11;
pub const GL_INDEX_SHIFT: u32 = 0x0D12;
pub const GL_INDEX_OFFSET: u32 = 0x0D13;
pub const GL_RED_SCALE: u32 = 0x0D14;
pub const GL_RED_BIAS: u32 = 0x0D15;
pub const GL_ZOOM_X: u32 = 0x0D16;
pub const GL_ZOOM_Y: u32 = 0x0D17;
pub const GL_GREEN_SCALE: u32 = 0x0D18;
pub const GL_GREEN_BIAS: u32 = 0x0D19;
pub const GL_BLUE_SCALE: u32 = 0x0D1A;
pub const GL_BLUE_BIAS: u32 = 0x0D1B;
pub const GL_ALPHA_SCALE: u32 = 0x0D1C;
pub const GL_ALPHA_BIAS: u32 = 0x0D1D;
pub const GL_DEPTH_SCALE: u32 = 0x0D1E;
pub const GL_DEPTH_BIAS: u32 = 0x0D1F;
pub const GL_MAX_EVAL_ORDER: u32 = 0x0D30;
pub const GL_MAX_LIGHTS: u32 = 0x0D31;
pub const GL_MAX_CLIP_PLANES: u32 = 0x0D32;
pub const GL_MAX_TEXTURE_SIZE: u32 = 0x0D33;
pub const GL_MAX_PIXEL_MAP_TABLE: u32 = 0x0D34;
pub const GL_MAX_ATTRIB_STACK_DEPTH: u32 = 0x0D35;
pub const GL_MAX_MODELVIEW_STACK_DEPTH: u32 = 0x0D36;
pub const GL_MAX_NAME_STACK_DEPTH: u32 = 0x0D37;
pub const GL_MAX_PROJECTION_STACK_DEPTH: u32 = 0x0D38;
pub const GL_MAX_TEXTURE_STACK_DEPTH: u32 = 0x0D39;
pub const GL_MAX_VIEWPORT_DIMS: u32 = 0x0D3A;
pub const GL_MAX_CLIENT_ATTRIB_STACK_DEPTH: u32 = 0x0D3B;
pub const GL_SUBPIXEL_BITS: u32 = 0x0D50;
pub const GL_INDEX_BITS: u32 = 0x0D51;
pub const GL_RED_BITS: u32 = 0x0D52;
pub const GL_GREEN_BITS: u32 = 0x0D53;
pub const GL_BLUE_BITS: u32 = 0x0D54;
pub const GL_ALPHA_BITS: u32 = 0x0D55;
pub const GL_DEPTH_BITS: u32 = 0x0D56;
pub const GL_STENCIL_BITS: u32 = 0x0D57;
pub const GL_ACCUM_RED_BITS: u32 = 0x0D58;
pub const GL_ACCUM_GREEN_BITS: u32 = 0x0D59;
pub const GL_ACCUM_BLUE_BITS: u32 = 0x0D5A;
pub const GL_ACCUM_ALPHA_BITS: u32 = 0x0D5B;
pub const GL_NAME_STACK_DEPTH: u32 = 0x0D70;
pub const GL_AUTO_NORMAL: u32 = 0x0D80;
pub const GL_MAP1_COLOR_4: u32 = 0x0D90;
pub const GL_MAP1_INDEX: u32 = 0x0D91;
pub const GL_MAP1_NORMAL: u32 = 0x0D92;
pub const GL_MAP1_TEXTURE_COORD_1: u32 = 0x0D93;
pub const GL_MAP1_TEXTURE_COORD_2: u32 = 0x0D94;
pub const GL_MAP1_TEXTURE_COORD_3: u32 = 0x0D95;
pub const GL_MAP1_TEXTURE_COORD_4: u32 = 0x0D96;
pub const GL_MAP1_VERTEX_3: u32 = 0x0D97;
pub const GL_MAP1_VERTEX_4: u32 = 0x0D98;
pub const GL_MAP2_COLOR_4: u32 = 0x0DB0;
pub const GL_MAP2_INDEX: u32 = 0x0DB1;
pub const GL_MAP2_NORMAL: u32 = 0x0DB2;
pub const GL_MAP2_TEXTURE_COORD_1: u32 = 0x0DB3;
pub const GL_MAP2_TEXTURE_COORD_2: u32 = 0x0DB4;
pub const GL_MAP2_TEXTURE_COORD_3: u32 = 0x0DB5;
pub const GL_MAP2_TEXTURE_COORD_4: u32 = 0x0DB6;
pub const GL_MAP2_VERTEX_3: u32 = 0x0DB7;
pub const GL_MAP2_VERTEX_4: u32 = 0x0DB8;
pub const GL_MAP1_GRID_DOMAIN: u32 = 0x0DD0;
pub const GL_MAP1_GRID_SEGMENTS: u32 = 0x0DD1;
pub const GL_MAP2_GRID_DOMAIN: u32 = 0x0DD2;
pub const GL_MAP2_GRID_SEGMENTS: u32 = 0x0DD3;
pub const GL_TEXTURE_1D: u32 = 0x0DE0;
pub const GL_TEXTURE_2D: u32 = 0x0DE1;
pub const GL_FEEDBACK_BUFFER_POINTER: u32 = 0x0DF0;
pub const GL_FEEDBACK_BUFFER_SIZE: u32 = 0x0DF1;
pub const GL_FEEDBACK_BUFFER_TYPE: u32 = 0x0DF2;
pub const GL_SELECTION_BUFFER_POINTER: u32 = 0x0DF3;
pub const GL_SELECTION_BUFFER_SIZE: u32 = 0x0DF4;

/* GetTextureParameter */
pub const GL_TEXTURE_WIDTH: u32 = 0x1000;
pub const GL_TEXTURE_HEIGHT: u32 = 0x1001;
pub const GL_TEXTURE_INTERNAL_FORMAT: u32 = 0x1003;
pub const GL_TEXTURE_BORDER_COLOR: u32 = 0x1004;
pub const GL_TEXTURE_BORDER: u32 = 0x1005;

/* HintMode */
pub const GL_DONT_CARE: u32 = 0x1100;
pub const GL_FASTEST: u32 = 0x1101;
pub const GL_NICEST: u32 = 0x1102;

/* LightName */
pub const GL_LIGHT0: u32 = 0x4000;
pub const GL_LIGHT1: u32 = 0x4001;
pub const GL_LIGHT2: u32 = 0x4002;
pub const GL_LIGHT3: u32 = 0x4003;
pub const GL_LIGHT4: u32 = 0x4004;
pub const GL_LIGHT5: u32 = 0x4005;
pub const GL_LIGHT6: u32 = 0x4006;
pub const GL_LIGHT7: u32 = 0x4007;

/* LightParameter */
pub const GL_AMBIENT: u32 = 0x1200;
pub const GL_DIFFUSE: u32 = 0x1201;
pub const GL_SPECULAR: u32 = 0x1202;
pub const GL_POSITION: u32 = 0x1203;
pub const GL_SPOT_DIRECTION: u32 = 0x1204;
pub const GL_SPOT_EXPONENT: u32 = 0x1205;
pub const GL_SPOT_CUTOFF: u32 = 0x1206;
pub const GL_CONSTANT_ATTENUATION: u32 = 0x1207;
pub const GL_LINEAR_ATTENUATION: u32 = 0x1208;
pub const GL_QUADRATIC_ATTENUATION: u32 = 0x1209;

/* ListMode */
pub const GL_COMPILE: u32 = 0x1300;
pub const GL_COMPILE_AND_EXECUTE: u32 = 0x1301;

/* LogicOp */
pub const GL_CLEAR: u32 = 0x1500;
pub const GL_AND: u32 = 0x1501;
pub const GL_AND_REVERSE: u32 = 0x1502;
pub const GL_COPY: u32 = 0x1503;
pub const GL_AND_INVERTED: u32 = 0x1504;
pub const GL_NOOP: u32 = 0x1505;
pub const GL_XOR: u32 = 0x1506;
pub const GL_OR: u32 = 0x1507;
pub const GL_NOR: u32 = 0x1508;
pub const GL_EQUIV: u32 = 0x1509;
pub const GL_INVERT: u32 = 0x150A;
pub const GL_OR_REVERSE: u32 = 0x150B;
pub const GL_COPY_INVERTED: u32 = 0x150C;
pub const GL_OR_INVERTED: u32 = 0x150D;
pub const GL_NAND: u32 = 0x150E;
pub const GL_SET: u32 = 0x150F;

/* MaterialParameter */
pub const GL_EMISSION: u32 = 0x1600;
pub const GL_SHININESS: u32 = 0x1601;
pub const GL_AMBIENT_AND_DIFFUSE: u32 = 0x1602;
pub const GL_COLOR_INDEXES: u32 = 0x1603;

/* MatrixMode */
pub const GL_MODELVIEW: u32 = 0x1700;
pub const GL_PROJECTION: u32 = 0x1701;
pub const GL_TEXTURE0: u32 = 0x1702;
pub const GL_TEXTURE1: u32 = 0x1703;
pub const GL_TEXTURE2: u32 = 0x1704;
pub const GL_TEXTURE3: u32 = 0x1705;

/* PixelCopyType */
pub const GL_COLOR: u32 = 0x1800;
pub const GL_DEPTH: u32 = 0x1801;
pub const GL_STENCIL: u32 = 0x1802;

/* PixelFormat */
pub const GL_COLOR_INDEX: u32 = 0x1900;
pub const GL_STENCIL_INDEX: u32 = 0x1901;
pub const GL_DEPTH_COMPONENT: u32 = 0x1902;
pub const GL_RED: u32 = 0x1903;
pub const GL_GREEN: u32 = 0x1904;
pub const GL_BLUE: u32 = 0x1905;
pub const GL_ALPHA: u32 = 0x1906;
pub const GL_RGB: u32 = 0x1907;
pub const GL_RGBA: u32 = 0x1908;
pub const GL_LUMINANCE: u32 = 0x1909;
pub const GL_LUMINANCE_ALPHA: u32 = 0x190A;

/* PixelType */
pub const GL_BITMAP: u32 = 0x1A00;

/* PolygonMode */
pub const GL_POINT: u32 = 0x1B00;
pub const GL_LINE: u32 = 0x1B01;
pub const GL_FILL: u32 = 0x1B02;

/* RenderingMode */
pub const GL_RENDER: u32 = 0x1C00;
pub const GL_FEEDBACK: u32 = 0x1C01;
pub const GL_SELECT: u32 = 0x1C02;

/* ShadingModel */
pub const GL_FLAT: u32 = 0x1D00;
pub const GL_SMOOTH: u32 = 0x1D01;

/* StencilOp */
/*      GL_ZERO */
pub const GL_KEEP: u32 = 0x1E00;
pub const GL_REPLACE: u32 = 0x1E01;
pub const GL_INCR: u32 = 0x1E02;
pub const GL_DECR: u32 = 0x1E03;
/*      GL_INVERT */

/* StringName */
pub const GL_VENDOR: u32 = 0x1F00;
pub const GL_RENDERER: u32 = 0x1F01;
pub const GL_VERSION: u32 = 0x1F02;
pub const GL_EXTENSIONS: u32 = 0x1F03;

/* TextureCoordName */
pub const GL_S: u32 = 0x2000;
pub const GL_T: u32 = 0x2001;
pub const GL_R: u32 = 0x2002;
pub const GL_Q: u32 = 0x2003;

/* TextureEnvMode */
pub const GL_MODULATE: u32 = 0x2100;
pub const GL_DECAL: u32 = 0x2101;

/* TextureEnvParameter */
pub const GL_TEXTURE_ENV_MODE: u32 = 0x2200;
pub const GL_TEXTURE_ENV_COLOR: u32 = 0x2201;

/* TextureEnvTarget */
pub const GL_TEXTURE_ENV: u32 = 0x2300;

/* TextureGenMode */
pub const GL_EYE_LINEAR: u32 = 0x2400;
pub const GL_OBJECT_LINEAR: u32 = 0x2401;
pub const GL_SPHERE_MAP: u32 = 0x2402;

/* TextureGenParameter */
pub const GL_TEXTURE_GEN_MODE: u32 = 0x2500;
pub const GL_OBJECT_PLANE: u32 = 0x2501;
pub const GL_EYE_PLANE: u32 = 0x2502;

/* TextureMagFilter */
pub const GL_NEAREST: u32 = 0x2600;
pub const GL_LINEAR: u32 = 0x2601;

/* TextureMinFilter */
pub const GL_NEAREST_MIPMAP_NEAREST: u32 = 0x2700;
pub const GL_LINEAR_MIPMAP_NEAREST: u32 = 0x2701;
pub const GL_NEAREST_MIPMAP_LINEAR: u32 = 0x2702;
pub const GL_LINEAR_MIPMAP_LINEAR: u32 = 0x2703;

/* TextureParameterName */
pub const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const GL_TEXTURE_WRAP_S: u32 = 0x2802;
pub const GL_TEXTURE_WRAP_T: u32 = 0x2803;

// PORT: Anisotropy stuff
pub const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
pub const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;

//PORT - TPL stuff
pub const GL_TPL4_EXT: u32 = 0x9991;
pub const GL_TPL8_EXT: u32 = 0x9992;
pub const GL_TPL16_EXT: u32 = 0x9993;
pub const GL_TPL32_EXT: u32 = 0x9994;

// PORT: DDS Stuff
pub const GL_DDS_RGBA_EXT: u32 = 0x9998;
pub const GL_RGB_SWIZZLE_EXT: u32 = 0x9999;

/* TextureWrapMode */
pub const GL_CLAMP: u32 = 0x2900;
pub const GL_REPEAT: u32 = 0x2901;

/* ClientAttribMask */
pub const GL_CLIENT_PIXEL_STORE_BIT: u32 = 0x00000001;
pub const GL_CLIENT_VERTEX_ARRAY_BIT: u32 = 0x00000002;
pub const GL_CLIENT_ALL_ATTRIB_BITS: u32 = 0xffffffff;

/* polygon_offset */
pub const GL_POLYGON_OFFSET_FACTOR: u32 = 0x8038;
pub const GL_POLYGON_OFFSET_UNITS: u32 = 0x2A00;
pub const GL_POLYGON_OFFSET_POINT: u32 = 0x2A01;
pub const GL_POLYGON_OFFSET_LINE: u32 = 0x2A02;
pub const GL_POLYGON_OFFSET_FILL: u32 = 0x8037;

/* texture */
pub const GL_ALPHA4: u32 = 0x803B;
pub const GL_ALPHA8: u32 = 0x803C;
pub const GL_ALPHA12: u32 = 0x803D;
pub const GL_ALPHA16: u32 = 0x803E;
pub const GL_LUMINANCE4: u32 = 0x803F;
pub const GL_LUMINANCE8: u32 = 0x8040;
pub const GL_LUMINANCE12: u32 = 0x8041;
pub const GL_LUMINANCE16: u32 = 0x8042;
pub const GL_LUMINANCE4_ALPHA4: u32 = 0x8043;
pub const GL_LUMINANCE6_ALPHA2: u32 = 0x8044;
pub const GL_LUMINANCE8_ALPHA8: u32 = 0x8045;
pub const GL_LUMINANCE12_ALPHA4: u32 = 0x8046;
pub const GL_LUMINANCE12_ALPHA12: u32 = 0x8047;
pub const GL_LUMINANCE16_ALPHA16: u32 = 0x8048;
pub const GL_INTENSITY: u32 = 0x8049;
pub const GL_INTENSITY4: u32 = 0x804A;
pub const GL_INTENSITY8: u32 = 0x804B;
pub const GL_INTENSITY12: u32 = 0x804C;
pub const GL_INTENSITY16: u32 = 0x804D;
pub const GL_R3_G3_B2: u32 = 0x2A10;
pub const GL_RGB4: u32 = 0x804F;
pub const GL_RGB5: u32 = 0x8050;
pub const GL_RGB8: u32 = 0x8051;
pub const GL_RGB10: u32 = 0x8052;
pub const GL_RGB12: u32 = 0x8053;
pub const GL_RGB16: u32 = 0x8054;
pub const GL_RGBA2: u32 = 0x8055;
pub const GL_RGBA4: u32 = 0x8056;
pub const GL_RGB5_A1: u32 = 0x8057;
pub const GL_RGBA8: u32 = 0x8058;
pub const GL_RGB10_A2: u32 = 0x8059;
pub const GL_RGBA12: u32 = 0x805A;
pub const GL_RGBA16: u32 = 0x805B;
pub const GL_TEXTURE_RED_SIZE: u32 = 0x805C;
pub const GL_TEXTURE_GREEN_SIZE: u32 = 0x805D;
pub const GL_TEXTURE_BLUE_SIZE: u32 = 0x805E;
pub const GL_TEXTURE_ALPHA_SIZE: u32 = 0x805F;
pub const GL_TEXTURE_LUMINANCE_SIZE: u32 = 0x8060;
pub const GL_TEXTURE_INTENSITY_SIZE: u32 = 0x8061;
pub const GL_PROXY_TEXTURE_1D: u32 = 0x8063;
pub const GL_PROXY_TEXTURE_2D: u32 = 0x8064;

/* texture_object */
pub const GL_TEXTURE_PRIORITY: u32 = 0x8066;
pub const GL_TEXTURE_RESIDENT: u32 = 0x8067;
pub const GL_TEXTURE_BINDING_1D: u32 = 0x8068;
pub const GL_TEXTURE_BINDING_2D: u32 = 0x8069;

/* vertex_array */
pub const GL_VERTEX_ARRAY: u32 = 0x8074;
pub const GL_NORMAL_ARRAY: u32 = 0x8075;
pub const GL_COLOR_ARRAY: u32 = 0x8076;
pub const GL_INDEX_ARRAY: u32 = 0x8077;
pub const GL_TEXTURE_COORD_ARRAY: u32 = 0x8078;
pub const GL_EDGE_FLAG_ARRAY: u32 = 0x8079;
pub const GL_VERTEX_ARRAY_SIZE: u32 = 0x807A;
pub const GL_VERTEX_ARRAY_TYPE: u32 = 0x807B;
pub const GL_VERTEX_ARRAY_STRIDE: u32 = 0x807C;
pub const GL_NORMAL_ARRAY_TYPE: u32 = 0x807E;
pub const GL_NORMAL_ARRAY_STRIDE: u32 = 0x807F;
pub const GL_COLOR_ARRAY_SIZE: u32 = 0x8081;
pub const GL_COLOR_ARRAY_TYPE: u32 = 0x8082;
pub const GL_COLOR_ARRAY_STRIDE: u32 = 0x8083;
pub const GL_INDEX_ARRAY_TYPE: u32 = 0x8085;
pub const GL_INDEX_ARRAY_STRIDE: u32 = 0x8086;
pub const GL_TEXTURE_COORD_ARRAY_SIZE: u32 = 0x8088;
pub const GL_TEXTURE_COORD_ARRAY_TYPE: u32 = 0x8089;
pub const GL_TEXTURE_COORD_ARRAY_STRIDE: u32 = 0x808A;
pub const GL_EDGE_FLAG_ARRAY_STRIDE: u32 = 0x808C;
pub const GL_VERTEX_ARRAY_POINTER: u32 = 0x808E;
pub const GL_NORMAL_ARRAY_POINTER: u32 = 0x808F;
pub const GL_COLOR_ARRAY_POINTER: u32 = 0x8090;
pub const GL_INDEX_ARRAY_POINTER: u32 = 0x8091;
pub const GL_TEXTURE_COORD_ARRAY_POINTER: u32 = 0x8092;
pub const GL_EDGE_FLAG_ARRAY_POINTER: u32 = 0x8093;
pub const GL_V2F: u32 = 0x2A20;
pub const GL_V3F: u32 = 0x2A21;
pub const GL_C4UB_V2F: u32 = 0x2A22;
pub const GL_C4UB_V3F: u32 = 0x2A23;
pub const GL_C3F_V3F: u32 = 0x2A24;
pub const GL_N3F_V3F: u32 = 0x2A25;
pub const GL_C4F_N3F_V3F: u32 = 0x2A26;
pub const GL_T2F_V3F: u32 = 0x2A27;
pub const GL_T4F_V4F: u32 = 0x2A28;
pub const GL_T2F_C4UB_V3F: u32 = 0x2A29;
pub const GL_T2F_C3F_V3F: u32 = 0x2A2A;
pub const GL_T2F_N3F_V3F: u32 = 0x2A2B;
pub const GL_T2F_C4F_N3F_V3F: u32 = 0x2A2C;
pub const GL_T4F_C4F_N3F_V4F: u32 = 0x2A2D;

/* Extensions */
pub const GL_EXT_vertex_array: u32 = 1;
pub const GL_EXT_bgra: u32 = 1;
pub const GL_EXT_paletted_texture: u32 = 1;

/* EXT_vertex_array */
pub const GL_VERTEX_ARRAY_EXT: u32 = 0x8074;
pub const GL_NORMAL_ARRAY_EXT: u32 = 0x8075;
pub const GL_COLOR_ARRAY_EXT: u32 = 0x8076;
pub const GL_INDEX_ARRAY_EXT: u32 = 0x8077;
pub const GL_TEXTURE_COORD_ARRAY_EXT: u32 = 0x8078;
pub const GL_EDGE_FLAG_ARRAY_EXT: u32 = 0x8079;
pub const GL_VERTEX_ARRAY_SIZE_EXT: u32 = 0x807A;
pub const GL_VERTEX_ARRAY_TYPE_EXT: u32 = 0x807B;
pub const GL_VERTEX_ARRAY_STRIDE_EXT: u32 = 0x807C;
pub const GL_VERTEX_ARRAY_COUNT_EXT: u32 = 0x807D;
pub const GL_NORMAL_ARRAY_TYPE_EXT: u32 = 0x807E;
pub const GL_NORMAL_ARRAY_STRIDE_EXT: u32 = 0x807F;
pub const GL_NORMAL_ARRAY_COUNT_EXT: u32 = 0x8080;
pub const GL_COLOR_ARRAY_SIZE_EXT: u32 = 0x8081;
pub const GL_COLOR_ARRAY_TYPE_EXT: u32 = 0x8082;
pub const GL_COLOR_ARRAY_STRIDE_EXT: u32 = 0x8083;
pub const GL_COLOR_ARRAY_COUNT_EXT: u32 = 0x8084;
pub const GL_INDEX_ARRAY_TYPE_EXT: u32 = 0x8085;
pub const GL_INDEX_ARRAY_STRIDE_EXT: u32 = 0x8086;
pub const GL_INDEX_ARRAY_COUNT_EXT: u32 = 0x8087;
pub const GL_TEXTURE_COORD_ARRAY_SIZE_EXT: u32 = 0x8088;
pub const GL_TEXTURE_COORD_ARRAY_TYPE_EXT: u32 = 0x8089;
pub const GL_TEXTURE_COORD_ARRAY_STRIDE_EXT: u32 = 0x808A;
pub const GL_TEXTURE_COORD_ARRAY_COUNT_EXT: u32 = 0x808B;
pub const GL_EDGE_FLAG_ARRAY_STRIDE_EXT: u32 = 0x808C;
pub const GL_EDGE_FLAG_ARRAY_COUNT_EXT: u32 = 0x808D;
pub const GL_VERTEX_ARRAY_POINTER_EXT: u32 = 0x808E;
pub const GL_NORMAL_ARRAY_POINTER_EXT: u32 = 0x808F;
pub const GL_COLOR_ARRAY_POINTER_EXT: u32 = 0x8090;
pub const GL_INDEX_ARRAY_POINTER_EXT: u32 = 0x8091;
pub const GL_TEXTURE_COORD_ARRAY_POINTER_EXT: u32 = 0x8092;
pub const GL_EDGE_FLAG_ARRAY_POINTER_EXT: u32 = 0x8093;
pub const GL_DOUBLE_EXT: u32 = GL_DOUBLE;

/* EXT_bgra */
pub const GL_BGR_EXT: u32 = 0x80E0;
pub const GL_BGRA_EXT: u32 = 0x80E1;

/* EXT_paletted_texture */

/* These must match the GL_COLOR_TABLE_*_SGI enumerants */
pub const GL_COLOR_TABLE_FORMAT_EXT: u32 = 0x80D8;
pub const GL_COLOR_TABLE_WIDTH_EXT: u32 = 0x80D9;
pub const GL_COLOR_TABLE_RED_SIZE_EXT: u32 = 0x80DA;
pub const GL_COLOR_TABLE_GREEN_SIZE_EXT: u32 = 0x80DB;
pub const GL_COLOR_TABLE_BLUE_SIZE_EXT: u32 = 0x80DC;
pub const GL_COLOR_TABLE_ALPHA_SIZE_EXT: u32 = 0x80DD;
pub const GL_COLOR_TABLE_LUMINANCE_SIZE_EXT: u32 = 0x80DE;
pub const GL_COLOR_TABLE_INTENSITY_SIZE_EXT: u32 = 0x80DF;

pub const GL_COLOR_INDEX1_EXT: u32 = 0x80E2;
pub const GL_COLOR_INDEX2_EXT: u32 = 0x80E3;
pub const GL_COLOR_INDEX4_EXT: u32 = 0x80E4;
pub const GL_COLOR_INDEX8_EXT: u32 = 0x80E5;
pub const GL_COLOR_INDEX12_EXT: u32 = 0x80E6;
pub const GL_COLOR_INDEX16_EXT: u32 = 0x80E7;

// VVFIXME New Constants from Jedi
pub const GL_VSYNC: u32 = 0x813F;
pub const GL_DDS_RGB16_EXT: u32 = 0x9997;
pub const GL_DDS_RGBA32_EXT: u32 = 0x9998;
pub const GL_RGB_SWIZZLE_EXT: u32 = 0x9999;

//	VVFIXME - New constants for linear format textures.
// These numbers are just made up. This is awful.
pub const GL_LIN_RGBA8: u32 = 0x8E01;
pub const GL_LIN_RGBA: u32 = 0x8E02;
pub const GL_LIN_RGB8: u32 = 0x8E03;
pub const GL_LIN_RGB: u32 = 0x8E04;

//===========================================================================

/*
** multitexture extension definitions
*/
pub const GL_ACTIVE_TEXTURE_ARB: u32 = 0x84E0;
pub const GL_CLIENT_ACTIVE_TEXTURE_ARB: u32 = 0x84E1;
pub const GL_MAX_ACTIVE_TEXTURES_ARB: u32 = 0x84E2;

pub const GL_TEXTURE0_ARB: u32 = 0x84C0;
pub const GL_TEXTURE1_ARB: u32 = 0x84C1;
pub const GL_TEXTURE2_ARB: u32 = 0x84C2;
pub const GL_TEXTURE3_ARB: u32 = 0x84C3;

/* Function pointer types */
pub type PFNGLMULTITEXCOORD1DARBPROC = extern "C" fn(GLenum, GLdouble);
pub type PFNGLMULTITEXCOORD1DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD1FARBPROC = extern "C" fn(GLenum, GLfloat);
pub type PFNGLMULTITEXCOORD1FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD1IARBPROC = extern "C" fn(GLenum, GLint);
pub type PFNGLMULTITEXCOORD1IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD1SARBPROC = extern "C" fn(GLenum, GLshort);
pub type PFNGLMULTITEXCOORD1SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLMULTITEXCOORD2DARBPROC = extern "C" fn(GLenum, GLdouble, GLdouble);
pub type PFNGLMULTITEXCOORD2DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD2FARBPROC = extern "C" fn(GLenum, GLfloat, GLfloat);
pub type PFNGLMULTITEXCOORD2FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD2IARBPROC = extern "C" fn(GLenum, GLint, GLint);
pub type PFNGLMULTITEXCOORD2IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD2SARBPROC = extern "C" fn(GLenum, GLshort, GLshort);
pub type PFNGLMULTITEXCOORD2SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLMULTITEXCOORD3DARBPROC = extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble);
pub type PFNGLMULTITEXCOORD3DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD3FARBPROC = extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat);
pub type PFNGLMULTITEXCOORD3FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD3IARBPROC = extern "C" fn(GLenum, GLint, GLint, GLint);
pub type PFNGLMULTITEXCOORD3IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD3SARBPROC = extern "C" fn(GLenum, GLshort, GLshort, GLshort);
pub type PFNGLMULTITEXCOORD3SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLMULTITEXCOORD4DARBPROC = extern "C" fn(GLenum, GLdouble, GLdouble, GLdouble, GLdouble);
pub type PFNGLMULTITEXCOORD4DVARBPROC = extern "C" fn(GLenum, *const GLdouble);
pub type PFNGLMULTITEXCOORD4FARBPROC = extern "C" fn(GLenum, GLfloat, GLfloat, GLfloat, GLfloat);
pub type PFNGLMULTITEXCOORD4FVARBPROC = extern "C" fn(GLenum, *const GLfloat);
pub type PFNGLMULTITEXCOORD4IARBPROC = extern "C" fn(GLenum, GLint, GLint, GLint, GLint);
pub type PFNGLMULTITEXCOORD4IVARBPROC = extern "C" fn(GLenum, *const GLint);
pub type PFNGLMULTITEXCOORD4SARBPROC = extern "C" fn(GLenum, GLshort, GLshort, GLshort, GLshort);
pub type PFNGLMULTITEXCOORD4SVARBPROC = extern "C" fn(GLenum, *const GLshort);
pub type PFNGLACTIVETEXTUREARBPROC = extern "C" fn(GLenum);
pub type PFNGLCLIENTACTIVETEXTUREARBPROC = extern "C" fn(GLenum);

/*
** extension constants
*/
pub static mut qglMultiTexCoord2fARB: Option<extern "C" fn(GLenum, GLfloat, GLfloat)> = None;
pub static mut qglActiveTextureARB: Option<extern "C" fn(GLenum)> = None;
pub static mut qglClientActiveTextureARB: Option<extern "C" fn(GLenum)> = None;

pub static mut qglLockArraysEXT: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglUnlockArraysEXT: Option<extern "C" fn()> = None;

//----(SA)	from Raven
pub static mut qglPointParameterfEXT: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglPointParameterfvEXT: Option<extern "C" fn(GLenum, *mut GLfloat)> = None;
//----(SA)	end

// S3TC compression constants
pub const GL_RGB_S3TC: u32 = 0x83A0;
pub const GL_RGB4_S3TC: u32 = 0x83A1;
// More, grabbed from wolf code PORT
pub const GL_COMPRESSED_RGB_S3TC_DXT1_EXT: u32 = 0x83F0;
pub const GL_COMPRESSED_RGBA_S3TC_DXT1_EXT: u32 = 0x83F1;
pub const GL_COMPRESSED_RGBA_S3TC_DXT3_EXT: u32 = 0x83F2;
pub const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT: u32 = 0x83F3;

// And more, also from old wolf code:
// GR - update enumerants
pub const GL_PN_TRIANGLES_ATI: u32 = 0x87F0;
pub const GL_MAX_PN_TRIANGLES_TESSELATION_LEVEL_ATI: u32 = 0x87F1;
pub const GL_PN_TRIANGLES_POINT_MODE_ATI: u32 = 0x87F2;
pub const GL_PN_TRIANGLES_NORMAL_MODE_ATI: u32 = 0x87F3;
pub const GL_PN_TRIANGLES_TESSELATION_LEVEL_ATI: u32 = 0x87F4;
pub const GL_PN_TRIANGLES_POINT_MODE_LINEAR_ATI: u32 = 0x87F5;
pub const GL_PN_TRIANGLES_POINT_MODE_CUBIC_ATI: u32 = 0x87F6;
pub const GL_PN_TRIANGLES_NORMAL_MODE_LINEAR_ATI: u32 = 0x87F7;
pub const GL_PN_TRIANGLES_NORMAL_MODE_QUADRATIC_ATI: u32 = 0x87F8;

pub static mut qglPNTrianglesiATI: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglPNTrianglesfATI: Option<extern "C" fn(GLenum, GLfloat)> = None;

pub const GL_FOG_DISTANCE_MODE_NV: u32 = 0x855A;
pub const GL_EYE_RADIAL_NV: u32 = 0x855B;
pub const GL_EYE_PLANE_ABSOLUTE_NV: u32 = 0x855C;

//===========================================================================

pub static mut qglAccum: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglAlphaFunc: Option<extern "C" fn(GLenum, GLclampf)> = None;
pub static mut qglAreTexturesResident: Option<extern "C" fn(GLsizei, *const GLuint, *mut GLboolean) -> GLboolean> = None;
pub static mut qglArrayElement: Option<extern "C" fn(GLint)> = None;
pub static mut qglBegin: Option<extern "C" fn(GLenum)> = None;
pub static mut qglBeginEXT: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLint)> = None;
pub static mut qglBeginFrame: Option<extern "C" fn() -> GLboolean> = None;
pub static mut qglBeginShadow: Option<extern "C" fn()> = None;
pub static mut qglBindTexture: Option<extern "C" fn(GLenum, GLuint)> = None;
pub static mut qglBitmap: Option<extern "C" fn(GLsizei, GLsizei, GLfloat, GLfloat, GLfloat, GLfloat, *const GLubyte)> = None;
pub static mut qglBlendFunc: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglCallList: Option<extern "C" fn(GLuint)> = None;
pub static mut qglCallLists: Option<extern "C" fn(GLsizei, GLenum, *const GLvoid)> = None;
pub static mut qglClear: Option<extern "C" fn(GLbitfield)> = None;
pub static mut qglClearAccum: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglClearColor: Option<extern "C" fn(GLclampf, GLclampf, GLclampf, GLclampf)> = None;
pub static mut qglClearDepth: Option<extern "C" fn(GLclampd)> = None;
pub static mut qglClearIndex: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglClearStencil: Option<extern "C" fn(GLint)> = None;
pub static mut qglClipPlane: Option<extern "C" fn(GLenum, *const GLdouble)> = None;
pub static mut qglColor3b: Option<extern "C" fn(GLbyte, GLbyte, GLbyte)> = None;
pub static mut qglColor3bv: Option<extern "C" fn(*const GLbyte)> = None;
pub static mut qglColor3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglColor3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglColor3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglColor3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglColor3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglColor3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglColor3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglColor3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglColor3ub: Option<extern "C" fn(GLubyte, GLubyte, GLubyte)> = None;
pub static mut qglColor3ubv: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglColor3ui: Option<extern "C" fn(GLuint, GLuint, GLuint)> = None;
pub static mut qglColor3uiv: Option<extern "C" fn(*const GLuint)> = None;
pub static mut qglColor3us: Option<extern "C" fn(GLushort, GLushort, GLushort)> = None;
pub static mut qglColor3usv: Option<extern "C" fn(*const GLushort)> = None;
pub static mut qglColor4b: Option<extern "C" fn(GLbyte, GLbyte, GLbyte, GLbyte)> = None;
pub static mut qglColor4bv: Option<extern "C" fn(*const GLbyte)> = None;
pub static mut qglColor4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglColor4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglColor4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglColor4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglColor4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglColor4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglColor4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglColor4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglColor4ub: Option<extern "C" fn(GLubyte, GLubyte, GLubyte, GLubyte)> = None;
pub static mut qglColor4ubv: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglColor4ui: Option<extern "C" fn(GLuint, GLuint, GLuint, GLuint)> = None;
pub static mut qglColor4uiv: Option<extern "C" fn(*const GLuint)> = None;
pub static mut qglColor4us: Option<extern "C" fn(GLushort, GLushort, GLushort, GLushort)> = None;
pub static mut qglColor4usv: Option<extern "C" fn(*const GLushort)> = None;
pub static mut qglColorMask: Option<extern "C" fn(GLboolean, GLboolean, GLboolean, GLboolean)> = None;
pub static mut qglColorMaterial: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglColorPointer: Option<extern "C" fn(GLint, GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglCopyPixels: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei, GLenum)> = None;
pub static mut qglCopyTexImage1D: Option<extern "C" fn(GLenum, GLint, GLenum, GLint, GLint, GLsizei, GLint)> = None;
pub static mut qglCopyTexImage2D: Option<extern "C" fn(GLenum, GLint, GLenum, GLint, GLint, GLsizei, GLsizei, GLint)> = None;
pub static mut qglCopyTexSubImage1D: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLsizei)> = None;
pub static mut qglCopyTexSubImage2D: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLint, GLint, GLsizei, GLsizei)> = None;
pub static mut qglCullFace: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDeleteLists: Option<extern "C" fn(GLuint, GLsizei)> = None;
pub static mut qglDeleteTextures: Option<extern "C" fn(GLsizei, *const GLuint)> = None;
pub static mut qglDepthFunc: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDepthMask: Option<extern "C" fn(GLboolean)> = None;
pub static mut qglDepthRange: Option<extern "C" fn(GLclampd, GLclampd)> = None;
pub static mut qglDisable: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDisableClientState: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDrawArrays: Option<extern "C" fn(GLenum, GLint, GLsizei)> = None;
pub static mut qglDrawBuffer: Option<extern "C" fn(GLenum)> = None;
pub static mut qglDrawElements: Option<extern "C" fn(GLenum, GLsizei, GLenum, *const GLvoid)> = None;
pub static mut qglDrawPixels: Option<extern "C" fn(GLsizei, GLsizei, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglEdgeFlag: Option<extern "C" fn(GLboolean)> = None;
pub static mut qglEdgeFlagPointer: Option<extern "C" fn(GLsizei, *const GLvoid)> = None;
pub static mut qglEdgeFlagv: Option<extern "C" fn(*const GLboolean)> = None;
pub static mut qglEnable: Option<extern "C" fn(GLenum)> = None;
pub static mut qglEnableClientState: Option<extern "C" fn(GLenum)> = None;
pub static mut qglEnd: Option<extern "C" fn()> = None;
pub static mut qglEndFrame: Option<extern "C" fn()> = None;
pub static mut qglEndShadow: Option<extern "C" fn()> = None;
pub static mut qglEndList: Option<extern "C" fn()> = None;
pub static mut qglEvalCoord1d: Option<extern "C" fn(GLdouble)> = None;
pub static mut qglEvalCoord1dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglEvalCoord1f: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglEvalCoord1fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglEvalCoord2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglEvalCoord2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglEvalCoord2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglEvalCoord2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglEvalMesh1: Option<extern "C" fn(GLenum, GLint, GLint)> = None;
pub static mut qglEvalMesh2: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLint)> = None;
pub static mut qglEvalPoint1: Option<extern "C" fn(GLint)> = None;
pub static mut qglEvalPoint2: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglFeedbackBuffer: Option<extern "C" fn(GLsizei, GLenum, *mut GLfloat)> = None;
pub static mut qglFinish: Option<extern "C" fn()> = None;
pub static mut qglFlush: Option<extern "C" fn()> = None;
pub static mut qglFlushShadow: Option<extern "C" fn()> = None;
pub static mut qglFogf: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglFogfv: Option<extern "C" fn(GLenum, *const GLfloat)> = None;
pub static mut qglFogi: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglFogiv: Option<extern "C" fn(GLenum, *const GLint)> = None;
pub static mut qglFrontFace: Option<extern "C" fn(GLenum)> = None;
pub static mut qglFrustum: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglGenLists: Option<extern "C" fn(GLsizei) -> GLuint> = None;
pub static mut qglGenTextures: Option<extern "C" fn(GLsizei, *mut GLuint)> = None;
pub static mut qglGetBooleanv: Option<extern "C" fn(GLenum, *mut GLboolean)> = None;
pub static mut qglGetClipPlane: Option<extern "C" fn(GLenum, *mut GLdouble)> = None;
pub static mut qglGetDoublev: Option<extern "C" fn(GLenum, *mut GLdouble)> = None;
pub static mut qglGetError: Option<extern "C" fn() -> GLenum> = None;
pub static mut qglGetFloatv: Option<extern "C" fn(GLenum, *mut GLfloat)> = None;
pub static mut qglGetIntegerv: Option<extern "C" fn(GLenum, *mut GLint)> = None;
pub static mut qglGetLightfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetLightiv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetMapdv: Option<extern "C" fn(GLenum, GLenum, *mut GLdouble)> = None;
pub static mut qglGetMapfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetMapiv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetMaterialfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetMaterialiv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetPixelMapfv: Option<extern "C" fn(GLenum, *mut GLfloat)> = None;
pub static mut qglGetPixelMapuiv: Option<extern "C" fn(GLenum, *mut GLuint)> = None;
pub static mut qglGetPixelMapusv: Option<extern "C" fn(GLenum, *mut GLushort)> = None;
pub static mut qglGetPointerv: Option<extern "C" fn(GLenum, *mut *mut GLvoid)> = None;
pub static mut qglGetPolygonStipple: Option<extern "C" fn(*mut GLubyte)> = None;
pub static mut qglGetString: Option<extern "C" fn(GLenum) -> *const GLubyte> = None;
pub static mut qglGetTexEnvfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexEnviv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetTexGendv: Option<extern "C" fn(GLenum, GLenum, *mut GLdouble)> = None;
pub static mut qglGetTexGenfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexGeniv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglGetTexImage: Option<extern "C" fn(GLenum, GLint, GLenum, GLenum, *mut GLvoid)> = None;
pub static mut qglGetTexLevelParameterfv: Option<extern "C" fn(GLenum, GLint, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexLevelParameteriv: Option<extern "C" fn(GLenum, GLint, GLenum, *mut GLint)> = None;
pub static mut qglGetTexParameterfv: Option<extern "C" fn(GLenum, GLenum, *mut GLfloat)> = None;
pub static mut qglGetTexParameteriv: Option<extern "C" fn(GLenum, GLenum, *mut GLint)> = None;
pub static mut qglHint: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglIndexedTriToStrip: Option<extern "C" fn(GLsizei, *const GLushort)> = None;
pub static mut qglIndexMask: Option<extern "C" fn(GLuint)> = None;
pub static mut qglIndexPointer: Option<extern "C" fn(GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglIndexd: Option<extern "C" fn(GLdouble)> = None;
pub static mut qglIndexdv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglIndexf: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglIndexfv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglIndexi: Option<extern "C" fn(GLint)> = None;
pub static mut qglIndexiv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglIndexs: Option<extern "C" fn(GLshort)> = None;
pub static mut qglIndexsv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglIndexub: Option<extern "C" fn(GLubyte)> = None;
pub static mut qglIndexubv: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglInitNames: Option<extern "C" fn()> = None;
pub static mut qglInterleavedArrays: Option<extern "C" fn(GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglIsEnabled: Option<extern "C" fn(GLenum) -> GLboolean> = None;
pub static mut qglIsList: Option<extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut qglIsTexture: Option<extern "C" fn(GLuint) -> GLboolean> = None;
pub static mut qglLightModelf: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglLightModelfv: Option<extern "C" fn(GLenum, *const GLfloat)> = None;
pub static mut qglLightModeli: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglLightModeliv: Option<extern "C" fn(GLenum, *const GLint)> = None;
pub static mut qglLightf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglLightfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglLighti: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglLightiv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglLineStipple: Option<extern "C" fn(GLint, GLushort)> = None;
pub static mut qglLineWidth: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglListBase: Option<extern "C" fn(GLuint)> = None;
pub static mut qglLoadIdentity: Option<extern "C" fn()> = None;
pub static mut qglLoadMatrixd: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglLoadMatrixf: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglLoadName: Option<extern "C" fn(GLuint)> = None;
pub static mut qglLogicOp: Option<extern "C" fn(GLenum)> = None;
pub static mut qglMap1d: Option<extern "C" fn(GLenum, GLdouble, GLdouble, GLint, GLint, *const GLdouble)> = None;
pub static mut qglMap1f: Option<extern "C" fn(GLenum, GLfloat, GLfloat, GLint, GLint, *const GLfloat)> = None;
pub static mut qglMap2d: Option<extern "C" fn(GLenum, GLdouble, GLdouble, GLint, GLint, GLdouble, GLdouble, GLint, GLint, *const GLdouble)> = None;
pub static mut qglMap2f: Option<extern "C" fn(GLenum, GLfloat, GLfloat, GLint, GLint, GLfloat, GLfloat, GLint, GLint, *const GLfloat)> = None;
pub static mut qglMapGrid1d: Option<extern "C" fn(GLint, GLdouble, GLdouble)> = None;
pub static mut qglMapGrid1f: Option<extern "C" fn(GLint, GLfloat, GLfloat)> = None;
pub static mut qglMapGrid2d: Option<extern "C" fn(GLint, GLdouble, GLdouble, GLint, GLdouble, GLdouble)> = None;
pub static mut qglMapGrid2f: Option<extern "C" fn(GLint, GLfloat, GLfloat, GLint, GLfloat, GLfloat)> = None;
pub static mut qglMaterialf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglMaterialfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglMateriali: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglMaterialiv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglMatrixMode: Option<extern "C" fn(GLenum)> = None;
pub static mut qglMultMatrixd: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglMultMatrixf: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglNewList: Option<extern "C" fn(GLuint, GLenum)> = None;
pub static mut qglNormal3b: Option<extern "C" fn(GLbyte, GLbyte, GLbyte)> = None;
pub static mut qglNormal3bv: Option<extern "C" fn(*const GLbyte)> = None;
pub static mut qglNormal3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglNormal3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglNormal3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglNormal3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglNormal3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglNormal3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglNormal3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglNormal3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglNormalPointer: Option<extern "C" fn(GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglOrtho: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglPassThrough: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglPixelMapfv: Option<extern "C" fn(GLenum, GLsizei, *const GLfloat)> = None;
pub static mut qglPixelMapuiv: Option<extern "C" fn(GLenum, GLsizei, *const GLuint)> = None;
pub static mut qglPixelMapusv: Option<extern "C" fn(GLenum, GLsizei, *const GLushort)> = None;
pub static mut qglPixelStoref: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglPixelStorei: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglPixelTransferf: Option<extern "C" fn(GLenum, GLfloat)> = None;
pub static mut qglPixelTransferi: Option<extern "C" fn(GLenum, GLint)> = None;
pub static mut qglPixelZoom: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglPointSize: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglPolygonMode: Option<extern "C" fn(GLenum, GLenum)> = None;
pub static mut qglPolygonOffset: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglPolygonStipple: Option<extern "C" fn(*const GLubyte)> = None;
pub static mut qglPopAttrib: Option<extern "C" fn()> = None;
pub static mut qglPopClientAttrib: Option<extern "C" fn()> = None;
pub static mut qglPopMatrix: Option<extern "C" fn()> = None;
pub static mut qglPopName: Option<extern "C" fn()> = None;
pub static mut qglPrioritizeTextures: Option<extern "C" fn(GLsizei, *const GLuint, *const GLclampf)> = None;
pub static mut qglPushAttrib: Option<extern "C" fn(GLbitfield)> = None;
pub static mut qglPushClientAttrib: Option<extern "C" fn(GLbitfield)> = None;
pub static mut qglPushMatrix: Option<extern "C" fn()> = None;
pub static mut qglPushName: Option<extern "C" fn(GLuint)> = None;
pub static mut qglRasterPos2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglRasterPos2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglRasterPos2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglRasterPos2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglRasterPos2i: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglRasterPos2iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglRasterPos2s: Option<extern "C" fn(GLshort, GLshort)> = None;
pub static mut qglRasterPos2sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglRasterPos3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRasterPos3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglRasterPos3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglRasterPos3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglRasterPos3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglRasterPos3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglRasterPos3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglRasterPos3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglRasterPos4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRasterPos4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglRasterPos4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglRasterPos4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglRasterPos4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglRasterPos4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglRasterPos4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglRasterPos4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglReadBuffer: Option<extern "C" fn(GLenum)> = None;
//extern  void ( * qglReadPixels )(GLint x, GLint y, GLsizei width, GLsizei height, GLenum format, GLenum type, GLsizei twidth, GLsizei theight, GLvoid *pixels);
pub static mut qglReadPixels: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, *mut GLvoid)> = None;
pub static mut qglCopyBackBufferToTexEXT: Option<extern "C" fn(c_float, c_float, c_float, c_float, c_float, c_float)> = None;
pub static mut qglCopyBackBufferToTex: Option<extern "C" fn()> = None;
pub static mut qglRectd: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRectdv: Option<extern "C" fn(*const GLdouble, *const GLdouble)> = None;
pub static mut qglRectf: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglRectfv: Option<extern "C" fn(*const GLfloat, *const GLfloat)> = None;
pub static mut qglRecti: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglRectiv: Option<extern "C" fn(*const GLint, *const GLint)> = None;
pub static mut qglRects: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglRectsv: Option<extern "C" fn(*const GLshort, *const GLshort)> = None;
pub static mut qglRenderMode: Option<extern "C" fn(GLenum) -> GLint> = None;
pub static mut qglRotated: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglRotatef: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglScaled: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglScalef: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglScissor: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei)> = None;
pub static mut qglSelectBuffer: Option<extern "C" fn(GLsizei, *mut GLuint)> = None;
pub static mut qglShadeModel: Option<extern "C" fn(GLenum)> = None;
pub static mut qglStencilFunc: Option<extern "C" fn(GLenum, GLint, GLuint)> = None;
pub static mut qglStencilMask: Option<extern "C" fn(GLuint)> = None;
pub static mut qglStencilOp: Option<extern "C" fn(GLenum, GLenum, GLenum)> = None;
pub static mut qglTexCoord1d: Option<extern "C" fn(GLdouble)> = None;
pub static mut qglTexCoord1dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord1f: Option<extern "C" fn(GLfloat)> = None;
pub static mut qglTexCoord1fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord1i: Option<extern "C" fn(GLint)> = None;
pub static mut qglTexCoord1iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord1s: Option<extern "C" fn(GLshort)> = None;
pub static mut qglTexCoord1sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoord2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglTexCoord2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglTexCoord2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord2i: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglTexCoord2iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord2s: Option<extern "C" fn(GLshort, GLshort)> = None;
pub static mut qglTexCoord2sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoord3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglTexCoord3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglTexCoord3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglTexCoord3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglTexCoord3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoord4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglTexCoord4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglTexCoord4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglTexCoord4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglTexCoord4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglTexCoord4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglTexCoord4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglTexCoord4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglTexCoordPointer: Option<extern "C" fn(GLint, GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglTexEnvf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglTexEnvfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglTexEnvi: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglTexEnviv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglTexGend: Option<extern "C" fn(GLenum, GLenum, GLdouble)> = None;
pub static mut qglTexGendv: Option<extern "C" fn(GLenum, GLenum, *const GLdouble)> = None;
pub static mut qglTexGenf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglTexGenfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglTexGeni: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglTexGeniv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglTexImage1D: Option<extern "C" fn(GLenum, GLint, GLint, GLsizei, GLint, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTexImage2D: Option<extern "C" fn(GLenum, GLint, GLint, GLsizei, GLsizei, GLint, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTexImage2DEXT: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLsizei, GLsizei, GLint, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTexParameterf: Option<extern "C" fn(GLenum, GLenum, GLfloat)> = None;
pub static mut qglTexParameterfv: Option<extern "C" fn(GLenum, GLenum, *const GLfloat)> = None;
pub static mut qglTexParameteri: Option<extern "C" fn(GLenum, GLenum, GLint)> = None;
pub static mut qglTexParameteriv: Option<extern "C" fn(GLenum, GLenum, *const GLint)> = None;
pub static mut qglTexSubImage1D: Option<extern "C" fn(GLenum, GLint, GLint, GLsizei, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTexSubImage2D: Option<extern "C" fn(GLenum, GLint, GLint, GLint, GLsizei, GLsizei, GLenum, GLenum, *const GLvoid)> = None;
pub static mut qglTranslated: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglTranslatef: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglVertex2d: Option<extern "C" fn(GLdouble, GLdouble)> = None;
pub static mut qglVertex2dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglVertex2f: Option<extern "C" fn(GLfloat, GLfloat)> = None;
pub static mut qglVertex2fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglVertex2i: Option<extern "C" fn(GLint, GLint)> = None;
pub static mut qglVertex2iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglVertex2s: Option<extern "C" fn(GLshort, GLshort)> = None;
pub static mut qglVertex2sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglVertex3d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglVertex3dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglVertex3f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglVertex3fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglVertex3i: Option<extern "C" fn(GLint, GLint, GLint)> = None;
pub static mut qglVertex3iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglVertex3s: Option<extern "C" fn(GLshort, GLshort, GLshort)> = None;
pub static mut qglVertex3sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglVertex4d: Option<extern "C" fn(GLdouble, GLdouble, GLdouble, GLdouble)> = None;
pub static mut qglVertex4dv: Option<extern "C" fn(*const GLdouble)> = None;
pub static mut qglVertex4f: Option<extern "C" fn(GLfloat, GLfloat, GLfloat, GLfloat)> = None;
pub static mut qglVertex4fv: Option<extern "C" fn(*const GLfloat)> = None;
pub static mut qglVertex4i: Option<extern "C" fn(GLint, GLint, GLint, GLint)> = None;
pub static mut qglVertex4iv: Option<extern "C" fn(*const GLint)> = None;
pub static mut qglVertex4s: Option<extern "C" fn(GLshort, GLshort, GLshort, GLshort)> = None;
pub static mut qglVertex4sv: Option<extern "C" fn(*const GLshort)> = None;
pub static mut qglVertexPointer: Option<extern "C" fn(GLint, GLenum, GLsizei, *const GLvoid)> = None;
pub static mut qglViewport: Option<extern "C" fn(GLint, GLint, GLsizei, GLsizei)> = None;
