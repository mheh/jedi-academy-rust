#![allow(non_snake_case)]

use core::ffi::c_char;

// maximum random choices
pub const MAX_RANDOM_CHOICES: i32 = 100;

pub type rmIntVector_t = Vec<i32>;

// Forward declarations for external types
pub struct rmObjectiveList_t;
pub struct rmInstanceList_t;
pub struct CRMInstanceFile;
pub struct CRMObjective;
pub struct CRandomTerrain;
pub struct CGPGroup;
pub struct CRMInstance;
pub struct CRMAreaManager;
pub struct CRMPathManager;

// Type aliases from C
pub type vec3_t = [f32; 3];
pub type qboolean = i32;
pub type symmetry_t = i32;

#[repr(C)]
pub struct CRMMission {
    mObjectives: *mut rmObjectiveList_t,
    mInstances: *mut rmInstanceList_t,

    mInstanceFile: *mut CRMInstanceFile,
    mCurrentObjective: *mut CRMObjective,

    mValidNodes: bool,
    mValidPaths: bool,
    mValidRivers: bool,
    mValidWeapons: bool,
    mValidAmmo: bool,
    mValidObjectives: bool,
    mValidInstances: bool,

    mTimeLimit: i32,
    mMaxInstancePosition: i32,

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
    mBackUpPath: i32,

    mDefaultPadding: i32,

    mAreaManager: *mut CRMAreaManager,

    mPathManager: *mut CRMPathManager,

    mLandScape: *mut CRandomTerrain,
}

impl CRMMission {
    pub fn new(landscape: *mut CRandomTerrain) -> Self {
        unimplemented!()
    }

    pub fn Load(&mut self, name: *const c_char, instances: *const c_char, difficulty: *const c_char) -> bool {
        unimplemented!()
    }

    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unimplemented!()
    }

    pub fn Preview(&self, from: *const vec3_t) {
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

    pub fn CompleteObjective(&mut self, objective: *mut CRMObjective) {
        unimplemented!()
    }

    pub fn GetTimeLimit(&self) -> i32 {
        self.mTimeLimit
    }

    pub fn GetMaxInstancePosition(&self) -> i32 {
        self.mMaxInstancePosition
    }

    pub fn GetDescription(&self) -> *const c_char {
        self.mDescription.as_ptr() as *const c_char
    }

    pub fn GetExitScreen(&self) -> *const c_char {
        self.mExitScreen.as_ptr() as *const c_char
    }

    pub fn GetSymmetric(&self) -> i32 {
        self.mSymmetric
    }

    pub fn GetBackUpPath(&self) -> i32 {
        self.mBackUpPath
    }

    pub fn GetDefaultPadding(&self) -> i32 {
        self.mDefaultPadding
    }

    pub fn DenyPickupHealth(&self) -> bool {
        unsafe { (*self.mLandScape).flrand(0.0f32, 1.0f32) > self.mPickupHealth }
    }

    pub fn DenyPickupArmor(&self) -> bool {
        unsafe { (*self.mLandScape).flrand(0.0f32, 1.0f32) > self.mPickupArmor }
    }

    pub fn DenyPickupAmmo(&self) -> bool {
        unsafe { (*self.mLandScape).flrand(0.0f32, 1.0f32) > self.mPickupAmmo }
    }

    pub fn DenyPickupWeapon(&self) -> bool {
        unsafe { (*self.mLandScape).flrand(0.0f32, 1.0f32) > self.mPickupWeapon }
    }

    pub fn DenyPickupEquipment(&self) -> bool {
        unsafe { (*self.mLandScape).flrand(0.0f32, 1.0f32) > self.mPickupEquipment }
    }

    //	void			CreateMap				( void );

    fn MirrorPos(&mut self, pos: *mut vec3_t) {
        unimplemented!()
    }

    fn ParseRandom(&mut self, random: *mut CGPGroup) -> *mut CGPGroup {
        unimplemented!()
    }

    fn ParseOrigin(&mut self, originGroup: *mut CGPGroup, origin: *mut vec3_t, lookat: *mut vec3_t, flattenHeight: *mut i32) -> bool {
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

    fn PlaceWallInstance(&mut self, instance: *mut CRMInstance, xpos: f32, ypos: f32, zpos: f32, x: i32, y: i32, angle: f32) {
        unimplemented!()
    }

    //	void			PurgeUnlinkedTriggers	( );
    //	void			PurgeTrigger			( CEntity* trigger );

    fn ParseDifficulty(&mut self, difficulty: *mut CGPGroup) -> bool {
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

    fn ParseWallRect(&mut self, group: *mut CGPGroup, side: i32) -> bool {
        unimplemented!()
    }

    //	void			SpawnNPCTriggers		( CCMLandScape* landscape );
    //	void			AttachNPCTriggers		( CCMLandScape* landscape );
}
