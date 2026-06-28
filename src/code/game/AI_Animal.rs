// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

// Local stub for ratl::vector_vs<gentity_t*, MAX_PACKS>
// Mirrors the fixed-size vector behavior used by the original C++ code
struct VectorVs<T, const N: usize> {
    data: [Option<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> VectorVs<T, N> {
    fn new() -> Self {
        VectorVs {
            data: [None; N],
            len: 0,
        }
    }

    fn size(&self) -> usize {
        self.len
    }

    fn full(&self) -> bool {
        self.len >= N
    }

    fn push_back(&mut self, item: T) {
        if self.len < N {
            self.data[self.len] = Some(item);
            self.len += 1;
        }
    }

    fn erase_swap(&mut self, index: usize) {
        if index < self.len {
            if index < self.len - 1 {
                self.data[index] = self.data[self.len - 1];
            }
            self.data[self.len - 1] = None;
            self.len -= 1;
        }
    }
}

impl<T: Copy, const N: usize> core::ops::Index<usize> for VectorVs<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if let Some(item) = self.data[index] {
            unsafe { &*((&item) as *const T) }
        } else {
            panic!("Index out of bounds or None value")
        }
    }
}

impl<T: Copy, const N: usize> core::ops::IndexMut<usize> for VectorVs<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(item) = &mut self.data[index] {
            unsafe { &mut *(item as *mut T) }
        } else {
            panic!("Index out of bounds or None value")
        }
    }
}

// Stub types for entities and game structures
#[repr(C)]
pub struct gclient_t {
    pub leader: *mut gentity_t,
    pub confusionTime: c_int,
    pub charmedTime: c_int,
    pub investigateSoundDebounceTime: c_int,
}

#[repr(C)]
pub struct gNPC_t {
    pub investigateSoundDebounceTime: c_int,
    pub confusionTime: c_int,
    pub charmedTime: c_int,
    pub investigateGoal: [f32; 3],
    pub aiFlags: c_int,
    pub investigateDebounceTime: c_int,
}

#[repr(C)]
pub struct gentity_t {
    pub currentOrigin: [f32; 3],
    pub currentAngles: [f32; 3],
    pub health: c_int,
    pub client: *mut gclient_t,
    pub inuse: c_int,
    pub lastMoveTime: c_int,
    pub followPos: [f32; 3],
    pub followPosWaypoint: c_int,
    pub resultspeed: f32,
}

#[repr(C)]
pub struct alertEvent_t {
    pub owner: *mut gentity_t,
    pub position: [f32; 3],
    pub radius: f32,
}

#[repr(C)]
pub struct level_t {
    pub alertEvents: *mut alertEvent_t,
    pub time: c_int,
}

// Constants
const MAX_PACKS: usize = 10;

const LEAVE_PACK_DISTANCE: f32 = 1000.0;
const JOIN_PACK_DISTANCE: f32 = 800.0;
const WANDER_RANGE: f32 = 1000.0;
const FRIGHTEN_DISTANCE: f32 = 300.0;

// External C function declarations
extern "C" {
    fn G_PlayerSpawned() -> c_int;
    fn Distance(a: *const f32, b: *const f32) -> f32;
    fn NPC_CheckAlertEvents(
        bAlerts: c_int,
        bInvestigate: c_int,
        iIgnoreAlert: c_int,
        bSetAlertLevel: c_int,
        alertLevel: c_int,
        bUnAlert: c_int,
    ) -> c_int;
    fn NPC_UpdateAngles(bAdjustPitch: c_int, bAdjustYaw: c_int);

    // Namespace-like functions (stub declarations)
    fn STEER_Activate(NPC: *mut gentity_t);
    fn STEER_DeActivate(NPC: *mut gentity_t, ucmd: *mut c_void);
    fn STEER_Stop(NPC: *mut gentity_t);
    fn STEER_Flee(NPC: *mut gentity_t, pos: *const f32);
    fn STEER_FollowLeader(NPC: *mut gentity_t, leader: *mut gentity_t, dist: f32);
    fn STEER_Seek(
        NPC: *mut gentity_t,
        pos: *const f32,
        slowingDistance: f32,
        weight: f32,
        speed: f32,
    );
    fn STEER_Separation(NPC: *mut gentity_t, distance: f32);
    fn STEER_AvoidCollisions(NPC: *mut gentity_t, leader: *mut gentity_t);
    fn STEER_AvoidCollisions_single(NPC: *mut gentity_t);
    fn STEER_Path(NPC: *mut gentity_t);
    fn STEER_Wander(NPC: *mut gentity_t);

    fn NAV_GoTo(NPC: *mut gentity_t, waypoint: c_int);
    fn NAV_ClearPath(NPC: *mut gentity_t);
    fn NAV_HasPath(NPC: *mut gentity_t) -> c_int;
    fn NAV_UpdatePath(NPC: *mut gentity_t) -> c_int;
    fn NAV_FindPath(NPC: *mut gentity_t, waypoint: c_int);
    fn NAV_GetNearestNode(NPC: *mut gentity_t) -> c_int;
    fn NAV_ChooseRandomNeighbor(node: c_int) -> c_int;
    fn NAV_OnNeighboringPoints(node1: c_int, node2: c_int) -> c_int;

    fn Q_irand(min: c_int, max: c_int) -> c_int;

    // Global variables
    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut gNPC_t;
    static mut player: *mut gentity_t;
    static mut level: level_t;
    static mut ucmd: c_void;
}

// Global pack vector - static mutable
static mut mPacks: VectorVs<*mut gentity_t, MAX_PACKS> = VectorVs {
    data: [None; MAX_PACKS],
    len: 0,
};

////////////////////////////////////////////////////////////////////////////////////////
// Update The Packs, Delete Dead Leaders, Join / Split Packs, Find MY Leader
////////////////////////////////////////////////////////////////////////////////////////
unsafe extern "C" fn NPC_AnimalUpdateLeader() -> *mut gentity_t {
    // Find The Closest Pack Leader, Not Counting Myself
    //---------------------------------------------------
    let mut closestLeader: *mut gentity_t = core::ptr::null_mut();
    let mut closestDist: f32 = 0.0;
    let mut myLeaderNum: usize = 0;

    let mut i = 0;
    while i < mPacks.size() {
        // Dump Dead Leaders
        //-------------------
        if mPacks.data[i].is_none()
            || (*mPacks.data[i].unwrap()).health <= 0
        {
            if mPacks.data[i] == Some((*NPC).client)
                || mPacks.data[i].unwrap() == (*(*NPC).client).leader
            {
                (*(*NPC).client).leader = core::ptr::null_mut();
            }

            mPacks.erase_swap(i);

            if i >= mPacks.size() {
                closestLeader = core::ptr::null_mut();
                break;
            }
        } else {
            // Don't Count Self
            //------------------
            if mPacks.data[i].unwrap() == NPC {
                myLeaderNum = i;
            } else {
                let dist: f32 = Distance(
                    (*mPacks.data[i].unwrap()).currentOrigin.as_ptr(),
                    (*NPC).currentOrigin.as_ptr(),
                );
                if closestLeader.is_null() || dist < closestDist {
                    closestDist = dist;
                    closestLeader = mPacks.data[i].unwrap();
                }
            }
            i += 1;
        }
    }

    // In Joining Distance?
    //----------------------
    if !closestLeader.is_null() && closestDist < JOIN_PACK_DISTANCE {
        // Am I Already A Leader?
        //------------------------
        if (*(*NPC).client).leader == NPC {
            mPacks.erase_swap(myLeaderNum); // Erase Myself From The Leader List
        }

        // Join The Pack!
        //----------------
        (*(*NPC).client).leader = closestLeader;
    }

    // Do I Have A Leader?
    //---------------------
    if !(*(*NPC).client).leader.is_null() {
        // AM I A Leader?
        //----------------
        if (*(*NPC).client).leader != NPC {
            // If Our Leader Is Dead, Clear Him Out

            if (*(*(*NPC).client).leader).health <= 0
                || (*(*(*NPC).client).leader).inuse == 0
            {
                (*(*NPC).client).leader = core::ptr::null_mut();
            }
            // If My Leader Isn't His Own Leader, Then, Use His Leader
            //---------------------------------------------------------
            else if (*(*(*(*NPC).client).leader).client).leader
                != (*(*NPC).client).leader
            {
                // Eh.  Can this get more confusing?
                (*(*NPC).client).leader =
                    (*(*(*(*NPC).client).leader).client).leader;
            }
            // If Our Leader Is Too Far Away, Clear Him Out
            //------------------------------------------------------
            else if Distance(
                (*(*(*NPC).client).leader).currentOrigin.as_ptr(),
                (*NPC).currentOrigin.as_ptr(),
            ) > LEAVE_PACK_DISTANCE
            {
                (*(*NPC).client).leader = core::ptr::null_mut();
            }
        }
    }

    // If We Couldn't Find A Leader, Then Become One
    //-----------------------------------------------
    else if !mPacks.full() {
        (*(*NPC).client).leader = NPC;
        mPacks.push_back(NPC);
    }
    (*(*NPC).client).leader
}

/*
-------------------------
NPC_BSAnimal_Default
-------------------------
*/
unsafe extern "C" fn NPC_BSAnimal_Default() {
    if NPC.is_null() || (*NPC).client.is_null() {
        return;
    }

    // Update Some Positions
    //-----------------------
    let CurrentLocation: [f32; 3] = (*NPC).currentOrigin;

    // Update The Leader
    //-------------------
    let leader: *mut gentity_t = NPC_AnimalUpdateLeader();

    // Select Closest Threat Location
    //--------------------------------
    let mut ThreatLocation: [f32; 3] = [0.0, 0.0, 0.0];
    let PlayerSpawned: c_int = G_PlayerSpawned();
    if PlayerSpawned != 0 {
        //player is actually in the level now
        ThreatLocation = (*player).currentOrigin;
    }
    let alertEvent: c_int = NPC_CheckAlertEvents(1, 1, -1, 0, 1, 0);
    if alertEvent >= 0 {
        let event: *mut alertEvent_t =
            &mut *level.alertEvents.offset(alertEvent as isize);
        if (*event).owner != NPC
            && Distance((*event).position.as_ptr(), CurrentLocation.as_ptr())
                < (*event).radius
        {
            ThreatLocation = (*event).position;
        }
    }

    //	float	DistToThreat	= CurrentLocation.Dist(ThreatLocation);
    //	float	DistFromHome	= CurrentLocation.Dist(mHome);

    let EvadeThreat: bool =
        (level.time < (*NPCInfo).investigateSoundDebounceTime);
    let CharmedDocile: bool = (level.time < (*NPCInfo).confusionTime);
    let CharmedApproach: bool = (level.time < (*NPCInfo).charmedTime);

    // If Not Already Evading, Test To See If We Should "Know" About The Threat
    //--------------------------------------------------------------------------
    /*	if (false && !EvadeThreat && PlayerSpawned && (DistToThreat<FRIGHTEN_DISTANCE))
    {
        CVec3	LookAim(NPC->currentAngles);
        LookAim.AngToVec();
        CVec3	MyPos(CurrentLocation);
        MyPos -= ThreatLocation;
        MyPos.SafeNorm();

        float	DirectionSimilarity = MyPos.Dot(LookAim);

        if (fabsf(DirectionSimilarity)<0.8f)
        {
            EvadeThreat = true;
            NPCInfo->investigateSoundDebounceTime = level.time + Q_irand(0, 1000);
            VectorCopy(ThreatLocation.v, NPCInfo->investigateGoal);
        }
    }*/

    STEER_Activate(NPC);
    {
        // Charmed Approach - Walk TOWARD The Threat Location
        //----------------------------------------------------
        if CharmedApproach {
            NAV_GoTo(NPC, (*NPCInfo).investigateGoal[0] as c_int);
        }
        // Charmed Docile - Stay Put
        //---------------------------
        else if CharmedDocile {
            NAV_ClearPath(NPC);
            STEER_Stop(NPC);
        }
        // Run Away From This Threat
        //---------------------------
        else if EvadeThreat {
            NAV_ClearPath(NPC);
            STEER_Flee(NPC, (*NPCInfo).investigateGoal.as_ptr());
        }
        // Normal Behavior
        //-----------------
        else {
            // Follow Our Pack Leader!
            //-------------------------
            if !leader.is_null() && leader != NPC {
                let followDist: f32 = 100.0;
                let curDist: f32 = Distance(
                    (*NPC).currentOrigin.as_ptr(),
                    (*leader).followPos.as_ptr(),
                );

                // Update The Leader's Follow Position
                //-------------------------------------
                STEER_FollowLeader(NPC, leader, followDist);

                let inSeekRange: bool = (curDist < followDist * 10.0);
                let onNbrPoints: bool = (NAV_OnNeighboringPoints(
                    NAV_GetNearestNode(NPC),
                    (*leader).followPosWaypoint,
                ) != 0);
                let leaderStop: bool =
                    ((level.time - (*leader).lastMoveTime) > 500);

                // If Close Enough, Dump Any Existing Path
                //-----------------------------------------
                if inSeekRange || onNbrPoints {
                    NAV_ClearPath(NPC);

                    // If The Leader Isn't Moving, Stop
                    //----------------------------------
                    if leaderStop {
                        STEER_Stop(NPC);
                    }
                    // Otherwise, Try To Get To The Follow Position
                    //----------------------------------------------
                    else {
                        STEER_Seek(
                            NPC,
                            (*leader).followPos.as_ptr(),
                            (followDist / 2.0).abs(), /*slowing distance*/
                            1.0,                       /*weight*/
                            (*leader).resultspeed,
                        );
                    }
                }
                // Otherwise, Get A Path To The Follow Position
                //----------------------------------------------
                else {
                    NAV_GoTo(NPC, (*leader).followPosWaypoint);
                }
                STEER_Separation(NPC, 4.0);
                STEER_AvoidCollisions(NPC, leader);
            }
            // Leader AI - Basically Wander
            //------------------------------
            else {
                // Are We Doing A Path?
                //----------------------
                let mut HasPath: bool = NAV_HasPath(NPC) != 0;
                if HasPath {
                    HasPath = NAV_UpdatePath(NPC) != 0;
                    if HasPath {
                        STEER_Path(NPC); // Follow The Path
                        STEER_AvoidCollisions_single(NPC);
                    }
                }

                if !HasPath {
                    // If Debounce Time Has Expired, Choose A New Sub State
                    //------------------------------------------------------
                    if (*NPCInfo).investigateDebounceTime < level.time {
                        // Clear Out Flags From The Previous Substate
                        //--------------------------------------------
                        (*NPCInfo).aiFlags &= !(0x00000004); // NPCAI_OFF_PATH
                        (*NPCInfo).aiFlags &= !(0x00000002); // NPCAI_WALKING

                        // Pick Another Spot
                        //-------------------
                        let NEXTSUBSTATE: c_int = Q_irand(0, 10);

                        let RandomPathNode: bool = (NEXTSUBSTATE < 8); //(NEXTSUBSTATE<9);
                        let PathlessWander: bool = (NEXTSUBSTATE < 9); //false;

                        // Random Path Node
                        //------------------
                        if RandomPathNode {
                            // Sometimes, Walk
                            //-----------------
                            if Q_irand(0, 1) == 0 {
                                (*NPCInfo).aiFlags |= 0x00000002; // NPCAI_WALKING
                            }

                            (*NPCInfo).investigateDebounceTime =
                                level.time + Q_irand(3000, 10000);
                            NAV_FindPath(
                                NPC,
                                NAV_ChooseRandomNeighbor(NAV_GetNearestNode(
                                    NPC,
                                )),
                            ); //,
                               // mHome.v, WANDER_RANGE));
                        }
                        // Pathless Wandering
                        //--------------------
                        else if PathlessWander {
                            // Sometimes, Walk
                            //-----------------
                            if Q_irand(0, 1) == 0 {
                                (*NPCInfo).aiFlags |= 0x00000002; // NPCAI_WALKING
                            }

                            (*NPCInfo).investigateDebounceTime =
                                level.time + Q_irand(3000, 10000);
                            (*NPCInfo).aiFlags |= 0x00000004; // NPCAI_OFF_PATH
                        }
                        // Just Stand Here
                        //-----------------
                        else {
                            (*NPCInfo).investigateDebounceTime =
                                level.time + Q_irand(2000, 6000);
                            //NPC_SetAnim(NPC, SETANIM_BOTH, ((Q_irand(0, 1)==0)?(BOTH_GUARD_LOOKAROUND1):(BOTH_GUARD_IDLE1)), SETANIM_FLAG_NORMAL);
                        }
                    }
                    // Ok, So We Don't Have A Path, And Debounce Time Is Still Active, So We Are Either Wandering Or Looking Around
                    //--------------------------------------------------------------------------------------------------------------
                    else {
                        //	if (DistFromHome>(WANDER_RANGE))
                        //	{
                        //		STEER::Seek(NPC, mHome);
                        //	}
                        //	else
                        {
                            if ((*NPCInfo).aiFlags & 0x00000004) != 0 {
                                // NPCAI_OFF_PATH
                                STEER_Wander(NPC);
                                STEER_AvoidCollisions_single(NPC);
                            } else {
                                STEER_Stop(NPC);
                            }
                        }
                    }
                }
            }
        }
    }
    STEER_DeActivate(NPC, &mut ucmd);

    NPC_UpdateAngles(1, 1);
}
