---
description: Build and Prepare for Testing
---

This workflow follows the user's strict requirement for project updates:

1. **Bump Version**: Update the version number in `Cargo.toml`.
2. **UI Verification**: Ensure the version number is correctly displayed in the UI (footer).
3. **Compile & Sync**: Run `.\test-matrix-iptv.bat`. This builds the release binary AND copies it to the `bin/` folder used by the Node wrapper.
4. **Notify**: Inform the user of the new version number before they test.

// turbo-all
// 1. Run the test/build script
.\test-matrix-iptv.bat
