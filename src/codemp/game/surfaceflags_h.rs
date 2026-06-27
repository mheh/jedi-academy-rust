//! `surfaceflags.h` — the brush **contents** flags (`CONTENTS_*`), **surface** flags
//! (`SURF_*`) and **material** types (`MATERIAL_*`) shared by the BSP tools and the
//! game. Quote from the header: "contents flags are seperate bits — a given brush can
//! contribute multiple content bits" and "these definitions also need to be in
//! q_shared.h!".
//!
//! Pure `#define` constants → faithful `pub const`s, no layout concerns. The one
//! sign-edge case is `CONTENTS_TRANSLUCENT = 0x80000000`, which does not fit a signed
//! `int`; C stores the unsigned literal into the `int` masks as the bit pattern
//! `-2147483648`, so the Rust const is written `0x8000_0000u32 as c_int` to reproduce
//! exactly that pattern (the oracle confirms the two agree).
//!
//! The companion `MASK_*` composites (e.g. `MASK_SOLID`) live in `bg_public.h`, not
//! here, so they are ported in [`bg_public`](crate::codemp::game::bg_public).
//!
//! Oracle: `surfaceflags.h` is a clang-clean pure-`#define` header, so
//! `oracle/surfaceflags_h_oracle.c` `#include`s it directly and exposes every value in
//! declaration order; the test compares element-wise against the consts below, catching
//! any single-value typo on either side.

use core::ffi::c_int;

// contents flags are seperate bits
// a given brush can contribute multiple content bits

pub const CONTENTS_SOLID: c_int = 0x00000001; // Default setting. An eye is never valid in a solid
pub const CONTENTS_LAVA: c_int = 0x00000002;
pub const CONTENTS_WATER: c_int = 0x00000004;
pub const CONTENTS_FOG: c_int = 0x00000008;
pub const CONTENTS_PLAYERCLIP: c_int = 0x00000010;
pub const CONTENTS_MONSTERCLIP: c_int = 0x00000020; // Physically block bots
pub const CONTENTS_BOTCLIP: c_int = 0x00000040; // A hint for bots - do not enter this brush by navigation (if possible)
pub const CONTENTS_SHOTCLIP: c_int = 0x00000080;
pub const CONTENTS_BODY: c_int = 0x00000100; // should never be on a brush, only in game
pub const CONTENTS_CORPSE: c_int = 0x00000200; // should never be on a brush, only in game
pub const CONTENTS_TRIGGER: c_int = 0x00000400;
pub const CONTENTS_NODROP: c_int = 0x00000800; // don't leave bodies or items (death fog, lava)
pub const CONTENTS_TERRAIN: c_int = 0x00001000; // volume contains terrain data
pub const CONTENTS_LADDER: c_int = 0x00002000;
pub const CONTENTS_ABSEIL: c_int = 0x00004000; // (SOF2) used like ladder to define where an NPC can abseil
pub const CONTENTS_OPAQUE: c_int = 0x00008000; // defaults to on, when off, solid can be seen through
pub const CONTENTS_OUTSIDE: c_int = 0x00010000; // volume is considered to be in the outside (i.e. not indoors)

pub const CONTENTS_INSIDE: c_int = 0x10000000; // volume is considered to be inside (i.e. indoors)

pub const CONTENTS_SLIME: c_int = 0x00020000; // CHC needs this since we use same tools
pub const CONTENTS_LIGHTSABER: c_int = 0x00040000; // ""
pub const CONTENTS_TELEPORTER: c_int = 0x00080000; // ""
pub const CONTENTS_ITEM: c_int = 0x00100000; // ""
pub const CONTENTS_NOSHOT: c_int = 0x00200000; // shots pass through me
pub const CONTENTS_DETAIL: c_int = 0x08000000; // brushes not used for the bsp
pub const CONTENTS_TRANSLUCENT: c_int = 0x8000_0000u32 as c_int; // don't consume surface fragments inside

pub const SURF_SKY: c_int = 0x00002000; // lighting from environment map
pub const SURF_SLICK: c_int = 0x00004000; // affects game physics
pub const SURF_METALSTEPS: c_int = 0x00008000; // CHC needs this since we use same tools (though this flag is temp?)
pub const SURF_FORCEFIELD: c_int = 0x00010000; // CHC ""			(but not temp)
pub const SURF_NODAMAGE: c_int = 0x00040000; // never give falling damage
pub const SURF_NOIMPACT: c_int = 0x00080000; // don't make missile explosions
pub const SURF_NOMARKS: c_int = 0x00100000; // don't leave missile marks
pub const SURF_NODRAW: c_int = 0x00200000; // don't generate a drawsurface at all
pub const SURF_NOSTEPS: c_int = 0x00400000; // no footstep sounds
pub const SURF_NODLIGHT: c_int = 0x00800000; // don't dlight even if solid (solid lava, skies)
pub const SURF_NOMISCENTS: c_int = 0x01000000; // no client models allowed on this surface

pub const MATERIAL_BITS: c_int = 5;
pub const MATERIAL_MASK: c_int = 0x1f; // mask to get the material type

pub const MATERIAL_NONE: c_int = 0; // for when the artist hasn't set anything up =)
pub const MATERIAL_SOLIDWOOD: c_int = 1; // freshly cut timber
pub const MATERIAL_HOLLOWWOOD: c_int = 2; // termite infested creaky wood
pub const MATERIAL_SOLIDMETAL: c_int = 3; // solid girders
pub const MATERIAL_HOLLOWMETAL: c_int = 4; // hollow metal machines
pub const MATERIAL_SHORTGRASS: c_int = 5; // manicured lawn
pub const MATERIAL_LONGGRASS: c_int = 6; // long jungle grass
pub const MATERIAL_DIRT: c_int = 7; // hard mud
pub const MATERIAL_SAND: c_int = 8; // sandy beach
pub const MATERIAL_GRAVEL: c_int = 9; // lots of small stones
pub const MATERIAL_GLASS: c_int = 10; //
pub const MATERIAL_CONCRETE: c_int = 11; // hardened concrete pavement
pub const MATERIAL_MARBLE: c_int = 12; // marble floors
pub const MATERIAL_WATER: c_int = 13; // light covering of water on a surface
pub const MATERIAL_SNOW: c_int = 14; // freshly laid snow
pub const MATERIAL_ICE: c_int = 15; // packed snow/solid ice
pub const MATERIAL_FLESH: c_int = 16; // hung meat, corpses in the world
pub const MATERIAL_MUD: c_int = 17; // wet soil
pub const MATERIAL_BPGLASS: c_int = 18; // bulletproof glass
pub const MATERIAL_DRYLEAVES: c_int = 19; // dried up leaves on the floor
pub const MATERIAL_GREENLEAVES: c_int = 20; // fresh leaves still on a tree
pub const MATERIAL_FABRIC: c_int = 21; // Cotton sheets
pub const MATERIAL_CANVAS: c_int = 22; // tent material
pub const MATERIAL_ROCK: c_int = 23; //
pub const MATERIAL_RUBBER: c_int = 24; // hard tire like rubber
pub const MATERIAL_PLASTIC: c_int = 25; //
pub const MATERIAL_TILES: c_int = 26; // tiled floor
pub const MATERIAL_CARPET: c_int = 27; // lush carpet
pub const MATERIAL_PLASTER: c_int = 28; // drywall style plaster
pub const MATERIAL_SHATTERGLASS: c_int = 29; // glass with the Crisis Zone style shattering
pub const MATERIAL_ARMOR: c_int = 30; // body armor
pub const MATERIAL_COMPUTER: c_int = 31; // computers/electronic equipment
pub const MATERIAL_LAST: c_int = 32; // number of materials

/// `MATERIALS` (surfaceflags.h) — "Defined as a macro here so one change will affect
/// all the relevant files." The C token-paste macro expands to a comma-separated string
/// list; here it is the equivalent `&str` array, indexed by the `MATERIAL_*` ids above.
/// No oracle (a string list, not a numeric value); transcribed verbatim.
pub const MATERIALS: [&str; MATERIAL_LAST as usize] = [
    "none",
    "solidwood",
    "hollowwood",
    "solidmetal",
    "hollowmetal",
    "shortgrass",
    "longgrass",
    "dirt",
    "sand",
    "gravel",
    "glass",
    "concrete",
    "marble",
    "water",
    "snow",
    "ice",
    "flesh",
    "mud",
    "bpglass",
    "dryleaves",
    "greenleaves",
    "fabric",
    "canvas",
    "rock",
    "rubber",
    "plastic",
    "tiles",
    "carpet",
    "plaster",
    "shatterglass",
    "armor",
    "computer", // this was missing, see enums above, plus ShaderEd2 pulldown options
];

#[cfg(all(test, feature = "oracle"))]
mod tests {
    use super::*;
    use crate::oracle::jka_surfaceflags_values;

    /// Value parity: every numeric `#define` in `surfaceflags.h`, in declaration order,
    /// compared against the authentic header (`#include`d in the oracle TU). Catches a
    /// transcription typo (including the `CONTENTS_TRANSLUCENT` sign edge) on either side.
    #[test]
    fn surfaceflags_values_match_c() {
        // SAME ORDER as oracle/surfaceflags_h_oracle.c's array.
        let rust: &[c_int] = &[
            CONTENTS_SOLID,
            CONTENTS_LAVA,
            CONTENTS_WATER,
            CONTENTS_FOG,
            CONTENTS_PLAYERCLIP,
            CONTENTS_MONSTERCLIP,
            CONTENTS_BOTCLIP,
            CONTENTS_SHOTCLIP,
            CONTENTS_BODY,
            CONTENTS_CORPSE,
            CONTENTS_TRIGGER,
            CONTENTS_NODROP,
            CONTENTS_TERRAIN,
            CONTENTS_LADDER,
            CONTENTS_ABSEIL,
            CONTENTS_OPAQUE,
            CONTENTS_OUTSIDE,
            CONTENTS_INSIDE,
            CONTENTS_SLIME,
            CONTENTS_LIGHTSABER,
            CONTENTS_TELEPORTER,
            CONTENTS_ITEM,
            CONTENTS_NOSHOT,
            CONTENTS_DETAIL,
            CONTENTS_TRANSLUCENT,
            SURF_SKY,
            SURF_SLICK,
            SURF_METALSTEPS,
            SURF_FORCEFIELD,
            SURF_NODAMAGE,
            SURF_NOIMPACT,
            SURF_NOMARKS,
            SURF_NODRAW,
            SURF_NOSTEPS,
            SURF_NODLIGHT,
            SURF_NOMISCENTS,
            MATERIAL_BITS,
            MATERIAL_MASK,
            MATERIAL_NONE,
            MATERIAL_SOLIDWOOD,
            MATERIAL_HOLLOWWOOD,
            MATERIAL_SOLIDMETAL,
            MATERIAL_HOLLOWMETAL,
            MATERIAL_SHORTGRASS,
            MATERIAL_LONGGRASS,
            MATERIAL_DIRT,
            MATERIAL_SAND,
            MATERIAL_GRAVEL,
            MATERIAL_GLASS,
            MATERIAL_CONCRETE,
            MATERIAL_MARBLE,
            MATERIAL_WATER,
            MATERIAL_SNOW,
            MATERIAL_ICE,
            MATERIAL_FLESH,
            MATERIAL_MUD,
            MATERIAL_BPGLASS,
            MATERIAL_DRYLEAVES,
            MATERIAL_GREENLEAVES,
            MATERIAL_FABRIC,
            MATERIAL_CANVAS,
            MATERIAL_ROCK,
            MATERIAL_RUBBER,
            MATERIAL_PLASTIC,
            MATERIAL_TILES,
            MATERIAL_CARPET,
            MATERIAL_PLASTER,
            MATERIAL_SHATTERGLASS,
            MATERIAL_ARMOR,
            MATERIAL_COMPUTER,
            MATERIAL_LAST,
        ];
        let c = unsafe { core::slice::from_raw_parts(jka_surfaceflags_values(), rust.len()) };
        assert_eq!(rust, c, "surfaceflags.h values");
    }
}
