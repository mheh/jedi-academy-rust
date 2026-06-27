/* Oracle: NPC_GetPainChance extracted from the original
   raven-jediacademy/codemp/game/NPC_reactions.c (:157), compared against
   OpenJK/codemp/game/NPC_reactions.c. Compiled and linked under the `oracle` cargo
   feature (see build.rs) so the Rust port can be asserted bit-exact against the real C.

   The function takes `gentity_t *self` but reads only four scalar fields
   (self->enemy != NULL, self->client != NULL, self->health,
   self->client->ps.stats[STAT_MAX_HEALTH]) plus the file-global `g_spskill.integer`.
   Following the bg_misc_oracle.c scalar-marshaling precedent, the wrapper passes those
   as positional scalars and the body is transcribed VERBATIM with the field reads
   substituted by the marshaled locals. */
#include "qshared_shim.h"

/* Verbatim NPC_reactions.c:157, with the four entity-state reads replaced by the
   marshaled scalars (has_enemy / has_client / health / max_health) and
   g_spskill.integer by `spskill`. */
float jka_npc_getpainchance(int has_enemy, int has_client, int health,
                            int max_health, int damage, int spskill)
{
	float pain_chance;
	if ( !has_enemy )
	{//surprised, always take pain
		return 1.0f;
	}

	if (!has_client)
	{
		return 1.0f;
	}

	//if ( damage > self->max_health/2.0f )
	if (damage > max_health/2.0f)
	{
		return 1.0f;
	}

	pain_chance = (float)(max_health-health)/(max_health*2.0f) + (float)damage/(max_health/2.0f);
	switch ( spskill )
	{
	case 0:	//easy
		//return 0.75f;
		break;

	case 1://med
		pain_chance *= 0.5f;
		//return 0.35f;
		break;

	case 2://hard
	default:
		pain_chance *= 0.1f;
		//return 0.05f;
		break;
	}
	//Com_Printf( "%s: %4.2f\n", self->NPC_type, pain_chance );
	return pain_chance;
}
