#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_assignments)]

use crate::code::game::q_shared_h::*;
use crate::code::qcommon::qcommon_h::*;
use crate::code::server::server_h::*;
use core::ffi::{c_char, c_float, c_int, c_void};
use core::ptr::{addr_of, addr_of_mut};

extern "C" {
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    // extern cvar_t *cl_shownet; -- declared twice in C source (lines 381 and 827); declared once here
    pub static mut cl_shownet: *mut cvar_t;
    // extern serverStatic_t svs;
    pub static mut svs: serverStatic_t;
}

// if (int)f == f and (int)f + ( 1<<(FLOAT_INT_BITS-1) ) < ( 1 << FLOAT_INT_BITS )
// the float will be sent with FLOAT_INT_BITS, otherwise all 32 bits will be sent
const FLOAT_INT_BITS: c_int = 13;
const FLOAT_INT_BIAS: c_int = 1 << (FLOAT_INT_BITS - 1);

// netField_t is defined in this file (used by entityState/playerState delta functions)
#[repr(C)]
pub struct netField_t {
    pub name: *const c_char,
    pub offset: c_int,
    pub bits: c_int,        // 0 = float
}

// SAFETY: netField_t holds only a 'static string pointer and two integers;
// the table is never mutated after construction.
unsafe impl Sync for netField_t {}

// #define LOG(x) if( cl_shownet->integer == 4 ) { Com_Printf("%s ", x ); };
macro_rules! LOG {
    ($x:expr) => {
        // SAFETY: cl_shownet is a valid pointer set by the engine before use
        unsafe {
            if (*cl_shownet).integer == 4 {
                Com_Printf(b"%s \0".as_ptr() as *const c_char, $x as *const c_char);
            }
        }
    };
}

/*
==============================================================================

            MESSAGE IO FUNCTIONS

Handles byte ordering and avoids alignment errors
==============================================================================
*/


pub unsafe fn MSG_Init(buf: *mut msg_t, data: *mut u8, length: c_int) {
    memset(buf as *mut c_void, 0, core::mem::size_of::<msg_t>());
    (*buf).data = data;
    (*buf).maxsize = length;
}

pub unsafe fn MSG_Clear(buf: *mut msg_t) {
    (*buf).cursize = 0;
    (*buf).overflowed = qfalse;
    (*buf).bit = 0;
}


pub unsafe fn MSG_BeginReading(msg: *mut msg_t) {
    (*msg).readcount = 0;
    (*msg).bit = 0;
}


pub unsafe fn MSG_ReadByteAlign(buf: *mut msg_t) {
    // round up to the next byte
    if (*buf).bit != 0 {
        (*buf).bit = 0;
        (*buf).readcount += 1;
    }
}

pub unsafe fn MSG_GetSpace(buf: *mut msg_t, length: c_int) -> *mut c_void {
    let data: *mut c_void;

    // round up to the next byte
    if (*buf).bit != 0 {
        (*buf).bit = 0;
        (*buf).cursize += 1;
    }

    if (*buf).cursize + length > (*buf).maxsize {
        if (*buf).allowoverflow == qfalse {
            Com_Error(ERR_FATAL, b"MSG_GetSpace: overflow without allowoverflow set\0".as_ptr() as *const c_char);
        }
        if length > (*buf).maxsize {
            Com_Error(ERR_FATAL, b"MSG_GetSpace: %i is > full buffer size\0".as_ptr() as *const c_char, length);
        }
        Com_Printf(b"MSG_GetSpace: overflow\n\0".as_ptr() as *const c_char);
        MSG_Clear(buf);
        (*buf).overflowed = qtrue;
    }

    data = (*buf).data.add((*buf).cursize as usize) as *mut c_void;
    (*buf).cursize += length;

    data
}

pub unsafe fn MSG_WriteData(buf: *mut msg_t, data: *const c_void, length: c_int) {
    memcpy(MSG_GetSpace(buf, length), data, length as usize);
}


/*
=============================================================================

bit functions

=============================================================================
*/

pub static mut overflows: c_int = 0;

// negative bit values include signs
pub unsafe fn MSG_WriteBits(msg: *mut msg_t, mut value: c_int, mut bits: c_int) {
    let mut put: c_int;
    let mut fraction: c_int;

    // this isn't an exact overflow check, but close enough
    if (*msg).maxsize - (*msg).cursize < 4 {
        (*msg).overflowed = qtrue;
        #[cfg(not(feature = "final_build"))]
        {
            // porting note: S_COLOR_RED passed as %s arg; C source concatenated it as a string literal prefix
            Com_Printf(
                b"%sMSG_WriteBits: buffer Full writing %d in %d bits\n\0".as_ptr() as *const c_char,
                S_COLOR_RED,
                value,
                bits,
            );
        }
        return;
    }

    if bits == 0 || bits < -31 || bits > 32 {
        Com_Error(ERR_DROP, b"MSG_WriteBits: bad bits %i\0".as_ptr() as *const c_char, bits);
    }

    // check for overflows
    if bits != 32 {
        if bits > 0 {
            if value > ((1 << bits) - 1) || value < 0 {
                *addr_of_mut!(overflows) += 1;
                #[cfg(not(feature = "final_build"))]
                {
                    #[cfg(debug_assertions)]
                    {
                        Com_Printf(
                            b"%sMSG_WriteBits: overflow writing %d in %d bits\n\0".as_ptr() as *const c_char,
                            S_COLOR_RED,
                            value,
                            bits,
                        );
                    }
                }
            }
        } else {
            let r: c_int;

            r = 1 << (bits - 1);

            if value > r - 1 || value < -r {
                *addr_of_mut!(overflows) += 1;
                #[cfg(not(feature = "final_build"))]
                {
                    #[cfg(debug_assertions)]
                    {
                        Com_Printf(
                            b"%sMSG_WriteBits: overflow writing %d in %d bits\n\0".as_ptr() as *const c_char,
                            S_COLOR_RED,
                            value,
                            bits,
                        );
                    }
                }
            }
        }
    }
    if bits < 0 {
        bits = -bits;
    }

    while bits != 0 {
        if (*msg).bit == 0 {
            *(*msg).data.add((*msg).cursize as usize) = 0;
            (*msg).cursize += 1;
        }
        put = 8 - (*msg).bit;
        if put > bits {
            put = bits;
        }
        fraction = value & ((1 << put) - 1);
        *(*msg).data.add((*msg).cursize as usize - 1) |= (fraction << (*msg).bit) as u8;
        bits -= put;
        value >>= put;
        (*msg).bit = ((*msg).bit + put) & 7;
    }
}

pub unsafe fn MSG_ReadBits(msg: *mut msg_t, mut bits: c_int) -> c_int {
    let mut value: c_int;
    let mut valueBits: c_int;
    let mut get: c_int;
    let mut fraction: c_int;
    let sgn: qboolean;

    value = 0;
    valueBits = 0;

    if bits < 0 {
        bits = -bits;
        sgn = qtrue;
    } else {
        sgn = qfalse;
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
        fraction = *(*msg).data.add((*msg).readcount as usize - 1) as c_int;
        fraction >>= (*msg).bit;
        fraction &= (1 << get) - 1;
        value |= fraction << valueBits;

        valueBits += get;
        (*msg).bit = ((*msg).bit + get) & 7;
    }

    if sgn != qfalse {
        if value & (1 << (bits - 1)) != 0 {
            value |= -1 ^ ((1 << bits) - 1);
        }
    }

    value
}



//================================================================================

//
// writing functions
//

pub unsafe fn MSG_WriteByte(sb: *mut msg_t, c: c_int) {
    #[cfg(feature = "paranoid")]
    {
        if c < 0 || c > 255 {
            Com_Error(ERR_FATAL, b"MSG_WriteByte: range error\0".as_ptr() as *const c_char);
        }
    }

    MSG_WriteBits(sb, c, 8);
}

pub unsafe fn MSG_WriteShort(sb: *mut msg_t, c: c_int) {
    #[cfg(feature = "paranoid")]
    {
        if c < (0x8000u16 as i16 as c_int) || c > 0x7fffi32 {
            Com_Error(ERR_FATAL, b"MSG_WriteShort: range error\0".as_ptr() as *const c_char);
        }
    }

    MSG_WriteBits(sb, c, 16);
}

unsafe fn MSG_WriteSShort(sb: *mut msg_t, c: c_int) {
    MSG_WriteBits(sb, c, -16);
}

pub unsafe fn MSG_WriteLong(sb: *mut msg_t, c: c_int) {
    MSG_WriteBits(sb, c, 32);
}

pub unsafe fn MSG_WriteString(sb: *mut msg_t, s: *const c_char) {
    if s.is_null() {
        MSG_WriteData(sb, b"\0".as_ptr() as *const c_void, 1);
    } else {
        let l: c_int;
        let i: c_int;
        let mut string: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];

        l = strlen(s) as c_int;
        if l >= MAX_STRING_CHARS {
            Com_Printf(b"MSG_WriteString: MAX_STRING_CHARS\0".as_ptr() as *const c_char);
            MSG_WriteData(sb, b"\0".as_ptr() as *const c_void, 1);
            return;
        }
        Q_strncpyz(string.as_mut_ptr(), s, core::mem::size_of_val(&string));

        // get rid of 0xff chars, because old clients don't like them
        let mut i: c_int = 0;
        while i < l {
            if *(string.as_ptr() as *const u8).add(i as usize) > 127 {
                string[i as usize] = b'.' as c_char;
            }
            i += 1;
        }

        MSG_WriteData(sb, string.as_ptr() as *const c_void, l + 1);
    }
}



//============================================================

//
// reading functions
//

// returns -1 if no more characters are available
pub unsafe fn MSG_ReadByte(msg: *mut msg_t) -> c_int {
    let c: c_int;

    if (*msg).readcount + 1 > (*msg).cursize {
        c = -1;
    } else {
        c = MSG_ReadBits(msg, 8) as u8 as c_int;
    }

    c
}

pub unsafe fn MSG_ReadShort(msg: *mut msg_t) -> c_int {
    let c: c_int;

    if (*msg).readcount + 2 > (*msg).cursize {
        c = -1;
    } else {
        c = MSG_ReadBits(msg, 16);
    }

    c
}

unsafe fn MSG_ReadSShort(msg: *mut msg_t) -> c_int {
    let c: c_int;

    if (*msg).readcount + 2 > (*msg).cursize {
        c = -1;
    } else {
        c = MSG_ReadBits(msg, -16);
    }

    c
}

pub unsafe fn MSG_ReadLong(msg: *mut msg_t) -> c_int {
    let c: c_int;

    if (*msg).readcount + 4 > (*msg).cursize {
        c = -1;
    } else {
        c = MSG_ReadBits(msg, 32);
    }

    c
}

pub unsafe fn MSG_ReadString(msg: *mut msg_t) -> *mut c_char {
    static mut string: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];
    let mut l: c_int;
    let mut c: c_int;

    MSG_ReadByteAlign(msg);
    l = 0;
    loop {
        c = MSG_ReadByte(msg);      // use ReadByte so -1 is out of bounds
        if c == -1 || c == 0 {
            break;
        }
        // translate all fmt spec to avoid crash bugs
        if c == b'%' as c_int {
            c = b'.' as c_int;
        }

        string[l as usize] = c as c_char;
        l += 1;
        if l >= (core::mem::size_of_val(&string) - 1) as c_int {
            break;
        }
    }

    string[l as usize] = 0;

    string.as_mut_ptr()
}

pub unsafe fn MSG_ReadStringLine(msg: *mut msg_t) -> *mut c_char {
    static mut string: [c_char; MAX_STRING_CHARS as usize] = [0; MAX_STRING_CHARS as usize];
    let mut l: c_int;
    let mut c: c_int;

    MSG_ReadByteAlign(msg);
    l = 0;
    loop {
        c = MSG_ReadByte(msg);      // use ReadByte so -1 is out of bounds
        if c == -1 || c == 0 || c == b'\n' as c_int {
            break;
        }
        // translate all fmt spec to avoid crash bugs
        if c == b'%' as c_int {
            c = b'.' as c_int;
        }
        string[l as usize] = c as c_char;
        l += 1;
        if l >= (core::mem::size_of_val(&string) - 1) as c_int {
            break;
        }
    }

    string[l as usize] = 0;

    string.as_mut_ptr()
}


pub unsafe fn MSG_ReadData(msg: *mut msg_t, data: *mut c_void, len: c_int) {
    let mut i: c_int;

    MSG_ReadByteAlign(msg);
    i = 0;
    while i < len {
        *(data as *mut u8).add(i as usize) = MSG_ReadByte(msg) as u8;
        i += 1;
    }
}


/*
=============================================================================

delta functions

=============================================================================
*/

pub unsafe fn MSG_WriteDelta(msg: *mut msg_t, oldV: c_int, newV: c_int, bits: c_int) {
    if oldV == newV {
        MSG_WriteBits(msg, 0, 1);
        return;
    }
    MSG_WriteBits(msg, 1, 1);
    MSG_WriteBits(msg, newV, bits);
}

pub unsafe fn MSG_ReadDelta(msg: *mut msg_t, oldV: c_int, bits: c_int) -> c_int {
    if MSG_ReadBits(msg, 1) != 0 {
        return MSG_ReadBits(msg, bits);
    }
    oldV
}

pub unsafe fn MSG_WriteDeltaFloat(msg: *mut msg_t, oldV: c_float, newV: c_float) {
    if oldV == newV {
        MSG_WriteBits(msg, 0, 1);
        return;
    }
    MSG_WriteBits(msg, 1, 1);
    // *(int *)&newV — reinterpret float bits as int
    MSG_WriteBits(msg, newV.to_bits() as c_int, 32);
}

pub unsafe fn MSG_ReadDeltaFloat(msg: *mut msg_t, oldV: c_float) -> c_float {
    if MSG_ReadBits(msg, 1) != 0 {
        let newV: c_float;

        // *(int *)&newV = MSG_ReadBits( msg, 32 )
        newV = c_float::from_bits(MSG_ReadBits(msg, 32) as u32);
        return newV;
    }
    oldV
}


/*
============================================================================

usercmd_t communication

============================================================================
*/

// ms is allways sent, the others are optional
const CM_ANGLE1:  c_int = 1 << 0;
const CM_ANGLE2:  c_int = 1 << 1;
const CM_ANGLE3:  c_int = 1 << 2;
const CM_FORWARD: c_int = 1 << 3;
const CM_SIDE:    c_int = 1 << 4;
const CM_UP:      c_int = 1 << 5;
const CM_BUTTONS: c_int = 1 << 6;
const CM_WEAPON:  c_int = 1 << 7;

/*
=====================
MSG_WriteDeltaUsercmd
=====================
*/
pub unsafe fn MSG_WriteDeltaUsercmd(msg: *mut msg_t, from: *mut usercmd_t, to: *mut usercmd_t) {
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


/*
=====================
MSG_ReadDeltaUsercmd
=====================
*/
pub unsafe fn MSG_ReadDeltaUsercmd(msg: *mut msg_t, from: *mut usercmd_t, to: *mut usercmd_t) {
    (*to).serverTime   = MSG_ReadDelta(msg, (*from).serverTime, 32);
    (*to).angles[0]    = MSG_ReadDelta(msg, (*from).angles[0], 16);
    (*to).angles[1]    = MSG_ReadDelta(msg, (*from).angles[1], 16);
    (*to).angles[2]    = MSG_ReadDelta(msg, (*from).angles[2], 16);
    (*to).forwardmove  = MSG_ReadDelta(msg, (*from).forwardmove as c_int, -8) as i8;
    (*to).rightmove    = MSG_ReadDelta(msg, (*from).rightmove as c_int, -8) as i8;
    (*to).upmove       = MSG_ReadDelta(msg, (*from).upmove as c_int, -8) as i8;
    (*to).buttons      = MSG_ReadDelta(msg, (*from).buttons, 16);//FIXME:  We're only really using 9 bits...can this be changed to that?
    (*to).weapon       = MSG_ReadDelta(msg, (*from).weapon as c_int, 8) as u8;
    (*to).generic_cmd  = MSG_ReadDelta(msg, (*from).generic_cmd as c_int, 8) as u8;
}

/*
=============================================================================

entityState_t communication

=============================================================================
*/

// using the stringizing operator to save typing...
// #define NETF(x) #x,(int)&((entityState_t*)0)->x

// #if 0   // Removed by BTO (VV)
// const netField_t entityStateFields[] =
// {
// { NETF(eType), 8 },
// { NETF(eFlags), 32 },
//
// { NETF(pos.trType), 8 },
// { NETF(pos.trTime), 32 },
// { NETF(pos.trDuration), 32 },
// { NETF(pos.trBase[0]), 0 },
// { NETF(pos.trBase[1]), 0 },
// { NETF(pos.trBase[2]), 0 },
// { NETF(pos.trDelta[0]), 0 },
// { NETF(pos.trDelta[1]), 0 },
// { NETF(pos.trDelta[2]), 0 },
//
// { NETF(apos.trType), 8 },
// { NETF(apos.trTime), 32 },
// { NETF(apos.trDuration), 32 },
// { NETF(apos.trBase[0]), 0 },
// { NETF(apos.trBase[1]), 0 },
// { NETF(apos.trBase[2]), 0 },
// { NETF(apos.trDelta[0]), 0 },
// { NETF(apos.trDelta[1]), 0 },
// { NETF(apos.trDelta[2]), 0 },
//
// { NETF(time), 32 },
// { NETF(time2), 32 },
//
// { NETF(origin[0]), 0 },
// { NETF(origin[1]), 0 },
// { NETF(origin[2]), 0 },
//
// { NETF(origin2[0]), 0 },
// { NETF(origin2[1]), 0 },
// { NETF(origin2[2]), 0 },
//
// { NETF(angles[0]), 0 },
// { NETF(angles[1]), 0 },
// { NETF(angles[2]), 0 },
//
// { NETF(angles2[0]), 0 },
// { NETF(angles2[1]), 0 },
// { NETF(angles2[2]), 0 },
//
// { NETF(otherEntityNum), GENTITYNUM_BITS },
// //{ NETF(otherEntityNum2), GENTITYNUM_BITS },
// { NETF(groundEntityNum), GENTITYNUM_BITS },
//
// { NETF(constantLight), 32 },
// { NETF(loopSound), 16 },
// { NETF(modelindex), 9 },    //0 to 511
// { NETF(modelindex2), 8 },
// { NETF(modelindex3), 8 },
// { NETF(clientNum), 32 },
// { NETF(frame), 16 },
//
// { NETF(solid), 24 },
//
// { NETF(event), 10 },
// { NETF(eventParm), 16 },
//
// { NETF(powerups), 16 },
// { NETF(weapon), 8 },
// { NETF(legsAnim), 16 },
// { NETF(legsAnimTimer), 8 },
// { NETF(torsoAnim), 16 },
// { NETF(torsoAnimTimer), 8 },
// { NETF(scale), 8 },
//
// { NETF(saberInFlight), 4 },
// { NETF(saberActive), 4 },
// { NETF(vehicleArmor), 32 },
// { NETF(vehicleAngles[0]), 0 },
// { NETF(vehicleAngles[1]), 0 },
// { NETF(vehicleAngles[2]), 0 },
// { NETF(m_iVehicleNum), 32 },
//
// /*
// Ghoul2 Insert Start
// */
// { NETF(modelScale[0]), 0 },
// { NETF(modelScale[1]), 0 },
// { NETF(modelScale[2]), 0 },
// { NETF(radius), 16 },
// { NETF(boltInfo), 32 },
// //{ NETF(ghoul2), 32 },
//
// { NETF(isPortalEnt), 1 },
//
// };
// #endif


pub unsafe fn MSG_WriteField(msg: *mut msg_t, toF: *const c_int, field: *const netField_t) {
    let trunc: c_int;
    let fullFloat: c_float;

    if (*field).bits == -1 {
        // a -1 in the bits field means it's a float that's always between -1 and 1
        let temp: c_int = (c_float::from_bits(*toF as u32) * 32767.0f32) as c_int;
        MSG_WriteBits(msg, temp, -16);
    } else if (*field).bits == 0 {
        // float
        fullFloat = c_float::from_bits(*toF as u32);
        trunc = fullFloat as c_int;

        if fullFloat == 0.0f32 {
            MSG_WriteBits(msg, 0, 1);   //it's a zero
        } else {
            MSG_WriteBits(msg, 1, 1);   //not a zero
            if trunc as c_float == fullFloat && trunc + FLOAT_INT_BIAS >= 0 &&
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
            MSG_WriteBits(msg, 0, 1);   //it's a zero
        } else {
            MSG_WriteBits(msg, 1, 1);   //not a zero
            // integer
            MSG_WriteBits(msg, *toF, (*field).bits);
        }
    }
}

pub unsafe fn MSG_ReadField(msg: *mut msg_t, toF: *mut c_int, field: *const netField_t, print: c_int) {
    let trunc: c_int;

    if (*field).bits == -1 {
        // a -1 in the bits field means it's a float that's always between -1 and 1
        let temp: c_int = MSG_ReadBits(msg, -16);
        // *(float *)toF = (float)temp / 32767
        *toF = (temp as c_float / 32767.0f32).to_bits() as c_int;
    } else if (*field).bits == 0 {
        // float
        if MSG_ReadBits(msg, 1) == 0 {
            // *(float *)toF = 0.0f
            *toF = 0.0f32.to_bits() as c_int;
        } else {
            if MSG_ReadBits(msg, 1) == 0 {
                // integral float
                trunc = MSG_ReadBits(msg, FLOAT_INT_BITS);
                // bias to allow equal parts positive and negative
                let trunc = trunc - FLOAT_INT_BIAS;
                // *(float *)toF = trunc
                *toF = (trunc as c_float).to_bits() as c_int;
                if print != 0 {
                    Com_Printf(b"%s:%i \0".as_ptr() as *const c_char, (*field).name, trunc);
                }
            } else {
                // full floating point value
                *toF = MSG_ReadBits(msg, 32);
                if print != 0 {
                    Com_Printf(b"%s:%f \0".as_ptr() as *const c_char, (*field).name, c_float::from_bits(*toF as u32));
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


/*
==================
MSG_WriteDeltaEntity


GENTITYNUM_BITS 1 : remove this entity
GENTITYNUM_BITS 0 1 SMALL_VECTOR_BITS <data>
GENTITYNUM_BITS 0 0 LARGE_VECTOR_BITS >data>

Writes part of a packetentities message, including the entity number.
Can delta from either a baseline or a previous packet_entity
If to is NULL, a remove entity update will be sent
If force is not set, then nothing at all will be generated if the entity is
identical, under the assumption that the in-order delta code will catch it.
==================
*/
// #if 0 // Removed by BTO (VV)
// void MSG_WriteDeltaEntity( msg_t *msg, struct entityState_s *from, struct entityState_s *to,
//                            qboolean force ) {
//     int         c;
//     int         i;
//     const netField_t    *field;
//     int         *fromF, *toF;
//     int         blah;
//     bool        stuffChanged = false;
//     const int numFields = sizeof(entityStateFields)/sizeof(entityStateFields[0]);
//     byte        changeVector[(numFields/8) + 1];
//
//
//     // all fields should be 32 bits to avoid any compiler packing issues
//     // the "number" field is not part of the field list
//     // if this assert fails, someone added a field to the entityState_t
//     // struct without updating the message fields
//     blah = sizeof( *from );
//     assert( numFields + 1 == blah/4);
//
//     c = msg->cursize;
//
//     // a NULL to is a delta remove message
//     if ( to == NULL ) {
//         if ( from == NULL ) {
//             return;
//         }
//         MSG_WriteBits( msg, from->number, GENTITYNUM_BITS );
//         MSG_WriteBits( msg, 1, 1 );
//         return;
//     }
//
//     if ( to->number < 0 || to->number >= MAX_GENTITIES ) {
//         Com_Error (ERR_FATAL, "MSG_WriteDeltaEntity: Bad entity number: %i", to->number );
//     }
//
//     memset(changeVector, 0, sizeof(changeVector));
//
//     // build the change vector as bytes so it is endien independent
//     for ( i = 0, field = entityStateFields ; i < numFields ; i++, field++ ) {
//         fromF = (int *)( (byte *)from + field->offset );
//         toF = (int *)( (byte *)to + field->offset );
//         if ( *fromF != *toF ) {
//             changeVector[ i>>3 ] |= 1 << ( i & 7 );
//             stuffChanged = true;
//         }
//     }
//
//     if ( stuffChanged )
//     {
//         MSG_WriteBits( msg, to->number, GENTITYNUM_BITS );
//         MSG_WriteBits( msg, 0, 1 );         // not removed
//         MSG_WriteBits( msg, 1, 1 );         // we have a delta
//
//         // we need to write the entire delta
//         for ( i = 0 ; i + 8 <= numFields ; i += 8 ) {
//             MSG_WriteByte( msg, changeVector[i>>3] );
//         }
//         if ( numFields & 7 ) {
//             MSG_WriteBits( msg, changeVector[i>>3], numFields & 7 );
//         }
//
//         for ( i = 0, field = entityStateFields ; i < numFields ; i++, field++ ) {
//             fromF = (int *)( (byte *)from + field->offset );
//             toF = (int *)( (byte *)to + field->offset );
//
//             if ( *fromF == *toF ) {
//                 continue;
//             }
//
//             MSG_WriteField(msg, toF, field);
//         }
//     }
//     else
//     {
//         // nothing at all changed
//         // write two bits for no change
//         MSG_WriteBits( msg, to->number, GENTITYNUM_BITS );
//         MSG_WriteBits( msg, 0, 1 );     // not removed
//         MSG_WriteBits( msg, 0, 1 );     // no delta
//     }
//
//     c = msg->cursize - c;
// }
// #endif


pub unsafe fn MSG_WriteEntity(msg: *mut msg_t, to: *mut entityState_s, removeNum: c_int) {

    if to.is_null() {
        MSG_WriteBits(msg, removeNum, GENTITYNUM_BITS);
        MSG_WriteBits(msg, 1, 1); //removed
        return;
    } else {
        MSG_WriteBits(msg, (*to).number, GENTITYNUM_BITS);
        MSG_WriteBits(msg, 0, 1); //not removed
    }
    // SAFETY: svs is a valid extern global; snapshotEntities is a valid array pointer
    let svs_ptr = addr_of!(svs);
    let snap = (*svs_ptr).snapshotEntities;
    let diff = (to as *const entityState_s).offset_from(snap);
    assert!(diff >= 0 && diff < 512);
    MSG_WriteLong(msg, diff as c_int);
}

pub unsafe fn MSG_ReadEntity(msg: *mut msg_t, to: *mut entityState_t) {
    // check for a remove
    if MSG_ReadBits(msg, 1) == 1 {
        memset(to as *mut c_void, 0, core::mem::size_of::<entityState_t>());
        (*to).number = MAX_GENTITIES - 1;
        return;
    }

    //No remove, read data
    let index: c_int;
    index = MSG_ReadLong(msg);
    // SAFETY: svs is a valid extern global; index is bounds-checked below
    let svs_ptr = addr_of!(svs);
    assert!(index >= 0 && index < (*svs_ptr).numSnapshotEntities);
    *to = *(*svs_ptr).snapshotEntities.add(index as usize);
}

/*
==================
MSG_ReadDeltaEntity

The entity number has already been read from the message, which
is how the from state is identified.

If the delta removes the entity, entityState_t->number will be set to MAX_GENTITIES-1

Can go from either a baseline or a previous packet_entity
==================
*/
// (extern cvar_t *cl_shownet; -- second declaration deduplicated, see extern block above)
// #if 0 // Removed by BTO (VV)
// void MSG_ReadDeltaEntity( msg_t *msg, entityState_t *from, entityState_t *to, int number)
// {
//     int         i;
//     const netField_t    *field;
//     int         *fromF, *toF;
//     int         print = 0;
//     int         startBit, endBit;
//     const int numFields = sizeof(entityStateFields)/sizeof(entityStateFields[0]);
//     byte        expandedVector[(numFields/8) + 1];
//
//     if ( number < 0 || number >= MAX_GENTITIES) {
//         Com_Error( ERR_DROP, "Bad delta entity number: %i", number );
//     }
//
//     if ( msg->bit == 0 ) {
//         startBit = msg->readcount * 8 - GENTITYNUM_BITS;
//     } else {
//         startBit = ( msg->readcount - 1 ) * 8 + msg->bit - GENTITYNUM_BITS;
//     }
//
//     // check for a remove
//     if ( MSG_ReadBits( msg, 1 ) == 1 ) {
//         memset( to, 0, sizeof( *to ) );
//         to->number = MAX_GENTITIES - 1;
//         if ( cl_shownet->integer >= 2 || cl_shownet->integer == -1 ) {
//             Com_Printf( "%3i: #%-3i remove\n", msg->readcount, number );
//         }
//         return;
//     }
//
//     // check for no delta
//     if ( MSG_ReadBits( msg, 1 ) != 0 )
//     {
//         const int numFields = sizeof(entityStateFields)/sizeof(entityStateFields[0]);
//
//         // shownet 2/3 will interleave with other printed info, -1 will
//         // just print the delta records`
//         if ( cl_shownet->integer >= 2 || cl_shownet->integer == -1 ) {
//             print = 1;
//             Com_Printf( "%3i: #%-3i ", msg->readcount, to->number );
//         } else {
//             print = 0;
//         }
//
//         // we need to write the entire delta
//         for ( i = 0 ; i + 8 <= numFields ; i += 8 ) {
//             expandedVector[i>>3] = MSG_ReadByte( msg );
//         }
//         if ( numFields & 7 ) {
//             expandedVector[i>>3] = MSG_ReadBits( msg, numFields & 7 );
//         }
//
//         to->number = number;
//
//         for ( i = 0, field = entityStateFields ; i < numFields ; i++, field++ ) {
//             fromF = (int *)( (byte *)from + field->offset );
//             toF = (int *)( (byte *)to + field->offset );
//
//             if ( ! ( expandedVector[ i >> 3 ] & ( 1 << ( i & 7 ) ) ) ) {
//                 // no change
//                 *toF = *fromF;
//             } else {
//                 MSG_ReadField(msg, toF, field, print);
//             }
//         }
//     }
//     else
//     {
//         memcpy(to, from,sizeof(entityState_t));
//         to->number = number;
//     }
//     if ( print ) {
//         if ( msg->bit == 0 ) {
//             endBit = msg->readcount * 8 - GENTITYNUM_BITS;
//         } else {
//             endBit = ( msg->readcount - 1 ) * 8 + msg->bit - GENTITYNUM_BITS;
//         }
//         Com_Printf( " (%i bits)\n", endBit - startBit  );
//     }
// }
// #endif

/*
Ghoul2 Insert End
*/

/*
============================================================================

plyer_state_t communication

============================================================================
*/

// using the stringizing operator to save typing...
// #define PSF(x) #x,(int)&((playerState_t*)0)->x

static playerStateFields: [netField_t; 50] = [
    netField_t { name: b"commandTime\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, commandTime) as c_int,      bits: 32 },
    netField_t { name: b"pm_type\0".as_ptr() as *const c_char,           offset: core::mem::offset_of!(playerState_t, pm_type) as c_int,           bits: 8 },
    netField_t { name: b"bobCycle\0".as_ptr() as *const c_char,          offset: core::mem::offset_of!(playerState_t, bobCycle) as c_int,          bits: 8 },
    netField_t { name: b"pm_flags\0".as_ptr() as *const c_char,          offset: core::mem::offset_of!(playerState_t, pm_flags) as c_int,          bits: 32 },
    netField_t { name: b"pm_time\0".as_ptr() as *const c_char,           offset: core::mem::offset_of!(playerState_t, pm_time) as c_int,           bits: -16 },
    netField_t { name: b"origin[0]\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, origin[0]) as c_int,         bits: 0 },
    netField_t { name: b"origin[1]\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, origin[1]) as c_int,         bits: 0 },
    netField_t { name: b"origin[2]\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, origin[2]) as c_int,         bits: 0 },
    netField_t { name: b"velocity[0]\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, velocity[0]) as c_int,       bits: 0 },
    netField_t { name: b"velocity[1]\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, velocity[1]) as c_int,       bits: 0 },
    netField_t { name: b"velocity[2]\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, velocity[2]) as c_int,       bits: 0 },
    netField_t { name: b"weaponTime\0".as_ptr() as *const c_char,        offset: core::mem::offset_of!(playerState_t, weaponTime) as c_int,        bits: -16 },
    netField_t { name: b"weaponChargeTime\0".as_ptr() as *const c_char,  offset: core::mem::offset_of!(playerState_t, weaponChargeTime) as c_int,  bits: 32 }, //? really need 32 bits??
    netField_t { name: b"gravity\0".as_ptr() as *const c_char,           offset: core::mem::offset_of!(playerState_t, gravity) as c_int,           bits: 16 },
    netField_t { name: b"leanofs\0".as_ptr() as *const c_char,           offset: core::mem::offset_of!(playerState_t, leanofs) as c_int,           bits: -8 },
    netField_t { name: b"friction\0".as_ptr() as *const c_char,          offset: core::mem::offset_of!(playerState_t, friction) as c_int,          bits: 16 },
    netField_t { name: b"speed\0".as_ptr() as *const c_char,             offset: core::mem::offset_of!(playerState_t, speed) as c_int,             bits: 16 },
    netField_t { name: b"delta_angles[0]\0".as_ptr() as *const c_char,   offset: core::mem::offset_of!(playerState_t, delta_angles[0]) as c_int,   bits: 16 },
    netField_t { name: b"delta_angles[1]\0".as_ptr() as *const c_char,   offset: core::mem::offset_of!(playerState_t, delta_angles[1]) as c_int,   bits: 16 },
    netField_t { name: b"delta_angles[2]\0".as_ptr() as *const c_char,   offset: core::mem::offset_of!(playerState_t, delta_angles[2]) as c_int,   bits: 16 },
    netField_t { name: b"groundEntityNum\0".as_ptr() as *const c_char,   offset: core::mem::offset_of!(playerState_t, groundEntityNum) as c_int,   bits: GENTITYNUM_BITS },
    //{ PSF(animationTimer), 16 },
    netField_t { name: b"legsAnim\0".as_ptr() as *const c_char,          offset: core::mem::offset_of!(playerState_t, legsAnim) as c_int,          bits: 16 },
    netField_t { name: b"torsoAnim\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, torsoAnim) as c_int,         bits: 16 },
    netField_t { name: b"movementDir\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, movementDir) as c_int,       bits: 4 },
    netField_t { name: b"eFlags\0".as_ptr() as *const c_char,            offset: core::mem::offset_of!(playerState_t, eFlags) as c_int,            bits: 32 },
    netField_t { name: b"eventSequence\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, eventSequence) as c_int,     bits: 16 },
    netField_t { name: b"events[0]\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, events[0]) as c_int,         bits: 8 },
    netField_t { name: b"events[1]\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, events[1]) as c_int,         bits: 8 },
    netField_t { name: b"eventParms[0]\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, eventParms[0]) as c_int,     bits: -9 },
    netField_t { name: b"eventParms[1]\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, eventParms[1]) as c_int,     bits: -9 },
    netField_t { name: b"externalEvent\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, externalEvent) as c_int,     bits: 8 },
    netField_t { name: b"externalEventParm\0".as_ptr() as *const c_char, offset: core::mem::offset_of!(playerState_t, externalEventParm) as c_int, bits: 8 },
    netField_t { name: b"clientNum\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, clientNum) as c_int,         bits: 32 },
    netField_t { name: b"weapon\0".as_ptr() as *const c_char,            offset: core::mem::offset_of!(playerState_t, weapon) as c_int,            bits: 5 },
    netField_t { name: b"weaponstate\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, weaponstate) as c_int,       bits: 4 },
    netField_t { name: b"batteryCharge\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, batteryCharge) as c_int,     bits: 16 },
    netField_t { name: b"viewangles[0]\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, viewangles[0]) as c_int,     bits: 0 },
    netField_t { name: b"viewangles[1]\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, viewangles[1]) as c_int,     bits: 0 },
    netField_t { name: b"viewangles[2]\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, viewangles[2]) as c_int,     bits: 0 },
    netField_t { name: b"viewheight\0".as_ptr() as *const c_char,        offset: core::mem::offset_of!(playerState_t, viewheight) as c_int,        bits: -8 },
    netField_t { name: b"damageEvent\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, damageEvent) as c_int,       bits: 8 },
    netField_t { name: b"damageYaw\0".as_ptr() as *const c_char,         offset: core::mem::offset_of!(playerState_t, damageYaw) as c_int,         bits: 8 },
    netField_t { name: b"damagePitch\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, damagePitch) as c_int,       bits: -8 },
    netField_t { name: b"damageCount\0".as_ptr() as *const c_char,       offset: core::mem::offset_of!(playerState_t, damageCount) as c_int,       bits: 8 },
    //{ PSF(saberColor), 8 },
    //{ PSF(saberActive), 8 },
    //{ PSF(saberLength), 32 },
    //{ PSF(saberLengthMax), 32 },
    netField_t { name: b"forcePowersActive\0".as_ptr() as *const c_char, offset: core::mem::offset_of!(playerState_t, forcePowersActive) as c_int, bits: 32},
    netField_t { name: b"saberInFlight\0".as_ptr() as *const c_char,     offset: core::mem::offset_of!(playerState_t, saberInFlight) as c_int,     bits: 8 },

    /*{ PSF(vehicleIndex), 32 },          // WOAH, what do we do with this stuff???
    { PSF(vehicleArmor), 32 },
    { PSF(vehicleAngles[0]), 0 },
    { PSF(vehicleAngles[1]), 0 },
    { PSF(vehicleAngles[2]), 0 },*/

    netField_t { name: b"viewEntity\0".as_ptr() as *const c_char,        offset: core::mem::offset_of!(playerState_t, viewEntity) as c_int,        bits: 32 },
    netField_t { name: b"serverViewOrg[0]\0".as_ptr() as *const c_char,  offset: core::mem::offset_of!(playerState_t, serverViewOrg[0]) as c_int,  bits: 0 },
    netField_t { name: b"serverViewOrg[1]\0".as_ptr() as *const c_char,  offset: core::mem::offset_of!(playerState_t, serverViewOrg[1]) as c_int,  bits: 0 },
    netField_t { name: b"serverViewOrg[2]\0".as_ptr() as *const c_char,  offset: core::mem::offset_of!(playerState_t, serverViewOrg[2]) as c_int,  bits: 0 },
];

/*
=============
MSG_WriteDeltaPlayerstate

=============
*/
pub unsafe fn MSG_WriteDeltaPlayerstate(msg: *mut msg_t, mut from: *mut playerState_s, to: *mut playerState_s) {
    let mut i: c_int;
    // porting note: dummy pre-zeroed for soundness; C zeroes it via memset only when from is null
    let mut dummy: playerState_t = core::mem::zeroed();
    let mut statsbits: c_int;
    let mut persistantbits: c_int;
    let mut ammobits: c_int;
    let mut powerupbits: c_int;
    let numFields: c_int;
    let mut c: c_int;
    let field: *const netField_t;
    let fromF: *const c_int;
    let toF: *mut c_int;

    if from.is_null() {
        from = addr_of_mut!(dummy) as *mut playerState_s;
        memset(addr_of_mut!(dummy) as *mut c_void, 0, core::mem::size_of::<playerState_t>());
    }

    c = (*msg).cursize;

    numFields = playerStateFields.len() as c_int;
    i = 0;
    let mut field_ptr: *const netField_t = playerStateFields.as_ptr();
    while i < numFields {
        let field = &*field_ptr;
        let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
        let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;

        if *fromF == *toF {
            MSG_WriteBits(msg, 0, 1);    // no change
            i += 1;
            field_ptr = field_ptr.add(1);
            continue;
        }

        MSG_WriteBits(msg, 1, 1);    // changed
        MSG_WriteField(msg, toF, field);
        i += 1;
        field_ptr = field_ptr.add(1);
    }
    c = (*msg).cursize - c;


    //
    // send the arrays
    //
    statsbits = 0;
    i = 0;
    while i < MAX_STATS as c_int {
        if (*to).stats[i as usize] != (*from).stats[i as usize] {
            statsbits |= 1 << i;
        }
        i += 1;
    }
    if statsbits != 0 {
        MSG_WriteBits(msg, 1, 1);    // changed
        MSG_WriteShort(msg, statsbits);
        i = 0;
        while i < MAX_STATS as c_int {
            if statsbits & (1 << i) != 0 {
                MSG_WriteBits(msg, (*to).stats[i as usize], 32);
            }
            i += 1;
        }
    } else {
        MSG_WriteBits(msg, 0, 1);    // no change
    }


    persistantbits = 0;
    i = 0;
    while i < MAX_PERSISTANT as c_int {
        if (*to).persistant[i as usize] != (*from).persistant[i as usize] {
            persistantbits |= 1 << i;
        }
        i += 1;
    }
    if persistantbits != 0 {
        MSG_WriteBits(msg, 1, 1);    // changed
        MSG_WriteShort(msg, persistantbits);
        i = 0;
        while i < MAX_PERSISTANT as c_int {
            if persistantbits & (1 << i) != 0 {
                MSG_WriteSShort(msg, (*to).persistant[i as usize]);
            }
            i += 1;
        }
    } else {
        MSG_WriteBits(msg, 0, 1);    // no change
    }


    ammobits = 0;
    i = 0;
    while i < MAX_AMMO as c_int {
        if (*to).ammo[i as usize] != (*from).ammo[i as usize] {
            ammobits |= 1 << i;
        }
        i += 1;
    }
    if ammobits != 0 {
        MSG_WriteBits(msg, 1, 1);    // changed
        MSG_WriteShort(msg, ammobits);
        i = 0;
        while i < MAX_AMMO as c_int {
            if ammobits & (1 << i) != 0 {
                MSG_WriteSShort(msg, (*to).ammo[i as usize]);
            }
            i += 1;
        }
    } else {
        MSG_WriteBits(msg, 0, 1);    // no change
    }

    powerupbits = 0;
    i = 0;
    while i < MAX_POWERUPS as c_int {
        if (*to).powerups[i as usize] != (*from).powerups[i as usize] {
            powerupbits |= 1 << i;
        }
        i += 1;
    }
    if powerupbits != 0 {
        MSG_WriteBits(msg, 1, 1);    // changed
        MSG_WriteShort(msg, powerupbits);
        i = 0;
        while i < MAX_POWERUPS as c_int {
            if powerupbits & (1 << i) != 0 {
                MSG_WriteLong(msg, (*to).powerups[i as usize]);
            }
            i += 1;
        }
    } else {
        MSG_WriteBits(msg, 0, 1);    // no change
    }


    statsbits = 0;
    i = 0;
    while i < MAX_INVENTORY as c_int
    {
        if (*to).inventory[i as usize] != (*from).inventory[i as usize]
        {
            statsbits |= 1 << i;
        }
        i += 1;
    }
    if statsbits != 0
    {
        MSG_WriteBits(msg, 1, 1);    // changed
        MSG_WriteShort(msg, statsbits);
        i = 0;
        while i < MAX_INVENTORY as c_int
        {
            if statsbits & (1 << i) != 0
            {
                MSG_WriteShort(msg, (*to).inventory[i as usize]);
            }
            i += 1;
        }
    }
    else
    {
        MSG_WriteBits(msg, 0, 1);    // no change
    }
}


/*
===================
MSG_ReadDeltaPlayerstate
===================
*/
pub unsafe fn MSG_ReadDeltaPlayerstate(msg: *mut msg_t, mut from: *mut playerState_t, to: *mut playerState_t) {
    let mut i: c_int;
    let mut bits: c_int;
    let field: *const netField_t;
    let numFields: c_int;
    let startBit: c_int;
    let endBit: c_int;
    let mut print: c_int;
    let fromF: *const c_int;
    let toF: *mut c_int;
    // porting note: dummy pre-zeroed for soundness; C zeroes it via memset when from is null
    let mut dummy: playerState_t = core::mem::zeroed();

    if from.is_null() {
        from = addr_of_mut!(dummy) as *mut playerState_t;
        memset(addr_of_mut!(dummy) as *mut c_void, 0, core::mem::size_of::<playerState_t>());
    }
    *to = *from;

    if (*msg).bit == 0 {
        startBit = (*msg).readcount * 8 - GENTITYNUM_BITS;
    } else {
        startBit = ((*msg).readcount - 1) * 8 + (*msg).bit - GENTITYNUM_BITS;
    }

    // shownet 2/3 will interleave with other printed info, -2 will
    // just print the delta records
    if (*cl_shownet).integer >= 2 || (*cl_shownet).integer == -2 {
        print = 1;
        Com_Printf(b"%3i: playerstate \0".as_ptr() as *const c_char, (*msg).readcount);
    } else {
        print = 0;
    }

    numFields = playerStateFields.len() as c_int;
    i = 0;
    let mut field_ptr: *const netField_t = playerStateFields.as_ptr();
    while i < numFields {
        let field = &*field_ptr;
        let fromF = (from as *const u8).add(field.offset as usize) as *const c_int;
        let toF = (to as *mut u8).add(field.offset as usize) as *mut c_int;

        if MSG_ReadBits(msg, 1) == 0 {
            // no change
            *toF = *fromF;
        } else {
            MSG_ReadField(msg, toF, field, print);
        }
        i += 1;
        field_ptr = field_ptr.add(1);
    }

    // read the arrays

    // parse stats
    if MSG_ReadBits(msg, 1) != 0 {
        LOG!(b"PS_STATS\0".as_ptr() as *const c_char);
        bits = MSG_ReadShort(msg);
        i = 0;
        while i < MAX_STATS as c_int {
            if bits & (1 << i) != 0 {
                (*to).stats[i as usize] = MSG_ReadBits(msg, 32);
            }
            i += 1;
        }
    }

    // parse persistant stats
    if MSG_ReadBits(msg, 1) != 0 {
        LOG!(b"PS_PERSISTANT\0".as_ptr() as *const c_char);
        bits = MSG_ReadShort(msg);
        i = 0;
        while i < MAX_PERSISTANT as c_int {
            if bits & (1 << i) != 0 {
                (*to).persistant[i as usize] = MSG_ReadSShort(msg);
            }
            i += 1;
        }
    }

    // parse ammo
    if MSG_ReadBits(msg, 1) != 0 {
        LOG!(b"PS_AMMO\0".as_ptr() as *const c_char);
        bits = MSG_ReadShort(msg);
        i = 0;
        while i < MAX_AMMO as c_int {
            if bits & (1 << i) != 0 {
                (*to).ammo[i as usize] = MSG_ReadSShort(msg);
            }
            i += 1;
        }
    }

    // parse powerups
    if MSG_ReadBits(msg, 1) != 0 {
        LOG!(b"PS_POWERUPS\0".as_ptr() as *const c_char);
        bits = MSG_ReadShort(msg);
        i = 0;
        while i < MAX_POWERUPS as c_int {
            if bits & (1 << i) != 0 {
                (*to).powerups[i as usize] = MSG_ReadLong(msg);
            }
            i += 1;
        }
    }

    // parse inventory
    if MSG_ReadBits(msg, 1) != 0 {
        LOG!(b"PS_INVENTORY\0".as_ptr() as *const c_char);
        bits = MSG_ReadShort(msg);
        i = 0;
        while i < MAX_INVENTORY as c_int {
            if bits & (1 << i) != 0 {
                (*to).inventory[i as usize] = MSG_ReadShort(msg);
            }
            i += 1;
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


//===========================================================================
