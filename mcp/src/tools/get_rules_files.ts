import { z } from "zod";
import { Tool } from "./index.js";
import { parseSchema } from "./utils.js";
import * as G from "../graph/graph.js";

export const RULES_PATTERNS = [
  "/.windsurfrules",
  "/.cursorrules",
  "/CLAUDE.md",
  "/.cursor/rules/",
  "/AGENTS.md",
  "**/.goosehints",
];

export const GetRulesFilesTool: Tool = {
  name: "mesh_rules_files",
  description:
    "Fetch rules files (e.g. .cursorrules, .windsurfrules, CLAUDE.md, etc.) as code snippets.",
  inputSchema: {},
};

export async function getRulesFiles() {
  const snippets = await G.get_rules_files();
  return {
    patterns_searched: RULES_PATTERNS,
    files_found: snippets.length,
    snippets,
  };
}
