/*
 * Logic oracle for PredictedAngularDecrement (FighterNPC.c:234) ported into
 * src/codemp/game/fighternpc.rs.
 *
 * The real FighterNPC.c cannot be #include'd (its quoted includes drag in the whole
 * clang-hostile reference tree, and the function is a file-local `static`), so the body
 * is transcribed here VERBATIM. It is pure scalar float math — three floats in, one
 * float out — so it parity-tests cleanly with no struct marshalling. Built only under
 * the `oracle` feature.
 */

/* verbatim body of FighterNPC.c:234 (the file-static helper, with `static` dropped so
 * it links) */
float jka_PredictedAngularDecrement(float scale, float timeMod, float originalAngle)
{
	float fixedBaseDec = originalAngle*0.05f;
	float r = 0.0f;

	if (fixedBaseDec < 0.0f)
	{
		fixedBaseDec = -fixedBaseDec;
	}

	fixedBaseDec *= (1.0f+(1.0f-scale));

	if (fixedBaseDec < 0.1f)
	{ //don't increment in incredibly small fractions, it would eat up unnecessary bandwidth.
		fixedBaseDec = 0.1f;
	}

	fixedBaseDec *= (timeMod*0.1f);
	if (originalAngle > 0.0f)
	{ //subtract
		r = (originalAngle-fixedBaseDec);
		if (r < 0.0f)
		{
			r = 0.0f;
		}
	}
	else if (originalAngle < 0.0f)
	{ //add
		r = (originalAngle+fixedBaseDec);
		if (r > 0.0f)
		{
			r = 0.0f;
		}
	}

	return r;
}
