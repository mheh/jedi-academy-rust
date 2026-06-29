// Task Manager
//
//	-- jweier

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::collections::BTreeMap;
use crate::codemp::icarus::taskmanager_h::{
    CTask, CTaskGroup, CTaskManager, CSequencer, CBlock,
    TaskEnum, TaskReturnEnum,
    RUNAWAY_LIMIT, TASKFLAG_NORMAL,
    taskGroup_v, tasks_l, taskGroupName_m, taskGroupID_m
};
use crate::codemp::icarus::blockstream_h::{
    POP_FRONT, POP_BACK, PUSH_FRONT, PUSH_BACK, vector_t,
    CBlockMember
};
use crate::codemp::icarus::interpreter_h::{
    TK_INT, TK_FLOAT, TK_STRING, TK_VECTOR, TK_IDENTIFIER,
    ID_WAIT, ID_WAITSIGNAL, ID_PRINT, ID_SOUND, ID_MOVE,
    ID_ROTATE, ID_KILL, ID_REMOVE, ID_CAMERA, ID_SET,
    ID_USE, ID_DECLARE, ID_FREE, ID_SIGNAL, ID_PLAY,
    ID_GET, ID_RANDOM, ID_TAG,
    TYPE_PAN, TYPE_ZOOM, TYPE_MOVE, TYPE_ROLL, TYPE_FOLLOW,
    TYPE_TRACK, TYPE_DISTANCE, TYPE_FADE, TYPE_PATH,
    TYPE_ENABLE, TYPE_DISABLE, TYPE_SHAKE
};
use crate::codemp::icarus::blockstream_h::POP_BACK as MACRO_POP_BACK;

// Local constants
const Q3_INFINITE: f32 = 16777216.0f32;
const WL_ERROR: c_int = 0;
const WL_WARNING: c_int = 1;
const WL_DEBUG: c_int = 2;

// Forward declarations for external engine functions
extern "C" {
    fn SV_GentityNum(num: i32) -> *mut sharedEntity_t;
}

// Local type stub for sharedEntity_t
#[repr(C)]
pub struct sharedEntity_t {
    r: gentityState_t,
}

#[repr(C)]
pub struct gentityState_t {
    pub svFlags: i32,
}

const SVF_ICARUS_FREEZE: i32 = 0x00000040;

// Macro stub for ICARUS_VALIDATE
macro_rules! ICARUS_VALIDATE {
    ($a:expr) => {
        if ($a) == false as i32 { return TaskEnum::TASK_FAILED as i32; }
    };
}

/*
=================================================

CTask

=================================================
*/

impl CTask {
    pub unsafe fn new() -> Self {
        CTask {
            m_id: 0,
            m_timeStamp: 0,
            m_block: std::ptr::null_mut(),
        }
    }

    pub unsafe fn Create(GUID: i32, block: *mut CBlock) -> *mut CTask {
        let task = Box::new(CTask {
            m_id: GUID,
            m_timeStamp: 0,
            m_block: block,
        });

        // TODO: Emit warning
        let task_ptr = Box::into_raw(task);
        if task_ptr.is_null() {
            return std::ptr::null_mut();
        }

        (*task_ptr).SetTimeStamp(0);
        (*task_ptr).SetBlock(block);
        (*task_ptr).SetGUID(GUID);

        task_ptr
    }

    pub fn SetTimeStamp(&mut self, timeStamp: u32) {
        self.m_timeStamp = timeStamp;
    }

    pub fn SetBlock(&mut self, block: *mut CBlock) {
        self.m_block = block;
    }

    pub fn SetGUID(&mut self, id: i32) {
        self.m_id = id;
    }

    pub fn GetTimeStamp(&self) -> u32 {
        self.m_timeStamp
    }

    pub fn GetBlock(&self) -> *mut CBlock {
        self.m_block
    }

    pub fn GetGUID(&self) -> i32 {
        self.m_id
    }

    /*
    -------------------------
    Free
    -------------------------
    */

    pub unsafe fn Free(&mut self) {
        // NOTENOTE: The block is not consumed by the task, it is the sequencer's job to clean blocks up
        let _ = Box::from_raw(self as *mut CTask);
    }
}

/*
=================================================

CTaskGroup

=================================================
*/

impl CTaskGroup {
    pub fn new() -> Self {
        let mut group = CTaskGroup {
            m_completedTasks: BTreeMap::new(),
            m_parent: std::ptr::null_mut(),
            m_numCompleted: 0,
            m_GUID: 0,
        };
        group.Init();
        group
    }

    /*
    -------------------------
    SetGUID
    -------------------------
    */

    pub fn SetGUID(&mut self, GUID: i32) {
        self.m_GUID = GUID;
    }

    /*
    -------------------------
    Init
    -------------------------
    */

    pub fn Init(&mut self) {
        self.m_completedTasks.clear();

        self.m_numCompleted = 0;
        self.m_parent = std::ptr::null_mut();
    }

    /*
    -------------------------
    Add
    -------------------------
    */

    pub unsafe fn Add(&mut self, task: *mut CTask) -> i32 {
        self.m_completedTasks.insert((*task).GetGUID(), false);
        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    MarkTaskComplete
    -------------------------
    */

    pub fn MarkTaskComplete(&mut self, id: i32) -> bool {
        if self.m_completedTasks.contains_key(&id) {
            self.m_completedTasks.insert(id, true);
            self.m_numCompleted += 1;

            return true;
        }

        false
    }

    pub fn GetParent(&self) -> *mut CTaskGroup {
        self.m_parent
    }

    pub fn SetParent(&mut self, group: *mut CTaskGroup) {
        self.m_parent = group;
    }

    pub fn Complete(&self) -> bool {
        (self.m_numCompleted as usize) == self.m_completedTasks.len()
    }

    pub fn GetGUID(&self) -> i32 {
        self.m_GUID
    }
}

/*
=================================================

CTaskManager

=================================================
*/

impl CTaskManager {
    pub fn new() -> Self {
        CTaskManager {
            m_owner: std::ptr::null_mut(),
            m_ownerID: 0,
            m_curGroup: std::ptr::null_mut(),
            m_taskGroups: Vec::new(),
            m_tasks: Vec::new(),
            m_GUID: 0,
            m_count: 0,
            m_taskGroupNameMap: BTreeMap::new(),
            m_taskGroupIDMap: BTreeMap::new(),
            m_resident: false,
        }
    }

    /*
    -------------------------
    Create
    -------------------------
    */

    pub unsafe fn Create() -> *mut CTaskManager {
        Box::into_raw(Box::new(CTaskManager::new()))
    }

    /*
    -------------------------
    Init
    -------------------------
    */

    pub unsafe fn Init(&mut self, owner: *mut CSequencer) -> i32 {
        // TODO: Emit warning
        if owner.is_null() {
            return TaskEnum::TASK_FAILED as i32;
        }

        self.m_tasks.clear();
        self.m_owner = owner;
        self.m_ownerID = (*owner).GetOwnerID();
        self.m_curGroup = std::ptr::null_mut();
        self.m_GUID = 0;
        self.m_resident = false;

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Free
    -------------------------
    */

    pub unsafe fn Free(&mut self) -> i32 {
        // Clear out all pending tasks
        for ti in self.m_tasks.iter() {
            (*ti).Free();
        }

        self.m_tasks.clear();

        // Clear out all taskGroups
        for gi in self.m_taskGroups.iter() {
            let _ = Box::from_raw(*gi);
        }

        self.m_taskGroups.clear();
        self.m_taskGroupNameMap.clear();
        self.m_taskGroupIDMap.clear();

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Flush
    -------------------------
    */

    pub fn Flush(&mut self) -> i32 {
        // FIXME: Rewrite

        true as i32
    }

    /*
    -------------------------
    AddTaskGroup
    -------------------------
    */

    pub unsafe fn AddTaskGroup(&mut self, name: *const c_char) -> *mut CTaskGroup {
        let name_str = if !name.is_null() {
            std::ffi::CStr::from_ptr(name).to_string_lossy().to_string()
        } else {
            String::new()
        };

        // Collect any garbage
        if let Some(_) = self.m_taskGroupNameMap.get(&name_str) {
            let group = self.m_taskGroupNameMap[&name_str];

            // Clear it and just move on
            (*group).Init();

            return group;
        }

        // Allocate a new one
        let group = Box::new(CTaskGroup::new());

        // TODO: Emit warning
        let group_ptr = Box::into_raw(group);
        if group_ptr.is_null() {
            ((*self.m_owner).GetInterface()).I_DPrintf(
                WL_ERROR,
                b"Unable to allocate task group \"%s\"\n\0" as *const u8 as *const c_char,
                name,
            );
            return std::ptr::null_mut();
        }

        // Setup the internal information
        (*group_ptr).SetGUID(self.m_GUID);
        self.m_GUID += 1;

        // Add it to the list and associate it for retrieval later
        self.m_taskGroups.push(group_ptr);
        self.m_taskGroupNameMap.insert(name_str, group_ptr);
        self.m_taskGroupIDMap.insert((*group_ptr).GetGUID(), group_ptr);

        group_ptr
    }

    /*
    -------------------------
    GetTaskGroup
    -------------------------
    */

    pub unsafe fn GetTaskGroup_str(&self, name: *const c_char) -> *mut CTaskGroup {
        let name_str = if !name.is_null() {
            std::ffi::CStr::from_ptr(name).to_string_lossy().to_string()
        } else {
            String::new()
        };

        if let Some(group) = self.m_taskGroupNameMap.get(&name_str) {
            return *group;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_WARNING,
            b"Could not find task group \"%s\"\n\0" as *const u8 as *const c_char,
            name,
        );
        std::ptr::null_mut()
    }

    pub unsafe fn GetTaskGroup_id(&self, id: i32) -> *mut CTaskGroup {
        if let Some(group) = self.m_taskGroupIDMap.get(&id) {
            return *group;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_WARNING,
            b"Could not find task group \"%d\"\n\0" as *const u8 as *const c_char,
            id,
        );
        std::ptr::null_mut()
    }

    /*
    -------------------------
    Update
    -------------------------
    */

    pub unsafe fn Update(&mut self) -> i32 {
        let owner = SV_GentityNum(self.m_ownerID);

        if ((*owner).r.svFlags & SVF_ICARUS_FREEZE) != 0 {
            return TaskEnum::TASK_FAILED as i32;
        }
        self.m_count = 0; // Needed for runaway init
        self.m_resident = true;

        let returnVal = self.Go();

        self.m_resident = false;

        returnVal
    }

    /*
    -------------------------
    IsRunning
    -------------------------
    */

    pub fn IsRunning(&self) -> i32 {
        (self.m_tasks.is_empty() == false) as i32
    }

    /*
    -------------------------
    Check
    -------------------------
    */

    unsafe fn Check(&self, targetID: i32, block: *mut CBlock, memberNum: i32) -> bool {
        if ((*block).GetMember(memberNum)).GetID() == targetID {
            return true;
        }

        false
    }

    /*
    -------------------------
    GetFloat
    -------------------------
    */

    unsafe fn GetFloat(
        &mut self,
        entID: i32,
        block: *mut CBlock,
        memberNum: &mut i32,
        value: &mut f32,
    ) -> i32 {
        let mut name: *const c_char = std::ptr::null();
        let mut type_: i32 = 0;

        // See if this is a get() command replacement
        if self.Check(ID_GET, block, *memberNum) {
            // Update the member past the header id
            *memberNum += 1;

            // get( TYPE, NAME )
            type_ = (*((*block).GetMemberData(*memberNum as usize)) as *const f32) as i32;
            *memberNum += 1;
            name = (*block).GetMemberData(*memberNum as usize) as *const c_char;
            *memberNum += 1;

            // TODO: Emit warning
            if type_ != TK_FLOAT {
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_ERROR,
                    b"Get() call tried to return a non-FLOAT parameter!\n\0" as *const u8
                        as *const c_char,
                );
                return false as i32;
            }

            return ((*self.m_owner).GetInterface()).I_GetFloat(entID, type_, name, value);
        }

        // Look for a random() inline call
        if self.Check(ID_RANDOM, block, *memberNum) {
            let mut min: f32 = 0.0;
            let mut max: f32 = 0.0;

            *memberNum += 1;

            min = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;
            max = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;

            *value = ((*self.m_owner).GetInterface()).I_Random(min, max);

            return true as i32;
        }

        // Look for a tag() inline call
        if self.Check(ID_TAG, block, *memberNum) {
            ((*self.m_owner).GetInterface()).I_DPrintf(
                WL_WARNING,
                b"Invalid use of \"tag\" inline.  Not a valid replacement for type FLOAT\n\0"
                    as *const u8 as *const c_char,
            );
            return false as i32;
        }

        let bm = (*block).GetMember(*memberNum as usize);

        if (*bm).GetID() == TK_INT {
            *value = (*((*block).GetMemberData(*memberNum as usize) as *const i32) as f32);
            *memberNum += 1;
        } else if (*bm).GetID() == TK_FLOAT {
            *value = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;
        } else {
            assert!(false);
            ((*self.m_owner).GetInterface()).I_DPrintf(
                WL_WARNING,
                b"Unexpected value; expected type FLOAT\n\0" as *const u8 as *const c_char,
            );
            return false as i32;
        }

        true as i32
    }

    /*
    -------------------------
    GetVector
    -------------------------
    */

    unsafe fn GetVector(
        &mut self,
        entID: i32,
        block: *mut CBlock,
        memberNum: &mut i32,
        value: &mut vector_t,
    ) -> i32 {
        let mut name: *const c_char = std::ptr::null();
        let mut type_: i32 = 0;
        let mut i: i32 = 0;

        // See if this is a get() command replacement
        if self.Check(ID_GET, block, *memberNum) {
            // Update the member past the header id
            *memberNum += 1;

            // get( TYPE, NAME )
            type_ = (*((*block).GetMemberData(*memberNum as usize) as *const f32)) as i32;
            *memberNum += 1;
            name = (*block).GetMemberData(*memberNum as usize) as *const c_char;
            *memberNum += 1;

            // TODO: Emit warning
            if type_ != TK_VECTOR {
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_ERROR,
                    b"Get() call tried to return a non-VECTOR parameter!\n\0" as *const u8
                        as *const c_char,
                );
            }

            return ((*self.m_owner).GetInterface()).I_GetVector(entID, type_, name, value);
        }

        // Look for a random() inline call
        if self.Check(ID_RANDOM, block, *memberNum) {
            let mut min: f32 = 0.0;
            let mut max: f32 = 0.0;

            *memberNum += 1;

            min = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;
            max = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;

            for i in 0..3 {
                value[i as usize] =
                    ((*self.m_owner).GetInterface()).I_Random(min, max) as f32; // FIXME: Just truncating it for now.. should be fine though
            }

            return true as i32;
        }

        // Look for a tag() inline call
        if self.Check(ID_TAG, block, *memberNum) {
            let mut tagName: *const c_char = std::ptr::null();
            let mut tagLookup: f32 = 0.0;

            *memberNum += 1;
            if self.Get(entID, block, memberNum, &mut tagName) == false as i32 {
                return TaskEnum::TASK_FAILED as i32;
            }
            if self.GetFloat(entID, block, memberNum, &mut tagLookup) == false as i32 {
                return TaskEnum::TASK_FAILED as i32;
            }

            if ((*self.m_owner).GetInterface()).I_GetTag(entID, tagName, tagLookup as i32, value)
                == false as i32
            {
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_ERROR,
                    b"Unable to find tag \"%s\" for ent %i!\n\0" as *const u8 as *const c_char,
                    tagName,
                    entID,
                );
                // assert(0);
                return TaskEnum::TASK_FAILED as i32;
            }

            return true as i32;
        }

        // Check for a real vector here
        type_ = (*((*block).GetMemberData(*memberNum as usize) as *const f32)) as i32;

        if type_ != TK_VECTOR {
            // (m_owner->GetInterface())->I_DPrintf( WL_WARNING, "Unexpected value; expected type VECTOR\n" );
            return false as i32;
        }

        *memberNum += 1;

        for i in 0..3 {
            if self.GetFloat(entID, block, memberNum, &mut value[i as usize]) == false as i32 {
                return false as i32;
            }
        }

        true as i32
    }

    /*
    -------------------------
    Get
    -------------------------
    */

    unsafe fn Get(
        &mut self,
        entID: i32,
        block: *mut CBlock,
        memberNum: &mut i32,
        value: &mut *const c_char,
    ) -> i32 {
        static mut tempBuffer: [c_char; 128] = [0; 128]; // FIXME: EEEK!
        let mut vector: vector_t = [0.0; 3];
        let mut name: *const c_char = std::ptr::null();
        let mut tagName: *const c_char = std::ptr::null();
        let mut tagLookup: f32 = 0.0;
        let mut type_: i32 = 0;

        // Look for a get() inline call
        if self.Check(ID_GET, block, *memberNum) {
            // Update the member past the header id
            *memberNum += 1;

            // get( TYPE, NAME )
            type_ = (*((*block).GetMemberData(*memberNum as usize) as *const f32)) as i32;
            *memberNum += 1;
            name = (*block).GetMemberData(*memberNum as usize) as *const c_char;
            *memberNum += 1;

            // Format the return properly
            // FIXME: This is probably doing double formatting in certain cases...
            // FIXME: STRING MANAGEMENT NEEDS TO BE IMPLEMENTED, MY CURRENT SOLUTION IS NOT ACCEPTABLE!!
            match type_ {
                TK_STRING => {
                    if ((*self.m_owner).GetInterface()).I_GetString(entID, type_, name, value)
                        == false as i32
                    {
                        ((*self.m_owner).GetInterface()).I_DPrintf(
                            WL_ERROR,
                            b"Get() parameter \"%s\" could not be found!\n\0" as *const u8
                                as *const c_char,
                            name,
                        );
                        return false as i32;
                    }

                    return true as i32;
                }

                TK_FLOAT => {
                    let mut temp: f32 = 0.0;

                    if ((*self.m_owner).GetInterface()).I_GetFloat(entID, type_, name, &mut temp)
                        == false as i32
                    {
                        ((*self.m_owner).GetInterface()).I_DPrintf(
                            WL_ERROR,
                            b"Get() parameter \"%s\" could not be found!\n\0" as *const u8
                                as *const c_char,
                            name,
                        );
                        return false as i32;
                    }

                    let len = sprintf(
                        tempBuffer.as_mut_ptr(),
                        b"%f\0".as_ptr() as *const c_char,
                        temp,
                    );
                    *value = tempBuffer.as_ptr();

                    return true as i32;
                }

                TK_VECTOR => {
                    let mut vval: vector_t = [0.0; 3];

                    if ((*self.m_owner).GetInterface()).I_GetVector(entID, type_, name, &mut vval)
                        == false as i32
                    {
                        ((*self.m_owner).GetInterface()).I_DPrintf(
                            WL_ERROR,
                            b"Get() parameter \"%s\" could not be found!\n\0" as *const u8
                                as *const c_char,
                            name,
                        );
                        return false as i32;
                    }

                    sprintf(
                        tempBuffer.as_mut_ptr(),
                        b"%f %f %f\0".as_ptr() as *const c_char,
                        vval[0],
                        vval[1],
                        vval[2],
                    );
                    *value = tempBuffer.as_ptr();

                    return true as i32;
                }

                _ => {
                    ((*self.m_owner).GetInterface()).I_DPrintf(
                        WL_ERROR,
                        b"Get() call tried to return an unknown type!\n\0" as *const u8
                            as *const c_char,
                    );
                    return false as i32;
                }
            }
        }

        // Look for a random() inline call
        if self.Check(ID_RANDOM, block, *memberNum) {
            let mut min: f32 = 0.0;
            let mut max: f32 = 0.0;
            let mut ret: f32 = 0.0;

            *memberNum += 1;

            min = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;
            max = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;

            ret = ((*self.m_owner).GetInterface()).I_Random(min, max);

            sprintf(
                tempBuffer.as_mut_ptr(),
                b"%f\0".as_ptr() as *const c_char,
                ret,
            );
            *value = tempBuffer.as_ptr();

            return true as i32;
        }

        // Look for a tag() inline call
        if self.Check(ID_TAG, block, *memberNum) {
            *memberNum += 1;
            if self.Get(entID, block, memberNum, &mut tagName) == false as i32 {
                return TaskEnum::TASK_FAILED as i32;
            }
            if self.GetFloat(entID, block, memberNum, &mut tagLookup) == false as i32 {
                return TaskEnum::TASK_FAILED as i32;
            }

            if ((*self.m_owner).GetInterface()).I_GetTag(
                entID,
                tagName,
                tagLookup as i32,
                &mut vector,
            ) == false as i32
            {
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_ERROR,
                    b"Unable to find tag \"%s\"!\n\0" as *const u8 as *const c_char,
                    tagName,
                );
                assert!(false, "Unable to find tag");
                return false as i32;
            }

            sprintf(
                tempBuffer.as_mut_ptr(),
                b"%f %f %f\0".as_ptr() as *const c_char,
                vector[0],
                vector[1],
                vector[2],
            );
            *value = tempBuffer.as_ptr();

            return true as i32;
        }

        // Get an actual piece of data

        let bm = (*block).GetMember(*memberNum as usize);

        if (*bm).GetID() == TK_INT {
            let fval = (*((*block).GetMemberData(*memberNum as usize) as *const i32)) as f32;
            *memberNum += 1;
            sprintf(
                tempBuffer.as_mut_ptr(),
                b"%f\0".as_ptr() as *const c_char,
                fval,
            );
            *value = tempBuffer.as_ptr();

            return true as i32;
        } else if (*bm).GetID() == TK_FLOAT {
            let fval = *((*block).GetMemberData(*memberNum as usize) as *const f32);
            *memberNum += 1;
            sprintf(
                tempBuffer.as_mut_ptr(),
                b"%f\0".as_ptr() as *const c_char,
                fval,
            );
            *value = tempBuffer.as_ptr();

            return true as i32;
        } else if (*bm).GetID() == TK_VECTOR {
            let mut vval: vector_t = [0.0; 3];

            *memberNum += 1;

            for i in 0..3 {
                if self.GetFloat(entID, block, memberNum, &mut vval[i as usize]) == false as i32 {
                    return false as i32;
                }

                sprintf(
                    tempBuffer.as_mut_ptr(),
                    b"%f %f %f\0".as_ptr() as *const c_char,
                    vval[0],
                    vval[1],
                    vval[2],
                );
                *value = tempBuffer.as_ptr();
            }

            return true as i32;
        } else if ((*bm).GetID() == TK_STRING) || ((*bm).GetID() == TK_IDENTIFIER) {
            *value = (*block).GetMemberData(*memberNum as usize) as *const c_char;
            *memberNum += 1;

            return true as i32;
        }

        // TODO: Emit warning
        assert!(false);
        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_WARNING,
            b"Unexpected value; expected type STRING\n\0" as *const u8 as *const c_char,
        );

        false as i32
    }

    /*
    -------------------------
    Go
    -------------------------
    */

    unsafe fn Go(&mut self) -> i32 {
        let mut task: *mut CTask = std::ptr::null_mut();
        let mut completed: bool = false;

        // Check for run away scripts
        if self.m_count > RUNAWAY_LIMIT {
            self.m_count += 1;
            assert!(false);
            ((*self.m_owner).GetInterface()).I_DPrintf(
                WL_ERROR,
                b"Runaway loop detected!\n\0" as *const u8 as *const c_char,
            );
            return TaskEnum::TASK_FAILED as i32;
        }
        self.m_count += 1;

        // If there are tasks to complete, do so
        if self.m_tasks.is_empty() == false {
            // Get the next task
            task = self.PopTask(POP_BACK);

            assert!(!task.is_null());
            if task.is_null() {
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_ERROR,
                    b"Invalid task found in Go()!\n\0" as *const u8 as *const c_char,
                );
                return TaskEnum::TASK_FAILED as i32;
            }

            // If this hasn't been stamped, do so
            if (*task).GetTimeStamp() == 0 {
                (*task).SetTimeStamp(((*self.m_owner).GetInterface()).I_GetTime() as u32);
            }

            // Switch and call the proper function
            match (*task).GetID() {
                ID_WAIT => {
                    self.Wait(task, &mut completed);

                    // Push it to consider it again on the next frame if not complete
                    if completed == false {
                        self.PushTask(task, PUSH_BACK);
                        return TaskEnum::TASK_OK as i32;
                    }

                    self.Completed((*task).GetGUID());
                }

                ID_WAITSIGNAL => {
                    self.WaitSignal(task, &mut completed);

                    // Push it to consider it again on the next frame if not complete
                    if completed == false {
                        self.PushTask(task, PUSH_BACK);
                        return TaskEnum::TASK_OK as i32;
                    }

                    self.Completed((*task).GetGUID());
                }

                ID_PRINT => {
                    // print( STRING )
                    self.Print(task);
                }

                ID_SOUND => {
                    // sound( name )
                    self.Sound(task);
                }

                ID_MOVE => {
                    // move ( ORIGIN, ANGLES, DURATION )
                    self.Move(task);
                }

                ID_ROTATE => {
                    // rotate( ANGLES, DURATION )
                    self.Rotate(task);
                }

                ID_KILL => {
                    // kill( NAME )
                    self.Kill(task);
                }

                ID_REMOVE => {
                    // remove( NAME )
                    self.Remove(task);
                }

                ID_CAMERA => {
                    // camera( ? )
                    self.Camera(task);
                }

                ID_SET => {
                    // set( NAME, ? )
                    self.Set(task);
                }

                ID_USE => {
                    // use( NAME )
                    self.Use(task);
                }

                ID_DECLARE => {
                    // declare( TYPE, NAME )
                    self.DeclareVariable(task);
                }

                ID_FREE => {
                    // free( NAME )
                    self.FreeVariable(task);
                }

                ID_SIGNAL => {
                    // signal( NAME )
                    self.Signal(task);
                }

                ID_PLAY => {
                    // play ( NAME )
                    self.Play(task);
                }

                _ => {
                    assert!(false);
                    (*task).Free();
                    ((*self.m_owner).GetInterface()).I_DPrintf(
                        WL_ERROR,
                        b"Found unknown task type!\n\0" as *const u8 as *const c_char,
                    );
                    return TaskEnum::TASK_FAILED as i32;
                }
            }

            // Pump the sequencer for another task
            self.CallbackCommand(task, TaskReturnEnum::TASK_RETURN_COMPLETE as i32);

            (*task).Free();
        }

        // FIXME: A command surge limiter could be implemented at this point to be sure a script doesn't
        // 		 execute too many commands in one cycle.  This may, however, cause timing errors to surface.

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    SetCommand
    -------------------------
    */

    unsafe fn SetCommand(&mut self, command: *mut CBlock, type_: i32) -> i32 {
        let task = CTask::Create(self.m_GUID, command);
        self.m_GUID += 1;

        // If this is part of a task group, add it in
        if !self.m_curGroup.is_null() {
            (*self.m_curGroup).Add(task);
        }

        // TODO: Emit warning
        assert!(!task.is_null());
        if task.is_null() {
            ((*self.m_owner).GetInterface()).I_DPrintf(
                WL_ERROR,
                b"Unable to allocate new task!\n\0" as *const u8 as *const c_char,
            );
            return TaskEnum::TASK_FAILED as i32;
        }

        self.PushTask(task, type_);

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    MarkTask
    -------------------------
    */

    unsafe fn MarkTask(&mut self, id: i32, operation: i32) -> i32 {
        let group = self.GetTaskGroup_id(id);

        assert!(!group.is_null());

        if group.is_null() {
            return TaskEnum::TASK_FAILED as i32;
        }

        if operation == TaskEnum::TASK_START as i32 {
            // Reset all the completion information
            (*group).Init();

            (*group).SetParent(self.m_curGroup);
            self.m_curGroup = group;
        } else if operation == TaskEnum::TASK_END as i32 {
            assert!(!self.m_curGroup.is_null());
            if self.m_curGroup.is_null() {
                return TaskEnum::TASK_FAILED as i32;
            }

            self.m_curGroup = (*self.m_curGroup).GetParent();
        }

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Completed
    -------------------------
    */

    unsafe fn Completed(&mut self, id: i32) -> i32 {
        // Mark the task as completed
        for tgi in self.m_taskGroups.iter() {
            // If this returns true, then the task was marked properly
            if (*tgi).MarkTaskComplete(id) {
                break;
            }
        }

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    CallbackCommand
    -------------------------
    */

    unsafe fn CallbackCommand(&mut self, task: *mut CTask, returnCode: i32) -> i32 {
        if (*self.m_owner).Callback(self as *mut CTaskManager, (*task).GetBlock(), returnCode)
            == 0 // SEQ_OK
        {
            return self.Go();
        }

        assert!(false);

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_ERROR,
            b"Command callback failure!\n\0" as *const u8 as *const c_char,
        );
        TaskEnum::TASK_FAILED as i32
    }

    /*
    -------------------------
    RecallTask
    -------------------------
    */

    unsafe fn RecallTask(&mut self) -> *mut CBlock {
        let task = self.PopTask(POP_BACK);

        if !task.is_null() {
            // fixed 2/12/2 to free the task that has been popped (called from sequencer Recall)
            let retBlock = (*task).GetBlock();
            (*task).Free();

            return retBlock;
            // return task->GetBlock();
        }

        std::ptr::null_mut()
    }

    /*
    -------------------------
    PushTask
    -------------------------
    */

    unsafe fn PushTask(&mut self, task: *mut CTask, flag: i32) -> i32 {
        assert!((flag == PUSH_FRONT) || (flag == PUSH_BACK));

        match flag {
            PUSH_FRONT => {
                self.m_tasks.insert(0, task);

                TaskEnum::TASK_OK as i32
            }

            PUSH_BACK => {
                self.m_tasks.push(task);

                TaskEnum::TASK_OK as i32
            }

            _ => {
                // Invalid flag
                1 // SEQ_FAILED
            }
        }
    }

    /*
    -------------------------
    PopTask
    -------------------------
    */

    unsafe fn PopTask(&mut self, flag: i32) -> *mut CTask {
        let mut task: *mut CTask = std::ptr::null_mut();

        assert!((flag == POP_FRONT) || (flag == POP_BACK));

        if self.m_tasks.is_empty() {
            return std::ptr::null_mut();
        }

        match flag {
            POP_FRONT => {
                task = self.m_tasks.remove(0);

                task
            }

            POP_BACK => {
                if let Some(t) = self.m_tasks.pop() {
                    task = t;
                }

                task
            }

            _ => {
                // Invalid flag
                std::ptr::null_mut()
            }
        }
    }

    /*
    -------------------------
    GetCurrentTask
    -------------------------
    */

    unsafe fn GetCurrentTask(&mut self) -> *mut CBlock {
        let task = self.PopTask(POP_BACK);

        if task.is_null() {
            return std::ptr::null_mut();
        }
        // fixed 2/12/2 to free the task that has been popped (called from sequencer Interrupt)
        let retBlock = (*task).GetBlock();
        (*task).Free();

        retBlock
        // return task->GetBlock();
    }
}

/*
=================================================

  Task Functions

=================================================
*/

impl CTaskManager {
    unsafe fn Wait(&mut self, task: *mut CTask, completed: &mut bool) -> i32 {
        let mut bm: *const CBlockMember;
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut dwtime: f32 = 0.0;
        let mut memberNum: i32 = 0;

        *completed = false;

        bm = (*block).GetMember(memberNum as usize);

        // Check if this is a task completion wait
        if (*bm).GetID() == TK_STRING {
            if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
                return TaskEnum::TASK_FAILED as i32;
            }

            if (*task).GetTimeStamp() == ((*self.m_owner).GetInterface()).I_GetTime() as u32 {
                // Print out the debug info
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_DEBUG,
                    b"%4d wait(\"%s\"); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    sVal,
                    (*task).GetTimeStamp(),
                );
            }

            let group = self.GetTaskGroup_str(sVal);

            if group.is_null() {
                // TODO: Emit warning
                *completed = false;
                return TaskEnum::TASK_FAILED as i32;
            }

            *completed = (*group).Complete();
        } else {
            // Otherwise it's a time completion wait
            if self.Check(ID_RANDOM, block, memberNum) {
                // get it random only the first time
                let mut min: f32 = 0.0;
                let mut max: f32 = 0.0;

                dwtime = *((*block).GetMemberData(memberNum as usize) as *const f32);
                if dwtime == Q3_INFINITE {
                    // we have not evaluated this random yet
                    min = *((*block).GetMemberData((memberNum + 1) as usize) as *const f32);
                    max = *((*block).GetMemberData((memberNum + 2) as usize) as *const f32);

                    dwtime = ((*self.m_owner).GetInterface()).I_Random(min, max);

                    // store the result in the first member
                    (*bm).SetData(&dwtime, std::mem::size_of::<f32>() as i32);
                }
            } else {
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut dwtime)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
            }

            if (*task).GetTimeStamp() == ((*self.m_owner).GetInterface()).I_GetTime() as u32 {
                // Print out the debug info
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_DEBUG,
                    b"%4d wait( %d ); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    dwtime as i32,
                    (*task).GetTimeStamp(),
                );
            }

            if (((*task).GetTimeStamp() as f32 + dwtime) as i32)
                < (((*self.m_owner).GetInterface()).I_GetTime() as i32)
            {
                *completed = true;
                memberNum = 0;
                if self.Check(ID_RANDOM, block, memberNum) {
                    // set the data back to 0 so it will be re-randomized next time
                    dwtime = Q3_INFINITE;
                    (*bm).SetData(&dwtime, std::mem::size_of::<f32>() as i32);
                }
            }
        }

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    WaitSignal
    -------------------------
    */

    unsafe fn WaitSignal(&mut self, task: *mut CTask, completed: &mut bool) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        *completed = false;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        if (*task).GetTimeStamp() == ((*self.m_owner).GetInterface()).I_GetTime() as u32 {
            // Print out the debug info
            ((*self.m_owner).GetInterface()).I_DPrintf(
                WL_DEBUG,
                b"%4d waitsignal(\"%s\"); [%d]\0" as *const u8 as *const c_char,
                self.m_ownerID,
                sVal,
                (*task).GetTimeStamp(),
            );
        }

        if ((*self.m_owner).GetOwner()).CheckSignal(sVal) != 0 {
            *completed = true;
            ((*self.m_owner).GetOwner()).ClearSignal(sVal);
        }

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Print
    -------------------------
    */

    unsafe fn Print(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d print(\"%s\"); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            (*task).GetTimeStamp(),
        );

        ((*self.m_owner).GetInterface()).I_CenterPrint(sVal);

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Sound
    -------------------------
    */

    unsafe fn Sound(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut sVal2: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }
        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal2) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d sound(\"%s\", \"%s\"); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            sVal2,
            (*task).GetTimeStamp(),
        );

        // Only instantly complete if the user has requested it
        if ((*self.m_owner).GetInterface()).I_PlaySound(
            (*task).GetGUID(),
            self.m_ownerID,
            sVal2,
            sVal,
        ) != 0
        {
            self.Completed((*task).GetGUID());
        }

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Rotate
    -------------------------
    */

    unsafe fn Rotate(&mut self, task: *mut CTask) -> i32 {
        let mut vector: vector_t = [0.0; 3];
        let block = (*task).GetBlock();
        let mut tagName: *const c_char = std::ptr::null();
        let mut tagLookup: f32 = 0.0;
        let mut duration: f32 = 0.0;
        let mut memberNum: i32 = 0;

        // Check for a tag reference
        if self.Check(ID_TAG, block, memberNum) {
            memberNum += 1;

            if self.Get(self.m_ownerID, block, &mut memberNum, &mut tagName) == false as i32 {
                return TaskEnum::TASK_FAILED as i32;
            }
            if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut tagLookup)
                == false as i32
            {
                return TaskEnum::TASK_FAILED as i32;
            }

            if ((*self.m_owner).GetInterface()).I_GetTag(
                self.m_ownerID,
                tagName,
                tagLookup as i32,
                &mut vector,
            ) == false as i32
            {
                // TODO: Emit warning
                ((*self.m_owner).GetInterface()).I_DPrintf(
                    WL_ERROR,
                    b"Unable to find tag \"%s\"!\n\0" as *const u8 as *const c_char,
                    tagName,
                );
                assert!(false);
                return TaskEnum::TASK_FAILED as i32;
            }
        } else {
            // Get a normal vector
            if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector)
                == false as i32
            {
                return TaskEnum::TASK_FAILED as i32;
            }
        }

        // Find the duration
        if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut duration) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d rotate( <%f,%f,%f>, %d); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            vector[0],
            vector[1],
            vector[2],
            duration as i32,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_Lerp2Angles(
            (*task).GetGUID(),
            self.m_ownerID,
            &vector,
            duration,
        );

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Remove
    -------------------------
    */

    unsafe fn Remove(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d remove(\"%s\"); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_Remove(self.m_ownerID, sVal);

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Camera
    -------------------------
    */

    unsafe fn Camera(&mut self, task: *mut CTask) -> i32 {
        let ie = (*self.m_owner).GetInterface();
        let block = (*task).GetBlock();
        let mut vector: vector_t = [0.0; 3];
        let mut vector2: vector_t = [0.0; 3];
        let mut type_: f32 = 0.0;
        let mut fVal: f32 = 0.0;
        let mut fVal2: f32 = 0.0;
        let mut fVal3: f32 = 0.0;
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        // Get the camera function type
        if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut type_) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        match type_ as i32 {
            TYPE_PAN => {
                if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( PAN, <%f %f %f>, <%f %f %f>, %f); [%d]\0" as *const u8
                        as *const c_char,
                    self.m_ownerID,
                    vector[0],
                    vector[1],
                    vector[2],
                    vector2[0],
                    vector2[1],
                    vector2[2],
                    fVal,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraPan(&vector, &vector2, fVal);
            }

            TYPE_ZOOM => {
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( ZOOM, %f, %f); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    fVal,
                    fVal2,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraZoom(fVal, fVal2);
            }

            TYPE_MOVE => {
                if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( MOVE, <%f %f %f>, %f); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    vector[0],
                    vector[1],
                    vector[2],
                    fVal,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraMove(&vector, fVal);
            }

            TYPE_ROLL => {
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( ROLL, %f, %f); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    fVal,
                    fVal2,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraRoll(fVal, fVal2);
            }

            TYPE_FOLLOW => {
                if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( FOLLOW, \"%s\", %f, %f); [%d]\0" as *const u8
                        as *const c_char,
                    self.m_ownerID,
                    sVal,
                    fVal,
                    fVal2,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraFollow(sVal as *const u8, fVal, fVal2);
            }

            TYPE_TRACK => {
                if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( TRACK, \"%s\", %f, %f); [%d]\0" as *const u8
                        as *const c_char,
                    self.m_ownerID,
                    sVal,
                    fVal,
                    fVal2,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraTrack(sVal as *const u8, fVal, fVal2);
            }

            TYPE_DISTANCE => {
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( DISTANCE, %f, %f); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    fVal,
                    fVal2,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraDistance(fVal, fVal2);
            }

            TYPE_FADE => {
                if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal3)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( FADE, <%f %f %f>, %f, <%f %f %f>, %f, %f); [%d]\0"
                        as *const u8 as *const c_char,
                    self.m_ownerID,
                    vector[0],
                    vector[1],
                    vector[2],
                    fVal,
                    vector2[0],
                    vector2[1],
                    vector2[2],
                    fVal2,
                    fVal3,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraFade(
                    vector[0], vector[1], vector[2], fVal, vector2[0], vector2[1], vector2[2],
                    fVal2, fVal3,
                );
            }

            TYPE_PATH => {
                if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( PATH, \"%s\"); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    sVal,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraPath(sVal);
            }

            TYPE_ENABLE => {
                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( ENABLE ); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraEnable();
            }

            TYPE_DISABLE => {
                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( DISABLE ); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraDisable();
            }

            TYPE_SHAKE => {
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }
                if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal2)
                    == false as i32
                {
                    return TaskEnum::TASK_FAILED as i32;
                }

                (*ie).I_DPrintf(
                    WL_DEBUG,
                    b"%4d camera( SHAKE, %f, %f ); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    fVal,
                    fVal2,
                    (*task).GetTimeStamp(),
                );
                ie.I_CameraShake(fVal, fVal2 as i32);
            }

            _ => {}
        }

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Move
    -------------------------
    */

    unsafe fn Move(&mut self, task: *mut CTask) -> i32 {
        let mut vector: vector_t = [0.0; 3];
        let mut vector2: vector_t = [0.0; 3];
        let block = (*task).GetBlock();
        let mut duration: f32 = 0.0;
        let mut memberNum: i32 = 0;

        // Get the goal position
        if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        // Check for possible angles field
        if self.GetVector(self.m_ownerID, block, &mut memberNum, &mut vector2) == false as i32 {
            if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut duration)
                == false as i32
            {
                return TaskEnum::TASK_FAILED as i32;
            }

            (*self.m_owner)
                .GetInterface()
                .I_DPrintf(
                    WL_DEBUG,
                    b"%4d move( <%f %f %f>, %f ); [%d]\0" as *const u8 as *const c_char,
                    self.m_ownerID,
                    vector[0],
                    vector[1],
                    vector[2],
                    duration,
                    (*task).GetTimeStamp(),
                );
            (*self.m_owner).GetInterface().I_Lerp2Pos(
                (*task).GetGUID(),
                self.m_ownerID,
                &vector,
                std::ptr::null(),
                duration,
            );

            return TaskEnum::TASK_OK as i32;
        }

        // Get the duration and make the call
        if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut duration) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d move( <%f %f %f>, <%f %f %f>, %f ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            vector[0],
            vector[1],
            vector[2],
            vector2[0],
            vector2[1],
            vector2[2],
            duration,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_Lerp2Pos(
            (*task).GetGUID(),
            self.m_ownerID,
            &vector,
            &vector2,
            duration,
        );

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Kill
    -------------------------
    */

    unsafe fn Kill(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d kill( \"%s\" ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_Kill(self.m_ownerID, sVal);

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Set
    -------------------------
    */

    unsafe fn Set(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut sVal2: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }
        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal2) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d set( \"%s\", \"%s\" ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            sVal2,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_Set(
            (*task).GetGUID(),
            self.m_ownerID,
            sVal,
            sVal2,
        );

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Use
    -------------------------
    */

    unsafe fn Use(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d use( \"%s\" ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_Use(self.m_ownerID, sVal);

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    DeclareVariable
    -------------------------
    */

    unsafe fn DeclareVariable(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;
        let mut fVal: f32 = 0.0;

        if self.GetFloat(self.m_ownerID, block, &mut memberNum, &mut fVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }
        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d declare( %d, \"%s\" ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            fVal as i32,
            sVal,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_DeclareVariable(fVal as i32, sVal);

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    FreeVariable
    -------------------------
    */

    unsafe fn FreeVariable(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d free( \"%s\" ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_FreeVariable(sVal);

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Signal
    -------------------------
    */

    unsafe fn Signal(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d signal( \"%s\" ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            (*task).GetTimeStamp(),
        );
        (*self.m_owner).GetOwner().Signal(sVal as *const u8);

        self.Completed((*task).GetGUID());

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    Play
    -------------------------
    */

    unsafe fn Play(&mut self, task: *mut CTask) -> i32 {
        let block = (*task).GetBlock();
        let mut sVal: *const c_char = std::ptr::null();
        let mut sVal2: *const c_char = std::ptr::null();
        let mut memberNum: i32 = 0;

        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }
        if self.Get(self.m_ownerID, block, &mut memberNum, &mut sVal2) == false as i32 {
            return TaskEnum::TASK_FAILED as i32;
        }

        ((*self.m_owner).GetInterface()).I_DPrintf(
            WL_DEBUG,
            b"%4d play( \"%s\", \"%s\" ); [%d]\0" as *const u8 as *const c_char,
            self.m_ownerID,
            sVal,
            sVal2,
            (*task).GetTimeStamp(),
        );
        ((*self.m_owner).GetInterface()).I_Play(
            (*task).GetGUID(),
            self.m_ownerID,
            sVal as *const u8,
            sVal2 as *const u8,
        );

        TaskEnum::TASK_OK as i32
    }

    /*
    -------------------------
    SaveCommand
    -------------------------
    */

    // FIXME: ARGH!  This is duplicated from CSequence because I can't directly link it any other way...

    unsafe fn SaveCommand(&mut self, block: *mut CBlock) -> i32 {
        let mut flags: u8 = 0;
        let mut numMembers: i32 = 0;
        let mut bID: i32 = 0;
        let mut size: i32 = 0;
        let mut bm: *const CBlockMember;

        // Save out the block ID
        bID = (*block).GetBlockID();
        ((*self.m_owner).GetInterface()).I_WriteSaveData(
            1111837761i32, // 'BLID'
            &mut bID as *mut i32 as *mut c_void,
            std::mem::size_of::<i32>() as i32,
        );

        // Save out the block's flags
        flags = (*block).GetFlags();
        ((*self.m_owner).GetInterface()).I_WriteSaveData(
            1111837730i32, // 'BFLG'
            &mut flags as *mut u8 as *mut c_void,
            std::mem::size_of::<u8>() as i32,
        );

        // Save out the number of members to read
        numMembers = (*block).GetNumMembers();
        ((*self.m_owner).GetInterface()).I_WriteSaveData(
            1111837758i32, // 'BNUM'
            &mut numMembers as *mut i32 as *mut c_void,
            std::mem::size_of::<i32>() as i32,
        );

        for i in 0..numMembers {
            bm = (*block).GetMember(i as usize);

            // Save the block id
            bID = (*bm).GetID();
            ((*self.m_owner).GetInterface()).I_WriteSaveData(
                1111837773i32, // 'BMID'
                &mut bID as *mut i32 as *mut c_void,
                std::mem::size_of::<i32>() as i32,
            );

            // Save out the data size
            size = (*bm).GetSize();
            ((*self.m_owner).GetInterface()).I_WriteSaveData(
                1111837787i32, // 'BSIZ'
                &mut size as *mut i32 as *mut c_void,
                std::mem::size_of::<i32>() as i32,
            );

            // Save out the raw data
            ((*self.m_owner).GetInterface()).I_WriteSaveData(
                1111837773i32, // 'BMEM'
                (*bm).GetData(),
                size,
            );
        }

        true as i32
    }

    /*
    -------------------------
    Save
    -------------------------
    */

    pub fn Save(&mut self) {
        // #if 0
        // (disabled save/load implementation)
        // #endif
    }

    /*
    -------------------------
    Load
    -------------------------
    */

    pub fn Load(&mut self) {
        // #if 0
        // (disabled save/load implementation)
        // #endif
    }
}

// Helper function stubs for sprintf - these would normally come from libc
extern "C" {
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> i32;
}
