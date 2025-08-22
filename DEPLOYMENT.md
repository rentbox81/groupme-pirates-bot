# 🏴‍☠️ GroupMe Pirates Bot - Easy Deployment Guide

Deploy the GroupMe Pirates Bot using pre-built Docker images - no compilation required!

## Quick Start

### Prerequisites
- Docker and Docker Compose installed
- A GroupMe bot created at https://dev.groupme.com/bots  
- A Google Sheet with your team schedule
- (Optional) Google Service Account for volunteer assignment features

### 1. Download Deployment Files

```bash
# Create deployment directory
mkdir groupme-pirates-bot && cd groupme-pirates-bot

# Download the production docker-compose.yml
curl -O https://raw.githubusercontent.com/rentbox81/groupme-pirates-bot/main/deploy/docker-compose.yml

# Download environment template  
curl -O https://raw.githubusercontent.com/rentbox81/groupme-pirates-bot/main/deploy/.env.template
```

### 2. Configure Environment

```bash
# Copy template to .env file
cp .env.template .env

# Edit the configuration
nano .env
```

Fill in your configuration:

```bash
# GroupMe Bot Configuration
GROUPME_BOT_ID=your_actual_groupme_bot_id
GROUPME_BOT_NAME=PirateBot

# Google Sheets Configuration  
SHEET_ID=your_google_sheet_id_from_url
GOOGLE_API_KEY=your_google_api_key

# Calendar Configuration
CALENDAR_WEBCAL_URL=your_team_calendar_webcal_url

# Domain Configuration (for Traefik reverse proxy)
TEAM_NAME=pirates
BASE_DOMAIN=yourdomain.com
```

### 3. (Optional) Service Account Setup

For volunteer assignment features, add a Google Service Account:

```bash
# Place your service account JSON file in the deployment directory
cp path/to/your/service-account.json ./service-account.json

# Update .env to use service account
echo "GOOGLE_SERVICE_ACCOUNT_JSON=/app/service-account.json" >> .env
```

### 4. Deploy

```bash
# Create Traefik network (if using Traefik)
docker network create traefik

# Start the bot
docker-compose up -d

# Check status
docker-compose logs -f
```

## Deployment Options

### Option 1: Simple Docker Run (No Traefik)

```bash
docker run -d \
  --name groupme-pirates-bot \
  --restart unless-stopped \
  -p 18080:18080 \
  -e GROUPME_BOT_ID=your_bot_id \
  -e GROUPME_BOT_NAME=PirateBot \
  -e SHEET_ID=your_sheet_id \
  -e GOOGLE_API_KEY=your_api_key \
  -e CALENDAR_WEBCAL_URL=your_calendar_url \
  -v $(pwd)/service-account.json:/app/service-account.json:ro \
  rentbox81/groupme-pirates-bot:latest
```

### Option 2: Docker Compose with Traefik

Use the provided `docker-compose.yml` which includes Traefik labels for automatic SSL and domain routing.

### Option 3: Kubernetes Deployment

<details>
<summary>Kubernetes Deployment Example</summary>

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: groupme-pirates-bot
spec:
  replicas: 1
  selector:
    matchLabels:
      app: groupme-pirates-bot
  template:
    metadata:
      labels:
        app: groupme-pirates-bot
    spec:
      containers:
      - name: groupme-pirates-bot
        image: rentbox81/groupme-pirates-bot:latest
        ports:
        - containerPort: 18080
        env:
        - name: GROUPME_BOT_ID
          valueFrom:
            secretKeyRef:
              name: groupme-bot-secrets
              key: bot-id
        - name: GROUPME_BOT_NAME
          value: "PirateBot"
        - name: SHEET_ID
          valueFrom:
            secretKeyRef:
              name: groupme-bot-secrets  
              key: sheet-id
        # ... add other environment variables
        volumeMounts:
        - name: service-account
          mountPath: /app/service-account.json
          subPath: service-account.json
          readOnly: true
      volumes:
      - name: service-account
        secret:
          secretName: service-account-secret
---
apiVersion: v1
kind: Service
metadata:
  name: groupme-pirates-bot
spec:
  selector:
    app: groupme-pirates-bot
  ports:
  - port: 80
    targetPort: 18080
```

</details>

## Configuration Reference

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `GROUPME_BOT_ID` | Your GroupMe bot ID | `6602cda742671375feff740221` |
| `GROUPME_BOT_NAME` | Bot name in GroupMe | `PirateBot` |
| `SHEET_ID` | Google Sheet ID from URL | `1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms` |
| `CALENDAR_WEBCAL_URL` | Team calendar webcal URL | `webcal://example.com/calendar.ics` |

### Optional Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GOOGLE_API_KEY` | Google API key (read-only) | None |
| `GOOGLE_SERVICE_ACCOUNT_JSON` | Path to service account file | None |
| `TEAM_NAME` | Team name for domain routing | `team` |
| `BASE_DOMAIN` | Base domain for webhook URL | `localhost` |
| `PORT` | Internal port | `18080` |
| `RUST_LOG` | Log level | `info` |

## Features

### 🎯 Game Information
- `@BotName next game` - Full details for next game
- `@BotName next 3 games` - Show next 3 games  
- `@BotName next game snacks` - Get snacks info for next game

### 🏴‍☠️ Team Spirit
- `@BotName lets go pirates` - Get a Pirates fact!

### 👥 Volunteer Management (Requires Service Account)
- `@BotName volunteer snacks 2025-08-23 John` - Sign up for snacks
- `@BotName volunteer pitchcount 2025-08-23 Sarah` - Sign up for pitch counting
- `@BotName volunteer livestream 2025-08-23 Mike` - Sign up for livestream
- `@BotName volunteer scoreboard 2025-08-23 Lisa` - Sign up for scoreboard
- `@BotName volunteers` - Show all volunteer needs
- `@BotName volunteers 2025-08-23` - Show needs for specific date

## Troubleshooting

### Bot Not Responding
1. Check GroupMe webhook URL is set correctly
2. Verify bot is accessible at your domain/IP
3. Check container logs: `docker-compose logs -f`

### Volunteer Commands Not Working  
1. Ensure service account JSON is properly mounted
2. Verify service account has Editor access to Google Sheet
3. Check for error codes in chat (VOL001, SVC001, etc.)

### Permission Errors
- Error `SVC001`: Service processing failed - check logs
- Error `VOL001`: Volunteer update failed - check sheet permissions  
- Error `CMD001`: Command parsing failed - check command format

### Health Check

```bash
# Test bot health
curl http://localhost:18080/

# Should return:
# {"service":"GroupMe Bot","status":"ok","version":"0.1.0"}
```

## Security Notes

- Service account JSON files contain sensitive credentials
- Use secrets management in production environments
- Bot runs as non-root user inside container
- Only expose port 18080 if needed for direct access

## Docker Images

The bot is available as multi-architecture images supporting:
- `linux/amd64` (Intel/AMD x64)
- `linux/arm64` (Apple Silicon, ARM servers)

### Available Tags
- `latest` - Latest stable release
- `main` - Latest development build  
- `v1.0.0` - Specific version tags

## Support

- GitHub Issues: https://github.com/rentbox81/groupme-pirates-bot/issues
- Documentation: https://github.com/rentbox81/groupme-pirates-bot/blob/main/README.md

**🏴‍☠️ Raise the Jolly Roger! Your bot deployment is ready to sail! ⚾**
