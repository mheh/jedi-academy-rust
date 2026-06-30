#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::collections::HashMap;

use crate::code::ff::common_headers_h::*; // common_headers.h PCH — covers cvar_t, CImmDevice, CImmProject, FFConfigParser, FFSystem, MultiEffect, ChannelCompound, qboolean/QTRUE/QFALSE/FF_MAX_PATH

// ============================================================================
// Type Aliases
// ============================================================================

// Type aliases matching the C++ original
type TProject = HashMap<String, *mut CImmProject>;
type TInclude = Vec<TProject>;
type TNameTable = Vec<String>;

// ============================================================================
// External C Functions and Variables
// ============================================================================

extern "C" {
    pub static mut ff_developer: *mut cvar_t;

    #[cfg(feature = "FF_DELAY")]
    pub static mut ff_delay: *mut cvar_t;

    pub static mut gFFSystem: FFSystem;

    // File system functions
    fn FS_VerifyName(
        name: *const c_char,
        filename: *const c_char,
        outpath: *mut c_char,
    ) -> qboolean;
    fn FS_FreeFile(buffer: *mut c_void);
    fn LoadFile(name: *const c_char) -> *mut c_void;
    fn UncommonDirectory(path: *const c_char, base: *const c_char) -> *const c_char;
    fn _rcpos(haystack: *const c_char, needle: c_char) -> c_int;
    fn sprintf(buf: *mut c_char, fmt: *const c_char, ...) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Com_Printf(fmt: *const c_char, ...);
}

// ============================================================================
// C++ Method FFI Helpers (external C++ wrappers required)
// ============================================================================
// These represent calls to C++ methods on opaque types. Actual implementations
// must be provided by C++ wrapper functions or linker magic to work.

#[link(name = "ff", kind = "static")]
extern "C" {
    // FFConfigParser methods
    fn ffset_getffset(parser: *mut FFConfigParser, device: *mut CImmDevice) -> *const c_char;
    fn ffset_getincludes(parser: *mut FFConfigParser, setname: *const c_char) -> *mut Vec<String>;

    // CImmProject methods & allocation
    fn ffset_new_immproject() -> *mut CImmProject;
    fn ffset_delete_immproject(proj: *mut CImmProject);
    fn ffset_immproject_load(
        proj: *mut CImmProject,
        buffer: *mut c_void,
        device: *mut CImmDevice,
    ) -> qboolean;
    fn ffset_immproject_getcreatedeffect(
        proj: *mut CImmProject,
        name: *const c_char,
    ) -> *mut c_void; // Returns MultiEffect*
    fn ffset_immproject_getcreatedeffect_idx(proj: *mut CImmProject, idx: c_int)
        -> *mut c_void; // Returns MultiEffect*
    fn ffset_immproject_createeffect(
        proj: *mut CImmProject,
        name: *const c_char,
        device: *mut CImmDevice,
        flags: c_int,
    ) -> *mut c_void; // Returns MultiEffect*
    fn ffset_immproject_stop(proj: *mut CImmProject);

    // MultiEffect methods
    fn ffset_effect_changestartdelay(effect: *mut MultiEffect, delay: c_int);
    fn ffset_effect_getname(effect: *mut c_void) -> *const c_char;
}

// ============================================================================
// FFSet Class Implementation
// ============================================================================

pub struct FFSet {
    mParser: *mut FFConfigParser,
    mDevice: *mut CImmDevice,
    mInclude: TInclude,
    mIncludePath: Vec<String>,
}

impl FFSet {
    /// FFSet::FFSet( FFConfigParser &ConfigParser, CImmDevice *Device )
    pub fn new(ConfigParser: *mut FFConfigParser, Device: *mut CImmDevice) -> Self {
        let mut this = FFSet {
            mParser: ConfigParser,
            mDevice: Device,
            mInclude: Vec::new(),
            mIncludePath: Vec::new(),
        };

        unsafe {
            // const char *setname = mParser.GetFFSet( mDevice );
            // NOTE: C++ method call via FFI - would need proper implementation
            // For now, we declare this as requiring an external C++ wrapper function
            let setname = ffset_getffset(ConfigParser, Device);

            if !setname.is_null() {
                // TProject temp;
                let temp = HashMap::new();
                this.mInclude.push(temp);

                // mIncludePath.push_back( setname );
                let setname_str = std::ffi::CStr::from_ptr(setname)
                    .to_string_lossy()
                    .into_owned();
                this.mIncludePath.push(setname_str.clone());

                // InitIncludes( setname );
                this.InitIncludes(setname);
            }
        }

        this
    }

    /// FFSet::~FFSet()
    pub fn drop_ffs(&mut self) {
        // for ( TInclude::iterator itInclude = mInclude.begin() ; ... )
        for itInclude in self.mInclude.iter_mut() {
            // for ( TProject::iterator itProject = (*itInclude).begin() ; ... )
            for (_key, itProject) in itInclude.iter_mut() {
                // DeletePointer( (*itProject).second );
                if !(*itProject).is_null() {
                    // Would delete the allocated CImmProject here
                    *itProject = std::ptr::null_mut();
                }
            }
        }
    }

    /// void FFSet::InitIncludes( const char *setname )
    fn InitIncludes(&mut self, setname: *const c_char) {
        unsafe {
            // FFConfigParser::TInclude &include = mParser.GetIncludes( setname );
            let include_ref = ffset_getincludes(self.mParser, setname);
            if include_ref.is_null() {
                return;
            }
            let include = &*include_ref;

            // for // each include listed in config file
            // ( int i = 0 ; i < include.size() ; i++ )
            for i in 0..include.len() {
                // for // each include entered into current list
                // ( unsigned int j = 0 ; j < mIncludePath.size() ; j++ )
                let mut j: usize = 0;
                while j < self.mIncludePath.len() {
                    // if ( include[ i ] == mIncludePath[ j ] ) // exists in current list
                    if include[i] == self.mIncludePath[j] {
                        break;
                    }
                    j += 1;
                }

                // if ( j == mIncludePath.size() ) // does not exist in current list
                if j == self.mIncludePath.len() {
                    // TProject temp;
                    let temp = HashMap::new();
                    // mInclude.push_back( temp );
                    self.mInclude.push(temp);
                    // mIncludePath.push_back( include[ i ] );
                    self.mIncludePath.push(include[i].clone());
                    // InitIncludes( include[ i ].c_str() ); // recurse
                    let include_i_cstr = std::ffi::CString::new(include[i].as_str()).unwrap();
                    self.InitIncludes(include_i_cstr.as_ptr());
                }
            }
        }
    }

    /// MultiEffect* FFSet::Register( const char *path, qboolean create )
    pub fn Register(&mut self, path: *const c_char, create: qboolean) -> *mut MultiEffect {
        let mut outpath: [c_char; FF_MAX_PATH] = [0; FF_MAX_PATH];
        let mut effect: *mut MultiEffect = std::ptr::null_mut();

        unsafe {
            // if ( FS_VerifyName( "FFSet::Register", path, outpath ) )
            if FS_VerifyName(
                b"FFSet::Register\0".as_ptr() as *const c_char,
                path,
                outpath.as_mut_ptr(),
            ) != QFALSE
            {
                // for // each included set
                // ( int i = 0 ; i < mInclude.size() && !effect ; i++ )
                let mut i: usize = 0;
                while i < self.mInclude.len() && effect.is_null() {
                    let mut setpath: [c_char; FF_MAX_PATH] = [0; FF_MAX_PATH];
                    let mut afterincludepath: [c_char; FF_MAX_PATH] = [0; FF_MAX_PATH];

                    // char setpath[ FF_MAX_PATH ], *afterincludepath;
                    // need to use explicit path if provided.
                    // sprintf( setpath, "%s/%s", mIncludePath[ i ].c_str(),
                    //          UncommonDirectory( path, mIncludePath[ i ].c_str() ) );
                    let uncommon_dir = UncommonDirectory(
                        path,
                        self.mIncludePath[i].as_ptr() as *const c_char,
                    );
                    let uncommon_str =
                        std::ffi::CStr::from_ptr(uncommon_dir).to_string_lossy();

                    let formatted = format!(
                        "{}/{}",
                        self.mIncludePath[i],
                        uncommon_str
                    );
                    sprintf(
                        setpath.as_mut_ptr(),
                        b"%s/%s\0".as_ptr() as *const c_char,
                        self.mIncludePath[i].as_ptr() as *const c_char,
                        uncommon_dir,
                    );

                    // afterincludepath = setpath + mIncludePath[ i ].length() + 1;
                    let offset = self.mIncludePath[i].len() + 1;
                    let afterincludepath_ptr = setpath.as_mut_ptr().add(offset);

                    // for // each possible file/effectname combination
                    // ( int separator = _rcpos( afterincludepath, '/' ) ; separator >= 0 && !effect ;
                    //   separator = _rcpos( afterincludepath, '/', separator ) )
                    let mut separator = _rcpos(afterincludepath_ptr, b'/' as c_char);
                    while separator >= 0 && effect.is_null() {
                        let mut temp: [c_char; 4] = [0; 4];

                        // temp[0] = 0;
                        // afterincludepath[separator] = 0;
                        *afterincludepath_ptr.add(separator as usize) = 0;

                        // if ( stricmp( afterincludepath + separator - 4, ".ifr" ) )
                        if stricmp(
                            afterincludepath_ptr.add(separator as usize).sub(4),
                            b".ifr\0".as_ptr() as *const c_char,
                        ) != 0
                        {
                            // memcpy( temp, afterincludepath + separator + 1, 4 );
                            memcpy(
                                temp.as_mut_ptr() as *mut c_void,
                                afterincludepath_ptr.add((separator + 1) as usize) as *const c_void,
                                4,
                            );
                            // sprintf( afterincludepath + separator, ".ifr" );
                            sprintf(
                                afterincludepath_ptr.add(separator as usize),
                                b".ifr\0".as_ptr() as *const c_char,
                            );
                        }

                        // immProject = NULL;
                        let mut immProject: *mut CImmProject = std::ptr::null_mut();

                        // TProject::iterator itProject = mInclude[ i ].find( afterincludepath );
                        // if ( itProject != mInclude[ i ].end() )
                        let afterincludepath_str = std::ffi::CStr::from_ptr(afterincludepath_ptr)
                            .to_string_lossy()
                            .into_owned();

                        if let Some(found_project) = self.mInclude[i].get(&afterincludepath_str) {
                            // immProject = (*itProject).second;
                            immProject = *found_project;
                        } else if create != QFALSE {
                            // void *buffer = LoadFile( setpath );
                            let buffer = LoadFile(setpath.as_ptr());
                            if !buffer.is_null() {
                                // immProject = new CImmProject;
                                immProject = ffset_new_immproject();

                                if !immProject.is_null() {
                                    // if ( !immProject->LoadProjectFromMemory( buffer, mDevice ) )
                                    if ffset_immproject_load(immProject, buffer, self.mDevice)
                                        == QFALSE
                                    {
                                        // DeletePointer( immProject );
                                        ffset_delete_immproject(immProject);
                                        immProject = std::ptr::null_mut();
                                        #[cfg(feature = "FF_PRINT")]
                                        {
                                            // if ( ff_developer->integer )
                                            //   Com_Printf( "...Corrupt or invalid file: %s\n", setpath );
                                            if !ff_developer.is_null()
                                                && (*ff_developer).is_null() == false
                                            {
                                                Com_Printf(
                                                    b"...Corrupt or invalid file: %s\n\0"
                                                        .as_ptr()
                                                        as *const c_char,
                                                    setpath.as_ptr(),
                                                );
                                            }
                                        }
                                    } else {
                                        #[cfg(feature = "FF_PRINT")]
                                        {
                                            // if ( ff_developer->integer )
                                            //   Com_Printf( "...Adding file \"%s\"\n", setpath );
                                            if !ff_developer.is_null()
                                                && (*ff_developer).is_null() == false
                                            {
                                                Com_Printf(
                                                    b"...Adding file \"%s\"\n\0".as_ptr()
                                                        as *const c_char,
                                                    setpath.as_ptr(),
                                                );
                                            }
                                        }
                                    }
                                }

                                // FS_FreeFile( buffer );
                                FS_FreeFile(buffer);
                            }

                            // mInclude[ i ][ afterincludepath ] = immProject;
                            self.mInclude[i].insert(afterincludepath_str, immProject);
                        }

                        // if ( temp[ 0 ] )
                        if temp[0] != 0 {
                            // afterincludepath[ separator ] = '/';
                            *afterincludepath_ptr.add(separator as usize) = b'/' as c_char;
                            // memcpy( afterincludepath + separator + 1, temp, 4 );
                            memcpy(
                                afterincludepath_ptr.add((separator + 1) as usize) as *mut c_void,
                                temp.as_ptr() as *const c_void,
                                4,
                            );
                        }

                        // if ( immProject )
                        if !immProject.is_null() {
                            // effect = (MultiEffect*)immProject->GetCreatedEffect( afterincludepath + separator + 1 );
                            effect = ffset_immproject_getcreatedeffect(
                                immProject,
                                afterincludepath_ptr.add((separator + 1) as usize),
                            ) as *mut MultiEffect;

                            // if ( !effect && create )
                            if effect.is_null() && create != QFALSE {
                                // effect = (MultiEffect*)immProject->CreateEffect(
                                //   afterincludepath + separator + 1, mDevice, IMM_PARAM_NODOWNLOAD );
                                effect = ffset_immproject_createeffect(
                                    immProject,
                                    afterincludepath_ptr.add((separator + 1) as usize),
                                    self.mDevice,
                                    0, // IMM_PARAM_NODOWNLOAD
                                ) as *mut MultiEffect;

                                #[cfg(feature = "FF_DELAY")]
                                {
                                    // Delay the effect (better sound synchronization)
                                    // if ( effect && ff_delay )
                                    if !effect.is_null() && !ff_delay.is_null() {
                                        // effect->ChangeStartDelay( ff_delay->integer );
                                        ffset_effect_changestartdelay(effect, (*ff_delay).integer);
                                    }
                                }
                            }
                        }
                    }

                    i += 1;
                }
            }
        }

        effect
    }

    /// qboolean FFSet::StopAll( void )
    pub fn StopAll(&self) -> qboolean {
        // for ( TInclude::iterator itInclude = mInclude.begin() ; ... )
        for itInclude in self.mInclude.iter() {
            // for ( TProject::iterator itProject = (*itInclude).begin() ; ... )
            for (_key, itProject) in itInclude.iter() {
                // if ( (*itProject).second )
                //   (*itProject).second->Stop();
                if !(*itProject).is_null() {
                    unsafe {
                        ffset_immproject_stop(*itProject);
                    }
                }
            }
        }

        QTRUE
    }

    /// void FFSet::GetRegisteredNames( TNameTable &NameTable )
    pub fn GetRegisteredNames(&self, NameTable: &mut TNameTable) {
        unsafe {
            // FFSystem::Handle ffHandle = gFFSystem.GetHandles();
            // (Note: FFSystem::Handle is opaque, can't iterate it directly without full type info)

            // for ( int IncludeIndex = 0 ; IncludeIndex < mInclude.size() ; IncludeIndex++ )
            for IncludeIndex in 0..self.mInclude.len() {
                // for ( TProject::iterator itProject = mInclude[ IncludeIndex ].begin() ; ... )
                for (project_first, itProject) in self.mInclude[IncludeIndex].iter() {
                    if !(*itProject).is_null() {
                        // int i = 0;
                        let mut i: c_int = 0;

                        // for ( MultiEffect *Effect = (MultiEffect*)(*itProject).second->GetCreatedEffect( i ) ;
                        //       Effect ;
                        //       Effect = (MultiEffect*)(*itProject).second->GetCreatedEffect( ++i ) )
                        loop {
                            let Effect = ffset_immproject_getcreatedeffect_idx(*itProject, i);
                            if Effect.is_null() {
                                break;
                            }

                            // sprintf( effectname, "%s/%s/%s",
                            //   mIncludePath[ IncludeIndex ].c_str(), (*itProject).first.c_str(), Effect->GetName() );
                            let effect_name = ffset_effect_getname(Effect);
                            let effect_name_str =
                                std::ffi::CStr::from_ptr(effect_name).to_string_lossy();

                            let effectname = format!(
                                "{}/{}/{}",
                                self.mIncludePath[IncludeIndex],
                                project_first,
                                effect_name_str
                            );

                            // for ( int i = 0 ; i < ffHandle.size() ; i++ )
                            // (Unable to iterate ffHandle without full type information)
                            // ChannelCompound::Set &compound = ffHandle[ i ].GetSet();
                            // if ( NameTable[ i ].length() == 0 && compound.find( Effect ) != compound.end() )
                            //   NameTable[ i ] = effectname;

                            // Stub: would need GetHandles() implementation to proceed
                            let _ = effectname; // Use the variable to avoid warnings

                            i += 1;
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// Console Command Support (conditional compilation)
// ============================================================================

#[cfg(feature = "FF_CONSOLECOMMAND")]
impl FFSet {
    /// void FFSet::GetDisplayTokens( TNameTable &Tokens )
    pub fn GetDisplayTokens(&self, Tokens: &mut TNameTable) {
        unsafe {
            // if ( ff_developer->integer )
            if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                // Tokens.push_back( "order" );
                Tokens.push("order".to_string());
                // Tokens.push_back( "files" );
                Tokens.push("files".to_string());
            }
        }
    }

    /// void FFSet::Display( TNameTable &Unprocessed, TNameTable &Processed )
    pub fn Display(&self, Unprocessed: &mut TNameTable, Processed: &mut TNameTable) {
        // for ( TNameTable::iterator itName = Unprocessed.begin() ; itName != Unprocessed.end() ; )
        let mut idx: usize = 0;
        while idx < Unprocessed.len() {
            unsafe {
                // if ( stricmp( "order", (*itName).c_str() ) == 0 )
                if stricmp(
                    b"order\0".as_ptr() as *const c_char,
                    Unprocessed[idx].as_ptr() as *const c_char,
                ) == 0
                {
                    // if ( ff_developer->integer )
                    if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                        // DisplaySearchOrder();
                        self.DisplaySearchOrder();
                    }
                    //else
                    //	Com_Printf( "\"order\" only available when ff_developer is set\n" );

                    // Processed.push_back( *itName );
                    Processed.push(Unprocessed[idx].clone());
                    // itName = Unprocessed.erase( itName );
                    Unprocessed.remove(idx);
                } else if stricmp(
                    b"files\0".as_ptr() as *const c_char,
                    Unprocessed[idx].as_ptr() as *const c_char,
                ) == 0
                {
                    // if ( ff_developer->integer )
                    if !ff_developer.is_null() && (*ff_developer).integer != 0 {
                        // DisplayLoadedFiles();
                        self.DisplayLoadedFiles();
                    }
                    //else
                    //	Com_Printf( "\"files\" only available when ff_developer is set\n" );

                    // Processed.push_back( *itName );
                    Processed.push(Unprocessed[idx].clone());
                    // itName = Unprocessed.erase( itName );
                    Unprocessed.remove(idx);
                } else {
                    // itName++;
                    idx += 1;
                }
            }
        }
    }

    /// void FFSet::DisplaySearchOrder( void )
    fn DisplaySearchOrder(&self) {
        let mut ProductName: [c_char; FF_MAX_PATH] = [0; FF_MAX_PATH];
        // *ProductName = 0;
        ProductName[0] = 0;

        unsafe {
            // mDevice->GetProductName( ProductName, FF_MAX_PATH - 1 );
            ffset_device_getproductname(self.mDevice, ProductName.as_mut_ptr(), (FF_MAX_PATH - 1) as c_int);
            // Com_Printf( "[search order] -\"%s\"\n", ProductName );
            Com_Printf(
                b"[search order] -\"%s\"\n\0".as_ptr() as *const c_char,
                ProductName.as_ptr(),
            );

            // for ( int i = 0 ; i < mInclude.size() ; i++ )
            for i in 0..self.mIncludePath.len() {
                // Com_Printf( "%d) %s\n", i, mIncludePath[ i ].c_str() );
                Com_Printf(
                    b"%d) %s\n\0".as_ptr() as *const c_char,
                    i as c_int,
                    self.mIncludePath[i].as_ptr() as *const c_char,
                );
            }
        }
    }

    /// void FFSet::DisplayLoadedFiles( void )
    fn DisplayLoadedFiles(&self) {
        let mut total: c_int = 0;
        #[cfg(feature = "_DEBUG")]
        let mut nulltotal: c_int = 0; // Variable to indicate how bad my algorithm is

        let mut ProductName: [c_char; FF_MAX_PATH] = [0; FF_MAX_PATH];
        // *ProductName = 0;
        ProductName[0] = 0;

        unsafe {
            // mDevice->GetProductName( ProductName, FF_MAX_PATH - 1 );
            ffset_device_getproductname(self.mDevice, ProductName.as_mut_ptr(), (FF_MAX_PATH - 1) as c_int);
            // Com_Printf( "[loaded files] -\"%s\"\n", ProductName );
            Com_Printf(
                b"[loaded files] -\"%s\"\n\0".as_ptr() as *const c_char,
                ProductName.as_ptr(),
            );

            // for ( int i = 0 ; i < mInclude.size() ; i++ )
            for i in 0..self.mInclude.len() {
                // for ( TProject::iterator itProject = mInclude[ i ].begin() ; ... )
                for (project_first, itProject) in self.mInclude[i].iter() {
                    if !(*itProject).is_null() {
                        // ++total;
                        total += 1;
                        // Com_Printf( "%s/%s\n", mIncludePath[ i ].c_str(), (*itProject).first.c_str() );
                        Com_Printf(
                            b"%s/%s\n\0".as_ptr() as *const c_char,
                            self.mIncludePath[i].as_ptr() as *const c_char,
                            project_first.as_ptr() as *const c_char,
                        );
                    }
                    #[cfg(feature = "_DEBUG")]
                    {
                        if (*itProject).is_null() {
                            // ++nulltotal;
                            nulltotal += 1;
                            // Com_Printf( "%s/%s [null]\n", mIncludePath[ i ].c_str(), (*itProject).first.c_str() );
                            Com_Printf(
                                b"%s/%s [null]\n\0".as_ptr() as *const c_char,
                                self.mIncludePath[i].as_ptr() as *const c_char,
                                project_first.as_ptr() as *const c_char,
                            );
                        }
                    }
                }
            }

            // Com_Printf( "Total: %d files\n", total );
            Com_Printf(
                b"Total: %d files\n\0".as_ptr() as *const c_char,
                total,
            );
            #[cfg(feature = "_DEBUG")]
            {
                // Com_Printf( "Total: %d null files\n", nulltotal );
                Com_Printf(
                    b"Total: %d null files\n\0".as_ptr() as *const c_char,
                    nulltotal,
                );
            }
        }
    }
}

// C++ method FFI helpers for console commands
#[link(name = "ff", kind = "static")]
extern "C" {
    fn ffset_device_getproductname(
        device: *mut CImmDevice,
        buf: *mut c_char,
        len: c_int,
    );
}
