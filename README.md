# GroupMe Sports Team Bot ⚾

A conversational GroupMe bot designed for sports team management. It handles scheduling, volunteer assignments (snacks, scoreboard, etc.), and provides weather forecasts—all powered by Google Sheets and a natural language interface.

## 🌟 Features

- **📅 Automated Schedule**: Reads game schedule directly from a Google Sheet.
- **🗣️ Conversational Interface**: Talk naturally (e.g., "Who's on snacks?", "Where's the next game?").
- **🌤️ Weather Integration**: Automatic weather forecasts for game times and locations using Open-Meteo.
- **🤝 Volunteer Management**: Tracks volunteers for Snacks, Livestream, Scoreboard, and Pitch Count.
  - *Smart Logic*: "Scoreboard" is only requested for Away games!
- **🏴‍☠️ Team Customization**: Configurable team name, emoji, and "hype" facts.
- **🔐 Role-Based Access**: Admin and Moderator privileges for sensitive commands.

---

## 🚀 Deployment Guide (Step-by-Step)

Follow these steps to get your bot up and running.

### 1. Prerequisites

1.  **GroupMe Bot**:
    - Go to [dev.groupme.com](https://dev.groupme.com/bots).
    - Log in and create a new bot.
    - Select the group you want the bot to live in.
    - **Note the Bot ID** (you'll need this).

2.  **Google Cloud Service Account** (for Sheets access):
    - Go to [Google Cloud Console](https://console.cloud.google.com/).
    - Create a new project (or select existing).
    - Enable the **Google Sheets API**.
    - Go to **IAM & Admin > Service Accounts**.
    - Create a Service Account.
    - Create a **JSON Key** for this account and download it.
    - Rename the file to `service-account.json`.

3.  **Google Sheet Setup**:
    - Create a Google Sheet with the following columns (Order matters!):
      - **A**: Date (YYYY-MM-DD)
      - **B**: Time (e.g., 10:00 AM)
      - **C**: Location (e.g., Field 1)
      - **D**: Home Team (Used to detect Home/Away)
      - **E**: Snacks (Volunteer Name)
      - **F**: Livestream (Volunteer Name)
      - **G**: Scoreboard (Volunteer Name)
      - **H**: Pitch Count (Volunteer Name)
    - **Share** the sheet with the *Service Account Email* (found in your `service-account.json`) giving it **Editor** access.
    - **Note the Sheet ID** from the URL (e.g., `https://docs.google.com/spreadsheets/d/THIS_PART_IS_THE_ID/edit`).

### 2. Installation

Clone the repository to your server or local machine:

```bash
git clone <your-repo-url>
cd groupme-pirates-bot
```

### 3. Configuration

1.  **Service Account**:
    Move your downloaded `service-account.json` into the project root directory.
    ```bash
    mv /path/to/downloaded/key.json ./service-account.json
    ```

2.  **Environment Variables**:
    Copy the template and edit it:
    ```bash
    cp .env.template .env
    nano .env
    ```

    **Critical Settings to Change:**
    - `GROUPME_BOT_ID`: Your Bot ID from Step 1.
    - `GROUPME_BOT_NAME`: Name users will use to address the bot (e.g., "PirateBot").
    - `SHEET_ID`: Your Google Sheet ID from Step 1.
    - `GOOGLE_API_KEY`: API Key (optional if using Service Account, but recommended as backup).
    - `ADMIN_USER_ID`: Your GroupMe User ID (visit `https://api.groupme.com/v3/users/me` with an access token to find this, or check logs after sending a message).
    - `TEAM_NAME`: Your team name (e.g., "Pirates").
    - `TEAM_EMOJI`: Emoji to use in messages (e.g., "🏴‍☠️").

### 4. Build and Run

Run with Docker Compose (Recommended):

```bash
docker compose up -d --build
```

### 5. Verify

Check the logs to ensure everything started correctly:

```bash
docker compose logs -f
```

You should see "Configuration loaded successfully" and "Starting GroupMe bot...".

Test it in your GroupMe group:
> "@PirateBot next game"

### 6. External Access (Traefik)

The `docker-compose.yml` is pre-configured for Traefik. To expose the bot externally (e.g., via a reverse proxy):

1.  **Ensure Traefik is running** on your host and connected to the `traefik` Docker network.
2.  **Configure `.env`**:
    - `BOT_SUBDOMAIN`: e.g., `pirates` (results in `piratesbot.example.com`).
    - `BASE_DOMAIN`: e.g., `example.com`.
    - `CERT_RESOLVER`: e.g., `letsencrypt`.
3.  **Deploy**:
    ```bash
    docker compose up -d
    ```

**Example Traefik Configuration:**
If you have Traefik set up on your host (e.g., 192.168.1.221), the bot will automatically register itself using these labels in `docker-compose.yml`:

```yaml
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.piratesbot.rule=Host(`piratesbot.example.com`)"
  - "traefik.http.routers.piratesbot.entrypoints=websecure"
  - "traefik.http.routers.piratesbot.tls.certresolver=letsencrypt"
```

The bot expects the `traefik` network to exist external to this compose stack:
```bash
docker network create traefik
```

### 7. Configure GroupMe Callback URL

**Crucial Step**: GroupMe needs to know where to send messages.

1.  Go back to [dev.groupme.com/bots](https://dev.groupme.com/bots).
2.  Edit your bot.
3.  Set the **Callback URL** to your deployed endpoint:
    
    ```
    https://<BOT_SUBDOMAIN>bot.<BASE_DOMAIN>/webhook
    ```
    
    *Example:* `https://piratesbot.example.com/webhook`

    > **Note**: If you are testing locally (without Traefik/SSL), you will need to use a tool like **ngrok** to create a public HTTPS URL and use that instead.

---

## 🛠️ Usage

### 🗣️ Natural Language Commands
Users can ask questions naturally. The bot uses fuzzy matching to understand intent.

- **Game Info**:
  - "When is the next game?"
  - "Where are we playing?"
  - "What's the weather look like?"
  - "Show me the next 3 games"

- **Volunteering**:
  - "I can do snacks"
  - "Put me down for scoreboard"
  - "I'll do livestream for Saturday"
  - "Who is doing pitch count?"
  - "Do we need volunteers?"

- **Team Spirit**:
  - "Let's go Pirates!" (Responds with a team fact or hype message)

### 👮 Admin & Moderator Commands
Requires the user to be the Admin (set in `.env`) or a listed Moderator.

- **Manage Moderators**:
  - "@PirateBot add moderator @JohnDoe"
  - "@PirateBot remove moderator @JohnDoe"
  - "@PirateBot list moderators"

- **Manage Volunteers (Force Assign/Remove)**:
  - "@PirateBot assign @Jane to snacks"
  - "@PirateBot remove @Jane from livestream"

---

## ⚙️ Advanced Customization

### Team Facts
You can customize the "hype" facts for your team.
- **Built-in**: Facts for Pirates, Yankees, Red Sox, Cubs, Dodgers, Giants, Braves.
- **Custom**: Create a `data/team-facts.json` file and mount it, or just use the generic fallback.
  - Set `ENABLE_TEAM_FACTS=true` in `.env`.

### Weather
Weather data is sourced from [Open-Meteo](https://open-meteo.com/).
- No API key required.
- Automatically geocodes the "Location" field from your schedule.
- Provides temperature, condition, and precipitation chance.

### Home/Away Logic
The bot determines if a game is **Home** or **Away** to decide if a "Scoreboard" volunteer is needed.
- It checks the **Home Team** column (Column D) in your Google Sheet.
- If the cell contains "Home" or "H", or matches your `TEAM_NAME`, it's a **Home Game**.
- **Home Games**: Scoreboard volunteer is marked as "Not Needed".
- **Away Games**: Scoreboard volunteer is marked as "⚠️ NEEDED".

---

## 👩‍💻 Development

To run locally without Docker:

```bash
# Install dependencies (Rust)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run
cargo run
```

To run tests:
```bash
cargo test
```
