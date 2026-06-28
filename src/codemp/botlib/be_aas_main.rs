/*****************************************************************************
 * name:		be_aas_main.c
 *
 * desc:		AAS
 *
 * $Archive: /MissionPack/code/botlib/be_aas_main.c $
 * $Author: Mrelusive $
 * $Revision: 8 $
 * $Modtime: 11/28/00 7:52a $
 * $Date: 11/28/00 7:52a $
 *
 *****************************************************************************/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// Stubs for types and constants not yet ported:
// These types are declared to allow compilation of this module.
// They should be properly imported from their respective modules when available.

//PORTING: Define a stub for aas_t structure
// This mirrors the layout-critical portion of the C aas_t from be_aas_def.h
#[repr(C)]
pub struct aas_t {
    //basic info
    pub loaded: c_int,                                    //true when an AAS file is loaded
    pub initialized: c_int,                               //true when AAS has been initialized
    pub savefile: c_int,                                  //set true when file should be saved
    pub bspchecksum: c_int,
    //current time
    pub time: f32,
    pub numframes: c_int,
    //name of the aas file
    pub filename: [c_char; 64],                           //MAX_PATH = 64
    pub mapname: [c_char; 64],                            //MAX_PATH = 64
    //bounding boxes
    pub numbboxes: c_int,
    pub bboxes: *mut c_void,
    //vertexes
    pub numvertexes: c_int,
    pub vertexes: *mut c_void,
    //planes
    pub numplanes: c_int,
    pub planes: *mut c_void,
    //edges
    pub numedges: c_int,
    pub edges: *mut c_void,
    //edge index
    pub edgeindexsize: c_int,
    pub edgeindex: *mut c_void,
    //faces
    pub numfaces: c_int,
    pub faces: *mut c_void,
    //face index
    pub faceindexsize: c_int,
    pub faceindex: *mut c_void,
    //convex areas
    pub numareas: c_int,
    pub areas: *mut c_void,
    //convex area settings
    pub numareasettings: c_int,
    pub areasettings: *mut c_void,
    //reachablity list
    pub reachabilitysize: c_int,
    pub reachability: *mut c_void,
    //nodes of the bsp tree
    pub numnodes: c_int,
    pub nodes: *mut c_void,
    //cluster portals
    pub numportals: c_int,
    pub portals: *mut c_void,
    //cluster portal index
    pub portalindexsize: c_int,
    pub portalindex: *mut c_void,
    //clusters
    pub numclusters: c_int,
    pub clusters: *mut c_void,
    //
    pub numreachabilityareas: c_int,
    pub reachabilitytime: f32,
    //enities linked in the areas
    pub linkheap: *mut c_void,                            //heap with link structures
    pub linkheapsize: c_int,                              //size of the link heap
    pub freelinks: *mut c_void,                           //first free link
    pub arealinkedentities: *mut *mut c_void,             //entities linked into areas
    //entities
    pub maxentities: c_int,
    pub maxclients: c_int,
    pub entities: *mut c_void,
    //string indexes
    pub configstrings: [*mut c_char; 512],                //MAX_CONFIGSTRINGS = 512
    pub indexessetup: c_int,
    //index to retrieve travel flag for a travel type
    pub travelflagfortype: [c_int; 32],                   //MAX_TRAVELTYPES = 32
    //travel flags for each area based on contents
    pub areacontentstravelflags: *mut c_int,
    //routing update
    pub areaupdate: *mut c_void,
    pub portalupdate: *mut c_void,
    //number of routing updates during a frame (reset every frame)
    pub frameroutingupdates: c_int,
    //reversed reachability links
    pub reversedreachability: *mut c_void,
    //travel times within the areas
    pub areatraveltimes: *mut *mut *mut c_void,
    //array of size numclusters with cluster cache
    pub clusterareacache: *mut *mut *mut c_void,
    pub portalcache: *mut *mut c_void,
    //cache list sorted on time
    pub oldestcache: *mut c_void,                         // start of cache list sorted on time
    pub newestcache: *mut c_void,                         // end of cache list sorted on time
    //maximum travel time through portal areas
    pub portalmaxtraveltimes: *mut c_int,
    //areas the reachabilities go through
    pub reachabilityareaindex: *mut c_int,
    pub reachabilityareas: *mut c_void,
}

//PORTING: Define a stub for libvar_t structure
#[repr(C)]
pub struct libvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub flags: c_int,
    pub modified: c_int,                                  //qboolean
    pub value: f32,
    pub next: *mut libvar_t,
}

pub static mut aasworld: aas_t = aas_t {
    loaded: 0,
    initialized: 0,
    savefile: 0,
    bspchecksum: 0,
    time: 0.0,
    numframes: 0,
    filename: [0; 64],
    mapname: [0; 64],
    numbboxes: 0,
    bboxes: ptr::null_mut(),
    numvertexes: 0,
    vertexes: ptr::null_mut(),
    numplanes: 0,
    planes: ptr::null_mut(),
    numedges: 0,
    edges: ptr::null_mut(),
    edgeindexsize: 0,
    edgeindex: ptr::null_mut(),
    numfaces: 0,
    faces: ptr::null_mut(),
    faceindexsize: 0,
    faceindex: ptr::null_mut(),
    numareas: 0,
    areas: ptr::null_mut(),
    numareasettings: 0,
    areasettings: ptr::null_mut(),
    reachabilitysize: 0,
    reachability: ptr::null_mut(),
    numnodes: 0,
    nodes: ptr::null_mut(),
    numportals: 0,
    portals: ptr::null_mut(),
    portalindexsize: 0,
    portalindex: ptr::null_mut(),
    numclusters: 0,
    clusters: ptr::null_mut(),
    numreachabilityareas: 0,
    reachabilitytime: 0.0,
    linkheap: ptr::null_mut(),
    linkheapsize: 0,
    freelinks: ptr::null_mut(),
    arealinkedentities: ptr::null_mut(),
    maxentities: 0,
    maxclients: 0,
    entities: ptr::null_mut(),
    configstrings: [ptr::null_mut(); 512],
    indexessetup: 0,
    travelflagfortype: [0; 32],
    areacontentstravelflags: ptr::null_mut(),
    areaupdate: ptr::null_mut(),
    portalupdate: ptr::null_mut(),
    frameroutingupdates: 0,
    reversedreachability: ptr::null_mut(),
    areatraveltimes: ptr::null_mut(),
    clusterareacache: ptr::null_mut(),
    portalcache: ptr::null_mut(),
    oldestcache: ptr::null_mut(),
    newestcache: ptr::null_mut(),
    portalmaxtraveltimes: ptr::null_mut(),
    reachabilityareaindex: ptr::null_mut(),
    reachabilityareas: ptr::null_mut(),
};

pub static mut saveroutingcache: *mut libvar_t = ptr::null_mut();

extern "C" {
    //PORTING: External function declarations from various botlib modules
    pub fn GetMemory(size: core::ffi::c_ulong) -> *mut c_void;
    pub fn GetClearedHunkMemory(size: core::ffi::c_ulong) -> *mut c_void;
    pub fn FreeMemory(ptr: *mut c_void);
    pub fn PrintUsedMemorySize();
    pub fn PrintMemoryLabels();

    pub fn LibVarGetValue(var_name: *const c_char) -> f32;
    pub fn LibVarValue(var_name: *const c_char, value: *const c_char) -> f32;
    pub fn LibVar(var_name: *const c_char, value: *const c_char) -> *mut libvar_t;
    pub fn LibVarSet(var_name: *const c_char, value: *const c_char);

    pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, ...);
    pub fn Com_Memset(dest: *mut c_void, val: c_int, count: core::ffi::c_ulong);

    pub fn AAS_ContinueInitReachability(time: f32) -> c_int;
    pub fn AAS_InitClustering();
    pub fn AAS_Optimize();
    pub fn AAS_WriteAASFile(filename: *const c_char) -> c_int;
    pub fn AAS_InitRouting();
    pub fn AAS_UnlinkInvalidEntities();
    pub fn AAS_InvalidateEntities();
    pub fn AAS_RoutingInfo();
    pub fn AAS_WriteRouteCache();
    pub fn AAS_ResetEntityLinks();
    pub fn AAS_LoadBSPFile();
    pub fn AAS_LoadAASFile(filename: *const c_char) -> c_int;
    pub fn AAS_FreeRoutingCaches();
    pub fn AAS_InitSettings();
    pub fn AAS_InitAASLinkHeap();
    pub fn AAS_InitAASLinkedEntities();
    pub fn AAS_InitReachability();
    pub fn AAS_InitAlternativeRouting();
    pub fn AAS_ShutdownAlternativeRouting();
    pub fn AAS_DumpBSPData();
    pub fn AAS_FreeAASLinkHeap();
    pub fn AAS_FreeAASLinkedEntities();
    pub fn AAS_DumpAASData();

    //PORTING: botimport interface
    pub static mut botimport: botlib_import_t;

    pub static mut bot_developer: c_int;
}

//PORTING: Stub for botlib_import_t interface
#[repr(C)]
pub struct botlib_import_t {
    pub Print: unsafe extern "C" fn(c_int, *const c_char, ...),
}

const MAX_PATH: usize = 64;
const MAX_MODELS: c_int = 512;
const CS_MODELS: c_int = 64;  // CS_SCORES(32) + MAX_CLIENTS(32)
const MAX_CONFIGSTRINGS: usize = 512;
const PRT_FATAL: c_int = 0;
const PRT_ERROR: c_int = 1;
const PRT_MESSAGE: c_int = 2;
const BLERR_NOERROR: c_int = 0;
const QTRUE: c_int = 1;
const QFALSE: c_int = 0;

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
//PORTING: Variadic function implementation.
// In C, this uses vsprintf with va_list to format variable arguments.
// Rust's extern "C" supports variadic signatures but direct va_list access
// from Rust is limited. This stub accepts the variadic signature and
// calls the underlying C print function. Full varargs support requires
// C implementation or macro-level support.
pub unsafe extern "C" fn AAS_Error(fmt: *mut c_char, _args: ...) {
    let str: [c_char; 1024] = [0; 1024];

    //PORTING DEVIATION: vsprintf requires proper va_list handling which is
    // complex in Rust. For compilation, this calls the print function directly.
    // The actual formatting would require C-level varargs support or
    // implementing this function in C.
    (botimport.Print)(PRT_FATAL, fmt);
} //end of the function AAS_Error

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_StringFromIndex(
    indexname: *mut c_char,
    stringindex: *mut *mut c_char,
    numindexes: c_int,
    index: c_int,
) -> *mut c_char {
    if aasworld.indexessetup == 0 {
        (botimport.Print)(
            PRT_ERROR,
            "%s: index %d not setup\n\0".as_ptr() as *const c_char,
            indexname,
            index,
        );
        return "".as_ptr() as *mut c_char;
    } //end if
    if index < 0 || index >= numindexes {
        (botimport.Print)(
            PRT_ERROR,
            "%s: index %d out of range\n\0".as_ptr() as *const c_char,
            indexname,
            index,
        );
        return "".as_ptr() as *mut c_char;
    } //end if
    if (*stringindex.add(index as usize)).is_null() {
        if index != 0 {
            (botimport.Print)(
                PRT_ERROR,
                "%s: reference to unused index %d\n\0".as_ptr() as *const c_char,
                indexname,
                index,
            );
        } //end if
        return "".as_ptr() as *mut c_char;
    } //end if
    *stringindex.add(index as usize)
} //end of the function AAS_StringFromIndex

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_IndexFromString(
    indexname: *mut c_char,
    stringindex: *mut *mut c_char,
    numindexes: c_int,
    string: *mut c_char,
) -> c_int {
    let mut i: c_int;
    if aasworld.indexessetup == 0 {
        (botimport.Print)(
            PRT_ERROR,
            "%s: index not setup \"%s\"\n\0".as_ptr() as *const c_char,
            indexname,
            string,
        );
        return 0;
    } //end if
    i = 0;
    while i < numindexes {
        if !(*stringindex.add(i as usize)).is_null() {
            if Q_stricmp(*stringindex.add(i as usize), string) == 0 {
                return i;
            }
        }
        i += 1;
    } //end for
    0
} //end of the function AAS_IndexFromString

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ModelFromIndex(index: c_int) -> *mut c_char {
    AAS_StringFromIndex(
        "ModelFromIndex\0".as_ptr() as *mut c_char,
        aasworld.configstrings[CS_MODELS as usize..].as_mut_ptr(),
        MAX_MODELS,
        index,
    )
} //end of the function AAS_ModelFromIndex

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_IndexFromModel(modelname: *mut c_char) -> c_int {
    AAS_IndexFromString(
        "IndexFromModel\0".as_ptr() as *mut c_char,
        aasworld.configstrings[CS_MODELS as usize..].as_mut_ptr(),
        MAX_MODELS,
        modelname,
    )
} //end of the function AAS_IndexFromModel

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_UpdateStringIndexes(
    numconfigstrings: c_int,
    configstrings: *mut *mut c_char,
) {
    let mut i: c_int;
    //set string pointers and copy the strings
    i = 0;
    while i < numconfigstrings {
        if !(*configstrings.add(i as usize)).is_null() {
            //if (aasworld.configstrings[i]) FreeMemory(aasworld.configstrings[i]);
            aasworld.configstrings[i as usize] = GetMemory(
                (libc::strlen(*configstrings.add(i as usize)) + 1) as core::ffi::c_ulong,
            ) as *mut c_char;
            libc::strcpy(
                aasworld.configstrings[i as usize],
                *configstrings.add(i as usize),
            );
        } //end if
        i += 1;
    } //end for
    aasworld.indexessetup = QTRUE;
} //end of the function AAS_UpdateStringIndexes

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_Loaded() -> c_int {
    aasworld.loaded
} //end of the function AAS_Loaded

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_Initialized() -> c_int {
    aasworld.initialized
} //end of the function AAS_Initialized

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_SetInitialized() {
    aasworld.initialized = QTRUE;
    (botimport.Print)(PRT_MESSAGE, "AAS initialized.\n\0".as_ptr() as *const c_char);
    #[cfg(debug_assertions)]
    {
        //create all the routing cache
        //AAS_CreateAllRoutingCache();
        //
        //AAS_RoutingInfo();
    }
} //end of the function AAS_SetInitialized

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ContinueInit(time: f32) {
    //if no AAS file loaded
    if aasworld.loaded == 0 {
        return;
    }
    //if AAS is already initialized
    if aasworld.initialized != 0 {
        return;
    }
    //calculate reachability, if not finished return
    if AAS_ContinueInitReachability(time) != 0 {
        return;
    }
    //initialize clustering for the new map
    AAS_InitClustering();
    //if reachability has been calculated and an AAS file should be written
    //or there is a forced data optimization
    if aasworld.savefile != 0 || (LibVarGetValue("forcewrite\0".as_ptr() as *const c_char) as c_int) != 0
    {
        //optimize the AAS data
        if (LibVarValue(
            "aasoptimize\0".as_ptr() as *const c_char,
            "0\0".as_ptr() as *const c_char,
        ) as c_int) != 0
        {
            AAS_Optimize();
        }
        //save the AAS file
        if AAS_WriteAASFile(aasworld.filename.as_ptr()) != 0 {
            (botimport.Print)(
                PRT_MESSAGE,
                "%s written succesfully\n\0".as_ptr() as *const c_char,
                aasworld.filename.as_ptr(),
            );
        } //end if
        else {
            (botimport.Print)(
                PRT_ERROR,
                "couldn't write %s\n\0".as_ptr() as *const c_char,
                aasworld.filename.as_ptr(),
            );
        } //end else
    } //end if
    //initialize the routing
    AAS_InitRouting();
    //at this point AAS is initialized
    AAS_SetInitialized();
} //end of the function AAS_ContinueInit

//===========================================================================
// called at the start of every frame
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_StartFrame(time: f32) -> c_int {
    aasworld.time = time;
    //unlink all entities that were not updated last frame
    AAS_UnlinkInvalidEntities();
    //invalidate the entities
    AAS_InvalidateEntities();
    //initialize AAS
    AAS_ContinueInit(time);
    //
    aasworld.frameroutingupdates = 0;
    //
    if bot_developer != 0 {
        if LibVarGetValue("showcacheupdates\0".as_ptr() as *const c_char) != 0.0 {
            AAS_RoutingInfo();
            LibVarSet(
                "showcacheupdates\0".as_ptr() as *const c_char,
                "0\0".as_ptr() as *const c_char,
            );
        } //end if
        if LibVarGetValue("showmemoryusage\0".as_ptr() as *const c_char) != 0.0 {
            PrintUsedMemorySize();
            LibVarSet(
                "showmemoryusage\0".as_ptr() as *const c_char,
                "0\0".as_ptr() as *const c_char,
            );
        } //end if
        if LibVarGetValue("memorydump\0".as_ptr() as *const c_char) != 0.0 {
            PrintMemoryLabels();
            LibVarSet(
                "memorydump\0".as_ptr() as *const c_char,
                "0\0".as_ptr() as *const c_char,
            );
        } //end if
    } //end if
    //
    if (*saveroutingcache).value != 0.0 {
        AAS_WriteRouteCache();
        LibVarSet(
            "saveroutingcache\0".as_ptr() as *const c_char,
            "0\0".as_ptr() as *const c_char,
        );
    } //end if
    //
    aasworld.numframes += 1;
    BLERR_NOERROR
} //end of the function AAS_StartFrame

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_Time() -> f32 {
    aasworld.time
} //end of the function AAS_Time

//===========================================================================
//
// Parameter:			-
// Returns:				-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_ProjectPointOntoVector(
    point: *mut f32,
    vStart: *mut f32,
    vEnd: *mut f32,
    vProj: *mut f32,
) {
    let mut pVec: [f32; 3] = [0.0; 3];
    let mut vec: [f32; 3] = [0.0; 3];

    VectorSubtract(point, vStart, pVec.as_mut_ptr());
    VectorSubtract(vEnd, vStart, vec.as_mut_ptr());
    VectorNormalize(vec.as_mut_ptr());
    // project onto the directional vector for this segment
    VectorMA(
        vStart,
        DotProduct(pVec.as_ptr(), vec.as_ptr()),
        vec.as_ptr(),
        vProj,
    );
} //end of the function AAS_ProjectPointOntoVector

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_LoadFiles(mapname: *const c_char) -> c_int {
    let mut errnum: c_int;
    let mut aasfile: [c_char; MAX_PATH] = [0; MAX_PATH];

    libc::strcpy(aasworld.mapname.as_mut_ptr(), mapname as *mut c_char);
    //NOTE: first reset the entity links into the AAS areas and BSP leaves
    // the AAS link heap and BSP link heap are reset after respectively the
    // AAS file and BSP file are loaded
    AAS_ResetEntityLinks();
    // load bsp info
    AAS_LoadBSPFile();

    //load the aas file
    Com_sprintf(
        aasfile.as_mut_ptr(),
        MAX_PATH as c_int,
        "maps/%s.aas\0".as_ptr() as *const c_char,
        mapname,
    );
    errnum = AAS_LoadAASFile(aasfile.as_ptr());
    if errnum != BLERR_NOERROR {
        return errnum;
    }

    (botimport.Print)(
        PRT_MESSAGE,
        "loaded %s\n\0".as_ptr() as *const c_char,
        aasfile.as_ptr(),
    );
    libc::strncpy(
        aasworld.filename.as_mut_ptr(),
        aasfile.as_ptr(),
        MAX_PATH,
    );
    BLERR_NOERROR
} //end of the function AAS_LoadFiles

//===========================================================================
// called everytime a map changes
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_LoadMap(mapname: *const c_char) -> c_int {
    let mut errnum: c_int;

    //if no mapname is provided then the string indexes are updated
    if mapname.is_null() {
        return 0;
    } //end if
    //
    aasworld.initialized = QFALSE;
    //NOTE: free the routing caches before loading a new map because
    // to free the caches the old number of areas, number of clusters
    // and number of areas in a clusters must be available
    AAS_FreeRoutingCaches();
    //load the map
    errnum = AAS_LoadFiles(mapname);
    if errnum != BLERR_NOERROR {
        aasworld.loaded = QFALSE;
        return errnum;
    } //end if
    //
    AAS_InitSettings();
    //initialize the AAS link heap for the new map
    AAS_InitAASLinkHeap();
    //initialize the AAS linked entities for the new map
    AAS_InitAASLinkedEntities();
    //initialize reachability for the new map
    AAS_InitReachability();
    //initialize the alternative routing
    AAS_InitAlternativeRouting();
    //everything went ok
    0
} //end of the function AAS_LoadMap

//===========================================================================
// called when the library is first loaded
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_Setup() -> c_int {
    aasworld.maxclients =
        LibVarValue("maxclients\0".as_ptr() as *const c_char, "128\0".as_ptr() as *const c_char)
            as c_int;
    aasworld.maxentities =
        LibVarValue("maxentities\0".as_ptr() as *const c_char, "1024\0".as_ptr() as *const c_char)
            as c_int;
    // as soon as it's set to 1 the routing cache will be saved
    saveroutingcache = LibVar("saveroutingcache\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char);
    //allocate memory for the entities
    if !aasworld.entities.is_null() {
        FreeMemory(aasworld.entities);
    }
    aasworld.entities = GetClearedHunkMemory(
        (aasworld.maxentities as core::ffi::c_ulong)
            * (core::mem::size_of::<aas_entity_t>() as core::ffi::c_ulong),
    );
    //invalidate all the entities
    AAS_InvalidateEntities();
    //force some recalculations
    //LibVarSet("forceclustering", "1");			//force clustering calculation
    //LibVarSet("forcereachability", "1");		//force reachability calculation
    aasworld.numframes = 0;
    BLERR_NOERROR
} //end of the function AAS_Setup

//===========================================================================
//
// Parameter:				-
// Returns:					-
// Changes Globals:		-
//===========================================================================
pub unsafe extern "C" fn AAS_Shutdown() {
    AAS_ShutdownAlternativeRouting();
    //
    AAS_DumpBSPData();
    //free routing caches
    AAS_FreeRoutingCaches();
    //free aas link heap
    AAS_FreeAASLinkHeap();
    //free aas linked entities
    AAS_FreeAASLinkedEntities();
    //free the aas data
    AAS_DumpAASData();
    //free the entities
    if !aasworld.entities.is_null() {
        FreeMemory(aasworld.entities);
    }
    //clear the aasworld structure
    Com_Memset(
        core::ptr::addr_of_mut!(aasworld) as *mut c_void,
        0,
        core::mem::size_of::<aas_t>() as core::ffi::c_ulong,
    );
    //aas has not been initialized
    aasworld.initialized = QFALSE;
    //NOTE: as soon as a new .bsp file is loaded the .bsp file memory is
    // freed an reallocated, so there's no need to free that memory here
    //print shutdown
    //	botimport.Print(PRT_MESSAGE, "AAS shutdown.\n");
} //end of the function AAS_Shutdown

//PORTING: Stub for aas_entity_t structure
#[repr(C)]
pub struct aas_entity_t {
    pub i: c_int,                     //placeholder for aas_entityinfo_t
    pub areas: *mut c_void,           //aas_link_t *
    pub leaves: *mut c_void,          //bsp_link_t *
}

//PORTING: External vector math functions
extern "C" {
    pub fn VectorSubtract(a: *const f32, b: *const f32, out: *mut f32);
    pub fn VectorNormalize(v: *mut f32) -> f32;
    pub fn DotProduct(a: *const f32, b: *const f32) -> f32;
    pub fn VectorMA(a: *const f32, scale: f32, b: *const f32, out: *mut f32);
}

//PORTING: Stub for Q_stricmp
extern "C" {
    pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}
