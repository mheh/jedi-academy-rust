// Tokenizer.h
//

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use std::ptr;

// pragma warning( disable : 4786 )	// identifier was truncated
//
// pragma warning (push, 3)			// go back down to 3 for the stl include
// pragma warning (disable:4503)		// decorated name length xceeded, name was truncated
// #include <string>
// #include <vector>
// #include <map>
// pragma warning (pop)
// pragma warning (disable:4503)		// decorated name length xceeded, name was truncated
//
// using namespace std;
//
// //#include <windows.h>
// #include "../qcommon/platform.h"

pub type byte = u8;
pub type word = u16;

// Windows types
pub type LPCTSTR = *const c_char;
pub type UINT = c_uint;
pub type DWORD = u32;
pub type COLORREF = u32;
pub type HANDLE = *mut c_void;

pub const MAX_STRING_LENGTH: usize = 256;
pub const MAX_IDENTIFIER_LENGTH: usize = 128;

pub const TKF_IGNOREDIRECTIVES: u32 = 0x00000001;  // skip over lines starting with #
pub const TKF_USES_EOL: u32 = 0x00000002;  // generate end of line tokens
pub const TKF_NODIRECTIVES: u32 = 0x00000004;  // don't treat # in any special way
pub const TKF_WANTUNDEFINED: u32 = 0x00000008;  // if token not found in symbols create undefined token
pub const TKF_WIDEUNDEFINEDSYMBOLS: u32 = 0x00000010;  // when undefined token encountered, accumulate until space
pub const TKF_RAWSYMBOLSONLY: u32 = 0x00000020;
pub const TKF_NUMERICIDENTIFIERSTART: u32 = 0x00000040;
pub const TKF_IGNOREKEYWORDS: u32 = 0x00000080;
pub const TKF_NOCASEKEYWORDS: u32 = 0x00000100;
pub const TKF_NOUNDERSCOREINIDENTIFIER: u32 = 0x00000200;
pub const TKF_NODASHINIDENTIFIER: u32 = 0x00000400;
pub const TKF_COMMENTTOKENS: u32 = 0x00000800;

pub const TKERR_NONE: i32 = 0;
pub const TKERR_UNKNOWN: i32 = 1;
pub const TKERR_BUFFERCREATE: i32 = 2;
pub const TKERR_UNRECOGNIZEDSYMBOL: i32 = 3;
pub const TKERR_DUPLICATESYMBOL: i32 = 4;
pub const TKERR_STRINGLENGTHEXCEEDED: i32 = 5;
pub const TKERR_IDENTIFIERLENGTHEXCEEDED: i32 = 6;
pub const TKERR_EXPECTED_INTEGER: i32 = 7;
pub const TKERR_EXPECTED_IDENTIFIER: i32 = 8;
pub const TKERR_EXPECTED_STRING: i32 = 9;
pub const TKERR_EXPECTED_CHAR: i32 = 10;
pub const TKERR_EXPECTED_FLOAT: i32 = 11;
pub const TKERR_UNEXPECTED_TOKEN: i32 = 12;
pub const TKERR_INVALID_DIRECTIVE: i32 = 13;
pub const TKERR_INCLUDE_FILE_NOTFOUND: i32 = 14;
pub const TKERR_UNMATCHED_DIRECTIVE: i32 = 15;
pub const TKERR_USERERROR: i32 = 16;

pub const TK_EOF: i32 = -1;
pub const TK_UNDEFINED: i32 = 0;
pub const TK_COMMENT: i32 = 1;
pub const TK_EOL: i32 = 2;
pub const TK_CHAR: i32 = 3;
pub const TK_STRING: i32 = 4;
pub const TK_INT: i32 = 5;
pub const TK_INTEGER: i32 = TK_INT;
pub const TK_FLOAT: i32 = 6;
pub const TK_IDENTIFIER: i32 = 7;
pub const TK_USERDEF: i32 = 8;

#[repr(C)]
pub struct keywordArray_t {
    pub m_keyword: *mut c_char,
    pub m_tokenvalue: c_int,
}

// class lessstr
// {
// public:
//     bool operator()(LPCTSTR str1, LPCTSTR str2) const {return (strcmp(str1, str2) < 0);};
// };
pub struct lessstr;

#[repr(C)]
pub struct CParseStream {
    pub m_next: *mut CParseStream,
}

impl CParseStream {
    pub fn new() -> Self {
        CParseStream {
            m_next: ptr::null_mut(),
        }
    }

    // static CParseStream* Create();
    pub unsafe fn Create() -> *mut CParseStream {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual bool NextChar(byte& theByte);
    pub unsafe fn NextChar(&mut self, theByte: &mut byte) -> bool {
        unimplemented!()
    }

    // virtual int GetCurLine();
    pub fn GetCurLine(&self) -> c_int {
        unimplemented!()
    }

    // virtual void GetCurFilename(char** theBuff);
    pub unsafe fn GetCurFilename(&self, theBuff: &mut *mut c_char) {
        unimplemented!()
    }

    // virtual long GetRemainingSize();
    pub fn GetRemainingSize(&self) -> i32 {
        unimplemented!()
    }

    // CParseStream* GetNext();
    pub fn GetNext(&self) -> *mut CParseStream {
        self.m_next
    }

    // void SetNext(CParseStream* next);
    pub unsafe fn SetNext(&mut self, next: *mut CParseStream) {
        self.m_next = next;
    }

    // virtual bool IsThisDefinition(void* theDefinition);
    pub unsafe fn IsThisDefinition(&self, theDefinition: *mut c_void) -> bool {
        unimplemented!()
    }

    // protected:
    // virtual bool Init();
}

#[repr(C)]
pub struct CToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
}

impl CToken {
    pub fn new() -> Self {
        CToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
        }
    }

    // static CToken* Create();
    pub unsafe fn Create() -> *mut CToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // CToken* GetNext();
    pub fn GetNext(&self) -> *mut CToken {
        self.m_next
    }

    // void SetNext(CToken* theToken);
    pub unsafe fn SetNext(&mut self, theToken: *mut CToken) {
        self.m_next = theToken;
    }

    // virtual int GetIntValue();
    pub fn GetIntValue(&self) -> c_int {
        unimplemented!()
    }

    // virtual LPCTSTR GetStringValue();
    pub fn GetStringValue(&self) -> LPCTSTR {
        unimplemented!()
    }

    // virtual float GetFloatValue();
    pub fn GetFloatValue(&self) -> f32 {
        unimplemented!()
    }

    // protected:
    // virtual void Init();
}

#[repr(C)]
pub struct CCharToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
}

impl CCharToken {
    pub fn new() -> Self {
        CCharToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
        }
    }

    // static CCharToken* Create(byte theByte);
    pub unsafe fn Create(theByte: byte) -> *mut CCharToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // protected:
    // virtual void Init(byte theByte);
}

#[repr(C)]
pub struct CStringToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
}

impl CStringToken {
    pub fn new() -> Self {
        CStringToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
        }
    }

    // static CStringToken* Create(LPCTSTR theString);
    pub unsafe fn Create(theString: LPCTSTR) -> *mut CStringToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // protected:
    // virtual void Init(LPCTSTR theString);
}

#[repr(C)]
pub struct CIntToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
    pub m_value: i32,
}

impl CIntToken {
    pub fn new() -> Self {
        CIntToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
            m_value: 0,
        }
    }

    // static CIntToken* Create(long value);
    pub unsafe fn Create(value: i32) -> *mut CIntToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // virtual float GetFloatValue();
    pub fn GetFloatValue(&self) -> f32 {
        unimplemented!()
    }

    // virtual int GetIntValue();
    pub fn GetIntValue(&self) -> c_int {
        unimplemented!()
    }

    // virtual LPCTSTR GetStringValue();
    pub fn GetStringValue(&self) -> LPCTSTR {
        unimplemented!()
    }

    // protected:
    // virtual void Init(long value);
}

#[repr(C)]
pub struct CFloatToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
    pub m_value: f32,
}

impl CFloatToken {
    pub fn new() -> Self {
        CFloatToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
            m_value: 0.0,
        }
    }

    // static CFloatToken* Create(float value);
    pub unsafe fn Create(value: f32) -> *mut CFloatToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // virtual float GetFloatValue();
    pub fn GetFloatValue(&self) -> f32 {
        unimplemented!()
    }

    // virtual LPCTSTR GetStringValue();
    pub fn GetStringValue(&self) -> LPCTSTR {
        unimplemented!()
    }

    // protected:
    // virtual void Init(float value);
}

#[repr(C)]
pub struct CIdentifierToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
}

impl CIdentifierToken {
    pub fn new() -> Self {
        CIdentifierToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
        }
    }

    // static CIdentifierToken* Create(LPCTSTR name);
    pub unsafe fn Create(name: LPCTSTR) -> *mut CIdentifierToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // protected:
    // virtual void Init(LPCTSTR name);
}

#[repr(C)]
pub struct CCommentToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
}

impl CCommentToken {
    pub fn new() -> Self {
        CCommentToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
        }
    }

    // static CCommentToken* Create(LPCTSTR name);
    pub unsafe fn Create(name: LPCTSTR) -> *mut CCommentToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // protected:
    // virtual void Init(LPCTSTR name);
}

#[repr(C)]
pub struct CUserToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
    pub m_value: c_int,
}

impl CUserToken {
    pub fn new() -> Self {
        CUserToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
            m_value: 0,
        }
    }

    // static CUserToken* Create(int value, LPCTSTR string);
    pub unsafe fn Create(value: c_int, string: LPCTSTR) -> *mut CUserToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // protected:
    // virtual void Init(int value, LPCTSTR string);
}

#[repr(C)]
pub struct CUndefinedToken {
    pub m_string: *mut c_char,
    pub m_next: *mut CToken,
}

impl CUndefinedToken {
    pub fn new() -> Self {
        CUndefinedToken {
            m_string: ptr::null_mut(),
            m_next: ptr::null_mut(),
        }
    }

    // static CUndefinedToken* Create(LPCTSTR string);
    pub unsafe fn Create(string: LPCTSTR) -> *mut CUndefinedToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetType();
    pub fn GetType(&self) -> c_int {
        unimplemented!()
    }

    // protected:
    // virtual void Init(LPCTSTR string);
}

#[repr(C)]
pub struct CSymbol {
    pub m_symbolName: *mut c_char,
}

impl CSymbol {
    pub fn new() -> Self {
        CSymbol {
            m_symbolName: ptr::null_mut(),
        }
    }

    // static CSymbol* Create(LPCTSTR symbolName);
    pub unsafe fn Create(symbolName: LPCTSTR) -> *mut CSymbol {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // LPCTSTR GetName();
    pub fn GetName(&self) -> LPCTSTR {
        unimplemented!()
    }

    // protected:
    // virtual void Init(LPCTSTR symbolName);
}

// typedef map<LPCTSTR, CSymbol*, lessstr> symbolmap_t;
pub type symbolmap_t = std::collections::BTreeMap<String, *mut CSymbol>;

#[repr(C)]
pub struct CDirectiveSymbol {
    pub m_symbolName: *mut c_char,
    pub m_value: *mut c_char,
}

impl CDirectiveSymbol {
    pub fn new() -> Self {
        CDirectiveSymbol {
            m_symbolName: ptr::null_mut(),
            m_value: ptr::null_mut(),
        }
    }

    // static CDirectiveSymbol* Create(LPCTSTR symbolName);
    pub unsafe fn Create(symbolName: LPCTSTR) -> *mut CDirectiveSymbol {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // void SetValue(LPCTSTR value);
    pub unsafe fn SetValue(&mut self, value: LPCTSTR) {
        unimplemented!()
    }

    // LPCTSTR GetValue();
    pub fn GetValue(&self) -> LPCTSTR {
        unimplemented!()
    }

    // protected:
    // virtual void Init(LPCTSTR symbolName);
}

#[repr(C)]
pub struct CIntSymbol {
    pub m_symbolName: *mut c_char,
    pub m_value: c_int,
}

impl CIntSymbol {
    pub fn new() -> Self {
        CIntSymbol {
            m_symbolName: ptr::null_mut(),
            m_value: 0,
        }
    }

    // static CIntSymbol* Create(LPCTSTR symbolName, int value);
    pub unsafe fn Create(symbolName: LPCTSTR, value: c_int) -> *mut CIntSymbol {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // int GetValue();
    pub fn GetValue(&self) -> c_int {
        unimplemented!()
    }

    // protected:
    // virtual void Init(LPCTSTR symbolName, int value);
}

#[repr(C)]
pub struct CSymbolTable {
    pub m_symbols: symbolmap_t,
}

impl CSymbolTable {
    pub fn new() -> Self {
        CSymbolTable {
            m_symbols: std::collections::BTreeMap::new(),
        }
    }

    // static CSymbolTable* Create();
    pub unsafe fn Create() -> *mut CSymbolTable {
        unimplemented!()
    }

    // void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // bool AddSymbol(CSymbol* theSymbol);
    pub unsafe fn AddSymbol(&mut self, theSymbol: *mut CSymbol) -> bool {
        unimplemented!()
    }

    // CSymbol* FindSymbol(LPCTSTR symbolName);
    pub fn FindSymbol(&self, symbolName: LPCTSTR) -> *mut CSymbol {
        unimplemented!()
    }

    // CSymbol* ExtractSymbol(LPCTSTR symbolName);
    pub unsafe fn ExtractSymbol(&mut self, symbolName: LPCTSTR) -> *mut CSymbol {
        unimplemented!()
    }

    // void RemoveSymbol(LPCTSTR symbolName);
    pub unsafe fn RemoveSymbol(&mut self, symbolName: LPCTSTR) {
        unimplemented!()
    }

    // void DiscardSymbols();
    pub unsafe fn DiscardSymbols(&mut self) {
        unimplemented!()
    }

    // protected:
    // void Init();
}

#[repr(C)]
pub struct CSymbolLookup {
    pub m_child: *mut CSymbolLookup,
    pub m_sibling: *mut CSymbolLookup,
    pub m_parent: *mut CSymbolLookup,
    pub m_value: c_int,
    pub m_byte: byte,
}

impl CSymbolLookup {
    pub fn new() -> Self {
        CSymbolLookup {
            m_child: ptr::null_mut(),
            m_sibling: ptr::null_mut(),
            m_parent: ptr::null_mut(),
            m_value: 0,
            m_byte: 0,
        }
    }

    // static CSymbolLookup* Create(byte theByte);
    pub unsafe fn Create(theByte: byte) -> *mut CSymbolLookup {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // CSymbolLookup* GetNext();
    pub fn GetNext(&self) -> *mut CSymbolLookup {
        unimplemented!()
    }

    // void SetNext(CSymbolLookup* next);
    pub unsafe fn SetNext(&mut self, next: *mut CSymbolLookup) {
        unimplemented!()
    }

    // void SetParent(CSymbolLookup* parent);
    pub unsafe fn SetParent(&mut self, parent: *mut CSymbolLookup) {
        self.m_parent = parent;
    }

    // CSymbolLookup* GetParent();
    pub fn GetParent(&self) -> *mut CSymbolLookup {
        self.m_parent
    }

    // void SetValue(int value);
    pub unsafe fn SetValue(&mut self, value: c_int) {
        self.m_value = value;
    }

    // int GetValue();
    pub fn GetValue(&self) -> c_int {
        self.m_value
    }

    // byte GetByte();
    pub fn GetByte(&self) -> byte {
        self.m_byte
    }

    // protected:
    // void Init(byte theByte);
}

#[repr(C)]
pub struct CTokenizerState {
    pub m_skip: bool,
    pub m_elseHit: bool,
    pub m_next: *mut CTokenizerState,
}

impl CTokenizerState {
    pub fn new() -> Self {
        CTokenizerState {
            m_skip: false,
            m_elseHit: false,
            m_next: ptr::null_mut(),
        }
    }

    // static CTokenizerState* Create(bool skip);
    pub unsafe fn Create(skip: bool) -> *mut CTokenizerState {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // CTokenizerState* GetNext();
    pub fn GetNext(&self) -> *mut CTokenizerState {
        self.m_next
    }

    // void SetNext(CTokenizerState* next);
    pub unsafe fn SetNext(&mut self, next: *mut CTokenizerState) {
        self.m_next = next;
    }

    // virtual bool ProcessElse();
    pub fn ProcessElse(&mut self) -> bool {
        unimplemented!()
    }

    // bool Skipping();
    pub fn Skipping(&self) -> bool {
        self.m_skip
    }

    // protected:
    // void Init(bool skip);
}

#[repr(C)]
pub struct CTokenizerHolderState {
    pub m_skip: bool,
    pub m_elseHit: bool,
    pub m_next: *mut CTokenizerState,
}

impl CTokenizerHolderState {
    pub fn new() -> Self {
        CTokenizerHolderState {
            m_skip: false,
            m_elseHit: false,
            m_next: ptr::null_mut(),
        }
    }

    // static CTokenizerHolderState* Create();
    pub unsafe fn Create() -> *mut CTokenizerHolderState {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual bool ProcessElse();
    pub fn ProcessElse(&mut self) -> bool {
        unimplemented!()
    }

    // protected:
    // void Init();
}

// typedef void (*LPTokenizerErrorProc)(LPCTSTR errString);
pub type LPTokenizerErrorProc = Option<extern "C" fn(LPCTSTR)>;

#[repr(C)]
pub struct CTokenizer {
    pub m_curParseStream: *mut CParseStream,
    pub m_keywords: *mut keywordArray_t,
    pub m_symbols: *mut keywordArray_t,
    pub m_errors: *mut keywordArray_t,
    pub m_symbolLookup: *mut CSymbolLookup,
    pub m_nextToken: *mut CToken,
    pub m_defines: CSymbolTable,
    pub m_state: *mut CTokenizerState,
    pub m_flags: UINT,
    pub m_errorProc: LPTokenizerErrorProc,
}

impl CTokenizer {
    pub fn new() -> Self {
        CTokenizer {
            m_curParseStream: ptr::null_mut(),
            m_keywords: ptr::null_mut(),
            m_symbols: ptr::null_mut(),
            m_errors: ptr::null_mut(),
            m_symbolLookup: ptr::null_mut(),
            m_nextToken: ptr::null_mut(),
            m_defines: CSymbolTable::new(),
            m_state: ptr::null_mut(),
            m_flags: 0,
            m_errorProc: None,
        }
    }

    // static CTokenizer* Create(UINT dwFlags = 0);
    pub unsafe fn Create(dwFlags: UINT) -> *mut CTokenizer {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual void Error(int theError);
    pub unsafe fn Error_1(&mut self, theError: c_int) {
        unimplemented!()
    }

    // virtual void Error(int theError, LPCTSTR errString);
    pub unsafe fn Error_2(&mut self, theError: c_int, errString: LPCTSTR) {
        unimplemented!()
    }

    // virtual void Error(LPCTSTR errString, int theError = TKERR_UNKNOWN);
    pub unsafe fn Error_3(&mut self, errString: LPCTSTR, theError: c_int) {
        unimplemented!()
    }

    // CToken* GetToken(UINT onFlags = 0, UINT offFlags = 0);
    pub unsafe fn GetToken_1(&mut self, onFlags: UINT, offFlags: UINT) -> *mut CToken {
        unimplemented!()
    }

    // CToken* GetToken(keywordArray_t* keywords, UINT onFlags, UINT offFlags);
    pub unsafe fn GetToken_2(
        &mut self,
        keywords: *mut keywordArray_t,
        onFlags: UINT,
        offFlags: UINT,
    ) -> *mut CToken {
        unimplemented!()
    }

    // void PutBackToken(CToken* theToken, bool commented = false, LPCTSTR addedChars = NULL, bool bIgnoreThisTokenType = false);
    pub unsafe fn PutBackToken(
        &mut self,
        theToken: *mut CToken,
        commented: bool,
        addedChars: LPCTSTR,
        bIgnoreThisTokenType: bool,
    ) {
        unimplemented!()
    }

    // bool RequireToken(int tokenType);
    pub unsafe fn RequireToken(&mut self, tokenType: c_int) -> bool {
        unimplemented!()
    }

    // void ScanUntilToken(int tokenType);
    pub unsafe fn ScanUntilToken(&mut self, tokenType: c_int) {
        unimplemented!()
    }

    // void SkipToLineEnd();
    pub unsafe fn SkipToLineEnd(&mut self) {
        unimplemented!()
    }

    // CToken* GetToEndOfLine(int tokenType = TK_IDENTIFIER);
    pub unsafe fn GetToEndOfLine(&mut self, tokenType: c_int) -> *mut CToken {
        unimplemented!()
    }

    // keywordArray_t* SetKeywords(keywordArray_t* theKeywords);
    pub unsafe fn SetKeywords(
        &mut self,
        theKeywords: *mut keywordArray_t,
    ) -> *mut keywordArray_t {
        unimplemented!()
    }

    // void SetSymbols(keywordArray_t* theSymbols);
    pub unsafe fn SetSymbols(&mut self, theSymbols: *mut keywordArray_t) {
        unimplemented!()
    }

    // void SetAdditionalErrors(keywordArray_t* theErrors);
    pub unsafe fn SetAdditionalErrors(&mut self, theErrors: *mut keywordArray_t) {
        unimplemented!()
    }

    // void SetErrorProc(LPTokenizerErrorProc errorProc);
    pub unsafe fn SetErrorProc(&mut self, errorProc: LPTokenizerErrorProc) {
        unimplemented!()
    }

    // void AddParseStream(byte* data, long datasize);
    pub unsafe fn AddParseStream(&mut self, data: *mut byte, datasize: i32) {
        unimplemented!()
    }

    // bool AddParseFile(LPCTSTR filename);
    pub unsafe fn AddParseFile(&mut self, filename: LPCTSTR) -> bool {
        unimplemented!()
    }

    // COLORREF ParseRGB();
    pub unsafe fn ParseRGB(&mut self) -> COLORREF {
        unimplemented!()
    }

    // long GetRemainingSize();
    pub fn GetRemainingSize(&self) -> i32 {
        unimplemented!()
    }

    // UINT GetFlags();
    pub fn GetFlags(&self) -> UINT {
        unimplemented!()
    }

    // void SetFlags(UINT flags);
    pub unsafe fn SetFlags(&mut self, flags: UINT) {
        unimplemented!()
    }

    // void GetCurFilename(char** filename);
    pub unsafe fn GetCurFilename(&self, filename: &mut *mut c_char) {
        unimplemented!()
    }

    // int GetCurLine();
    pub fn GetCurLine(&self) -> c_int {
        unimplemented!()
    }

    // LPCTSTR LookupToken(int tokenID, keywordArray_t* theTable = NULL);
    pub fn LookupToken(&self, tokenID: c_int, theTable: *mut keywordArray_t) -> LPCTSTR {
        unimplemented!()
    }

    // protected:
    // void SetError(int theError, LPCTSTR errString);
    // virtual void Init(UINT dwFlags = 0);
    // CToken* FetchToken();
    // bool AddDefineSymbol(CDirectiveSymbol* definesymbol);
    // bool NextChar(byte& theByte);
    // byte Escapement();
    // void InsertSymbol(LPCTSTR theSymbol, int theValue);
    // void PutBackChar(byte theByte, int curLine = 0, LPCTSTR filename = NULL);
    // CToken* TokenFromName(LPCTSTR name);
    // CToken* HandleDirective();
    // CToken* HandleSlash();
    // CToken* HandleString();
    // CToken* HandleQuote();
    // CToken* HandleIdentifier(byte theByte);
    // CToken* HandleNumeric(byte theByte);
    // CToken* HandleFloat(bool thesign = false, long value = 0);
    // CToken* HandleDecimal(bool thesign = false);
    // CToken* HandleSymbol(byte theByte);
    // CToken* HandleHex(bool thesize);
    // CToken* HandleOctal(bool thesize);
    // int DirectiveFromName(LPCTSTR name);

    // static keywordArray_t errorMessages[];
    // static keywordArray_t directiveKeywords[];
}

pub static mut CTokenizer_errorMessages: [keywordArray_t; 0] = [];
pub static mut CTokenizer_directiveKeywords: [keywordArray_t; 0] = [];

#[repr(C)]
pub struct CKeywordTable {
    pub m_tokenizer: *mut CTokenizer,
    pub m_holdKeywords: *mut keywordArray_t,
}

impl CKeywordTable {
    pub fn new(tokenizer: *mut CTokenizer, keywords: *mut keywordArray_t) -> Self {
        CKeywordTable {
            m_tokenizer: tokenizer,
            m_holdKeywords: keywords,
        }
    }
}

#[repr(C)]
pub struct CParsePutBack {
    pub m_next: *mut CParseStream,
    pub m_byte: byte,
    pub m_consumed: bool,
    pub m_curLine: c_int,
    pub m_curFile: *mut c_char,
}

impl CParsePutBack {
    pub fn new() -> Self {
        CParsePutBack {
            m_next: ptr::null_mut(),
            m_byte: 0,
            m_consumed: false,
            m_curLine: 0,
            m_curFile: ptr::null_mut(),
        }
    }

    // static CParsePutBack* Create(byte theByte, int curLine, LPCTSTR filename);
    pub unsafe fn Create(theByte: byte, curLine: c_int, filename: LPCTSTR) -> *mut CParsePutBack {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual bool NextChar(byte& theByte);
    pub unsafe fn NextChar(&mut self, theByte: &mut byte) -> bool {
        unimplemented!()
    }

    // virtual int GetCurLine();
    pub fn GetCurLine(&self) -> c_int {
        unimplemented!()
    }

    // virtual void GetCurFilename(char** theBuff);
    pub unsafe fn GetCurFilename(&self, theBuff: &mut *mut c_char) {
        unimplemented!()
    }

    // virtual long GetRemainingSize();
    pub fn GetRemainingSize(&self) -> i32 {
        unimplemented!()
    }

    // protected:
    // virtual void Init(byte theByte, int curLine, LPCTSTR filename);
}

#[repr(C)]
pub struct CParseMemory {
    pub m_next: *mut CParseStream,
    pub m_data: *mut byte,
    pub m_curLine: c_int,
    pub m_curPos: i32,
    pub m_datasize: i32,
    pub m_offset: i32,
}

impl CParseMemory {
    pub fn new() -> Self {
        CParseMemory {
            m_next: ptr::null_mut(),
            m_data: ptr::null_mut(),
            m_curLine: 0,
            m_curPos: 0,
            m_datasize: 0,
            m_offset: 0,
        }
    }

    // static CParseMemory* Create(byte* data, long datasize);
    pub unsafe fn Create(data: *mut byte, datasize: i32) -> *mut CParseMemory {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual bool NextChar(byte& theByte);
    pub unsafe fn NextChar(&mut self, theByte: &mut byte) -> bool {
        unimplemented!()
    }

    // virtual int GetCurLine();
    pub fn GetCurLine(&self) -> c_int {
        unimplemented!()
    }

    // virtual void GetCurFilename(char** theBuff);
    pub unsafe fn GetCurFilename(&self, theBuff: &mut *mut c_char) {
        unimplemented!()
    }

    // virtual long GetRemainingSize();
    pub fn GetRemainingSize(&self) -> i32 {
        unimplemented!()
    }

    // protected:
    // virtual void Init(byte* data, long datasize);
}

#[repr(C)]
pub struct CParseBlock {
    pub m_next: *mut CParseStream,
    pub m_data: *mut byte,
    pub m_curLine: c_int,
    pub m_curPos: i32,
    pub m_datasize: i32,
    pub m_offset: i32,
}

impl CParseBlock {
    pub fn new() -> Self {
        CParseBlock {
            m_next: ptr::null_mut(),
            m_data: ptr::null_mut(),
            m_curLine: 0,
            m_curPos: 0,
            m_datasize: 0,
            m_offset: 0,
        }
    }

    // static CParseBlock* Create(byte* data, long datasize);
    pub unsafe fn Create(data: *mut byte, datasize: i32) -> *mut CParseBlock {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // protected:
    // virtual void Init(byte* data, long datasize);
}

#[repr(C)]
pub struct CParseToken {
    pub m_next: *mut CParseStream,
    pub m_data: *mut byte,
    pub m_curLine: c_int,
    pub m_curPos: i32,
    pub m_datasize: i32,
    pub m_offset: i32,
}

impl CParseToken {
    pub fn new() -> Self {
        CParseToken {
            m_next: ptr::null_mut(),
            m_data: ptr::null_mut(),
            m_curLine: 0,
            m_curPos: 0,
            m_datasize: 0,
            m_offset: 0,
        }
    }

    // static CParseToken* Create(CToken* token);
    pub unsafe fn Create(token: *mut CToken) -> *mut CParseToken {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual bool NextChar(byte& theByte);
    pub unsafe fn NextChar(&mut self, theByte: &mut byte) -> bool {
        unimplemented!()
    }

    // virtual int GetCurLine();
    pub fn GetCurLine(&self) -> c_int {
        unimplemented!()
    }

    // virtual void GetCurFilename(char** theBuff);
    pub unsafe fn GetCurFilename(&self, theBuff: &mut *mut c_char) {
        unimplemented!()
    }

    // virtual long GetRemainingSize();
    pub fn GetRemainingSize(&self) -> i32 {
        unimplemented!()
    }

    // protected:
    // virtual void Init(CToken* token);
}

#[repr(C)]
pub struct CParseDefine {
    pub m_next: *mut CParseStream,
    pub m_data: *mut byte,
    pub m_curLine: c_int,
    pub m_curPos: i32,
    pub m_datasize: i32,
    pub m_offset: i32,
    pub m_defineSymbol: *mut CDirectiveSymbol,
}

impl CParseDefine {
    pub fn new() -> Self {
        CParseDefine {
            m_next: ptr::null_mut(),
            m_data: ptr::null_mut(),
            m_curLine: 0,
            m_curPos: 0,
            m_datasize: 0,
            m_offset: 0,
            m_defineSymbol: ptr::null_mut(),
        }
    }

    // static CParseDefine* Create(CDirectiveSymbol* definesymbol);
    pub unsafe fn Create(definesymbol: *mut CDirectiveSymbol) -> *mut CParseDefine {
        unimplemented!()
    }

    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual bool IsThisDefinition(void* theDefinition);
    pub unsafe fn IsThisDefinition(&self, theDefinition: *mut c_void) -> bool {
        unimplemented!()
    }

    // protected:
    // virtual void Init(CDirectiveSymbol* definesymbol);
}

#[repr(C)]
pub struct CParseFile {
    pub m_next: *mut CParseStream,
    pub m_fileHandle: HANDLE,
    pub m_fileName: *mut c_char,
    pub m_curLine: c_int,
    pub m_curPos: c_int,
    pub m_buff: *mut byte,
    pub m_curByte: DWORD,
    pub m_filesize: DWORD,
    pub m_ownsFile: bool,
}

impl CParseFile {
    pub fn new() -> Self {
        CParseFile {
            m_next: ptr::null_mut(),
            m_fileHandle: ptr::null_mut(),
            m_fileName: ptr::null_mut(),
            m_curLine: 0,
            m_curPos: 0,
            m_buff: ptr::null_mut(),
            m_curByte: 0,
            m_filesize: 0,
            m_ownsFile: false,
        }
    }

    // static CParseFile* Create();
    pub unsafe fn Create() -> *mut CParseFile {
        unimplemented!()
    }

    // static CParseFile* Create(LPCTSTR filename, CTokenizer* tokenizer);
    pub unsafe fn Create_2(filename: LPCTSTR, tokenizer: *mut CTokenizer) -> *mut CParseFile {
        unimplemented!()
    }

    // //	static CParseFile* Create(CFile* file, CTokenizer* tokenizer);
    // virtual void Delete();
    pub unsafe fn Delete(&mut self) {
        unimplemented!()
    }

    // virtual int GetCurLine();
    pub fn GetCurLine(&self) -> c_int {
        unimplemented!()
    }

    // virtual void GetCurFilename(char** theBuff);
    pub unsafe fn GetCurFilename(&self, theBuff: &mut *mut c_char) {
        unimplemented!()
    }

    // virtual long GetRemainingSize();
    pub fn GetRemainingSize(&self) -> i32 {
        unimplemented!()
    }

    // virtual bool NextChar(byte& theByte);
    pub unsafe fn NextChar(&mut self, theByte: &mut byte) -> bool {
        unimplemented!()
    }

    // protected:
    // virtual bool Init();
    // virtual bool Init(LPCTSTR filename, CTokenizer* tokenizer);
    // //	virtual void Init(CFile* file, CTokenizer* tokenizer);
    // DWORD GetFileSize();
    // void Read(void* buff, UINT buffsize);

    // //	CFile*			m_file;
}

// #endif//__TOKENIZER_H
