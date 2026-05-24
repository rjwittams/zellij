import { defineCollection, z } from "astro:content";
import { file, glob } from "astro/loaders";

const audienceSchema = z.enum(["upstream", "fork-only", "undecided"]);
const kindSchema = z.enum(["leaf", "rollup"]);
const statusSchema = z.enum(["draft", "ready", "verified"]);

const verificationSchema = z.object({
  last_verified_at: z.string().nullable(),
  parents_hash: z.string().nullable(),
  tests_passed: z.boolean().nullable(),
  flatten_eligible: z.boolean().nullable(),
});

const sliceSchema = z.object({
  id: z.string(),
  branch: z.string(),
  purpose: z.string(),
  audience: audienceSchema,
  parents: z.array(z.string()),
  paths: z.array(z.string()),
  kind: kindSchema,
  children: z.array(z.string()),
  status: statusSchema,
  verification: verificationSchema,
  previous_bookmarks: z.array(z.string()),
});

const slices = defineCollection({
  loader: glob({ pattern: "*.json", base: "./src/content/slices" }),
  schema: sliceSchema,
});

const repoRefSchema = z.object({
  repo: z.string(),
  default_branch: z.string(),
});

const manifestMeta = defineCollection({
  loader: file("./src/content/manifest-meta.json"),
  schema: z.object({
    id: z.string(),
    version: z.number(),
    upstream: repoRefSchema,
    fork: repoRefSchema,
    test_command: z.object({
      default: z.string(),
      per_slice_overrides: z.record(z.string(), z.string()),
    }),
    slice_count: z.number(),
  }),
});

const dag = defineCollection({
  loader: file("./src/content/dag.json"),
  schema: z.object({
    id: z.string(),
    nodes: z.array(
      z.object({
        id: z.string(),
        audience: audienceSchema,
        kind: kindSchema,
        status: statusSchema,
      }),
    ),
    edges: z.array(
      z.object({
        from: z.string(),
        to: z.string(),
      }),
    ),
  }),
});

const prData = defineCollection({
  loader: glob({ pattern: "*.json", base: "./src/content/pr-data" }),
  schema: z.object({
    slice_id: z.string(),
    prs: z.array(
      z.object({
        url: z.string(),
        state: z.string(),
        mergedAt: z.string().nullable(),
        closedAt: z.string().nullable(),
        number: z.number(),
        title: z.string(),
      }),
    ),
  }),
});

export const collections = { slices, manifestMeta, dag, prData };
