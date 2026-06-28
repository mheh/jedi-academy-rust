//! Mechanical port of `codemp/png/png.cpp`.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_assignments)]

use core::ffi::{c_char, c_int, c_void, CStr};
use core::mem::{size_of, MaybeUninit};
use core::ptr::{null, null_mut};

use crate::codemp::png::png_h::{
    byte, png_ihdr_t, png_image_t, ulong, MAX_PNG_DEPTH, MAX_PNG_WIDTH, PNG_FILTER_VALUE_AVG,
    PNG_FILTER_VALUE_NONE, PNG_FILTER_VALUE_PAETH, PNG_FILTER_VALUE_SUB, PNG_FILTER_VALUE_UP,
    PNG_IDAT, PNG_IEND, PNG_IHDR, PNG_tEXt,
};
use crate::codemp::zlib32::zip_h::{
    crc32, deflate, deflateEnd, deflateInit, inflate, inflateEnd, inflateInit, z_stream, EFlush,
    ELevel, EStatus,
};
use crate::ffi::types::{fileHandle_t, qboolean, QFALSE, QTRUE};

// Error returns

const PNG_ERROR_OK: c_int = 0;
const PNG_ERROR_DECOMP: c_int = 1;
const PNG_ERROR_COMP: c_int = 2;
const PNG_ERROR_MEMORY: c_int = 3;
const PNG_ERROR_NOSIG: c_int = 4;
const PNG_ERROR_TOO_SMALL: c_int = 5;
const PNG_ERROR_WNP2: c_int = 6;
const PNG_ERROR_HNP2: c_int = 7;
const PNG_ERROR_NOT_TC: c_int = 8;
const PNG_ERROR_INV_FIL: c_int = 9;
const PNG_ERROR_FAILED_CRC: c_int = 10;
const PNG_ERROR_CREATE_FAIL: c_int = 11;
const PNG_ERROR_WRITE: c_int = 12;
const PNG_ERROR_NOT_PALETTE: c_int = 13;
const PNG_ERROR_NOT8BIT: c_int = 14;
const PNG_ERROR_TOO_LARGE: c_int = 15;

static mut png_error: c_int = PNG_ERROR_OK;

static png_signature: [byte; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
static png_copyright: [c_char; 35] = [
    b'C' as c_char,
    b'o' as c_char,
    b'p' as c_char,
    b'y' as c_char,
    b'r' as c_char,
    b'i' as c_char,
    b'g' as c_char,
    b'h' as c_char,
    b't' as c_char,
    0,
    b'R' as c_char,
    b'a' as c_char,
    b'v' as c_char,
    b'e' as c_char,
    b'n' as c_char,
    b' ' as c_char,
    b'S' as c_char,
    b'o' as c_char,
    b'f' as c_char,
    b't' as c_char,
    b'w' as c_char,
    b'a' as c_char,
    b'r' as c_char,
    b'e' as c_char,
    b' ' as c_char,
    b'I' as c_char,
    b'n' as c_char,
    b'c' as c_char,
    b'.' as c_char,
    b' ' as c_char,
    b'2' as c_char,
    b'0' as c_char,
    b'0' as c_char,
    b'1' as c_char,
    0,
];
static png_errors: [&CStr; 16] = [
    c"OK.",
    c"Error decompressing image data.",
    c"Error compressing image data.",
    c"Error allocating memory.",
    c"PNG signature not found.",
    c"Image is too small to load.",
    c"Width is not a power of two.",
    c"Height is not a power of two.",
    c"Image is not 24 or 32 bit.",
    c"Invalid filter or compression type.",
    c"Failed CRC check.",
    c"Could not create file.",
    c"Error writing to file.",
    c"Image is not indexed colour.",
    c"Image does not have 8 bits per sample.",
    c"Image is too large",
];

// `TAG_TEMP_PNG` is generated from `qcommon/tags.h` by TAGDEF. Non-Xbox order: TEMP_PNG == 32.
const TAG_TEMP_PNG: c_int = 32;

unsafe extern "C" {
    fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;

    fn Z_Malloc(iSize: c_int, eTag: c_int, bZeroit: qboolean) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);

    fn FS_FOpenFileWrite(qpath: *const c_char) -> fileHandle_t;
    fn FS_Write(buffer: *const c_void, len: c_int, f: fileHandle_t) -> c_int;
    fn FS_FCloseFile(f: fileHandle_t);
    fn FS_ReadFile(qpath: *const c_char, buffer: *mut *mut c_void) -> c_int;
    fn FS_FreeFile(buffer: *mut c_void);

    fn Com_Printf(fmt: *const c_char, ...);
}

#[inline]
fn BigLong(l: c_int) -> c_int {
    (((l as u32 & 0xff) << 24)
        | ((l as u32 & 0xff00) << 8)
        | ((l as u32 >> 8) & 0xff00)
        | ((l as u32 >> 24) & 0xff)) as c_int
}

#[inline]
unsafe fn read_ulong(p: *const byte) -> ulong {
    core::ptr::read_unaligned(p as *const ulong)
}

// Gets the error string for a failed PNG operation

pub unsafe extern "C" fn PNG_GetError() -> *const c_char {
    png_errors[png_error as usize].as_ptr()
}

// Create a header chunk

pub unsafe extern "C" fn PNG_CreateHeader(
    header: *mut png_ihdr_t,
    width: c_int,
    height: c_int,
    bytedepth: c_int,
) {
    (*header).width = BigLong(width) as u32 as ulong;
    (*header).height = BigLong(height) as u32 as ulong;
    (*header).bitdepth = 8;

    if bytedepth == 3 {
        (*header).colortype = 2;
    }
    if bytedepth == 4 {
        (*header).colortype = 6;
    }
    (*header).compression = 0;
    (*header).filter = 0;
    (*header).interlace = 0;
}

// Processes the header chunk and checks to see if all the data is valid

pub unsafe extern "C" fn PNG_HandleIHDR(data: *const byte, image: *mut png_image_t) -> bool {
    let ihdr: *mut png_ihdr_t = data as *mut png_ihdr_t;

    (*image).width = BigLong((*ihdr).width as c_int) as u32 as ulong;
    (*image).height = BigLong((*ihdr).height as c_int) as u32 as ulong;

    // Make sure image is a reasonable size
    if ((*image).width < 2) || ((*image).height < 2) {
        png_error = PNG_ERROR_TOO_SMALL;
        return false;
    }
    if (*image).width > MAX_PNG_WIDTH as ulong {
        png_error = PNG_ERROR_TOO_LARGE;
        return false;
    }
    if (*ihdr).bitdepth != 8 {
        png_error = PNG_ERROR_NOT8BIT;
        return false;
    }
    // Check for non power of two size (but not for data files)
    if (*image).isimage {
        if ((*image).width & ((*image).width - 1)) != 0 {
            png_error = PNG_ERROR_WNP2;
            return false;
        }
        if ((*image).height & ((*image).height - 1)) != 0 {
            png_error = PNG_ERROR_HNP2;
            return false;
        }
    }
    // Make sure we have a 24 or 32 bit image (for images)
    if (*image).isimage {
        if ((*ihdr).colortype != 2) && ((*ihdr).colortype != 6) {
            png_error = PNG_ERROR_NOT_TC;
            return false;
        }
    }
    // Make sure we have an 8 bit grayscale image for data files
    if !(*image).isimage {
        if (*ihdr).colortype != 0 && ((*ihdr).colortype != 3) {
            png_error = PNG_ERROR_NOT_PALETTE;
            return false;
        }
    }
    // Make sure we aren't using any wacky compression or filter algos
    if (*ihdr).compression != 0 || (*ihdr).filter != 0 {
        png_error = PNG_ERROR_INV_FIL;
        return false;
    }
    // Extract the data we need
    if (*ihdr).colortype == 0 || ((*ihdr).colortype == 3) {
        (*image).bytedepth = 1;
    }
    if (*ihdr).colortype == 2 {
        (*image).bytedepth = 3;
    }
    if (*ihdr).colortype == 6 {
        (*image).bytedepth = 4;
    }
    true
}

// Filter a row of data

pub unsafe extern "C" fn PNG_Filter(
    mut out: *mut byte,
    filter: byte,
    mut in_: *const byte,
    mut lastline: *const byte,
    rowbytes: ulong,
    bpp: ulong,
) {
    let mut i: ulong;

    match filter as c_int {
        PNG_FILTER_VALUE_NONE => {
            memcpy(out as *mut c_void, in_ as *const c_void, rowbytes as usize);
        }
        PNG_FILTER_VALUE_SUB => {
            i = 0;
            while i < bpp {
                *out = *in_;
                out = out.add(1);
                in_ = in_.add(1);
                i += 1;
            }
            i = bpp;
            while i < rowbytes {
                *out = (*in_).wrapping_sub(*in_.sub(bpp as usize));
                out = out.add(1);
                in_ = in_.add(1);
                i += 1;
            }
        }
        PNG_FILTER_VALUE_UP => {
            i = 0;
            while i < rowbytes {
                if !lastline.is_null() {
                    *out = (*in_).wrapping_sub(*lastline);
                    out = out.add(1);
                    in_ = in_.add(1);
                    lastline = lastline.add(1);
                } else {
                    *out = *in_;
                    out = out.add(1);
                    in_ = in_.add(1);
                }
                i += 1;
            }
        }
        PNG_FILTER_VALUE_AVG => {
            i = 0;
            while i < bpp {
                if !lastline.is_null() {
                    *out = (*in_).wrapping_sub(*lastline >> 1);
                    out = out.add(1);
                    in_ = in_.add(1);
                    lastline = lastline.add(1);
                } else {
                    *out = *in_;
                    out = out.add(1);
                    in_ = in_.add(1);
                }
                i += 1;
            }
            i = bpp;
            while i < rowbytes {
                if !lastline.is_null() {
                    *out = (*in_).wrapping_sub(((*lastline as c_int + *in_.sub(bpp as usize) as c_int) >> 1) as byte);
                    out = out.add(1);
                    lastline = lastline.add(1);
                } else {
                    *out = (*in_).wrapping_sub(*in_.sub(bpp as usize) >> 1);
                    out = out.add(1);
                }
                in_ = in_.add(1);
                i += 1;
            }
        }
        PNG_FILTER_VALUE_PAETH => {
            let mut a: c_int;
            let mut b: c_int;
            let mut c: c_int;
            let mut pa: c_int;
            let mut pb: c_int;
            let mut pc: c_int;
            let mut p: c_int;

            i = 0;
            while i < bpp {
                if !lastline.is_null() {
                    *out = (*in_).wrapping_sub(*lastline);
                    out = out.add(1);
                    in_ = in_.add(1);
                    lastline = lastline.add(1);
                } else {
                    *out = *in_;
                    out = out.add(1);
                    in_ = in_.add(1);
                }
                i += 1;
            }
            i = bpp;
            while i < rowbytes {
                a = *in_.sub(bpp as usize) as c_int;
                c = 0;
                b = 0;
                if !lastline.is_null() {
                    c = *lastline.sub(bpp as usize) as c_int;
                    b = *lastline as c_int;
                    lastline = lastline.add(1);
                }

                p = b - c;
                pc = a - c;

                pa = if p < 0 { -p } else { p };
                pb = if pc < 0 { -pc } else { pc };
                pc = if (p + pc) < 0 { -(p + pc) } else { p + pc };

                p = if pa <= pb && pa <= pc {
                    a
                } else if pb <= pc {
                    b
                } else {
                    c
                };

                *out = (*in_).wrapping_sub(p as byte);
                out = out.add(1);
                in_ = in_.add(1);
                i += 1;
            }
        }
        _ => {}
    }
}

// Unfilters a row of data

pub unsafe extern "C" fn PNG_Unfilter(
    mut out: *mut byte,
    filter: byte,
    mut lastline: *const byte,
    rowbytes: ulong,
    bpp: ulong,
) {
    let mut i: ulong;

    match filter as c_int {
        PNG_FILTER_VALUE_NONE => {}
        PNG_FILTER_VALUE_SUB => {
            out = out.add(bpp as usize);
            i = bpp;
            while i < rowbytes {
                *out = (*out).wrapping_add(*out.sub(bpp as usize));
                out = out.add(1);
                i += 1;
            }
        }
        PNG_FILTER_VALUE_UP => {
            i = 0;
            while i < rowbytes {
                if !lastline.is_null() {
                    *out = (*out).wrapping_add(*lastline);
                    lastline = lastline.add(1);
                }
                out = out.add(1);
                i += 1;
            }
        }
        PNG_FILTER_VALUE_AVG => {
            i = 0;
            while i < bpp {
                if !lastline.is_null() {
                    *out = (*out).wrapping_add(*lastline >> 1);
                    lastline = lastline.add(1);
                }
                out = out.add(1);
                i += 1;
            }
            i = bpp;
            while i < rowbytes {
                if !lastline.is_null() {
                    *out = (*out).wrapping_add(((*lastline as c_int + *out.sub(bpp as usize) as c_int) >> 1) as byte);
                    lastline = lastline.add(1);
                } else {
                    *out = (*out).wrapping_add(*out.sub(bpp as usize) >> 1);
                }
                out = out.add(1);
                i += 1;
            }
        }
        PNG_FILTER_VALUE_PAETH => {
            let mut a: c_int;
            let mut b: c_int;
            let mut c: c_int;
            let mut pa: c_int;
            let mut pb: c_int;
            let mut pc: c_int;
            let mut p: c_int;

            i = 0;
            while i < bpp {
                if !lastline.is_null() {
                    *out = (*out).wrapping_add(*lastline);
                    lastline = lastline.add(1);
                }
                out = out.add(1);
                i += 1;
            }
            i = bpp;
            while i < rowbytes {
                a = *out.sub(bpp as usize) as c_int;
                c = 0;
                b = 0;
                if !lastline.is_null() {
                    c = *lastline.sub(bpp as usize) as c_int;
                    b = *lastline as c_int;
                    lastline = lastline.add(1);
                }
                p = b - c;
                pc = a - c;

                pa = if p < 0 { -p } else { p };
                pb = if pc < 0 { -pc } else { pc };
                pc = if (p + pc) < 0 { -(p + pc) } else { p + pc };

                p = if pa <= pb && pa <= pc {
                    a
                } else if pb <= pc {
                    b
                } else {
                    c
                };

                *out = (*out).wrapping_add(p as byte);
                out = out.add(1);
                i += 1;
            }
        }
        _ => {}
    }
}

// Pack up the image data line by line

pub unsafe extern "C" fn PNG_Pack(
    out: *mut byte,
    size: *mut ulong,
    maxsize: ulong,
    data: *mut byte,
    width: c_int,
    height: c_int,
    bytedepth: c_int,
) -> bool {
    let mut zdata: z_stream = MaybeUninit::zeroed().assume_init();
    let rowbytes: ulong;
    let mut y: ulong;
    let mut lastline: *const byte;
    let mut source: *const byte;
    // Storage for filter type and filtered row
    let mut workline: [(byte); (MAX_PNG_WIDTH as usize * MAX_PNG_DEPTH as usize) + 1] =
        [0; (MAX_PNG_WIDTH as usize * MAX_PNG_DEPTH as usize) + 1];

    // Number of bytes per row
    rowbytes = (width * bytedepth) as ulong;

    memset(
        &mut zdata as *mut z_stream as *mut c_void,
        0,
        size_of::<z_stream>(),
    );
    if deflateInit(&mut zdata, ELevel::Z_FAST_COMPRESSION_HIGH, 0) != EStatus::Z_OK {
        png_error = PNG_ERROR_COMP;
        return false;
    }

    zdata.next_out = out;
    zdata.avail_out = maxsize;

    lastline = null();
    source = data.add(((height - 1) as ulong * rowbytes) as usize);
    y = 0;
    while y < height as ulong {
        // Refilter using the most compressable filter algo
        // Assume paeth to speed things up
        workline[0] = PNG_FILTER_VALUE_PAETH as byte;
        PNG_Filter(
            workline.as_mut_ptr().add(1),
            PNG_FILTER_VALUE_PAETH as byte,
            source,
            lastline,
            rowbytes,
            bytedepth as ulong,
        );

        zdata.next_in = workline.as_mut_ptr();
        zdata.avail_in = rowbytes + 1;
        if deflate(&mut zdata, EFlush::Z_SYNC_FLUSH) != EStatus::Z_OK {
            deflateEnd(&mut zdata);
            png_error = PNG_ERROR_COMP;
            return false;
        }
        lastline = source;
        source = source.sub(rowbytes as usize);
        y += 1;
    }
    if deflate(&mut zdata, EFlush::Z_FINISH) != EStatus::Z_STREAM_END {
        png_error = PNG_ERROR_COMP;
        return false;
    }
    *size = zdata.total_out;
    deflateEnd(&mut zdata);
    true
}

// Unpack the image data, line by line

pub unsafe extern "C" fn PNG_Unpack(
    data: *const byte,
    datasize: ulong,
    image: *mut png_image_t,
) -> bool {
    let rowbytes: ulong;
    let mut zerror: EStatus;
    let mut y: ulong;
    let mut filter: byte = 0;
    let mut zdata: z_stream = MaybeUninit::zeroed().assume_init();
    let mut lastline: *mut byte;
    let mut out: *mut byte;

    //	MD_PushTag(TAG_ZIP_TEMP);

    memset(
        &mut zdata as *mut z_stream as *mut c_void,
        0,
        size_of::<z_stream>(),
    );
    if inflateInit(&mut zdata, EFlush::Z_SYNC_FLUSH, 0) != EStatus::Z_OK {
        png_error = PNG_ERROR_DECOMP;
        //		MD_PopTag();
        return false;
    }
    zdata.next_in = data as *mut byte;
    zdata.avail_in = datasize;

    rowbytes = (*image).width * (*image).bytedepth;

    lastline = null_mut();
    out = (*image).data;
    y = 0;
    while y < (*image).height {
        // Inflate a row of data
        zdata.next_out = &mut filter;
        zdata.avail_out = 1;
        if inflate(&mut zdata) != EStatus::Z_OK {
            inflateEnd(&mut zdata);
            png_error = PNG_ERROR_DECOMP;
            //			MD_PopTag();
            return false;
        }
        zdata.next_out = out;
        zdata.avail_out = rowbytes;
        zerror = inflate(&mut zdata);
        if (zerror != EStatus::Z_OK) && (zerror != EStatus::Z_STREAM_END) {
            inflateEnd(&mut zdata);
            png_error = PNG_ERROR_DECOMP;
            //			MD_PopTag();
            return false;
        }

        // Unfilter a row of data
        PNG_Unfilter(out, filter, lastline, rowbytes, (*image).bytedepth);

        lastline = out;
        out = out.add(rowbytes as usize);
        y += 1;
    }
    inflateEnd(&mut zdata);
    //	MD_PopTag();
    true
}

// Scan through all chunks and process each one

pub unsafe extern "C" fn PNG_Load(
    mut data: *const byte,
    datasize: ulong,
    image: *mut png_image_t,
) -> bool {
    let mut moredata: bool;
    let mut next: *const byte;
    let mut workspace: *mut byte;
    let mut work: *mut byte;
    let mut length: ulong;
    let mut type_: ulong;
    let mut crc: ulong;
    let mut totallength: ulong;

    png_error = PNG_ERROR_OK;

    if memcmp(
        data as *const c_void,
        png_signature.as_ptr() as *const c_void,
        size_of::<[byte; 8]>(),
    ) != 0
    {
        png_error = PNG_ERROR_NOSIG;
        return false;
    }
    data = data.add(size_of::<[byte; 8]>());

    workspace = Z_Malloc(datasize as c_int, TAG_TEMP_PNG, QFALSE) as *mut byte;
    work = workspace;
    totallength = 0;

    moredata = true;
    while moredata {
        length = BigLong(read_ulong(data) as c_int) as u32 as ulong;
        data = data.add(size_of::<ulong>());

        type_ = BigLong(read_ulong(data) as c_int) as u32 as ulong;
        let crcbase: *const byte = data;
        data = data.add(size_of::<ulong>());

        // CRC checksum location
        next = data.add(length as usize + size_of::<ulong>());

        // CRC checksum includes header field
        crc = crc32(0, crcbase, length + size_of::<ulong>() as ulong);
        if crc != BigLong(read_ulong(next.sub(4)) as c_int) as u32 as ulong {
            if !(*image).data.is_null() {
                Z_Free((*image).data as *mut c_void);
                (*image).data = null_mut();
            }
            Z_Free(workspace as *mut c_void);
            png_error = PNG_ERROR_FAILED_CRC;
            return false;
        }
        match type_ as c_int {
            PNG_IHDR => {
                if !PNG_HandleIHDR(data, image) {
                    Z_Free(workspace as *mut c_void);
                    return false;
                }
                (*image).data = Z_Malloc(
                    ((*image).width * (*image).height * (*image).bytedepth) as c_int,
                    TAG_TEMP_PNG,
                    QFALSE,
                ) as *mut byte;
            }
            PNG_IDAT => {
                // Need to copy all the various IDAT chunks into one big one
                // Everything but 3dsmax has one IDAT chunk
                memcpy(work as *mut c_void, data as *const c_void, length as usize);
                work = work.add(length as usize);
                totallength += length;
            }
            PNG_IEND => {
                if !PNG_Unpack(workspace, totallength, image) {
                    Z_Free(workspace as *mut c_void);
                    Z_Free((*image).data as *mut c_void);
                    (*image).data = null_mut();
                    return false;
                }
                moredata = false;
            }
            _ => {}
        }
        data = next;
    }
    Z_Free(workspace as *mut c_void);
    true
}

// Outputs a crc'd chunk of PNG data

pub unsafe extern "C" fn PNG_OutputChunk(
    fp: fileHandle_t,
    type_: ulong,
    data: *mut byte,
    size: ulong,
) -> bool {
    let mut crc: ulong;
    let mut little: ulong;
    let mut outcount: ulong;

    // Output a standard PNG chunk - length, type, data, crc
    little = BigLong(size as c_int) as u32 as ulong;
    outcount = FS_Write(
        &little as *const ulong as *const c_void,
        size_of::<ulong>() as c_int,
        fp,
    ) as ulong;

    little = BigLong(type_ as c_int) as u32 as ulong;
    crc = crc32(0, &little as *const ulong as *const byte, size_of::<ulong>() as ulong);
    outcount += FS_Write(
        &little as *const ulong as *const c_void,
        size_of::<ulong>() as c_int,
        fp,
    ) as ulong;

    if size != 0 {
        crc = crc32(crc, data, size);
        outcount += FS_Write(data as *const c_void, size as c_int, fp) as ulong;
    }

    little = BigLong(crc as c_int) as u32 as ulong;
    outcount += FS_Write(
        &little as *const ulong as *const c_void,
        size_of::<ulong>() as c_int,
        fp,
    ) as ulong;

    if outcount != (size + 12) {
        png_error = PNG_ERROR_WRITE;
        return false;
    }
    true
}

// Saves a PNG format compressed image

pub unsafe extern "C" fn PNG_Save(
    name: *const c_char,
    data: *mut byte,
    width: c_int,
    height: c_int,
    bytedepth: c_int,
) -> bool {
    let mut work: *mut byte;
    let fp: fileHandle_t;
    let maxsize: c_int;
    let mut size: ulong = 0;
    let mut outcount: ulong;
    let mut png_header: png_ihdr_t = MaybeUninit::zeroed().assume_init();

    png_error = PNG_ERROR_OK;

    // Create the file
    fp = FS_FOpenFileWrite(name);
    if fp == 0 {
        png_error = PNG_ERROR_CREATE_FAIL;
        return false;
    }
    // Write out the PNG signature
    outcount = FS_Write(
        png_signature.as_ptr() as *const c_void,
        size_of::<[byte; 8]>() as c_int,
        fp,
    ) as ulong;
    if outcount != size_of::<[byte; 8]>() as ulong {
        FS_FCloseFile(fp);
        png_error = PNG_ERROR_WRITE;
        return false;
    }
    // Create and output a valid header
    PNG_CreateHeader(&mut png_header, width, height, bytedepth);
    if !PNG_OutputChunk(
        fp,
        PNG_IHDR as ulong,
        &mut png_header as *mut png_ihdr_t as *mut byte,
        size_of::<png_ihdr_t>() as ulong,
    ) {
        FS_FCloseFile(fp);
        return false;
    }
    // Create and output the copyright info
    if !PNG_OutputChunk(
        fp,
        PNG_tEXt as ulong,
        png_copyright.as_ptr() as *mut byte,
        size_of::<[c_char; 35]>() as ulong,
    ) {
        FS_FCloseFile(fp);
        return false;
    }
    // Max size of compressed image (source size + 0.1% + 12)
    maxsize = (width * height * bytedepth) + 4096;
    work = Z_Malloc(maxsize, TAG_TEMP_PNG, QTRUE) as *mut byte; // fixme: optimise to qfalse sometime - ok?

    // Pack up the image data
    if !PNG_Pack(
        work,
        &mut size,
        maxsize as ulong,
        data,
        width,
        height,
        bytedepth,
    ) {
        Z_Free(work as *mut c_void);
        FS_FCloseFile(fp);
        return false;
    }
    // Write out the compressed image data
    if !PNG_OutputChunk(fp, PNG_IDAT as ulong, work, size) {
        Z_Free(work as *mut c_void);
        FS_FCloseFile(fp);
        return false;
    }
    Z_Free(work as *mut c_void);
    // Output terminating chunk
    if !PNG_OutputChunk(fp, PNG_IEND as ulong, null_mut(), 0) {
        FS_FCloseFile(fp);
        return false;
    }
    FS_FCloseFile(fp);
    true
}

/*
=============
PNG_ConvertTo32
=============
*/

pub unsafe extern "C" fn PNG_ConvertTo32(image: *mut png_image_t) {
    let mut temp: *mut byte;
    let mut old: *mut byte;
    let old2: *mut byte;
    let mut i: ulong;

    temp = Z_Malloc(((*image).width * (*image).height * 4) as c_int, TAG_TEMP_PNG, QTRUE)
        as *mut byte;
    old = (*image).data;
    old2 = old;
    (*image).data = temp;
    (*image).bytedepth = 4;

    i = 0;
    while i < (*image).width * (*image).height {
        *temp = *old;
        temp = temp.add(1);
        old = old.add(1);
        *temp = *old;
        temp = temp.add(1);
        old = old.add(1);
        *temp = *old;
        temp = temp.add(1);
        old = old.add(1);
        *temp = 0xff;
        temp = temp.add(1);
        i += 1;
    }
    Z_Free(old2 as *mut c_void);
}

/*
=============
LoadPNG32
=============
*/
pub unsafe extern "C" fn LoadPNG32(
    name: *mut c_char,
    pixels: *mut *mut byte,
    width: *mut c_int,
    height: *mut c_int,
    bytedepth: *mut c_int,
) -> bool {
    let mut buffer: *mut byte = null_mut();
    let mut bufferptr: *mut *mut byte = &mut buffer;
    let nLen: c_int;
    let mut png_image = MaybeUninit::<png_image_t>::uninit();
    let png_image_ptr = png_image.as_mut_ptr();

    if pixels.is_null() {
        bufferptr = null_mut();
    }
    nLen = FS_ReadFile(name as *const c_char, bufferptr as *mut *mut c_void);
    if nLen == -1 {
        if !pixels.is_null() {
            *pixels = null_mut();
        }
        return false;
    }
    if pixels.is_null() {
        return true;
    }
    *pixels = null_mut();
    (*png_image_ptr).isimage = true;
    if !PNG_Load(buffer, nLen as ulong, png_image_ptr) {
        Com_Printf(
            c"Error parsing %s: %s\n".as_ptr(),
            name,
            PNG_GetError(),
        );
        return false;
    }
    if (*png_image_ptr).bytedepth != 4 {
        PNG_ConvertTo32(png_image_ptr);
    }
    *pixels = (*png_image_ptr).data;
    if !width.is_null() {
        *width = (*png_image_ptr).width as c_int;
    }
    if !height.is_null() {
        *height = (*png_image_ptr).height as c_int;
    }
    if !bytedepth.is_null() {
        *bytedepth = (*png_image_ptr).bytedepth as c_int;
    }
    FS_FreeFile(buffer as *mut c_void);
    true
}

/*
=============
LoadPNG8
=============
*/
pub unsafe extern "C" fn LoadPNG8(
    name: *mut c_char,
    pixels: *mut *mut byte,
    width: *mut c_int,
    height: *mut c_int,
) -> bool {
    let mut buffer: *mut byte = null_mut();
    let mut bufferptr: *mut *mut byte = &mut buffer;
    let nLen: c_int;
    let mut png_image = MaybeUninit::<png_image_t>::uninit();
    let png_image_ptr = png_image.as_mut_ptr();

    if pixels.is_null() {
        bufferptr = null_mut();
    }
    nLen = FS_ReadFile(name as *const c_char, bufferptr as *mut *mut c_void);
    if nLen == -1 {
        if !pixels.is_null() {
            *pixels = null_mut();
        }
        return false;
    }
    if pixels.is_null() {
        return true;
    }
    *pixels = null_mut();
    (*png_image_ptr).isimage = false;
    if !PNG_Load(buffer, nLen as ulong, png_image_ptr) {
        Com_Printf(
            c"Error parsing %s: %s\n".as_ptr(),
            name,
            PNG_GetError(),
        );
        return false;
    }
    *pixels = (*png_image_ptr).data;
    if !width.is_null() {
        *width = (*png_image_ptr).width as c_int;
    }
    if !height.is_null() {
        *height = (*png_image_ptr).height as c_int;
    }
    FS_FreeFile(buffer as *mut c_void);
    true
}

// end
