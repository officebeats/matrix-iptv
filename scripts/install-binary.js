const { spawnSync } = require("child_process");
const crypto = require("crypto");
const fs = require("fs");
const os = require("os");
const path = require("path");
const axios = require("axios");

const repo = "officebeats/matrix-iptv";
const userAgent = "matrix-iptv-installer";
const packageVersion = require("../package.json").version;
const binaryName = os.platform() === "win32" ? "matrix-iptv.exe" : "matrix-iptv";
const binDir = path.join(__dirname, "..", "bin");
const binaryPath = path.join(binDir, binaryName);
const minBinarySize = 1024 * 100;

const platformMap = {
  win32: "windows.exe",
  linux: "linux",
  darwin: "macos",
};

function fail(message) {
  console.error(`\nInstallation failed: ${message}`);
  console.error("Please check the GitHub release assets or reinstall with npm install -g matrix-iptv.");
  process.exit(1);
}

function parseVersion(output) {
  const match = String(output || "").match(/(\d+\.\d+\.\d+)/);
  return match ? match[1] : null;
}

function sha256(filePath) {
  return crypto.createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

async function fetchRelease() {
  const response = await axios.get(
    `https://api.github.com/repos/${repo}/releases/tags/v${packageVersion}`,
    {
      timeout: 15000,
      headers: {
        Accept: "application/vnd.github+json",
        "Cache-Control": "no-cache",
        "User-Agent": userAgent,
      },
    }
  );

  return response.data;
}

function selectAsset(release) {
  const platform = platformMap[os.platform()];
  if (!platform) {
    throw new Error(`Unsupported platform: ${os.platform()}`);
  }

  const expectedName = `matrix-iptv-${platform}`;
  const asset = (release.assets || []).find((item) => item.name === expectedName);
  if (!asset) {
    throw new Error(`Release ${release.tag_name} does not include ${expectedName}.`);
  }

  return asset;
}

async function download(url, destination) {
  const response = await axios({
    method: "get",
    url,
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
    response.data.on("error", (err) => {
      writer.destroy(err);
    });
    writer.on("error", (err) => {
      fs.rm(destination, { force: true }, () => {});
      reject(err);
    });
  });
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

function verifyBinary(filePath, asset) {
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
    throw new Error(`Unable to run downloaded binary: ${result.error.message}`);
  }

  const version = parseVersion(`${result.stdout}\n${result.stderr}`);
  if (result.status !== 0 || version !== packageVersion) {
    throw new Error(`Downloaded binary reports version ${version || "unknown"}, expected ${packageVersion}.`);
  }
}

function replaceBinary(tempPath, asset) {
  const backupPath = fs.existsSync(binaryPath)
    ? `${binaryPath}.old-${Date.now()}${os.platform() === "win32" ? ".exe" : ""}`
    : null;
  let backupCreated = false;
  let replacementMoved = false;

  try {
    if (backupPath) {
      fs.renameSync(binaryPath, backupPath);
      backupCreated = true;
    }

    fs.renameSync(tempPath, binaryPath);
    replacementMoved = true;
    verifyBinary(binaryPath, asset);

    if (backupCreated) {
      fs.rmSync(backupPath, { force: true });
    }
  } catch (err) {
    if (replacementMoved) {
      fs.rmSync(binaryPath, { force: true });
    }
    if (backupCreated && fs.existsSync(backupPath)) {
      fs.renameSync(backupPath, binaryPath);
    }
    throw err;
  }
}

async function install() {
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  const tempPath = path.join(
    binDir,
    os.platform() === "win32"
      ? `matrix-iptv-${process.pid}-${Date.now()}.download.exe`
      : `matrix-iptv-${process.pid}-${Date.now()}.download`
  );

  console.log("[*] Matrix IPTV binary bootstrap");
  console.log(`[*] Platform: ${os.platform()} (${os.arch()})`);
  console.log(`[*] Package version: ${packageVersion}`);

  try {
    const release = await fetchRelease();
    const asset = selectAsset(release);
    console.log(`[*] Downloading ${asset.name} from ${release.tag_name}...`);
    await download(asset.browser_download_url, tempPath);
    verifyBinary(tempPath, asset);
    replaceBinary(tempPath, asset);
    console.log("[+] Matrix IPTV binary ready.");
    console.log("Type 'matrix-iptv' to start.");
  } catch (err) {
    fs.rmSync(tempPath, { force: true });
    fail(err.message);
  }
}

install();
