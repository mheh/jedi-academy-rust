// Copyright (C) 1999-2000 Id Software, Inc.
//
// This file must be identical in the quake and utils directories

// contents flags are seperate bits
// a given brush can contribute multiple content bits

// these definitions also need to be in q_shared.h!

// flags pasted from SOF2, be VERY CAREFUL when adding new ones since we share their tools!!!
pub const CONTENTS_NONE: u32 = 0x00000000;
pub const CONTENTS_SOLID: u32 = 0x00000001;	// Default setting. An eye is never valid in a solid
pub const CONTENTS_LAVA: u32 = 0x00000002;
pub const CONTENTS_WATER: u32 = 0x00000004;
pub const CONTENTS_FOG: u32 = 0x00000008;
pub const CONTENTS_PLAYERCLIP: u32 = 0x00000010;	// Player physically blocked
pub const CONTENTS_MONSTERCLIP: u32 = 0x00000020;	// NPCs cannot physically pass through
pub const CONTENTS_BOTCLIP: u32 = 0x00000040;	// do not enter - NPCs try not to enter these
pub const CONTENTS_SHOTCLIP: u32 = 0x00000080;	// shots physically blocked
pub const CONTENTS_BODY: u32 = 0x00000100;	// should never be on a brush, only in game
pub const CONTENTS_CORPSE: u32 = 0x00000200;	// should never be on a brush, only in game
pub const CONTENTS_TRIGGER: u32 = 0x00000400;
pub const CONTENTS_NODROP: u32 = 0x00000800;	// don't leave bodies or items (death fog, lava)
pub const CONTENTS_TERRAIN: u32 = 0x00001000;	// volume contains terrain data
pub const CONTENTS_LADDER: u32 = 0x00002000;
pub const CONTENTS_ABSEIL: u32 = 0x00004000;  // used like ladder to define where an NPC can abseil
pub const CONTENTS_OPAQUE: u32 = 0x00008000;	// defaults to on, when off, solid can be seen through
pub const CONTENTS_OUTSIDE: u32 = 0x00010000;	// volume is considered to be in the outside (i.e. not indoors)
// CONTENTS_INSIDE added 10/31/02 by Aurelio.
pub const CONTENTS_INSIDE: u32 = 0x10000000;	// volume is considered to be inside (i.e. indoors)
pub const CONTENTS_SLIME: u32 = 0x00020000;	// CHC needs this since we use same tools
pub const CONTENTS_LIGHTSABER: u32 = 0x00040000;	// ""
pub const CONTENTS_TELEPORTER: u32 = 0x00080000;	// ""
pub const CONTENTS_ITEM: u32 = 0x00100000;	// ""
pub const CONTENTS_DETAIL: u32 = 0x08000000;	// brushes not used for the bsp
pub const CONTENTS_TRANSLUCENT: u32 = 0x80000000;	// don't consume surface fragments inside


// flags pasted from SOF2, be VERY CAREFUL when adding new ones since we share their tools!!!

pub const SURF_SKY: u32 = 0x00002000;	// lighting from environment map
pub const SURF_SLICK: u32 = 0x00004000;	// affects game physics
pub const SURF_METALSTEPS: u32 = 0x00008000;	// CHC needs this since we use same tools (though this flag is temp?)
pub const SURF_FORCEFIELD: u32 = 0x00010000;	// CHC ""			(but not temp)
pub const SURF_NODAMAGE: u32 = 0x00040000;	// never give falling damage
pub const SURF_NOIMPACT: u32 = 0x00080000;	// don't make missile explosions
pub const SURF_NOMARKS: u32 = 0x00100000;	// don't leave missile marks
pub const SURF_NODRAW: u32 = 0x00200000;	// don't generate a drawsurface at all
pub const SURF_NOSTEPS: u32 = 0x00400000;	// no footstep sounds
pub const SURF_NODLIGHT: u32 = 0x00800000;	// don't dlight even if solid (solid lava, skies)
pub const SURF_NOMISCENTS: u32 = 0x01000000;	// no client models allowed on this surface
pub const SURF_FORCESIGHT: u32 = 0x02000000;	// not visible without Force Sight

pub const SURF_PATCH: u32 = 0x80000000;	// Mark this face as a patch(editor only)

pub const MATERIAL_BITS: u32 = 5;
pub const MATERIAL_MASK: u32 = 0x1f;	// mask to get the material type

pub const MATERIAL_NONE: u32 = 0;			// for when the artist hasn't set anything up =)
pub const MATERIAL_SOLIDWOOD: u32 = 1;			// freshly cut timber
pub const MATERIAL_HOLLOWWOOD: u32 = 2;			// termite infested creaky wood
pub const MATERIAL_SOLIDMETAL: u32 = 3;			// solid girders
pub const MATERIAL_HOLLOWMETAL: u32 = 4;			// hollow metal machines
pub const MATERIAL_SHORTGRASS: u32 = 5;			// manicured lawn
pub const MATERIAL_LONGGRASS: u32 = 6;			// long jungle grass
pub const MATERIAL_DIRT: u32 = 7;			// hard mud
pub const MATERIAL_SAND: u32 = 8;			// sandy beach
pub const MATERIAL_GRAVEL: u32 = 9;			// lots of small stones
pub const MATERIAL_GLASS: u32 = 10;			//
pub const MATERIAL_CONCRETE: u32 = 11;			// hardened concrete pavement
pub const MATERIAL_MARBLE: u32 = 12;			// marble floors
pub const MATERIAL_WATER: u32 = 13;			// light covering of water on a surface
pub const MATERIAL_SNOW: u32 = 14;			// freshly laid snow
pub const MATERIAL_ICE: u32 = 15;			// packed snow/solid ice
pub const MATERIAL_FLESH: u32 = 16;			// hung meat, corpses in the world
pub const MATERIAL_MUD: u32 = 17;			// wet soil
pub const MATERIAL_BPGLASS: u32 = 18;			// bulletproof glass
pub const MATERIAL_DRYLEAVES: u32 = 19;			// dried up leaves on the floor
pub const MATERIAL_GREENLEAVES: u32 = 20;			// fresh leaves still on a tree
pub const MATERIAL_FABRIC: u32 = 21;			// Cotton sheets
pub const MATERIAL_CANVAS: u32 = 22;			// tent material
pub const MATERIAL_ROCK: u32 = 23;			//
pub const MATERIAL_RUBBER: u32 = 24;			// hard tire like rubber
pub const MATERIAL_PLASTIC: u32 = 25;			//
pub const MATERIAL_TILES: u32 = 26;			// tiled floor
pub const MATERIAL_CARPET: u32 = 27;			// lush carpet
pub const MATERIAL_PLASTER: u32 = 28;			// drywall style plaster
pub const MATERIAL_SHATTERGLASS: u32 = 29;			// glass with the Crisis Zone style shattering
pub const MATERIAL_ARMOR: u32 = 30;			// body armor
pub const MATERIAL_COMPUTER: u32 = 31;			// computers/electronic equipment
pub const MATERIAL_LAST: u32 = 32;			// number of materials

// Defined as a macro here so one change will affect all the relevant files

pub const MATERIALS: &[&str] = &[
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
	"computer", /* this was missing, see enums above, plus ShaderEd2 pulldown options */
];
