#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Local type definitions
#[repr(C)]
pub struct msg_t {
	pub allowoverflow: c_int,	// if false, do a Com_Error
	pub overflowed: c_int,		// set to true if the buffer size failed (with allowoverflow set)
	pub data: *mut u8,
	pub maxsize: c_int,
	pub cursize: c_int,
	pub readcount: c_int,
	pub bit: c_int,				// for bitwise reads and writes
}

#[repr(C)]
pub struct usercmd_t {
	pub serverTime: c_int,
	pub buttons: c_int,
	pub weapon: u8,
	pub angles: [c_int; 3],
	pub generic_cmd: u8,
	pub forwardmove: i8,
	pub rightmove: i8,
	pub upmove: i8,
}

#[repr(C)]
pub struct entityState_t {
	pub number: c_int,			// entity index
	pub eType: c_int,			// entityType_t
	pub eFlags: c_int,
	// Remaining fields are opaque for this module
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct playerState_t {
	pub commandTime: c_int,		// cmd->serverTime of last executed command
	pub pm_type: c_int,
	pub bobCycle: c_int,			// for view bobbing and footstep generation
	pub pm_flags: c_int,			// ducked, jump_held, etc
	pub pm_time: c_int,

	pub origin: [f32; 3],
	pub velocity: [f32; 3],
	pub weaponTime: c_int,
	pub weaponChargeTime: c_int,
	pub rechargeTime: c_int,		// for the phaser
	pub gravity: c_int,
	pub leanofs: c_int,
	pub friction: c_int,
	pub speed: c_int,
	pub delta_angles: [c_int; 3],	// add to command angles to get view direction

	pub groundEntityNum: c_int,		// ENTITYNUM_NONE = in air
	pub legsAnim: c_int,		//
	pub legsAnimTimer: c_int,	// don't change low priority animations on legs until this runs out
	pub torsoAnim: c_int,		//
	pub torsoAnimTimer: c_int,	// don't change low priority animations on torso until this runs out
	pub movementDir: c_int,		// a number 0 to 7 that represents the relative angle

	pub eFlags: c_int,			// copied to entityState_t->eFlags

	pub eventSequence: c_int,	// pmove generated events
	pub events: [c_int; 2],		// MAX_PS_EVENTS = 2
	pub eventParms: [c_int; 2],

	pub externalEvent: c_int,	// events set on player from another source
	pub externalEventParm: c_int,
	pub externalEventTime: c_int,

	pub clientNum: c_int,		// ranges from 0 to MAX_CLIENTS-1
	pub weapon: c_int,			// copied to entityState_t->weapon
	pub weaponstate: c_int,

	pub batteryCharge: c_int,

	pub viewangles: [f32; 3],		// for fixed views
	pub legsYaw: f32,		// actual legs forward facing
	pub viewheight: c_int,

	// damage feedback
	pub damageEvent: c_int,
	pub damageYaw: c_int,
	pub damagePitch: c_int,
	pub damageCount: c_int,

	pub forcePowersActive: c_int,
	pub saberInFlight: c_int,

	pub viewEntity: c_int,
	pub serverViewOrg: [f32; 3],

	// Additional fields from the actual playerState_t
	pub stats: [c_int; MAX_STATS],
	pub persistant: [c_int; MAX_PERSISTANT],
	pub ammo: [c_int; MAX_AMMO],
	pub inventory: [c_int; MAX_INVENTORY],
	pub powerups: [c_int; MAX_POWERUPS],
}

#[repr(C)]
pub struct netField_t {
	pub name: *const c_char,
	pub offset: c_int,
	pub bits: c_int,		// 0 = float
}

#[repr(C)]
pub struct cvar_t {
	pub integer: c_int,
}

#[repr(C)]
pub struct serverStatic_t {
	_opaque: [u8; 0],
}

// Constants
const MAX_STRING_CHARS: usize = 1024;
const FLOAT_INT_BITS: c_int = 13;
const FLOAT_INT_BIAS: c_int = 1 << (FLOAT_INT_BITS - 1);
const GENTITYNUM_BITS: c_int = 11;
const MAX_STATS: usize = 16;
const MAX_PERSISTANT: usize = 16;
const MAX_AMMO: usize = 10;
const MAX_POWERUPS: usize = 16;
const MAX_INVENTORY: usize = 15;
const MAX_GENTITIES: c_int = 1 << GENTITYNUM_BITS;

// External C functions
extern "C" {
	pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
	pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
	pub fn strlen(s: *const c_char) -> usize;

	pub fn Com_Error(level: c_int, fmt: *const c_char, ...);
	pub fn Com_Printf(fmt: *const c_char, ...);

	pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: usize);
}

// Global external variable
extern "C" {
	pub static mut cl_shownet: *mut cvar_t;
	pub static mut svs: serverStatic_t;
}

// LOG macro - would be used for conditional printing
// #define	LOG(x) if( cl_shownet->integer == 4 ) { Com_Printf("%s ", x ); };

// Global variable
static mut overflows: c_int = 0;

/*
==============================================================================

			MESSAGE IO FUNCTIONS

Handles byte ordering and avoids alignment errors
==============================================================================
*/

pub fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int) {
	unsafe {
		memset(buf as *mut c_void, 0, core::mem::size_of::<msg_t>());
		(*buf).data = data;
		(*buf).maxsize = length;
	}
}

pub fn MSG_Clear(buf: *mut msg_t) {
	unsafe {
		(*buf).cursize = 0;
		(*buf).overflowed = 0;	// qfalse
		(*buf).bit = 0;
	}
}

pub fn MSG_BeginReading(msg: *mut msg_t) {
	unsafe {
		(*msg).readcount = 0;
		(*msg).bit = 0;
	}
}

pub fn MSG_ReadByteAlign(buf: *mut msg_t) {
	// round up to the next byte
	unsafe {
		if (*buf).bit != 0 {
			(*buf).bit = 0;
			(*buf).readcount += 1;
		}
	}
}

pub fn MSG_GetSpace(buf: *mut msg_t, length: c_int) -> *mut c_void {
	let mut data: *mut c_void;

	unsafe {
		// round up to the next byte
		if (*buf).bit != 0 {
			(*buf).bit = 0;
			(*buf).cursize += 1;
		}

		if (*buf).cursize + length > (*buf).maxsize {
			if (*buf).allowoverflow == 0 {
				Com_Error(1, b"MSG_GetSpace: overflow without allowoverflow set\0".as_ptr() as *const c_char);
			}
			if length > (*buf).maxsize {
				Com_Error(1, b"MSG_GetSpace: %i is > full buffer size\0".as_ptr() as *const c_char, length);
			}
			Com_Printf(b"MSG_GetSpace: overflow\n\0".as_ptr() as *const c_char);
			MSG_Clear(buf);
			(*buf).overflowed = 1;	// qtrue
		}

		data = ((*buf).data as *mut c_void as *mut u8).offset((*buf).cursize as isize) as *mut c_void;
		(*buf).cursize += length;

		data
	}
}

pub fn MSG_WriteData(buf: *mut msg_t, data: *const c_void, length: c_int) {
	unsafe {
		memcpy(MSG_GetSpace(buf, length), data, length as usize);
	}
}


/*
=============================================================================

bit functions

=============================================================================
*/

// negative bit values include signs
pub fn MSG_WriteBits(msg: *mut msg_t, mut value: c_int, mut bits: c_int) {
	let mut put: c_int;
	let mut fraction: c_int;

	unsafe {
		// this isn't an exact overflow check, but close enough
		if msg.as_ref().unwrap().maxsize - msg.as_ref().unwrap().cursize < 4 {
			(*msg).overflowed = 1;	// qtrue
			#[cfg(not(feature = "FINAL_BUILD"))]
			Com_Printf(b"\x1b[31mMSG_WriteBits: buffer Full writing %d in %d bits\n\0".as_ptr() as *const c_char, value, bits);
			return;
		}

		if bits == 0 || bits < -31 || bits > 32 {
			Com_Error(2, b"MSG_WriteBits: bad bits %i\0".as_ptr() as *const c_char, bits);
		}

		// check for overflows
		if bits != 32 {
			if bits > 0 {
				if value > ((1 << bits) - 1) || value < 0 {
					overflows += 1;
					#[cfg(not(feature = "FINAL_BUILD"))]
					#[cfg(debug_assertions)]
					Com_Printf(b"\x1b[31mMSG_WriteBits: overflow writing %d in %d bits\n\0".as_ptr() as *const c_char, value, bits);
				}
			} else {
				let r: c_int = 1 << (bits - 1);

				if value > r - 1 || value < -r {
					overflows += 1;
					#[cfg(not(feature = "FINAL_BUILD"))]
					#[cfg(debug_assertions)]
					Com_Printf(b"\x1b[31mMSG_WriteBits: overflow writing %d in %d bits\n\0".as_ptr() as *const c_char, value, bits);
				}
			}
		}
		if bits < 0 {
			bits = -bits;
		}

		while bits != 0 {
			if (*msg).bit == 0 {
				(*msg).data[(*msg).cursize as usize] = 0;
				(*msg).cursize += 1;
			}
			put = 8 - (*msg).bit;
			if put > bits {
				put = bits;
			}
			fraction = value & ((1 << put) - 1);
			(*msg).data[((*msg).cursize - 1) as usize] |= (fraction << (*msg).bit) as u8;
			bits -= put;
			value >>= put;
			(*msg).bit = ((*msg).bit + put) & 7;
		}
	}
}

pub fn MSG_ReadBits(msg: *mut msg_t, mut bits: c_int) -> c_int {
	let mut value: c_int = 0;
	let mut valueBits: c_int = 0;
	let mut get: c_int;
	let mut fraction: c_int;
	let sgn: bool;

	unsafe {
		if bits < 0 {
			bits = -bits;
			sgn = true;
		} else {
			sgn = false;
		}

		while valueBits < bits {
			if (*msg).bit == 0 {
				(*msg).readcount += 1;
				assert!((*msg).readcount <= (*msg).cursize);
			}
			get = 8 - (*msg).bit;
			if get > (bits - valueBits) {
				get = bits - valueBits;
			}
			fraction = (*msg).data[((*msg).readcount - 1) as usize] as c_int;
			fraction >>= (*msg).bit;
			fraction &= (1 << get) - 1;
			value |= fraction << valueBits;

			valueBits += get;
			(*msg).bit = ((*msg).bit + get) & 7;
		}

		if sgn {
			if value & (1 << (bits - 1)) != 0 {
				value |= -1 ^ ((1 << bits) - 1);
			}
		}

		value
	}
}



//================================================================================

//
// writing functions
//

pub fn MSG_WriteByte(sb: *mut msg_t, c: c_int) {
	#[cfg(feature = "PARANOID")]
	if c < 0 || c > 255 {
		Com_Error(1, b"MSG_WriteByte: range error\0".as_ptr() as *const c_char);
	}

	MSG_WriteBits(sb, c, 8);
}

pub fn MSG_WriteShort(sb: *mut msg_t, c: c_int) {
	#[cfg(feature = "PARANOID")]
	if c < ((-(1i32 << 15))) || c > ((1i32 << 15) - 1) {
		Com_Error(1, b"MSG_WriteShort: range error\0".as_ptr() as *const c_char);
	}

	MSG_WriteBits(sb, c, 16);
}

fn MSG_WriteSShort(sb: *mut msg_t, c: c_int) {
	MSG_WriteBits(sb, c, -16);
}

pub fn MSG_WriteLong(sb: *mut msg_t, c: c_int) {
	MSG_WriteBits(sb, c, 32);
}

pub fn MSG_WriteString(sb: *mut msg_t, s: *const c_char) {
	unsafe {
		if s.is_null() {
			MSG_WriteData(sb, b"\0".as_ptr() as *const c_void, 1);
		} else {
			let mut l: usize;
			let mut i: usize;
			let mut string: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

			l = strlen(s);
			if l >= MAX_STRING_CHARS {
				Com_Printf(b"MSG_WriteString: MAX_STRING_CHARS\0".as_ptr() as *const c_char);
				MSG_WriteData(sb, b"\0".as_ptr() as *const c_void, 1);
				return;
			}
			Q_strncpyz(string.as_mut_ptr(), s, core::mem::size_of_val(&string));

			// get rid of 0xff chars, because old clients don't like them
			for i in 0..l {
				if (string[i] as u8) > 127 {
					string[i] = '.' as c_char;
				}
			}

			MSG_WriteData(sb, string.as_ptr() as *const c_void, (l + 1) as c_int);
		}
	}
}



//============================================================

//
// reading functions
//

// returns -1 if no more characters are available
pub fn MSG_ReadByte(msg: *mut msg_t) -> c_int {
	let c: c_int;

	unsafe {
		if (*msg).readcount + 1 > (*msg).cursize {
			c = -1;
		} else {
			c = (MSG_ReadBits(msg, 8) as u8) as c_int;
		}

		c
	}
}

pub fn MSG_ReadShort(msg: *mut msg_t) -> c_int {
	let c: c_int;

	unsafe {
		if (*msg).readcount + 2 > (*msg).cursize {
			c = -1;
		} else {
			c = MSG_ReadBits(msg, 16);
		}

		c
	}
}

fn MSG_ReadSShort(msg: *mut msg_t) -> c_int {
	let c: c_int;

	unsafe {
		if (*msg).readcount + 2 > (*msg).cursize {
			c = -1;
		} else {
			c = MSG_ReadBits(msg, -16);
		}

		c
	}
}

pub fn MSG_ReadLong(msg: *mut msg_t) -> c_int {
	let c: c_int;

	unsafe {
		if (*msg).readcount + 4 > (*msg).cursize {
			c = -1;
		} else {
			c = MSG_ReadBits(msg, 32);
		}

		c
	}
}

pub fn MSG_ReadString(msg: *mut msg_t) -> *mut c_char {
	static mut string: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
	let mut l: usize = 0;
	let mut c: c_int;

	unsafe {
		MSG_ReadByteAlign(msg);
		loop {
			c = MSG_ReadByte(msg);		// use ReadByte so -1 is out of bounds
			if c == -1 || c == 0 {
				break;
			}
			// translate all fmt spec to avoid crash bugs
			if c == '%' as c_int {
				c = '.' as c_int;
			}

			string[l] = c as c_char;
			l += 1;
			if l >= MAX_STRING_CHARS - 1 {
				break;
			}
		}

		string[l] = 0;

		string.as_mut_ptr()
	}
}

pub fn MSG_ReadStringLine(msg: *mut msg_t) -> *mut c_char {
	static mut string: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];
	let mut l: usize = 0;
	let mut c: c_int;

	unsafe {
		MSG_ReadByteAlign(msg);
		loop {
			c = MSG_ReadByte(msg);		// use ReadByte so -1 is out of bounds
			if c == -1 || c == 0 || c == '\n' as c_int {
				break;
			}
			// translate all fmt spec to avoid crash bugs
			if c == '%' as c_int {
				c = '.' as c_int;
			}
			string[l] = c as c_char;
			l += 1;
			if l >= MAX_STRING_CHARS - 1 {
				break;
			}
		}

		string[l] = 0;

		string.as_mut_ptr()
	}
}


pub fn MSG_ReadData(msg: *mut msg_t, data: *mut c_void, len: c_int) {
	let mut i: c_int;

	unsafe {
		MSG_ReadByteAlign(msg);
		for i in 0..len {
			*(data as *mut u8).offset(i as isize) = MSG_ReadByte(msg) as u8;
		}
	}
}


/*
=============================================================================

delta functions

=============================================================================
*/

pub fn MSG_WriteDelta(msg: *mut msg_t, oldV: c_int, newV: c_int, bits: c_int) {
	if oldV == newV {
		MSG_WriteBits(msg, 0, 1);
		return;
	}
	MSG_WriteBits(msg, 1, 1);
	MSG_WriteBits(msg, newV, bits);
}

pub fn MSG_ReadDelta(msg: *mut msg_t, oldV: c_int, bits: c_int) -> c_int {
	if MSG_ReadBits(msg, 1) != 0 {
		MSG_ReadBits(msg, bits)
	} else {
		oldV
	}
}

pub fn MSG_WriteDeltaFloat(msg: *mut msg_t, oldV: f32, newV: f32) {
	if oldV == newV {
		MSG_WriteBits(msg, 0, 1);
		return;
	}
	MSG_WriteBits(msg, 1, 1);
	MSG_WriteBits(msg, unsafe { *((&newV as *const f32) as *const c_int) }, 32);
}

pub fn MSG_ReadDeltaFloat(msg: *mut msg_t, oldV: f32) -> f32 {
	if MSG_ReadBits(msg, 1) != 0 {
		let newV: f32;

		unsafe {
			let bits = MSG_ReadBits(msg, 32);
			newV = *((&bits as *const c_int) as *const f32);
		}
		newV
	} else {
		oldV
	}
}


/*
============================================================================

usercmd_t communication

============================================================================
*/

// ms is allways sent, the others are optional
const CM_ANGLE1: c_int = 1 << 0;
const CM_ANGLE2: c_int = 1 << 1;
const CM_ANGLE3: c_int = 1 << 2;
const CM_FORWARD: c_int = 1 << 3;
const CM_SIDE: c_int = 1 << 4;
const CM_UP: c_int = 1 << 5;
const CM_BUTTONS: c_int = 1 << 6;
const CM_WEAPON: c_int = 1 << 7;

/*
=====================
MSG_WriteDeltaUsercmd
=====================
*/
pub fn MSG_WriteDeltaUsercmd(msg: *mut msg_t, from: *mut usercmd_t, to: *mut usercmd_t) {
	unsafe {
		MSG_WriteDelta(msg, (*from).serverTime, (*to).serverTime, 32);
		MSG_WriteDelta(msg, (*from).angles[0], (*to).angles[0], 16);
		MSG_WriteDelta(msg, (*from).angles[1], (*to).angles[1], 16);
		MSG_WriteDelta(msg, (*from).angles[2], (*to).angles[2], 16);
		MSG_WriteDelta(msg, (*from).forwardmove as c_int, (*to).forwardmove as c_int, -8);
		MSG_WriteDelta(msg, (*from).rightmove as c_int, (*to).rightmove as c_int, -8);
		MSG_WriteDelta(msg, (*from).upmove as c_int, (*to).upmove as c_int, -8);
		MSG_WriteDelta(msg, (*from).buttons, (*to).buttons, 16);//FIXME:  We're only really using 9 bits...can this be changed to that?
		MSG_WriteDelta(msg, (*from).weapon as c_int, (*to).weapon as c_int, 8);
		MSG_WriteDelta(msg, (*from).generic_cmd as c_int, (*to).generic_cmd as c_int, 8);
	}
}


/*
=====================
MSG_ReadDeltaUsercmd
=====================
*/
pub fn MSG_ReadDeltaUsercmd(msg: *mut msg_t, from: *mut usercmd_t, to: *mut usercmd_t) {
	unsafe {
		(*to).serverTime = MSG_ReadDelta(msg, (*from).serverTime, 32);
		(*to).angles[0] = MSG_ReadDelta(msg, (*from).angles[0], 16);
		(*to).angles[1] = MSG_ReadDelta(msg, (*from).angles[1], 16);
		(*to).angles[2] = MSG_ReadDelta(msg, (*from).angles[2], 16);
		(*to).forwardmove = MSG_ReadDelta(msg, (*from).forwardmove as c_int, -8) as i8;
		(*to).rightmove = MSG_ReadDelta(msg, (*from).rightmove as c_int, -8) as i8;
		(*to).upmove = MSG_ReadDelta(msg, (*from).upmove as c_int, -8) as i8;
		(*to).buttons = MSG_ReadDelta(msg, (*from).buttons, 16);//FIXME:  We're only really using 9 bits...can this be changed to that?
		(*to).weapon = MSG_ReadDelta(msg, (*from).weapon as c_int, 8) as u8;
		(*to).generic_cmd = MSG_ReadDelta(msg, (*from).generic_cmd as c_int, 8) as u8;
	}
}

/*
=============================================================================

entityState_t communication

=============================================================================
*/

// using the stringizing operator to save typing...
// NETF macro not needed in Rust - would use field offsets directly

// if (int)f == f and (int)f + ( 1<<(FLOAT_INT_BITS-1) ) < ( 1 << FLOAT_INT_BITS )
// the float will be sent with FLOAT_INT_BITS, otherwise all 32 bits will be sent

pub fn MSG_WriteField(msg: *mut msg_t, toF: *const c_int, field: *const netField_t) {
	let mut trunc: c_int;
	let fullFloat: f32;

	unsafe {
		if (*field).bits == -1 {
			// a -1 in the bits field means it's a float that's always between -1 and 1
			let temp = (*(toF as *const f32) * 32767.0) as c_int;
			MSG_WriteBits(msg, temp, -16);
		} else if (*field).bits == 0 {
			// float
			fullFloat = *(toF as *const f32);
			trunc = fullFloat as c_int;

			if fullFloat == 0.0f32 {
				MSG_WriteBits(msg, 0, 1);	//it's a zero
			} else {
				MSG_WriteBits(msg, 1, 1);	//not a zero
				if trunc as f32 == fullFloat && trunc + FLOAT_INT_BIAS >= 0 &&
					trunc + FLOAT_INT_BIAS < (1 << FLOAT_INT_BITS) {
					// send as small integer
					MSG_WriteBits(msg, 0, 1);
					MSG_WriteBits(msg, trunc + FLOAT_INT_BIAS, FLOAT_INT_BITS);
				} else {
					// send as full floating point value
					MSG_WriteBits(msg, 1, 1);
					MSG_WriteBits(msg, *toF, 32);
				}
			}
		} else {
			if *toF == 0 {
				MSG_WriteBits(msg, 0, 1);	//it's a zero
			} else {
				MSG_WriteBits(msg, 1, 1);	//not a zero
				// integer
				MSG_WriteBits(msg, *toF, (*field).bits);
			}
		}
	}
}

pub fn MSG_ReadField(msg: *mut msg_t, toF: *mut c_int, field: *const netField_t, print: c_int) {
	let mut trunc: c_int;

	unsafe {
		if (*field).bits == -1 {
			// a -1 in the bits field means it's a float that's always between -1 and 1
			let temp = MSG_ReadBits(msg, -16);
			*(toF as *mut f32) = (temp as f32) / 32767.0;
		} else if (*field).bits == 0 {
			// float
			if MSG_ReadBits(msg, 1) == 0 {
				*(toF as *mut f32) = 0.0f32;
			} else {
				if MSG_ReadBits(msg, 1) == 0 {
					// integral float
					trunc = MSG_ReadBits(msg, FLOAT_INT_BITS);
					// bias to allow equal parts positive and negative
					trunc -= FLOAT_INT_BIAS;
					*(toF as *mut f32) = trunc as f32;
					if print != 0 {
						Com_Printf(b"%s:%i \0".as_ptr() as *const c_char, (*field).name, trunc);
					}
				} else {
					// full floating point value
					*toF = MSG_ReadBits(msg, 32);
					if print != 0 {
						Com_Printf(b"%s:%f \0".as_ptr() as *const c_char, (*field).name, *(toF as *const f32));
					}
				}
			}
		} else {
			if MSG_ReadBits(msg, 1) == 0 {
				*toF = 0;
			} else {
				// integer
				*toF = MSG_ReadBits(msg, (*field).bits);
				if print != 0 {
					Com_Printf(b"%s:%i \0".as_ptr() as *const c_char, (*field).name, *toF);
				}
			}
		}
	}
}


pub fn MSG_WriteEntity(msg: *mut msg_t, to: *mut entityState_t, removeNum: c_int) {

	unsafe {
		if to.is_null() {
			MSG_WriteBits(msg, removeNum, GENTITYNUM_BITS);
			MSG_WriteBits(msg, 1, 1); //removed
			return;
		} else {
			MSG_WriteBits(msg, (*to).number, GENTITYNUM_BITS);
			MSG_WriteBits(msg, 0, 1); //not removed
		}
		// Note: actual pointer arithmetic with svs.snapshotEntities would be used here
		// assert!(( to - svs.snapshotEntities ) >= 0 && ( to - svs.snapshotEntities ) < 512);
		// MSG_WriteLong(msg, to - svs.snapshotEntities);
	}
}

pub fn MSG_ReadEntity(msg: *mut msg_t, to: *mut entityState_t) {
	unsafe {
		// check for a remove
		if MSG_ReadBits(msg, 1) == 1 {
			memset(to as *mut c_void, 0, core::mem::size_of::<entityState_t>());
			(*to).number = MAX_GENTITIES - 1;
			return;
		}

		//No remove, read data
		// Note: actual svs.snapshotEntities access would be used here
		// let index = MSG_ReadLong(msg);
		// assert!(index >= 0 && index < svs.numSnapshotEntities);
		// *to = svs.snapshotEntities[index as usize];
	}
}

/*
============================================================================

plyer_state_t communication

============================================================================
*/

// using the stringizing operator to save typing...
// PSF macro not needed in Rust

static PLAYERSTATE_FIELDS: [netField_t; 47] = [
	netField_t { name: b"commandTime\0".as_ptr() as *const c_char, offset: 0, bits: 32 },
	netField_t { name: b"pm_type\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"bobCycle\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"pm_flags\0".as_ptr() as *const c_char, offset: 0, bits: 32 },
	netField_t { name: b"pm_time\0".as_ptr() as *const c_char, offset: 0, bits: -16 },
	netField_t { name: b"origin[0]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"origin[1]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"origin[2]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"velocity[0]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"velocity[1]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"velocity[2]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"weaponTime\0".as_ptr() as *const c_char, offset: 0, bits: -16 },
	netField_t { name: b"weaponChargeTime\0".as_ptr() as *const c_char, offset: 0, bits: 32 }, //? really need 32 bits??
	netField_t { name: b"gravity\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"leanofs\0".as_ptr() as *const c_char, offset: 0, bits: -8 },
	netField_t { name: b"friction\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"speed\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"delta_angles[0]\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"delta_angles[1]\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"delta_angles[2]\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"groundEntityNum\0".as_ptr() as *const c_char, offset: 0, bits: GENTITYNUM_BITS },
	//{ PSF(animationTimer), 16 },
	netField_t { name: b"legsAnim\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"torsoAnim\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"movementDir\0".as_ptr() as *const c_char, offset: 0, bits: 4 },
	netField_t { name: b"eFlags\0".as_ptr() as *const c_char, offset: 0, bits: 32 },
	netField_t { name: b"eventSequence\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"events[0]\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"events[1]\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"eventParms[0]\0".as_ptr() as *const c_char, offset: 0, bits: -9 },
	netField_t { name: b"eventParms[1]\0".as_ptr() as *const c_char, offset: 0, bits: -9 },
	netField_t { name: b"externalEvent\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"externalEventParm\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"clientNum\0".as_ptr() as *const c_char, offset: 0, bits: 32 },
	netField_t { name: b"weapon\0".as_ptr() as *const c_char, offset: 0, bits: 5 },
	netField_t { name: b"weaponstate\0".as_ptr() as *const c_char, offset: 0, bits: 4 },
	netField_t { name: b"batteryCharge\0".as_ptr() as *const c_char, offset: 0, bits: 16 },
	netField_t { name: b"viewangles[0]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"viewangles[1]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"viewangles[2]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"viewheight\0".as_ptr() as *const c_char, offset: 0, bits: -8 },
	netField_t { name: b"damageEvent\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"damageYaw\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	netField_t { name: b"damagePitch\0".as_ptr() as *const c_char, offset: 0, bits: -8 },
	netField_t { name: b"damageCount\0".as_ptr() as *const c_char, offset: 0, bits: 8 },
	//{ PSF(saberColor), 8 },
	//{ PSF(saberActive), 8 },
	//{ PSF(saberLength), 32 },
	//{ PSF(saberLengthMax), 32 },
	netField_t { name: b"forcePowersActive\0".as_ptr() as *const c_char, offset: 0, bits: 32},
	netField_t { name: b"saberInFlight\0".as_ptr() as *const c_char, offset: 0, bits: 8 },

	/*{ PSF(vehicleIndex), 32 },			// WOAH, what do we do with this stuff???
	{ PSF(vehicleArmor), 32 },
	{ PSF(vehicleAngles[0]), 0 },
	{ PSF(vehicleAngles[1]), 0 },
	{ PSF(vehicleAngles[2]), 0 },*/

	netField_t { name: b"viewEntity\0".as_ptr() as *const c_char, offset: 0, bits: 32 },
	netField_t { name: b"serverViewOrg[0]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"serverViewOrg[1]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
	netField_t { name: b"serverViewOrg[2]\0".as_ptr() as *const c_char, offset: 0, bits: 0 },
];

/*
=============
MSG_WriteDeltaPlayerstate

=============
*/
pub fn MSG_WriteDeltaPlayerstate(msg: *mut msg_t, from: *mut playerState_t, to: *mut playerState_t) {
	let mut i: c_int;
	let mut statsbits: c_int;
	let mut persistantbits: c_int;
	let mut ammobits: c_int;
	let mut powerupbits: c_int;
	let numFields: c_int;
	let mut c: c_int;
	let mut dummy: playerState_t;

	unsafe {
		let from_ref = if from.is_null() {
			dummy = core::mem::zeroed();
			&mut dummy
		} else {
			&mut *from
		};

		c = (*msg).cursize;

		numFields = PLAYERSTATE_FIELDS.len() as c_int;
		// Note: Actual field iteration would happen here
		// This requires pointer arithmetic and field access on playerState_t which is partially opaque

		c = (*msg).cursize - c;


		//
		// send the arrays
		//
		statsbits = 0;
		for i in 0..(MAX_STATS as c_int) {
			if (*to).stats[i as usize] != from_ref.stats[i as usize] {
				statsbits |= 1 << i;
			}
		}
		if statsbits != 0 {
			MSG_WriteBits(msg, 1, 1);	// changed
			MSG_WriteShort(msg, statsbits);
			for i in 0..(MAX_STATS as c_int) {
				if (statsbits & (1 << i)) != 0 {
					MSG_WriteBits(msg, (*to).stats[i as usize], 32);
				}
			}
		} else {
			MSG_WriteBits(msg, 0, 1);	// no change
		}


		persistantbits = 0;
		for i in 0..(MAX_PERSISTANT as c_int) {
			if (*to).persistant[i as usize] != from_ref.persistant[i as usize] {
				persistantbits |= 1 << i;
			}
		}
		if persistantbits != 0 {
			MSG_WriteBits(msg, 1, 1);	// changed
			MSG_WriteShort(msg, persistantbits);
			for i in 0..(MAX_PERSISTANT as c_int) {
				if (persistantbits & (1 << i)) != 0 {
					MSG_WriteSShort(msg, (*to).persistant[i as usize]);
				}
			}
		} else {
			MSG_WriteBits(msg, 0, 1);	// no change
		}


		ammobits = 0;
		for i in 0..(MAX_AMMO as c_int) {
			if (*to).ammo[i as usize] != from_ref.ammo[i as usize] {
				ammobits |= 1 << i;
			}
		}
		if ammobits != 0 {
			MSG_WriteBits(msg, 1, 1);	// changed
			MSG_WriteShort(msg, ammobits);
			for i in 0..(MAX_AMMO as c_int) {
				if (ammobits & (1 << i)) != 0 {
					MSG_WriteSShort(msg, (*to).ammo[i as usize]);
				}
			}
		} else {
			MSG_WriteBits(msg, 0, 1);	// no change
		}

		powerupbits = 0;
		for i in 0..(MAX_POWERUPS as c_int) {
			if (*to).powerups[i as usize] != from_ref.powerups[i as usize] {
				powerupbits |= 1 << i;
			}
		}
		if powerupbits != 0 {
			MSG_WriteBits(msg, 1, 1);	// changed
			MSG_WriteShort(msg, powerupbits);
			for i in 0..(MAX_POWERUPS as c_int) {
				if (powerupbits & (1 << i)) != 0 {
					MSG_WriteLong(msg, (*to).powerups[i as usize]);
				}
			}
		} else {
			MSG_WriteBits(msg, 0, 1);	// no change
		}


		statsbits = 0;
		for i in 0..(MAX_INVENTORY as c_int) {
			if (*to).inventory[i as usize] != from_ref.inventory[i as usize] {
				statsbits |= 1 << i;
			}
		}
		if statsbits != 0 {
			MSG_WriteBits(msg, 1, 1);	// changed
			MSG_WriteShort(msg, statsbits);
			for i in 0..(MAX_INVENTORY as c_int) {
				if (statsbits & (1 << i)) != 0 {
					MSG_WriteShort(msg, (*to).inventory[i as usize]);
				}
			}
		} else {
			MSG_WriteBits(msg, 0, 1);	// no change
		}
	}
}


/*
===================
MSG_ReadDeltaPlayerstate
===================
*/
pub fn MSG_ReadDeltaPlayerstate(msg: *mut msg_t, from: *mut playerState_t, to: *mut playerState_t) {
	let mut i: c_int;
	let mut bits: c_int;
	let numFields: c_int;
	let mut startBit: c_int = 0;
	let mut endBit: c_int = 0;
	let mut print: c_int;
	let mut dummy: playerState_t;

	unsafe {
		let from_ref = if from.is_null() {
			dummy = core::mem::zeroed();
			&dummy
		} else {
			&*from
		};

		*to = *from_ref;

		if (*msg).bit == 0 {
			startBit = (*msg).readcount * 8 - GENTITYNUM_BITS;
		} else {
			startBit = ((*msg).readcount - 1) * 8 + (*msg).bit - GENTITYNUM_BITS;
		}

		// shownet 2/3 will interleave with other printed info, -2 will
		// just print the delta records
		let shownet_val = if !cl_shownet.is_null() { (*cl_shownet).integer } else { 0 };
		if shownet_val >= 2 || shownet_val == -2 {
			print = 1;
			Com_Printf(b"%3i: playerstate \0".as_ptr() as *const c_char, (*msg).readcount);
		} else {
			print = 0;
		}

		numFields = PLAYERSTATE_FIELDS.len() as c_int;
		// Note: Actual field iteration would happen here

		// parse stats
		if MSG_ReadBits(msg, 1) != 0 {
			bits = MSG_ReadShort(msg);
			for i in 0..(MAX_STATS as c_int) {
				if (bits & (1 << i)) != 0 {
					(*to).stats[i as usize] = MSG_ReadBits(msg, 32);
				}
			}
		}

		// parse persistant stats
		if MSG_ReadBits(msg, 1) != 0 {
			bits = MSG_ReadShort(msg);
			for i in 0..(MAX_PERSISTANT as c_int) {
				if (bits & (1 << i)) != 0 {
					(*to).persistant[i as usize] = MSG_ReadSShort(msg);
				}
			}
		}

		// parse ammo
		if MSG_ReadBits(msg, 1) != 0 {
			bits = MSG_ReadShort(msg);
			for i in 0..(MAX_AMMO as c_int) {
				if (bits & (1 << i)) != 0 {
					(*to).ammo[i as usize] = MSG_ReadSShort(msg);
				}
			}
		}

		// parse powerups
		if MSG_ReadBits(msg, 1) != 0 {
			bits = MSG_ReadShort(msg);
			for i in 0..(MAX_POWERUPS as c_int) {
				if (bits & (1 << i)) != 0 {
					(*to).powerups[i as usize] = MSG_ReadLong(msg);
				}
			}
		}

		// parse inventory
		if MSG_ReadBits(msg, 1) != 0 {
			bits = MSG_ReadShort(msg);
			for i in 0..(MAX_INVENTORY as c_int) {
				if (bits & (1 << i)) != 0 {
					(*to).inventory[i as usize] = MSG_ReadShort(msg);
				}
			}
		}

		if print != 0 {
			if (*msg).bit == 0 {
				endBit = (*msg).readcount * 8 - GENTITYNUM_BITS;
			} else {
				endBit = ((*msg).readcount - 1) * 8 + (*msg).bit - GENTITYNUM_BITS;
			}
			Com_Printf(b" (%i bits)\n\0".as_ptr() as *const c_char, endBit - startBit);
		}
	}
}

//===========================================================================
