#!/usr/bin/env node
/**
 * Pre-build hook: for each slice with audience=upstream, fetch PR data via gh
 * and write to src/content/pr-data/<id>.json. Graceful on gh unavailability.
 */
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const slicesDir = join(__dirname, "..", "src", "content", "slices");
const prDataDir = join(__dirname, "..", "src", "content", "pr-data");

function hasGh() {
  try {
    execFileSync("gh", ["--version"], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

function fetchPrsForBranch(branch) {
  try {
    const out = execFileSync(
      "gh",
      [
        "pr", "list",
        "--head", branch,
        "--state", "all",
        "--json", "url,state,mergedAt,closedAt,number,title",
      ],
      { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    );
    return JSON.parse(out);
  } catch (err) {
    console.warn(`  warning: failed to fetch PRs for ${branch}: ${err.message}`);
    return [];
  }
}

function main() {
  if (!existsSync(slicesDir)) {
    console.log("fetch-pr-data: no slices directory; skipping");
    return;
  }
  if (!hasGh()) {
    console.log("fetch-pr-data: gh CLI not available; skipping PR fetch");
    return;
  }
  const files = readdirSync(slicesDir).filter((f) => f.endsWith(".json"));
  console.log(`fetch-pr-data: scanning ${files.length} slice(s)`);
  mkdirSync(prDataDir, { recursive: true });

  for (const file of files) {
    const slice = JSON.parse(readFileSync(join(slicesDir, file), "utf8"));
    if (slice.audience !== "upstream") continue;
    console.log(`  fetching PRs for ${slice.id} (branch: ${slice.branch})`);
    const prs = fetchPrsForBranch(slice.branch);
    const payload = { slice_id: slice.id, prs };
    writeFileSync(
      join(prDataDir, `${slice.id}.json`),
      JSON.stringify(payload, null, 2) + "\n",
    );
  }
  console.log("fetch-pr-data: done");
}

main();
