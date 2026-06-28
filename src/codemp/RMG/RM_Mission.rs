//Anything above this #include will be ignored by the compiler

/************************************************************************************************
 *
 * RM_Mission.cpp
 *
 * implements the CRMMission class.  The CRMMission class loads and manages an arioche mission
 *
 ************************************************************************************************/

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

// RM_Headers.h equivalent stubs - unported dependencies
mod _stubs {
    use core::ffi::{c_char, c_int};
    use super::*;

    pub struct CRMAreaManager {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CRMPathManager {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CRandomTerrain {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CGPGroup {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CRMNode {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CRMInstance {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CRMArea {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CRMObjective {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CRMInstanceFile {
        // Stub: original depends on unported RM subsystem
    }

    pub struct CGenericParser2 {
        // Stub: original depends on unported RM subsystem
    }
}

const ARIOCHE_CLIPBRUSH_SIZE: c_int = 300;
const CVAR_OBJECTIVE: c_int = 0;

type vec3_t = [f32; 3];
type vec4_t = [f32; 4];
type vec3pair_t = [[f32; 3]; 2];
type rmObjectiveIter_t = usize;
type rmInstanceIter_t = usize;

const MAX_RANDOM_CHOICES: usize = 32;
const DIR_MAX: usize = 4;
const MAX_QPATH: usize = 64;

const SYMMETRY_NONE: i32 = 0;
const SYMMETRY_TOPLEFT: i32 = 1;
const SYMMETRY_BOTTOMRIGHT: i32 = 2;

const SIDE_RED: i32 = 1;
const SIDE_BLUE: i32 = 2;

const PRE_RELEASE_DEMO: bool = false;
const DEDICATED: bool = false;
const FINAL_BUILD: bool = false;

// Stubs for vec3_origin and other globals
const vec3_origin: vec3_t = [0.0f32, 0.0f32, 0.0f32];

#[allow(non_snake_case)]
pub struct CRMMission {
    mCurrentObjective: *mut c_char,  // NULL
    mValidPaths: bool,               // false
    mValidRivers: bool,              // false
    mValidNodes: bool,               // false
    mValidWeapons: bool,             // false
    mValidAmmo: bool,                // false
    mValidObjectives: bool,          // false
    mValidInstances: bool,           // false
    mTimeLimit: c_int,               // 0
    mMaxInstancePosition: c_int,     // 1
    mAccuracyMultiplier: f32,        // 1.0f
    mHealthMultiplier: f32,          // 1.0f
    mPickupHealth: f32,              // 1.0f
    mPickupArmor: f32,               // 1.0f
    mPickupAmmo: f32,                // 1.0f
    mPickupWeapon: f32,              // 1.0f
    mPickupEquipment: f32,           // 1.0f
    mDefaultPadding: c_int,          // 0
    mSymmetric: i32,                 // SYMMETRY_NONE
    mLandScape: *mut c_char,         // Stub: CRandomTerrain*
    mAreaManager: *mut c_char,       // Stub: CRMAreaManager*
    mPathManager: *mut c_char,       // Stub: CRMPathManager*
    mObjectives: Vec<*mut c_char>,   // Stub: vector<CRMObjective*>
    mInstances: Vec<*mut c_char>,    // Stub: vector<CRMInstance*>
    mInstanceFile: *mut c_char,      // Stub: CRMInstanceFile
    mDescription: String,
    mInfo: String,
    mExitScreen: String,
    mTimeExpiredScreen: String,
    mBackUpPath: c_int,
}

impl CRMMission {
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
    pub fn new(landscape: *mut c_char) -> Self {
        // cut down the possible area that is 'legal' for area manager to use by 20%
        let mut land_min: vec3_t = [0.0f32; 3];
        let mut land_max: vec3_t = [0.0f32; 3];

        // Note: Full C++ implementation requires CRandomTerrain, CRMAreaManager, CRMPathManager
        // which are unported dependencies. This is a structural stub.
        CRMMission {
            mCurrentObjective: std::ptr::null_mut(),
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
            mAreaManager: std::ptr::null_mut(),
            mPathManager: std::ptr::null_mut(),
            mObjectives: Vec::new(),
            mInstances: Vec::new(),
            mInstanceFile: std::ptr::null_mut(),
            mDescription: String::new(),
            mInfo: String::new(),
            mExitScreen: String::new(),
            mTimeExpiredScreen: String::new(),
            mBackUpPath: 0,
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
    pub fn drop(&mut self) {
        // Cleanup the objectives
        self.mObjectives.clear();

        // Cleanup the instances
        self.mInstances.clear();

        // mPathManager cleanup
        if !self.mPathManager.is_null() {
            unsafe {
                // Note: In C++, this would be: delete mPathManager; mPathManager = 0;
                // In Rust with unported dependency, we mark null after cleanup
            }
            self.mPathManager = std::ptr::null_mut();
        }

        // mAreaManager cleanup
        if !self.mAreaManager.is_null() {
            unsafe {
                // Note: In C++, this would be: delete mAreaManager; mAreaManager = 0;
                // In Rust with unported dependency, we mark null after cleanup
            }
            self.mAreaManager = std::ptr::null_mut();
        }
    }

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
    pub fn FindObjective(&self, name: &CStr) -> *mut c_char {
        for it in &self.mObjectives {
            // Does it match?
            // if (!stricmp ((*it)->GetName(), name ))
            // {
            //     return (*it);
            // }
        }

        // Not found
        std::ptr::null_mut()
    }

    pub fn MirrorPos(&self, pos: &mut vec3_t) {
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
    pub fn ParseOrigin(
        &mut self,
        originGroup: *mut c_char,
        origin: &mut vec3_t,
        lookat: &mut vec3_t,
        flattenHeight: *mut c_int,
    ) -> bool {
        let mut mins: vec3_t = [0.0f32; 3];
        let mut maxs: vec3_t = [0.0f32; 3];

        if !flattenHeight.is_null() {
            unsafe {
                *flattenHeight = 66;
            }
        }

        // If no group was given then use 0,0,0
        if originGroup.is_null() {
            // VectorCopy ( vec3_origin, origin );
            origin.copy_from_slice(&vec3_origin);
            return false;
        }

        // Note: Full C++ implementation requires CGPGroup::FindPairValue()
        // which is an unported dependency
        // See if attaching to a named node
        // szNodeName = originGroup->FindPairValue ( "node", "" );
        // if ( *szNodeName )
        // {
        //     ... find node logic ...
        // }

        // mins[0] = atof( originGroup->FindPairValue ( "left", ".1" ) );
        // mins[1] = atof( originGroup->FindPairValue ( "top", ".1" ) );
        // maxs[0] = atof( originGroup->FindPairValue ( "right", ".9" ) );
        // maxs[1] = atof( originGroup->FindPairValue ( "bottom", ".9" ) );

        // lookat[0] = origin[0] = mLandScape->flrand(mins[0],maxs[0]);
        // lookat[1] = origin[1] = mLandScape->flrand(mins[1],maxs[1]);
        // lookat[2] = origin[2] = 0;

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
    pub fn ParseNodes(&mut self, group: *mut c_char) -> bool {
        // If NULL that means this particular difficulty level has no named nodes
        if group.is_null() || self.mValidNodes {
            return true;
        }

        // Note: Full C++ implementation requires CGPGroup methods
        // which are unported dependencies
        // how many nodes spaced over map?
        let mut x_cells: c_int = 3;
        let mut y_cells: c_int = 3;

        // x_cells = atoi ( group->FindPairValue ( "x_cells", "3" ) );
        // y_cells = atoi ( group->FindPairValue ( "y_cells", "3" ) );

        // mPathManager->CreateArray(x_cells, y_cells);

        // Loop through all the nodes and generate each as specified
        // for ( group = group->GetSubGroups();
        //       group;
        //       group=group->GetNext() )
        // {
        //     ... create location logic ...
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
    pub fn ParsePaths(&mut self, group: *mut c_char) -> bool {
        // If NULL that means this particular difficulty level has no paths
        if group.is_null() || self.mValidPaths {
            return true;
        }

        // path style info
        let mut depth: f32 = 0.31f32;
        let mut deviation: f32 = 0.025f32;
        let mut breadth: f32 = 5.0f32;
        let mut minwidth: f32 = 0.03f32;
        let mut maxwidth: f32 = 0.05f32;
        let mut points: c_int = 10;

        // points    = atoi ( group->FindPairValue ( "points", "10" ) );
        // depth     = atof ( group->FindPairValue ( "depth", ".31" ) );
        // deviation = atof ( group->FindPairValue ( "deviation", ".025" ) );
        // breadth   = atof ( group->FindPairValue ( "breadth", "5" ) );
        // minwidth  = atof ( group->FindPairValue ( "minwidth", ".03" ) );
        // maxwidth  = atof ( group->FindPairValue ( "maxwidth", ".05" ) );

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
    pub fn ParseRivers(&mut self, group: *mut c_char) -> bool {
        // If NULL that means this particular difficulty level has no rivers
        if group.is_null() || self.mValidRivers {
            return true;
        }

        // river style info
        let mut maxdepth: c_int = 5;
        let mut beddepth: f32 = 1.0f32;
        let mut deviation: f32 = 0.03f32;
        let mut breadth: f32 = 7.0f32;
        let mut minwidth: f32 = 0.01f32;
        let mut maxwidth: f32 = 0.03f32;
        let mut points: c_int = 10;

        // maxdepth  = atoi ( group->FindPairValue ( "maxpathdepth", "5" ) );
        // points    = atoi ( group->FindPairValue ( "points", "10" ) );
        // beddepth  = atof ( group->FindPairValue ( "depth", "1" ) );
        // deviation = atof ( group->FindPairValue ( "deviation", ".03" ) );
        // breadth   = atof ( group->FindPairValue ( "breadth", "7" ) );
        // minwidth  = atof ( group->FindPairValue ( "minwidth", ".01" ) );
        // maxwidth  = atof ( group->FindPairValue ( "maxwidth", ".03" ) );

        // mPathManager->SetRiverStyle( maxdepth, points, minwidth, maxwidth, beddepth, deviation, breadth, bridge_name);

        if !self.mValidRivers && beddepth < 1.0f32 {
            // use a depth of 1 if we don't want any rivers
            // we must create rivers
            // mPathManager->GenerateRivers();
            self.mValidRivers = true;
        }

        true
    }

    pub fn PlaceBridges(&mut self) {
        // if (!mValidRivers || strlen(mPathManager->GetBridgeName()) < 1)
        //     return;

        let mut max_bridges: c_int = 0;
        let mut path: c_int = 0;
        let mut t: f32 = 0.0f32;
        // float river_depth = mLandScape->GetLandScape()->GetWaterHeight();
        let mut pos: vec3_t = [0.0f32; 3];
        let mut lastpos: vec3_t = [0.0f32; 3];
        let mut bounds: vec3pair_t = [[0.0f32; 3]; 2];

        // Note: Full C++ implementation requires CRandomTerrain, CCMLandScape methods
        // which are unported dependencies
        // walk along paths looking for dips
        // for (path = 0; path < mPathManager->GetPathCount(); path++)
        // {
        //     ... bridge placement logic ...
        // }
    }

    pub fn PlaceWallInstance(
        &mut self,
        instance: *mut c_char,
        xpos: f32,
        ypos: f32,
        zpos: f32,
        x: c_int,
        y: c_int,
        angle: f32,
    ) {
        // if (NULL == instance)
        //     return;

        // let spacing = instance->GetSpacingRadius();
        let mut origin: vec3_t = [0.0f32; 3];

        // origin[0] = xpos + spacing * x;
        // origin[1] = ypos + spacing * y;
        // origin[2] = zpos;

        // Note: Full C++ implementation requires CRMAreaManager, CRMInstance methods
        // which are unported dependencies
        // Set the area of position
        // area = mAreaManager->CreateArea ( origin, (spacing / 2.1f), 0, GetDefaultPadding(), 0, vec3_origin, origin, instance->GetFlattenRadius()?true:false, false, instance->GetLockOrigin() );
        // area->EnableLookAt(false);
        // area->SetAngle(angle);
        // instance->SetArea ( mAreaManager, area );

        // mInstances.push_back ( instance );
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
    pub fn ParseWallRect(&mut self, group: *mut c_char, side: c_int) -> bool {
        if !PRE_RELEASE_DEMO {
            // Note: Full C++ implementation requires CGPGroup methods
            // which are unported dependencies

            // CGPGroup* wallGroup = group->FindSubGroup ( "wallrect" ) ;

            // If NULL that means this particular instance has no wall rect
            // if ( NULL == group || NULL == wallGroup)
            // {
            //     return true;
            // }

            // ... wall rectangle parsing logic ...
        }

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
    pub fn ParseInstancesOnPath(&mut self, group: *mut c_char) -> bool {
        if !PRE_RELEASE_DEMO {
            // Note: Full C++ implementation requires CGPGroup iteration
            // which is an unported dependency

            // CGPGroup* defenseGroup;
            // for ( defenseGroup = group->GetSubGroups();
            //       defenseGroup;
            //       defenseGroup=defenseGroup->GetNext() )
            // if (stricmp ( defenseGroup->GetName ( ), "defenses" )==0 ||
            //     stricmp ( defenseGroup->GetName(), "instanceonpath")==0)
            // {
            //     ... defense instance parsing logic ...
            // }
        }

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
    pub fn ParseInstance(&mut self, group: *mut c_char) -> bool {
        let mut origin: vec3_t = [0.0f32; 3];
        let mut lookat: vec3_t = [0.0f32; 3];
        let mut flattenHeight: c_int = 0;

        // Note: Full C++ implementation requires CGPGroup, CRMInstance methods
        // which are unported dependencies

        // create fences / walls

        // Create the instance using the instance file helper class
        // instance = mInstanceFile.CreateInstance ( group->GetName ( ) );

        // Failed to create, not good
        // if ( NULL == instance )
        // {
        //     return false;
        // }

        // ... instance parsing logic ...

        // create defenses?
        // ParseInstancesOnPath(group );

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
    pub fn ParseInstances(&mut self, group: *mut c_char) -> bool {
        if !PRE_RELEASE_DEMO {
            // If NULL that means this particular difficulty level has no instances
            if group.is_null() {
                return true;
            }

            // Note: Full C++ implementation requires CGPGroup iteration
            // which is an unported dependency

            // Loop through all the instances in the mission and add each
            // to the master list of instances
            // for ( group = group->GetSubGroups();
            //       group;
            //       group=group->GetNext() )
            // {
            //     ParseInstance ( group );
            // }
        }

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
    pub fn ParseObjectives(&mut self, group: *mut c_char) -> bool {
        // If NULL that means this particular difficulty level has no objectives
        if group.is_null() {
            return true;
        }

        // Note: Full C++ implementation requires CGPGroup iteration and CRMObjective constructor
        // which are unported dependencies

        // Loop through all the objectives in the mission and add each
        // to the master list of objectives
        // for ( group = group->GetSubGroups();
        //       group;
        //       group=group->GetNext() )
        // {
        //     CRMObjective* objective;

        //     // Create the new objective
        //     objective = new CRMObjective ( group );

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
    pub fn ParseAmmo(&mut self, ammos: *mut c_char) -> bool {
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
    pub fn ParseWeapons(&mut self, weapons: *mut c_char) -> bool {
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
    pub fn ParseOutfit(&mut self, outfit: *mut c_char) -> bool {
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
    pub fn ParseRandom(&self, randomGroup: *mut c_char) -> *mut c_char {
        if randomGroup.is_null() {
            return std::ptr::null_mut();
        }

        let mut groups: [*mut c_char; MAX_RANDOM_CHOICES] = [std::ptr::null_mut(); MAX_RANDOM_CHOICES];
        let mut numGroups: usize = 0;

        // Note: Full C++ implementation requires CGPGroup iteration
        // which is an unported dependency

        // Build a list of the groups one can be chosen
        // for ( numGroups = 0, group = randomGroup->GetSubGroups ( );
        //       group;
        //       group = group->GetNext ( ) )
        // {
        //     if ( stricmp ( group->GetName ( ), "random_choice" ) )
        //     {
        //         continue;
        //     }

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
    pub fn ParseDifficulty(&mut self, difficulty: *mut c_char) -> bool {
        // If a null difficulty then stop the recursion.  Make sure to
        // return true here so the parsing doesnt fail
        if difficulty.is_null() {
            return true;
        }

        // Note: Full C++ implementation requires CGPGroup methods
        // which are unported dependencies

        // is map supposed to be symmetric?
        // mSymmetric = (symmetry_t)atoi(difficulty->GetParent()->FindPairValue ( "symmetric", "0" ));
        // mBackUpPath = atoi(difficulty->GetParent()->FindPairValue ( "backuppath", "0" ));

        if self.mSymmetric != 0 {
            // pick between the 2 starting corners -- yes this is a hack
            self.mSymmetric = SYMMETRY_TOPLEFT;
            // if( TheRandomMissionManager->GetLandScape()->irand(0, 1) )
            // {
            //     mSymmetric = SYMMETRY_BOTTOMRIGHT;
            // }
        }

        // mDefaultPadding = atoi(difficulty->GetParent()->FindPairValue ( "padding", "0" ));

        // Parse the nodes
        // if ( !ParseNodes (  ParseRandom ( difficulty->FindSubGroup ( "nodes" ) ) ) )
        // {
        //     return false;
        // }

        // Parse the paths
        // if ( !ParsePaths (  ParseRandom ( difficulty->FindSubGroup ( "paths" ) ) ) )
        // {
        //     return false;
        // }

        // Parse the rivers
        // if ( !ParseRivers (  ParseRandom ( difficulty->FindSubGroup ( "rivers" ) ) ) )
        // {
        //     return false;
        // }

        // Handle inherited properties
        // if ( !ParseDifficulty ( difficulty->GetParent ( )->FindSubGroup ( difficulty->FindPairValue ( "inherit", "" ) ) ) )
        // {
        //     return false;
        // }

        /*
        // parse the player's outfit (weapons and ammo)
        if ( !ParseOutfit( ParseRandom ( difficulty->FindSubGroup ( "outfit" ) ) ) )
        {
            // Its ok to fail parsing weapons as long as weapons have
            // already been parsed at some point
            if ( !ParseWeapons ( ParseRandom ( difficulty->FindSubGroup ( "weapons" ) ) ) )
            {
                if ( !mValidWeapons )
                {
                    return false;
                }
            }

            // Its ok to fail parsing ammo as long as ammo have
            // already been parsed at some point
            if ( !ParseAmmo ( ParseRandom ( difficulty->FindSubGroup ( "ammo" ) ) ) )
            {
                if ( !mValidAmmo)
                {
                    return false;
                }
            }
        }

        // Its ok to fail parsing objectives as long as objectives have
        // already been parsed at some point
        if ( !ParseObjectives ( ParseRandom ( difficulty->FindSubGroup ( "objectives" ) ) ) )
        {
            if ( !mValidObjectives )
            {
                return false;
            }
        }
        */

        // Set the cvars with the available values
        // Cvar_Set ( "mi_health", difficulty->FindPairValue ( "health", "100" ) );
        // Cvar_Set ( "mi_armor", difficulty->FindPairValue ( "armor", "0" ) );

        // Parse out the timelimit
        // mTimeLimit = atol(difficulty->FindPairValue("timelimit", "0"));

        // NPC multipliers
        // mAccuracyMultiplier = atof(difficulty->FindPairValue("npcaccuracy", "1"));
        // mHealthMultiplier = atof(difficulty->FindPairValue("npchealth", "1"));

        // keep only some of RMG pickups 1 = 100%
        // mPickupHealth = atof(difficulty->FindPairValue("pickup_health", "1"));
        // mPickupArmor = atof(difficulty->FindPairValue("pickup_armor", "1"));
        // mPickupAmmo = atof(difficulty->FindPairValue("pickup_ammo", "1"));
        // mPickupWeapon = atof(difficulty->FindPairValue("pickup_weapon", "1"));
        // mPickupEquipment = atof(difficulty->FindPairValue("pickup_equipment", "1"));

        // Its ok to fail parsing instances as long as instances have
        // already been parsed at some point
        // if ( !ParseInstances ( ParseRandom ( difficulty->FindSubGroup ( "instances" ) ) ) )
        // {
        //     if ( !mValidInstances )
        //     {
        //         return false;
        //     }
        // }

        true
    }

    pub fn GetDefaultPadding(&self) -> c_int {
        self.mDefaultPadding
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
    pub fn Load(
        &mut self,
        mission: &CStr,
        instances: &CStr,
        difficulty: &CStr,
    ) -> bool {
        // Note: Full C++ implementation requires CGenericParser2, Com_ParseTextFile
        // which are unported dependencies

        // CGenericParser2		parser;
        // CGPGroup*			root;

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
        // Cvar_Set("ar_obj_main0",mDescription.c_str(), CVAR_OBJECTIVE);
        // Cvar_Set("ar_obj_maincom0", "&OBJECTIVES_INPROGRESS&", CVAR_OBJECTIVE);
        // Cvar_SetValue ("ar_cur_objective", 0, CVAR_OBJECTIVE);

        // string mInfo = root->FindPairValue ( "info", "<MISSION ADDITIONAL INFO MISSING>" );
        // Cvar_Set("ar_obj_info0",mInfo.c_str(), CVAR_OBJECTIVE);

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
        // if ( !ParseDifficulty ( root->FindSubGroup ( difficulty ) ) )
        // {
        //     return false;
        // }

        // Generate the terrain now
        // mLandScape->Generate(mSymmetric);

        // Cleanup
        // parser.Clean();

        true
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
     ***********************************************************************************************/
    pub fn Spawn(&mut self, terrain: *mut c_char, IsServer: bool) -> bool {
        if !PRE_RELEASE_DEMO {
            let mut areaIndex: c_int = 0;

            if IsServer {
                // Prespawn all instances, this is mainly for flattening
                // for(it = mInstances.begin(); it != mInstances.end(); it++)
                // {
                //     CRMInstance* instance = *it;

                //     // Pre-Spawn
                //     instance->PreSpawn ( terrain, IsServer );

                //     if (mSymmetric)
                //     {
                //         instance->SetMirror(1);
                //         instance->PreSpawn ( terrain, IsServer );
                //         instance->SetMirror(0);
                //     }
                // }

                // mLandScape->Smooth ( );

                // place bridges
                // PlaceBridges();
            }

            if !DEDICATED {
                // else
                // {
                //     memcpy ( mLandScape->GetLandScape()->GetHeightMap ( ), clc.rmgHeightMap, mLandScape->GetLandScape()->GetRealArea ( ) );
                //     memcpy ( mLandScape->GetLandScape()->GetFlattenMap ( ), clc.rmgFlattenMap, mLandScape->GetLandScape()->GetRealArea ( ) );
                //     mLandScape->GetLandScape()->rand_seed ( clc.rmgSeed );
                // }
            }

            // mLandScape->GetLandScape()->UpdatePatches();

            if IsServer {
                // Spawn all instances
                // for(it = mInstances.begin(); it != mInstances.end(); it++)
                // {
                //     CRMInstance* instance = *it;

                //     // Spawn
                //     instance->Spawn ( terrain, IsServer );
                //     instance->PostSpawn ( terrain, IsServer );

                //     if (mSymmetric)
                //     {	// spawn the mirror version
                //         instance->SetMirror(1);
                //         instance->Spawn ( terrain, IsServer );
                //         instance->PostSpawn ( terrain, IsServer );
                //         instance->SetMirror(0);
                //     }
                // }
            }

            // create automap
            // if (!com_dedicated->integer)
            // {
            //     if !DEDICATED {
            //         // CM_TM_Create(mLandScape->GetLandScape());

            //         if ( IsServer )
            //         {
            //             CRMManager::ProcessAutomapSymbols ( TheRandomMissionManager->GetAutomapSymbolCount(), TheRandomMissionManager->GetAutomapSymbol(0) );
            //         }
            //         else
            //         {
            //             CRMManager::ProcessAutomapSymbols ( clc.rmgAutomapSymbolCount, clc.rmgAutomapSymbols );
            //         }
            //     }
            // }

            if !FINAL_BUILD {
                // make sure to write out after the mirror happens so red side is displayed on map
                // if (1 == Cvar_VariableIntegerValue("rmg_saveautomap"))
                // {	// write out automap for test purposes
                //     char seed[MAX_QPATH];
                //     char terrainName[MAX_QPATH];
                //     char missionName[MAX_QPATH];
                //     Cvar_VariableStringBuffer("RMG_seed", seed, MAX_QPATH);
                //     Cvar_VariableStringBuffer("RMG_terrain", terrainName, MAX_QPATH);
                //     Cvar_VariableStringBuffer("RMG_mission", missionName, MAX_QPATH);

                //     if !DEDICATED {
                //         // for(it = mInstances.begin(); it != mInstances.end(); it++)
                //         // {
                //         //     (*it)->DrawAutomapSymbol();
                //         // }
                //         // gi.CM_TM_SaveImageToDisk(terrainName, missionName, seed);
                //         // CM_TM_SaveImageToDisk(terrainName, missionName, seed);
                //     }
                //     // Com_Error (ERR_DROP, "RMG Automap written.");
                //     return false;
                // }
            }

            // draw player start on automap
            // CEntity	*spot = NULL;
            // spot = entitySystem->GetEntityFromClassname( spot, "info_player_start");
            // if (spot)
            // {
            //     gi.CM_TM_AddStart(spot->GetOrigin()[0], spot->GetOrigin()[1]);
            // }

            // Spawn NPC triggers now
            // SpawnNPCTriggers ( mLandScape );

            // Restory all the NPC's accuracies to the template accuracies times the
            // multiplier
            // INPCEnt::RestoreTemplate ( mAccuracyMultiplier, mHealthMultiplier );

            if IsServer {
                // Little trick to set the current objective to the first in the list
                // CompleteObjective ( NULL );

                // Iterate through the areas and add each to the landscapes list, this is sorta hacky
                // but bridges the game / common gap
                // for ( areaIndex = 0; NULL != (area = mAreaManager->EnumArea ( areaIndex )); areaIndex ++ )
                // {
                //     // Dont bother adding it to the list if collision isnt enabled
                //     if ( !area->IsCollisionEnabled() )
                //     {
                //         continue;
                //     }

                //     CArea* newarea = new CArea ( );
                //     newarea->Init ( area->GetOrigin(), area->GetSpacingRadius (), 0, area->IsFlattened()?AT_FLAT:AT_NONE );
                //     mLandScape->GetLandScape()->SaveArea( newarea );

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
            }
        }

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
        // Cvar_Set ("cl_paused", "1");

        // AddText(va("killserver; menu %s\n", mExitScreen.c_str()));
        return;
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
        // Cvar_Set ("cl_paused", "1");

        if TimeExpired {
            // AddText(va("killserver; menu %s\n", mTimeExpiredScreen.c_str()));
        }
        return;
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
    pub fn CompleteObjective(&mut self, objective: *mut c_char) {
        // Set the object as completed
        if !objective.is_null() {
            // objective->Complete ( true );

            // Set the completed text for the objective
            // gi.Cvar_Set( va("ar_obj_subcom0_%i", objective->GetOrderIndex ( )), "&OBJECTIVES_COMPLETE&", CVAR_OBJECTIVE) ;

            /*		CEntity *tent = G_TempEntity( vec3_origin, EV_SUB_PRINT );
            tent->s.time2 = gi.SP_GetStringID ( objective->GetMessage ( ) );
            tent->r.svFlags |= SVF_BROADCAST;
            G_AddTempEntity(tent);

            if (objective->CompleteSoundID())
                G_SoundBroadcast( tent, objective->CompleteSoundID());
            */
        }

        self.mCurrentObjective = std::ptr::null_mut();

        // Find the next objective
        // for (it = mObjectives.begin(); it != mObjectives.end(); it++)
        // {
        //     objective = (*it);

        //     // Skip completed objectives
        //     if ( objective->IsCompleted ( ) )
        //     {
        //         continue;
        //     }

        //     // Find the objective with the lowest priority
        //     if ( mCurrentObjective && objective->GetPriority ( ) > mCurrentObjective->GetPriority ( ) )
        //     {
        //         continue;
        //     }

        //     // Found one
        //     mCurrentObjective = objective;
        // }

        // if ( NULL != mCurrentObjective )
        // {
        //     Cvar_SetValue ("ar_cur_objective", mCurrentObjective->GetOrderIndex ( ), CVAR_OBJECTIVE);

        //     mCurrentObjective->Activate ( );
        // }
        // else
        // {
        //     // Set the completed text for the objective
        //     Cvar_Set( "ar_obj_maincom0", "&OBJECTIVES_COMPLETE&", CVAR_OBJECTIVE) ;
        // }
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
        // for(it = mInstances.begin(); it != mInstances.end(); it++)
        // {
        //     CRMInstance* instance = *it;

        //     vec3_t a;
        //     vec3_t b;

        //     VectorCopy ( from, a );
        //     VectorCopy ( instance->GetOrigin(), b );

        //     a[2] = 0;
        //     b[2] = 0;

        //     // Skip stuff thats too far away
        //     if ( Distance ( a, b) > 2000 )
        //     {
        //         continue;
        //     }

        //     instance->Preview ( from );
        // }
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
    /*pub fn PurgeTrigger ( &mut self, trigger: *mut c_char )
    {
        // CEntity* target;

        // Purge all targets
        // target = entitySystem->GetEntityFromTargetName ( NULL, trigger->GetTarget ( ) );
        // while ( target )
        // {
        //     PurgeTrigger ( target );

        //     target = entitySystem->GetEntityFromTargetName ( target, trigger->GetTarget ( ) );
        // }

        // Get rid of the purge trigger
        // entitySystem->RemoveEntityWithServer ( trigger );
        // entitySystem->RemoveEntity ( trigger );
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
    /*pub fn PurgeUnlinkedTriggers ( &mut self )
    {
        // CTriggerAriocheObjective*	search;

        // Start at the first match of the classname
        // search = (CTriggerAriocheObjective*) entitySystem->GetEntityFromClassname ( NULL, "trigger_arioche_objective" );

        // Continue on as long as there are triggers
        // while ( search )
        // {
        //     CTriggerAriocheObjective* purge = search;

        //     // move on to the next trigger before deleting the entity
        //     // just in case there are some state issues with the search
        //     search = (CTriggerAriocheObjective*) entitySystem->GetEntityFromClassname ( search, "trigger_arioche_objective" );

        //     // Dont purge linked triggers
        //     if ( purge->GetObjective ( ) )
        //     {
        //         continue;
        //     }

        //     // Purge the trigger and all its targets
        //     PurgeTrigger ( purge );
        // }
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
    /*pub fn SpawnNPCTriggers ( &mut self, landscape: *mut c_char )
    {
        // CEntity* ent;
        // int		 i;
        // int		 count;
        // float	 section;

        // Determine how many NPC sections there are in the map
        // count   = (landscape->GetBounds()[1][0] - landscape->GetBounds()[0][0]) / 5000.0f;
        // section = (landscape->GetBounds()[1][0] - landscape->GetBounds()[0][0]) / count;

        // Drop a trigger down at each NPC section interval except for the first and last.
        // for ( i = 1; i < count - 1; i ++ )
        // {
        //     vec3_t mins;
        //     vec3_t maxs;
        //     vec3_t origin;

        //     VectorCopy ( landscape->GetBounds()[0], mins );
        //     VectorCopy ( landscape->GetBounds()[1], maxs );

        //     // Set up the mins and maxs for the trigger
        //     mins[0] = mins[0] + (section * i) - 100;
        //     maxs[0] = mins[0] + 100;
        //     maxs[2] = maxs[2] + 100;
        //     mins[2] = mins[2] - 100;

        //     origin[0] = (maxs[0]-mins[0])/2 + mins[0];
        //     origin[1] = (maxs[1]-mins[1])/2 + mins[1];
        //     origin[2] = (maxs[2]-mins[2])/2 + mins[2];

        //     spawnSystem->ClearSpawnFields();
        //     spawnSystem->AddSpawnField("classname", "trigger_arioche_npcspawner" );
        //     spawnSystem->AddSpawnField("origin", va("%f %f %f",origin[0],origin[1],origin[2]) );
        //     spawnSystem->AddSpawnField("target", va("rmg_npc_%i", i + 1) );

        //     // Spawn the inhabitant, if it fails then fail
        //     ent = entitySystem->SpawnItem("trigger_arioche_npcspawner");
        //     if ( !ent )
        //     {
        //         continue;
        //     }

        //     // Fail if we cant register the entity
        //     if ( -1 == entitySystem->RegisterEntityWithServer( ent ) )
        //     {
        //         entitySystem->RemoveEntity ( ent );
        //         continue;
        //     }

        //     // Normalize the mins and maxs for the X axis since they arent
        //     // absolute mins and maxs
        //     mins[0] = -100;
        //     maxs[0] = 100;

        //     // Adjust the absmin and absmax for the trigger now
        //     VectorCopy ( mins, ent->r.mins[0] );
        //     VectorCopy ( maxs, ent->r.maxs[0] );

        //     Com_DPrintf( "NPC Trigger spawned at '%f %f %f' for targets 'rmg_npc_%i'\n", origin[0], origin[1], origin[2], i + 1 );

        //     // Set the "ONLY_ONCE" spawn flag
        //     ent->AddSpawnflags ( 2 );

        // #ifdef _GAME
        //     // initial linking
        //     gi.SV_LinkEntity( ent );
        // #endif
        // }

        // AttachNPCTriggers ( landscape );
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
    /*pub fn AttachNPCTriggers ( &mut self, landscape: *mut c_char )
    {
        // TNPCList	npcList;
        // TNPCFinder	npcFinder;
        // CNPC*		theNPC = 0;
        // int			npcsegment;

        // GetCharacterManager().GetCharacterList(npcList);

        // Loop through all npcs and reset their accuracy
        // for( npcFinder = npcList.begin(); npcFinder != npcList.end(); npcFinder++)
        // {
        //     theNPC = (CNPC*) INPCEnt::GetEntity(*npcFinder);
        //     if(!theNPC)
        //     {
        //         continue;
        //     }

        //     npcsegment = (theNPC->r.currentOrigin[0] - landscape->GetMins()[0]) / 5000.0f;

        //     // All npcs in segment 0 and 1 are immediately spawned, all others wait for the
        //     // trigger
        //     if ( npcsegment > 1 )
        //     {
        //         entitySystem->RemoveFromTargetNameMap(theNPC);
        //         theNPC->SetTargetName ( va("rmg_npc_%i", npcsegment ) );
        //         entitySystem->AddToTargetNameMap(theNPC);

        //         // Start the NPC in the off position
        //         theNPC->SetSpawnflags(1);
        //         theNPC->r.contents	= 0;
        //         theNPC->r.svFlags	|= SVF_NOCLIENT;
        //         theNPC->s.eFlags	|= EF_NODRAW;
        //     }
        // }
    }
    */
}
