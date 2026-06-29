#![allow(non_snake_case)]

use core::ffi::c_int;

// Forward declaration - CWorldEffectsSystem
// (Full definition in this module or imported from elsewhere)

// vec3_t is typically [f32; 3] from math types
pub type vec3_t = [f32; 3];

pub const PARTICLE_FLAG_RENDER: u32 = 0x00000001;

#[repr(C)]
pub struct SParticle {
    pub pos: vec3_t,
    pub velocity: vec3_t,
    pub flags: u32,
}

#[repr(C)]
pub struct CWorldEffect {
    pub mNext: *mut CWorldEffect,
    pub mSlave: *mut CWorldEffect,
    pub mOwner: *mut CWorldEffect,
    pub mEnabled: bool,
    pub mIsSlave: bool,
}

impl CWorldEffect {
    pub const WORLDEFFECT_ENABLED: c_int = 0;
    pub const WORLDEFFECT_PARTICLES: c_int = 1;
    pub const WORLDEFFECT_PARTICLE_COUNT: c_int = 2;
    pub const WORLDEFFECT_END: c_int = 3;

    // CWorldEffect(CWorldEffect *owner = 0);
    // Constructor - declaration only (implementation elsewhere)

    // virtual ~CWorldEffect(void);
    // Destructor - declaration only (implementation elsewhere)

    pub fn SetNext(&mut self, next: *mut CWorldEffect) {
        self.mNext = next;
    }

    pub fn GetNext(&self) -> *mut CWorldEffect {
        self.mNext
    }

    pub fn SetSlave(&mut self, slave: *mut CWorldEffect) {
        self.mSlave = slave;
    }

    pub fn GetSlave(&self) -> *mut CWorldEffect {
        self.mSlave
    }

    pub fn AddSlave(&mut self, slave: *mut CWorldEffect) {
        // virtual void AddSlave(CWorldEffect *slave);
        // Pure virtual method - no implementation in base class
    }

    pub fn SetIsSlave(&mut self, isSlave: bool) {
        self.mIsSlave = isSlave;
    }

    pub fn SetOwner(&mut self, owner: *mut CWorldEffect) {
        self.mOwner = owner;
    }

    pub fn Command(&mut self, command: *const i8) -> bool {
        // virtual bool Command(const char *command);
        // Pure virtual method - no implementation in base class
        false
    }

    pub fn ParmUpdate_System(&mut self, system: *mut CWorldEffectsSystem, which: c_int) {
        // virtual void ParmUpdate(CWorldEffectsSystem *system, int which);
        // Pure virtual method - no implementation in base class
    }

    pub fn ParmUpdate_Effect(&mut self, effect: *mut CWorldEffect, which: c_int) {
        // virtual void ParmUpdate(CWorldEffect *effect, int which);
        // Pure virtual method - no implementation in base class
    }

    pub fn SetVariable_Bool(&mut self, which: c_int, newValue: bool, doSlave: bool) {
        // virtual void SetVariable(int which, bool newValue, bool doSlave = false);
        // Pure virtual method - no implementation in base class
    }

    pub fn SetVariable_Float(&mut self, which: c_int, newValue: f32, doSlave: bool) {
        // virtual void SetVariable(int which, float newValue, bool doSlave = false);
        // Pure virtual method - no implementation in base class
    }

    pub fn SetVariable_Int(&mut self, which: c_int, newValue: c_int, doSlave: bool) {
        // virtual void SetVariable(int which, int newValue, bool doSlave = false);
        // Pure virtual method - no implementation in base class
    }

    pub fn SetVariable_Vec3(&mut self, which: c_int, newValue: vec3_t, doSlave: bool) {
        // virtual void SetVariable(int which, vec3_t newValue, bool doSlave = false);
        // Pure virtual method - no implementation in base class
    }

    pub fn GetIntVariable(&self, which: c_int) -> c_int {
        // virtual int GetIntVariable(int which) { return 0; }
        0
    }

    pub fn GetParticleVariable(&self, which: c_int) -> *mut SParticle {
        // virtual SParticle *GetParticleVariable(int which) { return 0; }
        std::ptr::null_mut()
    }

    pub fn Update(&mut self, system: *mut CWorldEffectsSystem, elapseTime: f32) {
        // virtual void Update(CWorldEffectsSystem *system, float elapseTime);
        // Pure virtual method - no implementation in base class
    }

    pub fn Render(&mut self, system: *mut CWorldEffectsSystem) {
        // virtual void Render(CWorldEffectsSystem *system);
        // Pure virtual method - no implementation in base class
    }
}

#[repr(C)]
pub struct CWorldEffectsSystem {
    pub mList: *mut CWorldEffect,
    pub mLast: *mut CWorldEffect,
}

impl CWorldEffectsSystem {
    // CWorldEffectsSystem(void);
    // Constructor - declaration only (implementation elsewhere)

    // virtual ~CWorldEffectsSystem(void);
    // Destructor - declaration only (implementation elsewhere)

    pub fn AddWorldEffect(&mut self, effect: *mut CWorldEffect) {
        // void AddWorldEffect(CWorldEffect *effect);
        // Method implementation - defined in implementation file
    }

    pub fn GetIntVariable(&self, which: c_int) -> c_int {
        // virtual int GetIntVariable(int which) { return 0; }
        0
    }

    pub fn GetParticleVariable(&self, which: c_int) -> *mut SParticle {
        // virtual SParticle *GetParticleVariable(int which) { return 0; }
        std::ptr::null_mut()
    }

    pub fn GetFloatVariable(&self, which: c_int) -> f32 {
        // virtual float GetFloatVariable(int which) { return 0.0; }
        0.0
    }

    pub fn GetVecVariable(&self, which: c_int) -> *mut f32 {
        // virtual float *GetVecVariable(int which) { return 0; }
        std::ptr::null_mut()
    }

    pub fn Command(&mut self, command: *const i8) -> bool {
        // virtual bool Command(const char *command);
        // Pure virtual method - no implementation in base class
        false
    }

    pub fn Update(&mut self, elapseTime: f32) {
        // virtual void Update(float elapseTime);
        // Pure virtual method - no implementation in base class
    }

    pub fn ParmUpdate(&mut self, which: c_int) {
        // virtual void ParmUpdate(int which);
        // Pure virtual method - no implementation in base class
    }

    pub fn Render(&mut self) {
        // virtual void Render(void);
        // Pure virtual method - no implementation in base class
    }
}

extern "C" {
    pub fn R_InitWorldEffects();
    pub fn R_ShutdownWorldEffects();
    pub fn RB_RenderWorldEffects();

    pub fn R_WorldEffectCommand(command: *const i8);
    pub fn R_WorldEffect_f();

    pub fn R_GetWindVector(windVector: *mut vec3_t) -> bool;
    pub fn R_GetWindSpeed(windSpeed: *mut f32) -> bool;

    pub fn R_IsRaining() -> bool;
    //pub fn R_IsSnowing() -> bool;
    pub fn R_IsPuffing() -> bool;
    pub fn R_AddWeatherZone(mins: vec3_t, maxs: vec3_t);
}
