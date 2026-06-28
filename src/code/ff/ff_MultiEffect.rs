// Original C++ wrapped in: #ifdef _IMMERSION ... #endif
// This module's contents are conditional on the _IMMERSION feature

use core::ffi::c_int;

// Type aliases matching C definitions
type DWORD = u32;
type qboolean = c_int;
const MAXDWORD: DWORD = u32::MAX;
const qtrue: qboolean = 1;
const qfalse: qboolean = 0;

// Placeholder for CImmEffect - actual definition in corresponding header
// This is a stub needed for structural coherence of this file
#[repr(C)]
pub struct CImmEffect {
    // Fields would be defined in the actual header
    pub m_dwLastStarted: DWORD,
}

pub struct MultiEffect {
    // Fields would be defined in the actual header
}

impl MultiEffect {
    ////--------------------------
    ///	MultiEffect::GetStartDelay
    //------------------------------
    //	Determines the shortest start delay.
    //
    pub fn GetStartDelay(&self, StartDelay: &mut DWORD) -> qboolean {
        *StartDelay = MAXDWORD;
        let mut result = qtrue;

        let max = self.GetNumberOfContainedEffects();
        for i in 0..max {
            let mut CurrentStartDelay: DWORD = 0;
            let pIE = self.GetContainedEffect(i);
            if pIE != std::ptr::null_mut()
                && unsafe { (*pIE).GetStartDelay(&mut CurrentStartDelay) != qfalse }
            {
                *StartDelay = (*StartDelay).min(CurrentStartDelay);
            } else {
                result = qfalse;
            }
        }

        if result != qfalse && max > 0 { qtrue } else { qfalse }
    }

    ////------------------------
    ///	MultiEffect::GetDelayEnd
    //----------------------------
    //	Computes end of earliest start delay. Compare this value with ::GetTickCount()
    //	to determine if any component waveform started playing on the device.
    //
    pub fn GetDelayEnd(&self, DelayEnd: &mut DWORD) -> qboolean {
        *DelayEnd = MAXDWORD;
        let mut result = qtrue;

        let max = self.GetNumberOfContainedEffects();
        for i in 0..max {
            let mut StartDelay: DWORD = 0;
            let pIE = self.GetContainedEffect(i);
            if pIE != std::ptr::null_mut()
                && unsafe { (*pIE).GetStartDelay(&mut StartDelay) != qfalse }
            {
                unsafe {
                    *DelayEnd = (*DelayEnd).min(StartDelay.wrapping_add((*pIE).m_dwLastStarted));
                }
            } else {
                result = qfalse;
            }
        }

        if result != qfalse && max > 0 { qtrue } else { qfalse }
    }

    ////---------------------------
    ///	MultiEffect::ChangeDuration
    //-------------------------------
    //	Analogous to CImmEffect::ChangeDuration. Changes duration of all component effects.
    //	Returns false if any effect returns false. Attempts to change duration of all effects
    //	regardless of individual return values.
    //
    pub fn ChangeDuration(&self, Duration: DWORD) -> qboolean {
        let mut CurrentDuration: DWORD = 0;
        let mut result = self.GetDuration(&mut CurrentDuration);

        if result != qfalse {
            let RelativeDuration = Duration.wrapping_sub(CurrentDuration);

            let max = self.GetNumberOfContainedEffects();
            for i in 0..max {
                let mut Envelope = IMM_ENVELOPE { dwAttackTime: 0, dwFadeTime: 0 };
                let pIE = self.GetContainedEffect(i);

                let mut CurrentDuration: DWORD = 0;
                if pIE != std::ptr::null_mut()
                    && unsafe { (*pIE).GetDuration(&mut CurrentDuration) != qfalse }
                    && unsafe {
                        (*pIE).ChangeDuration(CurrentDuration.wrapping_add(RelativeDuration))
                            != qfalse
                    }
                {
                    let envelope_check = unsafe { (*pIE).GetEnvelope(&mut Envelope) };
                    // If GetEnvelope failed, that's OK (the ! makes it true in the original)
                    // If GetEnvelope succeeded, we need to scale and call ChangeEnvelope
                    if envelope_check != qfalse {
                        if CurrentDuration != 0 {
                            Envelope.dwAttackTime = (((Envelope.dwAttackTime as f32)
                                * (Duration as f32)
                                / (CurrentDuration as f32)) as u32);
                            Envelope.dwFadeTime = (((Envelope.dwFadeTime as f32)
                                * (Duration as f32)
                                / (CurrentDuration as f32)) as u32);
                        } else {
                            Envelope.dwAttackTime = 0;
                            Envelope.dwFadeTime = 0;
                        }

                        if unsafe { (*pIE).ChangeEnvelope(&Envelope) == qfalse } {
                            result = qfalse;
                        }
                    }
                } else {
                    result = qfalse;
                }
            }

            if max == 0 {
                result = qfalse;
            }
        }

        result
    }

    ////-----------------------
    ///	MultiEffect::ChangeGain
    //---------------------------
    //	Analogous to CImmEffect::ChangeGain. Changes gain of all component effects.
    //	Returns false if any effect returns false. Attempts to change gain of all effects
    //	regardless of individual return values.
    //
    pub fn ChangeGain(&self, Gain: DWORD) -> qboolean {
        let mut CurrentGain: DWORD = 0;
        let mut result = self.GetGain(&mut CurrentGain);

        if result != qfalse {
            let RelativeGain = Gain.wrapping_sub(CurrentGain);

            let max = self.GetNumberOfContainedEffects();
            for i in 0..max {
                let pIE = self.GetContainedEffect(i);
                if pIE != std::ptr::null_mut() {
                    let mut CurrentGain: DWORD = 0;
                    if unsafe { (*pIE).GetGain(&mut CurrentGain) != qfalse }
                        && unsafe {
                            (*pIE).ChangeGain(CurrentGain.wrapping_add(RelativeGain)) != qfalse
                        }
                    {
                        // success
                    } else {
                        result = qfalse;
                    }
                } else {
                    result = qfalse;
                }
            }

            if max == 0 {
                result = qfalse;
            }
        }

        result
    }

    ////----------------------
    ///	MultiEffect::GetStatus
    //--------------------------
    //	Analogous to CImmEffect::GetStatus. ORs all status flags from all component effects.
    //	Returns false if any effect returns false. Attempts to get status of all effects
    //	regardless of individual return values.
    //
    pub fn GetStatus(&self, Status: &mut DWORD) -> qboolean {
        *Status = 0;
        let mut result = qtrue;

        let max = self.GetNumberOfContainedEffects();
        for i in 0..max {
            let mut CurrentStatus: DWORD = 0;
            let pIE = self.GetContainedEffect(i);
            if pIE != std::ptr::null_mut()
                && unsafe { (*pIE).GetStatus(&mut CurrentStatus) != qfalse }
            {
                *Status |= CurrentStatus;
            } else {
                result = qfalse;
            }
        }

        if result != qfalse && max > 0 { qtrue } else { qfalse }
    }

    pub fn ChangeStartDelay(&self, StartDelay: DWORD) -> qboolean {
        let mut CurrentStartDelay: DWORD = 0;
        let mut result = self.GetStartDelay(&mut CurrentStartDelay);

        if result != qfalse {
            let RelativeStartDelay = StartDelay.wrapping_sub(CurrentStartDelay);

            let max = self.GetNumberOfContainedEffects();
            for i in 0..max {
                let pIE = self.GetContainedEffect(i);
                if pIE != std::ptr::null_mut() {
                    let mut CurrentStartDelay: DWORD = 0;
                    if unsafe { (*pIE).GetStartDelay(&mut CurrentStartDelay) != qfalse }
                        && unsafe {
                            (*pIE).ChangeStartDelay(
                                CurrentStartDelay.wrapping_add(RelativeStartDelay),
                            ) != qfalse
                        }
                    {
                        // success
                    } else {
                        result = qfalse;
                    }
                } else {
                    result = qfalse;
                }
            }

            if max == 0 {
                result = qfalse;
            }
        }

        result
    }

    pub fn GetDuration(&self, Duration: &mut DWORD) -> qboolean {
        *Duration = 0;
        let mut result = qtrue;

        let max = self.GetNumberOfContainedEffects();
        for i in 0..max {
            let mut CurrentDuration: DWORD = 0;
            let pIE = self.GetContainedEffect(i);
            if pIE != std::ptr::null_mut()
                && unsafe { (*pIE).GetDuration(&mut CurrentDuration) != qfalse }
            {
                *Duration = (*Duration).max(CurrentDuration);
            } else {
                result = qfalse;
            }
        }

        if result != qfalse && max > 0 { qtrue } else { qfalse }
    }

    pub fn GetGain(&self, Gain: &mut DWORD) -> qboolean {
        *Gain = 0;
        let mut result = qtrue;

        let max = self.GetNumberOfContainedEffects();
        for i in 0..max {
            let mut CurrentGain: DWORD = 0;
            let pIE = self.GetContainedEffect(i);
            if pIE != std::ptr::null_mut()
                && unsafe { (*pIE).GetGain(&mut CurrentGain) != qfalse }
            {
                *Gain = (*Gain).max(CurrentGain);
            } else {
                result = qfalse;
            }
        }

        if result != qfalse && max > 0 { qtrue } else { qfalse }
    }

    // Stub methods needed for structural coherence
    fn GetNumberOfContainedEffects(&self) -> usize {
        0
    }

    fn GetContainedEffect(&self, _index: usize) -> *mut CImmEffect {
        std::ptr::null_mut()
    }
}

#[repr(C)]
pub struct IMM_ENVELOPE {
    pub dwAttackTime: DWORD,
    pub dwFadeTime: DWORD,
}
