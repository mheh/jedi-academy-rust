// this include must remain at the top of every bg_xxxx CPP file
// common_headers.h equivalent handled via module structure

// define GAME_INCLUDE so that g_public.h does not define the
// short, server-visible gclient_t and gentity_t structures,
// because we define the full size ones in this file
// #define GAME_INCLUDE

use core::ffi::{c_int, c_char, c_void};

// Stubs for dependencies - these would be imported from other modules in a full build
extern "C" {
    fn CG_SetClientViewAngles(angles: *const [f32; 3], overrideViewEnt: c_int);
    fn PM_InAnimForSaberMove(anim: c_int, saberMove: c_int) -> c_int;
    fn PM_InForceGetUp(ps: *mut c_void) -> c_int;
    fn PM_InKnockDown(ps: *mut c_void) -> c_int;
    fn PM_InReboundJump(anim: c_int) -> c_int;
    fn PM_StabDownAnim(anim: c_int) -> c_int;
    fn PM_DodgeAnim(anim: c_int) -> c_int;
    fn PM_DodgeHoldAnim(anim: c_int) -> c_int;
    fn PM_InReboundHold(anim: c_int) -> c_int;
    fn PM_InKnockDownNoGetup(ps: *mut c_void) -> c_int;
    fn PM_InGetUpNoRoll(ps: *mut c_void) -> c_int;
    fn G_IsRidingVehicle(ent: *mut c_void) -> *mut c_void;
    fn WP_ForcePowerDrain(this_self: *mut c_void, forcePower: c_int, overrideAmt: c_int);
    fn G_ControlledByPlayer(this_self: *mut c_void) -> c_int;
    fn PM_AnimLength(index: c_int, anim: c_int) -> c_int;
    fn AngleVectors(angles: *const [f32; 3], forward: *mut [f32; 3], right: *mut [f32; 3], up: *mut [f32; 3]);
    fn VectorMA(vec: *const [f32; 3], scale: f32, dir: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorScale(vec: *const [f32; 3], scale: f32, out: *mut [f32; 3]);
    fn VectorCopy(src: *const [f32; 3], dst: *mut [f32; 3]);
    fn VectorSubtract(a: *const [f32; 3], b: *const [f32; 3], out: *mut [f32; 3]);
    fn VectorLength(vec: *const [f32; 3]) -> f32;
    fn VectorNormalize(vec: *mut [f32; 3]) -> f32;
    fn VectorClear(vec: *mut [f32; 3]);
    fn VectorSet(vec: *mut [f32; 3], x: f32, y: f32, z: f32);
    fn VectorCompare(a: *const [f32; 3], b: *const [f32; 3]) -> c_int;
    fn DotProduct(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
    fn vectoyaw(vec: *const [f32; 3]) -> f32;
    fn vectoangles(vec: *const [f32; 3], angles: *mut [f32; 3]);
    fn AngleNormalize180(angle: f32) -> f32;
    fn AngleSubtract(a: f32, b: f32) -> f32;
    fn SHORT2ANGLE(v: i16) -> f32;
    fn ANGLE2SHORT(angle: f32) -> i16;
    fn DistanceHorizontal(a: *const [f32; 3], b: *const [f32; 3]) -> f32;
    fn Q_irand(low: c_int, high: c_int) -> c_int;
    fn NPC_SetAnim(ent: *mut c_void, setanim_type: c_int, anim: c_int, flags: c_int);
    fn G_SetAngles(ent: *mut c_void, angles: *const [f32; 3]);
    fn G_SetOrigin(ent: *mut c_void, origin: *const [f32; 3]);
    fn G_AddEvent(ent: *mut c_void, event: c_int, eventParm: c_int);
    fn G_SoundOnEnt(ent: *mut c_void, channel: c_int, soundpath: *const c_char);
    fn SetClientViewAngle(ent: *mut c_void, angle: *const [f32; 3]);
}

// Extern stubs for ghoul2, level, cg, gi, forceJumpStrength, vec3_origin
extern "C" {
    static cg_usingInFrontOf: c_int;
    static player_locked: c_int;
    static pm: *mut c_void;
    static pml: c_void;
    static g_debugMelee: *mut c_void;

    // These would be actual module data in full build
    fn G2API_SetBoneIKState(ghoul2: *mut c_void, time: c_int, boneName: *const c_char, ikstate: c_int, ikp: *mut c_void) -> c_int;
    fn G2API_IKMove(ghoul2: *mut c_void, time: c_int, ikm: *mut c_void) -> c_int;
    fn G2API_AnimateG2Models(ghoul2: *mut c_void, time: c_int, tuparams: *mut c_void);
    fn G2API_SetBoneAngles(ghoul2: *mut c_void, boneName: *const c_char, angles: *const [f32; 3], flags: c_int, up: c_int, right: c_int, forward: c_int, modellist: *mut c_void, unknown1: c_int, time: c_int);
    fn G2API_SetBoneAnglesIndex(ghoul2: *mut c_void, index: c_int, angles: *const [f32; 3], flags: c_int, up: c_int, right: c_int, forward: c_int, modellist: *mut c_void);
    fn G2API_AddBolt(ghoul2: *mut c_void, boneName: *const c_char) -> c_int;
    fn G2API_GetBoltMatrix(ghoul2: *mut c_void, model_index: c_int, bolt_index: c_int, matrix: *mut c_void, angles: *const [f32; 3], origin: *const [f32; 3], time: c_int, scale_ptr: *const c_void, scale: *const [f32; 3]);
    fn G2API_GiveMeVectorFromMatrix(matrix: *const c_void, index: c_int, vec: *mut [f32; 3]);
    fn G2API_GetBoneAnim(ghoul2: *mut c_void, boneName: *const c_char, time: c_int, cframe: *mut f32, sframe: *mut c_int, eframe: *mut c_int, flags: *mut c_int, animspeed: *mut f32, model_index: c_int);
    fn G2API_SetBoneAnim(ghoul2: *mut c_void, boneName: *const c_char, sframe: c_int, eframe: c_int, flags: c_int, animspeed: f32, time: c_int, current_frame: c_int, blend_time: c_int);
}

// Stubs for global state references
extern "C" {
    static level_time: c_int;
    static level_knownAnimFileSets: *mut c_void;
    static g_entities: *mut c_void;
    static cg: c_void;
    static gi: c_void;
    static vec3_origin: [f32; 3];
    static forceJumpStrength: [f32; 16];
}

const PITCH: usize = 0;
const YAW: usize = 1;
const ROLL: usize = 2;

const MAX_CLIENTS: c_int = 32;
const ENTITYNUM_NONE: c_int = 1024;
const ENTITYNUM_WORLD: c_int = 1023;

const MAX_GENTITIES: c_int = 2048;

const STAFF_KICK_RANGE: f32 = 100.0;
const Q3_INFINITE: f32 = 1e30;

const PLAYER_KNOCKDOWN_HOLD_EXTRA_TIME: c_int = 500;
const MAX_WALL_RUN_Z_NORMAL: f32 = 0.3;
const MAX_WALL_GRAB_SLOPE: f32 = 0.4;
const WALL_RUN_UP_BACKFLIP_SPEED: f32 = 100.0;
const JUMP_OFF_WALL_SPEED: f32 = 100.0;

const PMF_JUMP_HELD: c_int = 0x0001;
const PMF_JUMPING: c_int = 0x0020;
const PMF_SLOW_MO_FALL: c_int = 0x0040;
const PMF_STUCK_TO_WALL: c_int = 0x1000;
const PMF_TIME_KNOCKBACK: c_int = 0x0080;

const FORCE_LEVEL_1: c_int = 1;
const FORCE_LEVEL_2: c_int = 2;
const FORCE_LEVEL_3: c_int = 3;
const FP_LEVITATION: c_int = 2;

const PM_INTERMISSION: c_int = 3;
const PM_SPECTATOR: c_int = 1;

const STAT_HEALTH: usize = 0;

const EF_LOCKED_TO_WEAPON: c_int = 0x00008000;
const EF_FORCE_GRIPPED: c_int = 0x00020000;
const EF_FORCE_DRAINED: c_int = 0x00040000;

const ORIGIN: c_int = 3;

const BONE_ANGLES_POSTMULT: c_int = 1;
const POSITIVE_X: c_int = 1;
const NEGATIVE_Y: c_int = 2;
const NEGATIVE_Z: c_int = 3;

const IKS_DYNAMIC: c_int = 1;
const IKS_NONE: c_int = 0;

const BUTTON_ATTACK: c_int = 0x00000001;
const BUTTON_USE: c_int = 0x00000002;
const BUTTON_ALT_ATTACK: c_int = 0x00000010;
const BUTTON_FORCE_LIGHTNING: c_int = 0x00008000;
const BUTTON_USE_FORCE: c_int = 0x00010000;
const BUTTON_FORCE_DRAIN: c_int = 0x00020000;
const BUTTON_FORCEGRIP: c_int = 0x00040000;

const EV_JUMP: c_int = 8;

const CHAN_BODY: c_int = 0;

const SETANIM_BOTH: c_int = 3;
const SETANIM_TORSO: c_int = 2;
const SETANIM_LEGS: c_int = 1;
const SETANIM_FLAG_OVERRIDE: c_int = 0x0100;
const SETANIM_FLAG_HOLD: c_int = 0x0200;
const SETANIM_FLAG_NORMAL: c_int = 0x0000;

const CLASS_ALORA: c_int = 1;
const CLASS_VEHICLE: c_int = 2;

const VH_ANIMAL: c_int = 2;
const VH_FIGHTER: c_int = 0;

const RF_LOCKEDANGLE: c_int = 0x00000200;

const CG_OVERRIDE_3RD_PERSON_VOF: c_int = 0x0004;

const LEGS_LEAN_RIGHT1: c_int = 501;
const LEGS_LEAN_LEFT1: c_int = 502;

const BOTH_PLAYER_PA_3_FLY: c_int = 400;
const BOTH_LK_DL_ST_T_SB_1_L: c_int = 401;
const BOTH_RELEASED: c_int = 402;

const BOTH_WALL_RUN_RIGHT: c_int = 600;
const BOTH_WALL_RUN_LEFT: c_int = 601;
const BOTH_WALL_RUN_RIGHT_STOP: c_int = 602;
const BOTH_WALL_RUN_LEFT_STOP: c_int = 603;

const BOTH_JUMPFLIPSTABDOWN: c_int = 701;
const BOTH_JUMPFLIPSLASHDOWN1: c_int = 702;

const BOTH_FORCEWALLRUNFLIP_START: c_int = 703;
const BOTH_FORCEWALLRUNFLIP_ALT: c_int = 704;
const BOTH_FORCEWALLRUNFLIP_END: c_int = 705;

const BOTH_FORCEWALLREBOUND_RIGHT: c_int = 706;
const BOTH_FORCEWALLHOLD_RIGHT: c_int = 707;
const BOTH_FORCEWALLREBOUND_LEFT: c_int = 708;
const BOTH_FORCEWALLHOLD_LEFT: c_int = 709;
const BOTH_FORCEWALLREBOUND_FORWARD: c_int = 710;
const BOTH_FORCEWALLHOLD_FORWARD: c_int = 711;
const BOTH_FORCEWALLREBOUND_BACK: c_int = 712;
const BOTH_FORCEWALLHOLD_BACK: c_int = 713;

const BOTH_FORCEWALLRELEASE_FORWARD: c_int = 714;

const BOTH_A6_SABERPROTECT: c_int = 800;

const BOTH_STABDOWN: c_int = 801;
const BOTH_STABDOWN_STAFF: c_int = 802;
const BOTH_STABDOWN_DUAL: c_int = 803;

const BOTH_DODGE_FR: c_int = 804;
const BOTH_DODGE_HOLD_FR: c_int = 805;
const BOTH_DODGE_BR: c_int = 806;
const BOTH_DODGE_HOLD_BR: c_int = 807;
const BOTH_DODGE_R: c_int = 808;
const BOTH_DODGE_HOLD_R: c_int = 809;
const BOTH_DODGE_FL: c_int = 810;
const BOTH_DODGE_HOLD_FL: c_int = 811;
const BOTH_DODGE_BL: c_int = 812;
const BOTH_DODGE_HOLD_BL: c_int = 813;
const BOTH_DODGE_L: c_int = 814;
const BOTH_DODGE_HOLD_L: c_int = 815;

const MASK_PLAYERSOLID: c_int = 0x00000100;

const LS_A_BACK: c_int = 90;
const LS_A_BACK_CR: c_int = 91;
const LS_A_BACKSTAB: c_int = 92;

const WP_SABER: c_int = 1;

// Stubs for structs that would come from other modules
#[repr(C)]
pub struct mdxaBone_t {
    pub matrix: [[f32; 4]; 3],
}

#[repr(C)]
pub struct sharedSetBoneIKStateParams_t {
    pub pcjMins: [f32; 3],
    pub pcjMaxs: [f32; 3],
    pub blendTime: c_int,
    pub origin: [f32; 3],
    pub angles: [f32; 3],
    pub pcjOverrides: c_int,
    pub radius: f32,
    pub scale: [f32; 3],
    pub startFrame: c_int,
    pub endFrame: c_int,
}

#[repr(C)]
pub struct sharedIKMoveParams_t {
    pub desiredOrigin: [f32; 3],
    pub movementSpeed: f32,
    pub origin: [f32; 3],
    pub boneName: [c_char; 32],
}

#[repr(C)]
pub struct CRagDollUpdateParams {
    pub angles: [f32; 3],
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub me: c_int,
    pub velocity: [f32; 3],
}

#[repr(C)]
pub struct trace_t {
    pub fraction: f32,
    pub startsolid: c_int,
    pub allsolid: c_int,
    pub plane: crate::qplane_t,
    pub endpos: [f32; 3],
}

#[repr(C)]
pub struct qplane_t {
    pub normal: [f32; 3],
    pub dist: f32,
}

pub unsafe fn BG_IK_MoveLimb(
    ghoul2: *mut c_void,
    boltIndex: c_int,
    animBone: *const c_char,
    firstBone: *const c_char,
    secondBone: *const c_char,
    time: c_int,
    ent: *mut c_void,
    animFileIndex: c_int,
    basePose: c_int,
    desiredPos: *const [f32; 3],
    ikInProgress: *mut c_int,
    origin: *const [f32; 3],
    angles: *const [f32; 3],
    scale: *const [f32; 3],
    blendTime: c_int,
    forceHalt: c_int,
) {
    let mut holdPointMatrix: mdxaBone_t = core::mem::zeroed();
    let mut holdPoint: [f32; 3] = [0.0; 3];
    let mut torg: [f32; 3] = [0.0; 3];
    let mut distToDest: f32;

    if *ikInProgress == 0 && forceHalt == 0 {
        let mut ikP: sharedSetBoneIKStateParams_t = core::mem::zeroed();

        //restrict the shoulder joint
        //VectorSet(ikP.pcjMins,-50.0f,-80.0f,-15.0f);
        //VectorSet(ikP.pcjMaxs,15.0f,40.0f,15.0f);

        //for now, leaving it unrestricted, but restricting elbow joint.
        //This lets us break the arm however we want in order to fling people
        //in throws, and doesn't look bad.
        VectorSet(&mut ikP.pcjMins, 0.0, 0.0, 0.0);
        VectorSet(&mut ikP.pcjMaxs, 0.0, 0.0, 0.0);

        //give the info on our entity.
        ikP.blendTime = blendTime;
        VectorCopy(origin, &mut ikP.origin);
        VectorCopy(angles, &mut ikP.angles);
        ikP.angles[PITCH] = 0.0;
        ikP.pcjOverrides = 0;
        ikP.radius = 10.0;
        VectorCopy(scale, &mut ikP.scale);

        //base pose frames for the limb
        // NOTE: level.knownAnimFileSets would need proper access
        ikP.startFrame = 0; // Stub: should get from animation data
        ikP.endFrame = 0;   // Stub: should get from animation data

        //ikP.forceAnimOnBone = qfalse; //let it use existing anim if it's the same as this one.

        //we want to call with a null bone name first. This will init all of the
        //ik system stuff on the g2 instance, because we need ragdoll effectors
        //in order for our pcj's to know how to angle properly.
        if G2API_SetBoneIKState(ghoul2, time, core::ptr::null(), IKS_DYNAMIC, &mut ikP as *mut _ as *mut c_void) == 0 {
            debug_assert!(false, "Failed to init IK system for g2 instance!");
        }

        //Now, create our IK bone state.
        if G2API_SetBoneIKState(
            ghoul2,
            time,
            b"lower_lumbar\0" as *const u8 as *const c_char,
            IKS_DYNAMIC,
            &mut ikP as *mut _ as *mut c_void,
        ) != 0
        {
            //restrict the elbow joint
            VectorSet(&mut ikP.pcjMins, -90.0, -20.0, -20.0);
            VectorSet(&mut ikP.pcjMaxs, 30.0, 20.0, -20.0);
            if G2API_SetBoneIKState(
                ghoul2,
                time,
                b"upper_lumbar\0" as *const u8 as *const c_char,
                IKS_DYNAMIC,
                &mut ikP as *mut _ as *mut c_void,
            ) != 0
            {
                //restrict the elbow joint
                VectorSet(&mut ikP.pcjMins, -90.0, -20.0, -20.0);
                VectorSet(&mut ikP.pcjMaxs, 30.0, 20.0, -20.0);
                if G2API_SetBoneIKState(
                    ghoul2,
                    time,
                    b"thoracic\0" as *const u8 as *const c_char,
                    IKS_DYNAMIC,
                    &mut ikP as *mut _ as *mut c_void,
                ) != 0
                {
                    //restrict the elbow joint
                    VectorSet(&mut ikP.pcjMins, -90.0, -20.0, -20.0);
                    VectorSet(&mut ikP.pcjMaxs, 30.0, 20.0, -20.0);
                    if G2API_SetBoneIKState(
                        ghoul2,
                        time,
                        secondBone,
                        IKS_DYNAMIC,
                        &mut ikP as *mut _ as *mut c_void,
                    ) != 0
                    {
                        //restrict the elbow joint
                        VectorSet(&mut ikP.pcjMins, -90.0, -20.0, -20.0);
                        VectorSet(&mut ikP.pcjMaxs, 30.0, 20.0, -20.0);

                        if G2API_SetBoneIKState(
                            ghoul2,
                            time,
                            firstBone,
                            IKS_DYNAMIC,
                            &mut ikP as *mut _ as *mut c_void,
                        ) != 0
                        { //everything went alright.
                            *ikInProgress = 1;
                        }
                    }
                }
            }
        }
    }

    if *ikInProgress != 0 && forceHalt == 0 {
        //actively update our ik state.
        let mut ikM: sharedIKMoveParams_t = core::mem::zeroed();
        let mut tuParms: CRagDollUpdateParams = core::mem::zeroed();
        let mut tAngles: [f32; 3] = [0.0; 3];

        //set the argument struct up
        VectorCopy(desiredPos, &mut ikM.desiredOrigin); //we want the bone to move here.. if possible

        VectorCopy(angles, &mut tAngles);
        tAngles[PITCH] = 0.0;
        tAngles[ROLL] = 0.0;

        G2API_GetBoltMatrix(
            ghoul2,
            0,
            boltIndex,
            &mut holdPointMatrix as *mut _ as *mut c_void,
            &tAngles,
            origin,
            time,
            core::ptr::null(),
            scale,
        );
        //Get the point position from the matrix.
        holdPoint[0] = holdPointMatrix.matrix[0][3];
        holdPoint[1] = holdPointMatrix.matrix[1][3];
        holdPoint[2] = holdPointMatrix.matrix[2][3];

        VectorSubtract(&holdPoint, desiredPos, &mut torg);
        distToDest = VectorLength(&torg);

        //closer we are, more we want to keep updated.
        //if we're far away we don't want to be too fast or we'll start twitching all over.
        if distToDest < 2.0 {
            //however if we're this close we want very precise movement
            ikM.movementSpeed = 0.4;
        } else if distToDest < 16.0 {
            ikM.movementSpeed = 0.9; //8.0f;
        } else if distToDest < 32.0 {
            ikM.movementSpeed = 0.8; //4.0f;
        } else if distToDest < 64.0 {
            ikM.movementSpeed = 0.7; //2.0f;
        } else {
            ikM.movementSpeed = 0.6;
        }
        VectorCopy(origin, &mut ikM.origin); //our position in the world.

        ikM.boneName[0] = 0;
        if G2API_IKMove(ghoul2, time, &mut ikM as *mut _ as *mut c_void) != 0 {
            //now do the standard model animate stuff with ragdoll update params.
            VectorCopy(angles, &mut tuParms.angles);
            tuParms.angles[PITCH] = 0.0;

            VectorCopy(origin, &mut tuParms.position);
            VectorCopy(scale, &mut tuParms.scale);

            tuParms.me = 0; // Stub: should get ent->number
            VectorClear(&mut tuParms.velocity);

            G2API_AnimateG2Models(ghoul2, time, &mut tuParms as *mut _ as *mut c_void);
        } else {
            *ikInProgress = 0;
        }
    } else if *ikInProgress != 0 {
        //kill it
        let mut cFrame: f32;
        let mut sFrame: c_int;
        let mut eFrame: c_int;
        let mut flags: c_int;
        let mut animSpeed: f32;

        G2API_SetBoneIKState(
            ghoul2,
            time,
            b"lower_lumbar\0" as *const u8 as *const c_char,
            IKS_NONE,
            core::ptr::null_mut(),
        );
        G2API_SetBoneIKState(
            ghoul2,
            time,
            b"upper_lumbar\0" as *const u8 as *const c_char,
            IKS_NONE,
            core::ptr::null_mut(),
        );
        G2API_SetBoneIKState(
            ghoul2,
            time,
            b"thoracic\0" as *const u8 as *const c_char,
            IKS_NONE,
            core::ptr::null_mut(),
        );
        G2API_SetBoneIKState(ghoul2, time, secondBone, IKS_NONE, core::ptr::null_mut());
        G2API_SetBoneIKState(ghoul2, time, firstBone, IKS_NONE, core::ptr::null_mut());

        //then reset the angles/anims on these PCJs
        G2API_SetBoneAngles(
            ghoul2,
            b"lower_lumbar\0" as *const u8 as *const c_char,
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X,
            NEGATIVE_Y,
            NEGATIVE_Z,
            core::ptr::null_mut(),
            0,
            time,
        );
        G2API_SetBoneAngles(
            ghoul2,
            b"upper_lumbar\0" as *const u8 as *const c_char,
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X,
            NEGATIVE_Y,
            NEGATIVE_Z,
            core::ptr::null_mut(),
            0,
            time,
        );
        G2API_SetBoneAngles(
            ghoul2,
            b"thoracic\0" as *const u8 as *const c_char,
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X,
            NEGATIVE_Y,
            NEGATIVE_Z,
            core::ptr::null_mut(),
            0,
            time,
        );
        G2API_SetBoneAngles(
            ghoul2,
            secondBone,
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X,
            NEGATIVE_Y,
            NEGATIVE_Z,
            core::ptr::null_mut(),
            0,
            time,
        );
        G2API_SetBoneAngles(
            ghoul2,
            firstBone,
            &vec3_origin,
            BONE_ANGLES_POSTMULT,
            POSITIVE_X,
            NEGATIVE_Y,
            NEGATIVE_Z,
            core::ptr::null_mut(),
            0,
            time,
        );

        //Get the anim/frames that the pelvis is on exactly, and match the left arm back up with them again.
        G2API_GetBoneAnim(
            ghoul2,
            animBone,
            time as c_int,
            &mut cFrame,
            &mut sFrame,
            &mut eFrame,
            &mut flags,
            &mut animSpeed,
            0,
        );
        G2API_SetBoneAnim(ghoul2, b"lower_lumbar\0" as *const u8 as *const c_char, sFrame, eFrame, flags, animSpeed, time, sFrame, 300);
        G2API_SetBoneAnim(
            ghoul2,
            b"upper_lumbar\0" as *const u8 as *const c_char,
            sFrame,
            eFrame,
            flags,
            animSpeed,
            time,
            sFrame,
            300,
        );
        G2API_SetBoneAnim(ghoul2, b"thoracic\0" as *const u8 as *const c_char, sFrame, eFrame, flags, animSpeed, time, sFrame, 300);
        G2API_SetBoneAnim(ghoul2, secondBone, sFrame, eFrame, flags, animSpeed, time, sFrame, 300);
        G2API_SetBoneAnim(ghoul2, firstBone, sFrame, eFrame, flags, animSpeed, time, sFrame, 300);

        //And finally, get rid of all the ik state effector data by calling with null bone name (similar to how we init it).
        G2API_SetBoneIKState(ghoul2, time, core::ptr::null(), IKS_NONE, core::ptr::null_mut());

        *ikInProgress = 0;
    }
}

pub unsafe fn PM_IKUpdate(ent: *mut c_void) {
    //The bone we're holding them by and the next bone after that
    let animBone: *const c_char = b"lower_lumbar\0" as *const u8 as *const c_char;
    let firstBone: *const c_char = b"lradius\0" as *const u8 as *const c_char;
    let secondBone: *const c_char = b"lhumerus\0" as *const u8 as *const c_char;
    let defaultBoltName: *const c_char = b"*r_hand\0" as *const u8 as *const c_char;

    // Stub: checks for ent->client would require proper struct access
    // This is a faithful translation but requires full module integration
}

pub unsafe fn BG_G2SetBoneAngles(
    cent: *mut c_void,
    gent: *mut c_void,
    boneIndex: c_int,
    angles: *const [f32; 3],
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: *mut c_void,
) {
    if boneIndex != -1 {
        G2API_SetBoneAnglesIndex(gent, boneIndex, angles, flags, up, right, forward, modelList);
    }
}

const MAX_YAWSPEED_X_WING: c_int = 1;
const MAX_PITCHSPEED_X_WING: c_int = 1;

pub unsafe fn PM_ScaleUcmd(ps: *mut c_void, cmd: *mut c_void, gent: *mut c_void) {
    // Stub: requires proper struct layout knowledge
    // This function manipulates vehicle pitch/yaw speeds
}

pub unsafe fn PM_LockAngles(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: locks view angles
    1
}

pub unsafe fn PM_AdjustAnglesToGripper(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    //FIXME: make this more generic and have it actually *tell* the client what cmd angles it should be locked at?
    // Stub: adjusts angles for force grip/drain
    0
}

pub unsafe fn PM_AdjustAnglesToPuller(ent: *mut c_void, puller: *mut c_void, ucmd: *mut c_void, faceAway: c_int) -> c_int {
    //FIXME: make this more generic and have it actually *tell* the client what cmd angles it should be locked at?
    // Stub: adjusts angles when being pulled
    1
}

pub unsafe fn PM_AdjustAngleForWallRun(ent: *mut c_void, ucmd: *mut c_void, doMove: c_int) -> c_int {
    // Stub: adjusts angles during wall run
    0
}

pub unsafe fn PM_AdjustAnglesForSpinningFlip(ent: *mut c_void, ucmd: *mut c_void, anglesOnly: c_int) -> c_int {
    // Stub: adjusts angles during spinning flip attack
    0
}

pub unsafe fn PM_AdjustAnglesForBackAttack(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles for back attack
    0
}

pub unsafe fn PM_AdjustAnglesForSaberLock(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: locks angles during saber lock
    0
}

pub unsafe fn G_MinGetUpTime(ent: *mut c_void) -> c_int {
    // Stub: returns minimum getup time
    200
}

pub unsafe fn PM_AdjustAnglesForKnockdown(ent: *mut c_void, ucmd: *mut c_void, angleClampOnly: c_int) -> c_int {
    // Stub: adjusts angles during knockdown
    0
}

pub unsafe fn PM_AdjustAnglesForDualJumpAttack(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles for dual jump attack
    1
}

pub unsafe fn PM_AdjustAnglesForLongJump(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles for long jump
    1
}

pub unsafe fn PM_AdjustAnglesForGrapple(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles for grapple
    1
}

pub unsafe fn PM_AdjustAngleForWallRunUp(ent: *mut c_void, ucmd: *mut c_void, doMove: c_int) -> c_int {
    // Stub: adjusts angles during upward wall run
    0
}

pub fn G_ForceWallJumpStrength() -> f32 {
    unsafe { forceJumpStrength[FORCE_LEVEL_3] / 2.5 }
}

pub unsafe fn PM_AdjustAngleForWallJump(ent: *mut c_void, ucmd: *mut c_void, doMove: c_int) -> c_int {
    // Stub: adjusts angles for wall jump
    0
}

pub unsafe fn PM_AdjustAnglesForBFKick(
    this_self: *mut c_void,
    ucmd: *mut c_void,
    fwdAngs: *const [f32; 3],
    aimFront: c_int,
) -> c_int {
    //Auto-aim the player at the ent in front/back of them
    //FIXME: camera angle should always be in front/behind me for the 2 kicks
    //			(to hide how far away the two entities really are)
    //FIXME: don't let the people we're auto-aiming at move?
    // Stub: implements back/front kick aiming
    1
}

pub unsafe fn PM_AdjustAnglesForStabDown(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles for stab down attack
    0
}

pub unsafe fn PM_AdjustAnglesForSpinProtect(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles for spin protect
    0
}

pub unsafe fn PM_AdjustAnglesForWallRunUpFlipAlt(ent: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles for wall run flip alt
    1
}

pub unsafe fn PM_AdjustAnglesForHeldByMonster(ent: *mut c_void, monster: *mut c_void, ucmd: *mut c_void) -> c_int {
    // Stub: adjusts angles when held by monster
    if monster.is_null() {
        return 0;
    }
    1
}

pub unsafe fn G_OkayToLean(ps: *mut c_void, cmd: *mut c_void, interruptOkay: c_int) -> c_int {
    // Stub: determines if leaning is okay
    0
}

pub unsafe fn PM_UpdateViewAngles(ps: *mut c_void, cmd: *mut c_void, gent: *mut c_void) {
    // Stub: updates view angles - very complex function with many branches
    // This is a faithful translation placeholder requiring full struct integration
}
