# rask 技术设计文档

> rask — 用 Rust 编写的极简终端 AI 助手。

---

## 一、项目定位

轻量命令行 AI 对话工具，支持多 Provider、REPL 多轮对话和流式输出，设计原则是最小可用、逐步演进。

---

## 二、交互模式

### REPL 模式（默认）

```
$ rask
rask> 帮我写一个 Fibonacci
...（回答）...
rask> 解释一下上面的代码
...（多轮上下文保持）...
rask> exit
$
```

退出：输入 `exit` 或按 `Ctrl+D`。

**REPL 内置命令**（输入 `/` 弹出列表，上下键选择）：

| 命令 | 说明 |
|------|------|
| `/help` | 显示所有命令 |
| `/model <name>` | 切换模型，立即生效，上下文重置 |
| `/clear` | 清空当前对话上下文 |

**行编辑快捷键**：方向键移动光标，↑↓ 翻历史，Home/End 跳行首尾，Backspace/Delete 删字符。

### 单次模式（管道 / 脚本）

```bash
rask "问题"                        # 输出后直接退出
rask -m deepseek-chat "问题"       # 指定模型
echo "解释这段代码" | rask          # 管道输入
```

### 配置命令（REPL 外使用）

```bash
rask config set providers.deepseek.api_key sk-...
rask config set providers.openai.base_url https://my-proxy/v1
rask config set default_model deepseek-chat
rask config show
```

可配置的 key：

| Key | 说明 |
|-----|------|
| `default_model` | 默认模型 |
| `providers.<name>.api_key` | 指定 provider 的 API key |
| `providers.<name>.base_url` | 指定 provider 的端点（支持代理） |

---

## 三、Provider 设计

| Provider | 模型前缀 | 协议 |
|----------|---------|------|
| openai | 其他 | OpenAI |
| deepseek | `deepseek-*` | OpenAI 兼容 |
| glm | `glm-*` | OpenAI 兼容 |
| anthropic | `claude-*` | Anthropic |

DeepSeek / GLM 复用 OpenAI 兼容实现，仅 `base_url` 不同。模型名前缀自动路由 provider。

```rust
// core/src/client/mod.rs
#[async_trait]
pub trait AiClient: Send + Sync {
    async fn chat(&self, messages: &[Message]) -> Result<String>;
}
```

---

## 四、配置

**文件**：`~/.rask/config.toml`

```toml
default_model = "deepseek-chat"

[providers.openai]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"

[providers.anthropic]
api_key = "sk-ant-..."

[providers.deepseek]
api_key = "sk-..."
base_url = "https://api.deepseek.com/v1"

[providers.glm]
api_key = "..."
base_url = "https://open.bigmodel.cn/api/paas/v4"
```

优先级：命令行 `-m` > `config.toml`

---

## 五、本地存储

所有数据统一在 `~/.rask/`，路径管理集中在 `core/src/paths.rs`：

```
~/.rask/
├── config.toml      # 配置
├── history.jsonl    # 对话历史（每行一条 JSON）
└── sessions/        # 多会话存档（Phase 4）
```

---

## 六、项目结构

Workspace 两个 crate，目录短名，crate name 在各自 Cargo.toml 声明。

```
rask/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── cli/                    # crate: rask（binary）
│   │   └── src/
│   │       ├── main.rs         # 入口，路由单次/REPL/config 命令
│   │       ├── cli.rs          # clap 参数定义
│   │       └── repl.rs         # REPL 循环、spinner、once()
│   └── core/                   # crate: rask-core（lib）
│       └── src/
│           ├── lib.rs
│           ├── paths.rs        # 统一路径管理（~/.rask/）
│           ├── client/
│           │   ├── mod.rs      # AiClient trait + infer_provider()
│           │   ├── openai.rs   # OpenAI 兼容（含 DeepSeek/GLM）
│           │   └── anthropic.rs
│           ├── session.rs      # 消息上下文
│           ├── config.rs       # 配置读写、config set
│           ├── history.rs      # 历史持久化
│           └── error.rs        # RaskError
└── docs/
    └── design.md
```

---

## 七、核心数据结构

```rust
pub struct Message { pub role: String, pub content: String }

pub struct Session { pub model: String, pub messages: Vec<Message> }

pub struct Config {
    pub default_model: Option<String>,
    pub providers: HashMap<String, ProviderConfig>,
}

pub struct ProviderConfig { pub api_key: String, pub base_url: Option<String> }
```

---

## 八、依赖

```toml
# rask-core
tokio, reqwest, serde, serde_json, toml, dirs, thiserror, async-trait, futures-util

# rask (cli)
rask-core, clap, tokio, anyhow, rustyline, indicatif, toml
```

---

## 九、分阶段计划

| 阶段 | 目标 | 状态 |
|------|------|------|
| Phase 1 | 单次问答跑通 | ✓ 完成 |
| Phase 2 | 配置 + 错误处理 | ✓ 完成 |
| Phase 3 | REPL + /命令弹出 + 等待动画 | ✓ 完成 |
| Phase 4 | 流式输出 + `rask history` + 多会话管理 | 待开始 |
| Phase 5 | 发布 crates.io | 待开始 |

---

## 十、TODO：扩展能力规划

以下为规划中的能力，按需迭代，不作为 MVP 要求。

### Tool Use / Function Calling
AI Agent 核心能力。模型返回工具调用请求，rask 执行后将结果回传，形成 think → act → observe 循环。

涉及 Rust 知识：枚举、trait dispatch、async 递归、状态机。

### Skill 系统
类似 Claude Code 的 `/skill` 机制。用户可定义本地 skill（TOML/脚本），REPL 中 `/skill-name` 触发。

涉及 Rust 知识：动态加载、插件模式、proc-macro（进阶）。

### MCP（Model Context Protocol）
标准化工具协议，rask 作为 MCP client，接入任意 MCP server（文件系统、数据库、浏览器等）。

涉及 Rust 知识：JSON-RPC、async stream、进程间通信。

### 沙箱执行
执行代码类工具时隔离运行环境（容器 / WASM sandbox），防止副作用。

涉及 Rust 知识：subprocess、WASM runtime（wasmtime）、权限模型。

