//! `be_ai_chat.h` — char AI declarations.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_ulong};

pub const MAX_MESSAGE_SIZE: usize = 256;
pub const MAX_CHATTYPE_NAME: usize = 32;
pub const MAX_MATCHVARIABLES: usize = 8;

pub const CHAT_GENDERLESS: c_int = 0;
pub const CHAT_GENDERFEMALE: c_int = 1;
pub const CHAT_GENDERMALE: c_int = 2;

pub const CHAT_ALL: c_int = 0;
pub const CHAT_TEAM: c_int = 1;
pub const CHAT_TELL: c_int = 2;

// a console message
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct bot_consolemessage_t {
    pub handle: c_int,
    /// message time
    pub time: f32,
    /// message type
    pub type_: c_int,
    /// message
    pub message: [c_char; MAX_MESSAGE_SIZE],
    /// prev and next in list
    pub prev: *mut bot_consolemessage_t,
    pub next: *mut bot_consolemessage_t,
}

impl Default for bot_consolemessage_t {
    fn default() -> Self {
        // SAFETY: this mirrors C zero-initialization for a POD message node.
        unsafe { core::mem::zeroed() }
    }
}

// match variable
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct bot_matchvariable_t {
    pub offset: c_char,
    pub length: c_int,
}

// returned to AI when a match is found
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct bot_match_t {
    pub string: [c_char; MAX_MESSAGE_SIZE],
    pub type_: c_int,
    pub subtype: c_int,
    pub variables: [bot_matchvariable_t; MAX_MATCHVARIABLES],
}

impl Default for bot_match_t {
    fn default() -> Self {
        // SAFETY: this mirrors C zero-initialization for a POD match result.
        unsafe { core::mem::zeroed() }
    }
}

unsafe extern "C" {
    // setup the chat AI
    pub fn BotSetupChatAI() -> c_int;
    // shutdown the chat AI
    pub fn BotShutdownChatAI();
    // returns the handle to a newly allocated chat state
    pub fn BotAllocChatState() -> c_int;
    // frees the chatstate
    pub fn BotFreeChatState(handle: c_int);
    // adds a console message to the chat state
    pub fn BotQueueConsoleMessage(chatstate: c_int, type_: c_int, message: *mut c_char);
    // removes the console message from the chat state
    pub fn BotRemoveConsoleMessage(chatstate: c_int, handle: c_int);
    // returns the next console message from the state
    pub fn BotNextConsoleMessage(chatstate: c_int, cm: *mut bot_consolemessage_t) -> c_int;
    // returns the number of console messages currently stored in the state
    pub fn BotNumConsoleMessages(chatstate: c_int) -> c_int;
    // selects a chat message of the given type
    pub fn BotInitialChat(
        chatstate: c_int,
        type_: *mut c_char,
        mcontext: c_int,
        var0: *mut c_char,
        var1: *mut c_char,
        var2: *mut c_char,
        var3: *mut c_char,
        var4: *mut c_char,
        var5: *mut c_char,
        var6: *mut c_char,
        var7: *mut c_char,
    );
    // returns the number of initial chat messages of the given type
    pub fn BotNumInitialChats(chatstate: c_int, type_: *mut c_char) -> c_int;
    // find and select a reply for the given message
    pub fn BotReplyChat(
        chatstate: c_int,
        message: *mut c_char,
        mcontext: c_int,
        vcontext: c_int,
        var0: *mut c_char,
        var1: *mut c_char,
        var2: *mut c_char,
        var3: *mut c_char,
        var4: *mut c_char,
        var5: *mut c_char,
        var6: *mut c_char,
        var7: *mut c_char,
    ) -> c_int;
    // returns the length of the currently selected chat message
    pub fn BotChatLength(chatstate: c_int) -> c_int;
    // enters the selected chat message
    pub fn BotEnterChat(chatstate: c_int, clientto: c_int, sendto: c_int);
    // get the chat message ready to be output
    pub fn BotGetChatMessage(chatstate: c_int, buf: *mut c_char, size: c_int);
    // checks if the first string contains the second one, returns index into first string or -1 if not found
    pub fn StringContains(str1: *mut c_char, str2: *mut c_char, casesensitive: c_int) -> c_int;
    // finds a match for the given string using the match templates
    pub fn BotFindMatch(str: *mut c_char, match_: *mut bot_match_t, context: c_ulong) -> c_int;
    // returns a variable from a match
    pub fn BotMatchVariable(match_: *mut bot_match_t, variable: c_int, buf: *mut c_char, size: c_int);
    // unify all the white spaces in the string
    pub fn UnifyWhiteSpaces(string: *mut c_char);
    // replace all the context related synonyms in the string
    pub fn BotReplaceSynonyms(string: *mut c_char, context: c_ulong);
    // loads a chat file for the chat state
    pub fn BotLoadChatFile(chatstate: c_int, chatfile: *mut c_char, chatname: *mut c_char) -> c_int;
    // store the gender of the bot in the chat state
    pub fn BotSetChatGender(chatstate: c_int, gender: c_int);
    // store the bot name in the chat state
    pub fn BotSetChatName(chatstate: c_int, name: *mut c_char, client: c_int);
}
