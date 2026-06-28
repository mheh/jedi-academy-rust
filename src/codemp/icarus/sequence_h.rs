// Sequence Header File

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use std::collections::HashMap;

// Forward declarations
pub struct ICARUS_Instance;
pub struct CBlock;

extern "C" {
	fn Z_Malloc(size: usize, tag: c_int, zero: c_int) -> *mut c_void;
	fn Z_Free(ptr: *mut c_void);
}

pub const TAG_ICARUS3: c_int = 18;
pub const qtrue: c_int = 1;

// typedef list < CSequence * >	sequence_l
pub type sequence_l = Vec<*mut CSequence>;

// typedef	map	< int, CSequence *> sequenceID_m
pub type sequenceID_m = HashMap<c_int, *mut CSequence>;

// typedef list < CBlock * >		block_l
pub type block_l = Vec<*mut CBlock>;

pub struct CSequence {
	// Organization information
	pub m_children: sequence_l,
	pub m_parent: *mut CSequence,
	pub m_return: *mut CSequence,

	// Data information
	pub m_commands: block_l,
	pub m_flags: c_int,
	pub m_iterations: c_int,
	pub m_id: c_int,
	pub m_numCommands: c_int,

	pub m_owner: *mut ICARUS_Instance,
}

impl CSequence {
	//Constructors / Destructors
	pub fn new() -> Self {
		CSequence {
			m_children: Vec::new(),
			m_parent: std::ptr::null_mut(),
			m_return: std::ptr::null_mut(),
			m_commands: Vec::new(),
			m_flags: 0,
			m_iterations: 0,
			m_id: 0,
			m_numCommands: 0,
			m_owner: std::ptr::null_mut(),
		}
	}

	// Allocate the memory through custom allocator
	pub fn operator_new(size: usize) -> *mut c_void {
		unsafe { Z_Malloc(size, TAG_ICARUS3, qtrue) }
	}

	// Free the Memory through custom allocator
	pub fn operator_delete(pRawData: *mut c_void) {
		unsafe { Z_Free(pRawData); }
	}

	//Creation and deletion
	pub fn Create() -> *mut CSequence {
		todo!()
	}

	pub fn Delete(&mut self) {
		todo!()
	}

	//Organization functions
	pub fn AddChild(&mut self, sequence: *mut CSequence) {
		todo!()
	}

	pub fn RemoveChild(&mut self, sequence: *mut CSequence) {
		todo!()
	}

	pub fn SetParent(&mut self, parent: *mut CSequence) {
		todo!()
	}

	pub fn GetParent(&self) -> *mut CSequence {
		self.m_parent
	}

	//Block manipulation
	pub fn PopCommand(&mut self, index: c_int) -> *mut CBlock {
		todo!()
	}

	pub fn PushCommand(&mut self, block: *mut CBlock, index: c_int) -> c_int {
		todo!()
	}

	//Flag utilties
	pub fn SetFlag(&mut self, flag: c_int) {
		todo!()
	}

	pub fn RemoveFlag(&mut self, flag: c_int, bForce: bool) {
		todo!()
	}

	pub fn HasFlag(&self, flag: c_int) -> c_int {
		todo!()
	}

	pub fn GetFlags(&self) -> c_int {
		self.m_flags
	}

	pub fn SetFlags(&mut self, flags: c_int) {
		self.m_flags = flags;
	}

	//Various encapsulation utilities
	pub fn GetIterations(&self) -> c_int {
		self.m_iterations
	}

	pub fn SetIterations(&mut self, it: c_int) {
		self.m_iterations = it;
	}

	pub fn GetID(&self) -> c_int {
		self.m_id
	}

	pub fn SetID(&mut self, id: c_int) {
		self.m_id = id;
	}

	pub fn GetReturn(&self) -> *mut CSequence {
		self.m_return
	}

	pub fn SetReturn(&mut self, sequence: *mut CSequence) {
		todo!()
	}

	pub fn GetNumCommands(&self) -> c_int {
		self.m_numCommands
	}

	pub fn GetNumChildren(&self) -> c_int {
		self.m_children.len() as c_int
	}

	pub fn GetChildByIndex(&mut self, id: c_int) -> *mut CSequence {
		todo!()
	}

	pub fn HasChild(&self, sequence: *mut CSequence) -> bool {
		todo!()
	}

	pub fn SetOwner(&mut self, owner: *mut ICARUS_Instance) {
		self.m_owner = owner;
	}

	pub fn Save(&mut self) -> c_int {
		todo!()
	}

	pub fn Load(&mut self) -> c_int {
		todo!()
	}

	pub fn SaveCommand(&mut self, block: *mut CBlock) -> c_int {
		todo!()
	}
}
