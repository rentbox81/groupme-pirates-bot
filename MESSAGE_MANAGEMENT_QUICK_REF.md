# Message Management - Quick Reference

## Setup (One-time)

1. Get your access token: https://dev.groupme.com/session/new
2. Get your group ID: https://dev.groupme.com/groups
3. Add to `.env`:
   ```
   GROUPME_ACCESS_TOKEN=your_token_here
   GROUPME_GROUP_ID=your_group_id_here
   ```
4. Restart the bot

## Commands

### List Messages
```
@BotName list messages          # List last 10 messages
@BotName list 20 messages       # List last 20 messages
```

### Delete Message
```
@BotName delete message 162812345678912
```

### Clean Messages
```
@BotName clean messages         # Delete last 5 messages
@BotName clean 10 messages      # Delete last 10 messages
```

## Permissions

- ✅ Admin (configured in ADMIN_USER_ID)
- ✅ Moderators (added via `@Bot add moderator @User`)
- ❌ Regular users

## Common Workflows

### Clean Up After Testing
```
@BotName clean 10 messages
```

### Remove One Wrong Message
```
@BotName list messages
@BotName delete message [ID from list]
```

### Weekly Maintenance
```
@BotName clean 15 messages
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "Message management is not configured" | Add GROUPME_ACCESS_TOKEN and GROUPME_GROUP_ID to .env |
| "Only admins and moderators can..." | Check your user ID matches ADMIN_USER_ID or you're a moderator |
| "Message not found or is not a bot message" | Message may be deleted, or ID is wrong |
| Deletions failing | Check API rate limits, verify token is valid |

## Notes

- Only bot messages are deleted (user messages are never touched)
- Deletions are permanent
- Failed deletions are counted and reported
- All operations are logged for debugging

## Examples

```
User: @PirateBot list messages
Bot:  🏴‍☠️ Recent bot messages (last 10):
      1. ID: 162812345678912 - ⚾ Next Game: Pirates vs Cardinals...
      2. ID: 162812345678913 - ✅ John has been assigned to snacks...
      ...
      💡 To delete a message: @PirateBot delete message <id>

User: @PirateBot delete message 162812345678912
Bot:  ✅ Deleted message 162812345678912

User: @PirateBot clean 5 messages
Bot:  🧹 Cleaned 5 bot message(s). 0 failed.
```

## Related Commands

- `@Bot add moderator @User` - Add moderator
- `@Bot remove moderator @User` - Remove moderator
- `@Bot list moderators` - Show all moderators

---

For full documentation, see [MESSAGE_MANAGEMENT.md](./MESSAGE_MANAGEMENT.md)
