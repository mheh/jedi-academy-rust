#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, c_char, c_void};
use std::ptr::{addr_of, addr_of_mut, null_mut};
use std::mem::zeroed;

// Opaque type stubs for game engine types
#[repr(C)]
pub struct cvar_t {
    _opaque: [u8; 0],
}

#[repr(C)]
pub struct netadr_t {
    pub type_: c_int,          // netadrtype_t
    pub ip: [u8; 4],
    pub ipx: [u8; 10],
    pub port: u16,
}

#[repr(C)]
pub struct msg_t {
    pub allowoverflow: c_int,
    pub overflowed: c_int,
    pub oob: c_int,
    pub data: *mut u8,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

pub type qboolean = c_int;
pub type byte = u8;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

const MAX_IPS: usize = 16;
const PORT_ANY: c_int = -1;
const PORT_SERVER: c_int = 26000;

const NA_BOT: c_int = 0;
const NA_BAD: c_int = 1;
const NA_LOOPBACK: c_int = 2;
const NA_BROADCAST: c_int = 3;
const NA_IP: c_int = 4;
const NA_IPX: c_int = 5;
const NA_BROADCAST_IPX: c_int = 6;

const AF_INET: c_int = 2;
const AF_LINK: c_int = 18;
const SOCK_DGRAM: c_int = 2;
const IPPROTO_UDP: c_int = 17;
const SOL_SOCKET: c_int = 0xffff;
const SO_BROADCAST: c_int = 0x20;
const INADDR_ANY: u32 = 0;
const FIONBIO: c_int = 0x8004667e;
const SIOCGIFCONF: c_int = 0xc0086924;
const OSIOCGIFADDR: c_int = 0xc0086921;
const IFT_LOOP: u8 = 0x18;
const EWOULDBLOCK: c_int = 35;
const ECONNREFUSED: c_int = 61;
const ERR_FATAL: c_int = 1;

const IFNAMSIZ: usize = 16;
const PF_INET: c_int = 2;

// System includes translated to extern declarations
extern "C" {
    // From sys/socket.h
    fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int;
    fn bind(sockfd: c_int, addr: *const libc::sockaddr, addrlen: libc::socklen_t) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn sendto(
        sockfd: c_int,
        buf: *const c_void,
        len: usize,
        flags: c_int,
        dest_addr: *const libc::sockaddr,
        addrlen: libc::socklen_t,
    ) -> libc::ssize_t;
    fn recvfrom(
        sockfd: c_int,
        buf: *mut c_void,
        len: usize,
        flags: c_int,
        src_addr: *mut libc::sockaddr,
        addrlen: *mut libc::socklen_t,
    ) -> libc::ssize_t;
    fn setsockopt(
        sockfd: c_int,
        level: c_int,
        optname: c_int,
        optval: *const c_void,
        optlen: libc::socklen_t,
    ) -> c_int;
    fn ioctl(fd: c_int, request: libc::c_ulong, argp: *mut c_void) -> c_int;

    // From unistd.h
    fn gethostname(name: *mut c_char, len: usize) -> c_int;

    // From netdb.h
    fn gethostbyname(name: *const c_char) -> *mut libc::hostent;

    // From arpa/inet.h
    fn inet_addr(cp: *const c_char) -> u32;
    fn ntohl(netlong: u32) -> u32;
    fn htons(hostshort: u16) -> u16;

    // From errno.h
    fn strerror(errnum: c_int) -> *mut c_char;

    // From string.h
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // Game/engine functions from qcommon and q_shared
    fn Com_Printf(fmt: *const c_char, ...) -> ();
    fn Com_sprintf(dest: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn Com_Error(code: c_int, fmt: *const c_char, ...) -> !;
    fn Cvar_Get(var_name: *const c_char, var_value: *const c_char, flags: c_int) -> *mut cvar_t;
    fn Cvar_SetValue(var_name: *const c_char, value: f32) -> ();
    fn Q_stricmp(s0: *const c_char, s1: *const c_char) -> c_int;
    fn va(format: *const c_char, ...) -> *mut c_char;

    pub static mut errno: c_int;
    pub static mut com_dedicated: cvar_t;
    pub static stdin_active: qboolean;
}

// bk001204 - prototype needed
extern "C" {
    fn NET_Socket(net_interface: *mut c_char, port: c_int) -> c_int;
    fn NET_ErrorString() -> *mut c_char;
}

static mut noudp: *mut cvar_t = null_mut();

pub static mut net_local_adr: netadr_t = unsafe { zeroed() };

pub static mut ip_socket: c_int = 0;
pub static mut ipx_socket: c_int = 0;

static mut numIP: c_int = 0;
static mut localIP: [[byte; 4]; MAX_IPS] = [[0; 4]; MAX_IPS];

//=============================================================================

#[allow(non_snake_case)]
pub unsafe fn NetadrToSockadr(a: *mut netadr_t, s: *mut libc::sockaddr_in) {
    memset(s as *mut c_void, 0, std::mem::size_of::<libc::sockaddr_in>());

    if (*a).type_ == NA_BROADCAST {
        (*s).sin_family = AF_INET as u8;

        (*s).sin_port = (*a).port;
        *(addr_of_mut!((*s).sin_addr) as *mut i32) = -1;
    } else if (*a).type_ == NA_IP {
        (*s).sin_family = AF_INET as u8;

        *(addr_of_mut!((*s).sin_addr) as *mut i32) = *(addr_of!((*a).ip) as *const i32);
        (*s).sin_port = (*a).port;
    }
}

#[allow(non_snake_case)]
pub unsafe fn SockadrToNetadr(s: *mut libc::sockaddr_in, a: *mut netadr_t) {
    *(addr_of_mut!((*a).ip) as *mut i32) = *(addr_of!((*s).sin_addr) as *const i32);
    (*a).port = (*s).sin_port;
    (*a).type_ = NA_IP;
}

#[allow(non_snake_case)]
pub unsafe fn NET_BaseAdrToString(a: netadr_t) -> *mut c_char {
    static mut s: [c_char; 64] = [0; 64];

    Com_sprintf(
        s.as_mut_ptr(),
        64,
        b"%i.%i.%i.%i\0".as_ptr() as *const c_char,
        a.ip[0] as c_int,
        a.ip[1] as c_int,
        a.ip[2] as c_int,
        a.ip[3] as c_int,
    );

    s.as_mut_ptr()
}

/*
=============
Sys_StringToAdr

idnewt
192.246.40.70
=============
*/
#[allow(non_snake_case)]
pub unsafe fn Sys_StringToSockaddr(s: *const c_char, sadr: *mut libc::sockaddr) -> qboolean {
    let mut h: *mut libc::hostent;
    // char	*colon; // bk001204 - unused

    memset(sadr, 0, std::mem::size_of::<libc::sockaddr>());
    (*(sadr as *mut libc::sockaddr_in)).sin_family = AF_INET as u8;

    (*(sadr as *mut libc::sockaddr_in)).sin_port = 0;

    if *s as u8 >= b'0' && *s as u8 <= b'9' {
        *(addr_of_mut!((*(sadr as *mut libc::sockaddr_in)).sin_addr) as *mut u32) = inet_addr(s);
    } else {
        h = gethostbyname(s);
        if h.is_null() {
            return qfalse;
        }
        *(addr_of_mut!((*(sadr as *mut libc::sockaddr_in)).sin_addr) as *mut i32) =
            *((*(*h).h_addr_list.as_ptr()) as *const i32);
    }

    qtrue
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
#[allow(non_snake_case)]
pub unsafe fn Sys_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean {
    let mut sadr: libc::sockaddr_in = zeroed();

    if Sys_StringToSockaddr(s, addr_of_mut!(sadr) as *mut libc::sockaddr) == qfalse {
        return qfalse;
    }

    SockadrToNetadr(addr_of_mut!(sadr), a);

    qtrue
}

//=============================================================================

#[allow(non_snake_case)]
pub unsafe fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean {
    let mut ret: libc::ssize_t;
    let mut from: libc::sockaddr_in = zeroed();
    let mut fromlen: libc::socklen_t = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
    let mut net_socket: c_int;
    let mut protocol: c_int;
    let mut err: c_int;

    protocol = 0;
    while protocol < 2 {
        if protocol == 0 {
            net_socket = ip_socket;
        } else {
            net_socket = ipx_socket;
        }

        if net_socket == 0 {
            protocol += 1;
            continue;
        }

        fromlen = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
        ret = recvfrom(
            net_socket,
            (*net_message).data as *mut c_void,
            (*net_message).maxsize as usize,
            0,
            addr_of_mut!(from) as *mut libc::sockaddr,
            addr_of_mut!(fromlen),
        );

        SockadrToNetadr(addr_of_mut!(from), net_from);
        // bk000305: was missing
        (*net_message).readcount = 0;

        if ret == -1 {
            err = errno;

            if err == EWOULDBLOCK || err == ECONNREFUSED {
                protocol += 1;
                continue;
            }
            Com_Printf(
                b"NET_GetPacket: %s from %s\n\0".as_ptr() as *const c_char,
                NET_ErrorString(),
                NET_AdrToString(*net_from),
            );
            protocol += 1;
            continue;
        }

        if ret == (*net_message).maxsize as libc::ssize_t {
            Com_Printf(
                b"Oversize packet from %s\n\0".as_ptr() as *const c_char,
                NET_AdrToString(*net_from),
            );
            protocol += 1;
            continue;
        }

        (*net_message).cursize = ret as c_int;
        return qtrue;
    }

    qfalse
}

//=============================================================================

#[allow(non_snake_case)]
pub unsafe fn Sys_SendPacket(length: c_int, data: *const c_void, to: netadr_t) {
    let mut ret: libc::ssize_t;
    let mut addr: libc::sockaddr_in = zeroed();
    let mut net_socket: c_int;

    if to.type_ == NA_BROADCAST {
        net_socket = ip_socket;
    } else if to.type_ == NA_IP {
        net_socket = ip_socket;
    } else if to.type_ == NA_IPX {
        net_socket = ipx_socket;
    } else if to.type_ == NA_BROADCAST_IPX {
        net_socket = ipx_socket;
    } else {
        Com_Error(ERR_FATAL, b"NET_SendPacket: bad address type\0".as_ptr() as *const c_char);
        return;
    }

    if net_socket == 0 {
        return;
    }

    NetadrToSockadr(
        addr_of_mut!(to as netadr_t),
        addr_of_mut!(addr),
    );

    ret = sendto(
        net_socket,
        data,
        length as usize,
        0,
        addr_of!(addr) as *const libc::sockaddr,
        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
    );
    if ret == -1 {
        Com_Printf(
            b"NET_SendPacket ERROR: %s to %s\n\0".as_ptr() as *const c_char,
            NET_ErrorString(),
            NET_AdrToString(to),
        );
    }
}

//=============================================================================

/*
==================
Sys_IsLANAddress

LAN clients will have their rate var ignored
==================
*/
#[allow(non_snake_case)]
pub unsafe fn Sys_IsLANAddress(adr: netadr_t) -> qboolean {
    let mut i: c_int;

    if adr.type_ == NA_LOOPBACK {
        return qtrue;
    }

    if adr.type_ == NA_IPX {
        return qtrue;
    }

    if adr.type_ != NA_IP {
        return qfalse;
    }

    // choose which comparison to use based on the class of the address being tested
    // any local adresses of a different class than the address being tested will fail based on the first byte
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
    //we only look at class C since ISPs and Universities are using class A but we don't want to consider them on the same LAN.
    // Class C
    i = 0;
    while i < numIP {
        if adr.ip[0] == localIP[i as usize][0]
            && adr.ip[1] == localIP[i as usize][1]
            && adr.ip[2] == localIP[i as usize][2]
        {
            return qtrue;
        }
        // also check against the RFC1918 class c blocks
        if adr.ip[0] == 192 && localIP[i as usize][0] == 192 && adr.ip[1] == 168
            && localIP[i as usize][1] == 168
        {
            return qtrue;
        }
        i += 1;
    }
    qfalse
}

/*
==================
Sys_ShowIP
==================
*/
#[allow(non_snake_case)]
pub unsafe fn Sys_ShowIP() {
    let mut i: c_int = 0;

    while i < numIP {
        Com_Printf(
            b"IP: %i.%i.%i.%i\n\0".as_ptr() as *const c_char,
            localIP[i as usize][0] as c_int,
            localIP[i as usize][1] as c_int,
            localIP[i as usize][2] as c_int,
            localIP[i as usize][3] as c_int,
        );
        i += 1;
    }
}

/*
=====================
NET_GetLocalAddress
=====================
*/
#[cfg(target_os = "macos")]
#[allow(non_snake_case)]
pub unsafe fn NET_GetLocalAddress() {
    // Don't do a forward mapping from the hostname of the machine to the IP.  The reason is that we might have obtained an IP address from DHCP and there might not be any name registered for the machine.  On Mac OS X, the machine name defaults to 'localhost' and NetInfo has 127.0.0.1 listed for this name.  Instead, we want to get a list of all the IP network interfaces on the machine.
    // This code adapted from OmniNetworking.

    let mut requestBuffer: [libc::ifreq; MAX_IPS] = zeroed();
    let mut ifc: libc::ifconf = zeroed();
    let mut ifr: libc::ifreq = zeroed();
    let mut sdl: *mut libc::sockaddr_dl;
    let mut interfaceSocket: c_int;
    let mut family: c_int;

    //Com_Printf("NET_GetLocalAddress: Querying for network interfaces\n");

    // Set this early so we can just return if there is an error
    numIP = 0;

    ifc.ifc_len = std::mem::size_of_val(&requestBuffer) as c_int;
    ifc.ifc_buf = requestBuffer.as_mut_ptr() as *mut c_char;

    // Since we get at this info via an ioctl, we need a temporary little socket.  This will only get AF_INET interfaces, but we probably don't care about anything else.  If we do end up caring later, we should add a ONAddressFamily and at a -interfaces method to it.
    family = AF_INET;
    if (interfaceSocket = socket(family, SOCK_DGRAM, 0)) < 0 {
        Com_Printf(
            b"NET_GetLocalAddress: Unable to create temporary socket, errno = %d\n\0".as_ptr() as *const c_char,
            errno,
        );
        return;
    }

    if ioctl(interfaceSocket, SIOCGIFCONF as libc::c_ulong, addr_of_mut!(ifc) as *mut c_void) != 0 {
        Com_Printf(
            b"NET_GetLocalAddress: Unable to get list of network interfaces, errno = %d\n\0"
                .as_ptr() as *const c_char,
            errno,
        );
        return;
    }

    let mut linkInterface: *mut libc::ifreq = ifc.ifc_buf as *mut libc::ifreq;
    while (linkInterface as *mut c_char) < ifc.ifc_buf.add(ifc.ifc_len as usize) {
        let mut nameLength: c_int;

        // The ioctl returns both the entries having the address (AF_INET) and the link layer entries (AF_LINK).  The AF_LINK entry has the link layer address which contains the interface type.  This is the only way I can see to get this information.  We cannot assume that we will get bot an AF_LINK and AF_INET entry since the interface may not be configured.  For example, if you have a 10Mb port on the motherboard and a 100Mb card, you may not configure the motherboard port.

        // For each AF_LINK entry...
        if (*linkInterface).ifr_addr.sa_family == (AF_LINK as u8) {
            // if there is a matching AF_INET entry
            let mut inetInterface: *mut libc::ifreq = ifc.ifc_buf as *mut libc::ifreq;
            while (inetInterface as *mut c_char) < ifc.ifc_buf.add(ifc.ifc_len as usize) {
                if (*inetInterface).ifr_addr.sa_family == (AF_INET as u8)
                    && strncmp(
                        (*inetInterface).ifr_name.as_ptr(),
                        (*linkInterface).ifr_name.as_ptr(),
                        std::mem::size_of_val(&(*linkInterface).ifr_name),
                    ) == 0
                {
                    nameLength = 0;
                    while nameLength < (IFNAMSIZ as c_int) {
                        if (*linkInterface).ifr_name[nameLength as usize] == 0 {
                            break;
                        }
                        nameLength += 1;
                    }

                    sdl = addr_of_mut!((*linkInterface).ifr_addr) as *mut libc::sockaddr_dl;
                    // Skip loopback interfaces
                    if (*sdl).sdl_type != IFT_LOOP {
                        // Get the local interface address
                        strncpy(
                            ifr.ifr_name.as_mut_ptr(),
                            (*inetInterface).ifr_name.as_ptr(),
                            std::mem::size_of_val(&ifr.ifr_name),
                        );
                        if ioctl(
                            interfaceSocket,
                            OSIOCGIFADDR as libc::c_ulong,
                            addr_of_mut!(ifr) as *mut c_void,
                        ) < 0
                        {
                            Com_Printf(
                                b"NET_GetLocalAddress: Unable to get local address for interface '%s', errno = %d\n\0"
                                    .as_ptr() as *const c_char,
                                (*inetInterface).ifr_name.as_ptr(),
                                errno,
                            );
                        } else {
                            let mut sin: *mut libc::sockaddr_in;
                            let mut ip: u32;

                            sin = addr_of_mut!(ifr.ifr_addr) as *mut libc::sockaddr_in;

                            ip = ntohl((*sin).sin_addr.s_addr);
                            localIP[numIP as usize][0] = ((ip >> 24) & 0xff) as byte;
                            localIP[numIP as usize][1] = ((ip >> 16) & 0xff) as byte;
                            localIP[numIP as usize][2] = ((ip >> 8) & 0xff) as byte;
                            localIP[numIP as usize][3] = ((ip >> 0) & 0xff) as byte;
                            Com_Printf(
                                b"IP: %i.%i.%i.%i (%s)\n\0".as_ptr() as *const c_char,
                                localIP[numIP as usize][0] as c_int,
                                localIP[numIP as usize][1] as c_int,
                                localIP[numIP as usize][2] as c_int,
                                localIP[numIP as usize][3] as c_int,
                                (*inetInterface).ifr_name.as_ptr(),
                            );
                            numIP += 1;
                        }
                    }

                    // We will assume that there is only one AF_INET entry per AF_LINK entry.
                    // What happens when we have an interface that has multiple IP addresses, or
                    // can that even happen?
                    // break;
                }
                inetInterface = (inetInterface as *mut c_char).add(
                    std::mem::size_of::<libc::ifreq>()
                        + std::cmp::max(
                            0,
                            (*inetInterface).ifr_addr.sa_len as c_int
                                - std::mem::size_of::<libc::sockaddr>() as c_int,
                        ) as usize,
                ) as *mut libc::ifreq;
            }
        }
        linkInterface = (linkInterface as *mut c_char).add(
            std::mem::size_of::<libc::ifreq>()
                + std::cmp::max(
                    0,
                    (*linkInterface).ifr_addr.sa_len as c_int
                        - std::mem::size_of::<libc::sockaddr>() as c_int,
                ) as usize,
        ) as *mut libc::ifreq;
    }

    close(interfaceSocket);
}

#[cfg(not(target_os = "macos"))]
#[allow(non_snake_case)]
pub unsafe fn NET_GetLocalAddress() {
    let mut hostname: [c_char; 256] = [0; 256];
    let mut hostInfo: *mut libc::hostent;
    // int					error; // bk001204 - unused
    let mut p: *mut c_char;
    let mut ip: u32;
    let mut n: c_int;

    // Set this early so we can just return if there is an error
    numIP = 0;

    if gethostname(hostname.as_mut_ptr(), 256) == -1 {
        return;
    }

    hostInfo = gethostbyname(hostname.as_ptr());
    if hostInfo.is_null() {
        return;
    }

    Com_Printf(
        b"Hostname: %s\n\0".as_ptr() as *const c_char,
        (*hostInfo).h_name,
    );
    n = 0;
    while {
        p = *(*hostInfo).h_aliases.add(n as usize);
        n += 1;
        !p.is_null()
    } {
        Com_Printf(b"Alias: %s\n\0".as_ptr() as *const c_char, p);
    }

    if (*hostInfo).h_addrtype != (AF_INET as c_int) {
        return;
    }

    while {
        p = *(*hostInfo).h_addr_list.add(numIP as usize);
        !p.is_null() && numIP < (MAX_IPS as c_int)
    } {
        ip = ntohl(*(p as *const u32));
        localIP[numIP as usize][0] = *p as byte;
        localIP[numIP as usize][1] = *p.offset(1) as byte;
        localIP[numIP as usize][2] = *p.offset(2) as byte;
        localIP[numIP as usize][3] = *p.offset(3) as byte;
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

/*
====================
NET_OpenIP
====================
*/
// bk001204 - prototype needed
extern "C" {
    fn NET_IPSocket(net_interface: *mut c_char, port: c_int) -> c_int;
}

#[allow(non_snake_case)]
pub unsafe fn NET_OpenIP() {
    let mut ip: *mut cvar_t;
    let mut port: c_int;
    let mut i: c_int;

    ip = Cvar_Get(b"net_ip\0".as_ptr() as *const c_char, b"localhost\0".as_ptr() as *const c_char, 0);

    port = (*Cvar_Get(
        b"net_port\0".as_ptr() as *const c_char,
        va(b"%i\0".as_ptr() as *const c_char, PORT_SERVER),
        0,
    ))
    .value as c_int;

    i = 0;
    while i < 10 {
        ip_socket = NET_IPSocket((*ip).string as *mut c_char, port + i);
        if ip_socket != 0 {
            Cvar_SetValue(b"net_port\0".as_ptr() as *const c_char, (port + i) as f32);
            NET_GetLocalAddress();
            return;
        }
        i += 1;
    }
    Com_Error(ERR_FATAL, b"Couldn\'t allocate IP port\0".as_ptr() as *const c_char);
}

/*
====================
NET_Init
====================
*/
#[allow(non_snake_case)]
pub unsafe fn NET_Init() {
    noudp = Cvar_Get(b"net_noudp\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char, 0);
    // open sockets
    if (*noudp).value == 0.0 {
        NET_OpenIP();
    }
}

/*
====================
NET_IPSocket
====================
*/
#[allow(non_snake_case)]
pub unsafe fn NET_IPSocket(net_interface: *mut c_char, port: c_int) -> c_int {
    let mut newsocket: c_int;
    let mut address: libc::sockaddr_in = zeroed();
    let mut _qtrue: qboolean = qtrue;
    let mut i: c_int = 1;

    if !net_interface.is_null() {
        Com_Printf(
            b"Opening IP socket: %s:%i\n\0".as_ptr() as *const c_char,
            net_interface,
            port,
        );
    } else {
        Com_Printf(b"Opening IP socket: localhost:%i\n\0".as_ptr() as *const c_char, port);
    }

    if (newsocket = socket(PF_INET, SOCK_DGRAM, IPPROTO_UDP)) == -1 {
        Com_Printf(
            b"ERROR: UDP_OpenSocket: socket: %s\0".as_ptr() as *const c_char,
            NET_ErrorString(),
        );
        return 0;
    }

    // make it non-blocking
    if ioctl(newsocket, FIONBIO as libc::c_ulong, addr_of_mut!(_qtrue) as *mut c_void) == -1 {
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
        addr_of!(i) as *const c_void,
        std::mem::size_of::<c_int>() as libc::socklen_t,
    ) == -1
    {
        Com_Printf(
            b"ERROR: UDP_OpenSocket: setsockopt SO_BROADCAST:%s\n\0".as_ptr() as *const c_char,
            NET_ErrorString(),
        );
        return 0;
    }

    if net_interface.is_null() || *net_interface == 0 || Q_stricmp(net_interface, b"localhost\0".as_ptr() as *const c_char) == 0 {
        address.sin_addr.s_addr = INADDR_ANY;
    } else {
        Sys_StringToSockaddr(net_interface as *const c_char, addr_of_mut!(address) as *mut libc::sockaddr);
    }

    if port == PORT_ANY {
        address.sin_port = 0;
    } else {
        address.sin_port = htons(port as u16);
    }

    address.sin_family = AF_INET as u8;

    if bind(
        newsocket,
        addr_of!(address) as *const libc::sockaddr,
        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
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

/*
====================
NET_Shutdown
====================
*/
#[allow(non_snake_case)]
pub unsafe fn NET_Shutdown() {
    if ip_socket != 0 {
        close(ip_socket);
        ip_socket = 0;
    }
}

/*
====================
NET_ErrorString
====================
*/
#[allow(non_snake_case)]
pub unsafe fn NET_ErrorString() -> *mut c_char {
    let mut code: c_int;

    code = errno;
    strerror(code)
}

// sleeps msec or until net socket is ready
#[allow(non_snake_case)]
pub unsafe fn NET_Sleep(msec: c_int) {
    let mut timeout: libc::timeval = zeroed();
    let mut fdset: libc::fd_set = zeroed();

    if ip_socket == 0 || com_dedicated.value == 0.0 {
        return; // we're not a server, just run full speed
    }

    libc::FD_ZERO(addr_of_mut!(fdset));
    if stdin_active != 0 {
        libc::FD_SET(0, addr_of_mut!(fdset)); // stdin is processed too
    }
    libc::FD_SET(ip_socket, addr_of_mut!(fdset)); // network socket
    timeout.tv_sec = (msec / 1000) as libc::time_t;
    timeout.tv_usec = ((msec % 1000) * 1000) as libc::suseconds_t;
    libc::select(ip_socket + 1, addr_of_mut!(fdset), null_mut(), null_mut(), addr_of_mut!(timeout));
}
