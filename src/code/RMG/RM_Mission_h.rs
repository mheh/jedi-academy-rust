#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// maximum random choices
const MAX_RANDOM_CHOICES: usize = 100;

pub type rmIntVector_t = Vec<c_int>;

// Stub types for unported dependencies - defined in other modules
#[repr(C)]
pub struct rmObjectiveList_t;

#[repr(C)]
pub struct rmInstanceList_t;

#[repr(C)]
pub struct CRMInstanceFile;

#[repr(C)]
pub struct CRMObjective;

#[repr(C)]
pub struct CRMAreaManager;

#[repr(C)]
pub struct CRMPathManager;

#[repr(C)]
pub struct CRandomTerrain;

#[repr(C)]
pub struct CGPGroup;

#[repr(C)]
pub struct CRMInstance;

pub type vec3_t = [f32; 3];
pub type symmetry_t = c_int;
pub type qboolean = c_int;

pub struct CRMMission {
    // private members
    mObjectives: rmObjectiveList_t,
    mInstances: rmInstanceList_t,

    mInstanceFile: CRMInstanceFile,
    mCurrentObjective: *mut CRMObjective,

    mValidNodes: bool,
    mValidPaths: bool,
    mValidRivers: bool,
    mValidWeapons: bool,
    mValidAmmo: bool,
    mValidObjectives: bool,
    mValidInstances: bool,

    mTimeLimit: c_int,
    mMaxInstancePosition: c_int,

    // npc multipliers
    mAccuracyMultiplier: f32,
    mHealthMultiplier: f32,

    // % chance that RMG pickup is actually spawned
    mPickupHealth: f32,
    mPickupArmor: f32,
    mPickupAmmo: f32,
    mPickupWeapon: f32,
    mPickupEquipment: f32,

    mDescription: String,
    mExitScreen: String,
    mTimeExpiredScreen: String,

    // symmetric landscape style
    mSymmetric: symmetry_t,

    // if set to 1 in the mission file, adds an extra connecting path in symmetric maps
    // to ensure both sides actually do connect
    mBackUpPath: c_int,

    mDefaultPadding: c_int,

    mAreaManager: *mut CRMAreaManager,

    mPathManager: *mut CRMPathManager,

    mLandScape: *mut CRandomTerrain,
}

impl CRMMission {
    pub unsafe fn new(terrain: *mut CRandomTerrain) -> Self {
        unimplemented!()
    }

    pub unsafe fn drop(&mut self) {
        unimplemented!()
    }

    pub fn Load(
        &mut self,
        name: *const c_char,
        instances: *const c_char,
        difficulty: *const c_char,
    ) -> bool {
        unimplemented!()
    }

    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unimplemented!()
    }

    pub fn Preview(&self, from: &vec3_t) {
        unimplemented!()
    }

    pub fn FindObjective(&self, name: *const c_char) -> *mut CRMObjective {
        unimplemented!()
    }

    pub fn GetCurrentObjective(&self) -> *mut CRMObjective {
        self.mCurrentObjective
    }

    pub fn CompleteMission(&mut self) {
        unimplemented!()
    }

    pub fn FailedMission(&mut self, TimeExpired: bool) {
        unimplemented!()
    }

    pub fn CompleteObjective(&mut self, ojective: *mut CRMObjective) {
        unimplemented!()
    }

    pub fn GetTimeLimit(&self) -> c_int {
        self.mTimeLimit
    }

    pub fn GetMaxInstancePosition(&self) -> c_int {
        self.mMaxInstancePosition
    }

    pub fn GetDescription(&self) -> *const c_char {
        self.mDescription.as_ptr() as *const c_char
    }

    pub fn GetExitScreen(&self) -> *const c_char {
        self.mExitScreen.as_ptr() as *const c_char
    }

    pub fn GetSymmetric(&self) -> c_int {
        self.mSymmetric
    }

    pub fn GetBackUpPath(&self) -> c_int {
        self.mBackUpPath
    }

    pub fn GetDefaultPadding(&self) -> c_int {
        self.mDefaultPadding
    }

    // private methods

    fn MirrorPos(&mut self, pos: &vec3_t) {
        unimplemented!()
    }

    fn ParseRandom(&mut self, random: *mut CGPGroup) -> *mut CGPGroup {
        unimplemented!()
    }

    fn ParseOrigin(
        &mut self,
        originGroup: *mut CGPGroup,
        origin: *mut vec3_t,
        lookat: *mut vec3_t,
        flattenHeight: *mut c_int,
    ) -> bool {
        unimplemented!()
    }

    fn ParseNodes(&mut self, group: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParsePaths(&mut self, paths: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseRivers(&mut self, rivers: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn PlaceBridges(&mut self) {
        unimplemented!()
    }

    fn PlaceWallInstance(
        &mut self,
        instance: *mut CRMInstance,
        xpos: f32,
        ypos: f32,
        zpos: f32,
        x: c_int,
        y: c_int,
        angle: f32,
    ) {
        unimplemented!()
    }

    fn ParseDifficulty(
        &mut self,
        difficulty: *mut CGPGroup,
        parent: *mut CGPGroup,
    ) -> bool {
        unimplemented!()
    }

    fn ParseWeapons(&mut self, weapons: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseAmmo(&mut self, ammo: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseOutfit(&mut self, outfit: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseObjectives(&mut self, objectives: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseInstance(&mut self, instance: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseInstances(&mut self, instances: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseInstancesOnPath(&mut self, group: *mut CGPGroup) -> bool {
        unimplemented!()
    }

    fn ParseWallRect(&mut self, group: *mut CGPGroup, side: c_int) -> bool {
        unimplemented!()
    }
}
