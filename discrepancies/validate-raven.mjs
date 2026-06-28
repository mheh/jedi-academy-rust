export const meta = {
  name: 'validate-raven-source',
  description: 'Three-way compare each Rust port vs PC oracle vs Xbox grayj; Sonnet agents write per-file discrepancy reports',
  phases: [{ title: 'Compare', detail: 'one Sonnet agent per file writes discrepancies/<name>.md' }],
}

const pairs = typeof args === 'string' ? JSON.parse(args) : args
log(`Three-way comparing ${pairs.length} files with Sonnet agents`)

phase('Compare')

const prompt = (p) => `You are auditing an AI-assisted C++ -> Rust port of Star Wars Jedi Academy, to find leftover **Xbox-derived** code.

CONTEXT: The port originally started from the grayj **Xbox** codebase, then mid-way switched to target Raven's **PC** source. The PC source (\`oracle/\`) is the SOURCE OF TRUTH — the port should match it. The risk: some Rust function bodies still reflect the OLD Xbox logic instead of the PC logic. Your job is a THREE-WAY comparison to find and ATTRIBUTE those divergences.

Read all of these files (use multiple Read calls if large):
- Rust port (under audit): ${p.rust}
- PC oracle (source of truth): ${p.c}
${p.xbox ? `- Xbox grayj (old origin): ${p.xbox}` : `- Xbox grayj: NONE — this file has no Xbox counterpart (PC-only content). Do a two-way Rust-vs-PC comparison only; nothing here can be xbox-residue, so never use that class for this file.`}

Compare FUNCTION BY FUNCTION. For each function, first decide: does the Rust body's *behavior* match the PC oracle? If yes, it is NOT a finding. If it diverges from PC, it IS a finding — then ATTRIBUTE it by checking the Xbox version:

CLASS (this is the key column — base it on what the Rust matches):
- **xbox-residue** — Rust diverges from PC oracle but MATCHES the Xbox grayj version. This is the primary target: the port followed old Xbox logic. ${p.xbox ? '' : '(Not possible for this file — no Xbox source.)'}
- **port-bug** — Rust diverges from PC oracle AND from Xbox grayj (a novel translation error / incomplete stub belonging to neither).
- **intentional** — Rust diverges from PC but in a clearly deliberate, behavior-preserving way (rare; explain only if confident).
- **unsure** — diverges from PC but you cannot confidently attribute (e.g. Xbox and PC are both different in ways that don't cleanly match).

REPORT a finding for real BEHAVIORAL divergences from the PC oracle, e.g.:
- different numeric constants, thresholds, magic numbers
- different branch conditions, comparison operators, control flow
- extra/missing logic; a function present in one but absent in another

IGNORE (not findings): formatting, statement reordering, comments; idiomatic Rust translation that preserves semantics (Option vs raw pointer, slices vs ptr+len, iterators vs index loops, enums vs #define, match vs switch); naming/casing; \`unsafe\` blocks with identical semantics.

Use the Write tool to write a markdown report to EXACTLY this path: \`${p.out}\`

Structure:
\`\`\`
# Discrepancies: ${p.rust}

- **Rust:** ${p.rust}
- **PC oracle:** ${p.c}
- **Xbox grayj:** ${p.xbox || 'none (PC-only file)'}
- **Verdict:** <one line: e.g. "Clean", "N divergences (M xbox-residue)">

## Findings

| Function | Rust line(s) | PC line(s) | Xbox line(s) | First divergence | Class |
|----------|--------------|-----------|--------------|------------------|-------|
| ... | ... | ... | ... | <pointer to SUSPECTED FIRST point of divergence> | xbox-residue / port-bug / intentional / unsure |
\`\`\`

Rules for the columns:
- **Xbox line(s)**: where the matching/corresponding code sits in the Xbox grayj file (or \`n/a\` if that file is absent, or \`absent\` if the function/logic doesn't exist in Xbox).
- **First divergence**: POINT AT where the divergence from the PC oracle first appears — do NOT describe it. Give a \`file:line\` pointer plus the minimal offending token/snippet, e.g. \`g_misc.rs:419 — \\\`if health > 0 {\\\`\`.
- For any row classed **xbox-residue**, you MUST have verified the Rust matches the Xbox version at the cited Xbox line(s).

Under the table, add a short "### Detail" section ONLY for xbox-residue and unsure rows: 2-4 lines each showing the Rust snippet, the PC snippet, and the Xbox snippet so a human can confirm the attribution. Keep port-bug rows to the table only.

If there are NO behavioral divergences from the PC oracle, still write the file with the header and "No behavioral discrepancies found." under Findings.

After writing the file, return ONE line: the path plus counts, e.g. "${p.out}: 4 divergences (2 xbox-residue, 2 port-bug)". Your returned text is data, not a message to a human.`

const results = await parallel(pairs.map((p) => () =>
  agent(prompt(p), { label: p.out.replace('discrepancies/', '').replace('.md', ''), model: 'sonnet' })
))

const ok = results.filter(Boolean)
log(`Done: ${ok.length}/${pairs.length} reports written`)
return { written: ok.length, total: pairs.length, summaries: ok }
