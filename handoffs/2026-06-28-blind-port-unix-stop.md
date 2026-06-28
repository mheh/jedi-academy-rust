# Blind Port Handoff: codemp/unix stop

Date: 2026-06-28

Pairing command:
- `scripts/compare-src-oracle.sh`

Latest pairing result:
- Oracle files without paired Rust files: 1148
- `oracle/code`: 646
- `oracle/codemp`: 502

Delegated files this batch:
- `oracle/codemp/unix/vm_x86.c` -> `src/codemp/unix/vm_x86.rs`
- `oracle/codemp/unix/linux_joystick.c` -> `src/codemp/unix/linux_joystick.rs`
- `oracle/codemp/unix/linux_snd.c` -> `src/codemp/unix/linux_snd.rs`

Committed files this batch:
- `7517ef0 port oracle/codemp/unix/vm_x86.c`
- `6b6814d port oracle/codemp/unix/linux_joystick.c`
- `7219fff port oracle/codemp/unix/linux_snd.c`

Returned with unresolved dependencies:
- None blocked.
- `vm_x86.rs` has local opaque `vm_t` and `vmHeader_t` stubs; `VM_CallCompiled` preserves the C non-void no-return UB with `unreachable_unchecked`.
- `linux_joystick.rs` has local cvar/key/event/Linux joystick ABI stubs and externs for libc/syscall/engine calls.
- `linux_snd.rs` has local OSS/dma/cvar ABI mirrors, extern `saved_euid`/`dma`, and libc/engine externs. It preserves the C mmap null-only failure check and `tryrates[i]` out-of-bounds possibility.

Next recommended batch:
- Continue `oracle/codemp/unix/` with one file per agent:
  - `oracle/codemp/unix/linux_common.c`
  - `oracle/codemp/unix/unix_shared.cpp`
  - `oracle/codemp/unix/unix_net.c`
- Keep larger files (`files_linux.cpp`, `linux_glimp.c`, `linux_qgl.c`, `unix_main.c`) as single-agent jobs.

Current branch and HEAD:
- Branch: `full-port`
- HEAD: `7219fff port oracle/codemp/unix/linux_snd.c`

Agent failures:
- None.

Notes:
- No build, test, cargo check, or cargo fmt was run, per blind-port rules.
- `oracle/` was not modified.
