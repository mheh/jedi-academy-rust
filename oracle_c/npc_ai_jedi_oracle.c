/* Oracle: Jedi_ReCalcParryTime extracted from the original
   raven-jediacademy/codemp/game/NPC_AI_Jedi.c (:2253), compared against
   OpenJK/codemp/game/NPC_AI_Jedi.c. Compiled and linked under the `oracle` cargo
   feature (see build.rs) so the Rust port can be asserted bit-exact against the real C.

   The function takes `gentity_t *self` + `evasionType_t evasionType` but reads only a
   handful of scalar fields:
     self->client != NULL                                    -> has_client
     self->s.number                                          -> number
     self->client->ps.fd.forcePowerLevel[FP_SABER_DEFENSE]   -> saberDefenseLevel
     self->NPC != NULL                                       -> npc_present
     self->client->NPC_class                                 -> npc_class
     self->NPC->rank                                         -> npc_rank
     self->client->ps.torsoTimer                             -> torsoTimer
     self->client->ps.saberInFlight                          -> saberInFlight
   plus the file-globals g_saberRealisticCombat.integer (realCombat) and
   g_spskill.integer (spskill). Following the bg_misc_oracle.c / npc_reactions_oracle.c
   scalar-marshaling precedent, the wrapper passes those as positional scalars and the
   body is transcribed VERBATIM with the field reads substituted by the marshaled locals.

   The player branch indexes `bg_parryDebounce` (defined in bg_saber.c); its four values
   are inlined here verbatim from that table so the oracle is self-contained.

   Q_irand is the PC wrapper over irand (the holdrand MSVC LCG in q_math_oracle.c) —
   the same holdrand the Rust port's q_math::irand uses; the test re-seeds both sides
   via Rand_Init before each call so the consumed stream matches. */

#include <math.h> /* ceil */

int Q_irand(int value1, int value2); /* q_math_oracle.c */

/* CLASS_TAVION (teams.h:72). RANK_LT_JG/RANK_CIVILIAN/RANK_CREWMAN (b_public.h). The
   EVASION_* values (w_saber.h evasionType_t). */
#define O_CLASS_TAVION       47
#define O_RANK_CIVILIAN      0
#define O_RANK_CREWMAN       1
#define O_RANK_LT_JG         3
#define O_EVASION_PARRY      1
#define O_EVASION_DUCK_PARRY 2
#define O_EVASION_JUMP_PARRY 3
#define O_EVASION_DODGE      4
#define O_EVASION_JUMP       5
#define O_EVASION_DUCK       6
#define O_EVASION_FJUMP      7
#define O_EVASION_CARTWHEEL  8
#define O_EVASION_OTHER      9

/* bg_parryDebounce[NUM_FORCE_POWER_LEVELS] (bg_saber.c) — verbatim. */
static const int jka_bg_parryDebounce[4] = {
	500, /*if don't even have defense, can't use defense!*/
	300,
	150,
	50,
};

int jka_jedi_recalcparrytime(int has_client, int number, int saberDefenseLevel,
                             int npc_present, int npc_class, int npc_rank,
                             int torsoTimer, int saberInFlight, int evasionType,
                             int realCombat, int spskill)
{
	if ( !has_client )
	{
		return 0;
	}
	if ( !number )
	{//player
		return jka_bg_parryDebounce[saberDefenseLevel];
	}
	else if ( npc_present )
	{
		if ( !realCombat
			&& ( spskill == 2 || (spskill == 1 && npc_class == O_CLASS_TAVION) ) )
		{
			if ( npc_class == O_CLASS_TAVION )
			{
				return 0;
			}
			else
			{
				return Q_irand( 0, 150 );
			}
		}
		else
		{
			int	baseTime;
			if ( evasionType == O_EVASION_DODGE )
			{
				baseTime = torsoTimer;
			}
			else if ( evasionType == O_EVASION_CARTWHEEL )
			{
				baseTime = torsoTimer;
			}
			else if ( saberInFlight )
			{
				baseTime = Q_irand( 1, 3 ) * 50;
			}
			else
			{
				if ( realCombat )
				{
					baseTime = 500;

					switch ( spskill )
					{
					case 0:
						baseTime = 500;
						break;
					case 1:
						baseTime = 300;
						break;
					case 2:
					default:
						baseTime = 100;
						break;
					}
				}
				else
				{
					baseTime = 150;//500;

					switch ( spskill )
					{
					case 0:
						baseTime = 200;//500;
						break;
					case 1:
						baseTime = 100;//300;
						break;
					case 2:
					default:
						baseTime = 50;//100;
						break;
					}
				}
				if ( npc_class == O_CLASS_TAVION )
				{//Tavion is faster
					baseTime = ceil(baseTime/2.0f);
				}
				else if ( npc_rank >= O_RANK_LT_JG )
				{//fencers, bosses, shadowtroopers, luke, desann, et al use the norm
					if ( Q_irand( 0, 2 ) )
					{//medium speed parry
						baseTime = baseTime;
					}
					else
					{//with the occasional fast parry
						baseTime = ceil(baseTime/2.0f);
					}
				}
				else if ( npc_rank == O_RANK_CIVILIAN )
				{//grunts are slowest
					baseTime = baseTime*Q_irand(1,3);
				}
				else if ( npc_rank == O_RANK_CREWMAN )
				{//acrobats aren't so bad
					if ( evasionType == O_EVASION_PARRY
						|| evasionType == O_EVASION_DUCK_PARRY
						|| evasionType == O_EVASION_JUMP_PARRY )
					{//slower with parries
						baseTime = baseTime*Q_irand(1,2);
					}
					else
					{//faster with acrobatics
						//baseTime = baseTime;
					}
				}
				else
				{//force users are kinda slow
					baseTime = baseTime*Q_irand(1,2);
				}
				if ( evasionType == O_EVASION_DUCK || evasionType == O_EVASION_DUCK_PARRY )
				{
					baseTime += 100;
				}
				else if ( evasionType == O_EVASION_JUMP || evasionType == O_EVASION_JUMP_PARRY )
				{
					baseTime += 50;
				}
				else if ( evasionType == O_EVASION_OTHER )
				{
					baseTime += 100;
				}
				else if ( evasionType == O_EVASION_FJUMP )
				{
					baseTime += 100;
				}
			}

			return baseTime;
		}
	}
	return 0;
}
