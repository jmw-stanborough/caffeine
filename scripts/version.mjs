import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SEMVER_PATTERN = /^\d+\.\d+\.\d+$/;
const CARGO_VERSION_PATTERN = /(\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")/m;

const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJsonPath = resolve(rootDir, "package.json");
const tauriConfPath = resolve(rootDir, "src-tauri/tauri.conf.json");
const cargoTomlPath = resolve(rootDir, "src-tauri/Cargo.toml");

function assertValidSemver(version, source) {
  if (typeof version !== "string" || !SEMVER_PATTERN.test(version)) {
    throw new Error(`Invalid semver "${version}" in ${source}; expected X.Y.Z`);
  }
  return version;
}

function readJsonVersion(filePath) {
  const data = JSON.parse(readFileSync(filePath, "utf-8"));
  return assertValidSemver(data.version, filePath);
}

function writeJsonVersion(filePath, version) {
  const data = JSON.parse(readFileSync(filePath, "utf-8"));
  data.version = version;
  writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, "utf-8");
}

function readCargoVersion(filePath) {
  const content = readFileSync(filePath, "utf-8");
  const match = content.match(CARGO_VERSION_PATTERN);
  if (!match) {
    throw new Error(`Could not locate [package] version in ${filePath}`);
  }
  return assertValidSemver(match[2], filePath);
}

function writeCargoVersion(filePath, version) {
  const content = readFileSync(filePath, "utf-8");
  if (!CARGO_VERSION_PATTERN.test(content)) {
    throw new Error(`Could not locate [package] version in ${filePath}`);
  }
  const updated = content.replace(
    CARGO_VERSION_PATTERN,
    (_, prefix, _old, suffix) => `${prefix}${version}${suffix}`,
  );
  writeFileSync(filePath, updated, "utf-8");
}

function writeAllVersions(version) {
  assertValidSemver(version, "argument");
  writeJsonVersion(packageJsonPath, version);
  writeJsonVersion(tauriConfPath, version);
  writeCargoVersion(cargoTomlPath, version);
}

function patchVersion() {
  const current = readJsonVersion(packageJsonPath);
  const [major, minor, patch] = current.split(".").map(Number);
  const next = `${major}.${minor}.${patch + 1}`;
  writeAllVersions(next);
  console.log(`Updated version: ${current} -> ${next}`);
}

function setVersion(target) {
  if (!target) {
    throw new Error("Missing version argument; usage: node scripts/version.mjs set <X.Y.Z>");
  }
  assertValidSemver(target, "argument");
  writeAllVersions(target);
  console.log(`Set version to ${target}`);
}

function tagRelease() {
  const pkgVersion = readJsonVersion(packageJsonPath);
  const tauriVersion = readJsonVersion(tauriConfPath);
  const cargoVersion = readCargoVersion(cargoTomlPath);

  if (pkgVersion !== tauriVersion || pkgVersion !== cargoVersion) {
    throw new Error(
      `Version mismatch across manifests: package.json=${pkgVersion}, tauri.conf.json=${tauriVersion}, Cargo.toml=${cargoVersion}`,
    );
  }

  const tag = `v${pkgVersion}`;
  execFileSync("git", ["tag", tag], { cwd: rootDir, stdio: "inherit" });
  execFileSync("git", ["push", "origin", tag], { cwd: rootDir, stdio: "inherit" });
  console.log(`Created and pushed tag ${tag}`);
}

const [command, arg] = process.argv.slice(2);

switch (command) {
  case "patch":
    patchVersion();
    break;
  case "set":
    setVersion(arg);
    break;
  case "tag":
    tagRelease();
    break;
  default:
    throw new Error(`Unknown subcommand "${command ?? ""}"; expected "patch", "set", or "tag"`);
}
