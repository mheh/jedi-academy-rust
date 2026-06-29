#![allow(non_snake_case)]

use std::collections::{BTreeSet, BTreeMap};
use std::cmp::Ordering;
use core::ffi::c_int;

// Type definitions (would come from ff.h and ff_snd.h)
pub type qboolean = c_int;
pub type ffHandle_t = *mut ChannelCompound;

// Stub type definitions for external types
#[repr(C)]
pub struct ChannelCompound {
    _private: [u8; 0],
}

#[repr(C)]
pub struct FFSystem {
    _private: [u8; 0],
}

//
// Constants
//
const FF_GAIN_STEP: c_int = 500;
const FF_HANDLE_NULL: ffHandle_t = std::ptr::null_mut();

// qboolean values
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// External declarations
extern "C" {
    pub static mut gFFSystem: FFSystem;

    pub fn FF_Stop(handle: ffHandle_t);
    pub fn FF_Play(handle: ffHandle_t);
}

//
//	Internal data structures
//
//	This whole system should mirror snd_dma.cpp to some degree.
//	Right now, not much works.

/*
template<typename T>
static T RelativeDistance( T Volume, T Min, T Max )
{
	if ( Min == Max )
		if ( Volume < Min )
			return 0.f;
		else
			return 1.f;

	return (Volume - Min) / (Max - Min);
};

template<typename T>
int Round( T value, int mod )
{
	int intval = (int)value;
	int intmod = intval % mod;
	int roundup = intmod >= mod / 2;

	return
	(	intval
	?	roundup
	?	intval + mod - intmod
	:	intval - intmod
	:	roundup
	?	mod
	:	0
	);
}
*/

#[derive(Clone)]
pub struct SndForce {
    pub mHandle: ffHandle_t,
    pub mRefs: c_int,
    pub mPlaying: qboolean,
    //	int mEntNum;
    //	vec3_t mOrigin;
    //	struct SDistanceLimits
    //	{	int	min;
    //		int max;
    //	}	mDistance;
}

impl SndForce {
    pub fn zero(&mut self) {
        self.mHandle = FF_HANDLE_NULL;
        self.mRefs = 0;
        self.mPlaying = qfalse;
        //		mEntNum = 0;
        //		mDistance.min = 0;
        //		mDistance.max = 0;
        //		mOrigin[0] = 1.f;
        //		mOrigin[1] = 0.f;
        //		mOrigin[2] = 0.f;
    }

    pub fn new() -> Self {
        let mut sf = SndForce {
            mHandle: FF_HANDLE_NULL,
            mRefs: 0,
            mPlaying: qfalse,
        };
        sf.zero();
        sf
    }

    pub fn from_copy(other: &SndForce) -> Self {
        other.clone()
    }

    pub fn with_handle(handle: ffHandle_t) -> Self {
        //	/*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
        SndForce {
            mHandle: handle,
            mRefs: 0,
            mPlaying: qfalse,
            //	,	mEntNum( entNum )
        }
        //		mDistance.min = minDistance;
        //		mDistance.max = maxDistance;
        //		memcpy( mOrigin, origin, sizeof(mOrigin) );
    }

    pub fn AddRef(&mut self) {
        self.mRefs += 1;
    }

    pub fn SubRef(&mut self) {
        self.mRefs -= 1;
    }

    pub fn Update(&self) -> qboolean {
        (if self.mRefs != 0 && !self.mHandle.is_null() { 1 } else { 0 }) as qboolean
    }

    /*	int GetGain()
    {
        float distance = 1.f - GetRelativeDistance();

        return distance == 0.f
        ?	10000
        :	Clamp<int>
            (	Round<int>
                (	distance * 10000
                ,	FF_GAIN_STEP
                )
            ,	0
            ,	10000
            )
        ;
    }
    float GetRelativeDistance()
    {
        return !mRefs
        ?	1.f
        :	IsOrigin()
        ?	0.f
        :	RelativeDistance<float>
            (	sqrt
                (	mOrigin[0] * mOrigin[0]
                +	mOrigin[1] * mOrigin[1]
                +	mOrigin[2] * mOrigin[2]
                )
            ,	mDistance.min
            ,	mDistance.max
            ) / mRefs
        ;
    }
    qboolean IsOrigin()
    {
        return qboolean
        (	!mOrigin[0]
        &&	!mOrigin[1]
        &&	!mOrigin[2]
        );
    }
    void Respatialize( int entNum, const vec3_t origin )
    {
        extern vec3_t s_entityPosition[];

        if ( mEntNum != entNum )
        {
            // Assumes all forces follow its entity and is centered on entity
            mOrigin[0] = s_entityPosition[ entNum ][0] - origin[0];
            mOrigin[1] = s_entityPosition[ entNum ][1] - origin[1];
            mOrigin[2] = s_entityPosition[ entNum ][2] - origin[2];
        }
        else
        {
            memset( mOrigin, 0, sizeof(mOrigin) );
        }
    }*/
}

impl std::ops::AddAssign<&SndForce> for SndForce {
    fn add_assign(&mut self, other: &SndForce) {
        self.operator_plus_assign(other);
    }
}

// Fancy comparator
#[derive(Clone)]
pub struct SndForceLess;

impl SndForceLess {
    fn compare(x: &SndForce, y: &SndForce) -> bool {
        /* x.mEntNum < y.mEntNum
        ||*/x.mHandle < y.mHandle
        //		||	x.mOrigin < y.mOrigin		// uhhh... compare components
        || x.mPlaying < y.mPlaying
    }
}

impl std::cmp::PartialOrd for SndForce {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for SndForce {
    fn cmp(&self, other: &Self) -> Ordering {
        // Comparison order: mHandle first, then mPlaying
        match self.mHandle.cmp(&other.mHandle) {
            Ordering::Equal => self.mPlaying.cmp(&other.mPlaying),
            other_ord => other_ord,
        }
    }
}

impl std::cmp::PartialEq for SndForce {
    fn eq(&self, other: &Self) -> bool {
        self.mHandle == other.mHandle && self.mPlaying == other.mPlaying
    }
}

impl std::cmp::Eq for SndForce {}

#[derive(Clone)]
pub struct LoopForce {
    // Composition-based inheritance from SndForce
    base: SndForce,
}

impl LoopForce {
    pub fn new() -> Self {
        LoopForce {
            base: SndForce::new(),
        }
    }

    pub fn from_copy(other: &LoopForce) -> Self {
        other.clone()
    }

    pub fn with_handle(handle: ffHandle_t) -> Self {
        LoopForce {
            base: SndForce::with_handle(handle),
        }
        //	/*, int entNum, , const vec3_t origin, float maxDistance, float minDistance*/
    }

    pub fn Add(&mut self, ff: ffHandle_t) {
        //	/*, int entNum, const vec3_t origin*/
        // Implementation would go here
    }

    //	void Respatialize( int entNum, const vec3_t origin );
    pub fn Update(&mut self) -> qboolean {
        let result = self.base.Update();
        self.base.mRefs = 0;
        result
    }

    // Delegate methods to base
    pub fn AddRef(&mut self) {
        self.base.AddRef();
    }

    pub fn SubRef(&mut self) {
        self.base.SubRef();
    }

    // Direct field access for compatibility
    pub fn mHandle(&self) -> ffHandle_t {
        self.base.mHandle
    }

    pub fn set_mHandle(&mut self, handle: ffHandle_t) {
        self.base.mHandle = handle;
    }

    pub fn mRefs(&self) -> c_int {
        self.base.mRefs
    }

    pub fn set_mRefs(&mut self, refs: c_int) {
        self.base.mRefs = refs;
    }

    pub fn mPlaying(&self) -> qboolean {
        self.base.mPlaying
    }

    pub fn set_mPlaying(&mut self, playing: qboolean) {
        self.base.mPlaying = playing;
    }
}

impl std::cmp::PartialEq for LoopForce {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base
    }
}

impl std::cmp::Eq for LoopForce {}

impl std::cmp::PartialOrd for LoopForce {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for LoopForce {
    fn cmp(&self, other: &Self) -> Ordering {
        self.base.cmp(&other.base)
    }
}

pub struct SndForceSet {
    // typedef set<SndForce, SndForceLess> PendingSet;
    // typedef map<ffHandle_t, SndForce> ActiveSet;
    pub mActive: BTreeMap<ffHandle_t, SndForce>,
    pub mPending: BTreeSet<SndForce>,
}

impl SndForceSet {
    pub fn new() -> Self {
        SndForceSet {
            mActive: BTreeMap::new(),
            mPending: BTreeSet::new(),
        }
    }

    pub fn Add(&mut self, handle: ffHandle_t) {
        //	/*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
        let mut sf = SndForce::with_handle(handle);
        // const_cast <SndForce&> (*mPending.insert( SndForce( handle/*, entNum, origin, maxDistance, minDistance*/ ) ).first).AddRef();
        sf.AddRef();
        self.mPending.insert(sf);
    }

    pub fn Update(&mut self) -> qboolean {
        /*
        // Remove finished effects from active //and pending sets
        for
        (	itActive = mActive.begin()
        ;	itActive != mActive.end()
        ;	//itActive++
        ){
            if ( gFFSystem.IsPlaying( (*itActive).first ) )
            {
                ++itActive;
            }
            else
            {
#if( 0 )
                for
                (	itPending = mPending.begin()
                ;	itPending != mPending.end()
                ;	itPending++
                ){
                    if
                    (	(*itPending).mHandle == (*itActive).first
                    &&	(*itPending).mPlaying
                    ){
                        itPending = mPending.erase( itPending );
                    }
                }
#endif
                itActive = mActive.erase( itActive );
            }
        }
        */

        // Sum effects
        let mut start: BTreeMap<ffHandle_t, SndForce> = BTreeMap::new();
        for pending_force in &self.mPending {
            if pending_force.Update() != 0 {
                start
                    .entry(pending_force.mHandle)
                    .or_insert_with(|| pending_force.clone())
                    .add_assign_impl(pending_force);
            }
        }

        // Decide whether to start ( no updating one-shots )
        for (handle, _snd_force) in &start {
            /*		SndForce &sndForce = mActive[ (*itActive).first ];
            sndForce.mHandle = (*itActive).first;
            if ( (*itActive).second.GetGain() >= sndForce.GetGain() )
            {
                //gFFSystem.ChangeGain( sndForce.mHandle, sndForce.GetGain() );
                FF_Start( sndForce.mHandle );
                sndForce.mPlaying = qtrue;
            }
            */
            unsafe {
                FF_Play(*handle);
            }
        }

        self.mPending.clear();

        qfalse
    }
}

pub struct LoopForceSet {
    // typedef set<LoopForce, SndForceLess> PendingSet;
    // typedef map<ffHandle_t, LoopForce> ActiveSet;
    pub mActive: BTreeMap<ffHandle_t, LoopForce>,
    pub mPending: BTreeSet<LoopForce>,
}

impl LoopForceSet {
    pub fn new() -> Self {
        LoopForceSet {
            mActive: BTreeMap::new(),
            mPending: BTreeSet::new(),
        }
    }

    pub fn Add(&mut self, handle: ffHandle_t) {
        //	/*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
        let mut lf = LoopForce::with_handle(handle);
        // const_cast <LoopForce&>(*mPending.insert( LoopForce( handle/*, entNum, origin, maxDistance, minDistance*/ ) ).first).AddRef();
        lf.AddRef();
        self.mPending.insert(lf);
    }

    pub fn Update(&mut self) -> qboolean {
        // Sum effects
        let mut active: BTreeMap<ffHandle_t, LoopForce> = BTreeMap::new();

        for pending_force in &self.mPending {
            let mut mutable_pending = pending_force.clone();
            if mutable_pending.Update() != 0 {
                active
                    .entry(mutable_pending.mHandle())
                    .or_insert_with(|| mutable_pending.clone())
                    .add_assign_impl(&mutable_pending);
            }
        }

        // Stop and remove unreferenced effects
        let mut itActive = self.mActive.iter_mut();
        loop {
            match itActive.next() {
                None => break,
                Some((handle, snd_force)) => {
                    if active.contains_key(handle) {
                        // Keep it, continue
                        continue;
                    } else {
                        // Stop and remove
                        unsafe {
                            FF_Stop(snd_force.mHandle());
                        }
                        // Can't remove while iterating; would need to collect and remove after
                    }
                }
            }
        }

        // Remove keys not in active from mActive
        let keys_to_remove: Vec<_> = self
            .mActive
            .iter()
            .filter_map(|(k, _)| {
                if !active.contains_key(k) {
                    Some(*k)
                } else {
                    None
                }
            })
            .collect();
        for key in keys_to_remove {
            self.mActive.remove(&key);
        }

        // Decide whether to start or update
        for (_, active_force) in &active {
            let snd_force = self
                .mActive
                .entry(active_force.mHandle())
                .or_insert_with(|| active_force.clone());
            snd_force.set_mHandle(active_force.mHandle());
            if snd_force.mPlaying() != 0 {
                // Just update it

                //			if ( (*itActive).second.GetGain() != sndForce.GetGain() )
                //			{
                //				gFFSystem.ChangeGain( sndForce.mHandle, sndForce.GetGain() );
                //			}
            } else {
                // Update and start it

                //			gFFSystem.ChangeGain( sndForce.mHandle, sndForce.GetGain() );
                unsafe {
                    FF_Play(snd_force.mHandle());
                }
                snd_force.set_mPlaying(qtrue);
            }
        }

        self.mPending.clear();

        qtrue
    }

    /*	void Respatialize( int entNum, const vec3_t origin )
    {
        for
        (	PendingSet::iterator itPending = mPending.begin()
        ;	itPending != mPending.end()
        ;	itPending++
        ){
            (*itPending).Respatialize( entNum, origin );
        }
    }*/
}

pub struct MasterForceSet {
    mEntityNum: c_int,
    //	vec3_t mOrigin;
    mSnd: SndForceSet,
    mLoop: LoopForceSet,
}

impl MasterForceSet {
    pub fn new() -> Self {
        MasterForceSet {
            mEntityNum: 0,
            mSnd: SndForceSet::new(),
            mLoop: LoopForceSet::new(),
        }
    }

    pub fn Add(&mut self, handle: ffHandle_t) {
        //	/*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
        self.mSnd.Add(handle);
        //	/*, entNum, origin, maxDistance, minDistance*/
    }

    pub fn AddLoop(&mut self, handle: ffHandle_t) {
        //	/*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
        self.mLoop.Add(handle);
        //	/*, entNum, origin, maxDistance, minDistance*/
    }

    /*	void Respatialize( int entNum, const vec3_t origin )
    {
        memcpy( mOrigin, origin, sizeof(mOrigin) );
        mEntityNum = entNum;
        mSnd.Respatialize( entNum, origin );
        mLoop.Respatialize( entNum, origin );
    }
    */

    pub fn Update(&mut self) {
        self.mSnd.Update();
        self.mLoop.Update();
    }
}

//
//	===================================================================================
//

pub static mut _MasterForceSet: Option<MasterForceSet> = None;

pub fn FF_AddForce(ff: ffHandle_t) {
    //	/*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
    unsafe {
        if _MasterForceSet.is_none() {
            _MasterForceSet = Some(MasterForceSet::new());
        }
        if let Some(ref mut master) = _MasterForceSet {
            master.Add(ff);
            //	/*, entNum, origin, maxDistance, minDistance*/
        }
    }
}

pub fn FF_AddLoopingForce(ff: ffHandle_t) {
    //	/*, int entNum, const vec3_t origin, float maxDistance, float minDistance*/
    unsafe {
        if _MasterForceSet.is_none() {
            _MasterForceSet = Some(MasterForceSet::new());
        }
        if let Some(ref mut master) = _MasterForceSet {
            master.AddLoop(ff);
            //	/*, entNum, origin, maxDistance, minDistance*/
        }
    }
}

/*
void FF_Respatialize( int entNum, const vec3_t origin )
{
    _MasterForceSet.Respatialize( entNum, origin );
}
*/

pub fn FF_Update() {
    unsafe {
        if let Some(ref mut master) = _MasterForceSet {
            master.Update();
        }
    }
}

//
//	===================================================================================
//

////-----------------
/// LoopForce::Update
//---------------------
//	Starts/Stops/Updates looping forces.
//	Call once per frame after all looping forces have been added and respatialized.
//

impl SndForce {
    fn operator_plus_assign(&mut self, other: &SndForce) {
        /*
        float dist = other.GetRelativeDistance();

        if ( dist < 1.f )
        {
            float thisdist = GetRelativeDistance();

            if ( thisdist < 1.f )
            {
                if ( dist == 0.f || thisdist == 0.f )
                {
                    mOrigin[0] = 0.f;
                    mOrigin[1] = 0.f;
                    mOrigin[2] = 0.f;
                }
                else
                {
                    // This is so shitty
                    mOrigin[0] *= dist;
                    mOrigin[1] *= dist;
                    mOrigin[2] *= dist;
                }
            }
            else
            {
                memcpy( mOrigin, other.mOrigin, sizeof(mOrigin) );
            }
        */

        self.mRefs += other.mRefs;
        //	}
    }

    fn add_assign_impl(&mut self, other: &SndForce) {
        self.operator_plus_assign(other);
    }
}

impl LoopForce {
    fn add_assign_impl(&mut self, other: &LoopForce) {
        self.base.add_assign_impl(&other.base);
    }
}
