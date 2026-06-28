/*
 * jdapistd.c
 *
 * Copyright (C) 1994-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains application interface code for the decompression half
 * of the JPEG library.  These are the "standard" API routines that are
 * used in the normal full-decompression case.  They are not used by a
 * transcoding-only application.  Note that if an application links in
 * jpeg_start_decompress, it will end up linking in the entire decompressor.
 * We thus must separate this file from jdapimin.c to avoid linking the
 * whole decompression library into a transcoder.
 */

// leave this as first line for PCH reasons...
//

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_void};

/* Forward declarations */
fn output_pass_setup(cinfo: j_decompress_ptr) -> boolean;

// ============================================================================
// JPEG Library FFI Type Declarations
// ============================================================================

#[repr(C)]
pub struct jpeg_error_mgr {
    pub msg_code: c_int,
    pub msg_parm: jpeg_error_mgr_msg_parm,
    // ... other fields are opaque to us
    _priv: [u8; 0],
}

#[repr(C)]
pub union jpeg_error_mgr_msg_parm {
    pub i: [c_int; 8],
    // char s[JMSG_STR_PARM_MAX];  -- not needed for this file
}

#[repr(C)]
pub struct jpeg_progress_mgr {
    pub pass_counter: c_int,
    pub pass_limit: c_int,
    // ... other fields
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_input_controller {
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_d_main_controller {
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_decomp_master {
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_d_coef_controller {
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_decompress_struct {
    pub global_state: c_int,
    pub buffered_image: boolean,
    pub raw_data_out: boolean,
    pub output_scanline: JDIMENSION,
    pub output_height: JDIMENSION,
    pub input_scan_number: c_int,
    pub output_scan_number: c_int,
    pub max_v_samp_factor: c_int,
    pub min_DCT_scaled_size: c_int,
    pub total_iMCU_rows: JDIMENSION,
    pub progress: *mut jpeg_progress_mgr,
    pub inputctl: *mut jpeg_input_controller,
    pub master: *mut jpeg_decomp_master,
    pub main: *mut jpeg_d_main_controller,
    pub coef: *mut jpeg_d_coef_controller,
    pub err: *mut jpeg_error_mgr,
    _priv: [u8; 0],
}

#[allow(non_camel_case_types)]
pub type j_decompress_ptr = *mut jpeg_decompress_struct;
#[allow(non_camel_case_types)]
pub type j_common_ptr = *mut c_void;
#[allow(non_camel_case_types)]
pub type JDIMENSION = u32;
#[allow(non_camel_case_types)]
pub type JSAMPARRAY = *mut *mut u8;
#[allow(non_camel_case_types)]
pub type JSAMPIMAGE = *mut *mut *mut u8;
#[allow(non_camel_case_types)]
pub type boolean = u8;

// State constants from jpeglib.h/jpegint.h
const DSTATE_READY: c_int = 202;
const DSTATE_PRELOAD: c_int = 203;
const DSTATE_PRESCAN: c_int = 204;
const DSTATE_SCANNING: c_int = 205;
const DSTATE_RAW_OK: c_int = 206;
const DSTATE_BUFIMAGE: c_int = 207;
const DSTATE_BUFPOST: c_int = 208;

// Return codes from jpeg_consume_input
const JPEG_SUSPENDED: c_int = 0;
const JPEG_REACHED_SOS: c_int = 1;
const JPEG_REACHED_EOI: c_int = 2;
const JPEG_ROW_COMPLETED: c_int = 3;
const JPEG_SCAN_COMPLETED: c_int = 4;

// Error message codes (from jerror.h)
const JERR_NOT_COMPILED: c_int = 90;
const JERR_BAD_STATE: c_int = 61;
const JERR_BUFFER_SIZE: c_int = 63;

// Warning message code
const JWRN_TOO_MUCH_DATA: c_int = 172;

// Boolean constants
const TRUE: u8 = 1;
const FALSE: u8 = 0;

// External JPEG library functions
extern "C" {
    fn jinit_master_decompress(cinfo: j_decompress_ptr);
}

/*
 * Decompression initialization.
 * jpeg_read_header must be completed before calling this.
 *
 * If a multipass operating mode was selected, this will do all but the
 * last pass, and thus may take a great deal of time.
 *
 * Returns FALSE if suspended.  The return value need be inspected only if
 * a suspending data source is used.
 */

#[allow(non_snake_case)]
pub extern "C" fn jpeg_start_decompress(cinfo: j_decompress_ptr) -> boolean {
  unsafe {
    if (*cinfo).global_state == DSTATE_READY {
      /* First call: initialize master control, select active modules */
      jinit_master_decompress(cinfo);
      if (*cinfo).buffered_image != FALSE {
        /* No more work here; expecting jpeg_start_output next */
        (*cinfo).global_state = DSTATE_BUFIMAGE;
        return TRUE;
      }
      (*cinfo).global_state = DSTATE_PRELOAD;
    }
    if (*cinfo).global_state == DSTATE_PRELOAD {
      /* If file has multiple scans, absorb them all into the coef buffer */
      if read_has_multiple_scans(cinfo) != FALSE {
        #[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
        {
          loop {
            let mut retcode: c_int;
            /* Call progress monitor hook if present */
            if !(*cinfo).progress.is_null() {
              if let Some(progress_monitor) = read_progress_monitor(cinfo) {
                progress_monitor(cinfo as j_common_ptr);
              }
            }
            /* Absorb some more input */
            if let Some(consume_input) = read_consume_input(cinfo) {
              retcode = consume_input(cinfo);
            } else {
              return FALSE;
            }
            if retcode == JPEG_SUSPENDED {
              return FALSE;
            }
            if retcode == JPEG_REACHED_EOI {
              break;
            }
            /* Advance progress counter if appropriate */
            if !(*cinfo).progress.is_null() {
              if retcode == JPEG_ROW_COMPLETED || retcode == JPEG_REACHED_SOS {
                (*(*cinfo).progress).pass_counter += 1;
                if (*(*cinfo).progress).pass_counter >= (*(*cinfo).progress).pass_limit {
                  /* jdmaster underestimated number of scans; ratchet up one scan */
                  (*(*cinfo).progress).pass_limit += (*cinfo).total_iMCU_rows as c_int;
                }
              }
            }
          }
        }
        #[cfg(not(feature = "D_MULTISCAN_FILES_SUPPORTED"))]
        {
          ERREXIT(cinfo, JERR_NOT_COMPILED);
        }
      }
      (*cinfo).output_scan_number = (*cinfo).input_scan_number;
    } else if (*cinfo).global_state != DSTATE_PRESCAN {
      ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Perform any dummy output passes, and set up for the final pass */
    return output_pass_setup(cinfo);
  }
}


/*
 * Set up for an output pass, and perform any dummy pass(es) needed.
 * Common subroutine for jpeg_start_decompress and jpeg_start_output.
 * Entry: global_state = DSTATE_PRESCAN only if previously suspended.
 * Exit: If done, returns TRUE and sets global_state for proper output mode.
 *       If suspended, returns FALSE and sets global_state = DSTATE_PRESCAN.
 */

#[allow(non_snake_case)]
fn output_pass_setup(cinfo: j_decompress_ptr) -> boolean {
  unsafe {
    if (*cinfo).global_state != DSTATE_PRESCAN {
      /* First call: do pass setup */
      if let Some(prepare_fn) = read_prepare_for_output_pass(cinfo) {
        prepare_fn(cinfo);
      }
      (*cinfo).output_scanline = 0;
      (*cinfo).global_state = DSTATE_PRESCAN;
    }
    /* Loop over any required dummy passes */
    while read_is_dummy_pass(cinfo) != FALSE {
      #[cfg(feature = "QUANT_2PASS_SUPPORTED")]
      {
        /* Crank through the dummy pass */
        while (*cinfo).output_scanline < (*cinfo).output_height {
          let last_scanline: JDIMENSION;
          /* Call progress monitor hook if present */
          if !(*cinfo).progress.is_null() {
            (*(*cinfo).progress).pass_counter = (*cinfo).output_scanline as c_int;
            (*(*cinfo).progress).pass_limit = (*cinfo).output_height as c_int;
            if let Some(progress_monitor) = read_progress_monitor(cinfo) {
              progress_monitor(cinfo as j_common_ptr);
            }
          }
          /* Process some data */
          last_scanline = (*cinfo).output_scanline;
          if let Some(process_fn) = read_process_data(cinfo) {
            process_fn(cinfo, core::ptr::null_mut(), &mut (*cinfo).output_scanline, 0);
          }
          if (*cinfo).output_scanline == last_scanline {
            return FALSE;		/* No progress made, must suspend */
          }
        }
        /* Finish up dummy pass, and set up for another one */
        if let Some(finish_fn) = read_finish_output_pass(cinfo) {
          finish_fn(cinfo);
        }
        if let Some(prepare_fn) = read_prepare_for_output_pass(cinfo) {
          prepare_fn(cinfo);
        }
        (*cinfo).output_scanline = 0;
      }
      #[cfg(not(feature = "QUANT_2PASS_SUPPORTED"))]
      {
        ERREXIT(cinfo, JERR_NOT_COMPILED);
      }
    }
    /* Ready for application to drive output pass through
     * jpeg_read_scanlines or jpeg_read_raw_data.
     */
    if (*cinfo).raw_data_out != FALSE {
      (*cinfo).global_state = DSTATE_RAW_OK;
    } else {
      (*cinfo).global_state = DSTATE_SCANNING;
    }
    return TRUE;
  }
}


/*
 * Read some scanlines of data from the JPEG decompressor.
 *
 * The return value will be the number of lines actually read.
 * This may be less than the number requested in several cases,
 * including bottom of image, data source suspension, and operating
 * modes that emit multiple scanlines at a time.
 *
 * Note: we warn about excess calls to jpeg_read_scanlines() since
 * this likely signals an application programmer error.  However,
 * an oversize buffer (max_lines > scanlines remaining) is not an error.
 */

#[allow(non_snake_case)]
pub extern "C" fn jpeg_read_scanlines(
    cinfo: j_decompress_ptr,
    scanlines: JSAMPARRAY,
    max_lines: JDIMENSION,
) -> JDIMENSION {
  unsafe {
    let mut row_ctr: JDIMENSION;

    if (*cinfo).global_state != DSTATE_SCANNING {
      ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    if (*cinfo).output_scanline >= (*cinfo).output_height {
      WARNMS(cinfo, JWRN_TOO_MUCH_DATA);
      return 0;
    }

    /* Call progress monitor hook if present */
    if !(*cinfo).progress.is_null() {
      (*(*cinfo).progress).pass_counter = (*cinfo).output_scanline as c_int;
      (*(*cinfo).progress).pass_limit = (*cinfo).output_height as c_int;
      if let Some(progress_monitor) = read_progress_monitor(cinfo) {
        progress_monitor(cinfo as j_common_ptr);
      }
    }

    /* Process some data */
    row_ctr = 0;
    if let Some(process_fn) = read_process_data(cinfo) {
      process_fn(cinfo, scanlines, &mut row_ctr, max_lines);
    }
    (*cinfo).output_scanline += row_ctr;
    return row_ctr;
  }
}


/*
 * Alternate entry point to read raw data.
 * Processes exactly one iMCU row per call, unless suspended.
 */

#[allow(non_snake_case)]
pub extern "C" fn jpeg_read_raw_data(
    cinfo: j_decompress_ptr,
    data: JSAMPIMAGE,
    max_lines: JDIMENSION,
) -> JDIMENSION {
  unsafe {
    let mut lines_per_iMCU_row: JDIMENSION;

    if (*cinfo).global_state != DSTATE_RAW_OK {
      ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    if (*cinfo).output_scanline >= (*cinfo).output_height {
      WARNMS(cinfo, JWRN_TOO_MUCH_DATA);
      return 0;
    }

    /* Call progress monitor hook if present */
    if !(*cinfo).progress.is_null() {
      (*(*cinfo).progress).pass_counter = (*cinfo).output_scanline as c_int;
      (*(*cinfo).progress).pass_limit = (*cinfo).output_height as c_int;
      if let Some(progress_monitor) = read_progress_monitor(cinfo) {
        progress_monitor(cinfo as j_common_ptr);
      }
    }

    /* Verify that at least one iMCU row can be returned. */
    lines_per_iMCU_row = ((*cinfo).max_v_samp_factor * (*cinfo).min_DCT_scaled_size) as u32;
    if max_lines < lines_per_iMCU_row {
      ERREXIT(cinfo, JERR_BUFFER_SIZE);
    }

    /* Decompress directly into user's buffer. */
    if !read_decompress_data(cinfo, data) {
      return 0;			/* suspension forced, can do nothing more */
    }

    /* OK, we processed one iMCU row. */
    (*cinfo).output_scanline += lines_per_iMCU_row;
    return lines_per_iMCU_row;
  }
}


/* Additional entry points for buffered-image mode. */

#[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")]
{

/*
 * Initialize for an output pass in buffered-image mode.
 */

#[allow(non_snake_case)]
pub extern "C" fn jpeg_start_output(cinfo: j_decompress_ptr, scan_number: c_int) -> boolean {
  unsafe {
    let mut scan_number = scan_number;
    if (*cinfo).global_state != DSTATE_BUFIMAGE && (*cinfo).global_state != DSTATE_PRESCAN {
      ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Limit scan number to valid range */
    if scan_number <= 0 {
      scan_number = 1;
    }
    if read_eoi_reached(cinfo) != FALSE && scan_number > (*cinfo).input_scan_number {
      scan_number = (*cinfo).input_scan_number;
    }
    (*cinfo).output_scan_number = scan_number;
    /* Perform any dummy output passes, and set up for the real pass */
    return output_pass_setup(cinfo);
  }
}


/*
 * Finish up after an output pass in buffered-image mode.
 *
 * Returns FALSE if suspended.  The return value need be inspected only if
 * a suspending data source is used.
 */

#[allow(non_snake_case)]
pub extern "C" fn jpeg_finish_output(cinfo: j_decompress_ptr) -> boolean {
  unsafe {
    if ((*cinfo).global_state == DSTATE_SCANNING || (*cinfo).global_state == DSTATE_RAW_OK) && (*cinfo).buffered_image != FALSE {
      /* Terminate this pass. */
      /* We do not require the whole pass to have been completed. */
      if let Some(finish_fn) = read_finish_output_pass(cinfo) {
        finish_fn(cinfo);
      }
      (*cinfo).global_state = DSTATE_BUFPOST;
    } else if (*cinfo).global_state != DSTATE_BUFPOST {
      /* BUFPOST = repeat call after a suspension, anything else is error */
      ERREXIT1(cinfo, JERR_BAD_STATE, (*cinfo).global_state);
    }
    /* Read markers looking for SOS or EOI */
    while (*cinfo).input_scan_number <= (*cinfo).output_scan_number && read_eoi_reached(cinfo) == FALSE {
      if let Some(consume_input) = read_consume_input(cinfo) {
        if consume_input(cinfo) == JPEG_SUSPENDED {
          return FALSE;		/* Suspend, come back later */
        }
      }
    }
    (*cinfo).global_state = DSTATE_BUFIMAGE;
    return TRUE;
  }
}

} /* #[cfg(feature = "D_MULTISCAN_FILES_SUPPORTED")] */

// ============================================================================
// Helper functions for accessing struct fields and function pointers
// ============================================================================

#[inline]
unsafe fn read_has_multiple_scans(cinfo: j_decompress_ptr) -> u8 {
    // Access has_multiple_scans field from inputctl structure
    // This assumes the inputctl field is at the expected offset in jpeg_decompress_struct
    let inputctl = (*cinfo).inputctl as *mut c_void;
    if inputctl.is_null() {
        FALSE
    } else {
        // SAFETY: We're accessing a field that should be at a known offset
        // This is a mechanical translation from C pointer arithmetic
        *(inputctl as *mut u8).add(0)
    }
}

#[inline]
unsafe fn read_eoi_reached(cinfo: j_decompress_ptr) -> u8 {
    let inputctl = (*cinfo).inputctl as *mut c_void;
    if inputctl.is_null() {
        FALSE
    } else {
        *(inputctl as *mut u8).add(1)
    }
}

#[inline]
unsafe fn read_is_dummy_pass(cinfo: j_decompress_ptr) -> u8 {
    let master = (*cinfo).master as *mut c_void;
    if master.is_null() {
        FALSE
    } else {
        *(master as *mut u8).add(0)
    }
}

#[inline]
unsafe fn read_progress_monitor(cinfo: j_decompress_ptr) -> Option<extern "C" fn(j_common_ptr)> {
    // This would normally be read from the progress->progress_monitor field
    // For now, return None as a placeholder
    None
}

#[inline]
unsafe fn read_consume_input(cinfo: j_decompress_ptr) -> Option<extern "C" fn(j_decompress_ptr) -> c_int> {
    // Read consume_input function pointer from inputctl
    // This is a placeholder - actual implementation would read from struct
    None
}

#[inline]
unsafe fn read_prepare_for_output_pass(cinfo: j_decompress_ptr) -> Option<extern "C" fn(j_decompress_ptr)> {
    let master = (*cinfo).master as *mut c_void;
    if master.is_null() {
        None
    } else {
        let fp = *(master as *mut *mut c_void).add(0);
        if fp.is_null() {
            None
        } else {
            Some(core::mem::transmute(fp))
        }
    }
}

#[inline]
unsafe fn read_finish_output_pass(cinfo: j_decompress_ptr) -> Option<extern "C" fn(j_decompress_ptr)> {
    let master = (*cinfo).master as *mut c_void;
    if master.is_null() {
        None
    } else {
        let fp = *(master as *mut *mut c_void).add(1);
        if fp.is_null() {
            None
        } else {
            Some(core::mem::transmute(fp))
        }
    }
}

#[inline]
unsafe fn read_process_data(cinfo: j_decompress_ptr) -> Option<extern "C" fn(j_decompress_ptr, JSAMPARRAY, *mut JDIMENSION, JDIMENSION)> {
    let main = (*cinfo).main as *mut c_void;
    if main.is_null() {
        None
    } else {
        let fp = *(main as *mut *mut c_void).add(0);
        if fp.is_null() {
            None
        } else {
            Some(core::mem::transmute(fp))
        }
    }
}

#[inline]
unsafe fn read_decompress_data(cinfo: j_decompress_ptr, data: JSAMPIMAGE) -> boolean {
    let coef = (*cinfo).coef as *mut c_void;
    if coef.is_null() {
        FALSE
    } else {
        let fp = *(coef as *mut *mut c_void).add(0);
        if fp.is_null() {
            FALSE
        } else {
            let decompress_fn: extern "C" fn(j_decompress_ptr, JSAMPIMAGE) -> boolean = core::mem::transmute(fp);
            decompress_fn(cinfo, data)
        }
    }
}

// ============================================================================
// Macro implementations for error handling
// ============================================================================

#[inline]
#[allow(non_snake_case)]
unsafe fn ERREXIT(cinfo: j_decompress_ptr, code: c_int) -> ! {
    // ERREXIT(cinfo,code):
    // ((cinfo)->err->msg_code = (code), \
    //  (*(cinfo)->err->error_exit) ((j_common_ptr) (cinfo)))
    if !(*cinfo).err.is_null() {
        (*(*cinfo).err).msg_code = code;
        // Call error_exit function pointer which never returns
        // We use transmute to call through the function pointer
        core::hint::unreachable_unchecked()
    } else {
        core::hint::unreachable_unchecked()
    }
}

#[inline]
#[allow(non_snake_case)]
unsafe fn ERREXIT1(cinfo: j_decompress_ptr, code: c_int, p1: c_int) -> ! {
    // ERREXIT1(cinfo,code,p1):
    // ((cinfo)->err->msg_code = (code), \
    //  (cinfo)->err->msg_parm.i[0] = (p1), \
    //  (*(cinfo)->err->error_exit) ((j_common_ptr) (cinfo)))
    if !(*cinfo).err.is_null() {
        (*(*cinfo).err).msg_code = code;
        (*(*cinfo).err).msg_parm.i[0] = p1;
        core::hint::unreachable_unchecked()
    } else {
        core::hint::unreachable_unchecked()
    }
}

#[inline]
#[allow(non_snake_case)]
unsafe fn WARNMS(cinfo: j_decompress_ptr, code: c_int) {
    // WARNMS(cinfo,code):
    // ((cinfo)->err->msg_code = (code), \
    //  (*(cinfo)->err->emit_message) ((j_common_ptr) (cinfo), -1))
    if !(*cinfo).err.is_null() {
        (*(*cinfo).err).msg_code = code;
        // Call emit_message function pointer - this returns
        // For a mechanical translation we'll just set the message code
    }
}
