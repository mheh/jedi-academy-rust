// Filename:-	ui_debug.rs
//
// an entire temp module just for doing some evil menu hackery during development...
//

#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

// #if 0	// this entire module was special code to StripEd-ify *menu, it isn't needed now, but I'll keep the source around for a while	-ste

#[cfg(all(debug_assertions, false))]
mod ui_debug_impl {
    use std::collections::{HashMap, HashSet, LinkedList};
    use std::ffi::CStr;
    use std::ptr;
    use core::ffi::{c_char, c_int, c_void};

    // Stub type aliases for sstring classes
    type sstring_t = String;
    type sstringBIG_t = String;
    type StringSet_t = HashSet<String>;
    type ReferencesAndPackages_t = HashMap<String, StringSet_t>;

    // Stub for sstring implementation
    type References_t = LinkedList<Reference_t>;

    // #ifdef _DEBUG

    // typedef sstring<4096> sstringBIG_t;
    // typedef set<sstring_t>	StringSet_t;
    static mut MenusUsed: Option<StringSet_t> = None;
    // typedef map <sstring_t,	StringSet_t>	ReferencesAndPackages_t;
    static mut ReferencesAndPackage: Option<ReferencesAndPackages_t> = None;

    #[repr(C)]
    struct Reference_t {
        sString: sstringBIG_t,
        sReference: sstring_t,
        sMenu: sstring_t,
    }

    impl Reference_t {
        fn new() -> Self {
            Reference_t {
                sString: String::new(),
                sReference: String::new(),
                sMenu: String::new(),
            }
        }

        // sort by menu entry, then by reference within menu...
        //
        fn lt(&self, _X: &Reference_t) -> bool {
            let i = stricmp(&self.sMenu, &_X.sMenu);
            if i != 0 {
                return i < 0;
            }

            stricmp(&self.sReference, &_X.sReference) < 0
        }
    }

    type References_t = LinkedList<Reference_t>;
    static mut BadReferences: Option<References_t> = None;

    static mut sCurrentMenu: sstring_t = String::new();

    fn UI_Debug_AddMenuFilePath(psMenuFile: &str) {
        // eg "ui/blah.menu"
        unsafe {
            sCurrentMenu = psMenuFile.to_string();

            OutputDebugString(&format!("Current menu: \"{}\"\n", psMenuFile));
        }
    }

    #[repr(C)]
    struct CorrectionDataItem_t {
        // to correct...
        //
        sMenuFile: sstring_t,
        sTextToFind: sstringBIG_t,        // will be either @REFERENCE or "text"
        sTextToReplaceWith: sstringBIG_t, // will be @NEWREF
        //
        // to generate new data...
        //
        sStripEdReference: sstring_t,   // when NZ, this will create a new StripEd entry...
        sStripEdText: sstringBIG_t,
        sStripEdFileRef: sstring_t, // ... in this file reference (eg "SPMENUS%d"), where 0.255 of each all have this in them (for ease of coding)
    }

    type CorrectionData_t = LinkedList<CorrectionDataItem_t>;
    static mut CorrectionData: Option<CorrectionData_t> = None;

    fn CreateUniqueReference(psText: &str) -> String {
        static mut ReferencesSoFar: Option<HashSet<String>> = None;

        static mut NewReference: sstring_t = String::new();

        let mut psTextScanPos = psText;

        loop {
            while psTextScanPos.len() > 0 && psTextScanPos.chars().next().unwrap().is_whitespace() {
                psTextScanPos = &psTextScanPos[1..];
            }

            unsafe {
                NewReference = psTextScanPos.to_string();
            }

            // cap off text at an approx length...
            //
            const iApproxReferenceLength: usize = 20;
            unsafe {
                if iApproxReferenceLength < NewReference.len() {
                    let mut truncated = String::new();
                    for (i, c) in NewReference.chars().enumerate() {
                        if i >= iApproxReferenceLength {
                            break;
                        }
                        truncated.push(c);
                    }
                    // find the next space
                    for (i, c) in truncated[iApproxReferenceLength..].chars().enumerate() {
                        if c.is_whitespace() {
                            NewReference.truncate(iApproxReferenceLength + i);
                            break;
                        }
                    }
                }

                // now replace everything except digits and letters with underscores...
                //
                let mut p: Vec<u8> = NewReference.as_bytes().to_vec();
                for i in 0..p.len() {
                    if !((p[i] as char).is_alphabetic() || (p[i] as char).is_numeric()) {
                        p[i] = b'_';
                    }
                }
                NewReference = String::from_utf8(p).unwrap_or_default();
                NewReference = NewReference.to_uppercase();

                // remove any trailing underscores...
                //
                while NewReference.len() > 0 && NewReference.as_bytes()[NewReference.len() - 1] == b'_' {
                    NewReference.pop();
                }

                // remove any multiple underscores...
                //
                loop {
                    if let Some(pos) = NewReference.find("__") {
                        NewReference.remove(pos);
                    } else {
                        break;
                    }
                }

                // do we already have this reference?...
                //
                if NewReference.is_empty() {
                    break; // empty, shit.
                }

                if ReferencesSoFar.is_none() {
                    ReferencesSoFar = Some(HashSet::new());
                }

                if !ReferencesSoFar.as_ref().unwrap().contains(&NewReference) {
                    // no, so add it in then return...
                    //
                    ReferencesSoFar.as_mut().unwrap().insert(NewReference.clone());
                    return NewReference;
                }

                // skip past the first word in the reference and try again...
                //
                while psTextScanPos.len() > 0
                    && !psTextScanPos.chars().next().unwrap().is_whitespace()
                {
                    psTextScanPos = &psTextScanPos[1..];
                }
            }
        }

        // if we get here then we're getting desperate, so...
        //
        // (special case check first)...
        //
        let mut p2 = psText;
        while p2.len() > 0 && p2.chars().next().unwrap().is_whitespace() {
            p2 = &p2[1..];
        }

        let mut psText = psText;
        if p2.is_empty() {
            psText = "BLANK";
        }

        let mut iReScanDigit: c_int = 1;
        loop {
            unsafe {
                NewReference = format!("{}_{}", psText, iReScanDigit);
                iReScanDigit += 1;

                // now replace everything except digits and letters with underscores...
                //
                let mut p: Vec<u8> = NewReference.as_bytes().to_vec();
                for i in 0..p.len() {
                    if !((p[i] as char).is_alphabetic() || (p[i] as char).is_numeric()) {
                        p[i] = b'_';
                    }
                }
                NewReference = String::from_utf8(p).unwrap_or_default();
                NewReference = NewReference.to_uppercase();

                // remove any trailing underscores...
                //
                while NewReference.len() > 0 && NewReference.as_bytes()[NewReference.len() - 1] == b'_'
                {
                    NewReference.pop();
                }

                // remove any multiple underscores...
                //
                loop {
                    if let Some(pos) = NewReference.find("__") {
                        NewReference.remove(pos);
                    } else {
                        break;
                    }
                }

                if ReferencesSoFar.is_none() {
                    ReferencesSoFar = Some(HashSet::new());
                }

                if !ReferencesSoFar.as_ref().unwrap().contains(&NewReference) {
                    // no, so add it in then return...
                    //
                    ReferencesSoFar.as_mut().unwrap().insert(NewReference.clone());
                    return NewReference;
                }
            }
        }

        // should never get here...
        //
        // assert(0);
        String::new()
    }

    fn EnterBadRef(ps4LetterType: &str, psBad: &str) -> String {
        let mut BadReference = Reference_t::new();
        BadReference.sString = psBad.to_string();
        unsafe {
            BadReference.sMenu = sCurrentMenu.clone();

            if BadReferences.is_none() {
                BadReferences = Some(LinkedList::new());
            }
            BadReferences.as_mut().unwrap().push_back(BadReference);
        }

        let mut p: String = String::new();
        //			p = CreateUniqueReference(psBad);
        //	OutputDebugString(va("NEWREF: \"%s\"\n",p));
        p
    }

    fn EnterRef(psReference: &str, psText: &str, psMenuFile: &str) {
        // special hack, StripEd text at this point will have had any "\n" LITERALS replaced by the 0x0D 0x0A pair,
        //	so we need to put them back to "\n" text ready for saving out into StripEd files again...
        //
        let mut strNewText = psText.to_string();
        //
        // not sure whether just 0x0A or 0x0D/0x0A pairs (sigh), so first, eliminate 0x0Ds...
        //
        loop {
            if let Some(iLoc) = strNewText.find('\r') {
                strNewText.remove(iLoc);
            } else {
                break;
            }
        }
        // now replace any 0x0As with literal "\n" strings...
        //
        loop {
            if let Some(iLoc) = strNewText.find('\n') {
                strNewText.replace_range(iLoc..iLoc + 1, "\\n");
            } else {
                break;
            }
        }
        let psText = strNewText.as_str();

        // curiousity...
        //
        static mut iLongestText: c_int = 0;
        unsafe {
            if iLongestText < psText.len() as c_int {
                iLongestText = psText.len() as c_int;
                OutputDebugString(&format!("Longest StripEd text: {}\n", iLongestText));
            }
        }

        type TextConsolidationTable_t = HashMap<String, String>; // string and ref
        static mut TextConsolidationTable: Option<TextConsolidationTable_t> = None;
        static mut RefrConsolidationTable: Option<TextConsolidationTable_t> = None;
        static mut iIndex: c_int = 0; // INC'd every time a new StripEd entry is synthesised

        unsafe {
            if TextConsolidationTable.is_none() {
                TextConsolidationTable = Some(HashMap::new());
            }
            if RefrConsolidationTable.is_none() {
                RefrConsolidationTable = Some(HashMap::new());
            }

            let TextConsolidationTable_ref = TextConsolidationTable.as_ref().unwrap();

            if TextConsolidationTable_ref.get(psText).is_none() {
                // new entry...
                //
                let psNewReference = if psReference.len() > psText.len() {
                    CreateUniqueReference(psReference)
                } else {
                    CreateUniqueReference(psText)
                };

                let mut CorrectionDataItem = CorrectionDataItem_t {
                    sMenuFile: psMenuFile.to_string(),
                    sTextToFind: if psReference.len() > 0 {
                        format!("@{}", psReference)
                    } else {
                        format!("\"{}\"", psText)
                    },
                    sTextToReplaceWith: format!("@{}", psNewReference),
                    //
                    sStripEdReference: psNewReference.clone(),
                    sStripEdText: psText.to_string(),

                    //		qboolean bIsMulti = !!strstr(psMenuFile,"jk2mp");
                    //								CorrectionDataItem.sStripEdFileRef		= va("%sMENUS%d",bIsMulti?"MP":"SP",iIndex/256);
                    sStripEdFileRef: format!("MENUS{}", iIndex / 256),
                };
                iIndex += 1;

                if CorrectionData.is_none() {
                    CorrectionData = Some(LinkedList::new());
                }
                CorrectionData.as_mut().unwrap().push_back(CorrectionDataItem);

                TextConsolidationTable.as_mut().unwrap().insert(
                    psText.to_string(),
                    psNewReference.clone(),
                );
                RefrConsolidationTable.as_mut().unwrap().insert(
                    psText.to_string(),
                    format!("MENUS{}", (iIndex - 1) / 256),
                );
            } else {
                // text already entered, so do a little duplicate-resolving...
                //
                // need to find the reference for the existing version...
                //
                let psNewReference = TextConsolidationTable.as_ref().unwrap()
                    .get(psText)
                    .map(|s| s.clone())
                    .unwrap_or_default();
                let psPackageRef = RefrConsolidationTable.as_ref().unwrap()
                    .get(psText)
                    .map(|s| s.clone())
                    .unwrap_or_default(); // yeuch, hack-city

                // only enter correction data if references are different...
                //
                //		if (stricmp(psReference,psNewReference))
                {
                    let CorrectionDataItem = CorrectionDataItem_t {
                        sMenuFile: psMenuFile.to_string(),
                        sTextToFind: if psReference.len() > 0 {
                            format!("@{}", psReference)
                        } else {
                            format!("\"{}\"", psText)
                        },
                        sTextToReplaceWith: format!("@{}", psNewReference),
                        //
                        sStripEdReference: String::new(),
                        sStripEdText: String::new(),
                        sStripEdFileRef: psPackageRef,
                    };

                    if CorrectionData.is_none() {
                        CorrectionData = Some(LinkedList::new());
                    }
                    CorrectionData.as_mut().unwrap().push_back(CorrectionDataItem);
                }
            }
        }
    }

    fn EnterGoodRef(ps4LetterType: &str, psReference: &str, psPackageReference: &str, psText: &str) {
        unsafe {
            EnterRef(psReference, psText, sCurrentMenu.as_str());

            if ReferencesAndPackage.is_none() {
                ReferencesAndPackage = Some(HashMap::new());
            }

            ReferencesAndPackage
                .as_mut()
                .unwrap()
                .entry(psReference.to_string())
                .or_insert_with(HashSet::new)
                .insert(psPackageReference.to_string());

            if MenusUsed.is_none() {
                MenusUsed = Some(HashSet::new());
            }
            MenusUsed
                .as_mut()
                .unwrap()
                .insert(psPackageReference.to_string());
        }
    }

    fn SendFileToNotepad(psFilename: &str) -> bool {
        let mut bReturn = false;

        #[cfg(target_os = "windows")]
        {
            // Note: This is Windows-specific code that won't work on other platforms
            // The original uses WinExec which is a Windows API
            // For porting purposes, this is stubbed out
        }

        bReturn
    }

    // creates as temp file, then spawns notepad with it...
    //
    fn SendStringToNotepad(psWhatever: &str, psLocalFileName: &str) -> bool {
        let mut bReturn = false;

        #[cfg(target_os = "windows")]
        {
            let psOutputFileName = format!("c:\\{}", psLocalFileName);

            if let Ok(mut handle) = std::fs::File::create(&psOutputFileName) {
                use std::io::Write;
                if handle.write_all(psWhatever.as_bytes()).is_ok() {
                    bReturn = SendFileToNotepad(&psOutputFileName);
                }
            }
        }

        bReturn
    }

    fn DoFileFindReplace(psMenuFile: &str, psFind: &str, psReplace: &str) -> bool {
        OutputDebugString(&format!("Loading: \"{}\"\n", psMenuFile));

        // Stub: FS_ReadFile would be a game engine function
        // For now, we return false as this requires external engine integration
        false
    }

    fn UI_Dump_f() {
        let mut sFinalOutput = String::new();
        let mut vStripEdFiles: Vec<String> = Vec::new();

        unsafe {
            if ReferencesAndPackage.is_none() {
                ReferencesAndPackage = Some(HashMap::new());
            }
            if MenusUsed.is_none() {
                MenusUsed = Some(HashSet::new());
            }
            if BadReferences.is_none() {
                BadReferences = Some(LinkedList::new());
            }
            if CorrectionData.is_none() {
                CorrectionData = Some(LinkedList::new());
            }

            sFinalOutput.push_str("### UI_Dump(): Top\n");

            for (ref_key, ref_set) in ReferencesAndPackage.as_ref().unwrap().iter() {
                if ref_set.len() > 1 {
                    sFinalOutput.push_str(&format!("!!!DUP:  Ref \"{}\" exists in:\n", ref_key));
                    for package_ref in ref_set.iter() {
                        sFinalOutput.push_str(&format!("{}\n", package_ref));
                    }
                }
            }

            sFinalOutput.push_str("\nSP Package Reference list:\n");

            for menu_ref in MenusUsed.as_ref().unwrap().iter() {
                sFinalOutput.push_str(&format!("{}\n", menu_ref));
            }

            sFinalOutput.push_str("\nBad Text list:\n");

            for bad_ref in BadReferences.as_ref().unwrap().iter() {
                sFinalOutput.push_str(&format!(
                    "File: {:30}  \"{}\"\n",
                    bad_ref.sMenu, bad_ref.sString
                ));
            }

            sFinalOutput.push_str("\nAdding bad references to final correction list...\n");

            for bad_ref in BadReferences.as_ref().unwrap().iter() {
                EnterRef("", bad_ref.sString.as_str(), bad_ref.sMenu.as_str());
            }

            sFinalOutput.push_str("\nFinal correction list:\n");

            //	qboolean bIsMulti = !!strstr((*CorrectionData.begin()).sMenuFile.c_str(),"jk2mp");

            // actually do the find/replace...
            //
            for correction_item in CorrectionData.as_ref().unwrap().iter() {
                if !correction_item.sTextToFind.is_empty()
                    && !correction_item.sTextToReplaceWith.is_empty()
                {
                    sFinalOutput.push_str(&format!(
                        "Load File: \"{}\", find \"{}\", replace with \"{}\"\n",
                        correction_item.sMenuFile, correction_item.sTextToFind, correction_item.sTextToReplaceWith
                    ));

                    //			if (strstr(CorrectionDataItem.sTextToReplaceWith.c_str(),"START_A_NEW_GAME"))
                    //			{
                    //				int z=1;
                    //			}
                    // assert( CorrectionDataItem.sTextToReplaceWith.c_str()[0] );
                    let mut sReplace = correction_item.sTextToReplaceWith.clone();
                    sReplace.insert(1, '_');
                    sReplace.insert(1, correction_item.sStripEdFileRef.chars().next().unwrap_or(' '));

                    DoFileFindReplace(
                        correction_item.sMenuFile.as_str(),
                        correction_item.sTextToFind.as_str(),
                        sReplace.as_str(),
                    );
                }
            }

            // scan in all SP files into one huge string, so I can pick out any foreign translations to add in when generating
            //	new StripEd files...
            //
            // Stubs for file system operations
            let mut buffers: Vec<String> = Vec::new();
            let iNumFiles = 0;
            let mut sStripFiles = String::new();

            // scan for shader files
            // ppsFiles = FS_ListFiles( "strip", ".sp", &iNumFiles );
            // This requires integration with the game's file system

            let mut iIndex: c_int = 0;
            for correction_item in CorrectionData.as_ref().unwrap().iter() {
                if !correction_item.sStripEdReference.is_empty() {
                    // skip over duplicate-resolving entries
                    let mut strAnyForeignStringsFound = String::new(); // will be entire line plus CR
                    let mut strNotes = String::new(); // will be just the bit within quotes

                    // This section requires complex string processing from loaded files
                    // Stubbing out for now as it requires file I/O integration

                    if !strNotes.is_empty() {
                        strNotes = format!("   NOTES \"{}\"\n", strNotes);
                    }

                    // now do output...
                    //
                    if iIndex % 256 == 0 {
                        vStripEdFiles.push(String::new());

                        vStripEdFiles
                            .last_mut()
                            .unwrap()
                            .push_str(&format!(
                                "VERSION 1\nCONFIG W:\\bin\\striped.cfg\nID {}\nREFERENCE MENUS{}\nDESCRIPTION \"menu text\"\nCOUNT 256\n",
                                250 + (iIndex / 256),
                                iIndex / 256
                            ));
                    }

                    vStripEdFiles.last_mut().unwrap().push_str(&format!(
                        "INDEX {}\n{{\n   REFERENCE {}\n{}",
                        iIndex % 256,
                        correction_item.sStripEdReference,
                        if strNotes.is_empty() { "" } else { strNotes.as_str() }
                    ));
                    vStripEdFiles.last_mut().unwrap().push_str(&format!(
                        "   TEXT_LANGUAGE1 \"{}\"\n{}}}\n",
                        correction_item.sStripEdText, strAnyForeignStringsFound
                    ));

                    iIndex += 1;
                }
            }
        }

        sFinalOutput.push_str("### UI_Dump(): Bottom\n");

        SendStringToNotepad(sFinalOutput.as_str(), "temp.txt");

        // output the SP files...
        //
        for i in 0..vStripEdFiles.len() {
            // need to make local string, because ingame va() is crippled to 2 depths...
            //
            let sName = format!("Source\\StarWars\\code\\base\\strip\\MENUS{}.sp", i);
            SendStringToNotepad(vStripEdFiles[i].as_str(), sName.as_str());
        }
    }

    fn UI_Debug_EnterReference(ps4LetterType: &str, psItemString_raw: i64) {
        if psItemString_raw < 0 {
            // string package ID
            // Stub: SP_GetReferenceText would be a game engine function
            // For now, stubbing this out
        } else {
            // Note: Original code casts to LPCSTR which is problematic here
            // This section requires integration with the game's reference system
        }
    }

    // Stub for OutputDebugString - game-specific logging
    #[cfg(target_os = "windows")]
    fn OutputDebugString(s: &str) {
        // On Windows, this might use Windows API OutputDebugString
        // For now, just print to stderr
        eprintln!("{}", s);
    }

    #[cfg(not(target_os = "windows"))]
    fn OutputDebugString(s: &str) {
        eprintln!("{}", s);
    }

    // Stub for stricmp (case-insensitive string comparison)
    fn stricmp(a: &str, b: &str) -> i32 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        if a_lower < b_lower {
            -1
        } else if a_lower > b_lower {
            1
        } else {
            0
        }
    }

    // #endif
}

// #endif
