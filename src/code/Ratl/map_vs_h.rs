// ////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
// (c) 2002 Activision
//
//
// Map
// ---
// This map is based on a red black tree, which guarentees balanced data, no mater what
// order elements are added. The map uses a memory pool for storage of node data.
//
//
// ////////////////////////////////////////////////////////////////////////////////////////

// ////////////////////////////////////////////////////////////////////////////////////////
// Includes
// ////////////////////////////////////////////////////////////////////////////////////////
// ratl_common.h and pool_vs.h included via module dependencies
// For this port, we provide minimal trait definitions inline

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

// ////////////////////////////////////////////////////////////////////////////////////////
// Local stubs for unported dependencies (ratl_common.h, pool_vs.h)
// ////////////////////////////////////////////////////////////////////////////////////////

// Placeholder traits/types from ratl_common.h and pool_vs.h
// These represent the interface contracts needed by the tree implementations.

pub trait TRatlNew {
    // Marker trait for allocation/construction
}

pub trait StorageTraits {
    type TValue;
    const CAPACITY: i32;
    fn node(val: &mut Self::TValue) -> &mut tree_node;
}

pub struct pool_base<T: StorageTraits> {
    // Placeholder: represents the memory pool
    // In a full port, this would contain allocation management
    _phantom: core::marker::PhantomData<T>,
}

impl<T: StorageTraits> pool_base<T> {
    fn size(&self) -> i32 {
        0 // Placeholder
    }

    fn empty(&self) -> bool {
        true // Placeholder
    }

    fn full(&self) -> bool {
        false // Placeholder
    }

    fn clear(&mut self) {
        // Placeholder
    }

    fn alloc(&mut self, _key: T::TValue) -> i32 {
        0 // Placeholder
    }

    fn alloc(&mut self) -> i32 {
        0 // Placeholder
    }

    fn alloc_raw(&mut self) -> *mut u8 {
        core::ptr::null_mut()
    }

    fn pointer_to_index(&self, _ptr: *mut u8) -> i32 {
        0 // Placeholder
    }

    fn free(&mut self, _idx: i32) {
        // Placeholder
    }

    fn verify_alloc<T>(&self, p: *mut T) -> *mut T {
        p
    }
}

impl<T: StorageTraits> core::ops::Index<i32> for pool_base<T> {
    type Output = T::TValue;

    fn index(&self, _idx: i32) -> &Self::Output {
        panic!("placeholder")
    }
}

impl<T: StorageTraits> core::ops::IndexMut<i32> for pool_base<T> {
    fn index_mut(&mut self, _idx: i32) -> &mut Self::Output {
        panic!("placeholder")
    }
}

pub struct array_base<T: StorageTraits> {
    _phantom: core::marker::PhantomData<T>,
}

impl<T: StorageTraits> array_base<T> {
    fn clear(&mut self) {}

    fn construct(&mut self, _idx: i32, _val: T::TValue) {}

    fn construct(&mut self, _idx: i32) {}

    fn alloc_raw(&mut self, _idx: i32) -> *mut u8 {
        core::ptr::null_mut()
    }

    fn destruct(&mut self, _idx: i32) {}

    fn verify_alloc<U>(&self, p: *mut U) -> *mut U {
        p
    }
}

impl<T: StorageTraits> core::ops::Index<i32> for array_base<T> {
    type Output = T::TValue;

    fn index(&self, _idx: i32) -> &Self::Output {
        panic!("placeholder")
    }
}

impl<T: StorageTraits> core::ops::IndexMut<i32> for array_base<T> {
    fn index_mut(&mut self, _idx: i32) -> &mut Self::Output {
        panic!("placeholder")
    }
}

pub fn compile_assert<const COND: bool>() {
    // Compile-time assertion via const generic
}

// ////////////////////////////////////////////////////////////////////////////////////////
// this is private to the set, but you have no access to it, soooo..
// ////////////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
pub struct tree_node {
    mParent: i32,
    mLeft: i32,
    mRight: i32,
}

impl tree_node {
    // to save space we are putting the red bool in a high bit
    // this is in the parent only
    const RED_BIT: i32 = 0x40000000;
    // this must not have the red bit set
    const NULL_NODE: i32 = 0x3fffffff;

    pub fn init(&mut self) {
        self.mLeft = tree_node::NULL_NODE;
        self.mRight = tree_node::NULL_NODE;
        self.mParent = tree_node::NULL_NODE | tree_node::RED_BIT;
    }

    pub fn left(&self) -> i32 {
        self.mLeft
    }

    pub fn right(&self) -> i32 {
        self.mRight
    }

    pub fn parent(&self) -> i32 {
        self.mParent & (!tree_node::RED_BIT)
    }

    pub fn red(&self) -> bool {
        (self.mParent & tree_node::RED_BIT) != 0
    }

    pub fn set_left(&mut self, l: i32) {
        self.mLeft = l;
    }

    pub fn set_right(&mut self, r: i32) {
        self.mRight = r;
    }

    pub fn set_parent(&mut self, p: i32) {
        self.mParent &= tree_node::RED_BIT;
        self.mParent |= p;
    }

    pub fn set_red(&mut self, isRed: bool) {
        if isRed {
            self.mParent |= tree_node::RED_BIT;
        } else {
            self.mParent &= !tree_node::RED_BIT;
        }
    }
}

// fixme void *, comparison function pointer-ize this for code bloat.

pub struct tree_base<T: StorageTraits, const IS_MULTI: i32> {
    mPool: pool_base<T>,
    mRoot: i32,
    mLastAdd: i32,
}

impl<T: StorageTraits, const IS_MULTI: i32> tree_base<T, IS_MULTI>
where
    T::TValue: Ord,
{
    // ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    // ////////////////////////////////////////////////////////////////////////////////////
    const CAPACITY: i32 = T::CAPACITY;

    fn link_left(&mut self, node: i32, left: i32) {
        T::node(&mut self.mPool[node]).set_left(left);
        if left != tree_node::NULL_NODE {
            T::node(&mut self.mPool[left]).set_parent(node);
        }
    }

    fn link_right(&mut self, node: i32, right: i32) {
        T::node(&mut self.mPool[node]).set_right(right);
        if right != tree_node::NULL_NODE {
            T::node(&mut self.mPool[right]).set_parent(node);
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive find function - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn find_internal(&self, key: &T::TValue, at: i32) -> i32 {
        // FAIL TO FIND?
        // ---------------
        if at == tree_node::NULL_NODE {
            return tree_node::NULL_NODE;
        }

        // Should We Search Left?
        // ------------------------
        if key < &self.mPool[at] {
            return self.find_internal(key, T::node(&self.mPool[at]).left());
        }

        // Should We Search Right?
        // ------------------------
        if self.mPool[at] < *key {
            return self.find_internal(key, T::node(&self.mPool[at]).right());
        }

        // FOUND!
        // --------
        return at;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive find function - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn find_internal_with_target(
        &self,
        key: &T::TValue,
        target: i32,
        at: i32,
        parent: &mut i32,
    ) -> i32 {
        // FAIL TO FIND?
        // ---------------
        if at == tree_node::NULL_NODE {
            *parent = tree_node::NULL_NODE;
            return tree_node::NULL_NODE;
        }

        // FOUND!
        // --------
        if at == target {
            if at == self.mRoot {
                *parent = tree_node::NULL_NODE;
            }
            return at;
        }

        // Should We Search Left?
        // ------------------------
        if key < &self.mPool[at] {
            *parent = at;
            return self.find_internal_with_target(
                key,
                target,
                T::node(&self.mPool[at]).left(),
                parent,
            );
        }

        *parent = at;
        return self.find_internal_with_target(
            key,
            target,
            T::node(&self.mPool[at]).right(),
            parent,
        );
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive insertion - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn insert_internal(&mut self, key: &T::TValue, at: &mut i32) -> i32 {
        // If At Is A NULL_NODE, We Have Found A Leaf.
        // ----------------------------------------------
        if *at == tree_node::NULL_NODE {
            if self.mRoot == tree_node::NULL_NODE {
                self.mRoot = self.mLastAdd;
            }
            return tree_node::NULL_NODE; // There Are No Excess Red Children (No Childeren At All, Actually)
        }

        let mut nxtChild: i32; // The Child We Will Eventually Add Underneath
        let mut altChild: i32; // The "other" Child
        let mut nxtRotateLeft: bool;
        let mut excessRedChild: i32; // If The Insert Results In An Excess Red Child, This Will Be It

        // Choose Which Side To Add The New Node Under
        // ---------------------------------------------
        if key < &self.mPool[*at] {
            // The Key Classes Must Support A < Operator
            let mut tmp = T::node(&self.mPool[*at]).left();
            excessRedChild = self.insert_internal(key, &mut tmp);
            self.link_left(*at, tmp); // T::node(mPool[*at]).set_left(tmp);

            if tmp == tree_node::NULL_NODE {
                self.link_left(*at, self.mLastAdd); // T::node(mPool[*at]).set_left(mLastAdd);
            }
            nxtChild = T::node(&self.mPool[*at]).left();
            altChild = T::node(&self.mPool[*at]).right();
            nxtRotateLeft = false;
        } else if self.mPool[*at] < *key {
            let mut tmp = T::node(&self.mPool[*at]).right();
            excessRedChild = self.insert_internal(key, &mut tmp);
            self.link_right(*at, tmp); // T::node(mPool[*at]).set_right(tmp);

            if tmp == tree_node::NULL_NODE {
                self.link_right(*at, self.mLastAdd); // T::node(mPool[*at]).set_right(mLastAdd);
            }
            nxtChild = T::node(&self.mPool[*at]).right();
            altChild = T::node(&self.mPool[*at]).left();
            nxtRotateLeft = true;
        } else {
            // Exact Match
            // the node of interest is at
            return tree_node::NULL_NODE;
        }

        // If The Add Resulted In An Excess Red Child, We Need To Change Colors And Rotate
        // ---------------------------------------------------------------------------------
        if excessRedChild != tree_node::NULL_NODE {
            // If Both Childeren Are Red, Just Switch And Be Done
            // ----------------------------------------------------
            if T::node(&self.mPool[*at]).right() != tree_node::NULL_NODE
                && T::node(&self.mPool[*at]).left() != tree_node::NULL_NODE
                && T::node(&self.mPool[T::node(&self.mPool[*at]).right()]).red()
                && T::node(&self.mPool[T::node(&self.mPool[*at]).left()]).red()
            {
                self.set_colors(T::node(&mut self.mPool[*at]), true, false);
            } else {
                let excessRedChildCompare = if nxtRotateLeft {
                    T::node(&self.mPool[nxtChild]).right()
                } else {
                    T::node(&self.mPool[nxtChild]).left()
                };
                if excessRedChild == excessRedChildCompare {
                    // Single Rotation
                    // -----------------
                    self.rotate(at, nxtRotateLeft);
                } else {
                    if nxtRotateLeft {
                        let mut nxt = T::node(&self.mPool[*at]).right();
                        self.rotate(&mut nxt, false);
                        self.link_right(*at, nxt); // T::node(mPool[*at]).set_right(nxt);
                    } else {
                        let mut nxt = T::node(&self.mPool[*at]).left();
                        self.rotate(&mut nxt, true);
                        self.link_left(*at, nxt); // T::node(mPool[*at]).set_left(nxt);
                    }
                    self.rotate(at, nxtRotateLeft);
                }

                self.set_colors(T::node(&mut self.mPool[*at]), false, true);
            }
        }

        if T::node(&self.mPool[*at]).red() {
            if T::node(&self.mPool[*at]).left() != tree_node::NULL_NODE
                && T::node(&self.mPool[T::node(&self.mPool[*at]).left()]).red()
            {
                return T::node(&self.mPool[*at]).left();
            }
            if T::node(&self.mPool[*at]).right() != tree_node::NULL_NODE
                && T::node(&self.mPool[T::node(&self.mPool[*at]).right()]).red()
            {
                return T::node(&self.mPool[*at]).right();
            }
        }
        return tree_node::NULL_NODE;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive erase - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn erase_internal(&mut self, key: &T::TValue, at: &mut i32) -> bool {
        // If At Is A NULL_NODE, We Have Found A Leaf.
        // ---------------------------------------------
        if *at == tree_node::NULL_NODE {
            return true;
        }

        // ==============================================================================
        // Now The Question Is, Do We Need To Continue Searching?
        // ==============================================================================

        // Recurse To The Left?
        // ----------------------
        if key < &self.mPool[*at] {
            let mut a = T::node(&self.mPool[*at]).left();
            let r = self.erase_internal(key, &mut a);
            self.link_left(*at, a); // T::node(mPool[*at]).set_left(a);
            if !r {
                // If It Was Not Red, We Need To Rebalance
                return self.rebalance(at, true);
            }
            return true;
        }

        // Recurse To The Right?
        // -----------------------
        if self.mPool[*at] < *key {
            let mut a = T::node(&self.mPool[*at]).right();
            let r = self.erase_internal(key, &mut a);
            self.link_right(*at, a); // T::node(mPool[*at]).set_right(a);
            if !r {
                // If It Was Not Red, We Need To Rebalance
                return self.rebalance(at, false);
            }
            return true;
        }

        // ==============================================================================
        // At This Point, We Must Have Discovered An Exact Match For Our Key
        // ==============================================================================

        // Are There Any Open Childeren Slots?
        // -------------------------------------
        if T::node(&self.mPool[*at]).left() == tree_node::NULL_NODE
            || T::node(&self.mPool[*at]).right() == tree_node::NULL_NODE
        {
            let atWasRed = T::node(&self.mPool[*at]).red();
            let oldAt = *at;

            *at = if T::node(&self.mPool[*at]).left() == tree_node::NULL_NODE {
                T::node(&self.mPool[*at]).right()
            } else {
                T::node(&self.mPool[*at]).left()
            }; // If Left Is Null, At Goes Right

            // Actually Free It!
            // -------------------
            self.mPool.free(oldAt);

            // If We Are Now At A Null Node, Just Return
            // -------------------------------------------
            if *at == tree_node::NULL_NODE {
                return atWasRed;
            }

            // Otherwise, Mark The New Child As Red, And Return That Fact Up
            // ---------------------------------------------------------------
            T::node(&mut self.mPool[*at]).set_red(false);
            return true;
        }

        // ==============================================================================
        // There Are No Childeren To Link With, We Are In The Middle Of The Tree.
        // We Need To Find An Open Leaf, Swap Data With That Leaf, and Then Go Find It
        // ==============================================================================

        // Find A Successor Leaf
        // -----------------------
        let at_parent = T::node(&self.mPool[*at]).parent();
        let mut successor = T::node(&self.mPool[*at]).right();

        let mut parent_successor: i32 = -1;
        while T::node(&self.mPool[successor]).left() != tree_node::NULL_NODE {
            parent_successor = successor;
            successor = T::node(&self.mPool[successor]).left();
        }

        let successor_right = T::node(&self.mPool[successor]).right();

        self.link_left(successor, T::node(&self.mPool[*at]).left());

        let red = T::node(&self.mPool[successor]).red();
        T::node(&mut self.mPool[successor]).set_red(T::node(&self.mPool[*at]).red());
        T::node(&mut self.mPool[*at]).set_red(red);

        if parent_successor != -1 {
            self.link_right(successor, T::node(&self.mPool[*at]).right());
            self.link_left(parent_successor, *at);
        } else {
            self.link_right(successor, *at);
        }

        if at_parent != tree_node::NULL_NODE {
            if T::node(&self.mPool[at_parent]).left() == *at {
                // my parents left child
                self.link_left(at_parent, successor);
            } else {
                debug_assert_eq!(
                    T::node(&self.mPool[at_parent]).right(),
                    *at,
                    "better be my parents right child then"
                );
                self.link_right(at_parent, successor);
            }
        }

        self.link_left(*at, tree_node::NULL_NODE);
        self.link_right(*at, successor_right);

        *at = successor;
        let mut a = T::node(&self.mPool[*at]).right();
        let r = self.erase_internal(key, &mut a);
        self.link_right(*at, a); // T::node(mPool[*at]).set_right(a);
                                 // And Keep Going
                                 // ----------------
        if !r {
            // If It Was Not Red, We Need To Rebalance
            return self.rebalance(at, false);
        }
        return true;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // HELPER: Change the color of a node and children
    // ////////////////////////////////////////////////////////////////////////////////////
    fn set_colors(&mut self, at: &mut tree_node, red: bool, childRed: bool) {
        at.set_red(red);
        if at.left() != tree_node::NULL_NODE {
            T::node(&mut self.mPool[at.left()]).set_red(childRed);
        }
        if at.right() != tree_node::NULL_NODE {
            T::node(&mut self.mPool[at.right()]).set_red(childRed);
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // HELPER: Rotate node located at (at) either left or right
    // ////////////////////////////////////////////////////////////////////////////////////
    fn rotate(&mut self, at: &mut i32, left: bool) {
        let mut t: i32;
        if left {
            debug_assert_ne!(T::node(&self.mPool[*at]).right(), tree_node::NULL_NODE);

            t = T::node(&self.mPool[*at]).right();
            self.link_right(*at, T::node(&self.mPool[t]).left()); // T::node(mPool[*at]).set_right(T::node(mPool[t]).left());
            self.link_left(t, *at); // T::node(mPool[t]).set_left(*at);
            *at = t;
        } else {
            debug_assert_ne!(T::node(&self.mPool[*at]).left(), tree_node::NULL_NODE);

            t = T::node(&self.mPool[*at]).left();
            self.link_left(*at, T::node(&self.mPool[t]).right()); // T::node(mPool[*at]).set_left(T::node(mPool[t]).right());
            self.link_right(t, *at); // T::node(mPool[t]).set_right(*at);
            *at = t;
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // HELPER: Localally rebalance the tree here
    // ////////////////////////////////////////////////////////////////////////////////////
    fn rebalance(&mut self, at: &mut i32, left: bool) -> bool {
        // Decide Which Child, Left Or Right?
        // ------------------------------------
        let w = if left {
            T::node(&self.mPool[*at]).right()
        } else {
            T::node(&self.mPool[*at]).left()
        }; // w is the child of at
        if w == tree_node::NULL_NODE {
            let atWasRed = T::node(&self.mPool[*at]).red(); // Remember what mPool[*at] WAS
            T::node(&mut self.mPool[*at]).set_red(false); // Mark mPool[*at] as BLACK
            return atWasRed; // Return what it used to be
        }

        // Get A Reference To The Child W, And Record It's Children x And y
        // ------------------------------------------------------------------
        let x = if left {
            T::node(&self.mPool[w]).left()
        } else {
            T::node(&self.mPool[w]).right()
        }; // x and y are the grand children of at
        let y = if left {
            T::node(&self.mPool[w]).right()
        } else {
            T::node(&self.mPool[w]).left()
        };

        // Is The Child Black?
        // ---------------------
        if !T::node(&self.mPool[w]).red() {
            // If Both X and Y are Empty, Or Both Are Red
            // --------------------------------------------
            if (x == tree_node::NULL_NODE || !T::node(&self.mPool[x]).red())
                && (y == tree_node::NULL_NODE || !T::node(&self.mPool[y]).red())
            {
                let atWasRed = T::node(&self.mPool[*at]).red(); // Remember what mPool[*at] WAS
                T::node(&mut self.mPool[*at]).set_red(false); // Mark mPool[*at] as BLACK
                T::node(&mut self.mPool[w]).set_red(true); // Mark The Child As RED
                return atWasRed; // Return what it used to be
            }

            // If Y Is Valid
            // ---------------
            if y != tree_node::NULL_NODE && T::node(&self.mPool[y]).red() {
                T::node(&mut self.mPool[w])
                    .set_red(T::node(&self.mPool[*at]).red());
                self.rotate(at, left);
                T::node(&mut self.mPool[T::node(&self.mPool[*at]).left()])
                    .set_red(false);
                T::node(&mut self.mPool[T::node(&self.mPool[*at]).right()])
                    .set_red(false);
                return true;
            }

            // X Must Be Valid
            // -----------------
            T::node(&mut self.mPool[x])
                .set_red(T::node(&self.mPool[*at]).red());
            T::node(&mut self.mPool[*at]).set_red(false);

            if left {
                let mut r = T::node(&self.mPool[*at]).right();
                self.rotate(&mut r, false);
                self.link_right(*at, r); // T::node(mPool[*at]).set_right(r);
            } else {
                let mut r = T::node(&self.mPool[*at]).left();
                self.rotate(&mut r, true);
                self.link_left(*at, r); // T::node(mPool[*at]).set_left(r);
            }
            self.rotate(at, left);

            return true;
        }

        // The Child Must Have Been Red
        // ------------------------------
        T::node(&mut self.mPool[w]).set_red(T::node(&self.mPool[*at]).red()); // Switch Child Color
        T::node(&mut self.mPool[*at]).set_red(true);
        self.rotate(at, left); // Rotate At

        // Select The Next Rebalance Child And Recurse
        // ----------------------------------------------
        if left {
            let mut r = T::node(&self.mPool[*at]).left();
            self.rebalance(&mut r, true);
            self.link_left(*at, r); // T::node(mPool[*at]).set_left(r);
        } else {
            let mut r = T::node(&self.mPool[*at]).right();
            self.rebalance(&mut r, false);
            self.link_right(*at, r); // T::node(mPool[*at]).set_right(r);
        }
        return true;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive front function - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn front(&self, at: i32) -> i32 {
        if at != tree_node::NULL_NODE && T::node(&self.mPool[at]).left() != tree_node::NULL_NODE
        {
            return self.front(T::node(&self.mPool[at]).left());
        }
        return at;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive back function - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn back(&self, at: i32) -> i32 {
        if at != tree_node::NULL_NODE && T::node(&self.mPool[at]).right() != tree_node::NULL_NODE
        {
            return self.back(T::node(&self.mPool[at]).right());
        }
        return at;
    }

    pub fn front_root(&self) -> i32 {
        self.front(self.mRoot)
    }

    pub fn back_root(&self) -> i32 {
        self.back(self.mRoot)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive next function - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn next(&self, at: i32) -> i32 {
        debug_assert_ne!(at, tree_node::NULL_NODE);
        let kAt = &self.mPool[at];
        let nAt = T::node(kAt);
        if nAt.right() != tree_node::NULL_NODE {
            return self.front(nAt.right());
        }

        let mut child = at;
        let mut parent = tree_node::NULL_NODE;
        self.find_internal_with_target(kAt, at, self.mRoot, &mut parent);

        while parent != tree_node::NULL_NODE
            && (child == T::node(&self.mPool[parent]).right())
        {
            child = parent;
            self.find_internal_with_target(&self.mPool[parent], parent, self.mRoot, &mut parent);
        }
        return parent;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // This is the map internal recursive previous function - do not use externally
    // ////////////////////////////////////////////////////////////////////////////////////
    fn previous(&self, at: i32) -> i32 {
        debug_assert_ne!(at, tree_node::NULL_NODE);
        let kAt = &self.mPool[at];
        let nAt = T::node(&self.mPool[at]);
        if nAt.left() != tree_node::NULL_NODE {
            return self.back(nAt.left());
        }

        let mut child = at;
        let mut parent = tree_node::NULL_NODE;
        self.find_internal_with_target(kAt, at, self.mRoot, &mut parent);

        while parent != tree_node::NULL_NODE && (child == T::node(&self.mPool[parent]).left())
        {
            child = parent;
            self.find_internal_with_target(&self.mPool[parent], parent, self.mRoot, &mut parent);
        }
        return parent;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        tree_base {
            mPool: pool_base {
                _phantom: core::marker::PhantomData,
            },
            mRoot: tree_node::NULL_NODE,
            mLastAdd: -1,
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // How Many Objects Are In This Map
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> i32 {
        self.mPool.size()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Are There Any Objects In This Map?
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        self.mPool.empty()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Is This Map Filled?
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        self.mPool.full()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Clear All Data From The Map
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mRoot = tree_node::NULL_NODE;
        self.mPool.clear();
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Adds Element Value At Location Key - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_key(&mut self, key: T::TValue) {
        // fixme handle duplicates more sensibly?
        debug_assert!(!self.full());
        self.mLastAdd = self.mPool.alloc(key); // Grab A New One
        T::node(&mut self.mPool[self.mLastAdd]).init(); // Initialize Our Data And Color
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Allocs an item, when filled, call insert_alloced
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_key_ref(&mut self) -> &mut T::TValue {
        debug_assert!(!self.full());
        self.mLastAdd = self.mPool.alloc(); // Grab A New One
        T::node(&mut self.mPool[self.mLastAdd]).init(); // Initialize Our Data And Color
        &mut self.mPool[self.mLastAdd]
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Allocs an item (raw), when constucted, call insert_alloced
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_key_raw(&mut self) -> *mut u8 {
        debug_assert!(!self.full());
        let ret = self.mPool.alloc_raw(); // Grab A New One
        self.mLastAdd = self.mPool.pointer_to_index(ret);
        T::node(&mut self.mPool[self.mLastAdd]).init(); // Initialize Our Data And Color
        return ret;
    }

    pub fn verify_alloc_key<U>(&self, p: *mut U) -> *mut U {
        self.mPool.verify_alloc(p)
    }

    pub fn insert_alloced_key(&mut self) {
        debug_assert!(self.mLastAdd >= 0 && self.mLastAdd < Self::CAPACITY);
        debug_assert!(!IS_MULTI == 0 || self.find_index(&self.mPool[self.mLastAdd]) != tree_node::NULL_NODE); // fixme handle duplicates more sensibly?

        self.insert_internal(&self.mPool[self.mLastAdd].clone(), &mut self.mRoot);
        debug_assert_ne!(self.mRoot, tree_node::NULL_NODE);
        T::node(&mut self.mPool[self.mRoot]).set_red(false);
        T::node(&mut self.mPool[self.mRoot]).set_parent(tree_node::NULL_NODE);
    }

    pub fn index_of_alloced_key(&self) -> i32 {
        debug_assert!(self.mLastAdd >= 0 && self.mLastAdd < Self::CAPACITY);
        return self.mLastAdd;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Removes The Element Pointed To At (it) And Decrements (it) - O((log n)^2)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase_index(&mut self, i: i32) {
        debug_assert!(i >= 0 && i < Self::CAPACITY);
        debug_assert_ne!(self.mRoot, tree_node::NULL_NODE);

        // fixme this is lame to have to look by key to erase
        self.erase_internal(&self.mPool[i].clone(), &mut self.mRoot);
        if self.mRoot != tree_node::NULL_NODE {
            T::node(&mut self.mPool[self.mRoot]).set_red(false);
            T::node(&mut self.mPool[self.mRoot]).set_parent(tree_node::NULL_NODE);
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Seach For A Given Key. Will Return -1 if Failed - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn find_index(&self, key: &T::TValue) -> i32 {
        self.find_internal(key, self.mRoot)
    }

    pub fn index_to_key(&self, i: i32) -> &T::TValue {
        debug_assert!(i >= 0 && i < Self::CAPACITY);
        &self.mPool[i]
    }

    // fixme lower bound, upper bound, equal range
}

// Implement Clone for tree_base if needed
impl<T: StorageTraits, const IS_MULTI: i32> Clone for tree_base<T, IS_MULTI>
where
    T::TValue: Clone,
{
    fn clone(&self) -> Self {
        tree_base {
            mPool: pool_base {
                _phantom: core::marker::PhantomData,
            },
            mRoot: self.mRoot,
            mLastAdd: self.mLastAdd,
        }
    }
}

pub struct set_base<T: StorageTraits, const IS_MULTI: i32> {
    tree: tree_base<T, IS_MULTI>,
}

impl<T: StorageTraits, const IS_MULTI: i32> set_base<T, IS_MULTI>
where
    T::TValue: Ord + Clone,
{
    // ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    // ////////////////////////////////////////////////////////////////////////////////////
    const CAPACITY: i32 = T::CAPACITY;

    // ////////////////////////////////////////////////////////////////////////////////////
    // Adds Element Value At Location Key - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn insert(&mut self, key: T::TValue) {
        debug_assert!(!IS_MULTI == 0 || self.tree.find_index(&key) == tree_node::NULL_NODE); // fixme handle duplicates more sensibly?

        self.tree.alloc_key(key);
        self.tree.insert_alloced_key();
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Allocs an item, when filled, call insert_alloced
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc(&mut self) -> &mut T::TValue {
        self.tree.alloc_key_ref()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Allocs an item (raw), when constucted, call insert_alloced
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_raw(&mut self) -> *mut u8 {
        self.tree.alloc_key_raw()
    }

    pub fn verify_alloc<U>(&self, p: *mut U) -> *mut U {
        self.tree.verify_alloc_key(p)
    }

    pub fn insert_alloced(&mut self) {
        self.tree.insert_alloced_key();
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Removes The First Element With Key (key) - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase(&mut self, key: &T::TValue) {
        // fixme this is a double search currently
        let i = self.tree.find_index(key);
        if i != tree_node::NULL_NODE {
            self.tree.erase_index(i);
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Iterator
    //
    // A map is sorted in ascending order on the KEY type. ++ and -- are both
    // O((log n)^2) operations
    //
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn find(&mut self, key: &T::TValue) -> i32 {
        self.tree.find_index(key)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Smallest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin(&self) -> i32 {
        self.tree.front_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Largest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn rbegin(&self) -> i32 {
        self.tree.back_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // The Invalid Iterator, Use As A Stop Condition In Your For Loops - O(1)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn end(&self) -> i32 {
        tree_node::NULL_NODE
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Seach For A Given Key. Will Return end() if Failed - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn find_const(&self, key: &T::TValue) -> i32 {
        self.tree.find_index(key)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An const_iterator To The Smallest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin_const(&self) -> i32 {
        self.tree.front_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An const_iterator To The Largest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn rbegin_const(&self) -> i32 {
        self.tree.back_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // The Invalid const_iterator, Use As A Stop Condition In Your For Loops - O(1)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn end_const(&self) -> i32 {
        tree_node::NULL_NODE
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Removes The Element Pointed To At (it) And Decrements (it) - O((log n)^2)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase_at(&mut self, it: i32) {
        debug_assert!(it >= 0 && it < Self::CAPACITY);
        self.tree.erase_index(it);
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn lower_bound(&mut self, key: &T::TValue) -> i32 {
        self.tree.find_index(key)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn upper_bound(&mut self, key: &T::TValue) -> i32 {
        // fixme, this don't work
        let mut ubound = self.tree.find_index(key);
        if ubound != tree_node::NULL_NODE {
            ubound = self.tree.next(ubound);
        }
        return ubound;
    }

    pub fn new() -> Self {
        set_base {
            tree: tree_base::new(),
        }
    }

    pub fn size(&self) -> i32 {
        self.tree.size()
    }

    pub fn empty(&self) -> bool {
        self.tree.empty()
    }

    pub fn full(&self) -> bool {
        self.tree.full()
    }

    pub fn clear(&mut self) {
        self.tree.clear();
    }
}

pub struct set_vs<T: Clone, const ARG_CAPACITY: i32> {
    // Placeholder: storage::value_semantics_node<T, ARG_CAPACITY, tree_node>
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Clone, const ARG_CAPACITY: i32> set_vs<T, ARG_CAPACITY> {
    const CAPACITY: i32 = ARG_CAPACITY;

    pub fn new() -> Self {
        set_vs {
            _phantom: core::marker::PhantomData,
        }
    }
}

pub struct set_os<T: Clone, const ARG_CAPACITY: i32> {
    // Placeholder: storage::object_semantics_node<T, ARG_CAPACITY, tree_node>
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Clone, const ARG_CAPACITY: i32> set_os<T, ARG_CAPACITY> {
    const CAPACITY: i32 = ARG_CAPACITY;

    pub fn new() -> Self {
        set_os {
            _phantom: core::marker::PhantomData,
        }
    }
}

pub struct set_is<T: Clone, const ARG_CAPACITY: i32, const ARG_MAX_CLASS_SIZE: i32> {
    // Placeholder: storage::virtual_semantics_node<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE, tree_node>
    _phantom: core::marker::PhantomData<T>,
}

impl<T: Clone, const ARG_CAPACITY: i32, const ARG_MAX_CLASS_SIZE: i32>
    set_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>
{
    const CAPACITY: i32 = ARG_CAPACITY;
    const MAX_CLASS_SIZE: i32 = ARG_MAX_CLASS_SIZE;

    pub fn new() -> Self {
        set_is {
            _phantom: core::marker::PhantomData,
        }
    }
}

pub struct map_base<K: StorageTraits, V: StorageTraits, const IS_MULTI: i32> {
    tree: tree_base<K, IS_MULTI>,
    mValues: array_base<V>,
}

impl<K: StorageTraits, V: StorageTraits, const IS_MULTI: i32> map_base<K, V, IS_MULTI>
where
    K::TValue: Ord + Clone,
    V::TValue: Clone,
{
    // ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    // ////////////////////////////////////////////////////////////////////////////////////
    const CAPACITY: i32 = K::CAPACITY;

    pub fn new() -> Self {
        compile_assert::<{ K::CAPACITY == V::CAPACITY }>();
        map_base {
            tree: tree_base::new(),
            mValues: array_base {
                _phantom: core::marker::PhantomData,
            },
        }
    }

    pub fn clear(&mut self) {
        self.tree.clear();
        self.mValues.clear();
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Adds Element Value At Location Key - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn insert(&mut self, key: K::TValue, value: V::TValue) {
        debug_assert!(
            !IS_MULTI == 0 || self.tree.find_index(&key) == tree_node::NULL_NODE
        ); // fixme handle duplicates more sensibly?

        self.tree.alloc_key(key);
        self.tree.insert_alloced_key();
        debug_assert!(self.check_validity());
        self.mValues
            .construct(self.tree.index_of_alloced_key(), value);
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Adds Element Value At Location Key returns a reference
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn insert_ref(&mut self, key: K::TValue) -> &mut V::TValue {
        debug_assert!(
            !IS_MULTI == 0 || self.tree.find_index(&key) == tree_node::NULL_NODE
        ); // fixme handle duplicates more sensibly?

        self.tree.alloc_key(key);
        self.tree.insert_alloced_key();

        let idx = self.tree.index_of_alloced_key();
        debug_assert!(self.check_validity());
        self.mValues.construct(idx);
        &mut self.mValues[idx]
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Adds Element Value At Location Key returns a rew pointer for placement new
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn insert_raw(&mut self, key: K::TValue) -> *mut u8 {
        debug_assert!(
            !IS_MULTI == 0 || self.tree.find_index(&key) == tree_node::NULL_NODE
        ); // fixme handle duplicates more sensibly?

        self.tree.alloc_key(key);
        self.tree.insert_alloced_key();
        debug_assert!(self.check_validity());
        self.mValues.alloc_raw(self.tree.index_of_alloced_key())
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // After calling alloc_key*, you may call this to alloc the value
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_value(&mut self) -> &mut V::TValue {
        self.mValues
            .construct(self.tree.index_of_alloced_key());
        &mut self.mValues[self.tree.index_of_alloced_key()]
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // After calling alloc_key*, you may call this to alloc_raw the value
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_value_raw(&mut self) -> *mut u8 {
        self.mValues.alloc_raw(self.tree.index_of_alloced_key())
    }

    pub fn verify_alloc<U>(&self, p: *mut U) -> *mut U {
        self.mValues.verify_alloc(p)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Removes The First Element With Key (key) - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase(&mut self, key: &K::TValue) {
        // fixme this is a double search currently
        let i = self.tree.find_index(key);
        if i != tree_node::NULL_NODE {
            self.tree.erase_index(i);
            self.mValues.destruct(i);
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Iterator
    //
    // A map is sorted in ascending order on the KEY type. ++ and -- are both
    // O((log n)^2) operations
    //
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn find(&mut self, key: &K::TValue) -> i32 {
        self.tree.find_index(key)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Smallest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin(&self) -> i32 {
        self.tree.front_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Largest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn rbegin(&self) -> i32 {
        self.tree.back_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // The Invalid Iterator, Use As A Stop Condition In Your For Loops - O(1)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn end(&self) -> i32 {
        tree_node::NULL_NODE
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Seach For A Given Key. Will Return end() if Failed - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn find_const(&self, key: &K::TValue) -> i32 {
        self.tree.find_index(key)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An const_iterator To The Smallest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin_const(&self) -> i32 {
        self.tree.front_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Get An const_iterator To The Largest Element - O(log n)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn rbegin_const(&self) -> i32 {
        self.tree.back_root()
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // The Invalid const_iterator, Use As A Stop Condition In Your For Loops - O(1)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn end_const(&self) -> i32 {
        tree_node::NULL_NODE
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // Removes The Element Pointed To At (it) And Decrements (it) - O((log n)^2)
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase_at(&mut self, it: i32) {
        debug_assert!(it >= 0 && it < Self::CAPACITY);
        self.tree.erase_index(it);
        self.mValues.destruct(it);
    }

    fn check_validity(&self) -> bool {
        #[cfg(any())]
        {
            let mut cnt = 0;
            let it = self.begin();
            let mut cur = it;
            while cur != self.end() {
                cnt += 1;
                cur = self.tree.next(cur);
            }
            cnt == self.tree.size()
        }
        #[cfg(not(any()))]
        {
            true
        }
    }

    pub fn size(&self) -> i32 {
        self.tree.size()
    }

    pub fn empty(&self) -> bool {
        self.tree.empty()
    }

    pub fn full(&self) -> bool {
        self.tree.full()
    }

    pub fn clear_all(&mut self) {
        self.tree.clear();
        self.mValues.clear();
    }
}

pub struct map_vs<K: Clone, V: Clone, const ARG_CAPACITY: i32> {
    // Placeholder: storage::value_semantics_node<K, ARG_CAPACITY, tree_node>,
    //             storage::value_semantics<V, ARG_CAPACITY>
    _phantom: core::marker::PhantomData<(K, V)>,
}

impl<K: Clone, V: Clone, const ARG_CAPACITY: i32> map_vs<K, V, ARG_CAPACITY> {
    const CAPACITY: i32 = ARG_CAPACITY;

    pub fn new() -> Self {
        map_vs {
            _phantom: core::marker::PhantomData,
        }
    }
}

pub struct map_os<K: Clone, V: Clone, const ARG_CAPACITY: i32> {
    // Placeholder: storage::value_semantics_node<K, ARG_CAPACITY, tree_node>,
    //             storage::object_semantics<V, ARG_CAPACITY>
    _phantom: core::marker::PhantomData<(K, V)>,
}

impl<K: Clone, V: Clone, const ARG_CAPACITY: i32> map_os<K, V, ARG_CAPACITY> {
    const CAPACITY: i32 = ARG_CAPACITY;

    pub fn new() -> Self {
        map_os {
            _phantom: core::marker::PhantomData,
        }
    }
}

pub struct map_is<
    K: Clone,
    V: Clone,
    const ARG_CAPACITY: i32,
    const ARG_MAX_CLASS_SIZE: i32,
> {
    // Placeholder: storage::value_semantics_node<K, ARG_CAPACITY, tree_node>,
    //             storage::virtual_semantics<V, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>
    _phantom: core::marker::PhantomData<(K, V)>,
}

impl<K: Clone, V: Clone, const ARG_CAPACITY: i32, const ARG_MAX_CLASS_SIZE: i32>
    map_is<K, V, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>
{
    const CAPACITY: i32 = ARG_CAPACITY;
    const MAX_CLASS_SIZE: i32 = ARG_MAX_CLASS_SIZE;

    pub fn new() -> Self {
        map_is {
            _phantom: core::marker::PhantomData,
        }
    }
}
