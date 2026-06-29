use core::ffi::c_int;

pub const FF_HANDLE_NULL: c_int = 0;
pub const FF_CLIENT_LOCAL: c_int = -2;

#[inline]
pub fn FF_CLIENT(client: c_int) -> c_int {
    FF_CLIENT_LOCAL - client
}

pub type ffHandle_t = c_int;

/*
enum FFChannel_e
{	FF_CHANNEL_WEAPON
,	FF_CHANNEL_MENU
,	FF_CHANNEL_TOUCH
,	FF_CHANNEL_DAMAGE
,	FF_CHANNEL_VEHICLE
,	FF_CHANNEL_MAX
};
*/
pub const FF_CHANNEL_WEAPON: c_int = 0;
pub const FF_CHANNEL_MENU: c_int = 1;
pub const FF_CHANNEL_TOUCH: c_int = 2;
pub const FF_CHANNEL_DAMAGE: c_int = 3;
pub const FF_CHANNEL_BODY: c_int = 4;
pub const FF_CHANNEL_FORCE: c_int = 5;
pub const FF_CHANNEL_FOOT: c_int = 6;
pub const FF_CHANNEL_MAX: c_int = 7;

/*
inline qboolean operator &= ( qboolean &lvalue, qboolean rvalue )
{
	lvalue = qboolean( (int)lvalue && (int)rvalue );
	return lvalue;
}
*/

// #ifdef _FF
// #include "../ff/ff.h"		// basic system functions
// #include "../ff/ff_snd.h"	// sound system similarities
// #endif // _FF
