////////////////////////////////////////////////////////////////////////////////
// RAVEN SOFTWARE - STAR WARS: JK II
//  (c) 2002 Activision
//
// Troopers
//
// TODO
// ----
//
//
//
//
// leave this line at the top of all AI_xxxx.cpp files for PCH reasons...
////////////////////////////////////////////////////////////////////////////////

use core::ffi::{c_int, c_void};

// ===============================================================================
// LOCAL STUBS - RATL Container types
// These are stubbed locally to preserve structural coherence with the original
// ratl::vector_vs, ratl::array_vs, and ratl::handle_pool_vs containers.
// ===============================================================================

/// Stub for ratl::vector_vs<T, MAX>
/// Mimics a vector with a compile-time capacity
struct VectorVs<T, const MAX: usize> {
    data: [Option<T>; MAX],
    len: usize,
}

/// Stub for ratl::array_vs<T, MAX>
/// Mimics a dynamic array with a compile-time capacity
struct ArrayVs<T, const MAX: usize> {
    data: [Option<T>; MAX],
    len: usize,
}

/// Stub for ratl::handle_pool_vs<T, MAX>
/// Mimics a handle pool for managing allocated objects
struct HandlePoolVs<T, const MAX: usize> {
    data: [Option<T>; MAX],
    len: usize,
}

// ===============================================================================
// External declarations and global hooks
// ===============================================================================

extern "C" {
    fn G_AddVoiceEvent(actor: *mut gentity_t, event: c_int, speakDebounceTime: c_int);
    fn TIMER_Done(actor: *mut gentity_t, timerName: *const c_char) -> bool;
    fn TIMER_Set(actor: *mut gentity_t, timerName: *const c_char, duration: c_int);
    fn Q_irand(min: c_int, max: c_int) -> c_int;
    fn random() -> f32;
    fn NPC_ValidEnemy(actor: *mut gentity_t) -> bool;
    fn NPC_ClearLOS(pos: *const f32) -> bool;
    fn CalcEntitySpot(actor: *mut gentity_t, spot: c_int, pos: *mut f32);
    fn SaveNPCGlobals();
    fn SetNPCGlobals(actor: *mut gentity_t);
    fn RestoreNPCGlobals();
    fn NPC_SetAnim(
        actor: *mut gentity_t,
        setAnimPart: c_int,
        anim: c_int,
        setAnimFlags: c_int,
    );
    fn NPC_FreeCombatPoint(cpNum: c_int, removeSelf: bool);
    fn NPC_SetCombatPoint(cpNum: c_int);
    fn NPC_FindCombatPointRetry(
        origin: *const f32,
        enemy: *const f32,
        lastPos: *const f32,
        cpFlags: *mut c_int,
        avoidDist: f32,
        targetNum: c_int,
    ) -> c_int;
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn DistanceSquared(a: *const f32, b: *const f32) -> f32;
    fn Distance(a: *const f32, b: *const f32) -> f32;
    fn G_Throw(actor: *mut gentity_t, direction: *const f32, mag: f32);
    fn G_SetEnemy(actor: *mut gentity_t, enemy: *mut gentity_t);
    fn NPC_UpdateFiringAngles(aim: bool, useDesired: bool);
    fn NPC_UpdateAngles(aim: bool, useDesired: bool);
    fn WeaponThink(force: bool);
    fn NPC_BSST_Default();
    fn NPC_BSST_Investigate();
    fn NPC_BSST_Sleep();

    // Global data
    static mut level: LevelState;
    static mut g_entities: [gentity_t; 4096];
    static mut g_speed: *mut cvar_s;
    static mut NPC: *mut gentity_t;
    static mut NPCInfo: *mut gNPCstats_t;
    static mut ucmd: usercmd_t;
}

// Stub types referenced by the code
#[repr(C)]
pub struct gentity_t {
    // Stub: full structure not defined here
    // This is a game entity type from the engine
}

#[repr(C)]
pub struct npcclient_t {
    // Stub
}

#[repr(C)]
pub struct gNPCstats_t {
    // Stub
}

#[repr(C)]
pub struct cvar_s {
    // Stub
}

#[repr(C)]
pub struct usercmd_t {
    // Stub
}

#[repr(C)]
pub struct LevelState {
    // Stub
}

#[repr(C)]
pub struct CVec3 {
    v: [f32; 3],
}

// ===============================================================================
// Defines
// ===============================================================================

const MAX_TROOPS: usize = 100;
const MAX_ENTS_PER_TROOP: usize = 7;
const MAX_TROOP_JOIN_DIST2: i32 = 1000000; // 1000 units
const MAX_TROOP_MERGE_DIST2: i32 = 250000; // 500 units
const TARGET_POS_VISITED: i32 = 10000; // 100 units

// ===============================================================================
// Speech Events Enum
// ===============================================================================

enum SpeechType {
    SpeechChase = 0,
    SpeechConfused,
    SpeechCover,
    SpeechDetected,
    SpeechGiveup,
    SpeechLook,
    SpeechLost,
    SpeechOutflank,
    SpeechEscaping,
    SpeechSight,
    SpeechSound,
    SpeechSuspicious,
    SpeechYell,
    SpeechPushed,
}

// Forward declaration
fn NPC_IsTrooper(actor: *mut gentity_t) -> bool;

////////////////////////////////////////////////////////////////////////////////
// HT_Speech - Handle trooper speech with debouncing
////////////////////////////////////////////////////////////////////////////////
fn HT_Speech(actor: *mut gentity_t, speechType: SpeechType, failChance: f32) {
    unsafe {
        if random() < failChance {
            return;
        }

        if failChance >= 0.0 {
            // a negative failChance makes it always talk
            if !(*actor).NPC.is_null() && !(*(*actor).NPC).group.is_null() {
                // group AI speech debounce timer
                if (*(*(*actor).NPC).group).speechDebounceTime > level.time {
                    return;
                }
                /*
                else if ( !self->NPC->group->enemy )
                {
                    if ( groupSpeechDebounceTime[self->client->playerTeam] > level.time )
                    {
                        return;
                    }
                }
                */
            } else if !TIMER_Done(actor, b"chatter\0".as_ptr() as *const c_char) {
                // personal timer
                return;
            }
        }

        TIMER_Set(
            actor,
            b"chatter\0".as_ptr() as *const c_char,
            Q_irand(2000, 4000),
        );

        if !(*actor).NPC.is_null() {
            if (*(*actor).NPC).blockedSpeechDebounceTime > level.time {
                return;
            }
        }

        match speechType {
            SpeechType::SpeechChase => {
                G_AddVoiceEvent(actor, Q_irand(0, 2), 2000); // Placeholder for EV_CHASE1, EV_CHASE3
            }
            SpeechType::SpeechConfused => {
                G_AddVoiceEvent(actor, Q_irand(0, 2), 2000); // Placeholder for EV_CONFUSE1, EV_CONFUSE3
            }
            SpeechType::SpeechCover => {
                G_AddVoiceEvent(actor, Q_irand(0, 4), 2000); // Placeholder for EV_COVER1, EV_COVER5
            }
            SpeechType::SpeechDetected => {
                G_AddVoiceEvent(actor, Q_irand(0, 4), 2000); // Placeholder for EV_DETECTED1, EV_DETECTED5
            }
            SpeechType::SpeechGiveup => {
                G_AddVoiceEvent(actor, Q_irand(0, 3), 2000); // Placeholder for EV_GIVEUP1, EV_GIVEUP4
            }
            SpeechType::SpeechLook => {
                G_AddVoiceEvent(actor, Q_irand(0, 1), 2000); // Placeholder for EV_LOOK1, EV_LOOK2
            }
            SpeechType::SpeechLost => {
                G_AddVoiceEvent(actor, 0, 2000); // Placeholder for EV_LOST1
            }
            SpeechType::SpeechOutflank => {
                G_AddVoiceEvent(actor, Q_irand(0, 1), 2000); // Placeholder for EV_OUTFLANK1, EV_OUTFLANK2
            }
            SpeechType::SpeechEscaping => {
                G_AddVoiceEvent(actor, Q_irand(0, 2), 2000); // Placeholder for EV_ESCAPING1, EV_ESCAPING3
            }
            SpeechType::SpeechSight => {
                G_AddVoiceEvent(actor, Q_irand(0, 2), 2000); // Placeholder for EV_SIGHT1, EV_SIGHT3
            }
            SpeechType::SpeechSound => {
                G_AddVoiceEvent(actor, Q_irand(0, 2), 2000); // Placeholder for EV_SOUND1, EV_SOUND3
            }
            SpeechType::SpeechSuspicious => {
                G_AddVoiceEvent(actor, Q_irand(0, 4), 2000); // Placeholder for EV_SUSPICIOUS1, EV_SUSPICIOUS5
            }
            SpeechType::SpeechYell => {
                G_AddVoiceEvent(actor, Q_irand(0, 2), 2000); // Placeholder for EV_ANGER1, EV_ANGER3
            }
            SpeechType::SpeechPushed => {
                G_AddVoiceEvent(actor, Q_irand(0, 2), 2000); // Placeholder for EV_PUSHED1, EV_PUSHED3
            }
        }

        if !(*actor).NPC.is_null() {
            (*(*actor).NPC).blockedSpeechDebounceTime = level.time + 2000;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// The Troop
//
// Troopers primarly derive their behavior from cooperation as a collective group of
// individuals.  They join Troops, each of which has a leader responsible for direcing
// the movement of the rest of the group.
//
////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
pub struct SActorOrder {
    mPosition: CVec3,
    mCombatPoint: c_int,
    mKneelAndShoot: bool,
}

#[repr(C)]
enum ETroopState {
    TsNone = 0, // No troop wide activity active

    TsAdvance = 1,           // CHOOSE A NEW ADVANCE TACTIC
    TsAdvanceRegroup = 2,    // All ents move into squad position
    TsAdvanceSearch = 3,     // Slow advance, looking left to right, in formation
    TsAdvanceCover = 4,      // One at a time moves forward, goes off path, provides cover
    TsAdvanceFormation = 5,  // In formation jog to goal location

    TsAttack = 6,        // CHOOSE A NEW ATTACK TACTIC
    TsAttackLine = 7,    // Form 2 lines, front kneel, back stand
    TsAttackFlank = 8,   // Same As Line, except scouting group attemts to get around other side of target
    TsAttackSurround = 9, // Get on all sides of target
    TsAttackCover = 10,   //

    TsMax = 11,
}

pub struct CTroop {
    // Various Troop Wide Data
    mTroopHandle: c_int,
    mTroopTeam: c_int,
    mTroopReform: bool,

    mFormSpacingFwd: f32,
    mFormSpacingRight: f32,
    mSurroundFanAngle: f32,

    // The Actors
    // Actors are all the troopers who belong to the group, their positions in this
    // vector affect their positions in the troop, whith the first actor as the leader
    mActors: Vec<*mut gentity_t>,

    // Enemy
    // The troop has a collective enemy that it knows about, which is updated by all
    // the members of the group;
    mTarget: *mut gentity_t,
    mTargetVisable: bool,
    mTargetVisableStartTime: c_int,
    mTargetVisableStopTime: c_int,
    mTargetVisablePosition: CVec3,
    mTargetIndex: c_int,
    mTargetLastKnownTime: c_int,
    mTargetLastKnownPosition: CVec3,
    mTargetLastKnownPositionVisited: bool,

    // Troop State
    // The troop as a whole can be acting under a number of different "behavior states"
    mState: ETroopState,

    mFormHead: CVec3,
    mFormFwd: CVec3,
    mFormRight: CVec3,

    mOrders: Vec<SActorOrder>,
}

impl CTroop {
    // Initialize - Clear out all data, all actors, reset all variables
    fn Initialize(&mut self, TroopHandle: c_int) {
        self.mActors.clear();
        self.mTarget = core::ptr::null_mut();
        self.mState = ETroopState::TsNone;
        self.mTroopHandle = TroopHandle;
        self.mTroopTeam = 0;
        self.mTroopReform = false;
    }

    // DistanceSq - Quick Operation to see how far an ent is from the rest of the troop
    fn DistanceSq(&self, ent: *mut gentity_t) -> f32 {
        if !self.mActors.is_empty() {
            unsafe {
                return DistanceSquared(
                    (*ent).currentOrigin.as_ptr(),
                    (*self.mActors[0]).currentOrigin.as_ptr(),
                );
            }
        }
        0.0
    }

    // MakeActorLeader - Move A Given Index To A Leader Position
    fn MakeActorLeader(&mut self, index: usize) {
        if index != 0 {
            unsafe {
                (*(*self.mActors[0]).client).leader = core::ptr::null_mut();
                self.mActors.swap(0, index);
            }
        }
        unsafe {
            (*(*self.mActors[0]).client).leader = self.mActors[0];
            if !self.mActors[0].is_null() {
                if (*(*self.mActors[0]).NPC).NPC_class == 0 {
                    // CLASS_HAZARD_TROOPER
                    self.mFormSpacingFwd = 75.0;
                    self.mFormSpacingRight = 50.0;
                } else {
                    self.mFormSpacingFwd = 75.0;
                    self.mFormSpacingRight = 20.0;
                }
            }
        }
    }

    // AddActor - Adds a new actor to the troop & automatically promote to leader
    fn AddActor(&mut self, actor: *mut gentity_t) {
        unsafe {
            assert!(!(*actor).NPC.is_null() && (*(*actor).NPC).troop == 0);
            assert!(self.mActors.len() < MAX_ENTS_PER_TROOP);
            (*(*actor).NPC).troop = self.mTroopHandle;
            self.mActors.push(actor);
            self.mTroopReform = true;
            if (self.mActors.len() == 1)
                || ((*(*actor).NPC).rank > (*(*self.mActors[0]).NPC).rank)
            {
                self.MakeActorLeader(self.mActors.len() - 1);
            }
            if self.mTroopTeam == 0 {
                self.mTroopTeam = (*(*actor).client).playerTeam;
            }
        }
    }

    // RemoveActor - Removes an actor from the troop & automatically promote leader
    fn RemoveActor(&mut self, actor: *mut gentity_t) {
        unsafe {
            assert!((*actor).NPC.is_null() == false);
            assert!((*(*actor).NPC).troop == self.mTroopHandle);
            let mut bestNewLeader: i32 = -1;
            let mut numEnts = self.mActors.len();
            let mut found = false;
            self.mTroopReform = true;

            // Find The Actor
            for i in 0..numEnts {
                if self.mActors[i] == actor {
                    found = true;
                    self.mActors.remove(i);
                    numEnts -= 1;
                    if i == 0 && !self.mActors.is_empty() {
                        bestNewLeader = 0;
                    }
                }

                if bestNewLeader >= 0
                    && (i < self.mActors.len())
                    && ((*(*self.mActors[i as usize]).NPC).rank
                        > (*(*self.mActors[bestNewLeader as usize]).NPC).rank)
                {
                    bestNewLeader = i as i32;
                }
            }
            if !self.mActors.is_empty() && bestNewLeader >= 0 {
                self.MakeActorLeader(bestNewLeader as usize);
            }

            assert!(found);
            (*actor).NPC.as_mut().unwrap().troop = 0;
        }
    }

    // RegisterTarget - Records That the target is seen, when and where
    fn RegisterTarget(&mut self, target: *mut gentity_t, index: c_int, visable: bool) {
        unsafe {
            if self.mTarget.is_null() {
                HT_Speech(self.mActors[0], SpeechType::SpeechDetected, 0.0);
            } else if (level.time - self.mTargetLastKnownTime) > 8000 {
                HT_Speech(self.mActors[0], SpeechType::SpeechSight, 0.0);
            }

            if visable {
                self.mTargetVisableStopTime = level.time;
                if !self.mTargetVisable {
                    self.mTargetVisableStartTime = level.time;
                }

                CalcEntitySpot(target, 1, self.mTargetVisablePosition.v.as_mut_ptr()); // SPOT_HEAD
                self.mTargetVisablePosition.v[2] -= 10.0;
            }

            self.mTarget = target;
            self.mTargetVisable = visable;
            self.mTargetIndex = index;
            self.mTargetLastKnownTime = level.time;
            self.mTargetLastKnownPosition.v = (*target).currentOrigin;
            self.mTargetLastKnownPositionVisited = false;
        }
    }

    // TargetLastKnownPositionVisited - Records That the target is seen, when and where
    fn TargetLastKnownPositionVisited(&mut self) -> bool {
        unsafe {
            if !self.mTargetLastKnownPositionVisited {
                let dist = DistanceSquared(
                    self.mTargetLastKnownPosition.v.as_ptr(),
                    (*self.mActors[0]).currentOrigin.as_ptr(),
                );
                self.mTargetLastKnownPositionVisited = (dist < TARGET_POS_VISITED as f32);
            }
            self.mTargetLastKnownPositionVisited
        }
    }

    fn ClampScale(&self, val: f32) -> f32 {
        let mut v = val;
        if v > 1.0 {
            v = 1.0;
        }
        if v < 0.0 {
            v = 0.0;
        }
        v
    }

    // Target Visibility
    // Compute all factors that can add visibility to a target
    fn TargetVisibility(&self, target: *mut gentity_t) -> f32 {
        unsafe {
            let mut Scale = 0.8;
            if !(*target).client.is_null() {
                // && target->client->ps.weapon==WP_SABER && target->client->ps.SaberActive()
                // Stub: using placeholder conditions
                // if ((*(*target).client).ps.weapon == 0 && ...) {
                //     Scale += 0.1;
                // }
            }
            self.ClampScale(Scale as f32)
        }
    }

    // Compute noise level of a target
    fn TargetNoiseLevel(&self, target: *mut gentity_t) -> f32 {
        unsafe {
            let mut Scale = 0.1;
            // Scale += target->resultspeed / (float)g_speed->integer;
            // if (target->client && target->client->ps.weapon==WP_SABER && target->client->ps.SaberActive())
            // {
            //     Scale += 0.2f;
            // }
            self.ClampScale(Scale as f32)
        }
    }

    // Scan For Enemies
    fn ScanForTarget(&mut self, scannerIndex: usize) {
        unsafe {
            let mut targetIndex = 0;
            let mut targetStop = 4096; // ENTITYNUM_WORLD

            // If Existing Target, Only Check It
            if !self.mTarget.is_null() {
                targetIndex = self.mTargetIndex as usize;
                targetStop = (self.mTargetIndex as usize) + 1;
            }

            SaveNPCGlobals();
            SetNPCGlobals(self.mActors[scannerIndex]);

            for idx in targetIndex..targetStop {
                let target = &mut g_entities[idx];
                if !NPC_ValidEnemy(target as *mut gentity_t) {
                    continue;
                }

                // TODO: Implement full target scanning logic
                // This is a stub implementation
            }
            RestoreNPCGlobals();
        }
    }

    // TroopInFormation - A quick check to see if the troop is currently in formation
    fn TroopInFormation(&self) -> bool {
        let maxActorRangeSq = ((self.mActors.len() / 2) + 2) as f32 * self.mFormSpacingFwd;
        let maxActorRangeSq = maxActorRangeSq * maxActorRangeSq;
        for actorIndex in 1..self.mActors.len() {
            if self.DistanceSq(self.mActors[actorIndex]) > maxActorRangeSq {
                return false;
            }
        }
        true
    }

    // LeaderIssueAndUpdateOrders - Tell Everyone Where To Go
    fn LeaderIssueAndUpdateOrders(&mut self, NextState: ETroopState) {
        unsafe {
            let actorCount = self.mActors.len();

            // Always Put Guys Closest To The Order Locations In Those Locations
            for orderIndex in 1..actorCount {
                // Don't re-assign points combat point related orders
                if self.mOrders[orderIndex].mCombatPoint == -1 {
                    let mut closestActorIndex = orderIndex;
                    let mut closestActorDistance = DistanceSquared(
                        self.mOrders[orderIndex].mPosition.v.as_ptr(),
                        (*self.mActors[orderIndex]).currentOrigin.as_ptr(),
                    );
                    for actorIndex in (orderIndex + 1)..actorCount {
                        let currentDistance = DistanceSquared(
                            self.mOrders[orderIndex].mPosition.v.as_ptr(),
                            (*self.mActors[actorIndex]).currentOrigin.as_ptr(),
                        );
                        if currentDistance < closestActorDistance {
                            closestActorDistance = currentDistance;
                            closestActorIndex = actorIndex;
                        }
                    }
                    if orderIndex != closestActorIndex {
                        self.mActors.swap(orderIndex, closestActorIndex);
                    }
                }
            }

            // Now Copy The Orders Out To The Actors
            for actorIndex in 1..actorCount {
                VectorCopy(
                    self.mOrders[actorIndex].mPosition.v.as_ptr(),
                    (*self.mActors[actorIndex]).pos1.as_mut_ptr(),
                );
            }

            // PHASE I - VOICE COMMANDS & ANIMATIONS
            let leader = self.mActors[0];

            if (NextState as i32) != (self.mState as i32) {
                if !self.mActors.is_empty() {
                    match NextState {
                        ETroopState::TsAdvanceRegroup => {
                            // break;
                        }
                        ETroopState::TsAdvanceSearch => {
                            HT_Speech(leader, SpeechType::SpeechLook, 0.0);
                        }
                        ETroopState::TsAdvanceCover => {
                            HT_Speech(leader, SpeechType::SpeechCover, 0.0);
                            NPC_SetAnim(leader, 0, 0, 0); // SETANIM_TORSO, TORSO_HANDSIGNAL4
                        }
                        ETroopState::TsAdvanceFormation => {
                            HT_Speech(leader, SpeechType::SpeechEscaping, 0.0);
                        }
                        ETroopState::TsAttackLine => {
                            HT_Speech(leader, SpeechType::SpeechChase, 0.0);
                            NPC_SetAnim(leader, 0, 0, 0); // SETANIM_TORSO, TORSO_HANDSIGNAL1
                        }
                        ETroopState::TsAttackFlank => {
                            HT_Speech(leader, SpeechType::SpeechOutflank, 0.0);
                            NPC_SetAnim(leader, 0, 0, 0); // SETANIM_TORSO, TORSO_HANDSIGNAL3
                        }
                        ETroopState::TsAttackSurround => {
                            HT_Speech(leader, SpeechType::SpeechGiveup, 0.0);
                            NPC_SetAnim(leader, 0, 0, 0); // SETANIM_TORSO, TORSO_HANDSIGNAL2
                        }
                        ETroopState::TsAttackCover => {
                            HT_Speech(leader, SpeechType::SpeechCover, 0.0);
                        }
                        _ => {}
                    }
                }
            } else if (NextState as i32) > (ETroopState::TsAttack as i32) && !self.mTroopReform {
                return;
            }

            // PHASE II - COMPUTE THE NEW FORMATION HEAD, FORWARD, AND RIGHT VECTORS
            // TODO: Implement formation calculations with NAV functions
            self.mState = NextState;
            self.mTroopReform = false;
        }
    }

    // SufficientCoverNearby - Look at nearby combat points, see if there is enough
    fn SufficientCoverNearby(&self) -> bool {
        // TODO: Evaluate Available Combat Points
        false
    }

    // Update - This is the primary "think" function from the troop
    fn Update(&mut self) {
        if self.mActors.is_empty() {
            return;
        }
        self.ScanForTarget(0);
        if !self.mTarget.is_null() {
            unsafe {
                let mut NextState = self.mState;
                let TimeSinceLastSeen = (level.time - self.mTargetVisableStopTime);
                let Attack = (TimeSinceLastSeen < 2000);

                if Attack {
                    // If Not Currently Attacking, Or We Want To Pick A New Attack Tactic
                    if (self.mState as i32) < (ETroopState::TsAttack as i32) {
                        if self.TroopInFormation() {
                            NextState = if self.mActors.len() > 4 {
                                ETroopState::TsAttackFlank
                            } else {
                                ETroopState::TsAttackLine
                            };
                        } else {
                            NextState = if self.SufficientCoverNearby() {
                                ETroopState::TsAttackCover
                            } else {
                                ETroopState::TsAttackSurround
                            };
                        }
                    }
                } else {
                    if !self.TroopInFormation() {
                        NextState = ETroopState::TsAdvanceRegroup;
                    } else {
                        if self.TargetLastKnownPositionVisited() {
                            NextState = ETroopState::TsAdvanceSearch;
                        } else {
                            NextState = if TimeSinceLastSeen < 10000 {
                                ETroopState::TsAdvanceCover
                            } else {
                                ETroopState::TsAdvanceFormation
                            };
                        }
                    }
                }
                self.LeaderIssueAndUpdateOrders(NextState);
            }
        }
    }

    // MergeInto - Merges all actors into anther troop
    fn MergeInto(&mut self, Other: &mut CTroop) {
        unsafe {
            let numEnts = self.mActors.len();
            for i in 0..numEnts {
                (*self.mActors[i]).client.as_mut().unwrap().leader = core::ptr::null_mut();
                (*self.mActors[i]).NPC.as_mut().unwrap().troop = 0;
                Other.AddActor(self.mActors[i]);
            }
            self.mActors.clear();

            if Other.mTarget.is_null() && !self.mTarget.is_null() {
                Other.mTarget = self.mTarget;
                Other.mTargetIndex = self.mTargetIndex;
                Other.mTargetLastKnownPosition = self.mTargetLastKnownPosition;
                Other.mTargetLastKnownPositionVisited = self.mTargetLastKnownPositionVisited;
                Other.mTargetLastKnownTime = self.mTargetLastKnownTime;
                Other.mTargetVisableStartTime = self.mTargetVisableStartTime;
                Other.mTargetVisableStopTime = self.mTargetVisableStopTime;
                Other.mTargetVisable = self.mTargetVisable;
                Other.mTargetVisablePosition = self.mTargetVisablePosition;
                Other.LeaderIssueAndUpdateOrders(self.mState);
            }
        }
    }

    // TrackingTarget - Return the target being tracked
    fn TrackingTarget(&self) -> *mut gentity_t {
        self.mTarget
    }

    // TroopLeader - Return the troop leader
    fn TroopLeader(&self) -> *mut gentity_t {
        if self.mActors.is_empty() {
            core::ptr::null_mut()
        } else {
            self.mActors[0]
        }
    }

    // TimeSinceSeenTarget - Return time since target was last visible
    fn TimeSinceSeenTarget(&self) -> c_int {
        unsafe { level.time - self.mTargetVisableStopTime }
    }

    // TargetVisablePosition - Return the visible position of the target
    fn TargetVisablePosition(&self) -> CVec3 {
        self.mTargetVisablePosition
    }

    // FormSpacingFwd - Return forward formation spacing
    fn FormSpacingFwd(&self) -> f32 {
        self.mFormSpacingFwd
    }

    // TooCloseToTroopMember - Check if actor is too close to any troop member
    fn TooCloseToTroopMember(&self, actor: *mut gentity_t) -> *mut gentity_t {
        unsafe {
            for i in 0..self.mActors.len() {
                // Only avoid guys ahead of us in the formation
                if actor == self.mActors[i] {
                    return core::ptr::null_mut();
                }

                if i == 0 {
                    if Distance(
                        (*actor).currentOrigin.as_ptr(),
                        (*self.mActors[i]).currentOrigin.as_ptr(),
                    ) < (self.mFormSpacingFwd * 0.5)
                    {
                        return self.mActors[i];
                    }
                } else {
                    if Distance(
                        (*actor).currentOrigin.as_ptr(),
                        (*self.mActors[i]).currentOrigin.as_ptr(),
                    ) < (self.mFormSpacingFwd * 0.5)
                    {
                        return self.mActors[i];
                    }
                }
            }
            assert!("Somehow this actor is not actually in the troop..." == "");
            core::ptr::null_mut()
        }
    }

    // Public accessor methods
    fn Empty(&self) -> bool {
        self.mActors.is_empty()
    }

    fn Team(&self) -> c_int {
        self.mTroopTeam
    }

    fn Handle(&self) -> c_int {
        self.mTroopHandle
    }
}

// Global troop pool
static mut mTroops: Vec<CTroop> = Vec::new();

////////////////////////////////////////////////////////////////////////////////
// Erase All Data, Set To Default Vals Before Entities Spawn
////////////////////////////////////////////////////////////////////////////////
pub fn Troop_Reset() {
    unsafe {
        mTroops.clear();
    }
}

////////////////////////////////////////////////////////////////////////////////
// Entities Have Just Spawned, Initialize
////////////////////////////////////////////////////////////////////////////////
pub fn Troop_Initialize() {
    // TODO: Initialize all troops
}

////////////////////////////////////////////////////////////////////////////////
// Global Update Of All Troops
////////////////////////////////////////////////////////////////////////////////
pub fn Troop_Update() {
    unsafe {
        for i in 0..mTroops.len() {
            mTroops[i].Update();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Erase All Data, Set To Default Vals Before Entities Spawn
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_UpdateTroop(actor: *mut gentity_t) {
    unsafe {
        // Try To Join A Troop
        if (*actor).NPC.is_null() || (*(*actor).NPC).troop == 0 {
            let mut curDist = 0.0;
            let mut closestDist = 0.0;
            let mut closestTroopIdx = None;
            // TODO: Implement full troop joining logic
        } else if !(*actor).client.is_null() && (*(*actor).client).leader == actor {
            // If This Is A Leader, Then He Is Responsible For Merging Troops
            // TODO: Implement troop merging logic
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trooper_UpdateSmackAway - Handle trooper melee attack
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_UpdateSmackAway(actor: *mut gentity_t, target: *mut gentity_t) -> bool {
    unsafe {
        if (*actor).client.is_null() {
            return false;
        }
        // if (actor->client->ps.legsAnim==BOTH_MELEE1)
        // {
        //     if (TIMER_Done(actor, "Trooper_SmackAway"))
        //     {
        //         TODO: Implement smack away logic
        //     }
        //     return true;
        // }
        false
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trooper_SmackAway - Start a melee attack
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_SmackAway(actor: *mut gentity_t, target: *mut gentity_t) {
    unsafe {
        assert!(!actor.is_null() && !(*actor).NPC.is_null());
        // if (actor->client->ps.legsAnim!=BOTH_MELEE1)
        // {
        //     NPC_SetAnim(actor, SETANIM_BOTH, BOTH_MELEE1, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD);
        //     TIMER_Set(actor, "Trooper_SmackAway", actor->client->ps.torsoAnimTimer/4.0f);
        // }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trooper_Kneeling - Check if trooper is kneeling
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_Kneeling(actor: *mut gentity_t) -> bool {
    unsafe {
        if actor.is_null() || (*actor).NPC.is_null() || (*actor).client.is_null() {
            return false;
        }
        // return (actor->NPC->aiFlags&NPCAI_KNEEL || actor->client->ps.legsAnim==BOTH_STAND_TO_KNEEL);
        false
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trooper_KneelDown - Make trooper kneel down
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_KneelDown(actor: *mut gentity_t) {
    unsafe {
        assert!(!actor.is_null() && !(*actor).NPC.is_null());
        // if (!Trooper_Kneeling(actor) && level.time>actor->NPC->kneelTime)
        // {
        //     NPC_SetAnim(actor, SETANIM_BOTH, BOTH_STAND_TO_KNEEL, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD);
        //     actor->NPC->aiFlags |=  NPCAI_KNEEL;
        //     actor->NPC->kneelTime = level.time + Q_irand(3000, 6000);
        // }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trooper_StandUp - Make trooper stand up
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_StandUp(actor: *mut gentity_t, always: bool) {
    unsafe {
        assert!(!actor.is_null() && !(*actor).NPC.is_null());
        // if (Trooper_Kneeling(actor) && (always || level.time>actor->NPC->kneelTime))
        // {
        //     actor->NPC->aiFlags &= ~NPCAI_KNEEL;
        //     NPC_SetAnim(actor, SETANIM_BOTH, BOTH_KNEEL_TO_STAND, SETANIM_FLAG_OVERRIDE|SETANIM_FLAG_HOLD);
        //     actor->NPC->kneelTime = level.time + Q_irand(3000, 6000);
        // }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Trooper_CanHitTarget - Check if trooper can hit their target
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_CanHitTarget(
    actor: *mut gentity_t,
    target: *mut gentity_t,
    troop: &CTroop,
    MuzzleToTargetDistance: &mut f32,
    MuzzleToTarget: &mut CVec3,
) -> c_int {
    unsafe {
        // TODO: Implement line of sight check
        -1 // ENTITYNUM_NONE
    }
}

////////////////////////////////////////////////////////////////////////////////
// Run The Per Trooper Update
////////////////////////////////////////////////////////////////////////////////
pub fn Trooper_Think(actor: *mut gentity_t) {
    unsafe {
        if actor.is_null() || (*actor).NPC.is_null() {
            NPC_BSST_Default();
            return;
        }

        // TODO: Implement full trooper thinking logic
    }
}

////////////////////////////////////////////////////////////////////////////////
/*
-------------------------
NPC_BehaviorSet_Trooper
-------------------------
*/
////////////////////////////////////////////////////////////////////////////////
pub fn NPC_BehaviorSet_Trooper(bState: c_int) {
    unsafe {
        Trooper_UpdateTroop(NPC);
        match bState {
            0 | 1 | 2 | 3 | 4 => {
                // BS_STAND_GUARD, BS_PATROL, BS_STAND_AND_SHOOT, BS_HUNT_AND_KILL, BS_DEFAULT
                Trooper_Think(NPC);
            }
            5 => {
                // BS_INVESTIGATE
                NPC_BSST_Investigate();
            }
            6 => {
                // BS_SLEEP
                NPC_BSST_Sleep();
            }
            _ => {
                Trooper_Think(NPC);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// IsTrooper - return true if you want a given actor to use trooper AI
////////////////////////////////////////////////////////////////////////////////
pub fn NPC_IsTrooper(actor: *mut gentity_t) -> bool {
    if actor.is_null() {
        return false;
    }
    unsafe {
        if (*actor).NPC.is_null() {
            return false;
        }
        if (*actor).s.weapon == 0 {
            return false;
        }
        // return !!(actor->NPC->scriptFlags&SCF_NO_GROUPS);
        true
    }
}

////////////////////////////////////////////////////////////////////////////////
// NPC_LeaveTroop - Remove an actor from their troop
////////////////////////////////////////////////////////////////////////////////
pub fn NPC_LeaveTroop(actor: *mut gentity_t) {
    unsafe {
        assert!(!actor.is_null() && !(*actor).NPC.is_null());
        assert!((*(*actor).NPC).troop != 0);
        // TODO: Implement leaving troop logic
    }
}
