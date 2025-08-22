# GroupMe Calendar Bot

A Rust-based GroupMe bot that correlates Google Calendar events with Google Sheets data to provide event information to group members.

## Multi-Team Support

This bot is designed to support multiple teams using configurable domains:
- **Domain Pattern**: `{TEAM_NAME}bot.{BASE_DOMAIN}`
- **Examples**: 
  - `piratesbot.rentbox.us` (Pirates team)
  - `dragonsbot.rentbox.us` (Dragons team)
  - `eaglesbot.myteam.com` (Eagles team)

### Quick Team Setup

Use the setup script to configure a new team:
```bash
./setup-team.sh pirates rentbox.us
# Creates configuration for https://piratesbot.rentbox.us
```

## Features

- 🤖 **GroupMe Integration**: Responds to mentions in GroupMe chat
- 📅 **Public Calendar Support**: Fetches upcoming events from publicly accessible webcal URLs
- 📊 **Google Sheets**: Augments calendar events with additional data (location, snacks, etc.)
- 🔍 **Smart Commands**: Supports date queries and "next event" requests
- ⚾ **Team Filtering**: Automatically filters events to show only Pirates team events (excludes Dragonflies)
- 📝 **Structured Logging**: Uses tracing for comprehensive logging
- 🐳 **Docker Support**: Easy deployment with Docker containers
- 🛡️ **Security**: Secure credential management and input validation

## Architecture

The bot is built with a modular architecture:

- **`config.rs`**: Environment configuration and validation
- **`error.rs`**: Custom error types and handling
- **`models.rs`**: Data structures for events, API responses, and commands
- **`google_client.rs`**: Google Calendar and Sheets API clients
- **`groupme_client.rs`**: GroupMe API client
- **`service.rs`**: Business logic and data correlation
- **`parser.rs`**: Command parsing and validation
- **`main.rs`**: HTTP server and webhook handler

## Setup

### Prerequisites

1. **Rust**: Install Rust 1.78.0 or later
2. **GroupMe Bot**: Create a GroupMe bot and get the bot ID
3. **Google API**: Get Google API key for Sheets API access
4. **Google Sheet**: Create a sheet with event data (see format below)
5. **Public Calendar**: Have a publicly accessible calendar with a webcal:// URL

#### Getting Your Calendar's Webcal URL

**For Google Calendar:**
1. Open your Google Calendar
2. Click the settings gear → Settings
3. Select your calendar from the left sidebar
4. Scroll to "Integrate calendar" section
5. Copy the "Public address in iCal format" URL
6. Replace `https://` with `webcal://` in the URL

**For other calendar services:**
Most calendar services provide public iCal/.ics URLs that can be used by replacing `https://` with `webcal://`.

#### For Production Deployment with Traefik

6. **Traefik**: Running Traefik instance with:
   - External network named `traefik`
   - Entrypoints: `web` (port 80) and `websecure` (port 443)
   - Certificate resolver configured (e.g., Let's Encrypt)
   - Docker provider enabled

### Environment Variables

Copy `.env.template` to `.env` and fill in your credentials:

```bash
cp .env.template .env
```

Required variables:
- `GROUPME_BOT_ID`: Your GroupMe bot ID
- `GROUPME_BOT_NAME`: Your bot's name (for mentions)
- `GOOGLE_API_KEY`: Your Google API key (only needed for Sheets access)
- `SHEET_ID`: Your Google Sheet ID
- `CALENDAR_WEBCAL_URL`: Your public calendar's webcal:// URL

Domain configuration (for production deployment):
- `TEAM_NAME`: Team name (lowercase, used for subdomain)
- `BASE_DOMAIN`: Your registered domain
- `CERT_RESOLVER`: Certificate resolver name (default: letsencrypt)
- `TRAEFIK_NETWORK`: Traefik network name (default: traefik)

Optional variables:
- `PORT`: Server port (default: 18080)
- `RUST_LOG`: Log level (default: info)

### Domain Configuration Examples

| Team Name | BASE_DOMAIN | Result URL |
|-----------|-------------|------------|
| pirates | rentbox.us | `https://piratesbot.rentbox.us` |
| dragons | myteam.com | `https://dragonsbot.myteam.com` |
| eagles | sportsbot.net | `https://eaglesbot.sportsbot.net` |

### Google Sheet Format

Your Google Sheet should have columns in this order (starting from row 2):

| A: Date | B: Title | C: Location | D: Snacks | E: Livestream | F: Scoreboard | G: Pitch Count |
|---------|----------|-------------|-----------|---------------|---------------|----------------|
| 2023-12-25 | Game vs Team | Stadium | John | Sarah | Mike | Lisa |

### Installation

#### Local Development

1. Clone the repository
2. Set up environment variables
3. Run the bot:

```bash
cargo run
```

#### Docker

1. Build the Docker image:

```bash
docker build -t groupme-bot .
```

2. Run with environment file:

```bash
docker run --env-file .env -p 18080:18080 groupme-bot
```

#### Docker Compose

**For local development:**

```bash
docker-compose up -d
```

**For production with Traefik:**

The project includes Traefik configuration for automatic HTTPS and reverse proxy setup:

```bash
# Edit your .env file to set DOMAIN and CERT_RESOLVER
echo "DOMAIN=your-bot.yourdomain.com" >> .env
echo "CERT_RESOLVER=letsencrypt" >> .env

# Deploy with production configuration
docker-compose -f docker-compose.prod.yml up -d
```

The production configuration includes:
- Automatic HTTPS with Let's Encrypt
- HTTP to HTTPS redirects
- Security headers
- Rate limiting
- Resource constraints

### GroupMe Webhook Setup

1. Set your GroupMe bot's callback URL to: `https://your-domain.com/webhook`
2. The bot will respond to webhook events at this endpoint

## Usage

Once deployed, group members can interact with the bot using these commands:

### Commands

- `@BotName` or `@BotName help` - Show help message
- `@BotName next` - Get information about the next upcoming event
- `@BotName next [field]` - Get specific field for the next event
- `@BotName YYYY-MM-DD` - Get information for a specific date
- `@BotName YYYY-MM-DD [field]` - Get specific field for a specific date

### Available Fields

- `location` - Event location
- `snacks` - Who's bringing snacks
- `livestream` - Parent assigned to livestream
- `scoreboard` - Parent assigned to scoreboard
- `pitchcount` - Parent assigned to pitch count

### Examples

```
@Pirate Bot next
→ Event: Game vs Cardinals
→ Date: 2023-12-25
→ Location: Busch Stadium
→ Snacks: John
→ Livestream: Sarah
→ Scoreboard: Mike
→ Pitch Count: Lisa

@Pirate Bot next location
→ location: Busch Stadium

@Pirate Bot 2023-12-25 livestream
→ livestream: Sarah
```

## API Endpoints

- `GET /` - Health check endpoint
- `POST /webhook` - GroupMe webhook endpoint

## Security Considerations

⚠️ **Important**: Never commit sensitive credentials to version control!

- Use environment variables for all secrets
- Regenerate any exposed API keys
- The `.gitignore` file excludes common secret files
- Use `.env.template` as a reference for required variables

## Troubleshooting

### Common Issues

1. **Bot not responding**: Check that the webhook URL is correctly configured in GroupMe
2. **Google API errors**: Verify API key and ensure Calendar/Sheets APIs are enabled
3. **Date parsing errors**: Ensure dates in Google Sheet follow YYYY-MM-DD format
4. **Sheet correlation issues**: Check that event titles match between Calendar and Sheet

### Logging

Set `RUST_LOG=debug` to see detailed logs:

```bash
RUST_LOG=debug cargo run
```

### Health Check

Check if the bot is running:

```bash
curl http://localhost:18080/
```

## Testing

### CLI Testing

Before deploying to GroupMe, you can test the bot locally using the CLI testing tools:

**Mock Mode (No API credentials needed):**
```bash
cargo run --bin test-bot-mock
```

This uses mock data and doesn't require real Google API credentials. Perfect for testing command parsing and response formatting.

**Live Mode (Requires full .env configuration):**
```bash
cargo run --bin test-bot
```

This connects to your actual Google Calendar and Sheets APIs to test the full functionality.

### Example CLI Test Session

```
🤖 GroupMe Bot CLI Tester (Mock Mode)
Bot Name: TestBot
This mode uses mock data and doesn't require real API credentials.
Enter messages as if you're typing in GroupMe chat.
Type 'quit' or 'exit' to stop testing.

💡 Example commands to try:
  @TestBot help
  @TestBot next
  @TestBot next location
  @TestBot 2024-01-15
  @TestBot 2024-01-15 snacks

Enter message: @TestBot help
📝 Parsed command: Help
🤖 Bot Response:
─────────────────
Hello! I can tell you about upcoming events. Try these commands:\n• @TestBot next - Get info about the next event\n...
─────────────────

Enter message: quit
Goodbye! 👋
```

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run tests and linting
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
