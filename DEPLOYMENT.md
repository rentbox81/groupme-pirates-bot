# Multi-Team GroupMe Bot Deployment Guide

## Overview

This bot is designed to support multiple teams using a configurable domain pattern:
**`{TEAM_NAME}bot.{BASE_DOMAIN}`**

Examples:
- **Pirates Team**: `https://piratesbot.rentbox.us`
- **Dragons Team**: `https://dragonsbot.rentbox.us`  
- **Eagles Team**: `https://eaglesbot.myteambot.com`

## Configuration

### 1. Set Up Your Environment

Copy the template and customize for your team:
```bash
cp .env.template .env
```

Edit `.env` with your team's values:
```bash
# Team-specific configuration
TEAM_NAME=pirates            # lowercase, used for subdomain
BASE_DOMAIN=rentbox.us       # your registered domain
GROUPME_BOT_NAME=PirateBot   # matches your team theme

# Your actual API keys and IDs
GROUPME_BOT_ID=your_actual_bot_id
SHEET_ID=your_actual_sheet_id
GOOGLE_API_KEY=your_actual_api_key
CALENDAR_WEBCAL_URL=your_actual_webcal_url
```

### 2. Domain Examples

| Team Name | Configuration | Result URL |
|-----------|---------------|------------|
| Pirates | `TEAM_NAME=pirates` | `https://piratesbot.rentbox.us` |
| Dragons | `TEAM_NAME=dragons` | `https://dragonsbot.rentbox.us` |  
| Eagles | `TEAM_NAME=eagles` | `https://eaglesbot.myteambot.com` |
| Lions | `TEAM_NAME=lions` | `https://lionsbot.sportsbot.net` |

## Prerequisites

### 1. DNS Configuration
Set up DNS A records for your bot domains:
```
piratesbot.rentbox.us.  IN  A  YOUR_SERVER_IP
dragonsbot.rentbox.us.  IN  A  YOUR_SERVER_IP
```

Or use a wildcard record:
```
*.rentbox.us.  IN  A  YOUR_SERVER_IP
```

### 2. Traefik Setup
Make sure Traefik is running with Let's Encrypt:
```bash
# Create traefik network if it doesn't exist
docker network create traefik

# Traefik should be configured with:
# - Port 80 (web entrypoint)
# - Port 443 (websecure entrypoint)  
# - Let's Encrypt certificate resolver
```

## Deployment

### Development
```bash
docker-compose up -d
```

### Production
```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Multiple Team Deployment

You can run multiple teams on the same server by:

### Option 1: Separate Directories (Recommended)
```bash
# Set up different team directories
mkdir -p ~/bots/pirates
mkdir -p ~/bots/dragons

# Copy bot files to each directory
cp -r /path/to/groupme-bot/* ~/bots/pirates/
cp -r /path/to/groupme-bot/* ~/bots/dragons/

# Configure each team's .env file
cd ~/bots/pirates
# Edit .env with TEAM_NAME=pirates, etc.

cd ~/bots/dragons  
# Edit .env with TEAM_NAME=dragons, etc.

# Deploy each team
cd ~/bots/pirates && docker-compose -f docker-compose.prod.yml up -d
cd ~/bots/dragons && docker-compose -f docker-compose.prod.yml up -d
```

### Option 2: Docker Compose Override Files
```bash
# Create team-specific override files
# docker-compose.pirates.yml
# docker-compose.dragons.yml

docker-compose -f docker-compose.prod.yml -f docker-compose.pirates.yml up -d
```

## Traefik Labels Explained

The bot automatically generates Traefik labels based on your configuration:

```yaml
# Generated from TEAM_NAME=pirates, BASE_DOMAIN=rentbox.us
- "traefik.http.routers.piratesbot.rule=Host(`piratesbot.rentbox.us`)"
- "traefik.http.services.piratesbot.loadbalancer.server.port=18080"

# Each team gets its own router and service names to avoid conflicts
```

## Monitoring & Management

### Check Deployment Status
```bash
# View logs for specific team
docker-compose logs -f groupme-bot

# Check if container is running
docker-compose ps

# View Traefik routing (if dashboard enabled)
curl -s http://localhost:8080/api/http/routers | jq
```

### Health Checks
Each deployment includes automatic health checks:
```bash
# Test endpoint directly
curl -I https://piratesbot.rentbox.us/

# Check SSL certificate
curl -vI https://piratesbot.rentbox.us/ 2>&1 | grep -A 5 "SSL connection"
```

## GroupMe Webhook Configuration

After deployment, update your GroupMe bot's callback URL:

| Team | Callback URL |
|------|-------------|
| Pirates | `https://piratesbot.rentbox.us/` |
| Dragons | `https://dragonsbot.rentbox.us/` |
| Eagles | `https://eaglesbot.myteambot.com/` |

## Security Features (Production)

- **SSL/TLS**: Automatic Let's Encrypt certificates
- **Rate Limiting**: Team-specific rate limits (50 req/sec avg, 100 burst)
- **Security Headers**: HTTPS forwarding, robot exclusion
- **Resource Limits**: CPU and memory constraints
- **Health Monitoring**: Automatic restart on failures

## Troubleshooting

### Domain Resolution Issues
```bash
# Test DNS resolution
nslookup piratesbot.rentbox.us

# Test from different locations
dig piratesbot.rentbox.us @8.8.8.8
```

### Container Issues
```bash
# Check container logs
docker-compose logs groupme-bot

# Restart specific deployment
docker-compose restart groupme-bot

# Rebuild and redeploy
docker-compose down && docker-compose up -d --build
```

### Traefik Issues
```bash
# Check Traefik logs
docker logs traefik

# Verify network connectivity
docker network ls | grep traefik
docker network inspect traefik
```

## Example Multi-Team Setup

```bash
# Server directory structure
/home/user/bots/
├── pirates/
│   ├── .env                    # TEAM_NAME=pirates
│   ├── docker-compose.prod.yml
│   └── ... (bot files)
├── dragons/
│   ├── .env                    # TEAM_NAME=dragons  
│   ├── docker-compose.prod.yml
│   └── ... (bot files)
└── eagles/
    ├── .env                    # TEAM_NAME=eagles
    ├── docker-compose.prod.yml
    └── ... (bot files)

# Resulting URLs:
# https://piratesbot.rentbox.us
# https://dragonsbot.rentbox.us  
# https://eaglesbot.rentbox.us
```

This setup allows you to easily deploy and manage multiple team bots while maintaining clean separation and avoiding conflicts.
