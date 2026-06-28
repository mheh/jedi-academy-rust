//! Ported `codemp/game/` sources — the server game module proper.
//!
//! Each module mirrors its upstream C file name: `q_shared.c` → [`q_shared`],
//! `q_math.c` → [`q_math`], `g_main.c` → [`g_main`], `tri_coll_test.c` →
//! [`tri_coll_test`], and the headers `q_shared.h` → [`q_shared_h`] and
//! `anims.h` → [`anims`]. See `roadmap/stage-1-jka-rust/01-foundation.md`.

pub mod ai_h;
pub mod ai_main;
pub mod ai_main_h;
pub mod ai_util;
pub mod ai_wpnav;
#[allow(non_snake_case)] // mirrors the upstream C file name `AnimalNPC.c`
pub mod animalnpc;
pub mod anims;
pub mod b_local_h;
pub mod b_public_h;
pub mod be_aas_h;
pub mod be_ai_char_h;
pub mod be_ai_gen_h;
pub mod be_ai_goal_h;
pub mod be_ai_move_h;
pub mod be_ai_weap_h;
pub mod be_ea_h;
pub mod bg_g2_utils;
pub mod bg_lib;
pub mod bg_local_h;
pub mod bg_misc;
pub mod bg_panimate;
pub mod bg_pmove;
pub mod bg_public;
pub mod bg_strap_h;
pub mod bg_saber;
#[allow(non_snake_case)] // mirrors the upstream C file name `bg_saberLoad.c`
pub mod bg_saberLoad;
pub mod bg_saga;
pub mod bg_saga_h;
pub mod bg_slidemove;
#[allow(non_snake_case)] // mirrors the upstream C file name `bg_vehicleLoad.c`
pub mod bg_vehicleLoad;
pub mod bg_vehicles_h;
pub mod bg_weapons;
pub mod bg_weapons_h;
pub mod botlib_h;
pub mod chars_h;
pub mod fighternpc;
pub mod g_ICARUScb;
pub mod g_ICARUScb_h;
pub mod g_active;
pub mod g_headers_h;
pub mod g_bot;
pub mod g_client;
pub mod g_cmds;
pub mod g_combat;
pub mod g_exphysics;
pub mod g_items;
pub mod g_local;
pub mod g_log;
pub mod g_main;
pub mod g_mem;
pub mod g_misc;
pub mod g_missile;
pub mod g_mover;
pub mod g_nav;
pub mod g_navnew;
pub mod g_object;
pub mod g_public_h;
pub mod g_saga;
pub mod g_session;
pub mod g_spawn;
pub mod g_svcmds;
pub mod g_target;
pub mod g_team;
pub mod g_timer;
pub mod g_trigger;
pub mod g_turret;
pub mod g_turret_g2;
pub mod g_utils;
#[allow(non_snake_case)] // mirrors the upstream C file name `g_vehicleTurret.c`
pub mod g_vehicleTurret;
pub mod g_vehicles;
pub mod g_weapon;
pub mod npc;
pub mod npc_ai_atst;
pub mod npc_ai_default;
pub mod npc_ai_droid;
pub mod npc_ai_galakmech;
pub mod npc_ai_grenadier;
pub mod npc_ai_howler;
pub mod npc_ai_imperialprobe;
pub mod npc_ai_interrogator;
pub mod npc_ai_jedi;
pub mod npc_ai_mark1;
pub mod npc_ai_mark2;
pub mod npc_ai_minemonster;
pub mod npc_ai_rancor;
pub mod npc_ai_remote;
pub mod npc_ai_seeker;
pub mod npc_ai_sentry;
pub mod npc_ai_sniper;
pub mod npc_ai_stormtrooper;
pub mod npc_ai_utils;
pub mod npc_ai_wampa;
pub mod npc_behavior;
pub mod npc_headers_h;
pub mod npc_combat;
pub mod npc_goal;
pub mod npc_misc;
pub mod npc_move;
pub mod npc_reactions;
pub mod npc_senses;
pub mod npc_sounds;
pub mod npc_spawn;
pub mod npc_stats;
pub mod npc_utils;
pub mod q_math;
pub mod q_shared;
pub mod q_shared_h;
pub mod say_h;
pub mod speedernpc;
pub mod surfaceflags_h;
pub mod syn_h;
pub mod teams_h;
pub mod tri_coll_test;
pub mod w_force;
pub mod w_saber;
pub mod w_saber_h;
pub mod walkernpc; // WalkerNPC.c — AT-ST walker vehicle fn-ptr table
