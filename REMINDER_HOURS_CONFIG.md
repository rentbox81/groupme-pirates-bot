# Configurable Reminder Time Window - October 7, 2025

## Feature
Added configurable time-of-day restrictions for reminders to prevent them from being sent too early in the morning or too late at night.

## Configuration
Two new environment variables control the reminder window:

```bash
# In .env file:
REMINDER_START_HOUR=9   # Earliest hour to send reminders (9am)
REMINDER_END_HOUR=21    # Latest hour to send reminders (9pm)
```

### Default Values
- **Start Hour**: 9 (9:00 AM)
- **End Hour**: 21 (9:00 PM)
- If not specified, these defaults are used automatically

### Time Format
- Uses 24-hour format (0-23)
- Examples:
  - `6` = 6:00 AM
  - `9` = 9:00 AM
  - `12` = 12:00 PM (noon)
  - `21` = 9:00 PM
  - `23` = 11:00 PM

## How It Works

1. **Every 5 minutes**, the reminder scheduler checks for games that need reminders
2. **Before sending any reminder**, it checks if the current hour falls within the configured window
3. **If outside the window** (e.g., 3am or 10pm), reminders are silently skipped
4. **If inside the window** (e.g., 11am or 6pm), reminders are sent normally

## Example Scenarios

### Default Configuration (9am - 9pm)
- ✅ 9:00 AM - Reminders will send
- ✅ 11:00 AM - Reminders will send (24-hour reminder sent at this time)
- ✅ 5:45 PM - Reminders will send
- ✅ 8:59 PM - Reminders will send
- ❌ 3:00 AM - Reminders will NOT send (too early)
- ❌ 9:00 PM - Reminders will NOT send (equals end hour, must be before)
- ❌ 10:30 PM - Reminders will NOT send (too late)

### Custom Configuration (7am - 11pm)
```bash
REMINDER_START_HOUR=7
REMINDER_END_HOUR=23
```
- ✅ 7:00 AM - Reminders will send
- ✅ 10:30 PM - Reminders will send
- ❌ 6:59 AM - Reminders will NOT send
- ❌ 11:00 PM - Reminders will NOT send

## Validation
The config includes validation:
- Start hour must be between 0 and 23
- End hour must be between 1 and 24
- Start hour must be less than end hour
- Invalid configurations will cause the bot to fail at startup with a clear error message

## Log Output
When the scheduler starts, it logs the configured hours:
```
INFO groupme_bot::reminder: Reminder scheduler started (active hours: 9:00 - 21:00)
```

When a reminder is sent, it includes the current hour:
```
INFO groupme_bot::reminder: Sending 24-hour reminder for game on 2025-10-08 (current hour: 11)
```

## Files Changed
- `src/config.rs` - Added `reminder_start_hour` and `reminder_end_hour` fields with validation
- `src/reminder.rs` - Added `is_within_reminder_hours()` check before sending reminders
- `.env.template` - Documented the new environment variables
- `.env` - Set production values to 9am-9pm

## Use Cases
This feature is useful for:
- **Parent-friendly timing**: No notifications waking up families at 6am
- **Respectful boundaries**: No late-night pings at 11pm
- **Timezone adjustments**: If bot is deployed in different timezone than users
- **Custom preferences**: Each team can set their own preferred hours

## Related
- Commit: `b24782f` - Add configurable reminder time window (9am-9pm default)
