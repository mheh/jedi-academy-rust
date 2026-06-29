// leave this line at the top for all g_xxxx.cpp files...
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

// Forward declarations of external types and functions
extern "C" {
    fn InitMover(ent: *mut gentity_t);
    fn G_TestEntityPosition(ent: *mut gentity_t) -> *mut gentity_t;
    fn VectorCopy(in_: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorCompare(v1: *const [f32; 3], v2: *const [f32; 3]) -> c_int;
    fn G_UseTargets(self_: *mut gentity_t, activator: *mut gentity_t);
    fn G_UseTargets2(self_: *mut gentity_t, activator: *mut gentity_t, target: *const c_char);
    fn G_ActivateBehavior(self_: *mut gentity_t, bset: c_int);
    fn G_SpawnInt(key: *const c_char, defaultvalue: *const c_char, out: *mut c_int);
    fn GEntity_UseFunc(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Global objects
    static mut gi: gameImport_t;
    static mut level: level_locals_t;
    static mut g_entities: [gentity_t; 2048];
    static mut vec3_origin: [f32; 3];
}

// Stub constants
const CONTENTS_BODY: c_int = 1;
const EF_SHADER_ANIM: c_int = 1;
const EF_NODRAW: c_int = 1;
const EF_ANIM_ONCE: c_int = 1;
const EF_ANIM_ALLFAST: c_int = 1;
const EF_FORCE_VISIBLE: c_int = 1;
const SVF_NOCLIENT: c_int = 1;
const SVF_PLAYER_USABLE: c_int = 1;
const SVF_BROADCAST: c_int = 1;
const FRAMETIME: c_int = 100;
const BSET_USE: c_int = 0;

// Function pointers / enum values for use/think/die/pain function assignments
const useF_func_usable_use: c_int = 0;
const useF_NULL: c_int = 0;
const thinkF_func_usable_think: c_int = 0;
const thinkF_func_wait_return_solid: c_int = 0;
const thinkF_NULL: c_int = 0;
const painF_func_usable_pain: c_int = 0;
const dieF_func_usable_die: c_int = 0;

const qtrue: c_int = 1;
const qfalse: c_int = 0;

// Placeholder types for external dependencies
#[repr(C)]
pub struct gentity_t {
    pub s: entityState_t,
    pub model: *const c_char,
    pub classname: *const c_char,
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
    pub pos1: [f32; 3],
    pub activator: *mut gentity_t,
    pub target: *const c_char,
    pub target2: *const c_char,
    pub targetname: *const c_char,
    pub paintarget: *const c_char,
    pub svFlags: c_int,
    pub contents: c_int,
    pub clipmask: c_int,
    pub spawnflags: c_int,
    pub spawnContents: c_int,
    pub nextthink: c_int,
    pub e_ThinkFunc: c_int,
    pub e_UseFunc: c_int,
    pub e_DieFunc: c_int,
    pub e_PainFunc: c_int,
    pub wait: f32,
    pub count: c_int,
    pub takedamage: c_int,
    pub health: c_int,
    pub endFrame: c_int,
    pub startFrame: c_int,
    pub NPC: *mut (),
}

#[repr(C)]
pub struct entityState_t {
    pub eFlags: c_int,
    pub origin: [f32; 3],
    pub pos: trBase_t,
    pub solid: c_int,
    pub frame: c_int,
}

#[repr(C)]
pub struct trBase_t {
    pub trBase: [f32; 3],
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
}

#[repr(C)]
pub struct gameImport_t {
    pub Printf: extern "C" fn(*const c_char, ...) -> (),
    pub SetBrushModel: extern "C" fn(*mut gentity_t, *const c_char) -> (),
    pub linkentity: extern "C" fn(*mut gentity_t) -> (),
    pub AdjustAreaPortalState: extern "C" fn(*mut gentity_t, c_int) -> (),
}

#[allow(non_snake_case)]
pub fn func_wait_return_solid(self_: *mut gentity_t) {
    unsafe {
        //once a frame, see if it's clear.
        (*self_).clipmask = CONTENTS_BODY; //|CONTENTS_MONSTERCLIP|CONTENTS_BOTCLIP;
        if ((*self_).spawnflags & 16) == 0 || G_TestEntityPosition(self_).is_null() {
            (gi.SetBrushModel)(self_, (*self_).model);
            VectorCopy(&(*self_).currentOrigin as *const _, &mut (*self_).pos1 as *mut _);
            InitMover(self_);
            /*
            VectorCopy( self->s.origin, self->s.pos.trBase );
            VectorCopy( self->s.origin, self->currentOrigin );
            */
            //if we moved, we want the *current* origin, not our start origin!
            VectorCopy(&(*self_).currentOrigin as *const _, &mut (*self_).s.pos.trBase as *mut _);
            (gi.linkentity)(self_);
            (*self_).svFlags &= !SVF_NOCLIENT;
            (*self_).s.eFlags &= !EF_NODRAW;
            (*self_).e_UseFunc = useF_func_usable_use;
            (*self_).clipmask = 0;
            if !(*self_).target2.is_null() && *(*self_).target2 != 0 {
                G_UseTargets2(self_, (*self_).activator, (*self_).target2);
            }
            if ((*self_).s.eFlags & EF_ANIM_ONCE) != 0 {
                //Start our anim
                (*self_).s.frame = 0;
            }
            //NOTE: be sure to reset the brushmodel before doing this or else CONTENTS_OPAQUE may not be on when you call this
            if ((*self_).spawnflags & 1) == 0 {
                //START_OFF doesn't effect area portals
                (gi.AdjustAreaPortalState)(self_, qfalse);
            }
        } else {
            (*self_).clipmask = 0;
            (*self_).e_ThinkFunc = thinkF_func_wait_return_solid;
            (*self_).nextthink = level.time + FRAMETIME;
        }
    }
}

#[allow(non_snake_case)]
pub fn func_usable_think(self_: *mut gentity_t) {
    unsafe {
        if ((*self_).spawnflags & 8) != 0 {
            (*self_).svFlags |= SVF_PLAYER_USABLE; //Replace the usable flag
            (*self_).e_UseFunc = useF_func_usable_use;
            (*self_).e_ThinkFunc = thinkF_NULL;
        }
    }
}

#[allow(non_snake_case)]
pub fn G_EntIsRemovableUsable(entNum: c_int) -> c_int {
    unsafe {
        let ent: *mut gentity_t = &mut g_entities[entNum as usize];
        if !(*ent).classname.is_null() && Q_stricmp(b"func_usable\0".as_ptr() as *const c_char, (*ent).classname) == 0 {
            if ((*ent).s.eFlags & EF_SHADER_ANIM) == 0 && ((*ent).spawnflags & 8) == 0 && !(*ent).targetname.is_null() {
                //not just a shader-animator and not ALWAYS_ON, so it must be removable somehow
                return qtrue;
            }
        }
        return qfalse;
    }
}

#[allow(non_snake_case)]
pub fn func_usable_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    //Toggle on and off
    unsafe {
        if other == activator {
            //directly used by use button trace
            if ((*self_).spawnflags & 32) != 0 {
                //only usable by NPCs
                if (*activator).NPC.is_null() {
                    //Not an NPC
                    return;
                }
            }
        }

        G_ActivateBehavior(self_, BSET_USE);
        if ((*self_).s.eFlags & EF_SHADER_ANIM) != 0 {
            //animate shader when used
            (*self_).s.frame += 1; //inc frame
            if (*self_).s.frame > (*self_).endFrame {
                //wrap around
                (*self_).s.frame = 0;
            }
            if !(*self_).target.is_null() && *(*self_).target != 0 {
                G_UseTargets(self_, activator);
            }
        } else if ((*self_).spawnflags & 8) != 0 {
            //ALWAYS_ON
            //Remove the ability to use the entity directly
            (*self_).svFlags &= !SVF_PLAYER_USABLE;
            //also remove ability to call any use func at all!
            (*self_).e_UseFunc = useF_NULL;

            if !(*self_).target.is_null() && *(*self_).target != 0 {
                G_UseTargets(self_, activator);
            }

            if (*self_).wait != 0.0 {
                (*self_).e_ThinkFunc = thinkF_func_usable_think;
                (*self_).nextthink = level.time + (((*self_).wait * 1000.0) as c_int);
            }

            return;
        } else if (*self_).count == 0 {
            //become solid again
            (*self_).count = 1;
            (*self_).activator = activator;
            func_wait_return_solid(self_);
        } else {
            //NOTE: MUST do this BEFORE clearing contents, or you may not open the area portal!!!
            if ((*self_).spawnflags & 1) == 0 {
                //START_OFF doesn't effect area portals
                (gi.AdjustAreaPortalState)(self_, qtrue);
            }
            (*self_).s.solid = 0;
            (*self_).contents = 0;
            (*self_).clipmask = 0;
            (*self_).svFlags |= SVF_NOCLIENT;
            (*self_).s.eFlags |= EF_NODRAW;
            (*self_).count = 0;

            if !(*self_).target.is_null() && *(*self_).target != 0 {
                G_UseTargets(self_, activator);
            }
            (*self_).e_ThinkFunc = thinkF_NULL;
            (*self_).nextthink = -1;
        }
    }
}

#[allow(non_snake_case)]
pub fn func_usable_pain(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    point: *const [f32; 3],
    damage: c_int,
    mod_: c_int,
    hitLoc: c_int,
) {
    unsafe {
        if !(*self_).paintarget.is_null() {
            G_UseTargets2(self_, (*self_).activator, (*self_).paintarget);
        } else {
            GEntity_UseFunc(self_, attacker, attacker);
        }
    }
}

#[allow(non_snake_case)]
pub fn func_usable_die(
    self_: *mut gentity_t,
    inflictor: *mut gentity_t,
    attacker: *mut gentity_t,
    damage: c_int,
    mod_: c_int,
    dFlags: c_int,
    hitLoc: c_int,
) {
    unsafe {
        (*self_).takedamage = qfalse;
        GEntity_UseFunc(self_, inflictor, attacker);
    }
}

/*QUAKED func_usable (0 .5 .8) ? STARTOFF AUTOANIMATE ANIM_ONCE ALWAYS_ON BLOCKCHECK NPC_USE PLAYER_USE INACTIVE
START_OFF - the wall will not be there
AUTOANIMATE - if a model is used it will animate
ANIM_ONCE - When turned on, goes through anim once
ALWAYS_ON - Doesn't toggle on and off when used, just runs usescript and fires target
NPC_ONLY - Only NPCs can directly use this
PLAYER_USE - Player can use it with the use button
BLOCKCHECK - Will not turn on while something is inside it

A bmodel that just sits there, doing nothing.  Can be used for conditional walls and models.
"targetname" - When used, will toggle on and off
"target"	Will fire this target every time it is toggled OFF
"target2"	Will fire this target every time it is toggled ON
"model2"	.md3 model to also draw
"modelAngles" md3 model's angles <pitch yaw roll> (in addition to any rotation on the part of the brush entity itself)
"color"		constantLight color
"light"		constantLight radius
"usescript" script to run when turned on
"deathscript"  script to run when turned off
"wait"		amount of time before the object is usable again (only valid with ALWAYS_ON flag)
"health"	if it has health, it will be used whenever shot at/killed - if you want it to only be used once this way, set health to 1
"endframe"	Will make it animate to next shader frame when used, not turn on/off... set this to number of frames in the shader, minus 1
"forcevisible" - When you turn on force sight (any level), you can see these draw through the entire level...
*/

#[allow(non_snake_case)]
pub fn SP_func_usable(self_: *mut gentity_t) {
    unsafe {
        (gi.SetBrushModel)(self_, (*self_).model);
        InitMover(self_);
        VectorCopy(&(*self_).s.origin as *const _, &mut (*self_).s.pos.trBase as *mut _);
        VectorCopy(&(*self_).s.origin as *const _, &mut (*self_).currentOrigin as *mut _);
        VectorCopy(&(*self_).s.origin as *const _, &mut (*self_).pos1 as *mut _);

        (*self_).count = 1;
        if ((*self_).spawnflags & 1) != 0 {
            (*self_).spawnContents = (*self_).contents; // It Navs can temporarly turn it "on"
            (*self_).s.solid = 0;
            (*self_).contents = 0;
            (*self_).clipmask = 0;
            (*self_).svFlags |= SVF_NOCLIENT;
            (*self_).s.eFlags |= EF_NODRAW;
            (*self_).count = 0;
        }

        if ((*self_).spawnflags & 2) != 0 {
            (*self_).s.eFlags |= EF_ANIM_ALLFAST;
        }

        if ((*self_).spawnflags & 4) != 0 {
            //FIXME: need to be able to do change to something when it's done?  Or not be usable until it's done?
            (*self_).s.eFlags |= EF_ANIM_ONCE;
        }

        (*self_).e_UseFunc = useF_func_usable_use;

        if (*self_).health != 0 {
            (*self_).takedamage = qtrue;
            (*self_).e_DieFunc = dieF_func_usable_die;
            (*self_).e_PainFunc = painF_func_usable_pain;
        }

        if (*self_).endFrame > 0 {
            (*self_).s.frame = 0;
            (*self_).startFrame = 0;
            (*self_).s.eFlags |= EF_SHADER_ANIM;
        }

        (gi.linkentity)(self_);

        let mut forceVisible: c_int = 0;
        G_SpawnInt(b"forcevisible\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut forceVisible);
        if forceVisible != 0 {
            //can see these through walls with force sight, so must be broadcast
            if VectorCompare(&(*self_).s.origin as *const _, &vec3_origin as *const _) != 0 {
                //no origin brush
                (*self_).svFlags |= SVF_BROADCAST;
            }
            (*self_).s.eFlags |= EF_FORCE_VISIBLE;
        }
    }
}
