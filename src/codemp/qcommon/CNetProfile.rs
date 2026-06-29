//Anything above this #include will be ignored by the compiler
//#include "../qcommon/exe_headers.h"

#![allow(non_snake_case)]

#[cfg(feature = "donetprofile")]
mod cnetprofile_impl {
    use std::collections::{HashMap, BTreeMap};
    use core::ffi::c_char;

    // class CNetProfile : public INetProfile
    // {
    //     float						mElapsedTime;
    //     map <hstring,unsigned int>	mFieldCounts;
    //     float						mFrameCount;
    // public:
    pub struct CNetProfile {
        mElapsedTime: f32,
        mFieldCounts: HashMap<String, u32>,
        mFrameCount: f32,
    }

    impl CNetProfile {
        fn new() -> Self {
            CNetProfile {
                mElapsedTime: 0.0,
                mFieldCounts: HashMap::new(),
                mFrameCount: 0.0,
            }
        }

        // void Reset(void)
        // {
        //     mFieldCounts.clear();
        //     mFrameCount=0;
        // }
        fn Reset(&mut self) {
            self.mFieldCounts.clear();
            self.mFrameCount = 0.0;
        }

        // void AddField(char *fieldName,int sizeBytes)
        // {
        //     assert(sizeBytes>=0);
        //     if(sizeBytes==0)
        //     {
        //         return;
        //     }
        //     map<hstring,unsigned int>::iterator f=mFieldCounts.find(fieldName);
        //     if(f==mFieldCounts.end())
        //     {
        //         mFieldCounts[fieldName]=(unsigned int)sizeBytes;
        //     }
        //     else
        //     {
        //         mFieldCounts[fieldName]+=(unsigned int)sizeBytes;
        //     }
        // }
        fn AddField(&mut self, fieldName: *const c_char, sizeBytes: i32) {
            assert!(sizeBytes >= 0);
            if sizeBytes == 0 {
                return;
            }
            let field_name_str = unsafe {
                std::ffi::CStr::from_ptr(fieldName)
                    .to_string_lossy()
                    .into_owned()
            };
            let f = self.mFieldCounts.get(&field_name_str);
            if f.is_none() {
                self.mFieldCounts.insert(field_name_str, sizeBytes as u32);
            } else {
                *self.mFieldCounts.get_mut(&field_name_str).unwrap() += sizeBytes as u32;
            }
        }

        // void IncTime(int msec)
        // {
        //     mElapsedTime+=msec;
        // }
        fn IncTime(&mut self, msec: i32) {
            self.mElapsedTime += msec as f32;
        }

        // void ShowTotals(void)
        // {
        //     float									totalBytes=0;
        //     multimap<unsigned int,hstring>			sort;
        //     map<hstring,unsigned int>::iterator		f;
        //     for(f=mFieldCounts.begin();f!=mFieldCounts.end();f++)
        //     {
        //         sort.insert(pair<unsigned int,hstring> ((*f).second,(*f).first));
        //         totalBytes+=(*f).second;
        //     }
        //
        //     multimap<unsigned int,hstring>::iterator	j;
        //     char										msg[1024];
        //     float										percent;
        //     sprintf(msg,
        //         "******** Totals: bytes %d : bytes per sec %d ********\n",
        //         (unsigned int)totalBytes,
        //         (unsigned int)((totalBytes/mElapsedTime)*1000));
        //     Sleep(10);
        //     OutputDebugString(msg);
        //     for(j=sort.begin();j!=sort.end();j++)
        //     {
        //         percent=(((float)(*j).first)/totalBytes)*100.0f;
        //         assert(strlen((*j).second.c_str())<1024);
        //         sprintf(msg,"%36s : %3.4f percent : %d bytes \n",(*j).second.c_str(),percent,(*j).first);
        //         Sleep(10);
        //         OutputDebugString(msg);
        //     }
        // }
        fn ShowTotals(&self) {
            let mut total_bytes: f32 = 0.0;
            let mut sort: BTreeMap<u32, String> = BTreeMap::new();
            for (f_key, f_val) in &self.mFieldCounts {
                sort.insert(*f_val, f_key.clone());
                total_bytes += *f_val as f32;
            }

            let msg = format!(
                "******** Totals: bytes {} : bytes per sec {} ********\n",
                total_bytes as u32,
                ((total_bytes / self.mElapsedTime) * 1000.0) as u32
            );
            #[cfg(target_os = "windows")]
            {
                std::thread::sleep(std::time::Duration::from_millis(10));
                // OutputDebugString(msg.as_ptr());
                eprintln!("{}", msg);
            }
            for (j_first, j_second) in sort.iter().rev() {
                let percent = ((*j_first as f32) / total_bytes) * 100.0;
                assert!(j_second.len() < 1024);
                let msg = format!(
                    "{:36} : {:3.4} percent : {} bytes \n",
                    j_second, percent, j_first
                );
                #[cfg(target_os = "windows")]
                {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    // OutputDebugString(msg.as_ptr());
                    eprintln!("{}", msg);
                }
            }
        }
    }

    // INetProfile &ClReadProf(void)
    // {
    //     static CNetProfile theClReadProf;
    //     return(theClReadProf);
    // }
    pub fn ClReadProf() -> &'static mut CNetProfile {
        static mut THE_CL_READ_PROF: Option<CNetProfile> = None;
        unsafe {
            THE_CL_READ_PROF.get_or_insert_with(CNetProfile::new)
        }
    }

    // INetProfile &ClSendProf(void)
    // {
    //     static CNetProfile theClSendProf;
    //     return(theClSendProf);
    // }
    pub fn ClSendProf() -> &'static mut CNetProfile {
        static mut THE_CL_SEND_PROF: Option<CNetProfile> = None;
        unsafe {
            THE_CL_SEND_PROF.get_or_insert_with(CNetProfile::new)
        }
    }
}

// #endif // _DONETPROFILE_
#[cfg(feature = "donetprofile")]
pub use cnetprofile_impl::*;
