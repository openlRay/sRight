import Cocoa
import Darwin
import FinderSync
import UniformTypeIdentifiers

struct SRightConfig: Decodable {
    let enabled: Bool
    let show_icons: Bool?
    let dangerous_confirmation: DangerousConfirmationConfig?
    let file_templates: [FileTemplate]?
    let favorite_dirs: [FavoriteDirectory]?
    let menus: [MenuItem]
}

struct MenuItem: Decodable {
    let id: String
    let title: String
    let enabled: Bool
    let dangerous: Bool?
}

struct FileTemplate: Decodable {
    let id: String
    let title: String
    let file_name: String
    let enabled: Bool
}

struct FavoriteDirectory: Decodable {
    let id: String
    let title: String
    let path: String
    let enabled: Bool
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

        let enabledMenus = config.menus.filter(\.enabled)

        addNewFileSubmenu(to: menu, config: config, enabledMenus: enabledMenus)
        addFavoriteDirsSubmenu(to: menu, config: config, enabledMenus: enabledMenus)

        for menuItem in enabledMenus
            where !menuItem.id.hasPrefix("new_file.") && !menuItem.id.hasPrefix("favorite.open.")
        {
            let item = NSMenuItem(title: "sRight：\(menuItem.title)", action: #selector(runConfiguredAction(_:)), keyEquivalent: "")
            item.target = self
            item.isEnabled = true
            item.representedObject = menuItem.id
            menu.addItem(item)
        }
        guard !menu.items.isEmpty else {
            NSLog("sRight FinderSync has no enabled menu items. menus=\(config.menus.map(\.id))")
            return menu
        }
        NSLog("sRight FinderSync returning \(menu.items.count) menu item(s).")
        trace("menu returned itemCount=\(menu.items.count) ids=\(enabledMenus.map(\.id).joined(separator: ","))")

        return menu
    }

    private func addNewFileSubmenu(to menu: NSMenu, config: SRightConfig, enabledMenus: [MenuItem]) {
        let enabledMenuByID = Dictionary(uniqueKeysWithValues: enabledMenus.map { ($0.id, $0) })
        let templates = (config.file_templates ?? [])
            .filter(\.enabled)

        guard !templates.isEmpty else {
            return
        }

        let parent = NSMenuItem(title: "新建文件", action: nil, keyEquivalent: "")
        if config.show_icons ?? true {
            parent.image = menuIcon(forFileName: "Untitled.txt")
        }

        let submenu = NSMenu(title: "新建文件")
        for template in templates {
            let item = NSMenuItem(title: template.title, action: #selector(runConfiguredAction(_:)), keyEquivalent: "")
            item.target = self
            item.isEnabled = true
            item.representedObject = "new_file.\(template.id)"
            if config.show_icons ?? true {
                item.image = menuIcon(forFileName: template.file_name)
            }
            submenu.addItem(item)
        }

        parent.submenu = submenu
        menu.addItem(parent)

        for template in templates where enabledMenuByID["new_file.\(template.id)"] != nil {
            let item = NSMenuItem(title: "sRight：\(template.title)", action: #selector(runConfiguredAction(_:)), keyEquivalent: "")
            item.target = self
            item.isEnabled = true
            item.representedObject = "new_file.\(template.id)"
            if config.show_icons ?? true {
                item.image = menuIcon(forFileName: template.file_name)
            }
            menu.addItem(item)
        }
    }

    private func addFavoriteDirsSubmenu(to menu: NSMenu, config: SRightConfig, enabledMenus: [MenuItem]) {
        let enabledMenuIDs = Set(enabledMenus.map(\.id))
        let directories = (config.favorite_dirs ?? [])
            .filter { directory in
                directory.enabled && enabledMenuIDs.contains("favorite.open.\(directory.id)")
            }

        guard !directories.isEmpty else {
            return
        }

        let parent = NSMenuItem(title: "常用目录", action: nil, keyEquivalent: "")
        if config.show_icons ?? true {
            parent.image = NSWorkspace.shared.icon(forFile: NSHomeDirectory())
            parent.image?.size = NSSize(width: 20, height: 20)
        }

        let submenu = NSMenu(title: "常用目录")
        for directory in directories {
            let item = NSMenuItem(title: directory.title, action: #selector(runConfiguredAction(_:)), keyEquivalent: "")
            item.target = self
            item.isEnabled = true
            item.representedObject = "favorite.open.\(directory.id)"
            if config.show_icons ?? true {
                item.image = NSWorkspace.shared.icon(forFile: expandedPath(directory.path))
                item.image?.size = NSSize(width: 20, height: 20)
            }
            submenu.addItem(item)
        }

        parent.submenu = submenu
        menu.addItem(parent)
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
            trace("dangerous action queued without FinderSync modal actionID=\(actionID)")
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

        let title = item.title.replacingOccurrences(of: "sRight：", with: "")
        return loadConfig()?.menus.first(where: { $0.title == title })?.id
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
            NSLog("sRight FinderSync loaded config. enabled=\(config.enabled), menuCount=\(config.menus.count)")
            trace("config loaded enabled=\(config.enabled) menuCount=\(config.menus.count)")
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
