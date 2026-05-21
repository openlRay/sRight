# sRight macOS Native Integration

This directory contains the FinderSync Extension source used by the local macOS integration.

The repository does not commit a generated Xcode project yet. Create it locally:

1. Open Xcode and create a macOS App project named `sRightNative`.
2. Add a FinderSync Extension target named `sRightFinderSync`.
3. Use bundle IDs like:
   - App: `dev.sright.native`
   - Extension: `dev.sright.native.findersync`
4. Copy `FinderSyncExtension/FinderSync.swift` into the extension target.
5. Use `FinderSyncExtension/Info.plist` as the extension target plist reference or copy the `NSExtension` keys into Xcode's generated plist.
6. Build `sright-cli` with `cargo build -p sright-cli`.
7. Put a CLI bridge at `~/Library/Application Support/sRight/sright-cli-debug.sh` or install `sright-cli` into `/opt/homebrew/bin` or `/usr/local/bin`.
8. Enable the Finder Extension in System Settings, then relaunch Finder if needed.

The extension reads `~/Library/Application Support/sRight/config.json`, builds enabled sRight
menu actions dynamically, collects Finder selected paths, asks for native confirmation for
configured dangerous actions, then invokes:

```sh
sright-cli action run --id <action-id> --path <selected-path>
```

For dangerous actions confirmed by the user, it also passes `--confirmed-dangerous`.

Phase 2 includes basic file actions. Templates, compression, image tools, translations, and
scripts are intentionally out of scope.
