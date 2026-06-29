// leave this line at the top for all g_xxxx.cpp files...
// #include "g_headers.h"

// #include "g_local.h"
// #include "g_functions.h"
// #include "bg_public.h"

use core::ffi::c_int;

// Extern declarations for symbols defined elsewhere
extern "C" {
    pub static mut g_spskill: *mut crate::cvar_t;

    pub fn G_ModelIndex(modelName: *const core::ffi::c_char) -> c_int;
    pub fn G_SetOrigin(ent: *mut crate::gentity_t, origin: *const f32);
    pub fn G_SpawnInt(key: *const core::ffi::c_char, defaultValue: *const core::ffi::c_char, out: *mut c_int);
    pub fn G_Spawn() -> *mut crate::gentity_t;
    pub fn G_SpawnItem(ent: *mut crate::gentity_t, item: *const crate::gitem_t);
    pub fn FinishSpawningItem(ent: *mut crate::gentity_t);
    pub fn FindItemForWeapon(weapon: c_int) -> *mut crate::gitem_t;
    pub fn FindItemForAmmo(ammo: c_int) -> *mut crate::gitem_t;
    pub fn FindItem(name: *const core::ffi::c_char) -> *mut crate::gitem_t;
    pub fn RegisterItem(item: *mut crate::gitem_t);
    pub fn LaunchItem(item: *mut crate::gitem_t, origin: *const f32, velocity: *const f32, target: *mut core::ffi::c_char) -> *mut crate::gentity_t;
    pub fn AngleVectors(angles: *const f32, forward: *mut f32, right: *mut f32, up: *mut f32);
    pub fn AngleNormalize180(angle: f32) -> f32;
    pub fn VectorSet(v: *mut f32, x: f32, y: f32, z: f32);
    pub fn VectorCopy(src: *const f32, dst: *mut f32);
    pub fn VectorMA(veca: *const f32, scale: f32, vecb: *const f32, vecc: *mut f32);
    pub fn VectorScale(v: *const f32, scale: f32, out: *mut f32);
    pub fn G_SetAngles(ent: *mut crate::gentity_t, angles: *const f32);
    pub fn crandom() -> f32;
    pub fn random() -> f32;

    pub static mut level: crate::level_t;
    pub static mut gi: crate::game_import_t;
    pub static mut cg: crate::cg_t;
}

// Local stubs for C library functions
fn strlen(s: *const core::ffi::c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

fn strncpy(dst: *mut core::ffi::c_char, src: *const core::ffi::c_char, n: usize) {
    unsafe {
        for i in 0..n {
            let ch = *src.add(i);
            *dst.add(i) = ch;
            if ch == 0 {
                break;
            }
        }
    }
}

fn strcat(dst: *mut core::ffi::c_char, src: *const core::ffi::c_char) {
    unsafe {
        let mut dst_len = 0;
        while *dst.add(dst_len) != 0 {
            dst_len += 1;
        }
        let mut src_idx = 0;
        loop {
            let ch = *src.add(src_idx);
            *dst.add(dst_len + src_idx) = ch;
            if ch == 0 {
                break;
            }
            src_idx += 1;
        }
    }
}

//
// Helper functions
//
//------------------------------------------------------------
unsafe fn SetMiscModelModels(modelNameString: *const core::ffi::c_char, ent: *mut crate::gentity_t, damage_model: bool)
{
    let mut damageModel: [core::ffi::c_char; 256] = [0; 256];
    let mut chunkModel: [core::ffi::c_char; 256] = [0; 256];
    let mut len: usize;

    //Main model
    (*ent).s.modelindex = G_ModelIndex(modelNameString);

    if damage_model
    {
        len = strlen(modelNameString) - 4; // extract the extension

        //Dead/damaged model
        strncpy(damageModel.as_mut_ptr(), modelNameString, len);
        damageModel[len] = 0;
        strncpy(chunkModel.as_mut_ptr(), damageModel.as_ptr(), 256);
        strcat(damageModel.as_mut_ptr(), "_d1.md3\0".as_ptr() as *const core::ffi::c_char);
        (*ent).s.modelindex2 = G_ModelIndex(damageModel.as_ptr());

        (*ent).spawnflags |= 4; // deadsolid

        //Chunk model
        strcat(chunkModel.as_mut_ptr(), "_c1.md3\0".as_ptr() as *const core::ffi::c_char);
        (*ent).s.modelindex3 = G_ModelIndex(chunkModel.as_ptr());
    }
}

//------------------------------------------------------------
// Extern declarations for function pointers (for function references)
extern "C" {
    pub fn G_NewString(str: *const core::ffi::c_char) -> *const core::ffi::c_char;
    pub fn misc_model_breakable_die(
        self_: *mut crate::gentity_t,
        inflictor: *mut crate::gentity_t,
        attacker: *mut crate::gentity_t,
        damage: c_int,
        mod_: c_int,
    );
    pub fn G_ParseAnimFileSet(skeletonName: *const core::ffi::c_char, modelName: *const core::ffi::c_char) -> c_int;
}

unsafe fn SetMiscModelDefaults(
    ent: *mut crate::gentity_t,
    use_func: crate::useFunc_t,
    material: *const core::ffi::c_char,
    solid_mask: c_int,
    animFlag: c_int,
    take_damage: bool,
    damage_model: bool,
) {
    // Apply damage and chunk models if they exist
    SetMiscModelModels((*ent).model, ent, damage_model);

    (*ent).s.eFlags = animFlag as u32;
    (*ent).svFlags |= 0x00000001; // SVF_PLAYER_USABLE
    (*ent).contents = solid_mask;

    G_SetOrigin(ent, (*ent).s.origin.as_ptr());
    VectorCopy((*ent).s.angles.as_ptr(), (*ent).s.apos.trBase.as_mut_ptr());
    (*gi.linkentity)(ent);

    // Set a generic use function

    (*ent).e_UseFunc = use_func;
    /*	if (use_func == useF_health_use)
    {
        G_SoundIndex("sound/player/suithealth.wav");
    }
    else if (use_func == useF_ammo_use )
    {
        G_SoundIndex("sound/player/suitenergy.wav");
    }
    */
    G_SpawnInt(
        "material\0".as_ptr() as *const core::ffi::c_char,
        material,
        &mut (*ent).material as *mut c_int,
    );

    if (*ent).health != 0 {
        (*ent).max_health = (*ent).health;
        (*ent).takedamage = if take_damage { 1 } else { 0 };
        (*ent).e_PainFunc = crate::painF_misc_model_breakable_pain;
        (*ent).e_DieFunc = crate::dieF_misc_model_breakable_die;
    }
}

unsafe fn HealthStationSettings(ent: *mut crate::gentity_t)
{
    G_SpawnInt("count\0".as_ptr() as *const core::ffi::c_char, "0\0".as_ptr() as *const core::ffi::c_char, &mut (*ent).count as *mut c_int);

    if (*ent).count == 0
    {
        match (*g_spskill).integer
        {
        0 =>	{ //	EASY
            (*ent).count = 100;
        }
        1 =>	{ //	MEDIUM
            (*ent).count = 75;
        }
        _ => { // default
        2 =>	{ //	HARD
            (*ent).count = 50;
        }
        }
        }
    }
}


unsafe fn CrystalAmmoSettings(ent: *mut crate::gentity_t)
{
    G_SpawnInt("count\0".as_ptr() as *const core::ffi::c_char, "0\0".as_ptr() as *const core::ffi::c_char, &mut (*ent).count as *mut c_int);

    if (*ent).count == 0
    {
        match (*g_spskill).integer
        {
        0 =>	{ //	EASY
            (*ent).count = 75;
        }
        1 =>	{ //	MEDIUM
            (*ent).count = 75;
        }
        _ => { // default
        2 =>	{ //	HARD
            (*ent).count = 75;
        }
        }
        }
    }
}


//------------------------------------------------------------

//------------------------------------------------------------
/*QUAKED misc_model_ghoul (1 0 0) (-16 -16 -37) (16 16 32)
"model"		arbitrary .glm file to display
"health" - how much health the model has - default 60 (zero makes non-breakable)
*/
//------------------------------------------------------------
// #include "anims.h"

const BOTH_STAND3: c_int = 0; // Placeholder constant
const BOTH_PAIN3: c_int = 1;  // Placeholder constant
const BONE_ANIM_OVERRIDE_FREEZE: c_int = 0x00000020;
const BONE_ANIM_BLEND: c_int = 0x00000040;

static mut temp_animFileIndex: c_int = 0;

unsafe fn set_MiscAnim(ent: *mut crate::gentity_t)
{
    let animations = (*(*level.knownAnimFileSets.as_ptr().add(temp_animFileIndex as usize)).animations);
    if (*ent).playerModel & 1 != 0
    {
        let anim = BOTH_STAND3;
        let animSpeed = 50.0f32 / (*animations.add(anim as usize)).frameLerp;

        // yes, its the same animation, so work out where we are in the leg anim, and blend us
        (*gi.G2API_SetBoneAnim)(
            &mut (*ent).ghoul2[0],
            "model_root\0".as_ptr() as *const core::ffi::c_char,
            (*animations.add(anim as usize)).firstFrame,
            ((*animations.add(anim as usize)).numFrames - 1) + (*animations.add(anim as usize)).firstFrame,
            BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
            animSpeed,
            if cg.time != 0 { cg.time } else { level.time },
            -1,
            350,
        );
    }
    else
    {
        let anim = BOTH_PAIN3;
        let animSpeed = 50.0f32 / (*animations.add(anim as usize)).frameLerp;
        (*gi.G2API_SetBoneAnim)(
            &mut (*ent).ghoul2[0],
            "model_root\0".as_ptr() as *const core::ffi::c_char,
            (*animations.add(anim as usize)).firstFrame,
            ((*animations.add(anim as usize)).numFrames - 1) + (*animations.add(anim as usize)).firstFrame,
            BONE_ANIM_OVERRIDE_FREEZE | BONE_ANIM_BLEND,
            animSpeed,
            if cg.time != 0 { cg.time } else { level.time },
            -1,
            350,
        );
    }
    (*ent).nextthink = level.time + 900;
    (*ent).playerModel += 1;

}

unsafe fn SP_misc_model_ghoul(ent: *mut crate::gentity_t)
{
    // #if 1
    (*ent).s.modelindex = G_ModelIndex((*ent).model);
    (*gi.G2API_InitGhoul2Model)(&mut (*ent).ghoul2, (*ent).model, (*ent).s.modelindex);
    (*ent).s.radius = 50;
    // #else
    // let mut name1: [core::ffi::c_char; 200] = [0; 200];
    // // Initialize with "models/players/kyle/model.glm"
    // let model_name = "models/players/kyle/model.glm\0";
    // for (i, &ch) in model_name.as_bytes().iter().enumerate() {
    //     name1[i] = ch as core::ffi::c_char;
    // }
    // (*ent).s.modelindex = G_ModelIndex(name1.as_ptr());

    // (*gi.G2API_InitGhoul2Model)(&mut (*ent).ghoul2, name1.as_ptr(), (*ent).s.modelindex);
    // (*ent).s.radius = 150;

    // 			// we found the model ok - load it's animation config
    // temp_animFileIndex = G_ParseAnimFileSet("_humanoid\0".as_ptr() as *const core::ffi::c_char, "kyle\0".as_ptr() as *const core::ffi::c_char);
    //  	if temp_animFileIndex < 0 {
    //  		Com_Printf(b"Failed to load animation file set models/players/jedi/animation.cfg\n".as_ptr() as *const core::ffi::c_char);
    //  	}


    // (*ent).s.angles[0] = 0.0;
    // (*ent).s.angles[1] = 90.0;
    // (*ent).s.angles[2] = 0.0;

    // (*ent).s.origin[2] = 20.0;
    // (*ent).s.origin[1] = 80.0;
    // //	(*ent).s.modelScale[0] = (*ent).s.modelScale[1] = (*ent).s.modelScale[2] = 0.8f;

    // VectorSet((*ent).mins.as_mut_ptr(), -16.0, -16.0, -37.0);
    // VectorSet((*ent).maxs.as_mut_ptr(), 16.0, 16.0, 32.0);
    // //#if _DEBUG
    // //loadsavecrash
    // //	VectorCopy((*ent).mins.as_ptr(), (*ent).s.mins.as_mut_ptr());
    // //	VectorCopy((*ent).maxs.as_ptr(), (*ent).s.maxs.as_mut_ptr());
    // //#endif
    // (*ent).contents = 0x00000040; // CONTENTS_BODY
    // (*ent).clipmask = 0x00000200; // MASK_NPCSOLID

    // G_SetOrigin(ent, (*ent).s.origin.as_ptr());
    // VectorCopy((*ent).s.angles.as_ptr(), (*ent).s.apos.trBase.as_mut_ptr());
    // (*ent).health = 1000;

    // //	(*ent).s.modelindex = G_ModelIndex("models/weapons2/blaster_r/g2blaster_w.glm");
    // //	(*gi.G2API_InitGhoul2Model)(&mut (*ent).ghoul2, "models/weapons2/blaster_r/g2blaster_w.glm\0".as_ptr() as *const core::ffi::c_char, (*ent).s.modelindex);
    // //	(*gi.G2API_AddBolt)(&mut (*ent).ghoul2[0], "*weapon\0".as_ptr() as *const core::ffi::c_char);
    // //	(*gi.G2API_AttachG2Model)(&mut (*ent).ghoul2[1], &mut (*ent).ghoul2[0], 0, 0);

    // (*gi.linkentity)(ent);

    // let animations = (*level.knownAnimFileSets.as_ptr().add(temp_animFileIndex as usize).animations);
    // let anim = BOTH_STAND3;
    // let animSpeed = 50.0f32 / (*animations.add(anim as usize)).frameLerp;
    // (*gi.G2API_SetBoneAnim)(&mut (*ent).ghoul2[0], "model_root\0".as_ptr() as *const core::ffi::c_char, (*animations.add(anim as usize)).firstFrame,
    // 					((*animations.add(anim as usize)).numFrames - 1) + (*animations.add(anim as usize)).firstFrame,
    // 					BONE_ANIM_OVERRIDE_FREEZE, animSpeed, cg.time);

    // //	let mut test: c_int = (*gi.G2API_GetSurfaceRenderStatus)(&mut (*ent).ghoul2[0], "l_hand\0".as_ptr() as *const core::ffi::c_char);
    // //	(*gi.G2API_SetSurfaceOnOff)(&mut (*ent).ghoul2[0], "l_arm\0".as_ptr() as *const core::ffi::c_char, 0x00000100);
    // //	test = (*gi.G2API_GetSurfaceRenderStatus)(&mut (*ent).ghoul2[0], "l_hand\0".as_ptr() as *const core::ffi::c_char);

    // //	(*gi.G2API_SetNewOrigin)(&mut (*ent).ghoul2[0], (*gi.G2API_AddBolt)(&mut (*ent).ghoul2[0], "rhang_tag_bone\0".as_ptr() as *const core::ffi::c_char));
    // //	(*ent).s.apos.trDelta[1] = 10.0;
    // //	(*ent).s.apos.trType = TR_LINEAR;


    // (*ent).nextthink = level.time + 1000;
    // (*ent).e_ThinkFunc = thinkF_set_MiscAnim;
    // #endif
}


const RACK_BLASTER: c_int = 1;
const RACK_REPEATER: c_int = 2;
const RACK_ROCKET: c_int = 4;

/*QUAKED misc_model_gun_rack (1 0 0.25) (-14 -14 -4) (14 14 30) BLASTER REPEATER ROCKET
model="models/map_objects/kejim/weaponsrack.md3"

NOTE: can mix and match these spawnflags to get multi-weapon racks.  If only one type is checked the rack will be full of those weapons
BLASTER - Puts one or more blaster guns on the rack.
REPEATER - Puts one or more repeater guns on the rack.
ROCKET - Puts one or more rocket launchers on the rack.
*/

const WP_BLASTER: c_int = 0;          // Placeholder
const WP_REPEATER: c_int = 1;         // Placeholder
const WP_ROCKET_LAUNCHER: c_int = 2;  // Placeholder
const AMMO_BLASTER: c_int = 0;        // Placeholder
const YAW: usize = 1;                 // Placeholder
const FL_DROPPED_ITEM: c_int = 0x00000001;
const FL_FORCE_PULLABLE_ONLY: c_int = 0x00000002;
const IT_WEAPON: c_int = 1;
const IT_AMMO: c_int = 2;

unsafe fn GunRackAddItem(gun: *const crate::gitem_t, org: *const f32, angs: *const f32, ffwd: f32, fright: f32, fup: f32)
{
    let mut fwd: [f32; 3] = [0.0; 3];
    let mut right: [f32; 3] = [0.0; 3];
    let it_ent = G_Spawn();
    let mut rotate = true;

    AngleVectors(angs, fwd.as_mut_ptr(), right.as_mut_ptr(), core::ptr::null_mut());

    if !it_ent.is_null() && !gun.is_null()
    {
        // FIXME: scaling the ammo will probably need to be tweaked to a reasonable amount...adjust as needed
        // Set base ammo per type
        if (*gun).giType == IT_WEAPON
        {
            (*it_ent).spawnflags |= 16;// VERTICAL

            match (*gun).giTag
            {
            WP_BLASTER => {
                (*it_ent).count = 15;
            }
            WP_REPEATER => {
                (*it_ent).count = 100;
            }
            WP_ROCKET_LAUNCHER => {
                (*it_ent).count = 4;
            }
            _ => {}
            }
        }
        else
        {
            rotate = false;

            // must deliberately make it small, or else the objects will spawn inside of each other.
            VectorSet((*it_ent).maxs.as_mut_ptr(), 6.75f32, 6.75f32, 6.75f32);
            VectorScale((*it_ent).maxs.as_ptr(), -1.0f32, (*it_ent).mins.as_mut_ptr());
        }

        (*it_ent).spawnflags |= 1;// ITMSF_SUSPEND
        (*it_ent).classname = G_NewString((*gun).classname);	//copy it so it can be freed safely
        G_SpawnItem(it_ent, gun);

        // FinishSpawningItem handles everything, so clear the thinkFunc that was set in G_SpawnItem
        FinishSpawningItem(it_ent);

        if (*gun).giType == IT_AMMO
        {
            if (*gun).giTag == AMMO_BLASTER { // I guess this just has to use different logic??
                if (*g_spskill).integer >= 2 {
                    (*it_ent).count += 10; // give more on higher difficulty because there will be more/harder enemies?
                }
            }
            else
            {
                // scale ammo based on skill
                match (*g_spskill).integer {
                0 => { // do default
                }
                1 => {
                    (*it_ent).count = ((*it_ent).count as f32 * 0.75f32) as c_int;
                }
                2 => {
                    (*it_ent).count = ((*it_ent).count as f32 * 0.5f32) as c_int;
                }
                _ => {}
                }
            }
        }

        (*it_ent).nextthink = 0;

        VectorCopy(org, (*it_ent).s.origin.as_mut_ptr());
        VectorMA((*it_ent).s.origin.as_ptr(), fright, right.as_ptr(), (*it_ent).s.origin.as_mut_ptr());
        VectorMA((*it_ent).s.origin.as_ptr(), ffwd, fwd.as_ptr(), (*it_ent).s.origin.as_mut_ptr());
        (*it_ent).s.origin[2] += fup;

        VectorCopy(angs, (*it_ent).s.angles.as_mut_ptr());

        // by doing this, we can force the amount of ammo we desire onto the weapon for when it gets picked-up
        (*it_ent).flags |= FL_DROPPED_ITEM | FL_FORCE_PULLABLE_ONLY;
        (*it_ent).physicsBounce = 0.1f32;

        for t in 0..3 {
            if rotate {
                if t == YAW {
                    (*it_ent).s.angles[t] = AngleNormalize180((*it_ent).s.angles[t] + 180.0f32 + crandom() * 14.0f32);
                }
                else {
                    (*it_ent).s.angles[t] = AngleNormalize180((*it_ent).s.angles[t] + crandom() * 4.0f32);
                }
            }
            else {
                if t == YAW {
                    (*it_ent).s.angles[t] = AngleNormalize180((*it_ent).s.angles[t] + 90.0f32 + crandom() * 4.0f32);
                }
            }
        }

        G_SetAngles(it_ent, (*it_ent).s.angles.as_ptr());
        G_SetOrigin(it_ent, (*it_ent).s.origin.as_ptr());
        (*gi.linkentity)(it_ent);
    }
}

//---------------------------------------------
unsafe fn SP_misc_model_gun_rack(ent: *mut crate::gentity_t)
{
    let mut blaster: *mut crate::gitem_t = core::ptr::null_mut();
    let mut repeater: *mut crate::gitem_t = core::ptr::null_mut();
    let mut rocket: *mut crate::gitem_t = core::ptr::null_mut();
    let mut ct: c_int = 0;
    let mut ofz: [f32; 3] = [0.0; 3];
    let mut itemList: [*mut crate::gitem_t; 3] = [core::ptr::null_mut(); 3];

    // If BLASTER is checked...or nothing is checked then we'll do blasters
    if ((*ent).spawnflags & RACK_BLASTER) != 0 || ((*ent).spawnflags & (RACK_BLASTER | RACK_REPEATER | RACK_ROCKET)) == 0
    {
        blaster = FindItemForWeapon(WP_BLASTER);
    }

    if ((*ent).spawnflags & RACK_REPEATER) != 0 {
        repeater = FindItemForWeapon(WP_REPEATER);
    }

    if ((*ent).spawnflags & RACK_ROCKET) != 0 {
        rocket = FindItemForWeapon(WP_ROCKET_LAUNCHER);
    }

    //---------weapon types
    if !blaster.is_null()
    {
        ofz[ct as usize] = 23.0f32;
        itemList[ct as usize] = blaster;
        ct += 1;
    }

    if !repeater.is_null()
    {
        ofz[ct as usize] = 24.5f32;
        itemList[ct as usize] = repeater;
        ct += 1;
    }

    if !rocket.is_null()
    {
        ofz[ct as usize] = 25.5f32;
        itemList[ct as usize] = rocket;
        ct += 1;
    }

    if ct != 0 { //..should always have at least one item on their, but just being safe
        while ct < 3 {
            ofz[ct as usize] = ofz[0];
            itemList[ct as usize] = itemList[0]; // first weapon ALWAYS propagates to fill up the shelf
            ct += 1;
        }
    }

    // now actually add the items to the shelf...validate that we have a list to add
    if ct != 0
    {
        for i in 0..ct {
            GunRackAddItem(itemList[i as usize], (*ent).s.origin.as_ptr(), (*ent).s.angles.as_ptr(), crandom() * 2.0f32, ((i as f32) - 1.0f32) * 9.0f32 + crandom() * 2.0f32, ofz[i as usize]);
        }
    }

    (*ent).s.modelindex = G_ModelIndex("models/map_objects/kejim/weaponsrack.md3\0".as_ptr() as *const core::ffi::c_char);

    G_SetOrigin(ent, (*ent).s.origin.as_ptr());
    G_SetAngles(ent, (*ent).s.angles.as_ptr());

    (*ent).contents = 0x00000001; // CONTENTS_SOLID

    (*gi.linkentity)(ent);
}

const RACK_METAL_BOLTS: c_int = 2;
const RACK_ROCKETS: c_int = 4;
const RACK_WEAPONS: c_int = 8;
const RACK_HEALTH: c_int = 16;
const RACK_PWR_CELL: c_int = 32;
const RACK_NO_FILL: c_int = 64;

/*QUAKED misc_model_ammo_rack (1 0 0.25) (-14 -14 -4) (14 14 30) BLASTER METAL_BOLTS ROCKETS WEAPON HEALTH PWR_CELL NO_FILL
model="models/map_objects/kejim/weaponsrung.md3"

NOTE: can mix and match these spawnflags to get multi-ammo racks.  If only one type is checked the rack will be full of that ammo.  Only three ammo packs max can be displayed.


BLASTER - Adds one or more ammo packs that are compatible with Blasters and the Bryar pistol.
METAL_BOLTS - Adds one or more metal bolt ammo packs that are compatible with the heavy repeater and the flechette gun
ROCKETS - Puts one or more rocket packs on a rack.
WEAPON - adds a weapon matching a selected ammo type to the rack.
HEALTH - adds a health pack to the top shelf of the ammo rack
PWR_CELL - Adds one or more power cell packs that are compatible with the Disuptor, bowcaster, and demp2
NO_FILL - Only puts selected ammo on the rack, it never fills up all three slots if only one or two items were checked
*/

const AMMO_METAL_BOLTS: c_int = 1; // Placeholder
const AMMO_ROCKETS: c_int = 2;     // Placeholder
const AMMO_POWERCELL: c_int = 3;   // Placeholder

//---------------------------------------------
unsafe fn SP_misc_model_ammo_rack(ent: *mut crate::gentity_t)
{
// If BLASTER is checked...or nothing is checked then we'll do blasters
    if ((*ent).spawnflags & RACK_BLASTER) != 0 || ((*ent).spawnflags & (RACK_BLASTER | RACK_METAL_BOLTS | RACK_ROCKETS | RACK_PWR_CELL)) == 0
    {
        if ((*ent).spawnflags & RACK_WEAPONS) != 0
        {
            RegisterItem(FindItemForWeapon(WP_BLASTER));
        }
        RegisterItem(FindItemForAmmo(AMMO_BLASTER));
    }

    if ((*ent).spawnflags & RACK_METAL_BOLTS) != 0 {
        if ((*ent).spawnflags & RACK_WEAPONS) != 0 {
            RegisterItem(FindItemForWeapon(WP_REPEATER));
        }
        RegisterItem(FindItemForAmmo(AMMO_METAL_BOLTS));
    }

    if ((*ent).spawnflags & RACK_ROCKETS) != 0 {
        if ((*ent).spawnflags & RACK_WEAPONS) != 0 {
            RegisterItem(FindItemForWeapon(WP_ROCKET_LAUNCHER));
        }
        RegisterItem(FindItemForAmmo(AMMO_ROCKETS));
    }

    if ((*ent).spawnflags & RACK_PWR_CELL) != 0 {
        RegisterItem(FindItemForAmmo(AMMO_POWERCELL));
    }

    if ((*ent).spawnflags & RACK_HEALTH) != 0 {
        RegisterItem(FindItem("item_medpak_instant\0".as_ptr() as *const core::ffi::c_char));
    }

    (*ent).e_ThinkFunc = crate::thinkF_spawn_rack_goods;
    (*ent).nextthink = level.time + 100;

    G_SetOrigin(ent, (*ent).s.origin.as_ptr());
    G_SetAngles(ent, (*ent).s.angles.as_ptr());

    (*ent).contents = 0x00000100|0x00000010|0x00000008|0x00000004;//CONTENTS_SOLID;//so use traces can go through them

    (*gi.linkentity)(ent);
}

// AMMO RACK!!
unsafe fn spawn_rack_goods(ent: *mut crate::gentity_t)
{
    let mut v_off: f32 = 0.0f32;
    let mut blaster: *mut crate::gitem_t = core::ptr::null_mut();
    let mut metal_bolts: *mut crate::gitem_t = core::ptr::null_mut();
    let mut rockets: *mut crate::gitem_t = core::ptr::null_mut();
    let mut it: *mut crate::gitem_t = core::ptr::null_mut();
    let mut am_blaster: *mut crate::gitem_t = core::ptr::null_mut();
    let mut am_metal_bolts: *mut crate::gitem_t = core::ptr::null_mut();
    let mut am_rockets: *mut crate::gitem_t = core::ptr::null_mut();
    let mut am_pwr_cell: *mut crate::gitem_t = core::ptr::null_mut();
    let mut health: *mut crate::gitem_t = core::ptr::null_mut();
    let mut pos: c_int = 0;
    let mut ct: c_int = 0;
    let mut itemList: [*mut crate::gitem_t; 4] = [core::ptr::null_mut(); 4]; // allocating 4, but we only use 3.  done so I don't have to validate that the array isn't full before I add another

    (*gi.unlinkentity)(ent);

    // If BLASTER is checked...or nothing is checked then we'll do blasters
    if ((*ent).spawnflags & RACK_BLASTER) != 0 || ((*ent).spawnflags & (RACK_BLASTER | RACK_METAL_BOLTS | RACK_ROCKETS | RACK_PWR_CELL)) == 0
    {
        if ((*ent).spawnflags & RACK_WEAPONS) != 0 {
            blaster = FindItemForWeapon(WP_BLASTER);
        }
        am_blaster = FindItemForAmmo(AMMO_BLASTER);
    }

    if ((*ent).spawnflags & RACK_METAL_BOLTS) != 0 {
        if ((*ent).spawnflags & RACK_WEAPONS) != 0 {
            metal_bolts = FindItemForWeapon(WP_REPEATER);
        }
        am_metal_bolts = FindItemForAmmo(AMMO_METAL_BOLTS);
    }

    if ((*ent).spawnflags & RACK_ROCKETS) != 0 {
        if ((*ent).spawnflags & RACK_WEAPONS) != 0 {
            rockets = FindItemForWeapon(WP_ROCKET_LAUNCHER);
        }
        am_rockets = FindItemForAmmo(AMMO_ROCKETS);
    }

    if ((*ent).spawnflags & RACK_PWR_CELL) != 0 {
        am_pwr_cell = FindItemForAmmo(AMMO_POWERCELL);
    }

    if ((*ent).spawnflags & RACK_HEALTH) != 0 {
        health = FindItem("item_medpak_instant\0".as_ptr() as *const core::ffi::c_char);
        RegisterItem(health);
    }

    //---------Ammo types
    if !am_blaster.is_null()
    {
        itemList[ct as usize] = am_blaster;
        ct += 1;
    }

    if !am_metal_bolts.is_null()
    {
        itemList[ct as usize] = am_metal_bolts;
        ct += 1;
    }

    if !am_pwr_cell.is_null()
    {
        itemList[ct as usize] = am_pwr_cell;
        ct += 1;
    }

    if !am_rockets.is_null()
    {
        itemList[ct as usize] = am_rockets;
        ct += 1;
    }

    if ((*ent).spawnflags & RACK_NO_FILL) == 0 && ct != 0 { //double negative..should always have at least one item on there, but just being safe
        while ct < 3 {
            itemList[ct as usize] = itemList[0]; // first item ALWAYS propagates to fill up the shelf
            ct += 1;
        }
    }

    // now actually add the items to the shelf...validate that we have a list to add
    if ct != 0
    {
        for i in 0..ct {
            GunRackAddItem(itemList[i as usize], (*ent).s.origin.as_ptr(), (*ent).s.angles.as_ptr(), crandom() * 0.5f32, ((i as f32) - 1.0f32) * 8.0f32, 7.0f32);
        }
    }

    // -----Weapon option
    if ((*ent).spawnflags & RACK_WEAPONS) != 0 {
        if ((*ent).spawnflags & (RACK_BLASTER | RACK_METAL_BOLTS | RACK_ROCKETS | RACK_PWR_CELL)) == 0 {
            // nothing was selected, so we assume blaster pack
            it = blaster;
        }
        else {
            // if weapon is checked...and so are one or more ammo types, then pick a random weapon to display..always give weaker weapons first
            if !blaster.is_null() {
                it = blaster;
                v_off = 25.5f32;
            }
            else if !metal_bolts.is_null() {
                it = metal_bolts;
                v_off = 27.0f32;
            }
            else if !rockets.is_null() {
                it = rockets;
                v_off = 28.0f32;
            }
        }

        if !it.is_null() {
            // since we may have to put up a health pack on the shelf, we should know where we randomly put
            //	the gun so we don't put the pack on the same spot..so pick either the left or right side
            pos = if random() > 0.5f32 { -1 } else { 1 };

            GunRackAddItem(it, (*ent).s.origin.as_ptr(), (*ent).s.angles.as_ptr(), crandom() * 2.0f32, (random() * 6.0f32 + 4.0f32) * pos as f32, v_off);
        }
    }

    // ------Medpack
    if ((*ent).spawnflags & RACK_HEALTH) != 0 && !health.is_null() {
        if pos == 0 {
            // we haven't picked a side already...
            pos = if random() > 0.5f32 { -1 } else { 1 };
        }
        else {
            // switch to the opposite side
            pos *= -1;
        }

        GunRackAddItem(health, (*ent).s.origin.as_ptr(), (*ent).s.angles.as_ptr(), crandom() * 0.5f32, (random() * 4.0f32 + 4.0f32) * pos as f32, 24.0f32);
    }

    (*ent).s.modelindex = G_ModelIndex("models/map_objects/kejim/weaponsrung.md3\0".as_ptr() as *const core::ffi::c_char);

    G_SetOrigin(ent, (*ent).s.origin.as_ptr());
    G_SetAngles(ent, (*ent).s.angles.as_ptr());

    (*gi.linkentity)(ent);
}

const DROP_MEDPACK: c_int = 1;
const DROP_SHIELDS: c_int = 2;
const DROP_BACTA: c_int = 4;
const DROP_BATTERIES: c_int = 8;

/*QUAKED misc_model_cargo_small (1 0 0.25) (-14 -14 -4) (14 14 30) MEDPACK SHIELDS BACTA BATTERIES
model="models/map_objects/kejim/cargo_small.md3"

  Cargo crate that can only be destroyed by heavy class weapons ( turrets, emplaced guns, at-st )  Can spawn useful things when it breaks

MEDPACK - instant use medpacks
SHIELDS - instant shields
BACTA - bacta tanks
BATTERIES -

"health" - how much damage to take before blowing up ( default 25 )
"splashRadius" - damage range when it explodes ( default 96 )
"splashDamage" - damage to do within explode range ( default 1 )

*/

unsafe fn misc_model_cargo_die(
    self_: *mut crate::gentity_t,
    inflictor: *mut crate::gentity_t,
    attacker: *mut crate::gentity_t,
    damage: c_int,
    mod_: c_int,
    dFlags: c_int,
    hitLoc: c_int,
)
{
    let mut flags: c_int;
    let mut org: [f32; 3] = [0.0; 3];
    let mut temp: [f32; 3] = [0.0; 3];
    let mut health: *mut crate::gitem_t = core::ptr::null_mut();
    let mut shields: *mut crate::gitem_t = core::ptr::null_mut();
    let mut bacta: *mut crate::gitem_t = core::ptr::null_mut();
    let mut batteries: *mut crate::gitem_t = core::ptr::null_mut();

    // copy these for later
    flags = (*self_).spawnflags;
    VectorCopy((*self_).currentOrigin.as_ptr(), org.as_mut_ptr());

    // we already had spawn flags, but we don't care what they were...we just need to set up the flags we want for misc_model_breakable_die
    (*self_).spawnflags = 8; // NO_DMODEL

    // pass through to get the effects and such
    misc_model_breakable_die(self_, inflictor, attacker, damage, mod_);

    // now that the model is broken, we can safely spawn these in it's place without them being in solid
    temp[2] = org[2] + 16.0f32;

    // annoying, but spawn each thing in its own little quadrant so that they don't end up on top of each other
    if (flags & DROP_MEDPACK) != 0 {
        health = FindItem("item_medpak_instant\0".as_ptr() as *const core::ffi::c_char);

        if !health.is_null() {
            temp[0] = org[0] + crandom() * 8.0f32 + 16.0f32;
            temp[1] = org[1] + crandom() * 8.0f32 + 16.0f32;

            LaunchItem(health, temp.as_ptr(), [0.0f32; 3].as_ptr(), core::ptr::null_mut());
        }
    }
    if (flags & DROP_SHIELDS) != 0 {
        shields = FindItem("item_shield_sm_instant\0".as_ptr() as *const core::ffi::c_char);

        if !shields.is_null() {
            temp[0] = org[0] + crandom() * 8.0f32 - 16.0f32;
            temp[1] = org[1] + crandom() * 8.0f32 + 16.0f32;

            LaunchItem(shields, temp.as_ptr(), [0.0f32; 3].as_ptr(), core::ptr::null_mut());
        }
    }

    if (flags & DROP_BACTA) != 0 {
        bacta = FindItem("item_bacta\0".as_ptr() as *const core::ffi::c_char);

        if !bacta.is_null() {
            temp[0] = org[0] + crandom() * 8.0f32 - 16.0f32;
            temp[1] = org[1] + crandom() * 8.0f32 - 16.0f32;

            LaunchItem(bacta, temp.as_ptr(), [0.0f32; 3].as_ptr(), core::ptr::null_mut());
        }
    }

    if (flags & DROP_BATTERIES) != 0 {
        batteries = FindItem("item_battery\0".as_ptr() as *const core::ffi::c_char);

        if !batteries.is_null() {
            temp[0] = org[0] + crandom() * 8.0f32 + 16.0f32;
            temp[1] = org[1] + crandom() * 8.0f32 - 16.0f32;

            LaunchItem(batteries, temp.as_ptr(), [0.0f32; 3].as_ptr(), core::ptr::null_mut());
        }
    }
}

//---------------------------------------------
unsafe fn SP_misc_model_cargo_small(ent: *mut crate::gentity_t)
{
    G_SpawnInt("splashRadius\0".as_ptr() as *const core::ffi::c_char, "96\0".as_ptr() as *const core::ffi::c_char, &mut (*ent).splashRadius as *mut c_int);
    G_SpawnInt("splashDamage\0".as_ptr() as *const core::ffi::c_char, "1\0".as_ptr() as *const core::ffi::c_char, &mut (*ent).splashDamage as *mut c_int);

    if ((*ent).spawnflags & DROP_MEDPACK) != 0 {
        RegisterItem(FindItem("item_medpak_instant\0".as_ptr() as *const core::ffi::c_char));
    }

    if ((*ent).spawnflags & DROP_SHIELDS) != 0 {
        RegisterItem(FindItem("item_shield_sm_instant\0".as_ptr() as *const core::ffi::c_char));
    }

    if ((*ent).spawnflags & DROP_BACTA) != 0 {
        // RegisterItem(FindItem("item_bacta\0".as_ptr() as *const core::ffi::c_char));
    }

    if ((*ent).spawnflags & DROP_BATTERIES) != 0 {
        RegisterItem(FindItem("item_battery\0".as_ptr() as *const core::ffi::c_char));
    }

    G_SpawnInt("health\0".as_ptr() as *const core::ffi::c_char, "25\0".as_ptr() as *const core::ffi::c_char, &mut (*ent).health as *mut c_int);

    SetMiscModelDefaults(ent, crate::useF_NULL, "11\0".as_ptr() as *const core::ffi::c_char, 0x00000001|0x00000080|0x00000040|0x00000008|0x00000004, 0, true, false);
    (*ent).s.modelindex2 = G_ModelIndex("/models/map_objects/kejim/cargo_small.md3\0".as_ptr() as *const core::ffi::c_char);	// Precache model

    // we only take damage from a heavy weapon class missile
    (*ent).flags |= 0x00008000; // FL_DMG_BY_HEAVY_WEAP_ONLY

    (*ent).e_DieFunc = crate::dieF_misc_model_cargo_die;

    (*ent).radius = 1.5f32; // scale number of chunks spawned
}
