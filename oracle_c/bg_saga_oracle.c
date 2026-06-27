/* Oracle extraction of bg_saga.c's parsing helpers (refs/raven-jediacademy/codemp/game/bg_saga.c).
 *
 * Verbatim copies of the self-contained text/group parsers, renamed `jka_` to avoid
 * colliding with anything in the test binary. The Rust port in
 * src/codemp/game/bg_saga.rs is asserted byte-for-byte against these.
 *
 * Substitutions, all behavior-preserving for the tested (well-formed) inputs:
 *   - Q_stricmp is a faithful static copy of the q_shared.c original (same one used
 *     by oracle/bg_misc_parsefield_oracle.c).
 *   - Com_Error(ERR_DROP, ...) becomes a stub that aborts: the parity tests only feed
 *     well-formed input, so the error paths are never taken on either side; aborting
 *     turns any accidental trigger into a hard test failure rather than silent drift. */

#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>

typedef int qboolean;
#define qfalse 0
#define qtrue 1

#define ERR_DROP 1
static void jka_saga_com_error(const char *fmt, ...)
{
	va_list ap;
	fprintf(stderr, "oracle bg_saga Com_Error: ");
	va_start(ap, fmt);
	vfprintf(stderr, fmt, ap);
	va_end(ap);
	fprintf(stderr, "\n");
	abort();
}
#define Com_Error(level, ...) jka_saga_com_error(__VA_ARGS__)

/* faithful static copy of q_shared.c Q_stricmpn / Q_stricmp */
static int jka_Q_stricmpn(const char *s1, const char *s2, int n)
{
	int c1, c2;

	if (s1 == NULL) {
		if (s2 == NULL)
			return 0;
		else
			return -1;
	} else if (s2 == NULL)
		return 1;

	do {
		c1 = *s1++;
		c2 = *s2++;

		if (!n--) {
			return 0; // strings are equal until end point
		}

		if (c1 != c2) {
			if (c1 >= 'a' && c1 <= 'z') {
				c1 -= ('a' - 'A');
			}
			if (c2 >= 'a' && c2 <= 'z') {
				c2 -= ('a' - 'A');
			}
			if (c1 != c2) {
				return c1 < c2 ? -1 : 1;
			}
		}
	} while (c1);

	return 0; // strings are equal
}

static int jka_Q_stricmp(const char *s1, const char *s2)
{
	return (s1 && s2) ? jka_Q_stricmpn(s1, s2, 99999) : -1;
}

#define SIEGECHAR_TAB 9 //perhaps a bit hacky, but I don't think there's any define existing for "tab"

/* BG_SiegeStripTabs (bg_saga.c:170) — convert tabs to spaces in place. */
void jka_BG_SiegeStripTabs(char *buf)
{
	int i = 0;
	int i_r = 0;

	while (buf[i])
	{
		if (buf[i] != SIEGECHAR_TAB)
		{ //not a tab, just stick it in
			buf[i_r] = buf[i];
		}
		else
		{ //If it's a tab, convert it to a space.
			buf[i_r] = ' ';
		}

		i_r++;
		i++;
	}

	buf[i_r] = '\0';
}

/* BG_SiegeGetValueGroup (bg_saga.c:193) — extract the named { } group from buf into
 * outbuf. Verbatim but for Q_stricmp -> jka_Q_stricmp and BG_SiegeStripTabs ->
 * jka_BG_SiegeStripTabs. */
int jka_BG_SiegeGetValueGroup(char *buf, char *group, char *outbuf)
{
	int i = 0;
	int j;
	char checkGroup[4096];
	qboolean isGroup;
	int parseGroups = 0;

	while (buf[i])
	{
		if (buf[i] != ' ' && buf[i] != '{' && buf[i] != '}' && buf[i] != '\n' && buf[i] != '\r' && buf[i] != SIEGECHAR_TAB)
		{ //we're on a valid character
			if (buf[i] == '/' &&
				buf[i+1] == '/')
			{ //this is a comment, so skip over it
				while (buf[i] && buf[i] != '\n' && buf[i] != '\r' && buf[i] != SIEGECHAR_TAB)
				{
					i++;
				}
			}
			else
			{ //parse to the next space/endline/eos and check this value against our group value.
				j = 0;

				while (buf[i] != ' ' && buf[i] != '\n' && buf[i] != '\r' && buf[i] != SIEGECHAR_TAB && buf[i] != '{' && buf[i])
				{
					if (buf[i] == '/' && buf[i+1] == '/')
					{ //hit a comment, break out.
						break;
					}

					checkGroup[j] = buf[i];
					j++;
					i++;
				}
				checkGroup[j] = 0;

				//Make sure this is a group as opposed to a globally defined value.
				if (buf[i] == '/' && buf[i+1] == '/')
				{ //stopped on a comment, so first parse to the end of it.
                    while (buf[i] && buf[i] != '\n' && buf[i] != '\r')
					{
						i++;
					}
					while (buf[i] == '\n' || buf[i] == '\r')
					{
						i++;
					}
				}

				if (!buf[i])
				{
					Com_Error(ERR_DROP, "Unexpected EOF while looking for group '%s'", group);
				}

				isGroup = qfalse;

				while (buf[i] && buf[i] == ' ' || buf[i] == SIEGECHAR_TAB || buf[i] == '\n' || buf[i] == '\r')
				{ //parse to the next valid character
					i++;
				}

				if (buf[i] == '{')
				{ //if the next valid character is an opening bracket, then this is indeed a group
					isGroup = qtrue;
				}

				//Is this the one we want?
				if (isGroup && !jka_Q_stricmp(checkGroup, group))
				{ //guess so. Parse until we hit the { indicating the beginning of the group.
					while (buf[i] != '{' && buf[i])
					{
						i++;
					}

					if (buf[i])
					{ //We're at the start of the group now, so parse to the closing bracket.
						j = 0;

						parseGroups = 0;

						while ((buf[i] != '}' || parseGroups) && buf[i])
						{
							if (buf[i] == '{')
							{ //increment for the opening bracket.
								parseGroups++;
							}
							else if (buf[i] == '}')
							{ //decrement for the closing bracket
								parseGroups--;
							}

							if (parseGroups < 0)
							{ //Syntax error, I guess.
								Com_Error(ERR_DROP, "Found a closing bracket without an opening bracket while looking for group '%s'", group);
							}

							if ((buf[i] != '{' || parseGroups > 1) &&
								(buf[i] != '}' || parseGroups > 0))
							{ //don't put the start and end brackets for this group into the output buffer
								outbuf[j] = buf[i];
								j++;
							}

							if (buf[i] == '}' && !parseGroups)
							{ //Alright, we can break out now.
								break;
							}

							i++;
						}
						outbuf[j] = 0;

						//Verify that we ended up on the closing bracket.
						if (buf[i] != '}')
						{
							Com_Error(ERR_DROP, "Group '%s' is missing a closing bracket", group);
						}

						//Strip the tabs so we're friendly for value parsing.
						jka_BG_SiegeStripTabs(outbuf);

						return 1; //we got it, so return 1.
					}
					else
					{
						Com_Error(ERR_DROP, "Error parsing group in file, unexpected EOF before opening bracket while looking for group '%s'", group);
					}
				}
				else if (!isGroup)
				{ //if it wasn't a group, parse to the end of the line
					while (buf[i] && buf[i] != '\n' && buf[i] != '\r')
					{
						i++;
					}
				}
				else
				{ //this was a group but we not the one we wanted to find, so parse by it.
					parseGroups = 0;

					while (buf[i] && (buf[i] != '}' || parseGroups))
					{
						if (buf[i] == '{')
						{
							parseGroups++;
						}
						else if (buf[i] == '}')
						{
							parseGroups--;
						}

						if (parseGroups < 0)
						{ //Syntax error, I guess.
							Com_Error(ERR_DROP, "Found a closing bracket without an opening bracket while looking for group '%s'", group);
						}

						if (buf[i] == '}' && !parseGroups)
						{ //Alright, we can break out now.
							break;
						}

						i++;
					}

					if (buf[i] != '}')
					{
						Com_Error(ERR_DROP, "Found an opening bracket without a matching closing bracket while looking for group '%s'", group);
					}

					i++;
				}
			}
		}
		else if (buf[i] == '{')
		{ //we're in a group that isn't the one we want, so parse to the end.
			parseGroups = 0;

			while (buf[i] && (buf[i] != '}' || parseGroups))
			{
				if (buf[i] == '{')
				{
					parseGroups++;
				}
				else if (buf[i] == '}')
				{
					parseGroups--;
				}

				if (parseGroups < 0)
				{ //Syntax error, I guess.
					Com_Error(ERR_DROP, "Found a closing bracket without an opening bracket while looking for group '%s'", group);
				}

				if (buf[i] == '}' && !parseGroups)
				{ //Alright, we can break out now.
					break;
				}

				i++;
			}

			if (buf[i] != '}')
			{
				Com_Error(ERR_DROP, "Found an opening bracket without a matching closing bracket while looking for group '%s'", group);
			}
		}

		if (!buf[i])
		{
			break;
		}
		i++;
	}

	return 0; //guess we never found it.
}

/* BG_SiegeGetPairedValue (bg_saga.c:410) — find a top-level "key value" pair in buf
 * and copy the value into outbuf. Verbatim but for Q_stricmp -> jka_Q_stricmp. */
int jka_BG_SiegeGetPairedValue(char *buf, char *key, char *outbuf)
{
	int i = 0;
	int j;
	int k;
	char checkKey[4096];

	while (buf[i])
	{
		if (buf[i] != ' ' && buf[i] != '{' && buf[i] != '}' && buf[i] != '\n' && buf[i] != '\r')
		{ //we're on a valid character
			if (buf[i] == '/' &&
				buf[i+1] == '/')
			{ //this is a comment, so skip over it
				while (buf[i] && buf[i] != '\n' && buf[i] != '\r')
				{
					i++;
				}
			}
			else
			{ //parse to the next space/endline/eos and check this value against our key value.
				j = 0;

				while (buf[i] != ' ' && buf[i] != '\n' && buf[i] != '\r' && buf[i] != SIEGECHAR_TAB && buf[i])
				{
					if (buf[i] == '/' && buf[i+1] == '/')
					{ //hit a comment, break out.
						break;
					}

					checkKey[j] = buf[i];
					j++;
					i++;
				}
				checkKey[j] = 0;

				k = i;

				while (buf[k] && (buf[k] == ' ' || buf[k] == '\n' || buf[k] == '\r'))
				{
					k++;
				}

				if (buf[k] == '{')
				{ //this is not the start of a value but rather of a group. We don't want to look in subgroups so skip over the whole thing.
					int openB = 0;

					while (buf[i] && (buf[i] != '}' || openB))
					{
						if (buf[i] == '{')
						{
							openB++;
						}
						else if (buf[i] == '}')
						{
							openB--;
						}

						if (openB < 0)
						{
							Com_Error(ERR_DROP, "Unexpected closing bracket (too many) while parsing to end of group '%s'", checkKey);
						}

						if (buf[i] == '}' && !openB)
						{ //this is the end of the group
							break;
						}
						i++;
					}

					if (buf[i] == '}')
					{
						i++;
					}
				}
				else
				{
					//Is this the one we want?
					if (buf[i] != '/' || buf[i+1] != '/')
					{ //make sure we didn't stop on a comment, if we did then this is considered an error in the file.
						if (!jka_Q_stricmp(checkKey, key))
						{ //guess so. Parse along to the next valid character, then put that into the output buffer and return 1.
							while ((buf[i] == ' ' || buf[i] == '\n' || buf[i] == '\r' || buf[i] == SIEGECHAR_TAB) && buf[i])
							{
								i++;
							}

							if (buf[i])
							{ //We're at the start of the value now.
								qboolean parseToQuote = qfalse;

								if (buf[i] == '\"')
								{ //if the value is in quotes, then stop at the next quote instead of ' '
									i++;
									parseToQuote = qtrue;
								}

								j = 0;
								while ( ((!parseToQuote && buf[i] != ' ' && buf[i] != '\n' && buf[i] != '\r') || (parseToQuote && buf[i] != '\"')) )
								{
									if (buf[i] == '/' &&
										buf[i+1] == '/')
									{ //hit a comment after the value? This isn't an ideal way to be writing things, but we'll support it anyway.
										break;
									}
									outbuf[j] = buf[i];
									j++;
									i++;

									if (!buf[i])
									{
										if (parseToQuote)
										{
											Com_Error(ERR_DROP, "Unexpected EOF while looking for endquote, error finding paired value for '%s'", key);
										}
										else
										{
											Com_Error(ERR_DROP, "Unexpected EOF while looking for space or endline, error finding paired value for '%s'", key);
										}
									}
								}
								outbuf[j] = 0;

								return 1; //we got it, so return 1.
							}
							else
							{
								Com_Error(ERR_DROP, "Error parsing file, unexpected EOF while looking for valud '%s'", key);
							}
						}
						else
						{ //if that wasn't the desired key, then make sure we parse to the end of the line, so we don't mistake a value for a key
							while (buf[i] && buf[i] != '\n')
							{
								i++;
							}
						}
					}
					else
					{
						Com_Error(ERR_DROP, "Error parsing file, found comment, expected value for '%s'", key);
					}
				}
			}
		}

		if (!buf[i])
		{
			break;
		}
		i++;
	}

	return 0; //guess we never found it.
}

/* ---- BG_SiegeTranslate* (bg_saga.c:573 / :690) ---- */
#include <string.h>

#define NUM_FORCE_POWERS 18
#define FORCE_LEVEL_3 3
#define FORCE_LEVEL_5 5

typedef struct { char *name; int id; } stringID_table_t;

/* verbatim siegeClass_t layout (bg_saga.h:55) so a Rust siegeClass_t pointer can be
 * passed in unchanged — only forcePowerLevels is touched here. */
typedef struct {
	char		name[512];
	char		forcedModel[256];
	char		forcedSkin[256];
	char		saber1[64];
	char		saber2[64];
	int			saberStance;
	int			weapons;
	int			forcePowerLevels[NUM_FORCE_POWERS];
	int			classflags;
	int			maxhealth;
	int			starthealth;
	int			maxarmor;
	int			startarmor;
	float		speed;
	qboolean	hasForcedSaberColor;
	int			forcedSaberColor;
	qboolean	hasForcedSaber2Color;
	int			forcedSaber2Color;
	int			invenItems;
	int			powerups;
	int			uiPortraitShader;
	char		uiPortrait[256];
	int			classShader;
	short		playerClass;
} siegeClass_t;

/* verbatim FPTable (bg_saga.c:102): names in FP_ order, { "", -1 } terminator. The
 * translate fn maps by index, so only the order/names and the -1 terminator matter. */
static stringID_table_t FPTable[] =
{
	{"FP_HEAL", 0},
	{"FP_LEVITATION", 1},
	{"FP_SPEED", 2},
	{"FP_PUSH", 3},
	{"FP_PULL", 4},
	{"FP_TELEPATHY", 5},
	{"FP_GRIP", 6},
	{"FP_LIGHTNING", 7},
	{"FP_RAGE", 8},
	{"FP_PROTECT", 9},
	{"FP_ABSORB", 10},
	{"FP_TEAM_HEAL", 11},
	{"FP_TEAM_FORCE", 12},
	{"FP_DRAIN", 13},
	{"FP_SEE", 14},
	{"FP_SABER_OFFENSE", 15},
	{"FP_SABER_DEFENSE", 16},
	{"FP_SABERTHROW", 17},
	{"", -1},
};

/* BG_SiegeTranslateForcePowers (bg_saga.c:573). Verbatim but for Q_stricmp ->
 * jka_Q_stricmp. */
void jka_BG_SiegeTranslateForcePowers(char *buf, siegeClass_t *siegeClass)
{
	char checkPower[1024];
	char checkLevel[256];
	int l = 0;
	int k = 0;
	int j = 0;
	int i = 0;
	int parsedLevel = 0;
	qboolean allPowers = qfalse;
	qboolean noPowers = qfalse;

	if (!jka_Q_stricmp(buf, "FP_ALL"))
	{ //this is a special case, just give us all the powers on level 3
		allPowers = qtrue;
	}

	if (buf[0] == '0' && !buf[1])
	{ //no powers then
		noPowers = qtrue;
	}

	//First clear out the powers, or in the allPowers case, give us all level 3.
	while (i < NUM_FORCE_POWERS)
	{
		if (allPowers)
		{
			siegeClass->forcePowerLevels[i] = FORCE_LEVEL_3;
		}
		else
		{
			siegeClass->forcePowerLevels[i] = 0;
		}
		i++;
	}

	if (allPowers || noPowers)
	{ //we're done now then.
		return;
	}

	i = 0;
	while (buf[i])
	{ //parse through the list which is seperated by |, and add all the weapons into a bitflag
		if (buf[i] != ' ' && buf[i] != '|')
		{
			j = 0;

			while (buf[i] && buf[i] != ' ' && buf[i] != '|' && buf[i] != ',')
			{
				checkPower[j] = buf[i];
				j++;
				i++;
			}
			checkPower[j] = 0;

			if (buf[i] == ',')
			{ //parse the power level
				i++;
				l = 0;
				while (buf[i] && buf[i] != ' ' && buf[i] != '|')
				{
					checkLevel[l] = buf[i];
					l++;
					i++;
				}
				checkLevel[l] = 0;
				parsedLevel = atoi(checkLevel);

				//keep sane limits on the powers
				if (parsedLevel < 0)
				{
					parsedLevel = 0;
				}
				if (parsedLevel > FORCE_LEVEL_5)
				{
					parsedLevel = FORCE_LEVEL_5;
				}
			}
			else
			{ //if it's not there, assume level 3 I guess.
				parsedLevel = 3;
			}

			if (checkPower[0])
			{ //Got the name, compare it against the weapon table strings.
				k = 0;

				if (!jka_Q_stricmp(checkPower, "FP_JUMP"))
				{ //haqery
                    strcpy(checkPower, "FP_LEVITATION");
				}

				while (FPTable[k].id != -1 && FPTable[k].name[0])
				{
					if (!jka_Q_stricmp(checkPower, FPTable[k].name))
					{ //found it, add the weapon into the weapons value
						siegeClass->forcePowerLevels[k] = parsedLevel;
						break;
					}
					k++;
				}
			}
		}

		if (!buf[i])
		{
			break;
		}
		i++;
	}
}

/* BG_SiegeTranslateGenericTable (bg_saga.c:690). Verbatim but for Q_stricmp ->
 * jka_Q_stricmp. The Rust caller passes its own ported table pointer (identical
 * stringID_table_t layout), so this drives the very table under test. */
int jka_BG_SiegeTranslateGenericTable(char *buf, stringID_table_t *table, qboolean bitflag)
{
	int items = 0;
	char checkItem[1024];
	int i = 0;
	int j = 0;
	int k = 0;

	if (buf[0] == '0' && !buf[1])
	{ //special case, no items.
		return 0;
	}

	while (buf[i])
	{ //Using basically the same parsing method as we do for weapons and forcepowers.
		if (buf[i] != ' ' && buf[i] != '|')
		{
			j = 0;

			while (buf[i] && buf[i] != ' ' && buf[i] != '|')
			{
				checkItem[j] = buf[i];
				j++;
				i++;
			}
			checkItem[j] = 0;

			if (checkItem[0])
			{
				k = 0;

                while (table[k].name && table[k].name[0])
				{ //go through the list and check the parsed flag name against the hardcoded names
					if (!jka_Q_stricmp(checkItem, table[k].name))
					{ //Got it, so add the value into our items value.
						if (bitflag)
						{
							items |= (1 << table[k].id);
						}
						else
						{ //return the value directly then.
							return table[k].id;
						}
						break;
					}
					k++;
				}
			}
		}

		if (!buf[i])
		{
			break;
		}

		i++;
	}
	return items;
}

/* ---- siege team lookups (bg_saga.c:1073..1181, :1346) ----
 * The theme-based lookups take `stm` (or t1/t2) directly, standing in for the C's
 * BG_SiegeFindThemeForTeam(team) global select. The Rust caller exercises the real
 * global-reading path; passing the same team data here drives identical iteration,
 * and since both sides walk the SAME class objects the returned pointers are
 * bit-identical. */
#define MAX_SIEGE_CLASSES_PER_TEAM 16
typedef struct {
	char			name[512];
	siegeClass_t	*classes[MAX_SIEGE_CLASSES_PER_TEAM];
	int				numClasses;
	int				friendlyShader;
} siegeTeam_t;

#define SIEGETEAM_TEAM1 1
#define SIEGETEAM_TEAM2 2

siegeTeam_t *jka_BG_SiegeFindThemeForTeam(int team, siegeTeam_t *t1, siegeTeam_t *t2)
{
	if (team == SIEGETEAM_TEAM1) return t1;
	else if (team == SIEGETEAM_TEAM2) return t2;
	return NULL;
}

int jka_BG_SiegeCountBaseClass(siegeTeam_t *stm, short classIndex)
{
	int count = 0,i;
	if (!stm) return 0;
	for (i=0;i<stm->numClasses;i++)
	{
		if (stm->classes[i]->playerClass == classIndex)
		{
			count++;
		}
	}
	return count;
}

char *jka_BG_GetUIPortraitFile(siegeTeam_t *stm, short classIndex, short cntIndex)
{
	int count = 0,i;
	if (!stm) return 0;
	for (i=0;i<stm->numClasses;i++)
	{
		if (stm->classes[i]->playerClass == classIndex)
		{
			if (count==cntIndex)
			{
				return stm->classes[i]->uiPortrait;
			}
			++count;
		}
	}
	return 0;
}

int jka_BG_GetUIPortrait(siegeTeam_t *stm, short classIndex, short cntIndex)
{
	int count = 0,i;
	if (!stm) return 0;
	for (i=0;i<stm->numClasses;i++)
	{
		if (stm->classes[i]->playerClass == classIndex)
		{
			if (count==cntIndex)
			{
				return stm->classes[i]->uiPortraitShader;
			}
			++count;
		}
	}
	return 0;
}

siegeClass_t *jka_BG_GetClassOnBaseClass(siegeTeam_t *stm, short classIndex, short cntIndex)
{
	int count = 0,i;
	if (!stm) return 0;
	for (i=0;i<stm->numClasses;i++)
	{
		if (stm->classes[i]->playerClass == classIndex)
		{
			if (count==cntIndex)
			{
				return stm->classes[i];
			}
			++count;
		}
	}
	return 0;
}

/* ---- class/team finders (bg_saga.c:1221, :1418, :1457, :1491) ----
 * Parameterized over the module globals (bgSiegeClasses/bgNumSiegeClasses,
 * bgSiegeTeams/bgNumSiegeTeams, team1Theme/team2Theme) so the test drives the exact
 * data the Rust globals hold; returned pointers are bit-identical. The vacuous
 * `teams[i].name &&` array-truthiness test from the C is dropped (always true). */
siegeClass_t *jka_BG_SiegeFindClassByName(const char *classname, siegeClass_t *classes, int num)
{
	int i = 0;
	while (i < num)
	{
		if (!jka_Q_stricmp(classes[i].name, classname))
		{ //found it
			return &classes[i];
		}
		i++;
	}
	return 0;
}

int jka_BG_SiegeFindClassIndexByName(const char *classname, siegeClass_t *classes, int num)
{
	int i = 0;
	while (i < num)
	{
		if (!jka_Q_stricmp(classes[i].name, classname))
		{ //found it
			return i;
		}
		i++;
	}
	return -1;
}

siegeTeam_t *jka_BG_SiegeFindTeamForTheme(char *themeName, siegeTeam_t *teams, int num)
{
	int i = 0;
	while (i < num)
	{
		if (!jka_Q_stricmp(teams[i].name, themeName))
		{ //this is what we're looking for
			return &teams[i];
		}
		i++;
	}
	return 0;
}

int jka_BG_SiegeCheckClassLegality(int team, char *classname, siegeTeam_t *t1, siegeTeam_t *t2)
{
	siegeTeam_t **teamPtr = 0;
	int i = 0;

	if (team == SIEGETEAM_TEAM1)
	{
		teamPtr = &t1;
	}
	else if (team == SIEGETEAM_TEAM2)
	{
		teamPtr = &t2;
	}
	else
	{ //spectator? Whatever, you're legal then.
		return 1;
	}

	if (!teamPtr || !(*teamPtr))
	{ //no team theme to begin with.
		return 1;
	}

	while (i < (*teamPtr)->numClasses)
	{
		if (!jka_Q_stricmp(classname, (*teamPtr)->classes[i]->name))
		{ //found it, so it's alright
			return 1;
		}
		i++;
	}

	//Didn't find it, so copy the name of the first valid class over it.
	strcpy(classname, (*teamPtr)->classes[0]->name);

	return 0;
}
