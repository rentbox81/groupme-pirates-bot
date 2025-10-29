# Team Customization Guide

## Overview

The GroupMe bot now supports **any team**, not just the Pirates! You can customize the team name, emoji, and facts to match your team's identity.

## Quick Setup

### 1. Set Your Team Name

```bash
TEAM_NAME=Dragons
```

The team name is used throughout the bot in:
- Help text and commands
- Team spirit messages
- Volunteer announcements

### 2. Choose Your Team Emoji

```bash
TEAM_EMOJI=🐉
```

The emoji appears in:
- Game announcements
- Volunteer status messages
- Admin/moderator responses
- Help commands

**Popular Emoji Examples:**
- 🏴‍☠️ Pirates
- 🐉 Dragons
- 🦅 Eagles
- 🐻 Cubs/Bears
- 🐛 Bugs/Insects
- ⚡ Lightning/Storm
- 🔥 Fire/Phoenix
- 🦁 Lions

### 3. Configure Team Facts

```bash
ENABLE_TEAM_FACTS=true
```

When enabled, users can trigger team facts with:
```
@YourBot lets go dragons
```

## Built-in MLB Team Facts

The bot includes curated facts for these Major League Baseball teams:

| Team | Emoji | Facts Count |
|------|-------|-------------|
| Pirates | 🏴‍☠️ | 10 facts |
| Yankees | 🗽 | 8 facts |
| Red Sox | 🧦 | 8 facts |
| Cubs | 🐻 | 8 facts |
| Dodgers | 💙 | 8 facts |
| Giants | 🧡 | 8 facts |
| Braves | 🪓 | 8 facts |

If your `TEAM_NAME` matches any of these (case-insensitive), the bot will automatically use the built-in facts.

## Custom Team Facts

For youth leagues, rec teams, or other custom teams, you can provide your own facts!

### 1. Create a facts JSON file

```json
{
  "team_name": "Dragons",
  "facts": [
    "🐉 The Dragons won their first championship in 2020!",
    "⚾ Our mascot Spike has been with us since 2015!",
    "🐉 The Dragons have the best team spirit in the league!",
    "⚾ We practice every Tuesday and Thursday!",
    "🐉 Our team colors are green and gold!",
    "⚾ The Dragons dugout cheer is legendary!",
    "🐉 We've never lost when the whole team shows up!",
    "⚾ Teamwork, dedication, and having fun - that's the Dragons way!"
  ]
}
```

### 2. Mount the file in Docker

In your `docker-compose.yml`:

```yaml
volumes:
  - ./my-team-facts.json:/app/data/team-facts.json:ro
```

### 3. Configure the path

```bash
TEAM_FACTS_FILE=/app/data/team-facts.json
```

## Configuration Examples

### Youth Baseball Team

```bash
TEAM_NAME=Tigers
TEAM_EMOJI=🐯
ENABLE_TEAM_FACTS=true
TEAM_FACTS_FILE=/app/data/tigers-facts.json
```

### Generic Team (No Facts)

```bash
TEAM_NAME=Eagles
TEAM_EMOJI=🦅
ENABLE_TEAM_FACTS=false
```

When facts are disabled, `@Bot lets go eagles` returns a simple encouraging message.

### MLB Team (Built-in Facts)

```bash
TEAM_NAME=Cubs
TEAM_EMOJI=🐻
ENABLE_TEAM_FACTS=true
# No TEAM_FACTS_FILE needed - uses built-in Cubs facts
```

## How It Works

### Priority Order

1. **Custom Facts File** - If `TEAM_FACTS_FILE` is set and valid, uses those facts
2. **Built-in Facts** - If team name matches a supported MLB team, uses built-in facts
3. **Generic Response** - Falls back to encouraging generic messages

### Example Bot Responses

**With Facts (Pirates):**
```
User: @PirateBot lets go pirates
Bot: 🏴‍☠️ The Pirates' 'We Are Family' team of 1979 came back from a 3-1 deficit to win the World Series!
```

**Without Facts (Generic Team):**
```
User: @DragonsBot lets go dragons
Bot: 🐉 Go Dragons! Let's bring the energy and win this game! ⚾
```

**Facts Disabled:**
```
User: @Bot lets go team
Bot: ⚾ Let's go team! ⚾
```

## Testing Your Configuration

1. Start the bot with your configuration
2. Check the startup logs for team configuration
3. Try the team spirit command: `@YourBot lets go [team]`
4. Check help text: `@YourBot help`
5. Verify emoji appears in game announcements

## Tips

- **Keep facts short** - Around 100-150 characters works well in GroupMe
- **Add emoji** - Makes messages more engaging
- **Mix types** - Combine historical facts, current info, and motivational messages
- **Test facts** - Make sure they display correctly in GroupMe before deployment
- **Update regularly** - Add new facts as your team achieves milestones!

## Troubleshooting

### Facts not loading

1. Check file path is correct in `TEAM_FACTS_FILE`
2. Verify JSON is valid (use jsonlint.com)
3. Ensure file is mounted in Docker container
4. Check logs for JSON parsing errors

### Wrong emoji displaying

- Make sure emoji is properly set in `.env`
- Some emojis may not display correctly in all environments
- Test emoji in GroupMe before deployment

### Team name not appearing

- Verify `TEAM_NAME` is set in `.env`
- Restart bot after changing configuration
- Check logs for configuration loading messages

## Migration from Pirates-Only Version

If upgrading from an older version:

1. Add these lines to your `.env`:
   ```bash
   TEAM_NAME=Pirates
   TEAM_EMOJI=🏴‍☠️
   ENABLE_TEAM_FACTS=true
   ```

2. Rebuild and restart:
   ```bash
   docker compose up -d --build
   ```

3. Everything should work as before, but now it's customizable!

## Contributing New Built-in Teams

Want to add facts for another MLB team or popular team? 

1. Edit `src/team_facts.rs`
2. Add a new match arm in `get_builtin_fact()`
3. Include 8-10 interesting facts
4. Submit a pull request!

## Questions?

See the main [README.md](./README.md) for general bot documentation or open an issue on GitHub.
