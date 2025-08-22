# 🏴‍☠️ GroupMe Pirates Bot ⚾

A comprehensive team management bot for GroupMe that helps coordinate baseball/softball team activities, volunteer assignments, and game information.

## 🚀 Quick Deploy (Recommended)

**Deploy in minutes with pre-built Docker images - no compilation required!**

```bash
# Download deployment files
mkdir groupme-pirates-bot && cd groupme-pirates-bot
curl -O https://raw.githubusercontent.com/rentbox81/groupme-pirates-bot/main/deploy/docker-compose.yml
curl -O https://raw.githubusercontent.com/rentbox81/groupme-pirates-bot/main/deploy/.env.template

# Configure your bot
cp .env.template .env
nano .env  # Fill in your configuration

# Deploy
docker-compose up -d
```

**📖 Full deployment guide: [DEPLOYMENT.md](DEPLOYMENT.md)**

## 🐳 Docker Images

**Multi-architecture images available on Docker Hub:**

[![Docker Hub](https://img.shields.io/docker/v/rentwork/groupme-pirates-bot?label=Docker%20Hub&logo=docker)](https://hub.docker.com/r/rentwork/groupme-pirates-bot)
[![Image Size](https://img.shields.io/docker/image-size/rentwork/groupme-pirates-bot/latest?label=Image%20Size&logo=docker)](https://hub.docker.com/r/rentwork/groupme-pirates-bot)
[![Docker Pulls](https://img.shields.io/docker/pulls/rentwork/groupme-pirates-bot?label=Downloads&logo=docker)](https://hub.docker.com/r/rentwork/groupme-pirates-bot)

- `rentwork/groupme-pirates-bot:latest` - Latest stable release
- `rentwork/groupme-pirates-bot:v1.0.0` - Tagged releases
- `rentwork/groupme-pirates-bot:main` - Development builds (when CI/CD is enabled)

**Supported Platforms:**
- `linux/amd64` (Intel/AMD x64) ✅
- `linux/arm64` (Apple Silicon, ARM servers) - Coming soon with CI/CD

**Image Stats:**
- **Size**: ~116MB (optimized multi-stage build)
- **Base**: Debian Bookworm Slim (security updates)
- **Security**: Non-root user, minimal attack surface

## ✨ Features

### 🎯 Game Information
- **Next Game Details**: Get complete game information including time, location, and opponent
- **Game Schedule**: View upcoming games with snack assignments
- **Team Calendar Integration**: Automatic sync with team calendar systems

### 👥 Volunteer Management  
- **Snack Assignments**: Sign up and manage snack volunteers
- **Game Roles**: Coordinate pitch counting, scoreboard, and livestream volunteers
- **Automatic Updates**: Real-time Google Sheets integration for volunteer tracking

### 🏴‍☠️ Team Spirit
- **Pirates Facts**: Random fun facts about the Pittsburgh Pirates
- **Team Motivation**: Customizable team-specific responses and encouragement

### 🔧 Advanced Features
- **Multi-Architecture Support**: Runs on x64 and ARM64 (Apple Silicon, Raspberry Pi)
- **Secure Authentication**: Google Service Account integration for write operations
- **Error Resilience**: Graceful error handling with user-friendly error codes
- **Traefik Integration**: Automatic SSL and reverse proxy configuration

## 📋 Commands

### Basic Commands
```
@PirateBot commands              # Show all available commands
@PirateBot next game             # Full details for next game
@PirateBot next 3 games          # Show next 3 games
@PirateBot next game snacks      # Get snacks info for next game
@PirateBot lets go pirates       # Get a Pirates fact!
```

### Volunteer Management
```
@PirateBot volunteer snacks 2025-08-23 John        # Sign up for snacks
@PirateBot volunteer pitchcount 2025-08-23 Sarah   # Sign up for pitch counting  
@PirateBot volunteer livestream 2025-08-23 Mike    # Sign up for livestream
@PirateBot volunteer scoreboard 2025-08-23 Lisa    # Sign up for scoreboard
@PirateBot volunteers                               # Show all volunteer needs
@PirateBot volunteers 2025-08-23                   # Show needs for specific date
```

## 🚢 Deployment Options

### 1. Docker Run (Quick Start)
```bash
docker run -d --name pirates-bot \
  -e GROUPME_BOT_ID=your_bot_id \
  -e GROUPME_BOT_NAME=PirateBot \
  -e SHEET_ID=your_sheet_id \
  -e CALENDAR_WEBCAL_URL=your_calendar_url \
  -p 18080:18080 \
  rentwork/groupme-pirates-bot:latest
```

### 2. Docker Compose (Recommended)
```bash
# Download deployment files
curl -O https://raw.githubusercontent.com/rentbox81/groupme-pirates-bot/main/deploy/docker-compose.yml
curl -O https://raw.githubusercontent.com/rentbox81/groupme-pirates-bot/main/deploy/.env.template

# Configure
cp .env.template .env
nano .env

# Deploy with Traefik (production)
docker-compose up -d

# Or simple deployment (development)
docker run -d --name pirates-bot \
  --env-file .env \
  -p 18080:18080 \
  rentwork/groupme-pirates-bot:latest
```

### 3. Kubernetes
See [DEPLOYMENT.md](DEPLOYMENT.md#option-3-kubernetes-deployment) for Kubernetes manifests.

## 🔧 Configuration

### Required Environment Variables
| Variable | Description | Example |
|----------|-------------|---------|
| `GROUPME_BOT_ID` | Your GroupMe bot ID | `abc123def456` |
| `GROUPME_BOT_NAME` | Bot name for @mentions | `PirateBot` |
| `SHEET_ID` | Google Sheet ID from URL | `1abc...xyz` |
| `CALENDAR_WEBCAL_URL` | Team calendar webcal URL | `webcal://calendar.example.com/feed` |

### Optional Environment Variables  
| Variable | Description | Default |
|----------|-------------|---------|
| `GOOGLE_API_KEY` | Google API key | None |
| `GOOGLE_SERVICE_ACCOUNT_JSON` | Service account file path | None |
| `TEAM_NAME` | Team name for routing | `team` |
| `BASE_DOMAIN` | Domain for webhooks | `localhost` |
| `PORT` | HTTP port | `18080` |
| `RUST_LOG` | Log level | `info` |

## 🛠 Development Setup

<details>
<summary>Local Development (Click to expand)</summary>

### Prerequisites
- Rust 1.82+ 
- Docker and Docker Compose
- GroupMe Bot Token
- Google API access

### Setup
```bash
git clone https://github.com/rentbox81/groupme-pirates-bot.git
cd groupme-pirates-bot

# Copy environment template
cp .env.template .env
# Edit .env with your configuration

# Run locally
cargo run

# Or with Docker
docker-compose up --build
```

### Development Commands
```bash
# Run tests
cargo test

# Check code
cargo check

# Format code  
cargo fmt

# Run with debug logging
RUST_LOG=debug cargo run

# Test Google APIs
cargo run --bin test-google-apis

# Test bot commands (mock mode)
cargo run --bin test-bot-mock

# Build Docker image locally
docker build -t groupme-pirates-bot:local .
```

</details>

## 🔒 Security & Authentication

### Google Sheets Integration
- **API Key**: Read-only access to public sheets
- **Service Account**: Full read/write access (recommended for volunteer features)
- **Secure Credentials**: JSON credentials mounted as read-only volumes

### Container Security
- Non-root user execution
- Minimal base image with security updates
- Environment-based configuration (no hardcoded secrets)
- Health check monitoring

## 🏗 Architecture

```
GroupMe ← Webhook → Bot ← APIs → Google Sheets
                    ↓           ↗ Google Calendar  
                 Docker      ↗
                 Container ↗
```

### Components
- **Actix Web**: High-performance async web framework
- **Reqwest**: HTTP client for API integrations
- **Tokio**: Async runtime for handling concurrent requests
- **Docker**: Containerized deployment with multi-stage builds

## 📊 Monitoring & Observability

- **Health Checks**: Built-in health endpoint (`/health`)  
- **Structured Logging**: JSON-formatted logs with request tracing
- **Error Codes**: User-friendly error codes (SVC001, VOL001, etc.)
- **Metrics**: Request timing and success/failure tracking

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines
- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation for user-facing changes
- Ensure Docker builds succeed for all platforms

## 🔄 CI/CD Pipeline

The project includes GitHub Actions workflows for automated building and deployment:

- **Automated Docker Builds**: Multi-platform images (amd64, arm64)
- **Docker Hub Publishing**: Automatic tagging and publishing
- **Deployment Artifacts**: Generated docker-compose and configuration files
- **Security Scanning**: Container vulnerability assessments

> **Note**: CI/CD requires GitHub token with `workflow` scope. Currently using manual builds.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🏴‍☠️ Support

- **Issues**: [GitHub Issues](https://github.com/rentbox81/groupme-pirates-bot/issues)
- **Documentation**: [Full Documentation](https://github.com/rentbox81/groupme-pirates-bot/blob/main/DEPLOYMENT.md)
- **Docker Hub**: [Image Repository](https://hub.docker.com/r/rentwork/groupme-pirates-bot)

---

**🏴‍☠️ Raise the Jolly Roger! Your team management just got a first mate! ⚾**

*Built with ⚡ Rust and ❤️ for youth baseball teams*
