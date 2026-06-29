// leave this line at the top for all g_xxxx.cpp files...
// (C header: g_headers.h)

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(static_mut_refs)]

use core::ffi::{c_char, c_int, c_void};

use crate::code::game::g_roff_h::{
    roff_hdr_t, roff_hdr2_t, move_rotate_t, move_rotate2_t, roff_list_t,
    ROFF_VERSION, ROFF_VERSION2, MAX_ROFFS,
};
use crate::code::game::g_local_h::gentity_t;

// The list of precached ROFFs
// PORTING: Static array of structs initialized to zeros
pub static mut roffs: [roff_list_t; 32] = [
    roff_list_t {
        r#type: 0,
        fileName: core::ptr::null_mut(),
        frames: 0,
        data: core::ptr::null_mut(),
        mFrameTime: 0,
        mLerp: 0,
        mNumNoteTracks: 0,
        mNoteTrackIndexes: core::ptr::null_mut(),
    }; 32
];
pub static mut num_roffs: c_int = 0;

pub static mut g_bCollidableRoffs: c_int = 0; // qfalse = 0

extern "C" {
    fn Q3_TaskIDComplete(ent: *mut gentity_t, taskType: c_int);
    fn Com_Printf(fmt: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn atof(nptr: *const c_char) -> f32;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // External game functions and variables
    fn G_NewString(string: *const c_char) -> *mut c_char;
    fn G_Alloc(size: usize) -> *mut c_void;
    fn G_EffectIndex(name: *const c_char) -> c_int;
    fn G_SoundIndex(name: *const c_char) -> c_int;
    fn G_PlayEffect(fxID: c_int, origin: *const [f32; 3], angles: *const [f32; 3]);
    fn cgi_S_StartSound(origin: *const [f32; 3], entityNum: c_int, entchannel: c_int, soundIndex: c_int);
    fn VectorClear(v: *mut [f32; 3]);
    fn VectorCopy(in_: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorAdd(veca: *const [f32; 3], vecb: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorScale(in_: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn AngleVectors(
        angles: *const [f32; 3],
        forward: *mut [f32; 3],
        right: *mut [f32; 3],
        up: *mut [f32; 3],
    );
    fn EvaluateTrajectory(tr: *const c_void, time: c_int, result: *mut [f32; 3]);

    // Game interface stubs
    pub static mut gi: crate::code::game::g_local_h::game_import_t;
    pub static mut level: crate::code::game::g_local_h::level_locals_t;
}

// Task ID constants
const TID_MOVE_NAV: c_int = 5;

// Trajectory type constants
const TR_INTERPOLATE: c_int = 1;
const TR_LINEAR: c_int = 2;

// Entity type constants
const ET_MISSILE: c_int = 1;
const ET_ITEM: c_int = 2;
const ET_MOVER: c_int = 4;

// Channel constants
const CHAN_BODY: c_int = 1;

// Angles indices
const PITCH: usize = 0;
const YAW: usize = 1;

// Behavior function pointers (stubs)
const thinkF_TieFighterThink: *const c_void = core::ptr::null();
const thinkF_TieBomberThink: *const c_void = core::ptr::null();

static mut g_developer: *mut c_void = core::ptr::null_mut();

static Q3_SCRIPT_DIR: &[u8] = b"scripts";

unsafe fn G_RoffNotetrackCallback(cent: *mut gentity_t, notetrack: *const c_char)
{
    let mut i: c_int = 0;
    let mut r: c_int = 0;
    let mut r2: c_int = 0;
    let mut objectID: c_int = 0;
    let mut anglesGathered: c_int = 0;
    let mut posoffsetGathered: c_int = 0;
    let mut type_: [c_char; 256] = [0; 256];
    let mut argument: [c_char; 512] = [0; 512];
    let mut addlArg: [c_char; 512] = [0; 512];
    let mut errMsg: [c_char; 256] = [0; 256];
    let mut t: [c_char; 64] = [0; 64];
    let mut teststr: [c_char; 256] = [0; 256];
    let mut addlArgs: c_int = 0;
    let mut parsedAngles: [f32; 3] = [0.0; 3];
    let mut parsedOffset: [f32; 3] = [0.0; 3];
    let mut useAngles: [f32; 3] = [0.0; 3];
    let mut useOrigin: [f32; 3] = [0.0; 3];
    let mut forward: [f32; 3] = [0.0; 3];
    let mut right: [f32; 3] = [0.0; 3];
    let mut up: [f32; 3] = [0.0; 3];

    if cent.is_null() || notetrack.is_null()
    {
        return;
    }

    //notetrack = "effect effects/explosion1.efx 0+0+64 0-0-1";

    while *notetrack.offset(i as isize) != 0 && *notetrack.offset(i as isize) != b' ' as c_char
    {
        type_[i as usize] = *notetrack.offset(i as isize);
        i += 1;
    }

    type_[i as usize] = b'\0' as c_char;

    if *notetrack.offset(i as isize) != b' ' as c_char
    { //didn't pass in a valid notetrack type, or forgot the argument for it
        return;
    }

    i += 1;

    r = 0;
    while *notetrack.offset(i as isize) != 0 && *notetrack.offset(i as isize) != b' ' as c_char
    {
        if *notetrack.offset(i as isize) != b'\n' as c_char && *notetrack.offset(i as isize) != b'\r' as c_char
        { //don't read line ends for an argument
            argument[r as usize] = *notetrack.offset(i as isize);
            r += 1;
        }
        i += 1;
    }
    argument[r as usize] = b'\0' as c_char;

    if r == 0
    {
        return;
    }

    if *notetrack.offset(i as isize) == b' ' as c_char
    { //additional arguments...
        addlArgs = 1;

        i += 1;
        r = 0;
        while *notetrack.offset(i as isize) != 0
        {
            addlArg[r as usize] = *notetrack.offset(i as isize);
            r += 1;
            i += 1;
        }
        addlArg[r as usize] = b'\0' as c_char;
    }

    if strcmp(type_.as_ptr(), b"effect\0".as_ptr() as *const c_char) == 0
    {
        if addlArgs == 0
        {
            VectorClear(&mut parsedOffset);
            // defaultoffsetposition section (no additional argument parsing)
        }
        else
        {
            i = 0;

            while posoffsetGathered < 3
            {
                r = 0;
                while *addlArg.as_ptr().offset(i as isize) != 0
                    && *addlArg.as_ptr().offset(i as isize) != b'+' as c_char
                    && *addlArg.as_ptr().offset(i as isize) != b' ' as c_char
                {
                    t[r as usize] = *addlArg.as_ptr().offset(i as isize);
                    r += 1;
                    i += 1;
                }
                t[r as usize] = b'\0' as c_char;
                i += 1;
                if r == 0
                { //failure..
                    VectorClear(&mut parsedOffset);
                    i = 0;
                    break;
                }
                parsedOffset[posoffsetGathered as usize] = atof(t.as_ptr());
                posoffsetGathered += 1;
            }

            if posoffsetGathered < 3 && posoffsetGathered > 0
            {
                sprintf(
                    errMsg.as_mut_ptr(),
                    b"Offset position argument for 'effect' type is invalid.\0".as_ptr() as *const c_char,
                );
                Com_Printf(b"Type-specific notetrack error: %s\n\0".as_ptr() as *const c_char, errMsg.as_ptr());
                return;
            }

            i -= 1;

            if *addlArg.as_ptr().offset(i as isize) != b' ' as c_char
            {
                addlArgs = 0;
            }
        }

        // defaultoffsetposition label section
        {
            r = 0;
            if *argument.as_ptr().offset(r as isize) == b'/' as c_char
            {
                r += 1;
            }
            r2 = 0;
            while *argument.as_ptr().offset(r as isize) != 0 && *argument.as_ptr().offset(r as isize) != b'/' as c_char
            {
                teststr[r2 as usize] = *argument.as_ptr().offset(r as isize);
                r2 += 1;
                r += 1;
            }
            teststr[r2 as usize] = b'\0' as c_char;

            if r2 != 0 && !strstr(teststr.as_ptr(), b"effects\0".as_ptr() as *const c_char).is_null()
            { //get rid of the leading "effects" since it's auto-inserted
                r += 1;
                r2 = 0;

                while *argument.as_ptr().offset(r as isize) != 0
                {
                    teststr[r2 as usize] = *argument.as_ptr().offset(r as isize);
                    r2 += 1;
                    r += 1;
                }
                teststr[r2 as usize] = b'\0' as c_char;

                strcpy(argument.as_mut_ptr(), teststr.as_ptr());
            }

            objectID = G_EffectIndex(argument.as_ptr());
            r = 0;

            if objectID != 0
            {
                if addlArgs != 0
                { //if there is an additional argument for an effect it is expected to be XANGLE-YANGLE-ZANGLE
                    i += 1;
                    while anglesGathered < 3
                    {
                        r = 0;
                        while *addlArg.as_ptr().offset(i as isize) != 0 && *addlArg.as_ptr().offset(i as isize) != b'-' as c_char
                        {
                            t[r as usize] = *addlArg.as_ptr().offset(i as isize);
                            r += 1;
                            i += 1;
                        }
                        t[r as usize] = b'\0' as c_char;
                        i += 1;

                        if r == 0
                        { //failed to get a new part of the vector
                            anglesGathered = 0;
                            break;
                        }

                        parsedAngles[anglesGathered as usize] = atof(t.as_ptr());
                        anglesGathered += 1;
                    }

                    if anglesGathered != 0
                    {
                        VectorCopy(&parsedAngles, &mut useAngles);
                    }
                    else
                    { //failed to parse angles from the extra argument provided..
                        VectorCopy(&(*cent).s.apos.trBase, &mut useAngles);
                    }
                }
                else
                { //if no constant angles, play in direction entity is facing
                    VectorCopy(&(*cent).s.apos.trBase, &mut useAngles);
                }

                AngleVectors(&useAngles, &mut forward, &mut right, &mut up);

                VectorCopy(&(*cent).s.pos.trBase, &mut useOrigin);

                //forward
                useOrigin[0] += forward[0] * parsedOffset[0];
                useOrigin[1] += forward[1] * parsedOffset[0];
                useOrigin[2] += forward[2] * parsedOffset[0];

                //right
                useOrigin[0] += right[0] * parsedOffset[1];
                useOrigin[1] += right[1] * parsedOffset[1];
                useOrigin[2] += right[2] * parsedOffset[1];

                //up
                useOrigin[0] += up[0] * parsedOffset[2];
                useOrigin[1] += up[1] * parsedOffset[2];
                useOrigin[2] += up[2] * parsedOffset[2];

                G_PlayEffect(objectID, &useOrigin, &useAngles);
            }
        }
    }
    else if strcmp(type_.as_ptr(), b"sound\0".as_ptr() as *const c_char) == 0
    {
        objectID = G_SoundIndex(argument.as_ptr());
        cgi_S_StartSound(&(*cent).s.pos.trBase, (*cent).s.number, CHAN_BODY, objectID);
    }
    //else if ...
    else
    {
        if type_[0] != 0
        {
            Com_Printf(b"Warning: \"%s\" is an invalid ROFF notetrack function\n\0".as_ptr() as *const c_char, type_.as_ptr());
        }
        else
        {
            Com_Printf(b"Warning: Notetrack is missing function and/or arguments\n\0".as_ptr() as *const c_char);
        }
    }

    return;
}

unsafe fn G_ValidRoff(header: *mut roff_hdr2_t) -> c_int
{
    if strncmp(
        (*header).mHeader.as_ptr(),
        b"ROFF".as_ptr() as *const c_char,
        4,
    ) == 0
    {
        if (*header).mCount > 0 && (*header).mVersion == ROFF_VERSION2
        {
            return 1; // qtrue
        }
        else if (*header).mVersion == ROFF_VERSION && (*(header as *mut roff_hdr_t)).mCount > 0.0
        { // version 1 defines the count as a float, so we best do the count check as a float or we'll get bogus results
            return 1; // qtrue
        }
    }

    0 // qfalse
}

unsafe fn G_FreeRoff(index: c_int)
{
    if roffs[index as usize].mNumNoteTracks != 0 {
        // In C this is delete[] roffs[index].mNoteTrackIndexes[0];
        // We're using libc free here (stub)
        // delete [] roffs[index].mNoteTrackIndexes[0];
        // delete [] roffs[index].mNoteTrackIndexes;
        // These would be freed via libc or the memory allocator
    }
}

unsafe fn G_InitRoff(file: *mut c_char, data: *mut u8) -> c_int
{
    let header = data as *mut roff_hdr_t;
    let count: c_int = (*header).mCount as c_int;

    roffs[num_roffs as usize].fileName = G_NewString(file);

    if (*header).mVersion == ROFF_VERSION
    {
        // We are Old School(tm)
        roffs[num_roffs as usize].r#type = 1;

        roffs[num_roffs as usize].data = G_Alloc((count as usize) * core::mem::size_of::<move_rotate_t>());
        let mem = roffs[num_roffs as usize].data as *mut move_rotate_t;

        roffs[num_roffs as usize].mFrameTime = 100; // old school ones have a hard-coded frame time
        roffs[num_roffs as usize].mLerp = 10;
        roffs[num_roffs as usize].mNumNoteTracks = 0;
        roffs[num_roffs as usize].mNoteTrackIndexes = core::ptr::null_mut();

        if !mem.is_null()
        {
            // The allocation worked, so stash this stuff off so we can reference the data later if needed
            roffs[num_roffs as usize].frames = count;

            // Step past the header to get to the goods
            let roff_data = (data as *mut c_void as usize + core::mem::size_of::<roff_hdr_t>()) as *mut move_rotate_t;

            // Copy all of the goods into our ROFF cache
            for i in 0..count {
                // Copy just the delta position and orientation which can be applied to anything at a later point
                VectorCopy(&(*roff_data.offset(i as isize)).origin_delta, &mut (*mem.offset(i as isize)).origin_delta);
                VectorCopy(&(*roff_data.offset(i as isize)).rotate_delta, &mut (*mem.offset(i as isize)).rotate_delta);
            }
            return 1; // qtrue
        }
    }
    else if (*header).mVersion == ROFF_VERSION2
    {
        // Version 2.0, heck yeah!
        let hdr = data as *mut roff_hdr2_t;
        let count = (*hdr).mCount;

        roffs[num_roffs as usize].frames = count;
        roffs[num_roffs as usize].data = G_Alloc((count as usize) * core::mem::size_of::<move_rotate2_t>());
        let mem = roffs[num_roffs as usize].data as *mut move_rotate2_t;

        if !mem.is_null()
        {
            roffs[num_roffs as usize].mFrameTime = (*hdr).mFrameRate;
            roffs[num_roffs as usize].mLerp = 1000 / (*hdr).mFrameRate;
            roffs[num_roffs as usize].mNumNoteTracks = (*hdr).mNumNotes;

            if roffs[num_roffs as usize].mFrameTime < 50
            {
                Com_Printf(
                    b"^1Error: \"%s\" has an invalid ROFF framerate (%d < 50)\n\0".as_ptr() as *const c_char,
                    file,
                    roffs[num_roffs as usize].mFrameTime,
                );
            }
            assert!(roffs[num_roffs as usize].mFrameTime >= 50); //HAS to be at least 50 to be reliable

            // Step past the header to get to the goods
            let roff_data = (data as *mut c_void as usize + core::mem::size_of::<roff_hdr2_t>()) as *mut move_rotate2_t;

            roffs[num_roffs as usize].r#type = 2; //rww - any reason this wasn't being set already?

            // Copy all of the goods into our ROFF cache
            for i in 0..count {
                VectorCopy(&(*roff_data.offset(i as isize)).origin_delta, &mut (*mem.offset(i as isize)).origin_delta);
                VectorCopy(&(*roff_data.offset(i as isize)).rotate_delta, &mut (*mem.offset(i as isize)).rotate_delta);

                (*mem.offset(i as isize)).mStartNote = (*roff_data.offset(i as isize)).mStartNote;
                (*mem.offset(i as isize)).mNumNotes = (*roff_data.offset(i as isize)).mNumNotes;
            }

            if (*hdr).mNumNotes != 0
            {
                let mut size: usize = 0;
                let mut ptr: *mut c_char;
                let start: *mut c_char;

                ptr = (data as *mut c_void as usize + core::mem::size_of::<roff_hdr2_t>() + (count as usize) * core::mem::size_of::<move_rotate2_t>()) as *mut c_char;
                start = ptr;

                for _i in 0..(*hdr).mNumNotes {
                    size += strlen(ptr) + 1;
                    ptr = ptr.add(strlen(ptr) + 1);
                }

                // ? Get rid of dynamic memory ?
                roffs[num_roffs as usize].mNoteTrackIndexes = G_Alloc(((*hdr).mNumNotes as usize) * core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
                ptr = G_Alloc(size) as *mut c_char;
                *roffs[num_roffs as usize].mNoteTrackIndexes = ptr;
                memcpy(ptr as *mut c_void, start as *const c_void, size);

                for i in 1..(*hdr).mNumNotes {
                    ptr = ptr.add(strlen(ptr) + 1);
                    *roffs[num_roffs as usize].mNoteTrackIndexes.offset(i as isize) = ptr;
                }
            }
            return 1; // qtrue
        }
    }

    0 // qfalse
}

//-------------------------------------------------------
// G_LoadRoff
//
// Does the fun work of loading and caching a roff file
//	If the file is already cached, it just returns an
//	ID to the cached file.
//-------------------------------------------------------

pub unsafe fn G_LoadRoff(fileName: *const c_char) -> c_int
{
    let mut file: [c_char; 64] = [0; 64];
    let mut data: *mut u8 = core::ptr::null_mut();
    let mut len: c_int;
    let mut i: c_int;
    let mut roff_id: c_int = 0;

    // Before even bothering with all of this, make sure we have a place to store it.
    if num_roffs >= MAX_ROFFS
    {
        Com_Printf(
            b"^1MAX_ROFFS count exceeded.  Skipping load of .ROF '%s'\n\0".as_ptr() as *const c_char,
            fileName,
        );
        return roff_id;
    }

    // The actual path
    sprintf(
        file.as_mut_ptr(),
        b"%s/%s.rof\0".as_ptr() as *const c_char,
        Q3_SCRIPT_DIR.as_ptr() as *const c_char,
        fileName,
    );

    // See if I'm already precached
    i = 0;
    while i < num_roffs
    {
        if stricmp(file.as_ptr(), roffs[i as usize].fileName) == 0
        {
            // Good, just return me...avoid zero index
            return i + 1;
        }
        i += 1;
    }

    #[cfg(debug_assertions)]
    {
        //	Com_Printf( b"^2Caching ROF: '%s'\n\0".as_ptr() as *const c_char, file.as_ptr() );
    }

    // Read the file in one fell swoop
    let mut data_ptr: *mut c_void = core::ptr::null_mut();
    len = gi.FS_ReadFile(file.as_ptr(), &mut data_ptr);
    data = data_ptr as *mut u8;

    if len <= 0
    {
        Com_Printf(
            b"^1Could not open .ROF file '%s'\n\0".as_ptr() as *const c_char,
            fileName,
        );
        return roff_id;
    }

    // Now let's check the header info...
    let header = data as *mut roff_hdr2_t;

    // ..and make sure it's reasonably valid
    if G_ValidRoff(header) == 0
    {
        Com_Printf(
            b"^1Invalid roff format '%s'\n\0".as_ptr() as *const c_char,
            fileName,
        );
    }
    else
    {
        G_InitRoff(file.as_mut_ptr(), data);

        // Done loading this roff, so save off an id to it..increment first to avoid zero index
        num_roffs += 1;
        roff_id = num_roffs;
    }

    gi.FS_FreeFile(data as *mut c_void);

    roff_id
}


pub unsafe fn G_FreeRoffs()
{
    while num_roffs != 0 {
        G_FreeRoff(num_roffs - 1);
        num_roffs -= 1;
    }
}


//-------------------------------------------------------
// G_Roff
//
// Handles applying the roff data to the specified ent
//-------------------------------------------------------

pub unsafe fn G_Roff(ent: *mut gentity_t)
{
    if (*ent).next_roff_time == 0
    {
        return;
    }

    if (*ent).next_roff_time > level.time
    {// either I don't think or it's just not time to have me think yet
        return;
    }

    let roff_id: c_int = G_LoadRoff((*ent).roff);

    if roff_id == 0
    {	// Couldn't cache this rof
        return;
    }

    // The ID is one higher than the array index
    let roff: *const roff_list_t = &roffs[(roff_id - 1) as usize];
    let mut org: [f32; 3] = [0.0; 3];
    let mut ang: [f32; 3] = [0.0; 3];

    if (*roff).r#type == 2
    {
        let data: *mut move_rotate2_t = ((*roff).data as *mut move_rotate2_t).offset((*ent).roff_ctr as isize);
        VectorCopy(&(*data).origin_delta, &mut org);
        VectorCopy(&(*data).rotate_delta, &mut ang);
        if (*data).mStartNote != -1 || (*data).mNumNotes != 0
        {
            let notetrack_str = *roffs[(roff_id - 1) as usize].mNoteTrackIndexes.offset((*data).mStartNote as isize);
            G_RoffNotetrackCallback(ent, notetrack_str);
        }
    }
    else
    {
        let data: *mut move_rotate_t = ((*roff).data as *mut move_rotate_t).offset((*ent).roff_ctr as isize);
        VectorCopy(&(*data).origin_delta, &mut org);
        VectorCopy(&(*data).rotate_delta, &mut ang);
    }

    #[cfg(debug_assertions)]
    {
        if !g_developer.is_null()
        {
            // Note: g_developer->integer would require the full cvar_t struct definition
            // Stubbing this check for now
            // Com_Printf(
            //     b"^2ROFF dat: num: %d o:<%.2f %.2f %.2f> a:<%.2f %.2f %.2f>\n\0".as_ptr() as *const c_char,
            //     (*ent).roff_ctr,
            //     org[0],
            //     org[1],
            //     org[2],
            //     ang[0],
            //     ang[1],
            //     ang[2],
            // );
        }
    }

    if !(*ent).client.is_null()
    {
        // Set up the angle interpolation
        //-------------------------------------
        VectorAdd(&(*ent).s.apos.trBase, &ang, &mut (*ent).s.apos.trBase);
        (*ent).s.apos.trTime = level.time;
        (*ent).s.apos.trType = TR_INTERPOLATE;

        // Store what the next apos->trBase should be
        VectorCopy(&(*ent).s.apos.trBase, &mut (*(*ent).client).ps.viewangles);
        VectorCopy(&(*ent).s.apos.trBase, &mut (*ent).currentAngles);
        VectorCopy(&(*ent).s.apos.trBase, &mut (*ent).s.angles);
        if !(*ent).NPC.is_null()
        {
            //(*ent).NPC->desiredPitch = (*ent).s.apos.trBase[PITCH];
            (*(*ent).NPC).desiredYaw = (*ent).s.apos.trBase[YAW];
        }

        // Set up the origin interpolation
        //-------------------------------------
        VectorAdd(&(*ent).s.pos.trBase, &org, &mut (*ent).s.pos.trBase);
        (*ent).s.pos.trTime = level.time;
        (*ent).s.pos.trType = TR_INTERPOLATE;

        // Store what the next pos->trBase should be
        VectorCopy(&(*ent).s.pos.trBase, &mut (*(*ent).client).ps.origin);
        VectorCopy(&(*ent).s.pos.trBase, &mut (*ent).currentOrigin);
        //VectorCopy( (*ent).s.pos.trBase, (*ent).s.origin );
    }
    else
    {
        // Set up the angle interpolation
        //-------------------------------------
        VectorScale(&ang, (*roff).mLerp as f32, &mut (*ent).s.apos.trDelta);
        VectorCopy(&(*ent).pos2, &mut (*ent).s.apos.trBase);
        (*ent).s.apos.trTime = level.time;
        (*ent).s.apos.trType = TR_LINEAR;

        // Store what the next apos->trBase should be
        VectorAdd(&(*ent).pos2, &ang, &mut (*ent).pos2);

        // Set up the origin interpolation
        //-------------------------------------
        VectorScale(&org, (*roff).mLerp as f32, &mut (*ent).s.pos.trDelta);
        VectorCopy(&(*ent).pos1, &mut (*ent).s.pos.trBase);
        (*ent).s.pos.trTime = level.time;
        (*ent).s.pos.trType = TR_LINEAR;

        // Store what the next apos->trBase should be
        VectorAdd(&(*ent).pos1, &org, &mut (*ent).pos1);

        //make it true linear... FIXME: sticks around after ROFF is done, but do we really care?
        (*ent).alt_fire = 1; // qtrue

        if (*ent).e_ThinkFunc == thinkF_TieFighterThink || (*ent).e_ThinkFunc == thinkF_TieBomberThink ||
            ((*ent).e_ThinkFunc.is_null()
            && (*ent).s.eType != ET_MISSILE
            && (*ent).s.eType != ET_ITEM
            && (*ent).s.eType != ET_MOVER)
        {//will never set currentAngles & currentOrigin itself ( why do we limit which one's get set?, just set all the time? )
            EvaluateTrajectory(&(*ent).s.apos as *const c_void, level.time, &mut (*ent).currentAngles);
            EvaluateTrajectory(&(*ent).s.pos as *const c_void, level.time, &mut (*ent).currentOrigin);
        }
    }

    // Link just in case.
    gi.linkentity(ent);

    // See if the ROFF playback is done
    //-------------------------------------
    (*ent).roff_ctr += 1;
    if (*ent).roff_ctr >= (*roff).frames
    {
        // We are done, so let me think no more, then tell the task that we're done.
        (*ent).next_roff_time = 0;

        // Stop any rotation or movement.
        VectorClear(&mut (*ent).s.pos.trDelta);
        VectorClear(&mut (*ent).s.apos.trDelta);

        Q3_TaskIDComplete(ent, TID_MOVE_NAV);

        return;
    }

    (*ent).next_roff_time = level.time + (*roff).mFrameTime;
}


//-------------------------------------------------------
// G_SaveCachedRoffs
//
// Really fun savegame stuff
//-------------------------------------------------------

pub unsafe fn G_SaveCachedRoffs()
{
    let mut i: c_int;
    let mut len: c_int;

    // Write out the number of cached ROFFs
    gi.AppendToSaveGame(1380013904u32, &num_roffs as *const c_int as *mut c_void, core::mem::size_of::<c_int>()); // 'ROFF'

    // Now dump out the cached ROFF file names in order so they can be loaded on the other end
    i = 0;
    while i < num_roffs
    {
        // Dump out the string length to make things a bit easier on the other end...heh heh.
        len = (strlen(roffs[i as usize].fileName) + 1) as c_int;
        gi.AppendToSaveGame(1280066899u32, &len as *const c_int as *mut c_void, core::mem::size_of::<c_int>()); // 'SLEN'
        gi.AppendToSaveGame(1347638610u32, roffs[i as usize].fileName as *mut c_void, len as usize); // 'RSTR'
        i += 1;
    }
}


//-------------------------------------------------------
// G_LoadCachedRoffs
//
// Really fun loadgame stuff
//-------------------------------------------------------

pub unsafe fn G_LoadCachedRoffs()
{
    let mut i: c_int;
    let mut count: c_int = 0;
    let mut len: c_int;
    let mut buffer: [c_char; 64] = [0; 64];

    // Get the count of goodies we need to revive
    gi.ReadFromSaveGame(1380013904u32, &mut count as *mut c_int as *mut c_void, core::mem::size_of::<c_int>()); // 'ROFF'

    // Now bring 'em back to life
    i = 0;
    while i < count
    {
        gi.ReadFromSaveGame(1280066899u32, &mut len as *mut c_int as *mut c_void, core::mem::size_of::<c_int>()); // 'SLEN'
        gi.ReadFromSaveGame(1347638610u32, buffer.as_mut_ptr() as *mut c_void, len as usize); // 'RSTR'
        G_LoadRoff(buffer.as_ptr());
        i += 1;
    }
}
