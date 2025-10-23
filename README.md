# GroupMe Pirates Bot 🏴‍☠️

A Rust-based GroupMe bot that integrates with Google Sheets and Google Calendar to provide team scheduling and management for the Pirates team. **Now with conversational AI!**

## Features

### 🔐 Admin & Moderator System
- **@Mention Support**: `@PirateBot add moderator @UserName` - Uses GroupMe mentions to identify users
- **Role Management**: Admin can add/remove moderators, moderators can assign/remove volunteers
- **Persistent Storage**: Moderator list saved to `data/moderators.json` and survives restarts
- **Authorization**: Protected commands require admin or moderator permissions

- 🤖 **Conversational Interface**: Talk naturally to the bot - no strict commands needed!
- 📊 **Google Sheets API**: Fetches team data and schedules  
- 📅 **Calendar Integration**: Shows upcoming games and events
- 🌐 **Web Interface**: Webhook endpoint for GroupMe notifications
- 🔒 **Secure**: Built-in authentication and rate limiting
- 🐳 **Containerized**: Easy deployment with Docker

## Conversational Interface

The bot now understands natural language! You don't need to remember exact commands.

### Natural Volunteer Sign-ups
- `@PirateBot I've got snacks` - Sign up for snacks
- `@PirateBot I can do livestream for Saturday` - Volunteer for a specific date
- `@PirateBot put me down for scoreboard` - Sign up for scoreboard
- `@PirateBot I'll bring snacks next game for John` - Sign someone else up

### Natural Game Queries
- `@PirateBot when's the next game?` - Get next game info
- `@PirateBot where are we playing?` - Location info
- `@PirateBot what time is the game?` - Game time
- `@PirateBot show me the next 3 games` - Multiple games

### Volunteer Status
- `@PirateBot who's bringing snacks?` - Check volunteer status
- `@PirateBot do we need anything?` - See what roles are open
- `@PirateBot volunteers for Saturday` - Check specific date

### Team Spirit

### Admin Commands (Admin Only)
- `@PirateBot add moderator @UserName` - Add a moderator using @mention
- `@PirateBot remove moderator @UserName` - Remove a moderator
- `@PirateBot list moderators` - Show all moderators and admin

### Moderator Commands (Admin + Moderators)
- `@PirateBot assign @UserName to snacks` - Assign volunteer
- `@PirateBot remove @UserName from livestream` - Remove volunteer assignment
- `@PirateBot let's go pirates!` - Get a Pirates fact
- `@PirateBot go pirates!` - Team motivation

### Need Help?
- `@PirateBot help` - See what the bot can do
- Just mention `@PirateBot` - The bot will guide you

## Traditional Commands (Still Supported)

For those who prefer exact commands:

- `@PirateBot next game` - Show upcoming game details
- `@PirateBot next 3 games` - Show next 3 games
- `@PirateBot next game snacks` - Get specific category info
- `@PirateBot volunteer snacks 2025-01-15 John` - Sign up to volunteer
- `@PirateBot volunteers` - Show all volunteer needs
- `@PirateBot volunteers 2025-01-15` - Show needs for specific date

## Quick Start

### Prerequisites

1. **GroupMe Bot**: Create a bot at [dev.groupme.com/bots](https://dev.groupme.com/bots)
2. **Google Sheet**: Set up your team data spreadsheet
3. **Google Cloud**: Service account with Sheets API enabled
4. **Calendar**: WebCal URL for your team's schedule

### Installation

```bash
# 1. Clone the repository
git clone <your-repo-url>
cd groupme-pirates-bot

# 2. Set up environment variables
cp .env.template .env
# Edit .env with your configuration (see below)

# 3. Add Google service account key
# Place your service-account.json file in the project root
# Get from: Google Cloud Console > IAM & Admin > Service Accounts

# 4. Build and run
docker compose up -d --build

# 5. Check logs
docker compose logs -f

# 6. Configure GroupMe callback
# Set your bot's callback URL to: https://{TEAM_NAME}bot.{BASE_DOMAIN}/webhook
```

### Local Development

```bash
# Run locally without Docker
cargo run

# Run tests
cargo test

# Run with local docker-compose
docker compose -f deployment-variants/docker-compose.local.yml up -d
```

## Configuration

### Required Environment Variables

All environment variables are documented in [.env.template](./.env.template). Key required variables:

- **`GROUPME_BOT_ID`** - Your GroupMe bot ID from [dev.groupme.com/bots](https://dev.groupme.com/bots)
- **`GROUPME_BOT_NAME`** - Bot name (e.g., PirateBot) - used for @mentions
- **`SHEET_ID`** - Google Sheets ID from your spreadsheet URL
- **`GOOGLE_API_KEY`** - Google API key with Sheets API enabled
- **`CALENDAR_WEBCAL_URL`** - Team calendar WebCal URL
- **`ADMIN_USER_ID`** - GroupMe user ID of the bot administrator

### Optional Environment Variables

- **`PORT`** - Server port (default: 18080)
- **`RUST_LOG`** - Logging level: info, debug, trace (default: info)
- **`REMINDER_START_HOUR`** - Start sending reminders (default: 9, 24-hour format)
- **`REMINDER_END_HOUR`** - Stop sending reminders (default: 21, 24-hour format)
- **`TEAM_NAME`** - For subdomain (e.g., pirates → piratesbot.yourdomain.com)
- **`BASE_DOMAIN`** - Your domain for external access

See [.env.template](./.env.template) for complete documentation of all variables.

### Required Files

1. **`.env`** - Environment configuration (copy from .env.template)
2. **`service-account.json`** - Google Cloud service account key with Sheets API access

## Access Points

- **Webhook**: `https://{TEAM_NAME}bot.{BASE_DOMAIN}/webhook`
- **Health Check**: `http://localhost:18080/` or `https://{TEAM_NAME}bot.{BASE_DOMAIN}/`
- **Logs**: `docker compose logs -f`

## Deployment

See [DEPLOYMENT.md](./DEPLOYMENT.md) for detailed production deployment instructions.

## Development

```bash
# Local development
docker compose -f deployment-variants/docker-compose.local.yml up -d

# View logs
docker compose logs -f

# Rebuild after changes
docker compose up -d --build

# Run tests
cargo test
```

## Architecture

- **Runtime**: Rust with Actix-web framework
- **AI**: Natural language understanding with intent detection
- **APIs**: GroupMe Webhook + Google Sheets + Calendar
- **Deployment**: Docker with Traefik reverse proxy
- **Security**: Rate limiting, HTTPS, security headers

## What's New

### v0.2.1 - Moderator Persistence
- 💾 Moderator list persists across bot restarts
- 📂 Automatic data directory creation
- 🔄 Load moderators from data/moderators.json on startup
- ✅ Add/remove moderators saved immediately

### v0.2.0 - Conversational AI
- 🎯 Natural language understanding
- 💬 No more strict command syntax
- 😊 Friendly error messages (no technical codes!)
- 🤖 Smart intent detection for volunteers and queries
- 📱 Witty responses for unclear requests
- 🗓️ Understands dates like "Saturday", "tomorrow", "next week"

## Troubleshooting

1. **Check container status**: `docker compose ps`
2. **View logs**: `docker compose logs --tail 50`
3. **Test webhook**: See [DEPLOYMENT.md](./DEPLOYMENT.md#troubleshooting)
4. **Restart**: `docker compose restart`

## Examples of What Works Now

```
User: @PirateBot I've got snacks
Bot: ✅ John has been assigned to snacks for 2025-10-15!

User: @PirateBot when's the next game?
Bot: 🏴‍☠️ Next Game: Saturday Game
     Date: 2025-10-15
     Time: 10:00 AM
     Location: Field 3 (https://maps.google.com/...)
     ...

User: @PirateBot blah blah blah
Bot: 🏴‍☠️ Ahoy! I'm not quite sure what you're asking, but I'm here to help! 
     Try asking about the next game or volunteer to bring snacks! 🍪
```

## Team

Built for the Pirates baseball team! ⚾🏴‍☠️

---

**Status**: ✅ Fully operational with conversational AI
**Last Updated**: October 2025
