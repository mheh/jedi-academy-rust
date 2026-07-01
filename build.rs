// Build script. Only does work when the `oracle` cargo feature is enabled:
// compiles the extracted original C functions in `oracle_c/` into a static lib and
// links it, so tests can call the real Raven C as a parity oracle. Dependency-free
// (invokes the system `cc`/`ar` directly) to avoid any crates.io fetch.
// NOTE: uses `//` not `//!` so it stays valid when `include!`d from a module crate's
// build.rs (an inner doc comment is only legal at file top, not at an include site).

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // MSVC targets only: the UCRT inlines the stdio family (sscanf/sprintf/...) in
    // its headers, so those symbols are NOT in the default import lib. Our FFI calls
    // `sscanf` directly (g_spawn etc.), which links fine against glibc on the *-unix
    // targets but leaves an undefined symbol in the Windows `jampgame.dll`. Pulling in
    // legacy_stdio_definitions.lib provides the real exported definitions. Emitted for
    // EVERY build (incl. the shipped cdylib, before the oracle gate below), since the
    // shipped DLL is exactly what needs the symbol. See EXTERN_C_AUDIT.md section C.
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-lib=legacy_stdio_definitions");
    }

    // Normal builds (no `oracle` feature) do nothing — the cdylib stays pure Rust.
    if env::var_os("CARGO_FEATURE_ORACLE").is_none() {
        return;
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=oracle_c");

    let cc = env::var("CC").unwrap_or_else(|_| "cc".to_string());
    // (source, per-file extra flags). All sources also get the common base flags
    // below. `bg_lib_oracle.c` additionally needs -fwrapv so its rand() LCG's
    // signed-int overflow is defined to wrap as two's-complement, matching the Rust
    // port's wrapping_mul/wrapping_add. We scope -fwrapv to just that TU so the
    // existing q_math/q_shared oracle objects (bit-exact FP) are left untouched. It
    // also gets -Wno-null-pointer-subtraction so qsort's verbatim SWAPINIT address
    // trick `(char *)a - (char *)0` does not warn -- the source stays unmodified.
    //
    // `anims_oracle.c` is special: rather than copying the enum, it `#include`s the
    // AUTHENTIC Raven `anims.h` directly (it is a clang-clean pure enum), so the C
    // compiler reading the header is independent of the Rust port generated from it.
    // The extra -I points at the reference tree, relative to the package manifest dir
    // (this build script's cwd): the raven PC source lives in the `oracle/` submodule
    // at the project root, so headers resolve under `oracle/codemp/...`.
    let sources: &[(&str, &[&str])] = &[
        ("q_math_oracle.c", &[]),
        ("q_shared_oracle.c", &[]),
        ("q_shared_h_oracle.c", &[]),
        (
            "bg_lib_oracle.c",
            &["-fwrapv", "-Wno-null-pointer-subtraction"],
        ),
        ("tri_coll_test_oracle.c", &[]),
        ("anims_oracle.c", &["-Ioracle/codemp/game"]),
        (
            "animtable_oracle.c",
            &["-Ioracle/codemp/cgame", "-Ioracle/codemp/game"],
        ),
        ("bg_public_oracle.c", &[]),
        ("bg_public_enums_oracle.c", &[]),
        ("bg_public_structs_oracle.c", &[]),
        ("bg_vehicles_h_oracle.c", &[]),
        ("bg_weapons_h_oracle.c", &["-Ioracle/codemp/game"]),
        ("bg_weapons_oracle.c", &["-Ioracle/codemp/game"]),
        ("bg_local_h_oracle.c", &[]),
        ("bg_misc_oracle.c", &[]),
        ("bg_misc_items_oracle.c", &[]),
        ("bg_misc_parsefield_oracle.c", &[]),
        ("bg_misc_give_me_vector_from_matrix_oracle.c", &[]),
        ("bg_panimate_oracle.c", &["-Ioracle/codemp/game"]),
        ("bg_panimate_setters_oracle.c", &["-Ioracle/codemp/game"]),
        ("bg_pmove_oracle.c", &["-Ioracle/codemp/game"]),
        ("surfaceflags_h_oracle.c", &["-Ioracle/codemp/game"]),
        ("bg_saber_oracle.c", &["-Ioracle/codemp/game"]),
        ("bg_saberLoad_oracle.c", &[]),
        ("bg_saga_oracle.c", &[]),
        ("bg_vehicleLoad_oracle.c", &[]),
        ("fighternpc_oracle.c", &[]),
        ("w_saber_h_oracle.c", &["-Ioracle/codemp/game"]),
        ("w_saber_oracle.c", &[]),
        ("g_public_h_oracle.c", &[]),
        ("teams_h_oracle.c", &[]),
        ("b_public_h_oracle.c", &[]),
        ("g_local_oracle.c", &[]),
        ("ai_h_oracle.c", &[]),
        ("g_main_oracle.c", &[]),
        ("g_svcmds_oracle.c", &[]),
        ("g_spawn_oracle.c", &[]),
        ("g_utils_oracle.c", &[]),
        ("g_combat_oracle.c", &[]),
        ("g_client_oracle.c", &[]),
        ("g_mover_oracle.c", &[]),
        ("g_weapon_oracle.c", &[]),
        ("g_missile_oracle.c", &[]),
        ("g_items_oracle.c", &[]),
        ("g_misc_oracle.c", &[]),
        ("g_team_oracle.c", &[]),
        ("g_timer_oracle.c", &[]),
        ("g_cmds_oracle.c", &[]),
        ("g_trigger_oracle.c", &[]),
        ("g_vehicles_oracle.c", &[]),
        ("g_active_oracle.c", &["-Ioracle/codemp/game"]),
        ("w_force_oracle.c", &[]),
        ("ai_main_oracle.c", &[]),
        ("npc_senses_oracle.c", &[]),
        ("npc_reactions_oracle.c", &[]),
        ("npc_stats_oracle.c", &[]),
        ("npc_goal_oracle.c", &[]),
        ("g_nav_oracle.c", &[]),
        ("npc_spawn_oracle.c", &[]),
        ("npc_ai_jedi_oracle.c", &[]),
        ("g_ICARUScb_oracle.c", &[]),
    ];
    let mut objects = Vec::new();

    for (src, extra) in sources {
        let obj = out.join(format!("{src}.o"));
        let mut cmd = Command::new(&cc);
        // -ffp-contract=off: disable FMA fusion so the C does discrete IEEE-754
        // mul/add steps, matching Rust (which never auto-fuses) bit-for-bit.
        cmd.args([
            "-c",
            "-O2",
            "-fPIC",
            "-Wall",
            "-ffp-contract=off",
            "-Ioracle_c",
        ]);
        cmd.args(*extra);
        cmd.arg(format!("oracle_c/{src}")).arg("-o").arg(&obj);
        let status = cmd
            .status()
            .expect("failed to invoke C compiler for oracle");
        assert!(status.success(), "oracle: compiling {src} failed");
        objects.push(obj);
    }

    let lib = out.join("libja_oracle.a");
    let _ = std::fs::remove_file(&lib);
    let mut ar = Command::new("ar");
    ar.arg("rcs").arg(&lib);
    for obj in &objects {
        ar.arg(obj);
    }
    assert!(
        ar.status().expect("failed to invoke ar").success(),
        "oracle: archiving failed"
    );

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=ja_oracle");
}
