# AI Gateway Hub - 服务器部署指南

## 快速部署

### 1. 环境准备

服务器要求：
- Linux (Ubuntu 20.04+ / Debian 11+ / CentOS 8+)
- Docker 20.10+
- Docker Compose 2.0+
- 至少 1GB 内存
- 至少 5GB 磁盘空间

### 2. 安装 Docker

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y docker.io docker-compose-plugin

# 或者使用官方脚本
curl -fsSL https://get.docker.com | sh

# 启动 Docker
sudo systemctl enable docker
sudo systemctl start docker

# 将当前用户加入 docker 组（可选，免 sudo）
sudo usermod -aG docker $USER
newgrp docker
```

### 3. 部署 AI Gateway Hub

```bash
# 创建项目目录
mkdir -p ~/ai-gateway-hub && cd ~/ai-gateway-hub

# 克隆项目
git clone https://github.com/yourusername/ai-gateway-hub.git .

# 或者手动上传文件后解压
# unzip ai-gateway-hub.zip && cd ai-gateway-hub

# 创建必要的目录（如果不存在）
mkdir -p ~/.claude ~/.codex ~/.gemini ~/.opencode ~/.openclaw

# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f
```

服务将运行在 `http://服务器IP:8677`

### 4. 配置反向代理（推荐）

使用 Nginx 配置域名和 HTTPS：

```bash
# 安装 Nginx
sudo apt install -y nginx

# 创建配置文件
sudo tee /etc/nginx/sites-available/ai-gateway << 'EOF'
server {
    listen 80;
    server_name ai.yourdomain.com;  # 替换为你的域名

    location / {
        proxy_pass http://127.0.0.1:8677;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
EOF

# 启用配置
sudo ln -s /etc/nginx/sites-available/ai-gateway /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx

# 配置 HTTPS (使用 Certbot)
sudo apt install -y certbot python3-certbot-nginx
sudo certbot --nginx -d ai.yourdomain.com
```

或者使用 Caddy（更简单）：

```bash
# 安装 Caddy
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy

# 配置 Caddy
sudo tee /etc/caddy/Caddyfile << 'EOF'
ai.yourdomain.com {
    reverse_proxy localhost:8677
}
EOF

sudo systemctl restart caddy
```

### 5. 防火墙配置

```bash
# UFW (Ubuntu)
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 8677/tcp  # 如果直接暴露端口
sudo ufw enable

# 或者 Firewalld (CentOS)
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --permanent --add-port=8677/tcp
sudo firewall-cmd --reload
```

### 6. 后台管理和维护

```bash
# 查看运行状态
docker-compose ps

# 查看日志
docker-compose logs -f

# 重启服务
docker-compose restart

# 停止服务
docker-compose down

# 完全删除（包括数据）
docker-compose down -v

# 更新到最新版本
git pull
docker-compose down
docker-compose up -d --build

# 备份数据
docker run --rm -v ai-gateway-hub_ai-gateway-data:/data -v $(pwd):/backup alpine tar czf /backup/ai-gateway-backup-$(date +%Y%m%d).tar.gz -C /data .

# 恢复数据
docker run --rm -v ai-gateway-hub_ai-gateway-data:/data -v $(pwd):/backup alpine tar xzf /backup/ai-gateway-backup-XXXX.tar.gz -C /data
```

### 7. 开机自启

Docker Compose 默认已经配置了 `restart: unless-stopped`，容器会在系统重启后自动启动。

如果需要更严格的控制，可以创建 systemd 服务：

```bash
sudo tee /etc/systemd/system/ai-gateway-hub.service << 'EOF'
[Unit]
Description=AI Gateway Hub
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/root/ai-gateway-hub
ExecStart=/usr/bin/docker-compose up -d
ExecStop=/usr/bin/docker-compose down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable ai-gateway-hub
sudo systemctl start ai-gateway-hub
```

### 8. 配置说明

#### 环境变量

创建 `.env` 文件来自定义配置：

```bash
cat > .env << 'EOF'
# 服务端口
PORT=8080
PROXY_PORT=9090
ENABLE_PROXY=true

# 日志级别
RUST_LOG=info
EOF
```

#### 数据持久化

数据存储在 Docker 卷中：
- `ai-gateway-data`: SQLite 数据库
- `~/.claude`: Claude Code 配置
- `~/.codex`: Codex 配置
- `~/.gemini`: Gemini CLI 配置
- `~/.opencode`: OpenCode 配置
- `~/.openclaw`: OpenClaw 配置

### 9. 安全建议

1. **使用 HTTPS**: 强烈建议使用 Nginx/Caddy + SSL
2. **限制访问**: 使用防火墙限制 8677 端口只允许本地访问
3. **定期备份**: 设置定时任务备份数据库和配置文件
4. **更新镜像**: 定期更新基础镜像和依赖

```bash
# 设置自动更新（每周一凌晨）
(crontab -l 2>/dev/null; echo "0 2 * * 1 cd ~/ai-gateway-hub && docker-compose pull && docker-compose up -d") | crontab -
```

### 10. 故障排查

```bash
# 查看容器日志
docker-compose logs -f

# 检查容器状态
docker ps -a

# 进入容器调试
docker exec -it ai-gateway-hub sh

# 检查端口占用
sudo netstat -tlnp | grep 8677

# 重启服务
docker-compose restart

# 清理未使用的镜像和卷
docker system prune -a -f --volumes
```

### 11. 多服务器部署

如果有多台服务器需要管理 CLI 配置，可以：

1. 在一台服务器部署 AI Gateway Hub
2. 其他服务器通过 SSH/SCP 同步配置
3. 或者每台服务器独立部署，配置共享存储

使用 rsync 同步配置示例：

```bash
# 从 Hub 服务器同步到工作服务器
rsync -avz user@hub-server:~/.claude/settings.json ~/.claude/
```

## 使用流程

### 首次使用

1. 访问 `http://服务器IP:8677` 或 `https://ai.yourdomain.com`
2. 进入 **Providers** 页面，添加你的 AI 供应商（OpenAI、Anthropic 等）
3. 进入 **CLI Tools** 页面，创建 CLI 工具配置
4. 点击 **Apply** 将配置写入本地

### 切换配置

1. 在 CLI Tools 页面点击 **Backup** 备份当前配置
2. 选择另一个配置，点击 **Apply** 应用
3. 如需恢复，点击 **Restore**

## 更新升级

```bash
cd ~/ai-gateway-hub

# 拉取最新代码
git pull origin main

# 重新构建并启动
docker-compose down
docker-compose up -d --build

# 查看更新日志
docker-compose logs -f
```

## 卸载

```bash
cd ~/ai-gateway-hub

# 停止并删除容器
docker-compose down -v

# 删除镜像
docker rmi ai-gateway-hub_ai-gateway-hub

# 删除项目目录
cd ~ && rm -rf ai-gateway-hub

# 如果配置了 systemd
sudo systemctl stop ai-gateway-hub
sudo systemctl disable ai-gateway-hub
sudo rm /etc/systemd/system/ai-gateway-hub.service
```
