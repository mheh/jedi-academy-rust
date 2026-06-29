////////////////////////////////////////////////////////////////////////////////////////
// RAVEN SOFTWARE - STAR WARS: JK II
//  (c) 2002 Activision
//
// Rail System
//
// The rail system is intended to provide a means for generating moving entities along
// tracks of varying speed and direction.  The entities are pulled from the map based
// upon their targets and recycled in random positions and order
//
////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::{c_int, c_char};

////////////////////////////////////////////////////////////////////////////////////////
// Externs & Fwd Decl.
////////////////////////////////////////////////////////////////////////////////////////
//extern cvar_t*		g_nav1;
extern "C" {
    fn G_SoundAtSpot(org: *const [f32; 3], soundIndex: c_int, broadcast: c_int);
}

// Forward declarations for classes
struct CRailTrack;
struct CRailLane;
struct CRailMover;

////////////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////////////
const MAX_TRACKS: usize = 4;
const MAX_LANES: usize = 8;
const MAX_MOVERS: usize = 150;
const MAX_MOVER_ENTS: usize = 10;
const MAX_MOVERS_TRACK: usize = 80;
const MAX_COLS: usize = 32;
const MAX_ROWS: usize = 96;
const MAX_ROW_HISTORY: usize = 10;

const WOOSH_DEBUG: c_int = 0;
const WOOSH_ALL_RANGE: f32 = 1500.0f32;
const WOOSH_SUPPORT_RANGE: f32 = 2500.0f32;
const WOOSH_TUNNEL_RANGE: f32 = 3000.0f32;

static mut mRailSystemActive: bool = false;

////////////////////////////////////////////////////////////////////////////////////////
// The Rail Track
//
// Tracks are the central component to the rails system.  They provide the master
// repositry of all movers and maintain the list of available movers as well as
//
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct CRailTrack {
    mName: hstring,
    mRow: c_int,
    mNumMoversPerRow: c_int,
    mNextUpdateTime: c_int,
    mNextUpdateDelay: c_int,
    mStartTime: c_int,
    mRows: c_int,
    mCols: c_int,
    mVertical: bool,
    mNegative: bool,
    mHAxis: c_int,
    mWAxis: c_int,
    mSpeedGridCellsPerSecond: c_int,
    mSpeedUnitsPerMillisecond: f32,
    mTravelTimeMilliseconds: c_int,
    mTravelDistanceUnits: f32,
    mDirection: CVec3,
    mVelocity: CVec3,
    mMins: CVec3,
    mMaxs: CVec3,
    mGridBottomLeftCorner: CVec3,
    mGridCenter: CVec3,
    mGridCellSize: f32,
    mCenterLocked: bool,
    mCells: ratl_grid2_vs,
    mMovers: ratl_vector_vs,
    mTestCols: ratl_vector_vs_int,
}

impl CRailTrack {
    fn Setup(&mut self, ent: *const gentity_t) {
        unsafe {
            self.mName = (*ent).targetname;
            self.mSpeedGridCellsPerSecond = (*ent).speed;
            self.mNumMoversPerRow = (*ent).count;
            self.mMins = (*ent).mins;
            self.mMaxs = (*ent).maxs;
            self.mStartTime = (*ent).delay + level.time;
            self.mGridCellSize = if (*ent).radius != 0.0f32 {
                (*ent).radius
            } else {
                1.0f32
            };
            self.mVertical = ((*ent).s.angles[1] == 90.0f32) || ((*ent).s.angles[1] == 270.0f32);
            self.mNegative = ((*ent).s.angles[1] == 180.0f32) || ((*ent).s.angles[1] == 270.0f32);	// From Maxs To Mins
            self.mWAxis = if self.mVertical { 0 } else { 1 };
            self.mHAxis = if self.mVertical { 1 } else { 0 };
            self.mTravelDistanceUnits = (*ent).maxs[self.mHAxis as usize] - (*ent).mins[self.mHAxis as usize];

            self.mRow = 0;
            self.mNextUpdateTime = 0;
            self.mCenterLocked = false;

            self.SnapVectorToGrid(&mut self.mMins);
            self.SnapVectorToGrid(&mut self.mMaxs);

            // Calculate Number Of Rows And Columns
            //--------------------------------------
            self.mRows = ((self.mMaxs[self.mHAxis as usize] - self.mMins[self.mHAxis as usize]) / self.mGridCellSize) as c_int;
            self.mCols = ((self.mMaxs[self.mWAxis as usize] - self.mMins[self.mWAxis as usize]) / self.mGridCellSize) as c_int;

            // Calculate Grid Center
            //-----------------------
            self.mGridCenter = [
                (self.mMins[0] + self.mMaxs[0]) * 0.5f32,
                (self.mMins[1] + self.mMaxs[1]) * 0.5f32,
                (self.mMins[2] + self.mMaxs[2]) * 0.5f32,
            ];
            self.SnapVectorToGrid(&mut self.mGridCenter);

            // Calculate Speed & Velocity
            //----------------------------
            self.mSpeedUnitsPerMillisecond = self.mSpeedGridCellsPerSecond as f32 * self.mGridCellSize / 1000.0f32;
            self.mTravelTimeMilliseconds = (self.mTravelDistanceUnits / self.mSpeedUnitsPerMillisecond) as c_int;

            AngleVectors((*ent).s.angles, &mut self.mDirection, core::ptr::null_mut(), core::ptr::null_mut());
            self.mDirection.SafeNorm();
            self.mVelocity = self.mDirection;
            self.mVelocity[0] *= self.mSpeedGridCellsPerSecond as f32 * self.mGridCellSize;
            self.mVelocity[1] *= self.mSpeedGridCellsPerSecond as f32 * self.mGridCellSize;
            self.mVelocity[2] *= self.mSpeedGridCellsPerSecond as f32 * self.mGridCellSize;

            self.mNextUpdateDelay = (1000.0f32 / (self.mSpeedGridCellsPerSecond as f32)) as c_int;

            // Calculate Bottom Left Corner
            //------------------------------
            self.mGridBottomLeftCorner = (*ent).mins;
            if (*ent).s.angles[1] == 180.0f32 {
                self.mGridBottomLeftCorner[0] = self.mMaxs[0];
            } else if (*ent).s.angles[1] == 270.0f32 {
                self.mGridBottomLeftCorner[1] = self.mMaxs[1];
            }
            self.SnapVectorToGrid(&mut self.mGridBottomLeftCorner);

            self.mCells = ratl_grid2_vs::new();
            self.mCells.set_size(self.mCols as usize, self.mRows as usize);
            self.mCells.init(core::ptr::null_mut());

            self.mMovers = ratl_vector_vs::new();

            if self.mNumMoversPerRow == 0 {
                self.mNumMoversPerRow = 3;
            }

            // Safe Clamp Number Of Rows & Cols
            //----------------------------------
            if self.mRows > (MAX_ROWS as c_int - 1) {
                self.mRows = (MAX_ROWS as c_int - 1);
                assert!(false);
            }
            if self.mCols > (MAX_COLS as c_int - 1) {
                self.mCols = (MAX_COLS as c_int - 1);
                assert!(false);
            }
        }
    }

    fn SnapVectorToGrid(&self, vec: &mut CVec3) {
        self.SnapFloatToGrid(&mut vec[0]);
        self.SnapFloatToGrid(&mut vec[1]);
    }

    fn SnapFloatToGrid(&self, f: &mut f32) {
        *f = (*f as c_int) as f32;

        let fNeg = *f < 0.0f32;
        if fNeg {
            *f *= -1.0f32;		// Temporarly make it positive
        }

        let Offset = ((*f as c_int) % (self.mGridCellSize as c_int)) as c_int;
        let OffsetAbs = if Offset < 0 { -Offset } else { Offset };
        let mut Offset_mut = Offset;
        if OffsetAbs > (self.mGridCellSize / 2.0f32) as c_int {
            Offset_mut = ((self.mGridCellSize as c_int - OffsetAbs) * -1) as c_int;
        }

        *f -= Offset_mut as f32;

        if fNeg {
            *f *= -1.0f32;		// Put It Back To Negative
        }

        *f = (*f as c_int) as f32;

        assert!(((*f as c_int) % (self.mGridCellSize as c_int)) == 0);
    }

    fn Update(&mut self) {
        // Stub - to be implemented
    }

    fn TestMoverInCells(&self, mover: *const CRailMover, atCol: c_int) -> bool {
        // Stub - to be implemented
        true
    }

    fn InsertMoverInCells(&mut self, mover: *const CRailMover, atCol: c_int) {
        // Stub - to be implemented
    }

    fn RandomizeTestCols(&mut self, startCol: c_int, stopCol: c_int) {
        // Stub - to be implemented
    }
}

// Stub types for unported dependencies
#[repr(C)]
struct CVec3([f32; 3]);

impl CVec3 {
    fn SafeNorm(&mut self) {
        // Stub
    }
}

#[repr(C)]
struct hstring {
    // Stub
}

#[repr(C)]
struct ratl_grid2_vs {
    // Stub
}

impl ratl_grid2_vs {
    fn new() -> Self {
        ratl_grid2_vs {}
    }

    fn set_size(&mut self, cols: usize, rows: usize) {
        // Stub
    }

    fn init(&mut self, val: *mut CRailMover) {
        // Stub
    }

    fn get(&self, col: c_int, row: c_int) -> *mut CRailMover {
        // Stub
        core::ptr::null_mut()
    }
}

#[repr(C)]
struct ratl_vector_vs {
    // Stub
}

impl ratl_vector_vs {
    fn new() -> Self {
        ratl_vector_vs {}
    }

    fn clear(&mut self) {
        // Stub
    }
}

#[repr(C)]
struct ratl_vector_vs_int {
    // Stub
}

impl ratl_vector_vs_int {
    fn new() -> Self {
        ratl_vector_vs_int {}
    }

    fn clear(&mut self) {
        // Stub
    }

    fn empty(&self) -> bool {
        // Stub
        true
    }

    fn size(&self) -> c_int {
        // Stub
        0
    }

    fn push_back(&mut self, val: c_int) {
        // Stub
    }

    fn erase_swap(&mut self, index: c_int) {
        // Stub
    }
}

// Stub types for game engine types
#[repr(C)]
struct gentity_t {
    // Stub
    targetname: hstring,
    speed: c_int,
    count: c_int,
    mins: CVec3,
    maxs: CVec3,
    delay: c_int,
    radius: f32,
    s: gentity_s,
}

#[repr(C)]
struct gentity_s {
    angles: [f32; 3],
}

static mut level: level_t = level_t { time: 0 };

#[repr(C)]
struct level_t {
    time: c_int,
}

extern "C" {
    fn AngleVectors(angles: [f32; 3], forward: *mut CVec3, right: *mut CVec3, up: *mut CVec3);
    fn G_SoundIndex(name: *const c_char) -> c_int;
    fn G_SetOrigin(ent: *mut gentity_t, origin: *const [f32; 3]);
    fn VectorCopy(from: *const [f32; 3], to: *mut [f32; 3]);
    fn VectorAdd(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorNormalize(v: *mut [f32; 3]) -> f32;
    fn VectorMA(start: *const [f32; 3], scale: f32, direction: *const [f32; 3], end: *mut [f32; 3]);
    fn DotProduct(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn gi_SetBrushModel(ent: *mut gentity_t, model: *const c_char);
    fn G_SpawnInt(key: *const c_char, defValue: *const c_char, out: *mut c_int);
    fn G_FreeEntity(ent: *mut gentity_t);
    fn gi_linkentity(ent: *mut gentity_t);
    fn gi_WE_IsOutside(pos: *const [f32; 3]) -> bool;
    fn CG_DrawEdge(start: *const [f32; 3], end: *const [f32; 3], color: c_int);
    static mut player: *mut gentity_t;
}

const EDGE_WHITE_TWOSECOND: c_int = 0;

////////////////////////////////////////////////////////////////////////////////////////
// The Rail Lane
//
//
//
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct CRailLane {
    mName: hstring,
    mNameTrack: hstring,
    mMins: CVec3,
    mMaxs: CVec3,
    mStartTime: c_int,
    mTrack: *mut CRailTrack,
    mMinCol: c_int,
    mMaxCol: c_int,
}

impl CRailLane {
    ////////////////////////////////////////////////////////////////////////////////////
    // From Entity Setup Spawn
    ////////////////////////////////////////////////////////////////////////////////////
    fn Setup(&mut self, ent: *const gentity_t) {
        unsafe {
            self.mName = (*ent).targetname;
            self.mNameTrack = (*ent).target;
            self.mMins = (*ent).mins;
            self.mMaxs = (*ent).maxs;
            self.mStartTime = (*ent).delay + level.time;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initialize
    //
    // This function scans through the list of tracks and hooks itself up with the
    // track
    ////////////////////////////////////////////////////////////////////////////////////
    fn Initialize(&mut self) {
        self.mTrack = core::ptr::null_mut();
        self.mMinCol = 0;
        self.mMaxCol = 0;

        //		int		dummy;
        // for (int i=0; i<mRailTracks.size(); i++)
        // {
        //     if (mRailTracks[i].mName==mNameTrack)
        //     {
        //         mTrack	= &(mRailTracks[i]);
        //         mTrack->SnapVectorToGrid(mMins);
        //         mTrack->SnapVectorToGrid(mMaxs);
        //
        //         mMinCol = (int)((mMins[mTrack->mWAxis] - mTrack->mMins[mTrack->mWAxis])/mTrack->mGridCellSize);
        //         mMaxCol = (int)((mMaxs[mTrack->mWAxis] - mTrack->mMins[mTrack->mWAxis] - (mTrack->mGridCellSize/2.0f))/mTrack->mGridCellSize);
        //
        //         //if (mTrack->mNegative)
        //         //{
        //         //	mMinCol = (mTrack->mCols - mMinCol - 1);
        //         //	mMaxCol = (mTrack->mCols - mMaxCol - 1);
        //         //}
        //
        //
        // //				mTrack->mCells.get_cell_coords(mMins[mTrack->mWAxis], 0, mMinCol, dummy);
        // //				mTrack->mCells.get_cell_coords((mMaxs[mTrack->mWAxis]-10.0f), 0, mMaxCol, dummy);
        //         break;
        //     }
        // }
        // assert(mTrack!=0);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct CRailMover {
    mEnt: *mut gentity_t,
    mCenter: bool,
    mOriginOffset: CVec3,
    mSoundPlayed: bool,
    mTrack: *mut CRailTrack,
    mLane: *mut CRailLane,
    mCols: c_int,
    mRows: c_int,
}

impl CRailMover {
    ////////////////////////////////////////////////////////////////////////////////////
    // From Entity Setup Spawn
    ////////////////////////////////////////////////////////////////////////////////////
    fn Setup(&mut self, ent: *mut gentity_t) {
        unsafe {
            self.mEnt = ent;
            self.mCenter = ((*ent).spawnflags & 1) != 0;
            self.mSoundPlayed = false;
            self.mOriginOffset = [
                ((*ent).mins[0] + (*ent).maxs[0]) * 0.5f32,
                ((*ent).mins[1] + (*ent).maxs[1]) * 0.5f32,
                0.0f32,
            ];

            (*ent).e_ReachedFunc = 0; // reachedF_NULL
            (*ent).moverState = 0; // MOVER_POS1
            (*ent).svFlags = 0; // SVF_USE_CURRENT_ORIGIN
            (*ent).s.eType = 0; // ET_MOVER
            (*ent).s.eFlags |= 0; // EF_NODRAW
            (*ent).contents = 0;
            (*ent).clipmask = 0;

            (*ent).s.pos.trType = 0; // TR_STATIONARY
            (*ent).s.pos.trDuration = 0;
            (*ent).s.pos.trTime = 0;

            // VectorCopy( ent->pos1, ent->currentOrigin );
            // VectorCopy( ent->pos1, ent->s.pos.trBase );

            // gi.linkentity(ent);
        }
    }

    fn Active(&self) -> bool {
        unsafe {
            assert!(self.mEnt != core::ptr::null_mut());
            level.time < ((*self.mEnt).s.pos.trDuration + (*self.mEnt).s.pos.trTime)
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Initialize
    //
    // This function scans through the list of tracks and hooks itself up with the
    // track (and possibly lane)
    ////////////////////////////////////////////////////////////////////////////////////
    fn Initialize(&mut self) {
        self.mTrack = core::ptr::null_mut();
        self.mLane = core::ptr::null_mut();
        self.mCols = 0;
        self.mRows = 0;

        // hstring	target = mEnt->target;
        // for (int track=0; track<mRailTracks.size(); track++)
        // {
        //     if (mRailTracks[track].mName==target)
        //     {
        //         mTrack = &(mRailTracks[track]);
        //         break;
        //     }
        // }
        // if (mTrack==0)
        // {
        //     for (int lane=0; lane<mRailLanes.size(); lane++)
        //     {
        //         if (mRailLanes[lane].mName==target)
        //         {
        //             mLane	= &(mRailLanes[lane]);
        //             mTrack	= mLane->mTrack;
        //             break;
        //         }
        //     }
        // }
        // assert(mTrack!=0);
        // if (mTrack)
        // {
        //     mTrack->mMovers.push_back(this);
        //     mCols	= (int)((mEnt->maxs[mTrack->mWAxis] - mEnt->mins[mTrack->mWAxis]) / mTrack->mGridCellSize) + 1;
        //     mRows	= (int)((mEnt->maxs[mTrack->mHAxis] - mEnt->mins[mTrack->mHAxis]) / mTrack->mGridCellSize) + 1;
        //
        //     // Make Sure The Mover Fits In The Track And Lane
        //     //------------------------------------------------
        //     if (mRows>mTrack->mRows)
        //     {
        // //				assert(0);
        //         mRows = mTrack->mRows;
        //     }
        //     if (mCols>mTrack->mCols)
        //     {
        // //				assert(0);
        //         mCols = mTrack->mCols;
        //     }
        //     if (mLane && mCols>(mLane->mMaxCol - mLane->mMinCol + 1))
        //     {
        // //				assert(0);
        //         mCols = (mLane->mMaxCol - mLane->mMinCol + 1);
        //     }
        // }
    }
}

static mut mRailTracks: [CRailTrack; MAX_TRACKS] = unsafe {
    [core::mem::zeroed(); MAX_TRACKS]
};

static mut mRailLanes: [CRailLane; MAX_LANES] = unsafe {
    [core::mem::zeroed(); MAX_LANES]
};

static mut mRailMovers: [CRailMover; MAX_MOVERS] = unsafe {
    [core::mem::zeroed(); MAX_MOVERS]
};

static mut mWooshSml: ratl_vector_vs_int = ratl_vector_vs_int {};	// Small Building
static mut mWooshMed: ratl_vector_vs_int = ratl_vector_vs_int {};	// Medium Building
static mut mWooshLar: ratl_vector_vs_int = ratl_vector_vs_int {};	// Large Building
static mut mWooshSup: ratl_vector_vs_int = ratl_vector_vs_int {};	// Track Support
static mut mWooshTun: ratl_vector_vs_int = ratl_vector_vs_int {};	// Tunnel

////////////////////////////////////////////////////////////////////////////////////////
/*QUAKED rail_track (0 .5 .8) ? x x x x x x x x
A rail track determines what location and direction rail_mover entities go.  Don't bother with any origin brushes.  Make sure to set:

"radius"     Number of units to break down into grid size
"speed"      Number of grid sized units per second rail_movers will go at
"angle"      The direction rail_movers will go
"count"		 The number of mover ents the track will try to add per row
"delay"		 How long the ent will wait from the start of the level before placing movers
*/
////////////////////////////////////////////////////////////////////////////////////////
pub extern "C" fn SP_rail_track(ent: *mut gentity_t) {
    unsafe {
        gi_SetBrushModel(ent, (*ent).model);
        let mut delay = 0c_int;
        G_SpawnInt(b"delay\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut delay);
        // mRailTracks.push_back().Setup(ent);
        G_FreeEntity(ent);
        mRailSystemActive = true;
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Rail Lane
//
//
//
////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////
/*QUAKED rail_lane (0 .5 .8) ? x x x x x x x x
Use rail lanes to split up tracks.  Just target it to a track that you want to break up into pieces

"delay"		 How long the ent will wait from the start of the level before placing movers
*/
////////////////////////////////////////////////////////////////////////////////////////
pub extern "C" fn SP_rail_lane(ent: *mut gentity_t) {
    unsafe {
        gi_SetBrushModel(ent, (*ent).model);
        let mut delay = 0c_int;
        G_SpawnInt(b"delay\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, &mut delay);
        // mRailLanes.push_back().Setup(ent);
        G_FreeEntity(ent);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
pub extern "C" fn SP_rail_mover(ent: *mut gentity_t) {
    unsafe {
        gi_SetBrushModel(ent, (*ent).model);
        // mRailMovers.push_back().Setup(ent);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
pub extern "C" fn Rail_Reset() {
    unsafe {
        mRailSystemActive = false;
        // mRailTracks.clear();
        // mRailLanes.clear();
        // mRailMovers.clear();
        //
        // mWooshSml.clear();
        // mWooshMed.clear();
        // mWooshLar.clear();
        // mWooshSup.clear();
        // mWooshTun.clear();
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
pub extern "C" fn Rail_Initialize() {
    unsafe {
        // for (int lane=0; lane<mRailLanes.size(); lane++)
        // {
        //     mRailLanes[lane].Initialize();
        // }
        //
        // for (int mover=0; mover<mRailMovers.size(); mover++)
        // {
        //     mRailMovers[mover].Initialize();
        // }

        // Precache All The Woosh Sounds
        //-------------------------------
        // if (!mRailMovers.empty())
        // {
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh1\0".as_ptr() as *const c_char));
        mWooshSml.push_back(G_SoundIndex(b"sound/effects/woosh2\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh3\0".as_ptr() as *const c_char));
        mWooshSml.push_back(G_SoundIndex(b"sound/effects/woosh4\0".as_ptr() as *const c_char));
        mWooshLar.push_back(G_SoundIndex(b"sound/effects/woosh5\0".as_ptr() as *const c_char));
        mWooshSml.push_back(G_SoundIndex(b"sound/effects/woosh6\0".as_ptr() as *const c_char));
        mWooshSup.push_back(G_SoundIndex(b"sound/effects/woosh7\0".as_ptr() as *const c_char));
        mWooshSup.push_back(G_SoundIndex(b"sound/effects/woosh8\0".as_ptr() as *const c_char));
        mWooshSup.push_back(G_SoundIndex(b"sound/effects/woosh9\0".as_ptr() as *const c_char));
        mWooshLar.push_back(G_SoundIndex(b"sound/effects/woosh10\0".as_ptr() as *const c_char));
        mWooshLar.push_back(G_SoundIndex(b"sound/effects/woosh11\0".as_ptr() as *const c_char));
        mWooshLar.push_back(G_SoundIndex(b"sound/effects/woosh12\0".as_ptr() as *const c_char));
        mWooshSml.push_back(G_SoundIndex(b"sound/effects/woosh13\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh14\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh15\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh16\0".as_ptr() as *const c_char));
        mWooshSml.push_back(G_SoundIndex(b"sound/effects/woosh17\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh18\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh19\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh20\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh21\0".as_ptr() as *const c_char));
        mWooshLar.push_back(G_SoundIndex(b"sound/effects/woosh22\0".as_ptr() as *const c_char));
        mWooshLar.push_back(G_SoundIndex(b"sound/effects/woosh23\0".as_ptr() as *const c_char));
        mWooshSup.push_back(G_SoundIndex(b"sound/effects/woosh24\0".as_ptr() as *const c_char));
        mWooshSup.push_back(G_SoundIndex(b"sound/effects/woosh25\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh26\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh27\0".as_ptr() as *const c_char));
        mWooshMed.push_back(G_SoundIndex(b"sound/effects/woosh28\0".as_ptr() as *const c_char));
        mWooshLar.push_back(G_SoundIndex(b"sound/effects/woosh29\0".as_ptr() as *const c_char));
        mWooshTun.push_back(G_SoundIndex(b"sound/effects/whoosh_tunnel\0".as_ptr() as *const c_char));
        // }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
pub extern "C" fn Rail_Update() {
    unsafe {
        if mRailSystemActive {
            // && false)
            // for (int track=0; track<mRailTracks.size(); track++)
            // {
            //     if (level.time>mRailTracks[track].mNextUpdateTime && !mRailTracks[track].mMovers.empty())
            //     {
            //         mRailTracks[track].Update();
            //     }
            // }

            // Is The Player Outside?
            //------------------------
            if !player.is_null() && gi_WE_IsOutside((*player).currentOrigin) {
                let mut wooshSound: c_int = 0;
                let mut wooshSoundPos: [f32; 3] = [0.0f32; 3];
                let mut moverOrigin: [f32; 3] = [0.0f32; 3];
                let mut playerToMover: [f32; 3] = [0.0f32; 3];
                let mut playerToMoverDistance: f32 = 0.0f32;
                let mut playerToMoverDistanceFraction: f32 = 0.0f32;

                // Iterate Over All The Movers
                //-----------------------------
                // for (int moverIndex=0; moverIndex<mRailMovers.size(); moverIndex++)
                // {
                //     CRailMover&	mover = mRailMovers[moverIndex];
                //
                //     // Is It Active, And Has The Sound Already Played On It?
                //     //--------------------------------------------------------
                //     if (mover.Active() && !mover.mSoundPlayed)
                //     {
                //         VectorAdd(mover.mEnt->currentOrigin, mover.mOriginOffset.v, moverOrigin);
                //         VectorSubtract(moverOrigin, player->currentOrigin, playerToMover);
                //         playerToMover[2]		= 0.0f;
                //         playerToMoverDistance	= VectorNormalize(playerToMover);
                //
                //
                //         // Is It Close Enough?
                //         //---------------------
                //         if ((( mover.mLane || !mover.mCenter) &&								// Not Center Track
                //              (playerToMoverDistance<WOOSH_ALL_RANGE) && 						//  And Close Enough
                //              (DotProduct(playerToMover, mover.mTrack->mDirection.v)>-0.45f))	//  And On The Side
                //             ||																	//OR
                //             ((!mover.mLane &&  mover.mCenter) &&								// Is Center Track
                //               (playerToMoverDistance<WOOSH_SUPPORT_RANGE ||						//  And Close Enough for Support
                //               (playerToMoverDistance<WOOSH_TUNNEL_RANGE && mover.mRows>10))		//   Or Close Enough For Tunnel
                //              ))
                //         {
                //             mover.mSoundPlayed = true;
                //             wooshSound = 0;
                //
                //             // The Centered Entities Play Right On The Player's Head For Full Volume
                //             //-----------------------------------------------------------------------
                //             if (mover.mCenter && !mover.mLane)
                //             {
                //                 VectorCopy(player->currentOrigin, wooshSoundPos);
                //                 wooshSoundPos[2] += 50;
                //
                //                 // If It Is Very Long, Play The Tunnel Sound
                //                 //-------------------------------------------
                //                 if (mover.mRows>10)
                //                 {
                //                     wooshSound = mWooshTun[Q_irand(0, mWooshTun.size()-1)];
                //                 }
                //
                //                 // Otherwise It Is A Support
                //                 //---------------------------
                //                 else
                //                 {
                //                     wooshSound = mWooshSup[Q_irand(0, mWooshSup.size()-1)];
                //                 }
                //             }
                //
                //             // All Other Entities Play At A Fraction Of Their Normal Range
                //             //-------------------------------------------------------------
                //             else
                //             {
                //                 // Scale The Play Pos By The Square Of The Distance
                //                 //--------------------------------------------------
                //                 playerToMoverDistanceFraction = playerToMoverDistance/WOOSH_ALL_RANGE;
                //                 playerToMoverDistanceFraction *= playerToMoverDistanceFraction;
                //                 playerToMoverDistanceFraction *= 0.6f;
                //                 playerToMoverDistance *= playerToMoverDistanceFraction;
                //                 VectorMA(player->currentOrigin, playerToMoverDistance, playerToMover, wooshSoundPos);
                //
                //                 // Large Building
                //                 //----------------
                //                 if (mover.mRows>4)
                //                 {
                //                     wooshSound = mWooshLar[Q_irand(0, mWooshLar.size()-1)];
                //                 }
                //
                //                 // Medium Building
                //                 //-----------------
                //                 else if (mover.mRows>2)
                //                 {
                //                     wooshSound = mWooshMed[Q_irand(0, mWooshMed.size()-1)];
                //                 }
                //
                //                 // Small Building
                //                 //----------------
                //                 else
                //                 {
                //                     wooshSound = mWooshSml[Q_irand(0, mWooshSml.size()-1)];
                //                 }
                //             }
                //
                //             // If A Woosh Sound Was Selected, Play It Now
                //             //--------------------------------------------
                //             if (wooshSound)
                //             {
                //                 G_SoundAtSpot(wooshSoundPos, wooshSound, qfalse);
                //                 if (WOOSH_DEBUG)
                //                 {
                //                     CG_DrawEdge(player->currentOrigin, wooshSoundPos, EDGE_WHITE_TWOSECOND);
                //                 }
                //             }
                //         }
                //     }
                // }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
pub extern "C" fn Rail_LockCenterOfTrack(trackName: *const c_char) {
    unsafe {
        // hstring	name = trackName;
        // for (int track=0; track<mRailTracks.size(); track++)
        // {
        //     if (mRailTracks[track].mName==name)
        //     {
        //         mRailTracks[track].mCenterLocked = true;
        //         return;
        //     }
        // }
        // assert(0);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
//
////////////////////////////////////////////////////////////////////////////////////////
#[allow(non_snake_case)]
pub extern "C" fn Rail_UnLockCenterOfTrack(trackName: *const c_char) {
    unsafe {
        // hstring	name = trackName;
        // for (int track=0; track<mRailTracks.size(); track++)
        // {
        //     if (mRailTracks[track].mName==name)
        //     {
        //         mRailTracks[track].mCenterLocked = false;
        //         return;
        //     }
        // }
        // assert(0);
    }
}
