// Task Manager header file

use core::ffi::{c_char, c_int, c_uint, c_void, c_ulong};

pub type DWORD = c_ulong;

pub const MAX_TASK_NAME: usize = 64;
pub const TASKFLAG_NORMAL: c_int = 0x00000000;
pub const RUNAWAY_LIMIT: c_int = 256;

#[repr(C)]
pub enum TaskReturnType {
    TASK_RETURN_COMPLETE = 0,
    TASK_RETURN_FAILED = 1,
}

#[repr(C)]
pub enum TaskStatus {
    TASK_OK = 0,
    TASK_FAILED = 1,
    TASK_START = 2,
    TASK_END = 3,
}

// Forward declarations for external/unported types
#[repr(C)]
pub struct CBlock;

#[repr(C)]
pub struct CSequencer;

#[repr(C)]
pub struct CIcarus;

#[repr(C)]
pub struct IGameInterface;

#[repr(C)]
pub struct vec3_t;

// External method stubs for opaque types
extern "C" {
    fn CBlock_GetBlockID(block: *mut CBlock) -> c_int;
}

// C++ STL container types - declared as extern types to indicate opaque C++ STL layouts
extern "C" {
    pub type taskCallback_m;
    pub type taskID_m;
    pub type taskGroupName_m;
    pub type taskGroupID_m;
    pub type taskGroup_v;
    pub type tasks_l;
}

// CTask

#[repr(C)]
pub struct CTask {
    pub m_id: c_int,
    pub m_timeStamp: DWORD,
    pub m_block: *mut CBlock,
}

#[allow(non_snake_case)]
impl CTask {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn Create(GUID: c_int, block: *mut CBlock) -> *mut CTask {
        unsafe { core::mem::zeroed() }
    }

    pub fn Free(&mut self) {}

    pub fn GetTimeStamp(&self) -> DWORD {
        self.m_timeStamp
    }

    pub fn GetBlock(&self) -> *mut CBlock {
        self.m_block
    }

    pub fn GetGUID(&self) -> c_int {
        self.m_id
    }

    pub fn GetID(&self) -> c_int {
        // return m_block->GetBlockID();
        // SAFETY: Assumes m_block is valid and initialized.
        unsafe { CBlock_GetBlockID(self.m_block) }
    }

    pub fn SetTimeStamp(&mut self, timeStamp: DWORD) {
        self.m_timeStamp = timeStamp;
    }

    pub fn SetBlock(&mut self, block: *mut CBlock) {
        self.m_block = block;
    }

    pub fn SetGUID(&mut self, id: c_int) {
        self.m_id = id;
    }

    // Overloaded new operator.
    // Allocate the memory.
    pub fn malloc(size: usize) -> *mut c_void {
        unsafe { core::mem::zeroed() }
    }

    // Overloaded delete operator.
    // Free the Memory.
    pub fn free(pRawData: *mut c_void) {}
}

// CTaskGroup

#[repr(C)]
pub struct CTaskGroup {
    pub m_completedTasks: taskCallback_m,

    pub m_parent: *mut CTaskGroup,

    pub m_numCompleted: c_uint,
    pub m_GUID: c_int,
}

#[allow(non_snake_case)]
impl CTaskGroup {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn Init(&mut self) {}

    pub fn Add(&mut self, task: *mut CTask) -> c_int {
        0
    }

    pub fn SetGUID(&mut self, GUID: c_int) {}

    pub fn SetParent(&mut self, group: *mut CTaskGroup) {
        self.m_parent = group;
    }

    pub fn Complete(&self) -> bool {
        // return ( m_numCompleted == m_completedTasks.size() );
        false
    }

    pub fn MarkTaskComplete(&mut self, id: c_int) -> bool {
        false
    }

    pub fn GetParent(&self) -> *mut CTaskGroup {
        self.m_parent
    }

    pub fn GetGUID(&self) -> c_int {
        self.m_GUID
    }

    // Overloaded new operator.
    // Allocate the memory.
    pub fn malloc(size: usize) -> *mut c_void {
        unsafe { core::mem::zeroed() }
    }

    // Overloaded delete operator.
    // Free the Memory.
    pub fn free(pRawData: *mut c_void) {}
}

// CTaskManager
// class CSequencer;

#[repr(C)]
pub struct CTaskManager {
    pub m_owner: *mut CSequencer,
    pub m_ownerID: c_int,

    pub m_curGroup: *mut CTaskGroup,

    pub m_taskGroups: taskGroup_v,
    pub m_tasks: tasks_l,

    pub m_GUID: c_int,
    pub m_count: c_int,

    pub m_taskGroupNameMap: taskGroupName_m,
    pub m_taskGroupIDMap: taskGroupID_m,

    pub m_resident: bool,

    pub m_id: c_int,

    // CTask	*m_waitTask;		//Global pointer to the current task that is waiting for callback completion
}

#[allow(non_snake_case)]
impl CTaskManager {
    pub fn new() -> Self {
        unsafe { core::mem::zeroed() }
    }

    pub fn GetID(&self) -> c_int {
        0
    }

    pub fn Create() -> *mut CTaskManager {
        unsafe { core::mem::zeroed() }
    }

    pub fn GetCurrentTask(&self) -> *mut CBlock {
        core::ptr::null_mut()
    }

    pub fn Init(&mut self, owner: *mut CSequencer) -> c_int {
        0
    }

    pub fn Free(&mut self) -> c_int {
        0
    }

    pub fn Flush(&mut self) -> c_int {
        0
    }

    pub fn SetCommand(
        &mut self,
        block: *mut CBlock,
        type_: c_int,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn Completed(&mut self, id: c_int) -> c_int {
        0
    }

    pub fn Update(&mut self, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn IsRunning(&self) -> c_int {
        0
    }

    pub fn IsResident(&self) -> bool {
        self.m_resident
    }

    pub fn AddTaskGroup(
        &mut self,
        name: *const c_char,
        icarus: *mut CIcarus,
    ) -> *mut CTaskGroup {
        core::ptr::null_mut()
    }

    pub fn GetTaskGroup(
        &mut self,
        name: *const c_char,
        icarus: *mut CIcarus,
    ) -> *mut CTaskGroup {
        core::ptr::null_mut()
    }

    pub fn GetTaskGroupById(
        &mut self,
        id: c_int,
        icarus: *mut CIcarus,
    ) -> *mut CTaskGroup {
        core::ptr::null_mut()
    }

    pub fn MarkTask(
        &mut self,
        id: c_int,
        operation: c_int,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn RecallTask(&mut self) -> *mut CBlock {
        core::ptr::null_mut()
    }

    pub fn Save(&mut self) {}

    pub fn Load(&mut self, icarus: *mut CIcarus) {}

    // Overloaded new operator.
    // Allocate the memory.
    pub fn malloc(size: usize) -> *mut c_void {
        unsafe { core::mem::zeroed() }
    }

    // Overloaded delete operator.
    // Free the Memory.
    pub fn free(pRawData: *mut c_void) {}

    // protected:

    pub fn Go(&mut self, icarus: *mut CIcarus) -> c_int {
        //Heartbeat function called once per game frame
        0
    }

    pub fn CallbackCommand(
        &mut self,
        task: *mut CTask,
        returnCode: c_int,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn Check(&self, targetID: c_int, block: *mut CBlock, memberNum: c_int) -> bool {
        false
    }

    pub fn GetVector(
        &mut self,
        entID: c_int,
        block: *mut CBlock,
        memberNum: *mut c_int,
        value: *mut vec3_t,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn GetFloat(
        &mut self,
        entID: c_int,
        block: *mut CBlock,
        memberNum: *mut c_int,
        value: *mut f32,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn Get(
        &mut self,
        entID: c_int,
        block: *mut CBlock,
        memberNum: *mut c_int,
        value: *mut *mut c_char,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn PushTask(&mut self, task: *mut CTask, flag: c_int) -> c_int {
        0
    }

    pub fn PopTask(&mut self, flag: c_int) -> *mut CTask {
        core::ptr::null_mut()
    }

    // Task functions
    pub fn Rotate(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Remove(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Camera(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Print(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Sound(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Move(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Kill(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Set(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Use(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn DeclareVariable(
        &mut self,
        task: *mut CTask,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn FreeVariable(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Signal(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Play(&mut self, task: *mut CTask, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn Wait(&mut self, task: *mut CTask, completed: *mut bool, icarus: *mut CIcarus) -> c_int {
        0
    }

    pub fn WaitSignal(
        &mut self,
        task: *mut CTask,
        completed: *mut bool,
        icarus: *mut CIcarus,
    ) -> c_int {
        0
    }

    pub fn SaveCommand(&mut self, block: *mut CBlock) -> c_int {
        0
    }
}
