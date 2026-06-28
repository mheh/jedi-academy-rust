/*____________________________________________________________________________

    FreeAmp - The Free MP3 Player

        MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
        Corp.  http://www.xingtech.com

    Portions Copyright (C) 1998-1999 EMusic.com

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program; if not, write to the Free Software
    Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

    $Id: cupl3.c,v 1.8 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cupL3.c  ***************************************************
unpack Layer III


mod 8/18/97  bugfix crc problem

mod 10/9/97  add pMP3Stream->band_limit12 for short blocks

mod 10/22/97  zero buf_ptrs in init

mod 5/15/98 mpeg 2.5

mod 8/19/98 decode 22 sf bands

******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(static_mut_refs)]

use core::ffi::{c_int, c_short, c_uchar, c_uint, c_void};
use core::ptr;

use super::cup::{decinfo, gpNextByteAfterData};
use super::hwin::{hybrid, hybrid_sum, sum_f_bands, FreqInvert};
use super::l3_h::{BITDAT, CB_INFO, GR, IS_SF_INFO, SCALEFACT, SIDE_INFO};
use super::l3dq::dequant;
use super::mhead_h::MPEG_HEAD;
use super::mp3struct_h::{
    bFastEstimateOnly, pMP3Stream, SBT_FUNCTION, XFORM_FUNCTION, BUF_TRIGGER, NBUF,
};
use super::msis::{antialias, is_process_MPEG1, is_process_MPEG2, ms_process};
use super::small_header_h::{IN_OUT, SAMPLE};
use super::uph::{unpack_huff, unpack_huff_quad};
use super::upsf::{unpack_sf_sub_MPEG1, unpack_sf_sub_MPEG2};

const fn zero_gr() -> GR {
    GR {
        part2_3_length: 0,
        big_values: 0,
        global_gain: 0,
        scalefac_compress: 0,
        window_switching_flag: 0,
        block_type: 0,
        mixed_block_flag: 0,
        table_select: [0; 3],
        subblock_gain: [0; 3],
        region0_count: 0,
        region1_count: 0,
        preflag: 0,
        scalefac_scale: 0,
        count1table_select: 0,
    }
}

const fn zero_side_info() -> SIDE_INFO {
    SIDE_INFO {
        mode: 0,
        mode_ext: 0,
        main_data_begin: 0,
        private_bits: 0,
        scfsi: [0; 2],
        gr: [[zero_gr(); 2]; 2],
    }
}

const fn zero_scalefact() -> SCALEFACT {
    SCALEFACT {
        l: [0; 23],
        s: [[0; 13]; 3],
    }
}

const fn zero_cb_info() -> CB_INFO {
    CB_INFO {
        cbtype: 0,
        cbmax: 0,
        cbs0: 0,
        ncbl: 0,
        cbmax_s: [0; 3],
    }
}

const fn zero_is_sf_info() -> IS_SF_INFO {
    IS_SF_INFO {
        nr: [0; 3],
        slen: [0; 3],
        intensity_scale: 0,
    }
}

/*====================================================================*/
static mp_sr20_table: [[c_int; 4]; 2] = [[441, 480, 320, -999], [882, 960, 640, -999]];
static mp_br_tableL3: [[c_int; 16]; 2] = [
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0],
    [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0],
];

static mut cb_info: [[CB_INFO; 2]; 2] = [[zero_cb_info(); 2]; 2];
static mut is_sf_info: IS_SF_INFO = zero_is_sf_info();
static mut side_info: SIDE_INFO = zero_side_info();
static mut sf: [[SCALEFACT; 2]; 2] = [[zero_scalefact(); 2]; 2];
static mut nsamp: [[c_int; 2]; 2] = [[0; 2]; 2];
static mut yout: [f32; 576] = [0.0; 576];

#[unsafe(no_mangle)]
pub static mut bitdat: BITDAT = BITDAT {
    bitbuf: 0,
    bits: 0,
    bs_ptr: ptr::null_mut(),
    bs_ptr0: ptr::null_mut(),
    bs_ptr_end: ptr::null_mut(),
};

/*------------- initialize bit getter -------------*/
unsafe fn bitget_init(buf: *mut c_uchar) {
    bitdat.bs_ptr0 = buf;
    bitdat.bs_ptr = buf;
    bitdat.bits = 0;
    bitdat.bitbuf = 0;
}

/*------------- initialize bit getter -------------*/
unsafe fn bitget_init_end(buf_end: *mut c_uchar) {
    bitdat.bs_ptr_end = buf_end;
}

/*------------- get n bits from bitstream -------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitget_bits_used() -> c_int {
    ((bitdat.bs_ptr.offset_from(bitdat.bs_ptr0) as c_int) << 3) - bitdat.bits
}

/*------------- check for n bits in bitbuf -------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitget_check(n: c_int) {
    if bitdat.bits < n {
        while bitdat.bits <= 24 {
            bitdat.bitbuf = (bitdat.bitbuf << 8) | *bitdat.bs_ptr as c_uint;
            bitdat.bs_ptr = bitdat.bs_ptr.add(1);
            bitdat.bits += 8;
        }
    }
}

/*------------- get n bits from bitstream -------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitget(n: c_int) -> c_uint {
    let x: c_uint;

    if bitdat.bits < n {
        while bitdat.bits <= 24 {
            bitdat.bitbuf = (bitdat.bitbuf << 8) | *bitdat.bs_ptr as c_uint;
            bitdat.bs_ptr = bitdat.bs_ptr.add(1);
            bitdat.bits += 8;
        }
    }
    bitdat.bits -= n;
    x = bitdat.bitbuf >> bitdat.bits;
    bitdat.bitbuf = bitdat.bitbuf.wrapping_sub(x << bitdat.bits);
    x
}

/*------------- get 1 bit from bitstream -------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitget_1bit() -> c_uint {
    let x: c_uint;

    if bitdat.bits <= 0 {
        while bitdat.bits <= 24 {
            bitdat.bitbuf = (bitdat.bitbuf << 8) | *bitdat.bs_ptr as c_uint;
            bitdat.bs_ptr = bitdat.bs_ptr.add(1);
            bitdat.bits += 8;
        }
    }
    bitdat.bits -= 1;
    x = bitdat.bitbuf >> bitdat.bits;
    bitdat.bitbuf = bitdat.bitbuf.wrapping_sub(x << bitdat.bits);
    x
}

unsafe extern "C" {
    fn sbt_mono_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbt_dual_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbt16_mono_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbt16_dual_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbt8_mono_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbt8_dual_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);

    fn sbtB_mono_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbtB_dual_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbtB16_mono_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbtB16_dual_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbtB8_mono_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);
    fn sbtB8_dual_L3(sample: *mut f32, pcm: *mut c_short, ch: c_int);

    fn L3table_init() -> c_int;
    fn msis_init();
    fn sbt_init();
}

unsafe fn sample_ptr(ch: c_int, igr: c_int) -> *mut SAMPLE {
    (*pMP3Stream).u.L3.sample[ch as usize][igr as usize].as_mut_ptr()
}

unsafe fn sample_float_ptr(ch: c_int, igr: c_int) -> *mut f32 {
    sample_ptr(ch, igr).cast::<f32>()
}

unsafe fn call_sbt(pcm: *mut c_void, ch: c_int) {
    (*pMP3Stream)
        .u
        .L3
        .sbt_L3
        .unwrap_unchecked()(ptr::addr_of_mut!(yout).cast::<f32>(), pcm.cast::<c_short>(), ch);
}

unsafe fn call_xform(pcm: *mut c_uchar, igr: c_int) {
    (*pMP3Stream)
        .u
        .L3
        .Xform
        .unwrap_unchecked()(pcm.cast::<c_void>(), igr);
}

/*--------------------------------------------------------------------*/
unsafe extern "C" fn Xform_mono(pcm: *mut c_void, igr: c_int) {
    let igr_prev: c_int;
    let mut n1: c_int;
    let mut n2: c_int;

    n1 = nsamp[igr as usize][0];
    n2 = n1;
    if side_info.gr[igr as usize][0].block_type == 2 {
        n1 = 0;
        if side_info.gr[igr as usize][0].mixed_block_flag != 0 {
            n1 = (*pMP3Stream).u.L3.sfBandIndex[0][((*pMP3Stream).u.L3.ncbl_mixed - 1) as usize];
        }
    }
    if n1 > (*pMP3Stream).u.L3.band_limit {
        n1 = (*pMP3Stream).u.L3.band_limit;
    }
    if n2 > (*pMP3Stream).u.L3.band_limit {
        n2 = (*pMP3Stream).u.L3.band_limit;
    }
    igr_prev = igr ^ 1;

    nsamp[igr as usize][0] = hybrid(
        sample_float_ptr(0, igr),
        sample_float_ptr(0, igr_prev),
        ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(),
        side_info.gr[igr as usize][0].block_type,
        n1,
        n2,
        nsamp[igr_prev as usize][0],
    );
    FreqInvert(ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(), nsamp[igr as usize][0]);
    call_sbt(pcm, 0);
}

/*--------------------------------------------------------------------*/
unsafe extern "C" fn Xform_dual_right(pcm: *mut c_void, igr: c_int) {
    let igr_prev: c_int;
    let mut n1: c_int;
    let mut n2: c_int;

    n1 = nsamp[igr as usize][1];
    n2 = n1;
    if side_info.gr[igr as usize][1].block_type == 2 {
        n1 = 0;
        if side_info.gr[igr as usize][1].mixed_block_flag != 0 {
            n1 = (*pMP3Stream).u.L3.sfBandIndex[0][((*pMP3Stream).u.L3.ncbl_mixed - 1) as usize];
        }
    }
    if n1 > (*pMP3Stream).u.L3.band_limit {
        n1 = (*pMP3Stream).u.L3.band_limit;
    }
    if n2 > (*pMP3Stream).u.L3.band_limit {
        n2 = (*pMP3Stream).u.L3.band_limit;
    }
    igr_prev = igr ^ 1;
    nsamp[igr as usize][1] = hybrid(
        sample_float_ptr(1, igr),
        sample_float_ptr(1, igr_prev),
        ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(),
        side_info.gr[igr as usize][1].block_type,
        n1,
        n2,
        nsamp[igr_prev as usize][1],
    );
    FreqInvert(ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(), nsamp[igr as usize][1]);
    call_sbt(pcm, 0);
}

/*--------------------------------------------------------------------*/
unsafe extern "C" fn Xform_dual(pcm: *mut c_void, igr: c_int) {
    let mut ch: c_int;
    let igr_prev: c_int;
    let mut n1: c_int;
    let mut n2: c_int;

    igr_prev = igr ^ 1;
    ch = 0;
    while ch < (*pMP3Stream).u.L3.nchan {
        n1 = nsamp[igr as usize][ch as usize];
        n2 = n1;
        if side_info.gr[igr as usize][ch as usize].block_type == 2 {
            n1 = 0;
            if side_info.gr[igr as usize][ch as usize].mixed_block_flag != 0 {
                n1 = (*pMP3Stream).u.L3.sfBandIndex[0][((*pMP3Stream).u.L3.ncbl_mixed - 1) as usize];
            }
        }
        if n1 > (*pMP3Stream).u.L3.band_limit {
            n1 = (*pMP3Stream).u.L3.band_limit;
        }
        if n2 > (*pMP3Stream).u.L3.band_limit {
            n2 = (*pMP3Stream).u.L3.band_limit;
        }
        nsamp[igr as usize][ch as usize] = hybrid(
            sample_float_ptr(ch, igr),
            sample_float_ptr(ch, igr_prev),
            ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(),
            side_info.gr[igr as usize][ch as usize].block_type,
            n1,
            n2,
            nsamp[igr_prev as usize][ch as usize],
        );
        FreqInvert(
            ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(),
            nsamp[igr as usize][ch as usize],
        );
        call_sbt(pcm, ch);
        ch += 1;
    }
}

/*--------------------------------------------------------------------*/
unsafe extern "C" fn Xform_dual_mono(pcm: *mut c_void, igr: c_int) {
    let igr_prev: c_int;
    let mut n1: c_int;
    let mut n2: c_int;
    let mut n3: c_int;

    igr_prev = igr ^ 1;
    if (side_info.gr[igr as usize][0].block_type == side_info.gr[igr as usize][1].block_type)
        && (side_info.gr[igr as usize][0].mixed_block_flag == 0)
        && (side_info.gr[igr as usize][1].mixed_block_flag == 0)
    {
        n2 = nsamp[igr as usize][0];
        if n2 < nsamp[igr as usize][1] {
            n2 = nsamp[igr as usize][1];
        }
        if n2 > (*pMP3Stream).u.L3.band_limit {
            n2 = (*pMP3Stream).u.L3.band_limit;
        }
        n1 = n2;
        if side_info.gr[igr as usize][0].block_type == 2 {
            n1 = 0;
        }
        sum_f_bands(sample_float_ptr(0, igr), sample_float_ptr(1, igr), n2);
        nsamp[igr as usize][0] = hybrid(
            sample_float_ptr(0, igr),
            sample_float_ptr(0, igr_prev),
            ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(),
            side_info.gr[igr as usize][0].block_type,
            n1,
            n2,
            nsamp[igr_prev as usize][0],
        );
        n3 = nsamp[igr as usize][0];
    } else {
        n1 = nsamp[igr as usize][0];
        n2 = n1;
        if side_info.gr[igr as usize][0].block_type == 2 {
            n1 = 0;
            if side_info.gr[igr as usize][0].mixed_block_flag != 0 {
                n1 = (*pMP3Stream).u.L3.sfBandIndex[0][((*pMP3Stream).u.L3.ncbl_mixed - 1) as usize];
            }
        }
        nsamp[igr as usize][0] = hybrid(
            sample_float_ptr(0, igr),
            sample_float_ptr(0, igr_prev),
            ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(),
            side_info.gr[igr as usize][0].block_type,
            n1,
            n2,
            nsamp[igr_prev as usize][0],
        );
        n3 = nsamp[igr as usize][0];

        n1 = nsamp[igr as usize][1];
        n2 = n1;
        if side_info.gr[igr as usize][1].block_type == 2 {
            n1 = 0;
            if side_info.gr[igr as usize][1].mixed_block_flag != 0 {
                n1 = (*pMP3Stream).u.L3.sfBandIndex[0][((*pMP3Stream).u.L3.ncbl_mixed - 1) as usize];
            }
        }
        nsamp[igr as usize][1] = hybrid_sum(
            sample_float_ptr(1, igr),
            sample_float_ptr(0, igr),
            ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(),
            side_info.gr[igr as usize][1].block_type,
            n1,
            n2,
        );
        if n3 < nsamp[igr as usize][1] {
            n1 = nsamp[igr as usize][1];
        }
    }

    FreqInvert(ptr::addr_of_mut!(yout).cast::<[[f32; 32]; 18]>(), n3);
    call_sbt(pcm, 0);
}

/*====================================================================*/
unsafe fn unpack_side_MPEG1() -> c_int {
    let prot: c_int;
    let br_index: c_int;
    let mut igr: c_int;
    let mut ch: c_int;
    let side_bytes: c_int;

    (*pMP3Stream).id = bitget(1) as c_int;
    bitget(2);
    prot = bitget(1) as c_int;
    br_index = bitget(4) as c_int;
    (*pMP3Stream).sr_index = bitget(2) as c_int;
    (*pMP3Stream).pad = bitget(1) as c_int;
    bitget(1);
    side_info.mode = bitget(2) as c_int;
    side_info.mode_ext = bitget(2) as c_int;

    if side_info.mode != 1 {
        side_info.mode_ext = 0;
    }

    (*pMP3Stream).u.L3.ms_mode = side_info.mode_ext >> 1;
    (*pMP3Stream).u.L3.is_mode = side_info.mode_ext & 1;

    (*pMP3Stream).u.L3.crcbytes = 0;
    if prot != 0 {
        bitget(4);
    } else {
        bitget(20);
        (*pMP3Stream).u.L3.crcbytes = 2;
    }

    if br_index > 0 {
        (*pMP3Stream).framebytes = 2880 * mp_br_tableL3[(*pMP3Stream).id as usize][br_index as usize]
            / mp_sr20_table[(*pMP3Stream).id as usize][(*pMP3Stream).sr_index as usize];
    }

    side_info.main_data_begin = bitget(9) as c_int;
    if side_info.mode == 3 {
        side_info.private_bits = bitget(5) as c_int;
        (*pMP3Stream).u.L3.nchan = 1;
        side_bytes = 4 + 17;
    } else {
        side_info.private_bits = bitget(3) as c_int;
        (*pMP3Stream).u.L3.nchan = 2;
        side_bytes = 4 + 32;
    }
    ch = 0;
    while ch < (*pMP3Stream).u.L3.nchan {
        side_info.scfsi[ch as usize] = bitget(4) as c_int;
        ch += 1;
    }

    igr = 0;
    while igr < 2 {
        ch = 0;
        while ch < (*pMP3Stream).u.L3.nchan {
            let gr = &mut side_info.gr[igr as usize][ch as usize];
            gr.part2_3_length = bitget(12) as c_int;
            gr.big_values = bitget(9) as c_int;
            gr.global_gain = bitget(8) as c_int + (*pMP3Stream).u.L3.gain_adjust;
            if (*pMP3Stream).u.L3.ms_mode != 0 {
                gr.global_gain -= 2;
            }
            gr.scalefac_compress = bitget(4) as c_int;
            gr.window_switching_flag = bitget(1) as c_int;
            if gr.window_switching_flag != 0 {
                gr.block_type = bitget(2) as c_int;
                gr.mixed_block_flag = bitget(1) as c_int;
                gr.table_select[0] = bitget(5) as c_int;
                gr.table_select[1] = bitget(5) as c_int;
                gr.subblock_gain[0] = bitget(3) as c_int;
                gr.subblock_gain[1] = bitget(3) as c_int;
                gr.subblock_gain[2] = bitget(3) as c_int;
                gr.region0_count = 8 - 1;
                gr.region1_count = 20 - (8 - 1);
            } else {
                gr.mixed_block_flag = 0;
                gr.block_type = 0;
                gr.table_select[0] = bitget(5) as c_int;
                gr.table_select[1] = bitget(5) as c_int;
                gr.table_select[2] = bitget(5) as c_int;
                gr.region0_count = bitget(4) as c_int;
                gr.region1_count = bitget(3) as c_int;
            }
            gr.preflag = bitget(1) as c_int;
            gr.scalefac_scale = bitget(1) as c_int;
            gr.count1table_select = bitget(1) as c_int;
            ch += 1;
        }
        igr += 1;
    }

    side_bytes
}

/*====================================================================*/
unsafe fn unpack_side_MPEG2(igr: c_int) -> c_int {
    let prot: c_int;
    let br_index: c_int;
    let mut ch: c_int;
    let side_bytes: c_int;

    (*pMP3Stream).id = bitget(1) as c_int;
    bitget(2);
    prot = bitget(1) as c_int;
    br_index = bitget(4) as c_int;
    (*pMP3Stream).sr_index = bitget(2) as c_int;
    (*pMP3Stream).pad = bitget(1) as c_int;
    bitget(1);
    side_info.mode = bitget(2) as c_int;
    side_info.mode_ext = bitget(2) as c_int;

    if side_info.mode != 1 {
        side_info.mode_ext = 0;
    }

    (*pMP3Stream).u.L3.ms_mode = side_info.mode_ext >> 1;
    (*pMP3Stream).u.L3.is_mode = side_info.mode_ext & 1;

    (*pMP3Stream).u.L3.crcbytes = 0;
    if prot != 0 {
        bitget(4);
    } else {
        bitget(20);
        (*pMP3Stream).u.L3.crcbytes = 2;
    }

    if br_index > 0 {
        if (*pMP3Stream).u.L3.mpeg25_flag == 0 {
            (*pMP3Stream).framebytes = 1440 * mp_br_tableL3[(*pMP3Stream).id as usize][br_index as usize]
                / mp_sr20_table[(*pMP3Stream).id as usize][(*pMP3Stream).sr_index as usize];
        } else {
            (*pMP3Stream).framebytes = 2880 * mp_br_tableL3[(*pMP3Stream).id as usize][br_index as usize]
                / mp_sr20_table[(*pMP3Stream).id as usize][(*pMP3Stream).sr_index as usize];
        }
    }
    side_info.main_data_begin = bitget(8) as c_int;
    if side_info.mode == 3 {
        side_info.private_bits = bitget(1) as c_int;
        (*pMP3Stream).u.L3.nchan = 1;
        side_bytes = 4 + 9;
    } else {
        side_info.private_bits = bitget(2) as c_int;
        (*pMP3Stream).u.L3.nchan = 2;
        side_bytes = 4 + 17;
    }
    side_info.scfsi[0] = 0;
    side_info.scfsi[1] = side_info.scfsi[0];

    ch = 0;
    while ch < (*pMP3Stream).u.L3.nchan {
        let gr = &mut side_info.gr[igr as usize][ch as usize];
        gr.part2_3_length = bitget(12) as c_int;
        gr.big_values = bitget(9) as c_int;
        gr.global_gain = bitget(8) as c_int + (*pMP3Stream).u.L3.gain_adjust;
        if (*pMP3Stream).u.L3.ms_mode != 0 {
            gr.global_gain -= 2;
        }
        gr.scalefac_compress = bitget(9) as c_int;
        gr.window_switching_flag = bitget(1) as c_int;
        if gr.window_switching_flag != 0 {
            gr.block_type = bitget(2) as c_int;
            gr.mixed_block_flag = bitget(1) as c_int;
            gr.table_select[0] = bitget(5) as c_int;
            gr.table_select[1] = bitget(5) as c_int;
            gr.subblock_gain[0] = bitget(3) as c_int;
            gr.subblock_gain[1] = bitget(3) as c_int;
            gr.subblock_gain[2] = bitget(3) as c_int;
            if gr.block_type == 2 {
                gr.region0_count = 6 - 1;
                gr.region1_count = 20 - (6 - 1);
            } else {
                gr.region0_count = 8 - 1;
                gr.region1_count = 20 - (8 - 1);
            }
        } else {
            gr.mixed_block_flag = 0;
            gr.block_type = 0;
            gr.table_select[0] = bitget(5) as c_int;
            gr.table_select[1] = bitget(5) as c_int;
            gr.table_select[2] = bitget(5) as c_int;
            gr.region0_count = bitget(4) as c_int;
            gr.region1_count = bitget(3) as c_int;
        }
        gr.preflag = 0;
        gr.scalefac_scale = bitget(1) as c_int;
        gr.count1table_select = bitget(1) as c_int;
        ch += 1;
    }

    side_bytes
}

/*-----------------------------------------------------------------*/
unsafe fn unpack_main(pcm: *mut c_uchar, igr: c_int) {
    let mut ch: c_int;
    let bit0: c_int;
    let mut n1: c_int;
    let mut n2: c_int;
    let mut n3: c_int;
    let mut n4: c_int;
    let nn2: c_int;
    let nn3: c_int;
    let nn4: c_int;
    let qbits: c_int;
    let mut m0: c_int;

    ch = 0;
    while ch < (*pMP3Stream).u.L3.nchan {
        bitget_init((*pMP3Stream).u.L3.buf.as_mut_ptr().add(((*pMP3Stream).u.L3.main_pos_bit >> 3) as usize));
        bit0 = (*pMP3Stream).u.L3.main_pos_bit & 7;
        if bit0 != 0 {
            bitget(bit0);
        }
        (*pMP3Stream).u.L3.main_pos_bit += side_info.gr[igr as usize][ch as usize].part2_3_length;
        bitget_init_end(
            (*pMP3Stream)
                .u
                .L3
                .buf
                .as_mut_ptr()
                .add((((*pMP3Stream).u.L3.main_pos_bit + 39) >> 3) as usize),
        );
        if (*pMP3Stream).id != 0 {
            unpack_sf_sub_MPEG1(
                ptr::addr_of_mut!(sf[igr as usize][ch as usize]),
                ptr::addr_of_mut!(side_info.gr[igr as usize][ch as usize]),
                side_info.scfsi[ch as usize],
                igr,
            );
        } else {
            unpack_sf_sub_MPEG2(
                ptr::addr_of_mut!(sf[igr as usize][ch as usize]),
                ptr::addr_of_mut!(side_info.gr[igr as usize][ch as usize]),
                (*pMP3Stream).u.L3.is_mode & ch,
                ptr::addr_of_mut!(is_sf_info),
            );
        }
        n1 = (*pMP3Stream).u.L3.sfBandIndex[0][side_info.gr[igr as usize][ch as usize].region0_count as usize];
        n2 = (*pMP3Stream).u.L3.sfBandIndex[0][(side_info.gr[igr as usize][ch as usize].region0_count
            + side_info.gr[igr as usize][ch as usize].region1_count
            + 1) as usize];
        n3 = side_info.gr[igr as usize][ch as usize].big_values;
        n3 = n3 + n3;

        if n3 > (*pMP3Stream).u.L3.band_limit {
            n3 = (*pMP3Stream).u.L3.band_limit;
        }
        if n2 > n3 {
            n2 = n3;
        }
        if n1 > n3 {
            n1 = n3;
        }
        nn3 = n3 - n2;
        nn2 = n2 - n1;
        unpack_huff(
            sample_ptr(ch, igr).cast::<[c_int; 2]>(),
            n1,
            side_info.gr[igr as usize][ch as usize].table_select[0],
        );
        unpack_huff(
            sample_ptr(ch, igr).add(n1 as usize).cast::<[c_int; 2]>(),
            nn2,
            side_info.gr[igr as usize][ch as usize].table_select[1],
        );
        unpack_huff(
            sample_ptr(ch, igr).add(n2 as usize).cast::<[c_int; 2]>(),
            nn3,
            side_info.gr[igr as usize][ch as usize].table_select[2],
        );
        qbits = side_info.gr[igr as usize][ch as usize].part2_3_length - (bitget_bits_used() - bit0);
        nn4 = unpack_huff_quad(
            sample_ptr(ch, igr).add(n3 as usize).cast::<[c_int; 4]>(),
            (*pMP3Stream).u.L3.band_limit - n3,
            qbits,
            side_info.gr[igr as usize][ch as usize].count1table_select,
        );
        n4 = n3 + nn4;
        nsamp[igr as usize][ch as usize] = n4;
        if side_info.gr[igr as usize][ch as usize].block_type == 2 {
            n4 = crate::min!(n4, (*pMP3Stream).u.L3.band_limit12);
        } else {
            n4 = crate::min!(n4, (*pMP3Stream).u.L3.band_limit21);
        }
        if n4 < 576 {
            ptr::write_bytes(sample_ptr(ch, igr).add(n4 as usize), 0, (576 - n4) as usize);
        }
        if bitdat.bs_ptr > bitdat.bs_ptr_end {
            ptr::write_bytes(sample_ptr(ch, igr), 0, 576);
        }
        ch += 1;
    }

    ch = 0;
    while ch < (*pMP3Stream).u.L3.nchan {
        dequant(
            sample_ptr(ch, igr),
            ptr::addr_of_mut!(nsamp[igr as usize][ch as usize]),
            ptr::addr_of_mut!(sf[igr as usize][ch as usize]),
            ptr::addr_of_mut!(side_info.gr[igr as usize][ch as usize]),
            ptr::addr_of_mut!(cb_info[igr as usize][ch as usize]),
            (*pMP3Stream).u.L3.ncbl_mixed,
        );
        ch += 1;
    }

    if (*pMP3Stream).u.L3.ms_mode != 0 {
        if (*pMP3Stream).u.L3.is_mode == 0 {
            m0 = nsamp[igr as usize][0];
            if m0 < nsamp[igr as usize][1] {
                m0 = nsamp[igr as usize][1];
            }
        } else {
            m0 = (*pMP3Stream).u.L3.sfBandIndex[cb_info[igr as usize][1].cbtype as usize]
                [cb_info[igr as usize][1].cbmax as usize];
        }
        ms_process(sample_float_ptr(0, igr).cast::<[f32; 1152]>(), m0);
    }

    if (*pMP3Stream).u.L3.is_mode != 0 {
        if (*pMP3Stream).id != 0 {
            is_process_MPEG1(
                sample_float_ptr(0, igr).cast::<[f32; 1152]>(),
                ptr::addr_of_mut!(sf[igr as usize][1]),
                ptr::addr_of_mut!(cb_info[igr as usize]).cast::<CB_INFO>(),
                nsamp[igr as usize][0],
                (*pMP3Stream).u.L3.ms_mode,
            );
        } else {
            is_process_MPEG2(
                sample_float_ptr(0, igr).cast::<[f32; 1152]>(),
                ptr::addr_of_mut!(sf[igr as usize][1]),
                ptr::addr_of_mut!(cb_info[igr as usize]).cast::<CB_INFO>(),
                ptr::addr_of_mut!(is_sf_info),
                nsamp[igr as usize][0],
                (*pMP3Stream).u.L3.ms_mode,
            );
        }
    }

    if side_info.mode_ext != 0 {
        if nsamp[igr as usize][0] < nsamp[igr as usize][1] {
            nsamp[igr as usize][0] = nsamp[igr as usize][1];
        } else {
            nsamp[igr as usize][1] = nsamp[igr as usize][0];
        }
    }

    ch = 0;
    while ch < (*pMP3Stream).u.L3.nchan {
        if cb_info[igr as usize][ch as usize].ncbl == 0 {
            ch += 1;
            continue;
        }
        if side_info.gr[igr as usize][ch as usize].mixed_block_flag != 0 {
            n1 = 1;
        } else {
            n1 = (nsamp[igr as usize][ch as usize] + 7) / 18;
        }
        if n1 > 31 {
            n1 = 31;
        }
        antialias(sample_float_ptr(ch, igr), n1);
        n1 = 18 * n1 + 8;
        if n1 > nsamp[igr as usize][ch as usize] {
            nsamp[igr as usize][ch as usize] = n1;
        }
        ch += 1;
    }

    call_xform(pcm, igr);
}

/*-----------------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn L3audio_decode(bs: *mut c_uchar, pcm: *mut c_uchar) -> IN_OUT {
    (*pMP3Stream).u.L3.decode_function.unwrap_unchecked()(bs, pcm)
}

/*--------------------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn L3audio_decode_MPEG1(bs: *mut c_uchar, pcm: *mut c_uchar) -> IN_OUT {
    let sync: c_int;
    let mut in_out: IN_OUT;
    let side_bytes: c_int;
    let nbytes: c_int;
    let padframebytes: c_int;

    bitget_init(bs);
    in_out.in_bytes = 0;
    in_out.out_bytes = 0;
    sync = bitget(12) as c_int;

    if sync != 0xFFF {
        return in_out;
    }

    side_bytes = unpack_side_MPEG1();
    padframebytes = (*pMP3Stream).framebytes + (*pMP3Stream).pad;

    if bs.add(padframebytes as usize) > gpNextByteAfterData {
        return in_out;
    }
    in_out.in_bytes = padframebytes;

    (*pMP3Stream).u.L3.buf_ptr0 = (*pMP3Stream).u.L3.buf_ptr1 - side_info.main_data_begin;
    if (*pMP3Stream).u.L3.buf_ptr1 > BUF_TRIGGER as c_int {
        ptr::copy(
            (*pMP3Stream).u.L3.buf.as_ptr().add((*pMP3Stream).u.L3.buf_ptr0 as usize),
            (*pMP3Stream).u.L3.buf.as_mut_ptr(),
            side_info.main_data_begin as usize,
        );
        (*pMP3Stream).u.L3.buf_ptr0 = 0;
        (*pMP3Stream).u.L3.buf_ptr1 = side_info.main_data_begin;
    }
    nbytes = padframebytes - side_bytes - (*pMP3Stream).u.L3.crcbytes;

    if nbytes < 0 || nbytes > NBUF as c_int {
        in_out.in_bytes = 0;
        return in_out;
    }

    if bFastEstimateOnly != 0 {
        in_out.out_bytes = (*pMP3Stream).outbytes;
        return in_out;
    }

    ptr::copy(
        bs.add((side_bytes + (*pMP3Stream).u.L3.crcbytes) as usize),
        (*pMP3Stream).u.L3.buf.as_mut_ptr().add((*pMP3Stream).u.L3.buf_ptr1 as usize),
        nbytes as usize,
    );
    (*pMP3Stream).u.L3.buf_ptr1 += nbytes;

    if (*pMP3Stream).u.L3.buf_ptr0 >= 0 {
        (*pMP3Stream).u.L3.main_pos_bit = (*pMP3Stream).u.L3.buf_ptr0 << 3;
        unpack_main(pcm, 0);
        unpack_main(pcm.add((*pMP3Stream).u.L3.half_outbytes as usize), 1);
        in_out.out_bytes = (*pMP3Stream).outbytes;
    } else {
        ptr::write_bytes(pcm, (*pMP3Stream).u.L3.zero_level_pcm as u8, (*pMP3Stream).outbytes as usize);
        in_out.out_bytes = (*pMP3Stream).outbytes;
    }

    in_out
}

/*--------------------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn L3audio_decode_MPEG2(bs: *mut c_uchar, pcm: *mut c_uchar) -> IN_OUT {
    let sync: c_int;
    let mut in_out: IN_OUT;
    let side_bytes: c_int;
    let nbytes: c_int;
    static mut igr: c_int = 0;
    let padframebytes: c_int;

    bitget_init(bs);
    in_out.in_bytes = 0;
    in_out.out_bytes = 0;
    sync = bitget(12) as c_int;

    (*pMP3Stream).u.L3.mpeg25_flag = 0;
    if sync != 0xFFF {
        (*pMP3Stream).u.L3.mpeg25_flag = 1;
        if sync != 0xFFE {
            return in_out;
        }
    }

    side_bytes = unpack_side_MPEG2(igr);
    padframebytes = (*pMP3Stream).framebytes + (*pMP3Stream).pad;
    in_out.in_bytes = padframebytes;

    (*pMP3Stream).u.L3.buf_ptr0 = (*pMP3Stream).u.L3.buf_ptr1 - side_info.main_data_begin;
    if (*pMP3Stream).u.L3.buf_ptr1 > BUF_TRIGGER as c_int {
        ptr::copy(
            (*pMP3Stream).u.L3.buf.as_ptr().add((*pMP3Stream).u.L3.buf_ptr0 as usize),
            (*pMP3Stream).u.L3.buf.as_mut_ptr(),
            side_info.main_data_begin as usize,
        );
        (*pMP3Stream).u.L3.buf_ptr0 = 0;
        (*pMP3Stream).u.L3.buf_ptr1 = side_info.main_data_begin;
    }
    nbytes = padframebytes - side_bytes - (*pMP3Stream).u.L3.crcbytes;
    if nbytes < 0 || nbytes > NBUF as c_int {
        in_out.in_bytes = 0;
        return in_out;
    }

    if bFastEstimateOnly != 0 {
        in_out.out_bytes = (*pMP3Stream).outbytes;
        return in_out;
    }

    ptr::copy(
        bs.add((side_bytes + (*pMP3Stream).u.L3.crcbytes) as usize),
        (*pMP3Stream).u.L3.buf.as_mut_ptr().add((*pMP3Stream).u.L3.buf_ptr1 as usize),
        nbytes as usize,
    );
    (*pMP3Stream).u.L3.buf_ptr1 += nbytes;

    if (*pMP3Stream).u.L3.buf_ptr0 >= 0 {
        (*pMP3Stream).u.L3.main_pos_bit = (*pMP3Stream).u.L3.buf_ptr0 << 3;
        unpack_main(pcm, igr);
        in_out.out_bytes = (*pMP3Stream).outbytes;
    } else {
        ptr::write_bytes(pcm, (*pMP3Stream).u.L3.zero_level_pcm as u8, (*pMP3Stream).outbytes as usize);
        in_out.out_bytes = (*pMP3Stream).outbytes;
    }

    igr = igr ^ 1;
    in_out
}

/*--------------------------------------------------------------------*/
static sr_table: [c_int; 8] = [22050, 24000, 16000, 1, 44100, 48000, 32000, 1];

#[repr(C)]
#[derive(Clone, Copy)]
struct SFBAND_INDEX {
    l: [c_int; 23],
    s: [c_int; 14],
}

static sfBandIndexTable: [[SFBAND_INDEX; 3]; 3] = [
    [
        SFBAND_INDEX {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 24, 32, 42, 56, 74, 100, 132, 174, 192],
        },
        SFBAND_INDEX {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 114, 136, 162, 194, 232, 278, 332, 394, 464, 540, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 136, 180, 192],
        },
        SFBAND_INDEX {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 134, 174, 192],
        },
    ],
    [
        SFBAND_INDEX {
            l: [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 52, 62, 74, 90, 110, 134, 162, 196, 238, 288, 342, 418, 576],
            s: [0, 4, 8, 12, 16, 22, 30, 40, 52, 66, 84, 106, 136, 192],
        },
        SFBAND_INDEX {
            l: [0, 4, 8, 12, 16, 20, 24, 30, 36, 42, 50, 60, 72, 88, 106, 128, 156, 190, 230, 276, 330, 384, 576],
            s: [0, 4, 8, 12, 16, 22, 28, 38, 50, 64, 80, 100, 126, 192],
        },
        SFBAND_INDEX {
            l: [0, 4, 8, 12, 16, 20, 24, 30, 36, 44, 54, 66, 82, 102, 126, 156, 194, 240, 296, 364, 448, 550, 576],
            s: [0, 4, 8, 12, 16, 22, 30, 42, 58, 78, 104, 138, 180, 192],
        },
    ],
    [
        SFBAND_INDEX {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 134, 174, 192],
        },
        SFBAND_INDEX {
            l: [0, 6, 12, 18, 24, 30, 36, 44, 54, 66, 80, 96, 116, 140, 168, 200, 238, 284, 336, 396, 464, 522, 576],
            s: [0, 4, 8, 12, 18, 26, 36, 48, 62, 80, 104, 134, 174, 192],
        },
        SFBAND_INDEX {
            l: [0, 12, 24, 36, 48, 60, 72, 88, 108, 132, 160, 192, 232, 280, 336, 400, 476, 566, 568, 570, 572, 574, 576],
            s: [0, 8, 16, 24, 36, 52, 72, 96, 124, 160, 162, 164, 166, 192],
        },
    ],
];

static sbt_table: [[[SBT_FUNCTION; 2]; 3]; 2] = [
    [
        [Some(sbt_mono_L3), Some(sbt_dual_L3)],
        [Some(sbt16_mono_L3), Some(sbt16_dual_L3)],
        [Some(sbt8_mono_L3), Some(sbt8_dual_L3)],
    ],
    [
        [Some(sbtB_mono_L3), Some(sbtB_dual_L3)],
        [Some(sbtB16_mono_L3), Some(sbtB16_dual_L3)],
        [Some(sbtB8_mono_L3), Some(sbtB8_dual_L3)],
    ],
];

static xform_table: [XFORM_FUNCTION; 5] = [
    Some(Xform_mono),
    Some(Xform_dual),
    Some(Xform_dual_mono),
    Some(Xform_mono),
    Some(Xform_dual_right),
];

/*---------------------------------------------------------*/
#[unsafe(no_mangle)]
pub unsafe extern "C" fn L3audio_decode_init(
    h: *mut MPEG_HEAD,
    framebytes_arg: c_int,
    mut reduction_code: c_int,
    transform_code: c_int,
    mut convert_code: c_int,
    mut freq_limit: c_int,
) -> c_int {
    let mut i: c_int;
    let mut j: c_int;
    let mut k: c_int;
    let mut samprate: c_int;
    let mut limit: c_int;
    let mut bit_code: c_int;
    let mut out_chans: c_int;

    (*pMP3Stream).u.L3.buf_ptr0 = 0;
    (*pMP3Stream).u.L3.buf_ptr1 = 0;

    if (*h).option != 1 {
        return 0;
    }

    if (*h).id != 0 {
        (*pMP3Stream).u.L3.ncbl_mixed = 8;
    } else {
        (*pMP3Stream).u.L3.ncbl_mixed = 6;
    }

    (*pMP3Stream).framebytes = framebytes_arg;

    let _ = transform_code;
    bit_code = 0;
    if (convert_code & 8) != 0 {
        bit_code = 1;
    }
    convert_code = convert_code & 3;
    if reduction_code < 0 {
        reduction_code = 0;
    }
    if reduction_code > 2 {
        reduction_code = 2;
    }
    if freq_limit < 1000 {
        freq_limit = 1000;
    }

    samprate = sr_table[(4 * (*h).id + (*h).sr_index) as usize];
    if ((*h).sync & 1) == 0 {
        samprate = samprate / 2;
    }
    (*pMP3Stream).nsb_limit = ((freq_limit as i64 * 64 + (samprate / 2) as i64) / samprate as i64) as c_int;
    limit = 32 >> reduction_code;
    if limit > 8 {
        limit -= 1;
    }
    if (*pMP3Stream).nsb_limit > limit {
        (*pMP3Stream).nsb_limit = limit;
    }
    limit = 18 * (*pMP3Stream).nsb_limit;

    k = (*h).id;
    if ((*h).sync & 1) == 0 {
        k = 2;
    }

    if k == 1 {
        (*pMP3Stream).u.L3.band_limit12 = 3 * sfBandIndexTable[k as usize][(*h).sr_index as usize].s[13];
        (*pMP3Stream).u.L3.band_limit21 = sfBandIndexTable[k as usize][(*h).sr_index as usize].l[22];
        (*pMP3Stream).u.L3.band_limit = (*pMP3Stream).u.L3.band_limit21;
    } else {
        (*pMP3Stream).u.L3.band_limit12 = 3 * sfBandIndexTable[k as usize][(*h).sr_index as usize].s[12];
        (*pMP3Stream).u.L3.band_limit21 = sfBandIndexTable[k as usize][(*h).sr_index as usize].l[21];
        (*pMP3Stream).u.L3.band_limit = (*pMP3Stream).u.L3.band_limit21;
    }
    (*pMP3Stream).u.L3.band_limit += 8;
    if (*pMP3Stream).u.L3.band_limit > limit {
        (*pMP3Stream).u.L3.band_limit = limit;
    }

    if (*pMP3Stream).u.L3.band_limit21 > (*pMP3Stream).u.L3.band_limit {
        (*pMP3Stream).u.L3.band_limit21 = (*pMP3Stream).u.L3.band_limit;
    }
    if (*pMP3Stream).u.L3.band_limit12 > (*pMP3Stream).u.L3.band_limit {
        (*pMP3Stream).u.L3.band_limit12 = (*pMP3Stream).u.L3.band_limit;
    }

    (*pMP3Stream).u.L3.band_limit_nsb = ((*pMP3Stream).u.L3.band_limit + 17) / 18;
    (*pMP3Stream).u.L3.gain_adjust = 0;
    if ((*h).mode != 3) && (convert_code == 1) {
        (*pMP3Stream).u.L3.gain_adjust = -4;
    }

    (*pMP3Stream).outvalues = 1152 >> reduction_code;
    if (*h).id == 0 {
        (*pMP3Stream).outvalues /= 2;
    }

    out_chans = 2;
    if (*h).mode == 3 {
        out_chans = 1;
    }
    if convert_code != 0 {
        out_chans = 1;
    }

    (*pMP3Stream).u.L3.sbt_L3 = sbt_table[bit_code as usize][reduction_code as usize][(out_chans - 1) as usize];
    k = 1 + convert_code;
    if (*h).mode == 3 {
        k = 0;
    }
    (*pMP3Stream).u.L3.Xform = xform_table[k as usize];

    (*pMP3Stream).outvalues *= out_chans;

    if bit_code != 0 {
        (*pMP3Stream).outbytes = (*pMP3Stream).outvalues;
    } else {
        (*pMP3Stream).outbytes = core::mem::size_of::<c_short>() as c_int * (*pMP3Stream).outvalues;
    }

    if bit_code != 0 {
        (*pMP3Stream).u.L3.zero_level_pcm = 128;
    } else {
        (*pMP3Stream).u.L3.zero_level_pcm = 0;
    }

    decinfo.channels = out_chans;
    decinfo.outvalues = (*pMP3Stream).outvalues;
    decinfo.samprate = (samprate >> reduction_code) as _;
    if bit_code != 0 {
        decinfo.bits = 8;
    } else {
        decinfo.bits = core::mem::size_of::<c_short>() as c_int * 8;
    }

    decinfo.framebytes = (*pMP3Stream).framebytes;
    decinfo.r#type = 0;

    (*pMP3Stream).u.L3.half_outbytes = (*pMP3Stream).outbytes / 2;

    k = (*h).id;
    if ((*h).sync & 1) == 0 {
        k = 2;
    }

    i = 0;
    while i < 22 {
        (*pMP3Stream).u.L3.sfBandIndex[0][i as usize] = sfBandIndexTable[k as usize][(*h).sr_index as usize].l[(i + 1) as usize];
        i += 1;
    }
    i = 0;
    while i < 13 {
        (*pMP3Stream).u.L3.sfBandIndex[1][i as usize] = 3 * sfBandIndexTable[k as usize][(*h).sr_index as usize].s[(i + 1) as usize];
        i += 1;
    }
    i = 0;
    while i < 22 {
        (*pMP3Stream).u.L3.nBand[0][i as usize] = sfBandIndexTable[k as usize][(*h).sr_index as usize].l[(i + 1) as usize]
            - sfBandIndexTable[k as usize][(*h).sr_index as usize].l[i as usize];
        i += 1;
    }
    i = 0;
    while i < 13 {
        (*pMP3Stream).u.L3.nBand[1][i as usize] = sfBandIndexTable[k as usize][(*h).sr_index as usize].s[(i + 1) as usize]
            - sfBandIndexTable[k as usize][(*h).sr_index as usize].s[i as usize];
        i += 1;
    }

    L3table_init();
    msis_init();
    sbt_init();

    i = 0;
    while i < 576 {
        yout[i as usize] = 0.0f32;
        i += 1;
    }
    j = 0;
    while j < 2 {
        k = 0;
        while k < 2 {
            i = 0;
            while i < 576 {
                (*pMP3Stream).u.L3.sample[j as usize][k as usize][i as usize].x = 0.0f32;
                (*pMP3Stream).u.L3.sample[j as usize][k as usize][i as usize].s = 0;
                i += 1;
            }
            k += 1;
        }
        j += 1;
    }

    if (*h).id == 1 {
        (*pMP3Stream).u.L3.decode_function = Some(L3audio_decode_MPEG1 as unsafe extern "C" fn(*mut c_uchar, *mut c_uchar) -> IN_OUT);
    } else {
        (*pMP3Stream).u.L3.decode_function = Some(L3audio_decode_MPEG2 as unsafe extern "C" fn(*mut c_uchar, *mut c_uchar) -> IN_OUT);
    }

    1
}
