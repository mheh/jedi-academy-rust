////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// KD Tree
// -------
//
//
//
// NOTES:
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

use core::ffi::c_int;
use std::marker::PhantomData;

// PORT: Local stubs for external ratl types
// These are declared here for structural coherence per the porting style guide

pub mod ratl {
	/// Stub for ratl_base - empty base class equivalent
	pub struct ratl_base;

	/// Stub for vector_vs - sized vector equivalent
	pub struct vector_vs<T: Copy, const SIZE: usize> {
		data: std::vec::Vec<T>,
	}

	impl<T: Copy + Default, const SIZE: usize> vector_vs<T, SIZE> {
		pub fn new() -> Self {
			vector_vs {
				data: std::vec::Vec::new(),
			}
		}

		pub fn clear(&mut self) {
			self.data.clear();
		}

		pub fn push_back(&mut self, value: T) {
			self.data.push(value);
		}

		pub fn size(&self) -> usize {
			self.data.len()
		}
	}

	impl<T: Copy, const SIZE: usize> Default for vector_vs<T, SIZE> {
		fn default() -> Self {
			Self::new()
		}
	}

	/// Stub for handle_pool_vs - handle-based pool allocator
	pub struct handle_pool_vs<T: Copy, const SIZE: usize> {
		data: std::vec::Vec<T>,
		free_indices: std::vec::Vec<i32>,
	}

	impl<T: Copy + Default, const SIZE: usize> handle_pool_vs<T, SIZE> {
		pub fn new() -> Self {
			let mut free_indices = std::vec::Vec::new();
			for i in (0..SIZE as i32).rev() {
				free_indices.push(i);
			}
			handle_pool_vs {
				data: std::vec::vec![T::default(); SIZE],
				free_indices,
			}
		}

		pub fn alloc(&mut self) -> i32 {
			if let Some(idx) = self.free_indices.pop() {
				idx
			} else {
				-1 // Out of memory
			}
		}

		pub fn free(&mut self, idx: i32) {
			if idx >= 0 && (idx as usize) < SIZE {
				self.free_indices.push(idx);
			}
		}

		pub fn size(&self) -> usize {
			SIZE - self.free_indices.len()
		}

		pub fn full(&self) -> bool {
			self.free_indices.is_empty()
		}

		pub fn clear(&mut self) {
			self.data = std::vec::vec![T::default(); SIZE];
			self.free_indices.clear();
			for i in (0..SIZE as i32).rev() {
				self.free_indices.push(i);
			}
		}
	}

	impl<T: Copy, const SIZE: usize> std::ops::Index<i32> for handle_pool_vs<T, SIZE> {
		type Output = T;

		fn index(&self, idx: i32) -> &T {
			&self.data[idx as usize]
		}
	}

	impl<T: Copy, const SIZE: usize> std::ops::IndexMut<i32> for handle_pool_vs<T, SIZE> {
		fn index_mut(&mut self, idx: i32) -> &mut T {
			&mut self.data[idx as usize]
		}
	}

	impl<T: Copy, const SIZE: usize> std::ops::Index<usize> for handle_pool_vs<T, SIZE> {
		type Output = T;

		fn index(&self, idx: usize) -> &T {
			&self.data[idx]
		}
	}

	impl<T: Copy, const SIZE: usize> std::ops::IndexMut<usize> for handle_pool_vs<T, SIZE> {
		fn index_mut(&mut self, idx: usize) -> &mut T {
			&mut self.data[idx]
		}
	}

	impl<T: Copy + Default, const SIZE: usize> Default for handle_pool_vs<T, SIZE> {
		fn default() -> Self {
			Self::new()
		}
	}
}

// PORT: Local stub for ESide enum used in kdtree_vs
// These values are referenced in the tree search operations
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ESide {
	Side_AllIn,
	Side_In,
	Side_None,
}

////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone)]
struct node<T: Copy> {
	mParent: i32,
	mLeft: i32,
	mRight: i32,

	mData: T,
}

impl<T: Copy + Default> Default for node<T> {
	fn default() -> Self {
		node {
			mParent: 0,
			mLeft: 0,
			mRight: 0,
			mData: T::default(),
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone)]
struct range_bounds<const DIMENSION: usize> {
	mMins: [i32; DIMENSION],
	mMaxs: [i32; DIMENSION],
}

impl<const DIMENSION: usize> range_bounds<DIMENSION> {
	fn new() -> Self {
		range_bounds {
			mMins: [0; DIMENSION],
			mMaxs: [0; DIMENSION],
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////
pub struct range_query<T: Copy + Default, const SIZE: usize> {
	pub mReported: ratl::vector_vs<T, SIZE>,
	pub mMins: T,
	pub mMaxs: T,
}

impl<T: Copy + Default, const SIZE: usize> range_query<T, SIZE> {
	pub fn new() -> Self {
		range_query {
			mReported: ratl::vector_vs::new(),
			mMins: T::default(),
			mMaxs: T::default(),
		}
	}
}

impl<T: Copy + Default, const SIZE: usize> Default for range_query<T, SIZE> {
	fn default() -> Self {
		Self::new()
	}
}

////////////////////////////////////////////////////////////////////////////////////////
// The KD Tree Class
////////////////////////////////////////////////////////////////////////////////////////
pub struct kdtree_vs<T: Copy + Default + PartialOrd + std::ops::Index<usize, Output = f32>, const DIMENSION: usize, const SIZE: usize>
where
	T: Clone,
{
	////////////////////////////////////////////////////////////////////////////////////
	// Capacity Enum
	////////////////////////////////////////////////////////////////////////////////////
	// CAPACITY	= SIZE,
	// NULL_NODE	= SIZE+2,		// Invalid Node ID
	// TARG_NODE	= SIZE+3		// Used To Mark Nodes Add Location

	////////////////////////////////////////////////////////////////////////////////////
	// Data
	////////////////////////////////////////////////////////////////////////////////////
	mPool: ratl::handle_pool_vs<node<T>, SIZE>,
	// The Allocation Data Pool
	mRoot: i32,
	// The Beginning Of The Tree
}

impl<T: Copy + Default + PartialOrd + std::ops::Index<usize, Output = f32> + Clone, const DIMENSION: usize, const SIZE: usize> kdtree_vs<T, DIMENSION, SIZE> {
	const CAPACITY: usize = SIZE;
	const NULL_NODE: i32 = (SIZE + 2) as i32;
	const TARG_NODE: i32 = (SIZE + 3) as i32;

	////////////////////////////////////////////////////////////////////////////////////
	// Constructor
	////////////////////////////////////////////////////////////////////////////////////
	pub fn new() -> Self {
		kdtree_vs {
			mPool: ratl::handle_pool_vs::new(),
			mRoot: Self::NULL_NODE,
		}
	}

	////////////////////////////////////////////////////////////////////////////////////
	// How Many Objects Are In This Tree
	////////////////////////////////////////////////////////////////////////////////////
	pub fn size(&self) -> i32 {
		self.mPool.size() as i32
	}

	////////////////////////////////////////////////////////////////////////////////////
	// Are There Any Objects In This Tree?
	////////////////////////////////////////////////////////////////////////////////////
	pub fn empty(&self) -> bool {
		self.mRoot == Self::NULL_NODE
	}

	////////////////////////////////////////////////////////////////////////////////////
	// Is This List Filled?
	////////////////////////////////////////////////////////////////////////////////////
	pub fn full(&self) -> bool {
		self.mPool.full()
	}

	////////////////////////////////////////////////////////////////////////////////////
	// Clear All Elements
	////////////////////////////////////////////////////////////////////////////////////
	pub fn clear(&mut self) {
		self.mRoot = Self::NULL_NODE;
		self.mPool.clear();
	}

	////////////////////////////////////////////////////////////////////////////////////
	// Add A New Element To The Tree
	////////////////////////////////////////////////////////////////////////////////////
	pub fn add(&mut self, data: T) {
		// CREATE: New
		//--------------------------------------------
		let nNew = self.mPool.alloc();
		self.mPool[nNew].mData = data;
		self.mPool[nNew].mLeft = Self::NULL_NODE;
		self.mPool[nNew].mRight = Self::NULL_NODE;

		// LINK: (nNew)->(Parent)
		//--------------------------------------------
		if self.mRoot == Self::NULL_NODE {
			self.mRoot = nNew;
			self.mPool[nNew].mParent = Self::NULL_NODE;
			return;
		}

		// LINK: (nNew)->(Parent)
		//--------------------------------------------
		let parent_idx = self.find_index(data, self.mRoot, 0, true, true);
		self.mPool[nNew].mParent = parent_idx;

		// LINK: (Parent)->(nNew)
		//--------------------------------------------
		let parent_node_idx = self.mPool[nNew].mParent as usize;
		if self.mPool[parent_node_idx].mLeft == Self::TARG_NODE {
			self.mPool[parent_node_idx].mLeft = nNew;
		} else if self.mPool[parent_node_idx].mRight == Self::TARG_NODE {
			self.mPool[parent_node_idx].mRight = nNew;
		}
		// Hey!  It Didn't Mark Any Targets, Which Means We Found An Exact match To This Data
		//------------------------------------------------------------------------------------
		else {
			self.mPool.free(nNew);
		}
	}

	////////////////////////////////////////////////////////////////////////////////////
	// Does (data) Exist In The Tree?
	////////////////////////////////////////////////////////////////////////////////////
	pub fn find(&self, data: T) -> bool {
		assert!(self.mRoot != Self::NULL_NODE); // If You Hit This Assert, You Are Asking For Data On An Empty Tree

		let node = self.find_index(data, self.mRoot, 0, true, true);

		// Exact Find, Or Found Root?
		//----------------------------
		if self.mPool[node as usize].mData == data || self.mPool[node as usize].mParent == Self::NULL_NODE {
			return true;
		}
		false
	}

	////////////////////////////////////////////////////////////////////////////////////
	//
	////////////////////////////////////////////////////////////////////////////////////
	pub fn find_range(&mut self, query: &mut range_query<T, SIZE>) {
		if self.mRoot != Self::NULL_NODE {
			query.mReported.clear();
			self.tree_search(query);
		}
	}

	////////////////////////////////////////////////////////////////////////////////////
	// This Private Function Of The Class Does A Standard Binary Tree Search
	////////////////////////////////////////////////////////////////////////////////////
	fn find_index(&self, data: T, cur_node: i32, cur_dimension: usize, return_closest: bool, mark_target: bool) -> i32 {
		// Did We Just Go Off The End Of The Tree Or Find The Data We Were Looking For?
		//------------------------------------------------------------------------------
		if cur_node == Self::NULL_NODE || self.mPool[cur_node as usize].mData == data {
			return cur_node;
		}

		// Calculate The Next Dimension For Searching
		//--------------------------------------------
		let mut next_dimension = cur_dimension + 1;
		if next_dimension >= DIMENSION {
			next_dimension = 0;
		}

		// Search Recursivly Down The Tree Either Left (For Data > Current Node), Or Right
		//---------------------------------------------------------------------------------
		let go_left = data[cur_dimension] < self.mPool[cur_node as usize].mData[cur_dimension];
		let find_recursive = if go_left {
			self.find_index(data, self.mPool[cur_node as usize].mLeft, next_dimension, return_closest, mark_target)
		} else {
			self.find_index(data, self.mPool[cur_node as usize].mRight, next_dimension, return_closest, mark_target)
		};

		// Success!
		//----------
		if find_recursive != Self::NULL_NODE {
			return find_recursive;
		}

		// If We Want To Return The CLOSEST Node, And We Went Off The End, Then Return This One
		//--------------------------------------------------------------------------------------
		if return_closest {
			// If We Are Asked To Mark The Target, We Mark (TARG_NODE) At Either mLeft or mRight,
			// Depending On Where The Node Should Have Been
			//----------------------------------------------------------------------------------
			if mark_target {
				if go_left {
					self.mPool[cur_node as usize].mLeft = Self::TARG_NODE;
				} else {
					self.mPool[cur_node as usize].mRight = Self::TARG_NODE;
				}
			}

			// Go Ahead And Return This Node, It's The One We Would Have Put As The Child
			return cur_node;
		}

		// Return The Results Of The Recursive Call
		//------------------------------------------
		Self::NULL_NODE
	}

	////////////////////////////////////////////////////////////////////////////////////
	// This function just sets up the range bounds and starts the recursive tree search
	////////////////////////////////////////////////////////////////////////////////////
	fn tree_search(&mut self, query: &mut range_query<T, SIZE>) {
		let bounds = range_bounds::<DIMENSION>::new();
		self.tree_search_impl(query, self.mRoot, 0, bounds);
	}

	////////////////////////////////////////////////////////////////////////////////////
	//
	////////////////////////////////////////////////////////////////////////////////////
	fn tree_search_impl(&mut self, query: &mut range_query<T, SIZE>, cur_node: i32, cur_dimension: usize, mut bounds: range_bounds<DIMENSION>) {
		assert!((cur_node as usize) < SIZE);

		// Is This Node In The Query Range?  If So, Report It
		//----------------------------------------------------
		if cur_node != Self::NULL_NODE && self.tree_search_node_in_range(query, &self.mPool[cur_node as usize]) {
			query.mReported.push_back(self.mPool[cur_node as usize].mData);
		}

		// If This Is A Leaf Node, We're Done Here
		//-----------------------------------------
		if cur_node == Self::NULL_NODE
			|| (self.mPool[cur_node as usize].mLeft == Self::NULL_NODE && self.mPool[cur_node as usize].mRight == Self::NULL_NODE)
		{
			return;
		}

		// Calculate The Next Dimension For Searching
		//--------------------------------------------
		let mut next_dimension = cur_dimension + 1;
		if next_dimension >= DIMENSION {
			next_dimension = 0;
		}

		// Test To See If Our Subtree Is In Range
		//----------------------------------------
		let side = self.tree_search_bounds_in_range(query, &bounds);

		// If The Bounds Are Contained Entirely Within The Query Range, We Report The Sub Tree
		//-------------------------------------------------------------------------------------
		if side == ESide::Side_AllIn {
			self.tree_search_report_sub_tree(query, cur_node);
		}
		// Otherwise, If Our Bounds Intersect The Query Range, We Need To Look Further
		//-----------------------------------------------------------------------------
		else if side == ESide::Side_In {
			// Test The Left Child
			//---------------------
			if self.mPool[cur_node as usize].mLeft != Self::NULL_NODE {
				let old_maxs = bounds.mMaxs[cur_dimension];
				if bounds.mMins[cur_dimension] == 0
					|| self.mPool[cur_node as usize].mData[cur_dimension]
						< self.mPool[bounds.mMins[cur_dimension] as usize].mData[cur_dimension]
				{
					bounds.mMins[cur_dimension] = cur_node;
				}
				self.tree_search_impl(query, self.mPool[cur_node as usize].mLeft, next_dimension, bounds);
				bounds.mMaxs[cur_dimension] = old_maxs; // Restore Old Maxs For The Right Child Search
			}

			// Test The Right Child
			//----------------------
			if self.mPool[cur_node as usize].mRight != Self::NULL_NODE {
				if bounds.mMaxs[cur_dimension] == 0
					|| self.mPool[bounds.mMaxs[cur_dimension] as usize].mData[cur_dimension]
						< self.mPool[cur_node as usize].mData[cur_dimension]
				{
					bounds.mMaxs[cur_dimension] = cur_node;
				}
				self.tree_search_impl(query, self.mPool[cur_node as usize].mRight, next_dimension, bounds);
			}
		}
	}

	////////////////////////////////////////////////////////////////////////////////////
	// This Function Returns True If The Node Is Within The Query Range
	////////////////////////////////////////////////////////////////////////////////////
	fn tree_search_node_in_range(&self, query: &range_query<T, SIZE>, n: &node<T>) -> bool {
		for dim in 0..DIMENSION {
			if n.mData[dim] < query.mMins[dim] || query.mMaxs[dim] < n.mData[dim] {
				return false;
			}
		}
		true
	}

	////////////////////////////////////////////////////////////////////////////////////
	//
	////////////////////////////////////////////////////////////////////////////////////
	fn tree_search_bounds_in_range(&self, query: &range_query<T, SIZE>, bounds: &range_bounds<DIMENSION>) -> ESide {
		let mut s = ESide::Side_AllIn;
		for dim in 0..DIMENSION {
			// If Any Of Our Dimensions Are Undefined Right Now, Always Return INTERSECT
			//---------------------------------------------------------------------------
			if bounds.mMaxs[dim] == 0 || bounds.mMins[dim] == 0 {
				return ESide::Side_In;
			}

			// Check To See If They Intersect At All?
			//----------------------------------------
			if (self.mPool[bounds.mMaxs[dim] as usize].mData[dim] < query.mMins[dim])
				|| (query.mMaxs[dim] < self.mPool[bounds.mMins[dim] as usize].mData[dim])
			{
				return ESide::Side_None;
			}

			// Check To See If It Is Contained
			//---------------------------------
			if (self.mPool[bounds.mMins[dim] as usize].mData[dim] < query.mMins[dim])
				|| (query.mMaxs[dim] < self.mPool[bounds.mMaxs[dim] as usize].mData[dim])
			{
				s = ESide::Side_In;
			}
		}
		s
	}

	////////////////////////////////////////////////////////////////////////////////////
	// Add The Cur Node And All Childeren Of The Cur Node
	////////////////////////////////////////////////////////////////////////////////////
	fn tree_search_report_sub_tree(&mut self, query: &mut range_query<T, SIZE>, cur_node: i32) {
		assert!((cur_node as usize) < SIZE);

		if self.mPool[cur_node as usize].mLeft != Self::NULL_NODE {
			query.mReported.push_back(self.mPool[self.mPool[cur_node as usize].mLeft as usize].mData);
			self.tree_search_report_sub_tree(query, self.mPool[cur_node as usize].mRight);
		}
		if self.mPool[cur_node as usize].mRight != Self::NULL_NODE {
			query.mReported.push_back(self.mPool[self.mPool[cur_node as usize].mRight as usize].mData);
			self.tree_search_report_sub_tree(query, self.mPool[cur_node as usize].mRight);
		}
	}
}

impl<T: Copy + Default + PartialOrd + std::ops::Index<usize, Output = f32> + Clone, const DIMENSION: usize, const SIZE: usize> Default
	for kdtree_vs<T, DIMENSION, SIZE>
{
	fn default() -> Self {
		Self::new()
	}
}
