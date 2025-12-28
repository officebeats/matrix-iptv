#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");
const os = require("os");
const fs = require("fs");
const https = require("https");

const binaryName =
  os.platform() === "win32" ? "matrix-iptv.exe" : "matrix-iptv";
const binaryPath = path.join(__dirname, binaryName);

const platformMap = {
  win32: "windows.exe",
  linux: "linux",
  darwin: "macos",
};

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          download(response.headers.location, dest).then(resolve).catch(reject);
          return;
        }
        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: ${response.statusCode}`));
          return;
        }
        response.pipe(file);
        file.on("finish", () => {
          file.close();
          resolve();
        });
      })
      .on("error", (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
  });
}

async function performUpdate() {
  const platform = platformMap[os.platform()];
  if (!platform) {
    throw new Error(`Unsupported platform for auto-update: ${os.platform()}`);
  }
  const releaseUrl = `https://github.com/officebeats/matrix-iptv/releases/latest/download/matrix-iptv-${platform}`;

  console.log(`\n[*] Initiating Phase 4: System Update...`);
  console.log(`[*] Downloading: ${releaseUrl}`);

  const tempPath = binaryPath + ".tmp";

  try {
    await download(releaseUrl, tempPath);

    if (os.platform() !== "win32") {
      fs.chmodSync(tempPath, "755");
    }

    // Replace old binary
    // On Windows, sometimes the OS still has a lock for a split second
    let attempts = 0;
    while (attempts < 5) {
      try {
        if (fs.existsSync(binaryPath)) {
          fs.unlinkSync(binaryPath);
        }
        fs.renameSync(tempPath, binaryPath);
        break;
      } catch (e) {
        attempts++;
        if (attempts === 5) throw e;
        await new Promise((r) => setTimeout(r, 500));
      }
    }

    console.log(`[+] Update complete. Rebooting system...\n`);
  } catch (err) {
    if (fs.existsSync(tempPath)) fs.unlinkSync(tempPath);
    throw err;
  }
}

function launchApp() {
  if (!fs.existsSync(binaryPath)) {
    console.error("\n❌  Matrix IPTV binary not found.");
    console.log(
      "Please try reinstalling the package: npm install -g @officebeats/matrix-iptv-cli\n"
    );
    process.exit(1);
  }

  const child = spawn(binaryPath, process.argv.slice(2), {
    stdio: "inherit",
    windowsHide: false,
  });

  child.on("error", (err) => {
    console.error("Failed to start Matrix IPTV:", err);
    process.exit(1);
  });

  child.on("exit", async (code) => {
    if (code === 42) {
      try {
        await performUpdate();
        launchApp(); // Relaunch
      } catch (err) {
        console.error(`\n❌ Update failed: ${err.message}`);
        process.exit(1);
      }
    } else {
      process.exit(code || 0);
    }
  });
}

launchApp();
