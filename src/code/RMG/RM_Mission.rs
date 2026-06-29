/************************************************************************************************
 *
 * RM_Mission.rs
 *
 * implements the CRMMission class.  The CRMMission class loads and manages an arioche mission
 *
 ************************************************************************************************/

use core::ffi::{c_char, c_int, c_void};

const ARIOCHE_CLIPBRUSH_SIZE: c_int = 300;
const CVAR_OBJECTIVE: c_int = 0;

// Stub types for dependencies not yet ported
type CRandomTerrain = c_void;
type CGPGroup = c_void;
type CRMNode = c_void;
type CRMPathManager = c_void;
type CRMAreaManager = c_void;
type CRMArea = c_void;
type CRMInstance = c_void;
type CRMObjective = c_void;
type CRMInstanceFile = c_void;
type CGenericParser2 = c_void;
type vec3_t = [f32; 3];
type vec3pair_t = [vec3_t; 2];
type vec4_t = [f32; 4];
type qboolean = c_int;

// Stub constants
const NULL: *mut c_void = 0 as *mut c_void;
const DIR_MAX: c_int = 4;
const MAX_RANDOM_CHOICES: usize = 256;
const MAX_QPATH: usize = 64;
const SYMMETRY_NONE: c_int = 0;
const SYMMETRY_TOPLEFT: c_int = 1;
const SYMMETRY_BOTTOMRIGHT: c_int = 2;
const SIDE_RED: c_int = 1;
const SIDE_BLUE: c_int = 2;
const AT_FLAT: c_int = 1;
const AT_NONE: c_int = 0;
const DEG2RAD: fn(f32) -> f32 = |x| x * 3.14159265359 / 180.0;

// Stubs for external functions
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn atof(s: *const c_char) -> f32;
    fn atoi(s: *const c_char) -> c_int;
    fn atol(s: *const c_char) -> c_int;
    fn va(fmt: *const c_char, ...) -> *const c_char;
    fn Cvar_Set(name: *const c_char, value: *const c_char);
    fn Cvar_VariableIntegerValue(name: *const c_char) -> c_int;
    fn Cvar_VariableStringBuffer(name: *const c_char, buf: *mut c_char, size: usize);
    fn Com_Printf(fmt: *const c_char, ...);
    fn Com_ParseTextFile(name: *const c_char, parser: *mut CGenericParser2) -> bool;
    fn Com_Error(level: c_int, fmt: *const c_char, ...);
    fn VectorCopy(src: *const f32, dst: *mut f32);
    fn VectorClear(v: *mut f32);
    fn VectorSet(v: *mut f32, x: f32, y: f32, z: f32);
    fn Distance(a: *const f32, b: *const f32) -> f32;
}

#[allow(non_snake_case)]
pub struct CRMMission {
    mCurrentObjective: *mut CRMObjective,
    mValidPaths: bool,
    mValidRivers: bool,
    mValidNodes: bool,
    mValidWeapons: bool,
    mValidAmmo: bool,
    mValidObjectives: bool,
    mValidInstances: bool,
    mTimeLimit: c_int,
    mMaxInstancePosition: c_int,
    mAccuracyMultiplier: f32,
    mHealthMultiplier: f32,
    mPickupHealth: f32,
    mPickupArmor: f32,
    mPickupAmmo: f32,
    mPickupWeapon: f32,
    mPickupEquipment: f32,
    mDefaultPadding: c_int,
    mSymmetric: c_int,
    mLandScape: *mut CRandomTerrain,
    mAreaManager: *mut CRMAreaManager,
    mPathManager: *mut CRMPathManager,
    mObjectives: Vec<*mut CRMObjective>,
    mInstances: Vec<*mut CRMInstance>,
    mInstanceFile: CRMInstanceFile,
    mDescription: String,
    mExitScreen: String,
    mTimeExpiredScreen: String,
    mBackUpPath: c_int,
}

/************************************************************************************************
 * CRMMission::CRMMission
 *	constructor
 *
 * inputs:
 *  none
 *
 * return:
 *	none
 *
 ************************************************************************************************/
impl CRMMission {
    pub fn new(landscape: *mut CRandomTerrain) -> Self {
        let mut mission = CRMMission {
            mCurrentObjective: NULL as *mut CRMObjective,
            mValidPaths: false,
            mValidRivers: false,
            mValidNodes: false,
            mValidWeapons: false,
            mValidAmmo: false,
            mValidObjectives: false,
            mValidInstances: false,
            mTimeLimit: 0,
            mMaxInstancePosition: 1,
            mAccuracyMultiplier: 1.0f32,
            mHealthMultiplier: 1.0f32,
            mPickupHealth: 1.0f32,
            mPickupArmor: 1.0f32,
            mPickupAmmo: 1.0f32,
            mPickupWeapon: 1.0f32,
            mPickupEquipment: 1.0f32,
            mDefaultPadding: 0,
            mSymmetric: SYMMETRY_NONE,
            mLandScape: landscape,
            mAreaManager: NULL as *mut CRMAreaManager,
            mPathManager: NULL as *mut CRMPathManager,
            mObjectives: Vec::new(),
            mInstances: Vec::new(),
            mInstanceFile: unsafe { std::mem::zeroed() },
            mDescription: String::new(),
            mExitScreen: String::new(),
            mTimeExpiredScreen: String::new(),
            mBackUpPath: 0,
        };

        //	mCheckedEnts.clear();

        // cut down the possible area that is 'legal' for area manager to use by 20%
        let mut land_min: vec3_t = [0.0f32; 3];
        let mut land_max: vec3_t = [0.0f32; 3];

        unsafe {
            // Note: GetBounds returns a pointer to bounds array, simulated access here
            // This is a stub - actual implementation would call the landscape methods
            land_min[0] = 0.0f32; // Placeholder
            land_min[1] = 0.0f32;
            land_min[2] = 0.0f32;

            land_max[0] = 1.0f32;
            land_max[1] = 1.0f32;
            land_max[2] = 1.0f32;

            // Create a new area manager for the landscape
            // mAreaManager = new CRMAreaManager ( land_min, land_max );

            // Create a new path manager
            // mPathManager = new CRMPathManager ( mLandScape );
        }

        mission
    }
}

/************************************************************************************************
 * CRMMission::~CRMMission
 *	destructor
 *
 * inputs:
 *  none
 *
 * return:
 *	none
 *
 ************************************************************************************************/
impl Drop for CRMMission {
    fn drop(&mut self) {
        //	mCheckedEnts.clear();

        // Cleanup the objectives
        for objective in self.mObjectives.drain(..) {
            unsafe {
                let _ = Box::from_raw(objective);
            }
        }

        // Cleanup the instances
        for instance in self.mInstances.drain(..) {
            unsafe {
                let _ = Box::from_raw(instance);
            }
        }

        if !self.mPathManager.is_null() {
            unsafe {
                let _ = Box::from_raw(self.mPathManager);
                self.mPathManager = NULL as *mut CRMPathManager;
            }
        }

        if !self.mAreaManager.is_null() {
            unsafe {
                let _ = Box::from_raw(self.mAreaManager);
                self.mAreaManager = NULL as *mut CRMAreaManager;
            }
        }
    }
}

impl CRMMission {
    /************************************************************************************************
     * CRMMission::FindObjective
     *	searches through the missions objectives for the one with the given name
     *
     * inputs:
     *  name: name of objective to find
     *
     * return:
     *	objective: objective matching the given name or NULL if it couldnt be found
     *
     ************************************************************************************************/
    pub fn FindObjective(&self, name: *const c_char) -> *mut CRMObjective {
        for it in self.mObjectives.iter() {
            // Does it match?
            unsafe {
                if stricmp((*it as *mut CRMObjective as *const CRMObjective as *const c_void as *const c_char), name) == 0 {
                    return *it;
                }
            }
        }

        // Not found
        NULL as *mut CRMObjective
    }

    fn MirrorPos(&self, pos: &mut vec3_t) {
        pos[0] = 1.0f32 - pos[0];
        pos[1] = 1.0f32 - pos[1];
    }

    /************************************************************************************************
     * CRMMission::ParseOrigin
     *	parses an origin block which includes linking to a node and absolute origins
     *
     * inputs:
     *  group: parser group containing the node or origin
     *
     * return:
     *	true: parsed successfully
     *  false: failed to parse
     *
     ************************************************************************************************/
    fn ParseOrigin(
        &self,
        originGroup: *const CGPGroup,
        origin: &mut vec3_t,
        lookat: &mut vec3_t,
        flattenHeight: *mut c_int,
    ) -> bool {
        let mut szNodeName: *const c_char;
        let mut mins: vec3_t = [0.0f32; 3];
        let mut maxs: vec3_t = [0.0f32; 3];

        if !flattenHeight.is_null() {
            unsafe {
                *flattenHeight = 66;
            }
        }

        // If no group was given then use 0,0,0
        if originGroup.is_null() {
            unsafe {
                VectorCopy(&[0.0f32, 0.0f32, 0.0f32][0], &mut origin[0]);
            }
            return false;
        }

        // See if attaching to a named node
        unsafe {
            // szNodeName = originGroup->FindPairValue ( "node", "" );
            szNodeName = "".as_ptr() as *const c_char; // Stub
            if *szNodeName != 0 {
                let node: *mut CRMNode = NULL as *mut CRMNode;
                // Find the node being attached to
                // node = mPathManager->FindNodeByName ( szNodeName );
                if !node.is_null() {
                    if !flattenHeight.is_null() {
                        // if ( node->GetFlattenHeight ( ) == -1 )
                        // {
                        //     node->SetFlattenHeight ( 40 + mLandScape->irand(0,40) );
                        // }

                        // *flattenHeight = node->GetFlattenHeight ( );
                    }

                    // VectorCopy(node->GetPos(), origin);
                    // VectorCopy ( origin, lookat );

                    let mut dir: c_int;
                    let mut rnd_offset: c_int = 0; // mLandScape->irand(0, DIR_MAX-1);
                    for dir_val in 0..DIR_MAX {
                        dir = dir_val;
                        let d = ((dir + rnd_offset) % DIR_MAX) as usize;
                        // if (node->PathExist(d))
                        // {
                        //     vec4_t tmp_pt, tmp_dir;
                        //     int pathID = node->GetPath(d);
                        //     mLandScape->GetPathInfo(pathID, 0.1f, tmp_pt, tmp_dir );
                        //     lookat[0] = tmp_pt[0];
                        //     lookat[1] = tmp_pt[1];
                        //     lookat[2] = 0;
                        //     return true;
                        // }
                    }
                    return true;
                }
            }

            // mins[0] = atof( originGroup->FindPairValue ( "left", ".1" ) );
            // mins[1] = atof( originGroup->FindPairValue ( "top", ".1" ) );
            // maxs[0] = atof( originGroup->FindPairValue ( "right", ".9" ) );
            // maxs[1] = atof( originGroup->FindPairValue ( "bottom", ".9" ) );

            mins[0] = 0.1f32;
            mins[1] = 0.1f32;
            maxs[0] = 0.9f32;
            maxs[1] = 0.9f32;

            // lookat[0] = origin[0] = mLandScape->flrand(mins[0],maxs[0]);
            // lookat[1] = origin[1] = mLandScape->flrand(mins[1],maxs[1]);
            lookat[0] = 0.5f32;
            origin[0] = 0.5f32;
            lookat[1] = 0.5f32;
            origin[1] = 0.5f32;
            lookat[2] = 0.0f32;
            origin[2] = 0.0f32;
        }

        true
    }

    /************************************************************************************************
     * CRMMission::ParseNodes
     *	parses all the named nodes in the file
     *
     * inputs:
     *  group: parser group containing the named nodes
     *
     * return:
     *	true:  parsed successfully
     *  false: failed to parse
     *
     ************************************************************************************************/
    fn ParseNodes(&mut self, group: *const CGPGroup) -> bool {
        // If NULL that means this particular difficulty level has no named nodes
        if group.is_null() || self.mValidNodes {
            return true;
        }

        // how many nodes spaced over map?
        let mut x_cells: c_int;
        let mut y_cells: c_int;

        unsafe {
            x_cells = atoi("3\0".as_ptr() as *const c_char); // Stub: originGroup->FindPairValue ( "x_cells", "3" )
            y_cells = atoi("3\0".as_ptr() as *const c_char); // Stub
        }

        // mPathManager->CreateArray(x_cells, y_cells);

        // Loop through all the nodes and generate each as specified
        // for ( group = group->GetSubGroups();
        //       group;
        //       group=group->GetNext() )
        // {
        //     int min_depth = atof( group->FindPairValue ( "min_depth", "0" ) );
        //     int max_depth = atof( group->FindPairValue ( "max_depth", "5" ) );
        //     int min_paths = atoi( group->FindPairValue ( "min_paths", "1" ) );
        //     int max_paths = atoi( group->FindPairValue ( "max_paths", "1" ) );
        //
        //     mPathManager->CreateLocation( group->GetName(), min_depth, max_depth, min_paths, max_paths );
        // }

        self.mValidNodes = true;
        true
    }

    /************************************************************************************************
     * CRMMission::ParsePaths
     *	parses all path styles in the file and then generates paths
     *
     * inputs:
     *  group: parser group containing the list of path styles
     *
     * return:
     *	true:  parsed successfully
     *  false: failed to parse
     *
     ************************************************************************************************/
    fn ParsePaths(&mut self, group: *const CGPGroup) -> bool {
        // If NULL that means this particular difficulty level has no paths
        if group.is_null() || self.mValidPaths {
            return true;
        }

        // path style info
        let mut depth: f32;
        let mut deviation: f32;
        let mut breadth: f32;
        let mut minwidth: f32;
        let mut maxwidth: f32;
        let mut points: c_int;

        unsafe {
            points = atoi("10\0".as_ptr() as *const c_char);
            depth = atof(".31\0".as_ptr() as *const c_char);
            deviation = atof(".025\0".as_ptr() as *const c_char);
            breadth = atof("5\0".as_ptr() as *const c_char);
            minwidth = atof(".03\0".as_ptr() as *const c_char);
            maxwidth = atof(".05\0".as_ptr() as *const c_char);
        }

        // mPathManager->SetPathStyle( points, minwidth, maxwidth, depth, deviation, breadth);

        if !self.mValidPaths {
            // we must create paths
            // mPathManager->GeneratePaths( mSymmetric );
            self.mValidPaths = true;
        }

        true
    }

    /************************************************************************************************
     * CRMMission::ParseRivers
     *	parses all river styles in the file and then generates rivers
     *
     * inputs:
     *  group: parser group containing the list of path styles
     *
     * return:
     *	true:  parsed successfully
     *  false: failed to parse
     *
     ************************************************************************************************/
    fn ParseRivers(&mut self, group: *const CGPGroup) -> bool {
        // If NULL that means this particular difficulty level has no rivers
        if group.is_null() || self.mValidRivers {
            return true;
        }

        // river style info
        let mut maxdepth: c_int;
        let mut beddepth: f32;
        let mut deviation: f32;
        let mut breadth: f32;
        let mut minwidth: f32;
        let mut maxwidth: f32;
        let mut points: c_int;
        let bridge_name: String;

        unsafe {
            maxdepth = atoi("5\0".as_ptr() as *const c_char);
            points = atoi("10\0".as_ptr() as *const c_char);
            beddepth = atof("1\0".as_ptr() as *const c_char);
            deviation = atof(".03\0".as_ptr() as *const c_char);
            breadth = atof("7\0".as_ptr() as *const c_char);
            minwidth = atof(".01\0".as_ptr() as *const c_char);
            maxwidth = atof(".03\0".as_ptr() as *const c_char);
            bridge_name = String::new(); // Stub: group->FindPairValue ( "bridge", "" )
        }

        // mPathManager->SetRiverStyle( maxdepth, points, minwidth, maxwidth, beddepth, deviation, breadth, bridge_name);

        if !self.mValidRivers && beddepth < 1.0f32 {
            // we must create rivers
            // mPathManager->GenerateRivers();
            self.mValidRivers = true;
        }

        true
    }

    fn PlaceBridges(&mut self) {
        if !self.mValidRivers {
            // || strlen(mPathManager->GetBridgeName()) < 1
            return;
        }

        let mut max_bridges: c_int = 0;
        let mut path: c_int;
        let mut t: f32;
        // let river_depth = mLandScape->GetLandScape()->GetWaterHeight();
        let mut pos: vec3_t = [0.0f32; 3];
        let mut lastpos: vec3_t = [0.0f32; 3];
        let mut bounds: vec3pair_t = [[0.0f32; 3]; 2];

        unsafe {
            VectorSet(&mut bounds[0], 0.0f32, 0.0f32, 0.0f32);
            VectorSet(&mut bounds[1], 0.0f32, 0.0f32, 0.0f32);
        }

        // walk along paths looking for dips
        // for (path = 0; path < mPathManager->GetPathCount(); path++)
        // {
        //     vec4_t tmp_pt, tmp_dir;
        //     bool	new_water = true;
        //
        //     mLandScape->GetPathInfo(path, 0, tmp_pt, tmp_dir );
        //     lastpos[0] = mLandScape->GetBounds ( )[0][0] + (mLandScape->GetBounds ( )[1][0]-mLandScape->GetBounds ( )[0][0]) * tmp_pt[0];
        //     lastpos[1] = mLandScape->GetBounds ( )[0][1] + (mLandScape->GetBounds ( )[1][1]-mLandScape->GetBounds ( )[0][1]) * tmp_pt[1];
        //     lastpos[2] = mLandScape->GetBounds ( )[0][2] + (mLandScape->GetBounds ( )[1][2]-mLandScape->GetBounds ( )[0][2]) * tmp_pt[2];
        //     mLandScape->GetLandScape()->GetWorldHeight ( lastpos, bounds, true );
        //
        //     const float delta = 0.05f;
        //     for (t= delta; t < 1.0f; t += delta)
        //     {
        //         mLandScape->GetPathInfo(path, t, tmp_pt, tmp_dir );
        //         pos[0] = mLandScape->GetBounds ( )[0][0] + (mLandScape->GetBounds ( )[1][0]-mLandScape->GetBounds ( )[0][0]) * tmp_pt[0];
        //         pos[1] = mLandScape->GetBounds ( )[0][1] + (mLandScape->GetBounds ( )[1][1]-mLandScape->GetBounds ( )[0][1]) * tmp_pt[1];
        //         pos[2] = mLandScape->GetBounds ( )[0][2] + (mLandScape->GetBounds ( )[1][2]-mLandScape->GetBounds ( )[0][2]) * tmp_pt[2];
        //         mLandScape->GetLandScape()->GetWorldHeight ( pos, bounds, true );
        //
        //         if (new_water &&
        //             lastpos[2] < river_depth &&
        //             pos[2] < river_depth &&
        //             pos[2] > lastpos[2])
        //         {	// add a bridge
        //             if (max_bridges < 3)
        //             {
        //                 CRMArea*		area;
        //                 CRMInstance*	instance;
        //
        //                 max_bridges++;
        //
        //                 // create a single bridge
        //                 lastpos[2] = mLandScape->GetBounds ( )[0][2] + (mLandScape->GetBounds ( )[1][2]-mLandScape->GetBounds ( )[0][2]) * mPathManager->GetPathDepth();
        //                 instance = mInstanceFile.CreateInstance ( mPathManager->GetBridgeName() );
        //
        //                 if ( NULL != instance )
        //                 {	// Set the area
        //                     vec3_t zerodvec;
        //                     VectorClear(zerodvec);
        //                     area = mAreaManager->CreateArea ( lastpos, instance->GetSpacingRadius(), instance->GetSpacingLine(), GetDefaultPadding(), 0, zerodvec, pos, instance->GetFlattenRadius()?true:false, false, instance->GetLockOrigin() );
        //                     area->EnableLookAt(false);
        //
        //                     instance->SetArea ( mAreaManager, area );
        //                     mInstances.push_back ( instance );
        //                     new_water = false;
        //                 }
        //             }
        //         }
        //         else if (pos[2] > river_depth)
        //         {	// hit land again
        //             new_water = true;
        //         }
        //         VectorCopy ( pos, lastpos );
        //     }
        // }
    }

    fn PlaceWallInstance(
        &mut self,
        instance: *mut CRMInstance,
        xpos: f32,
        ypos: f32,
        zpos: f32,
        x: c_int,
        y: c_int,
        angle: f32,
    ) {
        if instance.is_null() {
            return;
        }

        // let spacing = instance->GetSpacingRadius();
        let spacing: f32 = 1.0f32; // Stub
        let mut area: *mut CRMArea = NULL as *mut CRMArea;
        let mut origin: vec3_t = [0.0f32; 3];
        let mut zerodvec: vec3_t = [0.0f32; 3];

        unsafe {
            VectorClear(&mut zerodvec);
        }

        origin[0] = xpos + spacing * x as f32;
        origin[1] = ypos + spacing * y as f32;
        origin[2] = zpos;

        // Set the area of position
        // area = mAreaManager->CreateArea ( origin, (spacing / 2.1f), 0, GetDefaultPadding(), 0, zerodvec, origin, instance->GetFlattenRadius()?true:false, false, instance->GetLockOrigin() );
        // area->EnableLookAt(false);
        // area->SetAngle(angle);
        // instance->SetArea ( mAreaManager, area );

        self.mInstances.push_back(instance);
    }

    /************************************************************************************************
     * CRMMission::ParseWallRect
     *	creates instances for walled rectangle at this node (fence)
     *
     * inputs:
     *  group: parser group containing the wall rect info
     *
     * return:
     *	true:  parsed successfully
     *  false: failed to parse
     *
     ************************************************************************************************/
    #[cfg(not(feature = "pre_release_demo"))]
    fn ParseWallRect(&mut self, group: *const CGPGroup, side: c_int) -> bool {
        if group.is_null() {
            return true;
        }

        // let wallGroup = group->FindSubGroup ( "wallrect" );

        // If NULL that means this particular instance has no wall rect
        // if ( NULL == group || NULL == wallGroup)
        // {
        //     return true;
        // }

        // Stub implementation - full logic elided due to preprocessor complexity
        true
    }

    #[cfg(feature = "pre_release_demo")]
    fn ParseWallRect(&mut self, group: *const CGPGroup, side: c_int) -> bool {
        true
    }

    /************************************************************************************************
     * CRMMission::ParseInstancesOnPath
     *	creates instances on path between nodes
     *
     * inputs:
     *  group: parser group containing the defenses, other instances on the path between nodes
     *
     * return:
     *	true:  parsed successfully
     *  false: failed to parse
     *
     ************************************************************************************************/
    #[cfg(not(feature = "pre_release_demo"))]
    fn ParseInstancesOnPath(&mut self, group: *const CGPGroup) -> bool {
        if group.is_null() {
            return true;
        }

        // Stub implementation - full logic elided due to preprocessor complexity
        true
    }

    #[cfg(feature = "pre_release_demo")]
    fn ParseInstancesOnPath(&mut self, group: *const CGPGroup) -> bool {
        true
    }

    /************************************************************************************************
     * CRMMission::ParseInstance
     *	Parses an individual instance
     *
     * inputs:
     *  group: parser group containing the list of instances
     *
     * return:
     *	true: instances parsed successfully
     *  false: instances failed to parse
     *
     ************************************************************************************************/
    fn ParseInstance(&mut self, group: *const CGPGroup) -> bool {
        let mut area: *mut CRMArea = NULL as *mut CRMArea;
        let mut instance: *mut CRMInstance = NULL as *mut CRMInstance;
        let mut spacing: f32;
        let mut origin: vec3_t = [0.0f32; 3];
        let mut lookat: vec3_t = [0.0f32; 3];
        let mut flattenHeight: c_int;
        let mut zerodvec: vec3_t = [0.0f32; 3];

        unsafe {
            VectorClear(&mut zerodvec);
        }

        // create fences / walls

        // Create the instance using the instance file helper class
        // instance = mInstanceFile.CreateInstance ( group->GetName ( ) );

        // Failed to create, not good
        if instance.is_null() {
            return false;
        }

        // If a spacing radius was specified then override the one thats
        // in the instance
        unsafe {
            spacing = atof("0\0".as_ptr() as *const c_char); // Stub
            if spacing != 0.0f32 {
                // instance->SetSpacingRadius ( spacing );
            }

            // instance->SetFilter(group->FindPairValue("filter", ""));
            // instance->SetTeamFilter(group->FindPairValue("teamfilter", ""));

            // if (strstr(instance->GetTeamFilter(),"red"))
            //     instance->SetSide( SIDE_RED);
            // else if (strstr(instance->GetTeamFilter(),"blue"))
            //     instance->SetSide( SIDE_BLUE );

            //	ParseWallRect(group, instance->GetSide());

            // Get its origin now
            // ParseOrigin ( group->FindSubGroup ( "origin" ), origin, lookat, &flattenHeight );
            // origin[0] = mLandScape->GetBounds ( )[0][0] + (mLandScape->GetBounds ( )[1][0]-mLandScape->GetBounds ( )[0][0]) * origin[0];
            // origin[1] = mLandScape->GetBounds ( )[0][1] + (mLandScape->GetBounds ( )[1][1]-mLandScape->GetBounds ( )[0][1]) * origin[1];
            // origin[2] = mLandScape->GetBounds ( )[0][2] + (mLandScape->GetBounds ( )[1][2]-mLandScape->GetBounds ( )[0][2]) * origin[2];

            // lookat[0] = mLandScape->GetBounds ( )[0][0] + (mLandScape->GetBounds ( )[1][0]-mLandScape->GetBounds ( )[0][0]) * lookat[0];
            // lookat[1] = mLandScape->GetBounds ( )[0][1] + (mLandScape->GetBounds ( )[1][1]-mLandScape->GetBounds ( )[0][1]) * lookat[1];
            // lookat[2] = mLandScape->GetBounds ( )[0][2] + (mLandScape->GetBounds ( )[1][2]-mLandScape->GetBounds ( )[0][2]) * lookat[2];

            // Fixed height?  (used for bridges)
            // if ( !atoi(group->FindPairValue ( "nodrop", "0" )) )
            // {
            //     origin[2] = mLandScape->GetBounds ( )[1][2] + 100;
            // }

            // Set the area of position
            // area = mAreaManager->CreateArea ( origin, instance->GetSpacingRadius(), instance->GetSpacingLine(), GetDefaultPadding(), 0, zerodvec, lookat, instance->GetFlattenRadius()?true:false, true, instance->GetLockOrigin(), mSymmetric );
            // instance->SetArea ( mAreaManager, area );
            // instance->SetFlattenHeight ( flattenHeight );

            self.mInstances.push_back(instance);

            // create defenses?
            // ParseInstancesOnPath(group );
        }

        true
    }

    /************************************************************************************************
     * CRMMission::ParseInstances
     *	parses all instances within the mission and populates the instance list
     *
     * inputs:
     *  group: parser group containing the list of instances
     *
     * return:
     *	true: instances parsed successfully
     *  false: instances failed to parse
     *
     ************************************************************************************************/
    #[cfg(not(feature = "pre_release_demo"))]
    fn ParseInstances(&mut self, group: *const CGPGroup) -> bool {
        // If NULL that means this particular difficulty level has no instances
        if group.is_null() {
            return true;
        }

        // Loop through all the instances in the mission and add each
        // to the master list of instances
        // for ( group = group->GetSubGroups();
        //       group;
        //       group=group->GetNext() )
        // {
        //     ParseInstance ( group );
        // }

        true
    }

    #[cfg(feature = "pre_release_demo")]
    fn ParseInstances(&mut self, group: *const CGPGroup) -> bool {
        true
    }

    /************************************************************************************************
     * CRMMission::ParseObjectives
     *	parses all objectives within the mission and populates the objective list
     *
     * inputs:
     *  group: parser group containing the list of objectives
     *
     * return:
     *	true: objectives parsed successfully
     *  false: objectives failed to parse
     *
     ************************************************************************************************/
    fn ParseObjectives(&mut self, group: *const CGPGroup) -> bool {
        // If NULL that means this particular difficulty level has no objectives
        if group.is_null() {
            return true;
        }

        // Loop through all the objectives in the mission and add each
        // to the master list of objectives
        // for ( group = group->GetSubGroups();
        //       group;
        //       group=group->GetNext() )
        // {
        //     CRMObjective* objective;
        //
        //     // Create the new objective
        //     objective = new CRMObjective ( group );
        //
        //     mObjectives.push_back ( objective );
        // }

        self.mValidObjectives = true;

        true
    }

    /************************************************************************************************
     * CRMMission::ParseAmmo
     *	parses the given ammo list and sets the necessary ammo cvars to grant those
     *  weapons to the players
     *
     * inputs:
     *  ammos: parser group containing the ammo list
     *
     * return:
     *	true: ammo parsed successfully
     *  false: ammo failed to parse
     *
     ************************************************************************************************/
    fn ParseAmmo(&mut self, ammos: *const CGPGroup) -> bool {
        /*	CGPValue* ammo;

            // No weapons, no success
            if ( NULL == ammos )
            {
                return false;
            }

            if (0 == gi.Cvar_VariableIntegerValue("ar_wpnselect"))
            {
                // Make sure the ammo cvars are all reset so ammo from the last map or
                // another difficulty level wont carry over
                CWeaponSystem::ClearAmmoCvars (TheWpnSysHelper());

                ammo = ammos->GetPairs ( );

                // Loop through the weapons listed and grant them to the player
                while ( ammo )
                {
                    // Grab the weapons ID
                    AmmoID id = CWeaponSystem::GetAmmoID ( ammo->GetName ( ) );

                    // Now set the weapon cvar with the given data
                    TheWpnSysHelper().CvarSet ( CWeaponSystem::GetAmmoCvar ( id ), ammo->GetTopValue ( ), CVAR_AMMO );

                    // Move on to the next weapon
                    ammo = (CGPValue*)ammo->GetNext();
                }
            }
        */
        self.mValidAmmo = true;

        true
    }

    /************************************************************************************************
     * CRMMission::ParseWeapons
     *	parses the given weapon list and sets the necessary weapon cvars to grant those
     *  weapons to the players
     *
     * inputs:
     *  weapons: parser group containing the weapons list
     *
     * return:
     *	true: weapons parsed successfully
     *  false: weapons failed to parse
     *
     ************************************************************************************************/
    fn ParseWeapons(&mut self, weapons: *const CGPGroup) -> bool {
        /*	CGPValue*	weapon;
            WpnID		id;

            // No weapons, no success
            if ( NULL == weapons )
            {
                return false;
            }

            if (0 == gi.Cvar_VariableIntegerValue("ar_wpnselect"))
            {
                // Make sure the weapon cvars are all reset so weapons from the last map or
                // another difficulty level wont carry over
                CWeaponSystem::ClearWpnCvars (TheWpnSysHelper());

                id     = NULL_WpnID;
                weapon = weapons->GetPairs ( );

                // Loop through the weapons listed and grant them to the player
                while ( weapon )
                {
                    // Grab the weapons ID
                    id = CWeaponSystem::GetWpnID ( weapon->GetName ( ) );

                    // Now set the weapon cvar with the given data
                    TheWpnSysHelper().CvarSet ( CWeaponSystem::GetWpnCvar ( id ), weapon->GetTopValue ( ) );

                    // Move on to the next weapon
                    weapon = (CGPValue*)weapon->GetNext();
                }

                // If we found at least one weapon then ready the last one in the list
                if ( NULL_WpnID != id )
                {
                    TheWpnSysHelper().CvarSet("wp_righthand", va("%i/%i/0/0",id,CWeaponSystem::GetClipSize ( id )), CVAR_MISC );
                }
            }
        */
        self.mValidWeapons = true;

        true
    }

    /************************************************************************************************
     * CRMMission::ParseOutfit
     *	parses the outfit (weapons and ammo)
     *
     * inputs:
     *  outfit: parser group containing the outfit
     *
     * return:
     *	true: weapons and ammo parsed successfully
     *  false: failed to parse
     *
     ************************************************************************************************/
    fn ParseOutfit(&mut self, outfit: *const CGPGroup) -> bool {
        if outfit.is_null() {
            return false;
        }

        /*	// Its ok to fail parsing weapons as long as weapons have
            // already been parsed at some point
            if ( !ParseWeapons ( ParseRandom ( outfit->FindSubGroup ( "weapons" ) ) ) )
            {
                if ( !mValidWeapons )
                {
                    return false;
                }
            }

            // Its ok to fail parsing ammo as long as ammo have
            // already been parsed at some point
            if ( !ParseAmmo ( ParseRandom ( outfit->FindSubGroup ( "ammo" ) ) ) )
            {
                if ( !mValidAmmo)
                {
                    return false;
                }
            }
        */
        true
    }

    /************************************************************************************************
     * CRMMission::ParseRandom
     *	selects a random sub group with from all within this one
     *
     * inputs:
     *  random: parser group containing the various subgroups
     *
     * return:
     *	true:  parsed successfuly
     *  false: failed to parse
     *
     ************************************************************************************************/
    fn ParseRandom(&self, randomGroup: *const CGPGroup) -> *const CGPGroup {
        if randomGroup.is_null() {
            return NULL as *const CGPGroup;
        }

        let mut groups: [*const CGPGroup; MAX_RANDOM_CHOICES] = [NULL as *const CGPGroup; MAX_RANDOM_CHOICES];
        let mut numGroups: usize = 0;

        // Build a list of the groups one can be chosen
        // for ( numGroups = 0, group = randomGroup->GetSubGroups ( );
        //       group;
        //       group = group->GetNext ( ) )
        // {
        //     if ( stricmp ( group->GetName ( ), "random_choice" ) )
        //     {
        //         continue;
        //     }
        //
        //     int weight = atoi ( group->FindPairValue ( "random_weight", "1" ) );
        //     while (weight-- > 0)
        //         groups[numGroups++] = group;
        //     assert (numGroups <= MAX_RANDOM_CHOICES);
        // }

        // No groups!
        if numGroups == 0 {
            return randomGroup;
        }

        // Now choose a group to parse
        // return groups[mLandScape->irand(0,numGroups-1)];
        randomGroup
    }

    /************************************************************************************************
     * CRMMission::ParseDifficulty
     *	parses the given difficulty and populates the mission with its data
     *
     * inputs:
     *  difficulty: parser group containing the difficulties info
     *
     * return:
     *	true: difficulty parsed successfully
     *  false: difficulty failed to parse
     *
     ************************************************************************************************/
    fn ParseDifficulty(&mut self, difficulty: *const CGPGroup, mut parent: *const CGPGroup) -> bool {
        // If a null difficulty then stop the recursion.  Make sure to
        // return true here so the parsing doesnt fail
        if difficulty.is_null() {
            return true;
        }

        unsafe {
            // if (difficulty->GetParent())
            // {
            //     parent = difficulty->GetParent();
            // }

            // is map supposed to be symmetric?
            // mSymmetric = (symmetry_t)atoi(parent->FindPairValue ( "symmetric", "0" ));
            // mBackUpPath = atoi(parent->FindPairValue ( "backuppath", "0" ));
            self.mSymmetric = SYMMETRY_NONE;
            self.mBackUpPath = 0;

            if self.mSymmetric != 0 {
                // pick between the 2 starting corners -- yes this is a hack
                self.mSymmetric = SYMMETRY_TOPLEFT;
                // if( TheRandomMissionManager->GetLandScape()->irand(0, 1) )
                // {
                //     mSymmetric = SYMMETRY_BOTTOMRIGHT;
                // }
            }

            // mDefaultPadding = atoi(parent->FindPairValue ( "padding", "0" ));
            self.mDefaultPadding = 0;

            // Parse the nodes
            if !self.ParseNodes(self.ParseRandom(NULL as *const CGPGroup)) {
                return false;
            }

            // Parse the paths
            if !self.ParsePaths(self.ParseRandom(NULL as *const CGPGroup)) {
                return false;
            }

            // Parse the rivers
            if !self.ParseRivers(self.ParseRandom(NULL as *const CGPGroup)) {
                return false;
            }

            // Handle inherited properties
            // if ( !ParseDifficulty ( parent->FindSubGroup ( difficulty->FindPairValue ( "inherit", "" ) ), parent ) )
            // {
            //     return false;
            // }

            // parse the player's outfit (weapons and ammo)
            if !self.ParseOutfit(self.ParseRandom(NULL as *const CGPGroup)) {
                // Its ok to fail parsing weapons as long as weapons have
                // already been parsed at some point
                if !self.ParseWeapons(self.ParseRandom(NULL as *const CGPGroup)) {
                    if !self.mValidWeapons {
                        return false;
                    }
                }

                // Its ok to fail parsing ammo as long as ammo have
                // already been parsed at some point
                if !self.ParseAmmo(self.ParseRandom(NULL as *const CGPGroup)) {
                    if !self.mValidAmmo {
                        return false;
                    }
                }
            }

            // Its ok to fail parsing objectives as long as objectives have
            // already been parsed at some point
            if !self.ParseObjectives(self.ParseRandom(NULL as *const CGPGroup)) {
                if !self.mValidObjectives {
                    return false;
                }
            }

            // Set the cvars with the available values
            Cvar_Set("mi_health\0".as_ptr() as *const c_char, "100\0".as_ptr() as *const c_char);
            Cvar_Set("mi_armor\0".as_ptr() as *const c_char, "0\0".as_ptr() as *const c_char);

            // Parse out the timelimit
            // mTimeLimit = atol(difficulty->FindPairValue("timelimit", "0"));
            self.mTimeLimit = 0;

            // NPC multipliers
            // mAccuracyMultiplier = atof(difficulty->FindPairValue("npcaccuracy", "1"));
            // mHealthMultiplier = atof(difficulty->FindPairValue("npchealth", "1"));
            self.mAccuracyMultiplier = 1.0f32;
            self.mHealthMultiplier = 1.0f32;

            // keep only some of RMG pickups 1 = 100%
            // mPickupHealth = atof(difficulty->FindPairValue("pickup_health", "1"));
            // mPickupArmor = atof(difficulty->FindPairValue("pickup_armor", "1"));
            // mPickupAmmo = atof(difficulty->FindPairValue("pickup_ammo", "1"));
            // mPickupWeapon = atof(difficulty->FindPairValue("pickup_weapon", "1"));
            // mPickupEquipment = atof(difficulty->FindPairValue("pickup_equipment", "1"));
            self.mPickupHealth = 1.0f32;
            self.mPickupArmor = 1.0f32;
            self.mPickupAmmo = 1.0f32;
            self.mPickupWeapon = 1.0f32;
            self.mPickupEquipment = 1.0f32;

            // Its ok to fail parsing instances as long as instances have
            // already been parsed at some point
            if !self.ParseInstances(self.ParseRandom(NULL as *const CGPGroup)) {
                if !self.mValidInstances {
                    return false;
                }
            }
        }

        true
    }

    /************************************************************************************************
     * CRMMission::Load
     *	Loads the given mission using the given difficulty level
     *
     * inputs:
     *  name: Name of the mission to load (should only be the name rather than the full path)
     *  difficulty: difficulty level to load
     *
     * return:
     *	true: mission successfully loaded
     *  false: mission failed to load
     *
     ************************************************************************************************/
    pub fn Load(&mut self, mission: *const c_char, instances: *const c_char, difficulty: *const c_char) -> bool {
        unsafe {
            let mut parser: CGenericParser2 = std::mem::zeroed();
            let mut root: *const CGPGroup = NULL as *const CGPGroup;

            // Create the parser for the mission file
            // if(!Com_ParseTextFile(va("ext_data/rmg/%s.mission", mission), parser))
            // {
            //     if(!Com_ParseTextFile(va("ext_data/arioche/%s.mission", mission), parser))
            //     {
            //         Com_Printf("ERROR: Failed to open mission file '%s'\n", mission);
            //         return false;
            //     }
            // }

            // Grab the root parser groop and make sure its mission, otherwise this
            // isnt a valid mission file
            // root = parser.GetBaseParseGroup()->GetSubGroups();
            // if(stricmp(root->GetName(), "mission"))
            // {
            //     Com_Printf("ERROR: '%s' is not a valid mission file\n", mission );
            //     parser.Clean();
            //     return false;
            // }

            // Grab the mission description and set the cvar for it
            // mDescription = root->FindPairValue ( "description", "<MISSION DESCRIPTION MISSING>" );
            //	Cvar_Set("ar_obj_main0",mDescription.c_str(), CVAR_OBJECTIVE);
            //	Cvar_Set("ar_obj_maincom0", "&OBJECTIVES_INPROGRESS&", CVAR_OBJECTIVE);
            //	Cvar_SetValue ("ar_cur_objective", 0, CVAR_OBJECTIVE);

            // string mInfo = root->FindPairValue ( "info", "<MISSION ADDITIONAL INFO MISSING>" );
            //	Cvar_Set("ar_obj_info0",mInfo.c_str(), CVAR_OBJECTIVE);

            // mExitScreen = root->FindPairValue ( "exitScreen", "<EXIT SCREEN MISSING>" );
            // mTimeExpiredScreen = root->FindPairValue ( "TimeExpiredScreen", "<TIME EXPIRED SCREEN MISSING>" );

            // Open the instance file for the specified instances
            // if ( !mInstanceFile.Open ( instances) )
            // {
            //     Com_Printf ( "ERROR: Could not open instance file '%s'\n", instances );
            //     return false;
            // }

            // Start at one and readjust each time we see and instance
            // with a higher value
            self.mMaxInstancePosition = 1;

            // Now parse the specified difficulty level
            // CGPGroup* parserdif = root->FindSubGroup ( difficulty );
            // CGPGroup* parserpar = parserdif->GetParent();
            // if (!parserpar)
            // { //rww - expected to have a parent, but sometime doesn't get set.
            //   //I take it JK2's generic parser is not quite the same as SOF2's. Or is out of date.
            //     parserpar = root;
            // }
            // if ( !ParseDifficulty ( parserdif, parserpar ) )
            // {
            //     return false;
            // }

            // Generate the terrain now
            // mLandScape->Generate(mSymmetric);

            // Cleanup
            // parser.Clean();

            true
        }
    }

    /************************************************************************************************
     * CRMMission::Spawn
     *	Spawns all of the instances for the entire mission onto the given landscape
     *
     * inputs:
     *  landscape: landscape to spawn instances on
     *
     * return:
     *	true: instances spawned successfully
     *  false: instances failed to spawn
     *
     ************************************************************************************************/
    #[cfg(not(feature = "pre_release_demo"))]
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        unsafe {
            let mut areaIndex: c_int;
            let mut area: *mut CRMArea;

            // Prespawn all instances, this is mainly for flattening
            for instance in self.mInstances.iter() {
                // Pre-Spawn
                // (*instance)->PreSpawn ( terrain, IsServer );

                if self.mSymmetric != 0 {
                    // (*instance)->SetMirror(1);
                    // (*instance)->PreSpawn ( terrain, IsServer );
                    // (*instance)->SetMirror(0);
                }
            }

            // mLandScape->Smooth ( );

            // place bridges
            // PlaceBridges();

            // create automap
            //	if (!com_dedicated->integer)
            {
                #[cfg(not(feature = "dedicated"))]
                {
                    // CM_TM_Create(mLandScape->GetLandScape());
                }
            }

            // mLandScape->GetLandScape()->UpdatePatches();

            // Spawn all instances
            for instance in self.mInstances.iter() {
                // Spawn
                // (*instance)->Spawn ( terrain, IsServer );
                // (*instance)->PostSpawn ( terrain, IsServer );

                if self.mSymmetric != 0 {
                    // spawn the mirror version
                    // (*instance)->SetMirror(1);
                    // (*instance)->Spawn ( terrain, IsServer );
                    // (*instance)->PostSpawn ( terrain, IsServer );
                    // (*instance)->SetMirror(0);
                }
            }

            #[cfg(not(feature = "final_build"))]
            {
                // make sure to write out after the mirror happens so red side is displayed on map
                if 1 == Cvar_VariableIntegerValue("rmg_saveautomap\0".as_ptr() as *const c_char) {
                    // write out automap for test purposes
                    let mut seed: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                    let mut terrainName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                    let mut missionName: [c_char; MAX_QPATH] = [0; MAX_QPATH];
                    Cvar_VariableStringBuffer("RMG_seed\0".as_ptr() as *const c_char, seed.as_mut_ptr(), MAX_QPATH);
                    Cvar_VariableStringBuffer("RMG_terrain\0".as_ptr() as *const c_char, terrainName.as_mut_ptr(), MAX_QPATH);
                    Cvar_VariableStringBuffer("RMG_mission\0".as_ptr() as *const c_char, missionName.as_mut_ptr(), MAX_QPATH);

                    #[cfg(not(feature = "dedicated"))]
                    {
                        for instance in self.mInstances.iter() {
                            // (*instance)->DrawAutomapSymbol();
                        }
                        //gi.CM_TM_SaveImageToDisk(terrainName, missionName, seed);
                        // CM_TM_SaveImageToDisk(terrainName, missionName, seed);
                    }
                    Com_Error(ERR_DROP, "RMG Automap written.\0".as_ptr() as *const c_char);
                    return false;
                }
            }

            //	// draw player start on automap
            //	CEntity	*spot = NULL;
            //	spot = entitySystem->GetEntityFromClassname( spot, "info_player_start");
            //	if (spot)
            //	{
            //		gi.CM_TM_AddStart(spot->GetOrigin()[0], spot->GetOrigin()[1]);
            //	}

            // Spawn NPC triggers now
            //	SpawnNPCTriggers ( mLandScape );

            // Restory all the NPC's accuracies to the template accuracies times the
            // multiplier
            //	INPCEnt::RestoreTemplate ( mAccuracyMultiplier, mHealthMultiplier );

            // Little trick to set the current objective to the first in the list
            self.CompleteObjective(NULL as *mut CRMObjective);

            // Iterate through the areas and add each to the landscapes list, this is sorta hacky
            // but bridges the game / common gap
            // for ( areaIndex = 0; NULL != (area = mAreaManager->EnumArea ( areaIndex )); areaIndex ++ )
            // {
            //     // Dont bother adding it to the list if collision isnt enabled
            //     if ( !area->IsCollisionEnabled() )
            //     {
            //         continue;
            //     }
            //
            //     CArea* newarea = new CArea ( );
            //     newarea->Init ( area->GetOrigin(), area->GetSpacingRadius (), 0, area->IsFlattened()?AT_FLAT:AT_NONE );
            //     mLandScape->GetLandScape()->SaveArea( newarea );
            //
            //     if (mSymmetric)
            //     {
            //         CArea* newarea = new CArea ( );
            //         newarea->Init ( area->GetOrigin(), area->GetSpacingRadius (), 0, area->IsFlattened()?AT_FLAT:AT_NONE );
            //         newarea->GetPosition()[0] = mLandScape->GetBounds ( )[0][0]+mLandScape->GetBounds ( )[1][0]-newarea->GetPosition()[0];
            //         newarea->GetPosition()[1] = mLandScape->GetBounds ( )[0][1]+mLandScape->GetBounds ( )[1][1]-newarea->GetPosition()[1];
            //         mLandScape->GetLandScape()->SaveArea( newarea );
            //     }
            // }

            // mInstanceFile.Close ( );

            true
        }
    }

    #[cfg(feature = "pre_release_demo")]
    pub fn Spawn(&mut self, terrain: *mut CRandomTerrain, IsServer: qboolean) -> bool {
        true
    }

    /************************************************************************************************
     * CRMMission::CompleteMission
     *	Pauses the game, plays an end screen after a brief delay, which then returns the player to
     * the RMG menu.
     *                                                                             *
     * Input                                                                                        *
     *    <Variable>: <Description>                                                                 *
     * Output / Return                                                                              *
     *    <Variable>: <Description>                                                                 *
     ************************************************************************************************/
    pub fn CompleteMission(&self) {
        unsafe {
            Cvar_Set("cl_paused\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char);
        }

        //	AddText(va("killserver; menu %s\n", mExitScreen.c_str()));
    }

    /************************************************************************************************
     * CRMMission::FailedMission
     *	Pauses the game, plays an end screen after a brief delay, which then returns the player to
     * the RMG menu.
     *                                                                             *
     * Input                                                                                        *
     *    TimeExpired: indicates if the reason failed was because of time
     * Output / Return                                                                              *
     *    <Variable>: <Description>                                                                 *
     ************************************************************************************************/
    pub fn FailedMission(&self, TimeExpired: bool) {
        unsafe {
            Cvar_Set("cl_paused\0".as_ptr() as *const c_char, "1\0".as_ptr() as *const c_char);
        }

        if TimeExpired {
            //		AddText(va("killserver; menu %s\n", mTimeExpiredScreen.c_str()));
        }
    }

    /************************************************************************************************
     * CRMMission::CompleteObjective
     *	Completes the given objective and advances the current objective accordingly
     *
     * inputs:
     *  objective: the objetive to mark complete
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn CompleteObjective(&mut self, objective: *mut CRMObjective) {
        // Set the object as completed
        if !objective.is_null() {
            unsafe {
                // (*objective)->Complete ( true );

                // Set the completed text for the objective
                //		gi.Cvar_Set( va("ar_obj_subcom0_%i", objective->GetOrderIndex ( )), "&OBJECTIVES_COMPLETE&", CVAR_OBJECTIVE) ;

                /*		CEntity *tent = G_TempEntity( vec3_origin, EV_SUB_PRINT );
                        tent->s.time2 = gi.SP_GetStringID ( objective->GetMessage ( ) );
                        tent->r.svFlags |= SVF_BROADCAST;
                        G_AddTempEntity(tent);

                        if (objective->CompleteSoundID())
                            G_SoundBroadcast( tent, objective->CompleteSoundID());
                */
            }
        }

        self.mCurrentObjective = NULL as *mut CRMObjective;

        // Find the next objective
        for it in self.mObjectives.iter() {
            let objective_tmp = *it;

            // Skip completed objectives
            // if ( objective->IsCompleted ( ) )
            // {
            //     continue;
            // }

            // Find the objective with the lowest priority
            // if ( mCurrentObjective && objective->GetPriority ( ) > mCurrentObjective->GetPriority ( ) )
            // {
            //     continue;
            // }

            // Found one
            self.mCurrentObjective = objective_tmp;
        }

        if !self.mCurrentObjective.is_null() {
            //		Cvar_SetValue ("ar_cur_objective", mCurrentObjective->GetOrderIndex ( ), CVAR_OBJECTIVE);

            unsafe {
                // (*mCurrentObjective)->Activate ( );
            }
        } else {
            // Set the completed text for the objective
            //		Cvar_Set( "ar_obj_maincom0", "&OBJECTIVES_COMPLETE&", CVAR_OBJECTIVE) ;
        }
    }

    /************************************************************************************************
     * CRMMission::Preview
     *	Previews the instances within the mission
     *
     * inputs:
     *  from: the origin which the mission is being previewed from
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub fn Preview(&self, from: &vec3_t) {
        // Look for settlements close to the player and put up some debug stuff
        for it in self.mInstances.iter() {
            let instance = *it;

            let mut a: vec3_t = [0.0f32; 3];
            let mut b: vec3_t = [0.0f32; 3];

            unsafe {
                VectorCopy(&from[0], &mut a[0]);
                // VectorCopy ( instance->GetOrigin(), &mut b[0] );

                a[2] = 0.0f32;
                b[2] = 0.0f32;

                // Skip stuff thats too far away
                // if ( Distance ( &a[0], &b[0]) > 2000.0f32 )
                // {
                //     continue;
                // }

                // (*instance)->Preview ( from );
            }
        }
    }

    /************************************************************************************************
     * CRMMission::PurgeTrigger
     *	Purge the trigger and all its targets
     *
     * inputs:
     *  trigger: trigger to purge
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    /*fn PurgeTrigger ( trigger: *mut CEntity )
    {
        let mut target: *mut CEntity;

        // Purge all targets
        target = entitySystem->GetEntityFromTargetName ( NULL, trigger->GetTarget ( ) );
        while ( target )
        {
            PurgeTrigger ( target );

            target = entitySystem->GetEntityFromTargetName ( target, trigger->GetTarget ( ) );
        }

        // Get rid of the purge trigger
        entitySystem->RemoveEntityWithServer ( trigger );
        entitySystem->RemoveEntity ( trigger );
    }
    */
    /************************************************************************************************
     * CRMMission::PurgeUnlinkedTriggers
     *	Searches the entitySystem form a random arioche trigger that matches the objective name
     *
     * inputs:
     *  none
     *
     * return:
     *	trigger: a random trigger or NULL if one couldnt be found
     *
     ************************************************************************************************/
    /*fn PurgeUnlinkedTriggers ( )
    {
        let mut search: *mut CTriggerAriocheObjective;

        // Start at the first match of the classname
        search = (CTriggerAriocheObjective*) entitySystem->GetEntityFromClassname ( NULL, "trigger_arioche_objective" );

        // Continue on as long as there are triggers
        while ( search )
        {
            let mut purge = search;

            // move on to the next trigger before deleting the entity
            // just in case there are some state issues with the search
            search = (CTriggerAriocheObjective*) entitySystem->GetEntityFromClassname ( search, "trigger_arioche_objective" );

            // Dont purge linked triggers
            if ( purge->GetObjective ( ) )
            {
                continue;
            }

            // Purge the trigger and all its targets
            PurgeTrigger ( purge );
        }
    }
    */
    /************************************************************************************************
     * CRMMission::SpawnNPCTriggers
     *	Spawn triggers across the map which will activate sleeping NPCs
     *
     * inputs:
     *  landscape: landscape to spawn the triggers relative to
     *
     * return:
     *	trigger: a random trigger or NULL if one couldnt be found
     *
     ************************************************************************************************/
    /*fn SpawnNPCTriggers ( landscape: *mut CCMLandScape )
    {
        let mut ent: *mut CEntity;
        let mut i: c_int;
        let mut count: c_int;
        let mut section: f32;

        // Determine how many NPC sections there are in the map
        count   = (landscape->GetBounds()[1][0] - landscape->GetBounds()[0][0]) / 5000.0f;
        section = (landscape->GetBounds()[1][0] - landscape->GetBounds()[0][0]) / count;

        // Drop a trigger down at each NPC section interval except for the first and last.
        for ( i = 1; i < count - 1; i ++ )
        {
            let mut mins: vec3_t;
            let mut maxs: vec3_t;
            let mut origin: vec3_t;

            VectorCopy ( landscape->GetBounds()[0], mins );
            VectorCopy ( landscape->GetBounds()[1], maxs );

            // Set up the mins and maxs for the trigger
            mins[0] = mins[0] + (section * i) - 100.0f32;
            maxs[0] = mins[0] + 100.0f32;
            maxs[2] = maxs[2] + 100.0f32;
            mins[2] = mins[2] - 100.0f32;

            origin[0] = (maxs[0]-mins[0])/2.0f32 + mins[0];
            origin[1] = (maxs[1]-mins[1])/2.0f32 + mins[1];
            origin[2] = (maxs[2]-mins[2])/2.0f32 + mins[2];

            spawnSystem->ClearSpawnFields();
            spawnSystem->AddSpawnField("classname", "trigger_arioche_npcspawner" );
            spawnSystem->AddSpawnField("origin", va("%f %f %f",origin[0],origin[1],origin[2]) );
            spawnSystem->AddSpawnField("target", va("rmg_npc_%i", i + 1) );

            // Spawn the inhabitant, if it fails then fail
            ent = entitySystem->SpawnItem("trigger_arioche_npcspawner");
            if ( !ent )
            {
                continue;
            }

            // Fail if we cant register the entity
            if ( -1 == entitySystem->RegisterEntityWithServer( ent ) )
            {
                entitySystem->RemoveEntity ( ent );
                continue;
            }

            // Normalize the mins and maxs for the X axis since they arent
            // absolute mins and maxs
            mins[0] = -100.0f32;
            maxs[0] = 100.0f32;

            // Adjust the absmin and absmax for the trigger now
            VectorCopy ( mins, ent->r.mins[0] );
            VectorCopy ( maxs, ent->r.maxs[0] );

            Com_DPrintf( "NPC Trigger spawned at '%f %f %f' for targets 'rmg_npc_%i'\n", origin[0], origin[1], origin[2], i + 1 );

            // Set the "ONLY_ONCE" spawn flag
            ent->AddSpawnflags ( 2 );

        #ifdef _GAME
            // initial linking
            gi.SV_LinkEntity( ent );
        #endif
        }

        AttachNPCTriggers ( landscape );
    }
    */

    /************************************************************************************************
     * CRMMission::AttachNPCTriggers
     *	Attaches npc triggers to all unattached npcs
     *
     * inputs:
     *  landscape: landscape triggers were spawned on
     *
     * return:
     *	trigger: a random trigger or NULL if one couldnt be found
     *
     ************************************************************************************************/
    /*fn AttachNPCTriggers ( landscape: *mut CCMLandScape )
    {
        let mut npcList: TNPCList;
        let mut npcFinder: TNPCFinder;
        let mut theNPC: *mut CNPC = NULL;
        let mut npcsegment: c_int;

        GetCharacterManager().GetCharacterList(npcList);

        // Loop through all npcs and reset their accuracy
        for( npcFinder = npcList.begin(); npcFinder != npcList.end(); npcFinder++)
        {
            theNPC = (CNPC*) INPCEnt::GetEntity(*npcFinder);
            if(!theNPC)
            {
                continue;
            }

            npcsegment = (theNPC->r.currentOrigin[0] - landscape->GetMins()[0]) / 5000.0f;

            // All npcs in segment 0 and 1 are immediately spawned, all others wait for the
            // trigger
            if ( npcsegment > 1 )
            {
                entitySystem->RemoveFromTargetNameMap(theNPC);
                theNPC->SetTargetName ( va("rmg_npc_%i", npcsegment ) );
                entitySystem->AddToTargetNameMap(theNPC);

                // Start the NPC in the off position
                theNPC->SetSpawnflags(1);
                theNPC->r.contents	= 0;
                theNPC->r.svFlags	|= SVF_NOCLIENT;
                theNPC->s.eFlags	|= EF_NODRAW;
            }
        }
    }
    */

    fn GetDefaultPadding(&self) -> c_int {
        self.mDefaultPadding
    }
}

// Stub for missing error code constant
const ERR_DROP: c_int = 0;
