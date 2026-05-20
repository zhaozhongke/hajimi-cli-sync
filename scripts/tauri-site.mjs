#!/usr/bin/env node

import { mkdirSync, readFileSync, statSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = join(__dirname, "..");
const tauriDir = join(repoRoot, "src-tauri");
const generatedConfigPath = join(tauriDir, "tauri.site.conf.json");
const profiles = JSON.parse(readFileSync(join(repoRoot, "src/site-profiles.json"), "utf8"));
const npxCommand = process.platform === "win32" ? "npx.cmd" : "npx";
const npxOptions = {
  cwd: repoRoot,
  stdio: "inherit",
  env: process.env,
  shell: process.platform === "win32"
};

const [, , command, ...rawArgs] = process.argv;
if (!command || !["build", "dev"].includes(command)) {
  console.error("Usage: node scripts/tauri-site.mjs <build|dev> [--site <site>] [tauri args...]");
  process.exit(1);
}

const args = [...rawArgs];
let site = process.env.SWITCH_SITE || "hajimi";
const siteIndex = args.indexOf("--site");
if (siteIndex >= 0) {
  site = args[siteIndex + 1];
  args.splice(siteIndex, 2);
}

const profile = profiles[site];
if (!profile) {
  console.error(`Unknown SWITCH_SITE: ${site}`);
  process.exit(1);
}

function ensureIcons() {
  const sourcePath = join(repoRoot, profile.iconSource);
  const outputDir = join(repoRoot, profile.iconOutputDir);
  const outputSentinel = join(outputDir, "icon.png");
  const sourceMtime = statSync(sourcePath).mtimeMs;
  const outputMtime = existsSync(outputSentinel) ? statSync(outputSentinel).mtimeMs : 0;
  if (outputMtime >= sourceMtime) {
    return;
  }

  mkdirSync(outputDir, { recursive: true });
  const result = spawnSync(
    npxCommand,
    ["@tauri-apps/cli", "icon", sourcePath, "--output", outputDir],
    npxOptions
  );
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function writeGeneratedConfig() {
  const iconDir = join(repoRoot, profile.iconOutputDir);
  const relativeIconDir = relative(tauriDir, iconDir).replaceAll("\\", "/");
  const config = {
    productName: profile.productName,
    identifier: profile.bundleIdentifier,
    app: {
      windows: [
        {
          title: profile.windowTitle
        }
      ]
    },
    bundle: {
      icon: [
        `${relativeIconDir}/32x32.png`,
        `${relativeIconDir}/128x128.png`,
        `${relativeIconDir}/128x128@2x.png`,
        `${relativeIconDir}/icon.icns`,
        `${relativeIconDir}/icon.ico`
      ]
    }
  };
  writeFileSync(generatedConfigPath, `${JSON.stringify(config, null, 2)}\n`, "utf8");
}

ensureIcons();
writeGeneratedConfig();

const env = {
  ...process.env,
  SWITCH_SITE: site,
  VITE_SWITCH_SITE: site
};

const tauriArgs = ["@tauri-apps/cli", command, "--config", generatedConfigPath, ...args];
const result = spawnSync(npxCommand, tauriArgs, {
  ...npxOptions,
  env
});

process.exit(result.status ?? 1);
