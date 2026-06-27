/* Extracted w_saber.c functions, compiled as a parity oracle. See the header in
   q_math_oracle.c for the method. Function *bodies* are the authentic Raven source
   from raven-jediacademy/codemp/game/w_saber.c, verbatim except for documented ORACLE
   DEVIATIONs. */

/* The game's rand() lives in bg_lib.c; here it is jka_rand (in bg_lib_oracle.c, same
   static lib). RandFloat's divisor is paired with that LCG, so the oracle must use the
   game's rand(), not libc rand(), to stay faithful to real JKA. */
int jka_rand( void );

/* PC w_saber.c:42 divides by (float)RAND_MAX (the Xbox source used the literal 32768.0F,
   now the commented-out //for linux: line). RAND_MAX for bg_lib's 15-bit LCG is 0x7fff;
   pin it here rather than pull the host stdlib.h RAND_MAX (which is 0x7fffffff on
   macOS/linux and would not match the LCG the oracle's jka_rand() draws from). */
#define RAND_MAX 0x7fff

/* w_saber.c:42. ORACLE DEVIATION: rand() -> jka_rand() (the game's bg_lib LCG, not
   libc); renamed jka_RandFloat to dodge any host symbol. Body otherwise verbatim
   (PC //for linux: divisor (float)RAND_MAX, RAND_MAX pinned to the LCG's 0x7fff above). */
float jka_RandFloat(float min, float max) {
//	return ((jka_rand() * (max - min)) / 32768.0F) + min;
//for linux:
	return ((jka_rand() * (max - min)) / (float)RAND_MAX) + min;
}

/* Mirror of the engine-side enum constants the body branches on
   (codemp/game/bg_public.h / bg_weapons.h). */
#define GT_JEDIMASTER	2
#define GT_DUEL			3
#define GT_POWERDUEL	4
#define WP_NONE			0
#define WP_SABER		3
#define WP_NUM_WEAPONS	19

/* w_saber.c:9006. ORACLE DEVIATION: the three cvar globals it reads
   (g_gametype.integer, g_duelWeaponDisable.integer, g_weaponDisable.integer) are passed in
   as ints so the oracle is a pure function — the body's control flow is otherwise verbatim.
   Renamed to jka_HasSetSaberOnly to dodge any host symbol; qboolean -> int. */
int jka_HasSetSaberOnly(int g_gametype, int g_duelWeaponDisable, int g_weaponDisable) {
	int i = 0;
	int wDisable = 0;

	if (g_gametype == GT_JEDIMASTER)
	{ /* set to 0 */
		return 0;
	}

	if (g_gametype == GT_DUEL || g_gametype == GT_POWERDUEL)
	{
		wDisable = g_duelWeaponDisable;
	}
	else
	{
		wDisable = g_weaponDisable;
	}

	while (i < WP_NUM_WEAPONS)
	{
		if (!(wDisable & (1 << i)) &&
			i != WP_SABER && i != WP_NONE)
		{
			return 0;
		}

		i++;
	}

	return 1;
}

/* w_saber.c:4858. The vec3_t epsilon-equality helper used by the saber damage trace.
   Renamed to jka_VectorCompare2; body otherwise verbatim. (vec3_t -> float[3].) */
int jka_VectorCompare2( const float v1[3], const float v2[3] ) {
	if ( v1[0] > v2[0]+0.0001f || v1[0] < v2[0]-0.0001f
		|| v1[1] > v2[1]+0.0001f || v1[1] < v2[1]-0.0001f
		|| v1[2] > v2[2]+0.0001f || v1[2] < v2[2]-0.0001f ) {
		return 0;
	}
	return 1;
}

/* Mirror of the saber_colors_t enumerators the body branches on (q_shared.h). */
#define SABER_RED		0
#define SABER_ORANGE	1
#define SABER_YELLOW	2
#define SABER_GREEN		3
#define SABER_BLUE		4
#define SABER_PURPLE	5

/* w_saber.c:2677. Maps a saber color to its packed debug-line RGBA int.
   Renamed to jka_WPDEBUG_SaberColor; saber_colors_t -> int; body otherwise verbatim. */
int jka_WPDEBUG_SaberColor( int saberColor ) {
	switch( (int)(saberColor) )
	{
		case SABER_RED:
			return 0x000000ff;
			break;
		case SABER_ORANGE:
			return 0x000088ff;
			break;
		case SABER_YELLOW:
			return 0x0000ffff;
			break;
		case SABER_GREEN:
			return 0x0000ff00;
			break;
		case SABER_BLUE:
			return 0x00ff0000;
			break;
		case SABER_PURPLE:
			return 0x00ff00ff;
			break;
		default:
			return 0x00ffffff;
			break;
	}
}

/* w_saber.c:7529. Float near-equality predicate used by the grapple/grab code.
   Renamed to jka_G_PrettyCloseIGuess; qboolean -> int; body otherwise verbatim. */
int jka_G_PrettyCloseIGuess(float a, float b, float tolerance) {
    if ((a-b) < tolerance &&
		(a-b) > -tolerance)
	{
		return 1;
	}

	return 0;
}

/* w_saber.c:2642. Largest blade lengthMax over a saber's blades. Renamed to
   jka_WP_SaberBladeLength; pass-the-read-fields precedent: the C reads only
   saber->numBlades and saber->blade[i].lengthMax, so the oracle takes those two
   (the lengthMax array + numBlades) instead of the full saberInfo_t. Loop body
   otherwise verbatim. */
float jka_WP_SaberBladeLength( const float lengthMax[], int numBlades ) {
	int	i;
	float len = 0.0f;
	for ( i = 0; i < numBlades; i++ )
	{
		if ( lengthMax[i] > len )
		{
			len = lengthMax[i];
		}
	}
	return len;
}

/* w_saber.c:2291. Back-attack saber-move predicate. Renamed to jka_G_SaberInBackAttack;
   qboolean -> int; the LS_A_BACK* enum values are inlined as their numeric constants
   (LS_A_BACKSTAB=11, LS_A_BACK=12, LS_A_BACK_CR=13). Body otherwise verbatim. */
int jka_G_SaberInBackAttack(int move) {
	switch (move)
	{
	case 12: /* LS_A_BACK */
	case 13: /* LS_A_BACK_CR */
	case 11: /* LS_A_BACKSTAB */
		return 1;
	}

	return 0;
}

/* Mirror of the saberBlockedType_t enumerators the body branches on (q_shared.h). */
#define BLOCKED_UPPER_RIGHT			4
#define BLOCKED_UPPER_LEFT			5
#define BLOCKED_LOWER_RIGHT			6
#define BLOCKED_LOWER_LEFT			7
#define BLOCKED_TOP					8
#define BLOCKED_UPPER_RIGHT_PROJ	9
#define BLOCKED_UPPER_LEFT_PROJ		10
#define BLOCKED_LOWER_RIGHT_PROJ	11
#define BLOCKED_LOWER_LEFT_PROJ		12
#define BLOCKED_TOP_PROJ			13

/* w_saber.c:8657. Maps a directional saber-block to its projectile-reflect counterpart.
   Renamed to jka_WP_MissileBlockForBlock; body otherwise verbatim. */
int jka_WP_MissileBlockForBlock( int saberBlock ) {
	switch( saberBlock )
	{
	case BLOCKED_UPPER_RIGHT:
		return BLOCKED_UPPER_RIGHT_PROJ;
		break;
	case BLOCKED_UPPER_LEFT:
		return BLOCKED_UPPER_LEFT_PROJ;
		break;
	case BLOCKED_LOWER_RIGHT:
		return BLOCKED_LOWER_RIGHT_PROJ;
		break;
	case BLOCKED_LOWER_LEFT:
		return BLOCKED_LOWER_LEFT_PROJ;
		break;
	case BLOCKED_TOP:
		return BLOCKED_TOP_PROJ;
		break;
	}
	return saberBlock;
}

/* Mirror of the saberMoveName_t parry/reflect enumerators the body returns (bg_public.h). */
#define LS_NONE			0
#define LS_PARRY_UP		152
#define LS_PARRY_UR		153
#define LS_PARRY_UL		154
#define LS_PARRY_LR		155
#define LS_PARRY_LL		156
#define LS_REFLECT_UP	157
#define LS_REFLECT_UR	158
#define LS_REFLECT_UL	159
#define LS_REFLECT_LR	160
#define LS_REFLECT_LL	161

/* w_saber.c:1764. Maps a saber-block result to its parry (directional) or reflect (*_PROJ)
   saber move. Renamed to jka_G_GetParryForBlock; body otherwise verbatim. */
int jka_G_GetParryForBlock(int block) {
	switch (block)
	{
		case BLOCKED_UPPER_RIGHT:
			return LS_PARRY_UR;
			break;
		case BLOCKED_UPPER_RIGHT_PROJ:
			return LS_REFLECT_UR;
			break;
		case BLOCKED_UPPER_LEFT:
			return LS_PARRY_UL;
			break;
		case BLOCKED_UPPER_LEFT_PROJ:
			return LS_REFLECT_UL;
			break;
		case BLOCKED_LOWER_RIGHT:
			return LS_PARRY_LR;
			break;
		case BLOCKED_LOWER_RIGHT_PROJ:
			return LS_REFLECT_LR;
			break;
		case BLOCKED_LOWER_LEFT:
			return LS_PARRY_LL;
			break;
		case BLOCKED_LOWER_LEFT_PROJ:
			return LS_REFLECT_LL;
			break;
		case BLOCKED_TOP:
			return LS_PARRY_UP;
			break;
		case BLOCKED_TOP_PROJ:
			return LS_REFLECT_UP;
			break;
		default:
			break;
	}

	return LS_NONE;
}

/* Mirror of the saberMoveName_t knockaway enumerators the body returns (bg_public.h). */
#define LS_K1_T_	147
#define LS_K1_TR	148
#define LS_K1_TL	149
#define LS_K1_BR	150
#define LS_K1_BL	151

/* w_saber.c:2083. Maps a parry move to its knockaway animation. Renamed to
   jka_G_KnockawayForParry; body otherwise verbatim (the missing terminal return is
   covered by the default case, exactly as the original). */
int jka_G_KnockawayForParry( int move ) {
	switch ( move )
	{
	case LS_PARRY_UP:
		return LS_K1_T_;
		break;
	case LS_PARRY_UR:
	default:
		return LS_K1_TR;
		break;
	case LS_PARRY_UL:
		return LS_K1_TL;
		break;
	case LS_PARRY_LR:
		return LS_K1_BR;
		break;
	case LS_PARRY_LL:
		return LS_K1_BL;
		break;
	}
}

/* Mirror of the saber_styles_t enumerators the body branches on (q_shared.h). */
#define SS_FAST		1
#define SS_TAVION	5
#define SS_DUAL		6
#define SS_STAFF	7

/* Mirror of the SABERLOCK_* enumerators (bg_public.h). */
#define SABERLOCK_TOP			0
#define SABERLOCK_LOCK			2
#define SABERLOCK_SUPERBREAK	4
#define SABERLOCK_WIN			5
#define SABERLOCK_LOSE			6

/* Mirror of the BOTH_LK_* saber-lock animNumber_t values the body returns (anims.h). */
#define BOTH_LK_S_DL_S_B_1_L	740
#define BOTH_LK_S_ST_S_B_1_L	750
#define BOTH_LK_S_S_S_B_1_L		760
#define BOTH_LK_DL_DL_S_B_1_L	770
#define BOTH_LK_DL_ST_S_B_1_L	780
#define BOTH_LK_DL_S_S_B_1_L	790
#define BOTH_LK_ST_DL_S_B_1_L	800
#define BOTH_LK_ST_ST_S_B_1_L	810
#define BOTH_LK_ST_S_S_B_1_L	820
#define BOTH_LK_S_S_S_L_2		830
#define BOTH_LK_S_S_T_L_2		831
#define BOTH_LK_DL_DL_S_L_2		832
#define BOTH_LK_DL_DL_T_L_2		833
#define BOTH_LK_ST_ST_S_L_2		834
#define BOTH_LK_ST_ST_T_L_2		835

/* w_saber.c:967. Picks the BOTH_LK_* saber-lock animation for a lock/break/superbreak pose.
   Renamed to jka_G_SaberLockAnim; body otherwise verbatim. */
int jka_G_SaberLockAnim( int attackerSaberStyle, int defenderSaberStyle, int topOrSide, int lockOrBreakOrSuperBreak, int winOrLose )
{
	int baseAnim = -1;
	if ( lockOrBreakOrSuperBreak == SABERLOCK_LOCK )
	{//special case: if we're using the same style and locking
		if ( attackerSaberStyle == defenderSaberStyle
			|| (attackerSaberStyle>=SS_FAST&&attackerSaberStyle<=SS_TAVION&&defenderSaberStyle>=SS_FAST&&defenderSaberStyle<=SS_TAVION) )
		{//using same style
			if ( winOrLose == SABERLOCK_LOSE )
			{//you want the defender's stance...
				switch ( defenderSaberStyle )
				{
				case SS_DUAL:
					if ( topOrSide == SABERLOCK_TOP )
					{
						baseAnim = BOTH_LK_DL_DL_T_L_2;
					}
					else
					{
						baseAnim = BOTH_LK_DL_DL_S_L_2;
					}
					break;
				case SS_STAFF:
					if ( topOrSide == SABERLOCK_TOP )
					{
						baseAnim = BOTH_LK_ST_ST_T_L_2;
					}
					else
					{
						baseAnim = BOTH_LK_ST_ST_S_L_2;
					}
					break;
				default:
					if ( topOrSide == SABERLOCK_TOP )
					{
						baseAnim = BOTH_LK_S_S_T_L_2;
					}
					else
					{
						baseAnim = BOTH_LK_S_S_S_L_2;
					}
					break;
				}
			}
		}
	}
	if ( baseAnim == -1 )
	{
		switch ( attackerSaberStyle )
		{
		case SS_DUAL:
			switch ( defenderSaberStyle )
			{
				case SS_DUAL:
					baseAnim = BOTH_LK_DL_DL_S_B_1_L;
					break;
				case SS_STAFF:
					baseAnim = BOTH_LK_DL_ST_S_B_1_L;
					break;
				default://single
					baseAnim = BOTH_LK_DL_S_S_B_1_L;
					break;
			}
			break;
		case SS_STAFF:
			switch ( defenderSaberStyle )
			{
				case SS_DUAL:
					baseAnim = BOTH_LK_ST_DL_S_B_1_L;
					break;
				case SS_STAFF:
					baseAnim = BOTH_LK_ST_ST_S_B_1_L;
					break;
				default://single
					baseAnim = BOTH_LK_ST_S_S_B_1_L;
					break;
			}
			break;
		default://single
			switch ( defenderSaberStyle )
			{
				case SS_DUAL:
					baseAnim = BOTH_LK_S_DL_S_B_1_L;
					break;
				case SS_STAFF:
					baseAnim = BOTH_LK_S_ST_S_B_1_L;
					break;
				default://single
					baseAnim = BOTH_LK_S_S_S_B_1_L;
					break;
			}
			break;
		}
		//side lock or top lock?
		if ( topOrSide == SABERLOCK_TOP )
		{
			baseAnim += 5;
		}
		//lock, break or superbreak?
		if ( lockOrBreakOrSuperBreak == SABERLOCK_LOCK )
		{
			baseAnim += 2;
		}
		else
		{//a break or superbreak
			if ( lockOrBreakOrSuperBreak == SABERLOCK_SUPERBREAK )
			{
				baseAnim += 3;
			}
			//winner or loser?
			if ( winOrLose == SABERLOCK_WIN )
			{
				baseAnim += 1;
			}
		}
	}
	return baseAnim;
}

/* w_saber.c:2436. Saber-collision plane-equation helper (-rww). Renamed to
   jka_G_SabCol_CalcPlaneEq; vec3_t -> float[3]; body otherwise verbatim. */
void jka_G_SabCol_CalcPlaneEq( const float x[3], const float y[3], const float z[3], float *planeEq ) {
	planeEq[0] = x[1]*(y[2]-z[2]) + y[1]*(z[2]-x[2]) + z[1]*(x[2]-y[2]);
	planeEq[1] = x[2]*(y[0]-z[0]) + y[2]*(z[0]-x[0]) + z[2]*(x[0]-y[0]);
	planeEq[2] = x[0]*(y[1]-z[1]) + y[0]*(z[1]-x[1]) + z[0]*(x[1]-y[1]);
	planeEq[3] = -(x[0]*(y[1]*z[2] - z[1]*y[2]) + y[0]*(z[1]*x[2] - x[1]*z[2]) + z[0]*(x[1]*y[2] - y[1]*x[2]) );
}

/* w_saber.c:2445. Saber-collision point-vs-plane classifier (-rww). Renamed to
   jka_G_SabCol_PointRelativeToPlane; vec3_t -> float[3]; body otherwise verbatim. */
int jka_G_SabCol_PointRelativeToPlane( const float pos[3], float *side, const float *planeEq ) {
	*side = planeEq[0]*pos[0] + planeEq[1]*pos[1] + planeEq[2]*pos[2] + planeEq[3];

	if (*side > 0.0f)
	{
		return 1;
	}
	else if (*side < 0.0f)
	{
		return -1;
	}

	return 0;
}

/* Mirror of the saber_styles_t enumerators G_SaberAttackPower branches on (q_shared.h).
   SS_FAST/SS_DUAL/SS_STAFF already #defined above; add the two missing here. */
#define SS_MEDIUM	2
#define SS_STRONG	3

/* Mirror of the brokenLimb_t bit-flags + gametype enumerators the body reads
   (bg_public.h). GT_DUEL/GT_POWERDUEL already #defined above. */
#define BROKENLIMB_LARM	1
#define BROKENLIMB_RARM	2
#define GT_SIEGE		7
#define DUELTEAM_LONE	1

#include <math.h>

/* w_saber.c:120. ORACLE DEVIATION: the gentity_t/client fields and cvar globals the body
   reads are passed in as scalars so the oracle is a pure function — the math/control flow is
   otherwise verbatim. The #ifndef FINAL_BUILD Com_Printf is dropped (no effect on return).
   VectorSubtract/VectorLength are inlined faithfully. Renamed jka_G_SaberAttackPower;
   qboolean -> int. */
int jka_G_SaberAttackPower(int saberAnimLevel, int attacking,
	int lastSaberStorageTime, int olderIsValid,
	const float lastSaberBase_Always[3], const float olderSaberBase[3],
	int brokenLimbs, int levelTime, int g_gametype, int duelTeam)
{
	int baseLevel;

	baseLevel = saberAnimLevel;

	/* Give "medium" strength for the two special stances. */
	if (baseLevel == SS_DUAL)
	{
		baseLevel = 2;
	}
	else if (baseLevel == SS_STAFF)
	{
		baseLevel = 2;
	}

	if (attacking)
	{ /* the attacker gets a boost to help penetrate defense. */
		baseLevel *= 2;

		baseLevel++;

		if (lastSaberStorageTime >= (levelTime-50) &&
			olderIsValid)
		{
			float vSub[3];
			int swingDist;
			int toleranceAmt;

			switch (saberAnimLevel)
			{
			case SS_STRONG:
				toleranceAmt = 8;
				break;
			case SS_MEDIUM:
				toleranceAmt = 16;
				break;
			case SS_FAST:
				toleranceAmt = 24;
				break;
			default: /* dual, staff, etc. */
				toleranceAmt = 16;
				break;
			}

			vSub[0] = lastSaberBase_Always[0] - olderSaberBase[0];
			vSub[1] = lastSaberBase_Always[1] - olderSaberBase[1];
			vSub[2] = lastSaberBase_Always[2] - olderSaberBase[2];
			swingDist = (int)((float)sqrt(vSub[0]*vSub[0] + vSub[1]*vSub[1] + vSub[2]*vSub[2]));

			while (swingDist > 0)
			{
				baseLevel++;
				swingDist -= toleranceAmt;
			}
		}
	}

	if ((brokenLimbs & (1 << BROKENLIMB_RARM)) ||
		(brokenLimbs & (1 << BROKENLIMB_LARM)))
	{ /* We're very weak when one of our arms is broken */
		baseLevel *= 0.3;
	}

	/* Cap at reasonable values now. */
	if (baseLevel < 1)
	{
		baseLevel = 1;
	}
	else if (baseLevel > 16)
	{
		baseLevel = 16;
	}

	if (g_gametype == GT_POWERDUEL &&
		duelTeam == DUELTEAM_LONE)
	{ /* get more power then */
		return baseLevel*2;
	}
	else if (attacking && g_gametype == GT_SIEGE)
	{ /* in siege, saber battles should be quicker and more biased toward the attacker */
		return baseLevel*3;
	}

	return baseLevel;
}

/* Mirror of the FORCE_LEVEL_* enumerators (q_shared.h) the body returns. */
#define FORCE_LEVEL_0	0
#define FORCE_LEVEL_1	1
#define FORCE_LEVEL_2	2
#define FORCE_LEVEL_3	3
#define FORCE_LEVEL_4	4
#define FORCE_LEVEL_5	5

/* Mirror of the saberType_t enumerators (q_shared.h) the body branches on. */
#define SABER_LANCE		9
#define SABER_TRIDENT	11

/* Mirror of the animNumber_t enumerators (anims.h) the body branches on. These are
   the *real* Raven anims.h numeric values; the parity test feeds the oracle each anim
   by its anims.h value (paired with the corresponding Rust anims.rs constant on the
   Rust side). */
#define BOTH_A1_T__B_			117
#define BOTH_D1_B____			193
#define BOTH_A2_T__B_			194
#define BOTH_D2_B____			270
#define BOTH_A3_T__B_			271
#define BOTH_D3_B____			347
#define BOTH_A4_T__B_			348
#define BOTH_D4_B____			424
#define BOTH_A5_T__B_			425
#define BOTH_D5_B____			501
#define BOTH_A6_T__B_			502
#define BOTH_D6_B____			578
#define BOTH_A7_T__B_			579
#define BOTH_D7_B____			655
#define BOTH_P1_S1_T_			656
#define BOTH_H1_S1_BR			680
#define BOTH_A2_STABBACK1		845
#define BOTH_ATTACK_BACK		846
#define BOTH_CROUCHATTACKBACK1	851
#define BOTH_BUTTERFLY_LEFT		1200
#define BOTH_BUTTERFLY_RIGHT	1201
#define BOTH_BUTTERFLY_FL1		1250
#define BOTH_BUTTERFLY_FR1		1249
#define BOTH_FJSS_TR_BL			1243
#define BOTH_FJSS_TL_BR			1244
#define BOTH_K1_S1_T_			661
#define BOTH_K1_S1_TR			662
#define BOTH_K1_S1_TL			663
#define BOTH_K1_S1_BL			664
#define BOTH_K1_S1_B_			665
#define BOTH_K1_S1_BR			666
#define BOTH_LUNGE2_B__T_		850
#define BOTH_FORCELEAP2_T__B_	849
#define BOTH_VS_ATR_S			1040
#define BOTH_VS_ATL_S			1039
#define BOTH_VT_ATR_S			1078
#define BOTH_VT_ATL_S			1077
#define BOTH_JUMPFLIPSLASHDOWN1	847
#define BOTH_JUMPFLIPSTABDOWN	848
#define BOTH_JUMPATTACK6		852
#define BOTH_JUMPATTACK7		853
#define BOTH_SPINATTACK6		854
#define BOTH_SPINATTACK7		855
#define BOTH_FORCELONGLEAP_ATTACK	861
#define BOTH_STABDOWN			897
#define BOTH_STABDOWN_STAFF		898
#define BOTH_STABDOWN_DUAL		899
#define BOTH_A6_SABERPROTECT	900
#define BOTH_A7_SOULCAL			901
#define BOTH_A1_SPECIAL			902
#define BOTH_A2_SPECIAL			903
#define BOTH_A3_SPECIAL			904
#define BOTH_FLIP_ATTACK7		890
#define BOTH_PULL_IMPALE_STAB	893
#define BOTH_PULL_IMPALE_SWING	894
#define BOTH_ALORA_SPIN_SLASH	1264
#define BOTH_A6_FB				1255
#define BOTH_A6_LR				1256
#define BOTH_A7_HILT			1257
#define BOTH_LK_S_DL_T_SB_1_W	740
#define BOTH_LK_S_ST_S_SB_1_W	745
#define BOTH_LK_S_DL_S_SB_1_W	735
#define BOTH_LK_S_S_S_SB_1_W	755
#define BOTH_LK_S_ST_T_SB_1_W	750
#define BOTH_LK_S_S_T_SB_1_W	760
#define BOTH_LK_DL_DL_T_SB_1_W	770
#define BOTH_LK_DL_DL_S_SB_1_W	765
#define BOTH_LK_DL_ST_S_SB_1_W	775
#define BOTH_LK_DL_ST_T_SB_1_W	780
#define BOTH_LK_DL_S_S_SB_1_W	785
#define BOTH_LK_DL_S_T_SB_1_W	790
#define BOTH_LK_ST_DL_S_SB_1_W	795
#define BOTH_LK_ST_DL_T_SB_1_W	800
#define BOTH_LK_ST_ST_S_SB_1_W	805
#define BOTH_LK_ST_S_S_SB_1_W	815
#define BOTH_LK_ST_ST_T_SB_1_W	810
#define BOTH_LK_ST_S_T_SB_1_W	820
#define BOTH_HANG_ATTACK		1294
#define BOTH_ROLL_STAB			905

/* w_saber.c:2859. ORACLE DEVIATION: pass-the-read-fields precedent (cf. jka_G_SaberAttackPower) —
   the C reads ent->client->ps.torsoAnim / .torsoTimer, derives animTimeElapsed via
   BG_AnimLength( ent->localAnimIndex, anim ), and reads ent->client->saber[saberNum].type. The
   oracle takes those four derived scalars (anim, animTimer, animTimeElapsed, saberType) plus the
   saberNum/mySaberHit args so it is a pure deterministic int mapper; the !ent/!client/saberNum>=
   MAX_SABERS guard is exercised on the Rust side (it would deref a null entity here). Renamed
   jka_G_PowerLevelForSaberAnim; gentity_t/saberInfo_t derefs replaced by the passed scalars;
   qboolean -> int. The switch/range body is otherwise verbatim from the C. */
int jka_G_PowerLevelForSaberAnim( int anim, int animTimer, int animTimeElapsed, int saberType, int saberNum, int mySaberHit )
{
	if ( anim >= BOTH_A1_T__B_ && anim <= BOTH_D1_B____ )
	{
		//FIXME: these two need their own style
		if ( saberType == SABER_LANCE )
		{
			return FORCE_LEVEL_4;
		}
		else if ( saberType == SABER_TRIDENT )
		{
			return FORCE_LEVEL_3;
		}
		return FORCE_LEVEL_1;
	}
	if ( anim >= BOTH_A2_T__B_ && anim <= BOTH_D2_B____ )
	{
		return FORCE_LEVEL_2;
	}
	if ( anim >= BOTH_A3_T__B_ && anim <= BOTH_D3_B____ )
	{
		return FORCE_LEVEL_3;
	}
	if ( anim >= BOTH_A4_T__B_ && anim <= BOTH_D4_B____ )
	{//desann
		return FORCE_LEVEL_4;
	}
	if ( anim >= BOTH_A5_T__B_ && anim <= BOTH_D5_B____ )
	{//tavion
		return FORCE_LEVEL_2;
	}
	if ( anim >= BOTH_A6_T__B_ && anim <= BOTH_D6_B____ )
	{//dual
		return FORCE_LEVEL_2;
	}
	if ( anim >= BOTH_A7_T__B_ && anim <= BOTH_D7_B____ )
	{//staff
		return FORCE_LEVEL_2;
	}
	if ( anim >= BOTH_P1_S1_T_ && anim <= BOTH_H1_S1_BR )
	{//parries, knockaways and broken parries
		return FORCE_LEVEL_1;//FIXME: saberAnimLevel?
	}
	switch ( anim )
	{
	case BOTH_A2_STABBACK1:
		if ( mySaberHit )
		{//someone else hit my saber, not asking for damage level, but defense strength
			return FORCE_LEVEL_1;
		}
		if ( animTimer < 450 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 400 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_ATTACK_BACK:
		if ( animTimer < 500 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_CROUCHATTACKBACK1:
		if ( animTimer < 800 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_BUTTERFLY_LEFT:
	case BOTH_BUTTERFLY_RIGHT:
	case BOTH_BUTTERFLY_FL1:
	case BOTH_BUTTERFLY_FR1:
		//FIXME: break up?
		return FORCE_LEVEL_3;
		break;
	case BOTH_FJSS_TR_BL:
	case BOTH_FJSS_TL_BR:
		//FIXME: break up?
		return FORCE_LEVEL_3;
		break;
	case BOTH_K1_S1_T_:	//# knockaway saber top
	case BOTH_K1_S1_TR:	//# knockaway saber top right
	case BOTH_K1_S1_TL:	//# knockaway saber top left
	case BOTH_K1_S1_BL:	//# knockaway saber bottom left
	case BOTH_K1_S1_B_:	//# knockaway saber bottom
	case BOTH_K1_S1_BR:	//# knockaway saber bottom right
		//FIXME: break up?
		return FORCE_LEVEL_3;
		break;
	case BOTH_LUNGE2_B__T_:
		if ( mySaberHit )
		{//someone else hit my saber, not asking for damage level, but defense strength
			return FORCE_LEVEL_1;
		}
		if ( animTimer < 400 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 150 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_FORCELEAP2_T__B_:
		if ( animTimer < 400 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 550 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_VS_ATR_S:
	case BOTH_VS_ATL_S:
	case BOTH_VT_ATR_S:
	case BOTH_VT_ATL_S:
		return FORCE_LEVEL_3;//???
		break;
	case BOTH_JUMPFLIPSLASHDOWN1:
		if ( animTimer <= 1000 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 600 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_JUMPFLIPSTABDOWN:
		if ( animTimer <= 1300 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed <= 300 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_JUMPATTACK6:
		/*
		if (pm->ps)
		{
			if ( ( pm->ps->legsAnimTimer >= 1450
					&& BG_AnimLength( g_entities[ps->clientNum].client->clientInfo.animFileIndex, BOTH_JUMPATTACK6 ) - pm->ps->legsAnimTimer >= 400 )
				||(pm->ps->legsAnimTimer >= 400
					&& BG_AnimLength( g_entities[ps->clientNum].client->clientInfo.animFileIndex, BOTH_JUMPATTACK6 ) - pm->ps->legsAnimTimer >= 1100 ) )
			{//pretty much sideways
				return FORCE_LEVEL_3;
			}
		}
		*/
		if ( ( animTimer >= 1450
				&& animTimeElapsed >= 400 )
			||(animTimer >= 400
				&& animTimeElapsed >= 1100 ) )
		{//pretty much sideways
			return FORCE_LEVEL_3;
		}
		return FORCE_LEVEL_0;
		break;
	case BOTH_JUMPATTACK7:
		if ( animTimer <= 1200 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 200 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_SPINATTACK6:
		if ( animTimeElapsed <= 200 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_2;//FORCE_LEVEL_3;
		break;
	case BOTH_SPINATTACK7:
		if ( animTimer <= 500 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 500 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_2;//FORCE_LEVEL_3;
		break;
	case BOTH_FORCELONGLEAP_ATTACK:
		if ( animTimeElapsed <= 200 )
		{//1st four frames of anim
			return FORCE_LEVEL_3;
		}
		break;
	/*
	case BOTH_A7_KICK_F://these kicks attack, too
	case BOTH_A7_KICK_B:
	case BOTH_A7_KICK_R:
	case BOTH_A7_KICK_L:
		//FIXME: break up
		return FORCE_LEVEL_3;
		break;
	*/
	case BOTH_STABDOWN:
		if ( animTimer <= 900 )
		{//end of anim
			return FORCE_LEVEL_3;
		}
		break;
	case BOTH_STABDOWN_STAFF:
		if ( animTimer <= 850 )
		{//end of anim
			return FORCE_LEVEL_3;
		}
		break;
	case BOTH_STABDOWN_DUAL:
		if ( animTimer <= 900 )
		{//end of anim
			return FORCE_LEVEL_3;
		}
		break;
	case BOTH_A6_SABERPROTECT:
		if ( animTimer < 650 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 250 )
		{//start of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_A7_SOULCAL:
		if ( animTimer < 650 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 600 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_A1_SPECIAL:
		if ( animTimer < 600 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 200 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_A2_SPECIAL:
		if ( animTimer < 300 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 200 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_A3_SPECIAL:
		if ( animTimer < 700 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 200 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_FLIP_ATTACK7:
		return FORCE_LEVEL_3;
		break;
	case BOTH_PULL_IMPALE_STAB:
		if ( mySaberHit )
		{//someone else hit my saber, not asking for damage level, but defense strength
			return FORCE_LEVEL_1;
		}
		if ( animTimer < 1000 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_PULL_IMPALE_SWING:
		if ( animTimer < 500 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 650 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_ALORA_SPIN_SLASH:
		if ( animTimer < 900 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 250 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_A6_FB:
		if ( mySaberHit )
		{//someone else hit my saber, not asking for damage level, but defense strength
			return FORCE_LEVEL_1;
		}
		if ( animTimer < 250 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 250 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_A6_LR:
		if ( mySaberHit )
		{//someone else hit my saber, not asking for damage level, but defense strength
			return FORCE_LEVEL_1;
		}
		if ( animTimer < 250 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 250 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_3;
		break;
	case BOTH_A7_HILT:
		return FORCE_LEVEL_0;
		break;
//===SABERLOCK SUPERBREAKS START===========================================================================
	case BOTH_LK_S_DL_T_SB_1_W:
		if ( animTimer < 700 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_S_ST_S_SB_1_W:
		if ( animTimer < 300 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_S_DL_S_SB_1_W:
	case BOTH_LK_S_S_S_SB_1_W:
		if ( animTimer < 700 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 400 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_S_ST_T_SB_1_W:
	case BOTH_LK_S_S_T_SB_1_W:
		if ( animTimer < 150 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 400 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_DL_DL_T_SB_1_W:
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_DL_DL_S_SB_1_W:
	case BOTH_LK_DL_ST_S_SB_1_W:
		if ( animTimeElapsed < 1000 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_DL_ST_T_SB_1_W:
		if ( animTimer < 950 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 650 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_DL_S_S_SB_1_W:
		if ( saberNum != 0 )
		{//only right hand saber does damage in this suberbreak
			return FORCE_LEVEL_0;
		}
		if ( animTimer < 900 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 450 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_DL_S_T_SB_1_W:
		if ( saberNum != 0 )
		{//only right hand saber does damage in this suberbreak
			return FORCE_LEVEL_0;
		}
		if ( animTimer < 250 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 150 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_ST_DL_S_SB_1_W:
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_ST_DL_T_SB_1_W:
		//special suberbreak - doesn't kill, just kicks them backwards
		return FORCE_LEVEL_0;
		break;
	case BOTH_LK_ST_ST_S_SB_1_W:
	case BOTH_LK_ST_S_S_SB_1_W:
		if ( animTimer < 800 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 350 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		return FORCE_LEVEL_5;
		break;
	case BOTH_LK_ST_ST_T_SB_1_W:
	case BOTH_LK_ST_S_T_SB_1_W:
		return FORCE_LEVEL_5;
		break;
//===SABERLOCK SUPERBREAKS START===========================================================================
	case BOTH_HANG_ATTACK:
		//FIME: break up
		if ( animTimer < 1000 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else if ( animTimeElapsed < 250 )
		{//beginning of anim
			return FORCE_LEVEL_0;
		}
		else
		{//sweet spot
			return FORCE_LEVEL_5;
		}
		break;
	case BOTH_ROLL_STAB:
		if ( mySaberHit )
		{//someone else hit my saber, not asking for damage level, but defense strength
			return FORCE_LEVEL_1;
		}
		if ( animTimeElapsed > 400 )
		{//end of anim
			return FORCE_LEVEL_0;
		}
		else
		{
			return FORCE_LEVEL_3;
		}
		break;
	}
	return FORCE_LEVEL_0;
}

/* w_saber.c:13. The default half-extent for a saber entity's collision box. */
#define SABER_BOX_SIZE 16.0f
/* w_saber.c / q_shared.h mirrors used by SetSaberBoxSize. */
#define MAX_SABERS_O	2
#define MAX_BLADES_O	8
/* saberInfo_t::saberFlags2 bits the PC body branches on (q_shared.h). */
#define SFL2_ALWAYS_BLOCK	(1<<6)
#define SFL2_ALWAYS_BLOCK2	(1<<15)

/* w_saber.c:376 (PC source of truth). ORACLE DEVIATION: pass-the-read-fields precedent
   (cf. jka_G_SaberAttackPower / jka_G_PowerLevelForSaberAnim) — the C derefs deep
   gentity_t/gclient_t/saberInfo_t owner+blade state and the level/g_entities globals, so the
   oracle takes the read fields as scalars/arrays and writes the resulting r.mins / r.maxs. The
   two state predicates PM_SaberInBrokenParry(saberMove) / BG_SuperBreakLoseAnim(torsoAnim) are
   passed in as the single `inBrokenParryOrLose` flag (precedent: derived predicate results
   passed as scalars). The owner-presence guards and the #ifndef FINAL_BUILD Com_Printf are
   exercised/dropped on the Rust side (they have no effect on the computed box). dualSabers is
   derived from modelPresent[1] (PC: saber[1].model && saber[1].model[0]). The j/k indices are
   0 at the stale-storage check in the tested (forceBlock==0) cases, so blade00StorageTime is
   saber[0].blade[0].storageTime. modelPresent[j] mirrors saber[j].model[0] != 0. saberFlags2[]
   / bladeStyle2Start[] feed the new broken-parry alwaysBlock/forceBlock pass. Body/control-flow
   otherwise verbatim; vec3_t -> float[3], qboolean -> int. */
void jka_SetSaberBoxSize(
	float r_mins[3], float r_maxs[3],
	const float currentOrigin[3],
	int inBrokenParryOrLose,
	int levelTime, int lastSaberStorageTime, int blade00StorageTime,
	int saberHolstered,
	const int modelPresent[MAX_SABERS_O],
	const int numBlades[MAX_SABERS_O],
	const int saberFlags2[MAX_SABERS_O],
	const int bladeStyle2Start[MAX_SABERS_O],
	const float muzzlePoint[MAX_SABERS_O][MAX_BLADES_O][3],
	const float muzzleDir[MAX_SABERS_O][MAX_BLADES_O][3],
	const float lengthMax[MAX_SABERS_O][MAX_BLADES_O])
{
	float saberOrg[3], saberTip[3];
	int i;
	int j = 0;
	int k = 0;
	int dualSabers = 0;
	int alwaysBlock[MAX_SABERS_O][MAX_BLADES_O];
	int forceBlock = 0;

	if ( modelPresent[1] )
	{
		dualSabers = 1;
	}

	if ( inBrokenParryOrLose )
	{ /* let swings go right through when we're in this state */
		for ( i = 0; i < MAX_SABERS_O; i++ )
		{
			if ( i > 0 && !dualSabers )
			{ /* not using a second saber, set it to not blocking */
				for ( j = 0; j < MAX_BLADES_O; j++ )
				{
					alwaysBlock[i][j] = 0;
				}
			}
			else
			{
				if ( (saberFlags2[i]&SFL2_ALWAYS_BLOCK) )
				{
					for ( j = 0; j < numBlades[i]; j++ )
					{
						alwaysBlock[i][j] = 1;
						forceBlock = 1;
					}
				}
				if ( bladeStyle2Start[i] > 0 )
				{
					for ( j = bladeStyle2Start[i]; j < numBlades[i]; j++ )
					{
						if ( (saberFlags2[i]&SFL2_ALWAYS_BLOCK2) )
						{
							alwaysBlock[i][j] = 1;
							forceBlock = 1;
						}
						else
						{
							alwaysBlock[i][j] = 0;
						}
					}
				}
			}
		}
		if ( !forceBlock )
		{ /* no sabers/blades to FORCE to be on, so turn off blocking altogether */
			r_mins[0] = r_mins[1] = r_mins[2] = 0;
			r_maxs[0] = r_maxs[1] = r_maxs[2] = 0;
			return;
		}
	}

	if ((levelTime - lastSaberStorageTime) > 200 ||
		(levelTime - blade00StorageTime) > 100)
	{ /* it's been too long since we got a reliable point storage, so use the defaults and leave. */
		r_mins[0] = r_mins[1] = r_mins[2] = -SABER_BOX_SIZE;
		r_maxs[0] = r_maxs[1] = r_maxs[2] = SABER_BOX_SIZE;
		return;
	}

	if ( dualSabers
		|| numBlades[0] > 1 )
	{ /* dual sabers or multi-blade saber */
		if ( saberHolstered > 1 )
		{ /* entirely off - no blocking at all */
			r_mins[0] = r_mins[1] = r_mins[2] = 0;
			r_maxs[0] = r_maxs[1] = r_maxs[2] = 0;
			return;
		}
	}
	else
	{ /* single saber */
		if ( saberHolstered )
		{ /* off - no blocking at all */
			r_mins[0] = r_mins[1] = r_mins[2] = 0;
			r_maxs[0] = r_maxs[1] = r_maxs[2] = 0;
			return;
		}
	}

	/* Start out at the saber origin, then go through all the blades and push out the extents
	   for each blade, then set the box relative to the origin. */
	r_mins[0] = currentOrigin[0]; r_mins[1] = currentOrigin[1]; r_mins[2] = currentOrigin[2];
	r_maxs[0] = currentOrigin[0]; r_maxs[1] = currentOrigin[1]; r_maxs[2] = currentOrigin[2];

	for (i = 0; i < 3; i++)
	{
		for (j = 0; j < MAX_SABERS_O; j++)
		{
			if (!modelPresent[j])
			{
				break;
			}
			if ( dualSabers
				&& saberHolstered == 1
				&& j == 1 )
			{ /* this mother is holstered, get outta here. */
				j++;
				continue;
			}
			for (k = 0; k < numBlades[j]; k++)
			{
				if ( k > 0 )
				{ /* not the first blade */
					if ( !dualSabers )
					{ /* using a single saber */
						if ( numBlades[j] > 1 )
						{ /* with multiple blades */
							if ( saberHolstered == 1 )
							{ /* all blades after the first one are off */
								break;
							}
						}
					}
				}
				if ( forceBlock )
				{ /* only do blocking with blades that are marked to block */
					if ( !alwaysBlock[j][k] )
					{ /* this blade shouldn't be blocking */
						continue;
					}
				}
				saberOrg[0] = muzzlePoint[j][k][0];
				saberOrg[1] = muzzlePoint[j][k][1];
				saberOrg[2] = muzzlePoint[j][k][2];
				/* VectorMA(muzzlePoint, lengthMax, muzzleDir, saberTip): o[i]=v[i]+b[i]*s */
				saberTip[0] = muzzlePoint[j][k][0] + muzzleDir[j][k][0]*lengthMax[j][k];
				saberTip[1] = muzzlePoint[j][k][1] + muzzleDir[j][k][1]*lengthMax[j][k];
				saberTip[2] = muzzlePoint[j][k][2] + muzzleDir[j][k][2]*lengthMax[j][k];

				if (saberOrg[i] < r_mins[i])
				{
					r_mins[i] = saberOrg[i];
				}
				if (saberTip[i] < r_mins[i])
				{
					r_mins[i] = saberTip[i];
				}

				if (saberOrg[i] > r_maxs[i])
				{
					r_maxs[i] = saberOrg[i];
				}
				if (saberTip[i] > r_maxs[i])
				{
					r_maxs[i] = saberTip[i];
				}
			}
		}
	}

	r_mins[0] -= currentOrigin[0]; r_mins[1] -= currentOrigin[1]; r_mins[2] -= currentOrigin[2];
	r_maxs[0] -= currentOrigin[0]; r_maxs[1] -= currentOrigin[1]; r_maxs[2] -= currentOrigin[2];
}

/* -------- saber blade-collision hull (G_BuildSaberFaces / G_SaberFaceCollisionCheck) --------
   saberFace_t is w_saber.c:2309 (three vec3_t). The q_shared vector macros the bodies use are
   inlined locally here (the oracle TU has no q_shared.h). The jka_G_SabCol_* helpers above are
   reused as-is. ORACLE DEVIATIONs: vec3_t -> float[3]; the function-local `static`
   working-buffers become plain locals (each is written before read within the call, so the
   result is identical); G_BuildSaberFaces writes into a caller-provided face buffer and returns
   fNum (instead of the *fNum/**fList out-params over its file-static) so the oracle is pure. */
typedef struct { float v1[3]; float v2[3]; float v3[3]; } jka_saberFace_t;

#define ORA_VCopy(a,b)      ((b)[0]=(a)[0],(b)[1]=(a)[1],(b)[2]=(a)[2])
#define ORA_VInverse(v)     ((v)[0]=-(v)[0],(v)[1]=-(v)[1],(v)[2]=-(v)[2])
#define ORA_VMA(v,s,b,o)    ((o)[0]=(v)[0]+(b)[0]*(s),(o)[1]=(v)[1]+(b)[1]*(s),(o)[2]=(v)[2]+(b)[2]*(s))
#define ORA_VSet(v,x,y,z)   ((v)[0]=(x),(v)[1]=(y),(v)[2]=(z))
#define ORA_VSub(a,b,c)     ((c)[0]=(a)[0]-(b)[0],(c)[1]=(a)[1]-(b)[1],(c)[2]=(a)[2]-(b)[2])
#define ORA_VAdd(a,b,c)     ((c)[0]=(a)[0]+(b)[0],(c)[1]=(a)[1]+(b)[1],(c)[2]=(a)[2]+(b)[2])
static int ORA_VCompare( const float a[3], const float b[3] ) {
	if (a[0]!=b[0] || a[1]!=b[1] || a[2]!=b[2]) return 0;
	return 1;
}
static const float ora_vec3_origin[3] = {0,0,0};

/* w_saber.c:2317. ORACLE DEVIATION: writes into caller buffer `faces` (>=12 entries) and
   returns fNum; vec3_t -> float[3]. Body otherwise verbatim. */
int jka_G_BuildSaberFaces( const float base[3], const float tip[3], float radius,
                           const float fwd[3], const float right[3], jka_saberFace_t *faces ) {
	int i = 0;
	const float *d1 = 0, *d2 = 0;
	float invFwd[3];
	float invRight[3];

	ORA_VCopy(fwd, invFwd);
	ORA_VInverse(invFwd);
	ORA_VCopy(right, invRight);
	ORA_VInverse(invRight);

	while (i < 8)
	{
		if (i < 2)        { d1 = &fwd[0];   d2 = &invRight[0]; }
		else if (i < 4)   { d1 = &fwd[0];   d2 = &right[0];    }
		else if (i < 6)   { d1 = &right[0]; d2 = &fwd[0];      }
		else if (i < 8)   { d1 = &right[0]; d2 = &invFwd[0];   }

		ORA_VMA(base, radius/2.0f, d1, faces[i].v1);
		ORA_VMA(faces[i].v1, radius/2.0f, d2, faces[i].v1);
		ORA_VMA(tip, radius/2.0f, d1, faces[i].v2);
		ORA_VMA(faces[i].v2, radius/2.0f, d2, faces[i].v2);
		ORA_VMA(tip, -radius/2.0f, d1, faces[i].v3);
		ORA_VMA(faces[i].v3, radius/2.0f, d2, faces[i].v3);
		i++;

		ORA_VMA(tip, -radius/2.0f, d1, faces[i].v1);
		ORA_VMA(faces[i].v1, radius/2.0f, d2, faces[i].v1);
		ORA_VMA(base, radius/2.0f, d1, faces[i].v2);
		ORA_VMA(faces[i].v2, radius/2.0f, d2, faces[i].v2);
		ORA_VMA(base, -radius/2.0f, d1, faces[i].v3);
		ORA_VMA(faces[i].v3, radius/2.0f, d2, faces[i].v3);
		i++;
	}

	/* top surface, face 1 */
	ORA_VMA(tip, radius/2.0f, fwd, faces[i].v1);
	ORA_VMA(faces[i].v1, -radius/2.0f, right, faces[i].v1);
	ORA_VMA(tip, radius/2.0f, fwd, faces[i].v2);
	ORA_VMA(faces[i].v2, radius/2.0f, right, faces[i].v2);
	ORA_VMA(tip, -radius/2.0f, fwd, faces[i].v3);
	ORA_VMA(faces[i].v3, -radius/2.0f, right, faces[i].v3);
	i++;
	/* face 2 */
	ORA_VMA(tip, radius/2.0f, fwd, faces[i].v1);
	ORA_VMA(faces[i].v1, radius/2.0f, right, faces[i].v1);
	ORA_VMA(tip, -radius/2.0f, fwd, faces[i].v2);
	ORA_VMA(faces[i].v2, -radius/2.0f, right, faces[i].v2);
	ORA_VMA(tip, -radius/2.0f, fwd, faces[i].v3);
	ORA_VMA(faces[i].v3, radius/2.0f, right, faces[i].v3);
	i++;

	/* bottom surface, face 1 */
	ORA_VMA(base, radius/2.0f, fwd, faces[i].v1);
	ORA_VMA(faces[i].v1, -radius/2.0f, right, faces[i].v1);
	ORA_VMA(base, radius/2.0f, fwd, faces[i].v2);
	ORA_VMA(faces[i].v2, radius/2.0f, right, faces[i].v2);
	ORA_VMA(base, -radius/2.0f, fwd, faces[i].v3);
	ORA_VMA(faces[i].v3, -radius/2.0f, right, faces[i].v3);
	i++;
	/* face 2 */
	ORA_VMA(base, radius/2.0f, fwd, faces[i].v1);
	ORA_VMA(faces[i].v1, radius/2.0f, right, faces[i].v1);
	ORA_VMA(base, -radius/2.0f, fwd, faces[i].v2);
	ORA_VMA(faces[i].v2, -radius/2.0f, right, faces[i].v2);
	ORA_VMA(base, -radius/2.0f, fwd, faces[i].v3);
	ORA_VMA(faces[i].v3, radius/2.0f, right, faces[i].v3);
	i++;

	return i;
}

/* w_saber.c:2462. ORACLE DEVIATION: vec3_t -> float[3]; the `static` working-buffers become
   plain locals; reuses jka_G_SabCol_* above. qboolean -> int. Body otherwise verbatim. */
int jka_G_SaberFaceCollisionCheck( int fNum, jka_saberFace_t *fList, const float atkStart[3],
                                   const float atkEnd[3], float atkMins[3], float atkMaxs[3],
                                   float impactPoint[3] ) {
	float planeEq[4];
	float side, side2, dist;
	float dir[3];
	float point[3];
	int i = 0;

	if (ORA_VCompare(atkMins, ora_vec3_origin) && ORA_VCompare(atkMaxs, ora_vec3_origin))
	{
		ORA_VSet(atkMins, -1.0f, -1.0f, -1.0f);
		ORA_VSet(atkMaxs, 1.0f, 1.0f, 1.0f);
	}

	ORA_VSub(atkEnd, atkStart, dir);

	while (i < fNum)
	{
		jka_G_SabCol_CalcPlaneEq(fList->v1, fList->v2, fList->v3, planeEq);

		if (jka_G_SabCol_PointRelativeToPlane(atkStart, &side, planeEq) !=
			jka_G_SabCol_PointRelativeToPlane(atkEnd, &side2, planeEq))
		{
			float extruded[3];
			float minPoint[3], maxPoint[3];
			float planeNormal[3];
			int facing;

			ORA_VCopy(&planeEq[0], planeNormal);
			side2 = planeNormal[0]*dir[0] + planeNormal[1]*dir[1] + planeNormal[2]*dir[2];

			dist = side/side2;
			ORA_VMA(atkStart, -dist, dir, point);

			ORA_VAdd(point, atkMins, minPoint);
			ORA_VAdd(point, atkMaxs, maxPoint);

			ORA_VMA(fList->v1, -2.0f, planeNormal, extruded);
			jka_G_SabCol_CalcPlaneEq(fList->v1, fList->v2, extruded, planeEq);
			facing = jka_G_SabCol_PointRelativeToPlane(point, &side, planeEq);

			if (facing < 0)
			{
				facing = jka_G_SabCol_PointRelativeToPlane(minPoint, &side, planeEq);
				if (facing < 0)
					facing = jka_G_SabCol_PointRelativeToPlane(maxPoint, &side, planeEq);
			}

			if (facing >= 0)
			{
				ORA_VMA(fList->v2, -2.0f, planeNormal, extruded);
				jka_G_SabCol_CalcPlaneEq(fList->v2, fList->v3, extruded, planeEq);
				facing = jka_G_SabCol_PointRelativeToPlane(point, &side, planeEq);

				if (facing < 0)
				{
					facing = jka_G_SabCol_PointRelativeToPlane(minPoint, &side, planeEq);
					if (facing < 0)
						facing = jka_G_SabCol_PointRelativeToPlane(maxPoint, &side, planeEq);
				}

				if (facing >= 0)
				{
					ORA_VMA(fList->v3, -2.0f, planeNormal, extruded);
					jka_G_SabCol_CalcPlaneEq(fList->v3, fList->v1, extruded, planeEq);
					facing = jka_G_SabCol_PointRelativeToPlane(point, &side, planeEq);

					if (facing < 0)
					{
						facing = jka_G_SabCol_PointRelativeToPlane(minPoint, &side, planeEq);
						if (facing < 0)
							facing = jka_G_SabCol_PointRelativeToPlane(maxPoint, &side, planeEq);
					}

					if (facing >= 0)
					{
						ORA_VCopy(point, impactPoint);
						return 1;
					}
				}
			}
		}

		i++;
		fList++;
	}

	return 0;
}
