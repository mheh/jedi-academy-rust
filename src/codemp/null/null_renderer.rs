/* Null renderer functions */

#[no_mangle]
pub extern "C" fn RB_StageIteratorGeneric() {}

#[no_mangle]
pub extern "C" fn RB_StageIteratorSky() {}

#[no_mangle]
pub extern "C" fn RB_StageIteratorVertexLitTexture() {}

#[no_mangle]
pub extern "C" fn RB_StageIteratorLightmappedMultitexture() {}

#[no_mangle]
pub extern "C" fn R_SyncRenderThread() {}
