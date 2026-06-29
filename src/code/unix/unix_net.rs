// unix_net.c

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::mem;
use std::ptr::{addr_of, addr_of_mut};

// Type stubs for types defined in q_shared.h and qcommon.h
// These represent opaque C types that would be imported from ported headers
#[repr(C)]
pub struct netadr_t {
    pub type_: c_int,
    pub ip: [u8; 4],
    pub port: u16,
}

#[repr(C)]
pub struct msg_t {
    pub data: *mut u8,
    pub maxsize: usize,
    pub cursize: c_int,
}

#[repr(C)]
pub struct cvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub value: f32,
    pub integer: c_int,
}

pub type qboolean = c_int;

extern "C" {
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int;
    fn recvfrom(
        sockfd: c_int,
        buf: *mut c_void,
        len: usize,
        flags: c_int,
        src_addr: *mut c_void,
        addrlen: *mut c_int,
    ) -> c_int;
    fn sendto(
        sockfd: c_int,
        buf: *const c_void,
        len: usize,
        flags: c_int,
        dest_addr: *const c_void,
        addrlen: c_int,
    ) -> c_int;
    fn ioctl(fd: c_int, request: c_int, argp: *mut c_void) -> c_int;
    fn setsockopt(
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *const c_void,
        optlen: c_int,
    ) -> c_int;
    fn bind(
        sockfd: c_int,
        addr: *const c_void,
        addrlen: c_int,
    ) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn select(
        nfds: c_int,
        readfds: *mut c_void,
        writefds: *mut c_void,
        exceptfds: *mut c_void,
        timeout: *mut c_void,
    ) -> c_int;
    fn gethostname(name: *mut c_char, len: usize) -> c_int;
    fn gethostbyname(name: *const c_char) -> *mut c_void;
    fn inet_addr(cp: *const c_char) -> u32;
    fn ntohl(netlong: u32) -> u32;
    fn htons(hostshort: u16) -> u16;
    fn strerror(errnum: c_int) -> *mut c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Engine functions
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_SetValue(var_name: *const c_char, value: f32);
    fn NET_IPSocket(net_interface: *const c_char, port: c_int) -> c_int;
    fn NET_AdrToString(adr: netadr_t) -> *mut c_char;

    // Global variables from other modules
    static mut com_dedicated: cvar_t;
    static mut stdin_active: qboolean;
}

static mut NOUDP: *mut cvar_t = core::ptr::null_mut();

pub static mut NET_LOCAL_ADR: netadr_t = netadr_t {
    type_: 0,
    ip: [0; 4],
    port: 0,
};

pub static mut IP_SOCKET: c_int = 0;
pub static mut IPX_SOCKET: c_int = 0;

const MAX_IPS: usize = 16;
static mut NUM_IP: c_int = 0;
static mut LOCAL_IP: [[u8; 4]; 16] = [[0; 4]; 16];

// Constants
const NA_BROADCAST: c_int = 0;
const NA_IP: c_int = 1;
const NA_IPX: c_int = 2;
const NA_BROADCAST_IPX: c_int = 3;
const NA_LOOPBACK: c_int = 4;

const ERR_FATAL: c_int = 1;
const PORT_SERVER: c_int = 26000;
const PORT_ANY: c_int = 0;

// Socket constants from sys/socket.h
const AF_INET: c_int = 2;
const PF_INET: c_int = 2;
const SOCK_DGRAM: c_int = 2;
const IPPROTO_UDP: c_int = 17;
const SOL_SOCKET: c_int = 1;
const SO_BROADCAST: c_int = 6;
const INADDR_ANY: u32 = 0;
const FIONBIO: c_int = 0x8004667e;

// errno constants
const EWOULDBLOCK: c_int = 35;
const ECONNREFUSED: c_int = 61;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// Helper to access errno
extern "C" {
    #[link_name = "__errno_location"]
    fn errno_location() -> *mut c_int;
}

fn get_errno() -> c_int {
    unsafe { *errno_location() }
}

// Stub for va() function
extern "C" {
    fn va(fmt: *const c_char, ...) -> *mut c_char;
}

//=============================================================================

fn NetadrToSockadr(a: *const netadr_t, s: *mut c_void) {
    unsafe {
        memset(s, 0, 16); // sizeof(struct sockaddr_in)

        let s = s as *mut SockaddrIn;
        let a = &*a;

        if a.type_ == NA_BROADCAST {
            (*s).sin_family = AF_INET as u16;
            (*s).sin_port = a.port;
            *(addr_of_mut!((*s).sin_addr) as *mut c_int) = -1;
        } else if a.type_ == NA_IP {
            (*s).sin_family = AF_INET as u16;
            *(addr_of_mut!((*s).sin_addr) as *mut c_int) = *(addr_of!(a.ip[0]) as *const c_int);
            (*s).sin_port = a.port;
        }
    }
}

fn SockadrToNetadr(s: *const c_void, a: *mut netadr_t) {
    unsafe {
        let s = s as *const SockaddrIn;
        let a = &mut *a;
        *(addr_of_mut!(a.ip[0]) as *mut c_int) = *(addr_of!((*s).sin_addr) as *const c_int);
        a.port = (*s).sin_port;
        a.type_ = NA_IP;
    }
}

fn NET_BaseAdrToString(a: netadr_t) -> *mut c_char {
    unsafe {
        static mut S: [c_char; 64] = [0; 64];

        Com_sprintf(
            S.as_mut_ptr(),
            64,
            b"%i.%i.%i.%i\0".as_ptr() as *const c_char,
            a.ip[0] as c_int,
            a.ip[1] as c_int,
            a.ip[2] as c_int,
            a.ip[3] as c_int,
        );

        S.as_mut_ptr()
    }
}

/*
=============
Sys_StringToAdr

idnewt
192.246.40.70
=============
*/
fn Sys_StringToSockaddr(s: *const c_char, sadr: *mut c_void) -> qboolean {
    unsafe {
        let mut h: *mut c_void;
        let sadr = sadr as *mut SockaddrIn;

        memset(sadr as *mut c_void, 0, 16); // sizeof(struct sockaddr_in)
        (*sadr).sin_family = AF_INET as u16;

        (*sadr).sin_port = 0;

        if *s >= b'0' as c_char && *s <= b'9' as c_char {
            *(addr_of_mut!((*sadr).sin_addr) as *mut c_int) = inet_addr(s) as c_int;
        } else {
            h = gethostbyname(s);
            if h.is_null() {
                return qfalse;
            }
            let h = h as *const HostEnt;
            let addr_list = (*h).h_addr_list;
            *(addr_of_mut!((*sadr).sin_addr) as *mut c_int) = *((*addr_list) as *const c_int);
        }

        qtrue
    }
}

/*
=============
Sys_StringToAdr

localhost
idnewt
idnewt:28000
192.246.40.70
192.246.40.70:28000
=============
*/
fn Sys_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean {
    unsafe {
        let mut sadr: SockaddrIn = mem::zeroed();

        if Sys_StringToSockaddr(s, &mut sadr as *mut _ as *mut c_void) == qfalse {
            return qfalse;
        }

        SockadrToNetadr(&sadr as *const _ as *const c_void, a);

        qtrue
    }
}


//=============================================================================

fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean {
    unsafe {
        let mut ret: c_int;
        let mut from: SockaddrIn = mem::zeroed();
        let mut fromlen: c_int;
        let net_socket: c_int;
        let err: c_int;

        for protocol in 0..2 {
            if protocol == 0 {
                net_socket = IP_SOCKET;
            } else {
                net_socket = IPX_SOCKET;
            }

            if net_socket == 0 {
                continue;
            }

            fromlen = mem::size_of::<SockaddrIn>() as c_int;
            ret = recvfrom(
                net_socket,
                (*net_message).data as *mut c_void,
                (*net_message).maxsize,
                0,
                &mut from as *mut _ as *mut c_void,
                &mut fromlen,
            );

            SockadrToNetadr(&from as *const _ as *const c_void, net_from);

            if ret == -1 {
                err = get_errno();

                if err == EWOULDBLOCK || err == ECONNREFUSED {
                    continue;
                }
                Com_Printf(
                    b"NET_GetPacket: %s from %s\n\0".as_ptr() as *const c_char,
                    NET_ErrorString(),
                    NET_AdrToString(*net_from),
                );
                continue;
            }

            if ret == (*net_message).maxsize as c_int {
                Com_Printf(
                    b"Oversize packet from %s\n\0".as_ptr() as *const c_char,
                    NET_AdrToString(*net_from),
                );
                continue;
            }

            (*net_message).cursize = ret;
            return qtrue;
        }

        qfalse
    }
}

//=============================================================================

fn Sys_SendPacket(length: c_int, data: *const c_void, to: netadr_t) {
    unsafe {
        let mut ret: c_int;
        let mut addr: SockaddrIn = mem::zeroed();
        let net_socket: c_int;

        if to.type_ == NA_BROADCAST {
            net_socket = IP_SOCKET;
        } else if to.type_ == NA_IP {
            net_socket = IP_SOCKET;
        } else if to.type_ == NA_IPX {
            net_socket = IPX_SOCKET;
        } else if to.type_ == NA_BROADCAST_IPX {
            net_socket = IPX_SOCKET;
        } else {
            Com_Error(
                ERR_FATAL,
                b"NET_SendPacket: bad address type\0".as_ptr() as *const c_char,
            );
            return;
        }

        if net_socket == 0 {
            return;
        }

        NetadrToSockadr(&to, &mut addr as *mut _ as *mut c_void);

        ret = sendto(
            net_socket,
            data,
            length as usize,
            0,
            &addr as *const _ as *const c_void,
            mem::size_of::<SockaddrIn>() as c_int,
        );
        if ret == -1 {
            Com_Printf(
                b"NET_SendPacket ERROR: %s to %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
                NET_AdrToString(to),
            );
        }
    }
}


//=============================================================================

/*
==================
Sys_IsLANAddress

LAN clients will have their rate var ignored
==================
*/
fn Sys_IsLANAddress(adr: netadr_t) -> qboolean {
    unsafe {
        let mut i: c_int;
        if adr.type_ == NA_LOOPBACK {
            return qtrue;
        }

        // FIXME: ipx?

        if adr.type_ == NA_IP {
            for i_loop in 0..NUM_IP {
                i = i_loop;
                // assuming a class C network, which may not be smart...
                if adr.ip[0] == LOCAL_IP[i as usize][0]
                    && adr.ip[1] == LOCAL_IP[i as usize][1]
                    && adr.ip[2] == LOCAL_IP[i as usize][2]
                {
                    return qtrue;
                }
            }
        }

        qfalse
    }
}

/*
=====================
NET_GetLocalAddress
=====================
*/
fn NET_GetLocalAddress() {
    unsafe {
        let mut hostname: [c_char; 256] = [0; 256];
        let mut hostInfo: *mut HostEnt;
        let mut p: *mut c_char;
        let ip: c_int;
        let mut n: c_int = 0;

        if gethostname(hostname.as_mut_ptr(), 256) == -1 {
            return;
        }

        hostInfo = gethostbyname(hostname.as_ptr()) as *mut HostEnt;
        if hostInfo.is_null() {
            return;
        }

        Com_Printf(
            b"Hostname: %s\n\0".as_ptr() as *const c_char,
            (*hostInfo).h_name,
        );
        n = 0;
        loop {
            p = *(*hostInfo).h_aliases.offset(n as isize);
            n += 1;
            if p.is_null() {
                break;
            }
            Com_Printf(b"Alias: %s\n\0".as_ptr() as *const c_char, p);
        }

        if (*hostInfo).h_addrtype != AF_INET {
            return;
        }

        NUM_IP = 0;
        loop {
            p = *(*hostInfo).h_addr_list.offset(NUM_IP as isize);
            if p.is_null() || NUM_IP >= MAX_IPS as c_int {
                break;
            }
            let ip = ntohl(*(p as *const u32)) as c_int;
            LOCAL_IP[(NUM_IP) as usize][0] = *p as u8;
            LOCAL_IP[(NUM_IP) as usize][1] = *(p.offset(1)) as u8;
            LOCAL_IP[(NUM_IP) as usize][2] = *(p.offset(2)) as u8;
            LOCAL_IP[(NUM_IP) as usize][3] = *(p.offset(3)) as u8;
            Com_Printf(
                b"IP: %i.%i.%i.%i\n\0".as_ptr() as *const c_char,
                (ip >> 24) & 0xff,
                (ip >> 16) & 0xff,
                (ip >> 8) & 0xff,
                ip & 0xff,
            );
            NUM_IP += 1;
        }
    }
}

/*
====================
NET_OpenIP
====================
*/
fn NET_OpenIP() {
    unsafe {
        let ip: *mut cvar_t;
        let mut port: c_int;
        let mut i: c_int;

        ip = Cvar_Get(b"net_ip\0".as_ptr() as *const c_char, b"localhost\0".as_ptr() as *const c_char, 0);

        port = (*Cvar_Get(b"net_port\0".as_ptr() as *const c_char, va(b"%i\0".as_ptr() as *const c_char, PORT_SERVER), 0)).value as c_int;

        for i_loop in 0..10 {
            i = i_loop;
            IP_SOCKET = NET_IPSocket((*ip).string, port + i);
            if IP_SOCKET != 0 {
                Cvar_SetValue(b"net_port\0".as_ptr() as *const c_char, (port + i) as f32);
                NET_GetLocalAddress();
                return;
            }
        }
        Com_Error(
            ERR_FATAL,
            b"Couldn't allocate IP port\0".as_ptr() as *const c_char,
        );
    }
}


/*
====================
NET_Init
====================
*/
fn NET_Init() {
    unsafe {
        NOUDP = Cvar_Get(b"net_noudp\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
        // open sockets
        if (*NOUDP).value != 0.0 {
            return;
        }
        NET_OpenIP();
    }
}


/*
====================
NET_Socket
====================
*/
fn NET_IPSocket(net_interface: *const c_char, port: c_int) -> c_int {
    unsafe {
        let newsocket: c_int;
        let mut address: SockaddrIn = mem::zeroed();
        let _qtrue: qboolean = qtrue;
        let i: c_int = 1;

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

        newsocket = socket(PF_INET, SOCK_DGRAM, IPPROTO_UDP);
        if newsocket == -1 {
            Com_Printf(
                b"ERROR: UDP_OpenSocket: socket: %s\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return 0;
        }

        // make it non-blocking
        let mut ioctl_arg = _qtrue as c_int;
        if ioctl(newsocket, FIONBIO, &mut ioctl_arg as *mut c_int as *mut c_void) == -1 {
            Com_Printf(
                b"ERROR: UDP_OpenSocket: ioctl FIONBIO:%s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return 0;
        }

        // make it broadcast capable
        if setsockopt(
            newsocket,
            SOL_SOCKET,
            SO_BROADCAST,
            &i as *const c_int as *const c_void,
            mem::size_of::<c_int>() as c_int,
        ) == -1
        {
            Com_Printf(
                b"ERROR: UDP_OpenSocket: setsockopt SO_BROADCAST:%s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            return 0;
        }

        if net_interface.is_null() || *net_interface == 0 || stricmp(net_interface, b"localhost\0".as_ptr() as *const c_char) == 0 {
            address.sin_addr = INADDR_ANY;
        } else {
            Sys_StringToSockaddr(net_interface, &mut address as *mut _ as *mut c_void);
        }

        if port == PORT_ANY {
            address.sin_port = 0;
        } else {
            address.sin_port = htons(port as u16);
        }

        address.sin_family = AF_INET as u16;

        if bind(
            newsocket,
            &address as *const _ as *const c_void,
            mem::size_of::<SockaddrIn>() as c_int,
        ) == -1
        {
            Com_Printf(
                b"ERROR: UDP_OpenSocket: bind: %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
            );
            close(newsocket);
            return 0;
        }

        newsocket
    }
}

/*
====================
NET_Shutdown
====================
*/
fn NET_Shutdown() {
    unsafe {
        if IP_SOCKET != 0 {
            close(IP_SOCKET);
            IP_SOCKET = 0;
        }
    }
}


/*
====================
NET_ErrorString
====================
*/
fn NET_ErrorString() -> *mut c_char {
    unsafe {
        let code: c_int = get_errno();
        strerror(code)
    }
}

// sleeps msec or until net socket is ready
fn NET_Sleep(msec: c_int) {
    unsafe {
        let mut timeout: TimeVal = mem::zeroed();
        let mut fdset: FdSet = mem::zeroed();

        if IP_SOCKET == 0 || (*addr_of_mut!(com_dedicated)).integer == 0 {
            return; // we're not a server, just run full speed
        }

        FD_ZERO(&mut fdset);
        if stdin_active != qfalse {
            FD_SET(0, &mut fdset); // stdin is processed too
        }
        FD_SET(IP_SOCKET, &mut fdset); // network socket
        timeout.tv_sec = (msec / 1000) as i64;
        timeout.tv_usec = ((msec % 1000) * 1000) as i64;
        select(
            IP_SOCKET + 1,
            &mut fdset as *mut _ as *mut c_void,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut timeout as *mut _ as *mut c_void,
        );
    }
}

// Minimal struct definitions for system types
#[repr(C)]
struct SockaddrIn {
    sin_family: u16,
    sin_port: u16,
    sin_addr: u32,
    sin_zero: [u8; 8],
}

#[repr(C)]
struct HostEnt {
    h_name: *mut c_char,
    h_aliases: *mut *mut c_char,
    h_addrtype: c_int,
    h_length: c_int,
    h_addr_list: *mut *mut c_char,
}

#[repr(C)]
struct TimeVal {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
struct FdSet {
    fds_bits: [i64; 8],
}

fn FD_ZERO(set: &mut FdSet) {
    for i in 0..8 {
        set.fds_bits[i] = 0;
    }
}

fn FD_SET(fd: c_int, set: &mut FdSet) {
    let idx = (fd >> 6) as usize;
    let bit = 1i64 << (fd & 0x3f);
    if idx < 8 {
        set.fds_bits[idx] |= bit;
    }
}
