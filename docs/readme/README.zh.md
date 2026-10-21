<p align="center">
  <img src="../../images/query2table-logo.png" alt="Query2Table 标志" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <b>简体中文</b> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="GitHub 发布版本"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="GitHub 下载量"></a>
  <a href="#许可证"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="许可证：MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="基于 Tauri v2 构建"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Rust 后端"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Svelte 5 前端"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Query2Table 访问量">
</p>

**Query2Table** 是一款本地优先的研究工具，支持桌面端（Linux、Windows、macOS）和 Android。只需描述所需的数据，应用便会自动在互联网上跨多个来源、多种语言进行搜索，获取页面，借助 LLM 提取结构化实体，去除重复项，并以实时更新的表格呈现结果，每一行都附有来源链接。一切都在本机运行，无需云端后端。

除表格外，应用还提供三种专用模式：**图片搜索**，查找相关图片并由 LLM 按视觉相关性排序；**链接搜索**，读取候选页面的完整内容，只返回最相关的链接，并附上 LLM 生成的描述和相关性评分；以及**研究**，一种智能体模式，由模型自主搜索、阅读页面、推理，并撰写附有来源的 Markdown 回答。

应用同样提供 **Android** 手机版：同一款应用，具有相同的模式、设置和历史，由同一套代码构建，并针对手机屏幕做了适配，采用底部标签栏。

例如提问：*“生成一张开源向量搜索数据库和库的对比表，包括编程语言、许可证、部署方式、使用场景和网站”*，即可看到表格实时填充。

![开源向量数据库对比表，附有依据和来源](../../screenshots/test-en-2026-10-06/44-table-results.png)

## 功能一览

### 用自然语言提问

选择结果格式，描述需求，并设置停止条件。对于表格，应用会先建议列，你可以在搜索开始前进行编辑。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="新建查询，可选四种结果格式"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="待审核的建议表格结构"></td>
  </tr>
</table>

### 每一行都保留依据

打开一行即可查看完整的值、复制它们，并检查提取这些值的页面。

![行详情，包含提取置信度和来源](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### 研究与追问

智能体会搜索、阅读页面，并撰写带有目录、附有来源的回答。每个步骤都可在“活动”中查看，追问会基于历史继续对话。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="研究回答，包含目录和推荐的追问"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="研究活动，包含编号的搜索和阅读步骤"></td>
  </tr>
</table>

### 图片和链接

图片搜索按相关性对结果排序，并可按网站和尺寸筛选；链接搜索会阅读每个页面，并说明其匹配原因。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="以对齐网格显示的图片搜索结果"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="附有描述和匹配程度的相关链接"></td>
  </tr>
</table>

### 历史和设置

搜索、筛选并重新打开以往的运行，再次运行它们，或一次导出多个。开始之前，可在设置中检查 LLM 和搜索的连接。

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="按日期分组的运行历史，显示结果、时长、费用和模型"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="搜索设置，连接测试成功"></td>
  </tr>
</table>

### 浅色和深色主题

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="深色主题下的研究回答"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="深色主题下的结果表格"></td>
  </tr>
</table>

## 下载

从 [Releases](https://github.com/hightemp/query2table/releases/latest) 获取最新版本：

| 平台 | 文件 |
|------|------|
| Linux | `.AppImage`、`.deb`、`.rpm` |
| Windows | `.msi`、`.exe` |
| macOS（Apple Silicon） | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

如需将电脑上的配置迁移到手机，请使用 **设置 → 导出设置…**，勾选 **包含 API 密钥和代理**，然后在手机上导入该文件。此类文件包含你的密钥：请妥善保管，导入后将其删除。

在 Android 上，打开下载的 APK，并在提示时允许浏览器或文件管理器安装应用。Android 版应用将数据保存在手机上；它没有托盘，通知以常规 Android 通知的形式显示。本地模型服务器（Ollama、llama.cpp）可通过局域网地址使用，例如 `http://192.168.1.10:11434`。

## 功能

- **自然语言查询**：用日常语言描述需求
- **自动推断结构**：应用会建议表格列，你可以在执行前确认或编辑
- **多服务商搜索**：Brave Search + Serper，支持自动备用切换
- **多语言查询扩展**：跨语言、跨地区搜索
- **基于 LLM 的提取**：OpenRouter、Anthropic Claude（提示词缓存，可选完整读取 PDF）、Ollama（本地或 Cloud）、DeepSeek、Groq、Mistral、Gemini、xAI、OpenAI、LM Studio、llama.cpp，或任意兼容 OpenAI 的地址
- **流式结果**：找到的行会实时出现在表格中
- **PDF 提取**：自动识别搜索结果中的 PDF 文档并提取其文本
- **图片搜索模式**：专门搜索图片而非文本的流水线：生成 6–12 个不同的查询变体（由 LLM 生成或使用静态备用方案），通过 Brave / Serper 执行图片搜索，再由 LLM 按严格的相关性标准（0.0–1.0）为每个结果评分；低于 0.7 的图片会被丢弃，其余按评分排序保存，界面逐张实时更新；全程遵守预算、取消和停止条件（数量 / 费用 / 时间）
- **链接搜索模式**：返回最相关页面而非表格的专用流水线：根据请求生成搜索查询变体，通过 Brave / Serper 执行网络搜索（按 URL 去重），获取并解析每个页面，再由 LLM 阅读完整内容，评估其与查询的相关性（0.0–1.0）并生成简短描述；低于置信度阈值的页面会被丢弃，其余按评分排序保存，界面逐条实时更新；全程遵守预算、取消和停止条件
- **研究模式**：以 Markdown 而非表格回答开放式问题的智能体模式：LLM 驱动工具调用循环（搜索网络、以 Markdown 形式获取页面、思考、回答），步数有固定上限，每一步都实时显示在时间线上；推理（`think`）步骤和所有失败（`error` 步骤，以红色显示）都会展示出来，使过程保持透明；最终附有来源的 Markdown 回答会经过清理后渲染，并可导出为 `.md`；全程遵守预算、取消和停止条件
- **研究回答中的公式和图片**：渲染 LaTeX 公式（MathJax，离线）；智能体可以搜索图片、查看图片，并将合适的图片放入回答，随运行一起保存
- **研究回答中的图表**：Mermaid 图表以及柱状图、折线图、面积图、饼图、散点图、雷达图和热力图，按应用主题离线绘制；可保存为 SVG 或 PNG，也可全屏打开
- **用量统计**：按月、按年和全部时间统计费用、token、模型调用、Brave 和 Serper 搜索以及数据量，并按模型和模式细分，列出费用最高的运行和磁盘占用，支持 CSV 导出，以及带提醒的每月预算
- **行级来源**：每一行都链接回提取它的页面
- **实体去重**：模糊匹配 + LLM 辅助消歧
- **可配置的停止条件**：目标行数、最高费用、最长时间
- **运行历史**：浏览、查看并重新导出以往的研究运行
- **导出**：CSV、JSON、XLSX，包含完整的来源元数据
- **深色 / 浅色主题**：在侧边栏中切换
- **系统托盘**：完成通知（桌面端）
- **Android 应用**：手机上的同一款应用，针对小屏幕布局
- **应用内更新**：新版本发布时会显示更新内容，校验签名后可直接在应用中安装（桌面端和 Android）
- **本地优先**：所有数据保存在 SQLite 中，无需云端后端

## 技术栈

| 层 | 技术 |
|----|------|
| 应用外壳 | Tauri v2（桌面端和 Android） |
| 后端 | Rust（Tokio 异步运行时） |
| 前端 | Svelte 5（SvelteKit SPA） |
| 界面样式 | 使用设计令牌的纯 CSS |
| 数据库 | SQLite（sqlx，WAL 模式） |
| LLM | OpenRouter / Ollama（本地） / Ollama Cloud / 兼容 OpenAI 的 API（例如 llama.cpp） |
| 搜索 | Brave Search API / Serper API |
| 图标 | Lucide |

## 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install)（stable 工具链）
- 适用于你的操作系统的 [Tauri v2 环境要求](https://v2.tauri.app/start/prerequisites/)

## 快速开始

```bash
# Clone the repository
git clone https://github.com/hightemp/query2table.git
cd query2table

# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Tauri)
npm run tauri dev

# Build for production
npm run tauri build
```

## 配置

首次启动时，应用会创建一个使用默认设置的本地 SQLite 数据库。从侧边栏打开 **设置** 进行配置：

| 分组 | 设置 |
|------|------|
| **LLM 服务商** | 服务商（OpenRouter / Ollama / Ollama Cloud / 兼容 OpenAI），URL、模型、API 密钥、JSON 模式、温度、最大 token 数 |
| **搜索服务商** | 服务商（Brave / Serper），API 密钥、备用切换开关、每个查询的结果数 |
| **执行** | 并行获取数（默认 8）、并行提取数（默认 3）、获取超时、速率限制、robots.txt |
| **质量** | 精确率/召回率平衡、依据严格程度、置信度阈值、去重相似度 |

停止条件（目标行数、最高费用、最长时间）在“查询”页面中为每个查询单独设置。

**使用另一个服务商作为备用** 控制网络搜索和图片搜索在 Brave/Serper 之间的自动备用切换。关闭后只使用所选的主服务商，即使两个 API 密钥都已保存。保存后此选择应用于新的运行；重试仍使用所选的服务商。开启时，另一个服务商仅在失败后使用，且需要已保存其密钥。

在 **设置 → 搜索 → 每个查询的结果数** 中设置四种模式（包括研究）中每个搜索查询请求的结果数，然后保存，以用于新的运行。Brave 请求会将此值限制在文档规定的端点范围内：[网络搜索为 1–20](https://api-dashboard.search.brave.com/api-reference/web/search/get)，[图片最多 200](https://api-dashboard.search.brave.com/documentation/services/image-search)。之前保存的过大值也同样适用此限制。更大的值不会自动触发额外的结果页。

### 用量、价格和费用上限

**用量与费用** 会区分服务商报告的费用、按模型费率的估算、部分合计和未知费用。每次服务商尝试都会单独计数，包括重试、备用搜索、带有用量数据的失败请求，以及无法获得最终费用的已取消请求。输入/输出用量来自服务商的响应；思考 token 不会在输出 token 之外重复计费。

- **OpenRouter：** 优先使用[报告的请求费用](https://openrouter.ai/docs/cookbook/administration/usage-accounting)。如果不可用，则根据报告的 token 数和实际返回模型的公开目录费率进行估算。
- **Ollama Cloud：** 根据[公布的模型费率](https://ollama.com/pricing)估算用量额度消耗，包括缓存输入和可识别的高峰计价时段。包含的额度、订阅付款和实际扣费可能与此估算不同。无法识别或不可用的资费保持未知。
- **自定义模型费率：** 在设置中配置每百万 token 的输入/输出 USD 费率。费率对应特定的服务商、端点和模型。明确填写零表示免费；缺失的费率绝不会被静默当作零。获取公布的费率时不会更改已保存的手动费率。
- **搜索：** Brave 和 Serper 默认每 1,000 次请求 0 USD，即估算中不包含搜索费用。设置套餐的实际费率即可将其计入；如果未知，可将价格留空。没有计费数据的失败尝试保持未定价。设置的更改适用于新的运行；已保存的历史不会重新计算。升级时，以前为空的默认值会一次性变为零，同时保留已有的数值费率以及之后有意留空的值。

运行的 **最高费用** 会在计入的费用达到上限后阻止新的服务商请求。它并不保证账单上限：估算可能与实际计费不同，未知费用无法计入，已在进行中的请求仍可能产生费用。需要严格上限时，请使用服务商一侧的限制。无论运行完成、失败、取消还是没有结果，费用记录都会保留。较早的历史记录会明确标注为历史估算。

去重会保留第一个成员的行 ID，以事务方式合并该组的值和来源记录，并使实时表格与历史和导出保持一致。以前被覆盖的历史结果无法自动恢复；请重新运行这些查询以获取新结果。

设置在滚动时始终显示 **保存** 和 **放弃**。各部分链接可直接跳转到相应的控件；带着未保存的更改离开时，会提供“保存 / 放弃 / 留下”选项。保存过程中所做的修改会保留到下一次保存。

结果使用连续的虚拟化表格，支持本地搜索和键盘排序。打开某一行的 **详情** 可查看或复制完整的值，并检查其已保存的来源。研究会将回答放在可展开的活动之前。图片支持网格/列表视图和键盘预览；导出和删除会明确显示进度和结果。清除诊断用的 **日志** 面板不会重置运行活动。

浏览器回归测试使用合成数据和模拟的 Tauri 命令：先用 `npx playwright install chromium webkit` 安装测试浏览器，然后运行 `npm run test:ui`。现有的前端测试通过 `npm test` 运行。

### API 密钥

至少需要一个搜索 API 密钥和一个已配置的 LLM 服务商。本地 Ollama 和无需身份验证的兼容 OpenAI 的服务器不需要 LLM API 密钥：

| 服务 | 获取密钥 |
|------|----------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama（本地） | https://ollama.com/ — 无需密钥 |
| Ollama Cloud | https://ollama.com/settings/keys |
| 兼容 OpenAI | 视服务器而定，密钥可选 |

在 **设置 → LLM → 服务商** 中选择服务商，填写相应字段，然后点击 **保存**。切换服务商时，每个服务商都会保留各自的模型和凭据。

设置迁移会在启动时自动运行：新增的选项使用默认值，已有的值保持不变。重命名的设置会先在后端显式迁移，然后再插入默认值；实现新版本时，如果更改了存储格式，也需要编写迁移代码。

在 **设置** 底部，**应用文件** 显示实际的数据文件夹、`data.db` 数据库和日志文件夹。使用 **复制路径** 可将位置复制到系统剪贴板，使用 **打开文件夹** 可在文件管理器中打开。打开数据库位置时会打开其所在的文件夹。设置、历史、结果和来源保存在 `data.db` 中；诊断日志单独存储。

- **OpenRouter：** 输入 API 密钥，并从可搜索的 [OpenRouter 目录](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties) 中选择模型。输入文字可筛选模型 ID，用鼠标或方向键加 Enter 选择，使用 **刷新** 重新加载列表。输入密钥前即可浏览目录；推理时需要密钥。
- **Ollama（本地）：** URL 为 `http://localhost:11434`，并填写已安装模型的名称。
- **Ollama Cloud：** URL 为 `https://ollama.com`，填写 Ollama API 密钥，并从可搜索的服务器目录中选择模型。点击模型字段并输入文字进行筛选，然后用鼠标或方向键加 Enter 选择；**刷新** 会重新加载目录。在选择某个结果之前，筛选不会改变已选的模型。直接访问云端使用[原生 Ollama API](https://docs.ollama.com/cloud)。云端目前不支持[结构化输出](https://docs.ollama.com/capabilities/structured-outputs)，因此通过提示词请求 JSON，并由现有流水线解析。
- **兼容 OpenAI：** 输入包含 `/v1` 的 API 基础 URL 以及服务器的模型 ID。对于 [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server)，默认 URL 为 `http://localhost:8080/v1`；请使用已加载模型的名称或配置的别名。除非服务器要求，否则 API 密钥留空即可。如果服务器拒绝 `response_format`，请关闭 **JSON 模式**。

**思考 / 推理强度** 控制模型在回答前的推理。**自动** 保持安全的默认值：Ollama 的结构化请求会关闭思考（GPT-OSS 使用“低”），其他服务商保持各自的默认值。**服务商默认** 将推理交由服务器决定。**关**、**开**、**低**、**中**、**高** 和 **最高** 会请求明确的模式或级别；是否支持取决于模型和服务商。Ollama 上的 GPT-OSS 接受“低/中/高”，不能使用“关”或“最高”。保存后，更改将应用于新的运行。

**最大输出 token** 是每次请求的额度，可能由思考 token 和回答 token 共享。它与运行的费用/时间限制以及服务商账户配额相互独立。推理强度越高，留给回答的 token 可能越少。

模型请求出现问题时，会显示服务商、模型、阶段、请求的输出上限、已报告的输入/输出/思考 token 数，以及是否安排了重试。输出上限、上下文限制、服务商配额、速率限制、身份验证和网络故障都有各自的说明和解决方法。跳过个别页面时，问题仍然可见，并可在运行历史中查看。其他错误会显示简短说明、建议的操作以及可折叠的技术详情。

**暂停** 会在当前阶段（包括结构审核）挂起处理，**继续** 会接着执行而无需重新提交请求。已发送到远程模型的请求在暂停期间仍可能在远端完成。**取消** 会停止本地工作并关闭待处理的请求。流水线失败会显示并保存为 `failed`，底层错误可在运行日志中查看。

## 架构

后端使用 **流水线状态机**，按顺序编排固定的角色：

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### 流水线角色

| 角色 | LLM | 用途 |
|------|-----|------|
| QueryInterpreter | 是 | 将查询解析为结构化意图 |
| SchemaPlanner | 是 | 建议表格列及其类型 |
| SearchPlanner | 是 | 为多种语言/地区生成搜索查询 |
| QueryExpander | 是 | 将查询翻译为目标语言 |
| SearchExecutor | 否 | 调用搜索 API，收集候选 URL |
| Fetcher | 否 | 带速率限制和 robots.txt 的 HTTP 获取 |
| DocumentParser | 否 | HTML → 纯净文本（去除样板内容） |
| PdfParser | 否 | PDF → 纯净文本（通过 pdf-extract） |
| Extractor | 是 | 文本 + 结构 → 结构化行 |
| Validator | 部分 | 结构一致性 + 语义检查 |
| Deduplicator | 部分 | 模糊匹配（strsim）+ LLM 处理边界情况 |
| LinkRanker | 是 | 阅读页面内容并评估相关性（0.0–1.0）+ 生成描述（链接搜索模式） |
| ResearchAgent | 是 | 驱动搜索/获取/思考/回答的智能体循环，生成 Markdown 回答（研究模式） |
| StoppingController | 否 | 评估停止条件（行数、预算、时间、饱和度） |

### 状态机

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## 项目结构

```
query2table/
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs          # Tauri entry point
│   │   ├── commands/        # IPC command handlers
│   │   ├── orchestrator/    # Pipeline state machine, budget tracker
│   │   ├── roles/           # Pipeline & research-agent roles
│   │   ├── providers/       # External API clients (LLM, search, HTTP)
│   │   ├── storage/         # SQLite models & repository
│   │   └── export/          # CSV, JSON, XLSX export
│   └── migrations/          # SQLite schema migrations
├── src/                     # Svelte frontend
│   ├── lib/
│   │   ├── components/      # UI components
│   │   ├── stores/          # Svelte stores (run, settings, ui, logs)
│   │   ├── types/           # TypeScript type definitions
│   │   └── api/             # Tauri invoke/listen wrappers
│   └── routes/              # Pages (query, history, settings)
├── package.json
└── src-tauri/Cargo.toml
```

## 数据库

使用 WAL 模式的 SQLite，包含以下表：

| 表 | 用途 |
|----|------|
| `settings` | 用户配置（API 密钥、模型、偏好） |
| `runs` | 研究运行记录，包括状态和统计 |
| `run_schemas` | 每个运行已确认的表格结构 |
| `search_queries` | 每个运行生成的搜索查询 |
| `search_results` | 从搜索 API 收集的 URL |
| `fetched_pages` | 下载的页面内容 |
| `entity_rows` | 提取出的结构化行（JSON 数据） |
| `row_sources` | 行级依据（URL、标题、摘录） |
| `image_results` | 每个运行排序后的图片（图片搜索模式） |
| `link_results` | 排序后的相关链接，附描述和评分（链接搜索模式） |
| `research_steps` | 每个运行的智能体工具调用步骤：search/fetch/think/error（研究模式） |
| `research_results` | 每个运行的最终 Markdown 回答（研究模式） |
| `run_logs` | 每个运行的执行日志 |

## 开发

```bash
# Frontend dev server only
npm run dev

# Type checking
npm run check

# Run tests
npm test

# Run Rust tests
cd src-tauri && cargo test

# Lint
npm run lint
```

### Android

需要满足 [Tauri Android 环境要求](https://v2.tauri.app/start/prerequisites/#android)：Android SDK 和 NDK（`ANDROID_HOME`、`NDK_HOME`）、Java 17（`JAVA_HOME`），以及 Rust 目标平台（`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`）。连接已开启 USB 调试的手机，或启动模拟器。

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

除非存在 `src-tauri/gen/android/keystore.properties`，否则构建使用调试密钥签名。`make android-keystore` 会创建发布密钥和该文件，并输出发布工作流用于为每个版本附带的 APK 签名的 GitHub secrets（`ANDROID_KEYSTORE_BASE64`、`ANDROID_KEY_PASSWORD`、`ANDROID_KEY_ALIAS`）。请妥善保存该密钥：只有使用相同密钥签名的更新才能覆盖安装已有的应用。

## 导出格式

| 格式 | 内容 |
|------|------|
| **CSV** | 所有列 + 以 JSON 列表示的来源 |
| **JSON** | 完整的行，包含嵌套的来源数组和运行元数据 |
| **XLSX** | 格式化的工作簿，包含数据工作表、来源工作表和元数据工作表 |

## 许可证

MIT
