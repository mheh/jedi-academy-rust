// Filename: g_savegame.cpp
//
// leave this line at the top for all g_xxxx.cpp files...

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::mem;
use std::ptr;

// Forward declarations for external types and functions
// (These would normally come from g_headers.h, IcarusInterface.h, Q3_Interface.h, etc.)

extern "C" {
    fn OBJ_LoadTacticalInfo();
    fn G_LoadSave_WriteMiscData();
    fn G_LoadSave_ReadMiscData();
    fn G_ReloadSaberData(ent: *mut gentity_t);
    fn FX_Read();
    fn FX_Write();

    fn G_NewString(string: *const c_char) -> *mut c_char;
    fn G_Alloc(size: usize) -> *mut c_void;
    fn G_Error(msg: *const c_char);
    fn G_FreeEntity(ent: *mut gentity_t);

    fn OBJ_SaveObjectiveData();
    fn OBJ_LoadObjectiveData();

    fn TIMER_Save();
    fn TIMER_Load();

    fn WriteInUseBits();
    fn ReadInUseBits();

    fn VALIDSTRING(string: *const c_char) -> c_int;
    fn CAS_GetBModelSound(soundSet: *const c_char, mid: c_int) -> c_int;
    fn WP_SaberSetDefaults(saber: *mut saberInfo_t, setColors: c_int);

    fn va(fmt: *const c_char) -> *const c_char;
    fn BigLong(l: c_int) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn strlen(s: *const c_char) -> usize;

    // External game types - these are opaque in this translation
    type gentity_t;
    type gclient_t;
    type gitem_t;
    type AIGroupInfo_t;
    type vehicleInfo_t;
    type level_locals_t;
    type game_globals_t;
    type entityState_t;
    type game_import_t;
    type parms_t;
    type Vehicle_t;
    type saberInfoRetail_t;
    type saberInfo_t;
    type alertEvent_t;
    type animFileSet_t;
    type gNPC_t;
}

// External global variables
extern "C" {
    static mut g_entities: *mut gentity_t;
    static mut level: level_locals_t;
    static mut globals: game_globals_t;
    static mut bg_itemlist: *mut gitem_t;
    static mut bg_numItems: c_int;
    static mut g_vehicleInfo: *mut vehicleInfo_t;
    static mut numVehicles: c_int;

    static mut gi: game_import_t;
    static mut killPlayerTimer: c_int;
    static mut in_camera: c_int;
}

// ============================================================================
// Type definitions
// ============================================================================

#[repr(C)]
pub struct save_field_t {
    psName: *const c_char,
    iOffset: usize,
    eFieldType: c_int,
}

// Field type constants (from fields.h, etc.)
const F_STRING: c_int = 1;
const F_GCLIENT: c_int = 2;
const F_GENTITY: c_int = 3;
const F_ITEM: c_int = 4;
const F_BOOLPTR: c_int = 5;
const F_NULL: c_int = 6;
const F_IGNORE: c_int = 7;
const F_BEHAVIORSET: c_int = 8;
const F_ALERTEVENT: c_int = 9;
const F_AIGROUPS: c_int = 10;
const F_ANIMFILESETS: c_int = 11;
const F_GROUP: c_int = 12;
const F_VEHINFO: c_int = 13;

// Entity and game constants
const MAX_GENTITIES: c_int = 1024;
const MAX_CLIENTS: c_int = 32;
const MAX_ALERT_EVENTS: c_int = 256;
const MAX_FRAME_GROUPS: c_int = 8;
const MAX_ANIM_FILES: c_int = 32;
const MAX_ANIM_EVENTS: c_int = 32;
const NUM_BSETS: c_int = 4;
const TAG_G_ALLOC: c_int = 0x67616c6c; // 'gall'
const TAG_TEMP_WORKSPACE: c_int = 0x74656d70; // 'temp'
const MAX_MISSION_OBJ: c_int = 32;
const ET_MOVER: c_int = 5;

// Offset constants (placeholder - actual offsets would be computed via offsetof or metadata)
const LEVEL_LOCALS_T_SAVESTOP: usize = 0;

// sstring_t type (from qcommon/sstring.h)
type sstring_t = *const c_char;

// ============================================================================
// Global state
// ============================================================================

static mut strList: *mut Vec<sstring_t> = ptr::null_mut();

// ============================================================================
// String handling
// ============================================================================

/////////// char * /////////////
//
//
// GetStringNum - convert a string pointer to a length value for storage
fn GetStringNum(psString: *const c_char) -> c_int {
    unsafe {
        assert_ne!(psString as usize, 0xcdcdcdcd);

        // NULL ptrs I'll write out as a strlen of -1...
        //
        if psString.is_null() {
            return -1;
        }

        (*strList).push(psString);
        (strlen(psString) + 1) as c_int    // this gives us the chunk length for the reader later
    }
}

// GetStringPtr - convert a length value back to a string pointer
fn GetStringPtr(iStrlen: c_int, psOriginal: *mut c_char) -> *mut c_char {
    unsafe {
        if iStrlen != -1 {
            let mut sString: [c_char; 768] = [0; 768];  // arb, inc if nec.

            sString[0] = 0;

            assert!((iStrlen as usize + 1) <= sString.len());

            // NOTE: gi.ReadFromSaveGame call - actual signature needs extern declaration
            // gi.ReadFromSaveGame('STRG', sString.as_mut_ptr() as *mut c_void, iStrlen as usize);

            #[cfg(not(target_os = "xbox"))]  // TAG_G_ALLOC is always blown away, we can never recycle
            {
                // if (psOriginal && gi.bIsFromZone(psOriginal, TAG_G_ALLOC)) {
                //     if (!strcmp(psOriginal,sString))
                //     {//it's a legal ptr and they're the same so let's just reuse it instead of free/alloc
                //         return psOriginal;
                //     }
                //     gi.Free(psOriginal);
                // }
            }

            return G_NewString(sString.as_ptr());
        }

        ptr::null_mut()
    }
}
//
//
////////////////////////////////

/////////// gentity_t * ////////
//
//
// GetGEntityNum - convert entity pointer to index
fn GetGEntityNum(ent: *mut gentity_t) -> c_int {
    unsafe {
        assert_ne!(ent as usize, 0xcdcdcdcd);

        if ent.is_null() {
            return -1;
        }

        // note that I now validate the return value (to avoid triggering asserts on re-load) because of the
        //  way that the level_locals_t alertEvents struct contains a count of which ones are valid, so I'm guessing
        //  that some of them aren't (valid)...
        //
        let iReturnIndex = ent as isize - g_entities as isize;

        if iReturnIndex < 0 || iReturnIndex >= MAX_GENTITIES as isize {
            -1  // will get a NULL ptr on reload
        } else {
            iReturnIndex as c_int
        }
    }
}

// GetGEntityPtr - convert index back to entity pointer
fn GetGEntityPtr(iEntNum: c_int) -> *mut gentity_t {
    unsafe {
        if iEntNum == -1 {
            return ptr::null_mut();
        }
        assert!(iEntNum >= 0);
        assert!(iEntNum < MAX_GENTITIES);
        (g_entities as *mut u8).add(iEntNum as usize) as *mut gentity_t
    }
}
//
//
////////////////////////////////

// GetGroupNumber - convert group pointer to index
fn GetGroupNumber(pGroup: *mut AIGroupInfo_t) -> c_int {
    unsafe {
        assert_ne!(pGroup as usize, 0xcdcdcdcd);

        if pGroup.is_null() {
            return -1;
        }

        // NOTE: Placeholder - actual calculation depends on level.groups array bounds
        let iReturnIndex = pGroup as isize - pGroup as isize; // Simplified for safety
        if iReturnIndex < 0 || iReturnIndex >= MAX_FRAME_GROUPS as isize {
            -1  // will get a NULL ptr on reload
        } else {
            iReturnIndex as c_int
        }
    }
}

// GetGroupPtr - convert index back to group pointer
fn GetGroupPtr(iGroupNum: c_int) -> *mut AIGroupInfo_t {
    unsafe {
        if iGroupNum == -1 {
            return ptr::null_mut();
        }
        assert!(iGroupNum >= 0);
        assert!(iGroupNum < MAX_FRAME_GROUPS);
        // NOTE: Placeholder - actual array pointer resolution needed
        ptr::null_mut()
    }
}

/////////// gclient_t * ////////
//
//
// GetGClientNum - convert client pointer to index or special value
fn GetGClientNum(c: *mut gclient_t, ent: *mut gentity_t) -> c_int {
    unsafe {
        // unfortunately, I now need to see if this is a 'real' client (and therefore resolve to an enum), or
        //  whether it's one of the NPC or SP_misc_weapon_shooter
        //
        assert_ne!(c as usize, 0xcdcdcdcd);

        if c.is_null() {
            return -1;
        }

        // NOTE: Placeholder - actual access to (*ent).s.number needs proper struct layout
        // if (*ent).s.number < MAX_CLIENTS {
        //     // regular client...
        //     (c as isize - level.clients as isize) as c_int
        // } else {
        //     // this must be an NPC or weapon_shooter, so mark it as special...
        //     -2  // yeuch, but distinguishes it from a valid 0 index, or -1 for client==NULL
        // }

        // Fallback: assume NPC/weapon_shooter
        -2
    }
}

// GetGClientPtr - convert index back to client pointer or special value
fn GetGClientPtr(c: c_int) -> *mut gclient_t {
    unsafe {
        if c == -1 {
            return ptr::null_mut();
        }
        if c == -2 {
            return c as *mut gclient_t;  // preserve this value so that I know to load in one of Mike's private NPC clients later
        }

        assert!(c >= 0);
        // assert!((c as usize) < level.maxclients as usize);  // Placeholder - level.maxclients access
        (c as *mut gclient_t)
    }
}
//
//
////////////////////////////////

/////////// gitem_t * //////////
//
//
// GetGItemNum - convert item pointer to index
fn GetGItemNum(pItem: *mut gitem_t) -> c_int {
    unsafe {
        assert_ne!(pItem as usize, 0xcdcdcdcd);

        if pItem.is_null() {
            return -1;
        }

        ((pItem as isize - bg_itemlist as isize)) as c_int
    }
}

// GetGItemPtr - convert index back to item pointer
fn GetGItemPtr(iItem: c_int) -> *mut gitem_t {
    unsafe {
        if iItem == -1 {
            return ptr::null_mut();
        }

        assert!(iItem >= 0);
        assert!(iItem < bg_numItems);
        bg_itemlist.add(iItem as usize)
    }
}
//
//
////////////////////////////////

/////////// vehicleInfo_t * //////////
//
//
// GetVehicleInfoNum - convert vehicle info pointer to index
fn GetVehicleInfoNum(pVehicleInfo: *mut vehicleInfo_t) -> c_int {
    unsafe {
        assert_ne!(pVehicleInfo as usize, 0xcdcdcdcd);

        if pVehicleInfo.is_null() {
            return -1;
        }

        ((pVehicleInfo as isize - g_vehicleInfo as isize)) as c_int
    }
}

// GetVehicleInfoPtr - convert index back to vehicle info pointer
fn GetVehicleInfoPtr(iVehicleIndex: c_int) -> *mut vehicleInfo_t {
    unsafe {
        if iVehicleIndex == -1 {
            return ptr::null_mut();
        }

        assert!(iVehicleIndex > 0);
        assert!(iVehicleIndex < numVehicles);
        g_vehicleInfo.add(iVehicleIndex as usize)
    }
}
//
//
////////////////////////////////

// EnumerateField - convert pointers to indices for saving
fn EnumerateField(pField: *const save_field_t, pbBase: *const u8) {
    unsafe {
        let pField = &*pField;
        let pv = (pbBase as *mut c_void).add(pField.iOffset);

        match pField.eFieldType {
        F_STRING => {
            *(pv as *mut c_int) = GetStringNum(*(pv as *const *const c_char));
        },

        F_GENTITY => {
            *(pv as *mut c_int) = GetGEntityNum(*(pv as *const *mut gentity_t));
        },

        F_GROUP => {
            *(pv as *mut c_int) = GetGroupNumber(*(pv as *const *mut AIGroupInfo_t));
        },

        F_GCLIENT => {
            *(pv as *mut c_int) = GetGClientNum(*(pv as *const *mut gclient_t), pbBase as *mut gentity_t);
        },

        F_ITEM => {
            *(pv as *mut c_int) = GetGItemNum(*(pv as *const *mut gitem_t));
        },

        F_VEHINFO => {
            *(pv as *mut c_int) = GetVehicleInfoNum(*(pv as *const *mut vehicleInfo_t));
        },

        F_BEHAVIORSET => {
            let p = pv as *const *const c_char;
            for i in 0..NUM_BSETS as usize {
                let pv_i = (p as *mut c_void).add(i * mem::size_of::<*const c_char>());
                *(pv_i as *mut c_int) = GetStringNum(*(pv_i as *const *const c_char));
            }
        },

        F_ALERTEVENT => {
            // convert all gentity_t ptrs in an alertEvent array into indexes...
            let p = pv as *mut alertEvent_t;

            for i in 0..MAX_ALERT_EVENTS as usize {
                // NOTE: Placeholder - alertEvent_t.owner field access needs proper struct layout
                // (*p.add(i)).owner = GetGEntityNum((*p.add(i)).owner as *mut gentity_t) as *mut gentity_t;
            }
        },

        F_AIGROUPS => {
            // convert to ptrs within this into indexes...
            let p = pv as *mut AIGroupInfo_t;

            for i in 0..MAX_FRAME_GROUPS as usize {
                // NOTE: Placeholder - AIGroupInfo_t field access needs proper struct layout
                // (*p.add(i)).enemy = GetGEntityNum((*p.add(i)).enemy as *mut gentity_t) as *mut gentity_t;
                // (*p.add(i)).commander = GetGEntityNum((*p.add(i)).commander as *mut gentity_t) as *mut gentity_t;
            }
        },

        F_ANIMFILESETS => {
            let p = pv as *mut animFileSet_t;

            for i in 0..MAX_ANIM_FILES as usize {
                for j in 0..MAX_ANIM_EVENTS as usize {
                    // NOTE: Placeholder - animFileSet_t field access needs proper struct layout
                    // let ptAnimEventStringData = (*p.add(i)).torsoAnimEvents[j].stringData as *const c_char;
                    // *((&mut (*p.add(i)).torsoAnimEvents[j].stringData) as *mut _ as *mut c_int) = GetStringNum(ptAnimEventStringData);
                }
            }
        },

        F_BOOLPTR => {
            *(pv as *mut c_int) = if *(pv as *const c_int) != 0 { 1 } else { 0 };
        },

        // These are pointers that are always recreated
        F_NULL => {
            *(pv as *mut *mut c_void) = ptr::null_mut();
        },

        F_IGNORE => {
        },

        _ => {
            G_Error(b"EnumerateField: unknown field type\0".as_ptr() as *const c_char);
        },
        }
    }
}

// ============================================================================
// Save field definitions
// ============================================================================

// static const save_field_t savefields_gEntity[]
const SAVEFIELDS_G_ENTITY: &[save_field_t] = &[
    save_field_t { psName: b"client\0".as_ptr() as *const c_char, iOffset: 0, eFieldType: F_GCLIENT },
    save_field_t { psName: b"owner\0".as_ptr() as *const c_char, iOffset: 0, eFieldType: F_GENTITY },
    save_field_t { psName: b"classname\0".as_ptr() as *const c_char, iOffset: 0, eFieldType: F_STRING },
    // ... (many more fields, offset values would be filled in via metadata/offsets)
    // {strFOFS(model3), F_STRING}, - MCG
    save_field_t { psName: ptr::null(), iOffset: 0, eFieldType: F_IGNORE },
];

// static const save_field_t savefields_gNPC[]
// {strNPCOFS(touchedByPlayer), F_GENTITY},
// ... etc
// {NULL, 0, F_IGNORE}

// static const save_field_t savefields_LevelLocals[]
// {strLLOFS(locationHead), F_GENTITY},
// {strLLOFS(alertEvents), F_ALERTEVENT},
// {strLLOFS(groups), F_AIGROUPS},
// {strLLOFS(knownAnimFileSets), F_ANIMFILESETS},
// {NULL, 0, F_IGNORE}

// static const save_field_t savefields_gVHIC[]
// {strVHICOFS(m_pPilot), F_GENTITY},
// {strVHICOFS(m_pOldPilot), F_GENTITY},
// {strVHICOFS(m_pDroidUnit), F_GENTITY},
// {strVHICOFS(m_pParentEntity), F_GENTITY},
// {strVHICOFS(m_pVehicleInfo), F_VEHINFO},
// {NULL, 0, F_IGNORE}

// static const save_field_t savefields_gClient[]
// {strCLOFS(ps.saber[0].name), F_STRING},
// {strCLOFS(ps.saber[1].name), F_STRING},
// {strCLOFS(leader), F_GENTITY},
// {strCLOFS(clientInfo.customBasicSoundDir), F_STRING},
// {strCLOFS(clientInfo.customCombatSoundDir), F_STRING},
// {strCLOFS(clientInfo.customExtraSoundDir), F_STRING},
// {strCLOFS(clientInfo.customJediSoundDir), F_STRING},
// {NULL, 0, F_IGNORE}

// ============================================================================
// EnumerateFields - convert all fields and save to archive
// ============================================================================

fn EnumerateFields(pFields: *const save_field_t, pbData: *mut u8, ulChid: c_int, iLen: usize) {
    unsafe {
        strList = Box::into_raw(Box::new(Vec::new()));

        // enumerate all the fields...
        //
        if !pFields.is_null() {
            let mut pField = pFields;
            while !(*pField).psName.is_null() {
                assert!((*pField).iOffset < iLen);
                EnumerateField(pField, pbData);
                pField = pField.add(1);
            }
        }

        // save out raw data...
        //
        // gi.AppendToSaveGame(ulChid, pbData, iLen);

        // save out any associated strings..
        //
        // list<sstring_t>::iterator it = strList->begin();
        // for (unsigned int i=0; i<strList->size(); i++, ++it)
        // {
        //     gi.AppendToSaveGame('STRG', (void *)(*it).c_str(), (*it).length() + 1);
        // }

        let _ = Box::from_raw(strList);
        strList = ptr::null_mut();
    }
}

// ============================================================================
// EvaluateField - convert indices back to pointers during load
// ============================================================================

fn EvaluateField(pField: *const save_field_t, pbBase: *mut u8, pbOriginalRefData: *mut u8) {
    unsafe {
        let pField = &*pField;
        let pv = (pbBase as *mut c_void).add(pField.iOffset);
        let pvOriginal = if !pbOriginalRefData.is_null() {
            (pbOriginalRefData as *mut c_void).add(pField.iOffset)
        } else {
            ptr::null_mut()
        };

        match pField.eFieldType {
        F_STRING => {
            let psOriginal = if !pbOriginalRefData.is_null() {
                *(pvOriginal as *const *mut c_char)
            } else {
                ptr::null_mut()
            };
            *(pv as *mut *mut c_char) = GetStringPtr(*(pv as *const c_int), psOriginal);
        },

        F_GENTITY => {
            *(pv as *mut *mut gentity_t) = GetGEntityPtr(*(pv as *const c_int));
        },

        F_GROUP => {
            *(pv as *mut *mut AIGroupInfo_t) = GetGroupPtr(*(pv as *const c_int));
        },

        F_GCLIENT => {
            *(pv as *mut *mut gclient_t) = GetGClientPtr(*(pv as *const c_int));
        },

        F_ITEM => {
            *(pv as *mut *mut gitem_t) = GetGItemPtr(*(pv as *const c_int));
        },

        F_VEHINFO => {
            *(pv as *mut *mut vehicleInfo_t) = GetVehicleInfoPtr(*(pv as *const c_int));
        },

        F_BEHAVIORSET => {
            let p = pv as *mut *mut c_char;
            let pO = if !pbOriginalRefData.is_null() {
                pvOriginal as *mut *mut c_char
            } else {
                ptr::null_mut()
            };
            for i in 0..NUM_BSETS as usize {
                let psOriginal = if !pbOriginalRefData.is_null() {
                    *pO.add(i)
                } else {
                    ptr::null_mut()
                };
                *p.add(i) = GetStringPtr(*p.add(i) as *const c_int, psOriginal);
            }
        },

        F_ALERTEVENT => {
            // NOTE: Placeholder - complex struct field access
            // alertEvent_t* p = (alertEvent_t *) pv;
            // for (int i=0; i<MAX_ALERT_EVENTS; i++)
            // {
            //     p[i].owner = GetGEntityPtr((int)(p[i].owner));
            // }
        },

        F_AIGROUPS => {
            // convert to ptrs within this into indexes...
            // NOTE: Placeholder - complex struct field access
            // AIGroupInfo_t* p = (AIGroupInfo_t *) pv;
            // for (int i=0; i<MAX_FRAME_GROUPS; i++)
            // {
            //     p[i].enemy = GetGEntityPtr((int)(p[i].enemy));
            //     p[i].commander = GetGEntityPtr((int)(p[i].commander));
            // }
        },

        F_ANIMFILESETS => {
            // NOTE: Placeholder - complex struct field access
            // animFileSet_t* p = (animFileSet_t *) pv;
            // char *pO;
            // for (int i=0; i<MAX_ANIM_FILES; i++)
            // {
            //     for ( int j=0; j<MAX_ANIM_EVENTS; j++ )
            //     {
            //         pO = pbOriginalRefData ? level.knownAnimFileSets[i].torsoAnimEvents[j].stringData : NULL;
            //         p[i].torsoAnimEvents[j].stringData = GetStringPtr((int)p[i].torsoAnimEvents[j].stringData, pO);
            //         pO = pbOriginalRefData ? level.knownAnimFileSets[i].legsAnimEvents[j].stringData : NULL;
            //         p[i].legsAnimEvents[j].stringData = GetStringPtr((int)p[i].legsAnimEvents[j].stringData, pO);
            //     }
            // }
        },

        // These fields are patched in when their relevant owners are loaded
        F_BOOLPTR | F_NULL => {
        },

        F_IGNORE => {
        },

        _ => {
            G_Error(b"EvaluateField: unknown field type\0".as_ptr() as *const c_char);
        },
        }
    }
}

// ============================================================================
// Savegame format utilities
// ============================================================================

// copy of function in sv_savegame
fn SG_GetChidText(chid: c_int) -> [c_char; 5] {
    unsafe {
        let mut chidtext: [c_char; 5] = [0; 5];
        let biglong_val = BigLong(chid);
        memcpy(chidtext.as_mut_ptr() as *mut c_void,
               &biglong_val as *const _ as *const c_void,
               mem::size_of::<c_int>());
        chidtext[4] = 0;
        chidtext
    }
}

// ============================================================================
// High-level savegame functions
// ============================================================================

// WriteLevelLocals - save level locals to savegame
// ==============
// All pointer variables (except function pointers) must be handled specially.
// ==============
fn WriteLevelLocals() {
    unsafe {
        // NOTE: Placeholder - requires proper level_locals_t definition and gi interface
        // level_locals_t *temp = (level_locals_t *)gi.Malloc(sizeof(level_locals_t), TAG_TEMP_WORKSPACE, qfalse);
        // *temp = level;  // copy out all data into a temp space
        // EnumerateFields(savefields_LevelLocals, (byte *)temp, 'LVLC', LLOFS(LEVEL_LOCALS_T_SAVESTOP));
        // gi.Free(temp);
    }
}

// ReadLevelLocals - load level locals from savegame
// ==============
// All pointer variables (except function pointers) must be handled specially.
// ==============
fn ReadLevelLocals() {
    unsafe {
        // NOTE: Placeholder - requires proper level_locals_t definition and gi interface
        // preserve client ptr either side of the load, because clients are already saved/loaded through Read/Writegame...
        // gclient_t *pClients = level.clients;  // save clients
        // level_locals_t *temp = (level_locals_t *)gi.Malloc(sizeof(level_locals_t), TAG_TEMP_WORKSPACE, qfalse);
        // *temp = level;  // struct copy
        // EvaluateFields(savefields_LevelLocals, (byte *)temp, (byte *)&level, 'LVLC', LLOFS(LEVEL_LOCALS_T_SAVESTOP),qfalse);
        // level = *temp;  // struct copy
        // level.clients = pClients;  // restore clients
        // gi.Free(temp);
    }
}

// Placeholder for major functions - actual implementations would require
// complete struct layouts and gi interface bindings

pub fn WriteLevel(_qbAutosave: c_int) {
    unsafe {
        // if (!qbAutosave) {
        //     write out one client - us!
        //     assert(level.maxclients == 1);
        //     gclient_t client = level.clients[0];
        //     EnumerateFields(savefields_gClient, (byte *)&client, 'GCLI', sizeof(client));
        //     WriteLevelLocals();
        // }
        // OBJ_SaveObjectiveData();
        // FX_Write();
        // WriteGEntities(qbAutosave);
        // Quake3Game()->VariableSave();
        // G_LoadSave_WriteMiscData();
        // CG_WriteTheEvilCGHackStuff();
        // static int iDONE = 1234;
        // gi.AppendToSaveGame('DONE', &iDONE, sizeof(iDONE));
    }
}

pub fn ReadLevel(_qbAutosave: c_int, _qbLoadTransition: c_int) {
    unsafe {
        // if ( qbLoadTransition ) {
        //     // Load transition logic here
        //     // ...
        // }
        // else {
        //     // Normal load logic
        //     // ...
        // }
    }
}

pub fn GameAllowedToSaveHere() -> c_int {
    unsafe {
        if in_camera == 0 && killPlayerTimer == 0 {
            1
        } else {
            0
        }
    }
}

//////////////////// eof /////////////////////
