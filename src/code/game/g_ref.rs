// Reference tag utility functions
// leave this line at the top for all g_xxxx.cpp files...

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::collections::HashMap;

// Type definitions
pub type qboolean = c_int;
pub type vec3_t = [f32; 3];

// Constants
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;
const MAX_REFNAME: usize = 32;
const TAG_GENERIC_NAME: &[u8] = b"__WORLD__\0"; //If a designer chooses this name, cut a finger off as an example to the others

const RTF_NAVGOAL: c_int = 0x00000001;
const NODE_NAVGOAL: c_int = 3;

// reference_tag_t structure
#[repr(C)]
pub struct reference_tag_t {
    pub name: [c_char; MAX_REFNAME],
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub flags: c_int,   //Just in case
    pub radius: c_int,  //For nav goals
}

// Stub types needed for declarations
#[repr(C)]
pub struct gentity_s {
    pub currentOrigin: vec3_t,
    pub s: entityState_t,
    pub targetname: *mut c_char,
    pub ownername: *mut c_char,
    pub target: *mut c_char,
    pub e_ThinkFunc: c_int,
    pub nextthink: c_int,
    _dummy: [u8; 0],
}
pub type gentity_t = gentity_s;

#[repr(C)]
pub struct entityState_t {
    pub origin: vec3_t,
    pub angles: vec3_t,
    _dummy: [u8; 0],
}

#[repr(C)]
pub struct level_locals_t {
    pub time: c_int,
    _dummy: [u8; 0],
}

// Partial gameImport_t stub
#[repr(C)]
pub struct gameImport_t {
    pub Printf: Option<unsafe extern "C" fn(*const c_char, ...)>,
    pub inPVS: Option<unsafe extern "C" fn(*const vec3_t, *const vec3_t) -> c_int>,
    _dummy: [u8; 0],
}

// tagOwner structure to hold vectors and maps of tags
pub struct tagOwner_t {
    pub tags: Vec<Box<reference_tag_t>>,
    pub tagMap: HashMap<String, *mut reference_tag_t>,
}

// External declarations
extern "C" {
    pub static mut delayedShutDown: c_int;
    pub static mut level: level_locals_t;
    pub static mut g_entities: [gentity_t; 1024];
    pub static mut gi: gameImport_t;

    pub fn G_Find(from: *mut gentity_t, fieldofs: c_int, match_: *const c_char) -> *mut gentity_t;
    pub fn G_FreeEntity(e: *mut gentity_t);
    pub fn CG_DrawNode(origin: vec3_t, node_type: c_int);
    pub fn VectorCopy(src: *const vec3_t, dst: *mut vec3_t);
    pub fn VectorSubtract(veca: *const vec3_t, vecb: *const vec3_t, out: *mut vec3_t);
    pub fn VectorNormalize(v: *mut vec3_t) -> f32;
    pub fn vectoangles(vec: *const vec3_t, angles: *mut vec3_t);
    pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int, bBarfIfTooLong: qboolean);
    pub fn strlwr(s: *mut c_char) -> *mut c_char;
}

// Global map for storing tag owners
static mut refTagOwnerMap: Option<HashMap<String, Box<tagOwner_t>>> = None;

// Utility functions
#[inline]
fn VALID(a: *const c_void) -> bool {
    !a.is_null()
}

#[inline]
fn VALIDATEP(a: *const c_void) -> *const c_void {
    if a.is_null() {
        assert!(false);
        return core::ptr::null();
    }
    a
}

#[inline]
fn VALIDATEB(a: *const c_void) -> bool {
    if a.is_null() {
        assert!(false);
        return false;
    }
    true
}

#[inline]
fn VALIDSTRING(a: *const c_char) -> bool {
    if a.is_null() {
        return false;
    }
    unsafe { *a != 0 }
}

/*
-------------------------
TAG_ShowTags
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_ShowTags(flags: c_int) {
    if let Some(map) = &refTagOwnerMap {
        for (_owner_name, tag_owner) in map.iter() {
            for tag in tag_owner.tags.iter() {
                if (*tag).flags & RTF_NAVGOAL != 0 {
                    if let Some(inPVS) = gi.inPVS {
                        if inPVS(core::ptr::addr_of!(g_entities[0].currentOrigin), core::ptr::addr_of!((*tag).origin)) != 0 {
                            CG_DrawNode((*tag).origin, NODE_NAVGOAL);
                        }
                    }
                }
            }
        }
    }
}

/*
-------------------------
TAG_Init
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_Init() {
    //Delete all owners
    if let Some(map) = &mut refTagOwnerMap {
        for (_key, tag_owner) in map.iter_mut() {
            //Delete all tags within the owner's scope
            tag_owner.tags.clear(); // Boxes will be freed when dropped

            //Clear the containers
            tag_owner.tagMap.clear();
        }

        //Clear the container
        map.clear();
    }

    // Reinitialize the map
    refTagOwnerMap = Some(HashMap::new());
}

/*
-------------------------
TAG_FindOwner
-------------------------
*/

#[allow(non_snake_case)]
unsafe fn TAG_FindOwner(owner: *const c_char) -> *mut tagOwner_t {
    if owner.is_null() {
        return core::ptr::null_mut();
    }

    let owner_str = match core::ffi::CStr::from_ptr(owner).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return core::ptr::null_mut(),
    };

    if let Some(map) = &mut refTagOwnerMap {
        if let Some(tag_owner) = map.get_mut(&owner_str) {
            return tag_owner.as_mut() as *mut tagOwner_t;
        }
    }

    core::ptr::null_mut()
}

/*
-------------------------
TAG_Find
-------------------------
*/

#[allow(non_snake_case)]
unsafe fn TAG_Find(owner: *const c_char, name: *const c_char) -> *mut reference_tag_t {
    if name.is_null() {
        return core::ptr::null_mut();
    }

    let owner_to_use = if VALIDSTRING(owner) {
        owner
    } else {
        TAG_GENERIC_NAME.as_ptr() as *const c_char
    };

    let mut tag_owner = TAG_FindOwner(owner_to_use);

    //Not found...
    if tag_owner.is_null() {
        tag_owner = TAG_FindOwner(TAG_GENERIC_NAME.as_ptr() as *const c_char);

        if tag_owner.is_null() {
            return core::ptr::null_mut();
        }
    }

    let name_str = match core::ffi::CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return core::ptr::null_mut(),
    };

    if let Some(map) = &refTagOwnerMap {
        // Look in the requested owner's tagMap first
        let owner_str = match core::ffi::CStr::from_ptr(owner_to_use).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return core::ptr::null_mut(),
        };
        if let Some(tag_owner_box) = map.get(&owner_str) {
            if let Some(tag_ptr) = tag_owner_box.tagMap.get(&name_str) {
                return *tag_ptr;
            }
        }
    }

    //Try the generic owner instead
    let generic_name = "__WORLD__".to_string();

    let mut temp_name: [c_char; MAX_REFNAME] = [0; MAX_REFNAME];

    Q_strncpyz(
        core::ptr::addr_of_mut!(temp_name[0]),
        name,
        MAX_REFNAME as c_int,
        qfalse,
    );
    strlwr(core::ptr::addr_of_mut!(temp_name[0])); //NOTENOTE: For case insensitive searches on a map

    let temp_name_str = match core::ffi::CStr::from_ptr(core::ptr::addr_of!(temp_name[0])).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return core::ptr::null_mut(),
    };

    if let Some(map) = &refTagOwnerMap {
        // Look in the generic owner's tagMap
        if let Some(tag_owner_box) = map.get(&generic_name) {
            if let Some(tag_ptr) = tag_owner_box.tagMap.get(&temp_name_str) {
                return *tag_ptr;
            }
        }
    }

    core::ptr::null_mut()
}

/*
-------------------------
TAG_Add
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_Add(
    name: *const c_char,
    owner: *const c_char,
    origin: vec3_t,
    angles: vec3_t,
    radius: c_int,
    flags: c_int,
) -> *mut reference_tag_t {
    let mut tag = Box::new(reference_tag_t {
        name: [0; MAX_REFNAME],
        origin,
        angles,
        radius,
        flags,
    });

    VALIDATEP(tag.as_mut() as *mut _ as *mut c_void);

    if VALIDSTRING(name) == false {
        //gi.Error("Nameless ref_tag found at (%i %i %i)", (int)origin[0], (int)origin[1], (int)origin[2]);
        if let Some(printf) = gi.Printf {
            printf(
                b"^1ERROR: Nameless ref_tag found at (%i %i %i)\n\0".as_ptr() as *const c_char,
                origin[0] as c_int,
                origin[1] as c_int,
                origin[2] as c_int,
            );
        }
        delayedShutDown = level.time + 100;
        return core::ptr::null_mut();
    }

    //Copy the name
    Q_strncpyz(
        core::ptr::addr_of_mut!(tag.name[0]),
        name,
        MAX_REFNAME as c_int,
        qfalse,
    );
    strlwr(core::ptr::addr_of_mut!(tag.name[0])); //NOTENOTE: For case insensitive searches on a map

    //Make sure this tag's name isn't alread in use
    if !TAG_Find(owner, name).is_null() {
        delayedShutDown = level.time + 100;
        if let Some(printf) = gi.Printf {
            printf(
                b"^1ERROR: Duplicate tag name \"%s\"\n\0".as_ptr() as *const c_char,
                name,
            );
        }
        return core::ptr::null_mut();
    }

    //Attempt to add this to the owner's list
    let owner_to_use = if VALIDSTRING(owner) == false {
        //If the owner isn't found, use the generic world name
        TAG_GENERIC_NAME.as_ptr() as *const c_char
    } else {
        owner
    };

    // Get string representation of the name (already lowercased)
    let name_str = match core::ffi::CStr::from_ptr(tag.name.as_ptr()).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return core::ptr::null_mut(),
    };

    let owner_str = match core::ffi::CStr::from_ptr(owner_to_use).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return core::ptr::null_mut(),
    };

    // Convert Box to stable raw pointer (like C++ 'new')
    let tag_ptr: *mut reference_tag_t = Box::leak(tag);

    // Initialize map if needed
    if refTagOwnerMap.is_none() {
        refTagOwnerMap = Some(HashMap::new());
    }

    if let Some(map) = &mut refTagOwnerMap {
        if let Some(tag_owner) = map.get_mut(&owner_str) {
            //If the owner is valid, add this tag to it
            tag_owner.tags.push(Box::from_raw(tag_ptr));
            tag_owner.tagMap.insert(name_str, tag_ptr);
        } else {
            //Create a new owner list
            let mut tag_owner = tagOwner_t {
                tags: Vec::new(),
                tagMap: HashMap::new(),
            };

            //Insert the information
            tag_owner.tags.push(Box::from_raw(tag_ptr));
            tag_owner.tagMap.insert(name_str, tag_ptr);

            //Map it
            map.insert(owner_str, Box::new(tag_owner));
        }
    }

    tag_ptr
}

/*
-------------------------
TAG_GetOrigin
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_GetOrigin(owner: *const c_char, name: *const c_char, origin: *mut vec3_t) -> c_int {
    let tag = TAG_Find(owner, name);

    if tag.is_null() {
        (*origin)[0] = 0.0;
        (*origin)[1] = 0.0;
        (*origin)[2] = 0.0;
        return qfalse;
    }

    VALIDATEB(tag as *const c_void);

    VectorCopy(core::ptr::addr_of!((*tag).origin), origin);

    qtrue
}

/*
-------------------------
TAG_GetOrigin2
Had to get rid of that damn assert for dev
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_GetOrigin2(owner: *const c_char, name: *const c_char, origin: *mut vec3_t) -> c_int {
    let tag = TAG_Find(owner, name);

    if tag.is_null() {
        return qfalse;
    }

    VectorCopy(core::ptr::addr_of!((*tag).origin), origin);

    qtrue
}

/*
-------------------------
TAG_GetAngles
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_GetAngles(owner: *const c_char, name: *const c_char, angles: *mut vec3_t) -> c_int {
    let tag = TAG_Find(owner, name);

    VALIDATEB(tag as *const c_void);

    VectorCopy(core::ptr::addr_of!((*tag).angles), angles);

    qtrue
}

/*
-------------------------
TAG_GetRadius
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_GetRadius(owner: *const c_char, name: *const c_char) -> c_int {
    let tag = TAG_Find(owner, name);

    VALIDATEB(tag as *const c_void);

    (*tag).radius
}

/*
-------------------------
TAG_GetFlags
-------------------------
*/

#[allow(non_snake_case)]
pub unsafe fn TAG_GetFlags(owner: *const c_char, name: *const c_char) -> c_int {
    let tag = TAG_Find(owner, name);

    VALIDATEB(tag as *const c_void);

    (*tag).flags
}

/*
==============================================================================

Spawn functions

==============================================================================
*/

/*QUAKED ref_tag (0.5 0.5 1) (-8 -8 -8) (8 8 8)

Reference tags which can be positioned throughout the level.
These tags can later be refered to by the scripting system
so that their origins and angles can be referred to.

If you set angles on the tag, these will be retained.

If you target a ref_tag at an entity, that will set the ref_tag's
angles toward that entity.

If you set the ref_tag's ownername to the ownername of an entity,
it makes that entity is the owner of the ref_tag.  This means
that the owner, and only the owner, may refer to that tag.

Tags may not have the same name as another tag with the same
owner.  However, tags with different owners may have the same
name as one another.  In this way, scripts can generically
refer to tags by name, and their owners will automatically
specifiy which tag is being referred to.

targetname	- the name of this tag
ownername	- the owner of this tag
target		- use to point the tag at something for angles
*/

const FOFS_targetname: c_int = 0; // Field offset - would need proper definition

#[allow(non_snake_case)]
unsafe fn ref_link(ent: *mut gentity_t) {
    if !(*ent).target.is_null() {
        //TODO: Find the target and set our angles to that direction
        let target = G_Find(core::ptr::null_mut(), FOFS_targetname, (*ent).target);

        if !target.is_null() {
            let mut dir: vec3_t = [0.0; 3];

            //Find the direction to the target
            VectorSubtract(
                core::ptr::addr_of!((*target).s.origin),
                core::ptr::addr_of!((*ent).s.origin),
                core::ptr::addr_of_mut!(dir),
            );
            VectorNormalize(core::ptr::addr_of_mut!(dir));
            vectoangles(
                core::ptr::addr_of!(dir),
                core::ptr::addr_of_mut!((*ent).s.angles),
            );

            //FIXME: Does pitch get flipped?
        } else {
            if let Some(printf) = gi.Printf {
                printf(
                    b"^1ERROR: ref_tag (%s) has invalid target (%s)\0".as_ptr() as *const c_char,
                    (*ent).targetname,
                    (*ent).target,
                );
            }
        }
    }

    //Add the tag
    TAG_Add(
        (*ent).targetname,
        (*ent).ownername,
        (*ent).s.origin,
        (*ent).s.angles,
        16,
        0,
    );

    //Delete immediately, cannot be refered to as an entity again
    //NOTE: this means if you wanted to link them in a chain for, say, a path, you can't
    G_FreeEntity(ent);
}

const START_TIME_LINK_ENTS: c_int = 100;

#[allow(non_snake_case)]
pub unsafe fn SP_reference_tag(ent: *mut gentity_t) {
    if !(*ent).target.is_null() {
        //Init cannot occur until all entities have been spawned
        // Note: e_ThinkFunc would be a function pointer in the real code
        // For now, we're using a placeholder constant
        (*ent).nextthink = level.time + START_TIME_LINK_ENTS;
    } else {
        ref_link(ent);
    }
}
