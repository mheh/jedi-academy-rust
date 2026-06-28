/*
	Copyright (c) 1999 - 2000 Immersion Corporation

	Permission to use, copy, modify, distribute, and sell this
	software and its documentation may be granted without fee;
	interested parties are encouraged to request permission from
		Immersion Corporation
		801 Fox Lane
		San Jose, CA 95131
		408-467-1900

	IMMERSION DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE,
	INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS.
	IN NO EVENT SHALL IMMERSION BE LIABLE FOR ANY SPECIAL, INDIRECT OR
	CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
	LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
	NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
	CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

	FILE:		ImmEffectSuite.h

	PURPOSE:	Caching of effects

	STARTED:	6/16/99 Jeff Mallett

	NOTES/REVISIONS:

*/

#![allow(non_snake_case)]

#[cfg(feature = "ifc_effect_caching")]
pub mod ifc_effect_caching_types {
	use core::ffi::c_int;
	use std::ptr::null_mut;

	// include "ImmBaseTypes.h" — unported external dependency

	// Forward declarations
	pub struct CImmDevice;
	pub struct CImmEffect;

	#[repr(C)]
	pub enum ECacheState {
		IMMCACHE_NOT_ON_DEVICE,
		IMMCACHE_ON_DEVICE,
		IMMCACHE_SWAPPED_OUT,
	}

	//================================================================
	// CEffectList, CEffectListElement
	//================================================================

	#[repr(C)]
	pub struct CEffectListElement {
		pub m_pImmEffect: *mut CImmEffect,
		pub m_pNext: *mut CEffectListElement,
	}

	impl CEffectListElement {
		pub fn new() -> Self {
			// CEffectListElement() : m_pImmEffect(NULL), m_pNext(NULL) { }
			CEffectListElement {
				m_pImmEffect: null_mut(),
				m_pNext: null_mut(),
			}
		}
	}

	#[repr(C)]
	pub struct CEffectList {
		pub m_pFirstEffect: *mut CEffectListElement,
	}

	impl CEffectList {
		pub fn new() -> Self {
			// CEffectList() : m_pFirstEffect(NULL) { }
			CEffectList {
				m_pFirstEffect: null_mut(),
			}
		}

		// ~CEffectList();
		// Destructor implementation unported (defined in .cpp source)

		pub fn AddEffect(&mut self, pImmEffect: *mut CImmEffect) -> c_int {
			// BOOL AddEffect(CImmEffect *pImmEffect);
			// Stub: implementation unported (defined in .cpp source)
			0
		}

		pub fn RemoveEffect(&self, pImmEffect: *const CImmEffect) -> c_int {
			// BOOL RemoveEffect(const CImmEffect *pImmEffect);
			// Stub: implementation unported (defined in .cpp source)
			0
		}

		pub fn ClearDevice(&mut self, pImmDevice: *mut CImmDevice) {
			// void ClearDevice(CImmDevice *pImmDevice);
			// Stub: implementation unported (defined in .cpp source)
		}
	}

	//================================================================
	// CImmEffectSuite
	//================================================================

	#[repr(C)]
	pub struct CImmEffectSuite {
		pub m_bCurrentSuite: c_int, // Is the suite the "current suite"?
		m_EffectList: CEffectList,   // List of effects in suite
	}

	impl CImmEffectSuite {
		pub fn new() -> Self {
			// CImmEffectSuite() : m_bCurrentSuite(false) { }
			CImmEffectSuite {
				m_bCurrentSuite: 0,
				m_EffectList: CEffectList::new(),
			}
		}

		pub fn GetFirstEffect(&self) -> *mut CEffectListElement {
			// CEffectListElement *GetFirstEffect();
			// Stub: implementation unported
			null_mut()
		}

		pub fn AddEffect(&mut self, pImmEffect: *mut CImmEffect) {
			// void AddEffect(CImmEffect *pImmEffect);
			// Stub: implementation unported
		}

		pub fn RemoveEffect(&mut self, pImmEffect: *mut CImmEffect) {
			// void RemoveEffect(CImmEffect *pImmEffect);
			// Stub: implementation unported
		}

		pub fn SetPriorities(&mut self, priority: i16) {
			// void SetPriorities(short priority);
			// Stub: implementation unported
		}
	}
}
