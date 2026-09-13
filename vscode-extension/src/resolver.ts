import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

export interface ResolverOptions {
  configuredPath?: string;
  workspaceFolders?: string[];
  documentPath?: string;
  envPath?: string;
  platform?: NodeJS.Platform;
  homeDir?: string;
  isExecutableFn?: (filePath: string) => boolean;
}

/**
 * Expands leading `~` or `~/` to the user's home directory.
 */
export function expandHome(filePath: string, homeDir: string = os.homedir()): string {
  if (filePath === "~") {
    return homeDir;
  }
  if (filePath.startsWith("~/") || filePath.startsWith("~\\")) {
    return path.join(homeDir, filePath.slice(2));
  }
  return filePath;
}

/**
 * Checks if a file exists and is executable.
 */
export function isExecutable(filePath: string): boolean {
  try {
    if (!fs.existsSync(filePath)) {
      return false;
    }
    const stat = fs.statSync(filePath);
    if (!stat.isFile()) {
      return false;
    }
    if (process.platform === "win32") {
      return true;
    }
    fs.accessSync(filePath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

/**
 * Checks if an executable exists within system PATH directories.
 */
export function findInPath(
  executable: string,
  envPath: string = process.env.PATH || "",
  isExecutableFn: (p: string) => boolean = isExecutable,
): string | undefined {
  if (!envPath) {
    return undefined;
  }
  const dirs = envPath.split(path.delimiter);
  for (const dir of dirs) {
    if (!dir) {
      continue;
    }
    const fullPath = path.join(dir, executable);
    if (isExecutableFn(fullPath)) {
      return fullPath;
    }
  }
  return undefined;
}

/**
 * Resolves the path to the agent-md executable:
 * 1. Honors explicitly configured path (with ~ expansion and relative path handling).
 * 2. If configured path is default 'agent-md', checks PATH.
 * 3. Falls back to workspace target directories (target/release, target/debug, parent target).
 * 4. Falls back to common user and system install directories (~/.cargo/bin, ~/.local/bin, ~/bin/release, ~/bin, /usr/local/bin, /opt/homebrew/bin).
 * 5. Returns 'agent-md' if no candidate was found so the spawn error handler can report.
 */
export function resolveAgentMdPath(options: ResolverOptions = {}): string {
  const platform = options.platform ?? process.platform;
  const binaryName = platform === "win32" ? "agent-md.exe" : "agent-md";
  const configured = options.configuredPath?.trim();
  const homeDir = options.homeDir ?? os.homedir();
  const checkExecutable = options.isExecutableFn ?? isExecutable;

  // 1. Explicitly configured path (non-default)
  if (configured && configured !== "agent-md" && configured !== "agent-md.exe") {
    const expanded = expandHome(configured, homeDir);
    if (path.isAbsolute(expanded)) {
      return expanded;
    }
    // Check if relative to workspace folders
    if (options.workspaceFolders && options.workspaceFolders.length > 0) {
      for (const folder of options.workspaceFolders) {
        const candidate = path.resolve(folder, expanded);
        if (checkExecutable(candidate)) {
          return candidate;
        }
      }
    }
    return expanded;
  }

  // 2. Check system PATH
  const envPath = options.envPath ?? process.env.PATH ?? "";
  const foundInPath = findInPath(binaryName, envPath, checkExecutable);
  if (foundInPath) {
    return binaryName;
  }

  // 3. Check workspace target directories
  const candidateFolders: string[] = [];
  if (options.workspaceFolders) {
    for (const folder of options.workspaceFolders) {
      if (!candidateFolders.includes(folder)) {
        candidateFolders.push(folder);
      }
      // Also check parent directory of workspace folder (e.g. repo root when workspace is vscode-extension)
      const parentDir = path.dirname(folder);
      if (!candidateFolders.includes(parentDir)) {
        candidateFolders.push(parentDir);
      }
    }
  }

  if (options.documentPath) {
    let currentDir = path.dirname(options.documentPath);
    while (currentDir && currentDir !== path.dirname(currentDir)) {
      if (candidateFolders.length < 8 && !candidateFolders.includes(currentDir)) {
        candidateFolders.push(currentDir);
      } else {
        break;
      }
      currentDir = path.dirname(currentDir);
    }
  }

  for (const folder of candidateFolders) {
    const releasePath = path.join(folder, "target", "release", binaryName);
    if (checkExecutable(releasePath)) {
      return releasePath;
    }
    const debugPath = path.join(folder, "target", "debug", binaryName);
    if (checkExecutable(debugPath)) {
      return debugPath;
    }
  }

  // 4. Check common user and system install directories
  const commonFallbacks = [
    path.join(homeDir, ".cargo", "bin", binaryName),
    path.join(homeDir, ".local", "bin", binaryName),
    path.join(homeDir, "bin", "release", binaryName),
    path.join(homeDir, "bin", binaryName),
    path.join("/usr", "local", "bin", binaryName),
    path.join("/opt", "homebrew", "bin", binaryName),
  ];

  for (const candidate of commonFallbacks) {
    if (checkExecutable(candidate)) {
      return candidate;
    }
  }

  return binaryName;
}
