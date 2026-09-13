import { describe, expect, it } from "bun:test";
import * as path from "node:path";
import { expandHome, findInPath, resolveAgentMdPath } from "./resolver";

describe("resolver", () => {
  describe("expandHome", () => {
    it("expands ~ alone", () => {
      expect(expandHome("~", "/home/testuser")).toBe("/home/testuser");
    });

    it("expands ~/path", () => {
      expect(expandHome("~/bin/agent-md", "/home/testuser")).toBe(
        path.join("/home/testuser", "bin/agent-md"),
      );
    });

    it("does not modify absolute or relative paths without leading ~", () => {
      expect(expandHome("/usr/bin/agent-md", "/home/testuser")).toBe("/usr/bin/agent-md");
      expect(expandHome("./target/release/agent-md", "/home/testuser")).toBe(
        "./target/release/agent-md",
      );
    });
  });

  describe("findInPath", () => {
    it("finds executable in PATH", () => {
      const fakeBin = path.join("/custom", "bin", "agent-md");
      const isExec = (p: string) => p === fakeBin;
      const result = findInPath("agent-md", "/usr/bin:/custom/bin:/bin", isExec);
      expect(result).toBe(fakeBin);
    });

    it("returns undefined if not found in PATH", () => {
      const result = findInPath("agent-md", "/usr/bin:/bin", () => false);
      expect(result).toBeUndefined();
    });
  });

  describe("resolveAgentMdPath", () => {
    it("returns configured non-default path with tilde expanded", () => {
      const result = resolveAgentMdPath({
        configuredPath: "~/custom/agent-md",
        homeDir: "/home/user",
        isExecutableFn: () => true,
      });
      expect(result).toBe(path.join("/home/user", "custom/agent-md"));
    });

    it("resolates relative configured path against workspace folders if executable", () => {
      const workspaceFolder = "/projects/my-repo";
      const expected = path.resolve(workspaceFolder, "bin/agent-md");
      const result = resolveAgentMdPath({
        configuredPath: "bin/agent-md",
        workspaceFolders: [workspaceFolder],
        isExecutableFn: (p) => p === expected,
      });
      expect(result).toBe(expected);
    });

    it("returns binaryName if found in PATH", () => {
      const result = resolveAgentMdPath({
        configuredPath: "agent-md",
        envPath: "/usr/bin:/bin",
        platform: "darwin",
        isExecutableFn: (p) => p === path.join("/usr/bin", "agent-md"),
      });
      expect(result).toBe("agent-md");
    });

    it("falls back to workspace target/release when not in PATH", () => {
      const workspaceFolder = "/projects/agent-md";
      const releaseBinary = path.join(workspaceFolder, "target", "release", "agent-md");
      const result = resolveAgentMdPath({
        configuredPath: "agent-md",
        workspaceFolders: [workspaceFolder],
        envPath: "/usr/bin:/bin",
        platform: "darwin",
        isExecutableFn: (p) => p === releaseBinary,
      });
      expect(result).toBe(releaseBinary);
    });

    it("falls back to parent workspace target/release if workspace is a subdirectory", () => {
      const workspaceFolder = "/projects/agent-md/vscode-extension";
      const rootRelease = path.join("/projects/agent-md", "target", "release", "agent-md");
      const result = resolveAgentMdPath({
        configuredPath: "agent-md",
        workspaceFolders: [workspaceFolder],
        envPath: "",
        platform: "darwin",
        isExecutableFn: (p) => p === rootRelease,
      });
      expect(result).toBe(rootRelease);
    });

    it("falls back to ~/.cargo/bin when no workspace binary exists", () => {
      const homeDir = "/home/developer";
      const cargoBinary = path.join(homeDir, ".cargo", "bin", "agent-md");
      const result = resolveAgentMdPath({
        configuredPath: "agent-md",
        workspaceFolders: ["/projects/other-repo"],
        homeDir,
        envPath: "",
        platform: "darwin",
        isExecutableFn: (p) => p === cargoBinary,
      });
      expect(result).toBe(cargoBinary);
    });

    it("falls back to default agent-md if no candidate exists", () => {
      const result = resolveAgentMdPath({
        configuredPath: "agent-md",
        workspaceFolders: ["/projects/empty"],
        homeDir: "/home/developer",
        envPath: "",
        platform: "darwin",
        isExecutableFn: () => false,
      });
      expect(result).toBe("agent-md");
    });
  });
});
