//dynamic music
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum dynamicMusic_t {
	//# dynamicMusic_e
	DM_AUTO = 0,	//# let the game determine the dynamic music as normal
	DM_SILENCE = 1,	//# stop the music
	DM_EXPLORE = 2,	//# force the exploration music to play
	DM_ACTION = 3,	//# force the action music to play
	DM_BOSS = 4,	//# force the boss battle music to play (if there is any)
	DM_DEATH = 5,	//# force the "player dead" music to play
}
