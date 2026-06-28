// simple text
pub const ITEM_TYPE_TEXT: i32 = 0;
// button, basically text with a border
pub const ITEM_TYPE_BUTTON: i32 = 1;
// toggle button, may be grouped
pub const ITEM_TYPE_RADIOBUTTON: i32 = 2;
// check box
pub const ITEM_TYPE_CHECKBOX: i32 = 3;
// editable text, associated with a cvar
pub const ITEM_TYPE_EDITFIELD: i32 = 4;
// drop down list
pub const ITEM_TYPE_COMBO: i32 = 5;
// scrollable list
pub const ITEM_TYPE_LISTBOX: i32 = 6;
// model
pub const ITEM_TYPE_MODEL: i32 = 7;
// owner draw, name specs what it is
pub const ITEM_TYPE_OWNERDRAW: i32 = 8;
// editable text, associated with a cvar
pub const ITEM_TYPE_NUMERICFIELD: i32 = 9;
// mouse speed, volume, etc.
pub const ITEM_TYPE_SLIDER: i32 = 10;
// yes no cvar setting
pub const ITEM_TYPE_YESNO: i32 = 11;
// multiple list setting, enumerated
pub const ITEM_TYPE_MULTI: i32 = 12;
// multiple list setting, enumerated
pub const ITEM_TYPE_BIND: i32 = 13;
// scrolling text
pub const ITEM_TYPE_TEXTSCROLL: i32 = 14;

// left alignment
pub const ITEM_ALIGN_LEFT: i32 = 0;
// center alignment
pub const ITEM_ALIGN_CENTER: i32 = 1;
// right alignment
pub const ITEM_ALIGN_RIGHT: i32 = 2;

// normal text
pub const ITEM_TEXTSTYLE_NORMAL: i32 = 0;
// fast blinking
pub const ITEM_TEXTSTYLE_BLINK: i32 = 1;
// slow pulsing
pub const ITEM_TEXTSTYLE_PULSE: i32 = 2;
// drop shadow ( need a color for this )
pub const ITEM_TEXTSTYLE_SHADOWED: i32 = 3;
// drop shadow ( need a color for this )
pub const ITEM_TEXTSTYLE_OUTLINED: i32 = 4;
// drop shadow ( need a color for this )
pub const ITEM_TEXTSTYLE_OUTLINESHADOWED: i32 = 5;
// drop shadow ( need a color for this )
pub const ITEM_TEXTSTYLE_SHADOWEDMORE: i32 = 6;

// no border
pub const WINDOW_BORDER_NONE: i32 = 0;
// full border based on border color ( single pixel )
pub const WINDOW_BORDER_FULL: i32 = 1;
// horizontal borders only
pub const WINDOW_BORDER_HORZ: i32 = 2;
// vertical borders only
pub const WINDOW_BORDER_VERT: i32 = 3;
// horizontal border using the gradient bars
pub const WINDOW_BORDER_KCGRADIENT: i32 = 4;

// no background
pub const WINDOW_STYLE_EMPTY: i32 = 0;
// filled with background color
pub const WINDOW_STYLE_FILLED: i32 = 1;
// gradient bar based on background color
pub const WINDOW_STYLE_GRADIENT: i32 = 2;
// gradient bar based on background color
pub const WINDOW_STYLE_SHADER: i32 = 3;
// team color
pub const WINDOW_STYLE_TEAMCOLOR: i32 = 4;
// cinematic
pub const WINDOW_STYLE_CINEMATIC: i32 = 5;

// uh.. true
pub const MENU_TRUE: i32 = 1;
// and false
pub const MENU_FALSE: i32 = 0;

pub const HUD_VERTICAL: i32 = 0x00;
pub const HUD_HORIZONTAL: i32 = 0x01;

// list box element types
pub const LISTBOX_TEXT: i32 = 0x00;
pub const LISTBOX_IMAGE: i32 = 0x01;

// list feeders
// save games
pub const FEEDER_SAVEGAMES: i32 = 0x00;
// text maps based on game type
pub const FEEDER_MAPS: i32 = 0x01;
// servers
pub const FEEDER_SERVERS: i32 = 0x02;
// clan names
pub const FEEDER_CLANS: i32 = 0x03;
// all maps available, in graphic format
pub const FEEDER_ALLMAPS: i32 = 0x04;
// red team members
pub const FEEDER_REDTEAM_LIST: i32 = 0x05;
// blue team members
pub const FEEDER_BLUETEAM_LIST: i32 = 0x06;
// players
pub const FEEDER_PLAYER_LIST: i32 = 0x07;
// team members for team voting
pub const FEEDER_TEAM_LIST: i32 = 0x08;
//
pub const FEEDER_MODS: i32 = 0x09;
//
pub const FEEDER_DEMOS: i32 = 0x0a;
//
pub const FEEDER_SCOREBOARD: i32 = 0x0b;
// model heads
pub const FEEDER_Q3HEADS: i32 = 0x0c;
// server status
pub const FEEDER_SERVERSTATUS: i32 = 0x0d;
// find player
pub const FEEDER_FINDPLAYER: i32 = 0x0e;
// cinematics
pub const FEEDER_CINEMATICS: i32 = 0x0f;
// models/player/*w
pub const FEEDER_PLAYER_SPECIES: i32 = 0x10;
// head*.skin files in species folder
pub const FEEDER_PLAYER_SKIN_HEAD: i32 = 0x11;
// torso*.skin files in species folder
pub const FEEDER_PLAYER_SKIN_TORSO: i32 = 0x12;
// lower*.skin files in species folder
pub const FEEDER_PLAYER_SKIN_LEGS: i32 = 0x13;
// special hack to feed text/actions from playerchoice.txt in species folder
pub const FEEDER_COLORCHOICES: i32 = 0x14;
// moves for the data pad moves screen
pub const FEEDER_MOVES: i32 = 0x15;
// move titles for the data pad moves screen
pub const FEEDER_MOVES_TITLES: i32 = 0x16;
// the list of languages
pub const FEEDER_LANGUAGES: i32 = 0x17;

pub const UI_VERSION: i32 = 200;
pub const UI_HANDICAP: i32 = 200;
pub const UI_EFFECTS: i32 = 201;
pub const UI_PLAYERMODEL: i32 = 202;
pub const UI_DATAPAD_MISSION: i32 = 203;
pub const UI_DATAPAD_WEAPONS: i32 = 204;
pub const UI_DATAPAD_INVENTORY: i32 = 205;
pub const UI_DATAPAD_FORCEPOWERS: i32 = 206;
pub const UI_SKILL: i32 = 207;
pub const UI_BLUETEAMNAME: i32 = 208;
pub const UI_REDTEAMNAME: i32 = 209;
pub const UI_BLUETEAM1: i32 = 210;
pub const UI_BLUETEAM2: i32 = 211;
pub const UI_BLUETEAM3: i32 = 212;
pub const UI_BLUETEAM4: i32 = 213;
pub const UI_BLUETEAM5: i32 = 214;
pub const UI_REDTEAM1: i32 = 215;
pub const UI_REDTEAM2: i32 = 216;
pub const UI_REDTEAM3: i32 = 217;
pub const UI_REDTEAM4: i32 = 218;
pub const UI_REDTEAM5: i32 = 219;
pub const UI_NETSOURCE: i32 = 220;
pub const UI_NETMAPPREVIEW: i32 = 221;
pub const UI_NETFILTER: i32 = 222;
pub const UI_TIER: i32 = 223;
pub const UI_OPPONENTMODEL: i32 = 224;
pub const UI_TIERMAP1: i32 = 225;
pub const UI_TIERMAP2: i32 = 226;
pub const UI_TIERMAP3: i32 = 227;
pub const UI_PLAYERLOGO: i32 = 228;
pub const UI_OPPONENTLOGO: i32 = 229;
pub const UI_PLAYERLOGO_METAL: i32 = 230;
pub const UI_OPPONENTLOGO_METAL: i32 = 231;
pub const UI_PLAYERLOGO_NAME: i32 = 232;
pub const UI_OPPONENTLOGO_NAME: i32 = 233;
pub const UI_TIER_MAPNAME: i32 = 234;
pub const UI_TIER_GAMETYPE: i32 = 235;
pub const UI_ALLMAPS_SELECTION: i32 = 236;
pub const UI_OPPONENT_NAME: i32 = 237;
pub const UI_VOTE_KICK: i32 = 238;
pub const UI_BOTNAME: i32 = 239;
pub const UI_BOTSKILL: i32 = 240;
pub const UI_REDBLUE: i32 = 241;
pub const UI_CROSSHAIR: i32 = 242;
pub const UI_SELECTEDPLAYER: i32 = 243;
pub const UI_MAPCINEMATIC: i32 = 244;
pub const UI_NETGAMETYPE: i32 = 245;
pub const UI_NETMAPCINEMATIC: i32 = 246;
pub const UI_SERVERREFRESHDATE: i32 = 247;
pub const UI_SERVERMOTD: i32 = 248;
pub const UI_GLINFO: i32 = 249;
pub const UI_KEYBINDSTATUS: i32 = 250;
pub const UI_CLANCINEMATIC: i32 = 251;
pub const UI_MAP_TIMETOBEAT: i32 = 252;
pub const UI_JOINGAMETYPE: i32 = 253;
pub const UI_PREVIEWCINEMATIC: i32 = 254;
pub const UI_STARTMAPCINEMATIC: i32 = 255;
pub const UI_MAPS_SELECTION: i32 = 256;
