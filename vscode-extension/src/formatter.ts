import { spawn } from "node:child_process";

export interface FormatOptions {
  removeBold: boolean;
  compactBlankLines: boolean;
  collapseSpaces: boolean;
  removeHorizontalRules: boolean;
  removeEmphasis: boolean;
}

export const DEFAULT_FORMAT_OPTIONS: FormatOptions = {
  removeBold: true,
  compactBlankLines: true,
  collapseSpaces: true,
  removeHorizontalRules: true,
  removeEmphasis: true,
};

/**
 * Builds the CLI arguments for `agent-md fmt --stdin` based on formatting options.
 *
 * When an option is false, an explicit `--<flag>=false` is passed to disable
 * the default agent-md behavior.
 */
export function buildFormatArgs(options: Partial<FormatOptions> = {}): string[] {
  const opts = { ...DEFAULT_FORMAT_OPTIONS, ...options };
  const args: string[] = ["fmt", "--stdin"];

  if (!opts.removeBold) {
    args.push("--remove-bold=false");
  }
  if (!opts.compactBlankLines) {
    args.push("--compact-blank-lines=false");
  }
  if (!opts.collapseSpaces) {
    args.push("--collapse-spaces=false");
  }
  if (!opts.removeHorizontalRules) {
    args.push("--remove-horizontal-rules=false");
  }
  if (!opts.removeEmphasis) {
    args.push("--remove-emphasis=false");
  }

  return args;
}

/**
 * Formats Markdown content by executing `agent-md fmt --stdin` with the provided options.
 */
export function formatContent(
  content: string,
  executablePath: string,
  options: Partial<FormatOptions> = {},
  cwd?: string,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const args = buildFormatArgs(options);
    const process = spawn(executablePath, args, { cwd });

    let stdout = "";
    let stderr = "";

    process.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    process.stderr.on("data", (data) => {
      stderr += data.toString();
    });

    process.on("close", (code) => {
      if (code === 0) {
        resolve(stdout);
      } else {
        reject(new Error(`agent-md fmt failed with code ${code}: ${stderr.trim()}`));
      }
    });

    process.on("error", (err) => {
      if ((err as NodeJS.ErrnoException).code === "ENOENT") {
        reject(
          new Error(
            `agent-md executable not found at "${executablePath}". Please ensure it's installed and in your PATH, or configure the correct path in settings.`,
          ),
        );
      } else {
        reject(err);
      }
    });

    process.stdin.write(content);
    process.stdin.end();
  });
}
