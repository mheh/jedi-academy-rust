#![allow(non_snake_case)]

// ...including RM_Instance_Void.h (DEBUG_LINKING)

// Stub for unported base class
pub struct CRMInstance;

// Stubs for unported dependencies
pub struct CGPGroup;
pub struct CRMInstanceFile;
pub struct CRMAreaManager;
pub struct CRMArea;

/// class CRMVoidInstance : public CRMInstance
#[repr(C)]
pub struct CRMVoidInstance {
    // Inherits from CRMInstance
}

impl CRMVoidInstance {
    /// CRMVoidInstance ( CGPGroup* instGroup, CRMInstanceFile& instFile );
    pub fn new(instGroup: *mut CGPGroup, instFile: *mut CRMInstanceFile) -> Self {
        CRMVoidInstance {}
    }

    /// virtual void SetArea ( CRMAreaManager* amanager, CRMArea* area );
    pub unsafe fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
    }
}
