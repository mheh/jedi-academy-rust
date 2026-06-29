#![allow(non_snake_case)]

use core::ffi::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum DSP_IMAGE_image_FX_INDICES {
    GraphI3DL2_I3DL2Reverb = 0,
    GraphXTalk_XTalk = 1,
    GraphVoice_Voice_0 = 2,
    GraphVoice_Voice_1 = 3,
    GraphVoice_Voice_2 = 4,
    GraphVoice_Voice_3 = 5,
}

pub const DSI3DL2_ENVIRONMENT_GraphI3DL2_I3DL2Reverb: [f32; 12] = [
    -1000.0, -100.0, 0.0, 1.49, 0.83, -2602.0, 0.007, 200.0, 0.011, 100.0, 100.0, 5000.0,
];

#[repr(C)]
pub struct GraphI3DL2_FX0_I3DL2Reverb_STATE {
    pub dwScratchOffset: u32,         // Offset in bytes, of scratch area for this FX
    pub dwScratchLength: u32,         // Length in DWORDS, of scratch area for this FX
    pub dwYMemoryOffset: u32,         // Offset in DSP WORDS, of Y memory area for this FX
    pub dwYMemoryLength: u32,         // Length in DSP WORDS, of Y memory area for this FX
    pub dwFlags: u32,                 // FX bitfield for various flags. See xgpimage documentation
    pub dwInMixbinPtrs: [u32; 2],     // XRAM offsets in DSP WORDS, of input mixbins
    pub dwOutMixbinPtrs: [u32; 35],   // XRAM offsets in DSP WORDS, of output mixbins
}

pub type LPGraphI3DL2_FX0_I3DL2Reverb_STATE = *mut GraphI3DL2_FX0_I3DL2Reverb_STATE;
pub type LPCGraphI3DL2_FX0_I3DL2Reverb_STATE = *const GraphI3DL2_FX0_I3DL2Reverb_STATE;

#[repr(C)]
pub struct GraphXTalk_FX0_XTalk_STATE {
    pub dwScratchOffset: u32,         // Offset in bytes, of scratch area for this FX
    pub dwScratchLength: u32,         // Length in DWORDS, of scratch area for this FX
    pub dwYMemoryOffset: u32,         // Offset in DSP WORDS, of Y memory area for this FX
    pub dwYMemoryLength: u32,         // Length in DSP WORDS, of Y memory area for this FX
    pub dwFlags: u32,                 // FX bitfield for various flags. See xgpimage documentation
    pub dwInMixbinPtrs: [u32; 4],     // XRAM offsets in DSP WORDS, of input mixbins
    pub dwOutMixbinPtrs: [u32; 4],    // XRAM offsets in DSP WORDS, of output mixbins
}

pub type LPGraphXTalk_FX0_XTalk_STATE = *mut GraphXTalk_FX0_XTalk_STATE;
pub type LPCGraphXTalk_FX0_XTalk_STATE = *const GraphXTalk_FX0_XTalk_STATE;

#[repr(C)]
pub struct GraphVoice_FX0_Voice_0_STATE {
    pub dwScratchOffset: u32,         // Offset in bytes, of scratch area for this FX
    pub dwScratchLength: u32,         // Length in DWORDS, of scratch area for this FX
    pub dwYMemoryOffset: u32,         // Offset in DSP WORDS, of Y memory area for this FX
    pub dwYMemoryLength: u32,         // Length in DSP WORDS, of Y memory area for this FX
    pub dwFlags: u32,                 // FX bitfield for various flags. See xgpimage documentation
    pub dwInMixbinPtrs: [u32; 1],     // XRAM offsets in DSP WORDS, of input mixbins
    pub dwOutMixbinPtrs: [u32; 1],    // XRAM offsets in DSP WORDS, of output mixbins
}

pub type LPGraphVoice_FX0_Voice_0_STATE = *mut GraphVoice_FX0_Voice_0_STATE;
pub type LPCGraphVoice_FX0_Voice_0_STATE = *const GraphVoice_FX0_Voice_0_STATE;

#[repr(C)]
pub struct GraphVoice_FX1_Voice_1_STATE {
    pub dwScratchOffset: u32,         // Offset in bytes, of scratch area for this FX
    pub dwScratchLength: u32,         // Length in DWORDS, of scratch area for this FX
    pub dwYMemoryOffset: u32,         // Offset in DSP WORDS, of Y memory area for this FX
    pub dwYMemoryLength: u32,         // Length in DSP WORDS, of Y memory area for this FX
    pub dwFlags: u32,                 // FX bitfield for various flags. See xgpimage documentation
    pub dwInMixbinPtrs: [u32; 1],     // XRAM offsets in DSP WORDS, of input mixbins
    pub dwOutMixbinPtrs: [u32; 1],    // XRAM offsets in DSP WORDS, of output mixbins
}

pub type LPGraphVoice_FX1_Voice_1_STATE = *mut GraphVoice_FX1_Voice_1_STATE;
pub type LPCGraphVoice_FX1_Voice_1_STATE = *const GraphVoice_FX1_Voice_1_STATE;

#[repr(C)]
pub struct GraphVoice_FX2_Voice_2_STATE {
    pub dwScratchOffset: u32,         // Offset in bytes, of scratch area for this FX
    pub dwScratchLength: u32,         // Length in DWORDS, of scratch area for this FX
    pub dwYMemoryOffset: u32,         // Offset in DSP WORDS, of Y memory area for this FX
    pub dwYMemoryLength: u32,         // Length in DSP WORDS, of Y memory area for this FX
    pub dwFlags: u32,                 // FX bitfield for various flags. See xgpimage documentation
    pub dwInMixbinPtrs: [u32; 1],     // XRAM offsets in DSP WORDS, of input mixbins
    pub dwOutMixbinPtrs: [u32; 1],    // XRAM offsets in DSP WORDS, of output mixbins
}

pub type LPGraphVoice_FX2_Voice_2_STATE = *mut GraphVoice_FX2_Voice_2_STATE;
pub type LPCGraphVoice_FX2_Voice_2_STATE = *const GraphVoice_FX2_Voice_2_STATE;

#[repr(C)]
pub struct GraphVoice_FX3_Voice_3_STATE {
    pub dwScratchOffset: u32,         // Offset in bytes, of scratch area for this FX
    pub dwScratchLength: u32,         // Length in DWORDS, of scratch area for this FX
    pub dwYMemoryOffset: u32,         // Offset in DSP WORDS, of Y memory area for this FX
    pub dwYMemoryLength: u32,         // Length in DSP WORDS, of Y memory area for this FX
    pub dwFlags: u32,                 // FX bitfield for various flags. See xgpimage documentation
    pub dwInMixbinPtrs: [u32; 1],     // XRAM offsets in DSP WORDS, of input mixbins
    pub dwOutMixbinPtrs: [u32; 1],    // XRAM offsets in DSP WORDS, of output mixbins
}

pub type LPGraphVoice_FX3_Voice_3_STATE = *mut GraphVoice_FX3_Voice_3_STATE;
pub type LPCGraphVoice_FX3_Voice_3_STATE = *const GraphVoice_FX3_Voice_3_STATE;
