//! Bot AI syscall wrappers — `trap_Bot*` goal/move/weapon/character AI traps
//! (`BOTLIB_AI_*` family). 1:1 with `refs/raven-jediacademy/codemp/game/g_syscalls.c`;
//! bodies land in Phase B. Types: be_ai_goal_h::bot_goal_t, be_ai_move_h::{bot_initmove_t,
//! bot_moveresult_t}, be_ai_weap_h::weaponinfo_t.

use core::ffi::c_char;

use crate::codemp::game::be_ai_goal_h::bot_goal_t;
use crate::codemp::game::be_ai_move_h::{bot_initmove_t, bot_moveresult_t};
use crate::codemp::game::be_ai_weap_h::weaponinfo_t;
use crate::codemp::game::q_shared_h::vec3_t;
use crate::ffi::syscalls::pass_float;
use crate::ffi::GameImport::*;

use super::cstr;

/// `trap_BotLoadCharacter` — load the bot character `charfile` at `skill`,
/// returning a character handle.
pub fn BotLoadCharacter(charfile: &str, skill: f32) -> i32 {
    let f = cstr(charfile);
    unsafe { syscall!(BOTLIB_AI_LOAD_CHARACTER, f.as_ptr(), pass_float(skill)) as i32 }
}

/// `trap_BotFreeCharacter` — free a bot character handle.
pub fn BotFreeCharacter(character: i32) {
    unsafe {
        syscall!(BOTLIB_AI_FREE_CHARACTER, character);
    }
}

/// `trap_Characteristic_Float` — read float characteristic `index` of `character`.
pub fn Characteristic_Float(character: i32, index: i32) -> f32 {
    let temp = unsafe { syscall!(BOTLIB_AI_CHARACTERISTIC_FLOAT, character, index) as i32 };
    f32::from_bits(temp as u32)
}

/// `trap_Characteristic_BFloat` — read float characteristic `index` of `character`,
/// clamped to `min`..`max`.
pub fn Characteristic_BFloat(character: i32, index: i32, min: f32, max: f32) -> f32 {
    let temp = unsafe {
        syscall!(
            BOTLIB_AI_CHARACTERISTIC_BFLOAT,
            character,
            index,
            pass_float(min),
            pass_float(max)
        ) as i32
    };
    f32::from_bits(temp as u32)
}

/// `trap_Characteristic_Integer` — read integer characteristic `index` of `character`.
pub fn Characteristic_Integer(character: i32, index: i32) -> i32 {
    unsafe { syscall!(BOTLIB_AI_CHARACTERISTIC_INTEGER, character, index) as i32 }
}

/// `trap_Characteristic_BInteger` — read integer characteristic `index` of `character`,
/// clamped to `min`..`max`.
pub fn Characteristic_BInteger(character: i32, index: i32, min: i32, max: i32) -> i32 {
    unsafe { syscall!(BOTLIB_AI_CHARACTERISTIC_BINTEGER, character, index, min, max) as i32 }
}

/// `trap_Characteristic_String` — read string characteristic `index` of `character`
/// into the caller-owned `buf` (whose length supplies `size`).
pub fn Characteristic_String(character: i32, index: i32, buf: &mut [c_char]) {
    unsafe {
        syscall!(
            BOTLIB_AI_CHARACTERISTIC_STRING,
            character,
            index,
            buf.as_mut_ptr(),
            buf.len() as i32
        );
    }
}

/// `trap_BotResetGoalState`.
pub fn BotResetGoalState(goalstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_RESET_GOAL_STATE, goalstate);
    }
}

/// `trap_BotResetAvoidGoals`.
pub fn BotResetAvoidGoals(goalstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_RESET_AVOID_GOALS, goalstate);
    }
}

/// `trap_BotRemoveFromAvoidGoals`.
pub fn BotRemoveFromAvoidGoals(goalstate: i32, number: i32) {
    unsafe {
        syscall!(BOTLIB_AI_REMOVE_FROM_AVOID_GOALS, goalstate, number);
    }
}

/// `trap_BotPushGoal` — push `goal` onto the goal stack. The C out-param is a
/// `void* /* struct bot_goal_s */`; given its faithful [`bot_goal_t`] type.
pub fn BotPushGoal(goalstate: i32, goal: &bot_goal_t) {
    unsafe {
        syscall!(BOTLIB_AI_PUSH_GOAL, goalstate, goal as *const bot_goal_t);
    }
}

/// `trap_BotPopGoal`.
pub fn BotPopGoal(goalstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_POP_GOAL, goalstate);
    }
}

/// `trap_BotEmptyGoalStack`.
pub fn BotEmptyGoalStack(goalstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_EMPTY_GOAL_STACK, goalstate);
    }
}

/// `trap_BotDumpAvoidGoals`.
pub fn BotDumpAvoidGoals(goalstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_DUMP_AVOID_GOALS, goalstate);
    }
}

/// `trap_BotDumpGoalStack`.
pub fn BotDumpGoalStack(goalstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_DUMP_GOAL_STACK, goalstate);
    }
}

/// `trap_BotGoalName` — write goal `number`'s name into `name` (whose length supplies `size`).
pub fn BotGoalName(number: i32, name: &mut [c_char]) {
    unsafe {
        syscall!(BOTLIB_AI_GOAL_NAME, number, name.as_mut_ptr(), name.len() as i32);
    }
}

/// `trap_BotGetTopGoal` — read the top goal into `goal` (engine fills it).
pub fn BotGetTopGoal(goalstate: i32, goal: &mut bot_goal_t) -> i32 {
    unsafe { syscall!(BOTLIB_AI_GET_TOP_GOAL, goalstate, goal as *mut bot_goal_t) as i32 }
}

/// `trap_BotGetSecondGoal` — read the second goal into `goal` (engine fills it).
pub fn BotGetSecondGoal(goalstate: i32, goal: &mut bot_goal_t) -> i32 {
    unsafe { syscall!(BOTLIB_AI_GET_SECOND_GOAL, goalstate, goal as *mut bot_goal_t) as i32 }
}

/// `trap_BotChooseLTGItem`.
pub fn BotChooseLTGItem(
    goalstate: i32,
    origin: &vec3_t,
    inventory: &mut [i32],
    travelflags: i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_CHOOSE_LTG_ITEM,
            goalstate,
            origin.as_ptr(),
            inventory.as_mut_ptr(),
            travelflags
        ) as i32
    }
}

/// `trap_BotChooseNBGItem`. `ltg` is the long-term goal (a `void* /* struct
/// bot_goal_s */`); given its faithful [`bot_goal_t`] type.
pub fn BotChooseNBGItem(
    goalstate: i32,
    origin: &vec3_t,
    inventory: &mut [i32],
    travelflags: i32,
    ltg: &bot_goal_t,
    maxtime: f32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_CHOOSE_NBG_ITEM,
            goalstate,
            origin.as_ptr(),
            inventory.as_mut_ptr(),
            travelflags,
            ltg as *const bot_goal_t,
            pass_float(maxtime)
        ) as i32
    }
}

/// `trap_BotTouchingGoal`.
pub fn BotTouchingGoal(origin: &vec3_t, goal: &bot_goal_t) -> i32 {
    unsafe { syscall!(BOTLIB_AI_TOUCHING_GOAL, origin.as_ptr(), goal as *const bot_goal_t) as i32 }
}

/// `trap_BotItemGoalInVisButNotVisible`.
pub fn BotItemGoalInVisButNotVisible(
    viewer: i32,
    eye: &vec3_t,
    viewangles: &vec3_t,
    goal: &bot_goal_t,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE,
            viewer,
            eye.as_ptr(),
            viewangles.as_ptr(),
            goal as *const bot_goal_t
        ) as i32
    }
}

/// `trap_BotGetLevelItemGoal` — read level item goal `index` (matching `classname`)
/// into `goal` (engine fills it).
pub fn BotGetLevelItemGoal(index: i32, classname: &str, goal: &mut bot_goal_t) -> i32 {
    let c = cstr(classname);
    unsafe {
        syscall!(
            BOTLIB_AI_GET_LEVEL_ITEM_GOAL,
            index,
            c.as_ptr(),
            goal as *mut bot_goal_t
        ) as i32
    }
}

/// `trap_BotGetNextCampSpotGoal` — read camp spot goal `num` into `goal` (engine fills it).
pub fn BotGetNextCampSpotGoal(num: i32, goal: &mut bot_goal_t) -> i32 {
    unsafe {
        syscall!(BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL, num, goal as *mut bot_goal_t) as i32
    }
}

/// `trap_BotGetMapLocationGoal` — read the map-location goal named `name` into
/// `goal` (engine fills it).
pub fn BotGetMapLocationGoal(name: &str, goal: &mut bot_goal_t) -> i32 {
    let c = cstr(name);
    unsafe {
        syscall!(BOTLIB_AI_GET_MAP_LOCATION_GOAL, c.as_ptr(), goal as *mut bot_goal_t) as i32
    }
}

/// `trap_BotAvoidGoalTime`.
pub fn BotAvoidGoalTime(goalstate: i32, number: i32) -> f32 {
    let temp = unsafe { syscall!(BOTLIB_AI_AVOID_GOAL_TIME, goalstate, number) as i32 };
    f32::from_bits(temp as u32)
}

/// `trap_BotSetAvoidGoalTime`.
pub fn BotSetAvoidGoalTime(goalstate: i32, number: i32, avoidtime: f32) {
    unsafe {
        syscall!(
            BOTLIB_AI_SET_AVOID_GOAL_TIME,
            goalstate,
            number,
            pass_float(avoidtime)
        );
    }
}

/// `trap_BotInitLevelItems`.
pub fn BotInitLevelItems() {
    unsafe {
        syscall!(BOTLIB_AI_INIT_LEVEL_ITEMS);
    }
}

/// `trap_BotUpdateEntityItems`.
pub fn BotUpdateEntityItems() {
    unsafe {
        syscall!(BOTLIB_AI_UPDATE_ENTITY_ITEMS);
    }
}

/// `trap_BotLoadItemWeights`.
pub fn BotLoadItemWeights(goalstate: i32, filename: &str) -> i32 {
    let f = cstr(filename);
    unsafe { syscall!(BOTLIB_AI_LOAD_ITEM_WEIGHTS, goalstate, f.as_ptr()) as i32 }
}

/// `trap_BotFreeItemWeights`.
pub fn BotFreeItemWeights(goalstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_FREE_ITEM_WEIGHTS, goalstate);
    }
}

/// `trap_BotInterbreedGoalFuzzyLogic`.
pub fn BotInterbreedGoalFuzzyLogic(parent1: i32, parent2: i32, child: i32) {
    unsafe {
        syscall!(BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC, parent1, parent2, child);
    }
}

/// `trap_BotSaveGoalFuzzyLogic`.
pub fn BotSaveGoalFuzzyLogic(goalstate: i32, filename: &str) {
    let f = cstr(filename);
    unsafe {
        syscall!(BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC, goalstate, f.as_ptr());
    }
}

/// `trap_BotMutateGoalFuzzyLogic`. Note: the C wrapper passes `range` directly,
/// **not** via `PASSFLOAT` — reproduced verbatim (the float rides the ABI as a
/// plain widened value here).
pub fn BotMutateGoalFuzzyLogic(goalstate: i32, range: f32) {
    unsafe {
        syscall!(BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC, goalstate, range);
    }
}

/// `trap_BotAllocGoalState`.
pub fn BotAllocGoalState(state: i32) -> i32 {
    unsafe { syscall!(BOTLIB_AI_ALLOC_GOAL_STATE, state) as i32 }
}

/// `trap_BotFreeGoalState`.
pub fn BotFreeGoalState(handle: i32) {
    unsafe {
        syscall!(BOTLIB_AI_FREE_GOAL_STATE, handle);
    }
}

/// `trap_BotResetMoveState`.
pub fn BotResetMoveState(movestate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_RESET_MOVE_STATE, movestate);
    }
}

/// `trap_BotAddAvoidSpot`.
pub fn BotAddAvoidSpot(movestate: i32, origin: &vec3_t, radius: f32, type_: i32) {
    unsafe {
        syscall!(
            BOTLIB_AI_ADD_AVOID_SPOT,
            movestate,
            origin.as_ptr(),
            pass_float(radius),
            type_
        );
    }
}

/// `trap_BotMoveToGoal` — compute movement toward `goal`, writing the result into
/// `result` (engine fills it). Both struct pointers are `void*` in C; given their
/// faithful [`bot_moveresult_t`] / [`bot_goal_t`] types.
pub fn BotMoveToGoal(
    result: &mut bot_moveresult_t,
    movestate: i32,
    goal: &bot_goal_t,
    travelflags: i32,
) {
    unsafe {
        syscall!(
            BOTLIB_AI_MOVE_TO_GOAL,
            result as *mut bot_moveresult_t,
            movestate,
            goal as *const bot_goal_t,
            travelflags
        );
    }
}

/// `trap_BotMoveInDirection`.
pub fn BotMoveInDirection(movestate: i32, dir: &vec3_t, speed: f32, type_: i32) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_MOVE_IN_DIRECTION,
            movestate,
            dir.as_ptr(),
            pass_float(speed),
            type_
        ) as i32
    }
}

/// `trap_BotResetAvoidReach`.
pub fn BotResetAvoidReach(movestate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_RESET_AVOID_REACH, movestate);
    }
}

/// `trap_BotResetLastAvoidReach`.
pub fn BotResetLastAvoidReach(movestate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_RESET_LAST_AVOID_REACH, movestate);
    }
}

/// `trap_BotReachabilityArea`.
pub fn BotReachabilityArea(origin: &vec3_t, testground: i32) -> i32 {
    unsafe { syscall!(BOTLIB_AI_REACHABILITY_AREA, origin.as_ptr(), testground) as i32 }
}

/// `trap_BotMovementViewTarget` — write the view target into `target` (engine fills it).
pub fn BotMovementViewTarget(
    movestate: i32,
    goal: &bot_goal_t,
    travelflags: i32,
    lookahead: f32,
    target: &mut vec3_t,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_MOVEMENT_VIEW_TARGET,
            movestate,
            goal as *const bot_goal_t,
            travelflags,
            pass_float(lookahead),
            target.as_mut_ptr()
        ) as i32
    }
}

/// `trap_BotPredictVisiblePosition` — write the predicted position into `target`
/// (engine fills it).
pub fn BotPredictVisiblePosition(
    origin: &vec3_t,
    areanum: i32,
    goal: &bot_goal_t,
    travelflags: i32,
    target: &mut vec3_t,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_PREDICT_VISIBLE_POSITION,
            origin.as_ptr(),
            areanum,
            goal as *const bot_goal_t,
            travelflags,
            target.as_mut_ptr()
        ) as i32
    }
}

/// `trap_BotAllocMoveState`.
pub fn BotAllocMoveState() -> i32 {
    unsafe { syscall!(BOTLIB_AI_ALLOC_MOVE_STATE) as i32 }
}

/// `trap_BotFreeMoveState`.
pub fn BotFreeMoveState(handle: i32) {
    unsafe {
        syscall!(BOTLIB_AI_FREE_MOVE_STATE, handle);
    }
}

/// `trap_BotInitMoveState` — initialise move state `handle` from `initmove` (a
/// `void* /* struct bot_initmove_s */`); given its faithful [`bot_initmove_t`] type.
pub fn BotInitMoveState(handle: i32, initmove: &bot_initmove_t) {
    unsafe {
        syscall!(BOTLIB_AI_INIT_MOVE_STATE, handle, initmove as *const bot_initmove_t);
    }
}

/// `trap_BotChooseBestFightWeapon`.
pub fn BotChooseBestFightWeapon(weaponstate: i32, inventory: &mut [i32]) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_CHOOSE_BEST_FIGHT_WEAPON,
            weaponstate,
            inventory.as_mut_ptr()
        ) as i32
    }
}

/// `trap_BotGetWeaponInfo` — read `weapon`'s info into `weaponinfo` (engine fills
/// it; a `void* /* struct weaponinfo_s */` in C).
pub fn BotGetWeaponInfo(weaponstate: i32, weapon: i32, weaponinfo: &mut weaponinfo_t) {
    unsafe {
        syscall!(
            BOTLIB_AI_GET_WEAPON_INFO,
            weaponstate,
            weapon,
            weaponinfo as *mut weaponinfo_t
        );
    }
}

/// `trap_BotLoadWeaponWeights`.
pub fn BotLoadWeaponWeights(weaponstate: i32, filename: &str) -> i32 {
    let f = cstr(filename);
    unsafe { syscall!(BOTLIB_AI_LOAD_WEAPON_WEIGHTS, weaponstate, f.as_ptr()) as i32 }
}

/// `trap_BotAllocWeaponState`.
pub fn BotAllocWeaponState() -> i32 {
    unsafe { syscall!(BOTLIB_AI_ALLOC_WEAPON_STATE) as i32 }
}

/// `trap_BotFreeWeaponState`.
pub fn BotFreeWeaponState(weaponstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_FREE_WEAPON_STATE, weaponstate);
    }
}

/// `trap_BotResetWeaponState`.
pub fn BotResetWeaponState(weaponstate: i32) {
    unsafe {
        syscall!(BOTLIB_AI_RESET_WEAPON_STATE, weaponstate);
    }
}

/// `trap_GeneticParentsAndChildSelection` — select genetic parents/child from
/// `ranks` (length supplies `numranks`), writing the chosen indices through
/// `parent1`/`parent2`/`child`.
pub fn GeneticParentsAndChildSelection(
    ranks: &[f32],
    parent1: &mut i32,
    parent2: &mut i32,
    child: &mut i32,
) -> i32 {
    unsafe {
        syscall!(
            BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION,
            ranks.len() as i32,
            ranks.as_ptr(),
            parent1 as *mut i32,
            parent2 as *mut i32,
            child as *mut i32
        ) as i32
    }
}
