# 阶段0：代码基线审计与CLI-only重构落地方案

本文档覆盖现有代码基线（android-proxy-setter）的GUI与ADB关键路径、问题清单与修复建议，并给出CLI-only的模块化设计与命令规划。同时明确兼容与范围边界，以及变更风险与回滚策略。


## 一、代码与资源全量审阅结论

- 主要源码
  - src/main.rs（~590行）：集成了CLI交互、直接命令旗标、GUI 托盘模式全部逻辑；ADB 调用与错误处理散落其间。
  - src/bin/resurrection_adb.sh：通过 pgrep/kill -9 重启 adb 的 Bash 脚本，运行方式为 include_str + `sh -c`。
- Cargo 配置与依赖
  - Cargo.toml
    - 依赖：local-ip-address、colored、anyhow、clap、winit、tray-icon、image、once_cell、rfd。
      - once_cell 目前在代码中未使用（src/main.rs 内注释“改为普通全局变量而不是 OnceCell”），属于库依赖遗留。
    - [package.metadata.bundle]：应用打包元数据（macOS bundle），指定图标 resources/icon.png。
    - 版本不一致：package.version = 0.1.2，但 bundle.version = 0.1.1。
  - 仓库根目录存在二进制包：Android Proxy Setter.dmg（应纳入 .gitignore 或移出仓库）。
- 资源与打包
  - resources/icon.png：GUI 托盘图标，代码通过 include_bytes 嵌入。
  - 使用 cargo-bundle 相关元数据进行 macOS 打包。


## 二、全部 UI 相关代码路径清单

- 依赖（Cargo.toml）
  - winit = "0.28"（事件循环）
  - tray-icon = "0.5"（系统托盘）
  - image = "0.24"（PNG 解码）
  - rfd = "0.11"（对话框）
  - [package.metadata.bundle]（打包/Bundle 配置，含 icon 指向 resources/icon.png）
- 源码（src/main.rs）
  - GUI imports：image、rfd::MessageDialog、tray_icon::{…}、winit::event_loop::{…}
  - 静态全局：`static mut TRAY_ICON: Option<Box<dyn std::any::Any>> = None;`（为防止托盘被 drop）
  - run_gui_mode(..)：完整 GUI 模式
    - 创建 winit EventLoop
    - 构建 tray-icon Menu（Set/Clear/View/Restart ADB/Quit）
    - include_bytes!("../resources/icon.png") + image 解码为 RGBA 并设置托盘图标
    - MenuEvent::receiver() -> 子线程转发事件到 mpsc::channel
    - 事件处理：spawn 后台线程执行 set_proxy/clear_proxy/get_proxy_info/restart_adb，并使用 rfd 弹窗提示
  - GUI 对话框函数：show_info_dialog(..)、show_error_dialog(..)
- 资源
  - resources/icon.png（托盘图标，PNG，嵌入式加载）
- 打包配置
  - Cargo.toml [package.metadata.bundle]（name、identifier、icon、osx_minimum_system_version 等）


## 三、ADB 关键路径与错误处理分支梳理

- 进程检测与自启
  - 程序启动：`pgrep adb` 判断 adb 是否运行；若无，调用 restart_adb()。
    - 问题：pgrep 非跨平台（Windows 不可用），且未检查 pgrep 的退出码，仅根据 stdout 解析 PID 列表。
- ADB 可用性检测
  - `adb version`：检查 status.success()，失败直接 bail。
- 设备检测
  - `adb devices`：解析输出，判断是否有“连接的设备”（简单地过滤第一行与空行）。
    - 问题：未区分 device/offline/unauthorized；多设备未选择；后续未使用 -s <serial> 指定设备。
- 读取当前代理
  - `adb shell settings get global http_proxy`：读取 stdout 得到 current_proxy_setting。
    - 问题：该处未检测退出码，失败场景可能被忽略（例如多设备/无设备/权限问题）。
- 设置代理（set_proxy）
  - 先清理：`settings put global http_proxy :0`（失败仅打印 warning，继续）
  - 设置：`settings put global http_proxy <ip:port>`（失败 bail）
  - 验证：`settings get global http_proxy`（失败仅提示“无法验证”）
  - 固定 sleep 500ms 等待，未有超时或重试机制。
- 清理代理（clear_proxy）
  - `settings put global http_proxy :0`（失败 bail）
  - 验证并提示。
- 查看代理（view_proxy / get_proxy_info）
  - 同样通过 `settings get global http_proxy` 获取并解析。
- 重启 ADB（restart_adb）
  - include_str!("./bin/resurrection_adb.sh") + `sh -c <script>` 执行
  - 脚本：pgrep 找出 adb PID -> kill -9 -> adb start-server
  - 仅打印成功/失败，但 Rust 端总是返回 Ok(())（即失败不向上传递错误）。


## 四、问题清单与优先级（含建议修复）

P0（必须修复，影响正确性/稳定性/安全性）
- 多设备未处理
  - 现状：`adb devices` 仅计数，不区分状态，后续 adb shell 命令未加 `-s <serial>`。
  - 风险：多设备/模拟器并存时报错“more than one device/emulator”；命令失败但程序未必能正确识别。
  - 建议：
    - 抽象 adb 层 list_devices()，解析出 (serial, state)；仅对 state==device 的设备执行。
    - CLI 提供 --serial/-s 指定；若只有一个 device 状态设备则默认选取；若多个则报错并列出；可提供 `android-proxy-setter devices` 子命令列出选择。
- ADB 命令错误处理与超时
  - 现状：大量 `.output()` 无超时；部分未检查 `status.success()`；stderr 解析不统一。
  - 风险：命令挂起、无响应、返回非 0 未被识别，导致假成功或阻塞。
  - 建议：
    - 统一封装 adb 调用，提供超时（如 wait-timeout crate 或异步超时），统一返回 Result<Output>，同时捕获并上抛 stdout/stderr。
    - 严格检查退出码；将 stderr 作为 anyhow 上下文。
- 重启 ADB 的跨平台与安全性
  - 现状：Bash 脚本 + pgrep + kill -9，仅类 Unix 可用，且 kill -9 粗暴。
  - 建议：
    - 优先使用 `adb kill-server && adb start-server` 作为“软重启”（跨平台）。
    - 若需要“硬重启”，按平台分别实现（但默认不建议强杀）。
    - Rust 侧根据 status 真实返回 Ok/Err。
- 设置键一致性与兼容
  - 现状：仅使用 `global http_proxy`（字符串 `<host>:<port>`）。
  - 风险：某些 ROM/版本仅认 `global_http_proxy_host`/`global_http_proxy_port` 或需要同时设置/清空。
  - 建议：
    - set：同时设置以下键，失败回滚：
      - settings put global http_proxy <host:port>
      - settings put global global_http_proxy_host <host>
      - settings put global global_http_proxy_port <port>
      - 可选：global_http_proxy_exclusion_list（保留/清空策略一致）
    - clear：同时清空上述所有键（http_proxy=>:0；host/port 删除或置空）。
    - verify：至少校验 `http_proxy` 与 host/port 两套值一致性。
- CLI 语义冲突
  - 现状：--set 与 --clear 同时传入时，代码按 if/else 优先 set，未显式冲突。
  - 建议：
    - 改为 Subcommand（set/clear/show/restart/devices），杜绝互斥旗标带来的歧义。

P1（较重要，提升鲁棒性与可维护性）
- 跨平台可用性
  - 现状：pgrep、sh、bash 脚本、GUI 依赖在 Windows 下不可用或体验差。
  - 建议：CLI-only 版本移除 GUI 依赖；adb 交互/重启逻辑采用跨平台实现；检测 ANDROID_HOME/platform-tools；支持自定义 adb 路径。
- 输入校验与默认值
  - 现状：port 未做范围校验（仅类型 u16）；ip 允许自动探测但未校验格式/网卡可达性。
  - 建议：
    - validate_port(1..=65535)；validate_host(支持 IPv4/IPv6/主机名)；
    - 自动探测本机 IP 时选择对外可达网卡，必要时允许 --iface 指定。
- 设备状态解析
  - 现状：未剖析 offline/unauthorized；
  - 建议：list_devices() 返回 Device{ serial, state }，state 枚举 { Device, Offline, Unauthorized, Unknown(String) }，仅允许 Device。
- 失败回滚策略
  - 现状：set 时先清空，后设置，失败后不会恢复原值。
  - 建议：set 前读取原值；设置任一步失败则恢复原值并提示。
- 日志与可观测性
  - 现状：println + colored；缺少统一日志级别/结构化输出。
  - 建议：引入 env_logger 或 tracing，支持 --verbose/-v、--quiet；支持 --json 输出用于脚本集成。
- 交互停顿
  - 现状：固定 sleep 500ms；
  - 建议：对 settings 读回值增加重试（退避、最大时限）。

P2（优化/整理）
- 代码结构
  - 现状：单文件（main.rs）承载全部逻辑；GUI/CLI/ADB 混杂；TRAY_ICON 使用 `static mut`。
  - 建议：迁移至模块化（见下文 CLI-only 设计），彻底删除 GUI 相关代码与依赖；移除 once_cell 依赖。
- 包版本与打包元数据不一致
  - 现状：0.1.2 vs 0.1.1；
  - 建议：统一版本来源；CI 侧校验。
- 仓库工件
  - 现状：DMG 文件纳入仓库；
  - 建议：添加到 .gitignore 或发布到 Release，不入源码仓库。


## 五、CLI-only 模块化设计与接口定义

目标：彻底移除 GUI 相关依赖（winit/tray-icon/rfd/image/Bundle 配置），仅保留可靠的 CLI 工具，并形成清晰的分层：cli -> proxy(service) -> adb(adapter)；验证与日志为横切关注点。

建议的文件结构：
- src/main.rs：入口，只做 CLI 解析和分发
- src/cli.rs：clap 子命令/参数定义与解析
- src/adb.rs：与 adb 的进程交互封装（命令构建、-s 绑定、超时、stdout/stderr 归一化）
- src/proxy.rs：领域逻辑（读取/设置/清理/验证 Android 全局 HTTP 代理）
- src/validate.rs：输入校验（host/port/iface/serial/格式）
- src/logging.rs：日志初始化（env_logger 或 tracing），支持 verbose/quiet/json

核心类型与接口（示例草案）：
- adb.rs
  - struct Adb { bin: PathBuf, timeout: Duration }
  - enum DeviceState { Device, Offline, Unauthorized, Unknown(String) }
  - struct Device { serial: String, state: DeviceState }
  - impl Adb {
    - fn detect() -> Result<Self>; // 从 PATH/ANDROID_HOME/platform-tools/ADB 环境变量探测
    - fn version(&self) -> Result<String>;
    - fn list_devices(&self) -> Result<Vec<Device>>;
    - fn shell(&self, serial: Option<&str>, args: &[&str]) -> Result<Output>; // 统一超时与错误包装
    - fn settings_get_global(&self, serial: Option<&str>, key: &str) -> Result<String>;
    - fn settings_put_global(&self, serial: Option<&str>, key: &str, value: &str) -> Result<()>;
    - fn kill_server(&self) -> Result<()>; // adb kill-server
    - fn start_server(&self) -> Result<()>; // adb start-server
  }
- proxy.rs
  - struct ProxyConfig { host: String, port: u16, exclusion_list: Option<String> }
  - fn read(adb: &Adb, serial: Option<&str>) -> Result<ProxyConfig>; // 从 http_proxy 与 host/port 两套键合并读取
  - fn set(adb: &Adb, serial: Option<&str>, cfg: &ProxyConfig) -> Result<()>; // 先备份，再设置 http_proxy & host/port & 可选 exclusion_list，最后验证
  - fn clear(adb: &Adb, serial: Option<&str>) -> Result<()>; // 同步清空两套键
  - fn verify(adb: &Adb, serial: Option<&str>, expected: &ProxyConfig, timeout: Duration) -> Result<()>; // 带重试
- validate.rs
  - fn validate_host(s: &str) -> Result<String>;
  - fn validate_port(p: u16) -> Result<u16>;
  - fn detect_local_ip(prefer_iface: Option<&str>) -> Result<String>;
- cli.rs（clap Subcommands）
  - android-proxy-setter set [--host <HOST>] [--port <PORT>] [--serial <SERIAL>] [--exclusion <LIST>] [--timeout <ms>] [--json]
  - android-proxy-setter clear [--serial <SERIAL>] [--timeout <ms>] [--json]
  - android-proxy-setter show [--serial <SERIAL>] [--json]
  - android-proxy-setter devices [--json] // 列出 serial 与状态
  - android-proxy-setter restart-adb [--method soft|hard] [--json] // 默认 soft: kill-server/start-server
  - 全局：--adb-path <PATH>、-v/--verbose、-q/--quiet
- logging.rs
  - fn init(verbosity: Verbosity, json: bool)

行为与返回码约定：
- 返回码 0 成功；非 0 则失败，stderr 输出明确可解析信息。
- --json 输出结构化 JSON，便于脚本集成（如 CI/管控平台）。


## 六、命令与参数规划（CLI-only）

- set
  - 参数：--host/--ip（默认自动探测）、--port（必填或默认 8083）、--serial、--exclusion、--timeout、--json
  - 行为：
    1) 前置检查 adb/version；2) 选定设备（只有一个则默认；多个未指定报错并列出）；
    3) 读取原配置；4) 执行设置（两套键）；5) 验证（重试）；6) 失败则回滚；7) 文本或 JSON 输出。
- clear
  - 参数：--serial、--timeout、--json
  - 行为：清空两套键，验证并报告。
- show
  - 参数：--serial、--json
  - 行为：读取 http_proxy 与 host/port，输出统一视图。
- devices
  - 行为：列出所有设备及状态，明确 device/offline/unauthorized；默认仅当存在 1 个 device 状态设备时，其他命令可省略 --serial。
- restart-adb
  - 参数：--method soft|hard（默认 soft）；--json
  - 行为：soft=kill-server/start-server；hard 为平台化强制措施（后续再实现/慎用）。


## 七、兼容性与范围确认

- 功能范围
  - 仅支持“系统全局 HTTP 代理”（非 PAC，非鉴权）。
  - 兼容旧键：同时操作 http_proxy 与 global_http_proxy_host/port（以及可选 exclusion_list）。
- 平台范围
  - Windows/macOS/Linux 均可运行（CLI-only）；
  - 依赖 adb 可执行（PATH、ANDROID_HOME/platform-tools 或 --adb-path 指定）。
  - 不再引入 GUI/系统托盘相关依赖，以避免桌面环境差异与发布复杂度。


## 八、变更风险与回滚策略

- 变更风险
  - 从单体 main.rs + GUI 混合逻辑迁移至模块化 CLI-only，涉及较大规模目录与代码移动；
  - ADB 调用从“直呼 .output()”改为封装+超时可能引入边界差异；
  - 同时设置/清空两套键会改变旧版本行为（但属于增强兼容）。
- 缓解策略
  - 分阶段推进：
    1) Phase 0（当前）：审计与方案（本文件）。
    2) Phase 1：引入新模块骨架（adb.rs/proxy.rs/cli.rs/validate.rs/logging.rs），保留旧 CLI 行为但内部走新实现；新增隐藏开关回退到旧实现用于对照测试。
    3) Phase 2：移除 GUI 依赖与代码，默认启用新实现；
    4) Phase 3：完善 JSON 输出、设备选择 UX、更多校验与回滚逻辑。
  - 增加集成测试：模拟
    - 无设备/多设备/离线/未授权；
    - 设置/清空/读取/验证失败；
    - 超时与 stderr 提示；
  - 发布前在三平台做冒烟测试。
- 回滚策略
  - 新实现通过 feature flag（或环境变量）保留旧路径回退；
  - 若出现严重问题，可切换到旧实现发布紧急版本，同时保留 CLI-only 代码以便后续修复。


## 九、后续实施要点清单（给开发）

- 移除 GUI 依赖：winit/tray-icon/rfd/image；删除 GUI 相关代码与资源加载；移除 bundle 元数据或保留仅 CLI 打包策略（可选）。
- 新建模块骨架与公共错误类型（anyhow 即可），集中实现 adb 封装与超时。
- 子命令化 CLI；添加 --serial/--adb-path/--json/--timeout/--verbose。
- 设置键双写/双清；read/verify 逻辑覆盖两套键；
- 设备状态解析与提示优化；
- 统一日志与输出风格（文本/JSON）。
- 清理依赖与仓库工件（once_cell、DMG 文件等）。


## 十、现有代码问题-到-修复建议映射（速览）

- static mut TRAY_ICON 使用不安全全局变量 -> 移除 GUI，问题消失；若保留 GUI，应使用 OnceCell 或生命周期持有者，而非 unsafe。
- Arc<Mutex<ProxyState>> 用于 GUI 菜单状态 -> CLI-only 无此共享状态；CLI 直接以参数传递。
- 线程/回调遗留 -> CLI-only 无需要；命令串行执行。
- adb 错误处理（无超时/未检查退出码/未解析 stderr） -> 统一 adb 封装 + 超时 + 规范错误包装。
- 多设备未处理 -> --serial + list_devices + 默认选择逻辑 + 错误提示。
- 跨平台问题（pgrep、bash 脚本、sh -c） -> 使用 adb kill-server/start-server；按平台扩展“硬重启”可选实现。
- settings 键不一致 -> set/clear 时 http_proxy 与 global_http_proxy_host/port 同步维护；可加 exclusion_list。
- 版本不一致（Cargo.toml） -> 统一版本号，CI 校验。
- 仓库包含 DMG 工件 -> .gitignore 管理，发布 Release artifacts。


—— 以上审计结论与设计方案可直接指导后续 Phase 1/2 的实现工作，并将验收目标限定为：
- CLI-only、跨平台、具备明确的子命令与参数、可靠的错误处理与设备选择逻辑；
- 对 http_proxy 与 host/port 两套键具备一致性维护与验证。
