// Oracle TU for bg_saberLoad.c — verbatim copies of the original Raven C, compiled
// independently of the Rust port so tests can assert bit-exact parity. The real
// bg_saberLoad.c cannot be #include'd (it drags in the clang-hostile engine tree
// and the C++ ClientManager), so the small self-contained helpers are transcribed
// verbatim here, against verbatim transcriptions of their by-value dependency types
// (the q_shared.h saber enums/structs — whose layout is verified separately in
// q_shared_h_oracle.c). Q_stricmp / Q_irand are declared and resolve to
// q_shared_oracle.c's definitions at link time.

typedef int qboolean;
#define qfalse 0
#define qtrue  1

// --- q_shared.h saber enums (verbatim) -------------------------------------
typedef enum
{
	SABER_RED,
	SABER_ORANGE,
	SABER_YELLOW,
	SABER_GREEN,
	SABER_BLUE,
	SABER_PURPLE,
	NUM_SABER_COLORS
} saber_colors_t;

typedef enum
{
	SS_NONE = 0,
	SS_FAST,
	SS_MEDIUM,
	SS_STRONG,
	SS_DESANN,
	SS_TAVION,
	SS_DUAL,	//both blades, both hands
	SS_STAFF,	//staff style
	SS_NUM_SABER_STYLES
} saber_styles_t;

// link-resolved against q_shared_oracle.c
extern int Q_stricmp( const char *s1, const char *s2 );
extern int Q_irand( int value1, int value2 );
extern char *COM_ParseExt( const char **data_p, qboolean allowLineBreaks );
extern void Com_Printf( const char *msg, ... );

// forward decl (defined below) — WP_SaberSetColor is transcribed above its definition
saber_colors_t TranslateSaberColor( const char *name );

// --- BG_ParseLiteral (bg_saberLoad.c:71) — verbatim --------------------------
qboolean BG_ParseLiteral( const char **data, const char *string )
{
	const char	*token;

	token = COM_ParseExt( data, qtrue );
	if ( token[0] == 0 )
	{
		Com_Printf( "unexpected EOF\n" );
		return qtrue;
	}

	if ( Q_stricmp( token, string ) )
	{
		Com_Printf( "required string '%s' missing\n", string );
		return qtrue;
	}

	return qfalse;
}

// --- q_shared.h saber structs (verbatim) — for the BG_SI_* / BG_BLADE_* accessors.
// The layout is verified identical to the Rust port in q_shared_h_oracle.c, so the
// tests pass a Rust *mut saberInfo_t straight into these C bodies.
typedef float vec3_t[3];
typedef int qhandle_t;
#define MAX_QPATH 64

typedef enum
{
	SABER_NONE = 0,
	SABER_SINGLE,
	SABER_STAFF,
	SABER_DAGGER,
	SABER_BROAD,
	SABER_PRONG,
	SABER_ARC,
	SABER_SAI,
	SABER_CLAW,
	SABER_LANCE,
	SABER_STAR,
	SABER_TRIDENT,
	SABER_SITH_SWORD,
	NUM_SABERS
} saberType_t;

typedef struct
{
	// Actual trail stuff
	int		inAction;	// controls whether should we even consider starting one
	int		duration;	// how long each trail seg stays in existence
	int		lastTime;	// time a saber segement was last stored
	vec3_t	base;
	vec3_t	tip;

	vec3_t	dualbase;
	vec3_t	dualtip;

	// Marks stuff
	qboolean	haveOldPos[2];
	vec3_t		oldPos[2];
	vec3_t		oldNormal[2];
} saberTrail_t;

typedef struct
{
	qboolean	active;
	saber_colors_t	color;
	float		radius;
	float		length;
	float		lengthMax;
	float		lengthOld;
	float		desiredLength;
	vec3_t		muzzlePoint;
	vec3_t		muzzlePointOld;
	vec3_t		muzzleDir;
	vec3_t		muzzleDirOld;
	saberTrail_t	trail;
	int			hitWallDebounceTime;
	int			storageTime;
	int			extendDebounce;
} bladeInfo_t;
#define MAX_BLADES 8

typedef struct
{
	char		name[64];
	char		fullName[64];
	saberType_t	type;
	char		model[MAX_QPATH];
	qhandle_t	skin;
	int			soundOn;
	int			soundLoop;
	int			soundOff;
	int			numBlades;
	bladeInfo_t	blade[MAX_BLADES];
	int			stylesLearned;
	int			stylesForbidden;
	int			maxChain;
	int			forceRestrictions;
	int			lockBonus;
	int			parryBonus;
	int			breakParryBonus;
	int			breakParryBonus2;
	int			disarmBonus;
	int			disarmBonus2;
	saber_styles_t	singleBladeStyle;
	int			saberFlags;
	int			saberFlags2;
	qhandle_t	spinSound;
	qhandle_t	swingSound[3];
	float		moveSpeedScale;
	float		animSpeedScale;
	int	kataMove;
	int	lungeAtkMove;
	int	jumpAtkUpMove;
	int	jumpAtkFwdMove;
	int	jumpAtkBackMove;
	int	jumpAtkRightMove;
	int	jumpAtkLeftMove;
	int	readyAnim;
	int	drawAnim;
	int	putawayAnim;
	int	tauntAnim;
	int	bowAnim;
	int	meditateAnim;
	int	flourishAnim;
	int	gloatAnim;
	int			bladeStyle2Start;
	int			trailStyle;
	int			g2MarksShader;
	int			g2WeaponMarkShader;
	qhandle_t	hitSound[3];
	qhandle_t	blockSound[3];
	qhandle_t	bounceSound[3];
	int			blockEffect;
	int			hitPersonEffect;
	int			hitOtherEffect;
	int			bladeEffect;
	float		knockbackScale;
	float		damageScale;
	float		splashRadius;
	int			splashDamage;
	float		splashKnockback;
	int			trailStyle2;
	int			g2MarksShader2;
	int			g2WeaponMarkShader2;
	qhandle_t	hit2Sound[3];
	qhandle_t	block2Sound[3];
	qhandle_t	bounce2Sound[3];
	int			blockEffect2;
	int			hitPersonEffect2;
	int			hitOtherEffect2;
	int			bladeEffect2;
	float		knockbackScale2;
	float		damageScale2;
	float		splashRadius2;
	int			splashDamage2;
	float		splashKnockback2;
} saberInfo_t;
#define MAX_SABERS 2

// --- BG_BLADE_* / BG_SI_* accessors (bg_saberLoad.c:2803-2900) — verbatim. PC
// `bladeInfo_t::trail` is a single (unindexed) `saberTrail_t`.
void BG_BLADE_ActivateTrail ( bladeInfo_t *blade, float duration )
{
	blade->trail.inAction = qtrue;
	blade->trail.duration = duration;
}

void BG_BLADE_DeactivateTrail ( bladeInfo_t *blade, float duration )
{
	blade->trail.inAction = qfalse;
	blade->trail.duration = duration;
}

void BG_SI_Activate( saberInfo_t *saber )
{
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		saber->blade[i].active = qtrue;
	}
}

void BG_SI_Deactivate( saberInfo_t *saber )
{
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		saber->blade[i].active = qfalse;
	}
}

void BG_SI_BladeActivate( saberInfo_t *saber, int iBlade, qboolean bActive )
{
	// Validate blade ID/Index.
	if ( iBlade < 0 || iBlade >= saber->numBlades )
		return;

	saber->blade[iBlade].active = bActive;
}

qboolean BG_SI_Active(saberInfo_t *saber)
{
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		if ( saber->blade[i].active )
		{
			return qtrue;
		}
	}
	return qfalse;
}

void BG_SI_SetLength( saberInfo_t *saber, float length )
{
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		saber->blade[i].length = length;
	}
}

void BG_SI_SetDesiredLength(saberInfo_t *saber, float len, int bladeNum )
{
	int i, startBlade = 0, maxBlades = saber->numBlades;

	if ( bladeNum >= 0 && bladeNum < saber->numBlades)
	{//doing this on a specific blade
		startBlade = bladeNum;
		maxBlades = bladeNum+1;
	}
	for (i = startBlade; i < maxBlades; i++)
	{
		saber->blade[i].desiredLength = len;
	}
}

void BG_SI_SetLengthGradual(saberInfo_t *saber, int time)
{
	int i;
	float amt, dLen;

	for (i = 0; i < saber->numBlades; i++)
	{
		dLen = saber->blade[i].desiredLength;

		if (dLen == -1)
		{ //assume we want max blade len
			dLen = saber->blade[i].lengthMax;
		}

		if (saber->blade[i].length == dLen)
		{
			continue;
		}

		if (saber->blade[i].length == saber->blade[i].lengthMax ||
			saber->blade[i].length == 0)
		{
			saber->blade[i].extendDebounce = time;
			if (saber->blade[i].length == 0)
			{
				saber->blade[i].length++;
			}
			else
			{
				saber->blade[i].length--;
			}
		}

		amt = (time - saber->blade[i].extendDebounce)*0.01;

		if (amt < 0.2f)
		{
			amt = 0.2f;
		}

		if (saber->blade[i].length < dLen)
		{
			saber->blade[i].length += amt;

			if (saber->blade[i].length > dLen)
			{
				saber->blade[i].length = dLen;
			}
			if (saber->blade[i].length > saber->blade[i].lengthMax)
			{
				saber->blade[i].length = saber->blade[i].lengthMax;
			}
		}
		else if (saber->blade[i].length > dLen)
		{
			saber->blade[i].length -= amt;

			if (saber->blade[i].length < dLen)
			{
				saber->blade[i].length = dLen;
			}
			if (saber->blade[i].length < 0)
			{
				saber->blade[i].length = 0;
			}
		}
	}
}

float BG_SI_Length(saberInfo_t *saber)
{//return largest length
	int len1 = 0;
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		if ( saber->blade[i].length > len1 )
		{
			len1 = saber->blade[i].length;
		}
	}
	return len1;
}

float BG_SI_LengthMax(saberInfo_t *saber)
{
	int len1 = 0;
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		if ( saber->blade[i].lengthMax > len1 )
		{
			len1 = saber->blade[i].lengthMax;
		}
	}
	return len1;
}

void BG_SI_ActivateTrail ( saberInfo_t *saber, float duration )
{
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		BG_BLADE_ActivateTrail(&saber->blade[i], duration);
	}
}

void BG_SI_DeactivateTrail ( saberInfo_t *saber, float duration )
{
	int i;

	for ( i = 0; i < saber->numBlades; i++ )
	{
		BG_BLADE_DeactivateTrail(&saber->blade[i], duration);
	}
}

// WP_SaberSetColor (bg_saberLoad.c:1213) — verbatim.
void WP_SaberSetColor( saberInfo_t *sabers, int saberNum, int bladeNum, char *colorName )
{
	if ( !sabers )
	{
		return;
	}
	sabers[saberNum].blade[bladeNum].color = TranslateSaberColor( colorName );
}

// --- verbatim bodies from bg_saberLoad.c -----------------------------------
saber_colors_t TranslateSaberColor( const char *name )
{
	if ( !Q_stricmp( name, "red" ) )
	{
		return SABER_RED;
	}
	if ( !Q_stricmp( name, "orange" ) )
	{
		return SABER_ORANGE;
	}
	if ( !Q_stricmp( name, "yellow" ) )
	{
		return SABER_YELLOW;
	}
	if ( !Q_stricmp( name, "green" ) )
	{
		return SABER_GREEN;
	}
	if ( !Q_stricmp( name, "blue" ) )
	{
		return SABER_BLUE;
	}
	if ( !Q_stricmp( name, "purple" ) )
	{
		return SABER_PURPLE;
	}
	if ( !Q_stricmp( name, "random" ) )
	{
		return ((saber_colors_t)(Q_irand( SABER_ORANGE, SABER_PURPLE )));
	}
	return SABER_BLUE;
}

saber_styles_t TranslateSaberStyle( const char *name )
{
	if ( !Q_stricmp( name, "fast" ) )
	{
		return SS_FAST;
	}
	if ( !Q_stricmp( name, "medium" ) )
	{
		return SS_MEDIUM;
	}
	if ( !Q_stricmp( name, "strong" ) )
	{
		return SS_STRONG;
	}
	if ( !Q_stricmp( name, "desann" ) )
	{
		return SS_DESANN;
	}
	if ( !Q_stricmp( name, "tavion" ) )
	{
		return SS_TAVION;
	}
	if ( !Q_stricmp( name, "dual" ) )
	{
		return SS_DUAL;
	}
	if ( !Q_stricmp( name, "staff" ) )
	{
		return SS_STAFF;
	}
	return SS_NONE;
}

// --- q_shared.h saberFlags2 bits used by the blade-style predicates (verbatim) ---
#define SFL2_NO_MANUAL_DEACTIVATE	(1<<7)
#define SFL2_TRANSITION_DAMAGE		(1<<8)
#define SFL2_NO_MANUAL_DEACTIVATE2	(1<<16)
#define SFL2_TRANSITION_DAMAGE2		(1<<17)

// --- saber blade-style / valid-style predicates (bg_saberLoad.c) — verbatim ------
qboolean WP_SaberBladeUseSecondBladeStyle( saberInfo_t *saber, int bladeNum )
{
	if ( saber )
	{
		if ( saber->bladeStyle2Start > 0 )
		{
			if ( bladeNum >= saber->bladeStyle2Start )
			{
				return qtrue;
			}
		}
	}
	return qfalse;
}

qboolean WP_SaberBladeDoTransitionDamage( saberInfo_t *saber, int bladeNum )
{
	if ( !WP_SaberBladeUseSecondBladeStyle( saber, bladeNum )
		&& (saber->saberFlags2&SFL2_TRANSITION_DAMAGE) )
	{//use first blade style for this blade
		return qtrue;
	}
	else if ( WP_SaberBladeUseSecondBladeStyle( saber, bladeNum )
		&& (saber->saberFlags2&SFL2_TRANSITION_DAMAGE2) )
	{//use second blade style for this blade
		return qtrue;
	}
	return qfalse;
}

qboolean WP_UseFirstValidSaberStyle( saberInfo_t *saber1, saberInfo_t *saber2, int saberHolstered, int *saberAnimLevel )
{
	qboolean styleInvalid = qfalse;
	qboolean saber1Active;
	qboolean saber2Active;
	qboolean dualSabers = qfalse;
	int	validStyles = 0, styleNum;

	if ( saber2 && saber2->model && saber2->model[0] )
	{
		dualSabers = qtrue;
	}

	if ( dualSabers )
	{//dual
		if ( saberHolstered > 1 )
		{
			saber1Active = saber2Active = qfalse;
		}
		else if ( saberHolstered > 0 )
		{
			saber1Active = qtrue;
			saber2Active = qfalse;
		}
		else
		{
			saber1Active = saber2Active = qtrue;
		}
	}
	else
	{
		saber2Active = qfalse;
		if ( !saber1
			|| !saber1->model
			|| !saber1->model[0] )
		{
			saber1Active = qfalse;
		}
		else if ( saber1->numBlades > 1 )
		{//staff
			if ( saberHolstered > 1 )
			{
				saber1Active = qfalse;
			}
			else
			{
				saber1Active = qtrue;
			}
		}
		else
		{//single
			if ( saberHolstered )
			{
				saber1Active = qfalse;
			}
			else
			{
				saber1Active = qtrue;
			}
		}
	}

	//initially, all styles are valid
	for ( styleNum = SS_NONE+1; styleNum < SS_NUM_SABER_STYLES; styleNum++ )
	{
		validStyles |= (1<<styleNum);
	}

	if ( saber1Active
		&& saber1
		&& saber1->model
		&& saber1->model[0]
		&& saber1->stylesForbidden )
	{
		if ( (saber1->stylesForbidden&(1<<*saberAnimLevel)) )
		{//not a valid style for first saber!
			styleInvalid = qtrue;
			validStyles &= ~saber1->stylesForbidden;
		}
	}
	if ( dualSabers )
	{//check second saber, too
		if ( saber2Active
			&& saber2->stylesForbidden )
		{
			if ( (saber2->stylesForbidden&(1<<*saberAnimLevel)) )
			{//not a valid style for second saber!
				styleInvalid = qtrue;
				//only the ones both sabers allow is valid
				validStyles &= ~saber2->stylesForbidden;
			}
		}
	}
	if ( styleInvalid && validStyles )
	{//using an invalid style and have at least one valid style to use, so switch to it
		int styleNum;
		for ( styleNum = SS_FAST; styleNum < SS_NUM_SABER_STYLES; styleNum++ )
		{
			if ( (validStyles&(1<<styleNum)) )
			{
				*saberAnimLevel = styleNum;
				return qtrue;
			}
		}
	}
	return qfalse;
}

qboolean WP_SaberStyleValidForSaber( saberInfo_t *saber1, saberInfo_t *saber2, int saberHolstered, int saberAnimLevel )
{
	qboolean saber1Active;
	qboolean saber2Active;
	qboolean dualSabers = qfalse;

	if ( saber2 && saber2->model && saber2->model[0] )
	{
		dualSabers = qtrue;
	}

	if ( dualSabers )
	{//dual
		if ( saberHolstered > 1 )
		{
			saber1Active = saber2Active = qfalse;
		}
		else if ( saberHolstered > 0 )
		{
			saber1Active = qtrue;
			saber2Active = qfalse;
		}
		else
		{
			saber1Active = saber2Active = qtrue;
		}
	}
	else
	{
		saber2Active = qfalse;
		if ( !saber1
			|| !saber1->model
			|| !saber1->model[0] )
		{
			saber1Active = qfalse;
		}
		else if ( saber1->numBlades > 1 )
		{//staff
			if ( saberHolstered > 1 )
			{
				saber1Active = qfalse;
			}
			else
			{
				saber1Active = qtrue;
			}
		}
		else
		{//single
			if ( saberHolstered )
			{
				saber1Active = qfalse;
			}
			else
			{
				saber1Active = qtrue;
			}
		}
	}

	if ( saber1Active
		&& saber1
		&& saber1->model
		&& saber1->model[0]
		&& saber1->stylesForbidden )
	{
		if ( (saber1->stylesForbidden&(1<<saberAnimLevel)) )
		{//not a valid style for first saber!
			return qfalse;
		}
	}
	if ( dualSabers
		&& saber2Active
		&& saber2
		&& saber2->model
		&& saber2->model[0] )
	{
		if ( saber2->stylesForbidden )
		{//check second saber, too
			if ( (saber2->stylesForbidden&(1<<saberAnimLevel)) )
			{//not a valid style for second saber!
				return qfalse;
			}
		}
		//now: if using dual sabers, only dual and tavion (if given with this saber) are allowed
		if ( saberAnimLevel != SS_DUAL )
		{//dual is okay
			if ( saberAnimLevel != SS_TAVION )
			{//tavion might be okay, all others are not
				return qfalse;
			}
			else
			{//see if "tavion" style is okay
				if ( saber1Active
					&& saber1
					&& saber1->model
					&& saber1->model[0]
					&& (saber1->stylesLearned&(1<<SS_TAVION)) )
				{//okay to use tavion style, first saber gave it to us
				}
				else if ( (saber2->stylesLearned&(1<<SS_TAVION)) )
				{//okay to use tavion style, second saber gave it to us
				}
				else
				{//tavion style is not allowed because neither of the sabers we're using gave it to us (I know, doesn't quite make sense, but...)
					return qfalse;
				}
			}
		}
	}
	return qtrue;
}

qboolean WP_SaberCanTurnOffSomeBlades( saberInfo_t *saber )
{
	if ( saber->bladeStyle2Start > 0
		&& saber->numBlades > saber->bladeStyle2Start )
	{
		if ( (saber->saberFlags2&SFL2_NO_MANUAL_DEACTIVATE)
			&& (saber->saberFlags2&SFL2_NO_MANUAL_DEACTIVATE2) )
		{//all blades are always on
			return qfalse;
		}
	}
	else
	{
		if ( (saber->saberFlags2&SFL2_NO_MANUAL_DEACTIVATE) )
		{//all blades are always on
			return qfalse;
		}
	}
	//you can turn some off
	return qtrue;
}
