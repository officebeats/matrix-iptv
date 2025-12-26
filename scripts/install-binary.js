const fs = require("fs");
const path = require("path");
const https = require("https");
const os = require("os");

const binaryName =
  os.platform() === "win32" ? "matrix-iptv.exe" : "matrix-iptv";
const binDir = path.join(__dirname, "..", "bin");
const binaryPath = path.join(binDir, binaryName);

const platformMap = {
  win32: "windows.exe",
  linux: "linux",
  darwin: "macos",
};

const archMap = {
  x64: "x64",
  arm64: "arm64",
};

const platform = platformMap[os.platform()];
if (!platform) {
  console.error(`Unsupported platform: ${os.platform()}`);
  process.exit(1);
}

// Note: Re-using the naming convention from install.ps1
// https://github.com/officebeats/matrix-iptv/releases/latest/download/matrix-iptv-windows.exe
const releaseUrl = `https://github.com/officebeats/matrix-iptv/releases/latest/download/matrix-iptv-${platform}`;

console.log(`[*] Matrix IPTV CLI // One-Click Install`);
console.log(`[*] Platform: ${os.platform()} (${os.arch()})`);
console.log(`[*] Downloading: ${releaseUrl}`);

if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Handle Redirect
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

download(releaseUrl, binaryPath)
  .then(() => {
    console.log(`[+] Download complete.`);
    if (os.platform() !== "win32") {
      fs.chmodSync(binaryPath, "755");
      console.log(`[+] Executable permissions set.`);
    }

    console.log(`\n✅  Matrix IPTV CLI installed successfully.`);

    // Auto-launch if running in an interactive terminal
    if (process.stdout.isTTY) {
      console.log(`🚀 Launching Matrix IPTV...`);
      const { spawn } = require("child_process");
      const child = spawn(binaryPath, [], { stdio: "inherit" });

      child.on("close", (code) => {
        process.exit(code);
      });
    } else {
      console.log(`Type 'matrix-iptv' to start.`);
    }
  })
  .catch((err) => {
    console.error(`\n❌ Installation failed: ${err.message}`);
    console.log(
      `Please ensure the GitHub repository is public and has a 'latest' release.`
    );
    process.exit(1);
  });
