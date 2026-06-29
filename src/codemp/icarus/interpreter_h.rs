// Interpreter.h

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::os::raw::c_float;

const ICARUS_VERSION: f32 = 1.33;
const MAX_STRING_SIZE: c_int = 256;
const MAX_VAR_NAME: c_int = 64;

pub type vector_t = [c_float; 3];

// If you modify this, you MUST modify in g_ICARUScb.c as well.
// Token defines
// Note: TK_BLOCK_START = TK_USERDEF; TK_USERDEF is defined elsewhere
pub const TK_BLOCK_START: c_int = 0; // Starts at TK_USERDEF (external dependency)
pub const TK_BLOCK_END: c_int = TK_BLOCK_START + 1;
pub const TK_VECTOR_START: c_int = TK_BLOCK_START + 2;
pub const TK_VECTOR_END: c_int = TK_BLOCK_START + 3;
pub const TK_OPEN_PARENTHESIS: c_int = TK_BLOCK_START + 4;
pub const TK_CLOSED_PARENTHESIS: c_int = TK_BLOCK_START + 5;
pub const TK_VECTOR: c_int = TK_BLOCK_START + 6;
pub const TK_GREATER_THAN: c_int = TK_BLOCK_START + 7;
pub const TK_LESS_THAN: c_int = TK_BLOCK_START + 8;
pub const TK_EQUALS: c_int = TK_BLOCK_START + 9;
pub const TK_NOT: c_int = TK_BLOCK_START + 10;

pub const NUM_USER_TOKENS: c_int = TK_BLOCK_START + 11;

// ID defines
pub const ID_AFFECT: c_int = NUM_USER_TOKENS;
pub const ID_SOUND: c_int = NUM_USER_TOKENS + 1;
pub const ID_MOVE: c_int = NUM_USER_TOKENS + 2;
pub const ID_ROTATE: c_int = NUM_USER_TOKENS + 3;
pub const ID_WAIT: c_int = NUM_USER_TOKENS + 4;
pub const ID_BLOCK_START: c_int = NUM_USER_TOKENS + 5;
pub const ID_BLOCK_END: c_int = NUM_USER_TOKENS + 6;
pub const ID_SET: c_int = NUM_USER_TOKENS + 7;
pub const ID_LOOP: c_int = NUM_USER_TOKENS + 8;
pub const ID_LOOPEND: c_int = NUM_USER_TOKENS + 9;
pub const ID_PRINT: c_int = NUM_USER_TOKENS + 10;
pub const ID_USE: c_int = NUM_USER_TOKENS + 11;
pub const ID_FLUSH: c_int = NUM_USER_TOKENS + 12;
pub const ID_RUN: c_int = NUM_USER_TOKENS + 13;
pub const ID_KILL: c_int = NUM_USER_TOKENS + 14;
pub const ID_REMOVE: c_int = NUM_USER_TOKENS + 15;
pub const ID_CAMERA: c_int = NUM_USER_TOKENS + 16;
pub const ID_GET: c_int = NUM_USER_TOKENS + 17;
pub const ID_RANDOM: c_int = NUM_USER_TOKENS + 18;
pub const ID_IF: c_int = NUM_USER_TOKENS + 19;
pub const ID_ELSE: c_int = NUM_USER_TOKENS + 20;
pub const ID_REM: c_int = NUM_USER_TOKENS + 21;
pub const ID_TASK: c_int = NUM_USER_TOKENS + 22;
pub const ID_DO: c_int = NUM_USER_TOKENS + 23;
pub const ID_DECLARE: c_int = NUM_USER_TOKENS + 24;
pub const ID_FREE: c_int = NUM_USER_TOKENS + 25;
pub const ID_DOWAIT: c_int = NUM_USER_TOKENS + 26;
pub const ID_SIGNAL: c_int = NUM_USER_TOKENS + 27;
pub const ID_WAITSIGNAL: c_int = NUM_USER_TOKENS + 28;
pub const ID_PLAY: c_int = NUM_USER_TOKENS + 29;

pub const ID_TAG: c_int = NUM_USER_TOKENS + 30;
pub const ID_EOF: c_int = NUM_USER_TOKENS + 31;
pub const NUM_IDS: c_int = NUM_USER_TOKENS + 32;

// Type defines
pub const TYPE_WAIT_COMPLETE: c_int = NUM_IDS;
pub const TYPE_WAIT_TRIGGERED: c_int = NUM_IDS + 1;

pub const TYPE_ANGLES: c_int = NUM_IDS + 2;
pub const TYPE_ORIGIN: c_int = NUM_IDS + 3;

pub const TYPE_INSERT: c_int = NUM_IDS + 4;
pub const TYPE_FLUSH: c_int = NUM_IDS + 5;

pub const TYPE_PAN: c_int = NUM_IDS + 6;
pub const TYPE_ZOOM: c_int = NUM_IDS + 7;
pub const TYPE_MOVE: c_int = NUM_IDS + 8;
pub const TYPE_FADE: c_int = NUM_IDS + 9;
pub const TYPE_PATH: c_int = NUM_IDS + 10;
pub const TYPE_ENABLE: c_int = NUM_IDS + 11;
pub const TYPE_DISABLE: c_int = NUM_IDS + 12;
pub const TYPE_SHAKE: c_int = NUM_IDS + 13;
pub const TYPE_ROLL: c_int = NUM_IDS + 14;
pub const TYPE_TRACK: c_int = NUM_IDS + 15;
pub const TYPE_DISTANCE: c_int = NUM_IDS + 16;
pub const TYPE_FOLLOW: c_int = NUM_IDS + 17;

pub const TYPE_VARIABLE: c_int = NUM_IDS + 18;

pub const TYPE_EOF: c_int = NUM_IDS + 19;
pub const NUM_TYPES: c_int = NUM_IDS + 20;

pub const MSG_COMPLETED: c_int = 0;
pub const MSG_EOF: c_int = 1;
pub const NUM_MESSAGES: c_int = 2;

#[repr(C)]
pub struct variable_s {
    pub name: [c_char; MAX_VAR_NAME as usize],
    pub r#type: c_int,
    pub data: *mut c_void,
}

pub type variable_t = variable_s;

// Stub types for C++ STL and forward declarations
// These are opaque types representing C++ class instances
pub type variable_v = c_void; // vector<variable_t*> - unported C++ STL
pub type variable_m = c_void; // map<string, variable_t*> - unported C++ STL
pub type keywordArray_t = c_void; // Forward declaration - defined elsewhere
pub type CTokenizer = c_void; // Forward declaration - defined elsewhere
pub type CBlockStream = c_void; // Forward declaration - defined elsewhere

// CInterpreter - C++ class representation as Rust struct
// The C++ class members that are STL containers are represented as opaque pointers
#[repr(C)]
pub struct CInterpreter {
    // Member variables (representation of C++ class)
    m_tokenizer: *mut CTokenizer,
    m_blockStream: *mut CBlockStream,
    m_vars: *mut variable_v,
    m_varMap: *mut variable_m,
    m_sCurrentLine: *mut c_char, // C++ string
    m_sCurrentFile: *mut c_char, // C++ string
    m_iCurrentLine: c_int,
    m_iBadCBlockNumber: c_int,
}

// Note: The following method declarations are for reference; they cannot be directly
// translated to Rust without the full C++ class implementation and may require
// different calling conventions.
//
// Original C++ methods (preserved for reference):
// public:
//   CInterpreter();
//   ~CInterpreter();
//   int Interpret( CTokenizer *, CBlockStream *, char *filename=NULL );   //Main interpretation function
//   int Match( int );                         //Looks ahead to the next token to try and match it to the passed token, consumes token on success
//   int LookAhead( int );                     //Looks ahead without consuming on success
//   int FindSymbol( const char *,  keywordArray_t * );     //Searches the symbol table for the given name.  Returns the ID if found
//   int GetAffect( void );                    //Handles the affect() function
//   int GetWait( void );                      //Handles the wait() function
//   int GetSet( void );                       //Handles the set() function
//   int GetBroadcast( void );                 //Handles the broadcast() function
//   int GetLoop( void );                      //Handles the loop() function
//   int GetPrint( void );                     //Handles the print() function
//   int GetUse( void );                       //Handles the use() function
//   int GetFlush( void );                     //Handles the flush() function
//   int GetRun( void );                       //Handles the run() function
//   int GetKill( void );                      //Handles the kill() function
//   int GetRemove( void );                    //Handles the remove() function
//   int GetCamera( void );                    //Handles the camera() function
//   int GetIf( void );                        //Handles the if() conditional statement
//   int GetSound( void );                     //Handles the sound() function
//   int GetMove( void );                      //Handles the move() function
//   int GetRotate( void );                    //Handles the rotate() function
//   int GetRem( void );                       //Handles the rem() function
//   int GetTask( void );
//   int GetDo( void );
//   int GetElse( void );
//   int GetDeclare( void );
//   int GetFree( void );
//   int GetDoWait( void );
//   int GetSignal( void );
//   int GetWaitSignal( void );
//   int GetPlay( void );
//   int GetRandom( CBlock *block );
//   int GetGet( CBlock *block );               //Heh
//   int GetTag( CBlock *block );               //Handles the tag() identifier
//   int GetVector( CBlock *block );
//   int GetNextType( void );
//   int GetType( char *get );
//   int GetAny( CBlock *block );
//   int GetEvaluator( CBlock *block );
//   int GetString( CBlock *);                  //Attempts to match and retrieve the value of a string token
//   int GetIdentifier( CBlock *get );         //Attempts to match and retrieve the value of an identifier token
//   int GetInteger( CBlock * );               //Attempts to match and retrieve the value of a int token
//   int GetFloat( CBlock * );                 //Attempts to match and retrieve the value of a float token
//   int GetVariable( int type );
//   int GetID ( char * );                     //Attempts to match and interpret an identifier
//   keywordArray_t *GetSymbols( void )        {   return (keywordArray_t *) &m_symbolKeywords;  }   //Returns the interpreter's symbol table
//   keywordArray_t *GetIDs( void )            {   return (keywordArray_t *) &m_IDKeywords;      }   //Returns the interpreter's ID table
//   keywordArray_t *GetTypes( void )          {   return (keywordArray_t *) &m_typeKeywords;   }   //Returns the interpreter's type table
//
// protected:
//   void InitVars( void );
//   void FreeVars( void );
//   variable_t *AddVar( const char *name, int type );
//   variable_t *FindVar( const char *name );
//   const char *GetTokenName( int );          //Returns the name of a token
//   int Error( char *, ... );                 //Prints an error message
//   int MatchTag( void );                     //Attempts to match to a tag identifier
//   int MatchGet( void );                     //Attempts to match to a get identifier
//   int MatchRandom( void );                  //Attempts to match to a random identifier
//
//   static keywordArray_t m_symbolKeywords[];      //Symbols
//   static keywordArray_t m_IDKeywords[];          //Identifiers
//   static keywordArray_t m_typeKeywords[];        //Types
//   static keywordArray_t m_conditionalKeywords[]; //Conditional
