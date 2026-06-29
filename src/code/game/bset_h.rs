//# bSet_e
// This should check to matching a behavior state name first, then look for a script
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum bSet_e {
    BSET_INVALID = -1,
    BSET_FIRST = 0,
    BSET_SPAWN = 0, //# script to use when first spawned
    BSET_USE = 1, //# script to use when used
    BSET_AWAKE = 2, //# script to use when awoken/startled
    BSET_ANGER = 3, //# script to use when aquire an enemy
    BSET_ATTACK = 4, //# script to run when you attack
    BSET_VICTORY = 5, //# script to run when you kill someone
    BSET_LOSTENEMY = 6, //# script to run when you can't find your enemy
    BSET_PAIN = 7, //# script to use when take pain
    BSET_FLEE = 8, //# script to use when take pain below 50% of health
    BSET_DEATH = 9, //# script to use when killed
    BSET_DELAYED = 10, //# script to run when self->delayScriptTime is reached
    BSET_BLOCKED = 11, //# script to run when blocked by a friendly NPC or player
    BSET_BUMPED = 12, //# script to run when bumped into a friendly NPC or player (can set bumpRadius)
    BSET_STUCK = 13, //# script to run when blocked by a wall
    BSET_FFIRE = 14, //# script to run when player shoots their own teammates
    BSET_FFDEATH = 15, //# script to run when player kills a teammate
    BSET_MINDTRICK = 16, //# script to run when player does a mind trick on this NPC

    NUM_BSETS = 17,
}

#[allow(non_camel_case_types)]
pub type bSet_t = bSet_e;
