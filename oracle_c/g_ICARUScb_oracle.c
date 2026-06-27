/* Extracted g_ICARUScb.c functions, compiled as a parity oracle. See the header in
   q_math_oracle.c for the method. Function *bodies* are the authentic Raven source
   from refs/raven-jediacademy/codemp/game/g_ICARUScb.c, verbatim.

   ORACLE DEVIATIONS:
   - Q3_TaskIDClear (g_ICARUScb.c:269) is wholly self-contained — its entire body is
     `*taskID = -1;` — so it is reproduced verbatim with no host context. Renamed with a
     `jka_` prefix so it cannot collide with any host symbol. */

void jka_Q3_TaskIDClear(int *taskID)
{
	*taskID = -1;
}
