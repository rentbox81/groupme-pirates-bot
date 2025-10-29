# Bot Message Management

## Overview
Bot admins and moderators can now manage bot messages in the GroupMe chat to keep conversations tidy and clean up old or duplicate messages.

## Configuration

To enable message management, add these optional settings to your `.env` file:

```env
# Your personal GroupMe API access token
GROUPME_ACCESS_TOKEN=your_access_token_here

# The GroupMe group ID where the bot operates
GROUPME_GROUP_ID=your_group_id_here
```

### Getting Your Access Token

1. Go to https://dev.groupme.com/session/new
2. Log in with your GroupMe account
3. Your access token will be displayed at the top of the page
4. Copy it to your `.env` file

### Finding Your Group ID

1. Go to https://dev.groupme.com/groups
2. Find your group in the list
3. The group ID is shown in the group details
4. Alternatively, look in the GroupMe API responses when testing

**Note**: Message management features will be disabled if these credentials are not configured. The bot will notify you if you try to use these commands without proper configuration.

## Commands

All message management commands require admin or moderator privileges.

### List Bot Messages

View recent bot messages with their IDs:

```
@BotName list messages
@BotName list 10 messages
@BotName list bot messages
```

**Example Output:**
```
🏴‍☠️ Recent bot messages (last 5):

1. ID: 162812345678912 - ⚾ Next Game: Pirates vs Cardinals...
2. ID: 162812345678913 - ✅ John has been assigned to snacks...
3. ID: 162812345678914 - ⏰ Game Reminder! 24 hours until...
4. ID: 162812345678915 - ⚾ Game starting in 15 minutes! 🏴‍☠️...
5. ID: 162812345678916 - 🏴‍☠️ Pirates Commands:...

💡 To delete a message: @PirateBot delete message <id>
```

### Delete a Specific Message

Delete a single bot message by ID:

```
@BotName delete message 162812345678912
```

**Example Output:**
```
✅ Deleted message 162812345678912
```

**Notes:**
- Only bot messages can be deleted (user messages are not affected)
- Message ID must be provided (get it from `list messages`)
- The message is permanently deleted from the chat

### Clean Recent Bot Messages

Delete multiple recent bot messages at once:

```
@BotName clean messages
@BotName clean 5 messages
@BotName clean bot messages
```

**Example Output:**
```
🧹 Cleaned 5 bot message(s). 0 failed.
```

**Notes:**
- Default: Cleans last 5 bot messages
- Specify a number to clean more (e.g., "clean 10 messages")
- Only deletes bot messages, not user messages
- Failed deletions are counted and reported

## Use Cases

### Clean Up After Testing
When testing the bot or fixing configuration issues:
```
@BotName clean 10 messages
```

### Remove Duplicate Reminders
If reminders were sent multiple times due to a bug:
```
@BotName list messages
@BotName delete message [ID of duplicate]
```

### Clear Old Information
Remove outdated game information after schedule changes:
```
@BotName clean 3 messages
```

### Keep Chat Tidy
Regularly clean up bot messages to prevent chat clutter:
```
@BotName clean messages
```

## Permissions

Message management commands are restricted to:
- **Admin** (configured via `ADMIN_USER_ID` in `.env`)
- **Moderators** (added by admin via `@Bot add moderator @User`)

If an unauthorized user tries to use these commands:
```
🏴‍☠️ Only admins and moderators can list bot messages
```

## API Rate Limits

GroupMe has API rate limits. When cleaning many messages:
- The bot processes deletions sequentially
- Failed deletions are counted and reported
- Consider waiting between large cleanup operations

## Security Notes

1. **Access Token**: Keep your GroupMe access token secure. It provides access to your entire GroupMe account.
2. **Group ID**: Ensure the correct group ID is configured to prevent accidental deletions in other groups.
3. **Message Verification**: The bot verifies messages belong to it before deletion.
4. **Audit Trail**: Message deletions are logged for debugging.

## Troubleshooting

### "Message management is not configured"
- Add `GROUPME_ACCESS_TOKEN` and `GROUPME_GROUP_ID` to your `.env` file
- Restart the bot after updating configuration

### "Message not found or is not a bot message"
- The message may have already been deleted
- The message ID might be incorrect
- The message might belong to a user, not the bot

### Deletions failing
- Check API rate limits
- Verify your access token is valid
- Ensure the bot has appropriate permissions in the group

## Examples

### Complete Workflow

1. **Check recent messages:**
   ```
   @PirateBot list messages
   ```

2. **Review the list and identify unwanted messages**

3. **Delete specific messages:**
   ```
   @PirateBot delete message 162812345678912
   @PirateBot delete message 162812345678914
   ```

4. **Or bulk clean:**
   ```
   @PirateBot clean 5 messages
   ```

### Regular Maintenance

Weekly cleanup to keep chat organized:
```
@PirateBot clean 10 messages
```

## Related Commands

- `@Bot add moderator @User` - Grant moderator privileges
- `@Bot remove moderator @User` - Revoke moderator privileges  
- `@Bot list moderators` - View current moderators

## Future Enhancements

Potential improvements under consideration:
- Filter messages by date range
- Delete messages by content pattern
- Schedule automatic cleanup
- Export message history before deletion
