# AI Gateway Hub

AI Gateway Hub 是一个服务器端的 AI 网关管理工具，让您可以在网页上统一管理各种 AI CLI 工具的配置（Claude Code、Codex、Gemini CLI、OpenCode、OpenClaw 等）。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)
![React](https://img.shields.io/badge/React-18+-61DAFB.svg)

## ✨ 功能特性

### 已实现
- ✅ **Provider 管理** - 管理 OpenAI、Anthropic、Gemini、Azure、Ollama 等 AI 供应商配置
- ✅ **CLI 工具配置** - 统一管理 Claude Code、Codex、Gemini CLI、OpenCode、OpenClaw 等工具配置
- ✅ **一键切换** - 在网页上一键切换不同 AI 供应商，自动同步到本地 CLI 工具
- ✅ **配置备份/恢复** - 自动备份原有配置，随时可恢复
- ✅ **配置文件生成** - 自动生成各 CLI 工具的配置文件

### 待实现
- 📋 Dashboard - 使用统计面板
- 📋 MCP Servers - Model Context Protocol 服务器管理
- 📋 Proxy - AI 请求代理服务
- 📋 Settings - 系统设置

## 🚀 支持的 CLI 工具

| 工具 | 配置文件路径 | 状态 |
|------|-------------|------|
| Claude Code | `~/.claude/settings.json` | ✅ 支持 |
| Codex | `~/.codex/config.json` | ✅ 支持 |
| Gemini CLI | `~/.gemini/config.json` | ✅ 支持 |
| OpenCode | `~/.opencode/config.json` | ✅ 支持 |
| OpenClaw | `~/.openclaw/config.json` | ✅ 支持 |

## 🐳 Docker 一键部署（推荐）

### 方式一：使用 Docker Compose（推荐）

```bash
# 克隆项目
git clone https://github.com/yourusername/ai-gateway-hub.git
cd ai-gateway-hub

# 启动服务
docker-compose up -d

# 访问 http://localhost:8677
```

### 方式二：使用 Docker 命令

```bash
# 构建镜像
docker build -t ai-gateway-hub .

# 运行容器
docker run -d \
  --name ai-gateway-hub \
  -p 8080:8080 \
  -v ai-gateway-data:/data \
  ai-gateway-hub

# 访问 http://localhost:8677
```

### 停止服务

```bash
docker-compose down

# 同时删除数据卷
docker-compose down -v
```

## 🛠️ 手动部署

### 环境要求
- Rust 1.75+
- Node.js 18+
- SQLite

### 后端部署

```bash
cd backend
cargo build --release
cargo run
# 服务运行在 http://localhost:3000
```

### 前端部署

```bash
cd frontend
npm install
npm run dev
# 开发服务器运行在 http://localhost:5173
```

## 📖 使用指南

### 1. 添加 AI 供应商

进入 **Providers** 页面，点击 "Add Provider" 添加您的 AI 供应商配置：
- OpenAI、Anthropic、Gemini 等
- 填写 API Key 和 Base URL
- 配置支持的模型列表

### 2. 配置 CLI 工具

进入 **CLI Tools** 页面：
1. 点击 "Add Configuration"
2. 选择 CLI 工具类型（如 Claude Code）
3. 选择对应的 Provider（自动填充 API 配置）
4. 填写模型名称（如 `claude-3-opus-20240229`）
5. 保存配置

### 3. 应用配置

在 CLI Tools 卡片上：
- 点击 📥 **Backup** - 备份当前本地配置
- 点击 ✓ **Apply** - 将配置写入本地文件
- 点击 ↩️ **Restore** - 恢复之前的备份

## 📁 项目结构

```
ai-gateway-hub/
├── backend/          # Rust 后端 (Axum + SQLite)
│   ├── src/
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/         # React 前端 (Vite + Tailwind)
│   ├── src/
│   ├── package.json
│   └── Dockerfile
├── docker-compose.yml
├── Dockerfile        # 多阶段构建
└── README.md
```

## 🔌 API 文档

### Providers API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/api/providers` | 获取供应商列表 |
| POST | `/api/providers` | 创建供应商 |
| PUT | `/api/providers/:id` | 更新供应商 |
| DELETE | `/api/providers/:id` | 删除供应商 |
| POST | `/api/providers/:id/default` | 设为默认 |

### CLI Tools API

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/api/cli-tools` | 获取 CLI 工具配置列表 |
| POST | `/api/cli-tools` | 创建配置 |
| PUT | `/api/cli-tools/:id` | 更新配置 |
| DELETE | `/api/cli-tools/:id` | 删除配置 |
| POST | `/api/cli-tools/:id/apply` | 应用到本地 |
| POST | `/api/cli-tools/:id/backup` | 备份当前配置 |
| POST | `/api/cli-tools/:id/restore` | 恢复备份 |

## ⚙️ 配置说明

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `PORT` | 后端服务端口 | `3000` |
| `PROXY_PORT` | 代理服务端口 | `8080` |
| `ENABLE_PROXY` | 是否启用代理 | `true` |
| `RUST_LOG` | 日志级别 | `info` |

### 数据存储

- **数据库**: `~/.local/share/ai-gateway/ai-gateway.db` (Linux/Mac)
- **配置文件备份**: `~/.claude/settings.json.backup` 等
- **Docker 数据卷**: `ai-gateway-data`

## 🗺️ 开发计划

- [x] Provider 管理
- [x] CLI 工具配置管理
- [ ] Dashboard 统计面板
- [ ] MCP 服务器管理
- [ ] AI 请求代理服务
- [ ] 多用户支持
- [ ] 配置云端同步

## 💻 技术栈

### 后端
- **框架**: [Axum](https://github.com/tokio-rs/axum) (Rust)
- **数据库**: SQLite + [rusqlite](https://github.com/rusqlite/rusqlite)
- **HTTP 客户端**: [reqwest](https://github.com/seanmonstar/reqwest)
- **异步运行时**: [Tokio](https://tokio.rs/)

### 前端
- **框架**: React 18 + TypeScript
- **构建**: [Vite](https://vitejs.dev/)
- **样式**: [Tailwind CSS](https://tailwindcss.com/)
- **数据获取**: [TanStack Query](https://tanstack.com/query)
- **状态管理**: [Zustand](https://github.com/pmndrs/zustand)
- **图标**: [Lucide React](https://lucide.dev/)

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建您的功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开一个 Pull Request

## 📄 许可证

[MIT](LICENSE) License

## 🙏 致谢

本项目灵感来源于 [cc-switch](https://github.com/farion1231/cc-switch) 桌面应用，将其扩展为服务器端版本，支持远程管理和 Web 界面。
