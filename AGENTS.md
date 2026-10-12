# AGENTS.md

给所有 AI 编码助手（Claude Code / Codex / opencode / pi 等）的唯一入口：**规范、开发流程、项目进度、更新记录、讨论记录、踩坑记录**都在这一份里。
面向人类读者的技术说明仍在 `docs/`（`design/` 设计与决定、`plan/` 路线与待办、`notes/` 工程记录、`user/` 用户文档），`CLAUDE.md` 是 Claude Code 的入口，指向本文件与 `docs/contributing.md`。
本仓库是「札输入法」（单名「札」，英文名 **zhaime**，仓库 https://github.com/Titor-Z/zhaime）自己的代码：从上游青简输入法 fork 后大改，**不再跟随上游**，上游那套发布流程（prepare 门禁 / 草稿 Release / `releases.json` / 官网触发）已全部拆掉，别再照着它写。

各段落之间用锚点互链：`changelog` 的每条链到对应的 `taolun` 记录与「项目进度」条目，反过来也一样，便于以后溯源。

---

## changelog

本项目的更新记录，**时间倒序**（最新的在最上面），一次改动一条。每条要写清「改了什么、修了什么、影响谁」，并链到 `taolun` 里对应的讨论记录与「项目进度」里的条目。

<a id="c-2026-10-13-2"></a>
### C-2026-10-13-2 · 2026-10-13 · 重写 AGENTS.md，收编规范与记录

- 旧的 `AGENTS.md`（三行，只指向 `CLAUDE.md`）改名为 `AGENTS.md.old` 存档；本文件按「changelog / taolun / Agents / 认知修正」四段重写。
- 新增：AI 开发规范六条、项目进度三段（计划中 / 代办 / 已完成）、开发流程三条、踩坑记录。
- 影响：以后所有 agent 的改动都要同时更新本文件的「changelog」与「项目进度」，并在「taolun」里留讨论记录（见「开发流程」）。
- 讨论记录：[T-2026-10-13-2](#t-2026-10-13-2)
- 项目进度：[已完成 · 重写 AGENTS.md](#p-done-agents-md)

<a id="c-2026-10-13-1"></a>
### C-2026-10-13-1 · 2026-10-13 · 重写发版流程，修好 Windows / Linux 出包，发出 v0.3.0

- **修 Windows 出包**：上一轮重构发版流程时把 `apps/windows/installer/build.ps1` 换成了直接调 `ISCC.exe`，而编三个产物 + 32 位 TSF DLL + 挑自包含 Windows App Runtime + 读版本号 + 编 `.iss` 全是 `build.ps1` 的活，结果 Inno 去 `target\release\` 取不存在的文件，报 `qingjian_tsf.dll does not exist`。改用 `build.ps1`，toolchain 补回 `i686-pc-windows-msvc` 目标。
- **修 Linux 出包**：装的是 Ubuntu 上不存在的 `libfcitx5-dev`（apt 退出 100），runner 也从 `ubuntu-26.04` 掉到了 `24.04`（Fcitx5 插件链接的是构建机上的库，只能发 26.04）；依赖表按 `ci.yml` 改回，并补回打完包的冒烟测试（装进临时目录 + `ldd` 查缺库）。
- **`.github/workflows/release.yml` 重写**：三个平台各自 checkout → 版本门禁 → 拉数据 → 打包 → 上传到同一个 Release；触发保留 `v<版本>` / `<平台>-v<版本>` 标签，另加 `workflow_dispatch` 填标签名复用同一次流程（按标签取提交，不跟分支走）；上传加 `overwrite_files`，同一个标签可以重跑。
- **新增 `tools/release/version-check.sh`**：标签 → 版本号，并核对各平台壳的 `Cargo.toml` 与标签一致，不一致直接失败（版本号决定安装包文件名、Inno 版本信息与「关于」页显示，对不上就会发出名字与 Release 不符的包）。
- **版本**：三个壳与 `Cargo.lock` 从 `0.2.1-dev` 提到 `0.3.0`，`CHANGELOG.md` 补 `0.3.0` 一节；GitHub Release 标记为 `v0.3.0`。
- 讨论记录：[T-2026-10-13-1](#t-2026-10-13-1)
- 项目进度：[已完成 · 发版流程](#p-done-release)

---

## taolun

每次对话的内容，**时间正序**（最早的在最上面）。一次对话一条：用户提了什么、我做了什么、结论与遗留问题。写清「为什么这么定」，后来的 agent 不用重新问一遍。

<a id="t-2026-10-13-1"></a>
### T-2026-10-13-1 · 2026-10-13 · 从「Windows 发布失败」到发版流程重写

- **用户**：GitHub 上跑着 Release，Windows 版本的包报错了，让看看。
- **我**：拉 Actions 日志，定位到 windows job 在 Inno 编译时报 `apps/windows/installer/qingjian.iss: Source file "target\release\qingjian_tsf.dll" does not exist`；确认 windows job 里**没有任何编译步骤**（checkout → toolchain → cache → data-fetch → 装 Inno → 直接 `ISCC`），并且顺带发现 linux job 装的是不存在的 `libfcitx5-dev`、runner 从 26.04 掉到 24.04。根因是上一轮「自己写发版流程」时把 `build.ps1` 当成上游遗留一起删了。
- **用户**：明确不需要跟上游一致，「我们自己从 0 写自己的 GitHub Actions 流程」，并要把用户可见的名字逐步改成自己的（札 / zhaime）。
- **我**：整份重写 `release.yml`（三 job 并行、版本门禁、dispatch 可重跑），新增 `version-check.sh`；推送过程中连踩三个坑：脚本可执行位（100644 → 126）、macOS runner 的 bash 3.2 吞全角括号（`$version（` → unbound variable），逐条修掉。
- **结论**：发 `v0.3.0`；标签与壳版本必须一致这条门禁保留（它防止发出名字与 Release 不符的包）。Windows 与 Linux 已出包，macOS 待跑完确认。
- **遗留**：① 数据源还在上游仓库（见 [认知修正 · 数据源](#k-data)）；② 命名改造只做了「札」的文案部分，包名 / 二进制名 / crate 名 / 环境变量仍是 `qingjian`（见 [项目进度 · 代办](#p-todo)）；③ 版本号格式按新规范要改成 `YYYY.MM.DD.<commit>`，`version-check.sh` 与三个打包脚本还没跟上（见 [规范 4](#rule-version)）。

<a id="t-2026-10-13-2"></a>
### T-2026-10-13-2 · 2026-10-13 · 定下协作文档与规范

- **用户**：交代项目背景——原先是别人的「青简输入法」，从源码直接改，因为自己对性能和内存占用要求高、又不想从零写；现在功能已基本满足设想，但 README / CHANGELOG / 包名 / 仓库地址等还留着上游的痕迹。随后给出要求：旧的 `AGENTS.md` 改名存档，重写一份包含 `changelog` / `taolun` / `Agents` / `认知修正` 四段的规范文件，并给出六条开发规范、三段项目进度与三条开发流程。
- **我**：`git mv AGENTS.md AGENTS.md.old`，按格式重写本文件，并把上一轮讨论（[T-2026-10-13-1](#t-2026-10-13-1)）补录进来。
- **结论**：命名统一用 **zhaime**；对外链接统一指 `https://github.com/Titor-Z/zhaime`；数据源暂不搬家；版本号格式改为 `YYYY.MM.DD.<commitID>`（实现见「代办」）。
- **遗留**：规范 4（日期版本号）与现有发版流程冲突，需要改写 `version-check.sh` 与三个平台打包脚本的版本读取逻辑。

---

## Agents

给所有 AI 编码助手（Claude Code / Codex / opencode / pi …）的规范、进度与开发流程。**与代码约定冲突时以本段为准**，实现层面的细节仍看 `docs/contributing.md`。

### 规范

<a id="rule-1"></a>
1. **三次失败就停**：同一个问题连续三次没解决，立刻停止尝试，把「试了什么、看到什么错、卡在哪」详细汇报给用户，等用户解答再动。不要靠反复重试或换写法硬撞。
<a id="rule-2"></a>
2. **全程中文**：与本项目的所有对话、提交信息、代码注释、文档，以及 **agent 自己打印在终端里的思考过程（thinking）**，一律用中文。
<a id="rule-3"></a>
3. **注释要详细**：代码必须有详细的中文注释，写「为什么这么写、这里不能怎么改」，不写「这行在赋一个值」这类废话；装饰性分隔注释（`// ====`）不要。
<a id="rule-4"></a>
4. **版本号格式**：`YYYY.MM.DD.<commitID>`，`commitID` 取打 tag 前那个提交的短哈希，好溯源。例：`v2026.10.13.9c91957`。产品里显示的版本号与 git tag 用同一个串。
<a id="rule-5"></a>
5. **测试分文件**：要写测试就按功能模块拆成多个文件，**禁止**把所有测试塞进一个文件里。
<a id="rule-6"></a>
6. **面向对象**：开发采用 OOP 思路，一个类型一个文件，职责单一、高内聚低耦合；模块边界比「少写几个文件」重要。

### 项目进度

三段各自维护，条目都链到对应的 `changelog` 与 `taolun` 记录。技术细节与长线计划仍在 `docs/plan/roadmap.md`、`docs/plan/todo.md`。

<a id="p-plan"></a>
#### 计划中（未来可能做，暂不纳入代办）

- 命名改造的第三层：`qingjian-*` crate 名、`QINGJIAN_*` 环境变量、二进制名与安装/数据目录全部改成 `zhaime`（见 [T-2026-10-13-1](#t-2026-10-13-1) 的遗留项）。
- 把产品数据（143 MB 的 `qingjian-data.tar.gz`）搬到本仓库自己的 `data-vN` Release，彻底与上游脱钩（用户当前指示：先不做）。
- Windows 代码签名证书（有了就不用再关 `QINGJIAN_UIACCESS`，SmartScreen 也不再拦）。

<a id="p-todo"></a>
#### 代办（接下来要做的）

- [ ] 按 [规范 4](#rule-version) 把版本号改成 `YYYY.MM.DD.<commitID>`：改 `tools/release/version-check.sh`（不再与 `Cargo.toml` 的 semver 比对）、`apps/macos/scripts/bundle.sh`、`apps/windows/installer/build.ps1`、`apps/linux/scripts/package.sh` 四处读版本的地方，并处理 pkg / Inno 只认数字点号版本的约束。
- [ ] 命名改造第一层（纯文案与链接）：`README.md`、`CHANGELOG.md`、`docs/**`、`.github/ISSUE_TEMPLATE/**`、两个「关于」页的 `WEBSITE_URL` / `REPOSITORY_URL`、`Cargo.toml` 的 `repository` 字段，统一成「札 / zhaime / https://github.com/Titor-Z/zhaime」。
- [ ] 命名改造第二层（产物名与安装布局）：包名 `qingjian-*`、二进制 `qingjian-server.exe` / `qingjian_tsf*.dll` / `qingjian-settings.exe` / `qingjian-linux-server` / `qingjian.so`、目录 `%APPDATA%\Qingjian`、`%LOCALAPPDATA%\Qingjian\logs`、`C:\Program Files\Qingjian`、`%ProgramData%\Qingjian`；改之前先定老用户数据目录要不要兼容读一次。
- [ ] 同步 `docs/notes/release.md`：它还写着已删除的 `prepare` / `publish` / `releases_json.py` / 官网触发那套，与现在的 `release.yml` 不一致。

<a id="p-done-release"></a>
#### 已完成

- 2026-10-13 重写发版流程，Windows / Linux 恢复出包，发 `v0.3.0`（[C-2026-10-13-1](#c-2026-10-13-1) / [T-2026-10-13-1](#t-2026-10-13-1)）。
<a id="p-done-agents-md"></a>
- 2026-10-13 重写 `AGENTS.md` 并归档旧的（[C-2026-10-13-2](#c-2026-10-13-2) / [T-2026-10-13-2](#t-2026-10-13-2)）。

### 开发流程

<a id="flow-1"></a>
1. **先留记录，再动文件**：每次改代码前，先把讨论内容写进本文件的 `taolun`（时间正序追加），然后才开始改文件。改完再补 `changelog` 与「项目进度」。
<a id="flow-2"></a>
2. **用 bash，不用 PowerShell**：Windows 已内置 coreutils，`grep` / `ls` / `sed` / `find` / `seq` / `sleep` 等照 Linux 用法直接用，不要写 PowerShell 与 cmdlet。例外是仓库里已有的 `.ps1`（Windows 打包链路本身依赖它们）以及必须调 Windows 原生工具（`ISCC.exe`、`regsvr32`）的场合。
<a id="flow-3"></a>
3. **改完要闭环**：开发完成后更新「项目进度」与 `changelog`，并让 `changelog` 条目、`taolun` 条目、「项目进度」条目互相加链接，方便以后溯源。

---

## 认知修正

agent 在开发中踩过的坑、用户纠正过的事、明确过的偏好。**动手前先读这一段**，别重新掉进同一个坑。每条写「现象 → 原因 → 以后怎么做」。

<a id="k-iscc"></a>
- **`ISCC.exe` 不是打包的全部**（2026-10-13，Windows 发布失败）：直接调 Inno 编译器只会拿 `.iss` 里写死的路径去取文件，不会编 Rust 产物。Windows 出包的入口永远是 `apps/windows/installer/build.ps1`（编三个产物 + 32 位 DLL、挑自包含运行时、读版本、编 `.iss`），`release.yml` 只负责调它。
- **拆上游流程时先分清「上游的」和「必需的」**（2026-10-13）：上一轮重构把 `build.ps1` 当上游遗留删掉，实际上它是构建入口。删流程步骤前先把每一步「产出什么、被谁依赖」列清楚。
- **仓库里的 `.sh` 不要靠可执行位**（2026-10-13，退出码 126）：本机是 Windows，`chmod +x` 进不了 git index（记成 `100644`），Linux / macOS runner 上直接执行就报权限错。workflow 里一律用 `bash <脚本>`；需要真可执行位时用 `git update-index --chmod=+x`。
- **macOS runner 的 `/bin/bash` 是 3.2**（2026-10-13，`unbound variable`）：变量名后面紧跟全角字符（`$version（`、`$TAG）`）会被当成变量名的一部分。shell 脚本里的变量一律写成 `${var}`，尤其是后面接中文标点的地方。本机是 bash 5，测不出来。
- **包名以 `ci.yml` 为准**（2026-10-13，apt 退出 100）：Ubuntu 26.04 上 Fcitx5 的开发包是 `libfcitx5core-dev` / `libfcitx5config-dev` / `libfcitx5utils-dev` + `nlohmann-json3-dev`，没有 `libfcitx5-dev`；Fcitx5 插件链接构建机上的库，只能在 `ubuntu-26.04` 上打包。
- **Windows runner 上跑 `.sh` 要显式 `shell: bash`**（2026-10-13）：不写就用默认的 `pwsh`，脚本要么跑不起来、要么静默什么也没做。
<a id="k-data"></a>
- **产品数据仍来自上游仓库**（2026-10-13）：`tools/release/data-fetch.sh` 默认从 `qingjian-team/qingjian` 的 `data-v6` 拉 143 MB 产品数据（按 `data.lock` 里的 SHA-256 校验），本仓库至今没有自己的数据 Release。在搬家之前，打包机器需要能访问该仓库（公开可读）；用户已明确「先不做」，但要清楚这是与上游**唯一剩下的硬依赖**。
- **不要复刻上游流程**（2026-10-13，用户明确）：`prepare` 门禁、草稿 Release、`SHA256SUMS` / `build-info.json` 片段、`releases_json.py` / `releases.json` 签名、官网触发这一套都不做；发布就是「标签 + 三个平台并行打包 + 挂到同一个 Release」。新增流程前先问用户，不要默默把上游那套搬回来。
- **用户对命名的要求**（2026-10-13）：产品叫「札输入法」，单名「札」，英文名 **zhaime**，仓库 `https://github.com/Titor-Z/zhaime`；对外文案、包名、注释里的「青简 / qingjian / Qingjian」都要逐步清掉，不再引用上游域名与仓库地址。
- **本机的 `git commit` 会被钩子拖很久**（2026-10-13）：`.githooks/pre-commit` 会跑 `cargo fmt` + 全 workspace `cargo clippy`，本机 target 为空时第一次要十几分钟，容易被误判成卡死。要么耐心等（放后台跑、轮询日志），要么在**本次改动不含 `.rs`** 时用 `--no-verify`，并在汇报里说明。
