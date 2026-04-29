#!/usr/bin/env node

const { spawn, spawnSync } = require("child_process");
const crypto = require("crypto");
const fs = require("fs");
const os = require("os");
const path = require("path");
const axios = require("axios");

const repo = "officebeats/matrix-iptv";
const userAgent = "matrix-iptv-updater";
const updateExitCode = 42;
const minBinarySize = 1024 * 100;
const binaryName = os.platform() === "win32" ? "matrix-iptv.exe" : "matrix-iptv";
const binaryPath = path.join(__dirname, binaryName);
const packageVersion = require("../package.json").version;

const platformMap = {
  win32: "windows.exe",
  linux: "linux",
  darwin: "macos",
};

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function compareVersions(a, b) {
  const parse = (value) =>
    String(value || "")
      .replace(/^v/i, "")
      .split(".")
      .map((part) => Number.parseInt(part, 10) || 0);
  const left = parse(a);
  const right = parse(b);
  const length = Math.max(left.length, right.length);

  for (let i = 0; i < length; i += 1) {
    const diff = (left[i] || 0) - (right[i] || 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

function parseVersion(output) {
  const match = String(output || "").match(/(\d+\.\d+\.\d+)/);
  return match ? match[1] : null;
}

function getBinaryVersion(filePath) {
  if (!fs.existsSync(filePath)) return null;

  const result = spawnSync(filePath, ["--version"], {
    encoding: "utf8",
    timeout: 10000,
    windowsHide: true,
    env: {
      ...process.env,
      MATRIX_IPTV_WRAPPER: "1",
      MATRIX_IPTV_SKIP_UPDATE: "1",
    },
  });

  if (result.error) {
    throw new Error(`Unable to run ${path.basename(filePath)} --version: ${result.error.message}`);
  }

  if (result.status !== 0) {
    const message = (result.stderr || result.stdout || "").trim();
    throw new Error(`Version check failed for ${path.basename(filePath)}${message ? `: ${message}` : ""}`);
  }

  return parseVersion(`${result.stdout}\n${result.stderr}`);
}

function currentInstalledVersion() {
  try {
    return getBinaryVersion(binaryPath);
  } catch (err) {
    console.log(`[!] Existing binary version check failed: ${err.message}`);
    return null;
  }
}

async function fetchRelease(targetVersion) {
  const tag = targetVersion ? `v${String(targetVersion).replace(/^v/i, "")}` : null;
  const url = tag
    ? `https://api.github.com/repos/${repo}/releases/tags/${tag}`
    : `https://api.github.com/repos/${repo}/releases/latest`;

  const response = await axios.get(url, {
    timeout: 15000,
    headers: {
      Accept: "application/vnd.github+json",
      "Cache-Control": "no-cache",
      "User-Agent": userAgent,
    },
  });

  return response.data;
}

function selectAsset(release) {
  const platform = platformMap[os.platform()];
  if (!platform) {
    throw new Error(`Unsupported platform for auto-update: ${os.platform()}`);
  }

  const expectedName = `matrix-iptv-${platform}`;
  const asset = (release.assets || []).find((item) => item.name === expectedName);
  if (!asset) {
    throw new Error(`Release ${release.tag_name} does not include ${expectedName}`);
  }
  return asset;
}

async function downloadAsset(asset, destination) {
  const response = await axios({
    method: "get",
    url: asset.browser_download_url,
    responseType: "stream",
    timeout: 60000,
    headers: {
      "Cache-Control": "no-cache",
      "User-Agent": userAgent,
    },
  });

  await new Promise((resolve, reject) => {
    const writer = fs.createWriteStream(destination, { flags: "wx" });
    response.data.pipe(writer);
    writer.on("finish", () => writer.close(resolve));
    writer.on("error", (err) => {
      fs.rm(destination, { force: true }, () => {});
      reject(err);
    });
  });
}

function sha256(filePath) {
  return crypto.createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function makeExecutable(filePath) {
  if (os.platform() !== "win32") {
    fs.chmodSync(filePath, 0o755);
  }

  if (os.platform() === "darwin") {
    spawnSync("xattr", ["-d", "com.apple.quarantine", filePath], {
      stdio: "ignore",
      windowsHide: true,
    });
  }
}

function verifyDownloadedBinary(filePath, asset, expectedVersion) {
  const stat = fs.statSync(filePath);
  if (stat.size < minBinarySize) {
    throw new Error(`Downloaded file is too small (${stat.size} bytes).`);
  }

  if (asset.size && stat.size !== asset.size) {
    throw new Error(`Downloaded file size mismatch: expected ${asset.size}, got ${stat.size}.`);
  }

  if (asset.digest && asset.digest.startsWith("sha256:")) {
    const expectedDigest = asset.digest.slice("sha256:".length).toLowerCase();
    const actualDigest = sha256(filePath);
    if (actualDigest !== expectedDigest) {
      throw new Error("Downloaded binary checksum did not match the GitHub release asset digest.");
    }
  } else {
    console.log("[!] GitHub did not provide a release asset digest; continuing with size and version checks.");
  }

  makeExecutable(filePath);

  const actualVersion = getBinaryVersion(filePath);
  if (actualVersion !== expectedVersion) {
    throw new Error(`Downloaded binary reports version ${actualVersion || "unknown"}, expected ${expectedVersion}.`);
  }
}

async function retry(label, action, attempts = 15) {
  let lastError;
  for (let i = 0; i < attempts; i += 1) {
    try {
      return action();
    } catch (err) {
      lastError = err;
      if (i === attempts - 1) break;
      const delay = 750 + i * 350;
      console.log(`[*] ${label} was blocked (${err.code || err.message}). Retrying in ${delay}ms...`);
      await sleep(delay);
    }
  }
  throw lastError;
}

async function withUpdateLock(action) {
  const lockPath = path.join(os.tmpdir(), "matrix-iptv-update.lock");
  let fd;

  try {
    if (fs.existsSync(lockPath)) {
      const ageMs = Date.now() - fs.statSync(lockPath).mtimeMs;
      if (ageMs > 30 * 60 * 1000) {
        fs.rmSync(lockPath, { force: true });
      }
    }

    fd = fs.openSync(lockPath, "wx");
    fs.writeSync(fd, `${process.pid}\n${new Date().toISOString()}\n`);
  } catch (err) {
    if (err.code === "EEXIST") {
      throw new Error("Another Matrix IPTV update is already running. Try again after it finishes.");
    }
    throw err;
  }

  try {
    return await action();
  } finally {
    if (fd !== undefined) fs.closeSync(fd);
    fs.rmSync(lockPath, { force: true });
  }
}

async function installBinary(tempPath, expectedVersion) {
  const backupPath = `${binaryPath}.old-${Date.now()}${os.platform() === "win32" ? ".exe" : ""}`;
  let backupCreated = false;

  try {
    await retry("Replacing binary", () => {
      if (fs.existsSync(binaryPath) && !backupCreated) {
        fs.renameSync(binaryPath, backupPath);
        backupCreated = true;
      }
      fs.renameSync(tempPath, binaryPath);
    });
  } catch (err) {
    if (backupCreated && !fs.existsSync(binaryPath) && fs.existsSync(backupPath)) {
      fs.renameSync(backupPath, binaryPath);
    }
    throw err;
  }

  try {
    makeExecutable(binaryPath);
    const installedVersion = getBinaryVersion(binaryPath);
    if (installedVersion !== expectedVersion) {
      throw new Error(`Installed binary reports version ${installedVersion || "unknown"}, expected ${expectedVersion}.`);
    }

    if (backupCreated) {
      fs.rmSync(backupPath, { force: true });
    }
  } catch (err) {
    console.log("[!] Installed binary failed verification. Restoring previous binary...");
    fs.rmSync(binaryPath, { force: true });
    if (backupCreated && fs.existsSync(backupPath)) {
      fs.renameSync(backupPath, binaryPath);
    }
    throw err;
  }
}

async function performUpdate(options = {}) {
  const targetVersion = options.targetVersion || null;
  const relaunch = Boolean(options.relaunch);
  const relaunchArgs = options.relaunchArgs || [];

  return withUpdateLock(async () => {
    const release = await fetchRelease(targetVersion);
    const releaseVersion = String(release.tag_name || "").replace(/^v/i, "");
    const currentVersion = currentInstalledVersion();

    if (!releaseVersion) {
      throw new Error("GitHub release did not include a tag name.");
    }

    console.log(`\n[*] Matrix IPTV update check`);
    console.log(`[*] Current: ${currentVersion || "unknown"}`);
    console.log(`[*] Target : ${releaseVersion}`);

    if (currentVersion && compareVersions(currentVersion, releaseVersion) >= 0) {
      console.log("[+] Matrix IPTV is already up to date.");
      return { updated: false, version: currentVersion };
    }

    const asset = selectAsset(release);
    const tempName =
      os.platform() === "win32"
        ? `matrix-iptv-${process.pid}-${Date.now()}.download.exe`
        : `matrix-iptv-${process.pid}-${Date.now()}.download`;
    const tempPath = path.join(path.dirname(binaryPath), tempName);

    try {
      console.log(`[*] Downloading ${asset.name} from ${release.tag_name}...`);
      await downloadAsset(asset, tempPath);
      verifyDownloadedBinary(tempPath, asset, releaseVersion);
      await installBinary(tempPath, releaseVersion);
      console.log(`[+] Updated Matrix IPTV to ${releaseVersion}.`);

      if (relaunch) {
        console.log("[*] Restarting Matrix IPTV...");
        launchApp(true, relaunchArgs);
      } else {
        console.log("[+] Run 'matrix-iptv' to start the updated app.");
      }

      return { updated: true, version: releaseVersion };
    } catch (err) {
      fs.rmSync(tempPath, { force: true });
      throw err;
    }
  });
}

function parseUpdateArgs(args) {
  let targetVersion = null;
  let relaunch = false;

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--launch") {
      relaunch = true;
    } else if (arg === "--target" || arg === "--version") {
      targetVersion = args[i + 1];
      i += 1;
    } else if (arg.startsWith("--target=")) {
      targetVersion = arg.slice("--target=".length);
    } else if (arg.startsWith("--version=")) {
      targetVersion = arg.slice("--version=".length);
    } else if (/^v?\d+\.\d+\.\d+$/.test(arg)) {
      targetVersion = arg;
    } else if (arg === "--help" || arg === "-h") {
      console.log(`Usage:
  matrix-iptv update
  matrix-iptv update --target 4.3.2
  matrix-iptv update --launch

Downloads the GitHub release asset, verifies its size/checksum/version,
installs it transactionally, and rolls back if verification fails.`);
      process.exit(0);
    }
  }

  return { targetVersion, relaunch };
}

function launchApp(isUpdateRelaunch = false, args = process.argv.slice(2)) {
  if (!fs.existsSync(binaryPath)) {
    console.error("\nMatrix IPTV binary not found.");
    console.log("Try one of these:\n  matrix-iptv update\n  npm install -g matrix-iptv\n");
    process.exit(1);
  }

  let child;
  try {
    child = spawn(binaryPath, args, {
      stdio: "inherit",
      windowsHide: false,
      env: {
        ...process.env,
        MATRIX_IPTV_WRAPPER: "1",
      },
    });
  } catch (err) {
    if (isUpdateRelaunch && (err.code === "EBUSY" || err.code === "EACCES" || err.code === "EPERM")) {
      console.log("[*] Executable locked by OS. Retrying in 2 seconds...");
      setTimeout(() => launchApp(true, args), 2000);
      return;
    }
    throw err;
  }

  child.on("error", (err) => {
    if (isUpdateRelaunch && (err.code === "EBUSY" || err.code === "EACCES" || err.code === "EPERM")) {
      console.log("[*] Executable locked by OS. Retrying in 2 seconds...");
      setTimeout(() => launchApp(true, args), 2000);
      return;
    }
    console.error("Failed to start Matrix IPTV:", err);
    process.exit(1);
  });

  child.on("exit", async (code) => {
    if (code === updateExitCode) {
      try {
        await performUpdate({ relaunch: true, relaunchArgs: args });
      } catch (err) {
        console.error(`\nUpdate failed: ${err.message}`);
        console.error("The previous Matrix IPTV binary was left in place or restored.");
        process.exit(1);
      }
    } else {
      process.exit(code || 0);
    }
  });
}

const args = process.argv.slice(2);
if (args[0] === "update" || args[0] === "self-update" || args[0] === "--update") {
  const updateArgs = parseUpdateArgs(args.slice(1));
  performUpdate(updateArgs).catch((err) => {
    console.error(`\nUpdate failed: ${err.message}`);
    console.error("The previous Matrix IPTV binary was left in place or restored.");
    process.exit(1);
  });
} else {
  launchApp(false, args);
}
