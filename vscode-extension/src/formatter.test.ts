import { describe, expect, it } from "bun:test";
import * as path from "node:path";
import * as fs from "node:fs";
import {
  DEFAULT_FORMAT_OPTIONS,
  buildFormatArgs,
  formatContent,
  type FormatOptions,
} from "./formatter";
import { resolveAgentMdPath } from "./resolver";

describe("formatter", () => {
  describe("DEFAULT_FORMAT_OPTIONS", () => {
    it("has all 5 settings enabled by default", () => {
      expect(DEFAULT_FORMAT_OPTIONS).toEqual({
        removeBold: true,
        compactBlankLines: true,
        collapseSpaces: true,
        removeHorizontalRules: true,
        removeEmphasis: true,
      });
    });
  });

  describe("buildFormatArgs", () => {
    it("returns base args ['fmt', '--stdin'] when default options are used", () => {
      const args = buildFormatArgs();
      expect(args).toEqual(["fmt", "--stdin"]);
    });

    it("returns base args ['fmt', '--stdin'] when all options are explicitly true", () => {
      const args = buildFormatArgs({
        removeBold: true,
        compactBlankLines: true,
        collapseSpaces: true,
        removeHorizontalRules: true,
        removeEmphasis: true,
      });
      expect(args).toEqual(["fmt", "--stdin"]);
    });

    describe("agentMd.format.removeBold", () => {
      it("adds --remove-bold=false when removeBold is false", () => {
        const args = buildFormatArgs({ removeBold: false });
        expect(args).toContain("--remove-bold=false");
        expect(args.length).toBe(3);
      });
    });

    describe("agentMd.format.compactBlankLines", () => {
      it("adds --compact-blank-lines=false when compactBlankLines is false", () => {
        const args = buildFormatArgs({ compactBlankLines: false });
        expect(args).toContain("--compact-blank-lines=false");
        expect(args.length).toBe(3);
      });
    });

    describe("agentMd.format.collapseSpaces", () => {
      it("adds --collapse-spaces=false when collapseSpaces is false", () => {
        const args = buildFormatArgs({ collapseSpaces: false });
        expect(args).toContain("--collapse-spaces=false");
        expect(args.length).toBe(3);
      });
    });

    describe("agentMd.format.removeHorizontalRules", () => {
      it("adds --remove-horizontal-rules=false when removeHorizontalRules is false", () => {
        const args = buildFormatArgs({ removeHorizontalRules: false });
        expect(args).toContain("--remove-horizontal-rules=false");
        expect(args.length).toBe(3);
      });
    });

    describe("agentMd.format.removeEmphasis", () => {
      it("adds --remove-emphasis=false when removeEmphasis is false", () => {
        const args = buildFormatArgs({ removeEmphasis: false });
        expect(args).toContain("--remove-emphasis=false");
        expect(args.length).toBe(3);
      });
    });

    describe("combinations of settings", () => {
      it("includes all corresponding --<flag>=false arguments when multiple settings are disabled", () => {
        const args = buildFormatArgs({
          removeBold: false,
          removeEmphasis: false,
        });
        expect(args).toEqual(["fmt", "--stdin", "--remove-bold=false", "--remove-emphasis=false"]);
      });

      it("includes all 5 false flags when every setting is disabled", () => {
        const args = buildFormatArgs({
          removeBold: false,
          compactBlankLines: false,
          collapseSpaces: false,
          removeHorizontalRules: false,
          removeEmphasis: false,
        });
        expect(args).toEqual([
          "fmt",
          "--stdin",
          "--remove-bold=false",
          "--compact-blank-lines=false",
          "--collapse-spaces=false",
          "--remove-horizontal-rules=false",
          "--remove-emphasis=false",
        ]);
      });
    });
  });

  describe("formatContent (end-to-end with binary)", () => {
    const repoRoot = path.resolve(__dirname, "../..");
    const binaryPath = resolveAgentMdPath({
      workspaceFolders: [repoRoot],
      envPath: "",
    });

    const isBinaryAvailable = fs.existsSync(binaryPath);

    it("rejects with descriptive error when binary does not exist", async () => {
      const nonExistentPath = "/non/existent/path/to/agent-md";
      await expect(
        formatContent("# Test", nonExistentPath, {}),
      ).rejects.toThrow("agent-md executable not found at");
    });

    if (isBinaryAvailable) {
      it("setting agentMd.format.removeBold: false preserves **bold** and __bold__ markers", async () => {
        const input = "Here is **bold text** and __more bold__.\n";
        const result = await formatContent(input, binaryPath, { removeBold: false });
        expect(result).toContain("**bold text**");
        expect(result).toContain("__more bold__");
      });

      it("setting agentMd.format.compactBlankLines: false preserves multiple blank lines", async () => {
        const input = "Line 1\n\n\n\nLine 2\n";
        const result = await formatContent(input, binaryPath, { compactBlankLines: false });
        expect(result).toBe("Line 1\n\n\n\nLine 2\n");
      });

      it("setting agentMd.format.compactBlankLines: true compacts multiple blank lines to one", async () => {
        const input = "Line 1\n\n\n\nLine 2\n";
        const result = await formatContent(input, binaryPath, { compactBlankLines: true });
        expect(result).toBe("Line 1\n\nLine 2\n");
      });

      it("setting agentMd.format.collapseSpaces: false preserves multiple spaces", async () => {
        const input = "Word1     Word2\n";
        const result = await formatContent(input, binaryPath, { collapseSpaces: false });
        expect(result).toContain("Word1     Word2");
      });

      it("setting agentMd.format.collapseSpaces: true collapses multiple spaces to single", async () => {
        const input = "Word1     Word2\n";
        const result = await formatContent(input, binaryPath, { collapseSpaces: true });
        expect(result).toBe("Word1 Word2\n");
      });

      it("setting agentMd.format.removeHorizontalRules: false preserves horizontal rules", async () => {
        const input = "Section 1\n\n---\n\nSection 2\n";
        const result = await formatContent(input, binaryPath, { removeHorizontalRules: false });
        expect(result).toContain("---");
      });

      it("setting agentMd.format.removeHorizontalRules: true removes horizontal rules", async () => {
        const input = "Section 1\n\n---\n\nSection 2\n";
        const result = await formatContent(input, binaryPath, { removeHorizontalRules: true });
        expect(result).not.toContain("---");
      });

      it("setting agentMd.format.removeEmphasis: false preserves *italic* and _italic_ markers", async () => {
        const input = "Here is *italic* and _italic2_.\n";
        const result = await formatContent(input, binaryPath, { removeEmphasis: false });
        expect(result).toContain("*italic*");
        expect(result).toContain("_italic2_");
      });

      it("setting agentMd.format.removeEmphasis: true removes emphasis markers", async () => {
        const input = "Here is *italic* and _italic2_.\n";
        const result = await formatContent(input, binaryPath, { removeEmphasis: true });
        expect(result).not.toContain("*italic*");
        expect(result).not.toContain("_italic2_");
        expect(result).toContain("italic and italic2");
      });

      it("rejects with syntax error when markdown has unclosed code fence", async () => {
        const malformed = "```typescript\nconst x = 1;\n";
        await expect(
          formatContent(malformed, binaryPath, {}),
        ).rejects.toThrow("Syntax Error: Code block starting at line 1 is missing a closing fence");
      });
    }
  });
});
