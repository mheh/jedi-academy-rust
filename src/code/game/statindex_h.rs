// Filename:	statindex.h
//
// accessed from both server and game modules

// player_state->stats[] indexes
#[repr(C)]
pub enum statIndex_t {
	STAT_HEALTH,
	STAT_ITEMS,
	STAT_WEAPONS,					// 16 bit fields
	STAT_ARMOR,
	STAT_DEAD_YAW,					// look this direction when dead (FIXME: get rid of?)
	STAT_CLIENTS_READY,				// bit mask of clients wishing to exit the intermission (FIXME: configstring?)
	STAT_MAX_HEALTH					// health / armor limit, changable by handicap
}

/////////////////////// eof /////////////////////
