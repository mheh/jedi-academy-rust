//Anything above this #include will be ignored by the compiler
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use crate::codemp::qcommon::exe_headers_h::*;

/************************************************************************************************
 *
 * RM_Objective.cpp
 *
 * Implements the CRMObjective class.  This class is reponsible for parsing an objective
 * from the mission file as well as linking the objective into the world.
 *
 ************************************************************************************************/

use crate::codemp::RMG::RM_Headers_h::*;
// port note: CRMObjective is this file's paired class (defined in RM_Objective.h);
// imported from its header module per triage.  CGPGroup resolves through the same glob
// (it is declared in RM_Objective_h.rs).  The _h stubs for new() and Link() will conflict
// with these real implementations at integration time and must be removed from _h.rs.
use crate::codemp::RMG::RM_Objective_h::*;

/************************************************************************************************
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
impl CRMObjective {
    pub unsafe fn new(group: *mut CGPGroup) -> Self {
        // port note: zero-initialised as the construction base; the C++ constructor body
        // calls setters for most fields and then assigns mCompleted/mOrderIndex/mActive
        // directly.  Fields must be pub or pub(crate) in RM_Objective_h for the direct
        // assignments below to compile at integration time.
        let mut self_: Self = core::mem::zeroed();

        self_.SetPriority(atoi((*group).FindPairValue(
            b"priority\0".as_ptr() as *const core::ffi::c_char,
            b"0\0".as_ptr() as *const core::ffi::c_char,
        )));
        self_.SetMessage((*group).FindPairValue(
            b"message\0".as_ptr() as *const core::ffi::c_char,
            va(
                b"Objective %i Completed\0".as_ptr() as *const core::ffi::c_char,
                self_.GetPriority(),
            ) as *const core::ffi::c_char,
        ));
        self_.SetDescription((*group).FindPairValue(
            b"description\0".as_ptr() as *const core::ffi::c_char,
            va(
                b"Objective %i\0".as_ptr() as *const core::ffi::c_char,
                self_.GetPriority(),
            ) as *const core::ffi::c_char,
        ));
        self_.SetInfo((*group).FindPairValue(
            b"info\0".as_ptr() as *const core::ffi::c_char,
            va(
                b"Info %i\0".as_ptr() as *const core::ffi::c_char,
                self_.GetPriority(),
            ) as *const core::ffi::c_char,
        ));
        self_.SetTrigger((*group).FindPairValue(
            b"trigger\0".as_ptr() as *const core::ffi::c_char,
            b"\0".as_ptr() as *const core::ffi::c_char,
        ));
        self_.SetName((*group).GetName());

    /*	const char * soundPath = group->FindPairValue("completed_sound", "" );
        if (soundPath)
            mCompleteSoundID = G_SoundIndex(soundPath);
    */

        self_.mCompleted  = false;
        self_.mOrderIndex = -1;

        // If no priority was specified for this objective then its active by default.
        if self_.GetPriority() != 0 {
            self_.mActive = false;
        } else {
            self_.mActive = true;
        }

        self_
    }

    /************************************************************************************************
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

    /************************************************************************************************
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
    pub fn Link(&mut self) -> bool {
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
