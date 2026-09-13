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
 * Expands leading `~` or `~/` or `~\` to the user's home directory.
 */
export function expandHome(filePath: string, homeDir: string = os.homedir()): string {
  if (filePath === "~") {
    return homeDir;
  }
  if (filePath.startsWith("~/") || filePath.startsWith("~\\")) {
    const subPath = filePath.slice(2);
    // Use backslash if input had backslash or if homeDir contains backslashes
    const separator = filePath.startsWith("~\\") || homeDir.includes("\\") ? "\\" : "/";
    return `${homeDir}${separator}${subPath}`;
  }
  return filePath;
}

/**
 * Checks if a file exists and is executable.
 */
export function isExecutable(
  filePath: string,
  platform: NodeJS.Platform = process.platform,
): boolean {
  try {
    if (!fs.existsSync(filePath)) {
      return false;
    }
    const stat = fs.statSync(filePath);
    if (!stat.isFile()) {
      return false;
    }
    if (platform === "win32") {
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
  envPath: string = process.env.PATH || process.env.Path || "",
  isExecutableFn: (p: string) => boolean = isExecutable,
  platform: NodeJS.Platform = process.platform,
): string | undefined {
  if (!envPath) {
    return undefined;
  }
  const pathMod = platform === "win32" ? path.win32 : path;
  const delimiter = platform === "win32" || envPath.includes(";") ? ";" : pathMod.delimiter;
  const dirs = envPath.split(delimiter);
  for (const dir of dirs) {
    if (!dir) {
      continue;
    }
    const fullPath = pathMod.join(dir, executable);
    if (isExecutableFn(fullPath)) {
      return fullPath;
    }
    // On Windows, also test with .exe suffix if not already present
    if (!executable.toLowerCase().endsWith(".exe")) {
      const fullPathExe = pathMod.join(dir, `${executable}.exe`);
      if (isExecutableFn(fullPathExe)) {
        return fullPathExe;
      }
    }
  }
  return undefined;
}

/**
 * Resolves the path to the agent-md executable:
 * 1. Honors explicitly configured path (with ~ expansion, .exe resolution, and relative path handling).
 * 2. If configured path is default ('agent-md' / 'agent-md.exe'), checks PATH.
 * 3. Falls back to workspace target directories (target/release, target/debug, parent target).
 * 4. Falls back to common user and system install directories:
 *    - Windows: ~/.cargo/bin, ~/scoop/shims, %LOCALAPPDATA%/Microsoft/WinGet/Links, Chocolatey, etc.
 *    - Unix: ~/.cargo/bin, ~/.local/bin, ~/bin/release, ~/bin, /usr/local/bin, /opt/homebrew/bin.
 * 5. Returns platform default binary name ('agent-md.exe' on Windows, 'agent-md' on Unix) if not found.
 */
export function resolveAgentMdPath(options: ResolverOptions = {}): string {
  const platform = options.platform ?? process.platform;
  const pathMod = platform === "win32" ? path.win32 : path;
  const binaryName = platform === "win32" ? "agent-md.exe" : "agent-md";
  const configured = options.configuredPath?.trim();
  const homeDir = options.homeDir ?? os.homedir();
  const checkExecutable = options.isExecutableFn ?? ((p: string) => isExecutable(p, platform));

  // 1. Explicitly configured path (non-default)
  if (configured && configured !== "agent-md" && configured !== "agent-md.exe") {
    let expanded = expandHome(configured, homeDir);
    if (platform === "win32" && !expanded.toLowerCase().endsWith(".exe")) {
      const withExe = `${expanded}.exe`;
      if (checkExecutable(withExe)) {
        expanded = withExe;
      }
    }
    if (pathMod.isAbsolute(expanded)) {
      return expanded;
    }
    // Check if relative to workspace folders
    if (options.workspaceFolders && options.workspaceFolders.length > 0) {
      for (const folder of options.workspaceFolders) {
        const candidate = pathMod.resolve(folder, expanded);
        if (checkExecutable(candidate)) {
          return candidate;
        }
        if (platform === "win32" && !candidate.toLowerCase().endsWith(".exe")) {
          const candidateExe = `${candidate}.exe`;
          if (checkExecutable(candidateExe)) {
            return candidateExe;
          }
        }
      }
    }
    return expanded;
  }

  // 2. Check system PATH
  const envPath = options.envPath ?? process.env.PATH ?? process.env.Path ?? "";
  const foundInPath = findInPath(binaryName, envPath, checkExecutable, platform);
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
      const parentDir = pathMod.dirname(folder);
      if (!candidateFolders.includes(parentDir)) {
        candidateFolders.push(parentDir);
      }
    }
  }

  if (options.documentPath) {
    let currentDir = pathMod.dirname(options.documentPath);
    while (currentDir && currentDir !== pathMod.dirname(currentDir)) {
      if (candidateFolders.length < 8 && !candidateFolders.includes(currentDir)) {
        candidateFolders.push(currentDir);
      } else {
        break;
      }
      currentDir = pathMod.dirname(currentDir);
    }
  }

  for (const folder of candidateFolders) {
    const releasePath = pathMod.join(folder, "target", "release", binaryName);
    if (checkExecutable(releasePath)) {
      return releasePath;
    }
    const debugPath = pathMod.join(folder, "target", "debug", binaryName);
    if (checkExecutable(debugPath)) {
      return debugPath;
    }
  }

  // 4. Check common user and system install directories
  const commonFallbacks: string[] = [];
  if (platform === "win32") {
    const localAppData = process.env.LOCALAPPDATA || pathMod.join(homeDir, "AppData", "Local");
    const programData = process.env.ProgramData || "C:\\ProgramData";
    const chocoInstall = process.env.ChocolateyInstall || pathMod.join(programData, "chocolatey");

    commonFallbacks.push(
      pathMod.join(homeDir, ".cargo", "bin", binaryName),
      pathMod.join(homeDir, "scoop", "shims", binaryName),
      pathMod.join(localAppData, "Microsoft", "WinGet", "Links", binaryName),
      pathMod.join(localAppData, "bin", binaryName),
      pathMod.join(chocoInstall, "bin", binaryName),
      pathMod.join(homeDir, "bin", binaryName),
    );
  } else {
    commonFallbacks.push(
      pathMod.join(homeDir, ".cargo", "bin", binaryName),
      pathMod.join(homeDir, ".local", "bin", binaryName),
      pathMod.join(homeDir, "bin", "release", binaryName),
      pathMod.join(homeDir, "bin", binaryName),
      pathMod.join("/usr", "local", "bin", binaryName),
      pathMod.join("/opt", "homebrew", "bin", binaryName),
    );
  }

  for (const candidate of commonFallbacks) {
    if (checkExecutable(candidate)) {
      return candidate;
    }
  }

  return binaryName;
}
