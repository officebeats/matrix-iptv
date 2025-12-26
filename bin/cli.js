#!/usr/bin/env node

const { spawn } = require("child_process");
const path = require("path");
const os = require("os");
const fs = require("fs");

const binaryName =
  os.platform() === "win32" ? "matrix-iptv.exe" : "matrix-iptv";
const binaryPath = path.join(__dirname, binaryName);

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

child.on("exit", (code) => {
  process.exit(code || 0);
});
