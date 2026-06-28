#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_uchar, c_uint};

use super::mhead_h::MPEG_HEAD;

static mp_br_table: [[c_int; 16]; 2] = [
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
    [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384, 0],
];
static mp_sr20_table: [[c_int; 4]; 2] = [[441, 480, 320, -999], [882, 960, 640, -999]];

static mp_br_tableL1: [[c_int; 16]; 2] = [
    [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256, 0], /* mpeg2 */
    [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448, 0],
];

static mp_br_tableL3: [[c_int; 16]; 2] = [
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0], /* mpeg 2 */
    [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0],
];

/*--------------------------------------------------------------*/
pub unsafe extern "C" fn head_info(
    buf: *mut c_uchar,
    mut n: c_uint,
    h: *mut MPEG_HEAD,
) -> c_int {
    let mut framebytes: c_int;
    let mpeg25_flag: c_int;

    if n > 10000 {
        n = 10000; /* limit scan for free format */
    }

    (*h).sync = 0;
    //if ((buf[0] == 0xFF) && ((buf[1] & 0xF0) == 0xF0))
    if ((*buf.add(0) as c_int) == 0xFF) && (((*buf.add(0 + 1) as c_int) & 0xF0) == 0xF0) {
        mpeg25_flag = 0; // mpeg 1 & 2
    } else if ((*buf.add(0) as c_int) == 0xFF)
        && (((*buf.add(0 + 1) as c_int) & 0xF0) == 0xE0)
    {
        mpeg25_flag = 1; // mpeg 2.5
    } else {
        return 0; // sync fail
    }

    (*h).sync = 1;
    if mpeg25_flag != 0 {
        (*h).sync = 2; //low bit clear signals mpeg25 (as in 0xFFE)
    }

    (*h).id = ((*buf.add(0 + 1) as c_int) & 0x08) >> 3;
    (*h).option = ((*buf.add(0 + 1) as c_int) & 0x06) >> 1;
    (*h).prot = (*buf.add(0 + 1) as c_int) & 0x01;

    (*h).br_index = ((*buf.add(0 + 2) as c_int) & 0xf0) >> 4;
    (*h).sr_index = ((*buf.add(0 + 2) as c_int) & 0x0c) >> 2;
    (*h).pad = ((*buf.add(0 + 2) as c_int) & 0x02) >> 1;
    (*h).private_bit = (*buf.add(0 + 2) as c_int) & 0x01;
    (*h).mode = ((*buf.add(0 + 3) as c_int) & 0xc0) >> 6;
    (*h).mode_ext = ((*buf.add(0 + 3) as c_int) & 0x30) >> 4;
    (*h).cr = ((*buf.add(0 + 3) as c_int) & 0x08) >> 3;
    (*h).original = ((*buf.add(0 + 3) as c_int) & 0x04) >> 2;
    (*h).emphasis = (*buf.add(0 + 3) as c_int) & 0x03;

    // if( mpeg25_flag ) {
    //    if( h->sr_index == 2 ) return 0;   // fail 8khz
    //}

    /* compute framebytes for Layer I, II, III */
    if (*h).option < 1 {
        return 0;
    }
    if (*h).option > 3 {
        return 0;
    }

    framebytes = 0;

    if (*h).br_index > 0 {
        if (*h).option == 3 {
            /* layer I */
            framebytes = 240 * mp_br_tableL1[(*h).id as usize][(*h).br_index as usize]
                / mp_sr20_table[(*h).id as usize][(*h).sr_index as usize];
            framebytes = 4 * framebytes;
        } else if (*h).option == 2 {
            /* layer II */
            framebytes = 2880 * mp_br_table[(*h).id as usize][(*h).br_index as usize]
                / mp_sr20_table[(*h).id as usize][(*h).sr_index as usize];
        } else if (*h).option == 1 {
            /* layer III */
            if (*h).id != 0 {
                // mpeg1

                framebytes = 2880 * mp_br_tableL3[(*h).id as usize][(*h).br_index as usize]
                    / mp_sr20_table[(*h).id as usize][(*h).sr_index as usize];
            } else {
                // mpeg2

                if mpeg25_flag != 0 {
                    // mpeg2.2

                    framebytes = 2880 * mp_br_tableL3[(*h).id as usize][(*h).br_index as usize]
                        / mp_sr20_table[(*h).id as usize][(*h).sr_index as usize];
                } else {
                    framebytes = 1440 * mp_br_tableL3[(*h).id as usize][(*h).br_index as usize]
                        / mp_sr20_table[(*h).id as usize][(*h).sr_index as usize];
                }
            }
        }
    } else {
        framebytes = find_sync(buf, n as c_int); /* free format */
    }

    return framebytes;
}

pub unsafe extern "C" fn head_info3(
    buf: *mut c_uchar,
    n: c_uint,
    h: *mut MPEG_HEAD,
    br: *mut c_int,
    searchForward: *mut c_uint,
) -> c_int {
    let mut pBuf: c_uint = 0;

    // jdw insertion...
    while (pBuf < n)
        && !(((*buf.add(pBuf as usize) as c_int) == 0xFF)
            && (((*buf.add((pBuf + 1) as usize) as c_int) & 0xF0) == 0xF0
                || (((*buf.add((pBuf + 1) as usize) as c_int) & 0xF0) == 0xE0)))
    {
        pBuf += 1;
    }

    if pBuf == n {
        return 0;
    }

    *searchForward = pBuf;
    return head_info2(buf.add(pBuf as usize), n, h, br);
}

/*--------------------------------------------------------------*/
pub unsafe extern "C" fn head_info2(
    buf: *mut c_uchar,
    n: c_uint,
    h: *mut MPEG_HEAD,
    br: *mut c_int,
) -> c_int {
    let framebytes: c_int;

    /*---  return br (in bits/sec) in addition to frame bytes ---*/

    *br = 0;
    /*-- assume fail --*/
    framebytes = head_info(buf, n, h);

    if framebytes == 0 {
        return 0;
    }

    match (*h).option {
        1 => {
            /* layer III */
            if (*h).br_index > 0 {
                *br = 1000 * mp_br_tableL3[(*h).id as usize][(*h).br_index as usize];
            } else {
                if (*h).id != 0 {
                    // mpeg1

                    *br = 1000 * framebytes * mp_sr20_table[(*h).id as usize][(*h).sr_index as usize]
                        / (144 * 20);
                } else {
                    // mpeg2

                    if ((*h).sync & 1) == 0 {
                        //  flags mpeg25

                        *br = 500 * framebytes
                            * mp_sr20_table[(*h).id as usize][(*h).sr_index as usize]
                            / (72 * 20);
                    } else {
                        *br = 1000 * framebytes
                            * mp_sr20_table[(*h).id as usize][(*h).sr_index as usize]
                            / (72 * 20);
                    }
                }
            }
        }

        2 => {
            /* layer II */
            if (*h).br_index > 0 {
                *br = 1000 * mp_br_table[(*h).id as usize][(*h).br_index as usize];
            } else {
                *br = 1000 * framebytes * mp_sr20_table[(*h).id as usize][(*h).sr_index as usize]
                    / (144 * 20);
            }
        }

        3 => {
            /* layer I */
            if (*h).br_index > 0 {
                *br = 1000 * mp_br_tableL1[(*h).id as usize][(*h).br_index as usize];
            } else {
                *br = 1000 * framebytes * mp_sr20_table[(*h).id as usize][(*h).sr_index as usize]
                    / (48 * 20);
            }
        }

        _ => {
            return 0; // fuck knows what this is, but it ain't one of ours...
        }
    }

    return framebytes;
}

/*--------------------------------------------------------------*/
unsafe fn compare(buf: *mut c_uchar, buf2: *mut c_uchar) -> c_int {
    if *buf.add(0) != *buf2.add(0) {
        return 0;
    }
    if *buf.add(1) != *buf2.add(1) {
        return 0;
    }
    return 1;
}

/*----------------------------------------------------------*/
/*-- does not scan for initial sync, initial sync assumed --*/
unsafe fn find_sync(buf: *mut c_uchar, mut n: c_int) -> c_int {
    let mut i0: c_int;
    let mut isync: c_int;
    let nmatch: c_int;
    let pad: c_int;
    let mut padbytes: c_int;
    let option: c_int;

    /* mod 4/12/95 i0 change from 72, allows as low as 8kbits for mpeg1 */
    i0 = 24;
    padbytes = 1;
    option = ((*buf.add(1) as c_int) & 0x06) >> 1;
    if option == 3 {
        padbytes = 4;
        i0 = 24; /* for shorter layer I frames */
    }

    pad = ((*buf.add(2) as c_int) & 0x02) >> 1;

    n -= 3; /*  need 3 bytes of header  */

    while i0 < 2000 {
        isync = sync_scan(buf, n, i0);
        i0 = isync + 1;
        isync -= pad;
        if isync <= 0 {
            return 0;
        }
        nmatch = sync_test(buf, n, isync, padbytes);
        if nmatch > 0 {
            return isync;
        }
    }

    return 0;
}

/*------------------------------------------------------*/
/*---- scan for next sync, assume start is valid -------*/
/*---- return number bytes to next sync ----------------*/
unsafe fn sync_scan(buf: *mut c_uchar, n: c_int, i0: c_int) -> c_int {
    let mut i: c_int;

    i = i0;
    while i < n {
        if compare(buf, buf.add(i as usize)) != 0 {
            return i;
        }
        i += 1;
    }

    return 0;
}

/*------------------------------------------------------*/
/*- test consecutative syncs, input isync without pad --*/
unsafe fn sync_test(buf: *mut c_uchar, n: c_int, isync: c_int, padbytes: c_int) -> c_int {
    let mut i: c_int;
    let mut nmatch: c_int;
    let pad: c_int;

    nmatch = 0;
    i = 0;
    loop {
        pad = padbytes * (((*buf.add((i + 2) as usize) as c_int) & 0x02) >> 1);
        i += pad + isync;
        if i > n {
            break;
        }
        if compare(buf, buf.add(i as usize)) == 0 {
            return -nmatch;
        }
        nmatch += 1;
    }
    return nmatch;
}
