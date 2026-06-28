// Filename:-	g_functions.cpp
//

// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// This file contains the 8 (so far) function calls that replace the 8 function ptrs in the gentity_t structure

// #include "g_local.h"
// #include "..\cgame\cg_local.h"
// #include "g_functions.h"

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// LOCAL STUBS - these types/functions are defined in unported headers
// The actual definitions will be linked from the compiled C/C++ code or need
// to be ported separately from g_local.h, cg_local.h, and g_functions.h

// Enum variant indices for gentity_t->e_ThinkFunc
pub const thinkF_NULL: c_int = 0;
pub const thinkF_funcBBrushDieGo: c_int = 1;
pub const thinkF_ExplodeDeath: c_int = 2;
pub const thinkF_RespawnItem: c_int = 3;
pub const thinkF_G_FreeEntity: c_int = 4;
pub const thinkF_FinishSpawningItem: c_int = 5;
pub const thinkF_locateCamera: c_int = 6;
pub const thinkF_G_RunObject: c_int = 7;
pub const thinkF_ReturnToPos1: c_int = 8;
pub const thinkF_Use_BinaryMover_Go: c_int = 9;
pub const thinkF_Think_MatchTeam: c_int = 10;
pub const thinkF_Think_BeginMoving: c_int = 11;
pub const thinkF_Think_SetupTrainTargets: c_int = 12;
pub const thinkF_Think_SpawnNewDoorTrigger: c_int = 13;
pub const thinkF_ref_link: c_int = 14;
pub const thinkF_Think_Target_Delay: c_int = 15;
pub const thinkF_target_laser_think: c_int = 16;
pub const thinkF_target_laser_start: c_int = 17;
pub const thinkF_target_location_linkup: c_int = 18;
pub const thinkF_scriptrunner_run: c_int = 19;
pub const thinkF_multi_wait: c_int = 20;
pub const thinkF_multi_trigger_run: c_int = 21;
pub const thinkF_trigger_always_think: c_int = 22;
pub const thinkF_AimAtTarget: c_int = 23;
pub const thinkF_func_timer_think: c_int = 24;
pub const thinkF_NPC_RemoveBody: c_int = 25;
pub const thinkF_Disappear: c_int = 26;
pub const thinkF_NPC_Think: c_int = 27;
pub const thinkF_NPC_Spawn_Go: c_int = 28;
pub const thinkF_NPC_Begin: c_int = 29;
pub const thinkF_moverCallback: c_int = 30;
pub const thinkF_anglerCallback: c_int = 31;
pub const thinkF_RemoveOwner: c_int = 32;
pub const thinkF_MakeOwnerInvis: c_int = 33;
pub const thinkF_MakeOwnerEnergy: c_int = 34;
pub const thinkF_func_usable_think: c_int = 35;
pub const thinkF_misc_dlight_think: c_int = 36;
pub const thinkF_health_think: c_int = 37;
pub const thinkF_ammo_think: c_int = 38;
pub const thinkF_trigger_teleporter_find_closest_portal: c_int = 39;
pub const thinkF_thermalDetonatorExplode: c_int = 40;
pub const thinkF_WP_ThermalThink: c_int = 41;
pub const thinkF_trigger_hurt_reset: c_int = 42;
pub const thinkF_turret_base_think: c_int = 43;
pub const thinkF_turret_head_think: c_int = 44;
pub const thinkF_laser_arm_fire: c_int = 45;
pub const thinkF_laser_arm_start: c_int = 46;
pub const thinkF_trigger_visible_check_player_visibility: c_int = 47;
pub const thinkF_target_relay_use_go: c_int = 48;
pub const thinkF_trigger_cleared_fire: c_int = 49;
pub const thinkF_MoveOwner: c_int = 50;
pub const thinkF_SolidifyOwner: c_int = 51;
pub const thinkF_cycleCamera: c_int = 52;
pub const thinkF_spawn_ammo_crystal_trigger: c_int = 53;
pub const thinkF_NPC_ShySpawn: c_int = 54;
pub const thinkF_func_wait_return_solid: c_int = 55;
pub const thinkF_InflateOwner: c_int = 56;
pub const thinkF_mega_ammo_think: c_int = 57;
pub const thinkF_misc_replicator_item_finish_spawn: c_int = 58;
pub const thinkF_fx_runner_link: c_int = 59;
pub const thinkF_fx_runner_think: c_int = 60;
pub const thinkF_fx_rain_think: c_int = 61;
pub const thinkF_removeBoltSurface: c_int = 62;
pub const thinkF_set_MiscAnim: c_int = 63;
pub const thinkF_LimbThink: c_int = 64;
pub const thinkF_laserTrapThink: c_int = 65;
pub const thinkF_TieFighterThink: c_int = 66;
pub const thinkF_TieBomberThink: c_int = 67;
pub const thinkF_rocketThink: c_int = 68;
pub const thinkF_prox_mine_think: c_int = 69;
pub const thinkF_emplaced_blow: c_int = 70;
pub const thinkF_WP_Explode: c_int = 71;
pub const thinkF_pas_think: c_int = 72;
pub const thinkF_ion_cannon_think: c_int = 73;
pub const thinkF_maglock_link: c_int = 74;
pub const thinkF_WP_flechette_alt_blow: c_int = 75;
pub const thinkF_WP_prox_mine_think: c_int = 76;
pub const thinkF_camera_aim: c_int = 77;
pub const thinkF_fx_explosion_trail_link: c_int = 78;
pub const thinkF_fx_explosion_trail_think: c_int = 79;
pub const thinkF_fx_target_beam_link: c_int = 80;
pub const thinkF_fx_target_beam_think: c_int = 81;
pub const thinkF_spotlight_think: c_int = 82;
pub const thinkF_spotlight_link: c_int = 83;
pub const thinkF_trigger_push_checkclear: c_int = 84;
pub const thinkF_DEMP2_AltDetonate: c_int = 85;
pub const thinkF_DEMP2_AltRadiusDamage: c_int = 86;
pub const thinkF_panel_turret_think: c_int = 87;
pub const thinkF_welder_think: c_int = 88;
pub const thinkF_gas_random_jet: c_int = 89;
pub const thinkF_poll_converter: c_int = 90;
pub const thinkF_spawn_rack_goods: c_int = 91;
pub const thinkF_NoghriGasCloudThink: c_int = 92;
pub const thinkF_G_PortalifyEntities: c_int = 93;
pub const thinkF_misc_weapon_shooter_aim: c_int = 94;
pub const thinkF_misc_weapon_shooter_fire: c_int = 95;
pub const thinkF_beacon_think: c_int = 96;

pub const clThinkF_NULL: c_int = 0;
pub const clThinkF_CG_DLightThink: c_int = 1;
pub const clThinkF_CG_MatrixEffect: c_int = 2;
pub const clThinkF_CG_Limb: c_int = 3;

pub const reachedF_NULL: c_int = 0;
pub const reachedF_Reached_BinaryMover: c_int = 1;
pub const reachedF_Reached_Train: c_int = 2;
pub const reachedF_moverCallback: c_int = 3;
pub const reachedF_moveAndRotateCallback: c_int = 4;

pub const blockedF_NULL: c_int = 0;
pub const blockedF_Blocked_Door: c_int = 1;
pub const blockedF_Blocked_Mover: c_int = 2;

pub const touchF_NULL: c_int = 0;
pub const touchF_Touch_Item: c_int = 1;
pub const touchF_teleporter_touch: c_int = 2;
pub const touchF_charge_stick: c_int = 3;
pub const touchF_Touch_DoorTrigger: c_int = 4;
pub const touchF_Touch_PlatCenterTrigger: c_int = 5;
pub const touchF_Touch_Plat: c_int = 6;
pub const touchF_Touch_Button: c_int = 7;
pub const touchF_Touch_Multi: c_int = 8;
pub const touchF_trigger_push_touch: c_int = 9;
pub const touchF_trigger_teleporter_touch: c_int = 10;
pub const touchF_hurt_touch: c_int = 11;
pub const touchF_NPC_Touch: c_int = 12;
pub const touchF_touch_ammo_crystal_tigger: c_int = 13;
pub const touchF_funcBBrushTouch: c_int = 14;
pub const touchF_touchLaserTrap: c_int = 15;
pub const touchF_prox_mine_stick: c_int = 16;
pub const touchF_func_rotating_touch: c_int = 17;
pub const touchF_TouchTieBomb: c_int = 18;

pub const useF_NULL: c_int = 0;
pub const useF_funcBBrushUse: c_int = 1;
pub const useF_misc_model_use: c_int = 2;
pub const useF_Use_Item: c_int = 3;
pub const useF_Use_Shooter: c_int = 4;
pub const useF_GoExplodeDeath: c_int = 5;
pub const useF_Use_BinaryMover: c_int = 6;
pub const useF_use_wall: c_int = 7;
pub const useF_Use_Target_Give: c_int = 8;
pub const useF_Use_Target_Delay: c_int = 9;
pub const useF_Use_Target_Score: c_int = 10;
pub const useF_Use_Target_Print: c_int = 11;
pub const useF_Use_Target_Speaker: c_int = 12;
pub const useF_target_laser_use: c_int = 13;
pub const useF_target_relay_use: c_int = 14;
pub const useF_target_kill_use: c_int = 15;
pub const useF_target_counter_use: c_int = 16;
pub const useF_target_random_use: c_int = 17;
pub const useF_target_scriptrunner_use: c_int = 18;
pub const useF_target_gravity_change_use: c_int = 19;
pub const useF_target_friction_change_use: c_int = 20;
pub const useF_target_teleporter_use: c_int = 21;
pub const useF_Use_Multi: c_int = 22;
pub const useF_Use_target_push: c_int = 23;
pub const useF_hurt_use: c_int = 24;
pub const useF_func_timer_use: c_int = 25;
pub const useF_trigger_entdist_use: c_int = 26;
pub const useF_func_usable_use: c_int = 27;
pub const useF_target_activate_use: c_int = 28;
pub const useF_target_deactivate_use: c_int = 29;
pub const useF_NPC_Use: c_int = 30;
pub const useF_NPC_Spawn: c_int = 31;
pub const useF_misc_dlight_use: c_int = 32;
pub const useF_health_use: c_int = 33;
pub const useF_ammo_use: c_int = 34;
pub const useF_mega_ammo_use: c_int = 35;
pub const useF_target_level_change_use: c_int = 36;
pub const useF_target_change_parm_use: c_int = 37;
pub const useF_turret_base_use: c_int = 38;
pub const useF_laser_arm_use: c_int = 39;
pub const useF_func_static_use: c_int = 40;
pub const useF_target_play_music_use: c_int = 41;
pub const useF_misc_model_useup: c_int = 42;
pub const useF_misc_portal_use: c_int = 43;
pub const useF_target_autosave_use: c_int = 44;
pub const useF_switch_models: c_int = 45;
pub const useF_misc_replicator_item_spawn: c_int = 46;
pub const useF_misc_replicator_item_remove: c_int = 47;
pub const useF_target_secret_use: c_int = 48;
pub const useF_func_bobbing_use: c_int = 49;
pub const useF_func_rotating_use: c_int = 50;
pub const useF_fx_runner_use: c_int = 51;
pub const useF_funcGlassUse: c_int = 52;
pub const useF_TrainUse: c_int = 53;
pub const useF_misc_trip_mine_activate: c_int = 54;
pub const useF_emplaced_gun_use: c_int = 55;
pub const useF_shield_power_converter_use: c_int = 56;
pub const useF_ammo_power_converter_use: c_int = 57;
pub const useF_bomb_planted_use: c_int = 58;
pub const useF_beacon_use: c_int = 59;
pub const useF_security_panel_use: c_int = 60;
pub const useF_ion_cannon_use: c_int = 61;
pub const useF_camera_use: c_int = 62;
pub const useF_fx_explosion_trail_use: c_int = 63;
pub const useF_fx_target_beam_use: c_int = 64;
pub const useF_sentry_use: c_int = 65;
pub const useF_spotlight_use: c_int = 66;
pub const useF_misc_atst_use: c_int = 67;
pub const useF_panel_turret_use: c_int = 68;
pub const useF_welder_use: c_int = 69;
pub const useF_jabba_cam_use: c_int = 70;
pub const useF_misc_use: c_int = 71;
pub const useF_pas_use: c_int = 72;
pub const useF_item_spawn_use: c_int = 73;
pub const useF_NPC_VehicleSpawnUse: c_int = 74;
pub const useF_misc_weapon_shooter_use: c_int = 75;
pub const useF_eweb_use: c_int = 76;
pub const useF_TieFighterUse: c_int = 77;

pub const painF_NULL: c_int = 0;
pub const painF_funcBBrushPain: c_int = 1;
pub const painF_misc_model_breakable_pain: c_int = 2;
pub const painF_NPC_Pain: c_int = 3;
pub const painF_station_pain: c_int = 4;
pub const painF_func_usable_pain: c_int = 5;
pub const painF_NPC_ATST_Pain: c_int = 6;
pub const painF_NPC_ST_Pain: c_int = 7;
pub const painF_NPC_Jedi_Pain: c_int = 8;
pub const painF_NPC_Droid_Pain: c_int = 9;
pub const painF_NPC_Probe_Pain: c_int = 10;
pub const painF_NPC_MineMonster_Pain: c_int = 11;
pub const painF_NPC_Howler_Pain: c_int = 12;
pub const painF_NPC_Rancor_Pain: c_int = 13;
pub const painF_NPC_Wampa_Pain: c_int = 14;
pub const painF_NPC_SandCreature_Pain: c_int = 15;
pub const painF_NPC_Seeker_Pain: c_int = 16;
pub const painF_NPC_Remote_Pain: c_int = 17;
pub const painF_emplaced_gun_pain: c_int = 18;
pub const painF_NPC_Mark1_Pain: c_int = 19;
pub const painF_NPC_Sentry_Pain: c_int = 20;
pub const painF_NPC_Mark2_Pain: c_int = 21;
pub const painF_PlayerPain: c_int = 22;
pub const painF_GasBurst: c_int = 23;
pub const painF_CrystalCratePain: c_int = 24;
pub const painF_TurretPain: c_int = 25;
pub const painF_eweb_pain: c_int = 26;

pub const dieF_NULL: c_int = 0;
pub const dieF_funcBBrushDie: c_int = 1;
pub const dieF_misc_model_breakable_die: c_int = 2;
pub const dieF_misc_model_cargo_die: c_int = 3;
pub const dieF_func_train_die: c_int = 4;
pub const dieF_player_die: c_int = 5;
pub const dieF_ExplodeDeath_Wait: c_int = 6;
pub const dieF_ExplodeDeath: c_int = 7;
pub const dieF_func_usable_die: c_int = 8;
pub const dieF_turret_die: c_int = 9;
pub const dieF_funcGlassDie: c_int = 10;
pub const dieF_emplaced_gun_die: c_int = 11;
pub const dieF_WP_ExplosiveDie: c_int = 12;
pub const dieF_ion_cannon_die: c_int = 13;
pub const dieF_maglock_die: c_int = 14;
pub const dieF_camera_die: c_int = 15;
pub const dieF_Mark1_die: c_int = 16;
pub const dieF_Interrogator_die: c_int = 17;
pub const dieF_misc_atst_die: c_int = 18;
pub const dieF_misc_panel_turret_die: c_int = 19;
pub const dieF_thermal_die: c_int = 20;
pub const dieF_eweb_die: c_int = 21;

// Stub types for gentity_t and centity_s - actual definitions in unported headers
#[repr(C)]
pub struct gentity_t {
    pub e_ThinkFunc: c_int,
    pub e_ReachedFunc: c_int,
    pub e_BlockedFunc: c_int,
    pub e_TouchFunc: c_int,
    pub e_UseFunc: c_int,
    pub e_PainFunc: c_int,
    pub e_DieFunc: c_int,
    // Other fields omitted - this is a stub
}

#[repr(C)]
pub struct centity_s {
    pub gent: *mut gentity_t,
    // Other fields omitted - this is a stub
}

#[repr(C)]
pub struct trace_t {
    // Stub - actual definition in unported headers
}

pub type vec3_t = [f32; 3];

// Stub function declarations - these are defined in unported code
extern "C" {
    pub fn funcBBrushDieGo(self_: *mut gentity_t);
    pub fn ExplodeDeath(self_: *mut gentity_t);
    pub fn RespawnItem(self_: *mut gentity_t);
    pub fn G_FreeEntity(self_: *mut gentity_t);
    pub fn FinishSpawningItem(self_: *mut gentity_t);
    pub fn locateCamera(self_: *mut gentity_t);
    pub fn G_RunObject(self_: *mut gentity_t);
    pub fn ReturnToPos1(self_: *mut gentity_t);
    pub fn Use_BinaryMover_Go(self_: *mut gentity_t);
    pub fn Think_MatchTeam(self_: *mut gentity_t);
    pub fn Think_BeginMoving(self_: *mut gentity_t);
    pub fn Think_SetupTrainTargets(self_: *mut gentity_t);
    pub fn Think_SpawnNewDoorTrigger(self_: *mut gentity_t);
    pub fn ref_link(self_: *mut gentity_t);
    pub fn Think_Target_Delay(self_: *mut gentity_t);
    pub fn target_laser_think(self_: *mut gentity_t);
    pub fn target_laser_start(self_: *mut gentity_t);
    pub fn target_location_linkup(self_: *mut gentity_t);
    pub fn scriptrunner_run(self_: *mut gentity_t);
    pub fn multi_wait(self_: *mut gentity_t);
    pub fn multi_trigger_run(self_: *mut gentity_t);
    pub fn trigger_always_think(self_: *mut gentity_t);
    pub fn AimAtTarget(self_: *mut gentity_t);
    pub fn func_timer_think(self_: *mut gentity_t);
    pub fn NPC_RemoveBody(self_: *mut gentity_t);
    pub fn Disappear(self_: *mut gentity_t);
    pub fn NPC_Think(self_: *mut gentity_t);
    pub fn NPC_Spawn_Go(self_: *mut gentity_t);
    pub fn NPC_Begin(self_: *mut gentity_t);
    pub fn moverCallback(self_: *mut gentity_t);
    pub fn anglerCallback(self_: *mut gentity_t);
    pub fn RemoveOwner(self_: *mut gentity_t);
    pub fn MakeOwnerInvis(self_: *mut gentity_t);
    pub fn MakeOwnerEnergy(self_: *mut gentity_t);
    pub fn func_usable_think(self_: *mut gentity_t);
    pub fn misc_dlight_think(self_: *mut gentity_t);
    pub fn health_think(self_: *mut gentity_t);
    pub fn ammo_think(self_: *mut gentity_t);
    pub fn trigger_teleporter_find_closest_portal(self_: *mut gentity_t);
    pub fn thermalDetonatorExplode(self_: *mut gentity_t);
    pub fn WP_ThermalThink(self_: *mut gentity_t);
    pub fn trigger_hurt_reset(self_: *mut gentity_t);
    pub fn turret_base_think(self_: *mut gentity_t);
    pub fn turret_head_think(self_: *mut gentity_t);
    pub fn laser_arm_fire(self_: *mut gentity_t);
    pub fn laser_arm_start(self_: *mut gentity_t);
    pub fn trigger_visible_check_player_visibility(self_: *mut gentity_t);
    pub fn target_relay_use_go(self_: *mut gentity_t);
    pub fn trigger_cleared_fire(self_: *mut gentity_t);
    pub fn MoveOwner(self_: *mut gentity_t);
    pub fn SolidifyOwner(self_: *mut gentity_t);
    pub fn cycleCamera(self_: *mut gentity_t);
    pub fn spawn_ammo_crystal_trigger(self_: *mut gentity_t);
    pub fn NPC_ShySpawn(self_: *mut gentity_t);
    pub fn func_wait_return_solid(self_: *mut gentity_t);
    pub fn InflateOwner(self_: *mut gentity_t);
    pub fn mega_ammo_think(self_: *mut gentity_t);
    pub fn misc_replicator_item_finish_spawn(self_: *mut gentity_t);
    pub fn fx_runner_link(self_: *mut gentity_t);
    pub fn fx_runner_think(self_: *mut gentity_t);
    pub fn fx_rain_think(self_: *mut gentity_t);
    pub fn removeBoltSurface(self_: *mut gentity_t);
    pub fn set_MiscAnim(self_: *mut gentity_t);
    pub fn LimbThink(self_: *mut gentity_t);
    pub fn laserTrapThink(self_: *mut gentity_t);
    pub fn TieFighterThink(self_: *mut gentity_t);
    pub fn TieBomberThink(self_: *mut gentity_t);
    pub fn rocketThink(self_: *mut gentity_t);
    pub fn prox_mine_think(self_: *mut gentity_t);
    pub fn emplaced_blow(self_: *mut gentity_t);
    pub fn WP_Explode(self_: *mut gentity_t);
    pub fn pas_think(self_: *mut gentity_t);
    pub fn ion_cannon_think(self_: *mut gentity_t);
    pub fn maglock_link(self_: *mut gentity_t);
    pub fn WP_flechette_alt_blow(self_: *mut gentity_t);
    pub fn WP_prox_mine_think(self_: *mut gentity_t);
    pub fn camera_aim(self_: *mut gentity_t);
    pub fn fx_explosion_trail_link(self_: *mut gentity_t);
    pub fn fx_explosion_trail_think(self_: *mut gentity_t);
    pub fn fx_target_beam_link(self_: *mut gentity_t);
    pub fn fx_target_beam_think(self_: *mut gentity_t);
    pub fn spotlight_think(self_: *mut gentity_t);
    pub fn spotlight_link(self_: *mut gentity_t);
    pub fn trigger_push_checkclear(self_: *mut gentity_t);
    pub fn DEMP2_AltDetonate(self_: *mut gentity_t);
    pub fn DEMP2_AltRadiusDamage(self_: *mut gentity_t);
    pub fn panel_turret_think(self_: *mut gentity_t);
    pub fn welder_think(self_: *mut gentity_t);
    pub fn gas_random_jet(self_: *mut gentity_t);
    pub fn poll_converter(self_: *mut gentity_t);
    pub fn spawn_rack_goods(self_: *mut gentity_t);
    pub fn NoghriGasCloudThink(self_: *mut gentity_t);
    pub fn G_PortalifyEntities(self_: *mut gentity_t);
    pub fn misc_weapon_shooter_aim(self_: *mut gentity_t);
    pub fn misc_weapon_shooter_fire(self_: *mut gentity_t);
    pub fn beacon_think(self_: *mut gentity_t);

    pub fn CG_DLightThink(cent: *mut centity_s);
    pub fn CG_MatrixEffect(cent: *mut centity_s);
    pub fn CG_Limb(cent: *mut centity_s);

    pub fn Reached_BinaryMover(self_: *mut gentity_t);
    pub fn Reached_Train(self_: *mut gentity_t);
    pub fn moveAndRotateCallback(self_: *mut gentity_t);

    pub fn Blocked_Door(self_: *mut gentity_t, other: *mut gentity_t);
    pub fn Blocked_Mover(self_: *mut gentity_t, other: *mut gentity_t);

    pub fn Touch_Item(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn teleporter_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn charge_stick(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn Touch_DoorTrigger(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn Touch_PlatCenterTrigger(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn Touch_Plat(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn Touch_Button(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn Touch_Multi(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn trigger_push_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn trigger_teleporter_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn hurt_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn NPC_Touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn touch_ammo_crystal_tigger(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn funcBBrushTouch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn touchLaserTrap(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn prox_mine_stick(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn func_rotating_touch(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);
    pub fn TouchTieBomb(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t);

    pub fn funcBBrushUse(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_model_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Item(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Shooter(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn GoExplodeDeath(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_BinaryMover(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn use_wall(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Target_Give(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Target_Delay(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Target_Score(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Target_Print(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Target_Speaker(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_laser_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_relay_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_kill_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_counter_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_random_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_scriptrunner_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_gravity_change_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_friction_change_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_teleporter_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_Multi(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn Use_target_push(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn hurt_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn func_timer_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn trigger_entdist_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn func_usable_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_activate_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_deactivate_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn NPC_Use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn NPC_Spawn(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_dlight_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn health_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn ammo_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn mega_ammo_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_level_change_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_change_parm_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn turret_base_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn laser_arm_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn func_static_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_play_music_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_model_useup(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_portal_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_autosave_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn switch_models(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_replicator_item_spawn(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_replicator_item_remove(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn target_secret_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn func_bobbing_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn func_rotating_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn fx_runner_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn funcGlassUse(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn TrainUse(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_trip_mine_activate(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn emplaced_gun_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn shield_power_converter_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn ammo_power_converter_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn bomb_planted_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn beacon_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn security_panel_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn ion_cannon_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn camera_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn fx_explosion_trail_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn fx_target_beam_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn sentry_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn spotlight_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_atst_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn panel_turret_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn welder_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn jabba_cam_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn pas_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn item_spawn_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn NPC_VehicleSpawnUse(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn misc_weapon_shooter_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn eweb_use(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);
    pub fn TieFighterUse(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t);

    pub fn funcBBrushPain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn misc_model_breakable_pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn station_pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn func_usable_pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_ATST_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_ST_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Jedi_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Droid_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Probe_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_MineMonster_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Howler_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Rancor_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Wampa_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_SandCreature_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Seeker_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Remote_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn emplaced_gun_pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Mark1_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Sentry_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn NPC_Mark2_Pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn PlayerPain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn GasBurst(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn CrystalCratePain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn TurretPain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);
    pub fn eweb_pain(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int);

    pub fn funcBBrushDie(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn misc_model_breakable_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn misc_model_cargo_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn func_train_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn player_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn ExplodeDeath_Wait(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn ExplodeDeath(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn func_usable_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn turret_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn funcGlassDie(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn emplaced_gun_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn WP_ExplosiveDie(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn ion_cannon_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn maglock_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn camera_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn Mark1_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn Interrogator_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn misc_atst_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn misc_panel_turret_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn thermal_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);
    pub fn eweb_die(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int);

    pub fn Com_Error(code: c_int, fmt: *const c_char, ...);
}

pub fn GEntity_ThinkFunc(self_: *mut gentity_t) {
    unsafe {
        match (*self_).e_ThinkFunc {
            thinkF_NULL => {
            }

            thinkF_funcBBrushDieGo => {
                funcBBrushDieGo(self_);
            }
            thinkF_ExplodeDeath => {
                ExplodeDeath(self_);
            }
            thinkF_RespawnItem => {
                RespawnItem(self_);
            }
            thinkF_G_FreeEntity => {
                G_FreeEntity(self_);
            }
            thinkF_FinishSpawningItem => {
                FinishSpawningItem(self_);
            }
            thinkF_locateCamera => {
                locateCamera(self_);
            }
            thinkF_G_RunObject => {
                G_RunObject(self_);
            }
            thinkF_ReturnToPos1 => {
                ReturnToPos1(self_);
            }
            thinkF_Use_BinaryMover_Go => {
                Use_BinaryMover_Go(self_);
            }
            thinkF_Think_MatchTeam => {
                Think_MatchTeam(self_);
            }
            thinkF_Think_BeginMoving => {
                Think_BeginMoving(self_);
            }
            thinkF_Think_SetupTrainTargets => {
                Think_SetupTrainTargets(self_);
            }
            thinkF_Think_SpawnNewDoorTrigger => {
                Think_SpawnNewDoorTrigger(self_);
            }
            thinkF_ref_link => {
                ref_link(self_);
            }
            thinkF_Think_Target_Delay => {
                Think_Target_Delay(self_);
            }
            thinkF_target_laser_think => {
                target_laser_think(self_);
            }
            thinkF_target_laser_start => {
                target_laser_start(self_);
            }
            thinkF_target_location_linkup => {
                target_location_linkup(self_);
            }
            thinkF_scriptrunner_run => {
                scriptrunner_run(self_);
            }
            thinkF_multi_wait => {
                multi_wait(self_);
            }
            thinkF_multi_trigger_run => {
                multi_trigger_run(self_);
            }
            thinkF_trigger_always_think => {
                trigger_always_think(self_);
            }
            thinkF_AimAtTarget => {
                AimAtTarget(self_);
            }
            thinkF_func_timer_think => {
                func_timer_think(self_);
            }
            thinkF_NPC_RemoveBody => {
                NPC_RemoveBody(self_);
            }
            thinkF_Disappear => {
                Disappear(self_);
            }
            thinkF_NPC_Think => {
                NPC_Think(self_);
            }
            thinkF_NPC_Spawn_Go => {
                NPC_Spawn_Go(self_);
            }
            thinkF_NPC_Begin => {
                NPC_Begin(self_);
            }
            thinkF_moverCallback => {
                moverCallback(self_);
            }
            thinkF_anglerCallback => {
                anglerCallback(self_);
            }
            // This RemoveOwner need to exist here anymore???
            thinkF_RemoveOwner => {
                RemoveOwner(self_);
            }
            thinkF_MakeOwnerInvis => {
                MakeOwnerInvis(self_);
            }
            thinkF_MakeOwnerEnergy => {
                MakeOwnerEnergy(self_);
            }
            thinkF_func_usable_think => {
                func_usable_think(self_);
            }
            thinkF_misc_dlight_think => {
                misc_dlight_think(self_);
            }
            thinkF_health_think => {
                health_think(self_);
            }
            thinkF_ammo_think => {
                ammo_think(self_);
            }
            thinkF_trigger_teleporter_find_closest_portal => {
                trigger_teleporter_find_closest_portal(self_);
            }
            thinkF_thermalDetonatorExplode => {
                thermalDetonatorExplode(self_);
            }
            thinkF_WP_ThermalThink => {
                WP_ThermalThink(self_);
            }
            thinkF_trigger_hurt_reset => {
                trigger_hurt_reset(self_);
            }
            thinkF_turret_base_think => {
                turret_base_think(self_);
            }
            thinkF_turret_head_think => {
                turret_head_think(self_);
            }
            thinkF_laser_arm_fire => {
                laser_arm_fire(self_);
            }
            thinkF_laser_arm_start => {
                laser_arm_start(self_);
            }
            thinkF_trigger_visible_check_player_visibility => {
                trigger_visible_check_player_visibility(self_);
            }
            thinkF_target_relay_use_go => {
                target_relay_use_go(self_);
            }
            thinkF_trigger_cleared_fire => {
                trigger_cleared_fire(self_);
            }
            thinkF_MoveOwner => {
                MoveOwner(self_);
            }
            thinkF_SolidifyOwner => {
                SolidifyOwner(self_);
            }
            thinkF_cycleCamera => {
                cycleCamera(self_);
            }
            thinkF_spawn_ammo_crystal_trigger => {
                spawn_ammo_crystal_trigger(self_);
            }
            thinkF_NPC_ShySpawn => {
                NPC_ShySpawn(self_);
            }
            thinkF_func_wait_return_solid => {
                func_wait_return_solid(self_);
            }
            thinkF_InflateOwner => {
                InflateOwner(self_);
            }
            thinkF_mega_ammo_think => {
                mega_ammo_think(self_);
            }
            thinkF_misc_replicator_item_finish_spawn => {
                misc_replicator_item_finish_spawn(self_);
            }
            thinkF_fx_runner_link => {
                fx_runner_link(self_);
            }
            thinkF_fx_runner_think => {
                fx_runner_think(self_);
            }
            thinkF_fx_rain_think => {
                // delay flagging entities as portal entities (for sky portals)
                fx_rain_think(self_);
            }
            thinkF_removeBoltSurface => {
                removeBoltSurface(self_);
            }
            thinkF_set_MiscAnim => {
                set_MiscAnim(self_);
            }
            thinkF_LimbThink => {
                LimbThink(self_);
            }
            thinkF_laserTrapThink => {
                laserTrapThink(self_);
            }
            thinkF_TieFighterThink => {
                TieFighterThink(self_);
            }
            thinkF_TieBomberThink => {
                TieBomberThink(self_);
            }
            thinkF_rocketThink => {
                rocketThink(self_);
            }
            thinkF_prox_mine_think => {
                prox_mine_think(self_);
            }
            thinkF_emplaced_blow => {
                emplaced_blow(self_);
            }
            thinkF_WP_Explode => {
                WP_Explode(self_);
            }
            thinkF_pas_think => {
                // personal assault sentry
                pas_think(self_);
            }
            thinkF_ion_cannon_think => {
                ion_cannon_think(self_);
            }
            thinkF_maglock_link => {
                maglock_link(self_);
            }
            thinkF_WP_flechette_alt_blow => {
                WP_flechette_alt_blow(self_);
            }
            thinkF_WP_prox_mine_think => {
                WP_prox_mine_think(self_);
            }
            thinkF_camera_aim => {
                camera_aim(self_);
            }
            thinkF_fx_explosion_trail_link => {
                fx_explosion_trail_link(self_);
            }
            thinkF_fx_explosion_trail_think => {
                fx_explosion_trail_think(self_);
            }
            thinkF_fx_target_beam_link => {
                fx_target_beam_link(self_);
            }
            thinkF_fx_target_beam_think => {
                fx_target_beam_think(self_);
            }
            thinkF_spotlight_think => {
                spotlight_think(self_);
            }
            thinkF_spotlight_link => {
                spotlight_link(self_);
            }
            thinkF_trigger_push_checkclear => {
                trigger_push_checkclear(self_);
            }
            thinkF_DEMP2_AltDetonate => {
                DEMP2_AltDetonate(self_);
            }
            thinkF_DEMP2_AltRadiusDamage => {
                DEMP2_AltRadiusDamage(self_);
            }
            thinkF_panel_turret_think => {
                panel_turret_think(self_);
            }
            thinkF_welder_think => {
                welder_think(self_);
            }
            thinkF_gas_random_jet => {
                gas_random_jet(self_);
            }
            thinkF_poll_converter => {
                // dumb loop sound handling
                poll_converter(self_);
            }
            thinkF_spawn_rack_goods => {
                // delay spawn of goods to help on ents
                spawn_rack_goods(self_);
            }
            thinkF_NoghriGasCloudThink => {
                NoghriGasCloudThink(self_);
            }

            thinkF_G_PortalifyEntities => {
                // delay flagging entities as portal entities (for sky portals)
                G_PortalifyEntities(self_);
            }

            thinkF_misc_weapon_shooter_aim => {
                misc_weapon_shooter_aim(self_);
            }
            thinkF_misc_weapon_shooter_fire => {
                misc_weapon_shooter_fire(self_);
            }

            thinkF_beacon_think => {
                beacon_think(self_);
            }

            _ => {
                Com_Error(3, b"GEntity_ThinkFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*self_).e_ThinkFunc);
            }
        }
    }
}

// note different switch-case code for CEntity as opposed to GEntity (CEntity goes through parent GEntity first)...
//
pub fn CEntity_ThinkFunc(cent: *mut centity_s) {
    unsafe {
        match (*(*cent).gent).e_clThinkFunc {
            clThinkF_NULL => {
            }

            clThinkF_CG_DLightThink => {
                CG_DLightThink(cent);
            }
            clThinkF_CG_MatrixEffect => {
                CG_MatrixEffect(cent);
            }
            clThinkF_CG_Limb => {
                CG_Limb(cent);
            }

            _ => {
                Com_Error(3, b"CEntity_ThinkFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*(*cent).gent).e_clThinkFunc);
            }
        }
    }
}


pub fn GEntity_ReachedFunc(self_: *mut gentity_t) {
    unsafe {
        match (*self_).e_ReachedFunc {
            reachedF_NULL => {
            }

            reachedF_Reached_BinaryMover => {
                Reached_BinaryMover(self_);
            }
            reachedF_Reached_Train => {
                Reached_Train(self_);
            }
            reachedF_moverCallback => {
                moverCallback(self_);
            }
            reachedF_moveAndRotateCallback => {
                moveAndRotateCallback(self_);
            }

            _ => {
                Com_Error(3, b"GEntity_ReachedFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*self_).e_ReachedFunc);
            }
        }
    }
}



pub fn GEntity_BlockedFunc(self_: *mut gentity_t, other: *mut gentity_t) {
    unsafe {
        match (*self_).e_BlockedFunc {
            blockedF_NULL => {
            }

            blockedF_Blocked_Door => {
                Blocked_Door(self_, other);
            }
            blockedF_Blocked_Mover => {
                Blocked_Mover(self_, other);
            }

            _ => {
                Com_Error(3, b"GEntity_BlockedFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*self_).e_BlockedFunc);
            }
        }
    }
}

pub fn GEntity_TouchFunc(self_: *mut gentity_t, other: *mut gentity_t, trace: *mut trace_t) {
    unsafe {
        match (*self_).e_TouchFunc {
            touchF_NULL => {
            }

            touchF_Touch_Item => {
                Touch_Item(self_, other, trace);
            }
            touchF_teleporter_touch => {
                teleporter_touch(self_, other, trace);
            }
            touchF_charge_stick => {
                charge_stick(self_, other, trace);
            }
            touchF_Touch_DoorTrigger => {
                Touch_DoorTrigger(self_, other, trace);
            }
            touchF_Touch_PlatCenterTrigger => {
                Touch_PlatCenterTrigger(self_, other, trace);
            }
            touchF_Touch_Plat => {
                Touch_Plat(self_, other, trace);
            }
            touchF_Touch_Button => {
                Touch_Button(self_, other, trace);
            }
            touchF_Touch_Multi => {
                Touch_Multi(self_, other, trace);
            }
            touchF_trigger_push_touch => {
                trigger_push_touch(self_, other, trace);
            }
            touchF_trigger_teleporter_touch => {
                trigger_teleporter_touch(self_, other, trace);
            }
            touchF_hurt_touch => {
                hurt_touch(self_, other, trace);
            }
            touchF_NPC_Touch => {
                NPC_Touch(self_, other, trace);
            }
            touchF_touch_ammo_crystal_tigger => {
                touch_ammo_crystal_tigger(self_, other, trace);
            }
            touchF_funcBBrushTouch => {
                funcBBrushTouch(self_, other, trace);
            }
            touchF_touchLaserTrap => {
                touchLaserTrap(self_, other, trace);
            }
            touchF_prox_mine_stick => {
                prox_mine_stick(self_, other, trace);
            }
            touchF_func_rotating_touch => {
                func_rotating_touch(self_, other, trace);
            }
            touchF_TouchTieBomb => {
                TouchTieBomb(self_, other, trace);
            }

            _ => {
                Com_Error(3, b"GEntity_TouchFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*self_).e_TouchFunc);
            }
        }
    }
}

pub fn GEntity_UseFunc(self_: *mut gentity_t, other: *mut gentity_t, activator: *mut gentity_t) {
    unsafe {
        if self_.is_null() || ((*self_).e_UseFunc & 0x80000000) != 0 {
            return;
        }

        match (*self_).e_UseFunc {
            useF_NULL => {
            }

            useF_funcBBrushUse => {
                funcBBrushUse(self_, other, activator);
            }
            useF_misc_model_use => {
                misc_model_use(self_, other, activator);
            }
            useF_Use_Item => {
                Use_Item(self_, other, activator);
            }
            useF_Use_Shooter => {
                Use_Shooter(self_, other, activator);
            }
            useF_GoExplodeDeath => {
                GoExplodeDeath(self_, other, activator);
            }
            useF_Use_BinaryMover => {
                Use_BinaryMover(self_, other, activator);
            }
            useF_use_wall => {
                use_wall(self_, other, activator);
            }
            useF_Use_Target_Give => {
                Use_Target_Give(self_, other, activator);
            }
            useF_Use_Target_Delay => {
                Use_Target_Delay(self_, other, activator);
            }
            useF_Use_Target_Score => {
                Use_Target_Score(self_, other, activator);
            }
            useF_Use_Target_Print => {
                Use_Target_Print(self_, other, activator);
            }
            useF_Use_Target_Speaker => {
                Use_Target_Speaker(self_, other, activator);
            }
            useF_target_laser_use => {
                target_laser_use(self_, other, activator);
            }
            useF_target_relay_use => {
                target_relay_use(self_, other, activator);
            }
            useF_target_kill_use => {
                target_kill_use(self_, other, activator);
            }
            useF_target_counter_use => {
                target_counter_use(self_, other, activator);
            }
            useF_target_random_use => {
                target_random_use(self_, other, activator);
            }
            useF_target_scriptrunner_use => {
                target_scriptrunner_use(self_, other, activator);
            }
            useF_target_gravity_change_use => {
                target_gravity_change_use(self_, other, activator);
            }
            useF_target_friction_change_use => {
                target_friction_change_use(self_, other, activator);
            }
            useF_target_teleporter_use => {
                target_teleporter_use(self_, other, activator);
            }
            useF_Use_Multi => {
                Use_Multi(self_, other, activator);
            }
            useF_Use_target_push => {
                Use_target_push(self_, other, activator);
            }
            useF_hurt_use => {
                hurt_use(self_, other, activator);
            }
            useF_func_timer_use => {
                func_timer_use(self_, other, activator);
            }
            useF_trigger_entdist_use => {
                trigger_entdist_use(self_, other, activator);
            }
            useF_func_usable_use => {
                func_usable_use(self_, other, activator);
            }
            useF_target_activate_use => {
                target_activate_use(self_, other, activator);
            }
            useF_target_deactivate_use => {
                target_deactivate_use(self_, other, activator);
            }
            useF_NPC_Use => {
                NPC_Use(self_, other, activator);
            }
            useF_NPC_Spawn => {
                NPC_Spawn(self_, other, activator);
            }
            useF_misc_dlight_use => {
                misc_dlight_use(self_, other, activator);
            }
            useF_health_use => {
                health_use(self_, other, activator);
            }
            useF_ammo_use => {
                ammo_use(self_, other, activator);
            }
            useF_mega_ammo_use => {
                mega_ammo_use(self_, other, activator);
            }
            useF_target_level_change_use => {
                target_level_change_use(self_, other, activator);
            }
            useF_target_change_parm_use => {
                target_change_parm_use(self_, other, activator);
            }
            useF_turret_base_use => {
                turret_base_use(self_, other, activator);
            }
            useF_laser_arm_use => {
                laser_arm_use(self_, other, activator);
            }
            useF_func_static_use => {
                func_static_use(self_, other, activator);
            }
            useF_target_play_music_use => {
                target_play_music_use(self_, other, activator);
            }
            useF_misc_model_useup => {
                misc_model_useup(self_, other, activator);
            }
            useF_misc_portal_use => {
                misc_portal_use(self_, other, activator);
            }
            useF_target_autosave_use => {
                target_autosave_use(self_, other, activator);
            }
            useF_switch_models => {
                switch_models(self_, other, activator);
            }
            useF_misc_replicator_item_spawn => {
                misc_replicator_item_spawn(self_, other, activator);
            }
            useF_misc_replicator_item_remove => {
                misc_replicator_item_remove(self_, other, activator);
            }
            useF_target_secret_use => {
                target_secret_use(self_, other, activator);
            }
            useF_func_bobbing_use => {
                func_bobbing_use(self_, other, activator);
            }
            useF_func_rotating_use => {
                func_rotating_use(self_, other, activator);
            }
            useF_fx_runner_use => {
                fx_runner_use(self_, other, activator);
            }
            useF_funcGlassUse => {
                funcGlassUse(self_, other, activator);
            }
            useF_TrainUse => {
                TrainUse(self_, other, activator);
            }
            useF_misc_trip_mine_activate => {
                misc_trip_mine_activate(self_, other, activator);
            }
            useF_emplaced_gun_use => {
                emplaced_gun_use(self_, other, activator);
            }
            useF_shield_power_converter_use => {
                shield_power_converter_use(self_, other, activator);
            }
            useF_ammo_power_converter_use => {
                ammo_power_converter_use(self_, other, activator);
            }
            useF_bomb_planted_use => {
                bomb_planted_use(self_, other, activator);
            }
            useF_beacon_use => {
                beacon_use(self_, other, activator);
            }
            useF_security_panel_use => {
                security_panel_use(self_, other, activator);
            }
            useF_ion_cannon_use => {
                ion_cannon_use(self_, other, activator);
            }
            useF_camera_use => {
                camera_use(self_, other, activator);
            }
            useF_fx_explosion_trail_use => {
                fx_explosion_trail_use(self_, other, activator);
            }
            useF_fx_target_beam_use => {
                fx_target_beam_use(self_, other, activator);
            }
            useF_sentry_use => {
                sentry_use(self_, other, activator);
            }
            useF_spotlight_use => {
                spotlight_use(self_, other, activator);
            }
            useF_misc_atst_use => {
                misc_atst_use(self_, other, activator);
            }
            useF_panel_turret_use => {
                panel_turret_use(self_, other, activator);
            }
            useF_welder_use => {
                welder_use(self_, other, activator);
            }
            useF_jabba_cam_use => {
                jabba_cam_use(self_, other, activator);
            }
            useF_misc_use => {
                misc_use(self_, other, activator);
            }
            useF_pas_use => {
                pas_use(self_, other, activator);
            }
            useF_item_spawn_use => {
                item_spawn_use(self_, other, activator);
            }
            useF_NPC_VehicleSpawnUse => {
                NPC_VehicleSpawnUse(self_, other, activator);
            }
            useF_misc_weapon_shooter_use => {
                misc_weapon_shooter_use(self_, other, activator);
            }
            useF_eweb_use => {
                eweb_use(self_, other, activator);
            }
            useF_TieFighterUse => {
                TieFighterUse(self_, other, activator);
            }

            _ => {
                Com_Error(3, b"GEntity_UseFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*self_).e_UseFunc);
            }
        }
    }
}

pub fn GEntity_PainFunc(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, point: *const vec3_t, damage: c_int, mod_: c_int, hitLoc: c_int) {
    unsafe {
        match (*self_).e_PainFunc {
            painF_NULL => {
            }

            painF_funcBBrushPain => {
                funcBBrushPain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_misc_model_breakable_pain => {
                misc_model_breakable_pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Pain => {
                NPC_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_station_pain => {
                station_pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_func_usable_pain => {
                func_usable_pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_ATST_Pain => {
                NPC_ATST_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_ST_Pain => {
                NPC_ST_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Jedi_Pain => {
                NPC_Jedi_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Droid_Pain => {
                NPC_Droid_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Probe_Pain => {
                NPC_Probe_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_MineMonster_Pain => {
                NPC_MineMonster_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Howler_Pain => {
                NPC_Howler_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Rancor_Pain => {
                NPC_Rancor_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Wampa_Pain => {
                NPC_Wampa_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_SandCreature_Pain => {
                NPC_SandCreature_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Seeker_Pain => {
                NPC_Seeker_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Remote_Pain => {
                NPC_Remote_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_emplaced_gun_pain => {
                emplaced_gun_pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Mark1_Pain => {
                NPC_Mark1_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Sentry_Pain => {
                NPC_Sentry_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_NPC_Mark2_Pain => {
                NPC_Mark2_Pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_PlayerPain => {
                PlayerPain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_GasBurst => {
                GasBurst(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_CrystalCratePain => {
                CrystalCratePain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_TurretPain => {
                TurretPain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }
            painF_eweb_pain => {
                eweb_pain(self_, inflictor, attacker, point, damage, mod_, hitLoc);
            }

            _ => {
                Com_Error(3, b"GEntity_PainFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*self_).e_PainFunc);
            }
        }
    }
}


pub fn GEntity_DieFunc(self_: *mut gentity_t, inflictor: *mut gentity_t, attacker: *mut gentity_t, damage: c_int, mod_: c_int, dFlags: c_int, hitLoc: c_int) {
    unsafe {
        match (*self_).e_DieFunc {
            dieF_NULL => {
            }

            dieF_funcBBrushDie => {
                funcBBrushDie(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_misc_model_breakable_die => {
                misc_model_breakable_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_misc_model_cargo_die => {
                misc_model_cargo_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_func_train_die => {
                func_train_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_player_die => {
                player_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_ExplodeDeath_Wait => {
                ExplodeDeath_Wait(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_ExplodeDeath => {
                ExplodeDeath(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_func_usable_die => {
                func_usable_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_turret_die => {
                turret_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_funcGlassDie => {
                funcGlassDie(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            //	DIECASE( laserTrapDelayedExplode )
            dieF_emplaced_gun_die => {
                emplaced_gun_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_WP_ExplosiveDie => {
                WP_ExplosiveDie(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_ion_cannon_die => {
                ion_cannon_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_maglock_die => {
                maglock_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_camera_die => {
                camera_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_Mark1_die => {
                Mark1_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_Interrogator_die => {
                Interrogator_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_misc_atst_die => {
                misc_atst_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_misc_panel_turret_die => {
                misc_panel_turret_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_thermal_die => {
                thermal_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }
            dieF_eweb_die => {
                eweb_die(self_, inflictor, attacker, damage, mod_, dFlags, hitLoc);
            }

            _ => {
                Com_Error(3, b"GEntity_DieFunc: case %d not handled!\n\0".as_ptr() as *const c_char, (*self_).e_DieFunc);
            }
        }
    }
}

//////////////////// eof /////////////////////
