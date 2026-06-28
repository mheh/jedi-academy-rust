#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// #pragma once
// #if !defined(RM_INSTANCE_H_INC)
// #define RM_INSTANCE_H_INC

// #ifdef DEBUG_LINKING
// #pragma message("...including RM_Instance.h")
// #endif

// #if !defined(CM_LANDSCAPE_H_INC)
// #include "../qcommon/cm_landscape.h"
// #endif

// Forward declarations for types from ../qcommon/cm_landscape.h and elsewhere
pub struct CRMArea;
pub struct CRMObjective;
pub struct CRandomTerrain;
pub struct CRMAreaManager;
pub struct CGPGroup;
pub struct CRMInstanceFile;

// Type aliases
pub type qboolean = c_int;
pub type vec_t = f32;
pub type vec3_t = [f32; 3];
pub type vec3pair_t = [[f32; 3]; 2];

// String type wrapper (approximating C++ std::string for ABI stability)
// In a faithful port, this would need to match the actual C++ string layout,
// but we use an opaque marker type to prevent Rust code from directly manipulating it.
#[repr(C)]
pub struct string {
    // Opaque representation - actual C++ std::string layout varies
    _data: *mut c_char,
    _capacity: usize,
    _size: usize,
}

impl string {
    pub fn c_str(&self) -> *const c_char {
        self._data
    }
}

// Common Quake constant (matching oracle/qcommon/q_shared.h conventions)
pub const MAX_QPATH: usize = 256;

// enum CRMAutomapSymbol
#[repr(C)]
#[derive(Copy, Clone)]
pub enum CRMAutomapSymbol {
    AUTOMAP_NONE = 0,
    AUTOMAP_BLD = 1,
    AUTOMAP_OBJ = 2,
    AUTOMAP_START = 3,
    AUTOMAP_END = 4,
    AUTOMAP_ENEMY = 5,
    AUTOMAP_FRIEND = 6,
    AUTOMAP_WALL = 7,
}

// class CRMInstance
#[repr(C)]
pub struct CRMInstance {
    // filter of entities inside of this
    mFilter: [c_char; MAX_QPATH],
    // team specific filter
    mTeamFilter: [c_char; MAX_QPATH],

    // Bounding box for instance itself
    mBounds: vec3pair_t,

    // Position of the instance
    mArea: *mut CRMArea,

    // Objective associated with this instance
    mObjective: *mut CRMObjective,

    // optional instance specific strings for objective
    // message outputed when objective is completed
    mMessage: string,
    // description of objective
    mDescription: string,
    // more info for objective
    mInfo: string,

    // Radius to space instances with
    mSpacingRadius: f32,
    // Radius to flatten under instances
    mFlattenRadius: f32,

    // Line of spacing radius's, forces locket
    mSpacingLine: c_int,
    // Origin cant move
    mLockOrigin: bool,

    // allow surface sprites under instance?
    mSurfaceSprites: bool,

    // show which symbol on automap 0=none
    mAutomapSymbol: c_int,

    // id of entity spawned
    mEntityID: c_int,
    // blue or red side
    mSide: c_int,
    // mirror origin, angle
    mMirror: c_int,

    // height to flatten land
    mFlattenHeight: c_int,
}

impl CRMInstance {
    // Constructor signature from C++
    pub fn new(instance: *mut CGPGroup, instFile: &mut CRMInstanceFile) -> Self;

    // virtual ~CRMInstance ( ) { }

    // virtual bool		IsValid				( )	{ return true; }
    pub fn IsValid(&self) -> bool {
        true
    }

    // virtual bool		PreSpawn			( CRandomTerrain* terrain, qboolean IsServer );
    pub fn PreSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool;
    // virtual bool		Spawn				( CRandomTerrain* terrain, qboolean IsServer ) { return false; }
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        false
    }
    // virtual bool		PostSpawn			( CRandomTerrain* terrain, qboolean IsServer );
    pub fn PostSpawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool;

    // virtual void		Preview				( const vec3_t from );
    pub fn Preview(&self, from: *const vec3_t);

    // virtual void		SetArea				( CRMAreaManager* amanager, CRMArea* area ) { mArea = area; }
    pub fn SetArea(&mut self, amanager: *mut CRMAreaManager, area: *mut CRMArea) {
        self.mArea = area;
    }
    // virtual void		SetFilter			( const char *filter ) { strcpy(mFilter, filter); }
    pub fn SetFilter(&mut self, filter: *const c_char) {
        unsafe {
            let mut i = 0;
            while i < MAX_QPATH && *filter.add(i) != 0 {
                *self.mFilter.as_mut_ptr().add(i) = *filter.add(i);
                i += 1;
            }
            if i < MAX_QPATH {
                *self.mFilter.as_mut_ptr().add(i) = 0;
            }
        }
    }
    // virtual void		SetTeamFilter		( const char *teamFilter ) { strcpy(mTeamFilter, teamFilter); }
    pub fn SetTeamFilter(&mut self, teamFilter: *const c_char) {
        unsafe {
            let mut i = 0;
            while i < MAX_QPATH && *teamFilter.add(i) != 0 {
                *self.mTeamFilter.as_mut_ptr().add(i) = *teamFilter.add(i);
                i += 1;
            }
            if i < MAX_QPATH {
                *self.mTeamFilter.as_mut_ptr().add(i) = 0;
            }
        }
    }
    // void				SetObjective		( CRMObjective* obj ) { mObjective = obj; }
    pub fn SetObjective(&mut self, obj: *mut CRMObjective) {
        self.mObjective = obj;
    }
    // CRMObjective*		GetObjective		(void) {return mObjective;}
    pub fn GetObjective(&self) -> *mut CRMObjective {
        self.mObjective
    }
    // bool				HasObjective		() {return mObjective != NULL;}
    pub fn HasObjective(&self) -> bool {
        !self.mObjective.is_null()
    }
    // int					GetAutomapSymbol	() {return mAutomapSymbol;}
    pub fn GetAutomapSymbol(&self) -> c_int {
        self.mAutomapSymbol
    }
    // void				DrawAutomapSymbol	();
    pub fn DrawAutomapSymbol(&self);
    // const char*			GetMessage(void)	{ return mMessage.c_str(); }
    pub fn GetMessage(&self) -> *const c_char {
        self.mMessage.c_str()
    }
    // const char*			GetDescription(void){ return mDescription.c_str(); }
    pub fn GetDescription(&self) -> *const c_char {
        self.mDescription.c_str()
    }
    // const char*			GetInfo(void)		{ return mInfo.c_str(); }
    pub fn GetInfo(&self) -> *const c_char {
        self.mInfo.c_str()
    }
    // void				SetMessage(const char* msg) { mMessage = msg; }
    pub fn SetMessage(&mut self, msg: *const c_char);
    // void				SetDescription(const char* desc) { mDescription = desc; }
    pub fn SetDescription(&mut self, desc: *const c_char);
    // void				SetInfo(const char* info) { mInfo = info; }
    pub fn SetInfo(&mut self, info: *const c_char);
    // void				SetSide(int side)	{mSide = side;}
    pub fn SetSide(&mut self, side: c_int) {
        self.mSide = side;
    }
    // int					GetSide				( ) {return mSide;}
    pub fn GetSide(&self) -> c_int {
        self.mSide
    }

    // NOTE: should consider making SetMirror also set all other variables that need flipping
    // like the origin and Side, etc... Otherwise an Instance may have had it's origin flipped
    // but then later will have mMirror set to false, but the origin is still flipped. So any functions
    // that look at the instance later will see mMirror set to false, but not realize the origin has ALREADY been flipped
    // virtual void  		SetMirror(int mirror)	{ mMirror = mirror;}
    pub fn SetMirror(&mut self, mirror: c_int) {
        self.mMirror = mirror;
    }
    // int					GetMirror			( ) { return mMirror;}
    pub fn GetMirror(&self) -> c_int {
        self.mMirror
    }

    // virtual bool		GetSurfaceSprites	( )		{ return mSurfaceSprites; }
    pub fn GetSurfaceSprites(&self) -> bool {
        self.mSurfaceSprites
    }

    // virtual bool		GetLockOrigin		( )		{ return mLockOrigin; }
    pub fn GetLockOrigin(&self) -> bool {
        self.mLockOrigin
    }
    // virtual int			GetSpacingLine		( )		{ return mSpacingLine; }
    pub fn GetSpacingLine(&self) -> c_int {
        self.mSpacingLine
    }

    // virtual int			GetPreviewColor		( )		{ return 0; }
    pub fn GetPreviewColor(&self) -> c_int {
        0
    }
    // virtual float		GetSpacingRadius	( )		{ return mSpacingRadius; }
    pub fn GetSpacingRadius(&self) -> f32 {
        self.mSpacingRadius
    }
    // virtual float		GetFlattenRadius	( )		{ return mFlattenRadius; }
    pub fn GetFlattenRadius(&self) -> f32 {
        self.mFlattenRadius
    }
    // const char			*GetFilter			( )		{ return mFilter; }
    pub fn GetFilter(&self) -> *const c_char {
        self.mFilter.as_ptr() as *const c_char
    }
    // const char			*GetTeamFilter		( )		{ return mTeamFilter; }
    pub fn GetTeamFilter(&self) -> *const c_char {
        self.mTeamFilter.as_ptr() as *const c_char
    }

    // CRMArea&			GetArea				( )		{ return *mArea; }
    pub fn GetArea(&mut self) -> &mut CRMArea {
        unsafe { &mut *self.mArea }
    }
    // vec_t*				GetOrigin			( ) 	{return mArea->GetOrigin(); }
    pub fn GetOrigin(&self) -> *mut vec_t;
    // float				GetAngle			( )		{return mArea->GetAngle();}
    pub fn GetAngle(&self) -> f32;
    // void				SetAngle(float ang )		{ mArea->SetAngle(ang);}
    pub fn SetAngle(&mut self, ang: f32);
    // const vec3pair_t&	GetBounds(void) const		{ return(mBounds); }
    pub fn GetBounds(&self) -> &vec3pair_t {
        &self.mBounds
    }

    // void				SetFlattenHeight	( int height ) { mFlattenHeight = height; }
    pub fn SetFlattenHeight(&mut self, height: c_int) {
        self.mFlattenHeight = height;
    }
    // int					GetFlattenHeight	( void )	   { return mFlattenHeight; }
    pub fn GetFlattenHeight(&self) -> c_int {
        self.mFlattenHeight
    }

    // void				SetSpacingRadius	(float spacing) { mSpacingRadius = spacing; }
    pub fn SetSpacingRadius(&mut self, spacing: f32) {
        self.mSpacingRadius = spacing;
    }
}

// typedef list<CRMInstance*>::iterator	rmInstanceIter_t;
pub type rmInstanceIter_t = std::collections::linked_list::Iter<'static, *mut CRMInstance>;
// typedef list<CRMInstance*>				rmInstanceList_t;
pub type rmInstanceList_t = std::collections::LinkedList<*mut CRMInstance>;

// #endif
