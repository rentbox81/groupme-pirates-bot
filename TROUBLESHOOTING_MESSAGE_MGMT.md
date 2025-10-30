# Message Management Troubleshooting

## Issue: "Message management is not configured" error

### Symptoms
- Bot responds with: "🏴‍☠️ Message management is not configured. Set GROUPME_ACCESS_TOKEN and GROUPME_GROUP_ID in .env"
- Message management commands don't work

### Checklist

1. **Verify .env file has credentials**
   ```bash
   cd /path/to/groupme-pirates-bot
   grep -E "GROUPME_ACCESS_TOKEN|GROUPME_GROUP_ID" .env
   ```
   
   Should show:
   ```
   GROUPME_ACCESS_TOKEN=W5p4wlQsgRftPMW0dEzG293FZ7BTZ0fTKSHZ2dXu
   GROUPME_GROUP_ID=108695303
   ```

2. **Recreate container (not just restart)**
   ```bash
   docker compose down
   docker compose up -d
   ```
   Note: `docker compose restart` does NOT reload .env file!

3. **Check container environment variables**
   ```bash
   docker compose exec groupme-bot env | grep GROUPME
   ```
   
   Should show both variables set.

4. **Check logs for configuration loading**
   ```bash
   docker compose logs | grep -i "configuration loaded"
   ```

5. **Verify bot can reach GroupMe API**
   ```bash
   curl -s "https://api.groupme.com/v3/groups?token=YOUR_TOKEN" | head
   ```

### Common Issues

#### Issue 1: Container not recreated
**Problem**: Used `docker compose restart` instead of `down/up`
**Solution**: Always use `docker compose down && docker compose up -d` when changing .env

#### Issue 2: .env file permissions
**Problem**: Docker can't read .env file
**Solution**: 
```bash
chmod 644 .env
ls -la .env  # Should be readable
```

#### Issue 3: Wrong token or group ID
**Problem**: Credentials are incorrect or expired
**Solution**: Re-fetch from:
- Token: https://dev.groupme.com/session/new
- Group ID: Use API to list groups

#### Issue 4: Docker compose version
**Problem**: Old docker-compose doesn't load .env properly
**Solution**: Check version, should be v2+
```bash
docker compose version
```

## Getting Your Credentials

### Access Token
```bash
# Visit in browser and copy token
https://dev.groupme.com/session/new
```

### Group ID
```bash
# Use this command with your token
curl -s "https://api.groupme.com/v3/groups?token=YOUR_TOKEN" | \
  python3 -c "
import sys, json
data = json.load(sys.stdin)
for group in data.get('response', []):
    print(f'Group: {group[\"name\"]} | ID: {group[\"id\"]}')"
```

For your Pirates group:
- Group Name: 8U Pirates 2025 Fall
- Group ID: 108695303

## Testing Message Management

Once configured, test with:

```
@PirateBot list messages
```

Expected response:
```
🏴‍☠️ Recent bot messages (last 10):
1. ID: 162812345678912 - ⚾ Next Game: ...
...
💡 To delete a message: @PirateBot delete message <id>
```

If you still get "not configured" error:
1. Check container was recreated (not restarted)
2. Verify environment variables in container
3. Check logs for any errors
4. Ensure .env file is in the correct directory

## Docker Compose Environment Variable Loading

### Correct (loads .env):
```bash
docker compose down
docker compose up -d
```

### Incorrect (doesn't load new .env values):
```bash
docker compose restart  # ❌ Won't reload .env
```

## Next Steps

If still not working after following this guide:
1. Check bot logs: `docker compose logs --tail=50`
2. Verify webhook is working (try `@PirateBot help`)
3. Check your user ID matches ADMIN_USER_ID in .env
4. Verify you're a moderator if not admin

## Reference

- [MESSAGE_MANAGEMENT.md](./MESSAGE_MANAGEMENT.md) - Complete documentation
- [MESSAGE_MANAGEMENT_QUICK_REF.md](./MESSAGE_MANAGEMENT_QUICK_REF.md) - Quick reference
- [.env.template](./.env.template) - Configuration template
