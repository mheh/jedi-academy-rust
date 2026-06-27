/* Oracle extraction of g_nav.c's pure navgoal-reached test:
 *   NAV_HitNavGoal (refs/raven-jediacademy/codemp/game/g_nav.c:167)
 *
 * Self-contained vector math — its only callee, G_BoundsOverlap (NPC_goal.c:94),
 * is inlined here (renamed jka_) so this object is dependency-free. `vec3_t` is
 * `float[3]`, decaying to `float*`. Renamed `jka_` to avoid colliding with the
 * test binary / other oracle objects. */

#include <math.h>

typedef float vec_t;
typedef vec_t vec3_t[3];
typedef int qboolean;
#define qfalse 0
#define qtrue 1

#define NAVGOAL_USE_RADIUS 16384

#define VectorSubtract(a,b,c) ((c)[0]=(a)[0]-(b)[0],(c)[1]=(a)[1]-(b)[1],(c)[2]=(a)[2]-(b)[2])
#define VectorAdd(a,b,c)      ((c)[0]=(a)[0]+(b)[0],(c)[1]=(a)[1]+(b)[1],(c)[2]=(a)[2]+(b)[2])
#define VectorSet(v,x,y,z)    ((v)[0]=(x),(v)[1]=(y),(v)[2]=(z))
#define VectorLengthSquared(v) ((v)[0]*(v)[0]+(v)[1]*(v)[1]+(v)[2]*(v)[2])
#define DistanceSquared(p1,p2) (((p1)[0]-(p2)[0])*((p1)[0]-(p2)[0])+((p1)[1]-(p2)[1])*((p1)[1]-(p2)[1])+((p1)[2]-(p2)[2])*((p1)[2]-(p2)[2]))

static qboolean jka_local_G_BoundsOverlap(const vec3_t mins1, const vec3_t maxs1, const vec3_t mins2, const vec3_t maxs2)
{//NOTE: flush up against counts as overlapping
	if(mins1[0]>maxs2[0])
		return qfalse;
	if(mins1[1]>maxs2[1])
		return qfalse;
	if(mins1[2]>maxs2[2])
		return qfalse;
	if(maxs1[0]<mins2[0])
		return qfalse;
	if(maxs1[1]<mins2[1])
		return qfalse;
	if(maxs1[2]<mins2[2])
		return qfalse;
	return qtrue;
}

qboolean jka_NAV_HitNavGoal( vec3_t point, vec3_t mins, vec3_t maxs, vec3_t dest, int radius, qboolean flying )
{
	vec3_t	dmins, dmaxs, pmins, pmaxs;

	if ( radius & NAVGOAL_USE_RADIUS )
	{
		radius &= ~NAVGOAL_USE_RADIUS;
		if ( !flying )
		{//Allow for a little z difference
			vec3_t	diff;
			VectorSubtract( point, dest, diff );
			if ( fabs(diff[2]) <= 24 )
			{
				diff[2] = 0;
			}
			return ( VectorLengthSquared( diff ) <= (radius*radius) );
		}
		else
		{//must hit exactly
			return ( DistanceSquared(dest, point) <= (radius*radius) );
		}
	}
	else
	{
		//Construct a dummy bounding box from our radius value
		VectorSet( dmins, -radius, -radius, -radius );
		VectorSet( dmaxs,  radius,  radius,  radius );

		//Translate it
		VectorAdd( dmins, dest, dmins );
		VectorAdd( dmaxs, dest, dmaxs );

		//Translate the starting box
		VectorAdd( point, mins, pmins );
		VectorAdd( point, maxs, pmaxs );

		//See if they overlap
		return jka_local_G_BoundsOverlap( pmins, pmaxs, dmins, dmaxs );
	}
}
