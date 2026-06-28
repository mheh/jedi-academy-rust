// Script Command Sequences
//
//	-- jweier

#![allow(non_snake_case)]

use core::ffi::c_int;
use crate::codemp::icarus::sequence_h::{CSequence, CBlock, SQ_COMMON, SQ_RETAIN, SQ_PENDING};
use crate::codemp::icarus::blockstream_h::{POP_FRONT, POP_BACK, PUSH_FRONT, PUSH_BACK, CBlockMember};

extern "C" {
	fn ICARUS_Malloc(size: usize) -> *mut core::ffi::c_void;
	fn ICARUS_Free(ptr: *mut core::ffi::c_void);
}

// ---- CSequence Implementation ----

impl CSequence {
	pub fn new() -> Self {
		CSequence {
			m_numCommands: 0,
			m_flags: 0,
			m_iterations: 1,
			m_parent: std::ptr::null_mut(),
			m_return: std::ptr::null_mut(),
			m_children: Vec::new(),
			m_commands: Vec::new(),
			m_id: 0,
			m_owner: std::ptr::null_mut(),
		}
	}
}

/*
-------------------------
Create
-------------------------
*/

impl CSequence {
	pub fn Create() -> *mut CSequence {
		let seq = Box::into_raw(Box::new(CSequence::new()));

		//TODO: Emit warning
		if seq.is_null() {
			return std::ptr::null_mut();
		}

		unsafe {
			(*seq).SetFlag(SQ_COMMON);
		}

		seq
	}
}

/*
-------------------------
Delete
-------------------------
*/

impl CSequence {
	pub fn Delete(&mut self) {
		//Notify the parent of the deletion
		if !self.m_parent.is_null() {
			unsafe {
				(*self.m_parent).RemoveChild(self as *mut CSequence);
			}
		}

		//Clear all children
		if self.m_children.len() > 0 {
			/*for ( iterSeq = m_childrenMap.begin(); iterSeq != m_childrenMap.end(); iterSeq++ )
			{
				(*iterSeq).second->SetParent( NULL );
			}*/

			for si in &self.m_children {
				unsafe {
					(*(*si)).SetParent(std::ptr::null_mut());
				}
			}
		}
		self.m_children.clear();

		//Clear all held commands
		for bi in &self.m_commands {
			unsafe {
				// Free the block (Free() handled internally)
				let _ = Box::from_raw(*bi);
			}
		}

		self.m_commands.clear();
	}
}


/*
-------------------------
AddChild
-------------------------
*/

impl CSequence {
	pub fn AddChild(&mut self, child: *mut CSequence) {
		if child.is_null() {
			return;
		}

		self.m_children.push(child);
	}
}

/*
-------------------------
RemoveChild
-------------------------
*/

impl CSequence {
	pub fn RemoveChild(&mut self, child: *mut CSequence) {
		if child.is_null() {
			return;
		}

		//Remove the child
		self.m_children.retain(|&c| !std::ptr::eq(c, child));
	}
}

/*
-------------------------
HasChild
-------------------------
*/

impl CSequence {
	pub fn HasChild(&self, sequence: *mut CSequence) -> bool {
		for ci in &self.m_children {
			if std::ptr::eq(*ci, sequence) {
				return true;
			}

			unsafe {
				if (*(*ci)).HasChild(sequence) {
					return true;
				}
			}
		}

		false
	}
}

/*
-------------------------
SetParent
-------------------------
*/

impl CSequence {
	pub fn SetParent(&mut self, parent: *mut CSequence) {
		self.m_parent = parent;

		if parent.is_null() {
			return;
		}

		//Inherit the parent's properties (this avoids messy tree walks later on)
		unsafe {
			if (*parent).m_flags & SQ_RETAIN != 0 {
				self.m_flags |= SQ_RETAIN;
			}

			if (*parent).m_flags & SQ_PENDING != 0 {
				self.m_flags |= SQ_PENDING;
			}
		}
	}
}

/*
-------------------------
PopCommand
-------------------------
*/

impl CSequence {
	pub fn PopCommand(&mut self, type_: c_int) -> *mut CBlock {
		let mut command: *mut CBlock = std::ptr::null_mut();

		//Make sure everything is ok
		assert!((type_ == POP_FRONT) || (type_ == POP_BACK));

		if self.m_commands.is_empty() {
			return std::ptr::null_mut();
		}

		match type_ {
		POP_FRONT => {
			command = self.m_commands.remove(0);
			self.m_numCommands -= 1;

			return command;
		},

		POP_BACK => {
			command = self.m_commands.pop().unwrap_or(std::ptr::null_mut());
			self.m_numCommands -= 1;

			return command;
		},

		_ => {
			//Invalid flag
			return std::ptr::null_mut();
		}
		}
	}
}

/*
-------------------------
PushCommand
-------------------------
*/

impl CSequence {
	pub fn PushCommand(&mut self, block: *mut CBlock, type_: c_int) -> c_int {
		//Make sure everything is ok
		assert!((type_ == PUSH_FRONT) || (type_ == PUSH_BACK));
		assert!(!block.is_null());

		match type_ {
		PUSH_FRONT => {
			self.m_commands.insert(0, block);
			self.m_numCommands += 1;

			return 1;
		},

		PUSH_BACK => {
			self.m_commands.push(block);
			self.m_numCommands += 1;

			return 1;
		},

		_ => {
			//Invalid flag
			return 0;
		}
		}
	}
}

/*
-------------------------
SetFlag
-------------------------
*/

impl CSequence {
	pub fn SetFlag(&mut self, flag: c_int) {
		self.m_flags |= flag;
	}
}

/*
-------------------------
RemoveFlag
-------------------------
*/

impl CSequence {
	pub fn RemoveFlag(&mut self, flag: c_int, children: bool) {
		self.m_flags &= !flag;

		if children {
			let mut children_copy = self.m_children.clone();

			for si in &mut children_copy {
				unsafe {
					(*(*si)).RemoveFlag(flag, true);
				}
			}
		}
	}
}

/*
-------------------------
HasFlag
-------------------------
*/

impl CSequence {
	pub fn HasFlag(&self, flag: c_int) -> c_int {
		return (self.m_flags & flag);
	}
}

/*
-------------------------
SetReturn
-------------------------
*/

impl CSequence {
	pub fn SetReturn(&mut self, sequence: *mut CSequence) {
		assert!(!std::ptr::eq(sequence, self as *mut CSequence));
		self.m_return = sequence;
	}
}

/*
-------------------------
GetChild
-------------------------
*/

impl CSequence {
	pub fn GetChildByIndex(&self, iIndex: c_int) -> *mut CSequence {
		if iIndex < 0 || iIndex >= (self.m_children.len() as c_int) {
			return std::ptr::null_mut();
		}

		let mut iterSeq = self.m_children.iter();
		for _i in 0..iIndex {
			iterSeq.next();
		}
		match iterSeq.next() {
			Some(seq) => *seq,
			None => std::ptr::null_mut(),
		}
	}
}

/*
-------------------------
SaveCommand
-------------------------
*/

impl CSequence {
	pub fn SaveCommand(&mut self, block: *mut CBlock) -> c_int {
		let mut flags: u8;
		let mut numMembers: c_int;
		let mut bID: c_int;
		let mut size: c_int;
		let mut bm: *mut CBlockMember;

		//Save out the block ID
		bID = unsafe { (*block).GetBlockID() };
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'BLID' as i32, &bID as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Save out the block's flags
		flags = unsafe { (*block).GetFlags() };
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'BFLG' as i32, &flags as *const _ as *const core::ffi::c_void, std::mem::size_of::<u8>());
		}

		//Save out the number of members to read
		numMembers = unsafe { (*block).GetNumMembers() };
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'BNUM' as i32, &numMembers as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		for i in 0..numMembers {
			bm = unsafe { (*block).GetMember(i) };

			//Save the block id
			bID = unsafe { (*bm).GetID() };
			unsafe {
				((*self.m_owner).GetInterface()).I_WriteSaveData(b'BMID' as i32, &bID as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
			}

			//Save out the data size
			size = unsafe { (*bm).GetSize() };
			unsafe {
				((*self.m_owner).GetInterface()).I_WriteSaveData(b'BSIZ' as i32, &size as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
			}

			//Save out the raw data
			unsafe {
				((*self.m_owner).GetInterface()).I_WriteSaveData(b'BMEM' as i32, (*bm).GetData(), size as usize);
			}
		}

		return 1;
	}
}

/*
-------------------------
Save
-------------------------
*/

impl CSequence {
	pub fn Save(&mut self) -> c_int {
		// piss off, stupid function
		/*
		let mut ci: sequence_l::iterator;
		let mut bi: block_l::iterator;
		let mut id: c_int;

		//Save the parent (by GUID)
		id = if !self.m_parent.is_null() { unsafe { (*self.m_parent).GetID() } } else { -1 };
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'SPID' as i32, &id as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Save the return (by GUID)
		id = if !self.m_return.is_null() { unsafe { (*self.m_return).GetID() } } else { -1 };
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'SRID' as i32, &id as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Save the number of children
	//	(m_owner->GetInterface())->I_WriteSaveData( 'SNCH', &m_numChildren, sizeof( m_numChildren ) );

		//Save out the children (only by GUID)
		for ci in &self.m_children {
			id = unsafe { (*(*ci)).GetID() };
			unsafe {
				((*self.m_owner).GetInterface()).I_WriteSaveData(b'SCHD' as i32, &id as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
			}
		}

		//Save flags
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'SFLG' as i32, &self.m_flags as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Save iterations
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'SITR' as i32, &self.m_iterations as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Save the number of commands
		unsafe {
			((*self.m_owner).GetInterface()).I_WriteSaveData(b'SNMC' as i32, &self.m_numCommands as *const _ as *const core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Save the commands
		for bi in &self.m_commands {
			self.SaveCommand(*bi);
		}

		return 1;
		*/
		return 0;
	}
}

/*
-------------------------
Load
-------------------------
*/

impl CSequence {
	pub fn Load(&mut self) -> c_int {
		// piss off, stupid function
		/*
		let mut flags: u8;
		let mut sequence: *mut CSequence;
		let mut block: *mut CBlock;
		let mut id: c_int;
		let mut numMembers: c_int;

		let mut bID: c_int;
		let mut bSize: c_int;
		let mut bData: *mut core::ffi::c_void;

		//Get the parent sequence
		unsafe {
			((*self.m_owner).GetInterface()).I_ReadSaveData(b'SPID' as i32, &mut id as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
		}
		self.m_parent = if id != -1 { unsafe { (*self.m_owner).GetSequence(id) } } else { std::ptr::null_mut() };

		//Get the return sequence
		unsafe {
			((*self.m_owner).GetInterface()).I_ReadSaveData(b'SRID' as i32, &mut id as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
		}
		self.m_return = if id != -1 { unsafe { (*self.m_owner).GetSequence(id) } } else { std::ptr::null_mut() };

		//Get the number of children
	//	(m_owner->GetInterface())->I_ReadSaveData( 'SNCH', &m_numChildren, sizeof( m_numChildren ) );

		//Reload all children
		for _i in 0..0 { // m_numChildren would be used here
			//Get the child sequence ID
			unsafe {
				((*self.m_owner).GetInterface()).I_ReadSaveData(b'SCHD' as i32, &mut id as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
			}

			//Get the desired sequence
			sequence = unsafe { (*self.m_owner).GetSequence(id) };
			if sequence.is_null() {
				return 0;
			}

			//Insert this into the list
			self.m_children.push(sequence);

			//Restore the connection in the child / ID map
	//		m_childrenMap[ i ] = sequence;
		}


		//Get the sequence flags
		unsafe {
			((*self.m_owner).GetInterface()).I_ReadSaveData(b'SFLG' as i32, &mut self.m_flags as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Get the number of iterations
		unsafe {
			((*self.m_owner).GetInterface()).I_ReadSaveData(b'SITR' as i32, &mut self.m_iterations as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		let mut numCommands: c_int;

		//Get the number of commands
		unsafe {
			((*self.m_owner).GetInterface()).I_ReadSaveData(b'SNMC' as i32, &mut numCommands as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
		}

		//Get all the commands
		for i in 0..numCommands {
			//Get the block ID and create a new container
			unsafe {
				((*self.m_owner).GetInterface()).I_ReadSaveData(b'BLID' as i32, &mut id as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
			}
			block = Box::into_raw(Box::new(CBlock::new()));

			unsafe {
				(*block).Create(id);
			}

			//Read the block's flags
			unsafe {
				((*self.m_owner).GetInterface()).I_ReadSaveData(b'BFLG' as i32, &mut flags as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<u8>());
				(*block).SetFlags(flags);
			}

			//Get the number of block members
			unsafe {
				((*self.m_owner).GetInterface()).I_ReadSaveData(b'BNUM' as i32, &mut numMembers as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
			}

			for j in 0..numMembers {
				//Get the member ID
				unsafe {
					((*self.m_owner).GetInterface()).I_ReadSaveData(b'BMID' as i32, &mut bID as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
				}

				//Get the member size
				unsafe {
					((*self.m_owner).GetInterface()).I_ReadSaveData(b'BSIZ' as i32, &mut bSize as *mut _ as *mut core::ffi::c_void, std::mem::size_of::<c_int>());
				}

				//Get the member's data
				bData = unsafe { ICARUS_Malloc(bSize as usize) };
				if bData.is_null() {
					return 0;
				}

				//Get the actual raw data
				unsafe {
					((*self.m_owner).GetInterface()).I_ReadSaveData(b'BMEM' as i32, bData, bSize as usize);
				}

				//Write out the correct type
				match bID {
				TK_INT => {
					assert!(false);
					let data = unsafe { *(bData as *const c_int) };
					unsafe {
						(*block).Write(TK_FLOAT, data as f32);
					}
				},

				TK_FLOAT => {
					unsafe {
						(*block).Write(TK_FLOAT, *(bData as *const f32));
					}
				},

				TK_STRING | TK_IDENTIFIER | TK_CHAR => {
					unsafe {
						(*block).Write(TK_STRING, bData as *const core::ffi::c_char);
					}
				},

				TK_VECTOR | TK_VECTOR_START => {
					unsafe {
						(*block).Write(TK_VECTOR, *(bData as *const [f32; 3]));
					}
				},

				ID_TAG => {
					unsafe {
						(*block).Write(ID_TAG, ID_TAG as f32);
					}
				},

				ID_GET => {
					unsafe {
						(*block).Write(ID_GET, ID_GET as f32);
					}
				},

				ID_RANDOM => {
					unsafe {
						(*block).Write(ID_RANDOM, *(bData as *const f32));
					}
				},

				TK_EQUALS | TK_GREATER_THAN | TK_LESS_THAN | TK_NOT => {
					unsafe {
						(*block).Write(bID, 0);
					}
				},

				_ => {
					assert!(false);
					unsafe {
						ICARUS_Free(bData);
					}
					return 0;
				},
				}

				//Get rid of the temp memory
				unsafe {
					ICARUS_Free(bData);
				}
			}

			//Save the block
			self.PushCommand(block, PUSH_BACK);
		}

		return 1;
		*/
		return 0;
	}
}
