/* ***************************************************************************************
 *
 * RM_Objective.cpp
 *
 * Implements the CRMObjective class.  This class is reponsible for parsing an objective
 * from the mission file as well as linking the objective into the world.
 *
 ************************************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char};

// #include "../server/exe_headers.h"
use crate::code::server::exe_headers_h::*;
// #include "rm_headers.h"  — brings in CRMObjective (via RM_Objective.h) and CGPGroup (via genericparser2.h)
use crate::code::RMG::rm_headers_h::*;

// atoi is a C stdlib function (not a module); declare it directly
extern "C" {
    fn atoi(nptr: *const c_char) -> c_int;
}

impl CRMObjective {
    /* ***************************************************************************************
     * CRMObjective::CRMObjective
     *	Constructs a random mission objective and fills in the default properties
     *
     * inputs:
     *  none
     *
     * return:
     *	none
     *
     ************************************************************************************************/
    pub unsafe fn new(group: *mut CGPGroup) -> Self {
        let mut obj: Self = core::mem::zeroed();
        obj.SetPriority(atoi((*group).FindPairValue(
            b"priority\0".as_ptr() as *const c_char,
            b"0\0".as_ptr() as *const c_char,
        )));
        // Capture priority once to avoid overlapping borrows in the va() argument expressions
        let prio = obj.GetPriority();
        obj.SetMessage((*group).FindPairValue(
            b"message\0".as_ptr() as *const c_char,
            va(b"Objective %i Completed\0".as_ptr() as *const c_char, prio),
        ));
        obj.SetDescription((*group).FindPairValue(
            b"description\0".as_ptr() as *const c_char,
            va(b"Objective %i\0".as_ptr() as *const c_char, prio),
        ));
        obj.SetInfo((*group).FindPairValue(
            b"info\0".as_ptr() as *const c_char,
            va(b"Info %i\0".as_ptr() as *const c_char, prio),
        ));
        obj.SetTrigger((*group).FindPairValue(
            b"trigger\0".as_ptr() as *const c_char,
            b"\0".as_ptr() as *const c_char,
        ));
        obj.SetName((*group).GetName());

    /*	const char * soundPath = group->FindPairValue("completed_sound", "" );
        if (soundPath)
            mCompleteSoundID = G_SoundIndex(soundPath);
    */

        obj.mCompleted  = false;
        obj.mOrderIndex = -1;

        // If no priority was specified for this objective then its active by default.
        if obj.GetPriority() != 0 {
            obj.mActive = false;
        } else {
            obj.mActive = true;
        }

        obj
    }

    /* ***************************************************************************************
     * CRMObjective::FindRandomTrigger
     *	Searches the entitySystem form a random arioche trigger that matches the objective name
     *
     * inputs:
     *  none
     *
     * return:
     *	trigger: a random trigger or NULL if one couldnt be found
     *
     ************************************************************************************************/
    /*CTriggerAriocheObjective* CRMObjective::FindRandomTrigger ( )
    {
        CEntity*	search;
        CEntity*	triggers[20];
        int			numTriggers;

        // Start off the first trigger
        numTriggers = 0;
        search      = entitySystem->GetEntityFromClassname ( NULL, "trigger_arioche_objective" );

        // Make a list of triggers
        while ( numTriggers < 20 && search )
        {
            CTriggerAriocheObjective* trigger = (CTriggerAriocheObjective*) search;

            // Move on to the next trigger
            search = entitySystem->GetEntityFromClassname ( search, "trigger_arioche_objective" );

            // See if this trigger is already in use
            if ( trigger->GetObjective ( ) )
            {
                continue;
            }

            // If the objective names dont match then ignore this trigger
            if ( stricmp ( trigger->GetObjectiveName ( ), GetTrigger() ) )
            {
                continue;
            }

            // Add the trigger to the list
            triggers[numTriggers++] = trigger;
        }

        // If no matching triggers then just return NULL
        if ( 0 == numTriggers )
        {
            return NULL;
        }

        // Return a random choice from the trigger list
        return (CTriggerAriocheObjective*)triggers[TheRandomMissionManager->GetLandScape()->irand(0,numTriggers-1)];
    }
    */

    /* ***************************************************************************************
     * CRMObjective::Link
     *	Links the objective into the world using the current state of the world to determine
     *  where it should link
     *
     * inputs:
     *  none
     *
     * return:
     *	true: objective successfully linked
     *  false: objective failed to link
     *
     ************************************************************************************************/
    pub unsafe fn Link(&mut self) -> bool {
    /*	CTriggerAriocheObjective* trigger;

        // Look for a random trigger to associate this objective to.
        trigger = FindRandomTrigger ( );
        if ( NULL != trigger )
        {
            trigger->SetObjective ( this );
        }
    */
        true
    }
}
