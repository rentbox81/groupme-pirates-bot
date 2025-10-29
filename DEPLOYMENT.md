# GroupMe Bot Deployment Guide

## Quick Start
```bash
# Clone/pull the project
git clone <repository-url>
cd groupme-pirates-bot

# Copy environment template and configure
cp .env.template .env
# Edit .env with your specific values

# Build and start the bot
docker compose up -d --build
```

## Environment Configuration

The bot requires the following environment variables in `.env`:

```bash
# GroupMe Configuration
GROUPME_BOT_ID=your_bot_id_here
GROUPME_BOT_NAME=PirateBot

# Google Sheets Configuration  
SHEET_ID=your_google_sheet_id
GOOGLE_API_KEY=your_google_api_key

# Calendar Configuration
CALENDAR_WEBCAL_URL=your_calendar_webcal_url

# Domain Configuration (for Traefik)
TEAM_NAME=pirate
BASE_DOMAIN=your-domain.com
CERT_RESOLVER=letsencrypt

# Server Configuration
PORT=18080
RUST_LOG=info
GOOGLE_SERVICE_ACCOUNT_JSON=/app/service-account.json
```

## Service Account Setup

1. Create a Google Cloud service account
2. Download the JSON key file
3. Rename it to `service-account.json` and place in project root
4. The Dockerfile will automatically include it in the container

## Deployment Features

### Automatic Capabilities
- ✅ Google Sheets API authentication (built into container)
- ✅ Health checks and monitoring
- ✅ Auto-restart on failure
- ✅ Resource limits and reservations
- ✅ Traefik integration with SSL
- ✅ Rate limiting and security headers
- ✅ HTTP to HTTPS redirect

### Network Access
- **Internal**: http://localhost:18080
- **External**: https://{TEAM_NAME}bot.{BASE_DOMAIN}

### Container Resources
- **CPU Limit**: 0.5 cores
- **Memory Limit**: 512MB
- **CPU Reserved**: 0.1 cores  
- **Memory Reserved**: 128MB

## Troubleshooting

### Check Status
```bash
docker compose ps
docker compose logs -f
```

### Rebuild After Changes
```bash
docker compose down
docker compose up -d --build
```

### Test Webhook
```bash
curl -X POST http://localhost:18080/webhook \
  -H "Content-Type: application/json" \
  -d '{"sender_type": "user", "text": "@PirateBot test"}'
```

## Development Variants

Alternative compose files are available in `deployment-variants/`:
- `docker-compose.local.yml` - Local development with direct port access
- `docker-compose.prod.yml` - Production with advanced Traefik config
- `docker-compose.debug.yml` - Debug mode configuration

To use a variant:
```bash
docker compose -f deployment-variants/docker-compose.local.yml up -d
```
