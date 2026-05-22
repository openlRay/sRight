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

enum SelectionContext {
    case noneSelected
    case onlyFiles(allImages: Bool)
    case onlyFolders
    case mixed(hasImage: Bool)
}

final class FinderSync: FIFinderSync {
    private let actionIDPrefix = "sright.action."
    private var menuActionIDsByTag: [Int: String] = [:]
    private var nextMenuItemTag = 1

    override init() {
        super.init()
        let directoryURLs = [realHomeDirectory()]
        FIFinderSyncController.default().directoryURLs = Set(directoryURLs)
        NSLog("sRight FinderSync initialized. directoryURLs=\(directoryURLs.map(\.path))")
    }

    override func menu(for menuKind: FIMenuKind) -> NSMenu? {
        let selectedURLs = FIFinderSyncController.default().selectedItemURLs() ?? []
        let selectedCount = selectedURLs.count
        let context = selectedContext(for: selectedURLs)
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

        menuActionIDsByTag.removeAll()
        nextMenuItemTag = 1
        renderMenuTree(config.menu_tree ?? [], to: menu, config: config, context: context, selectedURLs: selectedURLs)

        guard !menu.items.isEmpty else {
            NSLog("sRight FinderSync has no menu tree items.")
            return menu
        }
        NSLog("sRight FinderSync returning \(menu.items.count) menu item(s).")
        trace("menu returned itemCount=\(menu.items.count)")

        return menu
    }

    private func renderMenuTree(
        _ items: [MenuTreeItem],
        to menu: NSMenu,
        config: SRightConfig,
        context: SelectionContext,
        selectedURLs: [URL]
    ) {
        for item in items {
            renderMenuItem(item, to: menu, config: config, context: context, selectedURLs: selectedURLs)
        }
    }

    private func renderMenuItem(
        _ treeItem: MenuTreeItem,
        to menu: NSMenu,
        config: SRightConfig,
        context: SelectionContext,
        selectedURLs: [URL]
    ) {
        let children = treeItem.children ?? []
        if !children.isEmpty {
            let parent = NSMenuItem(title: treeItem.title, action: nil, keyEquivalent: "")
            applyIcon(treeItem.icon, to: parent, config: config)
            let submenu = NSMenu(title: treeItem.title)
            renderMenuTree(children, to: submenu, config: config, context: context, selectedURLs: selectedURLs)
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

        guard shouldRenderMenuItem(actionID, context: context, selectedURLs: selectedURLs) else {
            return
        }

        let item = NSMenuItem(title: treeItem.title, action: #selector(runConfiguredAction(_:)), keyEquivalent: "")
        let tag = nextMenuItemTag
        nextMenuItemTag += 1
        menuActionIDsByTag[tag] = actionID
        item.target = self
        item.isEnabled = true
        item.tag = tag
        item.representedObject = actionID
        item.identifier = NSUserInterfaceItemIdentifier(encodedActionID(actionID))
        item.toolTip = encodedActionID(actionID)
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
        let config = loadConfig()
        guard let item = menuItem(from: sender) else {
            NSLog("sRight FinderSync action missing menu item sender.")
            trace("action missing menu item sender=\(String(describing: sender))")
            return
        }

        guard let actionID = actionID(from: item, config: config) else {
            NSLog("sRight FinderSync action missing represented action id.")
            trace("action missing represented action id sender=\(String(describing: sender)) details=\(menuItemDebugDescription(item))")
            return
        }

        let selectedPaths = selectedActionPaths(for: actionID)
        trace("action entered actionID=\(actionID) selectedPaths=\(selectedPaths.joined(separator: "|"))")

        enqueueAction(actionID: actionID, selectedPaths: selectedPaths, confirmedDangerous: false)
    }

    private func selectedActionPaths(for actionID: String) -> [String] {
        let selectedURLs = FIFinderSyncController.default().selectedItemURLs() ?? []
        if selectedURLs.isEmpty,
           actionID.hasPrefix("new_file."),
           let targetURL = FIFinderSyncController.default().targetedURL() {
            return [targetURL.path]
        }

        return selectedURLs.map { $0.path }
    }

    private func selectedContext(for urls: [URL]) -> SelectionContext {
        guard !urls.isEmpty else {
            return .noneSelected
        }

        var fileCount = 0
        var folderCount = 0
        var imageCount = 0
        for url in urls {
            var isDirectory = ObjCBool(false)
            let exists = FileManager.default.fileExists(atPath: url.path, isDirectory: &isDirectory)
            if exists && isDirectory.boolValue {
                folderCount += 1
            } else {
                fileCount += 1
                if isImageFile(url) {
                    imageCount += 1
                }
            }
        }

        if fileCount > 0 && folderCount > 0 {
            return .mixed(hasImage: imageCount > 0)
        }
        if folderCount > 0 {
            return .onlyFolders
        }
        return .onlyFiles(allImages: fileCount > 0 && imageCount == fileCount)
    }

    private func shouldRenderMenuItem(
        _ actionID: String,
        context: SelectionContext,
        selectedURLs: [URL]
    ) -> Bool {
        if actionID.hasPrefix("favorite.open.") {
            return true
        }
        if actionID.hasPrefix("new_file.") {
            return true
        }

        switch context {
        case .noneSelected:
            return false
        case .onlyFiles(let allImages):
            return shouldRenderForOnlyFiles(actionID, allImages: allImages, selectedURLs: selectedURLs)
        case .onlyFolders:
            return shouldRenderForFolders(actionID)
        case .mixed:
            return shouldRenderForMixedSelection(actionID)
        }
    }

    private func shouldRenderForOnlyFiles(_ actionID: String, allImages: Bool, selectedURLs: [URL]) -> Bool {
        if actionID.hasPrefix("send.copy_to.") || actionID.hasPrefix("send.move_to.") {
            return true
        }
        if actionID.hasPrefix("image.") || actionID == "icon.make_iconset" || actionID == "icon.make_icns" {
            return allImages
        }
        if actionID == "archive.unzip_here" || actionID == "archive.unzip_to_folder" {
            return selectedURLs.allSatisfy(isZipFile)
        }
        if actionID == "folder.dissolve" {
            return false
        }

        return [
            "copy.path",
            "copy.name",
            "file.delete_permanently",
            "folder.create_from_filename",
            "file.info",
            "file.shortcut_desktop",
            "share.airdrop",
            "file.cut",
            "favorite.add_selected",
            "permission.grant_write",
            "visibility.unhide_all",
            "visibility.hide_all",
            "visibility.unhide_selected",
            "visibility.hide_selected",
            "finder.show_extensions",
            "finder.hide_extensions",
            "open.terminal",
            "open.vscode",
            "open.cursor",
            "archive.zip",
            "archive.zip_each",
            "tool.hash.md5",
            "tool.hash.sha1",
            "tool.hash.sha256",
            "tool.hash.sha512",
            "tool.qr.file",
            "tool.open_parent",
        ].contains(actionID)
    }

    private func shouldRenderForFolders(_ actionID: String) -> Bool {
        if actionID.hasPrefix("send.copy_to.") || actionID.hasPrefix("send.move_to.") {
            return true
        }

        return [
            "copy.path",
            "copy.name",
            "file.delete_permanently",
            "folder.dissolve",
            "file.info",
            "file.shortcut_desktop",
            "share.airdrop",
            "file.cut",
            "favorite.add_selected",
            "permission.grant_write",
            "visibility.unhide_all",
            "visibility.hide_all",
            "visibility.unhide_selected",
            "visibility.hide_selected",
            "open.terminal",
            "open.vscode",
            "open.cursor",
            "archive.zip",
            "archive.zip_each",
            "tool.open_parent",
        ].contains(actionID)
    }

    private func shouldRenderForMixedSelection(_ actionID: String) -> Bool {
        if actionID.hasPrefix("send.copy_to.") || actionID.hasPrefix("send.move_to.") {
            return true
        }

        return [
            "copy.path",
            "copy.name",
            "file.delete_permanently",
            "file.info",
            "file.shortcut_desktop",
            "share.airdrop",
            "file.cut",
            "favorite.add_selected",
            "permission.grant_write",
            "visibility.unhide_all",
            "visibility.hide_all",
            "visibility.unhide_selected",
            "visibility.hide_selected",
            "open.terminal",
            "open.vscode",
            "open.cursor",
            "archive.zip",
            "archive.zip_each",
            "tool.open_parent",
        ].contains(actionID)
    }

    private func isImageFile(_ url: URL) -> Bool {
        guard let type = UTType(filenameExtension: url.pathExtension.lowercased()) else {
            return false
        }

        return type.conforms(to: .image)
    }

    private func isZipFile(_ url: URL) -> Bool {
        url.pathExtension.lowercased() == "zip"
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
        process.arguments = ["-gj", "-b", "dev.sright.preferences", "--args", "--sright-background-action"]
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

    private func actionID(from item: NSMenuItem, config: SRightConfig?) -> String? {
        if let actionID = item.representedObject as? String, isKnownActionID(actionID, config: config) {
            return actionID
        }

        if let actionID = decodedActionID(from: item.identifier?.rawValue), isKnownActionID(actionID, config: config) {
            return actionID
        }

        if let actionID = decodedActionID(from: item.toolTip), isKnownActionID(actionID, config: config) {
            return actionID
        }

        if let actionID = menuActionIDsByTag[item.tag], isKnownActionID(actionID, config: config) {
            return actionID
        }

        return uniqueActionID(forTitle: item.title, config: config)
    }

    private func encodedActionID(_ actionID: String) -> String {
        "\(actionIDPrefix)\(actionID)"
    }

    private func decodedActionID(from value: String?) -> String? {
        guard let value, let actionID = value.stripPrefix(actionIDPrefix), !actionID.isEmpty else {
            return nil
        }

        return actionID
    }

    private func isKnownActionID(_ actionID: String, config: SRightConfig?) -> Bool {
        guard let config else {
            return false
        }

        return containsActionID(actionID, in: config.menu_tree ?? [])
    }

    private func containsActionID(_ actionID: String, in items: [MenuTreeItem]) -> Bool {
        for item in items {
            if item.action_id == actionID {
                return true
            }

            if containsActionID(actionID, in: item.children ?? []) {
                return true
            }
        }

        return false
    }

    private func uniqueActionID(forTitle title: String, config: SRightConfig?) -> String? {
        guard let config else {
            return nil
        }

        var matches: [String] = []
        collectActionIDs(forTitle: title, from: config.menu_tree ?? [], into: &matches)
        return matches.count == 1 ? matches[0] : nil
    }

    private func collectActionIDs(forTitle title: String, from items: [MenuTreeItem], into matches: inout [String]) {
        for item in items {
            if item.title == title, let actionID = item.action_id {
                matches.append(actionID)
            }

            collectActionIDs(forTitle: title, from: item.children ?? [], into: &matches)
        }
    }

    private func menuItemDebugDescription(_ item: NSMenuItem) -> String {
        let representedType = item.representedObject.map { String(describing: type(of: $0)) } ?? "nil"
        return "title=\(item.title) tag=\(item.tag) identifier=\(item.identifier?.rawValue ?? "nil") toolTip=\(item.toolTip ?? "nil") representedType=\(representedType)"
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
