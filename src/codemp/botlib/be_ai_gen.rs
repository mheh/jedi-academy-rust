/*****************************************************************************
 * name:		be_ai_gen.c
 *
 * desc:		genetic selection
 *
 * $Archive: /MissionPack/code/botlib/be_ai_gen.c $
 * $Author: Zaphod $
 * $Revision: 3 $
 * $Modtime: 11/22/00 8:50a $
 * $Date: 11/22/00 8:55a $
 *
 *****************************************************************************/

use core::ffi::{c_int, c_char};

// Stub for botimport structure defined in be_interface.h
// Print function pointer takes a print level and a format string
#[repr(C)]
pub struct BotImport {
    // Simplified stub - the actual structure has more fields
    // Only including Print since it's used in this module
    pub Print: extern "C" fn(c_int, *const c_char) -> (),
}

extern "C" {
    pub static botimport: BotImport;
    // random() - returns a float in range [0, 1)
    pub fn random() -> f32;
    // Com_Memcpy - copies memory, mirrors libc memcpy
    pub fn Com_Memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, count: usize) -> *mut core::ffi::c_void;
}

// Print level constants - stub value from be_interface.h
// FIXME: verify actual value from be_interface.h header
const PRT_WARNING: c_int = 1;

const qfalse: c_int = 0;
const qtrue: c_int = 1;

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
#[no_mangle]
pub extern "C" fn GeneticSelection(numranks: c_int, rankings: *mut f32) -> c_int {
    let mut sum: f32;
    let mut select: f32;
    let mut i: c_int;
    let mut index: c_int;

    sum = 0.0;
    i = 0;
    while i < numranks {
        unsafe {
            if *rankings.offset(i as isize) < 0.0 {
                i += 1;
                continue;
            }
            sum += *rankings.offset(i as isize);
        }
        i += 1;
    } //end for
    if sum > 0.0 {
        //select a bot where the ones with the higest rankings have
        //the highest chance of being selected
        unsafe {
            select = random() * sum;
        }
        i = 0;
        while i < numranks {
            unsafe {
                if *rankings.offset(i as isize) < 0.0 {
                    i += 1;
                    continue;
                }
                sum -= *rankings.offset(i as isize);
                if sum <= 0.0 {
                    return i;
                }
            }
            i += 1;
        } //end for
    } //end if
    //select a bot randomly
    unsafe {
        index = (random() * (numranks as f32)) as c_int;
    }
    i = 0;
    while i < numranks {
        unsafe {
            if *rankings.offset(index as isize) >= 0.0 {
                return index;
            }
        }
        index = (index + 1) % numranks;
        i += 1;
    } //end for
    return 0;
} //end of the function GeneticSelection
//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
#[no_mangle]
pub extern "C" fn GeneticParentsAndChildSelection(
    numranks: c_int,
    ranks: *mut f32,
    parent1: *mut c_int,
    parent2: *mut c_int,
    child: *mut c_int,
) -> c_int {
    let mut rankings: [f32; 256] = [0.0; 256];
    let mut max: f32;
    let mut i: c_int;

    if numranks > 256 {
        unsafe {
            (botimport.Print)(PRT_WARNING, "GeneticParentsAndChildSelection: too many bots\n".as_ptr() as *const c_char);
        }
        unsafe {
            *parent1 = 0;
            *parent2 = 0;
            *child = 0;
        }
        return qfalse;
    } //end if
    max = 0.0;
    i = 0;
    while i < numranks {
        unsafe {
            if *ranks.offset(i as isize) < 0.0 {
                i += 1;
                continue;
            }
            max += 1.0;
        }
        i += 1;
    } //end for
    if max < 3.0 {
        unsafe {
            (botimport.Print)(PRT_WARNING, "GeneticParentsAndChildSelection: too few valid bots\n".as_ptr() as *const c_char);
        }
        unsafe {
            *parent1 = 0;
            *parent2 = 0;
            *child = 0;
        }
        return qfalse;
    } //end if
    unsafe {
        Com_Memcpy(
            rankings.as_mut_ptr() as *mut core::ffi::c_void,
            ranks as *const core::ffi::c_void,
            (core::mem::size_of::<f32>() * numranks as usize),
        );
    }
    //select first parent
    unsafe {
        *parent1 = GeneticSelection(numranks, rankings.as_mut_ptr());
        *rankings.get_unchecked_mut(*parent1 as usize) = -1.0;
    }
    //select second parent
    unsafe {
        *parent2 = GeneticSelection(numranks, rankings.as_mut_ptr());
        *rankings.get_unchecked_mut(*parent2 as usize) = -1.0;
    }
    //reverse the rankings
    max = 0.0;
    i = 0;
    while i < numranks {
        unsafe {
            if *rankings.get_unchecked(i as usize) < 0.0 {
                i += 1;
                continue;
            }
            if *rankings.get_unchecked(i as usize) > max {
                max = *rankings.get_unchecked(i as usize);
            }
        }
        i += 1;
    } //end for
    i = 0;
    while i < numranks {
        unsafe {
            if *rankings.get_unchecked(i as usize) < 0.0 {
                i += 1;
                continue;
            }
            *rankings.get_unchecked_mut(i as usize) = max - *rankings.get_unchecked(i as usize);
        }
        i += 1;
    } //end for
    //select child
    unsafe {
        *child = GeneticSelection(numranks, rankings.as_mut_ptr());
    }
    return qtrue;
} //end of the function GeneticParentsAndChildSelection
