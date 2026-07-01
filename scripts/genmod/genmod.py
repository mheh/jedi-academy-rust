#!/usr/bin/env python3
"""Generate a Cargo module-crate manifest tree that mounts a transitive
`use crate::...` closure from the unchanged src/ pool via #[path].

- SEED dirs are mounted wholesale (point at the pool's own mod.rs; children
  resolve relative to the real pool file).
- Dependency files reached through the closure are grouped by dir and emitted
  as trimmed mod.rs files with #[path] leaves into the pool.
"""
import os, re, sys, json

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "src")

DIR_ALIAS = {"jpeg_6": "jpeg-6", "compiled_first": "0_compiled_first"}
ALIAS_REV = {v: k for k, v in DIR_ALIAS.items()}
CRATE_RE = re.compile(r'crate::((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*)')

def seg_to_dir(seg): return DIR_ALIAS.get(seg, seg)
def dir_to_mod(name): return ALIAS_REV.get(name, name)

def path_for(segments):
    parts = [seg_to_dir(s) for s in segments]
    f = os.path.join(SRC, *parts) + ".rs"
    if os.path.isfile(f): return os.path.join(*parts) + ".rs"
    d = os.path.join(SRC, *parts, "mod.rs")
    if os.path.isfile(d): return os.path.join(*parts, "mod.rs")
    if len(parts) >= 1:
        f2 = os.path.join(SRC, *parts[:-1]) + ".rs"
        if os.path.isfile(f2): return os.path.join(*parts[:-1]) + ".rs"
        d2 = os.path.join(SRC, *parts[:-1], "mod.rs")
        if os.path.isfile(d2): return os.path.join(*parts[:-1], "mod.rs")
    return None

def refs_in(abspath):
    out = set()
    try: txt = open(abspath, encoding="utf-8", errors="replace").read()
    except Exception: return out
    for m in CRATE_RE.finditer(txt):
        rel = path_for(m.group(1).split("::"))
        if rel: out.add(rel)
    return out

def closure(seed_files):
    seen, missing, stack = set(), set(), list(seed_files)
    while stack:
        rel = stack.pop()
        if rel in seen: continue
        seen.add(rel)
        ab = os.path.join(SRC, rel)
        if not os.path.isfile(ab): missing.add(rel); continue
        for r in refs_in(ab):
            if r not in seen: stack.append(r)
    return seen, missing

def relto(from_dir_abs, pool_rel):
    """relative #[path] string from a generated file's dir to a pool file."""
    return os.path.relpath(os.path.join(SRC, pool_rel), from_dir_abs)

def main():
    cfg = json.load(open(sys.argv[1]))
    crate = cfg["crate"]
    seed_dirs = cfg["seed_dirs"]
    reexport = cfg.get("reexport", [])
    crate_src = os.path.join(ROOT, "modules", crate, "src")

    seed_files = []
    for sd in seed_dirs:
        for dp, _, files in os.walk(os.path.join(SRC, sd)):
            for fn in files:
                if fn.endswith(".rs") and fn != "mod.rs":
                    seed_files.append(os.path.relpath(os.path.join(dp, fn), SRC))

    files, missing = closure(seed_files)
    def under_seed(rel): return any(rel == sd or rel.startswith(sd + "/") for sd in seed_dirs)
    dep_files = sorted(f for f in files if not under_seed(f) and os.path.isfile(os.path.join(SRC, f)))

    # Mount specs
    wholesale = {sd for sd in seed_dirs}          # dir rel paths mounted via pool mod.rs
    leaves = list(dep_files)                        # individual pool .rs files

    # Which dirs must be GENERATED (get their own mod.rs / lib.rs)?
    # "" = crate root (lib.rs). A wholesale dir is NOT generated (uses pool mod.rs),
    # but its PARENT is generated. A leaf's dir is generated (trimmed mod.rs).
    gen_dirs = set([""])
    def add_ancestors(d):
        # add all ancestor dirs (excluding d itself if wholesale) up to ""
        parts = d.split("/") if d else []
        for i in range(len(parts)):
            gen_dirs.add("/".join(parts[:i]))   # "" .. parent
    for w in wholesale: add_ancestors(w)
    for lf in leaves:
        d = os.path.dirname(lf)
        # the leaf's own dir is generated, plus ancestors
        parts = d.split("/") if d else []
        for i in range(len(parts) + 1):
            gen_dirs.add("/".join(parts[:i]))

    # children of each generated dir
    def children_of(D):
        pref = (D + "/") if D else ""
        kids = {}  # name -> ("wholesale"|"gen"|"leaf", info)
        # wholesale dirs whose parent == D
        for w in wholesale:
            if os.path.dirname(w) == D:
                kids[os.path.basename(w)] = ("wholesale", w)
        # leaves whose dir == D
        for lf in leaves:
            if os.path.dirname(lf) == D:
                kids[os.path.basename(lf)[:-3]] = ("leaf", lf)
        # generated subdirs whose parent == D
        for g in gen_dirs:
            if g and os.path.dirname(g) == D:
                # only if it's a real subdir producing a mod.rs (not itself wholesale)
                if g not in wholesale:
                    kids.setdefault(os.path.basename(g), ("gen", g))
        return kids

    written = []
    def emit(path, text):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        open(path, "w").write(text)
        written.append(os.path.relpath(path, ROOT))

    # Emit generated dirs
    for D in sorted(gen_dirs):
        kids = children_of(D)
        if D == "":
            here = crate_src
            lines = [f"//! {crate} — GENERATED module manifest.",
                     "//! Mounts the unchanged `src/` pool via #[path]; do not edit by hand.",
                     "//! Regenerate with scripts/genmod (see handoffs).", "",
                     "#![allow(non_snake_case)]", ""]
            # macros always
            lines += [f'#[macro_use]', f'#[path = "{relto(here, "macros.rs")}"]', "mod macros;", ""]
        else:
            here = os.path.join(crate_src, D)
            lines = [f"// GENERATED trimmed module for `{D}` — do not edit by hand.", ""]
        for name in sorted(kids):
            kind, info = kids[name]
            if kind == "wholesale":
                p = relto(here, os.path.join(info, "mod.rs"))
                gate = ""
                lines += [f'#[path = "{p}"]', f'pub mod {dir_to_mod(name)};']
            elif kind == "leaf":
                p = relto(here, info)
                base = name
                gate = '#[cfg(feature = "oracle")]\n' if base == "oracle" else ""
                lines += [f'{gate}#[path = "{p}"]', f'pub mod {base};']
            else:  # gen subdir
                lines += [f'pub mod {dir_to_mod(name)};']
        if D == "" and reexport:
            lines += [""] + [f"pub use {r};" for r in reexport]
        is_bin = cfg.get("kind") == "bin"
        if D == "" and is_bin:
            lines += ["",
                      "// WIP scaffold: this executable's C `main`/`WinMain` lives in one of the",
                      "// mounted platform modules; wiring it to Rust `fn main` is a Phase-2 task.",
                      "fn main() {",
                      '    unimplemented!("engine entry point not yet wired — see handoffs/workspace-scaffold.md");',
                      "}"]
        root_name = ("main.rs" if is_bin else "lib.rs")
        emit(os.path.join(here, "mod.rs") if D else os.path.join(here, root_name),
             "\n".join(lines) + "\n")

    print(json.dumps({"crate": crate, "closure": len(files), "dep_files": dep_files,
                      "missing": sorted(missing), "written": written}, indent=2))

if __name__ == "__main__":
    main()
