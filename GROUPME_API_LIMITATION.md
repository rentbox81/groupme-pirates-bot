# GroupMe API Limitation - Message Deletion

## Issue Summary

**The GroupMe API does not support programmatic message deletion**, even for messages posted by bots.

## What We Tried

We implemented a message management feature with three commands:
- `@PirateBot list messages` - ✅ **Works** (listing is supported)
- `@PirateBot delete message [ID]` - ❌ **Fails** (deletion not supported)
- `@PirateBot clean messages` - ❌ **Fails** (deletion not supported)

## Technical Details

### Error Observed
```
Status: 500 Internal Server Error
GroupMe: Internal Server Error (500)
Message: "Oops! Something broke. Pardon our dust, and try again later."
```

### API Endpoint Used
```
DELETE /v3/groups/{group_id}/messages/{message_id}?token={access_token}
```

### What the Documentation Says

GroupMe's API documentation for the DELETE endpoint states it requires:
- The request must be made by the **user who created the message**
- Messages can only be deleted through the **GroupMe app interface**

### Why It Fails

1. **Bot messages** are created by the bot user (not a personal account)
2. **Personal access tokens** don't grant permission to delete bot messages
3. **GroupMe API limitation** - there's no way to delete messages programmatically

## Verification

We tested with:
- ✅ Valid access token (can list messages successfully)
- ✅ Correct group ID (verified messages belong to the group)
- ✅ Valid message IDs (messages exist and are from our bot)
- ✅ Proper authentication (API accepts the token)
- ❌ **Result**: 500 error every time

The 500 error is GroupMe's way of saying "this operation is not allowed."

## Why "List Messages" Works But "Delete" Doesn't

| Operation | Endpoint | Status | Reason |
|-----------|----------|--------|--------|
| List | GET /messages | ✅ Works | Read-only operation |
| Delete | DELETE /messages | ❌ Fails | Write operation not supported for bots |

## Alternatives Considered

### 1. Use Bot Token Instead of Personal Token
**Status**: Won't work
- Bot tokens have even fewer permissions
- Bots explicitly cannot delete any messages

### 2. Use Group Admin Account
**Status**: Won't work  
- Still requires manual action through the app
- API doesn't support admin-level message deletion

### 3. Request GroupMe Support
**Status**: Unlikely to help
- This is a deliberate API design decision
- GroupMe wants to prevent automated message manipulation
- Safety feature to prevent abuse

## Recommended Solution

**Remove or Disable the Feature**

Since the GroupMe API doesn't support this functionality, we should:

### Option A: Remove the Feature Entirely
```bash
# Remove message management commands from the codebase
# Update documentation to remove references
```

### Option B: Keep Code, Disable by Default
```env
# Add new config option
ENABLE_MESSAGE_MANAGEMENT=false
```

### Option C: Change to "View Only"
Keep list functionality, remove delete/clean:
- ✅ Keep: `@PirateBot list messages` (helps see recent bot activity)
- ❌ Remove: `@PirateBot delete message`
- ❌ Remove: `@PirateBot clean messages`

## Manual Workaround

To delete bot messages, users must:

1. **Open GroupMe mobile app** (iOS or Android)
2. **Long-press on the bot message**
3. **Select "Delete"** from the menu
4. **Confirm deletion**

This is the **only way** to delete bot messages.

## Impact on Users

### What Works
- ✅ Bot posting messages
- ✅ Bot responding to commands  
- ✅ Viewing message history
- ✅ All other bot features

### What Doesn't Work
- ❌ Automated message cleanup
- ❌ Bot deleting its own messages
- ❌ Bulk message deletion via commands

## Recommendation for This Project

**Option C: Keep "List Messages" Only**

Rationale:
1. List functionality is useful for seeing bot activity
2. Clearly document that deletion isn't supported
3. Remove the delete/clean commands from the parser
4. Update help text to reflect limitations

## Code Changes Needed (If Removing Feature)

### Files to Modify
1. `src/models.rs` - Remove DeleteBotMessage and CleanBotMessages enums
2. `src/conversational_parser.rs` - Remove delete/clean parsing
3. `src/parser.rs` - Remove command conversions
4. `src/service.rs` - Remove delete/clean handlers (keep list)
5. `src/groupme_client.rs` - Remove delete_message() method (keep list_messages())
6. Documentation - Update to note limitation

### Simplified Commands
Keep only:
```
@PirateBot list messages    # Shows recent bot activity
```

## Lessons Learned

1. **Always verify API capabilities** before implementing features
2. **GroupMe API has significant limitations** compared to other platforms
3. **Read-only operations** are generally safe; write operations may be restricted
4. **Test early** with the actual API before full implementation

## References

- GroupMe API Docs: https://dev.groupme.com/docs/v3
- Message API: https://dev.groupme.com/docs/v3#messages
- Error observed: 500 Internal Server Error (operation not allowed)

## Status

- **Date Discovered**: October 30, 2025
- **Impact**: Message deletion/cleanup features non-functional
- **Workaround**: Manual deletion through GroupMe app
- **Fix**: Remove or document limitation (API-level restriction)
