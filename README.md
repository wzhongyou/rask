# rask

终端 AI 对话工具，用 Rust 编写。

## 安装

```bash
cargo install --path crates/cli
```

## 配置

配置文件位于 `~/.rask/config.toml`，首次安装后填入 API key：

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

也可以通过命令管理：

```bash
rask config set providers.deepseek.api_key sk-...
rask config set default_model deepseek-chat
rask config show
```

## 使用

```bash
rask                              # 进入 REPL，多轮对话
rask "帮我写一个 Fibonacci"        # 单次问答，输出后退出
rask -m claude-3-5-sonnet "问题"  # 指定模型（单次）
echo "解释这段代码" | rask         # 管道输入
```

### REPL 内置命令

REPL 模式下输入 `/` 会弹出命令列表，上下键选择，Enter 确认：

| 命令 | 说明 |
|------|------|
| `/help` | 显示所有命令 |
| `/model <name>` | 切换模型（立即生效，上下文重置） |
| `/clear` | 清空当前对话上下文 |
| `exit` | 退出（或 Ctrl+D） |

## 支持的 Provider

| 模型前缀 | Provider | 默认端点 |
|---------|----------|---------|
| `claude-*` | Anthropic | api.anthropic.com |
| `deepseek-*` | DeepSeek | api.deepseek.com |
| `glm-*` | 智谱 GLM | open.bigmodel.cn |
| 其他 | OpenAI | api.openai.com |

DeepSeek / GLM 兼容 OpenAI 协议，自定义端点通过 `base_url` 配置。

## 本地存储

```
~/.rask/
├── config.toml      # 配置
├── history.jsonl    # 对话历史
└── sessions/        # 会话存档（后续版本）
```
