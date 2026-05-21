import Cocoa
import Darwin
import FinderSync
import UniformTypeIdentifiers

struct SRightConfig: Decodable {
    let enabled: Bool
    let show_icons: Bool?
    let dangerous_confirmation: DangerousConfirmationConfig?
    let menu_tree: [MenuTreeItem]?
}

struct MenuTreeItem: Decodable {
    let title: String
    let action_id: String?
    let icon: String?
    let children: [MenuTreeItem]?
}

struct PendingFinderAction: Encodable {
    let request_id: String
    let action_id: String
    let paths: [String]
    let confirmed_dangerous: Bool
}

struct DangerousConfirmationConfig: Decodable {
    let enabled: Bool
    let action_ids: [String]

    func requiresConfirmation(for actionID: String) -> Bool {
        enabled && action_ids.contains(actionID)
    }
}

final class FinderSync: FIFinderSync {
    override init() {
        super.init()
        let directoryURLs = [realHomeDirectory()]
        FIFinderSyncController.default().directoryURLs = Set(directoryURLs)
        NSLog("sRight FinderSync initialized. directoryURLs=\(directoryURLs.map(\.path))")
    }

    override func menu(for menuKind: FIMenuKind) -> NSMenu? {
        let selectedCount = FIFinderSyncController.default().selectedItemURLs()?.count ?? 0
        NSLog("sRight FinderSync menu requested. kind=\(menuKind.rawValue), selectedCount=\(selectedCount)")
        trace("menu requested kind=\(menuKind.rawValue) selectedCount=\(selectedCount)")

        let menu = NSMenu(title: "sRight")

        guard let config = loadConfig() else {
            NSLog("sRight FinderSync config missing. Returning disabled diagnostic menu item.")
            let item = NSMenuItem(title: "sRight：未找到配置", action: nil, keyEquivalent: "")
            item.isEnabled = false
            menu.addItem(item)
            return menu
        }

        guard config.enabled else {
            NSLog("sRight FinderSync config disabled. Returning empty menu.")
            return menu
        }

        renderMenuTree(config.menu_tree ?? [], to: menu, config: config)

        guard !menu.items.isEmpty else {
            NSLog("sRight FinderSync has no menu tree items.")
            return menu
        }
        NSLog("sRight FinderSync returning \(menu.items.count) menu item(s).")
        trace("menu returned itemCount=\(menu.items.count)")

        return menu
    }

    private func renderMenuTree(_ items: [MenuTreeItem], to menu: NSMenu, config: SRightConfig) {
        for item in items {
            renderMenuItem(item, to: menu, config: config)
        }
    }

    private func renderMenuItem(_ treeItem: MenuTreeItem, to menu: NSMenu, config: SRightConfig) {
        let children = treeItem.children ?? []
        if !children.isEmpty {
            let parent = NSMenuItem(title: treeItem.title, action: nil, keyEquivalent: "")
            applyIcon(treeItem.icon, to: parent, config: config)
            let submenu = NSMenu(title: treeItem.title)
            renderMenuTree(children, to: submenu, config: config)
            guard !submenu.items.isEmpty else {
                return
            }

            menu.addItem(parent)
            menu.setSubmenu(submenu, for: parent)
            return
        }

        guard let actionID = treeItem.action_id else {
            return
        }

        let item = NSMenuItem(title: treeItem.title, action: #selector(runConfiguredAction(_:)), keyEquivalent: "")
        item.target = self
        item.isEnabled = true
        item.representedObject = actionID
        applyIcon(treeItem.icon, to: item, config: config)
        menu.addItem(item)
    }

    private func applyIcon(_ icon: String?, to item: NSMenuItem, config: SRightConfig) {
        guard config.show_icons ?? true, let icon else {
            return
        }

        if icon == "home" {
            item.image = NSWorkspace.shared.icon(forFile: NSHomeDirectory())
        } else if let fileName = icon.stripPrefix("file:") {
            item.image = menuIcon(forFileName: fileName)
        } else if let path = icon.stripPrefix("path:") {
            item.image = NSWorkspace.shared.icon(forFile: expandedPath(path))
        } else if let symbolName = icon.stripPrefix("system:") {
            item.image = NSImage(systemSymbolName: symbolName, accessibilityDescription: nil)
        }

        item.image?.size = NSSize(width: 20, height: 20)
    }

    private func menuIcon(forFileName fileName: String) -> NSImage {
        let fileExtension = URL(fileURLWithPath: fileName).pathExtension
        let contentType = fileExtension.isEmpty
            ? UTType.data
            : (UTType(filenameExtension: fileExtension) ?? .data)
        let icon = NSWorkspace.shared.icon(for: contentType)
        icon.size = NSSize(width: 20, height: 20)
        return icon
    }

    @objc func runConfiguredAction(_ sender: Any?) {
        guard let item = menuItem(from: sender), let actionID = actionID(from: item) else {
            NSLog("sRight FinderSync action missing represented action id.")
            trace("action missing represented action id sender=\(String(describing: sender))")
            return
        }

        let selectedPaths = FIFinderSyncController.default()
            .selectedItemURLs()?
            .map(\.path) ?? []
        trace("action entered actionID=\(actionID) selectedPaths=\(selectedPaths.joined(separator: "|"))")

        let config = loadConfig()
        let requiresConfirmation = config?.dangerous_confirmation?.requiresConfirmation(for: actionID) ?? false
        if requiresConfirmation {
            guard confirmDangerousAction(title: item.title, selectedCount: selectedPaths.count) else {
                trace("dangerous action cancelled actionID=\(actionID)")
                return
            }
            trace("dangerous action confirmed actionID=\(actionID)")
        }

        enqueueAction(actionID: actionID, selectedPaths: selectedPaths, confirmedDangerous: requiresConfirmation)
    }

    private func enqueueAction(actionID: String, selectedPaths: [String], confirmedDangerous: Bool) {
        let requestID = UUID().uuidString.lowercased()
        do {
            let pendingDirectory = pendingActionsDirectory()
            try FileManager.default.createDirectory(at: pendingDirectory, withIntermediateDirectories: true)
            let request = PendingFinderAction(
                request_id: requestID,
                action_id: actionID,
                paths: selectedPaths,
                confirmed_dangerous: confirmedDangerous
            )
            let data = try JSONEncoder().encode(request)
            let temporaryURL = pendingDirectory.appendingPathComponent("\(requestID).tmp")
            let finalURL = pendingDirectory.appendingPathComponent("\(requestID).json")
            try data.write(to: temporaryURL, options: .atomic)
            try FileManager.default.moveItem(at: temporaryURL, to: finalURL)
            trace("queued action requestID=\(requestID) actionID=\(actionID) selectedPaths=\(selectedPaths.joined(separator: "|"))")
        } catch {
            trace("queue action failed actionID=\(actionID) error=\(error.localizedDescription)")
            return
        }

        wakeMainApp(requestID: requestID, actionID: actionID)
    }

    private func wakeMainApp(requestID: String, actionID: String) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        process.arguments = ["-gj", "-b", "dev.sright.preferences"]
        let stderr = Pipe()
        process.standardError = stderr
        trace("wake app executable=/usr/bin/open arguments=\(process.arguments?.joined(separator: " ") ?? "")")

        do {
            try process.run()
            process.waitUntilExit()
            if process.terminationStatus != 0 {
                trace("wake app failed requestID=\(requestID) actionID=\(actionID) status=\(process.terminationStatus) stderr=\(readPipe(stderr))")
            }
        } catch {
            trace("wake app threw requestID=\(requestID) actionID=\(actionID) error=\(error.localizedDescription)")
        }
    }

    private func menuItem(from sender: Any?) -> NSMenuItem? {
        if let item = sender as? NSMenuItem {
            return item
        }

        let mirror = Mirror(reflecting: sender as Any)
        if mirror.displayStyle == .optional, let child = mirror.children.first?.value as? NSMenuItem {
            return child
        }

        return nil
    }

    private func actionID(from item: NSMenuItem) -> String? {
        if let actionID = item.representedObject as? String {
            return actionID
        }

        return nil
    }

    private func expandedPath(_ path: String) -> String {
        if path == "~" {
            return realHomeDirectory().path
        }

        if path.hasPrefix("~/") {
            let rest = String(path.dropFirst(2))
            return realHomeDirectory().appendingPathComponent(rest).path
        }

        return path
    }

    private func confirmDangerousAction(title: String, selectedCount: Int) -> Bool {
        let alert = NSAlert()
        alert.alertStyle = .warning
        alert.messageText = "确认执行危险动作？"
        alert.informativeText = "\(title) 将作用于 \(selectedCount) 个选中项。"
        alert.addButton(withTitle: "确认")
        alert.addButton(withTitle: "取消")
        return alert.runModal() == .alertFirstButtonReturn
    }

    private func loadConfig() -> SRightConfig? {
        let url = appSupportDirectory().appendingPathComponent("config.json")
        let data: Data

        do {
            data = try Data(contentsOf: url)
        } catch {
            NSLog("sRight FinderSync failed to read config at \(url.path): \(error.localizedDescription)")
            trace("config read failed path=\(url.path) error=\(error.localizedDescription)")
            return nil
        }

        do {
            let config = try JSONDecoder().decode(SRightConfig.self, from: data)
            NSLog("sRight FinderSync loaded config. enabled=\(config.enabled), menuTreeCount=\(config.menu_tree?.count ?? 0)")
            trace("config loaded enabled=\(config.enabled) menuTreeCount=\(config.menu_tree?.count ?? 0)")
            return config
        } catch {
            NSLog("sRight FinderSync failed to decode config at \(url.path): \(error.localizedDescription)")
            trace("config decode failed path=\(url.path) error=\(error.localizedDescription)")
            return nil
        }
    }

    private func findCLIExecutable() -> URL? {
        let environment = ProcessInfo.processInfo.environment
        let candidates = [
            environment["SRIGHT_CLI_PATH"],
            bundledCLIExecutable()?.path,
            appSupportDirectory().appendingPathComponent("sright-cli-debug.sh").path,
            "/opt/homebrew/bin/sright-cli",
            "/usr/local/bin/sright-cli"
        ].compactMap { $0 }

        return candidates
            .map(URL.init(fileURLWithPath:))
            .first { FileManager.default.isExecutableFile(atPath: $0.path) }
    }

    private func bundledCLIExecutable() -> URL? {
        let bundleURL = Bundle.main.bundleURL
        let appContentsURL = bundleURL
            .deletingLastPathComponent()
            .deletingLastPathComponent()

        return appContentsURL
            .appendingPathComponent("MacOS")
            .appendingPathComponent("sright-cli")
    }

    private func appSupportDirectory() -> URL {
        if let override = ProcessInfo.processInfo.environment["SRIGHT_APP_SUPPORT_DIR"] {
            return URL(fileURLWithPath: override)
        }

        return URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
            .appendingPathComponent("Library")
            .appendingPathComponent("Application Support")
            .appendingPathComponent("sRight")
    }

    private func pendingActionsDirectory() -> URL {
        appSupportDirectory().appendingPathComponent("pending-actions", isDirectory: true)
    }

    private func trace(_ message: String) {
        let url = appSupportDirectory().appendingPathComponent("finder-sync-trace.log")
        let line = "\(Date()) \(message)\n"
        guard let data = line.data(using: .utf8) else {
            return
        }

        do {
            try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            if FileManager.default.fileExists(atPath: url.path) {
                let handle = try FileHandle(forWritingTo: url)
                try handle.seekToEnd()
                try handle.write(contentsOf: data)
                try handle.close()
            } else {
                try data.write(to: url)
            }
        } catch {
            NSLog("sRight FinderSync trace failed: \(error.localizedDescription)")
        }
    }

    private func readPipe(_ pipe: Pipe) -> String {
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        guard let text = String(data: data, encoding: .utf8) else {
            return ""
        }
        return text.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func realHomeDirectory() -> URL {
        if let passwd = getpwuid(getuid()), let directory = passwd.pointee.pw_dir {
            return URL(fileURLWithPath: String(cString: directory), isDirectory: true)
        }

        return URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
    }
}

private extension String {
    func stripPrefix(_ prefix: String) -> String? {
        guard hasPrefix(prefix) else {
            return nil
        }

        return String(dropFirst(prefix.count))
    }
}
