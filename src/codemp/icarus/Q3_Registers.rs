// Anything above this #include will be ignored by the compiler

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::ptr;

use crate::codemp::icarus::Q3_Registers_h::{
    varString_m, varFloat_m, varStrings, varFloats, varVectors,
    VTYPE_FLOAT, VTYPE_STRING, VTYPE_VECTOR, VTYPE_NONE, MAX_VARIABLES, vec3_t,
};

extern "C" {
    fn Q3_DebugPrint(level: c_int, format: *const c_char, ...);
}

// Token type constants
const TK_FLOAT: c_int = 6;
const TK_STRING: c_int = 4;
const TK_VECTOR: c_int = 14;

// Debug print levels
const WL_ERROR: c_int = 1;
const WL_WARNING: c_int = 2;

// Rust equivalents of C++ true/false
const true_val: c_int = 1;
const false_val: c_int = 0;

/*
-------------------------
Q3_VariableDeclared
-------------------------
*/

pub extern "C" fn Q3_VariableDeclared(name: *const c_char) -> c_int {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return VTYPE_NONE,
    };

    // Check the strings
    unsafe {
        if let Some(ref strings_map) = *ptr::addr_of!(varStrings) {
            if strings_map.contains_key(name_str) {
                return VTYPE_STRING;
            }
        }

        // Check the floats
        if let Some(ref floats_map) = *ptr::addr_of!(varFloats) {
            if floats_map.contains_key(name_str) {
                return VTYPE_FLOAT;
            }
        }

        // Check the vectors
        if let Some(ref vectors_map) = *ptr::addr_of!(varVectors) {
            if vectors_map.contains_key(name_str) {
                return VTYPE_VECTOR;
            }
        }
    }

    VTYPE_NONE
}

/*
-------------------------
Q3_DeclareVariable
-------------------------
*/

pub extern "C" fn Q3_DeclareVariable(r#type: c_int, name: *const c_char) {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return,
    };

    // Cannot declare the same variable twice
    if Q3_VariableDeclared(name) != VTYPE_NONE {
        return;
    }

    unsafe {
        // Count current variables
        let num_variables = {
            let mut count = 0usize;
            if let Some(ref strings_map) = *ptr::addr_of!(varStrings) {
                count += strings_map.len();
            }
            if let Some(ref floats_map) = *ptr::addr_of!(varFloats) {
                count += floats_map.len();
            }
            if let Some(ref vectors_map) = *ptr::addr_of!(varVectors) {
                count += vectors_map.len();
            }
            count
        };

        if num_variables >= MAX_VARIABLES {
            Q3_DebugPrint(WL_ERROR, "too many variables already declared, maximum is %d\n\0".as_ptr() as *const c_char, MAX_VARIABLES as c_int);
            return;
        }

        match r#type {
            TK_FLOAT => {
                if let Some(ref mut floats_map) = *ptr::addr_of_mut!(varFloats) {
                    floats_map.insert(name_str, 0.0f32);
                }
            }
            TK_STRING => {
                if let Some(ref mut strings_map) = *ptr::addr_of_mut!(varStrings) {
                    strings_map.insert(name_str, "NULL".to_string());
                }
            }
            TK_VECTOR => {
                if let Some(ref mut vectors_map) = *ptr::addr_of_mut!(varVectors) {
                    vectors_map.insert(name_str, "0.0 0.0 0.0".to_string());
                }
            }
            _ => {
                Q3_DebugPrint(WL_ERROR, "unknown 'type' for declare() function!\n\0".as_ptr() as *const c_char);
                return;
            }
        }
    }
}

/*
-------------------------
Q3_FreeVariable
-------------------------
*/

pub extern "C" fn Q3_FreeVariable(name: *const c_char) {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    unsafe {
        // Check the strings
        if let Some(ref mut strings_map) = *ptr::addr_of_mut!(varStrings) {
            if strings_map.remove(name_str).is_some() {
                return;
            }
        }

        // Check the floats
        if let Some(ref mut floats_map) = *ptr::addr_of_mut!(varFloats) {
            if floats_map.remove(name_str).is_some() {
                return;
            }
        }

        // Check the vectors
        if let Some(ref mut vectors_map) = *ptr::addr_of_mut!(varVectors) {
            if vectors_map.remove(name_str).is_some() {
                return;
            }
        }
    }
}

/*
-------------------------
Q3_GetFloatVariable
-------------------------
*/

pub extern "C" fn Q3_GetFloatVariable(name: *const c_char, value: *mut f32) -> c_int {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false_val,
    };

    // Check the floats
    unsafe {
        if let Some(ref floats_map) = *ptr::addr_of!(varFloats) {
            if let Some(&float_val) = floats_map.get(name_str) {
                *value = float_val;
                return true_val;
            }
        }
    }

    false_val
}

/*
-------------------------
Q3_GetStringVariable
-------------------------
*/

pub extern "C" fn Q3_GetStringVariable(name: *const c_char, value: *mut *const c_char) -> c_int {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false_val,
    };

    // Check the strings
    unsafe {
        if let Some(ref strings_map) = *ptr::addr_of!(varStrings) {
            if let Some(string_val) = strings_map.get(name_str) {
                *value = string_val.as_ptr() as *const c_char;
                return true_val;
            }
        }
    }

    false_val
}

/*
-------------------------
Q3_GetVectorVariable
-------------------------
*/

pub extern "C" fn Q3_GetVectorVariable(name: *const c_char, value: vec3_t) -> c_int {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false_val,
    };

    // Check the vectors
    unsafe {
        if let Some(ref vectors_map) = *ptr::addr_of!(varVectors) {
            if let Some(vector_str) = vectors_map.get(name_str) {
                // Parse the vector string
                let mut parts = vector_str.split_whitespace();
                if let (Some(x_str), Some(y_str), Some(z_str)) = (parts.next(), parts.next(), parts.next()) {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        x_str.parse::<f32>(),
                        y_str.parse::<f32>(),
                        z_str.parse::<f32>(),
                    ) {
                        let value_ptr = value.as_ptr() as *mut f32;
                        *value_ptr = x;
                        *value_ptr.add(1) = y;
                        *value_ptr.add(2) = z;
                        return true_val;
                    }
                }
            }
        }
    }

    false_val
}

/*
-------------------------
Q3_InitVariables
-------------------------
*/

pub extern "C" fn Q3_InitVariables() {
    unsafe {
        if let Some(ref mut strings_map) = *ptr::addr_of_mut!(varStrings) {
            strings_map.clear();
        }
        if let Some(ref mut floats_map) = *ptr::addr_of_mut!(varFloats) {
            floats_map.clear();
        }
        if let Some(ref mut vectors_map) = *ptr::addr_of_mut!(varVectors) {
            vectors_map.clear();
        }
    }

    // Count residual variables
    unsafe {
        let num_residual = {
            let mut count = 0usize;
            if let Some(ref strings_map) = *ptr::addr_of!(varStrings) {
                count += strings_map.len();
            }
            if let Some(ref floats_map) = *ptr::addr_of!(varFloats) {
                count += floats_map.len();
            }
            if let Some(ref vectors_map) = *ptr::addr_of!(varVectors) {
                count += vectors_map.len();
            }
            count
        };

        if num_residual > 0 {
            Q3_DebugPrint(WL_WARNING, "%d residual variables found!\n\0".as_ptr() as *const c_char, num_residual as c_int);
        }
    }
}

/*
-------------------------
Q3_SetVariable_Float
-------------------------
*/

pub extern "C" fn Q3_SetFloatVariable(name: *const c_char, value: f32) -> c_int {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false_val,
    };

    // Check the floats
    unsafe {
        if let Some(ref mut floats_map) = *ptr::addr_of_mut!(varFloats) {
            if floats_map.contains_key(name_str) {
                floats_map.insert(name_str.to_string(), value);
                return true_val;
            }
        }
    }

    VTYPE_FLOAT
}

/*
-------------------------
Q3_SetVariable_String
-------------------------
*/

pub extern "C" fn Q3_SetStringVariable(name: *const c_char, value: *const c_char) -> c_int {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false_val,
    };

    let value_cstr = unsafe { CStr::from_ptr(value) };
    let value_str = match value_cstr.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return false_val,
    };

    // Check the strings
    unsafe {
        if let Some(ref mut strings_map) = *ptr::addr_of_mut!(varStrings) {
            if strings_map.contains_key(name_str) {
                strings_map.insert(name_str.to_string(), value_str);
                return true_val;
            }
        }
    }

    false_val
}

/*
-------------------------
Q3_SetVariable_Vector
-------------------------
*/

pub extern "C" fn Q3_SetVectorVariable(name: *const c_char, value: *const c_char) -> c_int {
    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false_val,
    };

    let value_cstr = unsafe { CStr::from_ptr(value) };
    let value_str = match value_cstr.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return false_val,
    };

    // Check the vectors
    unsafe {
        if let Some(ref mut vectors_map) = *ptr::addr_of_mut!(varVectors) {
            if vectors_map.contains_key(name_str) {
                vectors_map.insert(name_str.to_string(), value_str);
                return true_val;
            }
        }
    }

    false_val
}

/*
-------------------------
Q3_VariableSaveFloats
-------------------------
*/

pub extern "C" fn Q3_VariableSaveFloats(_fmap: *mut varFloat_m) {
    // Empty function - original was commented out
    return;
    /*
    int numFloats = fmap.size();
    gi.AppendToSaveGame( 'FVAR', &numFloats, sizeof( numFloats ) );

    varFloat_m::iterator	vfi;
    STL_ITERATE( vfi, fmap )
    {
        //Save out the map id
        int	idSize = strlen( ((*vfi).first).c_str() );

        //Save out the real data
        gi.AppendToSaveGame( 'FIDL', &idSize, sizeof( idSize ) );
        gi.AppendToSaveGame( 'FIDS', (void *) ((*vfi).first).c_str(), idSize );

        //Save out the float value
        gi.AppendToSaveGame( 'FVAL', &((*vfi).second), sizeof( float ) );
    }
    */
}

/*
-------------------------
Q3_VariableSaveStrings
-------------------------
*/

pub extern "C" fn Q3_VariableSaveStrings(_smap: *mut varString_m) {
    // Empty function - original was commented out
    return;
    /*
    int numStrings = smap.size();
    gi.AppendToSaveGame( 'SVAR', &numStrings, sizeof( numStrings ) );

    varString_m::iterator	vsi;
    STL_ITERATE( vsi, smap )
    {
        //Save out the map id
        int	idSize = strlen( ((*vsi).first).c_str() );

        //Save out the real data
        gi.AppendToSaveGame( 'SIDL', &idSize, sizeof( idSize ) );
        gi.AppendToSaveGame( 'SIDS', (void *) ((*vsi).first).c_str(), idSize );

        //Save out the string value
        idSize = strlen( ((*vsi).second).c_str() );

        gi.AppendToSaveGame( 'SVSZ', &idSize, sizeof( idSize ) );
        gi.AppendToSaveGame( 'SVAL', (void *) ((*vsi).second).c_str(), idSize );
    }
    */
}

/*
-------------------------
Q3_VariableSave
-------------------------
*/

pub extern "C" fn Q3_VariableSave() -> c_int {
    unsafe {
        if let Some(ref mut floats_map) = *ptr::addr_of_mut!(varFloats) {
            Q3_VariableSaveFloats(floats_map as *mut varFloat_m);
        }
        if let Some(ref mut strings_map) = *ptr::addr_of_mut!(varStrings) {
            Q3_VariableSaveStrings(strings_map as *mut varString_m);
        }
        if let Some(ref mut vectors_map) = *ptr::addr_of_mut!(varVectors) {
            Q3_VariableSaveStrings(vectors_map as *mut varString_m);
        }
    }

    1  // qtrue
}

/*
-------------------------
Q3_VariableLoadFloats
-------------------------
*/

pub extern "C" fn Q3_VariableLoadFloats(_fmap: *mut varFloat_m) {
    // Empty function - original was commented out
    return;
    /*
    int		numFloats;
    char	tempBuffer[1024];

    gi.ReadFromSaveGame( 'FVAR', &numFloats, sizeof( numFloats ) );

    for ( int i = 0; i < numFloats; i++ )
    {
        int idSize;

        gi.ReadFromSaveGame( 'FIDL', &idSize, sizeof( idSize ) );
        gi.ReadFromSaveGame( 'FIDS', &tempBuffer, idSize );
        tempBuffer[ idSize ] = 0;

        float	val;

        gi.ReadFromSaveGame( 'FVAL', &val, sizeof( float ) );

        Q3_DeclareVariable( TK_FLOAT, (const char *) &tempBuffer );
        Q3_SetFloatVariable( (const char *) &tempBuffer, val );
    }
    */
}

/*
-------------------------
Q3_VariableLoadStrings
-------------------------
*/

pub extern "C" fn Q3_VariableLoadStrings(_r#type: c_int, _fmap: *mut varString_m) {
    // Empty function - original was commented out
    return;
    /*
    int		numFloats;
    char	tempBuffer[1024];
    char	tempBuffer2[1024];

    gi.ReadFromSaveGame( 'SVAR', &numFloats, sizeof( numFloats ) );

    for ( int i = 0; i < numFloats; i++ )
    {
        int idSize;

        gi.ReadFromSaveGame( 'SIDL', &idSize, sizeof( idSize ) );
        gi.ReadFromSaveGame( 'SIDS', &tempBuffer, idSize );
        tempBuffer[ idSize ] = 0;

        gi.ReadFromSaveGame( 'SVSZ', &idSize, sizeof( idSize ) );
        gi.ReadFromSaveGame( 'SVAL', &tempBuffer2, idSize );
        tempBuffer2[ idSize ] = 0;

        switch ( type )
        {
        case TK_STRING:
            Q3_DeclareVariable( TK_STRING, (const char *) &tempBuffer );
            Q3_SetStringVariable( (const char *) &tempBuffer, (const char *) &tempBuffer2 );
            break;

        case TK_VECTOR:
            Q3_DeclareVariable( TK_VECTOR, (const char *) &tempBuffer );
            Q3_SetVectorVariable( (const char *) &tempBuffer, (const char *) &tempBuffer2 );
            break;
        }
    }
    */
}

/*
-------------------------
Q3_VariableLoad
-------------------------
*/

pub extern "C" fn Q3_VariableLoad() -> c_int {
    Q3_InitVariables();

    unsafe {
        if let Some(ref mut floats_map) = *ptr::addr_of_mut!(varFloats) {
            Q3_VariableLoadFloats(floats_map as *mut varFloat_m);
        }
        if let Some(ref mut strings_map) = *ptr::addr_of_mut!(varStrings) {
            Q3_VariableLoadStrings(TK_STRING, strings_map as *mut varString_m);
        }
        if let Some(ref mut vectors_map) = *ptr::addr_of_mut!(varVectors) {
            Q3_VariableLoadStrings(TK_VECTOR, vectors_map as *mut varString_m);
        }
    }

    0  // qfalse
}
