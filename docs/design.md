# sRight 设计文档

## 目标

sRight 是一个面向 macOS 的 Finder 右键增强工具，当前定位为本机自用工具，不面向 App Store 或官网公开发布。它的目标是对齐“超级右键”的主干能力，并在本地自定义、动作日志、菜单条件显示和脚本扩展方面做得更透明、更适合长期维护。

当前阶段使用本地开发签名，不处理 Developer ID 公证、App Store sandbox、自动更新和公开分发流程。

## 技术路线

项目采用 macOS 混合架构：

- `Tauri 2 + Vue 3 + TypeScript`：负责偏好设置界面。
- `Swift + FinderSync`：负责 Finder 右键菜单入口。
- `Rust`：负责核心动作能力和命令行执行器。
- 本地文件配置：默认放在 `~/Library/Application Support/sRight/`。

这样拆分的原因是：Finder 右键入口必须使用 macOS 原生扩展，而复杂偏好设置界面使用 Web 技术开发效率更高；动作执行单独放在 Rust 层，便于测试、日志记录和后续扩展。

## 通信模型

第一阶段采用简单可靠的文件与 CLI 通信模型：

1. Tauri 偏好设置 App 写入配置文件。
2. FinderSync 扩展读取配置文件并构建右键菜单。
3. FinderSync 扩展获取 Finder 当前选中的文件路径。
4. FinderSync 扩展调用 `sright-cli`，传入动作 ID 和文件路径。
5. `sright-cli` 通过 `sright-core` 执行动作。
6. `sright-cli` 写入结构化日志。
7. Tauri 偏好设置 App 读取日志并展示诊断信息。

后续如果文件通信无法满足实时性或稳定性要求，再升级为 App Group container、XPC 或 Distributed Notification。

## 模块划分

### `apps/desktop`

Tauri 偏好设置 App。

职责：

- 通用设置
- 菜单配置
- 新建文件模板管理
- 打开方式管理
- 发送到与常用目录管理
- 压缩解压设置
- 图片转换设置
- 工具箱设置
- 自定义脚本动作
- 日志与诊断
- Finder Extension 状态检测与权限引导

### `native/macos`

Swift macOS 原生集成层。

职责：

- FinderSync Extension
- Finder 右键菜单构建
- 选中文件 URL 收集
- 扩展基础诊断
- 调用 `sright-cli`

### `crates/sright-core`

Rust 核心能力库。

职责：

- 配置模型
- 动作模型
- 路径处理
- 文件操作
- 压缩与解压
- 图片转换
- 模板文件创建
- 日志模型
- 错误模型

### `crates/sright-cli`

Rust 命令行动作执行器。

职责：

- 解析 Finder 扩展传入的动作调用参数
- 加载配置
- 执行动作
- 写入结构化日志
- 提供终端调试命令

## 本地签名

当前只要求本地开发签名：

- 主 App 使用本地开发签名。
- FinderSync Extension 使用同一个 Team 的本地开发签名。
- 首次使用时需要在 Xcode 登录 Apple ID 并选择 Team。
- 首次运行后需要在系统设置中启用 Finder Extension。

当前不做：

- Developer ID 签名
- Notarization 公证
- App Store 分发
- 自动更新签名

## 工程仓库

后续使用 GitHub 仓库：

```text
git@github.com:openlRay/sRight.git
```

本地工程初始化后应配置该地址为 `origin`。

## 实现原则

- 优先保证 Finder 菜单入口稳定。
- 配置与动作模型先抽象清楚，再扩展具体动作。
- 危险动作必须支持二次确认或显式配置。
- 所有动作都应写入可诊断日志。
- 复杂能力可以分期实现，但需求文档保留完整能力边界。
- 不为公开发布提前引入不必要复杂度。
