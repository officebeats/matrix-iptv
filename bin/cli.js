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

const axios = require("axios");

async function download(url, dest) {
  const writer = fs.createWriteStream(dest);
  
  try {
    const response = await axios({
      method: 'get',
      url: url,
      responseType: 'stream',
      headers: {
        'Cache-Control': 'no-cache',
        'User-Agent': 'matrix-iptv-updater'
      }
    });

    response.data.pipe(writer);

    return new Promise((resolve, reject) => {
      writer.on('finish', resolve);
      writer.on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    });
  } catch (err) {
    fs.unlink(dest, () => {});
    if (err.response && err.response.status === 404) {
      throw new Error(`Asset not found (404). This usually means the GitHub release exists but the binary hasn't been uploaded yet.`);
    }
    throw err;
  }
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

  // Record old binary size for loop detection
  let oldSize = 0;
  try {
    oldSize = fs.statSync(binaryPath).size;
  } catch (e) {}

  try {
    await download(releaseUrl, tempPath);

    // Verify the downloaded file is not empty and is a valid size
    const newSize = fs.statSync(tempPath).size;
    if (newSize < 1024 * 100) { // Less than 100KB is almost certainly not a valid binary
      throw new Error(`Downloaded file is too small (${newSize} bytes). Update may have failed.`);
    }

    if (os.platform() !== "win32") {
      fs.chmodSync(tempPath, "755");

      // On macOS, remove quarantine flag to prevent Gatekeeper blocking
      if (os.platform() === "darwin") {
        try {
          const { execSync } = require("child_process");
          execSync(
            `xattr -d com.apple.quarantine "${tempPath}" 2>/dev/null || true`
          );
          console.log(`[+] macOS quarantine flag cleared.`);
        } catch (e) {
          // xattr might fail silently, that's okay
        }
      }
    }

    // Replace old binary
    let attempts = 0;
    const maxAttempts = 15;
    while (attempts < maxAttempts) {
      try {
        if (fs.existsSync(binaryPath)) {
          if (os.platform() === "win32") {
            const oldPath = binaryPath + ".old." + Date.now();
            fs.renameSync(binaryPath, oldPath);
            try {
              fs.unlinkSync(oldPath);
            } catch (e) {
              // Mark for deletion on next reboot or just ignore
            }
          } else {
            fs.unlinkSync(binaryPath);
          }
        }
        fs.renameSync(tempPath, binaryPath);
        break;
      } catch (e) {
        attempts++;
        if (attempts === maxAttempts) throw e;
        await new Promise((r) => setTimeout(r, 1000 + attempts * 500));
      }
    }

    if (os.platform() === "darwin") {
      try {
        const { execSync } = require("child_process");
        execSync(`xattr -d com.apple.quarantine "${binaryPath}" 2>/dev/null || true`);
      } catch (e) {}
    }

    // Loop detection: if the new binary is the exact same size as the old one,
    // the update likely downloaded the same version. Don't relaunch to avoid an infinite loop.
    if (oldSize > 0 && newSize === oldSize) {
      console.log(`[!] Update downloaded but binary size unchanged (${newSize} bytes).`);
      console.log(`[!] You may already be on the latest available binary. Skipping relaunch.`);
      process.exit(0);
    }

    console.log(`[+] Update complete. Rebooting system...\n`);

    if (os.platform() === "win32") {
      // Give Windows time to release filesystem locks on the new binary
      await new Promise((r) => setTimeout(r, 1500));

      const batchScript = `
@echo off
timeout /t 3 /nobreak > nul
start "" "${binaryPath}" %*
del "%~f0"
`;
      const batchPath = path.join(os.tmpdir(), "matrix-relaunch.bat");
      fs.writeFileSync(batchPath, batchScript);

      // Brief delay to let AV scanners release locks on newly written files
      await new Promise((r) => setTimeout(r, 500));

      let spawnAttempts = 0;
      const maxSpawnAttempts = 5;
      while (spawnAttempts < maxSpawnAttempts) {
        try {
          spawn("cmd.exe", ["/c", batchPath, ...process.argv.slice(2)], {
            detached: true,
            stdio: "ignore",
            windowsHide: true,
          }).unref();
          process.exit(0);
        } catch (spawnErr) {
          spawnAttempts++;
          if (spawnAttempts === maxSpawnAttempts) throw spawnErr;
          console.log(`[*] Relaunch blocked by OS. Retrying (${spawnAttempts}/${maxSpawnAttempts})...`);
          await new Promise((r) => setTimeout(r, 1000 + spawnAttempts * 500));
        }
      }
    }
  } catch (err) {
    if (fs.existsSync(tempPath)) fs.unlinkSync(tempPath);
    throw err;
  }
}

function launchApp(isUpdateRelaunch = false) {
  if (!fs.existsSync(binaryPath)) {
    console.error("\n❌  Matrix IPTV binary not found.");
    console.log(
      "Try one of these:\n  npx matrix-iptv\n  npm install -g matrix-iptv\n"
    );
    process.exit(1);
  }

  let child;
  try {
    child = spawn(binaryPath, process.argv.slice(2), {
      stdio: "inherit",
      windowsHide: false,
    });
  } catch (err) {
    if (isUpdateRelaunch && (err.code === "EBUSY" || err.code === "EACCES")) {
      console.log(`[*] Executable locked by OS. Retrying in 2 seconds...`);
      setTimeout(() => launchApp(true), 2000);
      return;
    }
    throw err;
  }

  child.on("error", (err) => {
    if (isUpdateRelaunch && (err.code === "EBUSY" || err.code === "EACCES")) {
      console.log(`[*] Executable locked by OS. Retrying in 2 seconds...`);
      setTimeout(() => launchApp(true), 2000);
      return;
    }
    console.error("Failed to start Matrix IPTV:", err);
    process.exit(1);
  });

  child.on("exit", async (code) => {
    if (code === 42) {
      try {
        await performUpdate();
        // If not win32 (which handles its own relaunch), relaunch here
        if (os.platform() !== "win32") {
          launchApp(true);
        }
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
