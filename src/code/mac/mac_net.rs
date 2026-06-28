#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// ============================================================================
// Includes (from original C headers)
// ============================================================================
// #include "../client/client.h"
// #include "mac_local.h"
// #include <OpenTransport.h>
// #include <OpenTptInternet.h>

use core::ffi::{c_char, c_int, c_void};

// ============================================================================
// Type definitions
// ============================================================================

pub type byte = u8;
pub type qboolean = c_int;

pub const qtrue: qboolean = 1;
pub const qfalse: qboolean = 0;

// Network address types from codemp/qcommon/qcommon.h
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum netadrtype_t {
    NA_BAD,
    NA_LOOPBACK,
    NA_BROADCAST,
    NA_IP,
    NA_IPX,
    NA_BROADCAST_IPX,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct netadr_t {
    pub r#type: netadrtype_t,
    pub ip: [byte; 4],
    pub ipx: [byte; 10],
    pub port: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct msg_t {
    pub allowoverflow: qboolean,
    pub overflowed: qboolean,
    pub data: *mut byte,
    pub maxsize: c_int,
    pub cursize: c_int,
    pub readcount: c_int,
    pub bit: c_int,
}

// OpenTransport types - macOS-specific (deprecated API)
pub type EndpointRef = *mut c_void;
pub type OSStatus = c_int;
pub type OTResult = c_int;
pub type OTXTILevel = c_int;
pub type OTXTIName = c_int;
pub type OTEventCode = c_int;
pub type OTFlags = c_int;

pub const kOTInvalidEndpointRef: EndpointRef = std::ptr::null_mut();
pub const noErr: OTResult = 0;
pub const kOTNoDataErr: OTResult = -3170;

pub const T_OPENCOMPLETE: OTEventCode = 0;
pub const T_UDERR: OTEventCode = 1;
pub const T_SUCCESS: c_int = 0;
pub const T_READONLY: c_int = 1;
pub const T_CURRENT: c_int = 0;
pub const T_NEGOTIATE: c_int = 1;
pub const T_YES: c_int = 1;

pub const INET_IP: OTXTILevel = 0;
pub const IP_BROADCAST: OTXTIName = 1;

pub const kOTFourByteOptionSize: usize = 12;
pub const kUDPName: *const c_char = b"udp\0".as_ptr() as *const c_char;

pub const AF_INET: c_int = 2;
pub const PORT_SERVER: u16 = 27960;

pub type InetPort = u16;
pub type InetHost = c_int;
pub type InetAddress = OTInetAddress;
pub type UInt32 = u32;
pub type UInt8 = u8;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OTInetAddress {
    pub fAddressType: c_int,
    pub fPort: InetPort,
    pub fHost: InetHost,
    pub fUnused: [UInt8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InetInterfaceInfo {
    pub fAddress: OTInetAddress,
    pub fNetmask: OTInetAddress,
    pub fBroadcastAddr: OTInetAddress,
    pub fDefaultGatewayAddr: OTInetAddress,
    pub fDNSAddr: OTInetAddress,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TOption {
    pub len: c_int,
    pub level: OTXTILevel,
    pub name: OTXTIName,
    pub status: c_int,
    pub value: [UInt32; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TNetbuf {
    pub maxlen: c_int,
    pub len: c_int,
    pub buf: *mut UInt8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TOptMgmt {
    pub opt: TNetbuf,
    pub flags: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TBind {
    pub addr: TNetbuf,
    pub qlen: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TUnitData {
    pub addr: TNetbuf,
    pub opt: TNetbuf,
    pub udata: TNetbuf,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TUDErr {
    pub addr: TNetbuf,
    pub opt: TNetbuf,
    pub error: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DNSAddress {
    pub fAddressType: c_int,
    pub fName: [c_char; 255],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OTConfiguration {
    _unused: [u8; 0],
}

// ============================================================================
// External C functions (from macOS OpenTransport API)
// ============================================================================

extern "C" {
    pub fn Com_Printf(fmt: *const c_char, ...);
    pub fn Com_DPrintf(fmt: *const c_char, ...);
    pub fn Com_Error(level: c_int, fmt: *const c_char, ...);

    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    pub fn InitOpenTransport() -> OSStatus;
    pub fn CloseOpenTransport();
    pub fn OTCreateConfiguration(name: *const c_char) -> *mut OTConfiguration;
    pub fn OTOpenEndpoint(
        config: *mut OTConfiguration,
        oflag: c_int,
        info: *mut c_void,
        err: *mut OSStatus,
    ) -> EndpointRef;
    pub fn OTAsyncOpenEndpoint(
        config: *mut OTConfiguration,
        oflag: c_int,
        info: *mut c_void,
        notifier: extern "C" fn(*mut c_void, OTEventCode, OTResult, *mut c_void),
        contextPtr: *mut c_void,
    ) -> OTResult;
    pub fn OTSetNonBlocking(ep: EndpointRef) -> OTResult;
    pub fn OTBind(ep: EndpointRef, req: *mut TBind, ret: *mut TBind) -> OTResult;
    pub fn OTOptionManagement(ep: EndpointRef, req: *mut TOptMgmt, ret: *mut TOptMgmt) -> OTResult;
    pub fn OTUnbind(ep: EndpointRef) -> OTResult;
    pub fn OTCloseProvider(ep: EndpointRef) -> OTResult;
    pub fn OTLook(ep: EndpointRef) -> OTResult;
    pub fn OTRcvUDErr(ep: EndpointRef, uderr: *mut TUDErr) -> OTResult;
    pub fn OTInetGetInterfaceInfo(
        info: *mut InetInterfaceInfo,
        index: c_int,
    ) -> OSStatus;
    pub fn OTSndUData(ep: EndpointRef, d: *mut TUnitData) -> OTResult;
    pub fn OTRcvUData(ep: EndpointRef, d: *mut TUnitData, flags: *mut OTFlags) -> OTResult;
    pub fn OTResolveAddress(
        resolver: EndpointRef,
        in_bind: *mut TBind,
        out_bind: *mut TBind,
        timeout: c_int,
    ) -> OTResult;
    pub fn OTInitDNSAddress(dnsAddr: *mut DNSAddress, name: *mut c_char) -> c_int;
}

// ============================================================================
// Global variables
// ============================================================================

static mut gOTInited: qboolean = qfalse;
static mut endpoint: EndpointRef = std::ptr::null_mut();
static mut resolverEndpoint: EndpointRef = std::ptr::null_mut();

const MAX_IPS: usize = 16;
static mut numIP: c_int = 0;
static mut sys_inetInfo: [InetInterfaceInfo; MAX_IPS] = [InetInterfaceInfo {
    fAddress: OTInetAddress {
        fAddressType: 0,
        fPort: 0,
        fHost: 0,
        fUnused: [0; 8],
    },
    fNetmask: OTInetAddress {
        fAddressType: 0,
        fPort: 0,
        fHost: 0,
        fUnused: [0; 8],
    },
    fBroadcastAddr: OTInetAddress {
        fAddressType: 0,
        fPort: 0,
        fHost: 0,
        fUnused: [0; 8],
    },
    fDefaultGatewayAddr: OTInetAddress {
        fAddressType: 0,
        fPort: 0,
        fHost: 0,
        fUnused: [0; 8],
    },
    fDNSAddr: OTInetAddress {
        fAddressType: 0,
        fPort: 0,
        fHost: 0,
        fUnused: [0; 8],
    },
}; MAX_IPS];

static mut uderr: TUDErr = TUDErr {
    addr: TNetbuf {
        maxlen: 0,
        len: 0,
        buf: std::ptr::null_mut(),
    },
    opt: TNetbuf {
        maxlen: 0,
        len: 0,
        buf: std::ptr::null_mut(),
    },
    error: 0,
};

// ============================================================================
// Functions
// ============================================================================

pub fn RcvUDErr() {
    unsafe {
        memset(&mut uderr as *mut _ as *mut c_void, 0, core::mem::size_of::<TUDErr>());
        uderr.addr.maxlen = 0;
        uderr.opt.maxlen = 0;
        OTRcvUDErr(endpoint, &mut uderr);
    }
}

pub fn HandleOTError(err: c_int, func: *const c_char) {
    unsafe {
        static mut lastErr: c_int = 0;
        let r: c_int;

        if err != lastErr {
            Com_Printf(b"%s: error %i\n\0".as_ptr() as *const c_char, func, err);
        }

        // if we don't call OTLook, things wedge
        r = OTLook(endpoint);
        if err != lastErr {
            Com_DPrintf(b"%s: OTLook %i\n\0".as_ptr() as *const c_char, func, r);
        }

        match r {
            T_UDERR => {
                RcvUDErr();
                if err != lastErr {
                    Com_DPrintf(
                        b"%s: OTRcvUDErr %i\n\0".as_ptr() as *const c_char,
                        func,
                        uderr.error,
                    );
                }
            }
            _ => {
                // Com_Printf( "%s: Unknown OTLook error %i\n", func, r );
            }
        }
        lastErr = err; // don't spew tons of messages
    }
}

/*
=================
NotifyProc
=================
*/
#[no_mangle]
pub extern "C" fn NotifyProc(
    _contextPtr: *mut c_void,
    code: OTEventCode,
    _result: OTResult,
    cookie: *mut c_void,
) {
    unsafe {
        match code {
            T_OPENCOMPLETE => {
                endpoint = cookie;
            }
            T_UDERR => {
                RcvUDErr();
            }
            _ => {}
        }
    }
}

/*
=================
GetFourByteOption
=================
*/
unsafe fn GetFourByteOption(
    ep: EndpointRef,
    level: OTXTILevel,
    name: OTXTIName,
    value: *mut UInt32,
) -> OTResult {
    let mut option: TOption = TOption {
        len: kOTFourByteOptionSize as c_int,
        level: level,
        name: name,
        status: 0,
        value: [0],
    };
    let mut request: TOptMgmt = TOptMgmt {
        opt: TNetbuf {
            maxlen: core::mem::size_of::<TOption>() as c_int,
            len: core::mem::size_of::<TOption>() as c_int,
            buf: (&option as *const TOption) as *mut UInt8,
        },
        flags: T_CURRENT,
    };
    let mut result_mgmt: TOptMgmt = TOptMgmt {
        opt: TNetbuf {
            maxlen: core::mem::size_of::<TOption>() as c_int,
            len: 0,
            buf: (&option as *const TOption) as *mut UInt8,
        },
        flags: 0,
    };

    /* Set up the option buffer */
    option.len = kOTFourByteOptionSize as c_int;
    option.level = level;
    option.name = name;
    option.status = 0;
    option.value[0] = 0; // Ignored because we're getting the value.

    let err = OTOptionManagement(ep, &mut request, &mut result_mgmt);

    if err == noErr {
        match option.status {
            T_SUCCESS | T_READONLY => {
                *value = option.value[0];
            }
            _ => {
                return option.status;
            }
        }
    }

    err
}

/*
=================
SetFourByteOption
=================
*/
unsafe fn SetFourByteOption(
    ep: EndpointRef,
    level: OTXTILevel,
    name: OTXTIName,
    value: UInt32,
) -> OTResult {
    let mut option: TOption = TOption {
        len: kOTFourByteOptionSize as c_int,
        level: level,
        name: name,
        status: 0,
        value: [value],
    };
    let mut request: TOptMgmt = TOptMgmt {
        opt: TNetbuf {
            maxlen: core::mem::size_of::<TOption>() as c_int,
            len: core::mem::size_of::<TOption>() as c_int,
            buf: (&option as *const TOption) as *mut UInt8,
        },
        flags: T_NEGOTIATE,
    };
    let mut result_mgmt: TOptMgmt = TOptMgmt {
        opt: TNetbuf {
            maxlen: core::mem::size_of::<TOption>() as c_int,
            len: 0,
            buf: (&option as *const TOption) as *mut UInt8,
        },
        flags: 0,
    };

    /* Set up the option buffer to specify the option and value to
         set. */
    option.len = kOTFourByteOptionSize as c_int;
    option.level = level;
    option.name = name;
    option.status = 0;
    option.value[0] = value;

    let err = OTOptionManagement(ep, &mut request, &mut result_mgmt);

    if err == noErr {
        if option.status != T_SUCCESS {
            return option.status;
        }
    }

    err
}

/*
=====================
NET_GetLocalAddress
=====================
*/
pub fn NET_GetLocalAddress() {
    let mut err: OSStatus;

    unsafe {
        for i in 0..MAX_IPS {
            numIP = i as c_int;
            err = OTInetGetInterfaceInfo(&mut sys_inetInfo[i], i as c_int);
            if err != 0 {
                break;
            }
            let addr_bytes = &sys_inetInfo[i].fAddress as *const OTInetAddress as *const byte;
            Com_Printf(
                b"LocalAddress: %i.%i.%i.%i\n\0".as_ptr() as *const c_char,
                *addr_bytes.offset(0) as c_int,
                *addr_bytes.offset(1) as c_int,
                *addr_bytes.offset(2) as c_int,
                *addr_bytes.offset(3) as c_int,
            );

            let mask_bytes = &sys_inetInfo[i].fNetmask as *const OTInetAddress as *const byte;
            Com_Printf(
                b"Netmask: %i.%i.%i.%i\n\0".as_ptr() as *const c_char,
                *mask_bytes.offset(0) as c_int,
                *mask_bytes.offset(1) as c_int,
                *mask_bytes.offset(2) as c_int,
                *mask_bytes.offset(3) as c_int,
            );
        }
    }
}

/*
==================
Sys_InitNetworking


struct InetAddress
{
		OTAddressType	fAddressType;	// always AF_INET
		InetPort		fPort;			// Port number
		InetHost		fHost;			// Host address in net byte order
		UInt8			fUnused[8];		// Traditional unused bytes
};
typedef struct InetAddress InetAddress;

==================
*/
pub fn Sys_InitNetworking() {
    unsafe {
        let mut err: OSStatus;
        let mut config: *mut OTConfiguration;
        let mut bind: TBind = TBind {
            addr: TNetbuf {
                maxlen: 0,
                len: 0,
                buf: std::ptr::null_mut(),
            },
            qlen: 0,
        };
        let mut bindOut: TBind = TBind {
            addr: TNetbuf {
                maxlen: 0,
                len: 0,
                buf: std::ptr::null_mut(),
            },
            qlen: 0,
        };
        let mut in_addr: OTInetAddress = OTInetAddress {
            fAddressType: 0,
            fPort: 0,
            fHost: 0,
            fUnused: [0; 8],
        };
        let out: OTInetAddress = OTInetAddress {
            fAddressType: 0,
            fPort: 0,
            fHost: 0,
            fUnused: [0; 8],
        };

        Com_Printf(b"----- Sys_InitNetworking -----\n\0".as_ptr() as *const c_char);
        // init OpenTransport
        Com_Printf(b"... InitOpenTransport()\n\0".as_ptr() as *const c_char);
        err = InitOpenTransport();
        if err != noErr {
            Com_Printf(b"InitOpenTransport() failed\n\0".as_ptr() as *const c_char);
            Com_Printf(b"------------------------------\n\0".as_ptr() as *const c_char);
            return;
        }

        gOTInited = qtrue;

        // get an endpoint
        Com_Printf(b"... OTOpenEndpoint()\n\0".as_ptr() as *const c_char);
        config = OTCreateConfiguration(kUDPName);

        #[allow(unreachable_code)]
        {
            endpoint = OTOpenEndpoint(config, 0, std::ptr::null_mut(), &mut err);

            /*
            err = OTAsyncOpenEndpoint( config, 0, 0, NotifyProc, 0 );
            if ( !endpoint ) {
                err = 1;
            }
            */
        }

        if err != noErr {
            endpoint = std::ptr::null_mut();
            Com_Printf(b"OTOpenEndpoint() failed\n\0".as_ptr() as *const c_char);
            Com_Printf(b"------------------------------\n\0".as_ptr() as *const c_char);
            return;
        }

        // set non-blocking
        err = OTSetNonBlocking(endpoint);

        // scan for a valid port in our range
        Com_Printf(b"... OTBind()\n\0".as_ptr() as *const c_char);
        for i in 0..10 {
            in_addr.fAddressType = AF_INET;
            in_addr.fPort = (PORT_SERVER as c_int + i as c_int) as u16;
            in_addr.fHost = 0;

            bind.addr.maxlen = core::mem::size_of::<OTInetAddress>() as c_int;
            bind.addr.len = core::mem::size_of::<OTInetAddress>() as c_int;
            bind.addr.buf = (&in_addr as *const OTInetAddress) as *mut UInt8;
            bind.qlen = 0;

            bindOut.addr.maxlen = core::mem::size_of::<OTInetAddress>() as c_int;
            bindOut.addr.len = core::mem::size_of::<OTInetAddress>() as c_int;
            bindOut.addr.buf = (&out as *const OTInetAddress) as *mut UInt8;
            bindOut.qlen = 0;

            err = OTBind(endpoint, &mut bind, &mut bindOut);
            if err == noErr {
                Com_Printf(
                    b"Opened UDP endpoint at port %i\n\0".as_ptr() as *const c_char,
                    out.fPort as c_int,
                );
                break;
            }
        }

        if err != noErr {
            Com_Printf(b"Couldn't bind a local port\n\0".as_ptr() as *const c_char);
        }

        // get the local address for LAN client detection
        NET_GetLocalAddress();

        // set to allow broadcasts
        err = SetFourByteOption(endpoint, INET_IP, IP_BROADCAST, T_YES as UInt32);

        if err != noErr {
            Com_Printf(b"IP_BROADCAST failed\n\0".as_ptr() as *const c_char);
        }

        // get an endpoint just for resolving addresses, because
        // I was having crashing problems doing it on the same endpoint
        config = OTCreateConfiguration(kUDPName);
        resolverEndpoint = OTOpenEndpoint(config, 0, std::ptr::null_mut(), &mut err);
        if err != noErr {
            resolverEndpoint = std::ptr::null_mut();
            Com_Printf(b"OTOpenEndpoint() for resolver failed\n\0".as_ptr() as *const c_char);
            Com_Printf(b"------------------------------\n\0".as_ptr() as *const c_char);
            return;
        }

        in_addr.fAddressType = AF_INET;
        in_addr.fPort = 0;
        in_addr.fHost = 0;

        bind.addr.maxlen = core::mem::size_of::<OTInetAddress>() as c_int;
        bind.addr.len = core::mem::size_of::<OTInetAddress>() as c_int;
        bind.addr.buf = (&in_addr as *const OTInetAddress) as *mut UInt8;
        bind.qlen = 0;

        bindOut.addr.maxlen = core::mem::size_of::<OTInetAddress>() as c_int;
        bindOut.addr.len = core::mem::size_of::<OTInetAddress>() as c_int;
        bindOut.addr.buf = (&out as *const OTInetAddress) as *mut UInt8;
        bindOut.qlen = 0;

        err = OTBind(resolverEndpoint, &mut bind, &mut bindOut);

        Com_Printf(b"------------------------------\n\0".as_ptr() as *const c_char);
    }
}

/*
==================
Sys_ShutdownNetworking
==================
*/
pub fn Sys_ShutdownNetworking() {
    unsafe {
        Com_Printf(b"Sys_ShutdownNetworking();\n\0".as_ptr() as *const c_char);

        if endpoint != kOTInvalidEndpointRef {
            OTUnbind(endpoint);
            OTCloseProvider(endpoint);
            endpoint = kOTInvalidEndpointRef;
        }
        if resolverEndpoint != kOTInvalidEndpointRef {
            OTUnbind(resolverEndpoint);
            OTCloseProvider(resolverEndpoint);
            resolverEndpoint = kOTInvalidEndpointRef;
        }
        if gOTInited != 0 {
            CloseOpenTransport();
            gOTInited = qfalse;
        }
    }
}

/*
=============
Sys_StringToAdr


Does NOT parse port numbers


idnewt
192.246.40.70
=============
*/
pub fn Sys_StringToAdr(s: *const c_char, a: *mut netadr_t) -> qboolean {
    unsafe {
        let mut err: OSStatus;
        let mut inAddr: OTInetAddress = OTInetAddress {
            fAddressType: 0,
            fPort: 0,
            fHost: 0,
            fUnused: [0; 8],
        };
        let mut dnsAddr: DNSAddress = DNSAddress {
            fAddressType: 0,
            fName: [0; 255],
        };
        let mut in_bind: TBind = TBind {
            addr: TNetbuf {
                maxlen: 0,
                len: 0,
                buf: std::ptr::null_mut(),
            },
            qlen: 0,
        };
        let mut out: TBind = TBind {
            addr: TNetbuf {
                maxlen: 0,
                len: 0,
                buf: std::ptr::null_mut(),
            },
            qlen: 0,
        };

        if resolverEndpoint.is_null() {
            return qfalse;
        }

        in_bind.addr.buf = (&dnsAddr as *const DNSAddress) as *mut UInt8;
        in_bind.addr.len = OTInitDNSAddress(&mut dnsAddr, s as *mut c_char);
        in_bind.qlen = 0;

        out.addr.buf = (&inAddr as *const OTInetAddress) as *mut UInt8;
        out.addr.maxlen = core::mem::size_of::<OTInetAddress>() as c_int;
        out.qlen = 0;

        err = OTResolveAddress(resolverEndpoint, &mut in_bind, &mut out, 10000);
        if err != 0 {
            HandleOTError(err, b"Sys_StringToAdr\0".as_ptr() as *const c_char);
            return qfalse;
        }

        (*a).r#type = netadrtype_t::NA_IP;
        *((&mut (*a).ip[0] as *mut byte) as *mut c_int) = inAddr.fHost;

        return qtrue;
    }
}

/*
==================
Sys_SendPacket
==================
*/
const MAX_PACKETLEN: c_int = 1400;

pub fn Sys_SendPacket(length: c_int, data: *const c_void, to: netadr_t) {
    unsafe {
        let mut inAddr: OTInetAddress = OTInetAddress {
            fAddressType: 0,
            fPort: 0,
            fHost: 0,
            fUnused: [0; 8],
        };
        let mut err: OSStatus;

        if endpoint.is_null() {
            return;
        }

        if length > MAX_PACKETLEN {
            Com_Error(
                0,
                b"Sys_SendPacket: length > MAX_PACKETLEN\0".as_ptr() as *const c_char,
            );
        }

        inAddr.fAddressType = AF_INET;
        inAddr.fPort = to.port;
        if to.r#type == netadrtype_t::NA_BROADCAST {
            inAddr.fHost = -1;
        } else {
            inAddr.fHost = *((&to.ip[0] as *const byte) as *const c_int);
        }

        let mut d: TUnitData = TUnitData {
            addr: TNetbuf {
                len: core::mem::size_of::<OTInetAddress>() as c_int,
                maxlen: core::mem::size_of::<OTInetAddress>() as c_int,
                buf: (&inAddr as *const OTInetAddress) as *mut UInt8,
            },
            opt: TNetbuf {
                len: 0,
                maxlen: 0,
                buf: std::ptr::null_mut(),
            },
            udata: TNetbuf {
                len: length,
                maxlen: length,
                buf: data as *mut UInt8,
            },
        };

        err = OTSndUData(endpoint, &mut d);
        if err != 0 {
            HandleOTError(err, b"Sys_SendPacket\0".as_ptr() as *const c_char);
        }
    }
}

/*
==================
Sys_GetPacket

Never called by the game logic, just the system event queing
==================
*/
pub fn Sys_GetPacket(net_from: *mut netadr_t, net_message: *mut msg_t) -> qboolean {
    unsafe {
        let mut inAddr: OTInetAddress = OTInetAddress {
            fAddressType: 0,
            fPort: 0,
            fHost: 0,
            fUnused: [0; 8],
        };
        let mut err: OSStatus;
        let mut flags: OTFlags = 0;

        if endpoint.is_null() {
            return qfalse;
        }

        inAddr.fAddressType = AF_INET;
        inAddr.fPort = 0;
        inAddr.fHost = 0;

        let mut d: TUnitData = TUnitData {
            addr: TNetbuf {
                len: core::mem::size_of::<OTInetAddress>() as c_int,
                maxlen: core::mem::size_of::<OTInetAddress>() as c_int,
                buf: (&inAddr as *const OTInetAddress) as *mut UInt8,
            },
            opt: TNetbuf {
                len: 0,
                maxlen: 0,
                buf: std::ptr::null_mut(),
            },
            udata: TNetbuf {
                len: (*net_message).maxsize,
                maxlen: (*net_message).maxsize,
                buf: (*net_message).data,
            },
        };

        err = OTRcvUData(endpoint, &mut d, &mut flags);
        if err != 0 {
            if err == kOTNoDataErr {
                return 0;
            }
            HandleOTError(err, b"Sys_GetPacket\0".as_ptr() as *const c_char);
            return qfalse;
        }

        (*net_from).r#type = netadrtype_t::NA_IP;
        (*net_from).port = inAddr.fPort;
        *((&mut (*net_from).ip[0] as *mut byte) as *mut c_int) = inAddr.fHost;

        (*net_message).cursize = d.udata.len;

        return qtrue;
    }
}

/*
==================
Sys_IsLANAddress

LAN clients will have their rate var ignored
==================
*/
pub fn Sys_IsLANAddress(adr: netadr_t) -> qboolean {
    if adr.r#type == netadrtype_t::NA_LOOPBACK {
        return qtrue;
    }

    if adr.r#type != netadrtype_t::NA_IP {
        return qfalse;
    }

    unsafe {
        for ip in 0..numIP {
            let mut i: c_int = 0;
            for byte_idx in 0..4 {
                let addr_byte_ptr = &sys_inetInfo[ip as usize].fAddress as *const OTInetAddress as *const byte;
                let mask_byte_ptr = &sys_inetInfo[ip as usize].fNetmask as *const OTInetAddress as *const byte;
                if (adr.ip[byte_idx as usize] & *mask_byte_ptr.offset(byte_idx as isize))
                    != (*addr_byte_ptr.offset(byte_idx as isize)
                        & *mask_byte_ptr.offset(byte_idx as isize))
                {
                    break;
                }
                i += 1;
            }
            if i == 4 {
                return qtrue; // matches this subnet
            }
        }
    }

    qfalse
}

pub fn NET_Sleep(_i: c_int) {}
