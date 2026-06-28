// These entries are now also duplicated in ModView, so tell me if you need any adding or removing.
//  Note that the order is ok to change, I only read/write text strings of them anyway, but tell me if there
//	are different choices to offer in the pulldown box. I know, it's tacky, but ModView wasn't planned as an
//	editor and this was never an external file. A great combination...   - Ste.
//

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum soundChannel_t {
	CHAN_AUTO = 0,		//## %s !!"W:\game\base\!!sound\*.wav;*.mp3" # Auto-picks an empty channel to play sound on
	CHAN_LOCAL = 1,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3" # menu sounds, etc
	CHAN_WEAPON = 2,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3"
	CHAN_VOICE = 3,	//## %s !!"W:\game\base\!!sound\voice\*.wav;*.mp3" # Voice sounds cause mouth animation
	CHAN_VOICE_ATTEN = 4,	//## %s !!"W:\game\base\!!sound\voice\*.wav;*.mp3" # Causes mouth animation but still use normal sound falloff
	CHAN_VOICE_GLOBAL = 5,	//## %s !!"W:\game\base\!!sound\voice\*.wav;*.mp3" # Causes mouth animation and is broadcast with no separation
	CHAN_ITEM = 6,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3"
	CHAN_BODY = 7,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3"
	CHAN_AMBIENT = 8,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3" # added for ambient sounds
	CHAN_LOCAL_SOUND = 9,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3" #chat messages, etc
	CHAN_ANNOUNCER = 10,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3" #announcer voices, etc
	CHAN_LESS_ATTEN = 11,	//## %s !!"W:\game\base\!!sound\*.wav;*.mp3" #attenuates similar to chan_voice, but uses empty channel auto-pick behaviour
	CHAN_MUSIC = 12,	//played as a looping sound - added by BTO (VV)
}
