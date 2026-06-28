//! `match.h` — match template defines.

use core::ffi::{c_char, c_int};

// make sure this is the same character as we use in chats in g_cmd.c
pub const EC: [c_char; 2] = [0x19 as c_char, 0];

// match template contexts
pub const MTCONTEXT_MISC: c_int = 2;
pub const MTCONTEXT_INITIALTEAMCHAT: c_int = 4;
pub const MTCONTEXT_TIME: c_int = 8;
pub const MTCONTEXT_TEAMMATE: c_int = 16;
pub const MTCONTEXT_ADDRESSEE: c_int = 32;
pub const MTCONTEXT_PATROLKEYAREA: c_int = 64;
pub const MTCONTEXT_REPLYCHAT: c_int = 128;
pub const MTCONTEXT_CTF: c_int = 256;

// message types
pub const MSG_NEWLEADER: c_int = 1; // new leader
pub const MSG_ENTERGAME: c_int = 2; // enter game message
pub const MSG_HELP: c_int = 3; // help someone
pub const MSG_ACCOMPANY: c_int = 4; // accompany someone
pub const MSG_DEFENDKEYAREA: c_int = 5; // defend a key area
pub const MSG_RUSHBASE: c_int = 6; // everyone rush to base
pub const MSG_GETFLAG: c_int = 7; // get the enemy flag
pub const MSG_STARTTEAMLEADERSHIP: c_int = 8; // someone wants to become the team leader
pub const MSG_STOPTEAMLEADERSHIP: c_int = 9; // someone wants to stop being the team leader
pub const MSG_WHOISTEAMLAEDER: c_int = 10; // who is the team leader
pub const MSG_WAIT: c_int = 11; // wait for someone
pub const MSG_WHATAREYOUDOING: c_int = 12; // what are you doing?
pub const MSG_JOINSUBTEAM: c_int = 13; // join a sub-team
pub const MSG_LEAVESUBTEAM: c_int = 14; // leave a sub-team
pub const MSG_CREATENEWFORMATION: c_int = 15; // create a new formation
pub const MSG_FORMATIONPOSITION: c_int = 16; // tell someone his/her position in a formation
pub const MSG_FORMATIONSPACE: c_int = 17; // set the formation intervening space
pub const MSG_DOFORMATION: c_int = 18; // form a known formation
pub const MSG_DISMISS: c_int = 19; // dismiss commanded team mates
pub const MSG_CAMP: c_int = 20; // camp somewhere
pub const MSG_CHECKPOINT: c_int = 21; // remember a check point
pub const MSG_PATROL: c_int = 22; // patrol between certain keypoints
pub const MSG_LEADTHEWAY: c_int = 23; // lead the way
pub const MSG_GETITEM: c_int = 24; // get an item
pub const MSG_KILL: c_int = 25; // kill someone
pub const MSG_WHEREAREYOU: c_int = 26; // where is someone
pub const MSG_RETURNFLAG: c_int = 27; // return the flag
pub const MSG_WHATISMYCOMMAND: c_int = 28; // ask the team leader what to do
pub const MSG_WHICHTEAM: c_int = 29; // ask which team a bot is in
pub const MSG_TASKPREFERENCE: c_int = 30; // tell your teamplay task preference
pub const MSG_ATTACKENEMYBASE: c_int = 31; // attack the enemy base
pub const MSG_HARVEST: c_int = 32; // go harvest
pub const MSG_SUICIDE: c_int = 33; // order to suicide

pub const MSG_ME: c_int = 100;
pub const MSG_EVERYONE: c_int = 101;
pub const MSG_MULTIPLENAMES: c_int = 102;
pub const MSG_NAME: c_int = 103;
pub const MSG_PATROLKEYAREA: c_int = 104;
pub const MSG_MINUTES: c_int = 105;
pub const MSG_SECONDS: c_int = 106;
pub const MSG_FOREVER: c_int = 107;
pub const MSG_FORALONGTIME: c_int = 108;
pub const MSG_FORAWHILE: c_int = 109;

pub const MSG_CHATALL: c_int = 200;
pub const MSG_CHATTEAM: c_int = 201;
pub const MSG_CHATTELL: c_int = 202;

pub const MSG_CTF: c_int = 300; // ctf message

// command sub types
pub const ST_SOMEWHERE: c_int = 0;
pub const ST_NEARITEM: c_int = 1;
pub const ST_ADDRESSED: c_int = 2;
pub const ST_METER: c_int = 4;
pub const ST_FEET: c_int = 8;
pub const ST_TIME: c_int = 16;
pub const ST_HERE: c_int = 32;
pub const ST_THERE: c_int = 64;
pub const ST_I: c_int = 128;
pub const ST_MORE: c_int = 256;
pub const ST_BACK: c_int = 512;
pub const ST_REVERSE: c_int = 1024;
pub const ST_SOMEONE: c_int = 2048;
pub const ST_GOTFLAG: c_int = 4096;
pub const ST_CAPTUREDFLAG: c_int = 8192;
pub const ST_RETURNEDFLAG: c_int = 16384;
pub const ST_TEAM: c_int = 32768;
pub const ST_1FCTFGOTFLAG: c_int = 65535;
// ctf task preferences
pub const ST_DEFENDER: c_int = 1;
pub const ST_ATTACKER: c_int = 2;
pub const ST_ROAMER: c_int = 4;

// word replacement variables
pub const THE_ENEMY: c_int = 7;
pub const THE_TEAM: c_int = 7;
// team message variables
pub const NETNAME: c_int = 0;
pub const PLACE: c_int = 1;
pub const FLAG: c_int = 1;
pub const MESSAGE: c_int = 2;
pub const ADDRESSEE: c_int = 2;
pub const ITEM: c_int = 3;
pub const TEAMMATE: c_int = 4;
pub const TEAMNAME: c_int = 4;
pub const ENEMY: c_int = 4;
pub const KEYAREA: c_int = 5;
pub const FORMATION: c_int = 5;
pub const POSITION: c_int = 5;
pub const NUMBER: c_int = 5;
pub const TIME: c_int = 6;
pub const NAME: c_int = 6;
pub const MORE: c_int = 6;
