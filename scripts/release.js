#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { spawnSync } = require("child_process");

const repoRoot = path.resolve(__dirname, "..");
const rootPackagePath = path.join(repoRoot, "package.json");
const lockfilePath = path.join(repoRoot, "package-lock.json");
const cargoTomlPath = path.join(repoRoot, "Cargo.toml");
const shimPackagePath = path.join(
  repoRoot,
  "packages",
  "officebeats-matrix-iptv-cli",
  "package.json"
);

function fail(message) {
  console.error(`\n[release] ${message}`);
  process.exit(1);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: "inherit",
    shell: false,
    ...options,
  });

  if (result.error) {
    fail(`Failed to run ${command}: ${result.error.message}`);
  }

  if (result.status !== 0) {
    fail(`${command} ${args.join(" ")} exited with status ${result.status}`);
  }
}

function capture(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    shell: false,
    ...options,
  });

  if (result.error) {
    fail(`Failed to run ${command}: ${result.error.message}`);
  }

  if (result.status !== 0) {
    const stderr = (result.stderr || "").trim();
    fail(
      `${command} ${args.join(" ")} exited with status ${result.status}${
        stderr ? `\n${stderr}` : ""
      }`
    );
  }

  return (result.stdout || "").trim();
}

function loadJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function saveJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function ensureClean(paths) {
  const output = capture("git", ["status", "--short", "--", ...paths]);
  if (output) {
    fail(
      `Release-managed files are dirty.\nCommit or stash these first:\n${output}`
    );
  }
}

function ensureVersion(version) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    fail(`Version must be semantic version only, e.g. 4.0.21. Received: ${version}`);
  }
}

function readCargoToml() {
  return fs.readFileSync(cargoTomlPath, "utf8");
}

function writeCargoToml(content) {
  fs.writeFileSync(cargoTomlPath, content, "utf8");
}

function bumpCargoVersion(version) {
  const content = readCargoToml();
  // Match the version line under [package] — first occurrence of version = "..."
  const updated = content.replace(
    /^(version\s*=\s*")([^"]+)(")/m,
    `$1${version}$3`
  );
  if (updated === content) {
    fail("Could not find version field in Cargo.toml to update.");
  }
  writeCargoToml(updated);
}

function bumpVersion(version) {
  const rootPackage = loadJson(rootPackagePath);
  const shimPackage = loadJson(shimPackagePath);
  const hasLockfile = fs.existsSync(lockfilePath);
  const lockfile = hasLockfile ? loadJson(lockfilePath) : null;

  rootPackage.version = version;
  shimPackage.version = version;
  shimPackage.dependencies["matrix-iptv"] = version;
  if (lockfile) {
    lockfile.version = version;
  }
  if (lockfile && lockfile.packages && lockfile.packages[""]) {
    lockfile.packages[""].version = version;
  }

  saveJson(rootPackagePath, rootPackage);
  saveJson(shimPackagePath, shimPackage);
  if (lockfile) {
    saveJson(lockfilePath, lockfile);
  }

  // Bump the Rust binary version so it matches the release tag
  bumpCargoVersion(version);
}

function printUsage() {
  console.log(`Usage:
  npm run release:update -- <version>
  npm run release:update -- <version> --execute

Default mode updates package files only.
Use --execute to also commit, push main, create tag v<version>, and push the tag.`);
}

const args = process.argv.slice(2);
if (args.length === 0 || args.includes("--help") || args.includes("-h")) {
  printUsage();
  process.exit(0);
}

const version = args.find((arg) => !arg.startsWith("-"));
if (!version) {
  fail("Missing version argument.");
}

const shouldExecute = args.includes("--execute");
const managedPaths = [
  "package.json",
  "package-lock.json",
  "Cargo.toml",
  "packages/officebeats-matrix-iptv-cli/package.json",
];

ensureVersion(version);
ensureClean(managedPaths);

console.log(`[release] Updating package versions to ${version}`);
bumpVersion(version);

if (!shouldExecute) {
  console.log("[release] Version files updated. Review changes, then run with --execute when ready.");
  process.exit(0);
}

const tag = `v${version}`;
const existingLocalTag = spawnSync("git", ["rev-parse", "-q", "--verify", tag], {
  cwd: repoRoot,
  stdio: "ignore",
  shell: false,
});
if (existingLocalTag.status === 0) {
  fail(`Tag ${tag} already exists locally.`);
}

const remoteTag = spawnSync("git", ["ls-remote", "--tags", "origin", tag], {
  cwd: repoRoot,
  encoding: "utf8",
  stdio: ["ignore", "pipe", "pipe"],
  shell: false,
});
if (remoteTag.status !== 0) {
  fail(`Unable to check remote tags:\n${(remoteTag.stderr || "").trim()}`);
}
if ((remoteTag.stdout || "").trim()) {
  fail(`Tag ${tag} already exists on origin.`);
}

run("git", ["add", ...managedPaths]);
run("git", ["commit", "-m", `chore(release): bump npm packages to ${version}`]);
run("git", ["push", "origin", "main"]);
run("git", ["tag", tag]);
run("git", ["push", "origin", tag]);

console.log(`[release] Release commit pushed and tag ${tag} published.`);
console.log("[release] GitHub Actions will handle build, GitHub release, and npm publish.");
