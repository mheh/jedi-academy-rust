/*
	File:		MacGamma.cpp

	Contains:	Functions to enable Mac OS device gamma adjustments using Windows common 3 channel 256 element 8 bit gamma ramps

	Written by:	Geoff Stahl

	Copyright:	Copyright © 1999 Apple Computer, Inc., All Rights Reserved

	Change History (most recent first):

	         <4>     5/20/99    GGS     Added handling for gamma tables with different data widths,
	                                    number of entries, and channels.  Forced updates to 3 channels
	                                    (poss. could break on rare card, but very unlikely).  Added
	                                    quick update with BlockMove for 3x256x8 tables. Updated function
	                                    names.
	         <3>     5/20/99    GGS     Cleaned up and commented
	         <2>     5/20/99    GGS     Added system wide get and restore gamma functions to enable
	                                    restoration of original for all devices.  Modified functionality
	                                    to return pointers vice squirreling away the memory.
	         <1>     5/20/99    GGS     Initial Add
*/

// system includes ----------------------------------------------------------

// Stub for Mac OS types and functions from <Devices.h>, <Files.h>, <MacTypes.h>, <QDOffscreen.h>, <Quickdraw.h>, <video.h>
use core::ffi::{c_int, c_short, c_void, c_char};

// Mac OS type stubs
pub type OSErr = c_short;
pub type GDHandle = *mut c_void;
pub type GammaTblPtr = *mut c_void;
pub type GWorldPtr = *mut c_void;
pub type CGrafPtr = *mut c_void;
pub type CTabHandle = *mut c_void;
pub type Boolean = c_int;

// Stub structures for Mac OS types
#[repr(C)]
pub struct VDGammaRecord {
	pub csGTable: *mut c_void,
}

#[repr(C)]
pub struct CntrlParam {
	pub ioCompletion: *mut c_void,
	pub ioNamePtr: *mut c_void,
	pub ioVRefNum: c_int,
	pub ioCRefNum: c_int,
	pub csCode: c_int,
	pub csParam: [u8; 16],	// Simplified; actual layout varies
}

#[repr(C)]
pub struct VDSetEntryRecord {
	pub csTable: *mut c_void,	// *ColorSpec
	pub csStart: c_int,
	pub csCount: c_int,
}

#[repr(C)]
pub struct ColorSpec {
	// Stub structure
	_padding: [u8; 8],
}

// External Mac OS functions
extern "C" {
	pub fn PBStatus(parmBlock: *mut c_void, async_flag: c_int) -> OSErr;
	pub fn NewPtrClear(size: c_int) -> *mut c_void;
	pub fn NewPtr(size: c_int) -> *mut c_void;
	pub fn BlockMove(srcPtr: *const c_void, dstPtr: *mut c_void, byteCount: c_int) -> ();
	pub fn DisposePtr(ptr: *mut c_void) -> ();
	pub fn Control(refNum: c_int, csCode: c_int, csParam: *mut c_void) -> OSErr;
	pub fn GetDeviceList() -> GDHandle;
	pub fn GetNextDevice(hGD: GDHandle) -> GDHandle;
	pub fn GetGWorldDevice(pGW: GWorldPtr) -> GDHandle;
	pub fn GetGWorld(pGrafPtr: *mut CGrafPtr, hGDPtr: *mut GDHandle) -> ();
	pub fn SetGWorld(pGraf: CGrafPtr, hGD: GDHandle) -> ();
	pub fn GetGDevice() -> GDHandle;
}

// Constants
const cscGetGamma: c_int = 0x0500;	// Get Gamma command to device
const cscSetGamma: c_int = 0x0501;	// Set Gamma command to device
const cscSetEntries: c_int = 0x0502; // Set Entries in CLUT command

// project includes ---------------------------------------------------------

// functions (external/public) ----------------------------------------------

// GetGammaTable

// Returns the device gamma table pointer in ppDeviceTable

#[no_mangle]
pub extern "C" fn GetGammaTable(hGD: GDHandle, ppTableGammaOut: *mut GammaTblPtr) -> OSErr
{
	let mut DeviceGammaRec: VDGammaRecord = VDGammaRecord {
		csGTable: core::ptr::null_mut(),
	};
	let mut cParam: CntrlParam = CntrlParam {
		ioCompletion: core::ptr::null_mut(),	// set up control params
		ioNamePtr: core::ptr::null_mut(),
		ioVRefNum: 0,
		ioCRefNum: 0,
		csCode: 0,
		csParam: [0; 16],
	};
	let mut err: OSErr;

	cParam.ioCompletion = core::ptr::null_mut();										// set up control params
	cParam.ioNamePtr = core::ptr::null_mut();
	cParam.ioVRefNum = 0;
	unsafe {
		cParam.ioCRefNum = (*(hGD as *mut c_void as *const c_int));
	}
	cParam.csCode = cscGetGamma;									// Get Gamma commnd to device
	let csParam_ptr = &mut cParam.csParam as *mut [u8; 16] as *mut *mut VDGammaRecord;
	unsafe {
		*csParam_ptr = &mut DeviceGammaRec;
	}

	unsafe {
		err = PBStatus(&mut cParam as *mut _ as *mut c_void, 0);						// get gamma

		*ppTableGammaOut = DeviceGammaRec.csGTable;		// pull table out of record
	}

	err
}

// --------------------------------------------------------------------------

// CreateEmptyGammaTable

// creates an empty gamma table of a given size, assume no formula data will be used

#[no_mangle]
pub extern "C" fn CreateEmptyGammaTable(channels: c_short, entries: c_short, bits: c_short) -> *mut c_void
{
	let mut pTableGammaOut: GammaTblPtr = core::ptr::null_mut();
	let tableSize: c_int;
	let dataWidth: c_int;

	dataWidth = ((bits as c_int) + 7) / 8;										// number of bytes per entry
	tableSize = core::mem::size_of::<GammaTbl>() as c_int + ((channels as c_int) * (entries as c_int) * dataWidth);
	unsafe {
		pTableGammaOut = NewPtrClear(tableSize) as GammaTblPtr;			// allocate new tabel

		if !pTableGammaOut.is_null()												// if we successfully allocated
		{
			let pTableGammaOut_ref = &mut *(pTableGammaOut as *mut GammaTbl);
			pTableGammaOut_ref.gVersion = 0;								// set parameters based on input
			pTableGammaOut_ref.gType = 0;
			pTableGammaOut_ref.gFormulaSize = 0;
			pTableGammaOut_ref.gChanCnt = channels;
			pTableGammaOut_ref.gDataCnt = entries;
			pTableGammaOut_ref.gDataWidth = bits;
		}
	}
	pTableGammaOut as *mut c_void										// return whatever we allocated
}

// --------------------------------------------------------------------------

// CopyGammaTable

// given a pointer toa device gamma table properly iterates and copies

#[no_mangle]
pub extern "C" fn CopyGammaTable(pTableGammaIn: GammaTblPtr) -> *mut c_void
{
	let mut pTableGammaOut: GammaTblPtr = core::ptr::null_mut();
	let tableSize: c_int;
	let dataWidth: c_int;

	if !pTableGammaIn.is_null()												// if there is a table to copy
	{
		unsafe {
			let pTableGammaIn_ref = &*(pTableGammaIn as *const GammaTbl);
			dataWidth = ((pTableGammaIn_ref.gDataWidth as c_int) + 7) / 8;			// number of bytes per entry
			tableSize = core::mem::size_of::<GammaTbl>() as c_int + pTableGammaIn_ref.gFormulaSize as c_int +
						((pTableGammaIn_ref.gChanCnt as c_int) * (pTableGammaIn_ref.gDataCnt as c_int) * dataWidth);
			pTableGammaOut = NewPtr(tableSize) as GammaTblPtr;			// allocate new table
			if !pTableGammaOut.is_null()
			{
				BlockMove(pTableGammaIn as *const c_void, pTableGammaOut as *mut c_void, tableSize);	// move everything
			}
		}
	}
	pTableGammaOut as *mut c_void										// return whatever we allocated, could be NULL
}

// --------------------------------------------------------------------------

// DisposeGammaTable

// disposes gamma table returned from GetGammaTable, GetDeviceGamma, or CopyGammaTable
// 5/20/99: (GGS) added

#[no_mangle]
pub extern "C" fn DisposeGammaTable(pGamma: *mut c_void)
{
	if !pGamma.is_null() {
		unsafe {
			DisposePtr(pGamma);									// get rid of it
		}
	}
}

// --------------------------------------------------------------------------

// GetDeviceGamma

// returns pointer to copy of orginal device gamma table in native format (allocates memory for gamma table, call DisposeDeviceGamma to delete)
// 5/20/99: (GGS) change spec to return the allocated pointer vice storing internally

#[no_mangle]
pub extern "C" fn GetDeviceGamma(hGD: GDHandle) -> *mut c_void
{
	let mut pTableGammaDevice: GammaTblPtr = core::ptr::null_mut();
	let mut pTableGammaReturn: GammaTblPtr = core::ptr::null_mut();
	let err: OSErr;

	unsafe {
		err = GetGammaTable(hGD, &mut pTableGammaDevice);					// get a pointer to the devices table
		if (err == 0) && !pTableGammaDevice.is_null()						// if succesful
		{
			pTableGammaReturn = CopyGammaTable(pTableGammaDevice) as GammaTblPtr; // copy to global
		}
	}

	pTableGammaReturn as *mut c_void
}

// --------------------------------------------------------------------------

// RestoreDeviceGamma

// sets device to saved table
// 5/20/99: (GGS) now does not delete table, avoids confusion

#[no_mangle]
pub extern "C" fn RestoreDeviceGamma(hGD: GDHandle, pGammaTable: *mut c_void)
{
	let mut setEntriesRec: VDSetEntryRecord = VDSetEntryRecord {
		csTable: core::ptr::null_mut(),
		csStart: 0,
		csCount: 0,
	};
	let mut gameRecRestore: VDGammaRecord = VDGammaRecord {
		csGTable: core::ptr::null_mut(),
	};
	let mut hCTabDeviceColors: CTabHandle;
	let mut csPtr: *mut c_void;
	let mut err: OSErr = 0;

	if !pGammaTable.is_null()												// if we have a table to restore
	{
		unsafe {
			gameRecRestore.csGTable = pGammaTable;						// setup restore record
			csPtr = &mut gameRecRestore as *mut _ as *mut c_void;
			err = Control((*((hGD as *mut c_void as *mut c_int)) as i32), cscSetGamma, &mut csPtr);	// restore gamma

			if (err == 0) && ((*(*(*(hGD as *mut c_void as *mut *mut c_void) as *mut c_void as *mut c_int)) as c_int) == 8)	// if successful and on an 8 bit device
			{
				hCTabDeviceColors = (*(*(hGD as *mut c_void as *mut *mut c_void) as *mut c_void as *mut *mut c_void) as *mut c_void);			// do SetEntries to force CLUT update
				setEntriesRec.csTable = &(*(*(hCTabDeviceColors as *mut *mut ColorSpec) as *mut ColorSpec) as *mut ColorSpec as *mut c_void);
				setEntriesRec.csStart = 0;
				setEntriesRec.csCount = (*(*(hCTabDeviceColors as *mut *mut c_int) as *mut c_int) as c_int);
				csPtr = &mut setEntriesRec as *mut _ as *mut c_void;

				err = Control((*((hGD as *mut c_void as *mut c_int)) as i32), cscSetEntries, &mut csPtr); // SetEntries in CLUT
			}
		}
	}
}

// --------------------------------------------------------------------------

// GetSystemGammas

// returns a pointer to a set of all current device gammas in native format (returns NULL on failure, which means reseting gamma will not be possible)
// 5/20/99: (GGS) added

#[no_mangle]
pub extern "C" fn GetSystemGammas() -> *mut c_void
{
	let mut pSysGammaOut: *mut recSystemGamma;									// return pointer to system device gamma info
	let mut devCount: c_short = 0;												// number of devices attached
	let mut fail: bool = false;
	let mut hGDevice: GDHandle;

	unsafe {
		pSysGammaOut = NewPtr(core::mem::size_of::<recSystemGamma>() as c_int) as *mut recSystemGamma; // allocate for structure

		hGDevice = GetDeviceList();							// top of device list
		loop																// iterate
		{
			devCount += 1;													// count devices
			hGDevice = GetNextDevice(hGDevice);						// next device
			if hGDevice.is_null() { break; }
		}

		(*pSysGammaOut).devGamma = NewPtr((core::mem::size_of::<*mut recDeviceGamma>() as c_int) * (devCount as c_int)) as *mut *mut recDeviceGamma; // allocate for array of pointers to device records
		if !pSysGammaOut.is_null()
		{
			(*pSysGammaOut).numDevices = devCount;						// stuff count

			devCount = 0;												// reset iteration
			hGDevice = GetDeviceList();
			loop
			{
				let pDeviceGammaRecord = NewPtr(core::mem::size_of::<recDeviceGamma>() as c_int) as *mut recDeviceGamma;	  // new device record
				*((*pSysGammaOut).devGamma.add(devCount as usize)) = pDeviceGammaRecord;
				if !pDeviceGammaRecord.is_null()					// if we actually allocated memory
				{
					(*pDeviceGammaRecord).hGD = hGDevice;										  // stuff handle
					(*pDeviceGammaRecord).pDeviceGamma = GetDeviceGamma(hGDevice) as GammaTblPtr; // copy gamma table
				}
				else													// otherwise dump record on exit
				{
					fail = true;
				}
				devCount += 1;												// next device
				hGDevice = GetNextDevice(hGDevice);
				if hGDevice.is_null() { break; }
			}
		}
		if !fail														// if we did not fail
		{
			return pSysGammaOut as *mut c_void;									// return pointer to structure
		}
		else
		{
			DisposeSystemGammas(&mut (pSysGammaOut as *mut c_void));					// otherwise dump the current structures (dispose does error checking)
			return core::ptr::null_mut();												// could not complete
		}
	}
}

// --------------------------------------------------------------------------

// RestoreSystemGammas

// restores all system devices to saved gamma setting
// 5/20/99: (GGS) added

#[no_mangle]
pub extern "C" fn RestoreSystemGammas(pSystemGammas: *mut c_void)
{
	let mut i: c_short;
	let pSysGammaIn: *mut recSystemGamma = pSystemGammas as *mut recSystemGamma;
	unsafe {
		if !pSysGammaIn.is_null()
		{
			i = 0;
			while i < (*pSysGammaIn).numDevices {			// for all devices
				RestoreDeviceGamma((*(*(*pSysGammaIn).devGamma.add(i as usize))).hGD, (*((*pSysGammaIn).devGamma.add(i as usize)) as *mut c_void).cast::<c_void>().cast_mut());	// restore gamma
				i += 1;
			}
		}
	}
}

// --------------------------------------------------------------------------

// DisposeSystemGammas

// iterates through and deletes stored gamma settings
// 5/20/99: (GGS) added

#[no_mangle]
pub extern "C" fn DisposeSystemGammas(ppSystemGammas: *mut *mut c_void)
{
	let mut pSysGammaIn: *mut recSystemGamma;
	unsafe {
		if !ppSystemGammas.is_null()
		{
			pSysGammaIn = *ppSystemGammas as *mut recSystemGamma;
			if !pSysGammaIn.is_null()
			{
				let mut i: c_short;
				i = 0;
				while i < (*pSysGammaIn).numDevices {		// for all devices
					if !(*(*pSysGammaIn).devGamma.add(i as usize)).is_null()						// if pointer is valid
					{
						DisposeGammaTable((*(*pSysGammaIn).devGamma.add(i as usize)) as *mut c_void); // dump gamma table
						DisposePtr((*(*pSysGammaIn).devGamma.add(i as usize)) as *mut c_void);						 // dump device info
					}
					i += 1;
				}
				DisposePtr((*pSysGammaIn).devGamma as *mut c_void);				// dump device pointer array
				DisposePtr(pSysGammaIn as *mut c_void);							// dump system structure
				*ppSystemGammas = core::ptr::null_mut();
			}
		}
	}
}

// --------------------------------------------------------------------------

// GetDeviceGammaRampGD

// retrieves the gamma ramp from a graphics device (pRamp: 3 arrays of 256 elements each)

#[no_mangle]
pub extern "C" fn GetDeviceGammaRampGD(hGD: GDHandle, pRamp: *mut c_void) -> Boolean
{
	let mut pTableGammaTemp: GammaTblPtr = core::ptr::null_mut();
	let mut indexChan: c_int;
	let mut indexEntry: c_int;
	let err: OSErr;

	if !pRamp.is_null()															// ensure pRamp is allocated
	{
		unsafe {
			err = GetGammaTable(hGD, &mut pTableGammaTemp);					// get a pointer to the current gamma
			if (err == 0) && !pTableGammaTemp.is_null()							// if successful
			{
				// fill ramp
				let pTableGammaTemp_ref = &*(pTableGammaTemp as *const GammaTbl);
				let mut pEntry: *const u8 = (&pTableGammaTemp_ref.gFormulaData as *const u8).add(pTableGammaTemp_ref.gFormulaSize as usize); // base of table
				let bytesPerEntry: c_int = ((pTableGammaTemp_ref.gDataWidth as c_int) + 7) / 8; // size, in bytes, of the device table entries
				let shiftRightValue: c_int = (pTableGammaTemp_ref.gDataWidth as c_int) - 8; 	 // number of right shifts device -> ramp
				let channels: c_int = pTableGammaTemp_ref.gChanCnt as c_int;
				let entries: c_int = pTableGammaTemp_ref.gDataCnt as c_int;
				if channels == 3										// RGB format
				{															// note, this will create runs of entries if dest. is bigger (not linear interpolate)
					indexChan = 0;
					while indexChan < channels {
						indexEntry = 0;
						while indexEntry < 256 {
							*((pRamp as *mut u8).add(((indexChan << 8) + indexEntry) as usize)) =
							  (*(pEntry.add(((indexChan * entries * bytesPerEntry) + indexEntry * ((entries * bytesPerEntry) >> 8)) as usize)) >> shiftRightValue) as u8;
							indexEntry += 1;
						}
						indexChan += 1;
					}
				}
				else														// single channel format
				{
					indexEntry = 0;
					while indexEntry < 256 {	// for all entries set vramp value
						indexChan = 0;
						while indexChan < channels {	// repeat for all channels
							*((pRamp as *mut u8).add(((indexChan << 8) + indexEntry) as usize)) =
							  (*(pEntry.add((((indexEntry * entries * bytesPerEntry) >> 8)) as usize)) >> shiftRightValue) as u8;
							indexChan += 1;
						}
						indexEntry += 1;
					}
				}
				return 1;	// true
			}
		}
	}
	0	// false
}

// --------------------------------------------------------------------------

// GetDeviceGammaRampGW

// retrieves the gamma ramp from a graphics device associated with a GWorld pointer (pRamp: 3 arrays of 256 elements each)

#[no_mangle]
pub extern "C" fn GetDeviceGammaRampGW(pGW: GWorldPtr, pRamp: *mut c_void) -> Boolean
{
	unsafe {
		let hGD: GDHandle = GetGWorldDevice(pGW);
		GetDeviceGammaRampGD(hGD, pRamp)
	}
}

// --------------------------------------------------------------------------

// GetDeviceGammaRampCGP

// retrieves the gamma ramp from a graphics device associated with a CGraf pointer (pRamp: 3 arrays of 256 elements each)

#[no_mangle]
pub extern "C" fn GetDeviceGammaRampCGP(pGraf: CGrafPtr, pRamp: *mut c_void) -> Boolean
{
	unsafe {
		let mut pGrafSave: CGrafPtr = core::ptr::null_mut();
		let mut hGDSave: GDHandle = core::ptr::null_mut();
		let mut hGD: GDHandle;
		let fResult: Boolean;

		GetGWorld(&mut pGrafSave, &mut hGDSave);
		SetGWorld(pGraf, core::ptr::null_mut());
		hGD = GetGDevice();
		fResult = GetDeviceGammaRampGD(hGD, pRamp);
		SetGWorld(pGrafSave, hGDSave);
		fResult
	}
}

// --------------------------------------------------------------------------

// SetDeviceGammaRampGD

// sets the gamma ramp for a graphics device (pRamp: 3 arrays of 256 elements each (R,G,B))

#[no_mangle]
pub extern "C" fn SetDeviceGammaRampGD(hGD: GDHandle, pRamp: *mut c_void) -> Boolean
{
	let mut setEntriesRec: VDSetEntryRecord = VDSetEntryRecord {
		csTable: core::ptr::null_mut(),
		csStart: 0,
		csCount: 0,
	};
	let mut gameRecRestore: VDGammaRecord = VDGammaRecord {
		csGTable: core::ptr::null_mut(),
	};
	let mut pTableGammaNew: GammaTblPtr;
	let mut pTableGammaCurrent: GammaTblPtr = core::ptr::null_mut();
	let mut hCTabDeviceColors: CTabHandle;
	let mut csPtr: *mut c_void;
	let err: OSErr;
	let dataBits: c_int;
	let entries: c_int;
	let channels: c_int = 3;						// force three channels in the gamma table

	if !pRamp.is_null()																// ensure pRamp is allocated
	{
		unsafe {
			err = GetGammaTable(hGD, &mut pTableGammaCurrent);						// get pointer to current table
			if (err == 0) && !pTableGammaCurrent.is_null()
			{
				let pTableGammaCurrent_ref = &*(pTableGammaCurrent as *const GammaTbl);
				dataBits = pTableGammaCurrent_ref.gDataWidth as c_int;						// table must have same data width
				entries = pTableGammaCurrent_ref.gDataCnt as c_int;							// table must be same size
				pTableGammaNew = CreateEmptyGammaTable(channels as c_short, entries as c_short, dataBits as c_short) as GammaTblPtr; // our new table
				if !pTableGammaNew.is_null()												// if successful fill table
				{
					let pTableGammaNew_ref = &mut *(pTableGammaNew as *mut GammaTbl);
					let mut pGammaBase: *mut u8 = (&mut pTableGammaNew_ref.gFormulaData as *mut u8).add(pTableGammaNew_ref.gFormulaSize as usize); // base of table
					if entries == 256 && dataBits == 8						// simple case: direct mapping
					{
						BlockMove(pRamp, pGammaBase as *mut c_void, channels * entries); // move everything
					}
					else														// tough case handle entry, channel and data size disparities
					{
						let bytesPerEntry: c_int = (dataBits + 7) / 8; 				// size, in bytes, of the device table entries
						let mut shiftRightValue: c_int = 8 - dataBits;					// number of right shifts ramp -> device
						let mut indexChan: c_int;
						let mut indexEntry: c_int;
						let mut indexByte: c_int;

						shiftRightValue += ((bytesPerEntry - 1) * 8);  			// multibyte entries and the need to map a byte at a time most sig. to least sig.
						indexChan = 0;
						while indexChan < channels { // for all the channels
							indexEntry = 0;
							while indexEntry < entries { // for all the entries
								let mut currentShift: c_int = shiftRightValue;			// reset current bit shift
								let temp: c_int = *((pRamp as *mut u8).add(((indexChan << 8) + (indexEntry << 8) / entries) as usize)) as c_int; // get data from ramp
								indexByte = 0;
								while indexByte < bytesPerEntry { // for all bytes
									if currentShift < 0						// shift data correctly for current byte
									{
										*pGammaBase = ((temp << -currentShift) & 0xFF) as u8;
									}
									else
									{
										*pGammaBase = ((temp >> currentShift) & 0xFF) as u8;
									}
									pGammaBase = pGammaBase.add(1);
									currentShift -= 8;							// increment shift to align to next less sig. byte
									indexByte += 1;
								}
								indexEntry += 1;
							}
							indexChan += 1;
						}
					}

					// set gamma
					gameRecRestore.csGTable = pTableGammaNew as *mut c_void;				// setup restore record
					csPtr = &mut gameRecRestore as *mut _ as *mut c_void;
					let err2 = Control((*((hGD as *mut c_void as *mut c_int)) as i32), cscSetGamma, &mut csPtr);	// restore gamma

					if ((*((hGD as *mut c_void as *mut c_int)) as i32) == 8) && (err2 == 0)	// if successful and on an 8 bit device
					{
						hCTabDeviceColors = (*(*(hGD as *mut c_void as *mut *mut c_void) as *mut c_void as *mut *mut c_void) as *mut c_void);			// do SetEntries to force CLUT update
						setEntriesRec.csTable = &(*(*(hCTabDeviceColors as *mut *mut ColorSpec) as *mut ColorSpec) as *mut ColorSpec as *mut c_void);
						setEntriesRec.csStart = 0;
						setEntriesRec.csCount = (*(*(hCTabDeviceColors as *mut *mut c_int) as *mut c_int) as c_int);
						csPtr = &mut setEntriesRec as *mut _ as *mut c_void;
						let _ = Control((*((hGD as *mut c_void as *mut c_int)) as i32), cscSetEntries, &mut csPtr);	// SetEntries in CLUT
					}
					DisposeGammaTable(pTableGammaNew as *mut c_void);					// dump table
					if err2 == 0
					{
						return 1; // true
					}
				}
			}
		}
	}
	else																	// set NULL gamma -> results in linear map
	{
		unsafe {
			gameRecRestore.csGTable = core::ptr::null_mut();								// setup restore record
			csPtr = &mut gameRecRestore as *mut _ as *mut c_void;
			let err3 = Control((*((hGD as *mut c_void as *mut c_int)) as i32), cscSetGamma, &mut csPtr);			// restore gamma

			if (((*((hGD as *mut c_void as *mut c_int)) as i32) == 8) && (err3 == 0))			// if successful and on an 8 bit device
			{
				hCTabDeviceColors = (*(*(hGD as *mut c_void as *mut *mut c_void) as *mut c_void as *mut *mut c_void) as *mut c_void);					// do SetEntries to force CLUT update
				setEntriesRec.csTable = &(*(*(hCTabDeviceColors as *mut *mut ColorSpec) as *mut ColorSpec) as *mut ColorSpec as *mut c_void);
				setEntriesRec.csStart = 0;
				setEntriesRec.csCount = (*(*(hCTabDeviceColors as *mut *mut c_int) as *mut c_int) as c_int);
				csPtr = &mut setEntriesRec as *mut _ as *mut c_void;
				let _ = Control((*((hGD as *mut c_void as *mut c_int)) as i32), cscSetEntries, &mut csPtr);	// SetEntries in CLUT
			}
			if err3 == 0
			{
				return 1; // true
			}
		}
	}
	0	// false; memory allocation or device control failed if we get here
}

// --------------------------------------------------------------------------

// SetDeviceGammaRampGW

// sets the gamma ramp for a graphics device associated with a GWorld pointer (pRamp: 3 arrays of 256 elements each (R,G,B))

#[no_mangle]
pub extern "C" fn SetDeviceGammaRampGW(pGW: GWorldPtr, pRamp: *mut c_void) -> Boolean
{
	unsafe {
		let hGD: GDHandle = GetGWorldDevice(pGW);
		SetDeviceGammaRampGD(hGD, pRamp)
	}
}

// --------------------------------------------------------------------------

// SetDeviceGammaRampCGP

// sets the gamma ramp for a graphics device associated with a CGraf pointer (pRamp: 3 arrays of 256 elements each (R,G,B))

#[no_mangle]
pub extern "C" fn SetDeviceGammaRampCGP(pGraf: CGrafPtr, pRamp: *mut c_void) -> Boolean
{
	unsafe {
		let mut pGrafSave: CGrafPtr = core::ptr::null_mut();
		let mut hGDSave: GDHandle = core::ptr::null_mut();
		let mut hGD: GDHandle;
		let fResult: Boolean;

		GetGWorld(&mut pGrafSave, &mut hGDSave);
		SetGWorld(pGraf, core::ptr::null_mut());
		hGD = GetGDevice();
		fResult = SetDeviceGammaRampGD(hGD, pRamp);
		SetGWorld(pGrafSave, hGDSave);
		fResult
	}
}

// --------------------------------------------------------------------------

// Stub structures for GammaTbl and device gamma records

#[repr(C)]
pub struct GammaTbl {
	pub gVersion: c_short,
	pub gType: c_short,
	pub gFormulaSize: c_short,
	pub gChanCnt: c_short,
	pub gDataCnt: c_short,
	pub gDataWidth: c_short,
	pub gFormulaData: u8,	// Start of formula data (variable length)
}

#[repr(C)]
pub struct recDeviceGamma {
	pub hGD: GDHandle,
	pub pDeviceGamma: GammaTblPtr,
}

#[repr(C)]
pub struct recSystemGamma {
	pub numDevices: c_short,
	pub devGamma: *mut *mut recDeviceGamma,
}
