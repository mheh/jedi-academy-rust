// net_wins.c
// Anything above this #include will be ignored by the compiler
// #include "../qcommon/exe_headers.h"
// #include "win_local.h"

use core::ffi::{c_int, c_char, c_void};

// ============================================================================
// External types and functions (stubs for cross-module dependencies)
// ============================================================================

#[repr(C)]
pub struct cvar_t {
    pub string: *const c_char,
    pub resetString: *const c_char,
    pub latchedString: *const c_char,
    pub name: *const c_char,
    pub documentation: *const c_char,
    pub modified: qboolean,
    pub modificationCount: c_int,
    pub value: f32,
    pub integer: c_int,
    pub resetValue: f32,
    pub resetInteger: c_int,
    pub range: f32,
    pub rangeMin: f32,
    pub rangeMax: f32,
    pub flags: c_int,
    pub next: *mut cvar_t,
}

#[repr(C)]
pub struct netadr_t {
    pub r#type: netadrtype_t,
    pub ip: [u8; 4],
    pub ipx: [u8; 10],
    pub port: u16,
}

#[repr(C)]
pub struct msg_t {
    pub allowoverflow: qboolean,
    pub overflowed: qboolean,
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
}

#[repr(C)]
pub struct sockaddr {
    pub sa_family: u16,
    pub sa_data: [c_char; 14],
}

#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_addr: in_addr,
    pub sin_zero: [c_char; 8],
}

#[repr(C)]
pub struct in_addr {
    pub S_un: in_addr_S_un,
}

#[repr(C)]
pub union in_addr_S_un {
    pub S_b: in_addr_S_b,
    pub S_w: in_addr_S_w,
    pub S_addr: u32,
}

#[repr(C)]
pub struct in_addr_S_b {
    pub s_b1: u8,
    pub s_b2: u8,
    pub s_b3: u8,
    pub s_b4: u8,
}

#[repr(C)]
pub struct in_addr_S_w {
    pub s_w1: u16,
    pub s_w2: u16,
}

#[repr(C)]
pub struct sockaddr_ipx {
    pub sa_family: u16,
    pub sa_netnum: [u8; 4],
    pub sa_nodenum: [u8; 6],
    pub sa_socket: u16,
}

#[repr(C)]
pub struct WSADATA {
    pub wVersion: u16,
    pub wHighVersion: u16,
    pub szDescription: [c_char; 257],
    pub szSystemStatus: [c_char; 129],
    pub iMaxSockets: c_int,
    pub iMaxUdpDg: c_int,
    pub lpVendorInfo: *mut c_char,
}

#[repr(C)]
pub struct hostent {
    pub h_name: *mut c_char,
    pub h_aliases: *mut *mut c_char,
    pub h_addrtype: c_int,
    pub h_length: c_int,
    pub h_addr_list: *mut *mut c_char,
}

pub type SOCKET = usize;
pub type qboolean = c_int;
pub type netadrtype_t = c_int;

// Address types
pub const NA_BOT: netadrtype_t = 0;
pub const NA_LOOPBACK: netadrtype_t = 1;
pub const NA_BROADCAST: netadrtype_t = 2;
pub const NA_IP: netadrtype_t = 3;
pub const NA_IPX: netadrtype_t = 4;
pub const NA_BROADCAST_IPX: netadrtype_t = 5;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

pub const INVALID_SOCKET: SOCKET = usize::MAX;
pub const SOCKET_ERROR: c_int = -1;

pub const AF_INET: u16 = 2;
pub const AF_IPX: u16 = 6;
pub const SOCK_DGRAM: c_int = 2;
pub const SOCK_STREAM: c_int = 1;
pub const IPPROTO_UDP: c_int = 17;
pub const IPPROTO_TCP: c_int = 6;
pub const NSPROTO_IPX: c_int = 1000;
pub const SOL_SOCKET: c_int = 0xffff;
pub const SO_BROADCAST: c_int = 32;
pub const FIONBIO: c_int = 0x8004667e;
pub const INADDR_ANY: u32 = 0x00000000;
pub const INADDR_BROADCAST: u32 = 0xffffffff;

pub const PORT_ANY: c_int = 0;
pub const PORT_SERVER: c_int = 26005;

pub const MAX_STRING_CHARS: usize = 4096;
pub const MAX_IPS: usize = 16;

pub const CVAR_ARCHIVE: c_int = 1;
pub const CVAR_LATCH: c_int = 4;

pub const ERR_FATAL: c_int = 0;

// Winsock error codes
pub const WSAEINTR: c_int = 10004;
pub const WSAEBADF: c_int = 10009;
pub const WSAEACCES: c_int = 10013;
pub const WSAEDISCON: c_int = 10101;
pub const WSAEFAULT: c_int = 10014;
pub const WSAEINVAL: c_int = 10022;
pub const WSAEMFILE: c_int = 10024;
pub const WSAEWOULDBLOCK: c_int = 10035;
pub const WSAEINPROGRESS: c_int = 10036;
pub const WSAEALREADY: c_int = 10037;
pub const WSAENOTSOCK: c_int = 10038;
pub const WSAEDESTADDRREQ: c_int = 10039;
pub const WSAEMSGSIZE: c_int = 10040;
pub const WSAEPROTOTYPE: c_int = 10041;
pub const WSAENOPROTOOPT: c_int = 10042;
pub const WSAEPROTONOSUPPORT: c_int = 10043;
pub const WSAESOCKTNOSUPPORT: c_int = 10044;
pub const WSAEOPNOTSUPP: c_int = 10045;
pub const WSAEPFNOSUPPORT: c_int = 10046;
pub const WSAEAFNOSUPPORT: c_int = 10047;
pub const WSAEADDRINUSE: c_int = 10048;
pub const WSAEADDRNOTAVAIL: c_int = 10049;
pub const WSAENETDOWN: c_int = 10050;
pub const WSAENETUNREACH: c_int = 10051;
pub const WSAENETRESET: c_int = 10052;
pub const WSAECONNABORTED: c_int = 10053;
pub const WSAECONNRESET: c_int = 10054;
pub const WSAENOBUFS: c_int = 10055;
pub const WSAEISCONN: c_int = 10056;
pub const WSAENOTCONN: c_int = 10057;
pub const WSAESHUTDOWN: c_int = 10058;
pub const WSAETOOMANYREFS: c_int = 10059;
pub const WSAETIMEDOUT: c_int = 10060;
pub const WSAECONNREFUSED: c_int = 10061;
pub const WSAELOOP: c_int = 10062;
pub const WSAENAMETOOLONG: c_int = 10063;
pub const WSAEHOSTDOWN: c_int = 10064;
pub const WSASYSNOTREADY: c_int = 11001;
pub const WSAVERNOTSUPPORTED: c_int = 11004;
pub const WSANOTINITIALISED: c_int = 11093;
pub const WSAHOST_NOT_FOUND: c_int = 11001;
pub const WSATRY_AGAIN: c_int = 11002;
pub const WSANO_RECOVERY: c_int = 11003;
pub const WSANO_DATA: c_int = 11004;
pub const WSAEHOSTUNREACH: c_int = 10065;

pub const MAKEWORD: fn(u8, u8) -> u16 = |a, b| ((b as u16) << 8) | (a as u16);

extern "C" {
    fn WSAStartup(wVersionRequested: u16, lpWSAData: *mut WSADATA) -> c_int;
    fn WSACleanup() -> c_int;
    fn WSAGetLastError() -> c_int;
    fn socket(af: c_int, r#type: c_int, protocol: c_int) -> SOCKET;
    fn closesocket(s: SOCKET) -> c_int;
    fn bind(s: SOCKET, name: *const sockaddr, namelen: c_int) -> c_int;
    fn connect(s: SOCKET, name: *const sockaddr, namelen: c_int) -> c_int;
    fn sendto(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int, to: *const sockaddr, tolen: c_int) -> c_int;
    fn recvfrom(s: SOCKET, buf: *mut c_char, len: c_int, flags: c_int, from: *mut sockaddr, fromlen: *mut c_int) -> c_int;
    fn ioctlsocket(s: SOCKET, cmd: c_int, argp: *mut c_int) -> c_int;
    fn setsockopt(s: SOCKET, level: c_int, optname: c_int, optval: *const c_char, optlen: c_int) -> c_int;
    fn send(s: SOCKET, buf: *const c_char, len: c_int, flags: c_int) -> c_int;
    fn recv(s: SOCKET, buf: *mut c_char, len: c_int, flags: c_int) -> c_int;
    fn htons(hostshort: u16) -> u16;
    fn ntohl(netlong: u32) -> u32;
    fn inet_addr(cp: *const c_char) -> u32;
    fn gethostbyname(name: *const c_char) -> *mut hostent;
    fn gethostname(name: *mut c_char, namelen: c_int) -> c_int;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;

    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_DPrintf(fmt: *const c_char, ...);
    fn Com_Error(code: c_int, fmt: *const c_char, ...);
    fn Cvar_Get(name: *const c_char, value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_SetValue(name: *const c_char, value: f32);
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn NET_AdrToString(adr: netadr_t) -> *const c_char;
}

// =============================================================================
// Global variables
// =============================================================================

static mut winsockdata: WSADATA = unsafe { core::mem::zeroed() };
static mut winsockInitialized: qboolean = qfalse;
static mut usingSocks: qboolean = qfalse;
static mut networkingEnabled: qboolean = qfalse;

static mut net_noudp: *mut cvar_t = core::ptr::null_mut();
static mut net_noipx: *mut cvar_t = core::ptr::null_mut();

static mut net_forcenonlocal: *mut cvar_t = core::ptr::null_mut();

static mut net_socksEnabled: *mut cvar_t = core::ptr::null_mut();
static mut net_socksServer: *mut cvar_t = core::ptr::null_mut();
static mut net_socksPort: *mut cvar_t = core::ptr::null_mut();
static mut net_socksUsername: *mut cvar_t = core::ptr::null_mut();
static mut net_socksPassword: *mut cvar_t = core::ptr::null_mut();
static mut socksRelayAddr: sockaddr = sockaddr {
    sa_family: 0,
    sa_data: [0; 14],
};

#[cfg(target_os = "windows")]
static mut v_socket: SOCKET = INVALID_SOCKET;
static mut ip_socket: SOCKET = INVALID_SOCKET;
static mut socks_socket: SOCKET = INVALID_SOCKET;
static mut ipx_socket: SOCKET = INVALID_SOCKET;

static mut numIP: c_int = 0;
static mut localIP: [[u8; 4]; MAX_IPS] = [[0; 4]; MAX_IPS];

// =============================================================================

/*
====================
NET_ErrorString
====================
*/
#[no_mangle]
pub extern "C" fn NET_ErrorString() -> *const c_char {
    let code: c_int = unsafe { WSAGetLastError() };
    match code {
        WSAEINTR => b"WSAEINTR\0".as_ptr() as *const c_char,
        WSAEBADF => b"WSAEBADF\0".as_ptr() as *const c_char,
        WSAEACCES => b"WSAEACCES\0".as_ptr() as *const c_char,
        WSAEDISCON => b"WSAEDISCON\0".as_ptr() as *const c_char,
        WSAEFAULT => b"WSAEFAULT\0".as_ptr() as *const c_char,
        WSAEINVAL => b"WSAEINVAL\0".as_ptr() as *const c_char,
        WSAEMFILE => b"WSAEMFILE\0".as_ptr() as *const c_char,
        WSAEWOULDBLOCK => b"WSAEWOULDBLOCK\0".as_ptr() as *const c_char,
        WSAEINPROGRESS => b"WSAEINPROGRESS\0".as_ptr() as *const c_char,
        WSAEALREADY => b"WSAEALREADY\0".as_ptr() as *const c_char,
        WSAENOTSOCK => b"WSAENOTSOCK\0".as_ptr() as *const c_char,
        WSAEDESTADDRREQ => b"WSAEDESTADDRREQ\0".as_ptr() as *const c_char,
        WSAEMSGSIZE => b"WSAEMSGSIZE\0".as_ptr() as *const c_char,
        WSAEPROTOTYPE => b"WSAEPROTOTYPE\0".as_ptr() as *const c_char,
        WSAENOPROTOOPT => b"WSAENOPROTOOPT\0".as_ptr() as *const c_char,
        WSAEPROTONOSUPPORT => b"WSAEPROTONOSUPPORT\0".as_ptr() as *const c_char,
        WSAESOCKTNOSUPPORT => b"WSAESOCKTNOSUPPORT\0".as_ptr() as *const c_char,
        WSAEOPNOTSUPP => b"WSAEOPNOTSUPP\0".as_ptr() as *const c_char,
        WSAEPFNOSUPPORT => b"WSAEPFNOSUPPORT\0".as_ptr() as *const c_char,
        WSAEAFNOSUPPORT => b"WSAEAFNOSUPPORT\0".as_ptr() as *const c_char,
        WSAEADDRINUSE => b"WSAEADDRINUSE\0".as_ptr() as *const c_char,
        WSAEADDRNOTAVAIL => b"WSAEADDRNOTAVAIL\0".as_ptr() as *const c_char,
        WSAENETDOWN => b"WSAENETDOWN\0".as_ptr() as *const c_char,
        WSAENETUNREACH => b"WSAENETUNREACH\0".as_ptr() as *const c_char,
        WSAENETRESET => b"WSAENETRESET\0".as_ptr() as *const c_char,
        WSAECONNABORTED => b"WSWSAECONNABORTEDAEINTR\0".as_ptr() as *const c_char,
        WSAECONNRESET => b"WSAECONNRESET\0".as_ptr() as *const c_char,
        WSAENOBUFS => b"WSAENOBUFS\0".as_ptr() as *const c_char,
        WSAEISCONN => b"WSAEISCONN\0".as_ptr() as *const c_char,
        WSAENOTCONN => b"WSAENOTCONN\0".as_ptr() as *const c_char,
        WSAESHUTDOWN => b"WSAESHUTDOWN\0".as_ptr() as *const c_char,
        WSAETOOMANYREFS => b"WSAETOOMANYREFS\0".as_ptr() as *const c_char,
        WSAETIMEDOUT => b"WSAETIMEDOUT\0".as_ptr() as *const c_char,
        WSAECONNREFUSED => b"WSAECONNREFUSED\0".as_ptr() as *const c_char,
        WSAELOOP => b"WSAELOOP\0".as_ptr() as *const c_char,
        WSAENAMETOOLONG => b"WSAENAMETOOLONG\0".as_ptr() as *const c_char,
        WSAEHOSTDOWN => b"WSAEHOSTDOWN\0".as_ptr() as *const c_char,
        WSASYSNOTREADY => b"WSASYSNOTREADY\0".as_ptr() as *const c_char,
        WSAVERNOTSUPPORTED => b"WSAVERNOTSUPPORTED\0".as_ptr() as *const c_char,
        WSANOTINITIALISED => b"WSANOTINITIALISED\0".as_ptr() as *const c_char,
        WSAHOST_NOT_FOUND => b"WSAHOST_NOT_FOUND\0".as_ptr() as *const c_char,
        WSATRY_AGAIN => b"WSATRY_AGAIN\0".as_ptr() as *const c_char,
        WSANO_RECOVERY => b"WSANO_RECOVERY\0".as_ptr() as *const c_char,
        WSANO_DATA => b"WSANO_DATA\0".as_ptr() as *const c_char,
        WSAEHOSTUNREACH => b"WSAEHOSTUNREACH\0".as_ptr() as *const c_char,
        _ => b"NO ERROR\0".as_ptr() as *const c_char,
    }
}

#[no_mangle]
pub extern "C" fn NetadrToSockadr(a: *mut netadr_t, s: *mut sockaddr) {
    unsafe {
        memset(s as *mut c_void, 0, core::mem::size_of::<sockaddr>());

        if (*a).r#type == NA_BROADCAST {
            let s_in = s as *mut sockaddr_in;
            (*s_in).sin_family = AF_INET;
            (*s_in).sin_port = (*a).port;
            (*s_in).sin_addr.S_un.S_addr = INADDR_BROADCAST;
        } else if (*a).r#type == NA_IP {
            let s_in = s as *mut sockaddr_in;
            (*s_in).sin_family = AF_INET;
            (*s_in).sin_addr.S_un.S_addr = *((&(*a).ip) as *const [u8; 4] as *const u32);
            (*s_in).sin_port = (*a).port;
        }
        #[cfg(not(target_os = "windows"))]
        if false {
            // Xbox-specific code path
        }
        #[cfg(not(target_os = "windows"))]
        {
            if (*a).r#type == NA_IPX {
                let s_ipx = s as *mut sockaddr_ipx;
                (*s_ipx).sa_family = AF_IPX;
                memcpy(
                    (&mut (*s_ipx).sa_netnum) as *mut [u8; 4] as *mut c_void,
                    (&(*a).ipx[0]) as *const u8 as *const c_void,
                    4,
                );
                memcpy(
                    (&mut (*s_ipx).sa_nodenum) as *mut [u8; 6] as *mut c_void,
                    (&(*a).ipx[4]) as *const u8 as *const c_void,
                    6,
                );
                (*s_ipx).sa_socket = (*a).port;
            } else if (*a).r#type == NA_BROADCAST_IPX {
                let s_ipx = s as *mut sockaddr_ipx;
                (*s_ipx).sa_family = AF_IPX;
                memset((&mut (*s_ipx).sa_netnum) as *mut [u8; 4] as *mut c_void, 0, 4);
                memset((&mut (*s_ipx).sa_nodenum) as *mut [u8; 6] as *mut c_void, 0xff, 6);
                (*s_ipx).sa_socket = (*a).port;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn SockadrToNetadr(s: *mut sockaddr, a: *mut netadr_t) {
    unsafe {
        if (*s).sa_family == AF_INET {
            (*a).r#type = NA_IP;
            *((&mut (*a).ip) as *mut [u8; 4] as *mut u32) =
                (*(s as *mut sockaddr_in)).sin_addr.S_un.S_addr;
            (*a).port = (*(s as *mut sockaddr_in)).sin_port;
        }
        #[cfg(not(target_os = "windows"))]
        {
            if (*s).sa_family == AF_IPX {
                (*a).r#type = NA_IPX;
                memcpy(
                    (&mut (*a).ipx[0]) as *mut u8 as *mut c_void,
                    (&(*(s as *mut sockaddr_ipx)).sa_netnum) as *const [u8; 4] as *const c_void,
                    4,
                );
                memcpy(
                    (&mut (*a).ipx[4]) as *mut u8 as *mut c_void,
                    (&(*(s as *mut sockaddr_ipx)).sa_nodenum) as *const [u8; 6] as *const c_void,
                    6,
                );
                (*a).port = (*(s as *mut sockaddr_ipx)).sa_socket;
            }
        }
    }
}

/*
=============
Sys_StringToAdr

idnewt
192.246.40.70
12121212.121212121212
=============
*/

#[no_mangle]
pub extern "C" fn Sys_StringToSockaddr(s: *const c_char, sadr: *mut sockaddr) -> qboolean {
    #[cfg(not(target_os = "windows"))]
    let mut h: *mut hostent = core::ptr::null_mut();
    let mut val: c_int = 0;
    let mut copy: [c_char; MAX_STRING_CHARS] = [0; MAX_STRING_CHARS];

    unsafe {
        memset(sadr as *mut c_void, 0, core::mem::size_of::<sockaddr>());

        // check for an IPX address
        if strlen(s) == 21 && *(s.add(8)) == b'.' as c_char {
            #[cfg(target_os = "windows")]
            {
                // assert(!"IPX not supported on XBox");
            }
            #[cfg(not(target_os = "windows"))]
            {
                let sadr_ipx = sadr as *mut sockaddr_ipx;
                (*sadr_ipx).sa_family = AF_IPX;
                (*sadr_ipx).sa_socket = 0;
                copy[2] = 0;

                // DO(0, sa_netnum[0]);
                copy[0] = *(s.add(0));
                copy[1] = *(s.add(1));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_netnum[0] = val as u8;

                // DO(2, sa_netnum[1]);
                copy[0] = *(s.add(2));
                copy[1] = *(s.add(3));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_netnum[1] = val as u8;

                // DO(4, sa_netnum[2]);
                copy[0] = *(s.add(4));
                copy[1] = *(s.add(5));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_netnum[2] = val as u8;

                // DO(6, sa_netnum[3]);
                copy[0] = *(s.add(6));
                copy[1] = *(s.add(7));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_netnum[3] = val as u8;

                // DO(9, sa_nodenum[0]);
                copy[0] = *(s.add(9));
                copy[1] = *(s.add(10));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_nodenum[0] = val as u8;

                // DO(11, sa_nodenum[1]);
                copy[0] = *(s.add(11));
                copy[1] = *(s.add(12));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_nodenum[1] = val as u8;

                // DO(13, sa_nodenum[2]);
                copy[0] = *(s.add(13));
                copy[1] = *(s.add(14));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_nodenum[2] = val as u8;

                // DO(15, sa_nodenum[3]);
                copy[0] = *(s.add(15));
                copy[1] = *(s.add(16));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_nodenum[3] = val as u8;

                // DO(17, sa_nodenum[4]);
                copy[0] = *(s.add(17));
                copy[1] = *(s.add(18));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_nodenum[4] = val as u8;

                // DO(19, sa_nodenum[5]);
                copy[0] = *(s.add(19));
                copy[1] = *(s.add(20));
                sscanf(copy.as_ptr(), b"%x\0".as_ptr() as *const c_char, &mut val);
                (*sadr_ipx).sa_nodenum[5] = val as u8;
            }
        } else {
            let sadr_in = sadr as *mut sockaddr_in;
            (*sadr_in).sin_family = AF_INET;
            (*sadr_in).sin_port = 0;

            if *(s) >= b'0' as c_char && *(s) <= b'9' as c_char {
                (*sadr_in).sin_addr.S_un.S_addr = inet_addr(s);
            } else {
                #[cfg(target_os = "windows")]
                {
                    // assert(!"gethostbyname() - unsupported on XBox");
                }
                #[cfg(not(target_os = "windows"))]
                {
                    h = gethostbyname(s);
                    if h.is_null() {
                        return qfalse;
                    }
                    (*sadr_in).sin_addr.S_un.S_addr =
                        *((*(*h).h_addr_list.add(0)) as *const u32);
                }
            }
        }

        return qtrue;
    }
}

/*
=============
Sys_StringToAdr

idnewt
192.246.40.70
=============
*/
#[no_mangle]
pub extern "C" fn Sys_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean {
    let mut sadr: sockaddr = sockaddr {
        sa_family: 0,
        sa_data: [0; 14],
    };

    if Sys_StringToSockaddr(s, &mut sadr) == qfalse {
        return qfalse;
    }

    SockadrToNetadr(&mut sadr, a);
    return qtrue;
}

// =============================================================================

/*
==================
Sys_GetPacket

Never called by the game logic, just the system event queing
==================
*/
static mut recvfromCount: c_int = 0;

#[no_mangle]
pub extern "C" fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean {
    let mut ret: c_int;
    let mut from: sockaddr = sockaddr {
        sa_family: 0,
        sa_data: [0; 14],
    };
    let mut fromlen: c_int;
    let mut net_socket: SOCKET;
    let mut protocol: c_int;
    let err: c_int;

    unsafe {
        for protocol in 0..2 {
            if protocol == 0 {
                net_socket = ip_socket;
            } else {
                net_socket = ipx_socket;
            }

            if net_socket == INVALID_SOCKET {
                continue;
            }

            fromlen = core::mem::size_of::<sockaddr>() as c_int;
            recvfromCount += 1; // performance check
            ret = recvfrom(
                net_socket,
                (*net_message).data as *mut c_char,
                (*net_message).maxsize,
                0,
                &mut from as *mut sockaddr,
                &mut fromlen,
            );

            if ret == SOCKET_ERROR {
                let err = WSAGetLastError();

                if err == WSAEWOULDBLOCK || err == WSAECONNRESET {
                    continue;
                }
                Com_Printf(
                    b"NET_GetPacket: %s\n\0".as_ptr() as *const c_char,
                    NET_ErrorString(),
                );
                continue;
            }

            #[cfg(target_os = "windows")]
            {
                // VVFIXME - SOF2 calls into XBL class
                // XBL_RcvdDataPacket(net_message->cursize);
            }

            if net_socket == ip_socket {
                memset(
                    (&mut (*(from as *mut sockaddr_in)).sin_zero) as *mut [c_char; 8] as *mut c_void,
                    0,
                    8,
                );
            }

            if usingSocks == qtrue
                && net_socket == ip_socket
                && memcmp(&from as *const sockaddr, &socksRelayAddr as *const sockaddr, fromlen as usize)
                    == 0
            {
                if ret < 10
                    || *((*net_message).data.add(0)) != 0
                    || *((*net_message).data.add(1)) != 0
                    || *((*net_message).data.add(2)) != 0
                    || *((*net_message).data.add(3)) != 1
                {
                    continue;
                }
                (*net_from).r#type = NA_IP;
                (*net_from).ip[0] = *((*net_message).data.add(4));
                (*net_from).ip[1] = *((*net_message).data.add(5));
                (*net_from).ip[2] = *((*net_message).data.add(6));
                (*net_from).ip[3] = *((*net_message).data.add(7));
                (*net_from).port = *((*net_message).data.add(8) as *const u16);
                (*net_message).readcount = 10;
            } else {
                SockadrToNetadr(&mut from, net_from);
                (*net_message).readcount = 0;
            }

            if ret == (*net_message).maxsize {
                Com_Printf(
                    b"Oversize packet from %s\n\0".as_ptr() as *const c_char,
                    NET_AdrToString(*net_from),
                );
                continue;
            }

            (*net_message).cursize = ret;
            return qtrue;
        }

        return qfalse;
    }
}

// =============================================================================

#[cfg(target_os = "windows")]
/*
==================
Sys_SendVoicePacket
==================
*/
#[no_mangle]
pub extern "C" fn Sys_SendVoicePacket(length: c_int, data: *const c_void, to: netadr_t) {
    let mut ret: c_int;
    let mut addr: sockaddr = sockaddr {
        sa_family: 0,
        sa_data: [0; 14],
    };

    // check for valid packet intentions (direct send or broadcast)
    //
    if to.r#type != NA_IP && to.r#type != NA_BROADCAST {
        unsafe {
            Com_Error(
                ERR_FATAL,
                b"Sys_SendVoicePacket: bad address type\0".as_ptr() as *const c_char,
            );
        }
        return;
    }

    // check we have our voice socket set up
    //
    unsafe {
        if v_socket == INVALID_SOCKET {
            return;
        }

        NetadrToSockadr(&mut (to as *const netadr_t as *mut netadr_t), &mut addr);
    }

    #[cfg(target_os = "windows")]
    unsafe {
        // #ifdef SOF2_METRICS
        // gXBL_NumberVoicePacketsSent++;
        // gXBL_SizeVoicePacketsSent += length;
        // #endif
        /*if( usingSocks && to.type == NA_IP ) {
            vsocksBuf[0] = 0;	// reserved
            vsocksBuf[1] = 0;
            vsocksBuf[2] = 0;	// fragment (not fragmented)
            vsocksBuf[3] = 1;	// address type: IPV4
            *(int *)&vsocksBuf[4] = ((struct sockaddr_in *)&addr)->sin_addr.s_addr;
            *(short *)&vsocksBuf[8] = ((struct sockaddr_in *)&addr)->sin_port;
            memcpy( &vsocksBuf[10], data, length );
            ret = sendto( v_socket, vsocksBuf, length+10, 0, &socksRelayAddr, sizeof(socksRelayAddr) );
        }
        else {*/
        ret = sendto(v_socket, data as *const c_char, length, 0, &addr as *const sockaddr, core::mem::size_of::<sockaddr>() as c_int);
        //}

        if ret == SOCKET_ERROR {
            let err = WSAGetLastError();

            // wouldblock is silent
            if err == WSAEWOULDBLOCK {
                return;
            }

            // some PPP links do not allow broadcasts and return an error
            if (err == WSAEADDRNOTAVAIL)
                && ((to.r#type == NA_BROADCAST) || (to.r#type == NA_BROADCAST_IPX))
            {
                return;
            }

            Com_DPrintf(
                b"NET_SendVoicePacket: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
        }
    }
}

static mut socksBuf: [c_char; 4096] = [0; 4096];

/*
==================
Sys_SendPacket
==================
*/
#[no_mangle]
pub extern "C" fn Sys_SendPacket(length: c_int, data: *const c_void, to: netadr_t) {
    let mut ret: c_int;
    let mut addr: sockaddr = sockaddr {
        sa_family: 0,
        sa_data: [0; 14],
    };
    let mut net_socket: SOCKET;

    unsafe {
        #[cfg(target_os = "windows")]
        {
            // VVFIXME - SOF2 calls into XBL code
            // XBL_SentDataPacket( length );
        }

        if to.r#type == NA_BROADCAST {
            net_socket = ip_socket;
        } else if to.r#type == NA_IP {
            net_socket = ip_socket;
        } else if to.r#type == NA_IPX {
            #[cfg(target_os = "windows")]
            {
                return;
            }
            #[cfg(not(target_os = "windows"))]
            {
                net_socket = ipx_socket;
            }
        } else if to.r#type == NA_BROADCAST_IPX {
            #[cfg(target_os = "windows")]
            {
                return;
            }
            #[cfg(not(target_os = "windows"))]
            {
                net_socket = ipx_socket;
            }
        } else {
            Com_Error(
                ERR_FATAL,
                b"Sys_SendPacket: bad address type\0".as_ptr() as *const c_char,
            );
            return;
        }

        if net_socket == INVALID_SOCKET {
            return;
        }

        NetadrToSockadr(&mut (to as *const netadr_t as *mut netadr_t), &mut addr);

        #[cfg(not(target_os = "windows"))]
        {
            if usingSocks == qtrue && to.r#type == NA_IP {
                socksBuf[0] = 0; // reserved
                socksBuf[1] = 0;
                socksBuf[2] = 0; // fragment (not fragmented)
                socksBuf[3] = 1; // address type: IPV4
                *((&mut socksBuf[4]) as *mut c_char as *mut u32) =
                    (*((&addr as *const sockaddr) as *const sockaddr_in)).sin_addr.S_un.S_addr;
                *((&mut socksBuf[8]) as *mut c_char as *mut u16) =
                    (*((&addr as *const sockaddr) as *const sockaddr_in)).sin_port;
                memcpy(
                    (&mut socksBuf[10]) as *mut c_char as *mut c_void,
                    data,
                    length as usize,
                );
                ret = sendto(
                    net_socket,
                    socksBuf.as_ptr(),
                    length + 10,
                    0,
                    &socksRelayAddr as *const sockaddr,
                    core::mem::size_of::<sockaddr>() as c_int,
                );
            } else {
                ret = sendto(
                    net_socket,
                    data as *const c_char,
                    length,
                    0,
                    &addr as *const sockaddr,
                    core::mem::size_of::<sockaddr>() as c_int,
                );
            }
        }
        #[cfg(target_os = "windows")]
        {
            ret = sendto(
                net_socket,
                data as *const c_char,
                length,
                0,
                &addr as *const sockaddr,
                core::mem::size_of::<sockaddr>() as c_int,
            );
        }

        if ret == SOCKET_ERROR {
            let err = WSAGetLastError();

            // wouldblock is silent
            if err == WSAEWOULDBLOCK {
                return;
            }

            // some PPP links do not allow broadcasts and return an error
            if (err == WSAEADDRNOTAVAIL)
                && ((to.r#type == NA_BROADCAST) || (to.r#type == NA_BROADCAST_IPX))
            {
                return;
            }

            Com_Printf(
                b"NET_SendPacket: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
        }
    }
}

// =============================================================================

/*
==================
Sys_IsLANAddress

LAN clients will have their rate var ignored
==================
*/
#[no_mangle]
pub extern "C" fn Sys_IsLANAddress(adr: netadr_t) -> qboolean {
    let mut i: c_int;

    unsafe {
        if net_forcenonlocal.is_null() {
            net_forcenonlocal = Cvar_Get(
                b"net_forcenonlocal\0".as_ptr() as *const c_char,
                b"0\0".as_ptr() as *const c_char,
                0,
            );
        }

        if !net_forcenonlocal.is_null() && (*net_forcenonlocal).integer != 0 {
            return qfalse;
        }

        if adr.r#type == NA_LOOPBACK {
            return qtrue;
        }

        if adr.r#type == NA_IPX {
            return qtrue;
        }

        if adr.r#type != NA_IP {
            return qfalse;
        }

        // choose which comparison to use based on the class of the address being tested
        // any local adresses of a different class than the address being tested will fail based on the first byte

        if adr.ip[0] == 127 && adr.ip[1] == 0 && adr.ip[2] == 0 && adr.ip[3] == 1 {
            return qtrue;
        }

        if (adr.ip[0] == 192 && adr.ip[1] == 168)
            || (adr.ip[0] == 10 && adr.ip[1] == 100)
            || (adr.ip[0] == 172 && adr.ip[1] == 16)
        {
            return qtrue;
        }

        /*
        // Class A
        if( (adr.ip[0] & 0x80) == 0x00 ) {
            for ( i = 0 ; i < numIP ; i++ ) {
                if( adr.ip[0] == localIP[i][0] ) {
                    return qtrue;
                }
            }
            // the RFC1918 class a block will pass the above test
            return qfalse;
        }

        // Class B
        if( (adr.ip[0] & 0xc0) == 0x80 ) {
            for ( i = 0 ; i < numIP ; i++ ) {
                if( adr.ip[0] == localIP[i][0] && adr.ip[1] == localIP[i][1] ) {
                    return qtrue;
                }
                // also check against the RFC1918 class b blocks
                if( adr.ip[0] == 172 && localIP[i][0] == 172 && (adr.ip[1] & 0xf0) == 16 && (localIP[i][1] & 0xf0) == 16 ) {
                    return qtrue;
                }
            }
            return qfalse;
        }
        */
        // we only look at class C since ISPs and Universities are using class A but we don't want to consider them on the same LAN.

        // Class C
        i = 0;
        while i < numIP {
            if adr.ip[0] == localIP[i as usize][0]
                && adr.ip[1] == localIP[i as usize][1]
                && adr.ip[2] == localIP[i as usize][2]
            {
                return qtrue;
            }

            // check for both on a local lan type thing
            if adr.ip[0] == 10 && localIP[i as usize][0] == 10 {
                return qtrue;
            }

            // also check against the RFC1918 class c blocks
            // 		if( adr.ip[0] == 192 && localIP[i][0] == 192 && adr.ip[1] == 168 && localIP[i][1] == 168 ) {
            // 			return qtrue;
            // 		}
            i += 1;
        }
        return qfalse;
    }
}

/*
==================
Sys_ShowIP
==================
*/
#[no_mangle]
pub extern "C" fn Sys_ShowIP() {
    let mut i: c_int;

    unsafe {
        i = 0;
        while i < numIP {
            Com_Printf(
                b"IP: %i.%i.%i.%i\n\0".as_ptr() as *const c_char,
                localIP[i as usize][0],
                localIP[i as usize][1],
                localIP[i as usize][2],
                localIP[i as usize][3],
            );
            i += 1;
        }
    }
}

// =============================================================================

/*
====================
NET_IPSocket
====================
*/
#[no_mangle]
pub extern "C" fn NET_IPSocket(net_interface: *const c_char, port: c_int) -> SOCKET {
    let mut newsocket: SOCKET;
    let mut address: sockaddr_in = sockaddr_in {
        sin_family: 0,
        sin_port: 0,
        sin_addr: in_addr {
            S_un: in_addr_S_un {
                S_addr: 0,
            },
        },
        sin_zero: [0; 8],
    };
    let mut _true: qboolean = qtrue;
    let mut i: c_int = 1;
    let mut err: c_int;

    unsafe {
        if !net_interface.is_null() {
            Com_Printf(
                b"Opening IP socket: %s:%i\n\0".as_ptr() as *const c_char,
                net_interface,
                port,
            );
        } else {
            Com_Printf(
                b"Opening IP socket: localhost:%i\n\0".as_ptr() as *const c_char,
                port,
            );
        }

        newsocket = socket(AF_INET as c_int, SOCK_DGRAM, IPPROTO_UDP);
        if newsocket == INVALID_SOCKET {
            err = WSAGetLastError();
            if err != WSAEAFNOSUPPORT {
                Com_Printf(
                    b"WARNING: UDP_OpenSocket: socket: %s\n\0".as_ptr() as *const c_char,
                    NET_ErrorString(),
                );
            }
            return INVALID_SOCKET;
        }

        // make it non-blocking
        if ioctlsocket(newsocket, FIONBIO, &mut (_true as *mut qboolean as *mut c_int)) == SOCKET_ERROR
        {
            Com_Printf(
                b"WARNING: UDP_OpenSocket: ioctl FIONBIO: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return INVALID_SOCKET;
        }

        // make it broadcast capable
        if setsockopt(
            newsocket,
            SOL_SOCKET,
            SO_BROADCAST,
            &i as *const c_int as *const c_char,
            core::mem::size_of::<c_int>() as c_int,
        ) == SOCKET_ERROR
        {
            Com_Printf(
                b"WARNING: UDP_OpenSocket: setsockopt SO_BROADCAST: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return INVALID_SOCKET;
        }

        if net_interface.is_null()
            || *(net_interface) == 0
            || Q_stricmp(net_interface, b"localhost\0".as_ptr() as *const c_char) == 0
        {
            address.sin_addr.S_un.S_addr = INADDR_ANY;
        } else {
            Sys_StringToSockaddr(net_interface, &mut address as *mut sockaddr_in as *mut sockaddr);
        }

        if port == PORT_ANY {
            address.sin_port = 0;
        } else {
            address.sin_port = htons(port as u16);
        }

        address.sin_family = AF_INET;

        if bind(
            newsocket,
            &address as *const sockaddr_in as *const sockaddr,
            core::mem::size_of::<sockaddr_in>() as c_int,
        ) == SOCKET_ERROR
        {
            Com_Printf(
                b"WARNING: UDP_OpenSocket: bind: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            closesocket(newsocket);
            return INVALID_SOCKET;
        }

        return newsocket;
    }
}

/*
====================
NET_OpenSocks
====================
*/
#[no_mangle]
pub extern "C" fn NET_OpenSocks(port: c_int) {
    let mut address: sockaddr_in = sockaddr_in {
        sin_family: 0,
        sin_port: 0,
        sin_addr: in_addr {
            S_un: in_addr_S_un {
                S_addr: 0,
            },
        },
        sin_zero: [0; 8],
    };
    let mut err: c_int;
    #[cfg(not(target_os = "windows"))]
    let mut h: *mut hostent;
    let mut len: c_int;
    let mut rfc1929: qboolean;
    let mut buf: [c_char; 64] = [0; 64];

    unsafe {
        usingSocks = qfalse;

        Com_Printf(b"Opening connection to SOCKS server.\n\0".as_ptr() as *const c_char);

        socks_socket = socket(AF_INET as c_int, SOCK_STREAM, IPPROTO_TCP);
        if socks_socket == INVALID_SOCKET {
            err = WSAGetLastError();
            Com_Printf(
                b"WARNING: NET_OpenSocks: socket: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return;
        }

        #[cfg(not(target_os = "windows"))]
        {
            h = gethostbyname((*net_socksServer).string);
            if h.is_null() {
                err = WSAGetLastError();
                Com_Printf(
                    b"WARNING: NET_OpenSocks: gethostbyname: %s\n\0".as_ptr() as *const c_char,
                    NET_ErrorString(),
                );
                return;
            }
            if (*h).h_addrtype != AF_INET as c_int {
                Com_Printf(b"WARNING: NET_OpenSocks: gethostbyname: address type was not AF_INET\n\0".as_ptr() as *const c_char);
                return;
            }
            address.sin_family = AF_INET;
            address.sin_addr.S_un.S_addr = *((*h).h_addr_list.add(0) as *const u32);
            address.sin_port = htons((*net_socksPort).integer as u16);
        }
        #[cfg(target_os = "windows")]
        {
            address.sin_family = AF_INET;
            address.sin_addr.S_un.S_addr = inet_addr((*net_socksServer).string);
            address.sin_port = htons((*net_socksPort).integer as u16);
        }

        if connect(
            socks_socket,
            &address as *const sockaddr_in as *const sockaddr,
            core::mem::size_of::<sockaddr_in>() as c_int,
        ) == SOCKET_ERROR
        {
            err = WSAGetLastError();
            Com_Printf(
                b"NET_OpenSocks: connect: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return;
        }

        // send socks authentication handshake
        if *((*net_socksUsername).string) != 0 || *((*net_socksPassword).string) != 0 {
            rfc1929 = qtrue;
        } else {
            rfc1929 = qfalse;
        }

        buf[0] = 5; // SOCKS version
        // method count
        if rfc1929 == qtrue {
            buf[1] = 2;
            len = 4;
        } else {
            buf[1] = 1;
            len = 3;
        }
        buf[2] = 0; // method #1 - method id #00: no authentication
        if rfc1929 == qtrue {
            buf[2] = 2; // method #2 - method id #02: username/password
        }
        if send(socks_socket, buf.as_ptr(), len, 0) == SOCKET_ERROR {
            err = WSAGetLastError();
            Com_Printf(
                b"NET_OpenSocks: send: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return;
        }

        // get the response
        len = recv(socks_socket, buf.as_mut_ptr(), 64, 0);
        if len == SOCKET_ERROR {
            err = WSAGetLastError();
            Com_Printf(
                b"NET_OpenSocks: recv: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return;
        }
        if len != 2 || buf[0] != 5 {
            Com_Printf(b"NET_OpenSocks: bad response\n\0".as_ptr() as *const c_char);
            return;
        }
        match buf[1] {
            0 => {
                // no authentication
            }
            2 => {
                // username/password authentication
            }
            _ => {
                Com_Printf(b"NET_OpenSocks: request denied\n\0".as_ptr() as *const c_char);
                return;
            }
        }

        // do username/password authentication if needed
        if buf[1] == 2 {
            let mut ulen: c_int;
            let mut plen: c_int;

            // build the request
            ulen = strlen((*net_socksUsername).string) as c_int;
            plen = strlen((*net_socksPassword).string) as c_int;

            buf[0] = 1; // username/password authentication version
            buf[1] = ulen as c_char;
            if ulen > 0 {
                memcpy(
                    (&mut buf[2]) as *mut c_char as *mut c_void,
                    (*net_socksUsername).string as *const c_void,
                    ulen as usize,
                );
            }
            buf[(2 + ulen) as usize] = plen as c_char;
            if plen > 0 {
                memcpy(
                    (&mut buf[(3 + ulen) as usize]) as *mut c_char as *mut c_void,
                    (*net_socksPassword).string as *const c_void,
                    plen as usize,
                );
            }

            // send it
            if send(socks_socket, buf.as_ptr(), 3 + ulen + plen, 0) == SOCKET_ERROR {
                err = WSAGetLastError();
                Com_Printf(
                    b"NET_OpenSocks: send: %s\n\0".as_ptr() as *const c_char,
                    NET_ErrorString(),
                );
                return;
            }

            // get the response
            len = recv(socks_socket, buf.as_mut_ptr(), 64, 0);
            if len == SOCKET_ERROR {
                err = WSAGetLastError();
                Com_Printf(
                    b"NET_OpenSocks: recv: %s\n\0".as_ptr() as *const c_char,
                    NET_ErrorString(),
                );
                return;
            }
            if len != 2 || buf[0] != 1 {
                Com_Printf(b"NET_OpenSocks: bad response\n\0".as_ptr() as *const c_char);
                return;
            }
            if buf[1] != 0 {
                Com_Printf(b"NET_OpenSocks: authentication failed\n\0".as_ptr() as *const c_char);
                return;
            }
        }

        // send the UDP associate request
        buf[0] = 5; // SOCKS version
        buf[1] = 3; // command: UDP associate
        buf[2] = 0; // reserved
        buf[3] = 1; // address type: IPV4
        *((&mut buf[4]) as *mut c_char as *mut u32) = INADDR_ANY;
        *((&mut buf[8]) as *mut c_char as *mut u16) = htons(port as u16); // port
        if send(socks_socket, buf.as_ptr(), 10, 0) == SOCKET_ERROR {
            err = WSAGetLastError();
            Com_Printf(
                b"NET_OpenSocks: send: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return;
        }

        // get the response
        len = recv(socks_socket, buf.as_mut_ptr(), 64, 0);
        if len == SOCKET_ERROR {
            err = WSAGetLastError();
            Com_Printf(
                b"NET_OpenSocks: recv: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return;
        }
        if len < 2 || buf[0] != 5 {
            Com_Printf(b"NET_OpenSocks: bad response\n\0".as_ptr() as *const c_char);
            return;
        }
        // check completion code
        if buf[1] != 0 {
            Com_Printf(
                b"NET_OpenSocks: request denied: %i\n\0".as_ptr() as *const c_char,
                buf[1] as i32,
            );
            return;
        }
        if buf[3] != 1 {
            Com_Printf(
                b"NET_OpenSocks: relay address is not IPV4: %i\n\0".as_ptr() as *const c_char,
                buf[3] as i32,
            );
            return;
        }
        ((&mut socksRelayAddr as *mut sockaddr) as *mut sockaddr_in).write(sockaddr_in {
            sin_family: AF_INET,
            sin_addr: in_addr {
                S_un: in_addr_S_un {
                    S_addr: *((&buf[4]) as *const c_char as *const u32),
                },
            },
            sin_port: *((&buf[8]) as *const c_char as *const u16),
            sin_zero: [0; 8],
        });
        memset(
            (&mut (*((&mut socksRelayAddr as *mut sockaddr) as *mut sockaddr_in)).sin_zero)
                as *mut [c_char; 8] as *mut c_void,
            0,
            8,
        );

        usingSocks = qtrue;
    }
}

/*
=====================
NET_GetLocalAddress
=====================
*/
#[no_mangle]
pub extern "C" fn NET_GetLocalAddress() {
    #[cfg(not(target_os = "windows"))]
    unsafe {
        let mut hostname: [c_char; 256] = [0; 256];
        let mut hostInfo: *mut hostent;
        let mut error: c_int;
        let mut p: *mut c_char;
        let mut ip: c_int;
        let mut n: c_int;

        // Set this early so we can just return if there is an error
        numIP = 0;

        if gethostname(hostname.as_mut_ptr(), 256) == SOCKET_ERROR {
            error = WSAGetLastError();
            return;
        }

        hostInfo = gethostbyname(hostname.as_ptr());
        if hostInfo.is_null() {
            error = WSAGetLastError();
            return;
        }

        Com_Printf(b"Hostname: %s\n\0".as_ptr() as *const c_char, (*hostInfo).h_name);
        n = 0;
        loop {
            p = *((*hostInfo).h_aliases.add(n as usize));
            n += 1;
            if p.is_null() {
                break;
            }
            Com_Printf(b"Alias: %s\n\0".as_ptr() as *const c_char, p);
        }

        if (*hostInfo).h_addrtype != AF_INET as c_int {
            return;
        }

        while numIP < MAX_IPS as c_int {
            p = *((*hostInfo).h_addr_list.add(numIP as usize));
            if p.is_null() {
                break;
            }
            ip = ntohl(*(p as *const u32));
            localIP[numIP as usize][0] = *(p.add(0)) as u8;
            localIP[numIP as usize][1] = *(p.add(1)) as u8;
            localIP[numIP as usize][2] = *(p.add(2)) as u8;
            localIP[numIP as usize][3] = *(p.add(3)) as u8;
            Com_Printf(
                b"IP: %i.%i.%i.%i\n\0".as_ptr() as *const c_char,
                (ip >> 24) & 0xff,
                (ip >> 16) & 0xff,
                (ip >> 8) & 0xff,
                ip & 0xff,
            );
            numIP += 1;
        }
    }

    #[cfg(target_os = "windows")]
    unsafe {
        // XNADDR xnMyAddr;
        // DWORD dwStatus;
        // do
        // {
        //    // Repeat while pending; OK to do other work in this loop
        //    dwStatus = XNetGetTitleXnAddr( &xnMyAddr );
        // } while( dwStatus == XNET_GET_XNADDR_PENDING );
        //
        // // Error checking
        // if( dwStatus == XNET_GET_XNADDR_NONE )
        // {
        //     assert(!"Error getting XBox title address.");
        //     return;
        // }
        //
        // *(u_long*)&localIP[0] = xnMyAddr.ina.S_un.S_addr;
        // *(u_long*)localIP[1] = 0;
        // *(u_long*)localIP[2] = 0;
        // *(u_long*)localIP[3] = 0;
        //
        // Com_Printf( "IP: %i.%i.%i.%i\n", localIP[0], localIP[1], localIP[2], localIP[3] );
    }
}

/*
====================
NET_OpenIP
====================
*/
#[no_mangle]
pub extern "C" fn NET_OpenIP() {
    let mut ip: *mut cvar_t;
    let mut port: c_int;
    let mut i: c_int;

    unsafe {
        ip = Cvar_Get(
            b"net_ip\0".as_ptr() as *const c_char,
            b"localhost\0".as_ptr() as *const c_char,
            CVAR_LATCH,
        );
        port = (*Cvar_Get(
            b"net_port\0".as_ptr() as *const c_char,
            va(b"%i\0".as_ptr() as *const c_char, PORT_SERVER),
            CVAR_LATCH,
        ))
        .integer;

        // automatically scan for a valid port, so multiple
        // dedicated servers can be started without requiring
        // a different net_port for each one
        i = 0;
        while i < 10 {
            ip_socket = NET_IPSocket((*ip).string, port + i);
            if ip_socket != INVALID_SOCKET {
                Cvar_SetValue(b"net_port\0".as_ptr() as *const c_char, (port + i) as f32);
                if (*net_socksEnabled).integer != 0 {
                    NET_OpenSocks(port + i);
                }
                NET_GetLocalAddress();
                return;
            }
            i += 1;
        }
        Com_Printf(b"WARNING: Couldn't allocate IP port\n\0".as_ptr() as *const c_char);
    }
}

/*
====================
NET_IPXSocket
====================
*/
#[no_mangle]
pub extern "C" fn NET_IPXSocket(port: c_int) -> c_int {
    #[cfg(target_os = "windows")]
    {
        // assert(!"NET_IPXSocket() - Not supported");
        return INVALID_SOCKET as c_int;
    }

    #[cfg(not(target_os = "windows"))]
    unsafe {
        let mut newsocket: SOCKET;
        let mut address: sockaddr_ipx = sockaddr_ipx {
            sa_family: 0,
            sa_netnum: [0; 4],
            sa_nodenum: [0; 6],
            sa_socket: 0,
        };
        let mut _true: c_int = 1;
        let mut err: c_int;

        newsocket = socket(AF_IPX as c_int, SOCK_DGRAM, NSPROTO_IPX);
        if newsocket == INVALID_SOCKET {
            err = WSAGetLastError();
            if err != WSAEAFNOSUPPORT {
                Com_Printf(
                    b"WARNING: IPX_Socket: socket: %s\n\0".as_ptr() as *const c_char,
                    NET_ErrorString(),
                );
            }
            return INVALID_SOCKET as c_int;
        }

        // make it non-blocking
        if ioctlsocket(newsocket, FIONBIO, &mut _true as *mut c_int) == SOCKET_ERROR {
            Com_Printf(
                b"WARNING: IPX_Socket: ioctl FIONBIO: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return INVALID_SOCKET as c_int;
        }

        // make it broadcast capable
        if setsockopt(
            newsocket,
            SOL_SOCKET,
            SO_BROADCAST,
            &_true as *const c_int as *const c_char,
            core::mem::size_of::<c_int>() as c_int,
        ) == SOCKET_ERROR
        {
            Com_Printf(
                b"WARNING: IPX_Socket: setsockopt SO_BROADCAST: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return INVALID_SOCKET as c_int;
        }

        address.sa_family = AF_IPX;
        memset(
            (&mut address.sa_netnum) as *mut [u8; 4] as *mut c_void,
            0,
            4,
        );
        memset(
            (&mut address.sa_nodenum) as *mut [u8; 6] as *mut c_void,
            0,
            6,
        );
        if port == PORT_ANY {
            address.sa_socket = 0;
        } else {
            address.sa_socket = htons(port as u16);
        }

        if bind(
            newsocket,
            &address as *const sockaddr_ipx as *const sockaddr,
            core::mem::size_of::<sockaddr_ipx>() as c_int,
        ) == SOCKET_ERROR
        {
            Com_Printf(
                b"WARNING: IPX_Socket: bind: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            closesocket(newsocket);
            return INVALID_SOCKET as c_int;
        }
        return newsocket as c_int;
    }
}

/*
====================
NET_OpenIPX
====================
*/
#[no_mangle]
pub extern "C" fn NET_OpenIPX() {
    let mut port: c_int;

    unsafe {
        port = (*Cvar_Get(
            b"net_port\0".as_ptr() as *const c_char,
            va(b"%i\0".as_ptr() as *const c_char, PORT_SERVER),
            CVAR_LATCH,
        ))
        .integer;
        ipx_socket = NET_IPXSocket(port) as usize;
    }
}

// ===================================================================

/*
====================
NET_GetCvars
====================
*/
static mut NET_GetCvars_impl: unsafe fn() -> qboolean = || {
    let mut modified: qboolean;

    modified = qfalse;

    unsafe {
        if !net_noudp.is_null() && (*net_noudp).modified != 0 {
            modified = qtrue;
        }
        net_noudp = Cvar_Get(
            b"net_noudp\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_noipx.is_null() && (*net_noipx).modified != 0 {
            modified = qtrue;
        }
        net_noipx = Cvar_Get(
            b"net_noipx\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_forcenonlocal.is_null() && (*net_forcenonlocal).modified != 0 {
            modified = qtrue;
        }
        net_forcenonlocal = Cvar_Get(
            b"net_forcenonlocal\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksEnabled.is_null() && (*net_socksEnabled).modified != 0 {
            modified = qtrue;
        }
        net_socksEnabled = Cvar_Get(
            b"net_socksEnabled\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksServer.is_null() && (*net_socksServer).modified != 0 {
            modified = qtrue;
        }
        net_socksServer = Cvar_Get(
            b"net_socksServer\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksPort.is_null() && (*net_socksPort).modified != 0 {
            modified = qtrue;
        }
        net_socksPort = Cvar_Get(
            b"net_socksPort\0".as_ptr() as *const c_char,
            b"1080\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksUsername.is_null() && (*net_socksUsername).modified != 0 {
            modified = qtrue;
        }
        net_socksUsername = Cvar_Get(
            b"net_socksUsername\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksPassword.is_null() && (*net_socksPassword).modified != 0 {
            modified = qtrue;
        }
        net_socksPassword = Cvar_Get(
            b"net_socksPassword\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        return modified;
    }
};

#[no_mangle]
pub extern "C" fn NET_GetCvars() -> qboolean {
    let mut modified: qboolean = qfalse;

    unsafe {
        if !net_noudp.is_null() && (*net_noudp).modified != 0 {
            modified = qtrue;
        }
        net_noudp = Cvar_Get(
            b"net_noudp\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_noipx.is_null() && (*net_noipx).modified != 0 {
            modified = qtrue;
        }
        net_noipx = Cvar_Get(
            b"net_noipx\0".as_ptr() as *const c_char,
            b"1\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_forcenonlocal.is_null() && (*net_forcenonlocal).modified != 0 {
            modified = qtrue;
        }
        net_forcenonlocal = Cvar_Get(
            b"net_forcenonlocal\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksEnabled.is_null() && (*net_socksEnabled).modified != 0 {
            modified = qtrue;
        }
        net_socksEnabled = Cvar_Get(
            b"net_socksEnabled\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksServer.is_null() && (*net_socksServer).modified != 0 {
            modified = qtrue;
        }
        net_socksServer = Cvar_Get(
            b"net_socksServer\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksPort.is_null() && (*net_socksPort).modified != 0 {
            modified = qtrue;
        }
        net_socksPort = Cvar_Get(
            b"net_socksPort\0".as_ptr() as *const c_char,
            b"1080\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksUsername.is_null() && (*net_socksUsername).modified != 0 {
            modified = qtrue;
        }
        net_socksUsername = Cvar_Get(
            b"net_socksUsername\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        if !net_socksPassword.is_null() && (*net_socksPassword).modified != 0 {
            modified = qtrue;
        }
        net_socksPassword = Cvar_Get(
            b"net_socksPassword\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
            CVAR_LATCH | CVAR_ARCHIVE,
        );

        return modified;
    }
}

/*
====================
NET_Config
====================
*/
#[no_mangle]
pub extern "C" fn NET_Config(mut enableNetworking: qboolean) {
    let mut modified: qboolean;
    let mut stop: qboolean;
    let mut start: qboolean;

    unsafe {
        // get any latched changes to cvars
        modified = NET_GetCvars();

        if (*net_noudp).integer != 0 && (*net_noipx).integer != 0 {
            enableNetworking = qfalse;
        }

        // if enable state is the same and no cvars were modified, we have nothing to do
        if enableNetworking == networkingEnabled && modified == qfalse {
            return;
        }

        if enableNetworking == networkingEnabled {
            if enableNetworking != 0 {
                stop = qtrue;
                start = qtrue;
            } else {
                stop = qfalse;
                start = qfalse;
            }
        } else {
            if enableNetworking != 0 {
                stop = qfalse;
                start = qtrue;
            } else {
                stop = qtrue;
                start = qfalse;
            }
            networkingEnabled = enableNetworking;
        }

        if stop != qfalse {
            if ip_socket != 0 && ip_socket != INVALID_SOCKET {
                closesocket(ip_socket);
                ip_socket = INVALID_SOCKET;
            }

            if socks_socket != 0 && socks_socket != INVALID_SOCKET {
                closesocket(socks_socket);
                socks_socket = INVALID_SOCKET;
            }

            if ipx_socket != 0 && ipx_socket != INVALID_SOCKET {
                closesocket(ipx_socket);
                ipx_socket = INVALID_SOCKET;
            }
        }

        if start != qfalse {
            if (*net_noudp).integer == 0 {
                NET_OpenIP();
            }
            #[cfg(not(target_os = "windows"))]
            {
                if (*net_noipx).integer == 0 {
                    NET_OpenIPX();
                }
            }
        }
    }
}

/*
====================
NET_Init
====================
*/
#[no_mangle]
pub extern "C" fn NET_Init() {
    let mut r: c_int;

    #[cfg(target_os = "windows")]
    unsafe {
        // Run NetStartup with security bypassed
        // this allows us to communicate with PCs while developing
        // XNetStartupParams xnsp;
        // ZeroMemory( &xnsp, sizeof(xnsp) );
        // xnsp.cfgSizeOfStruct = sizeof(xnsp);
        //
        // #ifdef _DEBUG
        // xnsp.cfgFlags |= XNET_STARTUP_BYPASS_SECURITY;
        // #else
        // xnsp.cfgFlags |= XNET_STARTUP_BYPASS_SECURITY;
        // //	xnsp.cfgFlags = 0;
        // #endif
        //
        // INT err = XNetStartup( &xnsp );
    }

    unsafe {
        r = WSAStartup(MAKEWORD(1, 1), &mut winsockdata);
        if r != 0 {
            Com_Printf(
                b"WARNING: Winsock initialization failed, returned %d\n\0".as_ptr() as *const c_char,
                r,
            );
            return;
        }

        winsockInitialized = qtrue;
        Com_Printf(b"Winsock Initialized\n\0".as_ptr() as *const c_char);

        // this is really just to get the cvars registered
        NET_GetCvars();

        // FIXME testing!
        NET_Config(qtrue);
    }
}

/*
====================
NET_Shutdown
====================
*/
#[no_mangle]
pub extern "C" fn NET_Shutdown() {
    unsafe {
        if winsockInitialized == qfalse {
            return;
        }

        NET_Config(qfalse);
        WSACleanup();
        winsockInitialized = qfalse;
    }
}

/*
====================
NET_Sleep

sleeps msec or until net socket is ready
====================
*/
#[no_mangle]
pub extern "C" fn NET_Sleep(msec: c_int) {
    // Empty implementation
}

/*
====================
NET_Restart_f
====================
*/
#[no_mangle]
pub extern "C" fn NET_Restart() {
    unsafe {
        NET_Config(networkingEnabled);
    }
}
