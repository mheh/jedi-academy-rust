#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const sourceRoots = ["codemp", "code"];
const sourceExtensions = new Set([".c", ".cpp", ".h"]);

function walk(dir) {
  if (!fs.existsSync(dir)) {
    return [];
  }

  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...walk(full));
    } else if (entry.isFile()) {
      out.push(full);
    }
  }
  return out;
}

function toPosix(p) {
  return p.split(path.sep).join("/");
}

function expectedRustPaths(oraclePath) {
  const rel = toPosix(path.relative(path.join(root, "oracle"), oraclePath));
  const parsed = path.posix.parse(rel);
  const base = parsed.name;
  const dir = parsed.dir;

  if (parsed.ext === ".h") {
    return [
      path.join(root, "src", dir, `${base}_h.rs`),
      path.join(root, "src", dir, `${base}.rs`),
    ];
  }

  return [path.join(root, "src", dir, `${base}.rs`)];
}

function includeCount(file) {
  const text = fs.readFileSync(file, "utf8");
  return (text.match(/^\s*#\s*include\b/gm) ?? []).length;
}

const unpaired = [];
const paired = [];

for (const sourceRoot of sourceRoots) {
  const oracleRoot = path.join(root, "oracle", sourceRoot);
  for (const file of walk(oracleRoot)) {
    const ext = path.extname(file);
    if (!sourceExtensions.has(ext)) {
      continue;
    }

    const candidates = expectedRustPaths(file);
    const match = candidates.find((candidate) => fs.existsSync(candidate));
    const rel = toPosix(path.relative(root, file));
    const destination = toPosix(path.relative(root, candidates[0]));
    const row = {
      oracle: rel,
      expected: destination,
      includes: includeCount(file),
    };

    if (match) {
      paired.push({ ...row, paired: toPosix(path.relative(root, match)) });
    } else {
      unpaired.push(row);
    }
  }
}

unpaired.sort((a, b) => a.includes - b.includes || a.oracle.localeCompare(b.oracle));
paired.sort((a, b) => a.oracle.localeCompare(b.oracle));

const result = {
  generatedBy: "scripts/find-unpaired-oracle-files.mjs",
  roots: sourceRoots.map((name) => ({ oracle: `oracle/${name}`, rust: `src/${name}` })),
  totals: {
    paired: paired.length,
    unpaired: unpaired.length,
  },
  unpaired,
  paired,
};

console.log(JSON.stringify(result, null, 2));
