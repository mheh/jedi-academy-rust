// Filename:-	tr_jpeg_interace.cpp
//

// leave this as first line for PCH reasons...
//

use core::ffi::{c_char, c_int, c_uint, c_void};

// Include file for users of JPEG library.
// You will need to have included system headers that define at least
// the typedefs FILE and size_t before you can include jpeglib.h.
// (stdio.h is sufficient on ANSI-conforming systems.)
// You may also wish to include "jerror.h".

// JPEG library types and functions
type j_compress_ptr = *mut c_void;
type j_common_ptr = *mut c_void;
type j_decompress_ptr = *mut c_void;
type JSAMPARRAY = *mut *mut u8;
type JSAMPROW = *mut u8;
type JDIMENSION = c_uint;
type boolean = c_uint;

const TRUE: boolean = 1;
const FALSE: boolean = 0;

// Game engine types
type byte = u8;
type qboolean = c_int;
type fileHandle_t = c_int;
type LPCSTR = *const c_char;

const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// JPEG library structures (opaque)
#[repr(C)]
pub struct jpeg_decompress_struct {
    pub err: *mut c_void,
    pub output_width: c_uint,
    pub output_height: c_uint,
    pub output_components: c_int,
    pub output_scanline: c_uint,
    // ... other fields omitted for brevity (opaque)
}

#[repr(C)]
pub struct jpeg_error_mgr {
    pub num_warnings: c_int,
    // ... other fields omitted (opaque)
}

#[repr(C)]
pub struct jpeg_compress_struct {
    pub err: *mut c_void,
    pub dest: *mut c_void,
    pub mem: *mut c_void,
    pub image_width: c_uint,
    pub image_height: c_uint,
    pub input_components: c_int,
    pub in_color_space: c_int,
    pub global_state: c_int,
    pub next_scanline: c_uint,
    pub master: *mut c_void,
    pub main: *mut c_void,
    pub raw_data_in: boolean,
    pub progress: *mut c_void,
    // ... other fields omitted (opaque)
}

#[repr(C)]
pub struct jpeg_destination_mgr {
    pub next_output_byte: *mut u8,
    pub free_in_buffer: usize,
    pub init_destination: Option<unsafe extern "C" fn(j_compress_ptr)>,
    pub empty_output_buffer: Option<unsafe extern "C" fn(j_compress_ptr) -> boolean>,
    pub term_destination: Option<unsafe extern "C" fn(j_compress_ptr)>,
}

// Custom destination manager
#[repr(C)]
struct my_destination_mgr {
    pub pub_: jpeg_destination_mgr,
    pub outfile: *mut byte,
    pub size: c_int,
}

type my_dest_ptr = *mut my_destination_mgr;

// Extern functions from JPEG library
extern "C" {
    fn jpeg_std_error(err: *mut jpeg_error_mgr) -> *mut jpeg_error_mgr;
    fn jpeg_create_decompress(cinfo: j_decompress_ptr);
    fn jpeg_stdio_src(cinfo: j_decompress_ptr, infile: *mut u8);
    fn jpeg_read_header(cinfo: j_decompress_ptr, require_image: boolean) -> c_int;
    fn jpeg_start_decompress(cinfo: j_decompress_ptr) -> boolean;
    fn jpeg_read_scanlines(
        cinfo: j_decompress_ptr,
        scanlines: JSAMPARRAY,
        max_lines: JDIMENSION,
    ) -> JDIMENSION;
    fn jpeg_finish_decompress(cinfo: j_decompress_ptr) -> boolean;
    fn jpeg_destroy_decompress(cinfo: j_decompress_ptr);

    fn jpeg_create_compress(cinfo: j_compress_ptr);
    fn jpeg_set_defaults(cinfo: j_compress_ptr);
    fn jpeg_set_quality(cinfo: j_compress_ptr, quality: c_int, force_baseline: boolean);
    fn jpeg_suppress_tables(cinfo: j_compress_ptr, suppress: boolean);
    fn jpeg_write_scanlines(
        cinfo: j_compress_ptr,
        scanlines: JSAMPARRAY,
        num_lines: JDIMENSION,
    ) -> JDIMENSION;
    fn jpeg_finish_compress(cinfo: j_compress_ptr);
    fn jpeg_destroy_compress(cinfo: j_compress_ptr);
    fn jinit_compress_master(cinfo: j_compress_ptr);
}

// Extern functions from game engine
extern "C" {
    fn Z_Malloc(size: usize, tag: c_int, clear: qboolean) -> *mut c_void;
    fn Z_Free(ptr: *mut c_void);
    fn FS_FOpenFileRead(filename: *const c_char, file: *mut fileHandle_t, unique: qboolean) -> c_int;
    fn FS_Read(buffer: *mut c_void, len: c_int, file: fileHandle_t);
    fn FS_FCloseFile(file: fileHandle_t);
    fn FS_WriteFile(filename: *const c_char, data: *const c_void, size: c_int);
    fn VID_Printf(print_level: c_int, fmt: *const c_char, ...);
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
}

// Extern constants
extern "C" {
    static PRINT_WARNING: c_int;
    static PRINT_ALL: c_int;
    static ERR_FATAL: c_int;
    static TAG_TEMP_JPG: c_int;
    static JCS_RGB: c_int;
    static CSTATE_START: c_int;
    static CSTATE_SCANNING: c_int;
    static CSTATE_RAW_OK: c_int;
    static JERR_BAD_STATE: c_int;
    static JWRN_TOO_MUCH_DATA: c_int;
    static JPOOL_PERMANENT: c_int;
}

// Macros used in original code
macro_rules! ERREXIT1 {
    ($cinfo:expr, $code:expr, $p1:expr) => {
        // This would call error handling, but we're using opaque pointers
        // Preserve the call pattern
        unsafe {
            // Call error handler through function pointer if available
            // For now, this is a stub
        }
    };
}

macro_rules! WARNMS {
    ($cinfo:expr, $code:expr) => {
        // This would call warning handler, but we're using opaque pointers
    };
}

// JPG decompression now subroutinised so I can call it from the savegame stuff...
//
// (note, the param "byte* pJPGData" should be a malloc of 4K more than the JPG data because the decompressor will read
//	up to 4K beyond what's actually presented during decompression).
//
// This will Z_Malloc the output data buffer that gets fed back into "pic", so Z_Free it yourself later.
//
unsafe fn Decompress_JPG(
    filename: *const c_char,
    pJPGData: *mut byte,
    pic: *mut *mut byte,
    width: *mut c_int,
    height: *mut c_int,
) {
    // This struct contains the JPEG decompression parameters and pointers to
    // working space (which is allocated as needed by the JPEG library).
    let mut cinfo: jpeg_decompress_struct = core::mem::zeroed();

    // We use our private extension JPEG error handler.
    // Note that this struct must live as long as the main JPEG parameter
    // struct, to avoid dangling-pointer problems.

    // This struct represents a JPEG error handler.  It is declared separately
    // because applications often want to supply a specialized error handler
    // (see the second half of this file for an example).  But here we just
    // take the easy way out and use the standard error handler, which will
    // print a message on stderr and call exit() if compression fails.
    // Note that this struct must live as long as the main JPEG parameter
    // struct, to avoid dangling-pointer problems.
    let mut jerr: jpeg_error_mgr = core::mem::zeroed();

    // More stuff
    let buffer: JSAMPARRAY;
    let row_stride: c_int;
    let out: *mut byte;
    let bbuf: *mut byte;

    // Step 1: allocate and initialize JPEG decompression object

    // We have to set up the error handler first, in case the initialization
    // step fails.  (Unlikely, but it could happen if you are out of memory.)
    // This routine fills in the contents of struct jerr, and returns jerr's
    // address which we place into the link field in cinfo.
    cinfo.err = jpeg_std_error(&mut jerr) as *mut c_void;

    // Now we can initialize the JPEG decompression object.
    jpeg_create_decompress(&mut cinfo as *mut _ as j_decompress_ptr);

    // Step 2: specify data source (eg, a file)

    jpeg_stdio_src(&mut cinfo as *mut _ as j_decompress_ptr, pJPGData);

    // Step 3: read file parameters with jpeg_read_header()

    let _ = jpeg_read_header(&mut cinfo as *mut _ as j_decompress_ptr, TRUE);
    // We can ignore the return value from jpeg_read_header since
    //   (a) suspension is not possible with the stdio data source, and
    //   (b) we passed TRUE to reject a tables-only JPEG file as an error.
    // See libjpeg.doc for more info.

    // Step 4: set parameters for decompression

    // In this example, we don't need to change any of the defaults set by
    // jpeg_read_header(), so we do nothing here.

    // Step 5: Start decompressor

    let _ = jpeg_start_decompress(&mut cinfo as *mut _ as j_decompress_ptr);
    // We can ignore the return value since suspension is not possible
    // with the stdio data source.

    // We may need to do some setup of our own at this point before reading
    // the data.  After jpeg_start_decompress() we have the correct scaled
    // output image dimensions available, as well as the output colormap
    // if we asked for color quantization.
    // In this example, we need to make an output work buffer of the right size.

    // JSAMPLEs per row in output buffer
    let row_stride_val = (cinfo.output_width as c_int) * (cinfo.output_components as c_int);
    let row_stride = row_stride_val;

    if cinfo.output_components != 4 && cinfo.output_components != 1 {
        VID_Printf(
            PRINT_WARNING,
            "JPG %s is unsupported color depth (%d)\n\0".as_ptr() as *const c_char,
            filename,
            cinfo.output_components,
        );
    }
    let out_val = Z_Malloc(
        (cinfo.output_width as usize) * (cinfo.output_height as usize) * 4,
        TAG_TEMP_JPG,
        qfalse,
    ) as *mut byte;
    let out = out_val;

    *pic = out;
    *width = cinfo.output_width as c_int;
    *height = cinfo.output_height as c_int;

    // Step 6: while (scan lines remain to be read)
    //           jpeg_read_scanlines(...);

    // Here we use the library's state variable cinfo.output_scanline as the
    // loop counter, so that we don't have to keep track ourselves.
    while cinfo.output_scanline < cinfo.output_height {
        // jpeg_read_scanlines expects an array of pointers to scanlines.
        // Here the array is only one element long, but you could ask for
        // more than one scanline at a time if that's more convenient.
        let bbuf_val = out.add((row_stride as usize) * (cinfo.output_scanline as usize));
        let bbuf = bbuf_val;
        let buffer_val = &mut (bbuf as *mut u8) as *mut *mut u8 as JSAMPARRAY;
        let buffer = buffer_val;
        let _ = jpeg_read_scanlines(&mut cinfo as *mut _ as j_decompress_ptr, buffer, 1);
    }

    // if we've just loaded a greyscale, then adjust it from 8-bit to 32bit by stretch-copying it over itself...
    //  (this also does the alpha stuff as well)
    //
    if cinfo.output_components == 1 {
        let pbDest = (*pic).add((cinfo.output_width as usize) * (cinfo.output_height as usize) * 4)
            .offset(-1);
        let pbSrc = (*pic).add((cinfo.output_width as usize) * (cinfo.output_height as usize))
            .offset(-1);
        let iPixels = (cinfo.output_width as c_int) * (cinfo.output_height as c_int);

        for i in 0..iPixels {
            let b = *pbSrc.offset(-(i as isize));
            *pbDest.offset(-(i as isize)) = 255;
            *pbDest.offset(-((i + 1) as isize)) = b;
            *pbDest.offset(-((i + 2) as isize)) = b;
            *pbDest.offset(-((i + 3) as isize)) = b;
        }
    } else {
        // clear all the alphas to 255
        let buf = *pic;

        let j = (cinfo.output_width as c_int) * (cinfo.output_height as c_int) * 4;
        let mut i = 3;
        while i < j {
            *buf.offset(i as isize) = 255;
            i += 4;
        }
    }

    // Step 7: Finish decompression

    let _ = jpeg_finish_decompress(&mut cinfo as *mut _ as j_decompress_ptr);
    // We can ignore the return value since suspension is not possible
    // with the stdio data source.

    // Step 8: Release JPEG decompression object

    // This is an important step since it will release a good deal of memory.
    jpeg_destroy_decompress(&mut cinfo as *mut _ as j_decompress_ptr);

    // After finish_decompress, we can close the input file.
    // Here we postpone it until after no more JPEG errors are possible,
    // so as to simplify the setjmp error logic above.  (Actually, I don't
    // think that jpeg_destroy can do an error exit, but why assume anything...)

    // At this point you may want to check to see whether any corrupt-data
    // warnings occurred (test whether jerr.pub.num_warnings is nonzero).

    // And we're done!
}

unsafe fn LoadJPG(
    filename: *const c_char,
    pic: *mut *mut byte,
    width: *mut c_int,
    height: *mut c_int,
) -> c_int {
    *pic = core::ptr::null_mut();

    let mut h: fileHandle_t = 0;
    let len = FS_FOpenFileRead(filename, &mut h, qfalse);
    if h == 0 {
        return 0;
    }
    // JPEG system reads 4K past input buffer so we tack on an additional 4k.
    let pJPGData = Z_Malloc((len + 4096) as usize, TAG_TEMP_JPG, qfalse) as *mut byte;
    FS_Read(pJPGData as *mut c_void, len, h);
    FS_FCloseFile(h);

    Decompress_JPG(filename, pJPGData, pic, width, height);

    Z_Free(pJPGData as *mut c_void);
    len
}

// Expanded data destination object for stdio output

#[repr(C)]
struct my_destination_mgr_def {
    pub pub_: jpeg_destination_mgr, /* public fields */
    pub outfile: *mut byte,          /* target stream */
    pub size: c_int,
}

// Initialized destination --- called by jpeg_start_compress
// before any data is actually written.

unsafe extern "C" fn init_destination(cinfo: j_compress_ptr) {
    let dest = cinfo as *mut jpeg_compress_struct;
    let dest_mgr = (*dest).dest as *mut my_destination_mgr_def;

    (*dest_mgr).pub_.next_output_byte = (*dest_mgr).outfile;
    (*dest_mgr).pub_.free_in_buffer = (*dest_mgr).size as usize;
}

// Empty the output buffer --- called whenever buffer fills up.
//
// In typical applications, this should write the entire output buffer
// (ignoring the current state of next_output_byte & free_in_buffer),
// reset the pointer & count to the start of the buffer, and return TRUE
// indicating that the buffer has been dumped.
//
// In applications that need to be able to suspend compression due to output
// overrun, a FALSE return indicates that the buffer cannot be emptied now.
// In this situation, the compressor will return to its caller (possibly with
// an indication that it has not accepted all the supplied scanlines).  The
// application should resume compression after it has made more room in the
// output buffer.  Note that there are substantial restrictions on the use of
// suspension --- see the documentation.
//
// When suspending, the compressor will back up to a convenient restart point
// (typically the start of the current MCU). next_output_byte & free_in_buffer
// indicate where the restart point will be if the current call returns FALSE.
// Data beyond this point will be regenerated after resumption, so do not
// write it out when emptying the buffer externally.

unsafe extern "C" fn empty_output_buffer(_cinfo: j_compress_ptr) -> boolean {
    TRUE
}

// Compression initialization.
// Before calling this, all parameters and a data destination must be set up.
//
// We require a write_all_tables parameter as a failsafe check when writing
// multiple datastreams from the same compression object.  Since prior runs
// will have left all the tables marked sent_table=TRUE, a subsequent run
// would emit an abbreviated stream (no tables) by default.  This may be what
// is wanted, but for safety's sake it should not be the default behavior:
// programmers should have to make a deliberate choice to emit abbreviated
// images.  Therefore the documentation and examples should encourage people
// to pass write_all_tables=TRUE; then it will take active thought to do the
// wrong thing.

unsafe fn jpeg_start_compress(cinfo: j_compress_ptr, write_all_tables: boolean) {
    let cinfo_mut = cinfo as *mut jpeg_compress_struct;

    if (*cinfo_mut).global_state != CSTATE_START {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo_mut).global_state);
    }

    if write_all_tables != FALSE {
        jpeg_suppress_tables(cinfo, FALSE); /* mark all tables to be written */
    }

    // (Re)initialize error mgr and destination modules
    (*((*cinfo_mut).err as *mut jpeg_error_mgr)
        as *const _ as *const *const ()) // stub for reset_error_mgr call

    let reset_error_mgr: Option<unsafe extern "C" fn(j_common_ptr)> = None;
    if let Some(f) = reset_error_mgr {
        f(cinfo_mut as j_common_ptr);
    }

    if let Some(dest) = (*cinfo_mut).dest as *const jpeg_destination_mgr {
        if let Some(f) = (*dest).init_destination {
            f(cinfo);
        }
    }

    // Perform master selection of active modules
    jinit_compress_master(cinfo);

    // Set up for the first pass
    if let Some(master) = (*cinfo_mut).master as *const _ {
        // Call prepare_for_pass through function pointer
        // This is a stub for now
    }

    // Ready for application to drive first pass through jpeg_write_scanlines
    // or jpeg_write_raw_data.
    (*cinfo_mut).next_scanline = 0;
    (*cinfo_mut).global_state = if (*cinfo_mut).raw_data_in != FALSE {
        CSTATE_RAW_OK
    } else {
        CSTATE_SCANNING
    };
}

// Write some scanlines of data to the JPEG compressor.
//
// The return value will be the number of lines actually written.
// This should be less than the supplied num_lines only in case that
// the data destination module has requested suspension of the compressor,
// or if more than image_height scanlines are passed in.
//
// Note: we warn about excess calls to jpeg_write_scanlines() since
// this likely signals an application programmer error.  However,
// excess scanlines passed in the last valid call are *silently* ignored,
// so that the application need not adjust num_lines for end-of-image
// when using a multiple-scanline buffer.

unsafe fn jpeg_write_scanlines(
    cinfo: j_compress_ptr,
    scanlines: JSAMPARRAY,
    num_lines: JDIMENSION,
) -> JDIMENSION {
    let cinfo_mut = cinfo as *mut jpeg_compress_struct;
    let mut row_ctr: JDIMENSION = 0;
    let mut rows_left: JDIMENSION;
    let mut num_lines_mut = num_lines;

    if (*cinfo_mut).global_state != CSTATE_SCANNING {
        ERREXIT1!(cinfo, JERR_BAD_STATE, (*cinfo_mut).global_state);
    }
    if (*cinfo_mut).next_scanline >= (*cinfo_mut).image_height {
        WARNMS!(cinfo, JWRN_TOO_MUCH_DATA);
    }

    // Call progress monitor hook if present
    if !(*cinfo_mut).progress.is_null() {
        let progress = (*cinfo_mut).progress as *mut c_void;
        // progress->pass_counter = (long) cinfo->next_scanline;
        // progress->pass_limit = (long) cinfo->image_height;
        // (*progress->progress_monitor)((j_common_ptr) cinfo);
    }

    // Give master control module another chance if this is first call to
    // jpeg_write_scanlines.  This lets output of the frame/scan headers be
    // delayed so that application can write COM, etc, markers between
    // jpeg_start_compress and jpeg_write_scanlines.
    if let Some(master) = (*cinfo_mut).master as *const _ {
        // if ((*master).call_pass_startup) {
        //   (*(*master).pass_startup)(cinfo);
        // }
    }

    // Ignore any extra scanlines at bottom of image.
    rows_left = (*cinfo_mut).image_height - (*cinfo_mut).next_scanline;
    if num_lines_mut > rows_left {
        num_lines_mut = rows_left;
    }

    row_ctr = 0;
    if let Some(main) = (*cinfo_mut).main as *const _ {
        // (*main->process_data)(cinfo, scanlines, &row_ctr, num_lines_mut);
    }
    (*cinfo_mut).next_scanline += row_ctr;
    row_ctr
}

// Terminate destination --- called by jpeg_finish_compress
// after all data has been written.  Usually needs to flush buffer.
//
// NB: *not* called by jpeg_abort or jpeg_destroy; surrounding
// application must deal with any cleanup that should happen even
// for error exit.

static mut hackSize: c_int = 0;

unsafe extern "C" fn term_destination(cinfo: j_compress_ptr) {
    let dest = cinfo as *mut jpeg_compress_struct;
    let dest_mgr = (*dest).dest as *mut my_destination_mgr_def;
    let datacount = (*dest_mgr).size as usize - (*dest_mgr).pub_.free_in_buffer;
    hackSize = datacount as c_int;
}

// Prepare for output to a stdio stream.
// The caller must have already opened the stream, and is responsible
// for closing it after finishing compression.

unsafe fn jpegDest(cinfo: j_compress_ptr, outfile: *mut byte, size: c_int) {
    let cinfo_mut = cinfo as *mut jpeg_compress_struct;
    let dest: my_dest_ptr;

    // The destination object is made permanent so that multiple JPEG images
    // can be written to the same file without re-executing jpeg_stdio_dest.
    // This makes it dangerous to use this manager and a different destination
    // manager serially with the same JPEG object, because their private object
    // sizes may be different.  Caveat programmer.
    if (*cinfo_mut).dest.is_null() {
        // first time for this JPEG object?
        // cinfo->dest = (struct jpeg_destination_mgr *)
        //   (*cinfo->mem->alloc_small) ((j_common_ptr) cinfo, JPOOL_PERMANENT,
        //                                 sizeof(my_destination_mgr));
        // Stub: allocation would happen through JPEG's memory manager
    }

    dest = (*cinfo_mut).dest as my_dest_ptr;
    (*dest).pub_.init_destination = Some(init_destination);
    (*dest).pub_.empty_output_buffer = Some(empty_output_buffer);
    (*dest).pub_.term_destination = Some(term_destination);
    (*dest).outfile = outfile;
    (*dest).size = size;
}

// returns a Z_Malloc'd piece of mem that you should free up yourself
//
unsafe fn Compress_JPG(
    pOutputSize: *mut c_int,
    quality: c_int,
    image_width: c_int,
    image_height: c_int,
    image_buffer: *mut byte,
    bInvertDuringCompression: qboolean,
) -> *mut byte {
    // This struct contains the JPEG compression parameters and pointers to
    // working space (which is allocated as needed by the JPEG library).
    // It is possible to have several such structures, representing multiple
    // compression/decompression processes, in existence at once.  We refer
    // to any one struct (and its associated working data) as a "JPEG object".
    let mut cinfo: jpeg_compress_struct = core::mem::zeroed();

    // This struct represents a JPEG error handler.  It is declared separately
    // because applications often want to supply a specialized error handler
    // (see the second half of this file for an example).  But here we just
    // take the easy way out and use the standard error handler, which will
    // print a message on stderr and call exit() if compression fails.
    // Note that this struct must live as long as the main JPEG parameter
    // struct, to avoid dangling-pointer problems.
    let mut jerr: jpeg_error_mgr = core::mem::zeroed();

    // More stuff
    let row_pointer: [JSAMPROW; 1];
    let row_stride: c_int;

    // Step 1: allocate and initialize JPEG compression object

    // We have to set up the error handler first, in case the initialization
    // step fails.  (Unlikely, but it could happen if you are out of memory.)
    // This routine fills in the contents of struct jerr, and returns jerr's
    // address which we place into the link field in cinfo.
    cinfo.err = jpeg_std_error(&mut jerr) as *mut c_void;

    // Now we can initialize the JPEG compression object.
    jpeg_create_compress(&mut cinfo as *mut _ as j_compress_ptr);

    // Step 2: specify data destination (eg, a file)
    // Note: steps 2 and 3 can be done in either order.

    // Here we use the library-supplied code to send compressed data to a
    // stdio stream.  You can also write your own code to do something else.
    // VERY IMPORTANT: use "b" option to fopen() if you are on a machine that
    // requires it in order to write binary files.
    let out = Z_Malloc(
        (image_width as usize) * (image_height as usize) * 4,
        TAG_TEMP_JPG,
        qfalse,
    ) as *mut byte;

    jpegDest(
        &mut cinfo as *mut _ as j_compress_ptr,
        out,
        (image_width * image_height * 4) as c_int,
    );

    // Step 3: set parameters for compression

    // First we supply a description of the input image.
    // Four fields of the cinfo struct must be filled in:
    cinfo.image_width = image_width as c_uint; /* image width and height, in pixels */
    cinfo.image_height = image_height as c_uint;
    cinfo.input_components = 4; /* # of color components per pixel */
    cinfo.in_color_space = JCS_RGB; /* colorspace of input image */

    // Now use the library's routine to set default compression parameters.
    // (You must set at least cinfo.in_color_space before calling this,
    // since the defaults depend on the source color space.)
    jpeg_set_defaults(&mut cinfo as *mut _ as j_compress_ptr);

    // Now you can set any non-default parameters you wish to.
    // Here we just illustrate the use of quality (quantization table) scaling:
    jpeg_set_quality(
        &mut cinfo as *mut _ as j_compress_ptr,
        quality,
        TRUE, /* limit to baseline-JPEG values */
    );

    // Step 4: Start compressor

    // TRUE ensures that we will write a complete interchange-JPEG file.
    // Pass TRUE unless you are very sure of what you're doing.
    jpeg_start_compress(&mut cinfo as *mut _ as j_compress_ptr, TRUE);

    // Step 5: while (scan lines remain to be written)
    //           jpeg_write_scanlines(...);

    // Here we use the library's state variable cinfo.next_scanline as the
    // loop counter, so that we don't have to keep track ourselves.
    // To keep things simple, we pass one scanline per call; you can pass
    // more if you wish, though.
    row_stride = image_width * 4; /* JSAMPLEs per row in image_buffer */

    while cinfo.next_scanline < cinfo.image_height {
        // jpeg_write_scanlines expects an array of pointers to scanlines.
        // Here the array is only one element long, but you could pass
        // more than one scanline at a time if that's more convenient.
        let row_ptr = if bInvertDuringCompression != qfalse {
            image_buffer.add(
                (((cinfo.image_height as c_int - 1) * row_stride)
                    - (cinfo.next_scanline as c_int) * row_stride) as usize,
            )
        } else {
            image_buffer.add(((cinfo.next_scanline as c_int) * row_stride) as usize)
        };

        let row_pointer_local = [row_ptr; 1];
        let _ = jpeg_write_scanlines(
            &mut cinfo as *mut _ as j_compress_ptr,
            row_pointer_local.as_ptr() as JSAMPARRAY,
            1,
        );
    }

    // Step 6: Finish compression

    jpeg_finish_compress(&mut cinfo as *mut _ as j_compress_ptr);

    // Step 7: release JPEG compression object

    // This is an important step since it will release a good deal of memory.
    jpeg_destroy_compress(&mut cinfo as *mut _ as j_compress_ptr);

    // And we're done!

    *pOutputSize = hackSize;
    out
}

unsafe fn SaveJPG(
    filename: *const c_char,
    quality: c_int,
    image_width: c_int,
    image_height: c_int,
    image_buffer: *mut byte,
) {
    let mut iOutputSize: c_int = 0;

    let pbOut = Compress_JPG(
        &mut iOutputSize,
        quality,
        image_width,
        image_height,
        image_buffer,
        qtrue,
    );

    FS_WriteFile(filename, pbOut as *const c_void, iOutputSize);

    Z_Free(pbOut as *mut c_void);
}

unsafe fn JPG_ErrorThrow(message: LPCSTR) {
    Com_Error(ERR_FATAL, "JPG: %s\n\0".as_ptr() as *const c_char, message);
}

unsafe fn JPG_MessageOut(message: LPCSTR) {
    VID_Printf(PRINT_ALL, "%s\n\0".as_ptr() as *const c_char, message);
}

//////////////// eof ////////////
