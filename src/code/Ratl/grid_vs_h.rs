////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Grid
// ----
// There are two versions of the Grid class.  Simply, they apply a discreet function
// mapping from a n dimensional space to a linear aray.
//
//
//
//
//
//
// NOTES:
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::c_int;

////////////////////////////////////////////////////////////////////////////////////////
// LOCAL STUB: array_vs container type
// This will be replaced when array_vs_h is ported
////////////////////////////////////////////////////////////////////////////////////////
pub struct array_vs<T: Clone + Default + Copy, const SIZE: usize> {
    data: [T; SIZE],
}

impl<T: Clone + Default + Copy, const SIZE: usize> array_vs<T, SIZE> {
    pub fn new() -> Self {
        array_vs {
            data: [T::default(); SIZE],
        }
    }

    pub fn clear(&mut self) {
        for i in 0..SIZE {
            self.data[i] = T::default();
        }
    }

    pub fn clone(&self) -> Self {
        array_vs {
            data: self.data,
        }
    }
}

impl<T: Clone + Default + Copy, const SIZE: usize> std::ops::Index<usize> for array_vs<T, SIZE> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Clone + Default + Copy, const SIZE: usize> std::ops::IndexMut<usize> for array_vs<T, SIZE> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The 2D Grid Class
////////////////////////////////////////////////////////////////////////////////////////
pub struct grid2_vs<T: Clone + Default + Copy, const XSIZE_MAX: usize, const YSIZE_MAX: usize> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Data
    ////////////////////////////////////////////////////////////////////////////////////
    mData: array_vs<T, { XSIZE_MAX * YSIZE_MAX }>,

    mSize: [c_int; 2],
    mMins: [f32; 2],
    mMaxs: [f32; 2],
    mScale: [f32; 2],
}

impl<T: Clone + Default + Copy, const XSIZE_MAX: usize, const YSIZE_MAX: usize>
    grid2_vs<T, XSIZE_MAX, YSIZE_MAX>
{
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        grid2_vs {
            mData: array_vs::<T, { XSIZE_MAX * YSIZE_MAX }>::new(),
            mSize: [XSIZE_MAX as c_int, YSIZE_MAX as c_int],
            mMins: [Self::RANGE_NULL as f32, Self::RANGE_NULL as f32],
            mMaxs: [Self::RANGE_NULL as f32, Self::RANGE_NULL as f32],
            mScale: [0.0f32, 0.0f32],
        }
    }

    pub const RANGE_NULL: c_int = 12345;

    ////////////////////////////////////////////////////////////////////////////////////
    // Assignment Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn assign(&mut self, other: &grid2_vs<T, XSIZE_MAX, YSIZE_MAX>) {
        self.mData = other.mData.clone();
        for i in 0..2 {
            self.mSize[i] = other.mSize[i];
            self.mMins[i] = other.mMins[i];
            self.mMaxs[i] = other.mMaxs[i];
            self.mScale[i] = other.mScale[i];
        }
    }

    pub fn set_size(&mut self, xSize: c_int, ySize: c_int) {
        if xSize < XSIZE_MAX as c_int {
            self.mSize[0] = xSize;
        }
        if ySize < YSIZE_MAX as c_int {
            self.mSize[1] = ySize;
        }
    }

    pub fn snap_scale(&mut self) {
        self.mScale[0] = (self.mScale[0] as c_int) as f32;
        self.mScale[1] = (self.mScale[1] as c_int) as f32;
    }

    pub fn get_size(&self, xSize: &mut c_int, ySize: &mut c_int) {
        *xSize = self.mSize[0];
        *ySize = self.mSize[1];
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Clear
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mSize[0] = XSIZE_MAX as c_int;
        self.mSize[1] = YSIZE_MAX as c_int;
        self.mData.clear();
        for i in 0..2 {
            self.mMins[i] = Self::RANGE_NULL as f32;
            self.mMaxs[i] = Self::RANGE_NULL as f32;
            self.mScale[i] = 0.0f32;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initialize The Entire Grid To A Value
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn init(&mut self, val: T) {
        for i in 0..(XSIZE_MAX * YSIZE_MAX) {
            self.mData[i] = val;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Copy The Bounds Of Another Grid
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn copy_bounds(&mut self, other: &grid2_vs<T, XSIZE_MAX, YSIZE_MAX>) {
        for i in 0..2 {
            self.mSize[i] = other.mSize[i];
            self.mMins[i] = other.mMins[i];
            self.mMaxs[i] = other.mMaxs[i];
            self.mScale[i] = other.mScale[i];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get(&mut self, x: c_int, y: c_int) -> &mut T {
        assert!(x >= 0 && y >= 0 && x < self.mSize[0] && y < self.mSize[1]);
        let index = (x + y * XSIZE_MAX as c_int) as usize;
        &mut self.mData[index]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_f(&mut self, mut x: f32, mut y: f32) -> &mut T {
        assert!(self.mScale[0] != 0.0f32 && self.mScale[1] != 0.0f32);
        self.truncate_position_to_bounds(&mut x, &mut y);

        let xint = ((x - self.mMins[0]) / self.mScale[0]) as c_int;
        let yint = ((y - self.mMins[1]) / self.mScale[1]) as c_int;

        assert!(xint >= 0 && yint >= 0 && xint < self.mSize[0] && yint < self.mSize[1]);
        let index = (xint + yint * XSIZE_MAX as c_int) as usize;
        &mut self.mData[index]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Convert The Scaled Coordinates To A Grid Coordinate
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_cell_coords(&self, mut x: f32, mut y: f32, xint: &mut c_int, yint: &mut c_int) {
        assert!(self.mScale[0] != 0.0f32 && self.mScale[1] != 0.0f32);
        self.truncate_position_to_bounds(&mut x, &mut y);

        *xint = ((x - self.mMins[0]) / self.mScale[0]) as c_int;
        *yint = ((y - self.mMins[1]) / self.mScale[1]) as c_int;

        assert!(*xint >= 0 && *yint >= 0 && *xint < self.mSize[0] && *yint < self.mSize[1]);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Expand
    //
    // NOTE:  This MUST be at least a 2 dimensional point
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn expand_bounds(&mut self, xReal: f32, yReal: f32) {
        let point = [xReal, yReal];
        for i in 0..2 {
            if point[i] < self.mMins[i] || self.mMins[i] == Self::RANGE_NULL as f32 {
                self.mMins[i] = point[i];
            }
            if point[i] > self.mMaxs[i] || self.mMaxs[i] == Self::RANGE_NULL as f32 {
                self.mMaxs[i] = point[i];
            }
        }
        assert!(self.mSize[0] > 0 && self.mSize[1] > 0);

        self.mScale[0] = (self.mMaxs[0] - self.mMins[0]) / self.mSize[0] as f32;
        self.mScale[1] = (self.mMaxs[1] - self.mMins[1]) / self.mSize[1] as f32;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn truncate_position_to_bounds(&self, xReal: &mut f32, yReal: &mut f32) {
        if *xReal < self.mMins[0] {
            *xReal = self.mMins[0];
        }
        if *xReal > (self.mMaxs[0] - 1.0f32) {
            *xReal = self.mMaxs[0] - 1.0f32;
        }
        if *yReal < self.mMins[1] {
            *yReal = self.mMins[1];
        }
        if *yReal > (self.mMaxs[1] - 1.0f32) {
            *yReal = self.mMaxs[1] - 1.0f32;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_cell_position(&self, x: c_int, y: c_int, xReal: &mut f32, yReal: &mut f32) {
        //	assert(mScale[0]!=0.0f && mScale[1]!=0.0f);
        *xReal = (x as f32 * self.mScale[0]) + self.mMins[0] + (self.mScale[0] * 0.5f32);
        *yReal = (y as f32 * self.mScale[1]) + self.mMins[1] + (self.mScale[1] * 0.5f32);
    }
    pub fn get_cell_upperleft(&self, x: c_int, y: c_int, xReal: &mut f32, yReal: &mut f32) {
        //	assert(mScale[0]!=0.0f && mScale[1]!=0.0f);
        *xReal = (x as f32 * self.mScale[0]) + self.mMins[0];
        *yReal = (y as f32 * self.mScale[1]) + self.mMins[1];
    }
    pub fn get_cell_lowerright(&self, x: c_int, y: c_int, xReal: &mut f32, yReal: &mut f32) {
        //	assert(mScale[0]!=0.0f && mScale[1]!=0.0f);
        *xReal = (x as f32 * self.mScale[0]) + self.mMins[0] + (self.mScale[0]);
        *yReal = (y as f32 * self.mScale[1]) + self.mMins[1] + (self.mScale[1]);
    }
    pub fn scale_by_largest_axis(&self, dist: &mut f32) {
        assert!(self.mScale[0] != 0.0f32 && self.mScale[1] != 0.0f32);
        if self.mScale[0] > self.mScale[1] {
            *dist /= self.mScale[0];
        } else {
            *dist /= self.mScale[1];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Raw Get - For The Iterator Dereference Function
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn rawGet(&mut self, Loc: usize) -> &mut T {
        assert!(Loc >= 0 && Loc < XSIZE_MAX * YSIZE_MAX);
        &mut self.mData[Loc]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Begin
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin(&self, x: c_int, y: c_int) -> iterator<T, XSIZE_MAX, YSIZE_MAX> {
        assert!(x >= 0 && y >= 0 && x < self.mSize[0] && y < self.mSize[1]);
        iterator::new_with(self as *const _, (x + y * XSIZE_MAX as c_int) as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Begin (default parameters)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin_default(&self) -> iterator<T, XSIZE_MAX, YSIZE_MAX> {
        self.begin(0, 0)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Begin (scaled position, use mins and maxs to calc real position)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin_f(&self, mut xReal: f32, mut yReal: f32) -> iterator<T, XSIZE_MAX, YSIZE_MAX> {
        assert!(self.mScale[0] != 0.0f32 && self.mScale[1] != 0.0f32);
        self.truncate_position_to_bounds(&mut xReal, &mut yReal);

        let x = ((xReal - self.mMins[0]) / self.mScale[0]) as c_int;
        let y = ((yReal - self.mMins[1]) / self.mScale[1]) as c_int;

        self.begin(x, y)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator End
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn end(&self) -> iterator<T, XSIZE_MAX, YSIZE_MAX> {
        iterator::new_with(self as *const _, XSIZE_MAX * YSIZE_MAX)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Ranged Iterator Begin (x and y are the center of the range)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn rangeBegin(&self, range: c_int, x: c_int, y: c_int) -> riterator<T, XSIZE_MAX, YSIZE_MAX> {
        assert!(x >= 0 && y >= 0 && x < XSIZE_MAX as c_int && y < YSIZE_MAX as c_int);
        riterator::new_with(self as *const _, range, x, y)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn rangeBegin_f(
        &self,
        range: c_int,
        mut xReal: f32,
        mut yReal: f32,
    ) -> riterator<T, XSIZE_MAX, YSIZE_MAX> {
        let position = [xReal, yReal];
        assert!(self.mScale[0] != 0.0f32 && self.mScale[1] != 0.0f32);
        self.truncate_position_to_bounds(&mut xReal, &mut yReal);
        let x = ((position[0] - self.mMins[0]) / self.mScale[0]) as c_int;
        let y = ((position[1] - self.mMins[1]) / self.mScale[1]) as c_int;

        assert!(x >= 0 && y >= 0 && x < XSIZE_MAX as c_int && y < YSIZE_MAX as c_int);
        riterator::new_with(self as *const _, range, x, y)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////
// Iterator
////////////////////////////////////////////////////////////////////////////////////////////
pub struct iterator<T: Clone + Default + Copy, const XSIZE_MAX: usize, const YSIZE_MAX: usize> {
    // Constructors
    //--------------
    mOwner: *const grid2_vs<T, XSIZE_MAX, YSIZE_MAX>,
    mLoc: usize,
}

impl<T: Clone + Default + Copy, const XSIZE_MAX: usize, const YSIZE_MAX: usize>
    iterator<T, XSIZE_MAX, YSIZE_MAX>
{
    // Constructors
    //--------------
    pub fn new() -> Self {
        iterator {
            mOwner: std::ptr::null(),
            mLoc: 0,
        }
    }

    pub fn new_with(
        p: *const grid2_vs<T, XSIZE_MAX, YSIZE_MAX>,
        t: usize,
    ) -> Self {
        iterator { mOwner: p, mLoc: t }
    }

    // Assignment Operator
    //---------------------
    pub fn assign(&mut self, t: &iterator<T, XSIZE_MAX, YSIZE_MAX>) {
        self.mOwner = t.mOwner;
        self.mLoc = t.mLoc;
    }

    // Equality & Inequality Operators
    //---------------------------------
    pub fn ne(&self, t: &iterator<T, XSIZE_MAX, YSIZE_MAX>) -> bool {
        self.mLoc != t.mLoc
    }
    pub fn eq(&self, t: &iterator<T, XSIZE_MAX, YSIZE_MAX>) -> bool {
        self.mLoc == t.mLoc
    }

    // Dereference Operator
    //----------------------
    pub fn deref(&mut self) -> &mut T {
        unsafe {
            // PORTING NOTE: Raw pointer dereference preserves C++ aliasing behavior
            let grid_mut = self.mOwner as *mut grid2_vs<T, XSIZE_MAX, YSIZE_MAX>;
            (*grid_mut).rawGet(self.mLoc)
        }
    }

    // Inc Operator
    //--------------
    pub fn inc(&mut self) {
        self.mLoc += 1;
    }

    // Row & Col Offsets
    //-------------------
    pub fn offsetRows(&mut self, num: usize) {
        self.mLoc += YSIZE_MAX * num;
    }
    pub fn offsetCols(&mut self, num: usize) {
        self.mLoc += num;
    }

    // Return True If On Frist Column Of A Row
    //-----------------------------------------
    pub fn onColZero(&self) -> bool {
        (self.mLoc % XSIZE_MAX) == 0
    }

    // Evaluate The XY Position Of This Iterator
    //-------------------------------------------
    pub fn position(&self, X: &mut c_int, Y: &mut c_int) {
        *Y = (self.mLoc / XSIZE_MAX) as c_int;
        *X = (self.mLoc - (*Y as usize * XSIZE_MAX)) as c_int;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////
// Ranged Iterator
////////////////////////////////////////////////////////////////////////////////////////////
pub struct riterator<T: Clone + Default + Copy, const XSIZE_MAX: usize, const YSIZE_MAX: usize> {
    mOwner: *const grid2_vs<T, XSIZE_MAX, YSIZE_MAX>,
    mMins: [c_int; 2],
    mMaxs: [c_int; 2],
    mLoc: [c_int; 2],
}

impl<T: Clone + Default + Copy, const XSIZE_MAX: usize, const YSIZE_MAX: usize>
    riterator<T, XSIZE_MAX, YSIZE_MAX>
{
    // Constructors
    //--------------
    pub fn new() -> Self {
        riterator {
            mOwner: std::ptr::null(),
            mMins: [0; 2],
            mMaxs: [0; 2],
            mLoc: [0; 2],
        }
    }

    pub fn new_with(
        p: *const grid2_vs<T, XSIZE_MAX, YSIZE_MAX>,
        Range: c_int,
        SX: c_int,
        SY: c_int,
    ) -> Self {
        let Start = [SX, SY];
        let Bounds = [(XSIZE_MAX as c_int) - 1, (YSIZE_MAX as c_int) - 1];

        let mut mMins = [0; 2];
        let mut mMaxs = [0; 2];
        let mut mLoc = [0; 2];

        for i in 0..2 {
            mMins[i] = Start[i] - Range;
            mMaxs[i] = Start[i] + Range;

            if mMins[i] < 0 {
                mMins[i] = 0;
            }
            if mMaxs[i] > Bounds[i] {
                mMaxs[i] = Bounds[i];
            }

            mLoc[i] = mMins[i];
        }

        riterator {
            mOwner: p,
            mMins,
            mMaxs,
            mLoc,
        }
    }

    // Assignment Operator
    //---------------------
    pub fn assign(&mut self, t: &riterator<T, XSIZE_MAX, YSIZE_MAX>) {
        self.mOwner = t.mOwner;
        for i in 0..2 {
            self.mMins[i] = t.mMins[i];
            self.mMaxs[i] = t.mMaxs[i];
            self.mLoc[i] = t.mLoc[i];
        }
    }

    // Equality & Inequality Operators
    //---------------------------------
    pub fn ne(&self, t: &riterator<T, XSIZE_MAX, YSIZE_MAX>) -> bool {
        self.mLoc[0] != t.mLoc[0] || self.mLoc[1] != t.mLoc[1]
    }
    pub fn eq(&self, t: &riterator<T, XSIZE_MAX, YSIZE_MAX>) -> bool {
        self.mLoc[0] == t.mLoc[0] && self.mLoc[1] == t.mLoc[1]
    }

    // Dereference Operator
    //----------------------
    pub fn deref(&mut self) -> &mut T {
        unsafe {
            // PORTING NOTE: Raw pointer dereference with x,y coordinates via get() method
            let grid_mut = self.mOwner as *mut grid2_vs<T, XSIZE_MAX, YSIZE_MAX>;
            (*grid_mut).get(self.mLoc[0], self.mLoc[1])
        }
    }

    // Inc Operator
    //--------------
    pub fn inc(&mut self) {
        if self.mLoc[1] <= self.mMaxs[1] {
            self.mLoc[0] += 1;
            if self.mLoc[0] > self.mMaxs[0] {
                self.mLoc[0] = self.mMins[0];
                self.mLoc[1] += 1;
            }
        }
    }

    pub fn at_end(&self) -> bool {
        self.mLoc[1] > self.mMaxs[1]
    }

    // Return True If On Frist Column Of A Row
    //-----------------------------------------
    pub fn onColZero(&self) -> bool {
        self.mLoc[0] == self.mMins[0]
    }

    // Evaluate The XY Position Of This Iterator
    //-------------------------------------------
    pub fn position(&self, X: &mut c_int, Y: &mut c_int) {
        *Y = self.mLoc[1];
        *X = self.mLoc[0];
    }
}
